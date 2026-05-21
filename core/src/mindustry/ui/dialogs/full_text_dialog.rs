//! Full text dialog shell mirroring upstream `mindustry.ui.dialogs.FullTextDialog`.

use super::{BaseDialog, DialogRuntime};

#[derive(Debug, Clone, PartialEq)]
pub struct FullTextDialog {
    pub base: BaseDialog,
    pub text: String,
    pub text_wrap: bool,
    pub text_centered: bool,
}

impl FullTextDialog {
    pub fn new() -> Self {
        let mut base = BaseDialog::new("");
        base.should_pause = true;
        base.add_close_button();
        Self {
            base,
            text: String::new(),
            text_wrap: true,
            text_centered: true,
        }
    }

    pub fn show(
        &mut self,
        title_text: impl Into<String>,
        text: impl Into<String>,
        runtime: &mut DialogRuntime,
    ) {
        self.base.title = title_text.into();
        self.text = text.into();
        self.base.show(runtime);
    }
}

impl Default for FullTextDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ui::dialogs::DialogState;

    #[test]
    fn constructor_sets_pause_and_close_button_like_java() {
        let dialog = FullTextDialog::new();

        assert_eq!(dialog.base.title, "");
        assert!(dialog.base.should_pause);
        assert_eq!(dialog.base.close_button_width(), Some(210.0));
        assert!(dialog.base.has_close_listener());
        assert!(dialog.text_wrap);
        assert!(dialog.text_centered);
    }

    #[test]
    fn show_replaces_title_and_text_then_opens_dialog() {
        let mut dialog = FullTextDialog::new();
        let mut runtime = DialogRuntime::game_playing();

        dialog.show("@rules", "long body", &mut runtime);
        assert_eq!(dialog.base.title, "@rules");
        assert_eq!(dialog.text, "long body");
        assert!(dialog.base.is_shown());
        assert_eq!(runtime.state, DialogState::Paused);

        dialog.show("@help", "new body", &mut runtime);
        assert_eq!(dialog.base.title, "@help");
        assert_eq!(dialog.text, "new body");
    }
}
