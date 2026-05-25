fn main() {
    let mut launcher = mindustry_server::run(std::env::args().collect());
    if let Some(error) = &launcher.network_error {
        eprintln!(
            "{} failed_to_open_network={}",
            mindustry_server::banner(),
            error
        );
        return;
    }

    println!(
        "{} port={}",
        mindustry_server::banner(),
        launcher.context.port
    );

    loop {
        launcher.update();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
