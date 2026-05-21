//! Water movement component mirroring upstream `mindustry.entities.comp.WaterMoveComp`.

use crate::mindustry::core::world::World;
use crate::mindustry::entities::comp::WaterCrawlSolidPred;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrailState {
    pub length: i32,
    pub last_x: f32,
    pub last_y: f32,
    pub last_active: f32,
    pub cleared: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterMoveDrawPlan {
    pub left: TrailState,
    pub right: TrailState,
    pub trail_scl: f32,
    pub color_rgba: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterMoveComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub speed_multiplier: f32,
    pub flying: bool,
    pub ignore_solids: bool,
    pub trail_length: i32,
    pub trail_scl: f32,
    pub wave_trail_x: f32,
    pub wave_trail_y: f32,
    pub trail_color_rgba: u32,
    pub tleft: TrailState,
    pub tright: TrailState,
}

impl WaterMoveComp {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            rotation: 0.0,
            speed_multiplier: 1.0,
            flying: false,
            ignore_solids: false,
            trail_length: 1,
            trail_scl: 1.0,
            wave_trail_x: 0.0,
            wave_trail_y: 0.0,
            trail_color_rgba: 0x4d79a8ff,
            tleft: TrailState::default(),
            tright: TrailState::default(),
        }
    }

    pub fn add(&mut self) {
        self.tleft = TrailState {
            cleared: true,
            ..TrailState::default()
        };
        self.tright = TrailState {
            cleared: true,
            ..TrailState::default()
        };
    }

    pub fn update<F>(&mut self, mut floor_is_liquid: F)
    where
        F: FnMut(f32, f32) -> bool,
    {
        let rotation = self.rotation - 90.0;
        for side in [-1.0, 1.0] {
            let cx = trnsx(rotation, self.wave_trail_x * side, self.wave_trail_y) + self.x;
            let cy = trnsy(rotation, self.wave_trail_x * side, self.wave_trail_y) + self.y;
            let active = if floor_is_liquid(cx, cy) && !self.flying {
                1.0
            } else {
                0.0
            };
            let trail = if side < 0.0 {
                &mut self.tleft
            } else {
                &mut self.tright
            };
            trail.length = self.trail_length;
            trail.last_x = cx;
            trail.last_y = cy;
            trail.last_active = active;
            trail.cleared = false;
        }
    }

    pub fn draw_plan(&self, floor_color_rgba: u32) -> WaterMoveDrawPlan {
        WaterMoveDrawPlan {
            left: self.tleft,
            right: self.tright,
            trail_scl: self.trail_scl,
            color_rgba: brighten_rgb(if floor_color_rgba == 0x000000ff {
                0x4d79a8ff
            } else {
                floor_color_rgba
            }),
        }
    }

    pub fn tile_x(&self) -> i32 {
        World::to_tile(self.x)
    }

    pub fn tile_y(&self) -> i32 {
        World::to_tile(self.y)
    }

    pub fn solidity(&self) -> Option<WaterCrawlSolidPred> {
        if self.flying || self.ignore_solids {
            None
        } else {
            Some(WaterCrawlSolidPred::WaterSolid)
        }
    }

    pub fn on_solid_with<F>(&self, water_solid: F) -> bool
    where
        F: FnOnce(i32, i32) -> bool,
    {
        water_solid(self.tile_x(), self.tile_y())
    }

    pub fn floor_speed_multiplier(&self, floor_shallow: bool) -> f32 {
        let shallow = !self.flying && floor_shallow;
        (if shallow { 1.0 } else { 1.3 }) * self.speed_multiplier
    }

    pub fn on_liquid(tile_exists: bool, floor_is_liquid: bool) -> bool {
        tile_exists && floor_is_liquid
    }
}

fn trnsx(angle_degrees: f32, x: f32, y: f32) -> f32 {
    let radians = angle_degrees.to_radians();
    radians.cos() * x - radians.sin() * y
}

fn trnsy(angle_degrees: f32, x: f32, y: f32) -> f32 {
    let radians = angle_degrees.to_radians();
    radians.sin() * x + radians.cos() * y
}

fn brighten_rgb(rgba: u32) -> u32 {
    let r = (((rgba >> 24) & 0xff) as f32 * 1.5).min(255.0) as u32;
    let g = (((rgba >> 16) & 0xff) as f32 * 1.5).min(255.0) as u32;
    let b = (((rgba >> 8) & 0xff) as f32 * 1.5).min(255.0) as u32;
    let a = rgba & 0xff;
    (r << 24) | (g << 16) | (b << 8) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_move_add_clears_trails_and_update_tracks_left_right_points() {
        let mut water = WaterMoveComp::new(10.0, 20.0);
        water.rotation = 90.0;
        water.wave_trail_x = 2.0;
        water.wave_trail_y = 3.0;
        water.trail_length = 6;

        water.add();
        assert!(water.tleft.cleared);
        assert!(water.tright.cleared);

        water.update(|_, _| true);

        assert_eq!(water.tleft.length, 6);
        assert_eq!(water.tright.length, 6);
        assert_eq!(water.tleft.last_active, 1.0);
        assert_eq!(water.tright.last_active, 1.0);
        assert!((water.tleft.last_x - 8.0).abs() < 0.0001);
        assert!((water.tright.last_x - 12.0).abs() < 0.0001);
    }

    #[test]
    fn water_move_flying_trails_are_inactive_and_draw_plan_brightens_floor() {
        let mut water = WaterMoveComp::new(0.0, 0.0);
        water.flying = true;
        water.update(|_, _| true);

        assert_eq!(water.tleft.last_active, 0.0);
        assert_eq!(water.tright.last_active, 0.0);
        assert_eq!(water.draw_plan(0x102030ff).color_rgba, 0x183048ff);
    }

    #[test]
    fn water_move_solidity_speed_and_liquid_checks_match_water_crawl_rules() {
        let mut water = WaterMoveComp::new(8.0, 16.0);
        water.speed_multiplier = 2.0;

        assert_eq!(water.solidity(), Some(WaterCrawlSolidPred::WaterSolid));
        assert!(water.on_solid_with(|x, y| x == 1 && y == 2));
        assert_eq!(water.floor_speed_multiplier(true), 2.0);
        assert!(WaterMoveComp::on_liquid(true, true));

        water.ignore_solids = true;
        assert_eq!(water.solidity(), None);
    }
}
