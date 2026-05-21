//! Velocity component mirroring upstream `mindustry.entities.comp.VelComp`.

use crate::mindustry::entities::comp::PosComp;
use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VelComp {
    pub pos: PosComp,
    pub vel: Vec2,
    pub drag: f32,
}

impl VelComp {
    pub const MOVING_EPSILON: f32 = 0.01;

    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            pos: PosComp::new(x, y),
            vel: Vec2 { x: 0.0, y: 0.0 },
            drag: 0.0,
        }
    }

    /// Java default `solidity()` returns null; callers can pass an explicit
    /// solid predicate to `can_pass_with` while collision movement is migrated.
    pub fn solidity(&self) -> Option<()> {
        None
    }

    pub fn ignore_solids(&self) -> bool {
        false
    }

    pub fn can_pass_with<F>(&self, tile_x: i32, tile_y: i32, solid: Option<F>) -> bool
    where
        F: Fn(i32, i32) -> bool,
    {
        solid.map(|check| !check(tile_x, tile_y)).unwrap_or(true)
    }

    pub fn can_pass_on_with<F>(&self, solid: Option<F>) -> bool
    where
        F: Fn(i32, i32) -> bool,
    {
        self.can_pass_with(self.pos.tile_x(), self.pos.tile_y(), solid)
    }

    pub fn moving(&self) -> bool {
        self.vel.x * self.vel.x + self.vel.y * self.vel.y
            > Self::MOVING_EPSILON * Self::MOVING_EPSILON
    }

    pub fn move_by(&mut self, cx: f32, cy: f32) {
        self.pos.trns(cx, cy);
    }

    pub fn move_vec(&mut self, v: Vec2) {
        self.move_by(v.x, v.y);
    }

    /// Java update body with explicit `delta`, `net.client()` and `isLocal()`.
    pub fn update(&mut self, delta: f32, net_client: bool, is_local: bool) {
        if !net_client || is_local {
            let px = self.pos.x;
            let py = self.pos.y;
            self.move_by(self.vel.x * delta, self.vel.y * delta);

            if nearly_equal(px, self.pos.x) {
                self.vel.x = 0.0;
            }
            if nearly_equal(py, self.pos.y) {
                self.vel.y = 0.0;
            }

            let scale = (1.0 - self.drag * delta).max(0.0);
            self.vel.x *= scale;
            self.vel.y *= scale;
        }
    }

    pub fn vel_add_net(&mut self, v: Vec2, is_remote: bool) {
        self.vel.x += v.x;
        self.vel.y += v.y;
        if is_remote {
            self.pos.trns(v.x, v.y);
        }
    }

    pub fn vel_add_net_xy(&mut self, vx: f32, vy: f32, is_remote: bool) {
        self.vel_add_net(Vec2 { x: vx, y: vy }, is_remote);
    }
}

fn nearly_equal(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.000001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_component_updates_position_and_applies_drag_when_not_remote_client() {
        let mut vel = VelComp::new(0.0, 0.0);
        vel.vel = Vec2 { x: 10.0, y: 0.0 };
        vel.drag = 0.1;

        vel.update(2.0, false, false);

        assert_eq!(vel.pos.x, 20.0);
        assert_eq!(vel.pos.y, 0.0);
        assert_eq!(vel.vel.x, 8.0);
        assert_eq!(vel.vel.y, 0.0);
        assert!(vel.moving());
    }

    #[test]
    fn velocity_component_skips_update_on_nonlocal_client_like_java() {
        let mut vel = VelComp::new(0.0, 0.0);
        vel.vel = Vec2 { x: 10.0, y: 1.0 };
        vel.drag = 1.0;

        vel.update(1.0, true, false);

        assert_eq!(vel.pos, PosComp::new(0.0, 0.0));
        assert_eq!(vel.vel, Vec2 { x: 10.0, y: 1.0 });
    }

    #[test]
    fn velocity_component_can_pass_and_remote_velocity_adjustment_follow_java_shape() {
        let mut vel = VelComp::new(8.0, 8.0);

        assert!(vel.can_pass_with(1, 1, Option::<fn(i32, i32) -> bool>::None));
        assert!(!vel.can_pass_with(1, 1, Some(|x, y| x == 1 && y == 1)));

        vel.vel_add_net_xy(2.0, 3.0, false);
        assert_eq!(vel.vel, Vec2 { x: 2.0, y: 3.0 });
        assert_eq!(vel.pos, PosComp::new(8.0, 8.0));

        vel.vel_add_net(Vec2 { x: -1.0, y: 4.0 }, true);
        assert_eq!(vel.vel, Vec2 { x: 1.0, y: 7.0 });
        assert_eq!(vel.pos, PosComp::new(7.0, 12.0));
    }
}
