use std::io::{self, Read, Write};

use crate::mindustry::{
    ctype::{ContentId, ContentType},
    r#type::{PayloadKey, PayloadSeq},
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemAmmoEntry {
    pub item_id: i16,
    pub amount: i16,
}

pub fn item_turret_accept_stack(
    ammo_multiplier: Option<f32>,
    max_ammo: i32,
    total_ammo: i32,
    amount: i32,
) -> i32 {
    ammo_multiplier
        .map(|multiplier| (((max_ammo - total_ammo) as f32 / multiplier) as i32).min(amount))
        .unwrap_or(0)
}

pub fn item_turret_accept_item(
    ammo_multiplier: Option<f32>,
    total_ammo: i32,
    max_ammo: i32,
) -> bool {
    ammo_multiplier
        .map(|multiplier| total_ammo as f32 + multiplier <= max_ammo as f32)
        .unwrap_or(false)
}

pub fn item_turret_handle_item(
    entries: &mut Vec<ItemAmmoEntry>,
    total_ammo: &mut i32,
    item_id: i16,
    ammo_multiplier: i16,
) {
    *total_ammo += ammo_multiplier as i32;
    if let Some(index) = entries.iter().position(|entry| entry.item_id == item_id) {
        entries[index].amount += ammo_multiplier;
        let entry = entries.remove(index);
        entries.push(entry);
    } else {
        entries.push(ItemAmmoEntry {
            item_id,
            amount: ammo_multiplier,
        });
    }
}

pub fn item_turret_consumer_efficiency(top_amount: i32, ammo_per_shot: i32, cheating: bool) -> f32 {
    if top_amount >= ammo_per_shot || cheating {
        1.0
    } else {
        0.0
    }
}

pub fn item_turret_write_ammo<W: Write>(
    write: &mut W,
    entries: &[ItemAmmoEntry],
) -> io::Result<()> {
    write.write_all(&[entries.len() as u8])?;
    for entry in entries {
        write_i16(write, entry.item_id)?;
        write_i16(write, entry.amount)?;
    }
    Ok(())
}

pub fn item_turret_read_ammo<R: Read>(
    read: &mut R,
    revision: u8,
    max_ammo: i32,
    is_valid_ammo: impl Fn(i16) -> bool,
) -> io::Result<(Vec<ItemAmmoEntry>, i32)> {
    let mut len = [0; 1];
    read.read_exact(&mut len)?;
    let mut entries = Vec::new();
    let mut total = 0;
    for _ in 0..len[0] {
        let item_id = if revision < 2 {
            let mut id = [0; 1];
            read.read_exact(&mut id)?;
            id[0] as i16
        } else {
            read_i16(read)?
        };
        let amount = read_i16(read)?.min(max_ammo as i16);
        if is_valid_ammo(item_id) {
            total += amount as i32;
            entries.push(ItemAmmoEntry { item_id, amount });
        }
    }
    Ok((entries, total))
}

pub fn payload_ammo_turret_write_payloads<W: Write>(
    write: &mut W,
    payloads: &PayloadSeq,
) -> io::Result<()> {
    write.write_all(&payloads.write_java_new())
}

pub fn payload_ammo_turret_read_payloads<R: Read>(
    read: &mut R,
    is_valid_ammo: impl Fn(PayloadKey) -> bool,
) -> io::Result<PayloadSeq> {
    let count = read_i16(read)?;
    if count >= 0 {
        let mut seq = PayloadSeq::new();
        for _ in 0..count {
            let id = read_i16(read)? as ContentId;
            let amount = read_i32(read)?;
            let key = PayloadKey::new(ContentType::Block, id);
            if is_valid_ammo(key) {
                seq.add(key, amount);
            }
        }
        return Ok(seq);
    }

    let mut seq = PayloadSeq::new();
    for _ in 0..(-(count as i32)) {
        let content_type = ContentType::from_ordinal(read_u8(read)?).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown PayloadSeq content type ordinal",
            )
        })?;
        let id = read_i16(read)? as ContentId;
        let amount = read_i32(read)?;
        let key = PayloadKey::new(content_type, id);
        if is_valid_ammo(key) {
            seq.add(key, amount);
        }
    }
    Ok(seq)
}

pub fn liquid_turret_has_ammo(ammo_multiplier: Option<f32>, current_amount: f32) -> bool {
    ammo_multiplier
        .map(|multiplier| current_amount >= 1.0 / multiplier)
        .unwrap_or(false)
}

pub fn liquid_turret_accept_liquid(
    incoming_is_ammo: bool,
    same_liquid: bool,
    current_is_ammo: bool,
    current_amount: f32,
    current_ammo_multiplier: f32,
) -> bool {
    incoming_is_ammo
        && (same_liquid
            || (!current_is_ammo || current_amount <= 1.0 / current_ammo_multiplier + 0.001))
}

pub fn liquid_turret_use_ammo(current_amount: f32, ammo_multiplier: f32, cheating: bool) -> f32 {
    if cheating {
        current_amount
    } else {
        (current_amount - 1.0 / ammo_multiplier).max(0.0)
    }
}

pub fn liquid_turret_unit_ammo_fraction(current_amount: f32, liquid_capacity: f32) -> f32 {
    current_amount / liquid_capacity
}

pub fn power_turret_sense_ammo(power_status: Option<f32>) -> f32 {
    power_status.unwrap_or(0.0)
}

pub fn power_turret_unit_ammo(power_status: Option<f32>, unit_ammo_capacity: f32) -> f32 {
    power_turret_sense_ammo(power_status) * unit_ammo_capacity
}

pub fn power_turret_has_ammo() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinuousTurretState {
    pub last_length: f32,
    pub bullets: usize,
}

impl ContinuousTurretState {
    pub fn new(size: i32) -> Self {
        Self {
            last_length: size as f32 * 4.0,
            bullets: 0,
        }
    }
}

pub fn continuous_turret_estimate_dps(damage: f32, damage_interval: Option<f32>) -> f32 {
    damage * 60.0 / damage_interval.unwrap_or(5.0)
}

pub fn continuous_turret_has_ammo(can_consume: bool) -> bool {
    can_consume
}

pub fn continuous_turret_should_consume(is_shooting: bool) -> bool {
    is_shooting
}

pub fn continuous_turret_ammo_fraction(
    efficiency: f32,
    liquid_amount: Option<f32>,
    liquid_capacity: f32,
) -> f32 {
    liquid_amount
        .map(|amount| efficiency.min(amount / liquid_capacity))
        .unwrap_or(efficiency)
}

pub fn continuous_turret_update_length(
    cur_length: f32,
    target_distance: f32,
    range: f32,
    aim_change_speed: f32,
) -> f32 {
    approach_delta(cur_length, target_distance.min(range), aim_change_speed)
}

pub fn continuous_turret_keepalive_time(
    bullet_lifetime: f32,
    optimal_life_fraction: f32,
    shoot_warmup: f32,
    efficiency: f32,
) -> f32 {
    bullet_lifetime * optimal_life_fraction * shoot_warmup.min(efficiency)
}

pub fn continuous_turret_scaled_damage(
    base_damage: f32,
    damage_multiplier: f32,
    efficiency: f32,
    time_scale: f32,
    scale_damage_efficiency: bool,
) -> f32 {
    if scale_damage_efficiency {
        base_damage * efficiency.min(1.0) * time_scale * damage_multiplier
    } else {
        base_damage
    }
}

pub fn continuous_turret_should_active_sound(bullets_any: bool) -> bool {
    bullets_any
}

pub fn continuous_turret_write_child<W: Write>(
    write: &mut W,
    state: &ContinuousTurretState,
) -> io::Result<()> {
    write_f32(write, state.last_length)
}

pub fn continuous_turret_read_child<R: Read>(
    read: &mut R,
    revision: u8,
    size: i32,
) -> io::Result<ContinuousTurretState> {
    let mut state = ContinuousTurretState::new(size);
    if revision >= 3 {
        state.last_length = read_f32(read)?;
    }
    Ok(state)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinuousLiquidTurretState {
    pub activated: bool,
}

pub fn continuous_liquid_update_activation(
    activated: bool,
    current_amount: f32,
    liquid_consumed: f32,
) -> bool {
    if current_amount >= liquid_consumed * 4.0 {
        true
    } else if current_amount < liquid_consumed {
        false
    } else {
        activated
    }
}

pub fn continuous_liquid_has_ammo(
    correct_ammo: bool,
    current_is_ammo: bool,
    current_amount: f32,
    activated: bool,
) -> bool {
    correct_ammo && current_is_ammo && current_amount > 0.0 && activated
}

pub fn continuous_liquid_should_consume(parent_should_consume: bool, activated: bool) -> bool {
    parent_should_consume && activated
}

pub fn continuous_liquid_consume_multiplier(ammo_multiplier: Option<f32>) -> f32 {
    ammo_multiplier.map(|value| 1.0 / value).unwrap_or(1.0)
}

pub fn laser_turret_placed_reload(reload: f32) -> f32 {
    reload
}

pub fn laser_turret_should_consume(bullets_any: bool, active: bool, shooting: bool) -> bool {
    bullets_any || active || shooting
}

pub fn laser_turret_progress(reload_counter: f32, reload: f32) -> f32 {
    1.0 - (reload_counter / reload).clamp(0.0, 1.0)
}

pub fn laser_turret_update_reload_counter(
    reload_counter: f32,
    bullets_any: bool,
    coolant_amount_available: Option<f32>,
    coolant_amount: f32,
    delta: f32,
    edelta: f32,
    heat_capacity: f32,
    coolant_multiplier: f32,
    cheating: bool,
) -> (f32, f32) {
    if bullets_any || reload_counter <= 0.0 {
        return (reload_counter, 0.0);
    }
    if let Some(available) = coolant_amount_available {
        let max_used = coolant_amount;
        let used = if cheating {
            max_used
        } else {
            available.min(max_used)
        } * delta;
        (
            reload_counter - used * heat_capacity * coolant_multiplier,
            used,
        )
    } else {
        (reload_counter - edelta, 0.0)
    }
}

pub fn laser_turret_update_bullet_life(
    life: f32,
    delta: f32,
    time_scale: f32,
    efficiency: f32,
) -> f32 {
    life - delta * time_scale / efficiency.max(0.00001)
}

pub fn laser_turret_turn_speed(
    efficiency: f32,
    rotate_speed: f32,
    delta: f32,
    bullets_any: bool,
    firing_move_fraction: f32,
) -> f32 {
    efficiency
        * rotate_speed
        * delta
        * if bullets_any {
            firing_move_fraction
        } else {
            1.0
        }
}

pub fn laser_turret_ready_to_shoot(
    bullets_any: bool,
    reload_counter: f32,
    efficiency: f32,
    charging: bool,
    shoot_warmup: f32,
    min_warmup: f32,
) -> bool {
    !bullets_any
        && reload_counter <= 0.0
        && efficiency > 0.0
        && !charging
        && shoot_warmup >= min_warmup
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointDefenseState {
    pub reload: ReloadTurretState,
    pub rotation: f32,
    pub has_target: bool,
}

impl Default for PointDefenseState {
    fn default() -> Self {
        Self {
            reload: ReloadTurretState::default(),
            rotation: 90.0,
            has_target: false,
        }
    }
}

pub fn point_defense_real_damage(bullet_damage: f32, block_damage_multiplier: f32) -> f32 {
    bullet_damage * block_damage_multiplier
}

pub fn point_defense_apply_damage(target_damage: f32, real_damage: f32) -> Option<f32> {
    if target_damage > real_damage {
        Some(target_damage - real_damage)
    } else {
        None
    }
}

pub fn point_defense_ready(
    angle_within_cone: bool,
    reload_counter: f32,
    reload: f32,
    has_valid_target: bool,
) -> bool {
    has_valid_target && angle_within_cone && reload_counter >= reload
}

pub fn point_defense_should_consume(parent_should_consume: bool, has_target: bool) -> bool {
    parent_should_consume && has_target
}

pub fn point_defense_write_child<W: Write>(
    write: &mut W,
    state: &PointDefenseState,
) -> io::Result<()> {
    write_f32(write, state.rotation)
}

pub fn point_defense_read_child<R: Read>(read: &mut R) -> io::Result<PointDefenseState> {
    Ok(PointDefenseState {
        rotation: read_f32(read)?,
        ..PointDefenseState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TractorBeamState {
    pub rotation: f32,
    pub last_x: f32,
    pub last_y: f32,
    pub strength: f32,
    pub any: bool,
    pub coolant_multiplier: f32,
}

impl Default for TractorBeamState {
    fn default() -> Self {
        Self {
            rotation: 90.0,
            last_x: 0.0,
            last_y: 0.0,
            strength: 0.0,
            any: false,
            coolant_multiplier: 1.0,
        }
    }
}

pub fn tractor_beam_effective_delta(efficiency: f32, coolant_multiplier: f32, delta: f32) -> f32 {
    efficiency * coolant_multiplier * delta
}

pub fn tractor_beam_coolant_step(
    current_amount: f32,
    coolant_amount: f32,
    delta: f32,
    heat_capacity: f32,
    coolant_multiplier_field: f32,
) -> (f32, f32) {
    let used = current_amount
        .min(coolant_amount * delta)
        .min(((1.0 / coolant_multiplier_field) / heat_capacity).max(0.0));
    (used, 1.0 + used * heat_capacity * coolant_multiplier_field)
}

pub fn tractor_beam_target_valid(
    target_within: bool,
    enemy_team: bool,
    target_matches_filter: bool,
    efficiency: f32,
) -> bool {
    target_within && enemy_team && target_matches_filter && efficiency > 0.02
}

pub fn tractor_beam_update_strength(strength: f32, valid_target: bool) -> f32 {
    lerp_delta(strength, if valid_target { 1.0 } else { 0.0 }, 0.1)
}

pub fn tractor_beam_damage_per_tick(
    damage: f32,
    effective_efficiency: f32,
    time_scale: f32,
    block_damage_multiplier: f32,
) -> f32 {
    damage * effective_efficiency * time_scale * block_damage_multiplier
}

pub fn tractor_beam_impulse_limit(
    force: f32,
    scaled_force: f32,
    distance: f32,
    range: f32,
    effective_delta: f32,
) -> f32 {
    (force + (1.0 - distance / range) * scaled_force) * effective_delta
}

pub fn tractor_beam_estimate_dps(
    any: bool,
    damage: f32,
    efficiency: f32,
    coolant_multiplier: f32,
) -> f32 {
    if !any || damage <= 0.0 {
        0.0
    } else {
        damage * 60.0 * efficiency * coolant_multiplier
    }
}

pub fn tractor_beam_should_consume(parent_should_consume: bool, has_target: bool) -> bool {
    parent_should_consume && has_target
}

pub fn tractor_beam_laser_width(strength: f32, efficiency: f32, laser_width: f32) -> f32 {
    strength * efficiency * laser_width
}

pub fn tractor_beam_write_child<W: Write>(
    write: &mut W,
    state: &TractorBeamState,
) -> io::Result<()> {
    write_f32(write, state.rotation)
}

pub fn tractor_beam_read_child<R: Read>(read: &mut R) -> io::Result<TractorBeamState> {
    Ok(TractorBeamState {
        rotation: read_f32(read)?,
        ..TractorBeamState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadAmmoEntry {
    pub content_id: i16,
    pub amount: i16,
}

pub fn payload_ammo_limit_lifetime(range: f32, margin: f32, speed: f32) -> f32 {
    (range + margin) / speed
}

pub fn payload_ammo_current(entries: &[PayloadAmmoEntry], ammo_keys: &[i16]) -> Option<i16> {
    ammo_keys.iter().copied().find(|key| {
        entries
            .iter()
            .any(|entry| entry.content_id == *key && entry.amount > 0)
    })
}

pub fn payload_ammo_accept_payload(
    total_payloads: i32,
    max_ammo: i32,
    content_has_ammo: bool,
) -> bool {
    total_payloads < max_ammo && content_has_ammo
}

pub fn payload_ammo_handle_payload(entries: &mut Vec<PayloadAmmoEntry>, content_id: i16) {
    if let Some(entry) = entries
        .iter_mut()
        .find(|entry| entry.content_id == content_id)
    {
        entry.amount += 1;
    } else {
        entries.push(PayloadAmmoEntry {
            content_id,
            amount: 1,
        });
    }
}

pub fn payload_ammo_total(entries: &[PayloadAmmoEntry]) -> i32 {
    entries.iter().map(|entry| entry.amount as i32).sum()
}

pub fn payload_ammo_has_ammo(entries: &[PayloadAmmoEntry]) -> bool {
    payload_ammo_total(entries) > 0
}

pub fn payload_ammo_use(entries: &mut Vec<PayloadAmmoEntry>, ammo_keys: &[i16]) -> Option<i16> {
    let content = payload_ammo_current(entries, ammo_keys)?;
    if let Some(index) = entries.iter().position(|entry| entry.content_id == content) {
        entries[index].amount -= 1;
        if entries[index].amount <= 0 {
            entries.remove(index);
        }
    }
    Some(content)
}

pub fn payload_ammo_unit_fraction(total_ammo: i32, max_ammo: i32) -> f32 {
    total_ammo as f32 / max_ammo as f32
}

pub fn payload_ammo_filter_invalid(
    entries: &mut Vec<PayloadAmmoEntry>,
    is_valid: impl Fn(i16) -> bool,
) {
    entries.retain(|entry| is_valid(entry.content_id));
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

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
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

    #[test]
    fn item_liquid_and_power_turret_helpers_follow_upstream_ammo_rules() {
        assert_eq!(item_turret_accept_stack(Some(2.0), 30, 20, 9), 5);
        assert_eq!(item_turret_accept_stack(None, 30, 20, 9), 0);
        assert!(item_turret_accept_item(Some(2.0), 28, 30));
        assert!(!item_turret_accept_item(Some(3.0), 28, 30));

        let mut entries = vec![ItemAmmoEntry {
            item_id: 1,
            amount: 2,
        }];
        let mut total = 2;
        item_turret_handle_item(&mut entries, &mut total, 2, 3);
        item_turret_handle_item(&mut entries, &mut total, 1, 3);
        assert_eq!(total, 8);
        assert_eq!(
            entries,
            vec![
                ItemAmmoEntry {
                    item_id: 2,
                    amount: 3
                },
                ItemAmmoEntry {
                    item_id: 1,
                    amount: 5
                }
            ]
        );
        assert_eq!(item_turret_consumer_efficiency(1, 2, false), 0.0);
        assert_eq!(item_turret_consumer_efficiency(1, 2, true), 1.0);

        let mut bytes = Vec::new();
        item_turret_write_ammo(&mut bytes, &entries).unwrap();
        assert_eq!(bytes, vec![2, 0, 2, 0, 3, 0, 1, 0, 5]);
        let (restored, restored_total) =
            item_turret_read_ammo(&mut bytes.as_slice(), 2, 30, |id| id == 1).unwrap();
        assert_eq!(
            restored,
            vec![ItemAmmoEntry {
                item_id: 1,
                amount: 5
            }]
        );
        assert_eq!(restored_total, 5);

        let mut legacy_payloads = Vec::new();
        legacy_payloads.extend_from_slice(&2i16.to_be_bytes());
        legacy_payloads.extend_from_slice(&5i16.to_be_bytes());
        legacy_payloads.extend_from_slice(&3i32.to_be_bytes());
        legacy_payloads.extend_from_slice(&7i16.to_be_bytes());
        legacy_payloads.extend_from_slice(&2i32.to_be_bytes());
        let restored = payload_ammo_turret_read_payloads(&mut legacy_payloads.as_slice(), |key| {
            key == PayloadKey::new(ContentType::Block, 5)
        })
        .unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored.total(), 3);
        assert_eq!(restored.get(PayloadKey::new(ContentType::Block, 5)), 3);

        assert!(liquid_turret_has_ammo(Some(2.0), 0.5));
        assert!(!liquid_turret_has_ammo(Some(2.0), 0.49));
        assert!(liquid_turret_accept_liquid(true, false, false, 5.0, 2.0));
        assert!(liquid_turret_accept_liquid(true, false, true, 0.501, 2.0));
        assert!(!liquid_turret_accept_liquid(true, false, true, 0.6, 2.0));
        assert_eq!(liquid_turret_use_ammo(2.0, 4.0, false), 1.75);
        assert_eq!(liquid_turret_unit_ammo_fraction(5.0, 20.0), 0.25);

        assert_eq!(power_turret_sense_ammo(Some(0.75)), 0.75);
        assert_eq!(power_turret_unit_ammo(Some(0.5), 10.0), 5.0);
        assert!(power_turret_has_ammo());
    }

    #[test]
    fn continuous_and_laser_turret_helpers_follow_upstream_runtime_edges() {
        let state = ContinuousTurretState::new(3);
        assert_eq!(state.last_length, 12.0);
        assert_eq!(continuous_turret_estimate_dps(10.0, Some(5.0)), 120.0);
        assert!(continuous_turret_has_ammo(true));
        assert!(continuous_turret_should_consume(true));
        assert_eq!(continuous_turret_ammo_fraction(0.8, Some(4.0), 20.0), 0.2);
        assert_eq!(continuous_turret_update_length(5.0, 20.0, 10.0, 3.0), 8.0);
        assert_eq!(continuous_turret_keepalive_time(100.0, 0.8, 0.5, 1.0), 40.0);
        assert_eq!(
            continuous_turret_scaled_damage(20.0, 1.5, 0.5, 2.0, true),
            30.0
        );
        assert!(continuous_turret_should_active_sound(true));
        let mut bytes = Vec::new();
        continuous_turret_write_child(&mut bytes, &state).unwrap();
        assert_eq!(
            continuous_turret_read_child(&mut bytes.as_slice(), 3, 3)
                .unwrap()
                .last_length,
            12.0
        );
        assert_eq!(
            continuous_turret_read_child(&mut [].as_slice(), 2, 4)
                .unwrap()
                .last_length,
            16.0
        );

        assert!(continuous_liquid_update_activation(false, 4.0, 1.0));
        assert!(!continuous_liquid_update_activation(true, 0.5, 1.0));
        assert!(continuous_liquid_has_ammo(true, true, 0.1, true));
        assert!(continuous_liquid_should_consume(true, true));
        assert_eq!(continuous_liquid_consume_multiplier(Some(4.0)), 0.25);

        assert_eq!(laser_turret_placed_reload(90.0), 90.0);
        assert!(laser_turret_should_consume(true, false, false));
        assert_eq!(laser_turret_progress(45.0, 90.0), 0.5);
        assert_eq!(
            laser_turret_update_reload_counter(
                10.0,
                false,
                Some(3.0),
                2.0,
                1.0,
                1.0,
                0.5,
                1.0,
                false
            ),
            (9.0, 2.0)
        );
        assert_eq!(
            laser_turret_update_reload_counter(10.0, false, None, 2.0, 1.0, 3.0, 0.5, 1.0, false),
            (7.0, 0.0)
        );
        assert_eq!(laser_turret_update_bullet_life(100.0, 1.0, 2.0, 0.5), 96.0);
        assert_eq!(laser_turret_turn_speed(1.0, 5.0, 2.0, true, 0.25), 2.5);
        assert!(laser_turret_ready_to_shoot(
            false, 0.0, 1.0, false, 1.0, 0.0
        ));
    }

    #[test]
    fn point_defense_tractor_and_payload_turrets_follow_upstream_edges() {
        assert_eq!(point_defense_real_damage(10.0, 1.5), 15.0);
        assert_eq!(point_defense_apply_damage(20.0, 15.0), Some(5.0));
        assert_eq!(point_defense_apply_damage(10.0, 15.0), None);
        assert!(point_defense_ready(true, 30.0, 30.0, true));
        assert!(point_defense_should_consume(true, true));
        let point = PointDefenseState {
            rotation: 135.0,
            ..PointDefenseState::default()
        };
        let mut bytes = Vec::new();
        point_defense_write_child(&mut bytes, &point).unwrap();
        assert_eq!(
            point_defense_read_child(&mut bytes.as_slice())
                .unwrap()
                .rotation,
            135.0
        );

        assert_eq!(tractor_beam_effective_delta(0.5, 2.0, 3.0), 3.0);
        assert_eq!(
            tractor_beam_coolant_step(10.0, 2.0, 1.0, 0.5, 1.0),
            (2.0, 2.0)
        );
        assert!(tractor_beam_target_valid(true, true, true, 0.5));
        assert_eq!(tractor_beam_update_strength(0.0, true), 0.1);
        assert_eq!(tractor_beam_damage_per_tick(4.0, 2.0, 3.0, 0.5), 12.0);
        assert_eq!(tractor_beam_impulse_limit(0.3, 0.7, 50.0, 100.0, 2.0), 1.3);
        assert_eq!(tractor_beam_estimate_dps(true, 2.0, 0.5, 3.0), 180.0);
        assert!(tractor_beam_should_consume(true, true));
        assert_eq!(tractor_beam_laser_width(0.5, 0.8, 0.6), 0.24000001);
        let tractor = TractorBeamState {
            rotation: 225.0,
            ..TractorBeamState::default()
        };
        let mut bytes = Vec::new();
        tractor_beam_write_child(&mut bytes, &tractor).unwrap();
        assert_eq!(
            tractor_beam_read_child(&mut bytes.as_slice())
                .unwrap()
                .rotation,
            225.0
        );

        assert_eq!(payload_ammo_limit_lifetime(80.0, 1.0, 3.0), 27.0);
        let mut payloads = vec![
            PayloadAmmoEntry {
                content_id: 2,
                amount: 1,
            },
            PayloadAmmoEntry {
                content_id: 5,
                amount: 2,
            },
        ];
        assert_eq!(payload_ammo_current(&payloads, &[5, 2]), Some(5));
        assert!(payload_ammo_accept_payload(2, 3, true));
        payload_ammo_handle_payload(&mut payloads, 2);
        assert_eq!(payload_ammo_total(&payloads), 4);
        assert!(payload_ammo_has_ammo(&payloads));
        assert_eq!(payload_ammo_use(&mut payloads, &[5, 2]), Some(5));
        assert_eq!(
            payload_ammo_unit_fraction(payload_ammo_total(&payloads), 3),
            1.0
        );
        payload_ammo_filter_invalid(&mut payloads, |id| id == 2);
        assert_eq!(
            payloads,
            vec![PayloadAmmoEntry {
                content_id: 2,
                amount: 2
            }]
        );
    }
}
