use std::io::{self, Read, Write};

use crate::mindustry::{
    ctype::ContentId,
    world::{
        meta::{BlockGroup, Env},
        Block,
    },
};

use super::{
    item_source_accept_item, item_source_configure, item_source_update, item_void_accept_item,
    item_void_handle_item, liquid_source_configure, liquid_source_update,
    liquid_void_accept_liquid, liquid_void_handle_liquid, power_source_production,
    power_void_consumption, read_item_source_config, read_liquid_source_config,
    write_item_source_config, write_liquid_source_config, ItemSourceState, ItemSourceStep,
    LiquidSourceState, LiquidSourceStep, DEFAULT_ITEM_SOURCE_ITEMS_PER_SECOND,
    DEFAULT_LIQUID_SOURCE_CAPACITY, DEFAULT_POWER_SOURCE_PRODUCTION, POWER_VOID_CONSUMPTION,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PowerSource {
    pub block: Block,
    pub max_nodes: i32,
    pub power_production: f32,
    pub draw_disabled: bool,
}

impl PowerSource {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = false;
        block.solid = true;
        block.has_power = true;
        block.outputs_power = true;
        block.consumes_power = false;
        block.group = BlockGroup::Power;
        block.destructible = true;
        block.env_enabled = Env::ANY;

        Self {
            block,
            max_nodes: 100,
            power_production: DEFAULT_POWER_SOURCE_PRODUCTION,
            draw_disabled: true,
        }
    }

    pub fn production(&self, enabled: bool) -> f32 {
        power_source_production(enabled, self.power_production)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PowerVoid {
    pub block: Block,
    pub consume_power: f32,
    pub enable_draw_status: bool,
}

impl PowerVoid {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.has_power = true;
        block.consumes_power = true;
        block.group = BlockGroup::Power;
        block.env_enabled = Env::ANY;

        Self {
            block,
            consume_power: POWER_VOID_CONSUMPTION,
            enable_draw_status: false,
        }
    }

    pub fn consumption(&self) -> f32 {
        power_void_consumption()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemSource {
    pub block: Block,
    pub items_per_second: i32,
    pub outputs_items: bool,
    pub no_update_disabled: bool,
}

impl ItemSource {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.has_items = true;
        block.group = BlockGroup::Transportation;
        block.configurable = true;
        block.save_config = true;
        block.clear_on_double_tap = true;
        block.env_enabled = Env::ANY;

        Self {
            block,
            items_per_second: DEFAULT_ITEM_SOURCE_ITEMS_PER_SECOND,
            outputs_items: true,
            no_update_disabled: true,
        }
    }

    pub fn accept_item(&self) -> bool {
        item_source_accept_item()
    }

    pub fn configure(&self, state: &mut ItemSourceState, item: Option<ContentId>) {
        item_source_configure(state, item);
    }

    pub fn update(&self, state: &mut ItemSourceState, edelta: f32) -> ItemSourceStep {
        item_source_update(state, self.items_per_second, edelta)
    }

    pub fn write_config<W: Write>(
        &self,
        write: &mut W,
        output_item: Option<ContentId>,
    ) -> io::Result<()> {
        write_item_source_config(write, output_item)
    }

    pub fn read_config<R: Read>(&self, read: &mut R) -> io::Result<Option<ContentId>> {
        read_item_source_config(read)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemVoid {
    pub block: Block,
    pub accepts_items: bool,
}

impl ItemVoid {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.group = BlockGroup::Transportation;
        block.env_enabled = Env::ANY;

        Self {
            block,
            accepts_items: true,
        }
    }

    pub fn accept_item(&self, enabled: bool) -> bool {
        item_void_accept_item(enabled)
    }

    pub fn handle_item(&self, enabled: bool, amount: i32) -> i32 {
        item_void_handle_item(enabled, amount)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidSource {
    pub block: Block,
    pub liquid_capacity: f32,
    pub outputs_liquid: bool,
    pub display_flow: bool,
    pub no_update_disabled: bool,
}

impl LiquidSource {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.has_liquids = true;
        block.liquid_capacity = DEFAULT_LIQUID_SOURCE_CAPACITY;
        block.configurable = true;
        block.save_config = true;
        block.clear_on_double_tap = true;
        block.group = BlockGroup::Liquids;
        block.env_enabled = Env::ANY;

        Self {
            block,
            liquid_capacity: DEFAULT_LIQUID_SOURCE_CAPACITY,
            outputs_liquid: true,
            display_flow: false,
            no_update_disabled: true,
        }
    }

    pub fn configure(&self, state: &mut LiquidSourceState, liquid: Option<ContentId>) {
        liquid_source_configure(state, liquid);
    }

    pub fn update(&self, state: &mut LiquidSourceState) -> LiquidSourceStep {
        liquid_source_update(state, self.liquid_capacity)
    }

    pub fn write_config<W: Write>(
        &self,
        write: &mut W,
        source: Option<ContentId>,
    ) -> io::Result<()> {
        write_liquid_source_config(write, source)
    }

    pub fn read_config<R: Read>(
        &self,
        read: &mut R,
        revision: u8,
    ) -> io::Result<Option<ContentId>> {
        read_liquid_source_config(read, revision)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidVoid {
    pub block: Block,
}

impl LiquidVoid {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.has_liquids = true;
        block.liquid_capacity = DEFAULT_LIQUID_SOURCE_CAPACITY;
        block.group = BlockGroup::Liquids;
        block.env_enabled = Env::ANY;

        Self { block }
    }

    pub fn accept_liquid(&self, enabled: bool) -> bool {
        liquid_void_accept_liquid(enabled)
    }

    pub fn handle_liquid(&self, enabled: bool, amount: f32) -> f32 {
        liquid_void_handle_liquid(enabled, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_block_shells_follow_java_constructor_defaults() {
        let power_source = PowerSource::new("power-source");
        assert!(power_source.block.has_power);
        assert!(power_source.block.outputs_power);
        assert!(!power_source.block.consumes_power);
        assert_eq!(power_source.block.group, BlockGroup::Power);
        assert_eq!(power_source.max_nodes, 100);
        assert_eq!(
            power_source.production(true),
            DEFAULT_POWER_SOURCE_PRODUCTION
        );

        let power_void = PowerVoid::new("power-void");
        assert!(power_void.block.has_power);
        assert!(power_void.block.consumes_power);
        assert_eq!(power_void.consumption(), f32::MAX);
        assert!(!power_void.enable_draw_status);

        let item_source = ItemSource::new("item-source");
        assert!(item_source.block.has_items);
        assert_eq!(
            item_source.items_per_second,
            DEFAULT_ITEM_SOURCE_ITEMS_PER_SECOND
        );
        assert!(item_source.outputs_items);

        let item_void = ItemVoid::new("item-void");
        assert!(item_void.accept_item(true));
        assert_eq!(item_void.handle_item(true, 7), 7);
        assert_eq!(item_void.handle_item(false, 7), 0);

        let liquid_source = LiquidSource::new("liquid-source");
        assert!(liquid_source.block.has_liquids);
        assert_eq!(
            liquid_source.liquid_capacity,
            DEFAULT_LIQUID_SOURCE_CAPACITY
        );
        assert!(liquid_source.outputs_liquid);
        assert!(!liquid_source.display_flow);

        let liquid_void = LiquidVoid::new("liquid-void");
        assert!(liquid_void.block.has_liquids);
        assert_eq!(liquid_void.handle_liquid(true, 3.5), 3.5);
        assert_eq!(liquid_void.handle_liquid(false, 3.5), 0.0);
    }

    #[test]
    fn sandbox_state_and_codec_helpers_delegate_to_migrated_functions() {
        let source = ItemSource::new("item-source");
        let mut item_state = ItemSourceState::default();
        source.configure(&mut item_state, Some(4));
        let step = source.update(&mut item_state, 0.30);
        assert_eq!(step.item, Some(4));
        assert_eq!(step.produced, 0);

        let mut bytes = Vec::new();
        source.write_config(&mut bytes, Some(4)).unwrap();
        assert_eq!(source.read_config(&mut bytes.as_slice()).unwrap(), Some(4));

        let liquid = LiquidSource::new("liquid-source");
        let mut liquid_state = LiquidSourceState::default();
        liquid.configure(&mut liquid_state, Some(7));
        let step = liquid.update(&mut liquid_state);
        assert_eq!(step.dumped, Some(7));
        assert_eq!(liquid_state.stored_liquid, Some(7));

        let mut bytes = Vec::new();
        liquid.write_config(&mut bytes, Some(7)).unwrap();
        assert_eq!(
            liquid.read_config(&mut bytes.as_slice(), 1).unwrap(),
            Some(7)
        );

        let mut power_state = super::PowerSourceState::default();
        super::power_source_on_proximity_update(&mut power_state, false);
        assert!(!power_state.enabled);
    }
}
