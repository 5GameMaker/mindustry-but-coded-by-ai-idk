use crate::mindustry::{
    ctype::{Content, ContentId, ContentType, UnlockableContentBase},
    entities::TextureRegionRef,
    logic::LAccess,
    r#type::Weapon,
    world::meta::Env,
};

pub const UNIT_SHADOW_TX: f32 = -12.0;
pub const UNIT_SHADOW_TY: f32 = -13.0;
pub const BUILDING_RANGE: f32 = 220.0;
pub const TILE_SIZE: f32 = 8.0;
pub const TILE_PAYLOAD: f32 = TILE_SIZE * TILE_SIZE;
pub const LAYER_GROUND_UNIT: f32 = 60.0;
pub const LAYER_FLYING_UNIT_LOW: f32 = 90.0;
pub const LAYER_FLYING_UNIT: f32 = 115.0;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitType {
    pub base: UnlockableContentBase,

    pub env_required: u32,
    pub env_enabled: u32,
    pub env_disabled: u32,

    pub speed: f32,
    pub boost_multiplier: f32,
    pub floor_multiplier: f32,
    pub rotate_speed: f32,
    pub base_rotate_speed: f32,
    pub drag: f32,
    pub accel: f32,
    pub hit_size: f32,
    pub death_shake: f32,
    pub step_shake: f32,
    pub ripple_scale: f32,
    pub rise_speed: f32,
    pub descent_speed: f32,
    pub fall_speed: f32,
    pub missile_accel_time: f32,
    pub health: f32,
    pub armor: f32,
    pub range: f32,
    pub max_range: f32,
    pub mine_range: f32,
    pub build_range: f32,
    pub circle_target_radius: f32,
    pub crash_damage_multiplier: f32,
    pub wreck_health_multiplier: f32,
    pub dps_estimate: f32,
    pub clip_size: f32,
    pub drown_time_multiplier: f32,
    pub strafe_penalty: f32,
    pub research_cost_multiplier: f32,
    pub ground_layer: f32,
    pub flying_layer: f32,
    pub payload_capacity: f32,
    pub build_speed: f32,
    pub aim_dst: f32,
    pub build_beam_offset: f32,
    pub mine_beam_offset: f32,
    pub target_priority: f32,
    pub shadow_elevation: f32,
    pub shadow_elevation_scl: f32,
    pub engine_offset: f32,
    pub engine_size: f32,
    pub engine_layer: f32,
    pub item_offset_y: f32,
    pub light_radius: f32,
    pub light_opacity: f32,
    pub soft_shadow_scl: f32,
    pub fog_radius: f32,
    pub wave_trail_x: f32,
    pub wave_trail_y: f32,
    pub trail_scl: f32,

    pub is_enemy: bool,
    pub flying: bool,
    pub wobble: bool,
    pub target_air: bool,
    pub target_ground: bool,
    pub face_target: bool,
    pub circle_target: bool,
    pub auto_drop_bombs: bool,
    pub target_buildings_mobile: bool,
    pub can_boost: bool,
    pub boost_when_building: bool,
    pub boost_when_mining: bool,
    pub logic_controllable: bool,
    pub player_controllable: bool,
    pub control_select_global: bool,
    pub allowed_in_payloads: bool,
    pub hittable: bool,
    pub killable: bool,
    pub targetable: bool,
    pub vulnerable_with_payloads: bool,
    pub pickup_units: bool,
    pub physics: bool,
    pub can_drown: bool,
    pub use_unit_cap: bool,
    pub core_unit_dock: bool,
    pub create_wreck: bool,
    pub create_scorch: bool,
    pub low_altitude: bool,
    pub rotate_to_building: bool,
    pub allow_leg_step: bool,
    pub leg_physics_layer: bool,
    pub hovering: bool,
    pub omni_movement: bool,
    pub rotate_move_first: bool,
    pub heal_flash: bool,
    pub can_heal: bool,
    pub single_target: bool,
    pub force_multi_target: bool,
    pub can_attack: bool,
    pub hidden: bool,
    pub internal: bool,
    pub internal_generate_sprites: bool,
    pub bounded: bool,
    pub naval: bool,
    pub auto_find_target: bool,
    pub target_under_blocks: bool,
    pub always_shoot_when_moving: bool,
    pub hoverable: bool,
    pub always_create_outline: bool,
    pub generate_full_icon: bool,
    pub square_shape: bool,
    pub draw_build_beam: bool,
    pub draw_mine_beam: bool,
    pub draw_cell: bool,
    pub draw_items: bool,
    pub draw_shields: bool,
    pub draw_body: bool,
    pub draw_soft_shadow: bool,
    pub draw_minimap: bool,

    pub abilities: Vec<String>,
    pub weapons: Vec<Weapon>,
    pub immunities: Vec<String>,

    pub heal_color_rgba: u32,
    pub light_color_rgba: u32,
    pub shield_color_rgba: Option<u32>,
    pub death_sound: String,
    pub death_sound_volume: f32,
    pub wreck_sound: String,
    pub wreck_sound_volume: f32,
    pub loop_sound: String,
    pub loop_sound_volume: f32,
    pub step_sound: String,
    pub step_sound_volume: f32,
    pub step_sound_pitch: f32,
    pub step_sound_pitch_range: f32,
    pub tank_move_sound: String,
    pub move_sound: String,
    pub move_sound_volume: f32,
    pub move_sound_pitch_min: f32,
    pub move_sound_pitch_max: f32,
    pub tank_move_volume: f32,
    pub fall_effect: String,
    pub fall_engine_effect: String,
    pub death_explosion_effect: String,
    pub tread_effect: Option<String>,
    pub parts: Vec<String>,
    pub engines: Vec<UnitEngine>,
    pub use_engine_elevation: bool,
    pub engine_color_rgba: Option<u32>,
    pub engine_color_inner_rgba: u32,
    pub trail_length: i32,
    pub trail_color_rgba: Option<u32>,

    pub flowfield_path_type: i32,
    pub path_cost: Option<String>,
    pub path_cost_id: i32,
    pub target_flags: Vec<Option<String>>,
    pub allow_change_commands: bool,
    pub commands: Vec<String>,
    pub default_command: Option<String>,
    pub stances: Vec<String>,

    pub outline_color_rgba: u32,
    pub outline_radius: i32,
    pub outlines: bool,
    pub item_capacity: i32,
    pub ammo_capacity: i32,
    pub ammo_type: String,
    pub mine_tier: i32,
    pub mine_speed: f32,
    pub mine_walls: bool,
    pub mine_floor: bool,
    pub mine_hardness_scaling: bool,
    pub mine_sound: String,
    pub mine_sound_volume: f32,
    pub mine_items: Vec<String>,

    pub leg_count: i32,
    pub leg_group_size: i32,
    pub leg_length: f32,
    pub leg_speed: f32,
    pub leg_forward_scl: f32,
    pub leg_base_offset: f32,
    pub leg_move_space: f32,
    pub leg_extension: f32,
    pub leg_pair_offset: f32,
    pub leg_length_scl: f32,
    pub leg_straight_length: f32,
    pub leg_max_length: f32,
    pub leg_min_length: f32,
    pub leg_splash_damage: f32,
    pub leg_splash_range: f32,
    pub leg_region: TextureRegionRef,
    pub leg_base_region: TextureRegionRef,
    pub base_leg_straightness: f32,
    pub leg_straightness: f32,
    pub leg_base_under: bool,
    pub lock_leg_base: bool,
    pub leg_continuous_move: bool,
    pub flip_back_legs: bool,
    pub flip_leg_side: bool,
    pub emit_walk_sound: bool,
    pub emit_walk_effect: bool,

    pub mech_land_shake: f32,
    pub mech_side_sway: f32,
    pub mech_front_sway: f32,
    pub mech_stride: f32,
    pub mech_step_particles: bool,
    pub mech_leg_color_rgba: u32,

    pub tread_frames: i32,
    pub tread_pull_offset: i32,
    pub crush_fragile: bool,

    pub segments: i32,
    pub segment_units: i32,
    pub segment_unit: Option<String>,
    pub segment_end_unit: Option<String>,
    pub segment_layer_order: bool,
    pub segment_mag: f32,
    pub segment_scl: f32,
    pub segment_phase: f32,
    pub segment_rot_speed: f32,
    pub segment_max_rot: f32,
    pub segment_spacing: f32,
    pub segment_rotation_range: f32,
    pub crawl_slowdown: f32,
    pub crush_damage: f32,
    pub crawl_slowdown_frac: f32,

    pub lifetime: f32,
    pub homing_delay: f32,
    pub build_time: f32,
}

impl UnitType {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        let mut base = UnlockableContentBase::new(id, ContentType::Unit, name);
        base.selection_size = 30.0;
        let atlas_name = base.mappable.name.clone();

        Self {
            base,
            env_required: Env::NONE,
            env_enabled: Env::TERRESTRIAL,
            env_disabled: Env::SCORCHING,
            speed: 1.1,
            boost_multiplier: 1.0,
            floor_multiplier: 1.0,
            rotate_speed: 5.0,
            base_rotate_speed: 5.0,
            drag: 0.3,
            accel: 0.5,
            hit_size: 6.0,
            death_shake: -1.0,
            step_shake: -1.0,
            ripple_scale: 1.0,
            rise_speed: 0.08,
            descent_speed: 0.08,
            fall_speed: 0.018,
            missile_accel_time: 0.0,
            health: 200.0,
            armor: 0.0,
            range: -1.0,
            max_range: -1.0,
            mine_range: 70.0,
            build_range: BUILDING_RANGE,
            circle_target_radius: 80.0,
            crash_damage_multiplier: 1.0,
            wreck_health_multiplier: 0.25,
            dps_estimate: -1.0,
            clip_size: -1.0,
            drown_time_multiplier: 1.0,
            strafe_penalty: 0.5,
            research_cost_multiplier: 50.0,
            ground_layer: LAYER_GROUND_UNIT,
            flying_layer: -1.0,
            payload_capacity: 8.0,
            build_speed: -1.0,
            aim_dst: -1.0,
            build_beam_offset: 3.8,
            mine_beam_offset: f32::NEG_INFINITY,
            target_priority: 0.0,
            shadow_elevation: -1.0,
            shadow_elevation_scl: 1.0,
            engine_offset: 5.0,
            engine_size: 2.5,
            engine_layer: -1.0,
            item_offset_y: 3.0,
            light_radius: -1.0,
            light_opacity: 0.6,
            soft_shadow_scl: 1.0,
            fog_radius: -1.0,
            wave_trail_x: 4.0,
            wave_trail_y: -3.0,
            trail_scl: 1.0,
            is_enemy: true,
            flying: false,
            wobble: true,
            target_air: true,
            target_ground: true,
            face_target: true,
            circle_target: false,
            auto_drop_bombs: false,
            target_buildings_mobile: true,
            can_boost: false,
            boost_when_building: true,
            boost_when_mining: true,
            logic_controllable: true,
            player_controllable: true,
            control_select_global: true,
            allowed_in_payloads: true,
            hittable: true,
            killable: true,
            targetable: true,
            vulnerable_with_payloads: false,
            pickup_units: true,
            physics: true,
            can_drown: true,
            use_unit_cap: true,
            core_unit_dock: false,
            create_wreck: true,
            create_scorch: true,
            low_altitude: false,
            rotate_to_building: true,
            allow_leg_step: false,
            leg_physics_layer: true,
            hovering: false,
            omni_movement: true,
            rotate_move_first: false,
            heal_flash: true,
            can_heal: false,
            single_target: false,
            force_multi_target: false,
            can_attack: true,
            hidden: false,
            internal: false,
            internal_generate_sprites: false,
            bounded: true,
            naval: false,
            auto_find_target: true,
            target_under_blocks: true,
            always_shoot_when_moving: false,
            hoverable: true,
            always_create_outline: false,
            generate_full_icon: true,
            square_shape: false,
            draw_build_beam: true,
            draw_mine_beam: true,
            draw_cell: true,
            draw_items: true,
            draw_shields: true,
            draw_body: true,
            draw_soft_shadow: true,
            draw_minimap: true,
            abilities: Vec::new(),
            weapons: Vec::new(),
            immunities: Vec::new(),
            heal_color_rgba: 0x98ffa9ff,
            light_color_rgba: 0xfbd367ff,
            shield_color_rgba: None,
            death_sound: "unset".into(),
            death_sound_volume: 1.0,
            wreck_sound: "unset".into(),
            wreck_sound_volume: 1.0,
            loop_sound: "none".into(),
            loop_sound_volume: 0.5,
            step_sound: "mechStepSmall".into(),
            step_sound_volume: 0.5,
            step_sound_pitch: 1.0,
            step_sound_pitch_range: 0.1,
            tank_move_sound: "tankMove".into(),
            move_sound: "none".into(),
            move_sound_volume: 1.0,
            move_sound_pitch_min: 1.0,
            move_sound_pitch_max: 1.0,
            tank_move_volume: 0.5,
            fall_effect: "fallSmoke".into(),
            fall_engine_effect: "fallSmoke".into(),
            death_explosion_effect: "dynamicExplosion".into(),
            tread_effect: None,
            parts: Vec::new(),
            engines: Vec::new(),
            use_engine_elevation: true,
            engine_color_rgba: None,
            engine_color_inner_rgba: 0xffffffff,
            trail_length: 0,
            trail_color_rgba: None,
            flowfield_path_type: -1,
            path_cost: None,
            path_cost_id: 0,
            target_flags: vec![None],
            allow_change_commands: true,
            commands: Vec::new(),
            default_command: None,
            stances: Vec::new(),
            outline_color_rgba: 0x2b2f36ff,
            outline_radius: 3,
            outlines: true,
            item_capacity: -1,
            ammo_capacity: -1,
            ammo_type: "item:copper".into(),
            mine_tier: -1,
            mine_speed: 1.0,
            mine_walls: false,
            mine_floor: true,
            mine_hardness_scaling: true,
            mine_sound: "loopMineBeam".into(),
            mine_sound_volume: 0.6,
            mine_items: ["copper", "lead", "titanium", "thorium"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            leg_count: 4,
            leg_group_size: 2,
            leg_length: 10.0,
            leg_speed: 0.1,
            leg_forward_scl: 1.0,
            leg_base_offset: 0.0,
            leg_move_space: 1.0,
            leg_extension: 0.0,
            leg_pair_offset: 0.0,
            leg_length_scl: 1.0,
            leg_straight_length: 1.0,
            leg_max_length: 1.75,
            leg_min_length: 0.0,
            leg_splash_damage: 0.0,
            leg_splash_range: 5.0,
            leg_region: TextureRegionRef::new(format!("{atlas_name}-leg")),
            leg_base_region: TextureRegionRef::new(format!("{atlas_name}-leg-base")),
            base_leg_straightness: 0.0,
            leg_straightness: 0.0,
            leg_base_under: false,
            lock_leg_base: false,
            leg_continuous_move: false,
            flip_back_legs: true,
            flip_leg_side: false,
            emit_walk_sound: true,
            emit_walk_effect: true,
            mech_land_shake: 0.0,
            mech_side_sway: 0.54,
            mech_front_sway: 0.1,
            mech_stride: -1.0,
            mech_step_particles: false,
            mech_leg_color_rgba: 0x3d4653ff,
            tread_frames: 18,
            tread_pull_offset: 0,
            crush_fragile: false,
            segments: 0,
            segment_units: 1,
            segment_unit: None,
            segment_end_unit: None,
            segment_layer_order: true,
            segment_mag: 2.0,
            segment_scl: 4.0,
            segment_phase: 5.0,
            segment_rot_speed: 1.0,
            segment_max_rot: 30.0,
            segment_spacing: -1.0,
            segment_rotation_range: 80.0,
            crawl_slowdown: 0.5,
            crush_damage: 0.0,
            crawl_slowdown_frac: 0.55,
            lifetime: 60.0 * 5.0,
            homing_delay: 10.0,
            build_time: -1.0,
        }
    }

    pub fn name(&self) -> &str {
        &self.base.mappable.name
    }

    pub fn localized_name(&self) -> &str {
        self.base
            .localized_name
            .as_deref()
            .unwrap_or_else(|| self.name())
    }

    pub fn has_weapons(&self) -> bool {
        !self.weapons.is_empty()
    }

    pub fn targetable_with_payload(&self, has_payload: bool) -> bool {
        self.targetable || (self.vulnerable_with_payloads && has_payload)
    }

    pub fn hittable_with_payload(&self, has_payload: bool) -> bool {
        self.hittable || (self.vulnerable_with_payloads && has_payload)
    }

    pub fn killable_with_payload(&self, _has_payload: bool) -> bool {
        self.killable
    }

    pub fn supports_env(&self, env: u32) -> bool {
        (self.env_enabled & env) != 0
            && (self.env_disabled & env) == 0
            && (self.env_required == 0 || (self.env_required & env) == self.env_required)
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn show_unlock(&self) -> bool {
        false
    }

    pub fn default_database_tag(&self) -> &'static str {
        if self.flying {
            "unit-air"
        } else if self.naval {
            "unit-naval"
        } else {
            "unit-ground"
        }
    }

    pub fn resolved_flying_layer(&self) -> f32 {
        if self.flying_layer >= 0.0 {
            self.flying_layer
        } else if self.low_altitude {
            LAYER_FLYING_UNIT_LOW
        } else {
            LAYER_FLYING_UNIT
        }
    }

    pub fn resolved_light_radius(&self) -> f32 {
        if self.light_radius >= 0.0 {
            self.light_radius
        } else {
            60.0_f32.max(self.hit_size * 2.3)
        }
    }

    pub fn resolved_fog_radius(&self) -> f32 {
        if self.fog_radius >= 0.0 {
            self.fog_radius
        } else {
            (58.0_f32 * 3.0).max(self.hit_size * 2.0) / TILE_SIZE
        }
    }

    pub fn resolved_item_capacity(&self) -> i32 {
        if self.item_capacity >= 0 {
            self.item_capacity
        } else {
            round_to(((self.hit_size * 4.0) as i32) as f32, 10.0).max(10.0) as i32
        }
    }

    pub fn resolved_mine_beam_offset(&self) -> f32 {
        if self.mine_beam_offset == f32::NEG_INFINITY {
            self.hit_size / 2.0
        } else {
            self.mine_beam_offset
        }
    }

    pub fn resolved_segment_spacing(&self) -> f32 {
        if self.segment_spacing < 0.0 {
            self.hit_size
        } else {
            self.segment_spacing
        }
    }

    pub fn resolved_aim_dst(&self) -> f32 {
        if self.aim_dst >= 0.0 {
            return self.aim_dst;
        }

        if self.weapons.iter().any(|weapon| !weapon.rotate) {
            self.hit_size * 2.0
        } else {
            self.hit_size / 2.0
        }
    }

    pub fn attack_range_plan(&self, margin: f32) -> UnitAttackRangePlan {
        let mut range = self.range;
        let mut max_range = self.max_range;

        if range < 0.0 {
            range = f32::MAX;
            for weapon in self.weapons.iter().filter(|weapon| weapon.use_attack_range) {
                let weapon_range = weapon.range() - margin;
                range = range.min(weapon_range);
                max_range = max_range.max(weapon_range);
            }
        }

        if max_range < 0.0 {
            max_range = 0.0_f32.max(range);
            for weapon in self.weapons.iter().filter(|weapon| weapon.use_attack_range) {
                max_range = max_range.max(weapon.range() - margin);
            }
        }

        if !self.weapons.iter().any(|weapon| weapon.use_attack_range) {
            if range < 0.0 || range == f32::MAX {
                range = self.mine_range;
            }
            if max_range < 0.0 || max_range == f32::MAX {
                max_range = self.mine_range;
            }
        }

        UnitAttackRangePlan { range, max_range }
    }

    pub fn mirrored_weapons_plan(&self) -> Vec<Weapon> {
        let mut mapped = Vec::new();
        for weapon in &self.weapons {
            let mut weapon = weapon.clone();
            if weapon.recoil_time < 0.0 {
                weapon.recoil_time = weapon.reload;
            }

            if weapon.mirror {
                let mut copy = weapon.copy();
                copy.flip();

                weapon.recoil_time *= 2.0;
                copy.recoil_time *= 2.0;
                weapon.reload *= 2.0;
                copy.reload *= 2.0;

                weapon.other_side = mapped.len() as i32 + 1;
                copy.other_side = mapped.len() as i32;
                mapped.push(weapon);
                mapped.push(copy);
            } else {
                mapped.push(weapon);
            }
        }
        mapped
    }

    pub fn resolved_ammo_capacity(&self) -> i32 {
        if self.ammo_capacity >= 0 {
            return self.ammo_capacity;
        }

        let shots_per_second: f32 = self
            .weapons
            .iter()
            .filter(|weapon| weapon.use_ammo && weapon.reload > 0.0)
            .map(|weapon| 60.0 / weapon.reload)
            .sum();
        ((shots_per_second * 35.0) as i32).max(1)
    }

    pub fn estimate_dps_with(&mut self, weapon_dps: impl Fn(&Weapon) -> f32) -> f32 {
        if self.dps_estimate < 0.0 {
            self.dps_estimate = self.weapons.iter().map(weapon_dps).sum();
        }
        self.dps_estimate
    }

    pub fn pure_init_plan(&self) -> UnitTypePureInitPlan {
        let light_radius = self.resolved_light_radius();
        let weapons = self.mirrored_weapons_plan();
        let has_attack_range_weapon = weapons.iter().any(|weapon| weapon.use_attack_range);
        let mut range_unit = self.clone();
        range_unit.weapons = weapons.clone();
        let attack_range = range_unit.attack_range_plan(4.0);

        UnitTypePureInitPlan {
            env_enabled: if self.flying {
                self.env_enabled | Env::SPACE
            } else {
                self.env_enabled
            },
            death_sound: if self.death_sound == "unset" {
                if self.hit_size < 12.0 {
                    "unitExplode1"
                } else if self.hit_size < 22.0 {
                    "unitExplode2"
                } else {
                    "unitExplode3"
                }
                .to_string()
            } else {
                self.death_sound.clone()
            },
            wreck_sound: if self.wreck_sound == "unset" {
                if self.hit_size >= 22.0 {
                    "wreckFallBig"
                } else {
                    "wreckFall"
                }
                .to_string()
            } else {
                self.wreck_sound.clone()
            },
            light_radius,
            flying_layer: self.resolved_flying_layer(),
            clip_size: self.clip_size.max(light_radius * 1.1),
            single_target: self.single_target || (weapons.len() <= 1 && !self.force_multi_target),
            item_capacity: self.resolved_item_capacity(),
            range: attack_range.range,
            max_range: attack_range.max_range,
            fog_radius: self.resolved_fog_radius(),
            mine_beam_offset: self.resolved_mine_beam_offset(),
            segment_spacing: self.resolved_segment_spacing(),
            aim_dst: if self.aim_dst >= 0.0 {
                self.aim_dst
            } else if weapons.iter().any(|weapon| !weapon.rotate) {
                self.hit_size * 2.0
            } else {
                self.hit_size / 2.0
            },
            mech_stride: if self.mech_stride < 0.0 {
                4.0 + (self.hit_size - 8.0) / 2.1
            } else {
                self.mech_stride
            },
            step_shake: if self.step_shake < 0.0 {
                (self.hit_size - 11.0) / 9.0
            } else {
                self.step_shake
            }
            .round(),
            mech_step_particles: self.mech_step_particles
                || (self.step_shake < 0.0 && self.hit_size > 15.0),
            engines: if self.engine_size > 0.0 && self.engines.is_empty() {
                vec![UnitEngine::new(
                    0.0,
                    -self.engine_offset,
                    self.engine_size,
                    -90.0,
                )]
            } else {
                self.engines.clone()
            },
            weapons,
            can_attack: self.weapons.iter().any(|weapon| !weapon.no_attack),
            ammo_capacity: self.resolved_ammo_capacity(),
            has_attack_range_weapon,
        }
    }

    pub fn sense(&self, sensor: LAccess, payload_capable: bool, logic_id: i32) -> f64 {
        match sensor {
            LAccess::Health | LAccess::MaxHealth => self.health as f64,
            LAccess::Size => (self.hit_size / TILE_SIZE) as f64,
            LAccess::ItemCapacity => self.resolved_item_capacity() as f64,
            LAccess::Speed => (self.speed * 60.0 / TILE_SIZE) as f64,
            LAccess::PayloadCapacity => {
                if payload_capable {
                    (self.payload_capacity / TILE_PAYLOAD) as f64
                } else {
                    0.0
                }
            }
            LAccess::Id => logic_id as f64,
            _ => f64::NAN,
        }
    }

    pub fn sense_object(&self, sensor: LAccess) -> Option<&str> {
        (sensor == LAccess::Name).then(|| self.name())
    }
}

impl Content for UnitType {
    fn id(&self) -> ContentId {
        self.base.mappable.base.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::Unit
    }
}

impl std::fmt::Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.localized_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitEngine {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitAttackRangePlan {
    pub range: f32,
    pub max_range: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitTypePureInitPlan {
    pub env_enabled: u32,
    pub death_sound: String,
    pub wreck_sound: String,
    pub light_radius: f32,
    pub flying_layer: f32,
    pub clip_size: f32,
    pub single_target: bool,
    pub item_capacity: i32,
    pub range: f32,
    pub max_range: f32,
    pub fog_radius: f32,
    pub mine_beam_offset: f32,
    pub segment_spacing: f32,
    pub aim_dst: f32,
    pub mech_stride: f32,
    pub step_shake: f32,
    pub mech_step_particles: bool,
    pub engines: Vec<UnitEngine>,
    pub weapons: Vec<Weapon>,
    pub can_attack: bool,
    pub ammo_capacity: i32,
    pub has_attack_range_weapon: bool,
}

impl UnitEngine {
    pub const fn new(x: f32, y: f32, radius: f32, rotation: f32) -> Self {
        Self {
            x,
            y,
            radius,
            rotation,
        }
    }
}

fn round_to(value: f32, increment: f32) -> f32 {
    (value / increment).round() * increment
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ctype::Content;

    #[test]
    fn unit_type_defaults_match_java_field_initializers() {
        let unit = UnitType::new(7, "alpha");

        assert_eq!(unit.id(), 7);
        assert_eq!(unit.content_type(), ContentType::Unit);
        assert_eq!(unit.name(), "alpha");
        assert_eq!(unit.to_string(), "alpha");
        assert_eq!(unit.base.selection_size, 30.0);
        assert_eq!(UNIT_SHADOW_TX, -12.0);
        assert_eq!(UNIT_SHADOW_TY, -13.0);
        assert_eq!(unit.env_required, Env::NONE);
        assert_eq!(unit.env_enabled, Env::TERRESTRIAL);
        assert_eq!(unit.env_disabled, Env::SCORCHING);

        assert_eq!(unit.speed, 1.1);
        assert_eq!(unit.boost_multiplier, 1.0);
        assert_eq!(unit.rotate_speed, 5.0);
        assert_eq!(unit.drag, 0.3);
        assert_eq!(unit.accel, 0.5);
        assert_eq!(unit.hit_size, 6.0);
        assert_eq!(unit.health, 200.0);
        assert_eq!(unit.armor, 0.0);
        assert_eq!(unit.range, -1.0);
        assert_eq!(unit.max_range, -1.0);
        assert_eq!(unit.mine_range, 70.0);
        assert_eq!(unit.build_range, BUILDING_RANGE);
        assert_eq!(unit.ground_layer, LAYER_GROUND_UNIT);
        assert_eq!(unit.flying_layer, -1.0);
        assert_eq!(unit.payload_capacity, 8.0);
        assert_eq!(unit.build_speed, -1.0);
        assert_eq!(unit.mine_beam_offset, f32::NEG_INFINITY);
        assert_eq!(unit.light_radius, -1.0);
        assert_eq!(unit.wave_trail_x, 4.0);
        assert_eq!(unit.wave_trail_y, -3.0);
    }

    #[test]
    fn unit_type_boolean_defaults_match_java_field_initializers() {
        let unit = UnitType::new(0, "dagger");

        assert!(unit.is_enemy);
        assert!(!unit.flying);
        assert!(unit.wobble);
        assert!(unit.target_air);
        assert!(unit.target_ground);
        assert!(unit.face_target);
        assert!(!unit.circle_target);
        assert!(!unit.auto_drop_bombs);
        assert!(unit.target_buildings_mobile);
        assert!(!unit.can_boost);
        assert!(unit.boost_when_building);
        assert!(unit.boost_when_mining);
        assert!(unit.logic_controllable);
        assert!(unit.player_controllable);
        assert!(unit.control_select_global);
        assert!(unit.allowed_in_payloads);
        assert!(unit.hittable);
        assert!(unit.killable);
        assert!(unit.targetable);
        assert!(!unit.vulnerable_with_payloads);
        assert!(unit.pickup_units);
        assert!(unit.physics);
        assert!(unit.can_drown);
        assert!(unit.use_unit_cap);
        assert!(!unit.core_unit_dock);
        assert!(unit.create_wreck);
        assert!(unit.create_scorch);
        assert!(!unit.low_altitude);
        assert!(unit.rotate_to_building);
        assert!(!unit.allow_leg_step);
        assert!(unit.leg_physics_layer);
        assert!(!unit.hovering);
        assert!(unit.omni_movement);
        assert!(!unit.rotate_move_first);
        assert!(unit.heal_flash);
        assert!(!unit.can_heal);
        assert!(!unit.single_target);
        assert!(!unit.force_multi_target);
        assert!(unit.can_attack);
        assert!(!unit.hidden);
        assert!(unit.bounded);
        assert!(unit.auto_find_target);
        assert!(unit.target_under_blocks);
        assert!(unit.hoverable);
        assert!(unit.generate_full_icon);
        assert!(unit.draw_build_beam);
        assert!(unit.draw_mine_beam);
        assert!(unit.draw_cell);
        assert!(unit.draw_items);
        assert!(unit.draw_shields);
        assert!(unit.draw_body);
        assert!(unit.draw_soft_shadow);
        assert!(unit.draw_minimap);
    }

    #[test]
    fn unit_type_late_section_defaults_match_java_field_initializers() {
        let unit = UnitType::new(0, "crawler");

        assert!(unit.abilities.is_empty());
        assert!(unit.weapons.is_empty());
        assert!(unit.immunities.is_empty());
        assert_eq!(unit.death_sound, "unset");
        assert_eq!(unit.death_sound_volume, 1.0);
        assert_eq!(unit.wreck_sound, "unset");
        assert_eq!(unit.loop_sound, "none");
        assert_eq!(unit.loop_sound_volume, 0.5);
        assert_eq!(unit.step_sound, "mechStepSmall");
        assert_eq!(unit.step_sound_volume, 0.5);
        assert_eq!(unit.step_sound_pitch, 1.0);
        assert_eq!(unit.step_sound_pitch_range, 0.1);
        assert_eq!(unit.tank_move_sound, "tankMove");
        assert_eq!(unit.move_sound, "none");
        assert_eq!(unit.mine_sound, "loopMineBeam");
        assert_eq!(
            unit.mine_items,
            vec!["copper", "lead", "titanium", "thorium"]
        );
        assert!(unit.use_engine_elevation);
        assert_eq!(unit.trail_length, 0);
        assert_eq!(unit.flowfield_path_type, -1);
        assert_eq!(unit.target_flags, vec![None]);
        assert!(unit.allow_change_commands);
        assert_eq!(unit.outline_radius, 3);
        assert!(unit.outlines);
        assert_eq!(unit.item_capacity, -1);
        assert_eq!(unit.ammo_capacity, -1);
        assert_eq!(unit.ammo_type, "item:copper");
        assert_eq!(unit.mine_tier, -1);
        assert_eq!(unit.mine_speed, 1.0);
        assert!(!unit.mine_walls);
        assert!(unit.mine_floor);
        assert!(unit.mine_hardness_scaling);
        assert_eq!(unit.mine_sound_volume, 0.6);
    }

    #[test]
    fn unit_type_leg_mech_tank_segment_and_missile_defaults_match_java() {
        let unit = UnitType::new(0, "toxopid");

        assert_eq!(unit.leg_count, 4);
        assert_eq!(unit.leg_group_size, 2);
        assert_eq!(unit.leg_length, 10.0);
        assert_eq!(unit.leg_speed, 0.1);
        assert_eq!(unit.leg_forward_scl, 1.0);
        assert_eq!(unit.leg_base_offset, 0.0);
        assert_eq!(unit.leg_move_space, 1.0);
        assert_eq!(unit.leg_extension, 0.0);
        assert_eq!(unit.leg_pair_offset, 0.0);
        assert_eq!(unit.leg_length_scl, 1.0);
        assert_eq!(unit.leg_straight_length, 1.0);
        assert_eq!(unit.leg_max_length, 1.75);
        assert_eq!(unit.leg_min_length, 0.0);
        assert_eq!(unit.leg_splash_damage, 0.0);
        assert_eq!(unit.leg_splash_range, 5.0);
        assert_eq!(unit.leg_region, TextureRegionRef::new("toxopid-leg"));
        assert_eq!(
            unit.leg_base_region,
            TextureRegionRef::new("toxopid-leg-base")
        );
        assert!(!unit.leg_base_under);
        assert!(!unit.lock_leg_base);
        assert!(!unit.leg_continuous_move);
        assert!(unit.flip_back_legs);
        assert!(!unit.flip_leg_side);
        assert!(unit.emit_walk_sound);
        assert!(unit.emit_walk_effect);

        assert_eq!(unit.mech_land_shake, 0.0);
        assert_eq!(unit.mech_side_sway, 0.54);
        assert_eq!(unit.mech_front_sway, 0.1);
        assert_eq!(unit.mech_stride, -1.0);
        assert!(!unit.mech_step_particles);
        assert_eq!(unit.tread_frames, 18);
        assert_eq!(unit.tread_pull_offset, 0);
        assert!(!unit.crush_fragile);
        assert_eq!(unit.segments, 0);
        assert_eq!(unit.segment_units, 1);
        assert_eq!(unit.segment_layer_order, true);
        assert_eq!(unit.segment_mag, 2.0);
        assert_eq!(unit.segment_scl, 4.0);
        assert_eq!(unit.segment_phase, 5.0);
        assert_eq!(unit.segment_rot_speed, 1.0);
        assert_eq!(unit.segment_max_rot, 30.0);
        assert_eq!(unit.segment_spacing, -1.0);
        assert_eq!(unit.segment_rotation_range, 80.0);
        assert_eq!(unit.crawl_slowdown, 0.5);
        assert_eq!(unit.crush_damage, 0.0);
        assert_eq!(unit.crawl_slowdown_frac, 0.55);
        assert_eq!(unit.lifetime, 300.0);
        assert_eq!(unit.homing_delay, 10.0);
        assert_eq!(unit.build_time, -1.0);
    }

    #[test]
    fn unit_type_predicates_and_env_support_follow_java_helpers() {
        let mut unit = UnitType::new(0, "alpha");
        assert!(unit.supports_env(Env::TERRESTRIAL));
        assert!(!unit.supports_env(Env::SPACE));
        assert!(!unit.supports_env(Env::SCORCHING));

        unit.env_enabled |= Env::SPACE;
        assert!(unit.supports_env(Env::SPACE));
        unit.env_required = Env::SPACE | Env::OXYGEN;
        assert!(!unit.supports_env(Env::SPACE));
        assert!(unit.supports_env(Env::SPACE | Env::OXYGEN));

        assert!(!unit.has_weapons());
        unit.weapons.push(Weapon::new("alpha-weapon"));
        assert!(unit.has_weapons());

        unit.targetable = false;
        unit.hittable = false;
        assert!(!unit.targetable_with_payload(false));
        assert!(!unit.hittable_with_payload(false));
        unit.vulnerable_with_payloads = true;
        assert!(unit.targetable_with_payload(true));
        assert!(unit.hittable_with_payload(true));
        unit.killable = false;
        assert!(!unit.killable_with_payload(true));
    }

    #[test]
    fn unit_type_resolved_helpers_match_java_init_fallbacks() {
        let mut unit = UnitType::new(0, "flare");
        assert_eq!(unit.default_database_tag(), "unit-ground");
        unit.flying = true;
        assert_eq!(unit.default_database_tag(), "unit-air");
        unit.flying = false;
        unit.naval = true;
        assert_eq!(unit.default_database_tag(), "unit-naval");

        unit.low_altitude = false;
        assert_eq!(unit.resolved_flying_layer(), LAYER_FLYING_UNIT);
        unit.low_altitude = true;
        assert_eq!(unit.resolved_flying_layer(), LAYER_FLYING_UNIT_LOW);
        unit.flying_layer = 123.0;
        assert_eq!(unit.resolved_flying_layer(), 123.0);

        unit.hit_size = 20.0;
        assert_eq!(unit.resolved_light_radius(), 60.0);
        assert_eq!(unit.resolved_fog_radius(), 21.75);
        assert_eq!(unit.resolved_mine_beam_offset(), 10.0);
        assert_eq!(unit.resolved_segment_spacing(), 20.0);
        assert_eq!(unit.resolved_item_capacity(), 80);

        unit.weapons.clear();
        assert_eq!(unit.resolved_aim_dst(), 10.0);
        let mut rotating = Weapon::new("rotating");
        rotating.rotate = true;
        unit.weapons.push(rotating);
        assert_eq!(unit.resolved_aim_dst(), 10.0);
        let mut fixed = Weapon::new("fixed");
        fixed.rotate = false;
        unit.weapons.push(fixed);
        assert_eq!(unit.resolved_aim_dst(), 40.0);
    }

    #[test]
    fn unit_type_logic_sense_matches_java_unit_type_contract_subset() {
        let mut unit = UnitType::new(42, "gamma");
        unit.health = 1500.0;
        unit.hit_size = 24.0;
        unit.item_capacity = 120;
        unit.speed = 0.8;
        unit.payload_capacity = 128.0;

        assert_eq!(unit.sense(LAccess::Health, false, 99), 1500.0);
        assert_eq!(unit.sense(LAccess::MaxHealth, false, 99), 1500.0);
        assert_eq!(unit.sense(LAccess::Size, false, 99), 3.0);
        assert_eq!(unit.sense(LAccess::ItemCapacity, false, 99), 120.0);
        assert_eq!(unit.sense(LAccess::Speed, false, 99), 6.0);
        assert_eq!(unit.sense(LAccess::PayloadCapacity, false, 99), 0.0);
        assert_eq!(unit.sense(LAccess::PayloadCapacity, true, 99), 2.0);
        assert_eq!(unit.sense(LAccess::Id, false, 99), 99.0);
        assert!(unit.sense(LAccess::Ammo, false, 99).is_nan());
        assert_eq!(unit.sense_object(LAccess::Name), Some("gamma"));
        assert_eq!(unit.sense_object(LAccess::Health), None);
    }

    #[test]
    fn unit_type_attack_range_plan_matches_java_range_fallbacks() {
        let mut unit = UnitType::new(0, "horizon");
        unit.mine_range = 70.0;
        assert_eq!(
            unit.attack_range_plan(4.0),
            UnitAttackRangePlan {
                range: 70.0,
                max_range: 70.0,
            }
        );

        let mut short = Weapon::new("short");
        short.bullet_range = 120.0;
        short.shoot_cone = 12.0;
        let mut long = Weapon::new("long");
        long.bullet_range = 200.0;
        long.shoot_cone = 8.0;
        unit.weapons = vec![short, long];
        assert_eq!(
            unit.attack_range_plan(4.0),
            UnitAttackRangePlan {
                range: 116.0,
                max_range: 196.0,
            }
        );

        unit.range = 90.0;
        unit.max_range = -1.0;
        assert_eq!(
            unit.attack_range_plan(4.0),
            UnitAttackRangePlan {
                range: 90.0,
                max_range: 196.0,
            }
        );

        unit.weapons[0].use_attack_range = false;
        unit.weapons[1].use_attack_range = false;
        unit.range = -1.0;
        unit.max_range = -1.0;
        assert_eq!(
            unit.attack_range_plan(4.0),
            UnitAttackRangePlan {
                range: 70.0,
                max_range: 70.0,
            }
        );
    }

    #[test]
    fn unit_type_mirrored_weapons_plan_matches_java_copy_flip_and_reload_rules() {
        let mut unit = UnitType::new(0, "duo");
        let mut mirrored = Weapon::new("cannon");
        mirrored.x = 6.0;
        mirrored.shoot_x = 1.0;
        mirrored.base_rotation = 15.0;
        mirrored.reload = 20.0;
        mirrored.recoil_time = -1.0;

        let mut single = Weapon::new("center");
        single.mirror = false;
        single.reload = 10.0;
        single.recoil_time = -1.0;
        unit.weapons = vec![mirrored, single];

        let planned = unit.mirrored_weapons_plan();
        assert_eq!(planned.len(), 3);
        assert_eq!(planned[0].x, 6.0);
        assert_eq!(planned[0].shoot_x, 1.0);
        assert_eq!(planned[0].base_rotation, 15.0);
        assert_eq!(planned[0].reload, 40.0);
        assert_eq!(planned[0].recoil_time, 40.0);
        assert_eq!(planned[0].other_side, 1);

        assert_eq!(planned[1].x, -6.0);
        assert_eq!(planned[1].shoot_x, -1.0);
        assert_eq!(planned[1].base_rotation, -15.0);
        assert!(planned[1].flip_sprite);
        assert_eq!(planned[1].reload, 40.0);
        assert_eq!(planned[1].recoil_time, 40.0);
        assert_eq!(planned[1].other_side, 0);

        assert_eq!(planned[2].name, "center");
        assert_eq!(planned[2].reload, 10.0);
        assert_eq!(planned[2].recoil_time, 10.0);
        assert_eq!(planned[2].other_side, -1);
    }

    #[test]
    fn unit_type_ammo_capacity_and_dps_cache_follow_java_formulas() {
        let mut unit = UnitType::new(0, "dagger");
        assert_eq!(unit.resolved_ammo_capacity(), 1);

        let mut a = Weapon::new("a");
        a.reload = 30.0;
        let mut b = Weapon::new("b");
        b.reload = 60.0;
        let mut ignored = Weapon::new("ignored");
        ignored.use_ammo = false;
        ignored.reload = 1.0;
        unit.weapons = vec![a, b, ignored];
        assert_eq!(unit.resolved_ammo_capacity(), 105);

        assert_eq!(unit.estimate_dps_with(|weapon| weapon.reload), 91.0);
        unit.weapons[0].reload = 999.0;
        assert_eq!(unit.estimate_dps_with(|weapon| weapon.reload), 91.0);
    }

    #[test]
    fn unit_type_pure_init_plan_matches_java_low_coupling_init_steps() {
        let mut unit = UnitType::new(0, "flare");
        unit.flying = true;
        unit.hit_size = 24.0;
        unit.low_altitude = true;
        unit.force_multi_target = false;
        unit.engine_size = 3.0;
        unit.engine_offset = 7.0;
        let mut weapon = Weapon::new("flare-gun");
        weapon.rotate = false;
        weapon.reload = 20.0;
        weapon.bullet_range = 160.0;
        weapon.shoot_cone = 12.0;
        unit.weapons.push(weapon);

        let plan = unit.pure_init_plan();
        assert_eq!(plan.env_enabled, Env::TERRESTRIAL | Env::SPACE);
        assert_eq!(plan.death_sound, "unitExplode3");
        assert_eq!(plan.wreck_sound, "wreckFallBig");
        assert_eq!(plan.light_radius, 60.0);
        assert_eq!(plan.flying_layer, LAYER_FLYING_UNIT_LOW);
        assert_eq!(plan.clip_size, 66.0);
        assert!(!plan.single_target);
        assert_eq!(plan.item_capacity, 100);
        assert_eq!(plan.range, 156.0);
        assert_eq!(plan.max_range, 156.0);
        assert_eq!(plan.fog_radius, 21.75);
        assert_eq!(plan.mine_beam_offset, 12.0);
        assert_eq!(plan.segment_spacing, 24.0);
        assert_eq!(plan.aim_dst, 48.0);
        assert!((plan.mech_stride - 11.619_048).abs() < 0.0001);
        assert_eq!(plan.step_shake, 1.0);
        assert!(plan.mech_step_particles);
        assert_eq!(plan.engines, vec![UnitEngine::new(0.0, -7.0, 3.0, -90.0)]);
        assert_eq!(plan.weapons.len(), 2);
        assert!(plan.can_attack);
        assert_eq!(plan.ammo_capacity, 105);
        assert!(plan.has_attack_range_weapon);
    }

    #[test]
    fn unit_engine_is_a_lightweight_data_shell() {
        assert_eq!(
            UnitEngine::new(0.0, -5.0, 2.5, -90.0),
            UnitEngine {
                x: 0.0,
                y: -5.0,
                radius: 2.5,
                rotation: -90.0,
            }
        );
    }
}
