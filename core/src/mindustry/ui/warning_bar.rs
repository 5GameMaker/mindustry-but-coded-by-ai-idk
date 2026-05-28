//! Data-only draw-plan mirror of upstream `mindustry.ui.WarningBar`.

use crate::mindustry::entities::entity_group::Rect;

pub const DEFAULT_WARNING_BAR_WIDTH: f32 = 40.0;
pub const DEFAULT_WARNING_STROKE: f32 = 3.0;
pub const DEFAULT_ACCENT_RGBA: u32 = 0xffd3_7fff;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarningBarLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub parent_alpha: f32,
}

impl WarningBarLayout {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            parent_alpha: 1.0,
        }
    }
}

impl Default for WarningBarLayout {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub x3: f32,
    pub y3: f32,
    pub x4: f32,
    pub y4: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub from_x: f32,
    pub from_y: f32,
    pub to_x: f32,
    pub to_y: f32,
    pub stroke: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarningBarStripeDraw {
    pub quad: Quad,
    pub color_rgba: u32,
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarningBarLineDraw {
    pub line: LineSegment,
    pub color_rgba: u32,
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WarningBarDrawCommand {
    Stripe(WarningBarStripeDraw),
    Line(WarningBarLineDraw),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarningBarDrawPlan {
    pub stripe_count: usize,
    pub commands: Vec<WarningBarDrawCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WarningBar {
    pub bar_width: f32,
    pub spacing: f32,
    pub skew: f32,
    pub color_rgba: u32,
    pub stroke: f32,
}

impl Default for WarningBar {
    fn default() -> Self {
        Self::new()
    }
}

impl WarningBar {
    pub fn new() -> Self {
        Self {
            bar_width: DEFAULT_WARNING_BAR_WIDTH,
            spacing: DEFAULT_WARNING_BAR_WIDTH * 2.0,
            skew: DEFAULT_WARNING_BAR_WIDTH,
            color_rgba: DEFAULT_ACCENT_RGBA,
            stroke: DEFAULT_WARNING_STROKE,
        }
    }

    pub fn draw_plan(&self, layout: WarningBarLayout) -> WarningBarDrawPlan {
        let spacing = sanitize_positive(self.spacing, DEFAULT_WARNING_BAR_WIDTH * 2.0);
        let amount = (layout.width / spacing) as i32 + 2;
        let stripe_count = amount.max(0) as usize;
        let mut commands = Vec::with_capacity(stripe_count + 2);

        for i in 0..stripe_count {
            let rx = layout.x + (i as f32 - 1.0) * spacing;
            commands.push(WarningBarDrawCommand::Stripe(WarningBarStripeDraw {
                quad: Quad {
                    x1: rx,
                    y1: layout.y,
                    x2: rx + self.skew,
                    y2: layout.y + layout.height,
                    x3: rx + self.skew + self.bar_width,
                    y3: layout.y + layout.height,
                    x4: rx + self.bar_width,
                    y4: layout.y,
                },
                color_rgba: self.color_rgba,
                alpha: layout.parent_alpha,
            }));
        }

        commands.push(WarningBarDrawCommand::Line(WarningBarLineDraw {
            line: LineSegment {
                from_x: layout.x,
                from_y: layout.y,
                to_x: layout.x + layout.width,
                to_y: layout.y,
                stroke: self.stroke,
            },
            color_rgba: self.color_rgba,
            alpha: layout.parent_alpha,
        }));
        commands.push(WarningBarDrawCommand::Line(WarningBarLineDraw {
            line: LineSegment {
                from_x: layout.x,
                from_y: layout.y + layout.height,
                to_x: layout.x + layout.width,
                to_y: layout.y + layout.height,
                stroke: self.stroke,
            },
            color_rgba: self.color_rgba,
            alpha: layout.parent_alpha,
        }));

        WarningBarDrawPlan {
            stripe_count,
            commands,
        }
    }

    pub fn bounds(layout: WarningBarLayout) -> Rect {
        Rect::new(layout.x, layout.y, layout.width, layout.height)
    }
}

fn sanitize_positive(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_java_initializer() {
        let warning = WarningBar::new();

        assert_eq!(warning.bar_width, 40.0);
        assert_eq!(warning.spacing, 80.0);
        assert_eq!(warning.skew, 40.0);
        assert_eq!(warning.stroke, 3.0);
    }

    #[test]
    fn draw_plan_matches_java_quad_and_border_layout() {
        let warning = WarningBar::new();
        let mut layout = WarningBarLayout::new(10.0, 20.0, 160.0, 30.0);
        layout.parent_alpha = 0.5;

        let plan = warning.draw_plan(layout);

        assert_eq!(plan.stripe_count, 4);
        assert_eq!(plan.commands.len(), 6);
        match &plan.commands[0] {
            WarningBarDrawCommand::Stripe(draw) => {
                assert_eq!(
                    draw.quad,
                    Quad {
                        x1: -70.0,
                        y1: 20.0,
                        x2: -30.0,
                        y2: 50.0,
                        x3: 10.0,
                        y3: 50.0,
                        x4: -30.0,
                        y4: 20.0,
                    }
                );
                assert_eq!(draw.alpha, 0.5);
            }
            other => panic!("expected stripe command, got {:?}", other),
        }

        match &plan.commands[4] {
            WarningBarDrawCommand::Line(draw) => {
                assert_eq!(draw.line.from_x, 10.0);
                assert_eq!(draw.line.from_y, 20.0);
                assert_eq!(draw.line.to_x, 170.0);
                assert_eq!(draw.line.to_y, 20.0);
                assert_eq!(draw.line.stroke, 3.0);
            }
            other => panic!("expected top border command, got {:?}", other),
        }
    }

    #[test]
    fn invalid_spacing_falls_back_to_java_default_spacing() {
        let mut warning = WarningBar::new();
        warning.spacing = f32::NAN;

        let plan = warning.draw_plan(WarningBarLayout::new(0.0, 0.0, 80.0, 10.0));

        assert_eq!(plan.stripe_count, 3);
    }
}
