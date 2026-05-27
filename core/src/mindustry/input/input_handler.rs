//! Pure input-handler helpers mirroring selected upstream `mindustry.input.InputHandler` paths.
//!
//! This module intentionally keeps UI, event-bus and live networking side effects
//! outside. Callers provide validation predicates and receive explicit plans
//! such as tile-config rollback packets.

use crate::mindustry::ai::{unit_command::UnitCommand, unit_stance::UnitStance};
use crate::mindustry::ctype::{Content, ContentId, ContentType};
use crate::mindustry::entities::comp::{
    building::{BuildingConfigChange, BuildingConfigRollback},
    player::PlayerUnitState,
    BuildingComp, PayloadState, PlayerComp, UnitComp, UnitControllerState,
};
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::game::{BlockPlan as TeamBlockPlan, Teams};
use crate::mindustry::io::type_io::{CommandWire, ContentRef};
use crate::mindustry::io::Point2;
use crate::mindustry::io::{BuildingRef, EntityRef, TeamId, TypeValue, UnitRef, Vec2};
use crate::mindustry::net::{
    BuildingControlSelectCallPacket, ClearItemsCallPacket, ClearLiquidsCallPacket,
    CommandBuildingCallPacket, CommandUnitsCallPacket, DeletePlansCallPacket, DropItemCallPacket,
    PayloadDroppedCallPacket, PickedBuildPayloadCallPacket, PickedUnitPayloadCallPacket,
    PingLocationCallPacket, RemoveQueueBlockCallPacket, RequestBuildPayloadCallPacket,
    RequestDropPayloadCallPacket, RequestItemCallPacket, RequestUnitPayloadCallPacket,
    RotateBlockCallPacket, SetItemCallPacket, SetItemsCallPacket, SetLiquidCallPacket,
    SetLiquidsCallPacket, SetTileItemsCallPacket, SetTileLiquidsCallPacket,
    SetUnitCommandCallPacket, SetUnitStanceCallPacket, TakeItemsCallPacket, TileConfigCallPacket,
    TileTapCallPacket, TransferInventoryCallPacket, TransferItemEffectCallPacket,
    TransferItemToCallPacket, TransferItemToUnitCallPacket, UnitBuildingControlSelectCallPacket,
    UnitClearCallPacket, UnitControlCallPacket, UnitEnteredPayloadCallPacket,
};
use crate::mindustry::r#type::{ItemStack, LiquidStack};
use crate::mindustry::vars::TILE_SIZE;
use crate::mindustry::world::{
    meta::{BlockFlag, BuildVisibility},
    placement_bounds, point2_pack, BuildBounds,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConfigTapFrame {
    pub config_shown: bool,
    pub selected_on_configure_tapped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileTappedFrame {
    pub build_present: bool,
    pub command_mode: bool,
    pub build_commandable: bool,
    pub block_configurable: bool,
    pub build_interactable: bool,
    pub config_shown: bool,
    pub should_show_configure: bool,
    pub selected_accepts_configure_build_tap: bool,
    pub config_has_mouse: bool,
    pub block_consumes_tap: bool,
    pub block_synthetic: bool,
    pub block_allow_config_inventory: bool,
    pub block_has_items: bool,
    pub items_total: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileTappedAction {
    HidePlanConfig,
    HideInventory,
    HideConfig,
    ClearCommandBuildings,
    PlayConfigureSound,
    ShowConfig,
    CallTapped,
    ShowInventory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileTappedPlan {
    pub consumed: bool,
    pub showed_inventory: bool,
    pub actions: Vec<TileTappedAction>,
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
pub enum ItemSyncRejectReason {
    MissingBuild,
    MissingItemStorage,
    MissingItem,
    UnknownItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetItemOutcome {
    pub accepted: bool,
    pub rejection: Option<ItemSyncRejectReason>,
    pub previous_amount: i32,
    pub new_amount: i32,
    pub packet: Option<SetItemCallPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetItemsOutcome {
    pub accepted: bool,
    pub rejection: Option<ItemSyncRejectReason>,
    pub applied_entries: usize,
    pub packet: Option<SetItemsCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearItemsOutcome {
    pub accepted: bool,
    pub rejection: Option<ItemSyncRejectReason>,
    pub cleared_total: i32,
    pub packet: Option<ClearItemsCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidSyncRejectReason {
    MissingBuild,
    MissingLiquidStorage,
    MissingLiquid,
    UnknownLiquid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetLiquidOutcome {
    pub accepted: bool,
    pub rejection: Option<LiquidSyncRejectReason>,
    pub previous_amount: f32,
    pub new_amount: f32,
    pub packet: Option<SetLiquidCallPacket>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetLiquidsOutcome {
    pub accepted: bool,
    pub rejection: Option<LiquidSyncRejectReason>,
    pub applied_entries: usize,
    pub packet: Option<SetLiquidsCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClearLiquidsOutcome {
    pub accepted: bool,
    pub rejection: Option<LiquidSyncRejectReason>,
    pub cleared_current: Option<i16>,
    pub cleared_amount: f32,
    pub packet: Option<ClearLiquidsCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferItemEffectRejectReason {
    MissingTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferItemEffectOutcome {
    pub accepted: bool,
    pub rejection: Option<TransferItemEffectRejectReason>,
    pub packet: Option<TransferItemEffectCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeItemsRejectReason {
    MissingBuild,
    MissingItemStorage,
    MissingUnit,
    MissingItem,
    UnknownItem,
    NothingRemoved,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemRemoveStackPlan {
    pub build: BuildingRef,
    pub item: Option<String>,
    pub item_id: i16,
    pub amount_removed: i32,
    pub source_is_core: bool,
    pub source_is_storage: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TakeItemsOutcome {
    pub accepted: bool,
    pub rejection: Option<TakeItemsRejectReason>,
    pub requested_amount: i32,
    pub removed_amount: i32,
    pub remove_stack: Option<ItemRemoveStackPlan>,
    pub transfer_effects: Vec<TransferItemEffectCallPacket>,
    pub packet: Option<TakeItemsCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferItemToUnitRejectReason {
    MissingTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferItemToUnitOutcome {
    pub accepted: bool,
    pub rejection: Option<TransferItemToUnitRejectReason>,
    pub item_added: bool,
    pub packet: Option<TransferItemToUnitCallPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTileItemsOutcome {
    pub accepted: bool,
    pub rejection: Option<ItemSyncRejectReason>,
    pub applied_positions: usize,
    pub packet: Option<SetTileItemsCallPacket>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTileLiquidsOutcome {
    pub accepted: bool,
    pub rejection: Option<LiquidSyncRejectReason>,
    pub applied_positions: usize,
    pub packet: Option<SetTileLiquidsCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferItemToRejectReason {
    MissingBuild,
    MissingItemStorage,
    MissingItem,
    UnknownItem,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferItemToOutcome {
    pub accepted: bool,
    pub rejection: Option<TransferItemToRejectReason>,
    pub requested_amount: i32,
    pub unit_previous_amount: Option<i32>,
    pub unit_new_amount: Option<i32>,
    pub building_previous_amount: i32,
    pub building_new_amount: i32,
    pub effect_count: usize,
    pub packet: Option<TransferItemToCallPacket>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandUnitsRejectReason {
    MissingPlayer,
    MissingUnits,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandUnitsContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandUnitsOutcome {
    pub accepted: bool,
    pub rejection: Option<CommandUnitsRejectReason>,
    pub commanded: usize,
    pub packet: Option<CommandUnitsCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetUnitCommandRejectReason {
    MissingPlayer,
    MissingUnits,
    MissingCommand,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetUnitCommandContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetUnitCommandOutcome {
    pub accepted: bool,
    pub rejection: Option<SetUnitCommandRejectReason>,
    pub commanded: usize,
    pub packet: Option<SetUnitCommandCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetUnitStanceRejectReason {
    MissingPlayer,
    MissingUnits,
    MissingStance,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetUnitStanceContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetUnitStanceOutcome {
    pub accepted: bool,
    pub rejection: Option<SetUnitStanceRejectReason>,
    pub commanded: usize,
    pub packet: Option<SetUnitStanceCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBuildingRejectReason {
    MissingPlayer,
    MissingBuildings,
    AdminDenied,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandBuildingContext {
    pub player: Option<EntityRef>,
    pub local_player: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBuildingOutcome {
    pub accepted: bool,
    pub rejection: Option<CommandBuildingRejectReason>,
    pub commanded_positions: Vec<i32>,
    pub packet: Option<CommandBuildingCallPacket>,
    pub should_raise_validate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveQueueBlockRejectReason {
    MissingUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveQueueBlockOutcome {
    pub accepted: bool,
    pub rejection: Option<RemoveQueueBlockRejectReason>,
    pub removed: Option<BuildPlan>,
    pub packet: Option<RemoveQueueBlockCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitBuildingControlSelectRejectReason {
    MissingUnit,
    MissingBuild,
    UnitDead,
    TeamMismatch,
    NotSelectable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitBuildingControlSelectOutcome {
    pub accepted: bool,
    pub rejection: Option<UnitBuildingControlSelectRejectReason>,
    pub packet: Option<UnitBuildingControlSelectCallPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildPlanBlockTransform {
    pub size: i32,
    pub offset: f32,
    pub rotate: bool,
    pub lock_rotation: bool,
    pub invert_flip: bool,
}

impl BuildPlanBlockTransform {
    pub const fn new(
        size: i32,
        offset: f32,
        rotate: bool,
        lock_rotation: bool,
        invert_flip: bool,
    ) -> Self {
        Self {
            size,
            offset,
            rotate,
            lock_rotation,
            invert_flip,
        }
    }

    pub fn single() -> Self {
        Self::new(1, TILE_SIZE as f32 / 2.0, true, false, false)
    }

    pub fn java_block(size: i32) -> Self {
        Self::new(
            size,
            ((size + 1) % 2) as f32 * TILE_SIZE as f32 / 2.0,
            true,
            false,
            false,
        )
    }

    pub fn plan_rotation(self, rotation: i32) -> i32 {
        if !self.rotate && self.lock_rotation {
            0
        } else {
            rotation.rem_euclid(4)
        }
    }

    pub fn flip_rotation(self, rotation: i32, flip_x: bool) -> i32 {
        if (flip_x == (rotation.rem_euclid(4) % 2 == 0)) != self.invert_flip {
            self.plan_rotation(rotation + 2)
        } else {
            self.plan_rotation(rotation)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueuedBuildPlanSnapshot {
    pub plan: BuildPlan,
    pub footprint: BuildPlanBlockTransform,
    /// Mirrors Java's identity `plan != ignore` check. The caller marks the
    /// exact queued plan that should be ignored for this placement probe.
    pub ignored: bool,
    /// Precomputed `candidate.canReplace(plan.block)` result for the queued
    /// plan. Keeping it explicit avoids pulling the whole block registry into
    /// this pure input planner.
    pub candidate_can_replace: bool,
}

impl QueuedBuildPlanSnapshot {
    pub fn new(plan: BuildPlan, footprint: BuildPlanBlockTransform) -> Self {
        Self {
            plan,
            footprint,
            ignored: false,
            candidate_can_replace: false,
        }
    }

    pub fn ignored(mut self) -> Self {
        self.ignored = true;
        self
    }

    pub fn replaceable_by_candidate(mut self) -> Self {
        self.candidate_can_replace = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidPlaceFrame {
    /// Result of upstream `Build.validPlace(...)` or
    /// `Build.validPlaceIgnoreUnits(...)`.
    pub base_valid: bool,
    pub player_is_builder: bool,
    pub x: i32,
    pub y: i32,
    pub block: BuildPlanBlockTransform,
    pub queued_plans: Vec<QueuedBuildPlanSnapshot>,
}

impl Default for ValidPlaceFrame {
    fn default() -> Self {
        Self {
            base_valid: false,
            player_is_builder: false,
            x: 0,
            y: 0,
            block: BuildPlanBlockTransform::single(),
            queued_plans: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakBlockTileSnapshot {
    pub x: i32,
    pub y: i32,
    /// Java redirects a clicked multiblock tile through `tile.build.tile`.
    pub build_origin: Option<(i32, i32)>,
}

impl BreakBlockTileSnapshot {
    pub const fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            build_origin: None,
        }
    }

    pub const fn with_build_origin(mut self, x: i32, y: i32) -> Self {
        self.build_origin = Some((x, y));
        self
    }

    pub const fn target(self) -> (i32, i32) {
        match self.build_origin {
            Some(origin) => origin,
            None => (self.x, self.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BreakBlockFrame {
    pub player_is_builder: bool,
    pub tile: Option<BreakBlockTileSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TryBreakBlockFrame {
    pub valid_break: bool,
    pub break_frame: BreakBlockFrame,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RebuildBlockPlanSnapshot {
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    pub block: String,
    pub config: TypeValue,
    pub footprint: BuildPlanBlockTransform,
}

impl RebuildBlockPlanSnapshot {
    pub fn new(
        x: i32,
        y: i32,
        rotation: i32,
        block: impl Into<String>,
        config: TypeValue,
        footprint: BuildPlanBlockTransform,
    ) -> Self {
        Self {
            x,
            y,
            rotation,
            block: block.into(),
            config,
            footprint,
        }
    }

    pub fn to_build_plan(&self) -> BuildPlan {
        BuildPlan::new_config(
            self.x,
            self.y,
            self.rotation,
            self.block.clone(),
            self.config.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RebuildRepairCandidate {
    pub scan_x: i32,
    pub scan_y: i32,
    pub tile_pos: i32,
    pub can_repair: bool,
    pub plan: BuildPlan,
}

impl RebuildRepairCandidate {
    pub fn new(scan_x: i32, scan_y: i32, tile_pos: i32, can_repair: bool, plan: BuildPlan) -> Self {
        Self {
            scan_x,
            scan_y,
            tile_pos,
            can_repair,
            plan,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RebuildAreaSelection {
    pub x: i32,
    pub y: i32,
    pub x2: i32,
    pub y2: i32,
    pub rotation: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RebuildAreaFrame {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub rotation: i32,
    pub max_length: i32,
    pub broken_plans: Vec<RebuildBlockPlanSnapshot>,
    pub repair_candidates: Vec<RebuildRepairCandidate>,
}

impl Default for RebuildAreaFrame {
    fn default() -> Self {
        Self {
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
            rotation: 0,
            max_length: 999_999_999,
            broken_plans: Vec::new(),
            repair_candidates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RebuildAreaPlan {
    pub selection: RebuildAreaSelection,
    pub rebuild_plans: Vec<BuildPlan>,
    pub repair_plans: Vec<BuildPlan>,
    pub repair_tile_positions: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlanSnapshot {
    pub plan: BuildPlan,
    pub footprint: BuildPlanBlockTransform,
    /// Optional packed tile position, used for Java `r.tile() == tile` style
    /// selection checks without requiring a live `Tile` reference.
    pub tile_pos: Option<i32>,
}

impl BuildPlanSnapshot {
    pub fn new(plan: BuildPlan, footprint: BuildPlanBlockTransform) -> Self {
        let tile_pos = Some(point2_pack(plan.x, plan.y));
        Self {
            plan,
            footprint,
            tile_pos,
        }
    }

    pub fn without_tile(mut self) -> Self {
        self.tile_pos = None;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlushBuildPlanCandidate {
    pub plan: BuildPlan,
    pub footprint: BuildPlanBlockTransform,
    /// Result of `validPlace(plan.x, plan.y, plan.block, plan.rotation, null, true)`.
    pub valid_place: bool,
}

impl FlushBuildPlanCandidate {
    pub fn new(plan: BuildPlan, footprint: BuildPlanBlockTransform, valid_place: bool) -> Self {
        Self {
            plan,
            footprint,
            valid_place,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildQueuePosition {
    Head,
    Tail,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlushBuildPlanAction {
    pub plan: BuildPlan,
    pub position: BuildQueuePosition,
    pub call_on_new_plan: bool,
    pub insert_into_plan_tree: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlushSelectPlanAction {
    Add { plan: BuildPlan },
    ReplaceSelect { index: usize, plan: BuildPlan },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlushSelectPlansPlan {
    pub actions: Vec<FlushSelectPlanAction>,
    pub final_select_plans: Vec<BuildPlan>,
    pub skipped: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveSelectionTileCandidate {
    pub scan_x: i32,
    pub scan_y: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub tile_pos: i32,
    pub valid_break: bool,
    pub break_tile: BreakBlockTileSnapshot,
}

impl RemoveSelectionTileCandidate {
    pub fn new(scan_x: i32, scan_y: i32, tile_x: i32, tile_y: i32, valid_break: bool) -> Self {
        Self {
            scan_x,
            scan_y,
            tile_x,
            tile_y,
            tile_pos: point2_pack(tile_x, tile_y),
            valid_break,
            break_tile: BreakBlockTileSnapshot::new(scan_x, scan_y),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveSelectionTeamPlanSnapshot {
    pub x: i32,
    pub y: i32,
    pub footprint: BuildPlanBlockTransform,
}

impl RemoveSelectionTeamPlanSnapshot {
    pub fn new(x: i32, y: i32, footprint: BuildPlanBlockTransform) -> Self {
        Self { x, y, footprint }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveSelectionFrame {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub rotation: i32,
    pub flush: bool,
    pub max_length: i32,
    pub player_dead: bool,
    pub player_is_builder: bool,
    pub net_active: bool,
    pub world_tiles: Vec<RemoveSelectionTileCandidate>,
    pub unit_plans: Vec<BuildPlanSnapshot>,
    pub select_plans: Vec<BuildPlanSnapshot>,
    pub team_plans: Vec<RemoveSelectionTeamPlanSnapshot>,
}

impl Default for RemoveSelectionFrame {
    fn default() -> Self {
        Self {
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
            rotation: 0,
            flush: false,
            max_length: 0,
            player_dead: false,
            player_is_builder: false,
            net_active: false,
            world_tiles: Vec::new(),
            unit_plans: Vec::new(),
            select_plans: Vec::new(),
            team_plans: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveSelectionPlan {
    pub selection: RebuildAreaSelection,
    pub immediate_break_plans: Vec<BuildPlan>,
    pub queued_break_plans: Vec<BuildPlan>,
    pub remove_unit_plan_indices: Vec<usize>,
    pub remove_select_plan_indices: Vec<usize>,
    pub remove_team_plan_indices: Vec<usize>,
    pub removed_team_plan_positions: Vec<i32>,
    pub network_delete_positions: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinePlacementBlock {
    pub name: String,
    pub footprint: BuildPlanBlockTransform,
    pub allow_diagonal: bool,
    pub allow_rectangle_placement: bool,
    pub conveyor_placement: bool,
    pub swap_diagonal_placement: bool,
    pub ignore_line_rotation: bool,
}

impl LinePlacementBlock {
    pub fn new(name: impl Into<String>, footprint: BuildPlanBlockTransform) -> Self {
        Self {
            name: name.into(),
            footprint,
            allow_diagonal: false,
            allow_rectangle_placement: false,
            conveyor_placement: false,
            swap_diagonal_placement: false,
            ignore_line_rotation: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainedBuildEndpoint {
    pub chained: bool,
    pub rotation: i32,
    pub candidate_can_replace: bool,
}

impl ChainedBuildEndpoint {
    pub const fn absent() -> Self {
        Self {
            chained: false,
            rotation: -1,
            candidate_can_replace: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IterateLineFrame {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub rotation: i32,
    pub override_line_rotation: bool,
    pub diagonal_pressed: bool,
    pub swap_diagonal_setting: bool,
    pub mobile: bool,
    pub conveyor_pathfinding: bool,
    pub block: Option<LinePlacementBlock>,
    pub start_build: ChainedBuildEndpoint,
    pub end_build: ChainedBuildEndpoint,
    pub second_to_last_chained: bool,
    pub astar_path: Option<Vec<Point2>>,
    pub upgrade_path: Option<Vec<Point2>>,
    /// Result of upstream `block.changePlacementPath(points, rotation, diagonal)`.
    /// Callers can provide an already-mutated path while this planner keeps the
    /// block-specific hook outside pure logic.
    pub changed_points: Option<Vec<Point2>>,
}

impl Default for IterateLineFrame {
    fn default() -> Self {
        Self {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 0,
            rotation: 0,
            override_line_rotation: false,
            diagonal_pressed: false,
            swap_diagonal_setting: false,
            mobile: false,
            conveyor_pathfinding: false,
            block: None,
            start_build: ChainedBuildEndpoint::absent(),
            end_build: ChainedBuildEndpoint::absent(),
            second_to_last_chained: false,
            astar_path: None,
            upgrade_path: None,
            changed_points: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceLine {
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    pub last: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IterateLinePlan {
    pub diagonal: bool,
    pub base_rotation: i32,
    pub end_rotation: Option<i32>,
    pub points: Vec<Point2>,
    pub lines: Vec<PlaceLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineReplacement {
    pub x: i32,
    pub y: i32,
    pub block: String,
    pub unlocked: bool,
}

impl LineReplacement {
    pub fn new(x: i32, y: i32, block: impl Into<String>, unlocked: bool) -> Self {
        Self {
            x,
            y,
            block: block.into(),
            unlocked,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateLineFrame {
    pub line: IterateLineFrame,
    pub next_config: TypeValue,
    pub block_replace: bool,
    pub replacements: Vec<LineReplacement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateLinePlan {
    pub line: IterateLinePlan,
    pub line_plans: Vec<BuildPlan>,
    pub final_rotation: i32,
    pub handle_placement_line: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TapPlayerFrame {
    pub within_select_range: bool,
    pub player_dead: bool,
    pub stack_amount: i32,
    pub block_selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapPlayerPlan {
    pub accepted: bool,
    pub dropping_item: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CanMineFrame {
    pub scene_has_mouse: bool,
    pub player_dead: bool,
    pub unit_valid_mine: bool,
    pub unit_accepts_mine_result: bool,
    pub double_tap_mine: bool,
    pub floor_player_unmineable: bool,
    pub overlay_player_unmineable: bool,
    pub overlay_has_item_drop: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BeginMineFrame {
    pub player_dead: bool,
    pub can_mine: bool,
    pub tile_pos: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeginMinePlan {
    pub accepted: bool,
    pub mine_tile: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StopMineFrame {
    pub player_dead: bool,
    pub current_mine_tile: Option<i32>,
    pub requested_tile: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopMinePlan {
    pub accepted: bool,
    pub clear_mine_tile: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairDerelictFrame {
    pub tile_present: bool,
    pub build_present: bool,
    pub player_dead: bool,
    pub editor: bool,
    pub player_team_derelict: bool,
    pub build_team_derelict: bool,
    pub block_unlocked_host: bool,
    pub valid_place: bool,
    pub build_x: i32,
    pub build_y: i32,
    pub build_rotation: i32,
    pub block: Option<String>,
    pub config: TypeValue,
}

impl Default for RepairDerelictFrame {
    fn default() -> Self {
        Self {
            tile_present: false,
            build_present: false,
            player_dead: false,
            editor: false,
            player_team_derelict: false,
            build_team_derelict: false,
            block_unlocked_host: false,
            valid_place: false,
            build_x: 0,
            build_y: 0,
            build_rotation: 0,
            block: None,
            config: TypeValue::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairDerelictPlan {
    pub accepted: bool,
    pub build_plan: Option<BuildPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectUnitCandidate {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub hit_size: f32,
    pub is_ai: bool,
    pub player_controllable: bool,
    pub commandable: bool,
    pub in_fog_to_player: bool,
}

impl SelectUnitCandidate {
    pub const fn new(id: i32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            hit_size: 8.0,
            is_ai: true,
            player_controllable: true,
            commandable: true,
            in_fog_to_player: false,
        }
    }

    pub fn dst_edge(self, x: f32, y: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt() - self.hit_size / 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectBuildingCandidate {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub same_team: bool,
    pub commandable: bool,
    pub control_block: bool,
    pub can_control: bool,
    pub controlled_unit_id: Option<i32>,
    pub controlled_unit_is_player: bool,
    pub controlled_unit_is_ai: bool,
    pub can_control_select: bool,
}

impl SelectBuildingCandidate {
    pub const fn new(id: i32, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            same_team: true,
            commandable: false,
            control_block: false,
            can_control: false,
            controlled_unit_id: None,
            controlled_unit_is_player: false,
            controlled_unit_is_ai: false,
            can_control_select: false,
        }
    }

    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    pub fn intersects(self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let left = self.x.min(self.x + self.width);
        let right = self.x.max(self.x + self.width);
        let bottom = self.y.min(self.y + self.height);
        let top = self.y.max(self.y + self.height);
        let q_left = x.min(x + w);
        let q_right = x.max(x + w);
        let q_bottom = y.min(y + h);
        let q_top = y.max(y + h);
        left < q_right && right > q_left && bottom < q_top && top > q_bottom
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedUnitSource {
    NearbyUnit,
    ControlBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedUnitPlan {
    pub unit_id: i32,
    pub source: SelectedUnitSource,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionRectFrame {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputStateFrame {
    pub state_menu: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputStatePlan {
    pub clear_controlled_type: bool,
    pub clear_logic_cutscene: bool,
    pub force_hide_config: bool,
    pub command_mode: Option<bool>,
    pub command_rect: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectUnitsRectFrame {
    pub command_mode: bool,
    pub command_rect: bool,
    pub tapped_one: bool,
    pub multi_unit_select: bool,
    pub selected_units: Vec<i32>,
    pub rect_units: Vec<i32>,
    pub rect_buildings: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectUnitsRectPlan {
    pub selected_units: Vec<i32>,
    pub command_buildings: Vec<i32>,
    pub command_rect: bool,
    pub fire_change_event: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectTypedUnitsFrame {
    pub command_mode: bool,
    pub selected_unit_type: Option<String>,
    pub visible_units: Vec<(i32, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectTypedUnitsPlan {
    pub selected_units: Vec<i32>,
    pub fire_change_event: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandBuildTapCandidate {
    pub id: i32,
    pub same_team: bool,
    pub commandable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapCommandUnitFrame {
    pub command_mode: bool,
    pub selected_unit: Option<i32>,
    pub build: Option<CommandBuildTapCandidate>,
    pub selected_units: Vec<i32>,
    pub command_buildings: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapCommandUnitPlan {
    pub selected_units: Vec<i32>,
    pub command_buildings: Vec<i32>,
    pub fire_change_event: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandAttackTarget {
    Building(i32),
    Unit(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandTapFrame {
    pub command_mode: bool,
    pub selected_units: Vec<i32>,
    pub command_buildings: Vec<i32>,
    pub target: Vec2,
    pub queue: bool,
    pub allied_build_target: Option<i32>,
    pub enemy_build_target: Option<i32>,
    pub enemy_unit_target: Option<i32>,
    pub max_chunk_size: usize,
}

impl Default for CommandTapFrame {
    fn default() -> Self {
        Self {
            command_mode: false,
            selected_units: Vec::new(),
            command_buildings: Vec::new(),
            target: Vec2::new(0.0, 0.0),
            queue: false,
            allied_build_target: None,
            enemy_build_target: None,
            enemy_unit_target: None,
            max_chunk_size: 200,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandUnitsBatchPlan {
    pub unit_ids: Vec<i32>,
    pub attack_target: Option<CommandAttackTarget>,
    pub target: Vec2,
    pub queue: bool,
    pub final_batch: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandTapPlan {
    pub unit_batches: Vec<CommandUnitsBatchPlan>,
    pub command_buildings: Option<(Vec<i32>, Vec2)>,
    pub fire_attack_event: bool,
    pub fire_position_event: bool,
}

pub const COMMAND_UNIT_SELECT_RADIUS_SCALE: f32 = 1.0;
pub const COMMAND_OVERLAY_LINE_LIMIT: f32 = 6.5;
pub const COMMAND_OVERLAY_ALPHA: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOverlayColor {
    Accent,
    Remove,
    Malis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOverlayController {
    CommandAi,
    LogicAi,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOverlayTargetKind {
    Position,
    Entity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandOverlayTarget {
    pub pos: Vec2,
    pub kind: CommandOverlayTargetKind,
}

impl CommandOverlayTarget {
    pub const fn position(pos: Vec2) -> Self {
        Self {
            pos,
            kind: CommandOverlayTargetKind::Position,
        }
    }

    pub const fn entity(pos: Vec2) -> Self {
        Self {
            pos,
            kind: CommandOverlayTargetKind::Entity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandOverlayUnit {
    pub id: i32,
    pub pos: Vec2,
    pub hit_size: f32,
    pub allow_command: bool,
    pub is_flying: bool,
    pub allow_leg_step: bool,
    pub controller: CommandOverlayController,
    pub command_draw_target: bool,
    pub target_pos: Option<Vec2>,
    pub attack_target: Option<CommandOverlayTarget>,
    pub command_queue: Vec<CommandOverlayTarget>,
    pub enter_payload_command: bool,
    pub enter_payload_target_accepts: bool,
    pub loop_payload_command: bool,
    pub has_payload: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandOverlayBuilding {
    pub id: i32,
    pub pos: Vec2,
    pub hit_size: f32,
    pub command_position: Option<Vec2>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandOverlayFrame {
    pub command_mode: bool,
    pub flying_pass: bool,
    pub selected_units: Vec<CommandOverlayUnit>,
    pub command_buildings: Vec<CommandOverlayBuilding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMarkerSource {
    Selection,
    Commanded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandUnitMarkerPlan {
    pub id: i32,
    pub pos: Vec2,
    pub sides: i32,
    pub radius: f32,
    pub pulse_radius: f32,
    pub color: CommandOverlayColor,
    pub source: CommandMarkerSource,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandBuildingMarkerPlan {
    pub id: i32,
    pub pos: Vec2,
    pub sides: i32,
    pub radius: f32,
    pub pulse_radius: f32,
    pub color: CommandOverlayColor,
    pub source: CommandMarkerSource,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandOverlayLinePlan {
    pub from: Vec2,
    pub to: Vec2,
    pub from_margin: f32,
    pub to_margin: f32,
    pub color: CommandOverlayColor,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandTargetMarkerKind {
    MoveSquare,
    AttackTarget,
    EnterPayloadAccepted,
    EnterPayloadRejected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandTargetMarkerPlan {
    pub pos: Vec2,
    pub kind: CommandTargetMarkerKind,
    pub color: CommandOverlayColor,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPayloadIconKind {
    Upload,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandPayloadIconPlan {
    pub pos: Vec2,
    pub kind: CommandPayloadIconKind,
    pub offset_y: f32,
    pub size: f32,
    pub color: CommandOverlayColor,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandOverlayPlan {
    pub retained_selected_units: Vec<i32>,
    pub removed_selected_units: Vec<i32>,
    pub unit_markers: Vec<CommandUnitMarkerPlan>,
    pub building_markers: Vec<CommandBuildingMarkerPlan>,
    pub target_lines: Vec<CommandOverlayLinePlan>,
    pub target_markers: Vec<CommandTargetMarkerPlan>,
    pub queue_lines: Vec<CommandOverlayLinePlan>,
    pub queue_markers: Vec<CommandTargetMarkerPlan>,
    pub payload_icons: Vec<CommandPayloadIconPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandTargetsOverlayFrame {
    pub command_mode: bool,
    pub selected_units: Vec<CommandOverlayUnit>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandTargetsOverlayPlan {
    pub target_markers: Vec<CommandTargetMarkerPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandSelectableUnit {
    pub id: i32,
    pub pos: Vec2,
    pub hit_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandSelectableBuilding {
    pub id: i32,
    pub pos: Vec2,
    pub hit_size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSelectionOverlayFrame {
    pub command_mode: bool,
    pub command_rect: bool,
    pub rect: SelectionRectFrame,
    pub selected_units: Vec<i32>,
    pub command_buildings: Vec<i32>,
    pub rect_units: Vec<CommandSelectableUnit>,
    pub rect_buildings: Vec<CommandSelectableBuilding>,
    pub hover_unit: Option<CommandSelectableUnit>,
    pub multi_unit_select: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandSelectionOverlayPlan {
    pub unit_markers: Vec<CommandUnitMarkerPlan>,
    pub building_markers: Vec<CommandBuildingMarkerPlan>,
    pub rect_fill: Option<SelectionRectFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OtherPlayerPreviewOverlayFrame {
    pub local_player_id: i32,
    pub local_team: TeamId,
    pub now_millis: i64,
    pub delta: f32,
    pub mouse_world: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OtherPlayerPreviewBlock {
    pub size: i32,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OtherPlayerPreviewEntryPlan {
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    pub block: String,
    pub world_pos: Vec2,
    pub size: i32,
    pub anim_scale: f32,
    pub alpha: f32,
    pub overlapping_mouse: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OtherPlayerPreviewOverlapPlan {
    pub x: i32,
    pub y: i32,
    pub block: String,
    pub player_name: String,
    pub player_pos: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OtherPlayerPreviewOverlayPlan {
    pub player_id: i32,
    pub player_name: String,
    pub player_pos: Vec2,
    pub entries: Vec<OtherPlayerPreviewEntryPlan>,
    pub overlap: Option<OtherPlayerPreviewOverlapPlan>,
    pub dirty_rebuilt: bool,
    pub cleared_irrelevant: bool,
}

impl Default for OtherPlayerPreviewOverlayPlan {
    fn default() -> Self {
        Self {
            player_id: 0,
            player_name: String::new(),
            player_pos: Vec2::new(0.0, 0.0),
            entries: Vec::new(),
            overlap: None,
            dirty_rebuilt: false,
            cleared_irrelevant: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputHandlerLocalAction {
    UnitControlRemote,
    UnitControlLocal,
    RequestUnitPayload,
    RequestBuildPayload,
    RequestDropPayload { x: f32, y: f32 },
    TransferInventory { new_item_deposit_cooldown: f32 },
    DropItem { angle: f32 },
    ClearDroppingItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckUnitFrame {
    pub controlled_type_present: bool,
    pub controlled_type_player_controllable: bool,
    pub closest_unit_present: bool,
    pub block_control_unit_present: bool,
    pub net_client: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckUnitPlan {
    pub accepted: bool,
    pub action: Option<InputHandlerLocalAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PayloadPickupFrame {
    pub unit_is_payload: bool,
    pub pickup_unit_available: bool,
    pub build_present: bool,
    pub teams_can_interact: bool,
    pub stored_payload_pickable: bool,
    pub build_visibility_hidden: bool,
    pub build_can_pickup: bool,
    pub payload_can_pickup_build: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadPickupPlan {
    pub accepted: bool,
    pub action: Option<InputHandlerLocalAction>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadDropFrame {
    pub unit_is_payload: bool,
    pub can_drop_payload: bool,
    pub player_x: f32,
    pub player_y: f32,
}

impl Default for PayloadDropFrame {
    fn default() -> Self {
        Self {
            unit_is_payload: false,
            can_drop_payload: false,
            player_x: 0.0,
            player_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadDropPlan {
    pub accepted: bool,
    pub action: Option<InputHandlerLocalAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CanShootFrame {
    pub block_selected: bool,
    pub on_configurable: bool,
    pub dropping_item: bool,
    pub actively_building: bool,
    pub mech_flying: bool,
    pub mining: bool,
    pub command_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepositItemFrame {
    pub item_deposit_cooldown: f32,
    pub rules_item_deposit_cooldown: f32,
    pub block_deposit_cooldown: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TryDropItemsFrame {
    pub player_dead: bool,
    pub dropping_item: bool,
    pub stack_amount: i32,
    pub can_tap_player: bool,
    pub state_paused: bool,
    pub build_present: bool,
    pub build_accepts_stack: i32,
    pub build_interactable: bool,
    pub build_has_items: bool,
    pub build_allow_deposit: bool,
    pub can_deposit_item: bool,
    pub rules_item_deposit_cooldown: f32,
    pub drop_angle: f32,
}

impl Default for TryDropItemsFrame {
    fn default() -> Self {
        Self {
            player_dead: false,
            dropping_item: false,
            stack_amount: 0,
            can_tap_player: false,
            state_paused: false,
            build_present: false,
            build_accepts_stack: 0,
            build_interactable: false,
            build_has_items: false,
            build_allow_deposit: false,
            can_deposit_item: false,
            rules_item_deposit_cooldown: 0.0,
            drop_angle: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryDropItemsPlan {
    pub player_dead_ignored: bool,
    pub dropping_item: bool,
    pub action: Option<InputHandlerLocalAction>,
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

impl SetItemOutcome {
    fn rejected(reason: ItemSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            previous_amount: 0,
            new_amount: 0,
            packet: None,
        }
    }
}

impl SetItemsOutcome {
    fn rejected(reason: ItemSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            applied_entries: 0,
            packet: None,
        }
    }
}

impl ClearItemsOutcome {
    fn rejected(reason: ItemSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            cleared_total: 0,
            packet: None,
        }
    }
}

impl SetLiquidOutcome {
    fn rejected(reason: LiquidSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            previous_amount: 0.0,
            new_amount: 0.0,
            packet: None,
        }
    }
}

impl SetLiquidsOutcome {
    fn rejected(reason: LiquidSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            applied_entries: 0,
            packet: None,
        }
    }
}

impl ClearLiquidsOutcome {
    fn rejected(reason: LiquidSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            cleared_current: None,
            cleared_amount: 0.0,
            packet: None,
        }
    }
}

impl TransferItemEffectOutcome {
    fn rejected(reason: TransferItemEffectRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            packet: None,
        }
    }
}

impl TakeItemsOutcome {
    fn rejected(reason: TakeItemsRejectReason, requested_amount: i32) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            requested_amount,
            removed_amount: 0,
            remove_stack: None,
            transfer_effects: Vec::new(),
            packet: None,
        }
    }
}

impl TransferItemToUnitOutcome {
    fn rejected(reason: TransferItemToUnitRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            item_added: false,
            packet: None,
        }
    }
}

impl SetTileItemsOutcome {
    fn rejected(reason: ItemSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            applied_positions: 0,
            packet: None,
        }
    }
}

impl SetTileLiquidsOutcome {
    fn rejected(reason: LiquidSyncRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            applied_positions: 0,
            packet: None,
        }
    }
}

impl TransferItemToOutcome {
    fn rejected(reason: TransferItemToRejectReason, requested_amount: i32) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            requested_amount,
            unit_previous_amount: None,
            unit_new_amount: None,
            building_previous_amount: 0,
            building_new_amount: 0,
            effect_count: 0,
            packet: None,
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

impl CommandUnitsOutcome {
    fn rejected(
        context: &CommandUnitsContext,
        reason: CommandUnitsRejectReason,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            commanded: 0,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl SetUnitCommandOutcome {
    fn rejected(
        context: &SetUnitCommandContext,
        reason: SetUnitCommandRejectReason,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            commanded: 0,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl SetUnitStanceOutcome {
    fn rejected(
        context: &SetUnitStanceContext,
        reason: SetUnitStanceRejectReason,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            commanded: 0,
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl CommandBuildingOutcome {
    fn rejected(
        context: &CommandBuildingContext,
        reason: CommandBuildingRejectReason,
        validate_rejection: bool,
    ) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            commanded_positions: Vec::new(),
            packet: None,
            should_raise_validate: validate_rejection && !context.local_player,
        }
    }
}

impl RemoveQueueBlockOutcome {
    fn rejected(reason: RemoveQueueBlockRejectReason) -> Self {
        Self {
            accepted: false,
            rejection: Some(reason),
            removed: None,
            packet: None,
        }
    }
}

impl UnitBuildingControlSelectOutcome {
    fn rejected(reason: UnitBuildingControlSelectRejectReason) -> Self {
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

fn item_transfer_effect_count(amount: i32) -> usize {
    (amount / 3).clamp(1, 8) as usize
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

pub fn client_unit_cargo_unload_item_config_packet(
    build: &BuildingComp,
    item_id: ContentId,
) -> TileConfigCallPacket {
    client_tile_config_packet(
        build,
        TypeValue::Content(ContentRef::new(ContentType::Item, item_id)),
    )
}

pub fn client_unit_cargo_unload_clear_config_packet(build: &BuildingComp) -> TileConfigCallPacket {
    client_tile_config_packet(build, TypeValue::Null)
}

pub fn client_unit_factory_plan_config_packet(
    build: &BuildingComp,
    plan_index: i32,
) -> TileConfigCallPacket {
    client_tile_config_packet(build, TypeValue::Int(plan_index))
}

pub fn client_unit_factory_unit_config_packet(
    build: &BuildingComp,
    unit_id: ContentId,
) -> TileConfigCallPacket {
    client_tile_config_packet(
        build,
        TypeValue::Content(ContentRef::new(ContentType::Unit, unit_id)),
    )
}

pub fn client_unit_factory_command_config_packet(
    build: &BuildingComp,
    command_id: ContentId,
) -> TileConfigCallPacket {
    client_tile_config_packet(
        build,
        TypeValue::Content(ContentRef::new(ContentType::UnitCommand, command_id)),
    )
}

pub fn client_unit_factory_clear_command_packet(build: &BuildingComp) -> TileConfigCallPacket {
    client_tile_config_packet(build, TypeValue::Null)
}

pub fn client_reconstructor_command_config_packet(
    build: &BuildingComp,
    command_id: ContentId,
) -> TileConfigCallPacket {
    client_tile_config_packet(
        build,
        TypeValue::Content(ContentRef::new(ContentType::UnitCommand, command_id)),
    )
}

pub fn client_reconstructor_clear_command_packet(build: &BuildingComp) -> TileConfigCallPacket {
    client_tile_config_packet(build, TypeValue::Null)
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

pub fn check_config_tap_plan(frame: ConfigTapFrame) -> bool {
    frame.config_shown && frame.selected_on_configure_tapped
}

pub fn tile_tapped_plan(frame: TileTappedFrame) -> TileTappedPlan {
    let mut actions = vec![TileTappedAction::HidePlanConfig];

    if !frame.build_present {
        actions.push(TileTappedAction::HideInventory);
        actions.push(TileTappedAction::HideConfig);
        actions.push(TileTappedAction::ClearCommandBuildings);
        return TileTappedPlan {
            consumed: false,
            showed_inventory: false,
            actions,
        };
    }

    let mut consumed = false;
    let mut showed_inventory = false;

    if frame.build_commandable && frame.command_mode {
        consumed = true;
    } else if frame.block_configurable && frame.build_interactable {
        consumed = true;
        if (!frame.config_shown && frame.should_show_configure)
            || (frame.config_shown
                && frame.selected_accepts_configure_build_tap
                && frame.should_show_configure)
        {
            actions.push(TileTappedAction::PlayConfigureSound);
            actions.push(TileTappedAction::ShowConfig);
        }
    } else if !frame.config_has_mouse {
        if frame.config_shown && frame.selected_accepts_configure_build_tap {
            consumed = true;
            actions.push(TileTappedAction::HideConfig);
        }

        if frame.config_shown {
            consumed = true;
        }
    }

    if !consumed && frame.build_interactable {
        actions.push(TileTappedAction::CallTapped);
    }

    if frame.build_interactable && frame.block_consumes_tap {
        consumed = true;
    } else if frame.build_interactable
        && frame.block_synthetic
        && (!consumed || frame.block_allow_config_inventory)
        && frame.block_has_items
        && frame.items_total > 0
    {
        actions.push(TileTappedAction::ShowInventory);
        consumed = true;
        showed_inventory = true;
    }

    if !showed_inventory {
        actions.push(TileTappedAction::HideInventory);
    }

    TileTappedPlan {
        consumed,
        showed_inventory,
        actions,
    }
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

pub fn set_item<R>(
    build: Option<&mut BuildingComp>,
    item: Option<String>,
    amount: i32,
    resolve_item_id: R,
) -> SetItemOutcome
where
    R: FnOnce(&str) -> Option<i16>,
{
    let Some(build) = build else {
        return SetItemOutcome::rejected(ItemSyncRejectReason::MissingBuild);
    };
    let build_ref = BuildingRef::new(build.tile_pos);

    let Some(items) = build.items.as_mut() else {
        return SetItemOutcome::rejected(ItemSyncRejectReason::MissingItemStorage);
    };

    let Some(item_name) = item else {
        return SetItemOutcome::rejected(ItemSyncRejectReason::MissingItem);
    };

    let Some(item_id) = resolve_item_id(&item_name) else {
        return SetItemOutcome::rejected(ItemSyncRejectReason::UnknownItem);
    };

    let previous_amount = items.get(item_id);
    items.set(item_id, amount);

    SetItemOutcome {
        accepted: true,
        rejection: None,
        previous_amount,
        new_amount: amount,
        packet: Some(SetItemCallPacket {
            build: build_ref,
            item: Some(item_name),
            amount,
        }),
    }
}

pub fn set_items<R>(
    build: Option<&mut BuildingComp>,
    stacks: Vec<ItemStack>,
    mut resolve_item_id: R,
) -> SetItemsOutcome
where
    R: FnMut(&str) -> Option<i16>,
{
    let Some(build) = build else {
        return SetItemsOutcome::rejected(ItemSyncRejectReason::MissingBuild);
    };
    let build_ref = BuildingRef::new(build.tile_pos);

    if build.items.is_none() {
        return SetItemsOutcome::rejected(ItemSyncRejectReason::MissingItemStorage);
    }

    let mut resolved = Vec::with_capacity(stacks.len());
    for stack in &stacks {
        let Some(item_id) = resolve_item_id(&stack.item) else {
            return SetItemsOutcome::rejected(ItemSyncRejectReason::UnknownItem);
        };
        resolved.push((item_id, stack.amount));
    }

    let items = build.items.as_mut().expect("checked item module presence");
    for (item_id, amount) in resolved {
        items.set(item_id, amount);
    }

    SetItemsOutcome {
        accepted: true,
        rejection: None,
        applied_entries: stacks.len(),
        packet: Some(SetItemsCallPacket {
            build: build_ref,
            items: stacks,
        }),
    }
}

pub fn clear_items(build: Option<&mut BuildingComp>) -> ClearItemsOutcome {
    let Some(build) = build else {
        return ClearItemsOutcome::rejected(ItemSyncRejectReason::MissingBuild);
    };
    let build_ref = BuildingRef::new(build.tile_pos);

    let Some(items) = build.items.as_mut() else {
        return ClearItemsOutcome::rejected(ItemSyncRejectReason::MissingItemStorage);
    };

    let cleared_total = items.total();
    items.clear();

    ClearItemsOutcome {
        accepted: true,
        rejection: None,
        cleared_total,
        packet: Some(ClearItemsCallPacket { build: build_ref }),
    }
}

pub fn set_liquid<R>(
    build: Option<&mut BuildingComp>,
    liquid: Option<String>,
    amount: f32,
    resolve_liquid_id: R,
) -> SetLiquidOutcome
where
    R: FnOnce(&str) -> Option<i16>,
{
    let Some(build) = build else {
        return SetLiquidOutcome::rejected(LiquidSyncRejectReason::MissingBuild);
    };
    let build_ref = BuildingRef::new(build.tile_pos);

    let Some(liquids) = build.liquids.as_mut() else {
        return SetLiquidOutcome::rejected(LiquidSyncRejectReason::MissingLiquidStorage);
    };

    let Some(liquid_name) = liquid else {
        return SetLiquidOutcome::rejected(LiquidSyncRejectReason::MissingLiquid);
    };

    let Some(liquid_id) = resolve_liquid_id(&liquid_name) else {
        return SetLiquidOutcome::rejected(LiquidSyncRejectReason::UnknownLiquid);
    };

    let previous_amount = liquids.get(liquid_id);
    liquids.set(liquid_id, amount);

    SetLiquidOutcome {
        accepted: true,
        rejection: None,
        previous_amount,
        new_amount: amount,
        packet: Some(SetLiquidCallPacket {
            build: build_ref,
            liquid: Some(liquid_name),
            amount,
        }),
    }
}

pub fn set_liquids<R>(
    build: Option<&mut BuildingComp>,
    stacks: Vec<LiquidStack>,
    mut resolve_liquid_id: R,
) -> SetLiquidsOutcome
where
    R: FnMut(&str) -> Option<i16>,
{
    let Some(build) = build else {
        return SetLiquidsOutcome::rejected(LiquidSyncRejectReason::MissingBuild);
    };
    let build_ref = BuildingRef::new(build.tile_pos);

    if build.liquids.is_none() {
        return SetLiquidsOutcome::rejected(LiquidSyncRejectReason::MissingLiquidStorage);
    }

    let mut resolved = Vec::with_capacity(stacks.len());
    for stack in &stacks {
        let Some(liquid_id) = resolve_liquid_id(&stack.liquid) else {
            return SetLiquidsOutcome::rejected(LiquidSyncRejectReason::UnknownLiquid);
        };
        resolved.push((liquid_id, stack.amount));
    }

    let liquids = build
        .liquids
        .as_mut()
        .expect("checked liquid module presence");
    for (liquid_id, amount) in resolved {
        liquids.set(liquid_id, amount);
    }

    SetLiquidsOutcome {
        accepted: true,
        rejection: None,
        applied_entries: stacks.len(),
        packet: Some(SetLiquidsCallPacket {
            build: build_ref,
            liquids: stacks,
        }),
    }
}

pub fn clear_liquids(build: Option<&mut BuildingComp>) -> ClearLiquidsOutcome {
    let Some(build) = build else {
        return ClearLiquidsOutcome::rejected(LiquidSyncRejectReason::MissingBuild);
    };
    let build_ref = BuildingRef::new(build.tile_pos);

    let Some(liquids) = build.liquids.as_mut() else {
        return ClearLiquidsOutcome::rejected(LiquidSyncRejectReason::MissingLiquidStorage);
    };

    let cleared_current = liquids.current();
    let cleared_amount = liquids.current_amount();
    liquids.clear();

    ClearLiquidsOutcome {
        accepted: true,
        rejection: None,
        cleared_current,
        cleared_amount,
        packet: Some(ClearLiquidsCallPacket { build: build_ref }),
    }
}

pub fn transfer_item_effect(
    item: Option<String>,
    x: f32,
    y: f32,
    to: Option<EntityRef>,
) -> TransferItemEffectOutcome {
    let Some(to) = to.filter(|to| to.id.is_some()) else {
        return TransferItemEffectOutcome::rejected(TransferItemEffectRejectReason::MissingTarget);
    };

    TransferItemEffectOutcome {
        accepted: true,
        rejection: None,
        packet: Some(TransferItemEffectCallPacket { item, x, y, to }),
    }
}

pub fn take_items<R>(
    build: Option<&mut BuildingComp>,
    item: Option<String>,
    amount: i32,
    to: Option<&mut UnitComp>,
    resolve_item_id: R,
) -> TakeItemsOutcome
where
    R: FnOnce(&str) -> Option<i16>,
{
    let Some(build) = build else {
        return TakeItemsOutcome::rejected(TakeItemsRejectReason::MissingBuild, amount);
    };
    let build_ref = BuildingRef::new(build.tile_pos);
    let (x, y) = (build.x, build.y);
    let source_is_core = build.block.flags.contains(&BlockFlag::Core);
    let source_is_storage = build.block.flags.contains(&BlockFlag::Storage);

    let Some(items) = build.items.as_mut() else {
        return TakeItemsOutcome::rejected(TakeItemsRejectReason::MissingItemStorage, amount);
    };

    let Some(to) = to else {
        return TakeItemsOutcome::rejected(TakeItemsRejectReason::MissingUnit, amount);
    };
    let to_ref = UnitRef::Unit { id: to.id() };
    let to_entity = EntityRef::new(to.id());

    let Some(item_name) = item else {
        return TakeItemsOutcome::rejected(TakeItemsRejectReason::MissingItem, amount);
    };

    let Some(item_id) = resolve_item_id(&item_name) else {
        return TakeItemsOutcome::rejected(TakeItemsRejectReason::UnknownItem, amount);
    };

    let requested_removal = to.items.max_accepted(&item_name).min(amount).max(0);
    let removed_amount = items.get(item_id).min(requested_removal).max(0);
    if removed_amount == 0 {
        return TakeItemsOutcome::rejected(TakeItemsRejectReason::NothingRemoved, amount);
    }

    items.remove(item_id, removed_amount);
    to.items.add_item_amount(item_name.clone(), removed_amount);

    let transfer_effects = (0..item_transfer_effect_count(removed_amount))
        .map(|_| TransferItemEffectCallPacket {
            item: Some(item_name.clone()),
            x,
            y,
            to: to_entity,
        })
        .collect();

    TakeItemsOutcome {
        accepted: true,
        rejection: None,
        requested_amount: amount,
        removed_amount,
        remove_stack: Some(ItemRemoveStackPlan {
            build: build_ref,
            item: Some(item_name.clone()),
            item_id,
            amount_removed: removed_amount,
            source_is_core,
            source_is_storage,
        }),
        transfer_effects,
        packet: Some(TakeItemsCallPacket {
            build: build_ref,
            item: Some(item_name),
            amount: removed_amount,
            to: to_ref,
        }),
    }
}

pub fn transfer_item_to_unit(
    item: Option<String>,
    x: f32,
    y: f32,
    to: Option<&mut UnitComp>,
    to_ref: Option<EntityRef>,
) -> TransferItemToUnitOutcome {
    let target_ref = to_ref
        .filter(|to| to.id.is_some())
        .or_else(|| to.as_ref().map(|unit| EntityRef::new(unit.id())));
    let Some(target_ref) = target_ref else {
        return TransferItemToUnitOutcome::rejected(TransferItemToUnitRejectReason::MissingTarget);
    };

    let mut item_added = false;
    if let (Some(unit), Some(item_name)) = (to, item.clone()) {
        unit.items.add_item(item_name);
        item_added = true;
    }

    TransferItemToUnitOutcome {
        accepted: true,
        rejection: None,
        item_added,
        packet: Some(TransferItemToUnitCallPacket {
            item,
            x,
            y,
            to: target_ref,
        }),
    }
}

pub fn set_tile_items<R>(
    buildings: &mut [BuildingComp],
    item: Option<String>,
    amount: i32,
    positions: Vec<i32>,
    resolve_item_id: R,
) -> SetTileItemsOutcome
where
    R: FnOnce(&str) -> Option<i16>,
{
    let Some(item_name) = item else {
        return SetTileItemsOutcome::rejected(ItemSyncRejectReason::MissingItem);
    };
    let Some(item_id) = resolve_item_id(&item_name) else {
        return SetTileItemsOutcome::rejected(ItemSyncRejectReason::UnknownItem);
    };

    let mut applied_positions = 0;
    for position in &positions {
        if let Some(build) = buildings
            .iter_mut()
            .find(|build| build.tile_pos == *position)
        {
            if let Some(items) = build.items.as_mut() {
                items.set(item_id, amount);
                applied_positions += 1;
            }
        }
    }

    SetTileItemsOutcome {
        accepted: true,
        rejection: None,
        applied_positions,
        packet: Some(SetTileItemsCallPacket {
            item: Some(item_name),
            amount,
            positions,
        }),
    }
}

pub fn set_tile_liquids<R>(
    buildings: &mut [BuildingComp],
    liquid: Option<String>,
    amount: f32,
    positions: Vec<i32>,
    resolve_liquid_id: R,
) -> SetTileLiquidsOutcome
where
    R: FnOnce(&str) -> Option<i16>,
{
    let Some(liquid_name) = liquid else {
        return SetTileLiquidsOutcome::rejected(LiquidSyncRejectReason::MissingLiquid);
    };
    let Some(liquid_id) = resolve_liquid_id(&liquid_name) else {
        return SetTileLiquidsOutcome::rejected(LiquidSyncRejectReason::UnknownLiquid);
    };

    let mut applied_positions = 0;
    for position in &positions {
        if let Some(build) = buildings
            .iter_mut()
            .find(|build| build.tile_pos == *position)
        {
            if let Some(liquids) = build.liquids.as_mut() {
                liquids.set(liquid_id, amount);
                applied_positions += 1;
            }
        }
    }

    SetTileLiquidsOutcome {
        accepted: true,
        rejection: None,
        applied_positions,
        packet: Some(SetTileLiquidsCallPacket {
            liquid: Some(liquid_name),
            amount,
            positions,
        }),
    }
}

pub fn transfer_item_to<R>(
    unit: Option<&mut UnitComp>,
    item: Option<String>,
    amount: i32,
    x: f32,
    y: f32,
    build: Option<&mut BuildingComp>,
    resolve_item_id: R,
) -> TransferItemToOutcome
where
    R: FnOnce(&str) -> Option<i16>,
{
    let Some(build) = build else {
        return TransferItemToOutcome::rejected(TransferItemToRejectReason::MissingBuild, amount);
    };
    let build_ref = BuildingRef::new(build.tile_pos);

    if build.items.is_none() {
        return TransferItemToOutcome::rejected(
            TransferItemToRejectReason::MissingItemStorage,
            amount,
        );
    }

    let Some(item_name) = item else {
        return TransferItemToOutcome::rejected(TransferItemToRejectReason::MissingItem, amount);
    };
    let Some(item_id) = resolve_item_id(&item_name) else {
        return TransferItemToOutcome::rejected(TransferItemToRejectReason::UnknownItem, amount);
    };

    let items = build.items.as_mut().expect("checked item module presence");

    let building_previous_amount = items.get(item_id);
    let mut unit_ref = UnitRef::Null;
    let mut unit_previous_amount = None;
    let mut unit_new_amount = None;

    if let Some(unit) = unit {
        unit_ref = UnitRef::Unit { id: unit.id() };
        if unit.items.item() == Some(item_name.as_str()) {
            let previous = unit.items.stack.amount;
            let next = (previous - amount).max(0);
            unit.items.stack.amount = next;
            unit_previous_amount = Some(previous);
            unit_new_amount = Some(next);
        }
    }

    if amount > 0 {
        items.add(item_id, amount);
    }
    let building_new_amount = items.get(item_id);

    TransferItemToOutcome {
        accepted: true,
        rejection: None,
        requested_amount: amount,
        unit_previous_amount,
        unit_new_amount,
        building_previous_amount,
        building_new_amount,
        effect_count: item_transfer_effect_count(amount),
        packet: Some(TransferItemToCallPacket {
            unit: unit_ref,
            item: Some(item_name),
            amount,
            x,
            y,
            build: build_ref,
        }),
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
    let displayed_text = PlayerComp::normalized_ping_text(&text, context.max_text_len);

    if context.same_team_visible {
        if let Some(player) = player {
            player.apply_ping_location(x, y, &text, context.max_text_len);
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

pub fn delete_team_plans<A>(
    context: DeletePlansContext,
    player_present: bool,
    teams: &mut Teams,
    team: u8,
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

    let removed = teams.delete_plans_at_positions(team, positions).len();

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

pub fn apply_removed_team_plan_positions(
    teams: &mut Teams,
    team: u8,
    positions: &[i32],
) -> Vec<TeamBlockPlan> {
    teams.delete_plans_at_positions(team, positions)
}

pub fn client_delete_plans_packet(positions: Vec<i32>) -> DeletePlansCallPacket {
    DeletePlansCallPacket {
        player_id: None,
        positions,
    }
}

pub fn command_units<A>(
    context: CommandUnitsContext,
    unit_ids: Vec<i32>,
    build_target: Option<&BuildingComp>,
    unit_target: Option<UnitRef>,
    pos_target: Vec2,
    queue_command: bool,
    final_batch: bool,
    admin_allows: A,
) -> CommandUnitsOutcome
where
    A: FnOnce(&[i32]) -> bool,
{
    if context.player.is_none() {
        return CommandUnitsOutcome::rejected(
            &context,
            CommandUnitsRejectReason::MissingPlayer,
            false,
        );
    }

    if unit_ids.is_empty() {
        return CommandUnitsOutcome::rejected(
            &context,
            CommandUnitsRejectReason::MissingUnits,
            false,
        );
    }

    if !admin_allows(&unit_ids) {
        return CommandUnitsOutcome::rejected(
            &context,
            CommandUnitsRejectReason::AdminDenied,
            true,
        );
    }

    let commanded = unit_ids.len();
    CommandUnitsOutcome {
        accepted: true,
        rejection: None,
        commanded,
        packet: Some(CommandUnitsCallPacket {
            player: context.player.unwrap_or_else(EntityRef::null),
            unit_ids,
            build_target: build_target
                .map(|build| BuildingRef::new(build.tile_pos))
                .unwrap_or_else(BuildingRef::null),
            unit_target: unit_target.unwrap_or(UnitRef::Null),
            pos_target,
            queue_command,
            final_batch,
        }),
        should_raise_validate: false,
    }
}

pub fn client_command_units_packet(
    unit_ids: Vec<i32>,
    build_target: Option<&BuildingComp>,
    unit_target: Option<UnitRef>,
    pos_target: Vec2,
    queue_command: bool,
    final_batch: bool,
) -> CommandUnitsCallPacket {
    CommandUnitsCallPacket {
        player: EntityRef::null(),
        unit_ids,
        build_target: build_target
            .map(|build| BuildingRef::new(build.tile_pos))
            .unwrap_or_else(BuildingRef::null),
        unit_target: unit_target.unwrap_or(UnitRef::Null),
        pos_target,
        queue_command,
        final_batch,
    }
}

fn command_wire_mut(unit: &mut UnitComp) -> Option<&mut CommandWire> {
    match &mut unit.controller {
        UnitControllerState::Command(command) => Some(command),
        _ => None,
    }
}

pub fn set_unit_command<A>(
    context: SetUnitCommandContext,
    player: Option<&PlayerComp>,
    units: &mut [UnitComp],
    unit_ids: Vec<i32>,
    command: Option<&UnitCommand>,
    admin_allows: A,
) -> SetUnitCommandOutcome
where
    A: FnOnce(&[i32]) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return SetUnitCommandOutcome::rejected(
            &context,
            SetUnitCommandRejectReason::MissingPlayer,
            false,
        );
    }
    let player = player.unwrap();

    if unit_ids.is_empty() {
        return SetUnitCommandOutcome::rejected(
            &context,
            SetUnitCommandRejectReason::MissingUnits,
            false,
        );
    }

    let Some(command) = command else {
        return SetUnitCommandOutcome::rejected(
            &context,
            SetUnitCommandRejectReason::MissingCommand,
            false,
        );
    };

    if !admin_allows(&unit_ids) {
        return SetUnitCommandOutcome::rejected(
            &context,
            SetUnitCommandRejectReason::AdminDenied,
            true,
        );
    }

    let command_name = command.name().to_string();
    let command_id = command.id();
    let colored_name = player.colored_name();
    let mut commanded = 0usize;

    for id in &unit_ids {
        let Some(unit) = units.iter_mut().find(|unit| unit.id() == *id) else {
            continue;
        };
        if unit.team.team != player.team
            || !unit
                .type_info
                .commands
                .iter()
                .any(|allowed| allowed == &command_name)
        {
            continue;
        }

        let applied = if let Some(command_wire) = command_wire_mut(unit) {
            command_wire.command_id = Some(command_id);
            if command.reset_target {
                command_wire.target_pos = None;
                command_wire.attack_target = None;
            }
            true
        } else {
            false
        };

        if applied {
            unit.last_commanded = Some(colored_name.clone());
            commanded += 1;
        }
    }

    SetUnitCommandOutcome {
        accepted: true,
        rejection: None,
        commanded,
        packet: Some(SetUnitCommandCallPacket {
            player: context.player.unwrap_or_else(EntityRef::null),
            unit_ids,
            command: command_name,
        }),
        should_raise_validate: false,
    }
}

pub fn client_set_unit_command_packet(
    unit_ids: Vec<i32>,
    command: impl Into<String>,
) -> SetUnitCommandCallPacket {
    SetUnitCommandCallPacket {
        player: EntityRef::null(),
        unit_ids,
        command: command.into(),
    }
}

pub fn set_unit_stance<A>(
    context: SetUnitStanceContext,
    player: Option<&PlayerComp>,
    units: &mut [UnitComp],
    unit_ids: Vec<i32>,
    stance: Option<&UnitStance>,
    enable: bool,
    admin_allows: A,
) -> SetUnitStanceOutcome
where
    A: FnOnce(&[i32]) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return SetUnitStanceOutcome::rejected(
            &context,
            SetUnitStanceRejectReason::MissingPlayer,
            false,
        );
    }
    let player = player.unwrap();

    if unit_ids.is_empty() {
        return SetUnitStanceOutcome::rejected(
            &context,
            SetUnitStanceRejectReason::MissingUnits,
            false,
        );
    }

    let Some(stance) = stance else {
        return SetUnitStanceOutcome::rejected(
            &context,
            SetUnitStanceRejectReason::MissingStance,
            false,
        );
    };

    if !admin_allows(&unit_ids) {
        return SetUnitStanceOutcome::rejected(
            &context,
            SetUnitStanceRejectReason::AdminDenied,
            true,
        );
    }

    let stance_name = stance.name().to_string();
    let stance_id = stance.id();
    let colored_name = player.colored_name();
    let mut commanded = 0usize;

    for id in &unit_ids {
        let Some(unit) = units.iter_mut().find(|unit| unit.id() == *id) else {
            continue;
        };
        if unit.team.team != player.team {
            continue;
        }

        let allowed_stance = stance_name == "stop"
            || unit
                .type_info
                .stances
                .iter()
                .any(|allowed| allowed == &stance_name);

        if !allowed_stance {
            continue;
        }

        let applied = if let Some(command_wire) = command_wire_mut(unit) {
            if stance_name == "stop" {
                command_wire.target_pos = None;
                command_wire.attack_target = None;
                command_wire.command_queue.clear();
                command_wire.stances.clear();
            } else if !stance.toggle || enable {
                if !command_wire.stances.contains(&stance_id) {
                    command_wire.stances.push(stance_id);
                }
            } else {
                command_wire.stances.retain(|id| *id != stance_id);
            }
            true
        } else {
            false
        };

        if applied {
            unit.last_commanded = Some(colored_name.clone());
            commanded += 1;
        }
    }

    SetUnitStanceOutcome {
        accepted: true,
        rejection: None,
        commanded,
        packet: Some(SetUnitStanceCallPacket {
            player: context.player.unwrap_or_else(EntityRef::null),
            unit_ids,
            stance: stance_name,
            enable,
        }),
        should_raise_validate: false,
    }
}

pub fn client_set_unit_stance_packet(
    unit_ids: Vec<i32>,
    stance: impl Into<String>,
    enable: bool,
) -> SetUnitStanceCallPacket {
    SetUnitStanceCallPacket {
        player: EntityRef::null(),
        unit_ids,
        stance: stance.into(),
        enable,
    }
}

pub fn command_building<A>(
    context: CommandBuildingContext,
    player: Option<&PlayerComp>,
    builds: &mut [BuildingComp],
    buildings: Vec<i32>,
    target: Vec2,
    admin_allows: A,
) -> CommandBuildingOutcome
where
    A: FnOnce(&[i32]) -> bool,
{
    if context.player.is_none() || player.is_none() {
        return CommandBuildingOutcome::rejected(
            &context,
            CommandBuildingRejectReason::MissingPlayer,
            false,
        );
    }
    let player = player.unwrap();

    if buildings.is_empty() {
        return CommandBuildingOutcome::rejected(
            &context,
            CommandBuildingRejectReason::MissingBuildings,
            false,
        );
    }

    if !admin_allows(&buildings) {
        return CommandBuildingOutcome::rejected(
            &context,
            CommandBuildingRejectReason::AdminDenied,
            true,
        );
    }

    let colored_name = player.colored_name();
    let mut commanded_positions = Vec::new();

    for pos in &buildings {
        let Some(build) = builds.iter_mut().find(|build| build.tile_pos == *pos) else {
            continue;
        };
        if build.team != player.team || !build.block.commandable {
            continue;
        }

        build.last_accessed = colored_name.clone();
        commanded_positions.push(build.tile_pos);
    }

    CommandBuildingOutcome {
        accepted: true,
        rejection: None,
        commanded_positions,
        packet: Some(CommandBuildingCallPacket {
            player: context.player.unwrap_or_else(EntityRef::null),
            buildings,
            target,
        }),
        should_raise_validate: false,
    }
}

pub fn client_command_building_packet(
    buildings: Vec<i32>,
    target: Vec2,
) -> CommandBuildingCallPacket {
    CommandBuildingCallPacket {
        player: EntityRef::null(),
        buildings,
        target,
    }
}

pub fn remove_queue_block(
    unit: Option<&mut UnitComp>,
    x: i32,
    y: i32,
    breaking: bool,
) -> RemoveQueueBlockOutcome {
    let Some(unit) = unit else {
        return RemoveQueueBlockOutcome::rejected(RemoveQueueBlockRejectReason::MissingUnit);
    };

    let removed = unit.builder.remove_build(x, y, breaking);

    RemoveQueueBlockOutcome {
        accepted: true,
        rejection: None,
        removed,
        packet: Some(RemoveQueueBlockCallPacket { x, y, breaking }),
    }
}

pub fn remove_queue_block_packet(x: i32, y: i32, breaking: bool) -> RemoveQueueBlockCallPacket {
    RemoveQueueBlockCallPacket { x, y, breaking }
}

pub fn rotate_build_plans<F>(
    plans: &[BuildPlan],
    origin_x: i32,
    origin_y: i32,
    direction: i32,
    mut block_info: F,
) -> Vec<BuildPlan>
where
    F: FnMut(&BuildPlan) -> Option<BuildPlanBlockTransform>,
{
    plans
        .iter()
        .map(|plan| {
            let Some(info) = block_info(plan) else {
                return plan.clone();
            };
            if plan.breaking || plan.block.is_none() {
                return plan.clone();
            }

            let mut rotated = plan.clone();
            rotated.config = transform_point_config(&rotated.config, |point| {
                rotate_config_point(point, direction, info.size)
            });

            let mut world_x = (plan.x - origin_x) as f32 * TILE_SIZE as f32 + info.offset;
            let mut world_y = (plan.y - origin_y) as f32 * TILE_SIZE as f32 + info.offset;
            let old_x = world_x;
            if direction >= 0 {
                world_x = -world_y;
                world_y = old_x;
            } else {
                world_x = world_y;
                world_y = -old_x;
            }

            rotated.x = world_to_tile_runtime(world_x - info.offset) + origin_x;
            rotated.y = world_to_tile_runtime(world_y - info.offset) + origin_y;
            rotated.rotation = info.plan_rotation(plan.rotation + direction);
            rotated
        })
        .collect()
}

pub fn flip_build_plans<F>(
    plans: &[BuildPlan],
    origin_x: i32,
    origin_y: i32,
    flip_x: bool,
    mut block_info: F,
) -> Vec<BuildPlan>
where
    F: FnMut(&BuildPlan) -> Option<BuildPlanBlockTransform>,
{
    let origin = if flip_x { origin_x } else { origin_y } * TILE_SIZE;
    plans
        .iter()
        .map(|plan| {
            let Some(info) = block_info(plan) else {
                return plan.clone();
            };
            if plan.breaking || plan.block.is_none() {
                return plan.clone();
            }

            let mut flipped = plan.clone();
            let coord = if flip_x { plan.x } else { plan.y };
            let value = -((coord * TILE_SIZE) as f32 - origin as f32 + info.offset) + origin as f32;

            if flip_x {
                flipped.x = ((value - info.offset) / TILE_SIZE as f32) as i32;
            } else {
                flipped.y = ((value - info.offset) / TILE_SIZE as f32) as i32;
            }

            flipped.config = transform_point_config(&flipped.config, |point| {
                flip_config_point(point, flip_x, info.size)
            });
            flipped.rotation = info.flip_rotation(plan.rotation, flip_x);
            flipped
        })
        .collect()
}

pub fn build_bounds_for_plan(x: i32, y: i32, footprint: BuildPlanBlockTransform) -> BuildBounds {
    placement_bounds(x, y, footprint.size, footprint.offset)
}

pub fn build_bounds_overlap(left: BuildBounds, right: BuildBounds) -> bool {
    left.x < right.x + right.width
        && left.x + left.width > right.x
        && left.y < right.y + right.height
        && left.y + left.height > right.y
}

pub fn valid_place_plan(frame: ValidPlaceFrame) -> bool {
    if !frame.base_valid {
        return false;
    }

    if !frame.player_is_builder || frame.queued_plans.is_empty() {
        return true;
    }

    let candidate = build_bounds_for_plan(frame.x, frame.y, frame.block);
    frame.queued_plans.iter().all(|snapshot| {
        if snapshot.ignored || snapshot.plan.breaking {
            return true;
        }

        if snapshot.candidate_can_replace
            && snapshot.plan.x == frame.x
            && snapshot.plan.y == frame.y
        {
            return true;
        }

        let queued = build_bounds_for_plan(snapshot.plan.x, snapshot.plan.y, snapshot.footprint);
        !build_bounds_overlap(candidate, queued)
    })
}

pub fn valid_break_plan(base_valid_break: bool) -> bool {
    base_valid_break
}

pub fn break_block_plan(frame: BreakBlockFrame) -> Option<BuildPlan> {
    if !frame.player_is_builder {
        return None;
    }
    let tile = frame.tile?;
    let (x, y) = tile.target();
    Some(BuildPlan::new_break(x, y))
}

pub fn try_break_block_plan(frame: TryBreakBlockFrame) -> Option<BuildPlan> {
    valid_break_plan(frame.valid_break).then(|| break_block_plan(frame.break_frame))?
}

pub fn rebuild_area_plan(frame: RebuildAreaFrame) -> RebuildAreaPlan {
    let result = super::placement::normalize_area(
        frame.x1,
        frame.y1,
        frame.x2,
        frame.y2,
        frame.rotation,
        false,
        frame.max_length,
    );
    let selection = RebuildAreaSelection {
        x: result.x,
        y: result.y,
        x2: result.x2,
        y2: result.y2,
        rotation: result.rotation,
    };
    let selection_bounds = BuildBounds {
        x: result.x as f32 * TILE_SIZE as f32,
        y: result.y as f32 * TILE_SIZE as f32,
        width: (result.x2 - result.x) as f32 * TILE_SIZE as f32,
        height: (result.y2 - result.y) as f32 * TILE_SIZE as f32,
    };

    let rebuild_plans = frame
        .broken_plans
        .iter()
        .filter(|plan| {
            build_bounds_overlap(
                build_bounds_for_plan(plan.x, plan.y, plan.footprint),
                selection_bounds,
            )
        })
        .map(RebuildBlockPlanSnapshot::to_build_plan)
        .collect();

    let mut seen = std::collections::BTreeSet::new();
    let mut repair_plans = Vec::new();
    let mut repair_tile_positions = Vec::new();
    for candidate in frame.repair_candidates {
        if candidate.scan_x < result.x
            || candidate.scan_x > result.x2
            || candidate.scan_y < result.y
            || candidate.scan_y > result.y2
            || !candidate.can_repair
            || !seen.insert(candidate.tile_pos)
        {
            continue;
        }
        repair_tile_positions.push(candidate.tile_pos);
        repair_plans.push(candidate.plan);
    }

    RebuildAreaPlan {
        selection,
        rebuild_plans,
        repair_plans,
        repair_tile_positions,
    }
}

fn plan_candidate_valid(candidate: &FlushBuildPlanCandidate) -> bool {
    candidate.valid_place && candidate.plan.block.is_some()
}

pub fn flush_build_plans_plan(
    plans: &[FlushBuildPlanCandidate],
    reverse: bool,
) -> Vec<FlushBuildPlanAction> {
    let mut actions = Vec::new();
    if reverse {
        for candidate in plans.iter().rev() {
            if plan_candidate_valid(candidate) {
                actions.push(FlushBuildPlanAction {
                    plan: candidate.plan.copy(),
                    position: BuildQueuePosition::Head,
                    call_on_new_plan: true,
                    insert_into_plan_tree: true,
                });
            }
        }
    } else {
        for candidate in plans {
            if plan_candidate_valid(candidate) {
                actions.push(FlushBuildPlanAction {
                    plan: candidate.plan.copy(),
                    position: BuildQueuePosition::Tail,
                    call_on_new_plan: true,
                    insert_into_plan_tree: true,
                });
            }
        }
    }
    actions
}

fn get_plan_snapshot_index(
    unit_plans: &[BuildPlanSnapshot],
    select_plans: &[BuildPlanSnapshot],
    x: i32,
    y: i32,
    size: i32,
    skip_select_index: Option<usize>,
) -> Option<(bool, usize)> {
    let offset = ((size + 1) % 2) as f32 * TILE_SIZE as f32 / 2.0;
    let query = BuildBounds {
        x: x as f32 * TILE_SIZE as f32 + offset - size as f32 * TILE_SIZE as f32 / 2.0,
        y: y as f32 * TILE_SIZE as f32 + offset - size as f32 * TILE_SIZE as f32 / 2.0,
        width: size as f32 * TILE_SIZE as f32,
        height: size as f32 * TILE_SIZE as f32,
    };

    if let Some(index) = unit_plans.iter().position(|snapshot| {
        snapshot.tile_pos.is_some()
            && build_bounds_overlap(
                query,
                build_bounds_for_plan(snapshot.plan.x, snapshot.plan.y, snapshot.footprint),
            )
    }) {
        return Some((false, index));
    }

    select_plans
        .iter()
        .enumerate()
        .find(|(index, snapshot)| {
            Some(*index) != skip_select_index
                && snapshot.tile_pos.is_some()
                && build_bounds_overlap(
                    query,
                    build_bounds_for_plan(snapshot.plan.x, snapshot.plan.y, snapshot.footprint),
                )
        })
        .map(|(index, _)| (true, index))
}

pub fn flush_select_plans_plan(
    plans: &[FlushBuildPlanCandidate],
    unit_plans: &[BuildPlanSnapshot],
    select_plans: &[BuildPlanSnapshot],
) -> FlushSelectPlansPlan {
    let mut actions = Vec::new();
    let mut final_select: Vec<BuildPlanSnapshot> = select_plans.to_vec();
    let mut skipped = 0;

    for candidate in plans {
        if !plan_candidate_valid(candidate) {
            skipped += 1;
            continue;
        }

        let Some(block) = candidate.plan.block.as_ref() else {
            skipped += 1;
            continue;
        };

        let other = get_plan_snapshot_index(
            unit_plans,
            &final_select,
            candidate.plan.x,
            candidate.plan.y,
            candidate.footprint.size,
            None,
        );

        match other {
            None => {
                let plan = candidate.plan.copy();
                final_select.push(BuildPlanSnapshot::new(plan.clone(), candidate.footprint));
                actions.push(FlushSelectPlanAction::Add { plan });
            }
            Some((true, index)) => {
                let other = &final_select[index].plan;
                if !other.breaking
                    && other.x == candidate.plan.x
                    && other.y == candidate.plan.y
                    && final_select[index].footprint.size == candidate.footprint.size
                {
                    let plan = BuildPlan {
                        block: Some(block.clone()),
                        ..candidate.plan.copy()
                    };
                    final_select.remove(index);
                    final_select.push(BuildPlanSnapshot::new(plan.clone(), candidate.footprint));
                    actions.push(FlushSelectPlanAction::ReplaceSelect { index, plan });
                } else {
                    skipped += 1;
                }
            }
            Some((false, _)) => {
                skipped += 1;
            }
        }
    }

    FlushSelectPlansPlan {
        actions,
        final_select_plans: final_select
            .into_iter()
            .map(|snapshot| snapshot.plan)
            .collect(),
        skipped,
    }
}

fn remove_selection_scan_positions(
    frame: &RemoveSelectionFrame,
    selection: RebuildAreaSelection,
) -> Vec<(i32, i32)> {
    let x_sign = (frame.x2 - frame.x1).signum();
    let y_sign = (frame.y2 - frame.y1).signum();
    let x_sign = if x_sign == 0 { 0 } else { x_sign };
    let y_sign = if y_sign == 0 { 0 } else { y_sign };
    let mut positions = Vec::new();
    for x in 0..=(selection.x2 - selection.x).abs() {
        for y in 0..=(selection.y2 - selection.y).abs() {
            positions.push((frame.x1 + x * x_sign, frame.y1 + y * y_sign));
        }
    }
    positions
}

pub fn remove_selection_plan(frame: RemoveSelectionFrame) -> RemoveSelectionPlan {
    let result = super::placement::normalize_area(
        frame.x1,
        frame.y1,
        frame.x2,
        frame.y2,
        frame.rotation,
        false,
        frame.max_length,
    );
    let selection = RebuildAreaSelection {
        x: result.x,
        y: result.y,
        x2: result.x2,
        y2: result.y2,
        rotation: result.rotation,
    };
    let selection_bounds = BuildBounds {
        x: result.x as f32 * TILE_SIZE as f32,
        y: result.y as f32 * TILE_SIZE as f32,
        width: (result.x2 - result.x) as f32 * TILE_SIZE as f32,
        height: (result.y2 - result.y) as f32 * TILE_SIZE as f32,
    };
    let scan_positions = remove_selection_scan_positions(&frame, selection);

    let mut immediate_break_plans = Vec::new();
    let mut queued_break_plans = Vec::new();
    let mut queued_break_tile_positions = std::collections::BTreeSet::new();

    for (scan_x, scan_y) in scan_positions {
        let Some(tile) = frame
            .world_tiles
            .iter()
            .find(|tile| tile.scan_x == scan_x && tile.scan_y == scan_y)
        else {
            continue;
        };

        if !frame.flush {
            if tile.valid_break {
                if let Some(plan) = break_block_plan(BreakBlockFrame {
                    player_is_builder: frame.player_is_builder,
                    tile: Some(tile.break_tile),
                }) {
                    immediate_break_plans.push(plan);
                }
            }
        } else if tile.valid_break
            && !frame
                .select_plans
                .iter()
                .any(|snapshot| snapshot.tile_pos == Some(tile.tile_pos))
            && queued_break_tile_positions.insert(tile.tile_pos)
        {
            queued_break_plans.push(BuildPlan::new_break(tile.tile_x, tile.tile_y));
        }
    }

    let mut remove_unit_plan_indices = Vec::new();
    let mut remove_select_plan_indices = Vec::new();
    let mut remove_team_plan_indices = Vec::new();
    let mut removed_team_plan_positions = Vec::new();

    if !frame.player_dead {
        remove_unit_plan_indices.extend(
            frame
                .unit_plans
                .iter()
                .enumerate()
                .filter(|(_, snapshot)| {
                    !snapshot.plan.breaking
                        && build_bounds_overlap(
                            build_bounds_for_plan(
                                snapshot.plan.x,
                                snapshot.plan.y,
                                snapshot.footprint,
                            ),
                            selection_bounds,
                        )
                })
                .map(|(index, _)| index),
        );

        if frame.flush {
            remove_select_plan_indices.extend(
                frame
                    .select_plans
                    .iter()
                    .enumerate()
                    .filter(|(_, snapshot)| {
                        !snapshot.plan.breaking
                            && build_bounds_overlap(
                                build_bounds_for_plan(
                                    snapshot.plan.x,
                                    snapshot.plan.y,
                                    snapshot.footprint,
                                ),
                                selection_bounds,
                            )
                    })
                    .map(|(index, _)| index),
            );
        }
    }

    for (index, plan) in frame.team_plans.iter().enumerate() {
        if build_bounds_overlap(
            build_bounds_for_plan(plan.x, plan.y, plan.footprint),
            selection_bounds,
        ) {
            remove_team_plan_indices.push(index);
            removed_team_plan_positions.push(point2_pack(plan.x, plan.y));
        }
    }

    let network_delete_positions = if frame.net_active && !removed_team_plan_positions.is_empty() {
        removed_team_plan_positions.clone()
    } else {
        Vec::new()
    };

    RemoveSelectionPlan {
        selection,
        immediate_break_plans,
        queued_break_plans,
        remove_unit_plan_indices,
        remove_select_plan_indices,
        remove_team_plan_indices,
        removed_team_plan_positions,
        network_delete_positions,
    }
}

fn line_angle_degrees(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> f32 {
    ((end_y - start_y) as f32)
        .atan2((end_x - start_x) as f32)
        .to_degrees()
        .rem_euclid(360.0)
}

fn line_base_rotation(start_x: i32, start_y: i32, end_x: i32, end_y: i32, rotation: i32) -> i32 {
    if start_x == end_x && start_y == end_y {
        rotation
    } else {
        (((line_angle_degrees(start_x, start_y, end_x, end_y) + 45.0) / 90.0) as i32).rem_euclid(4)
    }
}

pub fn iterate_line_plan(mut frame: IterateLineFrame) -> IterateLinePlan {
    let block = frame.block.clone();
    let mut diagonal = frame.diagonal_pressed;
    if frame.swap_diagonal_setting && frame.mobile {
        diagonal = !diagonal;
    }
    if block
        .as_ref()
        .is_some_and(|block| block.swap_diagonal_placement)
    {
        diagonal = !diagonal;
    }

    let diagonal_allowed = diagonal
        && block
            .as_ref()
            .map(|block| block.allow_diagonal)
            .unwrap_or(true);

    let mut points = if diagonal_allowed {
        if block.as_ref().is_some_and(|block| {
            frame.start_build.chained
                && frame.end_build.chained
                && frame.start_build.candidate_can_replace
                && frame.end_build.candidate_can_replace
                && block.allow_diagonal
        }) {
            frame.upgrade_path.take().unwrap_or_else(|| {
                super::placement::pathfind_line(
                    true,
                    frame.conveyor_pathfinding,
                    frame.astar_path.clone(),
                    frame.start_x,
                    frame.start_y,
                    frame.end_x,
                    frame.end_y,
                )
            })
        } else {
            super::placement::pathfind_line(
                block.as_ref().is_some_and(|block| block.conveyor_placement),
                frame.conveyor_pathfinding,
                frame.astar_path.clone(),
                frame.start_x,
                frame.start_y,
                frame.end_x,
                frame.end_y,
            )
        }
    } else if block
        .as_ref()
        .is_some_and(|block| block.allow_rectangle_placement)
    {
        super::placement::normalize_rectangle(
            frame.start_x,
            frame.start_y,
            frame.end_x,
            frame.end_y,
            block.as_ref().unwrap().footprint.size,
        )
    } else {
        super::placement::normalize_line(frame.start_x, frame.start_y, frame.end_x, frame.end_y)
    };

    if let Some(changed) = frame.changed_points.take() {
        points = changed;
    }

    let end_rotation =
        if points.len() > 1 && frame.end_build.chained && !frame.second_to_last_chained {
            Some(frame.end_build.rotation)
        } else {
            None
        };

    let mut base_rotation = frame.rotation;
    if !frame.override_line_rotation || diagonal {
        base_rotation = line_base_rotation(
            frame.start_x,
            frame.start_y,
            frame.end_x,
            frame.end_y,
            frame.rotation,
        );
    }

    let mut previous_bounds: Option<BuildBounds> = None;
    let mut lines = Vec::new();
    for (index, point) in points.iter().copied().enumerate() {
        if let Some(block) = block.as_ref() {
            let bounds = build_bounds_for_plan(point.x, point.y, block.footprint);
            if previous_bounds.is_some_and(|prev| build_bounds_overlap(bounds, prev)) {
                continue;
            }
        }

        let next = points.get(index + 1).copied();
        let mut line_rotation = if let Some(block) = block.as_ref() {
            if (!frame.override_line_rotation || diagonal)
                && !(block.ignore_line_rotation && !frame.mobile)
            {
                let mut result = base_rotation;
                if let Some(next) = next {
                    result = crate::mindustry::world::tile::relative_to(
                        point.x, point.y, next.x, next.y,
                    ) as i32;
                } else if let Some(end_rotation) = end_rotation {
                    result = end_rotation;
                } else if block.conveyor_placement && index > 0 {
                    let prev = points[index - 1];
                    result = crate::mindustry::world::tile::relative_to(
                        prev.x, prev.y, point.x, point.y,
                    ) as i32;
                }
                if result == -1 {
                    frame.rotation
                } else {
                    result
                }
            } else {
                frame.rotation
            }
        } else {
            base_rotation
        };
        if line_rotation == -1 {
            line_rotation = frame.rotation;
        }

        lines.push(PlaceLine {
            x: point.x,
            y: point.y,
            rotation: line_rotation,
            last: next.is_none(),
        });

        if let Some(block) = block.as_ref() {
            previous_bounds = Some(build_bounds_for_plan(point.x, point.y, block.footprint));
        }
    }

    IterateLinePlan {
        diagonal,
        base_rotation,
        end_rotation,
        points,
        lines,
    }
}

pub fn update_line_plan(frame: UpdateLineFrame) -> UpdateLinePlan {
    let Some(block) = frame.line.block.clone() else {
        let line = iterate_line_plan(frame.line);
        return UpdateLinePlan {
            final_rotation: line.lines.last().map(|line| line.rotation).unwrap_or(0),
            line,
            line_plans: Vec::new(),
            handle_placement_line: false,
        };
    };

    let line = iterate_line_plan(frame.line);
    let mut line_plans: Vec<BuildPlan> = line
        .lines
        .iter()
        .map(|line| {
            let mut plan = BuildPlan::new_config(
                line.x,
                line.y,
                line.rotation,
                block.name.clone(),
                frame.next_config.clone(),
            );
            plan.anim_scale = 1.0;
            plan
        })
        .collect();

    if frame.block_replace {
        for plan in &mut line_plans {
            if let Some(replacement) = frame
                .replacements
                .iter()
                .find(|replacement| replacement.x == plan.x && replacement.y == plan.y)
            {
                if replacement.unlocked {
                    plan.block = Some(replacement.block.clone());
                }
            }
        }
    }

    UpdateLinePlan {
        final_rotation: line
            .lines
            .last()
            .map(|line| line.rotation)
            .unwrap_or(block.footprint.plan_rotation(0)),
        line,
        line_plans,
        handle_placement_line: frame.block_replace,
    }
}

pub fn can_tap_player_plan(frame: TapPlayerFrame) -> bool {
    frame.within_select_range
        && !frame.player_dead
        && frame.stack_amount > 0
        && !frame.block_selected
}

pub fn try_tap_player_plan(frame: TapPlayerFrame) -> TapPlayerPlan {
    let accepted = can_tap_player_plan(frame);
    TapPlayerPlan {
        accepted,
        dropping_item: accepted,
    }
}

pub fn can_mine_plan(frame: CanMineFrame) -> bool {
    !frame.scene_has_mouse
        && !frame.player_dead
        && frame.unit_valid_mine
        && frame.unit_accepts_mine_result
        && !((!frame.double_tap_mine && frame.floor_player_unmineable)
            && !frame.overlay_has_item_drop)
        && !((!frame.double_tap_mine && frame.overlay_player_unmineable)
            && frame.overlay_has_item_drop)
}

pub fn try_begin_mine_plan(frame: BeginMineFrame) -> BeginMinePlan {
    let accepted = !frame.player_dead && frame.can_mine && frame.tile_pos.is_some();
    BeginMinePlan {
        accepted,
        mine_tile: accepted.then_some(frame.tile_pos).flatten(),
    }
}

pub fn try_stop_mine_plan(frame: StopMineFrame) -> StopMinePlan {
    let accepted = if frame.player_dead {
        false
    } else if let Some(requested) = frame.requested_tile {
        frame.current_mine_tile == Some(requested)
    } else {
        frame.current_mine_tile.is_some()
    };

    StopMinePlan {
        accepted,
        clear_mine_tile: accepted,
    }
}

pub fn can_repair_derelict_plan(frame: &RepairDerelictFrame) -> bool {
    frame.tile_present
        && frame.build_present
        && !frame.player_dead
        && !frame.editor
        && !frame.player_team_derelict
        && frame.build_team_derelict
        && frame.block_unlocked_host
        && frame.valid_place
        && frame.block.is_some()
}

pub fn try_repair_derelict_plan(frame: RepairDerelictFrame) -> RepairDerelictPlan {
    if !can_repair_derelict_plan(&frame) {
        return RepairDerelictPlan {
            accepted: false,
            build_plan: None,
        };
    }

    RepairDerelictPlan {
        accepted: true,
        build_plan: Some(BuildPlan::new_config(
            frame.build_x,
            frame.build_y,
            frame.build_rotation,
            frame.block.unwrap_or_default(),
            frame.config,
        )),
    }
}

pub fn selected_unit_plan(
    mouse_x: f32,
    mouse_y: f32,
    player_unit_id: Option<i32>,
    nearby_units: &[SelectUnitCandidate],
    world_build: Option<SelectBuildingCandidate>,
) -> Option<SelectedUnitPlan> {
    if let Some(unit) = nearby_units
        .iter()
        .copied()
        .filter(|unit| unit.is_ai && unit.player_controllable)
        .filter(|unit| unit.dst_edge(mouse_x, mouse_y) <= 40.0)
        .filter(|unit| {
            let grown = unit.hit_size / 2.0 + 6.0;
            (unit.x - mouse_x).abs() <= grown && (unit.y - mouse_y).abs() <= grown
        })
        .min_by(|a, b| {
            a.dst_edge(mouse_x, mouse_y)
                .total_cmp(&b.dst_edge(mouse_x, mouse_y))
        })
    {
        return Some(SelectedUnitPlan {
            unit_id: unit.id,
            source: SelectedUnitSource::NearbyUnit,
        });
    }

    let build = world_build?;
    if build.control_block
        && build.can_control
        && build.same_team
        && build.controlled_unit_id != player_unit_id
        && build.controlled_unit_is_ai
    {
        build.controlled_unit_id.map(|unit_id| SelectedUnitPlan {
            unit_id,
            source: SelectedUnitSource::ControlBlock,
        })
    } else {
        None
    }
}

pub fn selected_control_build_plan(
    player_dead: bool,
    build: Option<SelectBuildingCandidate>,
) -> Option<i32> {
    let build = build?;
    (!player_dead && build.can_control_select && build.same_team).then_some(build.id)
}

pub fn selected_command_unit_plan(x: f32, y: f32, units: &[SelectUnitCandidate]) -> Option<i32> {
    units
        .iter()
        .copied()
        .filter(|unit| unit.commandable)
        .filter(|unit| (unit.x - x).abs() <= 2.0 && (unit.y - y).abs() <= 2.0)
        .min_by(|a, b| a.dst_edge(x, y).total_cmp(&b.dst_edge(x, y)))
        .map(|unit| unit.id)
}

pub fn selected_enemy_unit_plan(x: f32, y: f32, units: &[SelectUnitCandidate]) -> Option<i32> {
    units
        .iter()
        .copied()
        .filter(|unit| !unit.in_fog_to_player)
        .filter(|unit| (unit.x - x).abs() <= 2.0 && (unit.y - y).abs() <= 2.0)
        .min_by(|a, b| a.dst_edge(x, y).total_cmp(&b.dst_edge(x, y)))
        .map(|unit| unit.id)
}

pub fn selected_command_buildings_plan(
    rect: SelectionRectFrame,
    buildings: &[SelectBuildingCandidate],
) -> Vec<i32> {
    let rad = 4.0;
    buildings
        .iter()
        .copied()
        .filter(|build| build.commandable)
        .filter(|build| {
            build.intersects(
                rect.x - rad / 2.0,
                rect.y - rad / 2.0,
                rad * 2.0 + rect.w,
                rad * 2.0 + rect.h,
            )
        })
        .map(|build| build.id)
        .collect()
}

pub fn selected_command_units_plan<F>(
    rect: SelectionRectFrame,
    units: &[SelectUnitCandidate],
    mut predicate: F,
) -> Vec<i32>
where
    F: FnMut(&SelectUnitCandidate) -> bool,
{
    let rad = 4.0;
    let qx = rect.x - rad / 2.0;
    let qy = rect.y - rad / 2.0;
    let qw = rad * 2.0 + rect.w;
    let qh = rad * 2.0 + rect.h;
    units
        .iter()
        .filter(|unit| unit.commandable && predicate(unit))
        .filter(|unit| {
            let half = unit.hit_size / 2.0;
            let left = unit.x - half;
            let right = unit.x + half;
            let bottom = unit.y - half;
            let top = unit.y + half;
            left < qx + qw && right > qx && bottom < qy + qh && top > qy
        })
        .map(|unit| unit.id)
        .collect()
}

pub fn update_state_plan(frame: InputStateFrame) -> InputStatePlan {
    if frame.state_menu {
        InputStatePlan {
            clear_controlled_type: true,
            clear_logic_cutscene: true,
            force_hide_config: true,
            command_mode: Some(false),
            command_rect: Some(false),
        }
    } else {
        InputStatePlan {
            clear_controlled_type: false,
            clear_logic_cutscene: false,
            force_hide_config: false,
            command_mode: None,
            command_rect: None,
        }
    }
}

pub fn select_units_rect_plan(frame: SelectUnitsRectFrame) -> SelectUnitsRectPlan {
    if !(frame.command_mode && frame.command_rect) {
        return SelectUnitsRectPlan {
            selected_units: frame.selected_units,
            command_buildings: Vec::new(),
            command_rect: frame.command_rect,
            fire_change_event: false,
        };
    }

    if frame.tapped_one {
        return SelectUnitsRectPlan {
            selected_units: frame.selected_units,
            command_buildings: Vec::new(),
            command_rect: false,
            fire_change_event: false,
        };
    }

    let mut selected_units = if frame.multi_unit_select {
        frame
            .selected_units
            .into_iter()
            .filter(|id| !frame.rect_units.contains(id))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    selected_units.extend(frame.rect_units);
    let command_buildings = if selected_units.is_empty() {
        frame.rect_buildings
    } else {
        Vec::new()
    };

    SelectUnitsRectPlan {
        selected_units,
        command_buildings,
        command_rect: false,
        fire_change_event: true,
    }
}

pub fn select_typed_units_plan(frame: SelectTypedUnitsFrame) -> SelectTypedUnitsPlan {
    if !frame.command_mode {
        return SelectTypedUnitsPlan {
            selected_units: Vec::new(),
            fire_change_event: false,
        };
    }
    let Some(unit_type) = frame.selected_unit_type else {
        return SelectTypedUnitsPlan {
            selected_units: Vec::new(),
            fire_change_event: false,
        };
    };
    SelectTypedUnitsPlan {
        selected_units: frame
            .visible_units
            .into_iter()
            .filter_map(|(id, ty)| (ty == unit_type).then_some(id))
            .collect(),
        fire_change_event: true,
    }
}

pub fn tap_command_unit_plan(frame: TapCommandUnitFrame) -> TapCommandUnitPlan {
    if !frame.command_mode {
        return TapCommandUnitPlan {
            selected_units: frame.selected_units,
            command_buildings: frame.command_buildings,
            fire_change_event: false,
        };
    }

    if let Some(unit_id) = frame.selected_unit {
        let mut selected_units = frame.selected_units;
        if let Some(index) = selected_units.iter().position(|id| *id == unit_id) {
            selected_units.remove(index);
        } else {
            selected_units.push(unit_id);
        }
        return TapCommandUnitPlan {
            selected_units,
            command_buildings: Vec::new(),
            fire_change_event: true,
        };
    }

    let mut command_buildings = Vec::new();
    if let Some(build) = frame.build {
        if build.same_team && build.commandable {
            command_buildings = frame.command_buildings;
            if let Some(index) = command_buildings.iter().position(|id| *id == build.id) {
                command_buildings.remove(index);
            } else {
                command_buildings.push(build.id);
            }
        }
    }

    TapCommandUnitPlan {
        selected_units: Vec::new(),
        command_buildings,
        fire_change_event: true,
    }
}

pub fn command_tap_plan(frame: CommandTapFrame) -> CommandTapPlan {
    if !frame.command_mode {
        return CommandTapPlan {
            unit_batches: Vec::new(),
            command_buildings: None,
            fire_attack_event: false,
            fire_position_event: false,
        };
    }

    let attack_target = if frame.enemy_build_target.is_some() {
        frame.enemy_build_target.map(CommandAttackTarget::Building)
    } else if frame.allied_build_target.is_none() {
        frame.enemy_unit_target.map(CommandAttackTarget::Unit)
    } else {
        None
    };

    let max_chunk = frame.max_chunk_size.max(1);
    let mut unit_batches = Vec::new();
    if !frame.selected_units.is_empty() {
        let chunks = frame.selected_units.chunks(max_chunk).collect::<Vec<_>>();
        for (index, chunk) in chunks.iter().enumerate() {
            unit_batches.push(CommandUnitsBatchPlan {
                unit_ids: chunk.to_vec(),
                attack_target,
                target: frame.target,
                queue: frame.queue,
                final_batch: index + 1 == chunks.len(),
            });
        }
    }

    let command_buildings =
        (!frame.command_buildings.is_empty()).then_some((frame.command_buildings, frame.target));

    CommandTapPlan {
        unit_batches,
        command_buildings,
        fire_attack_event: attack_target.is_some(),
        fire_position_event: !frame.selected_units.is_empty() && attack_target.is_none(),
    }
}

fn command_unit_marker_radius(hit_size: f32) -> f32 {
    hit_size / COMMAND_UNIT_SELECT_RADIUS_SCALE
}

fn commanded_unit_marker_radius(hit_size: f32) -> f32 {
    hit_size / COMMAND_UNIT_SELECT_RADIUS_SCALE + 1.0
}

fn command_building_marker_radius(hit_size: f32) -> f32 {
    hit_size / 1.4 + 0.5
}

fn commanded_building_marker_radius(hit_size: f32) -> f32 {
    hit_size / 1.4 + 1.0
}

fn command_overlay_color(controller: CommandOverlayController) -> CommandOverlayColor {
    match controller {
        CommandOverlayController::LogicAi => CommandOverlayColor::Malis,
        CommandOverlayController::CommandAi | CommandOverlayController::Other => {
            CommandOverlayColor::Accent
        }
    }
}

pub fn draw_command_unit_plan(
    unit: CommandSelectableUnit,
    selected: bool,
) -> CommandUnitMarkerPlan {
    CommandUnitMarkerPlan {
        id: unit.id,
        pos: unit.pos,
        sides: 6,
        radius: command_unit_marker_radius(unit.hit_size),
        pulse_radius: 1.0,
        color: if selected {
            CommandOverlayColor::Remove
        } else {
            CommandOverlayColor::Accent
        },
        source: CommandMarkerSource::Selection,
    }
}

pub fn draw_command_building_plan(
    build: CommandSelectableBuilding,
    selected: bool,
) -> CommandBuildingMarkerPlan {
    CommandBuildingMarkerPlan {
        id: build.id,
        pos: build.pos,
        sides: 4,
        radius: command_building_marker_radius(build.hit_size),
        pulse_radius: 1.0,
        color: if selected {
            CommandOverlayColor::Remove
        } else {
            CommandOverlayColor::Accent
        },
        source: CommandMarkerSource::Selection,
    }
}

pub fn command_targets_overlay_plan(
    frame: CommandTargetsOverlayFrame,
) -> CommandTargetsOverlayPlan {
    if !frame.command_mode {
        return CommandTargetsOverlayPlan::default();
    }

    CommandTargetsOverlayPlan {
        target_markers: frame
            .selected_units
            .into_iter()
            .filter(|unit| unit.command_draw_target)
            .filter_map(|unit| unit.attack_target.map(|target| target.pos))
            .map(|pos| CommandTargetMarkerPlan {
                pos,
                kind: CommandTargetMarkerKind::AttackTarget,
                color: CommandOverlayColor::Remove,
                alpha: 1.0,
            })
            .collect(),
    }
}

fn preview_lerp_delta(from: f32, to: f32, alpha: f32, delta: f32) -> f32 {
    let t = (alpha * delta.max(0.0)).clamp(0.0, 1.0);
    from + (to - from) * t
}

fn preview_plan_world_pos(plan: &BuildPlan, block: OtherPlayerPreviewBlock) -> Vec2 {
    let tile_size = TILE_SIZE as f32;
    Vec2 {
        x: plan.x as f32 * tile_size + block.offset,
        y: plan.y as f32 * tile_size + block.offset,
    }
}

fn preview_plan_contains_mouse(
    world_pos: Vec2,
    block: OtherPlayerPreviewBlock,
    mouse_world: Vec2,
) -> bool {
    let half = block.size.max(1) as f32 * TILE_SIZE as f32 / 2.0;
    mouse_world.x >= world_pos.x - half
        && mouse_world.x <= world_pos.x + half
        && mouse_world.y >= world_pos.y - half
        && mouse_world.y <= world_pos.y + half
}

pub fn other_player_preview_overlay_plan<F>(
    player: &mut PlayerComp,
    frame: OtherPlayerPreviewOverlayFrame,
    mut block_lookup: F,
) -> OtherPlayerPreviewOverlayPlan
where
    F: FnMut(&str) -> Option<OtherPlayerPreviewBlock>,
{
    player.get_preview_plans(frame.now_millis);

    let mut overlay = OtherPlayerPreviewOverlayPlan {
        player_id: player.id,
        player_name: player.name.clone(),
        player_pos: Vec2 {
            x: player.x,
            y: player.y,
        },
        ..OtherPlayerPreviewOverlayPlan::default()
    };

    if player.id == frame.local_player_id || player.team != frame.local_team {
        player.preview_plans_current.clear();
        player.preview_plans_dirty = false;
        overlay.cleared_irrelevant = true;
        return overlay;
    }

    overlay.dirty_rebuilt = player.preview_plans_dirty;
    if player.preview_plans_dirty {
        player.preview_plans_dirty = false;
    }

    for plan in &mut player.preview_plans_current {
        if plan.breaking {
            continue;
        }
        let Some(block_name) = plan.block.clone() else {
            continue;
        };
        let Some(block) = block_lookup(&block_name) else {
            continue;
        };

        plan.anim_scale = preview_lerp_delta(plan.anim_scale, 1.0, 0.2, frame.delta);
        let world_pos = preview_plan_world_pos(plan, block);
        let overlapping_mouse = frame
            .mouse_world
            .is_some_and(|mouse| preview_plan_contains_mouse(world_pos, block, mouse));
        let alpha = if overlapping_mouse { 0.7 } else { 0.25 };

        if overlapping_mouse {
            overlay.overlap = Some(OtherPlayerPreviewOverlapPlan {
                x: plan.x,
                y: plan.y,
                block: block_name.clone(),
                player_name: player.name.clone(),
                player_pos: Vec2 {
                    x: player.x,
                    y: player.y,
                },
            });
        }

        overlay.entries.push(OtherPlayerPreviewEntryPlan {
            x: plan.x,
            y: plan.y,
            rotation: plan.rotation,
            block: block_name,
            world_pos,
            size: block.size,
            anim_scale: plan.anim_scale,
            alpha,
            overlapping_mouse,
        });
    }

    overlay
}

pub fn command_overlay_plan(frame: CommandOverlayFrame) -> CommandOverlayPlan {
    if !frame.command_mode {
        return CommandOverlayPlan::default();
    }

    let mut plan = CommandOverlayPlan::default();

    for unit in frame.selected_units {
        if !unit.allow_command {
            plan.removed_selected_units.push(unit.id);
            continue;
        }
        plan.retained_selected_units.push(unit.id);

        if (unit.is_flying || unit.allow_leg_step) != frame.flying_pass {
            continue;
        }

        let color = command_overlay_color(unit.controller);
        let mut last_pos = unit.pos;

        if unit.controller == CommandOverlayController::CommandAi {
            let line_dest = unit
                .attack_target
                .map(|target| target.pos)
                .or(unit.target_pos);

            if unit.command_draw_target {
                if let (Some(target_pos), Some(line_to)) = (unit.target_pos, line_dest) {
                    plan.target_lines.push(CommandOverlayLinePlan {
                        from: unit.pos,
                        to: line_to,
                        from_margin: commanded_unit_marker_radius(unit.hit_size),
                        to_margin: COMMAND_OVERLAY_LINE_LIMIT,
                        color,
                        alpha: COMMAND_OVERLAY_ALPHA,
                    });

                    if unit.attack_target.is_none() {
                        plan.target_markers.push(CommandTargetMarkerPlan {
                            pos: line_to,
                            kind: CommandTargetMarkerKind::MoveSquare,
                            color,
                            alpha: COMMAND_OVERLAY_ALPHA,
                        });

                        if unit.enter_payload_command {
                            plan.target_markers.push(CommandTargetMarkerPlan {
                                pos: target_pos,
                                kind: if unit.enter_payload_target_accepts {
                                    CommandTargetMarkerKind::EnterPayloadAccepted
                                } else {
                                    CommandTargetMarkerKind::EnterPayloadRejected
                                },
                                color: if unit.enter_payload_target_accepts {
                                    color
                                } else {
                                    CommandOverlayColor::Remove
                                },
                                alpha: 1.0,
                            });
                        }
                    }
                }
            }

            if let Some(target) = unit.attack_target {
                last_pos = target.pos;
            } else if let Some(target_pos) = unit.target_pos {
                last_pos = target_pos;
            }
        }

        let radius = commanded_unit_marker_radius(unit.hit_size);
        plan.unit_markers.push(CommandUnitMarkerPlan {
            id: unit.id,
            pos: unit.pos,
            sides: 6,
            radius,
            pulse_radius: 0.5,
            color,
            source: CommandMarkerSource::Commanded,
        });

        if unit.controller == CommandOverlayController::CommandAi {
            if unit.command_draw_target {
                for next in &unit.command_queue {
                    plan.queue_lines.push(CommandOverlayLinePlan {
                        from: last_pos,
                        to: next.pos,
                        from_margin: COMMAND_OVERLAY_LINE_LIMIT,
                        to_margin: COMMAND_OVERLAY_LINE_LIMIT,
                        color,
                        alpha: COMMAND_OVERLAY_ALPHA,
                    });
                    plan.queue_markers.push(CommandTargetMarkerPlan {
                        pos: next.pos,
                        kind: match next.kind {
                            CommandOverlayTargetKind::Position => {
                                CommandTargetMarkerKind::MoveSquare
                            }
                            CommandOverlayTargetKind::Entity => {
                                CommandTargetMarkerKind::AttackTarget
                            }
                        },
                        color: match next.kind {
                            CommandOverlayTargetKind::Position => color,
                            CommandOverlayTargetKind::Entity => CommandOverlayColor::Remove,
                        },
                        alpha: COMMAND_OVERLAY_ALPHA,
                    });
                    last_pos = next.pos;
                }
            }

            if unit.target_pos.is_some() && unit.loop_payload_command {
                if let Some(target_pos) = unit.target_pos {
                    plan.payload_icons.push(CommandPayloadIconPlan {
                        pos: target_pos,
                        kind: if unit.has_payload {
                            CommandPayloadIconKind::Download
                        } else {
                            CommandPayloadIconKind::Upload
                        },
                        offset_y: 11.0,
                        size: 8.0,
                        color,
                    });
                }

                if let Some(first) = unit.command_queue.first() {
                    plan.payload_icons.push(CommandPayloadIconPlan {
                        pos: first.pos,
                        kind: if unit.has_payload {
                            CommandPayloadIconKind::Upload
                        } else {
                            CommandPayloadIconKind::Download
                        },
                        offset_y: 11.0,
                        size: 8.0,
                        color,
                    });
                }
            }
        }
    }

    if frame.flying_pass {
        for build in frame.command_buildings {
            plan.building_markers.push(CommandBuildingMarkerPlan {
                id: build.id,
                pos: build.pos,
                sides: 4,
                radius: commanded_building_marker_radius(build.hit_size),
                pulse_radius: 0.0,
                color: CommandOverlayColor::Accent,
                source: CommandMarkerSource::Commanded,
            });

            if let Some(command_position) = build.command_position {
                plan.target_lines.push(CommandOverlayLinePlan {
                    from: build.pos,
                    to: command_position,
                    from_margin: build.hit_size / 2.0,
                    to_margin: COMMAND_OVERLAY_LINE_LIMIT,
                    color: CommandOverlayColor::Accent,
                    alpha: COMMAND_OVERLAY_ALPHA,
                });
                plan.target_markers.push(CommandTargetMarkerPlan {
                    pos: command_position,
                    kind: CommandTargetMarkerKind::MoveSquare,
                    color: CommandOverlayColor::Accent,
                    alpha: COMMAND_OVERLAY_ALPHA,
                });
            }
        }
    }

    plan
}

pub fn command_selection_overlay_plan(
    frame: CommandSelectionOverlayFrame,
) -> CommandSelectionOverlayPlan {
    if !frame.command_mode {
        return CommandSelectionOverlayPlan::default();
    }

    if frame.command_rect {
        let unit_markers = frame
            .rect_units
            .into_iter()
            .map(|unit| draw_command_unit_plan(unit, frame.selected_units.contains(&unit.id)))
            .collect::<Vec<_>>();

        let building_markers = if unit_markers.is_empty() {
            frame
                .rect_buildings
                .into_iter()
                .map(|build| {
                    draw_command_building_plan(build, frame.command_buildings.contains(&build.id))
                })
                .collect()
        } else {
            Vec::new()
        };

        return CommandSelectionOverlayPlan {
            unit_markers,
            building_markers,
            rect_fill: Some(frame.rect),
        };
    }

    if let Some(unit) = frame.hover_unit {
        if !(!frame.multi_unit_select
            && frame.selected_units.len() == 1
            && frame.selected_units.contains(&unit.id))
        {
            return CommandSelectionOverlayPlan {
                unit_markers: vec![draw_command_unit_plan(
                    unit,
                    frame.selected_units.contains(&unit.id),
                )],
                building_markers: Vec::new(),
                rect_fill: None,
            };
        }
    }

    CommandSelectionOverlayPlan::default()
}

pub fn check_unit_plan(frame: CheckUnitFrame) -> CheckUnitPlan {
    if !frame.controlled_type_present || !frame.controlled_type_player_controllable {
        return CheckUnitPlan {
            accepted: false,
            action: None,
        };
    }

    let target_present = frame.closest_unit_present || frame.block_control_unit_present;
    CheckUnitPlan {
        accepted: target_present,
        action: target_present.then_some(if frame.net_client {
            InputHandlerLocalAction::UnitControlRemote
        } else {
            InputHandlerLocalAction::UnitControlLocal
        }),
    }
}

pub fn try_pickup_payload_plan(frame: PayloadPickupFrame) -> PayloadPickupPlan {
    if !frame.unit_is_payload {
        return PayloadPickupPlan {
            accepted: false,
            action: None,
        };
    }

    if frame.pickup_unit_available {
        return PayloadPickupPlan {
            accepted: true,
            action: Some(InputHandlerLocalAction::RequestUnitPayload),
        };
    }

    if !frame.build_present {
        return PayloadPickupPlan {
            accepted: false,
            action: None,
        };
    }

    let can_pick_build = frame.teams_can_interact
        && (frame.stored_payload_pickable
            || (!frame.build_visibility_hidden
                && frame.build_can_pickup
                && frame.payload_can_pickup_build));

    PayloadPickupPlan {
        accepted: can_pick_build,
        action: can_pick_build.then_some(InputHandlerLocalAction::RequestBuildPayload),
    }
}

pub fn try_drop_payload_plan(frame: PayloadDropFrame) -> PayloadDropPlan {
    let accepted = frame.unit_is_payload && frame.can_drop_payload;
    PayloadDropPlan {
        accepted,
        action: accepted.then_some(InputHandlerLocalAction::RequestDropPayload {
            x: frame.player_x,
            y: frame.player_y,
        }),
    }
}

pub fn can_shoot_plan(frame: CanShootFrame) -> bool {
    !frame.block_selected
        && !frame.on_configurable
        && !frame.dropping_item
        && !frame.actively_building
        && !frame.mech_flying
        && !frame.mining
        && !frame.command_mode
}

pub fn can_drop_item_plan(dropping_item: bool, can_tap_player: bool) -> bool {
    dropping_item && !can_tap_player
}

pub fn can_deposit_item_plan(frame: DepositItemFrame) -> bool {
    if frame.block_deposit_cooldown >= 0.0 {
        frame.item_deposit_cooldown - frame.rules_item_deposit_cooldown
            <= -frame.block_deposit_cooldown
    } else {
        frame.item_deposit_cooldown <= 0.0
    }
}

pub fn try_drop_items_plan(frame: TryDropItemsFrame) -> TryDropItemsPlan {
    if frame.player_dead {
        return TryDropItemsPlan {
            player_dead_ignored: true,
            dropping_item: frame.dropping_item,
            action: None,
        };
    }

    if !frame.dropping_item || frame.stack_amount <= 0 || frame.can_tap_player || frame.state_paused
    {
        return TryDropItemsPlan {
            player_dead_ignored: false,
            dropping_item: false,
            action: Some(InputHandlerLocalAction::ClearDroppingItem),
        };
    }

    let can_transfer = frame.build_present
        && frame.build_accepts_stack > 0
        && frame.build_interactable
        && frame.build_has_items
        && frame.stack_amount > 0
        && frame.build_allow_deposit
        && frame.can_deposit_item;

    TryDropItemsPlan {
        player_dead_ignored: false,
        dropping_item: false,
        action: Some(if can_transfer {
            InputHandlerLocalAction::TransferInventory {
                new_item_deposit_cooldown: frame.rules_item_deposit_cooldown,
            }
        } else {
            InputHandlerLocalAction::DropItem {
                angle: frame.drop_angle,
            }
        }),
    }
}

pub fn unit_building_control_select<C>(
    unit: Option<&UnitComp>,
    build: Option<&BuildingComp>,
    client_side: bool,
    can_control_select: C,
) -> UnitBuildingControlSelectOutcome
where
    C: FnOnce(&UnitComp, &BuildingComp) -> bool,
{
    let Some(unit) = unit else {
        return UnitBuildingControlSelectOutcome::rejected(
            UnitBuildingControlSelectRejectReason::MissingUnit,
        );
    };
    if unit.health.dead {
        return UnitBuildingControlSelectOutcome::rejected(
            UnitBuildingControlSelectRejectReason::UnitDead,
        );
    }

    let Some(build) = build else {
        return UnitBuildingControlSelectOutcome::rejected(
            UnitBuildingControlSelectRejectReason::MissingBuild,
        );
    };

    if unit.team.team != build.team {
        return UnitBuildingControlSelectOutcome::rejected(
            UnitBuildingControlSelectRejectReason::TeamMismatch,
        );
    }

    if !client_side && !can_control_select(unit, build) {
        return UnitBuildingControlSelectOutcome::rejected(
            UnitBuildingControlSelectRejectReason::NotSelectable,
        );
    }

    UnitBuildingControlSelectOutcome {
        accepted: true,
        rejection: None,
        packet: Some(UnitBuildingControlSelectCallPacket {
            unit: UnitRef::Unit { id: unit.id() },
            build: BuildingRef::new(build.tile_pos),
        }),
    }
}

fn transform_point_config<F>(config: &TypeValue, mut transform: F) -> TypeValue
where
    F: FnMut(Point2) -> Point2,
{
    match config {
        TypeValue::Point2(point) => TypeValue::Point2(transform(*point)),
        TypeValue::Point2Array(points) => {
            TypeValue::Point2Array(points.iter().copied().map(transform).collect())
        }
        _ => config.clone(),
    }
}

fn rotate_config_point(point: Point2, direction: i32, block_size: i32) -> Point2 {
    let offset = if block_size % 2 == 0 { -0.5 } else { 0.0 };
    let mut cx = point.x as f32 + offset;
    let mut cy = point.y as f32 + offset;
    let old_x = cx;
    if direction >= 0 {
        cx = -cy;
        cy = old_x;
    } else {
        cx = cy;
        cy = -old_x;
    }
    Point2::new((cx - offset).floor() as i32, (cy - offset).floor() as i32)
}

fn flip_config_point(mut point: Point2, flip_x: bool, block_size: i32) -> Point2 {
    if flip_x {
        if block_size % 2 == 0 {
            point.x -= 1;
        }
        point.x = -point.x;
    } else {
        if block_size % 2 == 0 {
            point.y -= 1;
        }
        point.y = -point.y;
    }
    point
}

fn world_to_tile_runtime(coord: f32) -> i32 {
    (coord / TILE_SIZE as f32 + 0.5).floor() as i32
}

pub fn unit_building_control_select_packet(
    unit: UnitRef,
    build: &BuildingComp,
) -> UnitBuildingControlSelectCallPacket {
    UnitBuildingControlSelectCallPacket {
        unit,
        build: BuildingRef::new(build.tile_pos),
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

    fn command_block() -> Block {
        let mut block = block();
        block.commandable = true;
        block
    }

    fn liquid_block() -> Block {
        let mut block = block();
        block.has_liquids = true;
        block.liquid_capacity = 40.0;
        block
    }

    fn item_id(name: &str) -> Option<i16> {
        match name {
            "copper" => Some(0),
            "lead" => Some(1),
            "scrap" => Some(2),
            _ => None,
        }
    }

    fn liquid_id(name: &str) -> Option<i16> {
        match name {
            "water" => Some(0),
            "slag" => Some(1),
            "oil" => Some(2),
            _ => None,
        }
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
    fn client_unit_cargo_unload_config_packets_use_item_content_and_clear_null() {
        let building = BuildingComp::new(point2_pack(8, 9), block(), TeamId(1));

        let item_packet = client_unit_cargo_unload_item_config_packet(&building, 3);
        assert_eq!(item_packet.player, EntityRef::null());
        assert_eq!(item_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(
            item_packet.value,
            TypeValue::Content(ContentRef::new(ContentType::Item, 3))
        );

        let clear_packet = client_unit_cargo_unload_clear_config_packet(&building);
        assert_eq!(clear_packet.player, EntityRef::null());
        assert_eq!(clear_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(clear_packet.value, TypeValue::Null);
    }

    #[test]
    fn client_unit_factory_config_packets_cover_plan_unit_command_and_clear_values() {
        let building = BuildingComp::new(point2_pack(8, 9), block(), TeamId(1));

        let plan_packet = client_unit_factory_plan_config_packet(&building, 2);
        assert_eq!(plan_packet.player, EntityRef::null());
        assert_eq!(plan_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(plan_packet.value, TypeValue::Int(2));

        let unit_packet = client_unit_factory_unit_config_packet(&building, 17);
        assert_eq!(unit_packet.player, EntityRef::null());
        assert_eq!(unit_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(
            unit_packet.value,
            TypeValue::Content(ContentRef::new(ContentType::Unit, 17))
        );

        let command_packet = client_unit_factory_command_config_packet(&building, 7);
        assert_eq!(command_packet.player, EntityRef::null());
        assert_eq!(command_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(
            command_packet.value,
            TypeValue::Content(ContentRef::new(ContentType::UnitCommand, 7))
        );

        let clear_packet = client_unit_factory_clear_command_packet(&building);
        assert_eq!(clear_packet.player, EntityRef::null());
        assert_eq!(clear_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(clear_packet.value, TypeValue::Null);
    }

    #[test]
    fn client_reconstructor_config_packets_use_unit_command_content_and_clear_null() {
        let building = BuildingComp::new(point2_pack(8, 9), block(), TeamId(1));

        let command_packet = client_reconstructor_command_config_packet(&building, 7);
        assert_eq!(command_packet.player, EntityRef::null());
        assert_eq!(command_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(
            command_packet.value,
            TypeValue::Content(ContentRef::new(ContentType::UnitCommand, 7))
        );

        let clear_packet = client_reconstructor_clear_command_packet(&building);
        assert_eq!(clear_packet.player, EntityRef::null());
        assert_eq!(clear_packet.build, BuildingRef::new(point2_pack(8, 9)));
        assert_eq!(clear_packet.value, TypeValue::Null);
    }

    #[test]
    fn config_and_tile_tap_plans_cover_null_and_command_mode_paths() {
        assert!(check_config_tap_plan(ConfigTapFrame {
            config_shown: true,
            selected_on_configure_tapped: true,
        }));
        assert!(!check_config_tap_plan(ConfigTapFrame {
            config_shown: false,
            selected_on_configure_tapped: true,
        }));

        let none = tile_tapped_plan(TileTappedFrame::default());
        assert!(!none.consumed);
        assert_eq!(
            none.actions,
            vec![
                TileTappedAction::HidePlanConfig,
                TileTappedAction::HideInventory,
                TileTappedAction::HideConfig,
                TileTappedAction::ClearCommandBuildings
            ]
        );

        let command = tile_tapped_plan(TileTappedFrame {
            build_present: true,
            build_commandable: true,
            command_mode: true,
            build_interactable: true,
            ..TileTappedFrame::default()
        });
        assert!(command.consumed);
        assert!(!command.actions.contains(&TileTappedAction::CallTapped));
        assert!(command.actions.contains(&TileTappedAction::HideInventory));
    }

    #[test]
    fn tile_tap_plan_shows_config_or_hides_existing_config_like_java() {
        let show = tile_tapped_plan(TileTappedFrame {
            build_present: true,
            block_configurable: true,
            build_interactable: true,
            should_show_configure: true,
            ..TileTappedFrame::default()
        });
        assert!(show.consumed);
        assert!(show.actions.contains(&TileTappedAction::PlayConfigureSound));
        assert!(show.actions.contains(&TileTappedAction::ShowConfig));

        let hide = tile_tapped_plan(TileTappedFrame {
            build_present: true,
            config_shown: true,
            selected_accepts_configure_build_tap: true,
            config_has_mouse: false,
            build_interactable: true,
            ..TileTappedFrame::default()
        });
        assert!(hide.consumed);
        assert!(hide.actions.contains(&TileTappedAction::HideConfig));
        assert!(!hide.actions.contains(&TileTappedAction::CallTapped));
    }

    #[test]
    fn tile_tap_plan_calls_tapped_and_handles_synthetic_inventory() {
        let tapped = tile_tapped_plan(TileTappedFrame {
            build_present: true,
            build_interactable: true,
            ..TileTappedFrame::default()
        });
        assert!(!tapped.consumed);
        assert!(tapped.actions.contains(&TileTappedAction::CallTapped));
        assert!(tapped.actions.contains(&TileTappedAction::HideInventory));

        let inventory = tile_tapped_plan(TileTappedFrame {
            build_present: true,
            build_interactable: true,
            block_synthetic: true,
            block_has_items: true,
            items_total: 3,
            ..TileTappedFrame::default()
        });
        assert!(inventory.consumed);
        assert!(inventory.showed_inventory);
        assert!(inventory.actions.contains(&TileTappedAction::ShowInventory));
        assert!(!inventory.actions.contains(&TileTappedAction::HideInventory));

        let consumes = tile_tapped_plan(TileTappedFrame {
            build_present: true,
            build_interactable: true,
            block_consumes_tap: true,
            ..TileTappedFrame::default()
        });
        assert!(consumes.consumed);
        assert!(consumes.actions.contains(&TileTappedAction::CallTapped));
        assert!(consumes.actions.contains(&TileTappedAction::HideInventory));
    }

    #[test]
    fn command_mode_state_rect_and_typed_selection_plans_match_java_state_changes() {
        assert_eq!(
            update_state_plan(InputStateFrame { state_menu: true }),
            InputStatePlan {
                clear_controlled_type: true,
                clear_logic_cutscene: true,
                force_hide_config: true,
                command_mode: Some(false),
                command_rect: Some(false),
            }
        );
        assert_eq!(
            update_state_plan(InputStateFrame { state_menu: false }),
            InputStatePlan {
                clear_controlled_type: false,
                clear_logic_cutscene: false,
                force_hide_config: false,
                command_mode: None,
                command_rect: None,
            }
        );

        let rect = select_units_rect_plan(SelectUnitsRectFrame {
            command_mode: true,
            command_rect: true,
            tapped_one: false,
            multi_unit_select: false,
            selected_units: vec![9],
            rect_units: vec![1, 2],
            rect_buildings: vec![7],
        });
        assert_eq!(rect.selected_units, vec![1, 2]);
        assert!(rect.command_buildings.is_empty());
        assert!(!rect.command_rect);
        assert!(rect.fire_change_event);

        let buildings = select_units_rect_plan(SelectUnitsRectFrame {
            command_mode: true,
            command_rect: true,
            tapped_one: false,
            multi_unit_select: false,
            selected_units: vec![9],
            rect_units: vec![],
            rect_buildings: vec![7, 8],
        });
        assert!(buildings.selected_units.is_empty());
        assert_eq!(buildings.command_buildings, vec![7, 8]);

        let typed = select_typed_units_plan(SelectTypedUnitsFrame {
            command_mode: true,
            selected_unit_type: Some("dagger".into()),
            visible_units: vec![
                (1, "dagger".into()),
                (2, "flare".into()),
                (3, "dagger".into()),
            ],
        });
        assert_eq!(typed.selected_units, vec![1, 3]);
        assert!(typed.fire_change_event);
    }

    #[test]
    fn tap_command_unit_plan_toggles_units_or_command_buildings() {
        let add = tap_command_unit_plan(TapCommandUnitFrame {
            command_mode: true,
            selected_unit: Some(2),
            selected_units: vec![1],
            command_buildings: vec![9],
            build: None,
        });
        assert_eq!(add.selected_units, vec![1, 2]);
        assert!(add.command_buildings.is_empty());
        assert!(add.fire_change_event);

        let remove = tap_command_unit_plan(TapCommandUnitFrame {
            command_mode: true,
            selected_unit: Some(2),
            selected_units: vec![1, 2],
            command_buildings: vec![],
            build: None,
        });
        assert_eq!(remove.selected_units, vec![1]);

        let build = tap_command_unit_plan(TapCommandUnitFrame {
            command_mode: true,
            selected_unit: None,
            selected_units: vec![1],
            command_buildings: vec![8],
            build: Some(CommandBuildTapCandidate {
                id: 9,
                same_team: true,
                commandable: true,
            }),
        });
        assert!(build.selected_units.is_empty());
        assert_eq!(build.command_buildings, vec![8, 9]);

        let clear = tap_command_unit_plan(TapCommandUnitFrame {
            command_mode: true,
            selected_unit: None,
            selected_units: vec![1],
            command_buildings: vec![8],
            build: Some(CommandBuildTapCandidate {
                id: 9,
                same_team: false,
                commandable: true,
            }),
        });
        assert!(clear.selected_units.is_empty());
        assert!(clear.command_buildings.is_empty());
    }

    #[test]
    fn command_tap_plan_splits_unit_batches_and_targets_enemies_or_positions() {
        let target = Vec2::new(10.0, 20.0);
        let plan = command_tap_plan(CommandTapFrame {
            command_mode: true,
            selected_units: vec![1, 2, 3, 4, 5],
            command_buildings: vec![100, 101],
            target,
            queue: true,
            enemy_unit_target: Some(9),
            max_chunk_size: 2,
            ..CommandTapFrame::default()
        });

        assert_eq!(plan.unit_batches.len(), 3);
        assert_eq!(plan.unit_batches[0].unit_ids, vec![1, 2]);
        assert_eq!(
            plan.unit_batches[0].attack_target,
            Some(CommandAttackTarget::Unit(9))
        );
        assert!(!plan.unit_batches[0].final_batch);
        assert_eq!(plan.unit_batches[2].unit_ids, vec![5]);
        assert!(plan.unit_batches[2].final_batch);
        assert_eq!(plan.command_buildings, Some((vec![100, 101], target)));
        assert!(plan.fire_attack_event);
        assert!(!plan.fire_position_event);

        let position = command_tap_plan(CommandTapFrame {
            command_mode: true,
            selected_units: vec![1],
            target,
            allied_build_target: Some(55),
            enemy_unit_target: Some(9),
            ..CommandTapFrame::default()
        });
        assert_eq!(position.unit_batches[0].attack_target, None);
        assert!(position.fire_position_event);
        assert!(!position.fire_attack_event);

        let inactive = command_tap_plan(CommandTapFrame {
            command_mode: false,
            selected_units: vec![1],
            command_buildings: vec![2],
            target,
            ..CommandTapFrame::default()
        });
        assert!(inactive.unit_batches.is_empty());
        assert!(inactive.command_buildings.is_none());
    }

    #[test]
    fn command_overlay_plan_emits_commanded_lines_queue_payload_and_buildings() {
        let unit = CommandOverlayUnit {
            id: 1,
            pos: Vec2::new(0.0, 0.0),
            hit_size: 8.0,
            allow_command: true,
            is_flying: true,
            allow_leg_step: false,
            controller: CommandOverlayController::CommandAi,
            command_draw_target: true,
            target_pos: Some(Vec2::new(10.0, 0.0)),
            attack_target: None,
            command_queue: vec![
                CommandOverlayTarget::position(Vec2::new(20.0, 0.0)),
                CommandOverlayTarget::entity(Vec2::new(30.0, 0.0)),
            ],
            enter_payload_command: true,
            enter_payload_target_accepts: false,
            loop_payload_command: true,
            has_payload: true,
        };
        let removed = CommandOverlayUnit {
            id: 2,
            allow_command: false,
            ..unit.clone()
        };
        let build = CommandOverlayBuilding {
            id: 9,
            pos: Vec2::new(5.0, 5.0),
            hit_size: 14.0,
            command_position: Some(Vec2::new(12.0, 12.0)),
        };

        let plan = command_overlay_plan(CommandOverlayFrame {
            command_mode: true,
            flying_pass: true,
            selected_units: vec![unit, removed],
            command_buildings: vec![build],
        });

        assert_eq!(plan.retained_selected_units, vec![1]);
        assert_eq!(plan.removed_selected_units, vec![2]);
        assert_eq!(plan.unit_markers.len(), 1);
        assert_eq!(plan.unit_markers[0].radius, 9.0);
        assert_eq!(plan.target_lines.len(), 2);
        assert_eq!(
            plan.target_markers[0].kind,
            CommandTargetMarkerKind::MoveSquare
        );
        assert!(plan
            .target_markers
            .iter()
            .any(|marker| marker.kind == CommandTargetMarkerKind::EnterPayloadRejected));
        assert_eq!(plan.queue_lines.len(), 2);
        assert_eq!(
            plan.queue_markers[0].kind,
            CommandTargetMarkerKind::MoveSquare
        );
        assert_eq!(
            plan.queue_markers[1].kind,
            CommandTargetMarkerKind::AttackTarget
        );
        assert_eq!(
            plan.payload_icons
                .iter()
                .map(|icon| icon.kind)
                .collect::<Vec<_>>(),
            vec![
                CommandPayloadIconKind::Download,
                CommandPayloadIconKind::Upload
            ]
        );
        assert_eq!(plan.building_markers.len(), 1);
    }

    #[test]
    fn preview_overlay_plan_commits_only_after_preview_group_delay() {
        let mut remote = PlayerComp::new(TeamId(1));
        remote.id = 7;
        remote.name = "ally-builder".into();
        remote.x = 64.0;
        remote.y = 72.0;
        remote.handle_preview_plans(1, &[BuildPlan::new_place(4, 5, 1, "router")], 0, 10);

        let frame = |now_millis, mouse_world| OtherPlayerPreviewOverlayFrame {
            local_player_id: 1,
            local_team: TeamId(1),
            now_millis,
            delta: 1.0,
            mouse_world,
        };
        let block_lookup = |name: &str| {
            (name == "router").then_some(OtherPlayerPreviewBlock {
                size: 1,
                offset: TILE_SIZE as f32 / 2.0,
            })
        };

        let early = other_player_preview_overlay_plan(&mut remote, frame(99, None), block_lookup);
        assert!(early.entries.is_empty());
        assert!(remote.receiving_new_plan_group);

        let committed = other_player_preview_overlay_plan(
            &mut remote,
            frame(
                100,
                Some(Vec2::new(
                    4.0 * TILE_SIZE as f32 + 4.0,
                    5.0 * TILE_SIZE as f32 + 4.0,
                )),
            ),
            block_lookup,
        );
        assert!(committed.dirty_rebuilt);
        assert_eq!(committed.entries.len(), 1);
        assert_eq!(committed.entries[0].block, "router");
        assert_eq!(committed.entries[0].rotation, 1);
        assert_eq!(committed.entries[0].world_pos, Vec2::new(36.0, 44.0));
        assert_eq!(committed.entries[0].anim_scale, 0.2);
        assert_eq!(committed.entries[0].alpha, 0.7);
        assert!(committed.entries[0].overlapping_mouse);
        assert_eq!(
            committed.overlap,
            Some(OtherPlayerPreviewOverlapPlan {
                x: 4,
                y: 5,
                block: "router".into(),
                player_name: "ally-builder".into(),
                player_pos: Vec2::new(64.0, 72.0),
            })
        );
        assert!(!remote.preview_plans_dirty);
        assert!(!remote.receiving_new_plan_group);
        assert!(remote.preview_plans_assembling.is_empty());
    }

    #[test]
    fn preview_overlay_plan_clears_local_or_enemy_preview_plans_like_java_draw_other_build_plans() {
        let mut local = PlayerComp::new(TeamId(1));
        local.id = 9;
        local.preview_plans_current = vec![BuildPlan::new_place(1, 2, 0, "router")];
        local.preview_plans_dirty = true;

        let frame = OtherPlayerPreviewOverlayFrame {
            local_player_id: 9,
            local_team: TeamId(1),
            now_millis: 0,
            delta: 1.0,
            mouse_world: None,
        };
        let plan = other_player_preview_overlay_plan(&mut local, frame, |_| {
            Some(OtherPlayerPreviewBlock {
                size: 1,
                offset: 4.0,
            })
        });
        assert!(plan.cleared_irrelevant);
        assert!(plan.entries.is_empty());
        assert!(local.preview_plans_current.is_empty());
        assert!(!local.preview_plans_dirty);

        let mut enemy = PlayerComp::new(TeamId(2));
        enemy.id = 10;
        enemy.preview_plans_current = vec![BuildPlan::new_place(3, 4, 0, "router")];
        let plan = other_player_preview_overlay_plan(&mut enemy, frame, |_| {
            Some(OtherPlayerPreviewBlock {
                size: 1,
                offset: 4.0,
            })
        });
        assert!(plan.cleared_irrelevant);
        assert!(enemy.preview_plans_current.is_empty());
    }

    #[test]
    fn command_targets_overlay_plan_matches_java_attack_target_layer() {
        let plan = command_targets_overlay_plan(CommandTargetsOverlayFrame {
            command_mode: true,
            selected_units: vec![
                CommandOverlayUnit {
                    id: 1,
                    pos: Vec2::new(0.0, 0.0),
                    hit_size: 8.0,
                    allow_command: true,
                    is_flying: false,
                    allow_leg_step: false,
                    controller: CommandOverlayController::CommandAi,
                    command_draw_target: true,
                    target_pos: Some(Vec2::new(5.0, 5.0)),
                    attack_target: Some(CommandOverlayTarget::entity(Vec2::new(9.0, 9.0))),
                    command_queue: Vec::new(),
                    enter_payload_command: false,
                    enter_payload_target_accepts: false,
                    loop_payload_command: false,
                    has_payload: false,
                },
                CommandOverlayUnit {
                    id: 2,
                    pos: Vec2::new(1.0, 1.0),
                    hit_size: 8.0,
                    allow_command: true,
                    is_flying: false,
                    allow_leg_step: false,
                    controller: CommandOverlayController::CommandAi,
                    command_draw_target: false,
                    target_pos: Some(Vec2::new(6.0, 6.0)),
                    attack_target: Some(CommandOverlayTarget::entity(Vec2::new(10.0, 10.0))),
                    command_queue: Vec::new(),
                    enter_payload_command: false,
                    enter_payload_target_accepts: false,
                    loop_payload_command: false,
                    has_payload: false,
                },
            ],
        });

        assert_eq!(plan.target_markers.len(), 1);
        assert_eq!(plan.target_markers[0].pos, Vec2::new(9.0, 9.0));
        assert_eq!(
            plan.target_markers[0].kind,
            CommandTargetMarkerKind::AttackTarget
        );
    }

    #[test]
    fn command_selection_overlay_plan_prefers_rect_units_then_buildings_and_hover() {
        let rect = SelectionRectFrame {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
        };
        let unit = CommandSelectableUnit {
            id: 1,
            pos: Vec2::new(3.0, 4.0),
            hit_size: 6.0,
        };
        let build = CommandSelectableBuilding {
            id: 7,
            pos: Vec2::new(5.0, 6.0),
            hit_size: 12.0,
        };

        let units = command_selection_overlay_plan(CommandSelectionOverlayFrame {
            command_mode: true,
            command_rect: true,
            rect,
            selected_units: vec![1],
            command_buildings: vec![7],
            rect_units: vec![unit],
            rect_buildings: vec![build],
            hover_unit: None,
            multi_unit_select: false,
        });
        assert_eq!(units.unit_markers.len(), 1);
        assert!(units.building_markers.is_empty());
        assert_eq!(units.unit_markers[0].color, CommandOverlayColor::Remove);
        assert_eq!(units.rect_fill, Some(rect));

        let buildings = command_selection_overlay_plan(CommandSelectionOverlayFrame {
            command_mode: true,
            command_rect: true,
            rect,
            selected_units: vec![],
            command_buildings: vec![],
            rect_units: vec![],
            rect_buildings: vec![build],
            hover_unit: None,
            multi_unit_select: false,
        });
        assert_eq!(buildings.building_markers.len(), 1);
        assert_eq!(
            buildings.building_markers[0].color,
            CommandOverlayColor::Accent
        );

        let hover_hidden = command_selection_overlay_plan(CommandSelectionOverlayFrame {
            command_mode: true,
            command_rect: false,
            rect,
            selected_units: vec![1],
            command_buildings: vec![],
            rect_units: vec![],
            rect_buildings: vec![],
            hover_unit: Some(unit),
            multi_unit_select: false,
        });
        assert!(hover_hidden.unit_markers.is_empty());

        let hover_multi = command_selection_overlay_plan(CommandSelectionOverlayFrame {
            multi_unit_select: true,
            hover_unit: Some(unit),
            ..CommandSelectionOverlayFrame {
                command_mode: true,
                command_rect: false,
                rect,
                selected_units: vec![1],
                command_buildings: vec![],
                rect_units: vec![],
                rect_buildings: vec![],
                hover_unit: None,
                multi_unit_select: false,
            }
        });
        assert_eq!(hover_multi.unit_markers.len(), 1);
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
    fn set_item_updates_building_items_and_records_packet() {
        let mut building = BuildingComp::new(point2_pack(12, 14), item_block(), TeamId(1));
        building.items.as_mut().unwrap().set(0, 5);

        let outcome = set_item(Some(&mut building), Some("copper".into()), 9, item_id);

        assert!(outcome.accepted);
        assert_eq!(outcome.previous_amount, 5);
        assert_eq!(outcome.new_amount, 9);
        assert_eq!(building.items.as_ref().unwrap().get(0), 9);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(12, 14)));
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!(packet.amount, 9);
    }

    #[test]
    fn set_items_applies_all_stacks_without_clearing_absent_items() {
        let mut building = BuildingComp::new(point2_pack(13, 15), item_block(), TeamId(1));
        building.items.as_mut().unwrap().set(2, 7);

        let stacks = vec![ItemStack::new("copper", 3), ItemStack::new("lead", 4)];
        let outcome = set_items(Some(&mut building), stacks.clone(), item_id);

        assert!(outcome.accepted);
        assert_eq!(outcome.applied_entries, 2);
        let items = building.items.as_ref().unwrap();
        assert_eq!(items.get(0), 3);
        assert_eq!(items.get(1), 4);
        assert_eq!(items.get(2), 7);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(13, 15)));
        assert_eq!(packet.items, stacks);
    }

    #[test]
    fn set_items_rejects_unknown_item_without_partial_mutation() {
        let mut building = BuildingComp::new(point2_pack(14, 16), item_block(), TeamId(1));
        building.items.as_mut().unwrap().set(0, 5);

        let outcome = set_items(
            Some(&mut building),
            vec![ItemStack::new("copper", 1), ItemStack::new("missing", 2)],
            item_id,
        );

        assert!(!outcome.accepted);
        assert_eq!(outcome.rejection, Some(ItemSyncRejectReason::UnknownItem));
        assert_eq!(building.items.as_ref().unwrap().get(0), 5);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn clear_items_clears_module_and_records_packet() {
        let mut building = BuildingComp::new(point2_pack(15, 17), item_block(), TeamId(1));
        let items = building.items.as_mut().unwrap();
        items.set(0, 5);
        items.set(1, 6);

        let outcome = clear_items(Some(&mut building));

        assert!(outcome.accepted);
        assert_eq!(outcome.cleared_total, 11);
        assert_eq!(building.items.as_ref().unwrap().total(), 0);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(15, 17)));
    }

    #[test]
    fn clear_items_rejects_building_without_item_module() {
        let mut building = BuildingComp::new(point2_pack(16, 18), block(), TeamId(1));

        let outcome = clear_items(Some(&mut building));

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(ItemSyncRejectReason::MissingItemStorage)
        );
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn set_liquid_updates_building_liquids_and_records_packet() {
        let mut building = BuildingComp::new(point2_pack(17, 19), liquid_block(), TeamId(1));
        building.liquids.as_mut().unwrap().set(0, 2.5);

        let outcome = set_liquid(Some(&mut building), Some("water".into()), 8.75, liquid_id);

        assert!(outcome.accepted);
        assert_eq!(outcome.previous_amount, 2.5);
        assert_eq!(outcome.new_amount, 8.75);
        assert_eq!(building.liquids.as_ref().unwrap().get(0), 8.75);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(17, 19)));
        assert_eq!(packet.liquid.as_deref(), Some("water"));
        assert_eq!(packet.amount, 8.75);
    }

    #[test]
    fn set_liquids_applies_all_stacks_without_clearing_absent_liquids() {
        let mut building = BuildingComp::new(point2_pack(18, 20), liquid_block(), TeamId(1));
        building.liquids.as_mut().unwrap().set(2, 9.0);

        let stacks = vec![
            LiquidStack::new("water", 1.5),
            LiquidStack::new("slag", 2.25),
        ];
        let outcome = set_liquids(Some(&mut building), stacks.clone(), liquid_id);

        assert!(outcome.accepted);
        assert_eq!(outcome.applied_entries, 2);
        let liquids = building.liquids.as_ref().unwrap();
        assert_eq!(liquids.get(0), 1.5);
        assert_eq!(liquids.get(1), 2.25);
        assert_eq!(liquids.get(2), 9.0);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(18, 20)));
        assert_eq!(packet.liquids, stacks);
    }

    #[test]
    fn set_liquids_rejects_unknown_liquid_without_partial_mutation() {
        let mut building = BuildingComp::new(point2_pack(19, 21), liquid_block(), TeamId(1));
        building.liquids.as_mut().unwrap().set(0, 5.0);

        let outcome = set_liquids(
            Some(&mut building),
            vec![
                LiquidStack::new("water", 1.0),
                LiquidStack::new("missing", 2.0),
            ],
            liquid_id,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(LiquidSyncRejectReason::UnknownLiquid)
        );
        assert_eq!(building.liquids.as_ref().unwrap().get(0), 5.0);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn clear_liquids_clears_module_and_records_packet() {
        let mut building = BuildingComp::new(point2_pack(20, 22), liquid_block(), TeamId(1));
        let liquids = building.liquids.as_mut().unwrap();
        liquids.set(0, 5.0);
        liquids.set(1, 6.0);

        let outcome = clear_liquids(Some(&mut building));

        assert!(outcome.accepted);
        assert_eq!(outcome.cleared_current, Some(1));
        assert_eq!(outcome.cleared_amount, 6.0);
        assert_eq!(building.liquids.as_ref().unwrap().current(), None);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(20, 22)));
    }

    #[test]
    fn clear_liquids_rejects_building_without_liquid_module() {
        let mut building = BuildingComp::new(point2_pack(21, 23), block(), TeamId(1));

        let outcome = clear_liquids(Some(&mut building));

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(LiquidSyncRejectReason::MissingLiquidStorage)
        );
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn transfer_item_effect_records_packet_and_rejects_null_target() {
        let outcome =
            transfer_item_effect(Some("copper".into()), 12.5, 24.25, Some(EntityRef::new(90)));

        assert!(outcome.accepted);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!((packet.x, packet.y), (12.5, 24.25));
        assert_eq!(packet.to, EntityRef::new(90));

        let rejected =
            transfer_item_effect(Some("copper".into()), 0.0, 0.0, Some(EntityRef::null()));
        assert!(!rejected.accepted);
        assert_eq!(
            rejected.rejection,
            Some(TransferItemEffectRejectReason::MissingTarget)
        );
        assert!(rejected.packet.is_none());
    }

    #[test]
    fn take_items_removes_from_building_adds_to_unit_and_plans_effects() {
        let mut building = BuildingComp::new(point2_pack(22, 24), item_block(), TeamId(1));
        building.items.as_mut().unwrap().set(0, 8);
        let mut unit = UnitComp::new(46, unit_type(5), TeamId(1));
        unit.items.add_item_amount("copper", 2);

        let outcome = take_items(
            Some(&mut building),
            Some("copper".into()),
            6,
            Some(&mut unit),
            item_id,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.requested_amount, 6);
        assert_eq!(outcome.removed_amount, 3);
        assert_eq!(
            outcome.remove_stack,
            Some(ItemRemoveStackPlan {
                build: BuildingRef::new(point2_pack(22, 24)),
                item: Some("copper".into()),
                item_id: 0,
                amount_removed: 3,
                source_is_core: false,
                source_is_storage: false,
            })
        );
        assert_eq!(building.items.as_ref().unwrap().get(0), 5);
        assert_eq!(unit.items.item(), Some("copper"));
        assert_eq!(unit.items.stack.amount, 5);
        assert_eq!(outcome.transfer_effects.len(), 1);
        assert_eq!(outcome.transfer_effects[0].to, EntityRef::new(46));
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.build, BuildingRef::new(point2_pack(22, 24)));
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!(packet.amount, 3);
        assert_eq!(packet.to, UnitRef::Unit { id: 46 });
    }

    #[test]
    fn take_items_records_core_remove_stack_side_effect_plan() {
        let mut core_block = item_block();
        core_block.flags.push(BlockFlag::Core);
        let core_tile = point2_pack(31, 24);
        let mut building = BuildingComp::new(core_tile, core_block, TeamId(1));
        building.items.as_mut().unwrap().set(0, 8);
        let mut unit = UnitComp::new(49, unit_type(10), TeamId(1));

        let outcome = take_items(
            Some(&mut building),
            Some("copper".into()),
            4,
            Some(&mut unit),
            item_id,
        );

        assert!(outcome.accepted);
        assert_eq!(
            outcome.remove_stack,
            Some(ItemRemoveStackPlan {
                build: BuildingRef::new(core_tile),
                item: Some("copper".into()),
                item_id: 0,
                amount_removed: 4,
                source_is_core: true,
                source_is_storage: false,
            })
        );
        assert_eq!(building.items.as_ref().unwrap().get(0), 4);
        assert_eq!(unit.items.stack.amount, 4);
    }

    #[test]
    fn take_items_rejects_when_unit_cannot_accept_without_mutating() {
        let mut building = BuildingComp::new(point2_pack(23, 25), item_block(), TeamId(1));
        building.items.as_mut().unwrap().set(0, 8);
        let mut unit = UnitComp::new(47, unit_type(2), TeamId(1));
        unit.items.add_item_amount("copper", 2);

        let outcome = take_items(
            Some(&mut building),
            Some("copper".into()),
            4,
            Some(&mut unit),
            item_id,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(TakeItemsRejectReason::NothingRemoved)
        );
        assert_eq!(building.items.as_ref().unwrap().get(0), 8);
        assert_eq!(unit.items.stack.amount, 2);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn transfer_item_to_unit_adds_one_item_and_records_packet() {
        let mut unit = UnitComp::new(48, unit_type(4), TeamId(1));
        unit.set_pos(30.0, 31.0);

        let outcome = transfer_item_to_unit(Some("lead".into()), 12.0, 13.0, Some(&mut unit), None);

        assert!(outcome.accepted);
        assert!(outcome.item_added);
        assert_eq!(unit.items.item(), Some("lead"));
        assert_eq!(unit.items.stack.amount, 1);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.item.as_deref(), Some("lead"));
        assert_eq!((packet.x, packet.y), (12.0, 13.0));
        assert_eq!(packet.to, EntityRef::new(48));
    }

    #[test]
    fn set_tile_items_applies_best_effort_by_positions() {
        let pos_a = point2_pack(24, 26);
        let pos_b = point2_pack(25, 26);
        let pos_c = point2_pack(26, 26);
        let missing = point2_pack(27, 26);
        let mut buildings = vec![
            BuildingComp::new(pos_a, item_block(), TeamId(1)),
            BuildingComp::new(pos_b, block(), TeamId(1)),
            BuildingComp::new(pos_c, item_block(), TeamId(1)),
        ];
        buildings[0].items.as_mut().unwrap().set(0, 1);
        buildings[2].items.as_mut().unwrap().set(0, 2);

        let positions = vec![pos_a, pos_b, missing, pos_c];
        let outcome = set_tile_items(
            &mut buildings,
            Some("copper".into()),
            9,
            positions.clone(),
            item_id,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.applied_positions, 2);
        assert_eq!(buildings[0].items.as_ref().unwrap().get(0), 9);
        assert_eq!(buildings[2].items.as_ref().unwrap().get(0), 9);
        assert!(buildings[1].items.is_none());
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!(packet.amount, 9);
        assert_eq!(packet.positions, positions);
    }

    #[test]
    fn set_tile_liquids_applies_best_effort_by_positions() {
        let pos_a = point2_pack(28, 26);
        let pos_b = point2_pack(29, 26);
        let pos_c = point2_pack(30, 26);
        let mut buildings = vec![
            BuildingComp::new(pos_a, liquid_block(), TeamId(1)),
            BuildingComp::new(pos_b, block(), TeamId(1)),
            BuildingComp::new(pos_c, liquid_block(), TeamId(1)),
        ];

        let positions = vec![pos_a, pos_b, pos_c];
        let outcome = set_tile_liquids(
            &mut buildings,
            Some("water".into()),
            7.25,
            positions.clone(),
            liquid_id,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.applied_positions, 2);
        assert_eq!(buildings[0].liquids.as_ref().unwrap().get(0), 7.25);
        assert_eq!(buildings[2].liquids.as_ref().unwrap().get(0), 7.25);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.liquid.as_deref(), Some("water"));
        assert_eq!(packet.amount, 7.25);
        assert_eq!(packet.positions, positions);
    }

    #[test]
    fn transfer_item_to_deducts_matching_unit_stack_and_adds_to_building() {
        let mut unit = UnitComp::new(49, unit_type(10), TeamId(1));
        unit.items.add_item_amount("copper", 7);
        let mut building = BuildingComp::new(point2_pack(31, 26), item_block(), TeamId(1));
        building.items.as_mut().unwrap().set(0, 1);

        let outcome = transfer_item_to(
            Some(&mut unit),
            Some("copper".into()),
            5,
            11.0,
            12.0,
            Some(&mut building),
            item_id,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.unit_previous_amount, Some(7));
        assert_eq!(outcome.unit_new_amount, Some(2));
        assert_eq!(unit.items.stack.amount, 2);
        assert_eq!(outcome.building_previous_amount, 1);
        assert_eq!(outcome.building_new_amount, 6);
        assert_eq!(building.items.as_ref().unwrap().get(0), 6);
        assert_eq!(outcome.effect_count, 1);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 49 });
        assert_eq!(packet.item.as_deref(), Some("copper"));
        assert_eq!(packet.amount, 5);
        assert_eq!((packet.x, packet.y), (11.0, 12.0));
        assert_eq!(packet.build, BuildingRef::new(point2_pack(31, 26)));
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

    #[test]
    fn delete_team_plans_removes_from_team_queue_and_records_packet() {
        use crate::mindustry::game::{BlockPlan as TeamBlockPlan, Teams, TEAM_SHARDED};

        let mut teams = Teams::default();
        teams.replace_plans([(
            TEAM_SHARDED,
            vec![
                TeamBlockPlan::new(1, 2, 0, "duo", None),
                TeamBlockPlan::new(3, 4, 1, "router", Some("cfg".into())),
            ],
        )]);
        let positions = vec![point2_pack(3, 4)];

        let outcome = delete_team_plans(
            DeletePlansContext {
                player_id: Some(17),
                local_player: false,
            },
            true,
            &mut teams,
            TEAM_SHARDED,
            &positions,
            |_| true,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.removed, 1);
        assert_eq!(
            outcome.packet,
            Some(DeletePlansCallPacket {
                player_id: Some(17),
                positions
            })
        );
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![TeamBlockPlan::new(1, 2, 0, "duo", None)]
        );
    }

    #[test]
    fn command_units_accepts_targets_and_records_packet() {
        let building = BuildingComp::new(point2_pack(7, 8), item_block(), TeamId(1));
        let outcome = command_units(
            CommandUnitsContext {
                player: Some(EntityRef::new(72)),
                local_player: false,
            },
            vec![1, 2, 3],
            Some(&building),
            Some(UnitRef::Unit { id: 4 }),
            Vec2::new(9.0, 10.0),
            true,
            false,
            |ids| ids == [1, 2, 3].as_slice(),
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.commanded, 3);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(72));
        assert_eq!(packet.unit_ids, vec![1, 2, 3]);
        assert_eq!(packet.build_target, BuildingRef::new(point2_pack(7, 8)));
        assert_eq!(packet.unit_target, UnitRef::Unit { id: 4 });
        assert_eq!(packet.pos_target, Vec2::new(9.0, 10.0));
        assert!(packet.queue_command);
        assert!(!packet.final_batch);
    }

    #[test]
    fn command_units_rejects_admin_denied_as_validate_error() {
        let outcome = command_units(
            CommandUnitsContext {
                player: Some(EntityRef::new(72)),
                local_player: false,
            },
            vec![1],
            None,
            None,
            Vec2::new(0.0, 0.0),
            false,
            true,
            |_| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(CommandUnitsRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn client_command_units_packet_uses_client_payload_shape() {
        let packet = client_command_units_packet(
            vec![9],
            None,
            Some(UnitRef::Unit { id: 10 }),
            Vec2::new(1.0, 2.0),
            false,
            true,
        );

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.unit_ids, vec![9]);
        assert_eq!(packet.build_target, BuildingRef::null());
        assert_eq!(packet.unit_target, UnitRef::Unit { id: 10 });
        assert!(packet.final_batch);
    }

    #[test]
    fn set_unit_command_updates_command_ai_units_and_records_packet() {
        let mut player = PlayerComp::new(TeamId(1));
        player.name = "commander".into();
        player.color = 0x1122_3344;
        let mut type_info = unit_type(10);
        type_info.commands.push("rebuild".into());
        let mut unit = UnitComp::new(80, type_info, TeamId(1));
        let mut command_wire = CommandWire::new();
        command_wire.target_pos = Some(Vec2::new(1.0, 2.0));
        unit.controller = UnitControllerState::Command(command_wire);

        let mut enemy_type = unit_type(10);
        enemy_type.commands.push("rebuild".into());
        let mut enemy = UnitComp::new(81, enemy_type, TeamId(2));
        enemy.controller = UnitControllerState::Command(CommandWire::new());
        let mut units = vec![unit, enemy];
        let command = UnitCommand::new(2, "rebuild", "hammer", None::<String>, None::<String>);

        let outcome = set_unit_command(
            SetUnitCommandContext {
                player: Some(EntityRef::new(70)),
                local_player: false,
            },
            Some(&player),
            &mut units,
            vec![80, 81],
            Some(&command),
            |ids| ids == [80, 81].as_slice(),
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.commanded, 1);
        assert_eq!(
            units[0].last_commanded.as_deref(),
            Some("[#11223344]commander")
        );
        match &units[0].controller {
            UnitControllerState::Command(command_wire) => {
                assert_eq!(command_wire.command_id, Some(2));
                assert!(command_wire.target_pos.is_none());
            }
            other => panic!("expected command controller, got {other:?}"),
        }
        assert!(units[1].last_commanded.is_none());
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(70));
        assert_eq!(packet.unit_ids, vec![80, 81]);
        assert_eq!(packet.command, "rebuild");
    }

    #[test]
    fn set_unit_command_rejects_admin_denied_as_validate_error() {
        let player = PlayerComp::new(TeamId(1));
        let mut type_info = unit_type(10);
        type_info.commands.push("move".into());
        let mut unit = UnitComp::new(80, type_info, TeamId(1));
        unit.controller = UnitControllerState::Command(CommandWire::new());
        let mut units = vec![unit];
        let command = UnitCommand::new(0, "move", "right", None::<String>, None::<String>);

        let outcome = set_unit_command(
            SetUnitCommandContext {
                player: Some(EntityRef::new(70)),
                local_player: false,
            },
            Some(&player),
            &mut units,
            vec![80],
            Some(&command),
            |_| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(SetUnitCommandRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        match &units[0].controller {
            UnitControllerState::Command(command_wire) => {
                assert!(command_wire.command_id.is_none())
            }
            other => panic!("expected command controller, got {other:?}"),
        }
    }

    #[test]
    fn set_unit_stance_toggles_allowed_stance_and_records_packet() {
        let player = PlayerComp::new(TeamId(1));
        let mut type_info = unit_type(10);
        type_info.stances.push("holdfire".into());
        let mut unit = UnitComp::new(82, type_info, TeamId(1));
        unit.controller = UnitControllerState::Command(CommandWire::new());
        let mut units = vec![unit];
        let stance = UnitStance::new(3, "holdfire", "pause", None::<String>, true);

        let enabled = set_unit_stance(
            SetUnitStanceContext {
                player: Some(EntityRef::new(71)),
                local_player: false,
            },
            Some(&player),
            &mut units,
            vec![82],
            Some(&stance),
            true,
            |_| true,
        );

        assert!(enabled.accepted);
        assert_eq!(enabled.commanded, 1);
        match &units[0].controller {
            UnitControllerState::Command(command_wire) => {
                assert_eq!(command_wire.stances, vec![3]);
            }
            other => panic!("expected command controller, got {other:?}"),
        }
        let packet = enabled.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(71));
        assert_eq!(packet.stance, "holdfire");
        assert!(packet.enable);

        let disabled = set_unit_stance(
            SetUnitStanceContext {
                player: Some(EntityRef::new(71)),
                local_player: false,
            },
            Some(&player),
            &mut units,
            vec![82],
            Some(&stance),
            false,
            |_| true,
        );

        assert!(disabled.accepted);
        match &units[0].controller {
            UnitControllerState::Command(command_wire) => assert!(command_wire.stances.is_empty()),
            other => panic!("expected command controller, got {other:?}"),
        }
    }

    #[test]
    fn unit_command_client_packets_use_client_payload_shape() {
        let command = client_set_unit_command_packet(vec![1, 2], "move");
        assert_eq!(command.player, EntityRef::null());
        assert_eq!(command.unit_ids, vec![1, 2]);
        assert_eq!(command.command, "move");

        let stance = client_set_unit_stance_packet(vec![3], "stop", false);
        assert_eq!(stance.player, EntityRef::null());
        assert_eq!(stance.unit_ids, vec![3]);
        assert_eq!(stance.stance, "stop");
        assert!(!stance.enable);
    }

    #[test]
    fn command_building_updates_commandable_same_team_buildings() {
        let mut player = PlayerComp::new(TeamId(1));
        player.name = "builder".into();
        player.color = 0xAABB_CCDD;
        let commandable = BuildingComp::new(point2_pack(11, 12), command_block(), TeamId(1));
        let enemy = BuildingComp::new(point2_pack(13, 14), command_block(), TeamId(2));
        let mut plain = BuildingComp::new(point2_pack(15, 16), block(), TeamId(1));
        plain.block.commandable = false;
        let mut builds = vec![commandable, enemy, plain];

        let outcome = command_building(
            CommandBuildingContext {
                player: Some(EntityRef::new(90)),
                local_player: false,
            },
            Some(&player),
            &mut builds,
            vec![
                point2_pack(11, 12),
                point2_pack(13, 14),
                point2_pack(15, 16),
            ],
            Vec2::new(4.0, 5.0),
            |_| true,
        );

        assert!(outcome.accepted);
        assert_eq!(outcome.commanded_positions, vec![point2_pack(11, 12)]);
        assert_eq!(builds[0].last_accessed, "[#AABBCCDD]builder");
        assert!(builds[1].last_accessed.is_empty());
        assert!(builds[2].last_accessed.is_empty());
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.player, EntityRef::new(90));
        assert_eq!(
            packet.buildings,
            vec![
                point2_pack(11, 12),
                point2_pack(13, 14),
                point2_pack(15, 16)
            ]
        );
        assert_eq!(packet.target, Vec2::new(4.0, 5.0));
    }

    #[test]
    fn command_building_rejects_admin_denied_as_validate_error() {
        let player = PlayerComp::new(TeamId(1));
        let mut builds = vec![BuildingComp::new(
            point2_pack(11, 12),
            command_block(),
            TeamId(1),
        )];

        let outcome = command_building(
            CommandBuildingContext {
                player: Some(EntityRef::new(90)),
                local_player: false,
            },
            Some(&player),
            &mut builds,
            vec![point2_pack(11, 12)],
            Vec2::new(4.0, 5.0),
            |_| false,
        );

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(CommandBuildingRejectReason::AdminDenied)
        );
        assert!(outcome.should_raise_validate);
        assert!(builds[0].last_accessed.is_empty());
    }

    #[test]
    fn client_command_building_packet_uses_client_payload_shape() {
        let packet = client_command_building_packet(vec![1, 2], Vec2::new(3.0, 4.0));

        assert_eq!(packet.player, EntityRef::null());
        assert_eq!(packet.buildings, vec![1, 2]);
        assert_eq!(packet.target, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn remove_queue_block_removes_matching_plan_and_records_packet() {
        let mut unit = UnitComp::new(80, unit_type(10), TeamId(1));
        let keep = BuildPlan::new_place(1, 2, 0, "router");
        let remove = BuildPlan::new_break(3, 4);
        unit.builder.plans.push_back(keep.clone());
        unit.builder.plans.push_back(remove.clone());

        let outcome = remove_queue_block(Some(&mut unit), 3, 4, true);

        assert!(outcome.accepted);
        assert_eq!(outcome.removed, Some(remove));
        assert_eq!(unit.builder.plans.len(), 1);
        assert_eq!(unit.builder.plans.front(), Some(&keep));
        let packet = outcome.packet.unwrap();
        assert_eq!(
            packet,
            RemoveQueueBlockCallPacket {
                x: 3,
                y: 4,
                breaking: true
            }
        );
    }

    #[test]
    fn remove_queue_block_keeps_queue_when_plan_missing() {
        let mut unit = UnitComp::new(81, unit_type(10), TeamId(1));
        let keep = BuildPlan::new_place(5, 6, 0, "router");
        unit.builder.plans.push_back(keep.clone());

        let outcome = remove_queue_block(Some(&mut unit), 7, 8, false);

        assert!(outcome.accepted);
        assert!(outcome.removed.is_none());
        assert_eq!(unit.builder.plans.front(), Some(&keep));
    }

    #[test]
    fn remove_queue_block_rejects_missing_unit_without_packet() {
        let outcome = remove_queue_block(None, 1, 2, false);

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(RemoveQueueBlockRejectReason::MissingUnit)
        );
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn remove_queue_block_packet_uses_java_payload_shape() {
        assert_eq!(
            remove_queue_block_packet(9, 10, true),
            RemoveQueueBlockCallPacket {
                x: 9,
                y: 10,
                breaking: true
            }
        );
    }

    #[test]
    fn rotate_build_plans_rotates_coordinates_rotation_and_point_configs() {
        let info = BuildPlanBlockTransform::new(2, 8.0, true, false, false);
        let plan = BuildPlan::new_config(
            3,
            1,
            1,
            "junction",
            TypeValue::Point2Array(vec![Point2::new(1, 0), Point2::new(0, 2)]),
        );
        let breaking = BuildPlan::new_break(9, 9);

        let rotated =
            rotate_build_plans(&[plan.clone(), breaking.clone()], 1, 1, 1, |_| Some(info));

        assert_eq!(rotated[0].x, -1);
        assert_eq!(rotated[0].y, 3);
        assert_eq!(rotated[0].rotation, 2);
        assert_eq!(
            rotated[0].config,
            TypeValue::Point2Array(vec![Point2::new(1, 1), Point2::new(-1, 0)])
        );
        assert_eq!(rotated[1], breaking);
    }

    #[test]
    fn rotate_build_plans_respects_locked_non_rotating_blocks() {
        let locked = BuildPlanBlockTransform::new(1, 4.0, false, true, false);
        let plan = BuildPlan::new_place(2, 1, 3, "core");

        let rotated = rotate_build_plans(&[plan], 1, 1, -1, |_| Some(locked));

        assert_eq!(rotated[0].rotation, 0);
    }

    #[test]
    fn flip_build_plans_flips_coordinates_config_and_rotation() {
        let info = BuildPlanBlockTransform::new(2, 8.0, true, false, false);
        let plan = BuildPlan::new_config(3, 2, 0, "bridge", TypeValue::Point2(Point2::new(2, 1)));

        let flipped = flip_build_plans(&[plan], 1, 1, true, |_| Some(info));

        assert_eq!(flipped[0].x, -3);
        assert_eq!(flipped[0].y, 2);
        assert_eq!(flipped[0].rotation, 2);
        assert_eq!(flipped[0].config, TypeValue::Point2(Point2::new(-1, 1)));
    }

    #[test]
    fn flip_build_plans_supports_y_axis_and_inverted_flip_rule() {
        let inverted = BuildPlanBlockTransform::new(1, 4.0, true, false, true);
        let plan = BuildPlan::new_config(
            2,
            4,
            1,
            "sorter",
            TypeValue::Point2Array(vec![Point2::new(1, 2)]),
        );

        let flipped = flip_build_plans(&[plan], 1, 1, false, |_| Some(inverted));

        assert_eq!(flipped[0].x, 2);
        assert_eq!(flipped[0].y, -3);
        assert_eq!(flipped[0].rotation, 1);
        assert_eq!(
            flipped[0].config,
            TypeValue::Point2Array(vec![Point2::new(1, -2)])
        );
    }

    #[test]
    fn valid_place_plan_combines_base_validity_and_queued_plan_conflicts() {
        let candidate = BuildPlanBlockTransform::new(2, 8.0, true, false, false);
        let queued = QueuedBuildPlanSnapshot::new(
            BuildPlan::new_place(4, 4, 0, "duo"),
            BuildPlanBlockTransform::single(),
        );

        assert!(!valid_place_plan(ValidPlaceFrame {
            base_valid: false,
            player_is_builder: true,
            x: 4,
            y: 4,
            block: candidate,
            queued_plans: vec![queued.clone()],
        }));

        assert!(valid_place_plan(ValidPlaceFrame {
            base_valid: true,
            player_is_builder: false,
            x: 4,
            y: 4,
            block: candidate,
            queued_plans: vec![queued.clone()],
        }));

        assert!(!valid_place_plan(ValidPlaceFrame {
            base_valid: true,
            player_is_builder: true,
            x: 4,
            y: 4,
            block: candidate,
            queued_plans: vec![queued],
        }));

        assert!(valid_place_plan(ValidPlaceFrame {
            base_valid: true,
            player_is_builder: true,
            x: 4,
            y: 4,
            block: candidate,
            queued_plans: vec![QueuedBuildPlanSnapshot::new(
                BuildPlan::new_place(10, 10, 0, "duo"),
                BuildPlanBlockTransform::single(),
            )],
        }));
    }

    #[test]
    fn valid_place_plan_respects_ignore_breaking_and_replace_exceptions() {
        let candidate = BuildPlanBlockTransform::new(2, 8.0, true, false, false);
        let ignored = QueuedBuildPlanSnapshot::new(
            BuildPlan::new_place(2, 2, 0, "router"),
            BuildPlanBlockTransform::single(),
        )
        .ignored();
        let breaking = QueuedBuildPlanSnapshot::new(
            BuildPlan::new_break(2, 2),
            BuildPlanBlockTransform::single(),
        );
        let replaceable = QueuedBuildPlanSnapshot::new(
            BuildPlan::new_place(2, 2, 0, "router"),
            BuildPlanBlockTransform::single(),
        )
        .replaceable_by_candidate();

        assert!(valid_place_plan(ValidPlaceFrame {
            base_valid: true,
            player_is_builder: true,
            x: 2,
            y: 2,
            block: candidate,
            queued_plans: vec![ignored, breaking, replaceable],
        }));

        assert!(!valid_place_plan(ValidPlaceFrame {
            base_valid: true,
            player_is_builder: true,
            x: 2,
            y: 2,
            block: candidate,
            queued_plans: vec![QueuedBuildPlanSnapshot::new(
                BuildPlan::new_place(2, 3, 0, "wall"),
                BuildPlanBlockTransform::single(),
            )
            .replaceable_by_candidate()],
        }));
    }

    #[test]
    fn break_block_plan_targets_build_origin_and_obeys_valid_break_gate() {
        assert_eq!(
            break_block_plan(BreakBlockFrame {
                player_is_builder: true,
                tile: Some(BreakBlockTileSnapshot::new(7, 8).with_build_origin(6, 6)),
            }),
            Some(BuildPlan::new_break(6, 6))
        );

        assert_eq!(
            break_block_plan(BreakBlockFrame {
                player_is_builder: false,
                tile: Some(BreakBlockTileSnapshot::new(7, 8)),
            }),
            None
        );

        assert_eq!(
            try_break_block_plan(TryBreakBlockFrame {
                valid_break: false,
                break_frame: BreakBlockFrame {
                    player_is_builder: true,
                    tile: Some(BreakBlockTileSnapshot::new(7, 8)),
                },
            }),
            None
        );

        assert_eq!(
            try_break_block_plan(TryBreakBlockFrame {
                valid_break: true,
                break_frame: BreakBlockFrame {
                    player_is_builder: true,
                    tile: Some(BreakBlockTileSnapshot::new(7, 8)),
                },
            }),
            Some(BuildPlan::new_break(7, 8))
        );
    }

    #[test]
    fn rebuild_area_plan_requeues_overlapping_broken_plans_and_dedupes_repairs() {
        let footprint = BuildPlanBlockTransform::single();
        let router = RebuildBlockPlanSnapshot::new(
            2,
            2,
            1,
            "router",
            TypeValue::String("cfg".into()),
            footprint,
        );
        let outside = RebuildBlockPlanSnapshot::new(20, 20, 0, "duo", TypeValue::Null, footprint);
        let repair = BuildPlan::new_config(3, 3, 2, "duo", TypeValue::Int(9));

        let plan = rebuild_area_plan(RebuildAreaFrame {
            x1: 1,
            y1: 1,
            x2: 4,
            y2: 4,
            rotation: 0,
            broken_plans: vec![router, outside],
            repair_candidates: vec![
                RebuildRepairCandidate::new(3, 3, point2_pack(3, 3), true, repair.clone()),
                RebuildRepairCandidate::new(3, 4, point2_pack(3, 3), true, repair.clone()),
                RebuildRepairCandidate::new(
                    4,
                    4,
                    point2_pack(4, 4),
                    false,
                    BuildPlan::new_place(4, 4, 0, "duo"),
                ),
                RebuildRepairCandidate::new(
                    8,
                    8,
                    point2_pack(8, 8),
                    true,
                    BuildPlan::new_place(8, 8, 0, "duo"),
                ),
            ],
            ..RebuildAreaFrame::default()
        });

        assert_eq!(
            plan.selection,
            RebuildAreaSelection {
                x: 1,
                y: 1,
                x2: 4,
                y2: 4,
                rotation: 0
            }
        );
        assert_eq!(plan.rebuild_plans.len(), 1);
        assert_eq!(plan.rebuild_plans[0].block.as_deref(), Some("router"));
        assert_eq!(
            plan.rebuild_plans[0].config,
            TypeValue::String("cfg".into())
        );
        assert_eq!(plan.repair_plans, vec![repair]);
        assert_eq!(plan.repair_tile_positions, vec![point2_pack(3, 3)]);
    }

    #[test]
    fn flush_build_plans_plan_filters_invalid_and_marks_queue_position() {
        let footprint = BuildPlanBlockTransform::single();
        let first =
            FlushBuildPlanCandidate::new(BuildPlan::new_place(1, 1, 0, "router"), footprint, true);
        let invalid =
            FlushBuildPlanCandidate::new(BuildPlan::new_place(2, 2, 0, "router"), footprint, false);
        let breaking = FlushBuildPlanCandidate::new(BuildPlan::new_break(3, 3), footprint, true);
        let second =
            FlushBuildPlanCandidate::new(BuildPlan::new_place(4, 4, 1, "duo"), footprint, true);

        let forward = flush_build_plans_plan(
            &[first.clone(), invalid.clone(), breaking, second.clone()],
            false,
        );
        assert_eq!(forward.len(), 2);
        assert_eq!(forward[0].plan.block.as_deref(), Some("router"));
        assert_eq!(forward[0].position, BuildQueuePosition::Tail);
        assert!(forward[0].call_on_new_plan);
        assert!(forward[0].insert_into_plan_tree);
        assert_eq!(forward[1].plan.block.as_deref(), Some("duo"));

        let reverse = flush_build_plans_plan(&[first, invalid, second], true);
        assert_eq!(reverse.len(), 2);
        assert_eq!(reverse[0].plan.block.as_deref(), Some("duo"));
        assert_eq!(reverse[0].position, BuildQueuePosition::Head);
        assert_eq!(reverse[1].plan.block.as_deref(), Some("router"));
    }

    #[test]
    fn flush_select_plans_adds_replaces_and_skips_like_java_selection_buffer() {
        let footprint = BuildPlanBlockTransform::single();
        let existing = BuildPlanSnapshot::new(BuildPlan::new_place(1, 1, 0, "router"), footprint);
        let blocking_unit =
            BuildPlanSnapshot::new(BuildPlan::new_place(6, 6, 0, "router"), footprint);

        let add =
            FlushBuildPlanCandidate::new(BuildPlan::new_place(3, 3, 0, "router"), footprint, true);
        let replace =
            FlushBuildPlanCandidate::new(BuildPlan::new_place(1, 1, 2, "duo"), footprint, true);
        let blocked_by_unit =
            FlushBuildPlanCandidate::new(BuildPlan::new_place(6, 6, 0, "duo"), footprint, true);
        let invalid =
            FlushBuildPlanCandidate::new(BuildPlan::new_place(9, 9, 0, "duo"), footprint, false);

        let plan = flush_select_plans_plan(
            &[add, replace, blocked_by_unit, invalid],
            &[blocking_unit],
            &[existing],
        );

        assert_eq!(
            plan.actions,
            vec![
                FlushSelectPlanAction::Add {
                    plan: BuildPlan::new_place(3, 3, 0, "router")
                },
                FlushSelectPlanAction::ReplaceSelect {
                    index: 0,
                    plan: BuildPlan::new_place(1, 1, 2, "duo")
                }
            ]
        );
        assert_eq!(plan.skipped, 2);
        assert_eq!(
            plan.final_select_plans
                .iter()
                .map(|plan| (plan.x, plan.y, plan.block.as_deref(), plan.rotation))
                .collect::<Vec<_>>(),
            vec![(3, 3, Some("router"), 0), (1, 1, Some("duo"), 2)]
        );
    }

    #[test]
    fn remove_selection_plan_immediate_breaks_and_cleans_team_rebuilds() {
        use crate::mindustry::game::{BlockPlan as TeamBlockPlan, Teams, TEAM_SHARDED};

        let footprint = BuildPlanBlockTransform::single();
        let tile = RemoveSelectionTileCandidate {
            break_tile: BreakBlockTileSnapshot::new(2, 2).with_build_origin(1, 1),
            ..RemoveSelectionTileCandidate::new(2, 2, 1, 1, true)
        };
        let unit_plan = BuildPlanSnapshot::new(BuildPlan::new_place(2, 2, 0, "router"), footprint);
        let select_plan = BuildPlanSnapshot::new(BuildPlan::new_place(2, 2, 0, "duo"), footprint);

        let plan = remove_selection_plan(RemoveSelectionFrame {
            x1: 2,
            y1: 2,
            x2: 3,
            y2: 3,
            player_is_builder: true,
            net_active: true,
            world_tiles: vec![tile],
            unit_plans: vec![unit_plan],
            select_plans: vec![select_plan],
            team_plans: vec![RemoveSelectionTeamPlanSnapshot::new(2, 2, footprint)],
            ..RemoveSelectionFrame::default()
        });

        assert_eq!(plan.immediate_break_plans, vec![BuildPlan::new_break(1, 1)]);
        assert!(plan.queued_break_plans.is_empty());
        assert_eq!(plan.remove_unit_plan_indices, vec![0]);
        assert!(plan.remove_select_plan_indices.is_empty());
        assert_eq!(plan.remove_team_plan_indices, vec![0]);
        assert_eq!(plan.removed_team_plan_positions, vec![point2_pack(2, 2)]);
        assert_eq!(plan.network_delete_positions, vec![point2_pack(2, 2)]);

        let mut teams = Teams::default();
        teams.replace_plans([(
            TEAM_SHARDED,
            vec![
                TeamBlockPlan::new(2, 2, 0, "router", None),
                TeamBlockPlan::new(4, 4, 0, "duo", None),
            ],
        )]);

        let removed = apply_removed_team_plan_positions(
            &mut teams,
            TEAM_SHARDED,
            &plan.removed_team_plan_positions,
        );

        assert_eq!(removed.len(), 1);
        assert!(removed[0].removed);
        assert_eq!(
            teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![TeamBlockPlan::new(4, 4, 0, "duo", None)]
        );
    }

    #[test]
    fn remove_selection_plan_flush_queues_breaks_and_removes_select_overlaps() {
        let footprint = BuildPlanBlockTransform::java_block(1);
        let selected_break = BuildPlanSnapshot::new(BuildPlan::new_break(1, 1), footprint);
        let stale_select =
            BuildPlanSnapshot::new(BuildPlan::new_place(3, 3, 0, "router"), footprint);

        let plan = remove_selection_plan(RemoveSelectionFrame {
            x1: 1,
            y1: 1,
            x2: 3,
            y2: 3,
            flush: true,
            player_is_builder: true,
            world_tiles: vec![
                RemoveSelectionTileCandidate::new(1, 1, 1, 1, true),
                RemoveSelectionTileCandidate::new(2, 2, 2, 2, true),
            ],
            select_plans: vec![selected_break, stale_select],
            ..RemoveSelectionFrame::default()
        });

        assert!(plan.immediate_break_plans.is_empty());
        assert_eq!(plan.queued_break_plans, vec![BuildPlan::new_break(2, 2)]);
        assert_eq!(plan.remove_select_plan_indices, vec![1]);
        assert!(plan.network_delete_positions.is_empty());
    }

    #[test]
    fn iterate_line_plan_normalizes_line_and_rotates_toward_next_point() {
        let block = LinePlacementBlock::new("router", BuildPlanBlockTransform::java_block(1));
        let plan = iterate_line_plan(IterateLineFrame {
            start_x: 0,
            start_y: 0,
            end_x: 0,
            end_y: 2,
            rotation: 0,
            override_line_rotation: false,
            block: Some(block),
            ..IterateLineFrame::default()
        });

        assert_eq!(
            plan.points,
            vec![Point2::new(0, 0), Point2::new(0, 1), Point2::new(0, 2)]
        );
        assert_eq!(
            plan.lines,
            vec![
                PlaceLine {
                    x: 0,
                    y: 0,
                    rotation: 1,
                    last: false,
                },
                PlaceLine {
                    x: 0,
                    y: 1,
                    rotation: 1,
                    last: false,
                },
                PlaceLine {
                    x: 0,
                    y: 2,
                    rotation: 1,
                    last: true,
                },
            ]
        );
    }

    #[test]
    fn iterate_line_plan_supports_rectangle_mode_and_multiblock_overlap_filter() {
        let mut block = LinePlacementBlock::new("large", BuildPlanBlockTransform::java_block(2));
        block.allow_rectangle_placement = true;

        let plan = iterate_line_plan(IterateLineFrame {
            start_x: 0,
            start_y: 0,
            end_x: 3,
            end_y: 3,
            rotation: 2,
            override_line_rotation: true,
            block: Some(block),
            ..IterateLineFrame::default()
        });

        assert_eq!(
            plan.points,
            vec![
                Point2::new(0, 0),
                Point2::new(2, 0),
                Point2::new(0, 2),
                Point2::new(2, 2)
            ]
        );
        assert_eq!(
            plan.lines
                .iter()
                .map(|line| (line.x, line.y, line.rotation))
                .collect::<Vec<_>>(),
            vec![(0, 0, 2), (2, 0, 2), (0, 2, 2), (2, 2, 2)]
        );
    }

    #[test]
    fn iterate_line_plan_uses_diagonal_path_and_chain_end_rotation() {
        let mut block = LinePlacementBlock::new("conveyor", BuildPlanBlockTransform::java_block(1));
        block.allow_diagonal = true;
        block.conveyor_placement = true;

        let plan = iterate_line_plan(IterateLineFrame {
            start_x: 0,
            start_y: 0,
            end_x: 2,
            end_y: 2,
            rotation: 0,
            override_line_rotation: true,
            diagonal_pressed: true,
            block: Some(block),
            end_build: ChainedBuildEndpoint {
                chained: true,
                rotation: 2,
                candidate_can_replace: false,
            },
            second_to_last_chained: false,
            astar_path: Some(vec![
                Point2::new(0, 0),
                Point2::new(1, 0),
                Point2::new(2, 2),
            ]),
            conveyor_pathfinding: true,
            ..IterateLineFrame::default()
        });

        assert!(plan.diagonal);
        assert_eq!(plan.end_rotation, Some(2));
        assert_eq!(
            plan.lines
                .iter()
                .map(|line| (line.x, line.y, line.rotation, line.last))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0, false), (1, 0, 0, false), (2, 2, 2, true)]
        );
    }

    #[test]
    fn update_line_plan_builds_line_plans_and_applies_unlocked_replacements() {
        let block = LinePlacementBlock::new("router", BuildPlanBlockTransform::java_block(1));
        let plan = update_line_plan(UpdateLineFrame {
            line: IterateLineFrame {
                start_x: 0,
                start_y: 0,
                end_x: 2,
                end_y: 0,
                rotation: 1,
                override_line_rotation: false,
                block: Some(block),
                ..IterateLineFrame::default()
            },
            next_config: TypeValue::Int(7),
            block_replace: true,
            replacements: vec![
                LineReplacement::new(1, 0, "junction", false),
                LineReplacement::new(2, 0, "overflow-gate", true),
            ],
        });

        assert!(plan.handle_placement_line);
        assert_eq!(plan.final_rotation, 0);
        assert_eq!(
            plan.line_plans
                .iter()
                .map(|plan| (
                    plan.x,
                    plan.y,
                    plan.rotation,
                    plan.block.as_deref(),
                    plan.anim_scale
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 0, 0, Some("router"), 1.0),
                (1, 0, 0, Some("router"), 1.0),
                (2, 0, 0, Some("overflow-gate"), 1.0),
            ]
        );
        assert!(plan
            .line_plans
            .iter()
            .all(|plan| plan.config == TypeValue::Int(7)));
    }

    #[test]
    fn tap_player_plan_requires_range_stack_alive_and_no_selected_block() {
        let accepted = try_tap_player_plan(TapPlayerFrame {
            within_select_range: true,
            stack_amount: 1,
            ..TapPlayerFrame::default()
        });
        assert!(accepted.accepted);
        assert!(accepted.dropping_item);

        for frame in [
            TapPlayerFrame {
                within_select_range: false,
                stack_amount: 1,
                ..TapPlayerFrame::default()
            },
            TapPlayerFrame {
                within_select_range: true,
                player_dead: true,
                stack_amount: 1,
                ..TapPlayerFrame::default()
            },
            TapPlayerFrame {
                within_select_range: true,
                stack_amount: 0,
                ..TapPlayerFrame::default()
            },
            TapPlayerFrame {
                within_select_range: true,
                stack_amount: 1,
                block_selected: true,
                ..TapPlayerFrame::default()
            },
        ] {
            assert!(!try_tap_player_plan(frame).accepted);
        }
    }

    #[test]
    fn mine_plans_follow_java_gates_and_tile_specific_stop() {
        assert!(can_mine_plan(CanMineFrame {
            unit_valid_mine: true,
            unit_accepts_mine_result: true,
            ..CanMineFrame::default()
        }));

        assert!(!can_mine_plan(CanMineFrame {
            scene_has_mouse: true,
            unit_valid_mine: true,
            unit_accepts_mine_result: true,
            ..CanMineFrame::default()
        }));
        assert!(!can_mine_plan(CanMineFrame {
            unit_valid_mine: true,
            unit_accepts_mine_result: true,
            floor_player_unmineable: true,
            overlay_has_item_drop: false,
            ..CanMineFrame::default()
        }));
        assert!(can_mine_plan(CanMineFrame {
            unit_valid_mine: true,
            unit_accepts_mine_result: true,
            floor_player_unmineable: true,
            overlay_has_item_drop: false,
            double_tap_mine: true,
            ..CanMineFrame::default()
        }));
        assert!(!can_mine_plan(CanMineFrame {
            unit_valid_mine: true,
            unit_accepts_mine_result: true,
            overlay_player_unmineable: true,
            overlay_has_item_drop: true,
            ..CanMineFrame::default()
        }));

        assert_eq!(
            try_begin_mine_plan(BeginMineFrame {
                can_mine: true,
                tile_pos: Some(point2_pack(4, 5)),
                ..BeginMineFrame::default()
            }),
            BeginMinePlan {
                accepted: true,
                mine_tile: Some(point2_pack(4, 5)),
            }
        );
        assert!(
            !try_begin_mine_plan(BeginMineFrame {
                player_dead: true,
                can_mine: true,
                tile_pos: Some(point2_pack(4, 5)),
            })
            .accepted
        );

        assert_eq!(
            try_stop_mine_plan(StopMineFrame {
                current_mine_tile: Some(point2_pack(1, 2)),
                requested_tile: None,
                player_dead: false,
            }),
            StopMinePlan {
                accepted: true,
                clear_mine_tile: true,
            }
        );
        assert!(
            !try_stop_mine_plan(StopMineFrame {
                current_mine_tile: Some(point2_pack(1, 2)),
                requested_tile: Some(point2_pack(2, 1)),
                player_dead: false,
            })
            .accepted
        );
        assert!(
            try_stop_mine_plan(StopMineFrame {
                current_mine_tile: Some(point2_pack(1, 2)),
                requested_tile: Some(point2_pack(1, 2)),
                player_dead: false,
            })
            .accepted
        );
    }

    #[test]
    fn repair_derelict_plan_requires_derelict_build_and_valid_place() {
        let accepted = try_repair_derelict_plan(RepairDerelictFrame {
            tile_present: true,
            build_present: true,
            build_team_derelict: true,
            block_unlocked_host: true,
            valid_place: true,
            build_x: 7,
            build_y: 8,
            build_rotation: 2,
            block: Some("duo".into()),
            config: TypeValue::String("cfg".into()),
            ..RepairDerelictFrame::default()
        });

        assert!(accepted.accepted);
        let plan = accepted.build_plan.unwrap();
        assert_eq!(plan.x, 7);
        assert_eq!(plan.y, 8);
        assert_eq!(plan.rotation, 2);
        assert_eq!(plan.block.as_deref(), Some("duo"));
        assert_eq!(plan.config, TypeValue::String("cfg".into()));

        for frame in [
            RepairDerelictFrame {
                tile_present: false,
                build_present: true,
                build_team_derelict: true,
                block_unlocked_host: true,
                valid_place: true,
                block: Some("duo".into()),
                ..RepairDerelictFrame::default()
            },
            RepairDerelictFrame {
                tile_present: true,
                build_present: true,
                player_dead: true,
                build_team_derelict: true,
                block_unlocked_host: true,
                valid_place: true,
                block: Some("duo".into()),
                ..RepairDerelictFrame::default()
            },
            RepairDerelictFrame {
                tile_present: true,
                build_present: true,
                editor: true,
                build_team_derelict: true,
                block_unlocked_host: true,
                valid_place: true,
                block: Some("duo".into()),
                ..RepairDerelictFrame::default()
            },
            RepairDerelictFrame {
                tile_present: true,
                build_present: true,
                player_team_derelict: true,
                build_team_derelict: true,
                block_unlocked_host: true,
                valid_place: true,
                block: Some("duo".into()),
                ..RepairDerelictFrame::default()
            },
            RepairDerelictFrame {
                tile_present: true,
                build_present: true,
                build_team_derelict: false,
                block_unlocked_host: true,
                valid_place: true,
                block: Some("duo".into()),
                ..RepairDerelictFrame::default()
            },
            RepairDerelictFrame {
                tile_present: true,
                build_present: true,
                build_team_derelict: true,
                block_unlocked_host: false,
                valid_place: true,
                block: Some("duo".into()),
                ..RepairDerelictFrame::default()
            },
            RepairDerelictFrame {
                tile_present: true,
                build_present: true,
                build_team_derelict: true,
                block_unlocked_host: true,
                valid_place: false,
                block: Some("duo".into()),
                ..RepairDerelictFrame::default()
            },
        ] {
            assert!(!try_repair_derelict_plan(frame).accepted);
        }
    }

    #[test]
    fn selected_unit_plan_prefers_nearby_ai_unit_then_control_block_unit() {
        let far = SelectUnitCandidate {
            id: 1,
            x: 100.0,
            y: 100.0,
            ..SelectUnitCandidate::new(1, 100.0, 100.0)
        };
        let nearby = SelectUnitCandidate {
            id: 2,
            x: 10.0,
            y: 10.0,
            hit_size: 8.0,
            ..SelectUnitCandidate::new(2, 10.0, 10.0)
        };
        let control = SelectBuildingCandidate {
            control_block: true,
            can_control: true,
            same_team: true,
            controlled_unit_id: Some(9),
            controlled_unit_is_ai: true,
            ..SelectBuildingCandidate::new(4, 0.0, 0.0, 8.0, 8.0)
        };

        assert_eq!(
            selected_unit_plan(10.0, 10.0, Some(99), &[far, nearby], Some(control)),
            Some(SelectedUnitPlan {
                unit_id: 2,
                source: SelectedUnitSource::NearbyUnit
            })
        );

        assert_eq!(
            selected_unit_plan(0.0, 0.0, Some(99), &[], Some(control)),
            Some(SelectedUnitPlan {
                unit_id: 9,
                source: SelectedUnitSource::ControlBlock
            })
        );

        assert_eq!(
            selected_unit_plan(0.0, 0.0, Some(9), &[], Some(control)),
            None
        );
    }

    #[test]
    fn selected_control_build_plan_requires_alive_same_team_selectable() {
        let build = SelectBuildingCandidate {
            same_team: true,
            can_control_select: true,
            ..SelectBuildingCandidate::new(7, 0.0, 0.0, 8.0, 8.0)
        };

        assert_eq!(selected_control_build_plan(false, Some(build)), Some(7));
        assert_eq!(selected_control_build_plan(true, Some(build)), None);
        assert_eq!(
            selected_control_build_plan(
                false,
                Some(SelectBuildingCandidate {
                    same_team: false,
                    ..build
                })
            ),
            None
        );
    }

    #[test]
    fn selected_command_and_enemy_units_pick_visible_closest_edge() {
        let close = SelectUnitCandidate {
            id: 1,
            x: 10.0,
            y: 10.0,
            hit_size: 4.0,
            ..SelectUnitCandidate::new(1, 10.0, 10.0)
        };
        let closer_edge = SelectUnitCandidate {
            id: 2,
            x: 11.0,
            y: 10.0,
            hit_size: 12.0,
            ..SelectUnitCandidate::new(2, 11.0, 10.0)
        };
        let not_commandable = SelectUnitCandidate {
            id: 3,
            commandable: false,
            ..SelectUnitCandidate::new(3, 10.0, 10.0)
        };

        assert_eq!(
            selected_command_unit_plan(10.0, 10.0, &[close, closer_edge, not_commandable]),
            Some(2)
        );

        assert_eq!(
            selected_enemy_unit_plan(
                10.0,
                10.0,
                &[
                    SelectUnitCandidate {
                        id: 4,
                        in_fog_to_player: true,
                        ..SelectUnitCandidate::new(4, 10.0, 10.0)
                    },
                    close,
                ],
            ),
            Some(1)
        );
    }

    #[test]
    fn selected_command_rects_filter_commandable_buildings_and_units() {
        let rect = SelectionRectFrame {
            x: 0.0,
            y: 0.0,
            w: 16.0,
            h: 16.0,
        };
        let buildings = vec![
            SelectBuildingCandidate {
                id: 1,
                commandable: true,
                ..SelectBuildingCandidate::new(1, 4.0, 4.0, 8.0, 8.0)
            },
            SelectBuildingCandidate {
                id: 2,
                commandable: false,
                ..SelectBuildingCandidate::new(2, 4.0, 4.0, 8.0, 8.0)
            },
            SelectBuildingCandidate {
                id: 3,
                commandable: true,
                ..SelectBuildingCandidate::new(3, 100.0, 100.0, 8.0, 8.0)
            },
        ];
        assert_eq!(selected_command_buildings_plan(rect, &buildings), vec![1]);

        let units = vec![
            SelectUnitCandidate {
                id: 4,
                x: 4.0,
                y: 4.0,
                commandable: true,
                ..SelectUnitCandidate::new(4, 4.0, 4.0)
            },
            SelectUnitCandidate {
                id: 5,
                x: 5.0,
                y: 5.0,
                commandable: true,
                ..SelectUnitCandidate::new(5, 5.0, 5.0)
            },
            SelectUnitCandidate {
                id: 6,
                x: 100.0,
                y: 100.0,
                commandable: true,
                ..SelectUnitCandidate::new(6, 100.0, 100.0)
            },
        ];
        assert_eq!(
            selected_command_units_plan(rect, &units, |unit| unit.id != 5),
            vec![4]
        );
    }

    #[test]
    fn unit_building_control_select_accepts_same_team_selectable_building() {
        let unit = UnitComp::new(90, unit_type(10), TeamId(1));
        let building = BuildingComp::new(point2_pack(22, 24), command_block(), TeamId(1));

        let outcome =
            unit_building_control_select(Some(&unit), Some(&building), false, |unit, build| {
                unit.id() == 90 && build.tile_pos == point2_pack(22, 24)
            });

        assert!(outcome.accepted);
        let packet = outcome.packet.unwrap();
        assert_eq!(packet.unit, UnitRef::Unit { id: 90 });
        assert_eq!(packet.build, BuildingRef::new(point2_pack(22, 24)));
    }

    #[test]
    fn unit_building_control_select_rejects_server_side_not_selectable() {
        let unit = UnitComp::new(91, unit_type(10), TeamId(1));
        let building = BuildingComp::new(point2_pack(23, 25), command_block(), TeamId(1));

        let outcome =
            unit_building_control_select(Some(&unit), Some(&building), false, |_, _| false);

        assert!(!outcome.accepted);
        assert_eq!(
            outcome.rejection,
            Some(UnitBuildingControlSelectRejectReason::NotSelectable)
        );
        assert!(outcome.packet.is_none());
    }

    #[test]
    fn unit_building_control_select_client_side_skips_selectable_check() {
        let unit = UnitComp::new(92, unit_type(10), TeamId(1));
        let building = BuildingComp::new(point2_pack(24, 26), command_block(), TeamId(1));

        let outcome =
            unit_building_control_select(Some(&unit), Some(&building), true, |_, _| false);

        assert!(outcome.accepted);
        assert!(outcome.packet.is_some());
    }

    #[test]
    fn unit_building_control_select_rejects_dead_or_other_team_unit() {
        let mut unit = UnitComp::new(93, unit_type(10), TeamId(1));
        unit.health.kill();
        let building = BuildingComp::new(point2_pack(25, 27), command_block(), TeamId(1));

        let dead = unit_building_control_select(Some(&unit), Some(&building), true, |_, _| true);
        assert_eq!(
            dead.rejection,
            Some(UnitBuildingControlSelectRejectReason::UnitDead)
        );

        let unit = UnitComp::new(94, unit_type(10), TeamId(2));
        let mismatch =
            unit_building_control_select(Some(&unit), Some(&building), true, |_, _| true);
        assert_eq!(
            mismatch.rejection,
            Some(UnitBuildingControlSelectRejectReason::TeamMismatch)
        );
    }

    #[test]
    fn unit_building_control_select_packet_uses_java_payload_shape() {
        let building = BuildingComp::new(point2_pack(26, 28), command_block(), TeamId(1));
        let packet = unit_building_control_select_packet(UnitRef::Unit { id: 95 }, &building);

        assert_eq!(packet.unit, UnitRef::Unit { id: 95 });
        assert_eq!(packet.build, BuildingRef::new(point2_pack(26, 28)));
    }

    #[test]
    fn check_unit_plan_selects_remote_or_local_control_target() {
        let missing = check_unit_plan(CheckUnitFrame::default());
        assert!(!missing.accepted);
        assert_eq!(missing.action, None);

        let remote = check_unit_plan(CheckUnitFrame {
            controlled_type_present: true,
            controlled_type_player_controllable: true,
            closest_unit_present: true,
            net_client: true,
            ..CheckUnitFrame::default()
        });
        assert!(remote.accepted);
        assert_eq!(
            remote.action,
            Some(InputHandlerLocalAction::UnitControlRemote)
        );

        let local_block_unit = check_unit_plan(CheckUnitFrame {
            controlled_type_present: true,
            controlled_type_player_controllable: true,
            block_control_unit_present: true,
            net_client: false,
            ..CheckUnitFrame::default()
        });
        assert_eq!(
            local_block_unit.action,
            Some(InputHandlerLocalAction::UnitControlLocal)
        );
    }

    #[test]
    fn payload_pickup_plan_prefers_unit_then_building_paths() {
        let unit = try_pickup_payload_plan(PayloadPickupFrame {
            unit_is_payload: true,
            pickup_unit_available: true,
            build_present: true,
            ..PayloadPickupFrame::default()
        });
        assert_eq!(
            unit.action,
            Some(InputHandlerLocalAction::RequestUnitPayload)
        );

        let stored = try_pickup_payload_plan(PayloadPickupFrame {
            unit_is_payload: true,
            build_present: true,
            teams_can_interact: true,
            stored_payload_pickable: true,
            ..PayloadPickupFrame::default()
        });
        assert_eq!(
            stored.action,
            Some(InputHandlerLocalAction::RequestBuildPayload)
        );

        let whole_build = try_pickup_payload_plan(PayloadPickupFrame {
            unit_is_payload: true,
            build_present: true,
            teams_can_interact: true,
            build_visibility_hidden: false,
            build_can_pickup: true,
            payload_can_pickup_build: true,
            ..PayloadPickupFrame::default()
        });
        assert!(whole_build.accepted);

        let rejected = try_pickup_payload_plan(PayloadPickupFrame {
            unit_is_payload: true,
            build_present: true,
            teams_can_interact: true,
            build_visibility_hidden: true,
            build_can_pickup: true,
            payload_can_pickup_build: true,
            ..PayloadPickupFrame::default()
        });
        assert!(!rejected.accepted);
    }

    #[test]
    fn payload_drop_plan_requires_payload_unit_and_drop_capability() {
        let rejected = try_drop_payload_plan(PayloadDropFrame {
            unit_is_payload: true,
            can_drop_payload: false,
            player_x: 1.0,
            player_y: 2.0,
        });
        assert!(!rejected.accepted);

        let accepted = try_drop_payload_plan(PayloadDropFrame {
            unit_is_payload: true,
            can_drop_payload: true,
            player_x: 3.0,
            player_y: 4.0,
        });
        assert_eq!(
            accepted.action,
            Some(InputHandlerLocalAction::RequestDropPayload { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn can_shoot_plan_matches_java_gate_conditions() {
        assert!(can_shoot_plan(CanShootFrame::default()));

        for frame in [
            CanShootFrame {
                block_selected: true,
                ..CanShootFrame::default()
            },
            CanShootFrame {
                on_configurable: true,
                ..CanShootFrame::default()
            },
            CanShootFrame {
                dropping_item: true,
                ..CanShootFrame::default()
            },
            CanShootFrame {
                actively_building: true,
                ..CanShootFrame::default()
            },
            CanShootFrame {
                mech_flying: true,
                ..CanShootFrame::default()
            },
            CanShootFrame {
                mining: true,
                ..CanShootFrame::default()
            },
            CanShootFrame {
                command_mode: true,
                ..CanShootFrame::default()
            },
        ] {
            assert!(!can_shoot_plan(frame));
        }
    }

    #[test]
    fn drop_item_and_deposit_plans_follow_cooldown_and_target_rules() {
        assert!(can_drop_item_plan(true, false));
        assert!(!can_drop_item_plan(true, true));

        assert!(can_deposit_item_plan(DepositItemFrame {
            item_deposit_cooldown: 3.0,
            rules_item_deposit_cooldown: 10.0,
            block_deposit_cooldown: 5.0,
        }));
        assert!(!can_deposit_item_plan(DepositItemFrame {
            item_deposit_cooldown: 6.0,
            rules_item_deposit_cooldown: 10.0,
            block_deposit_cooldown: 5.0,
        }));
        assert!(can_deposit_item_plan(DepositItemFrame {
            item_deposit_cooldown: 0.0,
            rules_item_deposit_cooldown: 10.0,
            block_deposit_cooldown: -1.0,
        }));

        let cleared = try_drop_items_plan(TryDropItemsFrame {
            dropping_item: true,
            stack_amount: 3,
            can_tap_player: true,
            ..TryDropItemsFrame::default()
        });
        assert_eq!(
            cleared.action,
            Some(InputHandlerLocalAction::ClearDroppingItem)
        );
        assert!(!cleared.dropping_item);

        let transfer = try_drop_items_plan(TryDropItemsFrame {
            dropping_item: true,
            stack_amount: 3,
            build_present: true,
            build_accepts_stack: 3,
            build_interactable: true,
            build_has_items: true,
            build_allow_deposit: true,
            can_deposit_item: true,
            rules_item_deposit_cooldown: 12.0,
            drop_angle: 45.0,
            ..TryDropItemsFrame::default()
        });
        assert_eq!(
            transfer.action,
            Some(InputHandlerLocalAction::TransferInventory {
                new_item_deposit_cooldown: 12.0
            })
        );

        let drop = try_drop_items_plan(TryDropItemsFrame {
            dropping_item: true,
            stack_amount: 3,
            build_present: true,
            build_accepts_stack: 0,
            drop_angle: 90.0,
            ..TryDropItemsFrame::default()
        });
        assert_eq!(
            drop.action,
            Some(InputHandlerLocalAction::DropItem { angle: 90.0 })
        );
    }
}
