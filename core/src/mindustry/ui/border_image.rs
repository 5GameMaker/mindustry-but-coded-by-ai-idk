//! Border image draw model mirroring upstream `mindustry.ui.BorderImage`.

use crate::mindustry::entities::entity_group::Rect;

pub const BORDER_IMAGE_DEFAULT_THICKNESS: f32 = 4.0;
pub const BORDER_IMAGE_DEFAULT_PAD: f32 = 0.0;
pub const BORDER_IMAGE_DEFAULT_BORDER_RGBA: u32 = 0x777777ff;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderImage {
    pub thickness: f32,
    pub pad: f32,
    pub border_color_rgba: u32,
}

impl Default for BorderImage {
    fn default() -> Self {
        Self::new()
    }
}

impl BorderImage {
    pub const fn new() -> Self {
        Self {
            thickness: BORDER_IMAGE_DEFAULT_THICKNESS,
            pad: BORDER_IMAGE_DEFAULT_PAD,
            border_color_rgba: BORDER_IMAGE_DEFAULT_BORDER_RGBA,
        }
    }

    pub const fn with_thickness(thickness: f32) -> Self {
        Self {
            thickness,
            ..Self::new()
        }
    }

    pub fn border(mut self, color_rgba: u32) -> Self {
        self.border_color_rgba = color_rgba;
        self
    }

    pub fn draw_plan(
        &self,
        image_x: f32,
        image_y: f32,
        image_width: f32,
        image_height: f32,
        scale_x: f32,
        scale_y: f32,
        parent_alpha: f32,
    ) -> BorderImageDrawPlan {
        BorderImageDrawPlan {
            stroke: self.thickness,
            color_rgba: self.border_color_rgba,
            alpha: parent_alpha,
            rect: Rect::new(
                image_x - self.pad,
                image_y - self.pad,
                image_width * scale_x + self.pad * 2.0,
                image_height * scale_y + self.pad * 2.0,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderImageDrawPlan {
    pub stroke: f32,
    pub color_rgba: u32,
    pub alpha: f32,
    pub rect: Rect,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_image_defaults_match_java_fields() {
        let image = BorderImage::new();
        assert_eq!(image.thickness, 4.0);
        assert_eq!(image.pad, 0.0);
        assert_eq!(image.border_color_rgba, BORDER_IMAGE_DEFAULT_BORDER_RGBA);
    }

    #[test]
    fn draw_plan_matches_java_lines_rect_formula() {
        let image = BorderImage {
            thickness: 3.0,
            pad: 2.0,
            border_color_rgba: 0xff00ffff,
        };

        let plan = image.draw_plan(10.0, 20.0, 30.0, 40.0, 2.0, 0.5, 0.75);

        assert_eq!(plan.stroke, 3.0);
        assert_eq!(plan.color_rgba, 0xff00ffff);
        assert_eq!(plan.alpha, 0.75);
        assert_eq!(plan.rect, Rect::new(8.0, 18.0, 64.0, 24.0));
    }
}
