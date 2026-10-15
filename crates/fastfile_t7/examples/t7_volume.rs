//! bo2mp: a Black Ops III map's probe volume (`gProbeX/Y/ZArray`, three 3D
//! BC6H textures in one streamed buffer): `t7_volume <zone.ff> <out.png>
//! [axis 0..2] [step]` writes a montage of every `step`th depth slice of one
//! axis's texture (luminance, Reinhard), and prints per slice the bounds of
//! the texels that are not black.
use fastfile_t7::walk::{Options, walk_with};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let out = std::path::Path::new(&a[2]);
    let axis: usize = a.get(3).and_then(|v| v.parse().ok()).unwrap_or(0);
    let step: usize = a.get(4).and_then(|v| v.parse().ok()).unwrap_or(8);
    let z = fastfile_t7::Zone::open(path).expect("zone");
    let w = walk_with(&z, Options::default());
    let g = w.capture.world.gfx_worlds.first().expect("gfxworld");
    let l = g.sun_lighting.first().expect("sun volume");
    let names = &l.probe_volumes[0];
    let im = w.capture.image(names[0].as_deref().unwrap().trim_start_matches(',')).expect("probe image");
    let (vw, vh, vd) = (im.width as usize, im.height as usize, im.depth as usize);
    let b = l.volume_buffers[0].expect("volume buffer");
    let key = u64::from_le_bytes(z.image[b + 8..b + 16].try_into().unwrap());
    let size = u32::from_le_bytes(z.image[b + 0x34..b + 0x38].try_into().unwrap()) as usize;
    let (paks, _) = fastfile_t7::xpak::XpakSet::open_dir(path.parent().unwrap()).expect("paks");
    let data = paks.read(key).expect("volume data");
    let per = (vw / 4) * (vh / 4) * 16 * vd;
    println!("volume {vw}x{vh}x{vd}, key {key:#x}, size {size:#x}, read {:#x}, per texture {per:#x}", data.len());
    let tex = &data[axis * per..(axis + 1) * per];
    let slices: Vec<usize> = (0..vd).step_by(step.max(1)).collect();
    let cols = 10usize;
    let rows = slices.len().div_ceil(cols);
    let (mw, mh) = (cols * (vw + 2), rows * (vh + 2));
    let mut img = vec![40u8; mw * mh * 3];
    let slice_bytes = (vw / 4) * (vh / 4) * 16;
    for zz in 0..vd {
        let s = &tex[zz * slice_bytes..(zz + 1) * slice_bytes];
        let mut px = vec![[0f32; 3]; vw * vh];
        let mut blk = [0f32; 48];
        for (i, c) in s.chunks_exact(16).enumerate() {
            let (bx, by) = (i % (vw / 4) * 4, i / (vw / 4) * 4);
            bcdec_rs::bc6h_float(c, &mut blk, 12, false);
            for y in 0..4 {
                for x in 0..4 {
                    let o = y * 12 + x * 3;
                    px[(by + y) * vw + bx + x] = [blk[o], blk[o + 1], blk[o + 2]];
                }
            }
        }
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        let mut sum = 0f64;
        let mut n = 0usize;
        for y in 0..vh {
            for x in 0..vw {
                let p = px[y * vw + x];
                let lum = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
                if lum > 1e-5 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                    sum += f64::from(lum);
                    n += 1;
                }
            }
        }
        if n > 0 {
            println!("slice {zz}: {n} lit, x {x0}..{x1}, y {y0}..{y1}, mean {:.4}", sum / n as f64);
        } else {
            println!("slice {zz}: black");
        }
        if let Some(k) = slices.iter().position(|&s| s == zz) {
            let (ox, oy) = (k % cols * (vw + 2), k / cols * (vh + 2));
            for y in 0..vh {
                for x in 0..vw {
                    let p = px[y * vw + x];
                    let o = ((oy + y) * mw + ox + x) * 3;
                    for c in 0..3 {
                        let v = p[c] / (1.0 + p[c]);
                        img[o + c] = (v.powf(1.0 / 2.2) * 255.0).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
    }
    let f = std::fs::File::create(out).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), mw as u32, mh as u32);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().expect("header").write_image_data(&img).expect("png data");
}
