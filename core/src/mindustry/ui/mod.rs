// Mirrors upstream core/src/mindustry/ui. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod bar;
pub mod dialogs;
pub mod displayable;
pub mod fonts;
pub mod mobile_button;
pub mod styles;
pub mod warning_bar;

pub use bar::{
    Bar, BarBackgroundDraw, BarDrawCommand, BarDrawPlan, BarFillDraw, BarFrameState, BarLayout,
    BarOutlineDraw, BarTextDraw,
};
pub use dialogs::BaseDialog;
pub use displayable::{DisplayTable, Displayable};
pub use fonts::{
    parse_upstream_icon_properties, upstream_font_asset, upstream_font_asset_by_name,
    upstream_font_assets, upstream_font_source_paths, IconPropertiesParseError,
    UpstreamContentIcon, UpstreamFontAsset, UpstreamFontRole, UPSTREAM_FONT_ASSETS,
    UPSTREAM_ICONS_PROPERTIES_SOURCE_PATH, UPSTREAM_ICON_FONT_SOURCE_PATH,
    UPSTREAM_JAPANESE_FONT_SOURCE_PATH, UPSTREAM_LOGIC_FONT_CHARACTERS,
    UPSTREAM_LOGIC_FONT_SOURCE_PATH, UPSTREAM_MAIN_FONT_SOURCE_PATH,
    UPSTREAM_MONOSPACE_FONT_SOURCE_PATH, UPSTREAM_TECH_FONT_SOURCE_PATH,
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
