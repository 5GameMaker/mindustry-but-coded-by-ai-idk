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
}
