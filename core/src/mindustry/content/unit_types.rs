use crate::mindustry::{
    ctype::ContentId,
    r#type::{
        unit::{
            erekir_unit_type::apply_erekir_unit_type_defaults,
            missile_unit_type::missile_unit_type,
            neoplasm_unit_type::apply_neoplasm_unit_type_defaults,
            tank_unit_type::apply_tank_unit_type_defaults,
        },
        UnitType, Weapon,
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
            let mut weapon = Weapon::new("large-weapon");
            weapon.reload = 13.0;
            weapon.x = 4.0;
            weapon.y = 2.0;
            weapon.top = false;
            weapon.eject_effect = "casing1".into();
            weapon.bullet = "dagger_basic".into();
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "mace", UnitKind::Standard, |u| {
            u.speed = 0.5;
            u.hit_size = 10.0;
            u.health = 550.0;
            u.armor = 4.0;
            u.ammo_type = "item:coal".into();
            u.immunities.push("burning".into());
            let mut weapon = Weapon::new("flamethrower");
            weapon.top = false;
            weapon.shoot_sound = "shootFlame".into();
            weapon.shoot_y = 2.0;
            weapon.reload = 22.0;
            weapon.recoil = 1.0;
            weapon.eject_effect = "none".into();
            weapon.bullet = "mace_flame".into();
            u.weapons.push(weapon);
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
            let mut weapon = Weapon::new("artillery");
            weapon.top = false;
            weapon.y = 1.0;
            weapon.x = 9.0;
            weapon.reload = 60.0;
            weapon.recoil = 4.0;
            weapon.shake = 2.0;
            weapon.eject_effect = "casing2".into();
            weapon.shoot_sound = "shootArtillery".into();
            weapon.bullet = "fortress_artillery".into();
            u.weapons.push(weapon);
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
            u.abilities
                .push("ShieldRegenFieldAbility:25:250:60:60".into());
            let mut main = Weapon::new("scepter-weapon");
            main.top = false;
            main.y = 1.0;
            main.x = 16.0;
            main.shoot_y = 8.0;
            main.reload = 45.0;
            main.recoil = 5.0;
            main.shake = 2.0;
            main.eject_effect = "casing3".into();
            main.shoot_sound = "shootScepter".into();
            main.shoot_sound_volume = 0.95;
            main.inaccuracy = 3.0;
            main.shoot_shots = 3;
            main.shoot_shot_delay = 4.0;
            main.bullet = "scepter_bullet".into();
            u.weapons.push(main);

            let mut upper_mount = Weapon::new("scepter-mount");
            upper_mount.reload = 12.0;
            upper_mount.x = 8.5;
            upper_mount.y = 6.0;
            upper_mount.rotate = true;
            upper_mount.eject_effect = "casing1".into();
            upper_mount.bullet = "scepter_small_bullet".into();
            upper_mount.shoot_sound = "shootScepterSecondary".into();
            upper_mount.rotate_speed = 3.0;
            u.weapons.push(upper_mount);

            let mut lower_mount = Weapon::new("scepter-mount");
            lower_mount.reload = 15.0;
            lower_mount.x = 8.5;
            lower_mount.y = -7.0;
            lower_mount.rotate = true;
            lower_mount.eject_effect = "casing1".into();
            lower_mount.bullet = "scepter_small_bullet".into();
            lower_mount.shoot_sound = "shootScepterSecondary".into();
            lower_mount.rotate_speed = 3.0;
            u.weapons.push(lower_mount);
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
            let mut weapon = Weapon::new("reign-weapon");
            weapon.top = false;
            weapon.y = 1.0;
            weapon.x = 21.5;
            weapon.shoot_y = 11.0;
            weapon.reload = 9.0;
            weapon.recoil = 5.0;
            weapon.shake = 2.0;
            weapon.eject_effect = "casing4".into();
            weapon.shoot_sound = "shootReign".into();
            weapon.bullet = "reign_shell".into();
            u.weapons.push(weapon);
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
            u.abilities.push("RepairFieldAbility:10:240:60".into());
            let mut weapon = Weapon::new("heal-weapon");
            weapon.top = false;
            weapon.shoot_y = 2.0;
            weapon.reload = 24.0;
            weapon.x = 4.5;
            weapon.alternate = false;
            weapon.eject_effect = "none".into();
            weapon.recoil = 2.0;
            weapon.shoot_sound = "shootLaser".into();
            weapon.bullet = "nova_heal_bolt".into();
            u.weapons.push(weapon);
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
            u.abilities
                .push("ShieldRegenFieldAbility:20:40:300:60".into());
            let mut weapon = Weapon::new("heal-shotgun-weapon");
            weapon.top = false;
            weapon.x = 5.0;
            weapon.shake = 2.2;
            weapon.y = 0.5;
            weapon.shoot_y = 2.5;
            weapon.reload = 36.0;
            weapon.inaccuracy = 35.0;
            weapon.shoot_shots = 3;
            weapon.shoot_shot_delay = 0.5;
            weapon.eject_effect = "none".into();
            weapon.recoil = 2.5;
            weapon.shoot_sound = "shootPulsar".into();
            weapon.bullet = "pulsar_heal_lightning".into();
            u.weapons.push(weapon);
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
            u.abilities.push("ForceFieldAbility:60:0.4:500:360".into());
            let mut weapon = Weapon::new("beam-weapon");
            weapon.top = false;
            weapon.shake = 2.0;
            weapon.shoot_y = 4.0;
            weapon.x = 6.5;
            weapon.reload = 55.0;
            weapon.recoil = 4.0;
            weapon.shoot_sound = "shootLancer".into();
            weapon.bullet = "quasar_beam".into();
            u.weapons.push(weapon);
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
            let mut laser = Weapon::new("vela-weapon");
            laser.mirror = false;
            laser.top = false;
            laser.shake = 4.0;
            laser.shoot_y = 14.0;
            laser.x = 0.0;
            laser.y = 0.0;
            laser.shoot_first_shot_delay = 40.0 - 1.0;
            laser.parentize_effects = true;
            laser.reload = 155.0;
            laser.recoil = 0.0;
            laser.charge_sound = "chargeVela".into();
            laser.shoot_sound = "beamPlasma".into();
            laser.initial_shoot_sound = "shootBeamPlasma".into();
            laser.continuous = true;
            laser.cooldown_time = 200.0;
            laser.shoot_status = "slow".into();
            laser.shoot_status_duration = 160.0 + laser.shoot_first_shot_delay;
            laser.bullet = "vela_continuous_laser".into();
            u.weapons.push(laser);

            let mut repair = Weapon::new("repair-beam-weapon-center-large");
            repair.x = 44.0 / 4.0;
            repair.y = -30.0 / 4.0;
            repair.shoot_y = 6.0;
            repair.beam_width = 0.8;
            repair.repair_speed = 1.4;
            repair.reload = 1.0;
            repair.predict_target = false;
            repair.auto_target = true;
            repair.controllable = false;
            repair.rotate = true;
            repair.mount_type = "HealBeamMount".into();
            repair.recoil = 0.0;
            repair.no_attack = true;
            repair.use_attack_range = false;
            repair.active_sound = "beamHeal".into();
            repair.bullet = "vela_repair_range".into();
            u.weapons.push(repair);
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
            let mut weapon = Weapon::new("corvus-weapon");
            weapon.shoot_sound = "shootCorvus".into();
            weapon.charge_sound = "chargeCorvus".into();
            weapon.sound_pitch_min = 1.0;
            weapon.top = false;
            weapon.mirror = false;
            weapon.shake = 14.0;
            weapon.shoot_y = 5.0;
            weapon.x = 0.0;
            weapon.y = 0.0;
            weapon.reload = 350.0;
            weapon.recoil = 0.0;
            weapon.cooldown_time = 350.0;
            weapon.shoot_status_duration = 60.0 * 2.0;
            weapon.shoot_status = "unmoving".into();
            weapon.shoot_first_shot_delay = 80.0;
            weapon.parentize_effects = true;
            weapon.bullet = "corvus_laser".into();
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "crawler", UnitKind::Standard, |u| {
            u.research_cost_multiplier = 0.5;
            u.speed = 1.0;
            u.hit_size = 8.0;
            u.health = 150.0;
            u.mech_side_sway = 0.25;
            u.range = 40.0;
            u.target_under_blocks = false;
            u.step_sound = "walkerStepTiny".into();
            u.step_sound_volume = 0.2;
            u.allow_leg_step = true;
            let mut weapon = Weapon::new("");
            weapon.shoot_on_death = true;
            weapon.reload = 24.0;
            weapon.shoot_cone = 180.0;
            weapon.eject_effect = "none".into();
            weapon.shoot_sound = "explosionCrawler".into();
            weapon.shoot_sound_volume = 0.4;
            weapon.x = 0.0;
            weapon.shoot_y = 0.0;
            weapon.mirror = false;
            weapon.bullet = "crawler_explosion".into();
            weapon.bullet_kill_shooter = true;
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "atrax", UnitKind::Standard, |u| {
            u.speed = 0.6;
            u.drag = 0.4;
            u.hit_size = 13.0;
            u.rotate_speed = 3.0;
            u.target_air = false;
            u.health = 600.0;
            u.immunities.push("burning".into());
            u.immunities.push("melting".into());
            u.step_sound = "walkerStepSmall".into();
            u.step_sound_pitch = 1.0;
            u.step_sound_volume = 0.25;
            u.leg_count = 4;
            u.leg_length = 9.0;
            u.leg_forward_scl = 0.6;
            u.leg_move_space = 1.4;
            u.hovering = true;
            u.armor = 3.0;
            u.shadow_elevation = 0.2;
            u.ground_layer = LAYER_LEG_UNIT - 1.0;
            u.allow_leg_step = true;
            let mut weapon = Weapon::new("atrax-weapon");
            weapon.top = false;
            weapon.shoot_y = 3.0;
            weapon.reload = 9.0;
            weapon.eject_effect = "none".into();
            weapon.recoil = 1.0;
            weapon.x = 7.0;
            weapon.shoot_sound = "shootAtrax".into();
            weapon.bullet = "atrax_slag".into();
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "spiroct", UnitKind::Standard, |u| {
            u.speed = 0.54;
            u.drag = 0.4;
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
            u.shadow_elevation = 0.3;
            u.ground_layer = LAYER_LEG_UNIT;
            u.step_sound = "walkerStepSmall".into();
            u.step_sound_pitch = 0.7;
            u.step_sound_volume = 0.35;
            u.allow_leg_step = true;
            let mut weapon = Weapon::new("spiroct-weapon");
            weapon.shoot_y = 4.0;
            weapon.reload = 14.0;
            weapon.eject_effect = "none".into();
            weapon.recoil = 2.0;
            weapon.rotate = true;
            weapon.shoot_sound = "shootSap".into();
            weapon.x = 8.5;
            weapon.y = -1.5;
            weapon.bullet = "spiroct_sap".into();
            u.weapons.push(weapon);

            let mut mount = Weapon::new("mount-purple-weapon");
            mount.reload = 18.0;
            mount.rotate = true;
            mount.x = 4.0;
            mount.y = 3.0;
            mount.shoot_sound = "shootSap".into();
            mount.bullet = "spiroct_mount_sap".into();
            u.weapons.push(mount);
        }),
        unit(&mut next_id, "arkyid", UnitKind::Standard, |u| {
            u.drag = 0.1;
            u.speed = 0.62;
            u.hit_size = 23.0;
            u.health = 8000.0;
            u.armor = 6.0;
            u.rotate_speed = 2.7;
            u.leg_count = 6;
            u.leg_move_space = 1.0;
            u.leg_pair_offset = 3.0;
            u.leg_length = 30.0;
            u.leg_extension = -15.0;
            u.leg_base_offset = 10.0;
            u.step_shake = 1.0;
            u.leg_length_scl = 0.96;
            u.ripple_scale = 2.0;
            u.leg_speed = 0.2;
            u.step_sound = "walkerStep".into();
            u.step_sound_volume = 0.85;
            u.step_sound_pitch = 1.1;
            u.leg_splash_damage = 32.0;
            u.leg_splash_range = 30.0;
            u.hovering = true;
            u.shadow_elevation = 0.65;
            u.ground_layer = LAYER_LEG_UNIT;
            u.allow_leg_step = true;

            for (reload, x, y) in [(9.0, 4.0, 8.0), (14.0, 9.0, 6.0), (22.0, 14.0, 0.0)] {
                let mut weapon = Weapon::new("spiroct-weapon");
                weapon.reload = reload;
                weapon.x = x;
                weapon.y = y;
                weapon.rotate = true;
                weapon.bullet = "arkyid_sapper".into();
                weapon.shoot_sound = "shootSap".into();
                u.weapons.push(weapon);
            }

            let mut artillery = Weapon::new("large-purple-mount");
            artillery.y = -7.0;
            artillery.x = 9.0;
            artillery.shoot_y = 7.0;
            artillery.reload = 45.0;
            artillery.shake = 3.0;
            artillery.rotate_speed = 2.0;
            artillery.eject_effect = "casing1".into();
            artillery.shoot_sound = "shootArtillerySap".into();
            artillery.rotate = true;
            artillery.shadow = 8.0;
            artillery.recoil = 3.0;
            artillery.shoot_pattern = "ShootSpread".into();
            artillery.shoot_shots = 2;
            artillery.shoot_spread = 17.0;
            artillery.bullet = "arkyid_artillery_sap".into();
            u.weapons.push(artillery);
        }),
        unit(&mut next_id, "toxopid", UnitKind::Standard, |u| {
            u.drag = 0.1;
            u.speed = 0.5;
            u.hit_size = 26.0;
            u.health = 22000.0;
            u.armor = 13.0;
            u.light_radius = 140.0;
            u.step_sound = "walkerStep".into();
            u.step_sound_volume = 1.1;
            u.rotate_speed = 1.9;
            u.leg_count = 8;
            u.leg_move_space = 0.8;
            u.leg_pair_offset = 3.0;
            u.leg_length = 75.0;
            u.leg_extension = -20.0;
            u.leg_base_offset = 8.0;
            u.step_shake = 1.0;
            u.leg_length_scl = 0.93;
            u.ripple_scale = 3.0;
            u.leg_speed = 0.19;
            u.leg_splash_damage = 80.0;
            u.leg_splash_range = 60.0;
            u.hovering = true;
            u.shadow_elevation = 0.95;
            u.ground_layer = LAYER_LEG_UNIT;
            u.allow_leg_step = true;

            let mut shrapnel = Weapon::new("large-purple-mount");
            shrapnel.y = -5.0;
            shrapnel.x = 11.0;
            shrapnel.shoot_y = 7.0;
            shrapnel.reload = 30.0;
            shrapnel.shake = 4.0;
            shrapnel.rotate_speed = 2.0;
            shrapnel.eject_effect = "casing1".into();
            shrapnel.shoot_sound = "shootToxopidShotgun".into();
            shrapnel.shoot_sound_volume = 0.8;
            shrapnel.rotate = true;
            shrapnel.shadow = 12.0;
            shrapnel.recoil = 3.0;
            shrapnel.shoot_pattern = "ShootSpread".into();
            shrapnel.shoot_shots = 2;
            shrapnel.shoot_spread = 17.0;
            shrapnel.bullet = "toxopid_shrapnel".into();
            u.weapons.push(shrapnel);

            let mut cannon = Weapon::new("toxopid-cannon");
            cannon.y = -14.0;
            cannon.x = 0.0;
            cannon.shoot_y = 22.0;
            cannon.mirror = false;
            cannon.reload = 210.0;
            cannon.shake = 10.0;
            cannon.recoil = 10.0;
            cannon.rotate_speed = 1.0;
            cannon.eject_effect = "casing3".into();
            cannon.shoot_sound = "shootArtillerySapBig".into();
            cannon.rotate = true;
            cannon.shadow = 30.0;
            cannon.rotation_limit = 80.0;
            cannon.bullet = "toxopid_cannon".into();
            u.weapons.push(cannon);
        }),
        unit(&mut next_id, "flare", UnitKind::Standard, |u| {
            u.research_cost_multiplier = 0.5;
            u.speed = 2.7;
            u.accel = 0.08;
            u.drag = 0.04;
            u.flying = true;
            u.health = 70.0;
            u.engine_offset = 5.75;
            u.target_flags = vec![Some("generator".into()), None];
            u.hit_size = 9.0;
            u.item_capacity = 10;
            u.circle_target = true;
            u.omni_movement = false;
            u.rotate_speed = 5.0;
            u.circle_target_radius = 60.0;
            u.wreck_sound_volume = 0.7;
            u.move_sound = "loopThruster".into();
            u.move_sound_pitch_min = 0.3;
            u.move_sound_pitch_max = 1.5;
            u.move_sound_volume = 0.2;
            let mut weapon = Weapon::new("");
            weapon.y = 1.0;
            weapon.x = 0.0;
            weapon.min_shoot_velocity = 2.0;
            weapon.shoot_cone = 10.0;
            weapon.reload = 80.0;
            weapon.shoot_shots = 3;
            weapon.shoot_shot_delay = 3.0;
            weapon.eject_effect = "casing1".into();
            weapon.mirror = false;
            weapon.bullet = "flare_basic".into();
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "horizon", UnitKind::Standard, |u| {
            u.health = 340.0;
            u.speed = 1.65;
            u.accel = 0.08;
            u.drag = 0.03;
            u.flying = true;
            u.hit_size = 11.0;
            u.target_air = false;
            u.engine_offset = 7.8;
            u.range = 140.0;
            u.face_target = false;
            u.auto_drop_bombs = true;
            u.armor = 3.0;
            u.item_capacity = 0;
            u.target_flags = vec![Some("factory".into()), None];
            u.circle_target = true;
            u.omni_movement = false;
            u.rotate_speed = 4.5;
            u.circle_target_radius = 40.0;
            u.move_sound = "loopThruster".into();
            u.move_sound_pitch_min = 0.6;
            u.move_sound_volume = 0.4;
            let mut weapon = Weapon::new("");
            weapon.min_shoot_velocity = 1.0;
            weapon.x = 3.0;
            weapon.shoot_y = 0.0;
            weapon.reload = 12.0;
            weapon.shoot_cone = 180.0;
            weapon.eject_effect = "none".into();
            weapon.inaccuracy = 15.0;
            weapon.ignore_rotation = true;
            weapon.shoot_sound = "shootHorizon".into();
            weapon.sound_pitch_max = 1.2;
            weapon.bullet = "horizon_bomb".into();
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "zenith", UnitKind::Standard, |u| {
            u.health = 700.0;
            u.speed = 1.7;
            u.accel = 0.04;
            u.drag = 0.016;
            u.flying = true;
            u.range = 140.0;
            u.hit_size = 20.0;
            u.low_altitude = true;
            u.force_multi_target = true;
            u.armor = 5.0;
            u.target_flags = vec![
                Some("launchPad".into()),
                Some("storage".into()),
                Some("battery".into()),
                None,
            ];
            u.engine_offset = 12.0;
            u.engine_size = 3.0;
            let mut weapon = Weapon::new("zenith-missiles");
            weapon.reload = 40.0;
            weapon.x = 7.0;
            weapon.rotate = true;
            weapon.shake = 1.0;
            weapon.shoot_shots = 2;
            weapon.inaccuracy = 5.0;
            weapon.velocity_rnd = 0.2;
            weapon.shoot_sound = "shootMissileLong".into();
            weapon.bullet = "zenith_missile".into();
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "antumbra", UnitKind::Standard, |u| {
            u.speed = 0.8;
            u.accel = 0.04;
            u.drag = 0.04;
            u.rotate_speed = 1.9;
            u.flying = true;
            u.low_altitude = true;
            u.health = 7200.0;
            u.armor = 9.0;
            u.engine_offset = 21.0;
            u.engine_size = 5.3;
            u.hit_size = 46.0;
            u.target_flags = vec![Some("generator".into()), Some("core".into()), None];
            u.loop_sound = "loopHover".into();

            let mut front_missile = Weapon::new("missiles-mount");
            front_missile.y = 8.0;
            front_missile.x = 17.0;
            front_missile.reload = 20.0;
            front_missile.eject_effect = "casing1".into();
            front_missile.rotate_speed = 8.0;
            front_missile.bullet = "antumbra_missile".into();
            front_missile.shoot_sound = "shootMissile".into();
            front_missile.rotate = true;
            front_missile.shadow = 6.0;
            u.weapons.push(front_missile);

            let mut rear_missile = Weapon::new("missiles-mount");
            rear_missile.y = -8.0;
            rear_missile.x = 17.0;
            rear_missile.reload = 35.0;
            rear_missile.rotate_speed = 8.0;
            rear_missile.eject_effect = "casing1".into();
            rear_missile.bullet = "antumbra_missile".into();
            rear_missile.shoot_sound = "shootMissile".into();
            rear_missile.rotate = true;
            rear_missile.shadow = 6.0;
            u.weapons.push(rear_missile);

            let mut large_mount = Weapon::new("large-bullet-mount");
            large_mount.y = 2.0;
            large_mount.x = 10.0;
            large_mount.shoot_y = 10.0;
            large_mount.reload = 12.0;
            large_mount.shake = 1.0;
            large_mount.rotate_speed = 2.0;
            large_mount.eject_effect = "casing1".into();
            large_mount.shoot_sound = "shootSpectre".into();
            large_mount.rotate = true;
            large_mount.shadow = 8.0;
            large_mount.bullet = "antumbra_large_bullet".into();
            u.weapons.push(large_mount);
        }),
        unit(&mut next_id, "eclipse", UnitKind::Standard, |u| {
            u.speed = 0.54;
            u.accel = 0.04;
            u.drag = 0.04;
            u.rotate_speed = 1.0;
            u.flying = true;
            u.low_altitude = true;
            u.health = 22000.0;
            u.engine_offset = 38.0;
            u.engine_size = 7.3;
            u.hit_size = 58.0;
            u.armor = 13.0;
            u.target_flags = vec![
                Some("reactor".into()),
                Some("battery".into()),
                Some("core".into()),
                None,
            ];
            u.loop_sound = "loopHover".into();

            let mut laser = Weapon::new("large-laser-mount");
            laser.shake = 4.0;
            laser.shoot_y = 9.0;
            laser.x = 18.0;
            laser.y = 5.0;
            laser.rotate_speed = 2.0;
            laser.reload = 45.0;
            laser.recoil = 4.0;
            laser.shoot_sound = "shootEclipse".into();
            laser.shadow = 20.0;
            laser.rotate = true;
            laser.bullet = "eclipse_laser".into();
            u.weapons.push(laser);

            let mut top_artillery = Weapon::new("large-artillery");
            top_artillery.x = 11.0;
            top_artillery.y = 27.0;
            top_artillery.rotate_speed = 2.0;
            top_artillery.reload = 9.0;
            top_artillery.shoot_sound = "shootCyclone".into();
            top_artillery.shadow = 7.0;
            top_artillery.rotate = true;
            top_artillery.recoil = 0.5;
            top_artillery.shoot_y = 7.25;
            top_artillery.bullet = "eclipse_flak".into();
            u.weapons.push(top_artillery);

            let mut lower_artillery = Weapon::new("large-artillery");
            lower_artillery.y = -13.0;
            lower_artillery.x = 20.0;
            lower_artillery.reload = 12.0;
            lower_artillery.eject_effect = "casing1".into();
            lower_artillery.rotate_speed = 7.0;
            lower_artillery.shake = 1.0;
            lower_artillery.shoot_sound = "shootCyclone".into();
            lower_artillery.rotate = true;
            lower_artillery.shadow = 12.0;
            lower_artillery.shoot_y = 7.25;
            lower_artillery.bullet = "eclipse_flak".into();
            u.weapons.push(lower_artillery);
        }),
        unit(&mut next_id, "mono", UnitKind::Standard, |u| {
            u.flying = true;
            u.drag = 0.06;
            u.accel = 0.12;
            u.speed = 1.5;
            u.health = 100.0;
            u.engine_size = 1.8;
            u.engine_offset = 5.7;
            u.range = 50.0;
            u.is_enemy = false;
            u.control_select_global = false;
            u.wreck_sound_volume = 0.7;
            u.death_sound_volume = 0.7;
            u.mine_tier = 1;
            u.mine_speed = 2.5;
            u.default_command = Some("mine".into());
        }),
        unit(&mut next_id, "poly", UnitKind::Standard, |u| {
            u.flying = true;
            u.drag = 0.05;
            u.speed = 2.6;
            u.rotate_speed = 15.0;
            u.accel = 0.1;
            u.range = 130.0;
            u.health = 400.0;
            u.build_speed = 0.5;
            u.engine_offset = 6.5;
            u.hit_size = 9.0;
            u.low_altitude = true;
            u.mine_tier = 2;
            u.mine_speed = 3.5;
            u.wreck_sound_volume = 0.9;
            u.default_command = Some("rebuild".into());
            u.abilities.push("RepairFieldAbility:5:480:50".into());

            let mut weapon = Weapon::new("poly-weapon");
            weapon.top = false;
            weapon.y = -2.5;
            weapon.x = 3.75;
            weapon.reload = 30.0;
            weapon.eject_effect = "none".into();
            weapon.recoil = 2.0;
            weapon.shoot_sound = "shootMissilePlasmaShort".into();
            weapon.velocity_rnd = 0.5;
            weapon.inaccuracy = 15.0;
            weapon.alternate = true;
            weapon.bullet = "poly_missile".into();
            u.weapons.push(weapon);
        }),
        unit(&mut next_id, "mega", UnitKind::Standard, |u| {
            u.mine_tier = 3;
            u.mine_speed = 4.0;
            u.health = 460.0;
            u.armor = 3.0;
            u.speed = 2.5;
            u.accel = 0.06;
            u.drag = 0.017;
            u.low_altitude = true;
            u.flying = true;
            u.engine_offset = 10.5;
            u.face_target = false;
            u.hit_size = 16.05;
            u.engine_size = 3.0;
            u.payload_capacity = 4.0 * super_tile_payload();
            u.build_speed = 2.6;
            u.is_enemy = false;
            u.default_command = Some("repair".into());

            let mut large = Weapon::new("heal-weapon-mount");
            large.shoot_sound = "shootLaser".into();
            large.reload = 24.0;
            large.x = 8.0;
            large.y = -6.0;
            large.rotate = true;
            large.bullet = "mega_heal_bolt_large".into();
            u.weapons.push(large);

            let mut small = Weapon::new("heal-weapon-mount");
            small.shoot_sound = "shootLaser".into();
            small.reload = 15.0;
            small.x = 4.0;
            small.y = 5.0;
            small.rotate = true;
            small.bullet = "mega_heal_bolt_small".into();
            u.weapons.push(small);
        }),
        unit(&mut next_id, "quad", UnitKind::Standard, |u| {
            u.armor = 8.0;
            u.health = 6000.0;
            u.speed = 1.2;
            u.rotate_speed = 2.0;
            u.accel = 0.05;
            u.drag = 0.017;
            u.low_altitude = false;
            u.flying = true;
            u.auto_drop_bombs = true;
            u.circle_target = true;
            u.engine_offset = 13.0;
            u.engine_size = 7.0;
            u.face_target = false;
            u.hit_size = 36.0;
            u.payload_capacity = 9.0 * super_tile_payload();
            u.build_speed = 2.5;
            u.build_beam_offset = 23.0;
            u.range = 140.0;
            u.target_air = false;
            u.target_flags = vec![Some("battery".into()), Some("factory".into()), None];
            u.loop_sound = "loopHover".into();

            let mut bomb = Weapon::new("");
            bomb.x = 0.0;
            bomb.y = 0.0;
            bomb.mirror = false;
            bomb.reload = 55.0;
            bomb.min_shoot_velocity = 0.01;
            bomb.sound_pitch_min = 1.0;
            bomb.shoot_sound = "shootQuad".into();
            bomb.bullet = "quad_bomb".into();
            u.weapons.push(bomb);
        }),
        unit(&mut next_id, "oct", UnitKind::Standard, |u| {
            u.armor = 16.0;
            u.health = 24000.0;
            u.speed = 0.8;
            u.rotate_speed = 1.0;
            u.accel = 0.04;
            u.drag = 0.018;
            u.flying = true;
            u.engine_offset = 46.0;
            u.engine_size = 7.8;
            u.face_target = false;
            u.hit_size = 66.0;
            u.payload_capacity = 30.25 * super_tile_payload();
            u.build_speed = 4.0;
            u.draw_shields = false;
            u.low_altitude = true;
            u.build_beam_offset = 43.0;
            u.loop_sound = "loopHover".into();
            u.ammo_capacity = 1;
            u.abilities
                .push("ForceFieldAbility:140:4:7000:480:8:0".into());
            u.abilities.push("RepairFieldAbility:130:120:140".into());
        }),
        unit(&mut next_id, "risso", UnitKind::Naval, |u| {
            u.speed = 1.1;
            u.drag = 0.13;
            u.hit_size = 10.0;
            u.health = 280.0;
            u.armor = 2.0;
            u.accel = 0.4;
            u.rotate_speed = 3.3;
            u.face_target = false;
            u.trail_length = 20;
            u.wave_trail_x = 4.0;
            u.trail_scl = 1.3;
            u.move_sound_volume = 0.4;
            u.move_sound = "shipMove".into();

            let mut basic = Weapon::new("mount-weapon");
            basic.reload = 13.0;
            basic.x = 4.0;
            basic.shoot_y = 4.0;
            basic.y = 1.5;
            basic.rotate = true;
            basic.eject_effect = "casing1".into();
            basic.bullet = "risso_basic".into();
            u.weapons.push(basic);

            let mut missile = Weapon::new("missiles-mount");
            missile.mirror = false;
            missile.reload = 25.0;
            missile.x = 0.0;
            missile.y = -5.0;
            missile.rotate = true;
            missile.eject_effect = "casing1".into();
            missile.shoot_sound = "shootMissileShort".into();
            missile.bullet = "risso_missile".into();
            u.weapons.push(missile);
        }),
        unit(&mut next_id, "minke", UnitKind::Naval, |u| {
            u.health = 600.0;
            u.speed = 0.9;
            u.drag = 0.15;
            u.hit_size = 13.0;
            u.armor = 4.0;
            u.accel = 0.3;
            u.rotate_speed = 2.6;
            u.face_target = false;
            u.move_sound_volume = 0.55;
            u.move_sound_pitch_min = 0.9;
            u.move_sound_pitch_max = 0.9;
            u.move_sound = "shipMove".into();
            u.trail_length = 20;
            u.wave_trail_x = 5.5;
            u.wave_trail_y = -4.0;
            u.trail_scl = 1.9;

            let mut flak = Weapon::new("mount-weapon");
            flak.reload = 10.0;
            flak.x = 5.0;
            flak.y = 3.5;
            flak.rotate = true;
            flak.rotate_speed = 5.0;
            flak.inaccuracy = 8.0;
            flak.eject_effect = "casing1".into();
            flak.shoot_sound = "shootDuo".into();
            flak.bullet = "minke_flak".into();
            u.weapons.push(flak);

            let mut artillery = Weapon::new("artillery-mount");
            artillery.reload = 30.0;
            artillery.x = 5.0;
            artillery.y = -5.0;
            artillery.rotate = true;
            artillery.inaccuracy = 2.0;
            artillery.rotate_speed = 2.0;
            artillery.shake = 1.5;
            artillery.eject_effect = "casing2".into();
            artillery.shoot_sound = "shootArtillerySmall".into();
            artillery.bullet = "minke_artillery".into();
            u.weapons.push(artillery);
        }),
        unit(&mut next_id, "bryde", UnitKind::Naval, |u| {
            u.health = 910.0;
            u.speed = 0.85;
            u.rotate_speed = 1.8;
            u.hit_size = 20.0;
            u.armor = 7.0;
            u.abilities
                .push("ShieldRegenFieldAbility:20:40:240:60".into());
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
            u.abilities
                .push("StatusFieldAbility:overclock:360:360:60".into());
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
            u.abilities
                .push("EnergyFieldAbility:40:65:180:1.5:0.5:25".into());
        }),
        unit(&mut next_id, "navanax", UnitKind::Naval, |u| {
            u.health = 20000.0;
            u.speed = 0.65;
            u.hit_size = 58.0;
            u.armor = 16.0;
            u.rotate_speed = 1.1;
            u.build_speed = 3.5;
            u.abilities
                .push("SuppressionFieldAbility:90:90:200:0:-10:true:13".into());
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
            u.target_buildings_mobile = false;
            u.flying = true;
            u.mine_speed = 7.0;
            u.mine_tier = 1;
            u.build_speed = 0.75;
            u.drag = 0.05;
            u.speed = 3.3;
            u.rotate_speed = 17.0;
            u.accel = 0.1;
            u.fog_radius = 0.0;
            u.item_capacity = 50;
            u.health = 170.0;
            u.engine_offset = 6.0;
            u.hit_size = 9.0;
            u.low_altitude = true;
            let mut weapon = Weapon::new("small-mount-weapon");
            weapon.top = false;
            weapon.reload = 20.0;
            weapon.x = 3.0;
            weapon.y = 1.0;
            weapon.recoil = 1.0;
            weapon.shoot_shots = 2;
            weapon.shoot_shot_delay = 4.0;
            weapon.shoot_sound = "shootAlpha".into();
            weapon.bullet = "beta_laser_bolt".into();
            u.weapons.push(weapon);
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
            u.abilities
                .push("ShieldArcAbility:45:0.75:2500:480:82:0:0:-20:false:8:1".into());
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
            u.abilities
                .push("MoveEffectAbility:0:-7:4:missileTrailShort:true".into());
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
            u.abilities
                .push("SuppressionFieldAbility:480:90:200:0:1:true:13".into());
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
            u.abilities
                .push("SuppressionFieldAbility:900:90:320:0:10:true:13".into());
            u.abilities
                .push("SuppressionFieldAbility:90:90:200:10.75:-8:false:13".into());
            u.abilities
                .push("SuppressionFieldAbility:90:90:200:-10.75:-8:false:13".into());
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
            u.abilities.push("SpawnDeathAbility:renale:5:11".into());
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
    unit.ensure_default_wreck_regions();
    apply_default_commands(unit);
}

fn apply_default_commands(unit: &mut UnitType) {
    if unit.commands.is_empty() {
        push_command_once(&mut unit.commands, "move");

        if unit.allowed_in_payloads {
            push_command_once(&mut unit.commands, "enterPayload");
        }

        if unit.can_boost {
            if unit.build_speed > 0.0 {
                push_command_once(&mut unit.commands, "rebuild");
                push_command_once(&mut unit.commands, "assist");
            }
            if unit.mine_tier > 0 {
                push_command_once(&mut unit.commands, "mine");
            }
        }

        if unit.flying {
            if unit.can_heal {
                push_command_once(&mut unit.commands, "repair");
            }
            if unit.build_speed > 0.0 {
                push_command_once(&mut unit.commands, "rebuild");
                push_command_once(&mut unit.commands, "assist");
            }
            if unit.mine_tier > 0 {
                push_command_once(&mut unit.commands, "mine");
            }
            if unit_has_payload_command_trait(unit) {
                push_command_once(&mut unit.commands, "loadUnits");
                push_command_once(&mut unit.commands, "loadBlocks");
                push_command_once(&mut unit.commands, "unloadPayload");
                push_command_once(&mut unit.commands, "loopPayload");
            }
        }
    }

    if let Some(default_command) = unit.default_command.clone() {
        push_command_once(&mut unit.commands, &default_command);
    } else if let Some(first) = unit.commands.first() {
        unit.default_command = Some(first.clone());
    }
}

fn push_command_once(commands: &mut Vec<String>, command: &str) {
    if !commands.iter().any(|existing| existing == command) {
        commands.push(command.to_string());
    }
}

fn unit_has_payload_command_trait(unit: &UnitType) -> bool {
    matches!(
        unit.name(),
        "mega" | "quad" | "oct" | "evoke" | "incite" | "emanate" | "quell" | "disrupt"
    )
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
    use crate::mindustry::{
        content::{blocks::BulletKind, bullets},
        ctype::{Content, ContentType},
    };

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
    fn vanilla_units_load_wreck_regions_like_java_unit_type_load() {
        let units = load();
        let crawler = by_name(&units, "crawler");

        assert_eq!(
            crawler
                .wreck_regions
                .iter()
                .map(|region| region.name.as_str())
                .collect::<Vec<_>>(),
            vec!["crawler-wreck0", "crawler-wreck1", "crawler-wreck2"]
        );
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
        assert_eq!(alpha.default_command.as_deref(), Some("move"));

        let mono = by_name(&units, "mono");
        assert_eq!(mono.default_command.as_deref(), Some("mine"));
        assert_eq!(
            mono.commands.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["move", "enterPayload", "mine"]
        );
        let poly = by_name(&units, "poly");
        assert_eq!(poly.default_command.as_deref(), Some("rebuild"));
        assert_eq!(
            poly.commands.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["move", "enterPayload", "rebuild", "assist", "mine"]
        );
        let mega = by_name(&units, "mega");
        assert_eq!(mega.default_command.as_deref(), Some("repair"));
        assert!(mega.commands.iter().any(|command| command == "repair"));
        for name in [
            "mega", "quad", "oct", "evoke", "incite", "emanate", "quell", "disrupt",
        ] {
            let unit = by_name(&units, name);
            for command in ["loadUnits", "loadBlocks", "unloadPayload", "loopPayload"] {
                assert!(
                    unit.commands.iter().any(|candidate| candidate == command),
                    "{name} should include Java Payloadc command {command}"
                );
            }
        }
        let evoke = by_name(&units, "evoke");
        assert_eq!(evoke.default_command.as_deref(), Some("move"));
    }

    #[test]
    fn dagger_large_weapon_uses_casing1_and_basic_bullet_profile() {
        let units = load();
        let dagger = by_name(&units, "dagger");
        assert_eq!(dagger.weapons.len(), 1);

        let weapon = &dagger.weapons[0];
        assert_eq!(weapon.name, "large-weapon");
        assert_eq!(weapon.reload, 13.0);
        assert_eq!(weapon.x, 4.0);
        assert_eq!(weapon.y, 2.0);
        assert!(!weapon.top);
        assert_eq!(weapon.eject_effect, "casing1");
        assert_eq!(weapon.bullet, "dagger_basic");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing dagger weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Basic);
        assert_eq!(bullet.spec.speed, 2.5);
        assert_eq!(bullet.spec.damage, 9.0);
        assert_eq!(bullet.spec.width, 7.0);
        assert_eq!(bullet.spec.height, 9.0);
        assert_eq!(bullet.spec.lifetime, 60.0);
    }

    #[test]
    fn mace_flamethrower_uses_flame_bullet_profile() {
        let units = load();
        let mace = by_name(&units, "mace");
        assert_eq!(mace.weapons.len(), 1);

        let weapon = &mace.weapons[0];
        assert_eq!(weapon.name, "flamethrower");
        assert!(!weapon.top);
        assert_eq!(weapon.shoot_sound, "shootFlame");
        assert_eq!(weapon.shoot_y, 2.0);
        assert_eq!(weapon.reload, 22.0);
        assert_eq!(weapon.recoil, 1.0);
        assert_eq!(weapon.eject_effect, "none");
        assert_eq!(weapon.bullet, "mace_flame");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing mace weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Generic);
        assert_eq!(bullet.spec.speed, 4.2);
        assert_eq!(bullet.spec.damage, 74.0);
        assert_eq!(bullet.spec.status, "burning");
        assert!(bullet.spec.pierce);
        assert!(bullet.spec.pierce_building);
        assert_eq!(bullet.spec.pierce_cap, 2);
        assert!(!bullet.spec.keep_velocity);
        assert!(!bullet.spec.hittable);
    }

    #[test]
    fn quasar_beam_weapon_uses_laser_bullet_profile() {
        let units = load();
        let quasar = by_name(&units, "quasar");
        assert_eq!(quasar.weapons.len(), 1);

        let weapon = &quasar.weapons[0];
        assert_eq!(weapon.name, "beam-weapon");
        assert!(!weapon.top);
        assert_eq!(weapon.shake, 2.0);
        assert_eq!(weapon.shoot_y, 4.0);
        assert_eq!(weapon.x, 6.5);
        assert_eq!(weapon.reload, 55.0);
        assert_eq!(weapon.recoil, 4.0);
        assert_eq!(weapon.shoot_sound, "shootLancer");
        assert_eq!(weapon.bullet, "quasar_beam");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing quasar weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Laser);
        assert_eq!(bullet.spec.damage, 45.0);
        assert_eq!(bullet.spec.heal_percent, 10.0);
        assert!(bullet.spec.collides_team);
        assert_eq!(bullet.spec.length, 150.0);
    }

    #[test]
    fn beta_small_mount_weapon_uses_laser_bolt_profile() {
        let units = load();
        let beta = by_name(&units, "beta");
        assert_eq!(beta.drag, 0.05);
        assert_eq!(beta.accel, 0.1);
        assert_eq!(beta.fog_radius, 0.0);
        assert!(!beta.target_buildings_mobile);
        assert_eq!(beta.weapons.len(), 1);

        let weapon = &beta.weapons[0];
        assert_eq!(weapon.name, "small-mount-weapon");
        assert!(!weapon.top);
        assert_eq!(weapon.reload, 20.0);
        assert_eq!(weapon.x, 3.0);
        assert_eq!(weapon.y, 1.0);
        assert_eq!(weapon.recoil, 1.0);
        assert_eq!(weapon.shoot_shots, 2);
        assert_eq!(weapon.shoot_shot_delay, 4.0);
        assert_eq!(weapon.shoot_sound, "shootAlpha");
        assert_eq!(weapon.bullet, "beta_laser_bolt");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing beta weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::LaserBolt);
        assert_eq!(bullet.spec.speed, 3.0);
        assert_eq!(bullet.spec.damage, 11.0);
        assert!(bullet.spec.scale_keep_velocity);
        assert_eq!(bullet.spec.homing_power, 0.03);
        assert_eq!(bullet.spec.building_damage_multiplier, 0.01);
    }

    #[test]
    fn nova_heal_weapon_uses_healing_laser_bolt_profile() {
        let units = load();
        let nova = by_name(&units, "nova");
        assert_eq!(nova.weapons.len(), 1);

        let weapon = &nova.weapons[0];
        assert_eq!(weapon.name, "heal-weapon");
        assert!(!weapon.top);
        assert_eq!(weapon.shoot_y, 2.0);
        assert_eq!(weapon.reload, 24.0);
        assert_eq!(weapon.x, 4.5);
        assert!(!weapon.alternate);
        assert_eq!(weapon.eject_effect, "none");
        assert_eq!(weapon.recoil, 2.0);
        assert_eq!(weapon.shoot_sound, "shootLaser");
        assert_eq!(weapon.bullet, "nova_heal_bolt");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing nova weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::LaserBolt);
        assert_eq!(bullet.spec.speed, 5.2);
        assert_eq!(bullet.spec.damage, 13.0);
        assert_eq!(bullet.spec.heal_percent, 5.0);
        assert!(bullet.spec.collides_team);
        assert_eq!(bullet.spec.back_color, "heal");
        assert_eq!(bullet.spec.front_color, "white");
    }

    #[test]
    fn fortress_artillery_weapon_uses_artillery_bullet_profile() {
        let units = load();
        let fortress = by_name(&units, "fortress");
        assert_eq!(fortress.weapons.len(), 1);

        let weapon = &fortress.weapons[0];
        assert_eq!(weapon.name, "artillery");
        assert!(!weapon.top);
        assert_eq!(weapon.y, 1.0);
        assert_eq!(weapon.x, 9.0);
        assert_eq!(weapon.reload, 60.0);
        assert_eq!(weapon.recoil, 4.0);
        assert_eq!(weapon.shake, 2.0);
        assert_eq!(weapon.eject_effect, "casing2");
        assert_eq!(weapon.shoot_sound, "shootArtillery");
        assert_eq!(weapon.bullet, "fortress_artillery");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing fortress weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Artillery);
        assert_eq!(bullet.spec.speed, 2.0);
        assert_eq!(bullet.spec.damage, 20.0);
        assert_eq!(bullet.spec.max_range, 240.0);
        assert_eq!(bullet.spec.splash_damage, 80.0);
        assert_eq!(bullet.spec.splash_damage_radius, 35.0);
    }

    #[test]
    fn pulsar_heal_shotgun_uses_nested_lightning_profile() {
        let units = load();
        let pulsar = by_name(&units, "pulsar");
        assert_eq!(pulsar.weapons.len(), 1);

        let weapon = &pulsar.weapons[0];
        assert_eq!(weapon.name, "heal-shotgun-weapon");
        assert!(!weapon.top);
        assert_eq!(weapon.x, 5.0);
        assert_eq!(weapon.shake, 2.2);
        assert_eq!(weapon.y, 0.5);
        assert_eq!(weapon.shoot_y, 2.5);
        assert_eq!(weapon.reload, 36.0);
        assert_eq!(weapon.inaccuracy, 35.0);
        assert_eq!(weapon.shoot_shots, 3);
        assert_eq!(weapon.shoot_shot_delay, 0.5);
        assert_eq!(weapon.eject_effect, "none");
        assert_eq!(weapon.recoil, 2.5);
        assert_eq!(weapon.shoot_sound, "shootPulsar");
        assert_eq!(weapon.bullet, "pulsar_heal_lightning");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing pulsar weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Lightning);
        assert_eq!(bullet.spec.damage, 15.0);
        assert_eq!(bullet.spec.heal_percent, 2.0);
        assert!(bullet.spec.lightning_type.is_some());
    }

    #[test]
    fn reign_weapon_uses_frag_shell_profile() {
        let units = load();
        let reign = by_name(&units, "reign");
        assert_eq!(reign.weapons.len(), 1);

        let weapon = &reign.weapons[0];
        assert_eq!(weapon.name, "reign-weapon");
        assert!(!weapon.top);
        assert_eq!(weapon.y, 1.0);
        assert_eq!(weapon.x, 21.5);
        assert_eq!(weapon.shoot_y, 11.0);
        assert_eq!(weapon.reload, 9.0);
        assert_eq!(weapon.recoil, 5.0);
        assert_eq!(weapon.shake, 2.0);
        assert_eq!(weapon.eject_effect, "casing4");
        assert_eq!(weapon.shoot_sound, "shootReign");
        assert_eq!(weapon.bullet, "reign_shell");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing reign weapon bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Basic);
        assert_eq!(bullet.spec.damage, 80.0);
        assert_eq!(bullet.spec.frag_bullets, 3);
        assert!(bullet.spec.frag_bullet.is_some());
    }

    #[test]
    fn scepter_weapons_use_main_and_shared_mount_bullets() {
        let units = load();
        let scepter = by_name(&units, "scepter");
        assert_eq!(scepter.weapons.len(), 3);

        let main = &scepter.weapons[0];
        assert_eq!(main.name, "scepter-weapon");
        assert!(!main.top);
        assert_eq!(main.y, 1.0);
        assert_eq!(main.x, 16.0);
        assert_eq!(main.shoot_y, 8.0);
        assert_eq!(main.reload, 45.0);
        assert_eq!(main.recoil, 5.0);
        assert_eq!(main.shake, 2.0);
        assert_eq!(main.eject_effect, "casing3");
        assert_eq!(main.shoot_sound, "shootScepter");
        assert_eq!(main.shoot_sound_volume, 0.95);
        assert_eq!(main.inaccuracy, 3.0);
        assert_eq!(main.shoot_shots, 3);
        assert_eq!(main.shoot_shot_delay, 4.0);
        assert_eq!(main.bullet, "scepter_bullet");

        for (mount, reload, y) in [
            (&scepter.weapons[1], 12.0_f32, 6.0_f32),
            (&scepter.weapons[2], 15.0_f32, -7.0_f32),
        ] {
            assert_eq!(mount.name, "scepter-mount");
            assert_eq!(mount.reload, reload);
            assert_eq!(mount.x, 8.5);
            assert_eq!(mount.y, y);
            assert!(mount.rotate);
            assert_eq!(mount.eject_effect, "casing1");
            assert_eq!(mount.bullet, "scepter_small_bullet");
            assert_eq!(mount.shoot_sound, "shootScepterSecondary");
            assert_eq!(mount.rotate_speed, 3.0);
        }

        let bullets = bullets::load();
        let main_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == main.bullet)
            .unwrap_or_else(|| panic!("missing scepter main bullet {}", main.bullet));
        assert_eq!(main_bullet.spec.kind, BulletKind::Basic);
        assert_eq!(main_bullet.spec.bullet_interval, 4.0);
        assert!(main_bullet.spec.interval_bullet.is_some());

        let small_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == "scepter_small_bullet")
            .expect("missing scepter shared small bullet");
        assert_eq!(small_bullet.spec.speed, 12.0);
        assert_eq!(small_bullet.spec.damage, 20.0);
        assert_eq!(small_bullet.spec.shrink_interp, "slope");
    }

    #[test]
    fn vela_weapons_use_continuous_laser_and_repair_beam_profiles() {
        let units = load();
        let vela = by_name(&units, "vela");
        assert_eq!(vela.weapons.len(), 2);

        let laser = &vela.weapons[0];
        assert_eq!(laser.name, "vela-weapon");
        assert!(!laser.mirror);
        assert!(!laser.top);
        assert_eq!(laser.shake, 4.0);
        assert_eq!(laser.shoot_y, 14.0);
        assert_eq!(laser.x, 0.0);
        assert_eq!(laser.y, 0.0);
        assert_eq!(laser.shoot_first_shot_delay, 39.0);
        assert!(laser.parentize_effects);
        assert_eq!(laser.reload, 155.0);
        assert_eq!(laser.recoil, 0.0);
        assert_eq!(laser.charge_sound, "chargeVela");
        assert_eq!(laser.shoot_sound, "beamPlasma");
        assert_eq!(laser.initial_shoot_sound, "shootBeamPlasma");
        assert!(laser.continuous);
        assert_eq!(laser.cooldown_time, 200.0);
        assert_eq!(laser.shoot_status, "slow");
        assert_eq!(laser.shoot_status_duration, 199.0);
        assert_eq!(laser.bullet, "vela_continuous_laser");

        let repair = &vela.weapons[1];
        assert_eq!(repair.name, "repair-beam-weapon-center-large");
        assert_eq!(repair.x, 44.0 / 4.0);
        assert_eq!(repair.y, -30.0 / 4.0);
        assert_eq!(repair.shoot_y, 6.0);
        assert_eq!(repair.beam_width, 0.8);
        assert_eq!(repair.repair_speed, 1.4);
        assert_eq!(repair.reload, 1.0);
        assert!(!repair.predict_target);
        assert!(repair.auto_target);
        assert!(!repair.controllable);
        assert!(repair.rotate);
        assert_eq!(repair.mount_type, "HealBeamMount");
        assert_eq!(repair.recoil, 0.0);
        assert!(repair.no_attack);
        assert!(!repair.use_attack_range);
        assert_eq!(repair.active_sound, "beamHeal");
        assert_eq!(repair.bullet, "vela_repair_range");

        let bullets = bullets::load();
        let laser_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == laser.bullet)
            .unwrap_or_else(|| panic!("missing vela laser bullet {}", laser.bullet));
        assert_eq!(laser_bullet.spec.kind, BulletKind::ContinuousLaser);
        assert_eq!(laser_bullet.spec.heal_percent, 1.0);
        assert!(laser_bullet.spec.collides_team);

        let repair_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == repair.bullet)
            .unwrap_or_else(|| panic!("missing vela repair bullet {}", repair.bullet));
        assert_eq!(repair_bullet.spec.max_range, 120.0);
    }

    #[test]
    fn corvus_weapon_uses_charged_laser_profile() {
        let units = load();
        let corvus = by_name(&units, "corvus");
        assert_eq!(corvus.weapons.len(), 1);

        let weapon = &corvus.weapons[0];
        assert_eq!(weapon.name, "corvus-weapon");
        assert_eq!(weapon.shoot_sound, "shootCorvus");
        assert_eq!(weapon.charge_sound, "chargeCorvus");
        assert_eq!(weapon.sound_pitch_min, 1.0);
        assert!(!weapon.top);
        assert!(!weapon.mirror);
        assert_eq!(weapon.shake, 14.0);
        assert_eq!(weapon.shoot_y, 5.0);
        assert_eq!(weapon.x, 0.0);
        assert_eq!(weapon.y, 0.0);
        assert_eq!(weapon.reload, 350.0);
        assert_eq!(weapon.recoil, 0.0);
        assert_eq!(weapon.cooldown_time, 350.0);
        assert_eq!(weapon.shoot_status_duration, 120.0);
        assert_eq!(weapon.shoot_status, "unmoving");
        assert_eq!(weapon.shoot_first_shot_delay, 80.0);
        assert!(weapon.parentize_effects);
        assert_eq!(weapon.bullet, "corvus_laser");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing corvus bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Laser);
        assert_eq!(bullet.spec.damage, 560.0);
        assert_eq!(bullet.spec.length, 460.0);
        assert_eq!(bullet.spec.heal_percent, 25.0);
        assert!(bullet.spec.collides_team);
    }

    #[test]
    fn crawler_death_weapon_uses_suicide_explosion_profile() {
        let units = load();
        let crawler = by_name(&units, "crawler");
        assert_eq!(crawler.mech_side_sway, 0.25);
        assert_eq!(crawler.range, 40.0);
        assert!(!crawler.target_under_blocks);
        assert_eq!(crawler.step_sound, "walkerStepTiny");
        assert_eq!(crawler.weapons.len(), 1);

        let weapon = &crawler.weapons[0];
        assert_eq!(weapon.name, "");
        assert!(weapon.shoot_on_death);
        assert_eq!(weapon.reload, 24.0);
        assert_eq!(weapon.shoot_cone, 180.0);
        assert_eq!(weapon.eject_effect, "none");
        assert_eq!(weapon.shoot_sound, "explosionCrawler");
        assert_eq!(weapon.shoot_sound_volume, 0.4);
        assert_eq!(weapon.x, 0.0);
        assert_eq!(weapon.shoot_y, 0.0);
        assert!(!weapon.mirror);
        assert_eq!(weapon.bullet, "crawler_explosion");
        assert!(weapon.bullet_kill_shooter);

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing crawler bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.range_override, 25.0);
        assert_eq!(bullet.spec.splash_damage, 80.0);
        assert!(bullet.spec.kill_shooter);
        assert!(!bullet.spec.collides);
    }

    #[test]
    fn atrax_weapon_uses_slag_liquid_bullet_profile() {
        let units = load();
        let atrax = by_name(&units, "atrax");
        assert_eq!(atrax.drag, 0.4);
        assert!(atrax.immunities.iter().any(|entry| entry == "burning"));
        assert!(atrax.immunities.iter().any(|entry| entry == "melting"));
        assert_eq!(atrax.step_sound, "walkerStepSmall");
        assert_eq!(atrax.shadow_elevation, 0.2);
        assert_eq!(atrax.ground_layer, LAYER_LEG_UNIT - 1.0);
        assert_eq!(atrax.weapons.len(), 1);

        let weapon = &atrax.weapons[0];
        assert_eq!(weapon.name, "atrax-weapon");
        assert!(!weapon.top);
        assert_eq!(weapon.shoot_y, 3.0);
        assert_eq!(weapon.reload, 9.0);
        assert_eq!(weapon.eject_effect, "none");
        assert_eq!(weapon.recoil, 1.0);
        assert_eq!(weapon.x, 7.0);
        assert_eq!(weapon.shoot_sound, "shootAtrax");
        assert_eq!(weapon.bullet, "atrax_slag");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing atrax bullet {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Liquid);
        assert_eq!(bullet.spec.liquid, "slag");
        assert_eq!(bullet.spec.damage, 13.0);
        assert_eq!(bullet.spec.speed, 2.5);
        assert!(!bullet.spec.collides_air);
    }

    #[test]
    fn spiroct_weapons_match_java_sap_profiles() {
        let units = load();
        let spiroct = by_name(&units, "spiroct");
        assert_eq!(spiroct.drag, 0.4);
        assert_eq!(spiroct.step_sound, "walkerStepSmall");
        assert_eq!(spiroct.shadow_elevation, 0.3);
        assert_eq!(spiroct.ground_layer, LAYER_LEG_UNIT);
        assert_eq!(spiroct.weapons.len(), 2);

        let primary = &spiroct.weapons[0];
        assert_eq!(primary.name, "spiroct-weapon");
        assert_eq!(primary.shoot_y, 4.0);
        assert_eq!(primary.reload, 14.0);
        assert_eq!(primary.eject_effect, "none");
        assert_eq!(primary.recoil, 2.0);
        assert!(primary.rotate);
        assert_eq!(primary.shoot_sound, "shootSap");
        assert_eq!(primary.x, 8.5);
        assert_eq!(primary.y, -1.5);
        assert_eq!(primary.bullet, "spiroct_sap");

        let mount = &spiroct.weapons[1];
        assert_eq!(mount.name, "mount-purple-weapon");
        assert_eq!(mount.reload, 18.0);
        assert!(mount.rotate);
        assert_eq!(mount.x, 4.0);
        assert_eq!(mount.y, 3.0);
        assert_eq!(mount.shoot_sound, "shootSap");
        assert_eq!(mount.bullet, "spiroct_mount_sap");
        assert_ne!(primary.bullet, mount.bullet);

        let bullets = bullets::load();
        let primary_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == primary.bullet)
            .unwrap_or_else(|| panic!("missing spiroct bullet {}", primary.bullet));
        let mount_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == mount.bullet)
            .unwrap_or_else(|| panic!("missing spiroct mount bullet {}", mount.bullet));

        assert_eq!(primary_bullet.spec.kind, BulletKind::Sap);
        assert_eq!(primary_bullet.spec.sap_strength, 0.5);
        assert_eq!(primary_bullet.spec.length, 75.0);
        assert_eq!(primary_bullet.spec.damage, 23.0);
        assert_eq!(mount_bullet.spec.kind, BulletKind::Sap);
        assert_eq!(mount_bullet.spec.sap_strength, 0.8);
        assert_eq!(mount_bullet.spec.length, 40.0);
        assert_eq!(mount_bullet.spec.damage, 18.0);
    }

    #[test]
    fn arkyid_weapons_match_java_sapper_and_artillery_profiles() {
        let units = load();
        let arkyid = by_name(&units, "arkyid");
        assert_eq!(arkyid.drag, 0.1);
        assert_eq!(arkyid.leg_pair_offset, 3.0);
        assert_eq!(arkyid.leg_extension, -15.0);
        assert_eq!(arkyid.leg_length_scl, 0.96);
        assert_eq!(arkyid.ripple_scale, 2.0);
        assert_eq!(arkyid.leg_speed, 0.2);
        assert_eq!(arkyid.step_sound, "walkerStep");
        assert_eq!(arkyid.leg_splash_damage, 32.0);
        assert_eq!(arkyid.leg_splash_range, 30.0);
        assert_eq!(arkyid.shadow_elevation, 0.65);
        assert_eq!(arkyid.weapons.len(), 4);

        for (index, (reload, x, y)) in [(9.0, 4.0, 8.0), (14.0, 9.0, 6.0), (22.0, 14.0, 0.0)]
            .into_iter()
            .enumerate()
        {
            let weapon = &arkyid.weapons[index];
            assert_eq!(weapon.name, "spiroct-weapon");
            assert_eq!(weapon.reload, reload);
            assert_eq!(weapon.x, x);
            assert_eq!(weapon.y, y);
            assert!(weapon.rotate);
            assert_eq!(weapon.bullet, "arkyid_sapper");
            assert_eq!(weapon.shoot_sound, "shootSap");
        }

        let artillery = &arkyid.weapons[3];
        assert_eq!(artillery.name, "large-purple-mount");
        assert_eq!(artillery.y, -7.0);
        assert_eq!(artillery.x, 9.0);
        assert_eq!(artillery.shoot_y, 7.0);
        assert_eq!(artillery.reload, 45.0);
        assert_eq!(artillery.shake, 3.0);
        assert_eq!(artillery.rotate_speed, 2.0);
        assert_eq!(artillery.eject_effect, "casing1");
        assert_eq!(artillery.shoot_sound, "shootArtillerySap");
        assert!(artillery.rotate);
        assert_eq!(artillery.shadow, 8.0);
        assert_eq!(artillery.recoil, 3.0);
        assert_eq!(artillery.shoot_pattern, "ShootSpread");
        assert_eq!(artillery.shoot_shots, 2);
        assert_eq!(artillery.shoot_spread, 17.0);
        assert_eq!(artillery.bullet, "arkyid_artillery_sap");

        let bullets = bullets::load();
        let sapper = bullets
            .iter()
            .find(|bullet| bullet.name() == "arkyid_sapper")
            .expect("missing arkyid_sapper");
        let artillery_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == "arkyid_artillery_sap")
            .expect("missing arkyid_artillery_sap");
        assert_eq!(sapper.spec.kind, BulletKind::Sap);
        assert_eq!(sapper.spec.sap_strength, 0.85);
        assert_eq!(artillery_bullet.spec.kind, BulletKind::Artillery);
        assert_eq!(artillery_bullet.spec.splash_damage, 65.0);
    }

    #[test]
    fn toxopid_weapons_match_java_shrapnel_and_cannon_profiles() {
        let units = load();
        let toxopid = by_name(&units, "toxopid");
        assert_eq!(toxopid.drag, 0.1);
        assert_eq!(toxopid.light_radius, 140.0);
        assert_eq!(toxopid.step_sound, "walkerStep");
        assert_eq!(toxopid.leg_pair_offset, 3.0);
        assert_eq!(toxopid.leg_extension, -20.0);
        assert_eq!(toxopid.leg_length_scl, 0.93);
        assert_eq!(toxopid.ripple_scale, 3.0);
        assert_eq!(toxopid.leg_speed, 0.19);
        assert_eq!(toxopid.leg_splash_damage, 80.0);
        assert_eq!(toxopid.leg_splash_range, 60.0);
        assert_eq!(toxopid.shadow_elevation, 0.95);
        assert_eq!(toxopid.weapons.len(), 2);

        let shrapnel = &toxopid.weapons[0];
        assert_eq!(shrapnel.name, "large-purple-mount");
        assert_eq!(shrapnel.y, -5.0);
        assert_eq!(shrapnel.x, 11.0);
        assert_eq!(shrapnel.shoot_y, 7.0);
        assert_eq!(shrapnel.reload, 30.0);
        assert_eq!(shrapnel.shake, 4.0);
        assert_eq!(shrapnel.rotate_speed, 2.0);
        assert_eq!(shrapnel.eject_effect, "casing1");
        assert_eq!(shrapnel.shoot_sound, "shootToxopidShotgun");
        assert_eq!(shrapnel.shoot_sound_volume, 0.8);
        assert!(shrapnel.rotate);
        assert_eq!(shrapnel.shadow, 12.0);
        assert_eq!(shrapnel.recoil, 3.0);
        assert_eq!(shrapnel.shoot_pattern, "ShootSpread");
        assert_eq!(shrapnel.shoot_shots, 2);
        assert_eq!(shrapnel.shoot_spread, 17.0);
        assert_eq!(shrapnel.bullet, "toxopid_shrapnel");

        let cannon = &toxopid.weapons[1];
        assert_eq!(cannon.name, "toxopid-cannon");
        assert_eq!(cannon.y, -14.0);
        assert_eq!(cannon.x, 0.0);
        assert_eq!(cannon.shoot_y, 22.0);
        assert!(!cannon.mirror);
        assert_eq!(cannon.reload, 210.0);
        assert_eq!(cannon.shake, 10.0);
        assert_eq!(cannon.recoil, 10.0);
        assert_eq!(cannon.rotate_speed, 1.0);
        assert_eq!(cannon.eject_effect, "casing3");
        assert_eq!(cannon.shoot_sound, "shootArtillerySapBig");
        assert!(cannon.rotate);
        assert_eq!(cannon.shadow, 30.0);
        assert_eq!(cannon.rotation_limit, 80.0);
        assert_eq!(cannon.bullet, "toxopid_cannon");

        let bullets = bullets::load();
        let shrapnel_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == shrapnel.bullet)
            .unwrap_or_else(|| panic!("missing toxopid shrapnel {}", shrapnel.bullet));
        let cannon_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == cannon.bullet)
            .unwrap_or_else(|| panic!("missing toxopid cannon {}", cannon.bullet));
        assert_eq!(shrapnel_bullet.spec.kind, BulletKind::Shrapnel);
        assert_eq!(shrapnel_bullet.spec.damage, 110.0);
        assert_eq!(cannon_bullet.spec.kind, BulletKind::Artillery);
        assert_eq!(cannon_bullet.spec.frag_bullets, 9);
        assert!(cannon_bullet.spec.frag_bullet.is_some());
    }

    #[test]
    fn flare_and_horizon_weapons_match_java_profiles() {
        let units = load();
        let flare = by_name(&units, "flare");
        assert_eq!(flare.accel, 0.08);
        assert_eq!(flare.drag, 0.04);
        assert_eq!(
            flare.target_flags,
            vec![Some("generator".to_string()), None]
        );
        assert!(flare.circle_target);
        assert!(!flare.omni_movement);
        assert_eq!(flare.circle_target_radius, 60.0);
        assert_eq!(flare.wreck_sound_volume, 0.7);
        assert_eq!(flare.move_sound, "loopThruster");
        assert_eq!(flare.move_sound_pitch_min, 0.3);
        assert_eq!(flare.move_sound_pitch_max, 1.5);
        assert_eq!(flare.move_sound_volume, 0.2);
        assert_eq!(flare.weapons.len(), 1);

        let flare_weapon = &flare.weapons[0];
        assert_eq!(flare_weapon.name, "");
        assert_eq!(flare_weapon.y, 1.0);
        assert_eq!(flare_weapon.x, 0.0);
        assert_eq!(flare_weapon.min_shoot_velocity, 2.0);
        assert_eq!(flare_weapon.shoot_cone, 10.0);
        assert_eq!(flare_weapon.reload, 80.0);
        assert_eq!(flare_weapon.shoot_shots, 3);
        assert_eq!(flare_weapon.shoot_shot_delay, 3.0);
        assert_eq!(flare_weapon.eject_effect, "casing1");
        assert!(!flare_weapon.mirror);
        assert_eq!(flare_weapon.bullet, "flare_basic");

        let horizon = by_name(&units, "horizon");
        assert_eq!(horizon.accel, 0.08);
        assert_eq!(horizon.drag, 0.03);
        assert_eq!(horizon.range, 140.0);
        assert!(!horizon.face_target);
        assert!(horizon.auto_drop_bombs);
        assert_eq!(
            horizon.target_flags,
            vec![Some("factory".to_string()), None]
        );
        assert!(horizon.circle_target);
        assert!(!horizon.omni_movement);
        assert_eq!(horizon.circle_target_radius, 40.0);
        assert_eq!(horizon.move_sound, "loopThruster");
        assert_eq!(horizon.move_sound_pitch_min, 0.6);
        assert_eq!(horizon.move_sound_volume, 0.4);
        assert_eq!(horizon.weapons.len(), 1);

        let horizon_weapon = &horizon.weapons[0];
        assert_eq!(horizon_weapon.name, "");
        assert_eq!(horizon_weapon.min_shoot_velocity, 1.0);
        assert_eq!(horizon_weapon.x, 3.0);
        assert_eq!(horizon_weapon.shoot_y, 0.0);
        assert_eq!(horizon_weapon.reload, 12.0);
        assert_eq!(horizon_weapon.shoot_cone, 180.0);
        assert_eq!(horizon_weapon.eject_effect, "none");
        assert_eq!(horizon_weapon.inaccuracy, 15.0);
        assert!(horizon_weapon.ignore_rotation);
        assert_eq!(horizon_weapon.shoot_sound, "shootHorizon");
        assert_eq!(horizon_weapon.sound_pitch_max, 1.2);
        assert_eq!(horizon_weapon.bullet, "horizon_bomb");

        let bullets = bullets::load();
        assert_eq!(
            bullets
                .iter()
                .find(|bullet| bullet.name() == flare_weapon.bullet)
                .expect("missing flare_basic")
                .spec
                .kind,
            BulletKind::Basic
        );
        assert_eq!(
            bullets
                .iter()
                .find(|bullet| bullet.name() == horizon_weapon.bullet)
                .expect("missing horizon_bomb")
                .spec
                .kind,
            BulletKind::Bomb
        );
    }

    #[test]
    fn zenith_weapon_uses_java_missile_profile() {
        let units = load();
        let zenith = by_name(&units, "zenith");
        assert_eq!(zenith.accel, 0.04);
        assert_eq!(zenith.drag, 0.016);
        assert_eq!(zenith.range, 140.0);
        assert!(zenith.force_multi_target);
        assert_eq!(
            zenith.target_flags,
            vec![
                Some("launchPad".to_string()),
                Some("storage".to_string()),
                Some("battery".to_string()),
                None,
            ]
        );
        assert_eq!(zenith.weapons.len(), 1);

        let weapon = &zenith.weapons[0];
        assert_eq!(weapon.name, "zenith-missiles");
        assert_eq!(weapon.reload, 40.0);
        assert_eq!(weapon.x, 7.0);
        assert!(weapon.rotate);
        assert_eq!(weapon.shake, 1.0);
        assert_eq!(weapon.shoot_shots, 2);
        assert_eq!(weapon.inaccuracy, 5.0);
        assert_eq!(weapon.velocity_rnd, 0.2);
        assert_eq!(weapon.shoot_sound, "shootMissileLong");
        assert_eq!(weapon.bullet, "zenith_missile");

        let bullets = bullets::load();
        let bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .unwrap_or_else(|| panic!("missing zenith missile {}", weapon.bullet));
        assert_eq!(bullet.spec.kind, BulletKind::Missile);
        assert_eq!(bullet.spec.splash_damage, 15.0);
        assert_eq!(bullet.spec.weave_scale, 6.0);
        assert_eq!(bullet.spec.weave_mag, 1.0);
    }

    #[test]
    fn antumbra_weapons_match_java_mount_profiles() {
        let units = load();
        let antumbra = by_name(&units, "antumbra");
        assert_eq!(antumbra.speed, 0.8);
        assert_eq!(antumbra.accel, 0.04);
        assert_eq!(antumbra.drag, 0.04);
        assert_eq!(antumbra.rotate_speed, 1.9);
        assert!(antumbra.flying);
        assert!(antumbra.low_altitude);
        assert_eq!(antumbra.health, 7200.0);
        assert_eq!(antumbra.armor, 9.0);
        assert_eq!(antumbra.engine_offset, 21.0);
        assert_eq!(antumbra.engine_size, 5.3);
        assert_eq!(antumbra.hit_size, 46.0);
        assert_eq!(
            antumbra.target_flags,
            vec![
                Some("generator".to_string()),
                Some("core".to_string()),
                None
            ]
        );
        assert_eq!(antumbra.loop_sound, "loopHover");
        assert_eq!(antumbra.weapons.len(), 3);

        let front_missile = &antumbra.weapons[0];
        assert_eq!(front_missile.name, "missiles-mount");
        assert_eq!(front_missile.y, 8.0);
        assert_eq!(front_missile.x, 17.0);
        assert_eq!(front_missile.reload, 20.0);
        assert_eq!(front_missile.eject_effect, "casing1");
        assert_eq!(front_missile.rotate_speed, 8.0);
        assert_eq!(front_missile.bullet, "antumbra_missile");
        assert_eq!(front_missile.shoot_sound, "shootMissile");
        assert!(front_missile.rotate);
        assert_eq!(front_missile.shadow, 6.0);

        let rear_missile = &antumbra.weapons[1];
        assert_eq!(rear_missile.name, "missiles-mount");
        assert_eq!(rear_missile.y, -8.0);
        assert_eq!(rear_missile.x, 17.0);
        assert_eq!(rear_missile.reload, 35.0);
        assert_eq!(rear_missile.eject_effect, "casing1");
        assert_eq!(rear_missile.rotate_speed, 8.0);
        assert_eq!(rear_missile.bullet, "antumbra_missile");
        assert_eq!(rear_missile.shoot_sound, "shootMissile");
        assert!(rear_missile.rotate);
        assert_eq!(rear_missile.shadow, 6.0);

        let large_mount = &antumbra.weapons[2];
        assert_eq!(large_mount.name, "large-bullet-mount");
        assert_eq!(large_mount.y, 2.0);
        assert_eq!(large_mount.x, 10.0);
        assert_eq!(large_mount.shoot_y, 10.0);
        assert_eq!(large_mount.reload, 12.0);
        assert_eq!(large_mount.shake, 1.0);
        assert_eq!(large_mount.rotate_speed, 2.0);
        assert_eq!(large_mount.eject_effect, "casing1");
        assert_eq!(large_mount.shoot_sound, "shootSpectre");
        assert!(large_mount.rotate);
        assert_eq!(large_mount.shadow, 8.0);
        assert_eq!(large_mount.bullet, "antumbra_large_bullet");

        let bullets = bullets::load();
        let missile = bullets
            .iter()
            .find(|bullet| bullet.name() == front_missile.bullet)
            .expect("missing antumbra_missile");
        assert_eq!(missile.spec.kind, BulletKind::Missile);
        assert_eq!(missile.spec.status, "blasted");
        let large = bullets
            .iter()
            .find(|bullet| bullet.name() == large_mount.bullet)
            .expect("missing antumbra_large_bullet");
        assert_eq!(large.spec.kind, BulletKind::Basic);
        assert_eq!(large.spec.shoot_effect, "shootBig");
    }

    #[test]
    fn eclipse_weapons_match_java_mount_profiles() {
        let units = load();
        let eclipse = by_name(&units, "eclipse");
        assert_eq!(eclipse.speed, 0.54);
        assert_eq!(eclipse.accel, 0.04);
        assert_eq!(eclipse.drag, 0.04);
        assert_eq!(eclipse.rotate_speed, 1.0);
        assert!(eclipse.flying);
        assert!(eclipse.low_altitude);
        assert_eq!(eclipse.health, 22000.0);
        assert_eq!(eclipse.engine_offset, 38.0);
        assert_eq!(eclipse.engine_size, 7.3);
        assert_eq!(eclipse.hit_size, 58.0);
        assert_eq!(eclipse.armor, 13.0);
        assert_eq!(
            eclipse.target_flags,
            vec![
                Some("reactor".to_string()),
                Some("battery".to_string()),
                Some("core".to_string()),
                None,
            ]
        );
        assert_eq!(eclipse.loop_sound, "loopHover");
        assert_eq!(eclipse.weapons.len(), 3);

        let laser = &eclipse.weapons[0];
        assert_eq!(laser.name, "large-laser-mount");
        assert_eq!(laser.shake, 4.0);
        assert_eq!(laser.shoot_y, 9.0);
        assert_eq!(laser.x, 18.0);
        assert_eq!(laser.y, 5.0);
        assert_eq!(laser.rotate_speed, 2.0);
        assert_eq!(laser.reload, 45.0);
        assert_eq!(laser.recoil, 4.0);
        assert_eq!(laser.shoot_sound, "shootEclipse");
        assert_eq!(laser.shadow, 20.0);
        assert!(laser.rotate);
        assert_eq!(laser.bullet, "eclipse_laser");

        let top_artillery = &eclipse.weapons[1];
        assert_eq!(top_artillery.name, "large-artillery");
        assert_eq!(top_artillery.x, 11.0);
        assert_eq!(top_artillery.y, 27.0);
        assert_eq!(top_artillery.rotate_speed, 2.0);
        assert_eq!(top_artillery.reload, 9.0);
        assert_eq!(top_artillery.shoot_sound, "shootCyclone");
        assert_eq!(top_artillery.shadow, 7.0);
        assert!(top_artillery.rotate);
        assert_eq!(top_artillery.recoil, 0.5);
        assert_eq!(top_artillery.shoot_y, 7.25);
        assert_eq!(top_artillery.bullet, "eclipse_flak");

        let lower_artillery = &eclipse.weapons[2];
        assert_eq!(lower_artillery.name, "large-artillery");
        assert_eq!(lower_artillery.y, -13.0);
        assert_eq!(lower_artillery.x, 20.0);
        assert_eq!(lower_artillery.reload, 12.0);
        assert_eq!(lower_artillery.eject_effect, "casing1");
        assert_eq!(lower_artillery.rotate_speed, 7.0);
        assert_eq!(lower_artillery.shake, 1.0);
        assert_eq!(lower_artillery.shoot_sound, "shootCyclone");
        assert!(lower_artillery.rotate);
        assert_eq!(lower_artillery.shadow, 12.0);
        assert_eq!(lower_artillery.shoot_y, 7.25);
        assert_eq!(lower_artillery.bullet, "eclipse_flak");

        let bullets = bullets::load();
        let laser_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == laser.bullet)
            .expect("missing eclipse_laser");
        assert_eq!(laser_bullet.spec.kind, BulletKind::Laser);
        assert_eq!(laser_bullet.spec.damage, 115.0);
        let flak_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == top_artillery.bullet)
            .expect("missing eclipse_flak");
        assert_eq!(flak_bullet.spec.kind, BulletKind::Flak);
        assert_eq!(flak_bullet.spec.splash_damage, 65.0);
    }

    #[test]
    fn mono_and_poly_support_profiles_match_java() {
        let units = load();

        let mono = by_name(&units, "mono");
        assert_eq!(mono.default_command.as_deref(), Some("mine"));
        assert!(mono.flying);
        assert_eq!(mono.drag, 0.06);
        assert_eq!(mono.accel, 0.12);
        assert_eq!(mono.speed, 1.5);
        assert_eq!(mono.health, 100.0);
        assert_eq!(mono.engine_size, 1.8);
        assert_eq!(mono.engine_offset, 5.7);
        assert_eq!(mono.range, 50.0);
        assert!(!mono.is_enemy);
        assert!(!mono.control_select_global);
        assert_eq!(mono.wreck_sound_volume, 0.7);
        assert_eq!(mono.death_sound_volume, 0.7);
        assert_eq!(mono.mine_tier, 1);
        assert_eq!(mono.mine_speed, 2.5);
        assert!(mono.weapons.is_empty());

        let poly = by_name(&units, "poly");
        assert_eq!(poly.default_command.as_deref(), Some("rebuild"));
        assert!(poly.flying);
        assert_eq!(poly.drag, 0.05);
        assert_eq!(poly.speed, 2.6);
        assert_eq!(poly.rotate_speed, 15.0);
        assert_eq!(poly.accel, 0.1);
        assert_eq!(poly.range, 130.0);
        assert_eq!(poly.health, 400.0);
        assert_eq!(poly.build_speed, 0.5);
        assert_eq!(poly.engine_offset, 6.5);
        assert_eq!(poly.hit_size, 9.0);
        assert!(poly.low_altitude);
        assert_eq!(poly.mine_tier, 2);
        assert_eq!(poly.mine_speed, 3.5);
        assert_eq!(poly.wreck_sound_volume, 0.9);
        assert!(poly
            .abilities
            .iter()
            .any(|entry| entry == "RepairFieldAbility:5:480:50"));
        assert_eq!(poly.weapons.len(), 1);

        let weapon = &poly.weapons[0];
        assert_eq!(weapon.name, "poly-weapon");
        assert!(!weapon.top);
        assert_eq!(weapon.y, -2.5);
        assert_eq!(weapon.x, 3.75);
        assert_eq!(weapon.reload, 30.0);
        assert_eq!(weapon.eject_effect, "none");
        assert_eq!(weapon.recoil, 2.0);
        assert_eq!(weapon.shoot_sound, "shootMissilePlasmaShort");
        assert_eq!(weapon.velocity_rnd, 0.5);
        assert_eq!(weapon.inaccuracy, 15.0);
        assert!(weapon.alternate);
        assert_eq!(weapon.bullet, "poly_missile");

        let bullets = bullets::load();
        let missile = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .expect("missing poly_missile");
        assert_eq!(missile.spec.kind, BulletKind::Missile);
        assert_eq!(missile.spec.heal_percent, 5.5);
        assert!(missile.spec.collides_team);
    }

    #[test]
    fn mega_support_profile_matches_java_heal_mounts() {
        let units = load();
        let mega = by_name(&units, "mega");

        assert_eq!(mega.default_command.as_deref(), Some("repair"));
        assert_eq!(mega.mine_tier, 3);
        assert_eq!(mega.mine_speed, 4.0);
        assert_eq!(mega.health, 460.0);
        assert_eq!(mega.armor, 3.0);
        assert_eq!(mega.speed, 2.5);
        assert_eq!(mega.accel, 0.06);
        assert_eq!(mega.drag, 0.017);
        assert!(mega.low_altitude);
        assert!(mega.flying);
        assert_eq!(mega.engine_offset, 10.5);
        assert!(!mega.face_target);
        assert_eq!(mega.hit_size, 16.05);
        assert_eq!(mega.engine_size, 3.0);
        assert_eq!(mega.payload_capacity, 4.0 * super_tile_payload());
        assert_eq!(mega.build_speed, 2.6);
        assert!(!mega.is_enemy);
        assert_eq!(mega.weapons.len(), 2);

        let large = &mega.weapons[0];
        assert_eq!(large.name, "heal-weapon-mount");
        assert_eq!(large.shoot_sound, "shootLaser");
        assert_eq!(large.reload, 24.0);
        assert_eq!(large.x, 8.0);
        assert_eq!(large.y, -6.0);
        assert!(large.rotate);
        assert_eq!(large.bullet, "mega_heal_bolt_large");

        let small = &mega.weapons[1];
        assert_eq!(small.name, "heal-weapon-mount");
        assert_eq!(small.shoot_sound, "shootLaser");
        assert_eq!(small.reload, 15.0);
        assert_eq!(small.x, 4.0);
        assert_eq!(small.y, 5.0);
        assert!(small.rotate);
        assert_eq!(small.bullet, "mega_heal_bolt_small");

        let bullets = bullets::load();
        let large_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == large.bullet)
            .expect("missing mega_heal_bolt_large");
        assert_eq!(large_bullet.spec.kind, BulletKind::LaserBolt);
        assert_eq!(large_bullet.spec.damage, 10.0);
        assert_eq!(large_bullet.spec.heal_percent, 5.5);
        let small_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == small.bullet)
            .expect("missing mega_heal_bolt_small");
        assert_eq!(small_bullet.spec.kind, BulletKind::LaserBolt);
        assert_eq!(small_bullet.spec.damage, 8.0);
        assert_eq!(small_bullet.spec.heal_percent, 3.0);
    }

    #[test]
    fn quad_bomber_profile_matches_java() {
        let units = load();
        let quad = by_name(&units, "quad");

        assert_eq!(quad.armor, 8.0);
        assert_eq!(quad.health, 6000.0);
        assert_eq!(quad.speed, 1.2);
        assert_eq!(quad.rotate_speed, 2.0);
        assert_eq!(quad.accel, 0.05);
        assert_eq!(quad.drag, 0.017);
        assert!(!quad.low_altitude);
        assert!(quad.flying);
        assert!(quad.auto_drop_bombs);
        assert!(quad.circle_target);
        assert_eq!(quad.engine_offset, 13.0);
        assert_eq!(quad.engine_size, 7.0);
        assert!(!quad.face_target);
        assert_eq!(quad.hit_size, 36.0);
        assert_eq!(quad.payload_capacity, 9.0 * super_tile_payload());
        assert_eq!(quad.build_speed, 2.5);
        assert_eq!(quad.build_beam_offset, 23.0);
        assert_eq!(quad.range, 140.0);
        assert!(!quad.target_air);
        assert_eq!(
            quad.target_flags,
            vec![
                Some("battery".to_string()),
                Some("factory".to_string()),
                None
            ]
        );
        assert_eq!(quad.loop_sound, "loopHover");
        assert_eq!(quad.weapons.len(), 1);

        let weapon = &quad.weapons[0];
        assert_eq!(weapon.name, "");
        assert_eq!(weapon.x, 0.0);
        assert_eq!(weapon.y, 0.0);
        assert!(!weapon.mirror);
        assert_eq!(weapon.reload, 55.0);
        assert_eq!(weapon.min_shoot_velocity, 0.01);
        assert_eq!(weapon.sound_pitch_min, 1.0);
        assert_eq!(weapon.shoot_sound, "shootQuad");
        assert_eq!(weapon.bullet, "quad_bomb");

        let bullets = bullets::load();
        let bomb = bullets
            .iter()
            .find(|bullet| bullet.name() == weapon.bullet)
            .expect("missing quad_bomb");
        assert_eq!(bomb.spec.kind, BulletKind::Basic);
        assert_eq!(bomb.spec.sprite, "large-bomb");
        assert_eq!(bomb.spec.splash_damage, 220.0);
        assert_eq!(bomb.spec.damage, 154.0);
        assert_eq!(bomb.spec.heal_percent, 15.0);
    }

    #[test]
    fn oct_defender_support_profile_matches_java() {
        let units = load();
        let oct = by_name(&units, "oct");

        assert_eq!(oct.armor, 16.0);
        assert_eq!(oct.health, 24000.0);
        assert_eq!(oct.speed, 0.8);
        assert_eq!(oct.rotate_speed, 1.0);
        assert_eq!(oct.accel, 0.04);
        assert_eq!(oct.drag, 0.018);
        assert!(oct.flying);
        assert_eq!(oct.engine_offset, 46.0);
        assert_eq!(oct.engine_size, 7.8);
        assert!(!oct.face_target);
        assert_eq!(oct.hit_size, 66.0);
        assert_eq!(oct.payload_capacity, 30.25 * super_tile_payload());
        assert_eq!(oct.build_speed, 4.0);
        assert!(!oct.draw_shields);
        assert!(oct.low_altitude);
        assert_eq!(oct.build_beam_offset, 43.0);
        assert_eq!(oct.loop_sound, "loopHover");
        assert_eq!(oct.ammo_capacity, 1);
        assert!(oct
            .abilities
            .iter()
            .any(|entry| entry == "ForceFieldAbility:140:4:7000:480:8:0"));
        assert!(oct
            .abilities
            .iter()
            .any(|entry| entry == "RepairFieldAbility:130:120:140"));
        assert!(oct.weapons.is_empty());
    }

    #[test]
    fn risso_naval_attack_profile_matches_java() {
        let units = load();
        let risso = by_name(&units, "risso");

        assert!(risso.naval);
        assert_eq!(risso.speed, 1.1);
        assert_eq!(risso.drag, 0.13);
        assert_eq!(risso.hit_size, 10.0);
        assert_eq!(risso.health, 280.0);
        assert_eq!(risso.armor, 2.0);
        assert_eq!(risso.accel, 0.4);
        assert_eq!(risso.rotate_speed, 3.3);
        assert!(!risso.face_target);
        assert_eq!(risso.trail_length, 20);
        assert_eq!(risso.wave_trail_x, 4.0);
        assert_eq!(risso.trail_scl, 1.3);
        assert_eq!(risso.move_sound_volume, 0.4);
        assert_eq!(risso.move_sound, "shipMove");
        assert_eq!(risso.weapons.len(), 2);

        let basic = &risso.weapons[0];
        assert_eq!(basic.name, "mount-weapon");
        assert_eq!(basic.reload, 13.0);
        assert_eq!(basic.x, 4.0);
        assert_eq!(basic.shoot_y, 4.0);
        assert_eq!(basic.y, 1.5);
        assert!(basic.rotate);
        assert_eq!(basic.eject_effect, "casing1");
        assert_eq!(basic.bullet, "risso_basic");

        let missile = &risso.weapons[1];
        assert_eq!(missile.name, "missiles-mount");
        assert!(!missile.mirror);
        assert_eq!(missile.reload, 25.0);
        assert_eq!(missile.x, 0.0);
        assert_eq!(missile.y, -5.0);
        assert!(missile.rotate);
        assert_eq!(missile.eject_effect, "casing1");
        assert_eq!(missile.shoot_sound, "shootMissileShort");
        assert_eq!(missile.bullet, "risso_missile");

        let bullets = bullets::load();
        let basic_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == basic.bullet)
            .expect("missing risso_basic");
        assert_eq!(basic_bullet.spec.kind, BulletKind::Basic);
        assert_eq!(basic_bullet.spec.ammo_multiplier, 2.0);
        let missile_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == missile.bullet)
            .expect("missing risso_missile");
        assert_eq!(missile_bullet.spec.kind, BulletKind::Missile);
        assert_eq!(missile_bullet.spec.splash_damage, 10.0);
        assert_eq!(missile_bullet.spec.weave_scale, 8.0);
    }

    #[test]
    fn minke_naval_attack_profile_matches_java() {
        let units = load();
        let minke = by_name(&units, "minke");

        assert!(minke.naval);
        assert_eq!(minke.health, 600.0);
        assert_eq!(minke.speed, 0.9);
        assert_eq!(minke.drag, 0.15);
        assert_eq!(minke.hit_size, 13.0);
        assert_eq!(minke.armor, 4.0);
        assert_eq!(minke.accel, 0.3);
        assert_eq!(minke.rotate_speed, 2.6);
        assert!(!minke.face_target);
        assert_eq!(minke.move_sound_volume, 0.55);
        assert_eq!(minke.move_sound_pitch_min, 0.9);
        assert_eq!(minke.move_sound_pitch_max, 0.9);
        assert_eq!(minke.move_sound, "shipMove");
        assert_eq!(minke.trail_length, 20);
        assert_eq!(minke.wave_trail_x, 5.5);
        assert_eq!(minke.wave_trail_y, -4.0);
        assert_eq!(minke.trail_scl, 1.9);
        assert_eq!(minke.weapons.len(), 2);

        let flak = &minke.weapons[0];
        assert_eq!(flak.name, "mount-weapon");
        assert_eq!(flak.reload, 10.0);
        assert_eq!(flak.x, 5.0);
        assert_eq!(flak.y, 3.5);
        assert!(flak.rotate);
        assert_eq!(flak.rotate_speed, 5.0);
        assert_eq!(flak.inaccuracy, 8.0);
        assert_eq!(flak.eject_effect, "casing1");
        assert_eq!(flak.shoot_sound, "shootDuo");
        assert_eq!(flak.bullet, "minke_flak");

        let artillery = &minke.weapons[1];
        assert_eq!(artillery.name, "artillery-mount");
        assert_eq!(artillery.reload, 30.0);
        assert_eq!(artillery.x, 5.0);
        assert_eq!(artillery.y, -5.0);
        assert!(artillery.rotate);
        assert_eq!(artillery.inaccuracy, 2.0);
        assert_eq!(artillery.rotate_speed, 2.0);
        assert_eq!(artillery.shake, 1.5);
        assert_eq!(artillery.eject_effect, "casing2");
        assert_eq!(artillery.shoot_sound, "shootArtillerySmall");
        assert_eq!(artillery.bullet, "minke_artillery");

        let bullets = bullets::load();
        let flak_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == flak.bullet)
            .expect("missing minke_flak");
        assert_eq!(flak_bullet.spec.kind, BulletKind::Flak);
        assert_eq!(flak_bullet.spec.splash_damage, 40.5);
        let artillery_bullet = bullets
            .iter()
            .find(|bullet| bullet.name() == artillery.bullet)
            .expect("missing minke_artillery");
        assert_eq!(artillery_bullet.spec.kind, BulletKind::Artillery);
        assert_eq!(artillery_bullet.spec.splash_damage_radius, 22.5);
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
            .any(|entry| entry == "RegenAbility:0.023809524:0"));
        assert!(renale
            .abilities
            .iter()
            .any(|entry| entry == "LiquidExplodeAbility:neoplasm"));

        let latum = by_name(&units, "latum");
        assert!(latum
            .abilities
            .iter()
            .any(|entry| entry == "SpawnDeathAbility:renale:5:11"));

        let scepter = by_name(&units, "scepter");
        assert!(scepter
            .abilities
            .iter()
            .any(|entry| entry == "ShieldRegenFieldAbility:25:250:60:60"));

        let pulsar = by_name(&units, "pulsar");
        assert!(pulsar
            .abilities
            .iter()
            .any(|entry| entry == "ShieldRegenFieldAbility:20:40:300:60"));

        let bryde = by_name(&units, "bryde");
        assert!(bryde
            .abilities
            .iter()
            .any(|entry| entry == "ShieldRegenFieldAbility:20:40:240:60"));

        let nova = by_name(&units, "nova");
        assert!(nova
            .abilities
            .iter()
            .any(|entry| entry == "RepairFieldAbility:10:240:60"));

        let poly = by_name(&units, "poly");
        assert!(poly
            .abilities
            .iter()
            .any(|entry| entry == "RepairFieldAbility:5:480:50"));

        let oct = by_name(&units, "oct");
        assert!(oct
            .abilities
            .iter()
            .any(|entry| entry == "ForceFieldAbility:140:4:7000:480:8:0"));
        assert!(oct
            .abilities
            .iter()
            .any(|entry| entry == "RepairFieldAbility:130:120:140"));

        let quasar = by_name(&units, "quasar");
        assert!(quasar
            .abilities
            .iter()
            .any(|entry| entry == "ForceFieldAbility:60:0.4:500:360"));

        let tecta = by_name(&units, "tecta");
        assert!(tecta
            .abilities
            .iter()
            .any(|entry| { entry == "ShieldArcAbility:45:0.75:2500:480:82:0:0:-20:false:8:1" }));

        let elude = by_name(&units, "elude");
        assert!(elude
            .abilities
            .iter()
            .any(|entry| entry == "MoveEffectAbility:0:-7:4:missileTrailShort:true"));

        let oxynoe = by_name(&units, "oxynoe");
        assert!(oxynoe
            .abilities
            .iter()
            .any(|entry| entry == "StatusFieldAbility:overclock:360:360:60"));

        let aegires = by_name(&units, "aegires");
        assert!(aegires
            .abilities
            .iter()
            .any(|entry| entry == "EnergyFieldAbility:40:65:180:1.5:0.5:25"));

        let navanax = by_name(&units, "navanax");
        assert!(navanax
            .abilities
            .iter()
            .any(|entry| entry == "SuppressionFieldAbility:90:90:200:0:-10:true:13"));

        let quell = by_name(&units, "quell");
        assert!(quell
            .abilities
            .iter()
            .any(|entry| entry == "SuppressionFieldAbility:480:90:200:0:1:true:13"));

        let disrupt = by_name(&units, "disrupt");
        assert!(disrupt
            .abilities
            .iter()
            .any(|entry| entry == "SuppressionFieldAbility:900:90:320:0:10:true:13"));
        assert_eq!(
            disrupt
                .abilities
                .iter()
                .filter(|entry| entry.starts_with("SuppressionFieldAbility:"))
                .count(),
            3
        );

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
