use crate::mindustry::r#type::Sector;
use crate::mindustry::world::{BlockId, Tile};

/// Rust counterpart of upstream `mindustry.world.WorldContext`.
///
/// Implementations own the tile storage; `create` returns the tile value that
/// was inserted, keeping the trait object-safe and independent from a concrete
/// backing array.
pub trait WorldContext {
    /// Return a tile by array index.
    fn tile(&self, index: usize) -> Option<&Tile>;

    /// Create or resize the tile array.
    fn resize(&mut self, width: usize, height: usize);

    /// Create a tile, put it into the backing tile array, then return it.
    fn create(
        &mut self,
        x: i32,
        y: i32,
        floor_id: BlockId,
        overlay_id: BlockId,
        wall_id: BlockId,
    ) -> Tile;

    /// Returns whether the world is already generating.
    fn is_generating(&self) -> bool;

    /// Begins generating.
    fn begin(&mut self);

    /// Ends generating and prepares tiles.
    fn end(&mut self);

    /// Called when a building is finished reading.
    fn on_read_building(&mut self) {}

    /// Called when data finishes reading for a tile.
    fn on_read_tile_data(&mut self) {}

    fn sector(&self) -> Option<&Sector> {
        None
    }

    /// Whether the save/load event fired after `end` counts as a new map load.
    fn is_map(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockWorldContext {
        width: usize,
        height: usize,
        tiles: Vec<Tile>,
        generating: bool,
        read_buildings: usize,
        read_tile_data: usize,
    }

    impl WorldContext for MockWorldContext {
        fn tile(&self, index: usize) -> Option<&Tile> {
            self.tiles.get(index)
        }

        fn resize(&mut self, width: usize, height: usize) {
            self.width = width;
            self.height = height;
            self.tiles.clear();
            self.tiles.reserve(width * height);
        }

        fn create(
            &mut self,
            x: i32,
            y: i32,
            floor_id: BlockId,
            overlay_id: BlockId,
            wall_id: BlockId,
        ) -> Tile {
            let tile = Tile::with_blocks(x, y, floor_id, overlay_id, wall_id);
            self.tiles.push(tile.clone());
            tile
        }

        fn is_generating(&self) -> bool {
            self.generating
        }

        fn begin(&mut self) {
            self.generating = true;
        }

        fn end(&mut self) {
            self.generating = false;
        }

        fn on_read_building(&mut self) {
            self.read_buildings += 1;
        }

        fn on_read_tile_data(&mut self) {
            self.read_tile_data += 1;
        }
    }

    #[test]
    fn world_context_lifecycle_and_defaults_match_java_interface() {
        let mut context = MockWorldContext::default();
        context.resize(2, 2);
        assert_eq!((context.width, context.height), (2, 2));

        context.begin();
        assert!(context.is_generating());
        let tile = context.create(1, 1, 2, 3, 4);
        assert_eq!(tile.floor, 2);
        assert_eq!(tile.overlay, 3);
        assert_eq!(tile.block, 4);
        assert_eq!(context.tile(0), Some(&tile));

        context.on_read_building();
        context.on_read_tile_data();
        assert_eq!(context.read_buildings, 1);
        assert_eq!(context.read_tile_data, 1);
        assert!(context.sector().is_none());
        assert!(!context.is_map());

        context.end();
        assert!(!context.is_generating());
    }
}
