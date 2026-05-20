use std::collections::HashMap;

use super::Sector;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlanetData {
    pub presets: HashMap<String, i32>,
    pub attack_sectors: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetMeta {
    pub name: String,
    pub localized_name: String,
    pub orbit_spacing: f32,
    pub radius: f32,
    pub total_radius: f32,
    pub orbit_radius: f32,
    pub start_sector: usize,
    pub sector_seed: i32,
    pub launch_capacity_multiplier: f32,
    pub allow_launch_schematics: bool,
    pub allow_launch_loadout: bool,
    pub allow_sector_invasion: bool,
    pub allow_legacy_launch_pads: bool,
    pub clear_sector_on_lose: bool,
    pub enemy_build_speed_multiplier: f32,
    pub enemy_factory_activation_delay: f32,
    pub enemy_infinite_items: bool,
    pub enemy_core_spawn_replace: bool,
    pub prebuild_base: bool,
    pub allow_waves: bool,
    pub allow_launch_to_numbered: bool,
    pub allow_campaign_rules: bool,
    pub allow_self_sector_launch: bool,
    pub auto_assign_planet: bool,
    pub accessible: bool,
    pub visible: bool,
    pub draw_orbit: bool,
    pub update_lighting: bool,
    pub default_env: u32,
    pub default_core: String,
    pub icon: String,
    pub has_generator: bool,
    pub has_grid: bool,
}

impl PlanetMeta {
    pub fn new(name: impl Into<String>, radius: f32) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            orbit_spacing: 12.0,
            radius,
            total_radius: radius,
            orbit_radius: 0.0,
            start_sector: 0,
            sector_seed: -1,
            launch_capacity_multiplier: 0.25,
            allow_launch_schematics: false,
            allow_launch_loadout: false,
            allow_sector_invasion: false,
            allow_legacy_launch_pads: false,
            clear_sector_on_lose: false,
            enemy_build_speed_multiplier: 1.0,
            enemy_factory_activation_delay: 0.0,
            enemy_infinite_items: true,
            enemy_core_spawn_replace: false,
            prebuild_base: true,
            allow_waves: false,
            allow_launch_to_numbered: true,
            allow_campaign_rules: false,
            allow_self_sector_launch: false,
            auto_assign_planet: true,
            accessible: true,
            visible: true,
            draw_orbit: true,
            update_lighting: true,
            default_env: default_planet_env(),
            default_core: "core-shard".into(),
            icon: "planet".into(),
            has_generator: false,
            has_grid: false,
        }
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn with_grid(mut self, has_grid: bool) -> Self {
        self.has_grid = has_grid;
        self
    }

    pub fn with_generator(mut self, has_generator: bool) -> Self {
        self.has_generator = has_generator;
        self
    }

    pub fn get_start_sector<'a>(&self, sectors: &'a [Sector]) -> Option<&'a Sector> {
        if sectors.is_empty() {
            None
        } else {
            sectors
                .get(self.start_sector)
                .or_else(|| sectors.get(sectors.len() - 1))
        }
    }

    pub fn get_last_sector<'a>(
        &self,
        sectors: &'a [Sector],
        last_sector: Option<usize>,
    ) -> Option<&'a Sector> {
        if sectors.is_empty() {
            None
        } else {
            let index = last_sector
                .unwrap_or(self.start_sector)
                .min(sectors.len() - 1);
            sectors.get(index)
        }
    }

    pub fn has_grid(&self, sectors: &[Sector]) -> bool {
        self.has_grid && self.has_generator && !sectors.is_empty()
    }

    pub fn is_landable(&self, sectors: &[Sector]) -> bool {
        !sectors.is_empty()
    }

    pub fn update_total_radius(&mut self, children: &[PlanetOrbit]) {
        self.total_radius = self.radius;
        for child in children {
            self.total_radius = self
                .total_radius
                .max(child.orbit_radius + child.total_radius);
        }
    }

    pub fn to_sector_defaults(&self, sector_count: usize) -> super::SectorPlanetDefaults {
        super::SectorPlanetDefaults::new(self.name.clone())
            .localized(self.localized_name.clone())
            .launch_defaults(self.allow_launch_schematics, self.allow_launch_loadout)
            .sector_count(sector_count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetOrbit {
    pub orbit_radius: f32,
    pub total_radius: f32,
}

impl PlanetOrbit {
    pub const fn new(orbit_radius: f32, total_radius: f32) -> Self {
        Self {
            orbit_radius,
            total_radius,
        }
    }
}

pub fn default_planet_env() -> u32 {
    crate::mindustry::world::meta::Env::TERRESTRIAL
        | crate::mindustry::world::meta::Env::SPORES
        | crate::mindustry::world::meta::Env::GROUND_OIL
        | crate::mindustry::world::meta::Env::GROUND_WATER
        | crate::mindustry::world::meta::Env::OXYGEN
}

pub fn last_sector_key(planet_name: &str) -> String {
    format!("{planet_name}-last-sector")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planet_data_and_meta_defaults_match_upstream_initializers() {
        let data = PlanetData::default();
        assert!(data.presets.is_empty());
        assert!(data.attack_sectors.is_empty());

        let meta = PlanetMeta::new("serpulo", 3.0);
        assert_eq!(meta.name, "serpulo");
        assert_eq!(meta.localized_name, "serpulo");
        assert_eq!(meta.orbit_spacing, 12.0);
        assert_eq!(meta.radius, 3.0);
        assert_eq!(meta.total_radius, 3.0);
        assert_eq!(meta.orbit_radius, 0.0);
        assert_eq!(meta.start_sector, 0);
        assert_eq!(meta.sector_seed, -1);
        assert_eq!(meta.launch_capacity_multiplier, 0.25);
        assert!(!meta.allow_launch_schematics);
        assert!(!meta.allow_launch_loadout);
        assert!(!meta.allow_sector_invasion);
        assert!(!meta.allow_legacy_launch_pads);
        assert!(!meta.clear_sector_on_lose);
        assert_eq!(meta.enemy_build_speed_multiplier, 1.0);
        assert_eq!(meta.enemy_factory_activation_delay, 0.0);
        assert!(meta.enemy_infinite_items);
        assert!(!meta.enemy_core_spawn_replace);
        assert!(meta.prebuild_base);
        assert!(!meta.allow_waves);
        assert!(meta.allow_launch_to_numbered);
        assert!(!meta.allow_campaign_rules);
        assert!(!meta.allow_self_sector_launch);
        assert!(meta.auto_assign_planet);
        assert!(meta.accessible);
        assert!(meta.visible);
        assert!(meta.draw_orbit);
        assert!(meta.update_lighting);
        assert_eq!(meta.default_env, default_planet_env());
        assert_eq!(meta.default_core, "core-shard");
        assert_eq!(meta.icon, "planet");
    }

    #[test]
    fn start_last_grid_and_landable_helpers_follow_java_edges() {
        let mut meta = PlanetMeta::new("serpulo", 3.0)
            .with_grid(true)
            .with_generator(true);
        let sectors = vec![Sector::new(0), Sector::new(1), Sector::new(2)];

        assert_eq!(meta.get_start_sector(&[]), None);
        assert!(!meta.has_grid(&[]));
        assert!(!meta.is_landable(&[]));

        meta.start_sector = 1;
        assert_eq!(meta.get_start_sector(&sectors).unwrap().id, 1);
        assert_eq!(meta.get_last_sector(&sectors, None).unwrap().id, 1);
        assert_eq!(meta.get_last_sector(&sectors, Some(99)).unwrap().id, 2);
        assert!(meta.has_grid(&sectors));
        assert!(meta.is_landable(&sectors));

        meta.has_generator = false;
        assert!(!meta.has_grid(&sectors));
    }

    #[test]
    fn update_total_radius_uses_largest_child_outer_bound() {
        let mut meta = PlanetMeta::new("sun", 10.0);
        meta.update_total_radius(&[
            PlanetOrbit::new(12.0, 3.0),
            PlanetOrbit::new(2.0, 4.0),
            PlanetOrbit::new(20.0, 5.0),
        ]);
        assert_eq!(meta.total_radius, 25.0);

        meta.update_total_radius(&[]);
        assert_eq!(meta.total_radius, 10.0);
    }

    #[test]
    fn sector_defaults_and_settings_keys_bridge_planet_to_sector_logic() {
        let mut meta = PlanetMeta::new("serpulo", 3.0).localized("Serpulo");
        meta.allow_launch_schematics = true;
        meta.allow_launch_loadout = false;

        let defaults = meta.to_sector_defaults(10);
        assert_eq!(defaults.name, "serpulo");
        assert_eq!(defaults.localized_name, "Serpulo");
        assert!(defaults.allow_launch_schematics);
        assert!(!defaults.allow_launch_loadout);
        assert_eq!(defaults.sector_count, 10);
        assert_eq!(last_sector_key("serpulo"), "serpulo-last-sector");
    }
}
