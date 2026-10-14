//! What reading a map reports: `iw7_report <map.ff>...` prints each map's
//! `Iw7Map::report` lines and its static model counts.
use fastfile_iw7::map::{Iw7Map, MapSources};

fn main() {
    for path in std::env::args().skip(1) {
        let started = std::time::Instant::now();
        match Iw7Map::load(&MapSources::beside(std::path::Path::new(&path))) {
            Ok(map) => {
                println!("{path}: read in {:.1}s", started.elapsed().as_secs_f32());
                for l in &map.report {
                    println!("  {l}");
                }
            }
            Err(e) => println!("{path}: {e}"),
        }
    }
}
