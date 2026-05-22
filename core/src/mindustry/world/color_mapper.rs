use std::collections::HashMap;

use crate::mindustry::world::block::{Block, BlockId};

pub const BLACK_AIR_RGBA: u32 = 0x000000ff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorMapper {
    air: BlockId,
    color_to_block: HashMap<u32, BlockId>,
}

impl ColorMapper {
    pub fn new(air: BlockId) -> Self {
        let mut mapper = Self {
            air,
            color_to_block: HashMap::new(),
        };
        mapper.color_to_block.insert(BLACK_AIR_RGBA, air);
        mapper
    }

    pub fn load<'a>(air: BlockId, blocks: impl IntoIterator<Item = &'a Block>) -> Self {
        let mut mapper = Self {
            air,
            color_to_block: HashMap::new(),
        };

        for block in blocks {
            mapper.color_to_block.insert(block.map_color_rgba, block.id);
        }
        mapper.color_to_block.insert(BLACK_AIR_RGBA, air);
        mapper
    }

    pub fn get(&self, color: u32) -> BlockId {
        self.color_to_block.get(&color).copied().unwrap_or(self.air)
    }

    pub fn len(&self) -> usize {
        self.color_to_block.len()
    }

    pub fn is_empty(&self) -> bool {
        self.color_to_block.is_empty()
    }
}

impl Default for ColorMapper {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: BlockId, color: u32) -> Block {
        let mut block = Block::new(id, format!("block-{id}"));
        block.map_color_rgba = color;
        block
    }

    #[test]
    fn color_mapper_loads_block_map_colors_and_black_air_override() {
        let air = block(0, 0xffffffff);
        let copper = block(1, copper_u32());
        let lead = block(2, 0x6f6f6fff);
        let mapper = ColorMapper::load(air.id, [&air, &copper, &lead]);

        assert_eq!(mapper.get(copper.map_color_rgba), 1);
        assert_eq!(mapper.get(lead.map_color_rgba), 2);
        assert_eq!(mapper.get(BLACK_AIR_RGBA), 0);
        assert_eq!(mapper.get(0x12345678), 0);
    }

    #[test]
    fn color_mapper_uses_later_blocks_for_duplicate_colors_like_int_map_put() {
        let first = block(1, 0xaabbccdd);
        let second = block(2, 0xaabbccdd);
        let mapper = ColorMapper::load(0, [&first, &second]);

        assert_eq!(mapper.get(0xaabbccdd), 2);
        assert_eq!(mapper.len(), 2);
    }

    const fn copper_u32() -> u32 {
        0xd99d73ff
    }
}
