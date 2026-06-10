// Mirrors upstream core/src/mindustry/graphics. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod block_renderer;
pub mod cache_layer;
pub mod cubemap_mesh;
pub mod debug_collision_renderer;
pub mod drawf;
pub mod env_renderers;
pub mod floor_renderer;
pub mod fog_renderer;
pub mod g3d;
pub mod intel_gpu_check;
pub mod inverse_kinematics;
pub mod layer;
pub mod light_renderer;
pub mod load_renderer;
pub mod menu_renderer;
pub mod minimap_renderer;
pub mod minimap_world_adapter;
pub mod multi_packer;
pub mod nv_gpu_info;
pub mod overlay_renderer;
pub mod pal;
pub mod particle_renderer;
pub mod pixelator;
pub mod render_bridge;
pub mod render_engine;
pub mod shaders;
pub mod texture_atlas;
pub mod trail;
pub mod voronoi;

pub use block_renderer::*;
pub use cache_layer::{CacheLayer, CacheLayerEntry};
pub use cubemap_mesh::*;
pub use debug_collision_renderer::*;
pub use drawf::{BeamMode, BeamPlan, Drawf, FlamePlan, LightDrawPlan};
pub use env_renderers::*;
pub use floor_renderer::{
    CacheInvalidation, CacheInvalidationReason, CacheInvalidationState, ChunkCoord, ChunkRange,
    FloorRenderPlan, FloorRenderStage, FloorRendererState, FloorStagePlan, TileExtent, Viewport,
    ViewportTileRange,
};
pub use fog_renderer::*;
pub use intel_gpu_check::*;
pub use inverse_kinematics::{InverseKinematics, SolveOutput};
pub use layer::{Layer, LayerEntry};
pub use light_renderer::{
    LightCommand, LightPrimitive, LightRendererPlan, LightRendererState, RegionLightCommand,
};
pub use load_renderer::*;
pub use menu_renderer::{
    MenuBlockKind, MenuButtonPlan, MenuButtonRole, MenuCustomButton, MenuFrameInput, MenuFramePlan,
    MenuRenderCommand, MenuRendererConfig, MenuRendererState, MenuTile, MenuUiPlan,
    MenuUiRenderCommandCache, MenuWorldPlan, MENU_DARKNESS, MENU_DARKNESS_LAYER, MENU_TILE_SIZE,
};
pub use minimap_renderer::*;
pub use minimap_world_adapter::*;
pub use multi_packer::*;
pub use nv_gpu_info::*;
pub use overlay_renderer::*;
pub use pal::{Pal, PalEntry};
pub use particle_renderer::{
    Particle, ParticleCamera, ParticleRenderPlan, ParticleRendererState, ParticleVertex,
};
pub use pixelator::{
    PixelatorCamera, PixelatorFramePlan, PixelatorInput, PixelatorRestorePlan, PixelatorState,
};
pub use render_bridge::*;
pub use render_engine::*;
pub use shaders::*;
pub use texture_atlas::*;
pub use trail::{Trail, TrailPoint, TrailQuadPlan, TrailSegmentPlan};
pub use voronoi::*;
