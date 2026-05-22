pub const MAX_BLOCK_SIZE: i32 = 16;
pub const MAX_PIXEL_POLYGON_RADIUS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point2 {
    pub x: i32,
    pub y: i32,
}

impl Point2 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub fn get_facing_edge(
    block_size: i32,
    tile_x: i32,
    tile_y: i32,
    other_x: i32,
    other_y: i32,
) -> Point2 {
    if block_size <= 1 {
        return Point2::new(tile_x, tile_y);
    }

    Point2::new(
        tile_x + (other_x - tile_x).clamp(-((block_size - 1) / 2), block_size / 2),
        tile_y + (other_y - tile_y).clamp(-((block_size - 1) / 2), block_size / 2),
    )
}

pub fn get_edges(size: i32) -> Vec<Point2> {
    assert!(
        (1..=MAX_BLOCK_SIZE).contains(&size),
        "Block size must be between 1 and {MAX_BLOCK_SIZE}"
    );
    edge_ring(size, false)
}

pub fn get_inside_edges(size: i32) -> Vec<Point2> {
    assert!(
        (1..=MAX_BLOCK_SIZE).contains(&size),
        "Block size must be between 1 and {MAX_BLOCK_SIZE}"
    );
    edge_ring(size, true)
}

fn edge_ring(size: i32, inside: bool) -> Vec<Point2> {
    // Upstream precomputes arrays with zero-based `i = size - 1`.
    let i = size - 1;
    let bot = -((i as f32 / 2.0) as i32) - 1;
    let top = (i as f32 / 2.0 + 0.5) as i32 + 1;
    let inside_min = -((i as f32 / 2.0) as i32);
    let inside_max = (i as f32 / 2.0 + 0.5) as i32;

    let mut out = Vec::with_capacity((size as usize) * 4);
    for j in 0..size {
        out.push(Point2::new(bot + 1 + j, bot));
        out.push(Point2::new(bot + 1 + j, top));
        out.push(Point2::new(bot, bot + j + 1));
        out.push(Point2::new(top, bot + j + 1));
    }

    out.sort_by(|a, b| angle(a.x, a.y).total_cmp(&angle(b.x, b.y)));

    if inside {
        out.into_iter()
            .map(|point| {
                Point2::new(
                    point.x.clamp(inside_min, inside_max),
                    point.y.clamp(inside_min, inside_max),
                )
            })
            .collect()
    } else {
        out
    }
}

pub fn get_pixel_polygon(radius: f32) -> Vec<Vec2f> {
    if !(1.0..=MAX_PIXEL_POLYGON_RADIUS as f32).contains(&radius) {
        panic!("Polygon size must be between 1 and {MAX_PIXEL_POLYGON_RADIUS}");
    }

    // Arc's `Geometry.pixelCircle` returns a deterministic polygonal
    // approximation. Keeping this generated here avoids global caches while
    // preserving the same contract: a closed set of points on the requested
    // radius, ordered by angle.
    let steps = ((radius * 8.0).ceil() as usize).max(8);
    (0..steps)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / steps as f32;
            Vec2f::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect()
}

fn angle(x: i32, y: i32) -> f32 {
    let mut degrees = (y as f32).atan2(x as f32).to_degrees();
    if degrees < 0.0 {
        degrees += 360.0;
    }
    degrees
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_edge_clamps_to_multiblock_edge() {
        assert_eq!(get_facing_edge(1, 10, 10, 99, 99), Point2::new(10, 10));
        assert_eq!(get_facing_edge(2, 10, 10, 99, 99), Point2::new(11, 11));
        assert_eq!(get_facing_edge(3, 10, 10, 99, 8), Point2::new(11, 9));
        assert_eq!(get_facing_edge(4, 10, 10, 0, 99), Point2::new(9, 12));
    }

    #[test]
    fn one_by_one_edges_match_java_angle_sorted_order() {
        assert_eq!(
            get_edges(1),
            vec![
                Point2::new(1, 0),
                Point2::new(0, 1),
                Point2::new(-1, 0),
                Point2::new(0, -1),
            ]
        );
        assert_eq!(get_inside_edges(1), vec![Point2::new(0, 0); 4]);
    }

    #[test]
    fn larger_edges_have_four_sides_and_inside_points_are_clamped() {
        let edges = get_edges(3);
        let inside = get_inside_edges(3);

        assert_eq!(edges.len(), 12);
        assert_eq!(inside.len(), 12);
        assert!(edges.contains(&Point2::new(2, 0)));
        assert!(edges.contains(&Point2::new(-2, 0)));
        assert!(inside.iter().all(|point| (-1..=1).contains(&point.x)));
        assert!(inside.iter().all(|point| (-1..=1).contains(&point.y)));
    }

    #[test]
    #[should_panic(expected = "Block size must be between 1")]
    fn edges_reject_invalid_block_size() {
        get_edges(0);
    }

    #[test]
    fn pixel_polygon_is_radius_bounded_and_angle_ordered() {
        let polygon = get_pixel_polygon(2.0);
        assert_eq!(polygon.len(), 16);
        assert!((polygon[0].x - 2.0).abs() < 0.0001);
        assert!(polygon[0].y.abs() < 0.0001);
        for point in polygon {
            let len = (point.x * point.x + point.y * point.y).sqrt();
            assert!((len - 2.0).abs() < 0.0001);
        }
    }
}
