//! Player list model mirroring upstream `mindustry.ui.fragments.PlayerListFragment`.

use crate::mindustry::ui::upstream_menu_bundle_format_for_locale;

pub const PLAYER_LIST_CONTENT_MARGIN_HORIZONTAL: f32 = 13.0;
pub const PLAYER_LIST_ROW_HEIGHT: f32 = 64.0;
pub const PLAYER_LIST_ICON_SIZE: f32 = 50.0;
pub const PLAYER_LIST_WIDTH: f32 = 350.0;
pub const PLAYER_LIST_MENU_BUTTON_HEIGHT: f32 = 50.0;
pub const PLAYER_LIST_DIALOG_MIN_WIDTH: f32 = 360.0;
pub const PLAYER_LIST_MENU_DIALOG_BUTTON_WIDTH: f32 = 220.0;
pub const PLAYER_LIST_MENU_DIALOG_BUTTON_HEIGHT: f32 = 55.0;
pub const PLAYER_LIST_TEAM_BUTTON_SIZE: f32 = 50.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListPlayer {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub stripped_name: String,
    pub color: String,
    pub team: i32,
    pub local: bool,
    pub admin: bool,
    pub dead: bool,
    pub has_connection: bool,
}

impl PlayerListPlayer {
    pub fn new(id: i32, name: impl Into<String>, team: i32) -> Self {
        let name = name.into();
        Self {
            id,
            uuid: format!("uuid-{id}"),
            stripped_name: strip_colors(&name),
            name,
            color: "ffffff".into(),
            team,
            local: false,
            admin: false,
            dead: false,
            has_connection: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListContext {
    pub net_active: bool,
    pub net_server: bool,
    pub net_client: bool,
    pub state_is_game: bool,
    pub campaign: bool,
    pub pvp: bool,
    pub infinite_resources: bool,
    pub fog: bool,
    pub local_player_id: i32,
    pub local_player_team: i32,
    pub local_player_admin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListRow {
    pub player_id: i32,
    pub label: String,
    pub admin_icon_visible: bool,
    pub clickable: bool,
    pub spectate_action: Option<PlayerListRowAction>,
    pub menu_button_visible: bool,
    pub menu_action: Option<PlayerListRowAction>,
    pub menu_model: Option<PlayerListPlayerMenuModel>,
    pub votekick_button_visible: bool,
    pub votekick_action: Option<PlayerListRowAction>,
    pub can_switch_team: bool,
    pub can_admin_toggle: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerListFooterButtonAction {
    ShowBans,
    ShowAdmins,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListFooterButtonModel {
    pub text: &'static str,
    pub action: PlayerListFooterButtonAction,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerListRowAction {
    Spectate { player_id: i32 },
    OpenMenu { player_id: i32 },
    StartVoteKick { player_id: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerListPlayerMenuAction {
    Ban {
        player_id: i32,
    },
    Kick {
        player_id: i32,
    },
    Trace {
        player_id: i32,
    },
    OpenTeamSelect {
        player_id: i32,
    },
    ToggleAdmin {
        player_id: i32,
        uuid: String,
        currently_admin: bool,
    },
    Back,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListPlayerMenuButtonModel {
    pub text: &'static str,
    pub icon: &'static str,
    pub action: PlayerListPlayerMenuAction,
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListPlayerMenuModel {
    pub player_id: i32,
    pub title: String,
    pub buttons: Vec<PlayerListPlayerMenuButtonModel>,
    pub back_button: PlayerListPlayerMenuButtonModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListModel {
    pub visible: bool,
    pub title: String,
    pub search_text: String,
    pub rows: Vec<PlayerListRow>,
    pub not_found: bool,
    pub footer_buttons: Vec<PlayerListFooterButtonModel>,
    pub show_bans_button: bool,
    pub show_admins_button: bool,
    pub close_button_text: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayerListFragment {
    visible: bool,
    search_text: String,
}

impl PlayerListFragment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&mut self) {
        self.visible = false;
    }

    pub fn rebuild(
        &self,
        players: &[PlayerListPlayer],
        context: &PlayerListContext,
    ) -> PlayerListModel {
        let allow_team_switch = !context.campaign && (context.pvp || context.infinite_resources);
        let mut sorted = players.to_vec();
        sorted.sort_by_key(|player| (player.team, !player.admin));
        if !self.search_text.is_empty() {
            let needle = self.search_text.to_lowercase();
            sorted.retain(|player| player.stripped_name.to_lowercase().contains(&needle));
        }

        let mut rows = Vec::new();
        let mut stopped_for_missing_server_connection = false;
        for user in &sorted {
            if !user.has_connection && context.net_server && !user.local {
                stopped_for_missing_server_connection = true;
                break;
            }

            let clickable = !(context.fog && context.pvp && user.team != context.local_player_team);
            let is_local = user.id == context.local_player_id;
            let menu_button_visible = (context.net_server
                || (context.local_player_admin && (!user.admin || is_local)))
                && (allow_team_switch || !is_local);
            let votekick_button_visible = !user.local
                && !user.admin
                && context.net_client
                && players.len() >= 3
                && context.local_player_team == user.team;
            let can_admin_toggle = !context.net_client && !user.local;
            let spectate_action = (!user.dead && clickable)
                .then_some(PlayerListRowAction::Spectate { player_id: user.id });
            let menu_action =
                menu_button_visible.then_some(PlayerListRowAction::OpenMenu { player_id: user.id });
            let votekick_action = votekick_button_visible
                .then_some(PlayerListRowAction::StartVoteKick { player_id: user.id });
            let menu_model = menu_button_visible.then(|| {
                PlayerListPlayerMenuModel::like_java(user, context, allow_team_switch, is_local)
            });

            rows.push(PlayerListRow {
                player_id: user.id,
                label: format!("[#{}]{}", user.color.to_uppercase(), user.name),
                admin_icon_visible: user.admin && !(!user.local && context.net_server),
                clickable,
                spectate_action,
                menu_button_visible,
                menu_action,
                menu_model,
                votekick_button_visible,
                votekick_action,
                can_switch_team: allow_team_switch,
                can_admin_toggle,
            });
        }

        PlayerListModel {
            visible: self.visible,
            title: format_players_title(players.len()),
            search_text: self.search_text.clone(),
            not_found: rows.is_empty() && !stopped_for_missing_server_connection,
            rows,
            footer_buttons: PlayerListFooterButtonModel::like_java(context),
            show_bans_button: !context.net_client,
            show_admins_button: !context.net_client,
            close_button_text: "@close",
        }
    }

    pub fn update_visibility(&mut self, context: &PlayerListContext) {
        if !(context.net_active && context.state_is_game) {
            self.visible = false;
        }
    }

    pub fn toggle(
        &mut self,
        players: &[PlayerListPlayer],
        context: &PlayerListContext,
    ) -> Option<PlayerListModel> {
        self.visible = !self.visible;
        if self.visible {
            Some(self.rebuild(players, context))
        } else {
            self.search_text.clear();
            None
        }
    }

    pub fn set_search_text(&mut self, text: impl Into<String>) {
        self.search_text = text.into();
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}

impl PlayerListFooterButtonModel {
    fn like_java(context: &PlayerListContext) -> Vec<Self> {
        vec![
            Self {
                text: "@server.bans",
                action: PlayerListFooterButtonAction::ShowBans,
                disabled: context.net_client,
            },
            Self {
                text: "@server.admins",
                action: PlayerListFooterButtonAction::ShowAdmins,
                disabled: context.net_client,
            },
            Self {
                text: "@close",
                action: PlayerListFooterButtonAction::Close,
                disabled: false,
            },
        ]
    }
}

impl PlayerListPlayerMenuModel {
    fn like_java(
        user: &PlayerListPlayer,
        context: &PlayerListContext,
        allow_team_switch: bool,
        is_local: bool,
    ) -> Self {
        let mut buttons = Vec::new();

        if !is_local {
            buttons.push(PlayerListPlayerMenuButtonModel {
                text: "@player.ban",
                icon: "hammer",
                action: PlayerListPlayerMenuAction::Ban { player_id: user.id },
                checked: false,
            });
            buttons.push(PlayerListPlayerMenuButtonModel {
                text: "@player.kick",
                icon: "cancel",
                action: PlayerListPlayerMenuAction::Kick { player_id: user.id },
                checked: false,
            });
            buttons.push(PlayerListPlayerMenuButtonModel {
                text: "@player.trace",
                icon: "zoom",
                action: PlayerListPlayerMenuAction::Trace { player_id: user.id },
                checked: false,
            });
        }

        if allow_team_switch {
            buttons.push(PlayerListPlayerMenuButtonModel {
                text: "@player.team",
                icon: "redo",
                action: PlayerListPlayerMenuAction::OpenTeamSelect { player_id: user.id },
                checked: false,
            });
        }

        if !context.net_client && !user.local {
            buttons.push(PlayerListPlayerMenuButtonModel {
                text: "@player.admin",
                icon: "admin",
                action: PlayerListPlayerMenuAction::ToggleAdmin {
                    player_id: user.id,
                    uuid: user.uuid.clone(),
                    currently_admin: user.admin,
                },
                checked: user.admin,
            });
        }

        Self {
            player_id: user.id,
            title: user.name.clone(),
            buttons,
            back_button: PlayerListPlayerMenuButtonModel {
                text: "@back",
                icon: "left",
                action: PlayerListPlayerMenuAction::Back,
                checked: false,
            },
        }
    }
}

fn format_players_title(count: usize) -> String {
    let key = if count == 1 {
        "players.single"
    } else {
        "players"
    };
    upstream_menu_bundle_format_for_locale("en", key, &[&count.to_string()])
        .unwrap_or_else(|| format!("{count} players"))
}

fn strip_colors(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            for next in chars.by_ref() {
                if next == ']' {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> PlayerListContext {
        PlayerListContext {
            net_active: true,
            net_server: true,
            net_client: false,
            state_is_game: true,
            campaign: false,
            pvp: true,
            infinite_resources: false,
            fog: false,
            local_player_id: 1,
            local_player_team: 1,
            local_player_admin: true,
        }
    }

    #[test]
    fn rebuild_sorts_by_team_and_admin_first_like_java_comparator() {
        let mut a = PlayerListPlayer::new(1, "local", 2);
        a.local = true;
        let mut b = PlayerListPlayer::new(2, "admin", 1);
        b.admin = true;
        let c = PlayerListPlayer::new(3, "user", 1);

        let model = PlayerListFragment::new().rebuild(&[a, c, b], &context());

        assert_eq!(model.rows[0].player_id, 2);
        assert_eq!(model.rows[1].player_id, 3);
        assert_eq!(model.rows[2].player_id, 1);
    }

    #[test]
    fn search_filters_stripped_lowercase_names() {
        let mut fragment = PlayerListFragment::new();
        fragment.set_search_text("pla");
        let players = vec![PlayerListPlayer::new(1, "[scarlet]Player", 1)];

        let model = fragment.rebuild(&players, &context());

        assert_eq!(model.rows.len(), 1);
    }

    #[test]
    fn server_missing_connection_returns_without_not_found_like_java() {
        let mut disconnected = PlayerListPlayer::new(2, "remote", 1);
        disconnected.local = false;
        disconnected.has_connection = false;
        let mut context = context();
        context.net_server = true;

        let model = PlayerListFragment::new().rebuild(&[disconnected], &context);

        assert!(model.rows.is_empty());
        assert!(!model.not_found);
    }

    #[test]
    fn toggle_hides_and_clears_search_like_java() {
        let mut fragment = PlayerListFragment::new();
        fragment.set_search_text("abc");
        fragment.toggle(&[], &context());
        assert!(fragment.visible());
        assert!(fragment.toggle(&[], &context()).is_none());
        assert!(!fragment.visible());
        assert!(fragment.rebuild(&[], &context()).search_text.is_empty());
    }

    #[test]
    fn footer_buttons_match_java_menu_order_and_client_disabled_state() {
        let players = vec![PlayerListPlayer::new(1, "local", 1)];
        let server_model = PlayerListFragment::new().rebuild(&players, &context());
        assert_eq!(
            server_model
                .footer_buttons
                .iter()
                .map(|button| (button.text, button.action, button.disabled))
                .collect::<Vec<_>>(),
            vec![
                (
                    "@server.bans",
                    PlayerListFooterButtonAction::ShowBans,
                    false
                ),
                (
                    "@server.admins",
                    PlayerListFooterButtonAction::ShowAdmins,
                    false
                ),
                ("@close", PlayerListFooterButtonAction::Close, false),
            ]
        );

        let mut client_context = context();
        client_context.net_server = false;
        client_context.net_client = true;
        let client_model = PlayerListFragment::new().rebuild(&players, &client_context);
        assert_eq!(
            client_model
                .footer_buttons
                .iter()
                .map(|button| (button.text, button.action, button.disabled))
                .collect::<Vec<_>>(),
            vec![
                ("@server.bans", PlayerListFooterButtonAction::ShowBans, true),
                (
                    "@server.admins",
                    PlayerListFooterButtonAction::ShowAdmins,
                    true
                ),
                ("@close", PlayerListFooterButtonAction::Close, false),
            ]
        );
    }

    #[test]
    fn remote_player_menu_buttons_follow_java_order_and_conditions() {
        let mut remote = PlayerListPlayer::new(2, "remote", 1);
        remote.uuid = "remote-uuid".into();
        let model = PlayerListFragment::new().rebuild(&[remote], &context());
        let row = &model.rows[0];

        assert_eq!(
            row.spectate_action,
            Some(PlayerListRowAction::Spectate { player_id: 2 })
        );
        assert_eq!(
            row.menu_action,
            Some(PlayerListRowAction::OpenMenu { player_id: 2 })
        );
        assert!(!row.votekick_button_visible);

        let menu = row
            .menu_model
            .as_ref()
            .expect("server/admin row should expose the Java menu dialog model");
        assert_eq!(menu.title, "remote");
        assert_eq!(
            menu.buttons
                .iter()
                .map(|button| (
                    button.text,
                    button.icon,
                    button.action.clone(),
                    button.checked
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "@player.ban",
                    "hammer",
                    PlayerListPlayerMenuAction::Ban { player_id: 2 },
                    false,
                ),
                (
                    "@player.kick",
                    "cancel",
                    PlayerListPlayerMenuAction::Kick { player_id: 2 },
                    false,
                ),
                (
                    "@player.trace",
                    "zoom",
                    PlayerListPlayerMenuAction::Trace { player_id: 2 },
                    false,
                ),
                (
                    "@player.team",
                    "redo",
                    PlayerListPlayerMenuAction::OpenTeamSelect { player_id: 2 },
                    false,
                ),
                (
                    "@player.admin",
                    "admin",
                    PlayerListPlayerMenuAction::ToggleAdmin {
                        player_id: 2,
                        uuid: "remote-uuid".into(),
                        currently_admin: false,
                    },
                    false,
                ),
            ]
        );
        assert_eq!(menu.back_button.text, "@back");
        assert_eq!(menu.back_button.icon, "left");
        assert_eq!(menu.back_button.action, PlayerListPlayerMenuAction::Back);
    }

    #[test]
    fn client_same_team_non_admin_row_exposes_votekick_action_like_java() {
        let local = {
            let mut player = PlayerListPlayer::new(1, "local", 1);
            player.local = true;
            player
        };
        let mut same_team = PlayerListPlayer::new(2, "same", 1);
        same_team.admin = false;
        let other = PlayerListPlayer::new(3, "other", 2);
        let mut context = context();
        context.net_server = false;
        context.net_client = true;
        context.local_player_admin = false;
        context.local_player_team = 1;

        let model = PlayerListFragment::new().rebuild(&[local, same_team, other], &context);
        let row = model
            .rows
            .iter()
            .find(|row| row.player_id == 2)
            .expect("same-team remote player should render");

        assert!(!row.menu_button_visible);
        assert_eq!(row.menu_model, None);
        assert!(row.votekick_button_visible);
        assert_eq!(
            row.votekick_action,
            Some(PlayerListRowAction::StartVoteKick { player_id: 2 })
        );
    }
}
