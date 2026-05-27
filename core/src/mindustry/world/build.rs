use crate::mindustry::{
    vars::TILE_SIZE,
    world::{get_edges, get_inside_edges, Point2},
};

pub const ORTHOGONAL_NEIGHBORS: [Point2; 4] = [
    Point2 { x: 1, y: 0 },
    Point2 { x: 0, y: 1 },
    Point2 { x: -1, y: 0 },
    Point2 { x: 0, y: -1 },
];

pub const ORTHOGONAL_WITH_CENTER_NEIGHBORS: [Point2; 5] = [
    Point2 { x: 1, y: 0 },
    Point2 { x: 0, y: 1 },
    Point2 { x: -1, y: 0 },
    Point2 { x: 0, y: -1 },
    Point2 { x: 0, y: 0 },
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn placement_bounds(
    tile_x: i32,
    tile_y: i32,
    block_size: i32,
    block_offset: f32,
) -> BuildBounds {
    let size = block_size.max(1) as f32 * TILE_SIZE as f32;
    BuildBounds {
        x: tile_x as f32 * TILE_SIZE as f32 + block_offset - size / 2.0,
        y: tile_y as f32 * TILE_SIZE as f32 + block_offset - size / 2.0,
        width: size,
        height: size,
    }
}

pub fn check_no_unit_overlap<F>(
    solid: bool,
    solidifies: bool,
    tile_x: i32,
    tile_y: i32,
    block_size: i32,
    block_offset: f32,
    mut any_entities: F,
) -> bool
where
    F: FnMut(BuildBounds) -> bool,
{
    (!solid && !solidifies)
        || !any_entities(placement_bounds(tile_x, tile_y, block_size, block_offset))
}

pub fn valid_break(
    tile_exists: bool,
    block_air: bool,
    block_can_break: bool,
    tile_breakable: bool,
    allow_environment_deconstruct: bool,
    interactable: bool,
) -> bool {
    tile_exists
        && !block_air
        && block_can_break
        && (tile_breakable || allow_environment_deconstruct)
        && interactable
}

pub fn contacts_ground<F>(x: i32, y: i32, block_size: i32, mut floor_is_liquid: F) -> bool
where
    F: FnMut(i32, i32) -> Option<bool>,
{
    if block_size > 1 {
        get_edges(block_size)
            .into_iter()
            .any(|point| floor_is_liquid(x + point.x, y + point.y) == Some(false))
    } else {
        ORTHOGONAL_NEIGHBORS
            .into_iter()
            .any(|point| floor_is_liquid(x + point.x, y + point.y) == Some(false))
    }
}

pub fn contacts_shallows<F>(x: i32, y: i32, block_size: i32, mut floor_is_deep: F) -> bool
where
    F: FnMut(i32, i32) -> Option<bool>,
{
    if block_size > 1 {
        get_inside_edges(block_size)
            .into_iter()
            .chain(get_edges(block_size))
            .any(|point| floor_is_deep(x + point.x, y + point.y) == Some(false))
    } else {
        ORTHOGONAL_NEIGHBORS
            .into_iter()
            .any(|point| floor_is_deep(x + point.x, y + point.y) == Some(false))
            || floor_is_deep(x, y) == Some(false)
    }
}

pub fn satisfies_water_requirement(
    requires_water: bool,
    placeable_liquid: bool,
    contacts_shallows: bool,
) -> bool {
    requires_water || placeable_liquid || contacts_shallows
}

pub fn footprint_tiles(center_x: i32, center_y: i32, block_size: i32) -> Vec<(i32, i32)> {
    let size = block_size.max(1);
    let offset = -((size - 1) / 2);
    let mut out = Vec::with_capacity((size * size) as usize);
    for dx in 0..size {
        for dy in 0..size {
            out.push((center_x + offset + dx, center_y + offset + dy));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orthogonal_neighbor_constants_match_arc_geometry_d4_and_d4c() {
        assert_eq!(
            ORTHOGONAL_NEIGHBORS,
            [
                Point2 { x: 1, y: 0 },
                Point2 { x: 0, y: 1 },
                Point2 { x: -1, y: 0 },
                Point2 { x: 0, y: -1 },
            ]
        );
        assert_eq!(
            ORTHOGONAL_WITH_CENTER_NEIGHBORS,
            [
                Point2 { x: 1, y: 0 },
                Point2 { x: 0, y: 1 },
                Point2 { x: -1, y: 0 },
                Point2 { x: 0, y: -1 },
                Point2 { x: 0, y: 0 },
            ]
        );
    }

    #[test]
    fn placement_bounds_and_unit_overlap_match_java_formula() {
        let bounds = placement_bounds(4, 5, 2, 4.0);
        assert_eq!(
            bounds,
            BuildBounds {
                x: 28.0,
                y: 36.0,
                width: 16.0,
                height: 16.0,
            }
        );

        assert!(check_no_unit_overlap(false, false, 4, 5, 2, 4.0, |_| true));
        assert!(!check_no_unit_overlap(true, false, 4, 5, 2, 4.0, |b| b == bounds));
        assert!(!check_no_unit_overlap(false, true, 4, 5, 2, 4.0, |_| true));
        assert!(check_no_unit_overlap(true, false, 4, 5, 2, 4.0, |_| false));
    }

    #[test]
    fn valid_break_requires_existing_non_air_breakable_interactable_tile() {
        assert!(valid_break(true, false, true, true, false, true));
        assert!(valid_break(true, false, true, false, true, true));
        assert!(!valid_break(false, false, true, true, false, true));
        assert!(!valid_break(true, true, true, true, false, true));
        assert!(!valid_break(true, false, false, true, false, true));
        assert!(!valid_break(true, false, true, false, false, true));
        assert!(!valid_break(true, false, true, true, false, false));
    }

    #[test]
    fn contacts_ground_uses_edges_for_multiblocks_and_d4_for_single_blocks() {
        assert!(contacts_ground(10, 10, 1, |x, y| {
            Some(x == 11 && y == 10).map(|hit| !hit)
        }));
        assert!(!contacts_ground(10, 10, 1, |_, _| Some(true)));

        assert!(contacts_ground(10, 10, 3, |x, y| {
            Some(!(x == 12 && y == 10))
        }));
    }

    #[test]
    fn contacts_shallows_checks_center_for_single_blocks_and_edges_for_multiblocks() {
        assert!(contacts_shallows(5, 5, 1, |x, y| Some(x != 5 || y != 5)));
        assert!(!contacts_shallows(5, 5, 1, |_, _| Some(true)));

        assert!(contacts_shallows(5, 5, 2, |x, y| {
            Some(!(x == 5 && y == 5))
        }));
    }

    #[test]
    fn water_requirement_and_footprint_helpers_cover_valid_place_primitives() {
        assert!(satisfies_water_requirement(true, false, false));
        assert!(satisfies_water_requirement(false, true, false));
        assert!(satisfies_water_requirement(false, false, true));
        assert!(!satisfies_water_requirement(false, false, false));

        assert_eq!(footprint_tiles(10, 20, 1), vec![(10, 20)]);
        assert_eq!(
            footprint_tiles(10, 20, 2),
            vec![(10, 20), (10, 21), (11, 20), (11, 21)]
        );
        assert_eq!(
            footprint_tiles(10, 20, 3),
            vec![
                (9, 19),
                (9, 20),
                (9, 21),
                (10, 19),
                (10, 20),
                (10, 21),
                (11, 19),
                (11, 20),
                (11, 21),
            ]
        );
    }
}
