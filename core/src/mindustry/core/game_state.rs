//! Lightweight runtime state shell mirroring upstream `mindustry.core.GameState`.
//!
//! The Java class is mostly a mutable bag of game-wide runtime state plus a
//! small state enum. This Rust port keeps the same default values and query
//! helpers while representing event dispatch as a returned `StateChangeEvent`
//! until the full Arc event bus is migrated.

use std::collections::BTreeMap;

use crate::mindustry::{
    game::{GameStats, MapMarkers, Rules, Teams},
    maps::MapDescriptor,
    net::{NetworkWorldData, StateSnapshotCallPacket},
    r#type::{MapLocales, Sector},
    world::blocks::Attributes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStateState {
    Paused,
    Playing,
    Menu,
}

impl GameStateState {
    pub const fn java_name(self) -> &'static str {
        match self {
            GameStateState::Paused => "paused",
            GameStateState::Playing => "playing",
            GameStateState::Menu => "menu",
        }
    }

    pub fn from_java_name(name: &str) -> Option<Self> {
        Some(match name {
            "paused" => GameStateState::Paused,
            "playing" => GameStateState::Playing,
            "menu" => GameStateState::Menu,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateChangeEvent {
    pub from: GameStateState,
    pub to: GameStateState,
}

impl StateChangeEvent {
    pub const fn new(from: GameStateState, to: GameStateState) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateSnapshotApplyResult {
    pub wave_changed: bool,
    pub state_change: Option<StateChangeEvent>,
}

impl StateSnapshotApplyResult {
    pub const fn new(wave_changed: bool, state_change: Option<StateChangeEvent>) -> Self {
        Self {
            wave_changed,
            state_change,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkWorldApplyResult {
    pub map_width: i32,
    pub map_height: i32,
    pub map_locales_loaded: bool,
    pub map_locales_error: Option<String>,
    pub content_patches_loaded: usize,
    pub tail_parse_error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataPatcherState {
    pub patches: Vec<String>,
    pub warnings: Vec<String>,
}

impl DataPatcherState {
    pub fn is_patched(&self) -> bool {
        !self.patches.is_empty()
    }

    pub fn clear(&mut self) {
        self.patches.clear();
        self.warnings.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    /// Current wave number, can be anything in non-wave modes.
    pub wave: i32,
    /// Wave countdown in ticks.
    pub wavetime: f32,
    /// Logic tick.
    pub tick: f64,
    /// Continuously ticks up every non-paused update.
    pub update_id: i64,
    /// Whether the game is in game over state.
    pub game_over: bool,
    /// Campaign-only "after game over" marker; such maps are kept paused by higher layers.
    pub after_game_over: bool,
    /// Whether the player's team won the match.
    pub won: bool,
    /// Server ticks/second. Only valid in multiplayer.
    pub server_tps: i32,
    /// Map that is currently being played on.
    pub map: MapDescriptor,
    /// The current game rules.
    pub rules: Rules,
    /// Statistics for this save/game. Displayed after game over.
    pub stats: GameStats,
    /// Markers not linked to objectives. Controlled by world processors.
    pub markers: MapMarkers,
    /// Locale-specific string bundles of current map.
    pub map_locales: MapLocales,
    /// Global attributes of the environment, calculated by weather.
    pub env_attrs: Attributes,
    /// Team data. Gets reset every new game.
    pub teams: Teams,
    /// Lightweight stand-in for upstream `DataPatcher` until the mod patcher is migrated.
    pub patcher: DataPatcherState,
    /// Number of enemies in the game; only used clientside in servers.
    pub enemies: i32,
    /// Map being playtested, not edited.
    pub playtesting_map: Option<MapDescriptor>,
    /// Current campaign sector mirror. Upstream stores this as `rules.sector`.
    pub sector: Option<Sector>,
    state: GameStateState,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let rules = Rules::default();
        let teams = Teams::new(
            rules.default_team as u8,
            rules.waves,
            rules.wave_team as u8,
            rules.default_team as u8,
        );

        Self {
            wave: 1,
            wavetime: 0.0,
            tick: 0.0,
            update_id: 0,
            game_over: false,
            after_game_over: false,
            won: false,
            server_tps: -1,
            map: empty_map_descriptor(),
            rules,
            stats: GameStats::default(),
            markers: MapMarkers::default(),
            map_locales: MapLocales::default(),
            env_attrs: Attributes::new(0),
            teams,
            patcher: DataPatcherState::default(),
            enemies: 0,
            playtesting_map: None,
            sector: None,
            state: GameStateState::Menu,
        }
    }

    pub fn boss(&self) -> Option<i32> {
        self.teams.bosses().first().copied()
    }

    fn sector_ref(&self) -> Option<&Sector> {
        self.rules.sector.as_ref().or(self.sector.as_ref())
    }

    pub fn set_sector(&mut self, sector: Option<Sector>) {
        self.rules.sector = sector.clone();
        self.sector = sector;
    }

    pub fn set(&mut self, state: GameStateState) -> Option<StateChangeEvent> {
        if self.state == state {
            return None;
        }

        let event = StateChangeEvent::new(self.state, state);
        self.state = state;
        Some(event)
    }

    pub fn has_spawns(&self) -> bool {
        if !self.rules.waves {
            return false;
        }

        let wave_team_has_core = self
            .teams
            .get_or_null(self.rules.wave_team as u8)
            .is_some_and(|team| !team.cores.is_empty());

        (self.rules.attack_mode && wave_team_has_core) || !self.rules.spawns.is_empty()
    }

    /// Note that being in a campaign does not necessarily mean having a sector.
    pub fn is_campaign(&self) -> bool {
        self.sector_ref().is_some()
    }

    pub fn has_sector(&self) -> bool {
        self.sector_ref().is_some()
    }

    pub fn get_sector(&self) -> Option<&Sector> {
        self.sector_ref()
    }

    pub fn get_planet_name(&self) -> Option<&str> {
        self.sector_ref()
            .and_then(|sector| sector.preset.as_ref())
            .and_then(|preset| preset.planet_name.as_deref())
            .or_else(|| (!self.rules.planet.is_empty()).then_some(self.rules.planet.as_str()))
    }

    pub fn is_editor(&self) -> bool {
        self.rules.editor
    }

    pub fn is_paused(&self) -> bool {
        self.state == GameStateState::Paused
    }

    /// Returns whether there is an unpaused game in progress.
    pub fn is_playing(&self) -> bool {
        self.state == GameStateState::Playing
    }

    /// Returns whether the current state is not the menu.
    pub fn is_game(&self) -> bool {
        self.state != GameStateState::Menu
    }

    pub fn is_menu(&self) -> bool {
        self.state == GameStateState::Menu
    }

    pub fn is(&self, state: GameStateState) -> bool {
        self.state == state
    }

    pub fn get_state(&self) -> GameStateState {
        self.state
    }

    pub fn sync_teams_with_rules(&mut self) {
        self.teams.set_rules(
            self.rules.waves,
            self.rules.wave_team as u8,
            self.rules.default_team as u8,
        );
    }

    pub fn apply_state_snapshot(
        &mut self,
        snapshot: &StateSnapshotCallPacket,
    ) -> StateSnapshotApplyResult {
        let wave_changed = self.wave != snapshot.wave;

        self.game_over = snapshot.game_over;
        self.wavetime = snapshot.wave_time;
        self.wave = snapshot.wave;
        self.enemies = snapshot.enemies;
        self.server_tps = snapshot.tps as i32;

        let state_change = if self.is_menu() {
            None
        } else {
            self.set(if snapshot.paused {
                GameStateState::Paused
            } else {
                GameStateState::Playing
            })
        };

        StateSnapshotApplyResult::new(wave_changed, state_change)
    }

    /// Applies the game-state front matter from Java `NetworkIO.loadWorld`.
    ///
    /// Full rules JSON and marker/entity materialization are still migrated in
    /// their dedicated modules; this stage wires the stable fields that already
    /// have Rust mirrors so a parsed world stream can drive map/wave/locales
    /// state instead of only being logged by `NetClient`.
    pub fn apply_network_world_data(
        &mut self,
        world_data: &NetworkWorldData,
    ) -> NetworkWorldApplyResult {
        self.wave = world_data.wave;
        self.wavetime = world_data.wave_time;
        self.tick = world_data.tick;

        let map_width = world_data
            .map_snapshot
            .as_ref()
            .map(|map| map.width as i32)
            .or_else(|| parse_tag_i32(&world_data.map_tags, "width"))
            .unwrap_or_default();
        let map_height = world_data
            .map_snapshot
            .as_ref()
            .map(|map| map.height as i32)
            .or_else(|| parse_tag_i32(&world_data.map_tags, "height"))
            .unwrap_or_default();
        let map_version = parse_tag_i32(&world_data.map_tags, "version").unwrap_or_default();
        let map_build = parse_tag_i32(&world_data.map_tags, "build").unwrap_or_default();
        self.map = MapDescriptor::new(
            "network.msav",
            map_width,
            map_height,
            world_data.map_tags.clone(),
            true,
            map_version,
            map_build,
        );

        let map_locales_error = match MapLocales::from_json_str(&world_data.map_locales_json) {
            Ok(locales) => {
                self.map_locales = locales;
                None
            }
            Err(error) => Some(error),
        };

        self.patcher.patches = world_data
            .content_patches_snapshot
            .as_ref()
            .map(|patches| {
                patches
                    .patches
                    .iter()
                    .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                    .collect()
            })
            .unwrap_or_default();
        self.patcher.warnings.clear();

        NetworkWorldApplyResult {
            map_width,
            map_height,
            map_locales_loaded: map_locales_error.is_none(),
            map_locales_error,
            content_patches_loaded: self.patcher.patches.len(),
            tail_parse_error: world_data.tail_parse_error.clone(),
        }
    }
}

pub fn empty_map_descriptor() -> MapDescriptor {
    let mut tags = BTreeMap::new();
    tags.insert("name".to_string(), "empty".to_string());
    MapDescriptor::new("empty.msav", 0, 0, tags, false, 0, 0)
}

fn parse_tag_i32(tags: &BTreeMap<String, String>, key: &str) -> Option<i32> {
    tags.get(key).and_then(|value| value.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        game::{CoreInfo, SpawnGroup, TEAM_CRUX},
        io::{ContentPatchSet, LegacyMapBlockRecord, LegacyMapFloorRecord, LegacyShortChunkMap},
        net::{NetworkWorldData, StateSnapshotCallPacket},
        r#type::SectorPreset,
    };

    #[test]
    fn game_state_defaults_match_java_field_initializers() {
        let state = GameState::new();

        assert_eq!(state.wave, 1);
        assert_eq!(state.wavetime, 0.0);
        assert_eq!(state.tick, 0.0);
        assert_eq!(state.update_id, 0);
        assert!(!state.game_over);
        assert!(!state.after_game_over);
        assert!(!state.won);
        assert_eq!(state.server_tps, -1);
        assert_eq!(state.get_state(), GameStateState::Menu);
        assert!(state.is_menu());
        assert!(!state.is_game());
        assert_eq!(state.map.name(), "empty");
        assert_eq!(state.rules.planet, "serpulo");
        assert_eq!(state.get_planet_name(), Some("serpulo"));
        assert!(state.stats.placed_block_count.is_empty());
        assert!(state.markers.is_empty());
        assert!(state.map_locales.locales.is_empty());
        assert!(state.env_attrs.values().is_empty());
        assert!(state.patcher.patches.is_empty());
        assert_eq!(state.enemies, 0);
        assert!(state.playtesting_map.is_none());
        assert!(state.sector.is_none());
    }

    #[test]
    fn apply_network_world_data_updates_java_loadworld_front_matter() {
        let mut state = GameState::new();
        let mut map_tags = BTreeMap::new();
        map_tags.insert("name".into(), "Network Map".into());
        map_tags.insert("build".into(), "157".into());
        map_tags.insert("version".into(), "11".into());

        let world = NetworkWorldData {
            rules_json: "{}".into(),
            map_locales_json: r#"{"en":{"name":"Network Map"}}"#.into(),
            map_tags,
            wave: 12,
            wave_time: 30.5,
            tick: 99.25,
            content_patches_snapshot: Some(ContentPatchSet {
                patches: vec![b"one".to_vec(), b"two".to_vec()],
            }),
            map_snapshot: Some(LegacyShortChunkMap {
                width: 3,
                height: 2,
                floors: vec![LegacyMapFloorRecord {
                    index: 0,
                    floor_id: 1,
                    ore_id: 0,
                    consecutives: 5,
                }],
                blocks: vec![LegacyMapBlockRecord {
                    index: 0,
                    block_id: 0,
                    packed_flags: 0,
                    has_entity: false,
                    has_old_data: false,
                    has_new_data: false,
                    is_center: true,
                    new_data: None,
                    old_data: None,
                    building: None,
                    consecutives: 5,
                }],
            }),
            ..NetworkWorldData::default()
        };

        let result = state.apply_network_world_data(&world);

        assert_eq!(state.wave, 12);
        assert_eq!(state.wavetime, 30.5);
        assert_eq!(state.tick, 99.25);
        assert_eq!(state.map.name(), "Network Map");
        assert_eq!(state.map.width, 3);
        assert_eq!(state.map.height, 2);
        assert_eq!(state.map.version, 11);
        assert_eq!(state.map.build, 157);
        assert_eq!(state.map_locales.locales["en"]["name"], "Network Map");
        assert_eq!(state.patcher.patches, vec!["one", "two"]);
        assert_eq!(
            result,
            NetworkWorldApplyResult {
                map_width: 3,
                map_height: 2,
                map_locales_loaded: true,
                map_locales_error: None,
                content_patches_loaded: 2,
                tail_parse_error: None,
            }
        );
    }

    #[test]
    fn set_returns_state_change_event_only_when_state_changes() {
        let mut state = GameState::new();

        assert_eq!(state.set(GameStateState::Menu), None);
        assert_eq!(
            state.set(GameStateState::Playing),
            Some(StateChangeEvent::new(
                GameStateState::Menu,
                GameStateState::Playing
            ))
        );
        assert!(state.is_playing());
        assert!(state.is_game());
        assert!(!state.is_menu());

        assert_eq!(
            state.set(GameStateState::Paused),
            Some(StateChangeEvent::new(
                GameStateState::Playing,
                GameStateState::Paused
            ))
        );
        assert!(state.is_paused());
    }

    #[test]
    fn has_spawns_matches_java_wave_attack_and_spawn_rules() {
        let mut state = GameState::new();
        assert!(!state.has_spawns());

        state.rules.waves = true;
        state.rules.spawns.push(SpawnGroup::new("dagger"));
        assert!(state.has_spawns());

        state.rules.spawns.clear();
        assert!(!state.has_spawns());

        state.rules.attack_mode = true;
        state
            .teams
            .register_core(CoreInfo::new(7, TEAM_CRUX, 16.0, 24.0));
        assert!(state.has_spawns());
    }

    #[test]
    fn sector_and_planet_helpers_follow_java_null_fallback_shape() {
        let mut state = GameState::new();
        assert!(!state.is_campaign());
        assert!(!state.has_sector());
        assert_eq!(state.get_sector(), None);
        assert_eq!(state.get_planet_name(), Some("serpulo"));

        let mut sector = Sector::new(5);
        sector.preset = Some(SectorPreset::with_planet_sector("groundZero", "erekir", 5));
        state.set_sector(Some(sector));

        assert!(state.is_campaign());
        assert!(state.has_sector());
        assert_eq!(state.get_sector().map(|sector| sector.id), Some(5));
        assert_eq!(state.get_planet_name(), Some("erekir"));
    }

    #[test]
    fn rules_sector_is_the_primary_campaign_source_like_java() {
        let mut state = GameState::new();

        let mut mirrored = Sector::new(1);
        mirrored.preset = Some(SectorPreset::with_planet_sector("old", "serpulo", 1));
        state.sector = Some(mirrored);

        let mut rules_sector = Sector::new(2);
        rules_sector.preset = Some(SectorPreset::with_planet_sector("new", "erekir", 2));
        state.rules.sector = Some(rules_sector);

        assert!(state.is_campaign());
        assert_eq!(state.get_sector().map(|sector| sector.id), Some(2));
        assert_eq!(state.get_planet_name(), Some("erekir"));

        state.set_sector(None);
        assert!(!state.is_campaign());
        assert!(state.rules.sector.is_none());
        assert!(state.sector.is_none());
    }

    #[test]
    fn state_enum_keeps_upstream_java_names() {
        assert_eq!(GameStateState::Paused.java_name(), "paused");
        assert_eq!(GameStateState::Playing.java_name(), "playing");
        assert_eq!(GameStateState::Menu.java_name(), "menu");
        assert_eq!(
            GameStateState::from_java_name("playing"),
            Some(GameStateState::Playing)
        );
        assert_eq!(GameStateState::from_java_name("missing"), None);
    }

    #[test]
    fn apply_state_snapshot_updates_scalar_client_runtime_state() {
        let mut state = GameState::new();
        state.set(GameStateState::Playing);

        let snapshot = StateSnapshotCallPacket {
            wave_time: 12.5,
            wave: 9,
            enemies: 17,
            paused: true,
            game_over: true,
            time_data: 456,
            tps: 255,
            rand0: 11,
            rand1: 22,
            core_data: vec![1, 2, 3],
        };

        let result = state.apply_state_snapshot(&snapshot);

        assert_eq!(
            result,
            StateSnapshotApplyResult::new(
                true,
                Some(StateChangeEvent::new(
                    GameStateState::Playing,
                    GameStateState::Paused
                ))
            )
        );
        assert_eq!(state.wavetime, snapshot.wave_time);
        assert_eq!(state.wave, snapshot.wave);
        assert_eq!(state.enemies, snapshot.enemies);
        assert_eq!(state.game_over, snapshot.game_over);
        assert_eq!(state.server_tps, 255);
        assert!(state.is_paused());

        let next = StateSnapshotCallPacket {
            wave_time: 1.0,
            wave: 9,
            enemies: 0,
            paused: false,
            game_over: false,
            time_data: 789,
            tps: 60,
            rand0: 33,
            rand1: 44,
            core_data: Vec::new(),
        };

        let result = state.apply_state_snapshot(&next);

        assert_eq!(
            result,
            StateSnapshotApplyResult::new(
                false,
                Some(StateChangeEvent::new(
                    GameStateState::Paused,
                    GameStateState::Playing
                ))
            )
        );
        assert_eq!(state.wavetime, next.wave_time);
        assert_eq!(state.wave, next.wave);
        assert_eq!(state.enemies, next.enemies);
        assert_eq!(state.server_tps, 60);
        assert!(state.is_playing());
    }

    #[test]
    fn apply_state_snapshot_keeps_menu_state_like_java_guard() {
        let mut state = GameState::new();
        assert!(state.is_menu());

        let snapshot = StateSnapshotCallPacket {
            wave_time: 3.5,
            wave: 4,
            enemies: 5,
            paused: true,
            game_over: false,
            time_data: 123,
            tps: 30,
            rand0: 0,
            rand1: 0,
            core_data: Vec::new(),
        };

        let result = state.apply_state_snapshot(&snapshot);

        assert_eq!(result, StateSnapshotApplyResult::new(true, None));
        assert!(state.is_menu());
        assert_eq!(state.wavetime, snapshot.wave_time);
        assert_eq!(state.wave, snapshot.wave);
        assert_eq!(state.enemies, snapshot.enemies);
        assert_eq!(state.server_tps, 30);
    }
}
