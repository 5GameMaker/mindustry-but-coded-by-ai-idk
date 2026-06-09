//! Full text dialog shell mirroring upstream `mindustry.ui.dialogs.FullTextDialog`.

use super::{BaseDialog, DialogAlignment, DialogRuntime, DialogShellLayout, DialogStyle};
use crate::mindustry::graphics::{
    Pal, RenderCommand, RenderRect, RenderTextAlign, RenderTextVerticalAlign,
};

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
    use crate::mindustry::graphics::RenderFontId;
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

    #[test]
    fn full_text_dialog_show_replaces_title_and_text_and_centers_body_like_java() {
        let mut dialog = FullTextDialog::new();
        let mut runtime = DialogRuntime::game_playing();

        assert_eq!(dialog.base.style, DialogStyle::default());
        assert!(dialog.base.fill_parent);
        assert_eq!(dialog.base.title_alignment, DialogAlignment::Center);
        assert_eq!(dialog.base.title_separator_height, 3.0);
        assert_eq!(dialog.base.title_separator_pad, 4.0);
        assert!(dialog.text_wrap);
        assert!(dialog.text_centered);

        dialog.show("@rules", "long body", &mut runtime);

        assert_eq!(dialog.base.title, "@rules");
        assert_eq!(dialog.text, "long body");
        assert!(dialog.base.is_shown());
        assert_eq!(runtime.state, DialogState::Paused);

        let layout = DialogShellLayout::from_stage_and_panel(
            RenderRect::new(0.0, 0.0, 800.0, 600.0),
            RenderRect::new(140.0, 160.0, 520.0, 220.0),
        );
        let commands = dialog.base.shell_render_commands(layout);

        assert_eq!(commands.len(), 4);
        match &commands[0] {
            RenderCommand::DrawSprite {
                symbol, rect, tint, ..
            } => {
                assert_eq!(symbol, "whiteui");
                assert_eq!(*rect, layout.stage_rect);
                assert_eq!(*tint, [0.0, 0.0, 0.0, 0.9]);
            }
            other => panic!("expected full-screen black stage sprite, got {other:?}"),
        }
        match &commands[1] {
            RenderCommand::DrawSprite {
                symbol, rect, tint, ..
            } => {
                assert_eq!(symbol, "window-empty.9");
                assert_eq!(*rect, layout.panel_rect);
                assert_eq!(*tint, [1.0, 1.0, 1.0, 1.0]);
            }
            other => panic!("expected default dialog panel sprite, got {other:?}"),
        }
        match &commands[2] {
            RenderCommand::DrawText {
                text,
                color,
                align,
                style,
                ..
            } => {
                assert_eq!(text, "@rules");
                assert_eq!(
                    *color,
                    [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a]
                );
                assert_eq!(*align, RenderTextAlign::Center);
                assert_eq!(style.horizontal_align, RenderTextAlign::Center);
                assert_eq!(style.vertical_align, RenderTextVerticalAlign::Center);
                assert_eq!(
                    style.font,
                    RenderFontId::Default,
                    "Java FullTextDialog title uses the default UI font through BaseDialog"
                );
                assert!(style.integer_position);
                assert!(
                    !style.outline,
                    "Java BaseDialog title uses Fonts.def; FullTextDialog must not reintroduce a Rust-only runtime outline"
                );
            }
            other => panic!("expected centered title text, got {other:?}"),
        }
        match &commands[3] {
            RenderCommand::DrawSprite {
                symbol, rect, tint, ..
            } => {
                assert_eq!(symbol, "whiteui");
                assert_eq!(*rect, layout.accent_line_rect);
                assert_eq!(
                    *tint,
                    [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a]
                );
            }
            other => panic!("expected accent separator sprite, got {other:?}"),
        }
    }
}
