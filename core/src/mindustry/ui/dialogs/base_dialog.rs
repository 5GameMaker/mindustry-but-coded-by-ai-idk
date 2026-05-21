//! Base dialog state shell mirroring upstream `mindustry.ui.dialogs.BaseDialog`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogStyle {
    pub name: String,
}

impl Default for DialogStyle {
    fn default() -> Self {
        Self {
            name: "default".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAlignment {
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogState {
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogRuntime {
    pub is_game: bool,
    pub net_active: bool,
    pub state: DialogState,
    pub ui_back_sounds: usize,
}

impl DialogRuntime {
    pub fn game_playing() -> Self {
        Self {
            is_game: true,
            net_active: false,
            state: DialogState::Playing,
            ui_back_sounds: 0,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.state == DialogState::Paused
    }

    pub fn set(&mut self, state: DialogState) {
        self.state = state;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogResizeContext {
    pub is_shown: bool,
    pub is_top_dialog: bool,
    pub text_input_shown: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseDialog {
    pub title: String,
    pub style: DialogStyle,
    pub fill_parent: bool,
    pub title_alignment: DialogAlignment,
    pub title_separator_height: f32,
    pub title_separator_pad: f32,
    pub was_paused: bool,
    pub should_pause: bool,
    shown: bool,
    button_overlay: bool,
    close_listener: bool,
    close_button_width: Option<f32>,
    resize_focus_updates: usize,
}

impl BaseDialog {
    pub fn new(title: impl Into<String>) -> Self {
        Self::with_style(title, DialogStyle::default())
    }

    pub fn with_style(title: impl Into<String>, style: DialogStyle) -> Self {
        Self {
            title: title.into(),
            style,
            fill_parent: true,
            title_alignment: DialogAlignment::Center,
            title_separator_height: 3.0,
            title_separator_pad: 4.0,
            was_paused: false,
            should_pause: false,
            shown: false,
            button_overlay: false,
            close_listener: false,
            close_button_width: None,
            resize_focus_updates: 0,
        }
    }

    pub fn is_shown(&self) -> bool {
        self.shown
    }

    pub fn has_button_overlay(&self) -> bool {
        self.button_overlay
    }

    pub fn has_close_listener(&self) -> bool {
        self.close_listener
    }

    pub fn close_button_width(&self) -> Option<f32> {
        self.close_button_width
    }

    pub fn resize_focus_updates(&self) -> usize {
        self.resize_focus_updates
    }

    /// Places the buttons as an overlay on top of the content.
    pub fn make_button_overlay(&mut self) {
        self.button_overlay = true;
    }

    pub fn show(&mut self, runtime: &mut DialogRuntime) {
        self.shown = true;
        if self.should_pause && runtime.is_game && !runtime.net_active {
            self.was_paused = runtime.is_paused();
            runtime.set(DialogState::Paused);
        }
    }

    pub fn hide(&mut self, runtime: &mut DialogRuntime) {
        self.shown = false;
        if self.should_pause && runtime.is_game && !runtime.net_active && !self.was_paused {
            runtime.set(DialogState::Playing);
        }
        runtime.ui_back_sounds += 1;
    }

    pub fn on_resize<F: FnOnce()>(&mut self, context: DialogResizeContext, run: F) -> bool {
        if context.is_shown && context.is_top_dialog && !context.text_input_shown {
            run();
            self.resize_focus_updates += 1;
            true
        } else {
            false
        }
    }

    pub fn add_close_listener(&mut self) {
        self.close_listener = true;
    }

    pub fn add_close_button_with_width(&mut self, width: f32) {
        self.close_button_width = Some(width);
        self.add_close_listener();
    }

    pub fn add_close_button(&mut self) {
        self.add_close_button_with_width(210.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_dialog_constructor_sets_java_layout_defaults() {
        let dialog = BaseDialog::new("@title");

        assert_eq!(dialog.title, "@title");
        assert!(dialog.fill_parent);
        assert_eq!(dialog.title_alignment, DialogAlignment::Center);
        assert_eq!(dialog.title_separator_height, 3.0);
        assert_eq!(dialog.title_separator_pad, 4.0);
        assert!(!dialog.should_pause);
    }

    #[test]
    fn show_and_hide_pause_and_restore_game_when_configured() {
        let mut dialog = BaseDialog::new("@title");
        dialog.should_pause = true;
        let mut runtime = DialogRuntime::game_playing();

        dialog.show(&mut runtime);
        assert!(dialog.is_shown());
        assert_eq!(runtime.state, DialogState::Paused);
        assert!(!dialog.was_paused);

        dialog.hide(&mut runtime);
        assert!(!dialog.is_shown());
        assert_eq!(runtime.state, DialogState::Playing);
        assert_eq!(runtime.ui_back_sounds, 1);
    }

    #[test]
    fn hide_keeps_game_paused_if_it_was_paused_before_showing() {
        let mut dialog = BaseDialog::new("@title");
        dialog.should_pause = true;
        let mut runtime = DialogRuntime {
            state: DialogState::Paused,
            ..DialogRuntime::game_playing()
        };

        dialog.show(&mut runtime);
        dialog.hide(&mut runtime);

        assert_eq!(runtime.state, DialogState::Paused);
    }

    #[test]
    fn net_active_dialog_does_not_pause_or_resume() {
        let mut dialog = BaseDialog::new("@title");
        dialog.should_pause = true;
        let mut runtime = DialogRuntime {
            net_active: true,
            ..DialogRuntime::game_playing()
        };

        dialog.show(&mut runtime);
        assert_eq!(runtime.state, DialogState::Playing);
        dialog.hide(&mut runtime);
        assert_eq!(runtime.state, DialogState::Playing);
    }

    #[test]
    fn overlay_close_button_and_resize_hooks_match_base_dialog_shape() {
        let mut dialog = BaseDialog::new("@title");
        dialog.make_button_overlay();
        dialog.add_close_button();
        let mut resized = false;

        let ran = dialog.on_resize(
            DialogResizeContext {
                is_shown: true,
                is_top_dialog: true,
                text_input_shown: false,
            },
            || resized = true,
        );

        assert!(dialog.has_button_overlay());
        assert_eq!(dialog.close_button_width(), Some(210.0));
        assert!(dialog.has_close_listener());
        assert!(ran);
        assert!(resized);
        assert_eq!(dialog.resize_focus_updates(), 1);
    }

    #[test]
    fn resize_ignores_hidden_non_top_or_text_input_contexts() {
        let mut dialog = BaseDialog::new("@title");
        for context in [
            DialogResizeContext {
                is_shown: false,
                is_top_dialog: true,
                text_input_shown: false,
            },
            DialogResizeContext {
                is_shown: true,
                is_top_dialog: false,
                text_input_shown: false,
            },
            DialogResizeContext {
                is_shown: true,
                is_top_dialog: true,
                text_input_shown: true,
            },
        ] {
            assert!(!dialog.on_resize(context, || panic!("must not run")));
        }
        assert_eq!(dialog.resize_focus_updates(), 0);
    }
}
