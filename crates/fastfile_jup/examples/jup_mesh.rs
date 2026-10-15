//! A part zone's meshes: `jup_mesh <zone .ff | decompressed .zone> [out.obj]`
//! (an `.ff` needs the install's Oodle DLL: `JUP_OODLE_DLL` or next to it).
//! Prints each surface, checks the packed triangles, and scores tangent
//! frame decodings against the triangles' own normals.
use std::path::Path;

use fastfile_jup::{surfs, zone};

pub fn load_zone(path: &Path) -> Vec<u8> {
    if path.extension().is_some_and(|e| e == "zone") {
        return std::fs::read(path).expect("zone");
    }
    let install = path.parent().and_then(Path::parent).unwrap_or(Path::new("."));
    let oodle = fastfile_jup::oodle::load(install).expect("oodle");
    fastfile_jup::read_zone(path, oodle.as_ref()).expect("ff").1
}

fn norm(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-12);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Candidate decodings of a packed tangent frame word to a normal.
fn candidates(w: u32) -> Vec<(&'static str, [f32; 3])> {
    let u10 = |s: u32| ((w >> s) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0;
    let mut out = vec![("10:10:10 n", [u10(0), u10(10), u10(20)])];
    // Quaternion: top 2 bits the dropped (largest) component, three 10-bit
    // components in [-1/sqrt2, 1/sqrt2]; normal = q's rotated z, x or y.
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let c = |sh: u32| (((w >> sh) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0) * s;
    let (a, b, d) = (c(0), c(10), c(20));
    let m = (1.0 - a * a - b * b - d * d).max(0.0).sqrt();
    let idx = (w >> 30) as usize;
    let mut q = [0.0f32; 4];
    let mut k = 0;
    for (i, v) in q.iter_mut().enumerate() {
        if i == idx {
            *v = m;
        } else {
            *v = [a, b, d][k];
            k += 1;
        }
    }
    let [x, y, z, ww] = q;
    let axis_x = [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + ww * z), 2.0 * (x * z - ww * y)];
    let axis_y = [2.0 * (x * y - ww * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + ww * x)];
    let axis_z = [2.0 * (x * z + ww * y), 2.0 * (y * z - ww * x), 1.0 - 2.0 * (x * x + y * y)];
    out.push(("quat x", axis_x));
    out.push(("quat y", axis_y));
    out.push(("quat z", axis_z));
    out
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let z = load_zone(Path::new(&a[1]));
    let h = zone::parse_header(&z).expect("header");
    println!("assets {:?} data@{:#x}", h.assets.iter().map(|x| x.kind).collect::<Vec<_>>(), h.data_start);
    let rec = surfs::parse(&z, h.data_start).expect("surfs");
    println!("surfs {:016x} flags {:08x} {} surfaces blob {:#x}+{:#x} end {:#x}", rec.hash, rec.flags, rec.surfaces.len(), rec.blob.0, rec.blob.1, rec.end);
    let mut obj = String::new();
    let mut base = 1usize;
    for (i, s) in rec.surfaces.iter().enumerate() {
        println!(
            "  surface {i}: flags {:04x} verts {} tris {} packets {} streams {:?} centre {:?} extents {:?} ranges {:?}",
            s.flags, s.vert_count, s.tri_count, s.packet_count, s.streams, s.centre, s.extents, s.ranges
        );
        let g = match surfs::geometry(&z, &rec, i) {
            Ok(g) => g,
            Err(e) => {
                println!("    geometry: {e}");
                continue;
            }
        };
        // Face normals accumulated per vertex.
        let mut acc = vec![[0.0f32; 3]; g.positions.len()];
        let mut degenerate = 0;
        for t in g.indices.chunks_exact(3) {
            let [p0, p1, p2] = [g.positions[t[0] as usize], g.positions[t[1] as usize], g.positions[t[2] as usize]];
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                degenerate += 1;
            }
            for &v in t {
                for c in 0..3 {
                    acc[v as usize][c] += n[c];
                }
            }
        }
        let used = acc.iter().filter(|v| v.iter().any(|x| *x != 0.0)).count();
        println!("    {} indices ({} triangles, {} degenerate), {} of {} vertices used", g.indices.len(), g.indices.len() / 3, degenerate, used, g.positions.len());
        if !g.tangent_frames.is_empty() {
            let mut scores: Vec<(&str, f64)> = Vec::new();
            for (v, &w) in g.tangent_frames.iter().enumerate() {
                let f = norm(acc[v]);
                for (k, (name, n)) in candidates(w).into_iter().enumerate() {
                    let d = f64::from(f[0] * n[0] + f[1] * n[1] + f[2] * n[2]);
                    if scores.len() <= k {
                        scores.push((name, 0.0));
                    }
                    scores[k].1 += d;
                }
            }
            let n = g.tangent_frames.len() as f64;
            println!("    normal agreement: {}", scores.iter().map(|(k, s)| format!("{k} {:.3}", s / n)).collect::<Vec<_>>().join(", "));
            println!("    frames {:08x?}", &g.tangent_frames[..4.min(g.tangent_frames.len())]);
        }
        let mn = g.positions.iter().fold([f32::MAX; 3], |m, p| [m[0].min(p[0]), m[1].min(p[1]), m[2].min(p[2])]);
        let mx = g.positions.iter().fold([f32::MIN; 3], |m, p| [m[0].max(p[0]), m[1].max(p[1]), m[2].max(p[2])]);
        println!("    bounds {mn:?} .. {mx:?}");
        for p in &g.positions {
            obj += &format!("v {} {} {}\n", p[0], p[1], p[2]);
        }
        for t in g.indices.chunks_exact(3) {
            obj += &format!("f {} {} {}\n", base + t[0] as usize, base + t[1] as usize, base + t[2] as usize);
        }
        base += g.positions.len();
    }
    if let Some(out) = a.get(2) {
        std::fs::write(out, obj).expect("obj");
    }
}
