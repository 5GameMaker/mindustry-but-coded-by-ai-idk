//! Loading overlay state model mirroring upstream `mindustry.ui.fragments.LoadingFragment`.

pub const LOADING_FRAGMENT_FADE_OUT_DURATION: f32 = 0.5;
pub const LOADING_FRAGMENT_BAR_SIZE: (f32, f32) = (500.0, 40.0);
pub const LOADING_FRAGMENT_CANCEL_BUTTON_SIZE: (f32, f32) = (250.0, 70.0);

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingFragmentAction {
    ToFront,
    RequestCancelKeyboard,
    ClearKeyboardFocus,
    FadeOut { duration: f32 },
    Visible(bool),
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadingFragmentModel {
    pub visible: bool,
    pub touchable: bool,
    pub text: String,
    pub label_uses_tech_font: bool,
    pub label_accent: bool,
    pub bar_visible: bool,
    pub progress_label: Option<String>,
    pub progress_value: Option<f32>,
    pub button_visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadingFragment {
    visible: bool,
    touchable: bool,
    text: String,
    label_uses_tech_font: bool,
    label_accent: bool,
    bar_visible: bool,
    progress_value: Option<f32>,
    button_visible: bool,
    cancel_listener: bool,
}

impl Default for LoadingFragment {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadingFragment {
    pub fn new() -> Self {
        Self {
            visible: false,
            touchable: true,
            text: "@loading".into(),
            label_uses_tech_font: true,
            label_accent: false,
            bar_visible: false,
            progress_value: None,
            button_visible: false,
            cancel_listener: false,
        }
    }

    pub fn model(&self) -> LoadingFragmentModel {
        LoadingFragmentModel {
            visible: self.visible,
            touchable: self.touchable,
            text: self.text.clone(),
            label_uses_tech_font: self.label_uses_tech_font,
            label_accent: self.label_accent,
            bar_visible: self.bar_visible,
            progress_label: self
                .progress_value
                .map(|value| format!("{}%", (value * 100.0) as i32)),
            progress_value: self.progress_value,
            button_visible: self.button_visible,
        }
    }

    pub fn show(&mut self) -> Vec<LoadingFragmentAction> {
        self.show_text("@loading")
    }

    pub fn show_text(&mut self, text: impl Into<String>) -> Vec<LoadingFragmentAction> {
        self.button_visible = false;
        self.cancel_listener = false;
        self.label_accent = false;
        self.bar_visible = false;
        self.touchable = true;
        self.set_text_internal(text.into());
        self.visible = true;
        vec![
            LoadingFragmentAction::Visible(true),
            LoadingFragmentAction::ToFront,
        ]
    }

    pub fn hide(&mut self, keyboard_focus_is_button: bool) -> Vec<LoadingFragmentAction> {
        self.button_visible = false;
        self.touchable = false;
        self.visible = false;
        let mut actions = vec![
            LoadingFragmentAction::ToFront,
            LoadingFragmentAction::FadeOut {
                duration: LOADING_FRAGMENT_FADE_OUT_DURATION,
            },
            LoadingFragmentAction::Visible(false),
        ];
        if keyboard_focus_is_button {
            actions.push(LoadingFragmentAction::ClearKeyboardFocus);
        }
        actions
    }

    pub fn to_front(&self) -> LoadingFragmentAction {
        LoadingFragmentAction::ToFront
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress_value = Some(progress);
        self.bar_visible = true;
    }

    pub fn snap_progress(&mut self) {
        self.progress_value = self.progress_value.map(|value| value.clamp(0.0, 1.0));
    }

    pub fn set_button(&mut self) -> Vec<LoadingFragmentAction> {
        self.button_visible = true;
        self.cancel_listener = true;
        vec![LoadingFragmentAction::RequestCancelKeyboard]
    }

    pub fn press_cancel(&self) -> Option<LoadingFragmentAction> {
        self.cancel_listener
            .then_some(LoadingFragmentAction::Cancel)
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.set_text_internal(text.into());
        self.label_accent = true;
    }

    fn set_text_internal(&mut self, text: String) {
        self.label_uses_tech_font = text.chars().all(is_tech_font_supported);
        self.text = text;
    }
}

fn is_tech_font_supported(ch: char) -> bool {
    ch.is_ascii()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_resets_button_progress_color_and_brings_overlay_to_front() {
        let mut loading = LoadingFragment::new();
        loading.set_progress(0.5);
        loading.set_button();
        loading.set_text("custom");

        let actions = loading.show();
        let model = loading.model();

        assert_eq!(
            actions,
            vec![
                LoadingFragmentAction::Visible(true),
                LoadingFragmentAction::ToFront
            ]
        );
        assert!(model.visible);
        assert!(model.touchable);
        assert_eq!(model.text, "@loading");
        assert!(!model.label_accent);
        assert!(!model.bar_visible);
        assert!(!model.button_visible);
    }

    #[test]
    fn progress_button_hide_and_cancel_match_java_sequence() {
        let mut loading = LoadingFragment::new();

        loading.set_progress(0.42);
        assert_eq!(loading.model().progress_label, Some("42%".into()));
        assert_eq!(
            loading.set_button(),
            vec![LoadingFragmentAction::RequestCancelKeyboard]
        );
        assert_eq!(loading.press_cancel(), Some(LoadingFragmentAction::Cancel));

        let actions = loading.hide(true);
        assert_eq!(
            actions,
            vec![
                LoadingFragmentAction::ToFront,
                LoadingFragmentAction::FadeOut { duration: 0.5 },
                LoadingFragmentAction::Visible(false),
                LoadingFragmentAction::ClearKeyboardFocus
            ]
        );
        assert!(!loading.model().touchable);
    }

    #[test]
    fn set_text_uses_default_label_when_tech_font_missing_glyphs() {
        let mut loading = LoadingFragment::new();
        loading.set_text("加载中");

        let model = loading.model();
        assert!(model.label_accent);
        assert!(!model.label_uses_tech_font);
    }
}
