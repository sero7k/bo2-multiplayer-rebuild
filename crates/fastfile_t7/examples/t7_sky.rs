//! bo2mp: a Black Ops III map's sky picture as the sky shader reads it:
//! `t7_sky <zone.ff> <out.png>` writes the lat-long picture (1024 x 512,
//! Reinhard over the probes' units) with the sun volume's direction toward
//! the sun marked red, and prints the brightest small spots above the
//! horizon (the sun disc's candidates) with their azimuth and elevation.
use fastfile_t7::light::Bake;
use fastfile_t7::map::{MapSources, T7Map};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let map = T7Map::load(&MapSources::beside(std::path::Path::new(&a[1]))).expect("map");
    let bake = Bake::read(&map).expect("light");
    let sky = bake.sky.as_ref().expect("sky");
    let (w, h) = (1024usize, 512usize);
    let k = bake.probe_units.max(1e-3);
    let mut img = vec![0u8; w * h * 3];
    let dir = |x: usize, y: usize| {
        let phi = (0.5 - (x as f32 + 0.5) / w as f32) * std::f32::consts::TAU;
        let theta = (y as f32 + 0.5) / h as f32 * std::f32::consts::PI;
        [theta.sin() * phi.sin(), theta.sin() * phi.cos(), theta.cos()]
    };
    for y in 0..h {
        for x in 0..w {
            let c = sky.at(dir(x, y));
            for j in 0..3 {
                let v = c[j] / k / (1.0 + c[j] / k);
                img[(y * w + x) * 3 + j] = (v.powf(1.0 / 2.2) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    let mark = |img: &mut Vec<u8>, d: [f32; 3], rgb: [u8; 3]| {
        let u = 0.5 - d[0].atan2(d[1]) / std::f32::consts::TAU;
        let v = d[2].clamp(-1.0, 1.0).acos() / std::f32::consts::PI;
        let (cx, cy) = ((u.rem_euclid(1.0) * w as f32) as i32, (v * h as f32) as i32);
        for dy in -6..=6i32 {
            for dx in -6..=6i32 {
                if dx.abs() == 6 || dy.abs() == 6 {
                    let (x, y) = ((cx + dx).rem_euclid(w as i32) as usize, (cy + dy).clamp(0, h as i32 - 1) as usize);
                    img[(y * w + x) * 3..(y * w + x) * 3 + 3].copy_from_slice(&rgb);
                }
            }
        }
    };
    if let Some(s) = map.sun {
        let (p, yw) = (s.pitch.to_radians(), s.yaw.to_radians());
        let fwd = [p.cos() * yw.cos(), p.cos() * yw.sin(), -p.sin()];
        mark(&mut img, fwd.map(|v| -v), [255, 0, 0]);
        mark(&mut img, [fwd[0], fwd[1], -fwd[2]], [0, 255, 0]);
        println!("sun pitch {} yaw {}: toward the sun (-forward) {:.3?}", s.pitch, s.yaw, fwd.map(|v| -v));
    }
    // Spots: the brightest 1-degree cones above the horizon, kept apart.
    let mut spots: Vec<(f32, [f32; 3])> = Vec::new();
    for y in 0..h / 2 {
        for x in (0..w).step_by(2) {
            let d = dir(x, y);
            let c = sky.at(d);
            let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            spots.push((l, d));
        }
    }
    spots.sort_by(|p, q| q.0.total_cmp(&p.0));
    let mut kept: Vec<(f32, [f32; 3])> = Vec::new();
    for s in spots {
        if kept.iter().all(|k| k.1[0] * s.1[0] + k.1[1] * s.1[1] + k.1[2] * s.1[2] < 0.98) {
            kept.push(s);
            if kept.len() == 5 {
                break;
            }
        }
    }
    for (i, (l, d)) in kept.iter().enumerate() {
        let az = d[1].atan2(d[0]).to_degrees();
        let el = d[2].asin().to_degrees();
        println!("spot {i}: {l:.1} toward {d:.3?} azimuth {az:.1} elevation {el:.1}");
        mark(&mut img, *d, [255, 255, 0]);
    }
    let file = std::fs::File::create(&a[2]).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().expect("header").write_image_data(&img).expect("png data");
}
