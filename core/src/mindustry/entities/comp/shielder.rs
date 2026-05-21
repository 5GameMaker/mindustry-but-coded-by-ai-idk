//! Shielder component interface mirroring upstream
//! `mindustry.entities.comp.ShielderComp`.

use crate::mindustry::entities::{comp::DamageComp, EntityPosition};
use crate::mindustry::io::TeamId;

pub trait DamageState {
    fn damage(&self) -> f32;

    fn set_damage(&mut self, damage: f32);
}

impl DamageState for DamageComp {
    fn damage(&self) -> f32 {
        self.damage
    }

    fn set_damage(&mut self, damage: f32) {
        self.damage = damage;
    }
}

pub trait TeamState {
    fn team(&self) -> TeamId;
}

pub trait ShielderComp: DamageState + TeamState + EntityPosition {
    /// Java default `void absorb(){}` hook.
    fn absorb(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct Shielded {
        damage: DamageComp,
        team: TeamId,
        x: f32,
        y: f32,
    }

    impl DamageState for Shielded {
        fn damage(&self) -> f32 {
            self.damage.damage()
        }

        fn set_damage(&mut self, damage: f32) {
            self.damage.set_damage(damage);
        }
    }

    impl TeamState for Shielded {
        fn team(&self) -> TeamId {
            self.team
        }
    }

    impl EntityPosition for Shielded {
        fn x(&self) -> f32 {
            self.x
        }

        fn y(&self) -> f32 {
            self.y
        }
    }

    impl ShielderComp for Shielded {}

    #[test]
    fn shielder_component_keeps_empty_absorb_default_with_required_facets() {
        let mut shielded = Shielded {
            damage: DamageComp::new(4.0),
            team: TeamId(2),
            x: 8.0,
            y: 9.0,
        };

        shielded.absorb();

        assert_eq!(shielded.damage(), 4.0);
        assert_eq!(shielded.team(), TeamId(2));
        assert_eq!((shielded.x(), shielded.y()), (8.0, 9.0));

        shielded.set_damage(0.0);
        assert_eq!(shielded.damage(), 0.0);
    }
}
