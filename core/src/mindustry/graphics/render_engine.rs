//! Data-driven render planning core mirrored from Mindustry's graphics layer.
//!
//! This file intentionally stays backend-agnostic: it describes *what* should be
//! rendered, not *how* a GPU backend should render it.  The plan is built from
//! passes and commands so world modules such as block, floor, overlay and minimap
//! can reuse the same core vocabulary.

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderPoint {
    pub x: f32,
    pub y: f32,
}

impl RenderPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderSize {
    pub width: f32,
    pub height: f32,
}

impl RenderSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RenderRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_center(center: RenderPoint, width: f32, height: f32) -> Self {
        Self::new(
            center.x - width / 2.0,
            center.y - height / 2.0,
            width,
            height,
        )
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub fn center(self) -> RenderPoint {
        RenderPoint::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn contains_point(self, point: RenderPoint) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    pub fn intersects(self, other: Self) -> bool {
        self.x <= other.right()
            && self.right() >= other.x
            && self.y <= other.bottom()
            && self.bottom() >= other.y
    }

    pub fn inflate(self, amount: f32) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.width + amount * 2.0,
            self.height + amount * 2.0,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RenderViewport {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_rect(rect: RenderRect) -> Self {
        Self::new(rect.x, rect.y, rect.width, rect.height)
    }

    pub fn as_rect(self) -> RenderRect {
        RenderRect::new(self.x, self.y, self.width, self.height)
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub fn center(self) -> RenderPoint {
        RenderPoint::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn contains_point(self, point: RenderPoint) -> bool {
        self.as_rect().contains_point(point)
    }

    pub fn intersects(self, other: Self) -> bool {
        self.as_rect().intersects(other.as_rect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderCamera {
    pub center: RenderPoint,
    pub zoom: f32,
    pub rotation: f32,
    pub viewport: RenderViewport,
}

impl Default for RenderCamera {
    fn default() -> Self {
        Self {
            center: RenderPoint::default(),
            zoom: 1.0,
            rotation: 0.0,
            viewport: RenderViewport::default(),
        }
    }
}

impl RenderCamera {
    pub fn new(center: RenderPoint, viewport: RenderViewport) -> Self {
        Self {
            center,
            zoom: 1.0,
            rotation: 0.0,
            viewport,
        }
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn world_rect(self) -> RenderRect {
        if self.zoom <= 0.0 {
            return RenderRect::from_center(self.center, 0.0, 0.0);
        }

        let width = self.viewport.width / self.zoom;
        let height = self.viewport.height / self.zoom;
        RenderRect::from_center(self.center, width, height)
    }

    pub fn contains(self, point: RenderPoint) -> bool {
        self.world_rect().contains_point(point)
    }

    pub fn visible_tile_bounds(self, tile_size: f32) -> Option<(i32, i32, i32, i32)> {
        if tile_size <= 0.0 {
            return None;
        }

        let rect = self.world_rect();
        let min_x = (rect.x / tile_size).floor() as i32;
        let min_y = (rect.y / tile_size).floor() as i32;
        let max_x = (rect.right() / tile_size).ceil() as i32;
        let max_y = (rect.bottom() / tile_size).ceil() as i32;
        Some((min_x, min_y, max_x, max_y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBlendMode {
    Normal,
    Additive,
    Multiply,
    Screen,
    PremultipliedAlpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTextAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderTarget {
    Screen,
    Texture(String),
    Buffer(String),
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Screen
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderResolveKind {
    Blit,
    ShaderBlit,
    DrawRectSample,
    DrawFboSample,
}

impl RenderResolveKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Blit => "blit",
            Self::ShaderBlit => "shader_blit",
            Self::DrawRectSample => "draw_rect_sample",
            Self::DrawFboSample => "draw_fbo_sample",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProperty {
    pub key: String,
    pub value: String,
}

impl RenderProperty {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    Clear {
        color: [f32; 4],
    },
    SetBlend {
        mode: RenderBlendMode,
    },
    SetClip {
        rect: RenderRect,
    },
    ClearClip,
    FillRect {
        rect: RenderRect,
        color: [f32; 4],
        layer: f32,
    },
    StrokeRect {
        rect: RenderRect,
        color: [f32; 4],
        thickness: f32,
        layer: f32,
    },
    DrawSprite {
        symbol: String,
        rect: RenderRect,
        tint: [f32; 4],
        rotation: f32,
        layer: f32,
    },
    DrawLine {
        from: RenderPoint,
        to: RenderPoint,
        stroke: f32,
        color: [f32; 4],
        layer: f32,
    },
    DrawCircle {
        center: RenderPoint,
        radius: f32,
        color: [f32; 4],
        filled: bool,
        layer: f32,
    },
    DrawPolygon {
        center: RenderPoint,
        radius: f32,
        sides: usize,
        rotation: f32,
        color: [f32; 4],
        filled: bool,
        layer: f32,
    },
    DrawPixel {
        x: i32,
        y: i32,
        color: [f32; 4],
        layer: f32,
    },
    DrawText {
        text: String,
        position: RenderPoint,
        color: [f32; 4],
        size: f32,
        rotation: f32,
        align: RenderTextAlign,
        layer: f32,
    },
    Custom {
        name: String,
        properties: Vec<RenderProperty>,
    },
}

impl RenderCommand {
    pub fn clear(color: [f32; 4]) -> Self {
        Self::Clear { color }
    }

    pub fn set_blend(mode: RenderBlendMode) -> Self {
        Self::SetBlend { mode }
    }

    pub fn set_clip(rect: RenderRect) -> Self {
        Self::SetClip { rect }
    }

    pub fn clear_clip() -> Self {
        Self::ClearClip
    }

    pub fn fill_rect(rect: RenderRect, color: [f32; 4], layer: f32) -> Self {
        Self::FillRect { rect, color, layer }
    }

    pub fn stroke_rect(rect: RenderRect, color: [f32; 4], thickness: f32, layer: f32) -> Self {
        Self::StrokeRect {
            rect,
            color,
            thickness,
            layer,
        }
    }

    pub fn draw_sprite(
        symbol: impl Into<String>,
        rect: RenderRect,
        tint: [f32; 4],
        rotation: f32,
        layer: f32,
    ) -> Self {
        Self::DrawSprite {
            symbol: symbol.into(),
            rect,
            tint,
            rotation,
            layer,
        }
    }

    pub fn draw_line(
        from: RenderPoint,
        to: RenderPoint,
        stroke: f32,
        color: [f32; 4],
        layer: f32,
    ) -> Self {
        Self::DrawLine {
            from,
            to,
            stroke,
            color,
            layer,
        }
    }

    pub fn draw_circle(
        center: RenderPoint,
        radius: f32,
        color: [f32; 4],
        filled: bool,
        layer: f32,
    ) -> Self {
        Self::DrawCircle {
            center,
            radius,
            color,
            filled,
            layer,
        }
    }

    pub fn draw_polygon(
        center: RenderPoint,
        radius: f32,
        sides: usize,
        rotation: f32,
        color: [f32; 4],
        filled: bool,
        layer: f32,
    ) -> Self {
        Self::DrawPolygon {
            center,
            radius,
            sides,
            rotation,
            color,
            filled,
            layer,
        }
    }

    pub fn draw_pixel(x: i32, y: i32, color: [f32; 4], layer: f32) -> Self {
        Self::DrawPixel { x, y, color, layer }
    }

    pub fn draw_text(
        text: impl Into<String>,
        position: RenderPoint,
        color: [f32; 4],
        size: f32,
        rotation: f32,
        align: RenderTextAlign,
        layer: f32,
    ) -> Self {
        Self::DrawText {
            text: text.into(),
            position,
            color,
            size,
            rotation,
            align,
            layer,
        }
    }

    pub fn custom(name: impl Into<String>, properties: Vec<RenderProperty>) -> Self {
        Self::Custom {
            name: name.into(),
            properties,
        }
    }

    pub fn backend_flush_boundary(&self) -> Option<RenderBackendFlushBoundary> {
        match self {
            Self::Clear { .. } => Some(RenderBackendFlushBoundary::Clear),
            Self::SetBlend { .. } => Some(RenderBackendFlushBoundary::BlendState),
            Self::SetClip { .. } | Self::ClearClip => Some(RenderBackendFlushBoundary::ClipState),
            Self::Custom { .. } => Some(RenderBackendFlushBoundary::Custom),
            Self::FillRect { .. }
            | Self::StrokeRect { .. }
            | Self::DrawSprite { .. }
            | Self::DrawLine { .. }
            | Self::DrawCircle { .. }
            | Self::DrawPolygon { .. }
            | Self::DrawPixel { .. }
            | Self::DrawText { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendFlushBoundary {
    Clear,
    BlendState,
    ClipState,
    Custom,
}

impl RenderBackendFlushBoundary {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::BlendState => "blend_state",
            Self::ClipState => "clip_state",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPassExecutionStepKind {
    BeginPass,
    FlushBoundary,
    Command,
    EndPass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPassExecutionStep {
    pub kind: RenderPassExecutionStepKind,
    pub command_index: Option<usize>,
    pub boundary: Option<RenderBackendFlushBoundary>,
    pub label: &'static str,
}

impl RenderPassExecutionStep {
    pub const fn begin_pass() -> Self {
        Self {
            kind: RenderPassExecutionStepKind::BeginPass,
            command_index: None,
            boundary: None,
            label: "begin_pass",
        }
    }

    pub const fn flush_boundary(
        command_index: usize,
        boundary: RenderBackendFlushBoundary,
    ) -> Self {
        Self {
            kind: RenderPassExecutionStepKind::FlushBoundary,
            command_index: Some(command_index),
            boundary: Some(boundary),
            label: boundary.label(),
        }
    }

    pub const fn command(command_index: usize) -> Self {
        Self {
            kind: RenderPassExecutionStepKind::Command,
            command_index: Some(command_index),
            boundary: None,
            label: "command",
        }
    }

    pub const fn end_pass() -> Self {
        Self {
            kind: RenderPassExecutionStepKind::EndPass,
            command_index: None,
            boundary: None,
            label: "end_pass",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderPassKind {
    Background,
    Floor,
    BlockShadows,
    BlockWalls,
    Block,
    Overlay,
    Fog,
    Minimap,
    Lighting,
    Darkness,
    Ui,
    Custom(String),
}

impl RenderPassKind {
    pub fn label(&self) -> &str {
        match self {
            Self::Background => "background",
            Self::Floor => "floor",
            Self::BlockShadows => "block_shadows",
            Self::BlockWalls => "block_walls",
            Self::Block => "block",
            Self::Overlay => "overlay",
            Self::Fog => "fog",
            Self::Minimap => "minimap",
            Self::Lighting => "lighting",
            Self::Darkness => "darkness",
            Self::Ui => "ui",
            Self::Custom(name) => name.as_str(),
        }
    }

    pub const fn default_order(&self) -> i32 {
        match self {
            Self::Background => 0,
            Self::Floor => 10,
            Self::BlockShadows => 20,
            Self::BlockWalls => 30,
            Self::Block => 20,
            Self::Overlay => 30,
            Self::Fog => 90,
            Self::Minimap => 40,
            Self::Lighting => 50,
            Self::Darkness => 70,
            Self::Ui => 60,
            Self::Custom(_) => 1_000,
        }
    }

    pub const fn java_renderer_draw_stage(&self) -> RendererDrawStage {
        match self {
            Self::Background => RendererDrawStage::Background,
            Self::Floor => RendererDrawStage::Floor,
            Self::BlockShadows => RendererDrawStage::BlockShadows,
            Self::BlockWalls => RendererDrawStage::BlockWalls,
            Self::Block => RendererDrawStage::BlockBuild,
            Self::Overlay => RendererDrawStage::Overlay,
            Self::Fog => RendererDrawStage::Fog,
            Self::Minimap => RendererDrawStage::Debug,
            Self::Lighting => RendererDrawStage::Lighting,
            Self::Darkness => RendererDrawStage::Darkness,
            Self::Ui => RendererDrawStage::Ui,
            Self::Custom(_) => RendererDrawStage::Debug,
        }
    }

    pub const fn java_renderer_draw_rank(&self) -> i32 {
        self.java_renderer_draw_stage().sort_key()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RendererDrawStage {
    Background,
    Floor,
    BlockShadows,
    BlockWalls,
    BlockBuild,
    Environment,
    Lighting,
    Darkness,
    Overlay,
    Fog,
    BlockOverdraw,
    Ui,
    Debug,
}

impl RendererDrawStage {
    pub const ORDERED: [Self; 13] = [
        Self::Background,
        Self::Floor,
        Self::BlockShadows,
        Self::BlockWalls,
        Self::BlockBuild,
        Self::Environment,
        Self::Lighting,
        Self::Darkness,
        Self::Overlay,
        Self::Fog,
        Self::BlockOverdraw,
        Self::Ui,
        Self::Debug,
    ];

    pub const fn ordered() -> &'static [Self; 13] {
        &Self::ORDERED
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Floor => "floor",
            Self::BlockShadows => "block_shadow",
            Self::BlockWalls => "block_walls",
            Self::BlockBuild => "block_build",
            Self::Environment => "env",
            Self::Lighting => "light",
            Self::Darkness => "darkness",
            Self::Overlay => "overlay",
            Self::Fog => "fog",
            Self::BlockOverdraw => "block_overdraw",
            Self::Ui => "ui",
            Self::Debug => "debug",
        }
    }

    pub const fn sort_key(self) -> i32 {
        match self {
            Self::Background => 0,
            Self::Floor => 10,
            Self::BlockShadows => 20,
            Self::BlockWalls => 30,
            Self::BlockBuild => 40,
            Self::Environment => 50,
            Self::Lighting => 60,
            Self::Darkness => 70,
            Self::Overlay => 80,
            Self::Fog => 90,
            Self::BlockOverdraw => 100,
            Self::Ui => 110,
            Self::Debug => 120,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderPass {
    pub kind: RenderPassKind,
    pub order: i32,
    pub target: RenderTarget,
    pub resolve_target: Option<RenderTarget>,
    pub resolve_kind: Option<RenderResolveKind>,
    pub viewport: Option<RenderViewport>,
    pub camera: Option<RenderCamera>,
    pub commands: Vec<RenderCommand>,
}

impl RenderPass {
    pub fn new(kind: RenderPassKind) -> Self {
        let order = kind.default_order();
        Self {
            kind,
            order,
            target: RenderTarget::default(),
            resolve_target: None,
            resolve_kind: None,
            viewport: None,
            camera: None,
            commands: Vec::new(),
        }
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn with_target(mut self, target: RenderTarget) -> Self {
        self.target = target;
        self
    }

    pub fn with_resolve_target(mut self, target: RenderTarget) -> Self {
        self.resolve_target = Some(target);
        self.resolve_kind = Some(RenderResolveKind::Blit);
        self
    }

    pub fn with_resolve(mut self, target: RenderTarget, kind: RenderResolveKind) -> Self {
        self.resolve_target = Some(target);
        self.resolve_kind = Some(kind);
        self
    }

    pub fn with_viewport(mut self, viewport: RenderViewport) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn with_camera(mut self, camera: RenderCamera) -> Self {
        self.camera = Some(camera);
        self
    }

    pub fn push(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    pub fn extend(&mut self, commands: impl IntoIterator<Item = RenderCommand>) {
        self.commands.extend(commands);
    }

    pub fn backend_execution_steps(&self) -> Vec<RenderPassExecutionStep> {
        let mut steps = Vec::with_capacity(self.commands.len() * 2 + 2);
        steps.push(RenderPassExecutionStep::begin_pass());

        for (index, command) in self.commands.iter().enumerate() {
            if let Some(boundary) = command.backend_flush_boundary() {
                steps.push(RenderPassExecutionStep::flush_boundary(index, boundary));
            }
            steps.push(RenderPassExecutionStep::command(index));
        }

        steps.push(RenderPassExecutionStep::end_pass());
        steps
    }

    pub fn effective_viewport(&self, fallback: RenderViewport) -> RenderViewport {
        self.viewport.unwrap_or(fallback)
    }

    pub fn effective_camera(&self, fallback: RenderCamera) -> RenderCamera {
        self.camera.unwrap_or(fallback)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFramePlan {
    pub frame_index: u64,
    pub world_size: RenderSize,
    pub camera: RenderCamera,
    pub viewport: RenderViewport,
    pub passes: Vec<RenderPass>,
}

impl RenderFramePlan {
    pub fn new(
        frame_index: u64,
        world_size: RenderSize,
        camera: RenderCamera,
        viewport: RenderViewport,
    ) -> Self {
        Self {
            frame_index,
            world_size,
            camera,
            viewport,
            passes: Vec::new(),
        }
    }

    pub fn push_pass(&mut self, pass: RenderPass) -> usize {
        self.passes.push(pass);
        self.passes.len() - 1
    }

    pub fn sort_passes(&mut self) {
        self.passes.sort_by_key(|pass| pass.order);
    }

    pub fn sort_passes_like_java_renderer_draw(&mut self) {
        self.passes
            .sort_by_key(|pass| (pass.kind.java_renderer_draw_rank(), pass.order));
    }

    pub fn pass(&self, kind: &RenderPassKind) -> Option<&RenderPass> {
        self.passes.iter().find(|pass| &pass.kind == kind)
    }

    pub fn pass_mut(&mut self, kind: &RenderPassKind) -> Option<&mut RenderPass> {
        self.passes.iter_mut().find(|pass| &pass.kind == kind)
    }

    pub fn command_count(&self) -> usize {
        self.passes.iter().map(|pass| pass.commands.len()).sum()
    }

    pub fn commands(&self) -> impl Iterator<Item = &RenderCommand> {
        self.passes.iter().flat_map(|pass| pass.commands.iter())
    }

    pub const fn java_renderer_draw_stages() -> &'static [RendererDrawStage; 13] {
        RendererDrawStage::ordered()
    }

    pub fn matches_java_renderer_draw_order(&self) -> bool {
        let mut previous_rank = i32::MIN;

        for pass in &self.passes {
            let rank = pass.kind.java_renderer_draw_rank();
            if rank < previous_rank {
                return false;
            }
            previous_rank = rank;
        }

        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderEngineState {
    pub frame_index: u64,
    pub world_size: RenderSize,
    pub camera: RenderCamera,
    pub viewport: RenderViewport,
    pub passes: Vec<RenderPass>,
}

impl Default for RenderEngineState {
    fn default() -> Self {
        Self {
            frame_index: 0,
            world_size: RenderSize::default(),
            camera: RenderCamera::default(),
            viewport: RenderViewport::default(),
            passes: Vec::new(),
        }
    }
}

impl RenderEngineState {
    pub fn new(world_size: RenderSize, camera: RenderCamera) -> Self {
        Self {
            frame_index: 0,
            world_size,
            viewport: camera.viewport,
            camera,
            passes: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        self.frame_index = frame_index;
        self.passes.clear();
    }

    pub fn set_camera(&mut self, camera: RenderCamera) {
        self.viewport = camera.viewport;
        self.camera = camera;
    }

    pub fn set_viewport(&mut self, viewport: RenderViewport) {
        self.viewport = viewport;
        self.camera.viewport = viewport;
    }

    pub fn push_pass(&mut self, pass: RenderPass) -> usize {
        self.passes.push(pass);
        self.passes.len() - 1
    }

    pub fn pass_mut(&mut self, index: usize) -> Option<&mut RenderPass> {
        self.passes.get_mut(index)
    }

    pub fn finish(mut self) -> RenderFramePlan {
        self.passes.sort_by_key(|pass| pass.order);
        RenderFramePlan {
            frame_index: self.frame_index,
            world_size: self.world_size,
            camera: self.camera,
            viewport: self.viewport,
            passes: self.passes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_projects_viewport_into_world_bounds_and_tiles() {
        let viewport = RenderViewport::new(0.0, 0.0, 160.0, 90.0);
        let camera = RenderCamera::new(RenderPoint::new(50.0, 25.0), viewport).with_zoom(2.0);

        let rect = camera.world_rect();
        assert_eq!(rect, RenderRect::new(10.0, 2.5, 80.0, 45.0));
        assert!(camera.contains(RenderPoint::new(50.0, 25.0)));
        assert!(!camera.contains(RenderPoint::new(0.0, 0.0)));

        assert_eq!(camera.visible_tile_bounds(10.0), Some((1, 0, 9, 5)));
        assert_eq!(camera.visible_tile_bounds(0.0), None);
    }

    #[test]
    fn frame_plan_sorts_passes_by_render_role() {
        let viewport = RenderViewport::new(0.0, 0.0, 128.0, 72.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 8.0), viewport);
        let mut state = RenderEngineState::new(RenderSize::new(256.0, 256.0), camera);

        state.begin_frame(42);

        let minimap = state.push_pass(
            RenderPass::new(RenderPassKind::Minimap)
                .with_target(RenderTarget::Texture("minimap-buffer".into())),
        );
        state
            .pass_mut(minimap)
            .unwrap()
            .push(RenderCommand::draw_pixel(5, 7, [0.1, 0.2, 0.3, 1.0], 40.0));

        let block = state.push_pass(RenderPass::new(RenderPassKind::Block));
        state
            .pass_mut(block)
            .unwrap()
            .push(RenderCommand::draw_sprite(
                "block-core",
                RenderRect::new(12.0, 8.0, 16.0, 16.0),
                [1.0, 1.0, 1.0, 1.0],
                0.0,
                20.0,
            ));

        let floor = state.push_pass(RenderPass::new(RenderPassKind::Floor));
        state.pass_mut(floor).unwrap().extend([
            RenderCommand::clear([0.0, 0.0, 0.0, 0.0]),
            RenderCommand::fill_rect(
                RenderRect::new(0.0, 0.0, 32.0, 32.0),
                [0.7, 0.7, 0.7, 1.0],
                10.0,
            ),
        ]);

        let plan = state.finish();
        assert_eq!(plan.frame_index, 42);
        assert_eq!(plan.passes.len(), 3);
        assert_eq!(plan.passes[0].kind, RenderPassKind::Floor);
        assert_eq!(plan.passes[1].kind, RenderPassKind::Block);
        assert_eq!(plan.passes[2].kind, RenderPassKind::Minimap);
        assert_eq!(plan.command_count(), 4);
        assert_eq!(plan.commands().count(), 4);
    }

    #[test]
    fn full_frame_order_matches_java_renderer_draw() {
        let expected = [
            RendererDrawStage::Background,
            RendererDrawStage::Floor,
            RendererDrawStage::BlockShadows,
            RendererDrawStage::BlockWalls,
            RendererDrawStage::BlockBuild,
            RendererDrawStage::Environment,
            RendererDrawStage::Lighting,
            RendererDrawStage::Darkness,
            RendererDrawStage::Overlay,
            RendererDrawStage::Fog,
            RendererDrawStage::BlockOverdraw,
            RendererDrawStage::Ui,
            RendererDrawStage::Debug,
        ];

        assert_eq!(RendererDrawStage::ordered(), &expected);
        assert_eq!(RenderFramePlan::java_renderer_draw_stages(), &expected);
        assert_eq!(
            expected
                .iter()
                .map(|stage| stage.label())
                .collect::<Vec<_>>(),
            vec![
                "background",
                "floor",
                "block_shadow",
                "block_walls",
                "block_build",
                "env",
                "light",
                "darkness",
                "overlay",
                "fog",
                "block_overdraw",
                "ui",
                "debug",
            ]
        );

        let viewport = RenderViewport::new(0.0, 0.0, 128.0, 72.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 8.0), viewport);
        let mut state = RenderEngineState::new(RenderSize::new(256.0, 256.0), camera);
        state.begin_frame(7);

        let overlay = state.push_pass(RenderPass::new(RenderPassKind::Overlay).with_order(40));
        let floor = state.push_pass(RenderPass::new(RenderPassKind::Floor).with_order(10));
        let block = state.push_pass(RenderPass::new(RenderPassKind::Block).with_order(20));
        let ui = state.push_pass(RenderPass::new(RenderPassKind::Ui).with_order(50));
        let lighting = state.push_pass(RenderPass::new(RenderPassKind::Lighting).with_order(30));
        let background = state.push_pass(RenderPass::new(RenderPassKind::Background).with_order(0));

        state
            .pass_mut(background)
            .unwrap()
            .push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        state
            .pass_mut(floor)
            .unwrap()
            .push(RenderCommand::fill_rect(
                RenderRect::new(0.0, 0.0, 8.0, 8.0),
                [0.25, 0.25, 0.25, 1.0],
                10.0,
            ));
        state
            .pass_mut(block)
            .unwrap()
            .push(RenderCommand::draw_sprite(
                "block-core",
                RenderRect::new(4.0, 4.0, 8.0, 8.0),
                [1.0, 1.0, 1.0, 1.0],
                0.0,
                20.0,
            ));
        state
            .pass_mut(lighting)
            .unwrap()
            .push(RenderCommand::draw_circle(
                RenderPoint::new(8.0, 8.0),
                2.0,
                [1.0, 1.0, 0.8, 1.0],
                true,
                60.0,
            ));
        state
            .pass_mut(overlay)
            .unwrap()
            .push(RenderCommand::draw_text(
                "overlay",
                RenderPoint::new(2.0, 2.0),
                [1.0, 1.0, 1.0, 1.0],
                12.0,
                0.0,
                RenderTextAlign::Start,
                80.0,
            ));
        state.pass_mut(ui).unwrap().push(RenderCommand::custom(
            "debug-ui",
            vec![RenderProperty::new("stage", "ui")],
        ));

        let mut plan = state.finish();
        plan.sort_passes_like_java_renderer_draw();

        assert_eq!(
            plan.passes
                .iter()
                .map(|pass| pass.kind.label())
                .collect::<Vec<_>>(),
            vec!["background", "floor", "block", "lighting", "overlay", "ui"]
        );
        assert!(plan.matches_java_renderer_draw_order());
    }

    #[test]
    fn block_pass_sorts_after_shadows_and_before_lighting_like_java_renderer() {
        let viewport = RenderViewport::new(0.0, 0.0, 64.0, 64.0);
        let camera = RenderCamera::new(RenderPoint::new(16.0, 16.0), viewport);
        let mut plan = RenderFramePlan::new(7, RenderSize::new(64.0, 64.0), camera, viewport);

        plan.push_pass(RenderPass::new(RenderPassKind::Lighting).with_order(0));
        plan.push_pass(RenderPass::new(RenderPassKind::Block).with_order(0));
        plan.push_pass(RenderPass::new(RenderPassKind::BlockWalls).with_order(0));
        plan.push_pass(RenderPass::new(RenderPassKind::BlockShadows).with_order(99));
        plan.push_pass(RenderPass::new(RenderPassKind::Floor).with_order(100));
        plan.sort_passes_like_java_renderer_draw();

        assert_eq!(
            plan.passes
                .iter()
                .map(|pass| pass.kind.clone())
                .collect::<Vec<_>>(),
            vec![
                RenderPassKind::Floor,
                RenderPassKind::BlockShadows,
                RenderPassKind::BlockWalls,
                RenderPassKind::Block,
                RenderPassKind::Lighting,
            ]
        );
        assert_eq!(
            RenderPassKind::Block.java_renderer_draw_stage(),
            RendererDrawStage::BlockBuild
        );
        assert!(plan.matches_java_renderer_draw_order());
    }

    #[test]
    fn java_renderer_stage_and_pass_mapping_is_exhaustive_and_ordered() {
        let expected_stages = [
            (RendererDrawStage::Background, "background", 0),
            (RendererDrawStage::Floor, "floor", 10),
            (RendererDrawStage::BlockShadows, "block_shadow", 20),
            (RendererDrawStage::BlockWalls, "block_walls", 30),
            (RendererDrawStage::BlockBuild, "block_build", 40),
            (RendererDrawStage::Environment, "env", 50),
            (RendererDrawStage::Lighting, "light", 60),
            (RendererDrawStage::Darkness, "darkness", 70),
            (RendererDrawStage::Overlay, "overlay", 80),
            (RendererDrawStage::Fog, "fog", 90),
            (RendererDrawStage::BlockOverdraw, "block_overdraw", 100),
            (RendererDrawStage::Ui, "ui", 110),
            (RendererDrawStage::Debug, "debug", 120),
        ];

        assert_eq!(RendererDrawStage::ordered().len(), expected_stages.len());
        for ((stage, label, sort_key), expected_stage) in expected_stages
            .into_iter()
            .zip(RendererDrawStage::ordered().iter())
        {
            assert_eq!(*expected_stage, stage);
            assert_eq!(stage.label(), label);
            assert_eq!(stage.sort_key(), sort_key);
        }

        let pass_cases = [
            (
                RenderPassKind::Background,
                "background",
                0,
                RendererDrawStage::Background,
            ),
            (RenderPassKind::Floor, "floor", 10, RendererDrawStage::Floor),
            (
                RenderPassKind::BlockShadows,
                "block_shadows",
                20,
                RendererDrawStage::BlockShadows,
            ),
            (
                RenderPassKind::BlockWalls,
                "block_walls",
                30,
                RendererDrawStage::BlockWalls,
            ),
            (
                RenderPassKind::Block,
                "block",
                20,
                RendererDrawStage::BlockBuild,
            ),
            (
                RenderPassKind::Overlay,
                "overlay",
                30,
                RendererDrawStage::Overlay,
            ),
            (RenderPassKind::Fog, "fog", 90, RendererDrawStage::Fog),
            (
                RenderPassKind::Minimap,
                "minimap",
                40,
                RendererDrawStage::Debug,
            ),
            (
                RenderPassKind::Lighting,
                "lighting",
                50,
                RendererDrawStage::Lighting,
            ),
            (
                RenderPassKind::Darkness,
                "darkness",
                70,
                RendererDrawStage::Darkness,
            ),
            (RenderPassKind::Ui, "ui", 60, RendererDrawStage::Ui),
        ];

        for (kind, label, default_order, stage) in pass_cases {
            assert_eq!(kind.label(), label);
            assert_eq!(kind.default_order(), default_order);
            assert_eq!(kind.java_renderer_draw_stage(), stage);
            assert_eq!(kind.java_renderer_draw_rank(), stage.sort_key());
        }

        let custom = RenderPassKind::Custom("postprocess".into());
        assert_eq!(custom.label(), "postprocess");
        assert_eq!(custom.default_order(), 1_000);
        assert_eq!(custom.java_renderer_draw_stage(), RendererDrawStage::Debug);
        assert_eq!(
            custom.java_renderer_draw_rank(),
            RendererDrawStage::Debug.sort_key()
        );
    }

    #[test]
    fn command_payloads_round_trip_for_overlay_and_custom_data() {
        let rect = RenderRect::new(2.0, 4.0, 8.0, 16.0);
        let clip = RenderRect::new(0.0, 0.0, 32.0, 32.0);
        let pass = RenderPass::new(RenderPassKind::Overlay)
            .with_target(RenderTarget::Buffer("overlay".into()))
            .with_viewport(RenderViewport::from_rect(clip))
            .with_camera(RenderCamera::new(
                RenderPoint::new(10.0, 10.0),
                RenderViewport::from_rect(clip),
            ))
            .with_order(123);

        assert_eq!(pass.kind.label(), "overlay");
        assert_eq!(pass.order, 123);
        assert_eq!(pass.target, RenderTarget::Buffer("overlay".into()));
        assert_eq!(pass.resolve_target, None);
        assert_eq!(pass.resolve_kind, None);
        assert_eq!(
            pass.effective_viewport(RenderViewport::default()),
            clip.into()
        );

        let sprite =
            RenderCommand::draw_sprite("overlay-icon", rect, [1.0, 0.5, 0.25, 0.75], 90.0, 30.0);
        let text = RenderCommand::draw_text(
            "minimap label",
            RenderPoint::new(6.0, 7.0),
            [1.0, 1.0, 1.0, 1.0],
            12.0,
            0.0,
            RenderTextAlign::Center,
            60.0,
        );
        let polygon = RenderCommand::draw_polygon(
            RenderPoint::new(8.0, 9.0),
            6.5,
            5,
            15.0,
            [0.2, 0.3, 0.4, 0.5],
            true,
            70.0,
        );
        let custom = RenderCommand::custom(
            "module-bridge",
            vec![
                RenderProperty::new("source", "overlay"),
                RenderProperty::new("kind", "debug"),
            ],
        );

        match sprite {
            RenderCommand::DrawSprite {
                symbol,
                rect: sprite_rect,
                tint,
                rotation,
                layer,
            } => {
                assert_eq!(symbol, "overlay-icon");
                assert_eq!(sprite_rect, rect);
                assert_eq!(tint, [1.0, 0.5, 0.25, 0.75]);
                assert_eq!(rotation, 90.0);
                assert_eq!(layer, 30.0);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        match text {
            RenderCommand::DrawText {
                text,
                position,
                size,
                align,
                layer,
                ..
            } => {
                assert_eq!(text, "minimap label");
                assert_eq!(position, RenderPoint::new(6.0, 7.0));
                assert_eq!(size, 12.0);
                assert_eq!(align, RenderTextAlign::Center);
                assert_eq!(layer, 60.0);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        match polygon {
            RenderCommand::DrawPolygon {
                center,
                radius,
                sides,
                rotation,
                color,
                filled,
                layer,
            } => {
                assert_eq!(center, RenderPoint::new(8.0, 9.0));
                assert_eq!(radius, 6.5);
                assert_eq!(sides, 5);
                assert_eq!(rotation, 15.0);
                assert_eq!(color, [0.2, 0.3, 0.4, 0.5]);
                assert!(filled);
                assert_eq!(layer, 70.0);
            }
            other => panic!("unexpected command: {other:?}"),
        }

        match custom {
            RenderCommand::Custom { name, properties } => {
                assert_eq!(name, "module-bridge");
                assert_eq!(
                    properties,
                    vec![
                        RenderProperty::new("source", "overlay"),
                        RenderProperty::new("kind", "debug"),
                    ]
                );
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn render_pass_resolve_target_models_explicit_backend_blit() {
        let pass = RenderPass::new(RenderPassKind::Lighting)
            .with_target(RenderTarget::Texture("effect-buffer".into()))
            .with_resolve_target(RenderTarget::Screen);

        assert_eq!(pass.target, RenderTarget::Texture("effect-buffer".into()));
        assert_eq!(pass.resolve_target, Some(RenderTarget::Screen));
        assert_eq!(pass.resolve_kind, Some(RenderResolveKind::Blit));
    }

    #[test]
    fn render_resolve_kind_labels_cover_java_fbo_resolve_paths() {
        assert_eq!(RenderResolveKind::Blit.label(), "blit");
        assert_eq!(RenderResolveKind::ShaderBlit.label(), "shader_blit");
        assert_eq!(
            RenderResolveKind::DrawRectSample.label(),
            "draw_rect_sample"
        );
        assert_eq!(RenderResolveKind::DrawFboSample.label(), "draw_fbo_sample");

        let pass = RenderPass::new(RenderPassKind::Overlay)
            .with_target(RenderTarget::Buffer("dark".into()))
            .with_resolve(RenderTarget::Screen, RenderResolveKind::DrawFboSample);

        assert_eq!(pass.resolve_target, Some(RenderTarget::Screen));
        assert_eq!(pass.resolve_kind, Some(RenderResolveKind::DrawFboSample));
    }

    #[test]
    fn render_pass_backend_execution_steps_mark_state_flush_boundaries() {
        let mut pass = RenderPass::new(RenderPassKind::Block);
        pass.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        pass.push(RenderCommand::draw_sprite(
            "router",
            RenderRect::new(1.0, 2.0, 8.0, 8.0),
            [1.0, 1.0, 1.0, 1.0],
            0.0,
            20.0,
        ));
        pass.push(RenderCommand::set_blend(RenderBlendMode::Additive));
        pass.push(RenderCommand::set_clip(RenderRect::new(
            0.0, 0.0, 16.0, 16.0,
        )));
        pass.push(RenderCommand::draw_text(
            "label",
            RenderPoint::new(4.0, 5.0),
            [1.0, 1.0, 1.0, 1.0],
            12.0,
            0.0,
            RenderTextAlign::Center,
            30.0,
        ));
        pass.push(RenderCommand::draw_polygon(
            RenderPoint::new(6.0, 7.0),
            8.0,
            6,
            30.0,
            [0.3, 0.4, 0.5, 0.6],
            true,
            31.0,
        ));
        pass.push(RenderCommand::custom(
            "backend-marker",
            vec![RenderProperty::new("stage", "block")],
        ));

        assert_eq!(
            pass.commands
                .iter()
                .map(RenderCommand::backend_flush_boundary)
                .collect::<Vec<_>>(),
            vec![
                Some(RenderBackendFlushBoundary::Clear),
                None,
                Some(RenderBackendFlushBoundary::BlendState),
                Some(RenderBackendFlushBoundary::ClipState),
                None,
                None,
                Some(RenderBackendFlushBoundary::Custom),
            ]
        );

        let steps = pass.backend_execution_steps();
        assert_eq!(steps.first(), Some(&RenderPassExecutionStep::begin_pass()));
        assert_eq!(steps.last(), Some(&RenderPassExecutionStep::end_pass()));
        assert_eq!(
            steps
                .iter()
                .filter(|step| step.kind == RenderPassExecutionStepKind::Command)
                .map(|step| step.command_index)
                .collect::<Vec<_>>(),
            vec![
                Some(0),
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6)
            ]
        );
        assert_eq!(
            steps
                .iter()
                .filter_map(|step| step.boundary)
                .collect::<Vec<_>>(),
            vec![
                RenderBackendFlushBoundary::Clear,
                RenderBackendFlushBoundary::BlendState,
                RenderBackendFlushBoundary::ClipState,
                RenderBackendFlushBoundary::Custom,
            ]
        );
        assert_eq!(
            steps
                .iter()
                .filter(|step| step.kind == RenderPassExecutionStepKind::FlushBoundary)
                .map(|step| (step.command_index, step.label))
                .collect::<Vec<_>>(),
            vec![
                (Some(0), "clear"),
                (Some(2), "blend_state"),
                (Some(3), "clip_state"),
                (Some(6), "custom"),
            ]
        );
    }

    #[test]
    fn pass_helpers_preserve_fallbacks_and_state_update_camera_viewport() {
        let viewport = RenderViewport::new(1.0, 2.0, 3.0, 4.0);
        let camera = RenderCamera::new(RenderPoint::new(9.0, 10.0), viewport);
        let fallback_camera = RenderCamera::default();
        let fallback_viewport = RenderViewport::default();

        let pass = RenderPass::new(RenderPassKind::Background);
        assert_eq!(pass.effective_camera(fallback_camera), fallback_camera);
        assert_eq!(
            pass.effective_viewport(fallback_viewport),
            fallback_viewport
        );

        let mut state = RenderEngineState::new(RenderSize::new(100.0, 100.0), camera);
        assert_eq!(state.viewport, viewport);

        let updated = RenderViewport::new(8.0, 9.0, 10.0, 11.0);
        state.set_viewport(updated);
        assert_eq!(state.viewport, updated);
        assert_eq!(state.camera.viewport, updated);
    }
}

impl From<RenderRect> for RenderViewport {
    fn from(value: RenderRect) -> Self {
        Self::from_rect(value)
    }
}
