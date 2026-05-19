use std::io::{self, Read, Write};

pub const LOGIC_CONTROL_COOLDOWN: f32 = 60.0 * 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseTurretConfig {
    pub range: f32,
    pub place_overlap_margin: f32,
    pub fog_radius_multiplier: f32,
    pub disable_overlap_check: bool,
    pub activation_time: f32,
    pub tile_size: f32,
}

impl Default for BaseTurretConfig {
    fn default() -> Self {
        Self {
            range: 80.0,
            place_overlap_margin: 8.0 * 7.0,
            fog_radius_multiplier: 1.0,
            disable_overlap_check: false,
            activation_time: 0.0,
            tile_size: 8.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseTurretState {
    pub rotation: f32,
    pub activation_timer: f32,
}

impl Default for BaseTurretState {
    fn default() -> Self {
        Self {
            rotation: 90.0,
            activation_timer: 0.0,
        }
    }
}

pub fn base_turret_place_overlap_range(existing: f32, config: BaseTurretConfig) -> f32 {
    if config.disable_overlap_check {
        existing
    } else {
        existing.max(config.range + config.place_overlap_margin)
    }
}

pub fn base_turret_fog_radius(existing: i32, config: BaseTurretConfig) -> i32 {
    ((config.range / config.tile_size * config.fog_radius_multiplier).round() as i32).max(existing)
}

pub fn base_turret_placed(state: &mut BaseTurretState, activation_time: f32) {
    state.activation_timer = activation_time;
}

pub fn base_turret_status_inactive(activation_timer: f32) -> bool {
    activation_timer > 0.0
}

pub fn base_turret_activation_progress(activation_timer: f32, activation_time: f32) -> f32 {
    if activation_time <= 0.0 {
        1.0
    } else {
        1.0 - activation_timer / activation_time
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReloadTurretState {
    pub base: BaseTurretState,
    pub reload_counter: f32,
}

impl Default for ReloadTurretState {
    fn default() -> Self {
        Self {
            base: BaseTurretState::default(),
            reload_counter: 0.0,
        }
    }
}

pub fn reload_turret_can_reload(reload_counter: f32, reload: f32) -> bool {
    reload_counter < reload
}

pub fn reload_turret_update_reload(
    reload_counter: f32,
    delta: f32,
    ammo_reload_multiplier: f32,
    base_reload_speed: f32,
    reload: f32,
) -> f32 {
    (reload_counter + delta * ammo_reload_multiplier * base_reload_speed).min(reload)
}

pub fn reload_turret_update_cooling(
    reload_counter: f32,
    can_reload: bool,
    coolant_efficiency: f32,
    efficiency: f32,
    coolant_amount: f32,
    edelta: f32,
    heat_capacity: f32,
    coolant_multiplier: f32,
    ammo_reload_multiplier: f32,
) -> f32 {
    if can_reload && coolant_efficiency > 0.0 && efficiency > 0.0 {
        let amount = coolant_amount * coolant_efficiency;
        reload_counter
            + amount * edelta * heat_capacity * coolant_multiplier * ammo_reload_multiplier
    } else {
        reload_counter
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurretConfig {
    pub reload: f32,
    pub reload_while_charging: bool,
    pub first_shot_delay: f32,
    pub shoot_shots: i32,
    pub target_interval: f32,
    pub new_target_interval: f32,
    pub target_ground: bool,
    pub heat_requirement: f32,
    pub max_heat_efficiency: f32,
    pub min_warmup: f32,
    pub move_while_charging: bool,
    pub warmup_maintain_time: f32,
    pub linear_warmup: bool,
    pub shoot_warmup_speed: f32,
    pub recoil_time: f32,
    pub recoil_pow: f32,
    pub recoil: f32,
    pub cooldown_time: f32,
    pub range: f32,
    pub tracking_range: f32,
    pub fog_radius_multiplier: f32,
    pub tile_size: f32,
}

impl Default for TurretConfig {
    fn default() -> Self {
        Self {
            reload: 10.0,
            reload_while_charging: true,
            first_shot_delay: 0.0,
            shoot_shots: 1,
            target_interval: 20.0,
            new_target_interval: -1.0,
            target_ground: true,
            heat_requirement: -1.0,
            max_heat_efficiency: 3.0,
            min_warmup: 0.0,
            move_while_charging: true,
            warmup_maintain_time: 0.0,
            linear_warmup: false,
            shoot_warmup_speed: 0.1,
            recoil_time: -1.0,
            recoil_pow: 1.8,
            recoil: 1.0,
            cooldown_time: 20.0,
            range: 80.0,
            tracking_range: 0.0,
            fog_radius_multiplier: 1.0,
            tile_size: 8.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurretInit {
    pub shoot_y: f32,
    pub elevation: f32,
    pub recoil_time: f32,
    pub cooldown_time: f32,
    pub new_target_interval: f32,
    pub disable_overlap_check: bool,
    pub tracking_range: f32,
}

pub fn turret_init_values(
    mut shoot_y: f32,
    mut elevation: f32,
    mut recoil_time: f32,
    mut cooldown_time: f32,
    mut new_target_interval: f32,
    size: i32,
    tile_size: f32,
    target_ground: bool,
    range: f32,
    tracking_range: f32,
    reload: f32,
    target_interval: f32,
) -> TurretInit {
    if shoot_y == f32::NEG_INFINITY {
        shoot_y = size as f32 * tile_size / 2.0;
    }
    if elevation < 0.0 {
        elevation = size as f32 / 2.0;
    }
    if recoil_time < 0.0 {
        recoil_time = reload;
    }
    if cooldown_time < 0.0 {
        cooldown_time = reload;
    }
    if new_target_interval <= 0.0 {
        new_target_interval = target_interval;
    }
    TurretInit {
        shoot_y,
        elevation,
        recoil_time,
        cooldown_time,
        new_target_interval,
        disable_overlap_check: !target_ground,
        tracking_range: range.max(tracking_range),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurretState {
    pub reload_counter: f32,
    pub rotation: f32,
    pub total_ammo: i32,
    pub cur_recoil: f32,
    pub heat: f32,
    pub logic_control_time: f32,
    pub shoot_warmup: f32,
    pub charge: f32,
    pub warmup_hold: f32,
    pub total_shots: i32,
    pub barrel_counter: i32,
    pub logic_shooting: bool,
    pub was_shooting: bool,
    pub queued_bullets: i32,
    pub heat_req: f32,
    pub side_heat: [f32; 4],
}

impl Default for TurretState {
    fn default() -> Self {
        Self {
            reload_counter: 0.0,
            rotation: 90.0,
            total_ammo: 0,
            cur_recoil: 0.0,
            heat: 0.0,
            logic_control_time: -1.0,
            shoot_warmup: 0.0,
            charge: 0.0,
            warmup_hold: 0.0,
            total_shots: 0,
            barrel_counter: 0,
            logic_shooting: false,
            was_shooting: false,
            queued_bullets: 0,
            heat_req: 0.0,
            side_heat: [0.0; 4],
        }
    }
}

pub fn turret_reload_stat_per_second(
    reload: f32,
    first_shot_delay: f32,
    reload_while_charging: bool,
    shots: i32,
) -> f32 {
    60.0 / (reload
        + if reload_while_charging {
            0.0
        } else {
            first_shot_delay
        })
        * shots as f32
}

pub fn turret_progress(reload_counter: f32, reload: f32) -> f32 {
    (reload_counter / reload).clamp(0.0, 1.0)
}

pub fn turret_fog_radius(
    range: f32,
    ammo_range_change: f32,
    tile_size: f32,
    fog_radius_multiplier: f32,
) -> f32 {
    (range + ammo_range_change) / tile_size * fog_radius_multiplier
}

pub fn turret_range(base_range: f32, ammo_range_change: Option<f32>) -> f32 {
    base_range + ammo_range_change.unwrap_or(0.0)
}

pub fn turret_tracking_range(base_range: f32, current_range: f32, tracking_range: f32) -> f32 {
    current_range + tracking_range - base_range
}

pub fn turret_min_range(base_min_range: f32, ammo_min_range_change: Option<f32>) -> f32 {
    base_min_range + ammo_min_range_change.unwrap_or(0.0)
}

pub fn turret_draw_rotation(rotation: f32) -> f32 {
    rotation - 90.0
}

pub fn turret_should_consume(is_shooting: bool, reload_counter: f32, reload: f32) -> bool {
    is_shooting || reload_counter < reload
}

pub fn turret_logic_controlled(logic_control_time: f32) -> bool {
    logic_control_time > 0.0
}

pub fn turret_is_active(
    has_target: bool,
    was_shooting: bool,
    enabled: bool,
    activation_timer: f32,
) -> bool {
    (has_target || was_shooting) && enabled && activation_timer <= 0.0
}

pub fn turret_is_shooting(
    always_shooting: bool,
    controlled: bool,
    unit_shooting: bool,
    logic_control_time: f32,
    logic_shooting: bool,
    has_target: bool,
) -> bool {
    always_shooting
        || if controlled {
            unit_shooting
        } else if turret_logic_controlled(logic_control_time) {
            logic_shooting
        } else {
            has_target
        }
}

pub fn turret_should_turn(move_while_charging: bool, charging: bool) -> bool {
    move_while_charging || !charging
}

pub fn turret_charging(queued_bullets: i32, first_shot_delay: f32) -> bool {
    queued_bullets > 0 && first_shot_delay > 0.0
}

pub fn turret_can_consume(heat_requirement: f32, heat_req: f32, parent_can_consume: bool) -> bool {
    parent_can_consume && !(heat_requirement > 0.0 && heat_req <= 0.0)
}

pub fn turret_efficiency_with_heat(
    efficiency: f32,
    heat_requirement: f32,
    heat_req: f32,
    max_heat_efficiency: f32,
    cheating: bool,
) -> f32 {
    if heat_requirement > 0.0 {
        efficiency
            * (heat_req / heat_requirement)
                .max(if cheating { 1.0 } else { 0.0 })
                .min(max_heat_efficiency)
    } else {
        efficiency
    }
}

pub fn turret_use_ammo(
    total_ammo: &mut i32,
    entry_amount: &mut i32,
    ammo_per_shot: i32,
    cheating: bool,
) -> bool {
    if cheating {
        return false;
    }
    *entry_amount -= ammo_per_shot;
    *total_ammo = (*total_ammo - ammo_per_shot).max(0);
    *entry_amount <= 0
}

pub fn turret_update_warmup(
    shoot_warmup: f32,
    mut warmup_hold: f32,
    is_shooting_and_can_consume: bool,
    charging: bool,
    controlled: bool,
    delta: f32,
    warmup_maintain_time: f32,
    linear_warmup: bool,
    shoot_warmup_speed: f32,
    efficiency: f32,
) -> (f32, f32) {
    let mut warmup_target = if is_shooting_and_can_consume || charging {
        1.0
    } else {
        0.0
    };
    if warmup_target > 0.0 && !controlled {
        warmup_hold = 1.0;
    }
    if warmup_hold > 0.0 && warmup_maintain_time > 0.0 {
        warmup_hold -= delta / warmup_maintain_time;
        warmup_target = 1.0;
    }
    let speed = shoot_warmup_speed * if warmup_target > 0.0 { efficiency } else { 1.0 };
    let next = if linear_warmup {
        approach_delta(shoot_warmup, warmup_target, speed)
    } else {
        lerp_delta(shoot_warmup, warmup_target, speed)
    };
    (next, warmup_hold)
}

pub fn turret_update_heat_recoil_charge(
    state: &mut TurretState,
    delta: f32,
    recoil_time: f32,
    cooldown_time: f32,
    charging: bool,
    first_shot_delay: f32,
) {
    state.cur_recoil = approach_delta(state.cur_recoil, 0.0, delta / recoil_time);
    state.heat = approach_delta(state.heat, 0.0, delta / cooldown_time);
    state.charge = if charging {
        approach_delta(state.charge, 1.0, delta / first_shot_delay)
    } else {
        0.0
    };
    if state.logic_control_time > 0.0 {
        state.logic_control_time -= delta;
    }
}

pub fn turret_recoil_offset(
    rotation: f32,
    cur_recoil: f32,
    recoil_pow: f32,
    recoil: f32,
) -> (f32, f32) {
    let length = -cur_recoil.powf(recoil_pow) * recoil;
    (
        rotation.to_radians().cos() * length,
        rotation.to_radians().sin() * length,
    )
}

pub fn turret_update_shooting_ready(
    reload_counter: f32,
    reload: f32,
    charging: bool,
    shoot_warmup: f32,
    min_warmup: f32,
) -> bool {
    reload_counter >= reload && !charging && shoot_warmup >= min_warmup
}

pub fn turret_after_shoot_reload(reload_counter: f32, reload: f32) -> f32 {
    reload_counter % reload
}

pub fn turret_bullet_life_scale(
    scale_life: bool,
    scale_lifetime_offset: f32,
    distance_to_target: f32,
    bullet_range: f32,
    min_range: f32,
    range: f32,
) -> f32 {
    if scale_life {
        ((1.0 + scale_lifetime_offset) * distance_to_target / bullet_range)
            .clamp(min_range / bullet_range, range / bullet_range)
    } else {
        1.0
    }
}

pub fn turret_write_child<W: Write>(write: &mut W, state: &TurretState) -> io::Result<()> {
    write_f32(write, state.reload_counter)?;
    write_f32(write, state.rotation)
}

pub fn turret_read_child<R: Read>(read: &mut R, revision: u8) -> io::Result<TurretState> {
    if revision < 1 {
        return Ok(TurretState::default());
    }
    Ok(TurretState {
        reload_counter: read_f32(read)?,
        rotation: read_f32(read)?,
        ..TurretState::default()
    })
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn approach_delta(from: f32, to: f32, amount: f32) -> f32 {
    if from < to {
        (from + amount).min(to)
    } else {
        (from - amount).max(to)
    }
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_and_reload_turret_helpers_follow_java_defaults() {
        let config = BaseTurretConfig::default();
        assert_eq!(base_turret_place_overlap_range(0.0, config), 136.0);
        assert_eq!(base_turret_fog_radius(0, config), 10);
        let mut state = BaseTurretState::default();
        base_turret_placed(&mut state, 120.0);
        assert!(base_turret_status_inactive(state.activation_timer));
        assert_eq!(base_turret_activation_progress(60.0, 120.0), 0.5);

        assert!(reload_turret_can_reload(5.0, 10.0));
        assert_eq!(reload_turret_update_reload(9.0, 2.0, 1.0, 1.0, 10.0), 10.0);
        assert_eq!(
            reload_turret_update_cooling(0.0, true, 0.5, 1.0, 2.0, 3.0, 0.4, 5.0, 2.0),
            12.0
        );
    }

    #[test]
    fn turret_init_status_and_heat_helpers_follow_upstream() {
        let init = turret_init_values(
            f32::NEG_INFINITY,
            -1.0,
            -1.0,
            -1.0,
            -1.0,
            3,
            8.0,
            false,
            80.0,
            120.0,
            30.0,
            20.0,
        );
        assert_eq!(init.shoot_y, 12.0);
        assert_eq!(init.elevation, 1.5);
        assert_eq!(init.recoil_time, 30.0);
        assert_eq!(init.cooldown_time, 30.0);
        assert_eq!(init.new_target_interval, 20.0);
        assert!(init.disable_overlap_check);
        assert_eq!(init.tracking_range, 120.0);

        assert_eq!(turret_reload_stat_per_second(30.0, 10.0, false, 2), 3.0);
        assert_eq!(turret_progress(15.0, 30.0), 0.5);
        assert_eq!(turret_fog_radius(80.0, 16.0, 8.0, 0.5), 6.0);
        assert_eq!(turret_range(80.0, Some(10.0)), 90.0);
        assert_eq!(turret_tracking_range(80.0, 90.0, 120.0), 130.0);
        assert_eq!(turret_min_range(10.0, Some(5.0)), 15.0);
        assert_eq!(turret_draw_rotation(180.0), 90.0);
        assert!(turret_is_active(true, false, true, 0.0));
        assert!(turret_is_shooting(
            false,
            false,
            false,
            LOGIC_CONTROL_COOLDOWN,
            true,
            false
        ));
        assert!(turret_should_turn(false, false));
        assert!(turret_charging(1, 5.0));
        assert!(!turret_can_consume(1.0, 0.0, true));
        assert_eq!(
            turret_efficiency_with_heat(1.0, 10.0, 15.0, 3.0, false),
            1.5
        );
    }

    #[test]
    fn turret_runtime_and_serialization_follow_java_child_fields() {
        let (warmup, hold) =
            turret_update_warmup(0.0, 0.0, true, false, false, 1.0, 60.0, false, 0.1, 0.5);
        assert_eq!(warmup, 0.05);
        assert!((hold - 0.98333335).abs() < 0.00001);

        let mut state = TurretState {
            reload_counter: 12.0,
            rotation: 45.0,
            cur_recoil: 1.0,
            heat: 1.0,
            charge: 0.0,
            logic_control_time: 5.0,
            ..TurretState::default()
        };
        turret_update_heat_recoil_charge(&mut state, 1.0, 10.0, 20.0, true, 5.0);
        assert_eq!(state.cur_recoil, 0.9);
        assert_eq!(state.heat, 0.95);
        assert_eq!(state.charge, 0.2);
        assert_eq!(state.logic_control_time, 4.0);
        let offset = turret_recoil_offset(0.0, 1.0, 1.8, 2.0);
        assert_eq!(offset, (-2.0, -0.0));
        assert!(turret_update_shooting_ready(10.0, 10.0, false, 1.0, 0.5));
        assert_eq!(turret_after_shoot_reload(25.0, 10.0), 5.0);
        assert_eq!(
            turret_bullet_life_scale(true, 0.0, 50.0, 100.0, 20.0, 80.0),
            0.5
        );
        let mut total = 5;
        let mut entry = 2;
        assert!(turret_use_ammo(&mut total, &mut entry, 2, false));
        assert_eq!(total, 3);

        let mut bytes = Vec::new();
        turret_write_child(&mut bytes, &state).unwrap();
        let restored = turret_read_child(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(restored.reload_counter, state.reload_counter);
        assert_eq!(restored.rotation, state.rotation);
        assert_eq!(
            turret_read_child(&mut [].as_slice(), 0).unwrap(),
            TurretState::default()
        );
    }
}
