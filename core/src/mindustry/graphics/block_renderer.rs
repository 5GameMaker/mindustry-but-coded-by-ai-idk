//! BlockRenderer 的纯数据化状态/计划层镜像。
//!
//! 这里不持有任何 GPU 资源，只保留 upstream `BlockRenderer` 在
//! 缓存、绘制顺序、计划预览、暗度与覆盖层上的可序列化数据。

use std::collections::BTreeSet;

use crate::mindustry::{
    entities::comp::DecalColor,
    graphics::{CacheLayer, Layer},
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
        let mut plan = BlockRendererPlan::default();

        plan.broken_fade = self.broken_fade;
        plan.draw_quadtree_debug = self.draw_quadtree_debug;
        plan.update_floors = self.cache.update_floors.clone();

        if let Some(pass) = build_tile_pass(
            BlockDrawStage::TileBase,
            self.cache.tile_view.iter().copied(),
        ) {
            plan.tile_passes.push(pass);
        }

        if let Some(pass) = build_tile_pass(
            BlockDrawStage::TileShadow,
            self.cache.shadow_events.iter().copied(),
        ) {
            plan.tile_passes.push(pass);
        }

        if let Some(pass) =
            build_tile_pass(BlockDrawStage::Light, self.cache.light_view.iter().copied())
        {
            plan.tile_passes.push(pass);
        }

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

fn build_tile_pass<I>(stage: BlockDrawStage, coords: I) -> Option<TilePassPlan>
where
    I: IntoIterator<Item = TileCoord>,
{
    let tiles = coords
        .into_iter()
        .map(|coord| TileDrawPlan {
            coord,
            // 这里仅承载坐标与阶段，不伪造具体 block 细节。
            ..TileDrawPlan::default()
        })
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
}
