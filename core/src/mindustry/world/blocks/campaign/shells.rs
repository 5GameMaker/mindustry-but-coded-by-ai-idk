use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
};

use crate::mindustry::{
    ctype::ContentId,
    world::{meta::BlockFlag, Block, BlockId},
};

use super::{
    accelerator_can_launch, accelerator_consume_launch, accelerator_is_core_built,
    accelerator_maximum_accepted, accelerator_update_progress, landing_pad_accept_item,
    landing_pad_handle_landing, landing_pad_ready_to_queue, landing_pad_update_arrival,
    landing_pad_update_cooldown, launch_pad_accept_item, launch_pad_progress,
    launch_pad_should_consume, read_accelerator_state, read_landing_pad_state,
    read_launch_pad_state, write_accelerator_state, write_landing_pad_state,
    write_launch_pad_state, AcceleratorState, LandingPadArrivalStep, LandingPadState,
    LaunchPadState, LaunchPadStep, DEFAULT_ACCELERATOR_BUILD_DURATION,
    DEFAULT_ACCELERATOR_CHARGE_DURATION, DEFAULT_ACCELERATOR_LAUNCH_DURATION,
    DEFAULT_LANDING_ARRIVAL_DURATION, DEFAULT_LANDING_COOLDOWN_TIME,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchPad {
    pub block: Block,
    pub launch_time: f32,
    pub launch_sound_pitch_rand: f32,
    pub accept_multiple_items: bool,
    pub light_step: f32,
    pub light_steps: i32,
    pub liquid_pad: f32,
    pub draw_liquid: Option<ContentId>,
    pub bottom_color_rgba: u32,
    pub light_color_rgba: u32,
}

impl LaunchPad {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.has_items = true;
        block.solid = true;
        block.update = true;
        block.configurable = true;
        block.flags = vec![BlockFlag::LaunchPad];

        Self {
            block,
            launch_time: 1.0,
            launch_sound_pitch_rand: 0.1,
            accept_multiple_items: false,
            light_step: 1.0,
            light_steps: 3,
            liquid_pad: 2.0,
            draw_liquid: None,
            bottom_color_rgba: 0,
            light_color_rgba: 0xeab6_78ff,
        }
    }

    pub fn should_consume(&self, launch_counter: f32) -> bool {
        launch_pad_should_consume(launch_counter, self.launch_time)
    }

    pub fn progress(&self, launch_counter: f32) -> f32 {
        launch_pad_progress(launch_counter, self.launch_time)
    }

    pub fn accept_item(&self, total_items: i32, first_item: Option<i16>, item: i16) -> bool {
        launch_pad_accept_item(
            self.block.item_capacity,
            self.accept_multiple_items,
            total_items,
            first_item,
            item,
        )
    }

    pub fn update(
        &self,
        state: &mut LaunchPadState,
        total_items: i32,
        edelta: f32,
    ) -> LaunchPadStep {
        super::launch_pad_update(
            state,
            self.launch_time,
            self.block.item_capacity,
            total_items,
            edelta,
        )
    }

    pub fn write_state<W: Write>(&self, write: &mut W, state: &LaunchPadState) -> io::Result<()> {
        write_launch_pad_state(write, state)
    }

    pub fn read_state<R: Read>(&self, read: &mut R, revision: u8) -> io::Result<LaunchPadState> {
        read_launch_pad_state(read, revision)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LandingPad {
    pub block: Block,
    pub arrival_duration: f32,
    pub cooldown_time: f32,
    pub consume_liquid_amount: f32,
    pub consume_liquid: Option<ContentId>,
    pub land_sound_volume: f32,
    pub liquid_pad: f32,
    pub bottom_color_rgba: u32,
    pub cooling_effect_chance: f32,
    pub land_effect: String,
    pub cooling_effect: String,
    pub land_sound: String,
}

impl LandingPad {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.has_items = true;
        block.has_liquids = true;
        block.solid = true;
        block.update = true;
        block.configurable = true;
        block.emit_light = true;
        block.light_radius = 90.0;

        Self {
            block,
            arrival_duration: DEFAULT_LANDING_ARRIVAL_DURATION,
            cooldown_time: DEFAULT_LANDING_COOLDOWN_TIME,
            consume_liquid_amount: 100.0,
            consume_liquid: None,
            land_sound_volume: 0.75,
            liquid_pad: 2.0,
            bottom_color_rgba: 0,
            cooling_effect_chance: 0.2,
            land_effect: "podLandShockwave".into(),
            cooling_effect: "none".into(),
            land_sound: "padLand".into(),
        }
    }

    pub fn accept_item(&self) -> bool {
        landing_pad_accept_item()
    }

    pub fn handle_landing(&self, state: &mut LandingPadState) -> bool {
        landing_pad_handle_landing(state)
    }

    pub fn update_arrival(
        &self,
        state: &mut LandingPadState,
        item_capacity: i32,
        delta: f32,
    ) -> LandingPadArrivalStep {
        landing_pad_update_arrival(
            state,
            self.arrival_duration,
            self.consume_liquid_amount,
            item_capacity,
            delta,
        )
    }

    pub fn update_cooldown(&self, state: &mut LandingPadState, delta: f32) -> f32 {
        landing_pad_update_cooldown(state, self.cooldown_time, delta)
    }

    pub fn ready_to_queue(
        &self,
        state: &LandingPadState,
        efficiency: f32,
        total_items: i32,
        is_fake: bool,
        is_campaign_non_legacy: bool,
        import_rate_positive: bool,
        import_cooldown: f32,
    ) -> bool {
        landing_pad_ready_to_queue(
            state,
            efficiency,
            total_items,
            is_fake,
            is_campaign_non_legacy,
            import_rate_positive,
            import_cooldown,
        )
    }

    pub fn write_state<W: Write>(&self, write: &mut W, state: &LandingPadState) -> io::Result<()> {
        write_landing_pad_state(write, state)
    }

    pub fn read_state<R: Read>(&self, read: &mut R, revision: u8) -> io::Result<LandingPadState> {
        read_landing_pad_state(read, revision)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Accelerator {
    pub block: Block,
    pub launch_block: BlockId,
    pub power_buffer_requirement: f32,
    pub launch_candidates: Option<Vec<BlockId>>,
    pub launch_duration: f32,
    pub charge_duration: f32,
    pub build_duration: f32,
    pub launch_lightning: i32,
    pub lightning_sound_volume: f32,
    pub lightning_damage: f32,
}

impl Accelerator {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.has_items = true;
        block.has_power = true;
        block.item_capacity = 8000;
        block.configurable = true;
        block.emit_light = true;
        block.light_radius = 70.0;
        block.light_color_rgba = 0xffd3_7fff;

        Self {
            block,
            launch_block: 0,
            power_buffer_requirement: 0.0,
            launch_candidates: None,
            launch_duration: DEFAULT_ACCELERATOR_LAUNCH_DURATION,
            charge_duration: DEFAULT_ACCELERATOR_CHARGE_DURATION,
            build_duration: DEFAULT_ACCELERATOR_BUILD_DURATION,
            launch_lightning: 20,
            lightning_sound_volume: 0.85,
            lightning_damage: 40.0,
        }
    }

    pub fn update_progress(
        &self,
        state: &mut AcceleratorState,
        delta: f32,
        efficiency: f32,
    ) -> f32 {
        accelerator_update_progress(state, delta, efficiency, self.build_duration)
    }

    pub fn is_core_built(&self, progress: f32) -> bool {
        accelerator_is_core_built(progress)
    }

    pub fn can_launch(
        &self,
        valid: bool,
        net_client: bool,
        campaign: bool,
        efficiency: f32,
        battery_stored: f32,
        progress: f32,
        launching: bool,
    ) -> bool {
        accelerator_can_launch(
            valid,
            net_client,
            campaign,
            efficiency,
            battery_stored,
            self.power_buffer_requirement,
            progress,
            launching,
        )
    }

    pub fn maximum_accepted(
        &self,
        capacities: &BTreeMap<i16, i32>,
        item: i16,
        core_built: bool,
    ) -> i32 {
        accelerator_maximum_accepted(capacities, item, core_built, self.block.item_capacity)
    }

    pub fn accept_item(
        &self,
        capacities: &BTreeMap<i16, i32>,
        item: i16,
        current_amount: i32,
        core_built: bool,
    ) -> bool {
        current_amount < self.maximum_accepted(capacities, item, core_built)
    }

    pub fn consume_launch(&self, state: &mut AcceleratorState) {
        accelerator_consume_launch(state);
    }

    pub fn write_state<W: Write>(&self, write: &mut W, state: &AcceleratorState) -> io::Result<()> {
        write_accelerator_state(write, state)
    }

    pub fn read_state<R: Read>(&self, read: &mut R, revision: u8) -> io::Result<AcceleratorState> {
        read_accelerator_state(read, revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_shell_defaults_match_java_constructor_shape() {
        let launch = LaunchPad::new("launch-pad");
        assert!(launch.block.has_items);
        assert_eq!(launch.block.flags, vec![BlockFlag::LaunchPad]);
        assert_eq!(launch.launch_time, 1.0);
        assert_eq!(launch.accept_multiple_items, false);
        assert_eq!(launch.progress(0.5), 0.5);

        let landing = LandingPad::new("landing-pad");
        assert!(landing.block.has_items);
        assert!(landing.block.has_liquids);
        assert!(landing.block.emit_light);
        assert_eq!(landing.arrival_duration, DEFAULT_LANDING_ARRIVAL_DURATION);
        assert_eq!(landing.cooldown_time, DEFAULT_LANDING_COOLDOWN_TIME);

        let accelerator = Accelerator::new("accelerator");
        assert!(accelerator.block.has_power);
        assert_eq!(accelerator.block.item_capacity, 8000);
        assert_eq!(
            accelerator.launch_duration,
            DEFAULT_ACCELERATOR_LAUNCH_DURATION
        );
        assert_eq!(
            accelerator.charge_duration,
            DEFAULT_ACCELERATOR_CHARGE_DURATION
        );
        assert_eq!(
            accelerator.build_duration,
            DEFAULT_ACCELERATOR_BUILD_DURATION
        );
        assert_eq!(accelerator.launch_lightning, 20);
    }

    #[test]
    fn campaign_state_and_codec_helpers_delegate_to_runtime_functions() {
        let mut launch = LaunchPad::new("launch-pad");
        launch.launch_time = 1200.0;
        launch.block.item_capacity = 100;
        let mut launch_state = LaunchPadState {
            launch_counter: 1199.0,
        };
        assert!(launch.should_consume(launch_state.launch_counter));
        let step = launch.update(&mut launch_state, 99, 1.0);
        assert!(!step.launched);
        assert_eq!(step.launch_counter, 1200.0);
        let step = launch.update(&mut launch_state, 100, 0.1);
        assert!(step.launched);

        let landing = LandingPad::new("landing-pad");
        let mut landing_state = LandingPadState {
            config: Some(7),
            ..Default::default()
        };
        assert!(landing.handle_landing(&mut landing_state));
        let step = landing.update_arrival(&mut landing_state, 100, 75.0);
        assert_eq!(step.finished_item, None);
        assert_eq!(landing.update_cooldown(&mut landing_state, 75.0), 1.0);
        landing_state.arriving = None;
        landing_state.cooldown = 0.0;
        assert!(landing.ready_to_queue(&landing_state, 1.0, 0, true, false, false, 0.0));

        let accelerator = Accelerator::new("accelerator");
        let mut accel_state = AcceleratorState::default();
        assert_eq!(
            accelerator.update_progress(&mut accel_state, 60.0, 1.0),
            0.5
        );
        assert!(!accelerator.is_core_built(accel_state.progress));
        accelerator.consume_launch(&mut accel_state);
        assert!(accel_state.launching);

        let mut bytes = Vec::new();
        launch
            .write_state(
                &mut bytes,
                &LaunchPadState {
                    launch_counter: 42.5,
                },
            )
            .unwrap();
        assert_eq!(
            launch.read_state(&mut bytes.as_slice(), 1).unwrap(),
            LaunchPadState {
                launch_counter: 42.5
            }
        );

        let mut bytes = Vec::new();
        landing
            .write_state(
                &mut bytes,
                &LandingPadState {
                    config: Some(3),
                    priority: 9,
                    cooldown: 0.5,
                    arriving: Some(4),
                    arriving_timer: 0.25,
                    liquid_removed: 12.0,
                },
            )
            .unwrap();
        assert_eq!(
            landing.read_state(&mut bytes.as_slice(), 1).unwrap().config,
            Some(3)
        );

        let mut capacities = BTreeMap::new();
        capacities.insert(3, 8000);
        assert_eq!(accelerator.maximum_accepted(&capacities, 3, false), 8000);
        assert!(accelerator.accept_item(&capacities, 3, 7999, false));
    }
}
