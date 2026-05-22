use std::time::{SystemTime, UNIX_EPOCH};

use crate::mindustry::{
    ctype::{Content, ContentId, ContentType, UnlockableContentBase},
    logic::{LogicWeatherEvent, LogicWeatherState, LOGIC_WEATHER_FADE_TIME},
};

pub mod magnetic_storm;
pub mod particle_weather;
pub mod rain_weather;
pub mod solar_flare;
pub use magnetic_storm::MagneticStorm;
pub use particle_weather::{
    ParticleDrawParticlesPlan, ParticleDrawPlan, ParticleNoiseLayerPlan, ParticleWeather,
};
pub use rain_weather::{RainDrawPlan, RainWeather, SplashDrawPlan};
pub use solar_flare::SolarFlare;

pub const TIME_TO_MINUTES: f32 = 60.0 * 60.0;
pub const WEATHER_DEFAULT_DURATION: f32 = 10.0 * TIME_TO_MINUTES;
pub const WEATHER_STATE_FADE_TIME: f32 = 60.0 * 4.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Weather {
    pub base: UnlockableContentBase,
    pub duration: f32,
    pub opacity_multiplier: f32,
    pub attrs: Vec<(String, f32)>,
    pub sound: String,
    pub sound_vol: f32,
    pub sound_vol_min: f32,
    pub sound_vol_osc_mag: f32,
    pub sound_vol_osc_scl: f32,
    pub hidden: bool,
    pub state_type: String,
    pub status: String,
    pub status_duration: f32,
    pub status_air: bool,
    pub status_ground: bool,
}

impl Weather {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self::with_state_type(id, name, "WeatherState")
    }

    pub fn with_state_type(
        id: ContentId,
        name: impl Into<String>,
        state_type: impl Into<String>,
    ) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Weather, name),
            duration: WEATHER_DEFAULT_DURATION,
            opacity_multiplier: 1.0,
            attrs: Vec::new(),
            sound: "none".to_string(),
            sound_vol: 0.1,
            sound_vol_min: 0.0,
            sound_vol_osc_mag: 0.0,
            sound_vol_osc_scl: 20.0,
            hidden: false,
            state_type: state_type.into(),
            status: "none".to_string(),
            status_duration: 60.0 * 2.0,
            status_air: true,
            status_ground: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.base.mappable.name
    }

    pub fn localized_name(&self) -> &str {
        self.base
            .localized_name
            .as_deref()
            .unwrap_or_else(|| self.name())
    }

    pub fn create(&self) -> WeatherState {
        self.create_with_intensity(1.0)
    }

    pub fn create_with_intensity(&self, intensity: f32) -> WeatherState {
        self.create_with_intensity_duration(intensity, self.duration)
    }

    pub fn create_with_intensity_duration(&self, intensity: f32, duration: f32) -> WeatherState {
        WeatherState::new(self.name(), intensity.clamp(0.0, 1.0), duration)
    }

    pub fn is_active(&self, states: &[WeatherState]) -> bool {
        self.instance(states).is_some()
    }

    pub fn instance<'a>(&self, states: &'a [WeatherState]) -> Option<&'a WeatherState> {
        states
            .iter()
            .find(|state| state.weather_name == self.name() && state.added)
    }

    pub fn remove(&self, states: &mut [WeatherState]) -> bool {
        if let Some(state) = states
            .iter_mut()
            .find(|state| state.weather_name == self.name() && state.added)
        {
            state.remove();
            true
        } else {
            false
        }
    }

    pub fn update(&self, _state: &mut WeatherState) {}

    pub fn update_effect(&self, state: &mut WeatherState, delta: f32) -> bool {
        if self.status == "none" {
            return false;
        }

        if state.effect_timer <= 0.0 {
            state.effect_timer = self.status_duration - 5.0;
            true
        } else {
            state.effect_timer -= delta;
            false
        }
    }

    pub fn draw_over(&self, _state: &WeatherState) {}

    pub fn draw_under(&self, _state: &WeatherState) {}

    pub fn is_hidden(&self) -> bool {
        true
    }

    pub fn create_weather_event(
        &self,
        intensity: f32,
        duration: f32,
        wind_x: f32,
        wind_y: f32,
    ) -> WeatherState {
        let mut state = self.create_with_intensity_duration(intensity, duration);
        state.wind_vector = (wind_x, wind_y);
        state
    }
}

impl Content for Weather {
    fn id(&self) -> ContentId {
        self.base.mappable.base.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::Weather
    }
}

impl std::fmt::Display for Weather {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.localized_name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherState {
    pub weather_name: String,
    pub intensity: f32,
    pub opacity: f32,
    pub life: f32,
    pub effect_timer: f32,
    pub wind_vector: (f32, f32),
    pub added: bool,
}

impl WeatherState {
    pub fn new(weather_name: impl Into<String>, intensity: f32, life: f32) -> Self {
        Self {
            weather_name: weather_name.into(),
            intensity: intensity.clamp(0.0, 1.0),
            opacity: 0.0,
            life,
            effect_timer: 0.0,
            wind_vector: (1.0, 0.0),
            added: true,
        }
    }

    pub fn update_life(&mut self, delta: f32) {
        if self.life < WEATHER_STATE_FADE_TIME {
            self.opacity = (self.life / WEATHER_STATE_FADE_TIME).min(self.opacity);
        } else {
            self.opacity += (1.0 - self.opacity) * 0.004 * delta;
        }

        self.life -= delta;
        if self.life < 0.0 {
            self.remove();
        }
    }

    pub fn remove(&mut self) {
        self.added = false;
    }

    pub fn to_logic_state(&self) -> LogicWeatherState {
        LogicWeatherState {
            active: self.added,
            life: self.life.max(0.0),
        }
    }

    pub fn from_logic_event(event: &LogicWeatherEvent) -> Self {
        Self {
            weather_name: event.weather_name.trim_start_matches('@').to_string(),
            intensity: 1.0,
            opacity: if event.active { 1.0 } else { 0.0 },
            life: event.life,
            effect_timer: 0.0,
            wind_vector: (1.0, 0.0),
            added: event.active,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherEntry {
    pub weather: String,
    pub min_frequency: f32,
    pub max_frequency: f32,
    pub min_duration: f32,
    pub max_duration: f32,
    pub cooldown: f32,
    pub intensity: f32,
    pub always: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherTriggerPlan {
    pub weather: String,
    pub intensity: f32,
    pub duration: f32,
    pub wind_x: f32,
    pub wind_y: f32,
}

impl WeatherEntry {
    pub fn new(weather: &Weather) -> Self {
        Self::with_ranges(
            weather,
            weather.duration * 2.0,
            weather.duration * 6.0,
            weather.duration / 2.0,
            weather.duration * 1.5,
        )
    }

    pub fn with_ranges(
        weather: &Weather,
        min_frequency: f32,
        max_frequency: f32,
        min_duration: f32,
        max_duration: f32,
    ) -> Self {
        Self {
            weather: weather.name().to_string(),
            min_frequency,
            max_frequency,
            min_duration,
            max_duration,
            cooldown: random_weather_entry_cooldown(min_frequency, max_frequency),
            intensity: 1.0,
            always: false,
        }
    }
}

impl Default for WeatherEntry {
    fn default() -> Self {
        Self {
            weather: "rain".to_string(),
            min_frequency: 0.0,
            max_frequency: 0.0,
            min_duration: 0.0,
            max_duration: 0.0,
            cooldown: 0.0,
            intensity: 1.0,
            always: false,
        }
    }
}

pub fn logic_weather_event_from_state(state: &WeatherState) -> LogicWeatherEvent {
    LogicWeatherEvent {
        weather_name: format!("@{}", state.weather_name),
        active: state.added,
        life: if state.added {
            state.life.max(LOGIC_WEATHER_FADE_TIME)
        } else {
            state.life.min(LOGIC_WEATHER_FADE_TIME).max(0.0)
        },
    }
}

pub fn reset_weather_cooldowns_on_play(entries: &mut [WeatherEntry]) {
    reset_weather_cooldowns_on_play_seeded(entries, random_seed());
}

pub fn reset_weather_cooldowns_on_play_seeded(entries: &mut [WeatherEntry], seed: u64) {
    let mut rng = WeatherRandom::new(seed);
    let mut order: Vec<usize> = (0..entries.len()).collect();
    shuffle_indices(&mut order, &mut rng);

    let mut sum = 0.0;
    for index in order {
        let cooldown = sum + weather_entry_cooldown(0.0, entries[index].max_frequency, rng.next());
        entries[index].cooldown = cooldown;
        sum += cooldown;
    }
}

pub fn update_weather_entries<F>(
    entries: &mut [WeatherEntry],
    delta: f32,
    mut is_active: F,
) -> Vec<WeatherTriggerPlan>
where
    F: FnMut(&str) -> bool,
{
    update_weather_entries_seeded(entries, delta, &mut is_active, random_seed())
}

pub fn update_weather_entries_seeded<F>(
    entries: &mut [WeatherEntry],
    delta: f32,
    mut is_active: F,
    seed: u64,
) -> Vec<WeatherTriggerPlan>
where
    F: FnMut(&str) -> bool,
{
    let mut rng = WeatherRandom::new(seed);
    let mut plans = Vec::new();

    for entry in entries {
        entry.cooldown -= delta;

        if (entry.cooldown < 0.0 || entry.always) && !is_active(&entry.weather) {
            let duration = if entry.always {
                f32::INFINITY
            } else {
                weather_entry_cooldown(entry.min_duration, entry.max_duration, rng.next())
            };
            entry.cooldown = duration
                + weather_entry_cooldown(entry.min_frequency, entry.max_frequency, rng.next());
            let (wind_x, wind_y) = random_wind_vector(&mut rng);

            plans.push(WeatherTriggerPlan {
                weather: entry.weather.clone(),
                intensity: entry.intensity,
                duration,
                wind_x,
                wind_y,
            });
        }
    }

    plans
}

fn random_weather_entry_cooldown(min_frequency: f32, max_frequency: f32) -> f32 {
    let mut rng = WeatherRandom::from_time();
    weather_entry_cooldown(min_frequency, max_frequency, rng.next())
}

fn weather_entry_cooldown(min_frequency: f32, max_frequency: f32, random_unit: f32) -> f32 {
    let unit = random_unit.clamp(0.0, 1.0);
    min_frequency + (max_frequency - min_frequency) * unit
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

fn random_wind_vector(rng: &mut WeatherRandom) -> (f32, f32) {
    let angle = rng.next() * std::f32::consts::PI * 2.0;
    (angle.cos(), angle.sin())
}

fn shuffle_indices(indices: &mut [usize], rng: &mut WeatherRandom) {
    for index in (1..indices.len()).rev() {
        let swap_with = (rng.next() * (index as f32 + 1.0)).floor() as usize;
        indices.swap(index, swap_with.min(index));
    }
}

#[derive(Debug, Clone)]
struct WeatherRandom {
    state: u64,
}

impl WeatherRandom {
    fn from_time() -> Self {
        Self::new(random_seed())
    }

    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }

    fn next(&mut self) -> f32 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let value = self.state.wrapping_mul(0x2545_f491_4f6c_dd1d);
        ((value >> 40) as f32) / ((1u64 << 24) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ctype::Content;

    #[test]
    fn weather_defaults_match_java_base_field_initializers() {
        let weather = Weather::new(7, "rain");
        assert_eq!(weather.id(), 7);
        assert_eq!(weather.content_type(), ContentType::Weather);
        assert_eq!(weather.name(), "rain");
        assert_eq!(weather.localized_name(), "rain");
        assert_eq!(weather.duration, 10.0 * 60.0 * 60.0);
        assert_eq!(weather.opacity_multiplier, 1.0);
        assert!(weather.attrs.is_empty());
        assert_eq!(weather.sound, "none");
        assert_eq!(weather.sound_vol, 0.1);
        assert_eq!(weather.sound_vol_min, 0.0);
        assert_eq!(weather.sound_vol_osc_mag, 0.0);
        assert_eq!(weather.sound_vol_osc_scl, 20.0);
        assert!(!weather.hidden);
        assert_eq!(weather.state_type, "WeatherState");
        assert_eq!(weather.status, "none");
        assert_eq!(weather.status_duration, 120.0);
        assert!(weather.status_air);
        assert!(weather.status_ground);
        assert!(weather.is_hidden());
        assert_eq!(weather.to_string(), "rain");
    }

    #[test]
    fn weather_create_clamps_intensity_sets_life_and_tracks_active_instance() {
        let weather = Weather::new(0, "sandstorm");
        let low = weather.create_with_intensity(-2.0);
        let high = weather.create_with_intensity_duration(2.0, 90.0);

        assert_eq!(low.intensity, 0.0);
        assert_eq!(low.life, weather.duration);
        assert!(low.added);
        assert_eq!(high.intensity, 1.0);
        assert_eq!(high.life, 90.0);

        let mut states = vec![high.clone()];
        assert!(weather.is_active(&states));
        assert_eq!(weather.instance(&states).unwrap().weather_name, "sandstorm");
        assert!(weather.remove(&mut states));
        assert!(!states[0].added);
        assert!(!weather.is_active(&states));
    }

    #[test]
    fn weather_state_update_fades_life_and_removes_after_expiry() {
        let mut state = WeatherState::new("rain", 1.0, WEATHER_STATE_FADE_TIME + 10.0);
        state.update_life(1.0);
        assert!(state.opacity > 0.0);
        assert_eq!(state.life, WEATHER_STATE_FADE_TIME + 9.0);
        assert!(state.added);

        state.life = 10.0;
        state.opacity = 0.5;
        state.update_life(20.0);
        assert!(!state.added);
    }

    #[test]
    fn weather_update_effect_uses_status_duration_timer_without_world_side_effects() {
        let mut weather = Weather::new(0, "sporestorm");
        let mut state = weather.create();
        assert!(!weather.update_effect(&mut state, 1.0));

        weather.status = "spore-slowed".into();
        weather.status_duration = 120.0;
        assert!(weather.update_effect(&mut state, 1.0));
        assert_eq!(state.effect_timer, 115.0);
        assert!(!weather.update_effect(&mut state, 5.0));
        assert_eq!(state.effect_timer, 110.0);
    }

    #[test]
    fn weather_create_weather_sets_wind_vector_like_remote_helper() {
        let weather = Weather::new(0, "rain");
        let state = weather.create_weather_event(0.75, 300.0, -0.2, 1.3);

        assert_eq!(state.weather_name, "rain");
        assert_eq!(state.intensity, 0.75);
        assert_eq!(state.life, 300.0);
        assert_eq!(state.wind_vector, (-0.2, 1.3));
    }

    #[test]
    fn weather_entry_defaults_follow_java_range_constructor_shape() {
        let weather = Weather::new(0, "rain");
        let entry = WeatherEntry::new(&weather);
        assert_eq!(entry.weather, "rain");
        assert_eq!(entry.min_frequency, weather.duration * 2.0);
        assert_eq!(entry.max_frequency, weather.duration * 6.0);
        assert_eq!(entry.min_duration, weather.duration / 2.0);
        assert_eq!(entry.max_duration, weather.duration * 1.5);
        assert!(entry.cooldown >= entry.min_frequency);
        assert!(entry.cooldown <= entry.max_frequency);
        assert_eq!(entry.intensity, 1.0);
        assert!(!entry.always);

        let default = WeatherEntry::default();
        assert_eq!(default.weather, "rain");
        assert_eq!(default.intensity, 1.0);
    }

    #[test]
    fn weather_entry_cooldown_uses_java_random_range_between_min_and_max() {
        assert_eq!(weather_entry_cooldown(10.0, 30.0, 0.0), 10.0);
        assert_eq!(weather_entry_cooldown(10.0, 30.0, 0.25), 15.0);
        assert_eq!(weather_entry_cooldown(10.0, 30.0, 1.0), 30.0);
        assert_eq!(weather_entry_cooldown(12.0, 12.0, 0.8), 12.0);

        let reversed = weather_entry_cooldown(30.0, 10.0, 0.25);
        assert_eq!(reversed, 25.0);
    }

    #[test]
    fn play_event_reset_shuffles_and_staggers_weather_cooldowns_like_java() {
        let weather = Weather::new(0, "rain");
        let mut entries = vec![
            WeatherEntry::with_ranges(&weather, 10.0, 20.0, 30.0, 40.0),
            WeatherEntry::with_ranges(&weather, 10.0, 50.0, 30.0, 40.0),
            WeatherEntry::with_ranges(&weather, 10.0, 80.0, 30.0, 40.0),
        ];
        for entry in &mut entries {
            entry.cooldown = -1.0;
        }

        reset_weather_cooldowns_on_play_seeded(&mut entries, 12345);

        assert!(entries.iter().all(|entry| entry.cooldown >= 0.0));
        assert!(
            entries[0].cooldown <= 20.0
                || entries[1].cooldown <= 50.0
                || entries[2].cooldown <= 80.0
        );
        assert!(entries
            .iter()
            .any(|entry| entry.cooldown > entry.max_frequency));

        let first_pass = entries
            .iter()
            .map(|entry| entry.cooldown)
            .collect::<Vec<_>>();
        reset_weather_cooldowns_on_play_seeded(&mut entries, 12345);
        let second_pass = entries
            .iter()
            .map(|entry| entry.cooldown)
            .collect::<Vec<_>>();
        assert_eq!(first_pass, second_pass);
    }

    #[test]
    fn update_weather_entries_samples_duration_resets_cooldown_and_emits_trigger_plan() {
        let weather = Weather::new(0, "sandstorm");
        let mut entries = vec![WeatherEntry::with_ranges(&weather, 10.0, 20.0, 30.0, 40.0)];
        entries[0].cooldown = 1.0;
        entries[0].intensity = 0.65;

        let plans = update_weather_entries_seeded(&mut entries, 2.0, |_| false, 999);

        assert_eq!(plans.len(), 1);
        let plan = &plans[0];
        assert_eq!(plan.weather, "sandstorm");
        assert_eq!(plan.intensity, 0.65);
        assert!(plan.duration >= 30.0);
        assert!(plan.duration <= 40.0);
        assert!(entries[0].cooldown >= plan.duration + 10.0);
        assert!(entries[0].cooldown <= plan.duration + 20.0);
        let wind_len = (plan.wind_x * plan.wind_x + plan.wind_y * plan.wind_y).sqrt();
        assert!((wind_len - 1.0).abs() < 0.0001);
    }

    #[test]
    fn update_weather_entries_respects_active_weather_and_always_entries() {
        let weather = Weather::new(0, "rain");
        let mut active_entries = vec![WeatherEntry::with_ranges(&weather, 10.0, 20.0, 30.0, 40.0)];
        active_entries[0].cooldown = -1.0;

        let plans = update_weather_entries_seeded(&mut active_entries, 1.0, |_| true, 1);
        assert!(plans.is_empty());
        assert_eq!(active_entries[0].cooldown, -2.0);

        let mut always_entries = vec![WeatherEntry::with_ranges(&weather, 10.0, 20.0, 30.0, 40.0)];
        always_entries[0].always = true;
        always_entries[0].cooldown = 999.0;

        let plans = update_weather_entries_seeded(&mut always_entries, 1.0, |_| false, 2);
        assert_eq!(plans.len(), 1);
        assert!(plans[0].duration.is_infinite());
        assert!(always_entries[0].cooldown.is_infinite());
    }

    #[test]
    fn weather_state_bridges_to_logic_weather_events() {
        let mut state = WeatherState::new("rain", 1.0, 10.0);
        let logic = state.to_logic_state();
        assert!(logic.active);
        assert_eq!(logic.life, 10.0);

        let event = logic_weather_event_from_state(&state);
        assert_eq!(
            event,
            LogicWeatherEvent {
                weather_name: "@rain".into(),
                active: true,
                life: LOGIC_WEATHER_FADE_TIME,
            }
        );

        state.remove();
        let event = logic_weather_event_from_state(&state);
        assert_eq!(event.weather_name, "@rain");
        assert!(!event.active);

        let restored = WeatherState::from_logic_event(&LogicWeatherEvent {
            weather_name: "@rain".into(),
            active: true,
            life: 240.0,
        });
        assert_eq!(restored.weather_name, "rain");
        assert!(restored.added);
        assert_eq!(restored.life, 240.0);
    }
}
