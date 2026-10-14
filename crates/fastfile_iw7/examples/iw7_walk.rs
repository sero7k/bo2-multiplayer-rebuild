//! Walks Infinite Warfare zones and says whether each walk ended exactly
//! at the zone's end: `iw7_walk <zone.ff>...` (`IW7_LIST=1`: every asset).
use fastfile_iw7::stream::VIRTUAL;

fn main() {
    for path in std::env::args().skip(1) {
        let zone = match fastfile_iw7::Zone::open(std::path::Path::new(&path)) {
            Ok(z) => z,
            Err(e) => {
                println!("{path}: {e}");
                continue;
            }
        };
        let mut w = fastfile_iw7::walk::Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
        let stop = w.walk().err();
        println!(
            "{path}: walked {}/{} pos {:#x}/{:#x} VIRTUAL {:#x} (block {:#x}) unresolved {} stop {:?}",
            w.cap.walked,
            zone.assets.assets.len(),
            w.stream_pos(),
            zone.image.len(),
            w.virtual_offset(),
            zone.header.block_sizes[VIRTUAL as usize],
            w.unresolved.get(),
            stop
        );
        if std::env::var_os("IW7_LIST").is_some() {
            for (i, a) in zone.assets.assets.iter().enumerate().take(w.cap.walked + 1) {
                let name = w.cap.by_table.get(&i).map(|id| w.cap.name_of(*id).to_owned()).unwrap_or_default();
                println!("  {i:4} {:<20} {:?} at {:#x} {name}", a.asset_type().map_or("?", |t| t.name()), a.header, w.cap.starts.get(i).copied().unwrap_or(0));
            }
        }
    }
}
