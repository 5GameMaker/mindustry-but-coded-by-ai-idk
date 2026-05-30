//! Base dialog state shell mirroring upstream `mindustry.ui.dialogs.BaseDialog`.
//!
//! This module intentionally stays data-oriented: it captures the dialog skin
//! contract from Java (`Styles.defaultDialog` / `Styles.fullDialog`) without
//! implementing any rendering.  The symbolic fields here are meant to be wired
//! into real Tex / nine-patch resources later.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogDrawableRef {
    Named(&'static str),
}

impl DialogDrawableRef {
    pub const fn named(name: &'static str) -> Self {
        Self::Named(name)
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Named(name) => name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogColorRef {
    Named(&'static str),
}

impl DialogColorRef {
    pub const fn named(name: &'static str) -> Self {
        Self::Named(name)
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Named(name) => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogTitleStyle {
    pub font: Option<&'static str>,
    pub color: Option<DialogColorRef>,
}

impl Default for DialogTitleStyle {
    fn default() -> Self {
        Self {
            font: None,
            color: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogAccentLineStyle {
    pub drawable: DialogDrawableRef,
    pub tint: Option<DialogColorRef>,
    pub height: f32,
    pub pad: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogStyle {
    pub name: String,
    pub stage_background: Option<DialogDrawableRef>,
    pub background: Option<DialogDrawableRef>,
    pub title: DialogTitleStyle,
    pub accent_line: Option<DialogAccentLineStyle>,
}

impl DialogStyle {
    pub fn bare(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stage_background: None,
            background: None,
            title: DialogTitleStyle::default(),
            accent_line: None,
        }
    }

    pub fn default_dialog() -> Self {
        Self::preset("default", DialogDrawableRef::named("black9"))
    }

    pub fn full_dialog() -> Self {
        Self::preset("full", DialogDrawableRef::named("black"))
    }

    fn preset(name: impl Into<String>, stage_background: DialogDrawableRef) -> Self {
        Self {
            name: name.into(),
            stage_background: Some(stage_background),
            background: Some(DialogDrawableRef::named("window-empty")),
            title: DialogTitleStyle {
                font: Some("Fonts.def"),
                color: Some(DialogColorRef::named("accent")),
            },
            accent_line: Some(DialogAccentLineStyle {
                drawable: DialogDrawableRef::named("whiteui"),
                tint: Some(DialogColorRef::named("accent")),
                height: 3.0,
                pad: 4.0,
            }),
        }
    }
}

impl Default for DialogStyle {
    fn default() -> Self {
        Self::default_dialog()
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
    fn dialog_style_default_matches_java_default_dialog_skin_contract() {
        let style = DialogStyle::default();

        assert_eq!(style.name, "default");
        assert_eq!(
            style.stage_background,
            Some(DialogDrawableRef::named("black9"))
        );
        assert_eq!(
            style.background,
            Some(DialogDrawableRef::named("window-empty"))
        );
        assert_eq!(style.title.font, Some("Fonts.def"));
        assert_eq!(style.title.color, Some(DialogColorRef::named("accent")));
        assert_eq!(
            style.accent_line,
            Some(DialogAccentLineStyle {
                drawable: DialogDrawableRef::named("whiteui"),
                tint: Some(DialogColorRef::named("accent")),
                height: 3.0,
                pad: 4.0,
            })
        );
    }

    #[test]
    fn dialog_style_full_matches_java_full_dialog_skin_contract() {
        let style = DialogStyle::full_dialog();

        assert_eq!(style.name, "full");
        assert_eq!(
            style.stage_background,
            Some(DialogDrawableRef::named("black"))
        );
        assert_eq!(
            style.background,
            Some(DialogDrawableRef::named("window-empty"))
        );
        assert_eq!(style.title.font, Some("Fonts.def"));
        assert_eq!(style.title.color, Some(DialogColorRef::named("accent")));
        assert_eq!(
            style.accent_line,
            Some(DialogAccentLineStyle {
                drawable: DialogDrawableRef::named("whiteui"),
                tint: Some(DialogColorRef::named("accent")),
                height: 3.0,
                pad: 4.0,
            })
        );
    }

    #[test]
    fn bare_dialog_style_keeps_name_only_shell_available() {
        let style = DialogStyle::bare("custom");

        assert_eq!(style.name, "custom");
        assert_eq!(style.stage_background, None);
        assert_eq!(style.background, None);
        assert_eq!(style.title, DialogTitleStyle::default());
        assert_eq!(style.accent_line, None);
    }

    #[test]
    fn base_dialog_constructor_sets_java_layout_defaults() {
        let dialog = BaseDialog::new("@title");

        assert_eq!(dialog.title, "@title");
        assert!(dialog.fill_parent);
        assert_eq!(dialog.title_alignment, DialogAlignment::Center);
        assert_eq!(dialog.title_separator_height, 3.0);
        assert_eq!(dialog.title_separator_pad, 4.0);
        assert!(!dialog.should_pause);
        assert_eq!(dialog.style, DialogStyle::default());
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
