//! bo2mp: where a Black Ops III map's reflection probes keep their bricks
//! in the probe volume atlas: `t7_bricks <zone.ff> [slice z]` decodes the
//! atlas's X texture and prints, per axis, how sharply neighbouring texel
//! planes differ (brick seams show as peaks), and with a slice its texels'
//! luminance as a character map (16 levels; `.` black).
use fastfile_t7::walk::{Options, walk_with};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let slice: Option<usize> = a.get(2).and_then(|v| v.parse().ok());
    let z = fastfile_t7::Zone::open(path).expect("zone");
    let w = walk_with(&z, Options::default());
    let g = w.capture.world.gfx_worlds.first().expect("gfxworld");
    let l = g.sun_lighting.first().expect("sun volume");
    let im = w.capture.image(l.probe_volumes[0][0].as_deref().unwrap().trim_start_matches(',')).expect("probe image");
    let (vw, vh, vd) = (im.width as usize, im.height as usize, im.depth as usize);
    let b = l.volume_buffers[0].expect("volume buffer");
    let key = u64::from_le_bytes(z.image[b + 8..b + 16].try_into().unwrap());
    let (paks, _) = fastfile_t7::xpak::XpakSet::open_dir(path.parent().unwrap()).expect("paks");
    let data = paks.read(key).expect("volume data");
    let slice_bytes = (vw / 4) * (vh / 4) * 16;
    let mut lum = vec![0f32; vw * vh * vd];
    let mut blk = [0f32; 48];
    for zz in 0..vd {
        let s = &data[zz * slice_bytes..(zz + 1) * slice_bytes];
        for (i, c) in s.chunks_exact(16).enumerate() {
            let (bx, by) = (i % (vw / 4) * 4, i / (vw / 4) * 4);
            bcdec_rs::bc6h_float(c, &mut blk, 12, false);
            for y in 0..4 {
                for x in 0..4 {
                    let o = y * 12 + x * 3;
                    let v = 0.2126 * blk[o] + 0.7152 * blk[o + 1] + 0.0722 * blk[o + 2];
                    lum[(zz * vh + by + y) * vw + bx + x] = if v.is_finite() { v } else { 0.0 };
                }
            }
        }
    }
    let at = |x: usize, y: usize, zz: usize| lum[(zz * vh + y) * vw + x];
    let seam = |d: f32, m: f32| if d > 0.25 * m.max(0.05) { 1usize } else { 0 };
    println!("atlas {vw}x{vh}x{vd}");
    let mut line = String::from("x seams:");
    for x in 0..vw - 1 {
        let mut n = 0;
        for zz in 0..vd {
            for y in 0..vh {
                let (p, q) = (at(x, y, zz), at(x + 1, y, zz));
                n += seam((p - q).abs(), p.max(q));
            }
        }
        line.push_str(&format!(" {}:{}", x + 1, n * 1000 / (vd * vh)));
    }
    println!("{line}");
    let mut line = String::from("y seams:");
    for y in 0..vh - 1 {
        let mut n = 0;
        for zz in 0..vd {
            for x in 0..vw {
                let (p, q) = (at(x, y, zz), at(x, y + 1, zz));
                n += seam((p - q).abs(), p.max(q));
            }
        }
        line.push_str(&format!(" {}:{}", y + 1, n * 1000 / (vd * vw)));
    }
    println!("{line}");
    let mut line = String::from("z seams:");
    for zz in 0..vd - 1 {
        let mut n = 0;
        for y in 0..vh {
            for x in 0..vw {
                let (p, q) = (at(x, y, zz), at(x, y, zz + 1));
                n += seam((p - q).abs(), p.max(q));
            }
        }
        line.push_str(&format!(" {}:{}", zz + 1, n * 1000 / (vh * vw)));
    }
    println!("{line}");
    if let Some(zz) = slice {
        let mut max = 0f32;
        for y in 0..vh {
            for x in 0..vw {
                max = max.max(at(x, y, zz));
            }
        }
        println!("slice {zz} (max {max:.3}):");
        for y in 0..vh {
            let row: String = (0..vw)
                .map(|x| {
                    let v = at(x, y, zz);
                    if v <= 1e-6 {
                        '.'
                    } else {
                        let k = ((v / max).sqrt() * 15.0).round() as usize;
                        "0123456789abcdef".as_bytes()[k.min(15)] as char
                    }
                })
                .collect();
            println!("{row}");
        }
    }
}
