//! What a Modern Warfare III install holds, at the container level: every
//! zone with its sizes (and whether its blocks walk), some of them fully
//! decompressed, and the KAPI packages with their entry counts.
//!
//! ```text
//! cargo run -p fastfile_jup --example jup_inventory -- "F:\Modern Warfare III 1.21" [options]
//!   --list              print one line per zone (default: failures and a summary)
//!   --match <substr>    only zones whose name contains <substr>
//!   --decode-every <n>  fully decompress every n-th zone (default 64; 0 = none)
//!   --decode-all        fully decompress every zone listed
//!   --entries <n>       decode the first, middle and last n entries of each package (default 1)
//! ```
//! Opens nothing for writing. Decompressing needs the game's
//! `oo2core_8_win64.dll` (Windows); elsewhere the walk still runs.
use std::path::{Path, PathBuf};
use std::time::Instant;

use fastfile_jup::container::{self, walk_blocks_in};
use fastfile_jup::oodle::{self, Decompressor};
use fastfile_jup::xsub::{self, XPak, XSub};

struct Opts {
    install: PathBuf,
    list: bool,
    filter: Option<String>,
    decode_every: usize,
    decode_all: bool,
    entries: usize,
}

fn parse_args() -> Opts {
    let mut a = std::env::args().skip(1);
    let mut o = Opts {
        install: PathBuf::new(),
        list: false,
        filter: None,
        decode_every: 64,
        decode_all: false,
        entries: 1,
    };
    let mut have_install = false;
    while let Some(x) = a.next() {
        match x.as_str() {
            "--list" => o.list = true,
            "--match" => o.filter = a.next(),
            "--decode-every" => {
                o.decode_every = a.next().and_then(|v| v.parse().ok()).unwrap_or(64)
            }
            "--decode-all" => o.decode_all = true,
            "--entries" => o.entries = a.next().and_then(|v| v.parse().ok()).unwrap_or(1),
            _ if !have_install => {
                o.install = PathBuf::from(x);
                have_install = true;
            }
            _ => {
                eprintln!("unknown argument {x}");
                std::process::exit(2);
            }
        }
    }
    if !have_install {
        eprintln!(
            "usage: jup_inventory <install dir> [--list] [--match s] [--decode-every n] [--decode-all] [--entries n]"
        );
        std::process::exit(2);
    }
    o
}

fn files_with(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let rd = std::fs::read_dir(dir).unwrap_or_else(|e| {
        eprintln!("cannot list {}: {e}", dir.display());
        std::process::exit(1);
    });
    let mut v: Vec<PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == ext))
        .collect();
    v.sort();
    v
}

fn name_of(p: &Path) -> String {
    p.file_name()
        .unwrap_or(p.as_os_str())
        .to_string_lossy()
        .into_owned()
}

fn mib(n: u64) -> f64 {
    n as f64 / (1024.0 * 1024.0)
}

fn main() {
    let o = parse_args();
    let zone_dir = o.install.join("zone");
    let oodle: Option<Box<dyn Decompressor>> = match oodle::load(&o.install) {
        Ok(d) => {
            println!("oodle: {}", d.name());
            Some(d)
        }
        Err(e) => {
            println!("oodle: none ({e}); walking only");
            None
        }
    };

    // Zones.
    let t = Instant::now();
    let mut zones = files_with(&zone_dir, "ff");
    if let Some(f) = &o.filter {
        zones.retain(|p| name_of(p).contains(f.as_str()));
    }
    let (mut walked, mut bad, mut decoded) = (0usize, 0usize, 0usize);
    let (mut file_bytes, mut zone_bytes, mut decoded_bytes, mut blocks_total) =
        (0u64, 0u64, 0u64, 0usize);
    for (i, p) in zones.iter().enumerate() {
        let name = name_of(p);
        let walk = std::fs::File::open(p)
            .map_err(container::ContainerError::from)
            .and_then(|mut f| walk_blocks_in(&mut f));
        let (h, blocks) = match walk {
            Ok(x) => x,
            Err(e) => {
                bad += 1;
                println!("BAD  {name}: {e}");
                continue;
            }
        };
        walked += 1;
        let len = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        file_bytes += len;
        zone_bytes += h.zone_size;
        blocks_total += blocks.len();
        let want_decode = o.decode_all || (o.decode_every > 0 && i % o.decode_every == 0);
        let mut status = "walked";
        if let (true, Some(dec)) = (want_decode, oodle.as_deref()) {
            match container::read_zone(p, dec) {
                Ok((_, zone)) if zone.len() as u64 == h.zone_size => {
                    decoded += 1;
                    decoded_bytes += zone.len() as u64;
                    status = "decoded";
                }
                Ok((_, zone)) => {
                    bad += 1;
                    println!(
                        "BAD  {name}: decoded {} bytes, header says {}",
                        zone.len(),
                        h.zone_size
                    );
                    continue;
                }
                Err(e) => {
                    bad += 1;
                    println!("BAD  {name}: {e}");
                    continue;
                }
            }
        }
        if o.list {
            println!(
                "zone {name:<64} file {len:>11} zone {:>11} blocks {:>5} flags {:02x?} {status}",
                h.zone_size,
                blocks.len(),
                h.flags
            );
        }
    }
    println!(
        "zones: {} files, {walked} walk, {bad} bad, {decoded} decompressed ({:.1} MiB) | {:.1} MiB on disk -> {:.1} MiB of zone, {blocks_total} blocks | {:.1}s",
        zones.len(),
        mib(decoded_bytes),
        mib(file_bytes),
        mib(zone_bytes),
        t.elapsed().as_secs_f64()
    );

    // Packages.
    let t = Instant::now();
    let mut xsubs = 0usize;
    let mut entries_total = 0usize;
    let mut checked = 0usize;
    let mut pkg_bad = 0usize;
    for p in files_with(&zone_dir, "xsub") {
        let name = name_of(&p);
        let mut x = match XSub::open(&p) {
            Ok(x) => x,
            Err(e) => {
                pkg_bad += 1;
                println!("BAD  {name}: {e}");
                continue;
            }
        };
        xsubs += 1;
        let n = x.entries.len();
        entries_total += n;
        let stored = x.entries.iter().filter(|e| !e.compressed).count();
        let raw: u64 = x.entries.iter().map(|e| e.decompressed as u64).sum();
        let mut picks: Vec<usize> = Vec::new();
        for k in 0..o.entries.min(n) {
            picks.extend([k, n / 2 + k, n.saturating_sub(1 + k)]);
        }
        picks.sort();
        picks.dedup();
        picks.retain(|&i| i < n);
        let mut status = String::new();
        if let Some(dec) = oodle.as_deref() {
            let mut ok = 0;
            for &i in &picks {
                let e = x.entries[i];
                match x.read(&e, dec) {
                    Ok(b) if b.len() == e.decompressed as usize => ok += 1,
                    Ok(_) => println!("BAD  {name} entry {:016x}: wrong length", e.key),
                    Err(err) => println!("BAD  {name}: {err}"),
                }
            }
            checked += ok;
            if ok != picks.len() {
                pkg_bad += 1;
            }
            status = format!(" decoded {ok}/{}", picks.len());
        }
        println!(
            "xsub {name:<32} {:>8.1} MiB  entries {n:>7} ({stored} stored)  {:>9.1} MiB decoded{status}",
            mib(x.header.file_len),
            mib(raw)
        );
    }
    for p in files_with(&zone_dir, "xpak") {
        let name = name_of(&p);
        match XPak::open(&p) {
            Ok(x) if x.parts.is_empty() => println!(
                "xpak {name:<32} entries {:>7}, no local parts (kind {:#x}, CDN)",
                x.entries.len(),
                x.header.kind
            ),
            Ok(x) => {
                let mut status = String::new();
                if let (Some(dec), Some(e)) = (oodle.as_deref(), x.entries.first()) {
                    status = match x.read(e, dec) {
                        Ok(b) if b.len() == e.decompressed as usize => {
                            " first entry decoded".into()
                        }
                        Ok(_) => {
                            pkg_bad += 1;
                            " first entry: wrong length".into()
                        }
                        Err(err) => {
                            pkg_bad += 1;
                            format!(" first entry: {err}")
                        }
                    };
                }
                println!(
                    "xpak {name:<32} entries {:>7} over {} parts (stride {} MiB){status}",
                    x.entries.len(),
                    x.parts.len(),
                    x.stride >> 20
                );
            }
            Err(e) => {
                // The index may still be readable when its parts are not.
                let idx = std::fs::File::open(&p)
                    .map_err(xsub::XSubError::from)
                    .and_then(|mut f| xsub::read_index(&mut f));
                match idx {
                    Ok((h, rows)) => println!(
                        "xpak {name:<32} entries {:>7}, parts {}: {e}",
                        rows.len(),
                        h.part_count
                    ),
                    Err(e2) => println!("BAD  {name}: {e2}"),
                }
                pkg_bad += 1;
            }
        }
    }
    println!(
        "packages: {xsubs} xsub, {entries_total} entries, {checked} entries decoded, {pkg_bad} with problems | {:.1}s",
        t.elapsed().as_secs_f64()
    );
}
