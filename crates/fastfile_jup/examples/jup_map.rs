//! Reads a Modern Warfare III map as the game lane does and prints the
//! report: `jup_map <install> <map>` (e.g. `mp_jup_rust`).
//! `JUP_MAP_OBJ=<file>` also writes the world as an OBJ;
//! `JUP_MAP_PNG=<dir>` the colour pictures of the most drawn materials
//! (`JUP_MAP_PNG_COUNT`, 12) as PNGs.

use std::io::Write;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(install), Some(map)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: jup_map <install> <map>");
        std::process::exit(2);
    };
    let oodle = fastfile_jup::oodle::load(std::path::Path::new(install)).expect("oodle");
    let src = fastfile_jup::map::MapSources {
        install: PathBuf::from(install),
        name: map.clone(),
    };
    let t = std::time::Instant::now();
    match fastfile_jup::map::JupMap::load(&src, oodle.as_ref()) {
        Ok(m) => {
            for l in &m.report {
                println!("{l}");
            }
            println!(
                "bounds {:?} {:?}; read in {:.1}s",
                m.mins,
                m.maxs,
                t.elapsed().as_secs_f32()
            );
            if let Ok(dir) = std::env::var("JUP_MAP_PNG") {
                pictures(
                    &m,
                    &PathBuf::from(install),
                    &PathBuf::from(dir),
                    oodle.as_ref(),
                );
            }
            if let Ok(path) = std::env::var("JUP_MAP_OBJ") {
                let mut f = std::io::BufWriter::new(std::fs::File::create(path).expect("obj"));
                let mut base = 1usize;
                for s in &m.surfaces {
                    writeln!(f, "o s{}", s.index).unwrap();
                    for p in &s.positions {
                        writeln!(f, "v {} {} {}", p[0], p[2], -p[1]).unwrap();
                    }
                    for t in &s.tris {
                        writeln!(
                            f,
                            "f {} {} {}",
                            base + usize::from(t[0]),
                            base + usize::from(t[1]),
                            base + usize::from(t[2])
                        )
                        .unwrap();
                    }
                    base += s.positions.len();
                }
            }
        }
        Err(e) => {
            println!("error: {e}");
            std::process::exit(1);
        }
    }
}

/// The colour pictures of the most drawn materials, decoded to PNG.
fn pictures(
    m: &fastfile_jup::map::JupMap,
    install: &std::path::Path,
    dir: &std::path::Path,
    oodle: &dyn fastfile_jup::oodle::Decompressor,
) {
    let count: usize = std::env::var("JUP_MAP_PNG_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12);
    let mut tris = vec![0usize; m.materials.len()];
    for s in &m.surfaces {
        if let Some(k) = s.material {
            tris[k] += s.tris.len();
        }
    }
    let mut order: Vec<usize> = (0..m.materials.len()).collect();
    order.sort_by_key(|&k| std::cmp::Reverse(tris[k]));
    let paks = fastfile_jup::map::materials::Packages::open(install);
    println!("packages: {}", paks.len());
    let _ = std::fs::create_dir_all(dir);
    for &k in order.iter().take(count) {
        let mat = &m.materials[k];
        let formats: Vec<String> = mat
            .images
            .iter()
            .map(|&i| {
                format!(
                    "{:#x}:{}x{}",
                    m.images[i].format, m.images[i].width, m.images[i].height
                )
            })
            .collect();
        print!(
            "material {k} {:016x} ({} triangles) pictures {formats:?}",
            mat.hash, tris[k]
        );
        let Some(c) = mat.colour else {
            println!(" no colour");
            continue;
        };
        match paks.level(&m.images[c], 512, oodle) {
            Ok(l) => {
                let (w, h) = (l.width as usize, l.height as usize);
                let mut px = vec![0u8; w * h * 4];
                let bs = if l.dxgi == 71 { 8 } else { 16 };
                let mut tmp = [0u8; 64];
                for by in 0..h.div_ceil(4) {
                    for bx in 0..w.div_ceil(4) {
                        let at = (by * w.div_ceil(4) + bx) * bs;
                        let Some(src) = l.data.get(at..at + bs) else {
                            continue;
                        };
                        if l.dxgi == 71 {
                            bcdec_rs::bc1(src, &mut tmp, 16);
                        } else {
                            bcdec_rs::bc7(src, &mut tmp, 16);
                        }
                        for y in 0..4 {
                            for x in 0..4 {
                                let (px_, py) = (bx * 4 + x, by * 4 + y);
                                if px_ < w && py < h {
                                    px[(py * w + px_) * 4..][..4]
                                        .copy_from_slice(&tmp[y * 16 + x * 4..][..4]);
                                }
                            }
                        }
                    }
                }
                let path = dir.join(format!("m{k}.png"));
                let f = std::fs::File::create(&path).unwrap();
                let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
                e.set_color(png::ColorType::Rgba);
                e.set_depth(png::BitDepth::Eight);
                e.write_header().unwrap().write_image_data(&px).unwrap();
                println!(" -> {} ({w}x{h})", path.display());
            }
            Err(e) => println!(" colour: {e}"),
        }
    }
}
