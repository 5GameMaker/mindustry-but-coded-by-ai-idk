fn main() {
    let mut launcher = mindustry_desktop::run(std::env::args().collect());
    let mut effect_renderer = mindustry_desktop::HeadlessDesktopEffectRenderer::default();
    let mut graphics_renderer = mindustry_desktop::HeadlessDesktopGraphicsRenderer::default();
    let mut frame_index = 0u64;

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

    loop {
        launcher.update();
        launcher.render_default_graphics_frame_with(frame_index, &mut graphics_renderer);
        launcher.render_standard_effect_frame_with(&mut effect_renderer);
        frame_index = frame_index.wrapping_add(1);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
