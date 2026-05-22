// Mirrors upstream core/src/mindustry/input. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod input_handler;
pub mod place_mode;

pub use input_handler::{
    client_request_build_payload_packet, client_request_item_packet, client_rotate_block_packet,
    client_tile_config_packet, client_tile_tap_packet, client_transfer_inventory_packet,
    picked_build_payload, request_build_payload, request_item, rotate_block, tile_config, tile_tap,
    transfer_inventory, BuildPayloadPickupKind, PickedBuildPayloadOutcome,
    PickedBuildPayloadRejectReason, RequestBuildPayloadContext, RequestBuildPayloadOutcome,
    RequestBuildPayloadRejectReason, RequestItemContext, RequestItemOutcome,
    RequestItemRejectReason, RotateBlockContext, RotateBlockOutcome, RotateBlockRejectReason,
    TileConfigContext, TileConfigOutcome, TileConfigRejectReason, TileConfigRollbackPlan,
    TileTapContext, TileTapOutcome, TransferInventoryContext, TransferInventoryOutcome,
    TransferInventoryRejectReason,
};
pub use place_mode::PlaceMode;
