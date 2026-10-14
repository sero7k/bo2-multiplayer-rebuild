//! Black Ops III `.xpak` packages: `t7_xpak info <file.xpak>...` prints the
//! header; `t7_xpak find <zone dir> <hex key>...` names the packages that
//! hold each key, with the entry's chunks and decoded size
//! (`T7_XPAK_OUT=<dir>`: also writes `<key>.raw` and `<key>.bin`).
use fastfile_t7::xpak::{Xpak, XpakSet, chunks, decode};
use std::path::{Path, PathBuf};

fn parse_key(s: &str) -> Option<u64> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("info") => {
            for p in &args[1..] {
                match Xpak::open(Path::new(p)) {
                    Ok(x) => {
                        let h = &x.header;
                        println!(
                            "{p}: version {}, {} entries, data {:#x}+{:#x}, hash {:#x} ({} x 24), index {:#x}+{:#x} ({})",
                            h.version, h.entry_count, h.data_offset, h.data_size, h.hash_offset, h.hash_count, h.index_offset, h.index_size, h.index_count
                        );
                        let flagged = x.entries.iter().filter(|e| e.flags != 0).count();
                        println!("  {flagged} entries with size flags");
                    }
                    Err(e) => println!("{p}: {e}"),
                }
            }
        }
        Some("find") if args.len() > 2 => {
            let (set, bad) = XpakSet::open_dir(Path::new(&args[1])).expect("zone dir");
            for (p, e) in bad {
                eprintln!("{}: {e}", p.display());
            }
            let out = std::env::var_os("T7_XPAK_OUT").map(PathBuf::from);
            for k in &args[2..] {
                let Some(key) = parse_key(k) else {
                    eprintln!("bad key {k}");
                    continue;
                };
                let hits = set.find_all(key);
                if hits.is_empty() {
                    println!("{key:#018x}: not found in {} packages", set.paks.len());
                }
                for (x, e) in hits {
                    let name = x.path.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned());
                    let raw = match x.read_raw(e) {
                        Ok(r) => r,
                        Err(err) => {
                            println!("{key:#018x}: {name}: {err}");
                            continue;
                        }
                    };
                    let cs = chunks(&raw);
                    let summary = match &cs {
                        Ok(cs) => {
                            let lz4 = cs.iter().filter(|c| c.codec == 3).count();
                            let other: Vec<u8> = cs.iter().map(|c| c.codec).filter(|c| *c != 3).collect();
                            format!("{} chunks ({lz4} LZ4, others {other:?})", cs.len())
                        }
                        Err(err) => format!("chunks: {err}"),
                    };
                    let dec = decode(&raw);
                    println!(
                        "{key:#018x}: {name} at {:#x}, {} bytes stored (flags {:#04x}), {summary}, decodes to {}",
                        e.offset,
                        e.size,
                        e.flags,
                        dec.as_ref().map_or_else(|err| err.to_string(), |d| format!("{} bytes", d.len()))
                    );
                    if let Some(dir) = &out {
                        std::fs::create_dir_all(dir).expect("out dir");
                        std::fs::write(dir.join(format!("{key:016x}.raw")), &raw).expect("write raw");
                        if let Ok(d) = dec {
                            std::fs::write(dir.join(format!("{key:016x}.bin")), d).expect("write bin");
                        }
                    }
                }
            }
        }
        _ => {
            eprintln!("usage: t7_xpak info <file.xpak>... | t7_xpak find <zone dir> <hex key>...");
            std::process::exit(2);
        }
    }
}
