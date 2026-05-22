//! Pure input-handler helpers mirroring selected upstream `mindustry.input.InputHandler` paths.
//!
//! This module intentionally keeps UI, event-bus and live networking side effects
//! outside. Callers provide validation predicates and receive explicit plans
//! such as tile-config rollback packets.

use crate::mindustry::entities::comp::building::{
    BuildingComp, BuildingConfigChange, BuildingConfigRollback,
};
use crate::mindustry::io::{BuildingRef, EntityRef, TypeValue};
use crate::mindustry::net::{RotateBlockCallPacket, TileConfigCallPacket, TileTapCallPacket};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileConfigRejectReason {
    MissingBuild,
    CannotInteract,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileConfigRollbackPlan {
    pub connection_id: i32,
    pub packet: TileConfigCallPacket,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TileConfigContext {
    pub connection_id: Option<i32>,
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub last_accessed: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileConfigOutcome {
    pub accepted: bool,
    pub rejection: Option<TileConfigRejectReason>,
    pub change: Option<BuildingConfigChange>,
    pub rollback: Option<TileConfigRollbackPlan>,
    /// Mirrors Java remote validation behavior: rejected non-local players raise
    /// `ValidateException`; local clients just return after optional rollback.
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateBlockRejectReason {
    MissingBuild,
    CannotInteract,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RotateBlockContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub last_accessed: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotateBlockOutcome {
    pub accepted: bool,
    pub rejection: Option<RotateBlockRejectReason>,
    pub previous_rotation: Option<i32>,
    pub current_rotation: Option<i32>,
    pub packet: Option<RotateBlockCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileTapContext {
    pub player: Option<EntityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileTapOutcome {
    pub accepted: bool,
    pub packet: Option<TileTapCallPacket>,
}

impl RotateBlockOutcome {
    fn rejected(context: RotateBlockContext, reason: RotateBlockRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            previous_rotation: None,
            current_rotation: None,
            packet: None,
            should_raise_validate: !context.local_player,
        }
    }
}

impl TileConfigOutcome {
    fn rejected(
        context: TileConfigContext,
        reason: TileConfigRejectReason,
        build: Option<&BuildingComp>,
    ) -> Self {
        let rollback = match (context.connection_id, build) {
            (Some(connection_id), Some(build)) => Some(TileConfigRollbackPlan {
                connection_id,
                packet: TileConfigCallPacket::rollback_for(
                    context.player.unwrap_or_else(EntityRef::null),
                    BuildingConfigRollback {
                        tile_pos: build.tile_pos,
                        value: build.config_value(),
                    },
                ),
            }),
            _ => None,
        };

        Self {
            accepted: false,
            rejection: Some(reason),
            change: None,
            rollback,
            should_raise_validate: !context.local_player,
        }
    }
}

pub fn tile_config<F, A>(
    context: TileConfigContext,
    build: Option<&mut BuildingComp>,
    value: TypeValue,
    can_interact: F,
    admin_allows: A,
) -> TileConfigOutcome
where
    F: FnOnce(&BuildingComp) -> bool,
    A: FnOnce(&BuildingComp, &TypeValue) -> bool,
{
    let Some(build) = build else {
        return TileConfigOutcome::rejected(context, TileConfigRejectReason::MissingBuild, None);
    };

    if !can_interact(build) {
        return TileConfigOutcome::rejected(
            context,
            TileConfigRejectReason::CannotInteract,
            Some(build),
        );
    }

    if !admin_allows(build, &value) {
        return TileConfigOutcome::rejected(
            context,
            TileConfigRejectReason::AdminDenied,
            Some(build),
        );
    }

    let change =
        build.configure_any_checked_accessed(value, |_| true, context.last_accessed.clone());

    TileConfigOutcome {
        accepted: true,
        rejection: None,
        change: Some(change),
        rollback: None,
        should_raise_validate: false,
    }
}

pub fn client_tile_config_packet(build: &BuildingComp, value: TypeValue) -> TileConfigCallPacket {
    TileConfigCallPacket::client(BuildingRef::new(build.tile_pos), value)
}

pub fn rotate_block<F, A>(
    context: RotateBlockContext,
    build: Option<&mut BuildingComp>,
    direction: bool,
    can_interact: F,
    admin_allows: A,
) -> RotateBlockOutcome
where
    F: FnOnce(&BuildingComp) -> bool,
    A: FnOnce(&BuildingComp, i32) -> bool,
{
    let Some(build) = build else {
        return RotateBlockOutcome::rejected(context, RotateBlockRejectReason::MissingBuild);
    };

    let delta = if direction { 1 } else { -1 };
    let previous_rotation = build.rotation;
    let next_rotation = (build.rotation + delta).rem_euclid(4);

    if !can_interact(build) {
        return RotateBlockOutcome::rejected(context, RotateBlockRejectReason::CannotInteract);
    }

    if !admin_allows(build, next_rotation) {
        return RotateBlockOutcome::rejected(context, RotateBlockRejectReason::AdminDenied);
    }

    build.set_rotation(next_rotation);
    if let Some(last_accessed) = context.last_accessed {
        build.last_accessed = last_accessed;
    }

    RotateBlockOutcome {
        accepted: true,
        rejection: None,
        previous_rotation: Some(previous_rotation),
        current_rotation: Some(build.rotation),
        packet: Some(RotateBlockCallPacket::server(
            context.player.unwrap_or_else(EntityRef::null),
            BuildingRef::new(build.tile_pos),
            direction,
        )),
        should_raise_validate: false,
    }
}

pub fn client_rotate_block_packet(build: &BuildingComp, direction: bool) -> RotateBlockCallPacket {
    RotateBlockCallPacket::client(BuildingRef::new(build.tile_pos), direction)
}

pub fn tile_tap(context: TileTapContext, tile: Option<i32>) -> TileTapOutcome {
    let Some(tile) = tile else {
        return TileTapOutcome {
            accepted: false,
            packet: None,
        };
    };

    TileTapOutcome {
        accepted: true,
        packet: Some(TileTapCallPacket::server(
            context.player.unwrap_or_else(EntityRef::null),
            Some(tile),
        )),
    }
}

pub fn client_tile_tap_packet(tile: Option<i32>) -> Option<TileTapCallPacket> {
    tile.map(|tile| TileTapCallPacket::client(Some(tile)))
}

#[cfg(test)]
mod tests {
    use crate::mindustry::io::TeamId;
    use crate::mindustry::world::block::Block;
    use crate::mindustry::world::point2_pack;

    use super::*;

    fn block() -> Block {
        let mut block = Block::new(5, "router");
        block.health = 100;
        block
    }

    #[test]
    fn tile_config_accepts_after_validation_and_records_last_access() {
        let mut building = BuildingComp::new(point2_pack(2, 3), block(), TeamId(1));
        let outcome = tile_config(
            TileConfigContext {
                connection_id: Some(9),
                player: Some(EntityRef::new(7)),
                local_player: false,
                last_accessed: Some("[#ffaa00]frog".into()),
            },
            Some(&mut building),
            TypeValue::String("next".into()),
            |_| true,
            |_, value| matches!(value, TypeValue::String(_)),
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.rejection, None);
        assert!(outcome.rollback.is_none());
        assert!(!outcome.should_raise_validate);
        assert_eq!(building.config_value(), TypeValue::String("next".into()));
        assert_eq!(building.last_accessed, "[#ffaa00]frog");
        assert_eq!(
            outcome.change.unwrap().current,
            TypeValue::String("next".into())
        );
    }

    #[test]
    fn tile_config_rejects_remote_and_plans_authoritative_rollback() {
        let mut building = BuildingComp::new(point2_pack(4, 5), block(), TeamId(2));
        building.set_config_value(TypeValue::String("old".into()));

        let outcome = tile_config(
            TileConfigContext {
                connection_id: Some(17),
                player: Some(EntityRef::new(33)),
                local_player: false,
                last_accessed: None,
            },
            Some(&mut building),
            TypeValue::Int(9),
            |_| true,
            |_, value| matches!(value, TypeValue::String(_)),
        );

        assert!(!outcome.accepted);
        assert_eq!(outcome.rejection, Some(TileConfigRejectReason::AdminDenied));
        assert!(outcome.should_raise_validate);
        assert_eq!(building.config_value(), TypeValue::String("old".into()));

        let rollback = outcome.rollback.unwrap();
        assert_eq!(rollback.connection_id, 17);
        assert_eq!(rollback.packet.player, EntityRef::new(33));
        assert_eq!(rollback.packet.build, BuildingRef::new(point2_pack(4, 5)));
        assert_eq!(rollback.packet.value, TypeValue::String("old".into()));
    }

    #[test]
    fn tile_config_rejects_local_without_validate_exception() {
        let mut building = BuildingComp::new(point2_pack(6, 7), block(), TeamId(3));

        let outcome = tile_config(
            TileConfigContext {
                connection_id: None,
                player: Some(EntityRef::new(1)),
                local_player: true,
                last_accessed: None,
            },
            Some(&mut building),
            TypeValue::String("blocked".into()),
            |_| false,
            |_, _| true,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(TileConfigRejectReason::CannotInteract)
        );
        assert!(!outcome.should_raise_validate);
        assert!(outcome.rollback.is_none());
        assert_eq!(building.config_value(), TypeValue::Null);
    }

    #[test]
    fn client_tile_config_packet_uses_client_payload_shape() {
        let building = BuildingComp::new(point2_pack(8, 9), block(), TeamId(1));
        let packet = client_tile_config_packet(&building, TypeValue::String("cfg".into()));

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(packet.value, TypeValue::String("cfg".into()));
    }

    #[test]
    fn rotate_block_applies_direction_after_validation() {
        let mut building = BuildingComp::new(point2_pack(2, 2), block(), TeamId(1));
        building.set_rotation(3);

        let outcome = rotate_block(
            RotateBlockContext {
                player: Some(EntityRef::new(7)),
                local_player: false,
                last_accessed: Some("[#ffaa00]frog".into()),
            },
            Some(&mut building),
            true,
            |_| true,
            |_, next_rotation| next_rotation == 0,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.previous_rotation, Some(3));
        assert_eq!(outcome.current_rotation, Some(0));
        assert_eq!(building.rotation, 0);
        assert_eq!(building.last_accessed, "[#ffaa00]frog");
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(7));
        assert_eq!(packet.build, BuildingRef::new(point2_pack(2, 2)));
        assert!(packet.direction);
    }

    #[test]
    fn rotate_block_rejects_remote_without_mutating_rotation() {
        let mut building = BuildingComp::new(point2_pack(3, 3), block(), TeamId(1));
        building.set_rotation(1);

        let outcome = rotate_block(
            RotateBlockContext {
                player: Some(EntityRef::new(8)),
                local_player: false,
                last_accessed: None,
            },
            Some(&mut building),
            false,
            |_| true,
            |_, _| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(RotateBlockRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert_eq!(building.rotation, 1);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn client_rotate_block_packet_uses_client_payload_shape() {
        let building = BuildingComp::new(point2_pack(4, 4), block(), TeamId(1));
        let packet = client_rotate_block_packet(&building, false);

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.build, BuildingRef::new(point2_pack(4, 4)));
        assert!(!packet.direction);
    }

    #[test]
    fn tile_tap_ignores_null_tile_and_keeps_player_for_event_packet() {
        let ignored = tile_tap(
            TileTapContext {
                player: Some(EntityRef::new(5)),
            },
            None,
        );
        assert!(!ignored.accepted);
        assert!(ignored.packet.is_none());

        let tile = point2_pack(1, 2);
        let outcome = tile_tap(
            TileTapContext {
                player: Some(EntityRef::new(5)),
            },
            Some(tile),
        );

        assert!(outcome.accepted);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(5));
        assert_eq!(packet.tile, Some(tile));
    }

    #[test]
    fn client_tile_tap_packet_uses_client_payload_shape() {
        let tile = point2_pack(3, 4);
        let packet = client_tile_tap_packet(Some(tile)).unwrap();

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.tile, Some(tile));
        assert!(client_tile_tap_packet(None).is_none());
    }
}
