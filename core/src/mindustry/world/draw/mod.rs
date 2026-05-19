#[derive(Debug, Clone, PartialEq)]
pub struct DrawBlockSpec {
    pub icon_override: Option<Vec<String>>,
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
}
