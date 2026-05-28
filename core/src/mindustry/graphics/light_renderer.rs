use crate::mindustry::{
    entities::effect::StandardEffectLightRenderPrimitive, graphics::drawf::LightDrawPlan,
};

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
}
