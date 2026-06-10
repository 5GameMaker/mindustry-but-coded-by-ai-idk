//! Requirement image state and invalid-overlay draw plan for upstream `mindustry.ui.ReqImage`.

use crate::mindustry::{
    entities::{comp::DecalColor, entity_group::Rect},
    graphics::Pal,
};

const INVALID_STROKE: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReqImageLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ReqImageLayout {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn rect(self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReqImageInvalidLineDraw {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub stroke: f32,
    pub color: DecalColor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReqImageDrawPlan {
    pub image_region: String,
    pub image_rect: Rect,
    pub invalid_lines: Vec<ReqImageInvalidLineDraw>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqImage {
    image_region: String,
    valid: bool,
    pub visible: bool,
}

impl ReqImage {
    pub fn new(image_region: impl Into<String>, valid: bool) -> Self {
        Self {
            image_region: image_region.into(),
            valid,
            visible: true,
        }
    }

    pub fn from_region(region: impl Into<String>, valid: bool) -> Self {
        Self::new(region, valid)
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    pub fn set_valid(&mut self, valid: bool) {
        self.valid = valid;
    }

    pub fn image_region(&self) -> &str {
        &self.image_region
    }

    pub fn draw_plan(&self, layout: ReqImageLayout) -> ReqImageDrawPlan {
        let invalid_lines = if self.valid {
            Vec::new()
        } else {
            vec![
                ReqImageInvalidLineDraw {
                    x1: layout.x,
                    y1: layout.y - 2.0 + layout.height,
                    x2: layout.x + layout.width,
                    y2: layout.y - 2.0,
                    stroke: INVALID_STROKE,
                    color: Pal::REMOVE_BACK,
                },
                ReqImageInvalidLineDraw {
                    x1: layout.x,
                    y1: layout.y + layout.height,
                    x2: layout.x + layout.width,
                    y2: layout.y,
                    stroke: INVALID_STROKE,
                    color: Pal::REMOVE,
                },
            ]
        };

        ReqImageDrawPlan {
            image_region: self.image_region.clone(),
            image_rect: layout.rect(),
            invalid_lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_image_has_no_invalid_overlay() {
        let image = ReqImage::new("copper", true);
        let plan = image.draw_plan(ReqImageLayout::new(1.0, 2.0, 16.0, 16.0));

        assert!(image.valid());
        assert!(plan.invalid_lines.is_empty());
        assert_eq!(plan.image_region, "copper");
    }

    #[test]
    fn invalid_image_draws_two_java_diagonal_lines() {
        let image = ReqImage::new("lead", false);
        let plan = image.draw_plan(ReqImageLayout::new(1.0, 2.0, 16.0, 16.0));

        assert_eq!(plan.invalid_lines.len(), 2);
        assert_eq!(
            plan.invalid_lines[0],
            ReqImageInvalidLineDraw {
                x1: 1.0,
                y1: 16.0,
                x2: 17.0,
                y2: 0.0,
                stroke: 2.0,
                color: Pal::REMOVE_BACK,
            }
        );
        assert_eq!(plan.invalid_lines[1].color, Pal::REMOVE);
    }
}
