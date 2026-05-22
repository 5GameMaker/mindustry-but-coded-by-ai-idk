//! Pure tile raycast traversal mirroring upstream `World#raycastEach`.
//!
//! This module returns or visits tile coordinates only; it does not depend on
//! global world state, effects, entities or networking.

/// Traverses the tile segment from `(start_x, start_y)` to `(end_x, end_y)`.
///
/// The order matches upstream `World#raycastEach`: both endpoints are visited,
/// and the intermediate coordinates follow the same Bresenham-style stepping.
#[must_use]
pub fn raycast_each(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let _ = raycast_until(start_x, start_y, end_x, end_y, |x, y| {
        points.push((x, y));
        false
    });
    points
}

/// Traverses the tile segment and calls `visit` for every coordinate.
///
/// Stops immediately when `visit` returns `true`; the return value indicates
/// whether traversal stopped early.
pub fn raycast_until<F>(start_x: i32, start_y: i32, end_x: i32, end_y: i32, mut visit: F) -> bool
where
    F: FnMut(i32, i32) -> bool,
{
    let mut x = start_x;
    let mut y = start_y;
    let dx = (end_x - x).abs();
    let sx = if x < end_x { 1 } else { -1 };
    let dy = (end_y - y).abs();
    let sy = if y < end_y { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        if visit(x, y) {
            return true;
        }

        if x == end_x && y == end_y {
            return false;
        }

        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }

        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{raycast_each, raycast_until};

    #[test]
    fn raycast_each_includes_both_endpoints_for_horizontal_line() {
        assert_eq!(
            raycast_each(2, 4, 6, 4),
            vec![(2, 4), (3, 4), (4, 4), (5, 4), (6, 4)]
        );
    }

    #[test]
    fn raycast_each_handles_vertical_line() {
        assert_eq!(
            raycast_each(3, 1, 3, 5),
            vec![(3, 1), (3, 2), (3, 3), (3, 4), (3, 5)]
        );
    }

    #[test]
    fn raycast_each_matches_bresenham_style_tile_order() {
        assert_eq!(
            raycast_each(0, 0, 5, 2),
            vec![(0, 0), (1, 0), (2, 1), (3, 1), (4, 2), (5, 2)]
        );

        assert_eq!(
            raycast_each(5, 2, 0, 0),
            vec![(5, 2), (4, 2), (3, 1), (2, 1), (1, 0), (0, 0)]
        );
    }

    #[test]
    fn raycast_until_can_stop_early() {
        let mut visited = Vec::new();
        let stopped = raycast_until(0, 0, 5, 5, |x, y| {
            visited.push((x, y));
            visited.len() >= 3
        });

        assert!(stopped);
        assert_eq!(visited, vec![(0, 0), (1, 1), (2, 2)]);
    }

    #[test]
    fn raycast_until_returns_false_when_reaching_destination() {
        let mut visited = Vec::new();
        let stopped = raycast_until(1, 1, 1, 1, |x, y| {
            visited.push((x, y));
            false
        });

        assert!(!stopped);
        assert_eq!(visited, vec![(1, 1)]);
    }
}
