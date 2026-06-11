//! Pause menu dialog model mirroring upstream `mindustry.ui.dialogs.PausedDialog`.

pub const PAUSED_DIALOG_TITLE: &str = "@menu";
pub const PAUSED_DESKTOP_BUTTON_WIDTH: f32 = 220.0;
pub const PAUSED_DESKTOP_BUTTON_HEIGHT: f32 = 55.0;
pub const PAUSED_DESKTOP_BUTTON_PAD: f32 = 5.0;
pub const PAUSED_MOBILE_BUTTON_SIZE: f32 = 130.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseContext {
    pub mobile: bool,
    pub steam: bool,
    pub confirm_exit: bool,
    pub disable_save: bool,
    pub state_is_menu: bool,
    pub state_is_campaign: bool,
    pub state_is_editor: bool,
    pub state_game_over: bool,
    pub playtesting_map: Option<String>,
    pub rules_sector_present: bool,
    pub sector_preset_description: Option<String>,
    pub rules_allow_edit_rules: bool,
    pub net_active: bool,
    pub net_server: bool,
    pub net_client: bool,
    pub current_save_present: bool,
    pub current_save_autosave: bool,
}

impl Default for PauseContext {
    fn default() -> Self {
        Self {
            mobile: false,
            steam: false,
            confirm_exit: true,
            disable_save: false,
            state_is_menu: false,
            state_is_campaign: false,
            state_is_editor: false,
            state_game_over: false,
            playtesting_map: None,
            rules_sector_present: false,
            sector_preset_description: None,
            rules_allow_edit_rules: false,
            net_active: false,
            net_server: false,
            net_client: false,
            current_save_present: false,
            current_save_autosave: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PausedDialogButtonAction {
    Objective,
    Abandon,
    Back,
    Settings,
    SaveGame,
    LoadGame,
    Host,
    InviteFriends,
    EditorWorldProcessors,
    Research,
    PlanetMap,
    Database,
    Quit,
    CustomizeRules,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PausedDialogButton {
    pub text: &'static str,
    pub icon_name: &'static str,
    pub action: PausedDialogButtonAction,
    pub disabled: bool,
    pub visible: bool,
    pub width: f32,
    pub height: f32,
    pub colspan: usize,
    pub name: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PausedDialogModel {
    pub title: &'static str,
    pub should_pause: bool,
    pub mobile: bool,
    pub button_width: f32,
    pub button_height: f32,
    pub button_pad: f32,
    pub customize_visible: bool,
    pub rows: Vec<Vec<PausedDialogButton>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PausedDialogAction {
    Hide,
    ConfirmQuit {
        title: &'static str,
        text: &'static str,
    },
    DisconnectQuietly,
    ResumeEditing,
    ResumeAfterPlaytest {
        map: String,
    },
    ResetLogic,
    SaveCurrentThenReset {
        loading_text: &'static str,
    },
    SaveCampaignStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PausedDialog;

impl PausedDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn update_should_hide(state_is_menu: bool, is_shown: bool) -> bool {
        state_is_menu && is_shown
    }

    pub fn rebuild(&self, context: &PauseContext) -> PausedDialogModel {
        if context.mobile {
            self.rebuild_mobile(context)
        } else {
            self.rebuild_desktop(context)
        }
    }

    pub fn shown_effects(context: &PauseContext) -> Vec<PausedDialogAction> {
        if context.state_is_campaign {
            vec![PausedDialogAction::SaveCampaignStats]
        } else {
            Vec::new()
        }
    }

    pub fn show_quit_confirm(context: &PauseContext) -> Vec<PausedDialogAction> {
        if context.confirm_exit {
            vec![PausedDialogAction::ConfirmQuit {
                title: "@confirm",
                text: "@quit.confirm",
            }]
        } else {
            Self::run_exit_save(context)
        }
    }

    pub fn run_exit_save(context: &PauseContext) -> Vec<PausedDialogAction> {
        let mut actions = Vec::new();
        let was_client = context.net_client;
        if context.net_client {
            actions.push(PausedDialogAction::DisconnectQuietly);
        }

        if context.state_is_editor && !was_client {
            actions.push(PausedDialogAction::ResumeEditing);
            return actions;
        }

        if let Some(map) = &context.playtesting_map {
            actions.push(PausedDialogAction::ResumeAfterPlaytest { map: map.clone() });
            return actions;
        }

        if !context.current_save_present
            || !context.current_save_autosave
            || was_client
            || context.state_game_over
            || context.disable_save
        {
            actions.push(PausedDialogAction::ResetLogic);
            return actions;
        }

        actions.push(PausedDialogAction::SaveCurrentThenReset {
            loading_text: "@saving",
        });
        actions
    }

    fn rebuild_desktop(&self, context: &PauseContext) -> PausedDialogModel {
        let dw = PAUSED_DESKTOP_BUTTON_WIDTH;
        let mut rows = Vec::new();
        let show_objective =
            context.rules_sector_present && context.sector_preset_description.is_some();

        if show_objective || context.rules_sector_present {
            let mut row = Vec::new();
            if show_objective {
                row.push(desktop_button(
                    "@objective",
                    "info",
                    PausedDialogButtonAction::Objective,
                ));
            }
            row.push(PausedDialogButton {
                text: "@abandon",
                icon_name: "cancel",
                action: PausedDialogButtonAction::Abandon,
                disabled: context.net_client || context.state_game_over,
                visible: context.rules_sector_present,
                width: if show_objective { dw } else { dw * 2.0 + 10.0 },
                height: PAUSED_DESKTOP_BUTTON_HEIGHT,
                colspan: if show_objective { 1 } else { 2 },
                name: None,
            });
            rows.push(row);
        }

        rows.push(vec![
            desktop_named_button("@back", "left", PausedDialogButtonAction::Back, "back"),
            desktop_named_button(
                "@settings",
                "settings",
                PausedDialogButtonAction::Settings,
                "settings",
            ),
        ]);

        if !context.state_is_campaign && !context.state_is_editor {
            rows.push(vec![
                desktop_button("@savegame", "save", PausedDialogButtonAction::SaveGame),
                PausedDialogButton {
                    disabled: context.net_active,
                    ..desktop_button("@loadgame", "upload", PausedDialogButtonAction::LoadGame)
                },
            ]);
        }

        let host_text = if context.net_server && context.steam {
            "@invitefriends"
        } else if context.state_is_editor {
            "@hostserver.mobile"
        } else {
            "@hostserver"
        };
        let mut host_row = vec![PausedDialogButton {
            text: host_text,
            icon_name: "host",
            action: if context.net_server && context.steam {
                PausedDialogButtonAction::InviteFriends
            } else {
                PausedDialogButtonAction::Host
            },
            disabled: !((context.steam && context.net_server) || !context.net_active),
            visible: true,
            width: if context.state_is_editor {
                dw
            } else {
                dw * 2.0 + 10.0
            },
            height: PAUSED_DESKTOP_BUTTON_HEIGHT,
            colspan: if context.state_is_editor { 1 } else { 2 },
            name: None,
        }];
        if context.state_is_editor {
            host_row.push(desktop_button(
                "@editor.worldprocessors",
                "logic",
                PausedDialogButtonAction::EditorWorldProcessors,
            ));
        }
        rows.push(host_row);

        rows.push(vec![PausedDialogButton {
            text: quit_button_text(context),
            icon_name: "exit",
            action: PausedDialogButtonAction::Quit,
            disabled: false,
            visible: true,
            width: dw + 10.0,
            height: PAUSED_DESKTOP_BUTTON_HEIGHT,
            colspan: 2,
            name: None,
        }]);

        PausedDialogModel {
            title: PAUSED_DIALOG_TITLE,
            should_pause: true,
            mobile: false,
            button_width: PAUSED_DESKTOP_BUTTON_WIDTH,
            button_height: PAUSED_DESKTOP_BUTTON_HEIGHT,
            button_pad: PAUSED_DESKTOP_BUTTON_PAD,
            customize_visible: context.rules_allow_edit_rules
                && (context.net_server || !context.net_active),
            rows,
        }
    }

    fn rebuild_mobile(&self, context: &PauseContext) -> PausedDialogModel {
        let mut rows = vec![
            vec![mobile_button(
                "@back",
                "play",
                PausedDialogButtonAction::Back,
            )],
            vec![mobile_button(
                "@settings",
                "settings",
                PausedDialogButtonAction::Settings,
            )],
        ];

        if !context.state_is_campaign && !context.state_is_editor {
            rows.push(vec![mobile_button(
                "@save",
                "save",
                PausedDialogButtonAction::SaveGame,
            )]);
            rows.push(vec![PausedDialogButton {
                text: if context.net_active {
                    "@database"
                } else {
                    "@load"
                },
                icon_name: if context.net_active {
                    "book"
                } else {
                    "download"
                },
                action: if context.net_active {
                    PausedDialogButtonAction::Database
                } else {
                    PausedDialogButtonAction::LoadGame
                },
                ..mobile_button("@load", "download", PausedDialogButtonAction::LoadGame)
            }]);
        } else if context.state_is_campaign {
            rows.push(vec![mobile_button(
                "@research",
                "tree",
                PausedDialogButtonAction::Research,
            )]);
            rows.push(vec![mobile_button(
                "@planetmap",
                "map",
                PausedDialogButtonAction::PlanetMap,
            )]);
        } else {
            rows.push(Vec::new());
        }

        rows.push(vec![PausedDialogButton {
            disabled: context.net_active,
            ..mobile_button("@hostserver.mobile", "host", PausedDialogButtonAction::Host)
        }]);
        rows.push(vec![PausedDialogButton {
            text: quit_button_text(context),
            ..mobile_button("@quit", "exit", PausedDialogButtonAction::Quit)
        }]);

        PausedDialogModel {
            title: PAUSED_DIALOG_TITLE,
            should_pause: true,
            mobile: true,
            button_width: PAUSED_MOBILE_BUTTON_SIZE,
            button_height: PAUSED_MOBILE_BUTTON_SIZE,
            button_pad: PAUSED_DESKTOP_BUTTON_PAD,
            customize_visible: context.rules_allow_edit_rules
                && (context.net_server || !context.net_active),
            rows,
        }
    }
}

fn desktop_button(
    text: &'static str,
    icon_name: &'static str,
    action: PausedDialogButtonAction,
) -> PausedDialogButton {
    PausedDialogButton {
        text,
        icon_name,
        action,
        disabled: false,
        visible: true,
        width: PAUSED_DESKTOP_BUTTON_WIDTH,
        height: PAUSED_DESKTOP_BUTTON_HEIGHT,
        colspan: 1,
        name: None,
    }
}

fn desktop_named_button(
    text: &'static str,
    icon_name: &'static str,
    action: PausedDialogButtonAction,
    name: &'static str,
) -> PausedDialogButton {
    PausedDialogButton {
        name: Some(name),
        ..desktop_button(text, icon_name, action)
    }
}

fn mobile_button(
    text: &'static str,
    icon_name: &'static str,
    action: PausedDialogButtonAction,
) -> PausedDialogButton {
    PausedDialogButton {
        text,
        icon_name,
        action,
        disabled: false,
        visible: true,
        width: PAUSED_MOBILE_BUTTON_SIZE,
        height: PAUSED_MOBILE_BUTTON_SIZE,
        colspan: 1,
        name: None,
    }
}

fn quit_button_text(context: &PauseContext) -> &'static str {
    if context.current_save_present && context.current_save_autosave {
        "@save.quit"
    } else {
        "@quit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_pause_menu_matches_java_button_visibility_and_gating() {
        let ctx = PauseContext {
            rules_sector_present: true,
            sector_preset_description: Some("Objective".into()),
            rules_allow_edit_rules: true,
            current_save_present: true,
            current_save_autosave: true,
            ..PauseContext::default()
        };

        let model = PausedDialog::new().rebuild(&ctx);
        assert_eq!(model.title, "@menu");
        assert!(model.should_pause);
        assert!(model.customize_visible);
        assert_eq!(model.rows[0][0].text, "@objective");
        assert_eq!(model.rows[0][1].text, "@abandon");
        assert_eq!(model.rows[1][0].name, Some("back"));
        assert_eq!(model.rows[2][0].text, "@savegame");
        assert_eq!(model.rows[2][1].text, "@loadgame");
        assert!(!model.rows[2][1].disabled);
        assert_eq!(model.rows.last().unwrap()[0].text, "@save.quit");
    }

    #[test]
    fn shown_effects_save_campaign_stats_only_in_campaign_like_java() {
        let campaign = PauseContext {
            state_is_campaign: true,
            ..PauseContext::default()
        };
        assert_eq!(
            PausedDialog::shown_effects(&campaign),
            vec![PausedDialogAction::SaveCampaignStats]
        );

        assert!(PausedDialog::shown_effects(&PauseContext::default()).is_empty());
    }

    #[test]
    fn desktop_host_button_switches_to_invite_for_steam_server_and_editor_adds_processors() {
        let ctx = PauseContext {
            state_is_editor: true,
            steam: true,
            net_server: true,
            net_active: true,
            ..PauseContext::default()
        };
        let model = PausedDialog::new().rebuild(&ctx);
        let host_row = &model.rows[1];

        assert_eq!(host_row[0].text, "@invitefriends");
        assert_eq!(host_row[0].action, PausedDialogButtonAction::InviteFriends);
        assert!(!host_row[0].disabled);
        assert_eq!(host_row[1].text, "@editor.worldprocessors");
    }

    #[test]
    fn mobile_pause_menu_switches_load_to_database_when_network_active() {
        let ctx = PauseContext {
            mobile: true,
            net_active: true,
            ..PauseContext::default()
        };
        let model = PausedDialog::new().rebuild(&ctx);

        assert!(model.mobile);
        assert_eq!(model.rows[2][0].text, "@save");
        assert_eq!(model.rows[3][0].text, "@database");
        assert_eq!(model.rows[3][0].icon_name, "book");
        assert_eq!(model.rows[3][0].action, PausedDialogButtonAction::Database);
        assert!(model.rows[4][0].disabled);
    }

    #[test]
    fn mobile_campaign_menu_uses_research_and_planet_buttons() {
        let ctx = PauseContext {
            mobile: true,
            state_is_campaign: true,
            ..PauseContext::default()
        };
        let model = PausedDialog::new().rebuild(&ctx);

        assert_eq!(model.rows[2][0].text, "@research");
        assert_eq!(model.rows[3][0].text, "@planetmap");
    }

    #[test]
    fn quit_confirm_and_exit_save_plan_match_java_branches() {
        let confirm = PauseContext {
            confirm_exit: true,
            ..PauseContext::default()
        };
        assert_eq!(
            PausedDialog::show_quit_confirm(&confirm),
            vec![PausedDialogAction::ConfirmQuit {
                title: "@confirm",
                text: "@quit.confirm",
            }]
        );

        let autosave = PauseContext {
            confirm_exit: false,
            current_save_present: true,
            current_save_autosave: true,
            ..PauseContext::default()
        };
        assert_eq!(
            PausedDialog::show_quit_confirm(&autosave),
            vec![PausedDialogAction::SaveCurrentThenReset {
                loading_text: "@saving"
            }]
        );

        let client = PauseContext {
            confirm_exit: false,
            net_client: true,
            current_save_present: true,
            current_save_autosave: true,
            ..PauseContext::default()
        };
        assert_eq!(
            PausedDialog::show_quit_confirm(&client),
            vec![
                PausedDialogAction::DisconnectQuietly,
                PausedDialogAction::ResetLogic,
            ]
        );
    }

    #[test]
    fn editor_and_playtest_exit_paths_skip_save_like_java() {
        let editor = PauseContext {
            state_is_editor: true,
            current_save_present: true,
            current_save_autosave: true,
            ..PauseContext::default()
        };
        assert_eq!(
            PausedDialog::run_exit_save(&editor),
            vec![PausedDialogAction::ResumeEditing]
        );

        let playtest = PauseContext {
            playtesting_map: Some("map.msav".into()),
            current_save_present: true,
            current_save_autosave: true,
            ..PauseContext::default()
        };
        assert_eq!(
            PausedDialog::run_exit_save(&playtest),
            vec![PausedDialogAction::ResumeAfterPlaytest {
                map: "map.msav".into()
            }]
        );
    }
}
