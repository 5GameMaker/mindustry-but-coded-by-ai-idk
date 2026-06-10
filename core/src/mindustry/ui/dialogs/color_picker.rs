//! Color picker dialog model mirroring upstream `mindustry.ui.dialogs.ColorPicker`.

pub const COLOR_PICKER_TITLE: &str = "@pickcolor";
pub const COLOR_PICKER_HUE_TEXTURE_WIDTH: i32 = 128;
pub const COLOR_PICKER_HUE_TEXTURE_HEIGHT: i32 = 1;
pub const COLOR_PICKER_HUE_TEXTURE_FILTER: &str = "TextureFilter.linear";
pub const COLOR_PICKER_PREVIEW_BACKGROUND: &str = "Tex.pane";
pub const COLOR_PICKER_ALPHA_BACKGROUND: &str = "Tex.alphaBg";
pub const COLOR_PICKER_ALPHA_LINE_BACKGROUND: &str = "Tex.alphaBgLine";
pub const COLOR_PICKER_PREVIEW_SIZE: f32 = 200.0;
pub const COLOR_PICKER_PREVIEW_PAD_BOTTOM: f32 = 5.0;
pub const COLOR_PICKER_SLIDER_PAD_BOTTOM: f32 = 6.0;
pub const COLOR_PICKER_SLIDER_WIDTH: f32 = 370.0;
pub const COLOR_PICKER_SLIDER_HEIGHT: f32 = 44.0;
pub const COLOR_PICKER_HUE_MIN: f32 = 0.0;
pub const COLOR_PICKER_HUE_MAX: f32 = 360.0;
pub const COLOR_PICKER_HUE_STEP: f32 = 0.3;
pub const COLOR_PICKER_UNIT_MIN: f32 = 0.0;
pub const COLOR_PICKER_UNIT_MAX: f32 = 1.0;
pub const COLOR_PICKER_UNIT_STEP: f32 = 0.001;
pub const COLOR_PICKER_HEX_FIELD_SIZE: (f32, f32) = (130.0, 40.0);
pub const COLOR_PICKER_OK_TEXT: &str = "@ok";
pub const COLOR_PICKER_OK_ICON: &str = "ok";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPickerColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorPickerColor {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub const fn from_rgba8888(rgba: u32) -> Self {
        Self {
            r: ((rgba >> 24) & 0xff) as f32 / 255.0,
            g: ((rgba >> 16) & 0xff) as f32 / 255.0,
            b: ((rgba >> 8) & 0xff) as f32 / 255.0,
            a: (rgba & 0xff) as f32 / 255.0,
        }
    }

    pub fn from_hex_like_java(hex: &str) -> Option<Self> {
        let chars = hex.chars().collect::<Vec<_>>();
        let offset = if chars.first().copied() == Some('#') {
            1
        } else {
            0
        };
        let r = parse_hex_like_java(&chars, offset, offset + 2)?;
        let g = parse_hex_like_java(&chars, offset + 2, offset + 4)?;
        let b = parse_hex_like_java(&chars, offset + 4, offset + 6)?;
        let a = if chars.len() - offset != 8 {
            255
        } else {
            parse_hex_like_java(&chars, offset + 6, offset + 8)?
        };
        Some(Self::new(
            (r as f32 / 255.0).clamp(0.0, 1.0),
            (g as f32 / 255.0).clamp(0.0, 1.0),
            (b as f32 / 255.0).clamp(0.0, 1.0),
            (a as f32 / 255.0).clamp(0.0, 1.0),
        ))
    }

    pub fn to_hsv(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let range = max - min;
        let h = if range == 0.0 {
            0.0
        } else if max == self.r {
            (60.0 * (self.g - self.b) / range + 360.0) % 360.0
        } else if max == self.g {
            60.0 * (self.b - self.r) / range + 120.0
        } else {
            60.0 * (self.r - self.g) / range + 240.0
        };
        let s = if max > 0.0 { 1.0 - min / max } else { 0.0 };
        (h, s, max)
    }

    pub fn from_hsv(h: f32, s: f32, v: f32, a: f32) -> Self {
        let x = (h / 60.0 + 6.0) % 6.0;
        let i = x as i32;
        let f = x - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));
        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        Self::new(
            r.clamp(0.0, 1.0),
            g.clamp(0.0, 1.0),
            b.clamp(0.0, 1.0),
            a.clamp(0.0, 1.0),
        )
    }

    pub fn to_java_hex(self) -> String {
        format!(
            "{:08x}",
            ((component_to_byte(self.r) as u32) << 24)
                | ((component_to_byte(self.g) as u32) << 16)
                | ((component_to_byte(self.b) as u32) << 8)
                | component_to_byte(self.a) as u32
        )
    }

    pub fn to_field_hex_after_update(self) -> String {
        let value = self.to_java_hex();
        if self.a >= 0.9999 {
            value[..6].to_string()
        } else {
            value
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerSliderKind {
    Hue,
    Saturation,
    Value,
    Alpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPickerSliderBackground {
    HueTexture,
    SaturationGradient,
    ValueGradient,
    AlphaGradient,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerSliderModel {
    pub kind: ColorPickerSliderKind,
    pub background: ColorPickerSliderBackground,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub value: f32,
    pub vertical: bool,
    pub width: f32,
    pub height: f32,
    pub pad_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerModel {
    pub title: &'static str,
    pub current: ColorPickerColor,
    pub preview: ColorPickerPreviewModel,
    pub sliders: Vec<ColorPickerSliderModel>,
    pub hex_field: ColorPickerHexFieldModel,
    pub close_button_added: bool,
    pub ok_button: ColorPickerButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerPreviewModel {
    pub pane_background: &'static str,
    pub alpha_background: &'static str,
    pub size: f32,
    pub pad_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerHexFieldModel {
    pub text: String,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerButton {
    pub text: &'static str,
    pub icon: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerUpdate {
    pub current: ColorPickerColor,
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
    pub hex_field_text: Option<String>,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorPickerAction {
    ShowDialog,
    CreateHueTexture { width: i32, height: i32 },
    SetHueTextureFilter { filter: &'static str },
    AcceptColor { color: ColorPickerColor },
    HideDialog,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPicker {
    pub hue_texture_loaded: bool,
    pub current: ColorPickerColor,
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
    pub alpha: bool,
    pub hex_field_text: String,
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            hue_texture_loaded: false,
            current: ColorPickerColor::new(0.0, 0.0, 0.0, 0.0),
            h: 0.0,
            s: 0.0,
            v: 0.0,
            a: 0.0,
            alpha: true,
            hex_field_text: String::new(),
        }
    }

    pub fn show(&mut self, color: ColorPickerColor) -> (Vec<ColorPickerAction>, ColorPickerModel) {
        self.show_with_alpha(color, true)
    }

    pub fn show_with_alpha(
        &mut self,
        color: ColorPickerColor,
        alpha: bool,
    ) -> (Vec<ColorPickerAction>, ColorPickerModel) {
        self.current = color;
        self.alpha = alpha;
        let (h, s, v) = color.to_hsv();
        self.h = h;
        self.s = s;
        self.v = v;
        self.a = color.a;
        self.hex_field_text = color.to_java_hex();

        let mut actions = vec![ColorPickerAction::ShowDialog];
        if !self.hue_texture_loaded {
            self.hue_texture_loaded = true;
            actions.extend([
                ColorPickerAction::CreateHueTexture {
                    width: COLOR_PICKER_HUE_TEXTURE_WIDTH,
                    height: COLOR_PICKER_HUE_TEXTURE_HEIGHT,
                },
                ColorPickerAction::SetHueTextureFilter {
                    filter: COLOR_PICKER_HUE_TEXTURE_FILTER,
                },
            ]);
        }
        (actions, self.model())
    }

    pub fn model(&self) -> ColorPickerModel {
        ColorPickerModel {
            title: COLOR_PICKER_TITLE,
            current: self.current,
            preview: ColorPickerPreviewModel {
                pane_background: COLOR_PICKER_PREVIEW_BACKGROUND,
                alpha_background: COLOR_PICKER_ALPHA_BACKGROUND,
                size: COLOR_PICKER_PREVIEW_SIZE,
                pad_bottom: COLOR_PICKER_PREVIEW_PAD_BOTTOM,
            },
            sliders: self.slider_models(),
            hex_field: ColorPickerHexFieldModel {
                text: self.hex_field_text.clone(),
                size: COLOR_PICKER_HEX_FIELD_SIZE,
            },
            close_button_added: true,
            ok_button: ColorPickerButton {
                text: COLOR_PICKER_OK_TEXT,
                icon: COLOR_PICKER_OK_ICON,
            },
        }
    }

    pub fn move_slider(&mut self, kind: ColorPickerSliderKind, value: f32) -> ColorPickerUpdate {
        match kind {
            ColorPickerSliderKind::Hue => self.h = value,
            ColorPickerSliderKind::Saturation => self.s = value,
            ColorPickerSliderKind::Value => self.v = value,
            ColorPickerSliderKind::Alpha => self.a = value,
        }
        self.update_color(true)
    }

    pub fn hex_changed(&mut self, value: &str) -> ColorPickerUpdate {
        if let Some(color) = ColorPickerColor::from_hex_like_java(value) {
            self.current = color;
            let (h, s, v) = color.to_hsv();
            self.h = h;
            self.s = s;
            self.v = v;
            self.a = color.a;
            self.update_color(false)
        } else {
            ColorPickerUpdate {
                current: self.current,
                h: self.h,
                s: self.s,
                v: self.v,
                a: self.a,
                hex_field_text: None,
                accepted: false,
            }
        }
    }

    pub fn valid_hex(text: &str) -> bool {
        ColorPickerColor::from_hex_like_java(text).is_some()
    }

    pub fn ok_plan(&self) -> Vec<ColorPickerAction> {
        vec![
            ColorPickerAction::AcceptColor {
                color: self.current,
            },
            ColorPickerAction::HideDialog,
        ]
    }

    fn update_color(&mut self, update_field: bool) -> ColorPickerUpdate {
        self.current = ColorPickerColor::from_hsv(self.h, self.s, self.v, self.a);
        let hex_field_text = if update_field {
            let text = self.current.to_field_hex_after_update();
            self.hex_field_text = text.clone();
            Some(text)
        } else {
            None
        };
        ColorPickerUpdate {
            current: self.current,
            h: self.h,
            s: self.s,
            v: self.v,
            a: self.a,
            hex_field_text,
            accepted: true,
        }
    }

    fn slider_models(&self) -> Vec<ColorPickerSliderModel> {
        let mut sliders = vec![
            slider_model(
                ColorPickerSliderKind::Hue,
                ColorPickerSliderBackground::HueTexture,
                COLOR_PICKER_HUE_MIN,
                COLOR_PICKER_HUE_MAX,
                COLOR_PICKER_HUE_STEP,
                self.h,
            ),
            slider_model(
                ColorPickerSliderKind::Saturation,
                ColorPickerSliderBackground::SaturationGradient,
                COLOR_PICKER_UNIT_MIN,
                COLOR_PICKER_UNIT_MAX,
                COLOR_PICKER_UNIT_STEP,
                self.s,
            ),
            slider_model(
                ColorPickerSliderKind::Value,
                ColorPickerSliderBackground::ValueGradient,
                COLOR_PICKER_UNIT_MIN,
                COLOR_PICKER_UNIT_MAX,
                COLOR_PICKER_UNIT_STEP,
                self.v,
            ),
        ];
        if self.alpha {
            sliders.push(slider_model(
                ColorPickerSliderKind::Alpha,
                ColorPickerSliderBackground::AlphaGradient,
                COLOR_PICKER_UNIT_MIN,
                COLOR_PICKER_UNIT_MAX,
                COLOR_PICKER_UNIT_STEP,
                self.a,
            ));
        }
        sliders
    }
}

fn slider_model(
    kind: ColorPickerSliderKind,
    background: ColorPickerSliderBackground,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
) -> ColorPickerSliderModel {
    ColorPickerSliderModel {
        kind,
        background,
        min,
        max,
        step,
        value,
        vertical: false,
        width: COLOR_PICKER_SLIDER_WIDTH,
        height: COLOR_PICKER_SLIDER_HEIGHT,
        pad_bottom: COLOR_PICKER_SLIDER_PAD_BOTTOM,
    }
}

fn parse_hex_like_java(chars: &[char], from: usize, to: usize) -> Option<i32> {
    if to > chars.len() {
        return None;
    }
    let mut total = 0;
    for (index, ch) in chars.iter().enumerate().take(to).skip(from) {
        let digit = ch.to_digit(16).map(|digit| digit as i32).unwrap_or(-1);
        total += digit * if index == from { 16 } else { 1 };
    }
    Some(total)
}

fn component_to_byte(component: f32) -> u8 {
    (255.0 * component) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_color_close(actual: ColorPickerColor, expected: ColorPickerColor) {
        assert_close(actual.r, expected.r);
        assert_close(actual.g, expected.g);
        assert_close(actual.b, expected.b);
        assert_close(actual.a, expected.a);
    }

    #[test]
    fn show_initializes_hsv_hex_model_and_lazy_hue_texture_like_java() {
        let mut picker = ColorPicker::new();
        let (actions, model) = picker.show(ColorPickerColor::from_rgba8888(0x336699ff));

        assert_eq!(
            actions,
            vec![
                ColorPickerAction::ShowDialog,
                ColorPickerAction::CreateHueTexture {
                    width: 128,
                    height: 1,
                },
                ColorPickerAction::SetHueTextureFilter {
                    filter: "TextureFilter.linear",
                },
            ]
        );
        assert_eq!(model.title, "@pickcolor");
        assert_eq!(model.preview.pane_background, "Tex.pane");
        assert_eq!(model.preview.alpha_background, "Tex.alphaBg");
        assert_eq!(model.preview.size, 200.0);
        assert_eq!(model.sliders.len(), 4);
        assert_eq!(model.sliders[0].kind, ColorPickerSliderKind::Hue);
        assert_eq!(
            model.sliders[0].background,
            ColorPickerSliderBackground::HueTexture
        );
        assert_eq!(model.sliders[0].min, 0.0);
        assert_eq!(model.sliders[0].max, 360.0);
        assert_eq!(model.sliders[0].step, 0.3);
        assert_eq!(model.sliders[1].step, 0.001);
        assert_eq!(model.hex_field.text, "336699ff");
        assert_eq!(model.hex_field.size, (130.0, 40.0));
        assert!(model.close_button_added);
        assert_eq!(model.ok_button.text, "@ok");
        assert_eq!(model.ok_button.icon, "ok");
        assert_close(picker.h, 210.0);
        assert_close(picker.s, 2.0 / 3.0);
        assert_close(picker.v, 0.6);
        assert_close(picker.a, 1.0);

        let (actions, no_alpha) =
            picker.show_with_alpha(ColorPickerColor::from_rgba8888(0xff000080), false);
        assert_eq!(actions, vec![ColorPickerAction::ShowDialog]);
        assert_eq!(no_alpha.sliders.len(), 3);
        assert!(!no_alpha
            .sliders
            .iter()
            .any(|slider| slider.kind == ColorPickerSliderKind::Alpha));
        assert_eq!(no_alpha.hex_field.text, "ff000080");
    }

    #[test]
    fn slider_moves_update_color_and_hex_field_with_six_digit_full_alpha_rule() {
        let mut picker = ColorPicker::new();
        picker.show(ColorPickerColor::from_rgba8888(0xff0000ff));

        let update = picker.move_slider(ColorPickerSliderKind::Hue, 120.0);
        assert_color_close(update.current, ColorPickerColor::from_rgba8888(0x00ff00ff));
        assert_eq!(update.hex_field_text, Some("00ff00".into()));
        assert_eq!(picker.hex_field_text, "00ff00");

        let update = picker.move_slider(ColorPickerSliderKind::Alpha, 0.5);
        assert_eq!(update.hex_field_text, Some("00ff007f".into()));
        assert_eq!(picker.hex_field_text, "00ff007f");
        assert_close(picker.a, 0.5);
    }

    #[test]
    fn hex_changes_parse_like_arc_color_update_sliders_but_not_field_text() {
        let mut picker = ColorPicker::new();
        picker.show(ColorPickerColor::white());
        picker.hex_field_text = "unchanged".into();

        let update = picker.hex_changed("#33669980");

        assert!(update.accepted);
        assert_eq!(update.hex_field_text, None);
        assert_eq!(picker.hex_field_text, "unchanged");
        assert_color_close(picker.current, ColorPickerColor::from_rgba8888(0x33669980));
        assert_close(picker.h, 210.0);
        assert_close(picker.s, 2.0 / 3.0);
        assert_close(picker.v, 0.6);
        assert_close(picker.a, 128.0 / 255.0);
        assert!(ColorPicker::valid_hex("336699"));
        assert!(ColorPicker::valid_hex("zzzzzz"));
        assert!(!ColorPicker::valid_hex("12345"));
    }

    #[test]
    fn ok_button_submits_current_color_then_hides_dialog() {
        let mut picker = ColorPicker::new();
        picker.show(ColorPickerColor::from_rgba8888(0x11223344));

        assert_eq!(
            picker.ok_plan(),
            vec![
                ColorPickerAction::AcceptColor {
                    color: ColorPickerColor::from_rgba8888(0x11223344),
                },
                ColorPickerAction::HideDialog,
            ]
        );
    }
}
