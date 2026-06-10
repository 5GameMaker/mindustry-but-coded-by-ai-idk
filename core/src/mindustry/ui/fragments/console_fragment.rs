//! Developer console HUD state model mirroring upstream `mindustry.ui.fragments.ConsoleFragment`.

pub const CONSOLE_MESSAGES_SHOWN: usize = 30;
pub const CONSOLE_INJECT_VARIABLES: &str =
    "var unit = Vars.player.unit();var player = Vars.player;var team = Vars.player.team();var core = Vars.player.core();var items = Vars.player.team().items();var build = Vars.world.buildWorld(Core.input.mouseWorldX(), Core.input.mouseWorldY());var cursor = Vars.world.tileWorld(Core.input.mouseWorldX(), Core.input.mouseWorldY());var cursorUnit = Units.closestEnemy(null, Core.input.mouseWorldX(), Core.input.mouseWorldY(), 70, u => true);\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleAction {
    FireOpenConsole,
    RequestKeyboard,
    ClearKeyboardFocus,
    ClearScrollFocus,
    ExecuteConsole(String),
    Hide,
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
}
