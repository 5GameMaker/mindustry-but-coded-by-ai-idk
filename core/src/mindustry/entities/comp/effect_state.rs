//! Effect state component mirroring upstream
//! `mindustry.entities.comp.EffectStateComp`.

use crate::mindustry::entities::comp::DecalColor;

#[derive(Debug, Clone, PartialEq)]
pub struct EffectRenderInput<'a> {
    pub id: i32,
    pub color: DecalColor,
    pub time: f32,
    pub lifetime: f32,
    pub rotation: f32,
    pub x: f32,
    pub y: f32,
    pub data: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectStateComp {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub time: f32,
    pub lifetime: f32,
    pub color: DecalColor,
    pub effect_clip: f32,
    pub data: Option<String>,
}

impl EffectStateComp {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            time: 0.0,
            lifetime: 0.0,
            color: DecalColor::WHITE,
            effect_clip: 0.0,
            data: None,
        }
    }

    /// Java: `lifetime = effect.render(...)`.
    pub fn draw_with<F>(&mut self, render: F)
    where
        F: FnOnce(EffectRenderInput<'_>) -> f32,
    {
        self.lifetime = render(EffectRenderInput {
            id: self.id,
            color: self.color,
            time: self.time,
            lifetime: self.lifetime,
            rotation: self.rotation,
            x: self.x,
            y: self.y,
            data: self.data.as_deref(),
        });
    }

    pub fn clip_size(&self) -> f32 {
        self.effect_clip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_state_draw_updates_lifetime_from_effect_renderer() {
        let mut state = EffectStateComp::new(9);
        state.x = 1.0;
        state.y = 2.0;
        state.rotation = 45.0;
        state.time = 3.0;
        state.lifetime = 4.0;
        state.data = Some("payload".into());

        state.draw_with(|input| {
            assert_eq!(input.id, 9);
            assert_eq!((input.x, input.y, input.rotation), (1.0, 2.0, 45.0));
            assert_eq!((input.time, input.lifetime), (3.0, 4.0));
            assert_eq!(input.data, Some("payload"));
            8.0
        });

        assert_eq!(state.lifetime, 8.0);
    }

    #[test]
    fn effect_state_clip_size_matches_effect_clip() {
        let mut state = EffectStateComp::new(1);
        state.effect_clip = 32.0;

        assert_eq!(state.clip_size(), 32.0);
    }
}
