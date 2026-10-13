//! bo2mp offline: a CoD4 / World at War map's collision traced through its
//! trees and through flat lists (every world brush, every triangle root),
//! with the same random player-sized sweeps; any difference is printed.
//! `legacy_clip_bench <zone.ff>...`
use asset_world::{ClipBspLeaf, ClipBspNode};

fn main() {
    for arg in std::env::args().skip(1) {
        let path = std::path::Path::new(&arg);
        let bytes = std::fs::read(path).unwrap();
        let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        drop(bytes);
        let zone = if version == 0x183 {
            t4::load_map(path).unwrap().zone
        } else {
            let data = iw3::fastfile::load(path).unwrap();
            iw3::zone::Zone::parse(&data, iw3::zone::ParseOptions::default()).unwrap()
        };
        let clip = zone.clip_map().expect("clipMap");
        let tree = asset_world::world_iw3::build_iw3_clip_collision(clip).unwrap();
        let mut flat = tree.clone();
        let m0 = flat.cmodels[0];
        flat.nodes = vec![ClipBspNode { plane: [0.0, 0.0, 1.0, -1.0e9], children: [-1, -1] }];
        flat.leaves = vec![ClipBspLeaf { first_brush: m0.first_brush, num_brushes: m0.num_brushes, first_coll_aabb_index: 0, coll_aabb_count: 0 }];
        std::sync::Arc::make_mut(&mut flat.mesh).aabb_roots = asset_world::world_iw3::legacy_aabb_roots(clip);
        let (lo, hi) = (m0_bounds(&tree).0, m0_bounds(&tree).1);
        let mut seed = 0x1234_5678u32;
        let mut rnd = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32
        };
        let mut sweeps = Vec::new();
        for _ in 0..20000 {
            let a = [lo[0] + rnd() * (hi[0] - lo[0]), lo[1] + rnd() * (hi[1] - lo[1]), lo[2] + rnd() * (hi[2] - lo[2])];
            let r = 64.0 + rnd() * 512.0;
            // Half drop to the floor below; half move about.
            let b = if rnd() < 0.5 {
                [a[0], a[1], lo[2] - 64.0]
            } else {
                [a[0] + (rnd() - 0.5) * r, a[1] + (rnd() - 0.5) * r, a[2] - rnd() * r * 0.5]
            };
            sweeps.push((a, b));
        }
        let run = |c: &asset_world::ClipCollision| {
            let t = std::time::Instant::now();
            let hits: Vec<_> = sweeps
                .iter()
                .map(|&(a, b)| c.sweep_box(a, b, [-15.0, -15.0, 0.0], [15.0, 15.0, 70.0], asset_world::MASK_PLAYER_SOLID))
                .collect();
            (hits, t.elapsed().as_secs_f64() * 1e6 / sweeps.len() as f64)
        };
        let (th, tt) = run(&tree);
        let (fh, ft) = run(&flat);
        let mut diff = 0;
        let mut hit = 0;
        for (i, (a, b)) in th.iter().zip(&fh).enumerate() {
            hit += usize::from(b.fraction < 1.0);
            if (a.fraction - b.fraction).abs() > 1e-4 || a.startsolid != b.startsolid {
                if diff < 5 {
                    println!("  differs {i}: tree {} {} flat {} {}", a.fraction, a.startsolid, b.fraction, b.startsolid);
                }
                diff += 1;
            }
        }
        println!(
            "{}: {} sweeps, {hit} hit; tree {tt:.1} us a sweep, flat {ft:.1} us ({:.1}x); {diff} differ",
            path.file_name().unwrap().to_string_lossy(),
            sweeps.len(),
            ft / tt
        );
    }
}

fn m0_bounds(c: &asset_world::ClipCollision) -> ([f32; 3], [f32; 3]) {
    let m = c.cmodels[0];
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for &b in &c.leafbrushes[m.first_brush as usize..][..usize::from(m.num_brushes)] {
        let p = &c.brushes[usize::from(b)].planes;
        for k in 0..3 {
            lo[k] = lo[k].min(-p[k * 2 + 1][3]);
            hi[k] = hi[k].max(p[k * 2][3]);
        }
    }
    (lo, hi)
}
