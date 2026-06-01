/// Data-oriented migration of upstream `MenuRenderer`.
///
/// The Java renderer generates a temporary menu world, draws floor/overlay and
/// wall caches, then renders flyers and a dark fullscreen overlay.  This module
/// keeps generation and render planning in `mindustry-core`; concrete backends
/// are expected to translate `MenuRenderCommand` into GPU/cache calls.
use super::{
    RenderCamera, RenderCommand, RenderFontId, RenderPass, RenderPassKind, RenderPoint, RenderRect,
    RenderTextAlign, RenderTextStyle, RenderTextVerticalAlign, RenderViewport,
};
use crate::mindustry::ui::{
    upstream_bundle_en_value, upstream_image_button_style_skin, upstream_text_button_style_skin,
    upstream_ui_drawable_alias, upstream_ui_icon_glyph_string, UiDrawableAlias, UiDrawableTint,
};

pub const MENU_DARKNESS: f32 = 0.3;
pub const MENU_DARKNESS_LAYER: f32 = 90.0;
pub const MENU_SHADOW_TEXTURE_LAYER: f32 = 0.5;
pub const MENU_SHADOW_TEXTURE_ALPHA: f32 = 1.0;
pub const MENU_TILE_SIZE: f32 = 8.0;
pub const MENU_SUBMENU_FADE_IN_SECONDS: f32 = 0.15;
pub const MENU_SUBMENU_FADE_OUT_SECONDS: f32 = 0.2;
pub const MENU_DESKTOP_BUTTON_WIDTH: f32 = 230.0;
pub const MENU_DESKTOP_BUTTON_HEIGHT: f32 = 70.0;
pub const MENU_DESKTOP_BUTTON_MARGIN_LEFT: f32 = 11.0;
pub const MENU_DESKTOP_BUTTON_ICON_X: f32 = 30.0;
pub const MENU_DESKTOP_BUTTON_LABEL_GAP: f32 = 23.0;
pub const MENU_DESKTOP_BUTTON_ICON_TEXT_SIZE: f32 = 30.0;
pub const MENU_BUTTON_ICON_LAYER_OFFSET: f32 = 0.01;
pub const MENU_BUTTON_LABEL_LAYER_OFFSET: f32 = 0.02;
pub const MENU_DESKTOP_BACKGROUND_LAYER: f32 = 100.95;
pub const MENU_MAX_NATIVE_COLOR_SPAN_TILES: usize = 4;
pub const MENU_MOBILE_BUTTON_ICON_OFFSET_Y: f32 = 17.0;
pub const MENU_MOBILE_BUTTON_LABEL_OFFSET_Y: f32 = -25.0;
pub const MENU_MOBILE_BUTTON_ICON_TEXT_SIZE: f32 = 42.0;

/// Native-safe approximation of Java `Styles.flatToggleMenut`.
///
/// The state names stay explicit so the renderer can later swap the fallback
/// fills for actual drawable/texture-backed skins without changing the menu
/// layout or role selection logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuFlatToggleMenuStyle {
    pub down_fill: [f32; 4],
    pub up_fill: [f32; 4],
    pub checked_fill: [f32; 4],
    pub over_fill: [f32; 4],
    pub disabled_fill: [f32; 4],
    pub down_drawable: &'static str,
    pub up_drawable: &'static str,
    pub checked_drawable: &'static str,
    pub over_drawable: &'static str,
    pub disabled_drawable: &'static str,
    pub text_color: [f32; 4],
    pub disabled_text_color: [f32; 4],
    pub text_style: RenderTextStyle,
    pub fill_layer: f32,
    pub drawable_layer: f32,
    pub text_layer: f32,
    pub desktop_text_size: f32,
    pub mobile_text_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuFlatToggleMenuState {
    Down,
    Up,
    Checked,
    Over,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuImageButtonState {
    Down,
    Up,
    Over,
    Disabled,
}

pub const MENU_FLAT_TOGGLE_MENU_STYLE: MenuFlatToggleMenuStyle = MenuFlatToggleMenuStyle {
    down_fill: [0.19, 0.38, 0.50, 0.92],
    up_fill: [0.0, 0.0, 0.0, 0.0],
    checked_fill: [0.19, 0.38, 0.50, 0.92],
    over_fill: [0.270_588_25, 0.270_588_25, 0.270_588_25, 1.0],
    disabled_fill: [0.0, 0.0, 0.0, 1.0],
    down_drawable: "flat-down-base.9",
    up_drawable: "",
    checked_drawable: "flat-down-base.9",
    over_drawable: "",
    disabled_drawable: "",
    text_color: [1.0, 1.0, 1.0, 1.0],
    disabled_text_color: [0.5, 0.5, 0.5, 1.0],
    text_style: RenderTextStyle::new(RenderTextAlign::Center)
        .with_vertical_align(RenderTextVerticalAlign::Center)
        .with_markup(true)
        .with_integer_position(true),
    fill_layer: 101.0,
    drawable_layer: 101.05,
    text_layer: 101.1,
    desktop_text_size: 18.0,
    mobile_text_size: 16.0,
};

impl MenuFlatToggleMenuStyle {
    pub const fn fill_for(self, state: MenuFlatToggleMenuState) -> [f32; 4] {
        match state {
            MenuFlatToggleMenuState::Down => self.down_fill,
            MenuFlatToggleMenuState::Up => self.up_fill,
            MenuFlatToggleMenuState::Checked => self.checked_fill,
            MenuFlatToggleMenuState::Over => self.over_fill,
            MenuFlatToggleMenuState::Disabled => self.disabled_fill,
        }
    }

    pub const fn drawable_for(self, state: MenuFlatToggleMenuState) -> &'static str {
        match state {
            MenuFlatToggleMenuState::Down => self.down_drawable,
            MenuFlatToggleMenuState::Up => self.up_drawable,
            MenuFlatToggleMenuState::Checked => self.checked_drawable,
            MenuFlatToggleMenuState::Over => self.over_drawable,
            MenuFlatToggleMenuState::Disabled => self.disabled_drawable,
        }
    }

    pub const fn text_size(self, mobile: bool) -> f32 {
        if mobile {
            self.mobile_text_size
        } else {
            self.desktop_text_size
        }
    }
}

fn menu_flat_toggle_menu_java_drawable_for_state(
    state: MenuFlatToggleMenuState,
) -> Option<&'static str> {
    let style = upstream_text_button_style_skin("flatToggleMenut")?;
    match state {
        MenuFlatToggleMenuState::Down => style.down,
        MenuFlatToggleMenuState::Up => style.up,
        MenuFlatToggleMenuState::Checked => style.checked.or(style.down),
        MenuFlatToggleMenuState::Over => style.over,
        MenuFlatToggleMenuState::Disabled => style.disabled,
    }
}

fn menu_flat_toggle_menu_drawable_for_state(
    state: MenuFlatToggleMenuState,
) -> Option<&'static UiDrawableAlias> {
    menu_flat_toggle_menu_java_drawable_for_state(state).and_then(upstream_ui_drawable_alias)
}

fn menu_drawable_alias_is_visually_clear(drawable: &UiDrawableAlias) -> bool {
    drawable.atlas_symbol == "clear"
        || drawable.tint == UiDrawableTint::Transparent
        || drawable.tint.rgba()[3] <= 0.0
}

fn menu_push_flat_toggle_menu_state_background(
    commands: &mut Vec<RenderCommand>,
    rect: RenderRect,
    state: MenuFlatToggleMenuState,
    style: MenuFlatToggleMenuStyle,
    alpha_scale: f32,
) {
    if let Some(drawable) = menu_flat_toggle_menu_drawable_for_state(state) {
        if menu_drawable_alias_is_visually_clear(drawable) {
            return;
        }
        let tint = menu_color_with_alpha(drawable.tint.rgba(), alpha_scale);
        if drawable.tint != UiDrawableTint::Transparent && tint[3] > 0.0 {
            commands.push(RenderCommand::draw_sprite(
                drawable.atlas_symbol,
                rect,
                tint,
                0.0,
                style.drawable_layer,
            ));
        }
        return;
    }

    let fill = menu_color_with_alpha(style.fill_for(state), alpha_scale);
    if fill[3] > 0.0 {
        commands.push(RenderCommand::fill_rect(rect, fill, style.fill_layer));
    }
    let drawable = style.drawable_for(state);
    if !drawable.is_empty() {
        commands.push(RenderCommand::draw_sprite(
            drawable,
            rect,
            [1.0, 1.0, 1.0, alpha_scale],
            0.0,
            style.drawable_layer,
        ));
    }
}

fn menu_image_button_java_drawable_for_state(state: MenuImageButtonState) -> Option<&'static str> {
    let style = upstream_image_button_style_skin("defaulti")?;
    match state {
        MenuImageButtonState::Down => style.down,
        MenuImageButtonState::Up => style.up,
        MenuImageButtonState::Over => style.over,
        MenuImageButtonState::Disabled => style.disabled,
    }
}

fn menu_image_button_drawable_for_state(
    state: MenuImageButtonState,
) -> Option<&'static UiDrawableAlias> {
    menu_image_button_java_drawable_for_state(state).and_then(upstream_ui_drawable_alias)
}

fn menu_push_mobile_image_button_background(
    commands: &mut Vec<RenderCommand>,
    rect: RenderRect,
    state: MenuImageButtonState,
    alpha_scale: f32,
) {
    if let Some(drawable) = menu_image_button_drawable_for_state(state) {
        let tint = menu_color_with_alpha(drawable.tint.rgba(), alpha_scale);
        if drawable.tint != UiDrawableTint::Transparent && tint[3] > 0.0 {
            commands.push(RenderCommand::draw_sprite(
                drawable.atlas_symbol,
                rect,
                tint,
                0.0,
                MENU_FLAT_TOGGLE_MENU_STYLE.drawable_layer,
            ));
        }
        return;
    }

    let color = match state {
        MenuImageButtonState::Down => [0.18, 0.28, 0.36, 0.92],
        MenuImageButtonState::Up => [0.055, 0.075, 0.095, 0.88],
        MenuImageButtonState::Over => [0.10, 0.16, 0.22, 0.90],
        MenuImageButtonState::Disabled => [0.0, 0.0, 0.0, 0.88],
    };
    commands.push(RenderCommand::fill_rect(
        rect,
        menu_color_with_alpha(color, alpha_scale),
        MENU_FLAT_TOGGLE_MENU_STYLE.fill_layer,
    ));
}

fn menu_push_black6_panel(commands: &mut Vec<RenderCommand>, rect: RenderRect, alpha_scale: f32) {
    if let Some(drawable) = upstream_ui_drawable_alias("black6") {
        let tint = menu_color_with_alpha(drawable.tint.rgba(), alpha_scale);
        if drawable.tint != UiDrawableTint::Transparent && tint[3] > 0.0 {
            commands.push(RenderCommand::draw_sprite(
                drawable.atlas_symbol,
                rect,
                tint,
                0.0,
                MENU_DESKTOP_BACKGROUND_LAYER,
            ));
            return;
        }
    }

    commands.push(RenderCommand::fill_rect(
        rect,
        menu_color_with_alpha([0.0, 0.0, 0.0, 0.6], alpha_scale),
        MENU_DESKTOP_BACKGROUND_LAYER,
    ));
    commands.push(RenderCommand::stroke_rect(
        rect,
        menu_color_with_alpha([0.0, 0.0, 0.0, 0.42], alpha_scale),
        1.0,
        MENU_DESKTOP_BACKGROUND_LAYER + 0.01,
    ));
}

fn menu_union_rect(a: RenderRect, b: RenderRect) -> RenderRect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let right = a.right().max(b.right());
    let top = a.bottom().max(b.bottom());
    RenderRect::new(x, y, right - x, top - y)
}

fn menu_push_desktop_panel_backgrounds(
    commands: &mut Vec<RenderCommand>,
    buttons: &[MenuButtonPlan],
    submenu_alpha: f32,
) {
    let mut main_bounds: Option<RenderRect> = None;
    let mut submenu_bounds: Option<RenderRect> = None;

    for button in buttons {
        let bounds = if button.submenu {
            &mut submenu_bounds
        } else {
            &mut main_bounds
        };
        *bounds = Some(bounds.map_or(button.rect, |rect| menu_union_rect(rect, button.rect)));
    }

    let Some(main_bounds) = main_bounds else {
        return;
    };
    let inferred_stage_height =
        (main_bounds.y * 2.0 + main_bounds.height).max(main_bounds.bottom());
    let main_panel = RenderRect::new(
        main_bounds.x,
        0.0,
        main_bounds.right() - main_bounds.x,
        inferred_stage_height,
    );
    menu_push_black6_panel(commands, main_panel, 1.0);

    if let Some(submenu_bounds) = submenu_bounds {
        if submenu_alpha > f32::EPSILON {
            let submenu_panel = RenderRect::new(
                submenu_bounds.x,
                0.0,
                submenu_bounds.right() - submenu_bounds.x,
                inferred_stage_height,
            );
            menu_push_black6_panel(commands, submenu_panel, submenu_alpha);
        }
    }
}

fn menu_color_with_alpha(mut color: [f32; 4], alpha_scale: f32) -> [f32; 4] {
    color[3] = (color[3] * alpha_scale.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    color
}

fn menu_shadow_texture_rect(x: f32, y: f32, width: f32, height: f32) -> RenderRect {
    let draw_width = width.abs();
    let draw_height = height.abs();
    RenderRect::new(
        x - draw_width * 0.5,
        y - draw_height * 0.5,
        draw_width,
        draw_height,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBlockKind {
    Air,
    Sand,
    SandWall,
    Shale,
    ShaleWall,
    Ice,
    IceWall,
    Moss,
    SporePine,
    Dirt,
    DirtWall,
    Dacite,
    DaciteWall,
    Basalt,
    DuneWall,
    Stone,
    StoneWall,
    SporeWall,
    Salt,
    CopperOre,
    LeadOre,
    ScrapOre,
    CoalOre,
    TitaniumOre,
    ThoriumOre,
    Hotrock,
    Magmarock,
    DarkPanel3,
    DarkPanel4,
    DarkMetal,
    SporeMoss,
}

impl MenuBlockKind {
    pub const fn sprite_name(self) -> Option<&'static str> {
        match self {
            Self::Air => None,
            Self::Sand => Some("sand"),
            Self::SandWall => Some("sand-wall"),
            Self::Shale => Some("shale"),
            Self::ShaleWall => Some("shale-wall"),
            Self::Ice => Some("ice-snow"),
            Self::IceWall => Some("ice-wall"),
            Self::Moss => Some("moss"),
            Self::SporePine => Some("spore-pine"),
            Self::Dirt => Some("dirt"),
            Self::DirtWall => Some("dirt-wall"),
            Self::Dacite => Some("dacite"),
            Self::DaciteWall => Some("dacite-wall"),
            Self::Basalt => Some("basalt"),
            Self::DuneWall => Some("dune-wall"),
            Self::Stone => Some("stone"),
            Self::StoneWall => Some("stone-wall"),
            Self::SporeWall => Some("spore-wall"),
            Self::Salt => Some("salt"),
            Self::CopperOre => Some("ore-copper"),
            Self::LeadOre => Some("ore-lead"),
            Self::ScrapOre => Some("ore-scrap"),
            Self::CoalOre => Some("ore-coal"),
            Self::TitaniumOre => Some("ore-titanium"),
            Self::ThoriumOre => Some("ore-thorium"),
            Self::Hotrock => Some("hotrock"),
            Self::Magmarock => Some("magmarock"),
            Self::DarkPanel3 => Some("dark-panel-3"),
            Self::DarkPanel4 => Some("dark-panel-4"),
            Self::DarkMetal => Some("dark-metal"),
            Self::SporeMoss => Some("spore-moss"),
        }
    }

    pub const fn menu_color(self) -> Option<[f32; 4]> {
        match self {
            Self::Air => None,
            Self::Sand => Some([0.66, 0.58, 0.38, 1.0]),
            Self::SandWall => Some([0.42, 0.34, 0.21, 1.0]),
            Self::Shale => Some([0.40, 0.40, 0.45, 1.0]),
            Self::ShaleWall => Some([0.27, 0.27, 0.33, 1.0]),
            Self::Ice => Some([0.68, 0.82, 0.88, 1.0]),
            Self::IceWall => Some([0.38, 0.53, 0.64, 1.0]),
            Self::Moss => Some([0.25, 0.39, 0.24, 1.0]),
            Self::SporePine => Some([0.30, 0.22, 0.36, 1.0]),
            Self::Dirt => Some([0.43, 0.30, 0.20, 1.0]),
            Self::DirtWall => Some([0.28, 0.20, 0.15, 1.0]),
            Self::Dacite => Some([0.45, 0.45, 0.43, 1.0]),
            Self::DaciteWall => Some([0.28, 0.28, 0.27, 1.0]),
            Self::Basalt => Some([0.22, 0.23, 0.25, 1.0]),
            Self::DuneWall => Some([0.48, 0.37, 0.22, 1.0]),
            Self::Stone => Some([0.47, 0.47, 0.45, 1.0]),
            Self::StoneWall => Some([0.31, 0.31, 0.30, 1.0]),
            Self::SporeWall => Some([0.34, 0.22, 0.38, 1.0]),
            Self::Salt => Some([0.68, 0.66, 0.58, 1.0]),
            Self::CopperOre => Some([0.90, 0.55, 0.30, 1.0]),
            Self::LeadOre => Some([0.55, 0.60, 0.76, 1.0]),
            Self::ScrapOre => Some([0.64, 0.50, 0.40, 1.0]),
            Self::CoalOre => Some([0.18, 0.18, 0.19, 1.0]),
            Self::TitaniumOre => Some([0.65, 0.68, 0.82, 1.0]),
            Self::ThoriumOre => Some([0.82, 0.55, 0.86, 1.0]),
            Self::Hotrock => Some([0.56, 0.21, 0.11, 1.0]),
            Self::Magmarock => Some([0.78, 0.28, 0.10, 1.0]),
            Self::DarkPanel3 => Some([0.12, 0.16, 0.20, 1.0]),
            Self::DarkPanel4 => Some([0.09, 0.12, 0.16, 1.0]),
            Self::DarkMetal => Some([0.08, 0.10, 0.13, 1.0]),
            Self::SporeMoss => Some([0.30, 0.36, 0.24, 1.0]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuTile {
    pub x: u16,
    pub y: u16,
    pub floor: MenuBlockKind,
    pub wall: MenuBlockKind,
    pub ore: MenuBlockKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuWorldPlan {
    pub width: usize,
    pub height: usize,
    pub seed: u64,
    pub tiles: Vec<MenuTile>,
    pub cache_floor_id: i32,
    pub cache_wall_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuRendererConfig {
    pub mobile: bool,
    pub seed: u64,
    pub tile_size: f32,
    pub desktop_workshop_enabled: bool,
    pub mobile_ios: bool,
}

impl MenuRendererConfig {
    pub const fn new(mobile: bool, seed: u64) -> Self {
        Self {
            mobile,
            seed,
            tile_size: MENU_TILE_SIZE,
            desktop_workshop_enabled: false,
            mobile_ios: false,
        }
    }

    pub const fn with_desktop_workshop_enabled(mut self, enabled: bool) -> Self {
        self.desktop_workshop_enabled = enabled;
        self
    }

    pub const fn with_mobile_ios(mut self, enabled: bool) -> Self {
        self.mobile_ios = enabled;
        self
    }

    pub const fn width(&self) -> usize {
        if self.mobile {
            60
        } else {
            100
        }
    }

    pub const fn height(&self) -> usize {
        if self.mobile {
            40
        } else {
            50
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuFrameInput {
    pub graphics_width: f32,
    pub graphics_height: f32,
    pub scene_margin_top: f32,
    pub scene_margin_bottom: f32,
    pub scl4: f32,
    pub delta: f32,
}

impl MenuFrameInput {
    pub const fn new(graphics_width: f32, graphics_height: f32, scl4: f32, delta: f32) -> Self {
        Self {
            graphics_width,
            graphics_height,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4,
            delta,
        }
    }

    pub const fn with_scene_margins(mut self, top: f32, bottom: f32) -> Self {
        self.scene_margin_top = top;
        self.scene_margin_bottom = bottom;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlyerPlan {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub unit_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuButtonRole {
    Play,
    Campaign,
    Join,
    CustomGame,
    LoadGame,
    Database,
    Schematics,
    ContentDatabase,
    TechTree,
    About,
    Editor,
    Workshop,
    Mods,
    Settings,
    Custom(u16),
    CustomSubmenu { root: u16, item: u16 },
    Quit,
}

impl MenuButtonRole {
    pub const fn bundle_key(self) -> Option<&'static str> {
        match self {
            Self::Play => Some("play"),
            Self::Campaign => Some("campaign"),
            Self::Join => Some("joingame"),
            Self::CustomGame => Some("customgame"),
            Self::LoadGame => Some("loadgame"),
            Self::Database => Some("database.button"),
            Self::Schematics => Some("schematics"),
            Self::ContentDatabase => Some("database"),
            Self::TechTree => Some("techtree"),
            Self::About => Some("about.button"),
            Self::Editor => Some("editor"),
            Self::Workshop => Some("workshop"),
            Self::Mods => Some("mods"),
            Self::Settings => Some("settings"),
            Self::Custom(_) | Self::CustomSubmenu { .. } => None,
            Self::Quit => Some("quit"),
        }
    }

    pub fn label(self) -> &'static str {
        if let Some(label) = self.bundle_key().and_then(upstream_bundle_en_value) {
            return label;
        }
        match self {
            Self::Play => "Play",
            Self::Campaign => "Campaign",
            Self::Join => "Join Game",
            Self::CustomGame => "Custom Game",
            Self::LoadGame => "Load Game",
            Self::Database => "Database",
            Self::ContentDatabase => "Core Database",
            Self::Schematics => "Schematics",
            Self::TechTree => "Tech Tree",
            Self::About => "About",
            Self::Editor => "Editor",
            Self::Workshop => "Workshop",
            Self::Mods => "Mods",
            Self::Settings => "Settings",
            Self::Custom(_) | Self::CustomSubmenu { .. } => "Custom",
            Self::Quit => "Quit",
        }
    }

    pub const fn is_submenu(self) -> bool {
        matches!(
            self,
            Self::Campaign
                | Self::Join
                | Self::CustomGame
                | Self::LoadGame
                | Self::Schematics
                | Self::ContentDatabase
                | Self::TechTree
                | Self::About
                | Self::CustomSubmenu { .. }
        )
    }

    pub const fn is_desktop_root(self) -> bool {
        matches!(
            self,
            Self::Play
                | Self::Database
                | Self::Editor
                | Self::Workshop
                | Self::Mods
                | Self::Settings
                | Self::Quit
        )
    }

    pub const fn has_desktop_submenu(self) -> bool {
        matches!(self, Self::Play | Self::Database)
    }

    pub const fn icon_name(self, mobile: bool) -> Option<&'static str> {
        match self {
            Self::Play | Self::Campaign => Some("play"),
            Self::Join => Some("add"),
            Self::CustomGame if mobile => Some("rightOpenOut"),
            Self::CustomGame => Some("terrain"),
            Self::LoadGame => Some("download"),
            Self::Database => Some("menu"),
            Self::Schematics => Some("paste"),
            Self::ContentDatabase | Self::Mods => Some("book"),
            Self::TechTree => Some("tree"),
            Self::About => Some("info"),
            Self::Editor => Some("terrain"),
            Self::Workshop => Some("steam"),
            Self::Settings => Some("settings"),
            Self::Quit => Some("exit"),
            Self::Custom(_) | Self::CustomSubmenu { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuCustomButton {
    pub label: String,
    pub icon_name: Option<String>,
    pub action_id: Option<String>,
    pub submenu_buttons: Vec<MenuCustomButton>,
}

impl MenuCustomButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon_name: None,
            action_id: None,
            submenu_buttons: Vec::new(),
        }
    }

    pub fn with_icon_name(mut self, icon_name: impl Into<String>) -> Self {
        self.icon_name = Some(icon_name.into());
        self
    }

    pub fn with_action_id(mut self, action_id: impl Into<String>) -> Self {
        self.action_id = Some(action_id.into());
        self
    }

    pub fn with_submenu_buttons(mut self, submenu_buttons: Vec<MenuCustomButton>) -> Self {
        self.submenu_buttons = submenu_buttons;
        self
    }

    pub fn has_submenu(&self) -> bool {
        !self.submenu_buttons.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuButtonPlan {
    pub role: MenuButtonRole,
    pub label: String,
    pub icon_name: Option<String>,
    pub rect: RenderRect,
    pub selected: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub submenu: bool,
}

impl MenuButtonPlan {
    pub const fn flat_toggle_menu_state(&self) -> MenuFlatToggleMenuState {
        if self.pressed {
            MenuFlatToggleMenuState::Down
        } else if self.selected {
            MenuFlatToggleMenuState::Checked
        } else if self.hovered {
            MenuFlatToggleMenuState::Over
        } else {
            MenuFlatToggleMenuState::Up
        }
    }

    pub const fn image_button_state(&self) -> MenuImageButtonState {
        if self.pressed {
            MenuImageButtonState::Down
        } else if self.hovered {
            MenuImageButtonState::Over
        } else {
            MenuImageButtonState::Up
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuUiPlan {
    pub mobile: bool,
    pub submenu_alpha: f32,
    pub buttons: Vec<MenuButtonPlan>,
}

impl MenuUiPlan {
    pub fn with_hovered_role(mut self, hovered_role: Option<MenuButtonRole>) -> Self {
        for button in &mut self.buttons {
            button.hovered = hovered_role == Some(button.role);
        }
        self
    }

    pub fn with_pressed_role(mut self, pressed_role: Option<MenuButtonRole>) -> Self {
        for button in &mut self.buttons {
            button.pressed = pressed_role == Some(button.role);
        }
        self
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<MenuButtonRole> {
        self.buttons
            .iter()
            .rev()
            .find(|button| {
                x >= button.rect.x
                    && x <= button.rect.x + button.rect.width
                    && y >= button.rect.y
                    && y <= button.rect.y + button.rect.height
            })
            .map(|button| button.role)
    }

    pub fn to_render_commands(&self) -> Vec<RenderCommand> {
        let style = MENU_FLAT_TOGGLE_MENU_STYLE;
        let mut commands = Vec::with_capacity(self.buttons.len() * 6 + 4);
        if !self.mobile {
            menu_push_desktop_panel_backgrounds(&mut commands, &self.buttons, self.submenu_alpha);
        }
        for button in &self.buttons {
            let alpha = if button.submenu {
                self.submenu_alpha
            } else {
                1.0
            };
            if alpha <= f32::EPSILON {
                continue;
            }
            if self.mobile {
                menu_push_mobile_image_button_background(
                    &mut commands,
                    button.rect,
                    button.image_button_state(),
                    alpha,
                );
            } else {
                let state = button.flat_toggle_menu_state();
                menu_push_flat_toggle_menu_state_background(
                    &mut commands,
                    button.rect,
                    state,
                    style,
                    alpha,
                );
            }
            let icon_name = button
                .icon_name
                .as_deref()
                .or_else(|| button.role.icon_name(self.mobile));
            let icon_layer = style.text_layer + MENU_BUTTON_ICON_LAYER_OFFSET;
            let label_layer = style.text_layer + MENU_BUTTON_LABEL_LAYER_OFFSET;
            if let Some(icon_name) = icon_name {
                let icon_point = if self.mobile {
                    RenderPoint::new(
                        button.rect.center().x,
                        button.rect.center().y + MENU_MOBILE_BUTTON_ICON_OFFSET_Y,
                    )
                } else {
                    RenderPoint::new(
                        button.rect.x
                            + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                            + MENU_DESKTOP_BUTTON_ICON_X,
                        button.rect.center().y,
                    )
                };
                menu_push_icon_render_commands(
                    &mut commands,
                    icon_name,
                    icon_point,
                    if self.mobile {
                        MENU_MOBILE_BUTTON_ICON_TEXT_SIZE
                    } else {
                        MENU_DESKTOP_BUTTON_ICON_TEXT_SIZE
                    },
                    menu_color_with_alpha(style.text_color, alpha),
                    icon_layer,
                );
            }
            let (label_point, label_style) = if self.mobile {
                (
                    RenderPoint::new(
                        button.rect.center().x,
                        button.rect.center().y + MENU_MOBILE_BUTTON_LABEL_OFFSET_Y,
                    ),
                    style.text_style,
                )
            } else if icon_name.is_some() {
                (
                    RenderPoint::new(
                        button.rect.x
                            + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                            + MENU_DESKTOP_BUTTON_ICON_X
                            + MENU_DESKTOP_BUTTON_LABEL_GAP,
                        button.rect.center().y,
                    ),
                    RenderTextStyle::new(RenderTextAlign::Start)
                        .with_vertical_align(RenderTextVerticalAlign::Center)
                        .with_markup(true)
                        .with_integer_position(true),
                )
            } else {
                (button.rect.center(), style.text_style)
            };
            commands.push(RenderCommand::draw_text_styled(
                button.label.as_str(),
                label_point,
                menu_color_with_alpha(style.text_color, alpha),
                style.text_size(self.mobile),
                0.0,
                label_style,
                label_layer,
            ));
        }
        commands
    }
}

fn menu_push_icon_render_commands(
    commands: &mut Vec<RenderCommand>,
    icon_name: &str,
    center: RenderPoint,
    size: f32,
    color: [f32; 4],
    layer: f32,
) {
    let size = size.max(1.0);
    if let Some(glyph) = upstream_ui_icon_glyph_string(icon_name) {
        commands.push(RenderCommand::draw_text_styled(
            glyph,
            center,
            color,
            size,
            0.0,
            RenderTextStyle::new(RenderTextAlign::Center)
                .with_font(RenderFontId::Icon)
                .with_vertical_align(RenderTextVerticalAlign::Center)
                .with_integer_position(true),
            layer,
        ));
        return;
    }

    let stroke = (size / 9.0).max(1.0);
    match icon_name {
        "play" | "rightOpenOut" => {
            commands.push(RenderCommand::draw_triangle(
                center,
                size * 0.70,
                size * 0.92,
                0.0,
                color,
                layer,
            ));
        }
        "add" | "host" => {
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x - size * 0.38, center.y),
                RenderPoint::new(center.x + size * 0.38, center.y),
                stroke,
                color,
                layer,
            ));
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x, center.y - size * 0.38),
                RenderPoint::new(center.x, center.y + size * 0.38),
                stroke,
                color,
                layer,
            ));
        }
        "settings" => {
            commands.push(RenderCommand::draw_polygon(
                center,
                size * 0.46,
                8,
                22.5,
                color,
                false,
                layer,
            ));
            commands.push(RenderCommand::draw_circle(
                center,
                size * 0.18,
                color,
                false,
                layer,
            ));
        }
        "info" => {
            commands.push(RenderCommand::draw_circle(
                center,
                size * 0.44,
                color,
                false,
                layer,
            ));
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x, center.y - size * 0.10),
                RenderPoint::new(center.x, center.y + size * 0.28),
                stroke,
                color,
                layer,
            ));
            commands.push(RenderCommand::draw_circle(
                RenderPoint::new(center.x, center.y - size * 0.28),
                stroke,
                color,
                true,
                layer,
            ));
        }
        "exit" => {
            commands.push(RenderCommand::stroke_rect(
                RenderRect::new(
                    center.x - size * 0.42,
                    center.y - size * 0.34,
                    size * 0.46,
                    size * 0.68,
                ),
                color,
                stroke,
                layer,
            ));
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x - size * 0.02, center.y),
                RenderPoint::new(center.x + size * 0.42, center.y),
                stroke,
                color,
                layer,
            ));
            commands.push(RenderCommand::draw_triangle(
                RenderPoint::new(center.x + size * 0.42, center.y),
                size * 0.35,
                size * 0.30,
                0.0,
                color,
                layer,
            ));
        }
        "download" => {
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x, center.y + size * 0.36),
                RenderPoint::new(center.x, center.y - size * 0.10),
                stroke,
                color,
                layer,
            ));
            commands.push(RenderCommand::draw_triangle(
                RenderPoint::new(center.x, center.y - size * 0.22),
                size * 0.42,
                size * 0.34,
                180.0,
                color,
                layer,
            ));
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x - size * 0.38, center.y - size * 0.38),
                RenderPoint::new(center.x + size * 0.38, center.y - size * 0.38),
                stroke,
                color,
                layer,
            ));
        }
        "terrain" => {
            commands.push(RenderCommand::draw_triangle(
                RenderPoint::new(center.x - size * 0.18, center.y - size * 0.05),
                size * 0.72,
                size * 0.64,
                0.0,
                color,
                layer,
            ));
            commands.push(RenderCommand::draw_triangle(
                RenderPoint::new(center.x + size * 0.22, center.y - size * 0.12),
                size * 0.56,
                size * 0.50,
                0.0,
                color,
                layer,
            ));
        }
        "menu" => {
            for offset in [-0.26_f32, 0.0, 0.26] {
                commands.push(RenderCommand::draw_line(
                    RenderPoint::new(center.x - size * 0.38, center.y + size * offset),
                    RenderPoint::new(center.x + size * 0.38, center.y + size * offset),
                    stroke,
                    color,
                    layer,
                ));
            }
        }
        "book" => {
            commands.push(RenderCommand::stroke_rect(
                RenderRect::new(
                    center.x - size * 0.42,
                    center.y - size * 0.36,
                    size * 0.84,
                    size * 0.72,
                ),
                color,
                stroke,
                layer,
            ));
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x, center.y - size * 0.34),
                RenderPoint::new(center.x, center.y + size * 0.34),
                stroke,
                color,
                layer,
            ));
        }
        "paste" => {
            commands.push(RenderCommand::stroke_rect(
                RenderRect::new(
                    center.x - size * 0.34,
                    center.y - size * 0.40,
                    size * 0.68,
                    size * 0.76,
                ),
                color,
                stroke,
                layer,
            ));
            commands.push(RenderCommand::stroke_rect(
                RenderRect::new(
                    center.x - size * 0.18,
                    center.y + size * 0.18,
                    size * 0.36,
                    size * 0.20,
                ),
                color,
                stroke,
                layer,
            ));
        }
        "tree" => {
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x, center.y - size * 0.36),
                RenderPoint::new(center.x, center.y + size * 0.24),
                stroke,
                color,
                layer,
            ));
            for (dx, dy) in [(-0.28_f32, 0.28_f32), (0.28, 0.28), (0.0, -0.30)] {
                let node = RenderPoint::new(center.x + size * dx, center.y + size * dy);
                commands.push(RenderCommand::draw_line(center, node, stroke, color, layer));
                commands.push(RenderCommand::draw_circle(
                    node,
                    size * 0.13,
                    color,
                    false,
                    layer,
                ));
            }
        }
        "steam" => {
            commands.push(RenderCommand::draw_circle(
                center,
                size * 0.42,
                color,
                false,
                layer,
            ));
            let small = RenderPoint::new(center.x + size * 0.22, center.y + size * 0.15);
            commands.push(RenderCommand::draw_circle(
                small,
                size * 0.15,
                color,
                false,
                layer,
            ));
            commands.push(RenderCommand::draw_line(
                RenderPoint::new(center.x - size * 0.22, center.y - size * 0.18),
                small,
                stroke,
                color,
                layer,
            ));
        }
        _ => {
            commands.push(RenderCommand::draw_text_styled(
                menu_icon_text(icon_name),
                center,
                color,
                size,
                0.0,
                RenderTextStyle::new(RenderTextAlign::Center)
                    .with_vertical_align(RenderTextVerticalAlign::Center)
                    .with_integer_position(true),
                layer,
            ));
        }
    }
}

fn menu_icon_text(icon_name: &str) -> String {
    icon_name
        .chars()
        .find(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "!".to_string())
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuRenderCommand {
    DrawCache {
        cache_id: i32,
        label: &'static str,
    },
    DrawShadowTexture {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    DrawFlyer(FlyerPlan),
    DrawDarkness {
        alpha: f32,
        width: f32,
        height: f32,
    },
}

impl MenuRenderCommand {
    pub fn to_render_commands(&self, world: &MenuWorldPlan, tile_size: f32) -> Vec<RenderCommand> {
        self.clone().into_render_commands(world, tile_size)
    }

    pub fn into_render_commands(self, world: &MenuWorldPlan, tile_size: f32) -> Vec<RenderCommand> {
        self.into_render_commands_with_transform(world, tile_size, None)
    }

    fn to_render_commands_with_transform(
        &self,
        world: &MenuWorldPlan,
        tile_size: f32,
        transform: MenuScreenTransform,
    ) -> Vec<RenderCommand> {
        self.clone()
            .into_render_commands_with_transform(world, tile_size, Some(transform))
    }

    fn into_render_commands_with_transform(
        self,
        world: &MenuWorldPlan,
        tile_size: f32,
        transform: Option<MenuScreenTransform>,
    ) -> Vec<RenderCommand> {
        match self {
            Self::DrawCache { label, .. } => {
                let include_tile_sprites = transform.is_none();
                let mut commands = Vec::with_capacity(world.width * world.height / 2 + 64);
                match label {
                    "floor+overlay" => {
                        menu_push_horizontal_tile_color_spans(
                            &mut commands,
                            world,
                            tile_size,
                            transform,
                            -0.2,
                            |tile| tile.floor.menu_color(),
                        );
                        for tile in &world.tiles {
                            let rect =
                                menu_transform_rect(menu_tile_rect(tile, tile_size), transform);
                            if include_tile_sprites {
                                if let Some(sprite) =
                                    menu_block_variant_sprite_name(tile.floor, tile)
                                {
                                    commands.push(RenderCommand::draw_sprite(
                                        sprite,
                                        rect,
                                        [1.0, 1.0, 1.0, 1.0],
                                        0.0,
                                        -0.1,
                                    ));
                                }
                            }
                            if let Some(color) = tile.ore.menu_color() {
                                commands.push(RenderCommand::fill_rect(
                                    menu_transform_rect(
                                        menu_tile_inset_rect(tile, tile_size, 0.24),
                                        transform,
                                    ),
                                    color,
                                    0.1,
                                ));
                            }
                            if include_tile_sprites {
                                if let Some(sprite) = menu_block_variant_sprite_name(tile.ore, tile)
                                {
                                    commands.push(RenderCommand::draw_sprite(
                                        sprite,
                                        rect,
                                        [1.0, 1.0, 1.0, 1.0],
                                        0.0,
                                        0.15,
                                    ));
                                }
                            }
                        }
                    }
                    "wall" => {
                        menu_push_horizontal_tile_color_spans(
                            &mut commands,
                            world,
                            tile_size,
                            transform,
                            1.0,
                            |tile| tile.wall.menu_color(),
                        );
                        if include_tile_sprites {
                            for tile in &world.tiles {
                                let rect =
                                    menu_transform_rect(menu_tile_rect(tile, tile_size), transform);
                                if let Some(sprite) =
                                    menu_block_variant_sprite_name(tile.wall, tile)
                                {
                                    commands.push(RenderCommand::draw_sprite(
                                        sprite,
                                        rect,
                                        [1.0, 1.0, 1.0, 1.0],
                                        0.0,
                                        1.05,
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
                commands
            }
            Self::DrawShadowTexture {
                x,
                y,
                width,
                height,
            } => {
                let mut commands = Vec::with_capacity(world.width * world.height / 4 + 16);
                let texture_rect = menu_shadow_texture_rect(x, y, width, height);
                menu_push_shadow_tile_spans(
                    &mut commands,
                    world,
                    tile_size,
                    transform,
                    texture_rect,
                );
                commands
            }
            Self::DrawFlyer(flyer) => {
                let include_sprite_fallbacks = transform.is_none();
                let center = menu_transform_point(RenderPoint::new(flyer.x, flyer.y), transform);
                let size_scale = transform.map_or(1.0, |transform| transform.scaling);
                let body_size = flyer_draw_size(flyer.unit_name);
                let body_rect =
                    RenderRect::from_center(center, body_size * size_scale, body_size * size_scale);
                let shadow_rect = RenderRect::from_center(
                    center,
                    body_size * 1.15 * size_scale,
                    body_size * 1.15 * size_scale,
                );

                let mut commands = Vec::with_capacity(3);
                if include_sprite_fallbacks {
                    commands.push(RenderCommand::draw_sprite(
                        "circle-shadow",
                        shadow_rect,
                        [1.0, 1.0, 1.0, 1.0],
                        0.0,
                        1.5,
                    ));
                }
                commands.push(RenderCommand::draw_triangle(
                    center,
                    body_size * 0.72 * size_scale,
                    body_size * 0.98 * size_scale,
                    flyer.rotation,
                    [0.45, 0.62, 0.72, 0.42],
                    1.95,
                ));
                if include_sprite_fallbacks {
                    commands.push(RenderCommand::draw_sprite(
                        flyer.unit_name,
                        body_rect,
                        [1.0, 1.0, 1.0, 1.0],
                        flyer.rotation,
                        2.0,
                    ));
                }
                commands
            }
            Self::DrawDarkness {
                alpha,
                width,
                height,
            } => vec![RenderCommand::fill_rect(
                RenderRect::new(0.0, 0.0, width, height),
                [0.0, 0.0, 0.0, alpha.clamp(0.0, 1.0)],
                MENU_DARKNESS_LAYER,
            )],
        }
    }
}
fn menu_render_command_visible_in_viewport(
    command: &RenderCommand,
    viewport: RenderViewport,
) -> bool {
    let viewport_rect = viewport.as_rect().inflate(64.0);
    match command {
        RenderCommand::FillRect { rect, .. }
        | RenderCommand::StrokeRect { rect, .. }
        | RenderCommand::DrawSprite { rect, .. } => rect.intersects(viewport_rect),
        RenderCommand::DrawCircle { center, radius, .. } => {
            RenderRect::from_center(*center, radius.max(0.0) * 2.0, radius.max(0.0) * 2.0)
                .intersects(viewport_rect)
        }
        RenderCommand::DrawLine {
            from, to, stroke, ..
        } => {
            let left = from.x.min(to.x) - stroke.max(0.0);
            let bottom = from.y.min(to.y) - stroke.max(0.0);
            let right = from.x.max(to.x) + stroke.max(0.0);
            let top = from.y.max(to.y) + stroke.max(0.0);
            RenderRect::new(left, bottom, right - left, top - bottom).intersects(viewport_rect)
        }
        RenderCommand::DrawPolygon { center, radius, .. }
        | RenderCommand::DrawTriangle {
            center,
            width: radius,
            ..
        } => RenderRect::from_center(*center, radius.max(0.0) * 2.0, radius.max(0.0) * 2.0)
            .intersects(viewport_rect),
        RenderCommand::DrawPixel { x, y, .. } => {
            viewport_rect.contains_point(RenderPoint::new(*x as f32, *y as f32))
        }
        RenderCommand::DrawText { position, .. } => viewport_rect.contains_point(*position),
        RenderCommand::Clear { .. }
        | RenderCommand::SetBlend { .. }
        | RenderCommand::SetClip { .. }
        | RenderCommand::ClearClip
        | RenderCommand::Custom { .. } => true,
    }
}

fn menu_extend_visible_render_commands(
    pass: &mut RenderPass,
    commands: impl IntoIterator<Item = RenderCommand>,
    viewport: RenderViewport,
) {
    pass.extend(
        commands
            .into_iter()
            .filter(|command| menu_render_command_visible_in_viewport(command, viewport)),
    );
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuFramePlan {
    pub camera_x: f32,
    pub camera_y: f32,
    pub camera_width: f32,
    pub camera_height: f32,
    pub scaling: f32,
    pub tile_size: f32,
    pub world: MenuWorldPlan,
    pub commands: Vec<MenuRenderCommand>,
    pub ui: MenuUiPlan,
}

impl MenuFramePlan {
    pub fn to_render_pass(&self) -> Option<RenderPass> {
        if self.commands.is_empty() && self.ui.buttons.is_empty() {
            return None;
        }

        let viewport = RenderViewport::new(
            0.0,
            0.0,
            self.camera_width * self.scaling,
            self.camera_height * self.scaling,
        );
        let camera = RenderCamera::new(RenderPoint::new(self.camera_x, self.camera_y), viewport)
            .with_zoom(self.scaling);

        let mut pass = RenderPass::new(RenderPassKind::Custom("menu".to_string()))
            .with_viewport(viewport)
            .with_camera(camera);
        pass.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        let transform = MenuScreenTransform::new(
            self.camera_x - self.camera_width * 0.5,
            self.camera_y - self.camera_height * 0.5,
            self.scaling,
        );
        for command in &self.commands {
            menu_extend_visible_render_commands(
                &mut pass,
                command.to_render_commands_with_transform(&self.world, self.tile_size, transform),
                viewport,
            );
        }
        pass.extend(self.ui.to_render_commands());
        Some(pass)
    }

    pub fn into_render_pass(self) -> Option<RenderPass> {
        let Self {
            camera_x,
            camera_y,
            camera_width,
            camera_height,
            scaling,
            tile_size,
            world,
            commands,
            ui,
        } = self;

        if commands.is_empty() && ui.buttons.is_empty() {
            return None;
        }

        let viewport =
            RenderViewport::new(0.0, 0.0, camera_width * scaling, camera_height * scaling);
        let camera =
            RenderCamera::new(RenderPoint::new(camera_x, camera_y), viewport).with_zoom(scaling);

        let mut pass = RenderPass::new(RenderPassKind::Custom("menu".to_string()))
            .with_viewport(viewport)
            .with_camera(camera);
        pass.push(RenderCommand::clear([0.0, 0.0, 0.0, 1.0]));
        let transform = MenuScreenTransform::new(
            camera_x - camera_width * 0.5,
            camera_y - camera_height * 0.5,
            scaling,
        );
        for command in commands {
            menu_extend_visible_render_commands(
                &mut pass,
                command.into_render_commands_with_transform(&world, tile_size, Some(transform)),
                viewport,
            );
        }
        pass.extend(ui.to_render_commands());
        Some(pass)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuRendererState {
    pub config: MenuRendererConfig,
    pub world: MenuWorldPlan,
    pub time: f32,
    pub flyer_rotation: f32,
    pub flyer_count: usize,
    pub flyer_type: &'static str,
    pub selected_root: MenuButtonRole,
    active_root: Option<MenuButtonRole>,
    submenu_root: Option<MenuButtonRole>,
    pub submenu_alpha: f32,
    pub submenu_target_alpha: f32,
    submenu_animation_from_alpha: f32,
    submenu_animation_elapsed: f32,
    pub custom_buttons: Vec<MenuCustomButton>,
}

impl MenuRendererState {
    pub fn new(config: MenuRendererConfig) -> Self {
        let world = generate_menu_world(config);
        Self {
            config,
            world,
            time: 0.0,
            flyer_rotation: 45.0,
            flyer_count: flyer_count_for_seed(config.seed),
            flyer_type: flyer_type_for_seed(config.seed),
            selected_root: MenuButtonRole::Play,
            active_root: None,
            submenu_root: None,
            submenu_alpha: 0.0,
            submenu_target_alpha: 0.0,
            submenu_animation_from_alpha: 0.0,
            submenu_animation_elapsed: 0.0,
            custom_buttons: Vec::new(),
        }
    }

    pub fn render_plan(&mut self, input: MenuFrameInput) -> MenuFramePlan {
        self.time += input.delta;
        self.tick_submenu_alpha(input.delta);

        let world_width = (self.world.width as f32 - 1.0) * self.config.tile_size;
        let world_height = (self.world.height as f32 - 1.0) * self.config.tile_size;
        let scaling = input
            .scl4
            .max(input.graphics_width / world_width)
            .max(input.graphics_height / world_height);
        let camera_width = input.graphics_width / scaling;
        let camera_height = input.graphics_height / scaling;
        let camera_x = self.world.width as f32 * self.config.tile_size / 2.0;
        let camera_y = self.world.height as f32 * self.config.tile_size / 2.0;

        let mut commands = vec![
            MenuRenderCommand::DrawCache {
                cache_id: self.world.cache_floor_id,
                label: "floor+overlay",
            },
            MenuRenderCommand::DrawShadowTexture {
                x: camera_x - 4.0,
                y: camera_y - 4.0,
                width: self.world.width as f32 * self.config.tile_size,
                height: -(self.world.height as f32 * self.config.tile_size),
            },
            MenuRenderCommand::DrawCache {
                cache_id: self.world.cache_wall_id,
                label: "wall",
            },
        ];

        commands.extend(self.flyers().into_iter().map(MenuRenderCommand::DrawFlyer));
        commands.push(MenuRenderCommand::DrawDarkness {
            alpha: MENU_DARKNESS,
            width: input.graphics_width,
            height: input.graphics_height,
        });

        MenuFramePlan {
            camera_x,
            camera_y,
            camera_width,
            camera_height,
            scaling,
            tile_size: self.config.tile_size,
            world: self.world.clone(),
            commands,
            ui: menu_ui_plan(
                input,
                self.config.mobile,
                self.selected_root,
                self.active_root,
                self.submenu_root,
                self.submenu_alpha,
                self.submenu_target_alpha,
                self.config.desktop_workshop_enabled,
                self.config.mobile_ios,
                &self.custom_buttons,
            ),
        }
    }

    pub fn flyers(&self) -> Vec<FlyerPlan> {
        let tw = self.world.width as f32 * self.config.tile_size + self.config.tile_size;
        let th = self.world.height as f32 * self.config.tile_size + self.config.tile_size;
        let range = 500.0;
        let offset = -100.0;
        let (dir_x, dir_y) = trns(
            self.flyer_rotation,
            self.time * flyer_speed(self.flyer_type),
        );

        (0..self.flyer_count)
            .map(|i| {
                let x_mod = tw + random_seed(i as u64 + 5, 0.0, 500.0);
                FlyerPlan {
                    x: positive_mod(
                        random_seed_range(i as u64, range)
                            + dir_x
                            + absin(
                                self.time + random_seed_range(i as u64 + 2, 500.0),
                                10.0,
                                3.4,
                            )
                            + offset,
                        x_mod,
                    ),
                    y: positive_mod(
                        random_seed_range(i as u64 + 1, range)
                            + dir_y
                            + absin(
                                self.time + random_seed_range(i as u64 + 3, 500.0),
                                10.0,
                                3.4,
                            )
                            + offset,
                        th,
                    ),
                    rotation: self.flyer_rotation,
                    unit_name: self.flyer_type,
                }
            })
            .collect()
    }

    pub fn ui_plan(&self, input: MenuFrameInput) -> MenuUiPlan {
        menu_ui_plan(
            input,
            self.config.mobile,
            self.selected_root,
            self.active_root,
            self.submenu_root,
            self.submenu_alpha,
            self.submenu_target_alpha,
            self.config.desktop_workshop_enabled,
            self.config.mobile_ios,
            &self.custom_buttons,
        )
    }

    pub fn hit_test_ui(&self, input: MenuFrameInput, x: f32, y: f32) -> Option<MenuButtonRole> {
        self.ui_plan(input).hit_test(x, y)
    }

    pub fn role_has_desktop_submenu(&self, role: MenuButtonRole) -> bool {
        role.has_desktop_submenu()
            || self
                .custom_button(role)
                .map(MenuCustomButton::has_submenu)
                .unwrap_or(false)
    }

    pub fn select_desktop_root(&mut self, role: MenuButtonRole) -> bool {
        if self.config.mobile || !self.role_has_desktop_submenu(role) {
            return false;
        }
        if self.active_root == Some(role) {
            self.active_root = None;
            self.start_submenu_alpha_animation(0.0, Some(1.0));
            return true;
        }
        self.active_root = Some(role);
        self.selected_root = role;
        self.submenu_root = Some(role);
        self.start_submenu_alpha_animation(1.0, None);
        true
    }

    fn start_submenu_alpha_animation(&mut self, target_alpha: f32, from_alpha: Option<f32>) {
        let from_alpha = from_alpha.unwrap_or(self.submenu_alpha).clamp(0.0, 1.0);
        self.submenu_alpha = from_alpha;
        self.submenu_target_alpha = target_alpha.clamp(0.0, 1.0);
        self.submenu_animation_from_alpha = from_alpha;
        self.submenu_animation_elapsed = 0.0;
    }

    pub fn tick_submenu_alpha(&mut self, delta: f32) {
        let delta = delta.max(0.0);
        if (self.submenu_animation_from_alpha - self.submenu_target_alpha).abs() <= f32::EPSILON {
            self.submenu_alpha = self.submenu_target_alpha;
            self.submenu_animation_elapsed = 0.0;
            self.submenu_animation_from_alpha = self.submenu_target_alpha;
            return;
        }
        let duration = if self.submenu_target_alpha > self.submenu_animation_from_alpha {
            MENU_SUBMENU_FADE_IN_SECONDS
        } else {
            MENU_SUBMENU_FADE_OUT_SECONDS
        }
        .max(f32::EPSILON);
        self.submenu_animation_elapsed = (self.submenu_animation_elapsed + delta).min(duration);
        let progress = (self.submenu_animation_elapsed / duration).clamp(0.0, 1.0);
        let eased = menu_interp_fade(progress);
        self.submenu_alpha = self.submenu_animation_from_alpha
            + (self.submenu_target_alpha - self.submenu_animation_from_alpha) * eased;
        if progress >= 1.0 {
            self.submenu_alpha = self.submenu_target_alpha;
            self.submenu_animation_from_alpha = self.submenu_target_alpha;
            self.submenu_animation_elapsed = 0.0;
        }
    }

    pub fn add_custom_button(&mut self, label: impl Into<String>) -> MenuButtonRole {
        self.add_custom_button_with(MenuCustomButton::new(label))
    }

    pub fn add_custom_button_with(&mut self, button: MenuCustomButton) -> MenuButtonRole {
        let index = self.custom_buttons.len().min(u16::MAX as usize) as u16;
        self.custom_buttons.push(button);
        MenuButtonRole::Custom(index)
    }

    pub fn clear_custom_buttons(&mut self) {
        self.custom_buttons.clear();
    }

    pub fn custom_button(&self, role: MenuButtonRole) -> Option<&MenuCustomButton> {
        match role {
            MenuButtonRole::Custom(index) => self.custom_buttons.get(index as usize),
            MenuButtonRole::CustomSubmenu { root, item } => self
                .custom_buttons
                .get(root as usize)
                .and_then(|button| button.submenu_buttons.get(item as usize)),
            _ => None,
        }
    }

    pub fn custom_button_action_id(&self, role: MenuButtonRole) -> Option<&str> {
        self.custom_button(role)
            .and_then(|button| button.action_id.as_deref())
    }

    pub fn has_active_desktop_submenu(&self) -> bool {
        !self.config.mobile
            && (self.active_root.is_some()
                || self.submenu_root.is_some()
                || self.submenu_alpha > f32::EPSILON
                || self.submenu_target_alpha > f32::EPSILON)
    }

    pub fn reset_desktop_root(&mut self) {
        self.selected_root = MenuButtonRole::Play;
        self.active_root = None;
        self.submenu_root = None;
        self.submenu_alpha = 0.0;
        self.submenu_target_alpha = 0.0;
        self.submenu_animation_from_alpha = 0.0;
        self.submenu_animation_elapsed = 0.0;
    }
}

fn menu_interp_fade(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * progress * (progress * (progress * 6.0 - 15.0) + 10.0)
}

pub fn generate_menu_world(config: MenuRendererConfig) -> MenuWorldPlan {
    let width = config.width();
    let height = config.height();
    let selected = terrain_pair(config.seed, true);
    let selected2 = terrain_pair(config.seed.rotate_left(13), false);
    let ores = [
        MenuBlockKind::CopperOre,
        MenuBlockKind::LeadOre,
        MenuBlockKind::ScrapOre,
        MenuBlockKind::CoalOre,
        MenuBlockKind::TitaniumOre,
        MenuBlockKind::ThoriumOre,
    ];
    let ore1_index = (hash_u64(config.seed ^ 0x51) as usize) % ores.len();
    let ore1 = ores[ore1_index];
    let ore2 = ores[(ore1_index + 1 + (hash_u64(config.seed ^ 0x52) as usize % (ores.len() - 1)))
        % ores.len()];
    let tr1 = 0.65 + noise01(config.seed, 7, 0, 0) as f64 * 0.20;
    let tr2 = 0.65 + noise01(config.seed, 8, 0, 0) as f64 * 0.20;
    let do_heat = chance(config.seed, 11, 0.25);
    let tendrils = chance(config.seed, 12, 0.25);
    let tech = chance(config.seed, 13, 0.25);
    let sec_size = 10usize;

    let mut tiles = Vec::with_capacity(width * height);
    for x in 0..width {
        for y in 0..height {
            let mut floor = selected.0;
            let mut ore = MenuBlockKind::Air;
            let mut wall = MenuBlockKind::Air;

            if octave_noise(config.seed, 3, 0.5, 1.0 / 20.0, x, y, 1) > 0.5 {
                wall = selected.1;
            }

            if octave_noise(config.seed, 3, 0.5, 1.0 / 20.0, x, y, 3) > 0.5 {
                floor = selected2.0;
                if wall != MenuBlockKind::Air {
                    wall = selected2.1;
                }
            }

            if octave_noise(config.seed, 3, 0.3, 1.0 / 30.0, x, y, 2) > tr1 {
                ore = ore1;
            }
            if octave_noise(config.seed, 2, 0.2, 1.0 / 15.0, x, y + 99_999, 2) > tr2 {
                ore = ore2;
            }

            if do_heat {
                let heat = octave_noise(config.seed, 4, 0.6, 1.0 / 50.0, x, y + 9_999, 3);
                let base = 0.65;
                if heat > base {
                    ore = MenuBlockKind::Air;
                    wall = MenuBlockKind::Air;
                    floor = MenuBlockKind::Basalt;
                    if heat > base + 0.1 {
                        floor = MenuBlockKind::Hotrock;
                    }
                    if heat > base + 0.15 {
                        floor = MenuBlockKind::Magmarock;
                    }
                }
            }

            if tech {
                let mx = x % sec_size;
                let my = y % sec_size;
                let sclx = x / sec_size;
                let scly = y / sec_size;
                let on_border = mx == 0 || my == 0 || mx == sec_size - 1 || my == sec_size - 1;
                if octave_noise(config.seed, 2, 0.1, 0.5, sclx, scly, 1) > 0.4 && on_border {
                    floor = MenuBlockKind::DarkPanel3;
                    if distance(
                        mx as f32,
                        my as f32,
                        sec_size as f32 / 2.0,
                        sec_size as f32 / 2.0,
                    ) > sec_size as f32 / 2.0 + 1.0
                    {
                        floor = MenuBlockKind::DarkPanel4;
                    }
                    if wall != MenuBlockKind::Air
                        && chance(config.seed ^ ((x as u64) << 32 | y as u64), 21, 0.7)
                    {
                        wall = MenuBlockKind::DarkMetal;
                    }
                }
            }

            if tendrils && ridged_noise(config.seed.wrapping_add(1), x, y, 1.0 / 17.0) > 0.0 {
                floor = if chance(config.seed ^ ((x as u64) << 16 | y as u64), 31, 0.2) {
                    MenuBlockKind::SporeMoss
                } else {
                    MenuBlockKind::Moss
                };
                if wall != MenuBlockKind::Air {
                    wall = MenuBlockKind::SporeWall;
                }
            }

            tiles.push(MenuTile {
                x: x as u16,
                y: y as u16,
                floor,
                wall,
                ore,
            });
        }
    }

    MenuWorldPlan {
        width,
        height,
        seed: config.seed,
        tiles,
        cache_floor_id: 1,
        cache_wall_id: 2,
    }
}

fn terrain_pair(seed: u64, primary: bool) -> (MenuBlockKind, MenuBlockKind) {
    let primary_pairs = [
        (MenuBlockKind::Sand, MenuBlockKind::SandWall),
        (MenuBlockKind::Shale, MenuBlockKind::ShaleWall),
        (MenuBlockKind::Ice, MenuBlockKind::IceWall),
        (MenuBlockKind::Sand, MenuBlockKind::SandWall),
        (MenuBlockKind::Shale, MenuBlockKind::ShaleWall),
        (MenuBlockKind::Ice, MenuBlockKind::IceWall),
        (MenuBlockKind::Moss, MenuBlockKind::SporePine),
        (MenuBlockKind::Dirt, MenuBlockKind::DirtWall),
        (MenuBlockKind::Dacite, MenuBlockKind::DaciteWall),
    ];
    let secondary_pairs = [
        (MenuBlockKind::Basalt, MenuBlockKind::DuneWall),
        (MenuBlockKind::Basalt, MenuBlockKind::DuneWall),
        (MenuBlockKind::Stone, MenuBlockKind::StoneWall),
        (MenuBlockKind::Stone, MenuBlockKind::StoneWall),
        (MenuBlockKind::Moss, MenuBlockKind::SporeWall),
        (MenuBlockKind::Salt, MenuBlockKind::StoneWall),
    ];
    let pairs = if primary {
        &primary_pairs[..]
    } else {
        &secondary_pairs[..]
    };
    pairs[(hash_u64(seed) as usize) % pairs.len()]
}

fn flyer_count_for_seed(seed: u64) -> usize {
    if chance(seed, 101, 0.2) {
        (hash_u64(seed ^ 0x35) as usize) % 36
    } else {
        (hash_u64(seed ^ 0x15) as usize) % 16
    }
}

fn flyer_type_for_seed(seed: u64) -> &'static str {
    const TYPES: [&str; 9] = [
        "flare", "horizon", "zenith", "mono", "poly", "mega", "alpha", "beta", "gamma",
    ];
    TYPES[(hash_u64(seed ^ 0xAA) as usize) % TYPES.len()]
}

fn flyer_speed(unit_name: &str) -> f32 {
    match unit_name {
        "horizon" => 2.1,
        "zenith" => 1.7,
        "mono" => 2.6,
        "poly" => 2.4,
        "mega" => 1.9,
        "alpha" => 2.8,
        "beta" => 2.5,
        "gamma" => 2.2,
        _ => 3.0,
    }
}

fn octave_noise(
    seed: u64,
    octaves: usize,
    persistence: f64,
    scale: f64,
    x: usize,
    y: usize,
    salt: u64,
) -> f64 {
    let mut amplitude = 1.0;
    let mut frequency = scale;
    let mut value = 0.0;
    let mut total = 0.0;
    for octave in 0..octaves {
        value += noise01(
            seed ^ salt.wrapping_mul(0x9E37_79B9) ^ octave as u64,
            (x as f64 * frequency * 10_000.0) as i64,
            (y as f64 * frequency * 10_000.0) as i64,
            octave as i64,
        ) as f64
            * amplitude;
        total += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }
    if total == 0.0 {
        0.0
    } else {
        value / total
    }
}

fn ridged_noise(seed: u64, x: usize, y: usize, scale: f64) -> f64 {
    (noise01(
        seed,
        (x as f64 * scale * 10_000.0) as i64,
        (y as f64 * scale * 10_000.0) as i64,
        0,
    ) as f64
        - 0.5)
        * 2.0
}

fn noise01(seed: u64, x: i64, y: i64, salt: i64) -> f32 {
    let mut v = seed
        ^ (x as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ (y as u64).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ (salt as u64).wrapping_mul(0xD6E8_FD9D_5A75_7955);
    v = hash_u64(v);
    ((v >> 40) as f32) / ((1u64 << 24) as f32)
}

fn chance(seed: u64, salt: u64, probability: f32) -> bool {
    noise01(
        seed ^ salt,
        salt as i64,
        salt.rotate_left(7) as i64,
        salt as i64,
    ) < probability
}

fn hash_u64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn random_seed(seed: u64, min: f32, max: f32) -> f32 {
    min + (max - min) * noise01(seed, seed as i64, !seed as i64, 0)
}

fn random_seed_range(seed: u64, range: f32) -> f32 {
    random_seed(seed, -range, range)
}

fn absin(time: f32, scale: f32, magnitude: f32) -> f32 {
    (time / scale).sin().abs() * magnitude
}

fn trns(degrees: f32, length: f32) -> (f32, f32) {
    let radians = degrees.to_radians();
    (radians.cos() * length, radians.sin() * length)
}

fn positive_mod(value: f32, modulus: f32) -> f32 {
    if modulus == 0.0 {
        return 0.0;
    }
    let result = value % modulus;
    if result < 0.0 {
        result + modulus
    } else {
        result
    }
}

fn distance(x: f32, y: f32, x2: f32, y2: f32) -> f32 {
    ((x - x2).powi(2) + (y - y2).powi(2)).sqrt()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MenuScreenTransform {
    world_left: f32,
    world_bottom: f32,
    scaling: f32,
}

impl MenuScreenTransform {
    const fn new(world_left: f32, world_bottom: f32, scaling: f32) -> Self {
        Self {
            world_left,
            world_bottom,
            scaling,
        }
    }

    fn point(self, point: RenderPoint) -> RenderPoint {
        RenderPoint::new(
            (point.x - self.world_left) * self.scaling,
            (point.y - self.world_bottom) * self.scaling,
        )
    }

    fn rect(self, rect: RenderRect) -> RenderRect {
        let point = self.point(RenderPoint::new(rect.x, rect.y));
        RenderRect::new(
            point.x,
            point.y,
            rect.width * self.scaling,
            rect.height * self.scaling,
        )
    }
}

fn menu_transform_point(point: RenderPoint, transform: Option<MenuScreenTransform>) -> RenderPoint {
    transform.map_or(point, |transform| transform.point(point))
}

fn menu_transform_rect(rect: RenderRect, transform: Option<MenuScreenTransform>) -> RenderRect {
    transform.map_or(rect, |transform| transform.rect(rect))
}

fn menu_tile_rect(tile: &MenuTile, tile_size: f32) -> RenderRect {
    RenderRect::new(
        tile.x as f32 * tile_size,
        tile.y as f32 * tile_size,
        tile_size,
        tile_size,
    )
}

fn menu_tile_inset_rect(tile: &MenuTile, tile_size: f32, inset_ratio: f32) -> RenderRect {
    let inset = (tile_size * inset_ratio).clamp(0.0, tile_size * 0.45);
    RenderRect::new(
        tile.x as f32 * tile_size + inset,
        tile.y as f32 * tile_size + inset,
        (tile_size - inset * 2.0).max(1.0),
        (tile_size - inset * 2.0).max(1.0),
    )
}

fn menu_variant_index(tile: &MenuTile, variants: usize) -> usize {
    if variants <= 1 {
        return 1;
    }
    ((tile.x as usize).wrapping_mul(31) ^ (tile.y as usize).wrapping_mul(17)) % variants + 1
}

fn menu_numbered_sprite(prefix: &str, tile: &MenuTile, variants: usize) -> String {
    format!("{prefix}{}", menu_variant_index(tile, variants))
}

fn menu_block_variant_sprite_name(block: MenuBlockKind, tile: &MenuTile) -> Option<String> {
    let sprite = match block {
        MenuBlockKind::Air => return None,
        MenuBlockKind::Sand => menu_numbered_sprite("sand-floor", tile, 3),
        MenuBlockKind::SandWall => menu_numbered_sprite("sand-wall", tile, 2),
        MenuBlockKind::Shale => menu_numbered_sprite("shale", tile, 3),
        MenuBlockKind::ShaleWall => menu_numbered_sprite("shale-wall", tile, 2),
        MenuBlockKind::Ice => menu_numbered_sprite("ice-snow", tile, 3),
        MenuBlockKind::IceWall => menu_numbered_sprite("ice-wall", tile, 2),
        MenuBlockKind::Moss => menu_numbered_sprite("moss", tile, 3),
        MenuBlockKind::SporePine => "spore-pine".to_string(),
        MenuBlockKind::Dirt => menu_numbered_sprite("dirt", tile, 3),
        MenuBlockKind::DirtWall => menu_numbered_sprite("dirt-wall", tile, 2),
        MenuBlockKind::Dacite => menu_numbered_sprite("dacite", tile, 3),
        MenuBlockKind::DaciteWall => menu_numbered_sprite("dacite-wall", tile, 2),
        MenuBlockKind::Basalt => menu_numbered_sprite("basalt", tile, 3),
        MenuBlockKind::DuneWall => menu_numbered_sprite("dune-wall", tile, 2),
        MenuBlockKind::Stone => menu_numbered_sprite("stone", tile, 3),
        MenuBlockKind::StoneWall => menu_numbered_sprite("stone-wall", tile, 2),
        MenuBlockKind::SporeWall => menu_numbered_sprite("spore-wall", tile, 2),
        MenuBlockKind::Salt => "salt".to_string(),
        MenuBlockKind::CopperOre => menu_numbered_sprite("ore-copper", tile, 3),
        MenuBlockKind::LeadOre => menu_numbered_sprite("ore-lead", tile, 3),
        MenuBlockKind::ScrapOre => menu_numbered_sprite("ore-scrap", tile, 3),
        MenuBlockKind::CoalOre => menu_numbered_sprite("ore-coal", tile, 3),
        MenuBlockKind::TitaniumOre => menu_numbered_sprite("ore-titanium", tile, 3),
        MenuBlockKind::ThoriumOre => menu_numbered_sprite("ore-thorium", tile, 3),
        MenuBlockKind::Hotrock => menu_numbered_sprite("hotrock", tile, 3),
        MenuBlockKind::Magmarock => menu_numbered_sprite("magmarock", tile, 3),
        MenuBlockKind::DarkPanel3 => "dark-panel-3".to_string(),
        MenuBlockKind::DarkPanel4 => "dark-panel-4".to_string(),
        MenuBlockKind::DarkMetal => menu_numbered_sprite("dark-metal", tile, 2),
        MenuBlockKind::SporeMoss => menu_numbered_sprite("spore-moss", tile, 3),
    };
    Some(sprite)
}

fn menu_tile_at(world: &MenuWorldPlan, x: usize, y: usize) -> Option<&MenuTile> {
    let index = x.saturating_mul(world.height).saturating_add(y);
    if let Some(tile) = world.tiles.get(index) {
        if tile.x as usize == x && tile.y as usize == y {
            return Some(tile);
        }
    }
    world
        .tiles
        .iter()
        .find(|tile| tile.x as usize == x && tile.y as usize == y)
}

fn menu_push_horizontal_tile_color_spans<F>(
    commands: &mut Vec<RenderCommand>,
    world: &MenuWorldPlan,
    tile_size: f32,
    transform: Option<MenuScreenTransform>,
    layer: f32,
    mut color_for_tile: F,
) where
    F: FnMut(&MenuTile) -> Option<[f32; 4]>,
{
    for y in 0..world.height {
        let mut span_start: Option<usize> = None;
        let mut span_color: Option<[f32; 4]> = None;

        for x in 0..=world.width {
            let next_color = if x < world.width {
                menu_tile_at(world, x, y).and_then(|tile| color_for_tile(tile))
            } else {
                None
            };

            if next_color == span_color
                && span_start.map_or(true, |start| {
                    x.saturating_sub(start) < MENU_MAX_NATIVE_COLOR_SPAN_TILES
                })
            {
                continue;
            }

            if let (Some(start), Some(color)) = (span_start, span_color) {
                let rect = RenderRect::new(
                    start as f32 * tile_size,
                    y as f32 * tile_size,
                    (x - start) as f32 * tile_size,
                    tile_size,
                );
                commands.push(RenderCommand::fill_rect(
                    menu_transform_rect(rect, transform),
                    color,
                    layer,
                ));
            }

            span_start = next_color.map(|_| x);
            span_color = next_color;
        }
    }
}

fn menu_push_shadow_tile_spans(
    commands: &mut Vec<RenderCommand>,
    world: &MenuWorldPlan,
    tile_size: f32,
    transform: Option<MenuScreenTransform>,
    texture_rect: RenderRect,
) {
    for y in 0..world.height {
        let mut span_start: Option<usize> = None;

        for x in 0..=world.width {
            let has_wall = if x < world.width {
                menu_tile_at(world, x, y)
                    .map(|tile| tile.wall != MenuBlockKind::Air)
                    .unwrap_or(false)
            } else {
                false
            };

            if has_wall {
                if let Some(start) = span_start {
                    if x.saturating_sub(start) >= MENU_MAX_NATIVE_COLOR_SPAN_TILES {
                        let rect = RenderRect::new(
                            texture_rect.x + start as f32 * tile_size,
                            texture_rect.y + y as f32 * tile_size,
                            (x - start) as f32 * tile_size,
                            tile_size,
                        );
                        commands.push(RenderCommand::fill_rect(
                            menu_transform_rect(rect, transform),
                            [0.0, 0.0, 0.0, MENU_SHADOW_TEXTURE_ALPHA],
                            MENU_SHADOW_TEXTURE_LAYER,
                        ));
                        span_start = Some(x);
                    }
                } else {
                    span_start = Some(x);
                }
                continue;
            }

            if let Some(start) = span_start.take() {
                let rect = RenderRect::new(
                    texture_rect.x + start as f32 * tile_size,
                    texture_rect.y + y as f32 * tile_size,
                    (x - start) as f32 * tile_size,
                    tile_size,
                );
                commands.push(RenderCommand::fill_rect(
                    menu_transform_rect(rect, transform),
                    [0.0, 0.0, 0.0, MENU_SHADOW_TEXTURE_ALPHA],
                    MENU_SHADOW_TEXTURE_LAYER,
                ));
            }
        }
    }
}

fn flyer_draw_size(unit_name: &str) -> f32 {
    match unit_name {
        "horizon" | "zenith" => 18.0,
        "poly" | "mega" | "alpha" | "gamma" => 16.0,
        "mono" | "beta" => 14.0,
        _ => 15.0,
    }
}

fn menu_button_plan(role: MenuButtonRole, rect: RenderRect, selected: bool) -> MenuButtonPlan {
    MenuButtonPlan {
        role,
        label: role.label().to_string(),
        icon_name: None,
        rect,
        selected,
        hovered: false,
        pressed: false,
        submenu: role.is_submenu(),
    }
}

fn menu_custom_button_plan(
    index: usize,
    button: &MenuCustomButton,
    rect: RenderRect,
    selected: bool,
) -> MenuButtonPlan {
    MenuButtonPlan {
        role: MenuButtonRole::Custom(index.min(u16::MAX as usize) as u16),
        label: button.label.clone(),
        icon_name: button.icon_name.clone(),
        rect,
        selected,
        hovered: false,
        pressed: false,
        submenu: button.has_submenu(),
    }
}

fn menu_custom_submenu_button_plan(
    root: usize,
    item: usize,
    button: &MenuCustomButton,
    rect: RenderRect,
) -> MenuButtonPlan {
    MenuButtonPlan {
        role: MenuButtonRole::CustomSubmenu {
            root: root.min(u16::MAX as usize) as u16,
            item: item.min(u16::MAX as usize) as u16,
        },
        label: button.label.clone(),
        icon_name: button.icon_name.clone(),
        rect,
        selected: false,
        hovered: false,
        pressed: false,
        submenu: false,
    }
}

fn menu_mobile_button_plan(
    role: MenuButtonRole,
    rect: RenderRect,
    selected: bool,
) -> MenuButtonPlan {
    MenuButtonPlan {
        role,
        label: role.label().to_string(),
        icon_name: None,
        rect,
        selected,
        hovered: false,
        pressed: false,
        submenu: false,
    }
}

fn menu_desktop_ui_plan(
    input: MenuFrameInput,
    selected_root: MenuButtonRole,
    active_root: Option<MenuButtonRole>,
    submenu_root: Option<MenuButtonRole>,
    submenu_alpha: f32,
    submenu_target_alpha: f32,
    desktop_workshop_enabled: bool,
    custom_buttons: &[MenuCustomButton],
) -> MenuUiPlan {
    let button_width = MENU_DESKTOP_BUTTON_WIDTH;
    let button_height = MENU_DESKTOP_BUTTON_HEIGHT;
    let gap = 0.0;
    let mut main_roles = vec![
        MenuButtonRole::Play,
        MenuButtonRole::Database,
        MenuButtonRole::Editor,
        MenuButtonRole::Mods,
        MenuButtonRole::Settings,
    ];
    if desktop_workshop_enabled {
        main_roles.insert(3, MenuButtonRole::Workshop);
    }
    let play_submenu_roles = [
        MenuButtonRole::Campaign,
        MenuButtonRole::Join,
        MenuButtonRole::CustomGame,
        MenuButtonRole::LoadGame,
    ];
    let database_submenu_roles = [
        MenuButtonRole::Schematics,
        MenuButtonRole::ContentDatabase,
        MenuButtonRole::About,
    ];
    let submenu_visible = submenu_alpha > f32::EPSILON || submenu_target_alpha > f32::EPSILON;
    let submenu_root = submenu_root.unwrap_or(selected_root);
    let submenu_roles: &[MenuButtonRole] = match submenu_root {
        MenuButtonRole::Play if submenu_visible => &play_submenu_roles,
        MenuButtonRole::Database if submenu_visible => &database_submenu_roles,
        _ => &[],
    };
    let custom_submenu_buttons: &[MenuCustomButton] = match submenu_root {
        MenuButtonRole::Custom(root) if submenu_visible => custom_buttons
            .get(root as usize)
            .map(|button| button.submenu_buttons.as_slice())
            .unwrap_or(&[]),
        _ => &[],
    };
    let main_role_count = main_roles.len();
    let main_button_count = main_role_count + custom_buttons.len() + 1;
    let total_height =
        main_button_count as f32 * button_height + main_button_count.saturating_sub(1) as f32 * gap;
    let start_y = ((input.graphics_height - total_height) * 0.5).max(0.0);
    let left_x = (input.graphics_width / 10.0).max(0.0);
    let submenu_x = left_x + button_width;
    let main_button_y = |index: usize| {
        start_y
            + main_button_count.saturating_sub(1).saturating_sub(index) as f32
                * (button_height + gap)
    };
    let selected_root_index = match submenu_root {
        MenuButtonRole::Custom(root) => main_role_count + root as usize,
        _ => main_roles
            .iter()
            .position(|role| *role == submenu_root)
            .unwrap_or(0),
    };
    let selected_root_y = main_button_y(selected_root_index);
    let submenu_start_y =
        (selected_root_y + input.scene_margin_top.max(0.0) + input.scene_margin_bottom.max(0.0))
            .max(0.0);

    let mut buttons =
        Vec::with_capacity(main_button_count + submenu_roles.len() + custom_submenu_buttons.len());
    for (index, role) in main_roles.iter().copied().enumerate() {
        buttons.push(menu_button_plan(
            role,
            RenderRect::new(left_x, main_button_y(index), button_width, button_height),
            active_root == Some(role),
        ));
    }
    for (custom_index, custom) in custom_buttons.iter().enumerate() {
        let index = main_role_count + custom_index;
        let role = MenuButtonRole::Custom(custom_index.min(u16::MAX as usize) as u16);
        buttons.push(menu_custom_button_plan(
            custom_index,
            custom,
            RenderRect::new(left_x, main_button_y(index), button_width, button_height),
            active_root == Some(role),
        ));
    }
    let quit_index = main_role_count + custom_buttons.len();
    buttons.push(menu_button_plan(
        MenuButtonRole::Quit,
        RenderRect::new(
            left_x,
            main_button_y(quit_index),
            button_width,
            button_height,
        ),
        active_root == Some(MenuButtonRole::Quit),
    ));
    for (index, role) in submenu_roles.iter().copied().enumerate() {
        buttons.push(menu_button_plan(
            role,
            RenderRect::new(
                submenu_x,
                submenu_start_y - index as f32 * (button_height + gap),
                button_width,
                button_height,
            ),
            false,
        ));
    }
    if let MenuButtonRole::Custom(root) = submenu_root {
        for (index, custom) in custom_submenu_buttons.iter().enumerate() {
            buttons.push(menu_custom_submenu_button_plan(
                root as usize,
                index,
                custom,
                RenderRect::new(
                    submenu_x,
                    submenu_start_y - index as f32 * (button_height + gap),
                    button_width,
                    button_height,
                ),
            ));
        }
    }

    MenuUiPlan {
        mobile: false,
        submenu_alpha: submenu_alpha.clamp(0.0, 1.0),
        buttons,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MenuMobileEntry {
    role: MenuButtonRole,
    label: String,
    selected: bool,
    icon_name: Option<String>,
}

fn menu_mobile_button_entry(role: MenuButtonRole) -> MenuMobileEntry {
    MenuMobileEntry {
        role,
        label: role.label().to_string(),
        selected: false,
        icon_name: None,
    }
}

fn menu_mobile_custom_entry(index: usize, button: &MenuCustomButton) -> MenuMobileEntry {
    MenuMobileEntry {
        role: MenuButtonRole::Custom(index.min(u16::MAX as usize) as u16),
        label: button.label.clone(),
        selected: false,
        icon_name: button.icon_name.clone(),
    }
}

fn menu_mobile_ui_plan(
    input: MenuFrameInput,
    custom_buttons: &[MenuCustomButton],
    mobile_ios: bool,
) -> MenuUiPlan {
    let final_role = if mobile_ios {
        MenuButtonRole::About
    } else {
        MenuButtonRole::Quit
    };
    let rows: Vec<Vec<MenuMobileEntry>> = if input.graphics_width > input.graphics_height {
        let mut first = vec![
            menu_mobile_button_entry(MenuButtonRole::Campaign),
            menu_mobile_button_entry(MenuButtonRole::Join),
            menu_mobile_button_entry(MenuButtonRole::CustomGame),
            menu_mobile_button_entry(MenuButtonRole::LoadGame),
        ];
        for index in (1..custom_buttons.len()).step_by(2) {
            first.push(menu_mobile_custom_entry(index, &custom_buttons[index]));
        }
        let mut second = vec![
            menu_mobile_button_entry(MenuButtonRole::Editor),
            menu_mobile_button_entry(MenuButtonRole::Settings),
            menu_mobile_button_entry(MenuButtonRole::Mods),
        ];
        for index in (0..custom_buttons.len()).step_by(2) {
            second.push(menu_mobile_custom_entry(index, &custom_buttons[index]));
        }
        second.push(menu_mobile_button_entry(final_role));
        vec![first, second]
    } else {
        let mut rows = vec![
            vec![
                menu_mobile_button_entry(MenuButtonRole::Campaign),
                menu_mobile_button_entry(MenuButtonRole::LoadGame),
            ],
            vec![
                menu_mobile_button_entry(MenuButtonRole::CustomGame),
                menu_mobile_button_entry(MenuButtonRole::Join),
            ],
            vec![
                menu_mobile_button_entry(MenuButtonRole::Editor),
                menu_mobile_button_entry(MenuButtonRole::Settings),
            ],
        ];
        let mut current = vec![menu_mobile_button_entry(MenuButtonRole::Mods)];
        for (index, custom) in custom_buttons.iter().enumerate() {
            current.push(menu_mobile_custom_entry(index, custom));
            if index % 2 == 0 {
                rows.push(current);
                current = Vec::new();
            }
        }
        current.push(menu_mobile_button_entry(final_role));
        rows.push(current);
        rows
    };
    let button_size = 120.0;
    let gap = 10.0;
    let column_count = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    let total_width =
        column_count as f32 * button_size + column_count.saturating_sub(1) as f32 * gap;
    let start_x = ((input.graphics_width - total_width) * 0.5).max(0.0);
    let start_y = if input.graphics_width > input.graphics_height {
        60.0
    } else {
        0.0
    };

    let mut buttons = Vec::with_capacity(rows.iter().map(|row| row.len()).sum());
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, entry) in row.iter().enumerate() {
            let mut button = menu_mobile_button_plan(
                entry.role,
                RenderRect::new(
                    start_x + column_index as f32 * (button_size + gap),
                    start_y + row_index as f32 * (button_size + gap),
                    button_size,
                    button_size,
                ),
                entry.selected,
            );
            button.label = entry.label.clone();
            button.icon_name = entry.icon_name.clone();
            buttons.push(button);
        }
    }

    MenuUiPlan {
        mobile: true,
        submenu_alpha: 0.0,
        buttons,
    }
}

fn menu_ui_plan(
    input: MenuFrameInput,
    mobile: bool,
    selected_root: MenuButtonRole,
    active_root: Option<MenuButtonRole>,
    submenu_root: Option<MenuButtonRole>,
    submenu_alpha: f32,
    submenu_target_alpha: f32,
    desktop_workshop_enabled: bool,
    mobile_ios: bool,
    custom_buttons: &[MenuCustomButton],
) -> MenuUiPlan {
    if mobile {
        menu_mobile_ui_plan(input, custom_buttons, mobile_ios)
    } else {
        menu_desktop_ui_plan(
            input,
            selected_root,
            active_root,
            submenu_root,
            submenu_alpha,
            submenu_target_alpha,
            desktop_workshop_enabled,
            custom_buttons,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ui::upstream_ui_icon_glyph_string;

    #[test]
    fn menu_dimensions_match_upstream_mobile_branch() {
        assert_eq!(MenuRendererConfig::new(false, 1).width(), 100);
        assert_eq!(MenuRendererConfig::new(false, 1).height(), 50);
        assert_eq!(MenuRendererConfig::new(true, 1).width(), 60);
        assert_eq!(MenuRendererConfig::new(true, 1).height(), 40);
    }

    #[test]
    fn generate_menu_world_creates_full_tile_grid() {
        let config = MenuRendererConfig::new(false, 42);
        let world = generate_menu_world(config);

        assert_eq!(world.tiles.len(), 100 * 50);
        assert_eq!(world.tiles[0].x, 0);
        assert_eq!(world.tiles[0].y, 0);
        assert_eq!(world.tiles.last().unwrap().x, 99);
        assert_eq!(world.tiles.last().unwrap().y, 49);
        assert_eq!(world.cache_floor_id, 1);
        assert_eq!(world.cache_wall_id, 2);
    }

    #[test]
    fn menu_renderer_state_starts_without_a_visible_desktop_submenu() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 0.0,
        });

        assert_eq!(state.selected_root, MenuButtonRole::Play);
        assert_eq!(state.active_root, None);
        assert_eq!(state.submenu_alpha, 0.0);
        assert_eq!(state.submenu_target_alpha, 0.0);
        assert_eq!(plan.ui.submenu_alpha, 0.0);
        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Play,
                MenuButtonRole::Database,
                MenuButtonRole::Editor,
                MenuButtonRole::Mods,
                MenuButtonRole::Settings,
                MenuButtonRole::Quit,
            ]
        );
        assert!(plan.ui.buttons.iter().all(|button| !button.selected));
        assert!(plan.ui.buttons.iter().all(|button| !button.submenu));
        assert!(plan.ui.hit_test(0.0, 0.0).is_none());
    }

    #[test]
    fn render_plan_keeps_java_cache_shadow_wall_flyers_darkness_order() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 7));
        state.flyer_count = 2;
        assert!(state.select_desktop_root(MenuButtonRole::Play));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1920.0,
            graphics_height: 1080.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0,
        });

        assert_eq!(plan.camera_x, 400.0);
        assert_eq!(plan.camera_y, 200.0);
        assert!(plan.scaling >= 4.0);
        assert!(matches!(
            plan.commands[0],
            MenuRenderCommand::DrawCache {
                label: "floor+overlay",
                ..
            }
        ));
        assert!(matches!(
            plan.commands[1],
            MenuRenderCommand::DrawShadowTexture {
                x,
                y,
                width,
                height,
            } if (x - 396.0).abs() < f32::EPSILON
                && (y - 196.0).abs() < f32::EPSILON
                && (width - 800.0).abs() < f32::EPSILON
                && (height + 400.0).abs() < f32::EPSILON
        ));
        assert!(matches!(
            plan.commands[2],
            MenuRenderCommand::DrawCache { label: "wall", .. }
        ));
        assert!(matches!(
            plan.commands.last().unwrap(),
            MenuRenderCommand::DrawDarkness {
                alpha: MENU_DARKNESS,
                ..
            }
        ));
        assert_eq!(plan.commands.len(), 6);
        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Play,
                MenuButtonRole::Database,
                MenuButtonRole::Editor,
                MenuButtonRole::Mods,
                MenuButtonRole::Settings,
                MenuButtonRole::Quit,
                MenuButtonRole::Campaign,
                MenuButtonRole::Join,
                MenuButtonRole::CustomGame,
                MenuButtonRole::LoadGame,
            ]
        );
        assert!(plan.ui.buttons[0].selected);
        assert!(plan.ui.buttons[6].submenu);
        assert!(plan.ui.submenu_alpha > f32::EPSILON);
        let play_center = plan.ui.buttons[0].rect.center();
        let campaign_center = plan.ui.buttons[6].rect.center();
        assert_eq!(
            plan.ui.hit_test(play_center.x, play_center.y),
            Some(MenuButtonRole::Play)
        );
        assert_eq!(
            plan.ui.hit_test(campaign_center.x, campaign_center.y),
            Some(MenuButtonRole::Campaign)
        );
        assert_eq!(plan.ui.hit_test(0.0, 0.0), None);
    }

    #[test]
    fn menu_frame_plan_to_render_pass_preserves_menu_command_order() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 7));
        state.flyer_count = 1;
        assert!(state.select_desktop_root(MenuButtonRole::Play));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1920.0,
            graphics_height: 1080.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0,
        });

        let borrowed = plan
            .to_render_pass()
            .expect("menu plan should produce a pass");
        let owned = plan
            .clone()
            .into_render_pass()
            .expect("menu plan should produce a pass");

        assert_eq!(borrowed, owned);
        assert_eq!(borrowed.kind, RenderPassKind::Custom("menu".to_string()));
        assert_eq!(
            borrowed.order,
            RenderPassKind::Custom("menu".to_string()).default_order()
        );
        assert_eq!(
            borrowed.viewport,
            Some(RenderViewport::new(0.0, 0.0, 1920.0, 1080.0))
        );
        assert_eq!(
            borrowed.camera,
            Some(
                RenderCamera::new(
                    RenderPoint::new(400.0, 200.0),
                    RenderViewport::new(0.0, 0.0, 1920.0, 1080.0),
                )
                .with_zoom(4.0)
            )
        );
        assert!(!borrowed.commands.is_empty());
        assert!(borrowed
            .commands
            .iter()
            .all(|command| !matches!(command, RenderCommand::Custom { .. })));
        assert!(borrowed
            .commands
            .iter()
            .any(|command| matches!(command, RenderCommand::DrawSprite { .. })));
        assert!(borrowed
            .commands
            .iter()
            .any(|command| matches!(command, RenderCommand::FillRect { .. })));
        assert!(borrowed.commands.iter().any(
            |command| matches!(command, RenderCommand::DrawText { text, .. } if text == "Play")
        ));
        assert!(borrowed.commands.iter().any(
            |command| matches!(command, RenderCommand::DrawText { text, .. } if text == "Campaign")
        ));

        assert!(borrowed.commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::FillRect { rect, color, layer }
                    if *rect == RenderRect::new(0.0, 0.0, 1920.0, 1080.0)
                        && *color == [0.0, 0.0, 0.0, MENU_DARKNESS]
                        && (*layer - MENU_DARKNESS_LAYER).abs() < f32::EPSILON
            )
        }));

        let darkness_command = plan
            .commands
            .last()
            .expect("menu frame should end with darkness");
        let darkness_render_commands =
            darkness_command.to_render_commands(&plan.world, plan.tile_size);
        assert_eq!(darkness_render_commands.len(), 1);
        assert!(matches!(
            darkness_render_commands[0],
            RenderCommand::FillRect { rect, color, layer }
                if rect == RenderRect::new(0.0, 0.0, 1920.0, 1080.0)
                    && color == [0.0, 0.0, 0.0, MENU_DARKNESS]
                    && (layer - MENU_DARKNESS_LAYER).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn menu_shadow_texture_offsets_wall_tiles_like_java_shadow_buffer() {
        let world = MenuWorldPlan {
            width: 2,
            height: 2,
            seed: 1,
            tiles: vec![MenuTile {
                x: 1,
                y: 1,
                floor: MenuBlockKind::Sand,
                wall: MenuBlockKind::SandWall,
                ore: MenuBlockKind::Air,
            }],
            cache_floor_id: 1,
            cache_wall_id: 2,
        };
        let commands = MenuRenderCommand::DrawShadowTexture {
            x: 4.0,
            y: 4.0,
            width: 16.0,
            height: -16.0,
        }
        .to_render_commands(&world, MENU_TILE_SIZE);

        assert!(commands.iter().any(|command| matches!(
            command,
            RenderCommand::FillRect { rect, color, layer }
                if (rect.x - 4.0).abs() < f32::EPSILON
                    && (rect.y - 4.0).abs() < f32::EPSILON
                    && (rect.width - MENU_TILE_SIZE).abs() < f32::EPSILON
                    && (rect.height - MENU_TILE_SIZE).abs() < f32::EPSILON
                    && *color == [0.0, 0.0, 0.0, MENU_SHADOW_TEXTURE_ALPHA]
                    && (*layer - MENU_SHADOW_TEXTURE_LAYER).abs() < f32::EPSILON
        )));
    }

    #[test]
    fn menu_shadow_texture_without_wall_tiles_stays_transparent_like_java_framebuffer() {
        let world = MenuWorldPlan {
            width: 2,
            height: 2,
            seed: 1,
            tiles: vec![MenuTile {
                x: 0,
                y: 0,
                floor: MenuBlockKind::Sand,
                wall: MenuBlockKind::Air,
                ore: MenuBlockKind::Air,
            }],
            cache_floor_id: 1,
            cache_wall_id: 2,
        };

        let commands = MenuRenderCommand::DrawShadowTexture {
            x: 4.0,
            y: 4.0,
            width: 16.0,
            height: -16.0,
        }
        .to_render_commands(&world, MENU_TILE_SIZE);

        assert!(
            commands.is_empty(),
            "Java shadow framebuffer starts clear; without wall pixels, drawing it must not add a black fallback rectangle"
        );
    }

    #[test]
    fn menu_flyer_commands_include_native_silhouette_before_unit_sprite() {
        let flyer = FlyerPlan {
            x: 24.0,
            y: 32.0,
            rotation: 45.0,
            unit_name: "mono",
        };
        let commands = MenuRenderCommand::DrawFlyer(flyer).to_render_commands(
            &MenuWorldPlan {
                width: 1,
                height: 1,
                seed: 1,
                tiles: Vec::new(),
                cache_floor_id: 1,
                cache_wall_id: 2,
            },
            MENU_TILE_SIZE,
        );
        let silhouette_index = commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    RenderCommand::DrawTriangle { center, rotation, layer, .. }
                        if *center == RenderPoint::new(24.0, 32.0)
                            && (*rotation - 45.0).abs() < f32::EPSILON
                            && (*layer - 1.95).abs() < f32::EPSILON
                )
            })
            .expect("flyer should include a native triangle silhouette fallback");
        let sprite_index = commands
            .iter()
            .position(|command| {
                matches!(
                    command,
                    RenderCommand::DrawSprite { symbol, layer, .. }
                        if symbol == "mono" && (*layer - 2.0).abs() < f32::EPSILON
                )
            })
            .expect("flyer should still draw the upstream unit sprite");
        assert!(silhouette_index < sprite_index);
    }

    #[test]
    fn menu_flyer_screen_transform_uses_native_silhouette_without_missing_sprite_blocks() {
        let flyer = FlyerPlan {
            x: 24.0,
            y: 32.0,
            rotation: 45.0,
            unit_name: "mono",
        };
        let commands = MenuRenderCommand::DrawFlyer(flyer).to_render_commands_with_transform(
            &MenuWorldPlan {
                width: 1,
                height: 1,
                seed: 1,
                tiles: Vec::new(),
                cache_floor_id: 1,
                cache_wall_id: 2,
            },
            MENU_TILE_SIZE,
            MenuScreenTransform::new(0.0, 0.0, 1.0),
        );

        assert!(commands
            .iter()
            .any(|command| matches!(command, RenderCommand::DrawTriangle { .. })));
        assert!(
            commands
                .iter()
                .all(|command| !matches!(command, RenderCommand::DrawSprite { .. })),
            "native menu screen path should not show magenta missing-texture blocks for flyers"
        );
    }

    #[test]
    fn menu_cache_render_commands_emit_real_tile_sprite_symbols_with_color_fallbacks() {
        let world = MenuWorldPlan {
            width: 1,
            height: 1,
            seed: 1,
            tiles: vec![MenuTile {
                x: 0,
                y: 0,
                floor: MenuBlockKind::Sand,
                wall: MenuBlockKind::SandWall,
                ore: MenuBlockKind::CopperOre,
            }],
            cache_floor_id: 1,
            cache_wall_id: 2,
        };

        let floor = MenuRenderCommand::DrawCache {
            cache_id: 1,
            label: "floor+overlay",
        }
        .to_render_commands(&world, MENU_TILE_SIZE);
        assert!(floor.iter().any(|command| matches!(
            command,
            RenderCommand::FillRect { layer, .. } if (*layer + 0.2).abs() < f32::EPSILON
        )));
        assert!(floor.iter().any(|command| matches!(
            command,
            RenderCommand::DrawSprite { symbol, layer, .. }
                if symbol == "sand-floor1" && (*layer + 0.1).abs() < f32::EPSILON
        )));
        assert!(floor.iter().any(|command| matches!(
            command,
            RenderCommand::DrawSprite { symbol, layer, .. }
                if symbol == "ore-copper1" && (*layer - 0.15).abs() < f32::EPSILON
        )));

        let wall = MenuRenderCommand::DrawCache {
            cache_id: 2,
            label: "wall",
        }
        .to_render_commands(&world, MENU_TILE_SIZE);
        assert!(wall.iter().any(|command| matches!(
            command,
            RenderCommand::DrawSprite { symbol, layer, .. }
                if symbol == "sand-wall1" && (*layer - 1.05).abs() < f32::EPSILON
        )));
    }

    #[test]
    fn menu_cache_screen_transform_batches_static_tile_color_spans() {
        let world = MenuWorldPlan {
            width: 3,
            height: 1,
            seed: 1,
            tiles: vec![
                MenuTile {
                    x: 0,
                    y: 0,
                    floor: MenuBlockKind::Sand,
                    wall: MenuBlockKind::SandWall,
                    ore: MenuBlockKind::Air,
                },
                MenuTile {
                    x: 1,
                    y: 0,
                    floor: MenuBlockKind::Sand,
                    wall: MenuBlockKind::SandWall,
                    ore: MenuBlockKind::Air,
                },
                MenuTile {
                    x: 2,
                    y: 0,
                    floor: MenuBlockKind::Sand,
                    wall: MenuBlockKind::Air,
                    ore: MenuBlockKind::Air,
                },
            ],
            cache_floor_id: 1,
            cache_wall_id: 2,
        };
        let transform = MenuScreenTransform::new(0.0, 0.0, 1.0);

        let floor = MenuRenderCommand::DrawCache {
            cache_id: 1,
            label: "floor+overlay",
        }
        .to_render_commands_with_transform(&world, MENU_TILE_SIZE, transform);
        let floor_fills = floor
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    RenderCommand::FillRect { layer, .. } if (*layer + 0.2).abs() < f32::EPSILON
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(floor_fills.len(), 1);
        assert!(matches!(
            floor_fills[0],
            RenderCommand::FillRect { rect, .. }
                if *rect == RenderRect::new(0.0, 0.0, MENU_TILE_SIZE * 3.0, MENU_TILE_SIZE)
        ));
        assert!(
            floor
                .iter()
                .all(|command| !matches!(command, RenderCommand::DrawSprite { .. })),
            "screen render path should not allocate tile sprites until native ordering can preserve UI over the menu world"
        );

        let wall = MenuRenderCommand::DrawCache {
            cache_id: 2,
            label: "wall",
        }
        .to_render_commands_with_transform(&world, MENU_TILE_SIZE, transform);
        let wall_fills = wall
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    RenderCommand::FillRect { layer, .. } if (*layer - 1.0).abs() < f32::EPSILON
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(wall_fills.len(), 1);
        assert!(matches!(
            wall_fills[0],
            RenderCommand::FillRect { rect, .. }
                if *rect == RenderRect::new(0.0, 0.0, MENU_TILE_SIZE * 2.0, MENU_TILE_SIZE)
        ));
    }

    #[test]
    fn menu_shadow_texture_batches_adjacent_wall_tiles_for_native_screen_path() {
        let world = MenuWorldPlan {
            width: 3,
            height: 1,
            seed: 1,
            tiles: vec![
                MenuTile {
                    x: 0,
                    y: 0,
                    floor: MenuBlockKind::Sand,
                    wall: MenuBlockKind::SandWall,
                    ore: MenuBlockKind::Air,
                },
                MenuTile {
                    x: 1,
                    y: 0,
                    floor: MenuBlockKind::Sand,
                    wall: MenuBlockKind::SandWall,
                    ore: MenuBlockKind::Air,
                },
                MenuTile {
                    x: 2,
                    y: 0,
                    floor: MenuBlockKind::Sand,
                    wall: MenuBlockKind::Air,
                    ore: MenuBlockKind::Air,
                },
            ],
            cache_floor_id: 1,
            cache_wall_id: 2,
        };

        let commands = MenuRenderCommand::DrawShadowTexture {
            x: 12.0,
            y: 4.0,
            width: 24.0,
            height: -8.0,
        }
        .to_render_commands_with_transform(
            &world,
            MENU_TILE_SIZE,
            MenuScreenTransform::new(0.0, 0.0, 1.0),
        );

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            commands[0],
            RenderCommand::FillRect { rect, color, layer }
                if rect == RenderRect::new(0.0, 0.0, MENU_TILE_SIZE * 2.0, MENU_TILE_SIZE)
                    && color == [0.0, 0.0, 0.0, MENU_SHADOW_TEXTURE_ALPHA]
                    && (layer - MENU_SHADOW_TEXTURE_LAYER).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn menu_ore_sprite_names_use_upstream_floor_region_symbols() {
        assert_eq!(MenuBlockKind::CopperOre.sprite_name(), Some("ore-copper"));
        assert_eq!(MenuBlockKind::LeadOre.sprite_name(), Some("ore-lead"));
        assert_eq!(MenuBlockKind::ScrapOre.sprite_name(), Some("ore-scrap"));
        assert_eq!(MenuBlockKind::CoalOre.sprite_name(), Some("ore-coal"));
        assert_eq!(
            MenuBlockKind::TitaniumOre.sprite_name(),
            Some("ore-titanium")
        );
        assert_eq!(MenuBlockKind::ThoriumOre.sprite_name(), Some("ore-thorium"));
    }

    #[test]
    fn menu_renderer_state_switches_desktop_submenu_roots() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        });

        assert_eq!(state.selected_root, MenuButtonRole::Database);
        assert!(plan
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Database && button.selected));
        assert!(plan
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Schematics));
        assert!(plan
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::ContentDatabase));
        assert!(plan
            .ui
            .buttons
            .iter()
            .all(|button| button.role != MenuButtonRole::Campaign));
        assert!(!state.select_desktop_root(MenuButtonRole::Quit));
        assert_eq!(state.selected_root, MenuButtonRole::Database);
    }

    #[test]
    fn menu_renderer_state_fades_out_and_in_current_desktop_submenu_like_java_actions() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        let input = |delta| MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta,
        };

        assert_eq!(state.selected_root, MenuButtonRole::Play);
        assert_eq!(state.active_root, None);
        assert_eq!(state.submenu_alpha, 0.0);
        assert_eq!(state.submenu_target_alpha, 0.0);
        assert!(state.select_desktop_root(MenuButtonRole::Play));
        assert_eq!(state.active_root, Some(MenuButtonRole::Play));
        assert_eq!(state.submenu_target_alpha, 1.0);

        let opened = state.render_plan(input(0.0));
        assert!(opened
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Campaign));
        assert!(opened
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Play && button.selected));

        let fading_in = state.render_plan(input(MENU_SUBMENU_FADE_IN_SECONDS * 0.5));
        assert!((state.submenu_alpha - 0.5).abs() < 0.0001);
        assert!((fading_in.ui.submenu_alpha - 0.5).abs() < 0.0001);
        assert!(fading_in
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Campaign));

        assert!(state.select_desktop_root(MenuButtonRole::Play));
        assert_eq!(state.active_root, None);
        assert_eq!(state.submenu_target_alpha, 0.0);
        assert_eq!(
            state.submenu_alpha, 1.0,
            "Java fadeOutMenu() runs Actions.alpha(1f) before fading to 0"
        );

        let fading_out = state.render_plan(input(MENU_SUBMENU_FADE_OUT_SECONDS * 0.25));
        let expected_fade_out_alpha = 1.0 - menu_interp_fade(0.25);
        assert!((state.submenu_alpha - expected_fade_out_alpha).abs() < 0.0001);
        assert!((fading_out.ui.submenu_alpha - expected_fade_out_alpha).abs() < 0.0001);
        assert!(fading_out
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Campaign));

        let faded_out = state.render_plan(input(MENU_SUBMENU_FADE_OUT_SECONDS));
        assert_eq!(state.submenu_alpha, 0.0);
        assert_eq!(faded_out.ui.submenu_alpha, 0.0);
        assert!(faded_out
            .ui
            .buttons
            .iter()
            .all(|button| button.role != MenuButtonRole::Campaign));

        assert!(state.select_desktop_root(MenuButtonRole::Play));
        assert_eq!(state.submenu_target_alpha, 1.0);
        let fading_in = state.render_plan(input(MENU_SUBMENU_FADE_IN_SECONDS * 0.5));
        assert!((state.submenu_alpha - 0.5).abs() < 0.0001);
        assert!((fading_in.ui.submenu_alpha - 0.5).abs() < 0.0001);
        assert!(fading_in
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Campaign));
    }

    #[test]
    fn menu_submenu_alpha_uses_java_interp_fade_curve() {
        assert_eq!(menu_interp_fade(0.0), 0.0);
        assert_eq!(menu_interp_fade(1.0), 1.0);
        assert!((menu_interp_fade(0.5) - 0.5).abs() < 0.0001);
        assert!((menu_interp_fade(0.25) - 0.103515625).abs() < 0.0001);
        assert!((menu_interp_fade(0.75) - 0.8964844).abs() < 0.0001);
    }

    #[test]
    fn menu_ui_plan_desktop_matches_upstream_main_and_submenu_geometry() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        });

        let play = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Play)
            .expect("desktop menu should include PLAY");
        let database = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Database)
            .expect("desktop menu should include DATABASE root");
        let schematics = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Schematics)
            .expect("database submenu should include SCHEMATICS");

        assert_eq!(play.rect, RenderRect::new(128.0, 500.0, 230.0, 70.0));
        assert_eq!(database.rect, RenderRect::new(128.0, 430.0, 230.0, 70.0));
        assert_eq!(schematics.rect, RenderRect::new(358.0, 430.0, 230.0, 70.0));
        assert!(database.selected);
        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .filter(|button| button.submenu)
                .count(),
            3
        );
    }

    #[test]
    fn menu_ui_plan_desktop_orders_root_buttons_top_to_bottom_like_java_table() {
        let mut state = MenuRendererState::new(
            MenuRendererConfig::new(false, 11).with_desktop_workshop_enabled(true),
        );

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        });

        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Play,
                MenuButtonRole::Database,
                MenuButtonRole::Editor,
                MenuButtonRole::Workshop,
                MenuButtonRole::Mods,
                MenuButtonRole::Settings,
                MenuButtonRole::Quit,
            ]
        );

        let y_positions = plan
            .ui
            .buttons
            .iter()
            .map(|button| button.rect.y)
            .collect::<Vec<_>>();
        assert!(y_positions.windows(2).all(|pair| pair[0] > pair[1]));
        assert!(plan.ui.buttons.iter().all(|button| button.rect.x == 128.0
            && button.rect.width == MENU_DESKTOP_BUTTON_WIDTH
            && button.rect.height == MENU_DESKTOP_BUTTON_HEIGHT));
    }

    #[test]
    fn menu_ui_plan_desktop_orders_database_submenu_down_from_the_root_anchor_like_java() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        });

        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Play,
                MenuButtonRole::Database,
                MenuButtonRole::Editor,
                MenuButtonRole::Mods,
                MenuButtonRole::Settings,
                MenuButtonRole::Quit,
                MenuButtonRole::Schematics,
                MenuButtonRole::ContentDatabase,
                MenuButtonRole::About,
            ]
        );

        let database = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Database)
            .expect("desktop menu should include DATABASE root");
        let schematics = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Schematics)
            .expect("database submenu should include SCHEMATICS");
        let content_database = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::ContentDatabase)
            .expect("database submenu should include CORE DATABASE");
        let about = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::About)
            .expect("database submenu should include ABOUT");

        assert_eq!(
            schematics.rect.x,
            database.rect.x + MENU_DESKTOP_BUTTON_WIDTH
        );
        assert_eq!(schematics.rect.y, database.rect.y);
        assert_eq!(
            content_database.rect.y,
            database.rect.y - MENU_DESKTOP_BUTTON_HEIGHT
        );
        assert_eq!(
            about.rect.y,
            database.rect.y - MENU_DESKTOP_BUTTON_HEIGHT * 2.0
        );
    }

    #[test]
    fn menu_ui_plan_desktop_submenu_anchor_respects_scene_margins_like_java() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(
            MenuFrameInput::new(1280.0, 720.0, 4.0, 1.0 / 60.0).with_scene_margins(10.0, 5.0),
        );

        let database = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Database)
            .expect("desktop menu should include DATABASE root");
        let schematics = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Schematics)
            .expect("database submenu should include SCHEMATICS");

        assert_eq!(database.rect.y, 430.0);
        assert_eq!(
            schematics.rect.y, 445.0,
            "Java submenu spacer subtracts Core.scene.marginTop and marginBottom from stage space, which moves the submenu row by the combined margins"
        );
    }

    #[test]
    fn menu_ui_plan_desktop_draws_black6_main_and_submenu_panels_like_java_tables() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        });
        let commands = plan.ui.to_render_commands();

        fn is_black6_panel_command(
            command: &RenderCommand,
            rect: RenderRect,
            alpha_scale: f32,
        ) -> bool {
            match command {
                RenderCommand::DrawSprite {
                    symbol,
                    rect: sprite_rect,
                    tint,
                    layer,
                    ..
                } => {
                    symbol == "whiteui"
                        && *sprite_rect == rect
                        && (tint[3] - UiDrawableTint::Black6.rgba()[3] * alpha_scale).abs() < 0.0001
                        && (*layer - MENU_DESKTOP_BACKGROUND_LAYER).abs() < f32::EPSILON
                }
                RenderCommand::FillRect {
                    rect: fill_rect,
                    color,
                    layer,
                } => {
                    *fill_rect == rect
                        && color[0] == 0.0
                        && color[1] == 0.0
                        && color[2] == 0.0
                        && (color[3] - UiDrawableTint::Black6.rgba()[3] * alpha_scale).abs()
                            < 0.0001
                        && (*layer - MENU_DESKTOP_BACKGROUND_LAYER).abs() < f32::EPSILON
                }
                _ => false,
            }
        }

        assert!(commands.iter().any(|command| {
            is_black6_panel_command(
                command,
                RenderRect::new(128.0, 0.0, MENU_DESKTOP_BUTTON_WIDTH, 720.0),
                1.0,
            )
        }));
        assert!(commands.iter().any(|command| {
            is_black6_panel_command(
                command,
                RenderRect::new(358.0, 0.0, MENU_DESKTOP_BUTTON_WIDTH, 720.0),
                plan.ui.submenu_alpha,
            )
        }));
    }

    #[test]
    fn menu_ui_plan_desktop_emits_black6_panels_before_button_draws_like_java_layering() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        });
        let commands = plan.ui.to_render_commands();

        fn is_black6_panel_command(
            command: &RenderCommand,
            rect: RenderRect,
            alpha_scale: f32,
        ) -> bool {
            match command {
                RenderCommand::DrawSprite {
                    symbol,
                    rect: sprite_rect,
                    tint,
                    layer,
                    ..
                } => {
                    symbol == "whiteui"
                        && *sprite_rect == rect
                        && (tint[3] - UiDrawableTint::Black6.rgba()[3] * alpha_scale).abs() < 0.0001
                        && (*layer - MENU_DESKTOP_BACKGROUND_LAYER).abs() < f32::EPSILON
                }
                RenderCommand::FillRect {
                    rect: fill_rect,
                    color,
                    layer,
                } => {
                    *fill_rect == rect
                        && color[0] == 0.0
                        && color[1] == 0.0
                        && color[2] == 0.0
                        && (color[3] - UiDrawableTint::Black6.rgba()[3] * alpha_scale).abs()
                            < 0.0001
                        && (*layer - MENU_DESKTOP_BACKGROUND_LAYER).abs() < f32::EPSILON
                }
                _ => false,
            }
        }

        assert!(commands.len() >= 2);
        assert!(is_black6_panel_command(
            &commands[0],
            RenderRect::new(128.0, 0.0, MENU_DESKTOP_BUTTON_WIDTH, 720.0),
            1.0,
        ));
        assert!(is_black6_panel_command(
            &commands[1],
            RenderRect::new(358.0, 0.0, MENU_DESKTOP_BUTTON_WIDTH, 720.0),
            plan.ui.submenu_alpha,
        ));
        assert!(commands.iter().skip(2).any(|command| matches!(
            command,
            RenderCommand::DrawText { text, .. } if text == "Play"
        )));
    }

    #[test]
    fn menu_ui_plan_desktop_up_buttons_leave_only_black6_panel_visible_like_java_clear_up() {
        let rect = RenderRect::new(10.0, 20.0, 230.0, 70.0);
        let plan = MenuUiPlan {
            mobile: false,
            submenu_alpha: 1.0,
            buttons: vec![MenuButtonPlan {
                role: MenuButtonRole::Mods,
                label: "Mods".to_string(),
                icon_name: None,
                rect,
                selected: false,
                hovered: false,
                pressed: false,
                submenu: false,
            }],
        };

        assert_eq!(
            plan.buttons[0].flat_toggle_menu_state(),
            MenuFlatToggleMenuState::Up
        );

        let commands = plan.to_render_commands();
        assert!(
            !commands.iter().any(|command| matches!(
                command,
                RenderCommand::DrawSprite {
                    symbol,
                    rect: sprite_rect,
                    ..
                } if symbol == "clear" && *sprite_rect == rect
            )),
            "Java Styles.flatToggleMenut uses clear for up state, so the Rust render plan should not emit a visible per-button up drawable"
        );
        assert!(
            !commands.iter().any(|command| matches!(
                command,
                RenderCommand::FillRect {
                    rect: fill_rect,
                    ..
                } if *fill_rect == rect
            )),
            "desktop up buttons should visually inherit the surrounding black6 table background instead of drawing a debug-style individual fill"
        );
        assert!(commands.iter().any(|command| matches!(
            command,
            RenderCommand::DrawSprite {
                symbol,
                rect: panel_rect,
                tint,
                layer,
                ..
            } if symbol == "whiteui"
                && *panel_rect == RenderRect::new(10.0, 0.0, 230.0, 110.0)
                && (tint[3] - UiDrawableTint::Black6.rgba()[3]).abs() < 0.0001
                && (*layer - MENU_DESKTOP_BACKGROUND_LAYER).abs() < f32::EPSILON
        )));
    }

    #[test]
    fn menu_ui_plan_mobile_matches_upstream_portrait_grid_geometry() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(true, 9));
        let input = MenuFrameInput {
            graphics_width: 720.0,
            graphics_height: 1280.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        };
        let plan = state.render_plan(input);

        assert!(plan.ui.mobile);
        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Campaign,
                MenuButtonRole::LoadGame,
                MenuButtonRole::CustomGame,
                MenuButtonRole::Join,
                MenuButtonRole::Editor,
                MenuButtonRole::Settings,
                MenuButtonRole::Mods,
                MenuButtonRole::Quit,
            ]
        );
        assert!(plan.ui.buttons.iter().all(|button| !button.submenu));
        assert!(plan.ui.buttons.iter().all(|button| !button.selected));
        assert_eq!(
            plan.ui.buttons[0].rect,
            RenderRect::new(235.0, 0.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[1].rect,
            RenderRect::new(365.0, 0.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[2].rect,
            RenderRect::new(235.0, 130.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[3].rect,
            RenderRect::new(365.0, 130.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[6].rect,
            RenderRect::new(235.0, 390.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[7].rect,
            RenderRect::new(365.0, 390.0, 120.0, 120.0)
        );

        let quit_center = plan.ui.buttons.last().unwrap().rect.center();
        assert_eq!(
            state.hit_test_ui(input, quit_center.x, quit_center.y),
            Some(MenuButtonRole::Quit)
        );
    }

    #[test]
    fn menu_ui_plan_mobile_matches_upstream_landscape_grid_geometry() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(true, 9));
        let input = MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        };
        let plan = state.render_plan(input);

        assert!(plan.ui.mobile);
        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Campaign,
                MenuButtonRole::Join,
                MenuButtonRole::CustomGame,
                MenuButtonRole::LoadGame,
                MenuButtonRole::Editor,
                MenuButtonRole::Settings,
                MenuButtonRole::Mods,
                MenuButtonRole::Quit,
            ]
        );
        assert!(plan.ui.buttons.iter().all(|button| !button.submenu));
        assert!(plan.ui.buttons.iter().all(|button| !button.selected));
        assert_eq!(
            plan.ui.buttons[0].rect,
            RenderRect::new(385.0, 60.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[1].rect,
            RenderRect::new(515.0, 60.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[2].rect,
            RenderRect::new(645.0, 60.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[3].rect,
            RenderRect::new(775.0, 60.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[4].rect,
            RenderRect::new(385.0, 190.0, 120.0, 120.0)
        );
        assert_eq!(
            plan.ui.buttons[7].rect,
            RenderRect::new(775.0, 190.0, 120.0, 120.0)
        );

        let join_center = plan.ui.buttons[1].rect.center();
        assert_eq!(
            state.hit_test_ui(input, join_center.x, join_center.y),
            Some(MenuButtonRole::Join)
        );
    }

    #[test]
    fn menu_ui_plan_mobile_includes_about_on_ios_and_exit_elsewhere_like_java() {
        let input = MenuFrameInput {
            graphics_width: 720.0,
            graphics_height: 1280.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        };

        let mut android_state = MenuRendererState::new(MenuRendererConfig::new(true, 9));
        let android_plan = android_state.render_plan(input);
        assert_eq!(
            android_plan.ui.buttons.last().map(|button| button.role),
            Some(MenuButtonRole::Quit)
        );

        let mut ios_state =
            MenuRendererState::new(MenuRendererConfig::new(true, 9).with_mobile_ios(true));
        let ios_plan = ios_state.render_plan(input);
        assert_eq!(
            ios_plan.ui.buttons.last().map(|button| button.role),
            Some(MenuButtonRole::About)
        );
        assert!(!ios_plan
            .ui
            .buttons
            .iter()
            .any(|button| button.role == MenuButtonRole::Quit));
    }

    #[test]
    fn menu_button_role_workshop_is_desktop_root_without_submenu() {
        assert_eq!(MenuButtonRole::Workshop.label(), "Workshop");
        assert!(MenuButtonRole::Workshop.is_desktop_root());
        assert!(!MenuButtonRole::Workshop.has_desktop_submenu());
        assert!(!MenuButtonRole::Workshop.is_submenu());
    }

    #[test]
    fn menu_flat_toggle_menu_style_keeps_upstream_state_names_and_java_clear_up_fallback() {
        let style = MENU_FLAT_TOGGLE_MENU_STYLE;

        assert_eq!(style.down_fill, [0.19, 0.38, 0.50, 0.92]);
        assert_eq!(
            style.up_fill,
            [0.0, 0.0, 0.0, 0.0],
            "Java Styles.flatToggleMenut.up is clear; fallback rendering should leave the black6 table panel visible"
        );
        assert_eq!(style.checked_fill, style.down_fill);
        assert_eq!(style.over_fill, UiDrawableTint::FlatOver.rgba());
        assert_eq!(style.disabled_fill, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(style.down_drawable, "flat-down-base.9");
        assert_eq!(style.up_drawable, "");
        assert_eq!(style.checked_drawable, "flat-down-base.9");
        assert_eq!(style.over_drawable, "");
        assert_eq!(style.disabled_drawable, "");
        assert_eq!(
            style.text_color,
            [1.0, 1.0, 1.0, 1.0],
            "Java Styles.flatToggleMenut uses Fonts.def with Color.white"
        );
        assert_eq!(style.disabled_text_color, [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(
            style.fill_for(MenuFlatToggleMenuState::Down),
            style.down_fill
        );
        assert_eq!(style.fill_for(MenuFlatToggleMenuState::Up), style.up_fill);
        assert_eq!(
            style.fill_for(MenuFlatToggleMenuState::Checked),
            style.checked_fill
        );
        assert_eq!(
            style.fill_for(MenuFlatToggleMenuState::Over),
            style.over_fill
        );
        assert_eq!(
            style.fill_for(MenuFlatToggleMenuState::Disabled),
            style.disabled_fill
        );
        assert_eq!(
            style.drawable_for(MenuFlatToggleMenuState::Down),
            "flat-down-base.9"
        );
        assert_eq!(style.drawable_for(MenuFlatToggleMenuState::Up), "");
        assert_eq!(
            style.drawable_for(MenuFlatToggleMenuState::Checked),
            "flat-down-base.9"
        );
        assert_eq!(style.drawable_for(MenuFlatToggleMenuState::Over), "");
        assert_eq!(style.drawable_for(MenuFlatToggleMenuState::Disabled), "");
        assert_eq!(style.text_size(false), 18.0);
        assert_eq!(style.text_size(true), 16.0);
        assert_eq!(
            style.text_style,
            RenderTextStyle::new(RenderTextAlign::Center)
                .with_vertical_align(RenderTextVerticalAlign::Center)
                .with_markup(true)
                .with_integer_position(true)
        );
    }

    #[test]
    fn menu_flat_toggle_menu_resolves_java_style_drawables_from_ui_registry() {
        assert_eq!(
            menu_flat_toggle_menu_java_drawable_for_state(MenuFlatToggleMenuState::Up),
            Some("clear")
        );
        assert_eq!(
            menu_flat_toggle_menu_java_drawable_for_state(MenuFlatToggleMenuState::Down),
            Some("flatDown")
        );
        assert_eq!(
            menu_flat_toggle_menu_java_drawable_for_state(MenuFlatToggleMenuState::Checked),
            Some("flatDown")
        );
        assert_eq!(
            menu_flat_toggle_menu_java_drawable_for_state(MenuFlatToggleMenuState::Over),
            Some("flatOver")
        );
        assert_eq!(
            menu_flat_toggle_menu_java_drawable_for_state(MenuFlatToggleMenuState::Disabled),
            Some("black")
        );

        let checked =
            menu_flat_toggle_menu_drawable_for_state(MenuFlatToggleMenuState::Checked).unwrap();
        assert_eq!(checked.atlas_symbol, "flat-down-base.9");
        assert_eq!(checked.tint, UiDrawableTint::None);

        let over = menu_flat_toggle_menu_drawable_for_state(MenuFlatToggleMenuState::Over).unwrap();
        assert_eq!(over.atlas_symbol, "whiteui");
        assert_eq!(over.tint, UiDrawableTint::FlatOver);

        let up = menu_flat_toggle_menu_drawable_for_state(MenuFlatToggleMenuState::Up).unwrap();
        assert_eq!(up.atlas_symbol, "clear");
    }

    #[test]
    fn menu_ui_plan_selected_buttons_emit_java_flat_down_drawable() {
        let rect = RenderRect::new(10.0, 20.0, 230.0, 70.0);
        let plan = MenuUiPlan {
            mobile: false,
            submenu_alpha: 1.0,
            buttons: vec![MenuButtonPlan {
                role: MenuButtonRole::Play,
                label: "Play".to_string(),
                icon_name: None,
                rect,
                selected: true,
                hovered: false,
                pressed: false,
                submenu: false,
            }],
        };

        let commands = plan.to_render_commands();
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawSprite {
                    symbol,
                    rect: sprite_rect,
                    layer,
                    ..
                } if symbol == "flat-down-base.9"
                    && *sprite_rect == rect
                    && (*layer - MENU_FLAT_TOGGLE_MENU_STYLE.drawable_layer).abs() < f32::EPSILON
            )
        }));
        assert!(
            !commands.iter().any(|command| {
                matches!(
                    command,
                    RenderCommand::StrokeRect {
                        rect: outline_rect,
                        layer,
                        ..
                    } if *outline_rect == rect
                        && (*layer
                            - (MENU_FLAT_TOGGLE_MENU_STYLE.drawable_layer + 0.01))
                            .abs()
                            < f32::EPSILON
                )
            }),
            "Java Styles.flatToggleMenut relies on flatDown/flatOver drawables and does not add an extra desktop-only focus outline"
        );
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, .. } if text == "Play"
            )
        }));
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, position, style, layer, .. }
                    if *text == upstream_ui_icon_glyph_string("play").unwrap()
                        && style.font == RenderFontId::Icon
                        && !style.outline
                        && (position.x
                            - (rect.x
                                + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                                + MENU_DESKTOP_BUTTON_ICON_X))
                            .abs()
                            < f32::EPSILON
                        && (position.y - rect.center().y).abs() < f32::EPSILON
                        && (*layer
                            - (MENU_FLAT_TOGGLE_MENU_STYLE.text_layer
                                + MENU_BUTTON_ICON_LAYER_OFFSET))
                            .abs()
                            < f32::EPSILON
            )
        }));
        assert!(!commands.iter().any(|command| matches!(
            command,
            RenderCommand::DrawText { text, position, .. }
                if text == "?"
                    && (position.x
                        - (rect.x
                            + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                            + MENU_DESKTOP_BUTTON_ICON_X))
                        .abs()
                        < f32::EPSILON
        )));
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, position, style, .. }
                    if text == "Play"
                        && (position.x
                            - (rect.x
                                + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                                + MENU_DESKTOP_BUTTON_ICON_X
                                + MENU_DESKTOP_BUTTON_LABEL_GAP))
                            .abs()
                            < f32::EPSILON
                        && style.horizontal_align == RenderTextAlign::Start
                        && style.markup
                        && !style.outline
            )
        }));
    }

    #[test]
    fn menu_ui_plan_hovered_buttons_emit_java_flat_over_drawable() {
        let rect = RenderRect::new(10.0, 20.0, 230.0, 70.0);
        let plan = MenuUiPlan {
            mobile: false,
            submenu_alpha: 1.0,
            buttons: vec![MenuButtonPlan {
                role: MenuButtonRole::Mods,
                label: "Mods".to_string(),
                icon_name: None,
                rect,
                selected: false,
                hovered: false,
                pressed: false,
                submenu: false,
            }],
        }
        .with_hovered_role(Some(MenuButtonRole::Mods));

        assert_eq!(
            plan.buttons[0].flat_toggle_menu_state(),
            MenuFlatToggleMenuState::Over
        );
        let commands = plan.to_render_commands();
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawSprite {
                    symbol,
                    rect: sprite_rect,
                    tint,
                    ..
                } if symbol == "whiteui"
                    && *sprite_rect == rect
                    && (tint[0] - UiDrawableTint::FlatOver.rgba()[0]).abs() < 0.0001
                    && (tint[1] - UiDrawableTint::FlatOver.rgba()[1]).abs() < 0.0001
                    && (tint[2] - UiDrawableTint::FlatOver.rgba()[2]).abs() < 0.0001
                    && (tint[3] - UiDrawableTint::FlatOver.rgba()[3]).abs() < 0.0001
            )
        }));
    }

    #[test]
    fn menu_ui_plan_pressed_buttons_emit_java_flat_down_drawable() {
        let rect = RenderRect::new(10.0, 20.0, 230.0, 70.0);
        let plan = MenuUiPlan {
            mobile: false,
            submenu_alpha: 1.0,
            buttons: vec![MenuButtonPlan {
                role: MenuButtonRole::Settings,
                label: "Settings".to_string(),
                icon_name: None,
                rect,
                selected: false,
                hovered: true,
                pressed: false,
                submenu: false,
            }],
        }
        .with_pressed_role(Some(MenuButtonRole::Settings));

        assert_eq!(
            plan.buttons[0].flat_toggle_menu_state(),
            MenuFlatToggleMenuState::Down
        );
        let commands = plan.to_render_commands();
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawSprite {
                    symbol,
                    rect: sprite_rect,
                    ..
                } if symbol == "flat-down-base.9" && *sprite_rect == rect
            )
        }));
    }

    #[test]
    fn menu_button_roles_expose_upstream_menu_fragment_icons() {
        assert_eq!(MenuButtonRole::Play.icon_name(false), Some("play"));
        assert_eq!(MenuButtonRole::Database.icon_name(false), Some("menu"));
        assert_eq!(MenuButtonRole::Join.icon_name(false), Some("add"));
        assert_eq!(MenuButtonRole::CustomGame.icon_name(false), Some("terrain"));
        assert_eq!(
            MenuButtonRole::CustomGame.icon_name(true),
            Some("rightOpenOut")
        );
        assert_eq!(MenuButtonRole::LoadGame.icon_name(true), Some("download"));
        assert_eq!(MenuButtonRole::Schematics.icon_name(false), Some("paste"));
        assert_eq!(
            MenuButtonRole::ContentDatabase.icon_name(false),
            Some("book")
        );
        assert_eq!(MenuButtonRole::About.icon_name(false), Some("info"));
        assert_eq!(MenuButtonRole::Workshop.icon_name(false), Some("steam"));
        assert_eq!(MenuButtonRole::Quit.icon_name(true), Some("exit"));
        assert_eq!(MenuButtonRole::Custom(0).icon_name(false), None);
        assert_eq!(MenuButtonRole::Play.bundle_key(), Some("play"));
        assert_eq!(MenuButtonRole::Join.bundle_key(), Some("joingame"));
        assert_eq!(
            MenuButtonRole::Database.bundle_key(),
            Some("database.button")
        );
        assert_eq!(
            MenuButtonRole::ContentDatabase.bundle_key(),
            Some("database")
        );
        assert_eq!(MenuButtonRole::About.bundle_key(), Some("about.button"));
        assert_eq!(MenuButtonRole::Custom(0).bundle_key(), None);
        assert_eq!(MenuButtonRole::Join.label(), "Join Game");
        assert_eq!(MenuButtonRole::CustomGame.label(), "Custom Game");
        assert_eq!(MenuButtonRole::ContentDatabase.label(), "Core Database");
        assert_eq!(MenuButtonRole::Settings.label(), "Settings");

        for icon in [
            "play",
            "menu",
            "add",
            "terrain",
            "rightOpenOut",
            "download",
            "paste",
            "book",
            "info",
            "steam",
            "settings",
            "exit",
        ] {
            assert!(
                upstream_ui_icon_glyph_string(icon).is_some(),
                "upstream Icon.{icon} glyph should be registered"
            );
        }
    }

    #[test]
    fn menu_icon_render_commands_cover_upstream_menu_fragment_icons_without_question_text() {
        for icon in [
            "play",
            "menu",
            "add",
            "terrain",
            "rightOpenOut",
            "download",
            "paste",
            "book",
            "tree",
            "info",
            "steam",
            "settings",
            "exit",
            "host",
        ] {
            let mut commands = Vec::new();
            menu_push_icon_render_commands(
                &mut commands,
                icon,
                RenderPoint::new(32.0, 48.0),
                MENU_DESKTOP_BUTTON_ICON_TEXT_SIZE,
                [1.0, 1.0, 1.0, 1.0],
                MENU_FLAT_TOGGLE_MENU_STYLE.text_layer,
            );
            assert!(
                !commands.is_empty(),
                "icon {icon} should emit visible commands"
            );
            assert!(
                !commands.iter().any(|command| matches!(
                    command,
                    RenderCommand::DrawText { text, .. } if text == "?"
                )),
                "icon {icon} must not regress to the placeholder question mark"
            );
            assert!(
                commands.iter().any(|command| matches!(
                    command,
                    RenderCommand::DrawText { style, .. } if style.font == RenderFontId::Icon
                )),
                "icon {icon} should be emitted through the upstream Icon font identity"
            );
        }
    }

    #[test]
    fn menu_mobile_buttons_render_icon_above_label_like_mobile_button() {
        let rect = RenderRect::new(30.0, 40.0, 120.0, 120.0);
        let plan = MenuUiPlan {
            mobile: true,
            submenu_alpha: 0.0,
            buttons: vec![MenuButtonPlan {
                role: MenuButtonRole::CustomGame,
                label: "Custom Game".to_string(),
                icon_name: None,
                rect,
                selected: false,
                hovered: false,
                pressed: false,
                submenu: false,
            }],
        };

        let commands = plan.to_render_commands();
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawSprite {
                    symbol,
                    rect: sprite_rect,
                    ..
                } if symbol == "button.9" && *sprite_rect == rect
            )
        }));
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, position, style, layer, .. }
                    if *text == upstream_ui_icon_glyph_string("rightOpenOut").unwrap()
                        && style.font == RenderFontId::Icon
                        && (position.x - rect.center().x).abs() < f32::EPSILON
                        && (position.y - (rect.center().y + MENU_MOBILE_BUTTON_ICON_OFFSET_Y)).abs() < f32::EPSILON
                        && (*layer
                            - (MENU_FLAT_TOGGLE_MENU_STYLE.text_layer
                                + MENU_BUTTON_ICON_LAYER_OFFSET))
                            .abs()
                            < f32::EPSILON
            )
        }));
        assert!(!commands.iter().any(|command| matches!(
            command,
            RenderCommand::DrawText { text, position, .. }
                if text == "?"
                    && (position.x - rect.center().x).abs() < f32::EPSILON
                    && (position.y - (rect.center().y + MENU_MOBILE_BUTTON_ICON_OFFSET_Y)).abs()
                        < f32::EPSILON
        )));
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, position, style, layer, .. }
                    if text == "Custom Game"
                        && (position.y - (rect.center().y + MENU_MOBILE_BUTTON_LABEL_OFFSET_Y)).abs() < f32::EPSILON
                        && style.horizontal_align == RenderTextAlign::Center
                        && style.markup
                        && (*layer
                            - (MENU_FLAT_TOGGLE_MENU_STYLE.text_layer
                                + MENU_BUTTON_LABEL_LAYER_OFFSET))
                            .abs()
                            < f32::EPSILON
            )
        }));
    }

    #[test]
    fn menu_ui_plan_desktop_inserts_workshop_before_mods_when_enabled_like_java() {
        let mut state = MenuRendererState::new(
            MenuRendererConfig::new(false, 11).with_desktop_workshop_enabled(true),
        );
        assert!(state.select_desktop_root(MenuButtonRole::Play));
        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        });

        let roles = plan
            .ui
            .buttons
            .iter()
            .map(|button| button.role)
            .collect::<Vec<_>>();
        assert_eq!(
            roles,
            vec![
                MenuButtonRole::Play,
                MenuButtonRole::Database,
                MenuButtonRole::Editor,
                MenuButtonRole::Workshop,
                MenuButtonRole::Mods,
                MenuButtonRole::Settings,
                MenuButtonRole::Quit,
                MenuButtonRole::Campaign,
                MenuButtonRole::Join,
                MenuButtonRole::CustomGame,
                MenuButtonRole::LoadGame,
            ]
        );

        let workshop = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Workshop)
            .expect("steam/workshop desktop menu should expose workshop button");
        let mods = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Mods)
            .expect("desktop menu should still expose mods after workshop");
        assert_eq!(workshop.label, "Workshop");
        assert!(workshop.rect.y > mods.rect.y);
    }

    #[test]
    fn menu_ui_plan_desktop_inserts_custom_buttons_before_quit_like_java() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert_eq!(
            state.add_custom_button_with(
                MenuCustomButton::new("SERVER BROWSER")
                    .with_icon_name("add")
                    .with_action_id("server-browser")
            ),
            MenuButtonRole::Custom(0)
        );
        assert!(state.select_desktop_root(MenuButtonRole::Play));
        let input = MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        };
        let plan = state.render_plan(input);

        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Play,
                MenuButtonRole::Database,
                MenuButtonRole::Editor,
                MenuButtonRole::Mods,
                MenuButtonRole::Settings,
                MenuButtonRole::Custom(0),
                MenuButtonRole::Quit,
                MenuButtonRole::Campaign,
                MenuButtonRole::Join,
                MenuButtonRole::CustomGame,
                MenuButtonRole::LoadGame,
            ]
        );

        let custom = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Custom(0))
            .expect("desktop menu should expose injected custom button");
        let quit = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Quit)
            .expect("desktop menu should still expose quit after custom buttons");
        assert_eq!(custom.label, "SERVER BROWSER");
        assert_eq!(custom.icon_name.as_deref(), Some("add"));
        assert!(custom.rect.y > quit.rect.y);
        assert_eq!(
            state.custom_button_action_id(MenuButtonRole::Custom(0)),
            Some("server-browser")
        );
        assert_eq!(
            state.hit_test_ui(input, custom.rect.center().x, custom.rect.center().y),
            Some(MenuButtonRole::Custom(0))
        );
        let render_commands = plan.ui.to_render_commands();
        let custom_label_layer = render_commands
            .iter()
            .find_map(|command| match command {
                RenderCommand::DrawText { text, layer, .. } if text == "SERVER BROWSER" => {
                    Some(*layer)
                }
                _ => None,
            })
            .expect("desktop custom label should be rendered");
        assert!(render_commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, position, style, layer, .. }
                    if *text == upstream_ui_icon_glyph_string("add").unwrap()
                        && style.font == RenderFontId::Icon
                        && (position.x
                            - (custom.rect.x
                                + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                            + MENU_DESKTOP_BUTTON_ICON_X))
                            .abs()
                            < f32::EPSILON
                        && (position.y - custom.rect.center().y).abs() < f32::EPSILON
                        && (*layer
                            - (MENU_FLAT_TOGGLE_MENU_STYLE.text_layer
                                + MENU_BUTTON_ICON_LAYER_OFFSET))
                            .abs()
                            < f32::EPSILON
            )
        }));
        assert!(!render_commands.iter().any(|command| matches!(
            command,
            RenderCommand::DrawText { text, position, .. }
                if text == "?"
                    && (position.x
                        - (custom.rect.x
                            + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                            + MENU_DESKTOP_BUTTON_ICON_X))
                        .abs()
                        < f32::EPSILON
        )));
        assert!(render_commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, position, style, layer, .. }
                    if text == "SERVER BROWSER"
                        && (position.x
                            - (custom.rect.x
                                + MENU_DESKTOP_BUTTON_MARGIN_LEFT
                                + MENU_DESKTOP_BUTTON_ICON_X
                                + MENU_DESKTOP_BUTTON_LABEL_GAP))
                            .abs()
                            < f32::EPSILON
                        && style.horizontal_align == RenderTextAlign::Start
                        && (*layer
                            - (MENU_FLAT_TOGGLE_MENU_STYLE.text_layer
                                + MENU_BUTTON_LABEL_LAYER_OFFSET))
                            .abs()
                            < f32::EPSILON
            )
        }));
        assert!(
            custom_label_layer
                > MENU_FLAT_TOGGLE_MENU_STYLE.text_layer + MENU_BUTTON_ICON_LAYER_OFFSET
        );
    }

    #[test]
    fn menu_ui_plan_desktop_text_labels_keep_markup_enabled_for_literal_color_tags() {
        let rect = RenderRect::new(10.0, 20.0, 230.0, 70.0);
        let plan = MenuUiPlan {
            mobile: false,
            submenu_alpha: 1.0,
            buttons: vec![MenuButtonPlan {
                role: MenuButtonRole::Custom(0),
                label: "[accent]SERVER[] BROWSER".to_string(),
                icon_name: None,
                rect,
                selected: false,
                hovered: false,
                pressed: false,
                submenu: false,
            }],
        };

        let commands = plan.to_render_commands();
        assert!(commands.iter().any(|command| {
            matches!(
                command,
                RenderCommand::DrawText { text, style, .. }
                    if text == "[accent]SERVER[] BROWSER" && style.markup
            )
        }));
    }

    #[test]
    fn menu_ui_plan_desktop_expands_custom_submenu_buttons_like_menu_fragment_submenu() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        let root = state.add_custom_button_with(
            MenuCustomButton::new("TOOLS")
                .with_icon_name("settings")
                .with_submenu_buttons(vec![
                    MenuCustomButton::new("LOCAL SERVER")
                        .with_icon_name("host")
                        .with_action_id("local-server"),
                    MenuCustomButton::new("REPLAY VIEWER")
                        .with_icon_name("play")
                        .with_action_id("replay-viewer"),
                ]),
        );
        assert_eq!(root, MenuButtonRole::Custom(0));
        assert!(state.role_has_desktop_submenu(root));
        assert!(state.select_desktop_root(root));

        let input = MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        };
        let plan = state.render_plan(input);
        let roles = plan
            .ui
            .buttons
            .iter()
            .map(|button| button.role)
            .collect::<Vec<_>>();

        assert!(roles.contains(&MenuButtonRole::Custom(0)));
        assert!(roles.contains(&MenuButtonRole::CustomSubmenu { root: 0, item: 0 }));
        assert!(roles.contains(&MenuButtonRole::CustomSubmenu { root: 0, item: 1 }));

        let root_button = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::Custom(0))
            .expect("custom submenu root should stay visible");
        assert!(
            root_button.selected,
            "Java flatToggleMenut root stays checked while its submenu is currentMenu"
        );

        let submenu = plan
            .ui
            .buttons
            .iter()
            .find(|button| button.role == MenuButtonRole::CustomSubmenu { root: 0, item: 0 })
            .expect("custom root with submenu buttons should render its first submenu item");
        assert_eq!(submenu.label, "LOCAL SERVER");
        assert_eq!(submenu.icon_name.as_deref(), Some("host"));
        assert!(!submenu.submenu);
        assert_eq!(
            state.custom_button_action_id(MenuButtonRole::CustomSubmenu { root: 0, item: 0 }),
            Some("local-server")
        );
        assert_eq!(
            state.hit_test_ui(input, submenu.rect.center().x, submenu.rect.center().y),
            Some(MenuButtonRole::CustomSubmenu { root: 0, item: 0 })
        );
    }

    #[test]
    fn menu_ui_plan_mobile_inserts_custom_buttons_in_java_odd_even_order() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(true, 9));
        state.add_custom_button("FIRST CUSTOM");
        state.add_custom_button("SECOND CUSTOM");
        let input = MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
            scene_margin_top: 0.0,
            scene_margin_bottom: 0.0,
            scl4: 4.0,
            delta: 1.0 / 60.0,
        };
        let plan = state.render_plan(input);

        assert_eq!(
            plan.ui
                .buttons
                .iter()
                .map(|button| button.role)
                .collect::<Vec<_>>(),
            vec![
                MenuButtonRole::Campaign,
                MenuButtonRole::Join,
                MenuButtonRole::CustomGame,
                MenuButtonRole::LoadGame,
                MenuButtonRole::Custom(1),
                MenuButtonRole::Editor,
                MenuButtonRole::Settings,
                MenuButtonRole::Mods,
                MenuButtonRole::Custom(0),
                MenuButtonRole::Quit,
            ]
        );
        assert_eq!(plan.ui.buttons[4].label, "SECOND CUSTOM");
        assert_eq!(plan.ui.buttons[8].label, "FIRST CUSTOM");
        assert_eq!(
            state.hit_test_ui(
                input,
                plan.ui.buttons[8].rect.center().x,
                plan.ui.buttons[8].rect.center().y
            ),
            Some(MenuButtonRole::Custom(0))
        );
    }
}
