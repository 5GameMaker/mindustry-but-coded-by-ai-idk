//! World generator interface mirroring upstream `mindustry.maps.generators.WorldGenerator`.

use crate::mindustry::world::{Tiles, WorldParams};

pub trait WorldGenerator {
    fn generate(&mut self, tiles: &mut Tiles, params: &WorldParams);

    /// Do not modify tiles here. This is only for specialized configuration.
    fn post_generate(&mut self, _tiles: &mut Tiles) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::Tile;

    #[derive(Debug, Default)]
    struct DummyGenerator {
        calls: Vec<&'static str>,
    }

    impl WorldGenerator for DummyGenerator {
        fn generate(&mut self, tiles: &mut Tiles, params: &WorldParams) {
            self.calls.push("generate");
            assert_eq!(params.seed_offset, 7);
            tiles.set(0, 0, Tile::new(3, 4));
        }
    }

    #[test]
    fn world_generator_generate_receives_tiles_and_params() {
        let mut generator = DummyGenerator::default();
        let mut tiles = Tiles::new(2, 2);
        let params = WorldParams {
            seed_offset: 7,
            ..WorldParams::new()
        };

        generator.generate(&mut tiles, &params);

        assert_eq!(generator.calls, vec!["generate"]);
        assert_eq!(tiles.get(0, 0).unwrap().x, 3);
        assert_eq!(tiles.get(0, 0).unwrap().y, 4);
    }

    #[test]
    fn post_generate_default_is_noop_like_java_interface() {
        let mut generator = DummyGenerator::default();
        let mut tiles = Tiles::new(1, 1);
        let before = tiles.clone();

        generator.post_generate(&mut tiles);

        assert_eq!(tiles, before);
        assert!(generator.calls.is_empty());
    }
}
