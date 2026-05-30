//! Data-oriented skin/style registry mirroring the stable resource contract of
//! upstream `mindustry.ui.Styles`.
//!
//! The Java implementation wires most Scene2D widgets through generated
//! `Tex.*` drawables.  This module keeps the drawable names, atlas symbols and
//! raw source paths in one Rust-visible place so renderers and future widget
//! ports consume the same skin table instead of hard-coding independent sprite
//! lists.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSkinGroup {
    Utility,
    Dialog,
    Pane,
    Button,
    Scroll,
    Slider,
    CheckBox,
    TextField,
    Bar,
    MenuChrome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiSkinSprite {
    pub symbol: &'static str,
    pub source_path: &'static str,
    pub group: UiSkinGroup,
}

impl UiSkinSprite {
    pub const fn new(symbol: &'static str, source_path: &'static str, group: UiSkinGroup) -> Self {
        Self {
            symbol,
            source_path,
            group,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiDrawableTint {
    None,
    Black,
    Black9,
    Black8,
    Black6,
    Black5,
    Black3,
    Transparent,
    FlatOver,
    Accent,
    DarkestGray,
    DarkestestGray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiDrawableAlias {
    pub java_name: &'static str,
    pub atlas_symbol: &'static str,
    pub source_path: &'static str,
    pub tint: UiDrawableTint,
}

impl UiDrawableAlias {
    pub const fn new(
        java_name: &'static str,
        atlas_symbol: &'static str,
        source_path: &'static str,
        tint: UiDrawableTint,
    ) -> Self {
        Self {
            java_name,
            atlas_symbol,
            source_path,
            tint,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiTextButtonStyleSkin {
    pub java_name: &'static str,
    pub up: Option<&'static str>,
    pub down: Option<&'static str>,
    pub over: Option<&'static str>,
    pub checked: Option<&'static str>,
    pub disabled: Option<&'static str>,
    pub font: &'static str,
}

impl UiTextButtonStyleSkin {
    pub const fn new(java_name: &'static str, font: &'static str) -> Self {
        Self {
            java_name,
            up: None,
            down: None,
            over: None,
            checked: None,
            disabled: None,
            font,
        }
    }

    pub const fn up(mut self, drawable: &'static str) -> Self {
        self.up = Some(drawable);
        self
    }

    pub const fn down(mut self, drawable: &'static str) -> Self {
        self.down = Some(drawable);
        self
    }

    pub const fn over(mut self, drawable: &'static str) -> Self {
        self.over = Some(drawable);
        self
    }

    pub const fn checked(mut self, drawable: &'static str) -> Self {
        self.checked = Some(drawable);
        self
    }

    pub const fn disabled(mut self, drawable: &'static str) -> Self {
        self.disabled = Some(drawable);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiImageButtonStyleSkin {
    pub java_name: &'static str,
    pub up: Option<&'static str>,
    pub down: Option<&'static str>,
    pub over: Option<&'static str>,
    pub checked: Option<&'static str>,
    pub disabled: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiScrollPaneStyleSkin {
    pub java_name: &'static str,
    pub v_scroll: Option<&'static str>,
    pub v_scroll_knob: Option<&'static str>,
    pub h_scroll: Option<&'static str>,
    pub h_scroll_knob: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiSliderStyleSkin {
    pub java_name: &'static str,
    pub background: &'static str,
    pub knob: &'static str,
    pub knob_over: &'static str,
    pub knob_down: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiCheckBoxStyleSkin {
    pub java_name: &'static str,
    pub checkbox_on: &'static str,
    pub checkbox_off: &'static str,
    pub checkbox_on_over: &'static str,
    pub checkbox_over: &'static str,
    pub checkbox_on_disabled: &'static str,
    pub checkbox_off_disabled: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UiTextFieldStyleSkin {
    pub java_name: &'static str,
    pub background: &'static str,
    pub disabled_background: Option<&'static str>,
    pub invalid_background: Option<&'static str>,
    pub selection: &'static str,
    pub cursor: &'static str,
}

pub const UPSTREAM_UI_SKIN_SPRITES: &[UiSkinSprite] = &[
    UiSkinSprite::new("whiteui", "sprites/ui/whiteui.png", UiSkinGroup::Utility),
    UiSkinSprite::new("clear", "sprites/ui/clear.png", UiSkinGroup::Utility),
    UiSkinSprite::new(
        "selection",
        "sprites/ui/selection.png",
        UiSkinGroup::TextField,
    ),
    UiSkinSprite::new("cursor", "sprites/ui/cursor.png", UiSkinGroup::TextField),
    UiSkinSprite::new(
        "window-empty.9",
        "sprites/ui/window-empty.9.png",
        UiSkinGroup::Dialog,
    ),
    UiSkinSprite::new("pane.9", "sprites/ui/pane.9.png", UiSkinGroup::Pane),
    UiSkinSprite::new("pane-2.9", "sprites/ui/pane-2.9.png", UiSkinGroup::Pane),
    UiSkinSprite::new(
        "pane-left.9",
        "sprites/ui/pane-left.9.png",
        UiSkinGroup::Pane,
    ),
    UiSkinSprite::new(
        "pane-right.9",
        "sprites/ui/pane-right.9.png",
        UiSkinGroup::Pane,
    ),
    UiSkinSprite::new(
        "pane-solid.9",
        "sprites/ui/pane-solid.9.png",
        UiSkinGroup::Pane,
    ),
    UiSkinSprite::new("pane-top.9", "sprites/ui/pane-top.9.png", UiSkinGroup::Pane),
    UiSkinSprite::new(
        "white-pane.9",
        "sprites/ui/white-pane.9.png",
        UiSkinGroup::Pane,
    ),
    UiSkinSprite::new("wavepane.9", "sprites/ui/wavepane.9.png", UiSkinGroup::Pane),
    UiSkinSprite::new(
        "inventory.9",
        "sprites/ui/inventory.9.png",
        UiSkinGroup::Pane,
    ),
    UiSkinSprite::new("button.9", "sprites/ui/button.9.png", UiSkinGroup::Button),
    UiSkinSprite::new(
        "button-down.9",
        "sprites/ui/button-down.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-over.9",
        "sprites/ui/button-over.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-disabled.9",
        "sprites/ui/button-disabled.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-red.9",
        "sprites/ui/button-red.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-right.9",
        "sprites/ui/button-right.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-right-down.9",
        "sprites/ui/button-right-down.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-right-over.9",
        "sprites/ui/button-right-over.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-right-disabled.9",
        "sprites/ui/button-right-disabled.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-select.9",
        "sprites/ui/button-select.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-select-trans.9",
        "sprites/ui/button-select-trans.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-side-left.9",
        "sprites/ui/button-side-left.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-side-left-down.9",
        "sprites/ui/button-side-left-down.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-side-left-over.9",
        "sprites/ui/button-side-left-over.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-side-right.9",
        "sprites/ui/button-side-right.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-side-right-down.9",
        "sprites/ui/button-side-right-down.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-side-right-over.9",
        "sprites/ui/button-side-right-over.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-trans.9",
        "sprites/ui/button-trans.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-1.9",
        "sprites/ui/button-edge-1.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-2.9",
        "sprites/ui/button-edge-2.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-3.9",
        "sprites/ui/button-edge-3.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-4.9",
        "sprites/ui/button-edge-4.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-down-1.9",
        "sprites/ui/button-edge-down-1.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-down-3.9",
        "sprites/ui/button-edge-down-3.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-over-1.9",
        "sprites/ui/button-edge-over-1.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-over-3.9",
        "sprites/ui/button-edge-over-3.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "button-edge-over-4.9",
        "sprites/ui/button-edge-over-4.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "flat-down-base.9",
        "sprites/ui/flat-down-base.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "sideline.9",
        "sprites/ui/sideline.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new(
        "sideline-over.9",
        "sprites/ui/sideline-over.9.png",
        UiSkinGroup::Button,
    ),
    UiSkinSprite::new("scroll.9", "sprites/ui/scroll.9.png", UiSkinGroup::Scroll),
    UiSkinSprite::new(
        "scroll-horizontal.9",
        "sprites/ui/scroll-horizontal.9.png",
        UiSkinGroup::Scroll,
    ),
    UiSkinSprite::new(
        "scroll-knob-horizontal-black",
        "sprites/ui/scroll-knob-horizontal-black.png",
        UiSkinGroup::Scroll,
    ),
    UiSkinSprite::new(
        "scroll-knob-vertical-black",
        "sprites/ui/scroll-knob-vertical-black.png",
        UiSkinGroup::Scroll,
    ),
    UiSkinSprite::new(
        "scroll-knob-vertical-thin",
        "sprites/ui/scroll-knob-vertical-thin.png",
        UiSkinGroup::Scroll,
    ),
    UiSkinSprite::new("slider", "sprites/ui/slider.png", UiSkinGroup::Slider),
    UiSkinSprite::new(
        "slider-back.9",
        "sprites/ui/slider-back.9.png",
        UiSkinGroup::Slider,
    ),
    UiSkinSprite::new(
        "slider-knob",
        "sprites/ui/slider-knob.png",
        UiSkinGroup::Slider,
    ),
    UiSkinSprite::new(
        "slider-knob-down",
        "sprites/ui/slider-knob-down.png",
        UiSkinGroup::Slider,
    ),
    UiSkinSprite::new(
        "slider-knob-over",
        "sprites/ui/slider-knob-over.png",
        UiSkinGroup::Slider,
    ),
    UiSkinSprite::new("check-on", "sprites/ui/check-on.png", UiSkinGroup::CheckBox),
    UiSkinSprite::new(
        "check-off",
        "sprites/ui/check-off.png",
        UiSkinGroup::CheckBox,
    ),
    UiSkinSprite::new(
        "check-on-over",
        "sprites/ui/check-on-over.png",
        UiSkinGroup::CheckBox,
    ),
    UiSkinSprite::new(
        "check-over",
        "sprites/ui/check-over.png",
        UiSkinGroup::CheckBox,
    ),
    UiSkinSprite::new(
        "check-on-disabled",
        "sprites/ui/check-on-disabled.png",
        UiSkinGroup::CheckBox,
    ),
    UiSkinSprite::new(
        "check-disabled",
        "sprites/ui/check-disabled.png",
        UiSkinGroup::CheckBox,
    ),
    UiSkinSprite::new(
        "underline.9",
        "sprites/ui/underline.9.png",
        UiSkinGroup::TextField,
    ),
    UiSkinSprite::new(
        "underline-2.9",
        "sprites/ui/underline-2.9.png",
        UiSkinGroup::TextField,
    ),
    UiSkinSprite::new(
        "underline-disabled.9",
        "sprites/ui/underline-disabled.9.png",
        UiSkinGroup::TextField,
    ),
    UiSkinSprite::new(
        "underline-over.9",
        "sprites/ui/underline-over.9.png",
        UiSkinGroup::TextField,
    ),
    UiSkinSprite::new(
        "underline-red.9",
        "sprites/ui/underline-red.9.png",
        UiSkinGroup::TextField,
    ),
    UiSkinSprite::new(
        "underline-white.9",
        "sprites/ui/underline-white.9.png",
        UiSkinGroup::TextField,
    ),
    UiSkinSprite::new("bar.9", "sprites/ui/bar.9.png", UiSkinGroup::Bar),
    UiSkinSprite::new("bar-top.9", "sprites/ui/bar-top.9.png", UiSkinGroup::Bar),
    UiSkinSprite::new(
        "discord-banner",
        "sprites/ui/discord-banner.png",
        UiSkinGroup::MenuChrome,
    ),
    UiSkinSprite::new(
        "info-banner",
        "sprites/ui/info-banner.png",
        UiSkinGroup::MenuChrome,
    ),
];

pub const UPSTREAM_UI_DRAWABLE_ALIASES: &[UiDrawableAlias] = &[
    UiDrawableAlias::new(
        "whiteui",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "black",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Black,
    ),
    UiDrawableAlias::new(
        "black9",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Black9,
    ),
    UiDrawableAlias::new(
        "black8",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Black8,
    ),
    UiDrawableAlias::new(
        "black6",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Black6,
    ),
    UiDrawableAlias::new(
        "black5",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Black5,
    ),
    UiDrawableAlias::new(
        "black3",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Black3,
    ),
    UiDrawableAlias::new(
        "none",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Transparent,
    ),
    UiDrawableAlias::new(
        "flatOver",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::FlatOver,
    ),
    UiDrawableAlias::new(
        "accentDrawable",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::Accent,
    ),
    UiDrawableAlias::new(
        "grayPanel",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::DarkestGray,
    ),
    UiDrawableAlias::new(
        "grayPanelDark",
        "whiteui",
        "sprites/ui/whiteui.png",
        UiDrawableTint::DarkestestGray,
    ),
    UiDrawableAlias::new(
        "windowEmpty",
        "window-empty.9",
        "sprites/ui/window-empty.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "pane",
        "pane.9",
        "sprites/ui/pane.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "flatDown",
        "flat-down-base.9",
        "sprites/ui/flat-down-base.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "flatDownBase",
        "flat-down-base.9",
        "sprites/ui/flat-down-base.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "button",
        "button.9",
        "sprites/ui/button.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "buttonDown",
        "button-down.9",
        "sprites/ui/button-down.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "buttonOver",
        "button-over.9",
        "sprites/ui/button-over.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "buttonDisabled",
        "button-disabled.9",
        "sprites/ui/button-disabled.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "buttonSelect",
        "button-select.9",
        "sprites/ui/button-select.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "scroll",
        "scroll.9",
        "sprites/ui/scroll.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "scrollHorizontal",
        "scroll-horizontal.9",
        "sprites/ui/scroll-horizontal.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "scrollKnobVerticalBlack",
        "scroll-knob-vertical-black",
        "sprites/ui/scroll-knob-vertical-black.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "scrollKnobHorizontalBlack",
        "scroll-knob-horizontal-black",
        "sprites/ui/scroll-knob-horizontal-black.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "scrollKnobVerticalThin",
        "scroll-knob-vertical-thin",
        "sprites/ui/scroll-knob-vertical-thin.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "sliderBack",
        "slider-back.9",
        "sprites/ui/slider-back.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "sliderKnob",
        "slider-knob",
        "sprites/ui/slider-knob.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "sliderKnobOver",
        "slider-knob-over",
        "sprites/ui/slider-knob-over.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "sliderKnobDown",
        "slider-knob-down",
        "sprites/ui/slider-knob-down.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "checkOn",
        "check-on",
        "sprites/ui/check-on.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "checkOff",
        "check-off",
        "sprites/ui/check-off.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "checkOnOver",
        "check-on-over",
        "sprites/ui/check-on-over.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "checkOver",
        "check-over",
        "sprites/ui/check-over.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "checkOnDisabled",
        "check-on-disabled",
        "sprites/ui/check-on-disabled.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "checkDisabled",
        "check-disabled",
        "sprites/ui/check-disabled.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "underline",
        "underline.9",
        "sprites/ui/underline.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "underlineWhite",
        "underline-white.9",
        "sprites/ui/underline-white.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "underlineDisabled",
        "underline-disabled.9",
        "sprites/ui/underline-disabled.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "underlineOver",
        "underline-over.9",
        "sprites/ui/underline-over.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "underlineRed",
        "underline-red.9",
        "sprites/ui/underline-red.9.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "selection",
        "selection",
        "sprites/ui/selection.png",
        UiDrawableTint::None,
    ),
    UiDrawableAlias::new(
        "cursor",
        "cursor",
        "sprites/ui/cursor.png",
        UiDrawableTint::None,
    ),
];

pub const UPSTREAM_TEXT_BUTTON_STYLE_SKINS: &[UiTextButtonStyleSkin] = &[
    UiTextButtonStyleSkin::new("defaultt", "Fonts.def")
        .up("button")
        .down("buttonDown")
        .over("buttonOver")
        .disabled("buttonDisabled"),
    UiTextButtonStyleSkin::new("flatBordert", "Fonts.def")
        .up("pane")
        .down("flatOver")
        .over("flatDownBase"),
    UiTextButtonStyleSkin::new("flatToggleMenut", "Fonts.def")
        .up("clear")
        .down("flatDown")
        .checked("flatDown")
        .over("flatOver")
        .disabled("black"),
    UiTextButtonStyleSkin::new("togglet", "Fonts.def")
        .up("button")
        .down("buttonDown")
        .checked("buttonDown")
        .over("buttonOver")
        .disabled("buttonDisabled"),
    UiTextButtonStyleSkin::new("clearTogglet", "Fonts.def")
        .up("black6")
        .down("flatDown")
        .checked("flatDown")
        .over("flatOver")
        .disabled("black"),
    UiTextButtonStyleSkin::new("squareTogglet", "Fonts.def")
        .up("pane")
        .down("flatOver")
        .checked("flatOver")
        .over("flatOver")
        .disabled("black"),
];

pub const UPSTREAM_IMAGE_BUTTON_STYLE_SKINS: &[UiImageButtonStyleSkin] = &[
    UiImageButtonStyleSkin {
        java_name: "defaulti",
        up: Some("button"),
        down: Some("buttonDown"),
        over: Some("buttonOver"),
        checked: None,
        disabled: Some("buttonDisabled"),
    },
    UiImageButtonStyleSkin {
        java_name: "selecti",
        up: Some("none"),
        down: None,
        over: None,
        checked: Some("buttonSelect"),
        disabled: None,
    },
    UiImageButtonStyleSkin {
        java_name: "squarei",
        up: Some("pane"),
        down: Some("whiteui"),
        over: Some("flatDown"),
        checked: None,
        disabled: None,
    },
    UiImageButtonStyleSkin {
        java_name: "squareTogglei",
        up: Some("black"),
        down: Some("flatDown"),
        over: Some("flatOver"),
        checked: Some("flatDown"),
        disabled: None,
    },
    UiImageButtonStyleSkin {
        java_name: "clearNoneTogglei",
        up: Some("none"),
        down: Some("flatDown"),
        over: Some("flatOver"),
        checked: Some("flatDown"),
        disabled: None,
    },
];

pub const UPSTREAM_SCROLL_PANE_STYLE_SKINS: &[UiScrollPaneStyleSkin] = &[
    UiScrollPaneStyleSkin {
        java_name: "defaultPane",
        v_scroll: Some("scroll"),
        v_scroll_knob: Some("scrollKnobVerticalBlack"),
        h_scroll: None,
        h_scroll_knob: None,
    },
    UiScrollPaneStyleSkin {
        java_name: "horizontalPane",
        v_scroll: Some("scroll"),
        v_scroll_knob: Some("scrollKnobVerticalBlack"),
        h_scroll: Some("scrollHorizontal"),
        h_scroll_knob: Some("scrollKnobHorizontalBlack"),
    },
    UiScrollPaneStyleSkin {
        java_name: "smallPane",
        v_scroll: Some("clear"),
        v_scroll_knob: Some("scrollKnobVerticalThin"),
        h_scroll: None,
        h_scroll_knob: None,
    },
    UiScrollPaneStyleSkin {
        java_name: "noBarPane",
        v_scroll: None,
        v_scroll_knob: None,
        h_scroll: None,
        h_scroll_knob: None,
    },
];

pub const UPSTREAM_SLIDER_STYLE_SKINS: &[UiSliderStyleSkin] = &[UiSliderStyleSkin {
    java_name: "defaultSlider",
    background: "sliderBack",
    knob: "sliderKnob",
    knob_over: "sliderKnobOver",
    knob_down: "sliderKnobDown",
}];

pub const UPSTREAM_CHECK_BOX_STYLE_SKINS: &[UiCheckBoxStyleSkin] = &[UiCheckBoxStyleSkin {
    java_name: "defaultCheck",
    checkbox_on: "checkOn",
    checkbox_off: "checkOff",
    checkbox_on_over: "checkOnOver",
    checkbox_over: "checkOver",
    checkbox_on_disabled: "checkOnDisabled",
    checkbox_off_disabled: "checkDisabled",
}];

pub const UPSTREAM_TEXT_FIELD_STYLE_SKINS: &[UiTextFieldStyleSkin] = &[
    UiTextFieldStyleSkin {
        java_name: "defaultField",
        background: "underline",
        disabled_background: Some("underlineDisabled"),
        invalid_background: Some("underlineRed"),
        selection: "selection",
        cursor: "cursor",
    },
    UiTextFieldStyleSkin {
        java_name: "nodeField",
        background: "underlineWhite",
        disabled_background: Some("underlineDisabled"),
        invalid_background: Some("underlineRed"),
        selection: "selection",
        cursor: "cursor",
    },
    UiTextFieldStyleSkin {
        java_name: "areaField",
        background: "underline",
        disabled_background: None,
        invalid_background: None,
        selection: "selection",
        cursor: "cursor",
    },
    UiTextFieldStyleSkin {
        java_name: "nodeArea",
        background: "underlineWhite",
        disabled_background: None,
        invalid_background: None,
        selection: "selection",
        cursor: "cursor",
    },
];

pub fn upstream_ui_skin_sprites() -> &'static [UiSkinSprite] {
    UPSTREAM_UI_SKIN_SPRITES
}

pub fn upstream_ui_skin_sprite_source_paths() -> impl Iterator<Item = &'static str> {
    UPSTREAM_UI_SKIN_SPRITES
        .iter()
        .map(|sprite| sprite.source_path)
}

pub fn upstream_ui_skin_sprite(symbol: &str) -> Option<&'static UiSkinSprite> {
    UPSTREAM_UI_SKIN_SPRITES
        .iter()
        .find(|sprite| sprite.symbol == symbol)
}

pub fn upstream_ui_drawable_alias(java_name: &str) -> Option<&'static UiDrawableAlias> {
    UPSTREAM_UI_DRAWABLE_ALIASES
        .iter()
        .find(|alias| alias.java_name == java_name)
}

pub fn upstream_text_button_style_skin(java_name: &str) -> Option<&'static UiTextButtonStyleSkin> {
    UPSTREAM_TEXT_BUTTON_STYLE_SKINS
        .iter()
        .find(|style| style.java_name == java_name)
}

pub fn upstream_image_button_style_skin(
    java_name: &str,
) -> Option<&'static UiImageButtonStyleSkin> {
    UPSTREAM_IMAGE_BUTTON_STYLE_SKINS
        .iter()
        .find(|style| style.java_name == java_name)
}

pub fn upstream_scroll_pane_style_skin(java_name: &str) -> Option<&'static UiScrollPaneStyleSkin> {
    UPSTREAM_SCROLL_PANE_STYLE_SKINS
        .iter()
        .find(|style| style.java_name == java_name)
}

pub fn upstream_slider_style_skin(java_name: &str) -> Option<&'static UiSliderStyleSkin> {
    UPSTREAM_SLIDER_STYLE_SKINS
        .iter()
        .find(|style| style.java_name == java_name)
}

pub fn upstream_check_box_style_skin(java_name: &str) -> Option<&'static UiCheckBoxStyleSkin> {
    UPSTREAM_CHECK_BOX_STYLE_SKINS
        .iter()
        .find(|style| style.java_name == java_name)
}

pub fn upstream_text_field_style_skin(java_name: &str) -> Option<&'static UiTextFieldStyleSkin> {
    UPSTREAM_TEXT_FIELD_STYLE_SKINS
        .iter()
        .find(|style| style.java_name == java_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_ui_skin_registry_exposes_core_scene2d_resources() {
        for symbol in [
            "window-empty.9",
            "pane.9",
            "button.9",
            "button-down.9",
            "button-over.9",
            "button-edge-2.9",
            "inventory.9",
            "scroll.9",
            "slider-back.9",
            "check-on",
            "underline.9",
            "bar.9",
            "discord-banner",
        ] {
            assert!(
                upstream_ui_skin_sprite(symbol).is_some(),
                "{symbol} should be in the upstream UI skin registry"
            );
        }
    }

    #[test]
    fn upstream_drawable_aliases_capture_styles_load_tinted_whiteui_contract() {
        let black9 = upstream_ui_drawable_alias("black9").unwrap();
        assert_eq!(black9.atlas_symbol, "whiteui");
        assert_eq!(black9.source_path, "sprites/ui/whiteui.png");
        assert_eq!(black9.tint, UiDrawableTint::Black9);

        let window = upstream_ui_drawable_alias("windowEmpty").unwrap();
        assert_eq!(window.atlas_symbol, "window-empty.9");
        assert_eq!(window.source_path, "sprites/ui/window-empty.9.png");
        assert_eq!(window.tint, UiDrawableTint::None);
    }

    #[test]
    fn upstream_text_button_style_skins_match_java_styles_names() {
        let defaultt = upstream_text_button_style_skin("defaultt").unwrap();
        assert_eq!(defaultt.up, Some("button"));
        assert_eq!(defaultt.down, Some("buttonDown"));
        assert_eq!(defaultt.over, Some("buttonOver"));
        assert_eq!(defaultt.disabled, Some("buttonDisabled"));

        let menu = upstream_text_button_style_skin("flatToggleMenut").unwrap();
        assert_eq!(menu.up, Some("clear"));
        assert_eq!(menu.down, Some("flatDown"));
        assert_eq!(menu.checked, Some("flatDown"));
        assert_eq!(menu.over, Some("flatOver"));
        assert_eq!(menu.disabled, Some("black"));
    }

    #[test]
    fn upstream_widget_style_skins_match_java_scroll_slider_check_and_field_names() {
        let horizontal = upstream_scroll_pane_style_skin("horizontalPane").unwrap();
        assert_eq!(horizontal.v_scroll, Some("scroll"));
        assert_eq!(horizontal.v_scroll_knob, Some("scrollKnobVerticalBlack"));
        assert_eq!(horizontal.h_scroll, Some("scrollHorizontal"));
        assert_eq!(horizontal.h_scroll_knob, Some("scrollKnobHorizontalBlack"));

        let slider = upstream_slider_style_skin("defaultSlider").unwrap();
        assert_eq!(slider.background, "sliderBack");
        assert_eq!(slider.knob, "sliderKnob");
        assert_eq!(slider.knob_over, "sliderKnobOver");
        assert_eq!(slider.knob_down, "sliderKnobDown");

        let check = upstream_check_box_style_skin("defaultCheck").unwrap();
        assert_eq!(check.checkbox_on, "checkOn");
        assert_eq!(check.checkbox_off, "checkOff");
        assert_eq!(check.checkbox_on_disabled, "checkOnDisabled");
        assert_eq!(check.checkbox_off_disabled, "checkDisabled");

        let field = upstream_text_field_style_skin("defaultField").unwrap();
        assert_eq!(field.background, "underline");
        assert_eq!(field.disabled_background, Some("underlineDisabled"));
        assert_eq!(field.invalid_background, Some("underlineRed"));
        assert_eq!(field.selection, "selection");
        assert_eq!(field.cursor, "cursor");
    }

    #[test]
    fn upstream_image_button_style_skins_match_java_image_button_names() {
        let defaulti = upstream_image_button_style_skin("defaulti").unwrap();
        assert_eq!(defaulti.up, Some("button"));
        assert_eq!(defaulti.down, Some("buttonDown"));
        assert_eq!(defaulti.over, Some("buttonOver"));
        assert_eq!(defaulti.disabled, Some("buttonDisabled"));

        let square = upstream_image_button_style_skin("squarei").unwrap();
        assert_eq!(square.up, Some("pane"));
        assert_eq!(square.down, Some("whiteui"));
        assert_eq!(square.over, Some("flatDown"));
    }

    #[test]
    fn upstream_ui_skin_source_paths_are_virtual_asset_paths() {
        let paths = upstream_ui_skin_sprite_source_paths().collect::<Vec<_>>();
        assert!(paths.contains(&"sprites/ui/button.9.png"));
        assert!(paths.contains(&"sprites/ui/window-empty.9.png"));
        assert!(paths.contains(&"sprites/ui/whiteui.png"));
        assert!(paths.iter().all(|path| path.starts_with("sprites/ui/")));
    }

    #[test]
    fn upstream_ui_skin_registry_pins_the_common_java_styles_drawables() {
        for (symbol, source_path) in [
            ("button.9", "sprites/ui/button.9.png"),
            ("button-down.9", "sprites/ui/button-down.9.png"),
            ("button-over.9", "sprites/ui/button-over.9.png"),
            ("button-disabled.9", "sprites/ui/button-disabled.9.png"),
            ("pane.9", "sprites/ui/pane.9.png"),
            ("window-empty.9", "sprites/ui/window-empty.9.png"),
            ("whiteui", "sprites/ui/whiteui.png"),
            ("scroll.9", "sprites/ui/scroll.9.png"),
            ("slider", "sprites/ui/slider.png"),
            ("check-on", "sprites/ui/check-on.png"),
            ("underline.9", "sprites/ui/underline.9.png"),
            ("bar.9", "sprites/ui/bar.9.png"),
        ] {
            let sprite = upstream_ui_skin_sprite(symbol)
                .unwrap_or_else(|| panic!("missing {symbol} in the upstream UI skin registry"));
            assert_eq!(sprite.source_path, source_path);
            assert!(
                sprite.source_path.starts_with("sprites/ui/"),
                "{symbol} must stay on the virtual ui asset path"
            );
        }
    }
}
