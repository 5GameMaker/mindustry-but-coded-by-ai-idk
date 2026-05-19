use std::io::{self, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallState {
    pub hit: f32,
}

pub fn wall_collision_hit(_previous_hit: f32) -> f32 {
    1.0
}

pub fn wall_draw_hit_decay(hit: f32, delta: f32, paused: bool) -> f32 {
    if paused {
        hit
    } else {
        (hit - delta / 10.0).clamp(0.0, 1.0)
    }
}

pub fn wall_should_lightning(lightning_chance: f32, random: f32) -> bool {
    lightning_chance > 0.0 && random < lightning_chance
}

pub fn wall_deflects_bullet(
    chance_deflect: f32,
    bullet_speed: f32,
    reflectable: bool,
    bullet_damage: f32,
    random: f32,
) -> bool {
    chance_deflect > 0.0
        && bullet_speed > 0.1
        && reflectable
        && bullet_damage > 0.0
        && random < chance_deflect / bullet_damage
}

pub fn wall_reflect_x(pen_x: f32, pen_y: f32) -> bool {
    pen_x > pen_y
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorState {
    pub open: bool,
}

pub fn door_check_solid(open: bool) -> bool {
    !open
}

pub fn door_sense_enabled(open: bool) -> f64 {
    if open {
        1.0
    } else {
        0.0
    }
}

pub fn door_can_toggle(
    open: bool,
    requested_open: bool,
    units_in_tile: bool,
    origin_timer_ready: bool,
) -> bool {
    open != requested_open && (!units_in_tile || requested_open) && origin_timer_ready
}

pub fn door_tapped_should_configure(
    open: bool,
    units_in_tile: bool,
    origin_timer_ready: bool,
) -> bool {
    !(units_in_tile && open) && origin_timer_ready
}

pub fn write_door_state<W: Write>(write: &mut W, state: DoorState) -> io::Result<()> {
    write.write_all(&[state.open as u8])
}

pub fn read_door_state<R: Read>(read: &mut R) -> io::Result<DoorState> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(DoorState { open: buf[0] != 0 })
}

pub fn auto_door_should_open(ground_units_in_trigger: bool) -> bool {
    ground_units_in_trigger
}

pub fn auto_door_trigger_size(block_size: i32, tile_size: f32, trigger_margin: f32) -> f32 {
    block_size as f32 * tile_size + trigger_margin * 2.0
}

pub fn shock_mine_should_trigger(enabled: bool, same_team: bool, timer_ready: bool) -> bool {
    enabled && !same_team && timer_ready
}

pub fn shock_mine_bullet_angles(shots: i32, inaccuracy_offsets: &[f32]) -> Vec<f32> {
    if shots <= 0 {
        return Vec::new();
    }
    (0..shots)
        .map(|index| {
            (360.0 / shots as f32) * index as f32
                + inaccuracy_offsets
                    .get(index as usize)
                    .copied()
                    .unwrap_or(0.0)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MendProjectorState {
    pub heat: f32,
    pub charge: f32,
    pub phase_heat: f32,
    pub smooth_efficiency: f32,
}

impl Default for MendProjectorState {
    fn default() -> Self {
        Self {
            heat: 0.0,
            charge: 0.0,
            phase_heat: 0.0,
            smooth_efficiency: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MendProjectorUpdate {
    pub fired: bool,
    pub real_range: f32,
    pub heal_fraction: f32,
    pub should_consume_optional: bool,
}

pub fn mend_projector_update(
    state: &mut MendProjectorState,
    efficiency: f32,
    optional_efficiency: f32,
    can_heal: bool,
    delta: f32,
    reload: f32,
    range: f32,
    heal_percent: f32,
    phase_boost: f32,
    phase_range_boost: f32,
) -> MendProjectorUpdate {
    state.smooth_efficiency = lerp_delta(state.smooth_efficiency, efficiency, 0.08);
    state.heat = lerp_delta(
        state.heat,
        if efficiency > 0.0 && can_heal {
            1.0
        } else {
            0.0
        },
        0.08,
    );
    state.charge += state.heat * delta;
    state.phase_heat = lerp_delta(state.phase_heat, optional_efficiency, 0.1);
    let real_range = range + state.phase_heat * phase_range_boost;
    let heal_fraction = (heal_percent + state.phase_heat * phase_boost) / 100.0 * efficiency;
    let fired = state.charge >= reload && can_heal;
    if fired {
        state.charge = 0.0;
    }
    MendProjectorUpdate {
        fired,
        real_range,
        heal_fraction,
        should_consume_optional: optional_efficiency > 0.0 && can_heal,
    }
}

pub fn write_mend_projector_state<W: Write>(
    write: &mut W,
    state: &MendProjectorState,
) -> io::Result<()> {
    write_f32(write, state.heat)?;
    write_f32(write, state.phase_heat)
}

pub fn read_mend_projector_state<R: Read>(read: &mut R) -> io::Result<MendProjectorState> {
    Ok(MendProjectorState {
        heat: read_f32(read)?,
        phase_heat: read_f32(read)?,
        ..MendProjectorState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverdriveProjectorState {
    pub heat: f32,
    pub charge: f32,
    pub phase_heat: f32,
    pub smooth_efficiency: f32,
    pub use_progress: f32,
}

impl Default for OverdriveProjectorState {
    fn default() -> Self {
        Self {
            heat: 0.0,
            charge: 0.0,
            phase_heat: 0.0,
            smooth_efficiency: 0.0,
            use_progress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverdriveProjectorUpdate {
    pub applied_boost: bool,
    pub consumed: bool,
    pub real_range: f32,
    pub real_boost: f32,
}

pub fn overdrive_real_boost(
    speed_boost: f32,
    phase_heat: f32,
    speed_boost_phase: f32,
    efficiency: f32,
) -> f32 {
    (speed_boost + phase_heat * speed_boost_phase) * efficiency
}

#[allow(clippy::too_many_arguments)]
pub fn overdrive_projector_update(
    state: &mut OverdriveProjectorState,
    efficiency: f32,
    optional_efficiency: f32,
    has_boost: bool,
    delta: f32,
    reload: f32,
    range: f32,
    phase_range_boost: f32,
    speed_boost: f32,
    speed_boost_phase: f32,
    use_time: f32,
) -> OverdriveProjectorUpdate {
    state.smooth_efficiency = lerp_delta(state.smooth_efficiency, efficiency, 0.08);
    state.heat = lerp_delta(state.heat, if efficiency > 0.0 { 1.0 } else { 0.0 }, 0.08);
    state.charge += state.heat * delta;
    if has_boost {
        state.phase_heat = lerp_delta(state.phase_heat, optional_efficiency, 0.1);
    }
    let applied_boost = state.charge >= reload;
    if applied_boost {
        state.charge = 0.0;
    }
    if efficiency > 0.0 {
        state.use_progress += delta;
    }
    let consumed = state.use_progress >= use_time;
    if consumed {
        state.use_progress %= use_time;
    }
    OverdriveProjectorUpdate {
        applied_boost,
        consumed,
        real_range: range + state.phase_heat * phase_range_boost,
        real_boost: overdrive_real_boost(
            speed_boost,
            state.phase_heat,
            speed_boost_phase,
            efficiency,
        ),
    }
}

pub fn write_overdrive_projector_state<W: Write>(
    write: &mut W,
    state: &OverdriveProjectorState,
) -> io::Result<()> {
    write_f32(write, state.heat)?;
    write_f32(write, state.phase_heat)
}

pub fn read_overdrive_projector_state<R: Read>(
    read: &mut R,
) -> io::Result<OverdriveProjectorState> {
    Ok(OverdriveProjectorState {
        heat: read_f32(read)?,
        phase_heat: read_f32(read)?,
        ..OverdriveProjectorState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceProjectorState {
    pub broken: bool,
    pub buildup: f32,
    pub radscl: f32,
    pub hit: f32,
    pub warmup: f32,
    pub phase_heat: f32,
}

impl Default for ForceProjectorState {
    fn default() -> Self {
        Self {
            broken: true,
            buildup: 0.0,
            radscl: 0.0,
            hit: 0.0,
            warmup: 0.0,
            phase_heat: 0.0,
        }
    }
}

pub fn force_projector_real_radius(
    radius: f32,
    phase_heat: f32,
    phase_radius_boost: f32,
    radscl: f32,
) -> f32 {
    (radius + phase_heat * phase_radius_boost) * radscl
}

pub fn force_projector_shield(
    broken: bool,
    shield_health: f32,
    phase_shield_boost: f32,
    phase_heat: f32,
    buildup: f32,
) -> f32 {
    if broken {
        0.0
    } else {
        (shield_health + phase_shield_boost * phase_heat - buildup).max(0.0)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn force_projector_update(
    state: &mut ForceProjectorState,
    efficiency: f32,
    phase_valid: bool,
    coolant_efficiency: f32,
    coolant_heat_capacity: f32,
    delta: f32,
    shield_health: f32,
    phase_shield_boost: f32,
    cooldown_normal: f32,
    cooldown_broken_base: f32,
    cooldown_liquid: f32,
) -> bool {
    state.phase_heat = lerp_delta(state.phase_heat, if phase_valid { 1.0 } else { 0.0 }, 0.1);
    state.radscl = lerp_delta(
        state.radscl,
        if state.broken { 0.0 } else { state.warmup },
        0.05,
    );
    state.warmup = lerp_delta(state.warmup, efficiency, 0.1);

    if state.buildup > 0.0 {
        let mut scale = if !state.broken {
            cooldown_normal
        } else {
            cooldown_broken_base
        };
        if coolant_efficiency > 0.0 {
            scale *= cooldown_liquid * (1.0 + (coolant_heat_capacity - 0.4) * 0.9);
        }
        state.buildup -= delta * scale;
    }
    if state.broken && state.buildup <= 0.0 {
        state.broken = false;
    }

    let broke_now =
        state.buildup >= shield_health + phase_shield_boost * state.phase_heat && !state.broken;
    if broke_now {
        state.broken = true;
        state.buildup = shield_health;
    }
    if state.hit > 0.0 {
        state.hit -= 1.0 / 5.0 * delta;
    }
    broke_now
}

pub fn force_projector_absorb_explosion(
    state: &mut ForceProjectorState,
    inside_polygon: bool,
    damage: f32,
    crash_damage_multiplier: f32,
) -> bool {
    let absorb = !state.broken && inside_polygon;
    if absorb {
        state.hit = 1.0;
        state.buildup += damage * crash_damage_multiplier;
    }
    absorb
}

pub fn write_force_projector_state<W: Write>(
    write: &mut W,
    state: &ForceProjectorState,
) -> io::Result<()> {
    write.write_all(&[state.broken as u8])?;
    write_f32(write, state.buildup)?;
    write_f32(write, state.radscl)?;
    write_f32(write, state.warmup)?;
    write_f32(write, state.phase_heat)
}

pub fn read_force_projector_state<R: Read>(read: &mut R) -> io::Result<ForceProjectorState> {
    let mut broken = [0; 1];
    read.read_exact(&mut broken)?;
    Ok(ForceProjectorState {
        broken: broken[0] != 0,
        buildup: read_f32(read)?,
        radscl: read_f32(read)?,
        warmup: read_f32(read)?,
        phase_heat: read_f32(read)?,
        hit: 0.0,
    })
}

pub fn regen_projector_heal_amount(
    optional_efficiency: f32,
    optional_multiplier: f32,
    heal_percent: f32,
    edelta: f32,
    block_health: f32,
    missing_health: f32,
) -> f32 {
    let amount =
        lerp(1.0, optional_multiplier, optional_efficiency) * heal_percent * edelta * block_health
            / 100.0;
    amount.min(missing_health)
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
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
    fn wall_door_auto_door_and_mine_helpers_follow_upstream() {
        assert_eq!(wall_collision_hit(0.0), 1.0);
        assert_eq!(wall_draw_hit_decay(1.0, 5.0, false), 0.5);
        assert!(wall_should_lightning(0.25, 0.2));
        assert!(wall_deflects_bullet(10.0, 1.0, true, 20.0, 0.4));
        assert!(!wall_deflects_bullet(10.0, 0.05, true, 20.0, 0.0));
        assert!(wall_reflect_x(6.0, 3.0));

        assert!(door_check_solid(false));
        assert_eq!(door_sense_enabled(true), 1.0);
        assert!(door_can_toggle(false, true, false, true));
        assert!(!door_can_toggle(false, false, false, true));
        assert!(door_tapped_should_configure(false, true, true));
        assert!(!door_tapped_should_configure(true, true, true));

        let mut bytes = Vec::new();
        write_door_state(&mut bytes, DoorState { open: true }).unwrap();
        assert_eq!(
            read_door_state(&mut bytes.as_slice()).unwrap(),
            DoorState { open: true }
        );

        assert!(auto_door_should_open(true));
        assert_eq!(auto_door_trigger_size(2, 8.0, 12.0), 40.0);

        assert!(shock_mine_should_trigger(true, false, true));
        assert_eq!(
            shock_mine_bullet_angles(4, &[1.0, 2.0]),
            vec![1.0, 92.0, 180.0, 270.0]
        );
    }

    #[test]
    fn mend_and_overdrive_projectors_update_and_serialize_java_fields() {
        let mut mend = MendProjectorState {
            charge: 249.95,
            ..MendProjectorState::default()
        };
        let update = mend_projector_update(
            &mut mend, 1.0, 0.5, true, 1.0, 250.0, 60.0, 12.0, 12.0, 50.0,
        );
        assert!(update.fired);
        assert_eq!(mend.charge, 0.0);
        assert_eq!(mend.heat, 0.08);
        assert_eq!(update.real_range, 62.5);
        assert!((update.heal_fraction - 0.126).abs() < 0.00001);

        let mut bytes = Vec::new();
        write_mend_projector_state(&mut bytes, &mend).unwrap();
        assert_eq!(
            read_mend_projector_state(&mut bytes.as_slice())
                .unwrap()
                .phase_heat,
            mend.phase_heat
        );

        let mut over = OverdriveProjectorState {
            charge: 59.95,
            use_progress: 399.5,
            ..OverdriveProjectorState::default()
        };
        let update = overdrive_projector_update(
            &mut over, 1.0, 0.5, true, 1.0, 60.0, 80.0, 20.0, 1.5, 0.75, 400.0,
        );
        assert!(update.applied_boost);
        assert!(update.consumed);
        assert_eq!(update.real_range, 81.0);
        assert!((update.real_boost - 1.5375).abs() < 0.00001);

        let mut bytes = Vec::new();
        write_overdrive_projector_state(&mut bytes, &over).unwrap();
        assert_eq!(
            read_overdrive_projector_state(&mut bytes.as_slice())
                .unwrap()
                .heat,
            over.heat
        );
    }

    #[test]
    fn force_and_regen_projector_helpers_follow_upstream() {
        let mut force = ForceProjectorState {
            broken: true,
            buildup: 0.1,
            ..ForceProjectorState::default()
        };
        let broke = force_projector_update(
            &mut force, 1.0, true, 0.0, 0.0, 1.0, 700.0, 400.0, 1.75, 0.35, 1.5,
        );
        assert!(!broke);
        assert!(!force.broken);
        assert_eq!(force.phase_heat, 0.1);
        assert_eq!(force_projector_real_radius(100.0, 0.5, 80.0, 0.5), 70.0);
        assert_eq!(
            force_projector_shield(false, 700.0, 400.0, 0.5, 100.0),
            800.0
        );
        assert!(force_projector_absorb_explosion(
            &mut force, true, 10.0, 2.0
        ));
        assert_eq!(force.hit, 1.0);

        let mut bytes = Vec::new();
        write_force_projector_state(&mut bytes, &force).unwrap();
        let restored = read_force_projector_state(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.broken, force.broken);
        assert_eq!(restored.buildup, force.buildup);

        assert_eq!(
            regen_projector_heal_amount(0.5, 2.0, 0.2, 1.0, 1000.0, 20.0),
            3.0
        );
        assert_eq!(
            regen_projector_heal_amount(1.0, 2.0, 12.0, 1.0, 1000.0, 20.0),
            20.0
        );
    }
}
