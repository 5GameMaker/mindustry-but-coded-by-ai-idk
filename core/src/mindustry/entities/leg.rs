//! Leg state data mirroring upstream `mindustry.entities.Leg`.

use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leg {
    pub joint: Vec2,
    pub base: Vec2,
    pub group: i32,
    pub moving: bool,
    pub stage: f32,
}

impl Default for Leg {
    fn default() -> Self {
        Self {
            joint: Vec2::new(0.0, 0.0),
            base: Vec2::new(0.0, 0.0),
            group: 0,
            moving: false,
            stage: 0.0,
        }
    }
}

impl Leg {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leg_defaults_match_java_field_initializers() {
        let leg = Leg::new();

        assert_eq!(leg.joint, Vec2::new(0.0, 0.0));
        assert_eq!(leg.base, Vec2::new(0.0, 0.0));
        assert_eq!(leg.group, 0);
        assert!(!leg.moving);
        assert_eq!(leg.stage, 0.0);
    }

    #[test]
    fn leg_fields_are_plain_mutable_state_like_java() {
        let mut leg = Leg::new();
        leg.joint = Vec2::new(1.0, 2.0);
        leg.base = Vec2::new(3.0, 4.0);
        leg.group = 2;
        leg.moving = true;
        leg.stage = 0.75;

        assert_eq!(leg.joint, Vec2::new(1.0, 2.0));
        assert_eq!(leg.base, Vec2::new(3.0, 4.0));
        assert_eq!(leg.group, 2);
        assert!(leg.moving);
        assert_eq!(leg.stage, 0.75);
    }
}
