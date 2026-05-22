//! Pure input-handler helpers mirroring selected upstream `mindustry.input.InputHandler` paths.
//!
//! This module intentionally keeps UI, event-bus and live networking side effects
//! outside. Callers provide validation predicates and receive explicit plans
//! such as tile-config rollback packets.

use crate::mindustry::entities::comp::{
    building::{BuildingConfigChange, BuildingConfigRollback},
    BuildingComp, PayloadState, PlayerComp, UnitComp,
};
use crate::mindustry::io::{BuildingRef, EntityRef, TypeValue, UnitRef};
use crate::mindustry::net::{
    PickedBuildPayloadCallPacket, RequestBuildPayloadCallPacket, RequestItemCallPacket,
    RotateBlockCallPacket, TakeItemsCallPacket, TileConfigCallPacket, TileTapCallPacket,
    TransferInventoryCallPacket, TransferItemToCallPacket,
};
use crate::mindustry::world::meta::BuildVisibility;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestItemRejectReason {
    MissingPlayer,
    MissingBuild,
    MissingUnit,
    MissingItem,
    NotInteractable,
    OutOfRange,
    PlayerDead,
    NonPositiveAmount,
    CannotInteract,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RequestItemContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub within_range: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestItemOutcome {
    pub accepted: bool,
    pub rejection: Option<RequestItemRejectReason>,
    pub requested_amount: i32,
    pub accepted_amount: i32,
    pub packet: Option<TakeItemsCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferInventoryRejectReason {
    MissingPlayer,
    MissingBuild,
    MissingUnit,
    MissingItem,
    OutOfRange,
    MissingItemStorage,
    PlayerDead,
    DepositBlocked,
    EmptyStack,
    CannotInteract,
    DepositRateLimited,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransferInventoryContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub within_range: bool,
    pub deposit_rate_allowed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferInventoryOutcome {
    pub accepted: bool,
    pub rejection: Option<TransferInventoryRejectReason>,
    pub stack_amount: i32,
    pub accepted_amount: i32,
    pub packet: Option<TransferItemToCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPayloadPickupKind {
    StoredPayload,
    WholeBuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestBuildPayloadRejectReason {
    MissingPlayer,
    MissingBuild,
    MissingUnit,
    MissingPayloadComponent,
    OutOfRange,
    AdminDenied,
    CannotInteract,
    NoPickupTarget,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RequestBuildPayloadContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub within_range: bool,
    pub teams_can_interact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestBuildPayloadOutcome {
    pub accepted: bool,
    pub rejection: Option<RequestBuildPayloadRejectReason>,
    pub pickup: Option<BuildPayloadPickupKind>,
    pub packet: Option<PickedBuildPayloadCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickedBuildPayloadRejectReason {
    MissingUnit,
    MissingBuild,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedBuildPayloadOutcome {
    pub accepted: bool,
    pub rejection: Option<PickedBuildPayloadRejectReason>,
    pub packet: Option<PickedBuildPayloadCallPacket>,
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

impl RequestItemOutcome {
    fn rejected(
        context: &RequestItemContext,
        reason: RequestItemRejectReason,
        requested_amount: i32,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            requested_amount,
            accepted_amount: 0,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl TransferInventoryOutcome {
    fn rejected(
        context: &TransferInventoryContext,
        reason: TransferInventoryRejectReason,
        stack_amount: i32,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            stack_amount,
            accepted_amount: 0,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl RequestBuildPayloadOutcome {
    fn rejected(
        context: &RequestBuildPayloadContext,
        reason: RequestBuildPayloadRejectReason,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            pickup: None,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl PickedBuildPayloadOutcome {
    fn rejected(reason: PickedBuildPayloadRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            packet: None,
        }
    }
}

fn player_unit_ref(player: &PlayerComp, unit: &UnitComp) -> UnitRef {
    player.unit_ref().unwrap_or(UnitRef::Unit { id: unit.id() })
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

pub fn request_item<I, C, A>(
    context: RequestItemContext,
    player: Option<&PlayerComp>,
    unit: Option<&UnitComp>,
    build: Option<&BuildingComp>,
    item: Option<String>,
    amount: i32,
    build_interactable: I,
    can_interact: C,
    admin_allows: A,
) -> RequestItemOutcome
where
    I: FnOnce(&BuildingComp, &PlayerComp) -> bool,
    C: FnOnce(&PlayerComp, &BuildingComp) -> bool,
    A: FnOnce(&BuildingComp, &str, i32) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::MissingPlayer,
            amount,
            false,
        );
    }
    let player = player.unwrap();

    let Some(build) = build else {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::MissingBuild,
            amount,
            false,
        );
    };

    if !build_interactable(build, player) {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::NotInteractable,
            amount,
            false,
        );
    }

    if !context.within_range {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::OutOfRange,
            amount,
            false,
        );
    }

    if player.dead() {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::PlayerDead,
            amount,
            false,
        );
    }

    if amount <= 0 {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::NonPositiveAmount,
            amount,
            false,
        );
    }

    let Some(unit) = unit else {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::MissingUnit,
            amount,
            false,
        );
    };

    let Some(item) = item else {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::MissingItem,
            amount,
            false,
        );
    };

    if !can_interact(player, build) {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::CannotInteract,
            amount,
            true,
        );
    }

    if !admin_allows(build, &item, amount) {
        return RequestItemOutcome::rejected(
            &context,
            RequestItemRejectReason::AdminDenied,
            amount,
            true,
        );
    }

    let accepted_amount = unit.items.max_accepted(&item).min(amount).max(0);
    RequestItemOutcome {
        accepted: true,
        rejection: None,
        requested_amount: amount,
        accepted_amount,
        packet: Some(TakeItemsCallPacket {
            build: BuildingRef::new(build.tile_pos),
            item: Some(item),
            amount: accepted_amount,
            to: player_unit_ref(player, unit),
        }),
        should_raise_validate: false,
    }
}

pub fn client_request_item_packet(
    build: &BuildingComp,
    item: Option<String>,
    amount: i32,
) -> RequestItemCallPacket {
    RequestItemCallPacket {
        player: EntityRef::null(),
        build: BuildingRef::new(build.tile_pos),
        item,
        amount,
    }
}

pub fn transfer_inventory<D, I, A, S>(
    context: TransferInventoryContext,
    player: Option<&PlayerComp>,
    unit: Option<&UnitComp>,
    build: Option<&BuildingComp>,
    allow_deposit: D,
    can_interact: I,
    admin_allows: A,
    accept_stack: S,
) -> TransferInventoryOutcome
where
    D: FnOnce(&BuildingComp) -> bool,
    I: FnOnce(&PlayerComp, &BuildingComp) -> bool,
    A: FnOnce(&PlayerComp, &BuildingComp, &str, i32) -> bool,
    S: FnOnce(&BuildingComp, &UnitComp, &str, i32) -> i32,
{
    if context.player.is_none() || player.is_none() {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::MissingPlayer,
            0,
            false,
        );
    }
    let player = player.unwrap();

    let Some(build) = build else {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::MissingBuild,
            0,
            false,
        );
    };

    if !context.within_range {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::OutOfRange,
            0,
            false,
        );
    }

    if build.items.is_none() {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::MissingItemStorage,
            0,
            false,
        );
    }

    if player.dead() {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::PlayerDead,
            0,
            false,
        );
    }

    if !allow_deposit(build) {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::DepositBlocked,
            0,
            false,
        );
    }

    let Some(unit) = unit else {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::MissingUnit,
            0,
            false,
        );
    };

    let stack_amount = unit.items.stack.amount;
    if stack_amount <= 0 {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::EmptyStack,
            stack_amount,
            true,
        );
    }

    let Some(item) = unit.items.item().map(str::to_owned) else {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::MissingItem,
            stack_amount,
            false,
        );
    };

    if !can_interact(player, build) {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::CannotInteract,
            stack_amount,
            true,
        );
    }

    if !context.deposit_rate_allowed {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::DepositRateLimited,
            stack_amount,
            true,
        );
    }

    if !admin_allows(player, build, &item, stack_amount) {
        return TransferInventoryOutcome::rejected(
            &context,
            TransferInventoryRejectReason::AdminDenied,
            stack_amount,
            true,
        );
    }

    let accepted_amount = accept_stack(build, unit, &item, stack_amount)
        .clamp(0, stack_amount)
        .max(0);
    TransferInventoryOutcome {
        accepted: true,
        rejection: None,
        stack_amount,
        accepted_amount,
        packet: Some(TransferItemToCallPacket {
            unit: player_unit_ref(player, unit),
            item: Some(item),
            amount: accepted_amount,
            x: unit.x(),
            y: unit.y(),
            build: BuildingRef::new(build.tile_pos),
        }),
        should_raise_validate: false,
    }
}

pub fn client_transfer_inventory_packet(build: &BuildingComp) -> TransferInventoryCallPacket {
    TransferInventoryCallPacket {
        player: EntityRef::null(),
        build: BuildingRef::new(build.tile_pos),
    }
}

pub fn request_build_payload<A>(
    context: RequestBuildPayloadContext,
    player: Option<&PlayerComp>,
    unit: Option<&UnitComp>,
    build: Option<&BuildingComp>,
    stored_payload: Option<&PayloadState>,
    build_can_pickup: bool,
    admin_allows: A,
) -> RequestBuildPayloadOutcome
where
    A: FnOnce(&PlayerComp, &BuildingComp, &UnitComp) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return RequestBuildPayloadOutcome::rejected(
            &context,
            RequestBuildPayloadRejectReason::MissingPlayer,
            false,
        );
    }
    let player = player.unwrap();

    let Some(unit) = unit else {
        return RequestBuildPayloadOutcome::rejected(
            &context,
            RequestBuildPayloadRejectReason::MissingUnit,
            false,
        );
    };

    let Some(build) = build else {
        return RequestBuildPayloadOutcome::rejected(
            &context,
            RequestBuildPayloadRejectReason::MissingBuild,
            false,
        );
    };

    let Some(payload) = unit.payload.as_ref() else {
        return RequestBuildPayloadOutcome::rejected(
            &context,
            RequestBuildPayloadRejectReason::MissingPayloadComponent,
            false,
        );
    };

    if !context.within_range {
        return RequestBuildPayloadOutcome::rejected(
            &context,
            RequestBuildPayloadRejectReason::OutOfRange,
            false,
        );
    }

    if !admin_allows(player, build, unit) {
        return RequestBuildPayloadOutcome::rejected(
            &context,
            RequestBuildPayloadRejectReason::AdminDenied,
            true,
        );
    }

    if !context.teams_can_interact {
        return RequestBuildPayloadOutcome::rejected(
            &context,
            RequestBuildPayloadRejectReason::CannotInteract,
            false,
        );
    }

    let unit_ref = player_unit_ref(player, unit);
    if let Some(current) = stored_payload {
        if payload.can_pickup_payload(current) {
            return RequestBuildPayloadOutcome {
                accepted: true,
                rejection: None,
                pickup: Some(BuildPayloadPickupKind::StoredPayload),
                packet: Some(PickedBuildPayloadCallPacket {
                    unit: unit_ref,
                    build_pos: Some(build.tile_pos),
                    on_ground: false,
                }),
                should_raise_validate: false,
            };
        }
    }

    if build.block.build_visibility != BuildVisibility::Hidden
        && payload.can_pickup_build(build.block.size as f32, build.team, build_can_pickup)
    {
        return RequestBuildPayloadOutcome {
            accepted: true,
            rejection: None,
            pickup: Some(BuildPayloadPickupKind::WholeBuild),
            packet: Some(PickedBuildPayloadCallPacket {
                unit: unit_ref,
                build_pos: Some(build.tile_pos),
                on_ground: true,
            }),
            should_raise_validate: false,
        };
    }

    RequestBuildPayloadOutcome::rejected(
        &context,
        RequestBuildPayloadRejectReason::NoPickupTarget,
        false,
    )
}

pub fn client_request_build_payload_packet(build: &BuildingComp) -> RequestBuildPayloadCallPacket {
    RequestBuildPayloadCallPacket {
        player: EntityRef::null(),
        build: BuildingRef::new(build.tile_pos),
    }
}

pub fn picked_build_payload(
    unit: Option<UnitRef>,
    build: Option<&BuildingComp>,
    on_ground: bool,
) -> PickedBuildPayloadOutcome {
    let Some(unit) = unit else {
        return PickedBuildPayloadOutcome::rejected(PickedBuildPayloadRejectReason::MissingUnit);
    };

    let Some(build) = build else {
        return PickedBuildPayloadOutcome::rejected(PickedBuildPayloadRejectReason::MissingBuild);
    };

    PickedBuildPayloadOutcome {
        accepted: true,
        rejection: None,
        packet: Some(PickedBuildPayloadCallPacket {
            unit,
            build_pos: Some(build.tile_pos),
            on_ground,
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::mindustry::entities::comp::{PayloadComp, PayloadKind, PlayerUnitState};
    use crate::mindustry::io::TeamId;
    use crate::mindustry::r#type::UnitType;
    use crate::mindustry::world::block::Block;
    use crate::mindustry::world::point2_pack;

    use super::*;

    fn block() -> Block {
        let mut block = Block::new(5, "router");
        block.health = 100;
        block
    }

    fn item_block() -> Block {
        let mut block = block();
        block.has_items = true;
        block.item_capacity = 30;
        block
    }

    fn unit_type(item_capacity: i32) -> UnitType {
        let mut unit_type = UnitType::new(1, "alpha");
        unit_type.item_capacity = item_capacity;
        unit_type
    }

    fn payload_unit(id: i32, team: TeamId, capacity: f32) -> UnitComp {
        let mut unit_type = unit_type(10);
        unit_type.payload_capacity = capacity;
        let mut unit = UnitComp::new(id, unit_type, team);
        unit.payload = Some(PayloadComp::new(team, capacity));
        unit
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

    #[test]
    fn request_item_accepts_after_validation_and_plans_take_items() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(44).with_valid(true));
        let unit = UnitComp::new(44, unit_type(10), TeamId(1));
        let building = BuildingComp::new(point2_pack(8, 9), item_block(), TeamId(1));

        let outcome = request_item(
            RequestItemContext {
                player: Some(EntityRef::new(7)),
                local_player: false,
                within_range: true,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            Some("copper".into()),
            15,
            |build, player| build.team == player.team,
            |player, build| player.team == build.team,
            |_, item, amount| item == "copper" && amount == 15,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.requested_amount, 15);
        assert_eq!(outcome.accepted_amount, 10);
        assert!(!outcome.should_raise_validate);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!(packet.amount, 10);
        assert_eq!(packet.to, UnitRef::Unit { id: 44 });
    }

    #[test]
    fn request_item_rejects_admin_denied_as_validate_error() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(44).with_valid(true));
        let unit = UnitComp::new(44, unit_type(10), TeamId(1));
        let building = BuildingComp::new(point2_pack(8, 9), item_block(), TeamId(1));

        let outcome = request_item(
            RequestItemContext {
                player: Some(EntityRef::new(7)),
                local_player: false,
                within_range: true,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            Some("lead".into()),
            3,
            |_, _| true,
            |_, _| true,
            |_, _, _| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(RequestItemRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn request_item_rejects_nonpositive_amount_without_validate_error() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(44).with_valid(true));
        let unit = UnitComp::new(44, unit_type(10), TeamId(1));
        let building = BuildingComp::new(point2_pack(8, 9), item_block(), TeamId(1));

        let outcome = request_item(
            RequestItemContext {
                player: Some(EntityRef::new(7)),
                local_player: false,
                within_range: true,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            Some("copper".into()),
            0,
            |_, _| true,
            |_, _| true,
            |_, _, _| true,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(RequestItemRejectReason::NonPositiveAmount)
        );
        assert!(!outcome.should_raise_validate);
    }

    #[test]
    fn client_request_item_packet_uses_client_payload_shape() {
        let building = BuildingComp::new(point2_pack(10, 11), item_block(), TeamId(1));
        let packet = client_request_item_packet(&building, Some("copper".into()), 4);

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.build, BuildingRef::new(point2_pack(10, 11)));
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!(packet.amount, 4);
    }

    #[test]
    fn transfer_inventory_accepts_after_validation_and_plans_transfer_item() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(45).with_valid(true));
        let mut unit = UnitComp::new(45, unit_type(10), TeamId(1));
        unit.set_pos(12.0, 24.0);
        unit.items.add_item_amount("copper", 7);
        let building = BuildingComp::new(point2_pack(12, 13), item_block(), TeamId(1));

        let outcome = transfer_inventory(
            TransferInventoryContext {
                player: Some(EntityRef::new(8)),
                local_player: false,
                within_range: true,
                deposit_rate_allowed: true,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            |_| true,
            |player, build| player.team == build.team,
            |_, _, item, amount| item == "copper" && amount == 7,
            |_, _, _, amount| amount - 2,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.stack_amount, 7);
        assert_eq!(outcome.accepted_amount, 5);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 45 });
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!(packet.amount, 5);
        assert_eq!((packet.x, packet.y), (12.0, 24.0));
        assert_eq!(packet.build, BuildingRef::new(point2_pack(12, 13)));
    }

    #[test]
    fn transfer_inventory_rejects_rate_limited_as_validate_error() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(45).with_valid(true));
        let mut unit = UnitComp::new(45, unit_type(10), TeamId(1));
        unit.items.add_item_amount("lead", 2);
        let building = BuildingComp::new(point2_pack(12, 13), item_block(), TeamId(1));

        let outcome = transfer_inventory(
            TransferInventoryContext {
                player: Some(EntityRef::new(8)),
                local_player: false,
                within_range: true,
                deposit_rate_allowed: false,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            |_| true,
            |_, _| true,
            |_, _, _, _| true,
            |_, _, _, amount| amount,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(TransferInventoryRejectReason::DepositRateLimited)
        );
        assert!(outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn client_transfer_inventory_packet_uses_client_payload_shape() {
        let building = BuildingComp::new(point2_pack(14, 15), item_block(), TeamId(1));
        let packet = client_transfer_inventory_packet(&building);

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.build, BuildingRef::new(point2_pack(14, 15)));
    }

    #[test]
    fn request_build_payload_prefers_stored_payload_when_pickable() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(46).with_valid(true));
        let unit = payload_unit(46, TeamId(1), 256.0);
        let building = BuildingComp::new(point2_pack(16, 17), item_block(), TeamId(1));
        let stored = PayloadState {
            kind: PayloadKind::Build,
            size: 2.0,
        };

        let outcome = request_build_payload(
            RequestBuildPayloadContext {
                player: Some(EntityRef::new(9)),
                local_player: false,
                within_range: true,
                teams_can_interact: true,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            Some(&stored),
            true,
            |player, build, unit| player.team == build.team && unit.team_id() == build.team,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.pickup, Some(BuildPayloadPickupKind::StoredPayload));
        assert!(!outcome.should_raise_validate);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 46 });
        assert_eq!(packet.build_pos, Some(point2_pack(16, 17)));
        assert!(!packet.on_ground);
    }

    #[test]
    fn request_build_payload_picks_whole_build_when_no_stored_payload() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(47).with_valid(true));
        let unit = payload_unit(47, TeamId(1), 1024.0);
        let mut block = item_block();
        block.size = 2;
        let building = BuildingComp::new(point2_pack(18, 19), block, TeamId(1));

        let outcome = request_build_payload(
            RequestBuildPayloadContext {
                player: Some(EntityRef::new(10)),
                local_player: false,
                within_range: true,
                teams_can_interact: true,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            None,
            true,
            |_, _, _| true,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.pickup, Some(BuildPayloadPickupKind::WholeBuild));
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 47 });
        assert_eq!(packet.build_pos, Some(point2_pack(18, 19)));
        assert!(packet.on_ground);
    }

    #[test]
    fn request_build_payload_rejects_out_of_range_without_packet() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(48).with_valid(true));
        let unit = payload_unit(48, TeamId(1), 1024.0);
        let building = BuildingComp::new(point2_pack(20, 21), item_block(), TeamId(1));

        let outcome = request_build_payload(
            RequestBuildPayloadContext {
                player: Some(EntityRef::new(11)),
                local_player: false,
                within_range: false,
                teams_can_interact: true,
            },
            Some(&player),
            Some(&unit),
            Some(&building),
            None,
            true,
            |_, _, _| true,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(RequestBuildPayloadRejectReason::OutOfRange)
        );
        assert!(!outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn picked_build_payload_records_ground_and_payload_variants() {
        let building = BuildingComp::new(point2_pack(22, 23), item_block(), TeamId(1));
        let ground = picked_build_payload(Some(UnitRef::Unit { id: 49 }), Some(&building), true);
        let stored = picked_build_payload(Some(UnitRef::Unit { id: 49 }), Some(&building), false);

        assert!(ground.accepted);
        assert!(ground.packet.as_ref().unwrap().on_ground);
        assert!(stored.accepted);
        assert!(!stored.packet.as_ref().unwrap().on_ground);
        assert_eq!(
            stored.packet.as_ref().unwrap().build_pos,
            Some(point2_pack(22, 23))
        );
    }

    #[test]
    fn picked_build_payload_rejects_missing_target_without_packet() {
        let building = BuildingComp::new(point2_pack(24, 25), item_block(), TeamId(1));
        let missing_unit = picked_build_payload(None, Some(&building), true);
        let missing_build = picked_build_payload(Some(UnitRef::Unit { id: 50 }), None, true);

        assert_eq!(
            missing_unit.rejection,
            Some(PickedBuildPayloadRejectReason::MissingUnit)
        );
        assert!(missing_unit.packet.is_none());
        assert_eq!(
            missing_build.rejection,
            Some(PickedBuildPayloadRejectReason::MissingBuild)
        );
        assert!(missing_build.packet.is_none());
    }

    #[test]
    fn client_request_build_payload_packet_uses_client_payload_shape() {
        let building = BuildingComp::new(point2_pack(26, 27), item_block(), TeamId(1));
        let packet = client_request_build_payload_packet(&building);

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.build, BuildingRef::new(point2_pack(26, 27)));
    }
}
