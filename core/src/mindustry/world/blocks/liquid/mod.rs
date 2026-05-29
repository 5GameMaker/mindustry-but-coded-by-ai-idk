use std::io::{self, Read, Write};

use crate::mindustry::ctype::ContentId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidAmount {
    pub liquid: ContentId,
    pub amount: f32,
}

pub fn accept_same_or_low_amount(
    current_liquid: Option<ContentId>,
    incoming_liquid: ContentId,
    current_amount: f32,
) -> bool {
    current_liquid == Some(incoming_liquid) || current_amount < 0.2
}

pub fn conduit_accept_liquid(
    current_liquid: Option<ContentId>,
    incoming_liquid: ContentId,
    current_amount: f32,
    source_is_self: bool,
    source_relative_to_this: Option<i32>,
    rotation: i32,
) -> bool {
    accept_same_or_low_amount(current_liquid, incoming_liquid, current_amount)
        && (source_is_self
            || source_relative_to_this
                .map(|relative| (relative + 2).rem_euclid(4) != rotation.rem_euclid(4))
                .unwrap_or(true))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidSourceKind {
    Conduit,
    DirectionLiquidBridge,
    LiquidJunction,
    Other,
}

pub fn armored_conduit_accept_liquid(
    base_accept: bool,
    source_kind: LiquidSourceKind,
    absolute_relative_to_this: Option<i32>,
    rotation: i32,
    source_proximity_contains_this: bool,
) -> bool {
    base_accept
        && (matches!(
            source_kind,
            LiquidSourceKind::Conduit
                | LiquidSourceKind::DirectionLiquidBridge
                | LiquidSourceKind::LiquidJunction
        ) || absolute_relative_to_this == Some(rotation.rem_euclid(4))
            || !source_proximity_contains_this)
}

pub fn calc_dump_transfer(
    source_amount: f32,
    source_capacity: f32,
    target_amount: f32,
    target_capacity: f32,
    scaling: f32,
) -> f32 {
    if source_capacity <= 0.0 || target_capacity <= 0.0 || scaling <= 0.0 {
        return 0.0;
    }
    let source_frac = source_amount / source_capacity;
    let target_frac = target_amount / target_capacity;
    if target_frac >= source_frac {
        return 0.0;
    }
    ((source_frac - target_frac) * source_capacity / scaling)
        .min(source_amount)
        .min((target_capacity - target_amount).max(0.0))
        .max(0.0)
}

pub fn calc_move_flow(
    source_amount: f32,
    source_capacity: f32,
    target_amount: f32,
    target_capacity: f32,
    liquid_pressure: f32,
    target_accepts: bool,
) -> f32 {
    if source_capacity <= 0.0 || target_capacity <= 0.0 || !target_accepts {
        return 0.0;
    }

    let ofract = target_amount / target_capacity;
    let fract = source_amount / source_capacity * liquid_pressure;
    if ofract > fract {
        return 0.0;
    }

    ((fract - ofract).clamp(0.0, 1.0) * source_capacity)
        .min(source_amount)
        .min((target_capacity - target_amount).max(0.0))
        .max(0.0)
}

pub fn calc_leak_amount(source_amount: f32) -> f32 {
    source_amount / 1.5
}

pub fn liquid_bridge_transport_success(warmup: f32, moved_amount: f32) -> bool {
    warmup >= 0.25 && moved_amount > 0.05
}

pub fn liquid_router_should_dump(current_amount: f32) -> bool {
    current_amount > 0.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiquidJunctionNode {
    pub enabled: bool,
    pub next: Option<usize>,
    pub next_accepts: bool,
    pub next_is_junction: bool,
}

pub fn choose_liquid_destination(nodes: &[LiquidJunctionNode], start: usize) -> usize {
    let mut current = start;
    let mut guard = 0;
    while let Some(node) = nodes.get(current) {
        if !node.enabled {
            return current;
        }
        let Some(next) = node.next else {
            return current;
        };
        if !node.next_accepts && !node.next_is_junction {
            return current;
        }
        if next >= nodes.len() || guard > nodes.len() {
            return current;
        }
        current = next;
        guard += 1;
    }
    start
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TiledFrameCrop {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub draw: bool,
}

pub fn calc_tiled_frame_crop(size: f32, padding: f32) -> TiledFrameCrop {
    let squish = padding * 2.0;
    let draw = squish < size;
    TiledFrameCrop {
        x: padding,
        y: padding,
        width: (size - squish).max(0.0),
        height: (size - squish).max(0.0),
        draw,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidBridgeState {
    pub link: i32,
    pub warmup: f32,
    pub incoming: Vec<i32>,
    pub was_moved: bool,
    pub moved: bool,
}

impl Default for LiquidBridgeState {
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

pub fn liquid_bridge_should_consume(link_valid: bool, enabled: bool) -> bool {
    link_valid && enabled
}

pub fn liquid_bridge_update_moved_window(state: &mut LiquidBridgeState) {
    state.was_moved = state.moved;
    state.moved = false;
}

pub fn liquid_bridge_mark_transport(state: &mut LiquidBridgeState, warmup: f32, moved_amount: f32) {
    if liquid_bridge_transport_success(warmup, moved_amount) {
        state.moved = true;
    }
}

pub fn write_liquid_bridge_state<W: Write>(
    write: &mut W,
    state: &LiquidBridgeState,
) -> io::Result<()> {
    write_i32(write, state.link)?;
    write_f32(write, state.warmup)?;
    write_u8(write, state.incoming.len() as u8)?;
    for incoming in &state.incoming {
        write_i32(write, *incoming)?;
    }
    write_bool(write, state.was_moved || state.moved)
}

pub fn read_liquid_bridge_state<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<LiquidBridgeState> {
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
    Ok(LiquidBridgeState {
        link,
        warmup,
        incoming,
        was_moved,
        moved,
    })
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
    fn liquid_acceptance_matches_router_and_conduit_thresholds() {
        assert!(accept_same_or_low_amount(Some(1), 1, 10.0));
        assert!(accept_same_or_low_amount(Some(1), 2, 0.19));
        assert!(!accept_same_or_low_amount(Some(1), 2, 0.2));

        assert!(conduit_accept_liquid(Some(1), 1, 5.0, false, Some(1), 0));
        assert!(!conduit_accept_liquid(Some(1), 1, 5.0, false, Some(2), 0));
        assert!(conduit_accept_liquid(Some(1), 1, 5.0, true, Some(2), 0));

        assert!(armored_conduit_accept_liquid(
            true,
            LiquidSourceKind::Conduit,
            None,
            0,
            true
        ));
        assert!(armored_conduit_accept_liquid(
            true,
            LiquidSourceKind::Other,
            Some(1),
            1,
            true
        ));
        assert!(!armored_conduit_accept_liquid(
            true,
            LiquidSourceKind::Other,
            Some(2),
            1,
            true
        ));
    }

    #[test]
    fn liquid_transfer_formulas_clamp_like_building_helpers() {
        assert_eq!(calc_dump_transfer(10.0, 10.0, 5.0, 10.0, 2.0), 2.5);
        assert_eq!(calc_dump_transfer(5.0, 10.0, 8.0, 10.0, 2.0), 0.0);
        assert_eq!(calc_dump_transfer(10.0, 10.0, 9.5, 10.0, 1.0), 0.5);

        assert_eq!(calc_move_flow(10.0, 10.0, 0.0, 10.0, 1.0, true), 10.0);
        assert_eq!(calc_move_flow(10.0, 10.0, 9.0, 10.0, 1.0, true), 1.0);
        assert_eq!(calc_move_flow(1.0, 10.0, 9.0, 10.0, 1.0, true), 0.0);
        assert_eq!(calc_move_flow(10.0, 10.0, 0.0, 10.0, 1.0, false), 0.0);

        assert_eq!(calc_leak_amount(9.0), 6.0);
    }

    #[test]
    fn bridge_junction_and_tiled_frame_helpers_follow_liquid_blocks() {
        assert!(!liquid_bridge_transport_success(0.24, 1.0));
        assert!(!liquid_bridge_transport_success(0.25, 0.05));
        assert!(liquid_bridge_transport_success(0.25, 0.051));
        assert!(liquid_router_should_dump(0.1));
        assert!(!liquid_router_should_dump(0.0));

        let nodes = [
            LiquidJunctionNode {
                enabled: true,
                next: Some(1),
                next_accepts: false,
                next_is_junction: true,
            },
            LiquidJunctionNode {
                enabled: true,
                next: Some(2),
                next_accepts: true,
                next_is_junction: false,
            },
            LiquidJunctionNode {
                enabled: false,
                next: None,
                next_accepts: false,
                next_is_junction: false,
            },
        ];
        assert_eq!(choose_liquid_destination(&nodes, 0), 2);

        let blocked_terminal = [
            LiquidJunctionNode {
                enabled: true,
                next: Some(1),
                next_accepts: true,
                next_is_junction: true,
            },
            LiquidJunctionNode {
                enabled: true,
                next: Some(2),
                next_accepts: false,
                next_is_junction: false,
            },
            LiquidJunctionNode {
                enabled: true,
                next: None,
                next_accepts: false,
                next_is_junction: false,
            },
        ];
        assert_eq!(choose_liquid_destination(&blocked_terminal, 0), 1);

        let cycle = [
            LiquidJunctionNode {
                enabled: true,
                next: Some(1),
                next_accepts: true,
                next_is_junction: true,
            },
            LiquidJunctionNode {
                enabled: true,
                next: Some(0),
                next_accepts: true,
                next_is_junction: true,
            },
        ];
        assert_eq!(choose_liquid_destination(&cycle, 0), 1);

        let crop = calc_tiled_frame_crop(8.0, 1.0);
        assert_eq!(crop.width, 6.0);
        assert!(crop.draw);
        assert!(!calc_tiled_frame_crop(8.0, 4.0).draw);
    }

    #[test]
    fn liquid_bridge_inherits_item_bridge_state_order_and_revision_flag() {
        assert!(liquid_bridge_should_consume(true, true));
        assert!(!liquid_bridge_should_consume(false, true));

        let mut state = LiquidBridgeState {
            link: 0x01020304,
            warmup: 0.5,
            incoming: vec![7, 9],
            was_moved: false,
            moved: true,
        };
        liquid_bridge_update_moved_window(&mut state);
        assert!(state.was_moved);
        assert!(!state.moved);
        liquid_bridge_mark_transport(&mut state, 0.25, 0.051);
        assert!(state.moved);

        let mut bytes = Vec::new();
        write_liquid_bridge_state(&mut bytes, &state).unwrap();
        assert_eq!(&bytes[0..4], &[1, 2, 3, 4]);
        assert_eq!(bytes[8], 2);
        assert_eq!(*bytes.last().unwrap(), 1);

        let restored = read_liquid_bridge_state(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(restored.link, state.link);
        assert_eq!(restored.warmup, state.warmup);
        assert_eq!(restored.incoming, state.incoming);
        assert!(restored.was_moved);
        assert!(restored.moved);

        let mut old_bytes = Vec::new();
        write_i32(&mut old_bytes, 5).unwrap();
        write_f32(&mut old_bytes, 0.25).unwrap();
        write_u8(&mut old_bytes, 0).unwrap();
        let old = read_liquid_bridge_state(&mut old_bytes.as_slice(), 0).unwrap();
        assert_eq!(old.link, 5);
        assert_eq!(old.warmup, 0.25);
        assert!(!old.was_moved);
        assert!(!old.moved);
    }
}
