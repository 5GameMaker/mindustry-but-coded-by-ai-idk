//! Building component shell mirroring upstream `mindustry.entities.comp.BuildingComp`.
//!
//! This first-stage port keeps the building as a mostly pure data object with
//! tile/world helpers, module snapshots and small state transitions. The heavy
//! Java runtime (`World`, `Events`, `Call`, `PowerGraph`, rendering, logic and
//! network side effects) is intentionally kept out for now.

use std::io::{self, Read, Write};

use crate::mindustry::ctype::ContentType;
use crate::mindustry::entities::{EntityPosition, SizedEntity};
use crate::mindustry::game::{BlockPlan, Teams};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingConfigKind {
    Null,
    Int,
    Long,
    Float,
    String,
    Content(ContentType),
    AnyContent,
    TechNode,
    Bool,
    Double,
    Building,
    LogicAccess,
    Unit,
    Point2,
    Vec2,
    Team,
    UnitCommand,
    IntSeq,
    IntArray,
    ByteArray,
    Point2Array,
    BoolArray,
    Vec2Array,
    ObjectArray,
}

impl BuildingConfigKind {
    pub fn of(value: &TypeValue) -> Self {
        match value {
            TypeValue::Null => Self::Null,
            TypeValue::Int(_) => Self::Int,
            TypeValue::Long(_) => Self::Long,
            TypeValue::Float(_) => Self::Float,
            TypeValue::String(_) => Self::String,
            TypeValue::Content(content) => Self::Content(content.content_type),
            TypeValue::TechNode(_) => Self::TechNode,
            TypeValue::Bool(_) => Self::Bool,
            TypeValue::Double(_) => Self::Double,
            TypeValue::Building(_) => Self::Building,
            TypeValue::LogicAccess(_) => Self::LogicAccess,
            TypeValue::Unit(_) => Self::Unit,
            TypeValue::Point2(_) => Self::Point2,
            TypeValue::Vec2(_) => Self::Vec2,
            TypeValue::Team(_) => Self::Team,
            TypeValue::UnitCommand(_) => Self::UnitCommand,
            TypeValue::IntSeq(_) => Self::IntSeq,
            TypeValue::IntArray(_) => Self::IntArray,
            TypeValue::ByteArray(_) => Self::ByteArray,
            TypeValue::Point2Array(_) => Self::Point2Array,
            TypeValue::BoolArray(_) => Self::BoolArray,
            TypeValue::Vec2Array(_) => Self::Vec2Array,
            TypeValue::ObjectArray(_) => Self::ObjectArray,
        }
    }

    pub fn matches(self, value: &TypeValue) -> bool {
        matches!(self, Self::AnyContent)
            && matches!(value, TypeValue::Content(_) | TypeValue::TechNode(_))
            || self == Self::of(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildingConfigRollback {
    pub tile_pos: i32,
    pub value: TypeValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildingConfigChange {
    pub accepted: bool,
    pub requested: TypeValue,
    pub previous: TypeValue,
    pub current: TypeValue,
    pub kind: BuildingConfigKind,
    pub last_accessed: Option<String>,
    pub rollback: Option<BuildingConfigRollback>,
}

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
    pub cdump: usize,
    pub dump_accum: f32,
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
            cdump: 0,
            dump_accum: 0.0,
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

    pub fn advance_update_timing(&mut self, delta: f32, can_overdrive: bool) {
        let delta = if delta.is_finite() { delta } else { 0.0 };
        self.time_scale_duration -= delta;
        if self.time_scale_duration <= 0.0 || !can_overdrive {
            self.time_scale = 1.0;
        }
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

    pub fn should_update_tile(&self, no_update_disabled: bool) -> bool {
        self.enabled || !no_update_disabled
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

    pub fn rebuild_plan(&self) -> BlockPlan {
        BlockPlan::new(
            self.tile_x(),
            self.tile_y(),
            self.rotation as i16,
            self.block.name.clone(),
            config_string(&self.config_value()),
        )
    }

    pub fn add_rebuild_plan(
        &self,
        teams: &mut Teams,
        check_previous: bool,
        ignore_conditions: bool,
        rebuildable: bool,
        visible: bool,
        default_team: TeamId,
        campaign: bool,
    ) -> Option<BlockPlan> {
        if !ignore_conditions
            && (!rebuildable || (self.team == default_team && campaign && !visible))
        {
            return None;
        }
        teams.add_plan_front(self.team.0, self.rebuild_plan(), check_previous)
    }

    pub fn configure_any(&mut self, value: TypeValue) -> BuildingConfigChange {
        self.configure_any_checked(value, |_| true)
    }

    pub fn configure_any_checked<F>(&mut self, value: TypeValue, accepts: F) -> BuildingConfigChange
    where
        F: FnOnce(&TypeValue) -> bool,
    {
        self.configure_any_checked_accessed(value, accepts, Option::<String>::None)
    }

    pub fn configure_any_checked_accessed<F>(
        &mut self,
        value: TypeValue,
        accepts: F,
        last_accessed: Option<impl Into<String>>,
    ) -> BuildingConfigChange
    where
        F: FnOnce(&TypeValue) -> bool,
    {
        let previous = self.config_value();
        let kind = BuildingConfigKind::of(&value);
        let accepted = accepts(&value);
        let last_accessed = last_accessed.map(Into::into);

        if accepted {
            self.set_config_value(value.clone());
            if let Some(last_accessed) = &last_accessed {
                self.last_accessed = last_accessed.clone();
            }
            let current = self.config_value();
            BuildingConfigChange {
                accepted: true,
                requested: value,
                previous,
                current,
                kind,
                last_accessed,
                rollback: None,
            }
        } else {
            BuildingConfigChange {
                accepted: false,
                requested: value,
                previous: previous.clone(),
                current: previous.clone(),
                kind,
                last_accessed: None,
                rollback: Some(BuildingConfigRollback {
                    tile_pos: self.tile_pos,
                    value: previous,
                }),
            }
        }
    }

    pub fn configure_any_allowed_kinds(
        &mut self,
        value: TypeValue,
        allowed: impl IntoIterator<Item = BuildingConfigKind>,
    ) -> BuildingConfigChange {
        let allowed = allowed.into_iter().collect::<Vec<_>>();
        self.configure_any_checked(value, |value| {
            allowed.iter().any(|kind| kind.matches(value))
        })
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

fn config_string(value: &TypeValue) -> Option<String> {
    match value {
        TypeValue::Null => None,
        TypeValue::Int(value) => Some(value.to_string()),
        TypeValue::Long(value) => Some(value.to_string()),
        TypeValue::Float(value) => Some(value.to_string()),
        TypeValue::String(value) => Some(value.clone()),
        TypeValue::Content(value) | TypeValue::TechNode(value) => Some(format!("{value:?}")),
        TypeValue::Bool(value) => Some(value.to_string()),
        TypeValue::Double(value) => Some(value.to_string()),
        TypeValue::Building(value) => Some(value.to_string()),
        TypeValue::LogicAccess(value) => Some(format!("{value:?}")),
        TypeValue::Unit(value) => Some(value.to_string()),
        TypeValue::Point2(value) => Some(format!("{},{}", value.x, value.y)),
        TypeValue::Vec2(value) => Some(format!("{},{}", value.x, value.y)),
        TypeValue::Team(value) => Some(value.to_string()),
        TypeValue::UnitCommand(value) => Some(value.to_string()),
        TypeValue::IntSeq(values) | TypeValue::IntArray(values) => Some(format!("{values:?}")),
        TypeValue::ByteArray(values) => Some(format!("{values:?}")),
        TypeValue::Point2Array(values) => Some(format!("{values:?}")),
        TypeValue::BoolArray(values) => Some(format!("{values:?}")),
        TypeValue::Vec2Array(values) => Some(format!("{values:?}")),
        TypeValue::ObjectArray(values) => Some(format!("{values:?}")),
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
    use crate::mindustry::io::{ContentRef, Point2};

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
    fn building_update_timing_matches_java_timescale_duration_gate() {
        let mut building = BuildingComp::new(point2_pack(1, 1), block(), TeamId(1));
        building.apply_boost(2.0, 45.0);

        building.advance_update_timing(15.0, true);
        assert_eq!(building.time_scale, 2.0);
        assert_eq!(building.time_scale_duration, 30.0);

        building.advance_update_timing(30.0, true);
        assert_eq!(building.time_scale, 1.0);
        assert_eq!(building.time_scale_duration, 0.0);

        building.apply_boost(3.0, 60.0);
        building.advance_update_timing(1.0, false);
        assert_eq!(building.time_scale, 1.0);
        assert_eq!(building.time_scale_duration, 59.0);
    }

    #[test]
    fn building_update_tile_gate_matches_java_no_update_disabled() {
        let mut building = BuildingComp::new(point2_pack(1, 1), block(), TeamId(1));

        building.enabled = true;
        assert!(building.should_update_tile(false));
        assert!(building.should_update_tile(true));

        building.enabled = false;
        assert!(building.should_update_tile(false));
        assert!(!building.should_update_tile(true));
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
    fn building_component_config_value_uses_java_null_sentinel() {
        let mut building = BuildingComp::new(point2_pack(2, 2), block(), TeamId(1));

        assert_eq!(building.config_value(), TypeValue::Null);

        building.set_config_value(TypeValue::String("alpha".into()));
        assert_eq!(building.config_value(), TypeValue::String("alpha".into()));

        building.set_config_value(TypeValue::Null);
        assert!(building.config.is_none());
        assert_eq!(building.config_value(), TypeValue::Null);
    }

    #[test]
    fn building_component_configure_any_preserves_value_and_access() {
        let mut building = BuildingComp::new(point2_pack(4, 4), block(), TeamId(1));
        let value = TypeValue::Point2(Point2::new(7, 9));

        let change = building.configure_any_checked_accessed(
            value.clone(),
            |value| BuildingConfigKind::Point2.matches(value),
            Some("[#ffaa00]frog"),
        );

        assert!(change.accepted);
        assert_eq!(change.kind, BuildingConfigKind::Point2);
        assert_eq!(change.previous, TypeValue::Null);
        assert_eq!(change.current, value);
        assert_eq!(change.rollback, None);
        assert_eq!(building.config_value(), change.current);
        assert_eq!(building.last_accessed, "[#ffaa00]frog");
    }

    #[test]
    fn building_component_add_rebuild_plan_matches_plain_java_add_plan_branch() {
        let mut building = BuildingComp::new(point2_pack(2, 3), block(), TeamId(1));
        building.set_rotation(2);
        building.set_config_value(TypeValue::String("cfg".into()));
        let mut teams = Teams::default();
        teams.replace_plans([(
            1,
            vec![
                BlockPlan::new(2, 3, 0, "old", Some("old-cfg".into())),
                BlockPlan::new(4, 5, 1, "duo", None),
            ],
        )]);

        let removed =
            building.add_rebuild_plan(&mut teams, true, false, true, true, TeamId(1), false);

        assert_eq!(
            removed,
            Some(BlockPlan::new(2, 3, 0, "old", Some("old-cfg".into())))
        );
        assert_eq!(
            teams.get_or_null(1).unwrap().plans,
            vec![
                BlockPlan::new(2, 3, 2, "router", Some("cfg".into())),
                BlockPlan::new(4, 5, 1, "duo", None),
            ]
        );
    }

    #[test]
    fn building_component_add_rebuild_plan_respects_rebuild_visibility_gate() {
        let building = BuildingComp::new(point2_pack(6, 7), block(), TeamId(1));
        let mut teams = Teams::default();

        assert_eq!(
            building.add_rebuild_plan(&mut teams, false, false, false, true, TeamId(1), false),
            None
        );
        assert!(teams
            .get_or_null(1)
            .is_none_or(|team| team.plans.is_empty()));

        assert_eq!(
            building.add_rebuild_plan(&mut teams, false, false, true, false, TeamId(1), true),
            None
        );
        assert!(teams
            .get_or_null(1)
            .is_none_or(|team| team.plans.is_empty()));

        building.add_rebuild_plan(&mut teams, false, true, false, false, TeamId(1), true);
        assert_eq!(
            teams.get_or_null(1).unwrap().plans,
            vec![BlockPlan::new(6, 7, 0, "router", None)]
        );
    }

    #[test]
    fn building_component_rejected_config_returns_rollback_without_mutation() {
        let mut building = BuildingComp::new(point2_pack(5, 6), block(), TeamId(2));
        building.set_config_value(TypeValue::String("old".into()));

        let change =
            building.configure_any_allowed_kinds(TypeValue::Int(99), [BuildingConfigKind::String]);

        assert!(!change.accepted);
        assert_eq!(change.kind, BuildingConfigKind::Int);
        assert_eq!(change.previous, TypeValue::String("old".into()));
        assert_eq!(change.current, TypeValue::String("old".into()));
        assert_eq!(building.config_value(), TypeValue::String("old".into()));
        assert_eq!(
            change.rollback,
            Some(BuildingConfigRollback {
                tile_pos: point2_pack(5, 6),
                value: TypeValue::String("old".into()),
            })
        );
    }

    #[test]
    fn building_component_allowed_kinds_distinguish_content_types() {
        let mut building = BuildingComp::new(point2_pack(7, 8), block(), TeamId(1));
        let item = TypeValue::Content(ContentRef::new(ContentType::Item, 3));
        let block_config = TypeValue::Content(ContentRef::new(ContentType::Block, 4));

        let rejected = building.configure_any_allowed_kinds(
            item.clone(),
            [BuildingConfigKind::Content(ContentType::Block)],
        );
        assert!(!rejected.accepted);
        assert_eq!(building.config_value(), TypeValue::Null);

        let accepted =
            building.configure_any_allowed_kinds(item.clone(), [BuildingConfigKind::AnyContent]);
        assert!(accepted.accepted);
        assert_eq!(building.config_value(), item);

        let block_accepted = building.configure_any_allowed_kinds(
            block_config.clone(),
            [BuildingConfigKind::Content(ContentType::Block)],
        );
        assert!(block_accepted.accepted);
        assert_eq!(building.config_value(), block_config);
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
