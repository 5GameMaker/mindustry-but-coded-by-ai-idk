//! Building component shell mirroring upstream `mindustry.entities.comp.BuildingComp`.
//!
//! This first-stage port keeps the building as a mostly pure data object with
//! tile/world helpers, module snapshots and small state transitions. The heavy
//! Java runtime (`World`, `Events`, `Call`, `PowerGraph`, rendering, logic and
//! network side effects) is intentionally kept out for now.

use std::io::{self, Read, Write};

use crate::mindustry::entities::{EntityPosition, SizedEntity};
use crate::mindustry::io::{
    type_io::{read_bool, read_f32, read_i32, read_i64, read_u8, write_f32, write_i32, write_i64},
    TeamId, TypeValue,
};
use crate::mindustry::vars::TILE_SIZE;
use crate::mindustry::world::block::{Block, BlockId};
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
        (self.items.is_some() as u8)
            | ((self.power.is_some() as u8) << 1)
            | ((self.liquids.is_some() as u8) << 2)
            | (1 << 3) // legacy consume module bit; Java still writes it for save compatibility.
            | (((self.time_scale - 1.0).abs() > f32::EPSILON) as u8) << 4
            | ((self.last_disabler.is_some() as u8) << 5)
    }

    pub fn write_base<W: Write>(&self, write: &mut W, fog_enabled: bool) -> io::Result<()> {
        let write_visibility = fog_enabled && self.visible_flags != 0;

        write_f32(write, self.health)?;
        write.write_all(&[(self.rotation.rem_euclid(128) as u8) | 0b1000_0000])?;
        write.write_all(&[self.team.0])?;
        write.write_all(&[if write_visibility { 4 } else { 3 }])?;
        write.write_all(&[self.enabled as u8])?;
        write.write_all(&[self.module_bitmask()])?;

        if let Some(items) = &self.items {
            items.write(write)?;
        }
        if let Some(power) = &self.power {
            power.write(write)?;
        }
        if let Some(liquids) = &self.liquids {
            liquids.write(write)?;
        }

        if (self.time_scale - 1.0).abs() > f32::EPSILON {
            write_f32(write, self.time_scale)?;
            write_f32(write, self.time_scale_duration)?;
        }

        if let Some(last_disabler) = &self.last_disabler {
            write_i32(write, last_disabler.tile_pos)?;
        }

        write.write_all(&[(clamp01(self.efficiency) * 255.0) as u8])?;
        write.write_all(&[(clamp01(self.optional_efficiency) * 255.0) as u8])?;

        if write_visibility {
            write_i64(write, self.visible_flags as i64)?;
        }

        Ok(())
    }

    pub fn read_base<R: Read>(&mut self, read: &mut R) -> io::Result<()> {
        self.health = read_f32(read)?.min(self.block.health as f32);
        let rot = read_u8(read)?;
        self.team = TeamId(read_u8(read)?);
        self.rotation = (rot & 0b0111_1111) as i32;

        let mut module_bits = self.module_bitmask();
        let mut legacy = true;
        let mut version = 0u8;

        if (rot & 0b1000_0000) != 0 {
            version = read_u8(read)?;
            if version >= 1 {
                self.enabled = read_u8(read)? == 1;
            }
            if version >= 2 {
                module_bits = read_u8(read)?;
            }
            legacy = false;
        }

        if (module_bits & 1) != 0 {
            let items = self.items.get_or_insert_with(ItemModule::default);
            items.read(read, legacy)?;
        }
        if (module_bits & (1 << 1)) != 0 {
            let power = self.power.get_or_insert_with(PowerModule::default);
            power.read(read)?;
        }
        if (module_bits & (1 << 2)) != 0 {
            let liquids = self.liquids.get_or_insert_with(LiquidModule::default);
            liquids.read(read, legacy)?;
        }

        if (module_bits & (1 << 4)) != 0 {
            self.time_scale = read_f32(read)?;
            self.time_scale_duration = read_f32(read)?;
        } else {
            self.time_scale = 1.0;
            self.time_scale_duration = 0.0;
        }

        if (module_bits & (1 << 5)) != 0 {
            self.last_disabler = Some(BuildingRef {
                tile_pos: read_i32(read)?,
                block: 0 as BlockId,
                team: -1,
                rotation: 0,
            });
        } else {
            self.last_disabler = None;
        }

        // Java saves a now-unused consume module presence boolean through version 2.
        if version <= 2 {
            let _ = read_bool(read)?;
        }

        if version >= 3 {
            self.efficiency = read_u8(read)? as f32 / 255.0;
            self.potential_efficiency = self.efficiency;
            self.optional_efficiency = read_u8(read)? as f32 / 255.0;
        }

        if version == 4 {
            self.visible_flags = read_i64(read)? as u64;
        } else {
            self.visible_flags = 0;
        }

        Ok(())
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

fn clamp01(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
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
        block.health = 100;
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

        assert_eq!(building.module_bitmask() & 0b1111, 0b1111);
        assert_eq!(building.time_scale(), 1.0);

        building.apply_boost(2.0, 5.0);
        assert_eq!(building.time_scale(), 2.0);
        assert_ne!(building.module_bitmask() & (1 << 4), 0);
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

    #[test]
    fn building_component_write_base_matches_java_v3_layout() {
        let mut building = BuildingComp::new(point2_pack(3, 4), block(), TeamId(2));
        building.health = 42.5;
        building.set_rotation(3);
        building.enabled = false;
        building.items.as_mut().unwrap().set(1, 4);
        building.power.as_mut().unwrap().links = vec![point2_pack(2, 2)];
        building.power.as_mut().unwrap().status = 0.75;
        building.liquids.as_mut().unwrap().set(5, 2.5);
        building.apply_boost(1.5, 9.0);
        building.efficiency = 0.5;
        building.optional_efficiency = 1.0;

        let mut bytes = Vec::new();
        building.write_base(&mut bytes, false).unwrap();

        let mut restored = BuildingComp::new(point2_pack(3, 4), block(), TeamId(0));
        restored.read_base(&mut bytes.as_slice()).unwrap();

        assert_eq!(restored.health, 42.5);
        assert_eq!(restored.rotation, 3);
        assert_eq!(restored.team, TeamId(2));
        assert!(!restored.enabled);
        assert_eq!(restored.items.as_ref().unwrap().get(1), 4);
        assert_eq!(
            restored.power.as_ref().unwrap().links,
            vec![point2_pack(2, 2)]
        );
        assert_eq!(restored.power.as_ref().unwrap().status, 0.75);
        assert_eq!(restored.liquids.as_ref().unwrap().get(5), 2.5);
        assert_eq!(restored.time_scale, 1.5);
        assert_eq!(restored.time_scale_duration, 9.0);
        assert!((restored.efficiency - 127.0 / 255.0).abs() < f32::EPSILON);
        assert_eq!(restored.optional_efficiency, 1.0);
        assert_eq!(restored.visible_flags, 0);
    }

    #[test]
    fn building_component_write_base_can_include_v4_visibility_and_disabler() {
        let mut building = BuildingComp::new(point2_pack(1, 1), block(), TeamId(1));
        building.visible_flags = 0x0102_0304_0506_0708;
        building.last_disabler = Some(BuildingRef {
            tile_pos: point2_pack(7, 8),
            block: 99,
            team: 1,
            rotation: 2,
        });

        let mut bytes = Vec::new();
        building.write_base(&mut bytes, true).unwrap();
        assert_eq!(bytes[6], 4);

        let mut restored = BuildingComp::new(point2_pack(1, 1), block(), TeamId(0));
        restored.read_base(&mut bytes.as_slice()).unwrap();

        assert_eq!(restored.visible_flags, 0x0102_0304_0506_0708);
        assert_eq!(
            restored.last_disabler.map(|build| build.tile_pos),
            Some(point2_pack(7, 8))
        );
    }
}
