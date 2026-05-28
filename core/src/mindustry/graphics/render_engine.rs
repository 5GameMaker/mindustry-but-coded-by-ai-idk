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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderPassKind {
    Background,
    Floor,
    Block,
    Overlay,
    Minimap,
    Lighting,
    Ui,
    Custom(String),
}

impl RenderPassKind {
    pub fn label(&self) -> &str {
        match self {
            Self::Background => "background",
            Self::Floor => "floor",
            Self::Block => "block",
            Self::Overlay => "overlay",
            Self::Minimap => "minimap",
            Self::Lighting => "lighting",
            Self::Ui => "ui",
            Self::Custom(name) => name.as_str(),
        }
    }

    pub const fn default_order(&self) -> i32 {
        match self {
            Self::Background => 0,
            Self::Floor => 10,
            Self::Block => 20,
            Self::Overlay => 30,
            Self::Minimap => 40,
            Self::Lighting => 50,
            Self::Ui => 60,
            Self::Custom(_) => 1_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderPass {
    pub kind: RenderPassKind,
    pub order: i32,
    pub target: RenderTarget,
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
