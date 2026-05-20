use super::{Weather, WeatherState};
use crate::mindustry::ctype::ContentId;

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleWeather {
    pub weather: Weather,
    pub particle_region: String,
    pub color_rgba: u32,
    pub region: Option<String>,
    pub yspeed: f32,
    pub xspeed: f32,
    pub padding: f32,
    pub size_min: f32,
    pub size_max: f32,
    pub density: f32,
    pub min_alpha: f32,
    pub max_alpha: f32,
    pub force: f32,
    pub noise_scale: f32,
    pub base_speed: f32,
    pub sin_scl_min: f32,
    pub sin_scl_max: f32,
    pub sin_mag_min: f32,
    pub sin_mag_max: f32,
    pub noise_color_rgba: u32,
    pub draw_noise: bool,
    pub draw_particles: bool,
    pub use_wind_vector: bool,
    pub random_particle_rotation: bool,
    pub noise_layers: i32,
    pub noise_layer_speed_m: f32,
    pub noise_layer_alpha_m: f32,
    pub noise_layer_scl_m: f32,
    pub noise_layer_color_m: f32,
    pub noise_path: String,
    pub noise: Option<String>,
}

impl ParticleWeather {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            weather: Weather::new(id, name),
            particle_region: "circle-shadow".to_string(),
            color_rgba: 0xffffffff,
            region: None,
            yspeed: -2.0,
            xspeed: 0.25,
            padding: 16.0,
            size_min: 2.4,
            size_max: 12.0,
            density: 1200.0,
            min_alpha: 1.0,
            max_alpha: 1.0,
            force: 0.0,
            noise_scale: 2000.0,
            base_speed: 6.1,
            sin_scl_min: 30.0,
            sin_scl_max: 80.0,
            sin_mag_min: 1.0,
            sin_mag_max: 7.0,
            noise_color_rgba: 0xffffffff,
            draw_noise: false,
            draw_particles: true,
            use_wind_vector: false,
            random_particle_rotation: false,
            noise_layers: 1,
            noise_layer_speed_m: 1.1,
            noise_layer_alpha_m: 0.8,
            noise_layer_scl_m: 0.99,
            noise_layer_color_m: 1.0,
            noise_path: "noiseAlpha".to_string(),
            noise: None,
        }
    }

    pub fn name(&self) -> &str {
        self.weather.name()
    }

    pub fn load_plan(&mut self) -> ParticleLoadPlan {
        self.region = Some(self.particle_region.clone());
        ParticleLoadPlan {
            region: self.particle_region.clone(),
            noise_texture: self
                .draw_noise
                .then(|| format!("sprites/{}.png", self.noise_path)),
        }
    }

    pub fn wind_speed(&self, state: &WeatherState) -> (f32, f32) {
        if self.use_wind_vector {
            let speed = self.base_speed * state.intensity;
            (state.wind_vector.0 * speed, state.wind_vector.1 * speed)
        } else {
            (self.xspeed, self.yspeed)
        }
    }

    pub fn impulse_plan(&self, state: &WeatherState, delta: f32) -> Option<(f32, f32)> {
        let speed = self.force * state.intensity * delta;
        (speed > 0.001).then(|| (state.wind_vector.0 * speed, state.wind_vector.1 * speed))
    }

    pub fn draw_over_plan(&self, state: &WeatherState) -> ParticleDrawPlan {
        let (windx, windy) = self.wind_speed(state);
        ParticleDrawPlan {
            windx,
            windy,
            noise_layers: self.noise_layer_plans(state, windx, windy),
            particles: self.draw_particles.then(|| ParticleDrawParticlesPlan {
                region: self
                    .region
                    .clone()
                    .unwrap_or_else(|| self.particle_region.clone()),
                color_rgba: self.color_rgba,
                size_min: self.size_min,
                size_max: self.size_max,
                density: self.density,
                intensity: state.intensity,
                opacity: state.opacity,
                windx,
                windy,
                min_alpha: self.min_alpha,
                max_alpha: self.max_alpha,
                sin_scl_min: self.sin_scl_min,
                sin_scl_max: self.sin_scl_max,
                sin_mag_min: self.sin_mag_min,
                sin_mag_max: self.sin_mag_max,
                random_particle_rotation: self.random_particle_rotation,
            }),
        }
    }

    pub fn noise_layer_plans(
        &self,
        state: &WeatherState,
        windx: f32,
        windy: f32,
    ) -> Vec<ParticleNoiseLayerPlan> {
        if !self.draw_noise {
            return Vec::new();
        }

        let mut plans = Vec::with_capacity(self.noise_layers.max(0) as usize);
        let mut sspeed = 1.0;
        let mut sscl = 1.0;
        let mut salpha = 1.0;
        let mut color_multiplier = 1.0;
        let mut offset = 0.0;
        for _ in 0..self.noise_layers.max(0) {
            plans.push(ParticleNoiseLayerPlan {
                noise_path: format!("sprites/{}.png", self.noise_path),
                color_rgba: scale_rgb(self.noise_color_rgba, color_multiplier),
                noise_scale: self.noise_scale * sscl,
                opacity: state.opacity * salpha * self.weather.opacity_multiplier,
                speed: sspeed
                    * if self.use_wind_vector {
                        1.0
                    } else {
                        self.base_speed
                    },
                intensity: state.intensity,
                windx,
                windy,
                offset,
            });
            sspeed *= self.noise_layer_speed_m;
            salpha *= self.noise_layer_alpha_m;
            sscl *= self.noise_layer_scl_m;
            offset += 0.29;
            color_multiplier *= self.noise_layer_color_m;
        }
        plans
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleLoadPlan {
    pub region: String,
    pub noise_texture: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleDrawPlan {
    pub windx: f32,
    pub windy: f32,
    pub noise_layers: Vec<ParticleNoiseLayerPlan>,
    pub particles: Option<ParticleDrawParticlesPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleNoiseLayerPlan {
    pub noise_path: String,
    pub color_rgba: u32,
    pub noise_scale: f32,
    pub opacity: f32,
    pub speed: f32,
    pub intensity: f32,
    pub windx: f32,
    pub windy: f32,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleDrawParticlesPlan {
    pub region: String,
    pub color_rgba: u32,
    pub size_min: f32,
    pub size_max: f32,
    pub density: f32,
    pub intensity: f32,
    pub opacity: f32,
    pub windx: f32,
    pub windy: f32,
    pub min_alpha: f32,
    pub max_alpha: f32,
    pub sin_scl_min: f32,
    pub sin_scl_max: f32,
    pub sin_mag_min: f32,
    pub sin_mag_max: f32,
    pub random_particle_rotation: bool,
}

fn scale_rgb(rgba: u32, multiplier: f32) -> u32 {
    let r = (((rgba >> 24) & 0xff) as f32 * multiplier).clamp(0.0, 255.0) as u32;
    let g = (((rgba >> 16) & 0xff) as f32 * multiplier).clamp(0.0, 255.0) as u32;
    let b = (((rgba >> 8) & 0xff) as f32 * multiplier).clamp(0.0, 255.0) as u32;
    let a = rgba & 0xff;
    (r << 24) | (g << 16) | (b << 8) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_weather_defaults_match_java_field_initializers() {
        let weather = ParticleWeather::new(0, "snowing");

        assert_eq!(weather.name(), "snowing");
        assert_eq!(weather.particle_region, "circle-shadow");
        assert_eq!(weather.color_rgba, 0xffffffff);
        assert_eq!(weather.region, None);
        assert_eq!(weather.yspeed, -2.0);
        assert_eq!(weather.xspeed, 0.25);
        assert_eq!(weather.padding, 16.0);
        assert_eq!(weather.size_min, 2.4);
        assert_eq!(weather.size_max, 12.0);
        assert_eq!(weather.density, 1200.0);
        assert_eq!(weather.min_alpha, 1.0);
        assert_eq!(weather.max_alpha, 1.0);
        assert_eq!(weather.force, 0.0);
        assert_eq!(weather.noise_scale, 2000.0);
        assert_eq!(weather.base_speed, 6.1);
        assert_eq!(weather.sin_scl_min, 30.0);
        assert_eq!(weather.sin_scl_max, 80.0);
        assert_eq!(weather.sin_mag_min, 1.0);
        assert_eq!(weather.sin_mag_max, 7.0);
        assert_eq!(weather.noise_color_rgba, 0xffffffff);
        assert!(!weather.draw_noise);
        assert!(weather.draw_particles);
        assert!(!weather.use_wind_vector);
        assert!(!weather.random_particle_rotation);
        assert_eq!(weather.noise_layers, 1);
        assert_eq!(weather.noise_layer_speed_m, 1.1);
        assert_eq!(weather.noise_layer_alpha_m, 0.8);
        assert_eq!(weather.noise_layer_scl_m, 0.99);
        assert_eq!(weather.noise_layer_color_m, 1.0);
        assert_eq!(weather.noise_path, "noiseAlpha");
        assert_eq!(weather.noise, None);
    }

    #[test]
    fn particle_weather_load_plan_records_region_and_optional_noise_texture() {
        let mut weather = ParticleWeather::new(0, "sandstorm");
        weather.particle_region = "particle".into();
        let plan = weather.load_plan();
        assert_eq!(plan.region, "particle");
        assert_eq!(plan.noise_texture, None);
        assert_eq!(weather.region.as_deref(), Some("particle"));

        weather.draw_noise = true;
        weather.noise_path = "fog".into();
        let plan = weather.load_plan();
        assert_eq!(plan.noise_texture.as_deref(), Some("sprites/fog.png"));
    }

    #[test]
    fn particle_weather_wind_and_impulse_follow_java_update_formula() {
        let mut weather = ParticleWeather::new(0, "sandstorm");
        let mut state = weather.weather.create_with_intensity(0.5);
        state.wind_vector = (2.0, -3.0);

        assert_eq!(weather.wind_speed(&state), (0.25, -2.0));
        weather.use_wind_vector = true;
        weather.base_speed = 6.0;
        assert_eq!(weather.wind_speed(&state), (6.0, -9.0));

        assert_eq!(weather.impulse_plan(&state, 1.0), None);
        weather.force = 0.1;
        assert_eq!(weather.impulse_plan(&state, 2.0), Some((0.2, -0.3)));
    }

    #[test]
    fn particle_weather_draw_plan_omits_disabled_noise_and_particles() {
        let mut weather = ParticleWeather::new(0, "snowing");
        weather.draw_particles = false;
        let mut state = weather.weather.create_with_intensity(0.75);
        state.opacity = 0.4;

        let plan = weather.draw_over_plan(&state);
        assert_eq!(plan.windx, 0.25);
        assert_eq!(plan.windy, -2.0);
        assert!(plan.noise_layers.is_empty());
        assert_eq!(plan.particles, None);
    }

    #[test]
    fn particle_weather_draw_plan_matches_noise_layers_and_particle_args() {
        let mut weather = ParticleWeather::new(0, "fog");
        weather.draw_noise = true;
        weather.noise_layers = 3;
        weather.noise_layer_speed_m = 2.0;
        weather.noise_layer_alpha_m = 0.7;
        weather.noise_layer_scl_m = 0.6;
        weather.noise_layer_color_m = 0.5;
        weather.noise_color_rgba = 0x808080ff;
        weather.weather.opacity_multiplier = 0.47;
        weather.use_wind_vector = true;
        weather.base_speed = 0.05;
        weather.particle_region = "circle-small".into();
        weather.random_particle_rotation = true;

        let mut state = weather.weather.create_with_intensity(0.5);
        state.opacity = 0.4;
        state.wind_vector = (2.0, 1.0);
        let plan = weather.draw_over_plan(&state);

        assert_eq!(plan.windx, 0.05);
        assert_eq!(plan.windy, 0.025);
        assert_eq!(plan.noise_layers.len(), 3);
        assert_eq!(plan.noise_layers[0].noise_scale, 2000.0);
        assert_eq!(plan.noise_layers[1].noise_scale, 1200.0);
        assert_eq!(plan.noise_layers[2].noise_scale, 720.0);
        assert!((plan.noise_layers[0].opacity - 0.188).abs() < 0.0001);
        assert_eq!(plan.noise_layers[0].speed, 1.0);
        assert_eq!(plan.noise_layers[1].speed, 2.0);
        assert_eq!(plan.noise_layers[2].speed, 4.0);
        assert_eq!(plan.noise_layers[1].offset, 0.29);
        assert_eq!(plan.noise_layers[2].color_rgba, 0x202020ff);

        let particles = plan.particles.unwrap();
        assert_eq!(particles.region, "circle-small");
        assert_eq!(particles.windx, 0.05);
        assert_eq!(particles.windy, 0.025);
        assert_eq!(particles.intensity, 0.5);
        assert_eq!(particles.opacity, 0.4);
        assert!(particles.random_particle_rotation);
    }
}
