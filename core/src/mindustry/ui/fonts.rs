//! Data-oriented font and content-icon registry mirroring the stable resource
//! contract of upstream `mindustry.ui.Fonts`.
//!
//! This module intentionally stops before rasterization/OpenGL work: it records
//! the real upstream font asset names, source paths and icon-property parsing so
//! the desktop renderer can later replace the `DrawText` placeholder with a
//! FreeType/icon-atlas backed pipeline without rediscovering Java-side names.

use crate::mindustry::graphics::RenderFontId;
use std::{char, fmt, num::ParseIntError};

pub const UPSTREAM_MAIN_FONT_SOURCE_PATH: &str = "fonts/font.woff";
pub const UPSTREAM_ICON_FONT_SOURCE_PATH: &str = "fonts/icon.ttf";
pub const UPSTREAM_LOGIC_FONT_SOURCE_PATH: &str = "fonts/logic.ttf";
pub const UPSTREAM_MONOSPACE_FONT_SOURCE_PATH: &str = "fonts/monospace.woff";
pub const UPSTREAM_TECH_FONT_SOURCE_PATH: &str = "fonts/tech.ttf";
pub const UPSTREAM_JAPANESE_FONT_SOURCE_PATH: &str = "fonts/font_jp.woff";
pub const UPSTREAM_ICONS_PROPERTIES_SOURCE_PATH: &str = "icons/icons.properties";
pub const UPSTREAM_UI_ICON_FONTGEN_CONFIG_SOURCE_PATH: &str = "fontgen/config.json";

pub const UPSTREAM_LOGIC_FONT_CHARACTERS: &str =
    "\0ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890\"!`?'.,;:()[]{}<>|/@\\^$€-%+=#_&~*";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpstreamFontRole {
    Default,
    Outline,
    Icon,
    IconLarge,
    Tech,
    Logic,
    Monospace,
    JapaneseDefaultOverride,
    JapaneseOutlineOverride,
}

impl UpstreamFontRole {
    pub const fn java_static_name(self) -> &'static str {
        match self {
            Self::Default => "Fonts.def",
            Self::Outline => "Fonts.outline",
            Self::Icon => "Fonts.icon",
            Self::IconLarge => "Fonts.iconLarge",
            Self::Tech => "Fonts.tech",
            Self::Logic => "Fonts.logic",
            Self::Monospace => "Fonts.monospace",
            Self::JapaneseDefaultOverride => "Fonts.def.data.override",
            Self::JapaneseOutlineOverride => "Fonts.outline.data.override",
        }
    }

    pub const fn render_font_id(self) -> Option<RenderFontId> {
        match self {
            Self::Default | Self::JapaneseDefaultOverride => Some(RenderFontId::Default),
            Self::Outline | Self::JapaneseOutlineOverride => Some(RenderFontId::Outline),
            Self::Icon => Some(RenderFontId::Icon),
            Self::IconLarge => Some(RenderFontId::IconLarge),
            Self::Logic => Some(RenderFontId::Logic),
            Self::Tech => Some(RenderFontId::Tech),
            Self::Monospace => Some(RenderFontId::Monospace),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpstreamFontAsset {
    pub role: UpstreamFontRole,
    pub asset_name: &'static str,
    pub source_path: &'static str,
    pub size: u16,
    pub incremental: bool,
    pub scaled: bool,
    pub border_width: Option<u16>,
    pub shadow_offset_y: Option<i16>,
    pub characters: Option<&'static str>,
    pub fallback_java_static_name: Option<&'static str>,
}

impl UpstreamFontAsset {
    pub const fn render_font_id(self) -> Option<RenderFontId> {
        self.role.render_font_id()
    }
}

pub const UPSTREAM_FONT_ASSETS: &[UpstreamFontAsset] = &[
    UpstreamFontAsset {
        role: UpstreamFontRole::Outline,
        asset_name: "outline",
        source_path: UPSTREAM_MAIN_FONT_SOURCE_PATH,
        size: 18,
        incremental: true,
        scaled: true,
        border_width: Some(2),
        shadow_offset_y: None,
        characters: None,
        fallback_java_static_name: None,
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::Tech,
        asset_name: "tech",
        source_path: UPSTREAM_TECH_FONT_SOURCE_PATH,
        size: 18,
        incremental: false,
        scaled: true,
        border_width: None,
        shadow_offset_y: None,
        characters: None,
        fallback_java_static_name: None,
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::Default,
        asset_name: "default",
        source_path: UPSTREAM_MAIN_FONT_SOURCE_PATH,
        size: 18,
        incremental: true,
        scaled: true,
        border_width: None,
        shadow_offset_y: Some(2),
        characters: None,
        fallback_java_static_name: None,
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::Monospace,
        asset_name: "monospace",
        source_path: UPSTREAM_MONOSPACE_FONT_SOURCE_PATH,
        size: 16,
        incremental: true,
        scaled: true,
        border_width: None,
        shadow_offset_y: None,
        characters: Some("\0 "),
        fallback_java_static_name: Some("Fonts.def"),
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::Icon,
        asset_name: "icon",
        source_path: UPSTREAM_ICON_FONT_SOURCE_PATH,
        size: 30,
        incremental: true,
        scaled: true,
        border_width: None,
        shadow_offset_y: None,
        characters: Some("\0"),
        fallback_java_static_name: None,
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::IconLarge,
        asset_name: "iconLarge",
        source_path: UPSTREAM_ICON_FONT_SOURCE_PATH,
        size: 48,
        incremental: false,
        scaled: false,
        border_width: Some(5),
        shadow_offset_y: None,
        characters: None,
        fallback_java_static_name: None,
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::Logic,
        asset_name: "logic",
        source_path: UPSTREAM_LOGIC_FONT_SOURCE_PATH,
        size: 16,
        incremental: false,
        scaled: false,
        border_width: None,
        shadow_offset_y: None,
        characters: Some(UPSTREAM_LOGIC_FONT_CHARACTERS),
        fallback_java_static_name: None,
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::JapaneseDefaultOverride,
        asset_name: "font_jp",
        source_path: UPSTREAM_JAPANESE_FONT_SOURCE_PATH,
        size: 18,
        incremental: true,
        scaled: true,
        border_width: None,
        shadow_offset_y: Some(2),
        characters: Some("\0 "),
        fallback_java_static_name: None,
    },
    UpstreamFontAsset {
        role: UpstreamFontRole::JapaneseOutlineOverride,
        asset_name: "font_jp_outline",
        source_path: UPSTREAM_JAPANESE_FONT_SOURCE_PATH,
        size: 18,
        incremental: true,
        scaled: true,
        border_width: Some(2),
        shadow_offset_y: None,
        characters: Some("\0 "),
        fallback_java_static_name: None,
    },
];

pub fn upstream_font_assets() -> &'static [UpstreamFontAsset] {
    UPSTREAM_FONT_ASSETS
}

pub fn upstream_font_asset(role: UpstreamFontRole) -> Option<&'static UpstreamFontAsset> {
    UPSTREAM_FONT_ASSETS.iter().find(|asset| asset.role == role)
}

pub fn upstream_font_asset_by_name(asset_name: &str) -> Option<&'static UpstreamFontAsset> {
    UPSTREAM_FONT_ASSETS
        .iter()
        .find(|asset| asset.asset_name == asset_name)
}

pub fn upstream_font_source_paths() -> impl Iterator<Item = &'static str> {
    UPSTREAM_FONT_ASSETS
        .iter()
        .map(|asset| asset.source_path)
        .chain([
            UPSTREAM_ICONS_PROPERTIES_SOURCE_PATH,
            UPSTREAM_UI_ICON_FONTGEN_CONFIG_SOURCE_PATH,
        ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpstreamUiIconGlyph {
    pub java_name: &'static str,
    pub css_name: &'static str,
    pub codepoint: u32,
}

impl UpstreamUiIconGlyph {
    pub fn glyph_char(self) -> Option<char> {
        char::from_u32(self.codepoint)
    }

    pub fn glyph_string(self) -> Option<String> {
        self.glyph_char().map(|glyph| glyph.to_string())
    }
}

pub const UPSTREAM_UI_ICON_GLYPHS: &[UpstreamUiIconGlyph] = &[
    UpstreamUiIconGlyph {
        java_name: "fileTextFill",
        css_name: "file-text-fill",
        codepoint: 0xf15c,
    },
    UpstreamUiIconGlyph {
        java_name: "file",
        css_name: "file",
        codepoint: 0xf15b,
    },
    UpstreamUiIconGlyph {
        java_name: "fileText",
        css_name: "file-text",
        codepoint: 0xf0f6,
    },
    UpstreamUiIconGlyph {
        java_name: "left",
        css_name: "left",
        codepoint: 0xe802,
    },
    UpstreamUiIconGlyph {
        java_name: "right",
        css_name: "right",
        codepoint: 0xe803,
    },
    UpstreamUiIconGlyph {
        java_name: "up",
        css_name: "up",
        codepoint: 0xe804,
    },
    UpstreamUiIconGlyph {
        java_name: "down",
        css_name: "down",
        codepoint: 0xe805,
    },
    UpstreamUiIconGlyph {
        java_name: "home",
        css_name: "home",
        codepoint: 0xe807,
    },
    UpstreamUiIconGlyph {
        java_name: "ok",
        css_name: "ok",
        codepoint: 0xe800,
    },
    UpstreamUiIconGlyph {
        java_name: "image",
        css_name: "image",
        codepoint: 0xe808,
    },
    UpstreamUiIconGlyph {
        java_name: "star",
        css_name: "star",
        codepoint: 0xe809,
    },
    UpstreamUiIconGlyph {
        java_name: "resize",
        css_name: "resize",
        codepoint: 0xe80b,
    },
    UpstreamUiIconGlyph {
        java_name: "wrench",
        css_name: "wrench",
        codepoint: 0xe80f,
    },
    UpstreamUiIconGlyph {
        java_name: "githubSquare",
        css_name: "github-square",
        codepoint: 0xf300,
    },
    UpstreamUiIconGlyph {
        java_name: "fileImage",
        css_name: "file-image",
        codepoint: 0xf1c5,
    },
    UpstreamUiIconGlyph {
        java_name: "add",
        css_name: "add",
        codepoint: 0xe813,
    },
    UpstreamUiIconGlyph {
        java_name: "edit",
        css_name: "edit",
        codepoint: 0xe816,
    },
    UpstreamUiIconGlyph {
        java_name: "chartBar",
        css_name: "chart-bar",
        codepoint: 0xe819,
    },
    UpstreamUiIconGlyph {
        java_name: "planeOutline",
        css_name: "plane-outline",
        codepoint: 0xe81a,
    },
    UpstreamUiIconGlyph {
        java_name: "filter",
        css_name: "filter",
        codepoint: 0xf0b0,
    },
    UpstreamUiIconGlyph {
        java_name: "folder",
        css_name: "folder",
        codepoint: 0xe81d,
    },
    UpstreamUiIconGlyph {
        java_name: "steam",
        css_name: "steam",
        codepoint: 0xe822,
    },
    UpstreamUiIconGlyph {
        java_name: "downOpen",
        css_name: "down-open",
        codepoint: 0xe824,
    },
    UpstreamUiIconGlyph {
        java_name: "leftOpen",
        css_name: "left-open",
        codepoint: 0xe825,
    },
    UpstreamUiIconGlyph {
        java_name: "upOpen",
        css_name: "up-open",
        codepoint: 0xe826,
    },
    UpstreamUiIconGlyph {
        java_name: "map",
        css_name: "map",
        codepoint: 0xe827,
    },
    UpstreamUiIconGlyph {
        java_name: "rotate",
        css_name: "rotate",
        codepoint: 0xe823,
    },
    UpstreamUiIconGlyph {
        java_name: "play",
        css_name: "play",
        codepoint: 0xe829,
    },
    UpstreamUiIconGlyph {
        java_name: "pause",
        css_name: "pause",
        codepoint: 0xe806,
    },
    UpstreamUiIconGlyph {
        java_name: "list",
        css_name: "list",
        codepoint: 0xe811,
    },
    UpstreamUiIconGlyph {
        java_name: "cancel",
        css_name: "cancel",
        codepoint: 0xe815,
    },
    UpstreamUiIconGlyph {
        java_name: "move",
        css_name: "move",
        codepoint: 0xe818,
    },
    UpstreamUiIconGlyph {
        java_name: "terminal",
        css_name: "terminal",
        codepoint: 0xf120,
    },
    UpstreamUiIconGlyph {
        java_name: "undo",
        css_name: "undo",
        codepoint: 0xe835,
    },
    UpstreamUiIconGlyph {
        java_name: "redo",
        css_name: "redo",
        codepoint: 0xe836,
    },
    UpstreamUiIconGlyph {
        java_name: "info",
        css_name: "info",
        codepoint: 0xf129,
    },
    UpstreamUiIconGlyph {
        java_name: "infoCircle",
        css_name: "info-circle",
        codepoint: 0xe837,
    },
    UpstreamUiIconGlyph {
        java_name: "rightOpenOut",
        css_name: "right-open-out",
        codepoint: 0xe839,
    },
    UpstreamUiIconGlyph {
        java_name: "rightOpen",
        css_name: "right-open",
        codepoint: 0xe83a,
    },
    UpstreamUiIconGlyph {
        java_name: "waves",
        css_name: "waves",
        codepoint: 0xe83b,
    },
    UpstreamUiIconGlyph {
        java_name: "filters",
        css_name: "filters",
        codepoint: 0xe83e,
    },
    UpstreamUiIconGlyph {
        java_name: "layers",
        css_name: "layers",
        codepoint: 0xe83f,
    },
    UpstreamUiIconGlyph {
        java_name: "eraser",
        css_name: "eraser",
        codepoint: 0xf12d,
    },
    UpstreamUiIconGlyph {
        java_name: "bookOpen",
        css_name: "book-open",
        codepoint: 0xe801,
    },
    UpstreamUiIconGlyph {
        java_name: "grid",
        css_name: "grid",
        codepoint: 0xf029,
    },
    UpstreamUiIconGlyph {
        java_name: "flipX",
        css_name: "flip-x",
        codepoint: 0xe812,
    },
    UpstreamUiIconGlyph {
        java_name: "flipY",
        css_name: "flip-y",
        codepoint: 0xe842,
    },
    UpstreamUiIconGlyph {
        java_name: "diagonal",
        css_name: "diagonal",
        codepoint: 0xe844,
    },
    UpstreamUiIconGlyph {
        java_name: "discord",
        css_name: "discord_",
        codepoint: 0xe80d,
    },
    UpstreamUiIconGlyph {
        java_name: "box",
        css_name: "box",
        codepoint: 0xe81e,
    },
    UpstreamUiIconGlyph {
        java_name: "redditAlien",
        css_name: "reddit-alien",
        codepoint: 0xf281,
    },
    UpstreamUiIconGlyph {
        java_name: "github",
        css_name: "github_",
        codepoint: 0xf308,
    },
    UpstreamUiIconGlyph {
        java_name: "googleplay",
        css_name: "googleplay",
        codepoint: 0xe83d,
    },
    UpstreamUiIconGlyph {
        java_name: "android",
        css_name: "android",
        codepoint: 0xe845,
    },
    UpstreamUiIconGlyph {
        java_name: "trello",
        css_name: "trello",
        codepoint: 0xf181,
    },
    UpstreamUiIconGlyph {
        java_name: "logic",
        css_name: "logic",
        codepoint: 0xe80e,
    },
    UpstreamUiIconGlyph {
        java_name: "distribution",
        css_name: "distribution",
        codepoint: 0xe814,
    },
    UpstreamUiIconGlyph {
        java_name: "hammer",
        css_name: "hammer",
        codepoint: 0xe817,
    },
    UpstreamUiIconGlyph {
        java_name: "save",
        css_name: "save",
        codepoint: 0xe81b,
    },
    UpstreamUiIconGlyph {
        java_name: "link",
        css_name: "link",
        codepoint: 0xe81c,
    },
    UpstreamUiIconGlyph {
        java_name: "itchio",
        css_name: "itchio",
        codepoint: 0xe82a,
    },
    UpstreamUiIconGlyph {
        java_name: "line",
        css_name: "line",
        codepoint: 0xe82b,
    },
    UpstreamUiIconGlyph {
        java_name: "admin",
        css_name: "admin",
        codepoint: 0xe82c,
    },
    UpstreamUiIconGlyph {
        java_name: "spray1",
        css_name: "spray-1",
        codepoint: 0xe82d,
    },
    UpstreamUiIconGlyph {
        java_name: "crafting",
        css_name: "crafting",
        codepoint: 0xe830,
    },
    UpstreamUiIconGlyph {
        java_name: "fill",
        css_name: "fill",
        codepoint: 0xe84c,
    },
    UpstreamUiIconGlyph {
        java_name: "paste",
        css_name: "paste",
        codepoint: 0xe852,
    },
    UpstreamUiIconGlyph {
        java_name: "effect",
        css_name: "effect",
        codepoint: 0xe853,
    },
    UpstreamUiIconGlyph {
        java_name: "book",
        css_name: "book",
        codepoint: 0xe85b,
    },
    UpstreamUiIconGlyph {
        java_name: "liquid",
        css_name: "liquid",
        codepoint: 0xe85c,
    },
    UpstreamUiIconGlyph {
        java_name: "host",
        css_name: "host",
        codepoint: 0xe85d,
    },
    UpstreamUiIconGlyph {
        java_name: "production",
        css_name: "production",
        codepoint: 0xe85e,
    },
    UpstreamUiIconGlyph {
        java_name: "exit",
        css_name: "exit",
        codepoint: 0xe85f,
    },
    UpstreamUiIconGlyph {
        java_name: "modePvp",
        css_name: "mode-pvp",
        codepoint: 0xe861,
    },
    UpstreamUiIconGlyph {
        java_name: "modeAttack",
        css_name: "mode-attack",
        codepoint: 0xe865,
    },
    UpstreamUiIconGlyph {
        java_name: "refresh1",
        css_name: "refresh-1",
        codepoint: 0xe867,
    },
    UpstreamUiIconGlyph {
        java_name: "none",
        css_name: "none",
        codepoint: 0xe868,
    },
    UpstreamUiIconGlyph {
        java_name: "pencil",
        css_name: "pencil_",
        codepoint: 0xe869,
    },
    UpstreamUiIconGlyph {
        java_name: "refresh",
        css_name: "refresh",
        codepoint: 0xe86a,
    },
    UpstreamUiIconGlyph {
        java_name: "modeSurvival",
        css_name: "mode-survival",
        codepoint: 0xe86b,
    },
    UpstreamUiIconGlyph {
        java_name: "commandRally",
        css_name: "command-rally",
        codepoint: 0xe86c,
    },
    UpstreamUiIconGlyph {
        java_name: "units",
        css_name: "units",
        codepoint: 0xe86d,
    },
    UpstreamUiIconGlyph {
        java_name: "commandAttack",
        css_name: "command-attack",
        codepoint: 0xe86e,
    },
    UpstreamUiIconGlyph {
        java_name: "trash",
        css_name: "trash",
        codepoint: 0xe86f,
    },
    UpstreamUiIconGlyph {
        java_name: "chat",
        css_name: "chat",
        codepoint: 0xe870,
    },
    UpstreamUiIconGlyph {
        java_name: "turret",
        css_name: "turret",
        codepoint: 0xe871,
    },
    UpstreamUiIconGlyph {
        java_name: "players",
        css_name: "players",
        codepoint: 0xe872,
    },
    UpstreamUiIconGlyph {
        java_name: "editor",
        css_name: "editor",
        codepoint: 0xe873,
    },
    UpstreamUiIconGlyph {
        java_name: "copy",
        css_name: "copy",
        codepoint: 0xe874,
    },
    UpstreamUiIconGlyph {
        java_name: "tree",
        css_name: "tree",
        codepoint: 0xe875,
    },
    UpstreamUiIconGlyph {
        java_name: "lockOpen",
        css_name: "lock-open",
        codepoint: 0xe876,
    },
    UpstreamUiIconGlyph {
        java_name: "pick",
        css_name: "pick",
        codepoint: 0xe877,
    },
    UpstreamUiIconGlyph {
        java_name: "export",
        css_name: "export",
        codepoint: 0xe878,
    },
    UpstreamUiIconGlyph {
        java_name: "download",
        css_name: "download",
        codepoint: 0xe879,
    },
    UpstreamUiIconGlyph {
        java_name: "upload",
        css_name: "upload",
        codepoint: 0xe87b,
    },
    UpstreamUiIconGlyph {
        java_name: "settings",
        css_name: "settings",
        codepoint: 0xe87c,
    },
    UpstreamUiIconGlyph {
        java_name: "spray",
        css_name: "spray",
        codepoint: 0xe87d,
    },
    UpstreamUiIconGlyph {
        java_name: "zoom",
        css_name: "zoom",
        codepoint: 0xe88a,
    },
    UpstreamUiIconGlyph {
        java_name: "powerOld",
        css_name: "power_old",
        codepoint: 0xe88b,
    },
    UpstreamUiIconGlyph {
        java_name: "power",
        css_name: "power_",
        codepoint: 0xe810,
    },
    UpstreamUiIconGlyph {
        java_name: "menu",
        css_name: "menu",
        codepoint: 0xe88c,
    },
    UpstreamUiIconGlyph {
        java_name: "lock",
        css_name: "lock",
        codepoint: 0xe88d,
    },
    UpstreamUiIconGlyph {
        java_name: "eye",
        css_name: "eye",
        codepoint: 0xe88e,
    },
    UpstreamUiIconGlyph {
        java_name: "eyeOff",
        css_name: "eye-off",
        codepoint: 0xe88f,
    },
    UpstreamUiIconGlyph {
        java_name: "warning",
        css_name: "warning",
        codepoint: 0x26a0,
    },
    UpstreamUiIconGlyph {
        java_name: "terrain",
        css_name: "terrain",
        codepoint: 0xe864,
    },
    UpstreamUiIconGlyph {
        java_name: "defense",
        css_name: "defense",
        codepoint: 0xe84d,
    },
    UpstreamUiIconGlyph {
        java_name: "planet",
        css_name: "planet",
        codepoint: 0xe833,
    },
    UpstreamUiIconGlyph {
        java_name: "arrowNote",
        css_name: "arrow-note",
        codepoint: 0xe834,
    },
];

pub fn upstream_ui_icon_glyph(name: &str) -> Option<&'static UpstreamUiIconGlyph> {
    UPSTREAM_UI_ICON_GLYPHS
        .iter()
        .find(|glyph| glyph.java_name == name || glyph.css_name == name)
}

pub fn upstream_ui_icon_glyph_char(name: &str) -> Option<char> {
    upstream_ui_icon_glyph(name).and_then(|glyph| glyph.glyph_char())
}

pub fn upstream_ui_icon_glyph_string(name: &str) -> Option<String> {
    upstream_ui_icon_glyph(name).and_then(|glyph| glyph.glyph_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpstreamContentIcon {
    pub unicode: u32,
    pub name: String,
    pub atlas_symbol: String,
}

impl UpstreamContentIcon {
    pub fn emoji_string(&self) -> Option<String> {
        char::from_u32(self.unicode).map(|ch| ch.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconPropertiesParseError {
    MissingEquals {
        line: usize,
    },
    MissingIconName {
        line: usize,
    },
    MissingAtlasSymbol {
        line: usize,
    },
    InvalidUnicode {
        line: usize,
        value: String,
        source: ParseIntError,
    },
}

impl fmt::Display for IconPropertiesParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEquals { line } => {
                write!(f, "icons.properties line {line} is missing '='")
            }
            Self::MissingIconName { line } => {
                write!(f, "icons.properties line {line} is missing an icon name")
            }
            Self::MissingAtlasSymbol { line } => {
                write!(f, "icons.properties line {line} is missing an atlas symbol")
            }
            Self::InvalidUnicode { line, value, .. } => {
                write!(
                    f,
                    "icons.properties line {line} has invalid unicode value '{value}'"
                )
            }
        }
    }
}

impl std::error::Error for IconPropertiesParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUnicode { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub fn parse_upstream_icon_properties(
    contents: &str,
) -> Result<Vec<UpstreamContentIcon>, IconPropertiesParseError> {
    let mut icons = Vec::new();

    for (line_index, raw_line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (unicode, payload) = line
            .split_once('=')
            .ok_or(IconPropertiesParseError::MissingEquals { line: line_number })?;
        let unicode = unicode.trim();
        let unicode =
            unicode
                .parse::<u32>()
                .map_err(|source| IconPropertiesParseError::InvalidUnicode {
                    line: line_number,
                    value: unicode.to_string(),
                    source,
                })?;

        let (name, atlas_symbol) = payload
            .split_once('|')
            .ok_or(IconPropertiesParseError::MissingAtlasSymbol { line: line_number })?;
        let name = name.trim();
        let atlas_symbol = atlas_symbol.trim();
        if name.is_empty() {
            return Err(IconPropertiesParseError::MissingIconName { line: line_number });
        }
        if atlas_symbol.is_empty() {
            return Err(IconPropertiesParseError::MissingAtlasSymbol { line: line_number });
        }

        icons.push(UpstreamContentIcon {
            unicode,
            name: name.to_string(),
            atlas_symbol: atlas_symbol.to_string(),
        });
    }

    Ok(icons)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_font_registry_matches_fonts_java_asset_names_and_paths() {
        let default = upstream_font_asset(UpstreamFontRole::Default).unwrap();
        assert_eq!(default.asset_name, "default");
        assert_eq!(default.source_path, "fonts/font.woff");
        assert_eq!(default.size, 18);
        assert!(default.incremental);
        assert_eq!(default.shadow_offset_y, Some(2));
        assert_eq!(default.render_font_id(), Some(RenderFontId::Default));

        let outline = upstream_font_asset_by_name("outline").unwrap();
        assert_eq!(outline.role, UpstreamFontRole::Outline);
        assert_eq!(outline.source_path, "fonts/font.woff");
        assert_eq!(outline.border_width, Some(2));
        assert_eq!(outline.render_font_id(), Some(RenderFontId::Outline));

        let logic = upstream_font_asset(UpstreamFontRole::Logic).unwrap();
        assert_eq!(logic.source_path, "fonts/logic.ttf");
        assert!(!logic.incremental);
        assert!(!logic.scaled);
        assert_eq!(logic.characters, Some(UPSTREAM_LOGIC_FONT_CHARACTERS));
        assert_eq!(logic.render_font_id(), Some(RenderFontId::Logic));
    }

    #[test]
    fn upstream_font_registry_exposes_icon_and_extra_font_resources() {
        let icon = upstream_font_asset_by_name("icon").unwrap();
        assert_eq!(icon.source_path, "fonts/icon.ttf");
        assert_eq!(icon.characters, Some("\0"));
        assert!(icon.scaled);
        assert_eq!(icon.render_font_id(), Some(RenderFontId::Icon));

        let icon_large = upstream_font_asset_by_name("iconLarge").unwrap();
        assert_eq!(icon_large.source_path, "fonts/icon.ttf");
        assert_eq!(icon_large.size, 48);
        assert_eq!(icon_large.border_width, Some(5));
        assert!(!icon_large.scaled);
        assert_eq!(icon_large.render_font_id(), Some(RenderFontId::IconLarge));

        let monospace = upstream_font_asset_by_name("monospace").unwrap();
        assert_eq!(monospace.source_path, "fonts/monospace.woff");
        assert_eq!(monospace.fallback_java_static_name, Some("Fonts.def"));
        assert_eq!(monospace.render_font_id(), Some(RenderFontId::Monospace));

        let tech = upstream_font_asset_by_name("tech").unwrap();
        assert_eq!(tech.source_path, "fonts/tech.ttf");
        assert_eq!(tech.render_font_id(), Some(RenderFontId::Tech));

        let paths = upstream_font_source_paths().collect::<Vec<_>>();
        assert!(paths.contains(&"fonts/font.woff"));
        assert!(paths.contains(&"fonts/font_jp.woff"));
        assert!(paths.contains(&"fonts/icon.ttf"));
        assert!(paths.contains(&"fonts/logic.ttf"));
        assert!(paths.contains(&"fonts/monospace.woff"));
        assert!(paths.contains(&"fonts/tech.ttf"));
        assert!(paths.contains(&"icons/icons.properties"));
        assert!(paths.contains(&"fontgen/config.json"));
    }

    #[test]
    fn upstream_ui_icon_glyph_registry_covers_all_config_glyphs_and_aliases() {
        assert_eq!(UPSTREAM_UI_ICON_GLYPHS.len(), 109);

        for (name, expected) in [
            ("cancel", 0xe815),
            ("downOpen", 0xe824),
            ("ok", 0xe800),
            ("left", 0xe802),
            ("upOpen", 0xe826),
            ("trash", 0xe86f),
            ("zoom", 0xe88a),
            ("copy", 0xe874),
            ("pencil", 0xe869),
            ("edit", 0xe816),
            ("upload", 0xe87b),
            ("save", 0xe81b),
            ("warning", 0x26a0),
            ("export", 0xe878),
            ("units", 0xe86d),
            ("filter", 0xf0b0),
            ("lock", 0xe88d),
            ("file", 0xf15b),
            ("pause", 0xe806),
        ] {
            let glyph = upstream_ui_icon_glyph(name).expect("icon glyph should be registered");
            assert_eq!(glyph.codepoint, expected);
            assert_eq!(upstream_ui_icon_glyph_char(name), char::from_u32(expected));
            assert_eq!(
                upstream_ui_icon_glyph_string(name),
                char::from_u32(expected).map(|ch| ch.to_string())
            );
        }

        for (name, expected) in [
            ("down-open", 0xe824),
            ("up-open", 0xe826),
            ("left-open", 0xe825),
            ("right-open-out", 0xe839),
            ("file-text", 0xf0f6),
            ("file-image", 0xf1c5),
            ("file-text-fill", 0xf15c),
            ("pencil_", 0xe869),
            ("power_", 0xe810),
            ("power_old", 0xe88b),
            ("discord_", 0xe80d),
            ("github_", 0xf308),
            ("reddit-alien", 0xf281),
        ] {
            let glyph = upstream_ui_icon_glyph(name).expect("css icon glyph should be registered");
            assert_eq!(glyph.codepoint, expected);
        }

        assert_eq!(
            upstream_ui_icon_glyph("reddit-alien").map(|glyph| glyph.java_name),
            Some("redditAlien")
        );
        assert_eq!(
            upstream_ui_icon_glyph("down-open").map(|glyph| glyph.java_name),
            Some("downOpen")
        );
        assert_eq!(upstream_ui_icon_glyph_char("missing-icon"), None);
        assert_eq!(upstream_ui_icon_glyph_string("missing-icon"), None);
    }

    #[test]
    fn parse_upstream_icon_properties_reads_java_content_icon_rows() {
        let icons = parse_upstream_icon_properties(
            "\
63743=spawn|block-spawn-ui
63684=ore-copper|block-ore-copper-ui
",
        )
        .unwrap();

        assert_eq!(icons.len(), 2);
        assert_eq!(
            icons[0],
            UpstreamContentIcon {
                unicode: 63743,
                name: "spawn".to_string(),
                atlas_symbol: "block-spawn-ui".to_string(),
            }
        );
        assert_eq!(icons[0].emoji_string(), Some('\u{f8ff}'.to_string()));
        assert_eq!(icons[1].name, "ore-copper");
        assert_eq!(icons[1].atlas_symbol, "block-ore-copper-ui");
    }

    #[test]
    fn parse_upstream_icon_properties_reports_line_numbered_errors() {
        let missing_equals = parse_upstream_icon_properties("63743=spawn|block-spawn-ui\nbad");
        assert_eq!(
            missing_equals.unwrap_err(),
            IconPropertiesParseError::MissingEquals { line: 2 }
        );

        let missing_symbol = parse_upstream_icon_properties("63743=spawn|");
        assert_eq!(
            missing_symbol.unwrap_err(),
            IconPropertiesParseError::MissingAtlasSymbol { line: 1 }
        );

        let invalid_unicode = parse_upstream_icon_properties("x=spawn|block-spawn-ui");
        assert!(matches!(
            invalid_unicode.unwrap_err(),
            IconPropertiesParseError::InvalidUnicode { line: 1, .. }
        ));
    }
}
