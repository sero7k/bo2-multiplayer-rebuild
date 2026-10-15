//! bo2mp: where each reflection probe's light volume sits in the probe
//! volume (`gProbeX/Y/ZArray`): `t7_pvol <zone.ff> [probes]` prints, per
//! probe, its config words 4..9 (read as the region's size and place in the
//! volume, uvw) and 33..37, and how much of that region (and of the same
//! region half the depth on: the negative directions) is lit, against a
//! frame of texels round it: a region that holds the probe's data is lit
//! inside and dark round it.
use fastfile_t7::walk::{Options, walk_with};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let want: usize = a.get(2).and_then(|v| v.parse().ok()).unwrap_or(12);
    let z = fastfile_t7::Zone::open(path).expect("zone");
    let w = walk_with(&z, Options::default());
    let g = w.capture.world.gfx_worlds.first().expect("gfxworld");
    let l = g.sun_lighting.first().expect("sun volume");
    let names = &l.probe_volumes[0];
    let im = w.capture.image(names[0].as_deref().unwrap().trim_start_matches(',')).expect("probe image");
    let (vw, vh, vd) = (im.width as usize, im.height as usize, im.depth as usize);
    let b = l.volume_buffers[0].expect("volume buffer");
    let key = u64::from_le_bytes(z.image[b + 8..b + 16].try_into().unwrap());
    let (paks, _) = fastfile_t7::xpak::XpakSet::open_dir(path.parent().unwrap()).expect("paks");
    let data = paks.read(key).expect("volume data");
    let per = (vw / 4) * (vh / 4) * 16 * vd;
    println!("volume {vw}x{vh}x{vd}");
    // One texel of texture `t` (BC6H blocks, slice by slice).
    let texel = |t: usize, x: usize, y: usize, zz: usize| -> f32 {
        let slice = (vw / 4) * (vh / 4) * 16;
        let at = t * per + zz * slice + ((y / 4) * (vw / 4) + x / 4) * 16;
        let Some(blk) = data.get(at..at + 16) else { return 0.0 };
        let mut px = [0f32; 48];
        bcdec_rs::bc6h_float(blk, &mut px, 12, false);
        let o = (y % 4) * 12 + (x % 4) * 3;
        0.2126 * px[o] + 0.7152 * px[o + 1] + 0.0722 * px[o + 2]
    };
    let (Some((cat, cn)), _, _) = (l.reflection_probes[0].1, 0, 0) else { return };
    let f = |i: usize, k: usize| f32::from_le_bytes(z.image[cat + 328 * i + 4 * k..cat + 328 * i + 4 * k + 4].try_into().unwrap());
    let u = |i: usize, k: usize| u32::from_le_bytes(z.image[cat + 328 * i + 4 * k..cat + 328 * i + 4 * k + 4].try_into().unwrap());
    for i in 0..cn.min(want) {
        let s = [f(i, 4), f(i, 5), f(i, 6)];
        let o = [f(i, 7), f(i, 8), f(i, 9)];
        // The region in texels (depth: the first half's).
        let r0 = [(o[0] * vw as f32) as i64, (o[1] * vh as f32) as i64, (o[2] * vd as f32) as i64];
        let r1 = [((o[0] + s[0]) * vw as f32) as i64, ((o[1] + s[1]) * vh as f32) as i64, ((o[2] + s[2]) * vd as f32) as i64];
        let stat = |dz: i64, inside: bool| -> (usize, usize, f32) {
            let (mut lit, mut n, mut sum) = (0usize, 0usize, 0f32);
            let pad = 3;
            for zz in (r0[2] - pad).max(0)..(r1[2] + pad).min(vd as i64 / 2) {
                for y in (r0[1] - pad).max(0)..(r1[1] + pad).min(vh as i64) {
                    for x in (r0[0] - pad).max(0)..(r1[0] + pad).min(vw as i64) {
                        let within = x >= r0[0] && x < r1[0] && y >= r0[1] && y < r1[1] && zz >= r0[2] && zz < r1[2];
                        if within != inside || (x + y + zz) % 3 != 0 {
                            continue;
                        }
                        let v = texel(0, x as usize, y as usize, (zz + dz) as usize);
                        n += 1;
                        if v > 1e-4 {
                            lit += 1;
                            sum += v;
                        }
                    }
                }
            }
            (lit, n, if lit > 0 { sum / lit as f32 } else { 0.0 })
        };
        let half = vd as i64 / 2;
        let a = stat(0, true);
        let b2 = stat(0, false);
        let c = stat(half, true);
        println!(
            "probe {i}: words 4..9 size {s:.4?} at {o:.4?} -> texels {r0:?}..{r1:?}; 33..37 {:.2} {:#x} {:#x} {:#x} {:#x}; inside {}/{} lit (mean {:.3}), frame {}/{} lit, second half inside {}/{} (mean {:.3})",
            f(i, 33),
            u(i, 34),
            u(i, 35),
            u(i, 36),
            u(i, 37),
            a.0,
            a.1,
            a.2,
            b2.0,
            b2.1,
            c.0,
            c.1,
            c.2
        );
    }
}
