//! Modern Warfare III's emblems and calling cards: how many `global`
//! defines, their picture shapes, and a contact sheet of a sample.
//!
//! ```text
//! cargo run -p fastfile_jup --example jup_cards -- <install> <out dir> [options]
//!   --sample n     pictures per class on the sheet (default 16)
//!   --list         every item to cards.tsv
//!   --detect-all   count the flipbooks (decoding the square emblems the size cannot tell)
//!   --raw name     that item's largest group as stored (<name>.bin)
//!   --where k..    which packages hold these keys (hex)
//!   --show a,b,..  only these items' whole pictures (cards_show.png)
//! ```
//! Writes only into `<out dir>`: `cards_sheet.png` (the sample: emblems,
//! then calling cards, then the first frames of the flipbooks found among
//! them; `cards_sheet.txt` names each tile) and `cards_frames.png` (the
//! first four frames of a few flipbooks, one per row).
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use fastfile_jup::cards::{self, Item, Kind, Packages, Rgba};
use fastfile_jup::oodle;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 {
        eprintln!("usage: jup_cards <install> <out dir> [--sample n] [--list] [--detect-all] [--raw name] [--where keys..]");
        std::process::exit(2);
    }
    let install = PathBuf::from(&a[1]);
    let out = PathBuf::from(&a[2]);
    std::fs::create_dir_all(&out).expect("out dir");
    let (mut sample, mut raw, mut list, mut detect_all) = (16usize, None, false, false);
    let mut show: Vec<String> = Vec::new();
    let mut i = 3;
    while i < a.len() {
        match a[i].as_str() {
            "--sample" => {
                sample = a[i + 1].parse().unwrap_or(16);
                i += 1;
            }
            "--raw" => {
                raw = Some(a[i + 1].clone());
                i += 1;
            }
            "--list" => list = true,
            "--show" => {
                show = a[i + 1].split(',').map(str::to_owned).collect();
                i += 1;
            }
            "--detect-all" => detect_all = true,
            "--where" => {
                where_keys(&install, &a[i + 1..]);
                return;
            }
            x => panic!("unknown argument {x}"),
        }
        i += 1;
    }
    let oodle = oodle::load(&install).expect("the game's Oodle DLL");
    let t = Instant::now();
    let zone_dir = install.join("zone");
    let (_, zone) = fastfile_jup::read_zone(&zone_dir.join(format!("{}.ff", cards::ZONE)), oodle.as_ref()).expect("global.ff");
    println!("{}: {} bytes in {:?}", cards::ZONE, zone.len(), t.elapsed());
    let t = Instant::now();
    let names = cards::scan_names(&zone);
    let items = cards::find_items(&zone);
    println!("{} names, {} with a picture ({:?})", names.len(), items.len(), t.elapsed());
    type Shape = (Kind, Option<(u32, u32)>, u16, u16, u8);
    let mut shapes: BTreeMap<Shape, usize> = BTreeMap::new();
    for it in &items {
        *shapes.entry((it.kind, it.candidate_grid(), it.image.width, it.image.height, it.image.format)).or_default() += 1;
    }
    for (k, n) in &shapes {
        println!("  {n:5} {k:?}");
    }
    let count = |k: Kind| items.iter().filter(|i| i.kind == k).count();
    let could = |k: Kind| items.iter().filter(|i| i.kind == k && i.candidate_grid().is_some()).count();
    println!(
        "emblems {} ({} sized as flipbooks), cards {} ({} sized as flipbooks)",
        count(Kind::Emblem),
        could(Kind::Emblem),
        count(Kind::Card),
        could(Kind::Card)
    );
    let missing: Vec<&str> = names.iter().filter(|(n, _)| !items.iter().any(|i| &i.name == n)).map(|(n, _)| n.as_str()).collect();
    println!("{} names without a picture here, e.g. {:?}", missing.len(), &missing[..missing.len().min(12)]);
    if list {
        let mut s = String::new();
        for it in &items {
            s.push_str(&format!(
                "{}\t{:?}\t{:?}\t{}x{}\t{:#x}\t{}\n",
                it.name,
                it.kind,
                it.candidate_grid(),
                it.image.width,
                it.image.height,
                it.image.format,
                it.image.groups.iter().map(|g| format!("{:016x}", g.key)).collect::<Vec<_>>().join(",")
            ));
        }
        std::fs::write(out.join("cards.tsv"), s).expect("tsv");
    }
    let mut packages = Packages::new(&zone_dir, true);
    if let Some(name) = raw {
        let it = items.iter().find(|i| i.name == name).expect("no such item");
        let g = it.image.groups.len() - 1;
        let bytes = packages.read(it.image.groups[g].key, oodle.as_ref()).expect("its group");
        println!("{name}: group {g} {} bytes, {}x{} format {:#x}", bytes.len(), it.image.width, it.image.height, it.image.format);
        std::fs::write(out.join(format!("{name}.bin")), bytes).expect("raw");
    }
    let mut fetch = |key: u64| packages.read(key, oodle.as_ref());
    if detect_all {
        let t = Instant::now();
        let (mut books, mut stills, mut failed) = (BTreeMap::<Kind, usize>::new(), BTreeMap::<Kind, usize>::new(), 0usize);
        // (Those the size decides need no pixels.)
        for it in &items {
            match it.grid_by_size() {
                Some((1, 1)) => *stills.entry(it.kind).or_default() += 1,
                Some(_) => *books.entry(it.kind).or_default() += 1,
                None => {}
            }
        }
        let mut s = String::new();
        for it in items.iter().filter(|i| i.grid_by_size().is_none()) {
            match cards::picture(Some(&zone), it, &mut fetch, 64) {
                Some(p) => {
                    let g = cards::detect_grid(it, &p);
                    let c = it.candidate_grid().unwrap_or((1, 1));
                    let (near, spread) = cards::likeness(&p, c);
                    let gut = cards::gutters(&p, c);
                    s.push_str(&format!("{}\t{}x{}\t{:?}\t{near:.1}\t{spread:.1}\t{gut:.3}\n", it.name, it.image.width, it.image.height, g));
                    *if g == (1, 1) { stills.entry(it.kind).or_default() } else { books.entry(it.kind).or_default() } += 1;
                }
                None => failed += 1,
            }
        }
        std::fs::write(out.join("cards_detect.tsv"), s).expect("tsv");
        // (Whose pixels the packages the game reads have, and which are in
        // no installed package at all.)
        let (mut likely, mut all) = (Packages::new(&zone_dir, false), Packages::new(&zone_dir, true));
        let mut missing = Vec::new();
        let mut elsewhere: BTreeMap<usize, usize> = BTreeMap::new();
        for it in &items {
            let Some(g) = it.image.groups.last() else { continue };
            if likely.find(g.key).is_none() {
                match all.find(g.key) {
                    Some((i, _)) => *elsewhere.entry(i).or_default() += 1,
                    None => missing.push(it.name.as_str()),
                }
            }
        }
        let names = all.opened();
        let others: Vec<String> = elsewhere.iter().map(|(i, n)| format!("{} {n}", names.get(*i).map_or("?", String::as_str))).collect();
        let n_else: usize = elsewhere.values().sum();
        println!("pixels: {} in the likely packages, {n_else} only in others {others:?}, {} in none: {:?}", items.len() - n_else - missing.len(), missing.len(), &missing[..missing.len().min(20)]);
        println!("flipbooks {books:?}, stills {stills:?}, square emblems not decoded {failed} ({:?})", t.elapsed());
    }
    if !show.is_empty() {
        let tiles: Vec<(String, Rgba)> = show
            .iter()
            .filter_map(|n| items.iter().find(|i| &i.name == n))
            .filter_map(|it| cards::picture(Some(&zone), it, &mut fetch, 1024).map(|p| (it.name.clone(), p)))
            .collect();
        sheet(&tiles, &out.join("cards_show.png"), 4);
        return;
    }
    // The sample: evenly spaced over each kind.
    let mut tiles: Vec<(String, Rgba)> = Vec::new();
    let mut books: Vec<(String, Rgba, (u32, u32))> = Vec::new();
    let t = Instant::now();
    let (mut ok, mut failed) = (0, 0);
    for k in [Kind::Emblem, Kind::Card] {
        let class: Vec<&Item> = items.iter().filter(|i| i.kind == k).collect();
        let n = (sample * 2).min(class.len());
        for j in 0..n {
            let it = class[j * class.len() / n.max(1)];
            match cards::picture(Some(&zone), it, &mut fetch, 256) {
                Some(p) => {
                    ok += 1;
                    let g = cards::detect_grid(it, &p);
                    let c = it.candidate_grid().unwrap_or((1, 1));
                    let (near, spread) = cards::likeness(&p, c);
                    println!("  {} {}x{} grid {g:?} (near {near:.1} spread {spread:.1} gutters {:.3})", it.name, p.width, p.height, cards::gutters(&p, c));
                    tiles.push((it.name.clone(), cards::frame(&p, g, cards::poster_frame(&p, g))));
                    if g != (1, 1) {
                        books.push((it.name.clone(), p, g));
                    }
                }
                None => {
                    failed += 1;
                    println!("  no picture: {} (format {:#x})", it.name, it.image.format);
                }
            }
        }
    }
    println!("sample: {ok} decoded, {failed} not, in {:?}; packages {:?}", t.elapsed(), packages.opened());
    sheet(&tiles, &out.join("cards_sheet.png"), 8);
    let mut names = String::new();
    for (i, (n, _)) in tiles.iter().enumerate() {
        names.push_str(&format!("{i}\t{n}\n"));
    }
    std::fs::write(out.join("cards_sheet.txt"), names).expect("names");
    let mut frames = Vec::new();
    for (name, p, g) in books.iter().take(6) {
        for f in 0..4 {
            frames.push((name.clone(), cards::frame(p, *g, f)));
        }
    }
    if !frames.is_empty() {
        sheet(&frames, &out.join("cards_frames.png"), 4);
    }
}

/// Which `.xpak` indexes (and package parts) hold each key.
fn where_keys(install: &std::path::Path, keys: &[String]) {
    let keys: Vec<u64> = keys.iter().filter_map(|k| u64::from_str_radix(k.trim_start_matches("0x"), 16).ok()).collect();
    for dir in [install.join("xpak_cache"), install.join("zone")] {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        files.sort();
        for f in files {
            let ext = f.extension().and_then(|x| x.to_str()).unwrap_or("");
            if ext != "xpak" && ext != "xsub" {
                continue;
            }
            let rows = match std::fs::File::open(&f).map_err(fastfile_jup::XSubError::from).and_then(|mut r| fastfile_jup::xsub::read_index(&mut r)) {
                Ok((_, rows)) => rows,
                Err(e) => {
                    println!("{}: {e}", f.display());
                    continue;
                }
            };
            for k in &keys {
                if let Ok(i) = rows.binary_search_by_key(k, |e| e.key) {
                    println!("  {k:016x} in {} (offset {:#x}, size {})", f.display(), rows[i].offset, rows[i].size);
                }
            }
        }
    }
}

/// Tiles in rows of `cols`, each fitted into 256x256, on grey.
fn sheet(tiles: &[(String, Rgba)], path: &std::path::Path, cols: usize) {
    let (cw, ch) = (264usize, 264usize);
    let rows = tiles.len().div_ceil(cols).max(1);
    let (w, h) = (cw * cols, ch * rows);
    let mut px = vec![0u8; w * h * 4];
    for p in px.chunks_mut(4) {
        p.copy_from_slice(&[48, 48, 52, 255]);
    }
    for (i, (_, t)) in tiles.iter().enumerate() {
        let t = cards::shrink(t, 256, 256);
        let (ox, oy) = ((i % cols) * cw + 4, (i / cols) * ch + 4);
        for y in 0..t.height as usize {
            for x in 0..t.width as usize {
                let s = &t.pixels[(y * t.width as usize + x) * 4..][..4];
                let d = &mut px[((oy + y) * w + ox + x) * 4..][..4];
                let a = u32::from(s[3]);
                for c in 0..3 {
                    d[c] = ((u32::from(s[c]) * a + u32::from(d[c]) * (255 - a)) / 255) as u8;
                }
            }
        }
    }
    let f = std::fs::File::create(path).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().expect("png").write_image_data(&px).expect("png");
    println!("{}", path.display());
}
