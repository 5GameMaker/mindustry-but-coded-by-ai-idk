//! Map generator interfaces and lightweight data carriers.

use crate::mindustry::{
    maps::MapDescriptor,
    world::{Tiles, WorldParams},
};

pub mod base_generator {
    //! Base generator metadata mirrored from upstream `mindustry.maps.generators.BaseGenerator`.

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct BaseGenerator {
        pub insanity: bool,
        pub range: i32,
    }
}

pub mod basic_generator {
    //! Basic generator metadata mirrored from upstream `mindustry.maps.generators.BasicGenerator`.

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BasicGenerator {
        pub rand_seed: u64,
        pub width: i32,
        pub height: i32,
        pub default_loadout: Option<String>,
        pub floor: Option<String>,
        pub block: Option<String>,
        pub ore: Option<String>,
    }

    impl Default for BasicGenerator {
        fn default() -> Self {
            Self {
                rand_seed: 0,
                width: 0,
                height: 0,
                default_loadout: Some("basicShard".into()),
                floor: None,
                block: None,
                ore: None,
            }
        }
    }
}

pub mod planet_generator {
    //! Planet generator metadata mirrored from upstream `mindustry.maps.generators.PlanetGenerator`.

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct PlanetGenerator {
        pub base_seed: i32,
        pub seed: i32,
        pub sector: Option<String>,
    }
}

pub mod blank_planet_generator {
    //! Marker metadata for upstream `BlankPlanetGenerator`.

    use super::planet_generator::PlanetGenerator;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct BlankPlanetGenerator {
        pub generator: PlanetGenerator,
    }
}

pub use base_generator::BaseGenerator;
pub use basic_generator::BasicGenerator;
pub use blank_planet_generator::BlankPlanetGenerator;
pub use planet_generator::PlanetGenerator;
pub mod world_generator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMapGenerator {
    pub map: MapDescriptor,
    pub preset: Option<String>,
}

impl FileMapGenerator {
    pub fn new<P: Into<String>>(map: MapDescriptor, preset: Option<P>) -> Self {
        Self {
            map,
            preset: preset.map(Into::into),
        }
    }

    pub fn map_name(&self) -> &str {
        self.map.name()
    }

    pub fn is_map(&self) -> bool {
        true
    }
}

impl world_generator::WorldGenerator for FileMapGenerator {
    fn generate(&mut self, _tiles: &mut Tiles, _params: &WorldParams) {}
}

pub use world_generator::WorldGenerator;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn generator_metadata_defaults_and_file_map_wrapper_match_upstream_surface() {
        let base = BaseGenerator::default();
        assert!(!base.insanity);
        assert_eq!(base.range, 0);

        let basic = BasicGenerator::default();
        assert_eq!(basic.default_loadout.as_deref(), Some("basicShard"));

        let planet = PlanetGenerator::default();
        assert_eq!(planet.base_seed, 0);
        assert_eq!(planet.seed, 0);
        assert!(planet.sector.is_none());

        let blank = BlankPlanetGenerator::default();
        assert_eq!(blank.generator, PlanetGenerator::default());

        let map = MapDescriptor::new(
            "maps/test.msav",
            10,
            10,
            BTreeMap::from([(String::from("name"), String::from("Test"))]),
            true,
            11,
            157,
        );
        let file_map = FileMapGenerator::new(map.clone(), Some("groundZero"));
        assert_eq!(file_map.map, map);
        assert_eq!(file_map.preset.as_deref(), Some("groundZero"));
        assert!(file_map.is_map());
        assert_eq!(file_map.map_name(), "Test");
    }
}
