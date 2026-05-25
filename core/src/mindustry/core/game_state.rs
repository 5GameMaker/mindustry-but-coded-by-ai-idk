//! Lightweight runtime state shell mirroring upstream `mindustry.core.GameState`.
//!
//! The Java class is mostly a mutable bag of game-wide runtime state plus a
//! small state enum. This Rust port keeps the same default values and query
//! helpers while representing event dispatch as a returned `StateChangeEvent`
//! until the full Arc event bus is migrated.

use std::{
    collections::BTreeMap,
    io::{self, Cursor},
};

use crate::mindustry::{
    core::World,
    ctype::{ContentId, ContentType},
    game::{BlockPlan, FogControl, GameStats, MapMarkers, Rules, Teams, Universe},
    io::{
        read_custom_chunks, type_io::read_u8, CustomChunkSet, LegacyTeamBlocks,
        MarkerRegionSummary, TypeValue, CUSTOM_CHUNK_STATIC_FOG_DATA,
    },
    maps::MapDescriptor,
    net::{NetworkWorldData, StateSnapshotCallPacket},
    r#type::{MapLocales, Sector},
    world::{
        blocks::Attributes, meta::build_visibility::BuildVisibilityContext, modules::ItemModule,
    },
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameUpdateFrameAdvance {
    pub advanced: bool,
    pub delta_ticks: f64,
    pub tick: f64,
    pub update_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkWorldApplyResult {
    pub rules_loaded: bool,
    pub rules_error: Option<String>,
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
    /// Mirror of upstream `GlobalVars.rand.seed0` loaded from network world data.
    pub rand_seed0: i64,
    /// Mirror of upstream `GlobalVars.rand.seed1` loaded from network world data.
    pub rand_seed1: i64,
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
    /// Runtime world tiles/load lifecycle.
    pub world: World,
    /// The current game rules.
    pub rules: Rules,
    /// Statistics for this save/game. Displayed after game over.
    pub stats: GameStats,
    /// Markers not linked to objectives. Controlled by world processors.
    pub markers: MapMarkers,
    /// Summary of marker region bytes loaded through `NetworkIO.loadWorld`.
    pub marker_summary: Option<MarkerRegionSummary>,
    /// Locale-specific string bundles of current map.
    pub map_locales: MapLocales,
    /// Custom save chunks loaded after marker data.
    pub custom_chunks: CustomChunkSet,
    /// Last custom chunk decode error, if the network tail could not be materialized.
    pub custom_chunks_error: Option<String>,
    /// Runtime static/dynamic fog controller restored from Java `static-fog-data`.
    pub fog_control: FogControl,
    /// Last static fog custom chunk decode error, if any.
    pub static_fog_error: Option<String>,
    /// Global attributes of the environment, calculated by weather.
    pub env_attrs: Attributes,
    /// Team data. Gets reset every new game.
    pub teams: Teams,
    /// Campaign/universe time mirror; multiplayer snapshots update `netSeconds`.
    pub universe: Universe,
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
            rand_seed0: 0,
            rand_seed1: 0,
            update_id: 0,
            game_over: false,
            after_game_over: false,
            won: false,
            server_tps: -1,
            map: empty_map_descriptor(),
            world: World::new(),
            rules,
            stats: GameStats::default(),
            markers: MapMarkers::default(),
            marker_summary: None,
            map_locales: MapLocales::default(),
            custom_chunks: CustomChunkSet::default(),
            custom_chunks_error: None,
            fog_control: FogControl::new(0, 0),
            static_fog_error: None,
            env_attrs: Attributes::new(0),
            teams,
            universe: Universe::new(Vec::<String>::new()),
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

    pub fn build_visibility_context(&self) -> BuildVisibilityContext {
        BuildVisibilityContext {
            has_state: true,
            game: self.is_game(),
            editor: self.is_editor(),
            campaign: self.is_campaign(),
            infinite_resources: self.rules.infinite_resources,
            core_zone_present: false,
            allow_edit_world_processors: self.rules.allow_edit_world_processors,
            legacy_launch_pads: false,
            advanced_launch_pad_present: false,
            advanced_launch_pad_unlocked: false,
            lighting: self.rules.lighting,
            unit_ammo: self.rules.unit_ammo,
            fog: self.rules.fog,
        }
    }

    pub fn advance_game_update_frame(&mut self, delta_seconds: f32) -> GameUpdateFrameAdvance {
        let delta_ticks = if delta_seconds.is_finite() {
            delta_seconds as f64 * 60.0
        } else {
            0.0
        };

        if self.is_game() && !self.is_paused() {
            self.tick += delta_ticks;
            self.update_id += 1;
            GameUpdateFrameAdvance {
                advanced: true,
                delta_ticks,
                tick: self.tick,
                update_id: self.update_id,
            }
        } else {
            GameUpdateFrameAdvance {
                advanced: false,
                delta_ticks: 0.0,
                tick: self.tick,
                update_id: self.update_id,
            }
        }
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
        self.rand_seed0 = snapshot.rand0;
        self.rand_seed1 = snapshot.rand1;
        self.universe.update_net_seconds(snapshot.time_data);
        self.apply_state_snapshot_core_data(&snapshot.core_data);

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

    fn apply_state_snapshot_core_data(&mut self, core_data: &[u8]) {
        if let Ok(items_by_team) = decode_state_snapshot_core_items(core_data) {
            for (team, items) in items_by_team {
                self.teams.replace_core_items(team, items);
            }
        }
    }

    /// Applies Java `SaveVersion.readTeamBlocks(...)` output to runtime team plans.
    ///
    /// The core layer owns the stable team/plan mutation; callers provide the
    /// content-id mapping because network content headers can remap ids at the
    /// client boundary.
    pub fn apply_legacy_team_blocks<BlockName, ContentName>(
        &mut self,
        team_blocks: Option<&LegacyTeamBlocks>,
        mut block_name: BlockName,
        mut content_name: ContentName,
    ) where
        BlockName: FnMut(ContentId) -> Option<String>,
        ContentName: FnMut(ContentType, ContentId) -> Option<String>,
    {
        let Some(team_blocks) = team_blocks else {
            self.teams.replace_plans(Vec::new());
            return;
        };

        let plans_by_team = team_blocks
            .groups
            .iter()
            .filter_map(|group| {
                let team = u8::try_from(group.team_id).ok()?;
                let plans = group
                    .plans
                    .iter()
                    .filter_map(|plan| {
                        legacy_team_block_plan(plan, &mut block_name, &mut content_name)
                    })
                    .collect::<Vec<_>>();
                Some((team, plans))
            })
            .collect::<Vec<_>>();

        self.teams.replace_plans(plans_by_team);
    }

    /// Exports runtime team plans in the same logical shape consumed by Java
    /// `SaveVersion.writeTeamBlocks(...)`.
    ///
    /// The caller supplies the block-name-to-id mapping so this remains usable
    /// both for base content and for temporary network content mappers.
    pub fn export_legacy_team_blocks<BlockId>(
        &mut self,
        block_id: BlockId,
        include_sharded: bool,
    ) -> LegacyTeamBlocks
    where
        BlockId: FnMut(&str) -> Option<ContentId>,
    {
        self.teams.to_legacy_team_blocks(block_id, include_sharded)
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
        let rules_error = self.rules.apply_json_str(&world_data.rules_json).err();
        if rules_error.is_none() {
            self.sync_teams_with_rules();
        }

        self.wave = world_data.wave;
        self.wavetime = world_data.wave_time;
        self.tick = world_data.tick;
        self.rand_seed0 = world_data.rand_seed0;
        self.rand_seed1 = world_data.rand_seed1;

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

        if let Some(map_snapshot) = world_data.map_snapshot.as_ref() {
            self.world.load_network_map(map_snapshot);
        }
        self.fog_control
            .reset_world(map_width.max(0) as usize, map_height.max(0) as usize);

        let content_header = world_data.content_header_snapshot.as_ref();
        self.apply_legacy_team_blocks(
            world_data.team_blocks_snapshot.as_ref(),
            |block_id| content_header_name(content_header, ContentType::Block, block_id),
            |content_type, content_id| {
                content_header_name(content_header, content_type, content_id)
            },
        );

        self.markers = world_data.markers_snapshot.clone().unwrap_or_default();
        self.marker_summary = world_data.marker_summary.clone();
        self.custom_chunks = CustomChunkSet::default();
        self.custom_chunks_error = None;
        self.static_fog_error = None;
        if let Some(custom_chunks) = world_data.custom_chunks_snapshot.as_ref() {
            self.custom_chunks = custom_chunks.clone();
        } else if !world_data.custom_chunks.is_empty() {
            let mut custom_chunks = world_data.custom_chunks.as_slice();
            match read_custom_chunks(&mut custom_chunks) {
                Ok(chunks) => {
                    self.custom_chunks = chunks;
                    if !custom_chunks.is_empty() {
                        self.custom_chunks_error =
                            Some("trailing bytes after custom chunks".into());
                    }
                }
                Err(error) => {
                    self.custom_chunks_error = Some(error.to_string());
                }
            }
        }
        if let Some(static_fog_bytes) = self
            .custom_chunks
            .get(CUSTOM_CHUNK_STATIC_FOG_DATA)
            .map(Vec::from)
        {
            if let Err(error) = self.fog_control.read_static_fog_bytes(&static_fog_bytes) {
                self.static_fog_error = Some(error);
            }
        }

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
            rules_loaded: rules_error.is_none(),
            rules_error,
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

fn decode_state_snapshot_core_items(core_data: &[u8]) -> io::Result<Vec<(u8, BTreeMap<i16, i32>)>> {
    if core_data.is_empty() {
        return Ok(Vec::new());
    }

    let mut cursor = Cursor::new(core_data);
    let teams = read_u8(&mut cursor)? as usize;
    let mut items_by_team = Vec::with_capacity(teams);

    for _ in 0..teams {
        let team = read_u8(&mut cursor)?;
        let mut items = ItemModule::default();
        items.read(&mut cursor, false)?;
        items_by_team.push((team, items.each().collect()));
    }

    Ok(items_by_team)
}

fn parse_tag_i32(tags: &BTreeMap<String, String>, key: &str) -> Option<i32> {
    tags.get(key).and_then(|value| value.parse().ok())
}

fn content_header_name(
    snapshot: Option<&crate::mindustry::io::ContentHeaderSnapshot>,
    content_type: ContentType,
    id: ContentId,
) -> Option<String> {
    let snapshot = snapshot?;
    let id = usize::try_from(id).ok()?;
    snapshot
        .entries
        .iter()
        .find(|entry| entry.content_type == content_type.ordinal())
        .and_then(|entry| entry.names.get(id))
        .cloned()
}

fn legacy_team_block_plan<BlockName, ContentName>(
    plan: &crate::mindustry::io::LegacyTeamBlockPlan,
    block_name: &mut BlockName,
    content_name: &mut ContentName,
) -> Option<BlockPlan>
where
    BlockName: FnMut(ContentId) -> Option<String>,
    ContentName: FnMut(ContentType, ContentId) -> Option<String>,
{
    let block = block_name(plan.block_id)?;
    Some(BlockPlan::with_config_value(
        plan.x as i32,
        plan.y as i32,
        plan.rotation,
        block,
        legacy_team_block_config(&plan.config, content_name),
        plan.config.clone(),
    ))
}

fn legacy_team_block_config<ContentName>(
    config: &TypeValue,
    content_name: &mut ContentName,
) -> Option<String>
where
    ContentName: FnMut(ContentType, ContentId) -> Option<String>,
{
    match config {
        TypeValue::Null => None,
        TypeValue::Int(value) => Some(value.to_string()),
        TypeValue::Long(value) => Some(value.to_string()),
        TypeValue::Float(value) => Some(value.to_string()),
        TypeValue::String(value) => Some(value.clone()),
        TypeValue::Content(value) | TypeValue::TechNode(value) => {
            content_name(value.content_type, value.id).or_else(|| Some(format!("{value:?}")))
        }
        TypeValue::Bool(value) => Some(value.to_string()),
        TypeValue::Double(value) => Some(value.to_string()),
        TypeValue::Building(value) => Some(value.to_string()),
        TypeValue::LogicAccess(value) => Some(format!("{value:?}")),
        TypeValue::Unit(value) => Some(value.to_string()),
        TypeValue::Point2(value) => Some(format!("{},{}", value.x, value.y)),
        TypeValue::Vec2(value) => Some(format!("{},{}", value.x, value.y)),
        TypeValue::Team(value) => Some(value.to_string()),
        TypeValue::UnitCommand(value) => {
            content_name(ContentType::UnitCommand, *value).or_else(|| Some(value.to_string()))
        }
        TypeValue::IntSeq(values) | TypeValue::IntArray(values) => Some(format!("{values:?}")),
        TypeValue::ByteArray(values) => Some(format!("{values:?}")),
        TypeValue::Point2Array(values) => Some(format!("{values:?}")),
        TypeValue::BoolArray(values) => Some(format!("{values:?}")),
        TypeValue::Vec2Array(values) => Some(format!("{values:?}")),
        TypeValue::ObjectArray(values) => Some(format!("{values:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        game::{
            CoreInfo, MapMarkers, ObjectiveMarker, SpawnGroup, TEAM_CRUX, TEAM_MALIS, TEAM_SHARDED,
        },
        io::{
            ContentHeaderEntry, ContentHeaderSnapshot, ContentPatchSet, ContentRef,
            LegacyMapBlockRecord, LegacyMapFloorRecord, LegacyShortChunkMap, LegacyTeamBlockGroup,
            LegacyTeamBlockPlan, LegacyTeamBlocks,
        },
        net::{NetworkWorldData, StateSnapshotCallPacket},
        r#type::SectorPreset,
    };

    #[test]
    fn game_state_defaults_match_java_field_initializers() {
        let state = GameState::new();

        assert_eq!(state.wave, 1);
        assert_eq!(state.wavetime, 0.0);
        assert_eq!(state.tick, 0.0);
        assert_eq!(state.rand_seed0, 0);
        assert_eq!(state.rand_seed1, 0);
        assert_eq!(state.update_id, 0);
        assert!(!state.game_over);
        assert!(!state.after_game_over);
        assert!(!state.won);
        assert_eq!(state.server_tps, -1);
        assert_eq!(state.get_state(), GameStateState::Menu);
        assert!(state.is_menu());
        assert!(!state.is_game());
        assert!(!state.build_visibility_context().game);
        assert_eq!(state.map.name(), "empty");
        assert_eq!(state.world.width(), 0);
        assert_eq!(state.world.height(), 0);
        assert!(state.world.load_events().is_empty());
        assert_eq!(state.rules.planet, "serpulo");
        assert_eq!(state.get_planet_name(), Some("serpulo"));
        assert!(state.stats.placed_block_count.is_empty());
        assert!(state.markers.is_empty());
        assert_eq!(state.marker_summary, None);
        assert!(state.map_locales.locales.is_empty());
        assert!(state.custom_chunks.chunks.is_empty());
        assert_eq!(state.custom_chunks_error, None);
        assert!(state.env_attrs.values().is_empty());
        assert!(state.patcher.patches.is_empty());
        assert_eq!(state.enemies, 0);
        assert!(state.playtesting_map.is_none());
        assert!(state.sector.is_none());
    }

    #[test]
    fn build_visibility_context_reflects_state_and_rules() {
        let mut state = GameState::new();
        let menu = state.build_visibility_context();
        assert!(menu.has_state);
        assert!(!menu.game);
        assert!(!menu.editor);
        assert!(!menu.campaign);
        assert!(!menu.infinite_resources);

        state.set(GameStateState::Playing);
        state.rules.editor = true;
        state.rules.infinite_resources = true;
        state.rules.allow_edit_world_processors = true;
        state.rules.lighting = true;
        state.rules.unit_ammo = true;
        state.rules.fog = true;
        state.set_sector(Some(Sector::new(7)));

        let context = state.build_visibility_context();
        assert!(context.game);
        assert!(context.editor);
        assert!(context.campaign);
        assert!(context.infinite_resources);
        assert!(context.allow_edit_world_processors);
        assert!(context.lighting);
        assert!(context.unit_ammo);
        assert!(context.fog);
        assert!(!context.core_zone_present);
        assert!(!context.legacy_launch_pads);
        assert!(!context.advanced_launch_pad_present);
        assert!(!context.advanced_launch_pad_unlocked);
    }

    #[test]
    fn apply_network_world_data_updates_java_loadworld_front_matter() {
        let mut state = GameState::new();
        let mut map_tags = BTreeMap::new();
        map_tags.insert("name".into(), "Network Map".into());
        map_tags.insert("build".into(), "157".into());
        map_tags.insert("version".into(), "11".into());
        let mut marker_counts = BTreeMap::new();
        marker_counts.insert("Minimap".into(), 2);
        let marker_summary = MarkerRegionSummary {
            total: 3,
            recognized_by_type: marker_counts,
            unrecognized_type_count: 1,
            missing_class_count: 0,
        };
        let mut markers_snapshot = MapMarkers::new();
        markers_snapshot.add(
            7,
            ObjectiveMarker::default_for_java_name("Minimap")
                .expect("Minimap should map to point marker"),
        );
        let mut custom_chunk_set = CustomChunkSet::default();
        custom_chunk_set.insert_or_replace("static-fog", vec![1, 2, 3]);
        let mut custom_chunks = Vec::new();
        crate::mindustry::io::write_custom_chunks(&mut custom_chunks, &custom_chunk_set).unwrap();

        let world = NetworkWorldData {
            rules_json: "{}".into(),
            map_locales_json: r#"{"en":{"name":"Network Map"}}"#.into(),
            map_tags,
            wave: 12,
            wave_time: 30.5,
            tick: 99.25,
            rand_seed0: 123,
            rand_seed1: 456,
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
            markers_snapshot: Some(markers_snapshot.clone()),
            marker_summary: Some(marker_summary.clone()),
            custom_chunks,
            ..NetworkWorldData::default()
        };

        let result = state.apply_network_world_data(&world);

        assert_eq!(state.wave, 12);
        assert_eq!(state.wavetime, 30.5);
        assert_eq!(state.tick, 99.25);
        assert_eq!(state.rand_seed0, 123);
        assert_eq!(state.rand_seed1, 456);
        assert_eq!(state.map.name(), "Network Map");
        assert_eq!(state.map.width, 3);
        assert_eq!(state.map.height, 2);
        assert_eq!(state.map.version, 11);
        assert_eq!(state.map.build, 157);
        assert_eq!(state.world.width(), 3);
        assert_eq!(state.world.height(), 2);
        assert_eq!(state.markers, markers_snapshot);
        assert_eq!(state.marker_summary, Some(marker_summary));
        assert_eq!(
            state.custom_chunks.get("static-fog"),
            Some([1, 2, 3].as_slice())
        );
        assert_eq!(state.custom_chunks_error, None);
        assert_eq!(
            state.world.load_events(),
            &[
                crate::mindustry::core::WorldLoadEventKind::Begin,
                crate::mindustry::core::WorldLoadEventKind::End,
                crate::mindustry::core::WorldLoadEventKind::Loaded,
            ]
        );
        assert_eq!(state.map_locales.locales["en"]["name"], "Network Map");
        assert_eq!(state.patcher.patches, vec!["one", "two"]);
        assert_eq!(
            result,
            NetworkWorldApplyResult {
                rules_loaded: true,
                rules_error: None,
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
    fn apply_network_world_data_materializes_team_block_plans_from_content_header() {
        let mut state = GameState::new();
        state
            .teams
            .replace_plans([(TEAM_SHARDED, vec![BlockPlan::new(1, 1, 0, "old", None)])]);

        let world = NetworkWorldData {
            content_header_snapshot: Some(ContentHeaderSnapshot {
                entries: vec![
                    ContentHeaderEntry {
                        content_type: ContentType::Block.ordinal(),
                        names: vec!["air".into(), "router".into(), "junction".into()],
                    },
                    ContentHeaderEntry {
                        content_type: ContentType::Item.ordinal(),
                        names: vec!["copper".into(), "lead".into()],
                    },
                ],
            }),
            team_blocks_snapshot: Some(LegacyTeamBlocks {
                groups: vec![LegacyTeamBlockGroup {
                    team_id: TEAM_SHARDED as i32,
                    plans: vec![
                        LegacyTeamBlockPlan {
                            x: 5,
                            y: 6,
                            rotation: 1,
                            block_id: 1,
                            config: TypeValue::String("cfg".into()),
                        },
                        LegacyTeamBlockPlan {
                            x: 7,
                            y: 8,
                            rotation: 2,
                            block_id: 2,
                            config: TypeValue::Content(ContentRef::new(ContentType::Item, 1)),
                        },
                    ],
                }],
            }),
            ..NetworkWorldData::default()
        };

        state.apply_network_world_data(&world);

        assert_eq!(
            state.teams.get_or_null(TEAM_SHARDED).unwrap().plans,
            vec![
                BlockPlan::new(5, 6, 1, "router", Some("cfg".into())),
                BlockPlan::with_config_value(
                    7,
                    8,
                    2,
                    "junction",
                    Some("lead".into()),
                    TypeValue::Content(ContentRef::new(ContentType::Item, 1)),
                ),
            ]
        );
    }

    #[test]
    fn apply_network_world_data_prefers_custom_chunk_snapshot_over_raw_bytes() {
        let mut state = GameState::new();
        let mut raw_chunks = CustomChunkSet::default();
        raw_chunks.insert_or_replace(CUSTOM_CHUNK_STATIC_FOG_DATA, vec![9]);
        let mut raw_bytes = Vec::new();
        crate::mindustry::io::write_custom_chunks(&mut raw_bytes, &raw_chunks).unwrap();

        let mut snapshot_chunks = CustomChunkSet::default();
        snapshot_chunks.insert_or_replace(CUSTOM_CHUNK_STATIC_FOG_DATA, vec![4, 5, 6]);

        state.apply_network_world_data(&NetworkWorldData {
            custom_chunks: raw_bytes,
            custom_chunks_snapshot: Some(snapshot_chunks.clone()),
            ..NetworkWorldData::default()
        });

        assert_eq!(state.custom_chunks, snapshot_chunks);
        assert_eq!(
            state.custom_chunks.get(CUSTOM_CHUNK_STATIC_FOG_DATA),
            Some([4, 5, 6].as_slice())
        );
        assert_eq!(state.custom_chunks_error, None);
    }

    #[test]
    fn apply_network_world_data_restores_static_fog_custom_chunk() {
        let mut source_fog = FogControl::new(4, 1);
        {
            let data = source_fog.ensure_data(3);
            data.static_data.set(0);
            data.static_data.set(1);
        }
        let mut chunks = CustomChunkSet::default();
        chunks.insert_or_replace(
            CUSTOM_CHUNK_STATIC_FOG_DATA,
            source_fog.write_static_fog_bytes(),
        );

        let mut state = GameState::new();
        state.apply_network_world_data(&NetworkWorldData {
            custom_chunks_snapshot: Some(chunks),
            ..NetworkWorldData::default()
        });

        assert_eq!(state.static_fog_error, None);
        assert_eq!(state.fog_control.width(), 4);
        assert_eq!(state.fog_control.height(), 1);
        assert!(state
            .fog_control
            .is_discovered(true, true, Some(3), false, 0, 0));
        assert!(state
            .fog_control
            .is_discovered(true, true, Some(3), false, 1, 0));
        assert!(!state
            .fog_control
            .is_discovered(true, true, Some(3), false, 2, 0));
    }

    #[test]
    fn apply_legacy_team_blocks_materializes_runtime_team_plans() {
        use crate::mindustry::io::{
            ContentRef, LegacyTeamBlockGroup, LegacyTeamBlockPlan, LegacyTeamBlocks,
        };

        let mut state = GameState::new();
        state.apply_legacy_team_blocks(
            Some(&LegacyTeamBlocks {
                groups: vec![LegacyTeamBlockGroup {
                    team_id: 7,
                    plans: vec![
                        LegacyTeamBlockPlan {
                            x: 5,
                            y: 6,
                            rotation: 1,
                            block_id: 2,
                            config: TypeValue::String("cfg".into()),
                        },
                        LegacyTeamBlockPlan {
                            x: 8,
                            y: 9,
                            rotation: 2,
                            block_id: 3,
                            config: TypeValue::Content(ContentRef::new(ContentType::Item, 4)),
                        },
                    ],
                }],
            }),
            |block_id| match block_id {
                2 => Some("router".into()),
                3 => Some("junction".into()),
                _ => None,
            },
            |content_type, content_id| match (content_type, content_id) {
                (ContentType::Item, 4) => Some("copper".into()),
                _ => None,
            },
        );

        assert_eq!(
            state.teams.get_or_null(7).unwrap().plans,
            vec![
                BlockPlan::new(5, 6, 1, "router", Some("cfg".into())),
                BlockPlan::with_config_value(
                    8,
                    9,
                    2,
                    "junction",
                    Some("copper".into()),
                    TypeValue::Content(ContentRef::new(ContentType::Item, 4)),
                ),
            ]
        );
        state.teams.register_core(CoreInfo::new(99, 7, 0.0, 0.0));
        let exported = state.export_legacy_team_blocks(
            |name| match name {
                "router" => Some(2),
                "junction" => Some(3),
                _ => None,
            },
            false,
        );
        assert_eq!(
            exported.groups[0].plans[1].config,
            TypeValue::Content(ContentRef::new(ContentType::Item, 4))
        );

        state.apply_legacy_team_blocks(None, |_| None, |_, _| None);
        assert!(state.teams.get_or_null(7).unwrap().plans.is_empty());
    }

    #[test]
    fn export_legacy_team_blocks_uses_runtime_teams_for_save_version_writer() {
        let mut state = GameState::new();
        state
            .teams
            .register_core(CoreInfo::new(1, TEAM_CRUX, 0.0, 0.0));
        state
            .teams
            .register_core(CoreInfo::new(2, TEAM_MALIS, 10.0, 0.0));
        state.teams.replace_plans([
            (
                TEAM_CRUX,
                vec![
                    BlockPlan::new(1, 2, 0, "duo", None),
                    BlockPlan::new(3, 4, 1, "router", Some("cfg".into())),
                ],
            ),
            (TEAM_MALIS, vec![BlockPlan::new(5, 6, 2, "wall", None)]),
        ]);

        let exported = state.export_legacy_team_blocks(
            |name| match name {
                "duo" => Some(10),
                "router" => Some(11),
                "wall" => Some(12),
                _ => None,
            },
            true,
        );

        assert_eq!(exported.groups.len(), 3);
        assert_eq!(exported.groups[0].team_id, TEAM_CRUX as i32);
        assert_eq!(exported.groups[0].plans[0].block_id, 10);
        assert_eq!(
            exported.groups[0].plans[1].config,
            TypeValue::String("cfg".into())
        );
        assert_eq!(exported.groups[1].team_id, TEAM_MALIS as i32);
        assert_eq!(exported.groups[1].plans[0].block_id, 12);
        assert_eq!(exported.groups[2].team_id, TEAM_SHARDED as i32);
    }

    #[test]
    fn apply_network_world_data_applies_supported_rules_json_and_resyncs_teams() {
        let mut state = GameState::new();
        let world = NetworkWorldData {
            rules_json: r#"{
                "waves": true,
                "waveTimer": false,
                "attackMode": true,
                "pvp": true,
                "defaultTeam": 6,
                "waveTeam": 9,
                "modeName": "PvP",
                "planet": "erekir",
                "env": 11
            }"#
            .into(),
            ..NetworkWorldData::default()
        };

        let result = state.apply_network_world_data(&world);

        assert!(result.rules_loaded);
        assert_eq!(result.rules_error, None);
        assert!(state.rules.waves);
        assert!(!state.rules.wave_timer);
        assert!(state.rules.attack_mode);
        assert!(state.rules.pvp);
        assert_eq!(state.rules.default_team, 6);
        assert_eq!(state.rules.wave_team, 9);
        assert_eq!(state.rules.mode_name.as_deref(), Some("PvP"));
        assert_eq!(state.rules.planet, "erekir");
        assert_eq!(state.rules.env, 11);
        assert!(state.teams.get_active().contains(&9));
    }

    #[test]
    fn apply_network_world_data_reports_invalid_rules_json_without_mutating_rules() {
        let mut state = GameState::new();
        state.rules.waves = true;
        state.rules.default_team = 3;

        let result = state.apply_network_world_data(&NetworkWorldData {
            rules_json: r#"{"waves": tru}"#.into(),
            ..NetworkWorldData::default()
        });

        assert!(!result.rules_loaded);
        assert!(result.rules_error.is_some());
        assert!(state.rules.waves);
        assert_eq!(state.rules.default_team, 3);
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
    fn advance_game_update_frame_matches_java_tick_and_update_id_gate() {
        let mut state = GameState::new();

        let menu = state.advance_game_update_frame(0.5);
        assert_eq!(
            menu,
            GameUpdateFrameAdvance {
                advanced: false,
                delta_ticks: 0.0,
                tick: 0.0,
                update_id: 0,
            }
        );

        state.set(GameStateState::Playing);
        let first = state.advance_game_update_frame(0.5);
        assert_eq!(
            first,
            GameUpdateFrameAdvance {
                advanced: true,
                delta_ticks: 30.0,
                tick: 30.0,
                update_id: 1,
            }
        );
        let invalid_delta = state.advance_game_update_frame(f32::NAN);
        assert!(invalid_delta.advanced);
        assert_eq!(invalid_delta.delta_ticks, 0.0);
        assert_eq!(invalid_delta.tick, 30.0);
        assert_eq!(invalid_delta.update_id, 2);

        state.set(GameStateState::Paused);
        let paused = state.advance_game_update_frame(1.0);
        assert_eq!(
            paused,
            GameUpdateFrameAdvance {
                advanced: false,
                delta_ticks: 0.0,
                tick: 30.0,
                update_id: 2,
            }
        );
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
        assert_eq!(state.rand_seed0, 11);
        assert_eq!(state.rand_seed1, 22);
        assert_eq!(state.universe.seconds(true), 456);
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
        assert_eq!(state.rand_seed0, 33);
        assert_eq!(state.rand_seed1, 44);
        assert_eq!(state.universe.seconds(true), 789);
        assert!(state.is_playing());
    }

    #[test]
    fn apply_state_snapshot_updates_core_items_from_java_core_data() {
        let mut state = GameState::new();
        state.set(GameStateState::Playing);

        let mut core_data = Vec::new();
        core_data.push(2);
        core_data.push(TEAM_SHARDED);
        core_data.extend_from_slice(&2i16.to_be_bytes());
        core_data.extend_from_slice(&0i16.to_be_bytes());
        core_data.extend_from_slice(&75i32.to_be_bytes());
        core_data.extend_from_slice(&3i16.to_be_bytes());
        core_data.extend_from_slice(&12i32.to_be_bytes());
        core_data.push(TEAM_CRUX);
        core_data.extend_from_slice(&1i16.to_be_bytes());
        core_data.extend_from_slice(&1i16.to_be_bytes());
        core_data.extend_from_slice(&5i32.to_be_bytes());

        let snapshot = StateSnapshotCallPacket {
            wave_time: 12.5,
            wave: 9,
            enemies: 17,
            paused: false,
            game_over: false,
            time_data: 456,
            tps: 60,
            rand0: 11,
            rand1: 22,
            core_data,
        };

        state.apply_state_snapshot(&snapshot);

        assert_eq!(
            state.teams.get_or_null(TEAM_SHARDED).unwrap().core_items,
            BTreeMap::from([(0, 75), (3, 12)])
        );
        assert_eq!(
            state.teams.get_or_null(TEAM_CRUX).unwrap().core_items,
            BTreeMap::from([(1, 5)])
        );
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
        assert_eq!(state.rand_seed0, 0);
        assert_eq!(state.rand_seed1, 0);
        assert_eq!(state.universe.seconds(true), 123);
    }
}
