//! Reads a Modern Warfare III map as the game lane does and prints the
//! report: `jup_map <install> <map>` (e.g. `mp_jup_rust`).
//! `JUP_MAP_OBJ=<file>` also writes the world as an OBJ.

use std::io::Write;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(install), Some(map)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: jup_map <install> <map>");
        std::process::exit(2);
    };
    let oodle = fastfile_jup::oodle::load(std::path::Path::new(install)).expect("oodle");
    let src = fastfile_jup::map::MapSources {
        install: PathBuf::from(install),
        name: map.clone(),
    };
    let t = std::time::Instant::now();
    match fastfile_jup::map::JupMap::load(&src, oodle.as_ref()) {
        Ok(m) => {
            for l in &m.report {
                println!("{l}");
            }
            println!(
                "bounds {:?} {:?}; read in {:.1}s",
                m.mins,
                m.maxs,
                t.elapsed().as_secs_f32()
            );
            if let Ok(path) = std::env::var("JUP_MAP_OBJ") {
                let mut f = std::io::BufWriter::new(std::fs::File::create(path).expect("obj"));
                let mut base = 1usize;
                for s in &m.surfaces {
                    writeln!(f, "o s{}", s.index).unwrap();
                    for p in &s.positions {
                        writeln!(f, "v {} {} {}", p[0], p[2], -p[1]).unwrap();
                    }
                    for t in &s.tris {
                        writeln!(
                            f,
                            "f {} {} {}",
                            base + usize::from(t[0]),
                            base + usize::from(t[1]),
                            base + usize::from(t[2])
                        )
                        .unwrap();
                    }
                    base += s.positions.len();
                }
            }
        }
        Err(e) => {
            println!("error: {e}");
            std::process::exit(1);
        }
    }
}
