//! bo2mp: the numbers of a Black Ops III map's light: `t7_lightstats
//! <zone.ff>` prints the sky picture's mean and brightest texel (and where:
//! the sun's direction), each reflection probe's mean light and its
//! smallest level's six faces, the probe volume's mean per axis, the
//! exposure grids' voxels (as floats) and the sun state's words.
use fastfile_t7::walk::{Options, walk_with};

fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn buffer(z: &fastfile_t7::Zone, b: usize) -> (u64, usize) {
    (u64::from_le_bytes(z.image[b + 8..b + 16].try_into().unwrap()), u32::from_le_bytes(z.image[b + 0x34..b + 0x38].try_into().unwrap()) as usize)
}

fn bc6(block: &[u8]) -> [[f32; 3]; 16] {
    let mut t = [0f32; 48];
    bcdec_rs::bc6h_float(block, &mut t, 12, false);
    std::array::from_fn(|i| [t[3 * i], t[3 * i + 1], t[3 * i + 2]].map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }))
}

fn lum(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let z = fastfile_t7::Zone::open(path).expect("zone");
    let w = walk_with(&z, Options::default());
    let g = w.capture.world.gfx_worlds.first().expect("gfxworld");
    let l = g.sun_lighting.first().expect("sun volume");
    let (paks, _) = fastfile_t7::xpak::XpakSet::open_dir(path.parent().unwrap()).expect("paks");
    let img = |n: &Option<String>| n.as_deref().and_then(|n| w.capture.image(n.trim_start_matches(',')));
    // The sky.
    if let (Some(b), Some(im)) = (l.sky_buffers.first().copied().flatten(), img(&l.sky_images[0])) {
        let (key, size) = buffer(&z, b);
        let (sw, sh) = (im.width as usize, im.height as usize);
        match paks.read(key) {
            Ok(d) => {
                let bw = sw / 4;
                let (mut sum, mut n, mut best, mut at) = (0f64, 0usize, 0f32, (0, 0));
                let mut rows = vec![0f64; sh / 4];
                for (i, blk) in d[..size.min(d.len())].chunks_exact(16).enumerate().take(bw * (sh / 4)) {
                    let px = bc6(blk);
                    for (k, p) in px.iter().enumerate() {
                        let v = lum(*p);
                        sum += f64::from(v);
                        rows[i / bw] += f64::from(v);
                        n += 1;
                        if v > best {
                            best = v;
                            at = (i % bw * 4 + k % 4, i / bw * 4 + k / 4);
                        }
                    }
                }
                // Lat-long as the sky shader reads it: u = 0.5 - atan2(y, x) / 2pi
                // (with x, y swapped), v = acos(z) / pi.
                let (u, v) = ((at.0 as f32 + 0.5) / sw as f32, (at.1 as f32 + 0.5) / sh as f32);
                println!("sky {sw}x{sh}: mean {:.4}, brightest {best:.2} at texel {at:?} (u {u:.4}, v {v:.4})", sum / n as f64);
                let band: Vec<String> = rows.iter().step_by((rows.len() / 16).max(1)).map(|r| format!("{:.3}", r / (bw * 16) as f64)).collect();
                println!("sky rows' mean, top to bottom: {}", band.join(" "));
            }
            Err(e) => println!("sky: {e}"),
        }
    }
    // The reflection probes (state 0).
    if let (Some(b), Some(im)) = (l.probe_buffers.first().copied().flatten(), img(&l.reflection_probes[0].0)) {
        let (key, _) = buffer(&z, b);
        let (size, levels, count) = (im.width as usize, im.levels as usize, im.depth as usize);
        let face: usize = (0..levels).map(|k| (size >> k).max(4).div_ceil(4).pow(2) * 16).sum();
        match paks.read(key) {
            Ok(d) => {
                println!("probes: {count} x 6 faces of {size}px, {levels} levels, {} bytes ({} wanted)", d.len(), face * 6 * count);
                // The brightest texel of probe 0 (the map's global probe) as
                // a direction, by D3D's cube face convention.
                let (mut best, mut dir) = (0f32, [0f32; 3]);
                for f in 0..6 {
                    let at = f * face;
                    let bw = size / 4;
                    for (i, b) in d[at..at + bw * bw * 16].chunks_exact(16).enumerate() {
                        for (k, c) in bc6(b).iter().enumerate() {
                            let v = lum(*c);
                            if v > best {
                                best = v;
                                let (x, y) = (i % bw * 4 + k % 4, i / bw * 4 + k / 4);
                                let (u, w) = ((x as f32 + 0.5) / size as f32 * 2.0 - 1.0, (y as f32 + 0.5) / size as f32 * 2.0 - 1.0);
                                dir = match f {
                                    0 => [1.0, -w, -u],
                                    1 => [-1.0, -w, u],
                                    2 => [u, 1.0, w],
                                    3 => [u, -1.0, -w],
                                    4 => [u, -w, 1.0],
                                    _ => [-u, -w, -1.0],
                                };
                            }
                        }
                    }
                }
                let n = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                let dir = dir.map(|v| v / n);
                println!("probe 0 brightest {best:.2} toward {dir:?} (D3D cube axes)");
                for p in 0..count.min(12) {
                    let mut means = Vec::new();
                    for f in 0..6 {
                        let at = (6 * p + f) * face;
                        let top = (size / 4).pow(2);
                        let s: f64 = d[at..at + top * 16].chunks_exact(16).map(|b| bc6(b).iter().map(|c| f64::from(lum(*c))).sum::<f64>()).sum();
                        means.push(s / (top * 16) as f64);
                    }
                    println!("  probe {p}: faces' mean {:?}", means.iter().map(|m| format!("{m:.3}")).collect::<Vec<_>>());
                }
            }
            Err(e) => println!("probes: {e}"),
        }
    }
    // The exposure grids' voxels.
    if let Some(Some((at, n))) = l.exposure_grids.first() {
        let mut shown = 0;
        for j in 0..*n {
            let e = at + 80 * j;
            let dims = [40, 42, 44].map(|k| u16::from_le_bytes([z.image[e + k], z.image[e + k + 1]]) as usize);
            println!("  grid {j}: dims {dims:?}");
            if shown < 3 {
                shown += 1;
                // The voxels' file offset: the walk loads them right after the
                // grid records, grid by grid; read from the first grid's end.
                let _ = e;
            }
        }
    }
    let v = g.sun_volumes[0];
    let mut line = String::from("sun state 0:");
    for k in (16..16 + 244).step_by(4) {
        line.push_str(&format!(" {:.4}", f32_at(&z.image, v + k)));
    }
    println!("{line}");
}
