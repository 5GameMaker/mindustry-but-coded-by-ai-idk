//! Hitbox component mirroring upstream `mindustry.entities.comp.HitboxComp`.

use crate::mindustry::entities::{EntityPosition, SizedEntity};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HitboxRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl HitboxRect {
    pub fn set_centered(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.x = x - width / 2.0;
        self.y = y - height / 2.0;
        self.width = width;
        self.height = height;
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HitboxComp {
    pub x: f32,
    pub y: f32,
    pub last_x: f32,
    pub last_y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub hit_size: f32,
}

impl HitboxComp {
    pub const fn new(x: f32, y: f32, hit_size: f32) -> Self {
        Self {
            x,
            y,
            last_x: 0.0,
            last_y: 0.0,
            delta_x: 0.0,
            delta_y: 0.0,
            hit_size,
        }
    }

    pub fn update(&mut self) {}

    pub fn add(&mut self) {
        self.update_last_position();
    }

    pub fn after_read(&mut self) {
        self.update_last_position();
    }

    pub fn get_collisions<F>(&self, _consumer: F)
    where
        F: FnMut(),
    {
    }

    pub fn update_last_position(&mut self) {
        self.delta_x = self.x - self.last_x;
        self.delta_y = self.y - self.last_y;
        self.last_x = self.x;
        self.last_y = self.y;
    }

    pub fn collision(&mut self, _other: &HitboxComp, _x: f32, _y: f32) {}

    pub fn delta_len(&self) -> f32 {
        (self.delta_x * self.delta_x + self.delta_y * self.delta_y).sqrt()
    }

    pub fn delta_angle(&self) -> f32 {
        self.delta_y
            .atan2(self.delta_x)
            .to_degrees()
            .rem_euclid(360.0)
    }

    pub fn collides(&self, _other: &HitboxComp) -> bool {
        true
    }

    pub fn hitbox(&self, rect: &mut HitboxRect) {
        rect.set_centered(self.x, self.y, self.hit_size, self.hit_size);
    }

    pub fn hitbox_tile(&self, rect: &mut HitboxRect) {
        let size = (self.hit_size * 0.66).min(7.8);
        rect.set_centered(self.x, self.y, size, size);
    }
}

impl EntityPosition for HitboxComp {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl SizedEntity for HitboxComp {
    fn hit_size(&self) -> f32 {
        self.hit_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hitbox_component_tracks_last_position_and_delta_metrics() {
        let mut hitbox = HitboxComp::new(3.0, 4.0, 10.0);

        hitbox.add();
        assert_eq!((hitbox.last_x, hitbox.last_y), (3.0, 4.0));
        assert_eq!(hitbox.delta_len(), 5.0);
        assert!((hitbox.delta_angle() - 53.130104).abs() < 0.0001);

        hitbox.x = 3.0;
        hitbox.y = 8.0;
        hitbox.update_last_position();
        assert_eq!((hitbox.delta_x, hitbox.delta_y), (0.0, 4.0));
        assert_eq!(hitbox.delta_angle(), 90.0);
    }

    #[test]
    fn hitbox_component_sets_entity_and_tile_rectangles_like_java() {
        let hitbox = HitboxComp::new(10.0, 20.0, 12.0);
        let mut rect = HitboxRect::default();

        hitbox.hitbox(&mut rect);
        assert_eq!(
            rect,
            HitboxRect {
                x: 4.0,
                y: 14.0,
                width: 12.0,
                height: 12.0,
            }
        );

        hitbox.hitbox_tile(&mut rect);
        assert_eq!(
            rect,
            HitboxRect {
                x: 6.1,
                y: 16.1,
                width: 7.8,
                height: 7.8,
            }
        );
    }

    #[test]
    fn hitbox_component_default_collision_hook_allows_collision() {
        let hitbox = HitboxComp::new(0.0, 0.0, 4.0);
        let other = HitboxComp::new(1.0, 1.0, 4.0);

        assert!(hitbox.collides(&other));
        assert_eq!(hitbox.hit_size(), 4.0);
    }
}
