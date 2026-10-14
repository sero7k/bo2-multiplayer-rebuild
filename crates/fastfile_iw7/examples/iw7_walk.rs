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
        if let Ok(spec) = std::env::var("IW7_HEX") {
            let (a, n) = spec.split_once(',').unwrap();
            let a = usize::from_str_radix(a.trim_start_matches("0x"), 16).unwrap();
            let n: usize = n.parse().unwrap();
            for (i, row) in zone.image[a..a + n].chunks(16).enumerate() {
                let hex: Vec<String> = row.iter().map(|b| format!("{b:02x}")).collect();
                let txt: String = row.iter().map(|&b| if (32..127).contains(&b) { b as char } else { '.' }).collect();
                println!("{:08x}  {}  {txt}", a + 16 * i, hex.join(" "));
            }
            continue;
        }
        let t = std::time::Instant::now();
        let (w, stop) = fastfile_iw7::walk::walk_fitted(&zone);
        eprintln!("walked in {:?}; {} delta segments {:x?}", t.elapsed(), w.deltas.len(), &w.deltas[..w.deltas.len().min(8)]);
        println!(
            "{path}: walked {}/{} pos {:#x}/{:#x} VIRTUAL {:#x} (block {:#x}) unresolved {} forward {}/{} stop {:?}",
            w.cap.walked,
            zone.assets.assets.len(),
            w.stream_pos(),
            zone.image.len(),
            w.virtual_offset(),
            zone.header.block_sizes[VIRTUAL as usize],
            w.unresolved.get(),
            w.unresolved_forward(),
            w.forward.len(),
            stop
        );
        if let Ok(spec) = std::env::var("IW7_DSCAN") {
            let (a, b) = spec.split_once(',').unwrap();
            let a = u64::from_str_radix(a, 16).unwrap();
            let b = u64::from_str_radix(b, 16).unwrap();
            let mut refs = w.string_refs.clone();
            refs.sort();
            refs.dedup();
            let mut hits = w.delta_hits(&refs, a..b);
            hits.sort_by_key(|h| std::cmp::Reverse(h.1));
            println!("{} string refs; best deltas {:x?}", refs.len(), &hits[..hits.len().min(6)]);
            for chunk in refs.chunks(refs.len().div_ceil(16).max(1)) {
                let mut h = w.delta_hits(chunk, a..b);
                h.sort_by_key(|h| std::cmp::Reverse(h.1));
                println!("  refs {:#x}..{:#x} n {} best {:x?}", chunk[0], chunk[chunk.len() - 1], chunk.len(), &h[..3]);
            }
        }
        if std::env::var_os("IW7_IMG").is_some() {
            let l1: u64 = w.cap.images.iter().map(|i| i.data_len1 as u64).sum();
            let l2: u64 = w.cap.images.iter().map(|i| i.data_len2 as u64).sum();
            let st = w.cap.images.iter().filter(|i| i.streamed).count();
            let px = w.cap.images.iter().filter(|i| i.pixels.is_some()).count();
            println!("images {} streamed {st} with pixels {px} len1 {l1:#x} len2 {l2:#x}", w.cap.images.len());
            let mut sums = [0u64; 4];
            for i in &w.cap.images {
                for k in 0..4 {
                    sums[k] += i.streams[k].2 as u64;
                }
            }
            println!("stream size sums {:x?}", sums);
            for i in w.cap.images.iter().take(5) {
                println!("    streams {:x?}", i.streams);
                println!("  {} len1 {:#x} len2 {:#x} streamed {} px {:?}", i.name, i.data_len1, i.data_len2, i.streamed, i.pixels.map(|p| p.len));
            }
        }
        if std::env::var_os("IW7_LIST").is_some() {
            for (i, a) in zone.assets.assets.iter().enumerate().take(w.cap.walked + 1) {
                let name = w.cap.by_table.get(&i).map(|id| w.cap.name_of(*id).to_owned()).unwrap_or_default();
                println!("  {i:4} {:<20} {:?} at {:#x} {name}", a.asset_type().map_or("?", |t| t.name()), a.header, w.cap.starts.get(i).copied().unwrap_or(0));
            }
        }
    }
}
