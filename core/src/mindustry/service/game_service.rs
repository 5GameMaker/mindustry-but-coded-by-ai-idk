//! Default game-service shell mirroring upstream `mindustry.service.GameService`.
//!
//! The Java class is the central platform service hook for achievements,
//! service statistics and event registration. This module keeps the default
//! no-op platform behavior and the deterministic state containers; individual
//! event bindings can be ported incrementally on top.

use std::collections::BTreeSet;

use super::{Achievement, AchievementService, SStat, StatService};

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

    pub fn state(&self) -> &GameServiceState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut GameServiceState {
        &mut self.state
    }

    pub fn events_registered(&self) -> bool {
        self.events_registered
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
    use super::*;
    use crate::mindustry::service::{Achievement, AchievementContext, AchievementState, SStat};

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
}
