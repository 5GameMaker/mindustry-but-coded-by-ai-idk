// Mirrors upstream core/src/mindustry/ui. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod bar;
pub mod dialogs;
pub mod displayable;
pub mod fonts;
pub mod mobile_button;
pub mod styles;
pub mod warning_bar;

pub const UPSTREAM_BUNDLE_PROPERTIES_SOURCE_PATH: &str = "core/assets/bundles/bundle.properties";

pub const UPSTREAM_MENU_BUNDLE_ENTRIES: &[(&str, &str)] = &[
    ("play", "Play"),
    ("campaign", "Campaign"),
    ("joingame", "Join Game"),
    ("customgame", "Custom Game"),
    ("loadgame", "Load Game"),
    ("database.button", "Database"),
    ("database", "Core Database"),
    ("schematics", "Schematics"),
    ("techtree", "Tech Tree"),
    ("about.button", "About"),
    ("editor", "Editor"),
    ("workshop", "Workshop"),
    ("mods", "Mods"),
    ("settings", "Settings"),
    ("quit", "Quit"),
];

pub const UPSTREAM_MENU_BUNDLE_ZH_CN_ENTRIES: &[(&str, &str)] = &[
    ("play", "开始游戏"),
    ("campaign", "战役模式"),
    ("joingame", "加入游戏"),
    ("customgame", "自定义游戏"),
    ("loadgame", "载入游戏"),
    ("database.button", "数据库"),
    ("database", "核心数据库"),
    ("schematics", "蓝图"),
    ("techtree", "科技树"),
    ("about.button", "关于"),
    ("editor", "地图编辑器"),
    ("workshop", "创意工坊"),
    ("mods", "模组"),
    ("settings", "设置"),
    ("quit", "退出"),
];

pub const UPSTREAM_MENU_BUNDLE_ZH_TW_ENTRIES: &[(&str, &str)] = &[
    ("play", "開始遊戲"),
    ("campaign", "戰役"),
    ("joingame", "多人連線"),
    ("customgame", "自訂遊戲"),
    ("loadgame", "載入遊戲"),
    ("database.button", "資料庫"),
    ("database", "核心資料庫"),
    ("schematics", "藍圖"),
    ("techtree", "科技樹"),
    ("about.button", "關於"),
    ("editor", "地圖編輯器"),
    ("workshop", "工作坊"),
    ("mods", "模組"),
    ("settings", "設定"),
    ("quit", "退出"),
];

pub fn upstream_bundle_en_value(key: &str) -> Option<&'static str> {
    UPSTREAM_MENU_BUNDLE_ENTRIES
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(*value))
}

fn upstream_bundle_value_from_entries(
    entries: &'static [(&'static str, &'static str)],
    key: &str,
) -> Option<&'static str> {
    entries
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(*value))
}

pub fn upstream_menu_bundle_entries_for_locale(
    locale: &str,
) -> &'static [(&'static str, &'static str)] {
    let locale = locale.trim().replace('-', "_");
    if locale.eq_ignore_ascii_case("zh_TW") || locale.eq_ignore_ascii_case("zh_HK") {
        UPSTREAM_MENU_BUNDLE_ZH_TW_ENTRIES
    } else if locale.eq_ignore_ascii_case("zh_CN")
        || locale.eq_ignore_ascii_case("zh")
        || locale.eq_ignore_ascii_case("zh_Hans")
    {
        UPSTREAM_MENU_BUNDLE_ZH_CN_ENTRIES
    } else {
        UPSTREAM_MENU_BUNDLE_ENTRIES
    }
}

pub fn upstream_menu_bundle_value_for_locale(locale: &str, key: &str) -> Option<&'static str> {
    upstream_bundle_value_from_entries(upstream_menu_bundle_entries_for_locale(locale), key)
        .or_else(|| upstream_bundle_en_value(key))
}

pub use bar::{
    Bar, BarBackgroundDraw, BarDrawCommand, BarDrawPlan, BarFillDraw, BarFrameState, BarLayout,
    BarOutlineDraw, BarTextDraw,
};
pub use dialogs::BaseDialog;
pub use displayable::{DisplayTable, Displayable};
pub use fonts::{
    parse_upstream_icon_properties, upstream_font_asset, upstream_font_asset_by_name,
    upstream_font_assets, upstream_font_source_paths, upstream_ui_icon_glyph,
    upstream_ui_icon_glyph_char, upstream_ui_icon_glyph_string, IconPropertiesParseError,
    UpstreamContentIcon, UpstreamFontAsset, UpstreamFontRole, UpstreamUiIconGlyph,
    UPSTREAM_FONT_ASSETS, UPSTREAM_ICONS_PROPERTIES_SOURCE_PATH, UPSTREAM_ICON_FONT_SOURCE_PATH,
    UPSTREAM_JAPANESE_FONT_SOURCE_PATH, UPSTREAM_LOGIC_FONT_CHARACTERS,
    UPSTREAM_LOGIC_FONT_SOURCE_PATH, UPSTREAM_MAIN_FONT_SOURCE_PATH,
    UPSTREAM_MONOSPACE_FONT_SOURCE_PATH, UPSTREAM_TECH_FONT_SOURCE_PATH,
    UPSTREAM_UI_ICON_FONTGEN_CONFIG_SOURCE_PATH, UPSTREAM_UI_ICON_GLYPHS,
};
pub use mobile_button::{MobileButton, MobileButtonLayout};
pub use styles::{
    upstream_check_box_style_skin, upstream_image_button_style_skin,
    upstream_scroll_pane_style_skin, upstream_slider_style_skin, upstream_text_button_style_skin,
    upstream_text_field_style_skin, upstream_ui_drawable_alias, upstream_ui_skin_sprite,
    upstream_ui_skin_sprite_source_paths, upstream_ui_skin_sprites, UiCheckBoxStyleSkin,
    UiDrawableAlias, UiDrawableTint, UiImageButtonStyleSkin, UiScrollPaneStyleSkin, UiSkinGroup,
    UiSkinSprite, UiSliderStyleSkin, UiTextButtonStyleSkin, UiTextFieldStyleSkin,
    UPSTREAM_CHECK_BOX_STYLE_SKINS, UPSTREAM_IMAGE_BUTTON_STYLE_SKINS,
    UPSTREAM_SCROLL_PANE_STYLE_SKINS, UPSTREAM_SLIDER_STYLE_SKINS,
    UPSTREAM_TEXT_BUTTON_STYLE_SKINS, UPSTREAM_TEXT_FIELD_STYLE_SKINS,
    UPSTREAM_UI_DRAWABLE_ALIASES, UPSTREAM_UI_SKIN_SPRITES,
};
pub use warning_bar::{
    LineSegment, Quad, WarningBar, WarningBarDrawCommand, WarningBarDrawPlan, WarningBarLayout,
    WarningBarLineDraw, WarningBarStripeDraw,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_menu_bundle_entries_cover_menu_fragment_buttons() {
        assert_eq!(
            UPSTREAM_BUNDLE_PROPERTIES_SOURCE_PATH,
            "core/assets/bundles/bundle.properties"
        );
        for (key, expected) in [
            ("play", "Play"),
            ("campaign", "Campaign"),
            ("joingame", "Join Game"),
            ("customgame", "Custom Game"),
            ("loadgame", "Load Game"),
            ("database.button", "Database"),
            ("database", "Core Database"),
            ("about.button", "About"),
            ("settings", "Settings"),
            ("quit", "Quit"),
        ] {
            assert_eq!(upstream_bundle_en_value(key), Some(expected));
        }
        assert_eq!(upstream_bundle_en_value("missing.menu.key"), None);
    }

    #[test]
    fn upstream_menu_bundle_locale_entries_cover_chinese_menu_fragment_buttons() {
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "play"),
            Some("开始游戏")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_CN", "database"),
            Some("核心数据库")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh-TW", "joingame"),
            Some("多人連線")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("zh_TW", "settings"),
            Some("設定")
        );
        assert_eq!(
            upstream_menu_bundle_value_for_locale("unknown", "database"),
            Some("Core Database")
        );
    }
}
