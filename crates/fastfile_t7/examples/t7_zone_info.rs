//! A Black Ops III zone's header and asset counts: `t7_zone_info <zone.ff>...`.
//! `T7_STRINGS=1` also prints the script strings, `T7_ORDER=1` the table
//! as runs of one type.
fn main() {
    for path in std::env::args().skip(1) {
        match fastfile_t7::Zone::open(std::path::Path::new(&path)) {
            Ok(z) => {
                let h = &z.header;
                println!(
                    "{path}: zone \"{}\" ({}, changelist {}), image {:#x} bytes, {} script strings, {} depends, {} assets (table at {:#x}, data at {:#x})",
                    h.name,
                    h.builder,
                    h.changelist,
                    z.image.len(),
                    z.assets.script_strings.len(),
                    z.assets.depends.len(),
                    z.assets.assets.len(),
                    z.assets.table_offset,
                    z.assets.body_offset
                );
                let blocks: Vec<String> = h
                    .block_sizes
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| **s != 0)
                    .map(|(i, s)| format!("{i}:{s:#x}"))
                    .collect();
                println!("  block sizes {}", blocks.join(" "));
                for d in z.assets.depends.iter().flatten() {
                    println!("  depends {d}");
                }
                if std::env::var_os("T7_STRINGS").is_some() {
                    for (i, s) in z.assets.script_strings.iter().enumerate() {
                        println!("  string {i:5} {}", s.as_deref().unwrap_or("<null>"));
                    }
                }
                if std::env::var_os("T7_ORDER").is_some() {
                    let mut runs: Vec<(u32, bool, usize, usize)> = Vec::new();
                    for (i, a) in z.assets.assets.iter().enumerate() {
                        match runs.last_mut() {
                            Some(r) if r.0 == a.raw_type && r.1 == a.is_inline() => r.3 += 1,
                            _ => runs.push((a.raw_type, a.is_inline(), i, 1)),
                        }
                    }
                    for (t, inline, from, n) in runs {
                        let name = fastfile_t7::AssetType::from_u32(t).map_or("?", |t| t.name());
                        println!("  {from:6} {n:5} x {name}{}", if inline { "" } else { " (ref)" });
                    }
                }
                println!("  {:<28} {:>6} {:>6}", "type", "inline", "ref");
                for (t, inline, refs) in z.assets.counts_split() {
                    let name = t.map_or("?".to_string(), |t| format!("{} ({:#x})", t.name(), t as u32));
                    println!("  {name:<28} {inline:>6} {refs:>6}");
                }
            }
            Err(e) => println!("{path}: {e}"),
        }
    }
}
