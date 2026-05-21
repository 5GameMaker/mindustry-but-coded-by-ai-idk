//! Underwater movement component mirroring upstream
//! `mindustry.entities.comp.UnderwaterMoveComp`.

use crate::mindustry::io::TeamId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnderwaterDrawPlan {
    pub type_name: String,
    pub underwater_wrapper: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnderwaterMoveComp {
    pub type_name: String,
}

impl UnderwaterMoveComp {
    pub const LAYER_UNDERWATER: i32 = 3;

    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
        }
    }

    pub fn draw(&self) -> UnderwaterDrawPlan {
        UnderwaterDrawPlan {
            type_name: self.type_name.clone(),
            underwater_wrapper: true,
        }
    }

    pub fn collision_layer(&self) -> i32 {
        Self::LAYER_UNDERWATER
    }

    pub fn hittable(&self) -> bool {
        false
    }

    pub fn targetable(&self, _targeter: TeamId) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn underwater_move_draw_and_collision_layer_match_java_strategy() {
        let comp = UnderwaterMoveComp::new("naval-unit");

        assert_eq!(
            comp.draw(),
            UnderwaterDrawPlan {
                type_name: "naval-unit".into(),
                underwater_wrapper: true,
            }
        );
        assert_eq!(comp.collision_layer(), UnderwaterMoveComp::LAYER_UNDERWATER);
    }

    #[test]
    fn underwater_move_is_never_hittable_or_targetable_like_java_false_and_type() {
        let comp = UnderwaterMoveComp::new("sub");

        assert!(!comp.hittable());
        assert!(!comp.targetable(TeamId(1)));
    }
}
