use crate::mindustry::{
    ctype::{Content, ContentId, ContentType},
    r#type::{
        weather::{TIME_TO_MINUTES, WEATHER_DEFAULT_DURATION},
        ParticleWeather, RainWeather, Weather,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum WeatherContent {
    Rain(RainWeather),
    Particle(ParticleWeather),
}

impl WeatherContent {
    pub fn weather(&self) -> &Weather {
        match self {
            Self::Rain(weather) => &weather.weather,
            Self::Particle(weather) => &weather.weather,
        }
    }

    pub fn weather_mut(&mut self) -> &mut Weather {
        match self {
            Self::Rain(weather) => &mut weather.weather,
            Self::Particle(weather) => &mut weather.weather,
        }
    }

    pub fn name(&self) -> &str {
        self.weather().name()
    }

    pub fn as_rain(&self) -> Option<&RainWeather> {
        match self {
            Self::Rain(weather) => Some(weather),
            Self::Particle(_) => None,
        }
    }

    pub fn as_particle(&self) -> Option<&ParticleWeather> {
        match self {
            Self::Particle(weather) => Some(weather),
            Self::Rain(_) => None,
        }
    }
}

impl Content for WeatherContent {
    fn id(&self) -> ContentId {
        self.weather().id()
    }

    fn content_type(&self) -> ContentType {
        ContentType::Weather
    }
}

pub fn load() -> Vec<WeatherContent> {
    let mut next_id = 0;

    let mut snow = make_particle(&mut next_id, "snowing");
    snow.particle_region = "particle".into();
    snow.size_max = 13.0;
    snow.size_min = 2.6;
    snow.density = 1200.0;
    set_attr(&mut snow.weather, "light", -0.15);
    snow.weather.sound = "windHowl".into();
    snow.weather.sound_vol = 0.0;
    snow.weather.sound_vol_osc_mag = 1.5;
    snow.weather.sound_vol_osc_scl = 1100.0;
    snow.weather.sound_vol_min = 0.02;

    let mut rain = make_rain(&mut next_id, "rain");
    set_attr(&mut rain.weather, "light", -0.2);
    set_attr(&mut rain.weather, "water", 0.2);
    rain.weather.status = "wet".into();
    rain.weather.sound = "rain".into();
    rain.weather.sound_vol = 0.25;

    let mut sandstorm = make_particle(&mut next_id, "sandstorm");
    sandstorm.color_rgba = 0xf7cba4ff;
    sandstorm.noise_color_rgba = 0xf7cba4ff;
    sandstorm.particle_region = "particle".into();
    sandstorm.draw_noise = true;
    sandstorm.use_wind_vector = true;
    sandstorm.size_max = 140.0;
    sandstorm.size_min = 70.0;
    sandstorm.min_alpha = 0.0;
    sandstorm.max_alpha = 0.2;
    sandstorm.density = 1500.0;
    sandstorm.base_speed = 5.4;
    set_attr(&mut sandstorm.weather, "light", -0.1);
    set_attr(&mut sandstorm.weather, "water", -0.1);
    sandstorm.weather.opacity_multiplier = 0.35;
    sandstorm.force = 0.1;
    sandstorm.weather.sound = "wind".into();
    sandstorm.weather.sound_vol = 0.8;
    sandstorm.weather.duration = 7.0 * TIME_TO_MINUTES;

    let mut sporestorm = make_particle(&mut next_id, "sporestorm");
    sporestorm.color_rgba = 0x7457ceff;
    sporestorm.noise_color_rgba = 0x7457ceff;
    sporestorm.particle_region = "circle-small".into();
    sporestorm.draw_noise = true;
    sporestorm.weather.status_ground = false;
    sporestorm.use_wind_vector = true;
    sporestorm.size_max = 5.0;
    sporestorm.size_min = 2.5;
    sporestorm.min_alpha = 0.1;
    sporestorm.max_alpha = 0.8;
    sporestorm.density = 2000.0;
    sporestorm.base_speed = 4.3;
    set_attr(&mut sporestorm.weather, "spores", 1.0);
    set_attr(&mut sporestorm.weather, "light", -0.15);
    sporestorm.weather.status = "spore-slowed".into();
    sporestorm.weather.opacity_multiplier = 0.5;
    sporestorm.force = 0.1;
    sporestorm.weather.sound = "wind".into();
    sporestorm.weather.sound_vol = 0.7;
    sporestorm.weather.duration = 7.0 * TIME_TO_MINUTES;

    let mut fog = make_particle(&mut next_id, "fog");
    fog.weather.duration = 15.0 * TIME_TO_MINUTES;
    fog.noise_layers = 3;
    fog.noise_layer_scl_m = 0.6;
    fog.noise_layer_alpha_m = 0.7;
    fog.noise_layer_speed_m = 2.0;
    fog.base_speed = 0.05;
    fog.color_rgba = 0x666666ff;
    fog.noise_color_rgba = 0x666666ff;
    fog.noise_scale = 1100.0;
    fog.noise_path = "fog".into();
    fog.draw_particles = false;
    fog.draw_noise = true;
    fog.use_wind_vector = false;
    fog.xspeed = 1.0;
    fog.yspeed = 0.01;
    set_attr(&mut fog.weather, "light", -0.3);
    set_attr(&mut fog.weather, "water", 0.05);
    fog.weather.opacity_multiplier = 0.47;

    let mut suspend_particles = make_particle(&mut next_id, "suspend-particles");
    suspend_particles.color_rgba = 0xa7c1faff;
    suspend_particles.noise_color_rgba = 0xa7c1faff;
    suspend_particles.particle_region = "particle".into();
    suspend_particles.weather.status_ground = false;
    suspend_particles.use_wind_vector = true;
    suspend_particles.weather.hidden = true;
    suspend_particles.size_max = 4.0;
    suspend_particles.size_min = 1.4;
    suspend_particles.min_alpha = 0.5;
    suspend_particles.max_alpha = 1.0;
    suspend_particles.density = 10000.0;
    suspend_particles.base_speed = 0.03;

    vec![
        WeatherContent::Particle(snow),
        WeatherContent::Rain(rain),
        WeatherContent::Particle(sandstorm),
        WeatherContent::Particle(sporestorm),
        WeatherContent::Particle(fog),
        WeatherContent::Particle(suspend_particles),
    ]
}

fn make_particle(next_id: &mut ContentId, name: &str) -> ParticleWeather {
    let weather = ParticleWeather::new(*next_id, name);
    *next_id += 1;
    weather
}

fn make_rain(next_id: &mut ContentId, name: &str) -> RainWeather {
    let weather = RainWeather::new(*next_id, name);
    *next_id += 1;
    weather
}

fn set_attr(weather: &mut Weather, name: &str, value: f32) {
    if let Some((_, existing)) = weather.attrs.iter_mut().find(|(attr, _)| attr == name) {
        *existing = value;
    } else {
        weather.attrs.push((name.to_string(), value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_name<'a>(weathers: &'a [WeatherContent], name: &str) -> &'a WeatherContent {
        weathers
            .iter()
            .find(|weather| weather.name() == name)
            .unwrap_or_else(|| panic!("missing weather: {name}"))
    }

    fn attr(weather: &Weather, name: &str) -> f32 {
        weather
            .attrs
            .iter()
            .find(|(attr, _)| attr == name)
            .map(|(_, value)| *value)
            .unwrap_or_default()
    }

    #[test]
    fn vanilla_weather_ids_follow_upstream_registration_order() {
        let weathers = load();
        let names: Vec<_> = weathers.iter().map(WeatherContent::name).collect();
        assert_eq!(
            names,
            vec![
                "snowing",
                "rain",
                "sandstorm",
                "sporestorm",
                "fog",
                "suspend-particles",
            ]
        );

        for (index, weather) in weathers.iter().enumerate() {
            assert_eq!(weather.id(), index as i16);
            assert_eq!(weather.content_type(), ContentType::Weather);
        }
    }

    #[test]
    fn vanilla_weather_base_fields_match_upstream_subset() {
        let weathers = load();

        let snow = by_name(&weathers, "snowing").as_particle().unwrap();
        assert_eq!(snow.particle_region, "particle");
        assert_eq!(snow.size_max, 13.0);
        assert_eq!(snow.size_min, 2.6);
        assert_eq!(snow.density, 1200.0);
        assert_eq!(attr(&snow.weather, "light"), -0.15);
        assert_eq!(snow.weather.sound, "windHowl");
        assert_eq!(snow.weather.sound_vol, 0.0);
        assert_eq!(snow.weather.sound_vol_osc_mag, 1.5);
        assert_eq!(snow.weather.sound_vol_osc_scl, 1100.0);
        assert_eq!(snow.weather.sound_vol_min, 0.02);

        let rain = by_name(&weathers, "rain").as_rain().unwrap();
        assert_eq!(attr(&rain.weather, "light"), -0.2);
        assert_eq!(attr(&rain.weather, "water"), 0.2);
        assert_eq!(rain.weather.status, "wet");
        assert_eq!(rain.weather.sound, "rain");
        assert_eq!(rain.weather.sound_vol, 0.25);

        let sandstorm = by_name(&weathers, "sandstorm").as_particle().unwrap();
        assert_eq!(sandstorm.color_rgba, 0xf7cba4ff);
        assert_eq!(sandstorm.noise_color_rgba, 0xf7cba4ff);
        assert!(sandstorm.draw_noise);
        assert!(sandstorm.use_wind_vector);
        assert_eq!(sandstorm.size_max, 140.0);
        assert_eq!(sandstorm.size_min, 70.0);
        assert_eq!(sandstorm.min_alpha, 0.0);
        assert_eq!(sandstorm.max_alpha, 0.2);
        assert_eq!(sandstorm.density, 1500.0);
        assert_eq!(sandstorm.base_speed, 5.4);
        assert_eq!(attr(&sandstorm.weather, "light"), -0.1);
        assert_eq!(attr(&sandstorm.weather, "water"), -0.1);
        assert_eq!(sandstorm.weather.opacity_multiplier, 0.35);
        assert_eq!(sandstorm.force, 0.1);
        assert_eq!(sandstorm.weather.sound, "wind");
        assert_eq!(sandstorm.weather.sound_vol, 0.8);
        assert_eq!(sandstorm.weather.duration, 7.0 * TIME_TO_MINUTES);
    }

    #[test]
    fn spore_fog_and_suspend_particle_weather_match_upstream_subset() {
        let weathers = load();

        let sporestorm = by_name(&weathers, "sporestorm").as_particle().unwrap();
        assert_eq!(sporestorm.color_rgba, 0x7457ceff);
        assert_eq!(sporestorm.noise_color_rgba, 0x7457ceff);
        assert_eq!(sporestorm.particle_region, "circle-small");
        assert!(sporestorm.draw_noise);
        assert!(!sporestorm.weather.status_ground);
        assert!(sporestorm.use_wind_vector);
        assert_eq!(sporestorm.size_max, 5.0);
        assert_eq!(sporestorm.size_min, 2.5);
        assert_eq!(sporestorm.min_alpha, 0.1);
        assert_eq!(sporestorm.max_alpha, 0.8);
        assert_eq!(sporestorm.density, 2000.0);
        assert_eq!(sporestorm.base_speed, 4.3);
        assert_eq!(attr(&sporestorm.weather, "spores"), 1.0);
        assert_eq!(attr(&sporestorm.weather, "light"), -0.15);
        assert_eq!(sporestorm.weather.status, "spore-slowed");
        assert_eq!(sporestorm.weather.opacity_multiplier, 0.5);
        assert_eq!(sporestorm.force, 0.1);
        assert_eq!(sporestorm.weather.duration, 7.0 * TIME_TO_MINUTES);

        let fog = by_name(&weathers, "fog").as_particle().unwrap();
        assert_eq!(fog.weather.duration, 15.0 * TIME_TO_MINUTES);
        assert_eq!(fog.noise_layers, 3);
        assert_eq!(fog.noise_layer_scl_m, 0.6);
        assert_eq!(fog.noise_layer_alpha_m, 0.7);
        assert_eq!(fog.noise_layer_speed_m, 2.0);
        assert_eq!(fog.base_speed, 0.05);
        assert_eq!(fog.color_rgba, 0x666666ff);
        assert_eq!(fog.noise_color_rgba, 0x666666ff);
        assert_eq!(fog.noise_scale, 1100.0);
        assert_eq!(fog.noise_path, "fog");
        assert!(!fog.draw_particles);
        assert!(fog.draw_noise);
        assert!(!fog.use_wind_vector);
        assert_eq!(fog.xspeed, 1.0);
        assert_eq!(fog.yspeed, 0.01);
        assert_eq!(attr(&fog.weather, "light"), -0.3);
        assert_eq!(attr(&fog.weather, "water"), 0.05);
        assert_eq!(fog.weather.opacity_multiplier, 0.47);

        let suspend = by_name(&weathers, "suspend-particles")
            .as_particle()
            .unwrap();
        assert_eq!(suspend.color_rgba, 0xa7c1faff);
        assert_eq!(suspend.noise_color_rgba, 0xa7c1faff);
        assert_eq!(suspend.particle_region, "particle");
        assert!(!suspend.weather.status_ground);
        assert!(suspend.use_wind_vector);
        assert!(suspend.weather.hidden);
        assert_eq!(suspend.size_max, 4.0);
        assert_eq!(suspend.size_min, 1.4);
        assert_eq!(suspend.min_alpha, 0.5);
        assert_eq!(suspend.max_alpha, 1.0);
        assert_eq!(suspend.density, 10000.0);
        assert_eq!(suspend.base_speed, 0.03);
        assert_eq!(suspend.weather.duration, WEATHER_DEFAULT_DURATION);
    }
}
