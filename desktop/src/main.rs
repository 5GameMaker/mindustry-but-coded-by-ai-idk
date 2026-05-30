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
use raw_window_handle::HasWindowHandle;
#[cfg(feature = "opengl-native-runtime")]
use std::num::NonZeroU32;

fn main() {
    let mut launcher = mindustry_desktop::run(std::env::args().collect());
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

    run_desktop_frame_loop(&mut launcher);
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
fn desktop_native_trace(message: impl AsRef<str>) {
    if desktop_native_trace_enabled() {
        eprintln!("[desktop-native] {}", message.as_ref());
    }
}

#[cfg(not(feature = "opengl-backend"))]
fn run_desktop_frame_loop(launcher: &mut mindustry_desktop::DesktopLauncher) {
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
fn run_desktop_frame_loop(launcher: &mut mindustry_desktop::DesktopLauncher) {
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
fn run_desktop_frame_loop(launcher: &mut mindustry_desktop::DesktopLauncher) {
    if std::env::var_os("MINDUSTRY_DESKTOP_FAST_MENU").is_none() {
        std::env::set_var("MINDUSTRY_DESKTOP_FAST_MENU", "1");
    }
    let event_loop = winit::event_loop::EventLoop::new()
        .expect("failed to create winit event loop for native OpenGL runtime");
    let native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::default();
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
    effect_renderer: mindustry_desktop::HeadlessDesktopEffectRenderer,
    pending_events: Vec<mindustry_desktop::DesktopFrameLoopEvent>,
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
    shaders: std::collections::BTreeMap<u32, glow::NativeShader>,
    programs: std::collections::BTreeMap<u32, glow::NativeProgram>,
    program_shaders: std::collections::BTreeMap<u32, mindustry_core::mindustry::graphics::ShaderId>,
    shader_sources: std::collections::BTreeMap<u32, String>,
    uniform_locations: std::collections::BTreeMap<(u32, String), glow::NativeUniformLocation>,
    framebuffer_handle_cache: mindustry_desktop::DesktopGraphicsOpenGlBackendHandleCache,
    framebuffer_handle_allocator: mindustry_desktop::DesktopGraphicsOpenGlBackendHandleAllocator,
    shader_asset_root: std::path::PathBuf,
    current_program: Option<u32>,
    current_vertex_array: Option<u32>,
    native_errors: Vec<String>,
}

#[cfg(feature = "opengl-native-runtime")]
fn desktop_native_opengl_shader_asset_root() -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("MINDUSTRY_ASSET_ROOT") {
        return std::path::PathBuf::from(path);
    }
    if let Ok(current_dir) = std::env::current_dir() {
        let local_assets = current_dir.join("core").join("assets");
        if local_assets.join("shaders").is_dir() {
            return local_assets;
        }
    }
    let reference_assets = std::path::PathBuf::from("D:/MDT/mindustry-upstream-v157.4/core/assets");
    if reference_assets.join("shaders").is_dir() {
        return reference_assets;
    }
    std::path::PathBuf::from("core/assets")
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
in vec2 a_position;
in vec4 a_color;
in vec2 a_texCoord0;
in vec4 a_mix_color;

uniform vec2 u_surfaceSize;

out vec4 v_color;
out vec2 v_texCoords;
out vec4 v_mix_color;

void main(){
    vec2 safeSize = max(u_surfaceSize, vec2(1.0, 1.0));
    vec2 clip = vec2(a_position.x / safeSize.x * 2.0 - 1.0, a_position.y / safeSize.y * 2.0 - 1.0);
    gl_Position = vec4(clip, 0.0, 1.0);
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
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(native_config.window_attributes()))
            .build(event_loop, template, |configs| {
                configs
                    .max_by_key(|config| config.num_samples())
                    .expect("no compatible OpenGL config was returned by glutin")
            })
            .map_err(|error| format!("failed to build native OpenGL display/window: {error}"))?;
        let window =
            window.ok_or_else(|| "glutin display builder did not return a window".to_string())?;
        desktop_native_trace("runtime.new: window created");
        let raw_window_handle = window
            .window_handle()
            .map_err(|error| format!("failed to read raw window handle: {error}"))?
            .as_raw();
        let context_attributes = ContextAttributesBuilder::new()
            .with_profile(GlProfile::Core)
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw_window_handle));
        let fallback_context_attributes =
            ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let gl_display = gl_config.display();
        let not_current_context =
            unsafe { gl_display.create_context(&gl_config, &context_attributes) }
                .or_else(|_| unsafe {
                    gl_display.create_context(&gl_config, &fallback_context_attributes)
                })
                .map_err(|error| format!("failed to create native OpenGL context: {error}"))?;
        desktop_native_trace("runtime.new: context created");
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
            shaders: std::collections::BTreeMap::new(),
            programs: std::collections::BTreeMap::new(),
            program_shaders: std::collections::BTreeMap::new(),
            shader_sources: std::collections::BTreeMap::new(),
            uniform_locations: std::collections::BTreeMap::new(),
            framebuffer_handle_cache:
                mindustry_desktop::DesktopGraphicsOpenGlBackendHandleCache::default(),
            framebuffer_handle_allocator:
                mindustry_desktop::DesktopGraphicsOpenGlBackendHandleAllocator::default(),
            shader_asset_root: desktop_native_opengl_shader_asset_root(),
            current_program: None,
            current_vertex_array: None,
            native_errors: Vec::new(),
        };
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

    fn resize_native_surface(&mut self, size: mindustry_desktop::DesktopSurfaceSize) {
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
}

#[cfg(feature = "opengl-native-runtime")]
struct DesktopNativeOpenGlDriver<'a> {
    gl: &'a glow::Context,
    recording: &'a mut mindustry_desktop::DesktopGraphicsRecordingOpenGlBackendDriver,
    textures: &'a mut std::collections::BTreeMap<u32, glow::NativeTexture>,
    framebuffers: &'a mut std::collections::BTreeMap<u32, glow::NativeFramebuffer>,
    buffers: &'a mut std::collections::BTreeMap<u32, glow::NativeBuffer>,
    vertex_arrays: &'a mut std::collections::BTreeMap<u32, glow::NativeVertexArray>,
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
    current_program: &'a mut Option<u32>,
    current_vertex_array: &'a mut Option<u32>,
    bound_render_target: Option<mindustry_core::mindustry::graphics::RenderTarget>,
    draw_target_available: bool,
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

        let loader = mindustry_desktop::DesktopGraphicsOpenGlBackendShaderSourceLoader::new(
            self.shader_asset_root,
        );
        let source = loader.load_stage_source(shader, stage, source_path.to_string());
        let source = match source {
            Ok(source) => source,
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to load native OpenGL shader source {source_path}: {error:?}"
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
        let Some(location) = self.uniform_location(program_handle, program, "u_surfaceSize") else {
            return;
        };
        unsafe {
            self.gl
                .uniform_2_f32(Some(&location), width as f32, height as f32);
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
    ) -> Option<Vec<u8>> {
        match pixel_source.load_rgba8888_pixels() {
            Ok(pixels) => Some(pixels.pixels),
            Err(error) => {
                self.native_errors.push(format!(
                    "failed to load native OpenGL texture pixel source: {error:?}"
                ));
                None
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
                let pixels = self.pixel_data_from_source(pixel_source);
                let unpack = glow::PixelUnpackData::Slice(pixels.as_deref());
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
                        unpack,
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
                if let Some(pixels) = self.pixel_data_from_source(pixel_source) {
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
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::BufferData {
                target,
                usage,
                bytes,
            } => unsafe {
                self.gl.buffer_data_u8_slice(*target, bytes, *usage);
            },
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::EnableVertexAttributeArray {
                attribute_location,
            } => {
                if let Ok(attribute_location) = u32::try_from(*attribute_location) {
                    unsafe {
                        self.gl.enable_vertex_attrib_array(attribute_location);
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendSpriteMeshUploadCommand::VertexAttributePointer {
                attribute_location,
                components,
                gl_type,
                normalized,
                stride_bytes,
                offset_bytes,
            } => {
                if let (Ok(attribute_location), Ok(components), Ok(stride_bytes), Ok(offset_bytes)) = (
                    u32::try_from(*attribute_location),
                    i32::try_from(*components),
                    i32::try_from(*stride_bytes),
                    i32::try_from(*offset_bytes),
                ) {
                    unsafe {
                        self.gl.vertex_attrib_pointer_f32(
                            attribute_location,
                            components,
                            *gl_type,
                            *normalized,
                            stride_bytes,
                            offset_bytes,
                        );
                    }
                }
            }
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
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::BufferData {
                target,
                usage,
                bytes,
            } => unsafe {
                self.gl.buffer_data_u8_slice(*target, bytes, *usage);
            },
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::EnableVertexAttributeArray {
                attribute_location,
            } => {
                if let Ok(attribute_location) = u32::try_from(*attribute_location) {
                    unsafe {
                        self.gl.enable_vertex_attrib_array(attribute_location);
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendResolveMeshUploadCommand::VertexAttributePointer {
                attribute_location,
                components,
                gl_type,
                normalized,
                stride_bytes,
                offset_bytes,
            } => {
                if let (Ok(attribute_location), Ok(components), Ok(stride_bytes), Ok(offset_bytes)) = (
                    u32::try_from(*attribute_location),
                    i32::try_from(*components),
                    i32::try_from(*stride_bytes),
                    i32::try_from(*offset_bytes),
                ) {
                    unsafe {
                        self.gl.vertex_attrib_pointer_f32(
                            attribute_location,
                            components,
                            *gl_type,
                            *normalized,
                            stride_bytes,
                            offset_bytes,
                        );
                    }
                }
            }
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
                if let Some(program) = self.existing_program(*program_handle) {
                    unsafe {
                        self.gl.use_program(Some(program));
                    }
                    *self.current_program = Some(*program_handle);
                    self.upload_builtin_sprite_uniforms_for_program(*program_handle, program);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::ActiveTexture {
                texture_unit,
            } => unsafe {
                if !self.draw_target_available {
                    return;
                }
                self.gl.active_texture(*texture_unit);
            },
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::BindTexture {
                target,
                texture_handle,
            } => {
                if !self.draw_target_available {
                    return;
                }
                if let Some(texture) = self.texture_for_handle(*texture_handle) {
                    unsafe {
                        self.gl.bind_texture(*target, Some(texture));
                    }
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::BindVertexArray {
                vertex_array_handle,
            } => {
                if !self.draw_target_available {
                    return;
                }
                if let Some(vertex_array) = self.vertex_array_for_handle(*vertex_array_handle) {
                    unsafe {
                        self.gl.bind_vertex_array(Some(vertex_array));
                    }
                    *self.current_vertex_array = Some(*vertex_array_handle);
                }
            }
            mindustry_desktop::DesktopGraphicsOpenGlBackendDrawCommand::DrawElements {
                primitive_type,
                index_count,
                index_type,
                index_offset_bytes,
            } => {
                if !self.draw_target_available {
                    return;
                }
                if self.current_program.is_some() && self.current_vertex_array.is_some() {
                    if let (Ok(index_count), Ok(index_offset_bytes)) = (
                        i32::try_from(*index_count),
                        i32::try_from(*index_offset_bytes),
                    ) {
                        if index_count > 0 {
                            unsafe {
                                self.gl.draw_elements(
                                    *primitive_type,
                                    index_count,
                                    *index_type,
                                    index_offset_bytes,
                                );
                            }
                        }
                    }
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
        let surface_size = self
            .state
            .surface_size
            .unwrap_or_else(|| self.window_surface_size());
        let mut driver = DesktopNativeOpenGlDriver {
            gl: &self.gl,
            recording: &mut self.driver,
            textures: &mut self.textures,
            framebuffers: &mut self.framebuffers,
            buffers: &mut self.buffers,
            vertex_arrays: &mut self.vertex_arrays,
            shaders: &mut self.shaders,
            programs: &mut self.programs,
            program_shaders: &mut self.program_shaders,
            shader_sources: &mut self.shader_sources,
            uniform_locations: &mut self.uniform_locations,
            framebuffer_handle_cache: &mut self.framebuffer_handle_cache,
            framebuffer_handle_allocator: &mut self.framebuffer_handle_allocator,
            surface_size,
            shader_asset_root: &self.shader_asset_root,
            current_program: &mut self.current_program,
            current_vertex_array: &mut self.current_vertex_array,
            bound_render_target: None,
            draw_target_available: true,
            native_errors: &mut self.native_errors,
        };
        desktop_native_trace("runtime.submit: drive native OpenGL driver");
        let driver_state = executor.drive_driver(&mut driver);
        if desktop_native_trace_enabled() {
            desktop_native_trace(format!(
                "runtime.submit: driver done draw_commands={} resolve_commands={}",
                driver_state.draw_commands, driver_state.resolve_commands
            ));
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
        native_config: mindustry_desktop::DesktopNativeOpenGlRuntimeConfig,
    ) -> Self {
        let frame_loop = mindustry_desktop::DesktopFrameLoopState::new(
            native_config.surface.clone(),
            mindustry_desktop::DesktopFramePacing::default(),
        );
        Self {
            launcher,
            native_config,
            window_id: None,
            frame_loop,
            next_redraw_at: std::time::Instant::now(),
            graphics_renderer: None,
            effect_renderer: mindustry_desktop::HeadlessDesktopEffectRenderer::default(),
            pending_events: Vec::new(),
        }
    }

    fn drain_present_frame(&mut self) -> Option<mindustry_desktop::DesktopPresentResult> {
        if self.pending_events.is_empty() {
            self.pending_events
                .push(mindustry_desktop::DesktopFrameLoopEvent::Tick);
        }
        let events = std::mem::take(&mut self.pending_events);
        let graphics_renderer = self.graphics_renderer.as_mut()?;
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
        let runtime = DesktopNativeOpenGlRuntime::new(event_loop, &self.native_config)
            .expect("failed to initialize native OpenGL desktop runtime");
        let window_id = runtime.window.id();
        let size = runtime.window_surface_size();
        runtime.request_redraw();
        self.pending_events
            .push(mindustry_desktop::DesktopFrameLoopEvent::Resize(size));
        self.window_id = Some(window_id);
        self.graphics_renderer =
            Some(mindustry_desktop::DesktopOpenGlBackendGraphicsRenderer::new(runtime));
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

        if let winit::event::WindowEvent::Resized(size) = event {
            if let Some(renderer) = self.graphics_renderer.as_mut() {
                renderer
                    .runtime
                    .resize_native_surface(mindustry_desktop::DesktopSurfaceSize::new(
                        size.width,
                        size.height,
                    ));
            }
        }

        let should_present = matches!(event, winit::event::WindowEvent::RedrawRequested);
        self.pending_events
            .extend(mindustry_desktop::desktop_frame_loop_events_from_winit_window_event(&event));

        if should_present {
            if let Some(result) = self.drain_present_frame() {
                if result.should_stop() {
                    event_loop.exit();
                }
            } else {
                event_loop.exit();
            }
        }

        if matches!(event, winit::event::WindowEvent::CloseRequested) {
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

    #[test]
    fn native_opengl_app_initializes_frame_loop_from_native_surface_config() {
        let mut launcher = mindustry_desktop::DesktopLauncher::new(Vec::new());
        let native_config = mindustry_desktop::DesktopNativeOpenGlRuntimeConfig::from_surface(
            mindustry_desktop::DesktopSurfaceConfig {
                title: "Native Test".into(),
                size: mindustry_desktop::DesktopSurfaceSize::new(960, 540),
                scale_factor: 1.0,
                resizable: true,
                visible: false,
            },
        );

        let app = DesktopNativeOpenGlApp::new(&mut launcher, native_config.clone());

        assert_eq!(app.frame_loop.surface, native_config.surface);
        assert_eq!(app.frame_loop.next_frame_index, 0);
        assert!(app.window_id.is_none());
        assert!(app.graphics_renderer.is_none());
        assert!(app.pending_events.is_empty());
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
    fn native_opengl_builtin_sprite_shader_maps_pixels_and_samples_texture() {
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

        assert!(vertex.contains("uniform vec2 u_surfaceSize"));
        assert!(vertex.contains("in vec4 a_mix_color"));
        assert!(vertex.contains("gl_Position = vec4(clip, 0.0, 1.0)"));
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
}
