//! What reading a map reports: `iw7_report <map.ff>...` prints each map's
//! `Iw7Map::report` lines and its static model counts.
use fastfile_iw7::map::{Iw7Map, MapSources};

fn main() {
    for path in std::env::args().skip(1) {
        let started = std::time::Instant::now();
        // (The common zones too, as the game reads the map; not with
        // `IW7_NO_COMMON`.)
        let src = MapSources::beside(std::path::Path::new(&path));
        let src = if std::env::var("IW7_NO_COMMON").is_ok() { src } else { src.with_common() };
        match Iw7Map::load(&src) {
            Ok(map) => {
                println!("{path}: read in {:.1}s", started.elapsed().as_secs_f32());
                for l in &map.report {
                    println!("  {l}");
                }
                let probe = map.probe_image.as_deref().and_then(|n| map.image_info(n).map(|r| (n, r.format, r.width, r.level_count, r.num_elements)));
                println!("  probes: {} origins, array {probe:?}", map.probe_origins.len());
                for i in 0..map.light_maps.len() {
                    match map.lightmap_page(i, 512) {
                        Ok(p) => {
                            let v = map.vision.as_ref().and_then(|(_, t)| fastfile_iw7::light::vision_exposure(t));
                            println!(
                                "  page {i}: mean light {:.3}, exposure {:.3} (pages alone {:.3}, vision alone {v:?}, log2 vision/pages {:?})",
                                p.mean_light,
                                p.exposure,
                                p.auto_exposure,
                                v.map(|v| (v / p.auto_exposure).log2())
                            )
                        }
                        Err(e) => println!("  page {i}: {e}"),
                    }
                }
                // The light volume the game's lane makes its light grid
                // from (spawns and path nodes, 1st..99th percentile, 1024
                // round, 32 x 32 x 64): its size, cover and time.
                {
                    let pts: Vec<[f32; 3]> = map.spawns.iter().map(|s| s.origin).chain(map.path_nodes.iter().map(|n| n.origin)).collect();
                    if !pts.is_empty() {
                        let bound = |k: usize, q: f32| {
                            let mut v: Vec<f32> = pts.iter().map(|p| p[k]).collect();
                            v.sort_by(f32::total_cmp);
                            v[((v.len() - 1) as f32 * q).round() as usize]
                        };
                        let lo = [(bound(0, 0.01) - 1024.0).max(map.mins[0]), (bound(1, 0.01) - 1024.0).max(map.mins[1]), bound(2, 0.01) - 192.0];
                        let hi = [(bound(0, 0.99) + 1024.0).min(map.maxs[0]), (bound(1, 0.99) + 1024.0).min(map.maxs[1]), bound(2, 0.99) + 704.0];
                        let pages: Vec<_> = (0..map.light_maps.len()).map(|i| map.lightmap_page(i, 2048).ok()).collect();
                        let t = std::time::Instant::now();
                        let vol = map.light_volume(&pages, lo, hi, [32.0, 32.0, 64.0], 640.0);
                        let found = vol.samples.iter().filter(|s| s.is_some()).count();
                        println!("  light volume: {:?} = {} points, {found} over a lit floor, in {:.2}s", vol.dims, vol.samples.len(), t.elapsed().as_secs_f32());
                    }
                }
                // Placed models twice in one place (the same model, or an
                // intact and a damaged one).
                let key = |o: [f32; 3]| o.map(|v| (v * 4.0).round() as i64);
                let mut at: std::collections::HashMap<[i64; 3], Vec<&str>> = std::collections::HashMap::new();
                for sm in &map.static_models {
                    at.entry(key(sm.origin)).or_default().push(map.models[sm.model].name.as_str());
                }
                let same: usize = at
                    .values()
                    .map(|v| {
                        let mut u = v.clone();
                        u.sort_unstable();
                        u.dedup();
                        v.len() - u.len()
                    })
                    .sum();
                let states = ["_dst", "_destroyed", "_dmg", "_damaged", "_broken"];
                let damaged: Vec<&str> = at.values().filter(|v| v.len() > 1 && v.iter().any(|n| states.iter().any(|s| n.contains(s)))).flat_map(|v| v.iter().copied()).take(8).collect();
                println!("  duplicates: {same} placed twice in one place; intact and damaged together: {damaged:?}");
                let sun = map.lights.iter().find(|l| l.kind == 1).map(|l| (l.color, l.dir));
                println!("  sun {sun:?}");
            }
            Err(e) => println!("{path}: {e}"),
        }
    }
}
