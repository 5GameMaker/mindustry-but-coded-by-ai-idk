//! Mirrors the mutable rules snapshot touched by upstream world logic instructions.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicRulesState {
    pub wave_timer: bool,
    pub wave: i32,
    pub wave_time: f32,
    pub waves: bool,
    pub wave_sending: bool,
    pub attack_mode: bool,
    pub wave_spacing: f32,
    pub enemy_core_build_radius: f32,
    pub drop_zone_radius: f32,
    pub unit_cap: i32,
    pub lighting: bool,
    pub can_game_over: bool,
    pub pause_disabled: bool,
    pub ambient_light: f64,
    pub solar_multiplier: f32,
    pub drag_multiplier: f32,
    pub map_area: Option<(i32, i32, i32, i32)>,
    pub banned_blocks: BTreeSet<String>,
    pub banned_units: BTreeSet<String>,
    pub team_rules: BTreeMap<u8, LogicTeamRules>,
    pub mission: String,
}

impl Default for LogicRulesState {
    fn default() -> Self {
        Self {
            wave_timer: false,
            wave: 1,
            wave_time: 0.0,
            waves: false,
            wave_sending: false,
            attack_mode: false,
            wave_spacing: 0.0,
            enemy_core_build_radius: 0.0,
            drop_zone_radius: 0.0,
            unit_cap: 0,
            lighting: false,
            can_game_over: true,
            pause_disabled: false,
            ambient_light: 0.0,
            solar_multiplier: 1.0,
            drag_multiplier: 1.0,
            map_area: None,
            banned_blocks: BTreeSet::new(),
            banned_units: BTreeSet::new(),
            team_rules: BTreeMap::new(),
            mission: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicTeamRules {
    pub build_speed_multiplier: f32,
    pub unit_health_multiplier: f32,
    pub unit_build_speed_multiplier: f32,
    pub unit_mine_speed_multiplier: f32,
    pub unit_cost_multiplier: f32,
    pub unit_damage_multiplier: f32,
    pub block_health_multiplier: f32,
    pub block_damage_multiplier: f32,
    pub rts_min_weight: f32,
    pub rts_min_squad: i32,
}

impl Default for LogicTeamRules {
    fn default() -> Self {
        Self {
            build_speed_multiplier: 1.0,
            unit_health_multiplier: 1.0,
            unit_build_speed_multiplier: 1.0,
            unit_mine_speed_multiplier: 1.0,
            unit_cost_multiplier: 1.0,
            unit_damage_multiplier: 1.0,
            block_health_multiplier: 1.0,
            block_damage_multiplier: 1.0,
            rts_min_weight: 0.0,
            rts_min_squad: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LogicRulesState, LogicTeamRules};

    #[test]
    fn logic_rules_state_defaults_match_java_rule_mutation_baseline() {
        let mut rules = LogicRulesState::default();
        assert!(!rules.wave_timer);
        assert_eq!(rules.wave, 1);
        assert_eq!(rules.wave_time, 0.0);
        assert!(!rules.waves);
        assert!(!rules.wave_sending);
        assert!(!rules.attack_mode);
        assert_eq!(rules.unit_cap, 0);
        assert!(!rules.lighting);
        assert!(rules.can_game_over);
        assert!(!rules.pause_disabled);
        assert_eq!(rules.solar_multiplier, 1.0);
        assert_eq!(rules.drag_multiplier, 1.0);
        assert!(rules.map_area.is_none());
        assert!(rules.banned_blocks.is_empty());
        assert!(rules.banned_units.is_empty());
        assert!(rules.team_rules.is_empty());
        assert!(rules.mission.is_empty());

        rules.banned_blocks.insert("@duo".into());
        rules.banned_units.insert("@dagger".into());
        rules.team_rules.insert(1, LogicTeamRules::default());
        assert!(rules.banned_blocks.contains("@duo"));
        assert!(rules.banned_units.contains("@dagger"));
        assert_eq!(rules.team_rules[&1].build_speed_multiplier, 1.0);
    }

    #[test]
    fn logic_team_rules_defaults_match_java_team_rule_multipliers() {
        let team = LogicTeamRules::default();
        assert_eq!(team.build_speed_multiplier, 1.0);
        assert_eq!(team.unit_health_multiplier, 1.0);
        assert_eq!(team.unit_build_speed_multiplier, 1.0);
        assert_eq!(team.unit_mine_speed_multiplier, 1.0);
        assert_eq!(team.unit_cost_multiplier, 1.0);
        assert_eq!(team.unit_damage_multiplier, 1.0);
        assert_eq!(team.block_health_multiplier, 1.0);
        assert_eq!(team.block_damage_multiplier, 1.0);
        assert_eq!(team.rts_min_weight, 0.0);
        assert_eq!(team.rts_min_squad, 0);
    }
}
