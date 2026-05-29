//! BlockRenderer 的纯数据化状态/计划层镜像。
//!
//! 这里不持有任何 GPU 资源，只保留 upstream `BlockRenderer` 在
//! 缓存、绘制顺序、计划预览、暗度与覆盖层上的可序列化数据。

use std::collections::{BTreeMap, BTreeSet};

use crate::mindustry::{
    ctype::ContentId,
    entities::comp::DecalColor,
    graphics::{
        particle_renderer::{BlockDrawerParticlePlan, ParticleColor, ParticleRendererState},
        CacheLayer, Layer, RenderBlendMode, RenderCommand, RenderPass, RenderPassKind, RenderPoint,
        RenderProperty, RenderRect, RenderResolveKind, RenderTarget,
    },
    world::draw::{DrawBlockParticleBlendMode, DrawBlockParticleRenderKind},
    world::point2_pack,
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
    /// 当前世界对该 tile 采样出的 darkness 值；有值时 dirty darkness 可直接生成
    /// Java `drawDarkness()` 等价的灰度 FBO 写入命令，无值时保留脏 tile 标记交给后端/后续 runtime 回填。
    pub darkness: Option<f32>,
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
            darkness: None,
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
    pub block_build_regions: Vec<String>,
    pub drawer: String,
    pub build_id_seed: i32,
    pub cache_layer: CacheLayer,
    pub size: u8,
    pub rotation: i16,
    pub team: u8,
    pub visible: bool,
    pub was_visible: bool,
    pub damaged: bool,
    pub health_fraction: f32,
    pub draw_cracks: bool,
    pub draw_team_overlay: bool,
    pub draw_status: bool,
    pub emits_light: bool,
    pub visual_runtime: Option<BlockRendererBuildingVisualRuntimeSnapshot>,
}

impl Default for BuildingDrawPlan {
    fn default() -> Self {
        Self {
            coord: TileCoord::default(),
            block: String::new(),
            block_build_regions: Vec::new(),
            drawer: String::new(),
            build_id_seed: point2_pack(0, 0),
            cache_layer: CacheLayer::None,
            size: 1,
            rotation: 0,
            team: 0,
            visible: false,
            was_visible: false,
            damaged: false,
            health_fraction: 1.0,
            draw_cracks: true,
            draw_team_overlay: false,
            draw_status: false,
            emits_light: false,
            visual_runtime: None,
        }
    }
}

impl BuildingDrawPlan {
    pub fn should_draw_cracks(&self, atlas: &CrackAtlasPlan) -> bool {
        self.draw_cracks && self.damaged && atlas.supports_size(self.size)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererBuildingSnapshot {
    pub coord: TileCoord,
    pub block: String,
    pub block_build_regions: Vec<String>,
    pub drawer: String,
    pub build_id_seed: i32,
    pub cache_layer: CacheLayer,
    pub size: u8,
    pub rotation: i16,
    pub team: u8,
    pub visible: bool,
    pub was_visible: bool,
    pub damaged: bool,
    pub health_fraction: f32,
    pub draw_cracks: bool,
    pub draw_team_overlay: bool,
    pub draw_status: bool,
    pub emits_light: bool,
    pub visual_runtime: Option<BlockRendererBuildingVisualRuntimeSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockRendererBuildingVisualRuntimeLiquidSnapshot {
    pub current: Option<ContentId>,
    pub amount: Option<f32>,
    pub capacity: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockRendererBuildingVisualRuntimePowerSnapshot {
    pub status: Option<f32>,
    pub production_efficiency: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockRendererBuildingVisualRuntimeTurretSnapshot {
    pub rotation: Option<f32>,
    pub recoil: Option<f32>,
    pub heat: Option<f32>,
    pub charge: Option<f32>,
    pub side_heat: Option<[f32; 4]>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockRendererBuildingVisualRuntimeSnapshot {
    pub liquid: Option<BlockRendererBuildingVisualRuntimeLiquidSnapshot>,
    pub progress: Option<f32>,
    pub heat: Option<f32>,
    pub warmup: Option<f32>,
    pub total_progress: Option<f32>,
    pub charge: Option<f32>,
    pub power: Option<BlockRendererBuildingVisualRuntimePowerSnapshot>,
    pub turret: Option<BlockRendererBuildingVisualRuntimeTurretSnapshot>,
}

impl BlockRendererBuildingSnapshot {
    pub fn new(coord: TileCoord, block: impl Into<String>) -> Self {
        Self {
            coord,
            block: block.into(),
            block_build_regions: Vec::new(),
            drawer: String::new(),
            build_id_seed: point2_pack(coord.x, coord.y),
            cache_layer: CacheLayer::None,
            size: 1,
            rotation: 0,
            team: 0,
            visible: false,
            was_visible: false,
            damaged: false,
            health_fraction: 1.0,
            draw_cracks: true,
            draw_team_overlay: false,
            draw_status: false,
            emits_light: false,
            visual_runtime: None,
        }
    }

    pub fn should_draw_base(&self) -> bool {
        self.visible || self.was_visible
    }

    pub fn with_visual_runtime(
        mut self,
        visual_runtime: BlockRendererBuildingVisualRuntimeSnapshot,
    ) -> Self {
        self.visual_runtime = Some(visual_runtime);
        self
    }

    pub fn to_draw_plan(&self) -> BuildingDrawPlan {
        BuildingDrawPlan {
            coord: self.coord,
            block: self.block.clone(),
            block_build_regions: self.block_build_regions.clone(),
            drawer: self.drawer.clone(),
            build_id_seed: self.build_id_seed,
            cache_layer: self.cache_layer,
            size: self.size.max(1),
            rotation: self.rotation,
            team: self.team,
            visible: self.visible,
            was_visible: self.was_visible,
            damaged: self.damaged,
            health_fraction: normalize_health_fraction(self.health_fraction),
            draw_cracks: self.draw_cracks,
            draw_team_overlay: self.draw_team_overlay,
            draw_status: self.draw_status,
            emits_light: self.emits_light,
            visual_runtime: self.visual_runtime.clone(),
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

    pub fn supports_size(&self, size: u8) -> bool {
        size >= 1 && usize::from(size) <= self.max_size
    }

    pub fn region_index_for(&self, health_fraction: f32) -> usize {
        if self.regions_per_size == 0 {
            return 0;
        }

        let health_fraction = normalize_health_fraction(health_fraction);
        let damage_fraction = 1.0 - health_fraction;
        let raw = (damage_fraction * self.regions_per_size as f32) as usize;
        raw.min(self.regions_per_size - 1)
    }

    pub fn region_name(&self, size: u8, index: usize) -> Option<String> {
        if !self.supports_size(size) || self.regions_per_size == 0 {
            return None;
        }

        Some(format!(
            "cracks-{}-{}",
            size,
            index.min(self.regions_per_size - 1)
        ))
    }

    pub fn region_names(&self) -> Vec<String> {
        if self.regions_per_size == 0 || self.max_size == 0 {
            return Vec::new();
        }

        (1..=self.max_size)
            .flat_map(|size| {
                (0..self.regions_per_size).map(move |index| format!("cracks-{}-{}", size, index))
            })
            .collect()
    }

    pub fn virtual_source_paths(&self) -> Vec<String> {
        self.region_names()
            .into_iter()
            .map(|name| format!("sprites/rubble/{}.png", name))
            .collect()
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
    pub rotation: f32,
    pub tint: [f32; 4],
}

impl CrackPlan {
    pub fn new(coord: TileCoord, size: u8, region_index: usize) -> Self {
        Self {
            coord,
            size: size.clamp(1, MAX_CRACK_SIZE as u8),
            region_index: region_index % CRACK_REGION_COUNT,
            layer: Layer::BLOCK_CRACKS,
            mirrored: false,
            rotation: crack_rotation_degrees(coord),
            tint: crack_tint_from_health_fraction(1.0),
        }
    }

    pub fn from_building(building: &BuildingDrawPlan, atlas: &CrackAtlasPlan) -> Option<Self> {
        if !building.should_draw_cracks(atlas) {
            return None;
        }

        let region_index = atlas.region_index_for(building.health_fraction);
        let mut plan = Self::new(building.coord, building.size, region_index);
        plan.tint = crack_tint_from_health_fraction(building.health_fraction);
        Some(plan)
    }

    pub fn region_symbol(&self) -> String {
        format!("cracks-{}-{}", self.size, self.region_index)
    }

    pub fn append_sprite_op(&self, tile_size_world: f32, order: i32, ops: &mut Vec<BlockSpriteOp>) {
        ops.push(BlockSpriteOp::new(
            order,
            self.region_symbol(),
            building_sprite_rect(self.coord, self.size, tile_size_world),
            self.tint,
            self.rotation,
            self.layer,
        ));
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

    pub fn color(&self) -> [f32; 4] {
        [self.opacity, self.opacity, self.opacity, 1.0]
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
    pub fn clear(&mut self) {
        self.tiles.clear();
        self.dirty_tiles.clear();
        self.limited_map_area = None;
        self.fill = DarknessFill::White;
    }

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
        self.range_x = 0;
        self.range_y = 0;
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
        if self.had_map_limit {
            plan.darkness.fill = DarknessFill::Black;
            plan.darkness.limited_map_area = Some(self.cache.block_tree.bounds);
        }

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
            for coord in self.cache.dark_events.iter().copied() {
                if let Some(darkness) = snapshot_tiles.get(&coord).and_then(|tile| tile.darkness) {
                    plan.darkness.push_tile(coord, darkness);
                } else {
                    plan.darkness.push_dirty_tile(coord);
                }
            }
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
pub enum BlockSpriteRegion {
    Symbol(String),
}

impl BlockSpriteRegion {
    pub fn symbol(&self) -> &str {
        match self {
            Self::Symbol(symbol) => symbol,
        }
    }
}

impl From<&str> for BlockSpriteRegion {
    fn from(value: &str) -> Self {
        Self::Symbol(value.into())
    }
}

impl From<String> for BlockSpriteRegion {
    fn from(value: String) -> Self {
        Self::Symbol(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockSpriteOp {
    pub order: i32,
    pub region: BlockSpriteRegion,
    pub rect: RenderRect,
    pub tint: [f32; 4],
    pub rotation: f32,
    pub layer: f32,
}

impl BlockSpriteOp {
    pub fn new(
        order: i32,
        region: impl Into<BlockSpriteRegion>,
        rect: RenderRect,
        tint: [f32; 4],
        rotation: f32,
        layer: f32,
    ) -> Self {
        Self {
            order,
            region: region.into(),
            rect,
            tint,
            rotation,
            layer,
        }
    }

    pub fn symbol(&self) -> &str {
        self.region.symbol()
    }

    pub fn to_draw_sprite_command(&self) -> Option<RenderCommand> {
        let symbol = self.symbol();
        if symbol.is_empty() {
            None
        } else {
            Some(RenderCommand::draw_sprite(
                symbol,
                self.rect,
                self.tint,
                self.rotation,
                self.layer,
            ))
        }
    }
}

pub fn drawer_icons_to_block_sprite_ops(
    icons: &[String],
    rect: RenderRect,
    tint: [f32; 4],
    rotation: f32,
    layer: f32,
    order_start: i32,
) -> Vec<BlockSpriteOp> {
    icons
        .iter()
        .enumerate()
        .map(|(index, symbol)| {
            BlockSpriteOp::new(
                order_start + index as i32,
                symbol.clone(),
                rect,
                tint,
                rotation,
                layer,
            )
        })
        .collect()
}

pub fn drawer_to_block_sprite_ops(
    block_name: &str,
    drawer: &str,
    rect: RenderRect,
    tint: [f32; 4],
    rotation: f32,
    layer: f32,
    order_start: i32,
) -> Vec<BlockSpriteOp> {
    let icons = crate::mindustry::world::draw::draw_block_dispatch_icons(block_name, drawer);
    drawer_icons_to_block_sprite_ops(&icons, rect, tint, rotation, layer, order_start)
}

pub fn draw_block_drawer_sprite_ops(
    block_name: &str,
    drawer: &str,
    rect: RenderRect,
    tint: [f32; 4],
    rotation: f32,
    layer: f32,
    order_start: i32,
) -> Vec<BlockSpriteOp> {
    drawer_to_block_sprite_ops(block_name, drawer, rect, tint, rotation, layer, order_start)
}

pub fn draw_block_dispatch_sprite_ops(
    block_name: &str,
    drawer: &str,
    rect: RenderRect,
    tint: [f32; 4],
    rotation: f32,
    layer: f32,
    order_start: i32,
) -> Vec<BlockSpriteOp> {
    drawer_to_block_sprite_ops(block_name, drawer, rect, tint, rotation, layer, order_start)
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockBuildPlan {
    pub coord: TileCoord,
    pub block: String,
    pub region: String,
    pub regions: Vec<String>,
    pub size: u8,
    pub rotation: i16,
    pub progress: f32,
    pub time: f32,
    pub alpha: f32,
}

impl BlockBuildPlan {
    pub fn from_building(building: &BuildingDrawPlan) -> Option<Self> {
        let visual_runtime = building.visual_runtime.as_ref()?;
        let progress = visual_runtime.progress?;
        if building.block.is_empty() {
            return None;
        }
        let regions = normalize_block_build_regions(&building.block_build_regions, &building.block);
        let region = regions
            .first()
            .cloned()
            .unwrap_or_else(|| building.block.clone());

        Some(Self {
            coord: building.coord,
            block: building.block.clone(),
            region,
            regions,
            size: building.size.max(1),
            rotation: building.rotation,
            progress: normalize_unit_fraction(progress),
            time: finite_or_zero(visual_runtime.total_progress.unwrap_or(0.0)),
            alpha: visual_runtime.warmup.map_or(1.0, normalize_unit_fraction),
        })
    }

    pub fn rect(&self, tile_size_world: f32) -> Option<RenderRect> {
        if tile_size_world <= 0.0 {
            None
        } else {
            Some(building_sprite_rect(
                self.coord,
                self.size.max(1),
                tile_size_world,
            ))
        }
    }

    pub fn shader_command(&self, tile_size_world: f32) -> Option<RenderCommand> {
        self.shader_command_with_time(tile_size_world, self.time)
    }

    pub fn shader_command_with_time(
        &self,
        tile_size_world: f32,
        time: f32,
    ) -> Option<RenderCommand> {
        self.shader_command_with_region_and_time(tile_size_world, &self.region, time)
    }

    pub fn shader_command_with_region_and_time(
        &self,
        tile_size_world: f32,
        region: &str,
        time: f32,
    ) -> Option<RenderCommand> {
        let rect = self.rect(tile_size_world)?;
        Some(RenderCommand::custom(
            "blockbuild-shader",
            vec![
                RenderProperty::new("shader", "blockbuild"),
                RenderProperty::new("source", "BuildingDrawPlan.visual_runtime"),
                RenderProperty::new("block", self.block.clone()),
                RenderProperty::new("region", region.to_string()),
                RenderProperty::new("tile_x", self.coord.x.to_string()),
                RenderProperty::new("tile_y", self.coord.y.to_string()),
                RenderProperty::new("x", rect.x.to_string()),
                RenderProperty::new("y", rect.y.to_string()),
                RenderProperty::new("width", rect.width.to_string()),
                RenderProperty::new("height", rect.height.to_string()),
                RenderProperty::new(
                    "rotation",
                    building_rotation_degrees(self.rotation).to_string(),
                ),
                RenderProperty::new("layer", Layer::BLOCK_BUILDING.to_string()),
                RenderProperty::new("u_progress", self.progress.to_string()),
                RenderProperty::new("u_time", finite_or_zero(time).to_string()),
                RenderProperty::new("u_alpha", self.alpha.to_string()),
            ],
        ))
    }

    pub fn draw_command(&self, tile_size_world: f32) -> Option<RenderCommand> {
        self.draw_command_with_region(tile_size_world, &self.region)
    }

    pub fn draw_command_with_region(
        &self,
        tile_size_world: f32,
        region: &str,
    ) -> Option<RenderCommand> {
        Some(RenderCommand::draw_sprite(
            region.to_string(),
            self.rect(tile_size_world)?,
            SPRITE_TINT_WHITE,
            building_rotation_degrees(self.rotation),
            Layer::BLOCK_BUILDING,
        ))
    }

    pub fn render_commands(&self, tile_size_world: f32) -> Vec<RenderCommand> {
        self.render_commands_with_time(tile_size_world, self.time)
    }

    pub fn render_commands_with_time(&self, tile_size_world: f32, time: f32) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        for region in normalize_block_build_regions(&self.regions, &self.region) {
            if let Some(command) =
                self.shader_command_with_region_and_time(tile_size_world, &region, time)
            {
                commands.push(command);
            }
            if let Some(command) = self.draw_command_with_region(tile_size_world, &region) {
                commands.push(command);
            }
        }
        commands
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererBlockParticlePlan {
    pub coord: TileCoord,
    pub block: String,
    pub size: u8,
    pub plan: BlockDrawerParticlePlan,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockRendererBlockParticleWorldSample {
    pub coord: TileCoord,
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub alpha: f32,
    pub layer: f32,
    pub color: ParticleColor,
    pub secondary_color: Option<ParticleColor>,
    pub color_t: Option<f32>,
    pub render_kind: DrawBlockParticleRenderKind,
    pub blend_mode: DrawBlockParticleBlendMode,
    pub sides: usize,
    pub rotation: f32,
    pub region: Option<&'static str>,
}

impl BlockRendererBlockParticleWorldSample {
    pub fn render_blend_mode(&self) -> RenderBlendMode {
        match self.blend_mode {
            DrawBlockParticleBlendMode::Normal => RenderBlendMode::Normal,
            DrawBlockParticleBlendMode::Additive => RenderBlendMode::Additive,
        }
    }

    pub fn tint(&self) -> [f32; 4] {
        let base = [self.color.r, self.color.g, self.color.b, self.color.a];
        let color = if let (Some(secondary), Some(t)) = (self.secondary_color, self.color_t) {
            let t = t.clamp(0.0, 1.0);
            [
                base[0] + (secondary.r - base[0]) * t,
                base[1] + (secondary.g - base[1]) * t,
                base[2] + (secondary.b - base[2]) * t,
                base[3] + (secondary.a - base[3]) * t,
            ]
        } else {
            base
        };

        [color[0], color[1], color[2], color[3] * self.alpha]
    }

    pub fn to_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = vec![RenderCommand::set_blend(self.render_blend_mode())];
        let center = RenderPoint::new(self.x, self.y);

        match self.render_kind {
            DrawBlockParticleRenderKind::Circle => {
                commands.push(RenderCommand::draw_circle(
                    center,
                    self.size.max(0.0),
                    self.tint(),
                    true,
                    self.layer,
                ));
            }
            DrawBlockParticleRenderKind::SoftSprite => {
                let symbol = self.region.unwrap_or("circle-shadow");
                commands.push(RenderCommand::draw_sprite(
                    symbol,
                    RenderRect::from_center(center, self.size, self.size),
                    self.tint(),
                    0.0,
                    self.layer,
                ));
            }
            DrawBlockParticleRenderKind::Polygon => {
                commands.push(RenderCommand::draw_polygon(
                    center,
                    self.size.max(0.0),
                    self.sides,
                    self.rotation,
                    self.tint(),
                    true,
                    self.layer,
                ));
            }
        }

        commands
    }
}

impl BlockRendererBlockParticlePlan {
    pub fn world_samples(
        &self,
        tile_size_world: f32,
    ) -> Vec<BlockRendererBlockParticleWorldSample> {
        if tile_size_world <= 0.0 {
            return Vec::new();
        }

        let center = building_sprite_rect(self.coord, self.size, tile_size_world).center();
        (0..self.plan.particle_count)
            .filter_map(|index| {
                let sample = self.plan.sample_for_index(index)?;
                let radians = sample.angle.to_radians();
                Some(BlockRendererBlockParticleWorldSample {
                    coord: self.coord,
                    index,
                    x: center.x + self.plan.x + radians.cos() * sample.length,
                    y: center.y + self.plan.y + radians.sin() * sample.length,
                    size: sample.size,
                    alpha: sample.alpha,
                    layer: self.plan.layer,
                    color: self.plan.color,
                    secondary_color: self.plan.secondary_color,
                    color_t: sample.color_t,
                    render_kind: self.plan.render_kind,
                    blend_mode: self.plan.blend_mode,
                    sides: self.plan.sides,
                    rotation: self.plan.particle_rotation,
                    region: self.plan.region,
                })
            })
            .collect()
    }

    pub fn render_commands(&self, tile_size_world: f32) -> Vec<RenderCommand> {
        self.world_samples(tile_size_world)
            .iter()
            .flat_map(BlockRendererBlockParticleWorldSample::to_render_commands)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockRendererPlan {
    pub sprite_ops: Vec<BlockSpriteOp>,
    pub tile_passes: Vec<TilePassPlan>,
    pub building_passes: Vec<BuildingPassPlan>,
    pub block_builds: Vec<BlockBuildPlan>,
    pub block_particles: Vec<BlockRendererBlockParticlePlan>,
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
        self.sprite_ops.clear();
        self.tile_passes.clear();
        self.building_passes.clear();
        self.block_builds.clear();
        self.block_particles.clear();
        self.cracks.clear();
        self.build_previews.clear();
        self.darkness.clear();
        self.overlays.clear();
        self.update_floors.clear();
        self.draw_quadtree_debug = false;
        self.broken_fade = 0.0;
    }

    pub fn is_empty(&self) -> bool {
        self.sprite_ops.is_empty()
            && self.tile_passes.is_empty()
            && self.building_passes.is_empty()
            && self.block_builds.is_empty()
            && self.block_particles.is_empty()
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
        let ops = self.to_block_sprite_ops(tile_size_world);
        let mut passes: Vec<RenderPass> = Vec::new();

        for op in ops {
            let Some(command) = op.to_draw_sprite_command() else {
                continue;
            };

            match passes.last_mut() {
                Some(pass) if pass.order == op.order => pass.push(command),
                _ => {
                    let mut pass = RenderPass::new(RenderPassKind::Block).with_order(op.order);
                    pass.push(command);
                    passes.push(pass);
                }
            }
        }

        passes
    }

    pub fn to_block_build_render_commands(&self, tile_size_world: f32) -> Vec<RenderCommand> {
        self.to_block_build_render_commands_with_time(tile_size_world, None)
    }

    pub fn to_block_build_render_commands_with_time(
        &self,
        tile_size_world: f32,
        frame_time: Option<f32>,
    ) -> Vec<RenderCommand> {
        self.block_builds
            .iter()
            .flat_map(|build| match frame_time {
                Some(time) => build.render_commands_with_time(tile_size_world, time),
                None => build.render_commands(tile_size_world),
            })
            .collect()
    }

    pub fn to_block_build_render_pass(&self, tile_size_world: f32) -> Option<RenderPass> {
        self.to_block_build_render_pass_with_time(tile_size_world, None)
    }

    pub fn to_block_build_render_pass_with_time(
        &self,
        tile_size_world: f32,
        frame_time: Option<f32>,
    ) -> Option<RenderPass> {
        let commands = self.to_block_build_render_commands_with_time(tile_size_world, frame_time);
        if commands.is_empty() {
            return None;
        }

        let mut pass = RenderPass::new(RenderPassKind::BlockBuild);
        pass.extend(commands);
        Some(pass)
    }

    pub fn to_resolve_render_passes(&self, tile_size_world: f32) -> Vec<RenderPass> {
        let mut passes = Vec::new();
        if let Some(pass) = self.to_shadow_resolve_pass(tile_size_world) {
            passes.push(pass);
        }
        if let Some(pass) = self.to_darkness_resolve_pass(tile_size_world) {
            passes.push(pass);
        }
        passes
    }

    pub fn to_block_particle_render_commands(&self, tile_size_world: f32) -> Vec<RenderCommand> {
        self.block_particles
            .iter()
            .flat_map(|particle| particle.render_commands(tile_size_world))
            .collect()
    }

    pub fn to_block_particle_render_pass(&self, tile_size_world: f32) -> Option<RenderPass> {
        let commands = self.to_block_particle_render_commands(tile_size_world);
        if commands.is_empty() {
            return None;
        }

        let mut pass = RenderPass::new(RenderPassKind::Block)
            .with_order(RenderPassKind::Block.default_order() + 1);
        pass.extend(commands);
        Some(pass)
    }

    pub fn to_shadow_resolve_pass(&self, tile_size_world: f32) -> Option<RenderPass> {
        let shadow_ops = self.to_shadow_sprite_ops(tile_size_world);
        if shadow_ops.is_empty() {
            return None;
        }

        let mut pass = RenderPass::new(RenderPassKind::BlockShadows)
            .with_target(RenderTarget::Buffer("block-shadows".into()))
            .with_resolve(RenderTarget::Screen, RenderResolveKind::DrawRectSample);
        for op in shadow_ops {
            if let Some(command) = op.to_draw_sprite_command() {
                pass.push(command);
            }
        }
        Some(pass)
    }

    pub fn to_darkness_resolve_pass(&self, tile_size_world: f32) -> Option<RenderPass> {
        let commands = self.to_darkness_render_commands(tile_size_world);
        if commands.is_empty() {
            return None;
        }

        let mut pass = RenderPass::new(RenderPassKind::Darkness)
            .with_target(RenderTarget::Buffer("block-darkness".into()))
            .with_resolve(RenderTarget::Screen, RenderResolveKind::DrawFboSample);
        pass.extend(commands);
        Some(pass)
    }

    pub fn to_darkness_render_commands(&self, tile_size_world: f32) -> Vec<RenderCommand> {
        if tile_size_world <= 0.0 {
            return Vec::new();
        }

        let has_darkness_work = self.darkness.effective_fill() == DarknessFill::Black
            || !self.darkness.tiles.is_empty()
            || !self.darkness.dirty_tiles.is_empty();
        if !has_darkness_work {
            return Vec::new();
        }

        let mut commands = Vec::new();

        if self.darkness.effective_fill() == DarknessFill::Black {
            commands.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        }

        for tile in &self.darkness.tiles {
            commands.push(RenderCommand::fill_rect(
                RenderRect::new(
                    tile.coord.x as f32 * tile_size_world,
                    tile.coord.y as f32 * tile_size_world,
                    tile_size_world,
                    tile_size_world,
                ),
                tile.color(),
                tile.layer,
            ));
        }

        for coord in &self.darkness.dirty_tiles {
            commands.push(RenderCommand::custom(
                "darkness-dirty-tile",
                vec![
                    RenderProperty::new("x", coord.x.to_string()),
                    RenderProperty::new("y", coord.y.to_string()),
                ],
            ));
        }

        commands
    }

    pub fn to_block_sprite_ops(&self, tile_size_world: f32) -> Vec<BlockSpriteOp> {
        if tile_size_world <= 0.0 {
            return Vec::new();
        }

        let mut ops = self.sprite_ops.clone();
        let mut order = RenderPassKind::Block.default_order();

        for pass in &self.tile_passes {
            if pass.stage == BlockDrawStage::TileShadow {
                continue;
            }
            if pass.append_sprite_ops(tile_size_world, order, &mut ops) {
                order += 1;
            }
        }

        for pass in &self.building_passes {
            if pass.stage == BlockDrawStage::BuildingCracks {
                if self.append_crack_sprite_ops(tile_size_world, order, &mut ops) {
                    order += 1;
                }
            } else if pass.append_sprite_ops(tile_size_world, order, &mut ops) {
                order += 1;
            }
        }

        ops.sort_by_key(|op| op.order);
        ops
    }

    pub fn to_shadow_sprite_ops(&self, tile_size_world: f32) -> Vec<BlockSpriteOp> {
        if tile_size_world <= 0.0 {
            return Vec::new();
        }

        let mut ops = Vec::new();
        let mut order = RenderPassKind::BlockShadows.default_order();
        for pass in &self.tile_passes {
            if pass.stage == BlockDrawStage::TileShadow
                && pass.append_sprite_ops(tile_size_world, order, &mut ops)
            {
                order += 1;
            }
        }
        ops.sort_by_key(|op| op.order);
        ops
    }

    pub fn append_crack_sprite_ops(
        &self,
        tile_size_world: f32,
        order: i32,
        ops: &mut Vec<BlockSpriteOp>,
    ) -> bool {
        if self.cracks.is_empty() {
            return false;
        }

        for crack in &self.cracks {
            crack.append_sprite_op(tile_size_world, order, ops);
        }

        true
    }
}

impl Default for BlockRendererPlan {
    fn default() -> Self {
        Self {
            sprite_ops: Vec::new(),
            tile_passes: Vec::new(),
            building_passes: Vec::new(),
            block_builds: Vec::new(),
            block_particles: Vec::new(),
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
    append_block_build_plans(plan, &buildings);
    append_block_particle_plans(plan, &buildings);
    push_building_pass(
        plan,
        BlockDrawStage::BuildingCracks,
        buildings
            .iter()
            .filter(|building| building.should_draw_cracks(&state.crack_atlas))
            .cloned(),
    );
    plan.cracks.extend(
        buildings
            .iter()
            .filter_map(|building| CrackPlan::from_building(building, &state.crack_atlas)),
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

fn append_block_particle_plans(plan: &mut BlockRendererPlan, buildings: &[BuildingDrawPlan]) {
    for building in buildings {
        if building.drawer.is_empty() {
            continue;
        }

        let visual_runtime = building.visual_runtime.as_ref();
        let warmup = visual_runtime
            .and_then(|runtime| runtime.warmup)
            .unwrap_or(0.0);
        let time = visual_runtime
            .and_then(|runtime| runtime.total_progress)
            .unwrap_or(0.0);

        plan.block_particles.extend(
            ParticleRendererState::block_drawer_particle_plans_from_drawer(
                &building.block,
                &building.drawer,
                building.build_id_seed,
                warmup,
                time,
                Layer::BLOCK,
            )
            .into_iter()
            .filter(|particle_plan| !particle_plan.is_noop())
            .map(|particle_plan| BlockRendererBlockParticlePlan {
                coord: building.coord,
                block: building.block.clone(),
                size: building.size,
                plan: particle_plan,
            }),
        );
    }
}

fn append_block_build_plans(plan: &mut BlockRendererPlan, buildings: &[BuildingDrawPlan]) {
    plan.block_builds
        .extend(buildings.iter().filter_map(BlockBuildPlan::from_building));
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
const SPRITE_SYMBOL_BLOCK_TEAM: &str = "block-team";
const SPRITE_SYMBOL_BLOCK_STATUS: &str = "block-status";
const SPRITE_SYMBOL_BLOCK_LIGHT: &str = "block-light";

impl TilePassPlan {
    pub fn append_sprite_ops(
        &self,
        tile_size_world: f32,
        order: i32,
        ops: &mut Vec<BlockSpriteOp>,
    ) -> bool {
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

            ops.push(BlockSpriteOp::new(
                order,
                symbol,
                tile_sprite_rect(tile.coord, tile_size_world),
                SPRITE_TINT_WHITE,
                0.0,
                self.layer,
            ));
            has_commands = true;
        }

        has_commands
    }
}

impl BuildingPassPlan {
    pub fn append_sprite_ops(
        &self,
        tile_size_world: f32,
        order: i32,
        ops: &mut Vec<BlockSpriteOp>,
    ) -> bool {
        let mut has_commands = false;

        for building in &self.buildings {
            let symbol = match self.stage {
                BlockDrawStage::BuildingBase => building.block.as_str(),
                BlockDrawStage::BuildingCracks => "",
                BlockDrawStage::BuildingTeamOverlay => SPRITE_SYMBOL_BLOCK_TEAM,
                BlockDrawStage::BuildingStatus => SPRITE_SYMBOL_BLOCK_STATUS,
                BlockDrawStage::Light => SPRITE_SYMBOL_BLOCK_LIGHT,
                _ => "",
            };

            if symbol.is_empty() {
                continue;
            }

            ops.push(BlockSpriteOp::new(
                order,
                symbol,
                building_sprite_rect(building.coord, building.size, tile_size_world),
                SPRITE_TINT_WHITE,
                building_rotation_degrees(building.rotation),
                self.layer,
            ));
            has_commands = true;
        }

        has_commands
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

fn crack_rotation_degrees(coord: TileCoord) -> f32 {
    point2_pack(coord.x, coord.y).rem_euclid(4) as f32 * 90.0
}

fn normalize_health_fraction(health_fraction: f32) -> f32 {
    if health_fraction.is_finite() {
        health_fraction.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn normalize_unit_fraction(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn normalize_block_build_regions(regions: &[String], fallback: &str) -> Vec<String> {
    let mut resolved = regions
        .iter()
        .filter(|region| !region.is_empty())
        .cloned()
        .collect::<Vec<_>>();

    if resolved.is_empty() && !fallback.is_empty() {
        resolved.push(fallback.to_string());
    }

    resolved
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn crack_tint_from_health_fraction(health_fraction: f32) -> [f32; 4] {
    let damage_fraction = 1.0 - normalize_health_fraction(health_fraction);
    [0.2, 0.2, 0.2, 0.1 + damage_fraction * 0.6]
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
        assert_eq!(crack.rotation, 0.0);
        assert_eq!(crack.tint, [0.2, 0.2, 0.2, 0.1]);
        assert_eq!(crack.region_symbol(), "cracks-7-1");

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
    fn crack_atlas_region_names_match_java_load_keys() {
        let atlas = CrackAtlasPlan::default();

        assert!(atlas.supports_size(1));
        assert!(atlas.supports_size(MAX_CRACK_SIZE as u8));
        assert!(!atlas.supports_size(0));
        assert!(!atlas.supports_size((MAX_CRACK_SIZE + 1) as u8));
        assert_eq!(atlas.region_name(1, 0).as_deref(), Some("cracks-1-0"));
        assert_eq!(
            atlas
                .region_name(MAX_CRACK_SIZE as u8, CRACK_REGION_COUNT - 1)
                .as_deref(),
            Some("cracks-7-7")
        );
        assert_eq!(atlas.region_name(8, 0), None);

        let names = atlas.region_names();
        assert_eq!(names.len(), MAX_CRACK_SIZE * CRACK_REGION_COUNT);
        assert_eq!(names.first().map(String::as_str), Some("cracks-1-0"));
        assert_eq!(names.last().map(String::as_str), Some("cracks-7-7"));

        let paths = atlas.virtual_source_paths();
        assert_eq!(
            paths.first().map(String::as_str),
            Some("sprites/rubble/cracks-1-0.png")
        );
        assert_eq!(
            paths.last().map(String::as_str),
            Some("sprites/rubble/cracks-7-7.png")
        );
    }

    #[test]
    fn crack_atlas_region_index_clamps_by_health_fraction() {
        let atlas = CrackAtlasPlan::default();

        assert_eq!(atlas.region_index_for(1.0), 0);
        assert_eq!(atlas.region_index_for(0.875), 1);
        assert_eq!(atlas.region_index_for(0.5), 4);
        assert_eq!(atlas.region_index_for(0.0), CRACK_REGION_COUNT - 1);
        assert_eq!(atlas.region_index_for(-0.25), CRACK_REGION_COUNT - 1);
        assert_eq!(atlas.region_index_for(1.25), 0);
        assert_eq!(atlas.region_index_for(f32::NAN), 0);
    }

    #[test]
    fn darkness_to_opacity_matches_java_piecewise_curve() {
        let cases = [
            (-0.75, 1.0),
            (0.0, 1.0),
            (0.25, 0.8125),
            (0.5, 0.75),
            (1.5, 0.5),
            (3.5, 0.0),
            (4.0, 0.0),
            (8.0, 0.0),
        ];

        for (darkness, expected) in cases {
            assert!(
                (darkness_to_opacity(darkness) - expected).abs() < 0.0001,
                "darkness={darkness}"
            );
        }
    }

    #[test]
    fn build_plan_preserves_dark_events_as_dirty_tiles() {
        let mut state = BlockRendererState::reload(TileBounds::new(0, 0, 16, 16), true);
        state.cache.dark_events.insert(TileCoord::new(5, 5));
        state.cache.dark_events.insert(TileCoord::new(1, 2));

        let plan = state.build_plan();

        assert_eq!(
            plan.darkness.dirty_tiles,
            vec![TileCoord::new(1, 2), TileCoord::new(5, 5)]
        );
        assert_eq!(plan.darkness.fill, DarknessFill::Black);
        assert_eq!(
            plan.darkness.limited_map_area,
            Some(TileBounds::new(0, 0, 16, 16))
        );
        assert_eq!(plan.darkness.effective_fill(), DarknessFill::Black);
    }

    #[test]
    fn build_plan_turns_sampled_dirty_darkness_into_tile_fill_plan() {
        let mut state = BlockRendererState::default();
        state.cache.dark_events.insert(TileCoord::new(2, 3));
        state.cache.dark_events.insert(TileCoord::new(4, 5));

        let mut sampled = BlockRendererTileSnapshot::new(TileCoord::new(2, 3), "copper-wall");
        sampled.darkness = Some(1.5);
        let snapshot = BlockRendererWorldSnapshot::new(vec![sampled]);

        let plan = state.build_plan_from_snapshot(&snapshot);

        assert_eq!(
            plan.darkness.tiles,
            vec![DarknessTilePlan::new(TileCoord::new(2, 3), 1.5)]
        );
        assert_eq!(plan.darkness.dirty_tiles, vec![TileCoord::new(4, 5)]);

        let commands = plan.to_darkness_render_commands(8.0);
        assert!(matches!(
            commands.first(),
            Some(RenderCommand::FillRect { rect, color, layer })
                if *rect == RenderRect::new(16.0, 24.0, 8.0, 8.0)
                    && *color == [0.5, 0.5, 0.5, 1.0]
                    && *layer == Layer::DARKNESS
        ));
        assert!(matches!(
            commands.get(1),
            Some(RenderCommand::Custom { name, properties })
                if name == "darkness-dirty-tile"
                    && properties
                        == &vec![
                            RenderProperty::new("x", "4"),
                            RenderProperty::new("y", "5"),
                        ]
        ));
    }

    #[test]
    fn darkness_plan_clear_resets_fill_tiles_and_dirty_tiles() {
        let mut plan = DarknessPlan::default();
        plan.fill = DarknessFill::Black;
        plan.limited_map_area = Some(TileBounds::new(2, 3, 4, 5));
        plan.push_tile(TileCoord::new(1, 1), 0.75);
        plan.push_dirty_tile(TileCoord::new(7, 8));

        assert_eq!(plan.effective_fill(), DarknessFill::Black);
        assert!(!plan.tiles.is_empty());
        assert!(!plan.dirty_tiles.is_empty());

        plan.clear();

        assert!(plan.tiles.is_empty());
        assert!(plan.dirty_tiles.is_empty());
        assert!(plan.limited_map_area.is_none());
        assert_eq!(plan.effective_fill(), DarknessFill::White);
    }

    #[test]
    fn clear_frame_queues_clears_dark_events_without_touching_plan_tiles() {
        let mut cache = BlockRendererCache::default();
        cache.block_tree.bounds = TileBounds::new(1, 2, 3, 4);
        cache.block_light_tree.bounds = TileBounds::new(5, 6, 7, 8);
        cache.overlay_tree.bounds = TileBounds::new(9, 10, 11, 12);
        cache.floor_tree.bounds = TileBounds::new(13, 14, 15, 16);
        cache.tile_view.push(TileCoord::new(1, 1));
        cache.light_view.push(TileCoord::new(2, 2));
        cache.update_floors.push(TileCoord::new(3, 3));
        cache.shadow_events.insert(TileCoord::new(4, 4));
        cache.dark_events.insert(TileCoord::new(5, 5));
        cache.proc_links.insert(6);
        cache.proc_lights.insert(7);

        cache.clear_frame_queues();

        assert!(cache.tile_view.is_empty());
        assert!(cache.light_view.is_empty());
        assert!(cache.update_floors.is_empty());
        assert!(cache.shadow_events.is_empty());
        assert!(cache.dark_events.is_empty());
        assert!(cache.proc_links.is_empty());
        assert!(cache.proc_lights.is_empty());
        assert_eq!(cache.block_tree.bounds, TileBounds::new(1, 2, 3, 4));
        assert_eq!(cache.block_light_tree.bounds, TileBounds::new(5, 6, 7, 8));
        assert_eq!(cache.overlay_tree.bounds, TileBounds::new(9, 10, 11, 12));
        assert_eq!(cache.floor_tree.bounds, TileBounds::new(13, 14, 15, 16));
    }

    #[test]
    fn reload_resets_camera_and_darkness_queues() {
        let mut state = BlockRendererState::default();
        state.last_camera = CameraCache::new(9, 8, 7, 6);
        state.cache.dark_events.insert(TileCoord::new(4, 4));
        state.cache.tile_view.push(TileCoord::new(5, 5));
        state.cache.light_view.push(TileCoord::new(6, 6));
        state.cache.update_floors.push(TileCoord::new(7, 7));
        state.cache.shadow_events.insert(TileCoord::new(8, 8));
        state.cache.proc_links.insert(9);
        state.cache.proc_lights.insert(10);

        state = BlockRendererState::reload(TileBounds::new(0, 0, 32, 32), true);

        assert_eq!(state.last_camera, CameraCache::default());
        assert!(state.cache.dark_events.is_empty());
        assert!(state.cache.tile_view.is_empty());
        assert!(state.cache.light_view.is_empty());
        assert!(state.cache.update_floors.is_empty());
        assert!(state.cache.shadow_events.is_empty());
        assert!(state.cache.proc_links.is_empty());
        assert!(state.cache.proc_lights.is_empty());
        assert_eq!(state.cache.block_tree.bounds, TileBounds::new(0, 0, 32, 32));
        assert!(state.had_map_limit);
    }

    #[test]
    fn crack_plan_uses_health_fraction_and_tile_position_rotation() {
        let atlas = CrackAtlasPlan::default();
        let building = BuildingDrawPlan {
            coord: TileCoord::new(10, 7),
            size: 2,
            rotation: 1,
            damaged: true,
            health_fraction: 0.25,
            draw_cracks: true,
            ..BuildingDrawPlan::default()
        };

        let crack = CrackPlan::from_building(&building, &atlas).unwrap();
        assert_eq!(crack.region_index, 6);
        assert_eq!(crack.region_symbol(), "cracks-2-6");
        assert_eq!(crack.rotation, 270.0);
        assert_ne!(crack.rotation, building_rotation_degrees(building.rotation));
        assert_eq!(crack.tint, [0.2, 0.2, 0.2, 0.55]);

        let oversized = BuildingDrawPlan {
            size: (MAX_CRACK_SIZE + 1) as u8,
            ..building.clone()
        };
        assert!(CrackPlan::from_building(&oversized, &atlas).is_none());

        let disabled = BuildingDrawPlan {
            draw_cracks: false,
            ..building
        };
        assert!(CrackPlan::from_building(&disabled, &atlas).is_none());
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
        assert!(empty_plan.block_builds.is_empty());
        assert!(empty_plan.darkness.dirty_tiles.is_empty());
        assert!(empty_plan.update_floors.is_empty());
        assert!(!empty_plan.draw_quadtree_debug);
        assert_eq!(empty_plan.broken_fade, 0.0);

        let mut plan = BlockRendererPlan::default();
        plan.sprite_ops.push(BlockSpriteOp::new(
            7,
            "custom-region",
            RenderRect::new(1.0, 2.0, 3.0, 4.0),
            [0.25, 0.5, 0.75, 0.8],
            45.0,
            Layer::BLOCK + 0.5,
        ));
        plan.tile_passes.push(TilePassPlan::default());
        plan.building_passes.push(BuildingPassPlan::default());
        plan.block_builds.push(BlockBuildPlan {
            coord: TileCoord::new(1, 2),
            block: "router".into(),
            region: "router".into(),
            regions: vec!["router".into()],
            size: 1,
            rotation: 0,
            progress: 0.5,
            time: 12.0,
            alpha: 1.0,
        });
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
        assert!(plan.sprite_ops.is_empty());
        assert!(plan.block_builds.is_empty());
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

        let damaged_building = BlockRendererBuildingSnapshot::new(TileCoord::new(4, 4), "duo")
            .with_visual_runtime(BlockRendererBuildingVisualRuntimeSnapshot {
                liquid: Some(BlockRendererBuildingVisualRuntimeLiquidSnapshot {
                    current: Some(11),
                    amount: Some(4.5),
                    capacity: Some(10.0),
                }),
                progress: Some(0.25),
                heat: Some(0.75),
                warmup: Some(0.5),
                total_progress: Some(0.9),
                charge: Some(0.6),
                power: Some(BlockRendererBuildingVisualRuntimePowerSnapshot {
                    status: Some(0.8),
                    production_efficiency: Some(0.7),
                }),
                turret: Some(BlockRendererBuildingVisualRuntimeTurretSnapshot {
                    rotation: Some(135.0),
                    recoil: Some(0.4),
                    heat: Some(0.3),
                    charge: Some(0.2),
                    side_heat: Some([0.1, 0.2, 0.3, 0.4]),
                }),
            });
        let mut damaged_building = damaged_building;
        damaged_building.cache_layer = CacheLayer::Normal;
        damaged_building.size = 2;
        damaged_building.rotation = 3;
        damaged_building.team = 7;
        damaged_building.visible = true;
        damaged_building.damaged = true;
        damaged_building.health_fraction = 0.5;
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
                darkness: None,
                building: Some(damaged_building),
            },
            BlockRendererTileSnapshot {
                coord: TileCoord::new(5, 5),
                block: "mender".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: false,
                obstructs_light: true,
                darkness: None,
                building: Some(remembered_building),
            },
            BlockRendererTileSnapshot {
                coord: TileCoord::new(6, 6),
                block: "hidden-core".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: false,
                obstructs_light: true,
                darkness: None,
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
        assert_eq!(
            duo.visual_runtime,
            Some(BlockRendererBuildingVisualRuntimeSnapshot {
                liquid: Some(BlockRendererBuildingVisualRuntimeLiquidSnapshot {
                    current: Some(11),
                    amount: Some(4.5),
                    capacity: Some(10.0),
                }),
                progress: Some(0.25),
                heat: Some(0.75),
                warmup: Some(0.5),
                total_progress: Some(0.9),
                charge: Some(0.6),
                power: Some(BlockRendererBuildingVisualRuntimePowerSnapshot {
                    status: Some(0.8),
                    production_efficiency: Some(0.7),
                }),
                turret: Some(BlockRendererBuildingVisualRuntimeTurretSnapshot {
                    rotation: Some(135.0),
                    recoil: Some(0.4),
                    heat: Some(0.3),
                    charge: Some(0.2),
                    side_heat: Some([0.1, 0.2, 0.3, 0.4]),
                }),
            })
        );
        assert_eq!(plan.block_builds.len(), 1);
        assert_eq!(plan.block_builds[0].coord, TileCoord::new(4, 4));
        assert_eq!(plan.block_builds[0].block, "duo");
        assert_eq!(plan.block_builds[0].region, "duo");
        assert_eq!(plan.block_builds[0].progress, 0.25);
        assert_eq!(plan.block_builds[0].time, 0.9);
        assert_eq!(plan.block_builds[0].alpha, 0.5);

        let remembered = &plan.building_passes[0].buildings[1];
        assert_eq!(remembered.block, "mender");
        assert!(!remembered.visible);
        assert!(remembered.was_visible);
        assert_eq!(remembered.rotation, 1);
        assert_eq!(remembered.team, 2);
        assert!(remembered.visual_runtime.is_none());

        assert_eq!(
            plan.building_passes[1].stage,
            BlockDrawStage::BuildingCracks
        );
        assert_eq!(plan.building_passes[1].buildings[0].block, "duo");
        assert_eq!(plan.cracks.len(), 1);
        assert_eq!(plan.cracks[0].coord, TileCoord::new(4, 4));
        assert_eq!(plan.cracks[0].size, 2);
        assert_eq!(plan.cracks[0].region_index, 4);
        assert_eq!(plan.cracks[0].region_symbol(), "cracks-2-4");
        assert_eq!(plan.cracks[0].rotation, 0.0);
        assert_eq!(plan.cracks[0].tint, [0.2, 0.2, 0.2, 0.4]);
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
    fn building_visual_runtime_snapshot_roundtrips_into_draw_plan_and_keeps_missing_fields_none() {
        let visual_runtime = BlockRendererBuildingVisualRuntimeSnapshot {
            liquid: Some(BlockRendererBuildingVisualRuntimeLiquidSnapshot {
                current: Some(3),
                amount: None,
                capacity: Some(12.0),
            }),
            progress: Some(0.2),
            heat: None,
            warmup: Some(0.6),
            total_progress: None,
            charge: Some(0.8),
            power: Some(BlockRendererBuildingVisualRuntimePowerSnapshot {
                status: Some(0.9),
                production_efficiency: None,
            }),
            turret: Some(BlockRendererBuildingVisualRuntimeTurretSnapshot {
                rotation: Some(45.0),
                recoil: None,
                heat: Some(0.4),
                charge: Some(0.5),
                side_heat: None,
            }),
        };

        let snapshot = BlockRendererBuildingSnapshot::new(TileCoord::new(8, 9), "reactor")
            .with_visual_runtime(visual_runtime.clone());
        let plan = snapshot.to_draw_plan();

        assert_eq!(plan.coord, TileCoord::new(8, 9));
        assert_eq!(plan.block, "reactor");
        assert_eq!(plan.visual_runtime, Some(visual_runtime));

        let missing_runtime_plan =
            BlockRendererBuildingSnapshot::new(TileCoord::new(10, 11), "router").to_draw_plan();
        assert!(missing_runtime_plan.visual_runtime.is_none());
        assert_eq!(missing_runtime_plan.coord, TileCoord::new(10, 11));
        assert_eq!(missing_runtime_plan.block, "router");
        assert!(missing_runtime_plan.draw_cracks);
    }

    #[test]
    fn block_build_plan_from_visual_runtime_emits_shader_state_and_sprite() {
        let building = BuildingDrawPlan {
            coord: TileCoord::new(8, 9),
            block: "constructor".into(),
            block_build_regions: vec![
                "constructor-bottom".into(),
                "constructor-top".into(),
                String::new(),
            ],
            drawer: String::new(),
            build_id_seed: point2_pack(8, 9),
            cache_layer: CacheLayer::Normal,
            size: 2,
            rotation: 3,
            team: 1,
            visible: true,
            was_visible: false,
            damaged: false,
            health_fraction: 1.0,
            draw_cracks: true,
            draw_team_overlay: false,
            draw_status: false,
            emits_light: false,
            visual_runtime: Some(BlockRendererBuildingVisualRuntimeSnapshot {
                progress: Some(1.25),
                warmup: Some(0.4),
                total_progress: Some(17.5),
                ..BlockRendererBuildingVisualRuntimeSnapshot::default()
            }),
        };

        let build = BlockBuildPlan::from_building(&building)
            .expect("runtime progress should create a blockbuild plan");
        assert_eq!(build.coord, TileCoord::new(8, 9));
        assert_eq!(build.block, "constructor");
        assert_eq!(build.region, "constructor-bottom");
        assert_eq!(
            build.regions,
            vec![
                String::from("constructor-bottom"),
                String::from("constructor-top")
            ]
        );
        assert_eq!(build.progress, 1.0);
        assert_eq!(build.time, 17.5);
        assert_eq!(build.alpha, 0.4);

        let commands = build.render_commands(8.0);
        assert_eq!(commands.len(), 4);
        match &commands[0] {
            RenderCommand::Custom { name, properties } => {
                assert_eq!(name, "blockbuild-shader");
                let property = |key: &str| {
                    properties
                        .iter()
                        .find(|property| property.key == key)
                        .map(|property| property.value.as_str())
                };
                assert_eq!(property("shader"), Some("blockbuild"));
                assert_eq!(property("region"), Some("constructor-bottom"));
                assert_eq!(property("u_progress"), Some("1"));
                assert_eq!(property("u_time"), Some("17.5"));
                assert_eq!(property("u_alpha"), Some("0.4"));
            }
            other => panic!("expected blockbuild shader custom command, got {other:?}"),
        }
        match &commands[1] {
            RenderCommand::DrawSprite {
                symbol,
                rect,
                rotation,
                layer,
                ..
            } => {
                assert_eq!(symbol, "constructor-bottom");
                assert_eq!(*rect, RenderRect::new(56.0, 64.0, 16.0, 16.0));
                assert_eq!(*rotation, 270.0);
                assert_eq!(*layer, Layer::BLOCK_BUILDING);
            }
            other => panic!("expected blockbuild sprite command, got {other:?}"),
        }
        match (&commands[2], &commands[3]) {
            (
                RenderCommand::Custom { properties, .. },
                RenderCommand::DrawSprite { symbol, .. },
            ) => {
                assert_eq!(
                    properties
                        .iter()
                        .find(|property| property.key == "region")
                        .map(|property| property.value.as_str()),
                    Some("constructor-top")
                );
                assert_eq!(symbol, "constructor-top");
            }
            other => panic!("expected second blockbuild shader/sprite pair, got {other:?}"),
        }
    }

    #[test]
    fn block_renderer_block_build_pass_uses_visual_runtime_shader_parameters() {
        let mut plan = BlockRendererPlan::default();
        plan.block_builds.push(BlockBuildPlan {
            coord: TileCoord::new(2, 3),
            block: "payload-constructor".into(),
            region: "payload-constructor".into(),
            regions: vec!["payload-constructor".into()],
            size: 3,
            rotation: 1,
            progress: 0.35,
            time: 42.0,
            alpha: 0.75,
        });

        let pass = plan
            .to_block_build_render_pass(8.0)
            .expect("non-empty blockbuild plan should emit a render pass");
        assert_eq!(pass.kind, RenderPassKind::BlockBuild);
        assert_eq!(pass.order, RenderPassKind::BlockBuild.default_order());
        assert_eq!(pass.commands.len(), 2);
        assert!(matches!(
            pass.commands[0],
            RenderCommand::Custom { ref name, .. } if name == "blockbuild-shader"
        ));
        match &pass.commands[0] {
            RenderCommand::Custom { properties, .. } => {
                assert_eq!(
                    properties
                        .iter()
                        .find(|property| property.key == "u_time")
                        .map(|property| property.value.as_str()),
                    Some("42")
                );
            }
            other => panic!("expected blockbuild shader custom command, got {other:?}"),
        }
        let pass_with_frame_time = plan
            .to_block_build_render_pass_with_time(8.0, Some(77.0))
            .expect("frame time override should keep the blockbuild pass");
        match &pass_with_frame_time.commands[0] {
            RenderCommand::Custom { properties, .. } => {
                assert_eq!(
                    properties
                        .iter()
                        .find(|property| property.key == "u_time")
                        .map(|property| property.value.as_str()),
                    Some("77")
                );
            }
            other => panic!("expected blockbuild shader custom command, got {other:?}"),
        }
        match &pass.commands[1] {
            RenderCommand::DrawSprite {
                symbol,
                rect,
                rotation,
                layer,
                ..
            } => {
                assert_eq!(symbol, "payload-constructor");
                assert_eq!(*rect, RenderRect::new(8.0, 16.0, 24.0, 24.0));
                assert_eq!(*rotation, 90.0);
                assert_eq!(*layer, Layer::BLOCK_BUILDING);
            }
            other => panic!("expected blockbuild sprite command, got {other:?}"),
        }
    }

    #[test]
    fn block_renderer_plan_collects_draw_particles_from_building_drawer_and_runtime_warmup() {
        let coord = TileCoord::new(2, 3);
        let mut state = BlockRendererState::default();
        state.cache.tile_view = vec![coord];

        let mut building = BlockRendererBuildingSnapshot::new(coord, "heat-press");
        building.visible = true;
        building.drawer = "DrawMulti(DrawParticles, DrawSoftParticles, DrawDefault)".into();
        building.build_id_seed = 42;
        building.visual_runtime = Some(BlockRendererBuildingVisualRuntimeSnapshot {
            warmup: Some(0.75),
            total_progress: Some(19.0),
            ..BlockRendererBuildingVisualRuntimeSnapshot::default()
        });

        let mut tile = BlockRendererTileSnapshot::new(coord, "heat-press");
        tile.building = Some(building);
        let snapshot = BlockRendererWorldSnapshot { tiles: vec![tile] };

        let plan = state.build_plan_from_snapshot(&snapshot);

        assert_eq!(plan.block_particles.len(), 2);
        assert!(plan
            .block_particles
            .iter()
            .all(|particle| particle.coord == coord && particle.block == "heat-press"));
        assert!(plan
            .block_particles
            .iter()
            .all(|particle| particle.plan.build_id_seed == 42));
        assert!(plan
            .block_particles
            .iter()
            .all(|particle| particle.plan.warmup == 0.75));
        assert!(plan
            .block_particles
            .iter()
            .all(|particle| particle.plan.time == 19.0));

        let samples = plan.block_particles[0].world_samples(8.0);
        assert_eq!(samples.len(), plan.block_particles[0].plan.particle_count);
        assert!(samples.iter().all(|sample| sample.coord == coord));
        assert!(samples.iter().all(|sample| sample.x.is_finite()
            && sample.y.is_finite()
            && sample.size > 0.0
            && sample.alpha > 0.0));
    }

    #[test]
    fn block_renderer_particle_world_samples_apply_draw_offsets_and_shape_fields() {
        let coord = TileCoord::new(2, 3);
        let mut config = crate::mindustry::world::draw::draw_particles_block_config();
        config.x = 1.5;
        config.y = -2.0;
        config.sides = 5;
        config.particle_rotation = 30.0;
        config.particle_radius = 0.0;
        config.particle_count = 1;
        config.render_kind = DrawBlockParticleRenderKind::Polygon;

        let plan = BlockRendererBlockParticlePlan {
            coord,
            block: "offset-particle-test".into(),
            size: 2,
            plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                config,
                1,
                1.0,
                0.0,
                Layer::BLOCK,
            ),
        };

        let samples = plan.world_samples(8.0);

        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].x, 17.5);
        assert_eq!(samples[0].y, 22.0);
        assert_eq!(samples[0].render_kind, DrawBlockParticleRenderKind::Polygon);
        assert_eq!(samples[0].blend_mode, DrawBlockParticleBlendMode::Normal);
        assert_eq!(samples[0].sides, 5);
        assert_eq!(samples[0].rotation, 30.0);
        assert_eq!(samples[0].region, None);
    }

    #[test]
    fn block_renderer_soft_particle_world_samples_carry_circle_shadow_region() {
        let coord = TileCoord::new(1, 1);
        let mut config = crate::mindustry::world::draw::draw_soft_particles_block_config();
        config.particle_radius = 0.0;
        config.particle_count = 1;

        let plan = BlockRendererBlockParticlePlan {
            coord,
            block: "soft-particle-test".into(),
            size: 3,
            plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                config,
                2,
                1.0,
                0.0,
                Layer::BLOCK,
            ),
        };

        let samples = plan.world_samples(8.0);

        assert_eq!(samples.len(), 1);
        assert_eq!(
            samples[0].render_kind,
            DrawBlockParticleRenderKind::SoftSprite
        );
        assert_eq!(samples[0].blend_mode, DrawBlockParticleBlendMode::Additive);
        assert_eq!(samples[0].region, Some("circle-shadow"));
    }

    #[test]
    fn block_particle_samples_enter_block_render_pass_commands() {
        let mut soft = crate::mindustry::world::draw::draw_soft_particles_block_config();
        soft.particle_radius = 0.0;
        soft.particle_count = 1;
        let mut polygon = crate::mindustry::world::draw::draw_particles_block_config();
        polygon.particle_radius = 0.0;
        polygon.particle_count = 1;
        polygon.sides = 6;
        polygon.particle_rotation = 25.0;
        polygon.render_kind = DrawBlockParticleRenderKind::Polygon;

        let mut plan = BlockRendererPlan::default();
        plan.block_particles = vec![
            BlockRendererBlockParticlePlan {
                coord: TileCoord::new(1, 1),
                block: "soft-particle-test".into(),
                size: 2,
                plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                    soft,
                    3,
                    1.0,
                    0.0,
                    Layer::BLOCK,
                ),
            },
            BlockRendererBlockParticlePlan {
                coord: TileCoord::new(2, 2),
                block: "poly-particle-test".into(),
                size: 2,
                plan: ParticleRendererState::block_drawer_particle_plan_from_draw_config(
                    polygon,
                    4,
                    1.0,
                    0.0,
                    Layer::BLOCK,
                ),
            },
        ];

        let pass = plan
            .to_block_particle_render_pass(8.0)
            .expect("particles should produce a block render pass");

        assert_eq!(pass.kind, RenderPassKind::Block);
        assert_eq!(pass.order, RenderPassKind::Block.default_order() + 1);
        assert_eq!(pass.commands.len(), 4);
        assert!(matches!(
            pass.commands.get(0),
            Some(RenderCommand::SetBlend { mode }) if *mode == RenderBlendMode::Additive
        ));
        assert!(matches!(
            pass.commands.get(1),
            Some(RenderCommand::DrawSprite { symbol, .. }) if symbol == "circle-shadow"
        ));
        assert!(matches!(
            pass.commands.get(2),
            Some(RenderCommand::SetBlend { mode }) if *mode == RenderBlendMode::Normal
        ));
        assert!(matches!(
            pass.commands.get(3),
            Some(RenderCommand::DrawPolygon { sides, rotation, filled, .. })
                if *sides == 6 && *rotation == 25.0 && *filled
        ));
    }

    #[test]
    fn explicit_block_sprite_ops_preserve_region_tint_layer_order_into_draw_sprite() {
        let mut plan = BlockRendererPlan::default();
        let op = BlockSpriteOp::new(
            42,
            BlockSpriteRegion::from("surge-wall-glow"),
            RenderRect::new(10.0, 20.0, 30.0, 40.0),
            [0.2, 0.4, 0.6, 0.8],
            37.0,
            Layer::BLOCK + 0.25,
        );
        plan.sprite_ops.push(op.clone());

        assert_eq!(op.symbol(), "surge-wall-glow");
        assert_eq!(plan.to_block_sprite_ops(8.0), vec![op.clone()]);

        let passes = plan.to_sprite_render_passes(8.0);
        assert_eq!(passes.len(), 1);
        assert_eq!(passes[0].order, 42);
        assert_eq!(passes[0].commands.len(), 1);

        match &passes[0].commands[0] {
            RenderCommand::DrawSprite {
                symbol,
                rect,
                tint,
                rotation,
                layer,
            } => {
                assert_eq!(symbol, "surge-wall-glow");
                assert_eq!(*rect, op.rect);
                assert_eq!(*tint, op.tint);
                assert_eq!(*rotation, op.rotation);
                assert_eq!(*layer, op.layer);
            }
            other => panic!("expected DrawSprite, got {other:?}"),
        }
    }

    #[test]
    fn block_renderer_plan_emits_shadow_and_darkness_resolve_passes() {
        let mut plan = BlockRendererPlan::default();
        plan.tile_passes.push(TilePassPlan {
            stage: BlockDrawStage::TileShadow,
            layer: BlockDrawStage::TileShadow.layer(),
            tiles: vec![TileDrawPlan {
                coord: TileCoord::new(2, 3),
                block: "router".into(),
                cache_layer: CacheLayer::Normal,
                draw_custom_shadow: false,
                emits_light: false,
                obstructs_light: false,
            }],
        });
        plan.darkness.fill = DarknessFill::Black;
        plan.darkness.push_tile(TileCoord::new(1, 2), 1.5);
        plan.darkness.push_dirty_tile(TileCoord::new(3, 4));

        let passes = plan.to_resolve_render_passes(8.0);

        assert_eq!(passes.len(), 2);
        assert_eq!(passes[0].kind, RenderPassKind::BlockShadows);
        assert_eq!(
            passes[0].target,
            RenderTarget::Buffer("block-shadows".into())
        );
        assert_eq!(passes[0].resolve_target, Some(RenderTarget::Screen));
        assert_eq!(
            passes[0].resolve_kind,
            Some(RenderResolveKind::DrawRectSample)
        );
        assert_eq!(passes[0].commands.len(), 1);
        assert!(matches!(
            &passes[0].commands[0],
            RenderCommand::DrawSprite { symbol, rect, layer, .. }
                if symbol == SPRITE_SYMBOL_BLOCK_SHADOW
                    && *rect == RenderRect::new(16.0, 24.0, 8.0, 8.0)
                    && *layer == BlockDrawStage::TileShadow.layer()
        ));

        assert_eq!(passes[1].kind, RenderPassKind::Darkness);
        assert_eq!(
            passes[1].target,
            RenderTarget::Buffer("block-darkness".into())
        );
        assert_eq!(passes[1].resolve_target, Some(RenderTarget::Screen));
        assert_eq!(
            passes[1].resolve_kind,
            Some(RenderResolveKind::DrawFboSample)
        );
        assert!(matches!(
            passes[1].commands.get(0),
            Some(RenderCommand::Clear { color }) if *color == [0.0, 0.0, 0.0, 1.0]
        ));
        assert!(matches!(
            passes[1].commands.get(1),
            Some(RenderCommand::FillRect { rect, color, layer })
                if *rect == RenderRect::new(8.0, 16.0, 8.0, 8.0)
                    && *color == [0.5, 0.5, 0.5, 1.0]
                    && *layer == Layer::DARKNESS
        ));
        assert!(matches!(
            passes[1].commands.get(2),
            Some(RenderCommand::Custom { name, properties })
                if name == "darkness-dirty-tile"
                    && properties
                        == &vec![
                            RenderProperty::new("x", "3"),
                            RenderProperty::new("y", "4"),
                        ]
        ));
    }

    #[test]
    fn drawer_dispatch_and_sprite_bridge_preserve_multi_order_with_glow_and_liquid_tile_noop() {
        let icons = crate::mindustry::world::draw::draw_block_drawer_icons(
            "separator",
            "DrawMulti(DrawRegion(-bottom), DrawLiquidTile, DrawGlowRegion(-glow), DrawDefault, DrawRegion(-top))",
        );
        assert_eq!(
            icons,
            vec![
                "separator-bottom",
                "separator-glow",
                "separator",
                "separator-top"
            ]
        );

        let ops = drawer_to_block_sprite_ops(
            "separator",
            "DrawMulti(DrawRegion(-bottom), DrawLiquidTile, DrawGlowRegion(-glow), DrawDefault, DrawRegion(-top))",
            RenderRect::new(1.0, 2.0, 3.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            90.0,
            Layer::BLOCK + 0.5,
            7,
        );
        assert_eq!(
            ops.iter().map(|op| op.symbol()).collect::<Vec<_>>(),
            vec![
                "separator-bottom",
                "separator-glow",
                "separator",
                "separator-top"
            ]
        );
        assert_eq!(
            ops.iter().map(|op| op.order).collect::<Vec<_>>(),
            vec![7, 8, 9, 10]
        );

        let mut plan = BlockRendererPlan::default();
        plan.sprite_ops = ops.clone();
        assert_eq!(plan.to_block_sprite_ops(8.0), ops);

        let passes = plan.to_sprite_render_passes(8.0);
        assert_eq!(passes.len(), 4);
        assert_eq!(passes[0].order, 7);
        assert_eq!(passes[1].order, 8);
        assert_eq!(passes[2].order, 9);
        assert_eq!(passes[3].order, 10);
    }

    #[test]
    fn drawer_dispatch_bridge_covers_static_heat_input_liquid_region_and_warmup_region() {
        let ops = drawer_to_block_sprite_ops(
            "reactor",
            "DrawMulti(DrawHeatInput, DrawLiquidTile, DrawGlowRegion, DrawLiquidRegion, DrawWarmupRegion)",
            RenderRect::new(1.0, 2.0, 3.0, 4.0),
            [0.7, 0.8, 0.9, 1.0],
            0.0,
            Layer::BLOCK,
            20,
        );
        assert_eq!(
            ops.iter().map(|op| op.symbol()).collect::<Vec<_>>(),
            vec![
                "reactor-heat",
                "reactor-glow",
                "reactor-liquid",
                "reactor-top"
            ]
        );
        assert_eq!(
            ops.iter().map(|op| op.order).collect::<Vec<_>>(),
            vec![20, 21, 22, 23]
        );
    }

    #[test]
    fn drawer_dispatch_bridge_covers_static_turret_power_and_heat_output() {
        let ops = drawer_to_block_sprite_ops(
            "battery",
            "DrawMulti(DrawDefault, DrawPower, DrawRegion(-top))",
            RenderRect::new(1.0, 2.0, 3.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            Layer::BLOCK,
            20,
        );
        assert_eq!(
            ops.iter().map(|op| op.symbol()).collect::<Vec<_>>(),
            vec!["battery", "battery-power", "battery-top"]
        );
        assert_eq!(
            ops.iter().map(|op| op.order).collect::<Vec<_>>(),
            vec![20, 21, 22]
        );

        let turret_ops = drawer_to_block_sprite_ops(
            "scatter",
            "DrawTurret",
            RenderRect::new(1.0, 2.0, 3.0, 4.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            Layer::BLOCK,
            30,
        );
        assert_eq!(
            turret_ops.iter().map(|op| op.symbol()).collect::<Vec<_>>(),
            vec!["scatter-base", "scatter-preview", "scatter-top"]
        );
        assert_eq!(
            turret_ops.iter().map(|op| op.order).collect::<Vec<_>>(),
            vec![30, 31, 32]
        );

        assert_eq!(
            drawer_to_block_sprite_ops(
                "heater",
                "DrawMulti(DrawHeatOutput, DrawHeatOutput(-1))",
                RenderRect::new(1.0, 2.0, 3.0, 4.0),
                [1.0, 1.0, 1.0, 1.0],
                0.0,
                Layer::BLOCK,
                40,
            )
            .iter()
            .map(|op| op.symbol())
            .collect::<Vec<_>>(),
            vec!["heater-top1", "heater-top2"]
        );
    }

    #[test]
    fn drawer_dispatch_bridge_covers_flame_heat_region_and_dynamic_shell_noops() {
        let drawer = "DrawMulti(DrawFlame(ffc999), DrawHeatRegion(-vents), DrawLiquidOutputs, DrawParticles, DrawDefault)";
        let rect = RenderRect::new(1.0, 2.0, 3.0, 4.0);
        let tint = [0.9, 0.8, 0.7, 0.6];
        let direct =
            drawer_to_block_sprite_ops("surge-crucible", drawer, rect, tint, 0.0, Layer::BLOCK, 60);
        let wrapped = draw_block_drawer_sprite_ops(
            "surge-crucible",
            drawer,
            rect,
            tint,
            0.0,
            Layer::BLOCK,
            60,
        );

        assert_eq!(direct, wrapped);
        assert_eq!(
            direct.iter().map(|op| op.symbol()).collect::<Vec<_>>(),
            vec![
                "surge-crucible-top",
                "surge-crucible-vents",
                "surge-crucible",
            ]
        );
        assert_eq!(
            direct.iter().map(|op| op.order).collect::<Vec<_>>(),
            vec![60, 61, 62]
        );
        assert!(direct.iter().all(|op| op.rect == rect));
        assert_eq!(
            crate::mindustry::world::draw::draw_block_dispatch_icons("surge-crucible", drawer),
            vec![
                "surge-crucible-top",
                "surge-crucible-vents",
                "surge-crucible",
            ]
        );
    }

    #[test]
    fn drawer_dispatch_bridge_covers_static_pistons_weave_and_side_region() {
        assert_eq!(
            crate::mindustry::world::draw::draw_block_dispatch_icons("press", "DrawPistons(-arm)"),
            vec!["press-arm-icon"]
        );
        assert_eq!(
            crate::mindustry::world::draw::draw_block_dispatch_icons("separator", "DrawSideRegion"),
            vec!["separator-top1"]
        );

        let ops = drawer_to_block_sprite_ops(
            "phase-weaver",
            "DrawMulti(DrawPistons, DrawWeave, DrawMultiWeave, DrawSideRegion)",
            RenderRect::new(2.0, 4.0, 6.0, 8.0),
            [0.5, 0.6, 0.7, 0.8],
            15.0,
            Layer::BLOCK + 0.75,
            50,
        );
        assert_eq!(
            ops.iter().map(|op| op.symbol()).collect::<Vec<_>>(),
            vec![
                "phase-weaver-piston-icon",
                "phase-weaver-weave",
                "phase-weaver-weave",
                "phase-weaver-top1",
            ]
        );
        assert_eq!(
            ops.iter().map(|op| op.order).collect::<Vec<_>>(),
            vec![50, 51, 52, 53]
        );
        assert!(ops
            .iter()
            .all(|op| op.rect == RenderRect::new(2.0, 4.0, 6.0, 8.0)));
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
            block_build_regions: vec!["duo".into()],
            drawer: String::new(),
            build_id_seed: point2_pack(4, 5),
            cache_layer: CacheLayer::Normal,
            size: 2,
            rotation: 1,
            team: 7,
            visible: true,
            was_visible: false,
            damaged: true,
            health_fraction: 0.5,
            draw_cracks: true,
            draw_team_overlay: true,
            draw_status: true,
            emits_light: true,
            visual_runtime: None,
        };
        plan.cracks
            .push(CrackPlan::from_building(&building, &CrackAtlasPlan::default()).unwrap());

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

        let shadow_ops = plan.to_shadow_sprite_ops(8.0);
        assert_eq!(shadow_ops.len(), 1);
        assert_eq!(shadow_ops[0].symbol(), SPRITE_SYMBOL_BLOCK_SHADOW);
        assert_eq!(shadow_ops[0].rect, RenderRect::new(16.0, 24.0, 8.0, 8.0));
        assert_eq!(shadow_ops[0].layer, BlockDrawStage::TileShadow.layer());

        let ops = plan.to_block_sprite_ops(8.0);
        assert_eq!(ops.len(), 7);
        assert!(ops.windows(2).all(|pair| pair[0].order <= pair[1].order));
        assert!(ops
            .iter()
            .filter(|op| !op.symbol().starts_with("cracks-"))
            .all(|op| op.tint == SPRITE_TINT_WHITE));
        assert_eq!(ops[0].symbol(), "router");
        assert_eq!(ops[0].rect, RenderRect::new(16.0, 24.0, 8.0, 8.0));
        assert_eq!(ops[0].rotation, 0.0);
        assert_eq!(ops[0].layer, BlockDrawStage::TileBase.layer());
        assert_eq!(ops[2].symbol(), "duo");
        assert_eq!(ops[2].rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(ops[2].rotation, 90.0);
        assert_eq!(ops[2].layer, BlockDrawStage::BuildingBase.layer());

        let passes = plan.to_sprite_render_passes(8.0);
        assert_eq!(passes.len(), 7);
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
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_LIGHT);
        assert_eq!(rect, RenderRect::new(16.0, 24.0, 8.0, 8.0));
        assert_eq!(rotation, 0.0);
        assert_eq!(layer, BlockDrawStage::Light.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[2].commands[0]);
        assert_eq!(symbol, "duo");
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingBase.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[3].commands[0]);
        assert_eq!(symbol, "cracks-2-4");
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingCracks.layer());
        match &passes[3].commands[0] {
            RenderCommand::DrawSprite { tint, .. } => assert_eq!(*tint, [0.2, 0.2, 0.2, 0.4]),
            other => panic!("expected DrawSprite, got {other:?}"),
        }

        let (symbol, rect, rotation, layer) = check_sprite(&passes[4].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_TEAM);
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingTeamOverlay.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[5].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_STATUS);
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::BuildingStatus.layer());

        let (symbol, rect, rotation, layer) = check_sprite(&passes[6].commands[0]);
        assert_eq!(symbol, SPRITE_SYMBOL_BLOCK_LIGHT);
        assert_eq!(rect, RenderRect::new(24.0, 32.0, 16.0, 16.0));
        assert_eq!(rotation, 90.0);
        assert_eq!(layer, BlockDrawStage::Light.layer());
    }
}
