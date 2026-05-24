pub mod turrets;

use std::collections::VecDeque;
use std::io::{self, Read, Write};

use crate::mindustry::core::content_loader::ContentLoader;
use crate::mindustry::ctype::ContentId;
use crate::mindustry::entities::comp::UnitComp;
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::game::BlockPlan;
use crate::mindustry::io::{type_io, TeamId, TypeValue};
use crate::mindustry::r#type::UnitType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallState {
    pub hit: f32,
}

pub fn wall_collision_hit(_previous_hit: f32) -> f32 {
    1.0
}

pub fn wall_draw_hit_decay(hit: f32, delta: f32, paused: bool) -> f32 {
    if paused {
        hit
    } else {
        (hit - delta / 10.0).clamp(0.0, 1.0)
    }
}

pub fn wall_should_lightning(lightning_chance: f32, random: f32) -> bool {
    lightning_chance > 0.0 && random < lightning_chance
}

pub fn wall_deflects_bullet(
    chance_deflect: f32,
    bullet_speed: f32,
    reflectable: bool,
    bullet_damage: f32,
    random: f32,
) -> bool {
    chance_deflect > 0.0
        && bullet_speed > 0.1
        && reflectable
        && bullet_damage > 0.0
        && random < chance_deflect / bullet_damage
}

pub fn wall_reflect_x(pen_x: f32, pen_y: f32) -> bool {
    pen_x > pen_y
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorState {
    pub open: bool,
}

pub fn door_check_solid(open: bool) -> bool {
    !open
}

pub fn door_sense_enabled(open: bool) -> f64 {
    if open {
        1.0
    } else {
        0.0
    }
}

pub fn door_can_toggle(
    open: bool,
    requested_open: bool,
    units_in_tile: bool,
    origin_timer_ready: bool,
) -> bool {
    open != requested_open && (!units_in_tile || requested_open) && origin_timer_ready
}

pub fn door_tapped_should_configure(
    open: bool,
    units_in_tile: bool,
    origin_timer_ready: bool,
) -> bool {
    !(units_in_tile && open) && origin_timer_ready
}

pub fn write_door_state<W: Write>(write: &mut W, state: DoorState) -> io::Result<()> {
    write.write_all(&[state.open as u8])
}

pub fn read_door_state<R: Read>(read: &mut R) -> io::Result<DoorState> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(DoorState { open: buf[0] != 0 })
}

pub fn auto_door_should_open(ground_units_in_trigger: bool) -> bool {
    ground_units_in_trigger
}

pub fn auto_door_trigger_size(block_size: i32, tile_size: f32, trigger_margin: f32) -> f32 {
    block_size as f32 * tile_size + trigger_margin * 2.0
}

pub fn shock_mine_should_trigger(enabled: bool, same_team: bool, timer_ready: bool) -> bool {
    enabled && !same_team && timer_ready
}

pub fn shock_mine_bullet_angles(shots: i32, inaccuracy_offsets: &[f32]) -> Vec<f32> {
    if shots <= 0 {
        return Vec::new();
    }
    (0..shots)
        .map(|index| {
            (360.0 / shots as f32) * index as f32
                + inaccuracy_offsets
                    .get(index as usize)
                    .copied()
                    .unwrap_or(0.0)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MendProjectorState {
    pub heat: f32,
    pub charge: f32,
    pub phase_heat: f32,
    pub smooth_efficiency: f32,
}

impl Default for MendProjectorState {
    fn default() -> Self {
        Self {
            heat: 0.0,
            charge: 0.0,
            phase_heat: 0.0,
            smooth_efficiency: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MendProjectorUpdate {
    pub fired: bool,
    pub real_range: f32,
    pub heal_fraction: f32,
    pub should_consume_optional: bool,
}

pub fn mend_projector_update(
    state: &mut MendProjectorState,
    efficiency: f32,
    optional_efficiency: f32,
    can_heal: bool,
    delta: f32,
    reload: f32,
    range: f32,
    heal_percent: f32,
    phase_boost: f32,
    phase_range_boost: f32,
) -> MendProjectorUpdate {
    state.smooth_efficiency = lerp_delta(state.smooth_efficiency, efficiency, 0.08);
    state.heat = lerp_delta(
        state.heat,
        if efficiency > 0.0 && can_heal {
            1.0
        } else {
            0.0
        },
        0.08,
    );
    state.charge += state.heat * delta;
    state.phase_heat = lerp_delta(state.phase_heat, optional_efficiency, 0.1);
    let real_range = range + state.phase_heat * phase_range_boost;
    let heal_fraction = (heal_percent + state.phase_heat * phase_boost) / 100.0 * efficiency;
    let fired = state.charge >= reload && can_heal;
    if fired {
        state.charge = 0.0;
    }
    MendProjectorUpdate {
        fired,
        real_range,
        heal_fraction,
        should_consume_optional: optional_efficiency > 0.0 && can_heal,
    }
}

pub fn write_mend_projector_state<W: Write>(
    write: &mut W,
    state: &MendProjectorState,
) -> io::Result<()> {
    write_f32(write, state.heat)?;
    write_f32(write, state.phase_heat)
}

pub fn read_mend_projector_state<R: Read>(read: &mut R) -> io::Result<MendProjectorState> {
    Ok(MendProjectorState {
        heat: read_f32(read)?,
        phase_heat: read_f32(read)?,
        ..MendProjectorState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverdriveProjectorState {
    pub heat: f32,
    pub charge: f32,
    pub phase_heat: f32,
    pub smooth_efficiency: f32,
    pub use_progress: f32,
}

impl Default for OverdriveProjectorState {
    fn default() -> Self {
        Self {
            heat: 0.0,
            charge: 0.0,
            phase_heat: 0.0,
            smooth_efficiency: 0.0,
            use_progress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverdriveProjectorUpdate {
    pub applied_boost: bool,
    pub consumed: bool,
    pub real_range: f32,
    pub real_boost: f32,
}

pub fn overdrive_real_boost(
    speed_boost: f32,
    phase_heat: f32,
    speed_boost_phase: f32,
    efficiency: f32,
) -> f32 {
    (speed_boost + phase_heat * speed_boost_phase) * efficiency
}

#[allow(clippy::too_many_arguments)]
pub fn overdrive_projector_update(
    state: &mut OverdriveProjectorState,
    efficiency: f32,
    optional_efficiency: f32,
    has_boost: bool,
    delta: f32,
    reload: f32,
    range: f32,
    phase_range_boost: f32,
    speed_boost: f32,
    speed_boost_phase: f32,
    use_time: f32,
) -> OverdriveProjectorUpdate {
    state.smooth_efficiency = lerp_delta(state.smooth_efficiency, efficiency, 0.08);
    state.heat = lerp_delta(state.heat, if efficiency > 0.0 { 1.0 } else { 0.0 }, 0.08);
    state.charge += state.heat * delta;
    if has_boost {
        state.phase_heat = lerp_delta(state.phase_heat, optional_efficiency, 0.1);
    }
    let applied_boost = state.charge >= reload;
    if applied_boost {
        state.charge = 0.0;
    }
    if efficiency > 0.0 {
        state.use_progress += delta;
    }
    let consumed = state.use_progress >= use_time;
    if consumed {
        state.use_progress %= use_time;
    }
    OverdriveProjectorUpdate {
        applied_boost,
        consumed,
        real_range: range + state.phase_heat * phase_range_boost,
        real_boost: overdrive_real_boost(
            speed_boost,
            state.phase_heat,
            speed_boost_phase,
            efficiency,
        ),
    }
}

pub fn write_overdrive_projector_state<W: Write>(
    write: &mut W,
    state: &OverdriveProjectorState,
) -> io::Result<()> {
    write_f32(write, state.heat)?;
    write_f32(write, state.phase_heat)
}

pub fn read_overdrive_projector_state<R: Read>(
    read: &mut R,
) -> io::Result<OverdriveProjectorState> {
    Ok(OverdriveProjectorState {
        heat: read_f32(read)?,
        phase_heat: read_f32(read)?,
        ..OverdriveProjectorState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceProjectorState {
    pub broken: bool,
    pub buildup: f32,
    pub radscl: f32,
    pub hit: f32,
    pub warmup: f32,
    pub phase_heat: f32,
}

impl Default for ForceProjectorState {
    fn default() -> Self {
        Self {
            broken: true,
            buildup: 0.0,
            radscl: 0.0,
            hit: 0.0,
            warmup: 0.0,
            phase_heat: 0.0,
        }
    }
}

pub fn force_projector_real_radius(
    radius: f32,
    phase_heat: f32,
    phase_radius_boost: f32,
    radscl: f32,
) -> f32 {
    (radius + phase_heat * phase_radius_boost) * radscl
}

pub fn force_projector_shield(
    broken: bool,
    shield_health: f32,
    phase_shield_boost: f32,
    phase_heat: f32,
    buildup: f32,
) -> f32 {
    if broken {
        0.0
    } else {
        (shield_health + phase_shield_boost * phase_heat - buildup).max(0.0)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn force_projector_update(
    state: &mut ForceProjectorState,
    efficiency: f32,
    phase_valid: bool,
    coolant_efficiency: f32,
    coolant_heat_capacity: f32,
    delta: f32,
    shield_health: f32,
    phase_shield_boost: f32,
    cooldown_normal: f32,
    cooldown_broken_base: f32,
    cooldown_liquid: f32,
) -> bool {
    state.phase_heat = lerp_delta(state.phase_heat, if phase_valid { 1.0 } else { 0.0 }, 0.1);
    state.radscl = lerp_delta(
        state.radscl,
        if state.broken { 0.0 } else { state.warmup },
        0.05,
    );
    state.warmup = lerp_delta(state.warmup, efficiency, 0.1);

    if state.buildup > 0.0 {
        let mut scale = if !state.broken {
            cooldown_normal
        } else {
            cooldown_broken_base
        };
        if coolant_efficiency > 0.0 {
            scale *= cooldown_liquid * (1.0 + (coolant_heat_capacity - 0.4) * 0.9);
        }
        state.buildup -= delta * scale;
    }
    if state.broken && state.buildup <= 0.0 {
        state.broken = false;
    }

    let broke_now =
        state.buildup >= shield_health + phase_shield_boost * state.phase_heat && !state.broken;
    if broke_now {
        state.broken = true;
        state.buildup = shield_health;
    }
    if state.hit > 0.0 {
        state.hit -= 1.0 / 5.0 * delta;
    }
    broke_now
}

pub fn force_projector_absorb_explosion(
    state: &mut ForceProjectorState,
    inside_polygon: bool,
    damage: f32,
    crash_damage_multiplier: f32,
) -> bool {
    let absorb = !state.broken && inside_polygon;
    if absorb {
        state.hit = 1.0;
        state.buildup += damage * crash_damage_multiplier;
    }
    absorb
}

pub fn write_force_projector_state<W: Write>(
    write: &mut W,
    state: &ForceProjectorState,
) -> io::Result<()> {
    write.write_all(&[state.broken as u8])?;
    write_f32(write, state.buildup)?;
    write_f32(write, state.radscl)?;
    write_f32(write, state.warmup)?;
    write_f32(write, state.phase_heat)
}

pub fn read_force_projector_state<R: Read>(read: &mut R) -> io::Result<ForceProjectorState> {
    let mut broken = [0; 1];
    read.read_exact(&mut broken)?;
    Ok(ForceProjectorState {
        broken: broken[0] != 0,
        buildup: read_f32(read)?,
        radscl: read_f32(read)?,
        warmup: read_f32(read)?,
        phase_heat: read_f32(read)?,
        hit: 0.0,
    })
}

pub fn regen_projector_heal_amount(
    optional_efficiency: f32,
    optional_multiplier: f32,
    heal_percent: f32,
    edelta: f32,
    block_health: f32,
    missing_health: f32,
) -> f32 {
    let amount =
        lerp(1.0, optional_multiplier, optional_efficiency) * heal_percent * edelta * block_health
            / 100.0;
    amount.min(missing_health)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseShieldState {
    pub broken: bool,
    pub hit: f32,
    pub smooth_radius: f32,
}

impl Default for BaseShieldState {
    fn default() -> Self {
        Self {
            broken: false,
            hit: 0.0,
            smooth_radius: 0.0,
        }
    }
}

pub fn base_shield_update(state: &mut BaseShieldState, radius: f32, efficiency: f32) -> f32 {
    state.smooth_radius = lerp_delta(state.smooth_radius, radius * efficiency, 0.05);
    state.smooth_radius
}

pub fn base_shield_should_interact(radius: f32) -> bool {
    radius > 1.0
}

pub fn base_shield_unit_overlap(unit_hit_size: f32, shield_radius: f32, distance: f32) -> f32 {
    (unit_hit_size / 2.0 + shield_radius) - distance
}

pub fn base_shield_unit_action(
    unit_hit_size: f32,
    shield_radius: f32,
    distance: f32,
) -> ShieldUnitAction {
    let overlap = base_shield_unit_overlap(unit_hit_size, shield_radius, distance);
    if overlap <= 0.0 {
        ShieldUnitAction::None
    } else if overlap > unit_hit_size * 1.5 {
        ShieldUnitAction::Kill
    } else {
        ShieldUnitAction::Repel {
            distance: overlap + 0.01,
        }
    }
}

pub fn write_base_shield_state<W: Write>(write: &mut W, state: &BaseShieldState) -> io::Result<()> {
    write_f32(write, state.smooth_radius)?;
    write.write_all(&[state.broken as u8])
}

pub fn read_base_shield_state<R: Read>(read: &mut R, revision: u8) -> io::Result<BaseShieldState> {
    if revision < 1 {
        return Ok(BaseShieldState::default());
    }
    Ok(BaseShieldState {
        smooth_radius: read_f32(read)?,
        broken: read_bool(read)?,
        hit: 0.0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShieldUnitAction {
    None,
    Repel { distance: f32 },
    Kill,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldWallState {
    pub shield: f32,
    pub shield_radius: f32,
    pub break_timer: f32,
    pub hit: f32,
}

impl ShieldWallState {
    pub fn new(shield_health: f32) -> Self {
        Self {
            shield: shield_health,
            shield_radius: 0.0,
            break_timer: 0.0,
            hit: 0.0,
        }
    }
}

pub fn shield_wall_broken(state: &ShieldWallState, can_consume: bool) -> bool {
    state.break_timer > 0.0 || !can_consume
}

pub fn shield_wall_update(
    state: &mut ShieldWallState,
    can_consume: bool,
    delta: f32,
    edelta: f32,
    shield_health: f32,
    regen_speed: f32,
) {
    if state.break_timer > 0.0 {
        state.break_timer -= delta;
    } else {
        state.shield = (state.shield + regen_speed * edelta).clamp(0.0, shield_health);
    }
    state.hit = wall_draw_hit_decay(state.hit, delta, false);
    state.shield_radius = lerp_delta(
        state.shield_radius,
        if shield_wall_broken(state, can_consume) {
            0.0
        } else {
            1.0
        },
        0.12,
    );
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldWallDamage {
    pub shield_taken: f32,
    pub passthrough_damage: f32,
    pub broke_now: bool,
}

pub fn shield_wall_damage(
    state: &mut ShieldWallState,
    can_consume: bool,
    damage: f32,
    break_cooldown: f32,
) -> ShieldWallDamage {
    let shield_taken = if shield_wall_broken(state, can_consume) {
        0.0
    } else {
        state.shield.min(damage)
    };
    state.shield -= shield_taken;
    if shield_taken > 0.0 {
        state.hit = 1.0;
    }
    let broke_now = state.shield <= 0.00001 && shield_taken > 0.0;
    if broke_now {
        state.break_timer = break_cooldown;
    }
    ShieldWallDamage {
        shield_taken,
        passthrough_damage: (damage - shield_taken).max(0.0),
        broke_now,
    }
}

pub fn shield_wall_pickup(state: &mut ShieldWallState) {
    state.shield_radius = 0.0;
}

pub fn write_shield_wall_state<W: Write>(write: &mut W, state: &ShieldWallState) -> io::Result<()> {
    write_f32(write, state.shield)
}

pub fn read_shield_wall_state<R: Read>(read: &mut R) -> io::Result<ShieldWallState> {
    let shield = read_f32(read)?;
    Ok(ShieldWallState {
        shield,
        shield_radius: if shield > 0.0 { 1.0 } else { 0.0 },
        break_timer: 0.0,
        hit: 0.0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalForceProjectorState {
    pub broken: bool,
    pub buildup: f32,
    pub hit: f32,
    pub warmup: f32,
    pub shield_radius: f32,
}

impl Default for DirectionalForceProjectorState {
    fn default() -> Self {
        Self {
            broken: true,
            buildup: 0.0,
            hit: 0.0,
            warmup: 0.0,
            shield_radius: 0.0,
        }
    }
}

pub fn directional_force_projector_update(
    state: &mut DirectionalForceProjectorState,
    efficiency: f32,
    delta: f32,
    width: f32,
    shield_health: f32,
) -> bool {
    state.shield_radius = lerp_delta(
        state.shield_radius,
        if state.broken {
            0.0
        } else {
            state.warmup * width
        },
        0.05,
    );
    state.warmup = lerp_delta(state.warmup, efficiency, 0.1);
    if state.broken && state.buildup <= 0.0 {
        state.broken = false;
    }
    let broke_now = state.buildup >= shield_health && !state.broken;
    if broke_now {
        state.broken = true;
        state.buildup = shield_health;
    }
    if state.hit > 0.0 {
        state.hit -= 1.0 / 5.0 * delta;
    }
    broke_now
}

pub fn directional_force_projector_picked_up(state: &mut DirectionalForceProjectorState) {
    state.shield_radius = 0.0;
    state.warmup = 0.0;
}

pub fn directional_force_projector_segment(
    origin_x: f32,
    origin_y: f32,
    length: f32,
    shield_radius: f32,
    rotation_degrees: f32,
) -> ((f32, f32), (f32, f32)) {
    (
        rotate_add(length, shield_radius, rotation_degrees, origin_x, origin_y),
        rotate_add(length, -shield_radius, rotation_degrees, origin_x, origin_y),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn directional_force_projector_absorb_bullet(
    state: &mut DirectionalForceProjectorState,
    enemy_team: bool,
    absorbable: bool,
    bullet_x: f32,
    bullet_y: f32,
    bullet_vel_x: f32,
    bullet_vel_y: f32,
    bullet_damage: f32,
    delta: f32,
    projector_x: f32,
    projector_y: f32,
    length: f32,
    rotation_degrees: f32,
) -> bool {
    if state.shield_radius <= 0.0 || state.broken || !enemy_team || !absorbable {
        return false;
    }
    let ((x1, y1), (x2, y2)) = directional_force_projector_segment(
        projector_x,
        projector_y,
        length,
        state.shield_radius,
        rotation_degrees,
    );
    if segments_intersect(
        (bullet_x, bullet_y),
        (
            bullet_x + bullet_vel_x * (delta + 1.1),
            bullet_y + bullet_vel_y * (delta + 1.1),
        ),
        (x1, y1),
        (x2, y2),
    ) {
        state.hit = 1.0;
        state.buildup += bullet_damage;
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarState {
    pub progress: f32,
    pub last_radius: f32,
    pub smooth_efficiency: f32,
    pub total_progress: f32,
}

impl Default for RadarState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            last_radius: 0.0,
            smooth_efficiency: 1.0,
            total_progress: 0.0,
        }
    }
}

pub fn radar_fog_radius(fog_radius: f32, progress: f32, smooth_efficiency: f32) -> f32 {
    fog_radius * progress * smooth_efficiency
}

pub fn radar_update(
    state: &mut RadarState,
    efficiency: f32,
    edelta: f32,
    fog_radius: f32,
    discovery_time: f32,
) -> bool {
    state.smooth_efficiency = lerp_delta(state.smooth_efficiency, efficiency, 0.05);
    let radius = radar_fog_radius(fog_radius, state.progress, state.smooth_efficiency);
    let force_update = (radius - state.last_radius).abs() >= 0.5;
    if force_update {
        state.last_radius = radius;
    }
    state.progress = (state.progress + edelta / discovery_time).clamp(0.0, 1.0);
    state.total_progress += efficiency * edelta;
    force_update
}

pub fn write_radar_state<W: Write>(write: &mut W, state: &RadarState) -> io::Result<()> {
    write_f32(write, state.progress)
}

pub fn read_radar_state<R: Read>(read: &mut R) -> io::Result<RadarState> {
    Ok(RadarState {
        progress: read_f32(read)?,
        ..RadarState::default()
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildTurretState {
    pub rotation: f32,
    pub warmup: f32,
    pub following: Option<i32>,
    pub last_plan: Option<BlockPlan>,
    pub plans: Vec<BuildPlan>,
    pub raw_plans: Vec<u8>,
}

impl Default for BuildTurretState {
    fn default() -> Self {
        Self {
            rotation: 90.0,
            warmup: 0.0,
            following: None,
            last_plan: None,
            plans: Vec::new(),
            raw_plans: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTurretPlanAction {
    NoPlan,
    Keep,
    DropConflictingBreak,
    DropInvalid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildTurretPlanValidation {
    pub action: BuildTurretPlanAction,
    pub removed_plan: Option<BuildPlan>,
    pub remove_team_plan_at: Option<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildTurretFollowCandidate {
    pub unit_id: i32,
    pub can_build: bool,
    pub actively_building: bool,
    pub plan: Option<BuildPlan>,
    pub construct_within_range: bool,
}

impl BuildTurretFollowCandidate {
    pub fn new(unit_id: i32, plan: BuildPlan) -> Self {
        Self {
            unit_id,
            can_build: true,
            actively_building: true,
            plan: Some(plan),
            construct_within_range: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTurretUpdateAction {
    Controlled,
    ClearInvalidFollowing,
    CopyFollowingPlan,
    ClaimTeamPlan,
    SelectFollowing,
    ValidateCurrentPlan,
    Idle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildTurretUpdateStep {
    pub action: BuildTurretUpdateAction,
    pub update_building: bool,
    pub update_build_logic: bool,
    pub copied_following_plan: Option<BuildPlan>,
    pub claimed_team_plan: Option<BlockPlan>,
    pub added_build_plan: Option<BuildPlan>,
    pub selected_following: Option<i32>,
    pub validation: Option<BuildTurretPlanValidation>,
    pub removed_self_plans: Vec<BuildPlan>,
    pub following: Option<i32>,
    pub last_plan: Option<BlockPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildTurretUnitTickInput {
    pub unit_rotation: f32,
    pub actively_building: bool,
    pub build_plan_angle: Option<f32>,
    pub suppressed: bool,
    pub efficiency: f32,
    pub potential_efficiency: f32,
    pub time_scale: f32,
    pub warmup: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildTurretUnitTickStep {
    pub rotation: f32,
    pub look_at: Option<f32>,
    pub efficiency: f32,
    pub potential_efficiency: f32,
    pub build_speed_multiplier: f32,
    pub speed_multiplier: f32,
    pub warmup: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildTurretUnitBinding {
    pub x: f32,
    pub y: f32,
    pub team: TeamId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTurretDrawCommand {
    Base,
    ResetColor,
    SetTurretLayer,
    Shadow,
    Region,
    Glow,
    UnitBuilding,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildTurretDrawPlan {
    pub commands: &'static [BuildTurretDrawCommand],
    pub x: f32,
    pub y: f32,
    pub elevation: f32,
    pub turret_rotation: f32,
    pub shadow_x: f32,
    pub shadow_y: f32,
    pub glow_alpha: f32,
    pub draw_unit_building: bool,
}

pub fn build_turret_elevation(configured: f32, size: i32) -> f32 {
    if configured < 0.0 {
        size as f32 / 2.0
    } else {
        configured
    }
}

pub fn build_turret_warmup_update(warmup: f32, actively_building: bool, efficiency: f32) -> f32 {
    lerp_delta(
        warmup,
        if actively_building { efficiency } else { 0.0 },
        0.1,
    )
}

pub fn build_turret_should_consume(plan_count: usize, heal_suppressed: bool) -> bool {
    plan_count > 0 && !heal_suppressed
}

pub fn build_turret_unit_tick(input: BuildTurretUnitTickInput) -> BuildTurretUnitTickStep {
    let efficiency = if input.suppressed {
        0.0
    } else {
        input.efficiency
    };
    let potential_efficiency = if input.suppressed {
        0.0
    } else {
        input.potential_efficiency
    };
    let multiplier = potential_efficiency * input.time_scale;

    BuildTurretUnitTickStep {
        rotation: input.unit_rotation,
        look_at: input
            .actively_building
            .then_some(input.build_plan_angle)
            .flatten(),
        efficiency,
        potential_efficiency,
        build_speed_multiplier: multiplier,
        speed_multiplier: multiplier,
        warmup: build_turret_warmup_update(input.warmup, input.actively_building, efficiency),
    }
}

pub fn apply_build_turret_unit_tick(
    state: &mut BuildTurretState,
    unit: &mut UnitComp,
    binding: BuildTurretUnitBinding,
    step: BuildTurretUnitTickStep,
    delta: f32,
) {
    unit.team.team = binding.team;
    unit.set_pos(binding.x, binding.y);
    state.rotation = step.rotation;

    if let Some(angle) = step.look_at {
        unit.look_at_angle(angle, delta);
    }

    unit.status.build_speed_multiplier = step.build_speed_multiplier;
    unit.status.speed_multiplier = step.speed_multiplier;
    state.warmup = step.warmup;
    unit.refresh_component_views();
}

const BUILD_TURRET_DRAW_COMMANDS_WITH_GLOW_AND_UNIT: &[BuildTurretDrawCommand] = &[
    BuildTurretDrawCommand::Base,
    BuildTurretDrawCommand::ResetColor,
    BuildTurretDrawCommand::SetTurretLayer,
    BuildTurretDrawCommand::Shadow,
    BuildTurretDrawCommand::Region,
    BuildTurretDrawCommand::Glow,
    BuildTurretDrawCommand::UnitBuilding,
];

const BUILD_TURRET_DRAW_COMMANDS_WITH_GLOW: &[BuildTurretDrawCommand] = &[
    BuildTurretDrawCommand::Base,
    BuildTurretDrawCommand::ResetColor,
    BuildTurretDrawCommand::SetTurretLayer,
    BuildTurretDrawCommand::Shadow,
    BuildTurretDrawCommand::Region,
    BuildTurretDrawCommand::Glow,
];

const BUILD_TURRET_DRAW_COMMANDS_WITH_UNIT: &[BuildTurretDrawCommand] = &[
    BuildTurretDrawCommand::Base,
    BuildTurretDrawCommand::ResetColor,
    BuildTurretDrawCommand::SetTurretLayer,
    BuildTurretDrawCommand::Shadow,
    BuildTurretDrawCommand::Region,
    BuildTurretDrawCommand::UnitBuilding,
];

const BUILD_TURRET_DRAW_COMMANDS_BASE: &[BuildTurretDrawCommand] = &[
    BuildTurretDrawCommand::Base,
    BuildTurretDrawCommand::ResetColor,
    BuildTurretDrawCommand::SetTurretLayer,
    BuildTurretDrawCommand::Shadow,
    BuildTurretDrawCommand::Region,
];

pub fn build_turret_draw_plan(
    x: f32,
    y: f32,
    rotation: f32,
    elevation: f32,
    warmup: f32,
    glow_region_found: bool,
    efficiency: f32,
) -> BuildTurretDrawPlan {
    let turret_rotation = rotation - 90.0;
    let draw_unit_building = efficiency > 0.0;
    let commands = match (glow_region_found, draw_unit_building) {
        (true, true) => BUILD_TURRET_DRAW_COMMANDS_WITH_GLOW_AND_UNIT,
        (true, false) => BUILD_TURRET_DRAW_COMMANDS_WITH_GLOW,
        (false, true) => BUILD_TURRET_DRAW_COMMANDS_WITH_UNIT,
        (false, false) => BUILD_TURRET_DRAW_COMMANDS_BASE,
    };

    BuildTurretDrawPlan {
        commands,
        x,
        y,
        elevation,
        turret_rotation,
        shadow_x: x - elevation,
        shadow_y: y - elevation,
        glow_alpha: if glow_region_found { warmup } else { 0.0 },
        draw_unit_building,
    }
}

pub const BUILD_TURRET_UNIT_TYPE_PREFIX: &str = "turret-unit-";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTurretUnitConstructor {
    BlockUnitUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildTurretUnitTypeConfig {
    pub unit_type: UnitType,
    pub constructor: BuildTurretUnitConstructor,
}

pub fn build_turret_unit_type(
    unit_type_id: ContentId,
    block_name: impl AsRef<str>,
    rotate_speed: f32,
    build_beam_offset: f32,
    range: f32,
    build_speed: f32,
) -> UnitType {
    let mut unit_type = UnitType::new(
        unit_type_id,
        format!("{}{}", BUILD_TURRET_UNIT_TYPE_PREFIX, block_name.as_ref()),
    );
    apply_build_turret_unit_type_defaults(
        &mut unit_type,
        rotate_speed,
        build_beam_offset,
        range,
        build_speed,
    );
    unit_type
}

pub fn build_turret_unit_type_config(
    unit_type_id: ContentId,
    block_name: impl AsRef<str>,
    rotate_speed: f32,
    build_beam_offset: f32,
    range: f32,
    build_speed: f32,
) -> BuildTurretUnitTypeConfig {
    BuildTurretUnitTypeConfig {
        unit_type: build_turret_unit_type(
            unit_type_id,
            block_name,
            rotate_speed,
            build_beam_offset,
            range,
            build_speed,
        ),
        constructor: BuildTurretUnitConstructor::BlockUnitUnit,
    }
}

pub fn apply_build_turret_unit_type_defaults(
    unit_type: &mut UnitType,
    rotate_speed: f32,
    build_beam_offset: f32,
    range: f32,
    build_speed: f32,
) {
    unit_type.hidden = true;
    unit_type.internal = true;
    unit_type.speed = 0.0;
    unit_type.hit_size = 0.0;
    unit_type.health = 1.0;
    unit_type.item_capacity = 0;
    build_turret_after_patch_unit_type(
        unit_type,
        rotate_speed,
        build_beam_offset,
        range,
        build_speed,
    );
}

pub fn build_turret_after_patch_unit_type(
    unit_type: &mut UnitType,
    rotate_speed: f32,
    build_beam_offset: f32,
    range: f32,
    build_speed: f32,
) {
    unit_type.rotate_speed = rotate_speed;
    unit_type.build_beam_offset = build_beam_offset;
    unit_type.build_range = range;
    unit_type.build_speed = build_speed;
}

pub fn build_turret_after_patch_unit_type_config(
    config: &mut BuildTurretUnitTypeConfig,
    rotate_speed: f32,
    build_beam_offset: f32,
    range: f32,
    build_speed: f32,
) {
    build_turret_after_patch_unit_type(
        &mut config.unit_type,
        rotate_speed,
        build_beam_offset,
        range,
        build_speed,
    );
}

pub fn build_turret_build_plan_from_team_plan(plan: &BlockPlan) -> BuildPlan {
    let mut build_plan = BuildPlan::new_place(
        plan.x as i32,
        plan.y as i32,
        plan.rotation as i32,
        plan.block.clone(),
    );
    if let Some(config) = &plan.config {
        build_plan.config = TypeValue::String(config.clone());
    }
    build_plan
}

pub fn build_turret_first_fit_plan<T, FWithin, FValid, FHasResources>(
    plans: &mut Vec<T>,
    within_range: FWithin,
    valid_place: FValid,
    has_resources: FHasResources,
) -> Option<T>
where
    T: Clone,
    FWithin: Fn(&T) -> bool,
    FValid: Fn(&T) -> bool,
    FHasResources: Fn(&T) -> bool,
{
    let index = plans
        .iter()
        .position(|plan| within_range(plan) && valid_place(plan) && has_resources(plan))?;
    let selected = plans.remove(index);
    let returned = selected.clone();
    plans.push(selected);
    Some(returned)
}

pub fn build_turret_choose_following<'a>(
    candidates: impl IntoIterator<Item = &'a BuildTurretFollowCandidate>,
) -> Option<i32> {
    candidates
        .into_iter()
        .find(|candidate| {
            candidate.can_build
                && candidate.actively_building
                && candidate.plan.is_some()
                && candidate.construct_within_range
        })
        .map(|candidate| candidate.unit_id)
}

pub fn build_turret_remove_self_plans(
    unit_plans: &mut VecDeque<BuildPlan>,
    self_plan_pos: Option<(i32, i32)>,
) -> Vec<BuildPlan> {
    let Some((self_x, self_y)) = self_plan_pos else {
        return Vec::new();
    };

    let mut removed = Vec::new();
    let mut kept = VecDeque::with_capacity(unit_plans.len());
    while let Some(plan) = unit_plans.pop_front() {
        if plan.x == self_x && plan.y == self_y && !plan.breaking {
            removed.push(plan);
        } else {
            kept.push_back(plan);
        }
    }
    *unit_plans = kept;
    removed
}

pub fn build_turret_validate_current_plan<FValidBreak, FValidPlace>(
    state: &mut BuildTurretState,
    unit_plans: &mut VecDeque<BuildPlan>,
    conflicting_breaker: bool,
    construct_current_matches: bool,
    mut valid_break: FValidBreak,
    mut valid_place: FValidPlace,
) -> BuildTurretPlanValidation
where
    FValidBreak: FnMut(&BuildPlan) -> bool,
    FValidPlace: FnMut(&BuildPlan) -> bool,
{
    let Some(request) = unit_plans.front().cloned() else {
        return BuildTurretPlanValidation {
            action: BuildTurretPlanAction::NoPlan,
            removed_plan: None,
            remove_team_plan_at: None,
        };
    };

    if !request.breaking && conflicting_breaker {
        let removed_plan = unit_plans.pop_front();
        return BuildTurretPlanValidation {
            action: BuildTurretPlanAction::DropConflictingBreak,
            removed_plan,
            remove_team_plan_at: Some((request.x, request.y)),
        };
    }

    let last_plan_removed = state
        .last_plan
        .as_ref()
        .is_some_and(|last_plan| last_plan.removed);
    let valid = !last_plan_removed
        && (construct_current_matches
            || if request.breaking {
                valid_break(&request)
            } else {
                valid_place(&request)
            });

    if valid {
        BuildTurretPlanValidation {
            action: BuildTurretPlanAction::Keep,
            removed_plan: None,
            remove_team_plan_at: None,
        }
    } else {
        let removed_plan = unit_plans.pop_front();
        state.last_plan = None;
        BuildTurretPlanValidation {
            action: BuildTurretPlanAction::DropInvalid,
            removed_plan,
            remove_team_plan_at: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_turret_update_tick<FWithin, FValidTeamPlan, FHasResources, FValidBreak, FValidPlace>(
    state: &mut BuildTurretState,
    unit_plans: &mut VecDeque<BuildPlan>,
    team_plans: &mut Vec<BlockPlan>,
    controlled: bool,
    timer_target_ready: bool,
    following_valid: bool,
    following_actively_building: bool,
    following_plan: Option<BuildPlan>,
    follow_candidates: &[BuildTurretFollowCandidate],
    conflicting_breaker: bool,
    construct_current_matches: bool,
    self_plan_pos: Option<(i32, i32)>,
    within_range: FWithin,
    valid_team_plan: FValidTeamPlan,
    has_resources: FHasResources,
    valid_break: FValidBreak,
    valid_place: FValidPlace,
) -> BuildTurretUpdateStep
where
    FWithin: Fn(&BlockPlan) -> bool,
    FValidTeamPlan: Fn(&BlockPlan) -> bool,
    FHasResources: Fn(&BlockPlan) -> bool,
    FValidBreak: FnMut(&BuildPlan) -> bool,
    FValidPlace: FnMut(&BuildPlan) -> bool,
{
    let mut action = BuildTurretUpdateAction::Idle;
    let mut copied_following_plan = None;
    let mut claimed_team_plan = None;
    let mut added_build_plan = None;
    let mut selected_following = None;
    let mut validation = None;

    if controlled {
        state.following = None;
        state.last_plan = None;
        action = BuildTurretUpdateAction::Controlled;
    } else if state.following.is_some() {
        if !following_valid || !following_actively_building {
            state.following = None;
            unit_plans.clear();
            action = BuildTurretUpdateAction::ClearInvalidFollowing;
        } else {
            unit_plans.clear();
            if let Some(plan) = following_plan {
                unit_plans.push_front(plan.clone());
                copied_following_plan = Some(plan);
            }
            state.last_plan = None;
            action = BuildTurretUpdateAction::CopyFollowingPlan;
        }
    } else if unit_plans.front().is_none() && timer_target_ready {
        if let Some(plan) =
            build_turret_first_fit_plan(team_plans, within_range, valid_team_plan, has_resources)
        {
            let build_plan = build_turret_build_plan_from_team_plan(&plan);
            unit_plans.push_back(build_plan.clone());
            state.last_plan = Some(plan.clone());
            claimed_team_plan = Some(plan);
            added_build_plan = Some(build_plan);
            action = BuildTurretUpdateAction::ClaimTeamPlan;
        }

        if unit_plans.front().is_none() {
            state.following = build_turret_choose_following(follow_candidates.iter());
            if let Some(following) = state.following {
                selected_following = Some(following);
                action = BuildTurretUpdateAction::SelectFollowing;
            }
        }
    } else if unit_plans.front().is_some() {
        let result = build_turret_validate_current_plan(
            state,
            unit_plans,
            conflicting_breaker,
            construct_current_matches,
            valid_break,
            valid_place,
        );
        validation = Some(result);
        action = BuildTurretUpdateAction::ValidateCurrentPlan;
    }

    let removed_self_plans = build_turret_remove_self_plans(unit_plans, self_plan_pos);

    BuildTurretUpdateStep {
        action,
        update_building: !controlled,
        update_build_logic: true,
        copied_following_plan,
        claimed_team_plan,
        added_build_plan,
        selected_following,
        validation,
        removed_self_plans,
        following: state.following,
        last_plan: state.last_plan.clone(),
    }
}

pub fn build_turret_write_child<W: Write>(
    write: &mut W,
    state: &BuildTurretState,
) -> io::Result<()> {
    write_f32(write, state.rotation)?;
    write.write_all(&state.raw_plans)
}

pub fn build_turret_read_child<R: Read>(read: &mut R) -> io::Result<BuildTurretState> {
    let rotation = read_f32(read)?;
    let mut raw_plans = Vec::new();
    read.read_to_end(&mut raw_plans)?;
    Ok(BuildTurretState {
        rotation,
        raw_plans,
        plans: Vec::new(),
        following: None,
        last_plan: None,
        warmup: 0.0,
    })
}

pub fn build_turret_write_child_with_loader<W: Write>(
    write: &mut W,
    loader: &ContentLoader,
    state: &BuildTurretState,
) -> io::Result<()> {
    write_f32(write, state.rotation)?;
    if state.plans.is_empty() && !state.raw_plans.is_empty() {
        write.write_all(&state.raw_plans)
    } else {
        type_io::write_build_plans(write, loader, Some(&state.plans))
    }
}

pub fn build_turret_read_child_with_loader<R: Read>(
    read: &mut R,
    loader: &ContentLoader,
) -> io::Result<BuildTurretState> {
    let rotation = read_f32(read)?;
    let plans = type_io::read_build_plans(read, loader)?.unwrap_or_default();
    Ok(BuildTurretState {
        rotation,
        plans,
        raw_plans: Vec::new(),
        following: None,
        last_plan: None,
        warmup: 0.0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockwaveTowerState {
    pub reload_counter: f32,
    pub heat: f32,
}

impl ShockwaveTowerState {
    pub fn new(initial_reload_counter: f32) -> Self {
        Self {
            reload_counter: initial_reload_counter,
            heat: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockwaveTowerFire {
    pub fired: bool,
    pub wave_damage: f32,
    pub removed_targets: usize,
}

pub fn shockwave_tower_update(
    state: &mut ShockwaveTowerState,
    potential_efficiency: f32,
    edelta: f32,
    delta: f32,
    reload: f32,
    bullet_damage: f32,
    falloff_count: f32,
    target_damages: &mut [f32],
    timer_ready: bool,
    cooldown_multiplier: f32,
) -> ShockwaveTowerFire {
    let mut fire = ShockwaveTowerFire {
        fired: false,
        wave_damage: 0.0,
        removed_targets: 0,
    };
    if potential_efficiency > 0.0 {
        state.reload_counter += edelta;
    }
    if potential_efficiency > 0.0
        && state.reload_counter >= reload
        && timer_ready
        && !target_damages.is_empty()
    {
        state.heat = 1.0;
        state.reload_counter = 0.0;
        fire.fired = true;
        fire.wave_damage =
            bullet_damage.min(bullet_damage * falloff_count / target_damages.len() as f32);
        for damage in target_damages {
            if *damage > fire.wave_damage {
                *damage -= fire.wave_damage;
            } else {
                *damage = 0.0;
                fire.removed_targets += 1;
            }
        }
    }
    state.heat = (state.heat - delta / reload * cooldown_multiplier).clamp(0.0, 1.0);
    fire
}

pub fn shockwave_tower_progress(reload_counter: f32, reload: f32) -> f32 {
    reload_counter / reload
}

pub fn shockwave_tower_should_consume(reload_counter: f32, reload: f32) -> bool {
    reload_counter < reload
}

pub fn thruster_top_rotation(rotation: i32) -> f32 {
    rotation as f32 * 90.0
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn read_bool<R: Read>(read: &mut R) -> io::Result<bool> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0] != 0)
}

fn rotate_add(x: f32, y: f32, degrees: f32, add_x: f32, add_y: f32) -> (f32, f32) {
    let radians = degrees.to_radians();
    let cos = radians.cos();
    let sin = radians.sin();
    (x * cos - y * sin + add_x, x * sin + y * cos + add_y)
}

fn segments_intersect(a1: (f32, f32), a2: (f32, f32), b1: (f32, f32), b2: (f32, f32)) -> bool {
    fn cross(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }
    let d1 = cross(a1, a2, b1);
    let d2 = cross(a1, a2, b2);
    let d3 = cross(b1, b2, a1);
    let d4 = cross(b1, b2, a2);
    d1.signum() != d2.signum() && d3.signum() != d4.signum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_door_auto_door_and_mine_helpers_follow_upstream() {
        assert_eq!(wall_collision_hit(0.0), 1.0);
        assert_eq!(wall_draw_hit_decay(1.0, 5.0, false), 0.5);
        assert!(wall_should_lightning(0.25, 0.2));
        assert!(wall_deflects_bullet(10.0, 1.0, true, 20.0, 0.4));
        assert!(!wall_deflects_bullet(10.0, 0.05, true, 20.0, 0.0));
        assert!(wall_reflect_x(6.0, 3.0));

        assert!(door_check_solid(false));
        assert_eq!(door_sense_enabled(true), 1.0);
        assert!(door_can_toggle(false, true, false, true));
        assert!(!door_can_toggle(false, false, false, true));
        assert!(door_tapped_should_configure(false, true, true));
        assert!(!door_tapped_should_configure(true, true, true));

        let mut bytes = Vec::new();
        write_door_state(&mut bytes, DoorState { open: true }).unwrap();
        assert_eq!(
            read_door_state(&mut bytes.as_slice()).unwrap(),
            DoorState { open: true }
        );

        assert!(auto_door_should_open(true));
        assert_eq!(auto_door_trigger_size(2, 8.0, 12.0), 40.0);

        assert!(shock_mine_should_trigger(true, false, true));
        assert_eq!(
            shock_mine_bullet_angles(4, &[1.0, 2.0]),
            vec![1.0, 92.0, 180.0, 270.0]
        );
    }

    #[test]
    fn mend_and_overdrive_projectors_update_and_serialize_java_fields() {
        let mut mend = MendProjectorState {
            charge: 249.95,
            ..MendProjectorState::default()
        };
        let update = mend_projector_update(
            &mut mend, 1.0, 0.5, true, 1.0, 250.0, 60.0, 12.0, 12.0, 50.0,
        );
        assert!(update.fired);
        assert_eq!(mend.charge, 0.0);
        assert_eq!(mend.heat, 0.08);
        assert_eq!(update.real_range, 62.5);
        assert!((update.heal_fraction - 0.126).abs() < 0.00001);

        let mut bytes = Vec::new();
        write_mend_projector_state(&mut bytes, &mend).unwrap();
        assert_eq!(
            read_mend_projector_state(&mut bytes.as_slice())
                .unwrap()
                .phase_heat,
            mend.phase_heat
        );

        let mut over = OverdriveProjectorState {
            charge: 59.95,
            use_progress: 399.5,
            ..OverdriveProjectorState::default()
        };
        let update = overdrive_projector_update(
            &mut over, 1.0, 0.5, true, 1.0, 60.0, 80.0, 20.0, 1.5, 0.75, 400.0,
        );
        assert!(update.applied_boost);
        assert!(update.consumed);
        assert_eq!(update.real_range, 81.0);
        assert!((update.real_boost - 1.5375).abs() < 0.00001);

        let mut bytes = Vec::new();
        write_overdrive_projector_state(&mut bytes, &over).unwrap();
        assert_eq!(
            read_overdrive_projector_state(&mut bytes.as_slice())
                .unwrap()
                .heat,
            over.heat
        );
    }

    #[test]
    fn force_and_regen_projector_helpers_follow_upstream() {
        let mut force = ForceProjectorState {
            broken: true,
            buildup: 0.1,
            ..ForceProjectorState::default()
        };
        let broke = force_projector_update(
            &mut force, 1.0, true, 0.0, 0.0, 1.0, 700.0, 400.0, 1.75, 0.35, 1.5,
        );
        assert!(!broke);
        assert!(!force.broken);
        assert_eq!(force.phase_heat, 0.1);
        assert_eq!(force_projector_real_radius(100.0, 0.5, 80.0, 0.5), 70.0);
        assert_eq!(
            force_projector_shield(false, 700.0, 400.0, 0.5, 100.0),
            800.0
        );
        assert!(force_projector_absorb_explosion(
            &mut force, true, 10.0, 2.0
        ));
        assert_eq!(force.hit, 1.0);

        let mut bytes = Vec::new();
        write_force_projector_state(&mut bytes, &force).unwrap();
        let restored = read_force_projector_state(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.broken, force.broken);
        assert_eq!(restored.buildup, force.buildup);

        assert_eq!(
            regen_projector_heal_amount(0.5, 2.0, 0.2, 1.0, 1000.0, 20.0),
            3.0
        );
        assert_eq!(
            regen_projector_heal_amount(1.0, 2.0, 12.0, 1.0, 1000.0, 20.0),
            20.0
        );
    }

    #[test]
    fn base_and_shield_wall_helpers_follow_upstream_state_order() {
        let mut base = BaseShieldState::default();
        assert_eq!(base_shield_update(&mut base, 200.0, 0.5), 5.0);
        assert!(base_shield_should_interact(5.0));
        assert_eq!(
            base_shield_unit_action(10.0, 20.0, 18.0),
            ShieldUnitAction::Repel { distance: 7.01 }
        );
        assert_eq!(
            base_shield_unit_action(10.0, 20.0, 5.0),
            ShieldUnitAction::Kill
        );

        let mut bytes = Vec::new();
        base.broken = true;
        write_base_shield_state(&mut bytes, &base).unwrap();
        assert_eq!(
            read_base_shield_state(&mut bytes.as_slice(), 1).unwrap(),
            base
        );
        assert_eq!(
            read_base_shield_state(&mut [].as_slice(), 0).unwrap(),
            BaseShieldState::default()
        );

        let mut wall = ShieldWallState {
            shield: 15.0,
            shield_radius: 1.0,
            break_timer: 0.0,
            hit: 0.8,
        };
        let damage = shield_wall_damage(&mut wall, true, 20.0, 600.0);
        assert_eq!(damage.shield_taken, 15.0);
        assert_eq!(damage.passthrough_damage, 5.0);
        assert!(damage.broke_now);
        assert_eq!(wall.break_timer, 600.0);
        shield_wall_update(&mut wall, true, 10.0, 10.0, 900.0, 2.0);
        assert_eq!(wall.break_timer, 590.0);
        assert_eq!(wall.hit, 0.0);
        shield_wall_pickup(&mut wall);
        assert_eq!(wall.shield_radius, 0.0);

        let mut bytes = Vec::new();
        write_shield_wall_state(&mut bytes, &wall).unwrap();
        let restored = read_shield_wall_state(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.shield, wall.shield);
        assert_eq!(restored.shield_radius, 0.0);
    }

    #[test]
    fn directional_force_projector_radar_build_turret_and_thruster_follow_upstream() {
        let mut directional = DirectionalForceProjectorState {
            broken: false,
            warmup: 1.0,
            shield_radius: 10.0,
            ..DirectionalForceProjectorState::default()
        };
        assert!(!directional_force_projector_update(
            &mut directional,
            0.5,
            1.0,
            30.0,
            3000.0
        ));
        assert!((directional.warmup - 0.95).abs() < 0.00001);
        let segment = directional_force_projector_segment(0.0, 0.0, 40.0, 10.0, 0.0);
        assert_eq!(segment, ((40.0, 10.0), (40.0, -10.0)));
        assert!(directional_force_projector_absorb_bullet(
            &mut directional,
            true,
            true,
            30.0,
            0.0,
            10.0,
            0.0,
            50.0,
            1.0,
            0.0,
            0.0,
            40.0,
            0.0
        ));
        assert_eq!(directional.hit, 1.0);
        assert!(directional.buildup >= 50.0);
        directional_force_projector_picked_up(&mut directional);
        assert_eq!(directional.shield_radius, 0.0);

        let mut radar = RadarState::default();
        assert!(!radar_update(&mut radar, 1.0, 60.0, 10.0, 600.0));
        assert_eq!(radar.progress, 0.1);
        assert_eq!(
            radar_fog_radius(10.0, radar.progress, radar.smooth_efficiency),
            1.0
        );
        assert!(radar_update(&mut radar, 1.0, 60.0, 10.0, 600.0));
        let mut bytes = Vec::new();
        write_radar_state(&mut bytes, &radar).unwrap();
        assert_eq!(
            read_radar_state(&mut bytes.as_slice()).unwrap().progress,
            radar.progress
        );

        assert_eq!(build_turret_elevation(-1.0, 3), 1.5);
        assert_eq!(build_turret_warmup_update(0.0, true, 0.8), 0.080000006);
        assert!(build_turret_should_consume(1, false));
        let mut unit_config =
            build_turret_unit_type_config(-1, "build-tower", 10.0, 5.0, 80.0, 1.5);
        assert_eq!(
            unit_config.unit_type.base.mappable.name,
            "turret-unit-build-tower"
        );
        assert_eq!(
            unit_config.constructor,
            BuildTurretUnitConstructor::BlockUnitUnit
        );
        assert!(unit_config.unit_type.hidden);
        assert!(unit_config.unit_type.internal);
        assert_eq!(unit_config.unit_type.speed, 0.0);
        assert_eq!(unit_config.unit_type.hit_size, 0.0);
        assert_eq!(unit_config.unit_type.health, 1.0);
        assert_eq!(unit_config.unit_type.item_capacity, 0);
        assert_eq!(unit_config.unit_type.rotate_speed, 10.0);
        assert_eq!(unit_config.unit_type.build_beam_offset, 5.0);
        assert_eq!(unit_config.unit_type.build_range, 80.0);
        assert_eq!(unit_config.unit_type.build_speed, 1.5);
        build_turret_after_patch_unit_type_config(&mut unit_config, 12.0, 7.5, 96.0, 2.25);
        assert_eq!(unit_config.unit_type.rotate_speed, 12.0);
        assert_eq!(unit_config.unit_type.build_beam_offset, 7.5);
        assert_eq!(unit_config.unit_type.build_range, 96.0);
        assert_eq!(unit_config.unit_type.build_speed, 2.25);
        assert!(unit_config.unit_type.hidden);
        assert!(unit_config.unit_type.internal);
        assert_eq!(unit_config.unit_type.speed, 0.0);
        assert_eq!(unit_config.unit_type.hit_size, 0.0);
        assert_eq!(unit_config.unit_type.health, 1.0);
        assert_eq!(unit_config.unit_type.item_capacity, 0);
        let build = BuildTurretState {
            rotation: 45.0,
            warmup: 0.6,
            following: None,
            last_plan: None,
            plans: Vec::new(),
            raw_plans: vec![0, 2, 7, 9],
        };
        let mut bytes = Vec::new();
        build_turret_write_child(&mut bytes, &build).unwrap();
        let restored = build_turret_read_child(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.rotation, 45.0);
        assert_eq!(restored.raw_plans, vec![0, 2, 7, 9]);

        assert_eq!(thruster_top_rotation(3), 270.0);
    }

    #[test]
    fn build_turret_child_read_write_with_loader_round_trips_java_typeio_plans() {
        let loader = ContentLoader::create_base_content().unwrap();
        let state = BuildTurretState {
            rotation: 135.0,
            warmup: 0.4,
            following: Some(99),
            last_plan: Some(BlockPlan::new(1, 1, 0, "router", None)),
            plans: vec![
                BuildPlan::new_string_config(2, 3, 1, "router", "cfg"),
                BuildPlan::new_break(4, 5),
            ],
            raw_plans: Vec::new(),
        };

        let mut bytes = Vec::new();
        build_turret_write_child_with_loader(&mut bytes, &loader, &state).unwrap();
        assert_eq!(&bytes[0..4], &135.0f32.to_be_bytes());
        assert_eq!(&bytes[4..6], &[0, 2]);

        let restored = build_turret_read_child_with_loader(&mut bytes.as_slice(), &loader).unwrap();
        assert_eq!(restored.rotation, 135.0);
        assert_eq!(restored.plans, state.plans);
        assert!(restored.raw_plans.is_empty());
        assert_eq!(restored.warmup, 0.0);
        assert_eq!(restored.following, None);
        assert_eq!(restored.last_plan, None);

        let empty = BuildTurretState {
            rotation: 90.0,
            ..BuildTurretState::default()
        };
        bytes.clear();
        build_turret_write_child_with_loader(&mut bytes, &loader, &empty).unwrap();
        assert_eq!(&bytes[0..4], &90.0f32.to_be_bytes());
        assert_eq!(&bytes[4..6], &[0, 0]);
        assert_eq!(
            build_turret_read_child_with_loader(&mut bytes.as_slice(), &loader)
                .unwrap()
                .plans,
            Vec::<BuildPlan>::new()
        );
    }

    #[test]
    fn build_turret_consumes_team_plan_queue_and_moves_consumed_plan_to_tail() {
        let mut plans = vec![
            crate::mindustry::game::BlockPlan::new(1, 1, 0, "duo", None),
            crate::mindustry::game::BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            crate::mindustry::game::BlockPlan::new(3, 3, 2, "wall", None),
        ];

        let selected = build_turret_first_fit_plan(
            &mut plans,
            |plan| plan.x == 2 || plan.x == 3,
            |plan| plan.block != "wall",
            |_| true,
        )
        .expect("first in-range valid plan should be selected");

        assert_eq!(
            selected,
            crate::mindustry::game::BlockPlan::new(2, 2, 1, "router", Some("cfg".into()))
        );
        assert_eq!(
            plans,
            vec![
                crate::mindustry::game::BlockPlan::new(1, 1, 0, "duo", None),
                crate::mindustry::game::BlockPlan::new(3, 3, 2, "wall", None),
                crate::mindustry::game::BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            ]
        );

        let build_plan =
            crate::mindustry::entities::comp::BuilderComp::build_plan_from_team_plan(&selected);
        assert_eq!(build_plan.x, 2);
        assert_eq!(build_plan.y, 2);
        assert_eq!(build_plan.rotation, 1);
        assert_eq!(build_plan.block.as_deref(), Some("router"));
        assert_eq!(
            build_plan.config,
            crate::mindustry::io::TypeValue::String("cfg".into())
        );
    }

    #[test]
    fn build_turret_unit_tick_matches_java_update_tile_front_half() {
        let building = build_turret_unit_tick(BuildTurretUnitTickInput {
            unit_rotation: 123.0,
            actively_building: true,
            build_plan_angle: Some(45.0),
            suppressed: false,
            efficiency: 0.8,
            potential_efficiency: 0.75,
            time_scale: 2.0,
            warmup: 0.2,
        });

        assert_eq!(building.rotation, 123.0);
        assert_eq!(building.look_at, Some(45.0));
        assert_eq!(building.efficiency, 0.8);
        assert_eq!(building.potential_efficiency, 0.75);
        assert_eq!(building.build_speed_multiplier, 1.5);
        assert_eq!(building.speed_multiplier, 1.5);
        assert!((building.warmup - 0.26).abs() < f32::EPSILON);

        let idle = build_turret_unit_tick(BuildTurretUnitTickInput {
            unit_rotation: 270.0,
            actively_building: false,
            build_plan_angle: Some(180.0),
            suppressed: false,
            efficiency: 0.9,
            potential_efficiency: 0.5,
            time_scale: 1.5,
            warmup: 0.6,
        });

        assert_eq!(idle.rotation, 270.0);
        assert_eq!(idle.look_at, None);
        assert_eq!(idle.build_speed_multiplier, 0.75);
        assert_eq!(idle.speed_multiplier, 0.75);
        assert_eq!(idle.warmup, 0.54);

        let suppressed = build_turret_unit_tick(BuildTurretUnitTickInput {
            unit_rotation: 30.0,
            actively_building: true,
            build_plan_angle: Some(90.0),
            suppressed: true,
            efficiency: 1.0,
            potential_efficiency: 1.0,
            time_scale: 3.0,
            warmup: 0.5,
        });

        assert_eq!(suppressed.rotation, 30.0);
        assert_eq!(suppressed.look_at, Some(90.0));
        assert_eq!(suppressed.efficiency, 0.0);
        assert_eq!(suppressed.potential_efficiency, 0.0);
        assert_eq!(suppressed.build_speed_multiplier, 0.0);
        assert_eq!(suppressed.speed_multiplier, 0.0);
        assert_eq!(suppressed.warmup, 0.45);
    }

    #[test]
    fn apply_build_turret_unit_tick_binds_unit_and_writes_runtime_state() {
        let mut unit_type = build_turret_unit_type(-1, "build-tower", 90.0, 5.0, 80.0, 1.0);
        unit_type.rotate_speed = 90.0;
        let mut unit = UnitComp::new(7, unit_type, TeamId(1));
        unit.set_rotation(0.0);
        unit.status.speed_multiplier = 1.0;
        unit.status.build_speed_multiplier = 1.0;
        let mut state = BuildTurretState {
            rotation: 270.0,
            warmup: 0.0,
            ..BuildTurretState::default()
        };

        let step = build_turret_unit_tick(BuildTurretUnitTickInput {
            unit_rotation: unit.rotation(),
            actively_building: true,
            build_plan_angle: Some(90.0),
            suppressed: false,
            efficiency: 0.6,
            potential_efficiency: 0.5,
            time_scale: 2.0,
            warmup: state.warmup,
        });

        apply_build_turret_unit_tick(
            &mut state,
            &mut unit,
            BuildTurretUnitBinding {
                x: 40.0,
                y: 48.0,
                team: TeamId(3),
            },
            step,
            1.0,
        );

        assert_eq!((unit.x(), unit.y()), (40.0, 48.0));
        assert_eq!(unit.team_id(), TeamId(3));
        assert_eq!(state.rotation, 0.0);
        assert_eq!(unit.rotation(), 90.0);
        assert_eq!(unit.status.build_speed_multiplier, 1.0);
        assert_eq!(unit.status.speed_multiplier, 1.0);
        assert_eq!(unit.builder.build_speed_multiplier, 1.0);
        assert_eq!(state.warmup, 0.060000002);

        let idle = build_turret_unit_tick(BuildTurretUnitTickInput {
            unit_rotation: unit.rotation(),
            actively_building: false,
            build_plan_angle: Some(180.0),
            suppressed: true,
            efficiency: 1.0,
            potential_efficiency: 1.0,
            time_scale: 3.0,
            warmup: state.warmup,
        });

        apply_build_turret_unit_tick(
            &mut state,
            &mut unit,
            BuildTurretUnitBinding {
                x: 56.0,
                y: 64.0,
                team: TeamId(4),
            },
            idle,
            1.0,
        );

        assert_eq!((unit.x(), unit.y()), (56.0, 64.0));
        assert_eq!(unit.team_id(), TeamId(4));
        assert_eq!(state.rotation, 90.0);
        assert_eq!(unit.rotation(), 90.0);
        assert_eq!(unit.status.build_speed_multiplier, 0.0);
        assert_eq!(unit.status.speed_multiplier, 0.0);
        assert_eq!(unit.builder.build_speed_multiplier, 0.0);
        assert!((state.warmup - 0.054).abs() < f32::EPSILON);
    }

    #[test]
    fn build_turret_draw_plan_matches_java_draw_order_and_conditions() {
        let glowing = build_turret_draw_plan(40.0, 48.0, 135.0, 3.0, 0.65, true, 0.5);
        assert_eq!(
            glowing.commands,
            &[
                BuildTurretDrawCommand::Base,
                BuildTurretDrawCommand::ResetColor,
                BuildTurretDrawCommand::SetTurretLayer,
                BuildTurretDrawCommand::Shadow,
                BuildTurretDrawCommand::Region,
                BuildTurretDrawCommand::Glow,
                BuildTurretDrawCommand::UnitBuilding,
            ]
        );
        assert_eq!((glowing.x, glowing.y), (40.0, 48.0));
        assert_eq!((glowing.shadow_x, glowing.shadow_y), (37.0, 45.0));
        assert_eq!(glowing.turret_rotation, 45.0);
        assert_eq!(glowing.glow_alpha, 0.65);
        assert!(glowing.draw_unit_building);

        let no_glow_idle = build_turret_draw_plan(8.0, 16.0, 90.0, 1.5, 0.9, false, 0.0);
        assert_eq!(
            no_glow_idle.commands,
            &[
                BuildTurretDrawCommand::Base,
                BuildTurretDrawCommand::ResetColor,
                BuildTurretDrawCommand::SetTurretLayer,
                BuildTurretDrawCommand::Shadow,
                BuildTurretDrawCommand::Region,
            ]
        );
        assert_eq!((no_glow_idle.shadow_x, no_glow_idle.shadow_y), (6.5, 14.5));
        assert_eq!(no_glow_idle.turret_rotation, 0.0);
        assert_eq!(no_glow_idle.glow_alpha, 0.0);
        assert!(!no_glow_idle.draw_unit_building);

        let unit_only = build_turret_draw_plan(0.0, 0.0, 180.0, 2.0, 0.4, false, 0.01);
        assert_eq!(
            unit_only.commands,
            &[
                BuildTurretDrawCommand::Base,
                BuildTurretDrawCommand::ResetColor,
                BuildTurretDrawCommand::SetTurretLayer,
                BuildTurretDrawCommand::Shadow,
                BuildTurretDrawCommand::Region,
                BuildTurretDrawCommand::UnitBuilding,
            ]
        );
        assert!(unit_only.draw_unit_building);
    }

    #[test]
    fn build_turret_update_claims_team_plan_before_following_candidates() {
        let mut state = BuildTurretState::default();
        let mut unit_plans = VecDeque::new();
        let mut team_plans = vec![
            BlockPlan::new(1, 1, 0, "duo", None),
            BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
        ];
        let followers = [BuildTurretFollowCandidate::new(
            77,
            BuildPlan::new_place(9, 9, 0, "wall"),
        )];

        let step = build_turret_update_tick(
            &mut state,
            &mut unit_plans,
            &mut team_plans,
            false,
            true,
            false,
            false,
            None,
            &followers,
            false,
            false,
            None,
            |plan| plan.x == 2,
            |plan| plan.block == "router",
            |_| true,
            |_| false,
            |_| true,
        );

        assert_eq!(step.action, BuildTurretUpdateAction::ClaimTeamPlan);
        assert_eq!(
            step.claimed_team_plan,
            Some(BlockPlan::new(2, 2, 1, "router", Some("cfg".into())))
        );
        assert_eq!(
            step.added_build_plan,
            Some(BuildPlan::new_string_config(2, 2, 1, "router", "cfg"))
        );
        assert_eq!(state.following, None);
        assert_eq!(
            team_plans,
            vec![
                BlockPlan::new(1, 1, 0, "duo", None),
                BlockPlan::new(2, 2, 1, "router", Some("cfg".into())),
            ]
        );
    }

    #[test]
    fn build_turret_update_selects_following_when_no_team_plan_matches() {
        let mut state = BuildTurretState::default();
        let mut unit_plans = VecDeque::new();
        let mut team_plans = vec![BlockPlan::new(1, 1, 0, "duo", None)];
        let followers = [
            BuildTurretFollowCandidate {
                unit_id: 1,
                actively_building: false,
                ..BuildTurretFollowCandidate::new(1, BuildPlan::new_place(1, 1, 0, "duo"))
            },
            BuildTurretFollowCandidate::new(2, BuildPlan::new_place(3, 3, 0, "router")),
        ];

        let step = build_turret_update_tick(
            &mut state,
            &mut unit_plans,
            &mut team_plans,
            false,
            true,
            false,
            false,
            None,
            &followers,
            false,
            false,
            None,
            |_| false,
            |_| true,
            |_| true,
            |_| false,
            |_| true,
        );

        assert_eq!(step.action, BuildTurretUpdateAction::SelectFollowing);
        assert_eq!(step.selected_following, Some(2));
        assert_eq!(state.following, Some(2));
        assert!(unit_plans.is_empty());
        assert_eq!(team_plans, vec![BlockPlan::new(1, 1, 0, "duo", None)]);
    }

    #[test]
    fn build_turret_update_following_copies_or_clears_plan() {
        let mut state = BuildTurretState {
            following: Some(7),
            last_plan: Some(BlockPlan::new(4, 4, 0, "duo", None)),
            ..BuildTurretState::default()
        };
        let mut unit_plans = VecDeque::from([BuildPlan::new_place(1, 1, 0, "wall")]);
        let mut team_plans = Vec::new();

        let copied = build_turret_update_tick(
            &mut state,
            &mut unit_plans,
            &mut team_plans,
            false,
            true,
            true,
            true,
            Some(BuildPlan::new_place(6, 6, 0, "router")),
            &[],
            false,
            false,
            None,
            |_| false,
            |_| false,
            |_| false,
            |_| false,
            |_| true,
        );

        assert_eq!(copied.action, BuildTurretUpdateAction::CopyFollowingPlan);
        assert_eq!(
            unit_plans,
            VecDeque::from([BuildPlan::new_place(6, 6, 0, "router")])
        );
        assert_eq!(state.last_plan, None);

        let cleared = build_turret_update_tick(
            &mut state,
            &mut unit_plans,
            &mut team_plans,
            false,
            true,
            false,
            false,
            None,
            &[],
            false,
            false,
            None,
            |_| false,
            |_| false,
            |_| false,
            |_| false,
            |_| true,
        );

        assert_eq!(
            cleared.action,
            BuildTurretUpdateAction::ClearInvalidFollowing
        );
        assert_eq!(state.following, None);
        assert!(unit_plans.is_empty());
    }

    #[test]
    fn build_turret_update_controlled_forgets_state_and_self_plan_is_removed() {
        let mut state = BuildTurretState {
            following: Some(9),
            last_plan: Some(BlockPlan::new(4, 4, 0, "duo", None)),
            ..BuildTurretState::default()
        };
        let mut unit_plans = VecDeque::from([
            BuildPlan::new_place(4, 4, 0, "build-tower"),
            BuildPlan::new_break(5, 5),
        ]);
        let mut team_plans = Vec::new();

        let step = build_turret_update_tick(
            &mut state,
            &mut unit_plans,
            &mut team_plans,
            true,
            true,
            true,
            true,
            Some(BuildPlan::new_place(1, 1, 0, "duo")),
            &[],
            false,
            false,
            Some((4, 4)),
            |_| false,
            |_| false,
            |_| false,
            |_| false,
            |_| true,
        );

        assert_eq!(step.action, BuildTurretUpdateAction::Controlled);
        assert_eq!(step.update_building, false);
        assert!(step.update_build_logic);
        assert_eq!(state.following, None);
        assert_eq!(state.last_plan, None);
        assert_eq!(
            step.removed_self_plans,
            vec![BuildPlan::new_place(4, 4, 0, "build-tower")]
        );
        assert_eq!(unit_plans, VecDeque::from([BuildPlan::new_break(5, 5)]));
    }

    #[test]
    fn build_turret_keeps_team_plan_queue_when_no_candidate_matches() {
        let mut plans = vec![
            crate::mindustry::game::BlockPlan::new(4, 4, 0, "duo", None),
            crate::mindustry::game::BlockPlan::new(5, 5, 0, "router", None),
        ];
        let original = plans.clone();

        assert_eq!(
            build_turret_first_fit_plan(
                &mut plans,
                |_| true,
                |_| true,
                |plan| plan.block == "missing",
            ),
            None
        );
        assert_eq!(plans, original);
    }

    #[test]
    fn build_turret_discards_invalid_current_plan_and_clears_last_plan() {
        let last_plan = crate::mindustry::game::BlockPlan {
            removed: true,
            ..crate::mindustry::game::BlockPlan::new(2, 2, 0, "router", None)
        };
        let mut state = BuildTurretState {
            last_plan: Some(last_plan),
            ..BuildTurretState::default()
        };
        let mut unit_plans =
            std::collections::VecDeque::from([BuildPlan::new_place(2, 2, 0, "router")]);

        let validation = build_turret_validate_current_plan(
            &mut state,
            &mut unit_plans,
            false,
            true,
            |_| true,
            |_| true,
        );

        assert_eq!(validation.action, BuildTurretPlanAction::DropInvalid);
        assert_eq!(
            validation.removed_plan,
            Some(BuildPlan::new_place(2, 2, 0, "router"))
        );
        assert_eq!(validation.remove_team_plan_at, None);
        assert!(unit_plans.is_empty());
        assert_eq!(state.last_plan, None);
    }

    #[test]
    fn build_turret_keeps_valid_current_plan_and_removes_conflicting_breaks() {
        let mut state = BuildTurretState {
            last_plan: Some(crate::mindustry::game::BlockPlan::new(4, 4, 1, "duo", None)),
            ..BuildTurretState::default()
        };
        let mut unit_plans =
            std::collections::VecDeque::from([BuildPlan::new_place(4, 4, 1, "duo")]);

        let keep = build_turret_validate_current_plan(
            &mut state,
            &mut unit_plans,
            false,
            false,
            |_| false,
            |plan| plan.block.as_deref() == Some("duo"),
        );

        assert_eq!(keep.action, BuildTurretPlanAction::Keep);
        assert_eq!(keep.removed_plan, None);
        assert_eq!(unit_plans.len(), 1);
        assert!(state.last_plan.is_some());

        let conflict = build_turret_validate_current_plan(
            &mut state,
            &mut unit_plans,
            true,
            false,
            |_| false,
            |_| true,
        );

        assert_eq!(conflict.action, BuildTurretPlanAction::DropConflictingBreak);
        assert_eq!(
            conflict.removed_plan,
            Some(BuildPlan::new_place(4, 4, 1, "duo"))
        );
        assert_eq!(conflict.remove_team_plan_at, Some((4, 4)));
        assert!(unit_plans.is_empty());
        assert!(state.last_plan.is_some());
    }

    #[test]
    fn shockwave_tower_update_matches_java_damage_edges() {
        let mut tower = ShockwaveTowerState {
            reload_counter: 90.0,
            heat: 0.0,
        };
        let mut targets = [100.0, 500.0, 10.0, 200.0];
        let fire = shockwave_tower_update(
            &mut tower,
            1.0,
            1.0,
            1.0,
            90.0,
            160.0,
            20.0,
            &mut targets,
            true,
            1.0,
        );
        assert!(fire.fired);
        assert_eq!(fire.wave_damage, 160.0);
        assert_eq!(fire.removed_targets, 2);
        assert_eq!(targets, [0.0, 340.0, 0.0, 40.0]);
        assert_eq!(tower.reload_counter, 0.0);
        assert!(tower.heat < 1.0);
        assert_eq!(shockwave_tower_progress(45.0, 90.0), 0.5);
        assert!(shockwave_tower_should_consume(45.0, 90.0));
    }
}
