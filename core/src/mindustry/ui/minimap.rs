//! Small HUD minimap widget model mirroring upstream `mindustry.ui.Minimap`.

use crate::mindustry::entities::entity_group::Rect;

pub const MINIMAP_WIDGET_SIZE: f32 = 140.0;
pub const MINIMAP_WIDGET_MARGIN: f32 = 5.0;
pub const MINIMAP_TAP_SQUARE_SIZE: f32 = 11.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapRegion {
    pub u: f32,
    pub v: f32,
    pub u2: f32,
    pub v2: f32,
}

impl MinimapRegion {
    pub const fn new(u: f32, v: f32, u2: f32, v2: f32) -> Self {
        Self { u, v, u2, v2 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapWidgetWorld {
    pub width_tiles: f32,
    pub height_tiles: f32,
    pub tile_size: f32,
}

impl MinimapWidgetWorld {
    pub const fn new(width_tiles: f32, height_tiles: f32, tile_size: f32) -> Self {
        Self {
            width_tiles,
            height_tiles,
            tile_size,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MinimapWidgetAction {
    PanCamera { x: f32, y: f32 },
    ZoomBy(f32),
    SetZoom(f32),
    ToggleFullMinimap,
    RequestScroll,
    ClearScrollFocus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapWidgetDrawPlan {
    pub background_pane: bool,
    pub margin: f32,
    pub size: f32,
    pub touchable: bool,
    pub draw_region: bool,
    pub draw_entities: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Minimap {
    pub margin: f32,
    pub size: f32,
    pub tap_square_size: f32,
}

impl Default for Minimap {
    fn default() -> Self {
        Self::new()
    }
}

impl Minimap {
    pub const fn new() -> Self {
        Self {
            margin: MINIMAP_WIDGET_MARGIN,
            size: MINIMAP_WIDGET_SIZE,
            tap_square_size: MINIMAP_TAP_SQUARE_SIZE,
        }
    }

    pub fn element_rect(&self, scl: f32) -> Rect {
        Rect::new(
            self.margin * scl,
            self.margin * scl,
            self.size * scl,
            self.size * scl,
        )
    }

    pub fn draw_plan(&self, region_present: bool, texture_present: bool) -> MinimapWidgetDrawPlan {
        MinimapWidgetDrawPlan {
            background_pane: true,
            margin: self.margin,
            size: self.size,
            touchable: true,
            draw_region: region_present,
            draw_entities: region_present && texture_present,
        }
    }

    pub fn right_click_pan(
        &self,
        click_x: f32,
        click_y: f32,
        element: Rect,
        region: Option<MinimapRegion>,
        world: MinimapWidgetWorld,
    ) -> Option<MinimapWidgetAction> {
        let region = region?;
        let sx = (click_x - element.x) / element.width;
        let sy = (click_y - element.y) / element.height;
        let scaled_x = lerp(region.u, region.u2, sx) * world.width_tiles * world.tile_size;
        let scaled_y =
            lerp(1.0 - region.v2, 1.0 - region.v, sy) * world.height_tiles * world.tile_size;
        Some(MinimapWidgetAction::PanCamera {
            x: scaled_x,
            y: scaled_y,
        })
    }

    pub fn scrolled(&self, amount_y: f32) -> MinimapWidgetAction {
        MinimapWidgetAction::ZoomBy(amount_y)
    }

    pub fn dragged(
        &self,
        local_y: f32,
        height: f32,
        mobile: bool,
        world: MinimapWidgetWorld,
    ) -> Option<MinimapWidgetAction> {
        if !mobile {
            return None;
        }

        let max = world.width_tiles.min(world.height_tiles) / 16.0 / 2.0;
        Some(MinimapWidgetAction::SetZoom(
            1.0 + local_y / height * (max - 1.0),
        ))
    }

    pub fn clicked(&self) -> MinimapWidgetAction {
        MinimapWidgetAction::ToggleFullMinimap
    }

    pub fn update_hover(
        &self,
        hover_is_descendant: bool,
        has_scroll: bool,
    ) -> Vec<MinimapWidgetAction> {
        if hover_is_descendant {
            vec![MinimapWidgetAction::RequestScroll]
        } else if has_scroll {
            vec![MinimapWidgetAction::ClearScrollFocus]
        } else {
            Vec::new()
        }
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_defaults_match_java_constructor_constants() {
        let minimap = Minimap::new();

        assert_eq!(minimap.margin, 5.0);
        assert_eq!(minimap.size, 140.0);
        assert_eq!(minimap.tap_square_size, 11.0);
        assert_eq!(
            minimap.element_rect(2.0),
            Rect::new(10.0, 10.0, 280.0, 280.0)
        );
    }

    #[test]
    fn right_click_pan_uses_region_uv_and_world_tile_scale() {
        let minimap = Minimap::new();
        let action = minimap.right_click_pan(
            80.0,
            45.0,
            Rect::new(10.0, 10.0, 140.0, 140.0),
            Some(MinimapRegion::new(0.2, 0.1, 0.7, 0.6)),
            MinimapWidgetWorld::new(100.0, 50.0, 8.0),
        );

        let Some(MinimapWidgetAction::PanCamera { x, y }) = action else {
            panic!("expected pan camera action");
        };
        assert_eq!(x, 360.0);
        assert!((y - 210.0).abs() < 0.001);
    }

    #[test]
    fn scroll_drag_click_and_hover_emit_java_widget_actions() {
        let minimap = Minimap::new();

        assert_eq!(minimap.scrolled(-1.0), MinimapWidgetAction::ZoomBy(-1.0));
        assert_eq!(minimap.clicked(), MinimapWidgetAction::ToggleFullMinimap);
        assert_eq!(
            minimap.dragged(70.0, 140.0, true, MinimapWidgetWorld::new(64.0, 64.0, 8.0)),
            Some(MinimapWidgetAction::SetZoom(1.5))
        );
        assert_eq!(
            minimap.update_hover(true, false),
            vec![MinimapWidgetAction::RequestScroll]
        );
        assert_eq!(
            minimap.update_hover(false, true),
            vec![MinimapWidgetAction::ClearScrollFocus]
        );
    }

    #[test]
    fn draw_plan_skips_region_when_renderer_has_no_minimap_region() {
        let minimap = Minimap::new();
        let plan = minimap.draw_plan(false, true);
        assert!(!plan.draw_region);
        assert!(!plan.draw_entities);

        let plan = minimap.draw_plan(true, true);
        assert!(plan.draw_region);
        assert!(plan.draw_entities);
    }
}
