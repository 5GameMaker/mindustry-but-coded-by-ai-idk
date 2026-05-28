//! Power generator shell mirroring upstream `mindustry.world.blocks.power.PowerGenerator`.

use crate::mindustry::{
    ctype::ContentId,
    vars::TILE_SIZE,
    world::{
        meta::{BlockFlag, BlockGroup, Stat},
        Block,
    },
};

use super::PowerDistributor;

#[derive(Debug, Clone, PartialEq)]
pub struct PowerGenerator {
    pub base: Block,
    pub power_production: f32,
    pub generation_type: Stat,
    pub drawer: String,
    pub base_explosiveness: f32,
    pub explosion_radius: i32,
    pub explosion_damage: i32,
    pub explode_effect: String,
    pub explode_sound: String,
    pub explosion_puddles: i32,
    pub explosion_puddle_range: f32,
    pub explosion_puddle_amount: f32,
    pub explosion_puddle_liquid: Option<ContentId>,
    pub explosion_min_warmup: f32,
    pub explosion_shake: f32,
    pub explosion_shake_duration: f32,
}

impl PowerGenerator {
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = PowerDistributor::new(name).base;
        base.group = BlockGroup::Power;
        base.sync = true;
        base.outputs_power = true;
        base.consumes_power = false;
        base.flags.push(BlockFlag::Generator);

        Self {
            base,
            power_production: 0.0,
            generation_type: Stat::BasePowerGeneration,
            drawer: "DrawDefault".into(),
            base_explosiveness: 5.0,
            explosion_radius: 12,
            explosion_damage: 0,
            explode_effect: "none".into(),
            explode_sound: "none".into(),
            explosion_puddles: 10,
            explosion_puddle_range: TILE_SIZE as f32 * 2.0,
            explosion_puddle_amount: 100.0,
            explosion_puddle_liquid: None,
            explosion_min_warmup: 0.0,
            explosion_shake: 0.0,
            explosion_shake_duration: 6.0,
        }
    }

    pub fn displayed_power_production(&self) -> f32 {
        self.power_production
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_generator_sets_java_constructor_defaults() {
        let generator = PowerGenerator::new("combustion-generator");

        assert_eq!(generator.base.name, "combustion-generator");
        assert!(generator.base.update);
        assert!(generator.base.solid);
        assert!(generator.base.has_power);
        assert_eq!(generator.base.group, BlockGroup::Power);
        assert!(generator.base.sync);
        assert!(generator.base.outputs_power);
        assert!(!generator.base.consumes_power);
        assert!(generator.base.flags.contains(&BlockFlag::Generator));

        assert_eq!(generator.power_production, 0.0);
        assert_eq!(generator.generation_type, Stat::BasePowerGeneration);
        assert_eq!(generator.drawer, "DrawDefault");
        assert_eq!(generator.base_explosiveness, 5.0);
        assert_eq!(generator.explosion_radius, 12);
        assert_eq!(generator.explosion_damage, 0);
        assert_eq!(generator.explode_effect, "none");
        assert_eq!(generator.explode_sound, "none");
        assert_eq!(generator.explosion_puddles, 10);
        assert_eq!(generator.explosion_puddle_range, TILE_SIZE as f32 * 2.0);
        assert_eq!(generator.explosion_puddle_amount, 100.0);
        assert_eq!(generator.explosion_puddle_liquid, None);
        assert_eq!(generator.explosion_min_warmup, 0.0);
        assert_eq!(generator.explosion_shake, 0.0);
        assert_eq!(generator.explosion_shake_duration, 6.0);
        assert_eq!(generator.displayed_power_production(), 0.0);
    }
}
