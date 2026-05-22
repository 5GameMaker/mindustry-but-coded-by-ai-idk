use crate::mindustry::io::Vec2;

const EPSILON: f32 = 1e-6;

/// Calculates the intercept point for a stationary shooter and a moving target.
///
/// `dst_vx` / `dst_vy` are delta-scaled velocities, matching upstream
/// `Predict.intercept(...)`, which divides target deltas by `Time.delta`.
pub fn intercept(
    src_x: f32,
    src_y: f32,
    dst_x: f32,
    dst_y: f32,
    dst_vx: f32,
    dst_vy: f32,
    bullet_speed: f32,
    delta: f32,
) -> Vec2 {
    let dst_vx = dst_vx / delta;
    let dst_vy = dst_vy / delta;
    let tx = dst_x - src_x;
    let ty = dst_y - src_y;

    let a = dst_vx * dst_vx + dst_vy * dst_vy - bullet_speed * bullet_speed;
    let b = 2.0 * (dst_vx * tx + dst_vy * ty);
    let c = tx * tx + ty * ty;

    let mut solution = Vec2::new(dst_x, dst_y);
    if let Some(times) = quad(a, b, c) {
        let t0 = times.x;
        let t1 = times.y;
        let mut t = t0.min(t1);
        if t < 0.0 {
            t = t0.max(t1);
        }
        if t > 0.0 {
            solution = Vec2::new(dst_x + dst_vx * t, dst_y + dst_vy * t);
        }
    }

    solution
}

pub fn intercept_positions(
    src: Vec2,
    dst: Vec2,
    dst_delta: Vec2,
    src_delta: Vec2,
    use_src_velocity: bool,
    bullet_speed: f32,
    delta: f32,
) -> Vec2 {
    let mut ddx = dst_delta.x;
    let mut ddy = dst_delta.y;
    if use_src_velocity {
        ddx -= src_delta.x;
        ddy -= src_delta.y;
    }
    intercept(src.x, src.y, dst.x, dst.y, ddx, ddy, bullet_speed, delta)
}

fn quad(a: f32, b: f32, c: f32) -> Option<Vec2> {
    if a.abs() < EPSILON {
        if b.abs() < EPSILON {
            if c.abs() < EPSILON {
                Some(Vec2::new(0.0, 0.0))
            } else {
                None
            }
        } else {
            // Mirrors upstream exactly: the linear solution writes into a
            // scratch vector but does not assign `sol`, so callers treat it as
            // no positive intercept solution.
            None
        }
    } else {
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            let disc = disc.sqrt();
            let denom = 2.0 * a;
            Some(Vec2::new((-b - disc) / denom, (-b + disc) / denom))
        } else {
            None
        }
    }
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
    fn intercept_returns_target_for_stationary_target() {
        let result = intercept(0.0, 0.0, 10.0, 5.0, 0.0, 0.0, 3.0, 1.0);

        assert_eq!(result, Vec2::new(10.0, 5.0));
    }

    #[test]
    fn intercept_leads_target_when_positive_solution_exists() {
        let result = intercept(0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 5.0, 1.0);

        assert_close(result.x, 12.5);
        assert_close(result.y, 0.0);
    }

    #[test]
    fn intercept_uses_delta_scaled_velocities() {
        let result_delta_one = intercept(0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 5.0, 1.0);
        let result_delta_half = intercept(0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 5.0, 0.5);

        assert!(result_delta_half.x > result_delta_one.x);
    }

    #[test]
    fn intercept_positions_subtracts_source_velocity_when_requested() {
        let with_source = intercept_positions(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(1.0, 0.0),
            true,
            5.0,
            1.0,
        );
        let without_source = intercept_positions(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(1.0, 0.0),
            false,
            5.0,
            1.0,
        );

        assert!(without_source.x > with_source.x);
    }

    #[test]
    fn quad_mirrors_java_linear_case_and_discriminant_behavior() {
        assert_eq!(quad(0.0, 0.0, 0.0), Some(Vec2::new(0.0, 0.0)));
        assert_eq!(quad(0.0, 2.0, -4.0), None);
        assert_eq!(quad(1.0, 0.0, 1.0), None);
        assert_eq!(quad(1.0, -3.0, 2.0), Some(Vec2::new(1.0, 2.0)));
    }
}
