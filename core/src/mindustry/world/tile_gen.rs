//! Temporary generated tile data mirroring upstream `mindustry.world.TileGen`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileGen {
    pub floor: String,
    pub block: String,
    pub overlay: String,
}

impl Default for TileGen {
    fn default() -> Self {
        let mut tile = Self {
            floor: String::new(),
            block: String::new(),
            overlay: String::new(),
        };
        tile.reset();
        tile
    }
}

impl TileGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.floor = "stone".into();
        self.block = "air".into();
        self.overlay = "air".into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_gen_defaults_match_java_initializer_reset() {
        let tile = TileGen::new();

        assert_eq!(tile.floor, "stone");
        assert_eq!(tile.block, "air");
        assert_eq!(tile.overlay, "air");
    }

    #[test]
    fn reset_restores_stone_floor_and_air_block_overlay() {
        let mut tile = TileGen {
            floor: "sand".into(),
            block: "router".into(),
            overlay: "ore-copper".into(),
        };

        tile.reset();

        assert_eq!(tile, TileGen::new());
    }
}
