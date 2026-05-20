use crate::mindustry::{
    ctype::{Content, ContentId, ContentType},
    r#type::PlanetMeta,
    world::meta::Env,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetContent {
    pub id: ContentId,
    pub meta: PlanetMeta,
    pub parent_name: Option<String>,
    pub sector_size: Option<i32>,
    pub sector_count: usize,
    pub bloom: bool,
    pub tidal_lock: bool,
    pub has_atmosphere: bool,
    pub load_planet_data: bool,
    pub always_unlocked: bool,
    pub icon_color_rgba: u32,
    pub atmosphere_color_rgba: u32,
    pub land_cloud_color_rgba: u32,
    pub default_attributes: Vec<(String, f32)>,
    pub generator: Option<String>,
}

impl PlanetContent {
    pub fn new(
        id: ContentId,
        name: impl Into<String>,
        parent_name: Option<&str>,
        radius: f32,
        sector_size: Option<i32>,
    ) -> Self {
        let mut meta = PlanetMeta::new(name, radius);
        meta.has_grid = sector_size.is_some_and(|size| size > 0);

        Self {
            id,
            meta,
            parent_name: parent_name.map(str::to_string),
            sector_size,
            sector_count: sector_size.map_or(0, planet_grid_tile_count),
            bloom: false,
            tidal_lock: false,
            has_atmosphere: true,
            load_planet_data: false,
            always_unlocked: false,
            icon_color_rgba: 0xffffffff,
            atmosphere_color_rgba: 0x4db3ffff,
            land_cloud_color_rgba: 0xffffff80,
            default_attributes: Vec::new(),
            generator: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.meta.name
    }

    pub fn localized_name(&self) -> &str {
        &self.meta.localized_name
    }

    pub fn is_landable(&self) -> bool {
        self.sector_count > 0
    }
}

impl Content for PlanetContent {
    fn id(&self) -> ContentId {
        self.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::Planet
    }
}

pub fn load() -> Vec<PlanetContent> {
    let mut next_id = 0;

    let mut sun = make_planet(&mut next_id, "sun", None, 4.0, None);
    sun.bloom = true;
    sun.meta.accessible = false;

    let mut erekir = make_planet(&mut next_id, "erekir", Some("sun"), 1.0, Some(2));
    erekir.generator = Some("ErekirPlanetGenerator".into());
    erekir.meta.has_generator = true;
    erekir.meta.allow_campaign_rules = true;
    erekir.meta.accessible = true;
    erekir.meta.default_env = Env::SCORCHING | Env::TERRESTRIAL;
    erekir.meta.start_sector = 10;
    erekir.meta.orbit_spacing = 2.0;
    erekir.meta.total_radius += 2.6;
    erekir.meta.clear_sector_on_lose = true;
    erekir.meta.default_core = "core-bastion".into();
    erekir.meta.enemy_build_speed_multiplier = 0.4;
    erekir.meta.allow_launch_to_numbered = false;
    erekir.meta.update_lighting = false;
    erekir.tidal_lock = true;
    erekir.icon_color_rgba = 0xff9266ff;
    erekir.atmosphere_color_rgba = 0xf07218ff;
    erekir.default_attributes.push(("heat".into(), 0.8));
    erekir.always_unlocked = true;

    let gier = make_asteroid(&mut next_id, "gier", "erekir");
    let notva = make_asteroid(&mut next_id, "notva", "sun");

    let mut tantros = make_planet(&mut next_id, "tantros", Some("sun"), 1.0, Some(2));
    tantros.generator = Some("TantrosPlanetGenerator".into());
    tantros.meta.has_generator = true;
    tantros.meta.allow_campaign_rules = true;
    tantros.meta.accessible = false;
    tantros.meta.visible = false;
    tantros.atmosphere_color_rgba = 0x3db899ff;
    tantros.icon_color_rgba = 0x597be3ff;
    tantros.meta.start_sector = 10;
    tantros.meta.default_env = Env::UNDERWATER | Env::TERRESTRIAL;

    let mut serpulo = make_planet(&mut next_id, "serpulo", Some("sun"), 1.0, Some(3));
    serpulo.load_planet_data = true;
    serpulo.generator = Some("SerpuloPlanetGenerator".into());
    serpulo.meta.has_generator = true;
    serpulo.meta.allow_campaign_rules = true;
    serpulo.meta.enemy_factory_activation_delay = 60.0 * 60.0 * 2.0;
    serpulo.meta.launch_capacity_multiplier = 0.5;
    serpulo.meta.sector_seed = 2;
    serpulo.meta.allow_waves = true;
    serpulo.meta.allow_legacy_launch_pads = true;
    serpulo.meta.allow_sector_invasion = true;
    serpulo.meta.allow_launch_schematics = true;
    serpulo.meta.enemy_core_spawn_replace = true;
    serpulo.meta.allow_launch_loadout = true;
    serpulo.meta.start_sector = 170;
    serpulo.always_unlocked = true;
    serpulo.meta.allow_self_sector_launch = true;
    serpulo.icon_color_rgba = 0x7d4dffff;
    serpulo.atmosphere_color_rgba = 0x3c1b8fff;
    serpulo.land_cloud_color_rgba = 0x7457ce80;

    let verilus = make_asteroid(&mut next_id, "verilus", "sun");

    vec![sun, erekir, gier, notva, tantros, serpulo, verilus]
}

fn make_planet(
    next_id: &mut ContentId,
    name: &str,
    parent_name: Option<&str>,
    radius: f32,
    sector_size: Option<i32>,
) -> PlanetContent {
    let planet = PlanetContent::new(*next_id, name, parent_name, radius, sector_size);
    *next_id += 1;
    planet
}

fn make_asteroid(next_id: &mut ContentId, name: &str, parent_name: &str) -> PlanetContent {
    let mut asteroid = make_planet(next_id, name, Some(parent_name), 0.12, None);
    asteroid.sector_count = 1;
    asteroid.has_atmosphere = false;
    asteroid.meta.update_lighting = false;
    asteroid.meta.draw_orbit = false;
    asteroid.meta.accessible = false;
    asteroid.meta.default_env = Env::SPACE;
    asteroid.meta.icon = "commandRally".into();
    asteroid.meta.has_generator = true;
    asteroid.generator = Some("AsteroidGenerator".into());
    asteroid
}

fn planet_grid_tile_count(size: i32) -> usize {
    if size <= 0 {
        12
    } else {
        (10_i32 * 3_i32.pow(size as u32) + 2) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_planet_vector_order_matches_upstream_load_order() {
        let planets = load();
        let names: Vec<_> = planets.iter().map(|planet| planet.name()).collect();
        assert_eq!(
            names,
            vec!["sun", "erekir", "gier", "notva", "tantros", "serpulo", "verilus"]
        );
        for (index, planet) in planets.iter().enumerate() {
            assert_eq!(planet.id(), index as ContentId);
            assert_eq!(planet.content_type(), ContentType::Planet);
        }
    }

    #[test]
    fn landable_planet_sector_counts_match_planet_grid_size() {
        let planets = load();

        let sun = by_name(&planets, "sun");
        assert_eq!(sun.sector_count, 0);
        assert!(!sun.is_landable());

        let erekir = by_name(&planets, "erekir");
        assert_eq!(erekir.sector_size, Some(2));
        assert_eq!(erekir.sector_count, 92);
        assert!(erekir.meta.has_grid);
        assert!(erekir.is_landable());

        let serpulo = by_name(&planets, "serpulo");
        assert_eq!(serpulo.sector_size, Some(3));
        assert_eq!(serpulo.sector_count, 272);
    }

    #[test]
    fn serpulo_and_erekir_campaign_flags_match_upstream_load() {
        let planets = load();
        let erekir = by_name(&planets, "erekir");
        assert_eq!(erekir.parent_name.as_deref(), Some("sun"));
        assert_eq!(erekir.meta.default_env, Env::SCORCHING | Env::TERRESTRIAL);
        assert_eq!(erekir.meta.start_sector, 10);
        assert!(erekir.always_unlocked);
        assert!(erekir.meta.clear_sector_on_lose);
        assert_eq!(erekir.meta.default_core, "core-bastion");
        assert_eq!(erekir.meta.enemy_build_speed_multiplier, 0.4);
        assert!(!erekir.meta.allow_launch_to_numbered);
        assert!(!erekir.meta.update_lighting);
        assert_eq!(erekir.default_attributes, vec![("heat".into(), 0.8)]);

        let serpulo = by_name(&planets, "serpulo");
        assert!(serpulo.load_planet_data);
        assert_eq!(serpulo.parent_name.as_deref(), Some("sun"));
        assert_eq!(serpulo.meta.enemy_factory_activation_delay, 7200.0);
        assert_eq!(serpulo.meta.launch_capacity_multiplier, 0.5);
        assert_eq!(serpulo.meta.sector_seed, 2);
        assert!(serpulo.meta.allow_waves);
        assert!(serpulo.meta.allow_legacy_launch_pads);
        assert!(serpulo.meta.allow_sector_invasion);
        assert!(serpulo.meta.allow_launch_schematics);
        assert!(serpulo.meta.enemy_core_spawn_replace);
        assert!(serpulo.meta.allow_launch_loadout);
        assert_eq!(serpulo.meta.start_sector, 170);
        assert!(serpulo.always_unlocked);
        assert!(serpulo.meta.allow_self_sector_launch);
    }

    #[test]
    fn asteroid_planets_match_upstream_single_sector_space_bodies() {
        let planets = load();
        for name in ["gier", "notva", "verilus"] {
            let asteroid = by_name(&planets, name);
            assert_eq!(asteroid.sector_count, 1);
            assert_eq!(asteroid.sector_size, None);
            assert!(asteroid.is_landable());
            assert!(!asteroid.has_atmosphere);
            assert!(!asteroid.meta.update_lighting);
            assert!(!asteroid.meta.draw_orbit);
            assert!(!asteroid.meta.accessible);
            assert_eq!(asteroid.meta.default_env, Env::SPACE);
            assert_eq!(asteroid.meta.icon, "commandRally");
            assert_eq!(asteroid.generator.as_deref(), Some("AsteroidGenerator"));
        }
    }

    #[test]
    fn hidden_tantros_and_sun_flags_match_upstream() {
        let planets = load();
        let sun = by_name(&planets, "sun");
        assert!(sun.bloom);
        assert!(!sun.meta.accessible);

        let tantros = by_name(&planets, "tantros");
        assert!(!tantros.meta.accessible);
        assert!(!tantros.meta.visible);
        assert_eq!(tantros.meta.default_env, Env::UNDERWATER | Env::TERRESTRIAL);
        assert_eq!(tantros.meta.start_sector, 10);
    }

    fn by_name<'a>(planets: &'a [PlanetContent], name: &str) -> &'a PlanetContent {
        planets
            .iter()
            .find(|planet| planet.name() == name)
            .unwrap_or_else(|| panic!("missing planet {name}"))
    }
}
