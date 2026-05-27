// Mirrors upstream core/src/mindustry/input. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod binding;
pub mod desktop_input;
pub mod input_handler;
pub mod mobile_input;
pub mod place_mode;
pub mod placement;

pub use binding::{Binding, KeyBindingInput, KeyBindingSpec, KeyCode};
pub use desktop_input::{
    DesktopCameraPlan, DesktopCursor, DesktopInput, DesktopInputAction, DesktopInputFrame,
    DesktopInputSettings, DesktopInputUpdate, DesktopPayloadAction, DesktopVec2,
};
pub use input_handler::{
    building_control_select, can_deposit_item_plan, can_drop_item_plan, can_shoot_plan,
    check_unit_plan, clear_items, clear_liquids, client_building_control_select_packet,
    client_command_building_packet, client_command_units_packet, client_delete_plans_packet,
    client_drop_item_packet, client_ping_location_packet, client_request_build_payload_packet,
    client_request_drop_payload_packet, client_request_item_packet,
    client_request_unit_payload_packet, client_rotate_block_packet, client_set_unit_command_packet,
    client_set_unit_stance_packet, client_tile_config_packet, client_tile_tap_packet,
    client_transfer_inventory_packet, client_unit_cargo_unload_clear_config_packet,
    client_unit_cargo_unload_item_config_packet, client_unit_clear_packet,
    client_unit_control_packet, command_building, command_overlay_plan,
    command_targets_overlay_plan, command_units, delete_plans, draw_command_building_plan,
    draw_command_unit_plan, drop_item, flip_build_plans, payload_dropped, picked_build_payload,
    picked_unit_payload, ping_location, remove_queue_block, remove_queue_block_packet,
    request_build_payload, request_drop_payload, request_item, request_unit_payload, rotate_block,
    rotate_build_plans, set_item, set_items, set_liquid, set_liquids, set_tile_items,
    set_tile_liquids, set_unit_command, set_unit_stance, take_items, tile_config, tile_tap,
    transfer_inventory, transfer_item_effect, transfer_item_to, transfer_item_to_unit,
    try_drop_items_plan, try_drop_payload_plan, try_pickup_payload_plan,
    unit_building_control_select, unit_building_control_select_packet, unit_clear, unit_control,
    unit_entered_payload, BuildPayloadPickupKind, BuildPlanBlockTransform,
    BuildingControlSelectContext, BuildingControlSelectOutcome, BuildingControlSelectRejectReason,
    CanShootFrame, CheckUnitFrame, CheckUnitPlan, ClearItemsOutcome, ClearLiquidsOutcome,
    CommandBuildingContext, CommandBuildingMarkerPlan, CommandBuildingOutcome,
    CommandBuildingRejectReason, CommandMarkerSource, CommandOverlayBuilding, CommandOverlayColor,
    CommandOverlayController, CommandOverlayFrame, CommandOverlayLinePlan, CommandOverlayPlan,
    CommandOverlayTarget, CommandOverlayTargetKind, CommandPayloadIconKind, CommandPayloadIconPlan,
    CommandSelectableBuilding, CommandSelectableUnit, CommandSelectionOverlayFrame,
    CommandSelectionOverlayPlan, CommandTargetMarkerKind, CommandTargetMarkerPlan,
    CommandTargetsOverlayFrame, CommandTargetsOverlayPlan, CommandUnitMarkerPlan,
    CommandUnitsContext, CommandUnitsOutcome, CommandUnitsRejectReason, DeletePlansContext,
    DeletePlansOutcome, DeletePlansRejectReason, DepositItemFrame, DropItemContext,
    DropItemOutcome, DropItemRejectReason, InputHandlerLocalAction, ItemSyncRejectReason,
    LiquidSyncRejectReason, PayloadDropFrame, PayloadDropPlan, PayloadDroppedOutcome,
    PayloadDroppedRejectReason, PayloadPickupFrame, PayloadPickupPlan, PickedBuildPayloadOutcome,
    PickedBuildPayloadRejectReason, PickedUnitPayloadOutcome, PickedUnitPayloadRejectReason,
    PingLocationContext, PingLocationOutcome, PingLocationRejectReason, RemoveQueueBlockOutcome,
    RemoveQueueBlockRejectReason, RequestBuildPayloadContext, RequestBuildPayloadOutcome,
    RequestBuildPayloadRejectReason, RequestDropPayloadContext, RequestDropPayloadOutcome,
    RequestDropPayloadRejectReason, RequestItemContext, RequestItemOutcome,
    RequestItemRejectReason, RequestUnitPayloadContext, RequestUnitPayloadOutcome,
    RequestUnitPayloadRejectReason, RotateBlockContext, RotateBlockOutcome,
    RotateBlockRejectReason, SetItemOutcome, SetItemsOutcome, SetLiquidOutcome, SetLiquidsOutcome,
    SetTileItemsOutcome, SetTileLiquidsOutcome, SetUnitCommandContext, SetUnitCommandOutcome,
    SetUnitCommandRejectReason, SetUnitStanceContext, SetUnitStanceOutcome,
    SetUnitStanceRejectReason, TakeItemsOutcome, TakeItemsRejectReason, TileConfigContext,
    TileConfigOutcome, TileConfigRejectReason, TileConfigRollbackPlan, TileTapContext,
    TileTapOutcome, TransferInventoryContext, TransferInventoryOutcome,
    TransferInventoryRejectReason, TransferItemEffectOutcome, TransferItemEffectRejectReason,
    TransferItemToOutcome, TransferItemToRejectReason, TransferItemToUnitOutcome,
    TransferItemToUnitRejectReason, TryDropItemsFrame, TryDropItemsPlan,
    UnitBuildingControlSelectOutcome, UnitBuildingControlSelectRejectReason, UnitClearContext,
    UnitClearOutcome, UnitClearRejectReason, UnitControlContext, UnitControlOutcome,
    UnitControlRejectReason, UnitEnteredPayloadOutcome, UnitEnteredPayloadRejectReason,
    COMMAND_OVERLAY_ALPHA, COMMAND_OVERLAY_LINE_LIMIT, COMMAND_UNIT_SELECT_RADIUS_SCALE,
};
pub use mobile_input::{
    check_mobile_overlap_placement, get_mobile_plan, has_mobile_plan, is_area_breaking,
    is_line_placing, mobile_schematic_origin, plan_rect, remove_mobile_plan, synced_mobile_plans,
    MobileActionPlan, MobileBlockFootprint, MobileCombatTargetKind, MobileGesturePlan, MobileInput,
    MobileInputAction, MobileInputFrame, MobileInputUpdate, MobileLongPressFrame,
    MobileMovementFrame, MobileMovementPlan, MobilePanFrame, MobilePanPlan,
    MobilePayloadTargetKind, MobilePlacementButton, MobilePlanSnapshot, MobileRemovePlanResult,
    MobileTapFrame, MobileTargetCheckFrame, MobileTouchDownFrame, MobileTouchUpFrame, MobileVec2,
    MobileZoomPlan, DEFAULT_EDGE_PAN, MAX_PAN_SPEED,
};
pub use place_mode::PlaceMode;
pub use placement::{
    bresenham_line_no_diagonal, calculate_bridge_plans, calculate_nodes, distance_heuristic,
    is_side_place, normalize_area, normalize_draw_area, normalize_line, normalize_rectangle,
    pathfind_line, BridgePlacementConfig, NormalizeDrawResult, NormalizeResult, Placement,
    PlacementBlockDraw, PlacementPlanState,
};
