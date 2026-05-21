//! Crawl component mirroring upstream `mindustry.entities.comp.CrawlComp`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrawlSolidPred {
    LegsSolid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrawlType {
    pub floor_multiplier: f32,
    pub segment_rot_speed: f32,
    pub segment_max_rot: f32,
    pub crawl_slowdown: f32,
    pub crawl_slowdown_frac: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrawlUpdateInput {
    pub moving: bool,
    pub delta: f32,
    pub delta_len: f32,
    pub solid_tiles: i32,
    pub deep_tiles: i32,
    pub sampled_tiles: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrawlComp {
    pub x: f32,
    pub y: f32,
    pub speed_multiplier: f32,
    pub rotation: f32,
    pub hit_size: f32,
    pub type_info: CrawlType,
    pub last_deep_floor: bool,
    pub last_crawl_slowdown: f32,
    pub segment_rot: f32,
    pub crawl_time: f32,
    pub ignore_solids: bool,
    pub flying: bool,
}

impl CrawlComp {
    pub const fn new(type_info: CrawlType) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            speed_multiplier: 1.0,
            rotation: 0.0,
            hit_size: 0.0,
            type_info,
            last_deep_floor: false,
            last_crawl_slowdown: 1.0,
            segment_rot: 0.0,
            crawl_time: 0.0,
            ignore_solids: false,
            flying: false,
        }
    }

    pub fn solidity(&self) -> Option<CrawlSolidPred> {
        if self.ignore_solids {
            None
        } else {
            Some(CrawlSolidPred::LegsSolid)
        }
    }

    pub fn floor_speed_multiplier(&self, floor_is_deep: bool, floor_speed_multiplier: f32) -> f32 {
        let base = if self.flying {
            1.0
        } else if floor_is_deep {
            0.45
        } else {
            floor_speed_multiplier
        };
        base.powf(self.type_info.floor_multiplier)
            * self.speed_multiplier
            * self.last_crawl_slowdown
    }

    pub fn add(&mut self) {
        self.segment_rot = self.rotation;
    }

    pub fn drown_floor(&self) -> bool {
        self.last_deep_floor
    }

    pub fn update(&mut self, input: CrawlUpdateInput) {
        if input.moving {
            self.segment_rot = move_toward(
                self.segment_rot,
                self.rotation,
                self.type_info.segment_rot_speed * input.delta,
            );
            let count = input.sampled_tiles.max(1) as f32;
            let deeps = input.deep_tiles.max(0) as f32;
            self.last_deep_floor = deeps / count >= 0.75;
            let solid_frac =
                input.solid_tiles.max(0) as f32 / count / self.type_info.crawl_slowdown_frac;
            self.last_crawl_slowdown = lerp(
                1.0,
                self.type_info.crawl_slowdown,
                solid_frac.clamp(0.0, 1.0),
            );
        }
        self.segment_rot = clamp_range(
            self.segment_rot,
            self.rotation,
            self.type_info.segment_max_rot,
        );
        self.crawl_time += input.delta_len;
    }
}

fn lerp(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn move_toward(from: f32, to: f32, amount: f32) -> f32 {
    let delta = (to - from + 540.0).rem_euclid(360.0) - 180.0;
    if delta.abs() <= amount {
        to
    } else {
        (from + amount * delta.signum()).rem_euclid(360.0)
    }
}

fn clamp_range(value: f32, target: f32, range: f32) -> f32 {
    let delta = (value - target + 540.0).rem_euclid(360.0) - 180.0;
    if delta.abs() <= range {
        value
    } else {
        (target + range * delta.signum()).rem_euclid(360.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crawl_type() -> CrawlType {
        CrawlType {
            floor_multiplier: 2.0,
            segment_rot_speed: 10.0,
            segment_max_rot: 30.0,
            crawl_slowdown: 0.5,
            crawl_slowdown_frac: 0.5,
        }
    }

    #[test]
    fn crawl_solidity_and_add_match_java_defaults() {
        let mut crawl = CrawlComp::new(crawl_type());
        crawl.rotation = 45.0;
        assert_eq!(crawl.solidity(), Some(CrawlSolidPred::LegsSolid));
        crawl.ignore_solids = true;
        assert_eq!(crawl.solidity(), None);

        crawl.add();
        assert_eq!(crawl.segment_rot, 45.0);
    }

    #[test]
    fn crawl_floor_speed_multiplier_uses_deep_floor_and_slowdown() {
        let mut crawl = CrawlComp::new(crawl_type());
        crawl.speed_multiplier = 2.0;
        crawl.last_crawl_slowdown = 0.5;

        assert!((crawl.floor_speed_multiplier(false, 0.8) - 0.64).abs() < 0.0001);
        assert!((crawl.floor_speed_multiplier(true, 1.0) - 0.2025).abs() < 0.0001);
        crawl.flying = true;
        assert_eq!(crawl.floor_speed_multiplier(true, 0.8), 1.0);
    }

    #[test]
    fn crawl_update_tracks_deep_floor_slowdown_rotation_and_time() {
        let mut crawl = CrawlComp::new(crawl_type());
        crawl.rotation = 90.0;

        crawl.update(CrawlUpdateInput {
            moving: true,
            delta: 1.0,
            delta_len: 5.0,
            solid_tiles: 2,
            deep_tiles: 3,
            sampled_tiles: 4,
        });

        assert_eq!(crawl.segment_rot, 60.0);
        assert!(crawl.drown_floor());
        assert_eq!(crawl.last_crawl_slowdown, 0.5);
        assert_eq!(crawl.crawl_time, 5.0);
    }
}
