use std::fmt;

use crate::mindustry::maps::MapDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gamemode {
    Survival,
    Sandbox,
    Attack,
    Pvp,
    Editor,
}

impl Gamemode {
    pub const ALL: [Gamemode; 5] = [
        Gamemode::Survival,
        Gamemode::Sandbox,
        Gamemode::Attack,
        Gamemode::Pvp,
        Gamemode::Editor,
    ];

    pub const fn hidden(self) -> bool {
        matches!(self, Gamemode::Editor)
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            Gamemode::Survival => "survival",
            Gamemode::Sandbox => "sandbox",
            Gamemode::Attack => "attack",
            Gamemode::Pvp => "pvp",
            Gamemode::Editor => "editor",
        }
    }

    pub fn valid(self, map: &MapDescriptor) -> bool {
        match self {
            Gamemode::Survival => map.spawns > 0,
            Gamemode::Attack | Gamemode::Pvp => map.teams.len() > 1,
            Gamemode::Sandbox | Gamemode::Editor => true,
        }
    }

    pub fn name_bundle_key(self) -> String {
        format!("mode.{}.name", self.wire_name())
    }

    pub fn description_bundle_key(self) -> String {
        format!("mode.{}.description", self.wire_name())
    }

    pub fn localized_name(self) -> String {
        self.localized_name_with(&GamemodeBundleKeys)
    }

    pub fn localized_name_with(self, bundle: &impl GamemodeTextBundle) -> String {
        bundle.get(&self.name_bundle_key())
    }

    pub fn description(self) -> String {
        self.description_with(&GamemodeBundleKeys)
    }

    pub fn description_with(self, bundle: &impl GamemodeTextBundle) -> String {
        bundle.get(&self.description_bundle_key())
    }
}

impl fmt::Display for Gamemode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.localized_name())
    }
}

pub trait GamemodeTextBundle {
    fn get(&self, key: &str) -> String;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GamemodeBundleKeys;

impl GamemodeTextBundle for GamemodeBundleKeys {
    fn get(&self, key: &str) -> String {
        key.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    struct TestBundle;

    impl GamemodeTextBundle for TestBundle {
        fn get(&self, key: &str) -> String {
            format!("@{key}")
        }
    }

    fn map_with(spawns: i32, teams: &[i32]) -> MapDescriptor {
        let mut map = MapDescriptor::new("maps/test.msav", 32, 32, BTreeMap::new(), true, 11, 157);
        map.spawns = spawns;
        map.teams = teams.to_vec();
        map
    }

    #[test]
    fn gamemode_valid_matches_java_map_validators() {
        let empty = map_with(0, &[]);
        assert!(!Gamemode::Survival.valid(&empty));
        assert!(Gamemode::Sandbox.valid(&empty));
        assert!(!Gamemode::Attack.valid(&empty));
        assert!(!Gamemode::Pvp.valid(&empty));
        assert!(Gamemode::Editor.valid(&empty));

        let survival = map_with(1, &[]);
        assert!(Gamemode::Survival.valid(&survival));

        let one_team = map_with(0, &[1]);
        assert!(!Gamemode::Attack.valid(&one_team));
        assert!(!Gamemode::Pvp.valid(&one_team));

        let two_teams = map_with(0, &[1, 2]);
        assert!(Gamemode::Attack.valid(&two_teams));
        assert!(Gamemode::Pvp.valid(&two_teams));
    }

    #[test]
    fn gamemode_bundle_keys_match_java_description_and_to_string_keys() {
        assert_eq!(Gamemode::Survival.name_bundle_key(), "mode.survival.name");
        assert_eq!(
            Gamemode::Survival.description_bundle_key(),
            "mode.survival.description"
        );
        assert_eq!(Gamemode::Editor.localized_name(), "mode.editor.name");
        assert_eq!(Gamemode::Attack.description(), "mode.attack.description");
        assert_eq!(
            Gamemode::Pvp.localized_name_with(&TestBundle),
            "@mode.pvp.name"
        );
        assert_eq!(
            Gamemode::Pvp.description_with(&TestBundle),
            "@mode.pvp.description"
        );
        assert_eq!(Gamemode::Sandbox.to_string(), "mode.sandbox.name");
    }
}
