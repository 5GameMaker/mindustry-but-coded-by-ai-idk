//! Elevation move component interface mirroring upstream
//! `mindustry.entities.comp.ElevationMoveComp`.

use crate::mindustry::entities::EntityPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolidPred {
    /// Java `EntityCollisions::solid`.
    EntityCollisionsSolid,
}

pub trait ElevationMoveComp: EntityPosition {
    fn is_flying(&self) -> bool;

    fn ignore_solids(&self) -> bool;

    /// Java replacement:
    /// `return isFlying() || ignoreSolids() ? null : EntityCollisions::solid;`.
    fn solidity(&self) -> Option<SolidPred> {
        if self.is_flying() || self.ignore_solids() {
            None
        } else {
            Some(SolidPred::EntityCollisionsSolid)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct UnitMove {
        x: f32,
        y: f32,
        flying: bool,
        ignore_solids: bool,
    }

    impl EntityPosition for UnitMove {
        fn x(&self) -> f32 {
            self.x
        }

        fn y(&self) -> f32 {
            self.y
        }
    }

    impl ElevationMoveComp for UnitMove {
        fn is_flying(&self) -> bool {
            self.flying
        }

        fn ignore_solids(&self) -> bool {
            self.ignore_solids
        }
    }

    #[test]
    fn elevation_move_solidity_matches_java_flying_or_ignore_solids_gate() {
        assert_eq!(
            UnitMove {
                x: 1.0,
                y: 2.0,
                flying: false,
                ignore_solids: false,
            }
            .solidity(),
            Some(SolidPred::EntityCollisionsSolid)
        );

        assert_eq!(
            UnitMove {
                x: 1.0,
                y: 2.0,
                flying: true,
                ignore_solids: false,
            }
            .solidity(),
            None
        );

        assert_eq!(
            UnitMove {
                x: 1.0,
                y: 2.0,
                flying: false,
                ignore_solids: true,
            }
            .solidity(),
            None
        );
    }
}
