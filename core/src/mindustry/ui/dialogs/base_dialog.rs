//! Base dialog state shell mirroring upstream `mindustry.ui.dialogs.BaseDialog`.
//!
//! This module intentionally keeps the runtime state data-oriented, while also
//! exposing a small backend-neutral render seam for the Java
//! `Styles.defaultDialog` / `Styles.fullDialog` skin contract.  The generated
//! commands name the real atlas resources (`window-empty.9`, `whiteui`) so the
//! desktop/backend layer can resolve them as Arc/Java drawables instead of
//! falling back to ad-hoc solid rectangles.

use crate::mindustry::graphics::{
    Layer, Pal, RenderCommand, RenderPoint, RenderRect, RenderTextAlign, RenderTextStyle,
    RenderTextVerticalAlign,
};
use crate::mindustry::ui::UiDrawableTint;

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

    /// Atlas symbol used by the render backend.
    ///
    /// Java exposes `Tex.windowEmpty` from `window-empty.9.png`, while the
    /// `Styles.load()` drawable aliases like `black8`, `black6`, `black5`,
    /// `black3`, `none`, `grayPanel`, `grayPanelDark`, `flatOver`, and
    /// `accentDrawable` are all `whiteui` texture drawables with different
    /// tints.
    pub fn atlas_symbol(self) -> &'static str {
        let Self::Named(name) = self;
        if name == "window-empty" || name == "window-empty.9" || name == "windowEmpty" {
            "window-empty.9"
        } else if matches!(
            name,
            "black"
                | "black9"
                | "black8"
                | "black6"
                | "black5"
                | "black3"
                | "none"
                | "grayPanel"
                | "grayPanelDark"
                | "flatOver"
                | "accentDrawable"
                | "whiteui"
        ) {
            "whiteui"
        } else {
            name
        }
    }

    pub fn ui_tint(self) -> Option<UiDrawableTint> {
        let Self::Named(name) = self;
        match name {
            "black" => Some(UiDrawableTint::Black),
            "black9" => Some(UiDrawableTint::Black9),
            "black8" => Some(UiDrawableTint::Black8),
            "black6" => Some(UiDrawableTint::Black6),
            "black5" => Some(UiDrawableTint::Black5),
            "black3" => Some(UiDrawableTint::Black3),
            "none" => Some(UiDrawableTint::Transparent),
            "flatOver" => Some(UiDrawableTint::FlatOver),
            "accentDrawable" => Some(UiDrawableTint::Accent),
            "grayPanel" => Some(UiDrawableTint::DarkestGray),
            "grayPanelDark" => Some(UiDrawableTint::DarkestestGray),
            _ => None,
        }
    }

    pub fn default_tint(self) -> [f32; 4] {
        self.ui_tint()
            .map(UiDrawableTint::rgba)
            .unwrap_or([1.0, 1.0, 1.0, 1.0])
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

    pub fn rgba(self) -> [f32; 4] {
        let Self::Named(name) = self;
        if name == "accent" {
            [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a]
        } else if name == "white" {
            [1.0, 1.0, 1.0, 1.0]
        } else if name == "black" {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogTitleStyle {
    pub font: Option<&'static str>,
    pub color: Option<DialogColorRef>,
    pub font_size: Option<f32>,
}

impl Default for DialogTitleStyle {
    fn default() -> Self {
        Self {
            font: None,
            color: None,
            font_size: None,
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
                font_size: Some(24.0),
            },
            accent_line: Some(DialogAccentLineStyle {
                drawable: DialogDrawableRef::named("whiteui"),
                tint: Some(DialogColorRef::named("accent")),
                height: 3.0,
                pad: 4.0,
            }),
        }
    }

    pub fn skin_commands(&self, layout: DialogShellLayout) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        if let Some(stage_background) = self.stage_background {
            commands.push(draw_dialog_drawable(
                stage_background,
                layout.stage_rect,
                None,
                layout.stage_layer,
            ));
        }
        if let Some(background) = self.background {
            commands.push(draw_dialog_drawable(
                background,
                layout.panel_rect,
                None,
                layout.panel_layer,
            ));
        }
        if let Some(accent_line) = &self.accent_line {
            commands.push(draw_dialog_drawable(
                accent_line.drawable,
                layout.accent_line_rect,
                accent_line.tint.map(DialogColorRef::rgba),
                layout.accent_layer,
            ));
        }
        commands
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialogShellLayout {
    pub stage_rect: RenderRect,
    pub panel_rect: RenderRect,
    pub title_position: RenderPoint,
    pub title_font_size: f32,
    pub accent_line_rect: RenderRect,
    pub stage_layer: f32,
    pub panel_layer: f32,
    pub accent_layer: f32,
    pub title_layer: f32,
}

impl DialogShellLayout {
    pub fn from_stage_and_panel(stage_rect: RenderRect, panel_rect: RenderRect) -> Self {
        Self {
            stage_rect,
            panel_rect,
            title_position: RenderPoint::new(
                panel_rect.x + panel_rect.width * 0.5,
                panel_rect.y + panel_rect.height - 38.0,
            ),
            title_font_size: 24.0,
            accent_line_rect: RenderRect::new(
                panel_rect.x + 4.0,
                panel_rect.y + panel_rect.height - 62.0,
                (panel_rect.width - 8.0).max(0.0),
                3.0,
            ),
            stage_layer: Layer::END_PIXELED - 0.03,
            panel_layer: Layer::END_PIXELED,
            accent_layer: Layer::END_PIXELED + 0.01,
            title_layer: Layer::END_PIXELED + 0.02,
        }
    }
}

fn draw_dialog_drawable(
    drawable: DialogDrawableRef,
    rect: RenderRect,
    tint: Option<[f32; 4]>,
    layer: f32,
) -> RenderCommand {
    RenderCommand::draw_sprite(
        drawable.atlas_symbol(),
        rect,
        tint.unwrap_or_else(|| drawable.default_tint()),
        0.0,
        layer,
    )
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

    pub fn shell_render_commands(&self, layout: DialogShellLayout) -> Vec<RenderCommand> {
        let mut commands = self.style.skin_commands(layout);
        if !self.title.is_empty() {
            commands.insert(
                2.min(commands.len()),
                RenderCommand::draw_text_styled(
                    self.title.clone(),
                    layout.title_position,
                    self.style
                        .title
                        .color
                        .map(DialogColorRef::rgba)
                        .unwrap_or([1.0, 1.0, 1.0, 1.0]),
                    self.style.title.font_size.unwrap_or(layout.title_font_size),
                    0.0,
                    RenderTextStyle::new(RenderTextAlign::Center)
                        .with_vertical_align(RenderTextVerticalAlign::Center)
                        .with_integer_position(true)
                        .with_outline(true),
                    layout.title_layer,
                ),
            );
        }
        commands
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
        assert_eq!(style.title.font_size, Some(24.0));
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
        assert_eq!(style.title.font_size, Some(24.0));
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
    fn dialog_style_default_carries_java_title_font_size() {
        let style = DialogStyle::default();

        assert_eq!(
            style.title.font_size,
            Some(24.0),
            "Arc Dialog title Label uses the default Fonts.def scale; the Rust shell keeps that Java-sized value in the title style metadata instead of scattering it in render code"
        );
    }

    #[test]
    fn dialog_style_full_carries_java_title_font_size() {
        let style = DialogStyle::full_dialog();

        assert_eq!(style.title.font_size, Some(24.0));
    }

    #[test]
    fn dialog_drawable_refs_resolve_to_java_tex_atlas_symbols() {
        assert_eq!(
            DialogDrawableRef::named("window-empty").atlas_symbol(),
            "window-empty.9"
        );
        assert_eq!(
            DialogDrawableRef::named("windowEmpty").atlas_symbol(),
            "window-empty.9"
        );
        assert_eq!(
            DialogDrawableRef::named("window-empty.9").atlas_symbol(),
            "window-empty.9"
        );
        assert_eq!(DialogDrawableRef::named("black9").atlas_symbol(), "whiteui");
        assert_eq!(DialogDrawableRef::named("black").atlas_symbol(), "whiteui");
        assert_eq!(
            DialogDrawableRef::named("whiteui").atlas_symbol(),
            "whiteui"
        );
        assert_eq!(
            DialogDrawableRef::named("black9").default_tint(),
            [0.0, 0.0, 0.0, 0.9]
        );
    }

    #[test]
    fn dialog_drawable_refs_cover_common_styles_load_aliases() {
        let cases = [
            (
                "black8",
                "whiteui",
                Some(UiDrawableTint::Black8),
                [0.0, 0.0, 0.0, 0.8],
            ),
            (
                "black6",
                "whiteui",
                Some(UiDrawableTint::Black6),
                [0.0, 0.0, 0.0, 0.6],
            ),
            (
                "black5",
                "whiteui",
                Some(UiDrawableTint::Black5),
                [0.0, 0.0, 0.0, 0.5],
            ),
            (
                "black3",
                "whiteui",
                Some(UiDrawableTint::Black3),
                [0.0, 0.0, 0.0, 0.3],
            ),
            (
                "none",
                "whiteui",
                Some(UiDrawableTint::Transparent),
                [1.0, 1.0, 1.0, 0.0],
            ),
            (
                "grayPanel",
                "whiteui",
                Some(UiDrawableTint::DarkestGray),
                [0.12, 0.13, 0.16, 1.0],
            ),
            (
                "grayPanelDark",
                "whiteui",
                Some(UiDrawableTint::DarkestestGray),
                [0.08, 0.09, 0.11, 1.0],
            ),
            (
                "flatOver",
                "whiteui",
                Some(UiDrawableTint::FlatOver),
                [0.270_588_25, 0.270_588_25, 0.270_588_25, 1.0],
            ),
            (
                "accentDrawable",
                "whiteui",
                Some(UiDrawableTint::Accent),
                [0.48, 0.74, 0.86, 1.0],
            ),
        ];

        for (name, atlas, tint, rgba) in cases {
            let drawable = DialogDrawableRef::named(name);

            assert_eq!(drawable.atlas_symbol(), atlas, "{name}");
            assert_eq!(drawable.ui_tint(), tint, "{name}");
            assert_eq!(drawable.default_tint(), rgba, "{name}");
        }
    }

    #[test]
    fn base_dialog_shell_commands_use_title_font_size_from_style() {
        let mut style = DialogStyle::default();
        style.title.font_size = Some(31.0);
        let dialog = BaseDialog::with_style("@title", style);
        let mut layout = DialogShellLayout::from_stage_and_panel(
            RenderRect::new(0.0, 0.0, 800.0, 600.0),
            RenderRect::new(140.0, 160.0, 520.0, 220.0),
        );
        layout.title_font_size = 11.0;

        let commands = dialog.shell_render_commands(layout);

        let title_size = commands
            .iter()
            .find_map(|command| match command {
                RenderCommand::DrawText { text, size, .. } if text == "@title" => Some(*size),
                _ => None,
            })
            .expect("dialog title should render");
        assert_eq!(title_size, 31.0);
    }

    #[test]
    fn base_dialog_shell_commands_use_window_empty_nine_patch_skin() {
        let dialog = BaseDialog::new("@title");
        let layout = DialogShellLayout::from_stage_and_panel(
            RenderRect::new(0.0, 0.0, 800.0, 600.0),
            RenderRect::new(140.0, 160.0, 520.0, 220.0),
        );

        let commands = dialog.shell_render_commands(layout);

        assert_eq!(commands.len(), 4);
        match &commands[0] {
            RenderCommand::DrawSprite {
                symbol, tint, rect, ..
            } => {
                assert_eq!(symbol, "whiteui");
                assert_eq!(*rect, layout.stage_rect);
                assert_eq!(*tint, [0.0, 0.0, 0.0, 0.9]);
            }
            other => panic!("expected black9 stage sprite, got {other:?}"),
        }
        match &commands[1] {
            RenderCommand::DrawSprite {
                symbol, rect, tint, ..
            } => {
                assert_eq!(symbol, "window-empty.9");
                assert_eq!(*rect, layout.panel_rect);
                assert_eq!(*tint, [1.0, 1.0, 1.0, 1.0]);
            }
            other => panic!("expected window-empty.9 panel sprite, got {other:?}"),
        }
        match &commands[2] {
            RenderCommand::DrawText { text, color, .. } => {
                assert_eq!(text, "@title");
                assert_eq!(
                    *color,
                    [Pal::ACCENT.r, Pal::ACCENT.g, Pal::ACCENT.b, Pal::ACCENT.a]
                );
            }
            other => panic!("expected dialog title text, got {other:?}"),
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
            other => panic!("expected accent whiteui separator sprite, got {other:?}"),
        }
    }

    #[test]
    fn base_dialog_title_separator_uses_java_title_table_pad() {
        let panel = RenderRect::new(140.0, 160.0, 520.0, 220.0);
        let layout =
            DialogShellLayout::from_stage_and_panel(RenderRect::new(0.0, 0.0, 800.0, 600.0), panel);

        assert_eq!(
            layout.accent_line_rect,
            RenderRect::new(
                panel.x + 4.0,
                panel.y + panel.height - 62.0,
                panel.width - 8.0,
                3.0
            ),
            "Java BaseDialog appends titleTable.image(Tex.whiteui, Pal.accent).growX().height(3f).pad(4f), so the separator should nearly span the whole dialog width"
        );
    }

    #[test]
    fn full_dialog_shell_uses_opaque_black_stage_background() {
        let dialog = BaseDialog::with_style("@full", DialogStyle::full_dialog());
        let layout = DialogShellLayout::from_stage_and_panel(
            RenderRect::new(0.0, 0.0, 320.0, 240.0),
            RenderRect::new(20.0, 30.0, 280.0, 180.0),
        );

        let commands = dialog.shell_render_commands(layout);

        match &commands[0] {
            RenderCommand::DrawSprite {
                symbol, tint, rect, ..
            } => {
                assert_eq!(symbol, "whiteui");
                assert_eq!(*rect, layout.stage_rect);
                assert_eq!(*tint, [0.0, 0.0, 0.0, 1.0]);
            }
            other => panic!("expected black stage sprite, got {other:?}"),
        }
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
