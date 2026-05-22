//! Building component shell mirroring upstream `mindustry.entities.comp.BuildingComp`.
//!
//! This first-stage port keeps the building as a mostly pure data object with
//! tile/world helpers, module snapshots and small state transitions. The heavy
//! Java runtime (`World`, `Events`, `Call`, `PowerGraph`, rendering, logic and
//! network side effects) is intentionally kept out for now.

use crate::mindustry::entities::{EntityPosition, SizedEntity};
use crate::mindustry::io::{TeamId, TypeValue};
use crate::mindustry::vars::TILE_SIZE;
use crate::mindustry::world::block::Block;
use crate::mindustry::world::modules::{ItemModule, LiquidModule, PowerModule};
use crate::mindustry::world::tile::{point2_pack, point2_x, point2_y, BuildingRef};

const TIME_TO_SLEEP: f32 = 60.0;
const RECENT_DAMAGE_TIME: f32 = 60.0 * 5.0;
const RECENT_HEAL_TIME: f32 = 60.0 * 10.0;
const DEFAULT_SUPPRESS_COLOR_RGBA: u32 = 0x98ff_a9ff;

#[derive(Debug, Clone, PartialEq)]
pub struct BuildingComp {
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub max_health: f32,
    pub dead: bool,
    pub team: TeamId,
    pub tile_pos: i32,
    pub block: Block,
    pub rotation: i32,
    pub payload_rotation: f32,
    pub visible_flags: u64,
    pub was_visible: bool,
    pub enabled: bool,
    pub last_disabler: Option<BuildingRef>,
    pub last_accessed: String,
    pub was_damaged: bool,
    pub visual_liquid: f32,
    pub efficiency: f32,
    pub optional_efficiency: f32,
    pub potential_efficiency: f32,
    pub should_consume_power: bool,
    pub heal_suppression_time: f32,
    pub last_heal_time: f32,
    pub suppress_color_rgba: u32,
    pub last_damage_time: f32,
    pub time_scale: f32,
    pub time_scale_duration: f32,
    pub sleeping: bool,
    pub sleep_time: f32,
    pub initialized: bool,
    pub items: Option<ItemModule>,
    pub liquids: Option<LiquidModule>,
    pub power: Option<PowerModule>,
    pub config: Option<TypeValue>,
    pub proximity: Vec<BuildingRef>,
}

impl BuildingComp {
    pub fn new(tile_pos: i32, block: Block, team: TeamId) -> Self {
        let tile_x = point2_x(tile_pos) as f32;
        let tile_y = point2_y(tile_pos) as f32;
        let x = tile_x * TILE_SIZE as f32;
        let y = tile_y * TILE_SIZE as f32;

        Self {
            x,
            y,
            health: block.health as f32,
            max_health: block.health as f32,
            dead: false,
            team,
            tile_pos,
            block: block.clone(),
            rotation: 0,
            payload_rotation: 0.0,
            visible_flags: 0,
            was_visible: false,
            enabled: true,
            last_disabler: None,
            last_accessed: String::new(),
            was_damaged: false,
            visual_liquid: 0.0,
            efficiency: 1.0,
            optional_efficiency: 1.0,
            potential_efficiency: 1.0,
            should_consume_power: block.has_power,
            heal_suppression_time: -1.0,
            last_heal_time: -RECENT_HEAL_TIME,
            suppress_color_rgba: DEFAULT_SUPPRESS_COLOR_RGBA,
            last_damage_time: -RECENT_DAMAGE_TIME,
            time_scale: 1.0,
            time_scale_duration: 0.0,
            sleeping: false,
            sleep_time: 0.0,
            initialized: true,
            items: block.has_items.then(ItemModule::default),
            liquids: block.has_liquids.then(LiquidModule::default),
            power: block.has_power.then(PowerModule::default),
            config: None,
            proximity: Vec::new(),
        }
    }

    pub fn tile_x(&self) -> i32 {
        point2_x(self.tile_pos) as i32
    }

    pub fn tile_y(&self) -> i32 {
        point2_y(self.tile_pos) as i32
    }

    pub fn set_tile_pos(&mut self, tile_pos: i32) {
        self.tile_pos = tile_pos;
        self.x = point2_x(tile_pos) as f32 * TILE_SIZE as f32;
        self.y = point2_y(tile_pos) as f32 * TILE_SIZE as f32;
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        self.rotation = rotation.rem_euclid(4);
    }

    pub fn pos_ref(&self) -> BuildingRef {
        BuildingRef {
            tile_pos: self.tile_pos,
            block: self.block.id,
            team: self.team.0 as i32,
            rotation: self.rotation,
        }
    }

    pub fn rotdeg(&self) -> f32 {
        self.rotation as f32 * 90.0
    }

    pub fn drawrot(&self) -> f32 {
        self.rotdeg()
    }

    pub fn module_bitmask(&self) -> u8 {
        let mut bits = 0u8;
        if self.items.is_some() {
            bits |= 1;
        }
        if self.power.is_some() {
            bits |= 1 << 1;
        }
        if self.liquids.is_some() {
            bits |= 1 << 2;
        }
        if (self.time_scale - 1.0).abs() > f32::EPSILON {
            bits |= 1 << 3;
        }
        if self.visible_flags != 0 {
            bits |= 1 << 4;
        }
        bits
    }

    pub fn apply_boost(&mut self, intensity: f32, duration: f32) {
        if intensity >= self.time_scale - 0.001 {
            self.time_scale_duration = self.time_scale_duration.max(duration);
        }
        self.time_scale = self.time_scale.max(intensity);
    }

    pub fn apply_slowdown(&mut self, intensity: f32, duration: f32) {
        if intensity <= self.time_scale + 0.001 {
            self.time_scale_duration = self.time_scale_duration.max(duration);
        }
        self.time_scale = self.time_scale.min(intensity);
    }

    pub fn apply_heal_suppression(&mut self, now: f32, duration: f32) {
        self.heal_suppression_time = now + duration;
    }

    pub fn is_heal_suppressed(&self, now: f32) -> bool {
        now <= self.heal_suppression_time
    }

    pub fn recently_damaged(&self, now: f32) -> bool {
        now <= self.last_damage_time + RECENT_DAMAGE_TIME
    }

    pub fn recently_healed(&self, now: f32) -> bool {
        now <= self.last_heal_time + RECENT_HEAL_TIME
    }

    pub fn hit_size(&self) -> f32 {
        self.block.size as f32 * TILE_SIZE as f32
    }

    pub fn is_valid(&self) -> bool {
        !self.dead && self.initialized
    }

    pub fn check_allow_update(&self, supports_env: bool, in_bounds: bool) -> bool {
        self.enabled && !self.dead && self.initialized && supports_env && in_bounds
    }

    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    pub fn damage(&mut self, amount: f32, now: f32) {
        if self.dead {
            return;
        }
        self.health = (self.health - amount).max(0.0);
        self.last_damage_time = now;
        self.was_damaged = true;
        if self.health <= 0.0 {
            self.dead = true;
        }
    }

    pub fn heal(&mut self, amount: f32, now: f32) {
        if self.dead || self.is_heal_suppressed(now) || amount <= 0.0 {
            return;
        }
        self.health = (self.health + amount).min(self.max_health);
        self.last_heal_time = now;
    }

    pub fn kill(&mut self) {
        self.dead = true;
        self.health = 0.0;
    }

    pub fn change_team(&mut self, team: TeamId) {
        self.team = team;
        self.was_visible = false;
        self.last_disabler = None;
    }

    pub fn set_config(&mut self, config: Option<TypeValue>) {
        self.config = config;
    }

    pub fn clear_config(&mut self) {
        self.config = None;
    }

    pub fn sleep(&mut self) {
        self.sleep_time += 1.0;
        if !self.sleeping && self.sleep_time >= TIME_TO_SLEEP {
            self.sleeping = true;
        }
    }

    pub fn wake(&mut self) {
        self.sleep_time = 0.0;
        self.sleeping = false;
    }

    pub fn active(&self) -> bool {
        self.enabled && !self.sleeping && !self.dead
    }

    pub fn module_counts(&self) -> (bool, bool, bool) {
        (
            self.items.is_some(),
            self.power.is_some(),
            self.liquids.is_some(),
        )
    }

    pub fn add_proximity(&mut self, build: BuildingRef) {
        if !self.proximity.contains(&build) {
            self.proximity.push(build);
        }
    }

    pub fn clear_proximity(&mut self) {
        self.proximity.clear();
    }
}

impl EntityPosition for BuildingComp {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl SizedEntity for BuildingComp {
    fn hit_size(&self) -> f32 {
        BuildingComp::hit_size(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block() -> Block {
        let mut block = Block::new(5, "router");
        block.has_items = true;
        block.has_liquids = true;
        block.has_power = true;
        block
    }

    #[test]
    fn building_component_initializes_tile_world_and_modules() {
        let building = BuildingComp::new(point2_pack(3, 4), block(), TeamId(2));

        assert_eq!((building.tile_x(), building.tile_y()), (3, 4));
        assert_eq!((building.x(), building.y()), (24.0, 32.0));
        assert_eq!(building.pos_ref().tile_pos, point2_pack(3, 4));
        assert!(building.items.is_some());
        assert!(building.liquids.is_some());
        assert!(building.power.is_some());
        assert_eq!(
            building.hit_size(),
            building.block.size as f32 * TILE_SIZE as f32
        );
    }

    #[test]
    fn building_component_module_bitmask_and_time_scale_helpers_work() {
        let mut building = BuildingComp::new(point2_pack(1, 2), block(), TeamId(1));

        assert_eq!(building.module_bitmask() & 0b111, 0b111);
        assert_eq!(building.time_scale(), 1.0);

        building.apply_boost(2.0, 5.0);
        assert_eq!(building.time_scale(), 2.0);
        building.apply_slowdown(0.5, 3.0);
        assert_eq!(building.time_scale(), 0.5);

        building.apply_heal_suppression(10.0, 5.0);
        assert!(building.is_heal_suppressed(14.0));
        assert!(!building.is_heal_suppressed(16.0));
    }

    #[test]
    fn building_component_damage_sleep_and_team_helpers_are_pure() {
        let mut building = BuildingComp::new(point2_pack(1, 1), block(), TeamId(1));

        building.damage(25.0, 100.0);
        assert_eq!(building.health, building.max_health - 25.0);
        assert!(building.recently_damaged(100.0));
        building.heal(10.0, 120.0);
        assert_eq!(building.health, building.max_health - 15.0);
        assert!(building.recently_healed(120.0));

        building.sleep();
        assert!(!building.sleeping);
        for _ in 0..60 {
            building.sleep();
        }
        assert!(building.sleeping);
        building.wake();
        assert!(!building.sleeping);

        building.change_team(TeamId(3));
        assert_eq!(building.team, TeamId(3));
        building.set_config(Some(TypeValue::String("alpha".into())));
        assert!(matches!(building.config, Some(TypeValue::String(_))));
        building.clear_config();
        assert!(building.config.is_none());
    }

    #[test]
    fn building_component_traits_expose_position_and_size() {
        let building = BuildingComp::new(point2_pack(8, 9), block(), TeamId(1));

        assert_eq!(building.x(), 64.0);
        assert_eq!(building.y(), 72.0);
        assert_eq!(
            building.hit_size(),
            building.block.size as f32 * TILE_SIZE as f32
        );
        assert!(building.check_allow_update(true, true));
        assert!(building.active());
    }
}
