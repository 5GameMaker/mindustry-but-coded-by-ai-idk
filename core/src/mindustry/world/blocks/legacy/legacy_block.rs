//! Legacy block shell mirroring upstream
//! `mindustry.world.blocks.legacy.LegacyBlock`.

use crate::mindustry::world::{Block, BlockId, Tile};

use super::{legacy_remove_self, LegacyRemoval};

/// Any subclass of this is removed upon world load in upstream.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBlock {
    pub base: Block,
    pub in_editor: bool,
    pub generate_icons: bool,
}

impl LegacyBlock {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        Self {
            base: Block::new(id, name),
            in_editor: false,
            generate_icons: false,
        }
    }

    /// Java: `tile.remove()`, which sets the tile block to `Blocks.air`.
    pub fn remove_self(&self, tile: &mut Tile) -> LegacyRemoval {
        tile.block = Tile::AIR;
        tile.build = None;
        legacy_remove_self()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_block_constructor_hides_editor_and_icon_generation_like_java() {
        let legacy = LegacyBlock::new(7, "legacy-block");

        assert_eq!(legacy.base.id, 7);
        assert_eq!(legacy.base.name, "legacy-block");
        assert!(!legacy.in_editor);
        assert!(!legacy.generate_icons);
    }

    #[test]
    fn legacy_block_remove_self_removes_tile_block() {
        let legacy = LegacyBlock::new(7, "legacy-block");
        let mut tile = Tile::with_blocks(1, 2, 3, 4, 7);

        let action = legacy.remove_self(&mut tile);

        assert_eq!(action, LegacyRemoval::Remove);
        assert_eq!(tile.block, Tile::AIR);
        assert_eq!(tile.build, None);
    }
}
