#[cfg(feature = "opengl-native-runtime")]
use glow::HasContext;
#[cfg(feature = "opengl-native-runtime")]
use glutin::config::ConfigTemplateBuilder;
#[cfg(feature = "opengl-native-runtime")]
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
#[cfg(feature = "opengl-native-runtime")]
use glutin::display::GetGlDisplay;
#[cfg(feature = "opengl-native-runtime")]
use glutin::prelude::*;
#[cfg(feature = "opengl-native-runtime")]
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
#[cfg(feature = "opengl-native-runtime")]
use glutin_winit::{DisplayBuilder, GlWindow};
#[cfg(feature = "opengl-native-runtime")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(feature = "opengl-native-runtime")]
use std::num::NonZeroU32;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let mut launcher = mindustry_desktop::run(args.clone());
    if let Some(error) = &launcher.connect_error {
        eprintln!(
            "{} failed_to_connect={}",
            mindustry_desktop::banner(),
            error
        );
        return;
    }

    println!(
        "{} data_dir={} graphics_backend={}",
        mindustry_desktop::banner(),
        launcher.client.context.paths.data_dir,
        desktop_graphics_backend_label(),
    );

    run_desktop_frame_loop(&mut launcher, args);
}

#[cfg(not(feature = "opengl-backend"))]
fn desktop_graphics_backend_label() -> &'static str {
    "headless"
}

#[cfg(all(feature = "opengl-backend", not(feature = "opengl-native-runtime")))]
fn desktop_graphics_backend_label() -> &'static str {
    "opengl-backend:null-runtime-submit"
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_graphics_backend_label() -> &'static str {
    "opengl-native-runtime:glutin-glow-recording-driver"
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_trace_enabled() -> bool {
    std::env::var_os("MINDUSTRY_DESKTOP_TRACE").is_some()
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_trace_summary_enabled() -> bool {
    desktop_native_trace_enabled() || std::env::var_os("MINDUSTRY_DESKTOP_TRACE_SUMMARY").is_some()
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_trace(message: impl AsRef<str>) {
    if desktop_native_trace_enabled() {
        eprintln!("[desktop-native] {}", message.as_ref());
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_trace_summary(message: impl AsRef<str>) {
    if desktop_native_trace_summary_enabled() {
        eprintln!("[desktop-native] {}", message.as_ref());
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_window_title_with_diagnostic(
    base_title: &str,
    diagnostic: Option<&str>,
) -> String {
    match diagnostic {
        Some(diagnostic) if !diagnostic.is_empty() => format!("{base_title} - {diagnostic}"),
        _ => base_title.to_string(),
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_error_categories(errors: &[String]) -> Vec<&'static str> {
    let mut categories = Vec::new();
    let mut push_category = |category: &'static str| {
        if !categories.contains(&category) {
            categories.push(category);
        }
    };

    for error in errors {
        let error = error.to_ascii_lowercase();
        if error.contains("shader") {
            push_category("shader");
        }
        if error.contains("texture") {
            push_category("texture");
        }
        if error.contains("program") {
            push_category("program");
        }
        if error.contains("framebuffer") {
            push_category("framebuffer");
        }
        if error.contains("uniform") {
            push_category("uniform");
        }
    }

    categories
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_submit_diagnostic(
    driver_state: &mindustry_desktop::DesktopGraphicsOpenGlBackendDriverExecutionState,
    invalid_draw_commands: usize,
    native_errors: &[String],
    shader_assets_available: bool,
    font_assets_available: bool,
) -> Option<String> {
    let total_commands = driver_state.framebuffer_attachment_plans
        + driver_state.texture_upload_commands
        + driver_state.sprite_mesh_upload_commands
        + driver_state.resolve_mesh_upload_commands
        + driver_state.shader_commands
        + driver_state.draw_commands
        + driver_state.resolve_draw_commands
        + driver_state.resolve_commands;

    let mut reasons = Vec::new();

    if !shader_assets_available {
        reasons.push("shader assets unavailable".to_string());
    }

    if !font_assets_available {
        reasons.push("font assets unavailable".to_string());
    }

    if total_commands == 0 {
        reasons.push("empty render frame: no GPU commands were recorded".to_string());
    }

    if driver_state.draw_commands == 0 && driver_state.resolve_draw_commands == 0 {
        if total_commands > 0 {
            reasons.push("no draw commands reached the GPU".to_string());
        }
    } else if driver_state.draw_commands > 0 {
        if invalid_draw_commands >= driver_state.draw_commands {
            reasons.push(format!(
                "no valid draw commands: {} draw submissions were skipped",
                invalid_draw_commands
            ));
        } else if invalid_draw_commands > 0 {
            reasons.push(format!(
                "some draw submissions were skipped: {}",
                invalid_draw_commands
            ));
        }
    }

    if !native_errors.is_empty() {
        let categories = desktop_native_opengl_error_categories(native_errors);
        let latest_error = native_errors
            .last()
            .map_or("unknown native OpenGL failure", String::as_str);
        if categories.is_empty() {
            reasons.push(format!("native failure: {latest_error}"));
        } else {
            reasons.push(format!(
                "native failure({}): {latest_error}",
                categories.join("/")
            ));
        }
    }

    if reasons.is_empty() {
        None
    } else {
        Some(reasons.join("; "))
    }
}

#[cfg(feature = "opengl-native-runtime")]
#[derive(Debug, Clone, Copy, PartialEq)]
struct DesktopNativeVisibleFallbackRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: [f32; 4],
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_submit_needs_visible_fallback(
    driver_state: &mindustry_desktop::DesktopGraphicsOpenGlBackendDriverExecutionState,
    invalid_draw_commands: usize,
    native_errors: &[String],
    shader_assets_available: bool,
    font_assets_available: bool,
) -> bool {
    if !shader_assets_available {
        return true;
    }
    if !font_assets_available {
        return true;
    }
    if !native_errors.is_empty() {
        return true;
    }
    if driver_state.draw_commands == 0 && driver_state.resolve_draw_commands == 0 {
        return true;
    }
    driver_state.draw_commands > 0 && invalid_draw_commands >= driver_state.draw_commands
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_visible_fallback_rects(
    surface_size: mindustry_desktop::DesktopSurfaceSize,
) -> Vec<DesktopNativeVisibleFallbackRect> {
    fn clamp_span(value: i32, min: i32, max: i32) -> i32 {
        if max < min {
            max.max(0)
        } else {
            value.clamp(min, max)
        }
    }

    let width = surface_size.width.max(1).min(i32::MAX as u32) as i32;
    let height = surface_size.height.max(1).min(i32::MAX as u32) as i32;
    let mut rects = Vec::with_capacity(20);
    let push_rect = |rects: &mut Vec<DesktopNativeVisibleFallbackRect>,
                     x: i32,
                     y: i32,
                     width: i32,
                     height: i32,
                     color: [f32; 4]| {
        if width > 0 && height > 0 {
            rects.push(DesktopNativeVisibleFallbackRect {
                x,
                y,
                width,
                height,
                color,
            });
        }
    };

    let background = [0.014, 0.017, 0.023, 1.0];
    let panel_shadow = [0.021, 0.026, 0.033, 1.0];
    let panel_fill = [0.040, 0.049, 0.061, 1.0];
    let panel_edge = [0.071, 0.084, 0.098, 1.0];
    let row_fill = [0.053, 0.063, 0.077, 1.0];
    let row_fill_alt = [0.047, 0.056, 0.069, 1.0];
    let accent = [0.18, 0.42, 0.56, 1.0];
    let accent_soft = [0.24, 0.53, 0.69, 1.0];
    let text = [0.82, 0.86, 0.92, 1.0];
    let text_dim = [0.58, 0.64, 0.70, 1.0];

    push_rect(&mut rects, 0, 0, width, height, background);

    let side_margin = clamp_span(width / 12, 16, 84);
    let top_margin = clamp_span(height / 10, 18, 72);
    let bottom_margin = clamp_span(height / 14, 18, 72);

    let panel_width = clamp_span(width / 5, 150, width.saturating_sub(side_margin * 2).max(1));
    let button_count = 6;
    let button_height = clamp_span(height / 16, 30, 54);
    let button_gap = clamp_span(button_height / 6, 4, 8);
    let panel_padding_y = clamp_span(height / 36, 8, 18);
    let panel_height =
        panel_padding_y * 2 + button_count * button_height + (button_count - 1) * button_gap;
    let panel_x = side_margin;
    let panel_y = clamp_span(
        (height - panel_height) / 2,
        top_margin,
        height.saturating_sub(bottom_margin + panel_height),
    );

    let panel_shadow_x = (panel_x - 4).max(0);
    let panel_shadow_y = (panel_y - 4).max(0);
    let panel_shadow_width = (panel_width + 8).min(width.saturating_sub(panel_shadow_x));
    let panel_shadow_height = (panel_height + 8).min(height.saturating_sub(panel_shadow_y));
    push_rect(
        &mut rects,
        panel_shadow_x,
        panel_shadow_y,
        panel_shadow_width,
        panel_shadow_height,
        panel_shadow,
    );
    push_rect(
        &mut rects,
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        panel_fill,
    );
    push_rect(&mut rects, panel_x, panel_y, panel_width, 4, panel_edge);
    push_rect(
        &mut rects,
        panel_x,
        panel_y + panel_height - 4,
        panel_width,
        4,
        panel_edge,
    );
    push_rect(&mut rects, panel_x, panel_y, 4, panel_height, panel_edge);
    push_rect(
        &mut rects,
        panel_x + panel_width - 4,
        panel_y,
        4,
        panel_height,
        panel_edge,
    );

    let row_inset_x = clamp_span(panel_width / 12, 10, 22);
    let row_width = (panel_width - row_inset_x * 2).max(1);
    let row_x = panel_x + row_inset_x;
    let icon_size = clamp_span(button_height - 14, 10, 22);
    let text_height = clamp_span(button_height / 8, 4, 6);
    let text_y_offset = (button_height - text_height) / 2;
    let row_top = panel_y + panel_padding_y;
    for index in 0..button_count {
        let row_y = row_top + index * (button_height + button_gap);
        let row_color = if index == 0 {
            accent
        } else if index % 2 == 0 {
            row_fill
        } else {
            row_fill_alt
        };
        let text_color = if index == 0 { text } else { text_dim };
        push_rect(
            &mut rects,
            row_x,
            row_y,
            row_width,
            button_height,
            row_color,
        );
        push_rect(
            &mut rects,
            row_x + 8,
            row_y + 7,
            icon_size,
            icon_size,
            if index == 0 { accent_soft } else { panel_edge },
        );
        push_rect(
            &mut rects,
            row_x + icon_size + 18,
            row_y + text_y_offset,
            (row_width - icon_size - 28).max(10),
            text_height,
            text_color,
        );
        if index == 0 {
            push_rect(&mut rects, row_x, row_y, 6, button_height, accent_soft);
        }
    }

    let logo_width = clamp_span(width / 3, 260, width.saturating_sub(side_margin * 2).max(1));
    let logo_height = clamp_span(logo_width / 6, 44, 92);
    let logo_x = (width - logo_width) / 2;
    let logo_y = height.saturating_sub(top_margin + logo_height);
    push_rect(
        &mut rects,
        (logo_x - 4).max(0),
        (logo_y - 4).max(0),
        (logo_width + 8).min(width.saturating_sub((logo_x - 4).max(0))),
        (logo_height + 8).min(height.saturating_sub((logo_y - 4).max(0))),
        panel_shadow,
    );
    push_rect(
        &mut rects,
        logo_x,
        logo_y,
        logo_width,
        logo_height,
        panel_fill,
    );
    push_rect(
        &mut rects,
        logo_x + logo_width / 12,
        logo_y + logo_height / 5,
        (logo_width * 5 / 6).max(1),
        (logo_height / 4).max(6),
        accent_soft,
    );
    push_rect(
        &mut rects,
        logo_x + logo_width / 4,
        logo_y + logo_height / 2,
        (logo_width / 2).max(1),
        (logo_height / 10).max(4),
        text,
    );
    push_rect(
        &mut rects,
        logo_x + logo_width / 3,
        logo_y + logo_height * 2 / 3,
        (logo_width / 3).max(1),
        (logo_height / 12).max(3),
        text_dim,
    );

    let version_width = clamp_span(logo_width / 2, 150, logo_width);
    let version_x = logo_x + (logo_width - version_width) / 2;
    let version_y = (logo_y - clamp_span(height / 60, 8, 14)).max(0);
    push_rect(
        &mut rects,
        version_x,
        version_y,
        version_width,
        text_height,
        text_dim,
    );
    push_rect(
        &mut rects,
        version_x + version_width / 6,
        version_y + text_height + 3,
        (version_width * 2 / 3).max(1),
        text_height,
        text,
    );

    let diagnostic_width = clamp_span(width / 4, 180, 420);
    let diagnostic_x = (width - diagnostic_width - side_margin).max(0);
    let diagnostic_y = clamp_span(height / 18, 10, 28);
    push_rect(
        &mut rects,
        diagnostic_x,
        diagnostic_y,
        diagnostic_width,
        text_height,
        text_dim,
    );
    push_rect(
        &mut rects,
        diagnostic_x + diagnostic_width / 8,
        diagnostic_y + text_height + 4,
        (diagnostic_width * 3 / 4).max(1),
        text_height,
        text,
    );
    push_rect(
        &mut rects,
        diagnostic_x + diagnostic_width / 4,
        diagnostic_y + (text_height + 4) * 2,
        (diagnostic_width / 2).max(1),
        text_height,
        accent_soft,
    );

    rects
}

#[cfg(feature = "opengl-native-runtime")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct DesktopNativeOpenGlShaderAssetRootResolution {
    path: std::path::PathBuf,
    source: &'static str,
    shaders_dir_exists: bool,
    fonts_dir_exists: bool,
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_shader_asset_root_resolution_from_candidates(
    candidates: Vec<(std::path::PathBuf, &'static str)>,
    fallback: std::path::PathBuf,
) -> DesktopNativeOpenGlShaderAssetRootResolution {
    for (candidate, source) in candidates {
        let shaders_dir_exists = candidate.join("shaders").is_dir();
        let fonts_dir_exists = candidate.join("fonts").is_dir();
        desktop_native_trace(format!(
            "shader_asset_root: candidate source={source} path={} shaders_dir_exists={shaders_dir_exists} fonts_dir_exists={fonts_dir_exists}",
            candidate.display()
        ));
        if shaders_dir_exists {
            return DesktopNativeOpenGlShaderAssetRootResolution {
                path: candidate,
                source,
                shaders_dir_exists,
                fonts_dir_exists,
            };
        }
    }

    let shaders_dir_exists = fallback.join("shaders").is_dir();
    let fonts_dir_exists = fallback.join("fonts").is_dir();
    desktop_native_trace(format!(
        "shader_asset_root: fallback source=fallback path={} shaders_dir_exists={shaders_dir_exists} fonts_dir_exists={fonts_dir_exists}",
        fallback.display()
    ));
    DesktopNativeOpenGlShaderAssetRootResolution {
        path: fallback,
        source: "fallback",
        shaders_dir_exists,
        fonts_dir_exists,
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_push_shader_asset_root_candidates_near(
    candidates: &mut Vec<(std::path::PathBuf, &'static str)>,
    base: &std::path::Path,
    source: &'static str,
) {
    candidates.push((base.join("core").join("assets"), source));
    candidates.push((base.join("assets"), source));
    candidates.push((
        base.join("..")
            .join("mindustry-upstream-v157.4")
            .join("core")
            .join("assets"),
        source,
    ));
    candidates.push((
        base.join("..")
            .join("_upstream_mindustry")
            .join("core")
            .join("assets"),
        source,
    ));
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_dedup_shader_asset_root_candidates(
    candidates: Vec<(std::path::PathBuf, &'static str)>,
) -> Vec<(std::path::PathBuf, &'static str)> {
    let mut deduped = Vec::new();
    for (path, source) in candidates {
        if !deduped
            .iter()
            .any(|(known, _): &(std::path::PathBuf, &'static str)| known == &path)
        {
            deduped.push((path, source));
        }
    }
    deduped
}

#[cfg(not(feature = "opengl-backend"))]
fn run_desktop_frame_loop(launcher: &mut mindustry_desktop::DesktopLauncher, _args: Vec<String>) {
    let mut effect_renderer = mindustry_desktop::HeadlessDesktopEffectRenderer::default();
    let mut graphics_renderer = mindustry_desktop::HeadlessDesktopGraphicsRenderer::default();
    let mut frame_loop = mindustry_desktop::DesktopFrameLoopState::default();

    launcher.run_with_desktop_frame_loop(
        &mut frame_loop,
        &mut graphics_renderer,
        &mut effect_renderer,
        None,
        |_| vec![mindustry_desktop::DesktopFrameLoopEvent::Tick],
        |_| {},
        std::thread::sleep,
    );
}

#[cfg(all(feature = "opengl-backend", not(feature = "opengl-native-runtime")))]
fn run_desktop_frame_loop(launcher: &mut mindustry_desktop::DesktopLauncher, _args: Vec<String>) {
    let opengl_runtime = mindustry_desktop::DesktopGraphicsNullOpenGlBackendRuntime::default();
    let mut effect_renderer = mindustry_desktop::HeadlessDesktopEffectRenderer::default();
    let mut graphics_renderer =
        mindustry_desktop::DesktopOpenGlBackendGraphicsRenderer::new(opengl_runtime);
    let mut frame_loop = mindustry_desktop::DesktopFrameLoopState::default();

    launcher.run_with_desktop_frame_loop(
        &mut frame_loop,
        &mut graphics_renderer,
        &mut effect_renderer,
        None,
        |_| vec![mindustry_desktop::DesktopFrameLoopEvent::Tick],
        |_| {},
        std::thread::sleep,
    );
}

#[cfg(feature = "opengl-native-runtime")]
fn run_desktop_frame_loop(launcher: &mut mindustry_desktop::DesktopLauncher, args: Vec<String>) {
    let event_loop = winit::event_loop::EventLoop::new()
        .expect("failed to create winit event loop for native OpenGL runtime");
    let native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_args(args);
    let mut app = DesktopNativeOpenGlApp::new(launcher, native_config);
    event_loop
        .run_app(&mut app)
        .expect("native OpenGL desktop event loop failed");
}

#[cfg(feature = "opengl-native-runtime")]
struct DesktopNativeOpenGlApp<'a> {
    launcher: &'a mut mindustry_desktop::DesktopLauncher,
    native_config: mindustry_desktop::DesktopNativeOpenGlRuntimeConfig,
    window_id: Option<winit::window::WindowId>,
    frame_loop: mindustry_desktop::DesktopFrameLoopState,
    next_redraw_at: std::time::Instant,
    graphics_renderer:
        Option<mindustry_desktop::DesktopOpenGlBackendGraphicsRenderer<DesktopNativeOpenGlRuntime>>,
    runtime_init_error: Option<String>,
    effect_renderer: mindustry_desktop::HeadlessDesktopEffectRenderer,
    pending_events: Vec<mindustry_desktop::DesktopFrameLoopEvent>,
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_frame_pacing(
    launcher: &mindustry_desktop::DesktopLauncher,
    native_config: &mindustry_desktop::DesktopNativeOpenGlRuntimeConfig,
) -> mindustry_desktop::DesktopFramePacing {
    if native_config.vsync {
        mindustry_desktop::DesktopFramePacing::uncapped()
    } else {
        launcher.settings_frame_pacing(native_config.fps_cap)
    }
}

#[cfg(feature = "opengl-native-runtime")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct DesktopNativeOpenGlBufferUploadCacheEntry {
    usage: u32,
    bytes: Vec<u8>,
}

#[cfg(feature = "opengl-native-runtime")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DesktopNativeOpenGlVertexAttributeLayoutCacheEntry {
    components: i32,
    gl_type: u32,
    normalized: bool,
    stride_bytes: i32,
    offset_bytes: i32,
}

#[cfg(feature = "opengl-native-runtime")]
struct DesktopNativeOpenGlRuntime {
    window: winit::window::Window,
    surface: Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
    gl: glow::Context,
    state: mindustry_desktop::DesktopGraphicsOpenGlBackendRuntimeState,
    driver: mindustry_desktop::DesktopGraphicsRecordingOpenGlBackendDriver,
    textures: std::collections::BTreeMap<u32, glow::NativeTexture>,
    framebuffers: std::collections::BTreeMap<u32, glow::NativeFramebuffer>,
    buffers: std::collections::BTreeMap<u32, glow::NativeBuffer>,
    vertex_arrays: std::collections::BTreeMap<u32, glow::NativeVertexArray>,
    buffer_upload_cache:
        std::collections::BTreeMap<(u32, u32), DesktopNativeOpenGlBufferUploadCacheEntry>,
    vertex_attribute_enabled_cache: std::collections::BTreeSet<(u32, u32)>,
    vertex_attribute_layout_cache:
        std::collections::BTreeMap<(u32, u32), DesktopNativeOpenGlVertexAttributeLayoutCacheEntry>,
    shaders: std::collections::BTreeMap<u32, glow::NativeShader>,
    programs: std::collections::BTreeMap<u32, glow::NativeProgram>,
    program_shaders: std::collections::BTreeMap<u32, mindustry_core::mindustry::graphics::ShaderId>,
    shader_sources: std::collections::BTreeMap<u32, String>,
    uniform_locations: std::collections::BTreeMap<(u32, String), glow::NativeUniformLocation>,
    framebuffer_handle_cache: mindustry_desktop::DesktopGraphicsOpenGlBackendHandleCache,
    framebuffer_handle_allocator: mindustry_desktop::DesktopGraphicsOpenGlBackendHandleAllocator,
    shader_asset_root: std::path::PathBuf,
    shader_asset_root_source: &'static str,
    shader_asset_root_shaders_dir_exists: bool,
    shader_asset_root_fonts_dir_exists: bool,
    base_window_title: String,
    current_window_title_diagnostic: Option<String>,
    current_program: Option<u32>,
    current_vertex_array: Option<u32>,
    active_texture_unit: u32,
    bound_textures: std::collections::BTreeMap<(u32, u32), u32>,
    buffer_upload_cache_hits: usize,
    buffer_upload_cache_misses: usize,
    vertex_attribute_cache_hits: usize,
    vertex_attribute_cache_misses: usize,
    draw_state_cache_hits: usize,
    draw_state_cache_misses: usize,
    native_errors: Vec<String>,
}

#[cfg(feature = "opengl-native-runtime")]
const DESKTOP_NATIVE_WINDOW_ICON_SOURCE_PATH: &str = "icons/icon_64.png";

#[cfg(feature = "opengl-native-runtime")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopNativeOpenGlContextProfile {
    Core,
    Compatibility,
}

#[cfg(feature = "opengl-native-runtime")]
impl DesktopNativeOpenGlContextProfile {
    fn to_glutin(self) -> GlProfile {
        match self {
            Self::Core => GlProfile::Core,
            Self::Compatibility => GlProfile::Compatibility,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Compatibility => "compatibility",
        }
    }
}

#[cfg(feature = "opengl-native-runtime")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DesktopNativeOpenGlContextCandidate {
    version: Option<(u8, u8)>,
    profile: Option<DesktopNativeOpenGlContextProfile>,
}

#[cfg(feature = "opengl-native-runtime")]
impl DesktopNativeOpenGlContextCandidate {
    const fn versioned(major: u8, minor: u8, profile: DesktopNativeOpenGlContextProfile) -> Self {
        Self {
            version: Some((major, minor)),
            profile: Some(profile),
        }
    }

    const fn generic() -> Self {
        Self {
            version: None,
            profile: None,
        }
    }

    fn label(self) -> String {
        match (self.version, self.profile) {
            (Some((major, minor)), Some(profile)) => {
                format!("OpenGL {major}.{minor} {}", profile.label())
            }
            (Some((major, minor)), None) => format!("OpenGL {major}.{minor} default-profile"),
            (None, Some(profile)) => format!("OpenGL default-version {}", profile.label()),
            (None, None) => "OpenGL default-version default-profile".into(),
        }
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_default_context_candidates_for_platform(
    is_macos: bool,
    prefer_legacy: bool,
    high_version_profile: DesktopNativeOpenGlContextProfile,
) -> Vec<DesktopNativeOpenGlContextCandidate> {
    let mut candidates = Vec::new();
    if prefer_legacy {
        for (major, minor) in [(2, 1), (2, 0)] {
            candidates.push(DesktopNativeOpenGlContextCandidate::versioned(
                major,
                minor,
                DesktopNativeOpenGlContextProfile::Compatibility,
            ));
        }
        candidates.push(DesktopNativeOpenGlContextCandidate::generic());
        return candidates;
    }
    if is_macos {
        for (major, minor) in [(4, 1), (3, 2)] {
            candidates.push(DesktopNativeOpenGlContextCandidate::versioned(
                major,
                minor,
                high_version_profile,
            ));
        }
    } else {
        for (major, minor) in [(4, 6), (4, 5), (4, 4), (4, 1), (3, 3), (3, 2), (3, 1)] {
            candidates.push(DesktopNativeOpenGlContextCandidate::versioned(
                major,
                minor,
                high_version_profile,
            ));
        }
    }
    for (major, minor) in [(2, 1), (2, 0)] {
        candidates.push(DesktopNativeOpenGlContextCandidate::versioned(
            major,
            minor,
            DesktopNativeOpenGlContextProfile::Compatibility,
        ));
    }
    candidates.push(DesktopNativeOpenGlContextCandidate::generic());
    candidates
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_default_context_candidates(
    high_version_profile: DesktopNativeOpenGlContextProfile,
) -> Vec<DesktopNativeOpenGlContextCandidate> {
    desktop_native_opengl_default_context_candidates_for_platform(
        cfg!(target_os = "macos"),
        desktop_native_opengl_prefers_legacy_context(),
        high_version_profile,
    )
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_parse_bool_env_flag(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "legacy" => Some(true),
        "0" | "false" | "no" | "off" | "modern" => Some(false),
        _ => None,
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_prefers_legacy_context() -> bool {
    if let Ok(value) = std::env::var("MINDUSTRY_DESKTOP_LEGACY_GL") {
        if let Some(enabled) = desktop_native_parse_bool_env_flag(&value) {
            return enabled;
        }
    }
    false
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_parse_gl_version(value: &str) -> Option<(u8, u8)> {
    let (major, minor) = value.split_once('.')?;
    let major = major.parse::<u8>().ok()?;
    let minor = minor.parse::<u8>().ok()?;
    Some((major, minor))
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_context_candidates_from_args<I, S>(
    args: I,
) -> Vec<DesktopNativeOpenGlContextCandidate>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();
    let mut explicit_gl = None;
    let mut profile = DesktopNativeOpenGlContextProfile::Core;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-gl" | "--gl" => {
                if let Some(value) = args.get(index + 1) {
                    explicit_gl = desktop_native_parse_gl_version(value);
                    index += 1;
                }
            }
            "-coreGl" | "--coreGl" => {
                profile = DesktopNativeOpenGlContextProfile::Core;
            }
            "-compatibilityGl" | "--compatibilityGl" => {
                profile = DesktopNativeOpenGlContextProfile::Compatibility;
            }
            _ => {}
        }
        index += 1;
    }

    if let Some((major, minor)) = explicit_gl {
        return vec![
            DesktopNativeOpenGlContextCandidate::versioned(major, minor, profile),
            DesktopNativeOpenGlContextCandidate::generic(),
        ];
    }

    desktop_native_opengl_default_context_candidates(profile)
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_antialias_enabled_from_args<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .any(|arg| matches!(arg.as_ref(), "-antialias" | "--antialias"))
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_context_attributes_for_candidate(
    candidate: DesktopNativeOpenGlContextCandidate,
    raw_window_handle: RawWindowHandle,
) -> glutin::context::ContextAttributes {
    let mut builder = ContextAttributesBuilder::new();
    if let Some(profile) = candidate.profile {
        builder = builder.with_profile(profile.to_glutin());
    }
    if let Some((major, minor)) = candidate.version {
        builder = builder.with_context_api(ContextApi::OpenGl(Some(Version::new(major, minor))));
    }
    builder.build(Some(raw_window_handle))
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_window_icon_candidate_paths_from_roots<I, P>(roots: I) -> Vec<std::path::PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<std::path::Path>,
{
    roots
        .into_iter()
        .map(|root| root.as_ref().join(DESKTOP_NATIVE_WINDOW_ICON_SOURCE_PATH))
        .collect()
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_window_icon_candidate_paths() -> Vec<std::path::PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = std::env::var_os("MINDUSTRY_ASSET_ROOT") {
        roots.push(std::path::PathBuf::from(path));
    }
    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
        roots.push(current_dir.join("core").join("assets"));
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            roots.push(exe_dir.join("assets"));
            roots.push(exe_dir.join("core").join("assets"));
        }
    }
    roots.push(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .join("core")
            .join("assets"),
    );
    roots.push(std::path::PathBuf::from(
        "D:/MDT/mindustry-upstream-v157.4/core/assets",
    ));
    desktop_native_window_icon_candidate_paths_from_roots(roots)
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_window_icon_from_rgba(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<winit::window::Icon, String> {
    winit::window::Icon::from_rgba(pixels, width, height)
        .map_err(|error| format!("invalid window icon rgba data: {error}"))
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_window_icon_from_path(
    path: &std::path::Path,
) -> Result<winit::window::Icon, String> {
    let image = mindustry_core::mindustry::graphics::png_rgba8888_from_path(path)
        .map_err(|error| format!("failed to decode window icon {}: {error:?}", path.display()))?;
    desktop_native_window_icon_from_rgba(image.pixels, image.width, image.height)
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_window_icon_resolution() -> Option<std::path::PathBuf> {
    for path in desktop_native_window_icon_candidate_paths() {
        match desktop_native_window_icon_from_path(&path) {
            Ok(_) => return Some(path),
            Err(error) => {
                desktop_native_trace(format!(
                    "runtime.new: window_icon candidate skipped: {error}"
                ));
            }
        }
    }
    None
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_window_attributes_with_icon(
    attributes: winit::window::WindowAttributes,
) -> (winit::window::WindowAttributes, Option<std::path::PathBuf>) {
    let icon_path = desktop_native_window_icon_resolution();
    let attributes = if let Some(path) = icon_path.as_ref() {
        match desktop_native_window_icon_from_path(path) {
            Ok(icon) => {
                desktop_native_trace_summary(format!(
                    "runtime.new: window_icon path={}",
                    path.display()
                ));
                attributes.with_window_icon(Some(icon))
            }
            Err(error) => {
                desktop_native_trace(format!(
                    "runtime.new: window_icon decode unexpectedly failed after resolution: {error}"
                ));
                attributes
            }
        }
    } else {
        attributes
    };
    (attributes, icon_path)
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_startup_diagnostic(
    runtime: &DesktopNativeOpenGlRuntime,
    window_icon_path: Option<&std::path::Path>,
) -> Option<String> {
    let mut reasons = Vec::new();

    if !runtime.shader_asset_root_shaders_dir_exists {
        reasons.push(format!(
            "shader assets unavailable: {} (source={})",
            runtime.shader_asset_root.display(),
            runtime.shader_asset_root_source
        ));
    }

    if !runtime.shader_asset_root_fonts_dir_exists {
        reasons.push(format!(
            "font assets unavailable: {} (source={})",
            runtime.shader_asset_root.join("fonts").display(),
            runtime.shader_asset_root_source
        ));
    }

    if window_icon_path.is_none() {
        reasons.push(format!(
            "window icon unavailable: {}",
            DESKTOP_NATIVE_WINDOW_ICON_SOURCE_PATH
        ));
    }

    if reasons.is_empty() {
        None
    } else {
        Some(reasons.join("; "))
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_shader_asset_root_resolution(
) -> DesktopNativeOpenGlShaderAssetRootResolution {
    static RESOLUTION: std::sync::OnceLock<DesktopNativeOpenGlShaderAssetRootResolution> =
        std::sync::OnceLock::new();
    RESOLUTION.get_or_init(|| {
    if let Some(path) = std::env::var_os("MINDUSTRY_ASSET_ROOT") {
        let path = std::path::PathBuf::from(path);
        let shaders_dir_exists = path.join("shaders").is_dir();
        let fonts_dir_exists = path.join("fonts").is_dir();
        desktop_native_trace(format!(
            "shader_asset_root: environment override path={} shaders_dir_exists={shaders_dir_exists} fonts_dir_exists={fonts_dir_exists}",
            path.display()
        ));
        return DesktopNativeOpenGlShaderAssetRootResolution {
            path,
            source: "environment override",
            shaders_dir_exists,
            fonts_dir_exists,
        };
    }

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    let repo_assets = repo_root.join("core").join("assets");
    let reference_assets = std::path::PathBuf::from("D:/MDT/mindustry-upstream-v157.4/core/assets");

    let mut candidates = Vec::new();
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push((current_dir.join("assets"), "current-dir"));
        candidates.push((current_dir.join("core").join("assets"), "current-dir"));
        for ancestor in current_dir.ancestors().take(5) {
            desktop_native_push_shader_asset_root_candidates_near(
                &mut candidates,
                ancestor,
                "current-dir-near",
            );
        }
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            candidates.push((exe_dir.join("assets"), "current-exe"));
            candidates.push((exe_dir.join("core").join("assets"), "current-exe"));
            for ancestor in exe_dir.ancestors().take(6) {
                desktop_native_push_shader_asset_root_candidates_near(
                    &mut candidates,
                    ancestor,
                    "current-exe-near",
                );
            }
        }
    }
    candidates.push((repo_assets, "repository"));
    candidates.push((reference_assets, "reference"));
    desktop_native_push_shader_asset_root_candidates_near(
        &mut candidates,
        repo_root,
        "repository-near",
    );
    let candidates = desktop_native_dedup_shader_asset_root_candidates(candidates);

    desktop_native_opengl_shader_asset_root_resolution_from_candidates(
        candidates,
        std::path::PathBuf::from("core/assets"),
    )
    }).clone()
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_builtin_sprite_shader_source(
    shader: mindustry_core::mindustry::graphics::ShaderId,
    stage: mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage,
    _source_path: &str,
) -> Option<&'static str> {
    if shader != mindustry_core::mindustry::graphics::ShaderId::Mesh {
        return None;
    }

    match stage {
        mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Vertex => Some(
            r#"#version 150
in vec4 a_position;
in vec4 a_color;
in vec2 a_texCoord0;
in vec4 a_mix_color;

uniform mat4 u_projTrans;
uniform vec2 u_viewportInverse;

out vec4 v_color;
out vec2 v_texCoords;
out vec4 v_mix_color;

void main(){
    gl_Position = u_projTrans * a_position;
    v_color = a_color;
    v_texCoords = a_texCoord0;
    v_mix_color = a_mix_color;
}
"#,
        ),
        mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Fragment => Some(
            r#"#version 150
in vec4 v_color;
in vec2 v_texCoords;
in vec4 v_mix_color;

uniform sampler2D u_texture;

out vec4 fragColor;

void main(){
    vec4 sampled = texture(u_texture, v_texCoords);
    fragColor = v_color * mix(sampled, vec4(v_mix_color.rgb, sampled.a), v_mix_color.a);
}
"#,
        ),
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_bind_builtin_mesh_attributes(gl: &glow::Context, program: glow::NativeProgram) {
    unsafe {
        gl.bind_attrib_location(program, 0, "a_position");
        gl.bind_attrib_location(program, 1, "a_color");
        gl.bind_attrib_location(program, 2, "a_texCoord0");
        gl.bind_attrib_location(program, 3, "a_mix_color");
    }
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_pixel_projection_matrix(width: i32, height: i32) -> [f32; 16] {
    let width = width.max(1) as f32;
    let height = height.max(1) as f32;
    [
        2.0 / width,
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / height,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        -1.0,
        -1.0,
        0.0,
        1.0,
    ]
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_placeholder_rgba8888_pixels(width: i32, height: i32) -> Vec<u8> {
    let width = width.max(1).min(4096) as usize;
    let height = height.max(1).min(4096) as usize;
    let mut pixels = Vec::with_capacity(width.saturating_mul(height).saturating_mul(4));
    for y in 0..height {
        for x in 0..width {
            let bright = ((x / 8) + (y / 8)) % 2 == 0;
            if bright {
                pixels.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                pixels.extend_from_slice(&[255, 0, 255, 255]);
            }
        }
    }
    pixels
}

#[cfg(feature = "opengl-native-runtime")]
fn prepare_default_framebuffer_state<BindFramebuffer, SetViewport, DisableScissor>(
    width: i32,
    height: i32,
    mut bind_framebuffer: BindFramebuffer,
    mut set_viewport: SetViewport,
    mut disable_scissor: DisableScissor,
) where
    BindFramebuffer: FnMut(),
    SetViewport: FnMut(i32, i32),
    DisableScissor: FnMut(),
{
    bind_framebuffer();
    set_viewport(width, height);
    disable_scissor();
}

#[cfg(feature = "opengl-native-runtime")]
impl DesktopNativeOpenGlRuntime {
    fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        native_config: &mindustry_desktop::DesktopNativeOpenGlRuntimeConfig,
    ) -> Result<Self, String> {
        desktop_native_trace("runtime.new: begin");
        let template = ConfigTemplateBuilder::new();
        let antialias_enabled =
            desktop_native_antialias_enabled_from_args(std::env::args().collect::<Vec<_>>());
        let (window_attributes, startup_icon_path) =
            desktop_native_window_attributes_with_icon(native_config.window_attributes());
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(event_loop, template, |configs| {
                if antialias_enabled {
                    configs
                        .max_by_key(|config| config.num_samples())
                        .expect("no compatible OpenGL config was returned by glutin")
                } else {
                    configs
                        .min_by_key(|config| config.num_samples())
                        .expect("no compatible OpenGL config was returned by glutin")
                }
            })
            .map_err(|error| format!("failed to build native OpenGL display/window: {error}"))?;
        let window =
            window.ok_or_else(|| "glutin display builder did not return a window".to_string())?;
        desktop_native_trace_summary(format!(
            "runtime.new: selected_config_samples={} antialias_enabled={antialias_enabled}",
            gl_config.num_samples()
        ));
        desktop_native_trace("runtime.new: window created");
        let raw_window_handle = window
            .window_handle()
            .map_err(|error| format!("failed to read raw window handle: {error}"))?
            .as_raw();
        let gl_display = gl_config.display();
        let mut context_attempt_errors = Vec::new();
        let mut selected_context_candidate = None;
        let mut not_current_context = None;
        for candidate in
            desktop_native_opengl_context_candidates_from_args(std::env::args().collect::<Vec<_>>())
        {
            let label = candidate.label();
            let attributes =
                desktop_native_context_attributes_for_candidate(candidate, raw_window_handle);
            match unsafe { gl_display.create_context(&gl_config, &attributes) } {
                Ok(context) => {
                    desktop_native_trace_summary(format!("runtime.new: context selected={label}"));
                    selected_context_candidate = Some(label);
                    not_current_context = Some(context);
                    break;
                }
                Err(error) => {
                    desktop_native_trace(format!(
                        "runtime.new: context candidate failed {label}: {error}"
                    ));
                    context_attempt_errors.push(format!("{label}: {error}"));
                }
            }
        }
        let not_current_context = not_current_context.ok_or_else(|| {
            format!(
                "failed to create native OpenGL context after {} attempts: {}",
                context_attempt_errors.len(),
                context_attempt_errors.join(" | ")
            )
        })?;
        desktop_native_trace(format!(
            "runtime.new: context created via {}",
            selected_context_candidate
                .as_deref()
                .unwrap_or("unknown OpenGL candidate")
        ));
        let surface_attributes = window
            .build_surface_attributes(SurfaceAttributesBuilder::<WindowSurface>::new())
            .map_err(|error| {
                format!("failed to build native OpenGL surface attributes: {error}")
            })?;
        let surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
            .map_err(|error| format!("failed to create native OpenGL window surface: {error}"))?;
        let context = not_current_context
            .make_current(&surface)
            .map_err(|error| format!("failed to make native OpenGL context current: {error}"))?;
        desktop_native_trace("runtime.new: surface current");
        if native_config.vsync {
            let _ = surface.set_swap_interval(
                &context,
                SwapInterval::Wait(NonZeroU32::new(1).expect("non-zero vsync interval")),
            );
        }
        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|symbol| {
                gl_display.get_proc_address(symbol) as *const _
            })
        };
        if desktop_native_trace_summary_enabled() {
            let version = unsafe { gl.get_parameter_string(glow::VERSION) };
            let renderer = unsafe { gl.get_parameter_string(glow::RENDERER) };
            desktop_native_trace_summary(format!(
                "runtime.new: gl version={version} renderer={renderer}"
            ));
        }

        let shader_asset_root_resolution = desktop_native_opengl_shader_asset_root_resolution();
        let selected_context_label = selected_context_candidate
            .as_deref()
            .unwrap_or("unknown OpenGL candidate");
        let base_window_title = format!(
            "{} [native runtime: {}]",
            native_config.surface.title, selected_context_label
        );
        desktop_native_trace_summary(format!(
            "runtime.new: native runtime entered context={} window_title={} shader_asset_root={} source={} shaders_dir_exists={} fonts_dir_exists={} window_icon={}",
            selected_context_label,
            base_window_title,
            shader_asset_root_resolution.path.display(),
            shader_asset_root_resolution.source,
            shader_asset_root_resolution.shaders_dir_exists,
            shader_asset_root_resolution.fonts_dir_exists,
            startup_icon_path
                .as_ref()
                .map_or("missing".to_string(), |path| path.display().to_string())
        ));

        let mut runtime = Self {
            window,
            surface,
            context,
            gl,
            state: mindustry_desktop::DesktopGraphicsOpenGlBackendRuntimeState::default(),
            driver: mindustry_desktop::DesktopGraphicsRecordingOpenGlBackendDriver::default(),
            textures: std::collections::BTreeMap::new(),
            framebuffers: std::collections::BTreeMap::new(),
            buffers: std::collections::BTreeMap::new(),
            vertex_arrays: std::collections::BTreeMap::new(),
            buffer_upload_cache: std::collections::BTreeMap::new(),
            vertex_attribute_enabled_cache: std::collections::BTreeSet::new(),
            vertex_attribute_layout_cache: std::collections::BTreeMap::new(),
            shaders: std::collections::BTreeMap::new(),
            programs: std::collections::BTreeMap::new(),
            program_shaders: std::collections::BTreeMap::new(),
            shader_sources: std::collections::BTreeMap::new(),
            uniform_locations: std::collections::BTreeMap::new(),
            framebuffer_handle_cache:
                mindustry_desktop::DesktopGraphicsOpenGlBackendHandleCache::default(),
            framebuffer_handle_allocator:
                mindustry_desktop::DesktopGraphicsOpenGlBackendHandleAllocator::default(),
            shader_asset_root: shader_asset_root_resolution.path,
            shader_asset_root_source: shader_asset_root_resolution.source,
            shader_asset_root_shaders_dir_exists: shader_asset_root_resolution.shaders_dir_exists,
            shader_asset_root_fonts_dir_exists: shader_asset_root_resolution.fonts_dir_exists,
            base_window_title,
            current_window_title_diagnostic: None,
            current_program: None,
            current_vertex_array: None,
            active_texture_unit: glow::TEXTURE0,
            bound_textures: std::collections::BTreeMap::new(),
            buffer_upload_cache_hits: 0,
            buffer_upload_cache_misses: 0,
            vertex_attribute_cache_hits: 0,
            vertex_attribute_cache_misses: 0,
            draw_state_cache_hits: 0,
            draw_state_cache_misses: 0,
            native_errors: Vec::new(),
        };
        runtime.update_window_title_diagnostic(None);
        if let Some(diagnostic) =
            desktop_native_opengl_startup_diagnostic(&runtime, startup_icon_path.as_deref())
        {
            runtime.update_window_title_diagnostic(Some(diagnostic.clone()));
            desktop_native_trace_summary(format!("runtime.new: diagnosis={diagnostic}"));
        }
        runtime.resize_native_surface(runtime.window_surface_size());
        desktop_native_trace("runtime.new: resized initial surface");
        Ok(runtime)
    }

    fn window_surface_size(&self) -> mindustry_desktop::DesktopSurfaceSize {
        let size = self.window.inner_size();
        mindustry_desktop::DesktopSurfaceSize::new(size.width, size.height)
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn set_vsync(&self, enabled: bool) {
        let interval = if enabled {
            SwapInterval::Wait(NonZeroU32::new(1).expect("non-zero vsync interval"))
        } else {
            SwapInterval::DontWait
        };
        let _ = self.surface.set_swap_interval(&self.context, interval);
    }

    fn set_fullscreen(&self, enabled: bool) {
        self.window
            .set_fullscreen(enabled.then(|| winit::window::Fullscreen::Borderless(None)));
    }

    fn sync_surface_after_scale_factor_changed(&mut self) -> mindustry_desktop::DesktopSurfaceSize {
        let size = self.window_surface_size();
        self.resize_native_surface(size);
        self.request_redraw();
        size
    }

    fn update_window_title_diagnostic(&mut self, diagnostic: Option<String>) {
        if self.current_window_title_diagnostic == diagnostic {
            return;
        }

        let title = desktop_native_window_title_with_diagnostic(
            &self.base_window_title,
            diagnostic.as_deref(),
        );
        self.window.set_title(&title);
        self.current_window_title_diagnostic = diagnostic;
    }

    fn resize_native_surface(&mut self, size: mindustry_desktop::DesktopSurfaceSize) {
        if self.state.surface_size == Some(size) {
            return;
        }
        self.state.surface_size = Some(size);
        self.state.resize_events += 1;
        if let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        {
            self.surface.resize(&self.context, width, height);
            unsafe {
                self.gl.viewport(
                    0,
                    0,
                    size.width.min(i32::MAX as u32) as i32,
                    size.height.min(i32::MAX as u32) as i32,
                );
            }
        }
    }

    fn clear_backbuffer(&self) {
        let size = self.window_surface_size();
        let width = size.width.min(i32::MAX as u32) as i32;
        let height = size.height.min(i32::MAX as u32) as i32;
        prepare_default_framebuffer_state(
            width,
            height,
            || unsafe {
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            },
            |width, height| unsafe {
                self.gl.viewport(0, 0, width, height);
            },
            || unsafe {
                // The no-world menu path uses SetClip/ClearClip heavily for preview
                // rects. Reset native scissor before the next backbuffer clear so a
                // stale clip state can never turn the following frame into a black
                // screen even if a previous frame was interrupted mid-pass.
                self.gl.disable(glow::SCISSOR_TEST);
            },
        );
        unsafe {
            self.gl.use_program(None);
            self.gl.bind_vertex_array(None);
            self.gl.disable(glow::BLEND);
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.disable(glow::STENCIL_TEST);
            self.gl.clear_color(0.015, 0.018, 0.025, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    fn draw_visible_fallback_overlay(&self) {
        let size = self.window_surface_size();
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.use_program(None);
            self.gl.bind_vertex_array(None);
            self.gl.disable(glow::BLEND);
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.disable(glow::STENCIL_TEST);
            self.gl.enable(glow::SCISSOR_TEST);
            for rect in desktop_native_visible_fallback_rects(size) {
                self.gl.scissor(rect.x, rect.y, rect.width, rect.height);
                self.gl
                    .clear_color(rect.color[0], rect.color[1], rect.color[2], rect.color[3]);
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }
            self.gl.disable(glow::SCISSOR_TEST);
        }
    }
}

#[cfg(feature = "opengl-native-runtime")]
struct DesktopNativeOpenGlDriver<'a> {
    gl: &'a glow::Context,
    recording: &'a mut mindustry_desktop::DesktopGraphicsRecordingOpenGlBackendDriver,
    textures: &'a mut std::collections::BTreeMap<u32, glow::NativeTexture>,
    framebuffers: &'a mut std::collections::BTreeMap<u32, glow::NativeFramebuffer>,
    buffers: &'a mut std::collections::BTreeMap<u32, glow::NativeBuffer>,
    vertex_arrays: &'a mut std::collections::BTreeMap<u32, glow::NativeVertexArray>,
    buffer_upload_cache:
        &'a mut std::collections::BTreeMap<(u32, u32), DesktopNativeOpenGlBufferUploadCacheEntry>,
    vertex_attribute_enabled_cache: &'a mut std::collections::BTreeSet<(u32, u32)>,
    vertex_attribute_layout_cache: &'a mut std::collections::BTreeMap<
        (u32, u32),
        DesktopNativeOpenGlVertexAttributeLayoutCacheEntry,
    >,
    shaders: &'a mut std::collections::BTreeMap<u32, glow::NativeShader>,
    programs: &'a mut std::collections::BTreeMap<u32, glow::NativeProgram>,
    program_shaders:
        &'a mut std::collections::BTreeMap<u32, mindustry_core::mindustry::graphics::ShaderId>,
    shader_sources: &'a mut std::collections::BTreeMap<u32, String>,
    uniform_locations:
        &'a mut std::collections::BTreeMap<(u32, String), glow::NativeUniformLocation>,
    framebuffer_handle_cache: &'a mut mindustry_desktop::DesktopGraphicsOpenGlBackendHandleCache,
    framebuffer_handle_allocator:
        &'a mut mindustry_desktop::DesktopGraphicsOpenGlBackendHandleAllocator,
    surface_size: mindustry_desktop::DesktopSurfaceSize,
    shader_asset_root: &'a std::path::Path,
    shader_asset_root_source: &'static str,
    shader_asset_root_shaders_dir_exists: bool,
    current_program: &'a mut Option<u32>,
    current_vertex_array: &'a mut Option<u32>,
    active_texture_unit: &'a mut u32,
    bound_textures: &'a mut std::collections::BTreeMap<(u32, u32), u32>,
    bound_render_target: Option<mindustry_core::mindustry::graphics::RenderTarget>,
    bound_buffer_handles: std::collections::BTreeMap<u32, u32>,
    draw_target_available: bool,
    invalid_draw_commands: usize,
    buffer_upload_cache_hits: &'a mut usize,
    buffer_upload_cache_misses: &'a mut usize,
    vertex_attribute_cache_hits: &'a mut usize,
    vertex_attribute_cache_misses: &'a mut usize,
    draw_state_cache_hits: &'a mut usize,
    draw_state_cache_misses: &'a mut usize,
    native_errors: &'a mut Vec<String>,
}

#[cfg(feature = "opengl-native-runtime")]
impl DesktopNativeOpenGlDriver<'_> {
    fn shader_type(stage: mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage) -> u32 {
        match stage {
            mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Vertex => {
                glow::VERTEX_SHADER
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Fragment => {
                glow::FRAGMENT_SHADER
            }
        }
    }

    fn texture_for_handle(&mut self, texture_handle: u32) -> Option<glow::NativeTexture> {
        if let Some(texture) = self.textures.get(&texture_handle) {
            return Some(*texture);
        }
        let texture = unsafe { self.gl.create_texture() };
        match texture {
            Ok(texture) => {
                unsafe {
                    self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                    self.gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MIN_FILTER,
                        glow::NEAREST as i32,
                    );
                    self.gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MAG_FILTER,
                        glow::NEAREST as i32,
                    );
                    self.gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_S,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    self.gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_T,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    self.gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA as i32,
                        1,
                        1,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(Some(&[255, 255, 255, 255])),
                    );
                }
                self.textures.insert(texture_handle, texture);
                Some(texture)
            }
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to create native OpenGL texture for logical handle {texture_handle}: {error}"
                ));
                None
            }
        }
    }

    fn delete_texture_handle(&mut self, texture_handle: u32) {
        if let Some(texture) = self.textures.remove(&texture_handle) {
            unsafe {
                self.gl.delete_texture(texture);
            }
        }
        self.bound_textures
            .retain(|_, bound_texture_handle| *bound_texture_handle != texture_handle);
    }

    fn framebuffer_for_handle(
        &mut self,
        framebuffer_handle: u32,
    ) -> Option<glow::NativeFramebuffer> {
        if let Some(framebuffer) = self.framebuffers.get(&framebuffer_handle) {
            return Some(*framebuffer);
        }
        let framebuffer = unsafe { self.gl.create_framebuffer() };
        match framebuffer {
            Ok(framebuffer) => {
                self.framebuffers.insert(framebuffer_handle, framebuffer);
                Some(framebuffer)
            }
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to create native OpenGL framebuffer for logical handle {framebuffer_handle}: {error}"
                ));
                None
            }
        }
    }

    fn framebuffer_key_for_render_target(
        target: &mindustry_core::mindustry::graphics::RenderTarget,
    ) -> Option<String> {
        match target {
            mindustry_core::mindustry::graphics::RenderTarget::Screen => None,
            mindustry_core::mindustry::graphics::RenderTarget::Texture(name) => {
                Some(format!("framebuffer:texture:{name}"))
            }
            mindustry_core::mindustry::graphics::RenderTarget::Buffer(name) => {
                Some(format!("framebuffer:buffer:{name}"))
            }
        }
    }

    fn framebuffer_attachment_key_for_render_target(
        target: &mindustry_core::mindustry::graphics::RenderTarget,
    ) -> Option<String> {
        match target {
            mindustry_core::mindustry::graphics::RenderTarget::Screen => None,
            mindustry_core::mindustry::graphics::RenderTarget::Texture(name) => {
                Some(format!("framebuffer-attachment:texture:{name}:color0"))
            }
            mindustry_core::mindustry::graphics::RenderTarget::Buffer(name) => {
                Some(format!("framebuffer-attachment:buffer:{name}:color0"))
            }
        }
    }

    fn native_framebuffer_for_bound_render_target(
        &mut self,
        target: Option<&mindustry_core::mindustry::graphics::RenderTarget>,
    ) -> Result<Option<glow::NativeFramebuffer>, ()> {
        let Some(target) = target else {
            return Ok(None);
        };
        let Some(framebuffer_key) = Self::framebuffer_key_for_render_target(target) else {
            return Ok(None);
        };
        if let Some(logical_handle) = self
            .framebuffer_handle_cache
            .framebuffers
            .get(&framebuffer_key)
            .copied()
        {
            return self
                .framebuffer_for_handle(logical_handle)
                .map(Some)
                .ok_or(());
        }

        let (width, height) = self.viewport_for_render_target(Some(target));
        let width = u32::try_from(width.max(1)).unwrap_or(1);
        let height = u32::try_from(height.max(1)).unwrap_or(1);
        self.native_errors.push(format!(
            "native OpenGL framebuffer for target {:?} was missing; lazily creating {}x{} attachment instead of binding the default framebuffer",
            target, width, height
        ));
        self.native_framebuffer_for_render_target(target, width, height)
            .map(Some)
            .ok_or(())
    }

    fn viewport_for_render_target(
        &self,
        target: Option<&mindustry_core::mindustry::graphics::RenderTarget>,
    ) -> (i32, i32) {
        let fallback_width = self.surface_size.width.min(i32::MAX as u32) as i32;
        let fallback_height = self.surface_size.height.min(i32::MAX as u32) as i32;
        let Some(target) = target else {
            return (fallback_width, fallback_height);
        };
        let Some(attachment_key) = Self::framebuffer_attachment_key_for_render_target(target)
        else {
            return (fallback_width, fallback_height);
        };
        let Some(attachment) = self
            .framebuffer_handle_cache
            .framebuffer_attachments
            .get(&attachment_key)
        else {
            return (fallback_width, fallback_height);
        };
        (
            attachment.width.min(i32::MAX as u32) as i32,
            attachment.height.min(i32::MAX as u32) as i32,
        )
    }

    fn bind_framebuffer_target(
        &mut self,
        target: Option<&mindustry_core::mindustry::graphics::RenderTarget>,
    ) -> bool {
        let Ok(framebuffer) = self.native_framebuffer_for_bound_render_target(target) else {
            *self.current_program = None;
            *self.current_vertex_array = None;
            return false;
        };
        let (width, height) = self.viewport_for_render_target(target);
        if framebuffer.is_none() {
            // Returning to the screen/default framebuffer must also drop any stale
            // clip state so an offscreen pass cannot crop the next visible frame.
            prepare_default_framebuffer_state(
                width,
                height,
                || unsafe {
                    self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                },
                |width, height| unsafe {
                    self.gl.viewport(0, 0, width, height);
                },
                || unsafe {
                    self.gl.disable(glow::SCISSOR_TEST);
                },
            );
            unsafe {
                self.gl.use_program(None);
                self.gl.bind_vertex_array(None);
                self.gl.disable(glow::BLEND);
                self.gl.disable(glow::DEPTH_TEST);
                self.gl.disable(glow::STENCIL_TEST);
            }
            *self.current_program = None;
            *self.current_vertex_array = None;
        } else {
            unsafe {
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer);
                self.gl.viewport(0, 0, width, height);
            }
        }
        true
    }

    fn native_blend_factor(factor: mindustry_core::mindustry::graphics::RenderBlendFactor) -> u32 {
        match factor {
            mindustry_core::mindustry::graphics::RenderBlendFactor::Zero => glow::ZERO,
            mindustry_core::mindustry::graphics::RenderBlendFactor::One => glow::ONE,
            mindustry_core::mindustry::graphics::RenderBlendFactor::SrcColor => glow::SRC_COLOR,
            mindustry_core::mindustry::graphics::RenderBlendFactor::OneMinusSrcColor => {
                glow::ONE_MINUS_SRC_COLOR
            }
            mindustry_core::mindustry::graphics::RenderBlendFactor::DstColor => glow::DST_COLOR,
            mindustry_core::mindustry::graphics::RenderBlendFactor::OneMinusDstColor => {
                glow::ONE_MINUS_DST_COLOR
            }
            mindustry_core::mindustry::graphics::RenderBlendFactor::SrcAlpha => glow::SRC_ALPHA,
            mindustry_core::mindustry::graphics::RenderBlendFactor::OneMinusSrcAlpha => {
                glow::ONE_MINUS_SRC_ALPHA
            }
            mindustry_core::mindustry::graphics::RenderBlendFactor::DstAlpha => glow::DST_ALPHA,
            mindustry_core::mindustry::graphics::RenderBlendFactor::OneMinusDstAlpha => {
                glow::ONE_MINUS_DST_ALPHA
            }
        }
    }

    fn consume_native_blend_state(
        &mut self,
        state: &mindustry_desktop::DesktopGraphicsOpenGlBackendBlendState,
    ) {
        unsafe {
            if state.enabled {
                self.gl.enable(glow::BLEND);
                if let (Some(source), Some(destination)) =
                    (state.source_factor, state.destination_factor)
                {
                    self.gl.blend_func(
                        Self::native_blend_factor(source),
                        Self::native_blend_factor(destination),
                    );
                }
            } else {
                self.gl.disable(glow::BLEND);
            }
        }
    }

    fn native_scissor_rect_for_target(
        &self,
        target: Option<&mindustry_core::mindustry::graphics::RenderTarget>,
        rect: mindustry_core::mindustry::graphics::RenderRect,
    ) -> (i32, i32, i32, i32) {
        let (target_width, target_height) = self.viewport_for_render_target(target);
        let left = rect.x.floor().max(0.0).min(target_width as f32) as i32;
        let bottom = rect.y.floor().max(0.0).min(target_height as f32) as i32;
        let right = rect.right().ceil().max(0.0).min(target_width as f32) as i32;
        let top = rect.bottom().ceil().max(0.0).min(target_height as f32) as i32;
        (left, bottom, (right - left).max(0), (top - bottom).max(0))
    }

    fn delete_framebuffer_handle(&mut self, framebuffer_handle: u32) {
        if let Some(framebuffer) = self.framebuffers.remove(&framebuffer_handle) {
            unsafe {
                self.gl.delete_framebuffer(framebuffer);
            }
        }
    }

    fn buffer_for_handle(&mut self, buffer_handle: u32) -> Option<glow::NativeBuffer> {
        if let Some(buffer) = self.buffers.get(&buffer_handle) {
            return Some(*buffer);
        }
        let buffer = unsafe { self.gl.create_buffer() };
        match buffer {
            Ok(buffer) => {
                self.buffers.insert(buffer_handle, buffer);
                Some(buffer)
            }
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to create native OpenGL buffer for logical handle {buffer_handle}: {error}"
                ));
                None
            }
        }
    }

    fn vertex_array_for_handle(
        &mut self,
        vertex_array_handle: u32,
    ) -> Option<glow::NativeVertexArray> {
        if let Some(vertex_array) = self.vertex_arrays.get(&vertex_array_handle) {
            return Some(*vertex_array);
        }
        let vertex_array = unsafe { self.gl.create_vertex_array() };
        match vertex_array {
            Ok(vertex_array) => {
                self.vertex_arrays.insert(vertex_array_handle, vertex_array);
                Some(vertex_array)
            }
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to create native OpenGL vertex array for logical handle {vertex_array_handle}: {error}"
                ));
                None
            }
        }
    }

    fn consume_native_bound_buffer_data(&mut self, target: u32, usage: u32, bytes: &[u8]) {
        let Some(buffer_handle) = self.bound_buffer_handles.get(&target).copied() else {
            unsafe {
                self.gl.buffer_data_u8_slice(target, bytes, usage);
            }
            return;
        };
        let cache_key = (target, buffer_handle);
        if self
            .buffer_upload_cache
            .get(&cache_key)
            .is_some_and(|entry| entry.usage == usage && entry.bytes.as_slice() == bytes)
        {
            *self.buffer_upload_cache_hits += 1;
            return;
        }

        unsafe {
            self.gl.buffer_data_u8_slice(target, bytes, usage);
        }
        self.buffer_upload_cache.insert(
            cache_key,
            DesktopNativeOpenGlBufferUploadCacheEntry {
                usage,
                bytes: bytes.to_vec(),
            },
        );
        *self.buffer_upload_cache_misses += 1;
    }

    fn consume_native_enable_vertex_attribute_array(&mut self, attribute_location: i32) {
        let (Some(vertex_array_handle), Ok(attribute_location)) = (
            *self.current_vertex_array,
            u32::try_from(attribute_location),
        ) else {
            return;
        };
        let cache_key = (vertex_array_handle, attribute_location);
        if self.vertex_attribute_enabled_cache.contains(&cache_key) {
            *self.vertex_attribute_cache_hits += 1;
            return;
        }
        unsafe {
            self.gl.enable_vertex_attrib_array(attribute_location);
        }
        self.vertex_attribute_enabled_cache.insert(cache_key);
        *self.vertex_attribute_cache_misses += 1;
    }

    fn consume_native_vertex_attribute_pointer(
        &mut self,
        attribute_location: i32,
        components: usize,
        gl_type: u32,
        normalized: bool,
        stride_bytes: usize,
        offset_bytes: usize,
    ) {
        let (
            Some(vertex_array_handle),
            Ok(attribute_location),
            Ok(components),
            Ok(stride_bytes),
            Ok(offset_bytes),
        ) = (
            *self.current_vertex_array,
            u32::try_from(attribute_location),
            i32::try_from(components),
            i32::try_from(stride_bytes),
            i32::try_from(offset_bytes),
        )
        else {
            return;
        };
        let cache_key = (vertex_array_handle, attribute_location);
        let cache_entry = DesktopNativeOpenGlVertexAttributeLayoutCacheEntry {
            components,
            gl_type,
            normalized,
            stride_bytes,
            offset_bytes,
        };
        if self
            .vertex_attribute_layout_cache
            .get(&cache_key)
            .is_some_and(|entry| *entry == cache_entry)
        {
            *self.vertex_attribute_cache_hits += 1;
            return;
        }
        unsafe {
            self.gl.vertex_attrib_pointer_f32(
                attribute_location,
                components,
                gl_type,
                normalized,
                stride_bytes,
                offset_bytes,
            );
        }
        self.vertex_attribute_layout_cache
            .insert(cache_key, cache_entry);
        *self.vertex_attribute_cache_misses += 1;
    }

    fn consume_native_use_program(&mut self, program_handle: u32) {
        if *self.current_program == Some(program_handle) {
            *self.draw_state_cache_hits += 1;
            return;
        }
        let Some(program) = self.existing_program(program_handle) else {
            return;
        };
        unsafe {
            self.gl.use_program(Some(program));
        }
        *self.current_program = Some(program_handle);
        self.upload_builtin_sprite_uniforms_for_program(program_handle, program);
        *self.draw_state_cache_misses += 1;
    }

    fn consume_native_active_texture(&mut self, texture_unit: u32) {
        if *self.active_texture_unit == texture_unit {
            *self.draw_state_cache_hits += 1;
            return;
        }
        unsafe {
            self.gl.active_texture(texture_unit);
        }
        *self.active_texture_unit = texture_unit;
        *self.draw_state_cache_misses += 1;
    }

    fn consume_native_bind_texture(&mut self, target: u32, texture_handle: u32) {
        let cache_key = (*self.active_texture_unit, target);
        if self
            .bound_textures
            .get(&cache_key)
            .is_some_and(|bound_texture_handle| *bound_texture_handle == texture_handle)
        {
            *self.draw_state_cache_hits += 1;
            return;
        }
        let Some(texture) = self.texture_for_handle(texture_handle) else {
            return;
        };
        unsafe {
            self.gl.bind_texture(target, Some(texture));
        }
        self.bound_textures.insert(cache_key, texture_handle);
        *self.draw_state_cache_misses += 1;
    }

    fn consume_native_bind_vertex_array_for_draw(&mut self, vertex_array_handle: u32) {
        if *self.current_vertex_array == Some(vertex_array_handle) {
            *self.draw_state_cache_hits += 1;
            return;
        }
        let Some(vertex_array) = self.vertex_array_for_handle(vertex_array_handle) else {
            return;
        };
        unsafe {
            self.gl.bind_vertex_array(Some(vertex_array));
        }
        *self.current_vertex_array = Some(vertex_array_handle);
        *self.draw_state_cache_misses += 1;
    }

    fn existing_program(&mut self, program_handle: u32) -> Option<glow::NativeProgram> {
        if let Some(program) = self.programs.get(&program_handle).copied() {
            return Some(program);
        }
        self.create_builtin_mesh_program_for_handle(program_handle)
            .or_else(|| {
                self.native_errors.push(format!(
                    "native OpenGL program for logical handle {program_handle} is not initialized"
                ));
                None
            })
    }

    fn create_builtin_mesh_program_for_handle(
        &mut self,
        program_handle: u32,
    ) -> Option<glow::NativeProgram> {
        let vertex_source = desktop_native_opengl_builtin_sprite_shader_source(
            mindustry_core::mindustry::graphics::ShaderId::Mesh,
            mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Vertex,
            "builtin-mesh.vert",
        )?;
        let fragment_source = desktop_native_opengl_builtin_sprite_shader_source(
            mindustry_core::mindustry::graphics::ShaderId::Mesh,
            mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Fragment,
            "builtin-mesh.frag",
        )?;

        unsafe {
            let vertex_shader = match self.gl.create_shader(glow::VERTEX_SHADER) {
                Ok(shader) => shader,
                Err(error) => {
                    self.native_errors.push(format!(
                        "failed to create builtin Mesh vertex shader for logical program {program_handle}: {error}"
                    ));
                    return None;
                }
            };
            self.gl.shader_source(vertex_shader, vertex_source);
            self.gl.compile_shader(vertex_shader);
            if !self.gl.get_shader_compile_status(vertex_shader) {
                self.native_errors.push(format!(
                    "builtin Mesh vertex shader compile failed for logical program {program_handle}: {}",
                    self.gl.get_shader_info_log(vertex_shader)
                ));
                self.gl.delete_shader(vertex_shader);
                return None;
            }

            let fragment_shader = match self.gl.create_shader(glow::FRAGMENT_SHADER) {
                Ok(shader) => shader,
                Err(error) => {
                    self.native_errors.push(format!(
                        "failed to create builtin Mesh fragment shader for logical program {program_handle}: {error}"
                    ));
                    self.gl.delete_shader(vertex_shader);
                    return None;
                }
            };
            self.gl.shader_source(fragment_shader, fragment_source);
            self.gl.compile_shader(fragment_shader);
            if !self.gl.get_shader_compile_status(fragment_shader) {
                self.native_errors.push(format!(
                    "builtin Mesh fragment shader compile failed for logical program {program_handle}: {}",
                    self.gl.get_shader_info_log(fragment_shader)
                ));
                self.gl.delete_shader(vertex_shader);
                self.gl.delete_shader(fragment_shader);
                return None;
            }

            let program = match self.gl.create_program() {
                Ok(program) => program,
                Err(error) => {
                    self.native_errors.push(format!(
                        "failed to create builtin Mesh program for logical handle {program_handle}: {error}"
                    ));
                    self.gl.delete_shader(vertex_shader);
                    self.gl.delete_shader(fragment_shader);
                    return None;
                }
            };
            self.gl.attach_shader(program, vertex_shader);
            self.gl.attach_shader(program, fragment_shader);
            desktop_native_bind_builtin_mesh_attributes(self.gl, program);
            self.gl.link_program(program);
            self.gl.detach_shader(program, vertex_shader);
            self.gl.detach_shader(program, fragment_shader);
            self.gl.delete_shader(vertex_shader);
            self.gl.delete_shader(fragment_shader);

            if !self.gl.get_program_link_status(program) {
                self.native_errors.push(format!(
                    "builtin Mesh program link failed for logical handle {program_handle}: {}",
                    self.gl.get_program_info_log(program)
                ));
                self.gl.delete_program(program);
                return None;
            }

            self.programs.insert(program_handle, program);
            self.program_shaders.insert(
                program_handle,
                mindustry_core::mindustry::graphics::ShaderId::Mesh,
            );
            Some(program)
        }
    }

    fn existing_shader(&mut self, shader_handle: u32) -> Option<glow::NativeShader> {
        self.shaders.get(&shader_handle).copied().or_else(|| {
            self.native_errors.push(format!(
                "native OpenGL shader for logical handle {shader_handle} is not initialized"
            ));
            None
        })
    }

    fn delete_program_handle(&mut self, program_handle: u32) {
        if let Some(program) = self.programs.remove(&program_handle) {
            unsafe {
                self.gl.delete_program(program);
            }
        }
        self.program_shaders.remove(&program_handle);
        self.uniform_locations
            .retain(|(handle, _), _| *handle != program_handle);
    }

    fn delete_shader_handle(&mut self, shader_handle: u32) {
        if let Some(shader) = self.shaders.remove(&shader_handle) {
            unsafe {
                self.gl.delete_shader(shader);
            }
        }
        self.shader_sources.remove(&shader_handle);
    }

    fn create_shader_for_handle(
        &mut self,
        shader_handle: u32,
        stage: mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage,
    ) {
        self.delete_shader_handle(shader_handle);
        match unsafe { self.gl.create_shader(Self::shader_type(stage)) } {
            Ok(shader) => {
                self.shaders.insert(shader_handle, shader);
            }
            Err(error) => self.native_errors.push(format!(
                "failed to create native OpenGL {stage:?} shader for logical handle {shader_handle}: {error}"
            )),
        }
    }

    fn create_program_for_handle(&mut self, program_handle: u32) {
        self.delete_program_handle(program_handle);
        match unsafe { self.gl.create_program() } {
            Ok(program) => {
                self.programs.insert(program_handle, program);
            }
            Err(error) => self.native_errors.push(format!(
                "failed to create native OpenGL program for logical handle {program_handle}: {error}"
            )),
        }
    }

    fn load_preprocessed_shader_source(
        &mut self,
        shader: mindustry_core::mindustry::graphics::ShaderId,
        stage: mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage,
        source_path: &str,
    ) -> Option<String> {
        if let Some(source) =
            desktop_native_opengl_builtin_sprite_shader_source(shader, stage, source_path)
        {
            return Some(source.to_string());
        }

        desktop_native_trace(format!(
            "shader.load: root={} source={} shaders_dir_exists={} shader={shader:?} stage={stage:?} source_path={source_path}",
            self.shader_asset_root.display(),
            self.shader_asset_root_source,
            self.shader_asset_root_shaders_dir_exists
        ));

        let loader = mindustry_desktop::DesktopGraphicsOpenGlBackendShaderSourceLoader::new(
            self.shader_asset_root,
        );
        let source = loader.load_stage_source(shader, stage, source_path.to_string());
        let source = match source {
            Ok(source) => source,
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to load native OpenGL shader source {source_path} from {} (source={}, shaders_dir_exists={}): {error:?}",
                    self.shader_asset_root.display(),
                    self.shader_asset_root_source,
                    self.shader_asset_root_shaders_dir_exists
                ));
                return None;
            }
        };
        let options = mindustry_desktop::DesktopGraphicsOpenGlBackendShaderPreprocessOptions {
            gl30: true,
            desktop: true,
            mobile: false,
            gl_version_at_least_3_2: true,
            ..Default::default()
        };
        match source.preprocess(&options) {
            Ok(source) => Some(source.source_text),
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to preprocess native OpenGL shader source {source_path}: {error:?}"
                ));
                None
            }
        }
    }

    fn uniform_location(
        &mut self,
        program_handle: u32,
        program: glow::NativeProgram,
        uniform: &'static str,
    ) -> Option<glow::NativeUniformLocation> {
        let key = (program_handle, uniform.to_string());
        if let Some(location) = self.uniform_locations.get(&key) {
            return Some(*location);
        }
        let location = unsafe { self.gl.get_uniform_location(program, uniform) };
        match location {
            Some(location) => {
                self.uniform_locations.insert(key, location);
                Some(location)
            }
            None => {
                self.native_errors.push(format!(
                    "native OpenGL uniform {uniform} was not found for logical program handle {program_handle}"
                ));
                None
            }
        }
    }

    fn upload_builtin_sprite_uniforms_for_program(
        &mut self,
        program_handle: u32,
        program: glow::NativeProgram,
    ) {
        if self.program_shaders.get(&program_handle).copied()
            != Some(mindustry_core::mindustry::graphics::ShaderId::Mesh)
        {
            return;
        }

        let (width, height) = self.viewport_for_render_target(self.bound_render_target.as_ref());
        let Some(location) = self.uniform_location(program_handle, program, "u_projTrans") else {
            return;
        };
        let projection = desktop_native_opengl_pixel_projection_matrix(width, height);
        unsafe {
            self.gl
                .uniform_matrix_4_f32_slice(Some(&location), false, &projection);
        }
        if let Some(texture_location) = self.uniform_location(program_handle, program, "u_texture")
        {
            unsafe {
                self.gl.uniform_1_i32(Some(&texture_location), 0);
            }
        }
    }

    fn upload_builtin_sprite_uniforms_for_current_program(&mut self) {
        let Some(program_handle) = *self.current_program else {
            return;
        };
        if let Some(program) = self.existing_program(program_handle) {
            self.upload_builtin_sprite_uniforms_for_program(program_handle, program);
        }
    }

    fn pixel_data_from_source(
        &mut self,
        pixel_source: &mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadPixelSource,
        fallback_width: i32,
        fallback_height: i32,
    ) -> Vec<u8> {
        match pixel_source.load_rgba8888_pixels() {
            Ok(pixels) => pixels.pixels,
            Err(error) => {
                desktop_native_trace_summary(format!(
                    "failed to load native OpenGL texture pixel source; using checker fallback: {error:?}"
                ));
                desktop_native_placeholder_rgba8888_pixels(fallback_width, fallback_height)
            }
        }
    }

    fn consume_native_resolved_framebuffer_attachment(
        &mut self,
        attachment: &mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedFramebufferAttachment,
    ) {
        if let Some(previous_framebuffer_handle) = attachment.previous_framebuffer_handle {
            self.delete_framebuffer_handle(previous_framebuffer_handle);
        }
        if let Some(previous_color_texture_handle) = attachment.previous_color_texture_handle {
            self.delete_texture_handle(previous_color_texture_handle);
        }

        let texture_needs_storage = attachment.color_texture_was_recreated
            || !self.textures.contains_key(&attachment.color_texture_handle);
        let Some(framebuffer) = self.framebuffer_for_handle(attachment.framebuffer_handle) else {
            return;
        };
        let Some(texture) = self.texture_for_handle(attachment.color_texture_handle) else {
            return;
        };
        let (Ok(width), Ok(height)) = (
            i32::try_from(attachment.width),
            i32::try_from(attachment.height),
        ) else {
            self.native_errors.push(format!(
                "native OpenGL framebuffer attachment {} has invalid size {}x{}",
                attachment.framebuffer_key, attachment.width, attachment.height
            ));
            return;
        };

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            if texture_needs_storage {
                self.gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    width,
                    height,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(None),
                );
            }
            self.gl
                .bind_framebuffer(attachment.framebuffer_target, Some(framebuffer));
            self.gl.framebuffer_texture_2d(
                attachment.framebuffer_target,
                attachment.color_attachment,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );
            let status = self
                .gl
                .check_framebuffer_status(attachment.framebuffer_target);
            if status != glow::FRAMEBUFFER_COMPLETE {
                self.native_errors.push(format!(
                    "native OpenGL framebuffer {} is incomplete: status=0x{status:04x}",
                    attachment.framebuffer_key
                ));
            }
            self.gl
                .bind_framebuffer(attachment.framebuffer_target, None);
        }
    }

    fn consume_native_framebuffer_attachment_plan(
        &mut self,
        plan: &mindustry_desktop::DesktopGraphicsOpenGlBackendFramebufferAttachmentPlan,
    ) {
        let attachment = self
            .framebuffer_handle_cache
            .resolve_framebuffer_attachment(plan, self.framebuffer_handle_allocator);
        self.consume_native_resolved_framebuffer_attachment(&attachment);
    }

    fn framebuffer_attachment_plan_for_render_target(
        target: &mindustry_core::mindustry::graphics::RenderTarget,
        width: u32,
        height: u32,
    ) -> Option<mindustry_desktop::DesktopGraphicsOpenGlBackendFramebufferAttachmentPlan> {
        match target {
            mindustry_core::mindustry::graphics::RenderTarget::Screen => None,
            mindustry_core::mindustry::graphics::RenderTarget::Texture(name) => Some(
                mindustry_desktop::DesktopGraphicsOpenGlBackendFramebufferAttachmentPlan::from_buffer_name_with_size(
                    format!("texture:{name}"),
                    width,
                    height,
                    0,
                ),
            ),
            mindustry_core::mindustry::graphics::RenderTarget::Buffer(name) => Some(
                mindustry_desktop::DesktopGraphicsOpenGlBackendFramebufferAttachmentPlan::from_buffer_name_with_size(
                    format!("buffer:{name}"),
                    width,
                    height,
                    0,
                ),
            ),
        }
    }

    fn native_framebuffer_for_attachment(
        &mut self,
        attachment: &mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedFramebufferAttachment,
    ) -> Option<glow::NativeFramebuffer> {
        self.consume_native_resolved_framebuffer_attachment(attachment);
        self.framebuffers
            .get(&attachment.framebuffer_handle)
            .copied()
    }

    fn native_framebuffer_for_render_target(
        &mut self,
        target: &mindustry_core::mindustry::graphics::RenderTarget,
        width: u32,
        height: u32,
    ) -> Option<glow::NativeFramebuffer> {
        let plan = Self::framebuffer_attachment_plan_for_render_target(target, width, height)?;
        let attachment = self
            .framebuffer_handle_cache
            .resolve_framebuffer_attachment(&plan, self.framebuffer_handle_allocator);
        self.native_framebuffer_for_attachment(&attachment)
    }

    fn consume_native_resolve_command(
        &mut self,
        command: &mindustry_desktop::DesktopGraphicsOpenGlBackendResolveCommand,
    ) {
        if command.resolve_kind != mindustry_core::mindustry::graphics::RenderResolveKind::Blit {
            return;
        }
        let Some(source_attachment) = command.source_attachment.as_ref() else {
            self.native_errors.push(format!(
                "native OpenGL Blit resolve from {:?} has no framebuffer attachment",
                command.source_target
            ));
            return;
        };
        let Some(source_framebuffer) = self.native_framebuffer_for_attachment(source_attachment)
        else {
            return;
        };
        let (Ok(width), Ok(height)) = (
            i32::try_from(source_attachment.width),
            i32::try_from(source_attachment.height),
        ) else {
            self.native_errors.push(format!(
                "native OpenGL Blit resolve source {:?} has invalid size {}x{}",
                command.source_target, source_attachment.width, source_attachment.height
            ));
            return;
        };
        let target_framebuffer = self.native_framebuffer_for_render_target(
            &command.resolve_target,
            source_attachment.width,
            source_attachment.height,
        );

        unsafe {
            self.gl
                .bind_framebuffer(glow::READ_FRAMEBUFFER, Some(source_framebuffer));
            self.gl
                .bind_framebuffer(glow::DRAW_FRAMEBUFFER, target_framebuffer);
            self.gl.blit_framebuffer(
                0,
                0,
                width,
                height,
                0,
                0,
                width,
                height,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );
            self.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
        }
    }

    fn consume_native_resolved_shader_lifecycle_command(
        &mut self,
        command: &mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand,
    ) {
        match command {
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::DeleteProgram {
                program_handle,
                ..
            } => {
                if let Some(program_handle) = program_handle {
                    self.delete_program_handle(*program_handle);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::CreateShader {
                stage,
                shader_handle,
                previous_shader_handle,
                ..
            } => {
                if let Some(previous_shader_handle) = previous_shader_handle {
                    self.delete_shader_handle(*previous_shader_handle);
                }
                self.create_shader_for_handle(*shader_handle, *stage);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::ShaderSource {
                shader,
                stage,
                shader_handle,
                source_path,
                ..
            } => {
                if let Some(native_shader) = self.existing_shader(*shader_handle) {
                    if let Some(source) =
                        self.load_preprocessed_shader_source(*shader, *stage, source_path)
                    {
                        unsafe {
                            self.gl.shader_source(native_shader, &source);
                        }
                        self.shader_sources.insert(*shader_handle, source);
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::CompileShader {
                stage,
                shader_handle,
                ..
            } => {
                if let Some(native_shader) = self.existing_shader(*shader_handle) {
                    unsafe {
                        self.gl.compile_shader(native_shader);
                        if !self.gl.get_shader_compile_status(native_shader) {
                            self.native_errors.push(format!(
                                "native OpenGL {stage:?} shader compile failed for logical handle {shader_handle}: {}",
                                self.gl.get_shader_info_log(native_shader)
                            ));
                        }
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::CreateProgram {
                shader,
                program_handle,
                previous_program_handle,
                ..
            } => {
                if let Some(previous_program_handle) = previous_program_handle {
                    self.delete_program_handle(*previous_program_handle);
                }
                self.create_program_for_handle(*program_handle);
                self.program_shaders.insert(*program_handle, *shader);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::AttachShader {
                program_handle,
                shader_handle,
                ..
            } => {
                if let (Some(program), Some(shader)) = (
                    self.existing_program(*program_handle),
                    self.existing_shader(*shader_handle),
                ) {
                    unsafe {
                        self.gl.attach_shader(program, shader);
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::LinkProgram {
                program_handle,
                ..
            } => {
                if let Some(program) = self.existing_program(*program_handle) {
                    unsafe {
                        if self.program_shaders.get(program_handle).copied()
                            == Some(mindustry_core::mindustry::graphics::ShaderId::Mesh)
                        {
                            desktop_native_bind_builtin_mesh_attributes(self.gl, program);
                        }
                        self.gl.link_program(program);
                        if !self.gl.get_program_link_status(program) {
                            self.native_errors.push(format!(
                                "native OpenGL program link failed for logical handle {program_handle}: {}",
                                self.gl.get_program_info_log(program)
                            ));
                        }
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand::DeleteShader {
                shader_handle,
                ..
            } => {
                if let Some(shader_handle) = shader_handle {
                    self.delete_shader_handle(*shader_handle);
                }
            }
        }
    }

    fn consume_native_resolved_shader_command(
        &mut self,
        command: &mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderCommand,
    ) {
        match command {
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderCommand::UseProgram {
                program_handle,
                ..
            } => {
                if let Some(program) = self.existing_program(*program_handle) {
                    unsafe {
                        self.gl.use_program(Some(program));
                    }
                    *self.current_program = Some(*program_handle);
                    self.upload_builtin_sprite_uniforms_for_program(*program_handle, program);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderCommand::UploadUniform {
                program_handle,
                uniform,
                value,
                ..
            } => {
                if let Some(program) = self.existing_program(*program_handle) {
                    if let Some(location) = self.uniform_location(*program_handle, program, uniform)
                    {
                        self.upload_uniform_value(&location, uniform, value);
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderCommand::ActiveTexture {
                texture_unit,
                ..
            } => unsafe {
                self.gl.active_texture(*texture_unit);
            },
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderCommand::BindTexture {
                target,
                texture_handle,
                ..
            } => {
                if let Some(texture) = self.texture_for_handle(*texture_handle) {
                    unsafe {
                        self.gl.bind_texture(*target, Some(texture));
                    }
                    self.bound_textures
                        .insert((*self.active_texture_unit, *target), *texture_handle);
                }
            }
        }
    }

    fn upload_uniform_value(
        &mut self,
        location: &glow::NativeUniformLocation,
        uniform: &'static str,
        value: &mindustry_core::mindustry::graphics::UniformValue,
    ) {
        match value {
            mindustry_core::mindustry::graphics::UniformValue::Int(value) => unsafe {
                self.gl.uniform_1_i32(Some(location), *value);
            },
            mindustry_core::mindustry::graphics::UniformValue::Float(value) => unsafe {
                self.gl.uniform_1_f32(Some(location), *value);
            },
            mindustry_core::mindustry::graphics::UniformValue::Vec2(value) => unsafe {
                self.gl.uniform_2_f32(Some(location), value[0], value[1]);
            },
            mindustry_core::mindustry::graphics::UniformValue::Vec3(value) => unsafe {
                self.gl
                    .uniform_3_f32(Some(location), value[0], value[1], value[2]);
            },
            mindustry_core::mindustry::graphics::UniformValue::Vec4(value) => unsafe {
                self.gl
                    .uniform_4_f32(Some(location), value[0], value[1], value[2], value[3]);
            },
            mindustry_core::mindustry::graphics::UniformValue::Vec4Array(values) => {
                let flattened = values.iter().flatten().copied().collect::<Vec<_>>();
                unsafe {
                    self.gl.uniform_4_f32_slice(Some(location), &flattened);
                }
            }
            mindustry_core::mindustry::graphics::UniformValue::Mat4(symbolic_matrix) => {
                self.native_errors.push(format!(
                    "native OpenGL uniform {uniform} uses unresolved symbolic Mat4 {symbolic_matrix}"
                ));
            }
        }
    }

    fn consume_native_texture_upload_command(
        &mut self,
        command: &mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand,
    ) {
        match command {
            mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand::DeleteTexture {
                texture_handle,
            } => {
                self.delete_texture_handle(*texture_handle);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand::BindTexture {
                target,
                texture_handle,
            } => {
                if let Some(texture) = self.texture_for_handle(*texture_handle) {
                    unsafe {
                        self.gl.bind_texture(*target, Some(texture));
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand::SetTextureParameter {
                target,
                pname,
                param,
            } => unsafe {
                self.gl.tex_parameter_i32(*target, *pname, *param);
            },
            mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand::TexImage2D {
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                pixel_type,
                pixel_source,
            } => {
                let pixels = self.pixel_data_from_source(pixel_source, *width, *height);
                unsafe {
                    self.gl.tex_image_2d(
                        *target,
                        *level,
                        *internal_format,
                        *width,
                        *height,
                        *border,
                        *format,
                        *pixel_type,
                        glow::PixelUnpackData::Slice(Some(&pixels)),
                    );
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand::TexSubImage2DFromSource {
                target,
                level,
                xoffset,
                yoffset,
                width,
                height,
                format,
                pixel_type,
                pixel_source,
            } => {
                let pixels = self.pixel_data_from_source(pixel_source, *width, *height);
                unsafe {
                    self.gl.tex_sub_image_2d(
                        *target,
                        *level,
                        *xoffset,
                        *yoffset,
                        *width,
                        *height,
                        *format,
                        *pixel_type,
                        glow::PixelUnpackData::Slice(Some(&pixels)),
                    );
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand::TexSubImage2D {
                target,
                level,
                xoffset,
                yoffset,
                width,
                height,
                format,
                pixel_type,
                pixels,
            } => unsafe {
                self.gl.tex_sub_image_2d(
                    *target,
                    *level,
                    *xoffset,
                    *yoffset,
                    *width,
                    *height,
                    *format,
                    *pixel_type,
                    glow::PixelUnpackData::Slice(Some(pixels)),
                );
            },
        }
    }

    fn consume_native_sprite_mesh_upload_command(
        &mut self,
        command: &mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand,
    ) {
        match command {
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::BindVertexArray {
                vertex_array_handle,
            } => {
                if let Some(vertex_array) = self.vertex_array_for_handle(*vertex_array_handle) {
                    unsafe {
                        self.gl.bind_vertex_array(Some(vertex_array));
                    }
                    *self.current_vertex_array = Some(*vertex_array_handle);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::BindBuffer {
                target,
                buffer_handle,
            } => {
                if let Some(buffer) = self.buffer_for_handle(*buffer_handle) {
                    unsafe {
                        self.gl.bind_buffer(*target, Some(buffer));
                    }
                    self.bound_buffer_handles.insert(*target, *buffer_handle);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::BufferData {
                target,
                usage,
                bytes,
            } => self.consume_native_bound_buffer_data(*target, *usage, bytes),
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::EnableVertexAttributeArray {
                attribute_location,
            } => self.consume_native_enable_vertex_attribute_array(*attribute_location),
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::VertexAttributePointer {
                attribute_location,
                components,
                gl_type,
                normalized,
                stride_bytes,
                offset_bytes,
            } => self.consume_native_vertex_attribute_pointer(
                *attribute_location,
                *components,
                *gl_type,
                *normalized,
                *stride_bytes,
                *offset_bytes,
            ),
        }
    }

    fn consume_native_resolve_mesh_upload_command(
        &mut self,
        command: &mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand,
    ) {
        match command {
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::BindVertexArray {
                vertex_array_handle,
            } => {
                if let Some(vertex_array) = self.vertex_array_for_handle(*vertex_array_handle) {
                    unsafe {
                        self.gl.bind_vertex_array(Some(vertex_array));
                    }
                    *self.current_vertex_array = Some(*vertex_array_handle);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::BindBuffer {
                target,
                buffer_handle,
            } => {
                if let Some(buffer) = self.buffer_for_handle(*buffer_handle) {
                    unsafe {
                        self.gl.bind_buffer(*target, Some(buffer));
                    }
                    self.bound_buffer_handles.insert(*target, *buffer_handle);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::BufferData {
                target,
                usage,
                bytes,
            } => self.consume_native_bound_buffer_data(*target, *usage, bytes),
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::EnableVertexAttributeArray {
                attribute_location,
            } => self.consume_native_enable_vertex_attribute_array(*attribute_location),
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::VertexAttributePointer {
                attribute_location,
                components,
                gl_type,
                normalized,
                stride_bytes,
                offset_bytes,
            } => self.consume_native_vertex_attribute_pointer(
                *attribute_location,
                *components,
                *gl_type,
                *normalized,
                *stride_bytes,
                *offset_bytes,
            ),
        }
    }

    fn consume_native_draw_command(
        &mut self,
        command: &mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand,
    ) {
        match command {
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::BindFramebuffer {
                target,
            } => {
                self.bound_render_target = target.clone();
                self.draw_target_available = self.bind_framebuffer_target(target.as_ref());
                if self.draw_target_available {
                    self.upload_builtin_sprite_uniforms_for_current_program();
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::SetViewport { target } => {
                if !self.draw_target_available {
                    return;
                }
                let (width, height) = self.viewport_for_render_target(target.as_ref());
                unsafe {
                    self.gl.viewport(0, 0, width, height);
                }
                self.bound_render_target = target.clone();
                self.upload_builtin_sprite_uniforms_for_current_program();
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::Clear { color } => {
                if !self.draw_target_available {
                    return;
                }
                unsafe {
                    self.gl.clear_color(color[0], color[1], color[2], color[3]);
                    self.gl.clear(glow::COLOR_BUFFER_BIT);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::SetBlend { state } => {
                if !self.draw_target_available {
                    return;
                }
                self.consume_native_blend_state(state);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::SetScissor {
                target,
                rect,
            } => {
                if !self.draw_target_available {
                    return;
                }
                let (x, y, width, height) =
                    self.native_scissor_rect_for_target(target.as_ref(), *rect);
                unsafe {
                    self.gl.enable(glow::SCISSOR_TEST);
                    self.gl.scissor(x, y, width, height);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::ClearScissor { .. } => {
                if !self.draw_target_available {
                    return;
                }
                unsafe {
                    self.gl.disable(glow::SCISSOR_TEST);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::UseProgram {
                program_handle,
            } => {
                if !self.draw_target_available {
                    return;
                }
                self.consume_native_use_program(*program_handle);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::ActiveTexture {
                texture_unit,
            } => {
                if !self.draw_target_available {
                    return;
                }
                self.consume_native_active_texture(*texture_unit);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::BindTexture {
                target,
                texture_handle,
            } => {
                if !self.draw_target_available {
                    return;
                }
                self.consume_native_bind_texture(*target, *texture_handle);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::BindVertexArray {
                vertex_array_handle,
            } => {
                if !self.draw_target_available {
                    return;
                }
                self.consume_native_bind_vertex_array_for_draw(*vertex_array_handle);
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::DrawElements {
                primitive_type,
                index_count,
                index_type,
                index_offset_bytes,
            } => {
                if !self.draw_target_available {
                    self.invalid_draw_commands += 1;
                    return;
                }
                let has_program = self.current_program.is_some();
                let has_vertex_array = self.current_vertex_array.is_some();
                if let (Ok(index_count), Ok(index_offset_bytes)) = (
                    i32::try_from(*index_count),
                    i32::try_from(*index_offset_bytes),
                ) {
                    if index_count > 0 && has_program && has_vertex_array {
                        unsafe {
                            self.gl.draw_elements(
                                *primitive_type,
                                index_count,
                                *index_type,
                                index_offset_bytes,
                            );
                        }
                    } else {
                        self.invalid_draw_commands += 1;
                    }
                } else {
                    self.invalid_draw_commands += 1;
                }
            }
        }
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommandSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_texture_upload_command(
        &mut self,
        command: mindustry_desktop::DesktopGraphicsOpenGlBackendTextureUploadCommand,
    ) {
        self.consume_native_texture_upload_command(&command);
        self.recording
            .consume_opengl_texture_upload_command(command);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommandSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_sprite_mesh_upload_command(
        &mut self,
        command: mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand,
    ) {
        self.consume_native_sprite_mesh_upload_command(&command);
        self.recording
            .consume_opengl_sprite_mesh_upload_command(command);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommandSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_resolve_mesh_upload_command(
        &mut self,
        command: mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand,
    ) {
        self.consume_native_resolve_mesh_upload_command(&command);
        self.recording
            .consume_opengl_resolve_mesh_upload_command(command);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderCommandSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_resolved_shader_command(
        &mut self,
        command: mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderCommand,
    ) {
        self.consume_native_resolved_shader_command(&command);
        self.recording
            .consume_opengl_resolved_shader_command(command);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommandSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_resolved_shader_lifecycle_command(
        &mut self,
        command: mindustry_desktop::DesktopGraphicsOpenGlBackendResolvedShaderLifecycleCommand,
    ) {
        self.consume_native_resolved_shader_lifecycle_command(&command);
        self.recording
            .consume_opengl_resolved_shader_lifecycle_command(command);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendFramebufferAttachmentSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_framebuffer_attachment(
        &mut self,
        plan: mindustry_desktop::DesktopGraphicsOpenGlBackendFramebufferAttachmentPlan,
    ) {
        self.consume_native_framebuffer_attachment_plan(&plan);
        self.recording.consume_opengl_framebuffer_attachment(plan);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommandSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_draw_command(
        &mut self,
        command: mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand,
    ) {
        self.consume_native_draw_command(&command);
        self.recording.consume_opengl_draw_command(command);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendResolveCommandSink
    for DesktopNativeOpenGlDriver<'_>
{
    fn consume_opengl_resolve_command(
        &mut self,
        command: mindustry_desktop::DesktopGraphicsOpenGlBackendResolveCommand,
    ) {
        self.consume_native_resolve_command(&command);
        self.recording.consume_opengl_resolve_command(command);
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl mindustry_desktop::DesktopGraphicsOpenGlBackendRuntime for DesktopNativeOpenGlRuntime {
    fn resize_surface(&mut self, size: mindustry_desktop::DesktopSurfaceSize) {
        self.resize_native_surface(size);
    }

    fn submit_resolving_executor(
        &mut self,
        executor: &mindustry_desktop::DesktopGraphicsResolvingOpenGlBackendCommandExecutor,
    ) -> mindustry_desktop::DesktopGraphicsOpenGlBackendDriverExecutionState {
        desktop_native_trace("runtime.submit: clear backbuffer");
        self.clear_backbuffer();
        self.current_program = None;
        self.current_vertex_array = None;
        let surface_size = self
            .state
            .surface_size
            .unwrap_or_else(|| self.window_surface_size());
        let native_errors_start = self.native_errors.len();
        let buffer_upload_cache_hits_start = self.buffer_upload_cache_hits;
        let buffer_upload_cache_misses_start = self.buffer_upload_cache_misses;
        let vertex_attribute_cache_hits_start = self.vertex_attribute_cache_hits;
        let vertex_attribute_cache_misses_start = self.vertex_attribute_cache_misses;
        let draw_state_cache_hits_start = self.draw_state_cache_hits;
        let draw_state_cache_misses_start = self.draw_state_cache_misses;
        let (driver_state, invalid_draw_commands) = {
            let mut driver = DesktopNativeOpenGlDriver {
                gl: &self.gl,
                recording: &mut self.driver,
                textures: &mut self.textures,
                framebuffers: &mut self.framebuffers,
                buffers: &mut self.buffers,
                vertex_arrays: &mut self.vertex_arrays,
                buffer_upload_cache: &mut self.buffer_upload_cache,
                vertex_attribute_enabled_cache: &mut self.vertex_attribute_enabled_cache,
                vertex_attribute_layout_cache: &mut self.vertex_attribute_layout_cache,
                shaders: &mut self.shaders,
                programs: &mut self.programs,
                program_shaders: &mut self.program_shaders,
                shader_sources: &mut self.shader_sources,
                uniform_locations: &mut self.uniform_locations,
                framebuffer_handle_cache: &mut self.framebuffer_handle_cache,
                framebuffer_handle_allocator: &mut self.framebuffer_handle_allocator,
                surface_size,
                shader_asset_root: &self.shader_asset_root,
                shader_asset_root_source: self.shader_asset_root_source,
                shader_asset_root_shaders_dir_exists: self.shader_asset_root_shaders_dir_exists,
                current_program: &mut self.current_program,
                current_vertex_array: &mut self.current_vertex_array,
                active_texture_unit: &mut self.active_texture_unit,
                bound_textures: &mut self.bound_textures,
                bound_render_target: None,
                bound_buffer_handles: std::collections::BTreeMap::new(),
                draw_target_available: true,
                invalid_draw_commands: 0,
                buffer_upload_cache_hits: &mut self.buffer_upload_cache_hits,
                buffer_upload_cache_misses: &mut self.buffer_upload_cache_misses,
                vertex_attribute_cache_hits: &mut self.vertex_attribute_cache_hits,
                vertex_attribute_cache_misses: &mut self.vertex_attribute_cache_misses,
                draw_state_cache_hits: &mut self.draw_state_cache_hits,
                draw_state_cache_misses: &mut self.draw_state_cache_misses,
                native_errors: &mut self.native_errors,
            };
            desktop_native_trace("runtime.submit: drive native OpenGL driver");
            let driver_state = executor.drive_driver(&mut driver);
            (driver_state, driver.invalid_draw_commands)
        };
        let frame_native_errors = &self.native_errors[native_errors_start..];
        let frame_buffer_upload_cache_hits =
            self.buffer_upload_cache_hits - buffer_upload_cache_hits_start;
        let frame_buffer_upload_cache_misses =
            self.buffer_upload_cache_misses - buffer_upload_cache_misses_start;
        let frame_vertex_attribute_cache_hits =
            self.vertex_attribute_cache_hits - vertex_attribute_cache_hits_start;
        let frame_vertex_attribute_cache_misses =
            self.vertex_attribute_cache_misses - vertex_attribute_cache_misses_start;
        let frame_draw_state_cache_hits = self.draw_state_cache_hits - draw_state_cache_hits_start;
        let frame_draw_state_cache_misses =
            self.draw_state_cache_misses - draw_state_cache_misses_start;
        if desktop_native_trace_summary_enabled() {
            let gl_error = unsafe { self.gl.get_error() };
            desktop_native_trace_summary(format!(
                "runtime.submit: driver done framebuffer_attachments={} texture_upload_commands={} sprite_mesh_upload_commands={} resolve_mesh_upload_commands={} shader_commands={} draw_commands={} resolve_draw_commands={} resolve_commands={} invalid_draw_commands={} buffer_upload_cache_hits={} buffer_upload_cache_misses={} vertex_attribute_cache_hits={} vertex_attribute_cache_misses={} draw_state_cache_hits={} draw_state_cache_misses={} textures={} vaos={} buffers={} programs={} gl_error=0x{gl_error:04x}",
                driver_state.framebuffer_attachment_plans,
                driver_state.texture_upload_commands,
                driver_state.sprite_mesh_upload_commands,
                driver_state.resolve_mesh_upload_commands,
                driver_state.shader_commands,
                driver_state.draw_commands,
                driver_state.resolve_draw_commands,
                driver_state.resolve_commands,
                invalid_draw_commands,
                frame_buffer_upload_cache_hits,
                frame_buffer_upload_cache_misses,
                frame_vertex_attribute_cache_hits,
                frame_vertex_attribute_cache_misses,
                frame_draw_state_cache_hits,
                frame_draw_state_cache_misses,
                self.textures.len(),
                self.vertex_arrays.len(),
                self.buffers.len(),
                self.programs.len()
            ));
            if !frame_native_errors.is_empty() {
                for error in frame_native_errors.iter().rev().take(5).rev() {
                    desktop_native_trace_summary(format!(
                        "runtime.submit: native warning: {error}"
                    ));
                }
            }
        }
        let needs_visible_fallback = desktop_native_opengl_submit_needs_visible_fallback(
            &driver_state,
            invalid_draw_commands,
            frame_native_errors,
            self.shader_asset_root_shaders_dir_exists,
            self.shader_asset_root_fonts_dir_exists,
        );
        if needs_visible_fallback {
            desktop_native_trace_summary("runtime.submit: drawing native visible fallback overlay");
            self.draw_visible_fallback_overlay();
        }
        let diagnostic = desktop_native_opengl_submit_diagnostic(
            &driver_state,
            invalid_draw_commands,
            frame_native_errors,
            self.shader_asset_root_shaders_dir_exists,
            self.shader_asset_root_fonts_dir_exists,
        );
        let diagnostic = match diagnostic {
            Some(diagnostic) if needs_visible_fallback => {
                Some(format!("{diagnostic}; fallback overlay enabled"))
            }
            other => other,
        };
        if let Some(diagnostic) = diagnostic {
            desktop_native_trace_summary(format!("runtime.submit: diagnosis={diagnostic}"));
            self.update_window_title_diagnostic(Some(diagnostic));
        } else {
            self.update_window_title_diagnostic(None);
        }
        self.state.frames_submitted += 1;
        self.state.last_driver_state = Some(driver_state);
        driver_state
    }

    fn present_frame(&mut self) {
        desktop_native_trace("runtime.present: swap begin");
        self.surface
            .swap_buffers(&self.context)
            .expect("native OpenGL swap_buffers failed");
        desktop_native_trace("runtime.present: swap done");
        self.state.present_events += 1;
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl<'a> DesktopNativeOpenGlApp<'a> {
    fn new(
        launcher: &'a mut mindustry_desktop::DesktopLauncher,
        mut native_config: mindustry_desktop::DesktopNativeOpenGlRuntimeConfig,
    ) -> Self {
        if launcher
            .setting_override_value("graphics", "vsync")
            .is_some()
        {
            native_config.vsync = launcher.graphics_vsync_enabled;
        } else {
            launcher.graphics_vsync_enabled = native_config.vsync;
        }
        if launcher
            .setting_override_value("graphics", "fullscreen")
            .is_some()
        {
            native_config.fullscreen = launcher.graphics_fullscreen_enabled;
        } else {
            launcher.graphics_fullscreen_enabled = native_config.fullscreen;
        }
        let frame_loop = mindustry_desktop::DesktopFrameLoopState::new(
            native_config.surface.clone(),
            desktop_native_opengl_frame_pacing(launcher, &native_config),
        );
        Self {
            launcher,
            native_config,
            window_id: None,
            frame_loop,
            next_redraw_at: std::time::Instant::now(),
            graphics_renderer: None,
            runtime_init_error: None,
            effect_renderer: mindustry_desktop::HeadlessDesktopEffectRenderer::default(),
            pending_events: Vec::new(),
        }
    }

    fn sync_graphics_settings_to_native_runtime(&mut self) {
        let settings_vsync = self.launcher.graphics_vsync_enabled;
        if self.native_config.vsync != settings_vsync {
            self.native_config.vsync = settings_vsync;
            if let Some(renderer) = self.graphics_renderer.as_ref() {
                renderer.runtime.set_vsync(settings_vsync);
            }
            self.frame_loop.pacing =
                desktop_native_opengl_frame_pacing(self.launcher, &self.native_config);
            self.next_redraw_at = std::time::Instant::now();
        }

        let settings_fullscreen = self.launcher.graphics_fullscreen_enabled;
        if self.native_config.fullscreen != settings_fullscreen {
            self.native_config.fullscreen = settings_fullscreen;
            if let Some(renderer) = self.graphics_renderer.as_ref() {
                renderer.runtime.set_fullscreen(settings_fullscreen);
                renderer.runtime.request_redraw();
            }
        }
    }

    fn drain_present_frame(&mut self) -> Option<mindustry_desktop::DesktopPresentResult> {
        if self.pending_events.is_empty() {
            self.pending_events
                .push(mindustry_desktop::DesktopFrameLoopEvent::Tick);
        }
        let events = std::mem::take(&mut self.pending_events);
        let Some(graphics_renderer) = self.graphics_renderer.as_mut() else {
            desktop_native_trace_summary(
                "app.frame: redraw requested before native OpenGL renderer was ready; deferring",
            );
            self.pending_events = events;
            return None;
        };
        if desktop_native_trace_enabled() {
            desktop_native_trace(format!(
                "app.frame: begin index={} events={}",
                self.frame_loop.next_frame_index,
                events.len()
            ));
        }
        let result = self.launcher.step_desktop_frame_loop(
            &mut self.frame_loop,
            &events,
            graphics_renderer,
            &mut self.effect_renderer,
        );
        self.sync_graphics_settings_to_native_runtime();
        if desktop_native_trace_enabled() {
            desktop_native_trace(format!(
                "app.frame: done index={} presented={}",
                result.frame_index, result.presented
            ));
        }
        Some(result)
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl winit::application::ApplicationHandler for DesktopNativeOpenGlApp<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.graphics_renderer.is_some() {
            return;
        }

        desktop_native_trace("app.resumed: begin");
        let runtime = match DesktopNativeOpenGlRuntime::new(event_loop, &self.native_config) {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("failed to initialize native OpenGL desktop runtime: {error}");
                desktop_native_trace_summary(format!("app.resumed: runtime init failed: {error}"));
                self.runtime_init_error = Some(error);
                event_loop.exit();
                return;
            }
        };
        let window_id = runtime.window.id();
        let size = runtime.window_surface_size();
        self.pending_events
            .push(mindustry_desktop::DesktopFrameLoopEvent::Resize(size));
        self.window_id = Some(window_id);
        self.graphics_renderer =
            Some(mindustry_desktop::DesktopOpenGlBackendGraphicsRenderer::new(runtime));
        if let Some(renderer) = self.graphics_renderer.as_ref() {
            renderer.runtime.request_redraw();
        }
        desktop_native_trace("app.resumed: renderer ready");
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.window_id != Some(window_id) {
            return;
        }

        if let winit::event::WindowEvent::Resized(size) = &event {
            if let Some(renderer) = self.graphics_renderer.as_mut() {
                renderer
                    .runtime
                    .resize_native_surface(mindustry_desktop::DesktopSurfaceSize::new(
                        size.width,
                        size.height,
                    ));
            }
        }

        let scale_factor_surface_size =
            if matches!(&event, winit::event::WindowEvent::ScaleFactorChanged { .. }) {
                self.graphics_renderer
                    .as_mut()
                    .map(|renderer| renderer.runtime.sync_surface_after_scale_factor_changed())
            } else {
                None
            };

        let should_present = matches!(&event, winit::event::WindowEvent::RedrawRequested);
        self.pending_events
            .extend(mindustry_desktop::desktop_frame_loop_events_from_winit_window_event(&event));
        if let Some(size) = scale_factor_surface_size {
            self.pending_events
                .push(mindustry_desktop::DesktopFrameLoopEvent::Resize(size));
        }

        if should_present {
            if let Some(result) = self.drain_present_frame() {
                if result.should_stop() {
                    event_loop.exit();
                }
            }
        }

        if matches!(&event, winit::event::WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self.frame_loop.closed {
            let now = std::time::Instant::now();
            if self.frame_loop.pacing.is_paced() && now < self.next_redraw_at {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    self.next_redraw_at,
                ));
                return;
            }
            if let Some(renderer) = self.graphics_renderer.as_ref() {
                renderer.runtime.request_redraw();
            }
            if self.frame_loop.pacing.is_paced() {
                self.next_redraw_at = now + self.frame_loop.pacing.target_frame_time;
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    self.next_redraw_at,
                ));
            } else {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
            }
        }
    }
}

#[cfg(all(test, feature = "opengl-native-runtime"))]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DefaultFramebufferStateOp {
        BindDefaultFramebuffer,
        SetViewport(i32, i32),
        DisableScissorTest,
    }

    fn unique_temp_shader_asset_root(label: &str) -> std::path::PathBuf {
        let unique = format!(
            "mindustry-desktop-shader-root-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after UNIX_EPOCH")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(root.join("shaders")).expect("should create temporary shader root");
        std::fs::create_dir_all(root.join("fonts")).expect("should create temporary font root");
        root
    }

    #[test]
    fn native_opengl_app_initializes_frame_loop_from_native_surface_config() {
        let mut launcher = mindustry_desktop::DesktopLauncher::new(Vec::new());
        let native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_surface(
            mindustry_desktop::DesktopSurfaceConfig {
                title: "Native Test".into(),
                size: mindustry_desktop::DesktopSurfaceSize::new(960, 540),
                scale_factor: 1.0,
                resizable: true,
                maximized: false,
                visible: false,
            },
        );

        let app = DesktopNativeOpenGlApp::new(&mut launcher, native_config.clone());

        assert_eq!(app.frame_loop.surface, native_config.surface);
        assert_eq!(app.frame_loop.next_frame_index, 0);
        assert_eq!(
            app.frame_loop.pacing,
            mindustry_desktop::DesktopFramePacing::uncapped(),
            "vsync already paces swap_buffers; adding the 16ms frame cap on top can halve native FPS"
        );
        assert!(app.window_id.is_none());
        assert!(app.graphics_renderer.is_none());
        assert!(app.runtime_init_error.is_none());
        assert!(app.pending_events.is_empty());
    }

    #[test]
    fn native_opengl_app_keeps_frame_cap_when_vsync_disabled() {
        let mut launcher = mindustry_desktop::DesktopLauncher::new(Vec::new());
        let mut native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_surface(
            mindustry_desktop::DesktopSurfaceConfig {
                title: "Native Test No VSync".into(),
                size: mindustry_desktop::DesktopSurfaceSize::new(960, 540),
                scale_factor: 1.0,
                resizable: true,
                maximized: false,
                visible: false,
            },
        );
        native_config.vsync = false;

        let app = DesktopNativeOpenGlApp::new(&mut launcher, native_config);

        assert_eq!(
            app.frame_loop.pacing,
            mindustry_desktop::DesktopFramePacing::from_java_fps_cap(
                mindustry_desktop::DESKTOP_JAVA_DEFAULT_RUNTIME_FPS_CAP
            ),
            "without vsync the software frame cap should follow ClientLauncher fpscap default instead of the old 16ms/60fps fallback"
        );
    }

    #[test]
    fn native_opengl_app_uses_java_fpscap_when_vsync_disabled() {
        let mut launcher = mindustry_desktop::DesktopLauncher::new(Vec::new());
        let mut native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_surface(
            mindustry_desktop::DesktopSurfaceConfig {
                title: "Native Test FPSCap".into(),
                size: mindustry_desktop::DesktopSurfaceSize::new(960, 540),
                scale_factor: 1.0,
                resizable: true,
                maximized: false,
                visible: false,
            },
        );
        native_config.vsync = false;
        native_config.fps_cap = 30;

        let app = DesktopNativeOpenGlApp::new(&mut launcher, native_config);

        assert_eq!(
            app.frame_loop.pacing,
            mindustry_desktop::DesktopFramePacing::from_java_fps_cap(30)
        );

        launcher.set_setting_override("graphics", "fpscap", "245");
        let mut native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_surface(
            mindustry_desktop::DesktopSurfaceConfig {
                title: "Native Test FPSCap None".into(),
                size: mindustry_desktop::DesktopSurfaceSize::new(960, 540),
                scale_factor: 1.0,
                resizable: true,
                maximized: false,
                visible: false,
            },
        );
        native_config.vsync = false;
        let app = DesktopNativeOpenGlApp::new(&mut launcher, native_config);

        assert_eq!(
            app.frame_loop.pacing,
            mindustry_desktop::DesktopFramePacing::uncapped(),
            "Java treats fpscap > 240, including the UI sentinel 245, as uncapped"
        );
    }

    #[test]
    fn native_opengl_app_initializes_window_flags_from_settings_overrides_like_java() {
        let mut launcher = mindustry_desktop::DesktopLauncher::new(Vec::new());
        launcher.set_setting_override("graphics", "vsync", "false");
        launcher.set_setting_override("graphics", "fullscreen", "true");
        let native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_surface(
            mindustry_desktop::DesktopSurfaceConfig {
                title: "Native Test Settings".into(),
                size: mindustry_desktop::DesktopSurfaceSize::new(960, 540),
                scale_factor: 1.0,
                resizable: true,
                maximized: false,
                visible: false,
            },
        );

        let app = DesktopNativeOpenGlApp::new(&mut launcher, native_config);

        assert!(!app.native_config.vsync);
        assert!(app.native_config.fullscreen);
        assert_eq!(
            app.frame_loop.pacing,
            mindustry_desktop::DesktopFramePacing::from_java_fps_cap(
                mindustry_desktop::DESKTOP_JAVA_DEFAULT_RUNTIME_FPS_CAP
            ),
            "Java SettingsMenuDialog.setVSync callback should drive runtime frame pacing when vsync is off"
        );
    }

    #[test]
    fn native_opengl_app_keeps_cli_window_flags_when_settings_are_default() {
        let mut launcher = mindustry_desktop::DesktopLauncher::new(Vec::new());
        let mut native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_surface(
            mindustry_desktop::DesktopSurfaceConfig {
                title: "Native Test CLI Flags".into(),
                size: mindustry_desktop::DesktopSurfaceSize::new(960, 540),
                scale_factor: 1.0,
                resizable: true,
                maximized: false,
                visible: false,
            },
        );
        native_config.vsync = false;
        native_config.fullscreen = true;

        let app = DesktopNativeOpenGlApp::new(&mut launcher, native_config);

        assert!(!app.native_config.vsync);
        assert!(app.native_config.fullscreen);
        assert!(!app.launcher.graphics_vsync_enabled);
        assert!(app.launcher.graphics_fullscreen_enabled);
    }

    #[test]
    fn native_opengl_context_candidates_follow_upstream_version_fallback_order() {
        let macos = desktop_native_opengl_default_context_candidates_for_platform(
            true,
            false,
            DesktopNativeOpenGlContextProfile::Core,
        );
        assert_eq!(
            macos.first().copied(),
            Some(DesktopNativeOpenGlContextCandidate::versioned(
                4,
                1,
                DesktopNativeOpenGlContextProfile::Core
            ))
        );
        assert!(
            macos.contains(&DesktopNativeOpenGlContextCandidate::versioned(
                3,
                2,
                DesktopNativeOpenGlContextProfile::Core
            ))
        );

        let modern = desktop_native_opengl_default_context_candidates_for_platform(
            false,
            false,
            DesktopNativeOpenGlContextProfile::Core,
        );
        assert_eq!(
            modern.first().copied(),
            Some(DesktopNativeOpenGlContextCandidate::versioned(
                4,
                6,
                DesktopNativeOpenGlContextProfile::Core
            ))
        );
        assert!(
            modern.contains(&DesktopNativeOpenGlContextCandidate::versioned(
                3,
                3,
                DesktopNativeOpenGlContextProfile::Core
            ))
        );
        assert!(
            modern.contains(&DesktopNativeOpenGlContextCandidate::versioned(
                2,
                1,
                DesktopNativeOpenGlContextProfile::Compatibility
            ))
        );
        assert!(
            modern.contains(&DesktopNativeOpenGlContextCandidate::versioned(
                2,
                0,
                DesktopNativeOpenGlContextProfile::Compatibility
            ))
        );
        assert_eq!(
            modern.last().copied(),
            Some(DesktopNativeOpenGlContextCandidate::generic())
        );

        let legacy = desktop_native_opengl_default_context_candidates_for_platform(
            false,
            true,
            DesktopNativeOpenGlContextProfile::Core,
        );
        assert_eq!(
            legacy,
            vec![
                DesktopNativeOpenGlContextCandidate::versioned(
                    2,
                    1,
                    DesktopNativeOpenGlContextProfile::Compatibility
                ),
                DesktopNativeOpenGlContextCandidate::versioned(
                    2,
                    0,
                    DesktopNativeOpenGlContextProfile::Compatibility
                ),
                DesktopNativeOpenGlContextCandidate::generic()
            ],
            "explicit legacy compatibility branch only tries 2.x compatibility contexts"
        );
    }

    #[test]
    fn native_opengl_context_candidates_honor_java_like_gl_profile_args() {
        assert_eq!(
            desktop_native_parse_gl_version("3.2"),
            Some((3, 2)),
            "Java -gl expects <major>.<minor>"
        );
        assert_eq!(desktop_native_parse_gl_version("bad"), None);
        assert_eq!(
            desktop_native_parse_bool_env_flag("legacy"),
            Some(true),
            "runtime env hint accepts an Intel/legacy-like value"
        );
        assert_eq!(desktop_native_parse_bool_env_flag("modern"), Some(false));

        let explicit = desktop_native_opengl_context_candidates_from_args([
            "mindustry-desktop",
            "-compatibilityGl",
            "-gl",
            "2.1",
        ]);
        assert_eq!(
            explicit,
            vec![
                DesktopNativeOpenGlContextCandidate::versioned(
                    2,
                    1,
                    DesktopNativeOpenGlContextProfile::Compatibility
                ),
                DesktopNativeOpenGlContextCandidate::generic()
            ]
        );

        let core = desktop_native_opengl_context_candidates_from_args([
            "mindustry-desktop",
            "-compatibilityGl",
            "-coreGl",
            "-gl",
            "3.3",
        ]);
        assert_eq!(
            core.first().copied(),
            Some(DesktopNativeOpenGlContextCandidate::versioned(
                3,
                3,
                DesktopNativeOpenGlContextProfile::Core
            ))
        );

        let compatibility_default = desktop_native_opengl_default_context_candidates_for_platform(
            false,
            false,
            DesktopNativeOpenGlContextProfile::Compatibility,
        );
        assert_eq!(
            compatibility_default.first().copied(),
            Some(DesktopNativeOpenGlContextCandidate::versioned(
                4,
                6,
                DesktopNativeOpenGlContextProfile::Compatibility
            )),
            "Java -compatibilityGl flips the profile without requiring an explicit -gl version"
        );
    }

    #[test]
    fn native_opengl_antialias_flag_matches_java_opt_in_samples() {
        assert!(!desktop_native_antialias_enabled_from_args([
            "mindustry-desktop",
            "-gl",
            "3.3"
        ]));
        assert!(desktop_native_antialias_enabled_from_args([
            "mindustry-desktop",
            "-antialias"
        ]));
        assert!(desktop_native_antialias_enabled_from_args([
            "mindustry-desktop",
            "--antialias"
        ]));
    }

    #[test]
    fn native_opengl_window_icon_candidates_match_upstream_asset_path() {
        let paths = desktop_native_window_icon_candidate_paths_from_roots([
            std::path::PathBuf::from("core/assets"),
            std::path::PathBuf::from("D:/MDT/mindustry-upstream-v157.4/core/assets"),
        ]);
        assert_eq!(
            paths,
            vec![
                std::path::PathBuf::from("core/assets").join("icons/icon_64.png"),
                std::path::PathBuf::from("D:/MDT/mindustry-upstream-v157.4/core/assets")
                    .join("icons/icon_64.png"),
            ]
        );
    }

    #[test]
    fn native_opengl_window_icon_accepts_rgba8888_pixels() {
        assert!(desktop_native_window_icon_from_rgba(vec![255, 255, 255, 255], 1, 1).is_ok());
        assert!(
            desktop_native_window_icon_from_rgba(vec![255, 255, 255], 1, 1).is_err(),
            "winit icon creation should reject non-RGBA pixel lengths"
        );
    }

    #[test]
    fn default_framebuffer_state_trace_binds_viewport_and_disables_scissor() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let bind_trace = Rc::clone(&trace);
        let viewport_trace = Rc::clone(&trace);
        let scissor_trace = Rc::clone(&trace);

        prepare_default_framebuffer_state(
            960,
            540,
            move || {
                bind_trace
                    .borrow_mut()
                    .push(DefaultFramebufferStateOp::BindDefaultFramebuffer);
            },
            move |width, height| {
                viewport_trace
                    .borrow_mut()
                    .push(DefaultFramebufferStateOp::SetViewport(width, height));
            },
            move || {
                scissor_trace
                    .borrow_mut()
                    .push(DefaultFramebufferStateOp::DisableScissorTest);
            },
        );

        assert_eq!(
            &*trace.borrow(),
            &[
                DefaultFramebufferStateOp::BindDefaultFramebuffer,
                DefaultFramebufferStateOp::SetViewport(960, 540),
                DefaultFramebufferStateOp::DisableScissorTest,
            ]
        );
    }

    #[test]
    fn native_opengl_builtin_sprite_shader_uses_default_projection_and_samples_texture() {
        let vertex = desktop_native_opengl_builtin_sprite_shader_source(
            mindustry_core::mindustry::graphics::ShaderId::Mesh,
            mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Vertex,
            "mesh.vert",
        )
        .expect("native sprite vertex shader should override Mesh");
        let fragment = desktop_native_opengl_builtin_sprite_shader_source(
            mindustry_core::mindustry::graphics::ShaderId::Mesh,
            mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Fragment,
            "planet.frag",
        )
        .expect("native sprite fragment shader should override Mesh");

        assert!(vertex.starts_with("#version 150"));
        assert!(fragment.starts_with("#version 150"));
        assert!(vertex.contains("in vec4 a_position"));
        assert!(vertex.contains("in vec4 a_mix_color"));
        assert!(!vertex.contains("layout(location"));
        assert!(vertex.contains("uniform mat4 u_projTrans"));
        assert!(vertex.contains("uniform vec2 u_viewportInverse"));
        assert!(vertex.contains("gl_Position = u_projTrans * a_position"));
        assert!(fragment.contains("uniform sampler2D u_texture"));
        assert!(fragment.contains("texture(u_texture, v_texCoords)"));
        assert!(fragment.contains("v_mix_color"));
        assert_eq!(
            desktop_native_opengl_builtin_sprite_shader_source(
                mindustry_core::mindustry::graphics::ShaderId::Water,
                mindustry_desktop::DesktopGraphicsOpenGlBackendShaderStage::Vertex,
                "screenspace.vert",
            ),
            None
        );
    }

    #[test]
    fn native_opengl_pixel_projection_matrix_maps_surface_pixels_to_clip_space() {
        assert_eq!(
            desktop_native_opengl_pixel_projection_matrix(100, 50),
            [0.02, 0.0, 0.0, 0.0, 0.0, 0.04, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0,]
        );
        assert_eq!(
            desktop_native_opengl_pixel_projection_matrix(0, -1),
            [2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0,]
        );
    }

    #[test]
    fn native_opengl_window_title_with_diagnostic_appends_reason() {
        assert_eq!(
            desktop_native_window_title_with_diagnostic("Mindustry", None),
            "Mindustry"
        );
        assert_eq!(
            desktop_native_window_title_with_diagnostic(
                "Mindustry",
                Some("empty render frame: no GPU commands were recorded"),
            ),
            "Mindustry - empty render frame: no GPU commands were recorded"
        );
    }

    #[test]
    fn native_opengl_submit_diagnostic_reports_empty_frame_and_shader_failure() {
        let driver_state =
            mindustry_desktop::DesktopGraphicsOpenGlBackendDriverExecutionState::default();
        let diagnostic = desktop_native_opengl_submit_diagnostic(
            &driver_state,
            0,
            &[String::from(
                "failed to compile native OpenGL shader source: boom",
            )],
            true,
            true,
        )
        .expect("empty frame with shader failure should yield a diagnostic");

        assert!(diagnostic.contains("empty render frame"));
        assert!(diagnostic.contains("shader"));
        assert!(diagnostic.contains("boom"));
    }

    #[test]
    fn native_opengl_submit_diagnostic_reports_no_valid_draw_commands() {
        let driver_state = mindustry_desktop::DesktopGraphicsOpenGlBackendDriverExecutionState {
            draw_commands: 2,
            ..Default::default()
        };
        let diagnostic = desktop_native_opengl_submit_diagnostic(&driver_state, 2, &[], true, true)
            .expect("skipped draw commands should yield a diagnostic");

        assert!(diagnostic.contains("no valid draw commands"));
        assert!(diagnostic.contains("2 draw submissions were skipped"));
    }

    #[test]
    fn native_opengl_visible_fallback_covers_empty_or_invalid_draw_frames() {
        let empty = mindustry_desktop::DesktopGraphicsOpenGlBackendDriverExecutionState::default();
        assert!(desktop_native_opengl_submit_needs_visible_fallback(
            &empty,
            0,
            &[],
            true,
            true,
        ));

        let invalid = mindustry_desktop::DesktopGraphicsOpenGlBackendDriverExecutionState {
            draw_commands: 3,
            ..Default::default()
        };
        assert!(desktop_native_opengl_submit_needs_visible_fallback(
            &invalid,
            3,
            &[],
            true,
            true,
        ));
        assert!(!desktop_native_opengl_submit_needs_visible_fallback(
            &invalid,
            1,
            &[],
            true,
            true,
        ));
        assert!(desktop_native_opengl_submit_needs_visible_fallback(
            &invalid,
            1,
            &["shader program link failed".into()],
            true,
            true,
        ));
        assert!(desktop_native_opengl_submit_needs_visible_fallback(
            &invalid,
            1,
            &[],
            false,
            true,
        ));
        assert!(desktop_native_opengl_submit_needs_visible_fallback(
            &invalid,
            1,
            &[],
            true,
            false,
        ));
        let shader_asset_diagnostic =
            desktop_native_opengl_submit_diagnostic(&invalid, 1, &[], false, true)
                .expect("missing shader assets should stay visible in native diagnostics");
        assert!(shader_asset_diagnostic.contains("shader assets unavailable"));
        let font_asset_diagnostic =
            desktop_native_opengl_submit_diagnostic(&invalid, 1, &[], true, false)
                .expect("missing font assets should stay visible in native diagnostics");
        assert!(font_asset_diagnostic.contains("font assets unavailable"));

        let rects = desktop_native_visible_fallback_rects(
            mindustry_desktop::DesktopSurfaceSize::new(1280, 720),
        );
        assert_eq!(rects[0].width, 1280);
        assert_eq!(rects[0].height, 720);
        assert!(
            rects
                .iter()
                .skip(1)
                .any(|rect| rect.color == [0.18, 0.42, 0.56, 1.0]),
            "fallback should show a non-black selected menu bar"
        );
        assert!(
            rects.iter().any(|rect| {
                rect.color == [0.040, 0.049, 0.061, 1.0] && rect.width >= 220 && rect.height >= 220
            }),
            "fallback should show a left-side menu panel silhouette"
        );
        assert!(
            rects.iter().any(|rect| {
                rect.color == [0.24, 0.53, 0.69, 1.0] && rect.width >= 250 && rect.height <= 92
            }),
            "fallback should show a top logo block"
        );
        assert!(
            rects
                .iter()
                .any(|rect| rect.height <= 6 && rect.width >= 150),
            "fallback should expose version or diagnostic text lines"
        );
    }

    #[test]
    fn native_opengl_shader_asset_root_tracks_fonts_directory() {
        let root = unique_temp_shader_asset_root("fonts-directory");
        let resolution = desktop_native_opengl_shader_asset_root_resolution_from_candidates(
            vec![(root.clone(), "test")],
            std::env::temp_dir().join("mindustry-desktop-shader-root-fonts-fallback"),
        );

        assert_eq!(resolution.path, root);
        assert!(resolution.shaders_dir_exists);
        assert!(resolution.fonts_dir_exists);
    }

    #[test]
    fn native_opengl_shader_asset_root_candidates_cover_packaged_layouts() {
        let base = std::path::PathBuf::from("D:/MDT/rust-mindustry/target/debug");
        let mut candidates = Vec::new();
        desktop_native_push_shader_asset_root_candidates_near(
            &mut candidates,
            &base,
            "current-exe-near",
        );

        assert!(candidates
            .iter()
            .any(|(path, source)| path == &base.join("assets") && *source == "current-exe-near"));
        assert!(candidates
            .iter()
            .any(|(path, _)| path == &base.join("core").join("assets")));
        assert!(candidates.iter().any(|(path, _)| path
            == &base
                .join("..")
                .join("mindustry-upstream-v157.4")
                .join("core")
                .join("assets")));

        let deduped = desktop_native_dedup_shader_asset_root_candidates(vec![
            (base.join("assets"), "first"),
            (base.join("assets"), "second"),
        ]);
        assert_eq!(deduped, vec![(base.join("assets"), "first")]);
    }

    #[test]
    fn native_opengl_shader_asset_root_prefers_repository_over_reference() {
        let repo_assets = unique_temp_shader_asset_root("repo-order-repo");
        let reference_assets = unique_temp_shader_asset_root("repo-order-reference");

        let resolution = desktop_native_opengl_shader_asset_root_resolution_from_candidates(
            vec![
                (repo_assets.clone(), "repository"),
                (reference_assets, "reference"),
            ],
            unique_temp_shader_asset_root("repo-order-fallback"),
        );

        assert_eq!(resolution.path, repo_assets);
        assert_eq!(resolution.source, "repository");
        assert!(resolution.shaders_dir_exists);
    }

    #[test]
    fn native_opengl_shader_asset_root_exposes_chosen_reference_path() {
        let reference_assets = unique_temp_shader_asset_root("reference-order-reference");
        let missing_repo_assets = std::env::temp_dir().join(format!(
            "mindustry-desktop-shader-root-reference-order-missing-repo-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after UNIX_EPOCH")
                .as_nanos()
        ));

        let resolution = desktop_native_opengl_shader_asset_root_resolution_from_candidates(
            vec![
                (missing_repo_assets, "repository"),
                (reference_assets.clone(), "reference"),
            ],
            std::env::temp_dir().join("mindustry-desktop-shader-root-reference-fallback"),
        );

        assert_eq!(resolution.path, reference_assets);
        assert_eq!(resolution.source, "reference");
        assert!(resolution.shaders_dir_exists);
    }
}
