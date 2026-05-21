//! Draw component interface mirroring upstream `mindustry.entities.comp.DrawComp`.

use crate::mindustry::entities::EntityPosition;

pub trait DrawComp: EntityPosition {
    /// Java: `float clipSize(){ return Float.MAX_VALUE; }`.
    fn clip_size(&self) -> f32 {
        f32::MAX
    }

    /// Java default draw hook is empty.
    fn draw(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Drawable {
        x: f32,
        y: f32,
    }

    impl EntityPosition for Drawable {
        fn x(&self) -> f32 {
            self.x
        }

        fn y(&self) -> f32 {
            self.y
        }
    }

    impl DrawComp for Drawable {}

    #[test]
    fn draw_component_defaults_match_java_empty_draw_hook() {
        let drawable = Drawable { x: 3.0, y: 4.0 };

        assert_eq!(drawable.x(), 3.0);
        assert_eq!(drawable.y(), 4.0);
        assert_eq!(drawable.clip_size(), f32::MAX);
        drawable.draw();
    }
}
