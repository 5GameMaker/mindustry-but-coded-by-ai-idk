fn main() {
    let mut launcher = mindustry_desktop::run(std::env::args().collect());
    let mut effect_renderer = mindustry_desktop::HeadlessDesktopEffectRenderer::default();
    let mut graphics_renderer = mindustry_desktop::HeadlessDesktopGraphicsRenderer::default();
    let mut frame_loop = mindustry_desktop::DesktopFrameLoopState::default();

    if let Some(error) = &launcher.connect_error {
        eprintln!(
            "{} failed_to_connect={}",
            mindustry_desktop::banner(),
            error
        );
        return;
    }

    println!(
        "{} data_dir={}",
        mindustry_desktop::banner(),
        launcher.client.context.paths.data_dir
    );

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
