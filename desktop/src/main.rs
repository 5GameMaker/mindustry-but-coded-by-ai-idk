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
        "{} data_dir={}",
        mindustry_desktop::banner(),
        launcher.client.context.paths.data_dir
    );

    loop {
        launcher.update();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
