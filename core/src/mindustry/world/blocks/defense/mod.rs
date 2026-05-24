pub mod turrets;

use std::collections::{BTreeMap, VecDeque};
use std::io::{self, Read, Write};

use crate::mindustry::core::content_loader::ContentLoader;
use crate::mindustry::ctype::ContentId;
use crate::mindustry::entities::comp::UnitComp;
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::game::BlockPlan;
use crate::mindustry::io::{type_io, TeamId, TypeValue};
use crate::mindustry::logic::LAccess;
use crate::mindustry::r#type::UnitType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallState {
    pub hit: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallStatsPlan {
    pub base_deflect_chance: Option<f32>,
    pub lightning_chance_percent: Option<f32>,
    pub lightning_damage: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallIconRegion {
    Main,
    Variant1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallDestroySound {
    Keep,
    BlockExplodeWall,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallDrawPlan {
    pub draw_flash: bool,
    pub flash_alpha: f32,
    pub flash_size: f32,
    pub next_hit: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WallReflectAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallCollisionPlan {
    pub hit: f32,
    pub create_lightning: bool,
    pub lightning_rotation: f32,
    pub deflected: bool,
    pub reflect_axis: Option<WallReflectAxis>,
    pub bullet_time_add: f32,
    pub transfer_owner_and_team: bool,
    pub continue_collision: bool,
}

pub fn wall_stats_plan(
    chance_deflect: f32,
    lightning_chance: f32,
    lightning_damage: f32,
) -> WallStatsPlan {
    WallStatsPlan {
        base_deflect_chance: (chance_deflect > 0.0).then_some(chance_deflect),
        lightning_chance_percent: (lightning_chance > 0.0).then_some(lightning_chance * 100.0),
        lightning_damage: (lightning_chance > 0.0).then_some(lightning_damage),
    }
}

pub fn wall_init_destroy_sound(size: i32, destroy_sound_unset: bool) -> WallDestroySound {
    if size == 2 && destroy_sound_unset {
        WallDestroySound::BlockExplodeWall
    } else {
        WallDestroySound::Keep
    }
}

pub fn wall_icon_region(has_main_region: bool) -> WallIconRegion {
    if has_main_region {
        WallIconRegion::Main
    } else {
        WallIconRegion::Variant1
    }
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

pub fn wall_collision_reflect_axis(pen_x: f32, pen_y: f32) -> WallReflectAxis {
    if wall_reflect_x(pen_x, pen_y) {
        WallReflectAxis::X
    } else {
        WallReflectAxis::Y
    }
}

pub fn wall_draw_plan(
    flash_hit: bool,
    hit: f32,
    tile_size: f32,
    size: i32,
    delta: f32,
    paused: bool,
) -> WallDrawPlan {
    let draw_flash = flash_hit && hit >= 0.0001;
    WallDrawPlan {
        draw_flash,
        flash_alpha: if draw_flash { hit * 0.5 } else { 0.0 },
        flash_size: tile_size * size as f32,
        next_hit: if draw_flash {
            wall_draw_hit_decay(hit, delta, paused)
        } else {
            hit
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn wall_collision_plan(
    lightning_chance: f32,
    lightning_random: f32,
    bullet_rotation: f32,
    chance_deflect: f32,
    bullet_speed: f32,
    reflectable: bool,
    bullet_damage: f32,
    deflect_random: f32,
    pen_x: f32,
    pen_y: f32,
) -> WallCollisionPlan {
    let create_lightning = wall_should_lightning(lightning_chance, lightning_random);
    let deflected = wall_deflects_bullet(
        chance_deflect,
        bullet_speed,
        reflectable,
        bullet_damage,
        deflect_random,
    );
    let reflect_axis = deflected.then_some(wall_collision_reflect_axis(pen_x, pen_y));
    WallCollisionPlan {
        hit: 1.0,
        create_lightning,
        lightning_rotation: bullet_rotation + 180.0,
        deflected,
        reflect_axis,
        bullet_time_add: if deflected { 1.0 } else { 0.0 },
        transfer_owner_and_team: deflected,
        continue_collision: !deflected,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorState {
    pub open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorEffectKind {
    Open,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorControlPlan {
    pub configure: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorTappedPlan {
    pub configure: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoDoorTriggerRect {
    pub center_x: f32,
    pub center_y: f32,
    pub size: f32,
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

pub fn door_effect_for_current_open(open: bool) -> DoorEffectKind {
    if open {
        DoorEffectKind::Close
    } else {
        DoorEffectKind::Open
    }
}

pub fn door_origin_id(self_id: i32, chained_first: Option<i32>) -> i32 {
    chained_first.unwrap_or(self_id)
}

pub fn door_control_enabled_plan(
    open: bool,
    p1: f64,
    net_client: bool,
    units_in_tile: bool,
    origin_timer_ready: bool,
) -> DoorControlPlan {
    let should_open = p1.abs() > f64::EPSILON;
    let blocked =
        net_client || open == should_open || (units_in_tile && !should_open) || !origin_timer_ready;
    DoorControlPlan {
        configure: (!blocked).then_some(should_open),
    }
}

pub fn door_tapped_plan(
    open: bool,
    units_in_tile: bool,
    origin_timer_ready: bool,
) -> DoorTappedPlan {
    DoorTappedPlan {
        configure: door_tapped_should_configure(open, units_in_tile, origin_timer_ready)
            .then_some(!open),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorChainNode {
    pub id: i32,
    pub open: bool,
    pub units_in_tile: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorChainToggle {
    pub id: i32,
    pub open: bool,
    pub play_chain_effect: bool,
    pub update_pathfinder: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoorChainTogglePlan {
    pub play_origin_sound: bool,
    pub play_origin_effect: bool,
    pub toggles: Vec<DoorChainToggle>,
}

pub fn door_chain_toggle_plan(
    doors: impl IntoIterator<Item = DoorChainNode>,
    requested_open: bool,
    world_generating: bool,
    chain_effect: bool,
) -> DoorChainTogglePlan {
    let mut toggles = Vec::new();
    for door in doors {
        if (!requested_open && door.units_in_tile) || door.open == requested_open {
            continue;
        }
        toggles.push(DoorChainToggle {
            id: door.id,
            open: requested_open,
            play_chain_effect: chain_effect,
            update_pathfinder: !world_generating,
        });
    }

    DoorChainTogglePlan {
        play_origin_sound: !world_generating,
        play_origin_effect: !world_generating,
        toggles,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorRegion {
    Closed,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorDrawCommand {
    Region(DoorRegion),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorDrawPlan {
    pub command: DoorDrawCommand,
}

pub fn door_region_for_open(open: bool) -> DoorRegion {
    if open {
        DoorRegion::Open
    } else {
        DoorRegion::Closed
    }
}

pub fn door_plan_region(config_open: Option<bool>) -> DoorRegion {
    door_region_for_open(config_open.unwrap_or(false))
}

pub fn door_draw_plan(open: bool) -> DoorDrawPlan {
    DoorDrawPlan {
        command: DoorDrawCommand::Region(door_region_for_open(open)),
    }
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

pub fn auto_door_ground_check(is_grounded: bool, allow_leg_step: bool) -> bool {
    is_grounded && !allow_leg_step
}

pub fn auto_door_trigger_size(block_size: i32, tile_size: f32, trigger_margin: f32) -> f32 {
    block_size as f32 * tile_size + trigger_margin * 2.0
}

pub fn auto_door_trigger_rect(
    center_x: f32,
    center_y: f32,
    block_size: i32,
    tile_size: f32,
    trigger_margin: f32,
) -> AutoDoorTriggerRect {
    AutoDoorTriggerRect {
        center_x,
        center_y,
        size: auto_door_trigger_size(block_size, tile_size, trigger_margin),
    }
}

pub fn auto_door_remote_toggle_valid(tile_exists: bool, is_auto_door_build: bool) -> bool {
    tile_exists && is_auto_door_build
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoDoorUpdatePlan {
    pub should_scan_units: bool,
    pub should_open: bool,
    pub send_toggle: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoDoorSetOpenPlan {
    pub open: bool,
    pub update_pathfinder: bool,
    pub play_effect: bool,
    pub play_sound: bool,
}

pub fn auto_door_update_plan(
    open: bool,
    timer_ready: bool,
    net_client: bool,
    ground_units_in_trigger: bool,
) -> AutoDoorUpdatePlan {
    let should_scan_units = timer_ready && !net_client;
    let should_open = auto_door_should_open(ground_units_in_trigger);
    AutoDoorUpdatePlan {
        should_scan_units,
        should_open,
        send_toggle: (should_scan_units && open != should_open).then_some(should_open),
    }
}

pub fn auto_door_set_open_plan(open: bool, was_visible: bool) -> AutoDoorSetOpenPlan {
    AutoDoorSetOpenPlan {
        open,
        update_pathfinder: true,
        play_effect: was_visible,
        play_sound: was_visible,
    }
}

pub fn shock_mine_should_trigger(enabled: bool, same_team: bool, timer_ready: bool) -> bool {
    enabled && !same_team && timer_ready
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShockMineStatsPlan {
    pub tendrils: i32,
    pub damage_fixed_decimals: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockMineDrawPlan {
    pub draw_base: bool,
    pub draw_team_top: bool,
    pub team_alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShockMineTriggerPlan {
    pub triggered: bool,
    pub self_damage: f32,
    pub lightning_angles: Vec<f32>,
    pub lightning_damage: f32,
    pub lightning_length: i32,
    pub bullet_angles: Vec<f32>,
}

pub fn shock_mine_stats_plan(tendrils: i32) -> ShockMineStatsPlan {
    ShockMineStatsPlan {
        tendrils,
        damage_fixed_decimals: 2,
    }
}

pub fn shock_mine_stats_text(tendrils: i32, damage: f32) -> String {
    format!(
        "[white]{tendrils}x[lightgray] lightning ~ [white]{}[lightgray] damage",
        format_fixed_trimmed(damage, 2)
    )
}

pub fn shock_mine_draw_plan(team_alpha: f32) -> ShockMineDrawPlan {
    ShockMineDrawPlan {
        draw_base: true,
        draw_team_top: true,
        team_alpha,
    }
}

pub fn shock_mine_lightning_angles(tendrils: i32, random_angles: &[f32]) -> Vec<f32> {
    if tendrils <= 0 {
        return Vec::new();
    }
    (0..tendrils)
        .map(|index| random_angles.get(index as usize).copied().unwrap_or(0.0))
        .collect()
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

#[allow(clippy::too_many_arguments)]
pub fn shock_mine_trigger_plan(
    enabled: bool,
    same_team: bool,
    timer_ready: bool,
    tile_damage: f32,
    damage: f32,
    length: i32,
    tendrils: i32,
    lightning_random_angles: &[f32],
    has_bullet: bool,
    shots: i32,
    bullet_inaccuracy_offsets: &[f32],
) -> ShockMineTriggerPlan {
    let triggered = shock_mine_should_trigger(enabled, same_team, timer_ready);
    ShockMineTriggerPlan {
        triggered,
        self_damage: if triggered { tile_damage } else { 0.0 },
        lightning_angles: if triggered {
            shock_mine_lightning_angles(tendrils, lightning_random_angles)
        } else {
            Vec::new()
        },
        lightning_damage: if triggered { damage } else { 0.0 },
        lightning_length: if triggered { length } else { 0 },
        bullet_angles: if triggered && has_bullet {
            shock_mine_bullet_angles(shots, bullet_inaccuracy_offsets)
        } else {
            Vec::new()
        },
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectorTargetFilter {
    AnyBlock,
    CanOverdrive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectorPlacementPlan {
    pub center_x: f32,
    pub center_y: f32,
    pub real_range: f32,
    pub selected_alpha: f32,
    pub target_filter: ProjectorTargetFilter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectorSelectPlan {
    pub real_range: f32,
    pub selected_alpha: f32,
    pub target_filter: ProjectorTargetFilter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectorLightPlan {
    pub radius: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MendProjectorDrawCommand {
    SetColor,
    DrawTop,
    ResetAlpha,
    StrokeSquare,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MendProjectorDrawPlan {
    pub commands: &'static [MendProjectorDrawCommand],
    pub phase_lerp: f32,
    pub top_alpha: f32,
    pub cycle: f32,
    pub stroke: f32,
    pub square_radius: f32,
}

const MEND_PROJECTOR_DRAW_COMMANDS: &[MendProjectorDrawCommand] = &[
    MendProjectorDrawCommand::SetColor,
    MendProjectorDrawCommand::DrawTop,
    MendProjectorDrawCommand::ResetAlpha,
    MendProjectorDrawCommand::StrokeSquare,
    MendProjectorDrawCommand::Reset,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MendProjectorPulsePlan {
    pub scan_targets: bool,
    pub play_sound: bool,
}

pub fn mend_projector_repair_time_seconds(reload: f32, heal_percent: f32) -> f32 {
    100.0 / heal_percent * reload / 60.0
}

pub fn mend_projector_range_blocks(range: f32, tile_size: f32) -> f32 {
    range / tile_size
}

pub fn mend_projector_booster_multiplier(heal_percent: f32, phase_boost: f32) -> f32 {
    (phase_boost + heal_percent) / heal_percent
}

pub fn mend_projector_real_range(range: f32, phase_heat: f32, phase_range_boost: f32) -> f32 {
    range + phase_heat * phase_range_boost
}

pub fn mend_projector_heal_fraction(
    heal_percent: f32,
    phase_heat: f32,
    phase_boost: f32,
    efficiency: f32,
) -> f32 {
    (heal_percent + phase_heat * phase_boost) / 100.0 * efficiency
}

pub fn mend_projector_progress(charge: f32, reload: f32) -> f32 {
    (charge / reload).clamp(0.0, 1.0)
}

pub fn mend_projector_sense(
    sensor: LAccess,
    state: &MendProjectorState,
    reload: f32,
) -> Option<f64> {
    match sensor {
        LAccess::Progress => Some(mend_projector_progress(state.charge, reload) as f64),
        _ => None,
    }
}

pub fn mend_projector_should_consume_optional(
    optional_efficiency: f32,
    timer_ready: bool,
    can_heal: bool,
) -> bool {
    optional_efficiency > 0.0 && timer_ready && can_heal
}

pub fn mend_projector_pulse_plan(fired: bool, healed_any: bool) -> MendProjectorPulsePlan {
    MendProjectorPulsePlan {
        scan_targets: fired,
        play_sound: fired && healed_any,
    }
}

pub fn mend_projector_place_plan(
    tile_x: i32,
    tile_y: i32,
    tile_size: f32,
    offset: f32,
    range: f32,
    time: f32,
) -> ProjectorPlacementPlan {
    ProjectorPlacementPlan {
        center_x: tile_x as f32 * tile_size + offset,
        center_y: tile_y as f32 * tile_size + offset,
        real_range: range,
        selected_alpha: absin_time(time, 4.0, 1.0),
        target_filter: ProjectorTargetFilter::AnyBlock,
    }
}

pub fn mend_projector_select_plan(
    state: &MendProjectorState,
    range: f32,
    phase_range_boost: f32,
    time: f32,
) -> ProjectorSelectPlan {
    ProjectorSelectPlan {
        real_range: mend_projector_real_range(range, state.phase_heat, phase_range_boost),
        selected_alpha: absin_time(time, 4.0, 1.0),
        target_filter: ProjectorTargetFilter::AnyBlock,
    }
}

pub fn mend_projector_light_plan(
    state: &MendProjectorState,
    light_radius: f32,
) -> ProjectorLightPlan {
    ProjectorLightPlan {
        radius: light_radius * state.smooth_efficiency,
        alpha: 0.7 * state.smooth_efficiency,
    }
}

pub fn mend_projector_draw_plan(
    state: &MendProjectorState,
    time: f32,
    size: i32,
    tile_size: f32,
) -> MendProjectorDrawPlan {
    let cycle = projector_cycle(time);
    let half_size = size as f32 * tile_size / 2.0;
    MendProjectorDrawPlan {
        commands: MEND_PROJECTOR_DRAW_COMMANDS,
        phase_lerp: state.phase_heat,
        top_alpha: state.heat * absin_time(time, 50.0 / (std::f32::consts::PI * 2.0), 1.0) * 0.5,
        cycle,
        stroke: (2.0 * cycle + 0.2) * state.heat,
        square_radius: (1.0 + (1.0 - cycle) * half_size).min(half_size),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn mend_projector_update_with_timer(
    state: &mut MendProjectorState,
    efficiency: f32,
    optional_efficiency: f32,
    timer_ready: bool,
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
    let real_range = mend_projector_real_range(range, state.phase_heat, phase_range_boost);
    let heal_fraction =
        mend_projector_heal_fraction(heal_percent, state.phase_heat, phase_boost, efficiency);
    let fired = state.charge >= reload && can_heal;
    if fired {
        state.charge = 0.0;
    }
    MendProjectorUpdate {
        fired,
        real_range,
        heal_fraction,
        should_consume_optional: mend_projector_should_consume_optional(
            optional_efficiency,
            timer_ready,
            can_heal,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
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
    mend_projector_update_with_timer(
        state,
        efficiency,
        optional_efficiency,
        true,
        can_heal,
        delta,
        reload,
        range,
        heal_percent,
        phase_boost,
        phase_range_boost,
    )
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverdriveProjectorDrawCommand {
    SetColor,
    DrawTop,
    ResetAlpha,
    StrokeLineLoop,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverdriveProjectorDrawPlan {
    pub commands: &'static [OverdriveProjectorDrawCommand],
    pub phase_lerp: f32,
    pub top_alpha: f32,
    pub cycle: f32,
    pub stroke: f32,
    pub line_radius: f32,
    pub line_width: f32,
    pub mirrored_points: bool,
}

const OVERDRIVE_PROJECTOR_DRAW_COMMANDS: &[OverdriveProjectorDrawCommand] = &[
    OverdriveProjectorDrawCommand::SetColor,
    OverdriveProjectorDrawCommand::DrawTop,
    OverdriveProjectorDrawCommand::ResetAlpha,
    OverdriveProjectorDrawCommand::StrokeLineLoop,
    OverdriveProjectorDrawCommand::Reset,
];

pub fn overdrive_speed_increase_percent(speed_boost: f32) -> f32 {
    speed_boost * 100.0 - 100.0
}

pub fn overdrive_production_time_seconds(use_time: f32) -> f32 {
    use_time / 60.0
}

pub fn overdrive_real_range(range: f32, phase_heat: f32, phase_range_boost: f32) -> f32 {
    range + phase_heat * phase_range_boost
}

pub fn overdrive_boost_multiplier_limit(
    has_boost: bool,
    speed_boost: f32,
    speed_boost_phase: f32,
) -> f32 {
    if has_boost {
        speed_boost + speed_boost_phase
    } else {
        speed_boost
    }
}

pub fn overdrive_projector_bar_fraction(
    real_boost: f32,
    has_boost: bool,
    speed_boost: f32,
    speed_boost_phase: f32,
) -> f32 {
    let limit = overdrive_boost_multiplier_limit(has_boost, speed_boost, speed_boost_phase);
    if limit == 0.0 {
        0.0
    } else {
        real_boost / limit
    }
}

pub fn overdrive_projector_place_plan(
    tile_x: i32,
    tile_y: i32,
    tile_size: f32,
    offset: f32,
    range: f32,
    time: f32,
) -> ProjectorPlacementPlan {
    ProjectorPlacementPlan {
        center_x: tile_x as f32 * tile_size + offset,
        center_y: tile_y as f32 * tile_size + offset,
        real_range: range,
        selected_alpha: absin_time(time, 4.0, 1.0),
        target_filter: ProjectorTargetFilter::CanOverdrive,
    }
}

pub fn overdrive_projector_select_plan(
    state: &OverdriveProjectorState,
    range: f32,
    phase_range_boost: f32,
    time: f32,
) -> ProjectorSelectPlan {
    ProjectorSelectPlan {
        real_range: overdrive_real_range(range, state.phase_heat, phase_range_boost),
        selected_alpha: absin_time(time, 4.0, 1.0),
        target_filter: ProjectorTargetFilter::CanOverdrive,
    }
}

pub fn overdrive_projector_light_plan(
    state: &OverdriveProjectorState,
    light_radius: f32,
) -> ProjectorLightPlan {
    ProjectorLightPlan {
        radius: light_radius * state.smooth_efficiency,
        alpha: 0.7 * state.smooth_efficiency,
    }
}

pub fn overdrive_projector_draw_plan(
    state: &OverdriveProjectorState,
    time: f32,
    size: i32,
    tile_size: f32,
) -> OverdriveProjectorDrawPlan {
    let cycle = projector_cycle(time);
    let block_half = size as f32 * tile_size / 2.0;
    OverdriveProjectorDrawPlan {
        commands: OVERDRIVE_PROJECTOR_DRAW_COMMANDS,
        phase_lerp: state.phase_heat,
        top_alpha: state.heat * absin_time(time, 50.0 / (std::f32::consts::PI * 2.0), 1.0) * 0.5,
        cycle,
        stroke: (2.0 * cycle + 0.1) * state.heat,
        line_radius: (clamp_unit(2.0 - cycle * 2.0) * block_half - cycle - 0.2).max(0.0),
        line_width: clamp_unit(0.5 - cycle) * size as f32 * tile_size,
        mirrored_points: cycle < 0.5,
    }
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
        real_range: overdrive_real_range(range, state.phase_heat, phase_range_boost),
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

pub fn force_projector_sense(
    sensor: LAccess,
    state: &ForceProjectorState,
    shield_health: f32,
    phase_shield_boost: f32,
) -> Option<f64> {
    match sensor {
        LAccess::Heat => Some(state.buildup as f64),
        LAccess::Shield => Some(force_projector_shield(
            state.broken,
            shield_health,
            phase_shield_boost,
            state.phase_heat,
            state.buildup,
        ) as f64),
        _ => None,
    }
}

pub fn force_projector_set_shield(
    state: &mut ForceProjectorState,
    value: f32,
    shield_health: f32,
    phase_shield_boost: f32,
) {
    state.buildup = (shield_health + phase_shield_boost * state.phase_heat - value).max(0.0);
}

pub fn force_projector_outputs_items() -> bool {
    false
}

pub fn force_projector_should_ambient_sound(
    state: &ForceProjectorState,
    radius: f32,
    phase_radius_boost: f32,
) -> bool {
    !state.broken
        && force_projector_real_radius(radius, state.phase_heat, phase_radius_boost, state.radscl)
            > 1.0
}

pub fn force_projector_in_fog_to() -> bool {
    false
}

pub fn force_projector_picked_up(state: &mut ForceProjectorState) {
    state.radscl = 0.0;
    state.warmup = 0.0;
}

pub fn force_projector_overwrote(
    state: &mut ForceProjectorState,
    previous_same_force_block: bool,
    previous_broken: bool,
    previous_buildup: f32,
) -> bool {
    if !previous_same_force_block {
        return false;
    }
    state.broken = previous_broken;
    state.buildup = previous_buildup;
    true
}

pub fn force_projector_bar_fraction(
    state: &ForceProjectorState,
    shield_health: f32,
    phase_shield_boost: f32,
) -> f32 {
    if state.broken {
        return 0.0;
    }
    let capacity = shield_health + phase_shield_boost * state.phase_heat;
    if capacity == 0.0 {
        0.0
    } else {
        1.0 - state.buildup / capacity
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceProjectorBulletAbsorb {
    pub absorbed: bool,
    pub hit_effect: bool,
    pub sound_effect: bool,
    pub buildup_added: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceProjectorRemovedPlan {
    pub call_super_removed: bool,
    pub play_force_shrink: bool,
    pub effect_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceProjectorDeflectPlan {
    pub active: bool,
    pub real_radius: f32,
    pub bounds_x: f32,
    pub bounds_y: f32,
    pub bounds_width: f32,
    pub bounds_height: f32,
}

pub fn force_projector_absorb_bullet(
    state: &mut ForceProjectorState,
    enemy_team: bool,
    absorbable: bool,
    already_absorbed: bool,
    inside_polygon: bool,
    bullet_shield_damage: f32,
) -> ForceProjectorBulletAbsorb {
    let absorbed = !state.broken && enemy_team && absorbable && !already_absorbed && inside_polygon;
    if absorbed {
        state.hit = 1.0;
        state.buildup += bullet_shield_damage;
        ForceProjectorBulletAbsorb {
            absorbed: true,
            hit_effect: true,
            sound_effect: true,
            buildup_added: bullet_shield_damage,
        }
    } else {
        ForceProjectorBulletAbsorb {
            absorbed: false,
            hit_effect: false,
            sound_effect: false,
            buildup_added: 0.0,
        }
    }
}

pub fn force_projector_on_removed_plan(
    state: &ForceProjectorState,
    radius: f32,
    phase_radius_boost: f32,
) -> ForceProjectorRemovedPlan {
    let effect_radius =
        force_projector_real_radius(radius, state.phase_heat, phase_radius_boost, state.radscl);
    ForceProjectorRemovedPlan {
        call_super_removed: true,
        play_force_shrink: !state.broken && effect_radius > 1.0,
        effect_radius,
    }
}

pub fn force_projector_deflect_plan(
    state: &ForceProjectorState,
    radius: f32,
    phase_radius_boost: f32,
) -> ForceProjectorDeflectPlan {
    let real_radius =
        force_projector_real_radius(radius, state.phase_heat, phase_radius_boost, state.radscl);
    let active = real_radius > 0.0 && !state.broken;
    ForceProjectorDeflectPlan {
        active,
        real_radius,
        bounds_x: if active { -real_radius } else { 0.0 },
        bounds_y: if active { -real_radius } else { 0.0 },
        bounds_width: if active { real_radius * 2.0 } else { 0.0 },
        bounds_height: if active { real_radius * 2.0 } else { 0.0 },
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceProjectorDrawCommand {
    TopAdditive,
    ResetTopAdditive,
    SetShieldColor,
    SetAnimatedShieldLayer,
    FillAnimatedPoly,
    SetStaticShieldLayer,
    StrokeStaticPoly,
    ResetStaticShield,
    ResetFinal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceProjectorDrawPlan {
    pub commands: &'static [ForceProjectorDrawCommand],
    pub radius: f32,
    pub sides: i32,
    pub shield_rotation: f32,
    pub hit_alpha: f32,
    pub shield_layer_offset: f32,
    pub top_alpha: f32,
    pub fill_alpha: f32,
    pub stroke: f32,
}

const FORCE_PROJECTOR_DRAW_TOP_ANIMATED: &[ForceProjectorDrawCommand] = &[
    ForceProjectorDrawCommand::TopAdditive,
    ForceProjectorDrawCommand::ResetTopAdditive,
    ForceProjectorDrawCommand::SetShieldColor,
    ForceProjectorDrawCommand::SetAnimatedShieldLayer,
    ForceProjectorDrawCommand::FillAnimatedPoly,
    ForceProjectorDrawCommand::ResetFinal,
];

const FORCE_PROJECTOR_DRAW_ANIMATED: &[ForceProjectorDrawCommand] = &[
    ForceProjectorDrawCommand::SetShieldColor,
    ForceProjectorDrawCommand::SetAnimatedShieldLayer,
    ForceProjectorDrawCommand::FillAnimatedPoly,
    ForceProjectorDrawCommand::ResetFinal,
];

const FORCE_PROJECTOR_DRAW_TOP_STATIC: &[ForceProjectorDrawCommand] = &[
    ForceProjectorDrawCommand::TopAdditive,
    ForceProjectorDrawCommand::ResetTopAdditive,
    ForceProjectorDrawCommand::SetShieldColor,
    ForceProjectorDrawCommand::SetStaticShieldLayer,
    ForceProjectorDrawCommand::StrokeStaticPoly,
    ForceProjectorDrawCommand::ResetStaticShield,
    ForceProjectorDrawCommand::ResetFinal,
];

const FORCE_PROJECTOR_DRAW_STATIC: &[ForceProjectorDrawCommand] = &[
    ForceProjectorDrawCommand::SetShieldColor,
    ForceProjectorDrawCommand::SetStaticShieldLayer,
    ForceProjectorDrawCommand::StrokeStaticPoly,
    ForceProjectorDrawCommand::ResetStaticShield,
    ForceProjectorDrawCommand::ResetFinal,
];

const FORCE_PROJECTOR_DRAW_TOP_ONLY: &[ForceProjectorDrawCommand] = &[
    ForceProjectorDrawCommand::TopAdditive,
    ForceProjectorDrawCommand::ResetTopAdditive,
    ForceProjectorDrawCommand::ResetFinal,
];

const FORCE_PROJECTOR_DRAW_RESET_ONLY: &[ForceProjectorDrawCommand] =
    &[ForceProjectorDrawCommand::ResetFinal];

pub fn force_projector_draw_plan(
    state: &ForceProjectorState,
    radius: f32,
    phase_radius_boost: f32,
    shield_health: f32,
    sides: i32,
    shield_rotation: f32,
    animate_shields: bool,
) -> ForceProjectorDrawPlan {
    let real_radius =
        force_projector_real_radius(radius, state.phase_heat, phase_radius_boost, state.radscl);
    let has_top = state.buildup > 0.0;
    let has_shield = !state.broken && real_radius > 0.001;
    let top_alpha = if has_top && shield_health != 0.0 {
        state.buildup / shield_health * 0.75
    } else {
        0.0
    };
    ForceProjectorDrawPlan {
        commands: match (has_top, has_shield, animate_shields) {
            (true, true, true) => FORCE_PROJECTOR_DRAW_TOP_ANIMATED,
            (false, true, true) => FORCE_PROJECTOR_DRAW_ANIMATED,
            (true, true, false) => FORCE_PROJECTOR_DRAW_TOP_STATIC,
            (false, true, false) => FORCE_PROJECTOR_DRAW_STATIC,
            (true, false, _) => FORCE_PROJECTOR_DRAW_TOP_ONLY,
            (false, false, _) => FORCE_PROJECTOR_DRAW_RESET_ONLY,
        },
        radius: real_radius,
        sides,
        shield_rotation,
        hit_alpha: state.hit.clamp(0.0, 1.0),
        shield_layer_offset: if has_shield && animate_shields {
            0.001 * state.hit
        } else {
            0.0
        },
        top_alpha,
        fill_alpha: if has_shield && !animate_shields {
            0.09 + (0.08 * state.hit).clamp(0.0, 1.0)
        } else if has_shield {
            1.0
        } else {
            0.0
        },
        stroke: if has_shield && !animate_shields {
            1.5
        } else {
            0.0
        },
    }
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
pub struct RegenProjectorState {
    pub warmup: f32,
    pub total_time: f32,
    pub optional_timer: f32,
    pub any_targets: bool,
    pub did_regen: bool,
}

impl Default for RegenProjectorState {
    fn default() -> Self {
        Self {
            warmup: 0.0,
            total_time: 0.0,
            optional_timer: 0.0,
            any_targets: false,
            did_regen: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegenProjectorUpdatePlan {
    pub suppressed: bool,
    pub any_targets: bool,
    pub consume_optional: bool,
    pub heal_amount_percent: f32,
    pub warmup: f32,
    pub total_time: f32,
    pub optional_timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegenProjectorStatsPlan {
    pub time_period: f32,
    pub repair_time_seconds: i32,
    pub range_blocks: i32,
    pub booster_multiplier: Option<f32>,
    pub booster_range_boost: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegenProjectorDrawCommand {
    DrawerDraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegenProjectorDrawPlan {
    pub commands: &'static [RegenProjectorDrawCommand],
    pub draw_region: bool,
}

const REGEN_PROJECTOR_DRAW_COMMANDS: &[RegenProjectorDrawCommand] =
    &[RegenProjectorDrawCommand::DrawerDraw];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegenProjectorRangePlan {
    pub center_x: f32,
    pub center_y: f32,
    pub square_size: f32,
    pub selected_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegenProjectorApplyPlan {
    pub apply_mend_map: bool,
    pub clear_mend_map: bool,
}

pub fn regen_projector_repair_time_seconds(heal_percent_per_frame: f32) -> f32 {
    1.0 / (heal_percent_per_frame / 100.0) / 60.0
}

pub fn regen_projector_repair_time_stat_seconds(heal_percent_per_frame: f32) -> i32 {
    regen_projector_repair_time_seconds(heal_percent_per_frame) as i32
}

pub fn regen_projector_range_blocks(range: i32) -> i32 {
    range
}

pub fn regen_projector_booster_multiplier(optional_multiplier: f32) -> f32 {
    optional_multiplier
}

pub fn regen_projector_stats_plan(
    optional_use_time: f32,
    heal_percent_per_frame: f32,
    range: i32,
    optional_multiplier: f32,
    has_item_booster: bool,
) -> RegenProjectorStatsPlan {
    RegenProjectorStatsPlan {
        time_period: optional_use_time,
        repair_time_seconds: regen_projector_repair_time_stat_seconds(heal_percent_per_frame),
        range_blocks: regen_projector_range_blocks(range),
        booster_multiplier: has_item_booster.then_some(optional_multiplier),
        booster_range_boost: 0.0,
    }
}

pub fn regen_projector_square_size(range: i32, tile_size: f32) -> f32 {
    range as f32 * tile_size
}

pub fn regen_projector_should_consume(any_targets: bool) -> bool {
    any_targets
}

pub fn regen_projector_effect_chance_delta(effect_chance: f32, block_size: i32, delta: f32) -> f32 {
    (effect_chance * block_size as f32 * block_size as f32 * delta).clamp(0.0, 1.0)
}

pub fn regen_projector_effect_offset_limit(block_size: i32, tile_size: f32) -> f32 {
    block_size as f32 * tile_size / 2.0 - 1.0
}

pub fn regen_projector_should_emit_effect(
    previous_mend_value: f32,
    effect_chance_delta: f32,
    random: f32,
) -> bool {
    previous_mend_value <= 0.0 && random < effect_chance_delta
}

pub fn regen_projector_place_plan(
    tile_x: i32,
    tile_y: i32,
    tile_size: f32,
    offset: f32,
    range: i32,
    time: f32,
) -> RegenProjectorRangePlan {
    RegenProjectorRangePlan {
        center_x: tile_x as f32 * tile_size + offset,
        center_y: tile_y as f32 * tile_size + offset,
        square_size: regen_projector_square_size(range, tile_size),
        selected_alpha: absin_time(time, 4.0, 1.0),
    }
}

pub fn regen_projector_select_plan(
    x: f32,
    y: f32,
    range: i32,
    tile_size: f32,
    time: f32,
) -> RegenProjectorRangePlan {
    RegenProjectorRangePlan {
        center_x: x,
        center_y: y,
        square_size: regen_projector_square_size(range, tile_size),
        selected_alpha: absin_time(time, 4.0, 1.0),
    }
}

pub fn regen_projector_light_plan(state: &RegenProjectorState) -> ProjectorLightPlan {
    ProjectorLightPlan {
        radius: 0.0,
        alpha: 0.0 * state.warmup,
    }
}

pub fn regen_projector_draw_plan() -> RegenProjectorDrawPlan {
    RegenProjectorDrawPlan {
        commands: REGEN_PROJECTOR_DRAW_COMMANDS,
        draw_region: true,
    }
}

pub fn regen_projector_apply_plan(
    last_update_frame: i64,
    current_update_id: i64,
) -> RegenProjectorApplyPlan {
    let apply_mend_map = last_update_frame != current_update_id;
    RegenProjectorApplyPlan {
        apply_mend_map,
        clear_mend_map: apply_mend_map,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn regen_projector_update(
    state: &mut RegenProjectorState,
    suppressed: bool,
    damaged_targets: bool,
    efficiency: f32,
    edelta: f32,
    delta: f32,
    optional_efficiency: f32,
    optional_use_time: f32,
    optional_multiplier: f32,
    heal_percent: f32,
) -> RegenProjectorUpdatePlan {
    state.warmup = approach_delta(
        state.warmup,
        if state.did_regen { 1.0 } else { 0.0 },
        1.0 / 70.0,
    );
    state.total_time += state.warmup * delta;
    state.did_regen = false;
    state.any_targets = false;

    if suppressed {
        return RegenProjectorUpdatePlan {
            suppressed: true,
            any_targets: false,
            consume_optional: false,
            heal_amount_percent: 0.0,
            warmup: state.warmup,
            total_time: state.total_time,
            optional_timer: state.optional_timer,
        };
    }

    state.any_targets = damaged_targets;
    let mut consume_optional = false;
    let mut heal_amount_percent = 0.0;

    if efficiency > 0.0 {
        state.optional_timer += edelta * optional_efficiency;
        if state.optional_timer >= optional_use_time {
            consume_optional = true;
            state.optional_timer = 0.0;
        }
        heal_amount_percent = lerp(1.0, optional_multiplier, optional_efficiency) * heal_percent;
        state.did_regen = damaged_targets;
    }

    RegenProjectorUpdatePlan {
        suppressed: false,
        any_targets: state.any_targets,
        consume_optional,
        heal_amount_percent,
        warmup: state.warmup,
        total_time: state.total_time,
        optional_timer: state.optional_timer,
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RegenProjectorMendMap {
    pub entries: BTreeMap<i32, f32>,
}

impl RegenProjectorMendMap {
    pub fn record(&mut self, pos: i32, amount: f32, missing_health: f32) {
        let capped = amount.min(missing_health);
        let entry = self.entries.entry(pos).or_insert(0.0);
        *entry = (*entry).max(capped);
    }

    pub fn drain(&mut self) -> Vec<(i32, f32)> {
        std::mem::take(&mut self.entries).into_iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseShieldState {
    pub broken: bool,
    pub hit: f32,
    pub smooth_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseShieldRangePlan {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseShieldInteractionPlan {
    pub active: bool,
    pub bullet_min_x: f32,
    pub bullet_min_y: f32,
    pub bullet_size: f32,
    pub unit_range: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseShieldTintPlan {
    pub use_team_color: bool,
    pub hit_alpha: u8,
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

pub fn base_shield_clip_radius(radius: f32) -> f32 {
    radius
}

pub fn base_shield_radius(state: &BaseShieldState) -> f32 {
    state.smooth_radius
}

pub fn base_shield_in_fog_to() -> bool {
    false
}

pub fn base_shield_should_interact(radius: f32) -> bool {
    radius > 1.0
}

pub fn base_shield_should_absorb_bullet(
    enemy_team: bool,
    absorbable: bool,
    within_radius: bool,
) -> bool {
    enemy_team && absorbable && within_radius
}

pub fn base_shield_tint_plan(shield_color_present: bool, hit: f32) -> BaseShieldTintPlan {
    BaseShieldTintPlan {
        use_team_color: !shield_color_present,
        hit_alpha: (hit.clamp(0.0, 1.0) * 255.0).round() as u8,
    }
}

pub fn base_shield_place_plan(
    tile_x: i32,
    tile_y: i32,
    tile_size: f32,
    offset: f32,
    radius: f32,
) -> BaseShieldRangePlan {
    BaseShieldRangePlan {
        center_x: tile_x as f32 * tile_size + offset,
        center_y: tile_y as f32 * tile_size + offset,
        radius,
    }
}

pub fn base_shield_select_plan(x: f32, y: f32, radius: f32) -> BaseShieldRangePlan {
    BaseShieldRangePlan {
        center_x: x,
        center_y: y,
        radius,
    }
}

pub fn base_shield_interaction_plan(x: f32, y: f32, radius: f32) -> BaseShieldInteractionPlan {
    if base_shield_should_interact(radius) {
        BaseShieldInteractionPlan {
            active: true,
            bullet_min_x: x - radius,
            bullet_min_y: y - radius,
            bullet_size: radius * 2.0,
            unit_range: radius + 10.0,
        }
    } else {
        BaseShieldInteractionPlan {
            active: false,
            bullet_min_x: x,
            bullet_min_y: y,
            bullet_size: 0.0,
            unit_range: 0.0,
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseShieldDrawCommand {
    SetShieldLayer,
    SetShieldColor,
    FillAnimatedPoly,
    StrokeStaticPoly,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseShieldDrawPlan {
    pub commands: &'static [BaseShieldDrawCommand],
    pub radius: f32,
    pub sides: i32,
    pub hit_alpha: f32,
    pub fill_alpha: f32,
    pub stroke: f32,
}

const BASE_SHIELD_DRAW_COMMANDS_ANIMATED: &[BaseShieldDrawCommand] = &[
    BaseShieldDrawCommand::SetShieldLayer,
    BaseShieldDrawCommand::SetShieldColor,
    BaseShieldDrawCommand::FillAnimatedPoly,
    BaseShieldDrawCommand::Reset,
];

const BASE_SHIELD_DRAW_COMMANDS_STATIC: &[BaseShieldDrawCommand] = &[
    BaseShieldDrawCommand::SetShieldLayer,
    BaseShieldDrawCommand::SetShieldColor,
    BaseShieldDrawCommand::StrokeStaticPoly,
    BaseShieldDrawCommand::Reset,
];

const BASE_SHIELD_DRAW_COMMANDS_BROKEN: &[BaseShieldDrawCommand] = &[BaseShieldDrawCommand::Reset];

pub fn base_shield_draw_plan(
    broken: bool,
    radius: f32,
    sides: i32,
    hit: f32,
    animate_shields: bool,
) -> BaseShieldDrawPlan {
    let hit_alpha = hit.clamp(0.0, 1.0);
    BaseShieldDrawPlan {
        commands: if broken {
            BASE_SHIELD_DRAW_COMMANDS_BROKEN
        } else if animate_shields {
            BASE_SHIELD_DRAW_COMMANDS_ANIMATED
        } else {
            BASE_SHIELD_DRAW_COMMANDS_STATIC
        },
        radius,
        sides,
        hit_alpha,
        fill_alpha: if broken {
            0.0
        } else if animate_shields {
            1.0
        } else {
            0.09 + (0.08 * hit).clamp(0.0, 1.0)
        },
        stroke: if broken || animate_shields { 0.0 } else { 1.5 },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShieldWallDrawCommand {
    Region,
    SetShieldLayer,
    SetShieldColor,
    FillAnimatedSquare,
    StrokeStaticSquare,
    ResetShield,
    Glow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldWallDrawPlan {
    pub commands: &'static [ShieldWallDrawCommand],
    pub radius: f32,
    pub hit_alpha: f32,
    pub fill_alpha: f32,
    pub stroke: f32,
    pub glow_alpha: f32,
}

const SHIELD_WALL_DRAW_COMMANDS_ANIMATED: &[ShieldWallDrawCommand] = &[
    ShieldWallDrawCommand::Region,
    ShieldWallDrawCommand::SetShieldLayer,
    ShieldWallDrawCommand::SetShieldColor,
    ShieldWallDrawCommand::FillAnimatedSquare,
    ShieldWallDrawCommand::ResetShield,
    ShieldWallDrawCommand::Glow,
];

const SHIELD_WALL_DRAW_COMMANDS_STATIC: &[ShieldWallDrawCommand] = &[
    ShieldWallDrawCommand::Region,
    ShieldWallDrawCommand::SetShieldLayer,
    ShieldWallDrawCommand::SetShieldColor,
    ShieldWallDrawCommand::StrokeStaticSquare,
    ShieldWallDrawCommand::ResetShield,
    ShieldWallDrawCommand::Glow,
];

const SHIELD_WALL_DRAW_COMMANDS_REGION: &[ShieldWallDrawCommand] = &[ShieldWallDrawCommand::Region];

pub fn shield_wall_draw_plan(
    shield_radius: f32,
    size: i32,
    tile_size: f32,
    hit: f32,
    animate_shields: bool,
    glow_mag: f32,
    absin: f32,
) -> ShieldWallDrawPlan {
    let has_shield = shield_radius > 0.0;
    ShieldWallDrawPlan {
        commands: if !has_shield {
            SHIELD_WALL_DRAW_COMMANDS_REGION
        } else if animate_shields {
            SHIELD_WALL_DRAW_COMMANDS_ANIMATED
        } else {
            SHIELD_WALL_DRAW_COMMANDS_STATIC
        },
        radius: shield_radius * tile_size * size as f32 / 2.0,
        hit_alpha: hit.clamp(0.0, 1.0),
        fill_alpha: if has_shield && !animate_shields {
            0.09 + (0.08 * hit).clamp(0.0, 1.0)
        } else if has_shield {
            1.0
        } else {
            0.0
        },
        stroke: if has_shield && !animate_shields {
            1.5
        } else {
            0.0
        },
        glow_alpha: if has_shield {
            (1.0 - glow_mag + absin) * shield_radius
        } else {
            0.0
        },
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalForceProjectorStatsPlan {
    pub shield_health: f32,
    pub cooldown_time_seconds: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalForceProjectorPlacePlan {
    pub origin: (f32, f32),
    pub top_point: (f32, f32),
    pub bottom_point: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalForceProjectorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalForceProjectorDeflectPlan {
    pub active: bool,
    pub segment: ((f32, f32), (f32, f32)),
    pub bounds: DirectionalForceProjectorRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionalForceProjectorDrawCommand {
    SuperDraw,
    TopAdditive,
    ResetTop,
    SetShieldLayer,
    SetShieldColor,
    FillAnimatedRect,
    DrawAnimatedEdges,
    FillAnimatedCaps,
    FillStaticRect,
    StrokeStaticRect,
    ResetShield,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalForceProjectorDrawPlan {
    pub commands: &'static [DirectionalForceProjectorDrawCommand],
    pub top_alpha: f32,
    pub shield_rect: DirectionalForceProjectorRect,
    pub segment: ((f32, f32), (f32, f32)),
    pub hit_alpha: f32,
    pub fill_alpha: f32,
    pub stroke: f32,
    pub edge_stroke: f32,
}

const DIRECTIONAL_FORCE_PROJECTOR_DRAW_TOP_ANIMATED: &[DirectionalForceProjectorDrawCommand] = &[
    DirectionalForceProjectorDrawCommand::SuperDraw,
    DirectionalForceProjectorDrawCommand::TopAdditive,
    DirectionalForceProjectorDrawCommand::ResetTop,
    DirectionalForceProjectorDrawCommand::SetShieldLayer,
    DirectionalForceProjectorDrawCommand::SetShieldColor,
    DirectionalForceProjectorDrawCommand::FillAnimatedRect,
    DirectionalForceProjectorDrawCommand::DrawAnimatedEdges,
    DirectionalForceProjectorDrawCommand::FillAnimatedCaps,
    DirectionalForceProjectorDrawCommand::ResetShield,
];

const DIRECTIONAL_FORCE_PROJECTOR_DRAW_TOP_STATIC: &[DirectionalForceProjectorDrawCommand] = &[
    DirectionalForceProjectorDrawCommand::SuperDraw,
    DirectionalForceProjectorDrawCommand::TopAdditive,
    DirectionalForceProjectorDrawCommand::ResetTop,
    DirectionalForceProjectorDrawCommand::SetShieldLayer,
    DirectionalForceProjectorDrawCommand::SetShieldColor,
    DirectionalForceProjectorDrawCommand::FillStaticRect,
    DirectionalForceProjectorDrawCommand::StrokeStaticRect,
    DirectionalForceProjectorDrawCommand::ResetShield,
];

const DIRECTIONAL_FORCE_PROJECTOR_DRAW_ANIMATED: &[DirectionalForceProjectorDrawCommand] = &[
    DirectionalForceProjectorDrawCommand::SuperDraw,
    DirectionalForceProjectorDrawCommand::SetShieldLayer,
    DirectionalForceProjectorDrawCommand::SetShieldColor,
    DirectionalForceProjectorDrawCommand::FillAnimatedRect,
    DirectionalForceProjectorDrawCommand::DrawAnimatedEdges,
    DirectionalForceProjectorDrawCommand::FillAnimatedCaps,
    DirectionalForceProjectorDrawCommand::ResetShield,
];

const DIRECTIONAL_FORCE_PROJECTOR_DRAW_STATIC: &[DirectionalForceProjectorDrawCommand] = &[
    DirectionalForceProjectorDrawCommand::SuperDraw,
    DirectionalForceProjectorDrawCommand::SetShieldLayer,
    DirectionalForceProjectorDrawCommand::SetShieldColor,
    DirectionalForceProjectorDrawCommand::FillStaticRect,
    DirectionalForceProjectorDrawCommand::StrokeStaticRect,
    DirectionalForceProjectorDrawCommand::ResetShield,
];

const DIRECTIONAL_FORCE_PROJECTOR_DRAW_TOP_ONLY: &[DirectionalForceProjectorDrawCommand] = &[
    DirectionalForceProjectorDrawCommand::SuperDraw,
    DirectionalForceProjectorDrawCommand::TopAdditive,
    DirectionalForceProjectorDrawCommand::ResetTop,
];

const DIRECTIONAL_FORCE_PROJECTOR_DRAW_SUPER_ONLY: &[DirectionalForceProjectorDrawCommand] =
    &[DirectionalForceProjectorDrawCommand::SuperDraw];

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

pub fn directional_force_projector_clip_radius(width: f32) -> f32 {
    width + 3.0
}

pub fn directional_force_projector_effective_length(length: f32, size: i32, tile_size: f32) -> f32 {
    if length < 0.0 {
        size as f32 * tile_size / 2.0
    } else {
        length
    }
}

pub fn directional_force_projector_stats_plan(
    shield_health: f32,
    cooldown_broken_base: f32,
) -> DirectionalForceProjectorStatsPlan {
    DirectionalForceProjectorStatsPlan {
        shield_health,
        cooldown_time_seconds: (shield_health / cooldown_broken_base / 60.0) as i32,
    }
}

pub fn directional_force_projector_bar_fraction(
    state: &DirectionalForceProjectorState,
    shield_health: f32,
) -> f32 {
    if state.broken {
        0.0
    } else {
        1.0 - state.buildup / shield_health
    }
}

pub fn directional_force_projector_outputs_items() -> bool {
    false
}

pub fn directional_force_projector_should_ambient_sound(
    state: &DirectionalForceProjectorState,
) -> bool {
    !state.broken && state.shield_radius > 1.0
}

pub fn directional_force_projector_place_plan(
    tile_x: i32,
    tile_y: i32,
    tile_size: f32,
    length: f32,
    width: f32,
    size: i32,
    rotation: i32,
) -> DirectionalForceProjectorPlacePlan {
    let origin_x = tile_x as f32 * tile_size;
    let origin_y = tile_y as f32 * tile_size;
    let rotation_degrees = rotation as f32 * 90.0;
    let x = length - size as f32 / 2.0;
    let y = width + size as f32 / 2.0;
    DirectionalForceProjectorPlacePlan {
        origin: (origin_x, origin_y),
        top_point: rotate_add(x, y, rotation_degrees, origin_x, origin_y),
        bottom_point: rotate_add(x, -y, rotation_degrees, origin_x, origin_y),
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

fn directional_force_projector_rect_from_segment(
    segment: ((f32, f32), (f32, f32)),
    grow: f32,
) -> DirectionalForceProjectorRect {
    let min_x = segment.0 .0.min(segment.1 .0);
    let min_y = segment.0 .1.min(segment.1 .1);
    let max_x = segment.0 .0.max(segment.1 .0);
    let max_y = segment.0 .1.max(segment.1 .1);
    DirectionalForceProjectorRect {
        x: min_x - grow,
        y: min_y - grow,
        width: max_x - min_x + grow * 2.0,
        height: max_y - min_y + grow * 2.0,
    }
}

pub fn directional_force_projector_deflect_plan(
    state: &DirectionalForceProjectorState,
    origin_x: f32,
    origin_y: f32,
    length: f32,
    rotation_degrees: f32,
    pad_size: f32,
) -> DirectionalForceProjectorDeflectPlan {
    let active = state.shield_radius > 0.0 && !state.broken;
    let segment = if active {
        directional_force_projector_segment(
            origin_x,
            origin_y,
            length,
            state.shield_radius,
            rotation_degrees,
        )
    } else {
        ((origin_x, origin_y), (origin_x, origin_y))
    };
    DirectionalForceProjectorDeflectPlan {
        active,
        segment,
        bounds: if active {
            directional_force_projector_rect_from_segment(segment, pad_size)
        } else {
            DirectionalForceProjectorRect {
                x: origin_x,
                y: origin_y,
                width: 0.0,
                height: 0.0,
            }
        },
    }
}

pub fn directional_force_projector_draw_plan(
    state: &DirectionalForceProjectorState,
    origin_x: f32,
    origin_y: f32,
    length: f32,
    rotation_degrees: f32,
    shield_health: f32,
    animate_shields: bool,
) -> DirectionalForceProjectorDrawPlan {
    let has_top = state.buildup > 0.0;
    let has_shield = !state.broken && state.shield_radius > 0.0;
    let segment = if has_shield {
        directional_force_projector_segment(
            origin_x,
            origin_y,
            length,
            state.shield_radius,
            rotation_degrees,
        )
    } else {
        ((origin_x, origin_y), (origin_x, origin_y))
    };
    DirectionalForceProjectorDrawPlan {
        commands: match (has_top, has_shield, animate_shields) {
            (true, true, true) => DIRECTIONAL_FORCE_PROJECTOR_DRAW_TOP_ANIMATED,
            (true, true, false) => DIRECTIONAL_FORCE_PROJECTOR_DRAW_TOP_STATIC,
            (false, true, true) => DIRECTIONAL_FORCE_PROJECTOR_DRAW_ANIMATED,
            (false, true, false) => DIRECTIONAL_FORCE_PROJECTOR_DRAW_STATIC,
            (true, false, _) => DIRECTIONAL_FORCE_PROJECTOR_DRAW_TOP_ONLY,
            (false, false, _) => DIRECTIONAL_FORCE_PROJECTOR_DRAW_SUPER_ONLY,
        },
        top_alpha: if has_top && shield_health != 0.0 {
            state.buildup / shield_health * 0.75
        } else {
            0.0
        },
        shield_rect: if has_shield {
            directional_force_projector_rect_from_segment(segment, 4.0)
        } else {
            DirectionalForceProjectorRect {
                x: origin_x,
                y: origin_y,
                width: 0.0,
                height: 0.0,
            }
        },
        segment,
        hit_alpha: state.hit.clamp(0.0, 1.0),
        fill_alpha: if has_shield && !animate_shields {
            0.09 + (0.08 * state.hit).clamp(0.0, 1.0)
        } else if has_shield {
            1.0
        } else {
            0.0
        },
        stroke: if has_shield && !animate_shields {
            1.5
        } else {
            0.0
        },
        edge_stroke: if has_shield && animate_shields {
            3.0
        } else {
            0.0
        },
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadarIconRegion {
    Base,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarRangePlan {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadarDrawCommand {
    Base,
    Region,
    GlowAdditive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarDrawPlan {
    pub commands: &'static [RadarDrawCommand],
    pub rotation: f32,
    pub glow_alpha: f32,
}

const RADAR_ICONS: &[RadarIconRegion] = &[RadarIconRegion::Base, RadarIconRegion::Main];

const RADAR_DRAW_COMMANDS: &[RadarDrawCommand] = &[
    RadarDrawCommand::Base,
    RadarDrawCommand::Region,
    RadarDrawCommand::GlowAdditive,
];

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

pub fn radar_icons() -> &'static [RadarIconRegion] {
    RADAR_ICONS
}

pub fn radar_fog_radius(fog_radius: f32, progress: f32, smooth_efficiency: f32) -> f32 {
    fog_radius * progress * smooth_efficiency
}

pub fn radar_force_update_needed(
    fog_radius: f32,
    progress: f32,
    smooth_efficiency: f32,
    last_radius: f32,
) -> bool {
    (radar_fog_radius(fog_radius, progress, smooth_efficiency) - last_radius).abs() >= 0.5
}

pub fn radar_progress(state: &RadarState) -> f32 {
    state.progress
}

pub fn radar_can_pickup() -> bool {
    false
}

pub fn radar_place_plan(
    tile_x: i32,
    tile_y: i32,
    tile_size: f32,
    offset: f32,
    fog_radius: f32,
) -> RadarRangePlan {
    RadarRangePlan {
        center_x: tile_x as f32 * tile_size + offset,
        center_y: tile_y as f32 * tile_size + offset,
        radius: fog_radius * tile_size,
    }
}

pub fn radar_select_plan(
    x: f32,
    y: f32,
    fog_radius: f32,
    progress: f32,
    smooth_efficiency: f32,
    tile_size: f32,
) -> RadarRangePlan {
    RadarRangePlan {
        center_x: x,
        center_y: y,
        radius: radar_fog_radius(fog_radius, progress, smooth_efficiency) * tile_size,
    }
}

pub fn radar_draw_rotation(rotate_speed: f32, total_progress: f32) -> f32 {
    rotate_speed * total_progress
}

pub fn radar_glow_alpha(time: f32, glow_scl: f32, glow_mag: f32, glow_color_alpha: f32) -> f32 {
    glow_color_alpha * (1.0 - glow_mag + absin_time(time, glow_scl, glow_mag))
}

pub fn radar_draw_plan(
    state: &RadarState,
    time: f32,
    rotate_speed: f32,
    glow_scl: f32,
    glow_mag: f32,
    glow_color_alpha: f32,
) -> RadarDrawPlan {
    RadarDrawPlan {
        commands: RADAR_DRAW_COMMANDS,
        rotation: radar_draw_rotation(rotate_speed, state.total_progress),
        glow_alpha: radar_glow_alpha(time, glow_scl, glow_mag, glow_color_alpha),
    }
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
    let force_update = radar_force_update_needed(
        fog_radius,
        state.progress,
        state.smooth_efficiency,
        state.last_radius,
    );
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildTurretStatsPlan {
    pub build_speed_percent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTurretIconRegion {
    Base,
    Main,
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

#[derive(Debug, Clone, PartialEq)]
pub enum BuildTurretSenseObject {
    BuildingAt { x: i32, y: i32 },
}

pub fn build_turret_stats_plan(build_speed: f32) -> BuildTurretStatsPlan {
    BuildTurretStatsPlan {
        build_speed_percent: build_speed,
    }
}

const BUILD_TURRET_ICON_REGIONS: &[BuildTurretIconRegion] =
    &[BuildTurretIconRegion::Base, BuildTurretIconRegion::Main];

pub fn build_turret_icons() -> &'static [BuildTurretIconRegion] {
    BUILD_TURRET_ICON_REGIONS
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

pub fn build_turret_capture_unit_plans(state: &mut BuildTurretState, unit: &UnitComp) {
    state.plans = unit.builder.plans.iter().cloned().collect();
    state.raw_plans.clear();
}

pub fn build_turret_apply_unit_plans(state: &BuildTurretState, unit: &mut UnitComp) {
    unit.builder.plans = state.plans.iter().cloned().collect();
    unit.refresh_component_views();
}

pub fn build_turret_sense_from_plan(sensor: LAccess, plan: Option<&BuildPlan>) -> Option<f64> {
    match sensor {
        LAccess::BuildX => Some(plan.map(|plan| plan.x).unwrap_or(-1) as f64),
        LAccess::BuildY => Some(plan.map(|plan| plan.y).unwrap_or(-1) as f64),
        _ => None,
    }
}

pub fn build_turret_sense_object_from_plan(
    sensor: LAccess,
    plan: Option<&BuildPlan>,
) -> Option<BuildTurretSenseObject> {
    let plan = plan?;
    match sensor {
        LAccess::Building if !plan.breaking => Some(BuildTurretSenseObject::BuildingAt {
            x: plan.x,
            y: plan.y,
        }),
        LAccess::Breaking if plan.breaking => Some(BuildTurretSenseObject::BuildingAt {
            x: plan.x,
            y: plan.y,
        }),
        LAccess::Building | LAccess::Breaking => None,
        _ => None,
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockwaveTowerStatsPlan {
    pub damage: f32,
    pub range_blocks: f32,
    pub reload_per_second: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockwaveTowerRangePlan {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockwaveTowerDrawCommand {
    SuperDraw,
    HeatAdditive,
    SetEffectLayer,
    FillShape,
    ResetColor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockwaveTowerDrawPlan {
    pub commands: &'static [ShockwaveTowerDrawCommand],
    pub heat_alpha: f32,
    pub color_lerp: f32,
    pub shape_sides: i32,
    pub shape_radius: f32,
    pub shape_rotation: f32,
}

const SHOCKWAVE_TOWER_DRAW_COMMANDS: &[ShockwaveTowerDrawCommand] = &[
    ShockwaveTowerDrawCommand::SuperDraw,
    ShockwaveTowerDrawCommand::HeatAdditive,
    ShockwaveTowerDrawCommand::SetEffectLayer,
    ShockwaveTowerDrawCommand::FillShape,
    ShockwaveTowerDrawCommand::ResetColor,
];

pub fn shockwave_tower_stats_plan(
    bullet_damage: f32,
    range: f32,
    tile_size: f32,
    reload: f32,
) -> ShockwaveTowerStatsPlan {
    ShockwaveTowerStatsPlan {
        damage: bullet_damage,
        range_blocks: range / tile_size,
        reload_per_second: 60.0 / reload,
    }
}

pub fn shockwave_tower_place_plan(
    tile_x: i32,
    tile_y: i32,
    tile_size: f32,
    offset: f32,
    range: f32,
) -> ShockwaveTowerRangePlan {
    ShockwaveTowerRangePlan {
        center_x: tile_x as f32 * tile_size + offset,
        center_y: tile_y as f32 * tile_size + offset,
        radius: range,
    }
}

pub fn shockwave_tower_select_plan(x: f32, y: f32, range: f32) -> ShockwaveTowerRangePlan {
    ShockwaveTowerRangePlan {
        center_x: x,
        center_y: y,
        radius: range,
    }
}

pub fn shockwave_tower_wave_damage(
    bullet_damage: f32,
    falloff_count: f32,
    target_count: usize,
) -> f32 {
    if target_count == 0 {
        0.0
    } else {
        bullet_damage.min(bullet_damage * falloff_count / target_count as f32)
    }
}

pub fn shockwave_tower_can_target(enemy_team: bool, hittable: bool) -> bool {
    enemy_team && hittable
}

pub fn shockwave_tower_can_fire(
    potential_efficiency: f32,
    reload_counter: f32,
    reload: f32,
    timer_ready: bool,
    target_count: usize,
) -> bool {
    potential_efficiency > 0.0 && reload_counter >= reload && timer_ready && target_count > 0
}

pub fn shockwave_tower_apply_damage(current_damage: f32, wave_damage: f32) -> (f32, bool) {
    if current_damage > wave_damage {
        (current_damage - wave_damage, false)
    } else {
        (0.0, true)
    }
}

pub fn shockwave_tower_heat_after_cooldown(
    heat: f32,
    delta: f32,
    reload: f32,
    cooldown_multiplier: f32,
) -> f32 {
    (heat - delta / reload * cooldown_multiplier).clamp(0.0, 1.0)
}

pub fn shockwave_tower_sense(
    sensor: LAccess,
    state: &ShockwaveTowerState,
    reload: f32,
) -> Option<f64> {
    match sensor {
        LAccess::Progress => Some(shockwave_tower_progress(state.reload_counter, reload) as f64),
        _ => None,
    }
}

pub fn shockwave_tower_warmup(state: &ShockwaveTowerState) -> f32 {
    state.heat
}

pub fn shockwave_tower_draw_plan(
    state: &ShockwaveTowerState,
    time: f32,
    potential_efficiency: f32,
    shape_sides: i32,
    shape_radius: f32,
    shape_rotate_speed: f32,
) -> ShockwaveTowerDrawPlan {
    ShockwaveTowerDrawPlan {
        commands: SHOCKWAVE_TOWER_DRAW_COMMANDS,
        heat_alpha: state.heat,
        color_lerp: state.heat.powi(2),
        shape_sides,
        shape_radius: shape_radius * potential_efficiency,
        shape_rotation: time * shape_rotate_speed,
    }
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
    if shockwave_tower_can_fire(
        potential_efficiency,
        state.reload_counter,
        reload,
        timer_ready,
        target_damages.len(),
    ) {
        state.heat = 1.0;
        state.reload_counter = 0.0;
        fire.fired = true;
        fire.wave_damage =
            shockwave_tower_wave_damage(bullet_damage, falloff_count, target_damages.len());
        for damage in target_damages {
            let (remaining, removed) = shockwave_tower_apply_damage(*damage, fire.wave_damage);
            *damage = remaining;
            if removed {
                fire.removed_targets += 1;
            }
        }
    }
    state.heat =
        shockwave_tower_heat_after_cooldown(state.heat, delta, reload, cooldown_multiplier);
    fire
}

pub fn shockwave_tower_progress(reload_counter: f32, reload: f32) -> f32 {
    reload_counter / reload
}

pub fn shockwave_tower_should_consume(reload_counter: f32, reload: f32) -> bool {
    reload_counter < reload
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThrusterBlockConfig {
    pub rotate: bool,
    pub quick_rotate: bool,
}

impl Default for ThrusterBlockConfig {
    fn default() -> Self {
        Self {
            rotate: true,
            quick_rotate: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrusterDrawCommand {
    Region,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThrusterDrawPlan {
    pub commands: &'static [ThrusterDrawCommand],
    pub x: f32,
    pub y: f32,
    pub top_rotation: f32,
}

const THRUSTER_DRAW_COMMANDS: &[ThrusterDrawCommand] =
    &[ThrusterDrawCommand::Region, ThrusterDrawCommand::Top];

pub fn thruster_top_rotation(rotation: i32) -> f32 {
    rotation as f32 * 90.0
}

pub fn thruster_draw_plan(x: f32, y: f32, rotation: i32) -> ThrusterDrawPlan {
    ThrusterDrawPlan {
        commands: THRUSTER_DRAW_COMMANDS,
        x,
        y,
        top_rotation: thruster_top_rotation(rotation),
    }
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

fn approach_delta(from: f32, to: f32, amount: f32) -> f32 {
    if from < to {
        (from + amount).min(to)
    } else {
        (from - amount).max(to)
    }
}

fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn format_fixed_trimmed(value: f32, decimals: usize) -> String {
    let mut formatted = format!("{value:.decimals$}");
    if let Some(dot_index) = formatted.find('.') {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.len() == dot_index + 1 {
            formatted.pop();
        }
    }
    formatted
}

fn projector_cycle(time: f32) -> f32 {
    1.0 - (time / 100.0).rem_euclid(1.0)
}

fn sin_time(time: f32, scl: f32, mag: f32) -> f32 {
    (time / scl).sin() * mag
}

fn absin_time(time: f32, scl: f32, mag: f32) -> f32 {
    sin_time(time, scl, mag).abs()
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
        assert_eq!(wall_draw_hit_decay(1.0, 5.0, true), 1.0);
        assert_eq!(
            wall_stats_plan(0.25, 0.5, 20.0),
            WallStatsPlan {
                base_deflect_chance: Some(0.25),
                lightning_chance_percent: Some(50.0),
                lightning_damage: Some(20.0),
            }
        );
        assert_eq!(
            wall_stats_plan(-1.0, -1.0, 20.0),
            WallStatsPlan {
                base_deflect_chance: None,
                lightning_chance_percent: None,
                lightning_damage: None,
            }
        );
        assert_eq!(
            wall_init_destroy_sound(2, true),
            WallDestroySound::BlockExplodeWall
        );
        assert_eq!(wall_init_destroy_sound(1, true), WallDestroySound::Keep);
        assert_eq!(wall_icon_region(true), WallIconRegion::Main);
        assert_eq!(wall_icon_region(false), WallIconRegion::Variant1);
        assert!(wall_should_lightning(0.25, 0.2));
        assert!(wall_deflects_bullet(10.0, 1.0, true, 20.0, 0.4));
        assert!(!wall_deflects_bullet(10.0, 0.05, true, 20.0, 0.0));
        assert!(wall_reflect_x(6.0, 3.0));
        assert_eq!(wall_collision_reflect_axis(6.0, 3.0), WallReflectAxis::X);
        assert_eq!(wall_collision_reflect_axis(3.0, 6.0), WallReflectAxis::Y);
        assert_eq!(wall_collision_reflect_axis(4.0, 4.0), WallReflectAxis::Y);
        assert_eq!(
            wall_draw_plan(true, 0.8, 8.0, 2, 5.0, false),
            WallDrawPlan {
                draw_flash: true,
                flash_alpha: 0.4,
                flash_size: 16.0,
                next_hit: 0.3,
            }
        );
        assert_eq!(
            wall_draw_plan(true, 0.00001, 8.0, 2, 5.0, false),
            WallDrawPlan {
                draw_flash: false,
                flash_alpha: 0.0,
                flash_size: 16.0,
                next_hit: 0.00001,
            }
        );
        assert_eq!(
            wall_collision_plan(0.5, 0.25, 30.0, 10.0, 1.0, true, 20.0, 0.4, 6.0, 3.0),
            WallCollisionPlan {
                hit: 1.0,
                create_lightning: true,
                lightning_rotation: 210.0,
                deflected: true,
                reflect_axis: Some(WallReflectAxis::X),
                bullet_time_add: 1.0,
                transfer_owner_and_team: true,
                continue_collision: false,
            }
        );
        assert_eq!(
            wall_collision_plan(0.0, 0.0, 30.0, 10.0, 0.05, true, 20.0, 0.0, 1.0, 3.0),
            WallCollisionPlan {
                hit: 1.0,
                create_lightning: false,
                lightning_rotation: 210.0,
                deflected: false,
                reflect_axis: None,
                bullet_time_add: 0.0,
                transfer_owner_and_team: false,
                continue_collision: true,
            }
        );

        assert!(door_check_solid(false));
        assert_eq!(door_sense_enabled(true), 1.0);
        assert!(door_can_toggle(false, true, false, true));
        assert!(!door_can_toggle(false, false, false, true));
        assert!(door_tapped_should_configure(false, true, true));
        assert!(!door_tapped_should_configure(true, true, true));
        assert_eq!(door_effect_for_current_open(false), DoorEffectKind::Open);
        assert_eq!(door_effect_for_current_open(true), DoorEffectKind::Close);
        assert_eq!(door_origin_id(10, None), 10);
        assert_eq!(door_origin_id(10, Some(7)), 7);
        assert_eq!(
            door_control_enabled_plan(false, 1.0, false, false, true),
            DoorControlPlan {
                configure: Some(true),
            }
        );
        assert_eq!(
            door_control_enabled_plan(true, 0.0, false, false, true),
            DoorControlPlan {
                configure: Some(false),
            }
        );
        assert_eq!(
            door_control_enabled_plan(true, 0.0, true, false, true),
            DoorControlPlan { configure: None }
        );
        assert_eq!(
            door_control_enabled_plan(true, 0.0, false, true, true),
            DoorControlPlan { configure: None }
        );
        assert_eq!(
            door_control_enabled_plan(false, 0.0, false, false, true),
            DoorControlPlan { configure: None }
        );
        assert_eq!(
            door_tapped_plan(false, true, true),
            DoorTappedPlan {
                configure: Some(true),
            }
        );
        assert_eq!(
            door_tapped_plan(true, true, true),
            DoorTappedPlan { configure: None }
        );
        assert_eq!(
            door_chain_toggle_plan(
                [
                    DoorChainNode {
                        id: 1,
                        open: false,
                        units_in_tile: false,
                    },
                    DoorChainNode {
                        id: 2,
                        open: true,
                        units_in_tile: false,
                    },
                    DoorChainNode {
                        id: 3,
                        open: false,
                        units_in_tile: true,
                    },
                ],
                true,
                false,
                true,
            ),
            DoorChainTogglePlan {
                play_origin_sound: true,
                play_origin_effect: true,
                toggles: vec![
                    DoorChainToggle {
                        id: 1,
                        open: true,
                        play_chain_effect: true,
                        update_pathfinder: true,
                    },
                    DoorChainToggle {
                        id: 3,
                        open: true,
                        play_chain_effect: true,
                        update_pathfinder: true,
                    },
                ],
            }
        );
        assert_eq!(
            door_chain_toggle_plan(
                [
                    DoorChainNode {
                        id: 1,
                        open: true,
                        units_in_tile: false,
                    },
                    DoorChainNode {
                        id: 2,
                        open: true,
                        units_in_tile: true,
                    },
                    DoorChainNode {
                        id: 3,
                        open: false,
                        units_in_tile: false,
                    },
                ],
                false,
                true,
                false,
            ),
            DoorChainTogglePlan {
                play_origin_sound: false,
                play_origin_effect: false,
                toggles: vec![DoorChainToggle {
                    id: 1,
                    open: false,
                    play_chain_effect: false,
                    update_pathfinder: false,
                }],
            }
        );
        assert_eq!(door_region_for_open(false), DoorRegion::Closed);
        assert_eq!(door_region_for_open(true), DoorRegion::Open);
        assert_eq!(door_plan_region(Some(true)), DoorRegion::Open);
        assert_eq!(door_plan_region(Some(false)), DoorRegion::Closed);
        assert_eq!(door_plan_region(None), DoorRegion::Closed);
        assert_eq!(
            door_draw_plan(true),
            DoorDrawPlan {
                command: DoorDrawCommand::Region(DoorRegion::Open),
            }
        );
        assert_eq!(
            door_draw_plan(false),
            DoorDrawPlan {
                command: DoorDrawCommand::Region(DoorRegion::Closed),
            }
        );

        let mut bytes = Vec::new();
        write_door_state(&mut bytes, DoorState { open: true }).unwrap();
        assert_eq!(
            read_door_state(&mut bytes.as_slice()).unwrap(),
            DoorState { open: true }
        );

        assert!(auto_door_should_open(true));
        assert!(auto_door_ground_check(true, false));
        assert!(!auto_door_ground_check(false, false));
        assert!(!auto_door_ground_check(true, true));
        assert_eq!(auto_door_trigger_size(2, 8.0, 12.0), 40.0);
        assert_eq!(
            auto_door_trigger_rect(20.0, 28.0, 2, 8.0, 12.0),
            AutoDoorTriggerRect {
                center_x: 20.0,
                center_y: 28.0,
                size: 40.0,
            }
        );
        assert!(auto_door_remote_toggle_valid(true, true));
        assert!(!auto_door_remote_toggle_valid(false, true));
        assert!(!auto_door_remote_toggle_valid(true, false));
        assert_eq!(
            auto_door_update_plan(false, true, false, true),
            AutoDoorUpdatePlan {
                should_scan_units: true,
                should_open: true,
                send_toggle: Some(true),
            }
        );
        assert_eq!(
            auto_door_update_plan(true, true, false, true).send_toggle,
            None
        );
        assert_eq!(
            auto_door_update_plan(false, false, false, true),
            AutoDoorUpdatePlan {
                should_scan_units: false,
                should_open: true,
                send_toggle: None,
            }
        );
        assert_eq!(
            auto_door_update_plan(false, true, true, true).send_toggle,
            None
        );
        assert_eq!(
            auto_door_set_open_plan(true, true),
            AutoDoorSetOpenPlan {
                open: true,
                update_pathfinder: true,
                play_effect: true,
                play_sound: true,
            }
        );
        assert_eq!(
            auto_door_set_open_plan(false, false),
            AutoDoorSetOpenPlan {
                open: false,
                update_pathfinder: true,
                play_effect: false,
                play_sound: false,
            }
        );

        assert!(shock_mine_should_trigger(true, false, true));
        assert!(!shock_mine_should_trigger(false, false, true));
        assert!(!shock_mine_should_trigger(true, true, true));
        assert!(!shock_mine_should_trigger(true, false, false));
        assert_eq!(
            shock_mine_stats_plan(6),
            ShockMineStatsPlan {
                tendrils: 6,
                damage_fixed_decimals: 2,
            }
        );
        assert_eq!(
            shock_mine_stats_text(6, 13.0),
            "[white]6x[lightgray] lightning ~ [white]13[lightgray] damage"
        );
        assert_eq!(
            shock_mine_stats_text(3, 12.25),
            "[white]3x[lightgray] lightning ~ [white]12.25[lightgray] damage"
        );
        assert_eq!(
            shock_mine_draw_plan(0.3),
            ShockMineDrawPlan {
                draw_base: true,
                draw_team_top: true,
                team_alpha: 0.3,
            }
        );
        assert_eq!(
            shock_mine_lightning_angles(3, &[10.0, 20.0]),
            vec![10.0, 20.0, 0.0]
        );
        assert_eq!(
            shock_mine_bullet_angles(4, &[1.0, 2.0]),
            vec![1.0, 92.0, 180.0, 270.0]
        );
        assert!(shock_mine_bullet_angles(0, &[1.0]).is_empty());
        assert_eq!(
            shock_mine_trigger_plan(
                true,
                false,
                true,
                5.0,
                13.0,
                10,
                3,
                &[90.0, 180.0],
                true,
                4,
                &[1.0, 2.0],
            ),
            ShockMineTriggerPlan {
                triggered: true,
                self_damage: 5.0,
                lightning_angles: vec![90.0, 180.0, 0.0],
                lightning_damage: 13.0,
                lightning_length: 10,
                bullet_angles: vec![1.0, 92.0, 180.0, 270.0],
            }
        );
        assert_eq!(
            shock_mine_trigger_plan(true, true, true, 5.0, 13.0, 10, 3, &[90.0], true, 4, &[1.0],),
            ShockMineTriggerPlan {
                triggered: false,
                self_damage: 0.0,
                lightning_angles: Vec::new(),
                lightning_damage: 0.0,
                lightning_length: 0,
                bullet_angles: Vec::new(),
            }
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
    fn mend_projector_draw_select_sense_and_timer_plans_follow_java() {
        assert_eq!(mend_projector_repair_time_seconds(250.0, 12.0), 34.72222);
        assert_eq!(mend_projector_range_blocks(60.0, 8.0), 7.5);
        assert_eq!(mend_projector_booster_multiplier(12.0, 12.0), 2.0);
        assert_eq!(mend_projector_real_range(60.0, 0.5, 50.0), 85.0);
        assert_eq!(mend_projector_heal_fraction(12.0, 0.5, 12.0, 0.75), 0.135);

        let mut mend = MendProjectorState {
            heat: 0.75,
            charge: 500.0,
            phase_heat: 0.5,
            smooth_efficiency: 0.8,
        };
        assert_eq!(
            mend_projector_sense(LAccess::Progress, &mend, 250.0),
            Some(1.0)
        );
        assert_eq!(mend_projector_sense(LAccess::Health, &mend, 250.0), None);
        assert!(mend_projector_should_consume_optional(1.0, true, true));
        assert!(!mend_projector_should_consume_optional(1.0, false, true));
        assert!(!mend_projector_should_consume_optional(0.0, true, true));
        assert!(!mend_projector_should_consume_optional(1.0, true, false));

        let blocked = mend_projector_update_with_timer(
            &mut mend, 1.0, 1.0, true, false, 1.0, 250.0, 60.0, 12.0, 12.0, 50.0,
        );
        assert!(!blocked.fired);
        assert_eq!(mend.charge, 500.0 + mend.heat);
        assert!(!blocked.should_consume_optional);

        let place = mend_projector_place_plan(2, 3, 8.0, 4.0, 60.0, 0.0);
        assert_eq!(
            place,
            ProjectorPlacementPlan {
                center_x: 20.0,
                center_y: 28.0,
                real_range: 60.0,
                selected_alpha: 0.0,
                target_filter: ProjectorTargetFilter::AnyBlock,
            }
        );

        let select = mend_projector_select_plan(&mend, 60.0, 50.0, 0.0);
        assert_eq!(select.real_range, 87.5);
        assert_eq!(select.target_filter, ProjectorTargetFilter::AnyBlock);

        let light = mend_projector_light_plan(&mend, 50.0);
        assert_eq!(light.radius, 40.8);
        assert!((light.alpha - 0.5712).abs() < 0.00001);

        let draw = mend_projector_draw_plan(&mend, 25.0, 2, 8.0);
        assert_eq!(
            draw.commands,
            &[
                MendProjectorDrawCommand::SetColor,
                MendProjectorDrawCommand::DrawTop,
                MendProjectorDrawCommand::ResetAlpha,
                MendProjectorDrawCommand::StrokeSquare,
                MendProjectorDrawCommand::Reset,
            ]
        );
        assert_eq!(draw.phase_lerp, mend.phase_heat);
        assert_eq!(draw.cycle, 0.75);
        assert_eq!(draw.stroke, (2.0 * 0.75 + 0.2) * mend.heat);
        assert_eq!(draw.square_radius, 3.0);
        assert!(draw.top_alpha > 0.0);

        assert_eq!(
            mend_projector_pulse_plan(true, true),
            MendProjectorPulsePlan {
                scan_targets: true,
                play_sound: true,
            }
        );
        assert_eq!(
            mend_projector_pulse_plan(true, false),
            MendProjectorPulsePlan {
                scan_targets: true,
                play_sound: false,
            }
        );
    }

    #[test]
    fn overdrive_projector_bar_draw_select_and_stats_follow_java() {
        assert_eq!(overdrive_speed_increase_percent(1.5), 50.0);
        assert_eq!(overdrive_production_time_seconds(400.0), 400.0 / 60.0);
        assert_eq!(overdrive_real_range(80.0, 0.5, 20.0), 90.0);
        assert_eq!(overdrive_boost_multiplier_limit(true, 1.5, 0.75), 2.25);
        assert_eq!(overdrive_boost_multiplier_limit(false, 1.5, 0.75), 1.5);
        assert_eq!(
            overdrive_projector_bar_fraction(1.5375, true, 1.5, 0.75),
            1.5375 / 2.25
        );
        assert_eq!(
            overdrive_projector_bar_fraction(1.2, false, 1.5, 0.75),
            1.2 / 1.5
        );

        let mut state = OverdriveProjectorState {
            heat: 0.6,
            charge: 0.0,
            phase_heat: 0.5,
            smooth_efficiency: 0.25,
            use_progress: 0.0,
        };
        let no_phase = overdrive_projector_update(
            &mut state, 1.0, 1.0, false, 1.0, 60.0, 80.0, 20.0, 1.5, 0.75, 400.0,
        );
        assert_eq!(state.phase_heat, 0.5);
        assert_eq!(no_phase.real_range, 90.0);

        let place = overdrive_projector_place_plan(4, 5, 8.0, 4.0, 80.0, 0.0);
        assert_eq!(
            place,
            ProjectorPlacementPlan {
                center_x: 36.0,
                center_y: 44.0,
                real_range: 80.0,
                selected_alpha: 0.0,
                target_filter: ProjectorTargetFilter::CanOverdrive,
            }
        );

        let select = overdrive_projector_select_plan(&state, 80.0, 20.0, 0.0);
        assert_eq!(select.real_range, 90.0);
        assert_eq!(select.target_filter, ProjectorTargetFilter::CanOverdrive);

        let light = overdrive_projector_light_plan(&state, 50.0);
        assert_eq!(light.radius, 15.5);
        assert_eq!(light.alpha, 0.217);

        let draw = overdrive_projector_draw_plan(&state, 25.0, 2, 8.0);
        assert_eq!(
            draw.commands,
            &[
                OverdriveProjectorDrawCommand::SetColor,
                OverdriveProjectorDrawCommand::DrawTop,
                OverdriveProjectorDrawCommand::ResetAlpha,
                OverdriveProjectorDrawCommand::StrokeLineLoop,
                OverdriveProjectorDrawCommand::Reset,
            ]
        );
        assert_eq!(draw.phase_lerp, state.phase_heat);
        assert_eq!(draw.cycle, 0.75);
        assert_eq!(draw.stroke, (2.0 * 0.75 + 0.1) * state.heat);
        assert_eq!(draw.line_radius, 3.05);
        assert_eq!(draw.line_width, 0.0);
        assert!(!draw.mirrored_points);

        let wide = overdrive_projector_draw_plan(&state, 75.0, 2, 8.0);
        assert_eq!(wide.cycle, 0.25);
        assert_eq!(wide.line_radius, 7.55);
        assert_eq!(wide.line_width, 4.0);
        assert!(wide.mirrored_points);
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
        assert_eq!(
            force_projector_sense(LAccess::Heat, &force, 700.0, 400.0),
            Some(force.buildup as f64)
        );
        assert_eq!(
            force_projector_sense(LAccess::Shield, &force, 700.0, 400.0),
            Some(
                force_projector_shield(force.broken, 700.0, 400.0, force.phase_heat, force.buildup)
                    as f64
            )
        );
        assert_eq!(
            force_projector_sense(LAccess::Health, &force, 700.0, 400.0),
            None
        );
        force_projector_set_shield(&mut force, 600.0, 700.0, 400.0);
        assert!((force.buildup - 140.0).abs() < 0.00001);
        assert!(
            (force_projector_bar_fraction(&force, 700.0, 400.0) - (600.0 / 740.0)).abs() < 0.00001
        );
        assert!(!force_projector_outputs_items());
        assert!(!force_projector_in_fog_to());
        assert!(!force_projector_should_ambient_sound(&force, 100.0, 80.0));
        let ambient_force = ForceProjectorState {
            broken: false,
            radscl: 0.2,
            phase_heat: 0.5,
            ..force
        };
        assert!(force_projector_should_ambient_sound(
            &ambient_force,
            100.0,
            80.0
        ));
        assert!(!force_projector_should_ambient_sound(
            &ForceProjectorState {
                broken: true,
                ..ambient_force
            },
            100.0,
            80.0
        ));
        assert_eq!(
            force_projector_on_removed_plan(&ambient_force, 100.0, 80.0),
            ForceProjectorRemovedPlan {
                call_super_removed: true,
                play_force_shrink: true,
                effect_radius: 28.0,
            }
        );
        assert_eq!(
            force_projector_deflect_plan(&ambient_force, 100.0, 80.0),
            ForceProjectorDeflectPlan {
                active: true,
                real_radius: 28.0,
                bounds_x: -28.0,
                bounds_y: -28.0,
                bounds_width: 56.0,
                bounds_height: 56.0,
            }
        );
        assert_eq!(
            force_projector_on_removed_plan(
                &ForceProjectorState {
                    broken: true,
                    ..ambient_force
                },
                100.0,
                80.0
            ),
            ForceProjectorRemovedPlan {
                call_super_removed: true,
                play_force_shrink: false,
                effect_radius: 28.0,
            }
        );
        assert_eq!(
            force_projector_deflect_plan(
                &ForceProjectorState {
                    broken: true,
                    ..ambient_force
                },
                100.0,
                80.0
            ),
            ForceProjectorDeflectPlan {
                active: false,
                real_radius: 28.0,
                bounds_x: 0.0,
                bounds_y: 0.0,
                bounds_width: 0.0,
                bounds_height: 0.0,
            }
        );

        let bullet = force_projector_absorb_bullet(&mut force, true, true, false, true, 35.0);
        assert_eq!(
            bullet,
            ForceProjectorBulletAbsorb {
                absorbed: true,
                hit_effect: true,
                sound_effect: true,
                buildup_added: 35.0,
            }
        );
        assert_eq!(force.hit, 1.0);
        assert!((force.buildup - 175.0).abs() < 0.00001);
        let unchanged = force;
        assert_eq!(
            force_projector_absorb_bullet(&mut force, false, true, false, true, 10.0),
            ForceProjectorBulletAbsorb {
                absorbed: false,
                hit_effect: false,
                sound_effect: false,
                buildup_added: 0.0,
            }
        );
        assert_eq!(force, unchanged);

        let top_only_draw = force_projector_draw_plan(&force, 101.7, 80.0, 700.0, 6, 0.0, true);
        assert_eq!(
            top_only_draw.commands,
            &[
                ForceProjectorDrawCommand::TopAdditive,
                ForceProjectorDrawCommand::ResetTopAdditive,
                ForceProjectorDrawCommand::ResetFinal,
            ]
        );

        let draw_force = ForceProjectorState {
            radscl: 1.0,
            ..force
        };
        let animated_draw =
            force_projector_draw_plan(&draw_force, 101.7, 80.0, 700.0, 6, 0.0, true);
        assert_eq!(
            animated_draw.commands,
            &[
                ForceProjectorDrawCommand::TopAdditive,
                ForceProjectorDrawCommand::ResetTopAdditive,
                ForceProjectorDrawCommand::SetShieldColor,
                ForceProjectorDrawCommand::SetAnimatedShieldLayer,
                ForceProjectorDrawCommand::FillAnimatedPoly,
                ForceProjectorDrawCommand::ResetFinal,
            ]
        );
        assert_eq!(animated_draw.sides, 6);
        assert_eq!(animated_draw.shield_rotation, 0.0);
        assert_eq!(animated_draw.hit_alpha, 1.0);
        assert_eq!(animated_draw.shield_layer_offset, 0.001);
        assert!(animated_draw.top_alpha > 0.0);
        assert_eq!(animated_draw.fill_alpha, 1.0);

        let static_draw =
            force_projector_draw_plan(&draw_force, 101.7, 80.0, 700.0, 6, 45.0, false);
        assert_eq!(
            static_draw.commands,
            &[
                ForceProjectorDrawCommand::TopAdditive,
                ForceProjectorDrawCommand::ResetTopAdditive,
                ForceProjectorDrawCommand::SetShieldColor,
                ForceProjectorDrawCommand::SetStaticShieldLayer,
                ForceProjectorDrawCommand::StrokeStaticPoly,
                ForceProjectorDrawCommand::ResetStaticShield,
                ForceProjectorDrawCommand::ResetFinal,
            ]
        );
        assert_eq!(static_draw.shield_rotation, 45.0);
        assert_eq!(static_draw.stroke, 1.5);
        assert!((static_draw.fill_alpha - (0.09 + 0.08)).abs() < 0.00001);

        let broken_draw = force_projector_draw_plan(
            &ForceProjectorState {
                broken: true,
                buildup: 0.0,
                ..force
            },
            101.7,
            80.0,
            700.0,
            6,
            0.0,
            true,
        );
        assert_eq!(
            broken_draw.commands,
            &[ForceProjectorDrawCommand::ResetFinal]
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

        let mut picked = ForceProjectorState {
            broken: false,
            buildup: 12.0,
            radscl: 0.9,
            hit: 0.4,
            warmup: 0.7,
            phase_heat: 0.5,
        };
        force_projector_picked_up(&mut picked);
        assert_eq!(picked.radscl, 0.0);
        assert_eq!(picked.warmup, 0.0);
        assert_eq!(picked.broken, false);
        assert_eq!(picked.buildup, 12.0);
        assert_eq!(picked.phase_heat, 0.5);
        assert!(force_projector_overwrote(&mut picked, true, true, 44.0));
        assert!(picked.broken);
        assert_eq!(picked.buildup, 44.0);
        picked.phase_heat = 0.8;
        assert!(!force_projector_overwrote(&mut picked, false, false, 1.0));
        assert!(picked.broken);
        assert_eq!(picked.buildup, 44.0);
        assert_eq!(picked.phase_heat, 0.8);

        let mut bytes = Vec::new();
        write_force_projector_state(&mut bytes, &picked).unwrap();
        let restored = read_force_projector_state(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.radscl, 0.0);
        assert_eq!(restored.warmup, 0.0);
        assert_eq!(restored.hit, 0.0);

        assert_eq!(
            regen_projector_heal_amount(0.5, 2.0, 0.2, 1.0, 1000.0, 20.0),
            3.0
        );
        assert_eq!(
            regen_projector_heal_amount(1.0, 2.0, 12.0, 1.0, 1000.0, 20.0),
            20.0
        );
        let mut regen = RegenProjectorState {
            did_regen: true,
            optional_timer: 470.0,
            ..RegenProjectorState::default()
        };
        let plan = regen_projector_update(
            &mut regen,
            false,
            true,
            1.0,
            20.0,
            2.0,
            1.0,
            480.0,
            2.0,
            12.0 / 60.0,
        );
        assert_eq!(plan.warmup, 1.0 / 70.0);
        assert_eq!(plan.total_time, (1.0 / 70.0) * 2.0);
        assert!(plan.any_targets);
        assert!(plan.consume_optional);
        assert_eq!(plan.optional_timer, 0.0);
        assert_eq!(plan.heal_amount_percent, 0.4);
        assert!(regen.did_regen);

        let suppressed = regen_projector_update(
            &mut regen,
            true,
            true,
            1.0,
            20.0,
            1.0,
            1.0,
            480.0,
            2.0,
            12.0 / 60.0,
        );
        assert!(suppressed.suppressed);
        assert!(!suppressed.any_targets);
        assert_eq!(suppressed.heal_amount_percent, 0.0);
        assert!(!regen.did_regen);

        let mut mend = RegenProjectorMendMap::default();
        mend.record(10, 3.0, 20.0);
        mend.record(10, 5.0, 20.0);
        mend.record(10, 12.0, 7.0);
        mend.record(11, 4.0, 2.5);
        assert_eq!(mend.entries.get(&10), Some(&7.0));
        assert_eq!(mend.entries.get(&11), Some(&2.5));
        assert_eq!(mend.drain(), vec![(10, 7.0), (11, 2.5)]);
        assert!(mend.is_empty());
    }

    #[test]
    fn regen_projector_range_draw_light_stats_and_effect_plans_follow_java() {
        assert_eq!(
            regen_projector_repair_time_seconds(12.0 / 60.0),
            1.0 / ((12.0 / 60.0) / 100.0) / 60.0
        );
        assert_eq!(regen_projector_repair_time_stat_seconds(12.0 / 60.0), 8);
        assert_eq!(regen_projector_range_blocks(14), 14);
        assert_eq!(regen_projector_booster_multiplier(2.0), 2.0);
        assert_eq!(
            regen_projector_stats_plan(480.0, 12.0 / 60.0, 14, 2.0, true),
            RegenProjectorStatsPlan {
                time_period: 480.0,
                repair_time_seconds: 8,
                range_blocks: 14,
                booster_multiplier: Some(2.0),
                booster_range_boost: 0.0,
            }
        );
        assert_eq!(
            regen_projector_stats_plan(480.0, 12.0 / 60.0, 14, 2.0, false).booster_multiplier,
            None
        );
        assert_eq!(regen_projector_square_size(14, 8.0), 112.0);
        assert!(regen_projector_should_consume(true));
        assert!(!regen_projector_should_consume(false));

        let place = regen_projector_place_plan(2, 3, 8.0, 4.0, 14, 0.0);
        assert_eq!(
            place,
            RegenProjectorRangePlan {
                center_x: 20.0,
                center_y: 28.0,
                square_size: 112.0,
                selected_alpha: 0.0,
            }
        );

        let select = regen_projector_select_plan(40.0, 48.0, 14, 8.0, 0.0);
        assert_eq!(
            select,
            RegenProjectorRangePlan {
                center_x: 40.0,
                center_y: 48.0,
                square_size: 112.0,
                selected_alpha: 0.0,
            }
        );

        let state = RegenProjectorState {
            warmup: 0.6,
            total_time: 10.0,
            optional_timer: 0.0,
            any_targets: true,
            did_regen: true,
        };
        assert_eq!(
            regen_projector_light_plan(&state),
            ProjectorLightPlan {
                radius: 0.0,
                alpha: 0.0,
            }
        );
        assert_eq!(
            regen_projector_draw_plan(),
            RegenProjectorDrawPlan {
                commands: &[RegenProjectorDrawCommand::DrawerDraw],
                draw_region: true,
            }
        );

        assert!((regen_projector_effect_chance_delta(0.003, 3, 2.0) - 0.054).abs() < 0.00001);
        assert_eq!(regen_projector_effect_offset_limit(3, 8.0), 11.0);
        assert!(regen_projector_should_emit_effect(0.0, 0.5, 0.49));
        assert!(!regen_projector_should_emit_effect(0.1, 0.5, 0.49));
        assert!(!regen_projector_should_emit_effect(0.0, 0.5, 0.51));

        assert_eq!(
            regen_projector_apply_plan(-1, 10),
            RegenProjectorApplyPlan {
                apply_mend_map: true,
                clear_mend_map: true,
            }
        );
        assert_eq!(
            regen_projector_apply_plan(10, 10),
            RegenProjectorApplyPlan {
                apply_mend_map: false,
                clear_mend_map: false,
            }
        );
    }

    #[test]
    fn base_and_shield_wall_helpers_follow_upstream_state_order() {
        let mut base = BaseShieldState::default();
        assert_eq!(base_shield_update(&mut base, 200.0, 0.5), 5.0);
        assert_eq!(base_shield_radius(&base), 5.0);
        assert_eq!(base_shield_clip_radius(200.0), 200.0);
        assert!(!base_shield_in_fog_to());
        assert!(base_shield_should_interact(5.0));
        assert_eq!(
            base_shield_place_plan(2, 3, 8.0, 4.0, 200.0),
            BaseShieldRangePlan {
                center_x: 20.0,
                center_y: 28.0,
                radius: 200.0,
            }
        );
        assert_eq!(
            base_shield_select_plan(40.0, 48.0, 200.0),
            BaseShieldRangePlan {
                center_x: 40.0,
                center_y: 48.0,
                radius: 200.0,
            }
        );
        assert_eq!(
            base_shield_interaction_plan(40.0, 48.0, 5.0),
            BaseShieldInteractionPlan {
                active: true,
                bullet_min_x: 35.0,
                bullet_min_y: 43.0,
                bullet_size: 10.0,
                unit_range: 15.0,
            }
        );
        assert_eq!(
            base_shield_interaction_plan(40.0, 48.0, 1.0),
            BaseShieldInteractionPlan {
                active: false,
                bullet_min_x: 40.0,
                bullet_min_y: 48.0,
                bullet_size: 0.0,
                unit_range: 0.0,
            }
        );
        assert!(base_shield_should_absorb_bullet(true, true, true));
        assert!(!base_shield_should_absorb_bullet(false, true, true));
        assert!(!base_shield_should_absorb_bullet(true, false, true));
        assert!(!base_shield_should_absorb_bullet(true, true, false));
        assert_eq!(
            base_shield_tint_plan(false, 0.5),
            BaseShieldTintPlan {
                use_team_color: true,
                hit_alpha: 128,
            }
        );
        assert_eq!(
            base_shield_tint_plan(true, 2.0),
            BaseShieldTintPlan {
                use_team_color: false,
                hit_alpha: 255,
            }
        );
        assert_eq!(
            base_shield_unit_action(10.0, 20.0, 18.0),
            ShieldUnitAction::Repel { distance: 7.01 }
        );
        assert_eq!(
            base_shield_unit_action(10.0, 20.0, 5.0),
            ShieldUnitAction::Kill
        );
        let animated = base_shield_draw_plan(false, 42.0, 24, 1.2, true);
        assert_eq!(
            animated.commands,
            &[
                BaseShieldDrawCommand::SetShieldLayer,
                BaseShieldDrawCommand::SetShieldColor,
                BaseShieldDrawCommand::FillAnimatedPoly,
                BaseShieldDrawCommand::Reset,
            ]
        );
        assert_eq!(animated.radius, 42.0);
        assert_eq!(animated.sides, 24);
        assert_eq!(animated.hit_alpha, 1.0);
        assert_eq!(animated.fill_alpha, 1.0);
        assert_eq!(animated.stroke, 0.0);

        let static_plan = base_shield_draw_plan(false, 30.0, 12, 0.5, false);
        assert_eq!(
            static_plan.commands,
            &[
                BaseShieldDrawCommand::SetShieldLayer,
                BaseShieldDrawCommand::SetShieldColor,
                BaseShieldDrawCommand::StrokeStaticPoly,
                BaseShieldDrawCommand::Reset,
            ]
        );
        assert_eq!(static_plan.fill_alpha, 0.09 + 0.08 * 0.5);
        assert_eq!(static_plan.stroke, 1.5);

        let broken = base_shield_draw_plan(true, 30.0, 12, 0.5, false);
        assert_eq!(broken.commands, &[BaseShieldDrawCommand::Reset]);
        assert_eq!(broken.fill_alpha, 0.0);

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
        let animated_wall =
            shield_wall_draw_plan(wall.shield_radius, 2, 8.0, wall.hit, true, 0.6, 0.25);
        assert_eq!(
            animated_wall.commands,
            &[
                ShieldWallDrawCommand::Region,
                ShieldWallDrawCommand::SetShieldLayer,
                ShieldWallDrawCommand::SetShieldColor,
                ShieldWallDrawCommand::FillAnimatedSquare,
                ShieldWallDrawCommand::ResetShield,
                ShieldWallDrawCommand::Glow,
            ]
        );
        assert_eq!(animated_wall.radius, 8.0);
        assert_eq!(animated_wall.hit_alpha, 0.8);
        assert_eq!(animated_wall.fill_alpha, 1.0);
        assert_eq!(animated_wall.stroke, 0.0);
        assert_eq!(animated_wall.glow_alpha, 0.65);

        let static_wall = shield_wall_draw_plan(wall.shield_radius, 2, 8.0, 1.5, false, 0.6, 0.25);
        assert_eq!(
            static_wall.commands,
            &[
                ShieldWallDrawCommand::Region,
                ShieldWallDrawCommand::SetShieldLayer,
                ShieldWallDrawCommand::SetShieldColor,
                ShieldWallDrawCommand::StrokeStaticSquare,
                ShieldWallDrawCommand::ResetShield,
                ShieldWallDrawCommand::Glow,
            ]
        );
        assert_eq!(static_wall.hit_alpha, 1.0);
        assert!((static_wall.fill_alpha - (0.09 + 0.08 * 1.5)).abs() < 0.00001);
        assert_eq!(static_wall.stroke, 1.5);

        let region_only_wall = shield_wall_draw_plan(0.0, 2, 8.0, 0.8, true, 0.6, 0.25);
        assert_eq!(region_only_wall.commands, &[ShieldWallDrawCommand::Region]);
        assert_eq!(region_only_wall.radius, 0.0);
        assert_eq!(region_only_wall.glow_alpha, 0.0);

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
        assert_eq!(directional_force_projector_clip_radius(30.0), 33.0);
        assert_eq!(
            directional_force_projector_effective_length(-1.0, 2, 8.0),
            8.0
        );
        assert_eq!(
            directional_force_projector_stats_plan(3000.0, 0.35),
            DirectionalForceProjectorStatsPlan {
                shield_health: 3000.0,
                cooldown_time_seconds: 142,
            }
        );
        assert_eq!(
            directional_force_projector_bar_fraction(&directional, 3000.0),
            1.0
        );
        assert!(!directional_force_projector_outputs_items());
        assert!(directional_force_projector_should_ambient_sound(
            &directional
        ));
        let place = directional_force_projector_place_plan(2, 3, 8.0, 40.0, 30.0, 2, 0);
        assert_eq!(
            place,
            DirectionalForceProjectorPlacePlan {
                origin: (16.0, 24.0),
                top_point: (55.0, 55.0),
                bottom_point: (55.0, -7.0),
            }
        );
        let segment = directional_force_projector_segment(0.0, 0.0, 40.0, 10.0, 0.0);
        assert_eq!(segment, ((40.0, 10.0), (40.0, -10.0)));
        let deflect_plan =
            directional_force_projector_deflect_plan(&directional, 0.0, 0.0, 40.0, 0.0, 40.0);
        assert!(deflect_plan.active);
        assert_eq!(deflect_plan.segment, ((40.0, 11.0), (40.0, -11.0)));
        assert_eq!(
            deflect_plan.bounds,
            DirectionalForceProjectorRect {
                x: 0.0,
                y: -51.0,
                width: 80.0,
                height: 102.0,
            }
        );
        let animated_draw =
            directional_force_projector_draw_plan(&directional, 0.0, 0.0, 40.0, 0.0, 3000.0, true);
        assert_eq!(
            animated_draw.commands,
            &[
                DirectionalForceProjectorDrawCommand::SuperDraw,
                DirectionalForceProjectorDrawCommand::SetShieldLayer,
                DirectionalForceProjectorDrawCommand::SetShieldColor,
                DirectionalForceProjectorDrawCommand::FillAnimatedRect,
                DirectionalForceProjectorDrawCommand::DrawAnimatedEdges,
                DirectionalForceProjectorDrawCommand::FillAnimatedCaps,
                DirectionalForceProjectorDrawCommand::ResetShield,
            ]
        );
        assert_eq!(animated_draw.fill_alpha, 1.0);
        assert_eq!(animated_draw.edge_stroke, 3.0);
        let static_draw = directional_force_projector_draw_plan(
            &DirectionalForceProjectorState {
                buildup: 300.0,
                hit: 2.0,
                ..directional
            },
            0.0,
            0.0,
            40.0,
            0.0,
            3000.0,
            false,
        );
        assert_eq!(
            static_draw.commands,
            &[
                DirectionalForceProjectorDrawCommand::SuperDraw,
                DirectionalForceProjectorDrawCommand::TopAdditive,
                DirectionalForceProjectorDrawCommand::ResetTop,
                DirectionalForceProjectorDrawCommand::SetShieldLayer,
                DirectionalForceProjectorDrawCommand::SetShieldColor,
                DirectionalForceProjectorDrawCommand::FillStaticRect,
                DirectionalForceProjectorDrawCommand::StrokeStaticRect,
                DirectionalForceProjectorDrawCommand::ResetShield,
            ]
        );
        assert_eq!(static_draw.top_alpha, 0.075);
        assert_eq!(static_draw.hit_alpha, 1.0);
        assert_eq!(static_draw.fill_alpha, 0.25);
        assert_eq!(static_draw.stroke, 1.5);
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
        assert_eq!(directional.warmup, 0.0);

        let mut radar = RadarState::default();
        assert!(!radar_update(&mut radar, 1.0, 60.0, 10.0, 600.0));
        assert_eq!(radar.progress, 0.1);
        assert_eq!(radar.total_progress, 60.0);
        assert_eq!(
            radar_fog_radius(10.0, radar.progress, radar.smooth_efficiency),
            1.0
        );
        assert!(!radar_force_update_needed(10.0, 0.049, 1.0, 0.0));
        assert!(radar_force_update_needed(10.0, 0.05, 1.0, 0.0));
        assert!(radar_force_update_needed(10.0, 0.051, 1.0, 0.0));
        assert!(radar_update(&mut radar, 1.0, 60.0, 10.0, 600.0));
        assert_eq!(radar.progress, 0.2);
        assert_eq!(radar.last_radius, 1.0);
        radar.progress = 0.99;
        assert!(radar_update(&mut radar, 1.0, 60.0, 10.0, 600.0));
        assert_eq!(radar.progress, 1.0);
        let mut bytes = Vec::new();
        write_radar_state(&mut bytes, &radar).unwrap();
        assert_eq!(bytes.len(), 4);
        let restored_radar = read_radar_state(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored_radar.progress, radar.progress);
        assert_eq!(restored_radar.last_radius, 0.0);
        assert_eq!(restored_radar.smooth_efficiency, 1.0);
        assert_eq!(restored_radar.total_progress, 0.0);
        assert_eq!(
            radar_icons(),
            &[RadarIconRegion::Base, RadarIconRegion::Main]
        );
        assert!(!radar_can_pickup());
        assert_eq!(radar_progress(&radar), radar.progress);
        assert_eq!(
            radar_place_plan(2, 3, 8.0, 4.0, 10.0),
            RadarRangePlan {
                center_x: 20.0,
                center_y: 28.0,
                radius: 80.0,
            }
        );
        assert_eq!(
            radar_select_plan(
                40.0,
                48.0,
                10.0,
                radar.progress,
                radar.smooth_efficiency,
                8.0
            ),
            RadarRangePlan {
                center_x: 40.0,
                center_y: 48.0,
                radius: radar_fog_radius(10.0, radar.progress, radar.smooth_efficiency) * 8.0,
            }
        );
        assert_eq!(radar_draw_rotation(2.0, radar.total_progress), 360.0);
        let draw = radar_draw_plan(&radar, 0.0, 2.0, 5.0, 0.6, 1.0);
        assert_eq!(
            draw.commands,
            &[
                RadarDrawCommand::Base,
                RadarDrawCommand::Region,
                RadarDrawCommand::GlowAdditive,
            ]
        );
        assert_eq!(draw.rotation, 360.0);
        assert!((draw.glow_alpha - 0.4).abs() < 0.00001);

        assert_eq!(
            build_turret_stats_plan(1.5),
            BuildTurretStatsPlan {
                build_speed_percent: 1.5,
            }
        );
        assert_eq!(
            build_turret_icons(),
            &[BuildTurretIconRegion::Base, BuildTurretIconRegion::Main]
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
        let thruster = ThrusterBlockConfig::default();
        assert!(thruster.rotate);
        assert!(!thruster.quick_rotate);
        let draw = thruster_draw_plan(16.0, 24.0, 2);
        assert_eq!(
            draw.commands,
            &[ThrusterDrawCommand::Region, ThrusterDrawCommand::Top]
        );
        assert_eq!((draw.x, draw.y), (16.0, 24.0));
        assert_eq!(draw.top_rotation, 180.0);
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
    fn build_turret_state_plans_bridge_unit_builder_queue() {
        let mut unit = UnitComp::new(
            12,
            build_turret_unit_type(-1, "build-tower", 90.0, 5.0, 80.0, 1.0),
            TeamId(1),
        );
        unit.builder
            .plans
            .push_back(BuildPlan::new_place(1, 2, 0, "router"));
        unit.builder
            .plans
            .push_back(BuildPlan::new_string_config(3, 4, 1, "conveyor", "cfg"));
        let mut state = BuildTurretState {
            raw_plans: vec![1, 2, 3],
            ..BuildTurretState::default()
        };

        build_turret_capture_unit_plans(&mut state, &unit);
        assert_eq!(
            state.plans,
            vec![
                BuildPlan::new_place(1, 2, 0, "router"),
                BuildPlan::new_string_config(3, 4, 1, "conveyor", "cfg"),
            ]
        );
        assert!(state.raw_plans.is_empty());

        unit.builder.plans.clear();
        build_turret_apply_unit_plans(&state, &mut unit);
        assert_eq!(unit.builder.plans.len(), 2);
        assert_eq!(
            unit.builder.plans.front(),
            Some(&BuildPlan::new_place(1, 2, 0, "router"))
        );
        assert_eq!(
            unit.builder.plans.back(),
            Some(&BuildPlan::new_string_config(3, 4, 1, "conveyor", "cfg"))
        );
        assert!(unit.is_building());
    }

    #[test]
    fn build_turret_sense_helpers_forward_unit_build_plan_fields() {
        let place = BuildPlan::new_place(7, 8, 1, "router");
        let breaking = BuildPlan::new_break(9, 10);

        assert_eq!(
            build_turret_sense_from_plan(LAccess::BuildX, Some(&place)),
            Some(7.0)
        );
        assert_eq!(
            build_turret_sense_from_plan(LAccess::BuildY, Some(&place)),
            Some(8.0)
        );
        assert_eq!(
            build_turret_sense_from_plan(LAccess::BuildX, None),
            Some(-1.0)
        );
        assert_eq!(
            build_turret_sense_from_plan(LAccess::Health, Some(&place)),
            None
        );

        assert_eq!(
            build_turret_sense_object_from_plan(LAccess::Building, Some(&place)),
            Some(BuildTurretSenseObject::BuildingAt { x: 7, y: 8 })
        );
        assert_eq!(
            build_turret_sense_object_from_plan(LAccess::Breaking, Some(&place)),
            None
        );
        assert_eq!(
            build_turret_sense_object_from_plan(LAccess::Breaking, Some(&breaking)),
            Some(BuildTurretSenseObject::BuildingAt { x: 9, y: 10 })
        );
        assert_eq!(
            build_turret_sense_object_from_plan(LAccess::Building, Some(&breaking)),
            None
        );
        assert_eq!(
            build_turret_sense_object_from_plan(LAccess::Building, None),
            None
        );
        assert_eq!(
            build_turret_sense_object_from_plan(LAccess::Health, Some(&place)),
            None
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
        assert_eq!(
            shockwave_tower_stats_plan(160.0, 110.0, 8.0, 90.0),
            ShockwaveTowerStatsPlan {
                damage: 160.0,
                range_blocks: 13.75,
                reload_per_second: 60.0 / 90.0,
            }
        );
        assert_eq!(
            shockwave_tower_place_plan(2, 3, 8.0, 4.0, 110.0),
            ShockwaveTowerRangePlan {
                center_x: 20.0,
                center_y: 28.0,
                radius: 110.0,
            }
        );
        assert_eq!(
            shockwave_tower_select_plan(40.0, 48.0, 110.0),
            ShockwaveTowerRangePlan {
                center_x: 40.0,
                center_y: 48.0,
                radius: 110.0,
            }
        );
        assert_eq!(shockwave_tower_wave_damage(160.0, 20.0, 4), 160.0);
        assert_eq!(shockwave_tower_wave_damage(160.0, 20.0, 20), 160.0);
        assert_eq!(shockwave_tower_wave_damage(160.0, 20.0, 40), 80.0);
        assert_eq!(shockwave_tower_wave_damage(160.0, 20.0, 0), 0.0);
        assert!(shockwave_tower_can_target(true, true));
        assert!(!shockwave_tower_can_target(false, true));
        assert!(!shockwave_tower_can_target(true, false));
        assert!(shockwave_tower_can_fire(1.0, 90.0, 90.0, true, 1));
        assert!(!shockwave_tower_can_fire(0.0, 90.0, 90.0, true, 1));
        assert!(!shockwave_tower_can_fire(1.0, 89.0, 90.0, true, 1));
        assert!(!shockwave_tower_can_fire(1.0, 90.0, 90.0, false, 1));
        assert!(!shockwave_tower_can_fire(1.0, 90.0, 90.0, true, 0));
        assert_eq!(shockwave_tower_apply_damage(200.0, 160.0), (40.0, false));
        assert_eq!(shockwave_tower_apply_damage(160.0, 160.0), (0.0, true));
        assert_eq!(shockwave_tower_apply_damage(10.0, 160.0), (0.0, true));
        assert_eq!(
            shockwave_tower_heat_after_cooldown(0.1, 45.0, 90.0, 1.0),
            0.0
        );
        assert_eq!(
            shockwave_tower_sense(LAccess::Progress, &tower, 90.0),
            Some(0.0)
        );
        assert_eq!(shockwave_tower_sense(LAccess::Health, &tower, 90.0), None);
        assert_eq!(shockwave_tower_warmup(&tower), tower.heat);
        let draw = shockwave_tower_draw_plan(&tower, 30.0, 0.5, 4, 6.0, 1.0);
        assert_eq!(
            draw.commands,
            &[
                ShockwaveTowerDrawCommand::SuperDraw,
                ShockwaveTowerDrawCommand::HeatAdditive,
                ShockwaveTowerDrawCommand::SetEffectLayer,
                ShockwaveTowerDrawCommand::FillShape,
                ShockwaveTowerDrawCommand::ResetColor,
            ]
        );
        assert_eq!(draw.heat_alpha, tower.heat);
        assert!((draw.color_lerp - tower.heat.powi(2)).abs() < 0.00001);
        assert_eq!(draw.shape_sides, 4);
        assert_eq!(draw.shape_radius, 3.0);
        assert_eq!(draw.shape_rotation, 30.0);
        assert_eq!(shockwave_tower_progress(45.0, 90.0), 0.5);
        assert!(shockwave_tower_should_consume(45.0, 90.0));
        assert!(!shockwave_tower_should_consume(90.0, 90.0));

        let mut idle = ShockwaveTowerState {
            reload_counter: 90.0,
            heat: 0.5,
        };
        let mut empty = [];
        let no_targets = shockwave_tower_update(
            &mut idle, 1.0, 1.0, 1.0, 90.0, 160.0, 20.0, &mut empty, true, 1.0,
        );
        assert!(!no_targets.fired);
        assert_eq!(idle.reload_counter, 91.0);
        assert!(idle.heat < 0.5);

        let mut no_efficiency = ShockwaveTowerState {
            reload_counter: 10.0,
            heat: 0.5,
        };
        let mut one_target = [100.0];
        let blocked = shockwave_tower_update(
            &mut no_efficiency,
            0.0,
            5.0,
            1.0,
            90.0,
            160.0,
            20.0,
            &mut one_target,
            true,
            1.0,
        );
        assert!(!blocked.fired);
        assert_eq!(no_efficiency.reload_counter, 10.0);
    }
}
