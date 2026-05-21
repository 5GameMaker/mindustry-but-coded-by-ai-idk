//! Base power block shell mirroring upstream `mindustry.world.blocks.power.PowerBlock`.

use crate::mindustry::world::{meta::BlockGroup, Block};

#[derive(Debug, Clone, PartialEq)]
pub struct PowerBlock {
    pub block: Block,
}

impl PowerBlock {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.has_power = true;
        block.group = BlockGroup::Power;
        Self { block }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_block_constructor_sets_java_base_fields() {
        let block = PowerBlock::new("power-node").block;

        assert_eq!(block.name, "power-node");
        assert!(block.update);
        assert!(block.solid);
        assert!(block.has_power);
        assert_eq!(block.group, BlockGroup::Power);
    }
}
