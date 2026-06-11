//! Developer console HUD state model mirroring upstream `mindustry.ui.fragments.ConsoleFragment`.

pub const CONSOLE_MESSAGES_SHOWN: usize = 30;
pub const CONSOLE_MOBILE_BUTTON_SIZE: f32 = 58.0;
pub const CONSOLE_MOBILE_BUTTON_PAD_LEFT: f32 = 4.0;
pub const CONSOLE_INJECT_VARIABLES: &str =
    "var unit = Vars.player.unit();var player = Vars.player;var team = Vars.player.team();var core = Vars.player.core();var items = Vars.player.team().items();var build = Vars.world.buildWorld(Core.input.mouseWorldX(), Core.input.mouseWorldY());var cursor = Vars.world.tileWorld(Core.input.mouseWorldX(), Core.input.mouseWorldY());var cursorUnit = Units.closestEnemy(null, Core.input.mouseWorldX(), Core.input.mouseWorldY(), 70, u => true);\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleAction {
    FireOpenConsole,
    RequestKeyboard,
    ClearKeyboardFocus,
    ClearScrollFocus,
    OpenMobileTextInput,
    HideOnscreenKeyboard,
    OpenScriptFileChooser,
    ExecuteConsole(String),
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleMobileButtonKind {
    Chat,
    ScrollUp,
    ScrollDown,
    File,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleMobileButtonAction {
    OpenTextInput,
    ScrollUp,
    ScrollDown,
    OpenScriptFileChooser,
    Hide,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleMobileButtonModel {
    pub kind: ConsoleMobileButtonKind,
    pub icon: &'static str,
    pub style: &'static str,
    pub action: ConsoleMobileButtonAction,
    pub disabled: bool,
    pub size: f32,
    pub pad_left: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleMobileToolbarModel {
    pub open: bool,
    pub field_label: &'static str,
    pub messages_shown: usize,
    pub buttons: Vec<ConsoleMobileButtonModel>,
}

impl ConsoleMobileToolbarModel {
    pub fn like_java(
        shown: bool,
        open: bool,
        scroll_pos: usize,
        message_count: usize,
    ) -> Option<Self> {
        if !shown {
            return None;
        }

        Some(Self {
            open,
            field_label: ">",
            messages_shown: CONSOLE_MESSAGES_SHOWN,
            buttons: vec![
                ConsoleMobileButtonModel::new(
                    ConsoleMobileButtonKind::Chat,
                    "chat",
                    ConsoleMobileButtonAction::OpenTextInput,
                    false,
                    0.0,
                ),
                ConsoleMobileButtonModel::new(
                    ConsoleMobileButtonKind::ScrollUp,
                    "upOpen",
                    ConsoleMobileButtonAction::ScrollUp,
                    scroll_pos >= message_count,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
                ConsoleMobileButtonModel::new(
                    ConsoleMobileButtonKind::ScrollDown,
                    "downOpen",
                    ConsoleMobileButtonAction::ScrollDown,
                    scroll_pos <= 0,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
                ConsoleMobileButtonModel::new(
                    ConsoleMobileButtonKind::File,
                    "fileText",
                    ConsoleMobileButtonAction::OpenScriptFileChooser,
                    false,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
                ConsoleMobileButtonModel::new(
                    ConsoleMobileButtonKind::Cancel,
                    "cancel",
                    ConsoleMobileButtonAction::Hide,
                    false,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
            ],
        })
    }
}

impl ConsoleMobileButtonModel {
    fn new(
        kind: ConsoleMobileButtonKind,
        icon: &'static str,
        action: ConsoleMobileButtonAction,
        disabled: bool,
        pad_left: f32,
    ) -> Self {
        Self {
            kind,
            icon,
            style: "cleari",
            action,
            disabled,
            size: CONSOLE_MOBILE_BUTTON_SIZE,
            pad_left,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleFragment {
    messages: Vec<String>,
    open: bool,
    shown: bool,
    history: Vec<String>,
    history_pos: usize,
    scroll_pos: usize,
    field_text: String,
}

impl Default for ConsoleFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleFragment {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            open: false,
            shown: false,
            history: vec![String::new()],
            history_pos: 0,
            scroll_pos: 0,
            field_text: String::new(),
        }
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn field_text(&self) -> &str {
        &self.field_text
    }

    pub fn scroll_pos(&self) -> usize {
        self.scroll_pos
    }

    pub fn set_input(&mut self, text: impl Into<String>) {
        self.field_text = text.into();
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.history.clear();
        self.history.push(String::new());
        self.history_pos = 0;
    }

    pub fn toggle_mobile(&mut self) {
        self.shown = !self.shown;
        self.open = false;
    }

    pub fn toggle(&mut self) -> Vec<ConsoleAction> {
        if !self.open {
            self.open = true;
            self.shown = true;
            vec![
                ConsoleAction::FireOpenConsole,
                ConsoleAction::RequestKeyboard,
            ]
        } else {
            self.open = false;
            self.scroll_pos = 0;
            let mut actions = vec![ConsoleAction::ClearKeyboardFocus];
            actions.extend(self.send_message());
            actions
        }
    }

    pub fn hide(&mut self) -> Vec<ConsoleAction> {
        self.open = false;
        self.clear_chat_input();
        vec![ConsoleAction::ClearKeyboardFocus, ConsoleAction::Hide]
    }

    pub fn clear_chat_input(&mut self) {
        self.history_pos = 0;
        self.history[0].clear();
        self.field_text.clear();
    }

    pub fn history_prev(&mut self) {
        if self.open && self.history_pos < self.history.len() - 1 {
            if self.history_pos == 0 {
                self.history[0] = self.field_text.clone();
            }
            self.history_pos += 1;
            self.update_chat();
        }
    }

    pub fn history_next(&mut self) {
        if self.open && self.history_pos > 0 {
            self.history_pos -= 1;
            self.update_chat();
        }
    }

    pub fn update_chat(&mut self) {
        self.field_text = self.history[self.history_pos].clone();
    }

    pub fn scroll(&mut self, axis: i32) {
        self.scroll_pos =
            (self.scroll_pos as i32 + axis).clamp(0, self.messages.len() as i32) as usize;
    }

    pub fn mobile_scroll_up(&mut self) {
        self.scroll(1);
    }

    pub fn mobile_scroll_down(&mut self) {
        self.scroll(-1);
    }

    pub fn mobile_toolbar_model(&self, mobile: bool) -> Option<ConsoleMobileToolbarModel> {
        if !mobile {
            return None;
        }

        ConsoleMobileToolbarModel::like_java(
            self.shown,
            self.open,
            self.scroll_pos,
            self.messages.len(),
        )
    }

    pub fn dispatch_mobile_toolbar_action(
        &mut self,
        action: ConsoleMobileButtonAction,
    ) -> Vec<ConsoleAction> {
        match action {
            ConsoleMobileButtonAction::OpenTextInput => vec![ConsoleAction::OpenMobileTextInput],
            ConsoleMobileButtonAction::ScrollUp => {
                self.mobile_scroll_up();
                Vec::new()
            }
            ConsoleMobileButtonAction::ScrollDown => {
                self.mobile_scroll_down();
                Vec::new()
            }
            ConsoleMobileButtonAction::OpenScriptFileChooser => {
                vec![ConsoleAction::OpenScriptFileChooser]
            }
            ConsoleMobileButtonAction::Hide => {
                self.shown = false;
                vec![ConsoleAction::Hide]
            }
        }
    }

    pub fn accept_mobile_text_input(&mut self, text: impl Into<String>) -> Vec<ConsoleAction> {
        self.set_input(text);
        let mut actions = self.send_message();
        self.clear_chat_input();
        actions.push(ConsoleAction::HideOnscreenKeyboard);
        actions
    }

    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.insert(0, message.into());
    }

    pub fn send_message(&mut self) -> Vec<ConsoleAction> {
        let message = self.field_text.clone();
        self.clear_chat_input();

        if message.replace(' ', "").is_empty() {
            return Vec::new();
        }

        if message == "clear" {
            self.clear_messages();
            return Vec::new();
        }

        if self.history.len() < 2 || self.history[1] != message {
            self.history.insert(1, message.clone());
        }

        self.add_message(format!("[lightgray]> {}", escape_brackets(&message)));
        vec![ConsoleAction::ExecuteConsole(format!(
            "{CONSOLE_INJECT_VARIABLES}{message}"
        ))]
    }

    pub fn add_console_result(&mut self, result: &str) {
        self.add_message(escape_brackets(result));
    }
}

fn escape_brackets(value: &str) -> String {
    value.replace('[', "[[")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_opens_console_and_closing_sends_current_message() {
        let mut console = ConsoleFragment::new();

        assert_eq!(
            console.toggle(),
            vec![
                ConsoleAction::FireOpenConsole,
                ConsoleAction::RequestKeyboard
            ]
        );
        assert!(console.open());
        console.set_input("1 + 1");
        let actions = console.toggle();

        assert!(!console.open());
        assert_eq!(actions[0], ConsoleAction::ClearKeyboardFocus);
        assert!(matches!(actions[1], ConsoleAction::ExecuteConsole(_)));
        assert_eq!(console.messages()[0], "[lightgray]> 1 + 1");
        assert_eq!(console.history()[1], "1 + 1");
    }

    #[test]
    fn send_message_ignores_blank_and_clear_command_clears_messages() {
        let mut console = ConsoleFragment::new();
        console.set_input("   ");
        assert!(console.send_message().is_empty());

        console.add_message("old");
        console.set_input("clear");
        assert!(console.send_message().is_empty());
        assert!(console.messages().is_empty());
    }

    #[test]
    fn console_result_and_command_escape_markup_brackets() {
        let mut console = ConsoleFragment::new();
        console.set_input("print('[x]')");
        let actions = console.send_message();

        let ConsoleAction::ExecuteConsole(source) = &actions[0] else {
            panic!("expected execute action");
        };
        assert!(source.starts_with("var unit = Vars.player.unit();"));
        assert!(source.ends_with("print('[x]')"));
        assert_eq!(console.messages()[0], "[lightgray]> print('[[x]')");

        console.add_console_result("[ok]");
        assert_eq!(console.messages()[0], "[[ok]");
    }

    #[test]
    fn history_navigation_and_mobile_toggle_match_java_state() {
        let mut console = ConsoleFragment::new();
        console.toggle();
        console.set_input("first");
        console.send_message();
        console.set_input("draft");

        console.history_prev();
        assert_eq!(console.field_text(), "first");
        console.history_next();
        assert_eq!(console.field_text(), "draft");

        console.toggle_mobile();
        assert!(!console.shown());
        assert!(!console.open());
    }

    #[test]
    fn mobile_toolbar_model_matches_java_icons_actions_and_disabled_state() {
        let mut console = ConsoleFragment::new();
        assert!(console.mobile_toolbar_model(false).is_none());
        assert!(console.mobile_toolbar_model(true).is_none());

        console.toggle_mobile();
        let model = console
            .mobile_toolbar_model(true)
            .expect("shown mobile console should expose toolbar model");
        assert!(!model.open);
        assert_eq!(model.field_label, ">");
        assert_eq!(model.messages_shown, CONSOLE_MESSAGES_SHOWN);
        assert_eq!(
            model
                .buttons
                .iter()
                .map(|button| (
                    button.kind,
                    button.icon,
                    button.style,
                    button.action,
                    button.disabled,
                    button.size,
                    button.pad_left,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    ConsoleMobileButtonKind::Chat,
                    "chat",
                    "cleari",
                    ConsoleMobileButtonAction::OpenTextInput,
                    false,
                    CONSOLE_MOBILE_BUTTON_SIZE,
                    0.0,
                ),
                (
                    ConsoleMobileButtonKind::ScrollUp,
                    "upOpen",
                    "cleari",
                    ConsoleMobileButtonAction::ScrollUp,
                    true,
                    CONSOLE_MOBILE_BUTTON_SIZE,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
                (
                    ConsoleMobileButtonKind::ScrollDown,
                    "downOpen",
                    "cleari",
                    ConsoleMobileButtonAction::ScrollDown,
                    true,
                    CONSOLE_MOBILE_BUTTON_SIZE,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
                (
                    ConsoleMobileButtonKind::File,
                    "fileText",
                    "cleari",
                    ConsoleMobileButtonAction::OpenScriptFileChooser,
                    false,
                    CONSOLE_MOBILE_BUTTON_SIZE,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
                (
                    ConsoleMobileButtonKind::Cancel,
                    "cancel",
                    "cleari",
                    ConsoleMobileButtonAction::Hide,
                    false,
                    CONSOLE_MOBILE_BUTTON_SIZE,
                    CONSOLE_MOBILE_BUTTON_PAD_LEFT,
                ),
            ]
        );

        console.add_message("one");
        let model = console.mobile_toolbar_model(true).unwrap();
        assert!(!model.buttons[1].disabled);
        assert!(model.buttons[2].disabled);

        console.mobile_scroll_up();
        let model = console.mobile_toolbar_model(true).unwrap();
        assert!(model.buttons[1].disabled);
        assert!(!model.buttons[2].disabled);
    }

    #[test]
    fn mobile_toolbar_actions_scroll_file_and_cancel_like_java() {
        let mut console = ConsoleFragment::new();
        console.toggle_mobile();
        console.add_message("newest");
        console.add_message("older");

        assert!(console
            .dispatch_mobile_toolbar_action(ConsoleMobileButtonAction::ScrollUp)
            .is_empty());
        assert_eq!(console.scroll_pos(), 1);
        assert!(console
            .dispatch_mobile_toolbar_action(ConsoleMobileButtonAction::ScrollDown)
            .is_empty());
        assert_eq!(console.scroll_pos(), 0);
        assert_eq!(
            console
                .dispatch_mobile_toolbar_action(ConsoleMobileButtonAction::OpenScriptFileChooser),
            vec![ConsoleAction::OpenScriptFileChooser]
        );
        assert_eq!(
            console.dispatch_mobile_toolbar_action(ConsoleMobileButtonAction::Hide),
            vec![ConsoleAction::Hide]
        );
        assert!(!console.shown());
        assert_eq!(console.messages().len(), 2);
    }

    #[test]
    fn mobile_text_input_accept_sends_and_hides_keyboard_like_java() {
        let mut console = ConsoleFragment::new();
        let actions = console.accept_mobile_text_input("print('[x]')");

        let ConsoleAction::ExecuteConsole(source) = &actions[0] else {
            panic!("expected execute action");
        };
        assert!(source.ends_with("print('[x]')"));
        assert_eq!(actions[1], ConsoleAction::HideOnscreenKeyboard);
        assert_eq!(console.messages()[0], "[lightgray]> print('[[x]')");
        assert_eq!(console.history()[1], "print('[x]')");
        assert_eq!(console.field_text(), "");
    }
}
