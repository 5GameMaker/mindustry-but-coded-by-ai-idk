use crate::mindustry::vars::TILE_SIZE;

pub const PHYSICS_LAYERS: i32 = 4;
pub const LAYER_GROUND: i32 = 0;
pub const LAYER_LEGS: i32 = 1;
pub const LAYER_FLYING: i32 = 2;
pub const LAYER_UNDERWATER: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitGroupMember {
    pub x: f32,
    pub y: f32,
    pub hit_size: f32,
    pub group_index: i32,
}

impl UnitGroupMember {
    pub const fn new(x: f32, y: f32, hit_size: f32) -> Self {
        Self {
            x,
            y,
            hit_size,
            group_index: -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitGroup {
    pub units: Vec<UnitGroupMember>,
    pub collision_layer: i32,
    pub positions: Vec<f32>,
    pub original_positions: Vec<f32>,
    pub valid: bool,
}

impl UnitGroup {
    pub fn new(units: Vec<UnitGroupMember>) -> Self {
        Self {
            units,
            collision_layer: LAYER_GROUND,
            positions: Vec::new(),
            original_positions: Vec::new(),
            valid: false,
        }
    }

    pub fn calculate_initial_formation(&mut self, collision_layer: i32) {
        self.collision_layer = collision_layer;
        self.positions = calculate_relative_positions(&mut self.units);
        self.original_positions = self.positions.clone();
        self.valid = true;
    }

    pub fn update_raycast(
        &mut self,
        index: usize,
        dest_x: f32,
        dest_y: f32,
        hit: Option<(i32, i32)>,
    ) {
        if self.collision_layer == LAYER_FLYING {
            return;
        }
        update_raycast_position(
            index,
            dest_x,
            dest_y,
            hit,
            &self.original_positions,
            &mut self.positions,
        );
    }
}

pub fn calculate_relative_positions(units: &mut [UnitGroupMember]) -> Vec<f32> {
    if units.is_empty() {
        return Vec::new();
    }

    let (mut cx, mut cy) = (0.0, 0.0);
    for unit in units.iter() {
        cx += unit.x;
        cy += unit.y;
    }
    cx /= units.len() as f32;
    cy /= units.len() as f32;

    let mut positions = vec![0.0; units.len() * 2];
    for (index, unit) in units.iter_mut().enumerate() {
        positions[index * 2] = unit.x - cx;
        positions[index * 2 + 1] = unit.y - cy;
        unit.group_index = index as i32;
    }
    positions
}

pub fn update_raycast_position(
    index: usize,
    dest_x: f32,
    dest_y: f32,
    hit: Option<(i32, i32)>,
    original_positions: &[f32],
    positions: &mut [f32],
) -> bool {
    let Some((hit_x, hit_y)) = hit else {
        return false;
    };
    let offset = index * 2;
    if offset + 1 >= original_positions.len() || offset + 1 >= positions.len() {
        return false;
    }

    let target_x = original_positions[offset] + dest_x;
    let target_y = original_positions[offset + 1] + dest_y;
    let mut vx = hit_x as f32 * TILE_SIZE as f32 - dest_x;
    let mut vy = hit_y as f32 * TILE_SIZE as f32 - dest_y;
    let len = (vx * vx + vy * vy).sqrt();
    let new_len = (len - TILE_SIZE as f32 - 4.0).max(0.0);

    if len > 0.00001 {
        let scale = new_len / len;
        vx *= scale;
        vy *= scale;
    } else {
        vx = 0.0;
        vy = 0.0;
    }

    positions[offset] = vx;
    positions[offset + 1] = vy;

    // Keep these computations visible to mirror Java's world-space ray target.
    let _world_target = (target_x, target_y);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} ~= {expected}"
        );
    }

    #[test]
    fn relative_positions_are_centered_and_assign_group_indices() {
        let mut units = vec![
            UnitGroupMember::new(0.0, 0.0, 8.0),
            UnitGroupMember::new(10.0, 0.0, 8.0),
            UnitGroupMember::new(10.0, 10.0, 8.0),
        ];

        let positions = calculate_relative_positions(&mut units);

        assert_close(positions[0], -20.0 / 3.0);
        assert_close(positions[1], -10.0 / 3.0);
        assert_close(positions[2], 10.0 / 3.0);
        assert_close(positions[3], -10.0 / 3.0);
        assert_close(positions[4], 10.0 / 3.0);
        assert_close(positions[5], 20.0 / 3.0);
        assert_eq!(
            units
                .iter()
                .map(|unit| unit.group_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn unit_group_calculate_initial_formation_sets_valid_original_positions_and_layer() {
        let mut group = UnitGroup::new(vec![
            UnitGroupMember::new(0.0, 0.0, 8.0),
            UnitGroupMember::new(8.0, 0.0, 8.0),
        ]);

        group.calculate_initial_formation(LAYER_LEGS);

        assert!(group.valid);
        assert_eq!(group.collision_layer, LAYER_LEGS);
        assert_eq!(group.positions, vec![-4.0, 0.0, 4.0, 0.0]);
        assert_eq!(group.original_positions, group.positions);
    }

    #[test]
    fn update_raycast_position_shortens_offset_before_collision_tile() {
        let original = vec![40.0, 0.0];
        let mut positions = original.clone();

        assert!(update_raycast_position(
            0,
            0.0,
            0.0,
            Some((5, 0)),
            &original,
            &mut positions,
        ));

        assert_eq!(positions, vec![28.0, 0.0]);
    }

    #[test]
    fn unit_group_skips_raycast_for_flying_layer() {
        let mut group = UnitGroup::new(vec![UnitGroupMember::new(40.0, 0.0, 8.0)]);
        group.calculate_initial_formation(LAYER_FLYING);
        group.original_positions = vec![40.0, 0.0];
        group.positions = vec![40.0, 0.0];

        group.update_raycast(0, 0.0, 0.0, Some((5, 0)));

        assert_eq!(group.positions, vec![40.0, 0.0]);
    }
}
