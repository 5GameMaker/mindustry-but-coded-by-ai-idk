//! BlockRenderer 的纯数据化状态/计划层镜像。
//!
//! 这里不持有任何 GPU 资源，只保留 upstream `BlockRenderer` 在
//! 缓存、绘制顺序、计划预览、暗度与覆盖层上的可序列化数据。

use std::collections::{BTreeMap, BTreeSet};

use crate::mindustry::{
    entities::comp::DecalColor,
    graphics::{
        CacheLayer, Layer, RenderCommand, RenderPass, RenderPassKind, RenderPoint, RenderRect,
    },
};

pub const CRACK_REGION_COUNT: usize = 8;
pub const MAX_CRACK_SIZE: usize = 7;
pub const DEFAULT_INITIAL_REQUESTS: usize = 32 * 32;
pub const DEFAULT_CAMERA_INVALIDATION: i32 = -99;
pub const DEFAULT_SHADOW_ALPHA: f32 = 0.71;
pub const DEFAULT_BROKEN_FADE_STEP: f32 = 0.1;
pub const DEFAULT_BUILD_PLAN_ALPHA: f32 = 0.33;
pub const DEFAULT_BUILD_PLAN_PULSE_PERIOD: f32 = 6.0;
pub const DEFAULT_BUILD_PLAN_PULSE_STRENGTH: f32 = 0.2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl TileBounds {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn unit() -> Self {
        Self::new(0, 0, 1, 1)
    }

    pub fn contains(&self, coord: TileCoord) -> bool {
        coord.x >= self.x
            && coord.y >= self.y
            && coord.x < self.x + self.width
            && coord.y < self.y + self.height
    }
}

impl Default for TileBounds {
    fn default() -> Self {
        Self::unit()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpatialCachePlan<T> {
    pub bounds: TileBounds,
    pub entries: Vec<T>,
}

impl<T> SpatialCachePlan<T> {
    pub fn new(bounds: TileBounds) -> Self {
        Self {
            bounds,
            entries: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn reset(&mut self, bounds: TileBounds) {
        self.bounds = bounds;
        self.entries.clear();
    }
}

impl<T> Default for SpatialCachePlan<T> {
    fn default() -> Self {
        Self::new(TileBounds::unit())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockDrawStage {
    DestroyedPlanPreview,
    TileBase,
    TileShadow,
    BuildingBase,
    BuildingCracks,
    BuildingTeamOverlay,
    BuildingStatus,
    Light,
    Darkness,
    Overlay,
    Debug,
}

impl BlockDrawStage {
    pub const ORDERED: [Self; 11] = [
        Self::DestroyedPlanPreview,
        Self::TileBase,
        Self::TileShadow,
        Self::BuildingBase,
        Self::BuildingCracks,
        Self::BuildingTeamOverlay,
        Self::BuildingStatus,
        Self::Light,
        Self::Darkness,
        Self::Overlay,
        Self::Debug,
    ];

    pub const fn ordered() -> &'static [Self; 11] {
        &Self::ORDERED
    }

    pub const fn layer(self) -> f32 {
        match self {
            Self::DestroyedPlanPreview => Layer::PLANS,
            Self::TileBase
            | Self::BuildingBase
            | Self::BuildingTeamOverlay
            | Self::BuildingStatus => Layer::BLOCK,
            Self::TileShadow => Layer::BLOCK - 1.0,
            Self::BuildingCracks => Layer::BLOCK_CRACKS,
            Self::Light => Layer::LIGHT,
            Self::Darkness => Layer::DARKNESS,
            Self::Overlay | Self::Debug => Layer::OVERLAY_UI,
        }
    }

    pub const fn is_tile_stage(self) -> bool {
        matches!(self, Self::TileBase | Self::TileShadow | Self::Light)
    }

    pub const fn is_building_stage(self) -> bool {
        matches!(
            self,
            Self::BuildingBase
                | Self::BuildingCracks
                | Self::BuildingTeamOverlay
                | Self::BuildingStatus
                | Self::Light
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileDrawPlan {
    pub coord: TileCoord,
    pub block: String,
    pub cache_layer: CacheLayer,
    pub draw_custom_shadow: bool,
    pub emits_light: bool,
    pub obstructs_light: bool,
}

impl Default for TileDrawPlan {
    fn default() -> Self {
        Self {
            coord: TileCoord::default(),
            block: String::new(),
            cache_layer: CacheLayer::None,
            draw_custom_shadow: false,
            emits_light: false,
            obstructs_light: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererTileSnapshot {
    pub coord: TileCoord,
    pub block: String,
    pub cache_layer: CacheLayer,
    pub draw_custom_shadow: bool,
    pub emits_light: bool,
    pub obstructs_light: bool,
    pub building: Option<BlockRendererBuildingSnapshot>,
}

impl BlockRendererTileSnapshot {
    pub fn new(coord: TileCoord, block: impl Into<String>) -> Self {
        Self {
            coord,
            block: block.into(),
            cache_layer: CacheLayer::None,
            draw_custom_shadow: false,
            emits_light: false,
            obstructs_light: false,
            building: None,
        }
    }

    pub fn to_draw_plan(&self) -> TileDrawPlan {
        TileDrawPlan {
            coord: self.coord,
            block: self.block.clone(),
            cache_layer: self.cache_layer,
            draw_custom_shadow: self.draw_custom_shadow,
            emits_light: self.emits_light,
            obstructs_light: self.obstructs_light,
        }
    }
}

impl Default for BlockRendererTileSnapshot {
    fn default() -> Self {
        Self::new(TileCoord::default(), "")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TilePassPlan {
    pub stage: BlockDrawStage,
    pub layer: f32,
    pub tiles: Vec<TileDrawPlan>,
}

impl TilePassPlan {
    pub fn new(stage: BlockDrawStage) -> Self {
        debug_assert!(stage.is_tile_stage());
        Self {
            stage,
            layer: stage.layer(),
            tiles: Vec::new(),
        }
    }
}

impl Default for TilePassPlan {
    fn default() -> Self {
        Self::new(BlockDrawStage::TileBase)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildingDrawPlan {
    pub coord: TileCoord,
    pub block: String,
    pub cache_layer: CacheLayer,
    pub size: u8,
    pub rotation: i16,
    pub team: u8,
    pub visible: bool,
    pub was_visible: bool,
    pub damaged: bool,
    pub draw_team_overlay: bool,
    pub draw_status: bool,
    pub emits_light: bool,
}

impl Default for BuildingDrawPlan {
    fn default() -> Self {
        Self {
            coord: TileCoord::default(),
            block: String::new(),
            cache_layer: CacheLayer::None,
            size: 1,
            rotation: 0,
            team: 0,
            visible: false,
            was_visible: false,
            damaged: false,
            draw_team_overlay: false,
            draw_status: false,
            emits_light: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererBuildingSnapshot {
    pub coord: TileCoord,
    pub block: String,
    pub cache_layer: CacheLayer,
    pub size: u8,
    pub rotation: i16,
    pub team: u8,
    pub visible: bool,
    pub was_visible: bool,
    pub damaged: bool,
    pub draw_team_overlay: bool,
    pub draw_status: bool,
    pub emits_light: bool,
}

impl BlockRendererBuildingSnapshot {
    pub fn new(coord: TileCoord, block: impl Into<String>) -> Self {
        Self {
            coord,
            block: block.into(),
            cache_layer: CacheLayer::None,
            size: 1,
            rotation: 0,
            team: 0,
            visible: false,
            was_visible: false,
            damaged: false,
            draw_team_overlay: false,
            draw_status: false,
            emits_light: false,
        }
    }

    pub fn should_draw_base(&self) -> bool {
        self.visible || self.was_visible
    }

    pub fn to_draw_plan(&self) -> BuildingDrawPlan {
        BuildingDrawPlan {
            coord: self.coord,
            block: self.block.clone(),
            cache_layer: self.cache_layer,
            size: self.size.max(1),
            rotation: self.rotation,
            team: self.team,
            visible: self.visible,
            was_visible: self.was_visible,
            damaged: self.damaged,
            draw_team_overlay: self.draw_team_overlay,
            draw_status: self.draw_status,
            emits_light: self.emits_light,
        }
    }
}

impl Default for BlockRendererBuildingSnapshot {
    fn default() -> Self {
        Self::new(TileCoord::default(), "")
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockRendererWorldSnapshot {
    pub tiles: Vec<BlockRendererTileSnapshot>,
}

impl BlockRendererWorldSnapshot {
    pub fn new(tiles: Vec<BlockRendererTileSnapshot>) -> Self {
        Self { tiles }
    }

    pub fn tile(&self, coord: TileCoord) -> Option<&BlockRendererTileSnapshot> {
        self.tiles.iter().find(|tile| tile.coord == coord)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildingPassPlan {
    pub stage: BlockDrawStage,
    pub layer: f32,
    pub buildings: Vec<BuildingDrawPlan>,
}

impl BuildingPassPlan {
    pub fn new(stage: BlockDrawStage) -> Self {
        debug_assert!(stage.is_building_stage());
        Self {
            stage,
            layer: stage.layer(),
            buildings: Vec::new(),
        }
    }
}

impl Default for BuildingPassPlan {
    fn default() -> Self {
        Self::new(BlockDrawStage::BuildingBase)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrackAtlasPlan {
    pub regions_per_size: usize,
    pub max_size: usize,
    pub loaded: bool,
}

impl CrackAtlasPlan {
    pub const fn new(regions_per_size: usize, max_size: usize) -> Self {
        Self {
            regions_per_size,
            max_size,
            loaded: false,
        }
    }
}

impl Default for CrackAtlasPlan {
    fn default() -> Self {
        Self::new(CRACK_REGION_COUNT, MAX_CRACK_SIZE)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrackPlan {
    pub coord: TileCoord,
    pub size: u8,
    pub region_index: usize,
    pub layer: f32,
    pub mirrored: bool,
}

impl CrackPlan {
    pub fn new(coord: TileCoord, size: u8, region_index: usize) -> Self {
        Self {
            coord,
            size: size.clamp(1, MAX_CRACK_SIZE as u8),
            region_index: region_index % CRACK_REGION_COUNT,
            layer: Layer::BLOCK_CRACKS,
            mirrored: false,
        }
    }
}

impl Default for CrackPlan {
    fn default() -> Self {
        Self::new(TileCoord::default(), 1, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DarknessFill {
    #[default]
    White,
    Black,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DarknessTilePlan {
    pub coord: TileCoord,
    pub darkness: f32,
    pub opacity: f32,
    pub layer: f32,
}

impl DarknessTilePlan {
    pub fn new(coord: TileCoord, darkness: f32) -> Self {
        Self {
            coord,
            darkness,
            opacity: darkness_to_opacity(darkness),
            layer: Layer::DARKNESS,
        }
    }
}

impl Default for DarknessTilePlan {
    fn default() -> Self {
        Self::new(TileCoord::default(), 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DarknessPlan {
    pub layer: f32,
    pub fill: DarknessFill,
    pub limited_map_area: Option<TileBounds>,
    /// 由 `dark_events` 驱动的脏 tile，仅表示需要重绘的坐标，不伪造 darkness 数值。
    pub dirty_tiles: Vec<TileCoord>,
    pub tiles: Vec<DarknessTilePlan>,
}

impl DarknessPlan {
    pub fn effective_fill(&self) -> DarknessFill {
        if self.limited_map_area.is_some() {
            DarknessFill::Black
        } else {
            self.fill
        }
    }

    pub fn push_tile(&mut self, coord: TileCoord, darkness: f32) {
        self.tiles.push(DarknessTilePlan::new(coord, darkness));
    }

    pub fn push_dirty_tile(&mut self, coord: TileCoord) {
        self.dirty_tiles.push(coord);
    }
}

impl Default for DarknessPlan {
    fn default() -> Self {
        Self {
            layer: Layer::DARKNESS,
            fill: DarknessFill::White,
            limited_map_area: None,
            dirty_tiles: Vec::new(),
            tiles: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverlayKind {
    #[default]
    Team,
    Status,
    DebugBounds,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayPlan {
    pub coord: TileCoord,
    pub kind: OverlayKind,
    pub layer: f32,
    pub tint: DecalColor,
    pub alpha: f32,
    pub visible: bool,
}

impl OverlayPlan {
    pub fn team(coord: TileCoord) -> Self {
        Self {
            coord,
            kind: OverlayKind::Team,
            layer: Layer::BLOCK,
            tint: DecalColor::WHITE,
            alpha: 1.0,
            visible: true,
        }
    }

    pub fn status(coord: TileCoord) -> Self {
        Self {
            coord,
            kind: OverlayKind::Status,
            layer: Layer::BLOCK,
            tint: DecalColor::WHITE,
            alpha: 1.0,
            visible: true,
        }
    }

    pub fn debug_bounds(coord: TileCoord) -> Self {
        Self {
            coord,
            kind: OverlayKind::DebugBounds,
            layer: Layer::OVERLAY_UI,
            tint: DecalColor::WHITE,
            alpha: 1.0,
            visible: true,
        }
    }
}

impl Default for OverlayPlan {
    fn default() -> Self {
        Self {
            coord: TileCoord::default(),
            kind: OverlayKind::Team,
            layer: Layer::BLOCK,
            tint: DecalColor::WHITE,
            alpha: 1.0,
            visible: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlanPreview {
    pub coord: TileCoord,
    pub block: String,
    pub size: u8,
    pub rotation: i16,
    pub layer: f32,
    pub alpha: f32,
    pub pulse_period: f32,
    pub pulse_strength: f32,
    pub tint: DecalColor,
}

impl BuildPlanPreview {
    pub fn new(coord: TileCoord, block: impl Into<String>, size: u8, rotation: i16) -> Self {
        Self {
            coord,
            block: block.into(),
            size: size.max(1),
            rotation,
            layer: Layer::PLANS,
            alpha: DEFAULT_BUILD_PLAN_ALPHA,
            pulse_period: DEFAULT_BUILD_PLAN_PULSE_PERIOD,
            pulse_strength: DEFAULT_BUILD_PLAN_PULSE_STRENGTH,
            tint: DecalColor::WHITE,
        }
    }

    pub fn draw_alpha(&self, broken_fade: f32) -> f32 {
        (self.alpha * broken_fade.max(0.0)).clamp(0.0, 1.0)
    }
}

impl Default for BuildPlanPreview {
    fn default() -> Self {
        Self::new(TileCoord::default(), "", 1, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockRendererVisuals {
    pub shadow_color: DecalColor,
    pub blend_shadow_color: DecalColor,
    pub broken_fade_step: f32,
}

impl Default for BlockRendererVisuals {
    fn default() -> Self {
        Self {
            shadow_color: DecalColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: DEFAULT_SHADOW_ALPHA,
            },
            blend_shadow_color: blend_shadow_color(DEFAULT_SHADOW_ALPHA),
            broken_fade_step: DEFAULT_BROKEN_FADE_STEP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraCache {
    pub avg_x: i32,
    pub avg_y: i32,
    pub range_x: i32,
    pub range_y: i32,
}

impl CameraCache {
    pub const fn new(avg_x: i32, avg_y: i32, range_x: i32, range_y: i32) -> Self {
        Self {
            avg_x,
            avg_y,
            range_x,
            range_y,
        }
    }

    pub fn invalidate(&mut self) {
        self.avg_x = DEFAULT_CAMERA_INVALIDATION;
        self.avg_y = DEFAULT_CAMERA_INVALIDATION;
    }
}

impl Default for CameraCache {
    fn default() -> Self {
        Self::new(
            DEFAULT_CAMERA_INVALIDATION,
            DEFAULT_CAMERA_INVALIDATION,
            0,
            0,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererCache {
    pub block_tree: SpatialCachePlan<TileCoord>,
    pub block_light_tree: SpatialCachePlan<TileCoord>,
    pub overlay_tree: SpatialCachePlan<TileCoord>,
    pub floor_tree: SpatialCachePlan<TileCoord>,
    pub tile_view: Vec<TileCoord>,
    pub light_view: Vec<TileCoord>,
    pub update_floors: Vec<TileCoord>,
    pub shadow_events: BTreeSet<TileCoord>,
    pub dark_events: BTreeSet<TileCoord>,
    pub proc_links: BTreeSet<i32>,
    pub proc_lights: BTreeSet<i32>,
}

impl BlockRendererCache {
    pub fn clear_frame_queues(&mut self) {
        self.tile_view.clear();
        self.light_view.clear();
        self.update_floors.clear();
        self.shadow_events.clear();
        self.dark_events.clear();
        self.proc_links.clear();
        self.proc_lights.clear();
    }

    pub fn reset(&mut self, bounds: TileBounds) {
        self.block_tree.reset(bounds);
        self.block_light_tree.reset(bounds);
        self.overlay_tree.reset(bounds);
        self.floor_tree.reset(bounds);
        self.clear_frame_queues();
    }
}

impl Default for BlockRendererCache {
    fn default() -> Self {
        Self {
            block_tree: SpatialCachePlan::default(),
            block_light_tree: SpatialCachePlan::default(),
            overlay_tree: SpatialCachePlan::default(),
            floor_tree: SpatialCachePlan::default(),
            tile_view: Vec::with_capacity(DEFAULT_INITIAL_REQUESTS),
            light_view: Vec::with_capacity(DEFAULT_INITIAL_REQUESTS),
            update_floors: Vec::new(),
            shadow_events: BTreeSet::new(),
            dark_events: BTreeSet::new(),
            proc_links: BTreeSet::new(),
            proc_lights: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererState {
    pub draw_quadtree_debug: bool,
    pub had_map_limit: bool,
    pub last_camera: CameraCache,
    pub broken_fade: f32,
    pub visuals: BlockRendererVisuals,
    pub crack_atlas: CrackAtlasPlan,
    pub cache: BlockRendererCache,
}

impl BlockRendererState {
    pub fn reload(bounds: TileBounds, had_map_limit: bool) -> Self {
        let mut state = Self::default();
        state.had_map_limit = had_map_limit;
        state.last_camera.invalidate();
        state.cache.reset(bounds);
        state
    }

    pub fn invalidate_camera(&mut self) {
        self.last_camera.invalidate();
    }

    pub fn build_plan(&self) -> BlockRendererPlan {
        self.build_plan_from_snapshot(&BlockRendererWorldSnapshot::default())
    }

    pub fn build_plan_from_snapshot(
        &self,
        snapshot: &BlockRendererWorldSnapshot,
    ) -> BlockRendererPlan {
        let mut plan = BlockRendererPlan::default();
        let snapshot_tiles = snapshot_tiles_by_coord(snapshot);

        plan.broken_fade = self.broken_fade;
        plan.draw_quadtree_debug = self.draw_quadtree_debug;
        plan.update_floors = self.cache.update_floors.clone();

        if let Some(pass) = build_tile_pass_from_snapshot(
            BlockDrawStage::TileBase,
            self.cache.tile_view.iter().copied(),
            &snapshot_tiles,
        ) {
            plan.tile_passes.push(pass);
        }

        if let Some(pass) = build_tile_pass_from_snapshot(
            BlockDrawStage::TileShadow,
            self.cache.shadow_events.iter().copied(),
            &snapshot_tiles,
        ) {
            plan.tile_passes.push(pass);
        }

        if let Some(pass) = build_tile_pass_from_snapshot(
            BlockDrawStage::Light,
            self.cache.light_view.iter().copied(),
            &snapshot_tiles,
        ) {
            plan.tile_passes.push(pass);
        }

        append_building_passes_from_snapshot(&mut plan, self, &snapshot_tiles);

        if !self.cache.dark_events.is_empty() {
            plan.darkness.dirty_tiles = self.cache.dark_events.iter().copied().collect();
        }

        if self.draw_quadtree_debug {
            plan.overlays.extend(quadtree_debug_overlays(
                self.cache.block_tree.bounds,
                self.cache.block_light_tree.bounds,
                self.cache.overlay_tree.bounds,
                self.cache.floor_tree.bounds,
            ));
        }

        plan
    }
}

impl Default for BlockRendererState {
    fn default() -> Self {
        Self {
            draw_quadtree_debug: false,
            had_map_limit: false,
            last_camera: CameraCache::default(),
            broken_fade: 0.0,
            visuals: BlockRendererVisuals::default(),
            crack_atlas: CrackAtlasPlan::default(),
            cache: BlockRendererCache::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererPlan {
    pub tile_passes: Vec<TilePassPlan>,
    pub building_passes: Vec<BuildingPassPlan>,
    pub cracks: Vec<CrackPlan>,
    pub build_previews: Vec<BuildPlanPreview>,
    pub darkness: DarknessPlan,
    pub overlays: Vec<OverlayPlan>,
    pub update_floors: Vec<TileCoord>,
    pub draw_quadtree_debug: bool,
    pub broken_fade: f32,
}

impl BlockRendererPlan {
    pub fn clear(&mut self) {
        self.tile_passes.clear();
        self.building_passes.clear();
        self.cracks.clear();
        self.build_previews.clear();
        self.darkness.tiles.clear();
        self.darkness.dirty_tiles.clear();
        self.darkness.limited_map_area = None;
        self.darkness.fill = DarknessFill::White;
        self.overlays.clear();
        self.update_floors.clear();
        self.draw_quadtree_debug = false;
        self.broken_fade = 0.0;
    }

    pub fn is_empty(&self) -> bool {
        self.tile_passes.is_empty()
            && self.building_passes.is_empty()
            && self.cracks.is_empty()
            && self.build_previews.is_empty()
            && self.darkness.tiles.is_empty()
            && self.darkness.dirty_tiles.is_empty()
            && self.darkness.limited_map_area.is_none()
            && matches!(self.darkness.fill, DarknessFill::White)
            && self.overlays.is_empty()
            && self.update_floors.is_empty()
            && !self.draw_quadtree_debug
            && self.broken_fade <= 0.0
    }

    pub fn to_sprite_render_passes(&self, tile_size_world: f32) -> Vec<RenderPass> {
        if tile_size_world <= 0.0 {
            return Vec::new();
        }

        let mut passes = Vec::new();
        let mut order = RenderPassKind::Block.default_order();

        for pass in &self.tile_passes {
            if let Some(render_pass) = pass.to_sprite_render_pass(tile_size_world, order) {
                passes.push(render_pass);
                order += 1;
            }
        }

        for pass in &self.building_passes {
            if let Some(render_pass) = pass.to_sprite_render_pass(tile_size_world, order) {
                passes.push(render_pass);
                order += 1;
            }
        }

        passes
    }
}

impl Default for BlockRendererPlan {
    fn default() -> Self {
        Self {
            tile_passes: Vec::new(),
            building_passes: Vec::new(),
            cracks: Vec::new(),
            build_previews: Vec::new(),
            darkness: DarknessPlan::default(),
            overlays: Vec::new(),
            update_floors: Vec::new(),
            draw_quadtree_debug: false,
            broken_fade: 0.0,
        }
    }
}

pub fn darkness_to_opacity(darkness: f32) -> f32 {
    if darkness <= 0.0 {
        1.0
    } else {
        1.0 - ((darkness + 0.5) / 4.0).min(1.0)
    }
}

const fn lerp(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

const fn blend_shadow_color(alpha: f32) -> DecalColor {
    DecalColor {
        r: lerp(1.0, 0.0, alpha),
        g: lerp(1.0, 0.0, alpha),
        b: lerp(1.0, 0.0, alpha),
        a: 1.0,
    }
}

fn snapshot_tiles_by_coord(
    snapshot: &BlockRendererWorldSnapshot,
) -> BTreeMap<TileCoord, &BlockRendererTileSnapshot> {
    snapshot
        .tiles
        .iter()
        .map(|tile| (tile.coord, tile))
        .collect()
}

fn tile_draw_plan_from_snapshot(
    coord: TileCoord,
    snapshot_tiles: &BTreeMap<TileCoord, &BlockRendererTileSnapshot>,
) -> TileDrawPlan {
    snapshot_tiles
        .get(&coord)
        .map(|tile| tile.to_draw_plan())
        .unwrap_or_else(|| TileDrawPlan {
            coord,
            ..TileDrawPlan::default()
        })
}

fn build_tile_pass_from_snapshot<I>(
    stage: BlockDrawStage,
    coords: I,
    snapshot_tiles: &BTreeMap<TileCoord, &BlockRendererTileSnapshot>,
) -> Option<TilePassPlan>
where
    I: IntoIterator<Item = TileCoord>,
{
    let tiles = coords
        .into_iter()
        .map(|coord| tile_draw_plan_from_snapshot(coord, snapshot_tiles))
        .collect::<Vec<_>>();

    if tiles.is_empty() {
        None
    } else {
        Some(TilePassPlan {
            stage,
            layer: stage.layer(),
            tiles,
        })
    }
}

fn append_building_passes_from_snapshot(
    plan: &mut BlockRendererPlan,
    state: &BlockRendererState,
    snapshot_tiles: &BTreeMap<TileCoord, &BlockRendererTileSnapshot>,
) {
    let buildings = state
        .cache
        .tile_view
        .iter()
        .filter_map(|coord| snapshot_tiles.get(coord))
        .filter_map(|tile| tile.building.as_ref())
        .filter(|building| building.should_draw_base())
        .map(BlockRendererBuildingSnapshot::to_draw_plan)
        .collect::<Vec<_>>();

    push_building_pass(
        plan,
        BlockDrawStage::BuildingBase,
        buildings.iter().cloned(),
    );
    push_building_pass(
        plan,
        BlockDrawStage::BuildingCracks,
        buildings
            .iter()
            .filter(|building| building.damaged)
            .cloned(),
    );
    push_building_pass(
        plan,
        BlockDrawStage::BuildingTeamOverlay,
        buildings
            .iter()
            .filter(|building| building.draw_team_overlay)
            .cloned(),
    );
    push_building_pass(
        plan,
        BlockDrawStage::BuildingStatus,
        buildings
            .iter()
            .filter(|building| building.draw_status)
            .cloned(),
    );

    let light_buildings = state
        .cache
        .light_view
        .iter()
        .filter_map(|coord| snapshot_tiles.get(coord))
        .filter_map(|tile| tile.building.as_ref())
        .filter(|building| building.emits_light)
        .map(BlockRendererBuildingSnapshot::to_draw_plan);
    push_building_pass(plan, BlockDrawStage::Light, light_buildings);
}

fn push_building_pass<I>(plan: &mut BlockRendererPlan, stage: BlockDrawStage, buildings: I)
where
    I: IntoIterator<Item = BuildingDrawPlan>,
{
    let buildings = buildings.into_iter().collect::<Vec<_>>();
    if !buildings.is_empty() {
        plan.building_passes.push(BuildingPassPlan {
            stage,
            layer: stage.layer(),
            buildings,
        });
    }
}

const SPRITE_TINT_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const SPRITE_SYMBOL_BLOCK_SHADOW: &str = "block-shadow";
const SPRITE_SYMBOL_BLOCK_CRACKS: &str = "block-cracks";
const SPRITE_SYMBOL_BLOCK_TEAM: &str = "block-team";
const SPRITE_SYMBOL_BLOCK_STATUS: &str = "block-status";
const SPRITE_SYMBOL_BLOCK_LIGHT: &str = "block-light";

impl TilePassPlan {
    pub fn to_sprite_render_pass(&self, tile_size_world: f32, order: i32) -> Option<RenderPass> {
        let mut render_pass = RenderPass::new(RenderPassKind::Block).with_order(order);
        let mut has_commands = false;

        for tile in &self.tiles {
            let symbol = match self.stage {
                BlockDrawStage::TileBase => tile.block.as_str(),
                BlockDrawStage::TileShadow => SPRITE_SYMBOL_BLOCK_SHADOW,
                BlockDrawStage::Light => SPRITE_SYMBOL_BLOCK_LIGHT,
                _ => "",
            };

            if symbol.is_empty() {
                continue;
            }

            render_pass.push(RenderCommand::draw_sprite(
                symbol,
                tile_sprite_rect(tile.coord, tile_size_world),
                SPRITE_TINT_WHITE,
                0.0,
                self.layer,
            ));
            has_commands = true;
        }

        has_commands.then_some(render_pass)
    }
}

impl BuildingPassPlan {
    pub fn to_sprite_render_pass(&self, tile_size_world: f32, order: i32) -> Option<RenderPass> {
        let mut render_pass = RenderPass::new(RenderPassKind::Block).with_order(order);
        let mut has_commands = false;

        for building in &self.buildings {
            let symbol = match self.stage {
                BlockDrawStage::BuildingBase => building.block.as_str(),
                BlockDrawStage::BuildingCracks => SPRITE_SYMBOL_BLOCK_CRACKS,
                BlockDrawStage::BuildingTeamOverlay => SPRITE_SYMBOL_BLOCK_TEAM,
                BlockDrawStage::BuildingStatus => SPRITE_SYMBOL_BLOCK_STATUS,
                BlockDrawStage::Light => SPRITE_SYMBOL_BLOCK_LIGHT,
                _ => "",
            };

            if symbol.is_empty() {
                continue;
            }

            render_pass.push(RenderCommand::draw_sprite(
                symbol,
                building_sprite_rect(building.coord, building.size, tile_size_world),
                SPRITE_TINT_WHITE,
                building_rotation_degrees(building.rotation),
                self.layer,
            ));
            has_commands = true;
        }

        has_commands.then_some(render_pass)
    }
}

fn tile_sprite_rect(coord: TileCoord, tile_size_world: f32) -> RenderRect {
    let center_x = coord.x as f32 * tile_size_world + tile_size_world / 2.0;
    let center_y = coord.y as f32 * tile_size_world + tile_size_world / 2.0;
    RenderRect::from_center(
        RenderPoint::new(center_x, center_y),
        tile_size_world,
        tile_size_world,
    )
}

fn building_sprite_rect(coord: TileCoord, size: u8, tile_size_world: f32) -> RenderRect {
    let size = size.max(1);
    let size_world = size as f32 * tile_size_world;
    let center_offset = if size % 2 == 1 {
        tile_size_world / 2.0
    } else {
        0.0
    };
    let center_x = coord.x as f32 * tile_size_world + center_offset;
    let center_y = coord.y as f32 * tile_size_world + center_offset;
    RenderRect::from_center(RenderPoint::new(center_x, center_y), size_world, size_world)
}

fn building_rotation_degrees(rotation: i16) -> f32 {
    rotation.rem_euclid(4) as f32 * 90.0
}

fn quadtree_debug_overlays(
    block_tree: TileBounds,
    block_light_tree: TileBounds,
    overlay_tree: TileBounds,
    floor_tree: TileBounds,
) -> Vec<OverlayPlan> {
    [block_tree, block_light_tree, overlay_tree, floor_tree]
        .iter()
        .copied()
        .map(|bounds| OverlayPlan::debug_bounds(TileCoord::new(bounds.x, bounds.y)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_defaults_mirror_upstream_renderer_bootstrap_values() {
        let state = BlockRendererState::default();

        assert!(!state.draw_quadtree_debug);
        assert!(!state.had_map_limit);
        assert_eq!(state.broken_fade, 0.0);
        assert_eq!(state.last_camera, CameraCache::default());
        assert_eq!(state.visuals.broken_fade_step, DEFAULT_BROKEN_FADE_STEP);
        assert_eq!(state.crack_atlas.regions_per_size, CRACK_REGION_COUNT);
        assert_eq!(state.crack_atlas.max_size, MAX_CRACK_SIZE);
        assert!(!state.crack_atlas.loaded);
        assert_eq!(state.cache.block_tree.bounds, TileBounds::unit());
        assert!(state.cache.tile_view.capacity() >= DEFAULT_INITIAL_REQUESTS);
        assert!(state.cache.light_view.capacity() >= DEFAULT_INITIAL_REQUESTS);

        assert_eq!(state.visuals.shadow_color.a, DEFAULT_SHADOW_ALPHA);
        assert!((state.visuals.blend_shadow_color.r - 0.29).abs() < 0.0001);
        assert!((state.visuals.blend_shadow_color.g - 0.29).abs() < 0.0001);
        assert!((state.visuals.blend_shadow_color.b - 0.29).abs() < 0.0001);
        assert_eq!(state.visuals.blend_shadow_color.a, 1.0);
    }

    #[test]
    fn stage_order_and_default_layers_match_the_java_render_pipeline() {
        assert_eq!(
            BlockDrawStage::ordered(),
            &[
                BlockDrawStage::DestroyedPlanPreview,
                BlockDrawStage::TileBase,
                BlockDrawStage::TileShadow,
                BlockDrawStage::BuildingBase,
                BlockDrawStage::BuildingCracks,
                BlockDrawStage::BuildingTeamOverlay,
                BlockDrawStage::BuildingStatus,
                BlockDrawStage::Light,
                BlockDrawStage::Darkness,
                BlockDrawStage::Overlay,
                BlockDrawStage::Debug,
            ]
        );

        assert_eq!(BlockDrawStage::DestroyedPlanPreview.layer(), Layer::PLANS);
        assert_eq!(BlockDrawStage::TileBase.layer(), Layer::BLOCK);
        assert_eq!(BlockDrawStage::TileShadow.layer(), Layer::BLOCK - 1.0);
        assert_eq!(BlockDrawStage::BuildingCracks.layer(), Layer::BLOCK_CRACKS);
        assert_eq!(BlockDrawStage::Darkness.layer(), Layer::DARKNESS);
        assert_eq!(BlockDrawStage::Overlay.layer(), Layer::OVERLAY_UI);
        assert_eq!(BlockDrawStage::Debug.layer(), Layer::OVERLAY_UI);
    }

    #[test]
    fn crack_build_plan_darkness_and_overlay_defaults_are_data_only() {
        let crack = CrackPlan::new(TileCoord::new(3, 4), 99, 17);
        assert_eq!(crack.coord, TileCoord::new(3, 4));
        assert_eq!(crack.size, MAX_CRACK_SIZE as u8);
        assert_eq!(crack.region_index, 1);
        assert_eq!(crack.layer, Layer::BLOCK_CRACKS);
        assert!(!crack.mirrored);

        let preview = BuildPlanPreview::new(TileCoord::new(5, 6), "router", 2, 1);
        assert_eq!(preview.layer, Layer::PLANS);
        assert_eq!(preview.draw_alpha(1.0), DEFAULT_BUILD_PLAN_ALPHA);
        assert_eq!(preview.block, "router");
        assert_eq!(preview.size, 2);
        assert_eq!(preview.rotation, 1);

        let darkness = DarknessTilePlan::new(TileCoord::new(1, 2), 1.5);
        assert!((darkness.opacity - 0.5).abs() < 0.0001);
        assert_eq!(darkness.layer, Layer::DARKNESS);

        let mut darkness_plan = DarknessPlan::default();
        darkness_plan.limited_map_area = Some(TileBounds::new(0, 0, 8, 8));
        assert_eq!(darkness_plan.effective_fill(), DarknessFill::Black);
        darkness_plan.push_tile(TileCoord::new(1, 2), 0.75);
        assert_eq!(darkness_plan.tiles.len(), 1);

        assert_eq!(
            OverlayPlan::team(TileCoord::new(7, 8)),
            OverlayPlan {
                coord: TileCoord::new(7, 8),
                kind: OverlayKind::Team,
                layer: Layer::BLOCK,
                tint: DecalColor::WHITE,
                alpha: 1.0,
                visible: true,
            }
        );
        assert_eq!(
            OverlayPlan::debug_bounds(TileCoord::new(7, 8)).layer,
            Layer::OVERLAY_UI
        );
    }

    #[test]
    fn renderer_plan_and_cache_can_be_reset_without_gpu_state() {
        let mut state = BlockRendererState::reload(TileBounds::new(0, 0, 16, 16), true);
        state.cache.tile_view.push(TileCoord::new(1, 1));
        state.cache.light_view.push(TileCoord::new(2, 2));
        state.cache.shadow_events.insert(TileCoord::new(3, 3));
        state.cache.dark_events.insert(TileCoord::new(4, 4));
        state.cache.update_floors.push(TileCoord::new(5, 5));
        state.cache.proc_links.insert(44);
        state.cache.proc_lights.insert(55);
        state.draw_quadtree_debug = true;
        state.broken_fade = 0.5;

        state.cache.clear_frame_queues();
        assert!(state.cache.tile_view.is_empty());
        assert!(state.cache.light_view.is_empty());
        assert!(state.cache.shadow_events.is_empty());
        assert!(state.cache.dark_events.is_empty());
        assert!(state.cache.update_floors.is_empty());
        assert!(state.cache.proc_links.is_empty());
        assert!(state.cache.proc_lights.is_empty());

        let empty_plan = BlockRendererState::default().build_plan();
        assert!(empty_plan.is_empty());
        assert!(empty_plan.tile_passes.is_empty());
        assert!(empty_plan.building_passes.is_empty());
        assert!(empty_plan.darkness.dirty_tiles.is_empty());
        assert!(empty_plan.update_floors.is_empty());
        assert!(!empty_plan.draw_quadtree_debug);
        assert_eq!(empty_plan.broken_fade, 0.0);

        let mut plan = BlockRendererPlan::default();
        plan.tile_passes.push(TilePassPlan::default());
        plan.building_passes.push(BuildingPassPlan::default());
        plan.cracks.push(CrackPlan::default());
        plan.build_previews.push(BuildPlanPreview::default());
        plan.darkness.push_tile(TileCoord::new(1, 2), 2.0);
        plan.darkness.push_dirty_tile(TileCoord::new(3, 4));
        plan.overlays.push(OverlayPlan::default());
        plan.update_floors.push(TileCoord::new(5, 6));
        plan.draw_quadtree_debug = true;
        plan.broken_fade = 0.25;
        assert!(!plan.is_empty());

        plan.clear();
        assert!(plan.is_empty());
        assert_eq!(plan.darkness.fill, DarknessFill::White);
        assert!(plan.darkness.tiles.is_empty());
        assert!(plan.darkness.dirty_tiles.is_empty());
        assert!(plan.update_floors.is_empty());
        assert!(!plan.draw_quadtree_debug);
        assert_eq!(plan.broken_fade, 0.0);
    }

    #[test]
    fn build_plan_preserves_tile_shadow_light_dark_and_debug_queues() {
        let mut state = BlockRendererState::default();
        state.cache.tile_view = vec![TileCoord::new(1, 1), TileCoord::new(2, 2)];
        state.cache.light_view = vec![TileCoord::new(3, 3)];
        state.cache.shadow_events.insert(TileCoord::new(4, 4));
        state.cache.dark_events.insert(TileCoord::new(5, 5));
        state.cache.update_floors.push(TileCoord::new(6, 6));
        state.draw_quadtree_debug = true;
        state.broken_fade = 0.75;

        let plan = state.build_plan();

        assert!(!plan.is_empty());
        assert_eq!(plan.broken_fade, 0.75);
        assert!(plan.draw_quadtree_debug);
        assert_eq!(plan.update_floors, vec![TileCoord::new(6, 6)]);
        assert_eq!(plan.tile_passes.len(), 3);
        assert_eq!(plan.tile_passes[0].stage, BlockDrawStage::TileBase);
        assert_eq!(
            plan.tile_passes[0]
                .tiles
                .iter()
                .map(|tile| tile.coord)
                .collect::<Vec<_>>(),
            vec![TileCoord::new(1, 1), TileCoord::new(2, 2)]
        );
        assert_eq!(plan.tile_passes[1].stage, BlockDrawStage::TileShadow);
        assert_eq!(
            plan.tile_passes[1]
                .tiles
                .iter()
                .map(|tile| tile.coord)
                .collect::<Vec<_>>(),
            vec![TileCoord::new(4, 4)]
        );
        assert_eq!(plan.tile_passes[2].stage, BlockDrawStage::Light);
        assert_eq!(
            plan.tile_passes[2]
                .tiles
                .iter()
                .map(|tile| tile.coord)
                .collect::<Vec<_>>(),
            vec![TileCoord::new(3, 3)]
        );
        assert_eq!(plan.darkness.dirty_tiles, vec![TileCoord::new(5, 5)]);
        assert_eq!(plan.overlays.len(), 4);
        assert!(plan
            .overlays
            .iter()
            .all(|overlay| overlay.kind == OverlayKind::DebugBounds));
    }

    #[test]
    fn build_plan_from_snapshot_populates_real_tile_fields() {
        let mut state = BlockRendererState::default();
        state.cache.tile_view = vec![TileCoord::new(1, 1), TileCoord::new(9, 9)];
        state.cache.light_view = vec![TileCoord::new(2, 2)];
        state.cache.shadow_events.insert(TileCoord::new(3, 3));

        let mut visible = BlockRendererTileSnapshot::new(TileCoord::new(1, 1), "router");
        visible.cache_layer = CacheLayer::Normal;
        visible.draw_custom_shadow = true;
        visible.emits_light = false;
        visible.obstructs_light = true;

        let mut light = BlockRendererTileSnapshot::new(TileCoord::new(2, 2), "illuminator");
        light.cache_layer = CacheLayer::Normal;
        light.emits_light = true;
        light.obstructs_light = false;

        let mut shadow = BlockRendererTileSnapshot::new(TileCoord::new(3, 3), "copper-wall");
        shadow.cache_layer = CacheLayer::Walls;
        shadow.draw_custom_shadow = true;
        shadow.obstructs_light = true;

        let snapshot = BlockRendererWorldSnapshot::new(vec![visible, light, shadow]);
        let plan = state.build_plan_from_snapshot(&snapshot);

        let tile_base = &plan.tile_passes[0].tiles;
        assert_eq!(tile_base[0].coord, TileCoord::new(1, 1));
        assert_eq!(tile_base[0].block, "router");
        assert_eq!(tile_base[0].cache_layer, CacheLayer::Normal);
        assert!(tile_base[0].draw_custom_shadow);
        assert!(tile_base[0].obstructs_light);

        // Missing snapshot data keeps the old coord-only fallback explicit.
        assert_eq!(tile_base[1].coord, TileCoord::new(9, 9));
        assert_eq!(tile_base[1].block, "");
        assert_eq!(tile_base[1].cache_layer, CacheLayer::None);

        assert_eq!(plan.tile_passes[1].stage, BlockDrawStage::TileShadow);
        assert_eq!(plan.tile_passes[1].tiles[0].block, "copper-wall");
        assert_eq!(plan.tile_passes[1].tiles[0].cache_layer, CacheLayer::Walls);
        assert!(plan.tile_passes[1].tiles[0].draw_custom_shadow);

        assert_eq!(plan.tile_passes[2].stage, BlockDrawStage::Light);
        assert_eq!(plan.tile_passes[2].tiles[0].block, "illuminator");
        assert!(plan.tile_passes[2].tiles[0].emits_light);
        assert!(!plan.tile_passes[2].tiles[0].obstructs_light);
    }

    #[test]
    fn build_plan_from_snapshot_populates_building_pass_fields() {
        let mut state = BlockRendererState::default();
        state.cache.tile_view = vec![
            TileCoord::new(4, 4),
            TileCoord::new(5, 5),
            TileCoord::new(6, 6),
        ];
        state.cache.light_view = vec![TileCoord::new(4, 4)];

        let mut damaged_building = BlockRendererBuildingSnapshot::new(TileCoord::new(4, 4), "duo");
        damaged_building.cache_layer = CacheLayer::Normal;
        damaged_building.size = 2;
        damaged_building.rotation = 3;
        damaged_building.team = 7;
        damaged_building.visible = true;
        damaged_building.damaged = true;
        damaged_building.draw_team_overlay = true;
        damaged_building.draw_status = true;
        damaged_building.emits_light = true;

        let mut remembered_building =
            BlockRendererBuildingSnapshot::new(TileCoord::new(5, 5), "mender");
        remembered_building.rotation = 1;
        remembered_building.team = 2;
        remembered_building.visible = false;
        remembered_building.was_visible = true;
        remembered_building.draw_status = true;

        let mut hidden_building =
            BlockRendererBuildingSnapshot::new(TileCoord::new(6, 6), "hidden-core");
        hidden_building.visible = false;
        hidden_building.was_visible = false;
        hidden_building.damaged = true;

        let snapshot = BlockRendererWorldSnapshot::new(vec![
            BlockRendererTileSnapshot {
                coord: TileCoord::new(4, 4),
                block: "duo".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: true,
                obstructs_light: true,
                building: Some(damaged_building),
            },
            BlockRendererTileSnapshot {
                coord: TileCoord::new(5, 5),
                block: "mender".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: false,
                obstructs_light: true,
                building: Some(remembered_building),
            },
            BlockRendererTileSnapshot {
                coord: TileCoord::new(6, 6),
                block: "hidden-core".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: false,
                obstructs_light: true,
                building: Some(hidden_building),
            },
        ]);

        let plan = state.build_plan_from_snapshot(&snapshot);

        assert_eq!(plan.building_passes.len(), 5);
        assert_eq!(plan.building_passes[0].stage, BlockDrawStage::BuildingBase);
        assert_eq!(plan.building_passes[0].buildings.len(), 2);

        let duo = &plan.building_passes[0].buildings[0];
        assert_eq!(duo.block, "duo");
        assert_eq!(duo.cache_layer, CacheLayer::Normal);
        assert_eq!(duo.size, 2);
        assert_eq!(duo.rotation, 3);
        assert_eq!(duo.team, 7);
        assert!(duo.visible);
        assert!(duo.damaged);
        assert!(duo.draw_team_overlay);
        assert!(duo.draw_status);
        assert!(duo.emits_light);

        let remembered = &plan.building_passes[0].buildings[1];
        assert_eq!(remembered.block, "mender");
        assert!(!remembered.visible);
        assert!(remembered.was_visible);
        assert_eq!(remembered.rotation, 1);
        assert_eq!(remembered.team, 2);

        assert_eq!(
            plan.building_passes[1].stage,
            BlockDrawStage::BuildingCracks
        );
        assert_eq!(plan.building_passes[1].buildings[0].block, "duo");
        assert_eq!(
            plan.building_passes[2].stage,
            BlockDrawStage::BuildingTeamOverlay
        );
        assert_eq!(plan.building_passes[2].buildings[0].block, "duo");
        assert_eq!(
            plan.building_passes[3].stage,
            BlockDrawStage::BuildingStatus
        );
        assert_eq!(
            plan.building_passes[3]
                .buildings
                .iter()
                .map(|building| building.block.as_str())
                .collect::<Vec<_>>(),
            vec!["duo", "mender"]
        );
        assert_eq!(plan.building_passes[4].stage, BlockDrawStage::Light);
        assert_eq!(plan.building_passes[4].buildings[0].block, "duo");
        assert!(plan
            .building_passes
            .iter()
            .flat_map(|pass| pass.buildings.iter())
            .all(|building| building.block != "hidden-core"));
    }

    #[test]
    fn block_renderer_plan_converts_sprite_passes_with_stable_symbols_and_rotation() {
        let mut plan = BlockRendererPlan::default();

        plan.tile_passes.push(TilePassPlan {
            stage: BlockDrawStage::TileBase,
            layer: BlockDrawStage::TileBase.layer(),
            tiles: vec![TileDrawPlan {
                coord: TileCoord::new(2, 3),
                block: "router".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: false,
                obstructs_light: false,
            }],
        });
        plan.tile_passes.push(TilePassPlan {
            stage: BlockDrawStage::TileShadow,
            layer: BlockDrawStage::TileShadow.layer(),
            tiles: vec![TileDrawPlan {
                coord: TileCoord::new(2, 3),
                block: "ignored-shadow".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: false,
                obstructs_light: false,
            }],
        });
        plan.tile_passes.push(TilePassPlan {
            stage: BlockDrawStage::Light,
            layer: BlockDrawStage::Light.layer(),
            tiles: vec![TileDrawPlan {
                coord: TileCoord::new(2, 3),
                block: "ignored-light".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: true,
                obstructs_light: false,
            }],
        });

        let building = BuildingDrawPlan {
            coord: TileCoord::new(4, 5),
            block: "duo".into(),
            cache_layer: CacheLayer::Normal,
            size: 2,
            rotation: 1,
            team: 7,
            visible: true,
            was_visible: false,
            damaged: true,
            draw_team_overlay: true,
            draw_status: true,
            emits_light: true,
        };

        for stage in [
            BlockDrawStage::BuildingBase,
            BlockDrawStage::BuildingCracks,
            BlockDrawStage::BuildingTeamOverlay,
            BlockDrawStage::BuildingStatus,
            BlockDrawStage::Light,
        ] {
            plan.building_passes.push(BuildingPassPlan {
                stage,
                layer: stage.layer(),
                buildings: vec![building.clone()],
            });
        }

        let passes = plan.to_sprite_render_passes(8.0);
        assert_eq!(passes.len(), 8);
        assert!(passes.windows(2).all(|pair| pair[0].order < pair[1].order));

        let check_sprite = |command: &RenderCommand| -> (String, RenderRect, f32, f32) {
            match command {
                RenderCommand::DrawSprite {
                    symbol,
                    rect,
                    rotation,
                    layer,
                    ..
                } => (symbol.clone(), *rect, *rotation, *layer),
                other => panic!("expected DrawSprite, got {other:?}"),
            }
        };

        let (symbol, rect, rotation, layer) = check_sprite(&passes[0].commands[0]);
        assert_eq!(symbol, "router");
        assert_eq!(rect, RenderRect::new(16.0, 24.0, 8.0, 8.0));
        assert_eq!(rotation, 0.0);
        assert_eq!(layer, BlockDrawStage::TileBase.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[1].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_SHADOW);
        assert_eq!(rect, RenderRect::new(16.0, 24.0, 8.0, 8.0));
        assert_eq!(rotation, 0.0);
        assert_eq!(layer, BlockDrawStage::TileShadow.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[2].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_LIGHT);
        assert_eq!(rect, RenderRect::new(16.0, 24.0, 8.0, 8.0));
        assert_eq!(rotation, 0.0);
        assert_eq!(layer, BlockDrawStage::Light.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[3].commands[0]);
        assert_eq!(symbol, "duo");
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingBase.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[4].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_CRACKS);
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingCracks.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[5].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_TEAM);
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingTeamOverlay.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[6].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_STATUS);
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingStatus.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[7].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_LIGHT);
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::Light.layer());
    }
}
