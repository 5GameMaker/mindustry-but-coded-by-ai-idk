//! Black overlay fade-in mirror of upstream `mindustry.ui.fragments.FadeInFragment`.

use crate::mindustry::entities::entity_group::Rect;

const DURATION: f32 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeInDrawPlan {
    pub rect: Rect,
    pub alpha: f32,
    pub touchable_disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeInFragment {
    time: f32,
    removed: bool,
}

impl Default for FadeInFragment {
    fn default() -> Self {
        Self {
            time: 0.0,
            removed: false,
        }
    }
}

impl FadeInFragment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&mut self) {
        self.time = 0.0;
        self.removed = false;
    }

    pub fn act(&mut self, _delta: f32) {
        self.time += 1.0 / DURATION;
        if self.time > 1.0 {
            self.removed = true;
        }
    }

    pub fn draw_plan(&self, graphics_width: f32, graphics_height: f32) -> Option<FadeInDrawPlan> {
        (!self.removed).then(|| FadeInDrawPlan {
            rect: Rect::new(0.0, 0.0, graphics_width, graphics_height),
            alpha: (1.0 - self.time).clamp(0.0, 1.0),
            touchable_disabled: true,
        })
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn removed(&self) -> bool {
        self.removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_in_draws_black_fullscreen_overlay_and_advances_by_java_duration() {
        let mut fragment = FadeInFragment::new();
        fragment.build();

        let before = fragment.draw_plan(800.0, 600.0).unwrap();
        assert_eq!(before.rect, Rect::new(0.0, 0.0, 800.0, 600.0));
        assert_eq!(before.alpha, 1.0);
        assert!(before.touchable_disabled);

        fragment.act(999.0);
        let after = fragment.draw_plan(800.0, 600.0).unwrap();
        assert_eq!(fragment.time(), 1.0 / 40.0);
        assert!((after.alpha - 0.975).abs() < 0.0001);
    }

    #[test]
    fn fade_in_removes_after_time_passes_one_like_java() {
        let mut fragment = FadeInFragment::new();
        for _ in 0..41 {
            fragment.act(1.0);
        }

        assert!(fragment.removed());
        assert!(fragment.draw_plan(1.0, 1.0).is_none());
    }
}
