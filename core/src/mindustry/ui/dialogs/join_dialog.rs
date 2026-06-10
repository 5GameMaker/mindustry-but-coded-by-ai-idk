//! Join-game dialog model mirroring upstream `mindustry.ui.dialogs.JoinDialog`.

use crate::mindustry::{
    game::Gamemode,
    net::{Host, ServerGroup},
    ui::{upstream_menu_bundle_format_for_locale, upstream_menu_bundle_value_for_locale},
};

pub const JOIN_DIALOG_TITLE: &str = "@joingame";
pub const JOIN_DIALOG_ADD_TITLE: &str = "@joingame.title";
pub const JOIN_DIALOG_DEFAULT_PORT: i32 = 6567;
pub const JOIN_DIALOG_TARGET_WIDTH_MAX: f32 = 550.0;
pub const JOIN_DIALOG_SERVER_CARD_PAD: f32 = 4.0;
pub const JOIN_DIALOG_CONNECT_DELAY_FRAMES: f32 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct JoinServer {
    pub ip: String,
    pub port: i32,
    pub last_host: Option<Host>,
}

impl JoinServer {
    pub fn new(ip: impl Into<String>) -> Self {
        let mut server = Self {
            ip: String::new(),
            port: JOIN_DIALOG_DEFAULT_PORT,
            last_host: None,
        };
        server.set_ip(ip.into());
        server
    }

    pub fn set_ip(&mut self, ip: impl Into<String>) {
        let ip = ip.into();
        let is_ipv6 = ip.chars().filter(|ch| *ch == ':').count() > 1;
        if is_ipv6 && ip.rfind("]:").is_some_and(|idx| idx != ip.len() - 2) {
            let idx = ip.find("]:").unwrap();
            self.ip = ip[1..idx].to_string();
            self.port = ip[idx + 2..].parse().unwrap_or(JOIN_DIALOG_DEFAULT_PORT);
        } else if !is_ipv6 && ip.rfind(':').is_some_and(|idx| idx != ip.len() - 1) {
            let idx = ip.rfind(':').unwrap();
            self.ip = ip[..idx].to_string();
            self.port = ip[idx + 1..].parse().unwrap_or(JOIN_DIALOG_DEFAULT_PORT);
        } else {
            self.ip = ip;
            self.port = JOIN_DIALOG_DEFAULT_PORT;
        }
    }

    pub fn display_ip(&self) -> String {
        if self.ip.chars().filter(|ch| *ch == ':').count() > 1 {
            if self.port != JOIN_DIALOG_DEFAULT_PORT {
                format!("[{}]:{}", self.ip, self.port)
            } else {
                self.ip.clone()
            }
        } else if self.port != JOIN_DIALOG_DEFAULT_PORT {
            format!("{}:{}", self.ip, self.port)
        } else {
            self.ip.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinRemoteStatus {
    Empty,
    Refreshing,
    Invalid,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinRemoteServerRow {
    pub index: usize,
    pub display_ip: String,
    pub status: JoinRemoteStatus,
    pub can_move_up: bool,
    pub can_move_down: bool,
    pub delete_confirm_title: &'static str,
    pub delete_confirm_text: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinHostView {
    pub name_line: String,
    pub version_string: String,
    pub description: Option<String>,
    pub players_line: String,
    pub map_mode_line: String,
    pub ping_line: Option<String>,
    pub banned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinCommunityGroupView {
    pub name: String,
    pub hidden: bool,
    pub favorite: bool,
    pub prioritized: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinDialogModel {
    pub title: &'static str,
    pub close_button_width: i32,
    pub add_button_text: &'static str,
    pub info_button_visible: bool,
    pub name: String,
    pub target_width: i32,
    pub columns: usize,
    pub show_hidden: bool,
    pub server_search: String,
    pub remote_rows: Vec<JoinRemoteServerRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinKickReason {
    ClientOutdated,
    ServerOutdated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinDialogAction {
    RefreshAll,
    RefreshLocal,
    RefreshRemote,
    RefreshCommunity,
    PingServer {
        ip: String,
        port: i32,
    },
    SaveServers,
    SetupRemote,
    AddDialog {
        title: &'static str,
    },
    EditDialog {
        index: usize,
        ip: String,
    },
    DeleteConfirm {
        index: usize,
        title: &'static str,
        text: &'static str,
    },
    MoveRemote {
        from: usize,
        to: usize,
    },
    AddRemote {
        ip: String,
        port: i32,
    },
    ToggleShowHidden(bool),
    ToggleGroupHidden {
        name: String,
        hidden: bool,
    },
    ToggleGroupFavorite {
        name: String,
        favorite: bool,
    },
    ShowInfo(&'static str),
    ShowConnecting,
    ShowReconnecting,
    SetLoadCancelButton,
    Delay(i32),
    ResetLogic,
    ResetNet,
    BeginConnecting,
    Connect {
        ip: String,
        port: i32,
    },
    HideJoinDialog,
    HideAddDialog,
    ShowVersionMismatch {
        reason: JoinKickReason,
        text: String,
    },
    ShowServerDisclaimer {
        ip: String,
        port: i32,
        version: i32,
    },
    StoreServerDisclaimer(bool),
    ScheduleReconnectPing {
        ip: String,
        port: i32,
    },
    CancelReconnectPing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinDialog {
    servers: Vec<JoinServer>,
    pub renaming: Option<usize>,
    pub show_hidden: bool,
    pub server_search: String,
    pub refreshes: i32,
    pub last_ip: Option<String>,
    pub last_port: i32,
}

impl Default for JoinDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl JoinDialog {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            renaming: None,
            show_hidden: false,
            server_search: String::new(),
            refreshes: 0,
            last_ip: None,
            last_port: JOIN_DIALOG_DEFAULT_PORT,
        }
    }

    pub fn with_servers(servers: Vec<JoinServer>) -> Self {
        Self {
            servers,
            ..Self::new()
        }
    }

    pub fn servers(&self) -> &[JoinServer] {
        &self.servers
    }

    pub fn model(
        &self,
        name: impl Into<String>,
        steam: bool,
        mobile: bool,
        graphics_width: f32,
        scale: f32,
    ) -> JoinDialogModel {
        JoinDialogModel {
            title: JOIN_DIALOG_TITLE,
            close_button_width: if mobile { 190 } else { 210 },
            add_button_text: "@server.add",
            info_button_visible: !steam && !mobile,
            name: name.into(),
            target_width: target_width(graphics_width, scale) as i32,
            columns: columns(graphics_width, scale),
            show_hidden: self.show_hidden,
            server_search: self.server_search.clone(),
            remote_rows: self.remote_rows(),
        }
    }

    pub fn shown_actions(&mut self, steam: bool, join_info_seen: bool) -> Vec<JoinDialogAction> {
        let mut actions = vec![JoinDialogAction::RefreshAll];
        self.refreshes += 1;
        if !steam && !join_info_seen {
            actions.push(JoinDialogAction::ShowInfo("@join.info"));
        }
        actions
    }

    pub fn refresh_all_actions(&mut self, community_enabled: bool) -> Vec<JoinDialogAction> {
        self.refreshes += 1;
        let mut actions = vec![
            JoinDialogAction::RefreshAll,
            JoinDialogAction::RefreshLocal,
            JoinDialogAction::RefreshRemote,
        ];
        if community_enabled {
            actions.push(JoinDialogAction::RefreshCommunity);
        }
        actions
    }

    pub fn set_server_search(&mut self, text: impl Into<String>) {
        self.server_search = text
            .into()
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
    }

    pub fn toggle_show_hidden(&mut self) -> JoinDialogAction {
        self.show_hidden = !self.show_hidden;
        JoinDialogAction::ToggleShowHidden(self.show_hidden)
    }

    pub fn begin_add_server(&mut self) -> JoinDialogAction {
        self.renaming = None;
        JoinDialogAction::AddDialog {
            title: "@server.add",
        }
    }

    pub fn begin_edit_server(&mut self, index: usize) -> JoinDialogAction {
        self.renaming = Some(index);
        JoinDialogAction::EditDialog {
            index,
            ip: self.servers[index].display_ip(),
        }
    }

    pub fn confirm_add_or_edit(&mut self, ip: impl Into<String>) -> Vec<JoinDialogAction> {
        let mut server = JoinServer::new(ip.into());
        let action = if let Some(index) = self.renaming.take() {
            self.servers[index].set_ip(server.display_ip());
            JoinDialogAction::EditDialog {
                index,
                ip: self.servers[index].display_ip(),
            }
        } else {
            server.last_host = None;
            let action = JoinDialogAction::AddRemote {
                ip: server.ip.clone(),
                port: server.port,
            };
            self.servers.push(server);
            action
        };
        vec![
            action,
            JoinDialogAction::SaveServers,
            JoinDialogAction::SetupRemote,
            JoinDialogAction::RefreshRemote,
            JoinDialogAction::HideAddDialog,
        ]
    }

    pub fn delete_server_action(index: usize) -> JoinDialogAction {
        JoinDialogAction::DeleteConfirm {
            index,
            title: "@confirm",
            text: "@server.delete",
        }
    }

    pub fn remove_server(&mut self, index: usize) -> Vec<JoinDialogAction> {
        self.servers.remove(index);
        vec![
            JoinDialogAction::SaveServers,
            JoinDialogAction::SetupRemote,
            JoinDialogAction::RefreshRemote,
        ]
    }

    pub fn move_remote(&mut self, index: usize, sign: i32) -> Vec<JoinDialogAction> {
        let target = index as i32 + sign;
        if target < 0 || target > self.servers.len() as i32 - 1 {
            return Vec::new();
        }
        let server = self.servers.remove(index);
        let target = target as usize;
        self.servers.insert(target, server);
        vec![
            JoinDialogAction::MoveRemote {
                from: index,
                to: target,
            },
            JoinDialogAction::SaveServers,
            JoinDialogAction::SetupRemote,
        ]
    }

    pub fn remote_rows(&self) -> Vec<JoinRemoteServerRow> {
        let last = self.servers.len().saturating_sub(1);
        self.servers
            .iter()
            .enumerate()
            .map(|(index, server)| JoinRemoteServerRow {
                index,
                display_ip: server.display_ip(),
                status: if server.last_host.is_some() {
                    JoinRemoteStatus::Ready
                } else {
                    JoinRemoteStatus::Empty
                },
                can_move_up: index > 0,
                can_move_down: index < last,
                delete_confirm_title: "@confirm",
                delete_confirm_text: "@server.delete",
            })
            .collect()
    }

    pub fn connect_plan(
        &mut self,
        player_name: &str,
        ip: impl Into<String>,
        port: i32,
    ) -> Vec<JoinDialogAction> {
        let ip = ip.into();
        if player_name.trim().is_empty() {
            return vec![JoinDialogAction::ShowInfo("@noname")];
        }

        self.last_ip = Some(ip.clone());
        self.last_port = port;
        vec![
            JoinDialogAction::ShowConnecting,
            JoinDialogAction::SetLoadCancelButton,
            JoinDialogAction::Delay(JOIN_DIALOG_CONNECT_DELAY_FRAMES as i32),
            JoinDialogAction::ResetLogic,
            JoinDialogAction::ResetNet,
            JoinDialogAction::BeginConnecting,
            JoinDialogAction::Connect { ip, port },
        ]
    }

    pub fn safe_connect_plan(
        &mut self,
        player_name: &str,
        ip: impl Into<String>,
        port: i32,
        server_version: i32,
        build: i32,
        locale: &str,
    ) -> Vec<JoinDialogAction> {
        let ip = ip.into();
        if server_version != build && build != -1 && server_version != -1 {
            let reason = if server_version > build {
                JoinKickReason::ClientOutdated
            } else {
                JoinKickReason::ServerOutdated
            };
            return vec![JoinDialogAction::ShowVersionMismatch {
                reason,
                text: bundle_format(
                    locale,
                    "server.versions",
                    &[&build.to_string(), &server_version.to_string()],
                ),
            }];
        }
        self.connect_plan(player_name, ip, port)
    }

    pub fn community_click_plan(
        &mut self,
        player_name: &str,
        host: &Host,
        disclaimer_seen: bool,
        build: i32,
        locale: &str,
    ) -> Vec<JoinDialogAction> {
        if !disclaimer_seen {
            return vec![JoinDialogAction::ShowServerDisclaimer {
                ip: host.address.clone(),
                port: host.port,
                version: host.version,
            }];
        }
        self.safe_connect_plan(
            player_name,
            host.address.clone(),
            host.port,
            host.version,
            build,
            locale,
        )
    }

    pub fn accept_disclaimer_plan(
        &mut self,
        player_name: &str,
        ip: impl Into<String>,
        port: i32,
        version: i32,
        build: i32,
        locale: &str,
    ) -> Vec<JoinDialogAction> {
        let mut actions = vec![JoinDialogAction::StoreServerDisclaimer(true)];
        actions.extend(self.safe_connect_plan(player_name, ip, port, version, build, locale));
        actions
    }

    pub fn reconnect_plan(&self) -> Vec<JoinDialogAction> {
        let Some(ip) = &self.last_ip else {
            return Vec::new();
        };
        if ip.is_empty() {
            return Vec::new();
        }
        vec![
            JoinDialogAction::ShowReconnecting,
            JoinDialogAction::ScheduleReconnectPing {
                ip: ip.clone(),
                port: self.last_port,
            },
            JoinDialogAction::SetLoadCancelButton,
        ]
    }

    pub fn cancel_reconnect_plan() -> Vec<JoinDialogAction> {
        vec![
            JoinDialogAction::CancelReconnectPing,
            JoinDialogAction::SetLoadCancelButton,
        ]
    }
}

pub fn target_width(graphics_width: f32, scale: f32) -> f32 {
    (graphics_width / scale * 0.9).min(JOIN_DIALOG_TARGET_WIDTH_MAX)
}

pub fn columns(graphics_width: f32, scale: f32) -> usize {
    ((graphics_width / scale * 0.9 / target_width(graphics_width, scale)) as usize).clamp(1, 4)
}

pub fn version_string(host: &Host, build: i32, version_type: &str, locale: &str) -> String {
    if host.version == -1 {
        bundle_format(
            locale,
            "server.version",
            &[&bundle_value(locale, "server.custombuild"), ""],
        )
    } else if host.version == 0 {
        bundle_value(locale, "server.outdated")
    } else if host.version < build && build != -1 {
        format!(
            "{}\n{}",
            bundle_value(locale, "server.outdated"),
            bundle_format(
                locale,
                "server.version",
                &[&host.version.to_string(), &host.version_type]
            )
        )
    } else if host.version > build && build != -1 {
        format!(
            "{}\n{}",
            bundle_value(locale, "server.outdated.client"),
            bundle_format(
                locale,
                "server.version",
                &[&host.version.to_string(), &host.version_type]
            )
        )
    } else if host.version == build && version_type == host.version_type {
        String::new()
    } else {
        bundle_format(
            locale,
            "server.version",
            &[&host.version.to_string(), &host.version_type],
        )
    }
}

pub fn host_view(
    host: &Host,
    local: bool,
    add_name: bool,
    build: i32,
    version_type: &str,
    locale: &str,
) -> JoinHostView {
    let banned = local && host.description == "[banned]";
    let mut version = version_string(host, build, version_type, locale);
    if banned {
        version.push_str("[red]\u{e817} [banned]");
    }
    let description = (!host.description.is_empty() && !banned)
        .then(|| limit_description_newlines(&host.description));
    JoinHostView {
        name_line: if add_name {
            format!("{}   {}", host.name, version).replace('\n', " ")
        } else {
            host.name.clone()
        },
        version_string: version,
        description,
        players_line: players_line(host.players, host.player_limit, locale),
        map_mode_line: format!(
            "[lightgray]{}[lightgray] / {}",
            bundle_format(locale, "save.map", &[&host.mapname]),
            host.mode_name
                .clone()
                .unwrap_or_else(|| mode_label(locale, host.mode))
        )
        .replace('\n', " "),
        ping_line: (host.ping > 0).then(|| format!("\u{f292} {}ms", host.ping)),
        banned,
    }
}

pub fn group_view(group: &ServerGroup, show_hidden: bool) -> JoinCommunityGroupView {
    JoinCommunityGroupView {
        name: group.name.clone(),
        hidden: group.hidden,
        favorite: group.favorite,
        prioritized: group.prioritized,
        visible: !group.hidden || show_hidden,
    }
}

pub fn host_matches_search(group: &ServerGroup, host: &Host, search: &str) -> bool {
    let search = search.trim().to_lowercase();
    if search.is_empty() {
        return true;
    }
    group.name.to_lowercase().contains(&search)
        || strip_colors(&host.name).to_lowercase().contains(&search)
        || strip_colors(&host.description)
            .to_lowercase()
            .contains(&search)
        || strip_colors(&host.mapname).to_lowercase().contains(&search)
        || host
            .mode_name
            .as_ref()
            .is_some_and(|mode| strip_colors(mode).to_lowercase().contains(&search))
}

fn players_line(players: i32, limit: i32, locale: &str) -> String {
    let key = if players == 1 && limit <= 0 {
        "players.single"
    } else {
        "players"
    };
    let value = format!(
        "{}{}{}",
        if players == 0 {
            "[lightgray]"
        } else {
            "[accent]"
        },
        players,
        if limit > 0 {
            format!("[lightgray]/[accent]{limit}")
        } else {
            String::new()
        }
    );
    format!(
        "[lightgray]{}",
        bundle_format(locale, key, &[&format!("{value}[lightgray]")])
    )
}

fn limit_description_newlines(description: &str) -> String {
    let mut count = 0;
    let mut result = String::with_capacity(description.len());
    for ch in description.chars() {
        if ch == '\n' {
            count += 1;
            if count < 3 {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    format!("[gray]{result}")
}

fn mode_label(locale: &str, mode: Gamemode) -> String {
    bundle_value(locale, &format!("mode.{}.name", mode.wire_name()))
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

    fn host(version: i32) -> Host {
        Host::new(
            42,
            "Server",
            "127.0.0.1",
            6567,
            "Map",
            1,
            2,
            version,
            "official",
            Gamemode::Survival,
            8,
            "line1\nline2\nline3\nline4",
            None,
        )
    }

    #[test]
    fn join_server_parses_ipv4_ipv6_and_display_ip_like_java() {
        assert_eq!(JoinServer::new("example.com").display_ip(), "example.com");
        assert_eq!(
            JoinServer::new("example.com:7000").display_ip(),
            "example.com:7000"
        );
        assert_eq!(JoinServer::new("::1").display_ip(), "::1");
        assert_eq!(JoinServer::new("[::1]:7000").display_ip(), "[::1]:7000");
    }

    #[test]
    fn target_width_and_columns_match_java_clamp() {
        assert_eq!(target_width(1920.0, 1.0), 550.0);
        assert_eq!(columns(1920.0, 1.0), 3);
        assert_eq!(target_width(400.0, 1.0), 360.0);
        assert_eq!(columns(400.0, 1.0), 1);
    }

    #[test]
    fn remote_server_crud_and_rows_match_join_dialog_buttons() {
        let mut dialog = JoinDialog::with_servers(vec![
            JoinServer::new("a:1"),
            JoinServer::new("b:2"),
            JoinServer::new("c:3"),
        ]);
        assert_eq!(dialog.remote_rows()[0].display_ip, "a:1");
        assert!(!dialog.remote_rows()[0].can_move_up);
        assert!(dialog.remote_rows()[0].can_move_down);

        assert_eq!(
            dialog.move_remote(1, -1),
            vec![
                JoinDialogAction::MoveRemote { from: 1, to: 0 },
                JoinDialogAction::SaveServers,
                JoinDialogAction::SetupRemote,
            ]
        );
        assert_eq!(dialog.servers()[0].display_ip(), "b:2");

        assert_eq!(
            JoinDialog::delete_server_action(0),
            JoinDialogAction::DeleteConfirm {
                index: 0,
                title: "@confirm",
                text: "@server.delete",
            }
        );
        dialog.remove_server(0);
        assert_eq!(dialog.servers().len(), 2);
    }

    #[test]
    fn connect_safe_connect_and_reconnect_actions_match_java_flow() {
        let mut dialog = JoinDialog::new();
        assert_eq!(
            dialog.connect_plan("", "127.0.0.1", 6567),
            vec![JoinDialogAction::ShowInfo("@noname")]
        );

        let plan = dialog.connect_plan("Player", "127.0.0.1", 6567);
        assert_eq!(
            plan,
            vec![
                JoinDialogAction::ShowConnecting,
                JoinDialogAction::SetLoadCancelButton,
                JoinDialogAction::Delay(2),
                JoinDialogAction::ResetLogic,
                JoinDialogAction::ResetNet,
                JoinDialogAction::BeginConnecting,
                JoinDialogAction::Connect {
                    ip: "127.0.0.1".into(),
                    port: 6567,
                },
            ]
        );
        assert_eq!(
            dialog.reconnect_plan(),
            vec![
                JoinDialogAction::ShowReconnecting,
                JoinDialogAction::ScheduleReconnectPing {
                    ip: "127.0.0.1".into(),
                    port: 6567,
                },
                JoinDialogAction::SetLoadCancelButton,
            ]
        );

        assert!(matches!(
            dialog.safe_connect_plan("Player", "127.0.0.1", 6567, 200, 157, "en")[0],
            JoinDialogAction::ShowVersionMismatch {
                reason: JoinKickReason::ClientOutdated,
                ..
            }
        ));
    }

    #[test]
    fn host_view_formats_version_description_players_and_search() {
        let host = host(157);
        let view = host_view(&host, false, true, 157, "official", "en");

        assert_eq!(view.version_string, "");
        assert_eq!(view.name_line, "Server   ");
        assert_eq!(
            view.description,
            Some("[gray]line1\nline2\nline3line4".into())
        );
        assert!(view.players_line.contains("[accent]2[lightgray]/[accent]8"));
        assert!(view.map_mode_line.contains("Map: Map"));
        assert_eq!(view.ping_line, Some("\u{f292} 42ms".into()));

        let group = ServerGroup::new("Group", vec!["127.0.0.1".into()], false);
        assert!(host_matches_search(&group, &host, "line2"));
        assert!(host_matches_search(&group, &host, "group"));
        assert!(!host_matches_search(&group, &host, "missing"));
    }

    #[test]
    fn version_string_matches_outdated_and_custom_branches() {
        let custom = host(-1);
        assert!(version_string(&custom, 157, "official", "en").contains("Custom Build"));
        let old = host(1);
        assert!(version_string(&old, 157, "official", "en").contains("Outdated Server"));
        let new = host(999);
        assert!(version_string(&new, 157, "official", "en").contains("Outdated Client"));
    }

    #[test]
    fn community_disclaimer_precedes_safe_connect_until_accepted() {
        let host = host(157);
        let mut dialog = JoinDialog::new();
        assert_eq!(
            dialog.community_click_plan("Player", &host, false, 157, "en"),
            vec![JoinDialogAction::ShowServerDisclaimer {
                ip: "127.0.0.1".into(),
                port: 6567,
                version: 157,
            }]
        );

        let accepted = dialog.accept_disclaimer_plan("Player", "127.0.0.1", 6567, 157, 157, "en");
        assert_eq!(accepted[0], JoinDialogAction::StoreServerDisclaimer(true));
        assert!(accepted.contains(&JoinDialogAction::Connect {
            ip: "127.0.0.1".into(),
            port: 6567,
        }));
    }
}
