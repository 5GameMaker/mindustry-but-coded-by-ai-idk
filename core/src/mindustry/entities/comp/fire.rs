//! Fire entity component mirroring upstream `mindustry.entities.comp.FireComp`.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FireTile {
    pub x: i32,
    pub y: i32,
    pub build_present: bool,
    pub flammability: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FireUpdateContext {
    pub delta: f32,
    pub headless: bool,
    pub env_water: f32,
    pub net_client: bool,
    pub puddle_flammability: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FireUpdatePlan {
    pub loop_sound: bool,
    pub removed: bool,
    pub spread: bool,
    pub fireball: bool,
    pub damage_building: bool,
    pub damage_units: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FireComp {
    pub time: f32,
    pub lifetime: f32,
    pub x: f32,
    pub y: f32,
    pub tile: Option<FireTile>,
    pub puddle_flammability: f32,
    pub damage_timer: f32,
    pub spread_timer: f32,
    pub fireball_timer: f32,
    pub warmup: f32,
    pub animation: f32,
    pub removed: bool,
    pub registered: bool,
}

impl FireComp {
    pub const FRAMES: i32 = 40;
    pub const DURATION: f32 = 90.0;
    pub const SPREAD_DELAY: f32 = 22.0;
    pub const FIREBALL_DELAY: f32 = 40.0;
    pub const TICKS_PER_FRAME: f32 = Self::DURATION / Self::FRAMES as f32;
    pub const WARMUP_DURATION: f32 = 20.0;
    pub const DAMAGE_DELAY: f32 = 40.0;
    pub const TILE_DAMAGE: f32 = 1.8;
    pub const UNIT_DAMAGE: f32 = 3.0;

    pub fn new(x: f32, y: f32, lifetime: f32) -> Self {
        Self {
            time: 0.0,
            lifetime,
            x,
            y,
            tile: None,
            puddle_flammability: 0.0,
            damage_timer: 0.0,
            spread_timer: 0.0,
            fireball_timer: 0.0,
            warmup: 0.0,
            animation: 0.0,
            removed: false,
            registered: false,
        }
    }

    pub fn update(&mut self, ctx: FireUpdateContext) -> FireUpdatePlan {
        let mut plan = FireUpdatePlan {
            loop_sound: !ctx.headless,
            ..FireUpdatePlan::default()
        };

        self.animation = (self.animation + ctx.delta / Self::TICKS_PER_FRAME) % Self::FRAMES as f32;
        self.warmup += ctx.delta;

        let speed_multiplier = 1.0 + (ctx.env_water * 10.0).max(0.0);
        self.time = (self.time + ctx.delta * speed_multiplier).clamp(0.0, self.lifetime);

        if ctx.net_client {
            return plan;
        }

        let Some(tile) = self.tile else {
            self.remove();
            plan.removed = true;
            return plan;
        };

        if self.time >= self.lifetime || self.lifetime.is_nan() {
            self.remove();
            plan.removed = true;
            return plan;
        }

        let damage = tile.build_present;
        let flammability = tile.flammability + self.puddle_flammability;

        if !damage && flammability <= 0.0 {
            self.time += ctx.delta * 8.0;
        }

        if damage {
            self.lifetime += (flammability / 8.0).clamp(0.0, 0.6) * ctx.delta;
        }

        if flammability > 1.0 {
            self.spread_timer += ctx.delta * (flammability / 5.0).clamp(0.3, 2.0);
            if self.spread_timer >= Self::SPREAD_DELAY {
                self.spread_timer = 0.0;
                plan.spread = true;
            }
        }

        if flammability > 0.0 {
            self.fireball_timer += ctx.delta * (flammability / 10.0).clamp(0.0, 0.5);
            if self.fireball_timer >= Self::FIREBALL_DELAY {
                self.fireball_timer = 0.0;
                plan.fireball = true;
            }
        }

        self.damage_timer += ctx.delta;
        if self.damage_timer >= Self::DAMAGE_DELAY {
            self.damage_timer = 0.0;
            self.puddle_flammability = ctx.puddle_flammability / 3.0;
            plan.damage_building = damage;
            plan.damage_units = true;
        }

        plan
    }

    pub fn clip_size(&self) -> f32 {
        25.0
    }

    pub fn remove(&mut self) {
        self.removed = true;
        self.registered = false;
    }

    pub fn after_read(&mut self) {
        self.registered = true;
    }

    pub fn after_sync(&mut self) {
        self.registered = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(delta: f32) -> FireUpdateContext {
        FireUpdateContext {
            delta,
            headless: false,
            env_water: 0.0,
            net_client: false,
            puddle_flammability: 0.0,
        }
    }

    #[test]
    fn fire_component_updates_animation_warmup_time_and_sound_plan() {
        let mut fire = FireComp::new(4.0, 5.0, 90.0);
        fire.tile = Some(FireTile {
            x: 1,
            y: 2,
            build_present: false,
            flammability: 2.0,
        });

        let plan = fire.update(ctx(1.0));

        assert!(plan.loop_sound);
        assert_eq!(fire.warmup, 1.0);
        assert_eq!(fire.animation, 1.0 / FireComp::TICKS_PER_FRAME);
        assert_eq!(fire.time, 1.0);
    }

    #[test]
    fn fire_component_removes_when_lifetime_reached_or_tile_missing() {
        let mut fire = FireComp::new(0.0, 0.0, 1.0);

        assert!(fire.update(ctx(1.0)).removed);
        assert!(fire.removed);

        let mut fire = FireComp::new(0.0, 0.0, 1.0);
        fire.tile = Some(FireTile {
            x: 0,
            y: 0,
            build_present: false,
            flammability: 0.0,
        });
        assert!(fire.update(ctx(1.0)).removed);
    }

    #[test]
    fn fire_component_spread_fireball_and_damage_timers_emit_plans() {
        let mut fire = FireComp::new(0.0, 0.0, 90.0);
        fire.tile = Some(FireTile {
            x: 0,
            y: 0,
            build_present: true,
            flammability: 10.0,
        });
        fire.spread_timer = FireComp::SPREAD_DELAY - 1.0;
        fire.fireball_timer = FireComp::FIREBALL_DELAY - 1.0;
        fire.damage_timer = FireComp::DAMAGE_DELAY - 1.0;

        let plan = fire.update(FireUpdateContext {
            puddle_flammability: 6.0,
            ..ctx(1.0)
        });

        assert!(plan.spread);
        assert!(!plan.fireball);
        assert!(plan.damage_building);
        assert!(plan.damage_units);
        assert_eq!(fire.puddle_flammability, 2.0);
        assert!(fire.lifetime > 90.0);
    }

    #[test]
    fn fire_component_client_update_skips_server_side_effects() {
        let mut fire = FireComp::new(0.0, 0.0, 90.0);
        fire.tile = None;

        let plan = fire.update(FireUpdateContext {
            net_client: true,
            ..ctx(1.0)
        });

        assert!(!plan.removed);
        assert!(!fire.removed);
    }

    #[test]
    fn fire_component_register_hooks_and_clip_size_match_java() {
        let mut fire = FireComp::new(0.0, 0.0, 90.0);
        assert_eq!(fire.clip_size(), 25.0);

        fire.after_read();
        assert!(fire.registered);
        fire.remove();
        assert!(!fire.registered);
        fire.after_sync();
        assert!(fire.registered);
    }
}
