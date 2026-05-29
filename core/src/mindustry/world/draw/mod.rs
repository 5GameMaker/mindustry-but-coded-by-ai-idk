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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawBubbleParams {
    pub life: f32,
    pub radius: f32,
    pub stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawCellParams {
    pub fin: f32,
    pub fslope: f32,
    pub radius: f32,
    pub color_t: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawParticleParams {
    pub fin: f32,
    pub fout: f32,
    pub alpha: f32,
    pub angle: f32,
    pub length: f32,
    pub size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawCrucibleFlameParams {
    pub alpha: f32,
    pub middle_radius: f32,
    pub circle_radius: f32,
    pub stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawFlameParams {
    pub top_alpha: f32,
    pub flame_alpha: f32,
    pub outer_radius: f32,
    pub inner_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawPlasmaLayerParams {
    pub radius: f32,
    pub alpha: f32,
    pub rotation: f32,
    pub color_t: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawLiquidOutputParams {
    pub output_index: usize,
    pub side_variant: usize,
    pub rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawArcSmeltParams {
    pub alpha: f32,
    pub middle_radius: f32,
    pub circle_radius: f32,
    pub circle_stroke: f32,
    pub particle_stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawArcSmeltParticleParams {
    pub fin: f32,
    pub fout: f32,
    pub angle: f32,
    pub length: f32,
    pub line_length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawPistonParams {
    pub region_variant: usize,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub flip_y: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawWeaveParams {
    pub rotation: f32,
    pub line_x: f32,
    pub line_length: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawMultiWeaveParams {
    pub first_rotation: f32,
    pub second_rotation: f32,
    pub weave_alpha: f32,
    pub glow_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawSoftParticleParams {
    pub fin: f32,
    pub fout: f32,
    pub alpha: f32,
    pub angle: f32,
    pub length: f32,
    pub size: f32,
    pub color_t: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawBlockParticleRenderKind {
    Circle,
    Polygon,
    SoftSprite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawBlockParticleSizeInterp {
    Slope,
    One,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawBlockParticleBlendMode {
    Normal,
    Additive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawBlockParticleConfig {
    pub x: f32,
    pub y: f32,
    pub color_rgba: u32,
    pub secondary_color_rgba: Option<u32>,
    pub alpha: f32,
    pub sides: usize,
    pub particle_count: usize,
    pub particle_rotation: f32,
    pub particle_life: f32,
    pub particle_radius: f32,
    pub particle_size: f32,
    pub fade_margin: f32,
    pub rotate_scl: f32,
    pub reverse: bool,
    pub random_life_range: f32,
    pub invert_life: bool,
    pub size_interp: DrawBlockParticleSizeInterp,
    pub blend_mode: DrawBlockParticleBlendMode,
    pub render_kind: DrawBlockParticleRenderKind,
    pub region: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawTurretRegionNames {
    pub preview: String,
    pub outline: String,
    pub liquid: String,
    pub top: String,
    pub heat: String,
    pub base: String,
    pub mod_base_fallback: Option<String>,
    pub vanilla_base_fallback: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawTurretParams {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawTurretShadowParams {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawBlurSpinParams<'a> {
    pub region: &'a str,
    pub rotation: f32,
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

pub fn draw_region_icons(region: &str) -> Vec<String> {
    vec![region.to_string()]
}

pub fn draw_liquid_region_fraction(stored: f32, liquid_capacity: f32, alpha: f32) -> f32 {
    stored / liquid_capacity * alpha
}

pub fn draw_liquid_region_name(block_name: &str, suffix: &str) -> String {
    format!("{block_name}{suffix}")
}

pub fn draw_liquid_region_requires_liquids(has_liquids: bool) -> Result<(), &'static str> {
    if has_liquids {
        Ok(())
    } else {
        Err("DrawLiquidRegion requires block.hasLiquids")
    }
}

pub fn draw_liquid_region_selected<'a>(
    draw_liquid: Option<&'a str>,
    current_liquid: &'a str,
) -> &'a str {
    draw_liquid.unwrap_or(current_liquid)
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

pub fn draw_heat_region_name(block_name: &str, suffix: &str) -> String {
    format!("{block_name}{suffix}")
}

pub fn draw_fade_alpha(total_progress_absin: f32, warmup: f32) -> f32 {
    total_progress_absin * warmup
}

pub fn draw_fade_region_name(block_name: &str, suffix: &str) -> String {
    format!("{block_name}{suffix}")
}

pub fn draw_warmup_region_alpha(warmup: f32, sin_mag: f32, absin: f32) -> f32 {
    warmup * (1.0 - sin_mag) + absin * warmup
}

pub fn draw_warmup_region_name(block_name: &str) -> String {
    format!("{block_name}-top")
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

pub fn draw_frames_icons(region_names: &[String]) -> Vec<String> {
    region_names
        .first()
        .cloned()
        .into_iter()
        .collect::<Vec<_>>()
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

pub fn draw_glow_region_name(block_name: &str, suffix: &str) -> String {
    format!("{block_name}{suffix}")
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

pub fn draw_heat_input_region_name(block_name: &str, suffix: &str) -> String {
    format!("{block_name}{suffix}")
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

pub fn draw_heat_output_region_names(block_name: &str) -> [String; 4] {
    [
        format!("{block_name}-heat"),
        format!("{block_name}-glow"),
        format!("{block_name}-top1"),
        format!("{block_name}-top2"),
    ]
}

pub fn draw_heat_output_static_icon(block_name: &str, rot_offset: i32) -> String {
    let names = draw_heat_output_region_names(block_name);
    if draw_heat_output_top_index(0, rot_offset) == 2 {
        names[3].clone()
    } else {
        names[2].clone()
    }
}

pub fn draw_multi_icons(drawer_icons: &[Vec<String>]) -> Vec<String> {
    drawer_icons.iter().flatten().cloned().collect()
}

pub fn draw_pump_liquid_fraction(liquid_amount: f32, liquid_capacity: f32) -> f32 {
    liquid_amount / liquid_capacity
}

pub fn draw_pump_liquid_region_name(block_name: &str) -> String {
    format!("{block_name}-liquid")
}

pub fn draw_pump_liquid_should_draw(is_pump_build: bool, liquid_drop_present: bool) -> bool {
    is_pump_build && liquid_drop_present
}

pub fn draw_blur_spin_region_names(block_name: &str, suffix: &str) -> (String, String) {
    (
        format!("{block_name}{suffix}"),
        format!("{block_name}{suffix}-blur"),
    )
}

pub fn draw_blur_spin_rotation(total_progress: f32, rotate_speed: f32, rotation: f32) -> f32 {
    total_progress * rotate_speed + rotation
}

pub fn draw_blur_spin_params<'a>(
    warmup: f32,
    blur_thresh: f32,
    region: &'a str,
    blur_region: &'a str,
    total_progress: f32,
    rotate_speed: f32,
) -> DrawBlurSpinParams<'a> {
    DrawBlurSpinParams {
        region: if warmup > blur_thresh {
            blur_region
        } else {
            region
        },
        rotation: total_progress * rotate_speed,
    }
}

pub fn draw_default_icons(block_region: &str) -> Vec<String> {
    vec![block_region.to_string()]
}

fn drawer_arg_is_numeric_or_bool(arg: &str) -> bool {
    let arg = arg.trim();
    if arg.eq_ignore_ascii_case("true") || arg.eq_ignore_ascii_case("false") {
        return true;
    }

    let stripped = arg.trim_end_matches(['f', 'F']);
    !stripped.is_empty() && stripped.parse::<f32>().is_ok()
}

fn drawer_suffix_or_default(args: &str, default_suffix: &str) -> String {
    let suffix = split_drawer_args(args)
        .first()
        .copied()
        .unwrap_or("")
        .trim();

    if suffix.is_empty() || drawer_arg_is_numeric_or_bool(suffix) {
        default_suffix.to_string()
    } else {
        suffix.to_string()
    }
}

pub fn draw_block_dispatch_icons(block_name: &str, drawer: &str) -> Vec<String> {
    let drawer = drawer.trim();
    if drawer.is_empty() {
        return Vec::new();
    }

    match split_drawer_call(drawer) {
        Some(("DrawMulti", args)) => draw_multi_icons(
            &split_drawer_args(args)
                .into_iter()
                .map(|child| draw_block_dispatch_icons(block_name, child))
                .collect::<Vec<_>>(),
        ),
        Some(("DrawDefault", _)) => draw_default_icons(block_name),
        Some(("DrawRegion", args)) => {
            let suffix = split_drawer_args(args)
                .first()
                .copied()
                .unwrap_or("")
                .trim();
            vec![draw_region_name(block_name, suffix, None)]
        }
        Some(("DrawPistons", args)) => {
            let suffix = split_drawer_args(args)
                .first()
                .copied()
                .unwrap_or("-piston")
                .trim();
            let suffix = if suffix.is_empty() { "-piston" } else { suffix };
            vec![draw_piston_region_names(block_name, suffix)[3].clone()]
        }
        Some(("DrawHeatInput", args)) => {
            let suffix = drawer_suffix_or_default(args, "-heat");
            vec![draw_heat_input_region_name(block_name, &suffix)]
        }
        Some(("DrawFlame", _)) => vec![draw_flame_top_name(block_name)],
        Some(("DrawHeatRegion", args)) => {
            let suffix = drawer_suffix_or_default(args, "-glow");
            vec![draw_heat_region_name(block_name, &suffix)]
        }
        Some(("DrawGlowRegion", args)) => {
            let suffix = drawer_suffix_or_default(args, "-glow");
            vec![draw_glow_region_name(block_name, &suffix)]
        }
        Some(("DrawLiquidOutputs", _)) => Vec::new(),
        Some(("DrawLiquidRegion", _)) => vec![draw_liquid_region_name(block_name, "-liquid")],
        Some(("DrawWarmupRegion", _)) => vec![draw_warmup_region_name(block_name)],
        Some(("DrawLiquidTile", _)) => Vec::new(),
        Some(("DrawParticles", _)) => Vec::new(),
        Some(("DrawWeave", _)) => vec![draw_weave_name(block_name)],
        Some(("DrawMultiWeave", _)) => {
            let (weave, _) = draw_multi_weave_region_names(block_name);
            draw_multi_weave_icons(false, &weave)
        }
        Some(("DrawSideRegion", _)) => {
            let (top1, _) = draw_side_region_names(block_name);
            vec![top1]
        }
        Some(("DrawTurret", _)) => {
            let names = draw_turret_region_names(block_name, 0, "", None);
            draw_turret_icons(&names.base, &names.preview, Some(&names.top))
        }
        Some(("DrawPower", args)) => {
            let suffix = split_drawer_args(args)
                .first()
                .copied()
                .unwrap_or("-power")
                .trim();
            let suffix = if suffix.is_empty() { "-power" } else { suffix };
            draw_power_region_names(block_name, suffix, true)
        }
        Some(("DrawHeatOutput", args)) => {
            let rot_offset = split_drawer_args(args)
                .first()
                .and_then(|arg| arg.trim().parse::<i32>().ok())
                .unwrap_or(0);
            vec![draw_heat_output_static_icon(block_name, rot_offset)]
        }
        Some(_) => Vec::new(),
        None => match drawer {
            "DrawDefault" => draw_default_icons(block_name),
            "DrawRegion" => draw_region_icons(block_name),
            "DrawPistons" => vec![draw_piston_region_names(block_name, "-piston")[3].clone()],
            "DrawHeatInput" => vec![draw_heat_input_region_name(block_name, "-heat")],
            "DrawFlame" => vec![draw_flame_top_name(block_name)],
            "DrawHeatRegion" => vec![draw_heat_region_name(block_name, "-glow")],
            "DrawGlowRegion" => vec![draw_glow_region_name(block_name, "-glow")],
            // These shells depend on runtime block liquid/particle state and do not have a
            // stable atlas symbol to bridge into sprite ops yet.
            "DrawLiquidOutputs" => Vec::new(),
            "DrawLiquidRegion" => vec![draw_liquid_region_name(block_name, "-liquid")],
            "DrawWarmupRegion" => vec![draw_warmup_region_name(block_name)],
            "DrawLiquidTile" => Vec::new(),
            "DrawParticles" => Vec::new(),
            "DrawWeave" => vec![draw_weave_name(block_name)],
            "DrawMultiWeave" => {
                let (weave, _) = draw_multi_weave_region_names(block_name);
                draw_multi_weave_icons(false, &weave)
            }
            "DrawSideRegion" => {
                let (top1, _) = draw_side_region_names(block_name);
                vec![top1]
            }
            "DrawTurret" => {
                let names = draw_turret_region_names(block_name, 0, "", None);
                draw_turret_icons(&names.base, &names.preview, Some(&names.top))
            }
            "DrawPower" => draw_power_region_names(block_name, "-power", true),
            "DrawHeatOutput" => vec![draw_heat_output_static_icon(block_name, 0)],
            _ => Vec::new(),
        },
    }
}

pub fn draw_particles_block_config() -> DrawBlockParticleConfig {
    DrawBlockParticleConfig {
        x: 0.0,
        y: 0.0,
        color_rgba: 0xf2d585ff,
        secondary_color_rgba: None,
        alpha: 0.5,
        sides: 12,
        particle_count: 30,
        particle_rotation: 0.0,
        particle_life: 70.0,
        particle_radius: 7.0,
        particle_size: 3.0,
        fade_margin: 0.4,
        rotate_scl: 3.0,
        reverse: false,
        random_life_range: 2.0,
        invert_life: false,
        size_interp: DrawBlockParticleSizeInterp::Slope,
        blend_mode: DrawBlockParticleBlendMode::Normal,
        render_kind: DrawBlockParticleRenderKind::Circle,
        region: None,
    }
}

pub fn draw_soft_particles_block_config() -> DrawBlockParticleConfig {
    DrawBlockParticleConfig {
        x: 0.0,
        y: 0.0,
        color_rgba: 0xe3ae6fff,
        secondary_color_rgba: Some(0xd04d46ff),
        alpha: 0.5,
        sides: 0,
        particle_count: 30,
        particle_rotation: 0.0,
        particle_life: 70.0,
        particle_radius: 7.0,
        particle_size: 3.0,
        fade_margin: 0.4,
        rotate_scl: 1.5,
        reverse: false,
        random_life_range: 1.0,
        invert_life: true,
        size_interp: DrawBlockParticleSizeInterp::Linear,
        blend_mode: DrawBlockParticleBlendMode::Additive,
        render_kind: DrawBlockParticleRenderKind::SoftSprite,
        region: Some(draw_soft_particle_region_name()),
    }
}

pub fn draw_particles_block_config_for(block_name: &str) -> DrawBlockParticleConfig {
    let mut config = draw_particles_block_config();
    match block_name {
        "atmospheric-concentrator" => {
            config.color_rgba = 0xd4f0ffff;
            config.alpha = 0.6;
            config.particle_size = 4.0;
            config.particle_count = 10;
            config.particle_radius = 12.0;
            config.particle_life = 140.0;
        }
        "cyanogen-synthesizer" => {
            config.color_rgba = 0x89e8b6ff;
            config.alpha = 0.5;
            config.particle_size = 3.0;
            config.particle_count = 10;
            config.particle_radius = 9.0;
            config.particle_life = 200.0;
            config.reverse = true;
            config.size_interp = DrawBlockParticleSizeInterp::One;
        }
        _ => {}
    }
    config
}

pub fn draw_soft_particles_block_config_for(block_name: &str) -> DrawBlockParticleConfig {
    let mut config = draw_soft_particles_block_config();
    if block_name == "flux-reactor" {
        config.alpha = 0.35;
        config.particle_radius = 12.0;
        config.particle_size = 9.0;
        config.particle_life = 120.0;
        config.particle_count = 27;
    }
    config
}

pub fn draw_block_dispatch_particle_configs(
    block_name: &str,
    drawer: &str,
) -> Vec<DrawBlockParticleConfig> {
    let drawer = drawer.trim();
    if drawer.is_empty() {
        return Vec::new();
    }

    match split_drawer_call(drawer) {
        Some(("DrawMulti", args)) => split_drawer_args(args)
            .into_iter()
            .flat_map(|child| draw_block_dispatch_particle_configs(block_name, child))
            .collect(),
        Some(("DrawParticles", _)) => vec![draw_particles_block_config_for(block_name)],
        Some(("DrawSoftParticles", _)) => vec![draw_soft_particles_block_config_for(block_name)],
        Some(_) => Vec::new(),
        None => match drawer {
            "DrawParticles" => vec![draw_particles_block_config_for(block_name)],
            "DrawSoftParticles" => vec![draw_soft_particles_block_config_for(block_name)],
            _ => Vec::new(),
        },
    }
}

pub fn draw_block_drawer_icons(block_name: &str, drawer: &str) -> Vec<String> {
    draw_block_dispatch_icons(block_name, drawer)
}

pub fn draw_block_drawer_particle_configs(
    block_name: &str,
    drawer: &str,
) -> Vec<DrawBlockParticleConfig> {
    draw_block_dispatch_particle_configs(block_name, drawer)
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

pub fn draw_bubble_params(
    warmup: f32,
    time: f32,
    time_scl: f32,
    recurrence: f32,
    random_offset: f32,
    radius: f32,
    stroke_min: f32,
) -> Option<DrawBubbleParams> {
    if warmup <= 0.001 {
        return None;
    }
    let life = 1.0 - ((time / time_scl + random_offset) % recurrence);
    if life <= 0.0 {
        return None;
    }
    Some(DrawBubbleParams {
        life,
        radius: (1.0 - life) * radius,
        stroke: warmup * (life + stroke_min),
    })
}

pub fn draw_cells_middle_name(block_name: &str) -> String {
    format!("{block_name}-middle")
}

pub fn draw_cell_params(
    warmup: f32,
    time: f32,
    offset: f32,
    lifetime: f32,
    recurrence: f32,
    radius: f32,
    color_t: f32,
) -> Option<DrawCellParams> {
    if warmup <= 0.001 {
        return None;
    }
    let fin = 1.0 - (((time + offset) / lifetime) % recurrence);
    if fin <= 0.0 {
        return None;
    }
    let fslope = slope(fin);
    Some(DrawCellParams {
        fin,
        fslope,
        radius: fslope * radius,
        color_t,
    })
}

pub fn draw_particle_params(
    warmup: f32,
    alpha: f32,
    time: f32,
    particle_life: f32,
    random_life: f32,
    reverse: bool,
    random_angle: f32,
    rotate_scl: f32,
    particle_rad: f32,
    particle_size: f32,
    fade_margin: f32,
) -> Option<DrawParticleParams> {
    if warmup <= 0.0 {
        return None;
    }
    let mut fin = (random_life + time / particle_life) % 1.0;
    if reverse {
        fin = 1.0 - fin;
    }
    let fout = 1.0 - fin;
    let angle = random_angle + (time / rotate_scl) % 360.0;
    let length = particle_rad * pow_in(fout, 1.5);
    let alpha = alpha * warmup * (1.0 - curve(fin, 1.0 - fade_margin, 1.0));
    let size = particle_size * slope(fin) * warmup;
    Some(DrawParticleParams {
        fin,
        fout,
        alpha,
        angle,
        length,
        size,
    })
}

pub fn draw_crucible_flame_params(
    warmup: f32,
    alpha: f32,
    flame_rad: f32,
    circle_space: f32,
    circle_stroke: f32,
    absin: f32,
) -> Option<DrawCrucibleFlameParams> {
    if warmup <= 0.0 {
        return None;
    }
    let a = alpha * warmup;
    Some(DrawCrucibleFlameParams {
        alpha: a,
        middle_radius: flame_rad + absin,
        circle_radius: (flame_rad + circle_space + absin) * warmup,
        stroke: circle_stroke * warmup,
    })
}

pub fn draw_crucible_particle_params(
    warmup: f32,
    alpha: f32,
    time: f32,
    particle_life: f32,
    random_life: f32,
    random_angle: f32,
    rotate_scl: f32,
    particle_rad: f32,
    particle_size: f32,
    fade_margin: f32,
) -> Option<DrawParticleParams> {
    if warmup <= 0.0 {
        return None;
    }
    let fin = (random_life + time / particle_life) % 1.0;
    let fout = 1.0 - fin;
    Some(DrawParticleParams {
        fin,
        fout,
        alpha: alpha * warmup * (1.0 - curve(fin, 1.0 - fade_margin, 1.0)),
        angle: random_angle + (time / rotate_scl) % 360.0,
        length: particle_rad * pow_in(fout, 1.5),
        size: particle_size * fin * warmup,
    })
}

pub fn draw_flame_top_name(block_name: &str) -> String {
    format!("{block_name}-top")
}

pub fn draw_flame_light_clip_size(
    current_light_clip_size: f32,
    light_radius: f32,
    light_sin_mag: f32,
    block_size: i32,
) -> f32 {
    current_light_clip_size.max((light_radius + light_sin_mag) * 2.0 * block_size as f32)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_flame_params(
    warmup: f32,
    flame_alpha: f32,
    absin_alpha: f32,
    random_alpha: f32,
    random_radius: f32,
    flame_radius: f32,
    flame_radius_in: f32,
    outer_absin: f32,
    inner_absin: f32,
) -> Option<DrawFlameParams> {
    if warmup <= 0.0 || flame_alpha <= 0.001 {
        return None;
    }
    let g = 0.3;
    let r = 0.06;
    Some(DrawFlameParams {
        top_alpha: warmup,
        flame_alpha: ((1.0 - g) + absin_alpha + random_alpha - r) * warmup,
        outer_radius: flame_radius + outer_absin + random_radius,
        inner_radius: flame_radius_in + inner_absin + random_radius,
    })
}

pub fn draw_flame_light_radius(light_radius: f32, absin: f32, warmup: f32, block_size: i32) -> f32 {
    (light_radius + absin) * warmup * block_size as f32
}

pub fn draw_plasma_region_names(block_name: &str, suffix: &str, plasmas: i32) -> Vec<String> {
    (0..plasmas)
        .map(|index| format!("{block_name}{suffix}{index}"))
        .collect()
}

pub fn draw_plasma_layer_params(
    region_width_scaled: f32,
    index: usize,
    total_layers: usize,
    absin_radius: f32,
    absin_alpha: f32,
    warmup: f32,
    total_progress: f32,
) -> DrawPlasmaLayerParams {
    DrawPlasmaLayerParams {
        radius: region_width_scaled - 3.0 + absin_radius,
        alpha: (0.3 + absin_alpha) * warmup,
        rotation: total_progress * (12.0 + index as f32 * 6.0),
        color_t: index as f32 / total_layers as f32,
    }
}

pub fn draw_plasma_light_radius(absin: f32, warmup: f32) -> f32 {
    (110.0 + absin) * warmup
}

pub fn draw_plasma_light_alpha(warmup: f32) -> f32 {
    0.8 * warmup
}

pub fn draw_power_region_names(block_name: &str, suffix: &str, mixcol: bool) -> Vec<String> {
    if mixcol {
        vec![format!("{block_name}{suffix}")]
    } else {
        vec![
            format!("{block_name}{suffix}-empty"),
            format!("{block_name}{suffix}-full"),
        ]
    }
}

pub fn draw_power_icons(mixcol: bool, empty_region_found: bool, empty_region: &str) -> Vec<String> {
    if !mixcol && empty_region_found {
        vec![empty_region.to_string()]
    } else {
        Vec::new()
    }
}

pub fn draw_power_square_radius(tilesize: f32, block_size: i32, draw_xscl: f32) -> f32 {
    (tilesize * block_size as f32 / 2.0 - 1.0) * draw_xscl
}

pub fn draw_liquid_output_region_names(
    block_name: &str,
    liquid_names: &[&str],
) -> Vec<[String; 2]> {
    liquid_names
        .iter()
        .map(|liquid| {
            [
                format!("{block_name}-{liquid}-output1"),
                format!("{block_name}-{liquid}-output2"),
            ]
        })
        .collect()
}

pub fn draw_liquid_output_params(
    output_directions: &[i32],
    liquid_count: usize,
    rotation: i32,
) -> Vec<DrawLiquidOutputParams> {
    (0..liquid_count)
        .filter_map(|index| {
            let side = output_directions.get(index).copied().unwrap_or(-1);
            if side == -1 {
                None
            } else {
                let real_rot = (side + rotation).rem_euclid(4);
                Some(DrawLiquidOutputParams {
                    output_index: index,
                    side_variant: if real_rot > 1 { 1 } else { 0 },
                    rotation: real_rot as f32 * 90.0,
                })
            }
        })
        .collect()
}

pub fn draw_arc_smelt_params(
    warmup: f32,
    flame_alpha: f32,
    alpha: f32,
    flame_rad: f32,
    circle_space: f32,
    circle_stroke: f32,
    particle_stroke: f32,
    absin: f32,
) -> Option<DrawArcSmeltParams> {
    if warmup <= 0.0 || flame_alpha <= 0.001 {
        return None;
    }
    Some(DrawArcSmeltParams {
        alpha: alpha * warmup,
        middle_radius: flame_rad + absin,
        circle_radius: (flame_rad + circle_space + absin) * warmup,
        circle_stroke: circle_stroke * warmup,
        particle_stroke: particle_stroke * warmup,
    })
}

pub fn draw_arc_smelt_particle_params(
    time: f32,
    particle_life: f32,
    random_life: f32,
    random_angle: f32,
    particle_rad: f32,
    particle_len: f32,
    warmup: f32,
) -> DrawArcSmeltParticleParams {
    let fin = (random_life + time / particle_life) % 1.0;
    let fout = 1.0 - fin;
    DrawArcSmeltParticleParams {
        fin,
        fout,
        angle: random_angle,
        length: particle_rad * pow2_out(fin),
        line_length: particle_len * fout * warmup,
    }
}

pub fn draw_cultivator_middle_name(block_name: &str) -> String {
    draw_cells_middle_name(block_name)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_piston_params(
    total_progress: f32,
    sin_offset: f32,
    side_offset: f32,
    sin_scl: f32,
    sin_mag: f32,
    len_offset: f32,
    hori_offset: f32,
    angle_offset: f32,
    sides: i32,
    side_index: i32,
) -> DrawPistonParams {
    let len = absin_time(
        total_progress + sin_offset + side_offset * sin_scl * side_index as f32,
        sin_scl,
        sin_mag,
    ) + len_offset;
    let angle = angle_offset + side_index as f32 * 360.0 / sides as f32;
    let region_variant = if approx_eq(angle, 315.0) || approx_eq(angle, 135.0) {
        2
    } else if (135.0..315.0).contains(&angle) {
        1
    } else {
        0
    };
    let radians = angle.to_radians();
    DrawPistonParams {
        region_variant,
        x: radians.cos() * len + (angle + 90.0).to_radians().cos() * -hori_offset,
        y: radians.sin() * len + (angle + 90.0).to_radians().sin() * -hori_offset,
        angle,
        flip_y: approx_eq(angle, 315.0),
    }
}

pub fn draw_piston_region_names(block_name: &str, suffix: &str) -> [String; 4] {
    [
        format!("{block_name}{suffix}0"),
        format!("{block_name}{suffix}1"),
        format!("{block_name}{suffix}-t"),
        format!("{block_name}{suffix}-icon"),
    ]
}

pub fn draw_weave_name(block_name: &str) -> String {
    format!("{block_name}-weave")
}

pub fn draw_weave_params(
    total_progress: f32,
    warmup: f32,
    block_size: i32,
    tilesize: f32,
    x: f32,
) -> DrawWeaveParams {
    DrawWeaveParams {
        rotation: total_progress,
        line_x: x + sin_time(total_progress, 6.0, tilesize / 3.0 * block_size as f32),
        line_length: block_size as f32 * tilesize / 2.0,
        alpha: warmup,
    }
}

pub fn draw_multi_weave_region_names(block_name: &str) -> (String, String) {
    (
        format!("{block_name}-weave"),
        format!("{block_name}-weave-glow"),
    )
}

pub fn draw_multi_weave_params(
    total_progress: f32,
    warmup: f32,
    rotate_speed: f32,
    rotate_speed2: f32,
    fade_weave: bool,
    glow_alpha: f32,
    pulse: f32,
    absin: f32,
) -> DrawMultiWeaveParams {
    DrawMultiWeaveParams {
        first_rotation: total_progress * rotate_speed,
        second_rotation: total_progress * rotate_speed * rotate_speed2,
        weave_alpha: if fade_weave { warmup } else { 1.0 },
        glow_alpha: warmup * (glow_alpha * (1.0 - pulse + absin)),
    }
}

pub fn draw_multi_weave_icons(fade_weave: bool, weave_region: &str) -> Vec<String> {
    if fade_weave {
        Vec::new()
    } else {
        vec![weave_region.to_string()]
    }
}

pub fn draw_soft_particle_region_name() -> &'static str {
    "circle-shadow"
}

#[allow(clippy::too_many_arguments)]
pub fn draw_soft_particle_params(
    warmup: f32,
    color_alpha: f32,
    alpha: f32,
    time: f32,
    particle_life: f32,
    random_life: f32,
    random_angle: f32,
    random_color: f32,
    rotate_scl: f32,
    particle_rad: f32,
    particle_size: f32,
    fade_margin: f32,
) -> Option<DrawSoftParticleParams> {
    if warmup <= 0.0 || color_alpha <= 0.001 {
        return None;
    }
    let raw_fin = (random_life + time / particle_life) % 1.0;
    let raw_fout = 1.0 - raw_fin;
    let fin = 1.0 - raw_fin;
    let fout = 1.0 - raw_fout;
    Some(DrawSoftParticleParams {
        fin,
        fout,
        alpha: alpha * warmup * (1.0 - curve(fin, 1.0 - fade_margin, 1.0)),
        angle: random_angle + (time / rotate_scl) % 360.0,
        length: particle_rad * pow_in(fout, 1.5),
        size: particle_size * fin * warmup * 2.0,
        color_t: random_color,
    })
}

pub fn draw_turret_region_names(
    block_name: &str,
    block_size: i32,
    base_prefix: &str,
    mod_name: Option<&str>,
) -> DrawTurretRegionNames {
    DrawTurretRegionNames {
        preview: format!("{block_name}-preview"),
        outline: format!("{block_name}-outline"),
        liquid: format!("{block_name}-liquid"),
        top: format!("{block_name}-top"),
        heat: format!("{block_name}-heat"),
        base: format!("{block_name}-base"),
        mod_base_fallback: mod_name.map(|name| format!("{name}-{base_prefix}block-{block_size}")),
        vanilla_base_fallback: format!("{base_prefix}block-{block_size}"),
    }
}

pub fn draw_turret_plan_rotation(block_rotate: bool, plan_rotation: i32) -> f32 {
    if block_rotate {
        plan_rotation as f32 * 90.0 - 90.0
    } else {
        0.0
    }
}

pub fn draw_turret_icons(base: &str, preview: &str, top: Option<&str>) -> Vec<String> {
    let mut icons = vec![base.to_string(), preview.to_string()];
    if let Some(top) = top {
        icons.push(top.to_string());
    }
    icons
}

pub fn draw_turret_recoiled(
    x: f32,
    y: f32,
    recoil_x: f32,
    recoil_y: f32,
    rotation: f32,
) -> DrawTurretParams {
    DrawTurretParams {
        x: x + recoil_x,
        y: y + recoil_y,
        rotation,
    }
}

pub fn draw_turret_shadow(
    x: f32,
    y: f32,
    recoil_x: f32,
    recoil_y: f32,
    elevation: f32,
    rotation: f32,
) -> DrawTurretShadowParams {
    DrawTurretShadowParams {
        x: x + recoil_x - elevation,
        y: y + recoil_y - elevation,
        rotation,
    }
}

pub fn draw_turret_liquid_fraction(stored: f32, liquid_capacity: f32) -> f32 {
    stored / liquid_capacity
}

pub fn draw_turret_should_draw_heat(heat: f32, heat_region_found: bool) -> bool {
    heat > 0.00001 && heat_region_found
}

pub fn draw_turret_part_params(
    warmup: f32,
    progress: f32,
    heat: f32,
    recoil: f32,
    charge: f32,
    x: f32,
    y: f32,
    rotation: f32,
) -> (f32, f32, f32, f32, f32, f32, f32, f32, f32) {
    (
        warmup,
        1.0 - progress,
        1.0 - progress,
        heat,
        recoil,
        charge,
        x,
        y,
        rotation,
    )
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn pow3_in_lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t.powi(3)
}

fn pow_in(value: f32, power: f32) -> f32 {
    value.powf(power)
}

fn pow2_out(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(2)
}

fn slope(value: f32) -> f32 {
    1.0 - (value - 0.5).abs() * 2.0
}

fn curve(value: f32, start: f32, end: f32) -> f32 {
    if value <= start {
        0.0
    } else if value >= end {
        1.0
    } else {
        (value - start) / (end - start)
    }
}

fn sin_time(time: f32, scl: f32, mag: f32) -> f32 {
    (time / scl).sin() * mag
}

fn absin_time(time: f32, scl: f32, mag: f32) -> f32 {
    sin_time(time, scl, mag).abs()
}

fn approx_eq(left: f32, right: f32) -> bool {
    (left - right).abs() <= 0.00001
}

fn split_drawer_call(drawer: &str) -> Option<(&str, &str)> {
    let open = drawer.find('(')?;
    if !drawer.ends_with(')') {
        return None;
    }

    Some((drawer[..open].trim(), &drawer[open + 1..drawer.len() - 1]))
}

fn split_drawer_args(args: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;

    for (index, ch) in args.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let part = args[start..index].trim();
                if !part.is_empty() {
                    out.push(part);
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    let tail = args[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }

    out
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
        assert_eq!(draw_region_icons("kiln-top"), vec!["kiln-top"]);
        assert_eq!(draw_region_rotation(10.0, 2.0, 5.0, true, 90.0), 115.0);
        assert_eq!(draw_region_plan_rotation(true, 2, 15.0), 180.0);
        assert_eq!(draw_region_plan_rotation(false, 2, 15.0), 15.0);
        assert_eq!(draw_liquid_region_name("tank", "-liquid"), "tank-liquid");
        assert!(draw_liquid_region_requires_liquids(true).is_ok());
        assert!(draw_liquid_region_requires_liquids(false).is_err());
        assert_eq!(draw_liquid_region_selected(Some("oil"), "water"), "oil");
        assert_eq!(draw_liquid_region_selected(None, "water"), "water");
        assert_eq!(draw_liquid_region_fraction(5.0, 20.0, 0.8), 0.2);
        assert_eq!(draw_heat_region_name("kiln", "-glow"), "kiln-glow");
        assert!((draw_heat_alpha(5.0, 10.0, 0.8, 0.3, 0.1) - 0.32).abs() < 0.00001);
    }

    #[test]
    fn draw_animation_helpers_follow_upstream_formulae() {
        assert_eq!(draw_fade_alpha(0.4, 0.5), 0.2);
        assert_eq!(draw_fade_region_name("press", "-top"), "press-top");
        assert!((draw_warmup_region_alpha(0.5, 0.6, 0.2) - 0.3).abs() < 0.00001);
        assert_eq!(draw_warmup_region_name("press"), "press-top");
        assert_eq!(draw_frames_index(16.0, 5.0, 3, false, 0.0), 0);
        assert_eq!(draw_frames_index(16.0, 5.0, 3, true, 1.9), 1);
        let frames = draw_frames_region_names("press", 3);
        assert_eq!(frames, vec!["press-frame0", "press-frame1", "press-frame2"]);
        assert_eq!(draw_frames_icons(&frames), vec!["press-frame0"]);
        assert_eq!(draw_glow_alpha(0.5, 0.5, 0.8, 0.9), 0.54);
        assert_eq!(draw_glow_rotation(10.0, 2.0, true, 90.0), 110.0);
        assert_eq!(draw_glow_region_name("press", "-glow"), "press-glow");
        assert_eq!(draw_blur_spin_rotation(3.0, 10.0, 15.0), 45.0);
        assert_eq!(
            draw_blur_spin_region_names("separator", "-rotator"),
            ("separator-rotator".into(), "separator-rotator-blur".into())
        );
        assert_eq!(
            draw_blur_spin_params(0.8, 0.7, "fan", "fan-blur", 5.0, 2.0),
            DrawBlurSpinParams {
                region: "fan-blur",
                rotation: 10.0
            }
        );
        assert_eq!(
            draw_blur_spin_params(0.7, 0.7, "fan", "fan-blur", 5.0, 2.0).region,
            "fan"
        );
    }

    #[test]
    fn draw_heat_output_multi_and_pump_helpers_follow_upstream() {
        let alphas = draw_heat_input_side_alphas([1.0, 0.0, 2.0, 4.0], 4.0, 0.8, 0.3, 0.1);
        assert!((alphas[0] - 0.16).abs() < 0.00001);
        assert_eq!(alphas[1], 0.0);
        assert!((alphas[2] - 0.32).abs() < 0.00001);
        assert!((alphas[3] - 0.64).abs() < 0.00001);
        assert_eq!(
            draw_heat_input_region_name("heater", "-heat"),
            "heater-heat"
        );
        assert_eq!(draw_heat_output_rotation(1, 2), 270.0);
        assert_eq!(draw_heat_output_top_index(0, 0), 1);
        assert_eq!(draw_heat_output_top_index(2, 0), 2);
        assert_eq!(
            draw_heat_output_region_names("heater"),
            [
                String::from("heater-heat"),
                String::from("heater-glow"),
                String::from("heater-top1"),
                String::from("heater-top2")
            ]
        );
        assert_eq!(draw_heat_output_static_icon("heater", 0), "heater-top1");
        assert_eq!(draw_heat_output_static_icon("heater", -1), "heater-top2");
        assert!((draw_heat_output_alpha(0.5, 0.8, 0.3, 0.1) - 0.32).abs() < 0.00001);
        assert_eq!(
            draw_multi_icons(&[vec!["a".into(), "b".into()], vec!["c".into()]]),
            vec!["a", "b", "c"]
        );
        assert_eq!(draw_pump_liquid_region_name("pump"), "pump-liquid");
        assert!(draw_pump_liquid_should_draw(true, true));
        assert!(!draw_pump_liquid_should_draw(true, false));
        assert!(!draw_pump_liquid_should_draw(false, true));
        assert_eq!(draw_pump_liquid_fraction(4.0, 16.0), 0.25);
    }

    #[test]
    fn draw_default_side_liquid_tile_and_parts_follow_upstream_shells() {
        assert_eq!(draw_default_icons("router"), vec!["router"]);
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawDefault"),
            vec!["router"]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawRegion(-top)"),
            vec!["router-top"]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawRegion"),
            vec!["router"]
        );
        assert_eq!(
            draw_block_dispatch_icons(
                "router",
                "DrawMulti(DrawRegion(-bottom), DrawLiquidTile, DrawDefault, DrawRegion(-top))"
            ),
            vec!["router-bottom", "router", "router-top"]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawHeatInput"),
            vec!["router-heat"]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawGlowRegion(-ventglow)"),
            vec!["router-ventglow"]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawLiquidRegion"),
            vec!["router-liquid"]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawWarmupRegion"),
            vec!["router-top"]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawLiquidTile"),
            Vec::<String>::new()
        );
        assert_eq!(
            draw_block_dispatch_icons(
                "press",
                "DrawMulti(DrawPistons, DrawWeave, DrawMultiWeave, DrawSideRegion)"
            ),
            vec![
                "press-piston-icon",
                "press-weave",
                "press-weave",
                "press-top1"
            ]
        );
        assert_eq!(
            draw_block_dispatch_icons("router", "DrawMulti(DrawDefault, DrawPumpLiquid)"),
            vec!["router"]
        );
        assert_eq!(
            draw_block_dispatch_icons("scatter", "DrawTurret"),
            vec!["scatter-base", "scatter-preview", "scatter-top"]
        );
        assert_eq!(
            draw_block_dispatch_icons("spectre", "DrawTurret(reinforced-, RegionPart(-barrel))"),
            vec!["spectre-base", "spectre-preview", "spectre-top"]
        );
        assert_eq!(
            draw_block_dispatch_icons("battery", "DrawPower"),
            vec!["battery-power"]
        );
        assert_eq!(
            draw_block_dispatch_icons("battery", "DrawPower(-charge)"),
            vec!["battery-charge"]
        );
        assert_eq!(
            draw_block_dispatch_icons("heater", "DrawHeatOutput"),
            vec!["heater-top1"]
        );
        assert_eq!(
            draw_block_dispatch_icons("heater", "DrawHeatOutput(-1)"),
            vec!["heater-top2"]
        );
        assert_eq!(
            draw_side_region_names("separator"),
            ("separator-top1".into(), "separator-top2".into())
        );
        assert_eq!(draw_side_region_index(0), 0);
        assert_eq!(draw_side_region_index(1), 0);
        assert_eq!(draw_side_region_index(2), 1);
        assert_eq!(draw_side_region_plan_rotation(3), 270.0);
        assert_eq!(
            draw_block_dispatch_icons("separator", "DrawSideRegion"),
            vec!["separator-top1"]
        );
        assert_eq!(
            draw_block_dispatch_icons("kiln", "DrawWeave"),
            vec!["kiln-weave"]
        );
        assert_eq!(
            draw_block_dispatch_icons("phase-weaver", "DrawMultiWeave"),
            vec!["phase-weaver-weave"]
        );
        assert_eq!(
            draw_block_dispatch_icons("press", "DrawPistons"),
            vec!["press-piston-icon"]
        );

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

    #[test]
    fn draw_bubbles_cells_particles_and_flames_follow_upstream_formulae() {
        assert_eq!(
            draw_bubble_params(0.001, 20.0, 30.0, 6.0, 0.0, 3.0, 0.2),
            None
        );
        let bubble = draw_bubble_params(0.5, 15.0, 30.0, 6.0, 0.25, 3.0, 0.2).unwrap();
        assert!((bubble.life - 0.25).abs() < 0.00001);
        assert!((bubble.radius - 2.25).abs() < 0.00001);
        assert!((bubble.stroke - 0.225).abs() < 0.00001);

        assert_eq!(draw_cells_middle_name("cultivator"), "cultivator-middle");
        let cell = draw_cell_params(0.8, 30.0, 60.0, 180.0, 2.0, 1.8, 0.35).unwrap();
        assert!((cell.fin - 0.5).abs() < 0.00001);
        assert!((cell.fslope - 1.0).abs() < 0.00001);
        assert!((cell.radius - 1.8).abs() < 0.00001);
        assert!((cell.color_t - 0.35).abs() < 0.00001);

        let particle =
            draw_particle_params(0.5, 0.5, 35.0, 70.0, 0.25, false, 90.0, 3.0, 7.0, 3.0, 0.4)
                .unwrap();
        assert!((particle.fin - 0.75).abs() < 0.00001);
        assert!((particle.fout - 0.25).abs() < 0.00001);
        assert!((particle.alpha - 0.15625).abs() < 0.00001);
        assert!((particle.angle - 101.666664).abs() < 0.00001);
        assert!((particle.length - 0.875).abs() < 0.00001);
        assert!((particle.size - 0.75).abs() < 0.00001);

        let reversed =
            draw_particle_params(0.5, 0.5, 35.0, 70.0, 0.25, true, 90.0, 3.0, 7.0, 3.0, 0.4)
                .unwrap();
        assert!((reversed.fin - 0.25).abs() < 0.00001);
        assert!((reversed.fout - 0.75).abs() < 0.00001);

        let crucible = draw_crucible_flame_params(0.5, 0.5, 1.0, 2.0, 1.5, 0.4).unwrap();
        assert!((crucible.alpha - 0.25).abs() < 0.00001);
        assert!((crucible.middle_radius - 1.4).abs() < 0.00001);
        assert!((crucible.circle_radius - 1.7).abs() < 0.00001);
        assert!((crucible.stroke - 0.75).abs() < 0.00001);

        let cparticle =
            draw_crucible_particle_params(0.5, 0.5, 35.0, 70.0, 0.25, 90.0, 1.5, 7.0, 3.0, 0.4)
                .unwrap();
        assert!((cparticle.fin - 0.75).abs() < 0.00001);
        assert!((cparticle.size - 1.125).abs() < 0.00001);
    }

    #[test]
    fn draw_flame_plasma_power_and_liquid_outputs_follow_upstream_shells() {
        assert_eq!(
            draw_flame_top_name("combustion-generator"),
            "combustion-generator-top"
        );
        assert_eq!(draw_flame_light_clip_size(10.0, 60.0, 5.0, 2), 260.0);
        assert_eq!(draw_flame_light_radius(60.0, 5.0, 0.5, 2), 65.0);
        let flame = draw_flame_params(0.5, 1.0, 0.2, 0.03, 0.04, 3.0, 1.9, 0.6, 0.2).unwrap();
        assert_eq!(flame.top_alpha, 0.5);
        assert!((flame.flame_alpha - 0.435).abs() < 0.00001);
        assert!((flame.outer_radius - 3.64).abs() < 0.00001);
        assert!((flame.inner_radius - 2.14).abs() < 0.00001);

        assert_eq!(
            draw_plasma_region_names("thorium-reactor", "-plasma-", 3),
            vec![
                "thorium-reactor-plasma-0",
                "thorium-reactor-plasma-1",
                "thorium-reactor-plasma-2"
            ]
        );
        let plasma = draw_plasma_layer_params(20.0, 2, 4, 3.0, 0.2, 0.5, 10.0);
        assert_eq!(plasma.radius, 20.0);
        assert_eq!(plasma.alpha, 0.25);
        assert_eq!(plasma.rotation, 240.0);
        assert_eq!(plasma.color_t, 0.5);
        assert_eq!(draw_plasma_light_radius(5.0, 0.5), 57.5);
        assert_eq!(draw_plasma_light_alpha(0.5), 0.4);

        assert_eq!(
            draw_power_region_names("battery", "-power", true),
            vec!["battery-power"]
        );
        assert_eq!(
            draw_power_region_names("battery", "-power", false),
            vec!["battery-power-empty", "battery-power-full"]
        );
        assert_eq!(
            draw_power_icons(false, true, "battery-power-empty"),
            vec!["battery-power-empty"]
        );
        assert!(draw_power_icons(true, true, "battery-power").is_empty());
        assert_eq!(draw_power_square_radius(8.0, 3, 1.0), 11.0);

        assert_eq!(
            draw_liquid_output_region_names("melter", &["slag", "water"]),
            vec![
                [
                    String::from("melter-slag-output1"),
                    String::from("melter-slag-output2")
                ],
                [
                    String::from("melter-water-output1"),
                    String::from("melter-water-output2")
                ]
            ]
        );
        assert_eq!(
            draw_liquid_output_params(&[0, 2, -1], 4, 1),
            vec![
                DrawLiquidOutputParams {
                    output_index: 0,
                    side_variant: 0,
                    rotation: 90.0
                },
                DrawLiquidOutputParams {
                    output_index: 1,
                    side_variant: 1,
                    rotation: 270.0
                }
            ]
        );

        assert_eq!(
            draw_block_dispatch_icons("combustion-generator", "DrawFlame(ffc999)"),
            vec!["combustion-generator-top"]
        );
        assert_eq!(
            draw_block_dispatch_icons("surge-crucible", "DrawHeatRegion"),
            vec!["surge-crucible-glow"]
        );
        assert_eq!(
            draw_block_dispatch_icons("surge-crucible", "DrawHeatRegion(-vents)"),
            vec!["surge-crucible-vents"]
        );
        assert_eq!(
            draw_block_dispatch_icons("cyanogen-synthesizer", "DrawLiquidOutputs"),
            Vec::<String>::new()
        );
        assert_eq!(
            draw_block_dispatch_icons("cyanogen-synthesizer", "DrawParticles"),
            Vec::<String>::new()
        );
        assert_eq!(
            draw_block_dispatch_icons(
                "surge-crucible",
                "DrawMulti(DrawFlame(ffc999), DrawHeatRegion(-vents), DrawLiquidOutputs, DrawParticles, DrawDefault)"
            ),
            vec![
                "surge-crucible-top",
                "surge-crucible-vents",
                "surge-crucible"
            ]
        );
    }

    #[test]
    fn draw_particles_and_soft_particles_dispatch_particle_configs_without_sprites() {
        assert_eq!(
            draw_block_dispatch_icons("cyanogen-synthesizer", "DrawParticles"),
            Vec::<String>::new()
        );
        assert_eq!(
            draw_block_dispatch_icons("cyanogen-synthesizer", "DrawSoftParticles"),
            Vec::<String>::new()
        );
        assert_eq!(
            draw_block_dispatch_particle_configs("cyanogen-synthesizer", "DrawParticles"),
            vec![draw_particles_block_config_for("cyanogen-synthesizer")]
        );
        assert_eq!(
            draw_block_dispatch_particle_configs("cyanogen-synthesizer", "DrawSoftParticles"),
            vec![draw_soft_particles_block_config()]
        );
        assert_eq!(
            draw_block_dispatch_particle_configs(
                "surge-crucible",
                "DrawMulti(DrawParticles, DrawSoftParticles, DrawDefault)"
            ),
            vec![
                draw_particles_block_config(),
                draw_soft_particles_block_config()
            ]
        );

        let atmospheric =
            draw_block_dispatch_particle_configs("atmospheric-concentrator", "DrawParticles");
        assert_eq!(atmospheric[0].color_rgba, 0xd4f0ffff);
        assert_eq!(atmospheric[0].particle_count, 10);
        assert_eq!(atmospheric[0].particle_radius, 12.0);
        assert_eq!(atmospheric[0].particle_life, 140.0);
        assert_eq!(atmospheric[0].random_life_range, 2.0);
        assert_eq!(
            atmospheric[0].render_kind,
            DrawBlockParticleRenderKind::Circle
        );
        assert_eq!(
            atmospheric[0].size_interp,
            DrawBlockParticleSizeInterp::Slope
        );

        let cyanogen =
            draw_block_dispatch_particle_configs("cyanogen-synthesizer", "DrawParticles");
        assert_eq!(cyanogen[0].color_rgba, 0x89e8b6ff);
        assert!(cyanogen[0].reverse);
        assert_eq!(cyanogen[0].size_interp, DrawBlockParticleSizeInterp::One);

        let flux = draw_block_dispatch_particle_configs("flux-reactor", "DrawSoftParticles");
        assert_eq!(flux[0].alpha, 0.35);
        assert_eq!(flux[0].particle_count, 27);
        assert_eq!(flux[0].particle_size, 9.0);
        assert_eq!(flux[0].blend_mode, DrawBlockParticleBlendMode::Additive);
        assert_eq!(flux[0].render_kind, DrawBlockParticleRenderKind::SoftSprite);
        assert_eq!(flux[0].region, Some(draw_soft_particle_region_name()));
        assert!(flux[0].invert_life);
    }

    #[test]
    fn draw_arc_smelt_pistons_and_weaves_follow_upstream_formulae() {
        let arc = draw_arc_smelt_params(0.5, 1.0, 0.68, 1.0, 2.0, 1.5, 1.1, 0.3).unwrap();
        assert!((arc.alpha - 0.34).abs() < 0.00001);
        assert!((arc.middle_radius - 1.3).abs() < 0.00001);
        assert!((arc.circle_radius - 1.65).abs() < 0.00001);
        assert!((arc.circle_stroke - 0.75).abs() < 0.00001);
        assert!((arc.particle_stroke - 0.55).abs() < 0.00001);
        let arc_particle = draw_arc_smelt_particle_params(20.0, 40.0, 0.25, 90.0, 7.0, 3.0, 0.5);
        assert!((arc_particle.fin - 0.75).abs() < 0.00001);
        assert!((arc_particle.fout - 0.25).abs() < 0.00001);
        assert!((arc_particle.length - 6.5625).abs() < 0.00001);
        assert!((arc_particle.line_length - 0.375).abs() < 0.00001);

        assert_eq!(
            draw_cultivator_middle_name("cultivator"),
            "cultivator-middle"
        );
        assert_eq!(
            draw_piston_region_names("press", "-piston"),
            [
                String::from("press-piston0"),
                String::from("press-piston1"),
                String::from("press-piston-t"),
                String::from("press-piston-icon")
            ]
        );
        let piston = draw_piston_params(0.0, 0.0, 0.0, 6.0, 4.0, -1.0, 2.0, 0.0, 4, 1);
        assert_eq!(piston.region_variant, 0);
        assert!((piston.x - 2.0).abs() < 0.00001);
        assert!((piston.y + 1.0).abs() < 0.00001);
        assert_eq!(piston.angle, 90.0);
        assert!(!piston.flip_y);
        let piston_t = draw_piston_params(0.0, 0.0, 0.0, 6.0, 4.0, -1.0, 0.0, 45.0, 4, 1);
        assert_eq!(piston_t.region_variant, 2);
        assert_eq!(piston_t.angle, 135.0);

        assert_eq!(draw_weave_name("kiln"), "kiln-weave");
        let weave = draw_weave_params(0.0, 0.75, 3, 8.0, 1.0);
        assert_eq!(weave.rotation, 0.0);
        assert_eq!(weave.line_x, 1.0);
        assert_eq!(weave.line_length, 12.0);
        assert_eq!(weave.alpha, 0.75);
        assert_eq!(
            draw_multi_weave_region_names("phase-weaver"),
            (
                String::from("phase-weaver-weave"),
                String::from("phase-weaver-weave-glow")
            )
        );
        let multi = draw_multi_weave_params(10.0, 0.5, 1.0, -0.9, true, 0.8, 0.3, 0.1);
        assert_eq!(multi.first_rotation, 10.0);
        assert_eq!(multi.second_rotation, -9.0);
        assert_eq!(multi.weave_alpha, 0.5);
        assert!((multi.glow_alpha - 0.32).abs() < 0.00001);
        assert!(draw_multi_weave_icons(true, "phase-weaver-weave").is_empty());
        assert_eq!(
            draw_multi_weave_icons(false, "phase-weaver-weave"),
            vec!["phase-weaver-weave"]
        );
    }

    #[test]
    fn draw_soft_particles_and_turret_shells_follow_upstream() {
        assert_eq!(draw_soft_particle_region_name(), "circle-shadow");
        let soft = draw_soft_particle_params(
            0.5, 1.0, 0.5, 35.0, 70.0, 0.25, 90.0, 0.4, 1.5, 7.0, 3.0, 0.4,
        )
        .unwrap();
        assert!((soft.fin - 0.25).abs() < 0.00001);
        assert!((soft.fout - 0.75).abs() < 0.00001);
        assert!((soft.alpha - 0.25).abs() < 0.00001);
        assert!((soft.angle - 113.333336).abs() < 0.00001);
        assert!((soft.length - 4.546633).abs() < 0.00001);
        assert!((soft.size - 0.75).abs() < 0.00001);
        assert_eq!(soft.color_t, 0.4);

        let names = draw_turret_region_names("duo", 2, "reinforced-", Some("erekir"));
        assert_eq!(names.preview, "duo-preview");
        assert_eq!(names.outline, "duo-outline");
        assert_eq!(names.liquid, "duo-liquid");
        assert_eq!(names.top, "duo-top");
        assert_eq!(names.heat, "duo-heat");
        assert_eq!(names.base, "duo-base");
        assert_eq!(
            names.mod_base_fallback,
            Some(String::from("erekir-reinforced-block-2"))
        );
        assert_eq!(names.vanilla_base_fallback, "reinforced-block-2");
        assert_eq!(draw_turret_plan_rotation(true, 2), 90.0);
        assert_eq!(draw_turret_plan_rotation(false, 2), 0.0);
        assert_eq!(
            draw_turret_icons("block-2", "duo-preview", Some("duo-top")),
            vec!["block-2", "duo-preview", "duo-top"]
        );
        assert_eq!(
            draw_turret_icons("block-2", "duo-preview", None),
            vec!["block-2", "duo-preview"]
        );
        assert_eq!(
            draw_turret_recoiled(10.0, 20.0, 1.0, -2.0, 90.0),
            DrawTurretParams {
                x: 11.0,
                y: 18.0,
                rotation: 90.0
            }
        );
        assert_eq!(
            draw_turret_shadow(10.0, 20.0, 1.0, -2.0, 3.0, 90.0),
            DrawTurretShadowParams {
                x: 8.0,
                y: 15.0,
                rotation: 90.0
            }
        );
        assert_eq!(draw_turret_liquid_fraction(5.0, 20.0), 0.25);
        assert!(!draw_turret_should_draw_heat(0.00001, true));
        assert!(!draw_turret_should_draw_heat(0.1, false));
        assert!(draw_turret_should_draw_heat(0.1, true));
        assert_eq!(
            draw_turret_part_params(0.5, 0.25, 0.8, 1.2, 0.3, 11.0, 18.0, 90.0),
            (0.5, 0.75, 0.75, 0.8, 1.2, 0.3, 11.0, 18.0, 90.0)
        );
    }
}
