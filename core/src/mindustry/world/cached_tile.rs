use super::{Block, BuildingRef, Tile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedTile {
    pub tile: Tile,
    pub silent: bool,
}

impl CachedTile {
    pub fn new() -> Self {
        Self {
            tile: Tile::new(0, 0),
            silent: true,
        }
    }

    pub fn pre_changed(&self) {}

    pub fn change_build(&mut self, block: &Block, team: i32, rotation: i32) {
        self.tile.build = None;
        self.tile.block = block.id;
        if block.has_building() {
            self.tile.build = Some(BuildingRef {
                tile_pos: self.tile.pos(),
                block: block.id,
                team,
                rotation,
            });
        }
    }
}

impl Default for CachedTile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::Block;

    #[test]
    fn cached_tile_creates_building_only_for_building_blocks() {
        let mut tile = CachedTile::new();
        let mut block = Block::new(3, "router");
        tile.change_build(&block, 1, 0);
        assert!(tile.tile.build.is_none());

        block.update = true;
        tile.change_build(&block, 1, 2);
        assert_eq!(tile.tile.build.unwrap().block, 3);
        assert_eq!(tile.tile.build.unwrap().rotation, 2);
    }
}
