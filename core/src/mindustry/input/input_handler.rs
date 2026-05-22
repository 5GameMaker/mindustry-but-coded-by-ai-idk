//! Pure input-handler helpers mirroring selected upstream `mindustry.input.InputHandler` paths.
//!
//! This module intentionally keeps UI, event-bus and live networking side effects
//! outside. Callers provide validation predicates and receive explicit plans
//! such as tile-config rollback packets.

use crate::mindustry::entities::comp::{
    building::{BuildingConfigChange, BuildingConfigRollback},
    player::PlayerUnitState,
    BuildingComp, PayloadState, PlayerComp, UnitComp,
};
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::io::{BuildingRef, EntityRef, TypeValue, UnitRef};
use crate::mindustry::net::{
    BuildingControlSelectCallPacket, DeletePlansCallPacket, DropItemCallPacket,
    PayloadDroppedCallPacket, PickedBuildPayloadCallPacket, PickedUnitPayloadCallPacket,
    PingLocationCallPacket, RequestBuildPayloadCallPacket, RequestDropPayloadCallPacket,
    RequestItemCallPacket, RequestUnitPayloadCallPacket, RotateBlockCallPacket,
    TakeItemsCallPacket, TileConfigCallPacket, TileTapCallPacket, TransferInventoryCallPacket,
    TransferItemToCallPacket, UnitClearCallPacket, UnitControlCallPacket,
    UnitEnteredPayloadCallPacket,
};
use crate::mindustry::vars::TILE_SIZE;
use crate::mindustry::world::{meta::BuildVisibility, point2_pack};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingControlSelectRejectReason {
    MissingPlayer,
    MissingBuild,
    PlayerDead,
    AdminDenied,
    TeamMismatch,
    CannotControl,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BuildingControlSelectContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingControlSelectOutcome {
    pub accepted: bool,
    pub rejection: Option<BuildingControlSelectRejectReason>,
    pub packet: Option<BuildingControlSelectCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitControlRejectReason {
    MissingPlayer,
    MissingUnit,
    AdminDenied,
    PossessionDisabled,
    InvalidUnit,
    TeamMismatch,
    CannotControl,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnitControlContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub possession_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitControlOutcome {
    pub accepted: bool,
    pub rejection: Option<UnitControlRejectReason>,
    pub previous_unit: Option<UnitRef>,
    pub current_unit: Option<UnitRef>,
    pub packet: Option<UnitControlCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitClearRejectReason {
    MissingPlayer,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnitClearContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub dock_respawn_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitClearOutcome {
    pub accepted: bool,
    pub rejection: Option<UnitClearRejectReason>,
    pub previous_unit: Option<UnitRef>,
    pub cleared_unit: bool,
    pub dock_respawn: bool,
    pub packet: Option<UnitClearCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestUnitPayloadRejectReason {
    MissingPlayer,
    MissingUnit,
    MissingPayloadComponent,
    MissingTarget,
    TargetNotAi,
    TargetNotGrounded,
    OutOfRange,
    CannotPickup,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RequestUnitPayloadContext {
    pub player: Option<EntityRef>,
    pub within_range: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestUnitPayloadOutcome {
    pub accepted: bool,
    pub rejection: Option<RequestUnitPayloadRejectReason>,
    pub packet: Option<PickedUnitPayloadCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickedUnitPayloadRejectReason {
    MissingUnit,
    MissingTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedUnitPayloadOutcome {
    pub accepted: bool,
    pub rejection: Option<PickedUnitPayloadRejectReason>,
    pub packet: Option<PickedUnitPayloadCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestDropPayloadRejectReason {
    MissingPlayer,
    ClientSide,
    PlayerDead,
    MissingUnit,
    MissingPayloadComponent,
    EmptyPayload,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RequestDropPayloadContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
    pub net_client: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestDropPayloadOutcome {
    pub accepted: bool,
    pub rejection: Option<RequestDropPayloadRejectReason>,
    pub requested_x: f32,
    pub requested_y: f32,
    pub clamped_x: f32,
    pub clamped_y: f32,
    pub packet: Option<PayloadDroppedCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadDroppedRejectReason {
    MissingUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadDroppedOutcome {
    pub accepted: bool,
    pub rejection: Option<PayloadDroppedRejectReason>,
    pub packet: Option<PayloadDroppedCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitEnteredPayloadRejectReason {
    MissingUnit,
    MissingBuild,
    TeamMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitEnteredPayloadOutcome {
    pub accepted: bool,
    pub rejection: Option<UnitEnteredPayloadRejectReason>,
    pub packet: Option<UnitEnteredPayloadCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropItemRejectReason {
    MissingPlayer,
    MissingUnit,
    EmptyStack,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DropItemContext {
    pub local_player: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropItemOutcome {
    pub accepted: bool,
    pub rejection: Option<DropItemRejectReason>,
    pub previous_item: Option<String>,
    pub previous_amount: i32,
    pub packet: Option<DropItemCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingLocationRejectReason {
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingLocationContext {
    pub player_id: Option<i32>,
    pub local_player: bool,
    pub same_team_visible: bool,
    pub max_text_len: usize,
}

impl Default for PingLocationContext {
    fn default() -> Self {
        Self {
            player_id: None,
            local_player: false,
            same_team_visible: false,
            max_text_len: 40,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingLocationOutcome {
    pub accepted: bool,
    pub rejection: Option<PingLocationRejectReason>,
    pub displayed_text: Option<String>,
    pub packet: Option<PingLocationCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletePlansRejectReason {
    MissingPlayer,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeletePlansContext {
    pub player_id: Option<i32>,
    pub local_player: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePlansOutcome {
    pub accepted: bool,
    pub rejection: Option<DeletePlansRejectReason>,
    pub removed: usize,
    pub packet: Option<DeletePlansCallPacket>,
    pub should_raise_validate: bool,
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

impl BuildingControlSelectOutcome {
    fn rejected(
        context: &BuildingControlSelectContext,
        reason: BuildingControlSelectRejectReason,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl UnitControlOutcome {
    fn rejected(
        context: &UnitControlContext,
        reason: UnitControlRejectReason,
        previous_unit: Option<UnitRef>,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            previous_unit,
            current_unit: previous_unit,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl UnitClearOutcome {
    fn rejected(
        context: &UnitClearContext,
        reason: UnitClearRejectReason,
        previous_unit: Option<UnitRef>,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            previous_unit,
            cleared_unit: false,
            dock_respawn: false,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl RequestUnitPayloadOutcome {
    fn rejected(reason: RequestUnitPayloadRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            packet: None,
        }
    }
}

impl PickedUnitPayloadOutcome {
    fn rejected(reason: PickedUnitPayloadRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            packet: None,
        }
    }
}

impl RequestDropPayloadOutcome {
    fn rejected(
        context: &RequestDropPayloadContext,
        reason: RequestDropPayloadRejectReason,
        requested_x: f32,
        requested_y: f32,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            requested_x,
            requested_y,
            clamped_x: requested_x,
            clamped_y: requested_y,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl PayloadDroppedOutcome {
    fn rejected(reason: PayloadDroppedRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            packet: None,
        }
    }
}

impl UnitEnteredPayloadOutcome {
    fn rejected(reason: UnitEnteredPayloadRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            packet: None,
        }
    }
}

impl DropItemOutcome {
    fn rejected(
        context: &DropItemContext,
        reason: DropItemRejectReason,
        previous_item: Option<String>,
        previous_amount: i32,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            previous_item,
            previous_amount,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl PingLocationOutcome {
    fn rejected(context: &PingLocationContext, reason: PingLocationRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            displayed_text: None,
            packet: None,
            should_raise_validate: !context.local_player,
        }
    }
}

impl DeletePlansOutcome {
    fn rejected(
        context: &DeletePlansContext,
        reason: DeletePlansRejectReason,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            removed: 0,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

fn player_unit_ref(player: &PlayerComp, unit: &UnitComp) -> UnitRef {
    player.unit_ref().unwrap_or(UnitRef::Unit { id: unit.id() })
}

fn clamp_drop_position(unit: &UnitComp, x: f32, y: f32) -> (f32, f32) {
    let max_distance = TILE_SIZE as f32 * 4.0;
    let dx = x - unit.x();
    let dy = y - unit.y();
    let distance = (dx * dx + dy * dy).sqrt();

    if distance <= max_distance || distance <= f32::EPSILON {
        (x, y)
    } else {
        let scale = max_distance / distance;
        (unit.x() + dx * scale, unit.y() + dy * scale)
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

pub fn building_control_select<A, C>(
    context: BuildingControlSelectContext,
    player: Option<&PlayerComp>,
    build: Option<&BuildingComp>,
    admin_allows: A,
    can_control_select: C,
) -> BuildingControlSelectOutcome
where
    A: FnOnce(&PlayerComp, &BuildingComp) -> bool,
    C: FnOnce(&PlayerComp, &BuildingComp) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return BuildingControlSelectOutcome::rejected(
            &context,
            BuildingControlSelectRejectReason::MissingPlayer,
            false,
        );
    }
    let player = player.unwrap();

    let Some(build) = build else {
        return BuildingControlSelectOutcome::rejected(
            &context,
            BuildingControlSelectRejectReason::MissingBuild,
            false,
        );
    };

    if player.dead() {
        return BuildingControlSelectOutcome::rejected(
            &context,
            BuildingControlSelectRejectReason::PlayerDead,
            false,
        );
    }

    if !admin_allows(player, build) {
        return BuildingControlSelectOutcome::rejected(
            &context,
            BuildingControlSelectRejectReason::AdminDenied,
            true,
        );
    }

    if player.team != build.team {
        return BuildingControlSelectOutcome::rejected(
            &context,
            BuildingControlSelectRejectReason::TeamMismatch,
            false,
        );
    }

    if !can_control_select(player, build) {
        return BuildingControlSelectOutcome::rejected(
            &context,
            BuildingControlSelectRejectReason::CannotControl,
            false,
        );
    }

    BuildingControlSelectOutcome {
        accepted: true,
        rejection: None,
        packet: Some(BuildingControlSelectCallPacket {
            player: context.player.unwrap_or_else(EntityRef::null),
            build: BuildingRef::new(build.tile_pos),
        }),
        should_raise_validate: false,
    }
}

pub fn client_building_control_select_packet(
    build: &BuildingComp,
) -> BuildingControlSelectCallPacket {
    BuildingControlSelectCallPacket {
        player: EntityRef::null(),
        build: BuildingRef::new(build.tile_pos),
    }
}

pub fn unit_control<A>(
    context: UnitControlContext,
    player: Option<&mut PlayerComp>,
    unit: Option<&UnitComp>,
    is_ai: bool,
    player_controllable: bool,
    admin_allows: A,
) -> UnitControlOutcome
where
    A: FnOnce(&PlayerComp, Option<&UnitComp>) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return UnitControlOutcome::rejected(
            &context,
            UnitControlRejectReason::MissingPlayer,
            None,
            false,
        );
    }
    let player = player.unwrap();
    let previous_unit = player.unit_ref();

    if !context.possession_allowed {
        return UnitControlOutcome::rejected(
            &context,
            UnitControlRejectReason::PossessionDisabled,
            previous_unit,
            true,
        );
    }

    if !admin_allows(player, unit) {
        return UnitControlOutcome::rejected(
            &context,
            UnitControlRejectReason::AdminDenied,
            previous_unit,
            true,
        );
    }

    let Some(unit) = unit else {
        return UnitControlOutcome::rejected(
            &context,
            UnitControlRejectReason::MissingUnit,
            previous_unit,
            false,
        );
    };

    if unit.health.dead || !unit.is_valid() || !is_ai {
        return UnitControlOutcome::rejected(
            &context,
            UnitControlRejectReason::InvalidUnit,
            previous_unit,
            true,
        );
    }

    if unit.team_id() != player.team {
        return UnitControlOutcome::rejected(
            &context,
            UnitControlRejectReason::TeamMismatch,
            previous_unit,
            true,
        );
    }

    if !player_controllable {
        return UnitControlOutcome::rejected(
            &context,
            UnitControlRejectReason::CannotControl,
            previous_unit,
            true,
        );
    }

    let current_unit = UnitRef::Unit { id: unit.id() };
    player.set_unit_state(PlayerUnitState::unit(unit.id()).with_valid(true));

    UnitControlOutcome {
        accepted: true,
        rejection: None,
        previous_unit,
        current_unit: Some(current_unit),
        packet: Some(UnitControlCallPacket {
            player: context.player.unwrap_or_else(EntityRef::null),
            unit: current_unit,
        }),
        should_raise_validate: false,
    }
}

pub fn client_unit_control_packet(unit: Option<UnitRef>) -> UnitControlCallPacket {
    UnitControlCallPacket {
        player: EntityRef::null(),
        unit: unit.unwrap_or(UnitRef::Null),
    }
}

pub fn unit_clear<A>(
    context: UnitClearContext,
    player: Option<&mut PlayerComp>,
    admin_allows: A,
) -> UnitClearOutcome
where
    A: FnOnce(&PlayerComp) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return UnitClearOutcome::rejected(
            &context,
            UnitClearRejectReason::MissingPlayer,
            None,
            false,
        );
    }
    let player = player.unwrap();
    let previous_unit = player.unit_ref();

    if !admin_allows(player) {
        return UnitClearOutcome::rejected(
            &context,
            UnitClearRejectReason::AdminDenied,
            previous_unit,
            true,
        );
    }

    let mut cleared_unit = false;
    if !context.dock_respawn_available {
        player.clear_unit();
        cleared_unit = true;
    }

    UnitClearOutcome {
        accepted: true,
        rejection: None,
        previous_unit,
        cleared_unit,
        dock_respawn: context.dock_respawn_available,
        packet: Some(UnitClearCallPacket {
            player: context.player.unwrap_or_else(EntityRef::null),
        }),
        should_raise_validate: false,
    }
}

pub fn client_unit_clear_packet() -> UnitClearCallPacket {
    UnitClearCallPacket {
        player: EntityRef::null(),
    }
}

pub fn request_unit_payload(
    context: RequestUnitPayloadContext,
    player: Option<&PlayerComp>,
    unit: Option<&UnitComp>,
    target: Option<&UnitComp>,
    target_is_ai: bool,
    target_allowed_in_payloads: bool,
) -> RequestUnitPayloadOutcome {
    if context.player.is_none() || player.is_none() {
        return RequestUnitPayloadOutcome::rejected(RequestUnitPayloadRejectReason::MissingPlayer);
    }
    let player = player.unwrap();

    let Some(unit) = unit else {
        return RequestUnitPayloadOutcome::rejected(RequestUnitPayloadRejectReason::MissingUnit);
    };

    let Some(payload) = unit.payload.as_ref() else {
        return RequestUnitPayloadOutcome::rejected(
            RequestUnitPayloadRejectReason::MissingPayloadComponent,
        );
    };

    let Some(target) = target else {
        return RequestUnitPayloadOutcome::rejected(RequestUnitPayloadRejectReason::MissingTarget);
    };

    if !target_is_ai {
        return RequestUnitPayloadOutcome::rejected(RequestUnitPayloadRejectReason::TargetNotAi);
    }

    if !target.is_grounded() {
        return RequestUnitPayloadOutcome::rejected(
            RequestUnitPayloadRejectReason::TargetNotGrounded,
        );
    }

    if !context.within_range {
        return RequestUnitPayloadOutcome::rejected(RequestUnitPayloadRejectReason::OutOfRange);
    }

    if !payload.can_pickup_unit(
        target.type_info.hit_size,
        target.team_id(),
        target_is_ai,
        target_allowed_in_payloads,
    ) {
        return RequestUnitPayloadOutcome::rejected(RequestUnitPayloadRejectReason::CannotPickup);
    }

    RequestUnitPayloadOutcome {
        accepted: true,
        rejection: None,
        packet: Some(PickedUnitPayloadCallPacket {
            unit: player_unit_ref(player, unit),
            target: UnitRef::Unit { id: target.id() },
        }),
    }
}

pub fn client_request_unit_payload_packet(target: UnitRef) -> RequestUnitPayloadCallPacket {
    RequestUnitPayloadCallPacket {
        player: EntityRef::null(),
        target,
    }
}

pub fn picked_unit_payload(
    unit: Option<UnitRef>,
    target: Option<UnitRef>,
) -> PickedUnitPayloadOutcome {
    let Some(unit) = unit else {
        return PickedUnitPayloadOutcome::rejected(PickedUnitPayloadRejectReason::MissingUnit);
    };

    let Some(target) = target else {
        return PickedUnitPayloadOutcome::rejected(PickedUnitPayloadRejectReason::MissingTarget);
    };

    PickedUnitPayloadOutcome {
        accepted: true,
        rejection: None,
        packet: Some(PickedUnitPayloadCallPacket { unit, target }),
    }
}

pub fn request_drop_payload<A>(
    context: RequestDropPayloadContext,
    player: Option<&PlayerComp>,
    unit: Option<&UnitComp>,
    x: f32,
    y: f32,
    admin_allows: A,
) -> RequestDropPayloadOutcome
where
    A: FnOnce(&PlayerComp, &UnitComp) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return RequestDropPayloadOutcome::rejected(
            &context,
            RequestDropPayloadRejectReason::MissingPlayer,
            x,
            y,
            false,
        );
    }
    let player = player.unwrap();

    if context.net_client {
        return RequestDropPayloadOutcome::rejected(
            &context,
            RequestDropPayloadRejectReason::ClientSide,
            x,
            y,
            false,
        );
    }

    if player.dead() {
        return RequestDropPayloadOutcome::rejected(
            &context,
            RequestDropPayloadRejectReason::PlayerDead,
            x,
            y,
            false,
        );
    }

    let Some(unit) = unit else {
        return RequestDropPayloadOutcome::rejected(
            &context,
            RequestDropPayloadRejectReason::MissingUnit,
            x,
            y,
            false,
        );
    };

    let Some(payload) = unit.payload.as_ref() else {
        return RequestDropPayloadOutcome::rejected(
            &context,
            RequestDropPayloadRejectReason::MissingPayloadComponent,
            x,
            y,
            false,
        );
    };

    if !payload.has_payload() {
        return RequestDropPayloadOutcome::rejected(
            &context,
            RequestDropPayloadRejectReason::EmptyPayload,
            x,
            y,
            false,
        );
    }

    if !admin_allows(player, unit) {
        return RequestDropPayloadOutcome::rejected(
            &context,
            RequestDropPayloadRejectReason::AdminDenied,
            x,
            y,
            true,
        );
    }

    let (clamped_x, clamped_y) = clamp_drop_position(unit, x, y);
    RequestDropPayloadOutcome {
        accepted: true,
        rejection: None,
        requested_x: x,
        requested_y: y,
        clamped_x,
        clamped_y,
        packet: Some(PayloadDroppedCallPacket {
            unit: player_unit_ref(player, unit),
            x: clamped_x,
            y: clamped_y,
        }),
        should_raise_validate: false,
    }
}

pub fn client_request_drop_payload_packet(x: f32, y: f32) -> RequestDropPayloadCallPacket {
    RequestDropPayloadCallPacket {
        player: EntityRef::null(),
        x,
        y,
    }
}

pub fn payload_dropped(unit: Option<UnitRef>, x: f32, y: f32) -> PayloadDroppedOutcome {
    let Some(unit) = unit else {
        return PayloadDroppedOutcome::rejected(PayloadDroppedRejectReason::MissingUnit);
    };

    PayloadDroppedOutcome {
        accepted: true,
        rejection: None,
        packet: Some(PayloadDroppedCallPacket { unit, x, y }),
    }
}

pub fn unit_entered_payload(
    unit: Option<&UnitComp>,
    build: Option<&BuildingComp>,
) -> UnitEnteredPayloadOutcome {
    let Some(unit) = unit else {
        return UnitEnteredPayloadOutcome::rejected(UnitEnteredPayloadRejectReason::MissingUnit);
    };

    let Some(build) = build else {
        return UnitEnteredPayloadOutcome::rejected(UnitEnteredPayloadRejectReason::MissingBuild);
    };

    if unit.team_id() != build.team {
        return UnitEnteredPayloadOutcome::rejected(UnitEnteredPayloadRejectReason::TeamMismatch);
    }

    UnitEnteredPayloadOutcome {
        accepted: true,
        rejection: None,
        packet: Some(UnitEnteredPayloadCallPacket {
            unit: UnitRef::Unit { id: unit.id() },
            build: BuildingRef::new(build.tile_pos),
        }),
    }
}

pub fn drop_item(
    context: DropItemContext,
    player: Option<&PlayerComp>,
    unit: Option<&mut UnitComp>,
    angle: f32,
) -> DropItemOutcome {
    if player.is_none() {
        return DropItemOutcome::rejected(
            &context,
            DropItemRejectReason::MissingPlayer,
            None,
            0,
            false,
        );
    }

    let Some(unit) = unit else {
        return DropItemOutcome::rejected(
            &context,
            DropItemRejectReason::MissingUnit,
            None,
            0,
            false,
        );
    };

    let previous_item = unit.items.item().map(str::to_owned);
    let previous_amount = unit.items.stack.amount;
    if previous_amount <= 0 {
        return DropItemOutcome::rejected(
            &context,
            DropItemRejectReason::EmptyStack,
            previous_item,
            previous_amount,
            true,
        );
    }

    unit.items.clear_item();
    DropItemOutcome {
        accepted: true,
        rejection: None,
        previous_item,
        previous_amount,
        packet: Some(DropItemCallPacket { angle }),
        should_raise_validate: false,
    }
}

pub fn client_drop_item_packet(angle: f32) -> DropItemCallPacket {
    DropItemCallPacket { angle }
}

pub fn ping_location<A>(
    context: PingLocationContext,
    player: Option<&mut PlayerComp>,
    x: f32,
    y: f32,
    text: Option<String>,
    admin_allows: A,
) -> PingLocationOutcome
where
    A: FnOnce(Option<&PlayerComp>, f32, f32, Option<&str>) -> bool,
{
    if !admin_allows(player.as_deref(), x, y, text.as_deref()) {
        return PingLocationOutcome::rejected(&context, PingLocationRejectReason::AdminDenied);
    }

    let text = text.unwrap_or_default();
    let displayed_text = if text.is_empty() {
        None
    } else if text.chars().count() > context.max_text_len {
        Some(text.chars().take(context.max_text_len).collect::<String>() + "...")
    } else {
        Some(text.clone())
    };

    if context.same_team_visible {
        if let Some(player) = player {
            player.ping_x = x;
            player.ping_y = y;
            player.ping_time = 1.0;
            player.ping_text = displayed_text.clone();
        }
    }

    PingLocationOutcome {
        accepted: true,
        rejection: None,
        displayed_text,
        packet: Some(PingLocationCallPacket {
            player_id: context.player_id,
            x,
            y,
            text,
        }),
        should_raise_validate: false,
    }
}

pub fn client_ping_location_packet(x: f32, y: f32, text: Option<String>) -> PingLocationCallPacket {
    PingLocationCallPacket {
        player_id: None,
        x,
        y,
        text: text.unwrap_or_default(),
    }
}

pub fn delete_plans<A>(
    context: DeletePlansContext,
    player_present: bool,
    plans: &mut Vec<BuildPlan>,
    positions: &[i32],
    admin_allows: A,
) -> DeletePlansOutcome
where
    A: FnOnce(&[i32]) -> bool,
{
    if !admin_allows(positions) {
        return DeletePlansOutcome::rejected(&context, DeletePlansRejectReason::AdminDenied, true);
    }

    if !player_present {
        return DeletePlansOutcome::rejected(
            &context,
            DeletePlansRejectReason::MissingPlayer,
            false,
        );
    }

    let before = plans.len();
    plans.retain(|plan| !positions.contains(&point2_pack(plan.x, plan.y)));
    let removed = before - plans.len();

    DeletePlansOutcome {
        accepted: true,
        rejection: None,
        removed,
        packet: Some(DeletePlansCallPacket {
            player_id: context.player_id,
            positions: positions.to_vec(),
        }),
        should_raise_validate: false,
    }
}

pub fn client_delete_plans_packet(positions: Vec<i32>) -> DeletePlansCallPacket {
    DeletePlansCallPacket {
        player_id: None,
        positions,
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

    #[test]
    fn building_control_select_accepts_valid_build_and_records_packet() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(51).with_valid(true));
        let building = BuildingComp::new(point2_pack(28, 29), item_block(), TeamId(1));

        let outcome = building_control_select(
            BuildingControlSelectContext {
                player: Some(EntityRef::new(12)),
                local_player: false,
            },
            Some(&player),
            Some(&building),
            |player, build| player.team == build.team,
            |_, _| true,
        );

        assert!(outcome.accepted);
        assert!(!outcome.should_raise_validate);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(12));
        assert_eq!(packet.build, BuildingRef::new(point2_pack(28, 29)));
    }

    #[test]
    fn building_control_select_rejects_admin_denied_as_validate_error() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(51).with_valid(true));
        let building = BuildingComp::new(point2_pack(28, 29), item_block(), TeamId(1));

        let outcome = building_control_select(
            BuildingControlSelectContext {
                player: Some(EntityRef::new(12)),
                local_player: false,
            },
            Some(&player),
            Some(&building),
            |_, _| false,
            |_, _| true,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(BuildingControlSelectRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn unit_control_accepts_valid_unit_and_updates_player_unit() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(51).with_valid(true));
        let mut unit = UnitComp::new(52, unit_type(10), TeamId(1));
        unit.add();

        let outcome = unit_control(
            UnitControlContext {
                player: Some(EntityRef::new(13)),
                local_player: false,
                possession_allowed: true,
            },
            Some(&mut player),
            Some(&unit),
            true,
            true,
            |player, unit| unit.is_some_and(|unit| player.team == unit.team_id()),
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.previous_unit, Some(UnitRef::Unit { id: 51 }));
        assert_eq!(outcome.current_unit, Some(UnitRef::Unit { id: 52 }));
        assert_eq!(player.unit_ref(), Some(UnitRef::Unit { id: 52 }));
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(13));
        assert_eq!(packet.unit, UnitRef::Unit { id: 52 });
    }

    #[test]
    fn unit_control_rejects_team_mismatch_without_mutating_player() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(51).with_valid(true));
        let mut unit = UnitComp::new(53, unit_type(10), TeamId(2));
        unit.add();

        let outcome = unit_control(
            UnitControlContext {
                player: Some(EntityRef::new(13)),
                local_player: false,
                possession_allowed: true,
            },
            Some(&mut player),
            Some(&unit),
            true,
            true,
            |_, _| true,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(UnitControlRejectReason::TeamMismatch)
        );
        assert!(outcome.should_raise_validate);
        assert_eq!(player.unit_ref(), Some(UnitRef::Unit { id: 51 }));
    }

    #[test]
    fn unit_clear_accepts_respawn_and_clears_player_unit() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(54).with_valid(true));

        let outcome = unit_clear(
            UnitClearContext {
                player: Some(EntityRef::new(14)),
                local_player: false,
                dock_respawn_available: false,
            },
            Some(&mut player),
            |_| true,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.previous_unit, Some(UnitRef::Unit { id: 54 }));
        assert!(outcome.cleared_unit);
        assert!(!outcome.dock_respawn);
        assert_eq!(player.unit_ref(), None);
        assert_eq!(outcome.packet.unwrap().player, EntityRef::new(14));
    }

    #[test]
    fn unit_clear_rejects_forbidden_respawn_without_packet() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(54).with_valid(true));

        let outcome = unit_clear(
            UnitClearContext {
                player: Some(EntityRef::new(14)),
                local_player: false,
                dock_respawn_available: false,
            },
            Some(&mut player),
            |_| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(outcome.rejection, Some(UnitClearRejectReason::AdminDenied));
        assert!(outcome.should_raise_validate);
        assert_eq!(player.unit_ref(), Some(UnitRef::Unit { id: 54 }));
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn control_client_packets_use_client_payload_shape() {
        let building = BuildingComp::new(point2_pack(30, 31), item_block(), TeamId(1));
        let build_packet = client_building_control_select_packet(&building);
        let unit_packet = client_unit_control_packet(Some(UnitRef::Unit { id: 55 }));
        let clear_packet = client_unit_clear_packet();

        assert_eq!(build_packet.player, EntityRef::null());
        assert_eq!(build_packet.build, BuildingRef::new(point2_pack(30, 31)));
        assert_eq!(unit_packet.player, EntityRef::null());
        assert_eq!(unit_packet.unit, UnitRef::Unit { id: 55 });
        assert_eq!(clear_packet.player, EntityRef::null());
    }

    #[test]
    fn request_unit_payload_accepts_ai_grounded_target() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(60).with_valid(true));
        let mut unit = payload_unit(60, TeamId(1), 512.0);
        unit.payload.as_mut().unwrap().pickup_units = true;
        let target = UnitComp::new(61, unit_type(10), TeamId(1));

        let outcome = request_unit_payload(
            RequestUnitPayloadContext {
                player: Some(EntityRef::new(15)),
                within_range: true,
            },
            Some(&player),
            Some(&unit),
            Some(&target),
            true,
            true,
        );

        assert!(outcome.accepted);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 60 });
        assert_eq!(packet.target, UnitRef::Unit { id: 61 });
    }

    #[test]
    fn request_unit_payload_rejects_out_of_range_without_packet() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(60).with_valid(true));
        let mut unit = payload_unit(60, TeamId(1), 512.0);
        unit.payload.as_mut().unwrap().pickup_units = true;
        let target = UnitComp::new(61, unit_type(10), TeamId(1));

        let outcome = request_unit_payload(
            RequestUnitPayloadContext {
                player: Some(EntityRef::new(15)),
                within_range: false,
            },
            Some(&player),
            Some(&unit),
            Some(&target),
            true,
            true,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(RequestUnitPayloadRejectReason::OutOfRange)
        );
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn picked_unit_payload_records_unit_and_target_refs() {
        let picked = picked_unit_payload(
            Some(UnitRef::Unit { id: 62 }),
            Some(UnitRef::Unit { id: 63 }),
        );
        let missing = picked_unit_payload(Some(UnitRef::Unit { id: 62 }), None);

        assert!(picked.accepted);
        let packet = picked.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 62 });
        assert_eq!(packet.target, UnitRef::Unit { id: 63 });
        assert_eq!(
            missing.rejection,
            Some(PickedUnitPayloadRejectReason::MissingTarget)
        );
        assert!(missing.packet.is_none());
    }

    #[test]
    fn request_drop_payload_clamps_to_java_margin_and_plans_drop_packet() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(64).with_valid(true));
        let mut unit = payload_unit(64, TeamId(1), 512.0);
        unit.set_pos(0.0, 0.0);
        unit.payload.as_mut().unwrap().add_payload(PayloadState {
            kind: PayloadKind::Build,
            size: 2.0,
        });

        let outcome = request_drop_payload(
            RequestDropPayloadContext {
                player: Some(EntityRef::new(16)),
                local_player: false,
                net_client: false,
            },
            Some(&player),
            Some(&unit),
            100.0,
            0.0,
            |player, unit| player.team == unit.team_id(),
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.requested_x, 100.0);
        assert_eq!(outcome.requested_y, 0.0);
        assert!((outcome.clamped_x - TILE_SIZE as f32 * 4.0).abs() <= f32::EPSILON);
        assert_eq!(outcome.clamped_y, 0.0);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 64 });
        assert_eq!(packet.x, outcome.clamped_x);
        assert_eq!(packet.y, outcome.clamped_y);
    }

    #[test]
    fn request_drop_payload_rejects_admin_denied_as_validate_error() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(64).with_valid(true));
        let mut unit = payload_unit(64, TeamId(1), 512.0);
        unit.payload.as_mut().unwrap().add_payload(PayloadState {
            kind: PayloadKind::Build,
            size: 2.0,
        });

        let outcome = request_drop_payload(
            RequestDropPayloadContext {
                player: Some(EntityRef::new(16)),
                local_player: false,
                net_client: false,
            },
            Some(&player),
            Some(&unit),
            1.0,
            2.0,
            |_, _| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(RequestDropPayloadRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn unit_payload_client_packets_use_client_payload_shape() {
        let request_unit = client_request_unit_payload_packet(UnitRef::Unit { id: 65 });
        let request_drop = client_request_drop_payload_packet(3.0, 4.0);
        let dropped = payload_dropped(Some(UnitRef::Unit { id: 66 }), 5.0, 6.0);

        assert_eq!(request_unit.player, EntityRef::null());
        assert_eq!(request_unit.target, UnitRef::Unit { id: 65 });
        assert_eq!(request_drop.player, EntityRef::null());
        assert_eq!((request_drop.x, request_drop.y), (3.0, 4.0));
        assert!(dropped.accepted);
        assert_eq!(dropped.packet.unwrap().unit, UnitRef::Unit { id: 66 });
    }

    #[test]
    fn unit_entered_payload_accepts_same_team_unit_and_building() {
        let unit = UnitComp::new(67, unit_type(10), TeamId(1));
        let building = BuildingComp::new(point2_pack(32, 33), item_block(), TeamId(1));

        let outcome = unit_entered_payload(Some(&unit), Some(&building));

        assert!(outcome.accepted);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 67 });
        assert_eq!(packet.build, BuildingRef::new(point2_pack(32, 33)));
    }

    #[test]
    fn unit_entered_payload_rejects_team_mismatch_without_packet() {
        let unit = UnitComp::new(68, unit_type(10), TeamId(2));
        let building = BuildingComp::new(point2_pack(32, 33), item_block(), TeamId(1));

        let outcome = unit_entered_payload(Some(&unit), Some(&building));

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(UnitEnteredPayloadRejectReason::TeamMismatch)
        );
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn drop_item_clears_unit_stack_and_records_angle_packet() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(69).with_valid(true));
        let mut unit = UnitComp::new(69, unit_type(10), TeamId(1));
        unit.items.add_item_amount("copper", 3);

        let outcome = drop_item(
            DropItemContext {
                local_player: false,
            },
            Some(&player),
            Some(&mut unit),
            45.5,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.previous_item.as_deref(), Some("copper"));
        assert_eq!(outcome.previous_amount, 3);
        assert_eq!(unit.items.stack.amount, 0);
        assert_eq!(outcome.packet.unwrap().angle, 45.5);
    }

    #[test]
    fn drop_item_rejects_empty_stack_as_validate_error() {
        let mut player = PlayerComp::new(TeamId(1));
        player.set_unit_state(PlayerUnitState::unit(69).with_valid(true));
        let mut unit = UnitComp::new(69, unit_type(10), TeamId(1));

        let outcome = drop_item(
            DropItemContext {
                local_player: false,
            },
            Some(&player),
            Some(&mut unit),
            -10.0,
        );

        assert!(!outcome.accepted);
        assert_eq!(outcome.rejection, Some(DropItemRejectReason::EmptyStack));
        assert!(outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn client_drop_item_packet_keeps_angle_payload() {
        let packet = client_drop_item_packet(-45.5);

        assert_eq!(packet.angle, -45.5);
    }

    #[test]
    fn ping_location_updates_visible_same_team_player_and_truncates_text() {
        let mut player = PlayerComp::new(TeamId(1));

        let outcome = ping_location(
            PingLocationContext {
                player_id: Some(70),
                local_player: false,
                same_team_visible: true,
                max_text_len: 4,
            },
            Some(&mut player),
            12.0,
            34.0,
            Some("abcdef".into()),
            |_, x, y, text| x == 12.0 && y == 34.0 && text == Some("abcdef"),
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.displayed_text.as_deref(), Some("abcd..."));
        assert_eq!((player.ping_x, player.ping_y), (12.0, 34.0));
        assert_eq!(player.ping_time, 1.0);
        assert_eq!(player.ping_text.as_deref(), Some("abcd..."));
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player_id, Some(70));
        assert_eq!(packet.text, "abcdef");
    }

    #[test]
    fn ping_location_rejects_admin_denied_as_validate_error() {
        let mut player = PlayerComp::new(TeamId(1));

        let outcome = ping_location(
            PingLocationContext {
                player_id: Some(70),
                local_player: false,
                same_team_visible: true,
                max_text_len: 40,
            },
            Some(&mut player),
            12.0,
            34.0,
            Some("blocked".into()),
            |_, _, _, _| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(PingLocationRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
        assert_eq!(player.ping_time, 0.0);
    }

    #[test]
    fn client_ping_location_packet_uses_client_payload_shape() {
        let packet = client_ping_location_packet(5.0, 6.0, Some("go".into()));

        assert_eq!(packet.player_id, None);
        assert_eq!((packet.x, packet.y), (5.0, 6.0));
        assert_eq!(packet.text, "go");
    }

    #[test]
    fn delete_plans_removes_matching_positions_and_records_packet() {
        let mut plans = vec![
            BuildPlan::new_place(1, 2, 0, "router"),
            BuildPlan::new_place(3, 4, 0, "duo"),
        ];
        let remove = vec![point2_pack(1, 2)];

        let outcome = delete_plans(
            DeletePlansContext {
                player_id: Some(71),
                local_player: false,
            },
            true,
            &mut plans,
            &remove,
            |positions| positions == remove.as_slice(),
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.removed, 1);
        assert_eq!(plans.len(), 1);
        assert_eq!((plans[0].x, plans[0].y), (3, 4));
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player_id, Some(71));
        assert_eq!(packet.positions, remove);
    }

    #[test]
    fn delete_plans_rejects_admin_denied_as_validate_error() {
        let mut plans = vec![BuildPlan::new_place(1, 2, 0, "router")];

        let outcome = delete_plans(
            DeletePlansContext {
                player_id: Some(71),
                local_player: false,
            },
            true,
            &mut plans,
            &[point2_pack(1, 2)],
            |_| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(DeletePlansRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert_eq!(plans.len(), 1);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn client_delete_plans_packet_uses_client_payload_shape() {
        let packet = client_delete_plans_packet(vec![point2_pack(5, 6)]);

        assert_eq!(packet.player_id, None);
        assert_eq!(packet.positions, vec![point2_pack(5, 6)]);
    }
}
