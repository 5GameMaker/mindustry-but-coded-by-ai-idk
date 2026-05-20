//! Campaign-specific rule overlay mirroring upstream `CampaignRules`.

use crate::mindustry::game::{Difficulty, Rules};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CampaignRules {
    pub difficulty: Difficulty,
    pub fog: bool,
    pub show_spawns: bool,
    pub sector_invasion: bool,
    pub random_wave_ai: bool,
    pub legacy_launch_pads: bool,
    pub rts_ai: bool,
    pub clear_sector_on_lose: bool,
    pub pause_disabled: bool,
}

impl Default for CampaignRules {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            fog: false,
            show_spawns: false,
            sector_invasion: false,
            random_wave_ai: false,
            legacy_launch_pads: false,
            rts_ai: false,
            clear_sector_on_lose: false,
            pause_disabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CampaignPlanetRules {
    pub show_rts_ai_rule: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CampaignRulesApplyResult {
    pub rts_ai_swapped: bool,
}

impl CampaignRules {
    pub fn apply(
        &self,
        planet: CampaignPlanetRules,
        rules: &mut Rules,
    ) -> CampaignRulesApplyResult {
        rules.static_fog = self.fog;
        rules.fog = self.fog;
        rules.show_spawns = self.show_spawns;
        rules.random_wave_ai = self.random_wave_ai;
        rules.pause_disabled = self.pause_disabled;

        let multipliers = self.difficulty.multipliers();
        rules.objective_timer_multiplier = multipliers.wave_time_multiplier;

        let wave_team = rules.wave_team as usize;
        let mut rts_ai_swapped = false;
        let team = rules.teams.get_or_insert(wave_team);

        if planet.show_rts_ai_rule {
            let enabled = self.rts_ai && rules.attack_mode;
            rts_ai_swapped = team.rts_ai != enabled;
            team.rts_ai = enabled;
            team.rts_max_squad = 15;
        }

        team.block_health_multiplier = multipliers.enemy_health_multiplier;
        team.unit_health_multiplier = multipliers.enemy_health_multiplier;
        team.unit_cost_multiplier = 1.0 / multipliers.enemy_spawn_multiplier;
        team.unit_build_speed_multiplier = multipliers.enemy_spawn_multiplier;

        CampaignRulesApplyResult { rts_ai_swapped }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::game::TeamRule;

    #[test]
    fn defaults_match_java_field_initializers() {
        let campaign = CampaignRules::default();
        assert_eq!(campaign.difficulty, Difficulty::Normal);
        assert!(!campaign.fog);
        assert!(!campaign.show_spawns);
        assert!(!campaign.sector_invasion);
        assert!(!campaign.random_wave_ai);
        assert!(!campaign.legacy_launch_pads);
        assert!(!campaign.rts_ai);
        assert!(!campaign.clear_sector_on_lose);
        assert!(!campaign.pause_disabled);

        let team = TeamRule::default();
        assert_eq!(team.rts_max_squad, 50);
        assert_eq!(team.unit_cost_multiplier, 1.0);
        assert_eq!(team.unit_build_speed_multiplier, 1.0);
    }

    #[test]
    fn apply_copies_campaign_flags_and_difficulty_multipliers() {
        let campaign = CampaignRules {
            difficulty: Difficulty::Hard,
            fog: true,
            show_spawns: true,
            random_wave_ai: true,
            pause_disabled: true,
            ..CampaignRules::default()
        };
        let mut rules = Rules::default();

        let result = campaign.apply(CampaignPlanetRules::default(), &mut rules);

        assert!(!result.rts_ai_swapped);
        assert!(rules.static_fog);
        assert!(rules.fog);
        assert!(rules.show_spawns);
        assert!(rules.random_wave_ai);
        assert!(rules.pause_disabled);
        assert_eq!(rules.objective_timer_multiplier, 0.8);

        let team = rules.teams.get_or_default(rules.wave_team as usize);
        assert_eq!(team.block_health_multiplier, 1.25);
        assert_eq!(team.unit_health_multiplier, 1.25);
        assert!((team.unit_cost_multiplier - (1.0 / 1.5)).abs() < f32::EPSILON);
        assert_eq!(team.unit_build_speed_multiplier, 1.5);
    }

    #[test]
    fn apply_toggles_rts_ai_only_when_planet_rule_is_visible_and_attack_mode_stays_enabled() {
        let campaign = CampaignRules {
            rts_ai: true,
            ..CampaignRules::default()
        };
        let mut rules = Rules {
            attack_mode: true,
            ..Rules::default()
        };
        let planet = CampaignPlanetRules {
            show_rts_ai_rule: true,
        };

        let first = campaign.apply(planet, &mut rules);
        assert!(first.rts_ai_swapped);
        let team = rules.teams.get_or_default(rules.wave_team as usize);
        assert!(team.rts_ai);
        assert_eq!(team.rts_max_squad, 15);

        let second = campaign.apply(planet, &mut rules);
        assert!(!second.rts_ai_swapped);

        rules.attack_mode = false;
        let disabled = campaign.apply(planet, &mut rules);
        assert!(disabled.rts_ai_swapped);
        assert!(!rules.teams.get_or_default(rules.wave_team as usize).rts_ai);
    }
}
