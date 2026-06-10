//! Draw-plan mirror of upstream `mindustry.ui.GridImage`.

use crate::mindustry::entities::entity_group::Rect;

const GRID_STROKE: f32 = 2.0;
const GRID_OFFSET: f32 = 1.0;
const MIN_SPACE: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridImageLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl GridImageLayout {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridImageLineDraw {
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridImageDrawPlan {
    pub image_width: i32,
    pub image_height: i32,
    pub lines: Vec<GridImageLineDraw>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridImage {
    image_width: i32,
    image_height: i32,
}

impl GridImage {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            image_width: width,
            image_height: height,
        }
    }

    pub fn set_image_size(&mut self, width: i32, height: i32) {
        self.image_width = width;
        self.image_height = height;
    }

    pub fn image_width(&self) -> i32 {
        self.image_width
    }

    pub fn image_height(&self) -> i32 {
        self.image_height
    }

    pub fn draw_plan(&self, layout: GridImageLayout) -> GridImageDrawPlan {
        let xspace = layout.width / self.image_width as f32;
        let yspace = layout.height / self.image_height as f32;
        let jumpx = (xspace.max(MIN_SPACE) / xspace) as i32;
        let jumpy = (yspace.max(MIN_SPACE) / yspace) as i32;
        let mut lines = Vec::new();

        let mut x = 0;
        while x <= self.image_width {
            lines.push(GridImageLineDraw {
                rect: Rect::new(
                    (layout.x + xspace * x as f32 - GRID_OFFSET) as i32 as f32,
                    layout.y - GRID_OFFSET,
                    GRID_STROKE,
                    layout.height
                        + if x == self.image_width {
                            GRID_OFFSET
                        } else {
                            0.0
                        },
                ),
            });
            x += jumpx;
        }

        let mut y = 0;
        while y <= self.image_height {
            lines.push(GridImageLineDraw {
                rect: Rect::new(
                    layout.x - GRID_OFFSET,
                    (layout.y + y as f32 * yspace - GRID_OFFSET) as i32 as f32,
                    layout.width,
                    GRID_STROKE,
                ),
            });
            y += jumpy;
        }

        GridImageDrawPlan {
            image_width: self.image_width,
            image_height: self.image_height,
            lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_draw_plan_matches_java_spacing_and_integer_casts() {
        let grid = GridImage::new(4, 2);
        let plan = grid.draw_plan(GridImageLayout::new(10.0, 20.0, 40.0, 20.0));

        assert_eq!(plan.lines.len(), 8);
        assert_eq!(
            plan.lines[0],
            GridImageLineDraw {
                rect: Rect::new(9.0, 19.0, 2.0, 20.0)
            }
        );
        assert_eq!(
            plan.lines[4],
            GridImageLineDraw {
                rect: Rect::new(49.0, 19.0, 2.0, 21.0)
            }
        );
        assert_eq!(
            plan.lines[5],
            GridImageLineDraw {
                rect: Rect::new(9.0, 19.0, 40.0, 2.0)
            }
        );
    }

    #[test]
    fn set_image_size_updates_source_dimensions() {
        let mut grid = GridImage::new(8, 8);
        grid.set_image_size(16, 12);

        assert_eq!(grid.image_width(), 16);
        assert_eq!(grid.image_height(), 12);
    }
}
