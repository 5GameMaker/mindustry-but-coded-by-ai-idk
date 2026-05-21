//! Tank component mirroring upstream `mindustry.entities.comp.TankComp`.

use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankType {
    pub floor_multiplier: f32,
    pub crawl_slowdown: f32,
    pub crawl_slowdown_frac: f32,
    pub hovering: bool,
    pub crush_fragile: bool,
    pub crush_damage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankUpdateInput {
    pub delta: f32,
    pub delta_len: f32,
    pub net_client: bool,
    pub headless_or_fogged: bool,
    pub solid_tiles: i32,
    pub sampled_tiles: i32,
    pub any_non_deep: bool,
    pub found_deep_floor: bool,
    pub fragile_targets: i32,
    pub crush_targets: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TankUpdatePlan {
    pub dust_effects: i32,
    pub fragile_crushes: i32,
    pub crush_damage_targets: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankComp {
    pub x: f32,
    pub y: f32,
    pub hit_size: f32,
    pub rotation: f32,
    pub speed_multiplier: f32,
    pub disarmed: bool,
    pub type_info: TankType,
    pub tread_effect_time: f32,
    pub last_slowdown: f32,
    pub tread_time: f32,
    pub walked: bool,
    pub last_deep_floor: bool,
    pub flying: bool,
}

impl TankComp {
    pub const fn new(type_info: TankType) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            hit_size: 0.0,
            rotation: 0.0,
            speed_multiplier: 1.0,
            disarmed: false,
            type_info,
            tread_effect_time: 0.0,
            last_slowdown: 1.0,
            tread_time: 0.0,
            walked: false,
            last_deep_floor: false,
            flying: false,
        }
    }

    pub fn update(&mut self, input: TankUpdateInput) -> TankUpdatePlan {
        let mut plan = TankUpdatePlan::default();
        if (self.walked || (input.net_client && input.delta_len >= 0.01))
            && !input.headless_or_fogged
        {
            self.tread_effect_time += input.delta;
            if self.tread_effect_time >= 6.0 {
                plan.dust_effects = 2;
                self.tread_effect_time = 0.0;
            }
        }

        if self.type_info.crush_fragile && !self.disarmed {
            plan.fragile_crushes = input.fragile_targets.max(0);
        }

        self.last_deep_floor = input.found_deep_floor && !input.any_non_deep;
        let total = input.sampled_tiles.max(1) as f32;
        self.last_slowdown = lerp(
            1.0,
            self.type_info.crawl_slowdown,
            (input.solid_tiles.max(0) as f32 / total / self.type_info.crawl_slowdown_frac)
                .clamp(0.0, 1.0),
        );

        if self.type_info.crush_damage > 0.0
            && !self.disarmed
            && (self.walked || input.delta_len >= 0.01)
        {
            plan.crush_damage_targets = input.crush_targets.max(0);
        }

        if self.walked || input.net_client {
            self.tread_time += input.delta_len;
            self.walked = false;
        }

        plan
    }

    pub fn floor_speed_multiplier(&self, floor_speed_multiplier: f32) -> f32 {
        let base = if self.flying || self.type_info.hovering {
            1.0
        } else {
            floor_speed_multiplier
        };
        base.powf(self.type_info.floor_multiplier) * self.speed_multiplier * self.last_slowdown
    }

    pub fn drown_floor(&self, can_drown: bool) -> bool {
        can_drown && self.last_deep_floor
    }

    pub fn move_at(&mut self, vector: Vec2) {
        if !is_zero(vector, 0.001) {
            self.walked = true;
        }
    }

    pub fn approach(&mut self, vector: Vec2) {
        if !is_zero(vector, 0.001) {
            self.walked = true;
        }
    }
}

fn is_zero(vec: Vec2, tolerance: f32) -> bool {
    vec.x * vec.x + vec.y * vec.y <= tolerance * tolerance
}

fn lerp(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tank_type() -> TankType {
        TankType {
            floor_multiplier: 2.0,
            crawl_slowdown: 0.5,
            crawl_slowdown_frac: 0.5,
            hovering: false,
            crush_fragile: true,
            crush_damage: 10.0,
        }
    }

    #[test]
    fn tank_move_at_and_update_advance_tread_time_and_emit_effects() {
        let mut tank = TankComp::new(tank_type());
        tank.move_at(Vec2 { x: 1.0, y: 0.0 });
        tank.tread_effect_time = 5.5;

        let plan = tank.update(TankUpdateInput {
            delta: 1.0,
            delta_len: 3.0,
            net_client: false,
            headless_or_fogged: false,
            solid_tiles: 2,
            sampled_tiles: 4,
            any_non_deep: false,
            found_deep_floor: true,
            fragile_targets: 1,
            crush_targets: 2,
        });

        assert_eq!(plan.dust_effects, 2);
        assert_eq!(plan.fragile_crushes, 1);
        assert_eq!(plan.crush_damage_targets, 2);
        assert_eq!(tank.tread_time, 3.0);
        assert!(!tank.walked);
        assert!(tank.last_deep_floor);
        assert_eq!(tank.last_slowdown, 0.5);
    }

    #[test]
    fn tank_floor_speed_and_drown_floor_follow_java_rules() {
        let mut tank = TankComp::new(tank_type());
        tank.speed_multiplier = 2.0;
        tank.last_slowdown = 0.5;
        assert!((tank.floor_speed_multiplier(0.8) - 0.64).abs() < 0.0001);

        tank.flying = true;
        assert_eq!(tank.floor_speed_multiplier(0.8), 1.0);

        tank.last_deep_floor = true;
        assert!(tank.drown_floor(true));
        assert!(!tank.drown_floor(false));
    }
}
