//! Default game-service shell mirroring upstream `mindustry.service.GameService`.
//!
//! The Java class is the central platform service hook for achievements,
//! service statistics and event registration. This module keeps the default
//! no-op platform behavior and the deterministic state containers; individual
//! event bindings can be ported incrementally on top.

use std::collections::BTreeSet;

use crate::mindustry::game::Trigger;

use super::{
    Achievement, AchievementContext, AchievementService, AchievementState, SStat, StatService,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameServiceInitAction {
    RegisterEventsNow,
    WaitForClientLoad,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceState {
    pub tmp_tiles: Vec<i32>,
    pub blocks_built: BTreeSet<String>,
    pub units_built: BTreeSet<String>,
    pub t5s: BTreeSet<String>,
    pub checked: BTreeSet<i32>,
    pub all_transport_serpulo: Vec<String>,
    pub all_transport_erekir: Vec<String>,
    pub all_erekir_blocks: Vec<String>,
    pub all_serpulo_blocks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceContentSeed {
    pub all_transport_serpulo: Vec<String>,
    pub all_transport_erekir: Vec<String>,
    pub all_erekir_blocks: Vec<String>,
    pub all_serpulo_blocks: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceRegisterSnapshot {
    pub thorium_unlocked: bool,
    pub titanium_unlocked: bool,
    pub origin_captured: bool,
    pub planetary_terminal_captured: bool,
    pub mods_installed: bool,
    pub yes_bundle_is_router: bool,
    pub all_serpulo_sectors_captured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameServiceUpdateSnapshot {
    pub campaign: bool,
    pub player_team_unit_count: i32,
    pub player_team_poly_count: i32,
    pub core_has_all_campaign_items: bool,
    pub power_balance_per_second: Option<f32>,
    pub battery_stored: f32,
}

impl GameServiceUpdateSnapshot {
    pub const fn non_campaign() -> Self {
        Self {
            campaign: false,
            player_team_unit_count: 0,
            player_team_poly_count: 0,
            core_has_all_campaign_items: false,
            power_balance_per_second: None,
            battery_stored: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceUpdatePlan {
    pub stat_max_updates: Vec<(SStat, i32)>,
    pub achievements: BTreeSet<Achievement>,
}

impl GameServiceUpdatePlan {
    pub fn is_empty(&self) -> bool {
        self.stat_max_updates.is_empty() && self.achievements.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameServiceBlockBuildSnapshot {
    pub campaign: bool,
    pub local_unit: bool,
    pub breaking: bool,
    pub block_name: String,
    pub planet_name: Option<String>,
    pub adjacent_router: bool,
    pub boosted_by_floor: bool,
    pub all_transport_on_map: bool,
    pub linked_to_water: bool,
    pub conveyor_loop: bool,
    pub rock_break_sound: bool,
}

impl GameServiceBlockBuildSnapshot {
    pub fn placed(block_name: impl Into<String>) -> Self {
        Self {
            campaign: true,
            local_unit: true,
            breaking: false,
            block_name: block_name.into(),
            planet_name: None,
            adjacent_router: false,
            boosted_by_floor: false,
            all_transport_on_map: false,
            linked_to_water: false,
            conveyor_loop: false,
            rock_break_sound: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceBlockBuildPlan {
    pub stat_additions: Vec<SStat>,
    pub achievements: BTreeSet<Achievement>,
    pub saved_built_sets: bool,
}

impl GameServiceBlockBuildPlan {
    pub fn is_empty(&self) -> bool {
        self.stat_additions.is_empty() && self.achievements.is_empty() && !self.saved_built_sets
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameServiceUnitCreateSnapshot {
    pub campaign: bool,
    pub default_team_unit: bool,
    pub unit_name: String,
    pub visible_unit_names: Vec<String>,
}

impl GameServiceUnitCreateSnapshot {
    pub fn default_team(unit_name: impl Into<String>) -> Self {
        Self {
            campaign: true,
            default_team_unit: true,
            unit_name: unit_name.into(),
            visible_unit_names: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceUnitCreatePlan {
    pub stat_max_updates: Vec<(SStat, i32)>,
    pub achievements: BTreeSet<Achievement>,
    pub saved_built_sets: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameServiceSaveLoadSnapshot {
    pub campaign: bool,
    pub present_unit_names: Vec<String>,
    pub visible_unit_names: Vec<String>,
}

impl GameServiceSaveLoadSnapshot {
    pub fn campaign_units(unit_names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            campaign: true,
            present_unit_names: unit_names.into_iter().map(Into::into).collect(),
            visible_unit_names: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceTurnSnapshot {
    pub production_per_minute: i32,
    pub total_campaign_items: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceTurnPlan {
    pub stat_max_updates: Vec<(SStat, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceUnitDestroySnapshot {
    pub campaign: bool,
    pub enemy_unit: bool,
    pub boss: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceBlockDestroySnapshot {
    pub campaign: bool,
    pub enemy_block: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameServiceSectorLaunchLoadoutSnapshot {
    pub planet_name: Option<String>,
    pub default_loadout: bool,
}

impl GameServiceSectorLaunchLoadoutSnapshot {
    pub fn serpulo_custom() -> Self {
        Self {
            planet_name: Some("serpulo".into()),
            default_loadout: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceWinSnapshot {
    pub pvp: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServicePlayerJoinSnapshot {
    pub server: bool,
    pub player_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceClientPreConnectSnapshot {
    pub host_address: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceClientChatSnapshot {
    pub contains_alphaaaa: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceResearchSnapshot {
    pub router_unlocked: bool,
    pub micro_processor_unlocked: bool,
    pub all_researched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceUnlockSnapshot {
    pub content_name: Option<String>,
    pub research: GameServiceResearchSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameServiceTriggerSnapshot {
    pub trigger: Trigger,
    pub campaign: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceWaveSnapshot {
    pub campaign: bool,
    pub wave: i32,
    pub buildings_built: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceLaunchItemSnapshot {
    pub campaign: bool,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServicePickupSnapshot {
    pub campaign: bool,
    pub carrier_player: bool,
    pub unit_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceUnitDrownSnapshot {
    pub campaign: bool,
    pub player_unit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceNewGameSnapshot {
    pub campaign: bool,
    pub core_items_total: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameServiceFrameUpdateSnapshot {
    pub campaign: bool,
    pub hover_unit_liquid_already_achieved: bool,
    pub hover_check_due: bool,
    pub elude_on_liquid: bool,
    pub player_dead: bool,
    pub player_unit_can_boost: bool,
    pub player_unit_elevation: f32,
}

impl GameServiceFrameUpdateSnapshot {
    pub fn non_campaign() -> Self {
        Self {
            campaign: false,
            hover_unit_liquid_already_achieved: false,
            hover_check_due: false,
            elude_on_liquid: false,
            player_dead: false,
            player_unit_can_boost: false,
            player_unit_elevation: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceNucleusGroundZeroSnapshot {
    pub campaign: bool,
    pub sector_preset_name: Option<String>,
    pub block_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceBuildingBulletDestroySnapshot {
    pub campaign: bool,
    pub build_block_name: String,
    pub build_team_is_wave_team: bool,
    pub bullet_owner_unit_name: Option<String>,
    pub bullet_owner_team_is_player_team: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceGeneratorPressureExplodeSnapshot {
    pub campaign: bool,
    pub block_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceUnitBulletDestroySnapshot {
    pub campaign: bool,
    pub bullet_team_is_player_team: bool,
    pub bullet_owner_wall_build: bool,
    pub killed_unit_name: String,
    pub bullet_owner_turret_block_name: Option<String>,
    pub bullet_type_mass_driver_bolt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServicePayloadDropSnapshot {
    pub campaign: bool,
    pub unit_present: bool,
    pub carrier_team_is_default: bool,
    pub within_enemy_core_radius: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceUnitControlSnapshot {
    pub controlled_router_block: bool,
    pub controlled_turret_build: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameServiceSectorCaptureSnapshot {
    pub sector_being_played: bool,
    pub net_client: bool,
    pub wave: i32,
    pub attack_mode: bool,
    pub buildings_destroyed: i32,
    pub planet_name: Option<String>,
    pub all_planet_sectors_have_base: bool,
    pub preset_last_sector: bool,
    pub sectors_with_base: i32,
}

impl GameServiceSectorCaptureSnapshot {
    pub fn played_serpulo_attack() -> Self {
        Self {
            sector_being_played: true,
            net_client: false,
            wave: 1,
            attack_mode: true,
            buildings_destroyed: 0,
            planet_name: Some("serpulo".into()),
            all_planet_sectors_have_base: false,
            preset_last_sector: false,
            sectors_with_base: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameServiceEventPlan {
    pub stat_additions: Vec<SStat>,
    pub stat_amount_additions: Vec<(SStat, i32)>,
    pub stat_sets: Vec<(SStat, i32)>,
    pub stat_max_updates: Vec<(SStat, i32)>,
    pub achievements: BTreeSet<Achievement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameServiceApplySummary {
    pub stat_additions: usize,
    pub stat_amount_additions: usize,
    pub stat_sets: usize,
    pub stat_max_updates: usize,
    pub achievements_completed: usize,
}

impl GameServiceEventPlan {
    pub fn is_empty(&self) -> bool {
        self.stat_additions.is_empty()
            && self.stat_amount_additions.is_empty()
            && self.stat_sets.is_empty()
            && self.stat_max_updates.is_empty()
            && self.achievements.is_empty()
    }

    pub fn apply_to<S>(
        &self,
        service: &mut S,
        achievement_state: &mut AchievementState,
        context: AchievementContext,
    ) -> GameServiceApplySummary
    where
        S: AchievementService,
    {
        let mut summary = GameServiceApplySummary::default();

        for stat in &self.stat_additions {
            stat.increment(service);
            summary.stat_additions += 1;
        }

        for (stat, amount) in &self.stat_amount_additions {
            stat.add(service, *amount);
            summary.stat_amount_additions += 1;
        }

        for (stat, amount) in &self.stat_sets {
            stat.set(service, *amount);
            summary.stat_sets += 1;
        }

        for (stat, amount) in &self.stat_max_updates {
            let before = stat.get(service);
            stat.max(service, *amount);
            if *amount > before {
                summary.stat_max_updates += 1;
            }
        }

        for achievement in &self.achievements {
            let was_achieved = achievement_state.is_achieved(*achievement, &*service);
            achievement_state.complete(*achievement, service, context);
            if !was_achieved && achievement_state.is_achieved(*achievement, &*service) {
                summary.achievements_completed += 1;
            }
        }

        summary
    }
}

impl GameServiceState {
    pub fn mark_block_built(&mut self, block: impl Into<String>) {
        self.blocks_built.insert(block.into());
    }

    pub fn mark_unit_built(&mut self, unit: impl Into<String>) {
        self.units_built.insert(unit.into());
    }

    pub fn mark_checked(&mut self, packed_position: i32) {
        self.checked.insert(packed_position);
    }

    pub fn has_all_blocks_built(&self, blocks: &[impl AsRef<str>]) -> bool {
        blocks
            .iter()
            .all(|block| self.blocks_built.contains(block.as_ref()))
    }

    pub fn has_all_units_built(&self, units: &[impl AsRef<str>]) -> bool {
        units
            .iter()
            .all(|unit| self.units_built.contains(unit.as_ref()))
    }

    pub fn seed_java_t5_units(&mut self) {
        self.t5s = java_t5_units();
    }

    pub fn apply_content_seed(&mut self, seed: GameServiceContentSeed) {
        self.all_transport_serpulo = seed.all_transport_serpulo;
        self.all_transport_erekir = seed.all_transport_erekir;
        self.all_erekir_blocks = seed.all_erekir_blocks;
        self.all_serpulo_blocks = seed.all_serpulo_blocks;
    }

    pub fn register_initial_plan(
        &self,
        snapshot: GameServiceRegisterSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();

        if snapshot.thorium_unlocked {
            plan.achievements.insert(Achievement::ObtainThorium);
        }
        if snapshot.titanium_unlocked {
            plan.achievements.insert(Achievement::ObtainTitanium);
        }
        if snapshot.origin_captured {
            plan.achievements.insert(Achievement::CompleteErekir);
        }
        if snapshot.planetary_terminal_captured {
            plan.achievements.insert(Achievement::CompleteSerpulo);
        }
        if snapshot.mods_installed {
            plan.achievements.insert(Achievement::InstallMod);
        }
        if snapshot.yes_bundle_is_router {
            plan.achievements.insert(Achievement::RouterLanguage);
        }
        if snapshot.all_serpulo_sectors_captured {
            plan.achievements.insert(Achievement::CaptureAllSectors);
        }
        if !self.all_erekir_blocks.is_empty() && self.has_all_blocks_built(&self.all_erekir_blocks)
        {
            plan.achievements.insert(Achievement::AllBlocksErekir);
        }
        if !self.all_serpulo_blocks.is_empty()
            && self.has_all_blocks_built(&self.all_serpulo_blocks)
        {
            plan.achievements.insert(Achievement::AllBlocksSerpulo);
        }

        plan
    }

    pub fn check_update_plan(&self, snapshot: GameServiceUpdateSnapshot) -> GameServiceUpdatePlan {
        if !snapshot.campaign {
            return GameServiceUpdatePlan::default();
        }

        let mut plan = GameServiceUpdatePlan {
            stat_max_updates: vec![(SStat::MaxUnitActive, snapshot.player_team_unit_count)],
            achievements: BTreeSet::new(),
        };

        if snapshot.player_team_poly_count >= 10 {
            plan.achievements.insert(Achievement::Active10Polys);
        }

        if snapshot.core_has_all_campaign_items {
            plan.achievements.insert(Achievement::FillCoreAllCampaign);
        }

        if let Some(balance) = snapshot.power_balance_per_second {
            if balance < -10_000.0 {
                plan.achievements.insert(Achievement::Negative10kPower);
            }
            if balance > 100_000.0 {
                plan.achievements.insert(Achievement::Positive100kPower);
            }
        }

        if snapshot.battery_stored > 1_000_000.0 {
            plan.achievements.insert(Achievement::Store1milPower);
        }

        plan
    }

    pub fn block_build_end_plan(
        &mut self,
        snapshot: GameServiceBlockBuildSnapshot,
    ) -> GameServiceBlockBuildPlan {
        let mut plan = GameServiceBlockBuildPlan::default();
        if !snapshot.campaign || !snapshot.local_unit {
            return plan;
        }

        if snapshot.breaking {
            if snapshot.rock_break_sound {
                plan.stat_additions.push(SStat::BouldersDeconstructed);
            }
            return plan;
        }

        plan.stat_additions.push(SStat::BlocksBuilt);

        match snapshot.block_name.as_str() {
            "router" if snapshot.adjacent_router => {
                plan.achievements.insert(Achievement::ChainRouters);
            }
            "ground-factory" => {
                plan.achievements.insert(Achievement::BuildGroundFactory);
            }
            "mend-projector" => {
                plan.achievements.insert(Achievement::BuildMendProjector);
            }
            "overdrive-projector" => {
                plan.achievements
                    .insert(Achievement::BuildOverdriveProjector);
            }
            "water-extractor" if snapshot.linked_to_water => {
                plan.achievements.insert(Achievement::BuildWexWater);
            }
            _ => {}
        }

        if snapshot.boosted_by_floor {
            plan.achievements.insert(Achievement::BoostBuildingFloor);
        }

        if snapshot.all_transport_on_map {
            plan.achievements.insert(Achievement::AllTransportOneMap);
        }

        if self.blocks_built.insert(snapshot.block_name.clone()) {
            plan.saved_built_sets = true;
            let all_blocks = match snapshot.planet_name.as_deref() {
                Some("erekir") => self.has_all_blocks_built(&self.all_erekir_blocks),
                _ => self.has_all_blocks_built(&self.all_serpulo_blocks),
            };

            if all_blocks {
                match snapshot.planet_name.as_deref() {
                    Some("erekir") => {
                        plan.achievements.insert(Achievement::AllBlocksErekir);
                    }
                    _ => {
                        plan.achievements.insert(Achievement::AllBlocksSerpulo);
                    }
                }
            }
        }

        if self.blocks_built.contains("meltdown")
            && self.blocks_built.contains("spectre")
            && self.blocks_built.contains("foreshadow")
        {
            plan.achievements.insert(Achievement::BuildMeltdownSpectre);
        }

        if snapshot.conveyor_loop {
            plan.achievements.insert(Achievement::CircleConveyor);
        }

        plan
    }

    pub fn unit_create_plan(
        &mut self,
        snapshot: GameServiceUnitCreateSnapshot,
    ) -> GameServiceUnitCreatePlan {
        let mut plan = GameServiceUnitCreatePlan::default();
        if !snapshot.campaign || !snapshot.default_team_unit {
            return plan;
        }

        if self.units_built.insert(snapshot.unit_name.clone()) {
            let visible_built = snapshot
                .visible_unit_names
                .iter()
                .filter(|unit| self.units_built.contains(unit.as_str()))
                .count() as i32;
            plan.stat_max_updates
                .push((SStat::UnitTypesBuilt, visible_built));
            plan.saved_built_sets = true;
        }

        if self.t5s.contains(&snapshot.unit_name) {
            plan.achievements.insert(Achievement::BuildT5);
        }

        plan
    }

    pub fn save_load_plan(
        &mut self,
        snapshot: GameServiceSaveLoadSnapshot,
    ) -> GameServiceUnitCreatePlan {
        let mut plan = GameServiceUnitCreatePlan::default();
        if !snapshot.campaign {
            return plan;
        }

        let mut added = false;
        for unit_name in snapshot.present_unit_names {
            if self.t5s.contains(&unit_name) {
                plan.achievements.insert(Achievement::BuildT5);
            }

            if self.units_built.insert(unit_name) {
                added = true;
            }
        }

        if added {
            let visible_built = snapshot
                .visible_unit_names
                .iter()
                .filter(|unit| self.units_built.contains(unit.as_str()))
                .count() as i32;
            plan.stat_max_updates
                .push((SStat::UnitTypesBuilt, visible_built));
            plan.saved_built_sets = true;
        }

        plan
    }

    pub fn turn_plan(&self, snapshot: GameServiceTurnSnapshot) -> GameServiceTurnPlan {
        GameServiceTurnPlan {
            stat_max_updates: vec![
                (SStat::MaxProduction, snapshot.production_per_minute),
                (SStat::TotalCampaignItems, snapshot.total_campaign_items),
            ],
        }
    }

    pub fn unit_destroy_plan(
        &self,
        snapshot: GameServiceUnitDestroySnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign && snapshot.enemy_unit {
            plan.stat_additions.push(SStat::UnitsDestroyed);
            if snapshot.boss {
                plan.stat_additions.push(SStat::BossesDefeated);
            }
        }
        plan
    }

    pub fn block_destroy_plan(
        &self,
        snapshot: GameServiceBlockDestroySnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign && snapshot.enemy_block {
            plan.stat_additions.push(SStat::BlocksDestroyed);
        }
        plan
    }

    pub fn schematic_create_plan(&self) -> GameServiceEventPlan {
        GameServiceEventPlan {
            stat_additions: vec![SStat::SchematicsCreated],
            ..Default::default()
        }
    }

    pub fn sector_launch_loadout_plan(
        &self,
        snapshot: GameServiceSectorLaunchLoadoutSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.planet_name.as_deref() == Some("serpulo") && !snapshot.default_loadout {
            plan.achievements.insert(Achievement::LaunchCoreSchematic);
        }
        plan
    }

    pub fn map_make_plan(&self) -> GameServiceEventPlan {
        GameServiceEventPlan {
            stat_additions: vec![SStat::MapsMade],
            ..Default::default()
        }
    }

    pub fn map_publish_plan(&self) -> GameServiceEventPlan {
        GameServiceEventPlan {
            stat_additions: vec![SStat::MapsPublished],
            ..Default::default()
        }
    }

    pub fn sector_launch_plan(&self) -> GameServiceEventPlan {
        GameServiceEventPlan {
            stat_additions: vec![SStat::TimesLaunched],
            ..Default::default()
        }
    }

    pub fn win_plan(&self, snapshot: GameServiceWinSnapshot) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.pvp {
            plan.stat_additions.push(SStat::PvpsWon);
        }
        plan
    }

    pub fn player_join_plan(
        &self,
        snapshot: GameServicePlayerJoinSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.server {
            plan.stat_max_updates
                .push((SStat::MaxPlayersServer, snapshot.player_count));
        }
        plan
    }

    pub fn client_pre_connect_plan(
        &self,
        snapshot: GameServiceClientPreConnectSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if let Some(host_address) = snapshot.host_address.as_deref() {
            if !host_address.starts_with("steam:") && !host_address.starts_with("192.") {
                plan.achievements.insert(Achievement::JoinCommunityServer);
            }
        }
        plan
    }

    pub fn client_chat_plan(
        &self,
        snapshot: GameServiceClientChatSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.contains_alphaaaa {
            plan.achievements.insert(Achievement::UseAnimdustryEmoji);
        }
        plan
    }

    pub fn research_plan(&self, snapshot: GameServiceResearchSnapshot) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.router_unlocked {
            plan.achievements.insert(Achievement::ResearchRouter);
        }
        if snapshot.micro_processor_unlocked {
            plan.achievements.insert(Achievement::ResearchLogic);
        }
        if snapshot.all_researched {
            plan.achievements.insert(Achievement::ResearchAll);
        }
        plan
    }

    pub fn unlock_plan(&self, snapshot: GameServiceUnlockSnapshot) -> GameServiceEventPlan {
        let mut plan = self.research_plan(snapshot.research);
        match snapshot.content_name.as_deref() {
            Some("thorium") => {
                plan.achievements.insert(Achievement::ObtainThorium);
            }
            Some("titanium") => {
                plan.achievements.insert(Achievement::ObtainTitanium);
            }
            _ => {}
        }
        plan
    }

    pub fn trigger_plan(&self, snapshot: GameServiceTriggerSnapshot) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();

        match snapshot.trigger {
            Trigger::OpenConsole => {
                plan.achievements.insert(Achievement::OpenConsole);
            }
            Trigger::OpenWiki => {
                plan.achievements.insert(Achievement::OpenWiki);
            }
            Trigger::ImportMod => {
                plan.achievements.insert(Achievement::InstallMod);
            }
            Trigger::ExclusionDeath => {
                plan.achievements.insert(Achievement::DieExclusion);
            }
            Trigger::EnablePixelation => {
                plan.achievements.insert(Achievement::EnablePixelation);
            }
            Trigger::UnitCommandAttack if snapshot.campaign => {
                plan.achievements.insert(Achievement::IssueAttackCommand);
            }
            Trigger::UnitCommandBoost if snapshot.campaign => {
                plan.achievements.insert(Achievement::BoostUnit);
            }
            Trigger::ThoriumReactorOverheat if snapshot.campaign => {
                plan.stat_additions.push(SStat::ReactorsOverheated);
            }
            Trigger::ImpactPower if snapshot.campaign => {
                plan.achievements.insert(Achievement::PowerupImpactReactor);
            }
            Trigger::FlameAmmo if snapshot.campaign => {
                plan.achievements.insert(Achievement::UseFlameAmmo);
            }
            Trigger::TurretCool if snapshot.campaign => {
                plan.achievements.insert(Achievement::CoolTurret);
            }
            Trigger::SuicideBomb if snapshot.campaign => {
                plan.achievements.insert(Achievement::SuicideBomb);
            }
            Trigger::BlastGenerator if snapshot.campaign => {
                plan.achievements.insert(Achievement::BlastGenerator);
            }
            Trigger::ForceProjectorBreak if snapshot.campaign => {
                plan.achievements.insert(Achievement::BreakForceProjector);
            }
            Trigger::NeoplasmReact if snapshot.campaign => {
                plan.achievements.insert(Achievement::NeoplasmWater);
            }
            Trigger::ShockwaveTowerUse if snapshot.campaign => {
                plan.achievements.insert(Achievement::ShockwaveTowerUse);
            }
            Trigger::Shock if snapshot.campaign => {
                plan.achievements.insert(Achievement::ShockWetEnemy);
            }
            Trigger::BlastFreeze if snapshot.campaign => {
                plan.achievements.insert(Achievement::BlastFrozenUnit);
            }
            _ => {}
        }

        plan
    }

    pub fn wave_plan(&self, snapshot: GameServiceWaveSnapshot) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign {
            plan.stat_max_updates
                .push((SStat::MaxWavesSurvived, snapshot.wave));
            if snapshot.buildings_built == 0 && snapshot.wave >= 10 {
                plan.achievements
                    .insert(Achievement::Survive10WavesNoBlocks);
            }
        }
        plan
    }

    pub fn launch_item_plan(
        &self,
        snapshot: GameServiceLaunchItemSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan {
            stat_amount_additions: vec![(SStat::ItemsLaunched, snapshot.amount)],
            ..Default::default()
        };
        if snapshot.campaign {
            plan.achievements.insert(Achievement::LaunchItemPad);
        }
        plan
    }

    pub fn pickup_plan(&self, snapshot: GameServicePickupSnapshot) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign
            && snapshot.carrier_player
            && snapshot
                .unit_name
                .as_deref()
                .is_some_and(|unit| self.t5s.contains(unit))
        {
            plan.achievements.insert(Achievement::PickupT5);
        }
        plan
    }

    pub fn unit_drown_plan(&self, snapshot: GameServiceUnitDrownSnapshot) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign && snapshot.player_unit {
            plan.achievements.insert(Achievement::Drown);
        }
        plan
    }

    pub fn new_game_plan(&self, snapshot: GameServiceNewGameSnapshot) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign && snapshot.core_items_total >= 10_000 {
            plan.achievements.insert(Achievement::Drop10kitems);
        }
        plan
    }

    pub fn frame_update_plan(
        &self,
        snapshot: GameServiceFrameUpdateSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if !snapshot.campaign {
            return plan;
        }

        if !snapshot.hover_unit_liquid_already_achieved
            && snapshot.hover_check_due
            && snapshot.elude_on_liquid
        {
            plan.achievements.insert(Achievement::HoverUnitLiquid);
        }

        if !snapshot.player_dead
            && snapshot.player_unit_can_boost
            && snapshot.player_unit_elevation >= 0.25
        {
            plan.achievements.insert(Achievement::BoostUnit);
        }

        plan
    }

    pub fn nucleus_ground_zero_plan(
        &self,
        snapshot: GameServiceNucleusGroundZeroSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign
            && snapshot.sector_preset_name.as_deref() == Some("groundZero")
            && snapshot.block_name == "core-nucleus"
        {
            plan.achievements.insert(Achievement::NucleusGroundZero);
        }
        plan
    }

    pub fn building_bullet_destroy_plan(
        &self,
        snapshot: GameServiceBuildingBulletDestroySnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign
            && snapshot.build_block_name == "scatter"
            && snapshot.build_team_is_wave_team
            && snapshot.bullet_owner_unit_name.as_deref() == Some("flare")
            && snapshot.bullet_owner_team_is_player_team
        {
            plan.achievements.insert(Achievement::DestroyScatterFlare);
        }
        plan
    }

    pub fn generator_pressure_explode_plan(
        &self,
        snapshot: GameServiceGeneratorPressureExplodeSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign && snapshot.block_name == "neoplasia-reactor" {
            plan.achievements.insert(Achievement::NeoplasiaExplosion);
        }
        plan
    }

    pub fn unit_bullet_destroy_plan(
        &self,
        snapshot: GameServiceUnitBulletDestroySnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if !snapshot.campaign || !snapshot.bullet_team_is_player_team {
            return plan;
        }

        if snapshot.bullet_owner_wall_build {
            plan.achievements.insert(Achievement::KillEnemyPhaseWall);
        }

        if snapshot.killed_unit_name == "eclipse"
            && snapshot.bullet_owner_turret_block_name.as_deref() == Some("duo")
        {
            plan.achievements.insert(Achievement::KillEclipseDuo);
        }

        if snapshot.bullet_type_mass_driver_bolt {
            plan.achievements.insert(Achievement::KillMassDriver);
        }

        plan
    }

    pub fn payload_drop_plan(
        &self,
        snapshot: GameServicePayloadDropSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.campaign
            && snapshot.unit_present
            && snapshot.carrier_team_is_default
            && snapshot.within_enemy_core_radius
        {
            plan.achievements.insert(Achievement::DropUnitsCoreZone);
        }
        plan
    }

    pub fn unit_control_plan(
        &self,
        snapshot: GameServiceUnitControlSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        if snapshot.controlled_router_block {
            plan.achievements.insert(Achievement::BecomeRouter);
        }
        if snapshot.controlled_turret_build {
            plan.achievements.insert(Achievement::ControlTurret);
        }
        plan
    }

    pub fn sector_capture_plan(
        &self,
        snapshot: GameServiceSectorCaptureSnapshot,
    ) -> GameServiceEventPlan {
        let mut plan = GameServiceEventPlan::default();
        let played_or_client = snapshot.sector_being_played || snapshot.net_client;

        if played_or_client {
            if snapshot.wave <= 5 && snapshot.attack_mode {
                plan.achievements.insert(Achievement::DefeatAttack5Waves);
            }

            if snapshot.buildings_destroyed == 0 {
                plan.achievements.insert(Achievement::CaptureNoBlocksBroken);
            }
        }

        if snapshot.attack_mode {
            plan.stat_additions.push(SStat::AttacksWon);
        }

        if !snapshot.sector_being_played && !snapshot.net_client {
            plan.achievements.insert(Achievement::CaptureBackground);
        }

        match snapshot.planet_name.as_deref() {
            Some("serpulo") => {
                if snapshot.all_planet_sectors_have_base {
                    plan.achievements.insert(Achievement::CaptureAllSectors);
                }
                if snapshot.preset_last_sector {
                    plan.achievements.insert(Achievement::CompleteSerpulo);
                }

                // Keep Java's current behavior: sectorsControlled is set only
                // for Serpulo captures, using the planet-wide hasBase count.
                plan.stat_sets
                    .push((SStat::SectorsControlled, snapshot.sectors_with_base));
            }
            Some("erekir") if snapshot.preset_last_sector => {
                plan.achievements.insert(Achievement::CompleteErekir);
            }
            _ => {}
        }

        plan
    }
}

pub trait GameService: AchievementService {
    fn init(&mut self, client_loaded: bool) -> GameServiceInitAction {
        if client_loaded {
            self.register_events();
            GameServiceInitAction::RegisterEventsNow
        } else {
            GameServiceInitAction::WaitForClientLoad
        }
    }

    fn enabled(&self) -> bool {
        false
    }

    fn register_events(&mut self) {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DefaultGameService {
    state: GameServiceState,
    events_registered: bool,
}

impl DefaultGameService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_content_seed(seed: GameServiceContentSeed) -> Self {
        let mut service = Self::new();
        service.apply_content_seed(seed);
        service
    }

    pub fn state(&self) -> &GameServiceState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut GameServiceState {
        &mut self.state
    }

    pub fn events_registered(&self) -> bool {
        self.events_registered
    }

    pub fn apply_content_seed(&mut self, seed: GameServiceContentSeed) {
        self.state.apply_content_seed(seed);
    }
}

impl StatService for DefaultGameService {}

impl AchievementService for DefaultGameService {}

impl GameService for DefaultGameService {
    fn register_events(&mut self) {
        self.events_registered = true;
        self.state.seed_java_t5_units();
    }
}

pub fn java_t5_units() -> BTreeSet<String> {
    ["omura", "reign", "toxopid", "eclipse", "oct", "corvus"]
        .into_iter()
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::mindustry::service::{Achievement, AchievementContext, AchievementState, SStat};

    #[derive(Debug, Default)]
    struct RecordingService {
        stats: BTreeMap<String, i32>,
        completed: BTreeSet<String>,
        stores: usize,
    }

    impl StatService for RecordingService {
        fn get_stat(&self, name: &str, def: i32) -> i32 {
            self.stats.get(name).copied().unwrap_or(def)
        }

        fn set_stat(&mut self, name: &str, amount: i32) {
            self.stats.insert(name.into(), amount);
        }

        fn store_stats(&mut self) {
            self.stores += 1;
        }
    }

    impl AchievementService for RecordingService {
        fn complete_achievement(&mut self, name: &str) {
            self.completed.insert(name.into());
        }

        fn clear_achievement(&mut self, name: &str) {
            self.completed.remove(name);
        }

        fn is_achieved(&self, name: &str) -> bool {
            self.completed.contains(name)
        }
    }

    #[test]
    fn default_game_service_platform_methods_are_noop_like_java_defaults() {
        let mut service = DefaultGameService::new();
        let mut achievements = AchievementState::new();

        assert!(!service.enabled());
        assert_eq!(SStat::UnitsDestroyed.get(&service), 0);

        SStat::UnitsDestroyed.set(&mut service, 10);
        assert_eq!(SStat::UnitsDestroyed.get(&service), 0);

        achievements.complete(
            Achievement::OpenWiki,
            &mut service,
            AchievementContext::normal(),
        );
        assert!(!achievements.is_achieved(Achievement::OpenWiki, &service));
    }

    #[test]
    fn init_matches_java_loaded_now_or_wait_for_client_load_shape() {
        let mut service = DefaultGameService::new();

        assert_eq!(
            service.init(false),
            GameServiceInitAction::WaitForClientLoad
        );
        assert!(!service.events_registered());

        assert_eq!(service.init(true), GameServiceInitAction::RegisterEventsNow);
        assert!(service.events_registered());
    }

    #[test]
    fn register_events_seeds_java_t5_unit_set_for_later_event_checks() {
        let mut service = DefaultGameService::new();
        service.register_events();

        assert_eq!(service.state().t5s, java_t5_units());
        assert!(service.state().t5s.contains("omura"));
        assert!(service.state().t5s.contains("corvus"));
    }

    #[test]
    fn content_seed_injects_java_startup_block_lists_without_content_registry_guessing() {
        let seed = GameServiceContentSeed {
            all_transport_serpulo: vec!["conveyor".into(), "router".into()],
            all_transport_erekir: vec!["duct".into()],
            all_erekir_blocks: vec!["duct".into(), "beam-node".into()],
            all_serpulo_blocks: vec!["router".into(), "conveyor".into()],
        };
        let mut service = DefaultGameService::with_content_seed(seed);

        assert_eq!(
            service.state().all_transport_serpulo,
            vec!["conveyor", "router"]
        );
        assert_eq!(service.state().all_transport_erekir, vec!["duct"]);

        service.state_mut().mark_block_built("router");
        service.state_mut().mark_block_built("conveyor");
        service.state_mut().mark_block_built("duct");
        service.state_mut().mark_block_built("beam-node");

        let plan = service
            .state()
            .register_initial_plan(GameServiceRegisterSnapshot::default());

        assert!(plan.achievements.contains(&Achievement::AllBlocksSerpulo));
        assert!(plan.achievements.contains(&Achievement::AllBlocksErekir));
    }

    #[test]
    fn register_initial_plan_matches_java_one_time_startup_checks() {
        let mut state = GameServiceState {
            all_serpulo_blocks: vec!["router".into(), "conveyor".into()],
            all_erekir_blocks: vec!["duct".into()],
            ..Default::default()
        };
        state.mark_block_built("router");
        state.mark_block_built("conveyor");
        state.mark_block_built("duct");

        let plan = state.register_initial_plan(GameServiceRegisterSnapshot {
            thorium_unlocked: true,
            titanium_unlocked: true,
            origin_captured: true,
            planetary_terminal_captured: true,
            mods_installed: true,
            yes_bundle_is_router: true,
            all_serpulo_sectors_captured: true,
        });

        for achievement in [
            Achievement::ObtainThorium,
            Achievement::ObtainTitanium,
            Achievement::CompleteErekir,
            Achievement::CompleteSerpulo,
            Achievement::InstallMod,
            Achievement::RouterLanguage,
            Achievement::CaptureAllSectors,
            Achievement::AllBlocksErekir,
            Achievement::AllBlocksSerpulo,
        ] {
            assert!(plan.achievements.contains(&achievement), "{achievement:?}");
        }

        assert!(GameServiceState::default()
            .register_initial_plan(GameServiceRegisterSnapshot::default())
            .is_empty());
    }

    #[test]
    fn service_state_tracks_built_blocks_units_and_checked_positions() {
        let mut state = GameServiceState::default();

        state.mark_block_built("router");
        state.mark_block_built("conveyor");
        state.mark_unit_built("dagger");
        state.mark_checked(1234);

        assert!(state.has_all_blocks_built(&["router", "conveyor"]));
        assert!(!state.has_all_blocks_built(&["router", "junction"]));
        assert!(state.has_all_units_built(&["dagger"]));
        assert!(state.checked.contains(&1234));
    }

    #[test]
    fn update_plan_ignores_non_campaign_like_java_check_update() {
        let state = GameServiceState::default();

        assert!(state
            .check_update_plan(GameServiceUpdateSnapshot::non_campaign())
            .is_empty());
    }

    #[test]
    fn update_plan_matches_java_campaign_periodic_checks() {
        let state = GameServiceState::default();
        let plan = state.check_update_plan(GameServiceUpdateSnapshot {
            campaign: true,
            player_team_unit_count: 12,
            player_team_poly_count: 10,
            core_has_all_campaign_items: true,
            power_balance_per_second: Some(120_000.0),
            battery_stored: 1_500_000.0,
        });

        assert_eq!(plan.stat_max_updates, vec![(SStat::MaxUnitActive, 12)]);
        assert!(plan.achievements.contains(&Achievement::Active10Polys));
        assert!(plan
            .achievements
            .contains(&Achievement::FillCoreAllCampaign));
        assert!(plan.achievements.contains(&Achievement::Positive100kPower));
        assert!(plan.achievements.contains(&Achievement::Store1milPower));
        assert!(!plan.achievements.contains(&Achievement::Negative10kPower));
    }

    #[test]
    fn update_plan_keeps_power_thresholds_strict_like_java() {
        let state = GameServiceState::default();
        let plan = state.check_update_plan(GameServiceUpdateSnapshot {
            campaign: true,
            player_team_unit_count: 0,
            player_team_poly_count: 0,
            core_has_all_campaign_items: false,
            power_balance_per_second: Some(-10_001.0),
            battery_stored: 1_000_000.0,
        });

        assert!(plan.achievements.contains(&Achievement::Negative10kPower));
        assert!(!plan.achievements.contains(&Achievement::Store1milPower));
    }

    #[test]
    fn block_build_plan_matches_java_place_event_achievements() {
        let mut state = GameServiceState::default();
        let mut snapshot = GameServiceBlockBuildSnapshot::placed("router");
        snapshot.adjacent_router = true;
        snapshot.conveyor_loop = true;

        let plan = state.block_build_end_plan(snapshot);

        assert_eq!(plan.stat_additions, vec![SStat::BlocksBuilt]);
        assert!(plan.achievements.contains(&Achievement::ChainRouters));
        assert!(plan.achievements.contains(&Achievement::CircleConveyor));
        assert!(plan.saved_built_sets);
        assert!(state.blocks_built.contains("router"));
    }

    #[test]
    fn block_build_plan_tracks_special_blocks_and_floor_boosts() {
        let mut state = GameServiceState::default();
        let mut snapshot = GameServiceBlockBuildSnapshot::placed("water-extractor");
        snapshot.linked_to_water = true;
        snapshot.boosted_by_floor = true;

        let plan = state.block_build_end_plan(snapshot);

        assert!(plan.achievements.contains(&Achievement::BuildWexWater));
        assert!(plan.achievements.contains(&Achievement::BoostBuildingFloor));

        let plan =
            state.block_build_end_plan(GameServiceBlockBuildSnapshot::placed("ground-factory"));
        assert!(plan.achievements.contains(&Achievement::BuildGroundFactory));
    }

    #[test]
    fn block_build_plan_completes_all_blocks_and_meltdown_spectre_foreshadow() {
        let mut state = GameServiceState {
            all_serpulo_blocks: vec!["meltdown".into(), "spectre".into(), "foreshadow".into()],
            ..Default::default()
        };

        state.block_build_end_plan(GameServiceBlockBuildSnapshot::placed("meltdown"));
        state.block_build_end_plan(GameServiceBlockBuildSnapshot::placed("spectre"));
        let plan = state.block_build_end_plan(GameServiceBlockBuildSnapshot::placed("foreshadow"));

        assert!(plan
            .achievements
            .contains(&Achievement::BuildMeltdownSpectre));
        assert!(plan.achievements.contains(&Achievement::AllBlocksSerpulo));
    }

    #[test]
    fn block_build_plan_counts_boulder_breaks_only_for_local_campaign_breaking() {
        let mut state = GameServiceState::default();
        let mut snapshot = GameServiceBlockBuildSnapshot::placed("boulder");
        snapshot.breaking = true;
        snapshot.rock_break_sound = true;

        let plan = state.block_build_end_plan(snapshot);

        assert_eq!(plan.stat_additions, vec![SStat::BouldersDeconstructed]);
        assert!(plan.achievements.is_empty());
        assert!(!plan.saved_built_sets);
    }

    #[test]
    fn unit_create_plan_tracks_new_default_team_units_and_t5_completion() {
        let mut state = GameServiceState::default();
        state.seed_java_t5_units();

        let mut snapshot = GameServiceUnitCreateSnapshot::default_team("omura");
        snapshot.visible_unit_names = vec!["dagger".into(), "omura".into()];

        let plan = state.unit_create_plan(snapshot);

        assert_eq!(plan.stat_max_updates, vec![(SStat::UnitTypesBuilt, 1)]);
        assert!(plan.achievements.contains(&Achievement::BuildT5));
        assert!(plan.saved_built_sets);
        assert!(state.units_built.contains("omura"));
    }

    #[test]
    fn unit_create_plan_ignores_non_campaign_or_enemy_units() {
        let mut state = GameServiceState::default();

        let mut snapshot = GameServiceUnitCreateSnapshot::default_team("dagger");
        snapshot.campaign = false;
        assert_eq!(
            state.unit_create_plan(snapshot),
            GameServiceUnitCreatePlan::default()
        );

        let mut snapshot = GameServiceUnitCreateSnapshot::default_team("dagger");
        snapshot.default_team_unit = false;
        assert_eq!(
            state.unit_create_plan(snapshot),
            GameServiceUnitCreatePlan::default()
        );
    }

    #[test]
    fn save_load_plan_backfills_existing_units_and_t5_like_java_delayed_scan() {
        let mut state = GameServiceState::default();
        state.seed_java_t5_units();
        state.mark_unit_built("dagger");

        let mut snapshot = GameServiceSaveLoadSnapshot::campaign_units(["dagger", "omura"]);
        snapshot.visible_unit_names = vec!["dagger".into(), "omura".into(), "hidden-test".into()];

        let plan = state.save_load_plan(snapshot);

        assert!(plan.achievements.contains(&Achievement::BuildT5));
        assert_eq!(plan.stat_max_updates, vec![(SStat::UnitTypesBuilt, 2)]);
        assert!(plan.saved_built_sets);
        assert!(state.units_built.contains("dagger"));
        assert!(state.units_built.contains("omura"));

        assert_eq!(
            state.save_load_plan(GameServiceSaveLoadSnapshot {
                campaign: false,
                present_unit_names: vec!["corvus".into()],
                visible_unit_names: vec!["corvus".into()],
            }),
            GameServiceUnitCreatePlan::default()
        );
    }

    #[test]
    fn turn_plan_updates_production_and_total_campaign_item_max_stats() {
        let state = GameServiceState::default();

        let plan = state.turn_plan(GameServiceTurnSnapshot {
            production_per_minute: 6000,
            total_campaign_items: 12345,
        });

        assert_eq!(
            plan.stat_max_updates,
            vec![
                (SStat::MaxProduction, 6000),
                (SStat::TotalCampaignItems, 12345)
            ]
        );
    }

    #[test]
    fn unit_destroy_plan_counts_enemy_and_boss_kills_like_java_event() {
        let state = GameServiceState::default();

        let plan = state.unit_destroy_plan(GameServiceUnitDestroySnapshot {
            campaign: true,
            enemy_unit: true,
            boss: true,
        });

        assert_eq!(
            plan.stat_additions,
            vec![SStat::UnitsDestroyed, SStat::BossesDefeated]
        );

        assert!(state
            .unit_destroy_plan(GameServiceUnitDestroySnapshot {
                campaign: false,
                enemy_unit: true,
                boss: true,
            })
            .is_empty());
        assert!(state
            .unit_destroy_plan(GameServiceUnitDestroySnapshot {
                campaign: true,
                enemy_unit: false,
                boss: true,
            })
            .is_empty());
    }

    #[test]
    fn block_destroy_and_schematic_create_plans_match_java_stat_events() {
        let state = GameServiceState::default();

        let block_plan = state.block_destroy_plan(GameServiceBlockDestroySnapshot {
            campaign: true,
            enemy_block: true,
        });
        assert_eq!(block_plan.stat_additions, vec![SStat::BlocksDestroyed]);

        assert!(state
            .block_destroy_plan(GameServiceBlockDestroySnapshot {
                campaign: true,
                enemy_block: false,
            })
            .is_empty());

        let schematic_plan = state.schematic_create_plan();
        assert_eq!(
            schematic_plan.stat_additions,
            vec![SStat::SchematicsCreated]
        );
    }

    #[test]
    fn launch_loadout_and_unit_control_plans_complete_direct_achievements() {
        let state = GameServiceState::default();

        let loadout_plan = state
            .sector_launch_loadout_plan(GameServiceSectorLaunchLoadoutSnapshot::serpulo_custom());
        assert!(loadout_plan
            .achievements
            .contains(&Achievement::LaunchCoreSchematic));

        let default_loadout_plan =
            state.sector_launch_loadout_plan(GameServiceSectorLaunchLoadoutSnapshot {
                planet_name: Some("serpulo".into()),
                default_loadout: true,
            });
        assert!(default_loadout_plan.is_empty());

        let control_plan = state.unit_control_plan(GameServiceUnitControlSnapshot {
            controlled_router_block: true,
            controlled_turret_build: true,
        });
        assert!(control_plan
            .achievements
            .contains(&Achievement::BecomeRouter));
        assert!(control_plan
            .achievements
            .contains(&Achievement::ControlTurret));
    }

    #[test]
    fn sector_capture_plan_matches_played_serpulo_attack_branches() {
        let state = GameServiceState::default();
        let mut snapshot = GameServiceSectorCaptureSnapshot::played_serpulo_attack();
        snapshot.all_planet_sectors_have_base = true;
        snapshot.preset_last_sector = true;
        snapshot.sectors_with_base = 64;

        let plan = state.sector_capture_plan(snapshot);

        assert_eq!(plan.stat_additions, vec![SStat::AttacksWon]);
        assert_eq!(plan.stat_sets, vec![(SStat::SectorsControlled, 64)]);
        assert!(plan.achievements.contains(&Achievement::DefeatAttack5Waves));
        assert!(plan
            .achievements
            .contains(&Achievement::CaptureNoBlocksBroken));
        assert!(plan.achievements.contains(&Achievement::CaptureAllSectors));
        assert!(plan.achievements.contains(&Achievement::CompleteSerpulo));
        assert!(!plan.achievements.contains(&Achievement::CaptureBackground));
    }

    #[test]
    fn sector_capture_plan_matches_background_and_erekir_last_sector() {
        let state = GameServiceState::default();

        let plan = state.sector_capture_plan(GameServiceSectorCaptureSnapshot {
            sector_being_played: false,
            net_client: false,
            wave: 20,
            attack_mode: false,
            buildings_destroyed: 3,
            planet_name: Some("erekir".into()),
            all_planet_sectors_have_base: true,
            preset_last_sector: true,
            sectors_with_base: 12,
        });

        assert!(plan.stat_additions.is_empty());
        assert!(plan.stat_sets.is_empty());
        assert!(plan.achievements.contains(&Achievement::CaptureBackground));
        assert!(plan.achievements.contains(&Achievement::CompleteErekir));
        assert!(!plan.achievements.contains(&Achievement::CaptureAllSectors));
        assert!(!plan
            .achievements
            .contains(&Achievement::CaptureNoBlocksBroken));
    }

    #[test]
    fn simple_event_plans_match_java_stat_and_achievement_branches() {
        let state = GameServiceState::default();

        assert_eq!(
            state.map_make_plan(),
            GameServiceEventPlan {
                stat_additions: vec![SStat::MapsMade],
                ..Default::default()
            }
        );
        assert_eq!(
            state.map_publish_plan(),
            GameServiceEventPlan {
                stat_additions: vec![SStat::MapsPublished],
                ..Default::default()
            }
        );
        assert_eq!(
            state.sector_launch_plan(),
            GameServiceEventPlan {
                stat_additions: vec![SStat::TimesLaunched],
                ..Default::default()
            }
        );
        assert_eq!(
            state.win_plan(GameServiceWinSnapshot { pvp: true }),
            GameServiceEventPlan {
                stat_additions: vec![SStat::PvpsWon],
                ..Default::default()
            }
        );
        assert_eq!(
            state.player_join_plan(GameServicePlayerJoinSnapshot {
                server: true,
                player_count: 42,
            }),
            GameServiceEventPlan {
                stat_max_updates: vec![(SStat::MaxPlayersServer, 42)],
                ..Default::default()
            }
        );

        let join_community_plan =
            state.client_pre_connect_plan(GameServiceClientPreConnectSnapshot {
                host_address: Some("example.org".into()),
            });
        assert!(join_community_plan
            .achievements
            .contains(&Achievement::JoinCommunityServer));
        assert!(state
            .client_pre_connect_plan(GameServiceClientPreConnectSnapshot {
                host_address: Some("steam:123".into()),
            })
            .is_empty());
        assert!(state
            .client_pre_connect_plan(GameServiceClientPreConnectSnapshot {
                host_address: Some("192.168.0.1".into()),
            })
            .is_empty());
        assert!(state
            .client_pre_connect_plan(GameServiceClientPreConnectSnapshot { host_address: None })
            .is_empty());

        let emoji_plan = state.client_chat_plan(GameServiceClientChatSnapshot {
            contains_alphaaaa: true,
        });
        assert!(emoji_plan
            .achievements
            .contains(&Achievement::UseAnimdustryEmoji));
        assert!(state
            .client_chat_plan(GameServiceClientChatSnapshot {
                contains_alphaaaa: false,
            })
            .is_empty());

        assert!(state
            .win_plan(GameServiceWinSnapshot { pvp: false })
            .is_empty());
        assert!(state
            .player_join_plan(GameServicePlayerJoinSnapshot {
                server: false,
                player_count: 99,
            })
            .is_empty());
    }

    #[test]
    fn event_plan_apply_to_writes_stats_and_achievements_into_service_runtime() {
        let mut service = RecordingService::default();
        let mut achievements = AchievementState::new();
        let mut plan = GameServiceEventPlan {
            stat_additions: vec![SStat::MapsMade],
            stat_amount_additions: vec![(SStat::ItemsLaunched, 30)],
            stat_sets: vec![(SStat::SectorsControlled, 7)],
            stat_max_updates: vec![(SStat::MaxPlayersServer, 4)],
            ..Default::default()
        };
        plan.achievements.insert(Achievement::JoinCommunityServer);

        let summary = plan.apply_to(
            &mut service,
            &mut achievements,
            AchievementContext::normal(),
        );

        assert_eq!(
            summary,
            GameServiceApplySummary {
                stat_additions: 1,
                stat_amount_additions: 1,
                stat_sets: 1,
                stat_max_updates: 1,
                achievements_completed: 1,
            }
        );
        assert_eq!(service.stats.get("mapsMade"), Some(&1));
        assert_eq!(service.stats.get("itemsLaunched"), Some(&30));
        assert_eq!(service.stats.get("sectorsControlled"), Some(&7));
        assert_eq!(service.stats.get("maxPlayersServer"), Some(&4));
        assert!(service.completed.contains("joinCommunityServer"));
        assert_eq!(service.stores, 5);

        let summary = plan.apply_to(
            &mut service,
            &mut achievements,
            AchievementContext::normal(),
        );
        assert_eq!(summary.achievements_completed, 0);
        assert_eq!(summary.stat_max_updates, 0);
        assert_eq!(service.stats.get("mapsMade"), Some(&2));
        assert_eq!(service.stats.get("itemsLaunched"), Some(&60));
        assert_eq!(service.stats.get("maxPlayersServer"), Some(&4));
    }

    #[test]
    fn research_and_unlock_plans_match_java_check_unlocks_branches() {
        let state = GameServiceState::default();

        let research_plan = state.research_plan(GameServiceResearchSnapshot {
            router_unlocked: true,
            micro_processor_unlocked: true,
            all_researched: true,
        });
        assert!(research_plan
            .achievements
            .contains(&Achievement::ResearchRouter));
        assert!(research_plan
            .achievements
            .contains(&Achievement::ResearchLogic));
        assert!(research_plan
            .achievements
            .contains(&Achievement::ResearchAll));

        let unlock_plan = state.unlock_plan(GameServiceUnlockSnapshot {
            content_name: Some("thorium".into()),
            research: GameServiceResearchSnapshot {
                router_unlocked: true,
                micro_processor_unlocked: false,
                all_researched: false,
            },
        });
        assert!(unlock_plan
            .achievements
            .contains(&Achievement::ObtainThorium));
        assert!(unlock_plan
            .achievements
            .contains(&Achievement::ResearchRouter));
        assert!(!unlock_plan
            .achievements
            .contains(&Achievement::ObtainTitanium));

        let titanium_plan = state.unlock_plan(GameServiceUnlockSnapshot {
            content_name: Some("titanium".into()),
            research: GameServiceResearchSnapshot::default(),
        });
        assert!(titanium_plan
            .achievements
            .contains(&Achievement::ObtainTitanium));
    }

    #[test]
    fn trigger_plan_maps_java_game_service_triggers() {
        let state = GameServiceState::default();

        assert!(state
            .trigger_plan(GameServiceTriggerSnapshot {
                trigger: Trigger::OpenConsole,
                campaign: false,
            })
            .achievements
            .contains(&Achievement::OpenConsole));
        assert!(state
            .trigger_plan(GameServiceTriggerSnapshot {
                trigger: Trigger::OpenWiki,
                campaign: false,
            })
            .achievements
            .contains(&Achievement::OpenWiki));
        assert!(state
            .trigger_plan(GameServiceTriggerSnapshot {
                trigger: Trigger::ImportMod,
                campaign: false,
            })
            .achievements
            .contains(&Achievement::InstallMod));
        assert!(state
            .trigger_plan(GameServiceTriggerSnapshot {
                trigger: Trigger::ExclusionDeath,
                campaign: false,
            })
            .achievements
            .contains(&Achievement::DieExclusion));
        assert!(state
            .trigger_plan(GameServiceTriggerSnapshot {
                trigger: Trigger::EnablePixelation,
                campaign: false,
            })
            .achievements
            .contains(&Achievement::EnablePixelation));

        let campaign_trigger_cases = [
            (Trigger::UnitCommandAttack, Achievement::IssueAttackCommand),
            (Trigger::UnitCommandBoost, Achievement::BoostUnit),
            (Trigger::ImpactPower, Achievement::PowerupImpactReactor),
            (Trigger::FlameAmmo, Achievement::UseFlameAmmo),
            (Trigger::TurretCool, Achievement::CoolTurret),
            (Trigger::SuicideBomb, Achievement::SuicideBomb),
            (Trigger::BlastGenerator, Achievement::BlastGenerator),
            (
                Trigger::ForceProjectorBreak,
                Achievement::BreakForceProjector,
            ),
            (Trigger::NeoplasmReact, Achievement::NeoplasmWater),
            (Trigger::ShockwaveTowerUse, Achievement::ShockwaveTowerUse),
            (Trigger::Shock, Achievement::ShockWetEnemy),
            (Trigger::BlastFreeze, Achievement::BlastFrozenUnit),
        ];

        for (trigger, achievement) in campaign_trigger_cases {
            assert!(state
                .trigger_plan(GameServiceTriggerSnapshot {
                    trigger,
                    campaign: true,
                })
                .achievements
                .contains(&achievement));
            assert!(state
                .trigger_plan(GameServiceTriggerSnapshot {
                    trigger,
                    campaign: false,
                })
                .is_empty());
        }

        let overheat_plan = state.trigger_plan(GameServiceTriggerSnapshot {
            trigger: Trigger::ThoriumReactorOverheat,
            campaign: true,
        });
        assert_eq!(
            overheat_plan.stat_additions,
            vec![SStat::ReactorsOverheated]
        );
    }

    #[test]
    fn wave_launch_pickup_drown_and_new_game_plans_match_java_events() {
        let mut state = GameServiceState::default();
        state.seed_java_t5_units();

        let wave_plan = state.wave_plan(GameServiceWaveSnapshot {
            campaign: true,
            wave: 10,
            buildings_built: 0,
        });
        assert_eq!(
            wave_plan.stat_max_updates,
            vec![(SStat::MaxWavesSurvived, 10)]
        );
        assert!(wave_plan
            .achievements
            .contains(&Achievement::Survive10WavesNoBlocks));

        assert!(state
            .wave_plan(GameServiceWaveSnapshot {
                campaign: false,
                wave: 100,
                buildings_built: 0,
            })
            .is_empty());

        let launch_plan = state.launch_item_plan(GameServiceLaunchItemSnapshot {
            campaign: true,
            amount: 250,
        });
        assert_eq!(
            launch_plan.stat_amount_additions,
            vec![(SStat::ItemsLaunched, 250)]
        );
        assert!(launch_plan
            .achievements
            .contains(&Achievement::LaunchItemPad));

        let pickup_plan = state.pickup_plan(GameServicePickupSnapshot {
            campaign: true,
            carrier_player: true,
            unit_name: Some("omura".into()),
        });
        assert!(pickup_plan.achievements.contains(&Achievement::PickupT5));

        assert!(state
            .unit_drown_plan(GameServiceUnitDrownSnapshot {
                campaign: true,
                player_unit: true,
            })
            .achievements
            .contains(&Achievement::Drown));

        assert!(state
            .new_game_plan(GameServiceNewGameSnapshot {
                campaign: true,
                core_items_total: 10_000,
            })
            .achievements
            .contains(&Achievement::Drop10kitems));
    }

    #[test]
    fn frame_update_and_ground_zero_nucleus_plans_match_java_branches() {
        let state = GameServiceState::default();

        let frame_plan = state.frame_update_plan(GameServiceFrameUpdateSnapshot {
            campaign: true,
            hover_unit_liquid_already_achieved: false,
            hover_check_due: true,
            elude_on_liquid: true,
            player_dead: false,
            player_unit_can_boost: true,
            player_unit_elevation: 0.25,
        });
        assert!(frame_plan
            .achievements
            .contains(&Achievement::HoverUnitLiquid));
        assert!(frame_plan.achievements.contains(&Achievement::BoostUnit));

        assert!(state
            .frame_update_plan(GameServiceFrameUpdateSnapshot::non_campaign())
            .is_empty());
        assert!(state
            .frame_update_plan(GameServiceFrameUpdateSnapshot {
                campaign: true,
                hover_unit_liquid_already_achieved: true,
                hover_check_due: true,
                elude_on_liquid: true,
                player_dead: true,
                player_unit_can_boost: true,
                player_unit_elevation: 1.0,
            })
            .is_empty());

        let nucleus_plan = state.nucleus_ground_zero_plan(GameServiceNucleusGroundZeroSnapshot {
            campaign: true,
            sector_preset_name: Some("groundZero".into()),
            block_name: "core-nucleus".into(),
        });
        assert!(nucleus_plan
            .achievements
            .contains(&Achievement::NucleusGroundZero));
    }

    #[test]
    fn combat_and_payload_event_plans_match_java_achievement_branches() {
        let state = GameServiceState::default();

        let scatter_plan =
            state.building_bullet_destroy_plan(GameServiceBuildingBulletDestroySnapshot {
                campaign: true,
                build_block_name: "scatter".into(),
                build_team_is_wave_team: true,
                bullet_owner_unit_name: Some("flare".into()),
                bullet_owner_team_is_player_team: true,
            });
        assert!(scatter_plan
            .achievements
            .contains(&Achievement::DestroyScatterFlare));

        assert!(state
            .building_bullet_destroy_plan(GameServiceBuildingBulletDestroySnapshot {
                campaign: true,
                build_block_name: "scatter".into(),
                build_team_is_wave_team: true,
                bullet_owner_unit_name: Some("dagger".into()),
                bullet_owner_team_is_player_team: true,
            })
            .is_empty());

        let neoplasia_plan =
            state.generator_pressure_explode_plan(GameServiceGeneratorPressureExplodeSnapshot {
                campaign: true,
                block_name: "neoplasia-reactor".into(),
            });
        assert!(neoplasia_plan
            .achievements
            .contains(&Achievement::NeoplasiaExplosion));

        let bullet_plan = state.unit_bullet_destroy_plan(GameServiceUnitBulletDestroySnapshot {
            campaign: true,
            bullet_team_is_player_team: true,
            bullet_owner_wall_build: true,
            killed_unit_name: "eclipse".into(),
            bullet_owner_turret_block_name: Some("duo".into()),
            bullet_type_mass_driver_bolt: true,
        });
        assert!(bullet_plan
            .achievements
            .contains(&Achievement::KillEnemyPhaseWall));
        assert!(bullet_plan
            .achievements
            .contains(&Achievement::KillEclipseDuo));
        assert!(bullet_plan
            .achievements
            .contains(&Achievement::KillMassDriver));

        assert!(state
            .unit_bullet_destroy_plan(GameServiceUnitBulletDestroySnapshot {
                campaign: true,
                bullet_team_is_player_team: false,
                bullet_owner_wall_build: true,
                killed_unit_name: "eclipse".into(),
                bullet_owner_turret_block_name: Some("duo".into()),
                bullet_type_mass_driver_bolt: true,
            })
            .is_empty());

        let payload_plan = state.payload_drop_plan(GameServicePayloadDropSnapshot {
            campaign: true,
            unit_present: true,
            carrier_team_is_default: true,
            within_enemy_core_radius: true,
        });
        assert!(payload_plan
            .achievements
            .contains(&Achievement::DropUnitsCoreZone));
    }
}
