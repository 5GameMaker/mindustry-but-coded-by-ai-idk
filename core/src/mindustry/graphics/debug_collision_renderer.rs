//! Debug collision overlay plan mirroring upstream `mindustry.graphics.DebugCollisionRenderer`.

use super::{RenderPoint, RenderRect};

pub const DEBUG_COLLISION_EDGES: [f32; 8] = [1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0];
pub const DEBUG_COLLISION_LAYER: &str = "Layer.overlayUI";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugCollisionColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl DebugCollisionColor {
    pub const GREEN_HITBOX: Self = Self::new(0.0, 1.0, 0.0, 0.3);
    pub const MAGENTA_TILE: Self = Self::new(1.0, 0.0, 1.0, 1.0);
    pub const CYAN_AVOIDANCE: Self = Self::new(0.0, 1.0, 1.0, 0.25);
    pub const RED_PHYSICS: Self = Self::new(1.0, 0.0, 0.0, 0.5);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugCollisionCamera {
    pub center: RenderPoint,
    pub width: f32,
    pub height: f32,
}

impl DebugCollisionCamera {
    pub const fn new(center: RenderPoint, width: f32, height: f32) -> Self {
        Self {
            center,
            width,
            height,
        }
    }

    pub fn bounds(self) -> RenderRect {
        RenderRect::from_center(self.center, self.width, self.height)
    }

    pub fn tile_center(self, tile_size: f32) -> (i32, i32) {
        (
            (self.center.x / tile_size).floor() as i32,
            (self.center.y / tile_size).floor() as i32,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugCollisionTile {
    pub x: i32,
    pub y: i32,
    pub solid: bool,
    /// Neighbor solidity in Java nearby order 0..4.
    pub nearby_solid: [bool; 4],
    pub avoidance: i32,
}

impl DebugCollisionTile {
    pub const fn new(x: i32, y: i32, solid: bool, nearby_solid: [bool; 4]) -> Self {
        Self {
            x,
            y,
            solid,
            nearby_solid,
            avoidance: 0,
        }
    }

    pub const fn with_avoidance(mut self, avoidance: i32) -> Self {
        self.avoidance = avoidance;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugCollisionDrawableKind {
    Hitbox,
    Unit {
        flying: bool,
        tile_hitbox: RenderRect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugCollisionDrawable {
    pub x: f32,
    pub y: f32,
    pub clip_size: f32,
    pub hit_size: f32,
    pub kind: DebugCollisionDrawableKind,
}

impl DebugCollisionDrawable {
    pub const fn hitbox(x: f32, y: f32, clip_size: f32, hit_size: f32) -> Self {
        Self {
            x,
            y,
            clip_size,
            hit_size,
            kind: DebugCollisionDrawableKind::Hitbox,
        }
    }

    pub const fn unit(
        x: f32,
        y: f32,
        clip_size: f32,
        hit_size: f32,
        flying: bool,
        tile_hitbox: RenderRect,
    ) -> Self {
        Self {
            x,
            y,
            clip_size,
            hit_size,
            kind: DebugCollisionDrawableKind::Unit {
                flying,
                tile_hitbox,
            },
        }
    }

    fn clip_rect(self) -> RenderRect {
        RenderRect::from_center(
            RenderPoint::new(self.x, self.y),
            self.clip_size,
            self.clip_size,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebugCollisionInput {
    pub camera: DebugCollisionCamera,
    pub world_width: i32,
    pub world_height: i32,
    pub tile_size: f32,
    pub unit_collision_radius_scale: f32,
    pub debug_draw_avoidance: bool,
    pub tiles: Vec<DebugCollisionTile>,
    pub drawables: Vec<DebugCollisionDrawable>,
}

impl DebugCollisionInput {
    pub fn new(camera: DebugCollisionCamera, world_width: i32, world_height: i32) -> Self {
        Self {
            camera,
            world_width,
            world_height,
            tile_size: 8.0,
            unit_collision_radius_scale: 1.0,
            debug_draw_avoidance: false,
            tiles: Vec::new(),
            drawables: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DebugCollisionCommand {
    FillSquare {
        center: RenderPoint,
        radius: f32,
        color: DebugCollisionColor,
    },
    Line {
        from: RenderPoint,
        to: RenderPoint,
        stroke: f32,
        color: DebugCollisionColor,
    },
    Rect {
        rect: RenderRect,
        stroke: f32,
        color: DebugCollisionColor,
    },
    Circle {
        center: RenderPoint,
        radius: f32,
        stroke: f32,
        color: DebugCollisionColor,
    },
    Reset,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebugCollisionPlan {
    pub layer: &'static str,
    pub commands: Vec<DebugCollisionCommand>,
}

pub struct DebugCollisionRenderer;

impl DebugCollisionRenderer {
    pub fn draw_plan(input: &DebugCollisionInput) -> DebugCollisionPlan {
        let camera_bounds = input.camera.bounds();
        let mut commands = Vec::new();

        for drawable in &input.drawables {
            if camera_bounds.intersects(drawable.clip_rect()) {
                commands.push(DebugCollisionCommand::FillSquare {
                    center: RenderPoint::new(drawable.x, drawable.y),
                    radius: drawable.hit_size / 2.0,
                    color: DebugCollisionColor::GREEN_HITBOX,
                });
            }
        }

        let rx = ((input.camera.width / input.tile_size / 2.0) as i32 + 1)
            .clamp(0, input.world_width / 2);
        let ry = ((input.camera.height / input.tile_size / 2.0) as i32 + 1)
            .clamp(0, input.world_height / 2);
        let (center_x, center_y) = input.camera.tile_center(input.tile_size);

        for tile in &input.tiles {
            if tile.x < center_x - rx
                || tile.x > center_x + rx
                || tile.y < center_y - ry
                || tile.y > center_y + ry
            {
                continue;
            }

            if tile.solid {
                for i in 0..4 {
                    if !tile.nearby_solid[i] {
                        commands.push(DebugCollisionCommand::Line {
                            from: tile_edge_point(tile.x, tile.y, i, input.tile_size),
                            to: tile_edge_point(tile.x, tile.y, (i + 1) % 4, input.tile_size),
                            stroke: 0.4,
                            color: DebugCollisionColor::MAGENTA_TILE,
                        });
                    }
                }
            }

            if input.debug_draw_avoidance && tile.avoidance != 0 {
                commands.push(DebugCollisionCommand::FillSquare {
                    center: RenderPoint::new(
                        tile.x as f32 * input.tile_size,
                        tile.y as f32 * input.tile_size,
                    ),
                    radius: 4.0,
                    color: DebugCollisionColor::CYAN_AVOIDANCE,
                });
            }
        }

        for drawable in &input.drawables {
            if let DebugCollisionDrawableKind::Unit {
                flying,
                tile_hitbox,
            } = drawable.kind
            {
                if !flying && camera_bounds.intersects(drawable.clip_rect()) {
                    commands.push(DebugCollisionCommand::Rect {
                        rect: tile_hitbox,
                        stroke: 0.4,
                        color: DebugCollisionColor::MAGENTA_TILE,
                    });
                }
            }
        }

        for drawable in &input.drawables {
            if matches!(drawable.kind, DebugCollisionDrawableKind::Unit { .. })
                && camera_bounds.intersects(drawable.clip_rect())
            {
                commands.push(DebugCollisionCommand::Circle {
                    center: RenderPoint::new(drawable.x, drawable.y),
                    radius: drawable.hit_size * input.unit_collision_radius_scale,
                    stroke: 0.5,
                    color: DebugCollisionColor::RED_PHYSICS,
                });
            }
        }

        commands.push(DebugCollisionCommand::Reset);
        DebugCollisionPlan {
            layer: DEBUG_COLLISION_LAYER,
            commands,
        }
    }
}

fn tile_edge_point(x: i32, y: i32, edge: usize, tile_size: f32) -> RenderPoint {
    RenderPoint::new(
        x as f32 * tile_size + DEBUG_COLLISION_EDGES[edge * 2] * tile_size / 2.0,
        y as f32 * tile_size + DEBUG_COLLISION_EDGES[edge * 2 + 1] * tile_size / 2.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> DebugCollisionCamera {
        DebugCollisionCamera::new(RenderPoint::new(0.0, 0.0), 64.0, 64.0)
    }

    #[test]
    fn debug_collision_edges_match_java_static_array() {
        assert_eq!(
            DEBUG_COLLISION_EDGES,
            [1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0]
        );
    }

    #[test]
    fn debug_collision_plan_draws_hitbox_tile_edges_avoidance_and_reset() {
        let mut input = DebugCollisionInput::new(camera(), 20, 20);
        input.debug_draw_avoidance = true;
        input.tiles.push(
            DebugCollisionTile::new(0, 0, true, [false, true, false, true]).with_avoidance(1),
        );
        input
            .drawables
            .push(DebugCollisionDrawable::hitbox(0.0, 0.0, 16.0, 10.0));

        let plan = DebugCollisionRenderer::draw_plan(&input);

        assert_eq!(plan.layer, DEBUG_COLLISION_LAYER);
        assert!(plan.commands.contains(&DebugCollisionCommand::FillSquare {
            center: RenderPoint::new(0.0, 0.0),
            radius: 5.0,
            color: DebugCollisionColor::GREEN_HITBOX,
        }));
        assert_eq!(
            plan.commands
                .iter()
                .filter(|command| matches!(command, DebugCollisionCommand::Line { .. }))
                .count(),
            2
        );
        assert!(plan.commands.contains(&DebugCollisionCommand::FillSquare {
            center: RenderPoint::new(0.0, 0.0),
            radius: 4.0,
            color: DebugCollisionColor::CYAN_AVOIDANCE,
        }));
        assert_eq!(plan.commands.last(), Some(&DebugCollisionCommand::Reset));
    }

    #[test]
    fn debug_collision_plan_draws_ground_unit_rect_and_all_unit_physics_circles() {
        let mut input = DebugCollisionInput::new(camera(), 20, 20);
        input.unit_collision_radius_scale = 1.5;
        input.drawables.push(DebugCollisionDrawable::unit(
            0.0,
            0.0,
            16.0,
            6.0,
            false,
            RenderRect::new(-4.0, -4.0, 8.0, 8.0),
        ));
        input.drawables.push(DebugCollisionDrawable::unit(
            10.0,
            0.0,
            16.0,
            4.0,
            true,
            RenderRect::new(8.0, -2.0, 4.0, 4.0),
        ));

        let plan = DebugCollisionRenderer::draw_plan(&input);

        assert_eq!(
            plan.commands
                .iter()
                .filter(|command| matches!(command, DebugCollisionCommand::Rect { .. }))
                .count(),
            1
        );
        assert_eq!(
            plan.commands
                .iter()
                .filter(|command| matches!(command, DebugCollisionCommand::Circle { .. }))
                .count(),
            2
        );
        assert!(plan.commands.contains(&DebugCollisionCommand::Circle {
            center: RenderPoint::new(0.0, 0.0),
            radius: 9.0,
            stroke: 0.5,
            color: DebugCollisionColor::RED_PHYSICS,
        }));
    }
}
