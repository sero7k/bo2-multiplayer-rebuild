//! Checks the shine conversion (`fastfile_iw7::shine`): `iw7_shine
//! <map.ff> <out dir> <material substring>...` writes, for each matching
//! material, its converted normal map (as the slope it adds, red and green,
//! with the shaded normal as a lit grey in blue: light from the upper
//! left), specular map (colour, then gloss) and diffuse colour as `.png`s,
//! and prints their means.
use fastfile_iw7::map::{Iw7Map, MapSources};
use fastfile_iw7::texture;

fn png(path: &std::path::Path, w: u32, h: u32, rgba: &[u8]) {
    let f = std::fs::File::create(path).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.write_header().and_then(|mut wr| wr.write_image_data(rgba)).expect("png");
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let map = Iw7Map::load(&MapSources::beside(std::path::Path::new(&a[1]))).expect("map");
    let out = std::path::Path::new(&a[2]);
    std::fs::create_dir_all(out).expect("out");
    let mut done = 0;
    for m in &map.materials {
        if !a[3..].iter().any(|s| m.name.contains(s.as_str())) || done >= 12 {
            continue;
        }
        done += 1;
        let safe: String = m.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        match map.shine(m, 512, 256, true) {
            Ok(sh) => {
                let mut line = format!("{} [{}]", m.name, m.technique_set);
                if let Some(l) = &sh.normal {
                    let (w, h, mut px) = texture::level_rgba(l, 0).expect("bc5");
                    let (mut sx, mut sy) = (0.0f64, 0.0f64);
                    for p in px.chunks_exact_mut(4) {
                        let (x, y) = (f32::from(p[0]) / 127.5 - 1.0, f32::from(p[1]) / 127.5 - 1.0);
                        sx += f64::from(x.abs());
                        sy += f64::from(y.abs());
                        let n = [x, y, 1.0f32];
                        let len = (n[0] * n[0] + n[1] * n[1] + 1.0).sqrt();
                        let lit = ((-0.5 * n[0] + 0.5 * n[1] + 0.7 * n[2]) / len).clamp(0.0, 1.0);
                        p[2] = (lit * 255.0) as u8;
                    }
                    let n = f64::from(w * h);
                    line += &format!(" normal {w}x{h} |slope x| {:.3} |slope y| {:.3}", sx / n, sy / n);
                    png(&out.join(format!("{safe}_n.png")), w, h, &px);
                }
                if let Some(l) = &sh.specular {
                    let (w, h, px) = texture::level_rgba(l, 0).expect("bc3");
                    let n = f64::from(w * h);
                    let mean = |k: usize| px.chunks_exact(4).map(|p| f64::from(p[k])).sum::<f64>() / n;
                    line += &format!(" specular {w}x{h} rgb {:.0} {:.0} {:.0} gloss {:.0}", mean(0), mean(1), mean(2), mean(3));
                    let mut gloss = px.clone();
                    for p in gloss.chunks_exact_mut(4) {
                        p[0] = p[3];
                        p[1] = p[3];
                        p[2] = p[3];
                        p[3] = 255;
                    }
                    let mut col = px.clone();
                    for p in col.chunks_exact_mut(4) {
                        p[3] = 255;
                    }
                    png(&out.join(format!("{safe}_s.png")), w, h, &col);
                    png(&out.join(format!("{safe}_g.png")), w, h, &gloss);
                }
                if let Some(l) = &sh.colour {
                    let (w, h, px) = texture::level_rgba(l, 0).expect("bc1");
                    line += &format!(" diffuse {w}x{h}");
                    png(&out.join(format!("{safe}_c.png")), w, h, &px);
                }
                println!("{line}");
            }
            Err(e) => println!("{}: {e}", m.name),
        }
    }
}
