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
                let probe = map.probe_image.as_deref().and_then(|n| map.image_info(n).map(|r| (n, r.format, r.width, r.level_count, r.num_elements)));
                println!("  probes: {} origins, array {probe:?}", map.probe_origins.len());
            }
            Err(e) => println!("{path}: {e}"),
        }
    }
}
