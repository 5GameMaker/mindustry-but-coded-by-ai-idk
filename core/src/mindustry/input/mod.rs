// Mirrors upstream core/src/mindustry/input. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod input_handler;
pub mod place_mode;

pub use input_handler::{
    client_rotate_block_packet, client_tile_config_packet, client_tile_tap_packet, rotate_block,
    tile_config, tile_tap, RotateBlockContext, RotateBlockOutcome, RotateBlockRejectReason,
    TileConfigContext, TileConfigOutcome, TileConfigRejectReason, TileConfigRollbackPlan,
    TileTapContext, TileTapOutcome,
};
pub use place_mode::PlaceMode;
