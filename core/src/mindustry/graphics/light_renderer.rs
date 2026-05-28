use crate::mindustry::{
    entities::effect::StandardEffectLightRenderPrimitive, graphics::drawf::LightDrawPlan,
};

use super::{RenderCommand, RenderPass, RenderPassKind, RenderPoint, RenderProperty};

pub const LIGHT_RENDER_LAYER: f32 = 50.0;

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLightCommand {
    pub x: f32,
    pub y: f32,
    pub region: String,
    pub rotation: f32,
    pub color: LightPrimitive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LightCommand {
    Runnable {
        label: String,
    },
    Region(RegionLightCommand),
    Line {
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        stroke: f32,
        tint: LightPrimitive,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRendererPlan {
    pub circle_lights: Vec<LightPrimitive>,
    pub commands: Vec<LightCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRendererState {
    pub enabled: bool,
    pub draw_light: bool,
    pub circle_lights: Vec<LightPrimitive>,
    pub commands: Vec<LightCommand>,
    pub circle_index: usize,
}

pub type LightPrimitive = LightDrawPlan;

fn light_color(light: LightPrimitive) -> [f32; 4] {
    [
        light.color.r,
        light.color.g,
        light.color.b,
        light.color.a * light.opacity,
    ]
}

fn float_property(key: &str, value: f32) -> RenderProperty {
    RenderProperty::new(key, value.to_string())
}

impl LightRendererPlan {
    pub fn render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::with_capacity(self.circle_lights.len() + self.commands.len());

        commands.extend(self.circle_lights.iter().copied().map(|light| {
            RenderCommand::draw_circle(
                RenderPoint::new(light.center.0, light.center.1),
                light.radius,
                light_color(light),
                true,
                LIGHT_RENDER_LAYER,
            )
        }));

        for command in &self.commands {
            match command {
                LightCommand::Runnable { label } => {
                    commands.push(RenderCommand::custom(
                        "light-runnable",
                        vec![RenderProperty::new("label", label.clone())],
                    ));
                }
                LightCommand::Region(region) => {
                    let color = light_color(region.color);
                    commands.push(RenderCommand::custom(
                        "light-region",
                        vec![
                            float_property("x", region.x),
                            float_property("y", region.y),
                            RenderProperty::new("region", region.region.clone()),
                            float_property("rotation", region.rotation),
                            float_property("r", color[0]),
                            float_property("g", color[1]),
                            float_property("b", color[2]),
                            float_property("a", color[3]),
                        ],
                    ));
                }
                LightCommand::Line {
                    x,
                    y,
                    x2,
                    y2,
                    stroke,
                    tint,
                } => {
                    commands.push(RenderCommand::draw_line(
                        RenderPoint::new(*x, *y),
                        RenderPoint::new(*x2, *y2),
                        *stroke,
                        light_color(*tint),
                        LIGHT_RENDER_LAYER,
                    ));
                }
            }
        }

        commands
    }

    pub fn to_render_pass(&self) -> Option<RenderPass> {
        let commands = self.render_commands();
        if commands.is_empty() {
            return None;
        }

        let mut pass = RenderPass::new(RenderPassKind::Lighting);
        pass.extend(commands);
        Some(pass)
    }

    pub fn into_render_pass(self) -> Option<RenderPass> {
        self.to_render_pass()
    }
}

impl Default for LightRendererState {
    fn default() -> Self {
        Self {
            enabled: true,
            draw_light: true,
            circle_lights: Vec::new(),
            commands: Vec::new(),
            circle_index: 0,
        }
    }
}

impl LightRendererState {
    pub fn enabled(&self) -> bool {
        self.enabled && self.draw_light
    }

    pub fn add_circle(&mut self, x: f32, y: f32, radius: f32, color: LightPrimitive) -> bool {
        if !self.enabled() || radius <= 0.0 {
            return false;
        }

        self.circle_lights.push(LightPrimitive {
            center: (x, y),
            radius,
            color: color.color,
            opacity: color.opacity,
        });
        self.circle_index = self.circle_lights.len();
        true
    }

    pub fn add_region(
        &mut self,
        x: f32,
        y: f32,
        region: impl Into<String>,
        rotation: f32,
        color: LightPrimitive,
    ) -> bool {
        if !self.enabled() {
            return false;
        }

        self.commands.push(LightCommand::Region(RegionLightCommand {
            x,
            y,
            region: region.into(),
            rotation,
            color,
        }));
        true
    }

    pub fn add_line(
        &mut self,
        x: f32,
        y: f32,
        x2: f32,
        y2: f32,
        stroke: f32,
        tint: LightPrimitive,
    ) -> bool {
        if !self.enabled() {
            return false;
        }

        self.commands.push(LightCommand::Line {
            x,
            y,
            x2,
            y2,
            stroke,
            tint,
        });
        true
    }

    pub fn add_runnable(&mut self, label: impl Into<String>) -> bool {
        if !self.enabled() {
            return false;
        }

        self.commands.push(LightCommand::Runnable {
            label: label.into(),
        });
        true
    }

    pub fn drain_plan(&mut self) -> LightRendererPlan {
        let plan = LightRendererPlan {
            circle_lights: std::mem::take(&mut self.circle_lights),
            commands: std::mem::take(&mut self.commands),
        };
        self.circle_index = 0;
        plan
    }

    pub fn as_effect_primitives(
        &self,
        color_symbol: &'static str,
    ) -> Vec<StandardEffectLightRenderPrimitive> {
        self.circle_lights
            .iter()
            .copied()
            .map(|light| light.as_effect_primitive(color_symbol))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::entities::comp::DecalColor;

    #[test]
    fn light_renderer_collects_circle_and_region_commands() {
        let mut renderer = LightRendererState::default();
        assert!(renderer.add_circle(
            1.0,
            2.0,
            3.0,
            LightPrimitive {
                center: (0.0, 0.0),
                radius: 0.0,
                color: DecalColor::WHITE,
                opacity: 0.5,
            }
        ));
        assert!(renderer.add_region(
            4.0,
            5.0,
            "circle-shadow",
            90.0,
            LightPrimitive {
                center: (0.0, 0.0),
                radius: 0.0,
                color: DecalColor::WHITE,
                opacity: 0.25,
            }
        ));
        assert!(renderer.add_line(
            6.0,
            7.0,
            8.0,
            9.0,
            10.0,
            LightPrimitive {
                center: (0.0, 0.0),
                radius: 0.0,
                color: DecalColor::WHITE,
                opacity: 0.1,
            }
        ));
        assert!(renderer.add_runnable("custom"));

        let plan = renderer.drain_plan();
        assert_eq!(plan.circle_lights.len(), 1);
        assert_eq!(plan.commands.len(), 3);
        assert_eq!(renderer.circle_index, 0);
    }

    #[test]
    fn light_renderer_plan_returns_none_for_empty_render_pass() {
        let plan = LightRendererPlan {
            circle_lights: Vec::new(),
            commands: Vec::new(),
        };

        assert_eq!(plan.to_render_pass(), None);
        assert_eq!(plan.into_render_pass(), None);
    }

    #[test]
    fn light_renderer_plan_maps_lights_to_lighting_render_pass() {
        let mut renderer = LightRendererState::default();
        let tint = LightPrimitive {
            center: (0.0, 0.0),
            radius: 0.0,
            color: DecalColor::from_rgba(0x20406080),
            opacity: 0.5,
        };

        assert!(renderer.add_circle(
            10.0,
            20.0,
            30.0,
            LightPrimitive {
                center: (0.0, 0.0),
                radius: 0.0,
                color: DecalColor::from_rgba(0x804020ff),
                opacity: 0.25,
            }
        ));
        assert!(renderer.add_line(1.0, 2.0, 3.0, 4.0, 5.0, tint));
        assert!(renderer.add_region(6.0, 7.0, "circle-shadow", 45.0, tint));
        assert!(renderer.add_runnable("custom-light"));

        let pass = renderer
            .drain_plan()
            .to_render_pass()
            .expect("non-empty light plan should produce lighting pass");

        assert_eq!(pass.kind, RenderPassKind::Lighting);
        assert_eq!(pass.order, RenderPassKind::Lighting.default_order());
        assert_eq!(pass.commands.len(), 4);

        match &pass.commands[0] {
            RenderCommand::DrawCircle {
                center,
                radius,
                color,
                filled,
                layer,
            } => {
                assert_eq!(*center, RenderPoint::new(10.0, 20.0));
                assert_eq!(*radius, 30.0);
                assert_eq!(*color, [0.5019608, 0.2509804, 0.1254902, 0.25]);
                assert!(*filled);
                assert_eq!(*layer, LIGHT_RENDER_LAYER);
            }
            other => panic!("expected circle light command, got {other:?}"),
        }

        match &pass.commands[1] {
            RenderCommand::DrawLine {
                from,
                to,
                stroke,
                color,
                layer,
            } => {
                assert_eq!(*from, RenderPoint::new(1.0, 2.0));
                assert_eq!(*to, RenderPoint::new(3.0, 4.0));
                assert_eq!(*stroke, 5.0);
                assert_eq!(*color, [0.1254902, 0.2509804, 0.3764706, 0.2509804]);
                assert_eq!(*layer, LIGHT_RENDER_LAYER);
            }
            other => panic!("expected line light command, got {other:?}"),
        }

        match &pass.commands[2] {
            RenderCommand::Custom { name, properties } => {
                assert_eq!(name, "light-region");
                assert_property(properties, "x", "6");
                assert_property(properties, "y", "7");
                assert_property(properties, "region", "circle-shadow");
                assert_property(properties, "rotation", "45");
                assert_property(properties, "r", "0.1254902");
                assert_property(properties, "g", "0.2509804");
                assert_property(properties, "b", "0.3764706");
                assert_property(properties, "a", "0.2509804");
            }
            other => panic!("expected region custom command, got {other:?}"),
        }

        match &pass.commands[3] {
            RenderCommand::Custom { name, properties } => {
                assert_eq!(name, "light-runnable");
                assert_property(properties, "label", "custom-light");
            }
            other => panic!("expected runnable custom command, got {other:?}"),
        }
    }

    #[test]
    fn light_renderer_draw_plan_can_convert_to_effect_primitives() {
        let mut renderer = LightRendererState::default();
        renderer.add_circle(
            1.0,
            2.0,
            3.0,
            LightPrimitive {
                center: (0.0, 0.0),
                radius: 0.0,
                color: DecalColor::from_rgba(0x112233ff),
                opacity: 0.4,
            },
        );

        let primitives = renderer.as_effect_primitives("Pal.accent");
        assert_eq!(primitives.len(), 1);
        assert_eq!(primitives[0].color, "Pal.accent");
        assert_eq!(
            primitives[0].color_rgba,
            Some(DecalColor::from_rgba(0x112233ff))
        );
    }

    fn assert_property(properties: &[RenderProperty], key: &str, value: &str) {
        let property = properties
            .iter()
            .find(|property| property.key == key)
            .unwrap_or_else(|| panic!("missing property {key}"));
        assert_eq!(property.value, value);
    }
}
