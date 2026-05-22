//! Unit component shell mirroring upstream `mindustry.entities.comp.UnitComp`.
//!
//! The Java component is the central runtime hub for movement, controllers,
//! health, inventory, weapons, building, mining and payload behavior. This
//! Rust stage keeps a compositional shell around already-ported components and
//! migrates pure helper behavior first; global `Vars`/`World` side effects are
//! left to explicit snapshots or future runtime adapters.

use crate::mindustry::core::world::World;
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::entities::{EntityPosition, SizedEntity};
use crate::mindustry::io::type_io::{CommandWire, ControllerWire};
use crate::mindustry::io::{AbilityWire, TeamId, Vec2};
use crate::mindustry::logic::{
    LAccess, LOGIC_CTRL_COMMAND, LOGIC_CTRL_PLAYER, LOGIC_CTRL_PROCESSOR,
};
use crate::mindustry::r#type::UnitType;

use super::builder::BuilderComp;
use super::entity::EntityComp;
use super::health::HealthComp;
use super::hitbox::HitboxComp;
use super::items::ItemsComp;
use super::miner::{MineTile, MinerComp, MinerType};
use super::payload::PayloadComp;
use super::physics::PhysicsComp;
use super::rot::RotComp;
use super::shield::ShieldComp;
use super::status::StatusComp;
use super::sync::SyncComp;
use super::team::TeamComp;
use super::vel::VelComp;
use super::weapons::WeaponsComp;

const TILE_SIZE: f32 = 8.0;
const TILE_PAYLOAD: f32 = TILE_SIZE * TILE_SIZE;

#[derive(Debug, Clone, PartialEq)]
pub enum UnitControllerState {
    Ground,
    Player { player_id: i32 },
    LegacyFormation { id: i32 },
    Logic { controller_pos: i32 },
    Assembler,
    Command(CommandWire),
    Unknown,
}

impl UnitControllerState {
    pub fn to_wire(&self) -> ControllerWire {
        match self {
            Self::Ground | Self::Unknown => ControllerWire::Ground,
            Self::Player { player_id } => ControllerWire::Player {
                player_id: *player_id,
            },
            Self::LegacyFormation { id } => ControllerWire::LegacyFormation { id: *id },
            Self::Logic { controller_pos } => ControllerWire::Logic {
                controller_pos: *controller_pos,
            },
            Self::Assembler => ControllerWire::Assembler,
            Self::Command(command) => ControllerWire::Command(command.clone()),
        }
    }

    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player { .. })
    }

    pub fn is_logic(&self) -> bool {
        matches!(self, Self::Logic { .. })
    }

    pub fn is_commandable(&self) -> bool {
        matches!(self, Self::Command(_))
    }

    pub fn controlled_code(&self, is_valid: bool) -> i32 {
        if !is_valid {
            return 0;
        }

        match self {
            Self::Logic { .. } => LOGIC_CTRL_PROCESSOR,
            Self::Player { .. } => LOGIC_CTRL_PLAYER,
            Self::Command(_) => LOGIC_CTRL_COMMAND,
            _ => 0,
        }
    }
}

impl Default for UnitControllerState {
    fn default() -> Self {
        Self::Ground
    }
}

impl From<ControllerWire> for UnitControllerState {
    fn from(value: ControllerWire) -> Self {
        match value {
            ControllerWire::Player { player_id } => Self::Player { player_id },
            ControllerWire::LegacyFormation { id } => Self::LegacyFormation { id },
            ControllerWire::Ground => Self::Ground,
            ControllerWire::Logic { controller_pos } => Self::Logic { controller_pos },
            ControllerWire::Assembler => Self::Assembler,
            ControllerWire::Command(command) => Self::Command(command),
        }
    }
}

impl From<UnitControllerState> for ControllerWire {
    fn from(value: UnitControllerState) -> Self {
        value.to_wire()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitCollisionLayer {
    Legs,
    Ground,
    Flying,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitTrailState {
    pub width: f32,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFloorSnapshot {
    pub name: String,
    pub speed_multiplier: f32,
    pub is_liquid: bool,
    pub drown_time: f32,
}

impl UnitFloorSnapshot {
    pub fn new(name: impl Into<String>, speed_multiplier: f32) -> Self {
        Self {
            name: name.into(),
            speed_multiplier,
            is_liquid: false,
            drown_time: 0.0,
        }
    }

    pub fn liquid(mut self, drown_time: f32) -> Self {
        self.is_liquid = true;
        self.drown_time = drown_time;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitComp {
    pub entity: EntityComp,
    pub sync: SyncComp,
    /// Canonical position and velocity.
    pub vel: VelComp,
    /// Canonical rotation.
    pub rot: RotComp,
    pub health: HealthComp,
    pub shield: ShieldComp,
    pub status: StatusComp,
    pub team: TeamComp,
    pub hitbox: HitboxComp,
    pub physics: PhysicsComp,
    pub items: ItemsComp,
    pub weapons: WeaponsComp,
    pub builder: BuilderComp,
    pub miner: MinerComp,
    pub payload: Option<PayloadComp>,
    pub type_info: UnitType,
    pub controller: UnitControllerState,
    pub abilities: Vec<AbilityWire>,
    pub spawned_by_core: bool,
    pub flag: f64,
    pub elevation: f32,
    pub trail: Option<UnitTrailState>,
    pub docked_type: Option<String>,
    pub last_commanded: Option<String>,
    pub shadow_alpha: f32,
    pub heal_time: f32,
    pub last_fog_pos: i32,
    pub has_target: bool,
    pub resupply_time: f32,
    pub was_player: bool,
    pub was_healed: bool,
    pub was_flying: bool,
    pub drown_time: f32,
    pub splash_timer: f32,
    pub last_drown_floor: Option<UnitFloorSnapshot>,
}

impl UnitComp {
    pub fn new(id: i32, type_info: UnitType, team: TeamId) -> Self {
        let mut unit = Self {
            entity: EntityComp::with_id(id, 0, true),
            sync: SyncComp::default(),
            vel: VelComp::new(0.0, 0.0),
            rot: RotComp::default(),
            health: HealthComp::new(type_info.health),
            shield: ShieldComp::new(type_info.health),
            status: StatusComp::new(),
            team: TeamComp::new(0.0, 0.0, team),
            hitbox: HitboxComp::new(0.0, 0.0, type_info.hit_size),
            physics: PhysicsComp::new(type_info.hit_size, 0.0, 0.0),
            items: ItemsComp::new(unit_item_capacity(&type_info)),
            weapons: WeaponsComp::new(unit_ammo_capacity(&type_info), type_info.aim_dst),
            builder: BuilderComp::new(type_info.clone(), team),
            miner: MinerComp::new(miner_type_from_unit_type(&type_info)),
            payload: None,
            type_info: type_info.clone(),
            controller: UnitControllerState::Ground,
            abilities: Vec::new(),
            spawned_by_core: false,
            flag: 0.0,
            elevation: 0.0,
            trail: None,
            docked_type: None,
            last_commanded: None,
            shadow_alpha: -1.0,
            heal_time: 0.0,
            last_fog_pos: 0,
            has_target: false,
            resupply_time: 0.0,
            was_player: false,
            was_healed: false,
            was_flying: false,
            drown_time: 0.0,
            splash_timer: 0.0,
            last_drown_floor: None,
        };

        unit.set_type(type_info);
        unit.health.health = unit.health.max_health;
        unit.shield.health = unit.health.health;
        unit.refresh_component_views();
        unit
    }

    pub fn id(&self) -> i32 {
        self.entity.id
    }

    pub fn x(&self) -> f32 {
        self.vel.pos.x
    }

    pub fn y(&self) -> f32 {
        self.vel.pos.y
    }

    pub fn rotation(&self) -> f32 {
        self.rot.rotation
    }

    pub fn team_id(&self) -> TeamId {
        self.team.team
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.vel.pos.set(x, y);
        self.refresh_component_views();
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rot.rotation = rotation.rem_euclid(360.0);
        self.refresh_component_views();
    }

    pub fn set_controller(&mut self, controller: UnitControllerState) {
        self.controller = controller;
        self.refresh_component_views();
    }

    pub fn reset_controller(&mut self) {
        self.set_controller(UnitControllerState::Ground);
    }

    pub fn set_type(&mut self, type_info: UnitType) {
        self.health.max_health = type_info.health;
        self.vel.drag = type_info.drag * self.status.drag_multiplier;
        self.shield.armor = type_info.armor;
        self.hitbox.hit_size = type_info.hit_size;
        self.physics.hit_size = type_info.hit_size;
        self.items.set_item_capacity(unit_item_capacity(&type_info));
        self.weapons.ammo_capacity = unit_ammo_capacity(&type_info);
        self.weapons.aim_dst = type_info.aim_dst;

        if self.weapons.mounts.len() != type_info.weapons.len() {
            self.weapons
                .setup_weapons(type_info.weapons.iter().cloned());
        }

        if self.abilities.len() != type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); type_info.abilities.len()];
        }

        self.builder.type_info = type_info.clone();
        self.miner.type_info = miner_type_from_unit_type(&type_info);
        self.status.type_speed = type_info.speed;
        self.status.type_build_speed = type_info.build_speed;
        self.status.type_drag = type_info.drag;
        self.status.max_health = type_info.health;
        self.status.immunities = type_info.immunities.iter().cloned().collect();

        if let Some(payload) = &mut self.payload {
            payload.payload_capacity = type_info.payload_capacity;
            payload.pickup_units = type_info.pickup_units;
        }

        self.type_info = type_info;
        self.refresh_component_views();
    }

    pub fn refresh_component_views(&mut self) {
        let x = self.x();
        let y = self.y();
        let rotation = self.rotation();
        let team = self.team.team;
        let hit_size = self.type_info.hit_size;

        self.team.pos.set(x, y);
        self.hitbox.x = x;
        self.hitbox.y = y;
        self.hitbox.hit_size = hit_size;
        self.physics.x = x;
        self.physics.y = y;
        self.physics.hit_size = hit_size;
        self.physics.vel = self.vel.vel;
        self.vel.drag = self.type_info.drag * self.status.drag_multiplier;

        self.shield.x = x;
        self.shield.y = y;
        self.shield.health = self.health.health;
        self.shield.dead = self.health.dead;
        self.shield.armor = self.type_info.armor;
        self.shield.armor_override = self.status.armor_override;
        self.shield.health_multiplier = self.status.health_multiplier;

        self.builder.x = x;
        self.builder.y = y;
        self.builder.rotation = rotation;
        self.builder.team = team;
        self.builder.build_speed_multiplier = self.status.build_speed_multiplier;

        self.miner.x = x;
        self.miner.y = y;
        self.miner.rotation = rotation;
        self.miner.hit_size = hit_size;
        self.miner.stack_item = self.items.stack.item.clone();
        self.miner.stack_amount = self.items.stack.amount;
        self.miner.item_capacity = self.items.item_capacity();
        self.miner.actively_building = self.builder.is_building();
        self.miner.is_player = self.controller.is_player();

        self.weapons.x = x;
        self.weapons.y = y;
        self.weapons.disarmed = self.status.disarmed;

        if let Some(payload) = &mut self.payload {
            payload.x = x;
            payload.y = y;
            payload.rotation = rotation;
            payload.team = team;
            payload.payload_capacity = self.type_info.payload_capacity;
            payload.pickup_units = self.type_info.pickup_units;
        }
    }

    pub fn after_sync(&mut self) {
        self.set_type(self.type_info.clone());
        self.sync.after_sync();
    }

    pub fn after_read(&mut self) {
        self.set_type(self.type_info.clone());
        self.entity.after_read();
        self.reset_controller();
    }

    pub fn add(&mut self) {
        self.entity.add();
    }

    pub fn remove(&mut self, net_client: bool) -> Option<i32> {
        self.entity.remove();
        self.sync.remove(net_client, self.id())
    }

    pub fn is_valid(&self) -> bool {
        self.health.is_valid(self.entity.is_added())
    }

    pub fn is_grounded(&self) -> bool {
        self.elevation < 0.001
    }

    pub fn is_flying(&self) -> bool {
        self.elevation >= 0.09
    }

    pub fn check_target(&self, target_air: bool, target_ground: bool) -> bool {
        (self.is_grounded() && target_ground) || (self.is_flying() && target_air)
    }

    pub fn can_drown(&self) -> bool {
        self.is_grounded() && self.type_info.can_drown
    }

    pub fn can_drown_on(&self, floor: Option<&UnitFloorSnapshot>) -> bool {
        self.can_drown() && floor.is_some_and(|floor| floor.is_liquid && floor.drown_time > 0.0)
    }

    pub fn physic_size(&self) -> f32 {
        self.hitbox.hit_size * 0.7
    }

    pub fn bounds(&self) -> f32 {
        self.hitbox.hit_size * 2.0
    }

    pub fn range(&self) -> f32 {
        self.type_info.max_range
    }

    pub fn displayable(&self) -> bool {
        self.type_info.hoverable
    }

    pub fn has_weapons(&self) -> bool {
        self.type_info.has_weapons()
    }

    pub fn can_shoot(&self) -> bool {
        !self.status.disarmed && !(self.type_info.can_boost && self.is_flying())
    }

    pub fn is_enemy(&self) -> bool {
        self.type_info.is_enemy
    }

    pub fn killable(&self) -> bool {
        self.type_info.killable_with_payload(self.has_payload())
    }

    pub fn hittable(&self) -> bool {
        self.type_info.hittable_with_payload(self.has_payload())
    }

    pub fn targetable(&self) -> bool {
        self.type_info.targetable_with_payload(self.has_payload())
    }

    pub fn effective_armor(&self) -> f32 {
        if self.status.armor_override >= 0.0 {
            self.status.armor_override
        } else {
            self.type_info.armor
        }
    }

    pub fn item_capacity(&self) -> i32 {
        self.type_info.item_capacity
    }

    pub fn has_payload(&self) -> bool {
        self.payload
            .as_ref()
            .is_some_and(|payload| payload.has_payload())
    }

    pub fn payload_count(&self) -> usize {
        self.payload
            .as_ref()
            .map(|payload| payload.payloads.len())
            .unwrap_or(0)
    }

    pub fn payload_used(&self) -> f32 {
        self.payload
            .as_ref()
            .map(PayloadComp::payload_used)
            .unwrap_or(0.0)
    }

    pub fn payload_capacity_tiles(&self) -> f32 {
        self.type_info.payload_capacity / TILE_PAYLOAD
    }

    pub fn is_building(&self) -> bool {
        self.builder.is_building()
    }

    pub fn build_plan(&self) -> Option<&BuildPlan> {
        self.builder.build_plan()
    }

    pub fn mining(&self) -> bool {
        self.miner.mining()
    }

    pub fn mine_tile(&self) -> Option<&MineTile> {
        self.miner.mine_tile.as_ref()
    }

    pub fn collision_layer(&self) -> UnitCollisionLayer {
        if self.type_info.allow_leg_step && self.type_info.leg_physics_layer {
            UnitCollisionLayer::Legs
        } else if self.is_grounded() {
            UnitCollisionLayer::Ground
        } else {
            UnitCollisionLayer::Flying
        }
    }

    pub fn move_at(&mut self, vector: Vec2, acceleration: f32, delta: f32) {
        let target = vector;
        let diff = Vec2 {
            x: target.x - self.vel.vel.x,
            y: target.y - self.vel.vel.y,
        };
        let limited = vector_limited(diff, acceleration * vector_length(vector) * delta);
        self.vel.vel.x += limited.x;
        self.vel.vel.y += limited.y;
        self.refresh_component_views();
    }

    pub fn move_at_type(&mut self, vector: Vec2, delta: f32) {
        self.move_at(vector, self.type_info.accel, delta);
    }

    pub fn move_pref(&mut self, movement: Vec2, delta: f32) {
        if self.type_info.omni_movement {
            self.move_at_type(movement, delta);
        } else {
            self.rotate_move(movement, delta);
        }
    }

    pub fn rotate_move(&mut self, movement: Vec2, delta: f32) {
        let target = vector_from_angle(self.rotation(), vector_length(movement));
        self.move_at_type(target, delta);

        if vector_length_sq(movement) > 0.000001 {
            let next = move_toward_angle(
                self.rotation(),
                vector_angle(movement),
                self.type_info.rotate_speed * delta * self.status.speed_multiplier,
            );
            self.set_rotation(next);
        }
    }

    pub fn look_at_angle(&mut self, angle: f32, delta: f32) {
        let next = move_toward_angle(
            self.rotation(),
            angle,
            self.type_info.rotate_speed * delta * self.status.speed_multiplier,
        );
        self.set_rotation(next);
    }

    pub fn look_at_xy(&mut self, x: f32, y: f32, delta: f32) {
        self.look_at_angle(angle_to(self.x(), self.y(), x, y), delta);
    }

    pub fn update_boosting(
        &mut self,
        boost: bool,
        event: bool,
        on_solid: bool,
        can_land: bool,
        delta: f32,
    ) -> bool {
        if !self.type_info.can_boost || self.health.dead {
            return false;
        }

        let should_boost = boost || on_solid || (self.is_flying() && !can_land);
        let target = if should_boost { 1.0 } else { 0.0 };
        let amount = if should_boost {
            self.type_info.rise_speed
        } else {
            self.type_info.descent_speed
        };
        self.elevation = approach_delta(self.elevation, target, amount * delta);
        event
    }

    pub fn speed_with_floor(&self, floor_speed_multiplier: f32) -> f32 {
        let strafe_penalty = if self.is_grounded() || !self.controller.is_player() {
            1.0
        } else {
            lerp(
                1.0,
                self.type_info.strafe_penalty,
                angle_dist(vector_angle(self.vel.vel), self.rotation()) / 180.0,
            )
        };
        let boost = lerp(
            1.0,
            if self.type_info.can_boost {
                self.type_info.boost_multiplier
            } else {
                1.0
            },
            self.elevation,
        );
        let floor = floor_speed_multiplier.powf(self.type_info.floor_multiplier);
        self.type_info.speed * strafe_penalty * boost * floor * self.status.speed_multiplier
    }

    pub fn pref_rotation_with(&self, build_target: Option<Vec2>, mine_target: Option<Vec2>) -> f32 {
        if self.is_building() && self.type_info.rotate_to_building {
            if let Some(target) = build_target {
                return angle_to(self.x(), self.y(), target.x, target.y);
            }
        } else if let Some(target) = mine_target {
            return angle_to(self.x(), self.y(), target.x, target.y);
        } else if self.vel.moving() && self.type_info.omni_movement {
            return vector_angle(self.vel.vel);
        }

        self.rotation()
    }

    pub fn clip_size_with_region(&self, region_width: f32, infinite_resources: bool) -> f32 {
        if self.is_building() {
            if infinite_resources {
                f32::MAX
            } else {
                self.type_info.clip_size.max(region_width)
                    + self.type_info.build_range
                    + TILE_SIZE * 4.0
            }
        } else if self.mining() {
            self.type_info.clip_size + self.type_info.mine_range
        } else {
            self.type_info.clip_size
        }
    }

    pub fn heal_mark(&mut self, amount: f32) {
        if self.health.health < self.health.max_health && amount > 0.0 {
            self.was_healed = true;
        }
    }

    pub fn sense_basic(&self, sensor: LAccess, rules_unit_ammo: bool) -> f64 {
        match sensor {
            LAccess::TotalItems => self.items.stack.amount as f64,
            LAccess::ItemCapacity => self.type_info.item_capacity as f64,
            LAccess::Rotation => self.rotation() as f64,
            LAccess::Health => self.health.health as f64,
            LAccess::Shield => self.shield.shield as f64,
            LAccess::MaxHealth => self.health.max_health as f64,
            LAccess::Ammo => {
                if !rules_unit_ammo {
                    self.type_info.ammo_capacity as f64
                } else {
                    self.weapons.ammo as f64
                }
            }
            LAccess::AmmoCapacity => self.type_info.ammo_capacity as f64,
            LAccess::X => World::conv(self.x()) as f64,
            LAccess::Y => World::conv(self.y()) as f64,
            LAccess::VelocityX => (self.vel.vel.x * 60.0 / TILE_SIZE) as f64,
            LAccess::VelocityY => (self.vel.vel.y * 60.0 / TILE_SIZE) as f64,
            LAccess::Dead => {
                if self.health.dead || !self.entity.is_added() {
                    1.0
                } else {
                    0.0
                }
            }
            LAccess::Team => self.team.team.0 as f64,
            LAccess::Shooting => {
                if self.weapons.is_shooting {
                    1.0
                } else {
                    0.0
                }
            }
            LAccess::Boosting => {
                if self.type_info.can_boost && self.is_flying() {
                    1.0
                } else {
                    0.0
                }
            }
            LAccess::Range => (self.range() / TILE_SIZE) as f64,
            LAccess::Mining => {
                if self.mining() {
                    1.0
                } else {
                    0.0
                }
            }
            LAccess::MineX => self.mine_tile().map(|tile| tile.world_x).unwrap_or(-1) as f64,
            LAccess::MineY => self.mine_tile().map(|tile| tile.world_y).unwrap_or(-1) as f64,
            LAccess::BuildX => self.build_plan().map(|plan| plan.x).unwrap_or(-1) as f64,
            LAccess::BuildY => self.build_plan().map(|plan| plan.y).unwrap_or(-1) as f64,
            LAccess::Armor => self.effective_armor() as f64,
            LAccess::Flag => self.flag,
            LAccess::Speed => {
                (self.type_info.speed * 60.0 / TILE_SIZE * self.status.speed_multiplier) as f64
            }
            LAccess::Controlled => self.controller.controlled_code(self.is_valid()) as f64,
            LAccess::PayloadCount => self.payload_count() as f64,
            LAccess::TotalPayload => (self.payload_used() / TILE_PAYLOAD) as f64,
            LAccess::PayloadCapacity => self.payload_capacity_tiles() as f64,
            LAccess::Size => (self.hitbox.hit_size / TILE_SIZE) as f64,
            LAccess::Id => self.id() as f64,
            _ => f64::NAN,
        }
    }

    pub fn set_prop_basic(&mut self, prop: LAccess, value: f64, net_client: bool) -> bool {
        match prop {
            LAccess::Health => {
                self.health.health = (value as f32).clamp(0.0, self.health.max_health);
                if self.health.health <= 0.0 && !self.health.dead {
                    self.health.kill();
                }
            }
            LAccess::Shield => self.shield.shield = (value as f32).max(0.0),
            LAccess::X => self.set_pos(World::unconv(value as f32), self.y()),
            LAccess::Y => self.set_pos(self.x(), World::unconv(value as f32)),
            LAccess::VelocityX => self.vel.vel.x = value as f32 * TILE_SIZE / 60.0,
            LAccess::VelocityY => self.vel.vel.y = value as f32 * TILE_SIZE / 60.0,
            LAccess::Rotation => self.set_rotation(value as f32),
            LAccess::Team if !net_client => {
                self.team.team = TeamId(value as u8);
                self.refresh_component_views();
            }
            LAccess::Flag => self.flag = value,
            LAccess::Armor => self.status.armor_override = (value as f32).max(0.0),
            _ => return false,
        }

        self.refresh_component_views();
        true
    }
}

impl EntityPosition for UnitComp {
    fn x(&self) -> f32 {
        self.x()
    }

    fn y(&self) -> f32 {
        self.y()
    }
}

impl SizedEntity for UnitComp {
    fn hit_size(&self) -> f32 {
        self.hitbox.hit_size
    }
}

fn miner_type_from_unit_type(type_info: &UnitType) -> MinerType {
    MinerType {
        mine_tier: type_info.mine_tier,
        mine_floor: type_info.mine_floor,
        mine_walls: type_info.mine_walls,
        mine_range: type_info.mine_range,
        mine_speed: type_info.mine_speed,
        mine_hardness_scaling: type_info.mine_hardness_scaling,
    }
}

fn unit_item_capacity(type_info: &UnitType) -> i32 {
    type_info.item_capacity.max(0)
}

fn unit_ammo_capacity(type_info: &UnitType) -> f32 {
    type_info.ammo_capacity.max(0) as f32
}

fn vector_length(v: Vec2) -> f32 {
    vector_length_sq(v).sqrt()
}

fn vector_length_sq(v: Vec2) -> f32 {
    v.x * v.x + v.y * v.y
}

fn vector_limited(v: Vec2, max: f32) -> Vec2 {
    let len = vector_length(v);
    if len <= max || len <= 0.000001 {
        v
    } else {
        let scale = max / len;
        Vec2 {
            x: v.x * scale,
            y: v.y * scale,
        }
    }
}

fn vector_from_angle(angle: f32, length: f32) -> Vec2 {
    let radians = angle.to_radians();
    Vec2 {
        x: radians.cos() * length,
        y: radians.sin() * length,
    }
}

fn vector_angle(v: Vec2) -> f32 {
    v.y.atan2(v.x).to_degrees().rem_euclid(360.0)
}

fn angle_to(x: f32, y: f32, tx: f32, ty: f32) -> f32 {
    vector_angle(Vec2 {
        x: tx - x,
        y: ty - y,
    })
}

fn angle_dist(a: f32, b: f32) -> f32 {
    let diff = (a - b).rem_euclid(360.0).abs();
    diff.min(360.0 - diff)
}

fn move_toward_angle(angle: f32, target: f32, max_delta: f32) -> f32 {
    let diff = (target - angle + 540.0).rem_euclid(360.0) - 180.0;
    if diff.abs() <= max_delta {
        target.rem_euclid(360.0)
    } else {
        (angle + diff.signum() * max_delta).rem_euclid(360.0)
    }
}

fn approach_delta(value: f32, target: f32, amount: f32) -> f32 {
    if value < target {
        (value + amount).min(target)
    } else {
        (value - amount).max(target)
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::entities::units::BuildPlan;
    use crate::mindustry::r#type::Weapon;

    fn unit_type() -> UnitType {
        let mut unit = UnitType::new(1, "alpha");
        unit.health = 120.0;
        unit.drag = 0.2;
        unit.armor = 3.0;
        unit.hit_size = 10.0;
        unit.item_capacity = 40;
        unit.ammo_capacity = 60;
        unit.aim_dst = 12.0;
        unit.build_speed = 1.0;
        unit.mine_tier = 2;
        unit.mine_speed = 2.0;
        unit.mine_range = 64.0;
        unit.max_range = 128.0;
        unit.clip_size = 30.0;
        unit.payload_capacity = TILE_PAYLOAD * 2.0;
        unit.weapons.push(Weapon::new("duo"));
        unit.abilities.push("regen".into());
        unit
    }

    #[test]
    fn unit_component_set_type_initializes_subcomponent_views() {
        let unit_type = unit_type();
        let mut unit = UnitComp::new(42, unit_type.clone(), TeamId(1));
        unit.add();

        assert_eq!(unit.id(), 42);
        assert_eq!(unit.health.max_health, 120.0);
        assert_eq!(unit.health.health, 120.0);
        assert_eq!(unit.shield.armor, 3.0);
        assert_eq!(unit.hitbox.hit_size, 10.0);
        assert_eq!(unit.physics.hit_size, 10.0);
        assert_eq!(unit.items.item_capacity(), 40);
        assert_eq!(unit.weapons.ammo_capacity, 60.0);
        assert_eq!(unit.weapons.mounts.len(), 1);
        assert_eq!(unit.abilities.len(), 1);
        assert!(unit.builder.can_build());
        assert_eq!(unit.miner.type_info.mine_tier, 2);
        assert!(unit.is_valid());

        let mut replacement = unit_type.clone();
        replacement.health = 200.0;
        replacement.weapons.push(Weapon::new("scatter"));
        unit.set_type(replacement);

        assert_eq!(unit.health.max_health, 200.0);
        assert_eq!(unit.weapons.mounts.len(), 2);
    }

    #[test]
    fn unit_component_refresh_mirrors_canonical_transform_to_children() {
        let mut unit = UnitComp::new(1, unit_type(), TeamId(2));
        unit.payload = Some(PayloadComp::new(TeamId(2), 10.0));

        unit.set_pos(24.0, 32.0);
        unit.set_rotation(450.0);
        unit.refresh_component_views();

        assert_eq!((unit.x(), unit.y(), unit.rotation()), (24.0, 32.0, 90.0));
        assert_eq!((unit.hitbox.x, unit.hitbox.y), (24.0, 32.0));
        assert_eq!((unit.physics.x, unit.physics.y), (24.0, 32.0));
        assert_eq!(
            (unit.builder.x, unit.builder.y, unit.builder.rotation),
            (24.0, 32.0, 90.0)
        );
        assert_eq!((unit.weapons.x, unit.weapons.y), (24.0, 32.0));
        assert_eq!(unit.payload.as_ref().unwrap().rotation, 90.0);
    }

    #[test]
    fn unit_component_movement_and_rotation_match_unitcomp_shape() {
        let mut unit_type = unit_type();
        unit_type.omni_movement = false;
        unit_type.accel = 1.0;
        unit_type.rotate_speed = 90.0;
        let mut unit = UnitComp::new(1, unit_type, TeamId(1));

        unit.move_pref(Vec2::new(0.0, 10.0), 1.0);

        assert!((unit.vel.vel.x - 10.0).abs() < 0.000001);
        assert!((unit.vel.vel.y - 0.0).abs() < 0.000001);
        assert_eq!(unit.rotation(), 90.0);

        unit.look_at_xy(10.0, 32.0, 1.0);
        assert!(unit.rotation() >= 0.0);
    }

    #[test]
    fn unit_component_boosting_grounding_and_speed_helpers_are_pure() {
        let mut unit_type = unit_type();
        unit_type.can_boost = true;
        unit_type.rise_speed = 0.5;
        unit_type.descent_speed = 0.25;
        unit_type.boost_multiplier = 2.0;
        unit_type.speed = 3.0;
        let mut unit = UnitComp::new(1, unit_type, TeamId(1));

        assert!(unit.is_grounded());
        assert!(unit.update_boosting(true, true, false, true, 1.0));
        assert_eq!(unit.elevation, 0.5);
        assert!(unit.is_flying());
        assert!(!unit.can_shoot());

        unit.update_boosting(false, false, false, true, 1.0);
        assert_eq!(unit.elevation, 0.25);
        assert_eq!(unit.speed_with_floor(1.0), 3.75);
    }

    #[test]
    fn controller_state_round_trips_wire_and_reports_control_codes() {
        let player = UnitControllerState::from(ControllerWire::Player { player_id: 7 });
        assert!(player.is_player());
        assert_eq!(player.to_wire(), ControllerWire::Player { player_id: 7 });
        assert_eq!(player.controlled_code(true), LOGIC_CTRL_PLAYER);

        let command = UnitControllerState::Command(CommandWire::new());
        assert!(command.is_commandable());
        assert_eq!(command.controlled_code(true), LOGIC_CTRL_COMMAND);
        assert_eq!(
            UnitControllerState::Logic { controller_pos: 5 }.controlled_code(true),
            LOGIC_CTRL_PROCESSOR
        );
        assert_eq!(command.controlled_code(false), 0);
    }

    #[test]
    fn unit_component_build_mine_payload_and_sense_helpers_are_snapshotted() {
        let mut unit = UnitComp::new(9, unit_type(), TeamId(3));
        unit.add();
        unit.set_pos(16.0, 24.0);
        unit.vel.vel = Vec2::new(4.0, 8.0);
        unit.items.add_item_amount("copper", 5);
        unit.weapons.ammo = 11.0;
        unit.shield.shield = 6.0;
        unit.flag = 123.0;
        unit.builder.add_build(BuildPlan::new_place(4, 5, 0, "duo"));
        unit.payload = Some(PayloadComp::new(TeamId(3), TILE_PAYLOAD * 2.0));
        unit.refresh_component_views();

        assert_eq!(unit.sense_basic(LAccess::TotalItems, true), 5.0);
        assert_eq!(unit.sense_basic(LAccess::Ammo, true), 11.0);
        assert_eq!(unit.sense_basic(LAccess::Ammo, false), 60.0);
        assert_eq!(unit.sense_basic(LAccess::Shield, true), 6.0);
        assert_eq!(unit.sense_basic(LAccess::BuildX, true), 4.0);
        assert_eq!(unit.sense_basic(LAccess::PayloadCapacity, true), 2.0);
        assert_eq!(unit.sense_basic(LAccess::Controlled, true), 0.0);

        unit.set_controller(UnitControllerState::Player { player_id: 1 });
        assert_eq!(
            unit.sense_basic(LAccess::Controlled, true),
            LOGIC_CTRL_PLAYER as f64
        );

        assert!(unit.set_prop_basic(LAccess::X, 10.0, false));
        assert_eq!(World::conv(unit.x()), 10.0);
        assert!(unit.set_prop_basic(LAccess::Team, 5.0, false));
        assert_eq!(unit.team_id(), TeamId(5));
        assert!(!unit.set_prop_basic(LAccess::Team, 6.0, true));
        assert_eq!(unit.team_id(), TeamId(5));
    }
}
