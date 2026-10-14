//! Walks Ghosts zones and says whether each walk is exact (stream end =
//! zone end, VIRTUAL end = the XFile's block size), with the map assets'
//! places: `iw6_walk_check <zone.ff>... [IW6_FROM=<asset index>]`.
fn main() {
    let from: Option<usize> = std::env::var("IW6_FROM").ok().and_then(|v| v.parse().ok());
    for path in std::env::args().skip(1) {
        let zone = fastfile_iw6::Zone::open(std::path::Path::new(&path)).expect("open");
        let mut w = fastfile_iw6::walk::Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
        let r = w.walk();
        println!(
            "{path}: walked {}/{} {:?}; pos {:#x} of {:#x}; virtual {:#x} of {:#x}; unresolved {} (first {:?}); suspect {} (first {:?})",
            w.cap.walked,
            zone.assets.assets.len(),
            r.err().map(|e| (e.asset_index, e.asset_type, e.error)),
            w.stream_pos(),
            zone.image.len(),
            w.virtual_offset(),
            zone.assets.xfile.block_sizes[3],
            w.unresolved.get(),
            w.first_unresolved.get(),
            w.suspect.get(),
            w.first_suspect.get(),
        );
        if let Ok(r) = std::env::var("IW6_HEX")
            && let Some((a, b)) = r.split_once(',')
        {
            let a = usize::from_str_radix(a.trim_start_matches("0x"), 16).unwrap();
            let b = usize::from_str_radix(b.trim_start_matches("0x"), 16).unwrap();
            for row in (a..b).step_by(16) {
                let bytes = &zone.image[row..(row + 16).min(zone.image.len())];
                println!(
                    "HEX {row:#x}: {}  {}",
                    bytes.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" "),
                    bytes.iter().map(|&x| if (32..127).contains(&x) { x as char } else { '.' }).collect::<String>()
                );
            }
        }
        for (i, a) in zone.assets.assets.iter().enumerate() {
            let ty = a.asset_type();
            let world = matches!(
                ty,
                Some(
                    fastfile_iw6::AssetType::GfxWorld
                        | fastfile_iw6::AssetType::ClipMap
                        | fastfile_iw6::AssetType::MapEnts
                        | fastfile_iw6::AssetType::ComWorld
                        | fastfile_iw6::AssetType::FxWorld
                        | fastfile_iw6::AssetType::GlassWorld
                )
            );
            if world || from.is_some_and(|f| i >= f && i < f + 40) {
                let name = w.cap.by_table.get(&i).map(|id| w.cap.name_of(*id).to_owned()).unwrap_or_default();
                println!("  {i:5} {:?} at {:#x} {name}", ty, w.cap.starts.get(i).copied().unwrap_or(0));
            }
        }
    }
}
