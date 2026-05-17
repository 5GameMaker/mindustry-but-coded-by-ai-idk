fn main() {
    let launcher = mindustry_server::run(std::env::args().collect());
    println!(
        "{} port={}",
        mindustry_server::banner(),
        launcher.context.port
    );
}
