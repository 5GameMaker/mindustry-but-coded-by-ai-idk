//! Leg destruction draw data mirroring upstream `mindustry.entities.LegDestroyData`.

use crate::mindustry::io::Vec2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureRegionRef {
    pub name: String,
}

impl TextureRegionRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegDestroyData {
    pub a: Vec2,
    pub b: Vec2,
    pub region: TextureRegionRef,
}

impl LegDestroyData {
    pub fn new(a: Vec2, b: Vec2, region: TextureRegionRef) -> Self {
        Self { a, b, region }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leg_destroy_data_constructor_stores_points_and_region() {
        let data = LegDestroyData::new(
            Vec2::new(1.0, 2.0),
            Vec2::new(3.0, 4.0),
            TextureRegionRef::new("crawler-leg"),
        );

        assert_eq!(data.a, Vec2::new(1.0, 2.0));
        assert_eq!(data.b, Vec2::new(3.0, 4.0));
        assert_eq!(data.region, TextureRegionRef::new("crawler-leg"));
    }
}
