//! Writes pictures of a zone as PNGs, by their headers' places:
//! `jup_img <install> <zone image (.zone)> <out dir> <hex offset>...`
//! (BC1 and BC7 pictures, at most 1024 pixels.)

use fastfile_jup::image;
use fastfile_jup::map::materials::{MapImage, Packages};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(install), Some(zone), Some(dir)) = (args.get(1), args.get(2), args.get(3)) else {
        eprintln!("usage: jup_img <install> <file.zone> <out dir> <hex offset>...");
        std::process::exit(2);
    };
    let oodle = fastfile_jup::oodle::load(std::path::Path::new(install)).expect("oodle");
    let z = std::fs::read(zone).expect("zone");
    let paks = Packages::open(std::path::Path::new(install));
    let _ = std::fs::create_dir_all(dir);
    for a in &args[4..] {
        let at = usize::from_str_radix(a.trim_start_matches("0x"), 16).expect("offset");
        let Ok(Some(r)) = image::parse(&z, at) else {
            println!("{a}: no image");
            continue;
        };
        let img = MapImage {
            hash: r.hash,
            width: r.width,
            height: r.height,
            format: r.format,
            groups: r.groups.clone(),
            inline: r.inline.map(|(o, n)| z[o..o + n].to_vec()),
        };
        match paks.level(&img, 1024, oodle.as_ref()) {
            Ok(l) => {
                let (w, h) = (l.width as usize, l.height as usize);
                let mut px = vec![0u8; w * h * 4];
                let bs = if l.dxgi == 71 { 8 } else { 16 };
                let mut tmp = [0u8; 64];
                for by in 0..h.div_ceil(4) {
                    for bx in 0..w.div_ceil(4) {
                        let at = (by * w.div_ceil(4) + bx) * bs;
                        let Some(src) = l.data.get(at..at + bs) else { continue };
                        match l.dxgi {
                            71 => bcdec_rs::bc1(src, &mut tmp, 16),
                            83 => bcdec_rs::bc5(src, &mut tmp, 8, false),
                            _ => bcdec_rs::bc7(src, &mut tmp, 16),
                        }
                        for y in 0..4 {
                            for x in 0..4 {
                                let (px_, py) = (bx * 4 + x, by * 4 + y);
                                if px_ < w && py < h {
                                    let s = if l.dxgi == 83 { y * 8 + x * 2 } else { y * 16 + x * 4 };
                                    let d = &mut px[(py * w + px_) * 4..][..4];
                                    if l.dxgi == 83 {
                                        d.copy_from_slice(&[tmp[s], tmp[s + 1], 0, 255]);
                                    } else {
                                        d.copy_from_slice(&tmp[s..s + 4]);
                                    }
                                }
                            }
                        }
                    }
                }
                let path = std::path::Path::new(dir).join(format!("i{a}.png"));
                let f = std::fs::File::create(&path).unwrap();
                let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
                e.set_color(png::ColorType::Rgba);
                e.set_depth(png::BitDepth::Eight);
                e.write_header().unwrap().write_image_data(&px).unwrap();
                println!("{a}: {}x{} format {:#x} -> {} ({w}x{h})", r.width, r.height, r.format, path.display());
            }
            Err(e) => println!("{a}: {e}"),
        }
    }
}
