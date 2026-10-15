//! A map's lightmap images as pictures: `iw7_lightmap <map.ff> <out dir>`
//! writes each `GfxLightMap` texture, a quarter size, as `.png` (BC6H as
//! `sqrt(light)`), and prints each one's channel means and peaks.
use fastfile_iw7::texture;
use fastfile_iw7::walk::walk_fitted;
use std::path::Path;

fn write_png(path: &Path, w: u32, h: u32, rgba: &[u8]) {
    let f = std::fs::File::create(path).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(rgba).unwrap();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = fastfile_iw7::Zone::open(Path::new(&a[1])).expect("zone");
    let out = Path::new(&a[2]);
    std::fs::create_dir_all(out).unwrap();
    if std::env::var("IW7_PAGE").is_ok() {
        let map = fastfile_iw7::map::Iw7Map::load(&fastfile_iw7::map::MapSources::beside(Path::new(&a[1]))).expect("map");
        for i in 0..map.light_maps.len() {
            let p = map.lightmap_page(i, 1024, map.vision_params().draw_scale()).expect("page");
            let (w, h, s) = &p.secondary;
            let mut rgba = Vec::with_capacity(s.len());
            let mut alpha = Vec::with_capacity(s.len());
            for px in s.chunks_exact(4) {
                rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
                alpha.extend_from_slice(&[px[3], px[3], px[3], 255]);
            }
            write_png(&out.join(format!("page{i}_secondary.png")), u32::from(*w), u32::from(*h), &rgba);
            write_png(&out.join(format!("page{i}_alpha.png")), u32::from(*w), u32::from(*h), &alpha);
            let (pw, ph, pp) = &p.primary;
            let g: Vec<u8> = pp.iter().flat_map(|&v| [v, v, v, 255]).collect();
            write_png(&out.join(format!("page{i}_primary.png")), u32::from(*pw), u32::from(*ph), &g);
        }
        return;
    }
    let (w, _) = walk_fitted(&zone);
    let cap = &w.cap;
    for l in &cap.light_maps {
        for id in l.textures.iter().flatten() {
            let name = cap.name_of(*id).trim_start_matches(',');
            let Some(im) = cap.images.iter().find(|i| i.name == name) else { continue };
            let Ok(levels) = texture::inline_levels(im, &zone.image) else { continue };
            let (iw, ih) = (levels.width as usize, levels.height as usize);
            // Linear float RGB per pixel.
            let mut px = vec![[0f32; 3]; iw * ih];
            match levels.format {
                94..=96 => {
                    let bw = iw / 4;
                    let mut tmp = [0f32; 48];
                    for (i, b) in levels.data.chunks_exact(16).take(bw * (ih / 4)).enumerate() {
                        bcdec_rs::bc6h_float(b, &mut tmp, 12, levels.format == 96);
                        let (bx, by) = (i % bw * 4, i / bw * 4);
                        for y in 0..4 {
                            for x in 0..4 {
                                let t = (y * 4 + x) * 3;
                                px[(by + y) * iw + bx + x] = [tmp[t], tmp[t + 1], tmp[t + 2]];
                            }
                        }
                    }
                }
                _ => {
                    let Some((_, _, rgba)) = texture::level_rgba(&levels, 0) else {
                        println!("{name}: format {} not decoded", levels.format);
                        continue;
                    };
                    for (i, p) in rgba.chunks_exact(4).enumerate() {
                        px[i] = [f32::from(p[0]) / 255.0, f32::from(p[1]) / 255.0, f32::from(p[2]) / 255.0];
                    }
                }
            }
            let (mut mean, mut peak) = ([0f64; 3], [0f32; 3]);
            for p in &px {
                for c in 0..3 {
                    mean[c] += f64::from(p[c]);
                    peak[c] = peak[c].max(p[c]);
                }
            }
            let n = px.len() as f64;
            println!("{name}: fmt {} {iw}x{ih} mean {:?} peak {peak:?}", levels.format, mean.map(|m| m / n));
            let (ow, oh) = (iw / 4, ih / 4);
            let mut rgba = vec![0u8; ow * oh * 4];
            let hdr = matches!(levels.format, 94..=96);
            for y in 0..oh {
                for x in 0..ow {
                    let p = px[(y * 4) * iw + x * 4];
                    let g = |v: f32| ((if hdr { v.max(0.0).sqrt() * 0.7 } else { v }).clamp(0.0, 1.0) * 255.0) as u8;
                    rgba[(y * ow + x) * 4..(y * ow + x) * 4 + 4].copy_from_slice(&[g(p[0]), g(p[1]), g(p[2]), 255]);
                }
            }
            write_png(&out.join(format!("{}.png", name.trim_start_matches('*'))), ow as u32, oh as u32, &rgba);
        }
    }
}
