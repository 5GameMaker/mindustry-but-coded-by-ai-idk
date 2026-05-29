//! Launch animation hook trait mirroring upstream `mindustry.world.blocks.LaunchAnimator`.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchAnimationStep {
    pub active_before_tick: bool,
    pub launching: bool,
    pub should_update_launch: bool,
    pub ended: bool,
    pub land_time: f32,
    pub land_time_in: f32,
    pub remaining_land_time: f32,
}

impl LaunchAnimationStep {
    pub const fn inactive() -> Self {
        Self {
            active_before_tick: false,
            launching: false,
            should_update_launch: false,
            ended: false,
            land_time: 0.0,
            land_time_in: 0.0,
            remaining_land_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaunchAnimationState {
    pub active: bool,
    pub launching: bool,
    pub land_time: f32,
    pub launch_duration: f32,
}

impl Default for LaunchAnimationState {
    fn default() -> Self {
        Self {
            active: false,
            launching: false,
            land_time: 0.0,
            launch_duration: 0.0,
        }
    }
}

impl LaunchAnimationState {
    pub fn show_launch(&mut self, launch_duration: f32) {
        let launch_duration = launch_duration.max(0.0);
        self.active = launch_duration > 0.0;
        self.launching = true;
        self.land_time = launch_duration;
        self.launch_duration = launch_duration;
    }

    pub fn show_landing(&mut self, launch_duration: f32) {
        let launch_duration = launch_duration.max(0.0);
        self.active = launch_duration > 0.0;
        self.launching = false;
        self.land_time = launch_duration;
        self.launch_duration = launch_duration;
    }

    pub fn clear(&mut self) {
        self.active = false;
        self.launching = false;
        self.land_time = 0.0;
        self.launch_duration = 0.0;
    }

    pub fn get_land_time(&self) -> f32 {
        if self.active {
            self.land_time.max(0.0)
        } else {
            0.0
        }
    }

    pub fn get_land_time_in(&self) -> f32 {
        if !self.active || self.launch_duration <= 0.0 {
            return 0.0;
        }

        let fin = (self.land_time / self.launch_duration).clamp(0.0, 1.0);
        if self.launching {
            fin
        } else {
            1.0 - fin
        }
    }

    pub fn is_cutscene(&self) -> bool {
        self.active && self.land_time > 0.0
    }

    pub fn tick(&mut self, delta: f32, paused: bool) -> LaunchAnimationStep {
        if !self.active {
            self.land_time = 0.0;
            return LaunchAnimationStep::inactive();
        }

        let active_before_tick = self.active;
        let launching = self.launching;
        let land_time = self.get_land_time();
        let land_time_in = self.get_land_time_in();
        let should_update_launch = land_time > 0.0 && !paused;

        if land_time > 0.0 && !paused {
            self.land_time = (self.land_time - delta.max(0.0)).max(0.0);
        }

        let ended = self.land_time <= 0.0;
        if ended {
            self.active = false;
            self.land_time = 0.0;
        }

        LaunchAnimationStep {
            active_before_tick,
            launching,
            should_update_launch,
            ended,
            land_time,
            land_time_in,
            remaining_land_time: self.get_land_time(),
        }
    }
}

pub trait LaunchAnimator {
    fn draw_launch(&mut self);

    fn draw_launch_global_z(&mut self) {}

    fn begin_launch(&mut self, launching: bool);

    fn end_launch(&mut self);

    fn update_launch(&mut self);

    fn launch_duration(&self) -> f32;

    fn land_music(&self) -> Option<&'static str> {
        Some("land")
    }

    fn zoom_launch(&self) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct Animator {
        calls: Vec<&'static str>,
        launching: bool,
    }

    impl LaunchAnimator for Animator {
        fn draw_launch(&mut self) {
            self.calls.push("draw");
        }

        fn begin_launch(&mut self, launching: bool) {
            self.calls.push("begin");
            self.launching = launching;
        }

        fn end_launch(&mut self) {
            self.calls.push("end");
            self.launching = false;
        }

        fn update_launch(&mut self) {
            self.calls.push("update");
        }

        fn launch_duration(&self) -> f32 {
            120.0
        }

        fn zoom_launch(&self) -> f32 {
            4.0
        }
    }

    #[test]
    fn launch_animator_required_hooks_are_callable() {
        let mut animator = Animator::default();

        animator.begin_launch(true);
        animator.draw_launch();
        animator.update_launch();
        animator.end_launch();

        assert_eq!(animator.calls, vec!["begin", "draw", "update", "end"]);
        assert!(!animator.launching);
        assert_eq!(animator.launch_duration(), 120.0);
        assert_eq!(animator.zoom_launch(), 4.0);
    }

    #[test]
    fn default_global_z_and_land_music_match_java_defaults() {
        let mut animator = Animator::default();

        animator.draw_launch_global_z();

        assert!(animator.calls.is_empty());
        assert_eq!(animator.land_music(), Some("land"));
    }

    #[test]
    fn launch_animation_state_show_launch_matches_renderer_land_time() {
        let mut state = LaunchAnimationState::default();

        state.show_launch(340.0);

        assert!(state.active);
        assert!(state.launching);
        assert_eq!(state.get_land_time(), 340.0);
        assert_eq!(state.get_land_time_in(), 1.0);
        assert!(state.is_cutscene());
    }

    #[test]
    fn launch_animation_state_show_landing_inverts_land_time_in() {
        let mut state = LaunchAnimationState::default();

        state.show_landing(340.0);

        assert!(state.active);
        assert!(!state.launching);
        assert_eq!(state.get_land_time(), 340.0);
        assert_eq!(state.get_land_time_in(), 0.0);
    }

    #[test]
    fn launch_animation_state_tick_updates_before_decrement_like_renderer() {
        let mut state = LaunchAnimationState::default();
        state.show_launch(100.0);

        let first = state.tick(25.0, false);
        assert!(first.should_update_launch);
        assert!(!first.ended);
        assert_eq!(first.land_time, 100.0);
        assert_eq!(first.land_time_in, 1.0);
        assert_eq!(first.remaining_land_time, 75.0);

        let second = state.tick(80.0, false);
        assert!(second.should_update_launch);
        assert!(second.ended);
        assert_eq!(second.land_time, 75.0);
        assert_eq!(second.land_time_in, 0.75);
        assert_eq!(second.remaining_land_time, 0.0);
        assert!(!state.active);
    }

    #[test]
    fn launch_animation_state_tick_respects_pause() {
        let mut state = LaunchAnimationState::default();
        state.show_launch(100.0);

        let tick = state.tick(25.0, true);

        assert!(!tick.should_update_launch);
        assert!(!tick.ended);
        assert_eq!(tick.land_time, 100.0);
        assert_eq!(state.get_land_time(), 100.0);
    }
}
