use std::io::{self, Read, Write};

use crate::mindustry::ctype::ContentId;

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
) -> bool {
    !has_current
        && items_empty
        && source_relative_to_edge
            .map(|relative| armored || relative != rotation.rem_euclid(4) || source_is_duct)
            .unwrap_or(false)
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
    write_i32(write, state.rec_dir)
}

pub fn read_duct_state<R: Read>(
    read: &mut R,
    revision: u8,
    current: Option<ContentId>,
) -> io::Result<DuctState> {
    Ok(DuctState {
        rec_dir: if revision >= 1 { read_i32(read)? } else { -1 },
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
    pub times: [f32; 4],
    pub item_data: [Option<ContentId>; 4],
}

impl Default for DuctJunctionState {
    fn default() -> Self {
        Self {
            times: [0.0; 4],
            item_data: [None; 4],
        }
    }
}

pub fn duct_junction_accept_item(
    relative: Option<usize>,
    side_empty: bool,
    target_exists_and_same_team: bool,
) -> bool {
    relative.is_some_and(|side| side < 4) && side_empty && target_exists_and_same_team
}

pub fn duct_junction_ready(time: f32, edelta: f32, speed: f32) -> (f32, bool) {
    let next = duct_progress_step(time, edelta, speed);
    (next, duct_ready_to_move(next, speed))
}

pub fn write_duct_junction_state<W: Write>(
    write: &mut W,
    state: &DuctJunctionState,
) -> io::Result<()> {
    for i in 0..4 {
        write_f32(write, state.times[i])?;
        write_i16(write, state.item_data[i].unwrap_or(-1))?;
    }
    Ok(())
}

pub fn read_duct_junction_state<R: Read>(read: &mut R) -> io::Result<DuctJunctionState> {
    let mut state = DuctJunctionState::default();
    for i in 0..4 {
        state.times[i] = read_f32(read)?;
        let item = read_i16(read)?;
        state.item_data[i] = (item >= 0).then_some(item);
    }
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
    write_i32(write, state.offset)
}

pub fn read_directional_unloader_state<R: Read>(
    read: &mut R,
) -> io::Result<DirectionalUnloaderState> {
    let item = read_i16(read)?;
    Ok(DirectionalUnloaderState {
        unload_item: (item >= 0).then_some(item),
        offset: read_i32(read)?,
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

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
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

        assert!(duct_accept_item(false, true, Some(1), 0, false, false));
        assert!(!duct_accept_item(false, true, Some(0), 0, false, false));
        assert!(duct_accept_item(false, true, Some(0), 0, true, false));
        assert!(!duct_accept_item(true, true, Some(1), 0, false, false));

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

        let state = DuctJunctionState {
            times: [0.1, 0.2, 0.3, 0.4],
            item_data: [Some(1), None, Some(3), Some(4)],
        };
        let mut bytes = Vec::new();
        write_duct_junction_state(&mut bytes, &state).unwrap();
        assert_eq!(bytes.len(), 24);
        assert_eq!(
            read_duct_junction_state(&mut bytes.as_slice()).unwrap(),
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
}
