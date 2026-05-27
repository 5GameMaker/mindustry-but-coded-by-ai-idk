//! Unit component shell mirroring upstream `mindustry.entities.comp.UnitComp`.
//!
//! The Java component is the central runtime hub for movement, controllers,
//! health, inventory, weapons, building, mining and payload behavior. This
//! Rust stage keeps a compositional shell around already-ported components and
//! migrates pure helper behavior first; global `Vars`/`World` side effects are
//! left to explicit snapshots or future runtime adapters.

use crate::mindustry::ai::{PrebuildAiPlanSnapshot, PrebuildAiRequirement};
use crate::mindustry::core::world::World;
use crate::mindustry::ctype::{Content, ContentId};
use crate::mindustry::entities::abilities::{
    EnergyFieldAbility, EnergyFieldPulse, EnergyFieldTarget, ForceFieldAbility, ForceFieldUpdate,
    MoveEffectAbility, MoveEffectPlan, RegenAbility, RepairFieldAbility, RepairFieldPulse,
    RepairFieldTarget, ShieldArcAbility, ShieldArcUpdate, ShieldRegenFieldAbility,
    ShieldRegenFieldPulse, ShieldRegenFieldTarget, SpawnDeathAbility, SpawnDeathSpawnPlan,
    StatusFieldAbility, StatusFieldPulse, SuppressionFieldAbility, SuppressionFieldPulse,
    UnitSpawnAbility, UnitSpawnPlan,
};
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::entities::{EntityPosition, SizedEntity};
use crate::mindustry::game::{BlockPlan as TeamBlockPlan, TeamData};
use crate::mindustry::io::type_io::{
    BuildPlanWire, CommandWire, ControllerWire, MountWire, UnitSyncWire,
};
use crate::mindustry::io::{AbilityWire, TeamId, Vec2};
use crate::mindustry::logic::{
    LAccess, LOGIC_CTRL_COMMAND, LOGIC_CTRL_PLAYER, LOGIC_CTRL_PROCESSOR,
};
use crate::mindustry::r#type::ItemStack;
use crate::mindustry::r#type::UnitType;
use crate::mindustry::world::{point2_pack, point2_x, point2_y};

use super::builder::{
    BuilderAiRuntimeInput, BuilderAiRuntimeState, BuilderAiRuntimeStep, BuilderComp,
    PrebuildAiRuntimeInput, PrebuildAiRuntimeState, PrebuildAiRuntimeStep,
};
use super::building_tether::BuildingTetherComp;
use super::entity::EntityComp;
use super::health::HealthComp;
use super::hitbox::HitboxComp;
use super::items::ItemsComp;
use super::miner::{MineTile, MinerComp, MinerType, PrebuildMiningRuntimeStep};
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
    Cargo,
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
            Self::Cargo => ControllerWire::Ground,
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

    pub fn is_cargo(&self) -> bool {
        matches!(self, Self::Cargo)
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
pub struct CargoAiRuntimeState {
    pub tether_tile_pos: Option<i32>,
    pub unload_target_tile_pos: Option<i32>,
    pub item_target: Option<String>,
    pub no_dest_timer: f32,
    pub drop_timer: f32,
    pub retarget_timer: f32,
    pub target_index: usize,
}

impl CargoAiRuntimeState {
    pub fn new(tether_tile_pos: Option<i32>) -> Self {
        Self {
            tether_tile_pos,
            ..Self::default()
        }
    }
}

impl Default for CargoAiRuntimeState {
    fn default() -> Self {
        Self {
            tether_tile_pos: None,
            unload_target_tile_pos: None,
            item_target: None,
            no_dest_timer: 0.0,
            drop_timer: 90.0,
            retarget_timer: 40.0,
            target_index: 0,
        }
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
    pub builder_ai: BuilderAiRuntimeState,
    pub prebuild_ai: PrebuildAiRuntimeState,
    pub building_tether: Option<BuildingTetherComp>,
    pub cargo_ai: Option<CargoAiRuntimeState>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrebuildAiUnitInput {
    pub runtime: PrebuildAiRuntimeInput,
    pub core_accept_stack_amount: i32,
    pub within_core_range: bool,
    pub timer_ore_ready: bool,
    pub build_cost_multiplier: f32,
}

impl Default for PrebuildAiUnitInput {
    fn default() -> Self {
        Self {
            runtime: PrebuildAiRuntimeInput::default(),
            core_accept_stack_amount: 0,
            within_core_range: false,
            timer_ore_ready: false,
            build_cost_multiplier: 1.0,
        }
    }
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
            builder_ai: BuilderAiRuntimeState::default(),
            prebuild_ai: PrebuildAiRuntimeState::default(),
            building_tether: None,
            cargo_ai: None,
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
        unit.apply_created_force_field_abilities();
        unit.apply_created_shield_arc_abilities();
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

    fn apply_created_force_field_abilities(&mut self) {
        for descriptor in &self.type_info.abilities {
            if let Some(ability) = ForceFieldAbility::from_descriptor(descriptor) {
                self.shield.shield = self.shield.shield.max(ability.created_shield());
            }
        }
    }

    fn apply_created_shield_arc_abilities(&mut self) {
        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = ShieldArcAbility::from_descriptor(descriptor) else {
                continue;
            };
            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = wire.data.max(ability.created_shield());
            }
        }
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

    pub fn update_regen_abilities(&mut self, delta: f32) -> Vec<f32> {
        if self.health.dead {
            return Vec::new();
        }
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let mut heals = Vec::new();
        let descriptors = self.type_info.abilities.clone();
        for descriptor in descriptors {
            let Some(ability) = RegenAbility::from_descriptor(&descriptor) else {
                continue;
            };
            let heal = ability.heal_amount(self.health.max_health, delta);
            if heal <= 0.0 {
                continue;
            }
            self.heal_mark(heal);
            self.health.heal(heal);
            heals.push(heal);
        }

        heals
    }

    pub fn update_force_field_abilities(&mut self, delta: f32) -> Vec<ForceFieldUpdate> {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let mut updates = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = ForceFieldAbility::from_descriptor(descriptor) else {
                continue;
            };
            let wire_data = self.abilities.get(index).map_or(0.0, |wire| wire.data);
            let initialized = wire_data != 0.0 || self.shield.shield != 0.0;
            if !initialized {
                self.shield.shield = ability.created_shield();
            }

            ability.radius_scale = wire_data.max(0.0);
            ability.was_broken = wire_data < 0.0 || (self.shield.shield <= 0.0 && wire_data <= 0.0);
            let update = ability.update_state(self.shield.shield, delta);
            self.shield.shield = update.shield;
            if update.shield > 0.0 {
                self.shield.shield_alpha = self.shield.shield_alpha.max(update.alpha);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = if update.shield > 0.0 {
                    update.radius_scale
                } else {
                    -1.0
                };
            }
            updates.push(update);
        }

        updates
    }

    pub fn update_shield_arc_abilities(&mut self, delta: f32) -> Vec<ShieldArcUpdate> {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let unit_x = self.x();
        let unit_y = self.y();
        let unit_rotation = self.rotation();
        let is_shooting = self.weapons.is_shooting;
        let mut updates = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = ShieldArcAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.data = self.abilities.get(index).map_or(0.0, |wire| wire.data);

            let update = ability.update_state(delta, is_shooting, unit_x, unit_y, unit_rotation);

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.data;
            }
            updates.push(update);
        }

        updates
    }

    pub fn update_move_effect_abilities(
        &mut self,
        delta: f32,
        in_fog: bool,
    ) -> Vec<MoveEffectPlan> {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let unit_x = self.x();
        let unit_y = self.y();
        let unit_rotation = self.rotation();
        let velocity_len2 = self.vel.vel.x * self.vel.vel.x + self.vel.vel.y * self.vel.vel.y;
        let mut plans = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = MoveEffectAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.counter = self.abilities.get(index).map_or(0.0, |wire| wire.data);

            if let Some(plan) = ability.update_plan(
                delta,
                velocity_len2,
                in_fog,
                false,
                unit_x,
                unit_y,
                unit_rotation,
                (0.0, 0.0),
            ) {
                plans.push(plan);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.counter;
            }
        }

        plans
    }

    pub fn update_unit_spawn_abilities<F>(
        &mut self,
        delta: f32,
        unit_build_speed: f32,
        mut can_create: F,
    ) -> Vec<UnitSpawnPlan>
    where
        F: FnMut(&str) -> bool,
    {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let parent_x = self.x();
        let parent_y = self.y();
        let parent_rotation = self.rotation();
        let mut plans = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = UnitSpawnAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.timer = self.abilities.get(index).map_or(0.0, |wire| wire.data);
            let reaches_spawn_time = ability.timer + delta * unit_build_speed >= ability.spawn_time;
            let allowed = reaches_spawn_time && can_create(&ability.unit);

            if let Some(plan) = ability.update_state(
                delta,
                unit_build_speed,
                allowed,
                parent_x,
                parent_y,
                parent_rotation,
            ) {
                plans.push(plan);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.timer;
            }
        }

        plans
    }

    pub fn spawn_death_ability_plans(&self) -> Vec<(String, SpawnDeathSpawnPlan)> {
        let unit_rotation = self.rotation();
        self.type_info
            .abilities
            .iter()
            .filter_map(|descriptor| SpawnDeathAbility::from_descriptor(descriptor))
            .flat_map(|ability| {
                let count = ability.planned_spawn_count(0).max(0) as usize;
                (0..count).map(move |index| {
                    let angle = if count == 0 {
                        0.0
                    } else {
                        index as f32 * 360.0 / count as f32
                    };
                    (
                        ability.unit.clone(),
                        ability.planned_spawn(unit_rotation, angle, 1.0, 0.0),
                    )
                })
            })
            .collect()
    }

    pub fn update_energy_field_abilities(
        &mut self,
        delta: f32,
        unit_damage_scale: f32,
        unit_ammo_rule: bool,
        targets: &[EnergyFieldTarget],
    ) -> Vec<EnergyFieldPulse> {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let unit_x = self.x();
        let unit_y = self.y();
        let unit_rotation = self.rotation();
        let mut pulses = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = EnergyFieldAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.timer = self.abilities.get(index).map_or(0.0, |wire| wire.data);

            if let Some(pulse) = ability.update_targets(
                delta,
                unit_x,
                unit_y,
                unit_rotation,
                unit_damage_scale,
                self.weapons.ammo as i32,
                unit_ammo_rule,
                targets,
            ) {
                self.weapons.ammo = pulse.ammo_after.max(0) as f32;
                pulses.push(pulse);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.timer;
            }
        }

        pulses
    }

    pub fn update_shield_regen_field_abilities<F>(
        &mut self,
        delta: f32,
        mut targets: F,
    ) -> Vec<ShieldRegenFieldPulse>
    where
        F: FnMut(&ShieldRegenFieldAbility) -> Vec<(u32, ShieldRegenFieldTarget)>,
    {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let mut pulses = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = ShieldRegenFieldAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.timer = self.abilities.get(index).map_or(0.0, |wire| wire.data);
            let target_entries = targets(&ability);
            let target_ids = target_entries
                .iter()
                .map(|(target_id, _target)| *target_id)
                .collect::<Vec<_>>();
            let target_values = target_entries
                .iter()
                .map(|(_target_id, target)| *target)
                .collect::<Vec<_>>();

            if let Some(mut pulse) = ability.update_targets(delta, &target_values) {
                pulse.target_ids = target_ids;
                pulses.push(pulse);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.timer;
            }
        }

        pulses
    }

    pub fn update_repair_field_abilities<F>(
        &mut self,
        delta: f32,
        mut targets: F,
    ) -> Vec<RepairFieldPulse>
    where
        F: FnMut(&RepairFieldAbility) -> Vec<(u32, RepairFieldTarget)>,
    {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let mut pulses = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = RepairFieldAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.timer = self.abilities.get(index).map_or(0.0, |wire| wire.data);
            let target_entries = targets(&ability);
            let target_ids = target_entries
                .iter()
                .map(|(target_id, _target)| *target_id)
                .collect::<Vec<_>>();
            let target_values = target_entries
                .iter()
                .map(|(_target_id, target)| *target)
                .collect::<Vec<_>>();

            if let Some(mut pulse) = ability.update_targets(delta, &target_values) {
                pulse.target_ids = target_ids;
                pulses.push(pulse);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.timer;
            }
        }

        pulses
    }

    pub fn update_status_field_abilities<F>(
        &mut self,
        delta: f32,
        mut target_ids: F,
    ) -> Vec<StatusFieldPulse>
    where
        F: FnMut(&StatusFieldAbility) -> Vec<u32>,
    {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let unit_x = self.x();
        let unit_y = self.y();
        let unit_rotation = self.rotation();
        let is_shooting = self.weapons.is_shooting;
        let mut pulses = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = StatusFieldAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.timer = self.abilities.get(index).map_or(0.0, |wire| wire.data);
            let ids = target_ids(&ability);

            if let Some(mut pulse) =
                ability.update_targets(delta, is_shooting, unit_x, unit_y, unit_rotation, ids.len())
            {
                pulse.target_ids = ids;
                pulses.push(pulse);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.timer;
            }
        }

        pulses
    }

    pub fn update_suppression_field_abilities(&mut self, delta: f32) -> Vec<SuppressionFieldPulse> {
        if self.abilities.len() != self.type_info.abilities.len() {
            self.abilities = vec![AbilityWire::default(); self.type_info.abilities.len()];
        }

        let unit_x = self.x();
        let unit_y = self.y();
        let unit_rotation = self.rotation();
        let mut pulses = Vec::new();

        for (index, descriptor) in self.type_info.abilities.iter().enumerate() {
            let Some(mut ability) = SuppressionFieldAbility::from_descriptor(descriptor) else {
                continue;
            };
            ability.timer = self.abilities.get(index).map_or(0.0, |wire| wire.data);

            if let Some(pulse) = ability.update_state(delta, unit_x, unit_y, unit_rotation) {
                pulses.push(pulse);
            }

            if let Some(wire) = self.abilities.get_mut(index) {
                wire.data = ability.timer;
            }
        }

        pulses
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

    #[allow(clippy::too_many_arguments)]
    pub fn tick_builder_ai<
        FValidBreak,
        FValidPlace,
        FWithinTeamPlanRange,
        FAlreadyPlaced,
        FTeamValidPlace,
        FNearEnemy,
    >(
        &mut self,
        team_data: &mut TeamData,
        input: BuilderAiRuntimeInput,
        valid_break: FValidBreak,
        valid_place: FValidPlace,
        within_team_plan_range: FWithinTeamPlanRange,
        already_placed: FAlreadyPlaced,
        team_valid_place: FTeamValidPlace,
        near_enemy: FNearEnemy,
    ) -> BuilderAiRuntimeStep
    where
        FValidBreak: FnMut(&BuildPlan) -> bool,
        FValidPlace: FnMut(&BuildPlan) -> bool,
        FWithinTeamPlanRange: FnMut(&TeamBlockPlan) -> bool,
        FAlreadyPlaced: FnMut(&TeamBlockPlan) -> bool,
        FTeamValidPlace: FnMut(&TeamBlockPlan) -> bool,
        FNearEnemy: FnMut(&TeamBlockPlan) -> bool,
    {
        self.refresh_component_views();
        let step = self.builder.apply_builder_ai_tick(
            &mut self.builder_ai,
            team_data,
            input,
            valid_break,
            valid_place,
            within_team_plan_range,
            already_placed,
            team_valid_place,
            near_enemy,
        );
        self.refresh_component_views();
        step
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick_prebuild_ai_builder<FValidBreak, FValidPlace, FPlanValid, FPlanCanBuild>(
        &mut self,
        input: PrebuildAiRuntimeInput,
        valid_break: FValidBreak,
        valid_place: FValidPlace,
        next_plan: Option<PrebuildAiPlanSnapshot>,
        plan_valid_place: FPlanValid,
        plan_can_build: FPlanCanBuild,
    ) -> PrebuildAiRuntimeStep
    where
        FValidBreak: FnMut(&BuildPlan) -> bool,
        FValidPlace: FnMut(&BuildPlan) -> bool,
        FPlanValid: FnMut(&PrebuildAiPlanSnapshot) -> bool,
        FPlanCanBuild: FnMut(&PrebuildAiPlanSnapshot) -> bool,
    {
        let step = self.builder.apply_prebuild_ai_tick(
            &mut self.prebuild_ai,
            input,
            valid_break,
            valid_place,
            next_plan,
            plan_valid_place,
            plan_can_build,
        );
        self.refresh_component_views();
        step
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick_prebuild_ai_mining<
        FCoreHas,
        FCoreAcceptOne,
        FAcceptsItem,
        FFindFloorOre,
        FFindWallOre,
        FOreTile,
        FCanBuild,
    >(
        &mut self,
        requirements: &[PrebuildAiRequirement],
        input: PrebuildAiUnitInput,
        core_accept_stack_one: FCoreAcceptOne,
        can_build_after_deposit: FCanBuild,
        core_has_item: FCoreHas,
        accepts_item: FAcceptsItem,
        find_floor_ore: FFindFloorOre,
        find_wall_ore: FFindWallOre,
        ore_tile: FOreTile,
    ) -> PrebuildMiningRuntimeStep
    where
        FCoreHas: FnMut(ContentId, i32) -> bool,
        FCoreAcceptOne: FnMut(ContentId) -> i32,
        FAcceptsItem: FnMut(ContentId) -> bool,
        FFindFloorOre: FnMut(ContentId) -> Option<i32>,
        FFindWallOre: FnMut(ContentId) -> Option<i32>,
        FOreTile: FnMut(i32) -> Option<MineTile>,
        FCanBuild: FnMut() -> bool,
    {
        self.refresh_component_views();
        let step = self.miner.apply_prebuild_mining_tick(
            &mut self.prebuild_ai,
            requirements,
            input.build_cost_multiplier,
            input.timer_ore_ready,
            core_accept_stack_one,
            input.core_accept_stack_amount,
            input.within_core_range,
            can_build_after_deposit,
            core_has_item,
            accepts_item,
            find_floor_ore,
            find_wall_ore,
            ore_tile,
        );
        self.items.stack.item = self.miner.stack_item.clone();
        self.items.stack.amount = self.miner.stack_amount;
        self.refresh_component_views();
        step
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

    pub fn to_sync_wire(&self) -> UnitSyncWire {
        UnitSyncWire {
            abilities: self.abilities.clone(),
            ammo: self.weapons.ammo,
            controller: self.controller.clone().into(),
            elevation: self.elevation,
            flag: self.flag,
            health: self.health.health,
            is_shooting: self.weapons.is_shooting,
            mine_tile: self
                .miner
                .mine_tile
                .as_ref()
                .map(|tile| point2_pack(world_to_tile(tile.world_x), world_to_tile(tile.world_y))),
            mounts: self
                .weapons
                .mounts
                .iter()
                .map(|mount| MountWire {
                    shoot: mount.shoot,
                    rotate: mount.rotate,
                    aim_x: mount.aim_x,
                    aim_y: mount.aim_y,
                })
                .collect(),
            plans: if self.builder.plans.is_empty() {
                None
            } else {
                Some(
                    self.builder
                        .plans
                        .iter()
                        .map(BuildPlanWire::from_build_plan)
                        .collect(),
                )
            },
            rotation: self.rotation(),
            shield: self.shield.shield,
            spawned_by_core: self.spawned_by_core,
            stack: ItemStack::new(
                self.items.item().unwrap_or_default(),
                self.items.stack.amount,
            ),
            statuses: self.status.statuses.clone(),
            team: self.team.team,
            type_id: self.type_info.id(),
            update_building: self.builder.is_building(),
            vel: self.vel.vel,
            x: self.x(),
            y: self.y(),
        }
    }

    pub fn apply_sync_wire(&mut self, sync: &UnitSyncWire) {
        self.abilities = sync.abilities.clone();
        self.weapons.ammo = sync.ammo;
        self.controller = sync.controller.clone().into();
        self.elevation = sync.elevation;
        self.flag = sync.flag;
        self.health.health = sync.health;
        self.weapons.is_shooting = sync.is_shooting;
        self.miner.mine_tile = sync.mine_tile.map(|tile_pos| mine_tile_from_wire(tile_pos));
        if self.weapons.mounts.len() != sync.mounts.len() {
            self.weapons
                .setup_weapons(self.type_info.weapons.iter().cloned());
        }
        for (mount, state) in self.weapons.mounts.iter_mut().zip(sync.mounts.iter()) {
            mount.shoot = state.shoot;
            mount.rotate = state.rotate;
            mount.aim_x = state.aim_x;
            mount.aim_y = state.aim_y;
        }
        self.builder.plans = sync
            .plans
            .as_ref()
            .map(|plans| {
                plans
                    .iter()
                    .filter_map(|plan| plan.to_build_plan().ok())
                    .collect::<std::collections::VecDeque<_>>()
            })
            .unwrap_or_default();
        self.set_rotation(sync.rotation);
        self.shield.shield = sync.shield;
        self.spawned_by_core = sync.spawned_by_core;
        self.items.stack.item = if sync.stack.item.is_empty() {
            None
        } else {
            Some(sync.stack.item.clone())
        };
        self.items.stack.amount = sync.stack.amount;
        self.status.statuses = sync.statuses.clone();
        self.team.team = sync.team;
        self.builder.update_building = sync.update_building;
        self.vel.vel = sync.vel;
        self.set_pos(sync.x, sync.y);
        self.refresh_component_views();
        let _ = sync.type_id;
    }
}

fn world_to_tile(value: i32) -> i32 {
    (value as f32 / TILE_SIZE).round() as i32
}

fn mine_tile_from_wire(tile_pos: i32) -> MineTile {
    crate::mindustry::entities::comp::miner::MineTile {
        world_x: point2_x(tile_pos) as i32 * TILE_SIZE as i32,
        world_y: point2_y(tile_pos) as i32 * TILE_SIZE as i32,
        block_air: false,
        floor_drop: None,
        wall_drop: None,
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
    use crate::mindustry::entities::comp::BuilderAiRuntimeBranch;
    use crate::mindustry::entities::units::BuildPlan;
    use crate::mindustry::entities::StatusEntry;
    use crate::mindustry::io::TypeValue;
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
        unit.build_range = 64.0;
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
    fn unit_component_ticks_unit_spawn_ability_from_runtime_ability_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["UnitSpawnAbility:flare:10:2:4".into()];
        let mut unit = UnitComp::new(42, unit_type, TeamId(1));
        unit.set_pos(100.0, 200.0);
        unit.set_rotation(90.0);

        let plans = unit.update_unit_spawn_abilities(4.0, 2.0, |_| true);
        assert!(plans.is_empty());
        assert_eq!(unit.abilities[0].data, 8.0);

        let plans = unit.update_unit_spawn_abilities(1.0, 2.0, |unit_name| unit_name == "flare");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].unit, "flare");
        assert!((plans[0].x - 102.0).abs() < 0.0001);
        assert!((plans[0].y - 204.0).abs() < 0.0001);
        assert_eq!(plans[0].rotation, 90.0);
        assert_eq!(unit.abilities[0].data, 0.0);
    }

    #[test]
    fn unit_component_keeps_unit_spawn_timer_ready_when_creation_is_blocked() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["UnitSpawnAbility:mono:5:0:0".into()];
        let mut unit = UnitComp::new(42, unit_type, TeamId(1));

        let blocked = unit.update_unit_spawn_abilities(5.0, 1.0, |_| false);
        assert!(blocked.is_empty());
        assert_eq!(unit.abilities[0].data, 5.0);

        let spawned = unit.update_unit_spawn_abilities(0.0, 1.0, |unit_name| unit_name == "mono");
        assert_eq!(spawned.len(), 1);
        assert_eq!(unit.abilities[0].data, 0.0);
    }

    #[test]
    fn unit_component_ticks_energy_field_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["EnergyFieldAbility:40:5:80:1.5:0.5:3".into()];
        let mut unit = UnitComp::new(42, unit_type, TeamId(1));
        unit.set_pos(100.0, 200.0);
        unit.weapons.ammo = 2.0;
        let targets = [EnergyFieldTarget {
            id: 7,
            x: 120.0,
            y: 200.0,
            air: false,
            targetable: true,
            same_team: false,
            damaged: false,
            max_health: 100.0,
            same_type: false,
        }];

        assert!(unit
            .update_energy_field_abilities(4.0, 1.0, true, &targets)
            .is_empty());
        assert_eq!(unit.abilities[0].data, 4.0);

        let pulses = unit.update_energy_field_abilities(1.0, 2.0, true, &targets);
        assert_eq!(pulses.len(), 1);
        assert!(pulses[0].any_nearby);
        assert_eq!(pulses[0].hits.len(), 1);
        assert_eq!(pulses[0].hits[0].id, 7);
        assert_eq!(pulses[0].hits[0].amount, 80.0);
        assert_eq!(unit.weapons.ammo, 1.0);
        assert_eq!(unit.abilities[0].data, 0.0);
    }

    #[test]
    fn unit_component_ticks_status_field_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["StatusFieldAbility:overclock:10:5:30".into()];
        let mut unit = UnitComp::new(42, unit_type, TeamId(1));

        assert!(unit
            .update_status_field_abilities(4.0, |_| vec![1, 2])
            .is_empty());
        assert_eq!(unit.abilities[0].data, 4.0);

        let pulses = unit.update_status_field_abilities(1.0, |_| vec![1, 2]);
        assert_eq!(pulses.len(), 1);
        assert_eq!(pulses[0].effect, "overclock");
        assert_eq!(pulses[0].duration, 10.0);
        assert_eq!(pulses[0].target_count, 2);
        assert_eq!(pulses[0].target_ids, vec![1, 2]);
        assert_eq!(unit.abilities[0].data, 0.0);
    }

    #[test]
    fn unit_component_ticks_shield_regen_field_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["ShieldRegenFieldAbility:25:250:60:60".into()];
        let mut unit = UnitComp::new(44, unit_type, TeamId(1));

        assert!(unit
            .update_shield_regen_field_abilities(59.0, |_| {
                vec![(1, ShieldRegenFieldTarget { shield: 10.0 })]
            })
            .is_empty());
        assert_eq!(unit.abilities[0].data, 59.0);

        let pulses = unit.update_shield_regen_field_abilities(1.0, |_| {
            vec![
                (1, ShieldRegenFieldTarget { shield: 10.0 }),
                (2, ShieldRegenFieldTarget { shield: 250.0 }),
            ]
        });
        assert_eq!(pulses.len(), 1);
        assert_eq!(pulses[0].target_ids, vec![1, 2]);
        assert_eq!(pulses[0].shields, vec![35.0, 250.0]);
        assert!(pulses[0].active_effect);
        assert_eq!(unit.abilities[0].data, 0.0);
    }

    #[test]
    fn unit_component_ticks_repair_field_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["RepairFieldAbility:10:240:60".into()];
        let mut unit = UnitComp::new(45, unit_type, TeamId(1));

        assert!(unit
            .update_repair_field_abilities(239.0, |_| {
                vec![(
                    1,
                    RepairFieldTarget {
                        damaged: true,
                        max_health: 100.0,
                        same_type: false,
                    },
                )]
            })
            .is_empty());
        assert_eq!(unit.abilities[0].data, 239.0);

        let pulses = unit.update_repair_field_abilities(1.0, |_| {
            vec![
                (
                    1,
                    RepairFieldTarget {
                        damaged: true,
                        max_health: 100.0,
                        same_type: false,
                    },
                ),
                (
                    2,
                    RepairFieldTarget {
                        damaged: false,
                        max_health: 100.0,
                        same_type: true,
                    },
                ),
            ]
        });
        assert_eq!(pulses.len(), 1);
        assert_eq!(pulses[0].target_ids, vec![1, 2]);
        assert_eq!(pulses[0].heals, vec![10.0, 10.0]);
        assert!(pulses[0].active_effect);
        assert_eq!(unit.abilities[0].data, 0.0);
    }

    #[test]
    fn unit_component_ticks_force_field_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["ForceFieldAbility:60:0.4:500:360".into()];
        let mut unit = UnitComp::new(46, unit_type, TeamId(1));

        assert_eq!(unit.shield.shield, 500.0);
        let updates = unit.update_force_field_abilities(1.0);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].shield, 500.0);
        assert!(unit.abilities[0].data > 0.0);

        unit.shield.shield = 400.0;
        let updates = unit.update_force_field_abilities(1.0);
        assert_eq!(updates.len(), 1);
        assert!((updates[0].shield - 400.4).abs() < 0.0001);
        assert!((unit.shield.shield - 400.4).abs() < 0.0001);
    }

    #[test]
    fn unit_component_ticks_shield_arc_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["ShieldArcAbility:45:0.75:2500:480:82:0:0:-20:false:8:1".into()];
        let mut unit = UnitComp::new(47, unit_type, TeamId(1));
        unit.set_pos(100.0, 200.0);
        unit.set_rotation(0.0);

        assert_eq!(unit.abilities[0].data, 2500.0);
        unit.abilities[0].data = 2000.0;
        let updates = unit.update_shield_arc_abilities(1.0);
        assert_eq!(updates.len(), 1);
        assert!((updates[0].data - 2000.75).abs() < 0.0001);
        assert!(updates[0].active);
        assert!((updates[0].x - 80.0).abs() < 0.0001);
        assert!((updates[0].y - 200.0).abs() < 0.0001);
        assert!((unit.abilities[0].data - 2000.75).abs() < 0.0001);
    }

    #[test]
    fn unit_component_plans_spawn_death_ability_from_runtime_descriptor() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["SpawnDeathAbility:renale:5:11".into()];
        let mut unit = UnitComp::new(48, unit_type, TeamId(1));
        unit.set_rotation(90.0);

        let plans = unit.spawn_death_ability_plans();
        assert_eq!(plans.len(), 5);
        assert!(plans.iter().all(|(unit, _plan)| unit == "renale"));
        assert!((plans[0].1.offset_x - 11.0).abs() < 0.0001);
        assert!(plans[0].1.offset_y.abs() < 0.0001);
        assert_eq!(plans[0].1.rotation, 0.0);
    }

    #[test]
    fn unit_component_ticks_move_effect_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["MoveEffectAbility:0:-7:4:missileTrailShort:true".into()];
        let mut unit = UnitComp::new(49, unit_type, TeamId(1));
        unit.set_pos(100.0, 200.0);
        unit.set_rotation(0.0);
        unit.vel.vel.x = 1.0;
        unit.vel.vel.y = 0.0;

        assert!(unit.update_move_effect_abilities(3.0, false).is_empty());
        assert_eq!(unit.abilities[0].data, 3.0);

        let plans = unit.update_move_effect_abilities(1.0, false);
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].effect, "missileTrailShort");
        assert!((plans[0].x - 93.0).abs() < 0.0001);
        assert!((plans[0].y - 200.0).abs() < 0.0001);
        assert!(plans[0].team_color);
        assert_eq!(unit.abilities[0].data, 0.0);
    }

    #[test]
    fn unit_component_ticks_regen_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.health = 100.0;
        unit_type.abilities = vec!["RegenAbility:1:2".into()];
        let mut unit = UnitComp::new(50, unit_type, TeamId(1));
        unit.health.damage(50.0);

        let heals = unit.update_regen_abilities(3.0);
        assert_eq!(heals, vec![9.0]);
        assert_eq!(unit.health.health, 59.0);
        assert!(unit.was_healed);
    }

    #[test]
    fn unit_component_ticks_suppression_field_ability_from_runtime_slot() {
        let mut unit_type = unit_type();
        unit_type.abilities = vec!["SuppressionFieldAbility:480:90:200:0:1:true:13".into()];
        let mut unit = UnitComp::new(43, unit_type, TeamId(1));
        unit.set_pos(100.0, 200.0);
        unit.set_rotation(0.0);

        assert!(unit.update_suppression_field_abilities(89.0).is_empty());
        assert_eq!(unit.abilities[0].data, 89.0);

        let pulses = unit.update_suppression_field_abilities(1.0);
        assert_eq!(pulses.len(), 1);
        assert!((pulses[0].x - 101.0).abs() < 0.0001);
        assert!((pulses[0].y - 200.0).abs() < 0.0001);
        assert_eq!(pulses[0].reload, 480.0);
        assert_eq!(pulses[0].max_delay, 90.0);
        assert_eq!(pulses[0].range, 200.0);
        assert_eq!(unit.abilities[0].data, 0.0);
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

    #[test]
    fn unit_component_holds_prebuild_ai_state_across_builder_and_miner_ticks() {
        let mut unit = UnitComp::new(10, unit_type(), TeamId(1));
        let block = crate::mindustry::ai::PrebuildAiBlockInfo::new(
            "router",
            crate::mindustry::r#type::Category::Distribution,
        );
        let plan = crate::mindustry::ai::PrebuildAiPlanSnapshot::new(
            crate::mindustry::game::BlockPlan::new(3, 4, 0, "router", None),
            block,
        );

        let build_step = unit.tick_prebuild_ai_builder(
            PrebuildAiRuntimeInput {
                timer_find_ready: true,
                ..PrebuildAiRuntimeInput::default()
            },
            |_| false,
            |_| false,
            Some(plan),
            |_| true,
            |_| false,
        );

        assert!(build_step.collecting_items);
        assert!(unit.prebuild_ai.collecting_items);
        assert_eq!(
            unit.builder.build_plan().unwrap().block.as_deref(),
            Some("router")
        );
        assert_eq!(unit.miner.actively_building, true);

        unit.prebuild_ai.mining = true;
        let mine_step = unit.tick_prebuild_ai_mining(
            &[PrebuildAiRequirement::new(2, 5)],
            PrebuildAiUnitInput {
                timer_ore_ready: true,
                build_cost_multiplier: 1.0,
                ..PrebuildAiUnitInput::default()
            },
            |_| 1,
            || false,
            |_, _| false,
            |_| true,
            |_| Some(77),
            |_| None,
            |pos| {
                (pos == 77).then_some(MineTile {
                    world_x: 16,
                    world_y: 24,
                    block_air: true,
                    floor_drop: Some(crate::mindustry::entities::MineItem::new("lead", 1)),
                    wall_drop: None,
                })
            },
        );

        assert_eq!(mine_step.target_item, Some(2));
        assert_eq!(unit.prebuild_ai.ore, Some(77));
        assert_eq!(
            unit.miner
                .mine_tile
                .as_ref()
                .map(|tile| (tile.world_x, tile.world_y)),
            Some((16, 24))
        );
    }

    #[test]
    fn unit_component_holds_builder_ai_state_and_applies_team_plan_claims() {
        let mut unit = UnitComp::new(12, unit_type(), TeamId(1));
        let mut team_data = crate::mindustry::game::TeamData::new(1);
        team_data.plans = vec![crate::mindustry::game::BlockPlan::new(
            6, 7, 2, "router", None,
        )];

        let step = unit.tick_builder_ai(
            &mut team_data,
            BuilderAiRuntimeInput {
                timer_find_ready: true,
                floor_is_duct: true,
                ..BuilderAiRuntimeInput::default()
            },
            |_| false,
            |_| true,
            |_| true,
            |_| false,
            |_| true,
            |_| false,
        );

        assert_eq!(step.branch, BuilderAiRuntimeBranch::FindNewPlan);
        assert_eq!(
            unit.builder_ai.last_plan,
            Some(crate::mindustry::game::BlockPlan::new(
                6, 7, 2, "router", None
            ))
        );
        assert_eq!(
            unit.builder.build_plan(),
            Some(&BuildPlan::new_place(6, 7, 2, "router"))
        );
        assert_eq!(unit.miner.actively_building, true);
        assert_eq!(step.boosting, Some(true));
    }

    #[test]
    fn unit_component_prebuild_ai_mining_deposit_updates_unit_items() {
        let mut unit = UnitComp::new(11, unit_type(), TeamId(1));
        unit.items.stack.item = Some("lead".into());
        unit.items.stack.amount = 4;
        unit.prebuild_ai.collecting_items = true;
        unit.prebuild_ai.mining = false;

        let step = unit.tick_prebuild_ai_mining(
            &[PrebuildAiRequirement::new(2, 5)],
            PrebuildAiUnitInput {
                core_accept_stack_amount: 4,
                within_core_range: true,
                build_cost_multiplier: 1.0,
                ..PrebuildAiUnitInput::default()
            },
            |_| 1,
            || true,
            |_, _| true,
            |_| true,
            |_| None,
            |_| None,
            |_| None,
        );

        assert!(step.transfer_to_core);
        assert_eq!(unit.items.stack.amount, 0);
        assert!(unit.items.stack.item.is_none());
        assert!(unit.prebuild_ai.mining);
        assert!(!unit.prebuild_ai.collecting_items);
    }

    #[test]
    fn unit_component_sync_wire_roundtrips_the_snapshot_subset() {
        let mut unit = UnitComp::new(7, unit_type(), TeamId(4));
        unit.add();
        unit.set_pos(48.0, 56.0);
        unit.set_rotation(270.0);
        unit.elevation = 0.25;
        unit.flag = 99.5;
        unit.health.health = 88.0;
        unit.weapons.is_shooting = true;
        unit.weapons.ammo = 14.0;
        unit.weapons.mounts[0].shoot = true;
        unit.weapons.mounts[0].rotate = true;
        unit.weapons.mounts[0].aim_x = 11.0;
        unit.weapons.mounts[0].aim_y = 22.0;
        unit.items.stack.item = Some("copper".into());
        unit.items.stack.amount = 8;
        unit.status.statuses.push(StatusEntry::default());
        unit.spawned_by_core = true;
        unit.builder.plans.push_back(BuildPlan::new_config(
            4,
            5,
            1,
            "router",
            TypeValue::String("cfg".into()),
        ));
        unit.miner.mine_tile = Some(MineTile {
            world_x: 32,
            world_y: 40,
            block_air: true,
            floor_drop: None,
            wall_drop: None,
        });
        unit.payload = Some(PayloadComp::new(TeamId(4), 12.0));
        unit.refresh_component_views();

        let wire = unit.to_sync_wire();
        assert_eq!(wire.mine_tile, Some(point2_pack(4, 5)));
        assert_eq!(wire.stack.item, "copper");
        assert_eq!(
            wire.plans.as_ref().unwrap()[0].config,
            TypeValue::String("cfg".into())
        );

        let mut restored = UnitComp::new(7, unit_type(), TeamId(4));
        restored.apply_sync_wire(&wire);

        assert_eq!(restored.x(), 48.0);
        assert_eq!(restored.y(), 56.0);
        assert_eq!(restored.rotation(), 270.0);
        assert_eq!(restored.elevation, 0.25);
        assert_eq!(restored.flag, 99.5);
        assert_eq!(restored.health.health, 88.0);
        assert_eq!(restored.weapons.ammo, 14.0);
        assert!(restored.weapons.is_shooting);
        assert_eq!(restored.weapons.mounts[0].aim_x, 11.0);
        assert_eq!(restored.items.stack.item.as_deref(), Some("copper"));
        assert_eq!(restored.items.stack.amount, 8);
        assert_eq!(restored.status.statuses.len(), 1);
        assert!(restored.spawned_by_core);
        assert_eq!(restored.builder.plans.len(), 1);
        assert_eq!(
            restored.builder.plans[0].config,
            TypeValue::String("cfg".into())
        );
        assert_eq!(
            restored
                .miner
                .mine_tile
                .as_ref()
                .map(|tile| (tile.world_x, tile.world_y)),
            Some((32, 40))
        );
    }
}
