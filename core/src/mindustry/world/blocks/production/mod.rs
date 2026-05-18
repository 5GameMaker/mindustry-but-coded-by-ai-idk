use std::io::{self, Read, Write};

use crate::mindustry::ctype::ContentId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemOutput {
    pub item: ContentId,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidOutput {
    pub liquid: ContentId,
    pub amount: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenericCrafterState {
    pub progress: f32,
    pub total_progress: f32,
    pub warmup: f32,
}

impl Default for GenericCrafterState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            total_progress: 0.0,
            warmup: 0.0,
        }
    }
}

pub fn generic_crafter_should_consume(
    output_items: &[ItemOutput],
    item_capacity: i32,
    item_amounts: &[(ContentId, i32)],
    output_liquids: &[LiquidOutput],
    liquid_capacity: f32,
    liquid_amounts: &[(ContentId, f32)],
    ignore_liquid_fullness: bool,
    dump_extra_liquid: bool,
    enabled: bool,
) -> bool {
    for output in output_items {
        let current = item_amounts
            .iter()
            .find_map(|(item, amount)| (*item == output.item).then_some(*amount))
            .unwrap_or(0);
        if current + output.amount > item_capacity {
            return false;
        }
    }

    if !ignore_liquid_fullness && !output_liquids.is_empty() {
        let mut all_full = true;
        for output in output_liquids {
            let current = liquid_amounts
                .iter()
                .find_map(|(liquid, amount)| (*liquid == output.liquid).then_some(*amount))
                .unwrap_or(0.0);
            if current >= liquid_capacity - 0.001 {
                if !dump_extra_liquid {
                    return false;
                }
            } else {
                all_full = false;
            }
        }
        if all_full {
            return false;
        }
    }

    enabled
}

pub fn generic_crafter_progress_increase(
    base_increase: f32,
    output_liquids: &[LiquidOutput],
    liquid_capacity: f32,
    liquid_amounts: &[(ContentId, f32)],
    edelta: f32,
    ignore_liquid_fullness: bool,
    dump_extra_liquid: bool,
) -> f32 {
    if ignore_liquid_fullness || output_liquids.is_empty() {
        return base_increase;
    }

    let mut scaling = 1.0f32;
    let mut max = 0.0f32;
    for output in output_liquids {
        let current = liquid_amounts
            .iter()
            .find_map(|(liquid, amount)| (*liquid == output.liquid).then_some(*amount))
            .unwrap_or(0.0);
        let value = (liquid_capacity - current) / (output.amount * edelta);
        scaling = scaling.min(value);
        max = max.max(value);
    }

    base_increase
        * if dump_extra_liquid {
            max.min(1.0)
        } else {
            scaling
        }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenericCrafterUpdate {
    pub crafted: bool,
    pub progress: f32,
    pub total_progress: f32,
    pub warmup: f32,
}

pub fn generic_crafter_update(
    state: &mut GenericCrafterState,
    efficiency: f32,
    progress_increase: f32,
    warmup_speed: f32,
    delta: f32,
) -> GenericCrafterUpdate {
    if efficiency > 0.0 {
        state.progress += progress_increase;
        state.warmup = approach_delta(state.warmup, 1.0, warmup_speed);
    } else {
        state.warmup = approach_delta(state.warmup, 0.0, warmup_speed);
    }

    state.total_progress += state.warmup * delta;
    let crafted = state.progress >= 1.0;
    if crafted {
        state.progress %= 1.0;
    }

    GenericCrafterUpdate {
        crafted,
        progress: state.progress,
        total_progress: state.total_progress,
        warmup: state.warmup,
    }
}

pub fn write_generic_crafter_state<W: Write>(
    write: &mut W,
    state: &GenericCrafterState,
    legacy_read_warmup: bool,
) -> io::Result<()> {
    write_f32(write, state.progress)?;
    write_f32(write, state.warmup)?;
    if legacy_read_warmup {
        write_f32(write, 0.0)?;
    }
    Ok(())
}

pub fn read_generic_crafter_state<R: Read>(
    read: &mut R,
    legacy_read_warmup: bool,
) -> io::Result<GenericCrafterState> {
    let progress = read_f32(read)?;
    let warmup = read_f32(read)?;
    if legacy_read_warmup {
        let _ = read_f32(read)?;
    }
    Ok(GenericCrafterState {
        progress,
        warmup,
        total_progress: 0.0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeparatorState {
    pub progress: f32,
    pub total_progress: f32,
    pub warmup: f32,
    pub seed: i32,
}

impl Default for SeparatorState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            total_progress: 0.0,
            warmup: 0.0,
            seed: 0,
        }
    }
}

pub fn separator_should_consume(
    total_items: i32,
    consumed_inputs: &[(ContentId, i32)],
    item_amounts: &[(ContentId, i32)],
    item_capacity: i32,
    enabled: bool,
) -> bool {
    let mut total = total_items;
    for (item, _) in consumed_inputs {
        total -= item_amounts
            .iter()
            .find_map(|(id, amount)| (*id == *item).then_some(*amount))
            .unwrap_or(0);
    }
    total < item_capacity && enabled
}

pub fn separator_weighted_result_index(results: &[ItemOutput], pick: i32) -> Option<usize> {
    let mut count = 0;
    for (index, stack) in results.iter().enumerate() {
        if pick >= count && pick < count + stack.amount {
            return Some(index);
        }
        count += stack.amount;
    }
    None
}

pub fn write_separator_state<W: Write>(write: &mut W, state: &SeparatorState) -> io::Result<()> {
    write_f32(write, state.progress)?;
    write_f32(write, state.warmup)?;
    write_i32(write, state.seed)
}

pub fn read_separator_state<R: Read>(read: &mut R, revision: u8) -> io::Result<SeparatorState> {
    let progress = read_f32(read)?;
    let warmup = read_f32(read)?;
    let seed = if revision == 1 { read_i32(read)? } else { 0 };
    Ok(SeparatorState {
        progress,
        warmup,
        seed,
        total_progress: 0.0,
    })
}

pub fn incinerator_update_heat(heat: f32, efficiency: f32) -> f32 {
    approach_delta(heat, efficiency, 0.04)
}

pub fn incinerator_accept_item(heat: f32, enabled: bool) -> bool {
    heat > 0.5 && enabled
}

pub fn incinerator_accept_liquid(heat: f32, liquid_incinerable: bool, enabled: bool) -> bool {
    heat > 0.5 && liquid_incinerable && enabled
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncineratorStatus {
    LogicDisable,
    Active,
    NoInput,
}

pub fn incinerator_status(enabled: bool, heat: f32) -> IncineratorStatus {
    if !enabled {
        IncineratorStatus::LogicDisable
    } else if heat > 0.5 {
        IncineratorStatus::Active
    } else {
        IncineratorStatus::NoInput
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrillState {
    pub progress: f32,
    pub warmup: f32,
    pub time_drilled: f32,
    pub last_drill_speed: f32,
}

impl Default for DrillState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            warmup: 0.0,
            time_drilled: 0.0,
            last_drill_speed: 0.0,
        }
    }
}

pub fn drill_time(
    drill_time: f32,
    hardness_drill_multiplier: f32,
    hardness: i32,
    multiplier: f32,
) -> f32 {
    (drill_time + hardness_drill_multiplier * hardness as f32) / multiplier
}

pub fn drill_should_consume(
    items_total: i32,
    item_capacity: i32,
    enabled: bool,
    has_dominant_item: bool,
) -> bool {
    items_total < item_capacity && enabled && has_dominant_item
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrillUpdate {
    pub produced: i32,
    pub progress: f32,
    pub warmup: f32,
    pub last_drill_speed: f32,
}

pub fn drill_update(
    state: &mut DrillState,
    items_total: i32,
    item_capacity: i32,
    dominant_items: i32,
    efficiency: f32,
    optional_efficiency: f32,
    liquid_boost_intensity: f32,
    warmup_speed: f32,
    delay: f32,
    delta: f32,
) -> DrillUpdate {
    state.time_drilled += state.warmup * delta;
    let mut produced = 0;

    if items_total < item_capacity && dominant_items > 0 && efficiency > 0.0 {
        let speed = lerp(1.0, liquid_boost_intensity, optional_efficiency) * efficiency;
        state.last_drill_speed = (speed * dominant_items as f32 * state.warmup) / delay;
        state.warmup = approach_delta(state.warmup, speed, warmup_speed);
        state.progress += delta * dominant_items as f32 * speed * state.warmup;

        if state.progress >= delay && items_total < item_capacity {
            produced = (state.progress / delay) as i32;
            state.progress %= delay;
        }
    } else {
        state.last_drill_speed = 0.0;
        state.warmup = approach_delta(state.warmup, 0.0, warmup_speed);
    }

    DrillUpdate {
        produced,
        progress: state.progress,
        warmup: state.warmup,
        last_drill_speed: state.last_drill_speed,
    }
}

pub fn write_drill_state<W: Write>(write: &mut W, state: &DrillState) -> io::Result<()> {
    write_f32(write, state.progress)?;
    write_f32(write, state.warmup)
}

pub fn read_drill_state<R: Read>(read: &mut R, revision: u8) -> io::Result<DrillState> {
    if revision >= 1 {
        Ok(DrillState {
            progress: read_f32(read)?,
            warmup: read_f32(read)?,
            ..Default::default()
        })
    } else {
        Ok(DrillState::default())
    }
}

pub fn pump_should_consume(
    liquid_drop: Option<ContentId>,
    stored_amount: f32,
    liquid_capacity: f32,
    enabled: bool,
) -> bool {
    liquid_drop.is_some() && stored_amount < liquid_capacity - 0.01 && enabled
}

pub fn pump_amount_to_add(
    liquid_capacity: f32,
    stored_amount: f32,
    floor_amount: f32,
    pump_amount: f32,
    edelta: f32,
) -> f32 {
    (liquid_capacity - stored_amount)
        .min(floor_amount * pump_amount * edelta)
        .max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidPumpUpdate {
    pub pumped: f32,
    pub last_pump: f32,
    pub warmup: f32,
}

pub fn solid_pump_update(
    stored_amount: f32,
    liquid_capacity: f32,
    valid_tiles: f32,
    boost: f32,
    attribute_env: f32,
    pump_amount: f32,
    efficiency: f32,
    delta: f32,
    warmup: f32,
) -> SolidPumpUpdate {
    let fraction = (valid_tiles + boost + attribute_env).max(0.0);
    if efficiency > 0.0 && stored_amount < liquid_capacity - 0.001 {
        let max_pump = (liquid_capacity - stored_amount)
            .min(pump_amount * delta * fraction * efficiency)
            .max(0.0);
        SolidPumpUpdate {
            pumped: max_pump,
            last_pump: if delta == 0.0 { 0.0 } else { max_pump / delta },
            warmup: lerp_delta(warmup, 1.0, 0.02),
        }
    } else {
        SolidPumpUpdate {
            pumped: 0.0,
            last_pump: 0.0,
            warmup: lerp_delta(warmup, 0.0, 0.02),
        }
    }
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
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

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
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
    fn generic_crafter_should_consume_and_progress_respect_outputs() {
        assert!(!generic_crafter_should_consume(
            &[ItemOutput { item: 1, amount: 2 }],
            10,
            &[(1, 9)],
            &[],
            0.0,
            &[],
            false,
            true,
            true,
        ));
        assert!(!generic_crafter_should_consume(
            &[],
            10,
            &[],
            &[LiquidOutput {
                liquid: 2,
                amount: 1.0,
            }],
            10.0,
            &[(2, 10.0)],
            false,
            true,
            true,
        ));
        assert!(generic_crafter_should_consume(
            &[],
            10,
            &[],
            &[LiquidOutput {
                liquid: 2,
                amount: 1.0,
            }],
            10.0,
            &[(2, 9.0)],
            false,
            true,
            true,
        ));

        assert_eq!(
            generic_crafter_progress_increase(
                0.5,
                &[LiquidOutput {
                    liquid: 2,
                    amount: 2.0,
                }],
                10.0,
                &[(2, 9.0)],
                1.0,
                false,
                false,
            ),
            0.25
        );

        let mut state = GenericCrafterState::default();
        let update = generic_crafter_update(&mut state, 1.0, 1.25, 0.019, 1.0);
        assert!(update.crafted);
        assert_eq!(state.progress, 0.25);
        assert_eq!(state.warmup, 0.019);

        let mut bytes = Vec::new();
        write_generic_crafter_state(&mut bytes, &state, true).unwrap();
        assert_eq!(
            read_generic_crafter_state(&mut bytes.as_slice(), true)
                .unwrap()
                .progress,
            0.25
        );
    }

    #[test]
    fn separator_should_consume_weighted_pick_and_roundtrip_seed() {
        assert!(separator_should_consume(20, &[(1, 5)], &[(1, 5)], 16, true));
        assert!(!separator_should_consume(
            20,
            &[(1, 5)],
            &[(1, 4)],
            16,
            true
        ));

        let results = [
            ItemOutput { item: 1, amount: 2 },
            ItemOutput { item: 2, amount: 3 },
        ];
        assert_eq!(separator_weighted_result_index(&results, 0), Some(0));
        assert_eq!(separator_weighted_result_index(&results, 2), Some(1));
        assert_eq!(separator_weighted_result_index(&results, 5), None);

        let state = SeparatorState {
            progress: 0.5,
            warmup: 0.25,
            seed: 123,
            total_progress: 0.0,
        };
        let mut bytes = Vec::new();
        write_separator_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_separator_state(&mut bytes.as_slice(), 1).unwrap(),
            state
        );
    }

    #[test]
    fn incinerator_heat_acceptance_and_status_follow_java_thresholds() {
        assert_eq!(incinerator_update_heat(0.0, 1.0), 0.04);
        assert!(!incinerator_accept_item(0.5, true));
        assert!(incinerator_accept_item(0.51, true));
        assert!(incinerator_accept_liquid(0.51, true, true));
        assert!(!incinerator_accept_liquid(0.51, false, true));
        assert_eq!(
            incinerator_status(false, 1.0),
            IncineratorStatus::LogicDisable
        );
        assert_eq!(incinerator_status(true, 0.51), IncineratorStatus::Active);
        assert_eq!(incinerator_status(true, 0.5), IncineratorStatus::NoInput);
    }

    #[test]
    fn drill_time_update_and_state_roundtrip_match_upstream_formulae() {
        assert_eq!(drill_time(300.0, 50.0, 2, 2.0), 200.0);
        assert!(drill_should_consume(9, 10, true, true));
        assert!(!drill_should_consume(10, 10, true, true));

        let mut state = DrillState::default();
        let update = drill_update(&mut state, 0, 100, 2, 1.0, 1.0, 1.6, 0.015, 10.0, 100.0);
        assert_eq!(update.produced, 0);
        assert_eq!(state.warmup, 0.015);
        let update = drill_update(&mut state, 0, 100, 2, 1.0, 1.0, 1.6, 0.015, 10.0, 100.0);
        assert!(update.produced > 0);

        let mut bytes = Vec::new();
        write_drill_state(&mut bytes, &state).unwrap();
        let restored = read_drill_state(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(restored.progress, state.progress);
        assert_eq!(restored.warmup, state.warmup);
    }

    #[test]
    fn pump_and_solid_pump_outputs_follow_capacity_and_efficiency() {
        assert!(pump_should_consume(Some(1), 9.98, 10.0, true));
        assert!(!pump_should_consume(Some(1), 9.99, 10.0, true));
        assert_eq!(pump_amount_to_add(10.0, 9.0, 3.0, 0.2, 10.0), 1.0);
        assert_eq!(pump_amount_to_add(10.0, 0.0, 3.0, 0.2, 10.0), 6.0);

        let update = solid_pump_update(0.0, 100.0, 1.0, 0.5, 0.0, 0.2, 1.0, 10.0, 0.0);
        assert_eq!(
            update,
            SolidPumpUpdate {
                pumped: 3.0,
                last_pump: 0.3,
                warmup: 0.02
            }
        );
        let idle = solid_pump_update(100.0, 100.0, 1.0, 0.5, 0.0, 0.2, 1.0, 10.0, 0.5);
        assert_eq!(idle.pumped, 0.0);
        assert_eq!(idle.last_pump, 0.0);
        assert_eq!(idle.warmup, 0.49);
    }
}
