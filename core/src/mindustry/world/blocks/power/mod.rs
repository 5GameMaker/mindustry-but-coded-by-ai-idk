use std::io::{self, Read, Write};

pub fn battery_status(power_status: f32) -> PowerBlockStatus {
    if nearly(power_status, 0.0, 0.001) {
        PowerBlockStatus::NoInput
    } else if nearly(power_status, 1.0, 0.001) {
        PowerBlockStatus::Active
    } else {
        PowerBlockStatus::NoOutput
    }
}

pub fn battery_overwrite_status(
    current_status: f32,
    incoming_capacity: f32,
    incoming_status: f32,
    battery_capacity: f32,
) -> f32 {
    (current_status + incoming_capacity * incoming_status / battery_capacity).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerBlockStatus {
    NoInput,
    Active,
    NoOutput,
    LogicDisable,
}

pub fn power_node_link_valid(
    same_building: bool,
    link_exists: bool,
    link_has_power: bool,
    link_connected_power: bool,
    same_team: bool,
    same_block_connection: bool,
    same_block: bool,
    overlaps_either_range: bool,
    check_max_nodes: bool,
    link_is_power_node: bool,
    link_count: i32,
    link_max_nodes: i32,
    already_linked: bool,
) -> bool {
    if same_building
        || !link_exists
        || !link_has_power
        || !link_connected_power
        || !same_team
        || (same_block_connection && !same_block)
        || !overlaps_either_range
    {
        return false;
    }
    if check_max_nodes && link_is_power_node {
        link_count < link_max_nodes || already_linked
    } else {
        true
    }
}

pub fn power_node_should_draw_link(
    link_valid: bool,
    link_is_power_node: bool,
    link_id: i32,
    self_id: i32,
) -> bool {
    link_valid && !(link_is_power_node && link_id >= self_id)
}

pub fn power_diode_bar(stored: f32, capacity: f32) -> f32 {
    if capacity == 0.0 {
        0.0
    } else {
        stored / capacity
    }
}

pub fn power_diode_transfer_amount(
    back_stored: f32,
    back_capacity: f32,
    front_stored: f32,
    front_capacity: f32,
) -> f32 {
    if back_capacity <= 0.0 || front_capacity <= 0.0 {
        return 0.0;
    }
    if back_stored / back_capacity <= front_stored / front_capacity {
        return 0.0;
    }
    let target_percentage = (front_stored + back_stored) / (front_capacity + back_capacity);
    ((target_percentage * front_capacity - front_stored) / 2.0)
        .clamp(0.0, front_capacity - front_stored)
}

pub fn solar_generator_efficiency(
    enabled: bool,
    solar_multiplier: f32,
    light_env: f32,
    lighting: bool,
    ambient_alpha: f32,
) -> f32 {
    if enabled {
        solar_multiplier * (light_env + if lighting { 1.0 - ambient_alpha } else { 1.0 }).max(0.0)
    } else {
        0.0
    }
}

pub fn thermal_generator_can_place(attribute_sum: f32, min_efficiency: f32) -> bool {
    attribute_sum > min_efficiency
}

pub fn thermal_generator_efficiency(attribute_sum: f32, env: f32) -> f32 {
    attribute_sum + env
}

pub fn thermal_generator_liquid_added(
    production_efficiency: f32,
    delta: f32,
    output_amount: f32,
    liquid_capacity: f32,
    stored: f32,
) -> f32 {
    (production_efficiency * delta * output_amount)
        .min(liquid_capacity - stored)
        .max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeGeneratorState {
    pub warmup: f32,
    pub total_time: f32,
    pub efficiency_multiplier: f32,
    pub item_duration_multiplier: f32,
    pub generate_time: f32,
}

impl Default for ConsumeGeneratorState {
    fn default() -> Self {
        Self {
            warmup: 0.0,
            total_time: 0.0,
            efficiency_multiplier: 1.0,
            item_duration_multiplier: 1.0,
            generate_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeGeneratorUpdate {
    pub should_consume: bool,
    pub liquid_added: f32,
    pub exploded: bool,
    pub production_efficiency: f32,
    pub generate_time: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn consume_generator_update(
    state: &mut ConsumeGeneratorState,
    efficiency: f32,
    warmup_speed: f32,
    delta: f32,
    has_items: bool,
    item_duration: f32,
    output_liquid_amount: Option<f32>,
    liquid_capacity: f32,
    stored_liquid: f32,
    explode_on_full: bool,
) -> ConsumeGeneratorUpdate {
    let valid = efficiency > 0.0;
    state.warmup = lerp_delta(state.warmup, if valid { 1.0 } else { 0.0 }, warmup_speed);
    let production_efficiency = efficiency * state.efficiency_multiplier;
    state.total_time += state.warmup * delta;

    let should_consume = has_items && valid && state.generate_time <= 0.0;
    if should_consume {
        state.generate_time = 1.0;
    }

    let liquid_added = output_liquid_amount
        .map(|amount| {
            thermal_generator_liquid_added(
                production_efficiency,
                delta,
                amount,
                liquid_capacity,
                stored_liquid,
            )
        })
        .unwrap_or(0.0);
    let after_liquid = stored_liquid + liquid_added;
    let exploded =
        explode_on_full && output_liquid_amount.is_some() && after_liquid >= liquid_capacity - 0.01;

    state.generate_time -= delta / (item_duration * state.item_duration_multiplier);

    ConsumeGeneratorUpdate {
        should_consume,
        liquid_added,
        exploded,
        production_efficiency,
        generate_time: state.generate_time,
    }
}

pub fn consume_generator_trigger_valid(generate_time: f32) -> bool {
    generate_time > 0.0
}

pub fn write_consume_generator_state<W: Write>(
    write: &mut W,
    state: &ConsumeGeneratorState,
) -> io::Result<()> {
    write_f32(write, state.warmup)?;
    write_f32(write, state.total_time)?;
    write_f32(write, state.efficiency_multiplier)?;
    write_f32(write, state.item_duration_multiplier)?;
    write_f32(write, state.generate_time)
}

pub fn read_consume_generator_state<R: Read>(read: &mut R) -> io::Result<ConsumeGeneratorState> {
    Ok(ConsumeGeneratorState {
        warmup: read_f32(read)?,
        total_time: read_f32(read)?,
        efficiency_multiplier: read_f32(read)?,
        item_duration_multiplier: read_f32(read)?,
        generate_time: read_f32(read)?,
    })
}

fn nearly(value: f32, target: f32, epsilon: f32) -> bool {
    (value - target).abs() <= epsilon
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_status_overwrite_and_node_links_follow_upstream_rules() {
        assert_eq!(battery_status(0.0), PowerBlockStatus::NoInput);
        assert_eq!(battery_status(1.0), PowerBlockStatus::Active);
        assert_eq!(battery_status(0.5), PowerBlockStatus::NoOutput);
        assert_eq!(battery_overwrite_status(0.2, 50.0, 0.5, 100.0), 0.45);

        assert!(power_node_link_valid(
            false, true, true, true, true, false, false, true, true, true, 2, 3, false
        ));
        assert!(!power_node_link_valid(
            true, true, true, true, true, false, false, true, true, true, 2, 3, false
        ));
        assert!(!power_node_link_valid(
            false, true, true, true, true, true, false, true, true, true, 2, 3, false
        ));
        assert!(power_node_link_valid(
            false, true, true, true, true, false, false, true, true, true, 3, 3, true
        ));
        assert!(!power_node_link_valid(
            false, true, true, true, true, false, false, true, true, true, 3, 3, false
        ));
        assert!(power_node_should_draw_link(true, true, 1, 2));
        assert!(!power_node_should_draw_link(true, true, 3, 2));
    }

    #[test]
    fn diode_solar_and_thermal_formulae_match_java_shells() {
        assert_eq!(power_diode_bar(5.0, 10.0), 0.5);
        assert_eq!(power_diode_bar(5.0, 0.0), 0.0);
        assert_eq!(power_diode_transfer_amount(80.0, 100.0, 20.0, 100.0), 15.0);
        assert_eq!(power_diode_transfer_amount(20.0, 100.0, 80.0, 100.0), 0.0);

        assert_eq!(solar_generator_efficiency(true, 2.0, 0.1, true, 0.25), 1.7);
        assert_eq!(solar_generator_efficiency(true, 2.0, 0.1, false, 0.25), 2.2);
        assert_eq!(
            solar_generator_efficiency(false, 2.0, 0.1, false, 0.25),
            0.0
        );

        assert!(thermal_generator_can_place(0.1, 0.0));
        assert!(!thermal_generator_can_place(0.0, 0.0));
        assert_eq!(thermal_generator_efficiency(0.5, 0.25), 0.75);
        assert_eq!(thermal_generator_liquid_added(2.0, 3.0, 0.1, 1.0, 0.5), 0.5);
    }

    #[test]
    fn consume_generator_update_and_state_roundtrip_follow_runtime_order() {
        let mut state = ConsumeGeneratorState::default();
        let update = consume_generator_update(
            &mut state,
            1.0,
            0.05,
            1.0,
            true,
            120.0,
            Some(0.2),
            10.0,
            9.9,
            true,
        );
        assert!(update.should_consume);
        assert_eq!(state.warmup, 0.05);
        assert!((update.liquid_added - 0.1).abs() < 0.00001);
        assert!(update.exploded);
        assert!(consume_generator_trigger_valid(update.generate_time));

        let mut bytes = Vec::new();
        write_consume_generator_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_consume_generator_state(&mut bytes.as_slice()).unwrap(),
            state
        );
    }
}
