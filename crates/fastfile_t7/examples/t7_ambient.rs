//! bo2mp: a Black Ops III map's baked ambient light seen from above, to
//! check how the reflection probe configs and the probe volume are read:
//! `t7_ambient <zone.ff> <out.png> <z> [x0 y0 x1 y1]` writes, for every
//! pixel over the box (default: the map's bounds), the up-facing ambient at
//! height z (`fastfile_t7::light::Bake`), and prints the configs.
use fastfile_t7::light::Bake;
use fastfile_t7::map::{MapSources, T7Map};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let out = std::path::Path::new(&a[2]);
    let z: f32 = a.get(3).and_then(|v| v.parse().ok()).unwrap_or(200.0);
    let map = T7Map::load(&MapSources::beside(path)).expect("map");
    let bake = Bake::read(&map).expect("light");
    for line in bake.describe().lines().take(40) {
        println!("{line}");
    }
    let f = |i: usize, d: f32| a.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
    let (x0, y0, x1, y1) = (f(4, bake.bounds[0][0]), f(5, bake.bounds[0][1]), f(6, bake.bounds[1][0]), f(7, bake.bounds[1][1]));
    let w = 512usize;
    let h = ((w as f32) * (y1 - y0).abs() / (x1 - x0).abs().max(1.0)).round().clamp(16.0, 2048.0) as usize;
    let mut img = vec![0u8; w * h * 3];
    let mut peak = 0f32;
    for py in 0..h {
        for px in 0..w {
            let x = x0 + (x1 - x0) * (px as f32 + 0.5) / w as f32;
            let y = y1 - (y1 - y0) * (py as f32 + 0.5) / h as f32;
            let c = bake.ambient([x, y, z], [0.0, 0.0, 1.0]);
            peak = peak.max(c[0].max(c[1]).max(c[2]));
            for k in 0..3 {
                let v = c[k] / (1.0 + c[k]);
                img[(py * w + px) * 3 + k] = (v.powf(1.0 / 2.2) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    println!("ambient image {w}x{h} over {x0} {y0} .. {x1} {y1} at z {z}: brightest {peak:.3}");
    let file = std::fs::File::create(out).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().expect("header").write_image_data(&img).expect("png data");
}
