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
                if let Ok(from) = std::env::var("IW7_FROM") {
                    let from: usize = from.parse().unwrap();
                    let mut last = None;
                    let mut run = 0;
                    for (i, a) in z.assets.assets.iter().enumerate().skip(from) {
                        let t = a.asset_type().map_or("?", |t| t.name());
                        if Some(t) != last {
                            if let Some(l) = last {
                                println!("  {run:5} x {l} (to {})", i - 1);
                            }
                            last = Some(t);
                            run = 0;
                        }
                        run += 1;
                    }
                    if let Some(l) = last {
                        println!("  {run:5} x {l}");
                    }
                }
                for (t, n) in z.assets.counts() {
                    println!("  {:<24} {n}", t.map_or("?", |t| t.name()));
                }
            }
            Err(e) => println!("{path}: {e}"),
        }
    }
}
