//! Campaign-rules dialog model mirroring upstream `mindustry.ui.dialogs.CampaignRulesDialog`.

use crate::mindustry::{
    game::{difficulty::DifficultyTextBundle, CampaignRules, Difficulty},
    ui::{upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale},
};

pub const CAMPAIGN_RULES_DIALOG_TITLE: &str = "@campaign.difficulty";
pub const CAMPAIGN_RULES_DIFFICULTY_TABLE_BACKGROUND: &str = "Tex.button";
pub const CAMPAIGN_RULES_DIFFICULTY_TABLE_MARGIN: f32 = 10.0;
pub const CAMPAIGN_RULES_DIFFICULTY_BUTTON_STYLE: &str = "Styles.flatTogglet";
pub const CAMPAIGN_RULES_DIFFICULTY_BUTTON_SIZE: (f32, f32) = (140.0, 50.0);
pub const CAMPAIGN_RULES_INNER_PAD: f32 = 5.0;

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignRulesDialogPlanet {
    pub name: String,
    pub campaign_rules: CampaignRules,
    pub allow_sector_invasion: bool,
    pub show_rts_ai_rule: bool,
    pub clear_sector_on_lose: bool,
}

impl CampaignRulesDialogPlanet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            campaign_rules: CampaignRules::default(),
            allow_sector_invasion: false,
            show_rts_ai_rule: false,
            clear_sector_on_lose: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignRulesDialogContext {
    pub portrait: bool,
}

impl Default for CampaignRulesDialogContext {
    fn default() -> Self {
        Self { portrait: false }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CampaignRulesDialogRuntime {
    pub state_is_game: bool,
    pub state_is_campaign: bool,
    pub current_planet_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignRulesDialogModel {
    pub title: &'static str,
    pub close_button_added: bool,
    pub content_top_aligned: bool,
    pub pane_grow_y: bool,
    pub inner_top_aligned: bool,
    pub inner_left_aligned: bool,
    pub inner_defaults_fill_x: bool,
    pub inner_defaults_left: bool,
    pub inner_defaults_pad: f32,
    pub difficulty_table: CampaignRulesDifficultyTable,
    pub checks: Vec<CampaignRulesCheckRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignRulesDifficultyTable {
    pub background: &'static str,
    pub margin: f32,
    pub button_style: &'static str,
    pub button_size: (f32, f32),
    pub buttons: Vec<CampaignRulesDifficultyButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignRulesDifficultyButton {
    pub difficulty: Difficulty,
    pub text: String,
    pub checked: bool,
    pub tooltip: String,
    pub row_after: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignRulesCheckKind {
    SectorInvasion,
    Fog,
    ShowSpawns,
    RandomWaveAi,
    PauseDisabled,
    RtsAi,
    ClearSectorOnLose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignRulesCheckRow {
    pub kind: CampaignRulesCheckKind,
    pub text: &'static str,
    pub checked: bool,
    pub disabled: bool,
    pub tooltip: Option<String>,
    pub left_aligned: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CampaignRulesDialogAction {
    ShowDialog,
    Rebuild,
    SavePlanetRules {
        planet_name: String,
        rules: CampaignRules,
    },
    ApplyCampaignRulesToState {
        planet_name: String,
        rules: CampaignRules,
    },
    SetRules,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignRulesDialog {
    pub planet: Option<CampaignRulesDialogPlanet>,
}

impl Default for CampaignRulesDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl CampaignRulesDialog {
    pub fn new() -> Self {
        Self { planet: None }
    }

    pub fn show(
        &mut self,
        planet: CampaignRulesDialogPlanet,
        context: &CampaignRulesDialogContext,
        locale: &str,
    ) -> CampaignRulesDialogModel {
        self.planet = Some(planet);
        self.rebuild(context, locale)
    }

    pub fn show_plan() -> Vec<CampaignRulesDialogAction> {
        vec![CampaignRulesDialogAction::ShowDialog]
    }

    pub fn rebuild(
        &self,
        context: &CampaignRulesDialogContext,
        locale: &str,
    ) -> CampaignRulesDialogModel {
        let planet = self
            .planet
            .as_ref()
            .expect("campaign rules planet must exist");
        CampaignRulesDialogModel {
            title: CAMPAIGN_RULES_DIALOG_TITLE,
            close_button_added: true,
            content_top_aligned: true,
            pane_grow_y: true,
            inner_top_aligned: true,
            inner_left_aligned: true,
            inner_defaults_fill_x: true,
            inner_defaults_left: true,
            inner_defaults_pad: CAMPAIGN_RULES_INNER_PAD,
            difficulty_table: difficulty_table(&planet.campaign_rules, context, locale),
            checks: check_rows(planet, locale),
        }
    }

    pub fn on_resize(
        &self,
        context: &CampaignRulesDialogContext,
        locale: &str,
    ) -> (Vec<CampaignRulesDialogAction>, CampaignRulesDialogModel) {
        (
            vec![CampaignRulesDialogAction::Rebuild],
            self.rebuild(context, locale),
        )
    }

    pub fn select_difficulty_plan(
        &mut self,
        difficulty: Difficulty,
    ) -> Vec<CampaignRulesDialogAction> {
        self.planet_mut().campaign_rules.difficulty = difficulty;
        Vec::new()
    }

    pub fn set_check_plan(
        &mut self,
        kind: CampaignRulesCheckKind,
        value: bool,
    ) -> Vec<CampaignRulesDialogAction> {
        let rules = &mut self.planet_mut().campaign_rules;
        match kind {
            CampaignRulesCheckKind::SectorInvasion => rules.sector_invasion = value,
            CampaignRulesCheckKind::Fog => rules.fog = value,
            CampaignRulesCheckKind::ShowSpawns => rules.show_spawns = value,
            CampaignRulesCheckKind::RandomWaveAi => rules.random_wave_ai = value,
            CampaignRulesCheckKind::PauseDisabled => rules.pause_disabled = value,
            CampaignRulesCheckKind::RtsAi => rules.rts_ai = value,
            CampaignRulesCheckKind::ClearSectorOnLose => rules.clear_sector_on_lose = value,
        }
        Vec::new()
    }

    pub fn hidden_plan(
        &self,
        runtime: &CampaignRulesDialogRuntime,
    ) -> Vec<CampaignRulesDialogAction> {
        let Some(planet) = &self.planet else {
            return Vec::new();
        };

        let mut actions = vec![CampaignRulesDialogAction::SavePlanetRules {
            planet_name: planet.name.clone(),
            rules: planet.campaign_rules,
        }];

        if runtime.state_is_game
            && runtime.state_is_campaign
            && runtime.current_planet_name.as_deref() == Some(planet.name.as_str())
        {
            actions.push(CampaignRulesDialogAction::ApplyCampaignRulesToState {
                planet_name: planet.name.clone(),
                rules: planet.campaign_rules,
            });
            actions.push(CampaignRulesDialogAction::SetRules);
        }

        actions
    }

    fn planet_mut(&mut self) -> &mut CampaignRulesDialogPlanet {
        self.planet
            .as_mut()
            .expect("campaign rules planet must exist")
    }
}

fn difficulty_table(
    rules: &CampaignRules,
    context: &CampaignRulesDialogContext,
    locale: &str,
) -> CampaignRulesDifficultyTable {
    let bundle = CampaignRulesDifficultyBundle { locale };
    CampaignRulesDifficultyTable {
        background: CAMPAIGN_RULES_DIFFICULTY_TABLE_BACKGROUND,
        margin: CAMPAIGN_RULES_DIFFICULTY_TABLE_MARGIN,
        button_style: CAMPAIGN_RULES_DIFFICULTY_BUTTON_STYLE,
        button_size: CAMPAIGN_RULES_DIFFICULTY_BUTTON_SIZE,
        buttons: Difficulty::ALL
            .iter()
            .copied()
            .enumerate()
            .map(|(index, difficulty)| CampaignRulesDifficultyButton {
                difficulty,
                text: difficulty.localized_with(&bundle),
                checked: rules.difficulty == difficulty,
                tooltip: difficulty.info_with(&bundle),
                row_after: context.portrait && index % 2 == 1,
            })
            .collect(),
    }
}

fn check_rows(planet: &CampaignRulesDialogPlanet, locale: &str) -> Vec<CampaignRulesCheckRow> {
    let rules = planet.campaign_rules;
    let mut rows = Vec::new();

    if planet.allow_sector_invasion {
        rows.push(check_row(
            CampaignRulesCheckKind::SectorInvasion,
            "@rules.invasions",
            rules.sector_invasion,
            locale,
        ));
    }

    rows.extend([
        check_row(CampaignRulesCheckKind::Fog, "@rules.fog", rules.fog, locale),
        check_row(
            CampaignRulesCheckKind::ShowSpawns,
            "@rules.showspawns",
            rules.show_spawns,
            locale,
        ),
        check_row(
            CampaignRulesCheckKind::RandomWaveAi,
            "@rules.randomwaveai",
            rules.random_wave_ai,
            locale,
        ),
        check_row(
            CampaignRulesCheckKind::PauseDisabled,
            "@rules.pauseDisabled",
            rules.pause_disabled,
            locale,
        ),
    ]);

    if planet.show_rts_ai_rule {
        rows.push(check_row(
            CampaignRulesCheckKind::RtsAi,
            "@rules.rtsai.campaign",
            rules.rts_ai,
            locale,
        ));
    }

    if !planet.clear_sector_on_lose {
        rows.push(check_row(
            CampaignRulesCheckKind::ClearSectorOnLose,
            "@rules.clearsectoronloss",
            rules.clear_sector_on_lose,
            locale,
        ));
    }

    rows
}

fn check_row(
    kind: CampaignRulesCheckKind,
    text: &'static str,
    checked: bool,
    locale: &str,
) -> CampaignRulesCheckRow {
    let info_key = format!("{}.info", text.trim_start_matches('@'));
    CampaignRulesCheckRow {
        kind,
        text,
        checked,
        disabled: false,
        tooltip: upstream_menu_bundle_value_for_locale(locale, &info_key)
            .map(|_| format!("{text}.info")),
        left_aligned: true,
    }
}

struct CampaignRulesDifficultyBundle<'a> {
    locale: &'a str,
}

impl DifficultyTextBundle for CampaignRulesDifficultyBundle<'_> {
    fn get(&self, key: &str) -> String {
        upstream_menu_bundle_value_for_locale(self.locale, key)
            .or_else(|| upstream_menu_bundle_value_for_locale("en", key))
            .unwrap_or(key)
            .to_string()
    }

    fn format(&self, key: &str, value: &str) -> String {
        upstream_menu_bundle_format_for_locale(self.locale, key, &[value])
            .or_else(|| upstream_menu_bundle_format_for_locale("en", key, &[value]))
            .unwrap_or_else(|| format!("{key}:{value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planet() -> CampaignRulesDialogPlanet {
        let mut planet = CampaignRulesDialogPlanet::new("serpulo");
        planet.allow_sector_invasion = true;
        planet.show_rts_ai_rule = true;
        planet.campaign_rules = CampaignRules {
            difficulty: Difficulty::Hard,
            fog: true,
            show_spawns: true,
            random_wave_ai: true,
            pause_disabled: false,
            rts_ai: true,
            clear_sector_on_lose: false,
            sector_invasion: true,
            ..CampaignRules::default()
        };
        planet
    }

    #[test]
    fn show_builds_difficulty_group_and_campaign_check_rows_like_java_rebuild() {
        let mut dialog = CampaignRulesDialog::new();
        let model = dialog.show(
            planet(),
            &CampaignRulesDialogContext { portrait: false },
            "en",
        );

        assert_eq!(model.title, "@campaign.difficulty");
        assert!(model.close_button_added);
        assert!(model.content_top_aligned);
        assert!(model.pane_grow_y);
        assert!(model.inner_top_aligned);
        assert!(model.inner_left_aligned);
        assert!(model.inner_defaults_fill_x);
        assert_eq!(model.inner_defaults_pad, 5.0);
        assert_eq!(model.difficulty_table.background, "Tex.button");
        assert_eq!(model.difficulty_table.margin, 10.0);
        assert_eq!(model.difficulty_table.button_style, "Styles.flatTogglet");
        assert_eq!(model.difficulty_table.button_size, (140.0, 50.0));
        assert_eq!(model.difficulty_table.buttons.len(), 5);
        assert_eq!(model.difficulty_table.buttons[0].text, "Casual");
        assert_eq!(
            model.difficulty_table.buttons[3].difficulty,
            Difficulty::Hard
        );
        assert!(model.difficulty_table.buttons[3].checked);
        assert!(model.difficulty_table.buttons[3]
            .tooltip
            .contains("Enemy Health"));

        assert_eq!(
            model.checks.iter().map(|row| row.kind).collect::<Vec<_>>(),
            vec![
                CampaignRulesCheckKind::SectorInvasion,
                CampaignRulesCheckKind::Fog,
                CampaignRulesCheckKind::ShowSpawns,
                CampaignRulesCheckKind::RandomWaveAi,
                CampaignRulesCheckKind::PauseDisabled,
                CampaignRulesCheckKind::RtsAi,
                CampaignRulesCheckKind::ClearSectorOnLose,
            ]
        );
        assert_eq!(model.checks[0].text, "@rules.invasions");
        assert!(model.checks[0].checked);
        assert_eq!(
            model.checks[3].tooltip,
            Some("@rules.randomwaveai.info".into())
        );
        assert_eq!(
            model.checks[5].tooltip,
            Some("@rules.rtsai.campaign.info".into())
        );
        assert_eq!(
            model.checks[6].tooltip,
            Some("@rules.clearsectoronloss.info".into())
        );
        assert!(model
            .checks
            .iter()
            .all(|row| !row.disabled && row.left_aligned));
    }

    #[test]
    fn portrait_rebuild_breaks_difficulty_rows_after_odd_ordinals() {
        let mut dialog = CampaignRulesDialog::new();
        let model = dialog.show(
            planet(),
            &CampaignRulesDialogContext { portrait: true },
            "en",
        );

        assert_eq!(
            model
                .difficulty_table
                .buttons
                .iter()
                .map(|button| button.row_after)
                .collect::<Vec<_>>(),
            vec![false, true, false, true, false]
        );
    }

    #[test]
    fn planet_flags_gate_optional_invasion_rts_and_clear_sector_checks() {
        let mut planet = CampaignRulesDialogPlanet::new("erekir");
        planet.clear_sector_on_lose = true;
        let mut dialog = CampaignRulesDialog::new();
        let model = dialog.show(planet, &CampaignRulesDialogContext::default(), "en");

        assert_eq!(
            model.checks.iter().map(|row| row.kind).collect::<Vec<_>>(),
            vec![
                CampaignRulesCheckKind::Fog,
                CampaignRulesCheckKind::ShowSpawns,
                CampaignRulesCheckKind::RandomWaveAi,
                CampaignRulesCheckKind::PauseDisabled,
            ]
        );
    }

    #[test]
    fn selecting_difficulty_and_checks_mutates_planet_rules_without_side_effect_actions() {
        let mut dialog = CampaignRulesDialog::new();
        dialog.show(planet(), &CampaignRulesDialogContext::default(), "en");

        assert!(dialog
            .select_difficulty_plan(Difficulty::Eradication)
            .is_empty());
        assert_eq!(
            dialog.planet.as_ref().unwrap().campaign_rules.difficulty,
            Difficulty::Eradication
        );
        assert!(dialog
            .set_check_plan(CampaignRulesCheckKind::Fog, false)
            .is_empty());
        assert!(!dialog.planet.as_ref().unwrap().campaign_rules.fog);
        assert!(dialog
            .set_check_plan(CampaignRulesCheckKind::ClearSectorOnLose, true)
            .is_empty());
        assert!(
            dialog
                .planet
                .as_ref()
                .unwrap()
                .campaign_rules
                .clear_sector_on_lose
        );
    }

    #[test]
    fn hidden_saves_planet_rules_and_applies_to_current_campaign_planet_only() {
        let mut dialog = CampaignRulesDialog::new();
        dialog.show(planet(), &CampaignRulesDialogContext::default(), "en");

        assert_eq!(
            dialog.hidden_plan(&CampaignRulesDialogRuntime::default()),
            vec![CampaignRulesDialogAction::SavePlanetRules {
                planet_name: "serpulo".into(),
                rules: planet().campaign_rules,
            }]
        );

        let actions = dialog.hidden_plan(&CampaignRulesDialogRuntime {
            state_is_game: true,
            state_is_campaign: true,
            current_planet_name: Some("serpulo".into()),
        });
        assert_eq!(
            actions,
            vec![
                CampaignRulesDialogAction::SavePlanetRules {
                    planet_name: "serpulo".into(),
                    rules: planet().campaign_rules,
                },
                CampaignRulesDialogAction::ApplyCampaignRulesToState {
                    planet_name: "serpulo".into(),
                    rules: planet().campaign_rules,
                },
                CampaignRulesDialogAction::SetRules,
            ]
        );

        assert_eq!(
            dialog.hidden_plan(&CampaignRulesDialogRuntime {
                state_is_game: true,
                state_is_campaign: true,
                current_planet_name: Some("erekir".into()),
            }),
            vec![CampaignRulesDialogAction::SavePlanetRules {
                planet_name: "serpulo".into(),
                rules: planet().campaign_rules,
            }]
        );
    }

    #[test]
    fn hidden_without_planet_is_noop_and_on_resize_rebuilds_current_model() {
        let dialog = CampaignRulesDialog::new();
        assert!(dialog
            .hidden_plan(&CampaignRulesDialogRuntime::default())
            .is_empty());

        let mut dialog = CampaignRulesDialog::new();
        dialog.show(planet(), &CampaignRulesDialogContext::default(), "en");
        let (actions, model) = dialog.on_resize(&CampaignRulesDialogContext::default(), "en");

        assert_eq!(actions, vec![CampaignRulesDialogAction::Rebuild]);
        assert_eq!(model.title, "@campaign.difficulty");
        assert_eq!(model.checks.len(), 7);
    }

    #[test]
    fn show_plan_only_requests_dialog_display_after_rebuild_like_java_show() {
        assert_eq!(
            CampaignRulesDialog::show_plan(),
            vec![CampaignRulesDialogAction::ShowDialog]
        );
    }
}
