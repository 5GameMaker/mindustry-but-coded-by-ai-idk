//! Looping positional sound helper mirroring upstream `mindustry.audio.SoundLoop`.
//!
//! The Java class owns an Arc `Sound` and talks directly to `Core.audio`.
//! This Rust port keeps the state machine and timing semantics pure by routing
//! audio operations through a small backend trait.

const FADE_SPEED: f32 = 0.05;
const STOP_EPSILON: f32 = 0.001;

pub trait SoundLoopBackend {
    fn calc_volume(&self, sound: &str, x: f32, y: f32) -> f32;

    fn calc_pan(&self, sound: &str, x: f32, y: f32) -> f32;

    fn loop_sound(&mut self, sound: &str, volume: f32, pitch: f32, pan: f32) -> i32;

    fn set_loop(&mut self, id: i32, pan: f32, volume: f32);

    fn stop_loop(&mut self, id: i32);
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundLoop {
    sound: String,
    id: i32,
    volume: f32,
    base_volume: f32,
}

impl SoundLoop {
    pub fn new(sound: impl Into<String>, base_volume: f32) -> Self {
        Self {
            sound: sound.into(),
            id: -1,
            volume: 0.0,
            base_volume,
        }
    }

    pub fn sound(&self) -> &str {
        &self.sound
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn base_volume(&self) -> f32 {
        self.base_volume
    }

    pub fn is_playing(&self) -> bool {
        self.id >= 0
    }

    pub fn update(
        &mut self,
        backend: &mut impl SoundLoopBackend,
        x: f32,
        y: f32,
        play: bool,
        delta: f32,
    ) {
        self.update_scaled(backend, x, y, play, 1.0, delta);
    }

    pub fn update_scaled(
        &mut self,
        backend: &mut impl SoundLoopBackend,
        x: f32,
        y: f32,
        play: bool,
        volume_scl: f32,
        delta: f32,
    ) {
        if self.base_volume <= 0.0 {
            return;
        }

        if self.id < 0 {
            if play {
                self.id = backend.loop_sound(
                    &self.sound,
                    backend.calc_volume(&self.sound, x, y)
                        * self.volume
                        * self.base_volume
                        * volume_scl,
                    1.0,
                    backend.calc_pan(&self.sound, x, y),
                );
            }
            return;
        }

        if play {
            self.volume = clamp01(self.volume + FADE_SPEED * delta);
        } else {
            self.volume = clamp01(self.volume - FADE_SPEED * delta);
            if self.volume <= STOP_EPSILON {
                backend.stop_loop(self.id);
                self.id = -1;
                return;
            }
        }

        backend.set_loop(
            self.id,
            backend.calc_pan(&self.sound, x, y),
            backend.calc_volume(&self.sound, x, y) * self.volume * self.base_volume * volume_scl,
        );
    }

    pub fn stop(&mut self, backend: &mut impl SoundLoopBackend) {
        if self.id != -1 {
            backend.stop_loop(self.id);
            self.id = -1;
            self.volume = 0.0;
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Action {
        Loop {
            sound: String,
            volume: f32,
            pitch: f32,
            pan: f32,
        },
        Set {
            id: i32,
            pan: f32,
            volume: f32,
        },
        Stop {
            id: i32,
        },
    }

    #[derive(Debug)]
    struct FakeBackend {
        next_id: i32,
        calc_volume: f32,
        calc_pan: f32,
        actions: Vec<Action>,
    }

    impl FakeBackend {
        fn new() -> Self {
            Self {
                next_id: 42,
                calc_volume: 0.8,
                calc_pan: -0.25,
                actions: Vec::new(),
            }
        }
    }

    impl SoundLoopBackend for FakeBackend {
        fn calc_volume(&self, _sound: &str, _x: f32, _y: f32) -> f32 {
            self.calc_volume
        }

        fn calc_pan(&self, _sound: &str, _x: f32, _y: f32) -> f32 {
            self.calc_pan
        }

        fn loop_sound(&mut self, sound: &str, volume: f32, pitch: f32, pan: f32) -> i32 {
            self.actions.push(Action::Loop {
                sound: sound.into(),
                volume,
                pitch,
                pan,
            });
            self.next_id
        }

        fn set_loop(&mut self, id: i32, pan: f32, volume: f32) {
            self.actions.push(Action::Set { id, pan, volume });
        }

        fn stop_loop(&mut self, id: i32) {
            self.actions.push(Action::Stop { id });
        }
    }

    #[test]
    fn first_update_starts_loop_with_current_zero_volume_like_java() {
        let mut backend = FakeBackend::new();
        let mut looped = SoundLoop::new("drill", 0.5);

        looped.update(&mut backend, 12.0, 34.0, true, 1.0);

        assert_eq!(looped.id(), 42);
        assert_eq!(looped.volume(), 0.0);
        assert_eq!(
            backend.actions,
            vec![Action::Loop {
                sound: "drill".into(),
                volume: 0.0,
                pitch: 1.0,
                pan: -0.25
            }]
        );
    }

    #[test]
    fn active_loop_fades_in_and_sets_scaled_positional_volume() {
        let mut backend = FakeBackend::new();
        let mut looped = SoundLoop::new("smelter", 0.5);
        looped.update(&mut backend, 0.0, 0.0, true, 1.0);
        backend.actions.clear();

        looped.update_scaled(&mut backend, 0.0, 0.0, true, 2.0, 10.0);

        assert_eq!(looped.volume(), 0.5);
        assert_eq!(
            backend.actions,
            vec![Action::Set {
                id: 42,
                pan: -0.25,
                volume: 0.4
            }]
        );
    }

    #[test]
    fn fade_out_stops_and_clears_id_at_java_threshold() {
        let mut backend = FakeBackend::new();
        let mut looped = SoundLoop::new("conveyor", 1.0);
        looped.update(&mut backend, 0.0, 0.0, true, 1.0);
        looped.update(&mut backend, 0.0, 0.0, true, 2.0);
        backend.actions.clear();

        looped.update(&mut backend, 0.0, 0.0, false, 2.0);

        assert_eq!(looped.id(), -1);
        assert_eq!(backend.actions, vec![Action::Stop { id: 42 }]);
    }

    #[test]
    fn zero_or_negative_base_volume_is_noop() {
        let mut backend = FakeBackend::new();
        let mut looped = SoundLoop::new("silent", 0.0);

        looped.update(&mut backend, 0.0, 0.0, true, 100.0);

        assert_eq!(looped.id(), -1);
        assert!(backend.actions.is_empty());
    }

    #[test]
    fn stop_matches_java_reset_semantics() {
        let mut backend = FakeBackend::new();
        let mut looped = SoundLoop::new("hum", 1.0);
        looped.update(&mut backend, 0.0, 0.0, true, 1.0);
        looped.update(&mut backend, 0.0, 0.0, true, 5.0);
        backend.actions.clear();

        looped.stop(&mut backend);

        assert_eq!(looped.id(), -1);
        assert_eq!(looped.volume(), 0.0);
        assert_eq!(backend.actions, vec![Action::Stop { id: 42 }]);
    }
}
