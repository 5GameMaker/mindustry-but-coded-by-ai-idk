//! Water crawl component mirroring upstream `mindustry.entities.comp.WaterCrawlComp`.

use crate::mindustry::core::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterCrawlSolidPred {
    WaterSolid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterCrawlComp {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub speed_multiplier: f32,
    pub flying: bool,
    pub ignore_solids: bool,
}

impl WaterCrawlComp {
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            rotation: 0.0,
            speed_multiplier: 1.0,
            flying: false,
            ignore_solids: false,
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

    /// Java uses air floor when flying; air is not shallow.
    pub fn floor_speed_multiplier(&self, floor_shallow: bool) -> f32 {
        let shallow = !self.flying && floor_shallow;
        (if shallow { 1.0 } else { 1.3 }) * self.speed_multiplier
    }

    pub fn on_liquid(tile_exists: bool, floor_is_liquid: bool) -> bool {
        tile_exists && floor_is_liquid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_crawl_solidity_returns_water_solid_unless_flying_or_ignoring() {
        let mut crawl = WaterCrawlComp::new(8.0, 16.0);
        assert_eq!(crawl.solidity(), Some(WaterCrawlSolidPred::WaterSolid));

        crawl.flying = true;
        assert_eq!(crawl.solidity(), None);

        crawl.flying = false;
        crawl.ignore_solids = true;
        assert_eq!(crawl.solidity(), None);
    }

    #[test]
    fn water_crawl_on_solid_uses_tile_coordinates_and_floor_speed_rule() {
        let mut crawl = WaterCrawlComp::new(8.0, 16.0);
        crawl.speed_multiplier = 2.0;

        assert!(crawl.on_solid_with(|x, y| x == 1 && y == 2));
        assert_eq!(crawl.floor_speed_multiplier(true), 2.0);
        assert_eq!(crawl.floor_speed_multiplier(false), 2.6);

        crawl.flying = true;
        assert_eq!(crawl.floor_speed_multiplier(true), 2.6);
    }

    #[test]
    fn water_crawl_on_liquid_requires_tile_and_liquid_floor() {
        assert!(WaterCrawlComp::on_liquid(true, true));
        assert!(!WaterCrawlComp::on_liquid(false, true));
        assert!(!WaterCrawlComp::on_liquid(true, false));
    }
}
