//! Decode raw BCn blocks to PNG under each candidate format, to tell an
//! image's format byte apart: `jup_bcn <raw.bin> <width> <height> <out prefix> [max side]`.
use std::io::Write;

fn decode(fmt: &str, data: &[u8], w: usize, h: usize) -> Option<Vec<u8>> {
    let (bs, f): (usize, fn(&[u8], &mut [u8], usize)) = match fmt {
        "bc1" => (8, |s, d, p| bcdec_rs::bc1(s, d, p)),
        "bc3" => (16, |s, d, p| bcdec_rs::bc3(s, d, p)),
        "bc4" => (8, |s, d, p| {
            let mut t = [0u8; 16];
            bcdec_rs::bc4(s, &mut t, 4, false);
            for y in 0..4 {
                for x in 0..4 {
                    let v = t[y * 4 + x];
                    d[y * p + x * 4..y * p + x * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                }
            }
        }),
        "bc5" => (16, |s, d, p| {
            let mut t = [0u8; 32];
            bcdec_rs::bc5(s, &mut t, 8, false);
            for y in 0..4 {
                for x in 0..4 {
                    let (r, g) = (t[y * 8 + x * 2], t[y * 8 + x * 2 + 1]);
                    d[y * p + x * 4..y * p + x * 4 + 4].copy_from_slice(&[r, g, 128, 255]);
                }
            }
        }),
        "bc7" => (16, |s, d, p| bcdec_rs::bc7(s, d, p)),
        _ => return None,
    };
    let (bw, bh) = (w.div_ceil(4), h.div_ceil(4));
    if data.len() < bw * bh * bs {
        return None;
    }
    let mut out = vec![0u8; w * h * 4];
    let mut tmp = [0u8; 64];
    for by in 0..bh {
        for bx in 0..bw {
            let s = &data[(by * bw + bx) * bs..][..bs];
            f(s, &mut tmp, 16);
            for y in 0..4 {
                for x in 0..4 {
                    let (px, py) = (bx * 4 + x, by * 4 + y);
                    if px < w && py < h {
                        out[(py * w + px) * 4..][..4].copy_from_slice(&tmp[y * 16 + x * 4..][..4]);
                    }
                }
            }
        }
    }
    Some(out)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).expect("raw");
    let (w, h): (usize, usize) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let max: usize = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(128);
    for fmt in ["bc1", "bc3", "bc4", "bc5", "bc7"] {
        let Some(px) = decode(fmt, &data, w, h) else { continue };
        // Smoothness: mean absolute difference to the right neighbour (a
        // wrong format decodes to noise), and the mean of each channel.
        let (mut diff, mut n) = (0u64, 0u64);
        let mut mean = [0u64; 4];
        for y in 0..h {
            for x in 0..w {
                let p = &px[(y * w + x) * 4..][..4];
                for c in 0..4 {
                    mean[c] += u64::from(p[c]);
                }
                if x + 1 < w {
                    let q = &px[(y * w + x + 1) * 4..][..4];
                    diff += (0..3).map(|c| u64::from(p[c].abs_diff(q[c]))).sum::<u64>();
                    n += 3;
                }
            }
        }
        let np = (w * h) as u64;
        println!(
            "{fmt}: neighbour diff {:.2}, mean rgba {:?}",
            diff as f64 / n.max(1) as f64,
            mean.map(|m| m / np)
        );
        let step = w.div_ceil(max).max(1);
        let (ow, oh) = (w / step, h / step);
        let mut small = Vec::with_capacity(ow * oh * 4);
        for y in 0..oh {
            for x in 0..ow {
                small.extend_from_slice(&px[((y * step) * w + x * step) * 4..][..4]);
            }
        }
        let path = format!("{}_{fmt}.png", a[4]);
        let f = std::fs::File::create(&path).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&small).unwrap();
        let _ = std::io::stdout().flush();
        println!("{path}");
    }
}
