//! Stateful and draw-plan based mirror of upstream `mindustry.ui.Bar`.

use crate::mindustry::entities::entity_group::Rect;

const BAR_BACKGROUND_SHADE: f32 = 0.1;
const DEFAULT_TEXT_COLOR_RGBA: u32 = 0xffff_ffff;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub parent_alpha: f32,
    pub label_width: f32,
    pub label_height: f32,
    pub top_min_width: f32,
}

impl BarLayout {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            parent_alpha: 1.0,
            label_width: 0.0,
            label_height: 0.0,
            top_min_width: 0.0,
        }
    }
}

impl Default for BarLayout {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarFrameState {
    pub raw_fraction: f32,
    pub computed_fraction: f32,
    pub value: f32,
    pub last_value: f32,
    pub blink: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarOutlineDraw {
    pub rect: Rect,
    pub color_rgba: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarBackgroundDraw {
    pub rect: Rect,
    pub shade: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarFillDraw {
    pub draw_rect: Rect,
    pub visible_width: f32,
    pub clip_rect: Option<Rect>,
    pub tint_rgba: u32,
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarTextDraw {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color_rgba: u32,
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BarDrawCommand {
    Outline(BarOutlineDraw),
    Background(BarBackgroundDraw),
    Fill(BarFillDraw),
    Text(BarTextDraw),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarDrawPlan {
    pub state: BarFrameState,
    pub commands: Vec<BarDrawCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub name: String,
    pub fraction: Option<f32>,
    pub base_color_rgba: u32,
    pub blink_color_rgba: u32,
    pub outline_color_rgba: Option<u32>,
    pub outline_stroke: f32,
    value: f32,
    last_value: f32,
    blink: f32,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            name: String::new(),
            fraction: None,
            base_color_rgba: DEFAULT_TEXT_COLOR_RGBA,
            blink_color_rgba: DEFAULT_TEXT_COLOR_RGBA,
            outline_color_rgba: None,
            outline_stroke: 0.0,
            value: 0.0,
            last_value: 0.0,
            blink: 0.0,
        }
    }
}

impl Bar {
    pub fn new(name: impl Into<String>, color_rgba: u32, fraction: f32) -> Self {
        Self {
            name: name.into(),
            fraction: Some(fraction),
            base_color_rgba: color_rgba,
            blink_color_rgba: color_rgba,
            outline_color_rgba: None,
            outline_stroke: 0.0,
            value: fraction,
            last_value: fraction,
            blink: 0.0,
        }
    }

    pub fn new_clamped(name: impl Into<String>, color_rgba: u32, fraction: f32) -> Self {
        let fraction = sanitize_nonfinite(clamp01_keep_nan(fraction));
        Self {
            name: name.into(),
            fraction: Some(fraction),
            base_color_rgba: color_rgba,
            blink_color_rgba: color_rgba,
            outline_color_rgba: None,
            outline_stroke: 0.0,
            value: fraction,
            last_value: fraction,
            blink: 0.0,
        }
    }

    pub fn set(&mut self, name: impl Into<String>, fraction: f32, color_rgba: u32) {
        self.name = name.into();
        self.fraction = Some(fraction);
        self.base_color_rgba = color_rgba;
        self.blink_color_rgba = color_rgba;
        self.value = fraction;
        self.last_value = fraction;
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn set_fraction(&mut self, fraction: f32) {
        self.fraction = Some(fraction);
    }

    pub fn set_color(&mut self, color_rgba: u32) {
        self.base_color_rgba = color_rgba;
        self.blink_color_rgba = color_rgba;
    }

    pub fn set_blink_color(&mut self, color_rgba: u32) {
        self.blink_color_rgba = color_rgba;
    }

    pub fn reset(&mut self, value: f32) {
        self.value = value;
        self.last_value = value;
        self.blink = value;
    }

    pub fn snap(&mut self) {
        if let Some(fraction) = self.fraction {
            self.value = fraction;
            self.last_value = fraction;
        }
    }

    pub fn outline(&mut self, color_rgba: u32, stroke: f32) -> &mut Self {
        self.outline_color_rgba = Some(color_rgba);
        self.outline_stroke = stroke.max(0.0);
        self
    }

    pub fn flash(&mut self) {
        self.blink = 1.0;
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn last_value(&self) -> f32 {
        self.last_value
    }

    pub fn blink_amount(&self) -> f32 {
        self.blink
    }

    pub fn draw_plan(&mut self, layout: BarLayout) -> Option<BarDrawPlan> {
        let raw_fraction = self.fraction?;
        let mut computed_fraction = clamp01_keep_nan(raw_fraction);

        if self.last_value > computed_fraction {
            self.blink = 1.0;
            self.last_value = computed_fraction;
        }

        if self.last_value.is_nan() || self.last_value.is_infinite() {
            self.last_value = if self.last_value.is_nan() { 0.0 } else { 1.0 };
        }
        if self.value.is_nan() || self.value.is_infinite() {
            self.value = if self.value.is_nan() { 0.0 } else { 1.0 };
        }
        computed_fraction = sanitize_nonfinite(computed_fraction);

        self.blink = lerp(self.blink, 0.0, 0.2);
        self.value = lerp(self.value, computed_fraction, 0.15);

        let mut commands = Vec::with_capacity(4);

        if let Some(color_rgba) = self.outline_color_rgba {
            if self.outline_stroke > 0.0 {
                let outline_rect = Rect::new(
                    layout.x - self.outline_stroke,
                    layout.y - self.outline_stroke,
                    layout.width + self.outline_stroke * 2.0,
                    layout.height + self.outline_stroke * 2.0,
                );
                commands.push(BarDrawCommand::Outline(BarOutlineDraw {
                    rect: outline_rect,
                    color_rgba,
                }));
            }
        }

        commands.push(BarDrawCommand::Background(BarBackgroundDraw {
            rect: Rect::new(layout.x, layout.y, layout.width, layout.height),
            shade: BAR_BACKGROUND_SHADE,
            alpha: layout.parent_alpha,
        }));

        let visible_width = layout.width * self.value;
        let tint_rgba = blend_rgba(self.base_color_rgba, self.blink_color_rgba, self.blink);
        let (draw_width, clip_rect) = if visible_width > layout.top_min_width {
            (visible_width, None)
        } else {
            (
                layout.top_min_width,
                Some(Rect::new(layout.x, layout.y, visible_width, layout.height)),
            )
        };

        commands.push(BarDrawCommand::Fill(BarFillDraw {
            draw_rect: Rect::new(layout.x, layout.y, draw_width, layout.height),
            visible_width,
            clip_rect,
            tint_rgba,
            alpha: layout.parent_alpha,
        }));

        let label_x = layout.x + layout.width / 2.0 - layout.label_width / 2.0;
        let label_y = layout.y + layout.height / 2.0 + layout.label_height / 2.0 + 1.0;
        commands.push(BarDrawCommand::Text(BarTextDraw {
            text: self.name.clone(),
            x: label_x,
            y: label_y,
            color_rgba: DEFAULT_TEXT_COLOR_RGBA,
            alpha: layout.parent_alpha,
        }));

        Some(BarDrawPlan {
            state: BarFrameState {
                raw_fraction,
                computed_fraction,
                value: self.value,
                last_value: self.last_value,
                blink: self.blink,
            },
            commands,
        })
    }
}

fn clamp01_keep_nan(value: f32) -> f32 {
    if value.is_nan() {
        f32::NAN
    } else if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

fn sanitize_nonfinite(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else if value.is_infinite() {
        1.0
    } else {
        value
    }
}

fn lerp(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn blend_rgba(from: u32, to: u32, mix: f32) -> u32 {
    let mix = if mix.is_nan() {
        0.0
    } else {
        mix.clamp(0.0, 1.0)
    };

    let (fr, fg, fb, fa) = rgba_components(from);
    let (tr, tg, tb, ta) = rgba_components(to);

    pack_rgba(
        lerp(fr, tr, mix).round() as u8,
        lerp(fg, tg, mix).round() as u8,
        lerp(fb, tb, mix).round() as u8,
        lerp(fa, ta, mix).round() as u8,
    )
}

fn rgba_components(rgba: u32) -> (f32, f32, f32, f32) {
    (
        ((rgba >> 24) & 0xff) as f32,
        ((rgba >> 16) & 0xff) as f32,
        ((rgba >> 8) & 0xff) as f32,
        (rgba & 0xff) as f32,
    )
}

fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> BarLayout {
        let mut layout = BarLayout::new(10.0, 20.0, 100.0, 40.0);
        layout.parent_alpha = 0.5;
        layout.label_width = 30.0;
        layout.label_height = 12.0;
        layout.top_min_width = 16.0;
        layout
    }

    #[test]
    fn raw_constructor_keeps_initial_value_like_the_java_string_ctor() {
        let bar = Bar::new("power", 0x1020_30ff, 1.25);

        assert_eq!(bar.value(), 1.25);
        assert_eq!(bar.last_value(), 1.25);
        assert_eq!(bar.fraction, Some(1.25));
    }

    #[test]
    fn clamped_constructor_matches_the_java_provider_ctor_shape() {
        let bar = Bar::new_clamped("power", 0x1020_30ff, 1.25);

        assert_eq!(bar.value(), 1.0);
        assert_eq!(bar.last_value(), 1.0);
        assert_eq!(bar.fraction, Some(1.0));
    }

    #[test]
    fn draw_plan_clamps_and_blinks_on_a_drop_then_centers_text() {
        let mut bar = Bar::new_clamped("loading", 0x1020_30ff, 1.0);
        bar.set_blink_color(0xff00_00ff);
        bar.outline(0x0000_ffff, 2.0);
        bar.reset(1.0);
        bar.set_fraction(0.25);

        let plan = bar.draw_plan(layout()).expect("bar draw plan");

        assert_eq!(
            plan.state,
            BarFrameState {
                raw_fraction: 0.25,
                computed_fraction: 0.25,
                value: 0.8875,
                last_value: 0.25,
                blink: 0.8,
            }
        );

        assert_eq!(plan.commands.len(), 4);
        match &plan.commands[0] {
            BarDrawCommand::Outline(draw) => {
                assert_eq!(draw.rect, Rect::new(8.0, 18.0, 104.0, 44.0));
                assert_eq!(draw.color_rgba, 0x0000_ffff);
            }
            other => panic!("expected outline command, got {:?}", other),
        }
        match &plan.commands[2] {
            BarDrawCommand::Fill(draw) => {
                assert_eq!(draw.visible_width, 88.75);
                assert_eq!(draw.draw_rect, Rect::new(10.0, 20.0, 88.75, 40.0));
                assert!(draw.clip_rect.is_none());
                assert_eq!(draw.alpha, 0.5);
                assert_eq!(draw.tint_rgba, blend_rgba(0x1020_30ff, 0xff00_00ff, 0.8));
            }
            other => panic!("expected fill command, got {:?}", other),
        }
        match &plan.commands[3] {
            BarDrawCommand::Text(draw) => {
                assert_eq!(draw.text, "loading");
                assert_eq!(draw.x, 45.0);
                assert_eq!(draw.y, 47.0);
            }
            other => panic!("expected text command, got {:?}", other),
        }
    }

    #[test]
    fn draw_plan_uses_scissor_style_clip_when_fill_is_small() {
        let mut bar = Bar::new_clamped("loading", 0xff00_00ff, 0.0);
        bar.reset(0.0);

        let mut layout = layout();
        layout.top_min_width = 20.0;

        let plan = bar.draw_plan(layout).expect("bar draw plan");

        match &plan.commands[1] {
            BarDrawCommand::Fill(draw) => {
                assert_eq!(draw.visible_width, 0.0);
                assert_eq!(draw.draw_rect, Rect::new(10.0, 20.0, 20.0, 40.0));
                assert_eq!(draw.clip_rect, Some(Rect::new(10.0, 20.0, 0.0, 40.0)));
            }
            other => panic!("expected fill command, got {:?}", other),
        }
    }

    #[test]
    fn draw_plan_sanitizes_nan_and_infinite_state() {
        let mut bar = Bar::new_clamped("loading", 0x1020_30ff, 1.0);
        bar.set_fraction(f32::NAN);

        let plan = bar.draw_plan(BarLayout::default()).expect("bar draw plan");

        assert_eq!(plan.state.computed_fraction, 0.0);
        assert_eq!(plan.state.last_value, 1.0);
        assert_eq!(plan.state.value, 0.85);
        assert_eq!(plan.state.blink, 0.0);
    }
}
