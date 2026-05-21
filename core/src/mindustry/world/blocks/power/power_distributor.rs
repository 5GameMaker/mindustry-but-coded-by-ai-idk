//! Power distributor shell mirroring upstream `mindustry.world.blocks.power.PowerDistributor`.

use super::PowerBlock;
use crate::mindustry::world::Block;

#[derive(Debug, Clone, PartialEq)]
pub struct PowerDistributor {
    pub block: Block,
}

impl PowerDistributor {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = PowerBlock::new(name).block;
        block.consumes_power = false;
        block.outputs_power = true;
        Self { block }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::meta::BlockGroup;

    #[test]
    fn power_distributor_inherits_power_block_and_outputs_power() {
        let distributor = PowerDistributor::new("battery").block;

        assert_eq!(distributor.group, BlockGroup::Power);
        assert!(distributor.update);
        assert!(distributor.solid);
        assert!(distributor.has_power);
        assert!(!distributor.consumes_power);
        assert!(distributor.outputs_power);
    }
}
