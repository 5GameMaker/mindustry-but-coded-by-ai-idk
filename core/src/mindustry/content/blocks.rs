use std::collections::BTreeMap;

use crate::mindustry::{
    ctype::ContentId,
    r#type::{Item, Liquid},
    vars::TILE_SIZE,
    world::{
        blocks::environment::{
            FloorData, OreBlockData, PropData, PropKind, SeaBushData, StaticTreeData,
            StaticWallData, TallBlockData, TreeBlockData,
        },
        meta::{BlockFlag, BlockGroup, BuildVisibility, Env},
        Block, BlockId, CacheLayer,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftingBlockKind {
    GenericCrafter,
    AttributeCrafter,
    Separator,
    HeatCrafter,
    HeatProducer,
    HeatConductor,
    Incinerator,
    ItemIncinerator,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemAmount {
    pub item: ContentId,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidAmount {
    pub liquid: ContentId,
    pub amount: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidConsume {
    pub liquid: ContentId,
    pub amount: f32,
    pub booster: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionBlockKind {
    Drill,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemMultiplier {
    pub item: ContentId,
    pub multiplier: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionBlockData {
    pub base: Block,
    pub kind: ProductionBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub consume_power: f32,
    pub consume_liquids: Vec<LiquidConsume>,
    pub hardness_drill_multiplier: f32,
    pub tier: i32,
    pub drill_time: f32,
    pub liquid_boost_intensity: f32,
    pub warmup_speed: f32,
    pub blocked_items: Vec<ContentId>,
    pub draw_mine_item: bool,
    pub drill_effect: String,
    pub drill_effect_rnd: f32,
    pub drill_effect_chance: f32,
    pub rotate_speed: f32,
    pub update_effect: String,
    pub update_effect_chance: f32,
    pub drill_multipliers: Vec<ItemMultiplier>,
    pub draw_rim: bool,
    pub draw_spin_sprite: bool,
    pub heat_color: String,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
}

impl ProductionBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: ProductionBlockKind) -> Self {
        let base = Block::new(id, name);
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            consume_power: 0.0,
            consume_liquids: Vec::new(),
            hardness_drill_multiplier: 50.0,
            tier: 0,
            drill_time: 300.0,
            liquid_boost_intensity: 1.6,
            warmup_speed: 0.015,
            blocked_items: Vec::new(),
            draw_mine_item: true,
            drill_effect: "mine".into(),
            drill_effect_rnd: -1.0,
            drill_effect_chance: 0.02,
            rotate_speed: 2.0,
            update_effect: "pulverizeSmall".into(),
            update_effect_chance: 0.02,
            drill_multipliers: Vec::new(),
            draw_rim: false,
            draw_spin_sprite: true,
            heat_color: "ff5512".into(),
            ambient_sound: "none".into(),
            ambient_sound_volume: 0.0,
        };
        block.apply_kind_defaults();
        block
    }

    fn apply_kind_defaults(&mut self) {
        match self.kind {
            ProductionBlockKind::Drill => {
                self.base.update = true;
                self.base.solid = true;
                self.base.group = BlockGroup::Drills;
                self.base.has_liquids = true;
                self.base.has_items = true;
                self.base.env_enabled |= Env::SPACE;
                self.base.flags.push(BlockFlag::Drill);
                self.ambient_sound = "loopDrill".into();
                self.ambient_sound_volume = 0.019;
            }
        }
    }

    fn finalize(&mut self) {
        match self.kind {
            ProductionBlockKind::Drill => {
                if self.drill_effect_rnd < 0.0 {
                    self.drill_effect_rnd = self.base.size as f32;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenseWallKind {
    Wall,
    Door,
    AutoDoor,
    ShieldWall,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefenseWallData {
    pub base: Block,
    pub kind: DefenseWallKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub build_cost_multiplier: f32,
    pub armor: f32,
    pub schematic_priority: i32,
    pub chance_deflect: f32,
    pub flash_hit: bool,
    pub lightning_chance: f32,
    pub lightning_damage: f32,
    pub insulated: bool,
    pub absorb_lasers: bool,
    pub solidifies: bool,
    pub consumes_tap: bool,
    pub team_passable: bool,
    pub no_update_disabled: bool,
    pub consume_power: f32,
    pub shield_health: f32,
    pub shield_break_cooldown: f32,
    pub shield_regen_speed: f32,
}

impl DefenseWallData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        let mut base = Block::new(id, name);
        base.solid = true;
        base.destructible = true;
        base.group = BlockGroup::Walls;
        base.unit_move_breakable = false;
        base.cache_layer = CacheLayer::Normal;
        base.env_enabled = Env::ANY;
        Self {
            base,
            kind: DefenseWallKind::Wall,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            build_cost_multiplier: 6.0,
            armor: 0.0,
            schematic_priority: 0,
            chance_deflect: -1.0,
            flash_hit: false,
            lightning_chance: -1.0,
            lightning_damage: 20.0,
            insulated: false,
            absorb_lasers: false,
            solidifies: false,
            consumes_tap: false,
            team_passable: false,
            no_update_disabled: false,
            consume_power: 0.0,
            shield_health: 0.0,
            shield_break_cooldown: 0.0,
            shield_regen_speed: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectBlockKind {
    MendProjector,
    OverdriveProjector,
    ForceProjector,
    ShockMine,
    Radar,
    BuildTurret,
    RegenProjector,
    ShockwaveTower,
    BaseShield,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectBlockData {
    pub base: Block,
    pub kind: EffectBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub consume_power: f32,
    pub consume_items: Vec<ItemAmount>,
    pub boost_items: Vec<ItemAmount>,
    pub consume_liquids: Vec<LiquidAmount>,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
    pub outline_color: String,
    pub fog_radius: f32,
    pub range: f32,
    pub reload: f32,
    pub heal_percent: f32,
    pub phase_boost: f32,
    pub phase_range_boost: f32,
    pub use_time: f32,
    pub speed_boost: f32,
    pub speed_boost_phase: f32,
    pub has_boost: bool,
    pub phase_use_time: f32,
    pub phase_radius_boost: f32,
    pub phase_shield_boost: f32,
    pub radius: f32,
    pub sides: i32,
    pub shield_rotation: f32,
    pub shield_health: f32,
    pub cooldown_normal: f32,
    pub cooldown_liquid: f32,
    pub cooldown_broken_base: f32,
    pub coolant_consumption: f32,
    pub consume_coolant: bool,
    pub damage: f32,
    pub tile_damage: f32,
    pub length: i32,
    pub tendrils: i32,
    pub shots: i32,
    pub team_alpha: f32,
    pub discovery_time: f32,
    pub rotate_speed: f32,
    pub glow_color: String,
    pub glow_scl: f32,
    pub glow_mag: f32,
    pub build_speed: f32,
    pub build_beam_offset: f32,
    pub target_interval: i32,
    pub elevation: f32,
    pub optional_multiplier: f32,
    pub optional_use_time: f32,
    pub effect_chance: f32,
    pub base_color: String,
    pub effect: String,
    pub drawer: String,
    pub rotate_draw: bool,
    pub rebuildable: bool,
    pub bullet_damage: f32,
    pub falloff_count: f32,
    pub shake: f32,
    pub check_interval: f32,
    pub cooldown_multiplier: f32,
    pub shape_rotate_speed: f32,
    pub shape_radius: f32,
    pub shape_sides: i32,
}

impl EffectBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: EffectBlockKind) -> Self {
        let base = Block::new(id, name);
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            consume_power: 0.0,
            consume_items: Vec::new(),
            boost_items: Vec::new(),
            consume_liquids: Vec::new(),
            ambient_sound: "none".into(),
            ambient_sound_volume: 0.0,
            outline_color: String::new(),
            fog_radius: 0.0,
            range: 0.0,
            reload: 0.0,
            heal_percent: 0.0,
            phase_boost: 0.0,
            phase_range_boost: 0.0,
            use_time: 0.0,
            speed_boost: 0.0,
            speed_boost_phase: 0.0,
            has_boost: false,
            phase_use_time: 0.0,
            phase_radius_boost: 0.0,
            phase_shield_boost: 0.0,
            radius: 0.0,
            sides: 0,
            shield_rotation: 0.0,
            shield_health: 0.0,
            cooldown_normal: 0.0,
            cooldown_liquid: 0.0,
            cooldown_broken_base: 0.0,
            coolant_consumption: 0.0,
            consume_coolant: false,
            damage: 0.0,
            tile_damage: 0.0,
            length: 0,
            tendrils: 0,
            shots: 0,
            team_alpha: 0.0,
            discovery_time: 0.0,
            rotate_speed: 0.0,
            glow_color: String::new(),
            glow_scl: 0.0,
            glow_mag: 0.0,
            build_speed: 0.0,
            build_beam_offset: 0.0,
            target_interval: 0,
            elevation: 0.0,
            optional_multiplier: 0.0,
            optional_use_time: 0.0,
            effect_chance: 0.0,
            base_color: String::new(),
            effect: String::new(),
            drawer: String::new(),
            rotate_draw: true,
            rebuildable: true,
            bullet_damage: 0.0,
            falloff_count: 0.0,
            shake: 0.0,
            check_interval: 0.0,
            cooldown_multiplier: 0.0,
            shape_rotate_speed: 0.0,
            shape_radius: 0.0,
            shape_sides: 0,
        };
        block.apply_kind_defaults();
        block
    }

    fn apply_kind_defaults(&mut self) {
        match self.kind {
            EffectBlockKind::MendProjector => {
                self.base.solid = true;
                self.base.update = true;
                self.base.group = BlockGroup::Projectors;
                self.base.has_power = true;
                self.base.has_items = true;
                self.base.emit_light = true;
                self.base.light_radius = 50.0;
                self.base.env_enabled |= Env::SPACE;
                self.base.flags.push(BlockFlag::BlockRepair);
                self.reload = 250.0;
                self.range = 60.0;
                self.heal_percent = 12.0;
                self.phase_boost = 12.0;
                self.phase_range_boost = 50.0;
                self.use_time = 400.0;
            }
            EffectBlockKind::OverdriveProjector => {
                self.base.solid = true;
                self.base.update = true;
                self.base.group = BlockGroup::Projectors;
                self.base.has_power = true;
                self.base.has_items = true;
                self.base.emit_light = true;
                self.base.light_radius = 50.0;
                self.base.env_enabled |= Env::SPACE;
                self.ambient_sound = "loopCircuit".into();
                self.ambient_sound_volume = 0.13;
                self.reload = 60.0;
                self.range = 80.0;
                self.speed_boost = 1.5;
                self.speed_boost_phase = 0.75;
                self.use_time = 400.0;
                self.phase_range_boost = 20.0;
                self.has_boost = true;
            }
            EffectBlockKind::ForceProjector => {
                self.base.update = true;
                self.base.solid = true;
                self.base.group = BlockGroup::Projectors;
                self.base.has_power = true;
                self.base.has_liquids = true;
                self.base.has_items = true;
                self.base.env_enabled |= Env::SPACE;
                self.base.flags.push(BlockFlag::Shield);
                self.ambient_sound = "loopShield".into();
                self.ambient_sound_volume = 0.1;
                self.phase_use_time = 350.0;
                self.phase_radius_boost = 80.0;
                self.phase_shield_boost = 400.0;
                self.radius = 101.7;
                self.sides = 6;
                self.shield_rotation = 0.0;
                self.shield_health = 700.0;
                self.cooldown_normal = 1.75;
                self.cooldown_liquid = 1.5;
                self.cooldown_broken_base = 0.35;
                self.coolant_consumption = 0.1;
                self.consume_coolant = true;
            }
            EffectBlockKind::ShockMine => {
                self.base.update = false;
                self.base.destructible = true;
                self.base.solid = false;
                self.base.targetable = false;
                self.reload = 80.0;
                self.tile_damage = 5.0;
                self.damage = 13.0;
                self.length = 10;
                self.tendrils = 6;
                self.shots = 6;
                self.team_alpha = 0.3;
            }
            EffectBlockKind::Radar => {
                self.base.update = true;
                self.base.solid = true;
                self.base.flags.push(BlockFlag::HasFogRadius);
                self.fog_radius = 10.0;
                self.discovery_time = 60.0 * 10.0;
                self.rotate_speed = 2.0;
                self.glow_color = "turretHeat".into();
                self.glow_scl = 5.0;
                self.glow_mag = 0.6;
            }
            EffectBlockKind::BuildTurret => {
                self.base.update = true;
                self.base.solid = true;
                self.base.sync = false;
                self.base.group = BlockGroup::Turrets;
                self.base.flags.push(BlockFlag::Turret);
                self.range = 80.0;
                self.rotate_speed = 10.0;
                self.build_speed = 1.0;
                self.build_beam_offset = 5.0;
                self.target_interval = 15;
                self.elevation = -1.0;
            }
            EffectBlockKind::RegenProjector => {
                self.base.solid = true;
                self.base.update = true;
                self.base.group = BlockGroup::Projectors;
                self.base.has_power = true;
                self.base.has_items = true;
                self.base.emit_light = true;
                self.base.env_enabled |= Env::SPACE;
                self.base.flags.push(BlockFlag::BlockRepair);
                self.ambient_sound = "loopRegen".into();
                self.ambient_sound_volume = 0.45;
                self.range = 14.0;
                self.heal_percent = 12.0 / 60.0;
                self.optional_multiplier = 2.0;
                self.optional_use_time = 60.0 * 8.0;
                self.effect_chance = 0.003;
                self.base_color = "accent".into();
                self.effect = "regenParticle".into();
                self.drawer = "DrawDefault".into();
                self.rotate_draw = false;
            }
            EffectBlockKind::ShockwaveTower => {
                self.base.update = true;
                self.base.solid = true;
                self.range = 110.0;
                self.reload = 60.0 * 1.5;
                self.bullet_damage = 160.0;
                self.falloff_count = 20.0;
                self.shake = 2.0;
                self.check_interval = 8.0;
                self.cooldown_multiplier = 1.0;
                self.shape_rotate_speed = 1.0;
                self.shape_radius = 6.0;
                self.shape_sides = 4;
            }
            EffectBlockKind::BaseShield => {
                self.base.has_power = true;
                self.base.update = true;
                self.base.solid = true;
                self.rebuildable = false;
                self.radius = 200.0;
                self.sides = 24;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionBlockKind {
    Conveyor,
    StackConveyor,
    ArmoredConveyor,
    Junction,
    BufferedItemBridge,
    ItemBridge,
    Sorter,
    Router,
    OverflowGate,
    Unloader,
    MassDriver,
    Duct,
    DuctRouter,
    OverflowDuct,
    DuctBridge,
    DirectionalUnloader,
    StackRouter,
    UnitCargoLoader,
    UnitCargoUnloadPoint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistributionBlockData {
    pub base: Block,
    pub kind: DistributionBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub build_cost_multiplier: f32,
    pub consume_power: f32,
    pub consume_liquids: Vec<LiquidAmount>,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
    pub rotate: bool,
    pub under_bullets: bool,
    pub unloadable: bool,
    pub no_update_disabled: bool,
    pub configurable: bool,
    pub save_config: bool,
    pub clear_on_double_tap: bool,
    pub instant_transfer: bool,
    pub can_overdrive: bool,
    pub no_side_blend: bool,
    pub armored: bool,
    pub is_duct: bool,
    pub region_rotated1: i32,
    pub output_router: bool,
    pub crush_fragile: bool,
    pub allow_core_unload: bool,
    pub fade_in: bool,
    pub move_arrows: bool,
    pub pulse: bool,
    pub invert: bool,
    pub speed: f32,
    pub displayed_speed: f32,
    pub capacity: i32,
    pub range: f32,
    pub transport_time: f32,
    pub arrow_spacing: f32,
    pub arrow_offset: f32,
    pub arrow_period: f32,
    pub arrow_time_scl: f32,
    pub bridge_width: f32,
    pub buffer_capacity: i32,
    pub recharge: f32,
    pub base_efficiency: f32,
    pub unit_build_time: f32,
    pub stale_time_duration: f32,
    pub poly_stroke: f32,
    pub poly_radius: f32,
    pub poly_sides: i32,
    pub poly_rotate_speed: f32,
    pub poly_color: String,
    pub reload: f32,
    pub rotate_speed: f32,
    pub translation: f32,
    pub min_distribute: i32,
    pub knockback: f32,
    pub bullet_speed: f32,
    pub bullet_lifetime: f32,
    pub shoot_sound_volume: f32,
    pub shake: f32,
}

impl DistributionBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: DistributionBlockKind) -> Self {
        let base = Block::new(id, name);
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            build_cost_multiplier: 1.0,
            consume_power: 0.0,
            consume_liquids: Vec::new(),
            ambient_sound: "none".into(),
            ambient_sound_volume: 0.0,
            rotate: false,
            under_bullets: false,
            unloadable: true,
            no_update_disabled: false,
            configurable: false,
            save_config: false,
            clear_on_double_tap: false,
            instant_transfer: false,
            can_overdrive: false,
            no_side_blend: false,
            armored: false,
            is_duct: false,
            region_rotated1: 0,
            output_router: false,
            crush_fragile: false,
            allow_core_unload: false,
            fade_in: false,
            move_arrows: false,
            pulse: false,
            invert: false,
            speed: 0.0,
            displayed_speed: 0.0,
            capacity: 0,
            range: 0.0,
            transport_time: 0.0,
            arrow_spacing: 0.0,
            arrow_offset: 0.0,
            arrow_period: 0.0,
            arrow_time_scl: 0.0,
            bridge_width: 0.0,
            buffer_capacity: 0,
            recharge: 0.0,
            base_efficiency: 0.0,
            unit_build_time: 0.0,
            stale_time_duration: 0.0,
            poly_stroke: 0.0,
            poly_radius: 0.0,
            poly_sides: 0,
            poly_rotate_speed: 0.0,
            poly_color: String::new(),
            reload: 0.0,
            rotate_speed: 0.0,
            translation: 0.0,
            min_distribute: 0,
            knockback: 0.0,
            bullet_speed: 0.0,
            bullet_lifetime: 0.0,
            shoot_sound_volume: 0.0,
            shake: 0.0,
        };
        block.apply_kind_defaults();
        block
    }

    fn apply_kind_defaults(&mut self) {
        match self.kind {
            DistributionBlockKind::Conveyor | DistributionBlockKind::ArmoredConveyor => {
                self.rotate = true;
                self.base.update = true;
                self.base.group = BlockGroup::Transportation;
                self.base.has_items = true;
                self.base.item_capacity = 3;
                self.base.priority = 1;
                self.under_bullets = true;
                self.ambient_sound = "loopConveyor".into();
                self.ambient_sound_volume = 0.0022;
                self.unloadable = false;
                if self.kind == DistributionBlockKind::ArmoredConveyor {
                    self.no_side_blend = true;
                }
            }
            DistributionBlockKind::StackConveyor => {
                self.rotate = true;
                self.base.update = true;
                self.base.group = BlockGroup::Transportation;
                self.base.has_items = true;
                self.base.item_capacity = 10;
                self.base.priority = 1;
                self.under_bullets = true;
                self.ambient_sound = "loopConveyor".into();
                self.ambient_sound_volume = 0.004;
                self.output_router = true;
                self.recharge = 2.0;
            }
            DistributionBlockKind::Junction => {
                self.base.update = true;
                self.base.solid = false;
                self.under_bullets = true;
                self.base.group = BlockGroup::Transportation;
                self.unloadable = false;
                self.no_update_disabled = true;
                self.speed = 26.0;
                self.capacity = 6;
                self.displayed_speed = 13.0;
            }
            DistributionBlockKind::ItemBridge | DistributionBlockKind::BufferedItemBridge => {
                self.base.update = true;
                self.base.solid = true;
                self.under_bullets = true;
                self.base.has_power = true;
                self.base.item_capacity = 10;
                self.configurable = true;
                self.base.has_items = true;
                self.unloadable = false;
                self.base.group = BlockGroup::Transportation;
                self.no_update_disabled = true;
                self.base.priority = 1;
                self.fade_in = true;
                self.move_arrows = true;
                self.arrow_spacing = 4.0;
                self.arrow_offset = 2.0;
                self.arrow_period = 0.4;
                self.arrow_time_scl = 6.2;
                self.bridge_width = 6.5;
                if self.kind == DistributionBlockKind::BufferedItemBridge {
                    self.base.has_power = false;
                    self.base.has_items = true;
                    self.can_overdrive = true;
                    self.speed = 40.0;
                    self.buffer_capacity = 50;
                    self.displayed_speed = 11.0;
                }
            }
            DistributionBlockKind::Sorter => {
                self.base.update = false;
                self.base.destructible = true;
                self.under_bullets = true;
                self.instant_transfer = true;
                self.base.group = BlockGroup::Transportation;
                self.configurable = true;
                self.unloadable = false;
                self.save_config = true;
                self.clear_on_double_tap = true;
            }
            DistributionBlockKind::Router => {
                self.base.solid = false;
                self.under_bullets = true;
                self.base.update = true;
                self.base.has_items = true;
                self.base.item_capacity = 1;
                self.base.group = BlockGroup::Transportation;
                self.unloadable = false;
                self.no_update_disabled = true;
                self.speed = 8.0;
            }
            DistributionBlockKind::OverflowGate => {
                self.base.has_items = true;
                self.under_bullets = true;
                self.base.update = false;
                self.base.destructible = true;
                self.base.group = BlockGroup::Transportation;
                self.instant_transfer = true;
                self.unloadable = false;
                self.can_overdrive = false;
                self.base.item_capacity = 0;
                self.speed = 1.0;
            }
            DistributionBlockKind::Unloader => {
                self.base.update = true;
                self.base.solid = true;
                self.base.health = 70;
                self.base.has_items = true;
                self.configurable = true;
                self.save_config = true;
                self.base.item_capacity = 0;
                self.no_update_disabled = true;
                self.clear_on_double_tap = true;
                self.unloadable = false;
                self.speed = 1.0;
                self.allow_core_unload = true;
            }
            DistributionBlockKind::MassDriver => {
                self.base.update = true;
                self.base.solid = true;
                self.configurable = true;
                self.base.has_items = true;
                self.base.has_power = true;
                self.base.sync = true;
                self.base.env_enabled |= Env::SPACE;
                self.rotate_speed = 5.0;
                self.translation = 7.0;
                self.min_distribute = 10;
                self.knockback = 4.0;
                self.reload = 100.0;
                self.bullet_speed = 5.5;
                self.bullet_lifetime = 200.0;
                self.shoot_sound_volume = 0.5;
                self.shake = 3.0;
            }
            DistributionBlockKind::Duct => {
                self.base.group = BlockGroup::Transportation;
                self.base.update = true;
                self.base.solid = false;
                self.base.has_items = true;
                self.unloadable = false;
                self.base.item_capacity = 1;
                self.no_update_disabled = true;
                self.under_bullets = true;
                self.rotate = true;
                self.no_side_blend = true;
                self.is_duct = true;
                self.base.priority = 1;
                self.base.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
                self.speed = 5.0;
            }
            DistributionBlockKind::DuctRouter => {
                self.base.group = BlockGroup::Transportation;
                self.base.update = true;
                self.base.solid = false;
                self.base.has_items = true;
                self.unloadable = false;
                self.base.item_capacity = 1;
                self.no_update_disabled = true;
                self.configurable = true;
                self.save_config = true;
                self.rotate = true;
                self.clear_on_double_tap = true;
                self.under_bullets = true;
                self.base.priority = 1;
                self.base.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
                self.speed = 5.0;
            }
            DistributionBlockKind::OverflowDuct => {
                self.base.group = BlockGroup::Transportation;
                self.base.update = true;
                self.base.solid = false;
                self.base.has_items = true;
                self.unloadable = false;
                self.base.item_capacity = 1;
                self.no_update_disabled = true;
                self.rotate = true;
                self.under_bullets = true;
                self.base.priority = 1;
                self.base.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
                self.region_rotated1 = 1;
                self.speed = 5.0;
            }
            DistributionBlockKind::DuctBridge => {
                self.base.update = true;
                self.base.solid = true;
                self.rotate = true;
                self.base.group = BlockGroup::Transportation;
                self.no_update_disabled = true;
                self.base.priority = 1;
                self.base.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
                self.region_rotated1 = 1;
                self.range = 4.0;
                self.base.item_capacity = 4;
                self.base.has_items = true;
                self.under_bullets = true;
                self.is_duct = true;
                self.speed = 5.0;
            }
            DistributionBlockKind::DirectionalUnloader => {
                self.base.group = BlockGroup::Transportation;
                self.base.update = true;
                self.base.solid = true;
                self.base.has_items = true;
                self.configurable = true;
                self.save_config = true;
                self.rotate = true;
                self.base.item_capacity = 0;
                self.no_update_disabled = true;
                self.unloadable = false;
                self.is_duct = true;
                self.clear_on_double_tap = true;
                self.base.priority = 1;
                self.speed = 1.0;
                self.allow_core_unload = false;
            }
            DistributionBlockKind::StackRouter => {
                self.base.group = BlockGroup::Transportation;
                self.base.update = true;
                self.base.solid = false;
                self.base.has_items = true;
                self.unloadable = false;
                self.base.item_capacity = 10;
                self.no_update_disabled = true;
                self.configurable = true;
                self.save_config = true;
                self.rotate = true;
                self.clear_on_double_tap = true;
                self.under_bullets = true;
                self.base.priority = 1;
                self.base.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
                self.speed = 5.0;
            }
            DistributionBlockKind::UnitCargoLoader => {
                self.base.solid = true;
                self.base.update = true;
                self.base.has_items = true;
                self.base.item_capacity = 200;
                self.ambient_sound = "loopUnitBuilding".into();
                self.unit_build_time = 60.0 * 8.0;
                self.poly_stroke = 1.8;
                self.poly_radius = 8.0;
                self.poly_sides = 6;
                self.poly_rotate_speed = 1.0;
                self.poly_color = "accent".into();
            }
            DistributionBlockKind::UnitCargoUnloadPoint => {
                self.base.update = true;
                self.base.solid = true;
                self.base.has_items = true;
                self.configurable = true;
                self.save_config = true;
                self.clear_on_double_tap = true;
                self.base.flags.push(BlockFlag::UnitCargoUnloadPoint);
                self.stale_time_duration = 60.0 * 6.0;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidBlockKind {
    Pump,
    Conduit,
    ArmoredConduit,
    LiquidRouter,
    LiquidJunction,
    LiquidBridge,
    DirectionLiquidBridge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidBlockData {
    pub base: Block,
    pub kind: LiquidBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub build_cost_multiplier: f32,
    pub consume_power: f32,
    pub consume_liquids: Vec<LiquidAmount>,
    pub outputs_liquid: bool,
    pub floating: bool,
    pub under_bullets: bool,
    pub rotate: bool,
    pub no_update_disabled: bool,
    pub can_overdrive: bool,
    pub configurable: bool,
    pub fade_in: bool,
    pub move_arrows: bool,
    pub pulse: bool,
    pub leaks: bool,
    pub pad_corners: bool,
    pub bot_color: String,
    pub pump_amount: f32,
    pub consume_time: f32,
    pub warmup_speed: f32,
    pub liquid_pressure: f32,
    pub liquid_padding: f32,
    pub explosiveness_scale: f32,
    pub flammability_scale: f32,
    pub range: f32,
    pub speed: f32,
    pub arrow_spacing: f32,
    pub arrow_offset: f32,
    pub arrow_period: f32,
    pub arrow_time_scl: f32,
    pub bridge_width: f32,
    pub region_rotated1: i32,
    pub junction_replacement: Option<BlockId>,
    pub rot_bridge_replacement: Option<BlockId>,
    pub drawer: String,
}

impl LiquidBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: LiquidBlockKind) -> Self {
        let base = Block::new(id, name);
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            build_cost_multiplier: 1.0,
            consume_power: 0.0,
            consume_liquids: Vec::new(),
            outputs_liquid: false,
            floating: false,
            under_bullets: false,
            rotate: false,
            no_update_disabled: false,
            can_overdrive: true,
            configurable: false,
            fade_in: false,
            move_arrows: false,
            pulse: false,
            leaks: false,
            pad_corners: false,
            bot_color: String::new(),
            pump_amount: 0.0,
            consume_time: 0.0,
            warmup_speed: 0.0,
            liquid_pressure: 1.0,
            liquid_padding: 0.0,
            explosiveness_scale: 1.0,
            flammability_scale: 1.0,
            range: 0.0,
            speed: 0.0,
            arrow_spacing: 0.0,
            arrow_offset: 0.0,
            arrow_period: 0.0,
            arrow_time_scl: 0.0,
            bridge_width: 0.0,
            region_rotated1: 0,
            junction_replacement: None,
            rot_bridge_replacement: None,
            drawer: String::new(),
        };
        block.apply_kind_defaults();
        block
    }

    fn apply_liquid_block_defaults(&mut self) {
        self.base.update = true;
        self.base.solid = true;
        self.base.has_liquids = true;
        self.base.group = BlockGroup::Liquids;
        self.outputs_liquid = true;
        self.base.env_enabled |= Env::SPACE | Env::UNDERWATER;
    }

    fn apply_item_bridge_defaults(&mut self) {
        self.base.update = true;
        self.base.solid = true;
        self.under_bullets = true;
        self.base.has_power = true;
        self.base.item_capacity = 10;
        self.configurable = true;
        self.no_update_disabled = true;
        self.base.priority = 1;
        self.fade_in = true;
        self.move_arrows = true;
        self.arrow_spacing = 4.0;
        self.arrow_offset = 2.0;
        self.arrow_period = 0.4;
        self.arrow_time_scl = 6.2;
        self.bridge_width = 6.5;
    }

    fn apply_direction_bridge_defaults(&mut self) {
        self.base.update = true;
        self.base.solid = true;
        self.rotate = true;
        self.base.group = BlockGroup::Transportation;
        self.no_update_disabled = true;
        self.base.priority = 1;
        self.base.env_enabled = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;
        self.region_rotated1 = 1;
        self.range = 4.0;
    }

    fn apply_kind_defaults(&mut self) {
        match self.kind {
            LiquidBlockKind::Pump => {
                self.apply_liquid_block_defaults();
                self.base.group = BlockGroup::Liquids;
                self.floating = true;
                self.base.env_enabled = Env::TERRESTRIAL;
                self.pump_amount = 0.2;
                self.consume_time = 60.0 * 5.0;
                self.warmup_speed = 0.019;
                self.drawer = "DrawMulti(DrawDefault, DrawPumpLiquid)".into();
            }
            LiquidBlockKind::Conduit | LiquidBlockKind::ArmoredConduit => {
                self.apply_liquid_block_defaults();
                self.rotate = true;
                self.base.solid = false;
                self.floating = true;
                self.under_bullets = true;
                self.no_update_disabled = true;
                self.can_overdrive = false;
                self.base.priority = 1;
                self.bot_color = "565656".into();
                self.pad_corners = true;
                self.leaks = self.kind == LiquidBlockKind::Conduit;
            }
            LiquidBlockKind::LiquidRouter => {
                self.apply_liquid_block_defaults();
                self.base.solid = true;
                self.no_update_disabled = true;
                self.can_overdrive = false;
                self.floating = true;
            }
            LiquidBlockKind::LiquidJunction => {
                self.apply_liquid_block_defaults();
                self.floating = true;
            }
            LiquidBlockKind::LiquidBridge => {
                self.apply_item_bridge_defaults();
                self.base.has_items = false;
                self.base.has_liquids = true;
                self.outputs_liquid = true;
                self.can_overdrive = false;
                self.base.group = BlockGroup::Liquids;
                self.base.env_enabled = Env::ANY;
            }
            LiquidBlockKind::DirectionLiquidBridge => {
                self.apply_direction_bridge_defaults();
                self.outputs_liquid = true;
                self.base.group = BlockGroup::Liquids;
                self.can_overdrive = false;
                self.base.liquid_capacity = 20.0;
                self.base.has_liquids = true;
                self.speed = 5.0;
                self.liquid_padding = 1.0;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerBlockKind {
    PowerNode,
    PowerDiode,
    Battery,
    ConsumeGenerator,
    ThermalGenerator,
    SolarGenerator,
    NuclearReactor,
    ImpactReactor,
    BeamNode,
    LongPowerNode,
    VariableReactor,
    HeaterGenerator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PowerBlockData {
    pub base: Block,
    pub kind: PowerBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub consume_power: f32,
    pub buffered_power: f32,
    pub consume_items: Vec<ItemAmount>,
    pub consume_liquids: Vec<LiquidAmount>,
    pub output_liquid: Option<LiquidAmount>,
    pub power_production: f32,
    pub item_duration: f32,
    pub item_capacity: i32,
    pub item_duration_multipliers: Vec<ItemAmount>,
    pub max_nodes: i32,
    pub laser_range: f32,
    pub autolink: bool,
    pub draw_range: bool,
    pub same_block_connection: bool,
    pub laser_scale: f32,
    pub schematic_priority: i32,
    pub under_bullets: bool,
    pub crush_fragile: bool,
    pub rotate: bool,
    pub insulated: bool,
    pub no_update_disabled: bool,
    pub base_explosiveness: f32,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
    pub generate_effect: String,
    pub effect_chance: f32,
    pub floating: bool,
    pub attribute: String,
    pub min_efficiency: f32,
    pub display_efficiency_scale: f32,
    pub display_efficiency: bool,
    pub health_scaled: f32,
    pub range: f32,
    pub fog_radius: f32,
    pub heating: f32,
    pub coolant_power: f32,
    pub heat_output: f32,
    pub warmup_speed: f32,
    pub warmup_rate: f32,
    pub liquid_capacity: f32,
    pub explosion_radius: i32,
    pub explosion_damage: i32,
    pub explosion_min_warmup: f32,
    pub explosion_puddles: i32,
    pub explosion_puddle_range: f32,
    pub explosion_puddle_liquid: Option<ContentId>,
    pub explosion_puddle_amount: f32,
    pub max_heat: f32,
    pub unstable_speed: f32,
    pub explode_on_full: bool,
    pub rotate_draw: bool,
    pub can_overdrive: bool,
    pub draw_arrow: bool,
    pub rebuildable: bool,
    pub power_layer: String,
    pub laser_color2: String,
    pub drawer: String,
}

impl PowerBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: PowerBlockKind) -> Self {
        let base = Block::new(id, name);
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            consume_power: 0.0,
            buffered_power: 0.0,
            consume_items: Vec::new(),
            consume_liquids: Vec::new(),
            output_liquid: None,
            power_production: 0.0,
            item_duration: 0.0,
            item_capacity: 0,
            item_duration_multipliers: Vec::new(),
            max_nodes: 0,
            laser_range: 0.0,
            autolink: false,
            draw_range: false,
            same_block_connection: false,
            laser_scale: 0.0,
            schematic_priority: 0,
            under_bullets: false,
            crush_fragile: false,
            rotate: false,
            insulated: false,
            no_update_disabled: false,
            base_explosiveness: 0.0,
            ambient_sound: "none".into(),
            ambient_sound_volume: 0.0,
            generate_effect: "none".into(),
            effect_chance: 0.0,
            floating: false,
            attribute: String::new(),
            min_efficiency: 0.0,
            display_efficiency_scale: 1.0,
            display_efficiency: true,
            health_scaled: 0.0,
            range: 0.0,
            fog_radius: 0.0,
            heating: 0.0,
            coolant_power: 0.0,
            heat_output: 0.0,
            warmup_speed: 0.0,
            warmup_rate: 0.0,
            liquid_capacity: 0.0,
            explosion_radius: 0,
            explosion_damage: 0,
            explosion_min_warmup: 0.0,
            explosion_puddles: 0,
            explosion_puddle_range: 0.0,
            explosion_puddle_liquid: None,
            explosion_puddle_amount: 0.0,
            max_heat: 0.0,
            unstable_speed: 0.0,
            explode_on_full: false,
            rotate_draw: true,
            can_overdrive: true,
            draw_arrow: false,
            rebuildable: true,
            power_layer: String::new(),
            laser_color2: String::new(),
            drawer: String::new(),
        };
        block.apply_kind_defaults();
        block
    }

    fn apply_power_block_defaults(&mut self) {
        self.base.update = true;
        self.base.solid = true;
        self.base.has_power = true;
        self.base.group = BlockGroup::Power;
    }

    fn apply_power_generator_defaults(&mut self) {
        self.apply_power_block_defaults();
        self.base.outputs_power = true;
        self.base.consumes_power = false;
        self.base.sync = true;
        self.base.flags.push(BlockFlag::Generator);
        self.base_explosiveness = 5.0;
        self.explosion_radius = 12;
        self.explosion_puddles = 10;
        self.explosion_puddle_range = TILE_SIZE as f32 * 2.0;
        self.explosion_puddle_amount = 100.0;
    }

    fn apply_kind_defaults(&mut self) {
        match self.kind {
            PowerBlockKind::PowerNode => {
                self.apply_power_block_defaults();
                self.configurable_defaults();
                self.base.consumes_power = false;
                self.base.outputs_power = false;
                self.autolink = true;
                self.draw_range = true;
                self.laser_range = 6.0;
                self.max_nodes = 3;
                self.laser_scale = 0.25;
                self.schematic_priority = -10;
                self.base.env_enabled |= Env::SPACE;
                self.base.destructible = true;
                self.base.update = false;
            }
            PowerBlockKind::PowerDiode => {
                self.rotate = true;
                self.base.update = true;
                self.base.solid = true;
                self.insulated = true;
                self.base.group = BlockGroup::Power;
                self.no_update_disabled = true;
                self.schematic_priority = 10;
                self.base.env_enabled |= Env::SPACE;
            }
            PowerBlockKind::Battery => {
                self.apply_power_block_defaults();
                self.base.outputs_power = true;
                self.base.consumes_power = true;
                self.base.flags.push(BlockFlag::Battery);
                self.base.env_enabled |= Env::SPACE;
                self.base.destructible = true;
                self.base.update = false;
            }
            PowerBlockKind::ConsumeGenerator => {
                self.apply_power_generator_defaults();
                self.item_duration = 120.0;
                self.warmup_speed = 0.05;
                self.effect_chance = 0.01;
                self.generate_effect = "none".into();
            }
            PowerBlockKind::ThermalGenerator => {
                self.apply_power_generator_defaults();
                self.no_update_disabled = true;
                self.generate_effect = "none".into();
                self.effect_chance = 0.05;
                self.attribute = "heat".into();
            }
            PowerBlockKind::SolarGenerator => {
                self.apply_power_generator_defaults();
                self.base.flags.clear();
                self.base.env_enabled = Env::ANY;
            }
            PowerBlockKind::NuclearReactor => {
                self.apply_power_generator_defaults();
                self.item_capacity = 30;
                self.base.item_capacity = 30;
                self.base.liquid_capacity = 30.0;
                self.base.has_items = true;
                self.base.has_liquids = true;
                self.base.flags.clear();
                self.base.flags.push(BlockFlag::Reactor);
                self.base.flags.push(BlockFlag::Generator);
                self.schematic_priority = -5;
                self.base.env_enabled = Env::ANY;
                self.item_duration = 120.0;
                self.heating = 0.01;
                self.heat_output = 15.0;
                self.coolant_power = 0.5;
                self.explosion_radius = 19;
                self.explosion_damage = 1250 * 4;
            }
            PowerBlockKind::ImpactReactor => {
                self.apply_power_generator_defaults();
                self.base.has_power = true;
                self.base.has_liquids = true;
                self.base.liquid_capacity = 30.0;
                self.base.has_items = true;
                self.base.outputs_power = true;
                self.base.consumes_power = true;
                self.base.flags.clear();
                self.base.flags.push(BlockFlag::Reactor);
                self.base.flags.push(BlockFlag::Generator);
                self.base.light_radius = 115.0;
                self.base.emit_light = true;
                self.base.env_enabled = Env::ANY;
                self.warmup_speed = 0.001;
                self.item_duration = 60.0;
                self.explosion_damage = 1900 * 4;
                self.explosion_min_warmup = 0.3;
                self.drawer = "DrawMulti(DrawRegion(-bottom), DrawPlasma, DrawDefault)".into();
            }
            PowerBlockKind::BeamNode => {
                self.apply_power_block_defaults();
                self.base.consumes_power = false;
                self.base.outputs_power = false;
                self.base.env_enabled |= Env::SPACE;
                self.under_bullets = true;
                self.base.priority = 1;
                self.range = 5.0;
                self.laser_color2 = "ffd9c2".into();
                self.laser_scale = 0.4;
            }
            PowerBlockKind::LongPowerNode => {
                self.apply_kind_defaults_for_long_power_node();
            }
            PowerBlockKind::VariableReactor => {
                self.apply_power_generator_defaults();
                self.power_production = 20.0;
                self.max_heat = 100.0;
                self.unstable_speed = 1.0 / 60.0 / 3.0;
                self.warmup_speed = 0.1;
                self.effect_chance = 0.05;
                self.generate_effect = "fluxVapor".into();
                self.explosion_radius = 16;
                self.explosion_damage = 1500;
                self.explosion_puddles = 70;
                self.explosion_puddle_range = TILE_SIZE as f32 * 6.0;
                self.explosion_puddle_amount = 100.0;
                self.rebuildable = false;
            }
            PowerBlockKind::HeaterGenerator => {
                self.apply_power_generator_defaults();
                self.item_duration = 120.0;
                self.warmup_speed = 0.05;
                self.effect_chance = 0.01;
                self.generate_effect = "none".into();
                self.heat_output = 10.0;
                self.warmup_rate = 0.15;
                self.rotate = true;
                self.rotate_draw = false;
                self.can_overdrive = false;
                self.draw_arrow = true;
                self.drawer = "DrawMulti(DrawDefault, DrawHeatOutput)".into();
            }
        }
    }

    fn configurable_defaults(&mut self) {
        // Captures PowerNode's configurable wiring defaults without widening
        // base Block with one-off UI fields yet.
    }

    fn apply_kind_defaults_for_long_power_node(&mut self) {
        self.apply_power_block_defaults();
        self.base.consumes_power = false;
        self.base.outputs_power = false;
        self.autolink = true;
        self.draw_range = false;
        self.laser_range = 6.0;
        self.max_nodes = 3;
        self.laser_scale = 0.25;
        self.schematic_priority = -10;
        self.base.env_enabled |= Env::SPACE;
        self.base.destructible = true;
        self.base.update = false;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CraftingBlockData {
    pub base: Block,
    pub kind: CraftingBlockKind,
    pub craft_time: f32,
    pub craft_effect: String,
    pub update_effect: String,
    pub output_item: Option<ItemAmount>,
    pub output_liquid: Option<LiquidAmount>,
    pub output_liquids: Vec<LiquidAmount>,
    pub liquid_output_directions: Vec<i32>,
    pub results: Vec<ItemAmount>,
    pub requirements: Vec<ItemAmount>,
    pub consume_items: Vec<ItemAmount>,
    pub consume_liquids: Vec<LiquidAmount>,
    pub consume_power: f32,
    pub research_cost: Vec<ItemAmount>,
    pub boost_scale: f32,
    pub attribute: String,
    pub base_efficiency: f32,
    pub min_efficiency: f32,
    pub max_boost: f32,
    pub display_efficiency_scale: f32,
    pub display_efficiency: bool,
    pub scale_liquid_consumption: bool,
    pub outputs_liquid: bool,
    pub rotate: bool,
    pub rotate_draw: bool,
    pub invert_flip: bool,
    pub region_rotated1: i32,
    pub fog_radius: f32,
    pub light_liquid: Option<ContentId>,
    pub research_cost_multiplier: f32,
    pub heat_requirement: f32,
    pub heat_output: f32,
    pub warmup_rate: f32,
    pub max_efficiency: f32,
    pub split_heat: bool,
    pub always_unlocked: bool,
    pub all_database_tabs: bool,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
    pub drawer: String,
}

impl CraftingBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: CraftingBlockKind) -> Self {
        let mut base = Block::new(id, name);
        base.destructible = true;
        base.update = true;
        Self {
            base,
            kind,
            craft_time: 80.0,
            craft_effect: "none".into(),
            update_effect: "none".into(),
            output_item: None,
            output_liquid: None,
            output_liquids: Vec::new(),
            liquid_output_directions: Vec::new(),
            results: Vec::new(),
            requirements: Vec::new(),
            consume_items: Vec::new(),
            consume_liquids: Vec::new(),
            consume_power: 0.0,
            research_cost: Vec::new(),
            boost_scale: 0.0,
            attribute: String::new(),
            base_efficiency: 0.0,
            min_efficiency: 0.0,
            max_boost: 0.0,
            display_efficiency_scale: 1.0,
            display_efficiency: true,
            scale_liquid_consumption: false,
            outputs_liquid: false,
            rotate: true,
            rotate_draw: true,
            invert_flip: false,
            region_rotated1: 0,
            fog_radius: 0.0,
            light_liquid: None,
            research_cost_multiplier: 1.0,
            heat_requirement: 0.0,
            heat_output: 0.0,
            warmup_rate: 0.0,
            max_efficiency: 0.0,
            split_heat: false,
            always_unlocked: false,
            all_database_tabs: false,
            ambient_sound: "none".into(),
            ambient_sound_volume: 0.0,
            drawer: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockDef {
    Plain(Block),
    Floor(FloorData),
    StaticWall(StaticWallData),
    StaticTree(StaticTreeData),
    TreeBlock(TreeBlockData),
    TallBlock(TallBlockData),
    Prop(PropData),
    Ore(OreBlockData),
    Production(ProductionBlockData),
    Crafting(CraftingBlockData),
    DefenseWall(DefenseWallData),
    Effect(EffectBlockData),
    Distribution(DistributionBlockData),
    Liquid(LiquidBlockData),
    Power(PowerBlockData),
}

impl BlockDef {
    pub fn base(&self) -> &Block {
        match self {
            Self::Plain(block) => block,
            Self::Floor(floor) => &floor.base,
            Self::StaticWall(wall) => &wall.base,
            Self::StaticTree(tree) => &tree.wall.base,
            Self::TreeBlock(tree) => &tree.base,
            Self::TallBlock(tall) => &tall.base,
            Self::Prop(prop) => &prop.base,
            Self::Ore(ore) => &ore.floor.base,
            Self::Production(production) => &production.base,
            Self::Crafting(crafting) => &crafting.base,
            Self::DefenseWall(wall) => &wall.base,
            Self::Effect(effect) => &effect.base,
            Self::Distribution(distribution) => &distribution.base,
            Self::Liquid(liquid) => &liquid.base,
            Self::Power(power) => &power.base,
        }
    }

    pub fn as_floor(&self) -> Option<&FloorData> {
        match self {
            Self::Floor(floor) => Some(floor),
            Self::Ore(ore) => Some(&ore.floor),
            _ => None,
        }
    }

    pub fn as_floor_mut(&mut self) -> Option<&mut FloorData> {
        match self {
            Self::Floor(floor) => Some(floor),
            Self::Ore(ore) => Some(&mut ore.floor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockRegistry {
    blocks: Vec<BlockDef>,
    by_name: BTreeMap<String, BlockId>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &BlockDef> {
        self.blocks.iter()
    }

    pub fn get(&self, id: BlockId) -> Option<&BlockDef> {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.blocks.get(index))
    }

    pub fn get_mut(&mut self, id: BlockId) -> Option<&mut BlockDef> {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.blocks.get_mut(index))
    }

    pub fn get_by_name(&self, name: &str) -> Option<&BlockDef> {
        self.id_by_name(name).and_then(|id| self.get(id))
    }

    pub fn get_floor_by_name(&self, name: &str) -> Option<&FloorData> {
        self.get_by_name(name).and_then(BlockDef::as_floor)
    }

    pub fn get_crafting_by_name(&self, name: &str) -> Option<&CraftingBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Crafting(crafting) => Some(crafting),
            _ => None,
        }
    }

    pub fn get_production_by_name(&self, name: &str) -> Option<&ProductionBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Production(production) => Some(production),
            _ => None,
        }
    }

    pub fn get_defense_wall_by_name(&self, name: &str) -> Option<&DefenseWallData> {
        match self.get_by_name(name)? {
            BlockDef::DefenseWall(wall) => Some(wall),
            _ => None,
        }
    }

    pub fn get_effect_by_name(&self, name: &str) -> Option<&EffectBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Effect(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn get_distribution_by_name(&self, name: &str) -> Option<&DistributionBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Distribution(distribution) => Some(distribution),
            _ => None,
        }
    }

    pub fn get_liquid_by_name(&self, name: &str) -> Option<&LiquidBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Liquid(liquid) => Some(liquid),
            _ => None,
        }
    }

    pub fn get_power_by_name(&self, name: &str) -> Option<&PowerBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Power(power) => Some(power),
            _ => None,
        }
    }

    pub fn id_by_name(&self, name: &str) -> Option<BlockId> {
        self.by_name.get(name).copied()
    }

    pub fn register_plain(&mut self, name: impl Into<String>) -> BlockId {
        let id = self.next_id();
        self.insert(BlockDef::Plain(Block::new(id, name)))
    }

    pub fn register_floor(
        &mut self,
        name: impl Into<String>,
        configure: impl FnOnce(&mut FloorData),
    ) -> BlockId {
        let id = self.next_id();
        let mut floor = FloorData::new(id, name);
        configure(&mut floor);
        self.insert(BlockDef::Floor(floor))
    }

    pub fn register_static_wall(
        &mut self,
        name: impl Into<String>,
        configure: impl FnOnce(&mut StaticWallData),
    ) -> BlockId {
        let id = self.next_id();
        let mut wall = StaticWallData::new(id, name);
        configure(&mut wall);
        self.insert(BlockDef::StaticWall(wall))
    }

    pub fn register_static_tree(
        &mut self,
        name: impl Into<String>,
        configure: impl FnOnce(&mut StaticTreeData),
    ) -> BlockId {
        let id = self.next_id();
        let mut tree = StaticTreeData::new(id, name);
        configure(&mut tree);
        self.insert(BlockDef::StaticTree(tree))
    }

    pub fn register_tree_block(
        &mut self,
        name: impl Into<String>,
        configure: impl FnOnce(&mut TreeBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut tree = TreeBlockData::new(id, name);
        configure(&mut tree);
        self.insert(BlockDef::TreeBlock(tree))
    }

    pub fn register_tall_block(
        &mut self,
        name: impl Into<String>,
        configure: impl FnOnce(&mut TallBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut tall = TallBlockData::new(id, name);
        configure(&mut tall);
        self.insert(BlockDef::TallBlock(tall))
    }

    pub fn register_prop(
        &mut self,
        name: impl Into<String>,
        kind: PropKind,
        configure: impl FnOnce(&mut PropData),
    ) -> BlockId {
        let id = self.next_id();
        let mut prop = PropData::with_kind(id, name, kind);
        configure(&mut prop);
        self.insert(BlockDef::Prop(prop))
    }

    pub fn register_ore(
        &mut self,
        ore: &Item,
        configure: impl FnOnce(&mut OreBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = OreBlockData::new(id, ore);
        configure(&mut block);
        self.insert(BlockDef::Ore(block))
    }

    pub fn register_named_ore(
        &mut self,
        name: impl Into<String>,
        ore: &Item,
        configure: impl FnOnce(&mut OreBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = OreBlockData::with_name(id, name, ore);
        configure(&mut block);
        self.insert(BlockDef::Ore(block))
    }

    pub fn register_crafting(
        &mut self,
        name: impl Into<String>,
        kind: CraftingBlockKind,
        configure: impl FnOnce(&mut CraftingBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = CraftingBlockData::new(id, name, kind);
        configure(&mut block);
        block.base.derive_layout_fields();
        self.insert(BlockDef::Crafting(block))
    }

    pub fn register_production_block(
        &mut self,
        name: impl Into<String>,
        kind: ProductionBlockKind,
        configure: impl FnOnce(&mut ProductionBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = ProductionBlockData::new(id, name, kind);
        configure(&mut block);
        block.finalize();
        block.base.derive_layout_fields();
        self.insert(BlockDef::Production(block))
    }

    pub fn register_defense_wall(
        &mut self,
        name: impl Into<String>,
        configure: impl FnOnce(&mut DefenseWallData),
    ) -> BlockId {
        let id = self.next_id();
        let mut wall = DefenseWallData::new(id, name);
        configure(&mut wall);
        wall.base.derive_layout_fields();
        self.insert(BlockDef::DefenseWall(wall))
    }

    pub fn register_effect_block(
        &mut self,
        name: impl Into<String>,
        kind: EffectBlockKind,
        configure: impl FnOnce(&mut EffectBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = EffectBlockData::new(id, name, kind);
        configure(&mut block);
        block.base.derive_layout_fields();
        self.insert(BlockDef::Effect(block))
    }

    pub fn register_distribution_block(
        &mut self,
        name: impl Into<String>,
        kind: DistributionBlockKind,
        configure: impl FnOnce(&mut DistributionBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = DistributionBlockData::new(id, name, kind);
        configure(&mut block);
        block.base.derive_layout_fields();
        self.insert(BlockDef::Distribution(block))
    }

    pub fn register_liquid_block(
        &mut self,
        name: impl Into<String>,
        kind: LiquidBlockKind,
        configure: impl FnOnce(&mut LiquidBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = LiquidBlockData::new(id, name, kind);
        configure(&mut block);
        block.base.derive_layout_fields();
        self.insert(BlockDef::Liquid(block))
    }

    pub fn register_power_block(
        &mut self,
        name: impl Into<String>,
        kind: PowerBlockKind,
        configure: impl FnOnce(&mut PowerBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = PowerBlockData::new(id, name, kind);
        configure(&mut block);
        block.base.derive_layout_fields();
        self.insert(BlockDef::Power(block))
    }

    pub fn set_floor_wall_by_name(
        &mut self,
        floor_name: &str,
        wall_name: &str,
    ) -> Result<(), String> {
        let wall_id = self
            .id_by_name(wall_name)
            .ok_or_else(|| format!("missing wall block: {wall_name}"))?;
        let floor_id = self
            .id_by_name(floor_name)
            .ok_or_else(|| format!("missing floor block: {floor_name}"))?;
        let floor = self
            .get_mut(floor_id)
            .and_then(BlockDef::as_floor_mut)
            .ok_or_else(|| format!("block is not a floor: {floor_name}"))?;
        floor.wall = wall_id;
        Ok(())
    }

    pub fn set_floor_decoration_by_name(
        &mut self,
        floor_name: &str,
        decoration_name: &str,
    ) -> Result<(), String> {
        let decoration_id = self
            .id_by_name(decoration_name)
            .ok_or_else(|| format!("missing decoration block: {decoration_name}"))?;
        let floor_id = self
            .id_by_name(floor_name)
            .ok_or_else(|| format!("missing floor block: {floor_name}"))?;
        let floor = self
            .get_mut(floor_id)
            .and_then(BlockDef::as_floor_mut)
            .ok_or_else(|| format!("block is not a floor: {floor_name}"))?;
        floor.decoration = decoration_id;
        Ok(())
    }

    pub fn finalize_floor_links(&mut self) {
        let floors: Vec<(BlockId, String, BlockId, BlockId)> = self
            .blocks
            .iter()
            .filter_map(|block| {
                let floor = block.as_floor()?;
                Some((
                    floor.base.id,
                    floor.base.name.clone(),
                    floor.wall,
                    floor.decoration,
                ))
            })
            .collect();

        for (id, name, current_wall, current_decoration) in floors {
            let wall = if current_wall != 0 {
                Some(current_wall)
            } else {
                self.resolve_floor_wall_id(&name)
            };
            let decoration = if current_decoration != 0 {
                Some(current_decoration)
            } else {
                self.id_by_name(&format!("{name}-boulder"))
            };

            if let Some(floor) = self.get_mut(id).and_then(BlockDef::as_floor_mut) {
                floor.init_links(wall, decoration);
            }
        }
    }

    fn resolve_floor_wall_id(&self, floor_name: &str) -> Option<BlockId> {
        self.id_by_name(&format!("{floor_name}-wall")).or_else(|| {
            let fallback = floor_name.replace("darksand", "dune");
            (fallback != floor_name)
                .then(|| self.id_by_name(&format!("{fallback}-wall")))
                .flatten()
        })
    }

    fn next_id(&self) -> BlockId {
        self.blocks
            .len()
            .try_into()
            .expect("block registry overflowed ContentId")
    }

    fn insert(&mut self, block: BlockDef) -> BlockId {
        let id = block.base().id;
        let name = block.base().name.clone();
        assert_eq!(
            id as usize,
            self.blocks.len(),
            "block ids must follow registration order"
        );
        assert!(
            self.by_name.insert(name.clone(), id).is_none(),
            "duplicate block name: {name}"
        );
        self.blocks.push(block);
        id
    }
}

pub fn load(items: &[Item], liquids: &[Liquid]) -> BlockRegistry {
    let mut registry = BlockRegistry::new();

    // Java Blocks.load() starts with AirBlock("air"). Keeping id 0 stable is
    // important because floors fall back to air when no wall/decoration exists.
    registry.register_plain("air");

    register_environment_base(&mut registry, liquids);
    register_environment_walls(&mut registry);
    register_environment_trees_and_tall_blocks(&mut registry);
    register_environment_props(&mut registry);
    apply_environment_wall_links(&mut registry);
    apply_environment_decoration_links(&mut registry);
    apply_environment_item_drops(&mut registry, items);
    register_ores(&mut registry, items);
    register_production_blocks(&mut registry, items, liquids);
    register_crafting_blocks(&mut registry, items, liquids);
    register_defense_walls(&mut registry, items);
    register_effect_blocks(&mut registry, items, liquids);
    register_distribution_blocks(&mut registry, items, liquids);
    register_liquid_blocks(&mut registry, items, liquids);
    register_power_blocks(&mut registry, items, liquids);

    registry.finalize_floor_links();
    registry
}

fn register_environment_base(registry: &mut BlockRegistry, liquids: &[Liquid]) {
    let water = liquid_id(liquids, "water");
    let oil = liquid_id(liquids, "oil");
    let slag_liquid = liquid_id(liquids, "slag");
    let cryofluid_liquid = liquid_id(liquids, "cryofluid");
    let arkycite_liquid = liquid_id(liquids, "arkycite");

    registry.register_floor("deep-water", |floor| {
        floor.speed_multiplier = 0.2;
        floor.base.variants = 0;
        floor.liquid_drop = water;
        floor.liquid_multiplier = 1.5;
        floor.is_liquid = true;
        floor.status = "wet".into();
        floor.status_duration = 120.0;
        floor.drown_time = 200.0;
        floor.base.cache_layer = CacheLayer::Water;
        floor.base.albedo = 0.9;
        floor.supports_overlay = true;
    });
    registry.register_floor("shallow-water", |floor| {
        floor.speed_multiplier = 0.5;
        floor.base.variants = 0;
        floor.status = "wet".into();
        floor.status_duration = 90.0;
        floor.liquid_drop = water;
        floor.is_liquid = true;
        floor.base.cache_layer = CacheLayer::Water;
        floor.base.albedo = 0.9;
        floor.supports_overlay = true;
    });
    registry.register_floor("tainted-water", |floor| {
        floor.speed_multiplier = 0.5;
        floor.base.variants = 0;
        floor.is_liquid = true;
        floor.liquid_drop = water;
        floor.base.cache_layer = CacheLayer::Water;
        floor.base.albedo = 0.9;
        floor.supports_overlay = true;
        floor.status = "wet".into();
        floor.status_duration = 90.0;
    });
    registry.register_floor("deep-tainted-water", |floor| {
        floor.speed_multiplier = 0.18;
        floor.base.variants = 0;
        floor.is_liquid = true;
        floor.liquid_drop = water;
        floor.liquid_multiplier = 1.5;
        floor.base.cache_layer = CacheLayer::Water;
        floor.base.albedo = 0.9;
        floor.supports_overlay = true;
        floor.status = "wet".into();
        floor.status_duration = 140.0;
        floor.drown_time = 200.0;
    });
    registry.register_floor("darksand-tainted-water", |floor| {
        floor.speed_multiplier = 0.75;
        floor.is_liquid = true;
        floor.liquid_drop = water;
        floor.base.cache_layer = CacheLayer::Water;
        floor.base.albedo = 0.9;
        floor.status = "wet".into();
        floor.status_duration = 60.0;
        floor.supports_overlay = true;
        floor.shallow = true;
    });
    registry.register_floor("sand-water", |floor| {
        floor.speed_multiplier = 0.8;
        floor.is_liquid = true;
        floor.liquid_drop = water;
        floor.base.cache_layer = CacheLayer::Water;
        floor.base.albedo = 0.9;
        floor.status = "wet".into();
        floor.status_duration = 50.0;
        floor.supports_overlay = true;
        floor.shallow = true;
    });
    registry.register_floor("darksand-water", |floor| {
        floor.speed_multiplier = 0.8;
        floor.is_liquid = true;
        floor.liquid_drop = water;
        floor.base.cache_layer = CacheLayer::Water;
        floor.base.albedo = 0.9;
        floor.status = "wet".into();
        floor.status_duration = 50.0;
        floor.supports_overlay = true;
        floor.shallow = true;
    });
    registry.register_floor("tar", |floor| {
        floor.drown_time = 230.0;
        floor.status = "tarred".into();
        floor.status_duration = 240.0;
        floor.speed_multiplier = 0.19;
        floor.base.variants = 0;
        floor.liquid_drop = oil;
        floor.is_liquid = true;
        floor.base.cache_layer = CacheLayer::Tar;
        floor.base.obstructs_light = true;
    });
    registry.register_floor("pooled-cryofluid", |floor| {
        floor.supports_overlay = true;
        floor.overlay_alpha = 0.35;
        floor.drown_time = 150.0;
        floor.status = "freezing".into();
        floor.status_duration = 240.0;
        floor.speed_multiplier = 0.5;
        floor.base.variants = 0;
        floor.liquid_drop = cryofluid_liquid;
        floor.liquid_multiplier = 0.5;
        floor.is_liquid = true;
        floor.base.cache_layer = CacheLayer::Cryofluid;
        floor.base.emit_light = true;
        floor.base.light_radius = 25.0;
        floor.base.light_color_rgba = 0x00ffffff;
        floor.base.obstructs_light = true;
        floor.force_draw_light = true;
    });
    registry.register_floor("molten-slag", |floor| {
        floor.drown_time = 230.0;
        floor.status = "melting".into();
        floor.status_duration = 240.0;
        floor.speed_multiplier = 0.19;
        floor.base.variants = 0;
        floor.liquid_drop = slag_liquid;
        floor.is_liquid = true;
        floor.base.cache_layer = CacheLayer::Slag;
        floor.base.emit_light = true;
        floor.base.light_radius = 40.0;
        floor.base.light_color_rgba = 0xffa50061;
        floor.base.obstructs_light = true;
        floor.force_draw_light = true;
    });
    registry.register_floor("space", |floor| {
        floor.base.cache_layer = CacheLayer::Space;
        floor.base.placeable_on = false;
        floor.base.solid = true;
        floor.base.variants = 0;
        floor.can_shadow = false;
        floor.draw_edge_out = false;
    });
    registry.register_floor("empty", |floor| {
        floor.base.variants = 0;
        floor.can_shadow = false;
        floor.base.placeable_on = false;
        floor.base.solid = true;
        floor.draw_edge_out = false;
    });

    let stone = registry.register_floor("stone", |_| {});
    registry.register_floor("crater-stone", |floor| {
        floor.base.variants = 3;
        floor.blend_group = stone;
    });
    registry.register_floor("char", |floor| {
        floor.blend_group = stone;
    });
    registry.register_floor("basalt", |_| {});
    let basalt = registry
        .id_by_name("basalt")
        .expect("basalt just registered");
    registry.register_floor("hotrock", |floor| {
        floor.blend_group = basalt;
        floor.base.emit_light = true;
        floor.base.light_radius = 30.0;
        floor.base.light_color_rgba = 0xffa50026;
    });
    registry.register_floor("magmarock", |floor| {
        floor.blend_group = basalt;
        floor.base.emit_light = true;
        floor.base.light_radius = 50.0;
        floor.base.light_color_rgba = 0xffa5004d;
    });
    registry.register_floor("sand-floor", |_| {});
    registry.register_floor("darksand", |_| {});
    registry.register_floor("dirt", |_| {});
    registry.register_floor("mud", |floor| {
        floor.speed_multiplier = 0.6;
        floor.base.variants = 3;
        floor.status = "muddy".into();
        floor.status_duration = 30.0;
        floor.base.cache_layer = CacheLayer::Mud;
        floor.walk_sound = "stepMud".into();
        floor.walk_sound_volume = 0.08;
        floor.walk_sound_pitch_min = 0.4;
        floor.walk_sound_pitch_max = 0.5;
    });
    registry.register_floor("dacite", |_| {});
    registry.register_floor("rhyolite", |_| {});
    let rhyolite = registry
        .id_by_name("rhyolite")
        .expect("rhyolite just registered");
    registry.register_floor("rhyolite-crater", |floor| {
        floor.blend_group = rhyolite;
    });
    registry.register_floor("rough-rhyolite", |floor| {
        floor.base.variants = 3;
    });
    registry.register_floor("regolith", |_| {});
    registry.register_floor("yellow-stone", |_| {});
    registry.register_floor("carbon-stone", |floor| {
        floor.base.variants = 4;
    });
    registry.register_floor("ferric-stone", |_| {});
    let ferric_stone = registry
        .id_by_name("ferric-stone")
        .expect("ferric-stone just registered");
    registry.register_floor("ferric-craters", |floor| {
        floor.base.variants = 3;
        floor.blend_group = ferric_stone;
    });
    registry.register_floor("beryllic-stone", |floor| {
        floor.base.variants = 4;
    });
    registry.register_floor("crystalline-stone", |floor| {
        floor.base.variants = 5;
    });
    registry.register_floor("crystal-floor", |floor| {
        floor.base.variants = 4;
    });
    registry.register_floor("yellow-stone-plates", |floor| {
        floor.base.variants = 3;
    });
    registry.register_floor("red-stone", |floor| {
        floor.base.variants = 4;
    });
    registry.register_floor("dense-red-stone", |floor| {
        floor.base.variants = 4;
    });
    registry.register_floor("red-ice", |floor| {
        floor.drag_multiplier = 0.4;
        floor.speed_multiplier = 0.9;
    });
    registry.register_floor("arkycite-floor", |floor| {
        floor.speed_multiplier = 0.3;
        floor.base.variants = 0;
        floor.liquid_drop = arkycite_liquid;
        floor.is_liquid = true;
        floor.drown_time = 200.0;
        floor.base.cache_layer = CacheLayer::Arkycite;
        floor.base.albedo = 0.9;
        floor.base.obstructs_light = true;
    });
    registry.register_floor("arkyic-stone", |floor| {
        floor.base.variants = 3;
    });
    registry.register_floor("redmat", |_| {});
    registry.register_floor("bluemat", |_| {});
    registry.register_floor("snow", |_| {});
    registry.register_floor("ice", |floor| {
        floor.drag_multiplier = 0.35;
        floor.speed_multiplier = 0.9;
    });
    registry.register_floor("shale", |floor| {
        floor.base.variants = 3;
        floor.ore_default = false;
    });
    registry.register_floor("moss", |floor| {
        floor.base.variants = 3;
    });
    registry.register_floor("grass", |floor| {
        floor.base.variants = 3;
    });
    registry.register_floor("core-zone", |floor| {
        floor.base.variants = 0;
        floor.allow_core_placement = true;
    });
    registry.register_floor("salt", |floor| {
        floor.base.variants = 0;
    });
    registry.register_floor("ice-snow", |floor| {
        floor.drag_multiplier = 0.6;
        floor.base.variants = 3;
    });
    registry.register_floor("spore-moss", |floor| {
        floor.base.variants = 3;
    });

    registry.register_floor("metal-floor", |floor| {
        floor.base.variants = 0;
    });
    registry.register_floor("metal-floor-damaged", |floor| {
        floor.base.variants = 3;
    });
    for floor_name in [
        "metal-floor-2",
        "metal-floor-3",
        "metal-floor-4",
        "metal-floor-5",
        "dark-panel-1",
        "dark-panel-2",
        "dark-panel-3",
        "dark-panel-4",
        "dark-panel-5",
        "dark-panel-6",
    ] {
        registry.register_floor(floor_name, |floor| {
            floor.base.variants = 0;
        });
    }

    for floor_name in [
        "metal-tiles-1",
        "metal-tiles-2",
        "metal-tiles-3",
        "metal-tiles-4",
        "metal-tiles-5",
        "metal-tiles-9",
        "metal-tiles-10",
    ] {
        registry.register_floor(floor_name, configure_metal_tiles_floor);
    }
    registry.register_floor("metal-tiles-6", |floor| {
        configure_metal_tiles_floor(floor);
        floor.base.emit_light = true;
        floor.base.light_radius = 30.0;
        // Team.crux.color.cpy().a(0.3f) in Java. Keep the alpha-bearing
        // serialized color shell stable until Team palette data is migrated.
        floor.base.light_color_rgba = 0xff00004d;
    });
    registry.register_floor("metal-tiles-7", |floor| {
        configure_metal_tiles_floor(floor);
        floor.autotile_mid_variants = 9;
    });
    registry.register_floor("metal-tiles-8", |floor| {
        configure_metal_tiles_floor(floor);
        floor.autotile_mid_variants = 2;
    });
    registry.register_floor("metal-tiles-11", |floor| {
        configure_metal_tiles_floor(floor);
        floor.autotile_variants = 3;
    });
    registry.register_floor("metal-tiles-12", |floor| {
        configure_metal_tiles_floor(floor);
        floor.autotile_variants = 4;
        floor.base.emit_light = true;
        floor.base.light_radius = 30.0;
        // Team.crux.color.cpy().a(0.3f) in Java.
        floor.base.light_color_rgba = 0xff00004d;
    });
    registry.register_floor("metal-tiles-13", |floor| {
        configure_metal_tiles_floor(floor);
        floor.autotile_mid_variants = 6;
    });
    registry.register_floor("pebbles", configure_overlay_floor);
    registry.register_floor("tendrils", configure_overlay_floor);
}

fn register_environment_walls(registry: &mut BlockRegistry) {
    registry.register_static_wall("stone-wall", |_| {});
    registry.register_static_wall("spore-wall", |_| {});
    registry.register_static_wall("ice-wall", |wall| {
        wall.base.albedo = 0.6;
    });
    registry.register_static_wall("dirt-wall", |_| {});
    registry.register_static_wall("dacite-wall", |_| {});
    registry.register_static_wall("dune-wall", |_| {});
    registry.register_static_wall("regolith-wall", |_| {});
    registry.register_static_wall("yellow-stone-wall", |_| {});
    registry.register_static_wall("rhyolite-wall", |_| {});
    registry.register_static_wall("carbon-wall", |_| {});
    registry.register_static_wall("ferric-stone-wall", |_| {});
    registry.register_static_wall("beryllic-stone-wall", |_| {});
    registry.register_static_wall("arkyic-wall", |wall| {
        wall.base.variants = 3;
    });
    registry.register_static_wall("crystalline-stone-wall", |wall| {
        wall.base.variants = 4;
    });
    registry.register_static_wall("red-ice-wall", |_| {});
    registry.register_static_wall("red-stone-wall", |_| {});
    registry.register_static_wall("snow-wall", |_| {});
    registry.register_static_wall("sand-wall", |_| {});
    registry.register_static_wall("shrubs", |_| {});
    registry.register_static_wall("shale-wall", |_| {});
    registry.register_static_wall("salt-wall", |_| {});
    registry.register_static_wall("dark-metal", |_| {});
    registry.register_static_wall("metal-wall-1", |wall| {
        wall.autotile = true;
    });
    registry.register_static_wall("metal-wall-2", |wall| {
        wall.autotile = true;
        wall.autotile_mid_variants = 2;
    });
    registry.register_static_wall("metal-wall-3", |wall| {
        wall.autotile = true;
    });
    registry.register_static_wall("graphitic-wall", |_| {});
}

fn configure_metal_tiles_floor(floor: &mut FloorData) {
    floor.autotile = true;
    floor.draw_edge_out = false;
    floor.draw_edge_in = false;
}

fn configure_overlay_floor(floor: &mut FloorData) {
    floor.overlay_floor = true;
    floor.base.use_color = false;
}

fn register_environment_trees_and_tall_blocks(registry: &mut BlockRegistry) {
    registry.register_static_tree("red-diamond-wall", |tree| {
        tree.wall.base.variants = 3;
    });
    registry.register_static_tree("spore-pine", |_| {});
    registry.register_static_tree("snow-pine", |_| {});
    registry.register_static_tree("pine", |_| {});
    registry.register_tree_block("white-tree-dead", |_| {});
    registry.register_tree_block("white-tree", |_| {});

    registry.register_tall_block("crystal-cluster", |tall| {
        tall.base.variants = 3;
        tall.base.clip_size = 128.0;
        tall.base.derive_layout_fields();
    });
    registry.register_tall_block("vibrant-crystal-cluster", |tall| {
        tall.base.variants = 3;
        tall.base.clip_size = 128.0;
        tall.base.derive_layout_fields();
    });
    registry.register_tall_block("crystal-blocks", |tall| {
        tall.base.variants = 3;
        tall.base.clip_size = 128.0;
        tall.base.derive_layout_fields();
        tall.shadow_alpha = 0.5;
        tall.shadow_offset = -2.5;
    });
    registry.register_tall_block("crystal-orbs", |tall| {
        tall.base.variants = 3;
        tall.base.clip_size = 128.0;
        tall.base.derive_layout_fields();
        tall.shadow_alpha = 0.5;
        tall.shadow_offset = -2.5;
    });
}

fn register_environment_props(registry: &mut BlockRegistry) {
    registry.register_prop("spore-cluster", PropKind::Prop, |prop| {
        prop.base.variants = 3;
        prop.base.break_sound = "plantBreak".into();
        prop.base.obstructs_light = false;
    });
    registry.register_prop("redweed", PropKind::Seaweed, |prop| {
        prop.base.variants = 3;
    });
    registry.register_prop("pur-bush", PropKind::SeaBush, |_| {});
    registry.register_prop("yellowcoral", PropKind::SeaBush, |prop| {
        prop.sea_bush = Some(SeaBushData {
            lobes_min: 2,
            lobes_max: 3,
            mag_min: 2.0,
            mag_max: 8.0,
            origin: 0.3,
            spread: 40.0,
            scl_min: 60.0,
            scl_max: 100.0,
            ..SeaBushData::default()
        });
    });

    registry.register_prop("boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("snow-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("shale-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("sand-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("basalt-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("dacite-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("carbon-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("ferric-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("beryllic-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("yellow-stone-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("arkyic-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 3;
        prop.base.custom_shadow = true;
        prop.base.obstructs_light = false;
    });
    registry.register_prop("crystalline-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 2;
    });
    registry.register_prop("red-ice-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 3;
    });
    registry.register_prop("rhyolite-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 3;
    });
    registry.register_prop("red-stone-boulder", PropKind::Prop, |prop| {
        prop.base.variants = 4;
    });
}

fn apply_environment_wall_links(registry: &mut BlockRegistry) {
    for floor in ["tainted-water", "deep-tainted-water", "spore-moss"] {
        registry
            .set_floor_wall_by_name(floor, "spore-wall")
            .expect("minimal spore wall links must resolve");
    }
    registry
        .set_floor_wall_by_name("ice-snow", "ice-wall")
        .expect("minimal ice wall links must resolve");
    registry
        .set_floor_wall_by_name("dirt", "dirt-wall")
        .expect("minimal dirt wall links must resolve");
    registry
        .set_floor_wall_by_name("snow", "snow-wall")
        .expect("minimal snow wall links must resolve");
    registry
        .set_floor_wall_by_name("shale", "shale-wall")
        .expect("minimal shale wall links must resolve");
    registry
        .set_floor_wall_by_name("salt", "salt-wall")
        .expect("minimal salt wall links must resolve");
    registry
        .set_floor_wall_by_name("moss", "spore-pine")
        .expect("spore pine wall link must resolve");
    for floor in [
        "hotrock",
        "magmarock",
        "basalt",
        "darksand-water",
        "darksand-tainted-water",
    ] {
        registry
            .set_floor_wall_by_name(floor, "dune-wall")
            .expect("minimal dune wall links must resolve");
    }
    for floor in ["sand-water", "deep-water", "shallow-water", "sand-floor"] {
        registry
            .set_floor_wall_by_name(floor, "sand-wall")
            .expect("minimal sand wall links must resolve");
    }
    for (floor, wall) in [
        ("molten-slag", "yellow-stone-wall"),
        ("yellow-stone-plates", "yellow-stone-wall"),
        ("rhyolite-crater", "rhyolite-wall"),
        ("rough-rhyolite", "rhyolite-wall"),
        ("carbon-stone", "carbon-wall"),
        ("arkycite-floor", "arkyic-wall"),
        ("arkyic-stone", "arkyic-wall"),
        ("crystal-floor", "crystalline-stone-wall"),
        ("crystalline-stone", "crystalline-stone-wall"),
        ("dense-red-stone", "red-stone-wall"),
    ] {
        registry
            .set_floor_wall_by_name(floor, wall)
            .expect("extended environment wall links must resolve");
    }
    for floor in [
        "metal-floor",
        "metal-floor-damaged",
        "metal-floor-2",
        "metal-floor-3",
        "metal-floor-4",
        "metal-floor-5",
        "dark-panel-1",
        "dark-panel-2",
        "dark-panel-3",
        "dark-panel-4",
        "dark-panel-5",
        "dark-panel-6",
    ] {
        registry
            .set_floor_wall_by_name(floor, "dark-metal")
            .expect("minimal dark metal wall links must resolve");
    }
}

fn apply_environment_decoration_links(registry: &mut BlockRegistry) {
    for floor in ["stone", "crater-stone"] {
        registry
            .set_floor_decoration_by_name(floor, "boulder")
            .expect("minimal boulder decoration links must resolve");
    }
    for floor in ["snow", "ice", "ice-snow", "salt"] {
        registry
            .set_floor_decoration_by_name(floor, "snow-boulder")
            .expect("minimal snow boulder decoration links must resolve");
    }
    registry
        .set_floor_decoration_by_name("shale", "shale-boulder")
        .expect("minimal shale boulder decoration link must resolve");
    registry
        .set_floor_decoration_by_name("sand-floor", "sand-boulder")
        .expect("minimal sand boulder decoration link must resolve");
    for floor in ["basalt", "hotrock", "darksand", "magmarock"] {
        registry
            .set_floor_decoration_by_name(floor, "basalt-boulder")
            .expect("minimal basalt boulder decoration links must resolve");
    }
    registry
        .set_floor_decoration_by_name("dacite", "dacite-boulder")
        .expect("dacite boulder decoration link must resolve");
    registry
        .set_floor_decoration_by_name("carbon-stone", "carbon-boulder")
        .expect("carbon boulder decoration link must resolve");
    for floor in ["ferric-stone", "ferric-craters"] {
        registry
            .set_floor_decoration_by_name(floor, "ferric-boulder")
            .expect("ferric boulder decoration links must resolve");
    }
    registry
        .set_floor_decoration_by_name("beryllic-stone", "beryllic-boulder")
        .expect("beryllic boulder decoration link must resolve");
    for floor in ["yellow-stone", "regolith", "yellow-stone-plates"] {
        registry
            .set_floor_decoration_by_name(floor, "yellow-stone-boulder")
            .expect("yellow stone boulder decoration links must resolve");
    }
    registry
        .set_floor_decoration_by_name("arkyic-stone", "arkyic-boulder")
        .expect("arkyic boulder decoration link must resolve");
    registry
        .set_floor_decoration_by_name("crystalline-stone", "crystalline-boulder")
        .expect("crystalline boulder decoration link must resolve");
    registry
        .set_floor_decoration_by_name("red-ice", "red-ice-boulder")
        .expect("red ice boulder decoration link must resolve");
    for floor in ["rhyolite", "rough-rhyolite"] {
        registry
            .set_floor_decoration_by_name(floor, "rhyolite-boulder")
            .expect("rhyolite boulder decoration links must resolve");
    }
    for floor in ["dense-red-stone", "red-stone"] {
        registry
            .set_floor_decoration_by_name(floor, "red-stone-boulder")
            .expect("red stone boulder decoration links must resolve");
    }
}

fn apply_environment_item_drops(registry: &mut BlockRegistry, items: &[Item]) {
    if let Some(sand) = find_item(items, "sand") {
        let sand_id = sand.base.mappable.base.id;
        for floor_name in ["sand-floor", "darksand"] {
            if let Some(id) = registry.id_by_name(floor_name) {
                if let Some(floor) = registry.get_mut(id).and_then(BlockDef::as_floor_mut) {
                    floor.base.item_drop = Some(sand_id);
                }
            }
        }
    }
    if let Some(graphite) = find_item(items, "graphite") {
        let graphite_id = graphite.base.mappable.base.id;
        if let Some(id) = registry.id_by_name("graphitic-wall") {
            if let Some(block) = registry.get_mut(id) {
                match block {
                    BlockDef::StaticWall(wall) => {
                        wall.base.item_drop = Some(graphite_id);
                        wall.base.variants = 3;
                    }
                    _ => panic!("graphitic-wall should be a static wall"),
                }
            }
        }
    }
}

fn register_ores(registry: &mut BlockRegistry, items: &[Item]) {
    if let Some(copper) = find_item(items, "copper") {
        registry.register_ore(copper, |ore| {
            ore.floor.ore_default = true;
            ore.floor.ore_threshold = 0.81;
            ore.floor.ore_scale = 23.47619;
        });
    }
    if let Some(lead) = find_item(items, "lead") {
        registry.register_ore(lead, |ore| {
            ore.floor.ore_default = true;
            ore.floor.ore_threshold = 0.828;
            ore.floor.ore_scale = 23.952381;
        });
    }
    if let Some(scrap) = find_item(items, "scrap") {
        registry.register_ore(scrap, |_| {});
    }
    if let Some(coal) = find_item(items, "coal") {
        registry.register_ore(coal, |ore| {
            ore.floor.ore_default = true;
            ore.floor.ore_threshold = 0.846;
            ore.floor.ore_scale = 24.428572;
        });
    }
    if let Some(titanium) = find_item(items, "titanium") {
        registry.register_ore(titanium, |ore| {
            ore.floor.ore_default = true;
            ore.floor.ore_threshold = 0.864;
            ore.floor.ore_scale = 24.904762;
        });
    }
    if let Some(thorium) = find_item(items, "thorium") {
        registry.register_ore(thorium, |ore| {
            ore.floor.ore_default = true;
            ore.floor.ore_threshold = 0.882;
            ore.floor.ore_scale = 25.380953;
        });
        registry.register_named_ore("ore-wall-thorium", thorium, |ore| {
            ore.floor.wall_ore = true;
            ore.setup(thorium);
        });
    }
    if let Some(beryllium) = find_item(items, "beryllium") {
        registry.register_ore(beryllium, |_| {});
        registry.register_named_ore("ore-wall-beryllium", beryllium, |ore| {
            ore.floor.wall_ore = true;
            ore.setup(beryllium);
        });
    }
    if let Some(tungsten) = find_item(items, "tungsten") {
        registry.register_ore(tungsten, |_| {});
        registry.register_named_ore("ore-wall-tungsten", tungsten, |ore| {
            ore.floor.wall_ore = true;
            ore.setup(tungsten);
        });
    }
    if let Some(graphite) = find_item(items, "graphite") {
        registry.register_named_ore("ore-wall-graphite", graphite, |ore| {
            ore.floor.wall_ore = true;
            ore.setup(graphite);
        });
    }
}

fn item_amount(items: &[Item], name: &str, amount: i32) -> Option<ItemAmount> {
    Some(ItemAmount {
        item: find_item(items, name)?.base.mappable.base.id,
        amount,
    })
}

fn liquid_amount(liquids: &[Liquid], name: &str, amount: f32) -> Option<LiquidAmount> {
    Some(LiquidAmount {
        liquid: liquid_id(liquids, name)?,
        amount,
    })
}

fn liquid_consume(
    liquids: &[Liquid],
    name: &str,
    amount: f32,
    booster: bool,
) -> Option<LiquidConsume> {
    Some(LiquidConsume {
        liquid: liquid_id(liquids, name)?,
        amount,
        booster,
    })
}

fn push_item_amount(target: &mut Vec<ItemAmount>, items: &[Item], name: &str, amount: i32) {
    if let Some(item) = item_amount(items, name, amount) {
        target.push(item);
    }
}

fn push_liquid_amount(target: &mut Vec<LiquidAmount>, liquids: &[Liquid], name: &str, amount: f32) {
    if let Some(liquid) = liquid_amount(liquids, name, amount) {
        target.push(liquid);
    }
}

fn set_requirements(target: &mut Vec<ItemAmount>, items: &[Item], specs: &[(&str, i32)]) {
    target.clear();
    for (name, amount) in specs {
        push_item_amount(target, items, name, *amount);
    }
}

fn push_liquid_consume(
    target: &mut Vec<LiquidConsume>,
    liquids: &[Liquid],
    name: &str,
    amount: f32,
    booster: bool,
) {
    if let Some(liquid) = liquid_consume(liquids, name, amount, booster) {
        target.push(liquid);
    }
}

fn register_production_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_production_block(
        "mechanical-drill",
        ProductionBlockKind::Drill,
        |production| {
            set_requirements(&mut production.requirements, items, &[("copper", 12)]);
            production.tier = 2;
            production.drill_time = 600.0;
            production.base.size = 2;
            production.base.env_enabled ^= Env::SPACE;
            set_requirements(&mut production.research_cost, items, &[("copper", 10)]);
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "water",
                0.05,
                true,
            );
        },
    );

    registry.register_production_block(
        "pneumatic-drill",
        ProductionBlockKind::Drill,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[("copper", 18), ("graphite", 10)],
            );
            production.tier = 3;
            production.drill_time = 400.0;
            production.base.size = 2;
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "water",
                3.5 / 60.0,
                true,
            );
        },
    );

    registry.register_production_block("laser-drill", ProductionBlockKind::Drill, |production| {
        set_requirements(
            &mut production.requirements,
            items,
            &[
                ("copper", 35),
                ("graphite", 30),
                ("silicon", 30),
                ("titanium", 20),
            ],
        );
        production.drill_time = 280.0;
        production.base.size = 3;
        production.base.has_power = true;
        production.tier = 4;
        production.update_effect = "pulverizeMedium".into();
        production.drill_effect = "mineBig".into();
        production.consume_power = 1.10;
        push_liquid_consume(
            &mut production.consume_liquids,
            liquids,
            "water",
            0.08,
            true,
        );
    });

    registry.register_production_block("blast-drill", ProductionBlockKind::Drill, |production| {
        set_requirements(
            &mut production.requirements,
            items,
            &[
                ("copper", 65),
                ("silicon", 60),
                ("titanium", 50),
                ("thorium", 75),
            ],
        );
        production.drill_time = 280.0;
        production.base.size = 4;
        production.draw_rim = true;
        production.base.has_power = true;
        production.tier = 5;
        production.update_effect = "pulverizeRed".into();
        production.update_effect_chance = 0.03;
        production.drill_effect = "mineHuge".into();
        production.rotate_speed = 6.0;
        production.warmup_speed = 0.01;
        production.base.item_capacity = 20;
        production.liquid_boost_intensity = 1.8;
        production.consume_power = 3.0;
        push_liquid_consume(&mut production.consume_liquids, liquids, "water", 0.1, true);
    });
}

fn register_defense_walls(registry: &mut BlockRegistry, items: &[Item]) {
    const WALL_HEALTH_MULTIPLIER: i32 = 4;

    registry.register_defense_wall("copper-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("copper", 6)]);
        wall.base.health = 80 * WALL_HEALTH_MULTIPLIER;
        wall.research_cost_multiplier = 0.1;
    });
    registry.register_defense_wall("copper-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("copper", 24)]);
        wall.base.health = 80 * 4 * WALL_HEALTH_MULTIPLIER;
        wall.base.size = 2;
    });
    registry.register_defense_wall("titanium-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("titanium", 6)]);
        wall.base.health = 110 * WALL_HEALTH_MULTIPLIER;
    });
    registry.register_defense_wall("titanium-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("titanium", 24)]);
        wall.base.health = 110 * WALL_HEALTH_MULTIPLIER * 4;
        wall.base.size = 2;
    });
    registry.register_defense_wall("plastanium-wall", |wall| {
        set_requirements(
            &mut wall.requirements,
            items,
            &[("plastanium", 5), ("metaglass", 2)],
        );
        wall.base.health = 125 * WALL_HEALTH_MULTIPLIER;
        wall.insulated = true;
        wall.absorb_lasers = true;
        wall.schematic_priority = 10;
    });
    registry.register_defense_wall("plastanium-wall-large", |wall| {
        set_requirements(
            &mut wall.requirements,
            items,
            &[("plastanium", 20), ("metaglass", 8)],
        );
        wall.base.health = 125 * WALL_HEALTH_MULTIPLIER * 4;
        wall.base.size = 2;
        wall.insulated = true;
        wall.absorb_lasers = true;
        wall.schematic_priority = 10;
    });
    registry.register_defense_wall("thorium-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("thorium", 6)]);
        wall.base.health = 200 * WALL_HEALTH_MULTIPLIER;
    });
    registry.register_defense_wall("thorium-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("thorium", 24)]);
        wall.base.health = 200 * WALL_HEALTH_MULTIPLIER * 4;
        wall.base.size = 2;
    });
    registry.register_defense_wall("phase-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("phase-fabric", 6)]);
        wall.base.health = 150 * WALL_HEALTH_MULTIPLIER;
        wall.chance_deflect = 10.0;
        wall.flash_hit = true;
    });
    registry.register_defense_wall("phase-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("phase-fabric", 24)]);
        wall.base.health = 150 * 4 * WALL_HEALTH_MULTIPLIER;
        wall.base.size = 2;
        wall.chance_deflect = 10.0;
        wall.flash_hit = true;
    });
    registry.register_defense_wall("surge-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("surge-alloy", 6)]);
        wall.base.health = 230 * WALL_HEALTH_MULTIPLIER;
        wall.lightning_chance = 0.05;
    });
    registry.register_defense_wall("surge-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("surge-alloy", 24)]);
        wall.base.health = 230 * 4 * WALL_HEALTH_MULTIPLIER;
        wall.base.size = 2;
        wall.lightning_chance = 0.05;
    });
    registry.register_defense_wall("door", |wall| {
        wall.kind = DefenseWallKind::Door;
        set_requirements(
            &mut wall.requirements,
            items,
            &[("titanium", 6), ("silicon", 4)],
        );
        wall.base.health = 100 * WALL_HEALTH_MULTIPLIER;
        wall.base.solid = false;
        wall.solidifies = true;
        wall.consumes_tap = true;
    });
    registry.register_defense_wall("door-large", |wall| {
        wall.kind = DefenseWallKind::Door;
        set_requirements(
            &mut wall.requirements,
            items,
            &[("titanium", 24), ("silicon", 16)],
        );
        wall.base.health = 100 * 4 * WALL_HEALTH_MULTIPLIER;
        wall.base.size = 2;
        wall.base.solid = false;
        wall.solidifies = true;
        wall.consumes_tap = true;
    });

    registry.register_defense_wall("scrap-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("scrap", 6)]);
        wall.base.health = 60 * WALL_HEALTH_MULTIPLIER;
        wall.base.variants = 5;
        wall.build_cost_multiplier = 4.0;
    });
    registry.register_defense_wall("scrap-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("scrap", 24)]);
        wall.base.health = 60 * 4 * WALL_HEALTH_MULTIPLIER;
        wall.base.size = 2;
        wall.base.variants = 4;
        wall.build_cost_multiplier = 4.0;
    });
    registry.register_defense_wall("scrap-wall-huge", |wall| {
        set_requirements(&mut wall.requirements, items, &[("scrap", 54)]);
        wall.base.health = 60 * 9 * WALL_HEALTH_MULTIPLIER;
        wall.base.size = 3;
        wall.base.variants = 3;
        wall.build_cost_multiplier = 4.0;
    });
    registry.register_defense_wall("scrap-wall-gigantic", |wall| {
        set_requirements(&mut wall.requirements, items, &[("scrap", 96)]);
        wall.base.health = 60 * 16 * WALL_HEALTH_MULTIPLIER;
        wall.base.size = 4;
        wall.build_cost_multiplier = 4.0;
    });

    registry.register_defense_wall("beryllium-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("beryllium", 6)]);
        wall.base.health = 130 * WALL_HEALTH_MULTIPLIER;
        wall.armor = 2.0;
        wall.build_cost_multiplier = 8.0;
    });
    registry.register_defense_wall("beryllium-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("beryllium", 24)]);
        wall.base.health = 130 * WALL_HEALTH_MULTIPLIER * 4;
        wall.armor = 2.0;
        wall.build_cost_multiplier = 5.0;
        wall.base.size = 2;
    });
    registry.register_defense_wall("tungsten-wall", |wall| {
        set_requirements(&mut wall.requirements, items, &[("tungsten", 6)]);
        wall.base.health = 180 * WALL_HEALTH_MULTIPLIER;
        wall.armor = 14.0;
        wall.build_cost_multiplier = 8.0;
    });
    registry.register_defense_wall("tungsten-wall-large", |wall| {
        set_requirements(&mut wall.requirements, items, &[("tungsten", 24)]);
        wall.base.health = 180 * WALL_HEALTH_MULTIPLIER * 4;
        wall.armor = 14.0;
        wall.build_cost_multiplier = 5.0;
        wall.base.size = 2;
    });
    registry.register_defense_wall("blast-door", |wall| {
        wall.kind = DefenseWallKind::AutoDoor;
        set_requirements(
            &mut wall.requirements,
            items,
            &[("tungsten", 24), ("silicon", 24)],
        );
        wall.base.health = 175 * WALL_HEALTH_MULTIPLIER * 4;
        wall.armor = 14.0;
        wall.base.size = 2;
        wall.base.solid = false;
        wall.solidifies = true;
        wall.team_passable = true;
        wall.no_update_disabled = true;
        wall.base.update = true;
    });
    registry.register_defense_wall("reinforced-surge-wall", |wall| {
        set_requirements(
            &mut wall.requirements,
            items,
            &[("surge-alloy", 6), ("tungsten", 2)],
        );
        wall.base.health = 250 * WALL_HEALTH_MULTIPLIER;
        wall.lightning_chance = 0.05;
        wall.lightning_damage = 30.0;
        wall.armor = 20.0;
        set_requirements(
            &mut wall.research_cost,
            items,
            &[("surge-alloy", 20), ("tungsten", 100)],
        );
    });
    registry.register_defense_wall("reinforced-surge-wall-large", |wall| {
        set_requirements(
            &mut wall.requirements,
            items,
            &[("surge-alloy", 24), ("tungsten", 8)],
        );
        wall.base.health = 250 * WALL_HEALTH_MULTIPLIER * 4;
        wall.lightning_chance = 0.05;
        wall.lightning_damage = 30.0;
        wall.armor = 20.0;
        wall.base.size = 2;
        set_requirements(
            &mut wall.research_cost,
            items,
            &[("surge-alloy", 40), ("tungsten", 200)],
        );
    });
    registry.register_defense_wall("carbide-wall", |wall| {
        set_requirements(
            &mut wall.requirements,
            items,
            &[("thorium", 6), ("carbide", 6)],
        );
        wall.base.health = 270 * WALL_HEALTH_MULTIPLIER;
        wall.armor = 16.0;
    });
    registry.register_defense_wall("carbide-wall-large", |wall| {
        set_requirements(
            &mut wall.requirements,
            items,
            &[("thorium", 24), ("carbide", 24)],
        );
        wall.base.health = 270 * WALL_HEALTH_MULTIPLIER * 4;
        wall.armor = 16.0;
        wall.base.size = 2;
    });
    registry.register_defense_wall("shielded-wall", |wall| {
        wall.kind = DefenseWallKind::ShieldWall;
        set_requirements(
            &mut wall.requirements,
            items,
            &[("phase-fabric", 20), ("surge-alloy", 12), ("beryllium", 12)],
        );
        wall.consume_power = 3.0 / 60.0;
        wall.base.outputs_power = false;
        wall.base.has_power = true;
        wall.base.consumes_power = true;
        wall.base.conductive_power = true;
        wall.chance_deflect = 8.0;
        wall.base.health = 260 * WALL_HEALTH_MULTIPLIER * 4;
        wall.armor = 15.0;
        wall.base.size = 2;
        wall.base.update = true;
        wall.shield_health = 900.0;
        wall.shield_break_cooldown = 60.0 * 10.0;
        wall.shield_regen_speed = 2.0;
    });
}

fn register_effect_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_effect_block("mender", EffectBlockKind::MendProjector, |effect| {
        set_requirements(
            &mut effect.requirements,
            items,
            &[("lead", 30), ("copper", 25)],
        );
        effect.consume_power = 0.3;
        effect.base.has_power = true;
        effect.base.consumes_power = true;
        effect.base.size = 1;
        effect.reload = 200.0;
        effect.range = 40.0;
        effect.heal_percent = 4.0;
        effect.phase_boost = 4.0;
        effect.phase_range_boost = 20.0;
        effect.base.health = 80;
        push_item_amount(&mut effect.boost_items, items, "silicon", 1);
    });

    registry.register_effect_block("mend-projector", EffectBlockKind::MendProjector, |effect| {
        set_requirements(
            &mut effect.requirements,
            items,
            &[
                ("lead", 100),
                ("titanium", 25),
                ("silicon", 40),
                ("copper", 50),
            ],
        );
        effect.consume_power = 1.5;
        effect.base.has_power = true;
        effect.base.consumes_power = true;
        effect.base.size = 2;
        effect.reload = 250.0;
        effect.range = 85.0;
        effect.heal_percent = 11.0;
        effect.phase_boost = 15.0;
        effect.base.health = 80 * 2 * 2;
        push_item_amount(&mut effect.boost_items, items, "phase-fabric", 1);
    });

    registry.register_effect_block(
        "overdrive-projector",
        EffectBlockKind::OverdriveProjector,
        |effect| {
            set_requirements(
                &mut effect.requirements,
                items,
                &[
                    ("lead", 100),
                    ("titanium", 75),
                    ("silicon", 75),
                    ("plastanium", 30),
                ],
            );
            effect.consume_power = 3.5;
            effect.base.has_power = true;
            effect.base.consumes_power = true;
            effect.base.size = 2;
            push_item_amount(&mut effect.boost_items, items, "phase-fabric", 1);
            effect.ambient_sound_volume = 0.08;
        },
    );

    registry.register_effect_block(
        "overdrive-dome",
        EffectBlockKind::OverdriveProjector,
        |effect| {
            set_requirements(
                &mut effect.requirements,
                items,
                &[
                    ("lead", 200),
                    ("titanium", 130),
                    ("silicon", 130),
                    ("plastanium", 80),
                    ("surge-alloy", 120),
                ],
            );
            effect.consume_power = 10.0;
            effect.base.has_power = true;
            effect.base.consumes_power = true;
            effect.base.size = 3;
            effect.range = 200.0;
            effect.speed_boost = 2.5;
            effect.use_time = 300.0;
            effect.ambient_sound_volume = 0.12;
            effect.has_boost = false;
            push_item_amount(&mut effect.consume_items, items, "phase-fabric", 1);
            push_item_amount(&mut effect.consume_items, items, "silicon", 1);
        },
    );

    registry.register_effect_block(
        "force-projector",
        EffectBlockKind::ForceProjector,
        |effect| {
            set_requirements(
                &mut effect.requirements,
                items,
                &[("lead", 100), ("titanium", 75), ("silicon", 125)],
            );
            effect.base.size = 3;
            effect.phase_radius_boost = 80.0;
            effect.radius = 101.7;
            effect.shield_health = 750.0;
            effect.cooldown_normal = 1.5;
            effect.cooldown_liquid = 1.2;
            effect.cooldown_broken_base = 0.35;
            push_item_amount(&mut effect.boost_items, items, "phase-fabric", 1);
            effect.consume_power = 4.0;
            effect.base.has_power = true;
            effect.base.consumes_power = true;
        },
    );

    registry.register_effect_block("shock-mine", EffectBlockKind::ShockMine, |effect| {
        set_requirements(
            &mut effect.requirements,
            items,
            &[("lead", 25), ("silicon", 12)],
        );
        effect.base.has_shadow = false;
        effect.base.health = 50;
        effect.damage = 25.0;
        effect.tile_damage = 7.0;
        effect.length = 10;
        effect.tendrils = 4;
    });

    registry.register_effect_block("radar", EffectBlockKind::Radar, |effect| {
        set_requirements(
            &mut effect.requirements,
            items,
            &[("silicon", 60), ("graphite", 50), ("beryllium", 10)],
        );
        effect.base.build_visibility = BuildVisibility::FogOnly;
        effect.outline_color = "4a4b53".into();
        effect.fog_radius = 34.0;
        set_requirements(
            &mut effect.research_cost,
            items,
            &[("silicon", 70), ("graphite", 70)],
        );
        effect.consume_power = 0.6;
        effect.base.has_power = true;
        effect.base.consumes_power = true;
    });

    registry.register_effect_block("build-tower", EffectBlockKind::BuildTurret, |effect| {
        set_requirements(
            &mut effect.requirements,
            items,
            &[("silicon", 150), ("oxide", 40), ("thorium", 60)],
        );
        effect.outline_color = "darkOutline".into();
        effect.range = 200.0;
        effect.base.size = 3;
        effect.build_speed = 1.5;
        effect.elevation = effect.base.size as f32 / 2.0;
        effect.consume_power = 3.0;
        effect.base.has_power = true;
        effect.base.has_liquids = true;
        effect.base.consumes_power = true;
        push_liquid_amount(&mut effect.consume_liquids, liquids, "nitrogen", 3.0 / 60.0);
    });

    registry.register_effect_block(
        "regen-projector",
        EffectBlockKind::RegenProjector,
        |effect| {
            set_requirements(
                &mut effect.requirements,
                items,
                &[
                    ("silicon", 80),
                    ("tungsten", 60),
                    ("oxide", 40),
                    ("beryllium", 80),
                ],
            );
            effect.base.size = 3;
            effect.range = 28.0;
            effect.base_color = "regen".into();
            effect.consume_power = 1.0;
            effect.base.has_power = true;
            effect.base.has_liquids = true;
            effect.base.consumes_power = true;
            push_liquid_amount(&mut effect.consume_liquids, liquids, "hydrogen", 1.0 / 60.0);
            push_item_amount(&mut effect.boost_items, items, "phase-fabric", 1);
            effect.heal_percent = 4.0 / 60.0;
            effect.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(hydrogen), DrawDefault, DrawGlowRegion(sky), DrawPulseShape(8ca9e8), DrawShape(8ca9e8))".into();
        },
    );

    // Blocks.java keeps barrierProjector under `if(false)`, so it must not be
    // registered here until upstream enables it.

    registry.register_effect_block(
        "shockwave-tower",
        EffectBlockKind::ShockwaveTower,
        |effect| {
            set_requirements(
                &mut effect.requirements,
                items,
                &[
                    ("surge-alloy", 50),
                    ("silicon", 150),
                    ("oxide", 30),
                    ("tungsten", 100),
                ],
            );
            effect.base.size = 3;
            push_liquid_amount(&mut effect.consume_liquids, liquids, "cyanogen", 1.5 / 60.0);
            effect.consume_power = 100.0 / 60.0;
            effect.base.has_power = true;
            effect.base.has_liquids = true;
            effect.base.consumes_power = true;
            effect.range = 170.0;
            effect.reload = 80.0;
        },
    );

    registry.register_effect_block("shield-projector", EffectBlockKind::BaseShield, |effect| {
        effect.base.build_visibility = BuildVisibility::EditorOnly;
        effect.base.size = 3;
        effect.consume_power = 5.0;
        effect.base.consumes_power = true;
    });

    registry.register_effect_block(
        "large-shield-projector",
        EffectBlockKind::BaseShield,
        |effect| {
            effect.base.build_visibility = BuildVisibility::EditorOnly;
            effect.base.size = 4;
            effect.radius = 400.0;
            effect.consume_power = 5.0;
            effect.base.consumes_power = true;
        },
    );
}

fn register_distribution_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_distribution_block(
        "conveyor",
        DistributionBlockKind::Conveyor,
        |distribution| {
            set_requirements(&mut distribution.requirements, items, &[("copper", 1)]);
            distribution.base.health = 45;
            distribution.speed = 0.03;
            distribution.displayed_speed = 4.2;
            distribution.build_cost_multiplier = 2.0;
            set_requirements(&mut distribution.research_cost, items, &[("copper", 5)]);
        },
    );

    registry.register_distribution_block(
        "titanium-conveyor",
        DistributionBlockKind::Conveyor,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("copper", 1), ("lead", 1), ("titanium", 1)],
            );
            distribution.base.health = 65;
            distribution.speed = 0.08;
            distribution.displayed_speed = 11.0;
        },
    );

    registry.register_distribution_block(
        "plastanium-conveyor",
        DistributionBlockKind::StackConveyor,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("plastanium", 1), ("silicon", 1), ("graphite", 1)],
            );
            distribution.base.health = 90;
            distribution.speed = 4.0 / 60.0;
            distribution.base.item_capacity = 10;
        },
    );

    registry.register_distribution_block(
        "armored-conveyor",
        DistributionBlockKind::ArmoredConveyor,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("plastanium", 1), ("thorium", 1), ("metaglass", 1)],
            );
            distribution.base.health = 280;
            distribution.speed = 0.08;
            distribution.displayed_speed = 11.0;
        },
    );

    registry.register_distribution_block(
        "junction",
        DistributionBlockKind::Junction,
        |distribution| {
            set_requirements(&mut distribution.requirements, items, &[("copper", 3)]);
            distribution.speed = 26.0;
            distribution.capacity = 6;
            distribution.base.health = 30;
            distribution.build_cost_multiplier = 6.0;
        },
    );

    registry.register_distribution_block(
        "bridge-conveyor",
        DistributionBlockKind::BufferedItemBridge,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("lead", 6), ("copper", 6)],
            );
            distribution.fade_in = false;
            distribution.move_arrows = false;
            distribution.range = 4.0;
            distribution.speed = 74.0;
            distribution.arrow_spacing = 6.0;
            distribution.buffer_capacity = 14;
            distribution.crush_fragile = true;
        },
    );

    registry.register_distribution_block(
        "phase-conveyor",
        DistributionBlockKind::ItemBridge,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[
                    ("phase-fabric", 5),
                    ("silicon", 7),
                    ("lead", 10),
                    ("graphite", 10),
                ],
            );
            distribution.range = 12.0;
            distribution.transport_time = 2.0;
            distribution.arrow_period = 0.9;
            distribution.arrow_time_scl = 2.75;
            distribution.base.has_power = true;
            distribution.pulse = true;
            distribution.base.env_enabled |= Env::SPACE;
            distribution.consume_power = 0.30;
            distribution.base.consumes_power = true;
        },
    );

    registry.register_distribution_block("sorter", DistributionBlockKind::Sorter, |distribution| {
        set_requirements(
            &mut distribution.requirements,
            items,
            &[("lead", 2), ("copper", 2)],
        );
        distribution.build_cost_multiplier = 3.0;
    });

    registry.register_distribution_block(
        "inverted-sorter",
        DistributionBlockKind::Sorter,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("lead", 2), ("copper", 2)],
            );
            distribution.build_cost_multiplier = 3.0;
            distribution.invert = true;
        },
    );

    registry.register_distribution_block("router", DistributionBlockKind::Router, |distribution| {
        set_requirements(&mut distribution.requirements, items, &[("copper", 3)]);
        distribution.build_cost_multiplier = 4.0;
    });

    registry.register_distribution_block(
        "distributor",
        DistributionBlockKind::Router,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("lead", 4), ("copper", 4)],
            );
            distribution.build_cost_multiplier = 3.0;
            distribution.base.size = 2;
        },
    );

    registry.register_distribution_block(
        "overflow-gate",
        DistributionBlockKind::OverflowGate,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("lead", 2), ("copper", 4)],
            );
            distribution.build_cost_multiplier = 3.0;
        },
    );

    registry.register_distribution_block(
        "underflow-gate",
        DistributionBlockKind::OverflowGate,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("lead", 2), ("copper", 4)],
            );
            distribution.build_cost_multiplier = 3.0;
            distribution.invert = true;
        },
    );

    registry.register_distribution_block(
        "unloader",
        DistributionBlockKind::Unloader,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("titanium", 25), ("silicon", 30)],
            );
            distribution.speed = 60.0 / 11.0;
            distribution.base.group = BlockGroup::Transportation;
        },
    );

    registry.register_distribution_block(
        "mass-driver",
        DistributionBlockKind::MassDriver,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[
                    ("titanium", 125),
                    ("silicon", 75),
                    ("lead", 125),
                    ("thorium", 50),
                ],
            );
            distribution.base.size = 3;
            distribution.base.item_capacity = 120;
            distribution.reload = 200.0;
            distribution.range = 440.0;
            distribution.consume_power = 1.75;
            distribution.base.consumes_power = true;
        },
    );

    registry.register_distribution_block("duct", DistributionBlockKind::Duct, |distribution| {
        set_requirements(&mut distribution.requirements, items, &[("beryllium", 1)]);
        distribution.base.health = 90;
        distribution.speed = 4.0;
        set_requirements(&mut distribution.research_cost, items, &[("beryllium", 5)]);
    });

    registry.register_distribution_block(
        "armored-duct",
        DistributionBlockKind::Duct,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("beryllium", 2), ("tungsten", 1)],
            );
            distribution.base.health = 140;
            distribution.speed = 4.0;
            distribution.armored = true;
            set_requirements(
                &mut distribution.research_cost,
                items,
                &[("beryllium", 300), ("tungsten", 100)],
            );
        },
    );

    registry.register_distribution_block(
        "duct-router",
        DistributionBlockKind::DuctRouter,
        |distribution| {
            set_requirements(&mut distribution.requirements, items, &[("beryllium", 10)]);
            distribution.base.health = 90;
            distribution.speed = 4.0;
            distribution.region_rotated1 = 1;
            distribution.base.solid = false;
            set_requirements(&mut distribution.research_cost, items, &[("beryllium", 30)]);
        },
    );

    registry.register_distribution_block(
        "overflow-duct",
        DistributionBlockKind::OverflowDuct,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("graphite", 8), ("beryllium", 8)],
            );
            distribution.base.health = 90;
            distribution.speed = 4.0;
            distribution.base.solid = false;
            distribution.research_cost_multiplier = 1.5;
        },
    );

    registry.register_distribution_block(
        "underflow-duct",
        DistributionBlockKind::OverflowDuct,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("graphite", 8), ("beryllium", 8)],
            );
            distribution.base.health = 90;
            distribution.speed = 4.0;
            distribution.base.solid = false;
            distribution.research_cost_multiplier = 1.5;
            distribution.invert = true;
        },
    );

    registry.register_distribution_block(
        "duct-bridge",
        DistributionBlockKind::DuctBridge,
        |distribution| {
            set_requirements(&mut distribution.requirements, items, &[("beryllium", 15)]);
            distribution.base.health = 90;
            distribution.speed = 4.0;
            distribution.build_cost_multiplier = 2.0;
            distribution.research_cost_multiplier = 0.3;
            distribution.crush_fragile = true;
        },
    );

    registry.register_distribution_block(
        "duct-unloader",
        DistributionBlockKind::DirectionalUnloader,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("graphite", 20), ("silicon", 20), ("tungsten", 10)],
            );
            distribution.base.health = 120;
            distribution.speed = 4.0;
            distribution.base.solid = false;
            distribution.under_bullets = true;
            distribution.region_rotated1 = 1;
        },
    );

    registry.register_distribution_block(
        "surge-conveyor",
        DistributionBlockKind::StackConveyor,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("surge-alloy", 1), ("tungsten", 1)],
            );
            distribution.base.health = 130;
            distribution.speed = 5.0 / 60.0;
            distribution.base.item_capacity = 10;
            distribution.output_router = false;
            distribution.base.has_power = true;
            distribution.base.consumes_power = true;
            distribution.base.conductive_power = true;
            distribution.under_bullets = true;
            distribution.base_efficiency = 1.0;
            distribution.consume_power = 1.0 / 60.0;
            set_requirements(
                &mut distribution.research_cost,
                items,
                &[("surge-alloy", 30), ("tungsten", 80)],
            );
        },
    );

    registry.register_distribution_block(
        "surge-router",
        DistributionBlockKind::StackRouter,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("surge-alloy", 5), ("tungsten", 1)],
            );
            distribution.base.health = 130;
            distribution.speed = 6.0;
            distribution.base.has_power = true;
            distribution.base.consumes_power = true;
            distribution.base.conductive_power = true;
            distribution.base_efficiency = 1.0;
            distribution.under_bullets = true;
            distribution.base.solid = false;
            distribution.consume_power = 3.0 / 60.0;
        },
    );

    registry.register_distribution_block(
        "unit-cargo-loader",
        DistributionBlockKind::UnitCargoLoader,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("silicon", 80), ("surge-alloy", 50), ("oxide", 20)],
            );
            distribution.base.size = 3;
            distribution.unit_build_time = 60.0 * 8.0;
            distribution.consume_power = 8.0 / 60.0;
            distribution.base.has_power = true;
            distribution.base.has_liquids = true;
            distribution.base.consumes_power = true;
            push_liquid_amount(
                &mut distribution.consume_liquids,
                liquids,
                "nitrogen",
                10.0 / 60.0,
            );
            distribution.base.item_capacity = 200;
            set_requirements(
                &mut distribution.research_cost,
                items,
                &[("silicon", 2500), ("surge-alloy", 20), ("oxide", 30)],
            );
        },
    );

    registry.register_distribution_block(
        "unit-cargo-unload-point",
        DistributionBlockKind::UnitCargoUnloadPoint,
        |distribution| {
            set_requirements(
                &mut distribution.requirements,
                items,
                &[("silicon", 60), ("tungsten", 60)],
            );
            distribution.base.size = 2;
            distribution.base.item_capacity = 100;
            set_requirements(
                &mut distribution.research_cost,
                items,
                &[("silicon", 3000), ("oxide", 20)],
            );
        },
    );
}

fn register_liquid_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_liquid_block("mechanical-pump", LiquidBlockKind::Pump, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[("copper", 15), ("metaglass", 10)],
        );
        liquid.pump_amount = 7.0 / 60.0;
        liquid.base.liquid_capacity = 20.0;
    });

    registry.register_liquid_block("rotary-pump", LiquidBlockKind::Pump, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[
                ("copper", 70),
                ("metaglass", 50),
                ("silicon", 20),
                ("titanium", 35),
            ],
        );
        liquid.pump_amount = 0.2;
        liquid.consume_power = 0.3;
        liquid.base.consumes_power = true;
        liquid.base.liquid_capacity = 80.0;
        liquid.base.has_power = true;
        liquid.base.size = 2;
    });

    registry.register_liquid_block("impulse-pump", LiquidBlockKind::Pump, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[
                ("copper", 80),
                ("metaglass", 90),
                ("silicon", 30),
                ("titanium", 40),
                ("thorium", 35),
            ],
        );
        liquid.pump_amount = 0.22;
        liquid.consume_power = 1.3;
        liquid.base.consumes_power = true;
        liquid.base.liquid_capacity = 200.0;
        liquid.base.has_power = true;
        liquid.base.size = 3;
    });

    registry.register_liquid_block("conduit", LiquidBlockKind::Conduit, |liquid| {
        set_requirements(&mut liquid.requirements, items, &[("metaglass", 1)]);
        liquid.base.liquid_capacity = 20.0;
        liquid.base.health = 45;
        liquid.explosiveness_scale = 10.0 / 20.0;
        liquid.flammability_scale = 10.0 / 20.0;
    });

    registry.register_liquid_block("pulse-conduit", LiquidBlockKind::Conduit, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[("titanium", 2), ("metaglass", 1)],
        );
        liquid.base.liquid_capacity = 40.0;
        liquid.liquid_pressure = 1.025;
        liquid.base.health = 90;
        liquid.explosiveness_scale = 16.0 / 40.0;
        liquid.flammability_scale = 16.0 / 40.0;
    });

    registry.register_liquid_block(
        "plated-conduit",
        LiquidBlockKind::ArmoredConduit,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("thorium", 2), ("metaglass", 1), ("plastanium", 1)],
            );
            liquid.base.liquid_capacity = 50.0;
            liquid.liquid_pressure = 1.025;
            liquid.base.health = 220;
            liquid.explosiveness_scale = 16.0 / 50.0;
            liquid.flammability_scale = 16.0 / 50.0;
        },
    );

    registry.register_liquid_block("liquid-router", LiquidBlockKind::LiquidRouter, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[("graphite", 4), ("metaglass", 2)],
        );
        liquid.base.liquid_capacity = 120.0;
        liquid.under_bullets = true;
        liquid.base.solid = false;
        liquid.explosiveness_scale = 20.0 / 120.0;
        liquid.flammability_scale = 20.0 / 120.0;
    });

    registry.register_liquid_block(
        "liquid-container",
        LiquidBlockKind::LiquidRouter,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("titanium", 10), ("metaglass", 15)],
            );
            liquid.base.liquid_capacity = 700.0;
            liquid.base.size = 2;
            liquid.base.solid = true;
        },
    );

    registry.register_liquid_block("liquid-tank", LiquidBlockKind::LiquidRouter, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[("titanium", 30), ("metaglass", 40)],
        );
        liquid.base.size = 3;
        liquid.base.solid = true;
        liquid.base.liquid_capacity = 1800.0;
        liquid.base.health = 500;
    });

    registry.register_liquid_block(
        "liquid-junction",
        LiquidBlockKind::LiquidJunction,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("graphite", 4), ("metaglass", 8)],
            );
            liquid.base.solid = false;
        },
    );

    registry.register_liquid_block("bridge-conduit", LiquidBlockKind::LiquidBridge, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[("graphite", 4), ("metaglass", 8)],
        );
        liquid.floating = true;
        liquid.fade_in = false;
        liquid.move_arrows = false;
        liquid.arrow_spacing = 6.0;
        liquid.range = 4.0;
        liquid.base.has_power = false;
        liquid.base.liquid_capacity = 100.0;
        liquid.explosiveness_scale = 20.0 / 100.0;
        liquid.flammability_scale = 20.0 / 100.0;
    });

    registry.register_liquid_block("phase-conduit", LiquidBlockKind::LiquidBridge, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[
                ("phase-fabric", 5),
                ("silicon", 7),
                ("metaglass", 20),
                ("titanium", 10),
            ],
        );
        liquid.floating = true;
        liquid.range = 12.0;
        liquid.arrow_period = 0.9;
        liquid.arrow_time_scl = 2.75;
        liquid.base.has_power = true;
        liquid.can_overdrive = false;
        liquid.pulse = true;
        liquid.explosiveness_scale = 20.0 / 100.0;
        liquid.flammability_scale = 20.0 / 100.0;
        liquid.base.liquid_capacity = 100.0;
        liquid.consume_power = 0.30;
        liquid.base.consumes_power = true;
    });

    registry.register_liquid_block("reinforced-pump", LiquidBlockKind::Pump, |liquid| {
        set_requirements(
            &mut liquid.requirements,
            items,
            &[("beryllium", 40), ("tungsten", 30), ("silicon", 20)],
        );
        push_liquid_amount(&mut liquid.consume_liquids, liquids, "hydrogen", 1.5 / 60.0);
        liquid.base.has_liquids = true;
        liquid.pump_amount = 80.0 / 60.0 / 4.0;
        liquid.base.liquid_capacity = 160.0;
        liquid.base.size = 2;
    });

    let reinforced_conduit = registry.register_liquid_block(
        "reinforced-conduit",
        LiquidBlockKind::ArmoredConduit,
        |liquid| {
            set_requirements(&mut liquid.requirements, items, &[("beryllium", 2)]);
            liquid.bot_color = "darkestMetal".into();
            liquid.leaks = true;
            liquid.base.liquid_capacity = 50.0;
            liquid.liquid_pressure = 1.03;
            liquid.base.health = 250;
            liquid.research_cost_multiplier = 3.0;
            liquid.under_bullets = true;
            liquid.explosiveness_scale = 20.0 / 50.0;
            liquid.flammability_scale = 20.0 / 50.0;
        },
    );

    let reinforced_junction = registry.register_liquid_block(
        "reinforced-liquid-junction",
        LiquidBlockKind::LiquidJunction,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("graphite", 4), ("beryllium", 8)],
            );
            liquid.build_cost_multiplier = 3.0;
            liquid.base.health = 250;
            liquid.research_cost_multiplier = 1.0;
            liquid.base.solid = false;
            liquid.under_bullets = true;
        },
    );

    let reinforced_bridge = registry.register_liquid_block(
        "reinforced-bridge-conduit",
        LiquidBlockKind::DirectionLiquidBridge,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("graphite", 8), ("beryllium", 20)],
            );
            liquid.range = 4.0;
            liquid.base.has_power = false;
            liquid.base.liquid_capacity = 120.0;
            liquid.research_cost_multiplier = 1.0;
            liquid.under_bullets = true;
            liquid.base.health = 250;
            liquid.explosiveness_scale = 20.0 / 120.0;
            liquid.flammability_scale = 20.0 / 120.0;
        },
    );

    if let Some(BlockDef::Liquid(conduit)) = registry.get_mut(reinforced_conduit) {
        conduit.junction_replacement = Some(reinforced_junction);
        conduit.rot_bridge_replacement = Some(reinforced_bridge);
    }

    registry.register_liquid_block(
        "reinforced-liquid-router",
        LiquidBlockKind::LiquidRouter,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("graphite", 8), ("beryllium", 4)],
            );
            liquid.base.liquid_capacity = 150.0;
            liquid.liquid_padding = 3.0 / 4.0;
            liquid.research_cost_multiplier = 3.0;
            liquid.under_bullets = true;
            liquid.base.solid = false;
            liquid.base.health = 250;
            liquid.explosiveness_scale = 40.0 / 150.0;
            liquid.flammability_scale = 40.0 / 150.0;
        },
    );

    registry.register_liquid_block(
        "reinforced-liquid-container",
        LiquidBlockKind::LiquidRouter,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("tungsten", 10), ("beryllium", 16)],
            );
            liquid.base.liquid_capacity = 1000.0;
            liquid.base.size = 2;
            liquid.liquid_padding = 6.0 / 4.0;
            liquid.research_cost_multiplier = 4.0;
            liquid.base.solid = true;
            liquid.base.health = 400;
        },
    );

    registry.register_liquid_block(
        "reinforced-liquid-tank",
        LiquidBlockKind::LiquidRouter,
        |liquid| {
            set_requirements(
                &mut liquid.requirements,
                items,
                &[("tungsten", 40), ("beryllium", 50)],
            );
            liquid.base.size = 3;
            liquid.base.solid = true;
            liquid.base.liquid_capacity = 2700.0;
            liquid.liquid_padding = 2.0;
            liquid.base.health = 900;
        },
    );
}

fn register_power_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_power_block("power-node", PowerBlockKind::PowerNode, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[("copper", 2), ("lead", 6)],
        );
        power.max_nodes = 10;
        power.laser_range = 6.0;
        power.under_bullets = true;
        power.crush_fragile = true;
    });

    registry.register_power_block("power-node-large", PowerBlockKind::PowerNode, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[("titanium", 5), ("lead", 10), ("silicon", 3)],
        );
        power.base.size = 2;
        power.max_nodes = 15;
        power.laser_range = 15.0;
    });

    registry.register_power_block("surge-tower", PowerBlockKind::PowerNode, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[
                ("titanium", 7),
                ("lead", 10),
                ("silicon", 15),
                ("surge-alloy", 15),
            ],
        );
        power.base.size = 2;
        power.max_nodes = 2;
        power.laser_range = 40.0;
        power.schematic_priority = -15;
    });

    registry.register_power_block("diode", PowerBlockKind::PowerDiode, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[("silicon", 10), ("plastanium", 5), ("metaglass", 10)],
        );
    });

    registry.register_power_block("battery", PowerBlockKind::Battery, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[("copper", 5), ("lead", 20)],
        );
        power.buffered_power = 4000.0;
        power.base_explosiveness = 1.0;
    });

    registry.register_power_block("battery-large", PowerBlockKind::Battery, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[("titanium", 20), ("lead", 50), ("silicon", 30)],
        );
        power.base.size = 3;
        power.buffered_power = 50000.0;
        power.base_explosiveness = 5.0;
    });

    registry.register_power_block(
        "combustion-generator",
        PowerBlockKind::ConsumeGenerator,
        |power| {
            set_requirements(
                &mut power.requirements,
                items,
                &[("copper", 25), ("lead", 15)],
            );
            power.power_production = 1.0;
            power.item_duration = 120.0;
            power.ambient_sound = "loopSmelter".into();
            power.ambient_sound_volume = 0.03;
            power.generate_effect = "generatespark".into();
            push_item_amount(&mut power.item_duration_multipliers, items, "pyratite", 3);
            power.drawer = "DrawMulti(DrawDefault, DrawWarmupRegion)".into();
        },
    );

    registry.register_power_block(
        "thermal-generator",
        PowerBlockKind::ThermalGenerator,
        |power| {
            set_requirements(
                &mut power.requirements,
                items,
                &[
                    ("copper", 40),
                    ("graphite", 35),
                    ("lead", 50),
                    ("silicon", 35),
                    ("metaglass", 40),
                ],
            );
            power.power_production = 1.8;
            power.generate_effect = "redgeneratespark".into();
            power.effect_chance = 0.011;
            power.base.size = 2;
            power.floating = true;
            power.ambient_sound = "loopHum".into();
            power.ambient_sound_volume = 0.06;
        },
    );

    registry.register_power_block("steam-generator", PowerBlockKind::ConsumeGenerator, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[
                ("copper", 35),
                ("graphite", 25),
                ("lead", 40),
                ("silicon", 30),
            ],
        );
        power.power_production = 5.5;
        power.item_duration = 90.0;
        push_liquid_amount(&mut power.consume_liquids, liquids, "water", 0.1);
        power.base.has_liquids = true;
        power.base.size = 2;
        power.generate_effect = "generatespark".into();
        power.ambient_sound = "loopSmelter".into();
        power.ambient_sound_volume = 0.06;
        push_item_amount(&mut power.item_duration_multipliers, items, "pyratite", 3);
        power.drawer = "DrawMulti(DrawDefault, DrawWarmupRegion, DrawRegion(-turbine,2), DrawRegion(-turbine,-2), DrawRegion(-cap), DrawLiquidRegion)".into();
    });

    registry.register_power_block(
        "differential-generator",
        PowerBlockKind::ConsumeGenerator,
        |power| {
            set_requirements(
                &mut power.requirements,
                items,
                &[
                    ("copper", 70),
                    ("titanium", 50),
                    ("lead", 100),
                    ("silicon", 65),
                    ("metaglass", 50),
                ],
            );
            power.power_production = 18.0;
            power.item_duration = 220.0;
            power.base.has_liquids = true;
            power.base.has_items = true;
            power.base.size = 3;
            power.ambient_sound = "loopDifferential".into();
            power.generate_effect = "generatespark".into();
            power.ambient_sound_volume = 0.12;
            power.drawer = "DrawMulti(DrawDefault, DrawWarmupRegion, DrawLiquidRegion)".into();
            push_item_amount(&mut power.consume_items, items, "pyratite", 1);
            push_liquid_amount(&mut power.consume_liquids, liquids, "cryofluid", 0.1);
        },
    );

    registry.register_power_block("rtg-generator", PowerBlockKind::ConsumeGenerator, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[
                ("lead", 100),
                ("silicon", 75),
                ("phase-fabric", 25),
                ("plastanium", 75),
                ("thorium", 50),
            ],
        );
        power.base.size = 2;
        power.power_production = 4.5;
        power.item_duration = 60.0 * 14.0;
        power.base.env_enabled = Env::ANY;
        power.generate_effect = "generatespark".into();
        push_item_amount(
            &mut power.item_duration_multipliers,
            items,
            "phase-fabric",
            15,
        );
        power.drawer = "DrawMulti(DrawDefault, DrawWarmupRegion)".into();
    });

    registry.register_power_block("solar-panel", PowerBlockKind::SolarGenerator, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[("lead", 10), ("silicon", 8)],
        );
        power.power_production = 0.12;
    });

    registry.register_power_block(
        "solar-panel-large",
        PowerBlockKind::SolarGenerator,
        |power| {
            set_requirements(
                &mut power.requirements,
                items,
                &[("lead", 60), ("silicon", 70), ("phase-fabric", 15)],
            );
            power.base.size = 3;
            power.power_production = 1.6;
        },
    );

    registry.register_power_block("thorium-reactor", PowerBlockKind::NuclearReactor, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[
                ("lead", 300),
                ("silicon", 200),
                ("graphite", 150),
                ("thorium", 150),
                ("metaglass", 50),
            ],
        );
        power.ambient_sound = "loopThoriumReactor".into();
        power.ambient_sound_volume = 0.11;
        power.base.size = 3;
        power.base.health = 700;
        power.item_duration = 360.0;
        power.power_production = 15.0;
        power.heating = 0.02;
        power.coolant_power = 0.5;
        push_item_amount(&mut power.consume_items, items, "thorium", 1);
        push_liquid_amount(
            &mut power.consume_liquids,
            liquids,
            "cryofluid",
            power.heating / power.coolant_power,
        );
    });

    registry.register_power_block("impact-reactor", PowerBlockKind::ImpactReactor, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[
                ("lead", 500),
                ("silicon", 300),
                ("graphite", 400),
                ("thorium", 100),
                ("surge-alloy", 250),
                ("metaglass", 250),
            ],
        );
        power.base.size = 4;
        power.base.health = 900;
        power.power_production = 130.0;
        power.item_duration = 140.0;
        power.ambient_sound = "loopPulse".into();
        power.ambient_sound_volume = 0.08;
        power.base.liquid_capacity = 80.0;
        power.liquid_capacity = 80.0;
        power.consume_power = 25.0;
        push_item_amount(&mut power.consume_items, items, "blast-compound", 1);
        push_liquid_amount(&mut power.consume_liquids, liquids, "cryofluid", 0.25);
    });

    registry.register_power_block("beam-node", PowerBlockKind::BeamNode, |power| {
        set_requirements(&mut power.requirements, items, &[("beryllium", 8)]);
        power.base.consumes_power = true;
        power.base.outputs_power = true;
        power.base.health = 90;
        power.range = 10.0;
        power.fog_radius = 1.0;
        set_requirements(&mut power.research_cost, items, &[("beryllium", 5)]);
        power.crush_fragile = true;
        power.buffered_power = 1000.0;
    });

    registry.register_power_block("beam-tower", PowerBlockKind::BeamNode, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[("beryllium", 30), ("oxide", 10), ("silicon", 10)],
        );
        power.base.size = 3;
        power.base.consumes_power = true;
        power.base.outputs_power = true;
        power.range = 23.0;
        power.health_scaled = 90.0;
        power.fog_radius = 2.0;
        power.buffered_power = 40000.0;
    });

    registry.register_power_block("beam-link", PowerBlockKind::LongPowerNode, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[
                ("beryllium", 250),
                ("silicon", 250),
                ("oxide", 150),
                ("carbide", 75),
                ("surge-alloy", 75),
                ("phase-fabric", 75),
            ],
        );
        power.base.size = 3;
        power.max_nodes = 1;
        power.laser_range = 500.0;
        power.power_layer = "Layer.legUnit+2".into();
        power.autolink = false;
        power.same_block_connection = true;
        power.laser_color2 = "ffd9c2".into();
        power.laser_scale = 0.8;
        power.health_scaled = 130.0;
    });

    registry.register_power_block(
        "turbine-condenser",
        PowerBlockKind::ThermalGenerator,
        |power| {
            set_requirements(&mut power.requirements, items, &[("beryllium", 60)]);
            power.attribute = "steam".into();
            power.base.group = BlockGroup::Liquids;
            power.display_efficiency_scale = 1.0 / 9.0;
            power.min_efficiency = 9.0 - 0.0001;
            power.power_production = 3.0 / 9.0;
            power.display_efficiency = false;
            power.generate_effect = "turbinegenerate".into();
            power.effect_chance = 0.04;
            power.base.size = 3;
            power.ambient_sound = "loopHum".into();
            power.ambient_sound_volume = 0.06;
            power.drawer = "DrawMulti(DrawDefault, DrawBlurSpin(-rotator))".into();
            power.base.has_liquids = true;
            power.output_liquid = liquid_amount(liquids, "water", 5.0 / 60.0 / 9.0);
            power.base.liquid_capacity = 20.0;
            power.liquid_capacity = 20.0;
            power.fog_radius = 3.0;
            set_requirements(&mut power.research_cost, items, &[("beryllium", 15)]);
        },
    );

    registry.register_power_block(
        "chemical-combustion-chamber",
        PowerBlockKind::ConsumeGenerator,
        |power| {
            set_requirements(
                &mut power.requirements,
                items,
                &[("graphite", 40), ("tungsten", 20), ("oxide", 40), ("silicon", 30)],
            );
            power.power_production = 550.0 / 60.0;
            set_requirements(
                &mut power.research_cost,
                items,
                &[("graphite", 2000), ("tungsten", 1000), ("oxide", 10), ("silicon", 1500)],
            );
            push_liquid_amount(&mut power.consume_liquids, liquids, "ozone", 2.0 / 60.0);
            push_liquid_amount(&mut power.consume_liquids, liquids, "arkycite", 40.0 / 60.0);
            power.base.has_liquids = true;
            power.base.size = 3;
            power.drawer = "DrawMulti(DrawRegion(-bottom), DrawPistons, DrawRegion(-mid), DrawLiquidTile(arkycite), DrawDefault, DrawGlowRegion)".into();
            power.generate_effect = "none".into();
            power.base.liquid_capacity = 20.0 * 5.0;
            power.liquid_capacity = 20.0 * 5.0;
            power.ambient_sound = "loopSmelter".into();
            power.ambient_sound_volume = 0.06;
        },
    );

    registry.register_power_block(
        "pyrolysis-generator",
        PowerBlockKind::ConsumeGenerator,
        |power| {
            set_requirements(
                &mut power.requirements,
                items,
                &[
                    ("graphite", 100),
                    ("carbide", 60),
                    ("oxide", 60),
                    ("silicon", 100),
                ],
            );
            power.power_production = 1400.0 / 60.0;
            power.drawer = "DrawMulti(DrawRegion(-bottom), DrawPistons, DrawRegion(-mid), DrawLiquidTile(arkycite), DrawDefault, DrawGlowRegion)".into();
            push_liquid_amount(&mut power.consume_liquids, liquids, "slag", 20.0 / 60.0);
            push_liquid_amount(&mut power.consume_liquids, liquids, "arkycite", 40.0 / 60.0);
            power.base.has_liquids = true;
            power.base.size = 3;
            power.base.liquid_capacity = 30.0 * 5.0;
            power.liquid_capacity = 30.0 * 5.0;
            power.output_liquid = liquid_amount(liquids, "water", 20.0 / 60.0);
            power.generate_effect = "none".into();
            power.ambient_sound = "loopSmelter".into();
            power.ambient_sound_volume = 0.06;
            power.research_cost_multiplier = 0.4;
        },
    );

    registry.register_power_block("flux-reactor", PowerBlockKind::VariableReactor, |power| {
        set_requirements(
            &mut power.requirements,
            items,
            &[
                ("graphite", 240),
                ("carbide", 60),
                ("oxide", 80),
                ("silicon", 480),
                ("surge-alloy", 120),
            ],
        );
        power.power_production = 18000.0 / 60.0;
        power.max_heat = 150.0;
        push_liquid_amount(&mut power.consume_liquids, liquids, "cyanogen", 9.0 / 60.0);
        power.base.has_liquids = true;
        power.base.liquid_capacity = 30.0;
        power.liquid_capacity = 30.0;
        power.explosion_min_warmup = 0.5;
        power.explosion_radius = 17;
        power.explosion_damage = 2500;
        power.explosion_puddle_liquid = liquid_id(liquids, "slag");
        power.ambient_sound = "loopFlux".into();
        power.ambient_sound_volume = 0.15;
        power.base.size = 5;
        power.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(cyanogen), DrawRegion(-mid), DrawSoftParticles, DrawDefault, DrawHeatInput, DrawGlowRegion)".into();
    });

    registry.register_power_block(
        "neoplasia-reactor",
        PowerBlockKind::HeaterGenerator,
        |power| {
            set_requirements(
                &mut power.requirements,
                items,
                &[
                    ("tungsten", 750),
                    ("carbide", 300),
                    ("oxide", 150),
                    ("silicon", 500),
                    ("phase-fabric", 150),
                    ("surge-alloy", 200),
                ],
            );
            power.base.size = 5;
            power.base.liquid_capacity = 80.0;
            power.liquid_capacity = 80.0;
            power.output_liquid = liquid_amount(liquids, "neoplasm", 20.0 / 60.0);
            power.explode_on_full = true;
            power.heat_output = 60.0;
            push_liquid_amount(&mut power.consume_liquids, liquids, "arkycite", 80.0 / 60.0);
            push_liquid_amount(&mut power.consume_liquids, liquids, "water", 10.0 / 60.0);
            push_item_amount(&mut power.consume_items, items, "phase-fabric", 1);
            power.base.has_liquids = true;
            power.base.has_items = true;
            power.item_duration = 60.0 * 3.0;
            power.item_capacity = 10;
            power.base.item_capacity = 10;
            power.explosion_radius = 9;
            power.explosion_damage = 2000;
            power.power_production = 140.0;
            power.ambient_sound = "loopBio".into();
            power.ambient_sound_volume = 0.2;
            power.explosion_puddles = 80;
            power.explosion_puddle_range = TILE_SIZE as f32 * 7.0;
            power.explosion_puddle_liquid = liquid_id(liquids, "neoplasm");
            power.explosion_puddle_amount = 200.0;
            power.explosion_min_warmup = 0.5;
            power.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(arkycite), DrawCircles, DrawRegion(-center), DrawCells, DrawDefault, DrawHeatOutput)".into();
        },
    );
}

fn register_crafting_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_crafting(
        "graphite-press",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_items = true;
            craft.base.size = 2;
            craft.craft_effect = "pulverizeMedium".into();
            craft.output_item = item_amount(items, "graphite", 1);
            craft.craft_time = 90.0;
            if let Some(coal) = item_amount(items, "coal", 2) {
                craft.consume_items.push(coal);
            }
        },
    );

    registry.register_crafting("multi-press", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_items = true;
        craft.base.has_liquids = true;
        craft.base.has_power = true;
        craft.base.item_capacity = 20;
        craft.base.size = 3;
        craft.craft_effect = "pulverizeMedium".into();
        craft.output_item = item_amount(items, "graphite", 2);
        craft.craft_time = 30.0;
        craft.consume_power = 1.8;
        if let Some(coal) = item_amount(items, "coal", 3) {
            craft.consume_items.push(coal);
        }
        if let Some(water) = liquid_amount(liquids, "water", 0.1) {
            craft.consume_liquids.push(water);
        }
    });

    registry.register_crafting(
        "silicon-smelter",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_power = true;
            craft.base.has_liquids = false;
            craft.base.size = 2;
            craft.craft_effect = "smeltsmoke".into();
            craft.output_item = item_amount(items, "silicon", 1);
            craft.craft_time = 40.0;
            craft.drawer = "DrawMulti(DrawDefault, DrawFlame(ffef99))".into();
            craft.ambient_sound = "loopSmelter".into();
            craft.ambient_sound_volume = 0.07;
            for (name, amount) in [("coal", 1), ("sand", 2)] {
                if let Some(item) = item_amount(items, name, amount) {
                    craft.consume_items.push(item);
                }
            }
            craft.consume_power = 0.50;
        },
    );

    registry.register_crafting(
        "silicon-crucible",
        CraftingBlockKind::AttributeCrafter,
        |craft| {
            craft.base.has_power = true;
            craft.base.has_liquids = false;
            craft.base.item_capacity = 30;
            craft.base.size = 3;
            craft.craft_effect = "smeltsmoke".into();
            craft.output_item = item_amount(items, "silicon", 8);
            craft.craft_time = 90.0;
            craft.boost_scale = 0.15;
            craft.drawer = "DrawMulti(DrawDefault, DrawFlame(ffef99))".into();
            craft.ambient_sound = "loopSmelter".into();
            craft.ambient_sound_volume = 0.07;
            for (name, amount) in [("coal", 4), ("sand", 6), ("pyratite", 1)] {
                if let Some(item) = item_amount(items, name, amount) {
                    craft.consume_items.push(item);
                }
            }
            craft.consume_power = 4.0;
        },
    );

    registry.register_crafting("kiln", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_items = true;
        craft.base.has_power = true;
        craft.base.size = 2;
        craft.craft_effect = "smeltsmoke".into();
        craft.output_item = item_amount(items, "metaglass", 1);
        craft.craft_time = 30.0;
        craft.drawer = "DrawMulti(DrawDefault, DrawFlame(ffc099))".into();
        craft.ambient_sound = "loopSmelter".into();
        craft.ambient_sound_volume = 0.07;
        for (name, amount) in [("lead", 1), ("sand", 1)] {
            if let Some(item) = item_amount(items, name, amount) {
                craft.consume_items.push(item);
            }
        }
        craft.consume_power = 0.60;
    });

    registry.register_crafting(
        "plastanium-compressor",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_items = true;
            craft.base.has_liquids = true;
            craft.base.has_power = true;
            craft.base.liquid_capacity = 60.0;
            craft.base.health = 320;
            craft.base.size = 2;
            craft.craft_effect = "formsmoke".into();
            craft.update_effect = "plasticburn".into();
            craft.output_item = item_amount(items, "plastanium", 1);
            craft.craft_time = 60.0;
            craft.drawer = "DrawMulti(DrawDefault, DrawFade)".into();
            if let Some(oil) = liquid_amount(liquids, "oil", 0.25) {
                craft.consume_liquids.push(oil);
            }
            if let Some(titanium) = item_amount(items, "titanium", 2) {
                craft.consume_items.push(titanium);
            }
            craft.consume_power = 3.0;
        },
    );

    registry.register_crafting("phase-weaver", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_power = true;
        craft.base.item_capacity = 30;
        craft.base.size = 2;
        craft.base.env_enabled |= Env::SPACE;
        craft.craft_effect = "smeltsmoke".into();
        craft.output_item = item_amount(items, "phase-fabric", 1);
        craft.craft_time = 120.0;
        craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawWeave, DrawDefault)".into();
        craft.ambient_sound = "loopTech".into();
        craft.ambient_sound_volume = 0.02;
        for (name, amount) in [("thorium", 4), ("sand", 10)] {
            if let Some(item) = item_amount(items, name, amount) {
                craft.consume_items.push(item);
            }
        }
        craft.consume_power = 5.0;
    });

    registry.register_crafting(
        "surge-smelter",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_power = true;
            craft.base.item_capacity = 20;
            craft.base.size = 3;
            craft.craft_effect = "smeltsmoke".into();
            craft.output_item = item_amount(items, "surge-alloy", 1);
            craft.craft_time = 75.0;
            craft.drawer = "DrawMulti(DrawDefault, DrawFlame)".into();
            for (name, amount) in [("copper", 3), ("lead", 4), ("titanium", 2), ("silicon", 3)] {
                if let Some(item) = item_amount(items, name, amount) {
                    craft.consume_items.push(item);
                }
            }
            craft.consume_power = 4.0;
        },
    );

    registry.register_crafting(
        "cryofluid-mixer",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_items = true;
            craft.base.has_liquids = true;
            craft.base.has_power = true;
            craft.base.liquid_capacity = 36.0;
            craft.base.size = 2;
            craft.base.solid = true;
            craft.base.env_enabled = Env::ANY;
            craft.output_liquid = liquid_amount(liquids, "cryofluid", 12.0 / 60.0);
            craft.craft_time = 120.0;
            craft.outputs_liquid = true;
            craft.rotate = false;
            craft.light_liquid = liquid_id(liquids, "cryofluid");
            craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(water), DrawLiquidTile(cryofluid), DrawDefault)".into();
            if let Some(titanium) = item_amount(items, "titanium", 1) {
                craft.consume_items.push(titanium);
            }
            if let Some(water) = liquid_amount(liquids, "water", 12.0 / 60.0) {
                craft.consume_liquids.push(water);
            }
            craft.consume_power = 1.0;
        },
    );

    registry.register_crafting(
        "pyratite-mixer",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_items = true;
            craft.base.has_power = true;
            craft.base.size = 2;
            craft.base.env_enabled |= Env::SPACE;
            craft.output_item = item_amount(items, "pyratite", 1);
            craft.ambient_sound = "loopMachineSpin".into();
            craft.ambient_sound_volume = 0.1;
            for (name, amount) in [("coal", 1), ("lead", 2), ("sand", 2)] {
                if let Some(item) = item_amount(items, name, amount) {
                    craft.consume_items.push(item);
                }
            }
            craft.consume_power = 0.20;
        },
    );

    registry.register_crafting("blast-mixer", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_items = true;
        craft.base.has_power = true;
        craft.base.size = 2;
        craft.base.env_enabled |= Env::SPACE;
        craft.output_item = item_amount(items, "blast-compound", 1);
        craft.ambient_sound = "loopMachineSpin".into();
        craft.ambient_sound_volume = 0.12;
        for (name, amount) in [("pyratite", 1), ("spore-pod", 1)] {
            if let Some(item) = item_amount(items, name, amount) {
                craft.consume_items.push(item);
            }
        }
        craft.consume_power = 0.40;
    });

    registry.register_crafting("melter", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_liquids = true;
        craft.base.has_power = true;
        craft.base.health = 200;
        craft.output_liquid = liquid_amount(liquids, "slag", 12.0 / 60.0);
        craft.craft_time = 10.0;
        craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile, DrawDefault)".into();
        if let Some(scrap) = item_amount(items, "scrap", 1) {
            craft.consume_items.push(scrap);
        }
        craft.consume_power = 1.0;
    });

    registry.register_crafting("separator", CraftingBlockKind::Separator, |craft| {
        craft.base.has_power = true;
        craft.base.size = 2;
        craft.craft_time = 35.0;
        for (name, amount) in [("copper", 5), ("lead", 3), ("graphite", 2), ("titanium", 2)] {
            if let Some(item) = item_amount(items, name, amount) {
                craft.results.push(item);
            }
        }
        if let Some(slag) = liquid_amount(liquids, "slag", 4.0 / 60.0) {
            craft.consume_liquids.push(slag);
        }
        craft.consume_power = 1.1;
        craft.drawer =
            "DrawMulti(DrawRegion(-bottom), DrawLiquidTile, DrawRegion(-spinner), DrawDefault)"
                .into();
    });

    registry.register_crafting("disassembler", CraftingBlockKind::Separator, |craft| {
        craft.base.has_power = true;
        craft.base.item_capacity = 20;
        craft.base.size = 3;
        craft.craft_time = 15.0;
        for (name, amount) in [
            ("sand", 2),
            ("graphite", 1),
            ("titanium", 1),
            ("thorium", 1),
        ] {
            if let Some(item) = item_amount(items, name, amount) {
                craft.results.push(item);
            }
        }
        if let Some(scrap) = item_amount(items, "scrap", 1) {
            craft.consume_items.push(scrap);
        }
        if let Some(slag) = liquid_amount(liquids, "slag", 0.12) {
            craft.consume_liquids.push(slag);
        }
        craft.consume_power = 4.0;
        craft.drawer =
            "DrawMulti(DrawRegion(-bottom), DrawLiquidTile, DrawRegion(-spinner), DrawDefault)"
                .into();
    });

    registry.register_crafting("spore-press", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_liquids = true;
        craft.base.has_power = true;
        craft.base.health = 320;
        craft.base.liquid_capacity = 60.0;
        craft.base.size = 2;
        craft.craft_effect = "none".into();
        craft.output_liquid = liquid_amount(liquids, "oil", 18.0 / 60.0);
        craft.craft_time = 20.0;
        craft.drawer = "DrawMulti(DrawPistons, DrawLiquidRegion)".into();
        if let Some(spore_pod) = item_amount(items, "spore-pod", 1) {
            craft.consume_items.push(spore_pod);
        }
        craft.consume_power = 0.7;
    });

    registry.register_crafting("pulverizer", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_items = true;
        craft.base.has_power = true;
        craft.craft_effect = "pulverize".into();
        craft.update_effect = "pulverizeSmall".into();
        craft.output_item = item_amount(items, "sand", 1);
        craft.craft_time = 40.0;
        craft.drawer = "DrawMulti(DrawDefault, DrawRegion(-rotator), DrawRegion(-top))".into();
        craft.ambient_sound = "loopGrind".into();
        craft.ambient_sound_volume = 0.025;
        if let Some(scrap) = item_amount(items, "scrap", 1) {
            craft.consume_items.push(scrap);
        }
        craft.consume_power = 0.50;
    });

    registry.register_crafting(
        "coal-centrifuge",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_items = true;
            craft.base.has_liquids = true;
            craft.base.has_power = true;
            craft.base.size = 2;
            craft.rotate_draw = false;
            craft.craft_effect = "coalSmeltsmoke".into();
            craft.output_item = item_amount(items, "coal", 1);
            craft.craft_time = 30.0;
            if let Some(oil) = liquid_amount(liquids, "oil", 0.1) {
                craft.consume_liquids.push(oil);
            }
            craft.consume_power = 0.7;
        },
    );

    registry.register_crafting(
        "silicon-arc-furnace",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_power = true;
            craft.base.has_liquids = false;
            craft.base.item_capacity = 30;
            craft.base.size = 3;
            craft.base.env_enabled |= Env::SPACE | Env::UNDERWATER;
            craft.base.env_disabled = Env::NONE;
            craft.fog_radius = 3.0;
            craft.craft_effect = "none".into();
            craft.output_item = item_amount(items, "silicon", 4);
            craft.craft_time = 50.0;
            craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawArcSmelt, DrawDefault)".into();
            craft.ambient_sound = "loopSmelter".into();
            craft.ambient_sound_volume = 0.12;
            for (name, amount) in [("beryllium", 150), ("graphite", 50)] {
                if let Some(item) = item_amount(items, name, amount) {
                    craft.research_cost.push(item);
                }
            }
            for (name, amount) in [("graphite", 1), ("sand", 4)] {
                if let Some(item) = item_amount(items, name, amount) {
                    craft.consume_items.push(item);
                }
            }
            craft.consume_power = 5.0;
        },
    );

    registry.register_crafting(
        "slag-centrifuge",
        CraftingBlockKind::GenericCrafter,
        |craft| {
            craft.base.has_liquids = true;
            craft.base.has_power = true;
            craft.base.liquid_capacity = 80.0;
            craft.base.size = 3;
            craft.base.build_visibility =
                crate::mindustry::world::meta::BuildVisibility::DebugOnly;
            craft.output_liquid = liquid_amount(liquids, "gallium", 1.0 / 60.0);
            craft.outputs_liquid = true;
            craft.craft_time = 120.0;
            craft.drawer =
                "DrawMulti(DrawRegion(-bottom), DrawLiquidRegion(slag), DrawGlowRegion*, DrawDefault)"
                    .into();
            if let Some(sand) = item_amount(items, "sand", 1) {
                craft.consume_items.push(sand);
            }
            if let Some(slag) = liquid_amount(liquids, "slag", 40.0 / 60.0) {
                craft.consume_liquids.push(slag);
            }
            craft.consume_power = 2.0 / 60.0;
        },
    );

    registry.register_crafting("electrolyzer", CraftingBlockKind::GenericCrafter, |craft| {
        craft.base.has_liquids = true;
        craft.base.has_power = true;
        craft.base.group = crate::mindustry::world::meta::BlockGroup::Liquids;
        craft.base.item_capacity = 0;
        craft.base.liquid_capacity = 50.0;
        craft.base.size = 3;
        craft.craft_time = 10.0;
        craft.rotate = true;
        craft.invert_flip = true;
        craft.region_rotated1 = 3;
        craft.research_cost_multiplier = 1.2;
        craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(water), DrawBubbles, DrawRegion, DrawLiquidOutputs, DrawGlowRegion)".into();
        craft.ambient_sound = "loopElectricHum".into();
        craft.ambient_sound_volume = 0.08;
        push_liquid_amount(&mut craft.consume_liquids, liquids, "water", 10.0 / 60.0);
        push_liquid_amount(&mut craft.output_liquids, liquids, "ozone", 4.0 / 60.0);
        push_liquid_amount(&mut craft.output_liquids, liquids, "hydrogen", 6.0 / 60.0);
        craft.liquid_output_directions = vec![1, 3];
        craft.consume_power = 1.0;
    });

    registry.register_crafting(
        "atmospheric-concentrator",
        CraftingBlockKind::HeatCrafter,
        |craft| {
            craft.base.has_liquids = true;
            craft.base.item_capacity = 0;
            craft.base.liquid_capacity = 60.0;
            craft.base.size = 3;
            craft.research_cost_multiplier = 1.1;
            craft.heat_requirement = 24.0;
            craft.max_efficiency = 1.0;
            craft.output_liquid = liquid_amount(liquids, "nitrogen", 16.0 / 60.0);
            craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(nitrogen), DrawDefault, DrawHeatInput, DrawParticles)".into();
            craft.ambient_sound = "loopExtract".into();
            craft.ambient_sound_volume = 0.06;
            for (name, amount) in [("silicon", 2000), ("oxide", 900), ("beryllium", 2400)] {
                push_item_amount(&mut craft.research_cost, items, name, amount);
            }
            craft.consume_power = 2.0;
        },
    );

    registry.register_crafting("vent-condenser", CraftingBlockKind::AttributeCrafter, |craft| {
        set_requirements(
            &mut craft.requirements,
            items,
            &[("graphite", 20), ("beryllium", 60)],
        );
        craft.attribute = "steam".into();
        craft.base.group = BlockGroup::Liquids;
        craft.min_efficiency = 9.0 - 0.0001;
        craft.base_efficiency = 0.0;
        craft.display_efficiency = false;
        craft.craft_effect = "turbinegenerate".into();
        craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawBlurSpin(-rotator), DrawRegion(-mid), DrawLiquidTile(water), DrawDefault)".into();
        craft.craft_time = 120.0;
        craft.base.size = 3;
        craft.ambient_sound = "loopHum".into();
        craft.ambient_sound_volume = 0.06;
        craft.base.has_liquids = true;
        craft.boost_scale = 1.0 / 9.0;
        craft.base.item_capacity = 0;
        craft.output_liquid = liquid_amount(liquids, "water", 30.0 / 60.0);
        craft.outputs_liquid = true;
        craft.consume_power = 0.5;
        craft.base.has_power = true;
        craft.base.liquid_capacity = 60.0;
    });

    registry.register_crafting(
        "oxidation-chamber",
        CraftingBlockKind::HeatProducer,
        |craft| {
            craft.base.liquid_capacity = 30.0;
            craft.base.size = 3;
            craft.research_cost_multiplier = 1.1;
            craft.rotate_draw = false;
            craft.region_rotated1 = 2;
            craft.craft_time = 120.0;
            craft.heat_output = 5.0;
            craft.output_item = item_amount(items, "oxide", 1);
            craft.drawer =
                "DrawMulti(DrawRegion(-bottom), DrawLiquidRegion, DrawDefault, DrawHeatOutput)"
                    .into();
            craft.ambient_sound = "loopExtract".into();
            craft.ambient_sound_volume = 0.08;
            push_liquid_amount(&mut craft.consume_liquids, liquids, "ozone", 2.0 / 60.0);
            push_item_amount(&mut craft.consume_items, items, "beryllium", 1);
            craft.consume_power = 0.5;
        },
    );

    registry.register_crafting(
        "electric-heater",
        CraftingBlockKind::HeatProducer,
        |craft| {
            craft.base.item_capacity = 0;
            craft.base.size = 2;
            craft.research_cost_multiplier = 4.0;
            craft.rotate_draw = false;
            craft.region_rotated1 = 1;
            craft.heat_output = 3.0;
            craft.drawer = "DrawMulti(DrawDefault, DrawHeatOutput)".into();
            craft.ambient_sound = "loopHum".into();
            craft.consume_power = 100.0 / 60.0;
        },
    );

    registry.register_crafting("slag-heater", CraftingBlockKind::HeatProducer, |craft| {
        craft.base.item_capacity = 0;
        craft.base.liquid_capacity = 120.0;
        craft.base.size = 3;
        craft.research_cost_multiplier = 4.0;
        craft.rotate_draw = false;
        craft.region_rotated1 = 1;
        craft.heat_output = 8.0;
        craft.drawer =
            "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(slag), DrawDefault, DrawHeatOutput)"
                .into();
        craft.ambient_sound = "loopHum".into();
        push_liquid_amount(&mut craft.consume_liquids, liquids, "slag", 40.0 / 60.0);
        for (name, amount) in [("tungsten", 1200), ("oxide", 900), ("beryllium", 2400)] {
            push_item_amount(&mut craft.research_cost, items, name, amount);
        }
    });

    registry.register_crafting("phase-heater", CraftingBlockKind::HeatProducer, |craft| {
        craft.base.size = 2;
        craft.heat_output = 15.0;
        craft.craft_time = 480.0;
        craft.drawer = "DrawMulti(DrawDefault, DrawHeatOutput)".into();
        craft.ambient_sound = "loopHum".into();
        push_item_amount(&mut craft.consume_items, items, "phase-fabric", 1);
    });

    registry.register_crafting(
        "heat-redirector",
        CraftingBlockKind::HeatConductor,
        |craft| {
            craft.base.group = crate::mindustry::world::meta::BlockGroup::Heat;
            craft.base.size = 3;
            craft.research_cost_multiplier = 10.0;
            craft.region_rotated1 = 1;
            craft.drawer = "DrawMulti(DrawDefault, DrawHeatOutput, DrawHeatInput(-heat))".into();
        },
    );

    registry.register_crafting(
        "small-heat-redirector",
        CraftingBlockKind::HeatConductor,
        |craft| {
            craft.base.group = crate::mindustry::world::meta::BlockGroup::Heat;
            craft.base.size = 2;
            craft.research_cost_multiplier = 2.0;
            craft.region_rotated1 = 1;
            craft.drawer = "DrawMulti(DrawDefault, DrawHeatOutput, DrawHeatInput(-heat))".into();
        },
    );

    registry.register_crafting("heat-router", CraftingBlockKind::HeatConductor, |craft| {
        craft.base.group = crate::mindustry::world::meta::BlockGroup::Heat;
        craft.base.size = 3;
        craft.research_cost_multiplier = 10.0;
        craft.region_rotated1 = 1;
        craft.split_heat = true;
        craft.drawer =
            "DrawMulti(DrawDefault, DrawHeatOutput(-1), DrawHeatOutput, DrawHeatOutput(1), DrawHeatInput(-heat))"
                .into();
    });

    registry.register_crafting(
        "slag-incinerator",
        CraftingBlockKind::ItemIncinerator,
        |craft| {
            craft.base.size = 1;
            push_liquid_amount(&mut craft.consume_liquids, liquids, "slag", 0.0);
        },
    );

    registry.register_crafting(
        "carbide-crucible",
        CraftingBlockKind::HeatCrafter,
        |craft| {
            craft.base.has_items = true;
            craft.base.has_power = true;
            craft.base.item_capacity = 20;
            craft.base.size = 3;
            craft.heat_requirement = 40.0;
            craft.max_efficiency = 1.0;
            craft.craft_effect = "none".into();
            craft.output_item = item_amount(items, "carbide", 1);
            craft.craft_time = 60.0 * 2.25 / 4.0;
            craft.drawer =
                "DrawMulti(DrawRegion(-bottom), DrawCrucibleFlame, DrawDefault, DrawHeatInput)"
                    .into();
            craft.ambient_sound = "loopSmelter".into();
            craft.ambient_sound_volume = 0.09;
            for (name, amount) in [("tungsten", 2), ("graphite", 3)] {
                push_item_amount(&mut craft.consume_items, items, name, amount);
            }
            craft.consume_power = 2.0;
        },
    );

    registry.register_crafting("surge-crucible", CraftingBlockKind::HeatCrafter, |craft| {
        craft.base.item_capacity = 20;
        craft.base.liquid_capacity = 400.0;
        craft.base.size = 3;
        craft.heat_requirement = 40.0;
        craft.max_efficiency = 1.0;
        craft.craft_time = 45.0;
        craft.output_item = item_amount(items, "surge-alloy", 1);
        craft.craft_effect = "RadialEffect(surgeCruciSmoke,4,90,5)".into();
        craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawCircles, DrawLiquidRegion(slag), DrawDefault, DrawHeatInput, DrawHeatRegion, DrawHeatRegion(-vents))".into();
        craft.ambient_sound = "loopSmelter".into();
        craft.ambient_sound_volume = 0.9;
        push_item_amount(&mut craft.consume_items, items, "silicon", 3);
        push_liquid_amount(&mut craft.consume_liquids, liquids, "slag", 160.0 / 60.0);
        craft.consume_power = 1.5;
    });

    registry.register_crafting(
        "cyanogen-synthesizer",
        CraftingBlockKind::HeatCrafter,
        |craft| {
            craft.base.liquid_capacity = 80.0;
            craft.base.size = 3;
            craft.heat_requirement = 20.0;
            craft.max_efficiency = 1.0;
            craft.output_liquid = liquid_amount(liquids, "cyanogen", 12.0 / 60.0);
            craft.craft_time = 20.0;
            craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(cyanogen), DrawParticles, DrawDefault, DrawHeatInput, DrawHeatRegion(-heat-top))".into();
            craft.ambient_sound = "loopExtract".into();
            craft.ambient_sound_volume = 0.08;
            push_liquid_amount(&mut craft.consume_liquids, liquids, "arkycite", 160.0 / 60.0);
            push_item_amount(&mut craft.consume_items, items, "graphite", 1);
            craft.consume_power = 2.0;
        },
    );

    registry.register_crafting(
        "phase-synthesizer",
        CraftingBlockKind::HeatCrafter,
        |craft| {
            craft.base.item_capacity = 40;
            craft.base.liquid_capacity = 40.0;
            craft.base.size = 3;
            craft.heat_requirement = 32.0;
            craft.max_efficiency = 1.0;
            craft.output_item = item_amount(items, "phase-fabric", 1);
            craft.craft_time = 30.0;
            craft.drawer = "DrawMulti(DrawRegion(-bottom), DrawSpikes, DrawMultiWeave, DrawDefault, DrawHeatInput, DrawHeatRegion(-vents))".into();
            craft.ambient_sound = "loopTech".into();
            craft.ambient_sound_volume = 0.04;
            for (name, amount) in [("thorium", 2), ("sand", 6)] {
                push_item_amount(&mut craft.consume_items, items, name, amount);
            }
            push_liquid_amount(&mut craft.consume_liquids, liquids, "ozone", 8.0 / 60.0);
            craft.consume_power = 8.0;
        },
    );

    registry.register_crafting("heat-reactor", CraftingBlockKind::HeatProducer, |craft| {
        craft.base.build_visibility = crate::mindustry::world::meta::BuildVisibility::DebugOnly;
        craft.base.item_capacity = 20;
        craft.base.size = 3;
        craft.craft_effect = "RadialEffect(heatReactorSmoke,4,90,7)".into();
        craft.output_item = item_amount(items, "fissile-matter", 1);
        craft.craft_time = 600.0;
        push_item_amount(&mut craft.consume_items, items, "thorium", 3);
        push_liquid_amount(&mut craft.consume_liquids, liquids, "nitrogen", 1.0 / 60.0);
    });

    registry.register_crafting("heat-source", CraftingBlockKind::HeatProducer, |craft| {
        craft.base.build_visibility = crate::mindustry::world::meta::BuildVisibility::SandboxOnly;
        craft.base.item_capacity = 0;
        craft.base.size = 1;
        craft.rotate_draw = false;
        craft.region_rotated1 = 1;
        craft.heat_output = 1000.0;
        craft.warmup_rate = 1000.0;
        craft.always_unlocked = true;
        craft.all_database_tabs = true;
        craft.ambient_sound = "none".into();
        craft.drawer = "DrawMulti(DrawDefault, DrawHeatOutput)".into();
    });
}

fn find_item<'a>(items: &'a [Item], name: &str) -> Option<&'a Item> {
    items
        .iter()
        .find(|item| item.base.mappable.name.as_str() == name)
}

fn liquid_id(liquids: &[Liquid], name: &str) -> Option<ContentId> {
    liquids
        .iter()
        .find(|liquid| liquid.base.mappable.name.as_str() == name)
        .map(|liquid| liquid.base.mappable.base.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::content::{items, liquids};

    fn load_test_registry() -> (Vec<Item>, Vec<Liquid>, BlockRegistry) {
        let all_items = items::load();
        let all_liquids = liquids::load();
        let registry = load(&all_items, &all_liquids);
        (all_items, all_liquids, registry)
    }

    #[test]
    fn minimal_registry_keeps_air_first_and_names_indexed() {
        let (_, _, registry) = load_test_registry();
        assert_eq!(registry.id_by_name("air"), Some(0));
        assert_eq!(registry.get(0).unwrap().base().name, "air");
        assert!(registry.id_by_name("stone").is_some());
        assert!(registry.id_by_name("stone-wall").is_some());
    }

    #[test]
    fn floor_links_match_name_lookup_and_manual_overrides() {
        let (_, _, registry) = load_test_registry();
        let stone_wall = registry.id_by_name("stone-wall").unwrap();
        let spore_wall = registry.id_by_name("spore-wall").unwrap();
        let ice_wall = registry.id_by_name("ice-wall").unwrap();
        let dune_wall = registry.id_by_name("dune-wall").unwrap();
        let dirt_wall = registry.id_by_name("dirt-wall").unwrap();
        let snow_wall = registry.id_by_name("snow-wall").unwrap();
        let shale_wall = registry.id_by_name("shale-wall").unwrap();
        let salt_wall = registry.id_by_name("salt-wall").unwrap();

        assert_eq!(
            registry.get_floor_by_name("stone").unwrap().wall,
            stone_wall
        );
        assert_eq!(
            registry.get_floor_by_name("spore-moss").unwrap().wall,
            spore_wall
        );
        assert_eq!(
            registry.get_floor_by_name("ice-snow").unwrap().wall,
            ice_wall
        );
        assert_eq!(registry.get_floor_by_name("dirt").unwrap().wall, dirt_wall);
        assert_eq!(registry.get_floor_by_name("snow").unwrap().wall, snow_wall);
        assert_eq!(
            registry.get_floor_by_name("shale").unwrap().wall,
            shale_wall
        );
        assert_eq!(registry.get_floor_by_name("salt").unwrap().wall, salt_wall);
        assert_eq!(
            registry.get_floor_by_name("basalt").unwrap().wall,
            dune_wall
        );
        assert_eq!(
            registry
                .get_floor_by_name("darksand-tainted-water")
                .unwrap()
                .wall,
            dune_wall
        );
    }

    #[test]
    fn metal_and_dark_panel_floors_link_to_dark_metal_wall() {
        let (_, _, registry) = load_test_registry();
        let dark_metal = registry.id_by_name("dark-metal").unwrap();
        assert!(matches!(
            registry.get(dark_metal).unwrap(),
            BlockDef::StaticWall(_)
        ));

        for (floor_name, variants) in [
            ("metal-floor", 0),
            ("metal-floor-damaged", 3),
            ("metal-floor-2", 0),
            ("metal-floor-3", 0),
            ("metal-floor-4", 0),
            ("metal-floor-5", 0),
            ("dark-panel-1", 0),
            ("dark-panel-2", 0),
            ("dark-panel-3", 0),
            ("dark-panel-4", 0),
            ("dark-panel-5", 0),
            ("dark-panel-6", 0),
        ] {
            let floor = registry.get_floor_by_name(floor_name).unwrap();
            assert_eq!(floor.wall, dark_metal, "{floor_name} wall link");
            assert_eq!(floor.base.variants, variants, "{floor_name} variants");
        }
    }

    #[test]
    fn metal_tiles_and_walls_keep_autotile_light_fields() {
        let (_, _, registry) = load_test_registry();

        for floor_name in [
            "metal-tiles-1",
            "metal-tiles-2",
            "metal-tiles-3",
            "metal-tiles-4",
            "metal-tiles-5",
            "metal-tiles-6",
            "metal-tiles-7",
            "metal-tiles-8",
            "metal-tiles-9",
            "metal-tiles-10",
            "metal-tiles-11",
            "metal-tiles-12",
            "metal-tiles-13",
        ] {
            let floor = registry.get_floor_by_name(floor_name).unwrap();
            assert!(floor.autotile, "{floor_name} autotile");
            assert!(!floor.draw_edge_out, "{floor_name} draw_edge_out");
            assert!(!floor.draw_edge_in, "{floor_name} draw_edge_in");
        }

        let tiles6 = registry.get_floor_by_name("metal-tiles-6").unwrap();
        assert!(tiles6.base.emit_light);
        assert_eq!(tiles6.base.light_radius, 30.0);
        assert_eq!(tiles6.base.light_color_rgba, 0xff00004d);

        let tiles12 = registry.get_floor_by_name("metal-tiles-12").unwrap();
        assert_eq!(tiles12.autotile_variants, 4);
        assert!(tiles12.base.emit_light);
        assert_eq!(tiles12.base.light_radius, 30.0);
        assert_eq!(tiles12.base.light_color_rgba, 0xff00004d);

        assert_eq!(
            registry
                .get_floor_by_name("metal-tiles-7")
                .unwrap()
                .autotile_mid_variants,
            9
        );
        assert_eq!(
            registry
                .get_floor_by_name("metal-tiles-8")
                .unwrap()
                .autotile_mid_variants,
            2
        );
        assert_eq!(
            registry
                .get_floor_by_name("metal-tiles-11")
                .unwrap()
                .autotile_variants,
            3
        );
        assert_eq!(
            registry
                .get_floor_by_name("metal-tiles-13")
                .unwrap()
                .autotile_mid_variants,
            6
        );

        for wall_name in ["metal-wall-1", "metal-wall-2", "metal-wall-3"] {
            let wall = registry.get_by_name(wall_name).unwrap();
            let BlockDef::StaticWall(wall) = wall else {
                panic!("{wall_name} should be a static wall");
            };
            assert!(wall.autotile, "{wall_name} autotile");
        }
        let BlockDef::StaticWall(metal_wall2) = registry.get_by_name("metal-wall-2").unwrap()
        else {
            panic!("metal-wall-2 should be a static wall");
        };
        assert_eq!(metal_wall2.autotile_mid_variants, 2);
    }

    #[test]
    fn extended_environment_floors_and_walls_keep_upstream_subset() {
        let (_, all_liquids, registry) = load_test_registry();
        let arkycite = liquid_id(&all_liquids, "arkycite").unwrap();
        let basalt = registry.id_by_name("basalt").unwrap();
        let rhyolite = registry.id_by_name("rhyolite").unwrap();
        let ferric_stone = registry.id_by_name("ferric-stone").unwrap();

        let hotrock = registry.get_floor_by_name("hotrock").unwrap();
        assert_eq!(hotrock.blend_group, basalt);
        assert!(hotrock.base.emit_light);
        assert_eq!(hotrock.base.light_radius, 30.0);

        let magmarock = registry.get_floor_by_name("magmarock").unwrap();
        assert_eq!(magmarock.blend_group, basalt);
        assert!(magmarock.base.emit_light);
        assert_eq!(magmarock.base.light_radius, 50.0);

        assert_eq!(
            registry
                .get_floor_by_name("rhyolite-crater")
                .unwrap()
                .blend_group,
            rhyolite
        );
        assert_eq!(
            registry
                .get_floor_by_name("rough-rhyolite")
                .unwrap()
                .base
                .variants,
            3
        );
        assert_eq!(
            registry
                .get_floor_by_name("carbon-stone")
                .unwrap()
                .base
                .variants,
            4
        );
        assert_eq!(
            registry
                .get_floor_by_name("ferric-craters")
                .unwrap()
                .blend_group,
            ferric_stone
        );
        assert_eq!(
            registry
                .get_floor_by_name("beryllic-stone")
                .unwrap()
                .base
                .variants,
            4
        );
        assert_eq!(
            registry
                .get_floor_by_name("crystalline-stone")
                .unwrap()
                .base
                .variants,
            5
        );
        assert_eq!(
            registry
                .get_floor_by_name("red-stone")
                .unwrap()
                .base
                .variants,
            4
        );
        let core_zone = registry.get_floor_by_name("core-zone").unwrap();
        assert_eq!(core_zone.base.variants, 0);
        assert!(core_zone.allow_core_placement);

        let arkycite_floor = registry.get_floor_by_name("arkycite-floor").unwrap();
        assert_eq!(arkycite_floor.speed_multiplier, 0.3);
        assert_eq!(arkycite_floor.base.variants, 0);
        assert_eq!(arkycite_floor.liquid_drop, Some(arkycite));
        assert!(arkycite_floor.is_liquid);
        assert_eq!(arkycite_floor.drown_time, 200.0);
        assert_eq!(arkycite_floor.base.cache_layer, CacheLayer::Arkycite);
        assert_eq!(arkycite_floor.base.albedo, 0.9);
        assert!(arkycite_floor.base.obstructs_light);

        let sand_wall = registry.id_by_name("sand-wall").unwrap();
        assert_eq!(
            registry.get_floor_by_name("sand-floor").unwrap().wall,
            sand_wall
        );
        assert_eq!(
            registry.get_floor_by_name("deep-water").unwrap().wall,
            sand_wall
        );
        assert_eq!(
            registry.get_floor_by_name("shallow-water").unwrap().wall,
            sand_wall
        );

        for (floor_name, wall_name) in [
            ("dacite", "dacite-wall"),
            ("regolith", "regolith-wall"),
            ("yellow-stone", "yellow-stone-wall"),
            ("molten-slag", "yellow-stone-wall"),
            ("yellow-stone-plates", "yellow-stone-wall"),
            ("rhyolite", "rhyolite-wall"),
            ("rhyolite-crater", "rhyolite-wall"),
            ("rough-rhyolite", "rhyolite-wall"),
            ("carbon-stone", "carbon-wall"),
            ("ferric-stone", "ferric-stone-wall"),
            ("beryllic-stone", "beryllic-stone-wall"),
            ("arkycite-floor", "arkyic-wall"),
            ("arkyic-stone", "arkyic-wall"),
            ("crystal-floor", "crystalline-stone-wall"),
            ("crystalline-stone", "crystalline-stone-wall"),
            ("red-ice", "red-ice-wall"),
            ("red-stone", "red-stone-wall"),
            ("dense-red-stone", "red-stone-wall"),
        ] {
            assert_eq!(
                registry.get_floor_by_name(floor_name).unwrap().wall,
                registry.id_by_name(wall_name).unwrap(),
                "{floor_name} wall"
            );
        }

        let BlockDef::StaticWall(arkyic_wall) = registry.get_by_name("arkyic-wall").unwrap() else {
            panic!("arkyic-wall should be a static wall");
        };
        assert_eq!(arkyic_wall.base.variants, 3);
        let BlockDef::StaticWall(crystalline_wall) =
            registry.get_by_name("crystalline-stone-wall").unwrap()
        else {
            panic!("crystalline-stone-wall should be a static wall");
        };
        assert_eq!(crystalline_wall.base.variants, 4);
    }

    #[test]
    fn static_trees_tree_blocks_and_tall_blocks_keep_upstream_subset() {
        let (_, _, registry) = load_test_registry();
        let spore_pine = registry.id_by_name("spore-pine").unwrap();
        assert_eq!(registry.get_floor_by_name("moss").unwrap().wall, spore_pine);

        let BlockDef::StaticTree(red_diamond) = registry.get_by_name("red-diamond-wall").unwrap()
        else {
            panic!("red-diamond-wall should be a static tree");
        };
        assert_eq!(red_diamond.wall.base.variants, 3);
        assert_eq!(red_diamond.wall.base.cache_layer, CacheLayer::Walls);

        for name in ["spore-pine", "snow-pine", "pine"] {
            let BlockDef::StaticTree(tree) = registry.get_by_name(name).unwrap() else {
                panic!("{name} should be a static tree");
            };
            assert_eq!(tree.wall.base.variants, 0, "{name} variants");
            assert_eq!(tree.wall.base.cache_layer, CacheLayer::Walls);
        }

        for name in ["white-tree-dead", "white-tree"] {
            let BlockDef::TreeBlock(tree) = registry.get_by_name(name).unwrap() else {
                panic!("{name} should be a tree block");
            };
            assert!(tree.base.solid);
            assert_eq!(tree.base.clip_size, 90.0);
            assert!(tree.base.custom_shadow);
            assert_eq!(tree.shadow_offset, -4.0);
        }

        for name in ["crystal-cluster", "vibrant-crystal-cluster"] {
            let BlockDef::TallBlock(tall) = registry.get_by_name(name).unwrap() else {
                panic!("{name} should be a tall block");
            };
            assert_eq!(tall.base.variants, 3);
            assert_eq!(tall.base.clip_size, 128.0);
            assert!(tall.base.solid);
            assert!(tall.base.custom_shadow);
            assert_eq!(tall.shadow_alpha, 0.6);
            assert_eq!(tall.shadow_offset, -3.0);
        }

        for name in ["crystal-blocks", "crystal-orbs"] {
            let BlockDef::TallBlock(tall) = registry.get_by_name(name).unwrap() else {
                panic!("{name} should be a tall block");
            };
            assert_eq!(tall.base.variants, 3);
            assert_eq!(tall.base.clip_size, 128.0);
            assert_eq!(tall.shadow_alpha, 0.5);
            assert_eq!(tall.shadow_offset, -2.5);
        }
    }

    #[test]
    fn overlay_floors_and_graphitic_wall_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        for name in ["pebbles", "tendrils"] {
            let floor = registry.get_floor_by_name(name).unwrap();
            assert!(floor.overlay_floor, "{name} overlay marker");
            assert!(!floor.base.use_color, "{name} use_color");
        }

        let graphite = find_item(&all_items, "graphite").unwrap();
        let BlockDef::StaticWall(graphitic) = registry.get_by_name("graphitic-wall").unwrap()
        else {
            panic!("graphitic-wall should be a static wall");
        };
        assert_eq!(
            graphitic.base.item_drop,
            Some(graphite.base.mappable.base.id)
        );
        assert_eq!(graphitic.base.variants, 3);
    }

    #[test]
    fn prop_and_boulder_blocks_keep_decoration_links() {
        let (_, _, registry) = load_test_registry();
        let boulder = registry.id_by_name("boulder").unwrap();
        let snow_boulder = registry.id_by_name("snow-boulder").unwrap();
        let shale_boulder = registry.id_by_name("shale-boulder").unwrap();
        let sand_boulder = registry.id_by_name("sand-boulder").unwrap();
        let basalt_boulder = registry.id_by_name("basalt-boulder").unwrap();
        let dacite_boulder = registry.id_by_name("dacite-boulder").unwrap();
        let carbon_boulder = registry.id_by_name("carbon-boulder").unwrap();
        let ferric_boulder = registry.id_by_name("ferric-boulder").unwrap();
        let beryllic_boulder = registry.id_by_name("beryllic-boulder").unwrap();
        let yellow_stone_boulder = registry.id_by_name("yellow-stone-boulder").unwrap();
        let arkyic_boulder = registry.id_by_name("arkyic-boulder").unwrap();
        let crystalline_boulder = registry.id_by_name("crystalline-boulder").unwrap();
        let red_ice_boulder = registry.id_by_name("red-ice-boulder").unwrap();
        let rhyolite_boulder = registry.id_by_name("rhyolite-boulder").unwrap();
        let red_stone_boulder = registry.id_by_name("red-stone-boulder").unwrap();

        let BlockDef::Prop(spore_cluster) = registry.get_by_name("spore-cluster").unwrap() else {
            panic!("spore-cluster should be a prop");
        };
        assert_eq!(spore_cluster.base.variants, 3);
        assert_eq!(spore_cluster.base.break_sound, "plantBreak");
        assert!(!spore_cluster.base.obstructs_light);

        let BlockDef::Prop(redweed) = registry.get_by_name("redweed").unwrap() else {
            panic!("redweed should be a prop");
        };
        assert_eq!(redweed.kind, PropKind::Seaweed);
        assert_eq!(redweed.base.variants, 3);
        assert!(!redweed.base.obstructs_light);

        let BlockDef::Prop(yellowcoral) = registry.get_by_name("yellowcoral").unwrap() else {
            panic!("yellowcoral should be a prop");
        };
        let sea_bush = yellowcoral.sea_bush.as_ref().unwrap();
        assert_eq!(yellowcoral.kind, PropKind::SeaBush);
        assert_eq!(sea_bush.lobes_min, 2);
        assert_eq!(sea_bush.lobes_max, 3);
        assert_eq!(sea_bush.mag_min, 2.0);
        assert_eq!(sea_bush.mag_max, 8.0);
        assert_eq!(sea_bush.origin, 0.3);
        assert_eq!(sea_bush.spread, 40.0);
        assert_eq!(sea_bush.scl_min, 60.0);
        assert_eq!(sea_bush.scl_max, 100.0);

        assert_eq!(
            registry.get_floor_by_name("stone").unwrap().decoration,
            boulder
        );
        assert_eq!(
            registry
                .get_floor_by_name("crater-stone")
                .unwrap()
                .decoration,
            boulder
        );
        for floor_name in ["snow", "ice", "ice-snow", "salt"] {
            assert_eq!(
                registry.get_floor_by_name(floor_name).unwrap().decoration,
                snow_boulder,
                "{floor_name} decoration"
            );
        }
        assert_eq!(
            registry.get_floor_by_name("shale").unwrap().decoration,
            shale_boulder
        );
        assert_eq!(
            registry.get_floor_by_name("sand-floor").unwrap().decoration,
            sand_boulder
        );
        for floor_name in ["basalt", "hotrock", "darksand", "magmarock"] {
            assert_eq!(
                registry.get_floor_by_name(floor_name).unwrap().decoration,
                basalt_boulder,
                "{floor_name} decoration"
            );
        }
        assert_eq!(
            registry.get_floor_by_name("dacite").unwrap().decoration,
            dacite_boulder
        );
        assert_eq!(
            registry
                .get_floor_by_name("carbon-stone")
                .unwrap()
                .decoration,
            carbon_boulder
        );
        for floor_name in ["ferric-stone", "ferric-craters"] {
            assert_eq!(
                registry.get_floor_by_name(floor_name).unwrap().decoration,
                ferric_boulder,
                "{floor_name} decoration"
            );
        }
        assert_eq!(
            registry
                .get_floor_by_name("beryllic-stone")
                .unwrap()
                .decoration,
            beryllic_boulder
        );
        for floor_name in ["yellow-stone", "regolith", "yellow-stone-plates"] {
            assert_eq!(
                registry.get_floor_by_name(floor_name).unwrap().decoration,
                yellow_stone_boulder,
                "{floor_name} decoration"
            );
        }
        assert_eq!(
            registry
                .get_floor_by_name("arkyic-stone")
                .unwrap()
                .decoration,
            arkyic_boulder
        );
        let BlockDef::Prop(arkyic) = registry.get_by_name("arkyic-boulder").unwrap() else {
            panic!("arkyic-boulder should be a prop");
        };
        assert_eq!(arkyic.base.variants, 3);
        assert!(arkyic.base.custom_shadow);
        assert!(!arkyic.base.obstructs_light);

        assert_eq!(
            registry
                .get_floor_by_name("crystalline-stone")
                .unwrap()
                .decoration,
            crystalline_boulder
        );
        assert_eq!(
            registry.get_floor_by_name("red-ice").unwrap().decoration,
            red_ice_boulder
        );
        for floor_name in ["rhyolite", "rough-rhyolite"] {
            assert_eq!(
                registry.get_floor_by_name(floor_name).unwrap().decoration,
                rhyolite_boulder,
                "{floor_name} decoration"
            );
        }
        for floor_name in ["dense-red-stone", "red-stone"] {
            assert_eq!(
                registry.get_floor_by_name(floor_name).unwrap().decoration,
                red_stone_boulder,
                "{floor_name} decoration"
            );
        }
    }

    #[test]
    fn ore_blocks_keep_item_drop_color_and_generation_values() {
        let (all_items, _, registry) = load_test_registry();
        let copper = find_item(&all_items, "copper").unwrap();
        let ore = registry.get_floor_by_name("ore-copper").unwrap();
        assert_eq!(ore.base.item_drop, Some(copper.base.mappable.base.id));
        assert_eq!(ore.base.map_color_rgba, copper.color_rgba);
        assert!(ore.ore_default);
        assert_eq!(ore.ore_threshold, 0.81);
        assert_eq!(ore.ore_scale, 23.47619);

        let thorium = find_item(&all_items, "thorium").unwrap();
        let wall_ore = registry.get_floor_by_name("ore-wall-thorium").unwrap();
        assert_eq!(wall_ore.base.item_drop, Some(thorium.base.mappable.base.id));
        assert!(wall_ore.wall_ore);
        assert_eq!(
            wall_ore.base.localized_name.as_deref(),
            Some("thorium wall ore")
        );
    }

    #[test]
    fn serpulo_drills_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };
        let water_boost = |amount| LiquidConsume {
            liquid: liquid_id("water"),
            amount,
            booster: true,
        };

        for name in [
            "mechanical-drill",
            "pneumatic-drill",
            "laser-drill",
            "blast-drill",
        ] {
            let drill = registry.get_production_by_name(name).unwrap();
            assert_eq!(drill.kind, ProductionBlockKind::Drill, "{name} kind");
            assert_eq!(drill.base.group, BlockGroup::Drills, "{name} group");
            assert!(drill.base.update, "{name} update");
            assert!(drill.base.solid, "{name} solid");
            assert!(drill.base.has_liquids, "{name} liquids");
            assert!(drill.base.has_items, "{name} items");
            assert!(
                drill.base.flags.contains(&BlockFlag::Drill),
                "{name} drill flag"
            );
            assert_eq!(drill.hardness_drill_multiplier, 50.0, "{name} hardness");
            assert_eq!(drill.draw_mine_item, true, "{name} draw mine item");
            assert_eq!(
                drill.drill_effect_chance, 0.02,
                "{name} drill effect chance"
            );
            assert_eq!(drill.draw_spin_sprite, true, "{name} spin sprite");
            assert_eq!(drill.heat_color, "ff5512", "{name} heat color");
            assert_eq!(drill.ambient_sound, "loopDrill", "{name} ambient");
            assert_eq!(drill.ambient_sound_volume, 0.019, "{name} ambient volume");
            assert!(drill.consume_liquids.iter().all(|consume| consume.booster));
        }

        let mechanical = registry.get_production_by_name("mechanical-drill").unwrap();
        assert_eq!(mechanical.tier, 2);
        assert_eq!(mechanical.drill_time, 600.0);
        assert_eq!(mechanical.base.size, 2);
        assert_eq!(mechanical.drill_effect_rnd, 2.0);
        assert_eq!(mechanical.base.env_enabled & Env::SPACE, 0);
        assert_eq!(
            mechanical.requirements,
            vec![ItemAmount {
                item: item_id("copper"),
                amount: 12
            }]
        );
        assert_eq!(
            mechanical.research_cost,
            vec![ItemAmount {
                item: item_id("copper"),
                amount: 10
            }]
        );
        assert_eq!(mechanical.consume_liquids, vec![water_boost(0.05)]);

        let pneumatic = registry.get_production_by_name("pneumatic-drill").unwrap();
        assert_eq!(pneumatic.tier, 3);
        assert_eq!(pneumatic.drill_time, 400.0);
        assert_eq!(pneumatic.base.size, 2);
        assert_eq!(pneumatic.drill_effect_rnd, 2.0);
        assert_ne!(pneumatic.base.env_enabled & Env::SPACE, 0);
        assert_eq!(
            pneumatic.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 18
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 10
                }
            ]
        );
        assert_eq!(pneumatic.consume_liquids, vec![water_boost(3.5 / 60.0)]);

        let laser = registry.get_production_by_name("laser-drill").unwrap();
        assert_eq!(laser.tier, 4);
        assert_eq!(laser.drill_time, 280.0);
        assert_eq!(laser.base.size, 3);
        assert_eq!(laser.drill_effect_rnd, 3.0);
        assert!(laser.base.has_power);
        assert_eq!(laser.update_effect, "pulverizeMedium");
        assert_eq!(laser.drill_effect, "mineBig");
        assert_eq!(laser.consume_power, 1.10);
        assert_eq!(laser.consume_liquids, vec![water_boost(0.08)]);
        assert_eq!(
            laser.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 35
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 20
                }
            ]
        );

        let blast = registry.get_production_by_name("blast-drill").unwrap();
        assert_eq!(blast.tier, 5);
        assert_eq!(blast.drill_time, 280.0);
        assert_eq!(blast.base.size, 4);
        assert_eq!(blast.drill_effect_rnd, 4.0);
        assert!(blast.draw_rim);
        assert!(blast.base.has_power);
        assert_eq!(blast.update_effect, "pulverizeRed");
        assert_eq!(blast.update_effect_chance, 0.03);
        assert_eq!(blast.drill_effect, "mineHuge");
        assert_eq!(blast.rotate_speed, 6.0);
        assert_eq!(blast.warmup_speed, 0.01);
        assert_eq!(blast.base.item_capacity, 20);
        assert_eq!(blast.liquid_boost_intensity, 1.8);
        assert_eq!(blast.consume_power, 3.0);
        assert_eq!(blast.consume_liquids, vec![water_boost(0.1)]);
        assert_eq!(
            blast.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 65
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 75
                }
            ]
        );
    }

    #[test]
    fn early_crafting_blocks_keep_upstream_recipe_and_runtime_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let graphite_press = registry.get_crafting_by_name("graphite-press").unwrap();
        assert_eq!(graphite_press.kind, CraftingBlockKind::GenericCrafter);
        assert_eq!(graphite_press.base.size, 2);
        assert!(graphite_press.base.has_items);
        assert_eq!(graphite_press.craft_effect, "pulverizeMedium");
        assert_eq!(graphite_press.craft_time, 90.0);
        assert_eq!(
            graphite_press.output_item,
            Some(ItemAmount {
                item: item_id("graphite"),
                amount: 1
            })
        );
        assert_eq!(
            graphite_press.consume_items,
            vec![ItemAmount {
                item: item_id("coal"),
                amount: 2
            }]
        );

        let multi_press = registry.get_crafting_by_name("multi-press").unwrap();
        assert_eq!(multi_press.base.size, 3);
        assert_eq!(multi_press.base.item_capacity, 20);
        assert!(multi_press.base.has_items);
        assert!(multi_press.base.has_liquids);
        assert!(multi_press.base.has_power);
        assert_eq!(multi_press.craft_time, 30.0);
        assert_eq!(multi_press.consume_power, 1.8);
        assert_eq!(
            multi_press.output_item,
            Some(ItemAmount {
                item: item_id("graphite"),
                amount: 2
            })
        );
        assert_eq!(
            multi_press.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("water"),
                amount: 0.1
            }]
        );

        let silicon = registry.get_crafting_by_name("silicon-smelter").unwrap();
        assert_eq!(silicon.craft_effect, "smeltsmoke");
        assert_eq!(silicon.craft_time, 40.0);
        assert_eq!(silicon.consume_power, 0.50);
        assert_eq!(silicon.ambient_sound, "loopSmelter");
        assert_eq!(silicon.ambient_sound_volume, 0.07);
        assert_eq!(
            silicon.consume_items,
            vec![
                ItemAmount {
                    item: item_id("coal"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("sand"),
                    amount: 2
                }
            ]
        );

        let crucible = registry.get_crafting_by_name("silicon-crucible").unwrap();
        assert_eq!(crucible.kind, CraftingBlockKind::AttributeCrafter);
        assert_eq!(crucible.base.size, 3);
        assert_eq!(crucible.base.item_capacity, 30);
        assert_eq!(crucible.boost_scale, 0.15);
        assert_eq!(
            crucible.output_item,
            Some(ItemAmount {
                item: item_id("silicon"),
                amount: 8
            })
        );

        let kiln = registry.get_crafting_by_name("kiln").unwrap();
        assert_eq!(
            kiln.output_item,
            Some(ItemAmount {
                item: item_id("metaglass"),
                amount: 1
            })
        );
        assert_eq!(kiln.craft_time, 30.0);
        assert!(kiln.base.has_items);
        assert!(kiln.base.has_power);
    }

    #[test]
    fn liquid_and_space_enabled_crafting_blocks_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let plastanium = registry
            .get_crafting_by_name("plastanium-compressor")
            .unwrap();
        assert_eq!(plastanium.base.health, 320);
        assert_eq!(plastanium.base.liquid_capacity, 60.0);
        assert_eq!(plastanium.craft_effect, "formsmoke");
        assert_eq!(plastanium.update_effect, "plasticburn");
        assert_eq!(plastanium.consume_power, 3.0);
        assert_eq!(
            plastanium.output_item,
            Some(ItemAmount {
                item: item_id("plastanium"),
                amount: 1
            })
        );
        assert_eq!(
            plastanium.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("oil"),
                amount: 0.25
            }]
        );

        let phase = registry.get_crafting_by_name("phase-weaver").unwrap();
        assert_ne!(phase.base.env_enabled & Env::SPACE, 0);
        assert_eq!(phase.base.item_capacity, 30);
        assert_eq!(phase.ambient_sound, "loopTech");
        assert_eq!(
            phase.output_item,
            Some(ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            })
        );

        let surge = registry.get_crafting_by_name("surge-smelter").unwrap();
        assert_eq!(surge.base.size, 3);
        assert_eq!(surge.base.item_capacity, 20);
        assert_eq!(surge.consume_power, 4.0);
        assert_eq!(
            surge.output_item,
            Some(ItemAmount {
                item: item_id("surge-alloy"),
                amount: 1
            })
        );

        let cryo = registry.get_crafting_by_name("cryofluid-mixer").unwrap();
        assert!(cryo.outputs_liquid);
        assert!(!cryo.rotate);
        assert!(cryo.base.solid);
        assert_eq!(cryo.base.env_enabled, Env::ANY);
        assert_eq!(cryo.base.liquid_capacity, 36.0);
        assert_eq!(cryo.light_liquid, Some(liquid_id("cryofluid")));
        assert_eq!(
            cryo.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("cryofluid"),
                amount: 12.0 / 60.0
            })
        );

        let pyratite = registry.get_crafting_by_name("pyratite-mixer").unwrap();
        assert_ne!(pyratite.base.env_enabled & Env::SPACE, 0);
        assert_eq!(pyratite.ambient_sound, "loopMachineSpin");
        assert_eq!(pyratite.ambient_sound_volume, 0.1);
        assert_eq!(
            pyratite.output_item,
            Some(ItemAmount {
                item: item_id("pyratite"),
                amount: 1
            })
        );
    }

    #[test]
    fn separator_and_remaining_early_crafting_blocks_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let blast = registry.get_crafting_by_name("blast-mixer").unwrap();
        assert_ne!(blast.base.env_enabled & Env::SPACE, 0);
        assert_eq!(blast.ambient_sound, "loopMachineSpin");
        assert_eq!(blast.ambient_sound_volume, 0.12);
        assert_eq!(
            blast.output_item,
            Some(ItemAmount {
                item: item_id("blast-compound"),
                amount: 1
            })
        );
        assert_eq!(
            blast.consume_items,
            vec![
                ItemAmount {
                    item: item_id("pyratite"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("spore-pod"),
                    amount: 1
                }
            ]
        );

        let melter = registry.get_crafting_by_name("melter").unwrap();
        assert_eq!(melter.base.health, 200);
        assert_eq!(melter.craft_time, 10.0);
        assert_eq!(melter.consume_power, 1.0);
        assert_eq!(
            melter.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("slag"),
                amount: 12.0 / 60.0
            })
        );

        let separator = registry.get_crafting_by_name("separator").unwrap();
        assert_eq!(separator.kind, CraftingBlockKind::Separator);
        assert_eq!(separator.base.size, 2);
        assert_eq!(separator.craft_time, 35.0);
        assert_eq!(separator.consume_power, 1.1);
        assert_eq!(
            separator.results,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 5
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 3
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 2
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 2
                }
            ]
        );

        let disassembler = registry.get_crafting_by_name("disassembler").unwrap();
        assert_eq!(disassembler.kind, CraftingBlockKind::Separator);
        assert_eq!(disassembler.base.size, 3);
        assert_eq!(disassembler.base.item_capacity, 20);
        assert_eq!(disassembler.craft_time, 15.0);
        assert_eq!(disassembler.consume_power, 4.0);
        assert_eq!(
            disassembler.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("slag"),
                amount: 0.12
            }]
        );

        let spore = registry.get_crafting_by_name("spore-press").unwrap();
        assert_eq!(spore.base.health, 320);
        assert_eq!(spore.base.liquid_capacity, 60.0);
        assert_eq!(spore.craft_time, 20.0);
        assert_eq!(spore.consume_power, 0.7);
        assert_eq!(
            spore.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("oil"),
                amount: 18.0 / 60.0
            })
        );
    }

    #[test]
    fn next_low_risk_crafting_blocks_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let pulverizer = registry.get_crafting_by_name("pulverizer").unwrap();
        assert_eq!(pulverizer.output_item, item_amount(&all_items, "sand", 1));
        assert_eq!(pulverizer.craft_effect, "pulverize");
        assert_eq!(pulverizer.update_effect, "pulverizeSmall");
        assert_eq!(pulverizer.craft_time, 40.0);
        assert_eq!(pulverizer.ambient_sound, "loopGrind");
        assert_eq!(pulverizer.ambient_sound_volume, 0.025);
        assert_eq!(
            pulverizer.consume_items,
            vec![ItemAmount {
                item: item_id("scrap"),
                amount: 1
            }]
        );

        let coal = registry.get_crafting_by_name("coal-centrifuge").unwrap();
        assert_eq!(coal.base.size, 2);
        assert!(coal.base.has_items);
        assert!(coal.base.has_liquids);
        assert!(coal.base.has_power);
        assert!(!coal.rotate_draw);
        assert_eq!(coal.craft_effect, "coalSmeltsmoke");
        assert_eq!(coal.output_item, item_amount(&all_items, "coal", 1));
        assert_eq!(
            coal.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("oil"),
                amount: 0.1
            }]
        );

        let arc = registry
            .get_crafting_by_name("silicon-arc-furnace")
            .unwrap();
        assert_eq!(arc.base.size, 3);
        assert_eq!(arc.base.item_capacity, 30);
        assert_eq!(arc.base.env_enabled & Env::SPACE, Env::SPACE);
        assert_eq!(arc.base.env_enabled & Env::UNDERWATER, Env::UNDERWATER);
        assert_eq!(arc.base.env_disabled, Env::NONE);
        assert_eq!(arc.fog_radius, 3.0);
        assert_eq!(arc.output_item, item_amount(&all_items, "silicon", 4));
        assert_eq!(
            arc.research_cost,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 50
                }
            ]
        );

        let slag = registry.get_crafting_by_name("slag-centrifuge").unwrap();
        assert_eq!(
            slag.base.build_visibility,
            crate::mindustry::world::meta::BuildVisibility::DebugOnly
        );
        assert_eq!(slag.base.size, 3);
        assert_eq!(slag.base.liquid_capacity, 80.0);
        assert!(slag.outputs_liquid);
        assert_eq!(slag.craft_time, 120.0);
        assert_eq!(
            slag.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("gallium"),
                amount: 1.0 / 60.0
            })
        );
        assert_eq!(
            slag.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("slag"),
                amount: 40.0 / 60.0
            }]
        );
        assert_eq!(slag.consume_power, 2.0 / 60.0);
    }

    #[test]
    fn erekir_multi_liquid_and_heat_producer_blocks_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let electrolyzer = registry.get_crafting_by_name("electrolyzer").unwrap();
        assert_eq!(electrolyzer.kind, CraftingBlockKind::GenericCrafter);
        assert_eq!(
            electrolyzer.base.group,
            crate::mindustry::world::meta::BlockGroup::Liquids
        );
        assert_eq!(electrolyzer.base.item_capacity, 0);
        assert_eq!(electrolyzer.base.liquid_capacity, 50.0);
        assert!(electrolyzer.rotate);
        assert!(electrolyzer.invert_flip);
        assert_eq!(electrolyzer.region_rotated1, 3);
        assert_eq!(electrolyzer.liquid_output_directions, vec![1, 3]);
        assert_eq!(
            electrolyzer.output_liquids,
            vec![
                LiquidAmount {
                    liquid: liquid_id("ozone"),
                    amount: 4.0 / 60.0
                },
                LiquidAmount {
                    liquid: liquid_id("hydrogen"),
                    amount: 6.0 / 60.0
                }
            ]
        );

        let atmospheric = registry
            .get_crafting_by_name("atmospheric-concentrator")
            .unwrap();
        assert_eq!(atmospheric.kind, CraftingBlockKind::HeatCrafter);
        assert_eq!(atmospheric.base.size, 3);
        assert_eq!(atmospheric.base.item_capacity, 0);
        assert_eq!(atmospheric.base.liquid_capacity, 60.0);
        assert_eq!(atmospheric.heat_requirement, 24.0);
        assert_eq!(atmospheric.max_efficiency, 1.0);
        assert_eq!(
            atmospheric.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("nitrogen"),
                amount: 16.0 / 60.0
            })
        );

        let vent = registry.get_crafting_by_name("vent-condenser").unwrap();
        assert_eq!(vent.kind, CraftingBlockKind::AttributeCrafter);
        assert_eq!(vent.attribute, "steam");
        assert_eq!(
            vent.base.group,
            crate::mindustry::world::meta::BlockGroup::Liquids
        );
        assert_eq!(vent.min_efficiency, 9.0 - 0.0001);
        assert_eq!(vent.base_efficiency, 0.0);
        assert!(!vent.display_efficiency);
        assert_eq!(vent.craft_effect, "turbinegenerate");
        assert_eq!(vent.craft_time, 120.0);
        assert_eq!(vent.base.size, 3);
        assert_eq!(vent.ambient_sound, "loopHum");
        assert_eq!(vent.ambient_sound_volume, 0.06);
        assert!(vent.base.has_liquids);
        assert_eq!(vent.boost_scale, 1.0 / 9.0);
        assert_eq!(vent.base.item_capacity, 0);
        assert_eq!(
            vent.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("water"),
                amount: 30.0 / 60.0
            })
        );
        assert!(vent.outputs_liquid);
        assert_eq!(vent.consume_power, 0.5);
        assert!(vent.base.has_power);
        assert_eq!(vent.base.liquid_capacity, 60.0);
        assert_eq!(
            vent.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 60
                }
            ]
        );

        let oxidation = registry.get_crafting_by_name("oxidation-chamber").unwrap();
        assert_eq!(oxidation.kind, CraftingBlockKind::HeatProducer);
        assert!(!oxidation.rotate_draw);
        assert_eq!(oxidation.region_rotated1, 2);
        assert_eq!(oxidation.heat_output, 5.0);
        assert_eq!(oxidation.output_item, item_amount(&all_items, "oxide", 1));
        assert_eq!(
            oxidation.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("ozone"),
                amount: 2.0 / 60.0
            }]
        );

        let electric = registry.get_crafting_by_name("electric-heater").unwrap();
        assert_eq!(electric.kind, CraftingBlockKind::HeatProducer);
        assert_eq!(electric.base.size, 2);
        assert_eq!(electric.heat_output, 3.0);
        assert_eq!(electric.consume_power, 100.0 / 60.0);

        let slag_heater = registry.get_crafting_by_name("slag-heater").unwrap();
        assert_eq!(slag_heater.base.liquid_capacity, 120.0);
        assert_eq!(slag_heater.heat_output, 8.0);
        assert_eq!(
            slag_heater.research_cost,
            vec![
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 1200
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 900
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 2400
                }
            ]
        );

        let phase_heater = registry.get_crafting_by_name("phase-heater").unwrap();
        assert_eq!(phase_heater.heat_output, 15.0);
        assert_eq!(phase_heater.craft_time, 480.0);
        assert_eq!(
            phase_heater.consume_items,
            vec![ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            }]
        );
    }

    #[test]
    fn heat_chain_blocks_keep_upstream_heat_and_recipe_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        for (name, size, split) in [
            ("heat-redirector", 3, false),
            ("small-heat-redirector", 2, false),
            ("heat-router", 3, true),
        ] {
            let heat = registry.get_crafting_by_name(name).unwrap();
            assert_eq!(heat.kind, CraftingBlockKind::HeatConductor);
            assert_eq!(
                heat.base.group,
                crate::mindustry::world::meta::BlockGroup::Heat
            );
            assert_eq!(heat.base.size, size);
            assert_eq!(heat.region_rotated1, 1);
            assert_eq!(heat.split_heat, split);
        }

        let incinerator = registry.get_crafting_by_name("slag-incinerator").unwrap();
        assert_eq!(incinerator.kind, CraftingBlockKind::ItemIncinerator);
        assert_eq!(
            incinerator.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("slag"),
                amount: 0.0
            }]
        );

        let carbide = registry.get_crafting_by_name("carbide-crucible").unwrap();
        assert_eq!(carbide.kind, CraftingBlockKind::HeatCrafter);
        assert_eq!(carbide.heat_requirement, 40.0);
        assert_eq!(carbide.max_efficiency, 1.0);
        assert_eq!(carbide.output_item, item_amount(&all_items, "carbide", 1));
        assert_eq!(carbide.craft_time, 60.0 * 2.25 / 4.0);

        let surge = registry.get_crafting_by_name("surge-crucible").unwrap();
        assert_eq!(surge.kind, CraftingBlockKind::HeatCrafter);
        assert_eq!(surge.base.liquid_capacity, 400.0);
        assert_eq!(surge.heat_requirement, 40.0);
        assert_eq!(surge.craft_time, 45.0);
        assert_eq!(
            surge.output_item,
            Some(ItemAmount {
                item: item_id("surge-alloy"),
                amount: 1
            })
        );
        assert_eq!(
            surge.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("slag"),
                amount: 160.0 / 60.0
            }]
        );

        let cyanogen = registry
            .get_crafting_by_name("cyanogen-synthesizer")
            .unwrap();
        assert_eq!(cyanogen.heat_requirement, 20.0);
        assert_eq!(cyanogen.craft_time, 20.0);
        assert_eq!(
            cyanogen.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("cyanogen"),
                amount: 12.0 / 60.0
            })
        );

        let phase = registry.get_crafting_by_name("phase-synthesizer").unwrap();
        assert_eq!(phase.heat_requirement, 32.0);
        assert_eq!(phase.base.item_capacity, 40);
        assert_eq!(phase.base.liquid_capacity, 40.0);
        assert_eq!(
            phase.output_item,
            Some(ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            })
        );

        let reactor = registry.get_crafting_by_name("heat-reactor").unwrap();
        assert_eq!(reactor.kind, CraftingBlockKind::HeatProducer);
        assert_eq!(
            reactor.base.build_visibility,
            crate::mindustry::world::meta::BuildVisibility::DebugOnly
        );
        assert_eq!(reactor.craft_time, 600.0);
        assert_eq!(reactor.base.item_capacity, 20);
        assert_eq!(
            reactor.output_item,
            Some(ItemAmount {
                item: item_id("fissile-matter"),
                amount: 1
            })
        );
        assert_eq!(
            reactor.consume_items,
            vec![ItemAmount {
                item: item_id("thorium"),
                amount: 3
            }]
        );
        assert_eq!(
            reactor.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("nitrogen"),
                amount: 1.0 / 60.0
            }]
        );

        let source = registry.get_crafting_by_name("heat-source").unwrap();
        assert_eq!(source.kind, CraftingBlockKind::HeatProducer);
        assert_eq!(
            source.base.build_visibility,
            crate::mindustry::world::meta::BuildVisibility::SandboxOnly
        );
        assert_eq!(source.base.item_capacity, 0);
        assert_eq!(source.base.size, 1);
        assert!(!source.rotate_draw);
        assert_eq!(source.region_rotated1, 1);
        assert_eq!(source.heat_output, 1000.0);
        assert_eq!(source.warmup_rate, 1000.0);
        assert!(source.always_unlocked);
        assert!(source.all_database_tabs);
    }

    #[test]
    fn serpulo_basic_defense_walls_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        for (name, item, health, size, amount, research_multiplier) in [
            ("copper-wall", "copper", 320, 1, 6, 0.1),
            ("copper-wall-large", "copper", 1280, 2, 24, 1.0),
            ("titanium-wall", "titanium", 440, 1, 6, 1.0),
            ("titanium-wall-large", "titanium", 1760, 2, 24, 1.0),
            ("thorium-wall", "thorium", 800, 1, 6, 1.0),
            ("thorium-wall-large", "thorium", 3200, 2, 24, 1.0),
        ] {
            let wall = registry.get_defense_wall_by_name(name).unwrap();
            assert_eq!(wall.base.health, health, "{name} health");
            assert_eq!(wall.base.size, size, "{name} size");
            assert_eq!(wall.base.group, BlockGroup::Walls, "{name} group");
            assert_eq!(wall.base.cache_layer, CacheLayer::Normal, "{name} cache");
            assert!(wall.base.destructible, "{name} destructible");
            assert!(wall.base.solid, "{name} solid");
            assert!(!wall.base.unit_move_breakable, "{name} unit move");
            assert!(
                !wall.base.is_static(),
                "{name} should not be static terrain"
            );
            assert_eq!(wall.research_cost_multiplier, research_multiplier);
            assert_eq!(
                wall.requirements,
                vec![ItemAmount {
                    item: item_id(item),
                    amount
                }]
            );
        }
    }

    #[test]
    fn scrap_wall_family_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let scrap_id = find_item(&all_items, "scrap")
            .unwrap()
            .base
            .mappable
            .base
            .id;

        for (name, health, size, amount, variants) in [
            ("scrap-wall", 240, 1, 6, 5),
            ("scrap-wall-large", 960, 2, 24, 4),
            ("scrap-wall-huge", 2160, 3, 54, 3),
            ("scrap-wall-gigantic", 3840, 4, 96, 0),
        ] {
            let wall = registry.get_defense_wall_by_name(name).unwrap();
            assert_eq!(wall.base.health, health, "{name} health");
            assert_eq!(wall.base.size, size, "{name} size");
            assert_eq!(wall.base.variants, variants, "{name} variants");
            assert_eq!(wall.build_cost_multiplier, 4.0, "{name} build cost");
            assert_eq!(
                wall.requirements,
                vec![ItemAmount {
                    item: scrap_id,
                    amount
                }]
            );
        }
    }

    #[test]
    fn erekir_basic_defense_walls_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        for (name, item, health, size, amount, armor, build_cost) in [
            ("beryllium-wall", "beryllium", 520, 1, 6, 2.0, 8.0),
            ("beryllium-wall-large", "beryllium", 2080, 2, 24, 2.0, 5.0),
            ("tungsten-wall", "tungsten", 720, 1, 6, 14.0, 8.0),
            ("tungsten-wall-large", "tungsten", 2880, 2, 24, 14.0, 5.0),
        ] {
            let wall = registry.get_defense_wall_by_name(name).unwrap();
            assert_eq!(wall.base.health, health, "{name} health");
            assert_eq!(wall.base.size, size, "{name} size");
            assert_eq!(wall.armor, armor, "{name} armor");
            assert_eq!(wall.build_cost_multiplier, build_cost, "{name} build cost");
            assert_eq!(
                wall.requirements,
                vec![ItemAmount {
                    item: item_id(item),
                    amount
                }]
            );
        }

        let carbide = registry.get_defense_wall_by_name("carbide-wall").unwrap();
        assert_eq!(carbide.base.health, 1080);
        assert_eq!(carbide.armor, 16.0);
        assert_eq!(
            carbide.requirements,
            vec![
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 6
                },
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 6
                }
            ]
        );

        let carbide_large = registry
            .get_defense_wall_by_name("carbide-wall-large")
            .unwrap();
        assert_eq!(carbide_large.base.health, 4320);
        assert_eq!(carbide_large.base.size, 2);
        assert_eq!(carbide_large.armor, 16.0);
        assert_eq!(
            carbide_large.requirements,
            vec![
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 24
                },
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 24
                }
            ]
        );
    }

    #[test]
    fn special_defense_walls_keep_upstream_combat_flags() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        for (name, health, size, requirements) in [
            (
                "plastanium-wall",
                500,
                1,
                vec![
                    ItemAmount {
                        item: item_id("plastanium"),
                        amount: 5,
                    },
                    ItemAmount {
                        item: item_id("metaglass"),
                        amount: 2,
                    },
                ],
            ),
            (
                "plastanium-wall-large",
                2000,
                2,
                vec![
                    ItemAmount {
                        item: item_id("plastanium"),
                        amount: 20,
                    },
                    ItemAmount {
                        item: item_id("metaglass"),
                        amount: 8,
                    },
                ],
            ),
        ] {
            let wall = registry.get_defense_wall_by_name(name).unwrap();
            assert_eq!(wall.kind, DefenseWallKind::Wall);
            assert_eq!(wall.base.health, health);
            assert_eq!(wall.base.size, size);
            assert!(wall.insulated);
            assert!(wall.absorb_lasers);
            assert_eq!(wall.schematic_priority, 10);
            assert_eq!(wall.requirements, requirements);
        }

        for (name, health, size, amount) in
            [("phase-wall", 600, 1, 6), ("phase-wall-large", 2400, 2, 24)]
        {
            let wall = registry.get_defense_wall_by_name(name).unwrap();
            assert_eq!(wall.base.health, health);
            assert_eq!(wall.base.size, size);
            assert_eq!(wall.chance_deflect, 10.0);
            assert!(wall.flash_hit);
            assert_eq!(
                wall.requirements,
                vec![ItemAmount {
                    item: item_id("phase-fabric"),
                    amount
                }]
            );
        }

        for (name, health, size, amount) in
            [("surge-wall", 920, 1, 6), ("surge-wall-large", 3680, 2, 24)]
        {
            let wall = registry.get_defense_wall_by_name(name).unwrap();
            assert_eq!(wall.base.health, health);
            assert_eq!(wall.base.size, size);
            assert_eq!(wall.lightning_chance, 0.05);
            assert_eq!(wall.lightning_damage, 20.0);
            assert_eq!(
                wall.requirements,
                vec![ItemAmount {
                    item: item_id("surge-alloy"),
                    amount
                }]
            );
        }

        let reinforced = registry
            .get_defense_wall_by_name("reinforced-surge-wall")
            .unwrap();
        assert_eq!(reinforced.base.health, 1000);
        assert_eq!(reinforced.armor, 20.0);
        assert_eq!(reinforced.lightning_chance, 0.05);
        assert_eq!(reinforced.lightning_damage, 30.0);
        assert_eq!(
            reinforced.requirements,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 6
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 2
                }
            ]
        );
        assert_eq!(
            reinforced.research_cost,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 100
                }
            ]
        );

        let reinforced_large = registry
            .get_defense_wall_by_name("reinforced-surge-wall-large")
            .unwrap();
        assert_eq!(reinforced_large.base.health, 4000);
        assert_eq!(reinforced_large.base.size, 2);
        assert_eq!(reinforced_large.armor, 20.0);
        assert_eq!(reinforced_large.lightning_chance, 0.05);
        assert_eq!(reinforced_large.lightning_damage, 30.0);
        assert_eq!(
            reinforced_large.research_cost,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 200
                }
            ]
        );
    }

    #[test]
    fn door_and_shield_defense_walls_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let door = registry.get_defense_wall_by_name("door").unwrap();
        assert_eq!(door.kind, DefenseWallKind::Door);
        assert_eq!(door.base.health, 400);
        assert!(!door.base.solid);
        assert!(door.solidifies);
        assert!(door.consumes_tap);
        assert_eq!(
            door.requirements,
            vec![
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 6
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 4
                }
            ]
        );

        let door_large = registry.get_defense_wall_by_name("door-large").unwrap();
        assert_eq!(door_large.kind, DefenseWallKind::Door);
        assert_eq!(door_large.base.health, 1600);
        assert_eq!(door_large.base.size, 2);
        assert!(!door_large.base.solid);
        assert!(door_large.solidifies);
        assert!(door_large.consumes_tap);

        let blast = registry.get_defense_wall_by_name("blast-door").unwrap();
        assert_eq!(blast.kind, DefenseWallKind::AutoDoor);
        assert_eq!(blast.base.health, 2800);
        assert_eq!(blast.base.size, 2);
        assert_eq!(blast.armor, 14.0);
        assert!(blast.base.update);
        assert!(!blast.base.solid);
        assert!(blast.solidifies);
        assert!(blast.team_passable);
        assert!(blast.no_update_disabled);
        assert_eq!(
            blast.requirements,
            vec![
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 24
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 24
                }
            ]
        );

        let shield = registry.get_defense_wall_by_name("shielded-wall").unwrap();
        assert_eq!(shield.kind, DefenseWallKind::ShieldWall);
        assert_eq!(shield.base.health, 4160);
        assert_eq!(shield.base.size, 2);
        assert_eq!(shield.armor, 15.0);
        assert_eq!(shield.chance_deflect, 8.0);
        assert!(shield.base.update);
        assert!(shield.base.has_power);
        assert!(shield.base.consumes_power);
        assert!(shield.base.conductive_power);
        assert!(!shield.base.outputs_power);
        assert_eq!(shield.consume_power, 3.0 / 60.0);
        assert_eq!(shield.shield_health, 900.0);
        assert_eq!(shield.shield_break_cooldown, 60.0 * 10.0);
        assert_eq!(shield.shield_regen_speed, 2.0);
        assert_eq!(
            shield.requirements,
            vec![
                ItemAmount {
                    item: item_id("phase-fabric"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 12
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 12
                }
            ]
        );
    }

    #[test]
    fn mend_projectors_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let mender = registry.get_effect_by_name("mender").unwrap();
        assert_eq!(mender.kind, EffectBlockKind::MendProjector);
        assert_eq!(mender.base.group, BlockGroup::Projectors);
        assert!(mender.base.solid);
        assert!(mender.base.update);
        assert!(mender.base.has_power);
        assert!(mender.base.has_items);
        assert!(mender.base.emit_light);
        assert!(mender.base.flags.contains(&BlockFlag::BlockRepair));
        assert_eq!(mender.base.size, 1);
        assert_eq!(mender.base.health, 80);
        assert_eq!(mender.consume_power, 0.3);
        assert_eq!(mender.reload, 200.0);
        assert_eq!(mender.range, 40.0);
        assert_eq!(mender.heal_percent, 4.0);
        assert_eq!(mender.phase_boost, 4.0);
        assert_eq!(mender.phase_range_boost, 20.0);
        assert_eq!(
            mender.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("copper"),
                    amount: 25
                }
            ]
        );
        assert_eq!(
            mender.boost_items,
            vec![ItemAmount {
                item: item_id("silicon"),
                amount: 1
            }]
        );

        let projector = registry.get_effect_by_name("mend-projector").unwrap();
        assert_eq!(projector.kind, EffectBlockKind::MendProjector);
        assert_eq!(projector.base.size, 2);
        assert_eq!(projector.base.health, 320);
        assert_eq!(projector.consume_power, 1.5);
        assert_eq!(projector.reload, 250.0);
        assert_eq!(projector.range, 85.0);
        assert_eq!(projector.heal_percent, 11.0);
        assert_eq!(projector.phase_boost, 15.0);
        assert_eq!(
            projector.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 25
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("copper"),
                    amount: 50
                }
            ]
        );
        assert_eq!(
            projector.boost_items,
            vec![ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            }]
        );
    }

    #[test]
    fn overdrive_projectors_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let projector = registry.get_effect_by_name("overdrive-projector").unwrap();
        assert_eq!(projector.kind, EffectBlockKind::OverdriveProjector);
        assert_eq!(projector.base.group, BlockGroup::Projectors);
        assert!(projector.base.solid);
        assert!(projector.base.update);
        assert!(projector.base.has_power);
        assert!(projector.base.has_items);
        assert_eq!(projector.base.size, 2);
        assert_eq!(projector.consume_power, 3.5);
        assert_eq!(projector.reload, 60.0);
        assert_eq!(projector.range, 80.0);
        assert_eq!(projector.speed_boost, 1.5);
        assert_eq!(projector.speed_boost_phase, 0.75);
        assert_eq!(projector.use_time, 400.0);
        assert_eq!(projector.phase_range_boost, 20.0);
        assert!(projector.has_boost);
        assert_eq!(projector.ambient_sound, "loopCircuit");
        assert_eq!(projector.ambient_sound_volume, 0.08);
        assert_eq!(
            projector.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 75
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 75
                },
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 30
                }
            ]
        );
        assert_eq!(
            projector.boost_items,
            vec![ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            }]
        );

        let dome = registry.get_effect_by_name("overdrive-dome").unwrap();
        assert_eq!(dome.kind, EffectBlockKind::OverdriveProjector);
        assert_eq!(dome.base.size, 3);
        assert_eq!(dome.consume_power, 10.0);
        assert_eq!(dome.range, 200.0);
        assert_eq!(dome.speed_boost, 2.5);
        assert_eq!(dome.use_time, 300.0);
        assert_eq!(dome.ambient_sound_volume, 0.12);
        assert!(!dome.has_boost);
        assert!(dome.boost_items.is_empty());
        assert_eq!(
            dome.consume_items,
            vec![
                ItemAmount {
                    item: item_id("phase-fabric"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 1
                }
            ]
        );
        assert_eq!(
            dome.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 130
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 130
                },
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 120
                }
            ]
        );
    }

    #[test]
    fn force_projector_and_shock_mine_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let force = registry.get_effect_by_name("force-projector").unwrap();
        assert_eq!(force.kind, EffectBlockKind::ForceProjector);
        assert_eq!(force.base.group, BlockGroup::Projectors);
        assert!(force.base.solid);
        assert!(force.base.update);
        assert!(force.base.has_power);
        assert!(force.base.has_items);
        assert!(force.base.has_liquids);
        assert!(force.base.flags.contains(&BlockFlag::Shield));
        assert_eq!(force.base.size, 3);
        assert_eq!(force.consume_power, 4.0);
        assert_eq!(force.phase_use_time, 350.0);
        assert_eq!(force.phase_radius_boost, 80.0);
        assert_eq!(force.phase_shield_boost, 400.0);
        assert_eq!(force.radius, 101.7);
        assert_eq!(force.sides, 6);
        assert_eq!(force.shield_health, 750.0);
        assert_eq!(force.cooldown_normal, 1.5);
        assert_eq!(force.cooldown_liquid, 1.2);
        assert_eq!(force.cooldown_broken_base, 0.35);
        assert_eq!(force.coolant_consumption, 0.1);
        assert!(force.consume_coolant);
        assert_eq!(force.ambient_sound, "loopShield");
        assert_eq!(force.ambient_sound_volume, 0.1);
        assert_eq!(
            force.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 75
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 125
                }
            ]
        );
        assert_eq!(
            force.boost_items,
            vec![ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            }]
        );

        let mine = registry.get_effect_by_name("shock-mine").unwrap();
        assert_eq!(mine.kind, EffectBlockKind::ShockMine);
        assert!(mine.base.destructible);
        assert!(!mine.base.update);
        assert!(!mine.base.solid);
        assert!(!mine.base.targetable);
        assert!(!mine.base.has_shadow);
        assert_eq!(mine.base.health, 50);
        assert_eq!(mine.reload, 80.0);
        assert_eq!(mine.damage, 25.0);
        assert_eq!(mine.tile_damage, 7.0);
        assert_eq!(mine.length, 10);
        assert_eq!(mine.tendrils, 4);
        assert_eq!(mine.shots, 6);
        assert_eq!(mine.team_alpha, 0.3);
        assert_eq!(
            mine.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 25
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 12
                }
            ]
        );
    }

    #[test]
    fn radar_and_build_tower_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let radar = registry.get_effect_by_name("radar").unwrap();
        assert_eq!(radar.kind, EffectBlockKind::Radar);
        assert!(radar.base.update);
        assert!(radar.base.solid);
        assert!(radar.base.has_power);
        assert!(radar.base.consumes_power);
        assert!(radar.base.flags.contains(&BlockFlag::HasFogRadius));
        assert_eq!(radar.base.build_visibility, BuildVisibility::FogOnly);
        assert_eq!(radar.outline_color, "4a4b53");
        assert_eq!(radar.fog_radius, 34.0);
        assert_eq!(radar.discovery_time, 60.0 * 10.0);
        assert_eq!(radar.rotate_speed, 2.0);
        assert_eq!(radar.glow_color, "turretHeat");
        assert_eq!(radar.glow_scl, 5.0);
        assert_eq!(radar.glow_mag, 0.6);
        assert_eq!(radar.consume_power, 0.6);
        assert_eq!(
            radar.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 10
                }
            ]
        );
        assert_eq!(
            radar.research_cost,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 70
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 70
                }
            ]
        );

        let build_tower = registry.get_effect_by_name("build-tower").unwrap();
        assert_eq!(build_tower.kind, EffectBlockKind::BuildTurret);
        assert_eq!(build_tower.base.group, BlockGroup::Turrets);
        assert!(build_tower.base.update);
        assert!(build_tower.base.solid);
        assert!(build_tower.base.has_power);
        assert!(build_tower.base.has_liquids);
        assert!(build_tower.base.consumes_power);
        assert!(build_tower.base.flags.contains(&BlockFlag::Turret));
        assert_eq!(build_tower.outline_color, "darkOutline");
        assert_eq!(build_tower.range, 200.0);
        assert_eq!(build_tower.base.size, 3);
        assert_eq!(build_tower.build_speed, 1.5);
        assert_eq!(build_tower.build_beam_offset, 5.0);
        assert_eq!(build_tower.target_interval, 15);
        assert_eq!(build_tower.rotate_speed, 10.0);
        assert_eq!(build_tower.elevation, 1.5);
        assert_eq!(build_tower.consume_power, 3.0);
        assert_eq!(
            build_tower.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 60
                }
            ]
        );
        assert_eq!(
            build_tower.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("nitrogen"),
                amount: 3.0 / 60.0
            }]
        );
    }

    #[test]
    fn regen_and_shockwave_projectors_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let regen = registry.get_effect_by_name("regen-projector").unwrap();
        assert_eq!(regen.kind, EffectBlockKind::RegenProjector);
        assert_eq!(regen.base.group, BlockGroup::Projectors);
        assert!(regen.base.solid);
        assert!(regen.base.update);
        assert!(regen.base.has_power);
        assert!(regen.base.has_items);
        assert!(regen.base.has_liquids);
        assert!(regen.base.emit_light);
        assert!(regen.base.consumes_power);
        assert!(regen.base.flags.contains(&BlockFlag::BlockRepair));
        assert_eq!(regen.base.size, 3);
        assert_eq!(regen.range, 28.0);
        assert_eq!(regen.base_color, "regen");
        assert_eq!(regen.consume_power, 1.0);
        assert_eq!(regen.heal_percent, 4.0 / 60.0);
        assert_eq!(regen.optional_multiplier, 2.0);
        assert_eq!(regen.optional_use_time, 60.0 * 8.0);
        assert_eq!(regen.effect_chance, 0.003);
        assert_eq!(regen.ambient_sound, "loopRegen");
        assert_eq!(regen.ambient_sound_volume, 0.45);
        assert!(!regen.rotate_draw);
        assert_eq!(
            regen.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 80
                }
            ]
        );
        assert_eq!(
            regen.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("hydrogen"),
                amount: 1.0 / 60.0
            }]
        );
        assert_eq!(
            regen.boost_items,
            vec![ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            }]
        );
        assert!(regen.drawer.contains("DrawLiquidTile(hydrogen)"));

        assert!(
            registry.get_effect_by_name("barrier-projector").is_none(),
            "upstream keeps barrierProjector behind if(false), so it must not be registered"
        );

        let shockwave = registry.get_effect_by_name("shockwave-tower").unwrap();
        assert_eq!(shockwave.kind, EffectBlockKind::ShockwaveTower);
        assert!(shockwave.base.update);
        assert!(shockwave.base.solid);
        assert!(shockwave.base.has_power);
        assert!(shockwave.base.has_liquids);
        assert_eq!(shockwave.base.size, 3);
        assert_eq!(shockwave.consume_power, 100.0 / 60.0);
        assert_eq!(shockwave.range, 170.0);
        assert_eq!(shockwave.reload, 80.0);
        assert_eq!(shockwave.bullet_damage, 160.0);
        assert_eq!(shockwave.falloff_count, 20.0);
        assert_eq!(shockwave.shake, 2.0);
        assert_eq!(shockwave.check_interval, 8.0);
        assert_eq!(shockwave.cooldown_multiplier, 1.0);
        assert_eq!(shockwave.shape_rotate_speed, 1.0);
        assert_eq!(shockwave.shape_radius, 6.0);
        assert_eq!(shockwave.shape_sides, 4);
        assert_eq!(
            shockwave.requirements,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 100
                }
            ]
        );
        assert_eq!(
            shockwave.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("cyanogen"),
                amount: 1.5 / 60.0
            }]
        );
    }

    #[test]
    fn base_shield_projectors_keep_upstream_subset() {
        let (_, _, registry) = load_test_registry();

        let shield = registry.get_effect_by_name("shield-projector").unwrap();
        assert_eq!(shield.kind, EffectBlockKind::BaseShield);
        assert_eq!(shield.base.build_visibility, BuildVisibility::EditorOnly);
        assert!(shield.base.has_power);
        assert!(shield.base.update);
        assert!(shield.base.solid);
        assert!(shield.base.consumes_power);
        assert!(!shield.rebuildable);
        assert_eq!(shield.base.size, 3);
        assert_eq!(shield.radius, 200.0);
        assert_eq!(shield.sides, 24);
        assert_eq!(shield.consume_power, 5.0);
        assert!(shield.requirements.is_empty());

        let large = registry
            .get_effect_by_name("large-shield-projector")
            .unwrap();
        assert_eq!(large.kind, EffectBlockKind::BaseShield);
        assert_eq!(large.base.build_visibility, BuildVisibility::EditorOnly);
        assert!(large.base.has_power);
        assert!(large.base.update);
        assert!(large.base.solid);
        assert!(large.base.consumes_power);
        assert!(!large.rebuildable);
        assert_eq!(large.base.size, 4);
        assert_eq!(large.radius, 400.0);
        assert_eq!(large.sides, 24);
        assert_eq!(large.consume_power, 5.0);
        assert!(large.requirements.is_empty());
    }

    #[test]
    fn serpulo_conveyors_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let conveyor = registry.get_distribution_by_name("conveyor").unwrap();
        assert_eq!(conveyor.kind, DistributionBlockKind::Conveyor);
        assert_eq!(conveyor.base.group, BlockGroup::Transportation);
        assert!(conveyor.rotate);
        assert!(conveyor.base.update);
        assert!(conveyor.base.has_items);
        assert_eq!(conveyor.base.item_capacity, 3);
        assert_eq!(conveyor.base.health, 45);
        assert_eq!(conveyor.speed, 0.03);
        assert_eq!(conveyor.displayed_speed, 4.2);
        assert_eq!(conveyor.build_cost_multiplier, 2.0);
        assert_eq!(conveyor.ambient_sound, "loopConveyor");
        assert_eq!(conveyor.ambient_sound_volume, 0.0022);
        assert!(!conveyor.unloadable);
        assert_eq!(
            conveyor.requirements,
            vec![ItemAmount {
                item: item_id("copper"),
                amount: 1
            }]
        );
        assert_eq!(
            conveyor.research_cost,
            vec![ItemAmount {
                item: item_id("copper"),
                amount: 5
            }]
        );

        let titanium = registry
            .get_distribution_by_name("titanium-conveyor")
            .unwrap();
        assert_eq!(titanium.kind, DistributionBlockKind::Conveyor);
        assert_eq!(titanium.base.health, 65);
        assert_eq!(titanium.speed, 0.08);
        assert_eq!(titanium.displayed_speed, 11.0);
        assert_eq!(
            titanium.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 1
                }
            ]
        );

        let plastanium = registry
            .get_distribution_by_name("plastanium-conveyor")
            .unwrap();
        assert_eq!(plastanium.kind, DistributionBlockKind::StackConveyor);
        assert_eq!(plastanium.base.health, 90);
        assert_eq!(plastanium.speed, 4.0 / 60.0);
        assert_eq!(plastanium.base.item_capacity, 10);
        assert!(plastanium.output_router);
        assert_eq!(plastanium.recharge, 2.0);
        assert_eq!(plastanium.ambient_sound_volume, 0.004);
        assert_eq!(
            plastanium.requirements,
            vec![
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 1
                }
            ]
        );

        let armored = registry
            .get_distribution_by_name("armored-conveyor")
            .unwrap();
        assert_eq!(armored.kind, DistributionBlockKind::ArmoredConveyor);
        assert_eq!(armored.base.health, 280);
        assert_eq!(armored.speed, 0.08);
        assert_eq!(armored.displayed_speed, 11.0);
        assert!(armored.no_side_blend);
    }

    #[test]
    fn serpulo_bridge_sorter_router_and_gate_blocks_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let junction = registry.get_distribution_by_name("junction").unwrap();
        assert_eq!(junction.kind, DistributionBlockKind::Junction);
        assert!(junction.base.update);
        assert!(!junction.base.solid);
        assert_eq!(junction.base.health, 30);
        assert_eq!(junction.speed, 26.0);
        assert_eq!(junction.capacity, 6);
        assert_eq!(junction.build_cost_multiplier, 6.0);
        assert!(junction.no_update_disabled);
        assert!(!junction.unloadable);

        let bridge = registry
            .get_distribution_by_name("bridge-conveyor")
            .unwrap();
        assert_eq!(bridge.kind, DistributionBlockKind::BufferedItemBridge);
        assert!(bridge.base.update);
        assert!(bridge.base.solid);
        assert!(bridge.base.has_items);
        assert!(!bridge.base.has_power);
        assert!(!bridge.fade_in);
        assert!(!bridge.move_arrows);
        assert_eq!(bridge.range, 4.0);
        assert_eq!(bridge.speed, 74.0);
        assert_eq!(bridge.arrow_spacing, 6.0);
        assert_eq!(bridge.buffer_capacity, 14);
        assert!(bridge.crush_fragile);
        assert!(bridge.can_overdrive);
        assert_eq!(
            bridge.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 6
                },
                ItemAmount {
                    item: item_id("copper"),
                    amount: 6
                }
            ]
        );

        let phase = registry.get_distribution_by_name("phase-conveyor").unwrap();
        assert_eq!(phase.kind, DistributionBlockKind::ItemBridge);
        assert_eq!(phase.range, 12.0);
        assert_eq!(phase.transport_time, 2.0);
        assert_eq!(phase.arrow_period, 0.9);
        assert_eq!(phase.arrow_time_scl, 2.75);
        assert!(phase.base.has_power);
        assert!(phase.base.consumes_power);
        assert!(phase.pulse);
        assert_eq!(phase.consume_power, 0.30);
        assert_ne!(phase.base.env_enabled & Env::SPACE, 0);

        let sorter = registry.get_distribution_by_name("sorter").unwrap();
        assert_eq!(sorter.kind, DistributionBlockKind::Sorter);
        assert!(sorter.instant_transfer);
        assert!(sorter.configurable);
        assert!(sorter.save_config);
        assert!(sorter.clear_on_double_tap);
        assert!(!sorter.invert);
        assert_eq!(sorter.build_cost_multiplier, 3.0);

        let inverted = registry
            .get_distribution_by_name("inverted-sorter")
            .unwrap();
        assert_eq!(inverted.kind, DistributionBlockKind::Sorter);
        assert!(inverted.invert);
        assert_eq!(inverted.build_cost_multiplier, 3.0);

        let router = registry.get_distribution_by_name("router").unwrap();
        assert_eq!(router.kind, DistributionBlockKind::Router);
        assert!(!router.base.solid);
        assert!(router.base.update);
        assert!(router.base.has_items);
        assert_eq!(router.base.item_capacity, 1);
        assert_eq!(router.speed, 8.0);
        assert_eq!(router.build_cost_multiplier, 4.0);

        let distributor = registry.get_distribution_by_name("distributor").unwrap();
        assert_eq!(distributor.kind, DistributionBlockKind::Router);
        assert_eq!(distributor.base.size, 2);
        assert_eq!(distributor.build_cost_multiplier, 3.0);

        let overflow = registry.get_distribution_by_name("overflow-gate").unwrap();
        assert_eq!(overflow.kind, DistributionBlockKind::OverflowGate);
        assert!(overflow.base.has_items);
        assert!(overflow.instant_transfer);
        assert_eq!(overflow.base.item_capacity, 0);
        assert_eq!(overflow.speed, 1.0);
        assert!(!overflow.invert);

        let underflow = registry.get_distribution_by_name("underflow-gate").unwrap();
        assert_eq!(underflow.kind, DistributionBlockKind::OverflowGate);
        assert!(underflow.invert);
        assert_eq!(underflow.build_cost_multiplier, 3.0);
    }

    #[test]
    fn unloader_and_mass_driver_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let unloader = registry.get_distribution_by_name("unloader").unwrap();
        assert_eq!(unloader.kind, DistributionBlockKind::Unloader);
        assert_eq!(unloader.base.group, BlockGroup::Transportation);
        assert!(unloader.base.update);
        assert!(unloader.base.solid);
        assert_eq!(unloader.base.health, 70);
        assert!(unloader.base.has_items);
        assert_eq!(unloader.base.item_capacity, 0);
        assert!(unloader.configurable);
        assert!(unloader.save_config);
        assert!(unloader.clear_on_double_tap);
        assert!(unloader.allow_core_unload);
        assert_eq!(unloader.speed, 60.0 / 11.0);
        assert_eq!(
            unloader.requirements,
            vec![
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 25
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 30
                }
            ]
        );

        let driver = registry.get_distribution_by_name("mass-driver").unwrap();
        assert_eq!(driver.kind, DistributionBlockKind::MassDriver);
        assert!(driver.base.update);
        assert!(driver.base.solid);
        assert!(driver.configurable);
        assert!(driver.base.has_items);
        assert!(driver.base.has_power);
        assert!(driver.base.consumes_power);
        assert!(driver.base.sync);
        assert_ne!(driver.base.env_enabled & Env::SPACE, 0);
        assert_eq!(driver.base.size, 3);
        assert_eq!(driver.base.item_capacity, 120);
        assert_eq!(driver.reload, 200.0);
        assert_eq!(driver.range, 440.0);
        assert_eq!(driver.consume_power, 1.75);
        assert_eq!(driver.rotate_speed, 5.0);
        assert_eq!(driver.translation, 7.0);
        assert_eq!(driver.min_distribute, 10);
        assert_eq!(driver.knockback, 4.0);
        assert_eq!(driver.bullet_speed, 5.5);
        assert_eq!(driver.bullet_lifetime, 200.0);
        assert_eq!(driver.shoot_sound_volume, 0.5);
        assert_eq!(driver.shake, 3.0);
        assert_eq!(
            driver.requirements,
            vec![
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 125
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 75
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 125
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 50
                }
            ]
        );
    }

    #[test]
    fn erekir_duct_transport_blocks_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let duct_env = Env::SPACE | Env::TERRESTRIAL | Env::UNDERWATER;

        let duct = registry.get_distribution_by_name("duct").unwrap();
        assert_eq!(duct.kind, DistributionBlockKind::Duct);
        assert_eq!(duct.base.group, BlockGroup::Transportation);
        assert!(duct.base.update);
        assert!(!duct.base.solid);
        assert!(duct.base.has_items);
        assert_eq!(duct.base.item_capacity, 1);
        assert!(duct.rotate);
        assert!(duct.no_side_blend);
        assert!(duct.is_duct);
        assert_eq!(duct.base.env_enabled, duct_env);
        assert_eq!(duct.base.health, 90);
        assert_eq!(duct.speed, 4.0);
        assert_eq!(
            duct.requirements,
            vec![ItemAmount {
                item: item_id("beryllium"),
                amount: 1
            }]
        );
        assert_eq!(
            duct.research_cost,
            vec![ItemAmount {
                item: item_id("beryllium"),
                amount: 5
            }]
        );

        let armored = registry.get_distribution_by_name("armored-duct").unwrap();
        assert_eq!(armored.kind, DistributionBlockKind::Duct);
        assert!(armored.armored);
        assert_eq!(armored.base.health, 140);
        assert_eq!(armored.speed, 4.0);
        assert_eq!(
            armored.requirements,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 2
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 1
                }
            ]
        );
        assert_eq!(
            armored.research_cost,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 300
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 100
                }
            ]
        );

        let router = registry.get_distribution_by_name("duct-router").unwrap();
        assert_eq!(router.kind, DistributionBlockKind::DuctRouter);
        assert!(router.base.update);
        assert!(!router.base.solid);
        assert!(router.base.has_items);
        assert!(router.configurable);
        assert!(router.save_config);
        assert!(router.clear_on_double_tap);
        assert!(router.rotate);
        assert_eq!(router.region_rotated1, 1);
        assert_eq!(router.base.env_enabled, duct_env);
        assert_eq!(router.base.health, 90);
        assert_eq!(router.speed, 4.0);
        assert_eq!(
            router.research_cost,
            vec![ItemAmount {
                item: item_id("beryllium"),
                amount: 30
            }]
        );

        for (name, inverted) in [("overflow-duct", false), ("underflow-duct", true)] {
            let duct_gate = registry.get_distribution_by_name(name).unwrap();
            assert_eq!(duct_gate.kind, DistributionBlockKind::OverflowDuct);
            assert!(duct_gate.base.update);
            assert!(!duct_gate.base.solid);
            assert!(duct_gate.base.has_items);
            assert!(duct_gate.rotate);
            assert_eq!(duct_gate.region_rotated1, 1);
            assert_eq!(duct_gate.base.env_enabled, duct_env);
            assert_eq!(duct_gate.base.health, 90);
            assert_eq!(duct_gate.speed, 4.0);
            assert_eq!(duct_gate.research_cost_multiplier, 1.5);
            assert_eq!(duct_gate.invert, inverted);
        }

        let bridge = registry.get_distribution_by_name("duct-bridge").unwrap();
        assert_eq!(bridge.kind, DistributionBlockKind::DuctBridge);
        assert!(bridge.base.update);
        assert!(bridge.base.solid);
        assert!(bridge.rotate);
        assert!(bridge.base.has_items);
        assert_eq!(bridge.base.item_capacity, 4);
        assert!(bridge.is_duct);
        assert_eq!(bridge.range, 4.0);
        assert_eq!(bridge.region_rotated1, 1);
        assert_eq!(bridge.base.env_enabled, duct_env);
        assert_eq!(bridge.base.health, 90);
        assert_eq!(bridge.speed, 4.0);
        assert_eq!(bridge.build_cost_multiplier, 2.0);
        assert_eq!(bridge.research_cost_multiplier, 0.3);
        assert!(bridge.crush_fragile);

        let unloader = registry.get_distribution_by_name("duct-unloader").unwrap();
        assert_eq!(unloader.kind, DistributionBlockKind::DirectionalUnloader);
        assert_eq!(unloader.base.group, BlockGroup::Transportation);
        assert!(unloader.base.update);
        assert!(!unloader.base.solid);
        assert!(unloader.base.has_items);
        assert!(unloader.configurable);
        assert!(unloader.save_config);
        assert!(unloader.rotate);
        assert_eq!(unloader.base.item_capacity, 0);
        assert!(unloader.is_duct);
        assert!(!unloader.allow_core_unload);
        assert_eq!(unloader.region_rotated1, 1);
        assert_eq!(unloader.base.health, 120);
        assert_eq!(unloader.speed, 4.0);
        assert_eq!(
            unloader.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 10
                }
            ]
        );
    }

    #[test]
    fn surge_conveyor_keeps_upstream_powered_stack_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let surge = registry.get_distribution_by_name("surge-conveyor").unwrap();
        assert_eq!(surge.kind, DistributionBlockKind::StackConveyor);
        assert_eq!(surge.base.health, 130);
        assert_eq!(surge.speed, 5.0 / 60.0);
        assert_eq!(surge.base.item_capacity, 10);
        assert!(!surge.output_router);
        assert!(surge.base.has_power);
        assert!(surge.base.consumes_power);
        assert!(surge.base.conductive_power);
        assert!(surge.under_bullets);
        assert_eq!(surge.base_efficiency, 1.0);
        assert_eq!(surge.consume_power, 1.0 / 60.0);
        assert_eq!(
            surge.requirements,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 1
                }
            ]
        );
        assert_eq!(
            surge.research_cost,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 80
                }
            ]
        );
    }

    #[test]
    fn surge_router_and_unit_cargo_distribution_blocks_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let surge_router = registry.get_distribution_by_name("surge-router").unwrap();
        assert_eq!(surge_router.kind, DistributionBlockKind::StackRouter);
        assert_eq!(surge_router.base.health, 130);
        assert_eq!(surge_router.base.item_capacity, 10);
        assert_eq!(surge_router.speed, 6.0);
        assert!(surge_router.base.has_power);
        assert!(surge_router.base.consumes_power);
        assert!(surge_router.base.conductive_power);
        assert_eq!(surge_router.base_efficiency, 1.0);
        assert!(surge_router.under_bullets);
        assert!(!surge_router.base.solid);
        assert_eq!(surge_router.consume_power, 3.0 / 60.0);
        assert_eq!(
            surge_router.requirements,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 5
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 1
                }
            ]
        );

        let loader = registry
            .get_distribution_by_name("unit-cargo-loader")
            .unwrap();
        assert_eq!(loader.kind, DistributionBlockKind::UnitCargoLoader);
        assert!(loader.base.solid);
        assert!(loader.base.update);
        assert!(loader.base.has_items);
        assert!(loader.base.has_power);
        assert!(loader.base.has_liquids);
        assert_eq!(loader.base.size, 3);
        assert_eq!(loader.unit_build_time, 60.0 * 8.0);
        assert_eq!(loader.consume_power, 8.0 / 60.0);
        assert_eq!(loader.base.item_capacity, 200);
        assert_eq!(loader.ambient_sound, "loopUnitBuilding");
        assert_eq!(loader.poly_stroke, 1.8);
        assert_eq!(loader.poly_radius, 8.0);
        assert_eq!(loader.poly_sides, 6);
        assert_eq!(loader.poly_rotate_speed, 1.0);
        assert_eq!(loader.poly_color, "accent");
        assert_eq!(
            loader.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("nitrogen"),
                amount: 10.0 / 60.0
            }]
        );
        assert_eq!(
            loader.research_cost,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 2500
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 30
                }
            ]
        );

        let unload = registry
            .get_distribution_by_name("unit-cargo-unload-point")
            .unwrap();
        assert_eq!(unload.kind, DistributionBlockKind::UnitCargoUnloadPoint);
        assert!(unload.base.update);
        assert!(unload.base.solid);
        assert!(unload.base.has_items);
        assert!(unload.configurable);
        assert!(unload.save_config);
        assert!(unload.clear_on_double_tap);
        assert!(unload.base.flags.contains(&BlockFlag::UnitCargoUnloadPoint));
        assert_eq!(unload.stale_time_duration, 60.0 * 6.0);
        assert_eq!(unload.base.size, 2);
        assert_eq!(unload.base.item_capacity, 100);
        assert_eq!(
            unload.research_cost,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 3000
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 20
                }
            ]
        );
    }

    #[test]
    fn serpulo_pumps_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let mechanical = registry.get_liquid_by_name("mechanical-pump").unwrap();
        assert_eq!(mechanical.kind, LiquidBlockKind::Pump);
        assert_eq!(mechanical.base.group, BlockGroup::Liquids);
        assert!(mechanical.base.update);
        assert!(mechanical.base.solid);
        assert!(mechanical.base.has_liquids);
        assert!(mechanical.outputs_liquid);
        assert!(mechanical.floating);
        assert_eq!(mechanical.base.env_enabled, Env::TERRESTRIAL);
        assert_eq!(mechanical.pump_amount, 7.0 / 60.0);
        assert_eq!(mechanical.base.liquid_capacity, 20.0);
        assert_eq!(mechanical.consume_time, 60.0 * 5.0);
        assert_eq!(mechanical.warmup_speed, 0.019);
        assert_eq!(mechanical.drawer, "DrawMulti(DrawDefault, DrawPumpLiquid)");
        assert_eq!(
            mechanical.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 15
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 10
                }
            ]
        );

        let rotary = registry.get_liquid_by_name("rotary-pump").unwrap();
        assert_eq!(rotary.kind, LiquidBlockKind::Pump);
        assert_eq!(rotary.pump_amount, 0.2);
        assert_eq!(rotary.consume_power, 0.3);
        assert!(rotary.base.has_power);
        assert!(rotary.base.consumes_power);
        assert_eq!(rotary.base.liquid_capacity, 80.0);
        assert_eq!(rotary.base.size, 2);

        let impulse = registry.get_liquid_by_name("impulse-pump").unwrap();
        assert_eq!(impulse.kind, LiquidBlockKind::Pump);
        assert_eq!(impulse.pump_amount, 0.22);
        assert_eq!(impulse.consume_power, 1.3);
        assert!(impulse.base.has_power);
        assert_eq!(impulse.base.liquid_capacity, 200.0);
        assert_eq!(impulse.base.size, 3);
        assert_eq!(
            impulse.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 90
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 35
                }
            ]
        );
    }

    #[test]
    fn serpulo_conduits_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let conduit = registry.get_liquid_by_name("conduit").unwrap();
        assert_eq!(conduit.kind, LiquidBlockKind::Conduit);
        assert_eq!(conduit.base.group, BlockGroup::Liquids);
        assert!(conduit.base.update);
        assert!(!conduit.base.solid);
        assert!(conduit.base.has_liquids);
        assert!(conduit.outputs_liquid);
        assert!(conduit.rotate);
        assert!(conduit.floating);
        assert!(conduit.under_bullets);
        assert!(conduit.no_update_disabled);
        assert!(!conduit.can_overdrive);
        assert!(conduit.leaks);
        assert!(conduit.pad_corners);
        assert_eq!(conduit.bot_color, "565656");
        assert_eq!(conduit.base.liquid_capacity, 20.0);
        assert_eq!(conduit.base.health, 45);
        assert_eq!(conduit.explosiveness_scale, 10.0 / 20.0);
        assert_eq!(conduit.flammability_scale, 10.0 / 20.0);
        assert_eq!(
            conduit.requirements,
            vec![ItemAmount {
                item: item_id("metaglass"),
                amount: 1
            }]
        );

        let pulse = registry.get_liquid_by_name("pulse-conduit").unwrap();
        assert_eq!(pulse.kind, LiquidBlockKind::Conduit);
        assert_eq!(pulse.base.liquid_capacity, 40.0);
        assert_eq!(pulse.liquid_pressure, 1.025);
        assert_eq!(pulse.base.health, 90);
        assert_eq!(pulse.explosiveness_scale, 16.0 / 40.0);
        assert_eq!(
            pulse.requirements,
            vec![
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 2
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 1
                }
            ]
        );

        let plated = registry.get_liquid_by_name("plated-conduit").unwrap();
        assert_eq!(plated.kind, LiquidBlockKind::ArmoredConduit);
        assert!(!plated.leaks);
        assert_eq!(plated.base.liquid_capacity, 50.0);
        assert_eq!(plated.liquid_pressure, 1.025);
        assert_eq!(plated.base.health, 220);
        assert_eq!(plated.explosiveness_scale, 16.0 / 50.0);
        assert_eq!(
            plated.requirements,
            vec![
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 2
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 1
                },
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 1
                }
            ]
        );
    }

    #[test]
    fn serpulo_liquid_router_junction_and_bridge_blocks_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let router = registry.get_liquid_by_name("liquid-router").unwrap();
        assert_eq!(router.kind, LiquidBlockKind::LiquidRouter);
        assert!(router.base.update);
        assert!(!router.base.solid);
        assert!(router.base.has_liquids);
        assert!(router.outputs_liquid);
        assert!(router.floating);
        assert!(router.no_update_disabled);
        assert!(!router.can_overdrive);
        assert!(router.under_bullets);
        assert_eq!(router.base.liquid_capacity, 120.0);
        assert_eq!(router.explosiveness_scale, 20.0 / 120.0);
        assert_eq!(
            router.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 4
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 2
                }
            ]
        );

        let container = registry.get_liquid_by_name("liquid-container").unwrap();
        assert_eq!(container.kind, LiquidBlockKind::LiquidRouter);
        assert_eq!(container.base.liquid_capacity, 700.0);
        assert_eq!(container.base.size, 2);
        assert!(container.base.solid);

        let tank = registry.get_liquid_by_name("liquid-tank").unwrap();
        assert_eq!(tank.kind, LiquidBlockKind::LiquidRouter);
        assert_eq!(tank.base.size, 3);
        assert!(tank.base.solid);
        assert_eq!(tank.base.liquid_capacity, 1800.0);
        assert_eq!(tank.base.health, 500);

        let junction = registry.get_liquid_by_name("liquid-junction").unwrap();
        assert_eq!(junction.kind, LiquidBlockKind::LiquidJunction);
        assert!(junction.floating);
        assert!(!junction.base.solid);
        assert_eq!(
            junction.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 4
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 8
                }
            ]
        );

        let bridge = registry.get_liquid_by_name("bridge-conduit").unwrap();
        assert_eq!(bridge.kind, LiquidBlockKind::LiquidBridge);
        assert!(bridge.base.update);
        assert!(bridge.base.solid);
        assert!(!bridge.base.has_items);
        assert!(bridge.base.has_liquids);
        assert!(bridge.outputs_liquid);
        assert!(bridge.floating);
        assert!(!bridge.fade_in);
        assert!(!bridge.move_arrows);
        assert_eq!(bridge.arrow_spacing, 6.0);
        assert_eq!(bridge.range, 4.0);
        assert!(!bridge.base.has_power);
        assert_eq!(bridge.base.liquid_capacity, 100.0);
        assert_eq!(bridge.explosiveness_scale, 20.0 / 100.0);
        assert_eq!(bridge.base.env_enabled, Env::ANY);

        let phase = registry.get_liquid_by_name("phase-conduit").unwrap();
        assert_eq!(phase.kind, LiquidBlockKind::LiquidBridge);
        assert_eq!(phase.range, 12.0);
        assert_eq!(phase.arrow_period, 0.9);
        assert_eq!(phase.arrow_time_scl, 2.75);
        assert!(phase.base.has_power);
        assert!(phase.base.consumes_power);
        assert!(!phase.can_overdrive);
        assert!(phase.pulse);
        assert_eq!(phase.base.liquid_capacity, 100.0);
        assert_eq!(phase.consume_power, 0.30);
        assert_eq!(
            phase.requirements,
            vec![
                ItemAmount {
                    item: item_id("phase-fabric"),
                    amount: 5
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 7
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 10
                }
            ]
        );
    }

    #[test]
    fn reinforced_liquid_blocks_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let pump = registry.get_liquid_by_name("reinforced-pump").unwrap();
        assert_eq!(pump.kind, LiquidBlockKind::Pump);
        assert_eq!(pump.base.size, 2);
        assert_eq!(pump.pump_amount, 80.0 / 60.0 / 4.0);
        assert_eq!(pump.base.liquid_capacity, 160.0);
        assert_eq!(
            pump.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("hydrogen"),
                amount: 1.5 / 60.0
            }]
        );
        assert_eq!(
            pump.requirements,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 20
                }
            ]
        );

        let conduit = registry.get_liquid_by_name("reinforced-conduit").unwrap();
        let junction_id = registry.id_by_name("reinforced-liquid-junction").unwrap();
        let bridge_id = registry.id_by_name("reinforced-bridge-conduit").unwrap();
        assert_eq!(conduit.kind, LiquidBlockKind::ArmoredConduit);
        assert_eq!(conduit.bot_color, "darkestMetal");
        assert!(conduit.leaks);
        assert_eq!(conduit.base.liquid_capacity, 50.0);
        assert_eq!(conduit.liquid_pressure, 1.03);
        assert_eq!(conduit.base.health, 250);
        assert_eq!(conduit.research_cost_multiplier, 3.0);
        assert!(conduit.under_bullets);
        assert_eq!(conduit.explosiveness_scale, 20.0 / 50.0);
        assert_eq!(conduit.junction_replacement, Some(junction_id));
        assert_eq!(conduit.rot_bridge_replacement, Some(bridge_id));

        let junction = registry
            .get_liquid_by_name("reinforced-liquid-junction")
            .unwrap();
        assert_eq!(junction.kind, LiquidBlockKind::LiquidJunction);
        assert_eq!(junction.build_cost_multiplier, 3.0);
        assert_eq!(junction.base.health, 250);
        assert_eq!(junction.research_cost_multiplier, 1.0);
        assert!(!junction.base.solid);
        assert!(junction.under_bullets);

        let bridge = registry
            .get_liquid_by_name("reinforced-bridge-conduit")
            .unwrap();
        assert_eq!(bridge.kind, LiquidBlockKind::DirectionLiquidBridge);
        assert_eq!(bridge.base.group, BlockGroup::Liquids);
        assert_eq!(bridge.range, 4.0);
        assert!(!bridge.base.has_power);
        assert_eq!(bridge.base.liquid_capacity, 120.0);
        assert_eq!(bridge.research_cost_multiplier, 1.0);
        assert!(bridge.under_bullets);
        assert_eq!(bridge.base.health, 250);
        assert_eq!(bridge.explosiveness_scale, 20.0 / 120.0);

        let router = registry
            .get_liquid_by_name("reinforced-liquid-router")
            .unwrap();
        assert_eq!(router.kind, LiquidBlockKind::LiquidRouter);
        assert_eq!(router.base.liquid_capacity, 150.0);
        assert_eq!(router.liquid_padding, 3.0 / 4.0);
        assert_eq!(router.research_cost_multiplier, 3.0);
        assert!(router.under_bullets);
        assert!(!router.base.solid);
        assert_eq!(router.base.health, 250);
        assert_eq!(router.explosiveness_scale, 40.0 / 150.0);

        let container = registry
            .get_liquid_by_name("reinforced-liquid-container")
            .unwrap();
        assert_eq!(container.base.liquid_capacity, 1000.0);
        assert_eq!(container.base.size, 2);
        assert_eq!(container.liquid_padding, 6.0 / 4.0);
        assert_eq!(container.research_cost_multiplier, 4.0);
        assert!(container.base.solid);
        assert_eq!(container.base.health, 400);

        let tank = registry
            .get_liquid_by_name("reinforced-liquid-tank")
            .unwrap();
        assert_eq!(tank.base.size, 3);
        assert!(tank.base.solid);
        assert_eq!(tank.base.liquid_capacity, 2700.0);
        assert_eq!(tank.liquid_padding, 2.0);
        assert_eq!(tank.base.health, 900);
    }

    #[test]
    fn power_nodes_and_batteries_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let node = registry.get_power_by_name("power-node").unwrap();
        assert_eq!(node.kind, PowerBlockKind::PowerNode);
        assert_eq!(node.base.group, BlockGroup::Power);
        assert!(node.base.has_power);
        assert!(!node.base.update);
        assert!(node.base.solid);
        assert!(!node.base.consumes_power);
        assert!(!node.base.outputs_power);
        assert!(node.base.destructible);
        assert_ne!(node.base.env_enabled & Env::SPACE, 0);
        assert_eq!(node.max_nodes, 10);
        assert_eq!(node.laser_range, 6.0);
        assert_eq!(node.schematic_priority, -10);
        assert!(node.under_bullets);
        assert!(node.crush_fragile);
        assert_eq!(
            node.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 2
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 6
                }
            ]
        );

        let large = registry.get_power_by_name("power-node-large").unwrap();
        assert_eq!(large.kind, PowerBlockKind::PowerNode);
        assert_eq!(large.base.size, 2);
        assert_eq!(large.max_nodes, 15);
        assert_eq!(large.laser_range, 15.0);

        let surge = registry.get_power_by_name("surge-tower").unwrap();
        assert_eq!(surge.kind, PowerBlockKind::PowerNode);
        assert_eq!(surge.base.size, 2);
        assert_eq!(surge.max_nodes, 2);
        assert_eq!(surge.laser_range, 40.0);
        assert_eq!(surge.schematic_priority, -15);

        let diode = registry.get_power_by_name("diode").unwrap();
        assert_eq!(diode.kind, PowerBlockKind::PowerDiode);
        assert_eq!(diode.base.group, BlockGroup::Power);
        assert!(diode.rotate);
        assert!(diode.base.update);
        assert!(diode.base.solid);
        assert!(diode.insulated);
        assert!(diode.no_update_disabled);
        assert_eq!(diode.schematic_priority, 10);
        assert_ne!(diode.base.env_enabled & Env::SPACE, 0);

        let battery = registry.get_power_by_name("battery").unwrap();
        assert_eq!(battery.kind, PowerBlockKind::Battery);
        assert!(battery.base.outputs_power);
        assert!(battery.base.consumes_power);
        assert!(!battery.base.update);
        assert!(battery.base.flags.contains(&BlockFlag::Battery));
        assert_eq!(battery.buffered_power, 4000.0);
        assert_eq!(battery.base_explosiveness, 1.0);

        let large_battery = registry.get_power_by_name("battery-large").unwrap();
        assert_eq!(large_battery.kind, PowerBlockKind::Battery);
        assert_eq!(large_battery.base.size, 3);
        assert_eq!(large_battery.buffered_power, 50000.0);
        assert_eq!(large_battery.base_explosiveness, 5.0);
    }

    #[test]
    fn consume_generators_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let combustion = registry.get_power_by_name("combustion-generator").unwrap();
        assert_eq!(combustion.kind, PowerBlockKind::ConsumeGenerator);
        assert_eq!(combustion.base.group, BlockGroup::Power);
        assert!(combustion.base.has_power);
        assert!(combustion.base.outputs_power);
        assert!(!combustion.base.consumes_power);
        assert!(combustion.base.flags.contains(&BlockFlag::Generator));
        assert_eq!(combustion.power_production, 1.0);
        assert_eq!(combustion.item_duration, 120.0);
        assert_eq!(combustion.ambient_sound, "loopSmelter");
        assert_eq!(combustion.ambient_sound_volume, 0.03);
        assert_eq!(combustion.generate_effect, "generatespark");
        assert_eq!(
            combustion.item_duration_multipliers,
            vec![ItemAmount {
                item: item_id("pyratite"),
                amount: 3
            }]
        );

        let steam = registry.get_power_by_name("steam-generator").unwrap();
        assert_eq!(steam.kind, PowerBlockKind::ConsumeGenerator);
        assert_eq!(steam.power_production, 5.5);
        assert_eq!(steam.item_duration, 90.0);
        assert!(steam.base.has_liquids);
        assert_eq!(steam.base.size, 2);
        assert_eq!(steam.ambient_sound, "loopSmelter");
        assert_eq!(
            steam.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("water"),
                amount: 0.1
            }]
        );

        let differential = registry
            .get_power_by_name("differential-generator")
            .unwrap();
        assert_eq!(differential.kind, PowerBlockKind::ConsumeGenerator);
        assert_eq!(differential.power_production, 18.0);
        assert_eq!(differential.item_duration, 220.0);
        assert!(differential.base.has_liquids);
        assert!(differential.base.has_items);
        assert_eq!(differential.base.size, 3);
        assert_eq!(differential.ambient_sound, "loopDifferential");
        assert_eq!(differential.ambient_sound_volume, 0.12);
        assert_eq!(
            differential.consume_items,
            vec![ItemAmount {
                item: item_id("pyratite"),
                amount: 1
            }]
        );
        assert_eq!(
            differential.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("cryofluid"),
                amount: 0.1
            }]
        );

        let rtg = registry.get_power_by_name("rtg-generator").unwrap();
        assert_eq!(rtg.kind, PowerBlockKind::ConsumeGenerator);
        assert_eq!(rtg.base.size, 2);
        assert_eq!(rtg.power_production, 4.5);
        assert_eq!(rtg.item_duration, 60.0 * 14.0);
        assert_eq!(rtg.base.env_enabled, Env::ANY);
        assert_eq!(rtg.generate_effect, "generatespark");
        assert_eq!(
            rtg.item_duration_multipliers,
            vec![ItemAmount {
                item: item_id("phase-fabric"),
                amount: 15
            }]
        );
    }

    #[test]
    fn thermal_and_solar_generators_keep_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        let thermal = registry.get_power_by_name("thermal-generator").unwrap();
        assert_eq!(thermal.kind, PowerBlockKind::ThermalGenerator);
        assert_eq!(thermal.power_production, 1.8);
        assert_eq!(thermal.generate_effect, "redgeneratespark");
        assert_eq!(thermal.effect_chance, 0.011);
        assert_eq!(thermal.base.size, 2);
        assert!(thermal.floating);
        assert_eq!(thermal.ambient_sound, "loopHum");
        assert_eq!(thermal.ambient_sound_volume, 0.06);
        assert!(thermal.no_update_disabled);
        assert_eq!(thermal.attribute, "heat");

        let solar = registry.get_power_by_name("solar-panel").unwrap();
        assert_eq!(solar.kind, PowerBlockKind::SolarGenerator);
        assert_eq!(solar.power_production, 0.12);
        assert_eq!(solar.base.env_enabled, Env::ANY);
        assert!(!solar.base.flags.contains(&BlockFlag::Generator));
        assert_eq!(
            solar.requirements,
            vec![
                ItemAmount {
                    item: item_id("lead"),
                    amount: 10
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 8
                }
            ]
        );

        let large = registry.get_power_by_name("solar-panel-large").unwrap();
        assert_eq!(large.kind, PowerBlockKind::SolarGenerator);
        assert_eq!(large.base.size, 3);
        assert_eq!(large.power_production, 1.6);
        assert_eq!(large.base.env_enabled, Env::ANY);
    }

    #[test]
    fn reactors_and_beam_power_blocks_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let thorium = registry.get_power_by_name("thorium-reactor").unwrap();
        assert_eq!(thorium.kind, PowerBlockKind::NuclearReactor);
        assert_eq!(thorium.ambient_sound, "loopThoriumReactor");
        assert_eq!(thorium.ambient_sound_volume, 0.11);
        assert_eq!(thorium.base.size, 3);
        assert_eq!(thorium.base.health, 700);
        assert_eq!(thorium.item_duration, 360.0);
        assert_eq!(thorium.power_production, 15.0);
        assert_eq!(thorium.heating, 0.02);
        assert_eq!(thorium.coolant_power, 0.5);
        assert!(thorium.base.flags.contains(&BlockFlag::Reactor));
        assert_eq!(
            thorium.consume_items,
            vec![ItemAmount {
                item: item_id("thorium"),
                amount: 1
            }]
        );
        assert_eq!(
            thorium.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("cryofluid"),
                amount: 0.02 / 0.5
            }]
        );

        let impact = registry.get_power_by_name("impact-reactor").unwrap();
        assert_eq!(impact.kind, PowerBlockKind::ImpactReactor);
        assert_eq!(impact.base.size, 4);
        assert_eq!(impact.base.health, 900);
        assert_eq!(impact.power_production, 130.0);
        assert_eq!(impact.item_duration, 140.0);
        assert_eq!(impact.ambient_sound, "loopPulse");
        assert_eq!(impact.ambient_sound_volume, 0.08);
        assert_eq!(impact.base.liquid_capacity, 80.0);
        assert_eq!(impact.consume_power, 25.0);
        assert!(impact.base.consumes_power);
        assert!(impact.base.outputs_power);
        assert_eq!(
            impact.consume_items,
            vec![ItemAmount {
                item: item_id("blast-compound"),
                amount: 1
            }]
        );
        assert_eq!(
            impact.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("cryofluid"),
                amount: 0.25
            }]
        );

        let node = registry.get_power_by_name("beam-node").unwrap();
        assert_eq!(node.kind, PowerBlockKind::BeamNode);
        assert!(node.base.consumes_power);
        assert!(node.base.outputs_power);
        assert_eq!(node.base.health, 90);
        assert_eq!(node.range, 10.0);
        assert_eq!(node.fog_radius, 1.0);
        assert_eq!(node.buffered_power, 1000.0);
        assert!(node.crush_fragile);
        assert_eq!(
            node.research_cost,
            vec![ItemAmount {
                item: item_id("beryllium"),
                amount: 5
            }]
        );

        let tower = registry.get_power_by_name("beam-tower").unwrap();
        assert_eq!(tower.kind, PowerBlockKind::BeamNode);
        assert_eq!(tower.base.size, 3);
        assert!(tower.base.consumes_power);
        assert!(tower.base.outputs_power);
        assert_eq!(tower.range, 23.0);
        assert_eq!(tower.health_scaled, 90.0);
        assert_eq!(tower.fog_radius, 2.0);
        assert_eq!(tower.buffered_power, 40000.0);

        let link = registry.get_power_by_name("beam-link").unwrap();
        assert_eq!(link.kind, PowerBlockKind::LongPowerNode);
        assert_eq!(link.base.size, 3);
        assert_eq!(link.max_nodes, 1);
        assert_eq!(link.laser_range, 500.0);
        assert_eq!(link.power_layer, "Layer.legUnit+2");
        assert!(!link.autolink);
        assert!(link.same_block_connection);
        assert_eq!(link.laser_color2, "ffd9c2");
        assert_eq!(link.laser_scale, 0.8);
        assert_eq!(link.health_scaled, 130.0);

        let turbine = registry.get_power_by_name("turbine-condenser").unwrap();
        assert_eq!(turbine.kind, PowerBlockKind::ThermalGenerator);
        assert_eq!(turbine.attribute, "steam");
        assert_eq!(turbine.base.group, BlockGroup::Liquids);
        assert_eq!(turbine.display_efficiency_scale, 1.0 / 9.0);
        assert_eq!(turbine.min_efficiency, 9.0 - 0.0001);
        assert_eq!(turbine.power_production, 3.0 / 9.0);
        assert!(!turbine.display_efficiency);
        assert_eq!(turbine.generate_effect, "turbinegenerate");
        assert_eq!(turbine.effect_chance, 0.04);
        assert_eq!(turbine.base.size, 3);
        assert!(turbine.base.has_liquids);
        assert_eq!(
            turbine.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("water"),
                amount: 5.0 / 60.0 / 9.0
            })
        );
        assert_eq!(turbine.base.liquid_capacity, 20.0);
        assert_eq!(turbine.fog_radius, 3.0);
    }

    #[test]
    fn erekir_advanced_power_generators_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };

        let chemical = registry
            .get_power_by_name("chemical-combustion-chamber")
            .unwrap();
        assert_eq!(chemical.kind, PowerBlockKind::ConsumeGenerator);
        assert_eq!(chemical.power_production, 550.0 / 60.0);
        assert_eq!(chemical.base.size, 3);
        assert!(chemical.base.has_liquids);
        assert_eq!(chemical.base.liquid_capacity, 100.0);
        assert_eq!(chemical.generate_effect, "none");
        assert_eq!(chemical.ambient_sound, "loopSmelter");
        assert_eq!(
            chemical.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 30
                }
            ]
        );
        assert_eq!(
            chemical.research_cost,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 2000
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 1000
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 10
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 1500
                }
            ]
        );
        assert_eq!(
            chemical.consume_liquids,
            vec![
                LiquidAmount {
                    liquid: liquid_id("ozone"),
                    amount: 2.0 / 60.0
                },
                LiquidAmount {
                    liquid: liquid_id("arkycite"),
                    amount: 40.0 / 60.0
                }
            ]
        );

        let pyrolysis = registry.get_power_by_name("pyrolysis-generator").unwrap();
        assert_eq!(pyrolysis.kind, PowerBlockKind::ConsumeGenerator);
        assert_eq!(pyrolysis.power_production, 1400.0 / 60.0);
        assert_eq!(pyrolysis.base.size, 3);
        assert_eq!(pyrolysis.base.liquid_capacity, 150.0);
        assert_eq!(pyrolysis.research_cost_multiplier, 0.4);
        assert_eq!(
            pyrolysis.consume_liquids,
            vec![
                LiquidAmount {
                    liquid: liquid_id("slag"),
                    amount: 20.0 / 60.0
                },
                LiquidAmount {
                    liquid: liquid_id("arkycite"),
                    amount: 40.0 / 60.0
                }
            ]
        );
        assert_eq!(
            pyrolysis.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("water"),
                amount: 20.0 / 60.0
            })
        );

        let flux = registry.get_power_by_name("flux-reactor").unwrap();
        assert_eq!(flux.kind, PowerBlockKind::VariableReactor);
        assert_eq!(flux.power_production, 18000.0 / 60.0);
        assert_eq!(flux.max_heat, 150.0);
        assert_eq!(flux.unstable_speed, 1.0 / 60.0 / 3.0);
        assert_eq!(flux.warmup_speed, 0.1);
        assert_eq!(flux.generate_effect, "fluxVapor");
        assert_eq!(flux.effect_chance, 0.05);
        assert!(!flux.rebuildable);
        assert_eq!(flux.base.size, 5);
        assert_eq!(flux.base.liquid_capacity, 30.0);
        assert_eq!(
            flux.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("cyanogen"),
                amount: 9.0 / 60.0
            }]
        );
        assert_eq!(flux.explosion_min_warmup, 0.5);
        assert_eq!(flux.explosion_radius, 17);
        assert_eq!(flux.explosion_damage, 2500);
        assert_eq!(flux.explosion_puddles, 70);
        assert_eq!(flux.explosion_puddle_range, TILE_SIZE as f32 * 6.0);
        assert_eq!(flux.explosion_puddle_liquid, Some(liquid_id("slag")));
        assert_eq!(flux.explosion_puddle_amount, 100.0);
        assert_eq!(flux.ambient_sound, "loopFlux");
        assert_eq!(flux.ambient_sound_volume, 0.15);

        let neoplasia = registry.get_power_by_name("neoplasia-reactor").unwrap();
        assert_eq!(neoplasia.kind, PowerBlockKind::HeaterGenerator);
        assert_eq!(neoplasia.base.size, 5);
        assert_eq!(neoplasia.base.liquid_capacity, 80.0);
        assert_eq!(
            neoplasia.output_liquid,
            Some(LiquidAmount {
                liquid: liquid_id("neoplasm"),
                amount: 20.0 / 60.0
            })
        );
        assert!(neoplasia.explode_on_full);
        assert_eq!(neoplasia.heat_output, 60.0);
        assert_eq!(neoplasia.warmup_rate, 0.15);
        assert!(neoplasia.rotate);
        assert!(!neoplasia.rotate_draw);
        assert!(!neoplasia.can_overdrive);
        assert!(neoplasia.draw_arrow);
        assert_eq!(
            neoplasia.consume_liquids,
            vec![
                LiquidAmount {
                    liquid: liquid_id("arkycite"),
                    amount: 80.0 / 60.0
                },
                LiquidAmount {
                    liquid: liquid_id("water"),
                    amount: 10.0 / 60.0
                }
            ]
        );
        assert_eq!(
            neoplasia.consume_items,
            vec![ItemAmount {
                item: item_id("phase-fabric"),
                amount: 1
            }]
        );
        assert_eq!(neoplasia.item_duration, 60.0 * 3.0);
        assert_eq!(neoplasia.item_capacity, 10);
        assert_eq!(neoplasia.base.item_capacity, 10);
        assert_eq!(neoplasia.explosion_radius, 9);
        assert_eq!(neoplasia.explosion_damage, 2000);
        assert_eq!(neoplasia.power_production, 140.0);
        assert_eq!(neoplasia.ambient_sound, "loopBio");
        assert_eq!(neoplasia.ambient_sound_volume, 0.2);
        assert_eq!(neoplasia.explosion_puddles, 80);
        assert_eq!(neoplasia.explosion_puddle_range, TILE_SIZE as f32 * 7.0);
        assert_eq!(
            neoplasia.explosion_puddle_liquid,
            Some(liquid_id("neoplasm"))
        );
        assert_eq!(neoplasia.explosion_puddle_amount, 200.0);
        assert_eq!(neoplasia.explosion_min_warmup, 0.5);
    }

    #[test]
    fn sand_floors_use_sand_item_drop_when_available() {
        let (all_items, _, registry) = load_test_registry();
        let sand = find_item(&all_items, "sand").unwrap();
        let sand_id = sand.base.mappable.base.id;

        assert_eq!(
            registry
                .get_floor_by_name("sand-floor")
                .unwrap()
                .base
                .item_drop,
            Some(sand_id)
        );
        assert_eq!(
            registry
                .get_floor_by_name("darksand")
                .unwrap()
                .base
                .item_drop,
            Some(sand_id)
        );
    }

    #[test]
    fn missing_wall_names_fall_back_to_air() {
        let mut registry = BlockRegistry::new();
        registry.register_plain("air");
        registry.register_floor("lonely-floor", |_| {});
        registry.finalize_floor_links();

        assert_eq!(registry.get_floor_by_name("lonely-floor").unwrap().wall, 0);
    }

    #[test]
    fn liquid_and_special_floors_match_upstream_field_subset() {
        let (_, all_liquids, registry) = load_test_registry();
        let water = liquid_id(&all_liquids, "water").unwrap();
        let oil = liquid_id(&all_liquids, "oil").unwrap();
        let slag = liquid_id(&all_liquids, "slag").unwrap();
        let cryofluid = liquid_id(&all_liquids, "cryofluid").unwrap();

        let deep = registry.get_floor_by_name("deep-water").unwrap();
        assert_eq!(deep.speed_multiplier, 0.2);
        assert_eq!(deep.base.variants, 0);
        assert_eq!(deep.liquid_drop, Some(water));
        assert_eq!(deep.liquid_multiplier, 1.5);
        assert!(deep.is_liquid);
        assert_eq!(deep.status, "wet");
        assert_eq!(deep.status_duration, 120.0);
        assert_eq!(deep.drown_time, 200.0);
        assert_eq!(deep.base.cache_layer, CacheLayer::Water);
        assert!(deep.supports_overlay);

        let tar = registry.get_floor_by_name("tar").unwrap();
        assert_eq!(tar.status, "tarred");
        assert_eq!(tar.liquid_drop, Some(oil));
        assert_eq!(tar.base.cache_layer, CacheLayer::Tar);
        assert!(tar.base.obstructs_light);

        let molten = registry.get_floor_by_name("molten-slag").unwrap();
        assert_eq!(molten.status, "melting");
        assert_eq!(molten.liquid_drop, Some(slag));
        assert_eq!(molten.base.cache_layer, CacheLayer::Slag);
        assert!(molten.base.emit_light);
        assert_eq!(molten.base.light_radius, 40.0);
        assert!(molten.force_draw_light);

        let pooled = registry.get_floor_by_name("pooled-cryofluid").unwrap();
        assert_eq!(pooled.status, "freezing");
        assert_eq!(pooled.liquid_drop, Some(cryofluid));
        assert_eq!(pooled.liquid_multiplier, 0.5);
        assert_eq!(pooled.overlay_alpha, 0.35);
        assert_eq!(pooled.base.cache_layer, CacheLayer::Cryofluid);
        assert!(pooled.base.emit_light);
    }

    #[test]
    fn space_empty_and_mud_keep_special_floor_flags() {
        let (_, _, registry) = load_test_registry();

        let space = registry.get_floor_by_name("space").unwrap();
        assert_eq!(space.base.cache_layer, CacheLayer::Space);
        assert!(!space.base.placeable_on);
        assert!(space.base.solid);
        assert_eq!(space.base.variants, 0);
        assert!(!space.can_shadow);
        assert!(!space.draw_edge_out);

        let empty = registry.get_floor_by_name("empty").unwrap();
        assert!(!empty.base.placeable_on);
        assert!(empty.base.solid);
        assert_eq!(empty.base.variants, 0);
        assert!(!empty.can_shadow);
        assert!(!empty.draw_edge_out);

        let mud = registry.get_floor_by_name("mud").unwrap();
        assert_eq!(mud.speed_multiplier, 0.6);
        assert_eq!(mud.status, "muddy");
        assert_eq!(mud.status_duration, 30.0);
        assert_eq!(mud.base.cache_layer, CacheLayer::Mud);
        assert_eq!(mud.walk_sound, "stepMud");
        assert_eq!(mud.walk_sound_volume, 0.08);
    }
}
