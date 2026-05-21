//! Default game-service shell mirroring upstream `mindustry.service.GameService`.
//!
//! The Java class is the central platform service hook for achievements,
//! service statistics and event registration. This module keeps the default
//! no-op platform behavior and the deterministic state containers; individual
//! event bindings can be ported incrementally on top.

use std::collections::BTreeSet;

use super::{AchievementService, StatService};

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
}
