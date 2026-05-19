use std::collections::BTreeMap;

use crate::mindustry::world::meta::Attribute;

pub mod campaign;
pub mod distribution;
pub mod environment;
pub mod heat;
pub mod legacy;
pub mod liquid;
pub mod logic;
pub mod payloads;
pub mod power;
pub mod production;
pub mod sandbox;

pub trait UnitTetherBlock {
    fn spawned(&mut self, id: i32);
}

pub trait RotBlock {
    fn build_rotation(&self) -> f32;
}

pub trait ExplosionShield {
    fn absorb_explosion(&mut self, x: f32, y: f32, damage: f32) -> bool;
}

pub const TILE_BITMASK_VALUES: [i32; 256] = [
    39, 36, 39, 36, 27, 16, 27, 24, 39, 36, 39, 36, 27, 16, 27, 24, 38, 37, 38, 37, 17, 41, 17, 43,
    38, 37, 38, 37, 26, 21, 26, 25, 39, 36, 39, 36, 27, 16, 27, 24, 39, 36, 39, 36, 27, 16, 27, 24,
    38, 37, 38, 37, 17, 41, 17, 43, 38, 37, 38, 37, 26, 21, 26, 25, 3, 4, 3, 4, 15, 40, 15, 20, 3,
    4, 3, 4, 15, 40, 15, 20, 5, 28, 5, 28, 29, 10, 29, 23, 5, 28, 5, 28, 31, 11, 31, 32, 3, 4, 3,
    4, 15, 40, 15, 20, 3, 4, 3, 4, 15, 40, 15, 20, 2, 30, 2, 30, 9, 46, 9, 22, 2, 30, 2, 30, 14,
    44, 14, 6, 39, 36, 39, 36, 27, 16, 27, 24, 39, 36, 39, 36, 27, 16, 27, 24, 38, 37, 38, 37, 17,
    41, 17, 43, 38, 37, 38, 37, 26, 21, 26, 25, 39, 36, 39, 36, 27, 16, 27, 24, 39, 36, 39, 36, 27,
    16, 27, 24, 38, 37, 38, 37, 17, 41, 17, 43, 38, 37, 38, 37, 26, 21, 26, 25, 3, 0, 3, 0, 15, 42,
    15, 12, 3, 0, 3, 0, 15, 42, 15, 12, 5, 8, 5, 8, 29, 35, 29, 33, 5, 8, 5, 8, 31, 34, 31, 7, 3,
    0, 3, 0, 15, 42, 15, 12, 3, 0, 3, 0, 15, 42, 15, 12, 2, 1, 2, 1, 9, 45, 9, 19, 2, 1, 2, 1, 14,
    18, 14, 13,
];

pub fn tile_bitmask_value(mask: u8) -> i32 {
    TILE_BITMASK_VALUES[mask as usize]
}

pub fn tile_bitmask_region_names(name: &str) -> Vec<String> {
    (0..47).map(|index| format!("{name}-{index}")).collect()
}

pub fn tile_bitmask_variant_region_names(name: &str, variants: usize) -> Vec<Vec<String>> {
    (0..variants)
        .map(|variant| {
            (0..47)
                .map(|index| format!("{name}-{}-{index}", variant + 1))
                .collect()
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attributes {
    values: Vec<f32>,
}

impl Attributes {
    pub fn new(attribute_count: usize) -> Self {
        Self {
            values: vec![0.0; attribute_count],
        }
    }

    pub fn from_attributes(attributes: &[Attribute]) -> Self {
        let len = attributes
            .iter()
            .map(|attribute| attribute.id)
            .max()
            .map(|id| id + 1)
            .unwrap_or(0);
        Self::new(len)
    }

    pub fn clear(&mut self) {
        self.values.fill(0.0);
    }

    pub fn get(&self, attr: &Attribute) -> f32 {
        self.values.get(attr.id).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, attr: &Attribute, value: f32) {
        self.ensure_len(attr.id + 1);
        self.values[attr.id] = value;
    }

    pub fn add(&mut self, other: &Attributes) {
        self.add_scaled(other, 1.0);
    }

    pub fn add_scaled(&mut self, other: &Attributes, scale: f32) {
        self.ensure_len(other.values.len());
        for (index, value) in other.values.iter().enumerate() {
            self.values[index] += value * scale;
        }
    }

    pub fn json_entries(&self, attributes: &[Attribute]) -> BTreeMap<String, f32> {
        attributes
            .iter()
            .filter_map(|attribute| {
                let value = self.get(attribute);
                (value != 0.0).then(|| (attribute.name.clone(), value))
            })
            .collect()
    }

    pub fn read_json_entries(&mut self, attributes: &[Attribute], values: &BTreeMap<String, f32>) {
        self.ensure_len(
            attributes
                .iter()
                .map(|attribute| attribute.id)
                .max()
                .map(|id| id + 1)
                .unwrap_or(0),
        );
        for attribute in attributes {
            self.values[attribute.id] = values.get(&attribute.name).copied().unwrap_or(0.0);
        }
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    fn ensure_len(&mut self, len: usize) {
        if self.values.len() < len {
            self.values.resize(len, 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyTether {
        last_id: i32,
    }

    impl UnitTetherBlock for DummyTether {
        fn spawned(&mut self, id: i32) {
            self.last_id = id;
        }
    }

    struct DummyRot(f32);

    impl RotBlock for DummyRot {
        fn build_rotation(&self) -> f32 {
            self.0
        }
    }

    struct DummyShield {
        absorbed_damage: f32,
    }

    impl ExplosionShield for DummyShield {
        fn absorb_explosion(&mut self, _x: f32, _y: f32, damage: f32) -> bool {
            self.absorbed_damage += damage;
            true
        }
    }

    #[test]
    fn lightweight_block_interfaces_expose_upstream_contracts() {
        let mut tether = DummyTether { last_id: -1 };
        tether.spawned(42);
        assert_eq!(tether.last_id, 42);

        assert_eq!(DummyRot(90.0).build_rotation(), 90.0);

        let mut shield = DummyShield {
            absorbed_damage: 0.0,
        };
        assert!(shield.absorb_explosion(1.0, 2.0, 30.0));
        assert_eq!(shield.absorbed_damage, 30.0);
    }

    #[test]
    fn tile_bitmask_values_and_region_names_match_java_tables() {
        assert_eq!(TILE_BITMASK_VALUES.len(), 256);
        assert_eq!(tile_bitmask_value(0), 39);
        assert_eq!(tile_bitmask_value(127), 6);
        assert_eq!(tile_bitmask_value(255), 13);

        let names = tile_bitmask_region_names("wall");
        assert_eq!(names.len(), 47);
        assert_eq!(names[0], "wall-0");
        assert_eq!(names[46], "wall-46");

        let variants = tile_bitmask_variant_region_names("ore", 2);
        assert_eq!(variants[0][0], "ore-1-0");
        assert_eq!(variants[1][46], "ore-2-46");
    }

    #[test]
    fn attributes_clear_add_scale_and_json_entries_follow_upstream() {
        let attrs = Attribute::vanilla();
        let heat = &attrs[0];
        let spores = &attrs[1];

        let mut first = Attributes::from_attributes(&attrs);
        first.set(heat, 1.5);
        first.set(spores, -0.25);
        assert_eq!(first.get(heat), 1.5);

        let mut second = Attributes::from_attributes(&attrs);
        second.set(heat, 2.0);
        second.set(spores, 4.0);
        first.add_scaled(&second, 0.5);
        assert_eq!(first.get(heat), 2.5);
        assert_eq!(first.get(spores), 1.75);

        let json = first.json_entries(&attrs);
        assert_eq!(json.get("heat"), Some(&2.5));
        assert_eq!(json.get("spores"), Some(&1.75));
        assert!(!json.contains_key("water"));

        first.clear();
        assert_eq!(first.get(heat), 0.0);
        first.read_json_entries(&attrs, &json);
        assert_eq!(first.get(heat), 2.5);
        assert_eq!(first.get(spores), 1.75);
        assert_eq!(first.values().len(), attrs.len());
    }
}
