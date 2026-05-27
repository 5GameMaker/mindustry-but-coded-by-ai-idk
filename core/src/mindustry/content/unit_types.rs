use crate::mindustry::{
    ctype::{Content, ContentId, ContentType},
    r#type::{
        unit::{
            erekir_unit_type::apply_erekir_unit_type_defaults,
            missile_unit_type::missile_unit_type,
            neoplasm_unit_type::apply_neoplasm_unit_type_defaults,
            tank_unit_type::apply_tank_unit_type_defaults,
        },
        UnitType,
    },
    world::meta::Env,
};

const LAYER_LEG_UNIT: f32 = 75.0;

#[derive(Debug, Clone, Copy)]
enum UnitKind {
    Standard,
    Naval,
    Erekir,
    Tank,
    Neoplasm,
}

pub fn load() -> Vec<UnitType> {
    let mut next_id = 0;

    let mut units = vec![
        unit(&mut next_id, "dagger", UnitKind::Standard, |u| {
            u.research_cost_multiplier = 0.5;
            u.speed = 0.5;
            u.hit_size = 8.0;
            u.health = 150.0;
            u.step_sound_volume = 0.4;
        }),
        unit(&mut next_id, "mace", UnitKind::Standard, |u| {
            u.speed = 0.5;
            u.hit_size = 10.0;
            u.health = 550.0;
            u.armor = 4.0;
            u.ammo_type = "item:coal".into();
            u.immunities.push("burning".into());
        }),
        unit(&mut next_id, "fortress", UnitKind::Standard, |u| {
            u.speed = 0.43;
            u.hit_size = 13.0;
            u.rotate_speed = 3.0;
            u.target_air = false;
            u.health = 900.0;
            u.armor = 9.0;
            u.mech_front_sway = 0.55;
            u.ammo_type = "item:graphite".into();
            u.step_sound = "mechStepSmall".into();
            u.step_sound_pitch = 0.8;
            u.step_sound_volume = 0.65;
        }),
        unit(&mut next_id, "scepter", UnitKind::Standard, |u| {
            u.speed = 0.36;
            u.hit_size = 22.0;
            u.rotate_speed = 2.1;
            u.health = 9000.0;
            u.armor = 10.0;
            u.mech_front_sway = 1.0;
            u.ammo_type = "item:thorium".into();
            u.mech_step_particles = true;
            u.step_shake = 0.15;
            u.single_target = true;
            u.drown_time_multiplier = 1.5;
            u.step_sound = "mechStep".into();
            u.step_sound_pitch = 0.9;
            u.step_sound_volume = 0.35;
            u.abilities.push("ShieldRegenFieldAbility".into());
        }),
        unit(&mut next_id, "reign", UnitKind::Standard, |u| {
            u.speed = 0.4;
            u.hit_size = 30.0;
            u.rotate_speed = 1.65;
            u.health = 24000.0;
            u.armor = 18.0;
            u.mech_step_particles = true;
            u.step_shake = 0.75;
            u.drown_time_multiplier = 1.6;
            u.mech_front_sway = 1.9;
            u.mech_side_sway = 0.6;
            u.ammo_type = "item:thorium".into();
            u.step_sound = "mechStepHeavy".into();
            u.step_sound_pitch = 0.9;
            u.step_sound_volume = 0.45;
        }),
        unit(&mut next_id, "nova", UnitKind::Standard, |u| {
            u.can_boost = true;
            u.boost_multiplier = 1.5;
            u.speed = 0.55;
            u.hit_size = 8.0;
            u.health = 120.0;
            u.build_speed = 0.3;
            u.armor = 1.0;
            u.ammo_type = "power:1000".into();
            u.abilities.push("RepairFieldAbility".into());
        }),
        unit(&mut next_id, "pulsar", UnitKind::Standard, |u| {
            u.can_boost = true;
            u.boost_multiplier = 1.6;
            u.speed = 0.7;
            u.hit_size = 11.0;
            u.health = 320.0;
            u.build_speed = 0.5;
            u.armor = 4.0;
            u.rise_speed = 0.07;
            u.descent_speed = 0.07;
            u.mine_tier = 2;
            u.mine_speed = 3.0;
            u.ammo_type = "power:1300".into();
            u.abilities.push("ShieldRegenFieldAbility".into());
        }),
        unit(&mut next_id, "quasar", UnitKind::Standard, |u| {
            u.mine_tier = 3;
            u.boost_multiplier = 2.0;
            u.health = 640.0;
            u.build_speed = 1.1;
            u.can_boost = true;
            u.armor = 9.0;
            u.mech_land_shake = 2.0;
            u.rise_speed = 0.05;
            u.descent_speed = 0.05;
            u.mech_front_sway = 0.55;
            u.ammo_type = "power:1500".into();
            u.step_sound = "mechStepSmall".into();
            u.step_sound_pitch = 0.9;
            u.step_sound_volume = 0.6;
            u.speed = 0.5;
            u.hit_size = 13.0;
            u.mine_speed = 4.0;
            u.draw_shields = false;
            u.abilities.push("ForceFieldAbility".into());
        }),
        unit(&mut next_id, "vela", UnitKind::Standard, |u| {
            u.hit_size = 24.0;
            u.rotate_speed = 1.8;
            u.mech_front_sway = 1.0;
            u.build_speed = 3.0;
            u.mech_step_particles = true;
            u.step_shake = 0.15;
            u.ammo_type = "power:2500".into();
            u.drown_time_multiplier = 1.3;
            u.speed = 0.44;
            u.boost_multiplier = 2.4;
            u.engine_offset = 12.0;
            u.engine_size = 6.0;
            u.low_altitude = true;
            u.rise_speed = 0.02;
            u.descent_speed = 0.02;
            u.health = 8200.0;
            u.armor = 9.0;
            u.can_boost = true;
            u.mech_land_shake = 4.0;
            u.immunities.push("burning".into());
            u.single_target = true;
            u.step_sound = "mechStep".into();
            u.step_sound_pitch = 0.9;
            u.step_sound_volume = 0.25;
        }),
        unit(&mut next_id, "corvus", UnitKind::Standard, |u| {
            u.hit_size = 29.0;
            u.health = 18000.0;
            u.armor = 9.0;
            u.step_shake = 1.5;
            u.rotate_speed = 1.5;
            u.drown_time_multiplier = 1.6;
            u.step_sound = "walkerStep".into();
            u.step_sound_volume = 1.1;
            u.step_sound_pitch = 0.9;
            u.leg_count = 4;
            u.leg_length = 14.0;
            u.leg_base_offset = 11.0;
            u.leg_move_space = 1.5;
            u.leg_forward_scl = 0.58;
            u.hovering = true;
            u.shadow_elevation = 0.2;
            u.ammo_type = "power:4000".into();
            u.ground_layer = LAYER_LEG_UNIT;
            u.speed = 0.3;
            u.draw_shields = false;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "crawler", UnitKind::Standard, |u| {
            u.research_cost_multiplier = 0.5;
            u.speed = 1.0;
            u.hit_size = 8.0;
            u.health = 150.0;
            u.step_sound_volume = 0.2;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "atrax", UnitKind::Standard, |u| {
            u.speed = 0.6;
            u.hit_size = 13.0;
            u.rotate_speed = 3.0;
            u.target_air = false;
            u.health = 600.0;
            u.step_sound_pitch = 1.0;
            u.step_sound_volume = 0.25;
            u.leg_count = 4;
            u.leg_length = 9.0;
            u.leg_forward_scl = 0.6;
            u.leg_move_space = 1.4;
            u.hovering = true;
            u.armor = 3.0;
            u.ground_layer = LAYER_LEG_UNIT - 1.0;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "spiroct", UnitKind::Standard, |u| {
            u.speed = 0.54;
            u.hit_size = 15.0;
            u.rotate_speed = 3.0;
            u.health = 1000.0;
            u.leg_count = 6;
            u.leg_length = 13.0;
            u.leg_forward_scl = 0.8;
            u.leg_move_space = 1.4;
            u.leg_base_offset = 2.0;
            u.hovering = true;
            u.armor = 5.0;
            u.ground_layer = LAYER_LEG_UNIT;
            u.step_sound_pitch = 0.7;
            u.step_sound_volume = 0.35;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "arkyid", UnitKind::Standard, |u| {
            u.speed = 0.62;
            u.hit_size = 23.0;
            u.health = 8000.0;
            u.armor = 6.0;
            u.rotate_speed = 2.7;
            u.leg_count = 6;
            u.leg_move_space = 1.0;
            u.leg_length = 30.0;
            u.leg_base_offset = 10.0;
            u.step_shake = 1.0;
            u.step_sound_volume = 0.85;
            u.step_sound_pitch = 1.1;
            u.hovering = true;
            u.ground_layer = LAYER_LEG_UNIT;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "toxopid", UnitKind::Standard, |u| {
            u.speed = 0.5;
            u.hit_size = 26.0;
            u.health = 22000.0;
            u.armor = 13.0;
            u.step_sound_volume = 1.1;
            u.rotate_speed = 1.9;
            u.leg_count = 8;
            u.leg_move_space = 0.8;
            u.leg_length = 75.0;
            u.leg_base_offset = 8.0;
            u.step_shake = 1.0;
            u.hovering = true;
            u.ground_layer = LAYER_LEG_UNIT;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "flare", UnitKind::Standard, |u| {
            u.research_cost_multiplier = 0.5;
            u.speed = 2.7;
            u.flying = true;
            u.health = 70.0;
            u.engine_offset = 5.75;
            u.hit_size = 9.0;
            u.item_capacity = 10;
            u.rotate_speed = 5.0;
        }),
        unit(&mut next_id, "horizon", UnitKind::Standard, |u| {
            u.health = 340.0;
            u.speed = 1.65;
            u.flying = true;
            u.hit_size = 11.0;
            u.target_air = false;
            u.engine_offset = 7.8;
            u.armor = 3.0;
            u.item_capacity = 0;
            u.rotate_speed = 4.5;
        }),
        unit(&mut next_id, "zenith", UnitKind::Standard, |u| {
            u.health = 700.0;
            u.speed = 1.7;
            u.flying = true;
            u.hit_size = 20.0;
            u.low_altitude = true;
            u.armor = 5.0;
            u.engine_offset = 12.0;
            u.engine_size = 3.0;
        }),
        unit(&mut next_id, "antumbra", UnitKind::Standard, |u| {
            u.speed = 0.8;
            u.rotate_speed = 1.9;
            u.flying = true;
            u.low_altitude = true;
            u.health = 7200.0;
            u.armor = 9.0;
            u.engine_offset = 21.0;
            u.engine_size = 5.3;
            u.hit_size = 46.0;
        }),
        unit(&mut next_id, "eclipse", UnitKind::Standard, |u| {
            u.speed = 0.54;
            u.rotate_speed = 1.0;
            u.flying = true;
            u.low_altitude = true;
            u.health = 22000.0;
            u.engine_offset = 38.0;
            u.engine_size = 7.3;
            u.hit_size = 58.0;
            u.armor = 13.0;
        }),
        unit(&mut next_id, "mono", UnitKind::Standard, |u| {
            u.flying = true;
            u.speed = 1.5;
            u.health = 100.0;
            u.engine_size = 1.8;
            u.engine_offset = 5.7;
            u.is_enemy = false;
            u.mine_tier = 1;
            u.mine_speed = 2.5;
            u.default_command = Some("mine".into());
        }),
        unit(&mut next_id, "poly", UnitKind::Standard, |u| {
            u.flying = true;
            u.speed = 2.6;
            u.rotate_speed = 15.0;
            u.health = 400.0;
            u.build_speed = 0.5;
            u.engine_offset = 6.5;
            u.hit_size = 9.0;
            u.low_altitude = true;
            u.mine_tier = 2;
            u.mine_speed = 3.5;
            u.default_command = Some("rebuild".into());
        }),
        unit(&mut next_id, "mega", UnitKind::Standard, |u| {
            u.mine_tier = 3;
            u.mine_speed = 4.0;
            u.health = 460.0;
            u.armor = 3.0;
            u.speed = 2.5;
            u.low_altitude = true;
            u.flying = true;
            u.engine_offset = 10.5;
            u.hit_size = 16.05;
            u.engine_size = 3.0;
            u.payload_capacity = 4.0 * super_tile_payload();
            u.build_speed = 2.6;
            u.is_enemy = false;
            u.default_command = Some("repair".into());
        }),
        unit(&mut next_id, "quad", UnitKind::Standard, |u| {
            u.armor = 8.0;
            u.health = 6000.0;
            u.speed = 1.2;
            u.rotate_speed = 2.0;
            u.low_altitude = false;
            u.flying = true;
            u.engine_offset = 13.0;
            u.engine_size = 7.0;
            u.hit_size = 36.0;
            u.payload_capacity = 9.0 * super_tile_payload();
            u.build_speed = 2.5;
            u.target_air = false;
        }),
        unit(&mut next_id, "oct", UnitKind::Standard, |u| {
            u.armor = 16.0;
            u.health = 24000.0;
            u.speed = 0.8;
            u.rotate_speed = 1.0;
            u.flying = true;
            u.engine_offset = 46.0;
            u.engine_size = 7.8;
            u.hit_size = 66.0;
            u.payload_capacity = 30.25 * super_tile_payload();
            u.build_speed = 4.0;
            u.low_altitude = true;
            u.ammo_capacity = 1;
        }),
        unit(&mut next_id, "risso", UnitKind::Naval, |u| {
            u.speed = 1.1;
            u.hit_size = 10.0;
            u.health = 280.0;
            u.armor = 2.0;
            u.rotate_speed = 3.3;
        }),
        unit(&mut next_id, "minke", UnitKind::Naval, |u| {
            u.health = 600.0;
            u.speed = 0.9;
            u.hit_size = 13.0;
            u.armor = 4.0;
            u.rotate_speed = 2.6;
        }),
        unit(&mut next_id, "bryde", UnitKind::Naval, |u| {
            u.health = 910.0;
            u.speed = 0.85;
            u.rotate_speed = 1.8;
            u.hit_size = 20.0;
            u.armor = 7.0;
        }),
        unit(&mut next_id, "sei", UnitKind::Naval, |u| {
            u.health = 11000.0;
            u.armor = 12.0;
            u.speed = 0.73;
            u.hit_size = 39.0;
            u.rotate_speed = 1.3;
        }),
        unit(&mut next_id, "omura", UnitKind::Naval, |u| {
            u.health = 22000.0;
            u.speed = 0.62;
            u.hit_size = 58.0;
            u.armor = 16.0;
            u.rotate_speed = 0.9;
        }),
        unit(&mut next_id, "retusa", UnitKind::Naval, |u| {
            u.speed = 0.9;
            u.hit_size = 11.0;
            u.health = 270.0;
            u.rotate_speed = 5.0;
            u.armor = 3.0;
            u.build_speed = 1.5;
        }),
        unit(&mut next_id, "oxynoe", UnitKind::Naval, |u| {
            u.health = 560.0;
            u.speed = 0.83;
            u.hit_size = 14.0;
            u.armor = 4.0;
            u.rotate_speed = 4.0;
            u.build_speed = 2.0;
        }),
        unit(&mut next_id, "cyerce", UnitKind::Naval, |u| {
            u.health = 870.0;
            u.speed = 0.86;
            u.rotate_speed = 2.6;
            u.hit_size = 20.0;
            u.armor = 6.0;
            u.build_speed = 2.0;
        }),
        unit(&mut next_id, "aegires", UnitKind::Naval, |u| {
            u.health = 12000.0;
            u.armor = 12.0;
            u.speed = 0.7;
            u.hit_size = 44.0;
            u.rotate_speed = 1.4;
            u.ammo_capacity = 40;
            u.build_speed = 3.0;
        }),
        unit(&mut next_id, "navanax", UnitKind::Naval, |u| {
            u.health = 20000.0;
            u.speed = 0.65;
            u.hit_size = 58.0;
            u.armor = 16.0;
            u.rotate_speed = 1.1;
            u.build_speed = 3.5;
        }),
        unit(&mut next_id, "alpha", UnitKind::Standard, |u| {
            u.is_enemy = false;
            u.low_altitude = true;
            u.flying = true;
            u.mine_speed = 6.5;
            u.mine_tier = 1;
            u.build_speed = 0.5;
            u.speed = 3.0;
            u.rotate_speed = 15.0;
            u.item_capacity = 30;
            u.health = 150.0;
            u.engine_offset = 6.0;
            u.hit_size = 8.0;
        }),
        unit(&mut next_id, "beta", UnitKind::Standard, |u| {
            u.is_enemy = false;
            u.flying = true;
            u.mine_speed = 7.0;
            u.mine_tier = 1;
            u.build_speed = 0.75;
            u.speed = 3.3;
            u.rotate_speed = 17.0;
            u.item_capacity = 50;
            u.health = 170.0;
            u.engine_offset = 6.0;
            u.hit_size = 9.0;
            u.low_altitude = true;
        }),
        unit(&mut next_id, "gamma", UnitKind::Standard, |u| {
            u.is_enemy = false;
            u.low_altitude = true;
            u.flying = true;
            u.mine_speed = 8.0;
            u.mine_tier = 2;
            u.build_speed = 1.0;
            u.speed = 3.55;
            u.rotate_speed = 19.0;
            u.item_capacity = 70;
            u.health = 220.0;
            u.engine_offset = 6.0;
            u.hit_size = 11.0;
        }),
        unit(&mut next_id, "stell", UnitKind::Tank, |u| {
            u.hit_size = 12.0;
            u.speed = 0.75;
            u.rotate_speed = 3.5;
            u.health = 850.0;
            u.armor = 6.0;
            u.item_capacity = 0;
            u.research_cost_multiplier = 0.0;
        }),
        unit(&mut next_id, "locus", UnitKind::Tank, |u| {
            u.hit_size = 18.0;
            u.speed = 0.7;
            u.rotate_speed = 2.6;
            u.health = 2100.0;
            u.armor = 8.0;
            u.item_capacity = 0;
            u.research_cost_multiplier = 0.0;
        }),
        unit(&mut next_id, "precept", UnitKind::Tank, |u| {
            u.hit_size = 24.0;
            u.speed = 0.64;
            u.rotate_speed = 1.5;
            u.health = 5000.0;
            u.armor = 11.0;
            u.item_capacity = 0;
            u.drown_time_multiplier = 1.2;
            u.research_cost_multiplier = 0.0;
        }),
        unit(&mut next_id, "vanquish", UnitKind::Tank, |u| {
            u.hit_size = 28.0;
            u.speed = 0.63;
            u.health = 11000.0;
            u.armor = 20.0;
            u.item_capacity = 0;
            u.drown_time_multiplier = 1.25;
        }),
        unit(&mut next_id, "conquer", UnitKind::Tank, |u| {
            u.hit_size = 46.0;
            u.speed = 0.48;
            u.health = 22000.0;
            u.armor = 26.0;
            u.rotate_speed = 0.8;
        }),
        unit(&mut next_id, "merui", UnitKind::Erekir, |u| {
            u.speed = 0.72;
            u.hit_size = 9.0;
            u.rotate_speed = 3.0;
            u.health = 680.0;
            u.armor = 4.0;
            u.step_shake = 0.0;
            u.step_sound_volume = 0.4;
            u.leg_count = 6;
            u.leg_length = 8.0;
            u.leg_base_offset = 3.0;
            u.leg_forward_scl = 1.1;
            u.leg_move_space = 1.0;
            u.allow_leg_step = true;
            u.hovering = true;
            u.ground_layer = LAYER_LEG_UNIT - 1.0;
            u.target_air = false;
            u.research_cost_multiplier = 0.0;
        }),
        unit(&mut next_id, "cleroi", UnitKind::Erekir, |u| {
            u.speed = 0.6;
            u.hit_size = 14.0;
            u.rotate_speed = 3.0;
            u.health = 1100.0;
            u.armor = 5.0;
            u.step_shake = 0.0;
            u.leg_count = 4;
            u.leg_length = 14.0;
            u.leg_base_offset = 5.0;
            u.leg_forward_scl = 0.7;
            u.leg_move_space = 1.0;
            u.hovering = true;
            u.ground_layer = LAYER_LEG_UNIT - 1.0;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "anthicus", UnitKind::Erekir, |u| {
            u.speed = 0.65;
            u.hit_size = 21.0;
            u.rotate_speed = 3.0;
            u.health = 2700.0;
            u.armor = 7.0;
            u.step_shake = 0.0;
            u.step_sound_pitch = 0.78;
            u.leg_count = 6;
            u.leg_length = 18.0;
            u.leg_base_offset = 7.0;
            u.leg_forward_scl = 0.9;
            u.leg_move_space = 1.0;
            u.hovering = true;
            u.ground_layer = LAYER_LEG_UNIT - 1.0;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "tecta", UnitKind::Erekir, |u| {
            u.speed = 0.6;
            u.hit_size = 30.0;
            u.health = 6500.0;
            u.armor = 5.0;
            u.research_cost_multiplier = 0.0;
            u.step_sound_volume = 1.0;
            u.step_sound_pitch = 1.0;
            u.rotate_speed = 2.1;
            u.leg_count = 6;
            u.leg_length = 15.0;
            u.leg_forward_scl = 0.45;
            u.leg_move_space = 1.4;
            u.step_shake = 0.5;
            u.leg_base_offset = 5.0;
            u.drown_time_multiplier = 0.5;
            u.hovering = true;
            u.ground_layer = LAYER_LEG_UNIT;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "collaris", UnitKind::Erekir, |u| {
            u.speed = 1.1;
            u.hit_size = 44.0;
            u.health = 18000.0;
            u.armor = 9.0;
            u.rotate_speed = 1.6;
            u.step_sound_volume = 1.1;
            u.step_sound_pitch = 0.9;
            u.leg_count = 8;
            u.leg_length = 30.0;
            u.leg_forward_scl = 2.1;
            u.leg_move_space = 1.05;
            u.step_shake = 0.5;
            u.leg_base_offset = 19.0;
            u.drown_time_multiplier = 0.5;
            u.hovering = true;
            u.ground_layer = LAYER_LEG_UNIT;
            u.target_air = false;
            u.allow_leg_step = true;
        }),
        unit(&mut next_id, "elude", UnitKind::Erekir, |u| {
            u.hovering = true;
            u.speed = 1.8;
            u.rotate_speed = 5.0;
            u.health = 600.0;
            u.armor = 1.0;
            u.hit_size = 11.0;
            u.engine_offset = 7.0;
            u.engine_size = 2.0;
            u.item_capacity = 0;
            u.research_cost_multiplier = 0.0;
        }),
        unit(&mut next_id, "avert", UnitKind::Erekir, |u| {
            u.low_altitude = false;
            u.flying = true;
            u.speed = 2.0;
            u.rotate_speed = 8.0;
            u.health = 1100.0;
            u.armor = 3.0;
            u.hit_size = 12.0;
            u.engine_size = 0.0;
            u.item_capacity = 0;
        }),
        unit(&mut next_id, "obviate", UnitKind::Erekir, |u| {
            u.flying = true;
            u.speed = 1.8;
            u.rotate_speed = 2.5;
            u.health = 2300.0;
            u.armor = 6.0;
            u.hit_size = 25.0;
            u.engine_size = 4.3;
            u.engine_offset = 54.0 / 4.0;
            u.item_capacity = 0;
            u.low_altitude = true;
        }),
        unit(&mut next_id, "quell", UnitKind::Erekir, |u| {
            u.low_altitude = false;
            u.flying = true;
            u.speed = 1.1;
            u.rotate_speed = 3.2;
            u.health = 6000.0;
            u.armor = 4.0;
            u.hit_size = 36.0;
            u.payload_capacity = 16.0 * super_tile_payload();
            u.research_cost_multiplier = 0.0;
            u.target_air = false;
            u.engine_size = 4.8;
            u.engine_offset = 61.0 / 4.0;
        }),
        unit(&mut next_id, "disrupt", UnitKind::Erekir, |u| {
            u.low_altitude = false;
            u.flying = true;
            u.speed = 1.0;
            u.rotate_speed = 2.0;
            u.health = 12000.0;
            u.armor = 9.0;
            u.hit_size = 46.0;
            u.payload_capacity = 36.0 * super_tile_payload();
            u.target_air = false;
            u.engine_size = 6.0;
            u.engine_offset = 25.25;
        }),
        unit(&mut next_id, "renale", UnitKind::Neoplasm, |u| {
            u.health = 500.0;
            u.armor = 2.0;
            u.hit_size = 9.0;
            u.rotate_speed = 2.5;
            u.drown_time_multiplier = 1.75;
            u.hidden = true;
            u.target_air = false;
            u.speed = 1.2;
        }),
        unit(&mut next_id, "latum", UnitKind::Neoplasm, |u| {
            u.health = 20000.0;
            u.armor = 12.0;
            u.hit_size = 48.0;
            u.rotate_speed = 1.7;
            u.hidden = true;
            u.target_air = false;
            u.speed = 1.0;
        }),
        unit(&mut next_id, "evoke", UnitKind::Erekir, |u| {
            u.is_enemy = false;
            u.low_altitude = false;
            u.flying = true;
            u.mine_speed = 6.0;
            u.mine_tier = 3;
            u.build_speed = 1.2;
            u.speed = 5.6;
            u.rotate_speed = 7.0;
            u.item_capacity = 60;
            u.health = 300.0;
            u.armor = 1.0;
            u.hit_size = 9.0;
            u.engine_size = 0.0;
            u.payload_capacity = 4.0 * super_tile_payload();
            u.targetable = false;
            u.hittable = false;
        }),
        unit(&mut next_id, "incite", UnitKind::Erekir, |u| {
            u.is_enemy = false;
            u.low_altitude = false;
            u.flying = true;
            u.mine_speed = 8.0;
            u.mine_tier = 3;
            u.build_speed = 1.4;
            u.speed = 7.0;
            u.rotate_speed = 8.0;
            u.item_capacity = 90;
            u.health = 500.0;
            u.armor = 2.0;
            u.hit_size = 11.0;
            u.payload_capacity = 4.0 * super_tile_payload();
            u.targetable = false;
            u.hittable = false;
            u.engine_offset = 7.2;
            u.engine_size = 3.1;
        }),
        unit(&mut next_id, "emanate", UnitKind::Erekir, |u| {
            u.is_enemy = false;
            u.low_altitude = false;
            u.flying = true;
            u.mine_speed = 9.0;
            u.mine_tier = 3;
            u.build_speed = 1.5;
            u.speed = 7.5;
            u.rotate_speed = 8.0;
            u.item_capacity = 110;
            u.health = 700.0;
            u.armor = 3.0;
            u.hit_size = 12.0;
            u.payload_capacity = 4.0 * super_tile_payload();
            u.targetable = false;
            u.hittable = false;
            u.engine_offset = 7.5;
            u.engine_size = 3.4;
        }),
        unit(&mut next_id, "block", UnitKind::Standard, |u| {
            u.speed = 0.0;
            u.hit_size = 0.0;
            u.health = 1.0;
            u.rotate_speed = 360.0;
            u.item_capacity = 0;
            u.hidden = true;
            u.internal = true;
        }),
        unit(&mut next_id, "manifold", UnitKind::Erekir, |u| {
            u.is_enemy = false;
            u.allowed_in_payloads = false;
            u.logic_controllable = false;
            u.player_controllable = false;
            u.payload_capacity = 0.0;
            u.low_altitude = false;
            u.flying = true;
            u.speed = 3.5;
            u.rotate_speed = 9.0;
            u.item_capacity = 100;
            u.health = 200.0;
            u.hit_size = 11.0;
            u.engine_size = 2.3;
            u.engine_offset = 6.5;
            u.hidden = true;
        }),
        unit(&mut next_id, "assembly-drone", UnitKind::Erekir, |u| {
            u.flying = true;
            u.speed = 1.3;
            u.health = 90.0;
            u.engine_size = 2.0;
            u.engine_offset = 6.5;
            u.payload_capacity = 0.0;
            u.targetable = false;
            u.bounded = false;
            u.is_enemy = false;
            u.hidden = true;
            u.use_unit_cap = false;
            u.logic_controllable = false;
            u.player_controllable = false;
            u.allowed_in_payloads = false;
            u.create_wreck = false;
        }),
    ];

    for unit in &mut units {
        apply_low_coupling_init(unit);
    }

    units
}

pub fn load_nested_missiles() -> Vec<UnitType> {
    vec![
        nested_missile(-1, "anthicus-missile", |u| {
            u.trail_color_rgba = Some(0x6bb6ffff);
            u.engine_color_rgba = Some(0x6bb6ffff);
            u.engine_size = 1.75;
            u.engine_layer = LAYER_EFFECT;
            u.speed = 3.35;
            u.max_range = 6.0;
            u.lifetime = 60.0 * 1.66;
            u.health = 55.0;
            u.low_altitude = true;
            u.parts.push("FlarePart".into());
        }),
        nested_missile(-2, "quell-missile", |u| {
            u.target_air = false;
            u.speed = 4.3;
            u.max_range = 6.0;
            u.lifetime = 60.0 * (1.4 - 0.496);
            u.engine_color_rgba = Some(0xbf92f9ff);
            u.trail_color_rgba = Some(0xbf92f9ff);
            u.engine_layer = LAYER_EFFECT;
            u.health = 45.0;
            u.loop_sound_volume = 0.1;
            u.parts.push("shootOnDeathWeapon".into());
        }),
        nested_missile(-3, "disrupt-missile", |u| {
            u.target_air = false;
            u.speed = 4.6;
            u.max_range = 5.0;
            u.health = 70.0;
            u.homing_delay = 10.0;
            u.low_altitude = true;
            u.engine_size = 3.0;
            u.engine_color_rgba = Some(0xbf92f9ff);
            u.trail_color_rgba = Some(0xbf92f9ff);
            u.engine_layer = LAYER_EFFECT;
            u.death_explosion_effect = "none".into();
            u.loop_sound_volume = 0.1;
            u.parts.push("ShapePart".into());
        }),
    ]
}

fn unit(
    next_id: &mut ContentId,
    name: &str,
    kind: UnitKind,
    configure: impl FnOnce(&mut UnitType),
) -> UnitType {
    let mut unit = UnitType::new(*next_id, name);
    *next_id += 1;

    apply_kind_defaults(&mut unit, kind);
    configure(&mut unit);
    unit
}

fn apply_kind_defaults(unit: &mut UnitType, kind: UnitKind) {
    match kind {
        UnitKind::Standard => {}
        UnitKind::Naval => {
            unit.naval = true;
            unit.can_drown = false;
            unit.emit_walk_sound = false;
            unit.omni_movement = false;
            unit.immunities.push("wet".into());
            if unit.shadow_elevation < 0.0 {
                unit.shadow_elevation = 0.11;
            }
        }
        UnitKind::Erekir => {
            apply_erekir_unit_type_defaults(unit);
        }
        UnitKind::Tank => {
            apply_tank_unit_type_defaults(unit);
        }
        UnitKind::Neoplasm => {
            apply_neoplasm_unit_type_defaults(unit);
        }
    }
}

fn apply_low_coupling_init(unit: &mut UnitType) {
    if unit.flying {
        unit.env_enabled |= Env::SPACE;
    }
}

const fn super_tile_payload() -> f32 {
    8.0 * 8.0
}

fn nested_missile(id: ContentId, name: &str, configure: impl FnOnce(&mut UnitType)) -> UnitType {
    let mut unit = missile_unit_type(id, name);
    configure(&mut unit);
    unit
}

const LAYER_EFFECT: f32 = 110.0;

#[cfg(test)]
mod tests {
    use super::*;

    fn names(units: &[UnitType]) -> Vec<&str> {
        units.iter().map(UnitType::name).collect()
    }

    fn by_name<'a>(units: &'a [UnitType], name: &str) -> &'a UnitType {
        units
            .iter()
            .find(|unit| unit.name() == name)
            .unwrap_or_else(|| panic!("missing unit: {name}"))
    }

    fn nested_missile_by_name<'a>(name: &str, missiles: &'a [UnitType]) -> &'a UnitType {
        missiles
            .iter()
            .find(|unit| unit.name() == name)
            .unwrap_or_else(|| panic!("missing nested missile: {name}"))
    }

    #[test]
    fn vanilla_unit_vector_order_matches_upstream_registration_order() {
        let units = load();
        assert_eq!(
            names(&units),
            vec![
                "dagger",
                "mace",
                "fortress",
                "scepter",
                "reign",
                "nova",
                "pulsar",
                "quasar",
                "vela",
                "corvus",
                "crawler",
                "atrax",
                "spiroct",
                "arkyid",
                "toxopid",
                "flare",
                "horizon",
                "zenith",
                "antumbra",
                "eclipse",
                "mono",
                "poly",
                "mega",
                "quad",
                "oct",
                "risso",
                "minke",
                "bryde",
                "sei",
                "omura",
                "retusa",
                "oxynoe",
                "cyerce",
                "aegires",
                "navanax",
                "alpha",
                "beta",
                "gamma",
                "stell",
                "locus",
                "precept",
                "vanquish",
                "conquer",
                "merui",
                "cleroi",
                "anthicus",
                "tecta",
                "collaris",
                "elude",
                "avert",
                "obviate",
                "quell",
                "disrupt",
                "renale",
                "latum",
                "evoke",
                "incite",
                "emanate",
                "block",
                "manifold",
                "assembly-drone",
            ]
        );
        assert_eq!(units.len(), 61);
    }

    #[test]
    fn unit_content_ids_keep_java_unit_types_load_order() {
        let units = load();
        for (idx, unit) in units.iter().enumerate() {
            assert_eq!(unit.id(), idx as i16, "id mismatch for {}", unit.name());
            assert_eq!(unit.content_type(), ContentType::Unit);
        }
        assert_eq!(by_name(&units, "dagger").id(), 0);
        assert_eq!(by_name(&units, "flare").id(), 15);
        assert_eq!(by_name(&units, "risso").id(), 25);
        assert_eq!(by_name(&units, "assembly-drone").id(), 60);
        assert!(units.iter().all(|unit| !unit.name().ends_with("-missile")));
    }

    #[test]
    fn unit_core_properties_match_upstream_subset() {
        let units = load();

        let dagger = by_name(&units, "dagger");
        assert_eq!(dagger.speed, 0.5);
        assert_eq!(dagger.hit_size, 8.0);
        assert_eq!(dagger.health, 150.0);
        assert_eq!(dagger.research_cost_multiplier, 0.5);

        let fortress = by_name(&units, "fortress");
        assert_eq!(fortress.rotate_speed, 3.0);
        assert!(!fortress.target_air);
        assert_eq!(fortress.armor, 9.0);
        assert_eq!(fortress.ammo_type, "item:graphite");

        let oct = by_name(&units, "oct");
        assert!(oct.flying);
        assert_eq!(oct.payload_capacity, 30.25 * 64.0);
        assert_eq!(oct.ammo_capacity, 1);

        let alpha = by_name(&units, "alpha");
        assert!(!alpha.is_enemy);
        assert!(alpha.flying);
        assert_eq!(alpha.mine_speed, 6.5);
        assert_eq!(alpha.item_capacity, 30);
        assert_eq!(alpha.default_command, None);

        let mono = by_name(&units, "mono");
        assert_eq!(mono.default_command.as_deref(), Some("mine"));
        let poly = by_name(&units, "poly");
        assert_eq!(poly.default_command.as_deref(), Some("rebuild"));
        let mega = by_name(&units, "mega");
        assert_eq!(mega.default_command.as_deref(), Some("repair"));
        let evoke = by_name(&units, "evoke");
        assert_eq!(evoke.default_command, None);
    }

    #[test]
    fn unit_kind_defaults_cover_java_constructor_and_init_side_effects() {
        let units = load();

        let risso = by_name(&units, "risso");
        assert!(risso.naval);
        assert!(!risso.can_drown);
        assert!(!risso.omni_movement);
        assert!(risso.immunities.iter().any(|entry| entry == "wet"));
        assert_eq!(risso.shadow_elevation, 0.11);

        let stell = by_name(&units, "stell");
        assert!(stell.square_shape);
        assert!(stell.rotate_move_first);
        assert_eq!(stell.env_disabled, Env::NONE);
        assert_eq!(stell.research_cost_multiplier, 0.0);

        let merui = by_name(&units, "merui");
        assert_eq!(merui.env_disabled, Env::SPACE);
        assert_eq!(merui.ammo_type, "item:beryllium");
        assert!(merui.allow_leg_step);
        assert_eq!(merui.ground_layer, LAYER_LEG_UNIT - 1.0);

        let renale = by_name(&units, "renale");
        assert_eq!(renale.env_disabled, Env::NONE);
        assert!(!renale.draw_cell);
        assert!(renale.immunities.iter().any(|entry| entry == "burning"));
        assert!(renale.immunities.iter().any(|entry| entry == "melting"));
        assert!(renale
            .abilities
            .iter()
            .any(|entry| entry == "LiquidExplodeAbility:neoplasm"));

        let flare = by_name(&units, "flare");
        assert_eq!(flare.env_enabled, Env::TERRESTRIAL | Env::SPACE);

        let assembly = by_name(&units, "assembly-drone");
        assert!(!assembly.use_unit_cap);
        assert!(!assembly.allowed_in_payloads);
        assert!(!assembly.create_wreck);
    }

    #[test]
    fn nested_missile_units_match_upstream_spawn_unit_presets_without_main_registration() {
        let missiles = load_nested_missiles();
        assert_eq!(
            names(&missiles),
            vec!["anthicus-missile", "quell-missile", "disrupt-missile"]
        );
        assert_eq!(
            missiles.iter().map(Content::id).collect::<Vec<_>>(),
            vec![-1, -2, -3]
        );

        for missile in &missiles {
            assert_eq!(missile.content_type(), ContentType::Unit);
            assert!(missile.hidden);
            assert!(missile.flying);
            assert_eq!(missile.env_enabled, Env::ANY);
            assert!(!missile.allowed_in_payloads);
            assert!(!missile.physics);
            assert!(!missile.draw_minimap);
        }

        let anthicus = nested_missile_by_name("anthicus-missile", &missiles);
        assert_eq!(anthicus.speed, 3.35);
        assert_eq!(anthicus.max_range, 6.0);
        assert_eq!(anthicus.lifetime, 60.0 * 1.66);
        assert_eq!(anthicus.health, 55.0);
        assert!(anthicus.low_altitude);
        assert_eq!(anthicus.engine_size, 1.75);
        assert_eq!(anthicus.engine_layer, LAYER_EFFECT);
        assert!(anthicus.parts.iter().any(|part| part == "FlarePart"));

        let quell = nested_missile_by_name("quell-missile", &missiles);
        assert!(!quell.target_air);
        assert_eq!(quell.speed, 4.3);
        assert!((quell.lifetime - 54.24).abs() < 0.001);
        assert_eq!(quell.health, 45.0);
        assert_eq!(quell.loop_sound_volume, 0.1);
        assert!(quell.parts.iter().any(|part| part == "shootOnDeathWeapon"));

        let disrupt = nested_missile_by_name("disrupt-missile", &missiles);
        assert!(!disrupt.target_air);
        assert_eq!(disrupt.speed, 4.6);
        assert_eq!(disrupt.max_range, 5.0);
        assert_eq!(disrupt.health, 70.0);
        assert_eq!(disrupt.homing_delay, 10.0);
        assert_eq!(disrupt.engine_size, 3.0);
        assert_eq!(disrupt.death_explosion_effect, "none");
        assert!(disrupt.parts.iter().any(|part| part == "ShapePart"));
    }
}
