//! Contact sheets of every picture of some zones, to find the HUD's by eye:
//! `jup_hud_pics <install> <out dir> [max side] <zone>...`
//!
//! For each zone: `<out>/<zone>/sheet_<n>.png` (8 by 8 cells of 128 pixels
//! on a grey check, in asset order), `<out>/<zone>/pics/<hash>.png` (each
//! picture up to the max side, 256 by default) and `<out>/<zone>.tsv`
//! (index, sheet, cell, hash, width, height, format byte, bytes per pixel
//! of the whole chain, decoded size or why not). `JUP_FMT=0x1a=28,...` reads
//! more format bytes as DXGI formats (28: RGBA8, 61: R8, 98: BC7...).
//! `JUP_ONLY=0x30,0x24` keeps those format bytes only, `JUP_LARGEST=512`
//! pictures no wider or higher than that (the HUD's are small); the others
//! are listed with sheet -1.

use std::io::Write;
use std::path::Path;

use fastfile_jup::cards::{self, Rgba};
use fastfile_jup::image;
use fastfile_jup::map::materials::{MapImage, Packages};

const CELL: u32 = 128;
const PAD: u32 = 4;
const COLS: u32 = 8;
const ROWS: u32 = 8;

fn extra_formats() -> Vec<(u8, u32)> {
    std::env::var("JUP_FMT")
        .unwrap_or_default()
        .split(',')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            Some((u8::from_str_radix(k.trim().trim_start_matches("0x"), 16).ok()?, v.trim().parse().ok()?))
        })
        .collect()
}

fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
    let Ok(f) = std::fs::File::create(path) else { return };
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    if let Ok(mut wr) = e.write_header() {
        let _ = wr.write_image_data(px);
    }
}

/// Uncompressed levels the block decoder does not do.
fn decode_plain(dxgi: u32, w: u32, h: u32, data: &[u8]) -> Option<Vec<u8>> {
    let n = (w * h) as usize;
    match dxgi {
        28 | 29 => data.get(..n * 4).map(<[u8]>::to_vec),
        87 => data.get(..n * 4).map(|d| d.chunks(4).flat_map(|p| [p[2], p[1], p[0], p[3]]).collect()),
        61 => data.get(..n).map(|d| d.iter().flat_map(|&v| [255, 255, 255, v]).collect()),
        _ => None,
    }
}

fn level_size(dxgi: u32, w: u32, h: u32) -> Option<usize> {
    match dxgi {
        28 | 29 | 87 => Some((w * h * 4) as usize),
        61 => Some((w * h) as usize),
        _ => image::level_bytes(dxgi, w, h),
    }
}

/// The picture's largest level at most `max` that the zone or packages have.
fn picture(img: &image::ImageRec, zone: &[u8], dxgi: u32, paks: &Packages, oodle: &dyn fastfile_jup::Decompressor, max: u32) -> Result<Rgba, String> {
    let decode = |w: u32, h: u32, d: &[u8]| cards::decode_bcn(dxgi, w, h, d).or_else(|| decode_plain(dxgi, w, h, d));
    if image::block_bytes(dxgi).is_some() && img.inline.is_none() {
        let mi = MapImage { hash: img.hash, width: img.width, height: img.height, format: img.format, groups: img.groups.clone(), inline: None };
        // (`Packages::level` knows only the block formats.)
        if image::dxgi(img.format) == Some(dxgi) {
            let l = paks.level(&mi, max, oodle)?;
            return decode(l.width, l.height, &l.data).map(|p| Rgba { width: l.width, height: l.height, pixels: p }).ok_or("decode".into());
        }
    }
    // The chain from the top: in the zone, else the last group's bytes.
    let data: Vec<u8> = match img.inline {
        Some((at, n)) => zone[at..at + n].to_vec(),
        None => {
            let g = img.groups.last().ok_or("no groups")?;
            paks.fetch(g.key, oodle).ok_or("not in the packages")?
        }
    };
    let (mut w, mut h, mut off) = (u32::from(img.width), u32::from(img.height), 0usize);
    // (A streamed chain's last group starts at the top level.)
    while w.max(h) > max && w > 4 && h > 4 {
        off += level_size(dxgi, w, h).ok_or("size")?;
        (w, h) = (w / 2, h / 2);
    }
    let n = level_size(dxgi, w, h).ok_or("size")?;
    let d = data.get(off..off + n).ok_or_else(|| format!("short ({} of {})", data.len(), off + n))?;
    decode(w, h, d).map(|p| Rgba { width: w, height: h, pixels: p }).ok_or("decode".into())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(install), Some(out)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: jup_hud_pics <install> <out dir> [max side] <zone>...");
        std::process::exit(2);
    };
    let (max, zones) = match args.get(3).and_then(|a| a.parse::<u32>().ok()) {
        Some(m) => (m, &args[4..]),
        None => (256, &args[3..]),
    };
    let install = Path::new(install);
    let oodle = fastfile_jup::oodle::load(install).expect("oodle");
    let paks = Packages::open(install);
    println!("{} packages", paks.len());
    let extra = extra_formats();
    let only: Vec<u8> = std::env::var("JUP_ONLY")
        .unwrap_or_default()
        .split(',')
        .filter_map(|f| u8::from_str_radix(f.trim().trim_start_matches("0x"), 16).ok())
        .collect();
    let largest: u16 = std::env::var("JUP_LARGEST").ok().and_then(|v| v.parse().ok()).unwrap_or(u16::MAX);
    for zname in zones {
        let t = std::time::Instant::now();
        let (_, zone) = match fastfile_jup::read_zone(&install.join("zone").join(format!("{zname}.ff")), oodle.as_ref()) {
            Ok(z) => z,
            Err(e) => {
                println!("{zname}: {e}");
                continue;
            }
        };
        let header = match fastfile_jup::zone::parse_header(&zone) {
            Ok(h) => h,
            Err(e) => {
                println!("{zname}: header {e}");
                continue;
            }
        };
        let count = header.count(21);
        let Some(run) = image::ImageRun::find(&zone, count) else {
            println!("{zname}: {count} images listed, the run not found");
            continue;
        };
        let dir = Path::new(out).join(zname);
        let _ = std::fs::create_dir_all(dir.join("pics"));
        let mut tsv = std::fs::File::create(Path::new(out).join(format!("{zname}.tsv"))).expect("tsv");
        let _ = writeln!(tsv, "index\tsheet\tcell\thash\twidth\theight\tformat\tbpp\tresult");
        let per = (COLS * ROWS) as usize;
        let side = COLS * (CELL + PAD);
        let mut sheet = vec![0u8; (side * side * 4) as usize];
        let fresh = |s: &mut Vec<u8>| {
            for y in 0..side {
                for x in 0..side {
                    let v = if ((x / 8) + (y / 8)) % 2 == 0 { 70 } else { 100 };
                    s[((y * side + x) * 4) as usize..][..4].copy_from_slice(&[v, v, v, 255]);
                }
            }
        };
        fresh(&mut sheet);
        let mut formats: std::collections::BTreeMap<u8, (usize, f32)> = Default::default();
        let mut decoded = 0usize;
        let mut i = 0usize;
        let kept: Vec<usize> = run
            .at
            .iter()
            .copied()
            .filter(|&at| match image::parse(&zone, at) {
                Ok(Some(img)) => {
                    let keep = (only.is_empty() || only.contains(&img.format)) && img.width.max(img.height) <= largest;
                    if !keep {
                        let _ = writeln!(tsv, "-\t-1\t-\t{:016x}\t{}\t{}\t{:#04x}\t-\tfiltered", img.hash, img.width, img.height, img.format);
                    }
                    keep
                }
                _ => false,
            })
            .collect();
        for &at in &kept {
            let Ok(Some(img)) = image::parse(&zone, at) else { continue };
            i += 1;
            let i = i - 1;
            let px = f32::from(img.width) * f32::from(img.height) * f32::from(img.depth.max(1)) * f32::from(img.array.max(1));
            let bpp = img.data_size as f32 / px.max(1.0);
            let e = formats.entry(img.format).or_default();
            e.0 += 1;
            e.1 += bpp;
            let dxgi = image::dxgi(img.format).or_else(|| extra.iter().find(|(f, _)| *f == img.format).map(|(_, d)| *d));
            let res = match dxgi {
                None => Err("format".to_owned()),
                Some(_) if img.depth != 1 || img.array != 1 => Err("array".to_owned()),
                Some(d) => picture(&img, &zone, d, &paks, oodle.as_ref(), max),
            };
            let (sheet_no, cell) = (i / per, i % per);
            let note = match &res {
                Ok(p) => format!("{}x{}", p.width, p.height),
                Err(e) => e.clone(),
            };
            let _ = writeln!(tsv, "{i}\t{sheet_no}\t{cell}\t{:016x}\t{}\t{}\t{:#04x}\t{bpp:.2}\t{note}", img.hash, img.width, img.height, img.format);
            if let Ok(p) = &res {
                decoded += 1;
                write_png(&dir.join("pics").join(format!("{:016x}.png", img.hash)), p.width, p.height, &p.pixels);
                let s = cards::shrink(p, CELL, CELL);
                let (cx, cy) = ((cell as u32 % COLS) * (CELL + PAD), (cell as u32 / COLS) * (CELL + PAD));
                for y in 0..s.height {
                    for x in 0..s.width {
                        let src = &s.pixels[((y * s.width + x) * 4) as usize..][..4];
                        let d = &mut sheet[(((cy + y) * side + cx + x) * 4) as usize..][..4];
                        let a = u32::from(src[3]);
                        for c in 0..3 {
                            d[c] = ((u32::from(src[c]) * a + u32::from(d[c]) * (255 - a)) / 255) as u8;
                        }
                    }
                }
            }
            if cell == per - 1 || i + 1 == kept.len() {
                write_png(&dir.join(format!("sheet_{sheet_no:03}.png")), side, side, &sheet);
                fresh(&mut sheet);
            }
        }
        let f: Vec<String> = formats.iter().map(|(k, (n, b))| format!("{k:#04x}:{n} ({:.2} B/px)", b / *n as f32)).collect();
        println!("{zname}: {count} images, {decoded} decoded, {:?}; formats {}", t.elapsed(), f.join(" "));
    }
}
