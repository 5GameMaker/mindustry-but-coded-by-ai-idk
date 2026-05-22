use std::collections::BTreeMap;

use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub struct SoundControl {
    pub fin_time: f32,
    pub fout_time: f32,
    pub music_interval: f32,
    pub music_chance: f32,
    pub music_wave_chance: f32,
    pub ambient_music: Vec<String>,
    pub dark_music: Vec<String>,
    pub boss_music: Vec<String>,
    pub last_random_played: Option<String>,
    pub last_played_millis: u64,
    pub current: Option<String>,
    pub fade: f32,
    pub silenced: bool,
    pub was_playing: bool,
    pub sounds: BTreeMap<String, SoundData>,
}

impl Default for SoundControl {
    fn default() -> Self {
        Self {
            fin_time: 120.0,
            fout_time: 120.0,
            music_interval: 3.0,
            music_chance: 0.8,
            music_wave_chance: 0.46,
            ambient_music: Vec::new(),
            dark_music: Vec::new(),
            boss_music: Vec::new(),
            last_random_played: None,
            last_played_millis: 0,
            current: None,
            fade: 0.0,
            silenced: false,
            was_playing: false,
            sounds: BTreeMap::new(),
        }
    }
}

impl SoundControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reload_default_music(&mut self) -> MusicRegisterPlan {
        self.current = None;
        self.fade = 0.0;
        self.ambient_music = ["game1", "game3", "game6", "game8", "game9", "fine"]
            .into_iter()
            .map(str::to_string)
            .collect();
        self.dark_music = ["game2", "game5", "game7", "game4"]
            .into_iter()
            .map(str::to_string)
            .collect();
        self.boss_music = ["boss1", "boss2", "game2", "game5"]
            .into_iter()
            .map(str::to_string)
            .collect();

        MusicRegisterPlan {
            fire_music_register_event: true,
            ui_bus_sound_folder: "ui".into(),
        }
    }

    pub fn loop_sound(
        &mut self,
        sound: impl Into<String>,
        pos: Vec2,
        volume: f32,
        pitch: f32,
        base_falloff: f32,
        headless: bool,
    ) -> bool {
        let sound = sound.into();
        if headless || sound == "none" || volume <= 0.00001 {
            return false;
        }

        let vol = base_falloff * volume;
        let data = self.sounds.entry(sound).or_default();
        data.volume = (data.volume + vol).clamp(0.0, 1.0);
        data.pitch += pitch * vol;
        data.total += base_falloff;
        data.total_volume += vol;
        data.sum.x += pos.x * base_falloff;
        data.sum.y += pos.y * base_falloff;
        true
    }

    pub fn stop(&mut self) -> MusicPlaybackPlan {
        self.silenced = true;
        if let Some(stopped) = self.current.take() {
            self.fade = 0.0;
            MusicPlaybackPlan {
                action: MusicAction::Stop { music: stopped },
                fade: self.fade,
                current: None,
            }
        } else {
            MusicPlaybackPlan {
                action: MusicAction::None,
                fade: self.fade,
                current: None,
            }
        }
    }

    pub fn reset_event(&mut self, now_millis: u64) -> SoundBusPlan {
        self.last_played_millis = now_millis;
        SoundBusPlan {
            stop_sound_bus: true,
            play_sound_bus: true,
            play_music_bus: false,
        }
    }

    pub fn update_bus_playing(&mut self, playing: bool) -> Option<SoundBusPlan> {
        if playing == self.was_playing {
            return None;
        }

        self.was_playing = playing;
        Some(if playing {
            SoundBusPlan {
                stop_sound_bus: false,
                play_sound_bus: true,
                play_music_bus: false,
            }
        } else {
            SoundBusPlan {
                stop_sound_bus: true,
                play_sound_bus: true,
                play_music_bus: true,
            }
        })
    }

    pub fn update_filter_plan(timer_ready: bool, paused: bool) -> Option<FilterFadePlan> {
        timer_ready.then_some(FilterFadePlan {
            filter_index: 0,
            wet: if paused { 1.0 } else { 0.0 },
            duration: 0.4,
        })
    }

    pub fn play<S>(&mut self, music: Option<S>, input: MusicPlayInput) -> MusicPlaybackPlan
    where
        S: Into<String>,
    {
        let music = music.map(Into::into);
        if !input.should_play {
            if self.current.is_some() {
                self.fade = 0.0;
                return MusicPlaybackPlan {
                    action: MusicAction::SetVolume { volume: 0.0 },
                    fade: self.fade,
                    current: self.current.clone(),
                };
            }

            self.fade = 0.0;
            return MusicPlaybackPlan {
                action: MusicAction::None,
                fade: self.fade,
                current: None,
            };
        }

        if self.silenced {
            return MusicPlaybackPlan {
                action: MusicAction::None,
                fade: self.fade,
                current: self.current.clone(),
            };
        }

        match (&self.current, music) {
            (None, Some(next)) => {
                self.current = Some(next.clone());
                self.fade = 0.0;
                self.silenced = false;
                MusicPlaybackPlan {
                    action: MusicAction::Start {
                        music: next,
                        looping: true,
                        volume: 0.0,
                    },
                    fade: self.fade,
                    current: self.current.clone(),
                }
            }
            (Some(current), Some(next)) if *current == next => {
                self.fade = (self.fade + input.delta / self.fin_time).clamp(0.0, 1.0);
                MusicPlaybackPlan {
                    action: MusicAction::SetVolume {
                        volume: self.fade * input.music_volume,
                    },
                    fade: self.fade,
                    current: self.current.clone(),
                }
            }
            (Some(current), next) => {
                self.fade = (self.fade - input.delta / self.fout_time).clamp(0.0, 1.0);
                if self.fade <= 0.01 {
                    let stopped = current.clone();
                    self.current = None;
                    self.silenced = true;
                    if let Some(next) = next {
                        self.current = Some(next.clone());
                        self.fade = 0.0;
                        self.silenced = false;
                        MusicPlaybackPlan {
                            action: MusicAction::Switch {
                                stop: stopped,
                                start: next,
                            },
                            fade: self.fade,
                            current: self.current.clone(),
                        }
                    } else {
                        MusicPlaybackPlan {
                            action: MusicAction::Stop { music: stopped },
                            fade: self.fade,
                            current: None,
                        }
                    }
                } else {
                    MusicPlaybackPlan {
                        action: MusicAction::FadeOut {
                            music: current.clone(),
                            volume: self.fade * input.music_volume,
                        },
                        fade: self.fade,
                        current: self.current.clone(),
                    }
                }
            }
            (None, None) => MusicPlaybackPlan {
                action: MusicAction::None,
                fade: self.fade,
                current: None,
            },
        }
    }

    pub fn play_once(
        &mut self,
        music: Option<impl Into<String>>,
        should_play: bool,
    ) -> MusicPlaybackPlan {
        let Some(music) = music.map(Into::into) else {
            return MusicPlaybackPlan::none(self.fade, self.current.clone());
        };

        if self.current.is_some() || !should_play {
            return MusicPlaybackPlan::none(self.fade, self.current.clone());
        }

        self.last_random_played = Some(music.clone());
        self.fade = 1.0;
        self.current = Some(music.clone());
        MusicPlaybackPlan {
            action: MusicAction::Start {
                music,
                looping: false,
                volume: 1.0,
            },
            fade: self.fade,
            current: self.current.clone(),
        }
    }

    pub fn silence(&mut self, input: MusicPlayInput) -> MusicPlaybackPlan {
        self.play::<String>(None, input)
    }

    pub fn choose_random_track(
        &self,
        boss: bool,
        dark: bool,
        random_index: usize,
    ) -> Option<String> {
        let list = if boss {
            &self.boss_music
        } else if dark {
            &self.dark_music
        } else {
            &self.ambient_music
        };

        if list.is_empty() {
            None
        } else {
            let mut selected = list[random_index % list.len()].clone();
            if Some(&selected) == self.last_random_played.as_ref() && list.len() > 1 {
                selected = list[(random_index + 1) % list.len()].clone();
            }
            Some(selected)
        }
    }

    pub fn is_dark(
        core_health_fraction: Option<f32>,
        wave: f32,
        enemies: f32,
        chance: SoundDarkChance,
    ) -> bool {
        if core_health_fraction
            .map(|health| health < 0.85)
            .unwrap_or(false)
        {
            return true;
        }

        let wave_chance = ((wave - 17.0) / 19.0).log10() + 1.0;
        if chance.wave_roll < wave_chance / 4.0 {
            return true;
        }

        chance.enemy_roll < enemies / 70.0 + 0.1
    }

    pub fn update_loops_plan(
        &mut self,
        is_game: bool,
        paused: bool,
        ambient_volume: f32,
    ) -> Vec<LoopSoundPlan> {
        if !is_game {
            self.sounds.clear();
            return Vec::new();
        }

        if paused {
            return Vec::new();
        }

        let mut plans = Vec::new();
        for (sound, data) in &mut self.sounds {
            data.cur_volume = lerp(data.cur_volume, data.volume * ambient_volume, 0.11);
            let play = data.cur_volume > 0.01;
            let pan = if data.total.abs() <= 0.0001 {
                0.0
            } else {
                data.sum.x / data.total
            };
            let pitch = if data.total_volume.abs() <= 0.0001 {
                1.0
            } else {
                data.pitch / data.total_volume
            };

            let action = if data.sound_id <= 0 {
                if play {
                    LoopSoundAction::Start
                } else {
                    LoopSoundAction::None
                }
            } else if data.cur_volume <= 0.001 {
                data.sound_id = -1;
                LoopSoundAction::Stop
            } else {
                LoopSoundAction::Update
            };

            plans.push(LoopSoundPlan {
                sound: sound.clone(),
                action,
                volume: data.cur_volume,
                pitch,
                pan,
            });

            data.pitch = 0.0;
            data.volume = 0.0;
            data.total = 0.0;
            data.total_volume = 0.0;
            data.sum = Vec2::new(0.0, 0.0);
        }

        plans
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MusicPlayInput {
    pub should_play: bool,
    pub delta: f32,
    pub music_volume: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MusicRegisterPlan {
    pub fire_music_register_event: bool,
    pub ui_bus_sound_folder: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterFadePlan {
    pub filter_index: i32,
    pub wet: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundBusPlan {
    pub stop_sound_bus: bool,
    pub play_sound_bus: bool,
    pub play_music_bus: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundDarkChance {
    pub wave_roll: f32,
    pub enemy_roll: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MusicAction {
    None,
    SetVolume {
        volume: f32,
    },
    Start {
        music: String,
        looping: bool,
        volume: f32,
    },
    FadeOut {
        music: String,
        volume: f32,
    },
    Stop {
        music: String,
    },
    Switch {
        stop: String,
        start: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MusicPlaybackPlan {
    pub action: MusicAction,
    pub fade: f32,
    pub current: Option<String>,
}

impl MusicPlaybackPlan {
    fn none(fade: f32, current: Option<String>) -> Self {
        Self {
            action: MusicAction::None,
            fade,
            current,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundData {
    pub volume: f32,
    pub pitch: f32,
    pub total: f32,
    pub sum: Vec2,
    pub sound_id: i32,
    pub cur_volume: f32,
    pub total_volume: f32,
}

impl Default for SoundData {
    fn default() -> Self {
        Self {
            volume: 0.0,
            pitch: 0.0,
            total: 0.0,
            sum: Vec2::new(0.0, 0.0),
            sound_id: 0,
            cur_volume: 0.0,
            total_volume: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopSoundAction {
    None,
    Start,
    Update,
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopSoundPlan {
    pub sound: String,
    pub action: LoopSoundAction,
    pub volume: f32,
    pub pitch: f32,
    pub pan: f32,
}

fn lerp(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_control_defaults_reload_and_stop_match_java_state_shape() {
        let mut control = SoundControl::new();
        assert_eq!(control.fin_time, 120.0);
        assert_eq!(control.fout_time, 120.0);
        assert_eq!(control.music_chance, 0.8);
        assert_eq!(control.music_wave_chance, 0.46);

        let plan = control.reload_default_music();
        assert!(plan.fire_music_register_event);
        assert_eq!(plan.ui_bus_sound_folder, "ui");
        assert_eq!(control.ambient_music[0], "game1");
        assert_eq!(control.dark_music, vec!["game2", "game5", "game7", "game4"]);
        assert_eq!(control.boss_music[0], "boss1");

        control.current = Some("menu".into());
        control.fade = 0.75;
        let stopped = control.stop();
        assert_eq!(
            stopped.action,
            MusicAction::Stop {
                music: "menu".into()
            }
        );
        assert!(control.silenced);
        assert_eq!(control.current, None);
        assert_eq!(control.fade, 0.0);
    }

    #[test]
    fn loop_sound_accumulates_weighted_volume_pitch_and_position() {
        let mut control = SoundControl::new();
        assert!(control.loop_sound("wind", Vec2::new(10.0, 20.0), 0.5, 2.0, 0.4, false));
        assert!(control.loop_sound("wind", Vec2::new(20.0, 20.0), 0.5, 1.0, 0.6, false));
        assert!(!control.loop_sound("none", Vec2::new(0.0, 0.0), 1.0, 1.0, 1.0, false));

        let data = control.sounds.get("wind").unwrap();
        assert_eq!(data.volume, 0.5);
        assert!((data.pitch - 0.7).abs() < 0.0001);
        assert_eq!(data.total, 1.0);
        assert_eq!(data.total_volume, 0.5);
        assert_eq!(data.sum, Vec2::new(16.0, 20.0));
    }

    #[test]
    fn play_fades_in_fades_out_switches_and_play_once_respects_current() {
        let mut control = SoundControl::new();
        let input = MusicPlayInput {
            should_play: true,
            delta: 60.0,
            music_volume: 0.5,
        };

        let start = control.play(Some("menu"), input);
        assert_eq!(
            start.action,
            MusicAction::Start {
                music: "menu".into(),
                looping: true,
                volume: 0.0,
            }
        );

        let fade = control.play(Some("menu"), input);
        assert_eq!(fade.action, MusicAction::SetVolume { volume: 0.25 });
        assert_eq!(control.fade, 0.5);

        let fading_out = control.play(
            Some("editor"),
            MusicPlayInput {
                delta: 30.0,
                ..input
            },
        );
        assert_eq!(
            fading_out.action,
            MusicAction::FadeOut {
                music: "menu".into(),
                volume: 0.125,
            }
        );

        let switched = control.play(Some("editor"), input);
        assert_eq!(
            switched.action,
            MusicAction::Switch {
                stop: "menu".into(),
                start: "editor".into(),
            }
        );

        let ignored = control.play_once(Some("boss1"), true);
        assert_eq!(ignored.action, MusicAction::None);

        control.current = None;
        let once = control.play_once(Some("boss1"), true);
        assert_eq!(
            once.action,
            MusicAction::Start {
                music: "boss1".into(),
                looping: false,
                volume: 1.0,
            }
        );
        assert_eq!(control.last_random_played.as_deref(), Some("boss1"));
    }

    #[test]
    fn update_filter_bus_random_and_dark_music_plans_are_deterministic() {
        assert_eq!(
            SoundControl::update_filter_plan(true, true),
            Some(FilterFadePlan {
                filter_index: 0,
                wet: 1.0,
                duration: 0.4,
            })
        );
        assert_eq!(SoundControl::update_filter_plan(false, true), None);

        let mut control = SoundControl::new();
        control.reload_default_music();
        control.last_random_played = Some("game1".into());
        assert_eq!(
            control.choose_random_track(false, false, 0).as_deref(),
            Some("game3")
        );
        assert_eq!(
            control.choose_random_track(true, false, 0).as_deref(),
            Some("boss1")
        );

        assert!(SoundControl::is_dark(
            Some(0.5),
            1.0,
            0.0,
            SoundDarkChance {
                wave_roll: 1.0,
                enemy_roll: 1.0,
            }
        ));
        assert!(SoundControl::is_dark(
            None,
            36.0,
            0.0,
            SoundDarkChance {
                wave_roll: 0.1,
                enemy_roll: 1.0,
            }
        ));

        assert_eq!(
            control.reset_event(1234),
            SoundBusPlan {
                stop_sound_bus: true,
                play_sound_bus: true,
                play_music_bus: false,
            }
        );
        assert_eq!(control.last_played_millis, 1234);
        assert_eq!(
            control.update_bus_playing(true),
            Some(SoundBusPlan {
                stop_sound_bus: false,
                play_sound_bus: true,
                play_music_bus: false,
            })
        );
    }

    #[test]
    fn update_loops_plan_starts_updates_stops_and_clears_accumulators() {
        let mut control = SoundControl::new();
        control.loop_sound("wind", Vec2::new(10.0, 0.0), 1.0, 2.0, 1.0, false);

        let plans = control.update_loops_plan(true, false, 1.0);
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].sound, "wind");
        assert_eq!(plans[0].action, LoopSoundAction::Start);
        assert!((plans[0].volume - 0.11).abs() < 0.0001);
        assert_eq!(plans[0].pitch, 2.0);
        assert_eq!(plans[0].pan, 10.0);

        let data = control.sounds.get_mut("wind").unwrap();
        data.sound_id = 1;
        data.cur_volume = 0.0005;
        let stopped = control.update_loops_plan(true, false, 1.0);
        assert_eq!(stopped[0].action, LoopSoundAction::Stop);
        assert_eq!(control.sounds.get("wind").unwrap().sound_id, -1);

        control.loop_sound("wind", Vec2::new(0.0, 0.0), 1.0, 1.0, 1.0, false);
        assert!(control.update_loops_plan(false, false, 1.0).is_empty());
        assert!(control.sounds.is_empty());
    }
}
