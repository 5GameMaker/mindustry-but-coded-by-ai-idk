//! Player list model mirroring upstream `mindustry.ui.fragments.PlayerListFragment`.

use crate::mindustry::ui::upstream_menu_bundle_format_for_locale;

pub const PLAYER_LIST_CONTENT_MARGIN_HORIZONTAL: f32 = 13.0;
pub const PLAYER_LIST_ROW_HEIGHT: f32 = 64.0;
pub const PLAYER_LIST_ICON_SIZE: f32 = 50.0;
pub const PLAYER_LIST_WIDTH: f32 = 350.0;
pub const PLAYER_LIST_MENU_BUTTON_HEIGHT: f32 = 50.0;
pub const PLAYER_LIST_DIALOG_MIN_WIDTH: f32 = 360.0;

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
    pub menu_button_visible: bool,
    pub votekick_button_visible: bool,
    pub can_switch_team: bool,
    pub can_admin_toggle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListModel {
    pub visible: bool,
    pub title: String,
    pub search_text: String,
    pub rows: Vec<PlayerListRow>,
    pub not_found: bool,
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
        for user in &sorted {
            if !user.has_connection && context.net_server && !user.local {
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

            rows.push(PlayerListRow {
                player_id: user.id,
                label: format!("[#{}]{}", user.color.to_uppercase(), user.name),
                admin_icon_visible: user.admin && !(!user.local && context.net_server),
                clickable,
                menu_button_visible,
                votekick_button_visible,
                can_switch_team: allow_team_switch,
                can_admin_toggle,
            });
        }

        PlayerListModel {
            visible: self.visible,
            title: format_players_title(players.len()),
            search_text: self.search_text.clone(),
            not_found: rows.is_empty(),
            rows,
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
    fn toggle_hides_and_clears_search_like_java() {
        let mut fragment = PlayerListFragment::new();
        fragment.set_search_text("abc");
        fragment.toggle(&[], &context());
        assert!(fragment.visible());
        assert!(fragment.toggle(&[], &context()).is_none());
        assert!(!fragment.visible());
        assert!(fragment.rebuild(&[], &context()).search_text.is_empty());
    }
}
