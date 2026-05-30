// Mirrors upstream core/src/mindustry/ui. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod bar;
pub mod dialogs;
pub mod displayable;
pub mod mobile_button;
pub mod styles;
pub mod warning_bar;

pub use bar::{
    Bar, BarBackgroundDraw, BarDrawCommand, BarDrawPlan, BarFillDraw, BarFrameState, BarLayout,
    BarOutlineDraw, BarTextDraw,
};
pub use dialogs::BaseDialog;
pub use displayable::{DisplayTable, Displayable};
pub use mobile_button::{MobileButton, MobileButtonLayout};
pub use styles::{
    upstream_text_button_style_skin, upstream_ui_drawable_alias, upstream_ui_skin_sprite,
    upstream_ui_skin_sprite_source_paths, upstream_ui_skin_sprites, UiDrawableAlias,
    UiDrawableTint, UiSkinGroup, UiSkinSprite, UiTextButtonStyleSkin,
    UPSTREAM_TEXT_BUTTON_STYLE_SKINS, UPSTREAM_UI_DRAWABLE_ALIASES, UPSTREAM_UI_SKIN_SPRITES,
};
pub use warning_bar::{
    LineSegment, Quad, WarningBar, WarningBarDrawCommand, WarningBarDrawPlan, WarningBarLayout,
    WarningBarLineDraw, WarningBarStripeDraw,
};
