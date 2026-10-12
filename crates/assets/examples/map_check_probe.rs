//! Offline: `map_check_probe <game> <zone.ff>...` runs `assets::check_map`.
fn main() {
    let mut args = std::env::args().skip(1);
    let game = args.next().expect("game prefix");
    for path in args {
        let started = std::time::Instant::now();
        let line = assets::check_map(&game, std::path::Path::new(&path));
        println!("{game}:{path} [{:.1}s] {line}", started.elapsed().as_secs_f32());
    }
}
