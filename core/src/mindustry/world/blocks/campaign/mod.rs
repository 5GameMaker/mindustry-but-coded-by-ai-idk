use std::collections::BTreeMap;
use std::io::{self, Read, Write};

pub const DEFAULT_LAUNCH_PAYLOAD_LIFETIME: f32 = 120.0;
pub const DEFAULT_LANDING_ARRIVAL_DURATION: f32 = 150.0;
pub const DEFAULT_LANDING_COOLDOWN_TIME: f32 = 150.0;
pub const DEFAULT_ACCELERATOR_LAUNCH_DURATION: f32 = 120.0;
pub const DEFAULT_ACCELERATOR_CHARGE_DURATION: f32 = 220.0;
pub const DEFAULT_ACCELERATOR_BUILD_DURATION: f32 = 120.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchPadState {
    pub launch_counter: f32,
}

impl Default for LaunchPadState {
    fn default() -> Self {
        Self {
            launch_counter: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchPadStep {
    pub launched: bool,
    pub launch_counter: f32,
}

pub fn launch_pad_should_consume(launch_counter: f32, launch_time: f32) -> bool {
    launch_counter < launch_time
}

pub fn launch_pad_progress(launch_counter: f32, launch_time: f32) -> f32 {
    clamp01(launch_counter / launch_time)
}

pub fn launch_pad_accept_item(
    item_capacity: i32,
    accept_multiple_items: bool,
    total_items: i32,
    first_item: Option<i16>,
    item: i16,
) -> bool {
    total_items < item_capacity
        && (accept_multiple_items || total_items == 0 || first_item == Some(item))
}

pub fn launch_pad_update(
    state: &mut LaunchPadState,
    launch_time: f32,
    item_capacity: i32,
    total_items: i32,
    edelta: f32,
) -> LaunchPadStep {
    state.launch_counter += edelta;
    if state.launch_counter >= launch_time && total_items >= item_capacity {
        state.launch_counter = 0.0;
        LaunchPadStep {
            launched: true,
            launch_counter: 0.0,
        }
    } else {
        LaunchPadStep {
            launched: false,
            launch_counter: state.launch_counter,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LandingPadState {
    pub config: Option<i16>,
    pub priority: i32,
    pub cooldown: f32,
    pub arriving: Option<i16>,
    pub arriving_timer: f32,
    pub liquid_removed: f32,
}

impl Default for LandingPadState {
    fn default() -> Self {
        Self {
            config: None,
            priority: 0,
            cooldown: 0.0,
            arriving: None,
            arriving_timer: 0.0,
            liquid_removed: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LandingPadArrivalStep {
    pub removed_liquid: f32,
    pub finished_item: Option<i16>,
    pub produced_amount: i32,
}

pub fn landing_pad_accept_item() -> bool {
    false
}

pub fn landing_pad_handle_landing(state: &mut LandingPadState) -> bool {
    let Some(config) = state.config else {
        return false;
    };
    state.cooldown = 1.0;
    state.arriving = Some(config);
    state.arriving_timer = 0.0;
    state.liquid_removed = 0.0;
    true
}

pub fn landing_pad_update_arrival(
    state: &mut LandingPadState,
    arrival_duration: f32,
    consume_liquid_amount: f32,
    item_capacity: i32,
    delta: f32,
) -> LandingPadArrivalStep {
    let Some(arriving) = state.arriving else {
        return LandingPadArrivalStep {
            removed_liquid: 0.0,
            finished_item: None,
            produced_amount: 0,
        };
    };

    state.arriving_timer += delta / arrival_duration;
    let removed_liquid = (consume_liquid_amount / arrival_duration * delta)
        .min(consume_liquid_amount - state.liquid_removed);
    state.liquid_removed += removed_liquid;

    if state.arriving_timer >= 1.0 {
        let leftover = consume_liquid_amount - state.liquid_removed;
        state.arriving = None;
        state.arriving_timer = 0.0;
        state.liquid_removed = 0.0;
        LandingPadArrivalStep {
            removed_liquid: removed_liquid + leftover,
            finished_item: Some(arriving),
            produced_amount: item_capacity,
        }
    } else {
        LandingPadArrivalStep {
            removed_liquid,
            finished_item: None,
            produced_amount: 0,
        }
    }
}

pub fn landing_pad_update_cooldown(
    state: &mut LandingPadState,
    cooldown_time: f32,
    delta: f32,
) -> f32 {
    if state.arriving.is_none() {
        state.cooldown = clamp01(state.cooldown - delta / cooldown_time);
    }
    state.cooldown
}

pub fn landing_pad_ready_to_queue(
    state: &LandingPadState,
    efficiency: f32,
    total_items: i32,
    is_fake: bool,
    is_campaign_non_legacy: bool,
    import_rate_positive: bool,
    import_cooldown: f32,
) -> bool {
    state.config.is_some()
        && (is_fake || is_campaign_non_legacy)
        && state.cooldown <= 0.0
        && efficiency > 0.0
        && total_items == 0
        && (is_fake || (import_rate_positive && import_cooldown >= 1.0))
}

#[derive(Debug, Clone, PartialEq)]
pub struct AcceleratorState {
    pub progress: f32,
    pub launching: bool,
}

impl Default for AcceleratorState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            launching: false,
        }
    }
}

pub fn accelerator_update_progress(
    state: &mut AcceleratorState,
    delta: f32,
    efficiency: f32,
    build_duration: f32,
) -> f32 {
    if efficiency >= 0.0 {
        state.progress = (state.progress + delta * efficiency / build_duration).min(1.0);
    }
    state.progress
}

pub fn accelerator_is_core_built(progress: f32) -> bool {
    progress >= 1.0
}

pub fn accelerator_can_launch(
    valid: bool,
    net_client: bool,
    campaign: bool,
    efficiency: f32,
    battery_stored: f32,
    power_buffer_requirement: f32,
    progress: f32,
    launching: bool,
) -> bool {
    valid
        && !net_client
        && campaign
        && efficiency > 0.0
        && battery_stored >= power_buffer_requirement - 0.00001
        && progress >= 1.0
        && !launching
}

pub fn accelerator_maximum_accepted(
    capacities: &BTreeMap<i16, i32>,
    item: i16,
    core_built: bool,
    launch_block_item_capacity: i32,
) -> i32 {
    capacities.get(&item).copied().unwrap_or(0)
        + if core_built {
            launch_block_item_capacity
        } else {
            0
        }
}

pub fn accelerator_accept_item(
    capacities: &BTreeMap<i16, i32>,
    item: i16,
    current_amount: i32,
    core_built: bool,
    launch_block_item_capacity: i32,
) -> bool {
    current_amount
        < accelerator_maximum_accepted(capacities, item, core_built, launch_block_item_capacity)
}

pub fn accelerator_consume_launch(state: &mut AcceleratorState) {
    state.progress = 0.0;
    state.launching = true;
}

pub fn write_launch_pad_state<W: Write>(write: &mut W, state: &LaunchPadState) -> io::Result<()> {
    write_f32(write, state.launch_counter)
}

pub fn read_launch_pad_state<R: Read>(read: &mut R, revision: u8) -> io::Result<LaunchPadState> {
    if revision >= 1 {
        Ok(LaunchPadState {
            launch_counter: read_f32(read)?,
        })
    } else {
        Ok(LaunchPadState::default())
    }
}

pub fn write_landing_pad_state<W: Write>(write: &mut W, state: &LandingPadState) -> io::Result<()> {
    write_item_id(write, state.config)?;
    write_i32(write, state.priority)?;
    write_f32(write, state.cooldown)?;
    write_item_id(write, state.arriving)?;
    write_f32(write, state.arriving_timer)?;
    write_f32(write, state.liquid_removed)
}

pub fn read_landing_pad_state<R: Read>(read: &mut R, revision: u8) -> io::Result<LandingPadState> {
    let mut state = LandingPadState {
        config: read_item_id(read)?,
        priority: read_i32(read)?,
        cooldown: read_f32(read)?,
        ..LandingPadState::default()
    };

    if revision >= 1 {
        state.arriving = read_item_id(read)?;
        state.arriving_timer = read_f32(read)?;
        state.liquid_removed = read_f32(read)?;
    }

    Ok(state)
}

pub fn write_accelerator_state<W: Write>(
    write: &mut W,
    state: &AcceleratorState,
) -> io::Result<()> {
    write_f32(write, state.progress)
}

pub fn read_accelerator_state<R: Read>(read: &mut R, revision: u8) -> io::Result<AcceleratorState> {
    if revision >= 1 {
        Ok(AcceleratorState {
            progress: read_f32(read)?,
            launching: false,
        })
    } else {
        Ok(AcceleratorState::default())
    }
}

fn read_item_id<R: Read>(read: &mut R) -> io::Result<Option<i16>> {
    let id = read_i16(read)?;
    Ok((id != -1).then_some(id))
}

fn write_item_id<W: Write>(write: &mut W, item: Option<i16>) -> io::Result<()> {
    write_i16(write, item.unwrap_or(-1))
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
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

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_pad_accepts_single_item_type_unless_multiple_enabled() {
        assert!(launch_pad_accept_item(100, false, 0, None, 1));
        assert!(launch_pad_accept_item(100, false, 20, Some(1), 1));
        assert!(!launch_pad_accept_item(100, false, 20, Some(1), 2));
        assert!(launch_pad_accept_item(100, true, 20, Some(1), 2));
        assert!(!launch_pad_accept_item(100, true, 100, Some(1), 1));
    }

    #[test]
    fn launch_pad_updates_counter_and_resets_on_launch() {
        let mut state = LaunchPadState {
            launch_counter: 1199.0,
        };
        assert!(launch_pad_should_consume(state.launch_counter, 1200.0));
        let step = launch_pad_update(&mut state, 1200.0, 100, 99, 1.0);
        assert!(!step.launched);
        assert_eq!(step.launch_counter, 1200.0);
        assert!(!launch_pad_should_consume(state.launch_counter, 1200.0));

        let step = launch_pad_update(&mut state, 1200.0, 100, 100, 0.1);
        assert!(step.launched);
        assert_eq!(state.launch_counter, 0.0);
        assert_eq!(launch_pad_progress(600.0, 1200.0), 0.5);
    }

    #[test]
    fn landing_pad_landing_and_arrival_match_java_state_changes() {
        let mut state = LandingPadState {
            config: Some(7),
            ..Default::default()
        };
        assert!(!landing_pad_accept_item());
        assert!(landing_pad_handle_landing(&mut state));
        assert_eq!(state.cooldown, 1.0);
        assert_eq!(state.arriving, Some(7));

        let first = landing_pad_update_arrival(&mut state, 150.0, 1500.0, 100, 75.0);
        assert_eq!(first.removed_liquid, 750.0);
        assert_eq!(first.finished_item, None);
        assert_eq!(state.arriving_timer, 0.5);

        let second = landing_pad_update_arrival(&mut state, 150.0, 1500.0, 100, 75.0);
        assert_eq!(second.removed_liquid, 750.0);
        assert_eq!(second.finished_item, Some(7));
        assert_eq!(second.produced_amount, 100);
        assert_eq!(state.arriving, None);
        assert_eq!(state.arriving_timer, 0.0);
    }

    #[test]
    fn landing_pad_cooldown_and_queue_conditions_follow_upstream() {
        let mut state = LandingPadState {
            config: Some(1),
            cooldown: 1.0,
            ..Default::default()
        };
        assert_eq!(landing_pad_update_cooldown(&mut state, 150.0, 75.0), 0.5);
        assert!(!landing_pad_ready_to_queue(
            &state, 1.0, 0, false, true, true, 1.0
        ));
        state.cooldown = 0.0;
        assert!(landing_pad_ready_to_queue(
            &state, 1.0, 0, false, true, true, 1.0
        ));
        assert!(landing_pad_ready_to_queue(
            &state, 1.0, 0, true, false, false, 0.0
        ));
    }

    #[test]
    fn accelerator_progress_launch_and_acceptance_match_upstream() {
        let mut state = AcceleratorState::default();
        assert_eq!(
            accelerator_update_progress(&mut state, 60.0, 1.0, DEFAULT_ACCELERATOR_BUILD_DURATION),
            0.5
        );
        assert!(!accelerator_is_core_built(state.progress));
        accelerator_update_progress(&mut state, 60.0, 1.0, DEFAULT_ACCELERATOR_BUILD_DURATION);
        assert!(accelerator_is_core_built(state.progress));

        assert!(accelerator_can_launch(
            true,
            false,
            true,
            1.0,
            1_000_000.0,
            1_000_000.0,
            state.progress,
            false
        ));
        assert!(!accelerator_can_launch(
            true,
            false,
            true,
            1.0,
            999_999.0,
            1_000_000.0,
            state.progress,
            false
        ));

        let mut capacities = BTreeMap::new();
        capacities.insert(3, 8000);
        assert_eq!(
            accelerator_maximum_accepted(&capacities, 3, false, 13000),
            8000
        );
        assert_eq!(
            accelerator_maximum_accepted(&capacities, 3, true, 13000),
            21000
        );
        assert!(accelerator_accept_item(&capacities, 3, 20999, true, 13000));
        assert!(!accelerator_accept_item(&capacities, 3, 21000, true, 13000));

        accelerator_consume_launch(&mut state);
        assert_eq!(state.progress, 0.0);
        assert!(state.launching);
    }

    #[test]
    fn campaign_launch_and_accelerator_state_codecs_follow_revision_one_payloads() {
        let launch = LaunchPadState {
            launch_counter: 42.5,
        };
        let mut bytes = Vec::new();
        write_launch_pad_state(&mut bytes, &launch).unwrap();
        assert_eq!(
            read_launch_pad_state(&mut bytes.as_slice(), 1).unwrap(),
            launch
        );
        assert_eq!(
            read_launch_pad_state(&mut [].as_slice(), 0).unwrap(),
            LaunchPadState::default()
        );

        let accelerator = AcceleratorState {
            progress: 0.75,
            launching: true,
        };
        let mut bytes = Vec::new();
        write_accelerator_state(&mut bytes, &accelerator).unwrap();
        assert_eq!(
            read_accelerator_state(&mut bytes.as_slice(), 1).unwrap(),
            AcceleratorState {
                progress: 0.75,
                launching: false
            }
        );
    }

    #[test]
    fn landing_pad_state_codec_keeps_priority_and_revision_one_arrival_fields() {
        let state = LandingPadState {
            config: Some(3),
            priority: 123456,
            cooldown: 0.5,
            arriving: Some(4),
            arriving_timer: 0.25,
            liquid_removed: 750.0,
        };
        let mut bytes = Vec::new();
        write_landing_pad_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_landing_pad_state(&mut bytes.as_slice(), 1).unwrap(),
            state
        );

        let mut legacy = Vec::new();
        write_item_id(&mut legacy, Some(5)).unwrap();
        write_i32(&mut legacy, -9).unwrap();
        write_f32(&mut legacy, 0.25).unwrap();
        assert_eq!(
            read_landing_pad_state(&mut legacy.as_slice(), 0).unwrap(),
            LandingPadState {
                config: Some(5),
                priority: -9,
                cooldown: 0.25,
                ..LandingPadState::default()
            }
        );
    }
}
