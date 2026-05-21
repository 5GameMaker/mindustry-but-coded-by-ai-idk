//! Timed component mirroring upstream `mindustry.entities.comp.TimedComp`.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TimedComp {
    pub time: f32,
    pub lifetime: f32,
    removed: bool,
}

impl TimedComp {
    pub const fn new(lifetime: f32) -> Self {
        Self {
            time: 0.0,
            lifetime,
            removed: false,
        }
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }

    pub fn remove(&mut self) {
        self.removed = true;
    }

    /// Java update body with explicit delta instead of global `Time.delta`.
    pub fn update(&mut self, delta: f32) {
        self.time = (self.time + delta).min(self.lifetime);
        if self.time >= self.lifetime {
            self.remove();
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
    fn timed_component_accumulates_time_and_removes_at_lifetime() {
        let mut timed = TimedComp::new(10.0);

        timed.update(4.0);
        assert_eq!(timed.time, 4.0);
        assert_eq!(timed.fin(), 0.4);
        assert!(!timed.is_removed());

        timed.update(10.0);
        assert_eq!(timed.time, 10.0);
        assert_eq!(timed.fin(), 1.0);
        assert!(timed.is_removed());
    }
}
