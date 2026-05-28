//! Two-bone inverse kinematics helper mirroring upstream Mindustry semantics.
//!
//! The Java source exposes two overloads:
//! - one that receives a `side` hint and synthesizes an attractor,
//! - one that receives the attractor directly.
//!
//! This Rust port keeps the same math, while also making the degenerate cases
//! explicit:
//! - unreachable targets clamp to the nearest boundary pose and report `success = false`;
//! - over-close targets do the same instead of producing NaNs;
//! - collapsed targets (`end == 0`) stay finite and use the attractor side when possible.

use crate::mindustry::io::Vec2;

const EPS: f32 = 1.0e-6;
const SIDE_ROTATE_DEG: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolveOutput {
    pub joint: Vec2,
    pub success: bool,
}

impl SolveOutput {
    pub const fn new(joint: Vec2, success: bool) -> Self {
        Self { joint, success }
    }
}

pub struct InverseKinematics;

impl InverseKinematics {
    /// Solves the two-bone chain using a side hint, matching the Java overload
    /// that rotates the endpoint by one degree and uses that as an attractor.
    pub fn solve_with_side(length_a: f32, length_b: f32, end: Vec2, side: bool) -> SolveOutput {
        let end_len = vec_len(end);
        let attractor = if end_len > EPS {
            let rotated = rotate_deg(
                end,
                if side {
                    SIDE_ROTATE_DEG
                } else {
                    -SIDE_ROTATE_DEG
                },
            );
            add(scale_to(rotated, length_a + length_b), scale(end, 0.5))
        } else {
            // When the target collapses to the origin, Java's rotate-based attractor
            // becomes zero as well. We instead seed a deterministic side vector so the
            // math stays finite and the side choice still survives.
            let axis = Vec2::new(1.0, 0.0);
            let side_axis = if side { perp_ccw(axis) } else { perp_cw(axis) };
            scale(side_axis, length_a + length_b)
        };

        Self::solve(length_a, length_b, end, attractor)
    }

    /// Solves the chain using a direct attractor, mirroring the Java overload.
    pub fn solve(length_a: f32, length_b: f32, end: Vec2, attractor: Vec2) -> SolveOutput {
        solve_internal(length_a, length_b, end, attractor)
    }

    /// Java-style helper that writes into an output vector and returns whether
    /// the exact interior solution exists.
    #[allow(dead_code)]
    pub fn solve_with_side_into(
        length_a: f32,
        length_b: f32,
        end: Vec2,
        side: bool,
        result: &mut Vec2,
    ) -> bool {
        let out = Self::solve_with_side(length_a, length_b, end, side);
        *result = out.joint;
        out.success
    }

    /// Java-style helper that writes into an output vector and returns whether
    /// the exact interior solution exists.
    #[allow(dead_code)]
    pub fn solve_into(
        length_a: f32,
        length_b: f32,
        end: Vec2,
        attractor: Vec2,
        result: &mut Vec2,
    ) -> bool {
        let out = Self::solve(length_a, length_b, end, attractor);
        *result = out.joint;
        out.success
    }
}

#[allow(dead_code)]
pub fn solve_with_side(length_a: f32, length_b: f32, end: Vec2, side: bool) -> SolveOutput {
    InverseKinematics::solve_with_side(length_a, length_b, end, side)
}

#[allow(dead_code)]
pub fn solve(length_a: f32, length_b: f32, end: Vec2, attractor: Vec2) -> SolveOutput {
    InverseKinematics::solve(length_a, length_b, end, attractor)
}

fn solve_internal(length_a: f32, length_b: f32, end: Vec2, attractor: Vec2) -> SolveOutput {
    if !length_a.is_finite()
        || !length_b.is_finite()
        || !end.x.is_finite()
        || !end.y.is_finite()
        || !attractor.x.is_finite()
        || !attractor.y.is_finite()
        || length_a < 0.0
        || length_b < 0.0
    {
        return SolveOutput::new(Vec2::new(0.0, 0.0), false);
    }

    let end_len = vec_len(end);
    if end_len <= EPS {
        return solve_collapsed_target(length_a, length_b, attractor);
    }

    let axis = scale(end, 1.0 / end_len);
    let side_axis = side_axis_from_attractor(axis, attractor);

    let numerator = end_len * end_len + length_a * length_a - length_b * length_b;
    let x_raw = numerator / (2.0 * end_len);
    let x = x_raw.clamp(0.0, length_a);
    let y_sq = (length_a * length_a - x * x).max(0.0);
    let y = y_sq.sqrt();
    let joint = add(scale(axis, x), scale(side_axis, y));

    let success = x > EPS && x < length_a - EPS && y > EPS;
    SolveOutput::new(joint, success)
}

fn solve_collapsed_target(length_a: f32, length_b: f32, attractor: Vec2) -> SolveOutput {
    if (length_a - length_b).abs() <= EPS {
        let axis = Vec2::new(1.0, 0.0);
        let side_axis = side_axis_from_attractor(axis, attractor);
        return SolveOutput::new(scale(side_axis, length_a.max(0.0)), true);
    }

    let axis = Vec2::new(1.0, 0.0);
    let side_axis = side_axis_from_attractor(axis, attractor);

    // No exact solution exists when the target is collapsed but the arms are not
    // the same length. Return a deterministic boundary pose that stays finite.
    let joint = if length_a > length_b {
        scale(axis, length_a)
    } else {
        scale(side_axis, length_a)
    };

    SolveOutput::new(joint, false)
}

fn side_axis_from_attractor(axis: Vec2, attractor: Vec2) -> Vec2 {
    let projected = sub(attractor, scale(axis, dot(attractor, axis)));
    let projected_len = vec_len(projected);
    if projected_len > EPS {
        scale(projected, 1.0 / projected_len)
    } else if cross(axis, attractor) < 0.0 {
        perp_cw(axis)
    } else {
        perp_ccw(axis)
    }
}

fn vec_len(vec: Vec2) -> f32 {
    (vec.x * vec.x + vec.y * vec.y).sqrt()
}

fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn scale(vec: Vec2, factor: f32) -> Vec2 {
    Vec2::new(vec.x * factor, vec.y * factor)
}

fn scale_to(vec: Vec2, length: f32) -> Vec2 {
    let current = vec_len(vec);
    if current <= EPS {
        Vec2::new(0.0, 0.0)
    } else {
        scale(vec, length / current)
    }
}

fn add(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x + b.x, a.y + b.y)
}

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x - b.x, a.y - b.y)
}

fn rotate_deg(vec: Vec2, deg: f32) -> Vec2 {
    let radians = deg.to_radians();
    let (sin, cos) = radians.sin_cos();
    Vec2::new(vec.x * cos - vec.y * sin, vec.x * sin + vec.y * cos)
}

fn perp_ccw(vec: Vec2) -> Vec2 {
    Vec2::new(-vec.y, vec.x)
}

fn perp_cw(vec: Vec2) -> Vec2 {
    Vec2::new(vec.y, -vec.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_EPS: f32 = 1.0e-5;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= TEST_EPS,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn solve_with_side_matches_expected_two_bone_coordinates_on_both_sides() {
        let right = InverseKinematics::solve_with_side(4.0, 3.0, Vec2::new(5.0, 0.0), true);
        assert!(right.success);
        assert_close(right.joint.x, 3.2);
        assert_close(right.joint.y, 2.4);

        let left = InverseKinematics::solve_with_side(4.0, 3.0, Vec2::new(5.0, 0.0), false);
        assert!(left.success);
        assert_close(left.joint.x, 3.2);
        assert_close(left.joint.y, -2.4);
    }

    #[test]
    fn solve_with_attractor_prefers_the_requested_half_plane() {
        let out = InverseKinematics::solve(4.0, 3.0, Vec2::new(5.0, 0.0), Vec2::new(5.0, 5.0));
        assert!(out.success);
        assert_close(out.joint.x, 3.2);
        assert!(out.joint.y > 0.0);
        assert_close(out.joint.y, 2.4);
    }

    #[test]
    fn solve_clamps_unreachable_far_targets_and_reports_failure() {
        let out = InverseKinematics::solve(4.0, 3.0, Vec2::new(20.0, 0.0), Vec2::new(20.0, 7.0));
        assert!(!out.success);
        assert_close(out.joint.x, 4.0);
        assert_close(out.joint.y, 0.0);
    }

    #[test]
    fn solve_handles_collapsed_targets_without_nan() {
        let equal = InverseKinematics::solve_with_side(3.0, 3.0, Vec2::new(0.0, 0.0), false);
        assert!(equal.success);
        assert_close(equal.joint.x, 0.0);
        assert_close(equal.joint.y, -3.0);

        let too_close = InverseKinematics::solve_with_side(5.0, 2.0, Vec2::new(0.0, 0.0), true);
        assert!(!too_close.success);
        assert!(too_close.joint.x.is_finite());
        assert!(too_close.joint.y.is_finite());
        assert_close(too_close.joint.x, 5.0);
        assert_close(too_close.joint.y, 0.0);
    }
}
