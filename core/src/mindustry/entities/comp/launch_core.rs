//! Launch core component mirroring upstream `mindustry.entities.comp.LaunchCoreComp`.

use crate::mindustry::entities::comp::Interval;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchCoreBlock {
    pub size: i32,
    pub icon_width: f32,
    pub icon_height: f32,
    pub icon_scl: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchCoreDrawPlan {
    pub alpha: f32,
    pub scale: f32,
    pub cx: f32,
    pub cy: f32,
    pub rotation: f32,
    pub light_radius: f32,
    pub icon_width: f32,
    pub icon_height: f32,
    pub shadow_x: f32,
    pub shadow_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchCoreSmoke {
    pub x: f32,
    pub y: f32,
    pub fin: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchCoreComp {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub time: f32,
    pub lifetime: f32,
    pub interval: Interval,
    pub block: LaunchCoreBlock,
}

impl LaunchCoreComp {
    pub const LAYER_EFFECT_OFFSET: f32 = 0.001;

    pub fn new(id: i32, x: f32, y: f32, lifetime: f32, block: LaunchCoreBlock) -> Self {
        Self {
            id,
            x,
            y,
            time: 0.0,
            lifetime,
            interval: Interval::default(),
            block,
        }
    }

    pub fn fin(&self) -> f32 {
        self.time / self.lifetime
    }

    pub fn fout_pow5_out(&self) -> f32 {
        1.0 - (1.0 - self.fin()).powi(5)
    }

    pub fn fslope(&self) -> f32 {
        (1.0 - (self.fin() - 0.5).abs() * 2.0).clamp(0.0, 1.0)
    }

    pub fn cx(&self) -> f32 {
        self.x + self.fin().powi(2) * (12.0 + random_seed_range(self.id + 3, 4.0))
    }

    pub fn cy(&self) -> f32 {
        self.y + self.fin().powi(5) * (100.0 + random_seed_range(self.id + 2, 30.0))
    }

    pub fn draw_plan(&self) -> LaunchCoreDrawPlan {
        let alpha = self.fout_pow5_out();
        let mut scale = (1.0 - alpha) * 1.4 + 1.0;
        let cx = self.cx();
        let cy = self.cy();
        let rotation = self.fin() * (140.0 + random_seed_range(self.id, 50.0));
        let rad = 0.2 + self.fslope();
        let rscl = (self.block.size - 1) as f32 * 0.85;
        let light_radius = 25.0 * (rad + scale - 1.0) * rscl;

        scale *= self.block.icon_scl;
        let icon_width = self.block.icon_width * scale;
        let icon_height = self.block.icon_height * scale;
        let shadow_distance = self.fin().powi(3) * 250.0;
        let shadow_x = cx + trnsx(225.0, shadow_distance, 0.0);
        let shadow_y = cy + trnsy(225.0, shadow_distance, 0.0);

        LaunchCoreDrawPlan {
            alpha,
            scale,
            cx,
            cy,
            rotation,
            light_radius,
            icon_width,
            icon_height,
            shadow_x,
            shadow_y,
        }
    }

    pub fn update(&mut self, now: f32) -> Option<LaunchCoreSmoke> {
        self.interval.set_time(now);
        if self.interval.get(0, 3.0 - self.fin() * 2.0) {
            Some(LaunchCoreSmoke {
                x: self.cx(),
                y: self.cy(),
                fin: self.fin(),
            })
        } else {
            None
        }
    }
}

fn random_seed_range(seed: i32, range: f32) -> f32 {
    let mut x = seed as u32;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    let unit = x as f32 / u32::MAX as f32;
    (unit * 2.0 - 1.0) * range
}

fn trnsx(angle_degrees: f32, x: f32, y: f32) -> f32 {
    let radians = angle_degrees.to_radians();
    radians.cos() * x - radians.sin() * y
}

fn trnsy(angle_degrees: f32, x: f32, y: f32) -> f32 {
    let radians = angle_degrees.to_radians();
    radians.sin() * x + radians.cos() * y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block() -> LaunchCoreBlock {
        LaunchCoreBlock {
            size: 3,
            icon_width: 16.0,
            icon_height: 8.0,
            icon_scl: 1.0,
        }
    }

    #[test]
    fn launch_core_coordinates_follow_interpolated_offsets() {
        let mut launch = LaunchCoreComp::new(1, 10.0, 20.0, 100.0, block());
        launch.time = 50.0;

        assert!(launch.cx() != launch.x);
        assert!(launch.cy() != launch.y);
        assert_eq!(launch.fin(), 0.5);
    }

    #[test]
    fn launch_core_draw_plan_contains_alpha_scale_rotation_and_icon_sizes() {
        let mut launch = LaunchCoreComp::new(2, 0.0, 0.0, 100.0, block());
        launch.time = 50.0;

        let plan = launch.draw_plan();

        assert!(plan.alpha > 0.0);
        assert!(plan.scale >= 1.0);
        assert!(plan.rotation > 0.0);
        assert!(plan.light_radius > 0.0);
        assert!(plan.icon_width > 0.0);
        assert!(plan.icon_height > 0.0);
        assert!(plan.shadow_x != plan.cx || plan.shadow_y != plan.cy);
    }

    #[test]
    fn launch_core_update_emits_smoke_when_interval_is_ready() {
        let mut launch = LaunchCoreComp::new(3, 0.0, 0.0, 100.0, block());
        launch.time = 50.0;

        assert_eq!(launch.update(0.0), None);
        let smoke = launch.update(2.0).unwrap();
        assert_eq!(smoke.fin, 0.5);
    }
}
