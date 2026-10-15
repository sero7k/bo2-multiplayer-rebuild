//! A Black Ops III map's materials in Black Ops II's form
//! (`fastfile_t7::look`): `t7_look <map.ff> [out dir] [name substring]`
//! prints, per technique set, how many materials got which maps and what
//! they used (tint, metal, gloss map, occlusion, detail normals), and with
//! an out dir writes the first matching materials' colour, normal and
//! specular maps as PNGs (`T7_LOOK_MAX=<n>` of them, default 6).
use fastfile_t7::look::{Cover, Look};
use fastfile_t7::map::{MapSources, T7Map};
use std::collections::BTreeMap;

fn png(path: &std::path::Path, l: &fastfile_t7::texture::ImageLevels) {
    let Some((w, h, px)) = fastfile_t7::texture::level_rgba(l, 0) else { return };
    let f = std::fs::File::create(path).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().and_then(|mut w| w.write_image_data(&px)).expect("png data");
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let map = T7Map::load(&MapSources::beside(std::path::Path::new(&a[1]))).expect("map");
    let out = a.get(2).map(std::path::PathBuf::from);
    let want = a.get(3).cloned().unwrap_or_default();
    let max: usize = std::env::var("T7_LOOK_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(6);
    for l in &map.report {
        println!("{l}");
    }
    let mut by_ts: BTreeMap<String, [usize; 9]> = BTreeMap::new();
    let mut written = 0;
    let mut failures = Vec::new();
    for m in &map.materials {
        let ts = m.technique_set.split('#').next().unwrap_or_default().to_owned();
        let cover = if ts.contains("decal") || ts.contains("alphatest") || ts.contains("vegetation") || ts.contains("transparent") || ts.contains("glass") {
            Cover::Alpha
        } else {
            Cover::Opaque
        };
        let started = std::time::Instant::now();
        let r = map.look(m, 512, 256, cover);
        let e = by_ts.entry(ts).or_default();
        e[0] += 1;
        match r {
            Ok(Look { colour, normal, specular, used }) => {
                e[1] += usize::from(colour.is_some());
                e[2] += usize::from(normal.is_some());
                e[3] += usize::from(specular.is_some());
                for k in 0..5 {
                    e[4 + k] += usize::from(used[k]);
                }
                if let Some(dir) = &out
                    && written < max
                    && m.name.contains(&want)
                {
                    std::fs::create_dir_all(dir).expect("out dir");
                    let safe: String = m.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
                    for (k, l) in [("c", &colour), ("n", &normal), ("s", &specular)] {
                        if let Some(l) = l {
                            png(&dir.join(format!("{safe}_{k}.png")), l);
                        }
                    }
                    if let Ok(c) = map.rgba(m.texture("colorMap").unwrap_or_default(), 512) {
                        let l = fastfile_t7::texture::ImageLevels { format: 28, width: c.w, height: c.h, levels: 1, data: c.px, part: None };
                        png(&dir.join(format!("{safe}_orig.png")), &l);
                    }
                    println!(
                        "wrote {} ({:.0} ms): binds {:?}, constants {:?}",
                        m.name,
                        started.elapsed().as_secs_f64() * 1000.0,
                        m.bound,
                        m.constants.iter().filter(|(k, _)| !k.starts_with("debug")).collect::<BTreeMap<_, _>>()
                    );
                    written += 1;
                }
            }
            Err(err) => {
                if failures.len() < 12 {
                    failures.push(format!("{}: {err}", m.name));
                }
            }
        }
    }
    println!("technique set: materials, colour, normal, specular | tinted, metal, gloss map, ao, detail");
    for (ts, e) in &by_ts {
        println!("{:5} {:5} {:5} {:5} | {:4} {:4} {:4} {:4} {:4}  {ts}", e[0], e[1], e[2], e[3], e[4], e[5], e[6], e[7], e[8]);
    }
    for f in failures {
        println!("failed: {f}");
    }
}
