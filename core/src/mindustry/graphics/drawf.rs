use std::f32::consts::PI;

use crate::mindustry::{
    entities::{comp::DecalColor, effect::StandardEffectLightRenderPrimitive},
    graphics::Layer,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeamMode {
    Triangles,
    Lines,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamPlan {
    pub start: (f32, f32),
    pub target: (f32, f32),
    pub radius: f32,
    pub corners: [(f32, f32); 4],
    pub closest_corner: (f32, f32),
    pub mode: BeamMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlamePlan {
    pub center: (f32, f32),
    pub divisions: i32,
    pub rotation: f32,
    pub length: f32,
    pub width: f32,
    pub pan: f32,
    pub front: bool,
    pub points: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightDrawPlan {
    pub center: (f32, f32),
    pub radius: f32,
    pub color: DecalColor,
    pub opacity: f32,
}

impl LightDrawPlan {
    pub fn as_effect_primitive(self, color: &'static str) -> StandardEffectLightRenderPrimitive {
        StandardEffectLightRenderPrimitive {
            center: self.center,
            radius: self.radius,
            color,
            color_rgba: Some(self.color),
            opacity: self.opacity,
        }
    }
}

pub type LightPrimitive = LightDrawPlan;

pub struct Drawf;

impl Drawf {
    pub fn flame(
        x: f32,
        y: f32,
        divisions: i32,
        rotation: f32,
        length: f32,
        width: f32,
        pan: f32,
    ) -> FlamePlan {
        FlamePlan {
            center: (x, y),
            divisions,
            rotation,
            length,
            width,
            pan,
            front: false,
            points: flame_points(x, y, divisions, rotation, length, width, pan, false),
        }
    }

    pub fn flame_front(
        x: f32,
        y: f32,
        divisions: i32,
        rotation: f32,
        length: f32,
        width: f32,
    ) -> FlamePlan {
        let divisions = round_to_even(divisions) + 1;
        FlamePlan {
            center: (x, y),
            divisions,
            rotation,
            length,
            width,
            pan: 1.0,
            front: true,
            points: flame_points(x, y, divisions, rotation, length, width, 1.0, true),
        }
    }

    pub fn build_beam(
        x: f32,
        y: f32,
        tx: f32,
        ty: f32,
        radius: f32,
        animate_shields: bool,
    ) -> BeamPlan {
        let mut corners = [
            (tx - radius, ty - radius),
            (tx + radius, ty - radius),
            (tx - radius, ty + radius),
            (tx + radius, ty + radius),
        ];

        let target_angle = angle_deg(x, y, tx, ty);
        corners.sort_by(|a, b| {
            angle_distance(angle_deg(x, y, a.0, a.1), target_angle)
                .partial_cmp(&angle_distance(angle_deg(x, y, b.0, b.1), target_angle))
                .unwrap()
        });

        let closest_corner = *corners
            .iter()
            .min_by(|a, b| {
                distance_sq(x, y, a.0, a.1)
                    .partial_cmp(&distance_sq(x, y, b.0, b.1))
                    .unwrap()
            })
            .unwrap();

        BeamPlan {
            start: (x, y),
            target: (tx, ty),
            radius,
            corners,
            closest_corner,
            mode: if animate_shields {
                BeamMode::Triangles
            } else {
                BeamMode::Lines
            },
        }
    }

    pub fn light(x: f32, y: f32, radius: f32, color: DecalColor, opacity: f32) -> LightDrawPlan {
        LightDrawPlan {
            center: (x, y),
            radius,
            color,
            opacity,
        }
    }

    pub fn text_layer(pixelate: bool) -> f32 {
        if pixelate {
            Layer::END_PIXELED
        } else {
            0.0
        }
    }
}

fn flame_points(
    x: f32,
    y: f32,
    divisions: i32,
    rotation: f32,
    length: f32,
    width: f32,
    pan: f32,
    front: bool,
) -> Vec<(f32, f32)> {
    let len1 = length * pan;
    let len2 = length * (1.0 - pan);
    let mut points =
        Vec::with_capacity((divisions.max(0) as usize + 1) * if front { 1 } else { 2 });

    let half_arc = |start: f32, end: f32, count: i32, offset: f32, result: &mut Vec<(f32, f32)>| {
        let steps = if front { count + 1 } else { count };
        for i in 0..steps {
            let rot = start + (end - start) * (i as f32 / count as f32);
            let (dx, dy) = trns_exact(rot, width);
            let local_x = if front {
                (dx / width) * length
            } else {
                offset + ((dx + width) / width) * (if start > 0.0 { len1 } else { len2 })
            };
            let local_y = dy;
            result.push(rotate_point(local_x, local_y, x, y, rotation));
        }
    };

    if front {
        half_arc(-90.0, 90.0, divisions.max(1), 0.0, &mut points);
    } else {
        half_arc(90.0, 270.0, divisions.max(1), 0.0, &mut points);
        half_arc(-90.0, 90.0, divisions.max(1), len1, &mut points);
    }

    points
}

fn rotate_point(x: f32, y: f32, base_x: f32, base_y: f32, rotation: f32) -> (f32, f32) {
    let angle = rotation * PI / 180.0;
    let (sin, cos) = angle.sin_cos();
    (base_x + x * cos - y * sin, base_y + x * sin + y * cos)
}

fn trns_exact(rot: f32, length: f32) -> (f32, f32) {
    let angle = rot * PI / 180.0;
    let (sin, cos) = angle.sin_cos();
    (cos * length, sin * length)
}

fn angle_deg(x: f32, y: f32, x2: f32, y2: f32) -> f32 {
    (y2 - y).atan2(x2 - x).to_degrees()
}

fn angle_distance(a: f32, b: f32) -> f32 {
    let mut diff = (a - b).rem_euclid(360.0);
    if diff > 180.0 {
        diff -= 360.0;
    }
    diff.abs()
}

fn distance_sq(x: f32, y: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x;
    let dy = y2 - y;
    dx * dx + dy * dy
}

fn round_to_even(value: i32) -> i32 {
    if value % 2 == 0 {
        value
    } else {
        value + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawf_light_plans_convert_to_effect_primitives() {
        let plan = Drawf::light(10.0, 20.0, 30.0, DecalColor::from_rgba(0x336699cc), 0.5);
        let primitive = plan.as_effect_primitive("Input.color");

        assert_eq!(primitive.center, (10.0, 20.0));
        assert_eq!(primitive.radius, 30.0);
        assert_eq!(primitive.color, "Input.color");
        assert_eq!(
            primitive.color_rgba,
            Some(DecalColor::from_rgba(0x336699cc))
        );
        assert_eq!(primitive.opacity, 0.5);
    }

    #[test]
    fn drawf_beam_plan_distinguishes_triangles_and_lines() {
        let triangles = Drawf::build_beam(0.0, 0.0, 10.0, 0.0, 2.0, true);
        assert_eq!(triangles.mode, BeamMode::Triangles);
        assert_eq!(triangles.start, (0.0, 0.0));
        assert_eq!(triangles.target, (10.0, 0.0));
        assert_eq!(triangles.corners.len(), 4);

        let lines = Drawf::build_beam(0.0, 0.0, 10.0, 0.0, 2.0, false);
        assert_eq!(lines.mode, BeamMode::Lines);
    }

    #[test]
    fn drawf_flame_plans_emit_point_lists() {
        let flame = Drawf::flame(1.0, 2.0, 4, 90.0, 12.0, 3.0, 0.4);
        assert_eq!(flame.center, (1.0, 2.0));
        assert!(!flame.front);
        assert_eq!(flame.points.len(), 8);

        let front = Drawf::flame_front(1.0, 2.0, 3, 90.0, 12.0, 3.0);
        assert!(front.front);
        assert_eq!(front.points.len(), front.divisions as usize + 1);
        assert_eq!(front.divisions, 5);
    }
}
