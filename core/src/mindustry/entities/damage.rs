use crate::mindustry::entities::comp::apply_armor as apply_armor_formula;
use crate::mindustry::vars::TILE_SIZE;

pub const DAMAGE_FALLOFF: f32 = 0.4;

pub fn apply_armor(damage: f32, armor: f32) -> f32 {
    apply_armor_formula(damage, armor)
}

pub fn calculate_damage(distance: f32, radius: f32, damage: f32) -> f32 {
    let scaled = if radius <= 0.00001 {
        1.0
    } else {
        lerp(1.0 - distance / radius, 1.0, DAMAGE_FALLOFF)
    };
    damage * scaled
}

pub fn find_length(base_length: f32, laser_length: Option<f32>, pierce_length: Option<f32>) -> f32 {
    pierce_length.or(laser_length).unwrap_or(base_length)
}

pub fn pierce_result_length(
    base_length: f32,
    pierce_cap: i32,
    mut distances: Vec<f32>,
    max_absorb_distance: f32,
) -> f32 {
    distances.sort_by(f32::total_cmp);
    let pierced_length = if distances.len() < pierce_cap.max(0) as usize || pierce_cap <= 0 {
        base_length
    } else {
        distances[pierce_cap as usize - 1].max(6.0)
    };
    pierced_length.min(max_absorb_distance)
}

pub fn complete_damage_tiles(center_x: f32, center_y: f32, radius: f32) -> Vec<(i32, i32)> {
    let tile_size = TILE_SIZE as f32;
    let trad = (radius / tile_size) as i32;
    let cx = (center_x / tile_size).round() as i32;
    let cy = (center_y / tile_size).round() as i32;
    let mut out = Vec::new();

    for dx in -trad..=trad {
        for dy in -trad..=trad {
            if dx * dx + dy * dy <= trad * trad {
                out.push((cx + dx, cy + dy));
            }
        }
    }

    out
}

pub fn tile_damage_ray_count(radius_tiles: f32) -> i32 {
    (radius_tiles * 2.0 * std::f32::consts::PI).ceil() as i32
}

pub fn tile_damage_edge_scaled_damage(
    tile_x: i32,
    tile_y: i32,
    center_x: i32,
    center_y: i32,
    radius_tiles: f32,
    damage: f32,
) -> f32 {
    let edge_scale = 0.6;
    let rad2 = radius_tiles * radius_tiles;
    if rad2 <= f32::EPSILON {
        return damage;
    }
    let dx = tile_x - center_x;
    let dy = tile_y - center_y;
    let dst2 = (dx * dx + dy * dy) as f32;
    let mult = (1.0 - (dst2 / rad2) + edge_scale) / (1.0 + edge_scale);
    damage * mult
}

fn lerp(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
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
    fn apply_armor_matches_damage_java_helper() {
        assert_eq!(apply_armor(100.0, 25.0), 75.0);
        assert_eq!(apply_armor(100.0, 95.0), 10.0);
        assert_eq!(apply_armor(10.0, 100.0), 1.0);
    }

    #[test]
    fn calculate_damage_matches_radius_falloff_formula() {
        assert_eq!(calculate_damage(0.0, 100.0, 50.0), 50.0);
        assert_close(calculate_damage(50.0, 100.0, 50.0), 35.0);
        assert_eq!(calculate_damage(999.0, 0.0, 50.0), 50.0);
    }

    #[test]
    fn pierce_result_length_uses_sorted_distances_cap_and_absorber_limit() {
        assert_eq!(
            pierce_result_length(100.0, 0, vec![10.0], f32::INFINITY),
            100.0
        );
        assert_eq!(
            pierce_result_length(100.0, 2, vec![40.0, 5.0, 20.0], f32::INFINITY),
            20.0
        );
        assert_eq!(
            pierce_result_length(100.0, 2, vec![2.0, 4.0], f32::INFINITY),
            6.0
        );
        assert_eq!(pierce_result_length(100.0, 2, vec![20.0, 40.0], 15.0), 15.0);
    }

    #[test]
    fn complete_damage_tiles_match_java_radius_tile_circle() {
        let tiles = complete_damage_tiles(16.0, 16.0, 16.0);

        assert!(tiles.contains(&(2, 2)));
        assert!(tiles.contains(&(0, 2)));
        assert!(tiles.contains(&(4, 2)));
        assert!(!tiles.contains(&(0, 0)));
    }

    #[test]
    fn tile_damage_ray_count_and_edge_scaling_follow_java_formula() {
        assert_eq!(tile_damage_ray_count(10.0), 63);
        assert_close(
            tile_damage_edge_scaled_damage(10, 0, 0, 0, 10.0, 100.0),
            37.5,
        );
        assert_close(
            tile_damage_edge_scaled_damage(0, 0, 0, 0, 10.0, 100.0),
            100.0,
        );
    }

    #[test]
    fn find_length_prefers_pierce_then_laser_then_base() {
        assert_eq!(find_length(100.0, None, None), 100.0);
        assert_eq!(find_length(100.0, Some(80.0), None), 80.0);
        assert_eq!(find_length(100.0, Some(80.0), Some(60.0)), 60.0);
    }
}
