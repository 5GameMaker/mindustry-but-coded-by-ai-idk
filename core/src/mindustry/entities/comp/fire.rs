//! Fire entity component mirroring upstream `mindustry.entities.comp.FireComp`.

use crate::mindustry::{
    graphics::{Layer, LightPrimitive, Pal, RenderCommand, RenderPoint, RenderRect},
    io::FireSyncWire,
    world::{point2_x, point2_y},
};

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

#[derive(Debug, Clone, PartialEq)]
pub struct FireDrawPlan {
    pub frame_index: i32,
    pub region: String,
    pub sprite_x: f32,
    pub sprite_y: f32,
    pub sprite_size: f32,
    pub sprite_alpha: f32,
    pub sprite_layer: f32,
    pub light: LightPrimitive,
}

impl FireDrawPlan {
    pub fn sprite_tint(&self) -> [f32; 4] {
        [1.0, 1.0, 1.0, self.sprite_alpha]
    }

    pub fn sprite_rect(&self) -> RenderRect {
        RenderRect::from_center(
            RenderPoint::new(self.sprite_x, self.sprite_y),
            self.sprite_size,
            self.sprite_size,
        )
    }

    pub fn render_commands(&self) -> Vec<RenderCommand> {
        vec![RenderCommand::draw_sprite(
            self.region.clone(),
            self.sprite_rect(),
            self.sprite_tint(),
            0.0,
            self.sprite_layer,
        )]
    }

    pub fn light_primitive(&self) -> LightPrimitive {
        self.light
    }
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
    pub const DRAW_SIZE: f32 = 25.0;

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

    pub fn draw_plan(&self, global_time: f32) -> FireDrawPlan {
        let frame_index = fire_frame_index(self.animation);
        let sprite_alpha = fire_warmup_alpha(self.warmup);
        let sprite_x = self.x + fire_random_seed_range(self.y as i32 as i64, 2.0);
        let sprite_y = self.y + fire_random_seed_range(self.x as i32 as i64, 2.0);

        FireDrawPlan {
            frame_index,
            region: format!("fire{frame_index}"),
            sprite_x,
            sprite_y,
            sprite_size: Self::DRAW_SIZE,
            sprite_alpha,
            sprite_layer: Layer::EFFECT,
            light: LightPrimitive {
                center: (self.x, self.y),
                radius: 50.0 + fire_absin(global_time, 5.0, 5.0),
                color: Pal::LIGHT_FLAME,
                opacity: 0.6 * sprite_alpha,
            },
        }
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

    pub fn apply_sync_wire(&mut self, sync: &FireSyncWire) {
        self.lifetime = sync.lifetime;
        self.time = sync.time;
        self.x = sync.x;
        self.y = sync.y;
        self.tile = sync.tile_pos.map(|tile_pos| FireTile {
            x: point2_x(tile_pos) as i32,
            y: point2_y(tile_pos) as i32,
            build_present: false,
            flammability: 0.0,
        });
        self.after_sync();
    }
}

fn fire_frame_index(animation: f32) -> i32 {
    (animation as i32).clamp(0, FireComp::FRAMES - 1)
}

fn fire_warmup_alpha(warmup: f32) -> f32 {
    (warmup / FireComp::WARMUP_DURATION).clamp(0.0, 1.0)
}

fn fire_absin(time: f32, scl: f32, mag: f32) -> f32 {
    (time / (scl * 2.0)).sin() * mag / 2.0 + mag / 2.0
}

fn fire_random_seed_range(seed: i64, range: f32) -> f32 {
    ArcRand::with_seed(seed.wrapping_mul(99_999)).range(range)
}

#[derive(Debug, Clone, Copy)]
struct ArcRand {
    seed0: u64,
    seed1: u64,
}

impl ArcRand {
    fn with_seed(seed: i64) -> Self {
        let mut rand = Self { seed0: 0, seed1: 0 };
        rand.set_seed(seed);
        rand
    }

    fn set_seed(&mut self, seed: i64) {
        let seed = if seed == 0 {
            0x8000_0000_0000_0000
        } else {
            seed as u64
        };
        let hashed = murmur_hash3(seed);
        self.seed0 = hashed;
        self.seed1 = murmur_hash3(hashed);
    }

    fn next_long(&mut self) -> u64 {
        let mut seed0 = self.seed0;
        let seed1 = self.seed1;
        self.seed0 = seed1;
        seed0 ^= seed0 << 23;
        self.seed1 = seed0 ^ seed1 ^ (seed0 >> 17) ^ (seed1 >> 26);
        self.seed1.wrapping_add(seed1)
    }

    fn next_float(&mut self) -> f32 {
        ((self.next_long() >> 40) as f64 * (1.0 / (1u64 << 24) as f64)) as f32
    }

    fn range(&mut self, range: f32) -> f32 {
        self.next_float() * range * 2.0 - range
    }
}

fn murmur_hash3(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^= value >> 33;
    value
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

    #[test]
    fn fire_component_applies_sync_wire_and_registers_like_after_sync() {
        let mut fire = FireComp::new(0.0, 0.0, 1.0);
        let sync = FireSyncWire {
            lifetime: 120.0,
            tile_pos: Some(crate::mindustry::world::point2_pack(5, 6)),
            time: 30.0,
            x: 40.0,
            y: 48.0,
        };

        fire.apply_sync_wire(&sync);

        assert_eq!(fire.lifetime, 120.0);
        assert_eq!(fire.time, 30.0);
        assert_eq!(fire.x, 40.0);
        assert_eq!(fire.y, 48.0);
        assert_eq!(
            fire.tile,
            Some(FireTile {
                x: 5,
                y: 6,
                build_present: false,
                flammability: 0.0,
            })
        );
        assert!(fire.registered);
    }

    #[test]
    fn fire_component_draw_plan_matches_java_sprite_and_light_arguments() {
        let mut fire = FireComp::new(42.25, 133.75, 90.0);
        fire.animation = 39.9;
        fire.warmup = 10.0;

        let plan = fire.draw_plan(12.5);

        assert_eq!(plan.frame_index, 39);
        assert_eq!(plan.region, "fire39");
        assert_eq!(plan.sprite_layer, Layer::EFFECT);
        assert_eq!(plan.sprite_size, FireComp::DRAW_SIZE);
        assert_eq!(plan.sprite_alpha, 0.5);
        assert_eq!(plan.sprite_tint(), [1.0, 1.0, 1.0, 0.5]);
        assert!((plan.sprite_x - (42.25 + fire_random_seed_range(133, 2.0))).abs() < 0.0001);
        assert!((plan.sprite_y - (133.75 + fire_random_seed_range(42, 2.0))).abs() < 0.0001);

        assert_eq!(plan.light.center, (42.25, 133.75));
        assert!((plan.light.radius - (50.0 + fire_absin(12.5, 5.0, 5.0))).abs() < 0.0001);
        assert_eq!(plan.light.color, Pal::LIGHT_FLAME);
        assert!((plan.light.opacity - 0.3).abs() < 0.0001);
        assert_eq!(plan.light_primitive(), plan.light);

        let commands = plan.render_commands();
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            RenderCommand::DrawSprite {
                symbol,
                rect,
                origin,
                tint,
                rotation,
                layer,
                ..
            } => {
                assert_eq!(symbol, "fire39");
                assert!((rect.center().x - plan.sprite_x).abs() < 0.0001);
                assert!((rect.center().y - plan.sprite_y).abs() < 0.0001);
                assert_eq!(*origin, rect.center_origin());
                assert_eq!(*tint, [1.0, 1.0, 1.0, 0.5]);
                assert_eq!(*rotation, 0.0);
                assert_eq!(*layer, Layer::EFFECT);
            }
            command => panic!("expected fire sprite command, got {command:?}"),
        }
    }

    #[test]
    fn fire_component_draw_plan_clamps_frame_and_warmup_like_java() {
        let mut fire = FireComp::new(0.0, 0.0, 90.0);
        fire.animation = -7.0;
        fire.warmup = -10.0;
        let cold = fire.draw_plan(0.0);
        assert_eq!(cold.frame_index, 0);
        assert_eq!(cold.region, "fire0");
        assert_eq!(cold.sprite_alpha, 0.0);
        assert_eq!(cold.light.opacity, 0.0);

        fire.animation = 90.0;
        fire.warmup = 90.0;
        let hot = fire.draw_plan(0.0);
        assert_eq!(hot.frame_index, FireComp::FRAMES - 1);
        assert_eq!(hot.region, "fire39");
        assert_eq!(hot.sprite_alpha, 1.0);
        assert_eq!(hot.light.opacity, 0.6);
    }

    #[test]
    fn fire_random_seed_range_matches_arc_seeded_range() {
        assert!((fire_random_seed_range(133, 0.12) + 0.085_423_604).abs() < 0.000_001);
        assert!((fire_random_seed_range(42, 0.12) + 0.019_490_603).abs() < 0.000_001);
    }
}
