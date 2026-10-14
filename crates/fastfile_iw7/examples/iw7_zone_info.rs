//! An Infinite Warfare zone's asset counts: `iw7_zone_info <zone.ff>...`.
fn main() {
    for path in std::env::args().skip(1) {
        match fastfile_iw7::Zone::open(std::path::Path::new(&path)) {
            Ok(z) => {
                println!(
                    "{path}: image {:#x}, {} strings, {} assets, body at {:#x} (virtual {:#x})",
                    z.image.len(),
                    z.assets.script_strings.len(),
                    z.assets.assets.len(),
                    z.assets.body_offset,
                    z.assets.body_virtual
                );
                for (t, n) in z.assets.counts() {
                    println!("  {:<24} {n}", t.map_or("?", |t| t.name()));
                }
            }
            Err(e) => println!("{path}: {e}"),
        }
    }
}
