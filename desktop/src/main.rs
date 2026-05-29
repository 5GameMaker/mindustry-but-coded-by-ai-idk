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
}

#[cfg(feature = "opengl-native-runtime")]
impl DesktopNativeOpenGlRuntime {
    fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        native_config: &mindustry_desktop::DesktopNativeOpenGlRuntimeConfig,
    ) -> Result<Self, String> {
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
        };
        runtime.resize_native_surface(runtime.window_surface_size());
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
        unsafe {
            self.gl.clear_color(0.015, 0.018, 0.025, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
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
        self.clear_backbuffer();
        let driver_state = executor.drive_driver(&mut self.driver);
        self.state.frames_submitted += 1;
        self.state.last_driver_state = Some(driver_state);
        driver_state
    }

    fn present_frame(&mut self) {
        self.surface
            .swap_buffers(&self.context)
            .expect("native OpenGL swap_buffers failed");
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
        Some(self.launcher.step_desktop_frame_loop(
            &mut self.frame_loop,
            &events,
            graphics_renderer,
            &mut self.effect_renderer,
        ))
    }
}

#[cfg(feature = "opengl-native-runtime")]
impl winit::application::ApplicationHandler for DesktopNativeOpenGlApp<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.graphics_renderer.is_some() {
            return;
        }

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

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self.frame_loop.closed {
            if let Some(renderer) = self.graphics_renderer.as_ref() {
                renderer.runtime.request_redraw();
            }
        }
    }
}

#[cfg(all(test, feature = "opengl-native-runtime"))]
mod tests {
    use super::*;

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
}
