pub mod turrets;

use std::io::{self, Read, Write};

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
    pub raw_plans: Vec<u8>,
}

impl Default for BuildTurretState {
    fn default() -> Self {
        Self {
            rotation: 90.0,
            warmup: 0.0,
            raw_plans: Vec::new(),
        }
    }
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
        let build = BuildTurretState {
            rotation: 45.0,
            warmup: 0.6,
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
