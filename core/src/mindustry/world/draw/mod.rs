#[derive(Debug, Clone, PartialEq)]
pub struct DrawBlockSpec {
    pub icon_override: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawLiquidTilePadding {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawPulseShapeParams {
    pub f: f32,
    pub radius: f32,
    pub stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawPulseDiamondPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawCircleParams {
    pub life: f32,
    pub stroke: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawSpikeLayerParams {
    pub rotation: f32,
    pub speed: f32,
}

pub fn draw_block_final_icons(
    block_name: &str,
    icon_override: Option<&[String]>,
    icons: &[String],
) -> Vec<String> {
    if let Some(overrides) = icon_override {
        overrides
            .iter()
            .map(|suffix| format!("{block_name}{suffix}"))
            .collect()
    } else if icons.is_empty() {
        vec!["error".to_string()]
    } else {
        icons.to_vec()
    }
}

pub fn draw_region_name(block_name: &str, suffix: &str, name_override: Option<&str>) -> String {
    name_override
        .map(str::to_string)
        .unwrap_or_else(|| format!("{block_name}{suffix}"))
}

pub fn draw_region_rotation(
    total_progress: f32,
    rotate_speed: f32,
    rotation: f32,
    building_rotate: bool,
    build_rotation_degrees: f32,
) -> f32 {
    total_progress * rotate_speed
        + rotation
        + if building_rotate {
            build_rotation_degrees
        } else {
            0.0
        }
}

pub fn draw_region_plan_rotation(building_rotate: bool, plan_rotation: i32, rotation: f32) -> f32 {
    if building_rotate {
        plan_rotation as f32 * 90.0
    } else {
        0.0 + rotation
    }
}

pub fn draw_liquid_region_fraction(stored: f32, liquid_capacity: f32, alpha: f32) -> f32 {
    stored / liquid_capacity * alpha
}

pub fn draw_heat_alpha(
    heat: f32,
    heat_requirement: f32,
    color_alpha: f32,
    pulse: f32,
    absin: f32,
) -> f32 {
    (heat / heat_requirement).clamp(0.0, 1.0) * (color_alpha * (1.0 - pulse + absin))
}

pub fn draw_fade_alpha(total_progress_absin: f32, warmup: f32) -> f32 {
    total_progress_absin * warmup
}

pub fn draw_warmup_region_alpha(warmup: f32, sin_mag: f32, absin: f32) -> f32 {
    warmup * (1.0 - sin_mag) + absin * warmup
}

pub fn draw_frames_index(
    total_progress: f32,
    interval: f32,
    frames: i32,
    sine: bool,
    absin_value: f32,
) -> usize {
    if frames <= 0 {
        return 0;
    }
    if sine {
        absin_value as usize
    } else {
        ((total_progress / interval) as i32).rem_euclid(frames) as usize
    }
}

pub fn draw_frames_region_names(block_name: &str, frames: i32) -> Vec<String> {
    (0..frames)
        .map(|index| format!("{block_name}-frame{index}"))
        .collect()
}

pub fn draw_glow_alpha(absin: f32, glow_intensity: f32, warmup: f32, alpha: f32) -> f32 {
    (absin * glow_intensity + 1.0 - glow_intensity) * warmup * alpha
}

pub fn draw_glow_rotation(
    total_progress: f32,
    rotate_speed: f32,
    rotate: bool,
    rotdeg: f32,
) -> f32 {
    total_progress * rotate_speed + if rotate { rotdeg } else { 0.0 }
}

pub fn draw_heat_input_side_alphas(
    side_heat: [f32; 4],
    heat_requirement: f32,
    color_alpha: f32,
    heat_pulse: f32,
    absin: f32,
) -> [f32; 4] {
    side_heat.map(|heat| {
        if heat > 0.0 {
            heat / heat_requirement * (color_alpha * (1.0 - heat_pulse + absin))
        } else {
            0.0
        }
    })
}

pub fn draw_heat_output_rotation(rotation: i32, rot_offset: i32) -> f32 {
    (rotation + rot_offset) as f32 * 90.0
}

pub fn draw_heat_output_top_index(rotation: i32, rot_offset: i32) -> usize {
    if (rotation + rot_offset).rem_euclid(4) > 1 {
        2
    } else {
        1
    }
}

pub fn draw_heat_output_alpha(
    heat_frac: f32,
    color_alpha: f32,
    heat_pulse: f32,
    absin: f32,
) -> f32 {
    heat_frac * (color_alpha * (1.0 - heat_pulse + absin))
}

pub fn draw_multi_icons(drawer_icons: &[Vec<String>]) -> Vec<String> {
    drawer_icons.iter().flatten().cloned().collect()
}

pub fn draw_pump_liquid_fraction(liquid_amount: f32, liquid_capacity: f32) -> f32 {
    liquid_amount / liquid_capacity
}

pub fn draw_blur_spin_rotation(total_progress: f32, rotate_speed: f32, rotation: f32) -> f32 {
    total_progress * rotate_speed + rotation
}

pub fn draw_default_icons(block_region: &str) -> Vec<String> {
    vec![block_region.to_string()]
}

pub fn draw_side_region_names(block_name: &str) -> (String, String) {
    (format!("{block_name}-top1"), format!("{block_name}-top2"))
}

pub fn draw_side_region_index(rotation: i32) -> usize {
    if rotation > 1 {
        1
    } else {
        0
    }
}

pub fn draw_side_region_plan_rotation(plan_rotation: i32) -> f32 {
    plan_rotation as f32 * 90.0
}

pub fn draw_liquid_tile_padding(
    padding: f32,
    pad_left: f32,
    pad_right: f32,
    pad_top: f32,
    pad_bottom: f32,
) -> DrawLiquidTilePadding {
    DrawLiquidTilePadding {
        left: if pad_left < 0.0 { padding } else { pad_left },
        right: if pad_right < 0.0 { padding } else { pad_right },
        top: if pad_top < 0.0 { padding } else { pad_top },
        bottom: if pad_bottom < 0.0 {
            padding
        } else {
            pad_bottom
        },
    }
}

pub fn draw_liquid_tile_selected<'a>(
    draw_liquid: Option<&'a str>,
    current_liquid: &'a str,
) -> &'a str {
    draw_liquid.unwrap_or(current_liquid)
}

pub fn draw_liquid_tile_fraction(liquid_amount: f32, liquid_capacity: f32, alpha: f32) -> f32 {
    liquid_amount / liquid_capacity * alpha
}

pub fn draw_block_parts_preview_name(block_name: &str) -> String {
    format!("{block_name}-preview")
}

pub fn draw_block_parts_plan_rotation(block_rotate: bool, plan_rotation: i32) -> f32 {
    if block_rotate {
        plan_rotation as f32 * 90.0 - 90.0
    } else {
        0.0
    }
}

pub fn draw_block_parts_params(
    warmup: f32,
    progress: f32,
    x: f32,
    y: f32,
    rotation_degrees: f32,
) -> (f32, f32, f32, f32, f32, f32, f32, f32, f32) {
    (
        warmup,
        1.0 - progress,
        1.0 - progress,
        0.0,
        0.0,
        0.0,
        x,
        y,
        rotation_degrees,
    )
}

pub fn draw_shape_radius(radius: f32, warmup: f32, use_warmup_radius: bool) -> f32 {
    if use_warmup_radius {
        radius * warmup
    } else {
        radius
    }
}

pub fn draw_shape_rotation(total_progress: f32, time_scl: f32) -> f32 {
    total_progress * time_scl
}

pub fn draw_pulse_shape_params(
    time: f32,
    time_scl: f32,
    block_size: i32,
    tilesize: f32,
    radius_scl: f32,
    stroke: f32,
    min_stroke: f32,
    warmup: f32,
) -> DrawPulseShapeParams {
    let f = 1.0 - (time / time_scl) % 1.0;
    let radius = block_size as f32 * tilesize / 2.0 * radius_scl;
    DrawPulseShapeParams {
        f,
        radius,
        stroke: (stroke * f + min_stroke) * warmup,
    }
}

pub fn draw_pulse_shape_square_radius(f: f32, radius: f32) -> f32 {
    (1.0 + (1.0 - f) * radius).min(radius)
}

pub fn draw_pulse_shape_diamond_points(f: f32, radius: f32) -> Vec<DrawPulseDiamondPoint> {
    let r = (clamp01(2.0 - f * 2.0) * radius - f - 0.2).max(0.0);
    let w = clamp01(0.5 - f) * radius * 2.0;
    let directions = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
    let mut points = Vec::with_capacity(if f < 0.5 { 8 } else { 4 });
    for (dx, dy) in directions {
        points.push(DrawPulseDiamondPoint {
            x: dx * r + dy * w,
            y: dy * r - dx * w,
        });
        if f < 0.5 {
            points.push(DrawPulseDiamondPoint {
                x: dx * r - dy * w,
                y: dy * r + dx * w,
            });
        }
    }
    points
}

pub fn draw_circles_params(
    time: f32,
    amount: i32,
    warmup: f32,
    stroke_max: f32,
    stroke_min: f32,
    time_scl: f32,
    radius: f32,
    radius_offset: f32,
) -> Vec<DrawCircleParams> {
    if amount <= 0 {
        return Vec::new();
    }
    (0..amount)
        .map(|index| {
            let life = (time / time_scl + index as f32 / amount as f32) % 1.0;
            DrawCircleParams {
                life,
                stroke: warmup * pow3_in_lerp(stroke_max, stroke_min, life),
                radius: radius_offset + life * radius,
            }
        })
        .collect()
}

pub fn draw_spikes_layers(
    warmup: f32,
    layers: i32,
    total_progress: f32,
    rotate_speed: f32,
    layer_speed: f32,
) -> Vec<DrawSpikeLayerParams> {
    if warmup <= 0.001 || layers <= 0 {
        return Vec::new();
    }
    let mut cur_speed = 1.0;
    let mut out = Vec::with_capacity(layers as usize);
    for _ in 0..layers {
        out.push(DrawSpikeLayerParams {
            rotation: total_progress * rotate_speed * cur_speed,
            speed: cur_speed,
        });
        cur_speed *= layer_speed;
    }
    out
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn pow3_in_lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t.powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_block_region_liquid_and_heat_helpers_follow_upstream() {
        assert_eq!(
            draw_block_final_icons("router", Some(&["-a".into(), "-b".into()]), &[]),
            vec!["router-a", "router-b"]
        );
        assert_eq!(draw_block_final_icons("x", None, &[]), vec!["error"]);
        assert_eq!(draw_region_name("kiln", "-top", None), "kiln-top");
        assert_eq!(draw_region_name("kiln", "-top", Some("custom")), "custom");
        assert_eq!(draw_region_rotation(10.0, 2.0, 5.0, true, 90.0), 115.0);
        assert_eq!(draw_region_plan_rotation(true, 2, 15.0), 180.0);
        assert_eq!(draw_region_plan_rotation(false, 2, 15.0), 15.0);
        assert_eq!(draw_liquid_region_fraction(5.0, 20.0, 0.8), 0.2);
        assert!((draw_heat_alpha(5.0, 10.0, 0.8, 0.3, 0.1) - 0.32).abs() < 0.00001);
    }

    #[test]
    fn draw_animation_helpers_follow_upstream_formulae() {
        assert_eq!(draw_fade_alpha(0.4, 0.5), 0.2);
        assert!((draw_warmup_region_alpha(0.5, 0.6, 0.2) - 0.3).abs() < 0.00001);
        assert_eq!(draw_frames_index(16.0, 5.0, 3, false, 0.0), 0);
        assert_eq!(draw_frames_index(16.0, 5.0, 3, true, 1.9), 1);
        assert_eq!(
            draw_frames_region_names("press", 3),
            vec!["press-frame0", "press-frame1", "press-frame2"]
        );
        assert_eq!(draw_glow_alpha(0.5, 0.5, 0.8, 0.9), 0.54);
        assert_eq!(draw_glow_rotation(10.0, 2.0, true, 90.0), 110.0);
        assert_eq!(draw_blur_spin_rotation(3.0, 10.0, 15.0), 45.0);
    }

    #[test]
    fn draw_heat_output_multi_and_pump_helpers_follow_upstream() {
        let alphas = draw_heat_input_side_alphas([1.0, 0.0, 2.0, 4.0], 4.0, 0.8, 0.3, 0.1);
        assert!((alphas[0] - 0.16).abs() < 0.00001);
        assert_eq!(alphas[1], 0.0);
        assert!((alphas[2] - 0.32).abs() < 0.00001);
        assert!((alphas[3] - 0.64).abs() < 0.00001);
        assert_eq!(draw_heat_output_rotation(1, 2), 270.0);
        assert_eq!(draw_heat_output_top_index(0, 0), 1);
        assert_eq!(draw_heat_output_top_index(2, 0), 2);
        assert!((draw_heat_output_alpha(0.5, 0.8, 0.3, 0.1) - 0.32).abs() < 0.00001);
        assert_eq!(
            draw_multi_icons(&[vec!["a".into(), "b".into()], vec!["c".into()]]),
            vec!["a", "b", "c"]
        );
        assert_eq!(draw_pump_liquid_fraction(4.0, 16.0), 0.25);
    }

    #[test]
    fn draw_default_side_liquid_tile_and_parts_follow_upstream_shells() {
        assert_eq!(draw_default_icons("router"), vec!["router"]);
        assert_eq!(
            draw_side_region_names("separator"),
            ("separator-top1".into(), "separator-top2".into())
        );
        assert_eq!(draw_side_region_index(0), 0);
        assert_eq!(draw_side_region_index(1), 0);
        assert_eq!(draw_side_region_index(2), 1);
        assert_eq!(draw_side_region_plan_rotation(3), 270.0);

        assert_eq!(
            draw_liquid_tile_padding(2.0, -1.0, 3.0, -0.5, 4.0),
            DrawLiquidTilePadding {
                left: 2.0,
                right: 3.0,
                top: 2.0,
                bottom: 4.0
            }
        );
        assert_eq!(draw_liquid_tile_selected(Some("slag"), "water"), "slag");
        assert_eq!(draw_liquid_tile_selected(None, "water"), "water");
        assert_eq!(draw_liquid_tile_fraction(5.0, 20.0, 0.5), 0.125);

        assert_eq!(draw_block_parts_preview_name("melter"), "melter-preview");
        assert_eq!(draw_block_parts_plan_rotation(true, 2), 90.0);
        assert_eq!(draw_block_parts_plan_rotation(false, 2), 0.0);
        assert_eq!(
            draw_block_parts_params(0.75, 0.2, 10.0, 20.0, 180.0),
            (0.75, 0.8, 0.8, 0.0, 0.0, 0.0, 10.0, 20.0, 180.0)
        );
    }

    #[test]
    fn draw_shape_pulse_circles_and_spikes_follow_upstream_formulae() {
        assert_eq!(draw_shape_radius(4.0, 0.25, false), 4.0);
        assert_eq!(draw_shape_radius(4.0, 0.25, true), 1.0);
        assert_eq!(draw_shape_rotation(12.0, 1.5), 18.0);

        let pulse = draw_pulse_shape_params(25.0, 100.0, 3, 8.0, 1.25, 2.0, 0.2, 0.5);
        assert!((pulse.f - 0.75).abs() < 0.00001);
        assert!((pulse.radius - 15.0).abs() < 0.00001);
        assert!((pulse.stroke - 0.85).abs() < 0.00001);
        assert_eq!(draw_pulse_shape_square_radius(0.75, 15.0), 4.75);
        let diamond = draw_pulse_shape_diamond_points(0.25, 10.0);
        assert_eq!(diamond.len(), 8);
        assert_eq!(diamond[0], DrawPulseDiamondPoint { x: 9.55, y: -5.0 });
        assert_eq!(diamond[1], DrawPulseDiamondPoint { x: 9.55, y: 5.0 });

        let circles = draw_circles_params(80.0, 4, 0.5, 2.0, 0.2, 160.0, 12.0, 1.0);
        assert_eq!(circles.len(), 4);
        assert!((circles[0].life - 0.5).abs() < 0.00001);
        assert!((circles[0].stroke - 0.8875).abs() < 0.00001);
        assert_eq!(circles[0].radius, 7.0);
        assert!((circles[2].life - 0.0).abs() < 0.00001);
        assert_eq!(circles[2].stroke, 1.0);
        assert_eq!(circles[2].radius, 1.0);

        assert!(draw_spikes_layers(0.001, 3, 10.0, 0.8, -1.0).is_empty());
        let spikes = draw_spikes_layers(0.5, 3, 10.0, 0.8, -1.0);
        assert_eq!(
            spikes,
            vec![
                DrawSpikeLayerParams {
                    rotation: 8.0,
                    speed: 1.0
                },
                DrawSpikeLayerParams {
                    rotation: -8.0,
                    speed: -1.0
                },
                DrawSpikeLayerParams {
                    rotation: 8.0,
                    speed: 1.0
                }
            ]
        );
    }
}
