//! Host-game dialog model mirroring upstream `mindustry.ui.dialogs.HostDialog`.

pub const HOST_DIALOG_TITLE: &str = "@hostserver";
pub const HOST_DIALOG_WIDTH: f32 = 300.0;
pub const HOST_DIALOG_ROW_HEIGHT: f32 = 70.0;
pub const HOST_DIALOG_SIDE_BUTTON_WIDTH: f32 = 65.0;
pub const HOST_DIALOG_NAME_MAX_LENGTH: usize = 40;
pub const HOST_DIALOG_PORT_MAX_LENGTH: usize = 5;
pub const HOST_DIALOG_DEFAULT_PORT: i32 = 6567;
pub const HOST_DIALOG_DELAY_FRAMES: f32 = 5.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostDialogContext {
    pub steam: bool,
    pub name: String,
    pub color_rgba: u32,
    pub port: i32,
    pub steam_public_host: bool,
    pub host_info_seen: bool,
    pub version_modifier: String,
}

impl Default for HostDialogContext {
    fn default() -> Self {
        Self {
            steam: false,
            name: String::new(),
            color_rgba: 0xffffffff,
            port: HOST_DIALOG_DEFAULT_PORT,
            steam_public_host: false,
            host_info_seen: false,
            version_modifier: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostDialogModel {
    pub title: &'static str,
    pub width: i32,
    pub row_height: i32,
    pub name: String,
    pub name_max_length: usize,
    pub color_rgba: u32,
    pub steam_friends_only_visible: bool,
    pub steam_friends_only_checked: bool,
    pub port_text: String,
    pub port_valid: bool,
    pub port_max_length: usize,
    pub host_button_text: &'static str,
    pub host_button_disabled: bool,
    pub info_button_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostDialogAction {
    SetPlayerName(String),
    StoreName(String),
    RebuildPlayerList,
    OpenPalette,
    StoreColor(u32),
    StorePort(i32),
    SetSteamPublicHost(bool),
    ShowInfo(&'static str),
    ShowLoading(&'static str),
    Delay(i32),
    FetchSteamAdmins,
    NetHost(i32),
    SetPlayerAdmin(bool),
    FireHostEvent,
    DisableSteamPublicHost,
    UpdateLobby,
    ShowBetaPublicInfoOnce,
    HideLoading,
    HideDialog,
    ShowHostException(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HostDialog;

impl HostDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn model(&self, context: &HostDialogContext) -> HostDialogModel {
        HostDialogModel {
            title: HOST_DIALOG_TITLE,
            width: HOST_DIALOG_WIDTH as i32,
            row_height: HOST_DIALOG_ROW_HEIGHT as i32,
            name: context.name.clone(),
            name_max_length: HOST_DIALOG_NAME_MAX_LENGTH,
            color_rgba: context.color_rgba,
            steam_friends_only_visible: context.steam,
            steam_friends_only_checked: !context.steam_public_host,
            port_text: context.port.to_string(),
            port_valid: valid_port(context.port),
            port_max_length: HOST_DIALOG_PORT_MAX_LENGTH,
            host_button_text: "@host",
            host_button_disabled: !valid_port(context.port),
            info_button_visible: !context.steam,
        }
    }

    pub fn set_name_actions(name: impl Into<String>) -> Vec<HostDialogAction> {
        let name = name.into();
        vec![
            HostDialogAction::SetPlayerName(name.clone()),
            HostDialogAction::StoreName(name),
            HostDialogAction::RebuildPlayerList,
        ]
    }

    pub fn set_color_actions(color_rgba: u32) -> Vec<HostDialogAction> {
        vec![
            HostDialogAction::StoreColor(color_rgba),
            HostDialogAction::OpenPalette,
        ]
    }

    pub fn set_port_action(text: &str) -> HostDialogAction {
        HostDialogAction::StorePort(parse_port_or_default(text))
    }

    pub fn set_steam_friends_only_action(friends_only: bool) -> HostDialogAction {
        HostDialogAction::SetSteamPublicHost(!friends_only)
    }

    pub fn shown_action(context: &HostDialogContext) -> Option<HostDialogAction> {
        (!context.steam && !context.host_info_seen)
            .then_some(HostDialogAction::ShowInfo("@host.info"))
    }

    pub fn host_button_actions(context: &HostDialogContext) -> Vec<HostDialogAction> {
        if context.name.trim().is_empty() {
            return vec![HostDialogAction::ShowInfo("@noname")];
        }
        Self::run_host_plan(context)
    }

    pub fn run_host_plan(context: &HostDialogContext) -> Vec<HostDialogAction> {
        let mut actions = vec![
            HostDialogAction::ShowLoading("@hosting"),
            HostDialogAction::Delay(HOST_DIALOG_DELAY_FRAMES as i32),
        ];
        if context.steam {
            actions.push(HostDialogAction::FetchSteamAdmins);
        }
        actions.extend([
            HostDialogAction::NetHost(context.port),
            HostDialogAction::SetPlayerAdmin(true),
            HostDialogAction::FireHostEvent,
        ]);
        if context.steam
            && context.steam_public_host
            && (context.version_modifier.contains("beta")
                || context.version_modifier.contains("alpha"))
        {
            actions.extend([
                HostDialogAction::DisableSteamPublicHost,
                HostDialogAction::UpdateLobby,
                HostDialogAction::ShowBetaPublicInfoOnce,
            ]);
        }
        actions.extend([HostDialogAction::HideLoading, HostDialogAction::HideDialog]);
        actions
    }

    pub fn host_error_action(message: &str) -> HostDialogAction {
        if message.to_lowercase().contains("address already in use") {
            HostDialogAction::ShowHostException("@server.error.addressinuse")
        } else {
            HostDialogAction::ShowHostException("@server.error")
        }
    }
}

pub fn valid_port(port: i32) -> bool {
    (1..=65535).contains(&port)
}

pub fn parse_port_or_default(text: &str) -> i32 {
    text.parse::<i32>().unwrap_or(HOST_DIALOG_DEFAULT_PORT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_matches_java_field_sizes_and_steam_friend_toggle() {
        let ctx = HostDialogContext {
            steam: true,
            name: "Player".into(),
            color_rgba: 0x11223344,
            port: 6567,
            steam_public_host: false,
            ..HostDialogContext::default()
        };
        let model = HostDialog::new().model(&ctx);

        assert_eq!(model.title, "@hostserver");
        assert_eq!(model.width, 300);
        assert_eq!(model.row_height, 70);
        assert_eq!(model.name_max_length, 40);
        assert_eq!(model.port_max_length, 5);
        assert!(model.steam_friends_only_visible);
        assert!(model.steam_friends_only_checked);
        assert!(!model.info_button_visible);
        assert!(!model.host_button_disabled);
    }

    #[test]
    fn port_parsing_and_validation_match_text_field_contract() {
        assert_eq!(parse_port_or_default("1234"), 1234);
        assert_eq!(parse_port_or_default("abc"), 6567);
        assert!(valid_port(1));
        assert!(valid_port(65535));
        assert!(!valid_port(0));
        assert!(!valid_port(65536));
    }

    #[test]
    fn host_click_requires_non_empty_name_before_run_host() {
        let empty = HostDialogContext {
            name: "   ".into(),
            ..HostDialogContext::default()
        };
        assert_eq!(
            HostDialog::host_button_actions(&empty),
            vec![HostDialogAction::ShowInfo("@noname")]
        );

        let named = HostDialogContext {
            name: "Player".into(),
            port: 7000,
            ..HostDialogContext::default()
        };
        assert_eq!(
            HostDialog::host_button_actions(&named),
            vec![
                HostDialogAction::ShowLoading("@hosting"),
                HostDialogAction::Delay(5),
                HostDialogAction::NetHost(7000),
                HostDialogAction::SetPlayerAdmin(true),
                HostDialogAction::FireHostEvent,
                HostDialogAction::HideLoading,
                HostDialogAction::HideDialog,
            ]
        );
    }

    #[test]
    fn steam_beta_public_host_is_forced_private_like_java() {
        let ctx = HostDialogContext {
            steam: true,
            steam_public_host: true,
            name: "Player".into(),
            version_modifier: "beta".into(),
            ..HostDialogContext::default()
        };
        let plan = HostDialog::run_host_plan(&ctx);

        assert!(plan.contains(&HostDialogAction::FetchSteamAdmins));
        assert!(plan.contains(&HostDialogAction::DisableSteamPublicHost));
        assert!(plan.contains(&HostDialogAction::UpdateLobby));
        assert!(plan.contains(&HostDialogAction::ShowBetaPublicInfoOnce));
    }

    #[test]
    fn address_in_use_exception_uses_specific_bundle_key() {
        assert_eq!(
            HostDialog::host_error_action("Address already in use"),
            HostDialogAction::ShowHostException("@server.error.addressinuse")
        );
        assert_eq!(
            HostDialog::host_error_action("other"),
            HostDialogAction::ShowHostException("@server.error")
        );
    }
}
