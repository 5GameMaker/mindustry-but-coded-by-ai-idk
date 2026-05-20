use super::{Weather, WeatherState};
use crate::mindustry::ctype::ContentId;

pub const RAIN_COLOR_RGBA: u32 = 0x7a95eaff;

#[derive(Debug, Clone, PartialEq)]
pub struct RainWeather {
    pub weather: Weather,
    pub yspeed: f32,
    pub xspeed: f32,
    pub padding: f32,
    pub density: f32,
    pub stroke: f32,
    pub size_min: f32,
    pub size_max: f32,
    pub splash_time_scale: f32,
    pub liquid: String,
    pub splashes: Vec<String>,
    pub color_rgba: u32,
}

impl RainWeather {
    pub const SPLASH_COUNT: usize = 12;

    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            weather: Weather::new(id, name),
            yspeed: 5.0,
            xspeed: 1.5,
            padding: 16.0,
            density: 1200.0,
            stroke: 0.75,
            size_min: 8.0,
            size_max: 40.0,
            splash_time_scale: 22.0,
            liquid: "water".to_string(),
            splashes: vec![String::new(); Self::SPLASH_COUNT],
            color_rgba: RAIN_COLOR_RGBA,
        }
    }

    pub fn name(&self) -> &str {
        self.weather.name()
    }

    pub fn load(&mut self) {
        self.splashes = (0..Self::SPLASH_COUNT)
            .map(|index| format!("splash-{index}"))
            .collect();
    }

    pub fn draw_over_plan(&self, state: &WeatherState) -> RainDrawPlan {
        RainDrawPlan {
            size_min: self.size_min,
            size_max: self.size_max,
            xspeed: self.xspeed,
            yspeed: self.yspeed,
            density: self.density,
            intensity: state.intensity,
            stroke: self.stroke,
            color_rgba: self.color_rgba,
        }
    }

    pub fn draw_under_plan(&self, state: &WeatherState) -> SplashDrawPlan {
        SplashDrawPlan {
            splashes: self.splashes.clone(),
            padding: self.size_max,
            density: self.density,
            intensity: state.intensity,
            opacity: state.opacity,
            time_scale: self.splash_time_scale,
            stroke: self.stroke,
            color_rgba: self.color_rgba,
            liquid: self.liquid.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RainDrawPlan {
    pub size_min: f32,
    pub size_max: f32,
    pub xspeed: f32,
    pub yspeed: f32,
    pub density: f32,
    pub intensity: f32,
    pub stroke: f32,
    pub color_rgba: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplashDrawPlan {
    pub splashes: Vec<String>,
    pub padding: f32,
    pub density: f32,
    pub intensity: f32,
    pub opacity: f32,
    pub time_scale: f32,
    pub stroke: f32,
    pub color_rgba: u32,
    pub liquid: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rain_weather_defaults_match_java_field_initializers() {
        let rain = RainWeather::new(0, "rain");

        assert_eq!(rain.name(), "rain");
        assert_eq!(rain.yspeed, 5.0);
        assert_eq!(rain.xspeed, 1.5);
        assert_eq!(rain.padding, 16.0);
        assert_eq!(rain.density, 1200.0);
        assert_eq!(rain.stroke, 0.75);
        assert_eq!(rain.size_min, 8.0);
        assert_eq!(rain.size_max, 40.0);
        assert_eq!(rain.splash_time_scale, 22.0);
        assert_eq!(rain.liquid, "water");
        assert_eq!(rain.splashes.len(), 12);
        assert!(rain.splashes.iter().all(String::is_empty));
        assert_eq!(rain.color_rgba, RAIN_COLOR_RGBA);
    }

    #[test]
    fn rain_weather_load_records_java_splash_region_names() {
        let mut rain = RainWeather::new(0, "rain");
        rain.load();

        assert_eq!(rain.splashes.len(), 12);
        assert_eq!(rain.splashes[0], "splash-0");
        assert_eq!(rain.splashes[11], "splash-11");
    }

    #[test]
    fn rain_weather_draw_plans_match_java_draw_over_under_arguments() {
        let mut rain = RainWeather::new(0, "rain");
        rain.load();
        let mut state = rain.weather.create_with_intensity(0.65);
        state.opacity = 0.4;

        let over = rain.draw_over_plan(&state);
        assert_eq!(
            over,
            RainDrawPlan {
                size_min: 8.0,
                size_max: 40.0,
                xspeed: 1.5,
                yspeed: 5.0,
                density: 1200.0,
                intensity: 0.65,
                stroke: 0.75,
                color_rgba: RAIN_COLOR_RGBA,
            }
        );

        let under = rain.draw_under_plan(&state);
        assert_eq!(under.padding, 40.0);
        assert_eq!(under.density, 1200.0);
        assert_eq!(under.intensity, 0.65);
        assert_eq!(under.opacity, 0.4);
        assert_eq!(under.time_scale, 22.0);
        assert_eq!(under.stroke, 0.75);
        assert_eq!(under.color_rgba, RAIN_COLOR_RGBA);
        assert_eq!(under.liquid, "water");
        assert_eq!(under.splashes[3], "splash-3");
    }
}
