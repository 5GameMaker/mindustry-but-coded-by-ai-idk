//! Chat HUD state model mirroring upstream `mindustry.ui.fragments.ChatFragment`.

pub const CHAT_MESSAGES_SHOWN: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatMode {
    Normal,
    Team,
    Admin,
}

impl ChatMode {
    pub const ALL: [ChatMode; 3] = [ChatMode::Normal, ChatMode::Team, ChatMode::Admin];

    pub const fn prefix(self) -> &'static str {
        match self {
            ChatMode::Normal => "",
            ChatMode::Team => "/t",
            ChatMode::Admin => "/a",
        }
    }

    pub fn normalized_prefix(self) -> String {
        if self.prefix().is_empty() {
            String::new()
        } else {
            format!("{} ", self.prefix())
        }
    }

    pub fn next(self) -> Self {
        Self::ALL[(self.ordinal() + 1) % Self::ALL.len()]
    }

    const fn ordinal(self) -> usize {
        match self {
            ChatMode::Normal => 0,
            ChatMode::Team => 1,
            ChatMode::Admin => 2,
        }
    }

    fn valid(self, player_admin: bool) -> bool {
        !matches!(self, ChatMode::Admin) || player_admin
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatAction {
    RequestKeyboard,
    ClearKeyboardFocus,
    FireClientChatEvent(String),
    SendChatMessage(String),
    PingLocation { x: f32, y: f32, text: String },
    RequestMobileTextInput { max_length: usize },
    Hide,
    ScheduleSend { delay_frames: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatUpdateFrame {
    pub net_active: bool,
    pub chat_key_tap: bool,
    pub has_other_focus: bool,
    pub console_shown: bool,
    pub mobile: bool,
    pub max_text_length: usize,
    pub chat_history_prev_key_tap: bool,
    pub chat_history_next_key_tap: bool,
    pub chat_mode_key_tap: bool,
    pub chat_scroll_axis: i32,
    pub player_admin: bool,
}

impl Default for ChatUpdateFrame {
    fn default() -> Self {
        Self {
            net_active: true,
            chat_key_tap: false,
            has_other_focus: false,
            console_shown: false,
            mobile: false,
            max_text_length: 150,
            chat_history_prev_key_tap: false,
            chat_history_next_key_tap: false,
            chat_mode_key_tap: false,
            chat_scroll_axis: 0,
            player_admin: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatDrawMessage {
    pub text: String,
    pub fading_alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatFragment {
    messages: Vec<String>,
    fadetime: f32,
    shown: bool,
    mode: ChatMode,
    history: Vec<String>,
    history_pos: usize,
    scroll_pos: usize,
    field_text: String,
    last_frame_had_focus: bool,
}

impl Default for ChatFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatFragment {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            fadetime: 0.0,
            shown: false,
            mode: ChatMode::Normal,
            history: vec![String::new()],
            history_pos: 0,
            scroll_pos: 0,
            field_text: String::new(),
            last_frame_had_focus: false,
        }
    }

    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn mode(&self) -> ChatMode {
        self.mode
    }

    pub fn field_text(&self) -> &str {
        &self.field_text
    }

    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn scroll_pos(&self) -> usize {
        self.scroll_pos
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.history.clear();
        self.history.push(String::new());
        self.history_pos = 0;
    }

    pub fn update_like_java(&mut self, frame: &ChatUpdateFrame) -> Vec<ChatAction> {
        let mut actions = Vec::new();
        if frame.net_active
            && frame.chat_key_tap
            && !frame.has_other_focus
            && !self.last_frame_had_focus
            && !frame.console_shown
        {
            actions.extend(self.toggle(frame.mobile, frame.max_text_length));
        }

        if self.shown {
            if frame.chat_history_prev_key_tap {
                self.history_prev();
            }
            if frame.chat_history_next_key_tap {
                self.history_next();
            }
            if frame.chat_mode_key_tap {
                self.next_mode(frame.player_admin);
            }
            self.scroll(frame.chat_scroll_axis);
        }

        self.last_frame_had_focus = frame.has_other_focus;
        actions
    }

    pub fn toggle(&mut self, mobile: bool, max_text_length: usize) -> Vec<ChatAction> {
        if !self.shown {
            self.shown = true;
            if mobile {
                vec![
                    ChatAction::RequestKeyboard,
                    ChatAction::RequestMobileTextInput {
                        max_length: max_text_length,
                    },
                ]
            } else {
                vec![ChatAction::RequestKeyboard]
            }
        } else {
            vec![ChatAction::ScheduleSend { delay_frames: 2.0 }]
        }
    }

    pub fn finish_scheduled_send(
        &mut self,
        world_in_bounds: impl Fn(i32, i32) -> bool,
    ) -> Vec<ChatAction> {
        self.shown = false;
        self.scroll_pos = 0;
        let mut actions = vec![ChatAction::ClearKeyboardFocus];
        actions.extend(self.send_message(world_in_bounds));
        actions
    }

    pub fn hide(&mut self) -> Vec<ChatAction> {
        self.shown = false;
        self.clear_chat_input();
        vec![ChatAction::ClearKeyboardFocus, ChatAction::Hide]
    }

    pub fn set_input(&mut self, text: impl Into<String>) {
        self.field_text = text.into();
    }

    pub fn clear_chat_input(&mut self) {
        self.history_pos = 0;
        self.history[0].clear();
        self.field_text = self.mode.normalized_prefix();
    }

    pub fn next_mode(&mut self, player_admin: bool) {
        let previous = self.mode;
        loop {
            self.mode = self.mode.next();
            if self.mode.valid(player_admin) {
                break;
            }
        }

        let previous_prefix = previous.normalized_prefix();
        if self.field_text.starts_with(&previous_prefix) {
            self.field_text =
                self.mode.normalized_prefix() + &self.field_text[previous_prefix.len()..];
        } else {
            self.field_text = self.mode.normalized_prefix();
        }
    }

    pub fn history_prev(&mut self) {
        if self.history_pos < self.history.len() - 1 {
            if self.history_pos == 0 {
                self.history[0] = self.field_text.clone();
            }
            self.history_pos += 1;
            self.update_chat();
        }
    }

    pub fn history_next(&mut self) {
        if self.history_pos > 0 {
            self.history_pos -= 1;
            self.update_chat();
        }
    }

    pub fn update_chat(&mut self) {
        self.field_text = self.mode.normalized_prefix() + &self.history[self.history_pos];
    }

    pub fn scroll(&mut self, axis: i32) {
        let max = self.messages.len().saturating_sub(CHAT_MESSAGES_SHOWN);
        self.scroll_pos = (self.scroll_pos as i32 + axis).clamp(0, max as i32) as usize;
    }

    pub fn add_message(&mut self, message: Option<&str>) {
        if let Some(message) = message {
            self.messages.insert(0, message.to_string());
            self.fadetime += 1.0;
            self.fadetime = self.fadetime.min(CHAT_MESSAGES_SHOWN as f32) + 1.0;
            if self.scroll_pos > 0 {
                self.scroll_pos += 1;
            }
        }
    }

    pub fn send_message(&mut self, world_in_bounds: impl Fn(i32, i32) -> bool) -> Vec<ChatAction> {
        let mut message = self.field_text.trim().to_string();
        self.clear_chat_input();

        if message.is_empty()
            || (message.starts_with(self.mode.prefix())
                && message[self.mode.prefix().len()..].is_empty())
        {
            return Vec::new();
        }

        if self.history.len() < 2 || self.history[1] != message {
            self.history.insert(1, message.clone());
        }

        message = format_icons(&message);
        let mut actions = check_ping(&message, world_in_bounds);
        actions.push(ChatAction::FireClientChatEvent(message.clone()));
        actions.push(ChatAction::SendChatMessage(message));
        actions
    }

    pub fn draw_messages(&self) -> Vec<ChatDrawMessage> {
        let mut out = Vec::new();
        for i in self.scroll_pos
            ..self
                .messages
                .len()
                .min(CHAT_MESSAGES_SHOWN + self.scroll_pos)
        {
            if i < self.fadetime as usize || self.shown {
                let fade =
                    if !self.shown && self.fadetime - (i as f32) < 1.0 && self.fadetime >= i as f32
                    {
                        self.fadetime - i as f32
                    } else {
                        1.0
                    };
                out.push(ChatDrawMessage {
                    text: self.messages[i].clone(),
                    fading_alpha: fade,
                });
            }
        }
        out
    }

    pub fn set_last_frame_had_focus(&mut self, value: bool) {
        self.last_frame_had_focus = value;
    }

    pub fn last_frame_had_focus(&self) -> bool {
        self.last_frame_had_focus
    }
}

fn format_icons(message: &str) -> String {
    message.to_string()
}

fn check_ping(message: &str, world_in_bounds: impl Fn(i32, i32) -> bool) -> Vec<ChatAction> {
    let Some(comma) = message.find(',') else {
        return Vec::new();
    };
    let mut space = message[comma + 1..]
        .find(' ')
        .map(|index| index + comma + 1);
    let mut extra = false;
    if space == Some(comma + 1) {
        extra = true;
        space = message[comma + 2..]
            .find(' ')
            .map(|index| index + comma + 2);
    }
    let Some(space) = space else {
        return Vec::new();
    };

    let x = message[..comma].parse::<i32>().unwrap_or(-1);
    let y_start = comma + 1 + usize::from(extra);
    let y = message[y_start..space].parse::<i32>().unwrap_or(-1);
    if world_in_bounds(x, y) {
        vec![ChatAction::PingLocation {
            x: x as f32 * 8.0,
            y: y as f32 * 8.0,
            text: message[space..].trim().to_string(),
        }]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_message_inserts_newest_first_and_updates_fade_scroll() {
        let mut chat = ChatFragment::new();
        chat.add_message(Some("one"));
        chat.add_message(Some("two"));

        assert_eq!(chat.messages(), &["two".to_string(), "one".to_string()]);
        assert_eq!(chat.draw_messages()[0].text, "two");
        assert!(chat.draw_messages()[0].fading_alpha > 0.0);
    }

    #[test]
    fn toggle_open_requests_keyboard_and_close_schedules_send_like_java_delay() {
        let mut chat = ChatFragment::new();

        assert_eq!(chat.toggle(false, 100), vec![ChatAction::RequestKeyboard]);
        assert!(chat.shown());
        assert_eq!(
            chat.toggle(false, 100),
            vec![ChatAction::ScheduleSend { delay_frames: 2.0 }]
        );
    }

    #[test]
    fn send_message_trims_ignores_prefix_empty_updates_history_and_detects_ping() {
        let mut chat = ChatFragment::new();
        chat.set_input(" 12, 8 hello ");

        let actions = chat.send_message(|x, y| x == 12 && y == 8);

        assert_eq!(chat.history()[1], "12, 8 hello");
        assert!(actions.contains(&ChatAction::PingLocation {
            x: 96.0,
            y: 64.0,
            text: "hello".into()
        }));
        assert!(actions.contains(&ChatAction::FireClientChatEvent("12, 8 hello".into())));
        assert!(actions.contains(&ChatAction::SendChatMessage("12, 8 hello".into())));

        chat.next_mode(false);
        assert_eq!(chat.mode(), ChatMode::Team);
        chat.set_input("/t ");
        assert!(chat.send_message(|_, _| true).is_empty());
    }

    #[test]
    fn chat_mode_skips_admin_when_player_is_not_admin_and_preserves_body() {
        let mut chat = ChatFragment::new();
        chat.set_input("hello");
        chat.next_mode(false);
        assert_eq!(chat.mode(), ChatMode::Team);
        assert_eq!(chat.field_text(), "/t hello");
        chat.set_input("/t hello");
        chat.next_mode(false);
        assert_eq!(chat.mode(), ChatMode::Normal);
        assert_eq!(chat.field_text(), "hello");

        chat.next_mode(true);
        chat.next_mode(true);
        assert_eq!(chat.mode(), ChatMode::Admin);
        assert_eq!(chat.field_text(), "/a hello");
    }

    #[test]
    fn history_navigation_restores_typed_draft() {
        let mut chat = ChatFragment::new();
        chat.set_input("first");
        chat.send_message(|_, _| false);
        chat.set_input("draft");

        chat.history_prev();
        assert_eq!(chat.field_text(), "first");
        chat.history_next();
        assert_eq!(chat.field_text(), "draft");
    }

    #[test]
    fn update_opens_only_when_net_active_no_focus_and_console_closed_like_java() {
        let mut chat = ChatFragment::new();
        let mut frame = ChatUpdateFrame {
            chat_key_tap: true,
            ..Default::default()
        };

        assert_eq!(
            chat.update_like_java(&frame),
            vec![ChatAction::RequestKeyboard]
        );
        assert!(chat.shown());

        chat.hide();
        frame.console_shown = true;
        assert!(chat.update_like_java(&frame).is_empty());
        assert!(!chat.shown());

        frame.console_shown = false;
        frame.has_other_focus = true;
        assert!(chat.update_like_java(&frame).is_empty());
        assert!(chat.last_frame_had_focus());

        frame.has_other_focus = false;
        assert!(chat.update_like_java(&frame).is_empty());
        assert!(!chat.shown());

        assert_eq!(
            chat.update_like_java(&frame),
            vec![ChatAction::RequestKeyboard]
        );
        assert!(chat.shown());
    }

    #[test]
    fn update_navigates_history_mode_and_scroll_when_shown_like_java() {
        let mut chat = ChatFragment::new();
        chat.set_input("first");
        chat.send_message(|_, _| false);
        chat.set_input("draft");
        chat.toggle(false, 150);
        for index in 0..14 {
            chat.add_message(Some(&format!("message-{index}")));
        }

        chat.update_like_java(&ChatUpdateFrame {
            chat_history_prev_key_tap: true,
            chat_mode_key_tap: true,
            chat_scroll_axis: 8,
            player_admin: false,
            ..Default::default()
        });
        assert_eq!(chat.field_text(), "/t first");
        assert_eq!(chat.mode(), ChatMode::Team);
        assert_eq!(chat.scroll_pos(), 4);

        chat.update_like_java(&ChatUpdateFrame {
            chat_history_next_key_tap: true,
            chat_mode_key_tap: true,
            chat_scroll_axis: -99,
            player_admin: false,
            ..Default::default()
        });
        assert_eq!(chat.field_text(), "draft");
        assert_eq!(chat.mode(), ChatMode::Normal);
        assert_eq!(chat.scroll_pos(), 0);
    }
}
