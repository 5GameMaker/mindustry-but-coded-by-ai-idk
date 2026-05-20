use crate::mindustry::{
    ctype::{Content, ContentId, ContentType, UnlockableContentBase},
    logic::{LogicWeatherEvent, LogicWeatherState, LOGIC_WEATHER_FADE_TIME},
};

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
            cooldown: min_frequency,
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
        assert_eq!(entry.cooldown, entry.min_frequency);
        assert_eq!(entry.intensity, 1.0);
        assert!(!entry.always);

        let default = WeatherEntry::default();
        assert_eq!(default.weather, "rain");
        assert_eq!(default.intensity, 1.0);
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
