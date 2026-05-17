fn main() {
    let launcher = mindustry_desktop::run(std::env::args().collect());
    println!(
        "{} data_dir={}",
        mindustry_desktop::banner(),
        launcher.client.context.paths.data_dir
    );
}
