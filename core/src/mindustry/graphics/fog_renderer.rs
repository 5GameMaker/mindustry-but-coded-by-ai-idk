//! 后端无关的 FogRenderer 数据/计划层。
//!
//! 这个模块只保留 upstream `FogRenderer` 的状态与阶段信息：
//! - fog 纹理的静态/动态失效
//! - camera / world viewport
//! - drawFog / drawLight / clear / copyFromCpu 的计划化阶段
//! - 静态 fog 事件队列
//!
//! 目标是让不同渲染后端在不依赖 GPU API 的前提下，按同一份计划执行。

use super::{
    RenderCamera, RenderCommand, RenderPass, RenderPassKind, RenderPoint, RenderProperty,
    RenderResolveKind, RenderTarget, RenderTextureSamplePlan, RenderViewport,
};

pub const DEFAULT_FOG_EVENT_PADDING: i32 = 1;
pub const DEFAULT_STATIC_FOG_DYNAMIC_ALPHA: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FogTextureKind {
    #[default]
    Static,
    Dynamic,
}

impl FogTextureKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Dynamic => "dynamic",
        }
    }

    pub fn render_target(self) -> RenderTarget {
        RenderTarget::Buffer(format!("fog:{}", self.label()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl FogColor {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn dynamic_alpha(self) -> f32 {
        if self.a.is_nan() {
            DEFAULT_STATIC_FOG_DYNAMIC_ALPHA
        } else {
            self.a.max(DEFAULT_STATIC_FOG_DYNAMIC_ALPHA)
        }
    }

    pub const fn rgba(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for FogColor {
    fn default() -> Self {
        Self::BLACK
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogViewport {
    pub world_width: i32,
    pub world_height: i32,
    pub tile_size: i32,
    pub camera_x: f32,
    pub camera_y: f32,
    pub camera_width: f32,
    pub camera_height: f32,
}

impl FogViewport {
    pub const fn new(
        world_width: i32,
        world_height: i32,
        tile_size: i32,
        camera_x: f32,
        camera_y: f32,
        camera_width: f32,
        camera_height: f32,
    ) -> Self {
        Self {
            world_width,
            world_height,
            tile_size,
            camera_x,
            camera_y,
            camera_width,
            camera_height,
        }
    }

    pub fn world_pixel_width(&self) -> i32 {
        self.world_width * self.tile_size
    }

    pub fn world_pixel_height(&self) -> i32 {
        self.world_height * self.tile_size
    }

    pub fn camera_pixel_rect(&self) -> FogRect {
        FogRect::new(
            self.camera_x.floor() as i32,
            self.camera_y.floor() as i32,
            self.camera_width.ceil() as i32,
            self.camera_height.ceil() as i32,
        )
    }

    pub fn render_camera(&self) -> RenderCamera {
        RenderCamera::new(
            RenderPoint::new(
                self.camera_x + self.camera_width / 2.0,
                self.camera_y + self.camera_height / 2.0,
            ),
            RenderViewport::new(0.0, 0.0, self.camera_width, self.camera_height),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FogRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl FogRect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl Default for FogRect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogLightSource {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl FogLightSource {
    pub const fn new(x: f32, y: f32, radius: f32) -> Self {
        Self { x, y, radius }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogEvent {
    pub x: i32,
    pub y: i32,
    pub radius: f32,
    pub center_offset: f32,
}

impl FogEvent {
    pub const fn new(x: i32, y: i32, radius: f32) -> Self {
        Self {
            x,
            y,
            radius,
            center_offset: 0.0,
        }
    }

    pub const fn with_center_offset(mut self, center_offset: f32) -> Self {
        self.center_offset = center_offset;
        self
    }

    pub fn render_center(&self) -> (f32, f32) {
        (
            self.x as f32 + 0.5 + self.center_offset,
            self.y as f32 + 0.5 + self.center_offset,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FogTileReveal {
    pub x: i32,
    pub y: i32,
}

impl FogTileReveal {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogCpuCopyPlan {
    pub world_width: i32,
    pub world_height: i32,
    pub padding: i32,
    pub revealed_tiles: Vec<FogTileReveal>,
    pub clear_color: FogColor,
}

impl FogCpuCopyPlan {
    pub fn new(world_width: i32, world_height: i32, revealed_tiles: Vec<FogTileReveal>) -> Self {
        Self {
            world_width,
            world_height,
            padding: DEFAULT_FOG_EVENT_PADDING,
            revealed_tiles,
            clear_color: FogColor::BLACK,
        }
    }

    pub fn from_discovered_map(world_width: i32, world_height: i32, discovered: &[bool]) -> Self {
        let mut revealed_tiles = Vec::new();
        let len = world_width.saturating_mul(world_height).max(0) as usize;
        let width = world_width.max(0) as usize;
        let limit = discovered.len().min(len);

        for i in 0..limit {
            if discovered[i] {
                let x = i % width;
                let y = i / width;

                // 和 upstream 一致：边界保留 1 像素 padding，避免边缘完全露出。
                if x > 0 && y > 0 && x + 1 < width && y + 1 < world_height.max(0) as usize {
                    revealed_tiles.push(FogTileReveal::new(x as i32, y as i32));
                }
            }
        }

        Self::new(world_width, world_height, revealed_tiles)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogTextureSnapshot {
    pub width: i32,
    pub height: i32,
    pub invalidated: bool,
}

impl FogTextureSnapshot {
    pub const fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            invalidated: false,
        }
    }

    pub fn resize_check(&mut self, width: i32, height: i32) -> bool {
        let resized = self.width != width || self.height != height;
        if resized {
            self.width = width;
            self.height = height;
            self.invalidated = true;
        }
        resized
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        self.invalidated = true;
    }

    pub fn mark_invalidated(&mut self) {
        self.invalidated = true;
    }

    pub fn clear_invalidation(&mut self) {
        self.invalidated = false;
    }
}

impl Default for FogTextureSnapshot {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogTexturePlan {
    pub kind: FogTextureKind,
    pub width: i32,
    pub height: i32,
    pub resized: bool,
    pub invalidated: bool,
    pub clear: bool,
    pub filter_linear: bool,
}

impl FogTexturePlan {
    pub fn new(kind: FogTextureKind, width: i32, height: i32) -> Self {
        Self {
            kind,
            width,
            height,
            resized: false,
            invalidated: false,
            clear: false,
            filter_linear: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogClearStage {
    pub kind: FogTextureKind,
    pub color: FogColor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogLightStage {
    pub kind: FogTextureKind,
    pub viewport: FogViewport,
    pub clip_padding: i32,
    pub sources: Vec<FogLightSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogEventStage {
    pub kind: FogTextureKind,
    pub viewport: FogViewport,
    pub clip_padding: i32,
    pub events: Vec<FogEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogCpuCopyStage {
    pub kind: FogTextureKind,
    pub plan: FogCpuCopyPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogCompositeStage {
    pub kind: FogTextureKind,
    pub world_width: i32,
    pub world_height: i32,
    pub tile_size: i32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub resolve_sample: Option<RenderTextureSamplePlan>,
    pub color: FogColor,
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FogDrawStage {
    Clear(FogClearStage),
    DrawLight(FogLightStage),
    DrawFog(FogEventStage),
    CopyFromCpu(FogCpuCopyStage),
    Composite(FogCompositeStage),
}

impl FogDrawStage {
    pub fn texture_kind(&self) -> FogTextureKind {
        match self {
            Self::Clear(stage) => stage.kind,
            Self::DrawLight(stage) => stage.kind,
            Self::DrawFog(stage) => stage.kind,
            Self::CopyFromCpu(stage) => stage.kind,
            Self::Composite(stage) => stage.kind,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Clear(_) => "clear",
            Self::DrawLight(_) => "draw_light",
            Self::DrawFog(_) => "draw_fog",
            Self::CopyFromCpu(_) => "copy_from_cpu",
            Self::Composite(_) => "composite",
        }
    }

    pub fn to_render_pass(&self, stage_index: usize) -> RenderPass {
        let kind = self.texture_kind();
        let mut pass = RenderPass::new(RenderPassKind::Fog)
            .with_order(RenderPassKind::Fog.default_order() + stage_index as i32)
            .with_target(kind.render_target());

        match self {
            Self::Clear(stage) => {
                pass.push(RenderCommand::clear(stage.color.rgba()));
            }
            Self::DrawLight(stage) => {
                pass.push(RenderCommand::custom(
                    "fog-draw-light",
                    vec![
                        RenderProperty::new("texture", stage.kind.label()),
                        RenderProperty::new("sources", stage.sources.len().to_string()),
                        RenderProperty::new("clip_padding", stage.clip_padding.to_string()),
                    ],
                ));
            }
            Self::DrawFog(stage) => {
                pass.push(RenderCommand::custom(
                    "fog-draw-events",
                    vec![
                        RenderProperty::new("texture", stage.kind.label()),
                        RenderProperty::new("events", stage.events.len().to_string()),
                        RenderProperty::new("clip_padding", stage.clip_padding.to_string()),
                    ],
                ));
            }
            Self::CopyFromCpu(stage) => {
                pass.push(RenderCommand::custom(
                    "fog-copy-from-cpu",
                    vec![
                        RenderProperty::new("texture", stage.kind.label()),
                        RenderProperty::new(
                            "revealed_tiles",
                            stage.plan.revealed_tiles.len().to_string(),
                        ),
                    ],
                ));
            }
            Self::Composite(stage) => {
                pass = pass.with_resolve(RenderTarget::Screen, RenderResolveKind::DrawFboSample);
                if let Some(sample) = stage.resolve_sample {
                    pass = pass.with_resolve_sample(sample);
                }
                pass.push(RenderCommand::custom(
                    "fog-composite",
                    vec![
                        RenderProperty::new("texture", stage.kind.label()),
                        RenderProperty::new("world_width", stage.world_width.to_string()),
                        RenderProperty::new("world_height", stage.world_height.to_string()),
                        RenderProperty::new("tile_size", stage.tile_size.to_string()),
                        RenderProperty::new("offset_x", stage.offset_x.to_string()),
                        RenderProperty::new("offset_y", stage.offset_y.to_string()),
                        RenderProperty::new("alpha", stage.alpha.to_string()),
                    ],
                ));
            }
        }

        pass
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogFrameInput {
    pub viewport: FogViewport,
    pub team: u32,
    pub discovered_map_available: bool,
    pub static_fog_enabled: bool,
    pub static_color: FogColor,
    pub dynamic_color: FogColor,
    pub dynamic_sources: Vec<FogLightSource>,
    pub discovered_tiles: Option<Vec<bool>>,
}

impl FogFrameInput {
    pub fn new(
        viewport: FogViewport,
        team: u32,
        discovered_map_available: bool,
        static_fog_enabled: bool,
        static_color: FogColor,
        dynamic_color: FogColor,
    ) -> Self {
        Self {
            viewport,
            team,
            discovered_map_available,
            static_fog_enabled,
            static_color,
            dynamic_color,
            dynamic_sources: Vec::new(),
            discovered_tiles: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogFramePlan {
    pub viewport: FogViewport,
    pub static_texture: FogTexturePlan,
    pub dynamic_texture: FogTexturePlan,
    pub stages: Vec<FogDrawStage>,
    pub consumed_events: Vec<FogEvent>,
    pub team_changed: bool,
    pub static_fog_enabled: bool,
}

impl FogFramePlan {
    pub fn to_render_passes(&self) -> Vec<RenderPass> {
        self.stages
            .iter()
            .enumerate()
            .map(|(index, stage)| stage.to_render_pass(index))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FogRendererState {
    pub last_team: Option<u32>,
    pub queued_events: Vec<FogEvent>,
    pub static_texture: FogTextureSnapshot,
    pub dynamic_texture: FogTextureSnapshot,
}

impl Default for FogRendererState {
    fn default() -> Self {
        Self {
            last_team: None,
            queued_events: Vec::new(),
            static_texture: FogTextureSnapshot::default(),
            dynamic_texture: FogTextureSnapshot::default(),
        }
    }
}

impl FogRendererState {
    pub fn handle_event(&mut self, event: FogEvent) {
        self.queued_events.push(event);
    }

    pub fn clear(&mut self) {
        self.last_team = None;
        self.queued_events.clear();
    }

    pub fn invalidate_static_texture(&mut self) {
        self.static_texture.mark_invalidated();
    }

    pub fn invalidate_dynamic_texture(&mut self) {
        self.dynamic_texture.mark_invalidated();
    }

    pub fn draw_fog_plan(&mut self, input: FogFrameInput) -> Option<FogFramePlan> {
        if !input.discovered_map_available {
            return None;
        }

        let viewport = input.viewport;
        let world_width = viewport.world_width;
        let world_height = viewport.world_height;
        let tile_size = viewport.tile_size;

        let static_resized = self.static_texture.resize_check(world_width, world_height);
        let dynamic_resized = self.dynamic_texture.resize_check(world_width, world_height);

        let mut stages = Vec::new();
        let mut clear_static = static_resized || self.static_texture.invalidated;
        let mut team_changed = false;
        let render_camera = viewport.render_camera();

        if input.static_fog_enabled && self.last_team != Some(input.team) {
            team_changed = true;
            let discovered = input.discovered_tiles.as_deref().unwrap_or(&[]);
            let copy_plan =
                FogCpuCopyPlan::from_discovered_map(world_width, world_height, discovered);
            stages.push(FogDrawStage::CopyFromCpu(FogCpuCopyStage {
                kind: FogTextureKind::Static,
                plan: copy_plan,
            }));
            self.last_team = Some(input.team);
            clear_static = false;
        }

        stages.push(FogDrawStage::Clear(FogClearStage {
            kind: FogTextureKind::Dynamic,
            color: FogColor::BLACK,
        }));

        stages.push(FogDrawStage::DrawLight(FogLightStage {
            kind: FogTextureKind::Dynamic,
            viewport,
            clip_padding: DEFAULT_FOG_EVENT_PADDING,
            sources: input.dynamic_sources,
        }));

        let consumed_events =
            if input.static_fog_enabled && (clear_static || !self.queued_events.is_empty()) {
                if clear_static {
                    stages.push(FogDrawStage::Clear(FogClearStage {
                        kind: FogTextureKind::Static,
                        color: FogColor::BLACK,
                    }));
                }

                let events = std::mem::take(&mut self.queued_events);
                stages.push(FogDrawStage::DrawFog(FogEventStage {
                    kind: FogTextureKind::Static,
                    viewport,
                    clip_padding: DEFAULT_FOG_EVENT_PADDING,
                    events: events.clone(),
                }));
                events
            } else {
                Vec::new()
            };

        stages.push(FogDrawStage::Composite(FogCompositeStage {
            kind: FogTextureKind::Dynamic,
            world_width,
            world_height,
            tile_size,
            offset_x: 0.0,
            offset_y: 0.0,
            resolve_sample: RenderTextureSamplePlan::fbo_uv_window(
                render_camera,
                world_width,
                world_height,
                tile_size as f32,
                0.0,
                0.0,
            ),
            color: input.dynamic_color,
            alpha: input.dynamic_color.dynamic_alpha(),
        }));

        if input.static_fog_enabled {
            stages.push(FogDrawStage::Composite(FogCompositeStage {
                kind: FogTextureKind::Static,
                world_width,
                world_height,
                tile_size,
                offset_x: 0.0,
                offset_y: tile_size as f32 / 2.0,
                resolve_sample: RenderTextureSamplePlan::fbo_uv_window(
                    render_camera,
                    world_width,
                    world_height,
                    tile_size as f32,
                    0.0,
                    tile_size as f32 / 2.0,
                ),
                color: input.static_color,
                alpha: 1.0,
            }));
        }

        let static_texture = FogTexturePlan {
            kind: FogTextureKind::Static,
            width: world_width,
            height: world_height,
            resized: static_resized,
            invalidated: self.static_texture.invalidated || static_resized,
            clear: clear_static,
            filter_linear: true,
        };
        let dynamic_texture = FogTexturePlan {
            kind: FogTextureKind::Dynamic,
            width: world_width,
            height: world_height,
            resized: dynamic_resized,
            invalidated: self.dynamic_texture.invalidated || dynamic_resized,
            clear: true,
            filter_linear: true,
        };

        self.static_texture.clear_invalidation();
        self.dynamic_texture.clear_invalidation();

        Some(FogFramePlan {
            viewport,
            static_texture,
            dynamic_texture,
            stages,
            consumed_events,
            team_changed,
            static_fog_enabled: input.static_fog_enabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_renderer_clear_resets_team_and_event_queue() {
        let mut state = FogRendererState::default();
        state.last_team = Some(7);
        state.handle_event(FogEvent::new(1, 2, 3.0));

        state.clear();

        assert_eq!(state.last_team, None);
        assert!(state.queued_events.is_empty());
    }

    #[test]
    fn fog_renderer_copy_plan_clips_world_border_tiles() {
        let discovered = vec![true; 9];
        let plan = FogCpuCopyPlan::from_discovered_map(3, 3, &discovered);

        assert_eq!(plan.world_width, 3);
        assert_eq!(plan.world_height, 3);
        assert_eq!(plan.padding, DEFAULT_FOG_EVENT_PADDING);
        assert_eq!(plan.revealed_tiles, vec![FogTileReveal::new(1, 1)]);
    }

    #[test]
    fn fog_renderer_draw_plan_respects_team_switch_and_consumes_events() {
        let mut state = FogRendererState::default();
        state.handle_event(FogEvent::new(4, 5, 6.0));

        let viewport = FogViewport::new(8, 8, 4, 12.4, 24.9, 30.0, 40.0);
        let mut input = FogFrameInput::new(
            viewport,
            2,
            true,
            true,
            FogColor::WHITE,
            FogColor::new(0.1, 0.2, 0.3, f32::NAN),
        );
        input
            .dynamic_sources
            .push(FogLightSource::new(10.0, 11.0, 12.0));
        input.discovered_tiles = Some(vec![true; 64]);

        let plan = state.draw_fog_plan(input).expect("fog should be visible");

        assert!(plan.team_changed);
        assert_eq!(
            plan.viewport.camera_pixel_rect(),
            FogRect::new(12, 24, 30, 40)
        );
        assert_eq!(plan.static_texture.kind, FogTextureKind::Static);
        assert_eq!(plan.dynamic_texture.kind, FogTextureKind::Dynamic);
        assert!(plan.dynamic_texture.clear);
        assert!(plan.static_texture.resized);
        assert!(!plan.consumed_events.is_empty());

        assert!(matches!(
            plan.stages.first(),
            Some(FogDrawStage::CopyFromCpu(_))
        ));
        assert!(plan
            .stages
            .iter()
            .any(|stage| matches!(stage, FogDrawStage::DrawLight(_))));
        assert!(plan
            .stages
            .iter()
            .any(|stage| matches!(stage, FogDrawStage::DrawFog(_))));
        assert!(plan
            .stages
            .iter()
            .any(|stage| matches!(stage, FogDrawStage::Composite(stage) if stage.kind == FogTextureKind::Dynamic && stage.resolve_sample.is_some())));
        assert!(plan
            .stages
            .iter()
            .any(|stage| matches!(stage, FogDrawStage::Composite(stage) if stage.kind == FogTextureKind::Static && stage.resolve_sample.is_some())));

        let passes = plan.to_render_passes();
        assert_eq!(passes.len(), plan.stages.len());
        assert_eq!(passes[0].kind, RenderPassKind::Fog);
        assert_eq!(passes[0].target, RenderTarget::Buffer("fog:static".into()));
        assert!(matches!(
            passes[0].commands.first(),
            Some(RenderCommand::Custom { name, properties })
                if name == "fog-copy-from-cpu"
                    && properties.iter().any(|property| property.key == "revealed_tiles")
        ));
        let dynamic_stage_sample = plan
            .stages
            .iter()
            .find_map(|stage| match stage {
                FogDrawStage::Composite(stage) if stage.kind == FogTextureKind::Dynamic => {
                    stage.resolve_sample
                }
                _ => None,
            })
            .expect("dynamic fog composite stage should carry Draw.fbo UV window");
        let dynamic_composite = passes
            .iter()
            .find(|pass| {
                pass.target == RenderTarget::Buffer("fog:dynamic".into())
                    && pass.resolve_target == Some(RenderTarget::Screen)
                    && pass.resolve_kind == Some(RenderResolveKind::DrawFboSample)
            })
            .expect("dynamic fog composite pass should resolve with Draw.fbo semantics");
        let dynamic_sample = dynamic_composite
            .resolve_sample
            .expect("dynamic fog composite should carry Draw.fbo UV window");
        assert_eq!(dynamic_stage_sample, dynamic_sample);
        assert_eq!(
            dynamic_sample.flip,
            crate::mindustry::graphics::RenderTextureSampleFlip::UvY
        );
        assert!((dynamic_sample.uv.u - 0.3875).abs() < 0.0001);
        assert!((dynamic_sample.uv.v - 2.028125).abs() < 0.0001);
        assert!((dynamic_sample.uv.u2 - 1.325).abs() < 0.0001);
        assert!((dynamic_sample.uv.v2 - 0.778125).abs() < 0.0001);

        let static_composite = passes
            .iter()
            .find(|pass| {
                pass.target == RenderTarget::Buffer("fog:static".into())
                    && pass.resolve_target == Some(RenderTarget::Screen)
                    && pass.resolve_kind == Some(RenderResolveKind::DrawFboSample)
            })
            .expect("static fog composite pass should resolve with Draw.fbo semantics");
        let static_stage_sample = plan
            .stages
            .iter()
            .find_map(|stage| match stage {
                FogDrawStage::Composite(stage) if stage.kind == FogTextureKind::Static => {
                    stage.resolve_sample
                }
                _ => None,
            })
            .expect("static fog composite stage should carry half-tile Draw.fbo UV window");
        let static_sample = static_composite
            .resolve_sample
            .expect("static fog composite should carry half-tile Draw.fbo UV window");
        assert_eq!(static_stage_sample, static_sample);
        assert_eq!(
            static_sample.flip,
            crate::mindustry::graphics::RenderTextureSampleFlip::UvY
        );
        assert!((static_sample.uv.u - 0.3875).abs() < 0.0001);
        assert!((static_sample.uv.v - 2.090625).abs() < 0.0001);
        assert!((static_sample.uv.u2 - 1.325).abs() < 0.0001);
        assert!((static_sample.uv.v2 - 0.840625).abs() < 0.0001);
        assert!((static_sample.uv.v - dynamic_sample.uv.v - 0.0625).abs() < 0.0001);

        assert!(state.queued_events.is_empty());
        assert_eq!(state.last_team, Some(2));
    }

    #[test]
    fn fog_renderer_skips_when_no_discovered_map_exists() {
        let mut state = FogRendererState::default();
        state.handle_event(FogEvent::new(4, 5, 6.0));

        let input = FogFrameInput::new(
            FogViewport::new(4, 4, 8, 0.0, 0.0, 64.0, 64.0),
            1,
            false,
            true,
            FogColor::WHITE,
            FogColor::BLACK,
        );

        assert_eq!(state.draw_fog_plan(input), None);
        assert_eq!(state.queued_events.len(), 1);
        assert_eq!(state.last_team, None);
    }

    #[test]
    fn fog_renderer_event_center_uses_even_block_offset() {
        let event = FogEvent::new(3, 4, 7.5).with_center_offset(0.5);
        assert_eq!(event.render_center(), (4.0, 5.0));
    }

    #[test]
    fn fog_renderer_dynamic_alpha_defaults_to_half_for_nan() {
        let color = FogColor::new(0.1, 0.2, 0.3, f32::NAN);
        assert_eq!(color.dynamic_alpha(), DEFAULT_STATIC_FOG_DYNAMIC_ALPHA);
    }
}
