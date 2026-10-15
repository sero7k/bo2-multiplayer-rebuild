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
                let vision = map.vision_params();
                let sun = map.lights.iter().find(|l| l.kind == 1).map_or(0.0, |l| l.color.iter().copied().fold(0.0, f32::max));
                println!(
                    "  vision: ev {} auto {} tone map {:?} clut {:?}; exposure {:.4}, draw scale {:.5}",
                    vision.ev,
                    vision.auto,
                    (vision.black, vision.white, vision.crossover, vision.shoulder, vision.toe),
                    vision.clut,
                    vision.exposure(),
                    vision.draw_scale()
                );
                for i in 0..map.light_maps.len() {
                    match map.lightmap_page(i, 512, vision.draw_scale()) {
                        // (Exposed: the tone map's input for the mean
                        // texel and the sun at the vision's exposure.)
                        Ok(p) => println!(
                            "  page {i}: mean HDR light {:.3} (exposed {:.2}), sun peak {sun:.2} (exposed {:.1}), sun / mean light {:.1}",
                            p.mean_light,
                            p.mean_light * vision.exposure(),
                            sun * vision.exposure(),
                            sun / p.mean_light.max(1e-4)
                        ),
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
                        let pages: Vec<_> = (0..map.light_maps.len()).map(|i| map.lightmap_page(i, 2048, vision.draw_scale()).ok()).collect();
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
                // `IW7_VISION=1`: the vision's text (its tonemap, fog and
                // colour lines).
                if std::env::var("IW7_VISION").is_ok()
                    && let Some((name, text)) = &map.vision
                {
                    println!("  vision {name}:");
                    for l in text.lines().filter(|l| !l.trim().is_empty()) {
                        println!("    {}", l.trim());
                    }
                    // The colour table it names: its record and bytes.
                    let clut = text.lines().find_map(|l| {
                        let (k, v) = l.trim().split_once(' ')?;
                        k.eq_ignore_ascii_case("clut0Image").then(|| v.trim().trim_matches('"').to_owned())
                    });
                    if let Some(c) = clut {
                        let rec = map.image_info(&c).map(|r| (r.format, r.map_type, r.width, r.height, r.depth, r.level_count, r.streamed));
                        let px = map.image(&c, 4096).map(|l| (l.format, l.width, l.height, l.levels, l.data.len(), l.data.iter().take(16).copied().collect::<Vec<u8>>()));
                        println!("  clut {c}: {rec:?} {px:?}");
                    }
                }
                // `IW7_TEXSTATS=1`: the drawn materials' colour and normal
                // maps: how many at each largest size, and their bytes (as
                // stored, block compressed, with mips) at each cap.
                if std::env::var("IW7_TEXSTATS").is_ok() {
                    let mut used = std::collections::BTreeSet::new();
                    for s in &map.surfaces {
                        if let Some(m) = s.material.filter(|_| !s.hidden_brush) {
                            used.insert(m);
                        }
                    }
                    for sm in &map.static_models {
                        for s in &map.models[sm.model].surfaces {
                            if let Some(m) = s.material {
                                used.insert(m);
                            }
                        }
                    }
                    let mut images = std::collections::BTreeSet::new();
                    for &m in &used {
                        for slot in ["colorMap", "normalMap", "colorMap1", "colorMap2"] {
                            if let Some(n) = map.materials[m].texture(slot).filter(|n| !n.starts_with('$')) {
                                images.insert(n.to_owned());
                            }
                        }
                    }
                    let mut sizes = std::collections::BTreeMap::new();
                    let caps = [512u32, 1024, 2048, 4096];
                    let mut bytes = [0usize; 4];
                    for n in &images {
                        let Some(r) = map.image_info(n) else { continue };
                        let top = (0..4).rev().find(|&k| r.streams[k].0 != 0).map_or((r.width, r.height), |k| (r.streams[k].0, r.streams[k].1));
                        *sizes.entry(top.0.max(top.1)).or_insert(0usize) += 1;
                        for (c, cap) in caps.iter().enumerate() {
                            let (mut w, mut h) = (u32::from(top.0), u32::from(top.1));
                            while w.max(h) > *cap {
                                w /= 2;
                                h /= 2;
                            }
                            let levels = 32 - w.max(h).max(1).leading_zeros();
                            bytes[c] += fastfile_iw7::texture::chain_size(r.format, w, h, levels).unwrap_or(0);
                        }
                    }
                    println!("  textures: {} materials drawn, {} pictures; largest sizes {sizes:?}; MB at caps {caps:?}: {:?}", used.len(), images.len(), bytes.map(|b| b >> 20));
                }
                // `IW7_SMSTATS=1`: the placed models' baked light: how many
                // have vertex lighting (and whether its count is their
                // first LOD's vertex count), how many a lightmap place, and
                // a few values.
                if std::env::var("IW7_SMSTATS").is_ok() {
                    let half = |b: &[u8]| {
                        let h = u16::from_le_bytes([b[0], b[1]]);
                        let e = i32::from((h >> 10) & 0x1f);
                        let m = f32::from(h & 0x3ff);
                        let v = if e == 0 { m * 2f32.powi(-24) } else { (1.0 + m / 1024.0) * 2f32.powi(e - 15) };
                        if h & 0x8000 != 0 { -v } else { v }
                    };
                    let (mut with_vl, mut match_lod0, mut with_lm) = (0, 0, 0);
                    let mut lm_index = std::collections::BTreeMap::new();
                    for (i, sm) in map.static_models.iter().enumerate() {
                        let lod0: usize = map.models[sm.model].surfaces.iter().map(|s| s.verts.len()).sum();
                        if !sm.vertex_lighting.is_empty() {
                            with_vl += 1;
                            match_lod0 += usize::from(sm.vertex_lighting.len() == lod0);
                        }
                        *lm_index.entry(sm.lightmap.2).or_insert(0usize) += 1;
                        if sm.lightmap.2 != u32::MAX && sm.lightmap.1 != [0.0; 2] {
                            with_lm += 1;
                        }
                        if i < 6 || (!sm.vertex_lighting.is_empty() && i % 997 == 0) {
                            let vl: Vec<String> = sm
                                .vertex_lighting
                                .iter()
                                .take(3)
                                .map(|v| format!("vis {:02x?} amb {:?} hi {:?}", &v[0..4], [half(&v[4..]), half(&v[6..]), half(&v[8..]), half(&v[10..])], [half(&v[12..]), half(&v[14..]), half(&v[16..]), half(&v[18..])]))
                                .collect();
                            println!("    smodel {i} {} lod0 {lod0} verts, {} lighting values, lightmap {:?}: {vl:?}", map.models[sm.model].name, sm.vertex_lighting.len(), sm.lightmap);
                        }
                    }
                    println!("  smodels: {} placed, {with_vl} with vertex lighting ({match_lod0} one a first-LOD vertex), {with_lm} with a lightmap place; lightmap indices {lm_index:?}", map.static_models.len());
                }
                // `IW7_PROBES=1`: the light grid's probes: how many, their
                // bounds, and for the probes nearest a few spawn points
                // their bytes (as halves and as bytes) beside the floor's
                // HDR light there.
                if std::env::var("IW7_PROBES").is_ok() {
                    let half = |b: &[u8]| {
                        let h = u16::from_le_bytes([b[0], b[1]]);
                        let e = i32::from((h >> 10) & 0x1f);
                        let m = f32::from(h & 0x3ff);
                        let v = if e == 0 { m * 2f32.powi(-24) } else { (1.0 + m / 1024.0) * 2f32.powi(e - 15) };
                        if h & 0x8000 != 0 { -v } else { v }
                    };
                    let lp = &map.light_probes;
                    let lo: [f32; 3] = std::array::from_fn(|k| lp.iter().map(|p| p.0[k]).fold(f32::MAX, f32::min));
                    let hi: [f32; 3] = std::array::from_fn(|k| lp.iter().map(|p| p.0[k]).fold(f32::MIN, f32::max));
                    println!("  light probes: {} from {lo:?} to {hi:?}", lp.len());
                    let pages: Vec<_> = (0..map.light_maps.len()).map(|i| map.lightmap_page(i, 1024, 1.0).ok()).collect();
                    for sp in map.spawns.iter().step_by((map.spawns.len() / 6).max(1)).take(6) {
                        let Some((p, d)) = lp.iter().min_by(|a, b| {
                            let dist = |q: &[f32; 3]| (0..3).map(|k| (q[k] - sp.origin[k]).powi(2)).sum::<f32>();
                            dist(&a.0).total_cmp(&dist(&b.0))
                        }) else {
                            continue;
                        };
                        let floor = map.ground_light(&pages, &[[sp.origin[0], sp.origin[1], sp.origin[2] + 8.0]], 256.0)[0];
                        let halves: Vec<String> = d.chunks_exact(2).map(|c| format!("{:.3}", half(c))).collect();
                        println!("    spawn {:?}: probe at {p:?}; floor HDR {:?}", sp.origin, floor.map(|f| (f.base, f.directional, f.sun)));
                        println!("      halves {}", halves.join(" "));
                        println!("      bytes {:02x?}", &d[..]);
                    }
                }
                // `IW7_UNLIT=1`: the unlit materials' technique sets and
                // constants (one material each).
                if std::env::var("IW7_UNLIT").is_ok() {
                    let mut seen = std::collections::BTreeSet::new();
                    for m in &map.materials {
                        if (m.technique_set.contains("unlit") || m.technique_set.contains("mkhdr") || m.technique_set.contains("_add_")) && seen.insert(m.technique_set.clone()) {
                            println!("    {} {}: {:?}", m.technique_set, m.name, m.constants);
                        }
                    }
                }
                // `IW7_LMSTATS=1`: the light map images' value spread (the
                // primary's histogram in tenths, the secondary's format).
                if std::env::var("IW7_LMSTATS").is_ok() {
                    for names in &map.light_maps {
                        for n in names {
                            let Ok(l) = map.image(n, 1 << 14) else { continue };
                            print!("  {n}: format {} {}x{} levels {}", l.format, l.width, l.height, l.levels);
                            if matches!(l.format, 79..=81)
                                && let Some((w, h, px)) = fastfile_iw7::texture::level_rgba(&l, 0)
                            {
                                let mut hist = [0usize; 11];
                                for p in px.chunks_exact(4) {
                                    hist[usize::from(p[0]) * 10 / 255] += 1;
                                }
                                print!(" histogram {:?}", hist.map(|c| (c * 1000 / (w * h) as usize) as f32 / 10.0));
                            }
                            if l.format == 49
                                && let Some((w, h, px)) = fastfile_iw7::texture::level_rgba(&l, 0)
                            {
                                let n = (w * h) as f64;
                                let mean: [f64; 2] = std::array::from_fn(|k| px.chunks_exact(4).map(|p| f64::from(p[k])).sum::<f64>() / n);
                                print!(" mean {mean:?}");
                            }
                            println!();
                        }
                    }
                }
            }
            Err(e) => println!("{path}: {e}"),
        }
    }
}
