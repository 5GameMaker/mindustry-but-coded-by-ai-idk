// Mirrors upstream core/src/mindustry/input. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod input_handler;
pub mod place_mode;

pub use input_handler::{
    client_tile_config_packet, tile_config, TileConfigContext, TileConfigOutcome,
    TileConfigRejectReason, TileConfigRollbackPlan,
};
pub use place_mode::PlaceMode;
