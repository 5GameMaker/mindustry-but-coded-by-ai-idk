/// Data-oriented migration of upstream `MenuRenderer`.
///
/// The Java renderer generates a temporary menu world, draws floor/overlay and
/// wall caches, then renders flyers and a dark fullscreen overlay.  This module
/// keeps generation and render planning in `mindustry-core`; concrete backends
/// are expected to translate `MenuRenderCommand` into GPU/cache calls.
use super::{
    RenderCamera, RenderCommand, RenderPass, RenderPassKind, RenderPoint, RenderRect,
    RenderTextAlign, RenderTextStyle, RenderTextVerticalAlign, RenderViewport,
};

pub const MENU_DARKNESS: f32 = 0.3;
pub const MENU_TILE_SIZE: f32 = 8.0;

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
            Self::CopperOre => Some("copper-ore"),
            Self::LeadOre => Some("lead-ore"),
            Self::ScrapOre => Some("scrap-ore"),
            Self::CoalOre => Some("coal-ore"),
            Self::TitaniumOre => Some("titanium-ore"),
            Self::ThoriumOre => Some("thorium-ore"),
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
}

impl MenuRendererConfig {
    pub const fn new(mobile: bool, seed: u64) -> Self {
        Self {
            mobile,
            seed,
            tile_size: MENU_TILE_SIZE,
            desktop_workshop_enabled: false,
        }
    }

    pub const fn with_desktop_workshop_enabled(mut self, enabled: bool) -> Self {
        self.desktop_workshop_enabled = enabled;
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
    pub scl4: f32,
    pub delta: f32,
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
    Quit,
}

impl MenuButtonRole {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Play => "PLAY",
            Self::Campaign => "CAMPAIGN",
            Self::Join => "JOIN",
            Self::CustomGame => "CUSTOM GAME",
            Self::LoadGame => "LOAD GAME",
            Self::Database => "DATABASE",
            Self::Schematics => "SCHEMATICS",
            Self::ContentDatabase => "DATABASE",
            Self::TechTree => "TECH TREE",
            Self::About => "ABOUT",
            Self::Editor => "EDITOR",
            Self::Workshop => "WORKSHOP",
            Self::Mods => "MODS",
            Self::Settings => "SETTINGS",
            Self::Custom(_) => "CUSTOM",
            Self::Quit => "QUIT",
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuCustomButton {
    pub label: String,
}

impl MenuCustomButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuButtonPlan {
    pub role: MenuButtonRole,
    pub label: String,
    pub rect: RenderRect,
    pub selected: bool,
    pub submenu: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuUiPlan {
    pub mobile: bool,
    pub buttons: Vec<MenuButtonPlan>,
}

impl MenuUiPlan {
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
        let mut commands = Vec::with_capacity(self.buttons.len() * 2);
        for button in &self.buttons {
            let fill = if button.selected {
                [0.19, 0.38, 0.50, 0.92]
            } else if button.submenu {
                [0.10, 0.16, 0.22, 0.86]
            } else {
                [0.07, 0.11, 0.16, 0.88]
            };
            commands.push(RenderCommand::fill_rect(button.rect, fill, 101.0));
            commands.push(RenderCommand::draw_text_styled(
                button.label.as_str(),
                button.rect.center(),
                [0.88, 0.96, 1.0, 1.0],
                if self.mobile { 7.0 } else { 8.0 },
                0.0,
                RenderTextStyle::new(RenderTextAlign::Center)
                    .with_vertical_align(RenderTextVerticalAlign::Center)
                    .with_integer_position(true)
                    .with_outline(true),
                101.1,
            ));
        }
        commands
    }
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
                let mut commands = Vec::with_capacity(world.tiles.len() * 2);
                match label {
                    "floor+overlay" => {
                        for tile in &world.tiles {
                            let rect =
                                menu_transform_rect(menu_tile_rect(tile, tile_size), transform);
                            if let Some(color) = tile.floor.menu_color() {
                                commands.push(RenderCommand::fill_rect(rect, color, -0.2));
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
                        }
                    }
                    "wall" => {
                        for tile in &world.tiles {
                            if let Some(color) = tile.wall.menu_color() {
                                commands.push(RenderCommand::fill_rect(
                                    menu_transform_rect(menu_tile_rect(tile, tile_size), transform),
                                    color,
                                    1.0,
                                ));
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
                let mut commands = Vec::with_capacity(world.tiles.len());
                for tile in &world.tiles {
                    if tile.wall != MenuBlockKind::Air {
                        commands.push(RenderCommand::fill_rect(
                            menu_transform_rect(menu_tile_rect(tile, tile_size), transform),
                            [0.0, 0.0, 0.0, 0.35],
                            0.5,
                        ));
                    }
                }
                if commands.is_empty() {
                    let shadow_rect = if height.is_sign_negative() {
                        RenderRect::new(x, y + height, width, -height)
                    } else {
                        RenderRect::new(x, y, width, height)
                    };
                    commands.push(RenderCommand::fill_rect(
                        menu_transform_rect(shadow_rect, transform),
                        [0.0, 0.0, 0.0, 0.35],
                        0.5,
                    ));
                }
                commands
            }
            Self::DrawFlyer(flyer) => {
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

                vec![
                    RenderCommand::draw_sprite(
                        "circle-shadow",
                        shadow_rect,
                        [1.0, 1.0, 1.0, 1.0],
                        0.0,
                        1.5,
                    ),
                    RenderCommand::draw_sprite(
                        flyer.unit_name,
                        body_rect,
                        [1.0, 1.0, 1.0, 1.0],
                        flyer.rotation,
                        2.0,
                    ),
                ]
            }
            Self::DrawDarkness {
                alpha: _,
                width: _,
                height: _,
            } => Vec::new(),
        }
    }
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
        if self.commands.is_empty() {
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
            pass.extend(command.to_render_commands_with_transform(
                &self.world,
                self.tile_size,
                transform,
            ));
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

        if commands.is_empty() {
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
            pass.extend(command.into_render_commands_with_transform(
                &world,
                tile_size,
                Some(transform),
            ));
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
            custom_buttons: Vec::new(),
        }
    }

    pub fn render_plan(&mut self, input: MenuFrameInput) -> MenuFramePlan {
        self.time += input.delta;

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
                self.config.desktop_workshop_enabled,
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
            self.config.desktop_workshop_enabled,
            &self.custom_buttons,
        )
    }

    pub fn hit_test_ui(&self, input: MenuFrameInput, x: f32, y: f32) -> Option<MenuButtonRole> {
        self.ui_plan(input).hit_test(x, y)
    }

    pub fn select_desktop_root(&mut self, role: MenuButtonRole) -> bool {
        if self.config.mobile || !role.has_desktop_submenu() {
            return false;
        }
        self.selected_root = role;
        true
    }

    pub fn add_custom_button(&mut self, label: impl Into<String>) -> MenuButtonRole {
        let index = self.custom_buttons.len().min(u16::MAX as usize) as u16;
        self.custom_buttons.push(MenuCustomButton::new(label));
        MenuButtonRole::Custom(index)
    }

    pub fn clear_custom_buttons(&mut self) {
        self.custom_buttons.clear();
    }
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
        rect,
        selected,
        submenu: role.is_submenu(),
    }
}

fn menu_custom_button_plan(
    index: usize,
    button: &MenuCustomButton,
    rect: RenderRect,
) -> MenuButtonPlan {
    MenuButtonPlan {
        role: MenuButtonRole::Custom(index.min(u16::MAX as usize) as u16),
        label: button.label.clone(),
        rect,
        selected: false,
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
        rect,
        selected,
        submenu: false,
    }
}

fn menu_desktop_ui_plan(
    input: MenuFrameInput,
    selected_root: MenuButtonRole,
    desktop_workshop_enabled: bool,
    custom_buttons: &[MenuCustomButton],
) -> MenuUiPlan {
    let button_width = 230.0;
    let button_height = 70.0;
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
    let submenu_roles: &[MenuButtonRole] = match selected_root {
        MenuButtonRole::Play => &play_submenu_roles,
        MenuButtonRole::Database => &database_submenu_roles,
        _ => &[],
    };
    let main_role_count = main_roles.len();
    let main_button_count = main_role_count + custom_buttons.len() + 1;
    let total_height =
        main_button_count as f32 * button_height + main_button_count.saturating_sub(1) as f32 * gap;
    let start_y = ((input.graphics_height - total_height) * 0.5).max(0.0);
    let left_x = (input.graphics_width / 10.0).max(0.0);
    let submenu_x = left_x + button_width;
    let selected_root_index = main_roles
        .iter()
        .position(|role| *role == selected_root)
        .unwrap_or(0);
    let submenu_start_y = start_y + selected_root_index as f32 * (button_height + gap);

    let mut buttons = Vec::with_capacity(main_button_count + submenu_roles.len());
    for (index, role) in main_roles.iter().copied().enumerate() {
        buttons.push(menu_button_plan(
            role,
            RenderRect::new(
                left_x,
                start_y + index as f32 * (button_height + gap),
                button_width,
                button_height,
            ),
            role == selected_root,
        ));
    }
    for (custom_index, custom) in custom_buttons.iter().enumerate() {
        let index = main_role_count + custom_index;
        buttons.push(menu_custom_button_plan(
            custom_index,
            custom,
            RenderRect::new(
                left_x,
                start_y + index as f32 * (button_height + gap),
                button_width,
                button_height,
            ),
        ));
    }
    let quit_index = main_role_count + custom_buttons.len();
    buttons.push(menu_button_plan(
        MenuButtonRole::Quit,
        RenderRect::new(
            left_x,
            start_y + quit_index as f32 * (button_height + gap),
            button_width,
            button_height,
        ),
        MenuButtonRole::Quit == selected_root,
    ));
    for (index, role) in submenu_roles.iter().copied().enumerate() {
        buttons.push(menu_button_plan(
            role,
            RenderRect::new(
                submenu_x,
                submenu_start_y + index as f32 * (button_height + gap),
                button_width,
                button_height,
            ),
            false,
        ));
    }

    MenuUiPlan {
        mobile: false,
        buttons,
    }
}

fn menu_mobile_button_entry(role: MenuButtonRole) -> (MenuButtonRole, String, bool) {
    (
        role,
        role.label().to_string(),
        role == MenuButtonRole::Campaign,
    )
}

fn menu_mobile_custom_entry(
    index: usize,
    button: &MenuCustomButton,
) -> (MenuButtonRole, String, bool) {
    (
        MenuButtonRole::Custom(index.min(u16::MAX as usize) as u16),
        button.label.clone(),
        false,
    )
}

fn menu_mobile_ui_plan(input: MenuFrameInput, custom_buttons: &[MenuCustomButton]) -> MenuUiPlan {
    let rows: Vec<Vec<(MenuButtonRole, String, bool)>> =
        if input.graphics_width > input.graphics_height {
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
            second.push(menu_mobile_button_entry(MenuButtonRole::Quit));
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
            current.push(menu_mobile_button_entry(MenuButtonRole::Quit));
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
        for (column_index, (role, label, selected)) in row.iter().enumerate() {
            let mut button = menu_mobile_button_plan(
                *role,
                RenderRect::new(
                    start_x + column_index as f32 * (button_size + gap),
                    start_y + row_index as f32 * (button_size + gap),
                    button_size,
                    button_size,
                ),
                *selected,
            );
            button.label = label.clone();
            buttons.push(button);
        }
    }

    MenuUiPlan {
        mobile: true,
        buttons,
    }
}

fn menu_ui_plan(
    input: MenuFrameInput,
    mobile: bool,
    selected_root: MenuButtonRole,
    desktop_workshop_enabled: bool,
    custom_buttons: &[MenuCustomButton],
) -> MenuUiPlan {
    if mobile {
        menu_mobile_ui_plan(input, custom_buttons)
    } else {
        menu_desktop_ui_plan(
            input,
            selected_root,
            desktop_workshop_enabled,
            custom_buttons,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn render_plan_keeps_java_cache_shadow_wall_flyers_darkness_order() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 7));
        state.flyer_count = 2;

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1920.0,
            graphics_height: 1080.0,
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
            MenuRenderCommand::DrawShadowTexture { .. }
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

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1920.0,
            graphics_height: 1080.0,
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
            |command| matches!(command, RenderCommand::DrawText { text, .. } if text == "PLAY")
        ));
        assert!(borrowed.commands.iter().any(
            |command| matches!(command, RenderCommand::DrawText { text, .. } if text == "CAMPAIGN")
        ));

        assert!(
            borrowed.commands.iter().all(|command| {
                !matches!(
                    command,
                    RenderCommand::FillRect { color, layer, .. }
                        if *color == [0.0, 0.0, 0.0, MENU_DARKNESS]
                            && (*layer - 100.0).abs() < f32::EPSILON
                )
            }),
            "menu darkness stays in the logical Java-order plan, but the current native-safe fallback pre-darkens terrain instead of drawing an alpha fullscreen quad"
        );
    }

    #[test]
    fn menu_renderer_state_switches_desktop_submenu_roots() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
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
    fn menu_ui_plan_desktop_matches_upstream_main_and_submenu_geometry() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert!(state.select_desktop_root(MenuButtonRole::Database));

        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
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

        assert_eq!(play.rect, RenderRect::new(128.0, 150.0, 230.0, 70.0));
        assert_eq!(database.rect, RenderRect::new(128.0, 220.0, 230.0, 70.0));
        assert_eq!(schematics.rect, RenderRect::new(358.0, 220.0, 230.0, 70.0));
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
    fn menu_ui_plan_mobile_matches_upstream_portrait_grid_geometry() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(true, 9));
        let input = MenuFrameInput {
            graphics_width: 720.0,
            graphics_height: 1280.0,
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
        assert!(plan.ui.buttons[0].selected);
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
        assert!(plan.ui.buttons[0].selected);
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
    fn menu_button_role_workshop_is_desktop_root_without_submenu() {
        assert_eq!(MenuButtonRole::Workshop.label(), "WORKSHOP");
        assert!(MenuButtonRole::Workshop.is_desktop_root());
        assert!(!MenuButtonRole::Workshop.has_desktop_submenu());
        assert!(!MenuButtonRole::Workshop.is_submenu());
    }

    #[test]
    fn menu_ui_plan_desktop_inserts_workshop_before_mods_when_enabled_like_java() {
        let mut state = MenuRendererState::new(
            MenuRendererConfig::new(false, 11).with_desktop_workshop_enabled(true),
        );
        let plan = state.render_plan(MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
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
        assert_eq!(workshop.label, "WORKSHOP");
        assert!(workshop.rect.y < mods.rect.y);
    }

    #[test]
    fn menu_ui_plan_desktop_inserts_custom_buttons_before_quit_like_java() {
        let mut state = MenuRendererState::new(MenuRendererConfig::new(false, 11));
        assert_eq!(
            state.add_custom_button("SERVER BROWSER"),
            MenuButtonRole::Custom(0)
        );
        let input = MenuFrameInput {
            graphics_width: 1280.0,
            graphics_height: 720.0,
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
        assert!(custom.rect.y < quit.rect.y);
        assert_eq!(
            state.hit_test_ui(input, custom.rect.center().x, custom.rect.center().y),
            Some(MenuButtonRole::Custom(0))
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
