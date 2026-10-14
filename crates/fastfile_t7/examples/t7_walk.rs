//! Walks a Black Ops III zone's assets in table order as far as the known
//! layouts go: `t7_walk <zone.ff>...` (`T7_LIST=1` lists every asset walked,
//! `T7_RESYNC=1` skips assets it cannot read by finding the next one it can,
//! `T7_HEX=<hex offset>,<len>` hexdumps the inflated image, `T7_DUMP=<file>` writes it, `T7_DUMP_AT=<hex offset>,<len>` a window of it).
use fastfile_t7::walk::{Options, Stop, walk_with};

fn main() {
    for path in std::env::args().skip(1) {
        let z = match fastfile_t7::Zone::open(std::path::Path::new(&path)) {
            Ok(z) => z,
            Err(e) => {
                println!("{path}: {e}");
                continue;
            }
        };
        if let Ok(spec) = std::env::var("T7_HEX") {
            // `T7_HEX=<hex offset>,<len>`: hexdump the inflated image and stop.
            let (a, n) = spec.split_once(',').expect("T7_HEX=<hex offset>,<len>");
            let a = usize::from_str_radix(a.trim_start_matches("0x"), 16).expect("hex offset");
            let n: usize = n.parse().expect("length");
            let end = (a + n).min(z.image.len());
            for (i, row) in z.image[a.min(end)..end].chunks(16).enumerate() {
                let hex: Vec<String> = row.iter().map(|b| format!("{b:02x}")).collect();
                let txt: String = row.iter().map(|&b| if (32..127).contains(&b) { b as char } else { '.' }).collect();
                println!("{:08x}  {:<47}  {txt}", a + 16 * i, hex.join(" "));
            }
            continue;
        }
        if let Some(out) = std::env::var_os("T7_DUMP") {
            // `T7_DUMP=<file>`: write the inflated image (`T7_DUMP_AT=<hex
            // offset>,<len>`: only that window).
            let (a, n) = std::env::var("T7_DUMP_AT")
                .ok()
                .and_then(|v| {
                    let (a, n) = v.split_once(',')?;
                    Some((usize::from_str_radix(a.trim_start_matches("0x"), 16).ok()?, n.parse::<usize>().ok()?))
                })
                .unwrap_or((0, z.image.len()));
            let end = (a + n).min(z.image.len());
            std::fs::write(&out, &z.image[a.min(end)..end]).expect("write image");
        }
        let w = walk_with(&z, Options { resync: std::env::var_os("T7_RESYNC").is_some() });
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
        for k in &w.skipped {
            let name = fastfile_t7::AssetType::from_u32(k.raw_type).map_or("?", |t| t.name());
            println!(
                "  skipped {}..{} (first {name}) at {:#x}..{:#x}: {}",
                k.index, k.resumed_at, k.start, k.end, k.why
            );
        }
        let inline = z.assets.assets.iter().filter(|a| a.is_inline()).count();
        print!("{path}: walked {} of {inline} inline assets; ", w.walked.len());
        match &w.stop {
            Stop::Done { end } => println!("done at {end:#x} of {:#x}", z.image.len()),
            Stop::Unknown { index, raw_type, at } => {
                let name = fastfile_t7::AssetType::from_u32(*raw_type).map_or("?", |t| t.name());
                println!("stopped at asset {index} ({name}, type {raw_type:#x}) at {at:#x}: no walker yet");
                if std::env::var_os("T7_GUESS").is_some() {
                    // The first strings after the stop (a "," placeholder's
                    // name right after its header gives the header size).
                    let data = &z.image[*at..z.image.len().min(at + 0x40000)];
                    let mut shown = 0;
                    let mut i = 0;
                    while i < data.len() && shown < 4 {
                        let run = data[i..].iter().take_while(|&&b| (0x20..0x7f).contains(&b)).count();
                        if run >= 4 && data.get(i + run) == Some(&0) && (i == 0 || data[i - 1] == 0 || data[i - 1] == 0xff) {
                            println!("    string at +{i:#x}: {}", String::from_utf8_lossy(&data[i..i + run]));
                            shown += 1;
                        }
                        i += run.max(1);
                    }
                }
            }
            Stop::Failed { index, ty, at, error } => {
                println!("asset {index} ({}) at {at:#x} failed: {error}", ty.name());
            }
        }
        let r = &w.refs;
        println!(
            "  VIRTUAL {:#x} (header block size {:#x}), PHYSICAL {:#x} ({:#x}), STREAMER {:#x} ({:#x}); refs: {} resolved, {} unresolved, {} forward, {} other block{}",
            w.virtual_end,
            z.header.block_sizes[5],
            w.physical_end,
            z.header.block_sizes[6],
            w.streamer_end,
            z.header.block_sizes[8],
            r.resolved,
            r.unresolved,
            r.forward,
            r.other_block,
            w.first_unresolved.map_or(String::new(), |(at, v)| format!("; first unresolved at {at:#x} -> {v:#x}"))
        );
    }
}
