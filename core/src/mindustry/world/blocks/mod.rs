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
pub mod storage;
pub mod units;

pub trait UnitTetherBlock {
    fn spawned(&mut self, id: i32);
}

pub trait RotBlock {
    fn build_rotation(&self) -> f32;
}

pub trait ExplosionShield {
    fn absorb_explosion(&mut self, x: f32, y: f32, damage: f32) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceMode {
    None,
    Bottom,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureRegionShell {
    pub x: f32,
    pub width: f32,
}

pub fn autotiler_sliced_region(input: TextureRegionShell, mode: SliceMode) -> TextureRegionShell {
    match mode {
        SliceMode::None => input,
        SliceMode::Bottom => TextureRegionShell {
            x: input.x + input.width,
            width: input.width / 2.0,
        },
        SliceMode::Top => TextureRegionShell {
            x: input.x,
            width: input.width / 2.0,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutotilerBlendInput {
    pub directional_blends: [bool; 4],
    pub world_blends: [bool; 4],
    pub non_square_world_neighbors: [bool; 4],
}

pub fn autotiler_transform_case(num: i32, bits: &mut [i32; 5]) {
    match num {
        0 => bits[0] = 3,
        1 => bits[0] = 4,
        2 => bits[0] = 2,
        3 => {
            bits[0] = 2;
            bits[2] = -1;
        }
        4 => {
            bits[0] = 1;
            bits[2] = -1;
        }
        5 => bits[0] = 1,
        _ => {}
    }
}

pub fn autotiler_build_blending(input: AutotilerBlendInput, check_world: bool) -> [i32; 5] {
    let blends = |direction: usize| {
        input.directional_blends[direction] || (check_world && input.world_blends[direction])
    };

    let mut result = [0, 1, 1, 0, 0];
    let num = if blends(2) && blends(1) && blends(3) {
        0
    } else if blends(1) && blends(3) {
        1
    } else if blends(1) && blends(2) {
        2
    } else if blends(3) && blends(2) {
        3
    } else if blends(1) {
        4
    } else if blends(3) {
        5
    } else {
        -1
    };
    autotiler_transform_case(num, &mut result);

    for i in 0..4 {
        if blends(i) {
            result[3] |= 1 << i;
        }
        if blends(i) && check_world && input.non_square_world_neighbors[i] {
            result[4] |= 1 << i;
        }
    }

    result
}

pub fn autotiler_direction(rotation: i32) -> (i32, i32) {
    match rotation.rem_euclid(4) {
        0 => (1, 0),
        1 => (0, 1),
        2 => (-1, 0),
        _ => (0, -1),
    }
}

pub fn autotiler_facing(x: i32, y: i32, rotation: i32, x2: i32, y2: i32) -> bool {
    let (dx, dy) = autotiler_direction(rotation);
    x + dx == x2 && y + dy == y2
}

pub fn autotiler_not_looking_at(
    tile_x: i32,
    tile_y: i32,
    other_x: i32,
    other_y: i32,
    other_rot: i32,
    other_rotated_output: bool,
) -> bool {
    !(other_rotated_output && autotiler_facing(other_x, other_y, other_rot, tile_x, tile_y))
}

pub fn autotiler_looking_at_either(
    tile_x: i32,
    tile_y: i32,
    rotation: i32,
    other_x: i32,
    other_y: i32,
    other_rot: i32,
    other_rotated_output: bool,
) -> bool {
    autotiler_facing(tile_x, tile_y, rotation, other_x, other_y)
        || !other_rotated_output
        || autotiler_facing(other_x, other_y, other_rot, tile_x, tile_y)
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
    fn autotiler_slice_transform_and_blending_follow_java_cases() {
        let region = TextureRegionShell {
            x: 10.0,
            width: 64.0,
        };
        assert_eq!(autotiler_sliced_region(region, SliceMode::None), region);
        assert_eq!(
            autotiler_sliced_region(region, SliceMode::Top),
            TextureRegionShell {
                x: 10.0,
                width: 32.0,
            }
        );
        assert_eq!(
            autotiler_sliced_region(region, SliceMode::Bottom),
            TextureRegionShell {
                x: 74.0,
                width: 32.0,
            }
        );

        let mut bits = [0, 1, 1, 0, 0];
        autotiler_transform_case(3, &mut bits);
        assert_eq!(bits, [2, 1, -1, 0, 0]);

        let input = AutotilerBlendInput {
            directional_blends: [false, true, true, false],
            world_blends: [true, false, false, true],
            non_square_world_neighbors: [true, false, false, true],
        };
        assert_eq!(autotiler_build_blending(input, false), [2, 1, 1, 0b0110, 0]);
        assert_eq!(
            autotiler_build_blending(input, true),
            [3, 1, 1, 0b1111, 0b1001]
        );
    }

    #[test]
    fn autotiler_direction_and_look_checks_match_four_way_geometry() {
        assert_eq!(autotiler_direction(0), (1, 0));
        assert_eq!(autotiler_direction(1), (0, 1));
        assert_eq!(autotiler_direction(2), (-1, 0));
        assert_eq!(autotiler_direction(-1), (0, -1));

        assert!(autotiler_facing(5, 5, 0, 6, 5));
        assert!(!autotiler_facing(5, 5, 1, 6, 5));
        assert!(!autotiler_not_looking_at(5, 5, 6, 5, 2, true));
        assert!(autotiler_not_looking_at(5, 5, 6, 5, 1, true));
        assert!(autotiler_looking_at_either(5, 5, 0, 6, 5, 1, true));
        assert!(autotiler_looking_at_either(5, 5, 1, 6, 5, 1, false));
        assert!(autotiler_looking_at_either(5, 5, 1, 6, 5, 2, true));
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
