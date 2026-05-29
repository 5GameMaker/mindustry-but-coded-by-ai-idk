/// Data-oriented environment renderer registry and planning layer.
///
/// The upstream Java implementation registers ad-hoc callbacks in
/// `Renderer.addEnvRenderer(...)`.  This Rust module keeps the selection
/// rules, command payloads, and bucketed plan data separate so a backend can
/// translate them into concrete draw calls.
use super::{RenderCommand, RenderPass, RenderPassKind, RenderProperty};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Env;

impl Env {
    pub const TERRESTRIAL: u32 = 1;
    pub const SPACE: u32 = 1 << 1;
    pub const UNDERWATER: u32 = 1 << 2;
    pub const SPORES: u32 = 1 << 3;
    pub const SCORCHING: u32 = 1 << 4;
    pub const GROUND_OIL: u32 = 1 << 5;
    pub const GROUND_WATER: u32 = 1 << 6;
    pub const OXYGEN: u32 = 1 << 7;
    pub const ANY: u32 = 0xffff_ffff;
    pub const NONE: u32 = 0;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvColor {
    Hex(&'static str),
    Named(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvBlendMode {
    Normal,
    Additive,
}

impl EnvBlendMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Additive => "additive",
        }
    }
}

impl EnvColor {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hex(hex) => hex,
            Self::Named(name) => name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvRenderCommandTemplate {
    FillRect {
        layer: &'static str,
        color: EnvColor,
        alpha: f32,
    },
    BlitTexture {
        layer: &'static str,
        resource: &'static str,
        blend: EnvBlendMode,
    },
    RayField {
        layer: &'static str,
        texture: &'static str,
        count: usize,
        time_scale: f32,
        windx: f32,
        windy: f32,
        opacity_min: f32,
        opacity_max: f32,
        opacity_multiplier: f32,
        rotation_range: f32,
        size_scale_min: f32,
        size_scale_max: f32,
        depth_fade_distance: f32,
    },
    Particles {
        layer: &'static str,
        region: &'static str,
        color: EnvColor,
        size_min: f32,
        size_max: f32,
        density: f32,
        intensity: f32,
        opacity: f32,
        windx: f32,
        windy: f32,
        min_alpha: f32,
        max_alpha: f32,
        sin_scl_min: f32,
        sin_scl_max: f32,
        sin_mag_min: f32,
        sin_mag_max: f32,
        random_particle_rotation: bool,
    },
    NoiseLayers {
        clear_layer: &'static str,
        fog_layer: &'static str,
        texture: &'static str,
        color: EnvColor,
        noise_scale: f32,
        opacity: f32,
        base_speed: f32,
        intensity: f32,
        vwindx: f32,
        vwindy: f32,
        layers: i32,
        layer_speed_m: f32,
        layer_alpha_m: f32,
        layer_scl_m: f32,
        layer_color_m: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvRenderCommand {
    FillRect {
        layer: &'static str,
        color: EnvColor,
        alpha: f32,
    },
    BlitTexture {
        layer: &'static str,
        resource: &'static str,
        blend: EnvBlendMode,
    },
    RayField {
        layer: &'static str,
        texture: &'static str,
        count: usize,
        time_scale: f32,
        windx: f32,
        windy: f32,
        opacity_min: f32,
        opacity_max: f32,
        opacity_multiplier: f32,
        rotation_range: f32,
        size_scale_min: f32,
        size_scale_max: f32,
        depth_fade_distance: f32,
    },
    Particles {
        layer: &'static str,
        region: &'static str,
        color: EnvColor,
        size_min: f32,
        size_max: f32,
        density: f32,
        intensity: f32,
        opacity: f32,
        windx: f32,
        windy: f32,
        min_alpha: f32,
        max_alpha: f32,
        sin_scl_min: f32,
        sin_scl_max: f32,
        sin_mag_min: f32,
        sin_mag_max: f32,
        random_particle_rotation: bool,
    },
    NoiseLayers {
        layer: &'static str,
        texture: &'static str,
        color: EnvColor,
        noise_scale: f32,
        opacity: f32,
        base_speed: f32,
        intensity: f32,
        vwindx: f32,
        vwindy: f32,
        layers: i32,
        layer_speed_m: f32,
        layer_alpha_m: f32,
        layer_scl_m: f32,
        layer_color_m: f32,
    },
}

impl EnvRenderCommand {
    pub const fn kind_label(self) -> &'static str {
        match self {
            Self::FillRect { .. } => "fill_rect",
            Self::BlitTexture { .. } => "blit_texture",
            Self::RayField { .. } => "ray_field",
            Self::Particles { .. } => "particles",
            Self::NoiseLayers { .. } => "noise_layers",
        }
    }

    pub const fn layer(self) -> &'static str {
        match self {
            Self::FillRect { layer, .. }
            | Self::BlitTexture { layer, .. }
            | Self::RayField { layer, .. }
            | Self::Particles { layer, .. }
            | Self::NoiseLayers { layer, .. } => layer,
        }
    }

    pub fn to_render_command(self, bucket: &'static str) -> RenderCommand {
        let mut properties = vec![
            RenderProperty::new("bucket", bucket),
            RenderProperty::new("kind", self.kind_label()),
            RenderProperty::new("layer", self.layer()),
        ];

        match self {
            Self::FillRect { color, alpha, .. } => {
                properties.push(RenderProperty::new("color", color.label()));
                properties.push(RenderProperty::new("alpha", alpha.to_string()));
            }
            Self::BlitTexture {
                resource, blend, ..
            } => {
                properties.push(RenderProperty::new("resource", resource));
                properties.push(RenderProperty::new("blend", blend.label()));
            }
            Self::RayField {
                texture,
                count,
                time_scale,
                windx,
                windy,
                ..
            } => {
                properties.push(RenderProperty::new("texture", texture));
                properties.push(RenderProperty::new("count", count.to_string()));
                properties.push(RenderProperty::new("time_scale", time_scale.to_string()));
                properties.push(RenderProperty::new("windx", windx.to_string()));
                properties.push(RenderProperty::new("windy", windy.to_string()));
            }
            Self::Particles {
                region,
                color,
                density,
                intensity,
                opacity,
                ..
            } => {
                properties.push(RenderProperty::new("region", region));
                properties.push(RenderProperty::new("color", color.label()));
                properties.push(RenderProperty::new("density", density.to_string()));
                properties.push(RenderProperty::new("intensity", intensity.to_string()));
                properties.push(RenderProperty::new("opacity", opacity.to_string()));
            }
            Self::NoiseLayers {
                texture,
                color,
                noise_scale,
                opacity,
                layers,
                ..
            } => {
                properties.push(RenderProperty::new("texture", texture));
                properties.push(RenderProperty::new("color", color.label()));
                properties.push(RenderProperty::new("noise_scale", noise_scale.to_string()));
                properties.push(RenderProperty::new("opacity", opacity.to_string()));
                properties.push(RenderProperty::new("layers", layers.to_string()));
            }
        }

        RenderCommand::custom(format!("env-{}", self.kind_label()), properties)
    }
}

impl EnvRenderCommandTemplate {
    pub fn resolve(self, fog: bool) -> EnvRenderCommand {
        match self {
            Self::FillRect {
                layer,
                color,
                alpha,
            } => EnvRenderCommand::FillRect {
                layer,
                color,
                alpha,
            },
            Self::BlitTexture {
                layer,
                resource,
                blend,
            } => EnvRenderCommand::BlitTexture {
                layer,
                resource,
                blend,
            },
            Self::RayField {
                layer,
                texture,
                count,
                time_scale,
                windx,
                windy,
                opacity_min,
                opacity_max,
                opacity_multiplier,
                rotation_range,
                size_scale_min,
                size_scale_max,
                depth_fade_distance,
            } => EnvRenderCommand::RayField {
                layer,
                texture,
                count,
                time_scale,
                windx,
                windy,
                opacity_min,
                opacity_max,
                opacity_multiplier,
                rotation_range,
                size_scale_min,
                size_scale_max,
                depth_fade_distance,
            },
            Self::Particles {
                layer,
                region,
                color,
                size_min,
                size_max,
                density,
                intensity,
                opacity,
                windx,
                windy,
                min_alpha,
                max_alpha,
                sin_scl_min,
                sin_scl_max,
                sin_mag_min,
                sin_mag_max,
                random_particle_rotation,
            } => EnvRenderCommand::Particles {
                layer,
                region,
                color,
                size_min,
                size_max,
                density,
                intensity,
                opacity,
                windx,
                windy,
                min_alpha,
                max_alpha,
                sin_scl_min,
                sin_scl_max,
                sin_mag_min,
                sin_mag_max,
                random_particle_rotation,
            },
            Self::NoiseLayers {
                clear_layer,
                fog_layer,
                texture,
                color,
                noise_scale,
                opacity,
                base_speed,
                intensity,
                vwindx,
                vwindy,
                layers,
                layer_speed_m,
                layer_alpha_m,
                layer_scl_m,
                layer_color_m,
            } => EnvRenderCommand::NoiseLayers {
                layer: if fog { fog_layer } else { clear_layer },
                texture,
                color,
                noise_scale,
                opacity,
                base_speed,
                intensity,
                vwindx,
                vwindy,
                layers,
                layer_speed_m,
                layer_alpha_m,
                layer_scl_m,
                layer_color_m,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvRendererRecipe {
    pub name: &'static str,
    pub env_mask: u32,
    pub planet_names: Vec<&'static str>,
    pub surface: Vec<EnvRenderCommandTemplate>,
    pub water: Vec<EnvRenderCommandTemplate>,
    pub space: Vec<EnvRenderCommandTemplate>,
    pub weather: Vec<EnvRenderCommandTemplate>,
    pub effects: Vec<EnvRenderCommandTemplate>,
}

impl EnvRendererRecipe {
    pub fn new(name: &'static str, env_mask: u32) -> Self {
        Self {
            name,
            env_mask,
            planet_names: Vec::new(),
            surface: Vec::new(),
            water: Vec::new(),
            space: Vec::new(),
            weather: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn with_planets(mut self, planets: &[&'static str]) -> Self {
        self.planet_names = planets.to_vec();
        self
    }

    pub fn with_surface(mut self, commands: Vec<EnvRenderCommandTemplate>) -> Self {
        self.surface = commands;
        self
    }

    pub fn with_water(mut self, commands: Vec<EnvRenderCommandTemplate>) -> Self {
        self.water = commands;
        self
    }

    pub fn with_space(mut self, commands: Vec<EnvRenderCommandTemplate>) -> Self {
        self.space = commands;
        self
    }

    pub fn with_weather(mut self, commands: Vec<EnvRenderCommandTemplate>) -> Self {
        self.weather = commands;
        self
    }

    pub fn with_effects(mut self, commands: Vec<EnvRenderCommandTemplate>) -> Self {
        self.effects = commands;
        self
    }

    pub fn matches(&self, context: &EnvRendererContext<'_>) -> bool {
        let env = context.effective_env();
        if (env & self.env_mask) != self.env_mask {
            return false;
        }

        if self.planet_names.is_empty() {
            return true;
        }

        context.planet.is_some_and(|planet| {
            self.planet_names
                .iter()
                .any(|candidate| *candidate == planet)
        })
    }

    pub fn underwater() -> Self {
        let (windx, windy) = wind_vector(45.0, 0.03);
        Self::new("underwater", Env::UNDERWATER)
            .with_surface(vec![EnvRenderCommandTemplate::FillRect {
                layer: "Layer.light + 1",
                color: EnvColor::Hex("353982"),
                alpha: 0.4,
            }])
            .with_water(vec![EnvRenderCommandTemplate::BlitTexture {
                layer: "Layer.light + 1",
                resource: "Shaders.caustics",
                blend: EnvBlendMode::Additive,
            }])
            .with_weather(vec![EnvRenderCommandTemplate::Particles {
                layer: "Layer.weather",
                region: "particle",
                color: EnvColor::Hex("a7c1fa"),
                size_min: 1.4,
                size_max: 4.0,
                density: 10000.0,
                intensity: 1.0,
                opacity: 1.0,
                windx,
                windy,
                min_alpha: 0.5,
                max_alpha: 1.0,
                sin_scl_min: 30.0,
                sin_scl_max: 80.0,
                sin_mag_min: 1.0,
                sin_mag_max: 7.0,
                random_particle_rotation: false,
            }])
            .with_effects(vec![EnvRenderCommandTemplate::RayField {
                layer: "Layer.light + 2",
                texture: "sprites/rays.png",
                count: 50,
                time_scale: 2000.0,
                windx,
                windy,
                opacity_min: 0.2,
                opacity_max: 0.7,
                opacity_multiplier: 0.7,
                rotation_range: 7.0,
                size_scale_min: 0.7,
                size_scale_max: 1.3,
                depth_fade_distance: 1000.0,
            }])
    }

    pub fn scorching() -> Self {
        Self::new("scorching", Env::SCORCHING).with_effects(vec![
            EnvRenderCommandTemplate::NoiseLayers {
                clear_layer: "Layer.weather - 1",
                fog_layer: "Layer.fogOfWar + 1",
                texture: "sprites/distortAlpha.png",
                color: EnvColor::Named("scarlet"),
                noise_scale: 1000.0,
                opacity: 0.24,
                base_speed: 0.4,
                intensity: 1.0,
                vwindx: 1.0,
                vwindy: 0.0,
                layers: 4,
                layer_speed_m: -1.3,
                layer_alpha_m: 0.7,
                layer_scl_m: 0.8,
                layer_color_m: 0.9,
            },
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvRendererContext<'a> {
    pub env: u32,
    pub planet: Option<&'a str>,
    pub planet_default_env: u32,
    pub draw_weather: bool,
    pub fog: bool,
}

impl<'a> EnvRendererContext<'a> {
    pub fn new(
        env: u32,
        planet: Option<&'a str>,
        planet_default_env: u32,
        draw_weather: bool,
        fog: bool,
    ) -> Self {
        Self {
            env,
            planet,
            planet_default_env,
            draw_weather,
            fog,
        }
    }

    pub fn effective_env(&self) -> u32 {
        if self.env == Env::ANY || self.env == Env::NONE {
            self.planet_default_env
        } else {
            self.env
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvRendererPlan {
    pub matched: Vec<&'static str>,
    pub surface: Vec<EnvRenderCommand>,
    pub water: Vec<EnvRenderCommand>,
    pub space: Vec<EnvRenderCommand>,
    pub weather: Vec<EnvRenderCommand>,
    pub effects: Vec<EnvRenderCommand>,
}

impl EnvRendererPlan {
    pub fn is_empty(&self) -> bool {
        self.matched.is_empty()
            && self.surface.is_empty()
            && self.water.is_empty()
            && self.space.is_empty()
            && self.weather.is_empty()
            && self.effects.is_empty()
    }

    pub fn render_command_count(&self) -> usize {
        self.surface.len()
            + self.water.len()
            + self.space.len()
            + self.weather.len()
            + self.effects.len()
    }

    pub fn to_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::with_capacity(self.render_command_count());
        Self::extend_bucket_commands(&mut commands, "surface", &self.surface);
        Self::extend_bucket_commands(&mut commands, "water", &self.water);
        Self::extend_bucket_commands(&mut commands, "space", &self.space);
        Self::extend_bucket_commands(&mut commands, "weather", &self.weather);
        Self::extend_bucket_commands(&mut commands, "effects", &self.effects);
        commands
    }

    pub fn to_render_pass(&self) -> Option<RenderPass> {
        let commands = self.to_render_commands();
        if commands.is_empty() {
            return None;
        }

        let mut pass = RenderPass::new(RenderPassKind::Environment);
        pass.extend(commands);
        Some(pass)
    }

    fn extend_bucket_commands(
        commands: &mut Vec<RenderCommand>,
        bucket: &'static str,
        source: &[EnvRenderCommand],
    ) {
        commands.extend(
            source
                .iter()
                .copied()
                .map(|command| command.to_render_command(bucket)),
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvRendererRegistry {
    pub recipes: Vec<EnvRendererRecipe>,
}

impl EnvRendererRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(EnvRendererRecipe::underwater());
        registry.register(EnvRendererRecipe::scorching());
        registry
    }

    pub fn register(&mut self, recipe: EnvRendererRecipe) {
        self.recipes.push(recipe);
    }

    pub fn plan(&self, context: &EnvRendererContext<'_>) -> EnvRendererPlan {
        let mut plan = EnvRendererPlan::default();

        for recipe in &self.recipes {
            if !recipe.matches(context) {
                continue;
            }

            plan.matched.push(recipe.name);
            plan.surface.extend(
                recipe
                    .surface
                    .iter()
                    .copied()
                    .map(|cmd| cmd.resolve(context.fog)),
            );
            plan.water.extend(
                recipe
                    .water
                    .iter()
                    .copied()
                    .map(|cmd| cmd.resolve(context.fog)),
            );
            plan.space.extend(
                recipe
                    .space
                    .iter()
                    .copied()
                    .map(|cmd| cmd.resolve(context.fog)),
            );

            if context.draw_weather {
                plan.weather.extend(
                    recipe
                        .weather
                        .iter()
                        .copied()
                        .map(|cmd| cmd.resolve(context.fog)),
                );
            }

            plan.effects.extend(
                recipe
                    .effects
                    .iter()
                    .copied()
                    .map(|cmd| cmd.resolve(context.fog)),
            );
        }

        plan
    }
}

impl Default for EnvRendererRegistry {
    fn default() -> Self {
        Self {
            recipes: Vec::new(),
        }
    }
}

fn wind_vector(angle_deg: f32, speed: f32) -> (f32, f32) {
    let radians = angle_deg.to_radians();
    (radians.cos() * speed, radians.sin() * speed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(left: f32, right: f32) {
        assert!(
            (left - right).abs() < 1e-6,
            "expected {left} to be within 1e-6 of {right}"
        );
    }

    #[test]
    fn env_flags_match_upstream_bit_layout() {
        assert_eq!(Env::TERRESTRIAL, 1);
        assert_eq!(Env::SPACE, 2);
        assert_eq!(Env::UNDERWATER, 4);
        assert_eq!(Env::SPORES, 8);
        assert_eq!(Env::SCORCHING, 16);
        assert_eq!(Env::GROUND_OIL, 32);
        assert_eq!(Env::GROUND_WATER, 64);
        assert_eq!(Env::OXYGEN, 128);
        assert_eq!(Env::ANY, 0xffff_ffff);
        assert_eq!(Env::NONE, 0);
    }

    #[test]
    fn default_registry_emits_underwater_and_scorching_plans_in_registration_order() {
        let registry = EnvRendererRegistry::with_defaults();
        let context = EnvRendererContext::new(
            Env::UNDERWATER | Env::SCORCHING,
            Some("erekir"),
            Env::TERRESTRIAL,
            true,
            false,
        );

        let plan = registry.plan(&context);
        assert_eq!(plan.matched, vec!["underwater", "scorching"]);
        assert_eq!(plan.surface.len(), 1);
        assert_eq!(plan.water.len(), 1);
        assert_eq!(plan.weather.len(), 1);
        assert_eq!(plan.effects.len(), 2);

        assert_eq!(
            plan.surface[0],
            EnvRenderCommand::FillRect {
                layer: "Layer.light + 1",
                color: EnvColor::Hex("353982"),
                alpha: 0.4,
            }
        );
        assert_eq!(
            plan.water[0],
            EnvRenderCommand::BlitTexture {
                layer: "Layer.light + 1",
                resource: "Shaders.caustics",
                blend: EnvBlendMode::Additive,
            }
        );

        match plan.weather[0] {
            EnvRenderCommand::Particles {
                layer,
                region,
                color,
                size_min,
                size_max,
                density,
                intensity,
                opacity,
                windx,
                windy,
                min_alpha,
                max_alpha,
                sin_scl_min,
                sin_scl_max,
                sin_mag_min,
                sin_mag_max,
                random_particle_rotation,
            } => {
                assert_eq!(layer, "Layer.weather");
                assert_eq!(region, "particle");
                assert_eq!(color, EnvColor::Hex("a7c1fa"));
                assert_eq!(size_min, 1.4);
                assert_eq!(size_max, 4.0);
                assert_eq!(density, 10000.0);
                assert_eq!(intensity, 1.0);
                assert_eq!(opacity, 1.0);
                approx_eq(windx, 0.03_f32 * 45.0_f32.to_radians().cos());
                approx_eq(windy, 0.03_f32 * 45.0_f32.to_radians().sin());
                assert_eq!(min_alpha, 0.5);
                assert_eq!(max_alpha, 1.0);
                assert_eq!(sin_scl_min, 30.0);
                assert_eq!(sin_scl_max, 80.0);
                assert_eq!(sin_mag_min, 1.0);
                assert_eq!(sin_mag_max, 7.0);
                assert!(!random_particle_rotation);
            }
            other => panic!("unexpected weather command: {other:?}"),
        }

        match plan.effects[0] {
            EnvRenderCommand::RayField {
                layer,
                texture,
                count,
                time_scale,
                windx,
                windy,
                opacity_min,
                opacity_max,
                opacity_multiplier,
                rotation_range,
                size_scale_min,
                size_scale_max,
                depth_fade_distance,
            } => {
                assert_eq!(layer, "Layer.light + 2");
                assert_eq!(texture, "sprites/rays.png");
                assert_eq!(count, 50);
                assert_eq!(time_scale, 2000.0);
                approx_eq(windx, 0.03_f32 * 45.0_f32.to_radians().cos());
                approx_eq(windy, 0.03_f32 * 45.0_f32.to_radians().sin());
                assert_eq!(opacity_min, 0.2);
                assert_eq!(opacity_max, 0.7);
                assert_eq!(opacity_multiplier, 0.7);
                assert_eq!(rotation_range, 7.0);
                assert_eq!(size_scale_min, 0.7);
                assert_eq!(size_scale_max, 1.3);
                assert_eq!(depth_fade_distance, 1000.0);
            }
            other => panic!("unexpected effect command: {other:?}"),
        }

        match plan.effects[1] {
            EnvRenderCommand::NoiseLayers {
                layer,
                texture,
                color,
                noise_scale,
                opacity,
                base_speed,
                intensity,
                vwindx,
                vwindy,
                layers,
                layer_speed_m,
                layer_alpha_m,
                layer_scl_m,
                layer_color_m,
            } => {
                assert_eq!(layer, "Layer.weather - 1");
                assert_eq!(texture, "sprites/distortAlpha.png");
                assert_eq!(color, EnvColor::Named("scarlet"));
                assert_eq!(noise_scale, 1000.0);
                assert_eq!(opacity, 0.24);
                assert_eq!(base_speed, 0.4);
                assert_eq!(intensity, 1.0);
                assert_eq!(vwindx, 1.0);
                assert_eq!(vwindy, 0.0);
                assert_eq!(layers, 4);
                assert_eq!(layer_speed_m, -1.3);
                assert_eq!(layer_alpha_m, 0.7);
                assert_eq!(layer_scl_m, 0.8);
                assert_eq!(layer_color_m, 0.9);
            }
            other => panic!("unexpected effect command: {other:?}"),
        }

        let pass = plan
            .to_render_pass()
            .expect("matched environment renderer should enter RenderFramePlan");
        assert_eq!(pass.kind, RenderPassKind::Environment);
        assert_eq!(pass.order, RenderPassKind::Environment.default_order());
        assert_eq!(pass.commands.len(), plan.render_command_count());
        assert!(matches!(
            &pass.commands[0],
            RenderCommand::Custom { name, properties }
                if name == "env-fill_rect"
                    && properties.iter().any(|property| property.key == "bucket" && property.value == "surface")
                    && properties.iter().any(|property| property.key == "color" && property.value == "353982")
        ));
        assert!(matches!(
            &pass.commands[1],
            RenderCommand::Custom { name, properties }
                if name == "env-blit_texture"
                    && properties.iter().any(|property| property.key == "bucket" && property.value == "water")
                    && properties.iter().any(|property| property.key == "resource" && property.value == "Shaders.caustics")
                    && properties.iter().any(|property| property.key == "blend" && property.value == "additive")
        ));
    }

    #[test]
    fn rules_env_any_falls_back_to_planet_default_env_for_selection() {
        let mut registry = EnvRendererRegistry::new();
        registry.register(
            EnvRendererRecipe::new("space-background", Env::SPACE).with_space(vec![
                EnvRenderCommandTemplate::BlitTexture {
                    layer: "Layer.space",
                    resource: "sprites/space.png",
                    blend: EnvBlendMode::Normal,
                },
            ]),
        );

        let context = EnvRendererContext::new(Env::ANY, Some("notva"), Env::SPACE, true, false);
        let plan = registry.plan(&context);

        assert_eq!(plan.matched, vec!["space-background"]);
        assert_eq!(plan.space.len(), 1);
        assert_eq!(
            plan.space[0],
            EnvRenderCommand::BlitTexture {
                layer: "Layer.space",
                resource: "sprites/space.png",
                blend: EnvBlendMode::Normal,
            }
        );
    }

    #[test]
    fn planet_filters_can_gate_renderer_selection_independently_from_env_bits() {
        let mut registry = EnvRendererRegistry::new();
        registry.register(
            EnvRendererRecipe::new("serpulo-only", Env::SCORCHING)
                .with_planets(&["serpulo"])
                .with_effects(vec![EnvRenderCommandTemplate::FillRect {
                    layer: "Layer.overlay",
                    color: EnvColor::Hex("ffffff"),
                    alpha: 0.1,
                }]),
        );

        let context = EnvRendererContext::new(
            Env::SCORCHING,
            Some("erekir"),
            Env::SCORCHING | Env::TERRESTRIAL,
            true,
            true,
        );

        let plan = registry.plan(&context);
        assert!(plan.is_empty());
    }

    #[test]
    fn fog_state_resolves_noise_layers_to_the_correct_layer_label() {
        let recipe = EnvRendererRecipe::scorching();

        let clear = recipe
            .effects
            .first()
            .copied()
            .expect("scorching recipe should have one effect")
            .resolve(false);
        let fog = recipe
            .effects
            .first()
            .copied()
            .expect("scorching recipe should have one effect")
            .resolve(true);

        assert_eq!(
            clear,
            EnvRenderCommand::NoiseLayers {
                layer: "Layer.weather - 1",
                texture: "sprites/distortAlpha.png",
                color: EnvColor::Named("scarlet"),
                noise_scale: 1000.0,
                opacity: 0.24,
                base_speed: 0.4,
                intensity: 1.0,
                vwindx: 1.0,
                vwindy: 0.0,
                layers: 4,
                layer_speed_m: -1.3,
                layer_alpha_m: 0.7,
                layer_scl_m: 0.8,
                layer_color_m: 0.9,
            }
        );
        assert_eq!(
            fog,
            EnvRenderCommand::NoiseLayers {
                layer: "Layer.fogOfWar + 1",
                texture: "sprites/distortAlpha.png",
                color: EnvColor::Named("scarlet"),
                noise_scale: 1000.0,
                opacity: 0.24,
                base_speed: 0.4,
                intensity: 1.0,
                vwindx: 1.0,
                vwindy: 0.0,
                layers: 4,
                layer_speed_m: -1.3,
                layer_alpha_m: 0.7,
                layer_scl_m: 0.8,
                layer_color_m: 0.9,
            }
        );
    }
}
