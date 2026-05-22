// Mirrors upstream core/src/mindustry/input. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod input_handler;
pub mod place_mode;

pub use input_handler::{
    building_control_select, client_building_control_select_packet, client_delete_plans_packet,
    client_drop_item_packet, client_ping_location_packet, client_request_build_payload_packet,
    client_request_drop_payload_packet, client_request_item_packet,
    client_request_unit_payload_packet, client_rotate_block_packet, client_tile_config_packet,
    client_tile_tap_packet, client_transfer_inventory_packet, client_unit_clear_packet,
    client_unit_control_packet, delete_plans, drop_item, payload_dropped, picked_build_payload,
    picked_unit_payload, ping_location, request_build_payload, request_drop_payload, request_item,
    request_unit_payload, rotate_block, tile_config, tile_tap, transfer_inventory, unit_clear,
    unit_control, unit_entered_payload, BuildPayloadPickupKind, BuildingControlSelectContext,
    BuildingControlSelectOutcome, BuildingControlSelectRejectReason, DeletePlansContext,
    DeletePlansOutcome, DeletePlansRejectReason, DropItemContext, DropItemOutcome,
    DropItemRejectReason, PayloadDroppedOutcome, PayloadDroppedRejectReason,
    PickedBuildPayloadOutcome, PickedBuildPayloadRejectReason, PickedUnitPayloadOutcome,
    PickedUnitPayloadRejectReason, PingLocationContext, PingLocationOutcome,
    PingLocationRejectReason, RequestBuildPayloadContext, RequestBuildPayloadOutcome,
    RequestBuildPayloadRejectReason, RequestDropPayloadContext, RequestDropPayloadOutcome,
    RequestDropPayloadRejectReason, RequestItemContext, RequestItemOutcome,
    RequestItemRejectReason, RequestUnitPayloadContext, RequestUnitPayloadOutcome,
    RequestUnitPayloadRejectReason, RotateBlockContext, RotateBlockOutcome,
    RotateBlockRejectReason, TileConfigContext, TileConfigOutcome, TileConfigRejectReason,
    TileConfigRollbackPlan, TileTapContext, TileTapOutcome, TransferInventoryContext,
    TransferInventoryOutcome, TransferInventoryRejectReason, UnitClearContext, UnitClearOutcome,
    UnitClearRejectReason, UnitControlContext, UnitControlOutcome, UnitControlRejectReason,
    UnitEnteredPayloadOutcome, UnitEnteredPayloadRejectReason,
};
pub use place_mode::PlaceMode;
