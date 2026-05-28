/// Data-oriented migration of upstream `MenuRenderer`.
///
/// The Java renderer generates a temporary menu world, draws floor/overlay and
/// wall caches, then renders flyers and a dark fullscreen overlay.  This module
/// keeps generation and render planning in `mindustry-core`; concrete backends
/// are expected to translate `MenuRenderCommand` into GPU/cache calls.

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
}

impl MenuRendererConfig {
    pub const fn new(mobile: bool, seed: u64) -> Self {
        Self {
            mobile,
            seed,
            tile_size: MENU_TILE_SIZE,
        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct MenuFramePlan {
    pub camera_x: f32,
    pub camera_y: f32,
    pub camera_width: f32,
    pub camera_height: f32,
    pub scaling: f32,
    pub commands: Vec<MenuRenderCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuRendererState {
    pub config: MenuRendererConfig,
    pub world: MenuWorldPlan,
    pub time: f32,
    pub flyer_rotation: f32,
    pub flyer_count: usize,
    pub flyer_type: &'static str,
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
            commands,
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
    }
}
