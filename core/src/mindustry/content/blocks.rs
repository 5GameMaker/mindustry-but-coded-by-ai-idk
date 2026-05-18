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
    SolidPump,
    Fracker,
    WallCrafter,
    BeamDrill,
    BurstDrill,
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
    pub consume_items: Vec<ItemAmount>,
    pub consume_item_boosts: Vec<ItemAmount>,
    pub consume_liquids: Vec<LiquidConsume>,
    pub output_item: Option<ContentId>,
    pub outputs_liquid: bool,
    pub result_liquid: Option<ContentId>,
    pub pump_amount: f32,
    pub consume_time: f32,
    pub floating: bool,
    pub attribute: String,
    pub base_efficiency: f32,
    pub item_use_time: f32,
    pub boost_item_use_time: f32,
    pub item_boost_intensity: f32,
    pub has_liquid_booster: bool,
    pub rotate: bool,
    pub draw_arrow: bool,
    pub ignore_line_rotation: bool,
    pub region_rotated1: i32,
    pub fog_radius: f32,
    pub range: i32,
    pub laser_width: f32,
    pub optional_boost_intensity: f32,
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
    pub shake: f32,
    pub speed_curve: String,
    pub inverted_time: f32,
    pub arrow_spacing: f32,
    pub arrow_offset: f32,
    pub arrows: i32,
    pub arrow_color: String,
    pub base_arrow_color: String,
    pub glow_color_alpha: f32,
    pub drill_sound: String,
    pub drill_sound_volume: f32,
    pub drill_sound_pitch_rand: f32,
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
            consume_items: Vec::new(),
            consume_item_boosts: Vec::new(),
            consume_liquids: Vec::new(),
            output_item: None,
            outputs_liquid: false,
            result_liquid: None,
            pump_amount: 0.0,
            consume_time: 0.0,
            floating: false,
            attribute: String::new(),
            base_efficiency: 0.0,
            item_use_time: 0.0,
            boost_item_use_time: 0.0,
            item_boost_intensity: 0.0,
            has_liquid_booster: false,
            rotate: false,
            draw_arrow: true,
            ignore_line_rotation: false,
            region_rotated1: 0,
            fog_radius: 0.0,
            range: 0,
            laser_width: 0.0,
            optional_boost_intensity: 0.0,
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
            shake: 0.0,
            speed_curve: String::new(),
            inverted_time: 0.0,
            arrow_spacing: 0.0,
            arrow_offset: 0.0,
            arrows: 0,
            arrow_color: String::new(),
            base_arrow_color: String::new(),
            glow_color_alpha: 1.0,
            drill_sound: "none".into(),
            drill_sound_volume: 0.0,
            drill_sound_pitch_rand: 0.0,
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
            ProductionBlockKind::SolidPump => {
                self.apply_solid_pump_defaults();
            }
            ProductionBlockKind::Fracker => {
                self.apply_solid_pump_defaults();
                self.base.has_items = true;
                self.base.env_required |= Env::GROUND_OIL;
                self.ambient_sound = "loopDrill".into();
                self.ambient_sound_volume = 0.03;
                self.item_use_time = 100.0;
            }
            ProductionBlockKind::WallCrafter => {
                self.base.has_items = true;
                self.rotate = true;
                self.base.update = true;
                self.base.solid = true;
                self.ignore_line_rotation = true;
                self.region_rotated1 = 1;
                self.base.env_enabled |= Env::SPACE;
                self.base.flags.push(BlockFlag::Drill);
                self.drill_time = 150.0;
                self.liquid_boost_intensity = 1.6;
                self.update_effect = "mineWallSmall".into();
                self.update_effect_chance = 0.02;
                self.rotate_speed = 2.0;
                self.attribute = "sand".into();
                self.boost_item_use_time = 120.0;
                self.item_boost_intensity = 1.6;
            }
            ProductionBlockKind::BeamDrill => {
                self.base.has_items = true;
                self.rotate = true;
                self.base.update = true;
                self.base.solid = true;
                self.draw_arrow = false;
                self.region_rotated1 = 1;
                self.ignore_line_rotation = true;
                self.ambient_sound_volume = 0.05;
                self.ambient_sound = "loopMineBeam".into();
                self.base.env_enabled |= Env::SPACE;
                self.base.flags.push(BlockFlag::Drill);
                self.drill_time = 200.0;
                self.range = 5;
                self.tier = 1;
                self.laser_width = 0.65;
                self.optional_boost_intensity = 2.5;
            }
            ProductionBlockKind::BurstDrill => {
                self.base.update = true;
                self.base.solid = true;
                self.base.group = BlockGroup::Drills;
                self.base.has_liquids = true;
                self.base.has_items = true;
                self.ambient_sound = "drillCharge".into();
                self.ambient_sound_volume = 0.18;
                self.base.env_enabled |= Env::SPACE;
                self.base.flags.push(BlockFlag::Drill);
                self.hardness_drill_multiplier = 0.0;
                self.drill_effect_rnd = 0.0;
                self.drill_effect = "shockwave".into();
                self.shake = 2.0;
                self.speed_curve = "pow2In".into();
                self.inverted_time = 200.0;
                self.arrow_spacing = 4.0;
                self.arrow_offset = 0.0;
                self.arrows = 3;
                self.arrow_color = "feb380".into();
                self.base_arrow_color = "6e7080".into();
                self.glow_color_alpha = 1.0;
                self.drill_sound = "drillImpact".into();
                self.drill_sound_volume = 0.6;
                self.drill_sound_pitch_rand = 0.1;
            }
        }
    }

    fn apply_solid_pump_defaults(&mut self) {
        self.base.update = true;
        self.base.solid = true;
        self.base.has_liquids = true;
        self.base.has_power = true;
        self.base.group = BlockGroup::Liquids;
        self.base.env_enabled = Env::TERRESTRIAL;
        self.outputs_liquid = true;
        self.result_liquid = None;
        self.pump_amount = 0.2;
        self.consume_time = 60.0 * 5.0;
        self.warmup_speed = 0.019;
        self.update_effect = "none".into();
        self.update_effect_chance = 0.02;
        self.rotate_speed = 1.0;
        self.base_efficiency = 1.0;
        self.floating = true;
    }

    fn finalize(&mut self) {
        match self.kind {
            ProductionBlockKind::Drill => {
                if self.drill_effect_rnd < 0.0 {
                    self.drill_effect_rnd = self.base.size as f32;
                }
            }
            ProductionBlockKind::SolidPump
            | ProductionBlockKind::Fracker
            | ProductionBlockKind::WallCrafter
            | ProductionBlockKind::BeamDrill
            | ProductionBlockKind::BurstDrill => {
                self.has_liquid_booster =
                    self.consume_liquids.iter().any(|consume| consume.booster);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBlockKind {
    Storage,
    Core,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageBlockData {
    pub base: Block,
    pub kind: StorageBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub research_cost_multipliers: Vec<ItemMultiplier>,
    pub build_cost_multiplier: f32,
    pub scaled_health: f32,
    pub armor: f32,
    pub core_merge: bool,
    pub separate_item_capacity: bool,
    pub allow_resupply: bool,
    pub outputs_items: bool,
    pub always_unlocked: bool,
    pub always_allow_deposit: bool,
    pub draw_disabled: bool,
    pub can_overdrive: bool,
    pub commandable: bool,
    pub unit_cap_modifier: i32,
    pub unit_type: String,
    pub thruster_length: f32,
    pub thruster_offset: f32,
    pub is_first_tier: bool,
    pub allow_spawn: bool,
    pub requires_core_zone: bool,
    pub incinerate_non_buildable: bool,
    pub land_duration: f32,
    pub land_music: String,
    pub launch_sound: String,
    pub land_sound: String,
    pub launch_sound_volume: f32,
    pub land_sound_volume: f32,
    pub launch_effect: String,
    pub land_zoom_interp: String,
    pub land_zoom_from: f32,
    pub land_zoom_to: f32,
    pub capture_invincibility: f32,
    pub destroy_sound: String,
    pub destroy_sound_volume: f32,
}

impl StorageBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: StorageBlockKind) -> Self {
        let base = Block::new(id, name);
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost_multiplier: 1.0,
            research_cost_multipliers: Vec::new(),
            build_cost_multiplier: 1.0,
            scaled_health: -1.0,
            armor: 0.0,
            core_merge: true,
            separate_item_capacity: false,
            allow_resupply: false,
            outputs_items: true,
            always_unlocked: false,
            always_allow_deposit: false,
            draw_disabled: true,
            can_overdrive: true,
            commandable: false,
            unit_cap_modifier: 0,
            unit_type: String::new(),
            thruster_length: 0.0,
            thruster_offset: 0.0,
            is_first_tier: false,
            allow_spawn: false,
            requires_core_zone: false,
            incinerate_non_buildable: false,
            land_duration: 0.0,
            land_music: String::new(),
            launch_sound: String::new(),
            land_sound: String::new(),
            launch_sound_volume: 0.0,
            land_sound_volume: 0.0,
            launch_effect: String::new(),
            land_zoom_interp: String::new(),
            land_zoom_from: 0.0,
            land_zoom_to: 0.0,
            capture_invincibility: 0.0,
            destroy_sound: String::new(),
            destroy_sound_volume: 0.0,
        };
        block.apply_storage_defaults();
        if matches!(block.kind, StorageBlockKind::Core) {
            block.apply_core_defaults();
        }
        block
    }

    fn apply_storage_defaults(&mut self) {
        self.base.has_items = true;
        self.base.solid = true;
        self.base.update = false;
        self.base.sync = true;
        self.base.destructible = true;
        self.separate_item_capacity = true;
        self.base.group = BlockGroup::Transportation;
        self.base.flags.push(BlockFlag::Storage);
        self.allow_resupply = true;
        self.base.env_enabled = Env::ANY;
        self.outputs_items = false;
    }

    fn apply_core_defaults(&mut self) {
        self.base.solid = true;
        self.base.update = true;
        self.base.has_items = true;
        self.always_allow_deposit = true;
        self.base.priority = 2;
        self.base.flags.clear();
        self.base.flags.push(BlockFlag::Core);
        self.unit_cap_modifier = 10;
        self.base.sync = false;
        self.draw_disabled = false;
        self.can_overdrive = false;
        self.commandable = true;
        self.base.env_enabled |= Env::SPACE;
        self.base.replaceable = false;
        self.destroy_sound = "explosionCore".into();
        self.destroy_sound_volume = 1.6;
        self.thruster_length = 14.0 / 4.0;
        self.thruster_offset = 0.0;
        self.allow_spawn = true;
        self.unit_type = "alpha".into();
        self.land_duration = 160.0;
        self.land_music = "land".into();
        self.launch_sound = "coreLaunch".into();
        self.land_sound = "coreLand".into();
        self.launch_sound_volume = 1.0;
        self.land_sound_volume = 1.0;
        self.launch_effect = "launch".into();
        self.land_zoom_interp = "pow3".into();
        self.land_zoom_from = 0.02;
        self.land_zoom_to = 4.0;
        self.capture_invincibility = 60.0 * 15.0;
    }

    fn finalize(&mut self) {
        if self.scaled_health >= 0.0 {
            self.base.health =
                ((self.base.size * self.base.size) as f32 * self.scaled_health) as i32;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurretBlockKind {
    ItemTurret,
    LiquidTurret,
    PowerTurret,
    LaserTurret,
    ContinuousTurret,
    ContinuousLiquidTurret,
    TractorBeamTurret,
    PointDefenseTurret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulletKind {
    Generic,
    Basic,
    Missile,
    Flak,
    Artillery,
    Shrapnel,
    Liquid,
    Laser,
    Lightning,
    Rail,
    ContinuousLaser,
    ContinuousFlame,
    PointLaser,
    Explosion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitPartKind {
    Shape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPartSpec {
    pub kind: UnitPartKind,
    pub progress: String,
    pub color: String,
    pub sides: i32,
    pub radius: f32,
    pub rotate_speed: f32,
    pub hollow: bool,
    pub layer: String,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityKind {
    MoveEffect,
    ForceField,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbilitySpec {
    pub kind: AbilityKind,
    pub effect: String,
    pub rotation: f32,
    pub y: f32,
    pub color: String,
    pub interval: f32,
    pub radius: f32,
    pub regen: f32,
    pub max: f32,
    pub cooldown: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponSpec {
    pub shoot_cone: f32,
    pub mirror: bool,
    pub rotate: bool,
    pub rotation_limit: f32,
    pub rotate_speed: f32,
    pub reload: f32,
    pub death_explosion_effect: String,
    pub shoot_on_death: bool,
    pub shake: f32,
    pub bullet: Option<Box<BulletSpec>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MissileUnitSpec {
    pub name: String,
    pub speed: f32,
    pub max_range: f32,
    pub lifetime: f32,
    pub hit_size: f32,
    pub outline_color: String,
    pub engine_color: String,
    pub trail_color: String,
    pub engine_layer: String,
    pub engine_size: f32,
    pub engine_offset: f32,
    pub rotate_speed: f32,
    pub trail_length: i32,
    pub missile_accel_time: f32,
    pub low_altitude: bool,
    pub loop_sound: String,
    pub loop_sound_volume: f32,
    pub death_sound: String,
    pub target_air: bool,
    pub target_under_blocks: bool,
    pub fog_radius: f32,
    pub health: f32,
    pub parts: Vec<UnitPartSpec>,
    pub weapons: Vec<WeaponSpec>,
    pub abilities: Vec<AbilitySpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BulletSpec {
    pub kind: BulletKind,
    pub speed: f32,
    pub damage: f32,
    pub hit_size: f32,
    pub draw_size: f32,
    pub width: f32,
    pub height: f32,
    pub lifetime: f32,
    pub drag: f32,
    pub layer: String,
    pub sprite: String,
    pub back_sprite: String,
    pub color: String,
    pub ammo_multiplier: f32,
    pub reload_multiplier: f32,
    pub range_change: f32,
    pub extra_range_margin: f32,
    pub hit_effect: String,
    pub despawn_effect: String,
    pub shoot_effect: String,
    pub smoke_effect: String,
    pub charge_effect: String,
    pub bullet_shoot_sound: String,
    pub hit_sound: String,
    pub despawn_sound: String,
    pub hit_color: String,
    pub back_color: String,
    pub trail_color: String,
    pub front_color: String,
    pub from_color: String,
    pub to_color: String,
    pub light_color: String,
    pub colors: Vec<String>,
    pub homing_power: f32,
    pub homing_delay: f32,
    pub homing_range: f32,
    pub velocity_rnd: f32,
    pub point_effect: String,
    pub pierce_effect: String,
    pub line_effect: String,
    pub end_effect: String,
    pub beam_effect: String,
    pub beam_effect_interval: f32,
    pub beam_effect_size: f32,
    pub point_effect_space: f32,
    pub hit_shake: f32,
    pub shake: f32,
    pub damage_interval: f32,
    pub osc_scl: f32,
    pub osc_mag: f32,
    pub continuous: bool,
    pub timescale_damage: bool,
    pub impact: bool,
    pub optimal_life_fract: f32,
    pub laser_absorb: bool,
    pub pierce_armor: bool,
    pub flare_color: String,
    pub incend_chance: f32,
    pub incend_spread: f32,
    pub incend_amount: i32,
    pub trail_length: i32,
    pub trail_width: f32,
    pub trail_sin_scl: f32,
    pub trail_sin_mag: f32,
    pub trail_interp: String,
    pub splash_damage: f32,
    pub splash_damage_radius: f32,
    pub scaled_splash_damage: bool,
    pub range_override: f32,
    pub explode_range: f32,
    pub explode_delay: f32,
    pub flak_delay: f32,
    pub flak_interval: f32,
    pub frag_bullets: i32,
    pub frag_bullet: Option<Box<BulletSpec>>,
    pub frag_on_hit: bool,
    pub frag_life_min: f32,
    pub frag_random_spread: f32,
    pub frag_spread: f32,
    pub frag_velocity_min: f32,
    pub frag_velocity_max: f32,
    pub bullet_interval: f32,
    pub interval_random_spread: f32,
    pub interval_bullets: i32,
    pub interval_angle: f32,
    pub interval_spread: f32,
    pub interval_bullet: Option<Box<BulletSpec>>,
    pub collides_ground: bool,
    pub collides_air: bool,
    pub collides_tiles: bool,
    pub collides: bool,
    pub shrink_x: f32,
    pub shrink_y: f32,
    pub scale_life: bool,
    pub life_scale_rand_min: f32,
    pub life_scale_rand_max: f32,
    pub knockback: f32,
    pub pierce: bool,
    pub pierce_building: bool,
    pub pierce_damage_factor: f32,
    pub reflectable: bool,
    pub remove_after_pierce: bool,
    pub delay_frags: bool,
    pub pierce_cap: i32,
    pub hittable: bool,
    pub keep_velocity: bool,
    pub absorbable: bool,
    pub instant_disappear: bool,
    pub kill_shooter: bool,
    pub status: String,
    pub status_duration: f32,
    pub make_fire: bool,
    pub trail_effect: String,
    pub trail_interval: f32,
    pub trail_rotation: bool,
    pub trail_param: f32,
    pub trail_chance: f32,
    pub rotation_offset: f32,
    pub despawn_shake: f32,
    pub display_ammo_multiplier: bool,
    pub building_damage_multiplier: f32,
    pub shield_damage_multiplier: f32,
    pub armor_multiplier: f32,
    pub length: f32,
    pub side_angle: f32,
    pub side_width: f32,
    pub side_length: f32,
    pub hit_large: bool,
    pub serrations: i32,
    pub serration_len_scl: f32,
    pub serration_width: f32,
    pub serration_spacing: f32,
    pub serration_space_offset: f32,
    pub serration_fade_offset: f32,
    pub lightning_length: i32,
    pub lightning_length_rand: i32,
    pub lightning_cone: f32,
    pub lightning_color: String,
    pub lightning: i32,
    pub lightning_damage: f32,
    pub lightning_type: Option<Box<BulletSpec>>,
    pub puddle_size: f32,
    pub orb_size: f32,
    pub boil_time: f32,
    pub light_radius: f32,
    pub light_opacity: f32,
    pub spawn_unit: Option<Box<MissileUnitSpec>>,
}

impl BulletSpec {
    pub fn new(kind: BulletKind, speed: f32, damage: f32) -> Self {
        Self {
            kind,
            speed,
            damage,
            hit_size: 4.0,
            draw_size: 40.0,
            width: 0.0,
            height: 0.0,
            lifetime: 0.0,
            drag: 0.0,
            layer: String::new(),
            sprite: String::new(),
            back_sprite: String::new(),
            color: "white".into(),
            ammo_multiplier: 1.0,
            reload_multiplier: 1.0,
            range_change: 0.0,
            extra_range_margin: 0.0,
            hit_effect: "none".into(),
            despawn_effect: "none".into(),
            shoot_effect: "none".into(),
            smoke_effect: "shootSmallSmoke".into(),
            charge_effect: "none".into(),
            bullet_shoot_sound: "none".into(),
            hit_sound: "none".into(),
            despawn_sound: "none".into(),
            hit_color: String::new(),
            back_color: String::new(),
            trail_color: String::new(),
            front_color: String::new(),
            from_color: "white".into(),
            to_color: String::new(),
            light_color: "powerLight".into(),
            colors: Vec::new(),
            homing_power: 0.0,
            homing_delay: 0.0,
            homing_range: 0.0,
            velocity_rnd: 0.0,
            point_effect: "none".into(),
            pierce_effect: "hitBulletSmall".into(),
            line_effect: "none".into(),
            end_effect: "none".into(),
            beam_effect: "none".into(),
            beam_effect_interval: 0.0,
            beam_effect_size: 0.0,
            point_effect_space: 20.0,
            hit_shake: 0.0,
            shake: 0.0,
            damage_interval: 5.0,
            osc_scl: 0.0,
            osc_mag: 0.0,
            continuous: false,
            timescale_damage: false,
            impact: false,
            optimal_life_fract: 1.0,
            laser_absorb: true,
            pierce_armor: false,
            flare_color: "e189f5".into(),
            incend_chance: 0.0,
            incend_spread: 0.0,
            incend_amount: 0,
            trail_length: 0,
            trail_width: 0.0,
            trail_sin_scl: 3.0,
            trail_sin_mag: 0.0,
            trail_interp: String::new(),
            splash_damage: 0.0,
            splash_damage_radius: 0.0,
            scaled_splash_damage: false,
            range_override: 0.0,
            explode_range: 0.0,
            explode_delay: 0.0,
            flak_delay: 0.0,
            flak_interval: 0.0,
            frag_bullets: 0,
            frag_bullet: None,
            frag_on_hit: true,
            frag_life_min: 0.0,
            frag_random_spread: 360.0,
            frag_spread: 0.0,
            frag_velocity_min: 0.2,
            frag_velocity_max: 1.0,
            bullet_interval: 0.0,
            interval_random_spread: 360.0,
            interval_bullets: 1,
            interval_angle: 0.0,
            interval_spread: 0.0,
            interval_bullet: None,
            collides_ground: true,
            collides_air: true,
            collides_tiles: true,
            collides: true,
            shrink_x: 0.0,
            shrink_y: 0.0,
            scale_life: false,
            life_scale_rand_min: 1.0,
            life_scale_rand_max: 1.0,
            knockback: 0.0,
            pierce: false,
            pierce_building: false,
            pierce_damage_factor: 0.0,
            reflectable: true,
            remove_after_pierce: true,
            delay_frags: false,
            pierce_cap: -1,
            hittable: true,
            keep_velocity: true,
            absorbable: true,
            instant_disappear: false,
            kill_shooter: false,
            status: "none".into(),
            status_duration: 60.0 * 8.0,
            make_fire: false,
            trail_effect: "missileTrail".into(),
            trail_interval: 0.0,
            trail_rotation: false,
            trail_param: 2.0,
            trail_chance: 0.0,
            rotation_offset: 0.0,
            despawn_shake: 0.0,
            display_ammo_multiplier: true,
            building_damage_multiplier: 1.0,
            shield_damage_multiplier: 1.0,
            armor_multiplier: 1.0,
            length: 0.0,
            side_angle: 0.0,
            side_width: 0.0,
            side_length: 0.0,
            hit_large: false,
            serrations: 7,
            serration_len_scl: 10.0,
            serration_width: 4.0,
            serration_spacing: 8.0,
            serration_space_offset: 80.0,
            serration_fade_offset: 0.5,
            lightning_length: 5,
            lightning_length_rand: 0,
            lightning_cone: 360.0,
            lightning_color: String::new(),
            lightning: 0,
            lightning_damage: -1.0,
            lightning_type: None,
            puddle_size: 0.0,
            orb_size: 0.0,
            boil_time: 0.0,
            light_radius: 0.0,
            light_opacity: 0.0,
            spawn_unit: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurretAmmo {
    pub item: ContentId,
    pub bullet: BulletSpec,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidTurretAmmo {
    pub liquid: ContentId,
    pub bullet: BulletSpec,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShootBarrel {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurretBlockData {
    pub base: Block,
    pub kind: TurretBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub ammo: Vec<TurretAmmo>,
    pub liquid_ammo: Vec<LiquidTurretAmmo>,
    pub consume_liquids: Vec<LiquidAmount>,
    pub shoot_type: Option<Box<BulletSpec>>,
    pub research_cost_multiplier: f32,
    pub build_cost_multiplier: f32,
    pub consume_power: f32,
    pub liquid_consumed: f32,
    pub coolant_amount: f32,
    pub coolant_multiplier: f32,
    pub consume_coolant: bool,
    pub cool_effect: String,
    pub deposit_cooldown: f32,
    pub scale_damage_efficiency: bool,
    pub range: f32,
    pub fog_radius: f32,
    pub place_overlap_margin: f32,
    pub place_overlap_range: f32,
    pub rotate_speed: f32,
    pub aim_change_speed: f32,
    pub fog_radius_multiplier: f32,
    pub disable_overlap_check: bool,
    pub activation_time: f32,
    pub reload: f32,
    pub target_interval: f32,
    pub new_target_interval: f32,
    pub max_ammo: i32,
    pub ammo_per_shot: i32,
    pub consume_ammo_once: bool,
    pub heat_requirement: f32,
    pub max_heat_efficiency: f32,
    pub inaccuracy: f32,
    pub velocity_rnd: f32,
    pub scale_lifetime_offset: f32,
    pub shoot_cone: f32,
    pub shoot_x: f32,
    pub shoot_y: f32,
    pub x_rand: f32,
    pub draw_min_range: bool,
    pub tracking_range: f32,
    pub min_range: f32,
    pub min_warmup: f32,
    pub accurate_delay: bool,
    pub move_while_charging: bool,
    pub reload_while_charging: bool,
    pub warmup_maintain_time: f32,
    pub firing_move_fract: f32,
    pub shoot_duration: f32,
    pub shoot_pattern: String,
    pub shoot_shots: i32,
    pub shoot_first_shot_delay: f32,
    pub shoot_shot_delay: f32,
    pub shoot_barrels: Vec<ShootBarrel>,
    pub shoot_spread: f32,
    pub shoot_alternate_spread: f32,
    pub shoot_alternate_barrels: i32,
    pub shoot_helix_scl: f32,
    pub shoot_helix_mag: f32,
    pub shoot_summon_x: f32,
    pub shoot_summon_y: f32,
    pub shoot_summon_radius: f32,
    pub shoot_summon_spread: f32,
    pub target_air: bool,
    pub target_ground: bool,
    pub target_blocks: bool,
    pub target_healing: bool,
    pub player_controllable: bool,
    pub display_ammo_multiplier: bool,
    pub target_under_blocks: bool,
    pub always_shooting: bool,
    pub predict_target: bool,
    pub unit_sort: String,
    pub heat_color: String,
    pub shoot_effect: String,
    pub smoke_effect: String,
    pub ammo_use_effect: String,
    pub shoot_sound: String,
    pub shoot_sound_volume: f32,
    pub charge_sound: String,
    pub loop_sound: String,
    pub loop_sound_volume: f32,
    pub sound_pitch_min: f32,
    pub sound_pitch_max: f32,
    pub ammo_eject_back: f32,
    pub shoot_warmup_speed: f32,
    pub linear_warmup: bool,
    pub recoil: f32,
    pub recoils: i32,
    pub recoil_time: f32,
    pub recoil_pow: f32,
    pub cooldown_time: f32,
    pub elevation: f32,
    pub shake: f32,
    pub extinguish: bool,
    pub retarget_time: f32,
    pub shoot_length: f32,
    pub laser_width: f32,
    pub force: f32,
    pub scaled_force: f32,
    pub damage: f32,
    pub laser_color: String,
    pub color: String,
    pub beam_effect: String,
    pub point_hit_effect: String,
    pub bullet_damage: f32,
    pub status: String,
    pub status_duration: f32,
    pub drawer: String,
    pub outline_color: String,
    pub build_time: f32,
    pub liquid_capacity: f32,
    pub outlined_icon: bool,
    pub draw_liquid_light: bool,
    pub rotate: bool,
    pub quick_rotate: bool,
    pub draw_arrow: bool,
    pub ignore_line_rotation: bool,
    pub rotate_draw_editor: bool,
    pub visual_rotation_offset: f32,
    pub region_rotated1: i32,
    pub region_rotated2: i32,
    pub scaled_health: f32,
}

impl TurretBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: TurretBlockKind) -> Self {
        let base = Block::new(id, name);
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            ammo: Vec::new(),
            liquid_ammo: Vec::new(),
            consume_liquids: Vec::new(),
            shoot_type: None,
            research_cost_multiplier: 1.0,
            build_cost_multiplier: 1.0,
            consume_power: 0.0,
            liquid_consumed: 1.0 / 60.0,
            coolant_amount: 0.0,
            coolant_multiplier: 5.0,
            consume_coolant: false,
            cool_effect: "fuelburn".into(),
            deposit_cooldown: -1.0,
            scale_damage_efficiency: false,
            range: 80.0,
            fog_radius: 0.0,
            place_overlap_margin: 8.0 * 7.0,
            place_overlap_range: 0.0,
            rotate_speed: 5.0,
            aim_change_speed: f32::INFINITY,
            fog_radius_multiplier: 1.0,
            disable_overlap_check: false,
            activation_time: 0.0,
            reload: 10.0,
            target_interval: 20.0,
            new_target_interval: -1.0,
            max_ammo: 30,
            ammo_per_shot: 1,
            consume_ammo_once: true,
            heat_requirement: -1.0,
            max_heat_efficiency: 3.0,
            inaccuracy: 0.0,
            velocity_rnd: 0.0,
            scale_lifetime_offset: 0.0,
            shoot_cone: 8.0,
            shoot_x: 0.0,
            shoot_y: f32::NEG_INFINITY,
            x_rand: 0.0,
            draw_min_range: false,
            tracking_range: 0.0,
            min_range: 0.0,
            min_warmup: 0.0,
            accurate_delay: true,
            move_while_charging: true,
            reload_while_charging: true,
            warmup_maintain_time: 0.0,
            firing_move_fract: 0.25,
            shoot_duration: 100.0,
            shoot_pattern: "ShootPattern".into(),
            shoot_shots: 1,
            shoot_first_shot_delay: 0.0,
            shoot_shot_delay: 0.0,
            shoot_barrels: Vec::new(),
            shoot_spread: 5.0,
            shoot_alternate_spread: 0.0,
            shoot_alternate_barrels: 1,
            shoot_helix_scl: 0.0,
            shoot_helix_mag: 0.0,
            shoot_summon_x: 0.0,
            shoot_summon_y: 0.0,
            shoot_summon_radius: 0.0,
            shoot_summon_spread: 0.0,
            target_air: true,
            target_ground: true,
            target_blocks: true,
            target_healing: false,
            player_controllable: true,
            display_ammo_multiplier: true,
            target_under_blocks: true,
            always_shooting: false,
            predict_target: true,
            unit_sort: "closest".into(),
            heat_color: "turretHeat".into(),
            shoot_effect: String::new(),
            smoke_effect: String::new(),
            ammo_use_effect: "none".into(),
            shoot_sound: "shootDuo".into(),
            shoot_sound_volume: 1.0,
            charge_sound: "none".into(),
            loop_sound: "none".into(),
            loop_sound_volume: 0.5,
            sound_pitch_min: 0.9,
            sound_pitch_max: 1.1,
            ammo_eject_back: 1.0,
            shoot_warmup_speed: 0.1,
            linear_warmup: false,
            recoil: 1.0,
            recoils: -1,
            recoil_time: -1.0,
            recoil_pow: 1.8,
            cooldown_time: 20.0,
            elevation: -1.0,
            shake: 0.0,
            extinguish: false,
            retarget_time: 5.0,
            shoot_length: 5.0,
            laser_width: 0.6,
            force: 0.3,
            scaled_force: 0.0,
            damage: 0.0,
            laser_color: "white".into(),
            color: "white".into(),
            beam_effect: "none".into(),
            point_hit_effect: "none".into(),
            bullet_damage: 0.0,
            status: "none".into(),
            status_duration: 300.0,
            drawer: "DrawTurret".into(),
            outline_color: "darkOutline".into(),
            build_time: -1.0,
            liquid_capacity: 20.0,
            outlined_icon: true,
            draw_liquid_light: false,
            rotate: true,
            quick_rotate: false,
            draw_arrow: false,
            ignore_line_rotation: true,
            rotate_draw_editor: false,
            visual_rotation_offset: -90.0,
            region_rotated1: 1,
            region_rotated2: 2,
            scaled_health: -1.0,
        };
        block.apply_kind_defaults();
        block
    }

    fn apply_kind_defaults(&mut self) {
        self.base.update = true;
        self.base.solid = true;
        self.base.priority = 1;
        self.base.group = BlockGroup::Turrets;
        self.base.flags.push(BlockFlag::Turret);
        self.base.liquid_capacity = self.liquid_capacity;
        self.base.sync = true;

        match self.kind {
            TurretBlockKind::ItemTurret => {
                self.base.has_items = true;
            }
            TurretBlockKind::LiquidTurret => {
                self.base.has_liquids = true;
                self.loop_sound = "loopSpray".into();
                self.shoot_sound = "none".into();
                self.smoke_effect = "none".into();
                self.shoot_effect = "none".into();
                self.extinguish = true;
            }
            TurretBlockKind::PowerTurret => {
                self.base.has_power = true;
            }
            TurretBlockKind::LaserTurret => {
                self.base.has_power = true;
                self.coolant_multiplier = 1.0;
            }
            TurretBlockKind::ContinuousTurret => {
                self.coolant_multiplier = 1.0;
                self.display_ammo_multiplier = false;
                self.base.env_enabled |= Env::SPACE;
            }
            TurretBlockKind::ContinuousLiquidTurret => {
                self.base.has_liquids = true;
                self.loop_sound = "loopMineBeam".into();
                self.shoot_sound = "none".into();
                self.smoke_effect = "none".into();
                self.shoot_effect = "none".into();
            }
            TurretBlockKind::TractorBeamTurret => {
                self.base.has_power = true;
                self.target_ground = false;
                self.rotate_speed = 10.0;
                self.coolant_multiplier = 1.0;
                self.base.env_enabled |= Env::SPACE;
                self.shoot_sound = "beamParallax".into();
                self.shoot_sound_volume = 0.9;
                self.shoot_cone = 6.0;
            }
            TurretBlockKind::PointDefenseTurret => {
                self.base.has_power = true;
                self.rotate_speed = 20.0;
                self.reload = 30.0;
                self.coolant_multiplier = 2.0;
                self.retarget_time = 5.0;
                self.color = "white".into();
                self.beam_effect = "pointBeam".into();
                self.point_hit_effect = "pointHit".into();
                self.shoot_effect = "sparkShoot".into();
                self.shoot_sound = "shootSegment".into();
                self.shoot_cone = 5.0;
                self.bullet_damage = 10.0;
                self.shoot_length = 3.0;
            }
        }
    }

    fn limit_range(&mut self, margin: f32) {
        for ammo in &mut self.ammo {
            let real_range = ammo.bullet.range_change + self.range;
            ammo.bullet.lifetime =
                (real_range + margin + ammo.bullet.extra_range_margin + 10.0) / ammo.bullet.speed;
        }
        if let Some(shoot_type) = &mut self.shoot_type {
            if shoot_type.speed != 0.0 {
                let real_range = shoot_type.range_change + self.range;
                shoot_type.lifetime =
                    (real_range + margin + shoot_type.extra_range_margin + 10.0) / shoot_type.speed;
            }
        }
    }

    fn consume_coolant(&mut self, amount: f32) {
        self.coolant_amount = amount;
        self.consume_coolant = true;
    }

    fn finalize(&mut self) {
        if self.scaled_health >= 0.0 {
            self.base.health =
                ((self.base.size * self.base.size) as f32 * self.scaled_health) as i32;
        }
        if matches!(self.kind, TurretBlockKind::ItemTurret) && self.target_ground {
            for ammo in &self.ammo {
                self.place_overlap_range = self
                    .place_overlap_range
                    .max(self.range + ammo.bullet.range_change + self.place_overlap_margin);
            }
        }
        if matches!(
            self.kind,
            TurretBlockKind::LiquidTurret | TurretBlockKind::ContinuousLiquidTurret
        ) && self.target_ground
        {
            for ammo in &self.liquid_ammo {
                self.place_overlap_range = self
                    .place_overlap_range
                    .max(self.range + ammo.bullet.range_change + self.place_overlap_margin);
            }
        }
        if !self.disable_overlap_check {
            self.place_overlap_range = self
                .place_overlap_range
                .max(self.range + self.place_overlap_margin);
        }
        self.fog_radius = self
            .fog_radius
            .max((self.range / TILE_SIZE as f32 * self.fog_radius_multiplier).round());
        self.base.liquid_capacity = self.liquid_capacity;
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
    pub legacy_read_warmup: bool,
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
            legacy_read_warmup: false,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitBlockKind {
    UnitFactory,
    UnitAssembler,
    UnitAssemblerModule,
    RepairTower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadBlockKind {
    PayloadConveyor,
    PayloadRouter,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPlanSpec {
    pub unit: String,
    pub time: f32,
    pub requirements: Vec<ItemAmount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadContentSpec {
    Unit(String),
    Block(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadStackSpec {
    pub content: PayloadContentSpec,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssemblerUnitPlanSpec {
    pub unit: String,
    pub time: f32,
    pub payload_requirements: Vec<PayloadStackSpec>,
    pub item_requirements: Vec<ItemAmount>,
    pub liquid_requirements: Vec<LiquidAmount>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFactoryBlockData {
    pub base: Block,
    pub kind: UnitBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub consume_power: f32,
    pub plans: Vec<UnitPlanSpec>,
    pub region_suffix: String,
    pub configurable: bool,
    pub clear_on_double_tap: bool,
    pub outputs_payload: bool,
    pub floating: bool,
    pub rotate: bool,
    pub region_rotated1: i32,
    pub fog_radius: f32,
    pub commandable: bool,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
    pub create_sound: String,
    pub create_sound_volume: f32,
    pub capacities: Vec<ItemAmount>,
}

impl UnitFactoryBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: UnitBlockKind) -> Self {
        let mut base = Block::new(id, name);
        base.update = true;
        base.has_power = true;
        base.has_items = true;
        base.solid = true;
        base.group = BlockGroup::Units;
        base.env_enabled = Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER;
        base.flags.push(BlockFlag::Factory);
        Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            consume_power: 0.0,
            plans: Vec::new(),
            region_suffix: String::new(),
            configurable: true,
            clear_on_double_tap: true,
            outputs_payload: true,
            floating: false,
            rotate: true,
            region_rotated1: 1,
            fog_radius: 0.0,
            commandable: true,
            ambient_sound: "loopUnitBuilding".into(),
            ambient_sound_volume: 0.09,
            create_sound: "unitCreate".into(),
            create_sound_volume: 1.0,
            capacities: Vec::new(),
        }
    }

    fn finalize(&mut self) {
        self.base.consumes_power = self.consume_power > 0.0;
        self.base.item_capacity = 10;
        self.capacities.clear();
        for plan in &self.plans {
            for requirement in &plan.requirements {
                if let Some(existing) = self
                    .capacities
                    .iter_mut()
                    .find(|capacity| capacity.item == requirement.item)
                {
                    existing.amount = existing.amount.max(requirement.amount * 2);
                } else {
                    self.capacities.push(ItemAmount {
                        item: requirement.item,
                        amount: requirement.amount * 2,
                    });
                }
                self.base.item_capacity = self.base.item_capacity.max(requirement.amount * 2);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAssemblerBlockData {
    pub base: Block,
    pub kind: UnitBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub consume_power: f32,
    pub consume_liquids: Vec<LiquidAmount>,
    pub plans: Vec<AssemblerUnitPlanSpec>,
    pub region_suffix: String,
    pub outputs_payload: bool,
    pub accepts_payload: bool,
    pub accepts_unit_payloads: bool,
    pub floating: bool,
    pub rotate: bool,
    pub rotate_draw: bool,
    pub quick_rotate: bool,
    pub region_rotated1: i32,
    pub area_size: i32,
    pub drone_type: String,
    pub drones_created: i32,
    pub drone_construct_time: f32,
    pub commandable: bool,
    pub ambient_sound: String,
    pub ambient_sound_volume: f32,
    pub create_sound: String,
    pub create_sound_volume: f32,
    pub payload_speed: f32,
    pub payload_rotate_speed: f32,
    pub capacities: Vec<ItemAmount>,
    pub liquid_filter: Vec<ContentId>,
}

impl UnitAssemblerBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: UnitBlockKind) -> Self {
        let mut base = Block::new(id, name);
        base.update = true;
        base.sync = true;
        base.solid = true;
        base.has_items = true;
        base.group = BlockGroup::Units;
        base.env_enabled = Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER;
        base.flags.push(BlockFlag::UnitAssembler);
        Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            consume_power: 0.0,
            consume_liquids: Vec::new(),
            plans: Vec::new(),
            region_suffix: String::new(),
            outputs_payload: false,
            accepts_payload: true,
            accepts_unit_payloads: true,
            floating: false,
            rotate: true,
            rotate_draw: false,
            quick_rotate: false,
            region_rotated1: 1,
            area_size: 11,
            drone_type: "assembly-drone".into(),
            drones_created: 4,
            drone_construct_time: 60.0 * 4.0,
            commandable: true,
            ambient_sound: "loopUnitBuilding".into(),
            ambient_sound_volume: 0.13,
            create_sound: "unitCreateBig".into(),
            create_sound_volume: 1.0,
            payload_speed: 0.7,
            payload_rotate_speed: 5.0,
            capacities: Vec::new(),
            liquid_filter: Vec::new(),
        }
    }

    fn finalize(&mut self) {
        self.base.consumes_power = self.consume_power > 0.0;
        self.base.has_power = self.consume_power > 0.0;
        self.base.has_liquids = !self.consume_liquids.is_empty()
            || self
                .plans
                .iter()
                .any(|plan| !plan.liquid_requirements.is_empty());
        self.base.item_capacity = 10;
        self.capacities.clear();
        self.liquid_filter.clear();
        for plan in &self.plans {
            for requirement in &plan.item_requirements {
                if let Some(existing) = self
                    .capacities
                    .iter_mut()
                    .find(|capacity| capacity.item == requirement.item)
                {
                    existing.amount = existing.amount.max(requirement.amount * 2);
                } else {
                    self.capacities.push(ItemAmount {
                        item: requirement.item,
                        amount: requirement.amount * 2,
                    });
                }
                self.base.item_capacity = self.base.item_capacity.max(requirement.amount * 2);
            }
            for requirement in &plan.liquid_requirements {
                if !self.liquid_filter.contains(&requirement.liquid) {
                    self.liquid_filter.push(requirement.liquid);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAssemblerModuleBlockData {
    pub base: Block,
    pub kind: UnitBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub consume_power: f32,
    pub region_suffix: String,
    pub tier: i32,
    pub accepts_payload: bool,
    pub accepts_unit_payloads: bool,
    pub floating: bool,
    pub rotate: bool,
    pub rotate_draw: bool,
    pub region_rotated1: i32,
    pub payload_speed: f32,
    pub payload_rotate_speed: f32,
}

impl UnitAssemblerModuleBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: UnitBlockKind) -> Self {
        let mut base = Block::new(id, name);
        base.update = true;
        base.sync = true;
        base.group = BlockGroup::Payloads;
        base.env_enabled = Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER;
        Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            consume_power: 0.0,
            region_suffix: String::new(),
            tier: 1,
            accepts_payload: true,
            accepts_unit_payloads: true,
            floating: false,
            rotate: true,
            rotate_draw: false,
            region_rotated1: -1,
            payload_speed: 0.7,
            payload_rotate_speed: 5.0,
        }
    }

    fn finalize(&mut self) {
        self.base.consumes_power = self.consume_power > 0.0;
        self.base.has_power = self.consume_power > 0.0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRepairTowerBlockData {
    pub base: Block,
    pub kind: UnitBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub consume_power: f32,
    pub consume_liquids: Vec<LiquidAmount>,
    pub range: f32,
    pub heal_amount: f32,
    pub suppressable: bool,
    pub circle_color: String,
    pub glow_color: String,
    pub circle_speed: f32,
    pub circle_stroke: f32,
    pub square_rad: f32,
    pub square_spin_scl: f32,
    pub glow_mag: f32,
    pub glow_scl: f32,
}

impl UnitRepairTowerBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: UnitBlockKind) -> Self {
        let mut base = Block::new(id, name);
        base.update = true;
        base.solid = true;
        base.flags.push(BlockFlag::Repair);
        Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            consume_power: 0.0,
            consume_liquids: Vec::new(),
            range: 80.0,
            heal_amount: 1.0,
            suppressable: true,
            circle_color: "heal".into(),
            glow_color: "heal@0.5".into(),
            circle_speed: 120.0,
            circle_stroke: 3.0,
            square_rad: 3.0,
            square_spin_scl: 0.8,
            glow_mag: 0.5,
            glow_scl: 8.0,
        }
    }

    fn finalize(&mut self) {
        self.base.consumes_power = self.consume_power > 0.0;
        self.base.has_power = self.consume_power > 0.0;
        self.base.has_liquids = !self.consume_liquids.is_empty();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadBlockData {
    pub base: Block,
    pub kind: PayloadBlockKind,
    pub requirements: Vec<ItemAmount>,
    pub research_cost: Vec<ItemAmount>,
    pub research_cost_multiplier: f32,
    pub move_time: f32,
    pub move_force: f32,
    pub interp: String,
    pub payload_limit: f32,
    pub push_units: bool,
    pub rotate: bool,
    pub outputs_payload: bool,
    pub accepts_payload: bool,
    pub accepts_unit_payloads: bool,
    pub output_facing: bool,
    pub no_update_disabled: bool,
    pub under_bullets: bool,
    pub can_overdrive: bool,
    pub configurable: bool,
    pub clear_on_double_tap: bool,
    pub invert: bool,
}

impl PayloadBlockData {
    pub fn new(id: BlockId, name: impl Into<String>, kind: PayloadBlockKind) -> Self {
        let mut base = Block::new(id, name);
        base.group = BlockGroup::Payloads;
        base.size = 3;
        base.update = true;
        base.sync = true;
        base.priority = -1;
        base.env_enabled = Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER;
        let mut block = Self {
            base,
            kind,
            requirements: Vec::new(),
            research_cost: Vec::new(),
            research_cost_multiplier: 1.0,
            move_time: 45.0,
            move_force: 201.0,
            interp: "pow5".into(),
            payload_limit: 3.0,
            push_units: true,
            rotate: true,
            outputs_payload: true,
            accepts_payload: false,
            accepts_unit_payloads: true,
            output_facing: true,
            no_update_disabled: true,
            under_bullets: true,
            can_overdrive: true,
            configurable: false,
            clear_on_double_tap: false,
            invert: false,
        };
        if matches!(block.kind, PayloadBlockKind::PayloadRouter) {
            block.apply_router_defaults();
        }
        block
    }

    fn apply_router_defaults(&mut self) {
        self.outputs_payload = true;
        self.output_facing = false;
        self.configurable = true;
        self.clear_on_double_tap = true;
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
    Storage(StorageBlockData),
    Turret(TurretBlockData),
    Crafting(CraftingBlockData),
    DefenseWall(DefenseWallData),
    Effect(EffectBlockData),
    Distribution(DistributionBlockData),
    Liquid(LiquidBlockData),
    Power(PowerBlockData),
    UnitFactory(UnitFactoryBlockData),
    UnitAssembler(UnitAssemblerBlockData),
    UnitAssemblerModule(UnitAssemblerModuleBlockData),
    UnitRepairTower(UnitRepairTowerBlockData),
    Payload(PayloadBlockData),
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
            Self::Storage(storage) => &storage.base,
            Self::Turret(turret) => &turret.base,
            Self::Crafting(crafting) => &crafting.base,
            Self::DefenseWall(wall) => &wall.base,
            Self::Effect(effect) => &effect.base,
            Self::Distribution(distribution) => &distribution.base,
            Self::Liquid(liquid) => &liquid.base,
            Self::Power(power) => &power.base,
            Self::UnitFactory(factory) => &factory.base,
            Self::UnitAssembler(assembler) => &assembler.base,
            Self::UnitAssemblerModule(module) => &module.base,
            Self::UnitRepairTower(tower) => &tower.base,
            Self::Payload(payload) => &payload.base,
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

    pub fn get_storage_by_name(&self, name: &str) -> Option<&StorageBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Storage(storage) => Some(storage),
            _ => None,
        }
    }

    pub fn get_turret_by_name(&self, name: &str) -> Option<&TurretBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Turret(turret) => Some(turret),
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

    pub fn get_unit_factory_by_name(&self, name: &str) -> Option<&UnitFactoryBlockData> {
        match self.get_by_name(name)? {
            BlockDef::UnitFactory(factory) => Some(factory),
            _ => None,
        }
    }

    pub fn get_unit_assembler_by_name(&self, name: &str) -> Option<&UnitAssemblerBlockData> {
        match self.get_by_name(name)? {
            BlockDef::UnitAssembler(assembler) => Some(assembler),
            _ => None,
        }
    }

    pub fn get_unit_assembler_module_by_name(
        &self,
        name: &str,
    ) -> Option<&UnitAssemblerModuleBlockData> {
        match self.get_by_name(name)? {
            BlockDef::UnitAssemblerModule(module) => Some(module),
            _ => None,
        }
    }

    pub fn get_unit_repair_tower_by_name(&self, name: &str) -> Option<&UnitRepairTowerBlockData> {
        match self.get_by_name(name)? {
            BlockDef::UnitRepairTower(tower) => Some(tower),
            _ => None,
        }
    }

    pub fn get_payload_by_name(&self, name: &str) -> Option<&PayloadBlockData> {
        match self.get_by_name(name)? {
            BlockDef::Payload(payload) => Some(payload),
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

    pub fn register_storage_block(
        &mut self,
        name: impl Into<String>,
        kind: StorageBlockKind,
        configure: impl FnOnce(&mut StorageBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = StorageBlockData::new(id, name, kind);
        configure(&mut block);
        block.finalize();
        block.base.derive_layout_fields();
        self.insert(BlockDef::Storage(block))
    }

    pub fn register_turret_block(
        &mut self,
        name: impl Into<String>,
        kind: TurretBlockKind,
        configure: impl FnOnce(&mut TurretBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = TurretBlockData::new(id, name, kind);
        configure(&mut block);
        block.finalize();
        block.base.derive_layout_fields();
        self.insert(BlockDef::Turret(block))
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

    pub fn register_unit_factory_block(
        &mut self,
        name: impl Into<String>,
        kind: UnitBlockKind,
        configure: impl FnOnce(&mut UnitFactoryBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = UnitFactoryBlockData::new(id, name, kind);
        configure(&mut block);
        block.finalize();
        block.base.derive_layout_fields();
        self.insert(BlockDef::UnitFactory(block))
    }

    pub fn register_unit_assembler_block(
        &mut self,
        name: impl Into<String>,
        kind: UnitBlockKind,
        configure: impl FnOnce(&mut UnitAssemblerBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = UnitAssemblerBlockData::new(id, name, kind);
        configure(&mut block);
        block.finalize();
        block.base.derive_layout_fields();
        self.insert(BlockDef::UnitAssembler(block))
    }

    pub fn register_unit_assembler_module_block(
        &mut self,
        name: impl Into<String>,
        kind: UnitBlockKind,
        configure: impl FnOnce(&mut UnitAssemblerModuleBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = UnitAssemblerModuleBlockData::new(id, name, kind);
        configure(&mut block);
        block.finalize();
        block.base.derive_layout_fields();
        self.insert(BlockDef::UnitAssemblerModule(block))
    }

    pub fn register_unit_repair_tower_block(
        &mut self,
        name: impl Into<String>,
        kind: UnitBlockKind,
        configure: impl FnOnce(&mut UnitRepairTowerBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = UnitRepairTowerBlockData::new(id, name, kind);
        configure(&mut block);
        block.finalize();
        block.base.derive_layout_fields();
        self.insert(BlockDef::UnitRepairTower(block))
    }

    pub fn register_payload_block(
        &mut self,
        name: impl Into<String>,
        kind: PayloadBlockKind,
        configure: impl FnOnce(&mut PayloadBlockData),
    ) -> BlockId {
        let id = self.next_id();
        let mut block = PayloadBlockData::new(id, name, kind);
        configure(&mut block);
        block.base.derive_layout_fields();
        self.insert(BlockDef::Payload(block))
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
    register_storage_blocks(&mut registry, items);
    register_turret_blocks(&mut registry, items, liquids);
    register_crafting_blocks(&mut registry, items, liquids);
    register_defense_walls(&mut registry, items);
    register_effect_blocks(&mut registry, items, liquids);
    register_distribution_blocks(&mut registry, items, liquids);
    register_liquid_blocks(&mut registry, items, liquids);
    register_power_blocks(&mut registry, items, liquids);
    register_unit_blocks(&mut registry, items, liquids);
    register_payload_blocks(&mut registry, items);

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

fn item_multiplier(items: &[Item], name: &str, multiplier: f32) -> Option<ItemMultiplier> {
    Some(ItemMultiplier {
        item: find_item(items, name)?.base.mappable.base.id,
        multiplier,
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

fn push_item_multiplier(
    target: &mut Vec<ItemMultiplier>,
    items: &[Item],
    name: &str,
    multiplier: f32,
) {
    if let Some(item) = item_multiplier(items, name, multiplier) {
        target.push(item);
    }
}

fn push_blocked_item(target: &mut Vec<ContentId>, items: &[Item], name: &str) {
    if let Some(item) = find_item(items, name) {
        target.push(item.base.mappable.base.id);
    }
}

fn push_turret_ammo(target: &mut Vec<TurretAmmo>, items: &[Item], name: &str, bullet: BulletSpec) {
    if let Some(item) = find_item(items, name) {
        target.push(TurretAmmo {
            item: item.base.mappable.base.id,
            bullet,
        });
    }
}

fn push_liquid_turret_ammo(
    target: &mut Vec<LiquidTurretAmmo>,
    liquids: &[Liquid],
    name: &str,
    bullet: BulletSpec,
) {
    if let Some(liquid) = liquid_id(liquids, name) {
        target.push(LiquidTurretAmmo { liquid, bullet });
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

fn unit_plan(items: &[Item], unit: &str, time: f32, specs: &[(&str, i32)]) -> UnitPlanSpec {
    let mut requirements = Vec::new();
    set_requirements(&mut requirements, items, specs);
    UnitPlanSpec {
        unit: unit.into(),
        time,
        requirements,
    }
}

fn unit_payload(name: &str, amount: i32) -> PayloadStackSpec {
    PayloadStackSpec {
        content: PayloadContentSpec::Unit(name.into()),
        amount,
    }
}

fn block_payload(name: &str, amount: i32) -> PayloadStackSpec {
    PayloadStackSpec {
        content: PayloadContentSpec::Block(name.into()),
        amount,
    }
}

fn assembler_unit_plan(
    unit: &str,
    time: f32,
    payload_requirements: Vec<PayloadStackSpec>,
) -> AssemblerUnitPlanSpec {
    AssemblerUnitPlanSpec {
        unit: unit.into(),
        time,
        payload_requirements,
        item_requirements: Vec::new(),
        liquid_requirements: Vec::new(),
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

fn rgba_name(color: u32) -> String {
    format!("{color:08x}")
}

fn liquid_bullet(liquids: &[Liquid], name: &str) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Liquid, 3.5, 0.0);
    bullet.ammo_multiplier = 1.0;
    bullet.lifetime = 34.0;
    bullet.status_duration = 60.0 * 2.0;
    bullet.despawn_effect = "none".into();
    bullet.hit_effect = "hitLiquid".into();
    bullet.smoke_effect = "none".into();
    bullet.shoot_effect = "none".into();
    bullet.drag = 0.001;
    bullet.knockback = 0.55;
    bullet.display_ammo_multiplier = false;
    bullet.puddle_size = 6.0;
    bullet.orb_size = 3.0;
    bullet.boil_time = 5.0;

    if let Some(liquid) = liquids
        .iter()
        .find(|liquid| liquid.base.mappable.name.as_str() == name)
    {
        bullet.status = liquid.effect.clone().unwrap_or_else(|| "none".into());
        bullet.hit_color = rgba_name(liquid.color_rgba);
        bullet.light_color = rgba_name(liquid.light_color_rgba);
    }

    bullet
}

fn basic_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Basic, speed, damage);
    bullet.width = 5.0;
    bullet.height = 7.0;
    bullet.shrink_y = 0.5;
    bullet.back_color = "bulletYellowBack".into();
    bullet.front_color = "bulletYellow".into();
    bullet.hit_effect = "hitBulletSmall".into();
    bullet.despawn_effect = "hitBulletSmall".into();
    bullet
}

fn missile_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Missile, speed, damage);
    bullet.back_color = "missileYellowBack".into();
    bullet.front_color = "missileYellow".into();
    bullet.homing_power = 0.08;
    bullet.shrink_y = 0.0;
    bullet.width = 8.0;
    bullet.height = 8.0;
    bullet.lifetime = 52.0;
    bullet
}

fn flak_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Flak, speed, damage);
    bullet.splash_damage = 15.0;
    bullet.splash_damage_radius = 34.0;
    bullet.hit_effect = "flakExplosionBig".into();
    bullet.width = 8.0;
    bullet.height = 10.0;
    bullet.collides_ground = false;
    bullet.explode_range = 30.0;
    bullet.explode_delay = 5.0;
    bullet.flak_delay = 0.0;
    bullet.flak_interval = 6.0;
    bullet
}

fn artillery_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Artillery, speed, damage);
    bullet.collides_tiles = false;
    bullet.collides = false;
    bullet.collides_air = false;
    bullet.scale_life = true;
    bullet.hit_effect = "flakExplosion".into();
    bullet.shoot_effect = "shootBig".into();
    bullet.trail_effect = "artilleryTrail".into();
    bullet.shrink_x = 0.15;
    bullet.shrink_y = 0.5;
    bullet
}

fn rail_bullet() -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Rail, 0.0, 0.0);
    bullet.pierce_building = true;
    bullet.pierce = true;
    bullet.reflectable = false;
    bullet.hit_effect = "none".into();
    bullet.despawn_effect = "none".into();
    bullet.collides = false;
    bullet.keep_velocity = false;
    bullet.lifetime = 1.0;
    bullet.delay_frags = true;
    bullet.length = 100.0;
    bullet.point_effect_space = 20.0;
    bullet
}

fn continuous_laser_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::ContinuousLaser, 0.0, damage);
    bullet.length = 220.0;
    bullet.shake = 1.0;
    bullet.damage_interval = 5.0;
    bullet.hit_large = true;
    bullet.continuous = true;
    bullet.timescale_damage = false;
    bullet.remove_after_pierce = false;
    bullet.pierce_cap = -1;
    bullet.speed = 0.0;
    bullet.despawn_effect = "none".into();
    bullet.shoot_effect = "none".into();
    bullet.lifetime = 16.0;
    bullet.impact = true;
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet.hit_effect = "hitBeam".into();
    bullet.hit_size = 4.0;
    bullet.draw_size = 420.0;
    bullet.hit_color = "ff9c5a".into();
    bullet.incend_amount = 1;
    bullet.incend_spread = 5.0;
    bullet.incend_chance = 0.4;
    bullet.light_color = "orange".into();
    bullet.width = 9.0;
    bullet
}

fn continuous_flame_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::ContinuousFlame, 0.0, damage);
    bullet.length = 120.0;
    bullet.damage_interval = 5.0;
    bullet.continuous = true;
    bullet.remove_after_pierce = false;
    bullet.pierce_cap = -1;
    bullet.speed = 0.0;
    bullet.despawn_effect = "none".into();
    bullet.shoot_effect = "none".into();
    bullet.lifetime = 16.0;
    bullet.impact = true;
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet.optimal_life_fract = 0.5;
    bullet.hit_effect = "hitFlameBeam".into();
    bullet.hit_size = 4.0;
    bullet.draw_size = 420.0;
    bullet.hit_color = "e189f5".into();
    bullet.light_color = "e189f5".into();
    bullet.laser_absorb = false;
    bullet.ammo_multiplier = 1.0;
    bullet.pierce_armor = true;
    bullet.width = 3.7;
    bullet.colors = vec![
        "eb7abe88".into(),
        "e189f5b2".into(),
        "907ef7cc".into(),
        "91a4ff".into(),
        "white".into(),
    ];
    bullet
}

fn point_laser_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::PointLaser, 0.0, damage);
    bullet.sprite = "point-laser".into();
    bullet.color = "white".into();
    bullet.beam_effect = "colorTrail".into();
    bullet.beam_effect_interval = 3.0;
    bullet.beam_effect_size = 3.5;
    bullet.osc_scl = 2.0;
    bullet.osc_mag = 0.3;
    bullet.damage_interval = 5.0;
    bullet.shake = 0.0;
    bullet.remove_after_pierce = false;
    bullet.despawn_effect = "none".into();
    bullet.lifetime = 20.0;
    bullet.impact = true;
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet.optimal_life_fract = 0.5;
    bullet.shoot_effect = "none".into();
    bullet.smoke_effect = "none".into();
    bullet.draw_size = 1000.0;
    bullet.hit_effect = "hitBulletSmall".into();
    bullet.ammo_multiplier = 2.0;
    bullet
}

fn laser_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Laser, 0.0, damage);
    bullet.colors = vec![
        "lancerLaser@0.4".into(),
        "lancerLaser".into(),
        "white".into(),
    ];
    bullet.hit_effect = "hitLaserBlast".into();
    bullet.hit_color = "white".into();
    bullet.despawn_effect = "none".into();
    bullet.shoot_effect = "hitLancer".into();
    bullet.smoke_effect = "none".into();
    bullet.hit_size = 4.0;
    bullet.lifetime = 16.0;
    bullet.impact = true;
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet.length = 160.0;
    bullet.width = 15.0;
    bullet.side_length = 29.0;
    bullet.side_width = 0.7;
    bullet.side_angle = 90.0;
    bullet
}

fn explosion_bullet(splash_damage: f32, splash_damage_radius: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Explosion, 0.0, 0.0);
    bullet.splash_damage = splash_damage;
    bullet.splash_damage_radius = splash_damage_radius;
    bullet.range_override = (splash_damage_radius * 2.0 / 3.0).max(20.0);
    bullet.hittable = false;
    bullet.lifetime = 1.0;
    bullet.speed = 0.0;
    bullet.shoot_effect = "massiveExplosion".into();
    bullet.instant_disappear = true;
    bullet.scaled_splash_damage = true;
    bullet.kill_shooter = true;
    bullet.collides = false;
    bullet.keep_velocity = false;
    bullet
}

fn missile_unit(
    name: &str,
    color: &str,
    speed: f32,
    lifetime: f32,
    health: f32,
    rotate_speed: f32,
    missile_accel_time: f32,
    trail_length: i32,
    engine_size: f32,
    engine_offset: f32,
) -> MissileUnitSpec {
    MissileUnitSpec {
        name: name.into(),
        speed,
        max_range: 6.0,
        lifetime,
        hit_size: 10.0,
        outline_color: "darkOutline".into(),
        engine_color: color.into(),
        trail_color: color.into(),
        engine_layer: "effect".into(),
        engine_size,
        engine_offset,
        rotate_speed,
        trail_length,
        missile_accel_time,
        low_altitude: true,
        loop_sound: "loopMissileTrail".into(),
        loop_sound_volume: 0.6,
        death_sound: "explosionMissile".into(),
        target_air: false,
        target_under_blocks: false,
        fog_radius: 6.0,
        health,
        parts: Vec::new(),
        weapons: Vec::new(),
        abilities: Vec::new(),
    }
}

fn death_weapon(bullet: BulletSpec) -> WeaponSpec {
    WeaponSpec {
        shoot_cone: 360.0,
        mirror: false,
        rotate: false,
        rotation_limit: 0.0,
        rotate_speed: 0.0,
        reload: 1.0,
        death_explosion_effect: "massiveExplosion".into(),
        shoot_on_death: true,
        shake: 10.0,
        bullet: Some(Box::new(bullet)),
    }
}

fn move_effect_ability(effect: &str, color: &str, interval: f32) -> AbilitySpec {
    AbilitySpec {
        kind: AbilityKind::MoveEffect,
        effect: effect.into(),
        rotation: 180.0,
        y: -9.0,
        color: color.into(),
        interval,
        radius: 0.0,
        regen: 0.0,
        max: 0.0,
        cooldown: 0.0,
    }
}

fn force_field_ability(radius: f32, regen: f32, max: f32, cooldown: f32) -> AbilitySpec {
    AbilitySpec {
        kind: AbilityKind::ForceField,
        effect: String::new(),
        rotation: 0.0,
        y: 0.0,
        color: String::new(),
        interval: 0.0,
        radius,
        regen,
        max,
        cooldown,
    }
}

fn shrapnel_bullet() -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Shrapnel, 0.0, 1.0);
    bullet.length = 100.0;
    bullet.width = 20.0;
    bullet.from_color = "white".into();
    bullet.to_color = "lancerLaser".into();
    bullet.hit_large = false;
    bullet.serrations = 7;
    bullet.serration_len_scl = 10.0;
    bullet.serration_width = 4.0;
    bullet.serration_spacing = 8.0;
    bullet.serration_space_offset = 80.0;
    bullet.serration_fade_offset = 0.5;
    bullet.hit_effect = "hitLancer".into();
    bullet.shoot_effect = "lightningShoot".into();
    bullet.smoke_effect = "lightningShoot".into();
    bullet.lifetime = 10.0;
    bullet.despawn_effect = "none".into();
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet
}

fn shoot_barrels(specs: &[(f32, f32, f32)]) -> Vec<ShootBarrel> {
    specs
        .iter()
        .map(|(x, y, rotation)| ShootBarrel {
            x: *x,
            y: *y,
            rotation: *rotation,
        })
        .collect()
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

    registry.register_production_block(
        "water-extractor",
        ProductionBlockKind::SolidPump,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[
                    ("metaglass", 30),
                    ("graphite", 30),
                    ("lead", 30),
                    ("copper", 30),
                ],
            );
            production.result_liquid = liquid_id(liquids, "water");
            production.pump_amount = 0.11;
            production.base.size = 2;
            production.base.liquid_capacity = 40.0;
            production.rotate_speed = 1.4;
            production.attribute = "water".into();
            production.base.env_required |= Env::GROUND_WATER;
            production.consume_power = 1.5;
        },
    );

    registry.register_production_block(
        "oil-extractor",
        ProductionBlockKind::Fracker,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[
                    ("copper", 150),
                    ("graphite", 175),
                    ("lead", 115),
                    ("thorium", 115),
                    ("silicon", 75),
                ],
            );
            production.result_liquid = liquid_id(liquids, "oil");
            production.update_effect = "pulverize".into();
            production.update_effect_chance = 0.05;
            production.pump_amount = 0.25;
            production.base.size = 3;
            production.base.liquid_capacity = 40.0;
            production.attribute = "oil".into();
            production.base_efficiency = 0.0;
            production.item_use_time = 60.0;
            push_item_amount(&mut production.consume_items, items, "sand", 1);
            production.consume_power = 3.0;
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "water",
                0.15,
                false,
            );
        },
    );

    registry.register_production_block(
        "cliff-crusher",
        ProductionBlockKind::WallCrafter,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[("graphite", 25), ("beryllium", 20)],
            );
            production.consume_power = 11.0 / 60.0;
            production.drill_time = 110.0;
            production.base.size = 2;
            production.attribute = "sand".into();
            production.output_item = item_amount(items, "sand", 1).map(|amount| amount.item);
            production.fog_radius = 2.0;
            set_requirements(
                &mut production.research_cost,
                items,
                &[("beryllium", 100), ("graphite", 40)],
            );
            production.ambient_sound = "loopDrill".into();
            production.ambient_sound_volume = 0.04;
        },
    );

    registry.register_production_block(
        "large-cliff-crusher",
        ProductionBlockKind::WallCrafter,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[
                    ("silicon", 80),
                    ("surge-alloy", 15),
                    ("beryllium", 100),
                    ("tungsten", 50),
                ],
            );
            production.consume_power = 1.0;
            production.drill_time = 48.0;
            production.base.size = 3;
            production.attribute = "sand".into();
            production.output_item = item_amount(items, "sand", 1).map(|amount| amount.item);
            production.fog_radius = 3.0;
            production.ambient_sound = "loopDrill".into();
            production.ambient_sound_volume = 0.08;
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "hydrogen",
                1.0 / 60.0,
                false,
            );
            push_item_amount(&mut production.consume_item_boosts, items, "graphite", 1);
            production.base.item_capacity = 20;
            production.boost_item_use_time = 60.0 / 0.75;
        },
    );

    registry.register_production_block(
        "plasma-bore",
        ProductionBlockKind::BeamDrill,
        |production| {
            set_requirements(&mut production.requirements, items, &[("beryllium", 40)]);
            production.consume_power = 0.15;
            production.drill_time = 160.0;
            production.tier = 3;
            production.base.size = 2;
            production.range = 5;
            production.fog_radius = 3.0;
            set_requirements(&mut production.research_cost, items, &[("beryllium", 10)]);
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "hydrogen",
                0.25 / 60.0,
                true,
            );
        },
    );

    registry.register_production_block(
        "large-plasma-bore",
        ProductionBlockKind::BeamDrill,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[
                    ("silicon", 100),
                    ("oxide", 25),
                    ("beryllium", 100),
                    ("tungsten", 70),
                ],
            );
            production.consume_power = 0.8;
            production.drill_time = 100.0;
            production.tier = 5;
            production.base.size = 3;
            production.range = 6;
            production.fog_radius = 4.0;
            production.laser_width = 0.7;
            production.base.item_capacity = 20;
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "hydrogen",
                0.5 / 60.0,
                false,
            );
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "nitrogen",
                3.0 / 60.0,
                true,
            );
            set_requirements(
                &mut production.research_cost,
                items,
                &[
                    ("silicon", 1500),
                    ("oxide", 200),
                    ("beryllium", 3000),
                    ("tungsten", 1200),
                ],
            );
        },
    );

    registry.register_production_block(
        "impact-drill",
        ProductionBlockKind::BurstDrill,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[("silicon", 70), ("beryllium", 90), ("graphite", 60)],
            );
            production.drill_time = 60.0 * 12.0;
            production.base.size = 4;
            production.base.has_power = true;
            production.tier = 6;
            production.drill_effect =
                "MultiEffect(mineImpact, drillSteam, mineImpactWave(redLight, 40))".into();
            production.shake = 4.0;
            production.base.item_capacity = 40;
            push_blocked_item(&mut production.blocked_items, items, "thorium");
            production.research_cost_multiplier = 0.5;
            push_item_multiplier(&mut production.drill_multipliers, items, "beryllium", 2.0);
            production.liquid_boost_intensity = 1.75;
            production.fog_radius = 4.0;
            production.consume_power = 160.0 / 60.0;
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "water",
                10.0 / 60.0,
                false,
            );
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "ozone",
                3.0 / 60.0,
                true,
            );
        },
    );

    registry.register_production_block(
        "eruption-drill",
        ProductionBlockKind::BurstDrill,
        |production| {
            set_requirements(
                &mut production.requirements,
                items,
                &[
                    ("silicon", 300),
                    ("oxide", 20),
                    ("tungsten", 250),
                    ("thorium", 150),
                ],
            );
            production.drill_time = 281.25;
            production.base.size = 5;
            production.base.has_power = true;
            production.tier = 7;
            production.drill_effect = "MultiEffect(mineImpact, drillSteam, dynamicSpikes(hydrogen, 30), mineImpactWave(hydrogen, 45))".into();
            production.shake = 4.0;
            production.base.item_capacity = 60;
            production.arrow_offset = 2.0;
            production.arrow_spacing = 5.0;
            production.arrows = 2;
            production.glow_color_alpha = 0.6;
            production.fog_radius = 5.0;
            push_item_multiplier(&mut production.drill_multipliers, items, "beryllium", 2.0);
            production.liquid_boost_intensity = 2.0;
            production.consume_power = 6.0;
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "hydrogen",
                4.0 / 60.0,
                false,
            );
            push_liquid_consume(
                &mut production.consume_liquids,
                liquids,
                "cyanogen",
                0.75 / 60.0,
                true,
            );
        },
    );
}

fn register_storage_blocks(registry: &mut BlockRegistry, items: &[Item]) {
    registry.register_storage_block("core-shard", StorageBlockKind::Core, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[("copper", 1000), ("lead", 800)],
        );
        storage.base.build_visibility = BuildVisibility::CoreZoneOnly;
        storage.always_unlocked = true;
        storage.is_first_tier = true;
        storage.unit_type = "alpha".into();
        storage.base.health = 1100;
        storage.base.item_capacity = 4000;
        storage.base.size = 3;
        storage.build_cost_multiplier = 2.0;
        storage.unit_cap_modifier = 8;
    });

    registry.register_storage_block("core-foundation", StorageBlockKind::Core, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[("copper", 3000), ("lead", 3000), ("silicon", 2000)],
        );
        storage.unit_type = "beta".into();
        storage.base.health = 3500;
        storage.base.item_capacity = 9000;
        storage.base.size = 4;
        storage.thruster_length = 34.0 / 4.0;
        storage.unit_cap_modifier = 16;
        storage.research_cost_multiplier = 0.07;
    });

    registry.register_storage_block("core-nucleus", StorageBlockKind::Core, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[
                ("copper", 8000),
                ("lead", 8000),
                ("silicon", 5000),
                ("thorium", 4000),
            ],
        );
        storage.unit_type = "gamma".into();
        storage.base.health = 6000;
        storage.base.item_capacity = 13000;
        storage.base.size = 5;
        storage.thruster_length = 40.0 / 4.0;
        storage.unit_cap_modifier = 24;
        storage.research_cost_multiplier = 0.11;
    });

    registry.register_storage_block("core-bastion", StorageBlockKind::Core, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[("graphite", 1000), ("silicon", 1000), ("beryllium", 800)],
        );
        storage.is_first_tier = true;
        storage.unit_type = "evoke".into();
        storage.base.health = 4500;
        storage.base.item_capacity = 2000;
        storage.base.size = 4;
        storage.thruster_length = 34.0 / 4.0;
        storage.armor = 5.0;
        storage.always_unlocked = true;
        storage.incinerate_non_buildable = true;
        storage.requires_core_zone = true;
        storage.build_cost_multiplier = 0.7;
        storage.unit_cap_modifier = 15;
        storage.research_cost_multiplier = 0.07;
    });

    registry.register_storage_block("core-citadel", StorageBlockKind::Core, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[
                ("silicon", 4000),
                ("beryllium", 4000),
                ("tungsten", 3000),
                ("oxide", 1000),
            ],
        );
        storage.unit_type = "incite".into();
        storage.base.health = 16000;
        storage.base.item_capacity = 3000;
        storage.base.size = 5;
        storage.thruster_length = 40.0 / 4.0;
        storage.armor = 10.0;
        storage.incinerate_non_buildable = true;
        storage.build_cost_multiplier = 0.7;
        storage.requires_core_zone = true;
        storage.unit_cap_modifier = 15;
        push_item_multiplier(
            &mut storage.research_cost_multipliers,
            items,
            "silicon",
            0.5,
        );
        storage.research_cost_multiplier = 0.17;
    });

    registry.register_storage_block("core-acropolis", StorageBlockKind::Core, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[
                ("beryllium", 6000),
                ("silicon", 5000),
                ("tungsten", 5000),
                ("carbide", 3000),
                ("oxide", 3000),
            ],
        );
        storage.unit_type = "emanate".into();
        storage.base.health = 30000;
        storage.base.item_capacity = 4000;
        storage.base.size = 6;
        storage.thruster_length = 48.0 / 4.0;
        storage.armor = 15.0;
        storage.incinerate_non_buildable = true;
        storage.build_cost_multiplier = 0.7;
        storage.requires_core_zone = true;
        storage.unit_cap_modifier = 15;
        push_item_multiplier(
            &mut storage.research_cost_multipliers,
            items,
            "silicon",
            0.4,
        );
        storage.research_cost_multiplier = 0.1;
    });

    registry.register_storage_block("container", StorageBlockKind::Storage, |storage| {
        set_requirements(&mut storage.requirements, items, &[("titanium", 100)]);
        storage.base.size = 2;
        storage.base.item_capacity = 300;
        storage.scaled_health = 55.0;
    });

    registry.register_storage_block("vault", StorageBlockKind::Storage, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[("titanium", 250), ("thorium", 125)],
        );
        storage.base.size = 3;
        storage.base.item_capacity = 1000;
        storage.scaled_health = 55.0;
    });

    registry.register_storage_block(
        "reinforced-container",
        StorageBlockKind::Storage,
        |storage| {
            set_requirements(
                &mut storage.requirements,
                items,
                &[("tungsten", 30), ("graphite", 40)],
            );
            storage.base.size = 2;
            storage.base.item_capacity = 160;
            storage.scaled_health = 120.0;
            storage.core_merge = false;
        },
    );

    registry.register_storage_block("reinforced-vault", StorageBlockKind::Storage, |storage| {
        set_requirements(
            &mut storage.requirements,
            items,
            &[("tungsten", 125), ("thorium", 70), ("beryllium", 100)],
        );
        storage.base.size = 3;
        storage.base.item_capacity = 900;
        storage.scaled_health = 120.0;
        storage.core_merge = false;
    });
}

fn register_turret_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_turret_block("duo", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(&mut turret.requirements, items, &[("copper", 35)]);

        let mut copper = BulletSpec::new(BulletKind::Basic, 2.5, 9.0);
        copper.width = 7.0;
        copper.height = 9.0;
        copper.lifetime = 60.0;
        copper.ammo_multiplier = 2.0;
        copper.hit_effect = "hitBulletColor".into();
        copper.despawn_effect = "hitBulletColor".into();
        copper.hit_color = "copperAmmoBack".into();
        copper.back_color = "copperAmmoBack".into();
        copper.trail_color = "copperAmmoBack".into();
        copper.front_color = "copperAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "copper", copper);

        let mut graphite = BulletSpec::new(BulletKind::Basic, 3.5, 18.0);
        graphite.width = 9.0;
        graphite.height = 12.0;
        graphite.ammo_multiplier = 4.0;
        graphite.lifetime = 60.0;
        graphite.reload_multiplier = 0.8;
        graphite.range_change = 16.0;
        graphite.hit_effect = "hitBulletColor".into();
        graphite.despawn_effect = "hitBulletColor".into();
        graphite.hit_color = "graphiteAmmoBack".into();
        graphite.back_color = "graphiteAmmoBack".into();
        graphite.trail_color = "graphiteAmmoBack".into();
        graphite.front_color = "graphiteAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "graphite", graphite);

        let mut silicon = BulletSpec::new(BulletKind::Basic, 3.0, 12.0);
        silicon.width = 7.0;
        silicon.height = 9.0;
        silicon.homing_power = 0.2;
        silicon.reload_multiplier = 1.5;
        silicon.ammo_multiplier = 5.0;
        silicon.lifetime = 60.0;
        silicon.trail_length = 5;
        silicon.trail_width = 1.5;
        silicon.hit_effect = "hitBulletColor".into();
        silicon.despawn_effect = "hitBulletColor".into();
        silicon.hit_color = "siliconAmmoBack".into();
        silicon.back_color = "siliconAmmoBack".into();
        silicon.trail_color = "siliconAmmoBack".into();
        silicon.front_color = "siliconAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "silicon", silicon);

        turret.shoot_pattern = "ShootAlternate".into();
        turret.shoot_alternate_spread = 3.5;
        turret.shoot_alternate_barrels = 2;
        turret.recoils = 2;
        turret.drawer = "DrawTurret(RegionPart(-barrel-l recoil under moveY=-1.5), RegionPart(-barrel-r recoil under moveY=-1.5))".into();
        turret.shoot_sound = "shootDuo".into();
        turret.recoil = 0.5;
        turret.shoot_y = 3.0;
        turret.reload = 20.0;
        turret.range = 160.0;
        turret.shoot_cone = 15.0;
        turret.ammo_use_effect = "casing1".into();
        turret.base.health = 250;
        turret.inaccuracy = 2.0;
        turret.rotate_speed = 10.0;
        turret.consume_coolant(0.1);
        turret.coolant_multiplier = 10.0;
        turret.research_cost_multiplier = 0.05;
        turret.deposit_cooldown = 2.0;
        turret.limit_range(5.0);
    });

    registry.register_turret_block("scatter", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 85), ("lead", 45)],
        );

        let mut scrap = flak_bullet(4.0, 3.0);
        scrap.lifetime = 60.0;
        scrap.ammo_multiplier = 5.0;
        scrap.shoot_effect = "shootSmall".into();
        scrap.reload_multiplier = 0.5;
        scrap.width = 6.0;
        scrap.height = 8.0;
        scrap.hit_effect = "flakExplosion".into();
        scrap.splash_damage = 22.0 * 1.5;
        scrap.splash_damage_radius = 24.0;
        scrap.front_color = "scrapAmmoFront".into();
        scrap.back_color = "scrapAmmoBack".into();
        scrap.hit_color = "scrapAmmoBack".into();
        scrap.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "scrap", scrap);

        let mut lead = flak_bullet(4.2, 3.0);
        lead.lifetime = 60.0;
        lead.ammo_multiplier = 4.0;
        lead.shoot_effect = "shootSmall".into();
        lead.width = 6.0;
        lead.height = 8.0;
        lead.hit_effect = "flakExplosion".into();
        lead.splash_damage = 27.0 * 1.5;
        lead.splash_damage_radius = 15.0;
        push_turret_ammo(&mut turret.ammo, items, "lead", lead);

        let mut frag = BulletSpec::new(BulletKind::Basic, 3.0, 5.0);
        frag.width = 5.0;
        frag.height = 12.0;
        frag.shrink_y = 1.0;
        frag.lifetime = 20.0;
        frag.back_color = "glassAmmoBack".into();
        frag.trail_color = "glassAmmoBack".into();
        frag.hit_color = "glassAmmoFront".into();
        frag.front_color = "glassAmmoFront".into();
        frag.despawn_effect = "none".into();
        frag.collides_ground = false;

        let mut metaglass = flak_bullet(4.0, 3.0);
        metaglass.back_color = "glassAmmoBack".into();
        metaglass.trail_color = "glassAmmoBack".into();
        metaglass.hit_color = "glassAmmoFront".into();
        metaglass.front_color = "glassAmmoFront".into();
        metaglass.despawn_effect = "hitBulletColor".into();
        metaglass.lifetime = 60.0;
        metaglass.ammo_multiplier = 5.0;
        metaglass.shoot_effect = "shootSmall".into();
        metaglass.reload_multiplier = 0.8;
        metaglass.width = 6.0;
        metaglass.height = 8.0;
        metaglass.hit_effect = "flakExplosion".into();
        metaglass.splash_damage = 30.0 * 1.5;
        metaglass.splash_damage_radius = 20.0;
        metaglass.frag_bullets = 6;
        metaglass.frag_bullet = Some(Box::new(frag));
        push_turret_ammo(&mut turret.ammo, items, "metaglass", metaglass);

        turret.drawer = "DrawTurret(RegionPart(-mid recoil moveY=-1.25))".into();
        turret.reload = 18.0;
        turret.range = 220.0;
        turret.base.size = 2;
        turret.target_ground = false;
        turret.shoot_shot_delay = 5.0;
        turret.shoot_shots = 2;
        turret.recoil = 1.0;
        turret.rotate_speed = 15.0;
        turret.inaccuracy = 17.0;
        turret.shoot_cone = 35.0;
        turret.scaled_health = 200.0;
        turret.shoot_sound = "shootScatter".into();
        turret.consume_coolant(0.2);
        turret.research_cost_multiplier = 0.05;
        turret.deposit_cooldown = 0.5;
        turret.limit_range(2.0);
    });

    registry.register_turret_block("scorch", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 25), ("graphite", 22)],
        );

        let mut coal = BulletSpec::new(BulletKind::Generic, 3.35, 17.0);
        coal.ammo_multiplier = 3.0;
        coal.hit_size = 7.0;
        coal.lifetime = 18.0;
        coal.pierce = true;
        coal.collides_air = false;
        coal.status_duration = 60.0 * 4.0;
        coal.shoot_effect = "shootSmallFlame".into();
        coal.hit_effect = "hitFlameSmall".into();
        coal.despawn_effect = "none".into();
        coal.status = "burning".into();
        coal.hittable = false;
        push_turret_ammo(&mut turret.ammo, items, "coal", coal);

        let mut pyratite = BulletSpec::new(BulletKind::Generic, 4.0, 30.0);
        pyratite.ammo_multiplier = 10.0;
        pyratite.hit_size = 7.0;
        pyratite.lifetime = 18.0;
        pyratite.pierce = true;
        pyratite.collides_air = false;
        pyratite.status_duration = 60.0 * 10.0;
        pyratite.shoot_effect = "shootPyraFlame".into();
        pyratite.hit_effect = "hitFlameSmall".into();
        pyratite.despawn_effect = "none".into();
        pyratite.status = "burning".into();
        pyratite.hittable = false;
        push_turret_ammo(&mut turret.ammo, items, "pyratite", pyratite);

        turret.recoil = 0.0;
        turret.reload = 6.0;
        turret.coolant_multiplier = 1.5;
        turret.range = 60.0;
        turret.shoot_y = 3.0;
        turret.shoot_cone = 50.0;
        turret.target_air = false;
        turret.ammo_use_effect = "none".into();
        turret.base.health = 400;
        turret.shoot_sound = "shootFlame".into();
        turret.consume_coolant(0.1);
        turret.deposit_cooldown = 1.0;
    });

    registry.register_turret_block("hail", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 40), ("graphite", 17)],
        );

        let mut graphite = artillery_bullet(3.0, 20.0);
        graphite.knockback = 0.8;
        graphite.lifetime = 80.0;
        graphite.width = 11.0;
        graphite.height = 11.0;
        graphite.splash_damage_radius = 25.0 * 0.75;
        graphite.splash_damage = 33.0;
        graphite.hit_color = "graphiteAmmoBack".into();
        graphite.back_color = "graphiteAmmoBack".into();
        graphite.trail_color = "graphiteAmmoBack".into();
        graphite.front_color = "graphiteAmmoFront".into();
        graphite.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "graphite", graphite);

        let mut silicon = artillery_bullet(3.0, 20.0);
        silicon.knockback = 0.8;
        silicon.lifetime = 80.0;
        silicon.width = 11.0;
        silicon.height = 11.0;
        silicon.splash_damage_radius = 25.0 * 0.75;
        silicon.splash_damage = 33.0;
        silicon.reload_multiplier = 1.2;
        silicon.ammo_multiplier = 3.0;
        silicon.homing_power = 0.08;
        silicon.homing_range = 50.0;
        silicon.trail_length = 7;
        silicon.trail_width = 3.0;
        silicon.hit_color = "siliconAmmoBack".into();
        silicon.back_color = "siliconAmmoBack".into();
        silicon.trail_color = "siliconAmmoBack".into();
        silicon.front_color = "siliconAmmoFront".into();
        silicon.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "silicon", silicon);

        let mut pyratite = artillery_bullet(3.0, 25.0);
        pyratite.hit_effect = "blastExplosion".into();
        pyratite.knockback = 0.8;
        pyratite.lifetime = 80.0;
        pyratite.width = 13.0;
        pyratite.height = 13.0;
        pyratite.splash_damage_radius = 25.0 * 0.75;
        pyratite.splash_damage = 45.0;
        pyratite.status = "burning".into();
        pyratite.status_duration = 60.0 * 12.0;
        pyratite.front_color = "lightishOrange".into();
        pyratite.trail_color = "lightishOrange".into();
        pyratite.hit_color = "lightishOrange".into();
        pyratite.back_color = "lightOrange".into();
        pyratite.make_fire = true;
        pyratite.trail_effect = "incendTrail".into();
        pyratite.ammo_multiplier = 4.0;
        pyratite.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "pyratite", pyratite);

        turret.target_air = false;
        turret.reload = 60.0;
        turret.recoil = 2.0;
        turret.range = 235.0;
        turret.inaccuracy = 1.0;
        turret.shoot_cone = 10.0;
        turret.base.health = 260;
        turret.shoot_sound = "shootArtillerySmall".into();
        turret.consume_coolant(0.1);
        turret.coolant_multiplier = 10.0;
        turret.deposit_cooldown = 2.0;
        turret.limit_range(0.0);
    });

    registry.register_turret_block("wave", TurretBlockKind::LiquidTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("metaglass", 45), ("lead", 75), ("copper", 25)],
        );

        let mut water = liquid_bullet(liquids, "water");
        water.knockback = 0.7;
        water.drag = 0.01;
        water.layer = "Layer.bullet-2".into();
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "water", water);

        let mut slag = liquid_bullet(liquids, "slag");
        slag.damage = 4.0;
        slag.drag = 0.01;
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "slag", slag);

        let mut cryofluid = liquid_bullet(liquids, "cryofluid");
        cryofluid.drag = 0.01;
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "cryofluid", cryofluid);

        let mut oil = liquid_bullet(liquids, "oil");
        oil.drag = 0.01;
        oil.layer = "Layer.bullet-2".into();
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "oil", oil);

        turret.base.size = 2;
        turret.recoil = 0.0;
        turret.reload = 3.0;
        turret.inaccuracy = 5.0;
        turret.shoot_cone = 50.0;
        turret.liquid_capacity = 10.0;
        turret.shoot_effect = "shootLiquid".into();
        turret.range = 110.0;
        turret.scaled_health = 250.0;
        turret.base.flags.clear();
        turret.base.flags.push(BlockFlag::Turret);
        turret.base.flags.push(BlockFlag::Extinguisher);
    });

    registry.register_turret_block("lancer", TurretBlockKind::PowerTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("copper", 60),
                ("lead", 70),
                ("silicon", 60),
                ("titanium", 30),
            ],
        );
        turret.range = 165.0;
        turret.shoot_first_shot_delay = 40.0;
        turret.recoil = 2.0;
        turret.reload = 80.0;
        turret.shake = 2.0;
        turret.shoot_effect = "lancerLaserShoot".into();
        turret.smoke_effect = "none".into();
        turret.heat_color = "red".into();
        turret.base.size = 2;
        turret.scaled_health = 280.0;
        turret.target_air = false;
        turret.move_while_charging = false;
        turret.accurate_delay = false;
        turret.shoot_sound = "shootLancer".into();
        turret.consume_coolant(0.2);
        turret.charge_sound = "chargeLancer".into();
        turret.consume_power = 6.0;

        let mut shoot_type = BulletSpec::new(BulletKind::Laser, 0.0, 140.0);
        shoot_type.colors = vec!["a9d8ff66".into(), "a9d8ffff".into(), "ffffffff".into()];
        shoot_type.charge_effect = "MultiEffect(lancerLaserCharge, lancerLaserChargeBegin)".into();
        shoot_type.building_damage_multiplier = 0.25;
        shoot_type.armor_multiplier = 4.0;
        shoot_type.hit_effect = "hitLancer".into();
        shoot_type.hit_size = 4.0;
        shoot_type.lifetime = 16.0;
        shoot_type.draw_size = 400.0;
        shoot_type.collides_air = false;
        shoot_type.length = 173.0;
        shoot_type.ammo_multiplier = 1.0;
        shoot_type.pierce_cap = 4;
        shoot_type.despawn_effect = "none".into();
        shoot_type.shoot_effect = "hitLancer".into();
        shoot_type.smoke_effect = "none".into();
        shoot_type.hit_color = "ffffffff".into();
        shoot_type.keep_velocity = false;
        shoot_type.collides = false;
        shoot_type.pierce = true;
        shoot_type.hittable = false;
        shoot_type.absorbable = false;
        shoot_type.length = 173.0;
        turret.shoot_type = Some(Box::new(shoot_type));
    });

    registry.register_turret_block("arc", TurretBlockKind::PowerTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 50), ("lead", 50)],
        );

        let mut lightning_type = BulletSpec::new(BulletKind::Generic, 0.0001, 0.0);
        lightning_type.lifetime = 10.0;
        lightning_type.hit_effect = "hitLancer".into();
        lightning_type.despawn_effect = "none".into();
        lightning_type.status = "shocked".into();
        lightning_type.hittable = false;
        lightning_type.light_color = "ffffffff".into();
        lightning_type.collides_air = false;
        lightning_type.building_damage_multiplier = 0.25;
        lightning_type.shield_damage_multiplier = 0.2;

        let mut shoot_type = BulletSpec::new(BulletKind::Lightning, 0.0, 20.0);
        shoot_type.lifetime = 1.0;
        shoot_type.despawn_effect = "none".into();
        shoot_type.hit_effect = "hitLancer".into();
        shoot_type.keep_velocity = false;
        shoot_type.hittable = false;
        shoot_type.status = "shocked".into();
        shoot_type.lightning_length = 25;
        shoot_type.lightning_length_rand = 0;
        shoot_type.lightning_color = "a9d8ffff".into();
        shoot_type.collides_air = false;
        shoot_type.ammo_multiplier = 1.0;
        shoot_type.building_damage_multiplier = 0.25;
        shoot_type.lightning_type = Some(Box::new(lightning_type));
        turret.shoot_type = Some(Box::new(shoot_type));

        turret.research_cost_multiplier = 1.0 / 3.0;
        turret.reload = 35.0;
        turret.shoot_cone = 40.0;
        turret.rotate_speed = 8.0;
        turret.target_air = false;
        turret.range = 90.0;
        turret.shoot_effect = "lightningShoot".into();
        turret.heat_color = "red".into();
        turret.recoil = 1.0;
        turret.base.size = 1;
        turret.base.health = 260;
        turret.shoot_sound = "shootArc".into();
        turret.consume_power = 3.3;
        turret.consume_coolant(0.1);
    });

    registry.register_turret_block("parallax", TurretBlockKind::TractorBeamTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("silicon", 160), ("titanium", 110), ("graphite", 50)],
        );

        turret.base.size = 2;
        turret.force = 16.0;
        turret.scaled_force = 9.0;
        turret.range = 300.0;
        turret.damage = 0.5;
        turret.scaled_health = 160.0;
        turret.rotate_speed = 12.0;
        turret.consume_power = 3.3;
    });

    registry.register_turret_block("swarmer", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("graphite", 35),
                ("titanium", 35),
                ("plastanium", 45),
                ("silicon", 30),
            ],
        );

        let mut blast = missile_bullet(3.7, 10.0);
        blast.width = 8.0;
        blast.height = 8.0;
        blast.shrink_y = 0.0;
        blast.splash_damage_radius = 30.0;
        blast.splash_damage = 30.0 * 1.5;
        blast.ammo_multiplier = 5.0;
        blast.hit_effect = "blastExplosion".into();
        blast.despawn_effect = "blastExplosion".into();
        blast.status = "blasted".into();
        blast.hit_color = "blastAmmoBack".into();
        blast.back_color = "blastAmmoBack".into();
        blast.trail_color = "blastAmmoBack".into();
        blast.front_color = "blastAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "blast-compound", blast);

        let mut pyratite = missile_bullet(3.7, 12.0);
        pyratite.front_color = "lightishOrange".into();
        pyratite.back_color = "lightOrange".into();
        pyratite.width = 7.0;
        pyratite.height = 8.0;
        pyratite.shrink_y = 0.0;
        pyratite.homing_power = 0.08;
        pyratite.splash_damage_radius = 20.0;
        pyratite.splash_damage = 30.0 * 1.5;
        pyratite.make_fire = true;
        pyratite.ammo_multiplier = 5.0;
        pyratite.hit_effect = "blastExplosion".into();
        pyratite.status = "burning".into();
        push_turret_ammo(&mut turret.ammo, items, "pyratite", pyratite);

        let mut surge = missile_bullet(3.7, 18.0);
        surge.width = 8.0;
        surge.height = 8.0;
        surge.shrink_y = 0.0;
        surge.splash_damage_radius = 25.0;
        surge.splash_damage = 25.0 * 1.4;
        surge.hit_effect = "blastExplosion".into();
        surge.despawn_effect = "blastExplosion".into();
        surge.ammo_multiplier = 4.0;
        surge.lightning_damage = 10.0;
        surge.lightning = 2;
        surge.lightning_length = 10;
        surge.hit_color = "surgeAmmoBack".into();
        surge.back_color = "surgeAmmoBack".into();
        surge.trail_color = "surgeAmmoBack".into();
        surge.front_color = "surgeAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "surge-alloy", surge);

        turret.shoot_pattern = "ShootBarrel".into();
        turret.shoot_barrels =
            shoot_barrels(&[(-4.0, -1.25, 0.0), (0.0, 0.0, 0.0), (4.0, -1.25, 0.0)]);
        turret.shoot_shots = 4;
        turret.shoot_shot_delay = 5.0;
        turret.shoot_y = 4.5;
        turret.reload = 60.0 * 4.0 / 7.0;
        turret.inaccuracy = 10.0;
        turret.range = 240.0;
        turret.consume_ammo_once = false;
        turret.base.size = 2;
        turret.scaled_health = 300.0;
        turret.shoot_sound = "shootMissile".into();
        turret.base.env_enabled |= Env::SPACE;
        turret.limit_range(5.0);
        turret.consume_coolant(0.3);
        turret.deposit_cooldown = 2.0;
    });

    registry.register_turret_block("salvo", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 100), ("graphite", 80), ("titanium", 50)],
        );

        let mut copper = BulletSpec::new(BulletKind::Basic, 2.5, 15.0);
        copper.width = 7.0;
        copper.height = 9.0;
        copper.lifetime = 60.0;
        copper.ammo_multiplier = 5.0;
        copper.armor_multiplier = 1.5;
        copper.hit_effect = "hitBulletColor".into();
        copper.despawn_effect = "hitBulletColor".into();
        copper.hit_color = "copperAmmoBack".into();
        copper.back_color = "copperAmmoBack".into();
        copper.trail_color = "copperAmmoBack".into();
        copper.front_color = "copperAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "copper", copper);

        let mut graphite = BulletSpec::new(BulletKind::Basic, 3.5, 31.0);
        graphite.width = 9.0;
        graphite.height = 12.0;
        graphite.ammo_multiplier = 4.0;
        graphite.lifetime = 60.0;
        graphite.reload_multiplier = 0.8;
        graphite.range_change = 4.0 * 8.0;
        graphite.hit_effect = "hitBulletColor".into();
        graphite.despawn_effect = "hitBulletColor".into();
        graphite.hit_color = "graphiteAmmoBack".into();
        graphite.back_color = "graphiteAmmoBack".into();
        graphite.trail_color = "graphiteAmmoBack".into();
        graphite.front_color = "graphiteAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "graphite", graphite);

        let mut pyratite = BulletSpec::new(BulletKind::Basic, 3.2, 25.0);
        pyratite.width = 10.0;
        pyratite.height = 12.0;
        pyratite.front_color = "lightishOrange".into();
        pyratite.hit_color = "lightishOrange".into();
        pyratite.back_color = "lightOrange".into();
        pyratite.status = "burning".into();
        pyratite.hit_effect = "MultiEffect(hitBulletColor, fireHit)".into();
        pyratite.ammo_multiplier = 5.0;
        pyratite.splash_damage = 15.0;
        pyratite.splash_damage_radius = 22.0;
        pyratite.make_fire = true;
        pyratite.lifetime = 60.0;
        push_turret_ammo(&mut turret.ammo, items, "pyratite", pyratite);

        let mut silicon = BulletSpec::new(BulletKind::Basic, 3.0, 23.0);
        silicon.width = 8.0;
        silicon.height = 10.0;
        silicon.homing_power = 0.2;
        silicon.reload_multiplier = 1.5;
        silicon.ammo_multiplier = 5.0;
        silicon.lifetime = 60.0;
        silicon.trail_length = 5;
        silicon.trail_width = 1.5;
        silicon.hit_effect = "hitBulletColor".into();
        silicon.despawn_effect = "hitBulletColor".into();
        silicon.hit_color = "siliconAmmoBack".into();
        silicon.back_color = "siliconAmmoBack".into();
        silicon.trail_color = "siliconAmmoBack".into();
        silicon.front_color = "siliconAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "silicon", silicon);

        let mut thorium = BulletSpec::new(BulletKind::Basic, 4.0, 28.0);
        thorium.width = 8.0;
        thorium.height = 13.0;
        thorium.shoot_effect = "shootBig".into();
        thorium.smoke_effect = "shootBigSmoke".into();
        thorium.ammo_multiplier = 4.0;
        thorium.lifetime = 60.0;
        thorium.armor_multiplier = 0.8;
        thorium.hit_effect = "hitBulletColor".into();
        thorium.despawn_effect = "hitBulletColor".into();
        thorium.back_color = "thoriumAmmoBack".into();
        thorium.hit_color = "thoriumAmmoBack".into();
        thorium.trail_color = "thoriumAmmoBack".into();
        thorium.front_color = "thoriumAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "thorium", thorium);

        turret.drawer = "DrawTurret(RegionPart(-side warmup mirror layerOffset=0.001 moveX=0.6 moveRot=-15 PartMove(recoil,0.5,-0.5,-8)), RegionPart(-barrel recoil moveY=-2.5))".into();
        turret.base.size = 2;
        turret.range = 190.0;
        turret.reload = 29.0;
        turret.consume_ammo_once = false;
        turret.ammo_eject_back = 3.0;
        turret.recoil = 0.0;
        turret.shake = 1.0;
        turret.shoot_pattern = "ShootPattern".into();
        turret.shoot_shots = 4;
        turret.shoot_shot_delay = 3.0;
        turret.ammo_use_effect = "casing2".into();
        turret.scaled_health = 240.0;
        turret.shoot_sound = "shootSalvo".into();
        turret.limit_range(9.0);
        turret.consume_coolant(0.2);
        turret.deposit_cooldown = 2.0;
    });

    registry.register_turret_block("segment", TurretBlockKind::PointDefenseTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("silicon", 130),
                ("thorium", 80),
                ("phase-fabric", 40),
                ("titanium", 40),
            ],
        );

        turret.scaled_health = 250.0;
        turret.range = 180.0;
        turret.consume_power = 8.0;
        turret.base.size = 2;
        turret.shoot_length = 5.0;
        turret.bullet_damage = 30.0;
        turret.reload = 8.0;
        turret.base.env_enabled |= Env::SPACE;
    });

    registry.register_turret_block("tsunami", TurretBlockKind::LiquidTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("metaglass", 100),
                ("lead", 400),
                ("titanium", 250),
                ("thorium", 100),
            ],
        );

        let mut water = liquid_bullet(liquids, "water");
        water.lifetime = 49.0;
        water.speed = 4.0;
        water.knockback = 1.7;
        water.puddle_size = 8.0;
        water.orb_size = 4.0;
        water.drag = 0.001;
        water.ammo_multiplier = 0.4;
        water.status_duration = 60.0 * 4.0;
        water.damage = 0.2;
        water.layer = "Layer.bullet-2".into();
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "water", water);

        let mut slag = liquid_bullet(liquids, "slag");
        slag.lifetime = 49.0;
        slag.speed = 4.0;
        slag.knockback = 1.3;
        slag.puddle_size = 8.0;
        slag.orb_size = 4.0;
        slag.damage = 4.75;
        slag.drag = 0.001;
        slag.ammo_multiplier = 0.4;
        slag.status_duration = 60.0 * 4.0;
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "slag", slag);

        let mut cryofluid = liquid_bullet(liquids, "cryofluid");
        cryofluid.lifetime = 49.0;
        cryofluid.speed = 4.0;
        cryofluid.knockback = 1.3;
        cryofluid.puddle_size = 8.0;
        cryofluid.orb_size = 4.0;
        cryofluid.drag = 0.001;
        cryofluid.ammo_multiplier = 0.4;
        cryofluid.status_duration = 60.0 * 4.0;
        cryofluid.damage = 0.2;
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "cryofluid", cryofluid);

        let mut oil = liquid_bullet(liquids, "oil");
        oil.lifetime = 49.0;
        oil.speed = 4.0;
        oil.knockback = 1.3;
        oil.puddle_size = 8.0;
        oil.orb_size = 4.0;
        oil.drag = 0.001;
        oil.ammo_multiplier = 0.4;
        oil.status_duration = 60.0 * 4.0;
        oil.damage = 0.2;
        oil.layer = "Layer.bullet-2".into();
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "oil", oil);

        turret.base.size = 3;
        turret.reload = 3.0;
        turret.shoot_pattern = "ShootAlternate".into();
        turret.shoot_alternate_spread = 4.0;
        turret.shoot_shots = 2;
        turret.velocity_rnd = 0.1;
        turret.inaccuracy = 3.0;
        turret.recoil = 1.0;
        turret.shoot_cone = 45.0;
        turret.liquid_capacity = 40.0;
        turret.shoot_effect = "shootLiquid".into();
        turret.range = 190.0;
        turret.scaled_health = 250.0;
        turret.base.flags.clear();
        turret.base.flags.push(BlockFlag::Turret);
        turret.base.flags.push(BlockFlag::Extinguisher);
    });

    registry.register_turret_block("fuse", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 225), ("graphite", 225), ("thorium", 100)],
        );

        turret.reload = 35.0;
        turret.shake = 4.0;
        turret.range = 90.0;
        turret.recoil = 5.0;
        turret.shoot_pattern = "ShootSpread".into();
        turret.shoot_shots = 3;
        turret.shoot_spread = 20.0;
        turret.shoot_cone = 30.0;
        turret.base.size = 3;
        turret.base.env_enabled |= Env::SPACE;
        turret.scaled_health = 220.0;
        turret.shoot_sound = "shootFuse".into();
        turret.shoot_sound_volume = 0.9;
        turret.consume_coolant(0.3);

        let brange = turret.range + 10.0;

        let mut titanium = shrapnel_bullet();
        titanium.length = brange;
        titanium.damage = 66.0;
        titanium.ammo_multiplier = 4.0;
        titanium.width = 17.0;
        titanium.reload_multiplier = 1.3;
        push_turret_ammo(&mut turret.ammo, items, "titanium", titanium);

        let mut thorium = shrapnel_bullet();
        thorium.length = brange;
        thorium.damage = 105.0;
        thorium.ammo_multiplier = 5.0;
        thorium.to_color = "thoriumPink".into();
        thorium.shoot_effect = "thoriumShoot".into();
        thorium.smoke_effect = "thoriumShoot".into();
        push_turret_ammo(&mut turret.ammo, items, "thorium", thorium);

        turret.deposit_cooldown = 1.0;
    });

    registry.register_turret_block("ripple", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 150), ("graphite", 135), ("titanium", 60)],
        );

        let mut graphite = artillery_bullet(3.0, 40.0);
        graphite.hit_effect = "MultiEffect(flakExplosion, shockwaveSmaller)".into();
        graphite.knockback = 0.8;
        graphite.lifetime = 80.0;
        graphite.width = 12.0;
        graphite.height = 14.0;
        graphite.splash_damage_radius = 30.0 * 0.75;
        graphite.splash_damage = 70.0;
        graphite.back_color = "graphiteAmmoBack".into();
        graphite.hit_color = "graphiteAmmoBack".into();
        graphite.trail_color = "graphiteAmmoBack".into();
        graphite.front_color = "graphiteAmmoFront".into();
        graphite.despawn_effect = "hitBulletColor".into();
        graphite.life_scale_rand_max = 1.08;
        graphite.life_scale_rand_min = 0.95;
        push_turret_ammo(&mut turret.ammo, items, "graphite", graphite);

        let mut silicon = artillery_bullet(3.0, 40.0);
        silicon.hit_effect = "MultiEffect(flakExplosion, shockwaveSmaller)".into();
        silicon.knockback = 0.8;
        silicon.lifetime = 80.0;
        silicon.width = 12.0;
        silicon.height = 14.0;
        silicon.splash_damage_radius = 30.0 * 0.75;
        silicon.splash_damage = 70.0;
        silicon.reload_multiplier = 1.2;
        silicon.ammo_multiplier = 3.0;
        silicon.homing_power = 0.08;
        silicon.homing_range = 50.0;
        silicon.trail_length = 9;
        silicon.trail_width = 3.1;
        silicon.despawn_effect = "hitBulletColor".into();
        silicon.back_color = "siliconAmmoBack".into();
        silicon.hit_color = "siliconAmmoBack".into();
        silicon.trail_color = "siliconAmmoBack".into();
        silicon.front_color = "siliconAmmoFront".into();
        silicon.life_scale_rand_max = 1.08;
        silicon.life_scale_rand_min = 0.95;
        push_turret_ammo(&mut turret.ammo, items, "silicon", silicon);

        let mut pyratite = artillery_bullet(3.0, 48.0);
        pyratite.hit_effect = "MultiEffect(blastExplosion, shockwave)".into();
        pyratite.knockback = 0.8;
        pyratite.lifetime = 80.0;
        pyratite.width = 13.0;
        pyratite.height = 15.0;
        pyratite.splash_damage_radius = 30.0 * 0.75;
        pyratite.splash_damage = 90.0;
        pyratite.status = "burning".into();
        pyratite.status_duration = 60.0 * 12.0;
        pyratite.front_color = "lightishOrange".into();
        pyratite.back_color = "lightOrange".into();
        pyratite.hit_color = "lightOrange".into();
        pyratite.make_fire = true;
        pyratite.trail_effect = "incendTrail".into();
        pyratite.ammo_multiplier = 4.0;
        pyratite.despawn_effect = "hitBulletColor".into();
        pyratite.life_scale_rand_max = 1.08;
        pyratite.life_scale_rand_min = 0.95;
        push_turret_ammo(&mut turret.ammo, items, "pyratite", pyratite);

        let mut blast = artillery_bullet(2.0, 40.0);
        blast.hit_effect = "MultiEffect(blastExplosion, shockwave)".into();
        blast.knockback = 0.8;
        blast.lifetime = 80.0;
        blast.width = 14.0;
        blast.height = 16.0;
        blast.ammo_multiplier = 4.0;
        blast.splash_damage_radius = 50.0 * 0.75;
        blast.splash_damage = 90.0;
        blast.status = "blasted".into();
        blast.life_scale_rand_max = 1.08;
        blast.life_scale_rand_min = 0.95;
        blast.despawn_effect = "hitBulletColor".into();
        blast.back_color = "blastAmmoBack".into();
        blast.hit_color = "blastAmmoBack".into();
        blast.trail_color = "blastAmmoBack".into();
        blast.front_color = "blastAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "blast-compound", blast);

        let mut plastanium = artillery_bullet(3.4, 40.0);
        plastanium.hit_effect = "MultiEffect(plasticExplosion, shockwave)".into();
        plastanium.knockback = 1.0;
        plastanium.lifetime = 80.0;
        plastanium.width = 13.0;
        plastanium.height = 15.0;
        plastanium.splash_damage_radius = 40.0 * 0.75;
        plastanium.splash_damage = 90.0;
        let mut frag = BulletSpec::new(BulletKind::Basic, 2.5, 14.0);
        frag.width = 10.0;
        frag.height = 12.0;
        frag.shrink_y = 1.0;
        frag.lifetime = 15.0;
        frag.back_color = "plastaniumBack".into();
        frag.front_color = "plastaniumFront".into();
        frag.despawn_effect = "none".into();
        frag.collides_air = false;
        plastanium.frag_bullet = Some(Box::new(frag));
        plastanium.frag_bullets = 15;
        plastanium.back_color = "plastaniumBack".into();
        plastanium.front_color = "plastaniumFront".into();
        plastanium.life_scale_rand_max = 1.08;
        plastanium.life_scale_rand_min = 0.95;
        push_turret_ammo(&mut turret.ammo, items, "plastanium", plastanium);

        turret.target_air = false;
        turret.base.size = 3;
        turret.shoot_shots = 4;
        turret.inaccuracy = 11.0;
        turret.reload = 120.0;
        turret.ammo_eject_back = 5.0;
        turret.ammo_use_effect = "casing3Double".into();
        turret.ammo_per_shot = 2;
        turret.velocity_rnd = 0.2;
        turret.scale_lifetime_offset = 1.0 / 9.0;
        turret.recoil = 6.0;
        turret.shake = 2.0;
        turret.range = 290.0;
        turret.min_range = 50.0;
        turret.consume_coolant(0.3);
        turret.scaled_health = 130.0;
        turret.deposit_cooldown = 2.0;
        turret.shoot_sound = "shootRipple".into();
    });

    registry.register_turret_block("cyclone", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("copper", 200), ("titanium", 125), ("plastanium", 80)],
        );

        let mut metaglass_frag = BulletSpec::new(BulletKind::Basic, 3.0, 12.0);
        metaglass_frag.width = 5.0;
        metaglass_frag.height = 12.0;
        metaglass_frag.shrink_y = 1.0;
        metaglass_frag.lifetime = 20.0;
        metaglass_frag.back_color = "gray".into();
        metaglass_frag.front_color = "white".into();
        metaglass_frag.despawn_effect = "none".into();

        let mut metaglass = flak_bullet(4.0, 6.0);
        metaglass.ammo_multiplier = 2.0;
        metaglass.shoot_effect = "shootSmall".into();
        metaglass.reload_multiplier = 0.8;
        metaglass.width = 6.0;
        metaglass.height = 11.0;
        metaglass.hit_effect = "flakExplosion".into();
        metaglass.splash_damage = 45.0;
        metaglass.splash_damage_radius = 25.0;
        metaglass.frag_bullet = Some(Box::new(metaglass_frag));
        metaglass.frag_bullets = 4;
        metaglass.explode_range = 20.0;
        metaglass.collides_ground = true;
        metaglass.back_color = "glassAmmoBack".into();
        metaglass.hit_color = "glassAmmoBack".into();
        metaglass.trail_color = "glassAmmoBack".into();
        metaglass.front_color = "glassAmmoFront".into();
        metaglass.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "metaglass", metaglass);

        let mut blast = flak_bullet(4.0, 8.0);
        blast.shoot_effect = "shootBig".into();
        blast.ammo_multiplier = 5.0;
        blast.splash_damage = 45.0;
        blast.splash_damage_radius = 60.0;
        blast.collides_ground = true;
        blast.status = "blasted".into();
        blast.back_color = "blastAmmoBack".into();
        blast.hit_color = "blastAmmoBack".into();
        blast.trail_color = "blastAmmoBack".into();
        blast.front_color = "blastAmmoFront".into();
        blast.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "blast-compound", blast);

        let mut plastanium_frag = BulletSpec::new(BulletKind::Basic, 2.5, 12.0);
        plastanium_frag.width = 10.0;
        plastanium_frag.height = 12.0;
        plastanium_frag.shrink_y = 1.0;
        plastanium_frag.lifetime = 15.0;
        plastanium_frag.back_color = "plastaniumBack".into();
        plastanium_frag.front_color = "plastaniumFront".into();
        plastanium_frag.despawn_effect = "none".into();

        let mut plastanium = flak_bullet(4.0, 8.0);
        plastanium.ammo_multiplier = 4.0;
        plastanium.splash_damage_radius = 40.0;
        plastanium.splash_damage = 37.5;
        plastanium.frag_bullet = Some(Box::new(plastanium_frag));
        plastanium.frag_bullets = 6;
        plastanium.hit_effect = "plasticExplosion".into();
        plastanium.front_color = "plastaniumFront".into();
        plastanium.back_color = "plastaniumBack".into();
        plastanium.shoot_effect = "shootBig".into();
        plastanium.collides_ground = true;
        plastanium.explode_range = 20.0;
        plastanium.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "plastanium", plastanium);

        let mut surge = flak_bullet(4.5, 13.0);
        surge.ammo_multiplier = 5.0;
        surge.splash_damage = 50.0 * 1.5;
        surge.splash_damage_radius = 38.0;
        surge.lightning = 2;
        surge.lightning_length = 7;
        surge.shoot_effect = "shootBig".into();
        surge.collides_ground = true;
        surge.explode_range = 20.0;
        surge.back_color = "surgeAmmoBack".into();
        surge.hit_color = "surgeAmmoBack".into();
        surge.trail_color = "surgeAmmoBack".into();
        surge.front_color = "surgeAmmoFront".into();
        surge.despawn_effect = "hitBulletColor".into();
        push_turret_ammo(&mut turret.ammo, items, "surge-alloy", surge);

        turret.shoot_y = 10.0;
        turret.shoot_pattern = "ShootBarrel".into();
        turret.shoot_barrels = shoot_barrels(&[(0.0, 1.0, 0.0), (3.0, 0.0, 0.0), (-3.0, 0.0, 0.0)]);
        turret.recoils = 3;
        turret.drawer = "DrawTurret(RegionPart(-barrel-3 recoilIndex=2 under moveY=-2), RegionPart(-barrel-2 recoilIndex=1 under moveY=-2), RegionPart(-barrel-1 recoilIndex=0 under moveY=-2))".into();
        turret.reload = 10.0;
        turret.range = 200.0;
        turret.base.size = 3;
        turret.recoil = 1.5;
        turret.recoil_time = 10.0;
        turret.rotate_speed = 10.0;
        turret.inaccuracy = 10.0;
        turret.shoot_cone = 30.0;
        turret.shoot_sound = "shootCyclone".into();
        turret.consume_coolant(0.3);
        turret.scaled_health = 145.0;
        turret.deposit_cooldown = 2.0;
        turret.limit_range(9.0);
    });

    registry.register_turret_block("foreshadow", TurretBlockKind::ItemTurret, |turret| {
        let brange = 500.0;
        turret.range = brange;

        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("copper", 1000),
                ("metaglass", 600),
                ("surge-alloy", 300),
                ("plastanium", 200),
                ("silicon", 600),
            ],
        );

        let mut surge = rail_bullet();
        surge.shoot_effect = "instShoot".into();
        surge.hit_effect = "instHit".into();
        surge.pierce_effect = "railHit".into();
        surge.smoke_effect = "smokeCloud".into();
        surge.point_effect = "instTrail".into();
        surge.despawn_effect = "instBomb".into();
        surge.point_effect_space = 20.0;
        surge.damage = 1350.0;
        surge.building_damage_multiplier = 0.2;
        surge.pierce_damage_factor = 1.0;
        surge.length = brange;
        surge.hit_shake = 6.0;
        surge.ammo_multiplier = 1.0;
        push_turret_ammo(&mut turret.ammo, items, "surge-alloy", surge);

        turret.max_ammo = 40;
        turret.ammo_per_shot = 5;
        turret.rotate_speed = 2.0;
        turret.reload = 200.0;
        turret.ammo_use_effect = "casing3Double".into();
        turret.recoil = 5.0;
        turret.cooldown_time = turret.reload;
        turret.shake = 4.0;
        turret.base.size = 4;
        turret.shoot_cone = 2.0;
        turret.shoot_sound = "shootForeshadow".into();
        turret.unit_sort = "strongest".into();
        turret.base.env_enabled |= Env::SPACE;
        turret.coolant_multiplier = 0.4;
        turret.liquid_capacity = 60.0;
        turret.scaled_health = 150.0;
        turret.consume_coolant(1.0);
        turret.deposit_cooldown = 2.0;
        turret.consume_power = 10.0;
    });

    registry.register_turret_block("spectre", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("copper", 900),
                ("graphite", 300),
                ("surge-alloy", 250),
                ("plastanium", 175),
                ("thorium", 250),
            ],
        );

        let mut graphite = basic_bullet(7.5, 50.0);
        graphite.hit_size = 4.8;
        graphite.width = 15.0;
        graphite.height = 21.0;
        graphite.shoot_effect = "shootBig".into();
        graphite.ammo_multiplier = 4.0;
        graphite.reload_multiplier = 1.7;
        graphite.knockback = 0.3;
        graphite.hit_effect = "hitBulletColor".into();
        graphite.despawn_effect = "hitBulletColor".into();
        graphite.hit_color = "graphiteAmmoBack".into();
        graphite.back_color = "graphiteAmmoBack".into();
        graphite.trail_color = "graphiteAmmoBack".into();
        graphite.front_color = "graphiteAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "graphite", graphite);

        let mut thorium = basic_bullet(8.0, 80.0);
        thorium.hit_size = 5.0;
        thorium.width = 16.0;
        thorium.height = 23.0;
        thorium.shoot_effect = "shootBig".into();
        thorium.pierce_cap = 2;
        thorium.pierce_building = true;
        thorium.knockback = 0.7;
        thorium.back_color = "thoriumAmmoBack".into();
        thorium.hit_color = "thoriumAmmoBack".into();
        thorium.trail_color = "thoriumAmmoBack".into();
        thorium.front_color = "thoriumAmmoFront".into();
        push_turret_ammo(&mut turret.ammo, items, "thorium", thorium);

        let mut pyratite = basic_bullet(7.0, 70.0);
        pyratite.hit_size = 5.0;
        pyratite.width = 16.0;
        pyratite.height = 21.0;
        pyratite.front_color = "lightishOrange".into();
        pyratite.back_color = "lightOrange".into();
        pyratite.status = "burning".into();
        pyratite.hit_effect = "MultiEffect(hitBulletSmall, fireHit)".into();
        pyratite.shoot_effect = "shootBig".into();
        pyratite.make_fire = true;
        pyratite.pierce_cap = 2;
        pyratite.pierce_building = true;
        pyratite.knockback = 0.6;
        pyratite.ammo_multiplier = 3.0;
        pyratite.splash_damage = 20.0;
        pyratite.splash_damage_radius = 25.0;
        push_turret_ammo(&mut turret.ammo, items, "pyratite", pyratite);

        turret.reload = 7.0;
        turret.recoil_time = turret.reload * 2.0;
        turret.coolant_multiplier = 0.5;
        turret.liquid_capacity = 120.0;
        turret.ammo_use_effect = "casing3".into();
        turret.range = 260.0;
        turret.inaccuracy = 3.0;
        turret.recoil = 3.0;
        turret.shoot_pattern = "ShootAlternate".into();
        turret.shoot_alternate_spread = 8.0;
        turret.shake = 2.0;
        turret.base.size = 4;
        turret.shoot_cone = 24.0;
        turret.shoot_sound = "shootSpectre".into();
        turret.scaled_health = 160.0;
        turret.consume_coolant(1.0);
        turret.deposit_cooldown = 2.0;
        turret.limit_range(9.0);
    });

    registry.register_turret_block("meltdown", TurretBlockKind::LaserTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("copper", 1200),
                ("lead", 350),
                ("graphite", 300),
                ("surge-alloy", 325),
                ("silicon", 325),
            ],
        );

        turret.shoot_effect = "shootBigSmoke2".into();
        turret.shoot_cone = 40.0;
        turret.recoil = 4.0;
        turret.base.size = 4;
        turret.shake = 2.0;
        turret.range = 195.0;
        turret.reload = 90.0;
        turret.firing_move_fract = 0.5;
        turret.shoot_duration = 230.0;
        turret.shoot_sound = "shootMeltdown".into();
        turret.loop_sound = "beamMeltdown".into();
        turret.loop_sound_volume = 2.0;
        turret.base.env_enabled |= Env::SPACE;

        let mut shoot_type = continuous_laser_bullet(78.0);
        shoot_type.length = 200.0;
        shoot_type.hit_effect = "hitMeltdown".into();
        shoot_type.hit_color = "meltdownHit".into();
        shoot_type.status = "melting".into();
        shoot_type.draw_size = 420.0;
        shoot_type.timescale_damage = true;
        shoot_type.incend_chance = 0.4;
        shoot_type.incend_spread = 5.0;
        shoot_type.incend_amount = 1;
        shoot_type.ammo_multiplier = 1.0;
        turret.shoot_type = Some(Box::new(shoot_type));

        turret.scaled_health = 200.0;
        turret.liquid_capacity = 60.0;
        turret.consume_coolant(0.5);
        turret.consume_power = 17.0;
    });

    registry.register_turret_block("breach", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("beryllium", 150), ("silicon", 150), ("graphite", 125)],
        );

        let shoot_effect = "MultiEffect(shootBigColor, colorSparkBig)";

        let mut beryllium = basic_bullet(7.5, 85.0);
        beryllium.width = 12.0;
        beryllium.hit_size = 7.0;
        beryllium.height = 20.0;
        beryllium.shoot_effect = shoot_effect.into();
        beryllium.smoke_effect = "shootBigSmoke".into();
        beryllium.ammo_multiplier = 1.0;
        beryllium.pierce_cap = 2;
        beryllium.pierce = true;
        beryllium.pierce_building = true;
        beryllium.hit_color = "berylShot".into();
        beryllium.back_color = "berylShot".into();
        beryllium.trail_color = "berylShot".into();
        beryllium.front_color = "white".into();
        beryllium.trail_width = 2.1;
        beryllium.trail_length = 10;
        beryllium.hit_effect = "hitBulletColor".into();
        beryllium.despawn_effect = "hitBulletColor".into();
        beryllium.building_damage_multiplier = 0.3;
        push_turret_ammo(&mut turret.ammo, items, "beryllium", beryllium);

        let mut tungsten = basic_bullet(8.0, 95.0);
        tungsten.width = 13.0;
        tungsten.height = 19.0;
        tungsten.hit_size = 7.0;
        tungsten.shoot_effect = shoot_effect.into();
        tungsten.smoke_effect = "shootBigSmoke".into();
        tungsten.ammo_multiplier = 2.0;
        tungsten.reload_multiplier = 1.0;
        tungsten.pierce_cap = 4;
        tungsten.pierce = true;
        tungsten.pierce_building = true;
        tungsten.hit_color = "tungstenShot".into();
        tungsten.back_color = "tungstenShot".into();
        tungsten.trail_color = "tungstenShot".into();
        tungsten.front_color = "white".into();
        tungsten.trail_width = 2.2;
        tungsten.trail_length = 11;
        tungsten.hit_effect = "hitBulletColor".into();
        tungsten.despawn_effect = "hitBulletColor".into();
        tungsten.range_change = 40.0;
        tungsten.building_damage_multiplier = 0.3;
        push_turret_ammo(&mut turret.ammo, items, "tungsten", tungsten);

        let mut carbide_frag = basic_bullet(8.1, 227.0);
        carbide_frag.lifetime = 8.0;
        carbide_frag.width = 11.0;
        carbide_frag.height = 14.0;
        carbide_frag.hit_size = 7.0;
        carbide_frag.shoot_effect = shoot_effect.into();
        carbide_frag.ammo_multiplier = 1.0;
        carbide_frag.reload_multiplier = 1.0;
        carbide_frag.pierce_cap = 2;
        carbide_frag.pierce = true;
        carbide_frag.pierce_building = true;
        carbide_frag.hit_color = "ab8ec5".into();
        carbide_frag.back_color = "ab8ec5".into();
        carbide_frag.trail_color = "ab8ec5".into();
        carbide_frag.front_color = "white".into();
        carbide_frag.trail_width = 1.8;
        carbide_frag.trail_length = 11;
        carbide_frag.hit_effect = "hitBulletColor".into();
        carbide_frag.despawn_effect = "hitBulletColor".into();
        carbide_frag.building_damage_multiplier = 0.2;

        let mut carbide = basic_bullet(12.0, 325.0 / 0.75);
        carbide.width = 15.0;
        carbide.height = 21.0;
        carbide.hit_size = 7.0;
        carbide.shoot_effect = shoot_effect.into();
        carbide.smoke_effect = "shootBigSmoke".into();
        carbide.ammo_multiplier = 2.0;
        carbide.reload_multiplier = 0.2;
        carbide.hit_color = "ab8ec5".into();
        carbide.back_color = "ab8ec5".into();
        carbide.trail_color = "ab8ec5".into();
        carbide.front_color = "white".into();
        carbide.trail_width = 2.2;
        carbide.trail_length = 11;
        carbide.trail_effect = "disperseTrail".into();
        carbide.trail_interval = 2.0;
        carbide.hit_effect = "hitBulletColor".into();
        carbide.despawn_effect = "hitBulletColor".into();
        carbide.range_change = 7.0 * 8.0;
        carbide.building_damage_multiplier = 0.3;
        carbide.trail_rotation = true;
        carbide.bullet_shoot_sound = "shootBreachCarbide".into();
        carbide.frag_bullets = 3;
        carbide.frag_random_spread = 0.0;
        carbide.frag_spread = 25.0;
        carbide.frag_velocity_min = 1.0;
        carbide.frag_bullet = Some(Box::new(carbide_frag));
        push_turret_ammo(&mut turret.ammo, items, "carbide", carbide);

        turret.coolant_multiplier = 15.0;
        turret.shoot_sound = "shootBreach".into();
        turret.target_under_blocks = false;
        turret.shake = 1.0;
        turret.ammo_per_shot = 2;
        turret.drawer = "DrawTurret(reinforced-)".into();
        turret.shoot_y = -2.0;
        turret.outline_color = "darkOutline".into();
        turret.base.size = 3;
        turret.base.env_enabled |= Env::SPACE;
        turret.reload = 40.0;
        turret.recoil = 2.0;
        turret.range = 190.0;
        turret.shoot_cone = 3.0;
        turret.scaled_health = 180.0;
        turret.rotate_speed = 1.5;
        turret.research_cost_multiplier = 0.05;
        turret.build_time = 60.0 * 9.0;
        turret.consume_coolant(15.0 / 60.0);
        turret.limit_range(12.0);
    });

    registry.register_turret_block("diffuse", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("beryllium", 150),
                ("silicon", 200),
                ("graphite", 200),
                ("tungsten", 50),
            ],
        );

        let mut graphite = basic_bullet(8.0, 41.0);
        graphite.knockback = 4.0;
        graphite.width = 25.0;
        graphite.hit_size = 7.0;
        graphite.height = 20.0;
        graphite.shoot_effect = "shootBigColor".into();
        graphite.smoke_effect = "shootSmokeSquareSparse".into();
        graphite.ammo_multiplier = 1.0;
        graphite.hit_color = "ea8878".into();
        graphite.back_color = "ea8878".into();
        graphite.trail_color = "ea8878".into();
        graphite.front_color = "redLight".into();
        graphite.trail_width = 6.0;
        graphite.trail_length = 3;
        graphite.hit_effect = "hitSquaresColor".into();
        graphite.despawn_effect = "hitSquaresColor".into();
        graphite.building_damage_multiplier = 0.2;
        push_turret_ammo(&mut turret.ammo, items, "graphite", graphite);

        let mut oxide = basic_bullet(8.0, 90.0);
        oxide.knockback = 3.0;
        oxide.width = 25.0;
        oxide.hit_size = 7.0;
        oxide.height = 20.0;
        oxide.shoot_effect = "shootBigColor".into();
        oxide.smoke_effect = "shootSmokeSquareSparse".into();
        oxide.ammo_multiplier = 2.0;
        oxide.hit_color = "a0b380".into();
        oxide.back_color = "a0b380".into();
        oxide.trail_color = "a0b380".into();
        oxide.front_color = "e4ffd6".into();
        oxide.trail_width = 6.0;
        oxide.trail_length = 3;
        oxide.hit_effect = "hitSquaresColor".into();
        oxide.despawn_effect = "hitSquaresColor".into();
        oxide.building_damage_multiplier = 0.2;
        push_turret_ammo(&mut turret.ammo, items, "oxide", oxide);

        let mut silicon = basic_bullet(8.0, 35.0);
        silicon.knockback = 3.0;
        silicon.width = 25.0;
        silicon.hit_size = 7.0;
        silicon.height = 20.0;
        silicon.homing_power = 0.045;
        silicon.shoot_effect = "shootBigColor".into();
        silicon.smoke_effect = "shootSmokeSquareSparse".into();
        silicon.ammo_multiplier = 1.0;
        silicon.hit_color = "siliconAmmoBack".into();
        silicon.back_color = "siliconAmmoBack".into();
        silicon.trail_color = "siliconAmmoBack".into();
        silicon.front_color = "dae1ee".into();
        silicon.trail_width = 6.0;
        silicon.trail_length = 6;
        silicon.hit_effect = "hitSquaresColor".into();
        silicon.despawn_effect = "hitSquaresColor".into();
        silicon.building_damage_multiplier = 0.2;
        push_turret_ammo(&mut turret.ammo, items, "silicon", silicon);

        turret.shoot_pattern = "ShootSpread".into();
        turret.shoot_shots = 15;
        turret.shoot_spread = 4.0;
        turret.coolant_multiplier = 15.0;
        turret.inaccuracy = 0.2;
        turret.velocity_rnd = 0.17;
        turret.shake = 1.0;
        turret.ammo_per_shot = 3;
        turret.max_ammo = 30;
        turret.consume_ammo_once = true;
        turret.target_under_blocks = false;
        turret.shoot_sound = "shootDiffuse".into();
        turret.drawer = "DrawTurret(reinforced-, RegionPart(-front warmup mirror moveRot=-10 PartMove(recoil,0,-3,-5) heatColor=red))".into();
        turret.shoot_y = 5.0;
        turret.outline_color = "darkOutline".into();
        turret.base.size = 3;
        turret.base.env_enabled |= Env::SPACE;
        turret.reload = 30.0;
        turret.recoil = 2.0;
        turret.range = 125.0;
        turret.shoot_cone = 40.0;
        turret.scaled_health = 210.0;
        turret.rotate_speed = 3.0;
        turret.consume_coolant(15.0 / 60.0);
        turret.limit_range(25.0);
    });

    registry.register_turret_block("sublimate", TurretBlockKind::ContinuousLiquidTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("tungsten", 150),
                ("silicon", 200),
                ("oxide", 40),
                ("beryllium", 400),
            ],
        );

        turret.drawer = "DrawTurret(reinforced-, heatColor=fa2859, RegionPart(-back warmup mirror moveRot=40 under), RegionPart(-front warmup mirror moveRot=40 under), RegionPart(-nozzle warmup mirror))".into();
        turret.outline_color = "darkOutline".into();
        turret.liquid_capacity = 50.0;
        turret.liquid_consumed = 18.0 / 60.0;
        turret.target_interval = 5.0;
        turret.new_target_interval = 30.0;
        turret.target_under_blocks = false;
        turret.shoot_y = 8.0;

        let r = 130.0;
        turret.range = r;
        turret.loop_sound = "shootSublimate".into();
        turret.shoot_sound = "none".into();
        turret.loop_sound_volume = 1.0;

        let mut ozone = continuous_flame_bullet(60.0);
        ozone.length = r;
        ozone.ammo_multiplier = 1.2;
        ozone.knockback = 1.0;
        ozone.pierce_cap = 2;
        ozone.building_damage_multiplier = 0.3;
        ozone.timescale_damage = true;
        ozone.colors = vec![
            "eb7abe88".into(),
            "e189f5b2".into(),
            "907ef7cc".into(),
            "91a4ff".into(),
            "white".into(),
        ];
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "ozone", ozone);

        let mut cyanogen = continuous_flame_bullet(130.0);
        cyanogen.range_change = 70.0;
        cyanogen.length = r + cyanogen.range_change;
        cyanogen.knockback = 2.0;
        cyanogen.pierce_cap = 3;
        cyanogen.building_damage_multiplier = 0.3;
        cyanogen.timescale_damage = true;
        cyanogen.colors = vec![
            "465ab888".into(),
            "66a6d2b2".into(),
            "89e8b6cc".into(),
            "cafcbe".into(),
            "white".into(),
        ];
        cyanogen.flare_color = "89e8b6".into();
        cyanogen.light_color = "89e8b6".into();
        cyanogen.hit_color = "89e8b6".into();
        push_liquid_turret_ammo(&mut turret.liquid_ammo, liquids, "cyanogen", cyanogen);

        turret.scaled_health = 210.0;
        turret.base.size = 3;
    });

    registry.register_turret_block("titan", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[("tungsten", 250), ("silicon", 300), ("thorium", 400)],
        );

        let mut thorium = artillery_bullet(2.5, 350.0);
        thorium.hit_effect = "MultiEffect(titanExplosion, titanSmoke)".into();
        thorium.despawn_effect = "none".into();
        thorium.knockback = 2.0;
        thorium.lifetime = 140.0;
        thorium.height = 19.0;
        thorium.width = 17.0;
        thorium.splash_damage_radius = 65.0;
        thorium.splash_damage = 350.0;
        thorium.scaled_splash_damage = true;
        thorium.back_color = "f49e7c".into();
        thorium.hit_color = "f49e7c".into();
        thorium.trail_color = "f49e7c".into();
        thorium.front_color = "white".into();
        thorium.ammo_multiplier = 1.0;
        thorium.hit_sound = "explosionTitan".into();
        thorium.status = "blasted".into();
        thorium.trail_length = 32;
        thorium.trail_width = 3.35;
        thorium.trail_sin_scl = 2.5;
        thorium.trail_sin_mag = 0.5;
        thorium.trail_effect = "none".into();
        thorium.despawn_shake = 7.0;
        thorium.shoot_effect = "shootTitan".into();
        thorium.smoke_effect = "shootSmokeTitan".into();
        thorium.trail_interp = "max(slope,0.8)".into();
        thorium.shrink_x = 0.2;
        thorium.shrink_y = 0.1;
        thorium.building_damage_multiplier = 0.3;
        push_turret_ammo(&mut turret.ammo, items, "thorium", thorium);

        let mut carbide_frag = artillery_bullet(0.5, 50.0);
        carbide_frag.hit_effect =
            "MultiEffect(titanExplosionFrag, titanLightSmall, WaveEffect(sizeTo=8))".into();
        carbide_frag.despawn_effect = "hitBulletColor".into();
        carbide_frag.width = 8.0;
        carbide_frag.height = 12.0;
        carbide_frag.lifetime = 50.0;
        carbide_frag.knockback = 0.5;
        carbide_frag.splash_damage_radius = 22.0;
        carbide_frag.splash_damage = 50.0;
        carbide_frag.scaled_splash_damage = true;
        carbide_frag.pierce_armor = true;
        carbide_frag.back_color = "ab8ec5".into();
        carbide_frag.hit_color = "ab8ec5".into();
        carbide_frag.front_color = "white".into();
        carbide_frag.building_damage_multiplier = 0.25;
        carbide_frag.shrink_y = 0.3;

        let mut carbide = artillery_bullet(3.25, 700.0);
        carbide.hit_effect = "MultiEffect(titanExplosionSmall, titanSmokeSmall)".into();
        carbide.despawn_effect = "none".into();
        carbide.knockback = 3.0;
        carbide.lifetime = 140.0;
        carbide.height = 28.0;
        carbide.width = 15.0;
        carbide.splash_damage_radius = 36.0;
        carbide.splash_damage = 750.0;
        carbide.range_change = 10.0 * 8.0;
        carbide.reload_multiplier = 0.8;
        carbide.scaled_splash_damage = true;
        carbide.back_color = "ab8ec5".into();
        carbide.hit_color = "ab8ec5".into();
        carbide.trail_color = "ab8ec5".into();
        carbide.front_color = "white".into();
        carbide.ammo_multiplier = 1.0;
        carbide.hit_sound = "explosionTitan".into();
        carbide.status = "blasted".into();
        carbide.trail_length = 32;
        carbide.trail_width = 3.35;
        carbide.trail_sin_scl = 2.5;
        carbide.trail_sin_mag = 0.5;
        carbide.trail_effect = "disperseTrail".into();
        carbide.trail_interval = 2.0;
        carbide.despawn_shake = 7.0;
        carbide.shoot_effect = "shootTitan".into();
        carbide.smoke_effect = "shootSmokeTitan".into();
        carbide.trail_rotation = true;
        carbide.trail_interp = "max(slope,0.8)".into();
        carbide.shrink_x = 0.2;
        carbide.shrink_y = 0.1;
        carbide.building_damage_multiplier = 0.2;
        carbide.frag_life_min = 1.5;
        carbide.frag_bullets = 12;
        carbide.frag_bullet = Some(Box::new(carbide_frag));
        push_turret_ammo(&mut turret.ammo, items, "carbide", carbide);

        let mut oxide_interval = BulletSpec::new(BulletKind::Generic, 0.0, 0.0);
        oxide_interval.splash_damage = 15.0;
        oxide_interval.collides_ground = true;
        oxide_interval.collides_air = false;
        oxide_interval.collides = false;
        oxide_interval.hit_effect = "none".into();
        oxide_interval.despawn_effect = "none".into();
        oxide_interval.pierce = true;
        oxide_interval.instant_disappear = true;
        oxide_interval.splash_damage_radius = 90.0;
        oxide_interval.building_damage_multiplier = 0.0;

        let mut oxide_frag = BulletSpec::new(BulletKind::Generic, 0.0, 0.0);
        oxide_frag.damage = 0.0;
        oxide_frag.lifetime = 60.0 * 2.5;
        oxide_frag.bullet_interval = 20.0;
        oxide_frag.hit_effect = "none".into();
        oxide_frag.despawn_effect = "none".into();
        oxide_frag.interval_bullet = Some(Box::new(oxide_interval));

        let mut oxide = artillery_bullet(2.5, 300.0);
        oxide.hit_effect = "MultiEffect(titanExplosionLarge, titanSmokeLarge, smokeAoeCloud)".into();
        oxide.despawn_effect = "none".into();
        oxide.knockback = 2.0;
        oxide.lifetime = 190.0;
        oxide.height = 19.0;
        oxide.width = 17.0;
        oxide.reload_multiplier = 0.7;
        oxide.splash_damage_radius = 110.0;
        oxide.range_change = 8.0;
        oxide.splash_damage = 180.0;
        oxide.scaled_splash_damage = true;
        oxide.hit_color = "a0b380".into();
        oxide.back_color = "a0b380".into();
        oxide.trail_color = "a0b380".into();
        oxide.front_color = "e4ffd6".into();
        oxide.ammo_multiplier = 1.0;
        oxide.hit_sound = "explosionTitan".into();
        oxide.trail_length = 32;
        oxide.trail_width = 3.35;
        oxide.trail_sin_scl = 2.5;
        oxide.trail_sin_mag = 0.5;
        oxide.trail_effect = "vapor".into();
        oxide.trail_interval = 3.0;
        oxide.despawn_shake = 7.0;
        oxide.shoot_effect = "shootTitan".into();
        oxide.smoke_effect = "shootSmokeTitan".into();
        oxide.trail_interp = "max(slope,0.8)".into();
        oxide.shrink_x = 0.2;
        oxide.shrink_y = 0.1;
        oxide.building_damage_multiplier = 0.25;
        oxide.status = "corroded".into();
        oxide.status_duration = 60.0 * 8.0;
        oxide.frag_bullets = 1;
        oxide.frag_bullet = Some(Box::new(oxide_frag));
        push_turret_ammo(&mut turret.ammo, items, "oxide", oxide);

        turret.shoot_sound = "shootTank".into();
        turret.ammo_per_shot = 4;
        turret.max_ammo = turret.ammo_per_shot * 3;
        turret.target_air = false;
        turret.shake = 4.0;
        turret.recoil = 1.0;
        turret.reload = 60.0 * 2.3;
        turret.shoot_y = 7.0;
        turret.rotate_speed = 1.4;
        turret.min_warmup = 0.85;
        turret.new_target_interval = 40.0;
        turret.shoot_warmup_speed = 0.08;
        turret.warmup_maintain_time = 120.0;
        turret.consume_coolant(30.0 / 60.0);
        turret.coolant_multiplier = 3.75;
        turret.drawer = "DrawTurret(reinforced-, RegionPart(-barrel recoil moveY=-6.6667 heatColor=f03b0e), RegionPart(-side warmup mirror moveX=2.6667 moveY=-0.5 moveRot=-40 under heatColor=red))".into();
        turret.outline_color = "darkOutline".into();
        push_liquid_amount(&mut turret.consume_liquids, liquids, "hydrogen", 5.0 / 60.0);
        turret.base.has_liquids = !turret.consume_liquids.is_empty();
        turret.scaled_health = 250.0;
        turret.range = 390.0;
        turret.base.size = 4;
    });

    registry.register_turret_block("disperse", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("thorium", 50),
                ("oxide", 150),
                ("silicon", 200),
                ("beryllium", 350),
            ],
        );

        let disperse_bullet = |speed: f32, damage: f32| {
            let mut bullet = BulletSpec::new(BulletKind::Basic, speed, damage);
            bullet.width = 16.0;
            bullet.height = 16.0;
            bullet.shrink_y = 0.3;
            bullet.back_sprite = "large-bomb-back".into();
            bullet.sprite = "mine-bullet".into();
            bullet.velocity_rnd = 0.11;
            bullet.collides_ground = false;
            bullet.collides_tiles = false;
            bullet.shoot_effect = "shootBig2".into();
            bullet.smoke_effect = "shootSmokeDisperse".into();
            bullet.lifetime = 34.0;
            bullet.rotation_offset = 90.0;
            bullet.hit_effect = "hitBulletColor".into();
            bullet.despawn_effect = "hitBulletColor".into();
            bullet
        };

        let mut tungsten = disperse_bullet(8.5, 65.0);
        tungsten.front_color = "white".into();
        tungsten.back_color = "sky".into();
        tungsten.trail_color = "sky".into();
        tungsten.hit_color = "sky".into();
        tungsten.trail_chance = 0.44;
        tungsten.ammo_multiplier = 3.0;
        tungsten.trail_rotation = true;
        tungsten.trail_effect = "disperseTrail".into();
        push_turret_ammo(&mut turret.ammo, items, "tungsten", tungsten);

        let mut thorium = disperse_bullet(9.5, 90.0);
        thorium.reload_multiplier = 0.85;
        thorium.range_change = -120.0;
        thorium.pierce_cap = 2;
        thorium.velocity_rnd = 0.5;
        thorium.front_color = "white".into();
        thorium.back_color = "e89dbd".into();
        thorium.trail_color = "e89dbd".into();
        thorium.hit_color = "e89dbd".into();
        thorium.trail_chance = 0.44;
        thorium.ammo_multiplier = 1.0;
        thorium.extra_range_margin = 32.0;
        thorium.trail_rotation = true;
        thorium.trail_effect = "disperseTrail".into();
        push_turret_ammo(&mut turret.ammo, items, "thorium", thorium);

        let mut silicon = disperse_bullet(9.0, 37.0);
        silicon.homing_power = 0.045;
        silicon.front_color = "dae1ee".into();
        silicon.back_color = "siliconAmmoBack".into();
        silicon.trail_color = "siliconAmmoBack".into();
        silicon.hit_color = "siliconAmmoBack".into();
        silicon.ammo_multiplier = 4.0;
        silicon.trail_length = 7;
        silicon.extra_range_margin = 32.0;
        push_turret_ammo(&mut turret.ammo, items, "silicon", silicon);

        let mut surge_interval = BulletSpec::new(BulletKind::Generic, 0.0, 0.0);
        surge_interval.collides_ground = false;
        surge_interval.collides_tiles = false;
        surge_interval.lightning_length_rand = 4;
        surge_interval.lightning_length = 2;
        surge_interval.lightning_cone = 30.0;
        surge_interval.lightning_damage = 20.0;
        surge_interval.lightning = 1;
        surge_interval.hittable = false;
        surge_interval.collides = false;
        surge_interval.instant_disappear = true;
        surge_interval.hit_effect = "none".into();
        surge_interval.despawn_effect = "none".into();

        let mut surge = disperse_bullet(6.0, 65.0);
        surge.reload_multiplier = 0.75;
        surge.range_change = 8.0 * 3.0;
        surge.lightning = 3;
        surge.lightning_length = 4;
        surge.lightning_damage = 18.0;
        surge.lightning_length_rand = 3;
        surge.front_color = "white".into();
        surge.back_color = "surge".into();
        surge.trail_color = "surge".into();
        surge.hit_color = "surge".into();
        surge.trail_chance = 0.44;
        surge.ammo_multiplier = 3.0;
        surge.trail_rotation = true;
        surge.trail_effect = "disperseTrail".into();
        surge.bullet_interval = 4.0;
        surge.interval_bullet = Some(Box::new(surge_interval));
        push_turret_ammo(&mut turret.ammo, items, "surge-alloy", surge);

        turret.reload = 9.0;
        turret.shoot_y = 15.0;
        turret.rotate_speed = 5.0;
        turret.shoot_cone = 30.0;
        turret.consume_ammo_once = true;
        turret.shoot_sound = "shootDisperse".into();
        turret.drawer = "DrawTurret(reinforced-, RegionPart(-side mirror under moveX=1.75 moveY=-0.5), RegionPart(-mid under moveY=-1.5 recoil heatColor=sky@0.9), RegionPart(-blade warmup mirror under moveY=1 moveX=1.5 moveRot=8 heatColor=sky@0.9))".into();
        turret.shoot_pattern = "ShootAlternate".into();
        turret.shoot_alternate_spread = 4.7;
        turret.shoot_shots = 4;
        turret.shoot_alternate_barrels = 4;
        turret.target_ground = false;
        turret.inaccuracy = 8.0;
        turret.shoot_warmup_speed = 0.08;
        turret.outline_color = "darkOutline".into();
        turret.scaled_health = 280.0;
        turret.range = 310.0;
        turret.base.size = 4;
        turret.consume_coolant(20.0 / 60.0);
        turret.coolant_multiplier = 6.25;
        turret.limit_range(16.0);
    });

    registry.register_turret_block("afflict", TurretBlockKind::PowerTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("surge-alloy", 100),
                ("silicon", 200),
                ("graphite", 250),
                ("oxide", 40),
            ],
        );

        let mut child = BulletSpec::new(BulletKind::Basic, 3.0, 35.0);
        child.width = 9.0;
        child.hit_size = 5.0;
        child.height = 15.0;
        child.pierce_cap = 3;
        child.lifetime = 28.0;
        child.pierce_building = true;
        child.hit_color = "surge".into();
        child.back_color = "surge".into();
        child.trail_color = "surge".into();
        child.front_color = "white".into();
        child.trail_width = 2.1;
        child.trail_length = 5;
        child.hit_effect = "WaveEffect(surge,sizeTo=4,strokeFrom=4,lifetime=10)".into();
        child.despawn_effect = child.hit_effect.clone();
        child.building_damage_multiplier = 0.3;
        child.homing_power = 0.1;

        let mut shoot_type = BulletSpec::new(BulletKind::Basic, 5.0, 180.0);
        shoot_type.shoot_effect =
            "MultiEffect(shootTitan, WaveEffect(colorTo=surge,sizeTo=26,lifetime=14,strokeFrom=4))"
                .into();
        shoot_type.smoke_effect = "shootSmokeTitan".into();
        shoot_type.hit_color = "surge".into();
        shoot_type.sprite = "large-orb".into();
        shoot_type.trail_effect = "missileTrail".into();
        shoot_type.trail_interval = 3.0;
        shoot_type.trail_param = 4.0;
        shoot_type.pierce_cap = 2;
        shoot_type.building_damage_multiplier = 0.5;
        shoot_type.frag_on_hit = false;
        shoot_type.lifetime = 80.0;
        shoot_type.width = 16.0;
        shoot_type.height = 16.0;
        shoot_type.back_color = "surge".into();
        shoot_type.front_color = "white".into();
        shoot_type.shrink_x = 0.0;
        shoot_type.shrink_y = 0.0;
        shoot_type.trail_color = "surge".into();
        shoot_type.trail_length = 12;
        shoot_type.trail_width = 2.2;
        shoot_type.hit_effect =
            "ExplosionEffect(waveColor=surge,smokeColor=gray,sparkColor=sap,waveStroke=4,waveRad=40)"
                .into();
        shoot_type.despawn_effect = shoot_type.hit_effect.clone();
        shoot_type.despawn_sound = "explosionAfflict".into();
        shoot_type.bullet_shoot_sound = "shootAfflict".into();
        shoot_type.frag_bullet = Some(Box::new(child.clone()));
        shoot_type.interval_bullet = Some(Box::new(child));
        shoot_type.bullet_interval = 3.0;
        shoot_type.interval_random_spread = 20.0;
        shoot_type.interval_bullets = 2;
        shoot_type.interval_angle = 180.0;
        shoot_type.interval_spread = 300.0;
        shoot_type.frag_bullets = 20;
        shoot_type.frag_velocity_min = 0.5;
        shoot_type.frag_velocity_max = 1.2;
        shoot_type.frag_life_min = 0.5;
        turret.shoot_type = Some(Box::new(shoot_type));

        turret.drawer = "DrawTurret(reinforced-, RegionPart(-blade recoil mirror under moveX=2 moveY=-1 moveRot=-7 heatColor=ff6214), RegionPart(-blade-glow recoil warmup mirror under moveX=2 moveY=-1 moveRot=-7 heatColor=ff6214 drawRegion=false))".into();
        turret.consume_power = 5.0;
        turret.heat_requirement = 20.0;
        turret.max_heat_efficiency = 1.0;
        turret.new_target_interval = 40.0;
        turret.inaccuracy = 1.0;
        turret.shake = 2.0;
        turret.shoot_y = 4.0;
        turret.outline_color = "darkOutline".into();
        turret.base.size = 4;
        turret.base.env_enabled |= Env::SPACE;
        turret.reload = 50.0;
        turret.cooldown_time = 100.0;
        turret.recoil = 3.0;
        turret.range = 368.0;
        turret.shoot_cone = 20.0;
        turret.scaled_health = 220.0;
        turret.rotate_speed = 1.5;
        turret.research_cost_multiplier = 0.04;
        turret.build_cost_multiplier = 1.5;
        turret.limit_range(-55.0);
    });

    registry.register_turret_block("lustre", TurretBlockKind::ContinuousTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("silicon", 250),
                ("graphite", 200),
                ("oxide", 50),
                ("carbide", 90),
            ],
        );

        let mut shoot_type = point_laser_bullet(210.0);
        shoot_type.building_damage_multiplier = 0.3;
        shoot_type.hit_color = "fda981".into();
        turret.shoot_type = Some(Box::new(shoot_type));

        turret.drawer = "DrawTurret(reinforced-, RegionPart(-blade warmup mirror under moveX=2 moveRot=-7 PartMove(warmup,0,-2,3) heatColor=ff6214), RegionPart(-inner warmup mirror moveX=2 moveY=-8 heatColor=ff6214), RegionPart(-mid warmup under moveY=-8 heatColor=ff6214))".into();
        turret.scale_damage_efficiency = true;
        turret.shoot_sound = "none".into();
        turret.loop_sound_volume = 1.0;
        turret.loop_sound = "beamLustre".into();
        turret.shoot_warmup_speed = 0.08;
        turret.shoot_cone = 360.0;
        turret.aim_change_speed = 0.9;
        turret.rotate_speed = 0.9;
        turret.shoot_y = 0.5;
        turret.outline_color = "darkOutline".into();
        turret.base.size = 4;
        turret.base.env_enabled |= Env::SPACE;
        turret.range = 250.0;
        turret.scaled_health = 210.0;
        turret.unit_sort = "strongest".into();
        push_liquid_amount(&mut turret.consume_liquids, liquids, "nitrogen", 6.0 / 60.0);
        turret.base.has_liquids = !turret.consume_liquids.is_empty();
        turret.consume_power = 200.0 / 60.0;
        turret.base.has_power = true;
    });

    registry.register_turret_block("scathe", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("silicon", 450),
                ("graphite", 400),
                ("tungsten", 500),
                ("oxide", 100),
                ("carbide", 200),
            ],
        );

        let make_carrier = |hit_color: &str, ammo_multiplier: f32, reload_multiplier: f32| {
            let mut bullet = BulletSpec::new(BulletKind::Generic, 0.0, 0.0);
            bullet.shoot_effect = "shootBig".into();
            bullet.smoke_effect = "shootSmokeMissileColor".into();
            bullet.hit_color = hit_color.into();
            bullet.ammo_multiplier = ammo_multiplier;
            bullet.reload_multiplier = reload_multiplier;
            bullet
        };

        let make_frag = |color: &str, building_damage_multiplier: f32, splash_radius: f32, splash_damage: f32| {
            let mut frag = artillery_bullet(3.4, 32.0);
            frag.building_damage_multiplier = building_damage_multiplier;
            frag.drag = 0.02;
            frag.hit_effect = "massiveExplosion".into();
            frag.despawn_effect = "scatheSlash".into();
            frag.knockback = 0.8;
            frag.lifetime = 23.0;
            frag.width = 18.0;
            frag.height = 18.0;
            frag.collides_tiles = false;
            frag.splash_damage_radius = splash_radius;
            frag.splash_damage = splash_damage;
            frag.back_color = color.into();
            frag.trail_color = color.into();
            frag.hit_color = color.into();
            frag.front_color = "white".into();
            frag.smoke_effect = "shootBigSmoke2".into();
            frag.despawn_shake = 7.0;
            frag.light_radius = 30.0;
            frag.light_color = color.into();
            frag.light_opacity = 0.5;
            frag.trail_length = 20;
            frag.trail_width = 3.5;
            frag.trail_effect = "none".into();
            frag
        };

        let mut carbide_explosion = explosion_bullet(1000.0, 65.0);
        carbide_explosion.hit_color = "redLight".into();
        carbide_explosion.shoot_effect =
            "MultiEffect(massiveExplosion, scatheExplosion, scatheLight, WaveEffect(lifetime=10,strokeFrom=4,sizeTo=130))"
                .into();
        carbide_explosion.collides_air = false;
        carbide_explosion.building_damage_multiplier = 0.1;
        carbide_explosion.ammo_multiplier = 1.0;
        carbide_explosion.frag_life_min = 0.1;
        carbide_explosion.frag_bullets = 7;
        carbide_explosion.frag_bullet = Some(Box::new(make_frag("redLight", 0.1, 40.0, 100.0)));

        let mut carbide_unit = missile_unit(
            "scathe-missile",
            "redLight",
            4.6,
            60.0 * 5.5,
            240.0,
            0.25,
            50.0,
            18,
            3.1,
            10.0,
        );
        carbide_unit.weapons.push(death_weapon(carbide_explosion));
        carbide_unit
            .abilities
            .push(move_effect_ability("missileTrailSmoke", "gray-redLight-0.4", 7.0));

        let mut carbide = make_carrier("redLight", 1.0, 1.0);
        carbide.spawn_unit = Some(Box::new(carbide_unit));
        push_turret_ammo(&mut turret.ammo, items, "carbide", carbide);

        let phase_color = "ffd37f";
        let mut phase_explosion = explosion_bullet(320.0, 120.0);
        phase_explosion.reload_multiplier = 0.8;
        phase_explosion.ammo_multiplier = 5.0;
        phase_explosion.hit_color = phase_color.into();
        phase_explosion.shoot_effect =
            "MultiEffect(massiveExplosion, scatheExplosion, scatheLight, WaveEffect(lifetime=10,strokeFrom=4,sizeTo=130))"
                .into();
        phase_explosion.collides_air = false;
        phase_explosion.building_damage_multiplier = 0.1;
        phase_explosion.frag_life_min = 0.1;
        phase_explosion.frag_bullets = 7;
        phase_explosion.frag_bullet = Some(Box::new(make_frag(phase_color, 0.2, 56.0, 120.0)));

        let mut phase_unit = missile_unit(
            "scathe-missile-phase",
            phase_color,
            2.5,
            60.0 * 9.77,
            500.0,
            0.2,
            50.0,
            18,
            3.1,
            10.0,
        );
        phase_unit.parts.push(UnitPartSpec {
            kind: UnitPartKind::Shape,
            progress: "constant(1)".into(),
            color: "accent".into(),
            sides: 6,
            radius: 3.0,
            rotate_speed: 3.0,
            hollow: true,
            layer: "effect".into(),
            y: 1.8,
        });
        phase_unit.weapons.push(death_weapon(phase_explosion));
        phase_unit
            .abilities
            .push(move_effect_ability("missileTrailSmoke", "gray-redLight-0.4", 15.0));
        phase_unit
            .abilities
            .push(force_field_ability(120.0, 0.0, 3000.0, 999999999.0));

        let mut phase = make_carrier(phase_color, 5.0, 0.8);
        phase.spawn_unit = Some(Box::new(phase_unit));
        push_turret_ammo(&mut turret.ammo, items, "phase-fabric", phase);

        let surge_color = "f7e97e";
        let mut split_explosion = explosion_bullet(180.0, 35.0);
        split_explosion.lightning = 4;
        split_explosion.lightning_damage = 25.0;
        split_explosion.lightning_length = 6;
        split_explosion.hit_color = surge_color.into();
        split_explosion.shoot_effect =
            "MultiEffect(massiveExplosion, scatheExplosionSmall, scatheLightSmall, WaveEffect(lifetime=10,strokeFrom=4,sizeTo=100))"
                .into();
        split_explosion.collides_air = false;
        split_explosion.building_damage_multiplier = 0.1;

        let mut split_unit = missile_unit(
            "scathe-missile-surge-split",
            surge_color,
            4.8,
            60.0 * 3.7,
            50.0,
            1.4,
            0.0,
            12,
            2.2,
            8.0,
        );
        split_unit.hit_size = 0.0;
        split_unit.weapons.push(death_weapon(split_explosion));
        split_unit
            .abilities
            .push(move_effect_ability("missileTrailSmokeSmall", "gray-f7e97e-0.4", 5.0));

        let mut split_carrier = make_carrier(surge_color, 1.0, 1.0);
        split_carrier.spawn_unit = Some(Box::new(split_unit));

        let mut surge_explosion = explosion_bullet(1800.0, 40.0);
        surge_explosion.ammo_multiplier = 1.0;
        surge_explosion.reload_multiplier = 0.9;
        surge_explosion.lightning = 10;
        surge_explosion.lightning_damage = 45.0;
        surge_explosion.lightning_length = 12;
        surge_explosion.hit_color = surge_color.into();
        surge_explosion.shoot_effect =
            "MultiEffect(massiveExplosion, scatheExplosionSmall)".into();
        surge_explosion.collides_air = false;
        surge_explosion.building_damage_multiplier = 0.1;
        surge_explosion.frag_life_min = 0.1;
        surge_explosion.frag_bullets = 5;
        surge_explosion.frag_random_spread = 0.0;
        surge_explosion.frag_spread = 20.0;
        surge_explosion.frag_bullet = Some(Box::new(split_carrier));

        let mut surge_unit = missile_unit(
            "scathe-missile-surge",
            surge_color,
            4.4,
            60.0 * 1.4,
            300.0,
            0.25,
            30.0,
            18,
            3.1,
            10.0,
        );
        let mut surge_weapon = death_weapon(surge_explosion);
        surge_weapon.rotate = true;
        surge_weapon.rotation_limit = 0.0;
        surge_weapon.rotate_speed = 0.0;
        surge_unit.weapons.push(surge_weapon);
        surge_unit
            .abilities
            .push(move_effect_ability("missileTrailSmoke", "gray-f7e97e-0.4", 7.0));

        let mut surge = make_carrier(surge_color, 1.0, 0.9);
        surge.spawn_unit = Some(Box::new(surge_unit));
        push_turret_ammo(&mut turret.ammo, items, "surge-alloy", surge);

        turret.drawer = "DrawTurret(reinforced-, RegionPart(-blade warmup mirror child=-side moveRot=-22 moveY=-5 heatColor=red), RegionPart(-mid recoil under moveY=-5), RegionPart(-missile reload under outline=false PartMove(warmup.inv,0,-4,0)))".into();
        turret.predict_target = false;
        turret.recoil = 0.5;
        turret.fog_radius_multiplier = 0.4;
        turret.coolant_multiplier = 15.0;
        turret.shoot_sound = "shootScathe".into();
        turret.min_warmup = 0.94;
        turret.new_target_interval = 40.0;
        turret.unit_sort = "strongest".into();
        turret.shoot_warmup_speed = 0.03;
        turret.target_air = false;
        turret.target_under_blocks = false;
        turret.shake = 6.0;
        turret.ammo_per_shot = 15;
        turret.max_ammo = 45;
        turret.shoot_y = -1.0;
        turret.outline_color = "darkOutline".into();
        turret.base.size = 4;
        turret.base.env_enabled |= Env::SPACE;
        turret.reload = 600.0;
        turret.range = 1350.0;
        turret.shoot_cone = 1.0;
        turret.scaled_health = 220.0;
        turret.rotate_speed = 0.9;
        turret.consume_coolant(15.0 / 60.0);
        turret.limit_range(9.0);
    });

    registry.register_turret_block("smite", TurretBlockKind::ItemTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("oxide", 200),
                ("surge-alloy", 400),
                ("silicon", 800),
                ("carbide", 500),
                ("phase-fabric", 300),
            ],
        );

        let mut lightning_type = BulletSpec::new(BulletKind::Generic, 0.0001, 0.0);
        lightning_type.lifetime = 10.0;
        lightning_type.hit_effect = "hitLancer".into();
        lightning_type.despawn_effect = "none".into();
        lightning_type.status = "shocked".into();
        lightning_type.hittable = false;
        lightning_type.light_color = "ffffffff".into();
        lightning_type.building_damage_multiplier = 0.25;

        let mut interval = BulletSpec::new(BulletKind::Lightning, 0.0, 30.0);
        interval.lifetime = 1.0;
        interval.despawn_effect = "none".into();
        interval.hit_effect = "hitLancer".into();
        interval.keep_velocity = false;
        interval.hittable = false;
        interval.status = "shocked".into();
        interval.collides_air = false;
        interval.ammo_multiplier = 1.0;
        interval.lightning_color = "accent".into();
        interval.lightning_length = 5;
        interval.lightning_length_rand = 10;
        interval.building_damage_multiplier = 0.25;
        interval.lightning_type = Some(Box::new(lightning_type));

        let mut surge = basic_bullet(7.0, 250.0);
        surge.sprite = "large-orb".into();
        surge.width = 17.0;
        surge.height = 21.0;
        surge.hit_size = 8.0;
        surge.shoot_effect =
            "MultiEffect(shootTitan, colorSparkBig, WaveEffect(colorFrom=accent,colorTo=accent,lifetime=12,sizeTo=20,strokeFrom=3,strokeTo=0.3))"
                .into();
        surge.smoke_effect = "shootSmokeSmite".into();
        surge.ammo_multiplier = 1.0;
        surge.pierce_cap = 4;
        surge.pierce = true;
        surge.pierce_building = true;
        surge.hit_color = "accent".into();
        surge.back_color = "accent".into();
        surge.trail_color = "accent".into();
        surge.front_color = "white".into();
        surge.trail_width = 2.8;
        surge.trail_length = 9;
        surge.hit_effect = "hitBulletColor".into();
        surge.building_damage_multiplier = 0.3;
        surge.despawn_effect =
            "MultiEffect(hitBulletColor, WaveEffect(sizeTo=30,colorFrom=accent,colorTo=accent,lifetime=12))"
                .into();
        surge.trail_rotation = true;
        surge.trail_effect = "disperseTrail".into();
        surge.trail_interval = 3.0;
        surge.interval_bullet = Some(Box::new(interval));
        surge.bullet_interval = 3.0;
        push_turret_ammo(&mut turret.ammo, items, "surge-alloy", surge);

        let smite_parts = [
            "RegionPart(-mid heatProgress=heat.blend(warmup,0.5) mirror=false)",
            "RegionPart(-blade progress=warmup heatProgress=warmup mirror moveX=5.5 PartMove(recoil,0,-3,0))",
            "RegionPart(-front progress=warmup heatProgress=recoil mirror under moveY=4 moveX=6.5 PartMove(recoil,0,-5.5,0))",
            "RegionPart(-back progress=warmup heatProgress=warmup mirror under moveX=5.5)",
            "ShapePart(progress=warmup.delay(0.2) color=accent circle hollow stroke=0 strokeTo=2 radius=10 layer=effect y=-15 rotateSpeed=1)",
            "ShapePart(progress=warmup.delay(0.2) color=accent circle hollow stroke=0 strokeTo=1.6 radius=4 layer=effect y=-15 rotateSpeed=1)",
            "HaloPart(progress=warmup.delay(0.5) color=accent layer=effect y=-15 haloRotation=90 shapes=2 triLength=0 triLengthTo=20 haloRadius=16 tri radius=4)",
            "HaloPart(progress=warmup.delay(0.5) color=accent layer=effect y=-15 haloRotation=90 shapes=2 triLength=0 triLengthTo=5 haloRadius=16 tri radius=4 shapeRotation=180)",
            "HaloPart(progress=warmup.delay(0.5) color=accent layer=effect y=-15 haloRotateSpeed=-1 shapes=4 triLength=0 triLengthTo=5 haloRotation=45 haloRadius=16 tri radius=8)",
            "HaloPart(progress=warmup.delay(0.5) color=accent layer=effect y=-15 haloRotateSpeed=-1 shapes=4 shapeRotation=180 triLength=0 triLengthTo=2 haloRotation=45 haloRadius=16 tri radius=8)",
            "HaloPart(progress=warmup.delay(0.5) color=accent layer=effect y=-15 haloRotateSpeed=1 shapes=4 triLength=0 triLengthTo=3 haloRotation=45 haloRadius=10 tri radius=6)",
            "RegionPart(-blade-bar progress=warmup heatProgress=warmup mirror under outline=false layerOffset=-0.3 turretHeatLayer=turret-0.2 y=11 moveX=2 color=accent)",
            "RegionPart(-blade-bar progress=warmup heatProgress=warmup mirror under outline=false layerOffset=-0.3 turretHeatLayer=turret-0.2 y=1.5 moveX=2 color=accent)",
            "RegionPart(-blade-bar progress=warmup heatProgress=warmup mirror under outline=false layerOffset=-0.3 turretHeatLayer=turret-0.2 y=-8 moveX=2 color=accent)",
            "RegionPart(-spine progress=warmup.delay(0) heatProgress=warmup mirror under layerOffset=-0.3 turretHeatLayer=turret-0.2 moveY=-5.5 moveX=15 moveRot=0 color=accent PartMove(recoil.delay(0),0,0,35))",
            "RegionPart(-spine progress=warmup.delay(0.2) heatProgress=warmup mirror under layerOffset=-0.3 turretHeatLayer=turret-0.2 moveY=-8.5 moveX=14 moveRot=-30 color=accent PartMove(recoil.delay(0.2),0,0,35))",
            "RegionPart(-spine progress=warmup.delay(0.4) heatProgress=warmup mirror under layerOffset=-0.3 turretHeatLayer=turret-0.2 moveY=-11.5 moveX=13 moveRot=-60 color=accent PartMove(recoil.delay(0.4),0,0,35))",
            "RegionPart(-spine progress=warmup.delay(0.6) heatProgress=warmup mirror under layerOffset=-0.3 turretHeatLayer=turret-0.2 moveY=-14.5 moveX=12 moveRot=-90 color=accent PartMove(recoil.delay(0.6),0,0,35))",
        ];
        turret.drawer = format!("DrawTurret(reinforced-, {})", smite_parts.join(", "));

        turret.shoot_pattern = "ShootMulti(ShootAlternate,ShootHelix)".into();
        turret.shoot_alternate_spread = 3.3 * 1.9;
        turret.shoot_shots = 5;
        turret.shoot_alternate_barrels = 5;
        turret.shoot_helix_scl = 4.0;
        turret.shoot_helix_mag = 3.0;
        turret.shoot_sound = "shootSmite".into();
        turret.min_warmup = 0.99;
        turret.coolant_multiplier = 15.0;
        turret.shake = 2.0;
        turret.ammo_per_shot = 2;
        turret.shoot_warmup_speed = 0.04;
        turret.shoot_y = 15.0;
        turret.outline_color = "darkOutline".into();
        turret.base.size = 5;
        turret.base.env_enabled |= Env::SPACE;
        turret.warmup_maintain_time = 120.0;
        turret.reload = 100.0;
        turret.recoil = 2.0;
        turret.range = 300.0;
        turret.tracking_range = turret.range * 1.4;
        turret.shoot_cone = 30.0;
        turret.scaled_health = 350.0;
        turret.rotate_speed = 1.5;
        turret.consume_coolant(15.0 / 60.0);
        turret.limit_range(9.0);
        turret.loop_sound = "loopGlow".into();
        turret.loop_sound_volume = 0.8;
    });

    registry.register_turret_block("malign", TurretBlockKind::PowerTurret, |turret| {
        set_requirements(
            &mut turret.requirements,
            items,
            &[
                ("carbide", 200),
                ("beryllium", 1000),
                ("silicon", 500),
                ("graphite", 500),
                ("phase-fabric", 200),
            ],
        );

        let halo_color = "d370d3";
        let heat_color = "purple";
        let circle_rad = 11.0;
        let circle_y = 25.0;

        let mut interval = BulletSpec::new(BulletKind::Lightning, 0.0, 18.0);
        interval.lifetime = 1.0;
        interval.despawn_effect = "none".into();
        interval.hit_effect = "hitLancer".into();
        interval.keep_velocity = false;
        interval.hittable = false;
        interval.status = "shocked".into();
        interval.lightning_color = halo_color.into();
        interval.lightning_cone = 15.0;
        interval.lightning_length = 35;
        interval.lightning_length_rand = 5;

        let mut frag = laser_bullet(65.0);
        frag.colors = vec![format!("{halo_color}@0.4"), halo_color.into(), "white".into()];
        frag.building_damage_multiplier = 0.25;
        frag.width = 19.0;
        frag.hit_effect = "hitLancer".into();
        frag.side_angle = 175.0;
        frag.side_width = 1.0;
        frag.side_length = 40.0;
        frag.lifetime = 22.0;
        frag.draw_size = 400.0;
        frag.length = 120.0;
        frag.pierce_cap = 2;
        frag.optimal_life_fract = 1.0;

        let mut shoot_type = flak_bullet(8.0, 70.0);
        shoot_type.sprite = "missile-large".into();
        shoot_type.lifetime = 40.0;
        shoot_type.width = 12.0;
        shoot_type.height = 22.0;
        shoot_type.hit_size = 7.0;
        shoot_type.shoot_effect = "shootSmokeSquareBig".into();
        shoot_type.smoke_effect = "shootSmokeDisperse".into();
        shoot_type.ammo_multiplier = 1.0;
        shoot_type.hit_color = halo_color.into();
        shoot_type.back_color = halo_color.into();
        shoot_type.trail_color = halo_color.into();
        shoot_type.lightning_color = halo_color.into();
        shoot_type.front_color = "white".into();
        shoot_type.trail_width = 3.0;
        shoot_type.trail_length = 12;
        shoot_type.hit_effect = "hitSquaresColor".into();
        shoot_type.despawn_effect = "hitBulletColor".into();
        shoot_type.building_damage_multiplier = 0.3;
        shoot_type.trail_effect = "colorSpark".into();
        shoot_type.trail_rotation = true;
        shoot_type.trail_interval = 3.0;
        shoot_type.homing_power = 0.17;
        shoot_type.homing_delay = 19.0;
        shoot_type.homing_range = 160.0;
        shoot_type.explode_range = 100.0;
        shoot_type.explode_delay = 0.0;
        shoot_type.flak_interval = 20.0;
        shoot_type.despawn_shake = 3.0;
        shoot_type.interval_bullet = Some(Box::new(interval));
        shoot_type.frag_bullet = Some(Box::new(frag));
        shoot_type.interval_bullets = 1;
        shoot_type.frag_spread = 0.0;
        shoot_type.frag_random_spread = 0.0;
        shoot_type.interval_random_spread = 0.0;
        shoot_type.bullet_interval = 20.0;
        shoot_type.splash_damage = 0.0;
        shoot_type.collides_ground = true;
        turret.shoot_type = Some(Box::new(shoot_type));

        let mut malign_parts = vec![
            "ShapePart(progress=warmup.delay(0.9) color=d370d3 circle hollow stroke=0 strokeTo=1.6 radius=11 layer=effect y=25)".to_string(),
            "ShapePart(progress=warmup.delay(0.9) rotateSpeed=-3.5 color=d370d3 sides=4 hollow stroke=0 strokeTo=1.6 radius=10 layer=effect y=25)".to_string(),
            "ShapePart(progress=warmup.delay(0.9) rotateSpeed=-3.5 color=d370d3 sides=4 hollow stroke=0 strokeTo=1.6 radius=10 layer=effect y=25)".to_string(),
            "ShapePart(progress=warmup.delay(0.9) rotateSpeed=-1.75 color=d370d3 sides=4 hollow stroke=0 strokeTo=2 radius=3 layer=effect y=25)".to_string(),
            "HaloPart(progress=warmup.delay(0.9) color=d370d3 tri shapes=3 triLength=0 triLengthTo=5 radius=6 haloRadius=11 haloRotateSpeed=0.75 shapeRotation=180 haloRotation=180 layer=effect y=25)".to_string(),
            format!("RegionPart(-mouth heatColor={heat_color} heatProgress=warmup moveY=-8)"),
            "RegionPart(-end moveY=0)".to_string(),
            format!("RegionPart(-front heatColor={heat_color} heatProgress=warmup mirror moveRot=33 moveY=-4 moveX=10)"),
            format!("RegionPart(-back heatColor={heat_color} heatProgress=warmup mirror moveRot=10 moveX=2 moveY=5)"),
            format!("RegionPart(-mid heatColor={heat_color} heatProgress=recoil moveY=-9.5)"),
            "ShapePart(progress=warmup color=d370d3 circle hollow stroke=0 strokeTo=2 radius=10 layer=effect y=-15)".to_string(),
            "ShapePart(progress=warmup color=d370d3 sides=3 rotation=90 hollow stroke=0 strokeTo=2 radius=4 layer=effect y=-15)".to_string(),
            "HaloPart(progress=warmup color=d370d3 sides=3 shapes=3 hollow stroke=0 strokeTo=2 radius=3 haloRadius=11.5 haloRotateSpeed=1.5 layer=effect y=-15)".to_string(),
            "HaloPart(progress=warmup color=d370d3 tri shapes=3 triLength=0 triLengthTo=10 radius=6 haloRadius=16 haloRotation=180 layer=effect y=-15)".to_string(),
            "HaloPart(progress=warmup color=d370d3 tri shapes=3 triLength=0 triLengthTo=3 radius=6 haloRadius=16 shapeRotation=180 haloRotation=180 layer=effect y=-15)".to_string(),
            "HaloPart(progress=warmup color=d370d3 sides=3 tri shapes=3 triLength=0 triLengthTo=10 shapeRotation=180 radius=6 haloRadius=16 haloRotateSpeed=-1.5 haloRotation=60 layer=effect y=-15)".to_string(),
            "HaloPart(progress=warmup color=d370d3 sides=3 tri shapes=3 triLength=0 triLengthTo=4 radius=6 haloRadius=16 haloRotateSpeed=-1.5 haloRotation=60 layer=effect y=-15)".to_string(),
        ];
        for i in 1..4 {
            let fi = i as f32;
            malign_parts.push(format!(
                "RegionPart(-spine outline=false progress=warmup.delay({}) heatProgress=warmup.add(absin(3,0.2)-0.2) mirror under layerOffset=-0.3 turretHeatLayer=turret-0.2 moveY=9 moveX={} moveRot={} color=bb68c3 heatColor=heatCol2 PartMove(recoil.delay({}),1,0,3))",
                fi / 5.0,
                1.0 + fi * 4.0,
                fi * 60.0 - 130.0,
                fi / 5.0
            ));
        }
        turret.drawer = format!("DrawTurret(reinforced-, {})", malign_parts.join(", "));

        turret.shoot_sound = "shootMalign".into();
        turret.loop_sound = "loopMalign".into();
        turret.loop_sound_volume = 1.3;
        turret.base.size = 5;
        turret.velocity_rnd = 0.15;
        turret.heat_requirement = 144.0;
        turret.max_heat_efficiency = 1.0;
        turret.warmup_maintain_time = 120.0;
        turret.consume_power = 40.0;
        turret.base.has_power = true;
        turret.unit_sort = "strongest".into();
        turret.shoot_pattern = "ShootSummon".into();
        turret.shoot_summon_x = 0.0;
        turret.shoot_summon_y = 0.0;
        turret.shoot_summon_radius = circle_rad;
        turret.shoot_summon_spread = 20.0;
        turret.min_warmup = 0.96;
        turret.shoot_warmup_speed = 0.08;
        turret.shoot_y = circle_y - 5.0;
        turret.outline_color = "darkOutline".into();
        turret.base.env_enabled |= Env::SPACE;
        turret.reload = 3.5;
        turret.range = 410.0;
        turret.tracking_range = turret.range * 1.4;
        turret.shoot_cone = 100.0;
        turret.scaled_health = 370.0;
        turret.rotate_speed = 2.6;
        turret.recoil = 0.5;
        turret.recoil_time = 30.0;
        turret.shake = 3.0;
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

    registry.register_crafting("cultivator", CraftingBlockKind::AttributeCrafter, |craft| {
        set_requirements(
            &mut craft.requirements,
            items,
            &[("copper", 25), ("lead", 25), ("silicon", 10)],
        );
        craft.output_item = item_amount(items, "spore-pod", 1);
        craft.craft_time = 100.0;
        craft.base.size = 2;
        craft.base.has_liquids = true;
        craft.base.has_power = true;
        craft.base.has_items = true;
        craft.base.liquid_capacity = 80.0;
        craft.craft_effect = "none".into();
        craft.base.env_required |= Env::SPORES;
        craft.attribute = "spores".into();
        craft.base_efficiency = 1.0;
        craft.boost_scale = 1.0;
        craft.max_boost = 2.0;
        craft.min_efficiency = -1.0;
        craft.ambient_sound = "loopCultivator".into();
        craft.ambient_sound_volume = 0.075;
        craft.legacy_read_warmup = true;
        craft.drawer =
            "DrawMulti(DrawRegion(-bottom), DrawLiquidTile(water), DrawDefault, DrawCultivator, DrawRegion(-top))"
                .into();
        craft.consume_power = 80.0 / 60.0;
        push_liquid_amount(&mut craft.consume_liquids, liquids, "water", 18.0 / 60.0);
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

fn register_unit_blocks(registry: &mut BlockRegistry, items: &[Item], liquids: &[Liquid]) {
    registry.register_unit_factory_block("ground-factory", UnitBlockKind::UnitFactory, |factory| {
        set_requirements(
            &mut factory.requirements,
            items,
            &[("copper", 50), ("lead", 120), ("silicon", 80)],
        );
        factory.plans = vec![
            unit_plan(
                items,
                "dagger",
                60.0 * 15.0,
                &[("silicon", 10), ("lead", 10)],
            ),
            unit_plan(
                items,
                "crawler",
                60.0 * 10.0,
                &[("silicon", 8), ("coal", 10)],
            ),
            unit_plan(
                items,
                "nova",
                60.0 * 40.0,
                &[("silicon", 30), ("lead", 20), ("titanium", 20)],
            ),
        ];
        factory.base.size = 3;
        factory.consume_power = 1.2;
        factory.research_cost_multiplier = 0.5;
    });

    registry.register_unit_factory_block("air-factory", UnitBlockKind::UnitFactory, |factory| {
        set_requirements(
            &mut factory.requirements,
            items,
            &[("copper", 60), ("lead", 70), ("silicon", 60)],
        );
        factory.plans = vec![
            unit_plan(items, "flare", 60.0 * 15.0, &[("silicon", 15)]),
            unit_plan(items, "mono", 60.0 * 35.0, &[("silicon", 30), ("lead", 15)]),
        ];
        factory.base.size = 3;
        factory.consume_power = 1.2;
        factory.research_cost_multiplier = 0.5;
    });

    registry.register_unit_factory_block("naval-factory", UnitBlockKind::UnitFactory, |factory| {
        set_requirements(
            &mut factory.requirements,
            items,
            &[("copper", 150), ("lead", 130), ("metaglass", 120)],
        );
        factory.plans = vec![
            unit_plan(
                items,
                "risso",
                60.0 * 45.0,
                &[("silicon", 20), ("metaglass", 35)],
            ),
            unit_plan(
                items,
                "retusa",
                60.0 * 35.0,
                &[("silicon", 15), ("titanium", 20)],
            ),
        ];
        factory.base.size = 3;
        factory.consume_power = 1.2;
        factory.floating = true;
    });

    registry.register_unit_factory_block(
        "tank-fabricator",
        UnitBlockKind::UnitFactory,
        |factory| {
            set_requirements(
                &mut factory.requirements,
                items,
                &[("silicon", 200), ("beryllium", 150)],
            );
            factory.plans = vec![unit_plan(
                items,
                "stell",
                60.0 * 35.0,
                &[("beryllium", 40), ("silicon", 50)],
            )];
            set_requirements(
                &mut factory.research_cost,
                items,
                &[("beryllium", 200), ("graphite", 80), ("silicon", 80)],
            );
            factory.base.size = 3;
            factory.configurable = false;
            factory.region_suffix = "-dark".into();
            factory.fog_radius = 3.0;
            factory.consume_power = 1.5;
        },
    );

    registry.register_unit_factory_block(
        "ship-fabricator",
        UnitBlockKind::UnitFactory,
        |factory| {
            set_requirements(
                &mut factory.requirements,
                items,
                &[("silicon", 250), ("beryllium", 200)],
            );
            factory.plans = vec![unit_plan(
                items,
                "elude",
                60.0 * 40.0,
                &[("graphite", 50), ("silicon", 70)],
            )];
            factory.base.size = 3;
            factory.configurable = false;
            factory.region_suffix = "-dark".into();
            factory.fog_radius = 3.0;
            factory.research_cost_multiplier = 0.5;
            factory.consume_power = 1.5;
        },
    );

    registry.register_unit_factory_block(
        "mech-fabricator",
        UnitBlockKind::UnitFactory,
        |factory| {
            set_requirements(
                &mut factory.requirements,
                items,
                &[("silicon", 200), ("beryllium", 250), ("tungsten", 10)],
            );
            factory.plans = vec![unit_plan(
                items,
                "merui",
                60.0 * 40.0,
                &[("beryllium", 50), ("silicon", 70)],
            )];
            factory.base.size = 3;
            factory.configurable = false;
            factory.region_suffix = "-dark".into();
            factory.fog_radius = 3.0;
            factory.research_cost_multiplier = 0.65;
            factory.consume_power = 1.5;
        },
    );

    registry.register_unit_assembler_block(
        "tank-assembler",
        UnitBlockKind::UnitAssembler,
        |assembler| {
            set_requirements(
                &mut assembler.requirements,
                items,
                &[
                    ("thorium", 500),
                    ("oxide", 150),
                    ("carbide", 80),
                    ("silicon", 650),
                ],
            );
            assembler.region_suffix = "-dark".into();
            assembler.base.size = 5;
            assembler.plans = vec![
                assembler_unit_plan(
                    "vanquish",
                    60.0 * 50.0,
                    vec![
                        unit_payload("stell", 4),
                        block_payload("tungsten-wall-large", 10),
                    ],
                ),
                assembler_unit_plan(
                    "conquer",
                    60.0 * 60.0 * 3.0,
                    vec![
                        unit_payload("locus", 6),
                        block_payload("carbide-wall-large", 20),
                    ],
                ),
            ];
            assembler.area_size = 13;
            assembler.research_cost_multiplier = 0.4;
            assembler.consume_power = 2.5;
            push_liquid_amount(
                &mut assembler.consume_liquids,
                liquids,
                "cyanogen",
                9.0 / 60.0,
            );
        },
    );

    registry.register_unit_assembler_block(
        "ship-assembler",
        UnitBlockKind::UnitAssembler,
        |assembler| {
            set_requirements(
                &mut assembler.requirements,
                items,
                &[
                    ("carbide", 100),
                    ("oxide", 200),
                    ("tungsten", 550),
                    ("silicon", 900),
                    ("thorium", 400),
                ],
            );
            assembler.region_suffix = "-dark".into();
            assembler.base.size = 5;
            assembler.plans = vec![
                assembler_unit_plan(
                    "quell",
                    60.0 * 60.0,
                    vec![
                        unit_payload("elude", 4),
                        block_payload("beryllium-wall-large", 12),
                    ],
                ),
                assembler_unit_plan(
                    "disrupt",
                    60.0 * 60.0 * 3.0,
                    vec![
                        unit_payload("avert", 6),
                        block_payload("carbide-wall-large", 20),
                    ],
                ),
            ];
            assembler.area_size = 13;
            assembler.consume_power = 2.5;
            push_liquid_amount(
                &mut assembler.consume_liquids,
                liquids,
                "cyanogen",
                12.0 / 60.0,
            );
        },
    );

    registry.register_unit_assembler_block(
        "mech-assembler",
        UnitBlockKind::UnitAssembler,
        |assembler| {
            set_requirements(
                &mut assembler.requirements,
                items,
                &[
                    ("carbide", 200),
                    ("thorium", 600),
                    ("oxide", 200),
                    ("tungsten", 550),
                    ("silicon", 1000),
                ],
            );
            assembler.region_suffix = "-dark".into();
            assembler.base.size = 5;
            assembler.plans = vec![
                assembler_unit_plan(
                    "tecta",
                    60.0 * 70.0,
                    vec![
                        unit_payload("merui", 5),
                        block_payload("tungsten-wall-large", 12),
                    ],
                ),
                assembler_unit_plan(
                    "collaris",
                    60.0 * 60.0 * 3.0,
                    vec![
                        unit_payload("cleroi", 6),
                        block_payload("carbide-wall-large", 20),
                    ],
                ),
            ];
            assembler.area_size = 13;
            assembler.consume_power = 3.0;
            push_liquid_amount(
                &mut assembler.consume_liquids,
                liquids,
                "cyanogen",
                12.0 / 60.0,
            );
        },
    );

    registry.register_unit_assembler_module_block(
        "basic-assembler-module",
        UnitBlockKind::UnitAssemblerModule,
        |module| {
            set_requirements(
                &mut module.requirements,
                items,
                &[
                    ("carbide", 300),
                    ("thorium", 500),
                    ("oxide", 250),
                    ("phase-fabric", 400),
                ],
            );
            module.consume_power = 3.5;
            module.region_suffix = "-dark".into();
            module.research_cost_multiplier = 0.75;
            module.base.size = 5;
        },
    );

    registry.register_unit_repair_tower_block(
        "unit-repair-tower",
        UnitBlockKind::RepairTower,
        |tower| {
            set_requirements(
                &mut tower.requirements,
                items,
                &[("graphite", 90), ("silicon", 90), ("tungsten", 80)],
            );
            tower.base.size = 2;
            tower.range = 100.0;
            tower.heal_amount = 1.5;
            tower.consume_power = 1.0;
            push_liquid_amount(&mut tower.consume_liquids, liquids, "ozone", 3.0 / 60.0);
        },
    );
}

fn register_payload_blocks(registry: &mut BlockRegistry, items: &[Item]) {
    registry.register_payload_block(
        "payload-conveyor",
        PayloadBlockKind::PayloadConveyor,
        |payload| {
            set_requirements(
                &mut payload.requirements,
                items,
                &[("graphite", 10), ("copper", 10)],
            );
            payload.can_overdrive = false;
        },
    );

    registry.register_payload_block(
        "payload-router",
        PayloadBlockKind::PayloadRouter,
        |payload| {
            set_requirements(
                &mut payload.requirements,
                items,
                &[("graphite", 15), ("copper", 10)],
            );
            payload.can_overdrive = false;
        },
    );

    registry.register_payload_block(
        "reinforced-payload-conveyor",
        PayloadBlockKind::PayloadConveyor,
        |payload| {
            set_requirements(&mut payload.requirements, items, &[("tungsten", 10)]);
            payload.move_time = 35.0;
            payload.can_overdrive = false;
            payload.base.health = 800;
            payload.research_cost_multiplier = 4.0;
            payload.under_bullets = true;
        },
    );

    registry.register_payload_block(
        "reinforced-payload-router",
        PayloadBlockKind::PayloadRouter,
        |payload| {
            set_requirements(&mut payload.requirements, items, &[("tungsten", 15)]);
            payload.move_time = 35.0;
            payload.base.health = 800;
            payload.can_overdrive = false;
            payload.research_cost_multiplier = 4.0;
            payload.under_bullets = true;
        },
    );
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
            assert_eq!(tree.layer, 71.0);
            assert_eq!(tree.shadow_layer, 69.0);
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
            Some("thorium (Wall)")
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
    fn extractor_and_cultivator_production_blocks_keep_upstream_subset() {
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

        let water_extractor = registry.get_production_by_name("water-extractor").unwrap();
        assert_eq!(water_extractor.kind, ProductionBlockKind::SolidPump);
        assert_eq!(water_extractor.base.group, BlockGroup::Liquids);
        assert!(water_extractor.base.update);
        assert!(water_extractor.base.solid);
        assert!(water_extractor.base.has_liquids);
        assert!(water_extractor.base.has_power);
        assert_eq!(water_extractor.base.env_enabled, Env::TERRESTRIAL);
        assert_ne!(water_extractor.base.env_required & Env::GROUND_WATER, 0);
        assert_eq!(water_extractor.result_liquid, Some(liquid_id("water")));
        assert_eq!(water_extractor.pump_amount, 0.11);
        assert_eq!(water_extractor.base.size, 2);
        assert_eq!(water_extractor.base.liquid_capacity, 40.0);
        assert_eq!(water_extractor.rotate_speed, 1.4);
        assert_eq!(water_extractor.attribute, "water");
        assert_eq!(water_extractor.base_efficiency, 1.0);
        assert_eq!(water_extractor.consume_power, 1.5);
        assert_eq!(
            water_extractor.requirements,
            vec![
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("copper"),
                    amount: 30
                }
            ]
        );

        let cultivator = registry.get_crafting_by_name("cultivator").unwrap();
        assert_eq!(cultivator.kind, CraftingBlockKind::AttributeCrafter);
        assert_eq!(
            cultivator.output_item,
            Some(ItemAmount {
                item: item_id("spore-pod"),
                amount: 1
            })
        );
        assert_eq!(cultivator.craft_time, 100.0);
        assert_eq!(cultivator.base.size, 2);
        assert!(cultivator.base.has_liquids);
        assert!(cultivator.base.has_power);
        assert!(cultivator.base.has_items);
        assert_eq!(cultivator.base.liquid_capacity, 80.0);
        assert_eq!(cultivator.craft_effect, "none");
        assert_ne!(cultivator.base.env_required & Env::SPORES, 0);
        assert_eq!(cultivator.attribute, "spores");
        assert_eq!(cultivator.base_efficiency, 1.0);
        assert_eq!(cultivator.boost_scale, 1.0);
        assert_eq!(cultivator.max_boost, 2.0);
        assert_eq!(cultivator.min_efficiency, -1.0);
        assert_eq!(cultivator.ambient_sound, "loopCultivator");
        assert_eq!(cultivator.ambient_sound_volume, 0.075);
        assert!(cultivator.legacy_read_warmup);
        assert_eq!(cultivator.consume_power, 80.0 / 60.0);
        assert_eq!(
            cultivator.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("water"),
                amount: 18.0 / 60.0
            }]
        );

        let oil = registry.get_production_by_name("oil-extractor").unwrap();
        assert_eq!(oil.kind, ProductionBlockKind::Fracker);
        assert_eq!(oil.result_liquid, Some(liquid_id("oil")));
        assert_eq!(oil.update_effect, "pulverize");
        assert_eq!(oil.update_effect_chance, 0.05);
        assert_eq!(oil.pump_amount, 0.25);
        assert_eq!(oil.base.size, 3);
        assert_eq!(oil.base.liquid_capacity, 40.0);
        assert_eq!(oil.attribute, "oil");
        assert_eq!(oil.base_efficiency, 0.0);
        assert_eq!(oil.item_use_time, 60.0);
        assert!(oil.base.has_items);
        assert!(oil.base.has_power);
        assert_eq!(oil.base.env_enabled, Env::TERRESTRIAL);
        assert_ne!(oil.base.env_required & Env::GROUND_OIL, 0);
        assert_eq!(oil.ambient_sound, "loopDrill");
        assert_eq!(oil.ambient_sound_volume, 0.03);
        assert_eq!(
            oil.consume_items,
            vec![ItemAmount {
                item: item_id("sand"),
                amount: 1
            }]
        );
        assert_eq!(oil.consume_power, 3.0);
        assert_eq!(
            oil.consume_liquids,
            vec![LiquidConsume {
                liquid: liquid_id("water"),
                amount: 0.15,
                booster: false
            }]
        );
        assert_eq!(
            oil.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 175
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 115
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 115
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 75
                }
            ]
        );
    }

    #[test]
    fn cliff_crushers_keep_upstream_wall_crafter_subset() {
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
        let sand_id = item_id("sand");

        let cliff = registry.get_production_by_name("cliff-crusher").unwrap();
        assert_eq!(cliff.kind, ProductionBlockKind::WallCrafter);
        assert!(cliff.base.has_items);
        assert!(cliff.rotate);
        assert!(cliff.base.update);
        assert!(cliff.base.solid);
        assert!(cliff.ignore_line_rotation);
        assert_eq!(cliff.region_rotated1, 1);
        assert_ne!(cliff.base.env_enabled & Env::SPACE, 0);
        assert!(cliff.base.flags.contains(&BlockFlag::Drill));
        assert_eq!(cliff.consume_power, 11.0 / 60.0);
        assert_eq!(cliff.drill_time, 110.0);
        assert_eq!(cliff.base.size, 2);
        assert_eq!(cliff.attribute, "sand");
        assert_eq!(cliff.output_item, Some(sand_id));
        assert_eq!(cliff.fog_radius, 2.0);
        assert_eq!(cliff.ambient_sound, "loopDrill");
        assert_eq!(cliff.ambient_sound_volume, 0.04);
        assert_eq!(cliff.liquid_boost_intensity, 1.6);
        assert_eq!(cliff.item_boost_intensity, 1.6);
        assert_eq!(cliff.boost_item_use_time, 120.0);
        assert!(!cliff.has_liquid_booster);
        assert_eq!(
            cliff.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 25
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 20
                }
            ]
        );
        assert_eq!(
            cliff.research_cost,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 40
                }
            ]
        );

        let large = registry
            .get_production_by_name("large-cliff-crusher")
            .unwrap();
        assert_eq!(large.kind, ProductionBlockKind::WallCrafter);
        assert_eq!(large.consume_power, 1.0);
        assert_eq!(large.drill_time, 48.0);
        assert_eq!(large.base.size, 3);
        assert_eq!(large.attribute, "sand");
        assert_eq!(large.output_item, Some(sand_id));
        assert_eq!(large.fog_radius, 3.0);
        assert_eq!(large.ambient_sound, "loopDrill");
        assert_eq!(large.ambient_sound_volume, 0.08);
        assert_eq!(large.base.item_capacity, 20);
        assert_eq!(large.boost_item_use_time, 60.0 / 0.75);
        assert!(!large.has_liquid_booster);
        assert_eq!(
            large.consume_liquids,
            vec![LiquidConsume {
                liquid: liquid_id("hydrogen"),
                amount: 1.0 / 60.0,
                booster: false
            }]
        );
        assert_eq!(
            large.consume_item_boosts,
            vec![ItemAmount {
                item: item_id("graphite"),
                amount: 1
            }]
        );
        assert_eq!(
            large.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 15
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 50
                }
            ]
        );
    }

    #[test]
    fn plasma_bores_keep_upstream_beam_drill_subset() {
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

        let plasma = registry.get_production_by_name("plasma-bore").unwrap();
        assert_eq!(plasma.kind, ProductionBlockKind::BeamDrill);
        assert!(plasma.base.has_items);
        assert!(plasma.rotate);
        assert!(plasma.base.update);
        assert!(plasma.base.solid);
        assert!(!plasma.draw_arrow);
        assert_eq!(plasma.region_rotated1, 1);
        assert!(plasma.ignore_line_rotation);
        assert_ne!(plasma.base.env_enabled & Env::SPACE, 0);
        assert!(plasma.base.flags.contains(&BlockFlag::Drill));
        assert_eq!(plasma.ambient_sound, "loopMineBeam");
        assert_eq!(plasma.ambient_sound_volume, 0.05);
        assert_eq!(plasma.optional_boost_intensity, 2.5);
        assert_eq!(plasma.laser_width, 0.65);
        assert_eq!(plasma.consume_power, 0.15);
        assert_eq!(plasma.drill_time, 160.0);
        assert_eq!(plasma.tier, 3);
        assert_eq!(plasma.base.size, 2);
        assert_eq!(plasma.range, 5);
        assert_eq!(plasma.fog_radius, 3.0);
        assert!(plasma.has_liquid_booster);
        assert_eq!(
            plasma.requirements,
            vec![ItemAmount {
                item: item_id("beryllium"),
                amount: 40
            }]
        );
        assert_eq!(
            plasma.research_cost,
            vec![ItemAmount {
                item: item_id("beryllium"),
                amount: 10
            }]
        );
        assert_eq!(
            plasma.consume_liquids,
            vec![LiquidConsume {
                liquid: liquid_id("hydrogen"),
                amount: 0.25 / 60.0,
                booster: true
            }]
        );

        let large = registry
            .get_production_by_name("large-plasma-bore")
            .unwrap();
        assert_eq!(large.kind, ProductionBlockKind::BeamDrill);
        assert_eq!(large.consume_power, 0.8);
        assert_eq!(large.drill_time, 100.0);
        assert_eq!(large.tier, 5);
        assert_eq!(large.base.size, 3);
        assert_eq!(large.range, 6);
        assert_eq!(large.fog_radius, 4.0);
        assert_eq!(large.laser_width, 0.7);
        assert_eq!(large.base.item_capacity, 20);
        assert!(large.has_liquid_booster);
        assert_eq!(
            large.consume_liquids,
            vec![
                LiquidConsume {
                    liquid: liquid_id("hydrogen"),
                    amount: 0.5 / 60.0,
                    booster: false
                },
                LiquidConsume {
                    liquid: liquid_id("nitrogen"),
                    amount: 3.0 / 60.0,
                    booster: true
                }
            ]
        );
        assert_eq!(
            large.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 25
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 70
                }
            ]
        );
        assert_eq!(
            large.research_cost,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 1500
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 3000
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 1200
                }
            ]
        );
    }

    #[test]
    fn burst_drills_keep_upstream_burst_drill_subset() {
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

        let impact = registry.get_production_by_name("impact-drill").unwrap();
        assert_eq!(impact.kind, ProductionBlockKind::BurstDrill);
        assert!(impact.base.update);
        assert!(impact.base.solid);
        assert_eq!(impact.base.group, BlockGroup::Drills);
        assert!(impact.base.has_liquids);
        assert!(impact.base.has_items);
        assert_ne!(impact.base.env_enabled & Env::SPACE, 0);
        assert!(impact.base.flags.contains(&BlockFlag::Drill));
        assert_eq!(impact.hardness_drill_multiplier, 0.0);
        assert_eq!(impact.drill_effect_rnd, 0.0);
        assert_eq!(impact.ambient_sound, "drillCharge");
        assert_eq!(impact.ambient_sound_volume, 0.18);
        assert_eq!(impact.speed_curve, "pow2In");
        assert_eq!(impact.inverted_time, 200.0);
        assert_eq!(impact.arrow_spacing, 4.0);
        assert_eq!(impact.arrow_offset, 0.0);
        assert_eq!(impact.arrows, 3);
        assert_eq!(impact.arrow_color, "feb380");
        assert_eq!(impact.base_arrow_color, "6e7080");
        assert_eq!(impact.glow_color_alpha, 1.0);
        assert_eq!(impact.drill_sound, "drillImpact");
        assert_eq!(impact.drill_sound_volume, 0.6);
        assert_eq!(impact.drill_sound_pitch_rand, 0.1);
        assert_eq!(impact.drill_time, 720.0);
        assert_eq!(impact.base.size, 4);
        assert!(impact.base.has_power);
        assert_eq!(impact.tier, 6);
        assert_eq!(
            impact.drill_effect,
            "MultiEffect(mineImpact, drillSteam, mineImpactWave(redLight, 40))"
        );
        assert_eq!(impact.shake, 4.0);
        assert_eq!(impact.base.item_capacity, 40);
        assert_eq!(impact.blocked_items, vec![item_id("thorium")]);
        assert_eq!(impact.research_cost_multiplier, 0.5);
        assert_eq!(
            impact.drill_multipliers,
            vec![ItemMultiplier {
                item: item_id("beryllium"),
                multiplier: 2.0
            }]
        );
        assert_eq!(impact.liquid_boost_intensity, 1.75);
        assert_eq!(impact.fog_radius, 4.0);
        assert_eq!(impact.consume_power, 160.0 / 60.0);
        assert!(impact.has_liquid_booster);
        assert_eq!(
            impact.consume_liquids,
            vec![
                LiquidConsume {
                    liquid: liquid_id("water"),
                    amount: 10.0 / 60.0,
                    booster: false
                },
                LiquidConsume {
                    liquid: liquid_id("ozone"),
                    amount: 3.0 / 60.0,
                    booster: true
                }
            ]
        );
        assert_eq!(
            impact.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 70
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 90
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 60
                }
            ]
        );

        let eruption = registry.get_production_by_name("eruption-drill").unwrap();
        assert_eq!(eruption.kind, ProductionBlockKind::BurstDrill);
        assert_eq!(eruption.drill_time, 281.25);
        assert_eq!(eruption.base.size, 5);
        assert!(eruption.base.has_power);
        assert_eq!(eruption.tier, 7);
        assert_eq!(
            eruption.drill_effect,
            "MultiEffect(mineImpact, drillSteam, dynamicSpikes(hydrogen, 30), mineImpactWave(hydrogen, 45))"
        );
        assert_eq!(eruption.shake, 4.0);
        assert_eq!(eruption.base.item_capacity, 60);
        assert_eq!(eruption.arrow_offset, 2.0);
        assert_eq!(eruption.arrow_spacing, 5.0);
        assert_eq!(eruption.arrows, 2);
        assert_eq!(eruption.glow_color_alpha, 0.6);
        assert_eq!(eruption.fog_radius, 5.0);
        assert_eq!(
            eruption.drill_multipliers,
            vec![ItemMultiplier {
                item: item_id("beryllium"),
                multiplier: 2.0
            }]
        );
        assert_eq!(eruption.liquid_boost_intensity, 2.0);
        assert_eq!(eruption.consume_power, 6.0);
        assert!(eruption.has_liquid_booster);
        assert_eq!(
            eruption.consume_liquids,
            vec![
                LiquidConsume {
                    liquid: liquid_id("hydrogen"),
                    amount: 4.0 / 60.0,
                    booster: false
                },
                LiquidConsume {
                    liquid: liquid_id("cyanogen"),
                    amount: 0.75 / 60.0,
                    booster: true
                }
            ]
        );
        assert_eq!(
            eruption.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 300
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 150
                }
            ]
        );
    }

    #[test]
    fn storage_cores_and_vaults_keep_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;

        for name in [
            "core-shard",
            "core-foundation",
            "core-nucleus",
            "core-bastion",
            "core-citadel",
            "core-acropolis",
        ] {
            let core = registry.get_storage_by_name(name).unwrap();
            assert_eq!(core.kind, StorageBlockKind::Core, "{name} kind");
            assert!(core.base.has_items, "{name} has items");
            assert!(core.base.solid, "{name} solid");
            assert!(core.base.update, "{name} update");
            assert!(core.base.destructible, "{name} destructible");
            assert!(!core.base.sync, "{name} core item sync is external");
            assert!(core.always_allow_deposit, "{name} always deposit");
            assert_eq!(core.base.priority, 2, "{name} target priority");
            assert_eq!(core.base.flags, vec![BlockFlag::Core], "{name} flags");
            assert!(core.unit_cap_modifier > 0, "{name} unit cap modifier");
            assert!(!core.draw_disabled, "{name} disabled overlay");
            assert!(!core.can_overdrive, "{name} overdrive");
            assert!(core.commandable, "{name} commandable");
            assert_ne!(core.base.env_enabled & Env::SPACE, 0, "{name} space");
            assert!(!core.base.replaceable, "{name} replaceable");
            assert_eq!(core.destroy_sound, "explosionCore", "{name} destroy sound");
            assert_eq!(core.destroy_sound_volume, 1.6, "{name} destroy volume");
            assert!(core.allow_spawn, "{name} allow spawn");
            assert_eq!(core.land_duration, 160.0, "{name} land duration");
            assert_eq!(core.land_music, "land", "{name} land music");
            assert_eq!(core.launch_sound, "coreLaunch", "{name} launch sound");
            assert_eq!(core.land_sound, "coreLand", "{name} land sound");
            assert_eq!(core.launch_sound_volume, 1.0, "{name} launch volume");
            assert_eq!(core.land_sound_volume, 1.0, "{name} land volume");
            assert_eq!(core.launch_effect, "launch", "{name} launch effect");
            assert_eq!(core.land_zoom_interp, "pow3", "{name} zoom interp");
            assert_eq!(core.land_zoom_from, 0.02, "{name} zoom from");
            assert_eq!(core.land_zoom_to, 4.0, "{name} zoom to");
            assert_eq!(core.capture_invincibility, 900.0, "{name} iframes");
            assert!(!core.outputs_items, "{name} outputs items");
            assert_eq!(core.base.group, BlockGroup::Transportation, "{name} group");
        }

        let shard = registry.get_storage_by_name("core-shard").unwrap();
        assert_eq!(shard.base.build_visibility, BuildVisibility::CoreZoneOnly);
        assert!(shard.always_unlocked);
        assert!(shard.is_first_tier);
        assert_eq!(shard.unit_type, "alpha");
        assert_eq!(shard.base.health, 1100);
        assert_eq!(shard.base.item_capacity, 4000);
        assert_eq!(shard.base.size, 3);
        assert_eq!(shard.build_cost_multiplier, 2.0);
        assert_eq!(shard.unit_cap_modifier, 8);
        assert_eq!(
            shard.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 1000
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 800
                }
            ]
        );

        let foundation = registry.get_storage_by_name("core-foundation").unwrap();
        assert_eq!(foundation.unit_type, "beta");
        assert_eq!(foundation.base.health, 3500);
        assert_eq!(foundation.base.item_capacity, 9000);
        assert_eq!(foundation.base.size, 4);
        assert_eq!(foundation.thruster_length, 34.0 / 4.0);
        assert_eq!(foundation.unit_cap_modifier, 16);
        assert_eq!(foundation.research_cost_multiplier, 0.07);

        let nucleus = registry.get_storage_by_name("core-nucleus").unwrap();
        assert_eq!(nucleus.unit_type, "gamma");
        assert_eq!(nucleus.base.health, 6000);
        assert_eq!(nucleus.base.item_capacity, 13000);
        assert_eq!(nucleus.base.size, 5);
        assert_eq!(nucleus.thruster_length, 40.0 / 4.0);
        assert_eq!(nucleus.unit_cap_modifier, 24);
        assert_eq!(nucleus.research_cost_multiplier, 0.11);

        let bastion = registry.get_storage_by_name("core-bastion").unwrap();
        assert_eq!(bastion.unit_type, "evoke");
        assert_eq!(bastion.base.health, 4500);
        assert_eq!(bastion.base.item_capacity, 2000);
        assert_eq!(bastion.base.size, 4);
        assert_eq!(bastion.thruster_length, 34.0 / 4.0);
        assert_eq!(bastion.armor, 5.0);
        assert!(bastion.always_unlocked);
        assert!(bastion.is_first_tier);
        assert!(bastion.incinerate_non_buildable);
        assert!(bastion.requires_core_zone);
        assert_eq!(bastion.build_cost_multiplier, 0.7);
        assert_eq!(bastion.unit_cap_modifier, 15);
        assert_eq!(bastion.research_cost_multiplier, 0.07);

        let citadel = registry.get_storage_by_name("core-citadel").unwrap();
        assert_eq!(citadel.unit_type, "incite");
        assert_eq!(citadel.base.health, 16000);
        assert_eq!(citadel.base.item_capacity, 3000);
        assert_eq!(citadel.base.size, 5);
        assert_eq!(citadel.thruster_length, 40.0 / 4.0);
        assert_eq!(citadel.armor, 10.0);
        assert!(citadel.incinerate_non_buildable);
        assert!(citadel.requires_core_zone);
        assert_eq!(citadel.build_cost_multiplier, 0.7);
        assert_eq!(citadel.unit_cap_modifier, 15);
        assert_eq!(citadel.research_cost_multiplier, 0.17);
        assert_eq!(
            citadel.research_cost_multipliers,
            vec![ItemMultiplier {
                item: item_id("silicon"),
                multiplier: 0.5
            }]
        );

        let acropolis = registry.get_storage_by_name("core-acropolis").unwrap();
        assert_eq!(acropolis.unit_type, "emanate");
        assert_eq!(acropolis.base.health, 30000);
        assert_eq!(acropolis.base.item_capacity, 4000);
        assert_eq!(acropolis.base.size, 6);
        assert_eq!(acropolis.thruster_length, 48.0 / 4.0);
        assert_eq!(acropolis.armor, 15.0);
        assert!(acropolis.incinerate_non_buildable);
        assert!(acropolis.requires_core_zone);
        assert_eq!(acropolis.build_cost_multiplier, 0.7);
        assert_eq!(acropolis.unit_cap_modifier, 15);
        assert_eq!(acropolis.research_cost_multiplier, 0.1);
        assert_eq!(
            acropolis.research_cost_multipliers,
            vec![ItemMultiplier {
                item: item_id("silicon"),
                multiplier: 0.4
            }]
        );

        let container = registry.get_storage_by_name("container").unwrap();
        assert_eq!(container.kind, StorageBlockKind::Storage);
        assert!(container.base.has_items);
        assert!(container.base.solid);
        assert!(!container.base.update);
        assert!(container.base.sync);
        assert!(container.base.destructible);
        assert!(container.separate_item_capacity);
        assert_eq!(container.base.group, BlockGroup::Transportation);
        assert_eq!(container.base.flags, vec![BlockFlag::Storage]);
        assert!(container.allow_resupply);
        assert_eq!(container.base.env_enabled, Env::ANY);
        assert!(!container.outputs_items);
        assert!(container.core_merge);
        assert_eq!(container.base.size, 2);
        assert_eq!(container.base.item_capacity, 300);
        assert_eq!(container.scaled_health, 55.0);
        assert_eq!(container.base.health, 2 * 2 * 55);
        assert_eq!(
            container.requirements,
            vec![ItemAmount {
                item: item_id("titanium"),
                amount: 100
            }]
        );

        let vault = registry.get_storage_by_name("vault").unwrap();
        assert!(vault.core_merge);
        assert_eq!(vault.base.size, 3);
        assert_eq!(vault.base.item_capacity, 1000);
        assert_eq!(vault.scaled_health, 55.0);
        assert_eq!(vault.base.health, 3 * 3 * 55);
        assert_eq!(
            vault.requirements,
            vec![
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 125
                }
            ]
        );

        let reinforced_container = registry
            .get_storage_by_name("reinforced-container")
            .unwrap();
        assert!(!reinforced_container.core_merge);
        assert_eq!(reinforced_container.base.size, 2);
        assert_eq!(reinforced_container.base.item_capacity, 160);
        assert_eq!(reinforced_container.scaled_health, 120.0);
        assert_eq!(reinforced_container.base.health, 2 * 2 * 120);
        assert_eq!(
            reinforced_container.requirements,
            vec![
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 40
                }
            ]
        );

        let reinforced_vault = registry.get_storage_by_name("reinforced-vault").unwrap();
        assert!(!reinforced_vault.core_merge);
        assert_eq!(reinforced_vault.base.size, 3);
        assert_eq!(reinforced_vault.base.item_capacity, 900);
        assert_eq!(reinforced_vault.scaled_health, 120.0);
        assert_eq!(reinforced_vault.base.health, 3 * 3 * 120);
        assert_eq!(
            reinforced_vault.requirements,
            vec![
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 125
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 70
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 100
                }
            ]
        );
    }

    #[test]
    fn basic_item_turrets_keep_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let duo = registry.get_turret_by_name("duo").unwrap();
        assert_eq!(duo.kind, TurretBlockKind::ItemTurret);
        assert!(duo.base.update);
        assert!(duo.base.solid);
        assert_eq!(duo.base.priority, 1);
        assert_eq!(duo.base.group, BlockGroup::Turrets);
        assert_eq!(duo.base.flags, vec![BlockFlag::Turret]);
        assert!(duo.base.sync);
        assert!(duo.base.has_items);
        assert_eq!(duo.base.liquid_capacity, 20.0);
        assert!(duo.outlined_icon);
        assert!(!duo.draw_liquid_light);
        assert!(duo.rotate);
        assert!(!duo.quick_rotate);
        assert!(!duo.draw_arrow);
        assert!(duo.ignore_line_rotation);
        assert_eq!(duo.visual_rotation_offset, -90.0);
        assert_eq!(duo.region_rotated1, 1);
        assert_eq!(duo.region_rotated2, 2);
        assert_eq!(duo.range, 160.0);
        assert_eq!(duo.reload, 20.0);
        assert_eq!(duo.recoil, 0.5);
        assert_eq!(duo.recoils, 2);
        assert_eq!(duo.shoot_y, 3.0);
        assert_eq!(duo.shoot_cone, 15.0);
        assert_eq!(duo.inaccuracy, 2.0);
        assert_eq!(duo.rotate_speed, 10.0);
        assert_eq!(duo.shoot_sound, "shootDuo");
        assert_eq!(duo.ammo_use_effect, "casing1");
        assert_eq!(duo.base.health, 250);
        assert!(duo.consume_coolant);
        assert_eq!(duo.coolant_amount, 0.1);
        assert_eq!(duo.coolant_multiplier, 10.0);
        assert_eq!(duo.research_cost_multiplier, 0.05);
        assert_eq!(duo.deposit_cooldown, 2.0);
        assert_eq!(duo.fog_radius, 20.0);
        assert_eq!(duo.place_overlap_range, 160.0 + 16.0 + 8.0 * 7.0);
        assert_eq!(duo.shoot_pattern, "ShootAlternate");
        assert_eq!(duo.shoot_alternate_spread, 3.5);
        assert_eq!(duo.shoot_alternate_barrels, 2);
        assert_eq!(
            duo.requirements,
            vec![ItemAmount {
                item: item_id("copper"),
                amount: 35
            }]
        );

        let copper = &ammo_for(duo, item_id("copper")).bullet;
        assert_eq!(copper.kind, BulletKind::Basic);
        assert_eq!(copper.speed, 2.5);
        assert_eq!(copper.damage, 9.0);
        assert_eq!(copper.width, 7.0);
        assert_eq!(copper.height, 9.0);
        assert_close(copper.lifetime, (160.0 + 5.0 + 10.0) / 2.5);
        assert_eq!(copper.ammo_multiplier, 2.0);
        assert_eq!(copper.hit_effect, "hitBulletColor");
        assert_eq!(copper.despawn_effect, "hitBulletColor");
        assert_eq!(copper.front_color, "copperAmmoFront");
        assert_eq!(copper.back_color, "copperAmmoBack");

        let graphite = &ammo_for(duo, item_id("graphite")).bullet;
        assert_eq!(graphite.damage, 18.0);
        assert_eq!(graphite.width, 9.0);
        assert_eq!(graphite.height, 12.0);
        assert_eq!(graphite.ammo_multiplier, 4.0);
        assert_eq!(graphite.reload_multiplier, 0.8);
        assert_eq!(graphite.range_change, 16.0);
        assert_close(graphite.lifetime, (160.0 + 16.0 + 5.0 + 10.0) / 3.5);
        assert_eq!(graphite.front_color, "graphiteAmmoFront");

        let silicon = &ammo_for(duo, item_id("silicon")).bullet;
        assert_eq!(silicon.damage, 12.0);
        assert_eq!(silicon.homing_power, 0.2);
        assert_eq!(silicon.reload_multiplier, 1.5);
        assert_eq!(silicon.ammo_multiplier, 5.0);
        assert_eq!(silicon.trail_length, 5);
        assert_eq!(silicon.trail_width, 1.5);
        assert_close(silicon.lifetime, (160.0 + 5.0 + 10.0) / 3.0);
        assert_eq!(silicon.front_color, "siliconAmmoFront");

        let scatter = registry.get_turret_by_name("scatter").unwrap();
        assert_eq!(scatter.kind, TurretBlockKind::ItemTurret);
        assert_eq!(scatter.reload, 18.0);
        assert_eq!(scatter.range, 220.0);
        assert_eq!(scatter.base.size, 2);
        assert!(!scatter.target_ground);
        assert!(scatter.target_air);
        assert_eq!(scatter.shoot_shot_delay, 5.0);
        assert_eq!(scatter.shoot_shots, 2);
        assert_eq!(scatter.recoil, 1.0);
        assert_eq!(scatter.rotate_speed, 15.0);
        assert_eq!(scatter.inaccuracy, 17.0);
        assert_eq!(scatter.shoot_cone, 35.0);
        assert_eq!(scatter.scaled_health, 200.0);
        assert_eq!(scatter.base.health, 2 * 2 * 200);
        assert_eq!(scatter.shoot_sound, "shootScatter");
        assert!(scatter.consume_coolant);
        assert_eq!(scatter.coolant_amount, 0.2);
        assert_eq!(scatter.research_cost_multiplier, 0.05);
        assert_eq!(scatter.deposit_cooldown, 0.5);
        assert_eq!(scatter.fog_radius, 28.0);
        assert_eq!(scatter.place_overlap_range, 220.0 + 8.0 * 7.0);
        assert_eq!(
            scatter.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 85
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 45
                }
            ]
        );

        let scrap = &ammo_for(scatter, item_id("scrap")).bullet;
        assert_eq!(scrap.kind, BulletKind::Flak);
        assert_eq!(scrap.speed, 4.0);
        assert_eq!(scrap.damage, 3.0);
        assert_eq!(scrap.ammo_multiplier, 5.0);
        assert_eq!(scrap.reload_multiplier, 0.5);
        assert_eq!(scrap.shoot_effect, "shootSmall");
        assert_eq!(scrap.hit_effect, "flakExplosion");
        assert_eq!(scrap.splash_damage, 22.0 * 1.5);
        assert_eq!(scrap.splash_damage_radius, 24.0);
        assert_close(scrap.lifetime, (220.0 + 2.0 + 10.0) / 4.0);
        assert_eq!(scrap.front_color, "scrapAmmoFront");
        assert_eq!(scrap.back_color, "scrapAmmoBack");

        let lead = &ammo_for(scatter, item_id("lead")).bullet;
        assert_eq!(lead.speed, 4.2);
        assert_eq!(lead.ammo_multiplier, 4.0);
        assert_eq!(lead.splash_damage, 27.0 * 1.5);
        assert_eq!(lead.splash_damage_radius, 15.0);
        assert_close(lead.lifetime, (220.0 + 2.0 + 10.0) / 4.2);

        let metaglass = &ammo_for(scatter, item_id("metaglass")).bullet;
        assert_eq!(metaglass.ammo_multiplier, 5.0);
        assert_eq!(metaglass.reload_multiplier, 0.8);
        assert_eq!(metaglass.splash_damage, 30.0 * 1.5);
        assert_eq!(metaglass.splash_damage_radius, 20.0);
        assert_eq!(metaglass.frag_bullets, 6);
        assert!(!metaglass.collides_ground);
        assert_eq!(metaglass.explode_range, 30.0);
        assert_eq!(metaglass.explode_delay, 5.0);
        assert_eq!(metaglass.flak_interval, 6.0);
        assert_close(metaglass.lifetime, (220.0 + 2.0 + 10.0) / 4.0);
        let frag = metaglass.frag_bullet.as_ref().unwrap();
        assert_eq!(frag.kind, BulletKind::Basic);
        assert_eq!(frag.speed, 3.0);
        assert_eq!(frag.damage, 5.0);
        assert_eq!(frag.width, 5.0);
        assert_eq!(frag.height, 12.0);
        assert_eq!(frag.shrink_y, 1.0);
        assert_eq!(frag.lifetime, 20.0);
        assert!(!frag.collides_ground);
        assert_eq!(frag.front_color, "glassAmmoFront");
        assert_eq!(frag.back_color, "glassAmmoBack");
    }

    #[test]
    fn flame_and_artillery_item_turrets_keep_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let scorch = registry.get_turret_by_name("scorch").unwrap();
        assert_eq!(scorch.kind, TurretBlockKind::ItemTurret);
        assert_eq!(scorch.recoil, 0.0);
        assert_eq!(scorch.reload, 6.0);
        assert_eq!(scorch.coolant_multiplier, 1.5);
        assert_eq!(scorch.range, 60.0);
        assert_eq!(scorch.shoot_y, 3.0);
        assert_eq!(scorch.shoot_cone, 50.0);
        assert!(!scorch.target_air);
        assert!(scorch.target_ground);
        assert_eq!(scorch.ammo_use_effect, "none");
        assert_eq!(scorch.base.health, 400);
        assert_eq!(scorch.shoot_sound, "shootFlame");
        assert!(scorch.consume_coolant);
        assert_eq!(scorch.coolant_amount, 0.1);
        assert_eq!(scorch.deposit_cooldown, 1.0);
        assert_eq!(scorch.fog_radius, 8.0);
        assert_eq!(scorch.place_overlap_range, 60.0 + 8.0 * 7.0);
        assert_eq!(
            scorch.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 25
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 22
                }
            ]
        );

        let coal = &ammo_for(scorch, item_id("coal")).bullet;
        assert_eq!(coal.kind, BulletKind::Generic);
        assert_eq!(coal.speed, 3.35);
        assert_eq!(coal.damage, 17.0);
        assert_eq!(coal.ammo_multiplier, 3.0);
        assert_eq!(coal.hit_size, 7.0);
        assert_eq!(coal.lifetime, 18.0);
        assert!(coal.pierce);
        assert!(!coal.collides_air);
        assert_eq!(coal.status, "burning");
        assert_eq!(coal.status_duration, 60.0 * 4.0);
        assert_eq!(coal.shoot_effect, "shootSmallFlame");
        assert_eq!(coal.hit_effect, "hitFlameSmall");
        assert_eq!(coal.despawn_effect, "none");
        assert!(!coal.hittable);

        let pyratite = &ammo_for(scorch, item_id("pyratite")).bullet;
        assert_eq!(pyratite.kind, BulletKind::Generic);
        assert_eq!(pyratite.speed, 4.0);
        assert_eq!(pyratite.damage, 30.0);
        assert_eq!(pyratite.ammo_multiplier, 10.0);
        assert_eq!(pyratite.hit_size, 7.0);
        assert_eq!(pyratite.lifetime, 18.0);
        assert!(pyratite.pierce);
        assert!(!pyratite.collides_air);
        assert_eq!(pyratite.status_duration, 60.0 * 10.0);
        assert_eq!(pyratite.shoot_effect, "shootPyraFlame");
        assert_eq!(pyratite.status, "burning");
        assert!(!pyratite.hittable);

        let hail = registry.get_turret_by_name("hail").unwrap();
        assert_eq!(hail.kind, TurretBlockKind::ItemTurret);
        assert!(!hail.target_air);
        assert!(hail.target_ground);
        assert_eq!(hail.reload, 60.0);
        assert_eq!(hail.recoil, 2.0);
        assert_eq!(hail.range, 235.0);
        assert_eq!(hail.inaccuracy, 1.0);
        assert_eq!(hail.shoot_cone, 10.0);
        assert_eq!(hail.base.health, 260);
        assert_eq!(hail.shoot_sound, "shootArtillerySmall");
        assert!(hail.consume_coolant);
        assert_eq!(hail.coolant_amount, 0.1);
        assert_eq!(hail.coolant_multiplier, 10.0);
        assert_eq!(hail.deposit_cooldown, 2.0);
        assert_eq!(hail.fog_radius, 29.0);
        assert_eq!(hail.place_overlap_range, 235.0 + 8.0 * 7.0);
        assert_eq!(
            hail.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 17
                }
            ]
        );

        let graphite = &ammo_for(hail, item_id("graphite")).bullet;
        assert_eq!(graphite.kind, BulletKind::Artillery);
        assert_eq!(graphite.speed, 3.0);
        assert_eq!(graphite.damage, 20.0);
        assert_eq!(graphite.knockback, 0.8);
        assert_close(graphite.lifetime, (235.0 + 10.0) / 3.0);
        assert_eq!(graphite.width, 11.0);
        assert_eq!(graphite.height, 11.0);
        assert!(!graphite.collides_tiles);
        assert!(!graphite.collides_air);
        assert_eq!(graphite.splash_damage_radius, 25.0 * 0.75);
        assert_eq!(graphite.splash_damage, 33.0);
        assert_eq!(graphite.front_color, "graphiteAmmoFront");
        assert_eq!(graphite.back_color, "graphiteAmmoBack");
        assert_eq!(graphite.despawn_effect, "hitBulletColor");
        assert_eq!(graphite.trail_effect, "artilleryTrail");

        let silicon = &ammo_for(hail, item_id("silicon")).bullet;
        assert_eq!(silicon.kind, BulletKind::Artillery);
        assert_eq!(silicon.reload_multiplier, 1.2);
        assert_eq!(silicon.ammo_multiplier, 3.0);
        assert_eq!(silicon.homing_power, 0.08);
        assert_eq!(silicon.homing_range, 50.0);
        assert_eq!(silicon.trail_length, 7);
        assert_eq!(silicon.trail_width, 3.0);
        assert_eq!(silicon.front_color, "siliconAmmoFront");
        assert_close(silicon.lifetime, (235.0 + 10.0) / 3.0);

        let pyratite = &ammo_for(hail, item_id("pyratite")).bullet;
        assert_eq!(pyratite.kind, BulletKind::Artillery);
        assert_eq!(pyratite.damage, 25.0);
        assert_eq!(pyratite.hit_effect, "blastExplosion");
        assert_eq!(pyratite.width, 13.0);
        assert_eq!(pyratite.height, 13.0);
        assert_eq!(pyratite.splash_damage_radius, 25.0 * 0.75);
        assert_eq!(pyratite.splash_damage, 45.0);
        assert_eq!(pyratite.status, "burning");
        assert_eq!(pyratite.status_duration, 60.0 * 12.0);
        assert_eq!(pyratite.front_color, "lightishOrange");
        assert_eq!(pyratite.back_color, "lightOrange");
        assert!(pyratite.make_fire);
        assert_eq!(pyratite.trail_effect, "incendTrail");
        assert_eq!(pyratite.ammo_multiplier, 4.0);
        assert_eq!(pyratite.despawn_effect, "hitBulletColor");
        assert_close(pyratite.lifetime, (235.0 + 10.0) / 3.0);
    }

    #[test]
    fn liquid_and_power_turrets_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_content_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name.as_str() == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };
        fn liquid_ammo_for(turret: &TurretBlockData, liquid: ContentId) -> &LiquidTurretAmmo {
            turret
                .liquid_ammo
                .iter()
                .find(|ammo| ammo.liquid == liquid)
                .unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let wave = registry.get_turret_by_name("wave").unwrap();
        assert_eq!(wave.kind, TurretBlockKind::LiquidTurret);
        assert!(wave.base.has_liquids);
        assert_eq!(wave.loop_sound, "loopSpray");
        assert_eq!(wave.shoot_sound, "none");
        assert_eq!(wave.smoke_effect, "none");
        assert_eq!(wave.shoot_effect, "shootLiquid");
        assert!(wave.extinguish);
        assert_eq!(
            wave.base.flags,
            vec![BlockFlag::Turret, BlockFlag::Extinguisher]
        );
        assert_eq!(wave.base.size, 2);
        assert_eq!(wave.recoil, 0.0);
        assert_eq!(wave.reload, 3.0);
        assert_eq!(wave.inaccuracy, 5.0);
        assert_eq!(wave.shoot_cone, 50.0);
        assert_eq!(wave.liquid_capacity, 10.0);
        assert_eq!(wave.base.liquid_capacity, 10.0);
        assert_eq!(wave.range, 110.0);
        assert_eq!(wave.scaled_health, 250.0);
        assert_eq!(wave.base.health, 2 * 2 * 250);
        assert_eq!(wave.fog_radius, 14.0);
        assert_eq!(wave.place_overlap_range, 110.0 + 8.0 * 7.0);
        assert_eq!(
            wave.requirements,
            vec![
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 45
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 75
                },
                ItemAmount {
                    item: item_id("copper"),
                    amount: 25
                }
            ]
        );

        let water = &liquid_ammo_for(wave, liquid_content_id("water")).bullet;
        assert_eq!(water.kind, BulletKind::Liquid);
        assert_eq!(water.speed, 3.5);
        assert_eq!(water.damage, 0.0);
        assert_eq!(water.ammo_multiplier, 1.0);
        assert_eq!(water.lifetime, 34.0);
        assert_eq!(water.status_duration, 60.0 * 2.0);
        assert_eq!(water.despawn_effect, "none");
        assert_eq!(water.hit_effect, "hitLiquid");
        assert_eq!(water.smoke_effect, "none");
        assert_eq!(water.shoot_effect, "none");
        assert_eq!(water.knockback, 0.7);
        assert_eq!(water.drag, 0.01);
        assert_eq!(water.layer, "Layer.bullet-2");
        assert!(!water.display_ammo_multiplier);
        assert_eq!(water.puddle_size, 6.0);
        assert_eq!(water.orb_size, 3.0);
        assert_eq!(water.boil_time, 5.0);
        assert_eq!(water.status, "wet");
        assert_eq!(water.hit_color, "596ab8ff");

        let slag = &liquid_ammo_for(wave, liquid_content_id("slag")).bullet;
        assert_eq!(slag.damage, 4.0);
        assert_eq!(slag.drag, 0.01);
        assert_eq!(slag.status, "melting");
        assert_eq!(slag.light_color, "f0511d66");

        let cryofluid = &liquid_ammo_for(wave, liquid_content_id("cryofluid")).bullet;
        assert_eq!(cryofluid.drag, 0.01);
        assert_eq!(cryofluid.status, "freezing");

        let oil = &liquid_ammo_for(wave, liquid_content_id("oil")).bullet;
        assert_eq!(oil.drag, 0.01);
        assert_eq!(oil.layer, "Layer.bullet-2");
        assert_eq!(oil.status, "tarred");

        let lancer = registry.get_turret_by_name("lancer").unwrap();
        assert_eq!(lancer.kind, TurretBlockKind::PowerTurret);
        assert!(lancer.base.has_power);
        assert_eq!(lancer.range, 165.0);
        assert_eq!(lancer.shoot_first_shot_delay, 40.0);
        assert_eq!(lancer.recoil, 2.0);
        assert_eq!(lancer.reload, 80.0);
        assert_eq!(lancer.shake, 2.0);
        assert_eq!(lancer.shoot_effect, "lancerLaserShoot");
        assert_eq!(lancer.smoke_effect, "none");
        assert_eq!(lancer.heat_color, "red");
        assert_eq!(lancer.base.size, 2);
        assert_eq!(lancer.scaled_health, 280.0);
        assert_eq!(lancer.base.health, 2 * 2 * 280);
        assert!(!lancer.target_air);
        assert!(!lancer.move_while_charging);
        assert!(!lancer.accurate_delay);
        assert_eq!(lancer.shoot_sound, "shootLancer");
        assert!(lancer.consume_coolant);
        assert_eq!(lancer.coolant_amount, 0.2);
        assert_eq!(lancer.charge_sound, "chargeLancer");
        assert_eq!(lancer.consume_power, 6.0);
        assert_eq!(
            lancer.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 70
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 30
                }
            ]
        );
        let laser = lancer.shoot_type.as_ref().unwrap();
        assert_eq!(laser.kind, BulletKind::Laser);
        assert_eq!(laser.speed, 0.0);
        assert_eq!(laser.damage, 140.0);
        assert_eq!(laser.colors, vec!["a9d8ff66", "a9d8ffff", "ffffffff"]);
        assert_eq!(
            laser.charge_effect,
            "MultiEffect(lancerLaserCharge, lancerLaserChargeBegin)"
        );
        assert_eq!(laser.building_damage_multiplier, 0.25);
        assert_eq!(laser.armor_multiplier, 4.0);
        assert_eq!(laser.hit_effect, "hitLancer");
        assert_eq!(laser.hit_size, 4.0);
        assert_eq!(laser.lifetime, 16.0);
        assert_eq!(laser.draw_size, 400.0);
        assert!(!laser.collides_air);
        assert_eq!(laser.length, 173.0);
        assert_eq!(laser.ammo_multiplier, 1.0);
        assert_eq!(laser.pierce_cap, 4);
        assert!(!laser.collides);
        assert!(laser.pierce);
        assert!(!laser.hittable);
        assert!(!laser.keep_velocity);
        assert!(!laser.absorbable);

        let arc = registry.get_turret_by_name("arc").unwrap();
        assert_eq!(arc.kind, TurretBlockKind::PowerTurret);
        assert!(arc.base.has_power);
        assert_close(arc.research_cost_multiplier, 1.0 / 3.0);
        assert_eq!(arc.reload, 35.0);
        assert_eq!(arc.shoot_cone, 40.0);
        assert_eq!(arc.rotate_speed, 8.0);
        assert!(!arc.target_air);
        assert_eq!(arc.range, 90.0);
        assert_eq!(arc.shoot_effect, "lightningShoot");
        assert_eq!(arc.heat_color, "red");
        assert_eq!(arc.recoil, 1.0);
        assert_eq!(arc.base.size, 1);
        assert_eq!(arc.base.health, 260);
        assert_eq!(arc.shoot_sound, "shootArc");
        assert_close(arc.consume_power, 3.3);
        assert!(arc.consume_coolant);
        assert_eq!(arc.coolant_amount, 0.1);
        assert_eq!(
            arc.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 50
                }
            ]
        );
        let lightning = arc.shoot_type.as_ref().unwrap();
        assert_eq!(lightning.kind, BulletKind::Lightning);
        assert_eq!(lightning.damage, 20.0);
        assert_eq!(lightning.lifetime, 1.0);
        assert_eq!(lightning.despawn_effect, "none");
        assert_eq!(lightning.hit_effect, "hitLancer");
        assert!(!lightning.keep_velocity);
        assert!(!lightning.hittable);
        assert_eq!(lightning.status, "shocked");
        assert_eq!(lightning.lightning_length, 25);
        assert_eq!(lightning.lightning_length_rand, 0);
        assert_eq!(lightning.lightning_color, "a9d8ffff");
        assert!(!lightning.collides_air);
        assert_eq!(lightning.ammo_multiplier, 1.0);
        assert_eq!(lightning.building_damage_multiplier, 0.25);

        let nested = lightning.lightning_type.as_ref().unwrap();
        assert_eq!(nested.kind, BulletKind::Generic);
        assert_eq!(nested.speed, 0.0001);
        assert_eq!(nested.damage, 0.0);
        assert_eq!(nested.lifetime, 10.0);
        assert_eq!(nested.hit_effect, "hitLancer");
        assert_eq!(nested.despawn_effect, "none");
        assert_eq!(nested.status, "shocked");
        assert!(!nested.hittable);
        assert_eq!(nested.light_color, "ffffffff");
        assert!(!nested.collides_air);
        assert_eq!(nested.building_damage_multiplier, 0.25);
        assert_eq!(nested.shield_damage_multiplier, 0.2);
    }

    #[test]
    fn tractor_beam_turrets_keep_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let parallax = registry.get_turret_by_name("parallax").unwrap();
        assert_eq!(parallax.kind, TurretBlockKind::TractorBeamTurret);
        assert!(parallax.base.has_power);
        assert!(parallax.base.update);
        assert!(parallax.base.solid);
        assert_eq!(parallax.base.group, BlockGroup::Turrets);
        assert_eq!(parallax.base.flags, vec![BlockFlag::Turret]);
        assert_ne!(parallax.base.env_enabled & Env::SPACE, 0);
        assert!(parallax.target_air);
        assert!(!parallax.target_ground);
        assert_eq!(parallax.retarget_time, 5.0);
        assert_eq!(parallax.shoot_cone, 6.0);
        assert_eq!(parallax.shoot_length, 5.0);
        assert_eq!(parallax.laser_width, 0.6);
        assert_eq!(parallax.laser_color, "white");
        assert_eq!(parallax.status, "none");
        assert_eq!(parallax.status_duration, 300.0);
        assert_eq!(parallax.shoot_sound, "beamParallax");
        assert_eq!(parallax.shoot_sound_volume, 0.9);
        assert_eq!(parallax.coolant_multiplier, 1.0);
        assert_eq!(parallax.base.size, 2);
        assert_eq!(parallax.force, 16.0);
        assert_eq!(parallax.scaled_force, 9.0);
        assert_eq!(parallax.range, 300.0);
        assert_eq!(parallax.damage, 0.5);
        assert_eq!(parallax.scaled_health, 160.0);
        assert_eq!(parallax.base.health, 2 * 2 * 160);
        assert_eq!(parallax.rotate_speed, 12.0);
        assert_close(parallax.consume_power, 3.3);
        assert_eq!(parallax.fog_radius, 38.0);
        assert_eq!(parallax.place_overlap_range, 300.0 + 8.0 * 7.0);
        assert_eq!(
            parallax.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 160
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 110
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 50
                }
            ]
        );
    }

    #[test]
    fn salvo_item_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let salvo = registry.get_turret_by_name("salvo").unwrap();
        assert_eq!(salvo.kind, TurretBlockKind::ItemTurret);
        assert!(salvo.base.has_items);
        assert_eq!(salvo.base.size, 2);
        assert_eq!(salvo.range, 190.0);
        assert_eq!(salvo.reload, 29.0);
        assert!(!salvo.consume_ammo_once);
        assert_eq!(salvo.ammo_eject_back, 3.0);
        assert_eq!(salvo.recoil, 0.0);
        assert_eq!(salvo.shake, 1.0);
        assert_eq!(salvo.shoot_pattern, "ShootPattern");
        assert_eq!(salvo.shoot_shots, 4);
        assert_eq!(salvo.shoot_shot_delay, 3.0);
        assert_eq!(salvo.ammo_use_effect, "casing2");
        assert_eq!(salvo.scaled_health, 240.0);
        assert_eq!(salvo.base.health, 2 * 2 * 240);
        assert_eq!(salvo.shoot_sound, "shootSalvo");
        assert!(salvo.consume_coolant);
        assert_eq!(salvo.coolant_amount, 0.2);
        assert_eq!(salvo.deposit_cooldown, 2.0);
        assert_eq!(salvo.fog_radius, 24.0);
        assert_eq!(salvo.place_overlap_range, 190.0 + 32.0 + 8.0 * 7.0);
        assert!(salvo.drawer.contains("RegionPart(-side"));
        assert_eq!(
            salvo.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 50
                }
            ]
        );

        let copper = &ammo_for(salvo, item_id("copper")).bullet;
        assert_eq!(copper.kind, BulletKind::Basic);
        assert_eq!(copper.speed, 2.5);
        assert_eq!(copper.damage, 15.0);
        assert_eq!(copper.width, 7.0);
        assert_eq!(copper.height, 9.0);
        assert_close(copper.lifetime, (190.0 + 9.0 + 10.0) / 2.5);
        assert_eq!(copper.ammo_multiplier, 5.0);
        assert_eq!(copper.armor_multiplier, 1.5);
        assert_eq!(copper.hit_effect, "hitBulletColor");
        assert_eq!(copper.despawn_effect, "hitBulletColor");
        assert_eq!(copper.front_color, "copperAmmoFront");
        assert_eq!(copper.back_color, "copperAmmoBack");

        let graphite = &ammo_for(salvo, item_id("graphite")).bullet;
        assert_eq!(graphite.damage, 31.0);
        assert_eq!(graphite.width, 9.0);
        assert_eq!(graphite.height, 12.0);
        assert_eq!(graphite.ammo_multiplier, 4.0);
        assert_eq!(graphite.reload_multiplier, 0.8);
        assert_eq!(graphite.range_change, 32.0);
        assert_close(graphite.lifetime, (190.0 + 32.0 + 9.0 + 10.0) / 3.5);
        assert_eq!(graphite.front_color, "graphiteAmmoFront");
        assert_eq!(graphite.back_color, "graphiteAmmoBack");

        let pyratite = &ammo_for(salvo, item_id("pyratite")).bullet;
        assert_eq!(pyratite.speed, 3.2);
        assert_eq!(pyratite.damage, 25.0);
        assert_eq!(pyratite.width, 10.0);
        assert_eq!(pyratite.height, 12.0);
        assert_eq!(pyratite.front_color, "lightishOrange");
        assert_eq!(pyratite.hit_color, "lightishOrange");
        assert_eq!(pyratite.back_color, "lightOrange");
        assert_eq!(pyratite.status, "burning");
        assert_eq!(pyratite.hit_effect, "MultiEffect(hitBulletColor, fireHit)");
        assert_eq!(pyratite.ammo_multiplier, 5.0);
        assert_eq!(pyratite.splash_damage, 15.0);
        assert_eq!(pyratite.splash_damage_radius, 22.0);
        assert!(pyratite.make_fire);
        assert_close(pyratite.lifetime, (190.0 + 9.0 + 10.0) / 3.2);

        let silicon = &ammo_for(salvo, item_id("silicon")).bullet;
        assert_eq!(silicon.speed, 3.0);
        assert_eq!(silicon.damage, 23.0);
        assert_eq!(silicon.width, 8.0);
        assert_eq!(silicon.height, 10.0);
        assert_eq!(silicon.homing_power, 0.2);
        assert_eq!(silicon.reload_multiplier, 1.5);
        assert_eq!(silicon.ammo_multiplier, 5.0);
        assert_eq!(silicon.trail_length, 5);
        assert_eq!(silicon.trail_width, 1.5);
        assert_close(silicon.lifetime, (190.0 + 9.0 + 10.0) / 3.0);

        let thorium = &ammo_for(salvo, item_id("thorium")).bullet;
        assert_eq!(thorium.speed, 4.0);
        assert_eq!(thorium.damage, 28.0);
        assert_eq!(thorium.width, 8.0);
        assert_eq!(thorium.height, 13.0);
        assert_eq!(thorium.shoot_effect, "shootBig");
        assert_eq!(thorium.smoke_effect, "shootBigSmoke");
        assert_eq!(thorium.ammo_multiplier, 4.0);
        assert_eq!(thorium.armor_multiplier, 0.8);
        assert_eq!(thorium.front_color, "thoriumAmmoFront");
        assert_eq!(thorium.back_color, "thoriumAmmoBack");
        assert_close(thorium.lifetime, (190.0 + 9.0 + 10.0) / 4.0);
    }

    #[test]
    fn swarmer_missile_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let swarmer = registry.get_turret_by_name("swarmer").unwrap();
        assert_eq!(swarmer.kind, TurretBlockKind::ItemTurret);
        assert!(swarmer.base.has_items);
        assert_eq!(swarmer.shoot_pattern, "ShootBarrel");
        assert_eq!(
            swarmer.shoot_barrels,
            vec![
                ShootBarrel {
                    x: -4.0,
                    y: -1.25,
                    rotation: 0.0
                },
                ShootBarrel {
                    x: 0.0,
                    y: 0.0,
                    rotation: 0.0
                },
                ShootBarrel {
                    x: 4.0,
                    y: -1.25,
                    rotation: 0.0
                }
            ]
        );
        assert_eq!(swarmer.shoot_shots, 4);
        assert_eq!(swarmer.shoot_shot_delay, 5.0);
        assert_eq!(swarmer.shoot_y, 4.5);
        assert_close(swarmer.reload, 60.0 * 4.0 / 7.0);
        assert_eq!(swarmer.inaccuracy, 10.0);
        assert_eq!(swarmer.range, 240.0);
        assert!(!swarmer.consume_ammo_once);
        assert_eq!(swarmer.base.size, 2);
        assert_eq!(swarmer.scaled_health, 300.0);
        assert_eq!(swarmer.base.health, 2 * 2 * 300);
        assert_eq!(swarmer.shoot_sound, "shootMissile");
        assert_ne!(swarmer.base.env_enabled & Env::SPACE, 0);
        assert!(swarmer.consume_coolant);
        assert_eq!(swarmer.coolant_amount, 0.3);
        assert_eq!(swarmer.deposit_cooldown, 2.0);
        assert_eq!(swarmer.fog_radius, 30.0);
        assert_eq!(swarmer.place_overlap_range, 240.0 + 8.0 * 7.0);
        assert_eq!(
            swarmer.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 35
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 35
                },
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 45
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 30
                }
            ]
        );

        let blast = &ammo_for(swarmer, item_id("blast-compound")).bullet;
        assert_eq!(blast.kind, BulletKind::Missile);
        assert_eq!(blast.speed, 3.7);
        assert_eq!(blast.damage, 10.0);
        assert_eq!(blast.width, 8.0);
        assert_eq!(blast.height, 8.0);
        assert_eq!(blast.shrink_y, 0.0);
        assert_eq!(blast.homing_power, 0.08);
        assert_eq!(blast.splash_damage_radius, 30.0);
        assert_eq!(blast.splash_damage, 30.0 * 1.5);
        assert_eq!(blast.ammo_multiplier, 5.0);
        assert_eq!(blast.hit_effect, "blastExplosion");
        assert_eq!(blast.despawn_effect, "blastExplosion");
        assert_eq!(blast.status, "blasted");
        assert_eq!(blast.front_color, "blastAmmoFront");
        assert_eq!(blast.back_color, "blastAmmoBack");
        assert_close(blast.lifetime, (240.0 + 5.0 + 10.0) / 3.7);

        let pyratite = &ammo_for(swarmer, item_id("pyratite")).bullet;
        assert_eq!(pyratite.kind, BulletKind::Missile);
        assert_eq!(pyratite.damage, 12.0);
        assert_eq!(pyratite.front_color, "lightishOrange");
        assert_eq!(pyratite.back_color, "lightOrange");
        assert_eq!(pyratite.width, 7.0);
        assert_eq!(pyratite.height, 8.0);
        assert_eq!(pyratite.homing_power, 0.08);
        assert_eq!(pyratite.splash_damage_radius, 20.0);
        assert_eq!(pyratite.splash_damage, 30.0 * 1.5);
        assert!(pyratite.make_fire);
        assert_eq!(pyratite.ammo_multiplier, 5.0);
        assert_eq!(pyratite.hit_effect, "blastExplosion");
        assert_eq!(pyratite.status, "burning");
        assert_close(pyratite.lifetime, (240.0 + 5.0 + 10.0) / 3.7);

        let surge = &ammo_for(swarmer, item_id("surge-alloy")).bullet;
        assert_eq!(surge.kind, BulletKind::Missile);
        assert_eq!(surge.damage, 18.0);
        assert_eq!(surge.width, 8.0);
        assert_eq!(surge.height, 8.0);
        assert_eq!(surge.splash_damage_radius, 25.0);
        assert_eq!(surge.splash_damage, 25.0 * 1.4);
        assert_eq!(surge.hit_effect, "blastExplosion");
        assert_eq!(surge.despawn_effect, "blastExplosion");
        assert_eq!(surge.ammo_multiplier, 4.0);
        assert_eq!(surge.lightning_damage, 10.0);
        assert_eq!(surge.lightning, 2);
        assert_eq!(surge.lightning_length, 10);
        assert_eq!(surge.front_color, "surgeAmmoFront");
        assert_eq!(surge.back_color, "surgeAmmoBack");
        assert_close(surge.lifetime, (240.0 + 5.0 + 10.0) / 3.7);
    }

    #[test]
    fn point_defense_turrets_keep_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let segment = registry.get_turret_by_name("segment").unwrap();
        assert_eq!(segment.kind, TurretBlockKind::PointDefenseTurret);
        assert!(segment.base.has_power);
        assert!(segment.base.update);
        assert!(segment.base.solid);
        assert_eq!(segment.base.group, BlockGroup::Turrets);
        assert_eq!(segment.base.flags, vec![BlockFlag::Turret]);
        assert_ne!(segment.base.env_enabled & Env::SPACE, 0);
        assert_eq!(segment.rotate_speed, 20.0);
        assert_eq!(segment.retarget_time, 5.0);
        assert_eq!(segment.color, "white");
        assert_eq!(segment.beam_effect, "pointBeam");
        assert_eq!(segment.point_hit_effect, "pointHit");
        assert_eq!(segment.shoot_effect, "sparkShoot");
        assert_eq!(segment.shoot_sound, "shootSegment");
        assert_eq!(segment.shoot_cone, 5.0);
        assert_eq!(segment.coolant_multiplier, 2.0);
        assert_eq!(segment.scaled_health, 250.0);
        assert_eq!(segment.base.health, 2 * 2 * 250);
        assert_eq!(segment.range, 180.0);
        assert_close(segment.consume_power, 8.0);
        assert_eq!(segment.base.size, 2);
        assert_eq!(segment.shoot_length, 5.0);
        assert_eq!(segment.bullet_damage, 30.0);
        assert_eq!(segment.reload, 8.0);
        assert_eq!(segment.fog_radius, 23.0);
        assert_eq!(segment.place_overlap_range, 180.0 + 8.0 * 7.0);
        assert_eq!(
            segment.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 130
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("phase-fabric"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 40
                }
            ]
        );
    }

    #[test]
    fn shrapnel_item_turrets_keep_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }

        let fuse = registry.get_turret_by_name("fuse").unwrap();
        assert_eq!(fuse.kind, TurretBlockKind::ItemTurret);
        assert_eq!(fuse.reload, 35.0);
        assert_eq!(fuse.shake, 4.0);
        assert_eq!(fuse.range, 90.0);
        assert_eq!(fuse.recoil, 5.0);
        assert_eq!(fuse.shoot_pattern, "ShootSpread");
        assert_eq!(fuse.shoot_shots, 3);
        assert_eq!(fuse.shoot_spread, 20.0);
        assert_eq!(fuse.shoot_cone, 30.0);
        assert_eq!(fuse.base.size, 3);
        assert_ne!(fuse.base.env_enabled & Env::SPACE, 0);
        assert_eq!(fuse.scaled_health, 220.0);
        assert_eq!(fuse.base.health, 3 * 3 * 220);
        assert_eq!(fuse.shoot_sound, "shootFuse");
        assert_eq!(fuse.shoot_sound_volume, 0.9);
        assert!(fuse.consume_coolant);
        assert_eq!(fuse.coolant_amount, 0.3);
        assert_eq!(fuse.deposit_cooldown, 1.0);
        assert_eq!(fuse.fog_radius, 11.0);
        assert_eq!(fuse.place_overlap_range, 90.0 + 8.0 * 7.0);
        assert_eq!(
            fuse.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 225
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 225
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 100
                }
            ]
        );

        let titanium = &ammo_for(fuse, item_id("titanium")).bullet;
        assert_eq!(titanium.kind, BulletKind::Shrapnel);
        assert_eq!(titanium.speed, 0.0);
        assert_eq!(titanium.length, 100.0);
        assert_eq!(titanium.damage, 66.0);
        assert_eq!(titanium.ammo_multiplier, 4.0);
        assert_eq!(titanium.width, 17.0);
        assert_eq!(titanium.reload_multiplier, 1.3);
        assert_eq!(titanium.from_color, "white");
        assert_eq!(titanium.to_color, "lancerLaser");
        assert!(!titanium.hit_large);
        assert_eq!(titanium.serrations, 7);
        assert_eq!(titanium.serration_len_scl, 10.0);
        assert_eq!(titanium.serration_width, 4.0);
        assert_eq!(titanium.serration_spacing, 8.0);
        assert_eq!(titanium.serration_space_offset, 80.0);
        assert_eq!(titanium.serration_fade_offset, 0.5);
        assert_eq!(titanium.hit_effect, "hitLancer");
        assert_eq!(titanium.shoot_effect, "lightningShoot");
        assert_eq!(titanium.smoke_effect, "lightningShoot");
        assert_eq!(titanium.lifetime, 10.0);
        assert_eq!(titanium.despawn_effect, "none");
        assert!(!titanium.keep_velocity);
        assert!(!titanium.collides);
        assert!(titanium.pierce);
        assert!(!titanium.hittable);
        assert!(!titanium.absorbable);

        let thorium = &ammo_for(fuse, item_id("thorium")).bullet;
        assert_eq!(thorium.kind, BulletKind::Shrapnel);
        assert_eq!(thorium.length, 100.0);
        assert_eq!(thorium.damage, 105.0);
        assert_eq!(thorium.ammo_multiplier, 5.0);
        assert_eq!(thorium.to_color, "thoriumPink");
        assert_eq!(thorium.shoot_effect, "thoriumShoot");
        assert_eq!(thorium.smoke_effect, "thoriumShoot");
    }

    #[test]
    fn ripple_artillery_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let ripple = registry.get_turret_by_name("ripple").unwrap();
        assert_eq!(ripple.kind, TurretBlockKind::ItemTurret);
        assert_eq!(ripple.base.size, 3);
        assert!(!ripple.target_air);
        assert!(ripple.target_ground);
        assert_eq!(ripple.shoot_shots, 4);
        assert_eq!(ripple.inaccuracy, 11.0);
        assert_eq!(ripple.reload, 120.0);
        assert_eq!(ripple.ammo_eject_back, 5.0);
        assert_eq!(ripple.ammo_use_effect, "casing3Double");
        assert_eq!(ripple.ammo_per_shot, 2);
        assert_eq!(ripple.velocity_rnd, 0.2);
        assert_close(ripple.scale_lifetime_offset, 1.0 / 9.0);
        assert_eq!(ripple.recoil, 6.0);
        assert_eq!(ripple.shake, 2.0);
        assert_eq!(ripple.range, 290.0);
        assert_eq!(ripple.min_range, 50.0);
        assert!(ripple.consume_coolant);
        assert_eq!(ripple.coolant_amount, 0.3);
        assert_eq!(ripple.scaled_health, 130.0);
        assert_eq!(ripple.base.health, 3 * 3 * 130);
        assert_eq!(ripple.deposit_cooldown, 2.0);
        assert_eq!(ripple.shoot_sound, "shootRipple");
        assert_eq!(ripple.fog_radius, 36.0);
        assert_eq!(ripple.place_overlap_range, 290.0 + 8.0 * 7.0);
        assert_eq!(
            ripple.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 135
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 60
                }
            ]
        );

        let graphite = &ammo_for(ripple, item_id("graphite")).bullet;
        assert_eq!(graphite.kind, BulletKind::Artillery);
        assert_eq!(graphite.speed, 3.0);
        assert_eq!(graphite.damage, 40.0);
        assert_eq!(
            graphite.hit_effect,
            "MultiEffect(flakExplosion, shockwaveSmaller)"
        );
        assert_eq!(graphite.shoot_effect, "shootBig");
        assert_eq!(graphite.trail_effect, "artilleryTrail");
        assert_eq!(graphite.knockback, 0.8);
        assert_eq!(graphite.lifetime, 80.0);
        assert_eq!(graphite.width, 12.0);
        assert_eq!(graphite.height, 14.0);
        assert!(!graphite.collides_tiles);
        assert!(!graphite.collides);
        assert!(!graphite.collides_air);
        assert!(graphite.scale_life);
        assert_eq!(graphite.shrink_x, 0.15);
        assert_eq!(graphite.shrink_y, 0.5);
        assert_eq!(graphite.splash_damage_radius, 30.0 * 0.75);
        assert_eq!(graphite.splash_damage, 70.0);
        assert_eq!(graphite.front_color, "graphiteAmmoFront");
        assert_eq!(graphite.back_color, "graphiteAmmoBack");
        assert_eq!(graphite.hit_color, "graphiteAmmoBack");
        assert_eq!(graphite.trail_color, "graphiteAmmoBack");
        assert_eq!(graphite.despawn_effect, "hitBulletColor");
        assert_eq!(graphite.life_scale_rand_min, 0.95);
        assert_eq!(graphite.life_scale_rand_max, 1.08);

        let silicon = &ammo_for(ripple, item_id("silicon")).bullet;
        assert_eq!(silicon.kind, BulletKind::Artillery);
        assert_eq!(silicon.reload_multiplier, 1.2);
        assert_eq!(silicon.ammo_multiplier, 3.0);
        assert_eq!(silicon.homing_power, 0.08);
        assert_eq!(silicon.homing_range, 50.0);
        assert_eq!(silicon.trail_length, 9);
        assert_eq!(silicon.trail_width, 3.1);
        assert_eq!(silicon.front_color, "siliconAmmoFront");
        assert_eq!(silicon.back_color, "siliconAmmoBack");
        assert_eq!(silicon.life_scale_rand_min, 0.95);
        assert_eq!(silicon.life_scale_rand_max, 1.08);

        let pyratite = &ammo_for(ripple, item_id("pyratite")).bullet;
        assert_eq!(pyratite.kind, BulletKind::Artillery);
        assert_eq!(pyratite.damage, 48.0);
        assert_eq!(
            pyratite.hit_effect,
            "MultiEffect(blastExplosion, shockwave)"
        );
        assert_eq!(pyratite.width, 13.0);
        assert_eq!(pyratite.height, 15.0);
        assert_eq!(pyratite.splash_damage_radius, 30.0 * 0.75);
        assert_eq!(pyratite.splash_damage, 90.0);
        assert_eq!(pyratite.status, "burning");
        assert_eq!(pyratite.status_duration, 60.0 * 12.0);
        assert_eq!(pyratite.front_color, "lightishOrange");
        assert_eq!(pyratite.back_color, "lightOrange");
        assert_eq!(pyratite.hit_color, "lightOrange");
        assert!(pyratite.make_fire);
        assert_eq!(pyratite.trail_effect, "incendTrail");
        assert_eq!(pyratite.ammo_multiplier, 4.0);
        assert_eq!(pyratite.despawn_effect, "hitBulletColor");

        let blast = &ammo_for(ripple, item_id("blast-compound")).bullet;
        assert_eq!(blast.kind, BulletKind::Artillery);
        assert_eq!(blast.speed, 2.0);
        assert_eq!(blast.damage, 40.0);
        assert_eq!(blast.hit_effect, "MultiEffect(blastExplosion, shockwave)");
        assert_eq!(blast.width, 14.0);
        assert_eq!(blast.height, 16.0);
        assert_eq!(blast.ammo_multiplier, 4.0);
        assert_eq!(blast.splash_damage_radius, 50.0 * 0.75);
        assert_eq!(blast.splash_damage, 90.0);
        assert_eq!(blast.status, "blasted");
        assert_eq!(blast.front_color, "blastAmmoFront");
        assert_eq!(blast.back_color, "blastAmmoBack");

        let plastanium = &ammo_for(ripple, item_id("plastanium")).bullet;
        assert_eq!(plastanium.kind, BulletKind::Artillery);
        assert_eq!(plastanium.speed, 3.4);
        assert_eq!(
            plastanium.hit_effect,
            "MultiEffect(plasticExplosion, shockwave)"
        );
        assert_eq!(plastanium.knockback, 1.0);
        assert_eq!(plastanium.width, 13.0);
        assert_eq!(plastanium.height, 15.0);
        assert_eq!(plastanium.splash_damage_radius, 40.0 * 0.75);
        assert_eq!(plastanium.splash_damage, 90.0);
        assert_eq!(plastanium.frag_bullets, 15);
        assert_eq!(plastanium.front_color, "plastaniumFront");
        assert_eq!(plastanium.back_color, "plastaniumBack");
        assert_eq!(plastanium.life_scale_rand_min, 0.95);
        assert_eq!(plastanium.life_scale_rand_max, 1.08);
        let frag = plastanium.frag_bullet.as_ref().unwrap();
        assert_eq!(frag.kind, BulletKind::Basic);
        assert_eq!(frag.speed, 2.5);
        assert_eq!(frag.damage, 14.0);
        assert_eq!(frag.width, 10.0);
        assert_eq!(frag.height, 12.0);
        assert_eq!(frag.shrink_y, 1.0);
        assert_eq!(frag.lifetime, 15.0);
        assert_eq!(frag.back_color, "plastaniumBack");
        assert_eq!(frag.front_color, "plastaniumFront");
        assert_eq!(frag.despawn_effect, "none");
        assert!(!frag.collides_air);
    }

    #[test]
    fn cyclone_flak_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let cyclone = registry.get_turret_by_name("cyclone").unwrap();
        assert_eq!(cyclone.kind, TurretBlockKind::ItemTurret);
        assert_eq!(cyclone.base.size, 3);
        assert_eq!(cyclone.shoot_y, 10.0);
        assert_eq!(cyclone.shoot_pattern, "ShootBarrel");
        assert_eq!(
            cyclone.shoot_barrels,
            vec![
                ShootBarrel {
                    x: 0.0,
                    y: 1.0,
                    rotation: 0.0
                },
                ShootBarrel {
                    x: 3.0,
                    y: 0.0,
                    rotation: 0.0
                },
                ShootBarrel {
                    x: -3.0,
                    y: 0.0,
                    rotation: 0.0
                }
            ]
        );
        assert_eq!(cyclone.recoils, 3);
        assert!(cyclone.drawer.contains("RegionPart(-barrel-3"));
        assert_eq!(cyclone.reload, 10.0);
        assert_eq!(cyclone.range, 200.0);
        assert_eq!(cyclone.recoil, 1.5);
        assert_eq!(cyclone.recoil_time, 10.0);
        assert_eq!(cyclone.rotate_speed, 10.0);
        assert_eq!(cyclone.inaccuracy, 10.0);
        assert_eq!(cyclone.shoot_cone, 30.0);
        assert_eq!(cyclone.shoot_sound, "shootCyclone");
        assert!(cyclone.consume_coolant);
        assert_eq!(cyclone.coolant_amount, 0.3);
        assert_eq!(cyclone.scaled_health, 145.0);
        assert_eq!(cyclone.base.health, 3 * 3 * 145);
        assert_eq!(cyclone.deposit_cooldown, 2.0);
        assert_eq!(cyclone.fog_radius, 25.0);
        assert_eq!(cyclone.place_overlap_range, 200.0 + 8.0 * 7.0);
        assert_eq!(
            cyclone.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 125
                },
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 80
                }
            ]
        );

        let metaglass = &ammo_for(cyclone, item_id("metaglass")).bullet;
        assert_eq!(metaglass.kind, BulletKind::Flak);
        assert_eq!(metaglass.speed, 4.0);
        assert_eq!(metaglass.damage, 6.0);
        assert_close(metaglass.lifetime, (200.0 + 9.0 + 10.0) / 4.0);
        assert_eq!(metaglass.ammo_multiplier, 2.0);
        assert_eq!(metaglass.reload_multiplier, 0.8);
        assert_eq!(metaglass.shoot_effect, "shootSmall");
        assert_eq!(metaglass.width, 6.0);
        assert_eq!(metaglass.height, 11.0);
        assert_eq!(metaglass.hit_effect, "flakExplosion");
        assert_eq!(metaglass.splash_damage, 45.0);
        assert_eq!(metaglass.splash_damage_radius, 25.0);
        assert_eq!(metaglass.frag_bullets, 4);
        assert_eq!(metaglass.explode_range, 20.0);
        assert_eq!(metaglass.explode_delay, 5.0);
        assert_eq!(metaglass.flak_interval, 6.0);
        assert!(metaglass.collides_ground);
        assert_eq!(metaglass.front_color, "glassAmmoFront");
        assert_eq!(metaglass.back_color, "glassAmmoBack");
        assert_eq!(metaglass.hit_color, "glassAmmoBack");
        assert_eq!(metaglass.trail_color, "glassAmmoBack");
        assert_eq!(metaglass.despawn_effect, "hitBulletColor");
        let frag = metaglass.frag_bullet.as_ref().unwrap();
        assert_eq!(frag.kind, BulletKind::Basic);
        assert_eq!(frag.speed, 3.0);
        assert_eq!(frag.damage, 12.0);
        assert_eq!(frag.width, 5.0);
        assert_eq!(frag.height, 12.0);
        assert_eq!(frag.shrink_y, 1.0);
        assert_eq!(frag.lifetime, 20.0);
        assert_eq!(frag.back_color, "gray");
        assert_eq!(frag.front_color, "white");
        assert_eq!(frag.despawn_effect, "none");

        let blast = &ammo_for(cyclone, item_id("blast-compound")).bullet;
        assert_eq!(blast.kind, BulletKind::Flak);
        assert_eq!(blast.speed, 4.0);
        assert_eq!(blast.damage, 8.0);
        assert_eq!(blast.shoot_effect, "shootBig");
        assert_eq!(blast.ammo_multiplier, 5.0);
        assert_eq!(blast.splash_damage, 45.0);
        assert_eq!(blast.splash_damage_radius, 60.0);
        assert!(blast.collides_ground);
        assert_eq!(blast.status, "blasted");
        assert_eq!(blast.front_color, "blastAmmoFront");
        assert_eq!(blast.back_color, "blastAmmoBack");
        assert_eq!(blast.despawn_effect, "hitBulletColor");
        assert_close(blast.lifetime, (200.0 + 9.0 + 10.0) / 4.0);

        let plastanium = &ammo_for(cyclone, item_id("plastanium")).bullet;
        assert_eq!(plastanium.kind, BulletKind::Flak);
        assert_eq!(plastanium.ammo_multiplier, 4.0);
        assert_eq!(plastanium.splash_damage_radius, 40.0);
        assert_eq!(plastanium.splash_damage, 37.5);
        assert_eq!(plastanium.frag_bullets, 6);
        assert_eq!(plastanium.hit_effect, "plasticExplosion");
        assert_eq!(plastanium.front_color, "plastaniumFront");
        assert_eq!(plastanium.back_color, "plastaniumBack");
        assert_eq!(plastanium.shoot_effect, "shootBig");
        assert!(plastanium.collides_ground);
        assert_eq!(plastanium.explode_range, 20.0);
        assert_eq!(plastanium.despawn_effect, "hitBulletColor");
        let frag = plastanium.frag_bullet.as_ref().unwrap();
        assert_eq!(frag.kind, BulletKind::Basic);
        assert_eq!(frag.speed, 2.5);
        assert_eq!(frag.damage, 12.0);
        assert_eq!(frag.width, 10.0);
        assert_eq!(frag.height, 12.0);
        assert_eq!(frag.shrink_y, 1.0);
        assert_eq!(frag.lifetime, 15.0);
        assert_eq!(frag.back_color, "plastaniumBack");
        assert_eq!(frag.front_color, "plastaniumFront");
        assert_eq!(frag.despawn_effect, "none");
        assert_close(plastanium.lifetime, (200.0 + 9.0 + 10.0) / 4.0);

        let surge = &ammo_for(cyclone, item_id("surge-alloy")).bullet;
        assert_eq!(surge.kind, BulletKind::Flak);
        assert_eq!(surge.speed, 4.5);
        assert_eq!(surge.damage, 13.0);
        assert_eq!(surge.ammo_multiplier, 5.0);
        assert_eq!(surge.splash_damage, 50.0 * 1.5);
        assert_eq!(surge.splash_damage_radius, 38.0);
        assert_eq!(surge.lightning, 2);
        assert_eq!(surge.lightning_length, 7);
        assert_eq!(surge.shoot_effect, "shootBig");
        assert!(surge.collides_ground);
        assert_eq!(surge.explode_range, 20.0);
        assert_eq!(surge.front_color, "surgeAmmoFront");
        assert_eq!(surge.back_color, "surgeAmmoBack");
        assert_eq!(surge.despawn_effect, "hitBulletColor");
        assert_close(surge.lifetime, (200.0 + 9.0 + 10.0) / 4.5);
    }

    #[test]
    fn foreshadow_rail_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let foreshadow = registry.get_turret_by_name("foreshadow").unwrap();
        assert_eq!(foreshadow.kind, TurretBlockKind::ItemTurret);
        assert!(foreshadow.base.has_items);
        assert_eq!(foreshadow.range, 500.0);
        assert_eq!(foreshadow.max_ammo, 40);
        assert_eq!(foreshadow.ammo_per_shot, 5);
        assert_eq!(foreshadow.rotate_speed, 2.0);
        assert_eq!(foreshadow.reload, 200.0);
        assert_eq!(foreshadow.ammo_use_effect, "casing3Double");
        assert_eq!(foreshadow.recoil, 5.0);
        assert_eq!(foreshadow.cooldown_time, 200.0);
        assert_eq!(foreshadow.shake, 4.0);
        assert_eq!(foreshadow.base.size, 4);
        assert_eq!(foreshadow.shoot_cone, 2.0);
        assert_eq!(foreshadow.shoot_sound, "shootForeshadow");
        assert_eq!(foreshadow.unit_sort, "strongest");
        assert_ne!(foreshadow.base.env_enabled & Env::SPACE, 0);
        assert_eq!(foreshadow.coolant_multiplier, 0.4);
        assert_eq!(foreshadow.liquid_capacity, 60.0);
        assert_eq!(foreshadow.base.liquid_capacity, 60.0);
        assert_eq!(foreshadow.scaled_health, 150.0);
        assert_eq!(foreshadow.base.health, 4 * 4 * 150);
        assert!(foreshadow.consume_coolant);
        assert_eq!(foreshadow.coolant_amount, 1.0);
        assert_eq!(foreshadow.deposit_cooldown, 2.0);
        assert_eq!(foreshadow.consume_power, 10.0);
        assert_eq!(foreshadow.fog_radius, 63.0);
        assert_eq!(foreshadow.place_overlap_range, 500.0 + 8.0 * 7.0);
        assert_eq!(
            foreshadow.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 1000
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 600
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 300
                },
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 600
                }
            ]
        );

        let rail = &ammo_for(foreshadow, item_id("surge-alloy")).bullet;
        assert_eq!(rail.kind, BulletKind::Rail);
        assert_eq!(rail.speed, 0.0);
        assert_eq!(rail.damage, 1350.0);
        assert_eq!(rail.shoot_effect, "instShoot");
        assert_eq!(rail.hit_effect, "instHit");
        assert_eq!(rail.pierce_effect, "railHit");
        assert_eq!(rail.smoke_effect, "smokeCloud");
        assert_eq!(rail.point_effect, "instTrail");
        assert_eq!(rail.despawn_effect, "instBomb");
        assert_eq!(rail.point_effect_space, 20.0);
        assert_close(rail.building_damage_multiplier, 0.2);
        assert_eq!(rail.pierce_damage_factor, 1.0);
        assert_eq!(rail.length, 500.0);
        assert_eq!(rail.hit_shake, 6.0);
        assert_eq!(rail.ammo_multiplier, 1.0);
        assert!(rail.pierce);
        assert!(rail.pierce_building);
        assert!(!rail.reflectable);
        assert!(!rail.collides);
        assert!(!rail.keep_velocity);
        assert_eq!(rail.lifetime, 1.0);
        assert!(rail.delay_frags);
    }

    #[test]
    fn spectre_item_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let spectre = registry.get_turret_by_name("spectre").unwrap();
        assert_eq!(spectre.kind, TurretBlockKind::ItemTurret);
        assert_eq!(spectre.reload, 7.0);
        assert_eq!(spectre.recoil_time, 14.0);
        assert_eq!(spectre.coolant_multiplier, 0.5);
        assert_eq!(spectre.liquid_capacity, 120.0);
        assert_eq!(spectre.base.liquid_capacity, 120.0);
        assert_eq!(spectre.ammo_use_effect, "casing3");
        assert_eq!(spectre.range, 260.0);
        assert_eq!(spectre.inaccuracy, 3.0);
        assert_eq!(spectre.recoil, 3.0);
        assert_eq!(spectre.shoot_pattern, "ShootAlternate");
        assert_eq!(spectre.shoot_alternate_spread, 8.0);
        assert_eq!(spectre.shake, 2.0);
        assert_eq!(spectre.base.size, 4);
        assert_eq!(spectre.shoot_cone, 24.0);
        assert_eq!(spectre.shoot_sound, "shootSpectre");
        assert_eq!(spectre.scaled_health, 160.0);
        assert_eq!(spectre.base.health, 4 * 4 * 160);
        assert!(spectre.consume_coolant);
        assert_eq!(spectre.coolant_amount, 1.0);
        assert_eq!(spectre.deposit_cooldown, 2.0);
        assert_eq!(spectre.fog_radius, 33.0);
        assert_eq!(spectre.place_overlap_range, 260.0 + 8.0 * 7.0);
        assert_eq!(
            spectre.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 900
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 300
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("plastanium"),
                    amount: 175
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 250
                }
            ]
        );

        let graphite = &ammo_for(spectre, item_id("graphite")).bullet;
        assert_eq!(graphite.kind, BulletKind::Basic);
        assert_eq!(graphite.speed, 7.5);
        assert_eq!(graphite.damage, 50.0);
        assert_eq!(graphite.hit_size, 4.8);
        assert_eq!(graphite.width, 15.0);
        assert_eq!(graphite.height, 21.0);
        assert_eq!(graphite.shrink_y, 0.5);
        assert_eq!(graphite.shoot_effect, "shootBig");
        assert_eq!(graphite.ammo_multiplier, 4.0);
        assert_eq!(graphite.reload_multiplier, 1.7);
        assert_eq!(graphite.knockback, 0.3);
        assert_eq!(graphite.hit_effect, "hitBulletColor");
        assert_eq!(graphite.despawn_effect, "hitBulletColor");
        assert_eq!(graphite.front_color, "graphiteAmmoFront");
        assert_eq!(graphite.back_color, "graphiteAmmoBack");
        assert_eq!(graphite.hit_color, "graphiteAmmoBack");
        assert_eq!(graphite.trail_color, "graphiteAmmoBack");
        assert_close(graphite.lifetime, (260.0 + 9.0 + 10.0) / 7.5);

        let thorium = &ammo_for(spectre, item_id("thorium")).bullet;
        assert_eq!(thorium.kind, BulletKind::Basic);
        assert_eq!(thorium.speed, 8.0);
        assert_eq!(thorium.damage, 80.0);
        assert_eq!(thorium.hit_size, 5.0);
        assert_eq!(thorium.width, 16.0);
        assert_eq!(thorium.height, 23.0);
        assert_eq!(thorium.shoot_effect, "shootBig");
        assert_eq!(thorium.pierce_cap, 2);
        assert!(thorium.pierce_building);
        assert_eq!(thorium.knockback, 0.7);
        assert_eq!(thorium.front_color, "thoriumAmmoFront");
        assert_eq!(thorium.back_color, "thoriumAmmoBack");
        assert_eq!(thorium.hit_color, "thoriumAmmoBack");
        assert_eq!(thorium.trail_color, "thoriumAmmoBack");
        assert_eq!(thorium.hit_effect, "hitBulletSmall");
        assert_eq!(thorium.despawn_effect, "hitBulletSmall");
        assert_close(thorium.lifetime, (260.0 + 9.0 + 10.0) / 8.0);

        let pyratite = &ammo_for(spectre, item_id("pyratite")).bullet;
        assert_eq!(pyratite.kind, BulletKind::Basic);
        assert_eq!(pyratite.speed, 7.0);
        assert_eq!(pyratite.damage, 70.0);
        assert_eq!(pyratite.hit_size, 5.0);
        assert_eq!(pyratite.width, 16.0);
        assert_eq!(pyratite.height, 21.0);
        assert_eq!(pyratite.front_color, "lightishOrange");
        assert_eq!(pyratite.back_color, "lightOrange");
        assert_eq!(pyratite.status, "burning");
        assert_eq!(pyratite.hit_effect, "MultiEffect(hitBulletSmall, fireHit)");
        assert_eq!(pyratite.shoot_effect, "shootBig");
        assert!(pyratite.make_fire);
        assert_eq!(pyratite.pierce_cap, 2);
        assert!(pyratite.pierce_building);
        assert_eq!(pyratite.knockback, 0.6);
        assert_eq!(pyratite.ammo_multiplier, 3.0);
        assert_eq!(pyratite.splash_damage, 20.0);
        assert_eq!(pyratite.splash_damage_radius, 25.0);
        assert_close(pyratite.lifetime, (260.0 + 9.0 + 10.0) / 7.0);
    }

    #[test]
    fn meltdown_laser_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let meltdown = registry.get_turret_by_name("meltdown").unwrap();
        assert_eq!(meltdown.kind, TurretBlockKind::LaserTurret);
        assert!(meltdown.base.has_power);
        assert_eq!(meltdown.shoot_effect, "shootBigSmoke2");
        assert_eq!(meltdown.shoot_cone, 40.0);
        assert_eq!(meltdown.recoil, 4.0);
        assert_eq!(meltdown.base.size, 4);
        assert_eq!(meltdown.shake, 2.0);
        assert_eq!(meltdown.range, 195.0);
        assert_eq!(meltdown.reload, 90.0);
        assert_eq!(meltdown.firing_move_fract, 0.5);
        assert_eq!(meltdown.shoot_duration, 230.0);
        assert_eq!(meltdown.shoot_sound, "shootMeltdown");
        assert_eq!(meltdown.loop_sound, "beamMeltdown");
        assert_eq!(meltdown.loop_sound_volume, 2.0);
        assert_ne!(meltdown.base.env_enabled & Env::SPACE, 0);
        assert_eq!(meltdown.scaled_health, 200.0);
        assert_eq!(meltdown.base.health, 4 * 4 * 200);
        assert_eq!(meltdown.liquid_capacity, 60.0);
        assert_eq!(meltdown.base.liquid_capacity, 60.0);
        assert!(meltdown.consume_coolant);
        assert_eq!(meltdown.coolant_amount, 0.5);
        assert_eq!(meltdown.coolant_multiplier, 1.0);
        assert_eq!(meltdown.consume_power, 17.0);
        assert_eq!(meltdown.fog_radius, 24.0);
        assert_eq!(meltdown.place_overlap_range, 195.0 + 8.0 * 7.0);
        assert_eq!(
            meltdown.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 1200
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 350
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 300
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 325
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 325
                }
            ]
        );

        let laser = meltdown.shoot_type.as_ref().unwrap();
        assert_eq!(laser.kind, BulletKind::ContinuousLaser);
        assert_eq!(laser.speed, 0.0);
        assert_eq!(laser.damage, 78.0);
        assert_eq!(laser.length, 200.0);
        assert_eq!(laser.hit_effect, "hitMeltdown");
        assert_eq!(laser.hit_color, "meltdownHit");
        assert_eq!(laser.status, "melting");
        assert_eq!(laser.draw_size, 420.0);
        assert!(laser.timescale_damage);
        assert_eq!(laser.incend_chance, 0.4);
        assert_eq!(laser.incend_spread, 5.0);
        assert_eq!(laser.incend_amount, 1);
        assert_eq!(laser.ammo_multiplier, 1.0);
        assert_eq!(laser.shake, 1.0);
        assert_eq!(laser.damage_interval, 5.0);
        assert!(laser.hit_large);
        assert!(laser.continuous);
        assert!(!laser.remove_after_pierce);
        assert_eq!(laser.despawn_effect, "none");
        assert_eq!(laser.shoot_effect, "none");
        assert_eq!(laser.lifetime, 16.0);
        assert!(laser.impact);
        assert!(!laser.keep_velocity);
        assert!(!laser.collides);
        assert!(laser.pierce);
        assert!(!laser.hittable);
        assert!(!laser.absorbable);
        assert_eq!(laser.hit_size, 4.0);
        assert_close(laser.width, 9.0);
    }

    #[test]
    fn breach_item_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let breach = registry.get_turret_by_name("breach").unwrap();
        assert_eq!(breach.kind, TurretBlockKind::ItemTurret);
        assert_eq!(breach.coolant_multiplier, 15.0);
        assert_eq!(breach.shoot_sound, "shootBreach");
        assert!(!breach.target_under_blocks);
        assert_eq!(breach.shake, 1.0);
        assert_eq!(breach.ammo_per_shot, 2);
        assert_eq!(breach.drawer, "DrawTurret(reinforced-)");
        assert_eq!(breach.shoot_y, -2.0);
        assert_eq!(breach.outline_color, "darkOutline");
        assert_eq!(breach.base.size, 3);
        assert_ne!(breach.base.env_enabled & Env::SPACE, 0);
        assert_eq!(breach.reload, 40.0);
        assert_eq!(breach.recoil, 2.0);
        assert_eq!(breach.range, 190.0);
        assert_eq!(breach.shoot_cone, 3.0);
        assert_eq!(breach.scaled_health, 180.0);
        assert_eq!(breach.base.health, 3 * 3 * 180);
        assert_eq!(breach.rotate_speed, 1.5);
        assert_eq!(breach.research_cost_multiplier, 0.05);
        assert_eq!(breach.build_time, 60.0 * 9.0);
        assert!(breach.consume_coolant);
        assert_close(breach.coolant_amount, 15.0 / 60.0);
        assert_eq!(breach.fog_radius, 24.0);
        assert_eq!(breach.place_overlap_range, 190.0 + 7.0 * 8.0 + 8.0 * 7.0);
        assert_eq!(
            breach.requirements,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 125
                }
            ]
        );

        let beryllium = &ammo_for(breach, item_id("beryllium")).bullet;
        assert_eq!(beryllium.kind, BulletKind::Basic);
        assert_eq!(beryllium.speed, 7.5);
        assert_eq!(beryllium.damage, 85.0);
        assert_eq!(beryllium.width, 12.0);
        assert_eq!(beryllium.hit_size, 7.0);
        assert_eq!(beryllium.height, 20.0);
        assert_eq!(
            beryllium.shoot_effect,
            "MultiEffect(shootBigColor, colorSparkBig)"
        );
        assert_eq!(beryllium.smoke_effect, "shootBigSmoke");
        assert_eq!(beryllium.ammo_multiplier, 1.0);
        assert_eq!(beryllium.pierce_cap, 2);
        assert!(beryllium.pierce);
        assert!(beryllium.pierce_building);
        assert_eq!(beryllium.front_color, "white");
        assert_eq!(beryllium.back_color, "berylShot");
        assert_eq!(beryllium.hit_color, "berylShot");
        assert_eq!(beryllium.trail_color, "berylShot");
        assert_eq!(beryllium.trail_width, 2.1);
        assert_eq!(beryllium.trail_length, 10);
        assert_eq!(beryllium.hit_effect, "hitBulletColor");
        assert_eq!(beryllium.despawn_effect, "hitBulletColor");
        assert_close(beryllium.building_damage_multiplier, 0.3);
        assert_close(beryllium.lifetime, (190.0 + 12.0 + 10.0) / 7.5);

        let tungsten = &ammo_for(breach, item_id("tungsten")).bullet;
        assert_eq!(tungsten.kind, BulletKind::Basic);
        assert_eq!(tungsten.speed, 8.0);
        assert_eq!(tungsten.damage, 95.0);
        assert_eq!(tungsten.width, 13.0);
        assert_eq!(tungsten.height, 19.0);
        assert_eq!(tungsten.hit_size, 7.0);
        assert_eq!(tungsten.ammo_multiplier, 2.0);
        assert_eq!(tungsten.reload_multiplier, 1.0);
        assert_eq!(tungsten.pierce_cap, 4);
        assert!(tungsten.pierce);
        assert!(tungsten.pierce_building);
        assert_eq!(tungsten.front_color, "white");
        assert_eq!(tungsten.back_color, "tungstenShot");
        assert_eq!(tungsten.trail_width, 2.2);
        assert_eq!(tungsten.trail_length, 11);
        assert_eq!(tungsten.range_change, 40.0);
        assert_close(tungsten.building_damage_multiplier, 0.3);
        assert_close(tungsten.lifetime, (190.0 + 40.0 + 12.0 + 10.0) / 8.0);

        let carbide = &ammo_for(breach, item_id("carbide")).bullet;
        assert_eq!(carbide.kind, BulletKind::Basic);
        assert_eq!(carbide.speed, 12.0);
        assert_close(carbide.damage, 325.0 / 0.75);
        assert_eq!(carbide.width, 15.0);
        assert_eq!(carbide.height, 21.0);
        assert_eq!(carbide.hit_size, 7.0);
        assert_eq!(carbide.ammo_multiplier, 2.0);
        assert_eq!(carbide.reload_multiplier, 0.2);
        assert_eq!(carbide.front_color, "white");
        assert_eq!(carbide.back_color, "ab8ec5");
        assert_eq!(carbide.trail_width, 2.2);
        assert_eq!(carbide.trail_length, 11);
        assert_eq!(carbide.trail_effect, "disperseTrail");
        assert_eq!(carbide.trail_interval, 2.0);
        assert_eq!(carbide.hit_effect, "hitBulletColor");
        assert_eq!(carbide.despawn_effect, "hitBulletColor");
        assert_eq!(carbide.range_change, 7.0 * 8.0);
        assert_close(carbide.building_damage_multiplier, 0.3);
        assert!(carbide.trail_rotation);
        assert_eq!(carbide.bullet_shoot_sound, "shootBreachCarbide");
        assert_eq!(carbide.frag_bullets, 3);
        assert_eq!(carbide.frag_random_spread, 0.0);
        assert_eq!(carbide.frag_spread, 25.0);
        assert_eq!(carbide.frag_velocity_min, 1.0);
        assert_close(carbide.lifetime, (190.0 + 7.0 * 8.0 + 12.0 + 10.0) / 12.0);

        let frag = carbide.frag_bullet.as_ref().unwrap();
        assert_eq!(frag.kind, BulletKind::Basic);
        assert_eq!(frag.speed, 8.1);
        assert_eq!(frag.damage, 227.0);
        assert_eq!(frag.lifetime, 8.0);
        assert_eq!(frag.width, 11.0);
        assert_eq!(frag.height, 14.0);
        assert_eq!(frag.hit_size, 7.0);
        assert_eq!(
            frag.shoot_effect,
            "MultiEffect(shootBigColor, colorSparkBig)"
        );
        assert_eq!(frag.ammo_multiplier, 1.0);
        assert_eq!(frag.reload_multiplier, 1.0);
        assert_eq!(frag.pierce_cap, 2);
        assert!(frag.pierce);
        assert!(frag.pierce_building);
        assert_eq!(frag.front_color, "white");
        assert_eq!(frag.back_color, "ab8ec5");
        assert_eq!(frag.trail_width, 1.8);
        assert_eq!(frag.trail_length, 11);
        assert_eq!(frag.hit_effect, "hitBulletColor");
        assert_eq!(frag.despawn_effect, "hitBulletColor");
        assert_close(frag.building_damage_multiplier, 0.2);
    }

    #[test]
    fn diffuse_spread_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let diffuse = registry.get_turret_by_name("diffuse").unwrap();
        assert_eq!(diffuse.kind, TurretBlockKind::ItemTurret);
        assert_eq!(diffuse.shoot_pattern, "ShootSpread");
        assert_eq!(diffuse.shoot_shots, 15);
        assert_eq!(diffuse.shoot_spread, 4.0);
        assert_eq!(diffuse.coolant_multiplier, 15.0);
        assert_eq!(diffuse.inaccuracy, 0.2);
        assert_eq!(diffuse.velocity_rnd, 0.17);
        assert_eq!(diffuse.shake, 1.0);
        assert_eq!(diffuse.ammo_per_shot, 3);
        assert_eq!(diffuse.max_ammo, 30);
        assert!(diffuse.consume_ammo_once);
        assert!(!diffuse.target_under_blocks);
        assert_eq!(diffuse.shoot_sound, "shootDiffuse");
        assert!(diffuse.drawer.contains("RegionPart(-front"));
        assert_eq!(diffuse.shoot_y, 5.0);
        assert_eq!(diffuse.outline_color, "darkOutline");
        assert_eq!(diffuse.base.size, 3);
        assert_ne!(diffuse.base.env_enabled & Env::SPACE, 0);
        assert_eq!(diffuse.reload, 30.0);
        assert_eq!(diffuse.recoil, 2.0);
        assert_eq!(diffuse.range, 125.0);
        assert_eq!(diffuse.shoot_cone, 40.0);
        assert_eq!(diffuse.scaled_health, 210.0);
        assert_eq!(diffuse.base.health, 3 * 3 * 210);
        assert_eq!(diffuse.rotate_speed, 3.0);
        assert!(diffuse.consume_coolant);
        assert_close(diffuse.coolant_amount, 15.0 / 60.0);
        assert_eq!(diffuse.fog_radius, 16.0);
        assert_eq!(diffuse.place_overlap_range, 125.0 + 8.0 * 7.0);
        assert_eq!(
            diffuse.requirements,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 50
                }
            ]
        );

        let graphite = &ammo_for(diffuse, item_id("graphite")).bullet;
        assert_eq!(graphite.kind, BulletKind::Basic);
        assert_eq!(graphite.speed, 8.0);
        assert_eq!(graphite.damage, 41.0);
        assert_eq!(graphite.knockback, 4.0);
        assert_eq!(graphite.width, 25.0);
        assert_eq!(graphite.hit_size, 7.0);
        assert_eq!(graphite.height, 20.0);
        assert_eq!(graphite.shoot_effect, "shootBigColor");
        assert_eq!(graphite.smoke_effect, "shootSmokeSquareSparse");
        assert_eq!(graphite.ammo_multiplier, 1.0);
        assert_eq!(graphite.hit_color, "ea8878");
        assert_eq!(graphite.back_color, "ea8878");
        assert_eq!(graphite.trail_color, "ea8878");
        assert_eq!(graphite.front_color, "redLight");
        assert_eq!(graphite.trail_width, 6.0);
        assert_eq!(graphite.trail_length, 3);
        assert_eq!(graphite.hit_effect, "hitSquaresColor");
        assert_eq!(graphite.despawn_effect, "hitSquaresColor");
        assert_close(graphite.building_damage_multiplier, 0.2);
        assert_close(graphite.lifetime, (125.0 + 25.0 + 10.0) / 8.0);

        let oxide = &ammo_for(diffuse, item_id("oxide")).bullet;
        assert_eq!(oxide.kind, BulletKind::Basic);
        assert_eq!(oxide.damage, 90.0);
        assert_eq!(oxide.knockback, 3.0);
        assert_eq!(oxide.ammo_multiplier, 2.0);
        assert_eq!(oxide.hit_color, "a0b380");
        assert_eq!(oxide.back_color, "a0b380");
        assert_eq!(oxide.trail_color, "a0b380");
        assert_eq!(oxide.front_color, "e4ffd6");
        assert_eq!(oxide.trail_width, 6.0);
        assert_eq!(oxide.trail_length, 3);
        assert_eq!(oxide.hit_effect, "hitSquaresColor");
        assert_close(oxide.building_damage_multiplier, 0.2);
        assert_close(oxide.lifetime, (125.0 + 25.0 + 10.0) / 8.0);

        let silicon = &ammo_for(diffuse, item_id("silicon")).bullet;
        assert_eq!(silicon.kind, BulletKind::Basic);
        assert_eq!(silicon.damage, 35.0);
        assert_eq!(silicon.knockback, 3.0);
        assert_eq!(silicon.homing_power, 0.045);
        assert_eq!(silicon.ammo_multiplier, 1.0);
        assert_eq!(silicon.hit_color, "siliconAmmoBack");
        assert_eq!(silicon.back_color, "siliconAmmoBack");
        assert_eq!(silicon.trail_color, "siliconAmmoBack");
        assert_eq!(silicon.front_color, "dae1ee");
        assert_eq!(silicon.trail_width, 6.0);
        assert_eq!(silicon.trail_length, 6);
        assert_eq!(silicon.hit_effect, "hitSquaresColor");
        assert_close(silicon.building_damage_multiplier, 0.2);
        assert_close(silicon.lifetime, (125.0 + 25.0 + 10.0) / 8.0);
    }

    #[test]
    fn sublimate_continuous_liquid_turret_keeps_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_content_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name.as_str() == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };
        fn liquid_ammo_for(turret: &TurretBlockData, liquid: ContentId) -> &LiquidTurretAmmo {
            turret
                .liquid_ammo
                .iter()
                .find(|ammo| ammo.liquid == liquid)
                .unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let sublimate = registry.get_turret_by_name("sublimate").unwrap();
        assert_eq!(sublimate.kind, TurretBlockKind::ContinuousLiquidTurret);
        assert!(sublimate.base.has_liquids);
        assert_eq!(sublimate.drawer.contains("heatColor=fa2859"), true);
        assert_eq!(sublimate.outline_color, "darkOutline");
        assert_eq!(sublimate.liquid_capacity, 50.0);
        assert_eq!(sublimate.base.liquid_capacity, 50.0);
        assert_close(sublimate.liquid_consumed, 18.0 / 60.0);
        assert_eq!(sublimate.target_interval, 5.0);
        assert_eq!(sublimate.new_target_interval, 30.0);
        assert!(!sublimate.target_under_blocks);
        assert_eq!(sublimate.shoot_y, 8.0);
        assert_eq!(sublimate.range, 130.0);
        assert_eq!(sublimate.loop_sound, "shootSublimate");
        assert_eq!(sublimate.shoot_sound, "none");
        assert_eq!(sublimate.loop_sound_volume, 1.0);
        assert_eq!(sublimate.scaled_health, 210.0);
        assert_eq!(sublimate.base.size, 3);
        assert_eq!(sublimate.base.health, 3 * 3 * 210);
        assert_eq!(sublimate.fog_radius, 16.0);
        assert_eq!(sublimate.place_overlap_range, 130.0 + 70.0 + 8.0 * 7.0);
        assert_eq!(
            sublimate.requirements,
            vec![
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 400
                }
            ]
        );

        let ozone = &liquid_ammo_for(sublimate, liquid_content_id("ozone")).bullet;
        assert_eq!(ozone.kind, BulletKind::ContinuousFlame);
        assert_eq!(ozone.damage, 60.0);
        assert_eq!(ozone.length, 130.0);
        assert_eq!(ozone.ammo_multiplier, 1.2);
        assert_eq!(ozone.knockback, 1.0);
        assert_eq!(ozone.pierce_cap, 2);
        assert_close(ozone.building_damage_multiplier, 0.3);
        assert!(ozone.timescale_damage);
        assert_eq!(ozone.hit_effect, "hitFlameBeam");
        assert_eq!(ozone.hit_size, 4.0);
        assert_eq!(ozone.draw_size, 420.0);
        assert_eq!(ozone.lifetime, 16.0);
        assert_eq!(ozone.optimal_life_fract, 0.5);
        assert!(!ozone.laser_absorb);
        assert!(ozone.pierce_armor);
        assert!(ozone.continuous);
        assert!(!ozone.remove_after_pierce);
        assert!(!ozone.keep_velocity);
        assert!(!ozone.collides);
        assert!(ozone.pierce);
        assert!(!ozone.hittable);
        assert!(!ozone.absorbable);
        assert_eq!(
            ozone.colors,
            vec!["eb7abe88", "e189f5b2", "907ef7cc", "91a4ff", "white"]
        );

        let cyanogen = &liquid_ammo_for(sublimate, liquid_content_id("cyanogen")).bullet;
        assert_eq!(cyanogen.kind, BulletKind::ContinuousFlame);
        assert_eq!(cyanogen.damage, 130.0);
        assert_eq!(cyanogen.range_change, 70.0);
        assert_eq!(cyanogen.length, 200.0);
        assert_eq!(cyanogen.knockback, 2.0);
        assert_eq!(cyanogen.pierce_cap, 3);
        assert_close(cyanogen.building_damage_multiplier, 0.3);
        assert!(cyanogen.timescale_damage);
        assert_eq!(
            cyanogen.colors,
            vec!["465ab888", "66a6d2b2", "89e8b6cc", "cafcbe", "white"]
        );
        assert_eq!(cyanogen.flare_color, "89e8b6");
        assert_eq!(cyanogen.light_color, "89e8b6");
        assert_eq!(cyanogen.hit_color, "89e8b6");
    }

    #[test]
    fn titan_item_turret_keeps_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name.as_str() == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let titan = registry.get_turret_by_name("titan").unwrap();
        assert_eq!(titan.kind, TurretBlockKind::ItemTurret);
        assert!(titan.base.has_items);
        assert!(titan.base.has_liquids);
        assert_eq!(titan.shoot_sound, "shootTank");
        assert_eq!(titan.ammo_per_shot, 4);
        assert_eq!(titan.max_ammo, 12);
        assert!(!titan.target_air);
        assert!(titan.target_ground);
        assert_eq!(titan.shake, 4.0);
        assert_eq!(titan.recoil, 1.0);
        assert_close(titan.reload, 60.0 * 2.3);
        assert_eq!(titan.shoot_y, 7.0);
        assert_eq!(titan.rotate_speed, 1.4);
        assert_eq!(titan.min_warmup, 0.85);
        assert_eq!(titan.new_target_interval, 40.0);
        assert_eq!(titan.shoot_warmup_speed, 0.08);
        assert_eq!(titan.warmup_maintain_time, 120.0);
        assert!(titan.consume_coolant);
        assert_eq!(titan.coolant_amount, 30.0 / 60.0);
        assert_eq!(titan.coolant_multiplier, 3.75);
        assert_eq!(titan.outline_color, "darkOutline");
        assert!(titan.drawer.contains("reinforced-"));
        assert!(titan.drawer.contains("RegionPart(-barrel"));
        assert!(titan.drawer.contains("RegionPart(-side"));
        assert_eq!(titan.scaled_health, 250.0);
        assert_eq!(titan.range, 390.0);
        assert_eq!(titan.base.size, 4);
        assert_eq!(titan.base.health, 4 * 4 * 250);
        assert_eq!(titan.fog_radius, 49.0);
        assert_eq!(titan.place_overlap_range, 390.0 + 80.0 + 8.0 * 7.0);
        assert_eq!(
            titan.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("hydrogen"),
                amount: 5.0 / 60.0
            }]
        );
        assert_eq!(
            titan.requirements,
            vec![
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 300
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 400
                }
            ]
        );

        let thorium = &ammo_for(titan, item_id("thorium")).bullet;
        assert_eq!(thorium.kind, BulletKind::Artillery);
        assert_eq!(thorium.speed, 2.5);
        assert_eq!(thorium.damage, 350.0);
        assert_eq!(
            thorium.hit_effect,
            "MultiEffect(titanExplosion, titanSmoke)"
        );
        assert_eq!(thorium.despawn_effect, "none");
        assert_eq!(thorium.knockback, 2.0);
        assert_eq!(thorium.lifetime, 140.0);
        assert_eq!(thorium.width, 17.0);
        assert_eq!(thorium.height, 19.0);
        assert_eq!(thorium.splash_damage_radius, 65.0);
        assert_eq!(thorium.splash_damage, 350.0);
        assert!(thorium.scaled_splash_damage);
        assert_eq!(thorium.back_color, "f49e7c");
        assert_eq!(thorium.hit_color, "f49e7c");
        assert_eq!(thorium.trail_color, "f49e7c");
        assert_eq!(thorium.front_color, "white");
        assert_eq!(thorium.hit_sound, "explosionTitan");
        assert_eq!(thorium.status, "blasted");
        assert_eq!(thorium.trail_length, 32);
        assert_eq!(thorium.trail_width, 3.35);
        assert_eq!(thorium.trail_sin_scl, 2.5);
        assert_eq!(thorium.trail_sin_mag, 0.5);
        assert_eq!(thorium.trail_effect, "none");
        assert_eq!(thorium.despawn_shake, 7.0);
        assert_eq!(thorium.shoot_effect, "shootTitan");
        assert_eq!(thorium.smoke_effect, "shootSmokeTitan");
        assert_eq!(thorium.trail_interp, "max(slope,0.8)");
        assert_eq!(thorium.shrink_x, 0.2);
        assert_eq!(thorium.shrink_y, 0.1);
        assert_close(thorium.building_damage_multiplier, 0.3);

        let carbide = &ammo_for(titan, item_id("carbide")).bullet;
        assert_eq!(carbide.kind, BulletKind::Artillery);
        assert_eq!(carbide.speed, 3.25);
        assert_eq!(carbide.damage, 700.0);
        assert_eq!(
            carbide.hit_effect,
            "MultiEffect(titanExplosionSmall, titanSmokeSmall)"
        );
        assert_eq!(carbide.lifetime, 140.0);
        assert_eq!(carbide.width, 15.0);
        assert_eq!(carbide.height, 28.0);
        assert_eq!(carbide.splash_damage_radius, 36.0);
        assert_eq!(carbide.splash_damage, 750.0);
        assert_eq!(carbide.range_change, 10.0 * 8.0);
        assert_eq!(carbide.reload_multiplier, 0.8);
        assert!(carbide.scaled_splash_damage);
        assert_eq!(carbide.hit_sound, "explosionTitan");
        assert_eq!(carbide.status, "blasted");
        assert_eq!(carbide.trail_effect, "disperseTrail");
        assert_eq!(carbide.trail_interval, 2.0);
        assert!(carbide.trail_rotation);
        assert_close(carbide.building_damage_multiplier, 0.2);
        assert_eq!(carbide.frag_life_min, 1.5);
        assert_eq!(carbide.frag_bullets, 12);

        let carbide_frag = carbide.frag_bullet.as_ref().unwrap();
        assert_eq!(carbide_frag.kind, BulletKind::Artillery);
        assert_eq!(carbide_frag.speed, 0.5);
        assert_eq!(carbide_frag.damage, 50.0);
        assert_eq!(carbide_frag.despawn_effect, "hitBulletColor");
        assert_eq!(carbide_frag.width, 8.0);
        assert_eq!(carbide_frag.height, 12.0);
        assert_eq!(carbide_frag.lifetime, 50.0);
        assert_eq!(carbide_frag.knockback, 0.5);
        assert_eq!(carbide_frag.splash_damage_radius, 22.0);
        assert_eq!(carbide_frag.splash_damage, 50.0);
        assert!(carbide_frag.scaled_splash_damage);
        assert!(carbide_frag.pierce_armor);
        assert_eq!(carbide_frag.back_color, "ab8ec5");
        assert_eq!(carbide_frag.hit_color, "ab8ec5");
        assert_eq!(carbide_frag.front_color, "white");
        assert_close(carbide_frag.building_damage_multiplier, 0.25);
        assert_eq!(carbide_frag.shrink_y, 0.3);

        let oxide = &ammo_for(titan, item_id("oxide")).bullet;
        assert_eq!(oxide.kind, BulletKind::Artillery);
        assert_eq!(oxide.speed, 2.5);
        assert_eq!(oxide.damage, 300.0);
        assert_eq!(
            oxide.hit_effect,
            "MultiEffect(titanExplosionLarge, titanSmokeLarge, smokeAoeCloud)"
        );
        assert_eq!(oxide.lifetime, 190.0);
        assert_eq!(oxide.width, 17.0);
        assert_eq!(oxide.height, 19.0);
        assert_eq!(oxide.reload_multiplier, 0.7);
        assert_eq!(oxide.splash_damage_radius, 110.0);
        assert_eq!(oxide.range_change, 8.0);
        assert_eq!(oxide.splash_damage, 180.0);
        assert!(oxide.scaled_splash_damage);
        assert_eq!(oxide.hit_color, "a0b380");
        assert_eq!(oxide.front_color, "e4ffd6");
        assert_eq!(oxide.hit_sound, "explosionTitan");
        assert_eq!(oxide.trail_effect, "vapor");
        assert_eq!(oxide.trail_interval, 3.0);
        assert_close(oxide.building_damage_multiplier, 0.25);
        assert_eq!(oxide.status, "corroded");
        assert_eq!(oxide.status_duration, 60.0 * 8.0);
        assert_eq!(oxide.frag_bullets, 1);

        let oxide_frag = oxide.frag_bullet.as_ref().unwrap();
        assert_eq!(oxide_frag.kind, BulletKind::Generic);
        assert_eq!(oxide_frag.damage, 0.0);
        assert_eq!(oxide_frag.lifetime, 60.0 * 2.5);
        assert_eq!(oxide_frag.bullet_interval, 20.0);
        assert_eq!(oxide_frag.hit_effect, "none");
        assert_eq!(oxide_frag.despawn_effect, "none");

        let oxide_interval = oxide_frag.interval_bullet.as_ref().unwrap();
        assert_eq!(oxide_interval.kind, BulletKind::Generic);
        assert_eq!(oxide_interval.splash_damage, 15.0);
        assert!(oxide_interval.collides_ground);
        assert!(!oxide_interval.collides_air);
        assert!(!oxide_interval.collides);
        assert!(oxide_interval.pierce);
        assert!(oxide_interval.instant_disappear);
        assert_eq!(oxide_interval.splash_damage_radius, 90.0);
        assert_eq!(oxide_interval.building_damage_multiplier, 0.0);
    }

    #[test]
    fn disperse_item_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let disperse = registry.get_turret_by_name("disperse").unwrap();
        assert_eq!(disperse.kind, TurretBlockKind::ItemTurret);
        assert!(disperse.base.has_items);
        assert_eq!(disperse.reload, 9.0);
        assert_eq!(disperse.shoot_y, 15.0);
        assert_eq!(disperse.rotate_speed, 5.0);
        assert_eq!(disperse.shoot_cone, 30.0);
        assert!(disperse.consume_ammo_once);
        assert_eq!(disperse.shoot_sound, "shootDisperse");
        assert_eq!(disperse.shoot_pattern, "ShootAlternate");
        assert_eq!(disperse.shoot_alternate_spread, 4.7);
        assert_eq!(disperse.shoot_shots, 4);
        assert_eq!(disperse.shoot_alternate_barrels, 4);
        assert!(!disperse.target_ground);
        assert_eq!(disperse.inaccuracy, 8.0);
        assert_eq!(disperse.shoot_warmup_speed, 0.08);
        assert_eq!(disperse.outline_color, "darkOutline");
        assert!(disperse.drawer.contains("reinforced-"));
        assert!(disperse.drawer.contains("RegionPart(-side"));
        assert!(disperse.drawer.contains("RegionPart(-mid"));
        assert!(disperse.drawer.contains("RegionPart(-blade"));
        assert_eq!(disperse.scaled_health, 280.0);
        assert_eq!(disperse.range, 310.0);
        assert_eq!(disperse.base.size, 4);
        assert_eq!(disperse.base.health, 4 * 4 * 280);
        assert!(disperse.consume_coolant);
        assert_eq!(disperse.coolant_amount, 20.0 / 60.0);
        assert_eq!(disperse.coolant_multiplier, 6.25);
        assert_eq!(disperse.fog_radius, 39.0);
        assert_eq!(disperse.place_overlap_range, 310.0 + 8.0 * 7.0);
        assert_eq!(
            disperse.requirements,
            vec![
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 350
                }
            ]
        );

        let tungsten = &ammo_for(disperse, item_id("tungsten")).bullet;
        assert_eq!(tungsten.kind, BulletKind::Basic);
        assert_eq!(tungsten.speed, 8.5);
        assert_eq!(tungsten.damage, 65.0);
        assert_eq!(tungsten.width, 16.0);
        assert_eq!(tungsten.height, 16.0);
        assert_eq!(tungsten.shrink_y, 0.3);
        assert_eq!(tungsten.back_sprite, "large-bomb-back");
        assert_eq!(tungsten.sprite, "mine-bullet");
        assert_eq!(tungsten.velocity_rnd, 0.11);
        assert!(!tungsten.collides_ground);
        assert!(!tungsten.collides_tiles);
        assert_eq!(tungsten.shoot_effect, "shootBig2");
        assert_eq!(tungsten.smoke_effect, "shootSmokeDisperse");
        assert_eq!(tungsten.front_color, "white");
        assert_eq!(tungsten.back_color, "sky");
        assert_eq!(tungsten.trail_color, "sky");
        assert_eq!(tungsten.hit_color, "sky");
        assert_eq!(tungsten.trail_chance, 0.44);
        assert_eq!(tungsten.ammo_multiplier, 3.0);
        assert_eq!(tungsten.rotation_offset, 90.0);
        assert!(tungsten.trail_rotation);
        assert_eq!(tungsten.trail_effect, "disperseTrail");
        assert_eq!(tungsten.hit_effect, "hitBulletColor");
        assert_eq!(tungsten.despawn_effect, "hitBulletColor");
        assert_close(tungsten.lifetime, (310.0 + 16.0 + 10.0) / 8.5);

        let thorium = &ammo_for(disperse, item_id("thorium")).bullet;
        assert_eq!(thorium.kind, BulletKind::Basic);
        assert_eq!(thorium.damage, 90.0);
        assert_eq!(thorium.reload_multiplier, 0.85);
        assert_eq!(thorium.range_change, -120.0);
        assert_eq!(thorium.speed, 9.5);
        assert_eq!(thorium.pierce_cap, 2);
        assert_eq!(thorium.velocity_rnd, 0.5);
        assert_eq!(thorium.back_color, "e89dbd");
        assert_eq!(thorium.trail_color, "e89dbd");
        assert_eq!(thorium.hit_color, "e89dbd");
        assert_eq!(thorium.extra_range_margin, 32.0);
        assert!(thorium.trail_rotation);
        assert_eq!(thorium.trail_effect, "disperseTrail");
        assert_close(thorium.lifetime, (310.0 - 120.0 + 16.0 + 32.0 + 10.0) / 9.5);

        let silicon = &ammo_for(disperse, item_id("silicon")).bullet;
        assert_eq!(silicon.kind, BulletKind::Basic);
        assert_eq!(silicon.damage, 37.0);
        assert_eq!(silicon.homing_power, 0.045);
        assert_eq!(silicon.speed, 9.0);
        assert_eq!(silicon.ammo_multiplier, 4.0);
        assert_eq!(silicon.trail_length, 7);
        assert_eq!(silicon.extra_range_margin, 32.0);
        assert_eq!(silicon.front_color, "dae1ee");
        assert_eq!(silicon.back_color, "siliconAmmoBack");
        assert_eq!(silicon.trail_color, "siliconAmmoBack");
        assert_eq!(silicon.hit_color, "siliconAmmoBack");
        assert_close(silicon.lifetime, (310.0 + 16.0 + 32.0 + 10.0) / 9.0);

        let surge = &ammo_for(disperse, item_id("surge-alloy")).bullet;
        assert_eq!(surge.kind, BulletKind::Basic);
        assert_eq!(surge.reload_multiplier, 0.75);
        assert_eq!(surge.damage, 65.0);
        assert_eq!(surge.range_change, 8.0 * 3.0);
        assert_eq!(surge.lightning, 3);
        assert_eq!(surge.lightning_length, 4);
        assert_eq!(surge.lightning_damage, 18.0);
        assert_eq!(surge.lightning_length_rand, 3);
        assert_eq!(surge.speed, 6.0);
        assert_eq!(surge.back_color, "surge");
        assert_eq!(surge.trail_color, "surge");
        assert_eq!(surge.hit_color, "surge");
        assert_eq!(surge.trail_chance, 0.44);
        assert_eq!(surge.ammo_multiplier, 3.0);
        assert!(surge.trail_rotation);
        assert_eq!(surge.trail_effect, "disperseTrail");
        assert_eq!(surge.bullet_interval, 4.0);
        assert_close(surge.lifetime, (310.0 + 24.0 + 16.0 + 10.0) / 6.0);

        let interval = surge.interval_bullet.as_ref().unwrap();
        assert_eq!(interval.kind, BulletKind::Generic);
        assert!(!interval.collides_ground);
        assert!(!interval.collides_tiles);
        assert!(!interval.collides);
        assert!(!interval.hittable);
        assert_eq!(interval.lightning_length_rand, 4);
        assert_eq!(interval.lightning_length, 2);
        assert_eq!(interval.lightning_cone, 30.0);
        assert_eq!(interval.lightning_damage, 20.0);
        assert_eq!(interval.lightning, 1);
        assert!(interval.instant_disappear);
        assert_eq!(interval.hit_effect, "none");
        assert_eq!(interval.despawn_effect, "none");
    }

    #[test]
    fn afflict_power_turret_keeps_upstream_subset() {
        let (all_items, _all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let afflict = registry.get_turret_by_name("afflict").unwrap();
        assert_eq!(afflict.kind, TurretBlockKind::PowerTurret);
        assert!(afflict.base.has_power);
        assert_eq!(afflict.consume_power, 5.0);
        assert_eq!(afflict.heat_requirement, 20.0);
        assert_eq!(afflict.max_heat_efficiency, 1.0);
        assert_eq!(afflict.new_target_interval, 40.0);
        assert_eq!(afflict.inaccuracy, 1.0);
        assert_eq!(afflict.shake, 2.0);
        assert_eq!(afflict.shoot_y, 4.0);
        assert_eq!(afflict.outline_color, "darkOutline");
        assert_eq!(afflict.base.size, 4);
        assert_ne!(afflict.base.env_enabled & Env::SPACE, 0);
        assert_eq!(afflict.reload, 50.0);
        assert_eq!(afflict.cooldown_time, 100.0);
        assert_eq!(afflict.recoil, 3.0);
        assert_eq!(afflict.range, 368.0);
        assert_eq!(afflict.shoot_cone, 20.0);
        assert_eq!(afflict.scaled_health, 220.0);
        assert_eq!(afflict.base.health, 4 * 4 * 220);
        assert_eq!(afflict.rotate_speed, 1.5);
        assert_eq!(afflict.research_cost_multiplier, 0.04);
        assert_eq!(afflict.build_cost_multiplier, 1.5);
        assert_eq!(afflict.fog_radius, 46.0);
        assert_eq!(afflict.place_overlap_range, 368.0 + 8.0 * 7.0);
        assert!(afflict.drawer.contains("RegionPart(-blade"));
        assert!(afflict.drawer.contains("RegionPart(-blade-glow"));
        assert_eq!(
            afflict.requirements,
            vec![
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 40
                }
            ]
        );

        let shoot = afflict.shoot_type.as_ref().unwrap();
        assert_eq!(shoot.kind, BulletKind::Basic);
        assert_eq!(shoot.speed, 5.0);
        assert_eq!(shoot.damage, 180.0);
        assert_close(shoot.lifetime, (368.0 - 55.0 + 10.0) / 5.0);
        assert!(shoot.shoot_effect.contains("shootTitan"));
        assert_eq!(shoot.smoke_effect, "shootSmokeTitan");
        assert_eq!(shoot.hit_color, "surge");
        assert_eq!(shoot.sprite, "large-orb");
        assert_eq!(shoot.trail_effect, "missileTrail");
        assert_eq!(shoot.trail_interval, 3.0);
        assert_eq!(shoot.trail_param, 4.0);
        assert_eq!(shoot.pierce_cap, 2);
        assert_close(shoot.building_damage_multiplier, 0.5);
        assert!(!shoot.frag_on_hit);
        assert_eq!(shoot.width, 16.0);
        assert_eq!(shoot.height, 16.0);
        assert_eq!(shoot.back_color, "surge");
        assert_eq!(shoot.front_color, "white");
        assert_eq!(shoot.shrink_x, 0.0);
        assert_eq!(shoot.shrink_y, 0.0);
        assert_eq!(shoot.trail_color, "surge");
        assert_eq!(shoot.trail_length, 12);
        assert_eq!(shoot.trail_width, 2.2);
        assert!(shoot.hit_effect.contains("ExplosionEffect"));
        assert_eq!(shoot.despawn_effect, shoot.hit_effect);
        assert_eq!(shoot.despawn_sound, "explosionAfflict");
        assert_eq!(shoot.bullet_shoot_sound, "shootAfflict");
        assert_eq!(shoot.bullet_interval, 3.0);
        assert_eq!(shoot.interval_random_spread, 20.0);
        assert_eq!(shoot.interval_bullets, 2);
        assert_eq!(shoot.interval_angle, 180.0);
        assert_eq!(shoot.interval_spread, 300.0);
        assert_eq!(shoot.frag_bullets, 20);
        assert_eq!(shoot.frag_velocity_min, 0.5);
        assert_eq!(shoot.frag_velocity_max, 1.2);
        assert_eq!(shoot.frag_life_min, 0.5);

        let frag = shoot.frag_bullet.as_ref().unwrap();
        assert_eq!(frag.kind, BulletKind::Basic);
        assert_eq!(frag.speed, 3.0);
        assert_eq!(frag.damage, 35.0);
        assert_eq!(frag.width, 9.0);
        assert_eq!(frag.hit_size, 5.0);
        assert_eq!(frag.height, 15.0);
        assert_eq!(frag.pierce_cap, 3);
        assert_eq!(frag.lifetime, 28.0);
        assert!(frag.pierce_building);
        assert_eq!(frag.hit_color, "surge");
        assert_eq!(frag.back_color, "surge");
        assert_eq!(frag.trail_color, "surge");
        assert_eq!(frag.front_color, "white");
        assert_eq!(frag.trail_width, 2.1);
        assert_eq!(frag.trail_length, 5);
        assert!(frag.hit_effect.contains("WaveEffect"));
        assert_eq!(frag.despawn_effect, frag.hit_effect);
        assert_close(frag.building_damage_multiplier, 0.3);
        assert_eq!(frag.homing_power, 0.1);
        assert_eq!(
            shoot.interval_bullet.as_ref().unwrap().as_ref(),
            frag.as_ref()
        );
    }

    #[test]
    fn lustre_continuous_turret_keeps_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name.as_str() == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let lustre = registry.get_turret_by_name("lustre").unwrap();
        assert_eq!(lustre.kind, TurretBlockKind::ContinuousTurret);
        assert_eq!(lustre.base.group, BlockGroup::Turrets);
        assert_eq!(lustre.base.flags, vec![BlockFlag::Turret]);
        assert!(lustre.base.update);
        assert!(lustre.base.solid);
        assert!(lustre.base.sync);
        assert!(lustre.base.has_power);
        assert!(lustre.base.has_liquids);
        assert_ne!(lustre.base.env_enabled & Env::SPACE, 0);
        assert!(!lustre.display_ammo_multiplier);
        assert_eq!(lustre.coolant_multiplier, 1.0);
        assert!(lustre.scale_damage_efficiency);
        assert_eq!(lustre.aim_change_speed, 0.9);
        assert_eq!(lustre.rotate_speed, 0.9);
        assert_eq!(lustre.shoot_warmup_speed, 0.08);
        assert_eq!(lustre.shoot_cone, 360.0);
        assert_eq!(lustre.shoot_y, 0.5);
        assert_eq!(lustre.outline_color, "darkOutline");
        assert_eq!(lustre.base.size, 4);
        assert_eq!(lustre.range, 250.0);
        assert_eq!(lustre.scaled_health, 210.0);
        assert_eq!(lustre.base.health, 4 * 4 * 210);
        assert_eq!(lustre.unit_sort, "strongest");
        assert_eq!(lustre.loop_sound, "beamLustre");
        assert_eq!(lustre.loop_sound_volume, 1.0);
        assert_eq!(lustre.shoot_sound, "none");
        assert_close(lustre.consume_power, 200.0 / 60.0);
        assert_eq!(
            lustre.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("nitrogen"),
                amount: 6.0 / 60.0
            }]
        );
        assert_eq!(lustre.fog_radius, 31.0);
        assert_eq!(lustre.place_overlap_range, 250.0 + 8.0 * 7.0);
        assert_eq!(
            lustre.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 90
                }
            ]
        );
        assert!(lustre.drawer.contains("reinforced-"));
        assert!(lustre.drawer.contains("RegionPart(-blade"));
        assert!(lustre.drawer.contains("RegionPart(-inner"));
        assert!(lustre.drawer.contains("RegionPart(-mid"));
        assert!(lustre.drawer.contains("heatColor=ff6214"));
        assert!(lustre.drawer.contains("moveRot=-7"));
        assert!(lustre.drawer.contains("moveY=-8"));
        assert!(lustre.drawer.contains("PartMove(warmup,0,-2,3)"));

        let shoot = lustre.shoot_type.as_ref().unwrap();
        assert_eq!(shoot.kind, BulletKind::PointLaser);
        assert_eq!(shoot.damage, 210.0);
        assert_close(shoot.building_damage_multiplier, 0.3);
        assert_eq!(shoot.hit_color, "fda981");
        assert_eq!(shoot.color, "white");
        assert_eq!(shoot.sprite, "point-laser");
        assert_eq!(shoot.damage_interval, 5.0);
        assert_eq!(shoot.beam_effect, "colorTrail");
        assert_eq!(shoot.beam_effect_interval, 3.0);
        assert_eq!(shoot.beam_effect_size, 3.5);
        assert_eq!(shoot.osc_scl, 2.0);
        assert_eq!(shoot.osc_mag, 0.3);
        assert_eq!(shoot.draw_size, 1000.0);
        assert_eq!(shoot.lifetime, 20.0);
        assert_eq!(shoot.speed, 0.0);
        assert_eq!(shoot.hit_effect, "hitBulletSmall");
        assert_eq!(shoot.ammo_multiplier, 2.0);
        assert!(!shoot.remove_after_pierce);
        assert!(shoot.impact);
        assert!(!shoot.keep_velocity);
        assert!(!shoot.collides);
        assert!(shoot.pierce);
        assert!(!shoot.hittable);
        assert!(!shoot.absorbable);
        assert_eq!(shoot.optimal_life_fract, 0.5);
        assert_eq!(shoot.shoot_effect, "none");
        assert_eq!(shoot.smoke_effect, "none");
        assert_eq!(shoot.despawn_effect, "none");
    }

    #[test]
    fn scathe_item_turret_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let scathe = registry.get_turret_by_name("scathe").unwrap();
        assert_eq!(scathe.kind, TurretBlockKind::ItemTurret);
        assert_eq!(
            scathe.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 450
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 400
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 500
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 200
                }
            ]
        );
        assert!(!scathe.predict_target);
        assert!(!scathe.target_air);
        assert!(!scathe.target_under_blocks);
        assert_eq!(scathe.base.size, 4);
        assert_eq!(scathe.scaled_health, 220.0);
        assert_eq!(scathe.base.health, 4 * 4 * 220);
        assert_eq!(scathe.recoil, 0.5);
        assert_eq!(scathe.fog_radius_multiplier, 0.4);
        assert_eq!(scathe.coolant_multiplier, 15.0);
        assert_eq!(scathe.shoot_sound, "shootScathe");
        assert_eq!(scathe.min_warmup, 0.94);
        assert_eq!(scathe.new_target_interval, 40.0);
        assert_eq!(scathe.unit_sort, "strongest");
        assert_eq!(scathe.shoot_warmup_speed, 0.03);
        assert_eq!(scathe.shake, 6.0);
        assert_eq!(scathe.ammo_per_shot, 15);
        assert_eq!(scathe.max_ammo, 45);
        assert_eq!(scathe.shoot_y, -1.0);
        assert_eq!(scathe.outline_color, "darkOutline");
        assert_eq!(scathe.reload, 600.0);
        assert_eq!(scathe.range, 1350.0);
        assert_eq!(scathe.shoot_cone, 1.0);
        assert_eq!(scathe.rotate_speed, 0.9);
        assert!(scathe.consume_coolant);
        assert_close(scathe.coolant_amount, 15.0 / 60.0);
        assert_eq!(scathe.fog_radius, (1350.0_f32 / 8.0 * 0.4).round());
        assert_eq!(scathe.place_overlap_range, 1350.0 + 8.0 * 7.0);
        assert!(scathe.drawer.contains("reinforced-"));
        assert!(scathe.drawer.contains("blade"));
        assert!(scathe.drawer.contains("mid"));
        assert!(scathe.drawer.contains("missile"));

        let carbide = ammo_for(scathe, item_id("carbide"));
        assert_eq!(carbide.bullet.kind, BulletKind::Generic);
        assert_eq!(carbide.bullet.speed, 0.0);
        assert_eq!(carbide.bullet.damage, 0.0);
        assert_eq!(carbide.bullet.shoot_effect, "shootBig");
        assert_eq!(carbide.bullet.smoke_effect, "shootSmokeMissileColor");
        assert_eq!(carbide.bullet.hit_color, "redLight");
        assert_eq!(carbide.bullet.ammo_multiplier, 1.0);
        let carbide_unit = carbide.bullet.spawn_unit.as_ref().unwrap();
        assert_eq!(carbide_unit.name, "scathe-missile");
        assert_eq!(carbide_unit.speed, 4.6);
        assert_eq!(carbide_unit.lifetime, 330.0);
        assert_eq!(carbide_unit.health, 240.0);
        assert_eq!(carbide_unit.rotate_speed, 0.25);
        assert_eq!(carbide_unit.missile_accel_time, 50.0);
        assert_eq!(carbide_unit.trail_length, 18);
        assert_eq!(carbide_unit.engine_color, "redLight");
        assert_eq!(carbide_unit.weapons.len(), 1);
        assert_eq!(carbide_unit.abilities.len(), 1);
        assert_eq!(carbide_unit.abilities[0].kind, AbilityKind::MoveEffect);
        assert_eq!(carbide_unit.abilities[0].interval, 7.0);
        let carbide_explosion = carbide_unit.weapons[0].bullet.as_ref().unwrap();
        assert_eq!(carbide_explosion.kind, BulletKind::Explosion);
        assert_eq!(carbide_explosion.splash_damage, 1000.0);
        assert_eq!(carbide_explosion.splash_damage_radius, 65.0);
        assert_eq!(carbide_explosion.hit_color, "redLight");
        assert!(!carbide_explosion.collides_air);
        assert_close(carbide_explosion.building_damage_multiplier, 0.1);
        assert_eq!(carbide_explosion.frag_life_min, 0.1);
        assert_eq!(carbide_explosion.frag_bullets, 7);
        let carbide_frag = carbide_explosion.frag_bullet.as_ref().unwrap();
        assert_eq!(carbide_frag.kind, BulletKind::Artillery);
        assert_eq!(carbide_frag.speed, 3.4);
        assert_eq!(carbide_frag.damage, 32.0);
        assert_close(carbide_frag.building_damage_multiplier, 0.1);
        assert_eq!(carbide_frag.drag, 0.02);
        assert_eq!(carbide_frag.lifetime, 23.0);
        assert_eq!(carbide_frag.width, 18.0);
        assert_eq!(carbide_frag.height, 18.0);
        assert_eq!(carbide_frag.splash_damage_radius, 40.0);
        assert_eq!(carbide_frag.splash_damage, 100.0);
        assert_eq!(carbide_frag.light_radius, 30.0);
        assert_eq!(carbide_frag.light_color, "redLight");
        assert_eq!(carbide_frag.light_opacity, 0.5);

        let phase = ammo_for(scathe, item_id("phase-fabric"));
        assert_eq!(phase.bullet.hit_color, "ffd37f");
        assert_eq!(phase.bullet.ammo_multiplier, 5.0);
        assert_eq!(phase.bullet.reload_multiplier, 0.8);
        let phase_unit = phase.bullet.spawn_unit.as_ref().unwrap();
        assert_eq!(phase_unit.name, "scathe-missile-phase");
        assert_eq!(phase_unit.speed, 2.5);
        assert_close(phase_unit.lifetime, 586.2);
        assert_eq!(phase_unit.health, 500.0);
        assert_eq!(phase_unit.rotate_speed, 0.2);
        assert_eq!(phase_unit.parts.len(), 1);
        let phase_part = &phase_unit.parts[0];
        assert_eq!(phase_part.kind, UnitPartKind::Shape);
        assert_eq!(phase_part.color, "accent");
        assert_eq!(phase_part.sides, 6);
        assert_eq!(phase_part.radius, 3.0);
        assert_eq!(phase_part.rotate_speed, 3.0);
        assert!(phase_part.hollow);
        assert_eq!(phase_part.layer, "effect");
        assert_eq!(phase_part.y, 1.8);
        assert!(phase_unit
            .abilities
            .iter()
            .any(|ability| ability.kind == AbilityKind::MoveEffect && ability.interval == 15.0));
        assert!(phase_unit.abilities.iter().any(|ability| {
            ability.kind == AbilityKind::ForceField
                && ability.radius == 120.0
                && ability.regen == 0.0
                && ability.max == 3000.0
                && ability.cooldown == 999999999.0
        }));
        let phase_explosion = phase_unit.weapons[0].bullet.as_ref().unwrap();
        assert_eq!(phase_explosion.splash_damage, 320.0);
        assert_eq!(phase_explosion.splash_damage_radius, 120.0);
        assert_eq!(phase_explosion.reload_multiplier, 0.8);
        assert_eq!(phase_explosion.ammo_multiplier, 5.0);
        let phase_frag = phase_explosion.frag_bullet.as_ref().unwrap();
        assert_eq!(phase_frag.splash_damage_radius, 56.0);
        assert_eq!(phase_frag.splash_damage, 120.0);
        assert_close(phase_frag.building_damage_multiplier, 0.2);

        let surge = ammo_for(scathe, item_id("surge-alloy"));
        assert_eq!(surge.bullet.hit_color, "f7e97e");
        assert_eq!(surge.bullet.ammo_multiplier, 1.0);
        assert_eq!(surge.bullet.reload_multiplier, 0.9);
        let surge_unit = surge.bullet.spawn_unit.as_ref().unwrap();
        assert_eq!(surge_unit.name, "scathe-missile-surge");
        assert_eq!(surge_unit.speed, 4.4);
        assert_eq!(surge_unit.lifetime, 84.0);
        assert_eq!(surge_unit.health, 300.0);
        assert_eq!(surge_unit.missile_accel_time, 30.0);
        let surge_weapon = &surge_unit.weapons[0];
        assert!(surge_weapon.rotate);
        assert_eq!(surge_weapon.rotation_limit, 0.0);
        assert_eq!(surge_weapon.rotate_speed, 0.0);
        let surge_explosion = surge_weapon.bullet.as_ref().unwrap();
        assert_eq!(surge_explosion.splash_damage, 1800.0);
        assert_eq!(surge_explosion.splash_damage_radius, 40.0);
        assert_eq!(surge_explosion.lightning, 10);
        assert_eq!(surge_explosion.lightning_damage, 45.0);
        assert_eq!(surge_explosion.lightning_length, 12);
        assert_eq!(surge_explosion.frag_bullets, 5);
        assert_eq!(surge_explosion.frag_random_spread, 0.0);
        assert_eq!(surge_explosion.frag_spread, 20.0);
        let split_carrier = surge_explosion.frag_bullet.as_ref().unwrap();
        assert_eq!(split_carrier.kind, BulletKind::Generic);
        let split_unit = split_carrier.spawn_unit.as_ref().unwrap();
        assert_eq!(split_unit.name, "scathe-missile-surge-split");
        assert_eq!(split_unit.speed, 4.8);
        assert_eq!(split_unit.lifetime, 222.0);
        assert_eq!(split_unit.health, 50.0);
        assert_eq!(split_unit.engine_size, 2.2);
        assert_eq!(split_unit.engine_offset, 8.0);
        assert_eq!(split_unit.trail_length, 12);
        let split_explosion = split_unit.weapons[0].bullet.as_ref().unwrap();
        assert_eq!(split_explosion.splash_damage, 180.0);
        assert_eq!(split_explosion.splash_damage_radius, 35.0);
        assert_eq!(split_explosion.lightning, 4);
        assert_eq!(split_explosion.lightning_damage, 25.0);
        assert_eq!(split_explosion.lightning_length, 6);
    }

    #[test]
    fn smite_item_turret_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        fn ammo_for(turret: &TurretBlockData, item: ContentId) -> &TurretAmmo {
            turret.ammo.iter().find(|ammo| ammo.item == item).unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let smite = registry.get_turret_by_name("smite").unwrap();
        assert_eq!(smite.kind, TurretBlockKind::ItemTurret);
        assert_eq!(
            smite.requirements,
            vec![
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("surge-alloy"),
                    amount: 400
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 800
                },
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 500
                },
                ItemAmount {
                    item: item_id("phase-fabric"),
                    amount: 300
                }
            ]
        );
        assert_eq!(smite.ammo.len(), 1);
        assert_eq!(smite.shoot_pattern, "ShootMulti(ShootAlternate,ShootHelix)");
        assert_close(smite.shoot_alternate_spread, 3.3 * 1.9);
        assert_eq!(smite.shoot_shots, 5);
        assert_eq!(smite.shoot_alternate_barrels, 5);
        assert_eq!(smite.shoot_helix_scl, 4.0);
        assert_eq!(smite.shoot_helix_mag, 3.0);
        assert_eq!(smite.shoot_sound, "shootSmite");
        assert_eq!(smite.min_warmup, 0.99);
        assert_eq!(smite.coolant_multiplier, 15.0);
        assert_eq!(smite.shake, 2.0);
        assert_eq!(smite.ammo_per_shot, 2);
        assert_eq!(smite.shoot_warmup_speed, 0.04);
        assert_eq!(smite.shoot_y, 15.0);
        assert_eq!(smite.outline_color, "darkOutline");
        assert_eq!(smite.base.size, 5);
        assert_ne!(smite.base.env_enabled & Env::SPACE, 0);
        assert_eq!(smite.warmup_maintain_time, 120.0);
        assert_eq!(smite.reload, 100.0);
        assert_eq!(smite.recoil, 2.0);
        assert_eq!(smite.range, 300.0);
        assert_eq!(smite.tracking_range, 300.0 * 1.4);
        assert_eq!(smite.shoot_cone, 30.0);
        assert_eq!(smite.scaled_health, 350.0);
        assert_eq!(smite.base.health, 5 * 5 * 350);
        assert_eq!(smite.rotate_speed, 1.5);
        assert!(smite.consume_coolant);
        assert_close(smite.coolant_amount, 15.0 / 60.0);
        assert_eq!(smite.loop_sound, "loopGlow");
        assert_eq!(smite.loop_sound_volume, 0.8);
        assert_eq!(smite.fog_radius, (300.0_f32 / 8.0).round());
        assert_eq!(smite.place_overlap_range, 300.0 + 8.0 * 7.0);

        assert!(smite.drawer.contains("DrawTurret(reinforced-"));
        assert_eq!(smite.drawer.matches("RegionPart(").count(), 11);
        assert_eq!(smite.drawer.matches("ShapePart(").count(), 2);
        assert_eq!(smite.drawer.matches("HaloPart(").count(), 5);
        assert_eq!(smite.drawer.matches("RegionPart(-blade-bar").count(), 3);
        assert_eq!(smite.drawer.matches("RegionPart(-spine").count(), 4);
        assert!(smite.drawer.contains("heat.blend(warmup,0.5)"));
        assert!(smite.drawer.contains("warmup.delay(0.5)"));
        assert!(smite.drawer.contains("triLengthTo=20"));

        let surge = ammo_for(smite, item_id("surge-alloy"));
        let bullet = &surge.bullet;
        assert_eq!(bullet.kind, BulletKind::Basic);
        assert_eq!(bullet.speed, 7.0);
        assert_eq!(bullet.damage, 250.0);
        assert_close(bullet.lifetime, (300.0 + 9.0 + 10.0) / 7.0);
        assert_eq!(bullet.sprite, "large-orb");
        assert_eq!(bullet.width, 17.0);
        assert_eq!(bullet.height, 21.0);
        assert_eq!(bullet.hit_size, 8.0);
        assert!(bullet.shoot_effect.contains("shootTitan"));
        assert!(bullet.shoot_effect.contains("colorSparkBig"));
        assert!(bullet.shoot_effect.contains("strokeTo=0.3"));
        assert_eq!(bullet.smoke_effect, "shootSmokeSmite");
        assert_eq!(bullet.ammo_multiplier, 1.0);
        assert_eq!(bullet.pierce_cap, 4);
        assert!(bullet.pierce);
        assert!(bullet.pierce_building);
        assert_eq!(bullet.hit_color, "accent");
        assert_eq!(bullet.back_color, "accent");
        assert_eq!(bullet.trail_color, "accent");
        assert_eq!(bullet.front_color, "white");
        assert_eq!(bullet.trail_width, 2.8);
        assert_eq!(bullet.trail_length, 9);
        assert_eq!(bullet.hit_effect, "hitBulletColor");
        assert_close(bullet.building_damage_multiplier, 0.3);
        assert!(bullet.despawn_effect.contains("hitBulletColor"));
        assert!(bullet.despawn_effect.contains("sizeTo=30"));
        assert!(bullet.trail_rotation);
        assert_eq!(bullet.trail_effect, "disperseTrail");
        assert_eq!(bullet.trail_interval, 3.0);
        assert_eq!(bullet.bullet_interval, 3.0);

        let interval = bullet.interval_bullet.as_ref().unwrap();
        assert_eq!(interval.kind, BulletKind::Lightning);
        assert_eq!(interval.damage, 30.0);
        assert!(!interval.collides_air);
        assert_eq!(interval.ammo_multiplier, 1.0);
        assert_eq!(interval.lightning_color, "accent");
        assert_eq!(interval.lightning_length, 5);
        assert_eq!(interval.lightning_length_rand, 10);
        assert_close(interval.building_damage_multiplier, 0.25);
        assert_eq!(interval.status, "shocked");
        assert!(!interval.keep_velocity);
        assert!(!interval.hittable);

        let lightning_type = interval.lightning_type.as_ref().unwrap();
        assert_eq!(lightning_type.kind, BulletKind::Generic);
        assert_eq!(lightning_type.speed, 0.0001);
        assert_eq!(lightning_type.damage, 0.0);
        assert_eq!(lightning_type.lifetime, 10.0);
        assert_eq!(lightning_type.hit_effect, "hitLancer");
        assert_eq!(lightning_type.despawn_effect, "none");
        assert_eq!(lightning_type.status, "shocked");
        assert!(!lightning_type.hittable);
        assert_eq!(lightning_type.light_color, "ffffffff");
        assert_close(lightning_type.building_damage_multiplier, 0.25);
    }

    #[test]
    fn malign_power_turret_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let malign = registry.get_turret_by_name("malign").unwrap();
        assert_eq!(malign.kind, TurretBlockKind::PowerTurret);
        assert_eq!(
            malign.requirements,
            vec![
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 1000
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 500
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 500
                },
                ItemAmount {
                    item: item_id("phase-fabric"),
                    amount: 200
                }
            ]
        );
        assert!(malign.ammo.is_empty());
        assert!(!malign.consume_coolant);
        assert!(malign.consume_liquids.is_empty());
        assert_eq!(malign.consume_power, 40.0);
        assert!(malign.base.has_power);
        assert_eq!(malign.base.size, 5);
        assert_ne!(malign.base.env_enabled & Env::SPACE, 0);
        assert_eq!(malign.shoot_sound, "shootMalign");
        assert_eq!(malign.loop_sound, "loopMalign");
        assert_eq!(malign.loop_sound_volume, 1.3);
        assert_eq!(malign.velocity_rnd, 0.15);
        assert_eq!(malign.heat_requirement, 144.0);
        assert_eq!(malign.max_heat_efficiency, 1.0);
        assert_eq!(malign.warmup_maintain_time, 120.0);
        assert_eq!(malign.unit_sort, "strongest");
        assert_eq!(malign.shoot_pattern, "ShootSummon");
        assert_eq!(malign.shoot_summon_x, 0.0);
        assert_eq!(malign.shoot_summon_y, 0.0);
        assert_eq!(malign.shoot_summon_radius, 11.0);
        assert_eq!(malign.shoot_summon_spread, 20.0);
        assert_eq!(malign.min_warmup, 0.96);
        assert_eq!(malign.shoot_warmup_speed, 0.08);
        assert_eq!(malign.shoot_y, 20.0);
        assert_eq!(malign.outline_color, "darkOutline");
        assert_eq!(malign.reload, 3.5);
        assert_eq!(malign.range, 410.0);
        assert_close(malign.tracking_range, 410.0 * 1.4);
        assert_eq!(malign.shoot_cone, 100.0);
        assert_eq!(malign.scaled_health, 370.0);
        assert_eq!(malign.base.health, 5 * 5 * 370);
        assert_eq!(malign.rotate_speed, 2.6);
        assert_eq!(malign.recoil, 0.5);
        assert_eq!(malign.recoil_time, 30.0);
        assert_eq!(malign.shake, 3.0);
        assert_eq!(malign.fog_radius, (410.0_f32 / 8.0).round());
        assert_eq!(malign.place_overlap_range, 410.0 + 8.0 * 7.0);

        assert!(malign.drawer.contains("DrawTurret(reinforced-"));
        assert_eq!(malign.drawer.matches("ShapePart(").count(), 6);
        assert_eq!(malign.drawer.matches("HaloPart(").count(), 6);
        assert_eq!(malign.drawer.matches("RegionPart(").count(), 8);
        assert_eq!(malign.drawer.matches("RegionPart(-spine").count(), 3);
        assert!(malign.drawer.contains("warmup.delay(0.9)"));
        assert!(malign.drawer.contains("heatColor=purple"));
        assert!(malign.drawer.contains("color=bb68c3"));
        assert!(malign.drawer.contains("haloRotateSpeed=-1.5"));

        let shoot = malign.shoot_type.as_ref().unwrap();
        assert_eq!(shoot.kind, BulletKind::Flak);
        assert_eq!(shoot.speed, 8.0);
        assert_eq!(shoot.damage, 70.0);
        assert_eq!(shoot.sprite, "missile-large");
        assert_eq!(shoot.lifetime, 40.0);
        assert_eq!(shoot.width, 12.0);
        assert_eq!(shoot.height, 22.0);
        assert_eq!(shoot.hit_size, 7.0);
        assert_eq!(shoot.shoot_effect, "shootSmokeSquareBig");
        assert_eq!(shoot.smoke_effect, "shootSmokeDisperse");
        assert_eq!(shoot.ammo_multiplier, 1.0);
        assert_eq!(shoot.hit_color, "d370d3");
        assert_eq!(shoot.back_color, "d370d3");
        assert_eq!(shoot.trail_color, "d370d3");
        assert_eq!(shoot.lightning_color, "d370d3");
        assert_eq!(shoot.front_color, "white");
        assert_eq!(shoot.trail_width, 3.0);
        assert_eq!(shoot.trail_length, 12);
        assert_eq!(shoot.hit_effect, "hitSquaresColor");
        assert_eq!(shoot.despawn_effect, "hitBulletColor");
        assert_close(shoot.building_damage_multiplier, 0.3);
        assert_eq!(shoot.trail_effect, "colorSpark");
        assert!(shoot.trail_rotation);
        assert_eq!(shoot.trail_interval, 3.0);
        assert_eq!(shoot.homing_power, 0.17);
        assert_eq!(shoot.homing_delay, 19.0);
        assert_eq!(shoot.homing_range, 160.0);
        assert_eq!(shoot.explode_range, 100.0);
        assert_eq!(shoot.explode_delay, 0.0);
        assert_eq!(shoot.flak_interval, 20.0);
        assert_eq!(shoot.despawn_shake, 3.0);
        assert_eq!(shoot.interval_bullets, 1);
        assert_eq!(shoot.frag_spread, 0.0);
        assert_eq!(shoot.frag_random_spread, 0.0);
        assert_eq!(shoot.interval_random_spread, 0.0);
        assert_eq!(shoot.bullet_interval, 20.0);
        assert_eq!(shoot.splash_damage, 0.0);
        assert!(shoot.collides_ground);

        let interval = shoot.interval_bullet.as_ref().unwrap();
        assert_eq!(interval.kind, BulletKind::Lightning);
        assert_eq!(interval.damage, 18.0);
        assert_eq!(interval.lightning_color, "d370d3");
        assert_eq!(interval.lightning_cone, 15.0);
        assert_eq!(interval.lightning_length, 35);
        assert_eq!(interval.lightning_length_rand, 5);
        assert_eq!(interval.status, "shocked");
        assert_eq!(interval.hit_effect, "hitLancer");
        assert!(!interval.keep_velocity);
        assert!(!interval.hittable);

        let frag = shoot.frag_bullet.as_ref().unwrap();
        assert_eq!(frag.kind, BulletKind::Laser);
        assert_eq!(frag.damage, 65.0);
        assert_eq!(frag.colors, vec!["d370d3@0.4", "d370d3", "white"]);
        assert_close(frag.building_damage_multiplier, 0.25);
        assert_eq!(frag.width, 19.0);
        assert_eq!(frag.hit_effect, "hitLancer");
        assert_eq!(frag.side_angle, 175.0);
        assert_eq!(frag.side_width, 1.0);
        assert_eq!(frag.side_length, 40.0);
        assert_eq!(frag.lifetime, 22.0);
        assert_eq!(frag.draw_size, 400.0);
        assert_eq!(frag.length, 120.0);
        assert_eq!(frag.pierce_cap, 2);
        assert_eq!(frag.optimal_life_fract, 1.0);
        assert!(frag.impact);
        assert!(frag.pierce);
        assert!(!frag.hittable);
        assert!(!frag.absorbable);
    }

    #[test]
    fn heavy_liquid_turrets_keep_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_content_id = |name: &str| {
            all_liquids
                .iter()
                .find(|liquid| liquid.base.mappable.name.as_str() == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };
        fn liquid_ammo_for(turret: &TurretBlockData, liquid: ContentId) -> &LiquidTurretAmmo {
            turret
                .liquid_ammo
                .iter()
                .find(|ammo| ammo.liquid == liquid)
                .unwrap()
        }
        let assert_close = |actual: f32, expected: f32| {
            assert!(
                (actual - expected).abs() < 0.0001,
                "expected {expected}, got {actual}"
            );
        };

        let tsunami = registry.get_turret_by_name("tsunami").unwrap();
        assert_eq!(tsunami.kind, TurretBlockKind::LiquidTurret);
        assert!(tsunami.base.has_liquids);
        assert!(tsunami.extinguish);
        assert_eq!(
            tsunami.base.flags,
            vec![BlockFlag::Turret, BlockFlag::Extinguisher]
        );
        assert_eq!(tsunami.base.size, 3);
        assert_eq!(tsunami.reload, 3.0);
        assert_eq!(tsunami.shoot_pattern, "ShootAlternate");
        assert_eq!(tsunami.shoot_alternate_spread, 4.0);
        assert_eq!(tsunami.shoot_shots, 2);
        assert_eq!(tsunami.velocity_rnd, 0.1);
        assert_eq!(tsunami.inaccuracy, 3.0);
        assert_eq!(tsunami.recoil, 1.0);
        assert_eq!(tsunami.shoot_cone, 45.0);
        assert_eq!(tsunami.liquid_capacity, 40.0);
        assert_eq!(tsunami.base.liquid_capacity, 40.0);
        assert_eq!(tsunami.shoot_effect, "shootLiquid");
        assert_eq!(tsunami.range, 190.0);
        assert_eq!(tsunami.scaled_health, 250.0);
        assert_eq!(tsunami.base.health, 3 * 3 * 250);
        assert_eq!(tsunami.fog_radius, 24.0);
        assert_eq!(tsunami.place_overlap_range, 190.0 + 8.0 * 7.0);
        assert_eq!(
            tsunami.requirements,
            vec![
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 400
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 100
                }
            ]
        );

        let water = &liquid_ammo_for(tsunami, liquid_content_id("water")).bullet;
        assert_eq!(water.kind, BulletKind::Liquid);
        assert_eq!(water.lifetime, 49.0);
        assert_eq!(water.speed, 4.0);
        assert_eq!(water.knockback, 1.7);
        assert_eq!(water.puddle_size, 8.0);
        assert_eq!(water.orb_size, 4.0);
        assert_eq!(water.drag, 0.001);
        assert_eq!(water.ammo_multiplier, 0.4);
        assert_eq!(water.status_duration, 60.0 * 4.0);
        assert_eq!(water.damage, 0.2);
        assert_eq!(water.layer, "Layer.bullet-2");
        assert_eq!(water.status, "wet");

        let slag = &liquid_ammo_for(tsunami, liquid_content_id("slag")).bullet;
        assert_eq!(slag.lifetime, 49.0);
        assert_eq!(slag.speed, 4.0);
        assert_eq!(slag.knockback, 1.3);
        assert_eq!(slag.puddle_size, 8.0);
        assert_eq!(slag.orb_size, 4.0);
        assert_close(slag.damage, 4.75);
        assert_eq!(slag.drag, 0.001);
        assert_eq!(slag.ammo_multiplier, 0.4);
        assert_eq!(slag.status_duration, 60.0 * 4.0);
        assert_eq!(slag.status, "melting");

        let cryofluid = &liquid_ammo_for(tsunami, liquid_content_id("cryofluid")).bullet;
        assert_eq!(cryofluid.lifetime, 49.0);
        assert_eq!(cryofluid.speed, 4.0);
        assert_eq!(cryofluid.knockback, 1.3);
        assert_eq!(cryofluid.puddle_size, 8.0);
        assert_eq!(cryofluid.orb_size, 4.0);
        assert_eq!(cryofluid.drag, 0.001);
        assert_eq!(cryofluid.ammo_multiplier, 0.4);
        assert_eq!(cryofluid.status_duration, 60.0 * 4.0);
        assert_eq!(cryofluid.damage, 0.2);
        assert_eq!(cryofluid.status, "freezing");

        let oil = &liquid_ammo_for(tsunami, liquid_content_id("oil")).bullet;
        assert_eq!(oil.lifetime, 49.0);
        assert_eq!(oil.speed, 4.0);
        assert_eq!(oil.knockback, 1.3);
        assert_eq!(oil.puddle_size, 8.0);
        assert_eq!(oil.orb_size, 4.0);
        assert_eq!(oil.drag, 0.001);
        assert_eq!(oil.ammo_multiplier, 0.4);
        assert_eq!(oil.status_duration, 60.0 * 4.0);
        assert_eq!(oil.damage, 0.2);
        assert_eq!(oil.layer, "Layer.bullet-2");
        assert_eq!(oil.status, "tarred");
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
    fn ground_factory_unit_factory_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let factory = registry.get_unit_factory_by_name("ground-factory").unwrap();

        assert_eq!(factory.kind, UnitBlockKind::UnitFactory);
        assert_eq!(factory.base.group, BlockGroup::Units);
        assert_eq!(
            factory.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert_eq!(factory.base.flags, vec![BlockFlag::Factory]);
        assert!(factory.base.update);
        assert!(factory.base.has_power);
        assert!(factory.base.has_items);
        assert!(factory.base.solid);
        assert!(factory.configurable);
        assert!(factory.clear_on_double_tap);
        assert!(factory.outputs_payload);
        assert!(factory.rotate);
        assert_eq!(factory.region_rotated1, 1);
        assert!(factory.commandable);
        assert_eq!(factory.ambient_sound, "loopUnitBuilding");
        assert_eq!(factory.ambient_sound_volume, 0.09);
        assert_eq!(factory.create_sound, "unitCreate");
        assert_eq!(factory.create_sound_volume, 1.0);
        assert_eq!(factory.base.size, 3);
        assert_eq!(factory.consume_power, 1.2);
        assert!(factory.base.consumes_power);
        assert_eq!(factory.research_cost_multiplier, 0.5);
        assert_eq!(factory.base.item_capacity, 60);
        assert_eq!(
            factory.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 120
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 80
                }
            ]
        );

        assert_eq!(factory.plans.len(), 3);
        assert_eq!(factory.plans[0].unit, "dagger");
        assert_eq!(factory.plans[0].time, 60.0 * 15.0);
        assert_eq!(
            factory.plans[0].requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 10
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 10
                }
            ]
        );
        assert_eq!(factory.plans[1].unit, "crawler");
        assert_eq!(factory.plans[1].time, 60.0 * 10.0);
        assert_eq!(
            factory.plans[1].requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 8
                },
                ItemAmount {
                    item: item_id("coal"),
                    amount: 10
                }
            ]
        );
        assert_eq!(factory.plans[2].unit, "nova");
        assert_eq!(factory.plans[2].time, 60.0 * 40.0);
        assert_eq!(
            factory.plans[2].requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 20
                }
            ]
        );
        assert_eq!(
            factory.capacities,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("coal"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 40
                }
            ]
        );
    }

    #[test]
    fn air_factory_unit_factory_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let factory = registry.get_unit_factory_by_name("air-factory").unwrap();

        assert_eq!(factory.kind, UnitBlockKind::UnitFactory);
        assert_eq!(factory.base.group, BlockGroup::Units);
        assert_eq!(
            factory.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert_eq!(factory.base.flags, vec![BlockFlag::Factory]);
        assert!(factory.base.update);
        assert!(factory.base.has_power);
        assert!(factory.base.has_items);
        assert!(factory.base.solid);
        assert!(!factory.floating);
        assert!(factory.configurable);
        assert!(factory.clear_on_double_tap);
        assert!(factory.outputs_payload);
        assert!(factory.rotate);
        assert_eq!(factory.region_rotated1, 1);
        assert!(factory.commandable);
        assert_eq!(factory.ambient_sound, "loopUnitBuilding");
        assert_eq!(factory.ambient_sound_volume, 0.09);
        assert_eq!(factory.create_sound, "unitCreate");
        assert_eq!(factory.create_sound_volume, 1.0);
        assert_eq!(factory.base.size, 3);
        assert_eq!(factory.consume_power, 1.2);
        assert!(factory.base.consumes_power);
        assert_eq!(factory.research_cost_multiplier, 0.5);
        assert_eq!(factory.base.item_capacity, 60);
        assert_eq!(
            factory.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 70
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 60
                }
            ]
        );

        assert_eq!(factory.plans.len(), 2);
        assert_eq!(factory.plans[0].unit, "flare");
        assert_eq!(factory.plans[0].time, 60.0 * 15.0);
        assert_eq!(
            factory.plans[0].requirements,
            vec![ItemAmount {
                item: item_id("silicon"),
                amount: 15
            }]
        );
        assert_eq!(factory.plans[1].unit, "mono");
        assert_eq!(factory.plans[1].time, 60.0 * 35.0);
        assert_eq!(
            factory.plans[1].requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 30
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 15
                }
            ]
        );
        assert_eq!(
            factory.capacities,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 60
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 30
                }
            ]
        );
    }

    #[test]
    fn naval_factory_unit_factory_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let factory = registry.get_unit_factory_by_name("naval-factory").unwrap();

        assert_eq!(factory.kind, UnitBlockKind::UnitFactory);
        assert_eq!(factory.base.group, BlockGroup::Units);
        assert_eq!(
            factory.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert_eq!(factory.base.flags, vec![BlockFlag::Factory]);
        assert!(factory.base.update);
        assert!(factory.base.has_power);
        assert!(factory.base.has_items);
        assert!(factory.base.solid);
        assert!(factory.floating);
        assert!(factory.configurable);
        assert!(factory.clear_on_double_tap);
        assert!(factory.outputs_payload);
        assert!(factory.rotate);
        assert_eq!(factory.region_rotated1, 1);
        assert!(factory.commandable);
        assert_eq!(factory.ambient_sound, "loopUnitBuilding");
        assert_eq!(factory.ambient_sound_volume, 0.09);
        assert_eq!(factory.create_sound, "unitCreate");
        assert_eq!(factory.create_sound_volume, 1.0);
        assert_eq!(factory.base.size, 3);
        assert_eq!(factory.consume_power, 1.2);
        assert!(factory.base.consumes_power);
        assert_eq!(factory.research_cost_multiplier, 1.0);
        assert_eq!(factory.base.item_capacity, 70);
        assert_eq!(
            factory.requirements,
            vec![
                ItemAmount {
                    item: item_id("copper"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("lead"),
                    amount: 130
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 120
                }
            ]
        );

        assert_eq!(factory.plans.len(), 2);
        assert_eq!(factory.plans[0].unit, "risso");
        assert_eq!(factory.plans[0].time, 60.0 * 45.0);
        assert_eq!(
            factory.plans[0].requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 20
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 35
                }
            ]
        );
        assert_eq!(factory.plans[1].unit, "retusa");
        assert_eq!(factory.plans[1].time, 60.0 * 35.0);
        assert_eq!(
            factory.plans[1].requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 15
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 20
                }
            ]
        );
        assert_eq!(
            factory.capacities,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("metaglass"),
                    amount: 70
                },
                ItemAmount {
                    item: item_id("titanium"),
                    amount: 40
                }
            ]
        );
    }

    #[test]
    fn tank_fabricator_unit_factory_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let factory = registry
            .get_unit_factory_by_name("tank-fabricator")
            .unwrap();

        assert_eq!(factory.kind, UnitBlockKind::UnitFactory);
        assert_eq!(factory.base.group, BlockGroup::Units);
        assert_eq!(factory.base.flags, vec![BlockFlag::Factory]);
        assert!(factory.base.update);
        assert!(factory.base.has_power);
        assert!(factory.base.has_items);
        assert!(factory.base.solid);
        assert!(!factory.configurable);
        assert!(factory.clear_on_double_tap);
        assert!(factory.outputs_payload);
        assert!(!factory.floating);
        assert!(factory.rotate);
        assert_eq!(factory.region_rotated1, 1);
        assert_eq!(factory.region_suffix, "-dark");
        assert_eq!(factory.fog_radius, 3.0);
        assert!(factory.commandable);
        assert_eq!(factory.ambient_sound, "loopUnitBuilding");
        assert_eq!(factory.ambient_sound_volume, 0.09);
        assert_eq!(factory.create_sound, "unitCreate");
        assert_eq!(factory.create_sound_volume, 1.0);
        assert_eq!(factory.base.size, 3);
        assert_eq!(factory.consume_power, 1.5);
        assert!(factory.base.consumes_power);
        assert_eq!(factory.research_cost_multiplier, 1.0);
        assert_eq!(factory.base.item_capacity, 100);
        assert_eq!(
            factory.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 150
                }
            ]
        );
        assert_eq!(
            factory.research_cost,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 80
                }
            ]
        );

        assert_eq!(factory.plans.len(), 1);
        assert_eq!(factory.plans[0].unit, "stell");
        assert_eq!(factory.plans[0].time, 60.0 * 35.0);
        assert_eq!(
            factory.plans[0].requirements,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 40
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 50
                }
            ]
        );
        assert_eq!(
            factory.capacities,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 100
                }
            ]
        );
    }

    #[test]
    fn ship_fabricator_unit_factory_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let factory = registry
            .get_unit_factory_by_name("ship-fabricator")
            .unwrap();

        assert_eq!(factory.kind, UnitBlockKind::UnitFactory);
        assert_eq!(factory.base.group, BlockGroup::Units);
        assert_eq!(factory.base.flags, vec![BlockFlag::Factory]);
        assert!(factory.base.update);
        assert!(factory.base.has_power);
        assert!(factory.base.has_items);
        assert!(factory.base.solid);
        assert!(!factory.configurable);
        assert!(factory.clear_on_double_tap);
        assert!(factory.outputs_payload);
        assert!(!factory.floating);
        assert!(factory.rotate);
        assert_eq!(factory.region_rotated1, 1);
        assert_eq!(factory.region_suffix, "-dark");
        assert_eq!(factory.fog_radius, 3.0);
        assert!(factory.commandable);
        assert_eq!(factory.ambient_sound, "loopUnitBuilding");
        assert_eq!(factory.ambient_sound_volume, 0.09);
        assert_eq!(factory.create_sound, "unitCreate");
        assert_eq!(factory.create_sound_volume, 1.0);
        assert_eq!(factory.base.size, 3);
        assert_eq!(factory.consume_power, 1.5);
        assert!(factory.base.consumes_power);
        assert_eq!(factory.research_cost_multiplier, 0.5);
        assert!(factory.research_cost.is_empty());
        assert_eq!(factory.base.item_capacity, 140);
        assert_eq!(
            factory.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 200
                }
            ]
        );

        assert_eq!(factory.plans.len(), 1);
        assert_eq!(factory.plans[0].unit, "elude");
        assert_eq!(factory.plans[0].time, 60.0 * 40.0);
        assert_eq!(
            factory.plans[0].requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 70
                }
            ]
        );
        assert_eq!(
            factory.capacities,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 140
                }
            ]
        );
    }

    #[test]
    fn mech_fabricator_unit_factory_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let factory = registry
            .get_unit_factory_by_name("mech-fabricator")
            .unwrap();

        assert_eq!(factory.kind, UnitBlockKind::UnitFactory);
        assert_eq!(factory.base.group, BlockGroup::Units);
        assert_eq!(factory.base.flags, vec![BlockFlag::Factory]);
        assert!(factory.base.update);
        assert!(factory.base.has_power);
        assert!(factory.base.has_items);
        assert!(factory.base.solid);
        assert!(!factory.configurable);
        assert!(factory.clear_on_double_tap);
        assert!(factory.outputs_payload);
        assert!(!factory.floating);
        assert!(factory.rotate);
        assert_eq!(factory.region_rotated1, 1);
        assert_eq!(factory.region_suffix, "-dark");
        assert_eq!(factory.fog_radius, 3.0);
        assert!(factory.commandable);
        assert_eq!(factory.ambient_sound, "loopUnitBuilding");
        assert_eq!(factory.ambient_sound_volume, 0.09);
        assert_eq!(factory.create_sound, "unitCreate");
        assert_eq!(factory.create_sound_volume, 1.0);
        assert_eq!(factory.base.size, 3);
        assert_eq!(factory.consume_power, 1.5);
        assert!(factory.base.consumes_power);
        assert_eq!(factory.research_cost_multiplier, 0.65);
        assert!(factory.research_cost.is_empty());
        assert_eq!(factory.base.item_capacity, 140);
        assert_eq!(
            factory.requirements,
            vec![
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 10
                }
            ]
        );

        assert_eq!(factory.plans.len(), 1);
        assert_eq!(factory.plans[0].unit, "merui");
        assert_eq!(factory.plans[0].time, 60.0 * 40.0);
        assert_eq!(
            factory.plans[0].requirements,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 50
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 70
                }
            ]
        );
        assert_eq!(
            factory.capacities,
            vec![
                ItemAmount {
                    item: item_id("beryllium"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 140
                }
            ]
        );
    }

    #[test]
    fn tank_assembler_unit_assembler_keeps_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| liquid_id(&all_liquids, name).unwrap();
        let assembler = registry
            .get_unit_assembler_by_name("tank-assembler")
            .unwrap();

        assert_eq!(assembler.kind, UnitBlockKind::UnitAssembler);
        assert_eq!(assembler.base.group, BlockGroup::Units);
        assert_eq!(assembler.base.flags, vec![BlockFlag::UnitAssembler]);
        assert_eq!(
            assembler.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(assembler.base.update);
        assert!(assembler.base.sync);
        assert!(assembler.base.solid);
        assert!(assembler.base.has_items);
        assert!(assembler.base.has_power);
        assert!(assembler.base.has_liquids);
        assert!(assembler.base.consumes_power);
        assert_eq!(assembler.base.size, 5);
        assert_eq!(assembler.base.item_capacity, 10);
        assert_eq!(assembler.consume_power, 2.5);
        assert_eq!(assembler.consume_liquids.len(), 1);
        assert_eq!(
            assembler.consume_liquids[0],
            LiquidAmount {
                liquid: liquid_id("cyanogen"),
                amount: 9.0 / 60.0
            }
        );
        assert_eq!(assembler.region_suffix, "-dark");
        assert_eq!(assembler.area_size, 13);
        assert_eq!(assembler.research_cost_multiplier, 0.4);
        assert!(assembler.research_cost.is_empty());
        assert!(!assembler.outputs_payload);
        assert!(assembler.accepts_payload);
        assert!(assembler.accepts_unit_payloads);
        assert!(!assembler.floating);
        assert!(assembler.rotate);
        assert!(!assembler.rotate_draw);
        assert!(!assembler.quick_rotate);
        assert_eq!(assembler.region_rotated1, 1);
        assert!(assembler.commandable);
        assert_eq!(assembler.ambient_sound, "loopUnitBuilding");
        assert_eq!(assembler.ambient_sound_volume, 0.13);
        assert_eq!(assembler.create_sound, "unitCreateBig");
        assert_eq!(assembler.create_sound_volume, 1.0);
        assert_eq!(assembler.payload_speed, 0.7);
        assert_eq!(assembler.payload_rotate_speed, 5.0);
        assert_eq!(assembler.drone_type, "assembly-drone");
        assert_eq!(assembler.drones_created, 4);
        assert_eq!(assembler.drone_construct_time, 60.0 * 4.0);
        assert!(assembler.capacities.is_empty());
        assert!(assembler.liquid_filter.is_empty());
        assert_eq!(
            assembler.requirements,
            vec![
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 500
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 150
                },
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 80
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 650
                }
            ]
        );

        assert_eq!(assembler.plans.len(), 2);
        assert_eq!(assembler.plans[0].unit, "vanquish");
        assert_eq!(assembler.plans[0].time, 60.0 * 50.0);
        assert_eq!(
            assembler.plans[0].payload_requirements,
            vec![
                PayloadStackSpec {
                    content: PayloadContentSpec::Unit("stell".into()),
                    amount: 4
                },
                PayloadStackSpec {
                    content: PayloadContentSpec::Block("tungsten-wall-large".into()),
                    amount: 10
                }
            ]
        );
        assert!(assembler.plans[0].item_requirements.is_empty());
        assert!(assembler.plans[0].liquid_requirements.is_empty());
        assert_eq!(assembler.plans[1].unit, "conquer");
        assert_eq!(assembler.plans[1].time, 60.0 * 60.0 * 3.0);
        assert_eq!(
            assembler.plans[1].payload_requirements,
            vec![
                PayloadStackSpec {
                    content: PayloadContentSpec::Unit("locus".into()),
                    amount: 6
                },
                PayloadStackSpec {
                    content: PayloadContentSpec::Block("carbide-wall-large".into()),
                    amount: 20
                }
            ]
        );
        assert!(assembler.plans[1].item_requirements.is_empty());
        assert!(assembler.plans[1].liquid_requirements.is_empty());
    }

    #[test]
    fn ship_assembler_unit_assembler_keeps_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| liquid_id(&all_liquids, name).unwrap();
        let assembler = registry
            .get_unit_assembler_by_name("ship-assembler")
            .unwrap();

        assert_eq!(assembler.kind, UnitBlockKind::UnitAssembler);
        assert_eq!(assembler.base.group, BlockGroup::Units);
        assert_eq!(assembler.base.flags, vec![BlockFlag::UnitAssembler]);
        assert_eq!(
            assembler.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(assembler.base.update);
        assert!(assembler.base.sync);
        assert!(assembler.base.solid);
        assert!(assembler.base.has_items);
        assert!(assembler.base.has_power);
        assert!(assembler.base.has_liquids);
        assert!(assembler.base.consumes_power);
        assert_eq!(assembler.base.size, 5);
        assert_eq!(assembler.base.item_capacity, 10);
        assert_eq!(assembler.consume_power, 2.5);
        assert_eq!(
            assembler.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("cyanogen"),
                amount: 12.0 / 60.0
            }]
        );
        assert_eq!(assembler.region_suffix, "-dark");
        assert_eq!(assembler.area_size, 13);
        assert_eq!(assembler.research_cost_multiplier, 1.0);
        assert!(assembler.research_cost.is_empty());
        assert!(!assembler.outputs_payload);
        assert!(assembler.accepts_payload);
        assert!(assembler.accepts_unit_payloads);
        assert!(!assembler.floating);
        assert!(assembler.rotate);
        assert!(!assembler.rotate_draw);
        assert!(!assembler.quick_rotate);
        assert_eq!(assembler.region_rotated1, 1);
        assert!(assembler.commandable);
        assert_eq!(assembler.ambient_sound, "loopUnitBuilding");
        assert_eq!(assembler.ambient_sound_volume, 0.13);
        assert_eq!(assembler.create_sound, "unitCreateBig");
        assert_eq!(assembler.create_sound_volume, 1.0);
        assert_eq!(assembler.drone_type, "assembly-drone");
        assert_eq!(assembler.drones_created, 4);
        assert_eq!(assembler.drone_construct_time, 60.0 * 4.0);
        assert!(assembler.capacities.is_empty());
        assert!(assembler.liquid_filter.is_empty());
        assert_eq!(
            assembler.requirements,
            vec![
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 100
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 550
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 900
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 400
                }
            ]
        );

        assert_eq!(assembler.plans.len(), 2);
        assert_eq!(assembler.plans[0].unit, "quell");
        assert_eq!(assembler.plans[0].time, 60.0 * 60.0);
        assert_eq!(
            assembler.plans[0].payload_requirements,
            vec![
                PayloadStackSpec {
                    content: PayloadContentSpec::Unit("elude".into()),
                    amount: 4
                },
                PayloadStackSpec {
                    content: PayloadContentSpec::Block("beryllium-wall-large".into()),
                    amount: 12
                }
            ]
        );
        assert!(assembler.plans[0].item_requirements.is_empty());
        assert!(assembler.plans[0].liquid_requirements.is_empty());
        assert_eq!(assembler.plans[1].unit, "disrupt");
        assert_eq!(assembler.plans[1].time, 60.0 * 60.0 * 3.0);
        assert_eq!(
            assembler.plans[1].payload_requirements,
            vec![
                PayloadStackSpec {
                    content: PayloadContentSpec::Unit("avert".into()),
                    amount: 6
                },
                PayloadStackSpec {
                    content: PayloadContentSpec::Block("carbide-wall-large".into()),
                    amount: 20
                }
            ]
        );
        assert!(assembler.plans[1].item_requirements.is_empty());
        assert!(assembler.plans[1].liquid_requirements.is_empty());
    }

    #[test]
    fn mech_assembler_unit_assembler_keeps_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| liquid_id(&all_liquids, name).unwrap();
        let assembler = registry
            .get_unit_assembler_by_name("mech-assembler")
            .unwrap();

        assert_eq!(assembler.kind, UnitBlockKind::UnitAssembler);
        assert_eq!(assembler.base.group, BlockGroup::Units);
        assert_eq!(assembler.base.flags, vec![BlockFlag::UnitAssembler]);
        assert_eq!(
            assembler.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(assembler.base.update);
        assert!(assembler.base.sync);
        assert!(assembler.base.solid);
        assert!(assembler.base.has_items);
        assert!(assembler.base.has_power);
        assert!(assembler.base.has_liquids);
        assert!(assembler.base.consumes_power);
        assert_eq!(assembler.base.size, 5);
        assert_eq!(assembler.base.item_capacity, 10);
        assert_eq!(assembler.consume_power, 3.0);
        assert_eq!(
            assembler.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("cyanogen"),
                amount: 12.0 / 60.0
            }]
        );
        assert_eq!(assembler.region_suffix, "-dark");
        assert_eq!(assembler.area_size, 13);
        assert_eq!(assembler.research_cost_multiplier, 1.0);
        assert!(assembler.research_cost.is_empty());
        assert!(!assembler.outputs_payload);
        assert!(assembler.accepts_payload);
        assert!(assembler.accepts_unit_payloads);
        assert!(!assembler.floating);
        assert!(assembler.rotate);
        assert!(!assembler.rotate_draw);
        assert!(!assembler.quick_rotate);
        assert_eq!(assembler.region_rotated1, 1);
        assert!(assembler.commandable);
        assert_eq!(assembler.ambient_sound, "loopUnitBuilding");
        assert_eq!(assembler.ambient_sound_volume, 0.13);
        assert_eq!(assembler.create_sound, "unitCreateBig");
        assert_eq!(assembler.create_sound_volume, 1.0);
        assert_eq!(assembler.drone_type, "assembly-drone");
        assert_eq!(assembler.drones_created, 4);
        assert_eq!(assembler.drone_construct_time, 60.0 * 4.0);
        assert!(assembler.capacities.is_empty());
        assert!(assembler.liquid_filter.is_empty());
        assert_eq!(
            assembler.requirements,
            vec![
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 600
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 200
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 550
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 1000
                }
            ]
        );

        assert_eq!(assembler.plans.len(), 2);
        assert_eq!(assembler.plans[0].unit, "tecta");
        assert_eq!(assembler.plans[0].time, 60.0 * 70.0);
        assert_eq!(
            assembler.plans[0].payload_requirements,
            vec![
                PayloadStackSpec {
                    content: PayloadContentSpec::Unit("merui".into()),
                    amount: 5
                },
                PayloadStackSpec {
                    content: PayloadContentSpec::Block("tungsten-wall-large".into()),
                    amount: 12
                }
            ]
        );
        assert!(assembler.plans[0].item_requirements.is_empty());
        assert!(assembler.plans[0].liquid_requirements.is_empty());
        assert_eq!(assembler.plans[1].unit, "collaris");
        assert_eq!(assembler.plans[1].time, 60.0 * 60.0 * 3.0);
        assert_eq!(
            assembler.plans[1].payload_requirements,
            vec![
                PayloadStackSpec {
                    content: PayloadContentSpec::Unit("cleroi".into()),
                    amount: 6
                },
                PayloadStackSpec {
                    content: PayloadContentSpec::Block("carbide-wall-large".into()),
                    amount: 20
                }
            ]
        );
        assert!(assembler.plans[1].item_requirements.is_empty());
        assert!(assembler.plans[1].liquid_requirements.is_empty());
    }

    #[test]
    fn basic_assembler_module_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let module = registry
            .get_unit_assembler_module_by_name("basic-assembler-module")
            .unwrap();

        assert_eq!(module.kind, UnitBlockKind::UnitAssemblerModule);
        assert_eq!(module.base.group, BlockGroup::Payloads);
        assert_eq!(
            module.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(module.base.update);
        assert!(module.base.sync);
        assert!(module.base.has_power);
        assert!(module.base.consumes_power);
        assert!(!module.base.solid);
        assert_eq!(module.base.size, 5);
        assert!(module.base.flags.is_empty());
        assert_eq!(module.consume_power, 3.5);
        assert_eq!(module.region_suffix, "-dark");
        assert_eq!(module.research_cost_multiplier, 0.75);
        assert!(module.research_cost.is_empty());
        assert_eq!(module.tier, 1);
        assert!(module.accepts_payload);
        assert!(module.accepts_unit_payloads);
        assert!(!module.floating);
        assert!(module.rotate);
        assert!(!module.rotate_draw);
        assert_eq!(module.region_rotated1, -1);
        assert_eq!(module.payload_speed, 0.7);
        assert_eq!(module.payload_rotate_speed, 5.0);
        assert_eq!(
            module.requirements,
            vec![
                ItemAmount {
                    item: item_id("carbide"),
                    amount: 300
                },
                ItemAmount {
                    item: item_id("thorium"),
                    amount: 500
                },
                ItemAmount {
                    item: item_id("oxide"),
                    amount: 250
                },
                ItemAmount {
                    item: item_id("phase-fabric"),
                    amount: 400
                }
            ]
        );
    }

    #[test]
    fn unit_repair_tower_keeps_upstream_subset() {
        let (all_items, all_liquids, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let liquid_id = |name: &str| liquid_id(&all_liquids, name).unwrap();
        let tower = registry
            .get_unit_repair_tower_by_name("unit-repair-tower")
            .unwrap();

        assert_eq!(tower.kind, UnitBlockKind::RepairTower);
        assert!(tower.base.update);
        assert!(tower.base.solid);
        assert!(tower.suppressable);
        assert_eq!(tower.base.flags, vec![BlockFlag::Repair]);
        assert_eq!(tower.base.size, 2);
        assert!(tower.base.has_power);
        assert!(tower.base.consumes_power);
        assert!(tower.base.has_liquids);
        assert_eq!(tower.consume_power, 1.0);
        assert_eq!(
            tower.consume_liquids,
            vec![LiquidAmount {
                liquid: liquid_id("ozone"),
                amount: 3.0 / 60.0
            }]
        );
        assert_eq!(tower.range, 100.0);
        assert_eq!(tower.heal_amount, 1.5);
        assert_eq!(tower.research_cost_multiplier, 1.0);
        assert!(tower.research_cost.is_empty());
        assert_eq!(tower.circle_color, "heal");
        assert_eq!(tower.glow_color, "heal@0.5");
        assert_eq!(tower.circle_speed, 120.0);
        assert_eq!(tower.circle_stroke, 3.0);
        assert_eq!(tower.square_rad, 3.0);
        assert_eq!(tower.square_spin_scl, 0.8);
        assert_eq!(tower.glow_mag, 0.5);
        assert_eq!(tower.glow_scl, 8.0);
        assert_eq!(
            tower.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 90
                },
                ItemAmount {
                    item: item_id("silicon"),
                    amount: 90
                },
                ItemAmount {
                    item: item_id("tungsten"),
                    amount: 80
                }
            ]
        );
    }

    #[test]
    fn payload_conveyor_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let payload = registry.get_payload_by_name("payload-conveyor").unwrap();

        assert_eq!(payload.kind, PayloadBlockKind::PayloadConveyor);
        assert_eq!(payload.base.group, BlockGroup::Payloads);
        assert_eq!(payload.base.size, 3);
        assert!(payload.base.update);
        assert!(payload.base.sync);
        assert_eq!(payload.base.priority, -1);
        assert_eq!(
            payload.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(payload.rotate);
        assert!(payload.outputs_payload);
        assert!(!payload.accepts_payload);
        assert!(payload.accepts_unit_payloads);
        assert!(payload.output_facing);
        assert!(payload.no_update_disabled);
        assert!(payload.under_bullets);
        assert!(!payload.can_overdrive);
        assert!(!payload.configurable);
        assert!(!payload.clear_on_double_tap);
        assert!(!payload.invert);
        assert_eq!(payload.move_time, 45.0);
        assert_eq!(payload.move_force, 201.0);
        assert_eq!(payload.interp, "pow5");
        assert_eq!(payload.payload_limit, 3.0);
        assert!(payload.push_units);
        assert_eq!(payload.research_cost_multiplier, 1.0);
        assert!(payload.research_cost.is_empty());
        assert_eq!(
            payload.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 10
                },
                ItemAmount {
                    item: item_id("copper"),
                    amount: 10
                }
            ]
        );
    }

    #[test]
    fn payload_router_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let payload = registry.get_payload_by_name("payload-router").unwrap();

        assert_eq!(payload.kind, PayloadBlockKind::PayloadRouter);
        assert_eq!(payload.base.group, BlockGroup::Payloads);
        assert_eq!(payload.base.size, 3);
        assert!(payload.base.update);
        assert!(payload.base.sync);
        assert_eq!(payload.base.priority, -1);
        assert_eq!(
            payload.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(payload.rotate);
        assert!(payload.outputs_payload);
        assert!(!payload.accepts_payload);
        assert!(payload.accepts_unit_payloads);
        assert!(!payload.output_facing);
        assert!(payload.no_update_disabled);
        assert!(payload.under_bullets);
        assert!(!payload.can_overdrive);
        assert!(payload.configurable);
        assert!(payload.clear_on_double_tap);
        assert!(!payload.invert);
        assert_eq!(payload.move_time, 45.0);
        assert_eq!(payload.move_force, 201.0);
        assert_eq!(payload.interp, "pow5");
        assert_eq!(payload.payload_limit, 3.0);
        assert!(payload.push_units);
        assert_eq!(payload.research_cost_multiplier, 1.0);
        assert!(payload.research_cost.is_empty());
        assert_eq!(
            payload.requirements,
            vec![
                ItemAmount {
                    item: item_id("graphite"),
                    amount: 15
                },
                ItemAmount {
                    item: item_id("copper"),
                    amount: 10
                }
            ]
        );
    }

    #[test]
    fn reinforced_payload_conveyor_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let payload = registry
            .get_payload_by_name("reinforced-payload-conveyor")
            .unwrap();

        assert_eq!(payload.kind, PayloadBlockKind::PayloadConveyor);
        assert_eq!(payload.base.group, BlockGroup::Payloads);
        assert_eq!(payload.base.size, 3);
        assert_eq!(payload.base.health, 800);
        assert!(payload.base.update);
        assert!(payload.base.sync);
        assert_eq!(payload.base.priority, -1);
        assert_eq!(
            payload.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(payload.rotate);
        assert!(payload.outputs_payload);
        assert!(!payload.accepts_payload);
        assert!(payload.accepts_unit_payloads);
        assert!(payload.output_facing);
        assert!(payload.no_update_disabled);
        assert!(payload.under_bullets);
        assert!(!payload.can_overdrive);
        assert!(!payload.configurable);
        assert!(!payload.clear_on_double_tap);
        assert!(!payload.invert);
        assert_eq!(payload.move_time, 35.0);
        assert_eq!(payload.move_force, 201.0);
        assert_eq!(payload.interp, "pow5");
        assert_eq!(payload.payload_limit, 3.0);
        assert!(payload.push_units);
        assert_eq!(payload.research_cost_multiplier, 4.0);
        assert!(payload.research_cost.is_empty());
        assert_eq!(
            payload.requirements,
            vec![ItemAmount {
                item: item_id("tungsten"),
                amount: 10
            }]
        );
    }

    #[test]
    fn reinforced_payload_router_keeps_upstream_subset() {
        let (all_items, _, registry) = load_test_registry();
        let item_id = |name: &str| find_item(&all_items, name).unwrap().base.mappable.base.id;
        let payload = registry
            .get_payload_by_name("reinforced-payload-router")
            .unwrap();

        assert_eq!(payload.kind, PayloadBlockKind::PayloadRouter);
        assert_eq!(payload.base.group, BlockGroup::Payloads);
        assert_eq!(payload.base.size, 3);
        assert_eq!(payload.base.health, 800);
        assert!(payload.base.update);
        assert!(payload.base.sync);
        assert_eq!(payload.base.priority, -1);
        assert_eq!(
            payload.base.env_enabled,
            Env::TERRESTRIAL | Env::SPACE | Env::UNDERWATER
        );
        assert!(payload.rotate);
        assert!(payload.outputs_payload);
        assert!(!payload.accepts_payload);
        assert!(payload.accepts_unit_payloads);
        assert!(!payload.output_facing);
        assert!(payload.no_update_disabled);
        assert!(payload.under_bullets);
        assert!(!payload.can_overdrive);
        assert!(payload.configurable);
        assert!(payload.clear_on_double_tap);
        assert!(!payload.invert);
        assert_eq!(payload.move_time, 35.0);
        assert_eq!(payload.move_force, 201.0);
        assert_eq!(payload.interp, "pow5");
        assert_eq!(payload.payload_limit, 3.0);
        assert!(payload.push_units);
        assert_eq!(payload.research_cost_multiplier, 4.0);
        assert!(payload.research_cost.is_empty());
        assert_eq!(
            payload.requirements,
            vec![ItemAmount {
                item: item_id("tungsten"),
                amount: 15
            }]
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
