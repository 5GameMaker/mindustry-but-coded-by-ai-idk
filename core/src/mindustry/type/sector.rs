use std::collections::HashMap;

use crate::mindustry::game::SectorInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorPreset {
    pub name: String,
    pub localized_name: String,
    pub always_unlocked: bool,
    pub override_launch_defaults: bool,
    pub allow_launch_schematics: bool,
    pub allow_launch_loadout: bool,
    pub require_unlock: bool,
    pub show_hidden: bool,
    pub capture_wave: i32,
    pub shield_sector_ids: Vec<i32>,
}

impl SectorPreset {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            always_unlocked: false,
            override_launch_defaults: false,
            allow_launch_schematics: false,
            allow_launch_loadout: false,
            require_unlock: false,
            show_hidden: false,
            capture_wave: -1,
            shield_sector_ids: Vec::new(),
        }
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
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

    pub fn capture_wave(mut self, capture_wave: i32) -> Self {
        self.capture_wave = capture_wave;
        self
    }

    pub fn shielded_by(mut self, sector_id: i32) -> Self {
        self.shield_sector_ids.push(sector_id);
        self
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
