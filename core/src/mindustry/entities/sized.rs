//! Sized entity interface mirroring upstream `mindustry.entities.Sized`.
//!
//! Rust already has a prelude `Sized` marker trait, so the exported name is
//! `SizedEntity` while preserving the upstream `hitSize()` contract.

pub trait EntityPosition {
    fn x(&self) -> f32;

    fn y(&self) -> f32;
}

pub trait SizedEntity: EntityPosition {
    fn hit_size(&self) -> f32;
}

pub fn hit_radius(entity: &impl SizedEntity) -> f32 {
    entity.hit_size() / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Unit {
        x: f32,
        y: f32,
        hit_size: f32,
    }

    impl EntityPosition for Unit {
        fn x(&self) -> f32 {
            self.x
        }

        fn y(&self) -> f32 {
            self.y
        }
    }

    impl SizedEntity for Unit {
        fn hit_size(&self) -> f32 {
            self.hit_size
        }
    }

    #[test]
    fn sized_entity_exposes_position_and_hit_size_contract() {
        let unit = Unit {
            x: 12.0,
            y: 34.0,
            hit_size: 8.0,
        };

        assert_eq!(unit.x(), 12.0);
        assert_eq!(unit.y(), 34.0);
        assert_eq!(unit.hit_size(), 8.0);
        assert_eq!(hit_radius(&unit), 4.0);
    }
}
