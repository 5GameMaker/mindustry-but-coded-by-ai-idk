// Mirrors upstream core/src/mindustry/input. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod input_handler;
pub mod place_mode;

pub use input_handler::{
    building_control_select, client_building_control_select_packet,
    client_request_build_payload_packet, client_request_item_packet, client_rotate_block_packet,
    client_tile_config_packet, client_tile_tap_packet, client_transfer_inventory_packet,
    client_unit_clear_packet, client_unit_control_packet, picked_build_payload,
    request_build_payload, request_item, rotate_block, tile_config, tile_tap, transfer_inventory,
    unit_clear, unit_control, BuildPayloadPickupKind, BuildingControlSelectContext,
    BuildingControlSelectOutcome, BuildingControlSelectRejectReason, PickedBuildPayloadOutcome,
    PickedBuildPayloadRejectReason, RequestBuildPayloadContext, RequestBuildPayloadOutcome,
    RequestBuildPayloadRejectReason, RequestItemContext, RequestItemOutcome,
    RequestItemRejectReason, RotateBlockContext, RotateBlockOutcome, RotateBlockRejectReason,
    TileConfigContext, TileConfigOutcome, TileConfigRejectReason, TileConfigRollbackPlan,
    TileTapContext, TileTapOutcome, TransferInventoryContext, TransferInventoryOutcome,
    TransferInventoryRejectReason, UnitClearContext, UnitClearOutcome, UnitClearRejectReason,
    UnitControlContext, UnitControlOutcome, UnitControlRejectReason,
};
pub use place_mode::PlaceMode;
