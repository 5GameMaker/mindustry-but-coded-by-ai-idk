use crate::mindustry::world::BlockId;

use super::OverlayFloorData;

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnBlockData {
    pub overlay: OverlayFloorData,
}

impl SpawnBlockData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        let mut overlay = OverlayFloorData::new(id, name);
        overlay.floor.base.variants = 0;
        overlay.floor.needs_surface = false;
        Self { overlay }
    }

    pub fn should_draw_base(is_editor_tile: bool) -> bool {
        is_editor_tile
    }

    pub fn draw_base(&self, is_editor_tile: bool) -> bool {
        Self::should_draw_base(is_editor_tile)
    }
}

#[cfg(test)]
mod tests {
    use super::SpawnBlockData;

    #[test]
    fn spawn_block_constructor_matches_upstream_defaults() {
        let spawn = SpawnBlockData::new(21, "spawn");
        assert_eq!(spawn.overlay.floor.base.variants, 0);
        assert!(!spawn.overlay.floor.needs_surface);
    }

    #[test]
    fn spawn_block_draw_base_only_returns_true_for_editor_tiles() {
        let spawn = SpawnBlockData::new(22, "spawn");
        assert!(spawn.draw_base(true));
        assert!(SpawnBlockData::should_draw_base(true));
        assert!(!spawn.draw_base(false));
        assert!(!SpawnBlockData::should_draw_base(false));
    }
}
