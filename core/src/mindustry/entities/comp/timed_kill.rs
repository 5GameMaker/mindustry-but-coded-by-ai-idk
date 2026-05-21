//! Timed-kill component mirroring upstream
//! `mindustry.entities.comp.TimedKillComp`.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TimedKillComp {
    pub time: f32,
    pub lifetime: f32,
    killed: bool,
}

impl TimedKillComp {
    pub const fn new(lifetime: f32) -> Self {
        Self {
            time: 0.0,
            lifetime,
            killed: false,
        }
    }

    pub fn is_killed(&self) -> bool {
        self.killed
    }

    pub fn kill(&mut self) {
        self.killed = true;
    }

    /// Java update body with explicit delta instead of global `Time.delta`.
    pub fn update(&mut self, delta: f32) {
        self.time = (self.time + delta).min(self.lifetime);
        if self.time >= self.lifetime {
            self.kill();
        }
    }

    pub fn fin(&self) -> f32 {
        self.time / self.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_kill_component_kills_at_lifetime() {
        let mut timed = TimedKillComp::new(3.0);

        timed.update(1.0);
        assert_eq!(timed.fin(), 1.0 / 3.0);
        assert!(!timed.is_killed());

        timed.update(5.0);
        assert_eq!(timed.time, 3.0);
        assert_eq!(timed.fin(), 1.0);
        assert!(timed.is_killed());
    }
}
