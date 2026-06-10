//! Game-over dialog model mirroring upstream `mindustry.ui.dialogs.GameOverDialog`.

use crate::mindustry::{
    game::{Difficulty, GameStats},
    ui::{upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale},
};

pub const GAME_OVER_DIALOG_TITLE: &str = "@gameover";
pub const GAME_OVER_MIN_WIDTH: f32 = 370.0;
pub const GAME_OVER_MAX_SIZE: (f32, f32) = (600.0, 550.0);
pub const GAME_OVER_BUTTON_SIZE: (f32, f32) = (170.0, 60.0);
pub const GAME_OVER_MENU_BUTTON_SIZE: (f32, f32) = (140.0, 60.0);
pub const GAME_OVER_STATS_PANE_PAD: f32 = 12.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOverContext {
    pub winner_colored_name: Option<String>,
    pub winner_is_player_team: bool,
    pub rules_pvp: bool,
    pub rules_waves: bool,
    pub is_campaign: bool,
    pub sector_name: Option<String>,
    pub net_client: bool,
    pub high_score: bool,
    pub current_save_playtime: Option<String>,
    pub stats: GameStats,
    pub mobile: bool,
    pub allow_campaign_rules: bool,
    pub sector_attempts: i32,
    pub campaign_difficulty: Difficulty,
    pub difficulty_guide_once: bool,
}

impl Default for GameOverContext {
    fn default() -> Self {
        Self {
            winner_colored_name: None,
            winner_is_player_team: false,
            rules_pvp: false,
            rules_waves: false,
            is_campaign: false,
            sector_name: None,
            net_client: false,
            high_score: false,
            current_save_playtime: None,
            stats: GameStats::default(),
            mobile: false,
            allow_campaign_rules: false,
            sector_attempts: 0,
            campaign_difficulty: Difficulty::Normal,
            difficulty_guide_once: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameOverStatRow {
    pub label: String,
    pub value: i32,
    pub delay: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverButtonAction {
    Disconnect,
    Continue,
    Menu,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameOverButton {
    pub text: &'static str,
    pub action: GameOverButtonAction,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameOverDialogModel {
    pub title: &'static str,
    pub fill_parent: bool,
    pub title_table_removed: bool,
    pub header_text: String,
    pub high_score_visible: bool,
    pub stat_rows: Vec<GameOverStatRow>,
    pub playtime_label: Option<String>,
    pub playtime_value: Option<String>,
    pub waiting_label: Option<&'static str>,
    pub buttons: Vec<GameOverButton>,
    pub min_width: f32,
    pub max_size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameOverDialogAction {
    StoreHudShown(bool),
    SetHudShown(bool),
    SetAfterGameOver(bool),
    FireWinEvent,
    FireLoseEvent,
    HideDialog,
    ShowPlanet,
    ResetLogic,
    ResetNet,
    SetStateMenu,
    CheckPlaytest,
    ShowDifficultyGuide { width: i32 },
    ShowCampaignRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameOverDialog;

impl GameOverDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn shown_actions(previous_hud_shown: bool) -> Vec<GameOverDialogAction> {
        vec![
            GameOverDialogAction::StoreHudShown(previous_hud_shown),
            GameOverDialogAction::SetHudShown(false),
        ]
    }

    pub fn hidden_actions(previous_hud_shown: bool) -> Vec<GameOverDialogAction> {
        vec![GameOverDialogAction::SetHudShown(previous_hud_shown)]
    }

    pub fn show_actions(context: &GameOverContext) -> Vec<GameOverDialogAction> {
        let mut actions = vec![GameOverDialogAction::SetAfterGameOver(true)];
        actions.push(if context.winner_is_player_team {
            GameOverDialogAction::FireWinEvent
        } else {
            GameOverDialogAction::FireLoseEvent
        });
        actions
    }

    pub fn rebuild(&self, context: &GameOverContext, locale: &str) -> GameOverDialogModel {
        let buttons = if context.is_campaign {
            if context.net_client {
                vec![GameOverButton {
                    text: "@gameover.disconnect",
                    action: GameOverButtonAction::Disconnect,
                    size: GAME_OVER_BUTTON_SIZE,
                }]
            } else {
                vec![GameOverButton {
                    text: "@continue",
                    action: GameOverButtonAction::Continue,
                    size: GAME_OVER_BUTTON_SIZE,
                }]
            }
        } else {
            vec![GameOverButton {
                text: "@menu",
                action: GameOverButtonAction::Menu,
                size: GAME_OVER_MENU_BUTTON_SIZE,
            }]
        };

        GameOverDialogModel {
            title: GAME_OVER_DIALOG_TITLE,
            fill_parent: true,
            title_table_removed: true,
            header_text: header_text(context, locale),
            high_score_visible: context.high_score,
            stat_rows: stat_rows(context, locale),
            playtime_label: context
                .current_save_playtime
                .as_ref()
                .map(|_| bundle_value(locale, "stats.playtime")),
            playtime_value: context
                .current_save_playtime
                .as_ref()
                .map(|playtime| format!("[accent]{playtime}")),
            waiting_label: (context.is_campaign && context.net_client)
                .then_some("@gameover.waiting"),
            buttons,
            min_width: GAME_OVER_MIN_WIDTH,
            max_size: GAME_OVER_MAX_SIZE,
        }
    }

    pub fn disconnect_plan() -> Vec<GameOverDialogAction> {
        vec![
            GameOverDialogAction::ResetLogic,
            GameOverDialogAction::ResetNet,
            GameOverDialogAction::HideDialog,
            GameOverDialogAction::SetStateMenu,
        ]
    }

    pub fn continue_plan(context: &GameOverContext) -> Vec<GameOverDialogAction> {
        let mut actions = vec![
            GameOverDialogAction::HideDialog,
            GameOverDialogAction::ShowPlanet,
        ];
        if should_show_difficulty_guide(context) {
            actions.push(GameOverDialogAction::ShowDifficultyGuide {
                width: if context.mobile { 400 } else { 500 },
            });
        }
        actions
    }

    pub fn menu_plan(playtest_handled: bool) -> Vec<GameOverDialogAction> {
        let mut actions = vec![
            GameOverDialogAction::HideDialog,
            GameOverDialogAction::CheckPlaytest,
        ];
        if !playtest_handled {
            actions.push(GameOverDialogAction::ResetLogic);
        }
        actions
    }
}

pub fn should_show_difficulty_guide(context: &GameOverContext) -> bool {
    context.allow_campaign_rules
        && context.sector_attempts >= 2
        && context.campaign_difficulty == Difficulty::Normal
        && context.difficulty_guide_once
}

fn header_text(context: &GameOverContext, locale: &str) -> String {
    if context.rules_pvp {
        if let Some(winner) = &context.winner_colored_name {
            return bundle_format(locale, "gameover.pvp", &[winner.as_str()]);
        }
    }

    if context.is_campaign {
        bundle_format(
            locale,
            "sector.lost",
            &[context.sector_name.as_deref().unwrap_or("")],
        )
    } else {
        "@gameover".into()
    }
}

fn stat_rows(context: &GameOverContext, locale: &str) -> Vec<GameOverStatRow> {
    let mut rows = Vec::new();
    if context.rules_waves {
        rows.push(stat_row(
            bundle_value(locale, "stats.wave"),
            context.stats.waves_lasted,
            0.0,
        ));
    }
    rows.extend([
        stat_row(
            bundle_value(locale, "stats.unitsCreated"),
            context.stats.units_created,
            0.05,
        ),
        stat_row(
            bundle_value(locale, "stats.enemiesDestroyed"),
            context.stats.enemy_units_destroyed,
            0.1,
        ),
        stat_row(
            bundle_value(locale, "stats.built"),
            context.stats.buildings_built,
            0.15,
        ),
        stat_row(
            bundle_value(locale, "stats.destroyed"),
            context.stats.buildings_destroyed,
            0.2,
        ),
        stat_row(
            bundle_value(locale, "stats.deconstructed"),
            context.stats.buildings_deconstructed,
            0.25,
        ),
    ]);
    rows
}

fn stat_row(label: String, value: i32, delay: f32) -> GameOverStatRow {
    GameOverStatRow {
        label,
        value,
        delay,
        height: 50.0,
    }
}

fn bundle_value(locale: &str, key: &str) -> String {
    upstream_menu_bundle_value_for_locale(locale, key)
        .unwrap_or(key)
        .to_string()
}

fn bundle_format(locale: &str, key: &str, args: &[&str]) -> String {
    upstream_menu_bundle_format_for_locale(locale, key, args)
        .unwrap_or_else(|| replace_placeholders(key, args))
}

fn replace_placeholders(text: &str, args: &[&str]) -> String {
    let mut value = text.to_string();
    for (index, arg) in args.iter().enumerate() {
        value = value.replace(&format!("{{{index}}}"), arg);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats() -> GameStats {
        GameStats {
            waves_lasted: 12,
            units_created: 3,
            enemy_units_destroyed: 4,
            buildings_built: 5,
            buildings_destroyed: 6,
            buildings_deconstructed: 7,
            ..GameStats::default()
        }
    }

    #[test]
    fn pvp_game_over_uses_winner_text_highscore_stats_and_menu_button() {
        let context = GameOverContext {
            winner_colored_name: Some("[scarlet]crux".into()),
            rules_pvp: true,
            rules_waves: true,
            high_score: true,
            current_save_playtime: Some("1:23".into()),
            stats: stats(),
            ..GameOverContext::default()
        };
        let model = GameOverDialog::new().rebuild(&context, "en");

        assert_eq!(
            model.header_text,
            "The[accent] [scarlet]crux[] team is victorious!"
        );
        assert!(model.high_score_visible);
        assert_eq!(model.stat_rows.len(), 6);
        assert_eq!(model.stat_rows[0].label, "Waves Defeated");
        assert_eq!(model.stat_rows[0].delay, 0.0);
        assert_eq!(model.playtime_label, Some("Time Played".into()));
        assert_eq!(model.playtime_value, Some("[accent]1:23".into()));
        assert_eq!(model.buttons[0].text, "@menu");
        assert_eq!(model.buttons[0].size, (140.0, 60.0));
    }

    #[test]
    fn campaign_client_waits_and_uses_disconnect_button() {
        let context = GameOverContext {
            is_campaign: true,
            sector_name: Some("Ground Zero".into()),
            net_client: true,
            stats: stats(),
            ..GameOverContext::default()
        };
        let model = GameOverDialog::new().rebuild(&context, "en");

        assert_eq!(model.header_text, "Sector [accent]Ground Zero[white] lost!");
        assert_eq!(model.waiting_label, Some("@gameover.waiting"));
        assert_eq!(model.buttons[0].text, "@gameover.disconnect");
        assert_eq!(
            GameOverDialog::disconnect_plan(),
            vec![
                GameOverDialogAction::ResetLogic,
                GameOverDialogAction::ResetNet,
                GameOverDialogAction::HideDialog,
                GameOverDialogAction::SetStateMenu,
            ]
        );
    }

    #[test]
    fn campaign_host_continue_can_schedule_difficulty_guide() {
        let context = GameOverContext {
            is_campaign: true,
            allow_campaign_rules: true,
            sector_attempts: 2,
            campaign_difficulty: Difficulty::Normal,
            difficulty_guide_once: true,
            mobile: false,
            ..GameOverContext::default()
        };
        let model = GameOverDialog::new().rebuild(&context, "en");
        assert_eq!(model.buttons[0].text, "@continue");
        assert_eq!(
            GameOverDialog::continue_plan(&context),
            vec![
                GameOverDialogAction::HideDialog,
                GameOverDialogAction::ShowPlanet,
                GameOverDialogAction::ShowDifficultyGuide { width: 500 },
            ]
        );
    }

    #[test]
    fn show_hidden_and_menu_actions_match_java_event_flow() {
        let context = GameOverContext {
            winner_is_player_team: true,
            ..GameOverContext::default()
        };
        assert_eq!(
            GameOverDialog::shown_actions(true),
            vec![
                GameOverDialogAction::StoreHudShown(true),
                GameOverDialogAction::SetHudShown(false),
            ]
        );
        assert_eq!(
            GameOverDialog::hidden_actions(true),
            vec![GameOverDialogAction::SetHudShown(true)]
        );
        assert_eq!(
            GameOverDialog::show_actions(&context),
            vec![
                GameOverDialogAction::SetAfterGameOver(true),
                GameOverDialogAction::FireWinEvent,
            ]
        );
        assert_eq!(
            GameOverDialog::menu_plan(false),
            vec![
                GameOverDialogAction::HideDialog,
                GameOverDialogAction::CheckPlaytest,
                GameOverDialogAction::ResetLogic,
            ]
        );
        assert_eq!(
            GameOverDialog::menu_plan(true),
            vec![
                GameOverDialogAction::HideDialog,
                GameOverDialogAction::CheckPlaytest,
            ]
        );
    }
}
