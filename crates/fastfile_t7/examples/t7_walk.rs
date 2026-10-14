//! Walks a Black Ops III zone's assets in table order as far as the known
//! layouts go: `t7_walk <zone.ff>...` (`T7_LIST=1` lists every asset walked).
use fastfile_t7::walk::{Stop, walk};

fn main() {
    for path in std::env::args().skip(1) {
        let z = match fastfile_t7::Zone::open(std::path::Path::new(&path)) {
            Ok(z) => z,
            Err(e) => {
                println!("{path}: {e}");
                continue;
            }
        };
        let w = walk(&z);
        if std::env::var_os("T7_LIST").is_some() {
            for a in &w.walked {
                println!(
                    "  {:6} {:<16} {:#09x}..{:#09x} {}",
                    a.index,
                    a.ty.name(),
                    a.start,
                    a.end,
                    a.name.as_deref().unwrap_or("")
                );
            }
        }
        let inline = z.assets.assets.iter().filter(|a| a.is_inline()).count();
        print!("{path}: walked {} of {inline} inline assets; ", w.walked.len());
        match &w.stop {
            Stop::Done { end } => println!("done at {end:#x} of {:#x}", z.image.len()),
            Stop::Unknown { index, raw_type, at } => {
                let name = fastfile_t7::AssetType::from_u32(*raw_type).map_or("?", |t| t.name());
                println!("stopped at asset {index} ({name}, type {raw_type:#x}) at {at:#x}: no walker yet");
            }
            Stop::Failed { index, ty, at, error } => {
                println!("asset {index} ({}) at {at:#x} failed: {error}", ty.name());
            }
        }
    }
}
