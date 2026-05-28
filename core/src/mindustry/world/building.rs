//! Core world building runtime shared by block implementations.
//!
//! This mirrors the data-oriented parts of upstream `BuildingComp`: tile/block
//! identity, base module allocation, save/load base IO and small lifecycle
//! transitions. Rendering, events, network calls and concrete block subclasses
//! stay outside this base layer.

use std::io::{self, Read, Write};

use crate::mindustry::{
    io::{
        type_io::{
            read_bool, read_f32, read_i32, read_i64, read_u8, write_f32, write_i32, write_i64,
            write_u8,
        },
        TypeValue,
    },
    vars::TILE_SIZE,
    world::{
        meta::BlockStatus,
        modules::{ItemModule, LiquidModule, PowerModule},
        point2_pack, point2_x, point2_y, Block, BuildingRef,
    },
};

pub const BUILDING_TIME_TO_SLEEP: f32 = 60.0;
pub const BUILDING_RECENT_DAMAGE_TIME: f32 = 60.0 * 5.0;
pub const BUILDING_RECENT_HEAL_TIME: f32 = 60.0 * 10.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    pub tile_pos: i32,
    pub block: Block,
    pub team: i32,
    pub rotation: i32,
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub max_health: f32,
    pub dead: bool,
    pub visible_flags: u64,
    pub was_visible: bool,
    pub enabled: bool,
    pub last_disabler: Option<BuildingRef>,
    pub last_accessed: Option<String>,
    pub visual_liquid: f32,
    pub efficiency: f32,
    pub optional_efficiency: f32,
    pub potential_efficiency: f32,
    pub should_consume_power: bool,
    pub heal_suppression_time: f32,
    pub last_heal_time: f32,
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
    pub cdump: usize,
    pub dump_accum: f32,
}

impl Building {
    pub fn new(tile_pos: i32, block: &Block, team: i32, rotation: i32) -> Self {
        let mut building = Self {
            tile_pos,
            block: block.clone(),
            team,
            rotation: block.plan_rotation(rotation),
            x: 0.0,
            y: 0.0,
            health: block.health as f32,
            max_health: block.health as f32,
            dead: false,
            visible_flags: 0,
            was_visible: false,
            enabled: true,
            last_disabler: None,
            last_accessed: None,
            visual_liquid: 0.0,
            efficiency: 1.0,
            optional_efficiency: 1.0,
            potential_efficiency: 1.0,
            should_consume_power: block.has_power,
            heal_suppression_time: -1.0,
            last_heal_time: -BUILDING_RECENT_HEAL_TIME,
            last_damage_time: -BUILDING_RECENT_DAMAGE_TIME,
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
            cdump: 0,
            dump_accum: 0.0,
        };
        building.update_draw_position();
        building
    }

    pub fn from_tile(tile_x: i32, tile_y: i32, block: &Block, team: i32, rotation: i32) -> Self {
        Self::new(point2_pack(tile_x, tile_y), block, team, rotation)
    }

    pub fn tile_x(&self) -> i32 {
        point2_x(self.tile_pos) as i32
    }

    pub fn tile_y(&self) -> i32 {
        point2_y(self.tile_pos) as i32
    }

    pub fn set_tile_pos(&mut self, tile_pos: i32) {
        self.tile_pos = tile_pos;
        self.update_draw_position();
    }

    pub fn update_draw_position(&mut self) {
        self.x = self.tile_x() as f32 * TILE_SIZE as f32 + self.block.offset;
        self.y = self.tile_y() as f32 * TILE_SIZE as f32 + self.block.offset;
    }

    pub fn set_block(&mut self, block: &Block) {
        self.block = block.clone();
        self.max_health = block.health as f32;
        self.health = self.health.min(self.max_health);
        self.items = block.has_items.then(ItemModule::default);
        self.liquids = block.has_liquids.then(LiquidModule::default);
        self.power = block.has_power.then(PowerModule::default);
        self.should_consume_power = block.has_power;
        self.update_draw_position();
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        self.rotation = self.block.plan_rotation(rotation);
    }

    pub fn as_ref(&self) -> BuildingRef {
        BuildingRef {
            tile_pos: self.tile_pos,
            block: self.block.id,
            team: self.team,
            rotation: self.rotation,
        }
    }

    pub fn rotdeg(&self) -> f32 {
        self.rotation as f32 * 90.0
    }

    pub fn drawrot(&self) -> f32 {
        if self.block.rotate && self.block.rotate_draw {
            self.rotdeg() + self.block.visual_rotation_offset
        } else {
            0.0
        }
    }

    pub fn hit_size(&self) -> f32 {
        self.block.size as f32 * TILE_SIZE as f32
    }

    pub fn nearby_pos(&self, dx: i32, dy: i32) -> i32 {
        point2_pack(self.tile_x() + dx, self.tile_y() + dy)
    }

    pub fn nearby_rotation_pos(&self, rotation: i32) -> Option<i32> {
        let (dx, dy) = d4(rotation)?;
        Some(self.nearby_pos(dx, dy))
    }

    pub fn front_pos(&self) -> i32 {
        self.side_pos(self.rotation)
    }

    pub fn back_pos(&self) -> i32 {
        self.side_pos(self.rotation + 2)
    }

    pub fn left_pos(&self) -> i32 {
        self.side_pos(self.rotation + 1)
    }

    pub fn right_pos(&self) -> i32 {
        self.side_pos(self.rotation + 3)
    }

    fn side_pos(&self, rotation: i32) -> i32 {
        let trns = self.block.size / 2 + 1;
        let (dx, dy) = d4(rotation).unwrap_or((0, 0));
        self.nearby_pos(dx * trns, dy * trns)
    }

    pub fn relative_to_point(&self, cx: i32, cy: i32) -> i8 {
        let x = self.tile_x();
        let y = self.tile_y();
        if self.block.size % 2 == 1 {
            relative_to_absolute(x as f32, y as f32, cx as f32, cy as f32)
        } else {
            relative_to_absolute(x as f32 + 0.5, y as f32 + 0.5, cx as f32, cy as f32)
        }
    }

    pub fn module_bitmask(&self) -> u8 {
        (self.items.is_some() as u8)
            | ((self.power.is_some() as u8) << 1)
            | ((self.liquids.is_some() as u8) << 2)
            | (1 << 3)
            | (((self.time_scale - 1.0).abs() > f32::EPSILON) as u8) << 4
            | ((self.last_disabler.is_some() as u8) << 5)
    }

    pub fn write_base<W: Write>(&self, write: &mut W, fog_enabled: bool) -> io::Result<()> {
        let write_visibility = fog_enabled && self.visible_flags != 0;

        write_f32(write, self.health)?;
        write_u8(write, (self.rotation.rem_euclid(128) as u8) | 0b1000_0000)?;
        write_u8(write, self.team as u8)?;
        write_u8(write, if write_visibility { 4 } else { 3 })?;
        write_u8(write, self.enabled as u8)?;
        write_u8(write, self.module_bitmask())?;

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

        write_u8(write, (clamp01(self.efficiency) * 255.0) as u8)?;
        write_u8(write, (clamp01(self.optional_efficiency) * 255.0) as u8)?;

        if write_visibility {
            write_i64(write, self.visible_flags as i64)?;
        }

        Ok(())
    }

    pub fn read_base<R: Read>(&mut self, read: &mut R) -> io::Result<()> {
        self.health = read_f32(read)?.min(self.block.health as f32);
        let rot = read_u8(read)?;
        self.team = read_u8(read)? as i32;
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
            self.items
                .get_or_insert_with(ItemModule::default)
                .read(read, legacy)?;
        }
        if (module_bits & (1 << 1)) != 0 {
            self.power
                .get_or_insert_with(PowerModule::default)
                .read(read)?;
        }
        if (module_bits & (1 << 2)) != 0 {
            self.liquids
                .get_or_insert_with(LiquidModule::default)
                .read(read, legacy)?;
        }

        if (module_bits & (1 << 4)) != 0 {
            self.time_scale = read_f32(read)?;
            self.time_scale_duration = read_f32(read)?;
        } else {
            self.time_scale = 1.0;
            self.time_scale_duration = 0.0;
        }

        self.last_disabler = if (module_bits & (1 << 5)) != 0 {
            Some(BuildingRef {
                tile_pos: read_i32(read)?,
                block: 0,
                team: -1,
                rotation: 0,
            })
        } else {
            None
        };

        if version <= 2 {
            let _ = read_bool(read)?;
        }

        if version >= 3 {
            self.efficiency = read_u8(read)? as f32 / 255.0;
            self.potential_efficiency = self.efficiency;
            self.optional_efficiency = read_u8(read)? as f32 / 255.0;
        }

        self.visible_flags = if version == 4 {
            read_i64(read)? as u64
        } else {
            0
        };

        Ok(())
    }

    pub fn should_consume(&self) -> bool {
        self.enabled
    }

    pub fn production_valid(&self) -> bool {
        true
    }

    pub fn status(&self, tick: f32) -> BlockStatus {
        if !self.enabled {
            return BlockStatus::LogicDisable;
        }
        if !self.should_consume() {
            return BlockStatus::NoOutput;
        }
        if self.efficiency <= 0.0 || !self.production_valid() {
            return BlockStatus::NoInput;
        }
        if (tick / 30.0) % 1.0 < self.efficiency {
            BlockStatus::Active
        } else {
            BlockStatus::NoInput
        }
    }

    pub fn allow_update(&self, env: u32, in_bounds: bool) -> bool {
        self.team != 0 && self.block.supports_env(env) && in_bounds
    }

    pub fn check_allow_update(&self, env: u32, in_bounds: bool) -> bool {
        self.enabled && !self.dead && self.initialized && self.allow_update(env, in_bounds)
    }

    pub fn should_update_tile(&self) -> bool {
        self.enabled || !self.block.no_update_disabled
    }

    pub fn configure(&mut self, value: TypeValue) {
        self.block.set_last_config(value.clone());
        self.set_config_value(value);
    }

    pub fn config_value(&self) -> TypeValue {
        self.config.clone().unwrap_or(TypeValue::Null)
    }

    pub fn set_config_value(&mut self, value: TypeValue) {
        self.config = match value {
            TypeValue::Null => None,
            value => Some(value),
        };
    }

    pub fn clear_config(&mut self) {
        self.config = None;
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

    pub fn advance_update_timing(&mut self, delta: f32) {
        let delta = if delta.is_finite() {
            delta.max(0.0)
        } else {
            0.0
        };
        self.time_scale_duration -= delta;
        if self.time_scale_duration <= 0.0 || !self.block.can_overdrive {
            self.time_scale = 1.0;
            self.time_scale_duration = self.time_scale_duration.max(0.0);
        }
    }

    pub fn apply_heal_suppression(&mut self, now: f32, duration: f32) {
        self.heal_suppression_time = self.heal_suppression_time.max(now + duration);
    }

    pub fn is_heal_suppressed(&self, now: f32) -> bool {
        self.block.suppressable && now <= self.heal_suppression_time
    }

    pub fn damage(&mut self, amount: f32, now: f32) {
        if self.dead || amount <= 0.0 {
            return;
        }
        self.health = (self.health - amount).max(0.0);
        self.last_damage_time = now;
        if self.health <= 0.0 {
            self.dead = true;
        }
    }

    pub fn heal(&mut self, amount: f32, now: f32) {
        if self.dead || amount <= 0.0 || self.is_heal_suppressed(now) {
            return;
        }
        self.health = (self.health + amount).min(self.max_health);
        self.last_heal_time = now;
    }

    pub fn recently_damaged(&self, now: f32) -> bool {
        now <= self.last_damage_time + BUILDING_RECENT_DAMAGE_TIME
    }

    pub fn recently_healed(&self, now: f32) -> bool {
        now <= self.last_heal_time + BUILDING_RECENT_HEAL_TIME
    }

    pub fn kill(&mut self) {
        self.dead = true;
        self.health = 0.0;
    }

    pub fn change_team(&mut self, team: i32) {
        self.team = team;
        self.was_visible = false;
        self.last_disabler = None;
    }

    pub fn sleep(&mut self, delta: f32) {
        self.sleep_time += delta.max(0.0);
        if !self.sleeping && self.sleep_time >= BUILDING_TIME_TO_SLEEP {
            self.sleeping = true;
        }
    }

    pub fn no_sleep(&mut self) {
        self.sleep_time = 0.0;
        self.sleeping = false;
    }

    pub fn active(&self) -> bool {
        self.enabled && !self.sleeping && !self.dead
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

fn d4(rotation: i32) -> Option<(i32, i32)> {
    match rotation.rem_euclid(4) {
        0 => Some((1, 0)),
        1 => Some((0, 1)),
        2 => Some((-1, 0)),
        3 => Some((0, -1)),
        _ => None,
    }
}

fn relative_to_absolute(x: f32, y: f32, cx: f32, cy: f32) -> i8 {
    if (x - cx).abs() > (y - cy).abs() {
        if x <= cx - 1.0 {
            return 0;
        }
        if x >= cx + 1.0 {
            return 2;
        }
    } else {
        if y <= cy - 1.0 {
            return 1;
        }
        if y >= cy + 1.0 {
            return 3;
        }
    }
    -1
}

fn clamp01(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::mindustry::{
        content::ContentCatalog,
        io::TypeValue,
        world::{meta::Env, point2_pack, Block},
    };

    use super::*;

    fn runtime_block() -> Block {
        let mut block = Block::new(5, "router");
        block.health = 100;
        block.has_items = true;
        block.has_liquids = true;
        block.has_power = true;
        block.update = true;
        block.rotate = true;
        block
    }

    #[test]
    fn building_initializes_draw_position_modules_and_side_positions() {
        let mut block = runtime_block();
        block.size = 2;
        block.derive_layout_fields();
        let building = Building::new(point2_pack(3, 4), &block, 2, 5);

        assert_eq!((building.tile_x(), building.tile_y()), (3, 4));
        assert_eq!((building.x, building.y), (28.0, 36.0));
        assert_eq!(building.rotation, 1);
        assert_eq!(building.drawrot(), 90.0);
        assert_eq!(building.hit_size(), 16.0);
        assert!(building.items.is_some());
        assert!(building.liquids.is_some());
        assert!(building.power.is_some());
        assert_eq!(building.front_pos(), point2_pack(3, 6));
        assert_eq!(building.back_pos(), point2_pack(3, 2));
        assert_eq!(building.left_pos(), point2_pack(1, 4));
        assert_eq!(building.right_pos(), point2_pack(5, 4));
    }

    #[test]
    fn building_base_io_roundtrips_java_v3_layout() {
        let block = runtime_block();
        let mut building = Building::new(point2_pack(3, 4), &block, 2, 3);
        building.health = 42.5;
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
        assert_eq!(bytes[6], 3);

        let mut restored = Building::new(point2_pack(3, 4), &block, 0, 0);
        restored.read_base(&mut bytes.as_slice()).unwrap();

        assert_eq!(restored.health, 42.5);
        assert_eq!(restored.rotation, 3);
        assert_eq!(restored.team, 2);
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
    fn building_base_io_can_include_java_v4_visibility_and_disabler() {
        let block = runtime_block();
        let mut building = Building::new(point2_pack(1, 1), &block, 1, 0);
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

        let mut restored = Building::new(point2_pack(1, 1), &block, 0, 0);
        restored.read_base(&mut bytes.as_slice()).unwrap();

        assert_eq!(restored.visible_flags, 0x0102_0304_0506_0708);
        assert_eq!(
            restored.last_disabler.map(|build| build.tile_pos),
            Some(point2_pack(7, 8))
        );
    }

    #[test]
    fn building_status_damage_config_and_sleep_are_runtime_pure() {
        let mut block = runtime_block();
        block.suppressable = true;
        let mut building = Building::new(point2_pack(2, 2), &block, 1, 0);

        assert_eq!(building.status(0.0), BlockStatus::Active);
        building.enabled = false;
        assert_eq!(building.status(0.0), BlockStatus::LogicDisable);
        building.enabled = true;
        building.efficiency = 0.0;
        assert_eq!(building.status(0.0), BlockStatus::NoInput);

        building.configure(TypeValue::String("cfg".into()));
        assert_eq!(building.config_value(), TypeValue::String("cfg".into()));
        assert_eq!(building.block.next_config(), None);
        building.block.save_config = true;
        assert_eq!(
            building.block.next_config(),
            Some(TypeValue::String("cfg".into()))
        );
        building.clear_config();
        assert_eq!(building.config_value(), TypeValue::Null);

        building.damage(25.0, 100.0);
        assert_eq!(building.health, 75.0);
        assert!(building.recently_damaged(100.0));
        building.apply_heal_suppression(100.0, 20.0);
        building.heal(10.0, 110.0);
        assert_eq!(building.health, 75.0);
        building.heal(10.0, 121.0);
        assert_eq!(building.health, 85.0);
        assert!(building.recently_healed(121.0));

        building.sleep(59.0);
        assert!(!building.sleeping);
        building.sleep(1.0);
        assert!(building.sleeping);
        building.no_sleep();
        assert!(building.active());
    }

    #[test]
    fn building_uses_content_registry_block_and_save_base_without_subclass_data() {
        let catalog = ContentCatalog::load_base_content();
        let router = catalog
            .blocks
            .get_by_name("router")
            .expect("base content registry should contain router")
            .base();

        let mut building = Building::new(point2_pack(10, 11), router, 1, 0);
        building.health = (router.health as f32).min(12.0);

        let mut bytes = Vec::new();
        building.write_base(&mut bytes, false).unwrap();

        let mut restored = Building::new(point2_pack(10, 11), router, 0, 0);
        restored.read_base(&mut bytes.as_slice()).unwrap();

        assert_eq!(restored.block.id, router.id);
        assert_eq!(restored.block.name, router.name);
        assert_eq!(restored.team, 1);
        assert!(restored.block.supports_env(Env::TERRESTRIAL));
    }
}
