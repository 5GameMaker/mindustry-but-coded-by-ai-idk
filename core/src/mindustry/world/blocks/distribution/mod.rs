use std::io::{self, Read, Write};

use crate::mindustry::{ctype::ContentId, world::DirectionalItemBuffer};

pub mod chained_building;

pub use chained_building::ChainedBuilding;

pub fn positions_valid_line(x1: i32, y1: i32, x2: i32, y2: i32, range: i32) -> bool {
    if x1 == x2 {
        (y1 - y2).abs() <= range
    } else if y1 == y2 {
        (x1 - x2).abs() <= range
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideRoute {
    Left,
    Right,
}

pub fn sorter_should_direct(
    item: ContentId,
    sort_item: Option<ContentId>,
    invert: bool,
    enabled: bool,
) -> bool {
    ((Some(item) == sort_item) != invert) == enabled
}

pub fn sorter_rejects_instant_three_chain(
    direct: bool,
    source_instant: bool,
    target_instant: bool,
) -> bool {
    direct && source_instant && target_instant
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SorterState {
    pub sort_item: Option<ContentId>,
}

pub fn write_sorter_state<W: Write>(write: &mut W, state: &SorterState) -> io::Result<()> {
    write_i16(write, state.sort_item.unwrap_or(-1))
}

pub fn read_sorter_state<R: Read>(read: &mut R, revision: u8) -> io::Result<SorterState> {
    let item = read_i16(read)?;
    if revision == 1 {
        let mut legacy = DirectionalItemBuffer::new(20);
        legacy.read(read)?;
    }
    Ok(SorterState {
        sort_item: (item >= 0).then_some(item),
    })
}

pub fn read_overflow_gate_legacy_payload<R: Read>(read: &mut R, revision: u8) -> io::Result<()> {
    if revision == 1 {
        let mut legacy = DirectionalItemBuffer::new(25);
        legacy.read(read)?;
    } else if revision == 3 {
        let _legacy_rotation_bits = read_i32(read)?;
    }
    Ok(())
}

pub fn choose_side_route(
    left_accepts: bool,
    right_accepts: bool,
    rotation_bits: i32,
    dir: i32,
    flip: bool,
) -> Option<(SideRoute, i32)> {
    let bit = 1 << dir.rem_euclid(4);
    if left_accepts && !right_accepts {
        Some((SideRoute::Left, rotation_bits))
    } else if right_accepts && !left_accepts {
        Some((SideRoute::Right, rotation_bits))
    } else if !right_accepts {
        None
    } else {
        let route = if rotation_bits & bit == 0 {
            SideRoute::Left
        } else {
            SideRoute::Right
        };
        let next_bits = if flip {
            rotation_bits ^ bit
        } else {
            rotation_bits
        };
        Some((route, next_bits))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowRoute {
    Forward,
    Left,
    Right,
}

pub fn overflow_gate_route(
    can_forward: bool,
    invert_enabled: bool,
    left_accepts: bool,
    right_accepts: bool,
    rotation_bits: i32,
    from: i32,
    flip: bool,
) -> Option<(OverflowRoute, i32)> {
    if can_forward && !invert_enabled {
        return Some((OverflowRoute::Forward, rotation_bits));
    }

    let bit = 1 << from.rem_euclid(4);
    if !left_accepts && !right_accepts {
        if invert_enabled && can_forward {
            Some((OverflowRoute::Forward, rotation_bits))
        } else {
            None
        }
    } else if left_accepts && !right_accepts {
        Some((OverflowRoute::Left, rotation_bits))
    } else if right_accepts && !left_accepts {
        Some((OverflowRoute::Right, rotation_bits))
    } else {
        let route = if rotation_bits & bit == 0 {
            OverflowRoute::Left
        } else {
            OverflowRoute::Right
        };
        let next_bits = if flip {
            rotation_bits ^ bit
        } else {
            rotation_bits
        };
        Some((route, next_bits))
    }
}

pub fn junction_can_release(head_time: f32, now: f32, speed: f32, time_scale: f32) -> bool {
    now >= head_time + speed / time_scale || now < head_time
}

pub fn item_buffer_poll_ready(head_time: f32, now: f32, speed: f32, time_scale: f32) -> bool {
    now >= head_time + speed / time_scale || now < head_time
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemBridgeState {
    pub link: i32,
    pub warmup: f32,
    pub incoming: Vec<i32>,
    pub was_moved: bool,
    pub moved: bool,
}

impl Default for ItemBridgeState {
    fn default() -> Self {
        Self {
            link: -1,
            warmup: 0.0,
            incoming: Vec::new(),
            was_moved: false,
            moved: false,
        }
    }
}

pub fn item_bridge_should_consume(link_valid: bool, enabled: bool) -> bool {
    link_valid && enabled
}

pub fn item_bridge_update_moved_window(state: &mut ItemBridgeState) {
    state.was_moved = state.moved;
    state.moved = false;
}

pub fn item_bridge_warmup_step(warmup: f32, efficiency: f32) -> f32 {
    approach_delta(warmup, efficiency, 1.0 / 30.0)
}

pub fn item_bridge_time_speed_step(time_speed: f32, was_moved: bool) -> f32 {
    approach_delta(time_speed, if was_moved { 1.0 } else { 0.0 }, 1.0 / 60.0)
}

pub fn item_bridge_transport_iterations(
    transport_counter: f32,
    edelta: f32,
    transport_time: f32,
) -> (i32, f32) {
    if transport_time <= 0.0 {
        return (0, transport_counter + edelta);
    }
    let mut counter = transport_counter + edelta;
    let mut iterations = 0;
    while counter >= transport_time {
        iterations += 1;
        counter -= transport_time;
    }
    (iterations, counter)
}

pub fn write_item_bridge_state<W: Write>(write: &mut W, state: &ItemBridgeState) -> io::Result<()> {
    write_i32(write, state.link)?;
    write_f32(write, state.warmup)?;
    write_u8(write, state.incoming.len() as u8)?;
    for incoming in &state.incoming {
        write_i32(write, *incoming)?;
    }
    write_bool(write, state.was_moved || state.moved)
}

pub fn read_item_bridge_state<R: Read>(read: &mut R, revision: u8) -> io::Result<ItemBridgeState> {
    let link = read_i32(read)?;
    let warmup = read_f32(read)?;
    let links = read_u8(read)?;
    let mut incoming = Vec::with_capacity(links as usize);
    for _ in 0..links {
        incoming.push(read_i32(read)?);
    }
    let (was_moved, moved) = if revision >= 1 {
        let value = read_bool(read)?;
        (value, value)
    } else {
        (false, false)
    };
    Ok(ItemBridgeState {
        link,
        warmup,
        incoming,
        was_moved,
        moved,
    })
}

pub fn duct_progress_step(progress: f32, edelta: f32, speed: f32) -> f32 {
    progress + edelta / speed * 2.0
}

pub fn duct_ready_to_move(progress: f32, speed: f32) -> bool {
    progress >= 1.0 - 1.0 / speed
}

pub fn duct_accept_item(
    has_current: bool,
    items_empty: bool,
    source_relative_to_edge: Option<i32>,
    rotation: i32,
    armored: bool,
    source_is_duct: bool,
    source_front_points_to_target: bool,
) -> bool {
    if has_current || !items_empty {
        return false;
    }

    let Some(relative) = source_relative_to_edge else {
        return false;
    };
    let rotation = rotation.rem_euclid(4);

    if armored {
        (source_is_duct && source_front_points_to_target) || relative == rotation
    } else {
        relative != rotation || source_is_duct
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuctState {
    pub rec_dir: i32,
    pub current: Option<ContentId>,
}

impl Default for DuctState {
    fn default() -> Self {
        Self {
            rec_dir: -1,
            current: None,
        }
    }
}

pub fn write_duct_state<W: Write>(write: &mut W, state: &DuctState) -> io::Result<()> {
    write_i8(write, state.rec_dir as i8)
}

pub fn read_duct_state<R: Read>(
    read: &mut R,
    revision: u8,
    current: Option<ContentId>,
) -> io::Result<DuctState> {
    Ok(DuctState {
        rec_dir: if revision >= 1 {
            read_i8(read)? as i32
        } else {
            -1
        },
        current,
    })
}

pub fn duct_router_accept_item(
    has_current: bool,
    items_empty: bool,
    source_relative_to_edge: Option<i32>,
    rotation: i32,
) -> bool {
    !has_current && items_empty && source_relative_to_edge == Some((rotation + 2).rem_euclid(4))
}

pub fn duct_router_candidate_allowed(
    sort_item: Option<ContentId>,
    current: ContentId,
    neighbor_relative: i32,
    rotation: i32,
) -> bool {
    if neighbor_relative == (rotation + 2).rem_euclid(4) {
        return false;
    }
    sort_item
        .map(|sort| (sort == current) == (neighbor_relative == rotation.rem_euclid(4)))
        .unwrap_or(true)
}

pub fn overflow_duct_prefer_front(invert: bool) -> bool {
    !invert
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuctRouterState {
    pub sort_item: Option<ContentId>,
    pub current: Option<ContentId>,
}

pub fn write_duct_router_state<W: Write>(write: &mut W, state: &DuctRouterState) -> io::Result<()> {
    write_i16(write, state.sort_item.unwrap_or(-1))
}

pub fn read_duct_router_state<R: Read>(
    read: &mut R,
    revision: u8,
    current: Option<ContentId>,
) -> io::Result<DuctRouterState> {
    Ok(DuctRouterState {
        sort_item: if revision >= 1 {
            let id = read_i16(read)?;
            (id >= 0).then_some(id)
        } else {
            None
        },
        current,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct DuctJunctionState {
    pub buffer: DirectionalItemBuffer,
}

impl DuctJunctionState {
    pub const DEFAULT_CAPACITY: usize = 6;

    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: DirectionalItemBuffer::new(capacity),
        }
    }
}

impl Default for DuctJunctionState {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

pub fn duct_junction_accept_item(
    relative: Option<usize>,
    side_accepts: bool,
    target_exists_and_same_team: bool,
) -> bool {
    relative.is_some_and(|side| side < 4) && side_accepts && target_exists_and_same_team
}

pub fn duct_junction_ready(time: f32, edelta: f32, speed: f32) -> (f32, bool) {
    let next = duct_progress_step(time, edelta, speed);
    (next, duct_ready_to_move(next, speed))
}

pub fn write_duct_junction_state<W: Write>(
    write: &mut W,
    state: &DuctJunctionState,
) -> io::Result<()> {
    state.buffer.write(write)
}

pub fn read_duct_junction_state<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<DuctJunctionState> {
    let mut state = DuctJunctionState::default();
    state.buffer.read_with_legacy(read, revision == 0)?;
    Ok(state)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MassDriverStateKind {
    Idle = 0,
    Accepting = 1,
    Shooting = 2,
}

impl MassDriverStateKind {
    fn from_ordinal(value: u8) -> Self {
        match value {
            1 => Self::Accepting,
            2 => Self::Shooting,
            _ => Self::Idle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassDriverState {
    pub link: i32,
    pub rotation: f32,
    pub state: MassDriverStateKind,
}

pub fn mass_driver_link_valid(
    link_exists: bool,
    same_block: bool,
    same_team: bool,
    distance: f32,
    range: f32,
) -> bool {
    link_exists && same_block && same_team && distance <= range
}

pub fn mass_driver_time_to_arrive(distance: f32, bullet_speed: f32, bullet_lifetime: f32) -> f32 {
    (distance / bullet_speed).min(bullet_lifetime)
}

pub fn write_mass_driver_state<W: Write>(write: &mut W, state: &MassDriverState) -> io::Result<()> {
    write_i32(write, state.link)?;
    write_f32(write, state.rotation)?;
    write_u8(write, state.state as u8)
}

pub fn read_mass_driver_state<R: Read>(read: &mut R) -> io::Result<MassDriverState> {
    Ok(MassDriverState {
        link: read_i32(read)?,
        rotation: read_f32(read)?,
        state: MassDriverStateKind::from_ordinal(read_u8(read)?),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectionalUnloaderState {
    pub unload_item: Option<ContentId>,
    pub offset: i32,
}

pub fn directional_unloader_can_unload(
    front_exists: bool,
    back_exists: bool,
    same_team: bool,
    back_can_unload: bool,
    back_is_core_or_linked_core: bool,
    allow_core_unload: bool,
) -> bool {
    front_exists
        && back_exists
        && same_team
        && back_can_unload
        && (allow_core_unload || !back_is_core_or_linked_core)
}

pub fn directional_unloader_next_offset(item_id: i32) -> i32 {
    item_id + 1
}

pub fn write_directional_unloader_state<W: Write>(
    write: &mut W,
    state: &DirectionalUnloaderState,
) -> io::Result<()> {
    write_i16(write, state.unload_item.unwrap_or(-1))?;
    write_i16(write, state.offset as i16)
}

pub fn read_directional_unloader_state<R: Read>(
    read: &mut R,
) -> io::Result<DirectionalUnloaderState> {
    let item = read_i16(read)?;
    Ok(DirectionalUnloaderState {
        unload_item: (item >= 0).then_some(item),
        offset: read_i16(read)? as i32,
    })
}

pub const CONVEYOR_ITEM_SPACE: f32 = 0.4;
pub const CONVEYOR_CAPACITY: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConveyorItemState {
    pub item: ContentId,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConveyorState {
    pub items: Vec<ConveyorItemState>,
}

pub fn conveyor_accept_stack(min_item: f32, amount: i32) -> i32 {
    ((min_item / CONVEYOR_ITEM_SPACE) as i32).min(amount).max(0)
}

pub fn conveyor_accept_item(
    len: usize,
    min_item: f32,
    facing_relative_to_tile: Option<i32>,
    rotation: i32,
    source_rotate_and_is_next: bool,
) -> bool {
    if len >= CONVEYOR_CAPACITY || source_rotate_and_is_next {
        return false;
    }
    let Some(relative) = facing_relative_to_tile else {
        return false;
    };
    let direction = (relative - rotation).abs();
    ((direction == 0) && min_item >= CONVEYOR_ITEM_SPACE)
        || ((direction % 2 == 1) && min_item > 0.7)
}

pub fn conveyor_clog_heat(min_item: f32, blendbits: i32, current: f32) -> f32 {
    if min_item < CONVEYOR_ITEM_SPACE + if blendbits == 1 { 0.3 } else { 0.0 } {
        approach_delta(current, 1.0, 1.0 / 60.0)
    } else {
        0.0
    }
}

pub fn conveyor_next_max(aligned: bool, next_min_item: f32) -> f32 {
    if aligned {
        1.0 - (CONVEYOR_ITEM_SPACE - next_min_item).max(0.0)
    } else {
        1.0
    }
}

pub fn conveyor_encode_coord_x(x: f32) -> i8 {
    (x * 127.0) as i8
}

pub fn conveyor_encode_coord_y(y: f32) -> i8 {
    (y * 255.0 - 128.0) as i8
}

pub fn conveyor_decode_coord_x(x: i8) -> f32 {
    x as f32 / 127.0
}

pub fn conveyor_decode_coord_y(y: i8) -> f32 {
    (y as f32 + 128.0) / 255.0
}

pub fn write_conveyor_state<W: Write>(write: &mut W, state: &ConveyorState) -> io::Result<()> {
    write_i32(write, state.items.len() as i32)?;
    for item in &state.items {
        write_i16(write, item.item)?;
        write_i8(write, conveyor_encode_coord_x(item.x))?;
        write_i8(write, conveyor_encode_coord_y(item.y))?;
    }
    Ok(())
}

pub fn read_conveyor_state<R: Read>(read: &mut R, revision: u8) -> io::Result<ConveyorState> {
    let amount = read_i32(read)?;
    let mut items = Vec::new();
    for i in 0..amount {
        let (item, x, y) = if revision == 0 {
            let val = read_i32(read)?;
            (
                (((val >> 24) as i8) as i16 & 0xff) as i16,
                ((val >> 16) as i8) as f32 / 127.0,
                (((val >> 8) as i8) as f32 + 128.0) / 255.0,
            )
        } else {
            (
                read_i16(read)?,
                conveyor_decode_coord_x(read_i8(read)?),
                conveyor_decode_coord_y(read_i8(read)?),
            )
        };
        if i < CONVEYOR_CAPACITY as i32 {
            items.push(ConveyorItemState { item, x, y });
        }
    }
    Ok(ConveyorState { items })
}

#[derive(Debug, Clone, PartialEq)]
pub struct BufferedItemBridgeState {
    pub bridge: ItemBridgeState,
    pub index: i32,
    pub buffer: Vec<i64>,
}

pub fn buffered_bridge_can_accept(
    buffer_len: usize,
    buffer_capacity: usize,
    items_total: i32,
) -> bool {
    buffer_len < buffer_capacity && items_total > 0
}

pub fn buffered_bridge_delivers(
    timer_accept_ready: bool,
    polled_item: Option<ContentId>,
    target_accepts: bool,
) -> bool {
    timer_accept_ready && polled_item.is_some() && target_accepts
}

pub fn write_buffered_bridge_state<W: Write>(
    write: &mut W,
    state: &BufferedItemBridgeState,
) -> io::Result<()> {
    write_item_bridge_state(write, &state.bridge)?;
    write_i32(write, state.index)?;
    write_i32(write, state.buffer.len() as i32)?;
    for item in &state.buffer {
        write_i64(write, *item)?;
    }
    Ok(())
}

pub fn read_buffered_bridge_state<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<BufferedItemBridgeState> {
    let bridge = read_item_bridge_state(read, revision)?;
    let index = read_i32(read)?;
    let len = read_i32(read)?;
    let mut buffer = Vec::with_capacity(len as usize);
    for _ in 0..len {
        buffer.push(read_i64(read)?);
    }
    Ok(BufferedItemBridgeState {
        bridge,
        index,
        buffer,
    })
}

pub fn stack_router_accept_item(
    unloading: bool,
    current: Option<ContentId>,
    item: ContentId,
    items_total: i32,
    item_capacity: i32,
    source_relative_to_tile: Option<i32>,
    rotation: i32,
) -> bool {
    !unloading
        && (current.is_none() || current == Some(item))
        && items_total < item_capacity
        && source_relative_to_tile.map(|relative| relative.rem_euclid(4))
            == Some(rotation.rem_euclid(4))
}

pub fn stack_router_should_begin_unloading(progress: f32, speed: f32) -> bool {
    speed > 0.0 && progress >= speed
}

pub fn stack_router_progress_step(
    unloading: bool,
    current: Option<ContentId>,
    items_total: i32,
    item_capacity: i32,
    progress: f32,
    enabled: bool,
    efficiency: f32,
    base_efficiency: f32,
    speed: f32,
) -> (f32, bool) {
    if unloading || current.is_none() || items_total < item_capacity {
        return (progress, unloading);
    }

    let next_progress = progress
        + if enabled {
            efficiency + base_efficiency
        } else {
            0.0
        };
    let begin_unloading = stack_router_should_begin_unloading(next_progress, speed);
    if begin_unloading {
        (
            if speed > 0.0 {
                next_progress % speed
            } else {
                next_progress
            },
            true,
        )
    } else {
        (next_progress, false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackConveyorStateKind {
    Move,
    Load,
    Unload,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StackConveyorState {
    pub link: i32,
    pub cooldown: f32,
    pub last_item: Option<ContentId>,
}

pub fn stack_conveyor_accept_item(
    source_is_self: bool,
    items_total: i32,
    item_capacity: i32,
    items_empty_or_same: bool,
    cooldown: f32,
    recharge: f32,
    state: StackConveyorStateKind,
    maximum_accepted: i32,
    source_is_front: bool,
) -> bool {
    if source_is_self {
        return items_total < item_capacity && items_empty_or_same;
    }
    if cooldown > recharge - 1.0 {
        return false;
    }
    state == StackConveyorStateKind::Load
        && items_empty_or_same
        && items_total < maximum_accepted
        && !source_is_front
}

pub fn stack_conveyor_cooldown_step(
    cooldown: f32,
    speed: f32,
    efficiency: f32,
    delta: f32,
    recharge: f32,
) -> f32 {
    (cooldown - speed * efficiency * delta).clamp(0.0, recharge)
}

pub fn write_stack_conveyor_state<W: Write>(
    write: &mut W,
    state: &StackConveyorState,
) -> io::Result<()> {
    write_i32(write, state.link)?;
    write_f32(write, state.cooldown)
}

pub fn read_stack_conveyor_state<R: Read>(
    read: &mut R,
    last_item: Option<ContentId>,
) -> io::Result<StackConveyorState> {
    Ok(StackConveyorState {
        link: read_i32(read)?,
        cooldown: read_f32(read)?,
        last_item,
    })
}

fn approach_delta(from: f32, to: f32, amount: f32) -> f32 {
    if from < to {
        (from + amount).min(to)
    } else {
        (from - amount).max(to)
    }
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i64<R: Read>(read: &mut R) -> io::Result<i64> {
    let mut buf = [0; 8];
    read.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

fn write_i64<W: Write>(write: &mut W, value: i64) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i8<R: Read>(read: &mut R) -> io::Result<i8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0] as i8)
}

fn write_i8<W: Write>(write: &mut W, value: i8) -> io::Result<()> {
    write.write_all(&[value as u8])
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn write_u8<W: Write>(write: &mut W, value: u8) -> io::Result<()> {
    write.write_all(&[value])
}

fn read_bool<R: Read>(read: &mut R) -> io::Result<bool> {
    Ok(read_u8(read)? != 0)
}

fn write_bool<W: Write>(write: &mut W, value: bool) -> io::Result<()> {
    write_u8(write, u8::from(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_positions_and_item_bridge_runtime_helpers_follow_upstream() {
        assert!(positions_valid_line(0, 0, 0, 4, 4));
        assert!(positions_valid_line(0, 0, 3, 0, 4));
        assert!(!positions_valid_line(0, 0, 3, 1, 4));
        assert!(!positions_valid_line(0, 0, 0, 5, 4));

        assert!(item_bridge_should_consume(true, true));
        assert!(!item_bridge_should_consume(false, true));
        assert_eq!(item_bridge_warmup_step(0.0, 1.0), 1.0 / 30.0);
        assert_eq!(item_bridge_time_speed_step(0.0, true), 1.0 / 60.0);
        assert_eq!(item_bridge_transport_iterations(0.5, 2.0, 1.0), (2, 0.5));

        let mut state = ItemBridgeState {
            moved: true,
            ..Default::default()
        };
        item_bridge_update_moved_window(&mut state);
        assert!(state.was_moved);
        assert!(!state.moved);
    }

    #[test]
    fn sorter_and_overflow_routes_match_java_branching() {
        assert!(sorter_should_direct(1, Some(1), false, true));
        assert!(!sorter_should_direct(2, Some(1), false, true));
        assert!(!sorter_should_direct(1, Some(1), true, true));
        assert!(sorter_should_direct(1, Some(1), true, false));
        assert!(sorter_rejects_instant_three_chain(true, true, true));
        assert!(!sorter_rejects_instant_three_chain(false, true, true));

        assert_eq!(
            choose_side_route(true, false, 0, 2, true),
            Some((SideRoute::Left, 0))
        );
        assert_eq!(
            choose_side_route(false, true, 0, 2, true),
            Some((SideRoute::Right, 0))
        );
        assert_eq!(choose_side_route(false, false, 0, 2, true), None);
        assert_eq!(
            choose_side_route(true, true, 0, 2, true),
            Some((SideRoute::Left, 4))
        );
        assert_eq!(
            choose_side_route(true, true, 4, 2, false),
            Some((SideRoute::Right, 4))
        );

        assert_eq!(
            overflow_gate_route(true, false, false, false, 0, 1, true),
            Some((OverflowRoute::Forward, 0))
        );
        assert_eq!(
            overflow_gate_route(true, true, false, false, 0, 1, true),
            Some((OverflowRoute::Forward, 0))
        );
        assert_eq!(
            overflow_gate_route(false, false, true, true, 0, 1, true),
            Some((OverflowRoute::Left, 2))
        );
        assert_eq!(
            overflow_gate_route(false, false, false, false, 0, 1, true),
            None
        );
    }

    #[test]
    fn junction_and_item_buffer_release_timing_matches_directional_buffer_edges() {
        assert!(!junction_can_release(100.0, 110.0, 26.0, 1.0));
        assert!(junction_can_release(100.0, 126.0, 26.0, 1.0));
        assert!(junction_can_release(100.0, 90.0, 26.0, 1.0));
        assert!(item_buffer_poll_ready(100.0, 113.0, 26.0, 2.0));
    }

    #[test]
    fn item_bridge_state_serialization_matches_java_order() {
        let state = ItemBridgeState {
            link: 0x01020304,
            warmup: 0.75,
            incoming: vec![11, 12],
            was_moved: true,
            moved: false,
        };
        let mut bytes = Vec::new();
        write_item_bridge_state(&mut bytes, &state).unwrap();
        assert_eq!(&bytes[0..4], &[1, 2, 3, 4]);
        assert_eq!(bytes[8], 2);
        assert_eq!(*bytes.last().unwrap(), 1);

        let restored = read_item_bridge_state(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(
            restored,
            ItemBridgeState {
                moved: true,
                ..state
            }
        );
    }

    #[test]
    fn duct_acceptance_progress_and_router_filters_follow_upstream() {
        assert_eq!(duct_progress_step(0.0, 1.0, 4.0), 0.5);
        assert!(!duct_ready_to_move(0.7, 4.0));
        assert!(duct_ready_to_move(0.75, 4.0));

        assert!(duct_accept_item(
            false,
            true,
            Some(1),
            0,
            false,
            false,
            false
        ));
        assert!(!duct_accept_item(
            false,
            true,
            Some(0),
            0,
            false,
            false,
            false
        ));
        assert!(duct_accept_item(
            false,
            true,
            Some(0),
            0,
            true,
            false,
            false
        ));
        assert!(!duct_accept_item(
            false,
            true,
            Some(1),
            0,
            true,
            false,
            false
        ));
        assert!(duct_accept_item(false, true, Some(1), 0, true, true, true));
        assert!(!duct_accept_item(
            true,
            true,
            Some(1),
            0,
            false,
            false,
            false
        ));

        assert!(duct_router_accept_item(false, true, Some(2), 0));
        assert!(!duct_router_accept_item(false, true, Some(1), 0));
        assert!(duct_router_candidate_allowed(Some(5), 5, 0, 0));
        assert!(!duct_router_candidate_allowed(Some(5), 4, 0, 0));
        assert!(duct_router_candidate_allowed(Some(5), 4, 1, 0));
        assert!(!duct_router_candidate_allowed(None, 4, 2, 0));
        assert!(overflow_duct_prefer_front(false));
        assert!(!overflow_duct_prefer_front(true));

        let state = DuctState {
            rec_dir: 3,
            current: Some(7),
        };
        let mut bytes = Vec::new();
        write_duct_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_duct_state(&mut bytes.as_slice(), 1, Some(7)).unwrap(),
            state
        );

        let router = DuctRouterState {
            sort_item: Some(9),
            current: Some(1),
        };
        let mut bytes = Vec::new();
        write_duct_router_state(&mut bytes, &router).unwrap();
        assert_eq!(
            read_duct_router_state(&mut bytes.as_slice(), 1, Some(1)).unwrap(),
            router
        );
    }

    #[test]
    fn duct_junction_state_and_timing_follow_java_four_side_order() {
        assert!(duct_junction_accept_item(Some(2), true, true));
        assert!(!duct_junction_accept_item(Some(2), false, true));
        assert!(!duct_junction_accept_item(Some(4), true, true));
        let (time, ready) = duct_junction_ready(-1.0, 1.0, 4.0);
        assert_eq!(time, -0.5);
        assert!(!ready);
        let (_, ready) = duct_junction_ready(0.5, 1.0, 4.0);
        assert!(ready);

        let mut state = DuctJunctionState::default();
        assert_eq!(state.buffer.capacity(), DuctJunctionState::DEFAULT_CAPACITY);
        assert!(state.buffer.accept(0, 1, 0.1));
        assert!(state.buffer.accept(0, 2, 0.2));
        assert!(state.buffer.accept(2, 3, 0.3));
        assert!(state.buffer.accept(3, 4, 0.4));
        let mut bytes = Vec::new();
        write_duct_junction_state(&mut bytes, &state).unwrap();
        assert_eq!(
            bytes.len(),
            4 * (2 + DuctJunctionState::DEFAULT_CAPACITY * 8)
        );
        assert_eq!(
            read_duct_junction_state(&mut bytes.as_slice(), 1).unwrap(),
            state
        );
    }

    #[test]
    fn mass_driver_and_directional_unloader_serialization_follow_upstream() {
        assert!(mass_driver_link_valid(true, true, true, 99.0, 100.0));
        assert!(!mass_driver_link_valid(true, true, false, 99.0, 100.0));
        assert_eq!(mass_driver_time_to_arrive(100.0, 10.0, 20.0), 10.0);
        assert_eq!(mass_driver_time_to_arrive(100.0, 2.0, 20.0), 20.0);

        let state = MassDriverState {
            link: 17,
            rotation: 90.0,
            state: MassDriverStateKind::Shooting,
        };
        let mut bytes = Vec::new();
        write_mass_driver_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_mass_driver_state(&mut bytes.as_slice()).unwrap(),
            state
        );

        assert!(directional_unloader_can_unload(
            true, true, true, true, false, false
        ));
        assert!(!directional_unloader_can_unload(
            true, true, true, true, true, false
        ));
        assert!(directional_unloader_can_unload(
            true, true, true, true, true, true
        ));
        assert_eq!(directional_unloader_next_offset(5), 6);

        let unload = DirectionalUnloaderState {
            unload_item: Some(12),
            offset: 13,
        };
        let mut bytes = Vec::new();
        write_directional_unloader_state(&mut bytes, &unload).unwrap();
        assert_eq!(
            read_directional_unloader_state(&mut bytes.as_slice()).unwrap(),
            unload
        );
    }

    #[test]
    fn conveyor_acceptance_clog_and_serialization_follow_java_layout() {
        assert_eq!(conveyor_accept_stack(0.8, 5), 2);
        assert!(conveyor_accept_item(0, 0.4, Some(0), 0, false));
        assert!(!conveyor_accept_item(3, 1.0, Some(0), 0, false));
        assert!(conveyor_accept_item(0, 0.71, Some(1), 0, false));
        assert!(!conveyor_accept_item(0, 0.7, Some(1), 0, false));
        assert!(!conveyor_accept_item(0, 1.0, Some(0), 0, true));
        assert_eq!(conveyor_clog_heat(0.1, 0, 0.0), 1.0 / 60.0);
        assert_eq!(conveyor_clog_heat(0.8, 1, 0.5), 0.0);
        assert_eq!(conveyor_next_max(true, 0.2), 0.8);
        assert_eq!(conveyor_next_max(false, 0.0), 1.0);

        let state = ConveyorState {
            items: vec![
                ConveyorItemState {
                    item: 1,
                    x: 0.0,
                    y: 0.5,
                },
                ConveyorItemState {
                    item: 2,
                    x: -0.5,
                    y: 1.0,
                },
            ],
        };
        let mut bytes = Vec::new();
        write_conveyor_state(&mut bytes, &state).unwrap();
        assert_eq!(&bytes[0..4], &[0, 0, 0, 2]);
        let restored = read_conveyor_state(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(restored.items.len(), 2);
        assert_eq!(restored.items[0].item, 1);
        assert_eq!(
            restored.items[0].y,
            conveyor_decode_coord_y(conveyor_encode_coord_y(0.5))
        );
    }

    #[test]
    fn buffered_bridge_and_stack_conveyor_shells_follow_upstream_state() {
        assert!(buffered_bridge_can_accept(0, 10, 1));
        assert!(!buffered_bridge_can_accept(10, 10, 1));
        assert!(!buffered_bridge_can_accept(0, 10, 0));
        assert!(buffered_bridge_delivers(true, Some(1), true));
        assert!(!buffered_bridge_delivers(true, None, true));

        let bridge = ItemBridgeState {
            link: 5,
            warmup: 0.25,
            incoming: vec![1],
            was_moved: true,
            moved: false,
        };
        let state = BufferedItemBridgeState {
            bridge,
            index: 2,
            buffer: vec![0x0102030405060708],
        };
        let mut bytes = Vec::new();
        write_buffered_bridge_state(&mut bytes, &state).unwrap();
        let restored = read_buffered_bridge_state(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(restored.buffer, state.buffer);
        assert_eq!(restored.index, 2);

        assert!(stack_conveyor_accept_item(
            true,
            1,
            10,
            true,
            100.0,
            120.0,
            StackConveyorStateKind::Unload,
            4,
            true
        ));
        assert!(!stack_conveyor_accept_item(
            false,
            0,
            10,
            true,
            119.5,
            120.0,
            StackConveyorStateKind::Load,
            4,
            false
        ));
        assert!(stack_conveyor_accept_item(
            false,
            3,
            10,
            true,
            0.0,
            120.0,
            StackConveyorStateKind::Load,
            4,
            false
        ));
        assert_eq!(
            stack_conveyor_cooldown_step(10.0, 2.0, 1.0, 3.0, 120.0),
            4.0
        );

        let stack = StackConveyorState {
            link: 123,
            cooldown: 4.5,
            last_item: Some(9),
        };
        let mut bytes = Vec::new();
        write_stack_conveyor_state(&mut bytes, &stack).unwrap();
        assert_eq!(
            read_stack_conveyor_state(&mut bytes.as_slice(), Some(9)).unwrap(),
            stack
        );
    }

    #[test]
    fn stack_router_accept_item_and_progress_helpers_follow_java_branching() {
        assert!(stack_router_accept_item(false, None, 7, 0, 10, Some(2), 2));
        assert!(stack_router_accept_item(
            false,
            Some(7),
            7,
            9,
            10,
            Some(2),
            2
        ));
        assert!(!stack_router_accept_item(
            true,
            Some(7),
            7,
            9,
            10,
            Some(2),
            2
        ));
        assert!(!stack_router_accept_item(
            false,
            Some(7),
            8,
            9,
            10,
            Some(2),
            2
        ));
        assert!(!stack_router_accept_item(
            false,
            Some(7),
            7,
            10,
            10,
            Some(2),
            2
        ));
        assert!(!stack_router_accept_item(
            false,
            Some(7),
            7,
            9,
            10,
            Some(1),
            2
        ));

        assert!(!stack_router_should_begin_unloading(2.99, 3.0));
        assert!(stack_router_should_begin_unloading(3.0, 3.0));

        assert_eq!(
            stack_router_progress_step(false, Some(7), 10, 10, 2.5, true, 1.0, 0.5, 3.0),
            (1.0, true)
        );
        assert_eq!(
            stack_router_progress_step(false, Some(7), 10, 10, 2.5, false, 1.0, 0.5, 3.0),
            (2.5, false)
        );
        assert_eq!(
            stack_router_progress_step(false, None, 10, 10, 2.5, true, 1.0, 0.5, 3.0),
            (2.5, false)
        );
        assert_eq!(
            stack_router_progress_step(true, Some(7), 10, 10, 2.5, true, 1.0, 0.5, 3.0),
            (2.5, true)
        );
    }
}
