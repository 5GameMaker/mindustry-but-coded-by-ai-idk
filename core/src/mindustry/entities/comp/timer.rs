//! Timer component mirroring upstream `mindustry.entities.comp.TimerComp`.
//!
//! Upstream stores a transient `arc.util.Interval` with 6 slots. This Rust
//! version keeps the same slot-based timing semantics with explicit clock
//! control so tests and deterministic simulation code do not need a global
//! `Time.time`.

#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    times: Vec<f32>,
    now: f32,
}

impl Interval {
    pub fn new(timers: usize) -> Self {
        Self {
            times: vec![0.0; timers],
            now: 0.0,
        }
    }

    pub fn timers(&self) -> usize {
        self.times.len()
    }

    pub fn now(&self) -> f32 {
        self.now
    }

    pub fn set_time(&mut self, now: f32) {
        self.now = now;
    }

    pub fn advance(&mut self, delta: f32) {
        self.now += delta;
    }

    pub fn last_time(&self, index: usize) -> f32 {
        self.times[index]
    }

    pub fn get(&mut self, index: usize, time: f32) -> bool {
        let last = self.times[index];
        if self.now - last >= time || self.now < last {
            self.times[index] = self.now;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self, index: usize, time: f32) {
        self.times[index] = self.now + time;
    }

    pub fn clear(&mut self) {
        self.times.fill(0.0);
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::new(1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimerComp {
    pub timer: Interval,
}

impl TimerComp {
    pub const DEFAULT_TIMERS: usize = 6;

    pub fn new() -> Self {
        Self {
            timer: Interval::new(Self::DEFAULT_TIMERS),
        }
    }

    pub fn set_time(&mut self, now: f32) {
        self.timer.set_time(now);
    }

    pub fn advance_time(&mut self, delta: f32) {
        self.timer.advance(delta);
    }

    /// Java: `if(Float.isInfinite(time)) return false; return timer.get(index, time);`
    pub fn timer(&mut self, index: usize, time: f32) -> bool {
        if time.is_infinite() {
            return false;
        }
        self.timer.get(index, time)
    }
}

impl Default for TimerComp {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildingTimerState {
    pub timer: Interval,
}

impl BuildingTimerState {
    pub const DEFAULT_TIMERS: usize = TimerComp::DEFAULT_TIMERS;

    pub fn new(timers: usize) -> Self {
        Self {
            timer: Interval::new(timers),
        }
    }

    pub fn set_time(&mut self, now: f32) {
        self.timer.set_time(now);
    }

    pub fn advance_time(&mut self, delta: f32) {
        self.timer.advance(delta);
    }

    /// Java `TimerComp.timer(index, time)` semantics for building runtime
    /// sidecars: infinite intervals never fire, finite intervals delegate to
    /// the slot-based `Interval`.
    pub fn timer(&mut self, index: usize, time: f32) -> bool {
        if time.is_infinite() {
            return false;
        }
        self.timer.get(index, time)
    }

    pub fn reset(&mut self, index: usize, time: f32) {
        self.timer.reset(index, time);
    }

    pub fn clear(&mut self) {
        self.timer.clear();
    }
}

impl Default for BuildingTimerState {
    fn default() -> Self {
        Self::new(Self::DEFAULT_TIMERS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_component_uses_six_java_interval_slots() {
        let timer = TimerComp::new();

        assert_eq!(timer.timer.timers(), 6);
        assert_eq!(timer.timer.now(), 0.0);
    }

    #[test]
    fn timer_component_rejects_infinite_time_before_touching_interval() {
        let mut timer = TimerComp::new();

        assert!(!timer.timer(100, f32::INFINITY));
        assert_eq!(timer.timer.last_time(0), 0.0);
    }

    #[test]
    fn timer_component_fires_after_interval_and_on_time_rewind() {
        let mut timer = TimerComp::new();

        assert!(!timer.timer(0, 10.0));
        timer.set_time(9.0);
        assert!(!timer.timer(0, 10.0));

        timer.advance_time(1.0);
        assert!(timer.timer(0, 10.0));
        assert_eq!(timer.timer.last_time(0), 10.0);

        timer.set_time(1.0);
        assert!(timer.timer(0, 10.0));
        assert_eq!(timer.timer.last_time(0), 1.0);
    }

    #[test]
    fn interval_reset_and_clear_match_slot_based_shape() {
        let mut interval = Interval::new(2);
        interval.set_time(5.0);
        interval.reset(1, 3.0);

        assert_eq!(interval.last_time(1), 8.0);
        assert!(interval.get(1, 10.0));

        interval.clear();
        assert_eq!(interval.last_time(0), 0.0);
        assert_eq!(interval.last_time(1), 0.0);
    }

    #[test]
    fn building_timer_state_matches_java_building_timer_sidecar_shape() {
        let mut timer = BuildingTimerState::default();

        assert_eq!(timer.timer.timers(), 6);
        assert_eq!(timer.timer.now(), 0.0);
        assert!(!timer.timer(0, f32::INFINITY));
        assert!(!timer.timer(1, 8.0));

        timer.advance_time(8.0);
        assert!(timer.timer(1, 8.0));
        assert_eq!(timer.timer.last_time(1), 8.0);

        timer.reset(1, 4.0);
        assert_eq!(timer.timer.last_time(1), 12.0);
        timer.set_time(13.0);
        assert!(!timer.timer(1, 4.0));
        timer.set_time(3.0);
        assert!(timer.timer(1, 4.0));

        timer.clear();
        assert_eq!(timer.timer.last_time(1), 0.0);
    }
}
