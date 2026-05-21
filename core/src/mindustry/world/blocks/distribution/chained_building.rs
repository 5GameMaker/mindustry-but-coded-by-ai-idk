//! Chained building interface mirroring upstream
//! `mindustry.world.blocks.distribution.ChainedBuilding`.

use crate::mindustry::world::BuildingRef;

pub trait ChainedBuilding {
    /// Java: `Building next()`.
    ///
    /// `None` represents Java's nullable `Building` reference when a chain end
    /// has no next building yet.
    fn next(&self) -> Option<BuildingRef>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct ChainNode {
        next: Option<BuildingRef>,
    }

    impl ChainedBuilding for ChainNode {
        fn next(&self) -> Option<BuildingRef> {
            self.next
        }
    }

    #[test]
    fn chained_building_exposes_optional_next_building_reference() {
        let target = BuildingRef {
            tile_pos: 0x0001_0002,
            block: 7,
            team: 1,
            rotation: 2,
        };

        assert_eq!(ChainNode { next: Some(target) }.next(), Some(target));
        assert_eq!(ChainNode { next: None }.next(), None);
    }
}
