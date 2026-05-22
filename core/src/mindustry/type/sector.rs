use std::collections::HashMap;

use crate::mindustry::{
    ctype::{Content, ContentId, ContentType},
    game::SectorInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SectorPreset {
    pub id: ContentId,
    pub name: String,
    pub localized_name: String,
    pub description: Option<String>,
    pub always_unlocked: bool,
    pub override_launch_defaults: bool,
    pub allow_launch_schematics: bool,
    pub allow_launch_loadout: bool,
    pub require_unlock: bool,
    pub show_hidden: bool,
    pub show_sector_land_info: bool,
    pub capture_wave: i32,
    pub difficulty: f32,
    pub start_wave_time_multiplier: f32,
    pub add_starting_items: bool,
    pub no_lighting: bool,
    pub is_last_sector: bool,
    pub attack_after_waves: bool,
    pub original_position: i32,
    pub file_name: Option<String>,
    pub planet_name: Option<String>,
    pub sector_id: Option<i32>,
    pub shield_sector_ids: Vec<i32>,
}

impl SectorPreset {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            id: -1,
            localized_name: name.clone(),
            description: None,
            name,
            always_unlocked: false,
            override_launch_defaults: false,
            allow_launch_schematics: false,
            allow_launch_loadout: false,
            require_unlock: true,
            show_hidden: false,
            show_sector_land_info: true,
            capture_wave: 0,
            difficulty: 0.0,
            start_wave_time_multiplier: 2.0,
            add_starting_items: false,
            no_lighting: false,
            is_last_sector: false,
            attack_after_waves: false,
            original_position: 0,
            file_name: None,
            planet_name: None,
            sector_id: None,
            shield_sector_ids: Vec::new(),
        }
    }

    pub fn with_planet_sector(
        name: impl Into<String>,
        planet_name: impl Into<String>,
        sector: i32,
    ) -> Self {
        let mut preset = Self::new(name);
        preset.initialize(planet_name, sector, false);
        preset
    }

    pub fn with_id(mut self, id: ContentId) -> Self {
        self.id = id;
        self
    }

    pub fn with_file_planet_sector(
        name: impl Into<String>,
        file_name: impl Into<String>,
        planet_name: impl Into<String>,
        sector: i32,
    ) -> Self {
        let mut preset = Self::new(name);
        preset.file_name = Some(file_name.into());
        preset.initialize(planet_name, sector, false);
        preset
    }

    pub fn initialize(
        &mut self,
        planet_name: impl Into<String>,
        sector: i32,
        override_remap: bool,
    ) {
        self.planet_name = Some(planet_name.into());
        self.original_position = sector;
        self.sector_id = Some(if sector == -1 { 0 } else { sector });
        if override_remap {
            self.file_name.get_or_insert_with(|| self.name.clone());
        }
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn always_unlocked(mut self, always_unlocked: bool) -> Self {
        self.always_unlocked = always_unlocked;
        self
    }

    pub fn override_launch_defaults(
        mut self,
        allow_launch_schematics: bool,
        allow_launch_loadout: bool,
    ) -> Self {
        self.override_launch_defaults = true;
        self.allow_launch_schematics = allow_launch_schematics;
        self.allow_launch_loadout = allow_launch_loadout;
        self
    }

    pub fn require_unlock(mut self, require_unlock: bool) -> Self {
        self.require_unlock = require_unlock;
        self
    }

    pub fn show_hidden(mut self, show_hidden: bool) -> Self {
        self.show_hidden = show_hidden;
        self
    }

    pub fn show_sector_land_info(mut self, show_sector_land_info: bool) -> Self {
        self.show_sector_land_info = show_sector_land_info;
        self
    }

    pub fn capture_wave(mut self, capture_wave: i32) -> Self {
        self.capture_wave = capture_wave;
        self
    }

    pub fn difficulty(mut self, difficulty: f32) -> Self {
        self.difficulty = difficulty;
        self
    }

    pub fn start_wave_time_multiplier(mut self, multiplier: f32) -> Self {
        self.start_wave_time_multiplier = multiplier;
        self
    }

    pub fn add_starting_items(mut self, add_starting_items: bool) -> Self {
        self.add_starting_items = add_starting_items;
        self
    }

    pub fn no_lighting(mut self, no_lighting: bool) -> Self {
        self.no_lighting = no_lighting;
        self
    }

    pub fn last_sector(mut self, is_last_sector: bool) -> Self {
        self.is_last_sector = is_last_sector;
        self
    }

    pub fn attack_after_waves(mut self, attack_after_waves: bool) -> Self {
        self.attack_after_waves = attack_after_waves;
        self
    }

    pub fn shielded_by(mut self, sector_id: i32) -> Self {
        self.shield_sector_ids.push(sector_id);
        self
    }

    pub fn is_hidden(&self) -> bool {
        self.description.is_none()
    }

    pub fn content_type(&self) -> crate::mindustry::ctype::ContentType {
        crate::mindustry::ctype::ContentType::Sector
    }

    pub fn generator_map_name(&self) -> &str {
        self.file_name.as_deref().unwrap_or(&self.name)
    }
}

impl Content for SectorPreset {
    fn id(&self) -> ContentId {
        self.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::Sector
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorPlanetDefaults {
    pub name: String,
    pub localized_name: String,
    pub allow_launch_schematics: bool,
    pub allow_launch_loadout: bool,
    pub sector_count: usize,
}

impl SectorPlanetDefaults {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            allow_launch_schematics: false,
            allow_launch_loadout: false,
            sector_count: 0,
        }
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn launch_defaults(
        mut self,
        allow_launch_schematics: bool,
        allow_launch_loadout: bool,
    ) -> Self {
        self.allow_launch_schematics = allow_launch_schematics;
        self.allow_launch_loadout = allow_launch_loadout;
        self
    }

    pub fn sector_count(mut self, sector_count: usize) -> Self {
        self.sector_count = sector_count;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SectorRuntimeState {
    pub is_current_sector: bool,
    pub game_over: bool,
    pub net_client: bool,
    pub rules_waves: bool,
    pub rules_attack_mode: bool,
}

impl SectorRuntimeState {
    pub fn is_being_played(self) -> bool {
        self.is_current_sector && !self.game_over && !self.net_client
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sector {
    pub id: i32,
    pub info: SectorInfo,
    pub save_exists: bool,
    pub preset: Option<SectorPreset>,
    pub threat: f32,
    pub generate_enemy_base: bool,
}

impl Sector {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            info: SectorInfo::default(),
            save_exists: false,
            preset: None,
            threat: 0.0,
            generate_enemy_base: false,
        }
    }

    pub fn has_save(&self) -> bool {
        self.save_exists
    }

    pub fn clear_info(&mut self) {
        self.info = SectorInfo::default();
    }

    pub fn has_base(&self, runtime: SectorRuntimeState) -> bool {
        self.save_exists && self.info.has_core && !(runtime.is_current_sector && runtime.game_over)
    }

    pub fn unlocked(&self, runtime: SectorRuntimeState) -> bool {
        self.has_base(runtime)
            || self
                .preset
                .as_ref()
                .is_some_and(|preset| preset.always_unlocked)
    }

    pub fn locked(&self, runtime: SectorRuntimeState) -> bool {
        !self.unlocked(runtime)
    }

    pub fn allow_launch_schematics(&self, planet: &SectorPlanetDefaults) -> bool {
        self.preset
            .as_ref()
            .filter(|preset| preset.override_launch_defaults)
            .map_or(planet.allow_launch_schematics, |preset| {
                preset.allow_launch_schematics
            })
    }

    pub fn allow_launch_loadout(&self, planet: &SectorPlanetDefaults) -> bool {
        self.preset
            .as_ref()
            .filter(|preset| preset.override_launch_defaults)
            .map_or(planet.allow_launch_loadout, |preset| {
                preset.allow_launch_loadout
            })
    }

    pub fn is_attacked(&self, runtime: SectorRuntimeState) -> bool {
        if runtime.is_being_played() {
            runtime.rules_waves || runtime.rules_attack_mode
        } else {
            self.save_exists && (self.info.waves || self.info.attack) && self.info.has_core
        }
    }

    pub fn is_frozen(&self, runtime: SectorRuntimeState) -> bool {
        self.is_attacked(runtime) && !runtime.is_being_played()
    }

    pub fn has_enemy_base(&self, runtime: SectorRuntimeState) -> bool {
        ((self.generate_enemy_base && self.preset.is_none())
            || self
                .preset
                .as_ref()
                .is_some_and(|preset| preset.capture_wave == 0))
            && (!self.save_exists || self.info.attack || !self.has_base(runtime))
    }

    pub fn is_captured(&self, runtime: SectorRuntimeState) -> bool {
        if runtime.is_being_played() {
            !runtime.rules_waves && !runtime.rules_attack_mode
        } else {
            self.save_exists && !self.info.waves && !self.info.attack
        }
    }

    pub fn is_shielded(&self, captured_by_sector_id: &HashMap<i32, bool>) -> bool {
        self.preset.as_ref().is_some_and(|preset| {
            !preset.shield_sector_ids.is_empty()
                && preset
                    .shield_sector_ids
                    .iter()
                    .any(|id| !captured_by_sector_id.get(id).copied().unwrap_or(false))
        })
    }

    pub fn name(&self, planet: &SectorPlanetDefaults) -> String {
        if let Some(preset) = &self.preset {
            if self.info.name.is_none() && (preset.require_unlock || preset.show_hidden) {
                return preset.localized_name.clone();
            }
        }

        if self.info.name.is_none() && planet.sector_count == 1 {
            return planet.localized_name.clone();
        }

        self.info
            .name
            .clone()
            .unwrap_or_else(|| self.id.to_string())
    }

    pub fn stored_item_amount(&self, item: &str) -> i32 {
        self.info
            .items
            .iter()
            .find_map(|(name, amount)| (name == item).then_some(*amount))
            .unwrap_or(0)
    }

    pub fn add_stored_item(&mut self, item: impl Into<String>, amount: i32) {
        let item = item.into();
        let current = self.stored_item_amount(&item);
        let mut next = current + amount;
        if amount > 0 {
            next = next.min(self.info.storage_capacity);
        }
        next = next.max(0);
        set_stored_item(&mut self.info.items, item, next);
    }

    pub fn remove_stored_item(&mut self, item: impl Into<String>, amount: i32) {
        self.add_stored_item(item, -amount);
    }
}

fn set_stored_item(items: &mut Vec<(String, i32)>, item: String, amount: i32) {
    if let Some((_, existing)) = items.iter_mut().find(|(name, _)| *name == item) {
        *existing = amount;
    } else if amount != 0 {
        items.push((item, amount));
    }
    items.retain(|(_, amount)| *amount != 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_state_flags_follow_java_save_and_runtime_branches() {
        let mut sector = Sector::new(12);
        assert!(!sector.has_save());
        assert!(!sector.has_base(SectorRuntimeState::default()));
        assert!(sector.locked(SectorRuntimeState::default()));

        sector.save_exists = true;
        assert!(sector.has_base(SectorRuntimeState::default()));
        assert!(sector.unlocked(SectorRuntimeState::default()));
        assert!(sector.is_attacked(SectorRuntimeState::default()));

        sector.info.waves = false;
        sector.info.attack = false;
        assert!(sector.is_captured(SectorRuntimeState::default()));
        assert!(!sector.is_frozen(SectorRuntimeState::default()));

        let playing_attack = SectorRuntimeState {
            is_current_sector: true,
            rules_attack_mode: true,
            ..SectorRuntimeState::default()
        };
        assert!(sector.is_attacked(playing_attack));
        assert!(!sector.is_frozen(playing_attack));
        assert!(!sector.is_captured(playing_attack));

        let game_over = SectorRuntimeState {
            is_current_sector: true,
            game_over: true,
            ..SectorRuntimeState::default()
        };
        assert!(!sector.has_base(game_over));
    }

    #[test]
    fn preset_and_planet_launch_defaults_match_java_override_branch() {
        let planet = SectorPlanetDefaults::new("serpulo").launch_defaults(true, false);
        let mut sector = Sector::new(1);
        assert!(sector.allow_launch_schematics(&planet));
        assert!(!sector.allow_launch_loadout(&planet));

        sector.preset = Some(SectorPreset::new("preset").override_launch_defaults(false, true));
        assert!(!sector.allow_launch_schematics(&planet));
        assert!(sector.allow_launch_loadout(&planet));

        sector.save_exists = false;
        sector.info.has_core = false;
        assert!(!sector.unlocked(SectorRuntimeState::default()));
        sector.preset = Some(SectorPreset::new("preset").always_unlocked(true));
        assert!(sector.unlocked(SectorRuntimeState::default()));
    }

    #[test]
    fn sector_preset_defaults_match_java_field_initializers() {
        let preset = SectorPreset::new("groundZero");

        assert_eq!(preset.id, -1);
        assert_eq!(preset.id(), -1);
        assert_eq!(preset.name, "groundZero");
        assert_eq!(preset.localized_name, "groundZero");
        assert_eq!(preset.description, None);
        assert_eq!(preset.generator_map_name(), "groundZero");
        assert!(!preset.always_unlocked);
        assert!(!preset.override_launch_defaults);
        assert!(!preset.allow_launch_schematics);
        assert!(!preset.allow_launch_loadout);
        assert!(preset.require_unlock);
        assert!(!preset.show_hidden);
        assert!(preset.show_sector_land_info);
        assert_eq!(preset.capture_wave, 0);
        assert_eq!(preset.difficulty, 0.0);
        assert_eq!(preset.start_wave_time_multiplier, 2.0);
        assert!(!preset.add_starting_items);
        assert!(!preset.no_lighting);
        assert!(!preset.is_last_sector);
        assert!(!preset.attack_after_waves);
        assert_eq!(preset.original_position, 0);
        assert_eq!(preset.file_name, None);
        assert_eq!(preset.planet_name, None);
        assert_eq!(preset.sector_id, None);
        assert!(preset.shield_sector_ids.is_empty());
        assert!(preset.is_hidden());
        assert_eq!(
            preset.content_type(),
            crate::mindustry::ctype::ContentType::Sector
        );
    }

    #[test]
    fn sector_preset_hidden_depends_on_description_like_java() {
        let localized_without_description =
            SectorPreset::new("groundZero").localized("Ground Zero");
        assert!(localized_without_description.is_hidden());

        let described = SectorPreset::new("groundZero")
            .localized("Ground Zero")
            .description("The beginning.");
        assert!(!described.is_hidden());
    }

    #[test]
    fn sector_preset_initialize_records_planet_sector_and_file_name_like_java_constructor() {
        let preset =
            SectorPreset::with_file_planet_sector("craters", "craters-file", "serpulo", 18)
                .difficulty(4.0)
                .capture_wave(30)
                .start_wave_time_multiplier(1.5)
                .add_starting_items(true)
                .no_lighting(true)
                .last_sector(true)
                .attack_after_waves(true)
                .show_sector_land_info(false);

        assert_eq!(preset.name, "craters");
        assert_eq!(preset.file_name.as_deref(), Some("craters-file"));
        assert_eq!(preset.generator_map_name(), "craters-file");
        assert_eq!(preset.planet_name.as_deref(), Some("serpulo"));
        assert_eq!(preset.original_position, 18);
        assert_eq!(preset.sector_id, Some(18));
        assert_eq!(preset.difficulty, 4.0);
        assert_eq!(preset.capture_wave, 30);
        assert_eq!(preset.start_wave_time_multiplier, 1.5);
        assert!(preset.add_starting_items);
        assert!(preset.no_lighting);
        assert!(preset.is_last_sector);
        assert!(preset.attack_after_waves);
        assert!(!preset.show_sector_land_info);

        let mut unassigned = SectorPreset::new("orphan");
        unassigned.initialize("serpulo", -1, false);
        assert_eq!(unassigned.original_position, -1);
        assert_eq!(unassigned.sector_id, Some(0));
        assert_eq!(unassigned.generator_map_name(), "orphan");
    }

    #[test]
    fn sector_name_uses_preset_planet_or_custom_info_like_java() {
        let single = SectorPlanetDefaults::new("moon")
            .localized("Moon")
            .sector_count(1);
        let multi = SectorPlanetDefaults::new("serpulo")
            .localized("Serpulo")
            .sector_count(10);
        let mut sector = Sector::new(7);

        assert_eq!(sector.name(&single), "Moon");
        assert_eq!(sector.name(&multi), "7");

        sector.info.name = Some("Custom".into());
        assert_eq!(sector.name(&multi), "Custom");

        sector.info.name = None;
        sector.preset = Some(
            SectorPreset::new("groundZero")
                .localized("Ground Zero")
                .require_unlock(true),
        );
        assert_eq!(sector.name(&multi), "Ground Zero");
    }

    #[test]
    fn enemy_base_and_shield_logic_follow_java_predicates() {
        let mut sector = Sector::new(3);
        sector.generate_enemy_base = true;
        assert!(sector.has_enemy_base(SectorRuntimeState::default()));

        sector.save_exists = true;
        sector.info.has_core = true;
        assert!(!sector.has_enemy_base(SectorRuntimeState::default()));

        sector.info.attack = true;
        assert!(sector.has_enemy_base(SectorRuntimeState::default()));

        sector.preset = Some(SectorPreset::new("enemy").capture_wave(0).shielded_by(9));
        let mut captured = HashMap::new();
        captured.insert(9, false);
        assert!(sector.is_shielded(&captured));
        captured.insert(9, true);
        assert!(!sector.is_shielded(&captured));
    }

    #[test]
    fn stored_items_are_clamped_to_sector_capacity_and_never_negative() {
        let mut sector = Sector::new(4);
        sector.info.storage_capacity = 100;

        sector.add_stored_item("copper", 120);
        assert_eq!(sector.stored_item_amount("copper"), 100);

        sector.remove_stored_item("copper", 30);
        assert_eq!(sector.stored_item_amount("copper"), 70);

        sector.remove_stored_item("copper", 1000);
        assert_eq!(sector.stored_item_amount("copper"), 0);
        assert!(sector.info.items.is_empty());
    }

    #[test]
    fn clear_info_resets_sector_info_without_changing_identity() {
        let mut sector = Sector::new(5);
        sector.info.name = Some("Old".into());
        sector.info.storage_capacity = 50;
        sector.save_exists = true;

        sector.clear_info();

        assert_eq!(sector.id, 5);
        assert!(sector.save_exists);
        assert_eq!(sector.info, SectorInfo::default());
    }
}
