use crate::mindustry::{
    content::blocks::{BlockRegistry, LiquidBlockData, LiquidBlockKind},
    world::meta::{BlockGroup, Env},
    world::{Block, BlockId},
};

use crate::mindustry::world::blocks::distribution::ItemBridge;

fn base_block(name: impl Into<String>) -> Block {
    Block::new(0, name)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidBlock {
    pub block: Block,
    pub outputs_liquid: bool,
    pub floating: bool,
    pub under_bullets: bool,
    pub no_update_disabled: bool,
    pub can_overdrive: bool,
}

impl LiquidBlock {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = true;
        block.has_liquids = true;
        block.group = BlockGroup::Liquids;
        block.env_enabled |= Env::SPACE | Env::UNDERWATER;
        Self {
            block,
            outputs_liquid: true,
            floating: false,
            under_bullets: false,
            no_update_disabled: false,
            can_overdrive: true,
        }
    }

    pub fn from_data(data: &LiquidBlockData) -> Option<Self> {
        (matches!(
            data.kind,
            LiquidBlockKind::Pump
                | LiquidBlockKind::Conduit
                | LiquidBlockKind::ArmoredConduit
                | LiquidBlockKind::LiquidRouter
                | LiquidBlockKind::LiquidJunction
                | LiquidBlockKind::LiquidBridge
                | LiquidBlockKind::DirectionLiquidBridge
        ))
        .then(|| Self {
            block: data.base.clone(),
            outputs_liquid: data.outputs_liquid,
            floating: data.floating,
            under_bullets: data.under_bullets,
            no_update_disabled: data.no_update_disabled,
            can_overdrive: data.can_overdrive,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry.get_liquid_by_name(name).and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conduit {
    pub liquid_block: LiquidBlock,
    pub bot_color_rgba: u32,
    pub pad_corners: bool,
    pub leaks: bool,
    pub liquid_pressure: f32,
    pub liquid_padding: f32,
    pub speed: f32,
    pub armored: bool,
    pub junction_replacement: Option<BlockId>,
    pub bridge_replacement: Option<BlockId>,
    pub rot_bridge_replacement: Option<BlockId>,
}

impl Conduit {
    pub fn new(name: impl Into<String>) -> Self {
        let mut liquid_block = LiquidBlock::new(name);
        liquid_block.block.rotate = true;
        liquid_block.block.solid = false;
        liquid_block.block.priority = 1;
        liquid_block.floating = true;
        liquid_block.under_bullets = true;
        liquid_block.no_update_disabled = true;
        liquid_block.can_overdrive = false;
        Self {
            liquid_block,
            bot_color_rgba: 0x565656ff,
            pad_corners: true,
            leaks: true,
            liquid_pressure: 1.0,
            liquid_padding: 0.0,
            speed: 5.0,
            armored: false,
            junction_replacement: None,
            bridge_replacement: None,
            rot_bridge_replacement: None,
        }
    }

    pub fn from_data(data: &LiquidBlockData) -> Option<Self> {
        (matches!(data.kind, LiquidBlockKind::Conduit)).then(|| Self {
            liquid_block: LiquidBlock {
                block: data.base.clone(),
                outputs_liquid: data.outputs_liquid,
                floating: data.floating,
                under_bullets: data.under_bullets,
                no_update_disabled: data.no_update_disabled,
                can_overdrive: data.can_overdrive,
            },
            bot_color_rgba: 0x565656ff,
            pad_corners: data.pad_corners,
            leaks: data.leaks,
            liquid_pressure: data.liquid_pressure,
            liquid_padding: data.liquid_padding,
            speed: data.speed,
            armored: data.kind == LiquidBlockKind::ArmoredConduit,
            junction_replacement: data.junction_replacement,
            bridge_replacement: None,
            rot_bridge_replacement: data.rot_bridge_replacement,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry.get_liquid_by_name(name).and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArmoredConduit {
    pub conduit: Conduit,
}

impl ArmoredConduit {
    pub fn new(name: impl Into<String>) -> Self {
        let mut conduit = Conduit::new(name);
        conduit.leaks = false;
        conduit.armored = true;
        Self { conduit }
    }

    pub fn from_data(data: &LiquidBlockData) -> Option<Self> {
        (matches!(data.kind, LiquidBlockKind::ArmoredConduit)).then(|| Self {
            conduit: Conduit {
                liquid_block: LiquidBlock {
                    block: data.base.clone(),
                    outputs_liquid: data.outputs_liquid,
                    floating: data.floating,
                    under_bullets: data.under_bullets,
                    no_update_disabled: data.no_update_disabled,
                    can_overdrive: data.can_overdrive,
                },
                bot_color_rgba: 0x565656ff,
                pad_corners: data.pad_corners,
                leaks: false,
                liquid_pressure: data.liquid_pressure,
                liquid_padding: data.liquid_padding,
                speed: data.speed,
                armored: true,
                junction_replacement: data.junction_replacement,
                bridge_replacement: None,
                rot_bridge_replacement: data.rot_bridge_replacement,
            },
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry.get_liquid_by_name(name).and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidRouter {
    pub liquid_block: LiquidBlock,
    pub liquid_padding: f32,
}

impl LiquidRouter {
    pub fn new(name: impl Into<String>) -> Self {
        let mut liquid_block = LiquidBlock::new(name);
        liquid_block.block.solid = true;
        liquid_block.floating = true;
        liquid_block.no_update_disabled = true;
        liquid_block.can_overdrive = false;
        Self {
            liquid_block,
            liquid_padding: 0.0,
        }
    }

    pub fn from_data(data: &LiquidBlockData) -> Option<Self> {
        (matches!(data.kind, LiquidBlockKind::LiquidRouter)).then(|| Self {
            liquid_block: LiquidBlock {
                block: data.base.clone(),
                outputs_liquid: data.outputs_liquid,
                floating: data.floating,
                under_bullets: data.under_bullets,
                no_update_disabled: data.no_update_disabled,
                can_overdrive: data.can_overdrive,
            },
            liquid_padding: data.liquid_padding,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry.get_liquid_by_name(name).and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidJunction {
    pub liquid_block: LiquidBlock,
}

impl LiquidJunction {
    pub fn new(name: impl Into<String>) -> Self {
        let mut liquid_block = LiquidBlock::new(name);
        liquid_block.floating = true;
        Self { liquid_block }
    }

    pub fn from_data(data: &LiquidBlockData) -> Option<Self> {
        (matches!(data.kind, LiquidBlockKind::LiquidJunction)).then(|| Self {
            liquid_block: LiquidBlock {
                block: data.base.clone(),
                outputs_liquid: data.outputs_liquid,
                floating: data.floating,
                under_bullets: data.under_bullets,
                no_update_disabled: data.no_update_disabled,
                can_overdrive: data.can_overdrive,
            },
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry.get_liquid_by_name(name).and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidBridge {
    pub bridge: ItemBridge,
    pub outputs_liquid: bool,
    pub can_overdrive: bool,
    pub liquid_capacity: f32,
}

impl LiquidBridge {
    pub fn new(name: impl Into<String>) -> Self {
        let mut bridge = ItemBridge::new(name);
        bridge.block.has_items = false;
        bridge.block.has_liquids = true;
        bridge.block.group = BlockGroup::Liquids;
        bridge.block.env_enabled = Env::ANY;
        Self {
            bridge,
            outputs_liquid: true,
            can_overdrive: false,
            liquid_capacity: 100.0,
        }
    }

    pub fn from_data(data: &LiquidBlockData) -> Option<Self> {
        (matches!(data.kind, LiquidBlockKind::LiquidBridge)).then(|| Self {
            bridge: ItemBridge {
                block: data.base.clone(),
                range: data.range as i32,
                transport_time: 0.0,
                fade_in: data.fade_in,
                move_arrows: data.move_arrows,
                pulse: data.pulse,
                arrow_spacing: data.arrow_spacing,
                arrow_offset: data.arrow_offset,
                arrow_period: data.arrow_period,
                arrow_time_scl: data.arrow_time_scl,
                bridge_width: data.bridge_width,
                under_bullets: data.under_bullets,
                no_update_disabled: data.no_update_disabled,
                allow_diagonal: false,
                delay_landing_config: true,
                last_build: None,
            },
            outputs_liquid: data.outputs_liquid,
            can_overdrive: data.can_overdrive,
            liquid_capacity: data.base.liquid_capacity,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry.get_liquid_by_name(name).and_then(Self::from_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::blocks::distribution::DirectionLiquidBridge;

    #[test]
    fn liquid_block_family_defaults_follow_registry_shape() {
        let block = LiquidBlock::new("liquid-router");
        assert!(block.block.update);
        assert!(block.block.solid);
        assert!(block.block.has_liquids);
        assert_eq!(block.block.group, BlockGroup::Liquids);
        assert!(block.outputs_liquid);
        assert!(!block.under_bullets);
        assert!(block.can_overdrive);
    }

    #[test]
    fn conduit_and_router_read_registry_data() {
        let mut registry = BlockRegistry::new();
        registry.register_liquid_block("conduit", LiquidBlockKind::Conduit, |liquid| {
            liquid.pad_corners = true;
            liquid.leaks = true;
            liquid.liquid_pressure = 1.0;
        });
        registry.register_liquid_block("liquid-router", LiquidBlockKind::LiquidRouter, |liquid| {
            liquid.liquid_padding = 2.5;
            liquid.can_overdrive = false;
        });
        registry.register_liquid_block("liquid-bridge", LiquidBlockKind::LiquidBridge, |liquid| {
            liquid.range = 4.0;
            liquid.bridge_width = 6.5;
        });
        registry.register_liquid_block(
            "duct-liquid-bridge",
            LiquidBlockKind::DirectionLiquidBridge,
            |liquid| {
                liquid.speed = 5.0;
                liquid.liquid_padding = 1.0;
            },
        );

        let conduit = Conduit::from_registry(&registry, "conduit").unwrap();
        assert!(conduit.pad_corners);
        assert!(conduit.leaks);

        let router = LiquidRouter::from_registry(&registry, "liquid-router").unwrap();
        assert_eq!(router.liquid_padding, 2.5);
        assert!(!router.liquid_block.can_overdrive);

        let bridge = LiquidBridge::from_registry(&registry, "liquid-bridge").unwrap();
        assert_eq!(bridge.bridge.range, 4);
        assert_eq!(bridge.liquid_capacity, 100.0);

        let directional = DirectionLiquidBridge::new("duct-liquid-bridge");
        assert_eq!(directional.speed, 5.0);
        assert_eq!(directional.liquid_padding, 1.0);
        assert_eq!(directional.region_rotated1, 2);
        assert_eq!(directional.bridge.block.group, BlockGroup::Liquids);
    }

    #[test]
    fn armoured_conduit_and_junction_wrap_base_liquid_blocks() {
        let armored = ArmoredConduit::new("plated-conduit");
        assert!(armored.conduit.armored);
        assert!(!armored.conduit.leaks);

        let junction = LiquidJunction::new("liquid-junction");
        assert!(junction.liquid_block.floating);

        let dir = DirectionLiquidBridge::new("bridge-conduit");
        assert!(dir.outputs_liquid);
        assert!(!dir.can_overdrive);
        assert_eq!(dir.liquid_capacity, 20.0);
        assert_eq!(dir.region_rotated1, 2);
    }
}
