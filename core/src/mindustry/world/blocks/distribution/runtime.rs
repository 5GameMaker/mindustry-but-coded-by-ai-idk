use crate::mindustry::{
    content::blocks::{
        BlockRegistry, DistributionBlockData, DistributionBlockKind, LiquidBlockData,
        LiquidBlockKind,
    },
    world::meta::{BlockGroup, Env},
    world::{Block, BlockId},
};

fn base_block(name: impl Into<String>) -> Block {
    Block::new(0, name)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conveyor {
    pub block: Block,
    pub speed: f32,
    pub displayed_speed: f32,
    pub push_units: bool,
    pub conveyor_placement: bool,
    pub under_bullets: bool,
    pub unloadable: bool,
    pub no_update_disabled: bool,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
    pub no_side_blend: bool,
    pub junction_replacement: Option<BlockId>,
    pub bridge_replacement: Option<BlockId>,
}

impl Conveyor {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.rotate = true;
        block.update = true;
        block.group = BlockGroup::Transportation;
        block.has_items = true;
        block.item_capacity = 3;
        block.priority = 1;
        Self {
            block,
            speed: 0.0,
            displayed_speed: 0.0,
            push_units: true,
            conveyor_placement: true,
            under_bullets: true,
            unloadable: false,
            no_update_disabled: false,
            ambient_sound: "loopConveyor".into(),
            ambient_sound_volume: 0.0022,
            no_side_blend: false,
            junction_replacement: None,
            bridge_replacement: None,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::Conveyor)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            displayed_speed: data.displayed_speed,
            push_units: true,
            conveyor_placement: true,
            under_bullets: data.under_bullets,
            unloadable: data.unloadable,
            no_update_disabled: data.no_update_disabled,
            ambient_sound: data.ambient_sound.clone(),
            ambient_sound_volume: data.ambient_sound_volume,
            no_side_blend: data.no_side_blend,
            junction_replacement: None,
            bridge_replacement: None,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArmoredConveyor {
    pub conveyor: Conveyor,
}

impl ArmoredConveyor {
    pub fn new(name: impl Into<String>) -> Self {
        let mut conveyor = Conveyor::new(name);
        conveyor.no_side_blend = true;
        Self { conveyor }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::ArmoredConveyor)).then(|| Self {
            conveyor: Conveyor {
                block: data.base.clone(),
                speed: data.speed,
                displayed_speed: data.displayed_speed,
                push_units: true,
                conveyor_placement: true,
                under_bullets: data.under_bullets,
                unloadable: data.unloadable,
                no_update_disabled: data.no_update_disabled,
                ambient_sound: data.ambient_sound.clone(),
                ambient_sound_volume: data.ambient_sound_volume,
                no_side_blend: true,
                junction_replacement: None,
                bridge_replacement: None,
            },
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackConveyor {
    pub block: Block,
    pub glow_alpha: f32,
    pub glow_color_rgba: u32,
    pub speed: f32,
    pub displayed_speed: f32,
    pub base_efficiency: f32,
    pub output_router: bool,
    pub recharge: f32,
    pub under_bullets: bool,
    pub conveyor_placement: bool,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
}

impl StackConveyor {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.rotate = true;
        block.update = true;
        block.group = BlockGroup::Transportation;
        block.has_items = true;
        block.item_capacity = 10;
        block.priority = 1;
        Self {
            block,
            glow_alpha: 1.0,
            glow_color_rgba: 0xfeb380ff,
            speed: 0.0,
            displayed_speed: 0.0,
            base_efficiency: 0.0,
            output_router: true,
            recharge: 2.0,
            under_bullets: true,
            conveyor_placement: true,
            ambient_sound: "loopConveyor".into(),
            ambient_sound_volume: 0.004,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::StackConveyor)).then(|| Self {
            block: data.base.clone(),
            glow_alpha: 1.0,
            glow_color_rgba: 0xfeb380ff,
            speed: data.speed,
            displayed_speed: data.displayed_speed,
            base_efficiency: data.base_efficiency,
            output_router: data.output_router,
            recharge: data.recharge,
            under_bullets: data.under_bullets,
            conveyor_placement: true,
            ambient_sound: data.ambient_sound.clone(),
            ambient_sound_volume: data.ambient_sound_volume,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Junction {
    pub block: Block,
    pub speed: f32,
    pub capacity: i32,
    pub displayed_speed: f32,
    pub under_bullets: bool,
    pub unloadable: bool,
    pub no_update_disabled: bool,
}

impl Junction {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = false;
        block.group = BlockGroup::Transportation;
        block.priority = 0;
        Self {
            block,
            speed: 26.0,
            capacity: 6,
            displayed_speed: 13.0,
            under_bullets: true,
            unloadable: false,
            no_update_disabled: true,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::Junction)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            capacity: data.capacity,
            displayed_speed: data.displayed_speed,
            under_bullets: data.under_bullets,
            unloadable: data.unloadable,
            no_update_disabled: data.no_update_disabled,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemBridge {
    pub block: Block,
    pub range: i32,
    pub transport_time: f32,
    pub fade_in: bool,
    pub move_arrows: bool,
    pub pulse: bool,
    pub arrow_spacing: f32,
    pub arrow_offset: f32,
    pub arrow_period: f32,
    pub arrow_time_scl: f32,
    pub bridge_width: f32,
    pub under_bullets: bool,
    pub no_update_disabled: bool,
    pub allow_diagonal: bool,
    pub delay_landing_config: bool,
    pub last_build: Option<i32>,
}

impl ItemBridge {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = true;
        block.has_power = true;
        block.item_capacity = 10;
        block.configurable = true;
        block.has_items = true;
        block.group = BlockGroup::Transportation;
        block.copy_config = false;
        block.allow_config_inventory = false;
        block.ignore_resize_config = true;
        Self {
            block,
            range: 4,
            transport_time: 0.0,
            fade_in: true,
            move_arrows: true,
            pulse: false,
            arrow_spacing: 4.0,
            arrow_offset: 2.0,
            arrow_period: 0.4,
            arrow_time_scl: 6.2,
            bridge_width: 6.5,
            under_bullets: true,
            no_update_disabled: true,
            allow_diagonal: false,
            delay_landing_config: true,
            last_build: None,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::ItemBridge)).then(|| Self {
            block: data.base.clone(),
            range: data.range as i32,
            transport_time: data.transport_time,
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
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BufferedItemBridge {
    pub bridge: ItemBridge,
    pub speed: f32,
    pub buffer_capacity: i32,
    pub displayed_speed: f32,
}

impl BufferedItemBridge {
    pub fn new(name: impl Into<String>) -> Self {
        let mut bridge = ItemBridge::new(name);
        bridge.block.has_power = false;
        Self {
            bridge,
            speed: 40.0,
            buffer_capacity: 50,
            displayed_speed: 11.0,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::BufferedItemBridge)).then(|| Self {
            bridge: ItemBridge {
                block: data.base.clone(),
                range: data.range as i32,
                transport_time: data.transport_time,
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
            speed: data.speed,
            buffer_capacity: data.buffer_capacity,
            displayed_speed: data.displayed_speed,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sorter {
    pub block: Block,
    pub invert: bool,
}

impl Sorter {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = false;
        block.destructible = true;
        block.group = BlockGroup::Transportation;
        block.configurable = true;
        block.save_config = true;
        block.clear_on_double_tap = true;
        Self {
            block,
            invert: false,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::Sorter)).then(|| Self {
            block: data.base.clone(),
            invert: data.invert,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Router {
    pub block: Block,
    pub speed: f32,
    pub under_bullets: bool,
    pub no_update_disabled: bool,
}

impl Router {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = false;
        block.has_items = true;
        block.item_capacity = 1;
        block.group = BlockGroup::Transportation;
        Self {
            block,
            speed: 8.0,
            under_bullets: true,
            no_update_disabled: true,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::Router)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            under_bullets: data.under_bullets,
            no_update_disabled: data.no_update_disabled,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverflowGate {
    pub block: Block,
    pub speed: f32,
    pub invert: bool,
    pub under_bullets: bool,
    pub instant_transfer: bool,
}

impl OverflowGate {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.has_items = true;
        block.update = false;
        block.destructible = true;
        block.group = BlockGroup::Transportation;
        block.item_capacity = 0;
        Self {
            block,
            speed: 1.0,
            invert: false,
            under_bullets: true,
            instant_transfer: true,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::OverflowGate)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            invert: data.invert,
            under_bullets: data.under_bullets,
            instant_transfer: data.instant_transfer,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectionalUnloader {
    pub block: Block,
    pub speed: f32,
    pub allow_core_unload: bool,
    pub under_bullets: bool,
    pub no_update_disabled: bool,
    pub is_duct: bool,
}

impl DirectionalUnloader {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = true;
        block.has_items = true;
        block.configurable = true;
        block.save_config = true;
        block.rotate = true;
        block.item_capacity = 0;
        block.group = BlockGroup::Transportation;
        block.priority = 1;
        Self {
            block,
            speed: 1.0,
            allow_core_unload: false,
            under_bullets: true,
            no_update_disabled: true,
            is_duct: true,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::DirectionalUnloader)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            allow_core_unload: data.allow_core_unload,
            under_bullets: data.under_bullets,
            no_update_disabled: data.no_update_disabled,
            is_duct: data.is_duct,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectionBridge {
    pub block: Block,
    pub range: i32,
    pub draw_arrow: bool,
    pub allow_diagonal: bool,
    pub region_rotated1: i32,
}

impl DirectionBridge {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = true;
        block.rotate = true;
        block.group = BlockGroup::Transportation;
        block.priority = 1;
        Self {
            block,
            range: 4,
            draw_arrow: false,
            allow_diagonal: false,
            region_rotated1: 1,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::DuctBridge)).then(|| Self {
            block: data.base.clone(),
            range: data.range as i32,
            draw_arrow: false,
            allow_diagonal: false,
            region_rotated1: data.region_rotated1,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Duct {
    pub block: Block,
    pub speed: f32,
    pub armored: bool,
    pub transparent_color_rgba: u32,
    pub no_side_blend: bool,
    pub is_duct: bool,
    pub under_bullets: bool,
    pub conveyor_placement: bool,
    pub bridge_replacement: Option<BlockId>,
    pub junction_replacement: Option<BlockId>,
}

impl Duct {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = false;
        block.has_items = true;
        block.item_capacity = 1;
        block.rotate = true;
        block.group = BlockGroup::Transportation;
        block.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
        block.priority = 1;
        Self {
            block,
            speed: 5.0,
            armored: false,
            transparent_color_rgba: 0x6666661a,
            no_side_blend: true,
            is_duct: true,
            under_bullets: true,
            conveyor_placement: true,
            bridge_replacement: None,
            junction_replacement: None,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::Duct)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            armored: data.armored,
            transparent_color_rgba: 0x6666661a,
            no_side_blend: data.no_side_blend,
            is_duct: data.is_duct,
            under_bullets: data.under_bullets,
            conveyor_placement: true,
            bridge_replacement: None,
            junction_replacement: None,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DuctJunction {
    pub block: Block,
    pub transparent_color_rgba: u32,
    pub speed: f32,
    pub capacity: i32,
    pub displayed_speed: f32,
    pub under_bullets: bool,
    pub no_update_disabled: bool,
}

impl DuctJunction {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = false;
        block.has_items = true;
        block.item_capacity = 4;
        block.floating = true;
        block.group = BlockGroup::Transportation;
        block.priority = 1;
        block.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
        Self {
            block,
            transparent_color_rgba: 0x6666661a,
            speed: 5.0,
            capacity: 4,
            displayed_speed: 13.0,
            under_bullets: true,
            no_update_disabled: true,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::Junction)).then(|| Self {
            block: data.base.clone(),
            transparent_color_rgba: 0x6666661a,
            speed: data.speed,
            capacity: data.capacity,
            displayed_speed: data.displayed_speed,
            under_bullets: data.under_bullets,
            no_update_disabled: data.no_update_disabled,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DuctRouter {
    pub block: Block,
    pub speed: f32,
    pub under_bullets: bool,
    pub configurable: bool,
    pub save_config: bool,
    pub clear_on_double_tap: bool,
    pub no_update_disabled: bool,
}

impl DuctRouter {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = false;
        block.has_items = true;
        block.item_capacity = 1;
        block.group = BlockGroup::Transportation;
        block.configurable = true;
        block.save_config = true;
        block.rotate = true;
        block.clear_on_double_tap = true;
        block.priority = 1;
        block.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
        Self {
            block,
            speed: 5.0,
            under_bullets: true,
            configurable: true,
            save_config: true,
            clear_on_double_tap: true,
            no_update_disabled: true,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::DuctRouter)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            under_bullets: data.under_bullets,
            configurable: data.configurable,
            save_config: data.save_config,
            clear_on_double_tap: data.clear_on_double_tap,
            no_update_disabled: data.no_update_disabled,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverflowDuct {
    pub block: Block,
    pub speed: f32,
    pub invert: bool,
    pub under_bullets: bool,
    pub no_update_disabled: bool,
    pub region_rotated1: i32,
}

impl OverflowDuct {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = false;
        block.has_items = true;
        block.item_capacity = 1;
        block.rotate = true;
        block.group = BlockGroup::Transportation;
        block.priority = 1;
        block.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
        Self {
            block,
            speed: 5.0,
            invert: false,
            under_bullets: true,
            no_update_disabled: true,
            region_rotated1: 1,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::OverflowDuct)).then(|| Self {
            block: data.base.clone(),
            speed: data.speed,
            invert: data.invert,
            under_bullets: data.under_bullets,
            no_update_disabled: data.no_update_disabled,
            region_rotated1: data.region_rotated1,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DuctBridge {
    pub bridge: DirectionBridge,
    pub speed: f32,
    pub under_bullets: bool,
    pub is_duct: bool,
    pub item_capacity: i32,
}

impl DuctBridge {
    pub fn new(name: impl Into<String>) -> Self {
        let mut bridge = DirectionBridge::new(name);
        bridge.block.solid = true;
        bridge.block.has_items = true;
        bridge.block.item_capacity = 4;
        Self {
            bridge,
            speed: 5.0,
            under_bullets: true,
            is_duct: true,
            item_capacity: 4,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::DuctBridge)).then(|| Self {
            bridge: DirectionBridge {
                block: data.base.clone(),
                range: data.range as i32,
                draw_arrow: false,
                allow_diagonal: false,
                region_rotated1: data.region_rotated1,
            },
            speed: data.speed,
            under_bullets: data.under_bullets,
            is_duct: data.is_duct,
            item_capacity: data.base.item_capacity,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackRouter {
    pub router: DuctRouter,
    pub base_efficiency: f32,
    pub glow_alpha: f32,
    pub glow_color_rgba: u32,
}

impl StackRouter {
    pub fn new(name: impl Into<String>) -> Self {
        let mut router = DuctRouter::new(name);
        router.block.item_capacity = 10;
        Self {
            router,
            base_efficiency: 0.0,
            glow_alpha: 1.0,
            glow_color_rgba: 0xfeb380ff,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::StackRouter)).then(|| Self {
            router: DuctRouter {
                block: data.base.clone(),
                speed: data.speed,
                under_bullets: data.under_bullets,
                configurable: data.configurable,
                save_config: data.save_config,
                clear_on_double_tap: data.clear_on_double_tap,
                no_update_disabled: data.no_update_disabled,
            },
            base_efficiency: data.base_efficiency,
            glow_alpha: 1.0,
            glow_color_rgba: 0xfeb380ff,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MassDriver {
    pub block: Block,
    pub range: f32,
    pub rotate_speed: f32,
    pub translation: f32,
    pub min_distribute: i32,
    pub knockback: f32,
    pub reload: f32,
    pub bullet_speed: f32,
    pub bullet_lifetime: f32,
    pub shoot_sound_volume: f32,
    pub shake: f32,
}

impl MassDriver {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = base_block(name);
        block.update = true;
        block.solid = true;
        block.configurable = true;
        block.has_items = true;
        block.has_power = true;
        block.sync = true;
        block.env_enabled |= Env::SPACE;
        block.rotate = true;
        Self {
            block,
            range: 0.0,
            rotate_speed: 5.0,
            translation: 7.0,
            min_distribute: 10,
            knockback: 4.0,
            reload: 100.0,
            bullet_speed: 5.5,
            bullet_lifetime: 200.0,
            shoot_sound_volume: 0.5,
            shake: 3.0,
        }
    }

    pub fn from_data(data: &DistributionBlockData) -> Option<Self> {
        (matches!(data.kind, DistributionBlockKind::MassDriver)).then(|| Self {
            block: data.base.clone(),
            range: data.range,
            rotate_speed: data.rotate_speed,
            translation: data.translation,
            min_distribute: data.min_distribute,
            knockback: data.knockback,
            reload: data.reload,
            bullet_speed: data.bullet_speed,
            bullet_lifetime: data.bullet_lifetime,
            shoot_sound_volume: data.shoot_sound_volume,
            shake: data.shake,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry
            .get_distribution_by_name(name)
            .and_then(Self::from_data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectionLiquidBridge {
    pub bridge: DirectionBridge,
    pub speed: f32,
    pub liquid_padding: f32,
    pub outputs_liquid: bool,
    pub can_overdrive: bool,
    pub liquid_capacity: f32,
    pub region_rotated1: i32,
}

impl DirectionLiquidBridge {
    pub fn new(name: impl Into<String>) -> Self {
        let mut bridge = DirectionBridge::new(name);
        bridge.block.group = BlockGroup::Liquids;
        bridge.block.has_liquids = true;
        bridge.block.liquid_capacity = 20.0;
        bridge.region_rotated1 = 2;
        Self {
            bridge,
            speed: 5.0,
            liquid_padding: 1.0,
            outputs_liquid: true,
            can_overdrive: false,
            liquid_capacity: 20.0,
            region_rotated1: 2,
        }
    }

    pub fn from_data(data: &LiquidBlockData) -> Option<Self> {
        (matches!(data.kind, LiquidBlockKind::DirectionLiquidBridge)).then(|| Self {
            bridge: DirectionBridge {
                block: data.base.clone(),
                range: data.range as i32,
                draw_arrow: false,
                allow_diagonal: false,
                region_rotated1: data.region_rotated1,
            },
            speed: data.speed,
            liquid_padding: data.liquid_padding,
            outputs_liquid: data.outputs_liquid,
            can_overdrive: data.can_overdrive,
            liquid_capacity: data.base.liquid_capacity,
            region_rotated1: data.region_rotated1,
        })
    }

    pub fn from_registry(registry: &BlockRegistry, name: &str) -> Option<Self> {
        registry.get_liquid_by_name(name).and_then(Self::from_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conveyor_family_defaults_and_registry_roundtrip_follow_upstream_shapes() {
        let conveyor = Conveyor::new("conveyor");
        assert!(conveyor.block.rotate);
        assert!(conveyor.block.update);
        assert_eq!(conveyor.block.group, BlockGroup::Transportation);
        assert_eq!(conveyor.block.item_capacity, 3);
        assert!(conveyor.conveyor_placement);
        assert!(conveyor.under_bullets);
        assert!(!conveyor.unloadable);

        let armored = ArmoredConveyor::new("armored-conveyor");
        assert!(armored.conveyor.no_side_blend);

        let stack = StackConveyor::new("stack-conveyor");
        assert_eq!(stack.glow_alpha, 1.0);
        assert_eq!(stack.glow_color_rgba, 0xfeb380ff);

        let mut registry = BlockRegistry::new();
        registry.register_distribution_block(
            "titanium-conveyor",
            DistributionBlockKind::Conveyor,
            |distribution| {
                distribution.speed = 0.08;
                distribution.displayed_speed = 11.0;
                distribution.no_side_blend = false;
            },
        );
        let from_registry = Conveyor::from_registry(&registry, "titanium-conveyor").unwrap();
        assert_eq!(from_registry.speed, 0.08);
        assert_eq!(from_registry.displayed_speed, 11.0);
    }

    #[test]
    fn bridge_router_and_duct_shells_read_registry_data() {
        let mut registry = BlockRegistry::new();
        registry.register_distribution_block(
            "phase-conveyor",
            DistributionBlockKind::ItemBridge,
            |distribution| {
                distribution.range = 12.0;
                distribution.transport_time = 2.0;
                distribution.pulse = true;
            },
        );
        registry.register_distribution_block(
            "duct-router",
            DistributionBlockKind::DuctRouter,
            |distribution| {
                distribution.speed = 4.0;
                distribution.clear_on_double_tap = true;
            },
        );
        registry.register_distribution_block(
            "duct-bridge",
            DistributionBlockKind::DuctBridge,
            |distribution| {
                distribution.range = 4.0;
                distribution.speed = 4.0;
            },
        );
        registry.register_distribution_block(
            "surge-router",
            DistributionBlockKind::StackRouter,
            |distribution| {
                distribution.base_efficiency = 1.0;
            },
        );
        registry.register_distribution_block(
            "duct-unloader",
            DistributionBlockKind::DirectionalUnloader,
            |distribution| {
                distribution.allow_core_unload = false;
                distribution.speed = 4.0;
            },
        );

        let bridge = ItemBridge::from_registry(&registry, "phase-conveyor").unwrap();
        assert_eq!(bridge.range, 12);
        assert_eq!(bridge.transport_time, 2.0);
        assert!(bridge.pulse);

        let duct_router = DuctRouter::from_registry(&registry, "duct-router").unwrap();
        assert_eq!(duct_router.speed, 4.0);
        assert!(duct_router.clear_on_double_tap);

        let duct_bridge = DuctBridge::from_registry(&registry, "duct-bridge").unwrap();
        assert_eq!(duct_bridge.bridge.range, 4);

        let stack_router = StackRouter::from_registry(&registry, "surge-router").unwrap();
        assert_eq!(stack_router.base_efficiency, 1.0);

        let unloader = DirectionalUnloader::from_registry(&registry, "duct-unloader").unwrap();
        assert_eq!(unloader.speed, 4.0);
        assert!(!unloader.allow_core_unload);
    }

    #[test]
    fn mass_driver_and_direction_bridge_retain_base_block_metadata() {
        let driver = MassDriver::new("mass-driver");
        assert!(driver.block.rotate);
        assert!(driver.block.has_power);
        assert_eq!(driver.rotate_speed, 5.0);
        assert_eq!(driver.translation, 7.0);

        let bridge = DirectionBridge::new("duct-bridge");
        assert_eq!(bridge.range, 4);
        assert!(!bridge.draw_arrow);
        assert!(!bridge.allow_diagonal);
        assert_eq!(bridge.region_rotated1, 1);

        let liquid_bridge = DirectionLiquidBridge::new("liquid-duct-bridge");
        assert_eq!(liquid_bridge.region_rotated1, 2);
        assert_eq!(liquid_bridge.bridge.block.group, BlockGroup::Liquids);
    }
}
