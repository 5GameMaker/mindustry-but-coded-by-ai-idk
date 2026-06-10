//! Network menu and notification action model mirroring upstream `mindustry.ui.Menus`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuRequest {
    pub menu_id: i32,
    pub title: String,
    pub message: String,
    pub options: Vec<Vec<String>>,
    pub follow_up: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputRequest {
    pub text_input_id: i32,
    pub title: String,
    pub message: String,
    pub text_length: i32,
    pub default_text: String,
    pub numeric: bool,
    pub allow_empty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenusAction {
    ShowMenu(MenuRequest),
    HideFollowUpMenu(i32),
    MenuChooseEvent {
        player_id: i32,
        menu_id: i32,
        option: i32,
        listener_called: bool,
    },
    ShowTextInput(TextInputRequest),
    TextInputEvent {
        player_id: i32,
        text_input_id: i32,
        text: Option<String>,
        listener_called: bool,
    },
    SetHudText(String),
    HideHudText,
    Announce(String),
    InfoMessage(String),
    InfoPopup {
        message: Option<String>,
        id: Option<String>,
        duration: f32,
        align: i32,
        top: i32,
        left: i32,
        bottom: i32,
        right: i32,
    },
    Label {
        message: Option<String>,
        id: i32,
        duration: f32,
        world_x: f32,
        world_y: f32,
    },
    InfoToast {
        message: String,
        duration: f32,
    },
    WarningToast {
        unicode: i32,
        text: String,
    },
    OpenUriConfirm(String),
    CopyToClipboardConfirm(String),
    RemoveWorldLabel(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Menus {
    menu_listeners: usize,
    text_input_listeners: usize,
}

impl Menus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn menu_listener_count(&self) -> usize {
        self.menu_listeners
    }

    pub fn text_input_listener_count(&self) -> usize {
        self.text_input_listeners
    }

    pub fn register_menu(&mut self) -> i32 {
        let id = self.menu_listeners as i32;
        self.menu_listeners += 1;
        id
    }

    pub fn register_text_input(&mut self) -> i32 {
        let id = self.text_input_listeners as i32;
        self.text_input_listeners += 1;
        id
    }

    pub fn menu(
        &self,
        menu_id: i32,
        title: Option<&str>,
        message: Option<&str>,
        options: Option<Vec<Vec<String>>>,
    ) -> MenusAction {
        MenusAction::ShowMenu(MenuRequest {
            menu_id,
            title: title.unwrap_or("").to_string(),
            message: message.unwrap_or("").to_string(),
            options: options.unwrap_or_default(),
            follow_up: false,
        })
    }

    pub fn follow_up_menu(
        &self,
        menu_id: i32,
        title: Option<&str>,
        message: Option<&str>,
        options: Option<Vec<Vec<String>>>,
    ) -> MenusAction {
        MenusAction::ShowMenu(MenuRequest {
            menu_id,
            title: title.unwrap_or("").to_string(),
            message: message.unwrap_or("").to_string(),
            options: options.unwrap_or_default(),
            follow_up: true,
        })
    }

    pub fn hide_follow_up_menu(&self, menu_id: i32) -> MenusAction {
        MenusAction::HideFollowUpMenu(menu_id)
    }

    pub fn menu_choose(
        &self,
        player_id: Option<i32>,
        menu_id: i32,
        option: i32,
    ) -> Option<MenusAction> {
        player_id.map(|player_id| MenusAction::MenuChooseEvent {
            player_id,
            menu_id,
            option,
            listener_called: menu_id >= 0 && (menu_id as usize) < self.menu_listeners,
        })
    }

    pub fn text_input(
        &self,
        text_input_id: i32,
        title: Option<&str>,
        message: Option<&str>,
        text_length: i32,
        default_text: Option<&str>,
        numeric: bool,
        allow_empty: bool,
    ) -> MenusAction {
        MenusAction::ShowTextInput(TextInputRequest {
            text_input_id,
            title: title.unwrap_or("").to_string(),
            message: message.unwrap_or("").to_string(),
            text_length,
            default_text: default_text.unwrap_or("").to_string(),
            numeric,
            allow_empty,
        })
    }

    pub fn text_input_result(
        &self,
        player_id: Option<i32>,
        text_input_id: i32,
        text: Option<String>,
    ) -> Option<MenusAction> {
        player_id.map(|player_id| MenusAction::TextInputEvent {
            player_id,
            text_input_id,
            text,
            listener_called: text_input_id >= 0
                && (text_input_id as usize) < self.text_input_listeners,
        })
    }

    pub fn set_hud_text(message: Option<&str>) -> Option<MenusAction> {
        message.map(|message| MenusAction::SetHudText(message.to_string()))
    }

    pub fn hide_hud_text() -> MenusAction {
        MenusAction::HideHudText
    }

    pub fn announce(message: Option<&str>) -> Option<MenusAction> {
        message.map(|message| MenusAction::Announce(message.to_string()))
    }

    pub fn info_message(message: Option<&str>) -> Option<MenusAction> {
        message.map(|message| MenusAction::InfoMessage(message.to_string()))
    }

    pub fn info_popup(
        message: Option<&str>,
        id: Option<&str>,
        duration: f32,
        align: i32,
        top: i32,
        left: i32,
        bottom: i32,
        right: i32,
    ) -> MenusAction {
        MenusAction::InfoPopup {
            message: message.map(str::to_string),
            id: id.map(str::to_string),
            duration,
            align,
            top,
            left,
            bottom,
            right,
        }
    }

    pub fn label(
        message: Option<&str>,
        id: i32,
        duration: f32,
        world_x: f32,
        world_y: f32,
    ) -> MenusAction {
        MenusAction::Label {
            message: message.map(str::to_string),
            id,
            duration,
            world_x,
            world_y,
        }
    }

    pub fn info_toast(message: Option<&str>, duration: f32) -> Option<MenusAction> {
        message.map(|message| MenusAction::InfoToast {
            message: message.to_string(),
            duration,
        })
    }

    pub fn warning_toast(
        unicode: i32,
        text: Option<&str>,
        glyph_exists: bool,
    ) -> Option<MenusAction> {
        if glyph_exists {
            text.map(|text| MenusAction::WarningToast {
                unicode,
                text: text.to_string(),
            })
        } else {
            None
        }
    }

    pub fn open_uri(uri: Option<&str>) -> Option<MenusAction> {
        uri.map(|uri| MenusAction::OpenUriConfirm(uri.to_string()))
    }

    pub fn copy_to_clipboard(text: Option<&str>) -> Option<MenusAction> {
        text.map(|text| MenusAction::CopyToClipboardConfirm(text.to_string()))
    }

    pub fn remove_world_label(id: i32) -> MenusAction {
        MenusAction::RemoveWorldLabel(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_menu_and_text_input_return_java_seq_indices() {
        let mut menus = Menus::new();

        assert_eq!(menus.register_menu(), 0);
        assert_eq!(menus.register_menu(), 1);
        assert_eq!(menus.register_text_input(), 0);
        assert_eq!(menus.menu_listener_count(), 2);
        assert_eq!(menus.text_input_listener_count(), 1);
    }

    #[test]
    fn menu_and_follow_up_menu_normalize_nulls_like_java_remote_methods() {
        let menus = Menus::new();

        let MenusAction::ShowMenu(menu) = menus.menu(7, None, None, None) else {
            panic!("expected menu action");
        };
        assert_eq!(menu.menu_id, 7);
        assert_eq!(menu.title, "");
        assert_eq!(menu.message, "");
        assert!(menu.options.is_empty());
        assert!(!menu.follow_up);

        let MenusAction::ShowMenu(menu) =
            menus.follow_up_menu(8, Some("title"), Some("msg"), Some(vec![vec!["ok".into()]]))
        else {
            panic!("expected menu action");
        };
        assert!(menu.follow_up);
        assert_eq!(menu.options[0][0], "ok");
    }

    #[test]
    fn menu_choose_and_text_input_result_fire_only_for_present_player() {
        let mut menus = Menus::new();
        let menu_id = menus.register_menu();
        let text_id = menus.register_text_input();

        assert_eq!(menus.menu_choose(None, menu_id, 2), None);
        assert_eq!(
            menus.menu_choose(Some(3), menu_id, 2),
            Some(MenusAction::MenuChooseEvent {
                player_id: 3,
                menu_id,
                option: 2,
                listener_called: true,
            })
        );
        assert_eq!(
            menus.text_input_result(Some(4), text_id, Some("abc".into())),
            Some(MenusAction::TextInputEvent {
                player_id: 4,
                text_input_id: text_id,
                text: Some("abc".into()),
                listener_called: true,
            })
        );
    }

    #[test]
    fn notification_helpers_ignore_null_messages_like_java() {
        assert_eq!(Menus::set_hud_text(None), None);
        assert_eq!(Menus::announce(None), None);
        assert_eq!(Menus::info_message(None), None);
        assert_eq!(Menus::info_toast(None, 1.0), None);
        assert_eq!(Menus::open_uri(None), None);
        assert_eq!(Menus::copy_to_clipboard(None), None);
        assert_eq!(Menus::warning_toast(42, Some("x"), false), None);

        assert_eq!(
            Menus::set_hud_text(Some("hi")),
            Some(MenusAction::SetHudText("hi".into()))
        );
        assert_eq!(
            Menus::warning_toast(42, Some("x"), true),
            Some(MenusAction::WarningToast {
                unicode: 42,
                text: "x".into()
            })
        );
    }
}
