//! Puddle component mirroring upstream `mindustry.entities.comp.PuddleComp`.

use crate::mindustry::{
    io::PuddleSyncWire,
    world::{point2_x, point2_y},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuddleTile {
    pub x: i32,
    pub y: i32,
    pub build_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuddleLiquid {
    pub flammability: f32,
    pub viscosity: f32,
    pub move_through_blocks: bool,
    pub cap_puddles: bool,
    pub temperature: f32,
    pub particle_spacing: f32,
    pub has_particle_effect: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuddleUpdateContext {
    pub delta: f32,
    pub nearby_spread_targets: i32,
    pub registry_matches_self: bool,
    pub headless: bool,
    pub fire_chance_passed: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PuddleUpdatePlan {
    pub removed: bool,
    pub force_group_remove: bool,
    pub spread_targets: i32,
    pub deposited_per_target: f32,
    pub affect_units: bool,
    pub create_fire: bool,
    pub puddle_on_building: bool,
    pub particle_effect: bool,
    pub liquid_update: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PuddleComp {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub added: bool,
    pub accepting: f32,
    pub update_time: f32,
    pub last_ripple: f32,
    pub effect_time: f32,
    pub amount: f32,
    pub tile: Option<PuddleTile>,
    pub liquid: Option<PuddleLiquid>,
    pub registered: bool,
    pub removed: bool,
}

impl PuddleComp {
    pub const MAX_LIQUID: f32 = 70.0;

    pub fn new(id: i32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            added: false,
            accepting: 0.0,
            update_time: 0.0,
            last_ripple: 0.0,
            effect_time: 0.0,
            amount: 0.0,
            tile: None,
            liquid: None,
            registered: false,
            removed: false,
        }
    }

    pub fn get_flammability(&self) -> f32 {
        self.liquid
            .map(|liquid| liquid.flammability * self.amount)
            .unwrap_or(0.0)
    }

    pub fn update(&mut self, ctx: PuddleUpdateContext) -> PuddleUpdatePlan {
        let mut plan = PuddleUpdatePlan::default();
        let Some(liquid) = self.liquid else {
            self.remove();
            plan.removed = true;
            return plan;
        };
        let Some(tile) = self.tile else {
            self.remove();
            plan.removed = true;
            return plan;
        };

        let add_speed = if self.accepting > 0.0 { 3.0 } else { 0.0 };
        self.amount -= ctx.delta * (1.0 - liquid.viscosity) / (5.0 + add_speed);
        self.amount += self.accepting;
        self.amount = self.amount.min(Self::MAX_LIQUID);
        self.accepting = 0.0;

        if self.amount >= Self::MAX_LIQUID / 1.5 {
            let deposited = ((self.amount - Self::MAX_LIQUID / 1.5) / 4.0).min(0.3 * ctx.delta);
            let targets = ctx.nearby_spread_targets.max(0);
            self.amount -= deposited * targets as f32;
            plan.spread_targets = targets;
            plan.deposited_per_target = deposited;
        }

        if liquid.cap_puddles {
            self.amount = self.amount.clamp(0.0, Self::MAX_LIQUID);
        }

        if self.amount <= 0.0 {
            self.remove();
            plan.removed = true;
            return plan;
        }

        if !ctx.registry_matches_self && self.added {
            self.added = false;
            plan.force_group_remove = true;
            return plan;
        }

        if self.amount >= Self::MAX_LIQUID / 2.0 && self.update_time <= 0.0 {
            plan.affect_units = true;
            plan.create_fire =
                liquid.temperature > 0.7 && tile.build_present && ctx.fire_chance_passed;
            plan.puddle_on_building = tile.build_present;
            self.update_time = 40.0;
        }

        if !ctx.headless && liquid.has_particle_effect {
            self.effect_time += ctx.delta;
            if self.effect_time >= liquid.particle_spacing {
                plan.particle_effect = true;
                self.effect_time = 0.0;
            }
        }

        self.update_time -= ctx.delta;
        plan.liquid_update = true;
        plan
    }

    pub fn clip_size(&self) -> f32 {
        50.0
    }

    pub fn remove(&mut self) {
        self.removed = true;
        self.registered = false;
    }

    pub fn after_read(&mut self) {
        self.registered = true;
    }

    pub fn after_sync(&mut self) {
        if self.liquid.is_some() {
            self.registered = true;
        }
    }

    pub fn apply_sync_wire(
        &mut self,
        sync: &PuddleSyncWire,
        liquid: Option<PuddleLiquid>,
        tile: Option<PuddleTile>,
    ) {
        self.amount = sync.amount;
        self.liquid = liquid;
        self.tile = tile.or_else(|| {
            sync.tile_pos.map(|tile_pos| PuddleTile {
                x: point2_x(tile_pos) as i32,
                y: point2_y(tile_pos) as i32,
                build_present: false,
            })
        });
        self.x = sync.x;
        self.y = sync.y;
        self.after_sync();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn liquid() -> PuddleLiquid {
        PuddleLiquid {
            flammability: 0.5,
            viscosity: 0.5,
            move_through_blocks: false,
            cap_puddles: true,
            temperature: 1.0,
            particle_spacing: 5.0,
            has_particle_effect: true,
        }
    }

    fn ctx(delta: f32) -> PuddleUpdateContext {
        PuddleUpdateContext {
            delta,
            nearby_spread_targets: 0,
            registry_matches_self: true,
            headless: false,
            fire_chance_passed: false,
        }
    }

    #[test]
    fn puddle_component_flammability_is_liquid_flammability_times_amount() {
        let mut puddle = PuddleComp::new(1, 0.0, 0.0);
        puddle.amount = 10.0;
        puddle.liquid = Some(liquid());

        assert_eq!(puddle.get_flammability(), 5.0);
    }

    #[test]
    fn puddle_component_update_evaporates_accepts_and_spreads_over_cap_threshold() {
        let mut puddle = PuddleComp::new(1, 0.0, 0.0);
        puddle.amount = 60.0;
        puddle.accepting = 20.0;
        puddle.tile = Some(PuddleTile {
            x: 1,
            y: 2,
            build_present: false,
        });
        puddle.liquid = Some(liquid());

        let plan = puddle.update(PuddleUpdateContext {
            nearby_spread_targets: 2,
            ..ctx(1.0)
        });

        assert_eq!(puddle.amount, 69.4);
        assert_eq!(puddle.accepting, 0.0);
        assert_eq!(plan.spread_targets, 2);
        assert_eq!(plan.deposited_per_target, 0.3);
        assert!(plan.liquid_update);
    }

    #[test]
    fn puddle_component_removes_invalid_or_empty_puddles() {
        let mut puddle = PuddleComp::new(1, 0.0, 0.0);
        assert!(puddle.update(ctx(1.0)).removed);

        let mut puddle = PuddleComp::new(1, 0.0, 0.0);
        puddle.amount = 0.1;
        puddle.tile = Some(PuddleTile {
            x: 0,
            y: 0,
            build_present: false,
        });
        puddle.liquid = Some(liquid());
        assert!(puddle.update(ctx(10.0)).removed);
    }

    #[test]
    fn puddle_component_effect_branch_emits_unit_fire_building_and_particle_plans() {
        let mut puddle = PuddleComp::new(1, 0.0, 0.0);
        puddle.amount = 40.0;
        puddle.effect_time = 4.0;
        puddle.tile = Some(PuddleTile {
            x: 0,
            y: 0,
            build_present: true,
        });
        puddle.liquid = Some(liquid());

        let plan = puddle.update(PuddleUpdateContext {
            fire_chance_passed: true,
            ..ctx(1.0)
        });

        assert!(plan.affect_units);
        assert!(plan.create_fire);
        assert!(plan.puddle_on_building);
        assert!(plan.particle_effect);
    }

    #[test]
    fn puddle_component_registry_hooks_and_force_group_remove_match_java_shape() {
        let mut puddle = PuddleComp::new(1, 0.0, 0.0);
        puddle.amount = 10.0;
        puddle.added = true;
        puddle.tile = Some(PuddleTile {
            x: 0,
            y: 0,
            build_present: false,
        });
        puddle.liquid = Some(liquid());

        assert!(
            puddle
                .update(PuddleUpdateContext {
                    registry_matches_self: false,
                    ..ctx(1.0)
                })
                .force_group_remove
        );
        assert!(!puddle.added);

        assert_eq!(puddle.clip_size(), 50.0);
        puddle.after_read();
        assert!(puddle.registered);
        puddle.remove();
        assert!(!puddle.registered);
        puddle.after_sync();
        assert!(puddle.registered);
    }

    #[test]
    fn puddle_component_applies_sync_wire_and_registers_when_liquid_present() {
        let mut puddle = PuddleComp::new(9, 0.0, 0.0);
        let sync = PuddleSyncWire {
            amount: 24.0,
            liquid_id: Some(2),
            tile_pos: Some(crate::mindustry::world::point2_pack(5, 6)),
            x: 40.0,
            y: 48.0,
        };
        let tile = PuddleTile {
            x: 5,
            y: 6,
            build_present: true,
        };

        puddle.apply_sync_wire(&sync, Some(liquid()), Some(tile));

        assert_eq!(puddle.amount, 24.0);
        assert_eq!(puddle.x, 40.0);
        assert_eq!(puddle.y, 48.0);
        assert_eq!(puddle.tile, Some(tile));
        assert_eq!(puddle.liquid, Some(liquid()));
        assert!(puddle.registered);

        let mut null_liquid = PuddleComp::new(10, 0.0, 0.0);
        null_liquid.apply_sync_wire(
            &PuddleSyncWire {
                liquid_id: None,
                ..sync
            },
            None,
            None,
        );
        assert_eq!(
            null_liquid.tile,
            Some(PuddleTile {
                x: 5,
                y: 6,
                build_present: false,
            })
        );
        assert!(!null_liquid.registered);
    }
}
