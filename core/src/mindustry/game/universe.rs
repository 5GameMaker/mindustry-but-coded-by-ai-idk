use std::collections::{BTreeMap, HashMap};

use crate::mindustry::r#type::ItemSeq;

use super::{Schematic, SchematicTile};

pub const UNIVERSE_SECONDS_KEY: &str = "utimei";
pub const UNIVERSE_TURN_KEY: &str = "turn";
pub const LAUNCH_RESOURCES_KEY: &str = "launch-resources-seq";
pub const LAST_LOADOUT_PREFIX: &str = "lastloadout-";
pub const DEFAULT_LOADOUT_CORES: [&str; 3] = ["core-shard", "core-nucleus", "core-foundation"];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UniverseSettings {
    ints: BTreeMap<String, i32>,
    strings: BTreeMap<String, String>,
    item_seq_maps: BTreeMap<String, BTreeMap<String, i32>>,
}

impl UniverseSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_int(&self, key: &str) -> i32 {
        self.ints.get(key).copied().unwrap_or_default()
    }

    pub fn put_int(&mut self, key: impl Into<String>, value: i32) {
        self.ints.insert(key.into(), value);
    }

    pub fn get_string(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_default()
    }

    pub fn put_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.strings.insert(key.into(), value.into());
    }

    pub fn put_item_seq(&mut self, key: impl Into<String>, value: &ItemSeq) {
        self.item_seq_maps.insert(key.into(), value.to_json_map());
    }

    pub fn get_item_seq_map(&self, key: &str) -> Option<&BTreeMap<String, i32>> {
        self.item_seq_maps.get(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.ints.remove(key);
        self.strings.remove(key);
        self.item_seq_maps.remove(key);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniverseUpdate {
    pub ran_turn: bool,
    pub saved: bool,
    pub elapsed_seconds: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniverseTurn {
    pub turn: i32,
    pub new_seconds_passed: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Universe {
    item_names: Vec<String>,
    seconds: i32,
    net_seconds: i32,
    second_counter: f32,
    turn: i32,
    turn_counter: f32,
    last_loadout: Option<Schematic>,
    last_launch_resources: ItemSeq,
}

impl Universe {
    pub fn new<I, S>(item_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::with_settings(item_names, &UniverseSettings::new())
    }

    pub fn with_settings<I, S>(item_names: I, settings: &UniverseSettings) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let item_names: Vec<String> = item_names.into_iter().map(Into::into).collect();
        let mut last_launch_resources = ItemSeq::new(item_names.clone());
        if let Some(values) = settings.get_item_seq_map(LAUNCH_RESOURCES_KEY) {
            last_launch_resources.read_json_map(values);
        }

        Self {
            item_names,
            seconds: settings.get_int(UNIVERSE_SECONDS_KEY),
            net_seconds: 0,
            second_counter: 0.0,
            turn: settings.get_int(UNIVERSE_TURN_KEY),
            turn_counter: 0.0,
            last_loadout: None,
            last_launch_resources,
        }
    }

    pub fn turn(&self) -> i32 {
        self.turn
    }

    pub fn raw_seconds(&self) -> i32 {
        self.seconds
    }

    pub fn second_counter(&self) -> f32 {
        self.second_counter
    }

    pub fn turn_counter(&self) -> f32 {
        self.turn_counter
    }

    pub fn update(
        &mut self,
        delta_ticks: f32,
        turn_duration_ticks: f32,
        is_net_client: bool,
        settings: &mut UniverseSettings,
    ) -> UniverseUpdate {
        let mut update = UniverseUpdate {
            ran_turn: false,
            saved: false,
            elapsed_seconds: 0,
        };

        if is_net_client {
            return update;
        }

        self.second_counter += delta_ticks / 60.0;
        self.turn_counter += delta_ticks;

        if self.turn_counter >= turn_duration_ticks {
            self.turn_counter = 0.0;
            self.run_turn(turn_duration_ticks, settings);
            update.ran_turn = true;
            update.saved = true;
        }

        if self.second_counter >= 1.0 {
            let whole = self.second_counter as i32;
            self.seconds += whole;
            self.second_counter %= 1.0;
            update.elapsed_seconds = whole;

            if self.seconds % 10 == 1 {
                self.save(settings);
                update.saved = true;
            }
        }

        update
    }

    pub fn run_turn(
        &mut self,
        turn_duration_ticks: f32,
        settings: &mut UniverseSettings,
    ) -> UniverseTurn {
        self.turn += 1;
        let new_seconds_passed = (turn_duration_ticks / 60.0) as i32;
        self.save(settings);
        UniverseTurn {
            turn: self.turn,
            new_seconds_passed,
        }
    }

    pub fn update_net_seconds(&mut self, value: i32) {
        self.net_seconds = value;
    }

    pub fn seconds(&self, is_net_client: bool) -> i32 {
        if is_net_client {
            self.net_seconds
        } else {
            self.seconds
        }
    }

    pub fn seconds_mod(&self, modulus: f32, scale: f32, is_net_client: bool) -> f32 {
        (self.seconds(is_net_client) as f32 / scale) % modulus
    }

    pub fn set_seconds(&mut self, seconds: f32, settings: &mut UniverseSettings) {
        self.seconds = seconds as i32;
        self.second_counter = seconds - self.seconds as f32;
        self.save(settings);
    }

    pub fn secondsf(&self, is_net_client: bool) -> f32 {
        self.seconds(is_net_client) as f32 + self.second_counter
    }

    pub fn save(&self, settings: &mut UniverseSettings) {
        settings.put_int(UNIVERSE_SECONDS_KEY, self.seconds);
        settings.put_int(UNIVERSE_TURN_KEY, self.turn);
    }

    pub fn clear_loadout_info(&mut self, settings: &mut UniverseSettings) {
        self.last_loadout = None;
        self.last_launch_resources = ItemSeq::new(self.item_names.clone());
        settings.remove(LAUNCH_RESOURCES_KEY);
        for core in DEFAULT_LOADOUT_CORES {
            settings.remove(&last_loadout_key(core));
        }
    }

    pub fn get_launch_resources(&mut self, settings: &UniverseSettings) -> &ItemSeq {
        self.last_launch_resources = ItemSeq::new(self.item_names.clone());
        if let Some(values) = settings.get_item_seq_map(LAUNCH_RESOURCES_KEY) {
            self.last_launch_resources.read_json_map(values);
        }
        &self.last_launch_resources
    }

    pub fn update_launch_resources(&mut self, stacks: ItemSeq, settings: &mut UniverseSettings) {
        self.last_launch_resources = stacks;
        settings.put_item_seq(LAUNCH_RESOURCES_KEY, &self.last_launch_resources);
    }

    pub fn update_loadout_core_block(
        &mut self,
        core_name: impl Into<String>,
        core_size: i32,
        settings: &mut UniverseSettings,
    ) {
        let core_name = core_name.into();
        let schematic = Schematic::new(
            vec![SchematicTile::new(core_name.clone(), 0, 0, None, 0)],
            HashMap::new(),
            core_size,
            core_size,
        );
        self.update_loadout(core_name, schematic, settings);
    }

    pub fn update_loadout(
        &mut self,
        core_name: impl AsRef<str>,
        schematic: Schematic,
        settings: &mut UniverseSettings,
    ) {
        let file_name = schematic
            .file
            .as_deref()
            .map(file_stem_like_java)
            .unwrap_or_default();
        settings.put_string(last_loadout_key(core_name.as_ref()), file_name);
        self.last_loadout = Some(schematic);
    }

    pub fn get_last_loadout_or(&mut self, default_loadout: Schematic) -> Schematic {
        if self.last_loadout.is_none() {
            self.last_loadout = Some(default_loadout);
        }
        self.last_loadout.clone().unwrap()
    }

    pub fn get_loadout(
        &self,
        core_name: &str,
        loadouts: &[Schematic],
        settings: &UniverseSettings,
    ) -> Option<Schematic> {
        let selected = settings.get_string(&last_loadout_key(core_name));
        loadouts
            .iter()
            .find(|schematic| {
                schematic
                    .file
                    .as_deref()
                    .map(file_stem_like_java)
                    .as_deref()
                    == Some(selected.as_str())
            })
            .or_else(|| loadouts.first())
            .cloned()
    }
}

pub fn last_loadout_key(core_name: &str) -> String {
    format!("{LAST_LOADOUT_PREFIX}{core_name}")
}

pub fn file_stem_like_java(path: &str) -> String {
    let name = path
        .rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(path);
    name.rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schematic_with_file(file: Option<&str>, name: &str) -> Schematic {
        let mut tags = HashMap::new();
        tags.insert("name".into(), name.into());
        let mut schematic = Schematic::new(Vec::new(), tags, 1, 1);
        schematic.file = file.map(str::to_string);
        schematic
    }

    #[test]
    fn universe_loads_and_saves_java_settings_keys() {
        let mut settings = UniverseSettings::new();
        settings.put_int(UNIVERSE_SECONDS_KEY, 12);
        settings.put_int(UNIVERSE_TURN_KEY, 3);

        let mut universe = Universe::with_settings(["copper", "lead"], &settings);
        assert_eq!(universe.raw_seconds(), 12);
        assert_eq!(universe.turn(), 3);

        universe.set_seconds(42.75, &mut settings);
        assert_eq!(universe.raw_seconds(), 42);
        assert!((universe.second_counter() - 0.75).abs() < f32::EPSILON);
        assert_eq!(settings.get_int(UNIVERSE_SECONDS_KEY), 42);
        assert_eq!(settings.get_int(UNIVERSE_TURN_KEY), 3);
    }

    #[test]
    fn universe_update_matches_java_time_turn_and_autosave_edges() {
        let mut settings = UniverseSettings::new();
        let mut universe = Universe::new(["copper"]);

        let update = universe.update(61.0, 120.0, false, &mut settings);
        assert_eq!(
            update,
            UniverseUpdate {
                ran_turn: false,
                saved: true,
                elapsed_seconds: 1
            }
        );
        assert_eq!(universe.raw_seconds(), 1);
        assert_eq!(settings.get_int(UNIVERSE_SECONDS_KEY), 1);

        let update = universe.update(60.0, 120.0, false, &mut settings);
        assert!(update.ran_turn);
        assert!(update.saved);
        assert_eq!(universe.turn(), 1);
        assert_eq!(settings.get_int(UNIVERSE_TURN_KEY), 1);
        assert_eq!(universe.turn_counter(), 0.0);

        let before = universe.clone();
        let update = universe.update(600.0, 120.0, true, &mut settings);
        assert_eq!(
            update,
            UniverseUpdate {
                ran_turn: false,
                saved: false,
                elapsed_seconds: 0
            }
        );
        assert_eq!(universe, before);
    }

    #[test]
    fn network_seconds_and_mod_follow_java_accessors() {
        let mut universe = Universe::new(["copper"]);
        let mut settings = UniverseSettings::new();
        universe.set_seconds(10.5, &mut settings);
        universe.update_net_seconds(31);

        assert_eq!(universe.seconds(false), 10);
        assert_eq!(universe.seconds(true), 31);
        assert_eq!(universe.secondsf(false), 10.5);
        assert_eq!(universe.secondsf(true), 31.5);
        assert_eq!(universe.seconds_mod(8.0, 2.0, true), 7.5);
    }

    #[test]
    fn launch_resources_roundtrip_through_settings_json_map() {
        let mut settings = UniverseSettings::new();
        let mut universe = Universe::new(["copper", "lead"]);
        let mut resources = ItemSeq::new(["copper", "lead"]);
        resources.add_name("lead", 7);

        universe.update_launch_resources(resources, &mut settings);

        let mut reloaded = Universe::with_settings(["copper", "lead"], &settings);
        assert_eq!(reloaded.get_launch_resources(&settings).get_name("lead"), 7);
    }

    #[test]
    fn loadout_keys_and_selection_match_java_lastloadout_rules() {
        let mut settings = UniverseSettings::new();
        let mut universe = Universe::new(["copper"]);
        let alpha = schematic_with_file(Some("schematics/alpha.msch"), "alpha");
        let beta = schematic_with_file(Some("C:\\schematics\\beta.msch"), "beta");

        universe.update_loadout("core-shard", beta.clone(), &mut settings);
        assert_eq!(settings.get_string("lastloadout-core-shard"), "beta");
        assert_eq!(
            universe.get_loadout("core-shard", &[alpha.clone(), beta.clone()], &settings),
            Some(beta)
        );

        settings.put_string("lastloadout-core-shard", "missing");
        assert_eq!(
            universe.get_loadout("core-shard", &[alpha.clone()], &settings),
            Some(alpha)
        );

        universe.clear_loadout_info(&mut settings);
        assert_eq!(settings.get_string("lastloadout-core-shard"), "");
        assert!(settings.get_item_seq_map(LAUNCH_RESOURCES_KEY).is_none());
    }

    #[test]
    fn generated_core_loadout_is_single_core_tile_like_java_overload() {
        let mut settings = UniverseSettings::new();
        let mut universe = Universe::new(["copper"]);
        universe.update_loadout_core_block("core-foundation", 4, &mut settings);
        let schematic = universe.get_last_loadout_or(schematic_with_file(None, "fallback"));

        assert_eq!(schematic.width, 4);
        assert_eq!(schematic.height, 4);
        assert_eq!(schematic.tiles.len(), 1);
        assert_eq!(schematic.tiles[0].block, "core-foundation");
        assert_eq!(schematic.tiles[0].x, 0);
        assert_eq!(schematic.tiles[0].y, 0);
        assert_eq!(settings.get_string("lastloadout-core-foundation"), "");
    }

    #[test]
    fn file_stem_uses_last_path_segment_and_removes_extension() {
        assert_eq!(file_stem_like_java("foo.msch"), "foo");
        assert_eq!(file_stem_like_java("C:\\maps\\bar.msch"), "bar");
        assert_eq!(file_stem_like_java("/tmp/noext"), "noext");
    }
}
