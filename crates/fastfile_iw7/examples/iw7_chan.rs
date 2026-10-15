//! What each channel of a map's pictures holds: `iw7_chan <map.ff> <name
//! substring>...` prints, for each image of the map's materials whose name
//! holds one of the substrings, its format and per-channel mean, spread,
//! min and max at most 256 px.
use fastfile_iw7::map::{Iw7Map, MapSources};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let map = Iw7Map::load(&MapSources::beside(std::path::Path::new(&a[1]))).expect("map");
    let mut seen = std::collections::HashSet::new();
    let mut shown = 0;
    for m in &map.materials {
        for t in &m.textures {
            let name = &t.3;
            if !a[2..].iter().any(|s| name.contains(s.as_str())) || !seen.insert(name.clone()) {
                continue;
            }
            let sampler = fastfile_iw7::texture::sampler_name(t.0).unwrap_or("?");
            match map.image(name, std::env::var("IW7_CHAN_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(256)) {
                Ok(l) => {
                    let Some((w, h, px)) = fastfile_iw7::texture::level_rgba(&l, 0) else {
                        println!("{name} [{sampler}] fmt {}: not decoded", l.format);
                        continue;
                    };
                    let n = (w * h) as f64;
                    let mut s = String::new();
                    for c in 0..4 {
                        let vals: Vec<f64> = px.chunks_exact(4).map(|p| f64::from(p[c])).collect();
                        let mean = vals.iter().sum::<f64>() / n;
                        let sd = (vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
                        let (lo, hi) = vals.iter().fold((255.0f64, 0.0f64), |(a, b), &v| (a.min(v), b.max(v)));
                        s += &format!(" {}:{mean:.0}±{sd:.0}[{lo}..{hi}]", ["r", "g", "b", "a"][c]);
                    }
                    let at = |x: u32, y: u32, c: usize| f64::from(px[((y * w + x) * 4) as usize + c]) / 255.0 * 2.0 - 1.0;
                    let cands: [(&str, fn([f64; 4]) -> (f64, f64)); 8] = [
                        ("x=A y=G", |c| (c[3], c[1])),
                        ("x=G y=A", |c| (c[1], c[3])),
                        ("x=A y=-G", |c| (c[3], -c[1])),
                        ("x=R y=G", |c| (c[0], c[1])),
                        ("x=A+G y=A-G", |c| (c[3] + c[1], c[3] - c[1])),
                        ("x=A+G y=G-A", |c| (c[3] + c[1], c[1] - c[3])),
                        ("x=A-G y=A+G", |c| (c[3] - c[1], c[3] + c[1])),
                        ("x=G-A y=A+G", |c| (c[1] - c[3], c[3] + c[1])),
                    ];
                    let mut curl = String::new();
                    let k = 4u32;
                    for (label, f) in cands {
                        let (mut sxy, mut sxx, mut syy) = (0.0f64, 0.0f64, 0.0f64);
                        let px4 = |x: u32, y: u32| f([at(x, y, 0), at(x, y, 1), at(x, y, 2), at(x, y, 3)]);
                        for y in 0..h.saturating_sub(k) {
                            for x in 0..w.saturating_sub(k) {
                                let a = px4(x, y + k).0 - px4(x, y).0;
                                let b = px4(x + k, y).1 - px4(x, y).1;
                                sxy += a * b;
                                sxx += a * a;
                                syy += b * b;
                            }
                        }
                        curl += &format!(" [{label}]:{:.2}", sxy / (sxx * syy).sqrt().max(1e-9));
                    }
                    println!("{name} [{sampler} in {}] fmt {} {w}x{h}{s} |{curl}", m.technique_set, l.format);
                }
                Err(e) => println!("{name} [{sampler}]: {e}"),
            }
            shown += 1;
            if shown > 60 {
                return;
            }
        }
    }
}
