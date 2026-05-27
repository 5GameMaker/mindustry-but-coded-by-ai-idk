use std::io::{self, Read, Write};

use crate::mindustry::{
    ctype::{ContentId, ContentType},
    io::{
        read_vec2,
        type_io::{read_i16, write_i16},
        write_vec2, Vec2,
    },
    r#type::{PayloadKey, PayloadSeq},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitBlockState {
    pub progress: f32,
    pub time: f32,
    pub speed_scl: f32,
    pub has_payload: bool,
}

impl Default for UnitBlockState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            time: 0.0,
            speed_scl: 0.0,
            has_payload: false,
        }
    }
}

pub fn unit_block_spawned(state: &mut UnitBlockState) {
    state.progress = 0.0;
    state.has_payload = false;
}

pub fn unit_block_dump_payload(dumped: bool, state: &mut UnitBlockState) -> bool {
    if dumped {
        unit_block_spawned(state);
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitPlanRequirement {
    pub item_id: usize,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPlan {
    pub unit_id: i32,
    pub time: f32,
    pub requirements: Vec<UnitPlanRequirement>,
    pub banned: bool,
}

pub fn unit_factory_capacities(item_count: usize, plans: &[UnitPlan]) -> (Vec<i32>, i32) {
    let mut capacities = vec![0; item_count];
    let mut item_capacity = 10;
    for plan in plans {
        for stack in &plan.requirements {
            if stack.item_id < capacities.len() {
                capacities[stack.item_id] = capacities[stack.item_id].max(stack.amount * 2);
            }
            item_capacity = item_capacity.max(stack.amount * 2);
        }
    }
    (capacities, item_capacity)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitFactoryState {
    pub base: UnitBlockState,
    pub current_plan: i32,
    pub command_pos: Option<Vec2>,
    pub command_id: Option<u8>,
}

impl Default for UnitFactoryState {
    fn default() -> Self {
        Self {
            base: UnitBlockState::default(),
            current_plan: -1,
            command_pos: None,
            command_id: None,
        }
    }
}

pub fn unit_factory_configure_plan(
    state: &mut UnitFactoryState,
    requested: i32,
    plan_count: usize,
    command_valid_for_next_unit: bool,
) {
    if state.current_plan == requested {
        return;
    }
    state.current_plan = if requested < 0 || requested as usize >= plan_count {
        -1
    } else {
        requested
    };
    state.base.progress = 0.0;
    if !command_valid_for_next_unit {
        state.command_id = None;
    }
}

pub fn unit_factory_fraction(current_plan: i32, progress: f32, plan_time: f32) -> f32 {
    if current_plan == -1 || plan_time == 0.0 {
        0.0
    } else {
        progress / plan_time
    }
}

pub fn unit_factory_should_consume(
    current_plan: i32,
    enabled: bool,
    has_payload: bool,
    team_activates_factories: bool,
) -> bool {
    current_plan != -1 && enabled && !has_payload && team_activates_factories
}

pub fn unit_factory_accept_item(
    current_plan: i32,
    stored: i32,
    maximum_accepted: i32,
    plan_contains_item: bool,
) -> bool {
    current_plan != -1 && stored < maximum_accepted && plan_contains_item
}

pub fn unit_factory_maximum_accepted(base_capacity: i32, unit_cost: f32) -> i32 {
    (base_capacity as f32 * unit_cost).round() as i32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitFactoryUpdate {
    pub created_unit: bool,
    pub progress: f32,
    pub time: f32,
    pub speed_scl: f32,
    pub current_plan: i32,
}

#[allow(clippy::too_many_arguments)]
pub fn unit_factory_update(
    state: &mut UnitFactoryState,
    configurable: bool,
    plan_count: usize,
    efficiency: f32,
    edelta: f32,
    unit_build_speed: f32,
    plan_time: f32,
    plan_banned: bool,
) -> UnitFactoryUpdate {
    if !configurable {
        state.current_plan = 0;
    }
    if state.current_plan < 0 || state.current_plan as usize >= plan_count {
        state.current_plan = -1;
    }

    if efficiency > 0.0 && state.current_plan != -1 {
        state.base.time += edelta * state.base.speed_scl * unit_build_speed;
        state.base.progress += edelta * unit_build_speed;
        state.base.speed_scl = lerp_delta(state.base.speed_scl, 1.0, 0.05);
    } else {
        state.base.speed_scl = lerp_delta(state.base.speed_scl, 0.0, 0.05);
    }

    let mut created_unit = false;
    if state.current_plan != -1 && !state.base.has_payload {
        if plan_banned {
            state.current_plan = -1;
        } else if state.base.progress >= plan_time {
            state.base.progress %= 1.0;
            state.base.has_payload = true;
            created_unit = true;
        }
        state.base.progress = state.base.progress.clamp(0.0, plan_time);
    } else {
        state.base.progress = 0.0;
    }

    UnitFactoryUpdate {
        created_unit,
        progress: state.base.progress,
        time: state.base.time,
        speed_scl: state.base.speed_scl,
        current_plan: state.current_plan,
    }
}

pub fn write_unit_factory_state<W: Write>(
    write: &mut W,
    state: &UnitFactoryState,
) -> io::Result<()> {
    write_f32(write, state.base.progress)?;
    write_i16(write, state.current_plan as i16)?;
    write_vec_nullable(write, state.command_pos)?;
    write_command(write, state.command_id)
}

pub fn read_unit_factory_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<UnitFactoryState> {
    let progress = read_f32(read)?;
    let current_plan = read_i16(read)? as i32;
    let command_pos = if revision >= 2 {
        read_vec_nullable(read)?
    } else {
        None
    };
    let command_id = if revision >= 3 {
        read_command(read)?
    } else {
        None
    };
    Ok(UnitFactoryState {
        base: UnitBlockState {
            progress,
            ..UnitBlockState::default()
        },
        current_plan,
        command_pos,
        command_id,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairTowerState {
    pub refresh: f32,
    pub warmup: f32,
    pub total_progress: f32,
}

impl Default for RepairTowerState {
    fn default() -> Self {
        Self {
            refresh: 0.0,
            warmup: 0.0,
            total_progress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairTowerUpdate {
    pub should_refresh_targets: bool,
    pub should_consume: bool,
    pub heal_per_target: f32,
}

pub fn repair_tower_update(
    state: &mut RepairTowerState,
    potential_efficiency: f32,
    efficiency: f32,
    suppressed: bool,
    target_count: usize,
    damaged_target_count: usize,
    heal_amount: f32,
    delta: f32,
    circle_speed: f32,
) -> RepairTowerUpdate {
    let mut should_refresh_targets = false;
    if potential_efficiency > 0.0 {
        state.refresh += delta;
        if state.refresh >= REPAIR_TOWER_REFRESH_INTERVAL {
            state.refresh = 0.0;
            should_refresh_targets = true;
        }
    }

    if suppressed {
        state.warmup = 0.0;
        return RepairTowerUpdate {
            should_refresh_targets,
            should_consume: target_count > 0,
            heal_per_target: 0.0,
        };
    }

    let any = efficiency > 0.0 && damaged_target_count > 0;
    let heal_per_target = if efficiency > 0.0 {
        heal_amount * delta * efficiency
    } else {
        0.0
    };
    state.warmup = lerp_delta(state.warmup, if any { efficiency } else { 0.0 }, 0.08);
    state.total_progress += delta / circle_speed;

    RepairTowerUpdate {
        should_refresh_targets,
        should_consume: target_count > 0,
        heal_per_target,
    }
}

pub const REPAIR_TOWER_REFRESH_INTERVAL: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconstructorState {
    pub base: UnitBlockState,
    pub command_pos: Option<Vec2>,
    pub command_id: Option<u8>,
    pub constructing: bool,
}

impl Default for ReconstructorState {
    fn default() -> Self {
        Self {
            base: UnitBlockState::default(),
            command_pos: None,
            command_id: None,
            constructing: false,
        }
    }
}

pub fn reconstructor_capacities(
    item_count: usize,
    consume_items: &[UnitPlanRequirement],
) -> (Vec<i32>, i32) {
    let mut capacities = vec![0; item_count];
    let mut item_capacity = 10;
    for stack in consume_items {
        if stack.item_id < capacities.len() {
            capacities[stack.item_id] = capacities[stack.item_id].max(stack.amount * 2);
        }
        item_capacity = item_capacity.max(stack.amount * 2);
    }
    (capacities, item_capacity)
}

pub fn reconstructor_accept_item(stored: i32, maximum_accepted: i32) -> bool {
    maximum_accepted > 0 && stored < maximum_accepted
}

pub fn reconstructor_maximum_accepted(base_capacity: i32, unit_cost: f32) -> i32 {
    (base_capacity as f32 * unit_cost).round() as i32
}

pub fn reconstructor_fraction(progress: f32, construct_time: f32) -> f32 {
    if construct_time == 0.0 {
        0.0
    } else {
        progress / construct_time
    }
}

pub fn reconstructor_accept_payload(
    payload_empty: bool,
    enabled_or_self_source: bool,
    source_not_output_side: bool,
    has_upgrade: bool,
    upgrade_unlocked_or_ai: bool,
    upgrade_banned: bool,
) -> bool {
    payload_empty
        && enabled_or_self_source
        && source_not_output_side
        && has_upgrade
        && upgrade_unlocked_or_ai
        && !upgrade_banned
}

pub fn reconstructor_should_consume(
    constructing: bool,
    enabled: bool,
    team_activates_factories: bool,
) -> bool {
    constructing && enabled && team_activates_factories
}

pub fn reconstructor_update(
    state: &mut ReconstructorState,
    has_payload: bool,
    has_upgrade: bool,
    moved_in: bool,
    efficiency: f32,
    edelta: f32,
    unit_build_speed: f32,
    construct_time: f32,
) -> bool {
    state.constructing = has_payload && has_upgrade;
    let mut upgraded = false;
    let mut valid = false;
    if state.constructing && moved_in {
        if efficiency > 0.0 {
            valid = true;
            state.base.progress += edelta * unit_build_speed;
        }
        if state.base.progress >= construct_time {
            state.base.progress %= 1.0;
            upgraded = true;
        }
    }
    state.base.speed_scl = lerp_delta(state.base.speed_scl, if valid { 1.0 } else { 0.0 }, 0.05);
    state.base.time += edelta * state.base.speed_scl * unit_build_speed;
    upgraded
}

pub fn write_reconstructor_state<W: Write>(
    write: &mut W,
    state: &ReconstructorState,
) -> io::Result<()> {
    write_f32(write, state.base.progress)?;
    write_vec_nullable(write, state.command_pos)?;
    write_command(write, state.command_id)
}

pub fn read_reconstructor_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<ReconstructorState> {
    let progress = if revision >= 1 { read_f32(read)? } else { 0.0 };
    let command_pos = if revision >= 2 {
        read_vec_nullable(read)?
    } else {
        None
    };
    let command_id = if revision >= 3 {
        read_command(read)?
    } else {
        None
    };
    Ok(ReconstructorState {
        base: UnitBlockState {
            progress,
            ..UnitBlockState::default()
        },
        command_pos,
        command_id,
        constructing: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssemblerRect {
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

pub fn unit_assembler_rect(
    x: f32,
    y: f32,
    rotation: i32,
    area_size: i32,
    block_size: i32,
    tile_size: f32,
) -> AssemblerRect {
    let (dx, dy) = direction(rotation);
    let len = tile_size * (area_size + block_size) as f32 / 2.0;
    AssemblerRect {
        x: x + dx as f32 * len,
        y: y + dy as f32 * len,
        size: area_size as f32 * tile_size,
    }
}

pub fn unit_assembler_current_tier(mut tiers: Vec<i32>) -> i32 {
    tiers.sort_unstable();
    let mut max = 0;
    for tier in tiers {
        if tier == max || tier == max + 1 {
            max = tier;
        } else {
            break;
        }
    }
    max
}

pub fn unit_assembler_accept_item(
    plan_has_item_req: bool,
    stored: i32,
    maximum_accepted: i32,
    plan_contains_item: bool,
) -> bool {
    plan_has_item_req && stored < maximum_accepted && plan_contains_item
}

pub fn unit_assembler_accept_payload(
    payload_slot_empty: bool,
    source_is_module: bool,
    requirement_amount: i32,
    stored_blocks: i32,
    unit_cost: f32,
    same_payload_already_held_from_module: bool,
) -> bool {
    (payload_slot_empty || source_is_module)
        && stored_blocks
            < (requirement_amount as f32 * unit_cost).round() as i32
                - if same_payload_already_held_from_module {
                    1
                } else {
                    0
                }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAssemblerState {
    pub progress: f32,
    pub warmup: f32,
    pub drone_warmup: f32,
    pub power_warmup: f32,
    pub same_type_warmup: f32,
    pub invalid_warmup: f32,
    pub drone_progress: f32,
    pub total_drone_progress: f32,
    pub current_tier: i32,
    pub last_tier: i32,
    pub was_occupied: bool,
    pub read_unit_ids: Vec<i32>,
    pub blocks: PayloadSeq,
    pub command_pos: Option<Vec2>,
}

impl Default for UnitAssemblerState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            warmup: 0.0,
            drone_warmup: 0.0,
            power_warmup: 0.0,
            same_type_warmup: 0.0,
            invalid_warmup: 0.0,
            drone_progress: 0.0,
            total_drone_progress: 0.0,
            current_tier: 0,
            last_tier: -2,
            was_occupied: false,
            read_unit_ids: Vec::new(),
            blocks: PayloadSeq::new(),
            command_pos: None,
        }
    }
}

pub fn write_unit_assembler_state<W: Write>(
    write: &mut W,
    state: &UnitAssemblerState,
) -> io::Result<()> {
    write_f32(write, state.progress)?;
    write.write_all(&[state.read_unit_ids.len() as u8])?;
    for id in &state.read_unit_ids {
        write_i32(write, *id)?;
    }
    write_payload_seq(write, &state.blocks)?;
    write_vec_nullable(write, state.command_pos)
}

pub fn read_unit_assembler_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<UnitAssemblerState> {
    let progress = read_f32(read)?;
    let mut count = [0; 1];
    read.read_exact(&mut count)?;
    let mut read_unit_ids = Vec::with_capacity(count[0] as usize);
    for _ in 0..count[0] {
        read_unit_ids.push(read_i32(read)?);
    }
    let blocks = read_payload_seq(read)?;
    let command_pos = if revision >= 1 {
        read_vec_nullable(read)?
    } else {
        None
    };

    Ok(UnitAssemblerState {
        progress,
        read_unit_ids,
        blocks,
        command_pos,
        ..UnitAssemblerState::default()
    })
}

pub fn unit_assembler_update_progress(
    state: &mut UnitAssemblerState,
    enabled: bool,
    power_status: f32,
    units: usize,
    drones_created: usize,
    efficiency: f32,
    can_create: bool,
    requirements_met: bool,
    drones_in_position: usize,
    delta: f32,
    edelta: f32,
    unit_build_speed: f32,
    drone_construct_time: f32,
    plan_time: f32,
) -> bool {
    if state.last_tier != state.current_tier {
        if state.last_tier >= 0 {
            state.progress = 0.0;
        }
        state.last_tier = if state.last_tier == -2 {
            -1
        } else {
            state.current_tier
        };
    }
    let pstatus = if !enabled { 0.0 } else { power_status };
    state.power_warmup = lerp_delta(pstatus, if pstatus > 0.0001 { 1.0 } else { 0.0 }, 0.1);
    state.drone_warmup = lerp_delta(
        state.drone_warmup,
        if units < drones_created { pstatus } else { 0.0 },
        0.1,
    );
    state.total_drone_progress += state.drone_warmup * delta;
    let mut drone_spawned = false;
    if units < drones_created {
        state.drone_progress +=
            delta * unit_build_speed * pstatus / drone_construct_time.max(f32::EPSILON);
        if state.drone_progress >= 1.0 {
            drone_spawned = true;
        }
    } else {
        state.drone_progress = 0.0;
    }

    let eff = if drones_created == 0 {
        0.0
    } else {
        drones_in_position as f32 / drones_created as f32
    };
    if !state.was_occupied && efficiency > 0.0 && can_create && requirements_met {
        state.warmup = lerp_delta(state.warmup, efficiency, 0.1);
        state.progress += edelta * unit_build_speed * eff / plan_time;
    } else {
        state.warmup = lerp_delta(state.warmup, 0.0, 0.1);
    }
    drone_spawned
}

pub fn unit_assembler_spawned(state: &mut UnitAssemblerState) {
    state.progress = 0.0;
    state.blocks.clear();
}

pub fn unit_assembler_drone_spawned(
    state: &mut UnitAssemblerState,
    id: i32,
    net_client: bool,
) -> bool {
    if id < 0 {
        return false;
    }
    state.drone_progress = 0.0;
    if net_client && !state.read_unit_ids.contains(&id) {
        state.read_unit_ids.push(id);
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitCargoLoaderState {
    pub read_unit_id: i32,
    pub build_progress: f32,
    pub total_progress: f32,
    pub warmup: f32,
    pub readyness: f32,
    pub has_unit: bool,
}

impl Default for UnitCargoLoaderState {
    fn default() -> Self {
        Self {
            read_unit_id: -1,
            build_progress: 0.0,
            total_progress: 0.0,
            warmup: 0.0,
            readyness: 0.0,
            has_unit: false,
        }
    }
}

pub fn unit_cargo_loader_update(
    state: &mut UnitCargoLoaderState,
    efficiency: f32,
    can_create: bool,
    edelta: f32,
    delta: f32,
    unit_build_time: f32,
) -> bool {
    state.warmup = approach_delta(state.warmup, efficiency, 1.0 / 60.0);
    state.readyness = approach_delta(
        state.readyness,
        if state.has_unit { 1.0 } else { 0.0 },
        1.0 / 60.0,
    );
    if !state.has_unit && can_create {
        state.build_progress += edelta / unit_build_time;
        state.total_progress += edelta;
        state.build_progress >= 1.0
    } else {
        let _ = delta;
        false
    }
}

pub fn unit_cargo_loader_spawned(state: &mut UnitCargoLoaderState, id: i32, net_client: bool) {
    state.build_progress = 0.0;
    if net_client {
        state.read_unit_id = id;
    }
}

pub fn unit_cargo_loader_accept_item(total_items: i32, item_capacity: i32) -> bool {
    total_items < item_capacity
}

pub fn write_unit_cargo_loader_state<W: Write>(
    write: &mut W,
    unit_id: Option<i32>,
) -> io::Result<()> {
    write_i32(write, unit_id.unwrap_or(-1))
}

pub fn read_unit_cargo_loader_state<R: Read>(read: &mut R) -> io::Result<UnitCargoLoaderState> {
    Ok(UnitCargoLoaderState {
        read_unit_id: read_i32(read)?,
        ..UnitCargoLoaderState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitCargoUnloadPointState {
    pub item_id: Option<i32>,
    pub stale_timer: f32,
    pub stale: bool,
}

pub fn unit_cargo_unload_update(
    state: &mut UnitCargoUnloadPointState,
    items_total: i32,
    item_capacity: i32,
    dumped: bool,
    delta: f32,
    stale_time_duration: f32,
) {
    if items_total < item_capacity {
        state.stale_timer = 0.0;
        state.stale = false;
    }
    if dumped {
        state.stale_timer = 0.0;
        state.stale = false;
    } else if items_total >= item_capacity {
        state.stale_timer += delta;
        if state.stale_timer >= stale_time_duration {
            state.stale = true;
        }
    }
}

pub fn unit_cargo_unload_accept_stack(item_capacity: i32, items_total: i32, amount: i32) -> i32 {
    (item_capacity - items_total).min(amount).max(0)
}

pub fn write_unit_cargo_unload_state<W: Write>(
    write: &mut W,
    state: &UnitCargoUnloadPointState,
) -> io::Result<()> {
    write_i16(write, state.item_id.unwrap_or(-1) as i16)?;
    write.write_all(&[state.stale as u8])
}

pub fn read_unit_cargo_unload_state<R: Read>(
    read: &mut R,
) -> io::Result<UnitCargoUnloadPointState> {
    let id = read_i16(read)? as i32;
    let mut stale = [0; 1];
    read.read_exact(&mut stale)?;
    Ok(UnitCargoUnloadPointState {
        item_id: (id != -1).then_some(id),
        stale_timer: 0.0,
        stale: stale[0] != 0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairTurretState {
    pub target_present: bool,
    pub strength: f32,
    pub rotation: f32,
}

impl Default for RepairTurretState {
    fn default() -> Self {
        Self {
            target_present: false,
            strength: 0.0,
            rotation: 90.0,
        }
    }
}

pub fn repair_turret_multiplier(
    accept_coolant: bool,
    heat_capacity: f32,
    coolant_multiplier: f32,
    optional_efficiency: f32,
) -> f32 {
    if accept_coolant {
        1.0 + heat_capacity * coolant_multiplier * optional_efficiency
    } else {
        1.0
    }
}

pub fn repair_turret_target_valid(
    target_present: bool,
    target_dead: bool,
    target_distance: f32,
    target_hit_size: f32,
    repair_radius: f32,
    target_health: f32,
    target_max_health: f32,
) -> bool {
    target_present
        && !target_dead
        && target_distance - target_hit_size / 2.0 <= repair_radius
        && target_health < target_max_health
}

pub fn repair_turret_heal_amount(
    repair_speed: f32,
    strength: f32,
    edelta: f32,
    multiplier: f32,
    angle_dist: f32,
) -> f32 {
    if angle_dist < 30.0 {
        repair_speed * strength * edelta * multiplier
    } else {
        0.0
    }
}

pub fn repair_turret_update_strength(strength: f32, healed: bool, delta: f32) -> f32 {
    lerp_delta(strength, if healed { 1.0 } else { 0.0 }, 0.08 * delta)
}

pub fn repair_turret_should_consume(target_present: bool, enabled: bool) -> bool {
    target_present && enabled
}

pub fn write_repair_turret_state<W: Write>(
    write: &mut W,
    state: &RepairTurretState,
) -> io::Result<()> {
    write_f32(write, state.rotation)
}

pub fn read_repair_turret_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<RepairTurretState> {
    Ok(RepairTurretState {
        rotation: if revision >= 1 { read_f32(read)? } else { 90.0 },
        ..RepairTurretState::default()
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct DroneCenterState {
    pub read_units: Vec<i32>,
    pub read_target: i32,
    pub units: Vec<i32>,
    pub target: Option<i32>,
    pub drone_progress: f32,
    pub drone_warmup: f32,
    pub total_drone_progress: f32,
}

impl Default for DroneCenterState {
    fn default() -> Self {
        Self {
            read_units: Vec::new(),
            read_target: -1,
            units: Vec::new(),
            target: None,
            drone_progress: 0.0,
            drone_warmup: 0.0,
            total_drone_progress: 0.0,
        }
    }
}

pub fn drone_center_update(
    state: &mut DroneCenterState,
    units_spawned: usize,
    efficiency: f32,
    edelta: f32,
    delta: f32,
    drone_construct_time: f32,
) -> bool {
    if !state.read_units.is_empty() {
        state.units = state.read_units.clone();
        state.read_units.clear();
    }
    state.drone_warmup = lerp_delta(
        state.drone_warmup,
        if state.units.len() < units_spawned {
            efficiency
        } else {
            0.0
        },
        0.1,
    );
    state.total_drone_progress += state.drone_warmup * delta;

    if state.units.len() < units_spawned {
        state.drone_progress += edelta / drone_construct_time;
        if state.drone_progress >= 1.0 {
            state.drone_progress = 0.0;
            return true;
        }
    }
    false
}

pub fn drone_center_apply_status(
    within_range: bool,
    drone_range: f32,
    target_hit_size: f32,
    distance: f32,
) -> bool {
    within_range || distance <= drone_range + target_hit_size
}

pub fn write_drone_center_state<W: Write>(
    write: &mut W,
    target_id: Option<i32>,
    unit_ids: &[i32],
) -> io::Result<()> {
    write_i32(write, target_id.unwrap_or(-1))?;
    write_i16(write, unit_ids.len() as i16)?;
    for id in unit_ids {
        write_i32(write, *id)?;
    }
    Ok(())
}

pub fn read_drone_center_state<R: Read>(read: &mut R) -> io::Result<DroneCenterState> {
    let target = read_i32(read)?;
    let count = read_i16(read)? as usize;
    let mut read_units = Vec::with_capacity(count);
    for _ in 0..count {
        read_units.push(read_i32(read)?);
    }
    Ok(DroneCenterState {
        read_target: target,
        read_units,
        ..DroneCenterState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitAssemblerModuleState {
    pub linked: bool,
    pub last_change: i32,
    pub has_payload: bool,
}

impl Default for UnitAssemblerModuleState {
    fn default() -> Self {
        Self {
            linked: false,
            last_change: -2,
            has_payload: false,
        }
    }
}

pub fn assembler_module_should_find_link(last_change: i32, world_tile_changes: i32) -> bool {
    last_change != world_tile_changes
}

pub fn assembler_module_can_place(has_link: bool) -> bool {
    has_link
}

pub fn assembler_module_accept_payload(
    linked: bool,
    has_payload: bool,
    linked_accepts_payload: bool,
) -> bool {
    linked && !has_payload && linked_accepts_payload
}

pub fn assembler_module_transfer_payload(
    moved_in_payload: bool,
    linked: bool,
    module_fits: bool,
    link_was_occupied: bool,
    link_accepts_payload: bool,
    efficiency: f32,
) -> bool {
    moved_in_payload
        && linked
        && module_fits
        && !link_was_occupied
        && link_accepts_payload
        && efficiency > 0.0
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn approach_delta(from: f32, to: f32, amount: f32) -> f32 {
    if from < to {
        (from + amount).min(to)
    } else {
        (from - amount).max(to)
    }
}

fn direction(rotation: i32) -> (i32, i32) {
    match rotation.rem_euclid(4) {
        0 => (1, 0),
        1 => (0, 1),
        2 => (-1, 0),
        _ => (0, -1),
    }
}

fn write_payload_seq<W: Write>(write: &mut W, seq: &PayloadSeq) -> io::Result<()> {
    write_i16(write, -(seq.len() as i16))?;
    for (key, amount) in seq.entries() {
        write.write_all(&[key.content_type.ordinal()])?;
        write_i16(write, key.id as i16)?;
        write_i32(write, amount)?;
    }
    Ok(())
}

fn read_payload_seq<R: Read>(read: &mut R) -> io::Result<PayloadSeq> {
    let count = read_i16(read)?;
    if count >= 0 {
        let mut seq = PayloadSeq::new();
        for _ in 0..count {
            let id = read_i16(read)? as ContentId;
            let amount = read_i32(read)?;
            seq.add(PayloadKey::new(ContentType::Block, id), amount);
        }
        return Ok(seq);
    }

    let mut seq = PayloadSeq::new();
    for _ in 0..(-count) {
        let mut ordinal = [0; 1];
        read.read_exact(&mut ordinal)?;
        let content_type = ContentType::from_ordinal(ordinal[0]).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown PayloadSeq content type ordinal {}", ordinal[0]),
            )
        })?;
        let id = read_i16(read)? as ContentId;
        let amount = read_i32(read)?;
        seq.add(PayloadKey::new(content_type, id), amount);
    }
    Ok(seq)
}

fn write_command<W: Write>(write: &mut W, command_id: Option<u8>) -> io::Result<()> {
    write.write_all(&[command_id.unwrap_or(255)])
}

fn read_command<R: Read>(read: &mut R) -> io::Result<Option<u8>> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok((buf[0] != 255).then_some(buf[0]))
}

fn write_vec_nullable<W: Write>(write: &mut W, value: Option<Vec2>) -> io::Result<()> {
    match value {
        Some(value) => write_vec2(write, value),
        None => {
            write.write_all(&f32::NAN.to_be_bytes())?;
            write.write_all(&f32::NAN.to_be_bytes())
        }
    }
}

fn read_vec_nullable<R: Read>(read: &mut R) -> io::Result<Option<Vec2>> {
    let vec = read_vec2(read)?;
    Ok((!vec.x.is_nan() && !vec.y.is_nan()).then_some(vec))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_block_spawn_and_dump_reset_payload_state_like_upstream() {
        let mut state = UnitBlockState {
            progress: 10.0,
            time: 3.0,
            speed_scl: 0.5,
            has_payload: true,
        };
        unit_block_spawned(&mut state);
        assert_eq!(state.progress, 0.0);
        assert!(!state.has_payload);

        state.has_payload = true;
        state.progress = 5.0;
        assert!(unit_block_dump_payload(true, &mut state));
        assert_eq!(state.progress, 0.0);
        assert!(!state.has_payload);
    }

    #[test]
    fn unit_factory_capacities_config_and_acceptance_follow_upstream() {
        let plans = vec![
            UnitPlan {
                unit_id: 1,
                time: 60.0,
                requirements: vec![
                    UnitPlanRequirement {
                        item_id: 0,
                        amount: 20,
                    },
                    UnitPlanRequirement {
                        item_id: 1,
                        amount: 5,
                    },
                ],
                banned: false,
            },
            UnitPlan {
                unit_id: 2,
                time: 90.0,
                requirements: vec![UnitPlanRequirement {
                    item_id: 0,
                    amount: 30,
                }],
                banned: false,
            },
        ];
        let (capacities, item_capacity) = unit_factory_capacities(3, &plans);
        assert_eq!(capacities, vec![60, 10, 0]);
        assert_eq!(item_capacity, 60);

        let mut state = UnitFactoryState {
            current_plan: 1,
            command_id: Some(7),
            ..UnitFactoryState::default()
        };
        unit_factory_configure_plan(&mut state, 9, plans.len(), false);
        assert_eq!(state.current_plan, -1);
        assert_eq!(state.base.progress, 0.0);
        assert_eq!(state.command_id, None);

        assert_eq!(unit_factory_fraction(-1, 10.0, 60.0), 0.0);
        assert_eq!(unit_factory_fraction(0, 30.0, 60.0), 0.5);
        assert!(unit_factory_should_consume(0, true, false, true));
        assert!(!unit_factory_should_consume(0, true, true, true));
        assert!(unit_factory_accept_item(0, 4, 5, true));
        assert!(!unit_factory_accept_item(0, 4, 5, false));
        assert_eq!(unit_factory_maximum_accepted(10, 1.5), 15);
    }

    #[test]
    fn unit_factory_update_and_serialization_match_revision_three_order() {
        let mut state = UnitFactoryState {
            current_plan: 0,
            command_pos: Some(Vec2::new(3.0, 4.0)),
            command_id: Some(2),
            ..UnitFactoryState::default()
        };
        state.base.progress = 58.5;
        state.base.speed_scl = 0.0;
        let update = unit_factory_update(&mut state, true, 1, 1.0, 1.0, 1.0, 60.0, false);
        assert!(!update.created_unit);
        assert_eq!(update.progress, 59.5);
        assert_eq!(update.speed_scl, 0.05);

        let update = unit_factory_update(&mut state, true, 1, 1.0, 1.0, 1.0, 60.0, false);
        assert!(update.created_unit);
        assert!(state.base.has_payload);
        assert!(state.base.progress < 1.0);

        let mut bytes = Vec::new();
        write_unit_factory_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_unit_factory_state(&mut bytes.as_slice(), 3).unwrap(),
            UnitFactoryState {
                base: UnitBlockState {
                    progress: state.base.progress,
                    ..UnitBlockState::default()
                },
                current_plan: state.current_plan,
                command_pos: state.command_pos,
                command_id: state.command_id,
            }
        );

        let legacy = [
            1.0f32.to_be_bytes().as_slice(),
            3i16.to_be_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            read_unit_factory_state(&mut legacy.as_slice(), 1).unwrap(),
            UnitFactoryState {
                base: UnitBlockState {
                    progress: 1.0,
                    ..UnitBlockState::default()
                },
                current_plan: 3,
                command_pos: None,
                command_id: None,
            }
        );
    }

    #[test]
    fn repair_tower_refresh_heal_warmup_and_consumption_follow_upstream() {
        let mut state = RepairTowerState {
            refresh: 5.5,
            warmup: 0.0,
            total_progress: 0.0,
        };
        let update = repair_tower_update(&mut state, 1.0, 0.75, false, 2, 1, 3.0, 1.0, 120.0);
        assert!(update.should_refresh_targets);
        assert!(update.should_consume);
        assert_eq!(update.heal_per_target, 2.25);
        assert_eq!(state.refresh, 0.0);
        assert_eq!(state.warmup, 0.06);
        assert!((state.total_progress - 1.0 / 120.0).abs() < 0.00001);

        let update = repair_tower_update(&mut state, 1.0, 1.0, true, 1, 1, 3.0, 1.0, 120.0);
        assert_eq!(state.warmup, 0.0);
        assert_eq!(update.heal_per_target, 0.0);
    }

    #[test]
    fn reconstructor_progress_acceptance_and_serialization_follow_upstream() {
        let (caps, item_cap) = reconstructor_capacities(
            3,
            &[
                UnitPlanRequirement {
                    item_id: 0,
                    amount: 5,
                },
                UnitPlanRequirement {
                    item_id: 1,
                    amount: 30,
                },
            ],
        );
        assert_eq!(caps, vec![10, 60, 0]);
        assert_eq!(item_cap, 60);
        assert_eq!(reconstructor_maximum_accepted(60, 1.5), 90);
        assert!(reconstructor_accept_item(89, 90));
        assert!(!reconstructor_accept_item(90, 90));
        assert!(!reconstructor_accept_item(0, 0));
        assert_eq!(reconstructor_fraction(30.0, 120.0), 0.25);
        assert!(reconstructor_accept_payload(
            true, true, true, true, true, false
        ));
        assert!(!reconstructor_accept_payload(
            true, true, false, true, true, false
        ));
        assert!(reconstructor_should_consume(true, true, true));

        let mut state = ReconstructorState {
            base: UnitBlockState {
                progress: 119.5,
                speed_scl: 0.0,
                ..UnitBlockState::default()
            },
            command_pos: Some(Vec2::new(2.0, 3.0)),
            command_id: Some(4),
            constructing: false,
        };
        assert!(reconstructor_update(
            &mut state, true, true, true, 1.0, 1.0, 1.0, 120.0
        ));
        assert!(state.base.progress < 1.0);
        assert_eq!(state.base.speed_scl, 0.05);

        let mut bytes = Vec::new();
        write_reconstructor_state(&mut bytes, &state).unwrap();
        let restored = read_reconstructor_state(&mut bytes.as_slice(), 3).unwrap();
        assert_eq!(restored.base.progress, state.base.progress);
        assert_eq!(restored.command_pos, state.command_pos);
        assert_eq!(restored.command_id, state.command_id);
    }

    #[test]
    fn assembler_geometry_tiers_acceptance_and_progress_follow_upstream() {
        let rect = unit_assembler_rect(80.0, 40.0, 1, 11, 5, 8.0);
        assert_eq!(
            rect,
            AssemblerRect {
                x: 80.0,
                y: 104.0,
                size: 88.0,
            }
        );
        assert_eq!(unit_assembler_current_tier(vec![0, 1, 3]), 1);
        assert_eq!(unit_assembler_current_tier(vec![2, 3]), 0);
        assert!(unit_assembler_accept_item(true, 9, 10, true));
        assert!(!unit_assembler_accept_item(false, 0, 10, true));
        assert!(unit_assembler_accept_payload(true, false, 4, 3, 1.0, false));
        assert!(!unit_assembler_accept_payload(
            true, false, 4, 4, 1.0, false
        ));
        assert!(unit_assembler_accept_payload(false, true, 4, 2, 1.0, true));

        let mut state = UnitAssemblerState::default();
        let drone_spawned = unit_assembler_update_progress(
            &mut state, true, 1.0, 0, 4, 0.5, true, true, 2, 1.0, 1.0, 1.0, 240.0, 10.0,
        );
        assert!(!drone_spawned);
        assert_eq!(state.warmup, 0.05);
        assert_eq!(state.progress, 0.05);
        unit_assembler_spawned(&mut state);
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn cargo_loader_and_unload_point_state_match_java_fields() {
        let mut loader = UnitCargoLoaderState::default();
        assert!(unit_cargo_loader_accept_item(199, 200));
        assert!(!unit_cargo_loader_update(
            &mut loader,
            1.0,
            true,
            60.0,
            60.0,
            480.0
        ));
        assert_eq!(loader.warmup, 1.0 / 60.0);
        assert_eq!(loader.build_progress, 0.125);
        unit_cargo_loader_spawned(&mut loader, 77, true);
        assert_eq!(loader.build_progress, 0.0);
        assert_eq!(loader.read_unit_id, 77);

        let mut bytes = Vec::new();
        write_unit_cargo_loader_state(&mut bytes, Some(77)).unwrap();
        assert_eq!(bytes, 77i32.to_be_bytes());
        assert_eq!(
            read_unit_cargo_loader_state(&mut bytes.as_slice())
                .unwrap()
                .read_unit_id,
            77
        );

        let mut unload = UnitCargoUnloadPointState {
            item_id: Some(5),
            stale_timer: 359.0,
            stale: false,
        };
        unit_cargo_unload_update(&mut unload, 10, 10, false, 1.0, 360.0);
        assert!(unload.stale);
        assert_eq!(unit_cargo_unload_accept_stack(10, 7, 5), 3);

        let mut bytes = Vec::new();
        write_unit_cargo_unload_state(&mut bytes, &unload).unwrap();
        assert_eq!(
            read_unit_cargo_unload_state(&mut bytes.as_slice())
                .unwrap()
                .item_id,
            Some(5)
        );
    }

    #[test]
    fn repair_turret_formulae_and_rotation_state_follow_upstream() {
        assert_eq!(repair_turret_multiplier(false, 2.0, 3.0, 0.5), 1.0);
        assert_eq!(repair_turret_multiplier(true, 2.0, 3.0, 0.5), 4.0);
        assert!(repair_turret_target_valid(
            true, false, 50.0, 10.0, 45.0, 9.0, 10.0
        ));
        assert!(!repair_turret_target_valid(
            true, false, 51.0, 10.0, 45.0, 9.0, 10.0
        ));
        assert_eq!(repair_turret_heal_amount(0.3, 0.5, 2.0, 4.0, 20.0), 1.2);
        assert_eq!(repair_turret_heal_amount(0.3, 0.5, 2.0, 4.0, 30.0), 0.0);
        assert_eq!(repair_turret_update_strength(0.0, true, 1.0), 0.08);
        assert!(repair_turret_should_consume(true, true));

        let state = RepairTurretState {
            rotation: 135.0,
            ..RepairTurretState::default()
        };
        let mut bytes = Vec::new();
        write_repair_turret_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_repair_turret_state(&mut bytes.as_slice(), 1).unwrap(),
            state
        );
        assert_eq!(
            read_repair_turret_state(&mut bytes.as_slice(), 0)
                .unwrap()
                .rotation,
            90.0
        );
    }

    #[test]
    fn drone_center_update_and_state_order_follow_upstream() {
        let mut state = DroneCenterState {
            read_units: vec![1, 2],
            read_target: -1,
            units: Vec::new(),
            target: None,
            drone_progress: 0.9,
            drone_warmup: 0.0,
            total_drone_progress: 0.0,
        };
        assert!(!drone_center_update(&mut state, 4, 1.0, 6.0, 1.0, 180.0));
        assert_eq!(state.units, vec![1, 2]);
        assert_eq!(state.drone_warmup, 0.1);
        assert!((state.drone_progress - 0.93333334).abs() < 0.00001);
        assert!(drone_center_apply_status(false, 50.0, 12.0, 62.0));

        let mut bytes = Vec::new();
        write_drone_center_state(&mut bytes, Some(9), &[1, 2, 3]).unwrap();
        let restored = read_drone_center_state(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.read_target, 9);
        assert_eq!(restored.read_units, vec![1, 2, 3]);
    }

    #[test]
    fn assembler_module_link_payload_rules_follow_upstream() {
        assert!(assembler_module_should_find_link(-2, 10));
        assert!(!assembler_module_should_find_link(10, 10));
        assert!(assembler_module_can_place(true));
        assert!(!assembler_module_can_place(false));
        assert!(assembler_module_accept_payload(true, false, true));
        assert!(!assembler_module_accept_payload(true, true, true));
        assert!(assembler_module_transfer_payload(
            true, true, true, false, true, 1.0
        ));
        assert!(!assembler_module_transfer_payload(
            true, true, true, true, true, 1.0
        ));
        assert!(!assembler_module_transfer_payload(
            true, true, true, false, true, 0.0
        ));
    }
}
