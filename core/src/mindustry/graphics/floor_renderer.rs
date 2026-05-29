//! Data-only mirror of upstream `mindustry.graphics.FloorRenderer`.
//!
//! This module intentionally models the renderer as state + plan data instead
//! of a GPU-backed implementation. The goal is to preserve the important
//! scheduling concepts from upstream:
//! - render stage ordering (`floor`, `cache`, `ore`, `shadow`, `scorch`,
//!   `decals`)
//! - viewport tile/chunk range calculation
//! - cache invalidation markers
//!
//! The real renderer is still responsible for drawing, batching, and shader
//! binding. This file only exposes the planning layer so the rest of the Rust
//! port can reason about it deterministically.

#![allow(dead_code)]

use super::{
    cache_layer::CacheLayer,
    render_engine::{RenderEngineState, RenderPass},
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn chunk(self, chunk_size: i32) -> ChunkCoord {
        assert!(chunk_size > 0, "chunk_size must be positive");
        ChunkCoord::new(self.x.div_euclid(chunk_size), self.y.div_euclid(chunk_size))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

impl ChunkCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn tile_range(self, chunk_size: i32, world_tiles: Option<TileExtent>) -> TileRange {
        assert!(chunk_size > 0, "chunk_size must be positive");

        let range = TileRange::new(
            self.x.saturating_mul(chunk_size),
            self.y.saturating_mul(chunk_size),
            self.x.saturating_add(1).saturating_mul(chunk_size),
            self.y.saturating_add(1).saturating_mul(chunk_size),
        );

        if let Some(world_tiles) = world_tiles {
            range.clamp_to_world(world_tiles)
        } else {
            range
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRange {
    /// Inclusive lower bound.
    pub min_x: i32,
    /// Inclusive lower bound.
    pub min_y: i32,
    /// Exclusive upper bound.
    pub max_x: i32,
    /// Exclusive upper bound.
    pub max_y: i32,
}

impl TileRange {
    pub const fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub const fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn is_empty(self) -> bool {
        self.min_x >= self.max_x || self.min_y >= self.max_y
    }

    pub fn width(self) -> i32 {
        (self.max_x - self.min_x).max(0)
    }

    pub fn height(self) -> i32 {
        (self.max_y - self.min_y).max(0)
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.min_x && x < self.max_x && y >= self.min_y && y < self.max_y
    }

    pub fn clamp_to_world(self, world_tiles: TileExtent) -> Self {
        if world_tiles.is_empty() {
            return Self::empty();
        }

        let min_x = self.min_x.clamp(0, world_tiles.width);
        let min_y = self.min_y.clamp(0, world_tiles.height);
        let max_x = self.max_x.clamp(0, world_tiles.width);
        let max_y = self.max_y.clamp(0, world_tiles.height);

        Self::new(min_x, min_y, max_x, max_y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileExtent {
    pub width: i32,
    pub height: i32,
}

impl TileExtent {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn is_empty(self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    pub fn chunk_extent(self, chunk_size: i32) -> ChunkRange {
        assert!(chunk_size > 0, "chunk_size must be positive");
        ChunkRange::new(
            0,
            0,
            ceil_div_i32(self.width, chunk_size),
            ceil_div_i32(self.height, chunk_size),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub center_x: f32,
    pub center_y: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub const fn new(center_x: f32, center_y: f32, width: f32, height: f32) -> Self {
        Self {
            center_x,
            center_y,
            width,
            height,
        }
    }

    pub fn tile_range(
        self,
        tile_size_world: f32,
        padding_world: f32,
        world_tiles: Option<TileExtent>,
    ) -> ViewportTileRange {
        assert!(tile_size_world > 0.0, "tile_size_world must be positive");

        let half_width = self.width * 0.5 + padding_world;
        let half_height = self.height * 0.5 + padding_world;
        let min_x = ((self.center_x - half_width) / tile_size_world).floor() as i32;
        let min_y = ((self.center_y - half_height) / tile_size_world).floor() as i32;
        let max_x = ((self.center_x + half_width) / tile_size_world).ceil() as i32;
        let max_y = ((self.center_y + half_height) / tile_size_world).ceil() as i32;

        let range = ViewportTileRange::new(min_x, min_y, max_x, max_y);
        if let Some(world_tiles) = world_tiles {
            range.clamp_to_world(world_tiles)
        } else {
            range
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportTileRange {
    /// Inclusive lower bound.
    pub min_x: i32,
    /// Inclusive lower bound.
    pub min_y: i32,
    /// Exclusive upper bound.
    pub max_x: i32,
    /// Exclusive upper bound.
    pub max_y: i32,
}

impl ViewportTileRange {
    pub const fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub const fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn is_empty(self) -> bool {
        self.min_x >= self.max_x || self.min_y >= self.max_y
    }

    pub fn width(self) -> i32 {
        (self.max_x - self.min_x).max(0)
    }

    pub fn height(self) -> i32 {
        (self.max_y - self.min_y).max(0)
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.min_x && x < self.max_x && y >= self.min_y && y < self.max_y
    }

    pub fn clamp_to_world(self, world_tiles: TileExtent) -> Self {
        if world_tiles.is_empty() {
            return Self::empty();
        }

        let min_x = self.min_x.clamp(0, world_tiles.width);
        let min_y = self.min_y.clamp(0, world_tiles.height);
        let max_x = self.max_x.clamp(0, world_tiles.width);
        let max_y = self.max_y.clamp(0, world_tiles.height);

        Self::new(min_x, min_y, max_x, max_y)
    }

    pub fn chunk_range(self, chunk_size: i32) -> ChunkRange {
        assert!(chunk_size > 0, "chunk_size must be positive");
        if self.is_empty() {
            return ChunkRange::empty();
        }

        ChunkRange::new(
            self.min_x.div_euclid(chunk_size),
            self.min_y.div_euclid(chunk_size),
            ceil_div_i32(self.max_x, chunk_size),
            ceil_div_i32(self.max_y, chunk_size),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRange {
    /// Inclusive lower bound.
    pub min_x: i32,
    /// Inclusive lower bound.
    pub min_y: i32,
    /// Exclusive upper bound.
    pub max_x: i32,
    /// Exclusive upper bound.
    pub max_y: i32,
}

impl ChunkRange {
    pub const fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub const fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn is_empty(self) -> bool {
        self.min_x >= self.max_x || self.min_y >= self.max_y
    }

    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.min_x && x < self.max_x && y >= self.min_y && y < self.max_y
    }

    pub fn clamp_to_world(self, world_tiles: TileExtent, chunk_size: i32) -> Self {
        if world_tiles.is_empty() {
            return Self::empty();
        }

        let world_chunks = world_tiles.chunk_extent(chunk_size);
        Self::new(
            self.min_x.clamp(world_chunks.min_x, world_chunks.max_x),
            self.min_y.clamp(world_chunks.min_y, world_chunks.max_y),
            self.max_x.clamp(world_chunks.min_x, world_chunks.max_x),
            self.max_y.clamp(world_chunks.min_y, world_chunks.max_y),
        )
    }

    pub fn to_coords(self) -> Vec<ChunkCoord> {
        if self.is_empty() {
            return Vec::new();
        }

        let mut coords = Vec::new();
        for y in self.min_y..self.max_y {
            for x in self.min_x..self.max_x {
                coords.push(ChunkCoord::new(x, y));
            }
        }
        coords
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum FloorRenderStage {
    Floor,
    Cache,
    Ore,
    Shadow,
    Scorch,
    Decals,
}

impl FloorRenderStage {
    pub const ORDERED: [Self; 6] = [
        Self::Floor,
        Self::Cache,
        Self::Ore,
        Self::Shadow,
        Self::Scorch,
        Self::Decals,
    ];

    pub const fn ordered() -> [Self; 6] {
        Self::ORDERED
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Cache => "cache",
            Self::Ore => "ore",
            Self::Shadow => "shadow",
            Self::Scorch => "scorch",
            Self::Decals => "decals",
        }
    }

    pub const fn uses_cache(self) -> bool {
        matches!(self, Self::Floor | Self::Cache | Self::Ore)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheInvalidationReason {
    TileChanged,
    ChunkChanged,
    CacheSettingsChanged,
    WorldReloaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheInvalidation {
    pub reason: CacheInvalidationReason,
    pub tile: Option<TileCoord>,
    pub chunk: Option<ChunkCoord>,
}

impl CacheInvalidation {
    pub const fn new(
        reason: CacheInvalidationReason,
        tile: Option<TileCoord>,
        chunk: Option<ChunkCoord>,
    ) -> Self {
        Self {
            reason,
            tile,
            chunk,
        }
    }

    pub const fn tile(tile: TileCoord, chunk: ChunkCoord) -> Self {
        Self::new(
            CacheInvalidationReason::TileChanged,
            Some(tile),
            Some(chunk),
        )
    }

    pub const fn chunk(chunk: ChunkCoord) -> Self {
        Self::new(CacheInvalidationReason::ChunkChanged, None, Some(chunk))
    }

    pub const fn cache_settings_changed() -> Self {
        Self::new(CacheInvalidationReason::CacheSettingsChanged, None, None)
    }

    pub const fn world_reloaded() -> Self {
        Self::new(CacheInvalidationReason::WorldReloaded, None, None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheInvalidationState {
    pub ignore_walls: bool,
    pub full_reload: bool,
    pub dirty_chunks: BTreeSet<ChunkCoord>,
    pub pending: Vec<CacheInvalidation>,
}

impl Default for CacheInvalidationState {
    fn default() -> Self {
        Self {
            ignore_walls: false,
            full_reload: false,
            dirty_chunks: BTreeSet::new(),
            pending: Vec::new(),
        }
    }
}

impl CacheInvalidationState {
    pub fn mark_tile_dirty(&mut self, tile_x: i32, tile_y: i32, chunk_size: i32) -> ChunkCoord {
        let tile = TileCoord::new(tile_x, tile_y);
        let chunk = tile.chunk(chunk_size);
        self.dirty_chunks.insert(chunk);
        self.pending.push(CacheInvalidation::tile(tile, chunk));
        chunk
    }

    pub fn mark_chunk_dirty(&mut self, chunk_x: i32, chunk_y: i32) -> ChunkCoord {
        let chunk = ChunkCoord::new(chunk_x, chunk_y);
        self.dirty_chunks.insert(chunk);
        self.pending.push(CacheInvalidation::chunk(chunk));
        chunk
    }

    pub fn mark_cache_settings_changed(&mut self, ignore_walls: bool) {
        if self.ignore_walls != ignore_walls {
            self.ignore_walls = ignore_walls;
            self.full_reload = true;
            self.pending
                .push(CacheInvalidation::cache_settings_changed());
        }
    }

    pub fn mark_world_reloaded(&mut self) {
        self.full_reload = true;
        self.pending.push(CacheInvalidation::world_reloaded());
    }

    pub fn clear(&mut self) {
        self.full_reload = false;
        self.dirty_chunks.clear();
        self.pending.clear();
    }

    pub fn dirty_chunks_vec(&self) -> Vec<ChunkCoord> {
        self.dirty_chunks.iter().copied().collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloorStagePlan {
    pub stage: FloorRenderStage,
    pub uses_cache: bool,
    pub needs_cache_refresh: bool,
    pub dirty_chunks: Vec<ChunkCoord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloorChunkDrawBatch {
    pub chunk: ChunkCoord,
    pub tile_range: TileRange,
    pub stage_order: Vec<FloorRenderStage>,
    pub cache_dirty: bool,
    pub cache_invalidations: Vec<CacheInvalidation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloorRenderPlan {
    pub viewport_tiles: ViewportTileRange,
    pub viewport_chunks: ChunkRange,
    pub visible_chunks: Vec<ChunkCoord>,
    pub stage_plans: Vec<FloorStagePlan>,
    pub cache_layer_passes: Vec<RenderPass>,
    pub cache_dirty_chunks: Vec<ChunkCoord>,
    pub ignore_walls: bool,
    pub full_reload: bool,
    pub pending_invalidations: Vec<CacheInvalidation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloorRendererState {
    pub chunk_size_tiles: i32,
    pub tile_size_world: f32,
    pub viewport_padding_world: f32,
    pub stages: Vec<FloorRenderStage>,
    pub world_tiles: Option<TileExtent>,
    pub cache: CacheInvalidationState,
}

impl Default for FloorRendererState {
    fn default() -> Self {
        Self {
            chunk_size_tiles: 30,
            tile_size_world: 8.0,
            viewport_padding_world: 4.0,
            stages: FloorRenderStage::ordered().to_vec(),
            world_tiles: None,
            cache: CacheInvalidationState::default(),
        }
    }
}

impl FloorRendererState {
    pub fn with_world_tiles(mut self, width: i32, height: i32) -> Self {
        self.world_tiles = Some(TileExtent::new(width, height));
        self
    }

    pub fn set_world_tiles(&mut self, width: i32, height: i32) {
        self.world_tiles = Some(TileExtent::new(width, height));
    }

    pub fn set_stage_order(&mut self, stages: impl IntoIterator<Item = FloorRenderStage>) {
        self.stages = stages.into_iter().collect();
    }

    pub fn mark_tile_dirty(&mut self, tile_x: i32, tile_y: i32) -> ChunkCoord {
        self.cache
            .mark_tile_dirty(tile_x, tile_y, self.chunk_size_tiles)
    }

    pub fn mark_chunk_dirty(&mut self, chunk_x: i32, chunk_y: i32) -> ChunkCoord {
        self.cache.mark_chunk_dirty(chunk_x, chunk_y)
    }

    pub fn mark_cache_settings_changed(&mut self, ignore_walls: bool) {
        self.cache.mark_cache_settings_changed(ignore_walls);
    }

    pub fn mark_world_reloaded(&mut self) {
        self.cache.mark_world_reloaded();
    }

    pub fn clear_cache_marks(&mut self) {
        self.cache.clear();
    }

    pub fn cache_layer_passes(&self) -> Vec<RenderPass> {
        if self.stages.contains(&FloorRenderStage::Cache) {
            CacheLayer::builtin_entries()
                .iter()
                .map(|entry| entry.to_render_pass())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn push_cache_layer_passes(&self, render_engine: &mut RenderEngineState) -> Vec<usize> {
        self.cache_layer_passes()
            .into_iter()
            .map(|pass| render_engine.push_pass(pass))
            .collect()
    }

    pub fn build_plan(&self, viewport: Viewport) -> FloorRenderPlan {
        let viewport_tiles = viewport.tile_range(
            self.tile_size_world,
            self.viewport_padding_world,
            self.world_tiles,
        );
        let mut viewport_chunks = viewport_tiles.chunk_range(self.chunk_size_tiles);
        if let Some(world_tiles) = self.world_tiles {
            viewport_chunks = viewport_chunks.clamp_to_world(world_tiles, self.chunk_size_tiles);
        }

        let visible_chunks = viewport_chunks.to_coords();
        let cache_dirty_chunks = if self.cache.full_reload {
            visible_chunks.clone()
        } else {
            visible_chunks
                .iter()
                .copied()
                .filter(|chunk| self.cache.dirty_chunks.contains(chunk))
                .collect::<Vec<_>>()
        };

        let stage_plans = self
            .stages
            .iter()
            .copied()
            .map(|stage| FloorStagePlan {
                stage,
                uses_cache: stage.uses_cache(),
                needs_cache_refresh: stage.uses_cache()
                    && (self.cache.full_reload || !cache_dirty_chunks.is_empty()),
                dirty_chunks: if stage.uses_cache() {
                    cache_dirty_chunks.clone()
                } else {
                    Vec::new()
                },
            })
            .collect();
        let cache_layer_passes = self.cache_layer_passes();

        FloorRenderPlan {
            viewport_tiles,
            viewport_chunks,
            visible_chunks,
            stage_plans,
            cache_layer_passes,
            cache_dirty_chunks,
            ignore_walls: self.cache.ignore_walls,
            full_reload: self.cache.full_reload,
            pending_invalidations: self.cache.pending.clone(),
        }
    }

    pub fn build_chunk_draw_batches(&self, viewport: Viewport) -> Vec<FloorChunkDrawBatch> {
        let plan = self.build_plan(viewport);
        let stage_order = self.stages.clone();

        plan.visible_chunks
            .into_iter()
            .map(|chunk| {
                let cache_dirty = plan.full_reload || self.cache.dirty_chunks.contains(&chunk);
                let cache_invalidations = if plan.full_reload {
                    plan.pending_invalidations
                        .iter()
                        .copied()
                        .filter(|invalidation| invalidation.chunk.is_none())
                        .collect::<Vec<_>>()
                } else {
                    plan.pending_invalidations
                        .iter()
                        .copied()
                        .filter(|invalidation| invalidation.chunk == Some(chunk))
                        .collect::<Vec<_>>()
                };

                FloorChunkDrawBatch {
                    chunk,
                    tile_range: chunk.tile_range(self.chunk_size_tiles, self.world_tiles),
                    stage_order: stage_order.clone(),
                    cache_dirty,
                    cache_invalidations,
                }
            })
            .collect()
    }
}

fn ceil_div_i32(value: i32, divisor: i32) -> i32 {
    assert!(divisor > 0, "divisor must be positive");
    let quotient = value.div_euclid(divisor);
    if value.rem_euclid(divisor) == 0 {
        quotient
    } else {
        quotient + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::graphics::{
        RenderCamera, RenderEngineState, RenderPassKind, RenderPoint, RenderResolveKind,
        RenderSize, RenderTarget, RenderViewport,
    };

    #[test]
    fn viewport_tile_range_clamps_and_uses_exclusive_upper_bounds() {
        let viewport = Viewport::new(16.0, 16.0, 16.0, 16.0);
        let range = viewport.tile_range(8.0, 4.0, Some(TileExtent::new(10, 10)));

        assert_eq!(range, ViewportTileRange::new(0, 0, 4, 4));
        assert_eq!(range.width(), 4);
        assert_eq!(range.height(), 4);
        assert!(range.contains(0, 0));
        assert!(range.contains(3, 3));
        assert!(!range.contains(4, 4));

        let chunks = range.chunk_range(2);
        assert_eq!(chunks, ChunkRange::new(0, 0, 2, 2));
        assert_eq!(
            chunks.to_coords(),
            vec![
                ChunkCoord::new(0, 0),
                ChunkCoord::new(1, 0),
                ChunkCoord::new(0, 1),
                ChunkCoord::new(1, 1),
            ]
        );
    }

    #[test]
    fn cache_invalidation_marks_are_deduplicated_by_chunk() {
        let mut cache = CacheInvalidationState::default();

        assert_eq!(cache.mark_tile_dirty(1, 1, 8), ChunkCoord::new(0, 0));
        assert_eq!(cache.mark_tile_dirty(7, 7, 8), ChunkCoord::new(0, 0));
        assert_eq!(cache.mark_chunk_dirty(1, 2), ChunkCoord::new(1, 2));
        cache.mark_cache_settings_changed(true);
        cache.mark_world_reloaded();

        assert_eq!(cache.ignore_walls, true);
        assert!(cache.full_reload);
        assert_eq!(
            cache.dirty_chunks_vec(),
            vec![ChunkCoord::new(0, 0), ChunkCoord::new(1, 2)]
        );
        assert_eq!(cache.pending.len(), 5);
        assert!(matches!(
            cache.pending[0],
            CacheInvalidation {
                reason: CacheInvalidationReason::TileChanged,
                tile: Some(TileCoord { x: 1, y: 1 }),
                chunk: Some(ChunkCoord { x: 0, y: 0 }),
            }
        ));
        assert!(matches!(
            cache.pending[3],
            CacheInvalidation {
                reason: CacheInvalidationReason::CacheSettingsChanged,
                tile: None,
                chunk: None,
            }
        ));
        assert!(matches!(
            cache.pending[4],
            CacheInvalidation {
                reason: CacheInvalidationReason::WorldReloaded,
                tile: None,
                chunk: None,
            }
        ));

        cache.clear();
        assert!(!cache.full_reload);
        assert!(cache.dirty_chunks.is_empty());
        assert!(cache.pending.is_empty());
    }

    #[test]
    fn chunk_tile_range_clamps_to_world_edges() {
        let chunk = ChunkCoord::new(3, 1);

        assert_eq!(chunk.tile_range(30, None), TileRange::new(90, 30, 120, 60));
        assert_eq!(
            chunk.tile_range(30, Some(TileExtent::new(100, 50))),
            TileRange::new(90, 30, 100, 50)
        );
    }

    #[test]
    fn floor_renderer_plan_preserves_stage_order_and_marks_cacheable_stages() {
        let mut state = FloorRendererState::default().with_world_tiles(10, 10);
        state.chunk_size_tiles = 2;
        state.tile_size_world = 8.0;
        state.viewport_padding_world = 4.0;
        state.mark_tile_dirty(0, 0);
        state.mark_tile_dirty(7, 7);
        state.mark_chunk_dirty(3, 3);

        let plan = state.build_plan(Viewport::new(8.0, 8.0, 16.0, 16.0));

        assert_eq!(plan.viewport_tiles, ViewportTileRange::new(0, 0, 3, 3));
        assert_eq!(plan.viewport_chunks, ChunkRange::new(0, 0, 2, 2));
        assert_eq!(
            plan.visible_chunks,
            vec![
                ChunkCoord::new(0, 0),
                ChunkCoord::new(1, 0),
                ChunkCoord::new(0, 1),
                ChunkCoord::new(1, 1),
            ]
        );
        assert_eq!(plan.cache_dirty_chunks, vec![ChunkCoord::new(0, 0)]);
        assert!(!plan.full_reload);
        assert!(!plan.ignore_walls);

        let stages: Vec<_> = plan.stage_plans.iter().map(|stage| stage.stage).collect();
        assert_eq!(stages, FloorRenderStage::ordered().to_vec());

        let cacheable: Vec<_> = plan
            .stage_plans
            .iter()
            .filter(|stage| stage.stage.uses_cache())
            .map(|stage| {
                (
                    stage.stage,
                    stage.uses_cache,
                    stage.needs_cache_refresh,
                    stage.dirty_chunks.clone(),
                )
            })
            .collect();
        assert_eq!(cacheable.len(), 3);
        assert_eq!(
            cacheable,
            vec![
                (
                    FloorRenderStage::Floor,
                    true,
                    true,
                    vec![ChunkCoord::new(0, 0)]
                ),
                (
                    FloorRenderStage::Cache,
                    true,
                    true,
                    vec![ChunkCoord::new(0, 0)]
                ),
                (
                    FloorRenderStage::Ore,
                    true,
                    true,
                    vec![ChunkCoord::new(0, 0)]
                ),
            ]
        );

        let overlays: Vec<_> = plan
            .stage_plans
            .iter()
            .filter(|stage| !stage.stage.uses_cache())
            .map(|stage| {
                (
                    stage.stage,
                    stage.uses_cache,
                    stage.needs_cache_refresh,
                    stage.dirty_chunks.clone(),
                )
            })
            .collect();
        assert_eq!(
            overlays,
            vec![
                (FloorRenderStage::Shadow, false, false, Vec::new()),
                (FloorRenderStage::Scorch, false, false, Vec::new()),
                (FloorRenderStage::Decals, false, false, Vec::new()),
            ]
        );
        assert_eq!(plan.cache_layer_passes.len(), 9);
        assert_eq!(
            plan.cache_layer_passes
                .iter()
                .map(|pass| (
                    pass.kind.clone(),
                    pass.target.clone(),
                    pass.resolve_target.clone(),
                    pass.resolve_kind,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:water:effect".into()),
                    Some(RenderTarget::Buffer("cache-layer:water:floor".into())),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:mud:effect".into()),
                    Some(RenderTarget::Buffer("cache-layer:mud:floor".into())),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:tar:effect".into()),
                    Some(RenderTarget::Buffer("cache-layer:tar:floor".into())),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:slag:effect".into()),
                    Some(RenderTarget::Buffer("cache-layer:slag:floor".into())),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:arkycite:effect".into()),
                    Some(RenderTarget::Buffer("cache-layer:arkycite:floor".into())),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:cryofluid:effect".into()),
                    Some(RenderTarget::Buffer("cache-layer:cryofluid:floor".into())),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:space:effect".into()),
                    Some(RenderTarget::Buffer("cache-layer:space:floor".into())),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:normal:floor".into()),
                    None,
                    None,
                ),
                (
                    RenderPassKind::BlockWalls,
                    RenderTarget::Buffer("cache-layer:walls:floor".into()),
                    None,
                    None,
                ),
            ]
        );
    }

    #[test]
    fn floor_renderer_pushes_cache_layer_passes_into_render_engine_state() {
        let viewport = RenderViewport::new(0.0, 0.0, 64.0, 64.0);
        let camera = RenderCamera::new(RenderPoint::new(32.0, 32.0), viewport);
        let mut render_engine = RenderEngineState::new(RenderSize::new(64.0, 64.0), camera);
        let state = FloorRendererState::default();

        let indices = state.push_cache_layer_passes(&mut render_engine);
        let frame = render_engine.finish();

        assert_eq!(indices, (0..9).collect::<Vec<_>>());
        assert_eq!(frame.passes.len(), 9);
        assert_eq!(
            frame
                .passes
                .iter()
                .map(|pass| (pass.kind.clone(), pass.target.clone(), pass.resolve_kind))
                .collect::<Vec<_>>(),
            vec![
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:water:effect".into()),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:mud:effect".into()),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:tar:effect".into()),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:slag:effect".into()),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:arkycite:effect".into()),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:cryofluid:effect".into()),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:space:effect".into()),
                    Some(RenderResolveKind::ShaderBlit),
                ),
                (
                    RenderPassKind::Floor,
                    RenderTarget::Buffer("cache-layer:normal:floor".into()),
                    None,
                ),
                (
                    RenderPassKind::BlockWalls,
                    RenderTarget::Buffer("cache-layer:walls:floor".into()),
                    None,
                ),
            ]
        );
    }

    #[test]
    fn floor_renderer_omits_cache_layer_passes_when_cache_stage_is_disabled() {
        let mut state = FloorRendererState::default();
        state.set_stage_order([
            FloorRenderStage::Floor,
            FloorRenderStage::Ore,
            FloorRenderStage::Shadow,
            FloorRenderStage::Scorch,
            FloorRenderStage::Decals,
        ]);
        let plan = state.build_plan(Viewport::new(0.0, 0.0, 16.0, 16.0));

        assert!(plan.cache_layer_passes.is_empty());
    }

    #[test]
    fn floor_renderer_builds_chunk_draw_batches_with_tile_ranges_and_cache_marks() {
        let mut state = FloorRendererState::default().with_world_tiles(10, 10);
        state.chunk_size_tiles = 2;
        state.tile_size_world = 8.0;
        state.viewport_padding_world = 4.0;
        state.mark_tile_dirty(0, 0);
        state.mark_tile_dirty(1, 1);
        state.mark_chunk_dirty(1, 1);

        let batches = state.build_chunk_draw_batches(Viewport::new(8.0, 8.0, 16.0, 16.0));

        assert_eq!(batches.len(), 4);
        assert_eq!(batches[0].chunk, ChunkCoord::new(0, 0));
        assert_eq!(batches[0].tile_range, TileRange::new(0, 0, 2, 2));
        assert!(batches[0].cache_dirty);
        assert_eq!(batches[0].stage_order, FloorRenderStage::ordered().to_vec());
        assert_eq!(batches[0].cache_invalidations.len(), 2);
        assert!(matches!(
            batches[0].cache_invalidations[0],
            CacheInvalidation {
                reason: CacheInvalidationReason::TileChanged,
                tile: Some(TileCoord { x: 0, y: 0 }),
                chunk: Some(ChunkCoord { x: 0, y: 0 }),
            }
        ));
        assert!(matches!(
            batches[0].cache_invalidations[1],
            CacheInvalidation {
                reason: CacheInvalidationReason::TileChanged,
                tile: Some(TileCoord { x: 1, y: 1 }),
                chunk: Some(ChunkCoord { x: 0, y: 0 }),
            }
        ));

        assert_eq!(batches[3].chunk, ChunkCoord::new(1, 1));
        assert_eq!(batches[3].tile_range, TileRange::new(2, 2, 4, 4));
        assert!(batches[3].cache_dirty);
        assert_eq!(batches[3].cache_invalidations.len(), 1);
        assert!(matches!(
            batches[3].cache_invalidations[0],
            CacheInvalidation {
                reason: CacheInvalidationReason::ChunkChanged,
                tile: None,
                chunk: Some(ChunkCoord { x: 1, y: 1 }),
            }
        ));
    }
}
