//! bo2mp: one Black Ops III reflection probe's six faces as a strip:
//! `t7_cube <zone.ff> <out.png> [probe] [mip]` writes faces 0..5 (as
//! stored) side by side, each labelled by its index only by position,
//! Reinhard-mapped, to see which stored face looks where.
use fastfile_t7::map::{MapSources, T7Map};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let out = std::path::Path::new(&a[2]);
    let probe: usize = a.get(3).and_then(|v| v.parse().ok()).unwrap_or(0);
    let mip: u32 = a.get(4).and_then(|v| v.parse().ok()).unwrap_or(2);
    let map = T7Map::load(&MapSources::beside(path)).expect("map");
    let faces = fastfile_t7::light::probe_faces(&map, probe, mip).expect("probe");
    let s = faces.size as usize;
    let w = s * 6 + 5 * 4;
    let mut img = vec![60u8; w * s * 3];
    for f in 0..6 {
        for y in 0..s {
            for x in 0..s {
                let c = faces.texels[f][y * s + x];
                let o = (y * w + f * (s + 4) + x) * 3;
                for k in 0..3 {
                    let v = c[k] / (1.0 + c[k]);
                    img[o + k] = (v.powf(1.0 / 2.2) * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        let mean: [f32; 3] = std::array::from_fn(|k| faces.texels[f].iter().map(|c| c[k]).sum::<f32>() / (s * s) as f32);
        println!("face {f}: mean {mean:.3?}");
    }
    let file = std::fs::File::create(out).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), w as u32, s as u32);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().expect("header").write_image_data(&img).expect("png data");
}
