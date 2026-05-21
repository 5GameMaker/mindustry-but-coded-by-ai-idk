//! Physics component mirroring upstream `mindustry.entities.comp.PhysicsComp`.

use std::f32::consts::PI;

use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicRef {
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsComp {
    pub hit_size: f32,
    pub x: f32,
    pub y: f32,
    pub vel: Vec2,
    pub physref: Option<PhysicRef>,
}

impl PhysicsComp {
    pub const fn new(hit_size: f32, x: f32, y: f32) -> Self {
        Self {
            hit_size,
            x,
            y,
            vel: Vec2 { x: 0.0, y: 0.0 },
            physref: None,
        }
    }

    /// Java mass is simply circular area: `hitSize * hitSize * Mathf.pi`.
    pub fn mass(&self) -> f32 {
        self.hit_size * self.hit_size * PI
    }

    pub fn impulse(&mut self, x: f32, y: f32) {
        let mass = self.mass();
        self.vel.x += x / mass;
        self.vel.y += y / mass;
    }

    pub fn impulse_vec(&mut self, v: Vec2) {
        self.impulse(v.x, v.y);
    }

    pub fn move_by(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    pub fn impulse_net(&mut self, v: Vec2, is_remote: bool) {
        self.impulse_vec(v);

        if is_remote {
            let mass = self.mass();
            self.move_by(v.x / mass, v.y / mass);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_component_mass_is_hit_area_like_java() {
        let physics = PhysicsComp::new(2.0, 0.0, 0.0);

        assert!((physics.mass() - 4.0 * PI).abs() < 0.000001);
    }

    #[test]
    fn physics_component_impulse_adds_velocity_by_mass() {
        let mut physics = PhysicsComp::new(1.0, 0.0, 0.0);

        physics.impulse(PI, PI * 2.0);

        assert!((physics.vel.x - 1.0).abs() < 0.000001);
        assert!((physics.vel.y - 2.0).abs() < 0.000001);
    }

    #[test]
    fn physics_component_impulse_net_moves_remote_entities_only() {
        let mut local = PhysicsComp::new(1.0, 0.0, 0.0);
        local.impulse_net(Vec2 { x: PI, y: 0.0 }, false);
        assert_eq!((local.x, local.y), (0.0, 0.0));
        assert!((local.vel.x - 1.0).abs() < 0.000001);

        let mut remote = PhysicsComp::new(1.0, 0.0, 0.0);
        remote.impulse_net(Vec2 { x: PI, y: PI }, true);
        assert!((remote.x - 1.0).abs() < 0.000001);
        assert!((remote.y - 1.0).abs() < 0.000001);
    }
}
