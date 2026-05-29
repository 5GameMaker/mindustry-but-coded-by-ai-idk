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

#[cfg(feature = "opengl-backend")]
fn desktop_graphics_backend_label() -> &'static str {
    "opengl-backend:null-runtime-submit"
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

#[cfg(feature = "opengl-backend")]
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
