//! Walks a Ghosts map zone and checks its world, collision and spawns as
//! `fastfile_iw6::map` reads them: `iw6_map_check <mp_map.ff>...`.
fn main() {
    for path in std::env::args().skip(1) {
        let t = std::time::Instant::now();
        let zone = match fastfile_iw6::Zone::open(std::path::Path::new(&path)) {
            Ok(z) => z,
            Err(e) => {
                println!("{path}: open: {e}");
                continue;
            }
        };
        let (cap, stop) = fastfile_iw6::walk::walk_zone(&zone);
        println!(
            "{path}: walked {}/{} assets{} [{:.1}s]",
            cap.walked,
            zone.assets.assets.len(),
            stop.map(|s| format!(", stopped at {} ({:?}): {}", s.asset_index, s.asset_type, s.error)).unwrap_or_default(),
            t.elapsed().as_secs_f32()
        );
        println!(
            "  world assets: gfx {:?} clip {:?} ents {:?} com {:?}",
            cap.gfx_worlds.iter().map(|g| (g.name.as_str(), g.vertex_count, g.surface_count, g.vertices.is_some())).collect::<Vec<_>>(),
            cap.clip_maps.iter().map(|c| (c.name.as_str(), c.info.brush_count, c.tri_count)).collect::<Vec<_>>(),
            cap.map_ents.iter().map(|m| (m.name.as_str(), m.entity_string.len())).collect::<Vec<_>>(),
            cap.com_worlds.iter().map(|c| c.name.as_str()).collect::<Vec<_>>()
        );
        let map = fastfile_iw6::map::read_map(&cap, &zone.image, &zone.assets.script_strings);
        for n in &map.notes {
            println!("  note: {n}");
        }
        if let Some(w) = &map.world {
            let vn = w.vertices.len() as u64;
            let mut bad_tris = 0usize;
            let mut tris = 0usize;
            let mut out_of_bounds = 0usize;
            let mut normal_len = (f32::MAX, 0f32);
            for v in &w.vertices {
                let l = (v.normal.iter().map(|x| x * x).sum::<f32>()).sqrt();
                normal_len = (normal_len.0.min(l), normal_len.1.max(l));
                if (0..3).any(|k| v.xyz[k] < w.mins[k] - 1.0 || v.xyz[k] > w.maxs[k] + 1.0) {
                    out_of_bounds += 1;
                }
            }
            for s in &w.surfaces {
                let base = s.base_index as usize;
                for t in 0..s.tri_count as usize {
                    tris += 1;
                    let ok = (0..3).all(|k| {
                        w.indices
                            .get(base + 3 * t + k)
                            .is_some_and(|&ix| u64::from(s.first_vertex) + u64::from(ix) < vn && ix < s.vertex_count)
                    });
                    bad_tris += usize::from(!ok);
                }
            }
            let with_mat = w.surfaces.iter().filter(|s| s.material.is_some()).count();
            println!(
                "  world {}: {} verts, {} indices, {} surfaces ({} with material), {} tris ({} bad), {} verts outside bounds {:?}..{:?}, normal length {:.3}..{:.3}",
                w.name, vn, w.indices.len(), w.surfaces.len(), with_mat, tris, bad_tris, out_of_bounds, w.mins, w.maxs, normal_len.0, normal_len.1
            );
            println!(
                "  ranges {:?}; {} static models ({} with model), {} brush models, {} sky surfaces, {} lightmaps, sky images {}",
                w.ranges,
                w.static_models.len(),
                w.static_models.iter().filter(|m| m.model.is_some()).count(),
                w.brush_models.len(),
                w.sky_surfaces.len(),
                w.lightmaps.len(),
                w.sky_images.iter().flatten().map(|i| cap.name_of(*i)).collect::<Vec<_>>().join(",")
            );
            if std::env::var_os("IW6_DUMP_ENTS").is_some() {
                for (i, b) in w.brush_models.iter().enumerate() {
                    println!("    gfx brush model {i}: {:?}..{:?} surfs {}+{}", b.mins, b.maxs, b.start_surf, b.surface_count);
                }
            }
            if let Some(at) = cap.gfx_worlds[0].smodels {
                for i in [0usize, 1, 2, 100, 1000, 5000] {
                    let d = at + 176 * i + 64;
                    let b = &zone.image[d..d + 16];
                    let halves: Vec<f32> = b.chunks(2).map(|c| fastfile_iw6::map::half_to_f32(u16::from_le_bytes([c[0], c[1]]))).collect();
                    println!("    smodel {i} groundLighting {:02x?} as halves {:?}", b, halves);
                }
            }
            for m in w.static_models.iter().take(3) {
                println!("    smodel {} at {:?} scale {} axis {:?}", m.model.map(|i| cap.name_of(i)).unwrap_or(""), m.origin, m.scale, m.axis);
            }
            if std::env::var_os("IW6_LM_STATS").is_some() {
                let mut paks = fastfile_iw6::image::ImagePaks::new(std::path::Path::new("."));
                for (_, b) in w.lightmaps.iter().take(1) {
                    let Some(rec) = b.and_then(|i| cap.images.get(i.index as usize)) else { continue };
                    let px = fastfile_iw6::image::image_pixels(rec, &zone.image, &mut paks, 4096).expect("secondary");
                    let (pw, ph) = (px.width as usize, px.height as usize);
                    for (name, rows) in [("top", 0..ph / 2), ("bottom", ph / 2..ph)] {
                        let mut ch: [Vec<f32>; 4] = Default::default();
                        for y in rows.step_by(7) {
                            for x in (0..pw).step_by(7) {
                                let o = (y * pw + x) * 8;
                                for c in 0..4 {
                                    ch[c].push(fastfile_iw6::map::half_to_f32(u16::from_le_bytes([px.data[o + 2 * c], px.data[o + 2 * c + 1]])));
                                }
                            }
                        }
                        for (c, v) in ch.iter_mut().enumerate() {
                            v.sort_by(f32::total_cmp);
                            let q = |f: f32| v[((v.len() - 1) as f32 * f) as usize];
                            println!("    lm secondary {name} ch{c}: min {:.3} p10 {:.3} p50 {:.3} p90 {:.3} p99 {:.3} max {:.3}", q(0.0), q(0.1), q(0.5), q(0.9), q(0.99), q(1.0));
                        }
                    }
                }
            }
            for (i, pair) in w.lightmaps.iter().enumerate() {
                match fastfile_iw6::map::iw3_lightmap(&cap, &zone.image, *pair) {
                    Ok(l) => {
                        let avg = |b: &[u8], step: usize, c: usize| b.iter().skip(c).step_by(step).map(|&x| x as u64).sum::<u64>() as f64 / (b.len() / step).max(1) as f64;
                        println!(
                            "    lightmap {i} as CoD4's: primary {}x{} (mean {:.0}), secondary {}x{} (mean B {:.0} G {:.0} R {:.0} A {:.0})",
                            l.primary.0, l.primary.1, avg(&l.primary.2, 1, 0), l.secondary.0, l.secondary.1,
                            avg(&l.secondary.2, 4, 0), avg(&l.secondary.2, 4, 1), avg(&l.secondary.2, 4, 2), avg(&l.secondary.2, 4, 3)
                        );
                    }
                    Err(e) => println!("    lightmap {i}: {e}"),
                }
            }
            for (a, b) in w.lightmaps.iter().take(2) {
                let d = |id: &Option<fastfile_iw6::walk::AssetId>| {
                    id.and_then(|i| cap.images.get(i.index as usize)).map(|im| {
                        format!("{} fmt {} {}x{} streamed {} map_type {}", im.name, im.format, im.width, im.height, im.streamed, im.map_type)
                    })
                };
                println!("    lightmap {:?} / {:?}", d(a), d(b));
            }
            // Material names and technique sets of the first few surfaces.
            let mut seen = std::collections::BTreeMap::new();
            for s in &w.surfaces {
                if let Some(m) = s.material.and_then(|m| cap.materials.get(m.index as usize)) {
                    let ts = m.techset.map(|t| cap.name_of(t).to_owned()).unwrap_or_default();
                    *seen.entry((m.name.clone(), ts)).or_insert(0usize) += 1;
                }
            }
            println!("  {} distinct world materials; e.g.:", seen.len());
            for ((m, ts), n) in seen.iter().take(12) {
                println!("    {m} [{ts}] x{n}");
            }
            if std::env::var_os("IW6_DUMP_MATS").is_some() {
                let mut techs = std::collections::BTreeMap::new();
                for ((_, ts), n) in &seen {
                    *techs.entry(ts.clone()).or_insert(0usize) += n;
                }
                for (t, n) in &techs {
                    println!("    techset {t} x{n}");
                }
                for ((m, _), _) in seen.iter().step_by(97).take(10) {
                    if let Some(mr) = cap.materials.iter().find(|x| &x.name == m) {
                        let tex: Vec<String> = mr
                            .textures
                            .iter()
                            .map(|t| format!("{:08x}/{}={}", t.name_hash, t.semantic, t.image.map(|i| cap.name_of(i)).unwrap_or("")))
                            .collect();
                        println!("    material {m}: {}", tex.join(" "));
                    }
                }
            }
        }
        if let Some(c) = &map.clip {
            let sides: usize = c.brushes.iter().map(|b| b.sides.len()).sum();
            let solid = c.brushes.iter().filter(|b| b.contents & 1 != 0).count();
            let max_part = c.partitions.iter().map(|p| p.1 as i64 + p.0 as i64).max().unwrap_or(0);
            let seg_hist = {
                let mut h = std::collections::BTreeMap::new();
                for p in &c.partitions {
                    *h.entry(p.2).or_insert(0usize) += 1;
                }
                h
            };
            // Triangles whose segment-based vertices are in range, and whose
            // three vertices lie within 512 units of each other.
            let (mut seg_ok, mut seg_tight, mut abs_tight) = (0usize, 0usize, 0usize);
            for p in &c.partitions {
                for t in p.1 as usize..p.1 as usize + p.0 as usize {
                    let ix: Vec<usize> = (0..3).map(|k| c.tri_indices[3 * t + k] as usize).collect();
                    let tight = |v: &[[f32; 3]]| (0..3).all(|a| (0..3).all(|b| (0..3).all(|k| (v[a][k] - v[b][k]).abs() < 512.0)));
                    let base = p.2 as usize * 1024;
                    if ix.iter().all(|&i| base + i < c.verts.len()) {
                        seg_ok += 1;
                        let v: Vec<[f32; 3]> = ix.iter().map(|&i| c.verts[base + i]).collect();
                        seg_tight += usize::from(tight(&v));
                    }
                    let v: Vec<[f32; 3]> = ix.iter().map(|&i| c.verts[i]).collect();
                    abs_tight += usize::from(tight(&v));
                }
            }
            println!("    vert segments {seg_hist:?}; with segments: {seg_ok} in range, {seg_tight} compact; without: {abs_tight} compact");
            let bad_tri = c.tri_indices.iter().filter(|&&i| usize::from(i) >= c.verts.len()).count();
            println!(
                "  clip {}: {} planes, {} materials, {} brushes ({} solid, {} sides, {} sides without plane), {} verts, {} tris ({} bad indices), {} partitions (max tri end {}), {} aabb nodes, {} cmodels, {} leaf brush nodes, {} static models, {} dyn ents",
                c.name, c.planes.len(), c.materials.len(), c.brushes.len(), solid, sides, c.missing_planes,
                c.verts.len(), c.tri_indices.len() / 3, bad_tri, c.partitions.len(), max_part, c.aabb_trees.len(), c.cmodels.len(),
                c.leaf_brush_nodes.len(), c.static_models.len(), c.dyn_ents.len()
            );
            // Tree sanity, and a floor under each spawn (a triangle or a
            // brush top within 64 units below it).
            let bad_nodes = c.aabb_trees.iter().filter(|n| {
                if n.child_count > 0 {
                    n.index < 0 || n.index as usize + n.child_count as usize > c.aabb_trees.len()
                } else {
                    n.index < 0 || n.index as usize >= c.partitions.len()
                }
            }).count();
            let mut floored = 0usize;
            for sp in &map.spawns {
                let o = sp.origin;
                let tri_floor = c.tri_indices.chunks_exact(3).any(|t| {
                    let v: Vec<[f32; 3]> = t.iter().map(|&i| c.verts[i as usize]).collect();
                    // Barycentric in xy.
                    let d = (v[1][1] - v[2][1]) * (v[0][0] - v[2][0]) + (v[2][0] - v[1][0]) * (v[0][1] - v[2][1]);
                    if d.abs() < 1e-6 {
                        return false;
                    }
                    let a = ((v[1][1] - v[2][1]) * (o[0] - v[2][0]) + (v[2][0] - v[1][0]) * (o[1] - v[2][1])) / d;
                    let b = ((v[2][1] - v[0][1]) * (o[0] - v[2][0]) + (v[0][0] - v[2][0]) * (o[1] - v[2][1])) / d;
                    let g = 1.0 - a - b;
                    if a < 0.0 || b < 0.0 || g < 0.0 {
                        return false;
                    }
                    let z = a * v[0][2] + b * v[1][2] + g * v[2][2];
                    z <= o[2] + 16.0 && z >= o[2] - 64.0
                });
                let brush_floor = c.brushes.iter().any(|b| {
                    b.contents & 0x10001 != 0
                        && (0..2).all(|k| b.mins[k] <= o[k] && o[k] <= b.maxs[k])
                        && b.maxs[2] <= o[2] + 16.0
                        && b.maxs[2] >= o[2] - 64.0
                });
                floored += usize::from(tri_floor || brush_floor);
                if !(tri_floor || brush_floor) && std::env::var_os("IW6_FLOORS").is_some() {
                    // The nearest surface below or above, any contents.
                    let mut best = f32::MAX;
                    for b in &c.brushes {
                        if (0..2).all(|k| b.mins[k] <= o[k] && o[k] <= b.maxs[k]) {
                            let d = o[2] - b.maxs[2];
                            if d.abs() < best.abs() { best = d; }
                        }
                    }
                    println!("      no floor: {} at {:?}, nearest brush top {best:+.1} away", sp.classname, o);
                }
            }
            {
                let (mut placed, mut with_lod, mut tris_total, mut distinct) = (0usize, 0usize, 0usize, std::collections::BTreeMap::new());
                for sm in &c.static_models {
                    let Some(m) = sm.model.and_then(|m| cap.xmodels.get(m.index as usize)) else { continue };
                    placed += 1;
                    if m.coll_lod < 0 {
                        continue;
                    }
                    with_lod += 1;
                    let lod = m.lods.get(m.coll_lod as usize);
                    let n: usize = lod
                        .and_then(|l| l.surfs)
                        .and_then(|s| cap.model_surfs.get(s.index as usize))
                        .map_or(0, |s| s.surfs.iter().map(|x| x.tri_count as usize).sum());
                    tris_total += n;
                    distinct.insert(m.name.clone(), (n, m.coll_lod, m.contents));
                }
                for limit in [64.0f32, 128.0, 256.0] {
                    let (mut big_tris, mut big_n, mut small_n) = (0usize, 0usize, 0usize);
                    for sm in &c.static_models {
                        let Some(m) = sm.model.and_then(|m| cap.xmodels.get(m.index as usize)) else { continue };
                        if m.coll_lod < 0 {
                            continue;
                        }
                        let n: usize = m.lods.get(m.coll_lod as usize).and_then(|l| l.surfs).and_then(|s| cap.model_surfs.get(s.index as usize)).map_or(0, |s| s.surfs.iter().map(|x| x.tri_count as usize).sum());
                        if m.bounds.1.iter().copied().fold(0.0, f32::max) > limit {
                            big_tris += n;
                            big_n += 1;
                        } else {
                            small_n += 1;
                        }
                    }
                    println!("    over {limit}: {big_n} placements, {big_tris} tris; {small_n} smaller placements");
                }
                if let Ok(want) = std::env::var("IW6_MODEL") {
                    for m in cap.xmodels.iter().filter(|m| m.name.contains(&want)).take(4) {
                        println!("    model {} coll_lod {} contents {:#x} flags {:#x} lods {:?}", m.name, m.coll_lod, m.contents, m.flags, m.lods.iter().map(|l| (l.dist, l.numsurfs, l.surf_index)).collect::<Vec<_>>());
                        if let Some(lod) = m.lods.get(m.coll_lod.max(0) as usize)
                            && let Some(ss) = lod.surfs.and_then(|s| cap.model_surfs.get(s.index as usize))
                        {
                            for (i, sf) in ss.surfs.iter().enumerate() {
                                let mat = m.materials.get(usize::from(lod.surf_index) + i).copied().flatten().and_then(|id| cap.materials.get(id.index as usize));
                                println!("      surf {i}: {} tris, material {} techset {} game_flags {:#x}", sf.tri_count, mat.map_or("", |m| m.name.as_str()), mat.and_then(|m| m.techset).map_or("", |t| cap.name_of(t)), mat.map_or(0, |m| m.game_flags));
                            }
                        }
                    }
                }
                println!("    clip static models: {placed} with a model, {with_lod} with a coll lod, {tris_total} coll tris placed, {} distinct models", distinct.len());
                for (n, v) in distinct.iter().take(6) {
                    println!("      {n}: {v:?}");
                    if let Some(m) = cap.xmodels.iter().find(|m| &m.name == n)
                        && let Some(cs) = m.coll_surfs
                    {
                        for k in 0..cs.len / 36 {
                            let at = cs.pos + 36 * k;
                            let f = |o: usize| f32::from_le_bytes(zone.image[at + o..at + o + 4].try_into().unwrap());
                            let i = |o: usize| i32::from_le_bytes(zone.image[at + o..at + o + 4].try_into().unwrap());
                            println!("        coll surf {k}: mid [{:.1} {:.1} {:.1}] half [{:.1} {:.1} {:.1}] bone {} contents {:#x} flags {:#x}", f(0), f(4), f(8), f(12), f(16), f(20), i(24), i(28), i(32));
                        }
                        let (mid, half) = m.bounds;
                        println!("        model bounds mid {mid:?} half {half:?}");
                    }
                }
            }
            println!("    aabb nodes out of range: {bad_nodes}; spawns with a floor below: {floored}/{}", map.spawns.len());
            let mut hist = std::collections::BTreeMap::new();
            for b in &c.brushes {
                *hist.entry(b.contents).or_insert(0usize) += 1;
            }
            println!("    brush contents: {:?}", hist.iter().map(|(k, v)| format!("{k:#x}:{v}")).collect::<Vec<_>>());
            let unit = c.planes.iter().filter(|p| ((p.normal.iter().map(|x| x * x).sum::<f32>()).sqrt() - 1.0).abs() < 0.01).count();
            println!("    planes with unit normals: {unit}/{}", c.planes.len());
            for (i, m) in c.cmodels.iter().take(if std::env::var_os("IW6_DUMP_ENTS").is_some() { 999 } else { 4 }).enumerate() {
                println!("    cmodel {i}: {:?}..{:?} aabb {}+{} leaf node {}", m.mins, m.maxs, m.first_coll_aabb, m.coll_aabb_count, m.leaf_brush_node);
            }
            for b in c.brushes.iter().take(2) {
                println!("    brush {:?}..{:?} contents {:#x} sides {:?} axial {:?}", b.mins, b.maxs, b.contents, b.sides, b.axial_materials);
            }
            for m in c.materials.iter().take(4) {
                println!("    clip material {m:?}");
            }
        }
        println!(
            "  entity string {} chars; {} spawn records; {} trigger models ({} hulls)",
            map.entities.len(),
            map.spawns.len(),
            map.trigger_models.len(),
            map.trigger_models.iter().map(Vec::len).sum::<usize>()
        );
        if std::env::var_os("IW6_DUMP_ENTS").is_some() {
            println!("{}", map.entity_string());
        }
        let mut classes = std::collections::BTreeMap::new();
        for s in &map.spawns {
            *classes.entry(s.classname.clone()).or_insert(0usize) += 1;
        }
        println!("    spawn classes: {classes:?}");
        if let Some(s) = map.spawns.first() {
            println!("    first: {s:?}");
        }
        let mut ent_classes = std::collections::BTreeMap::new();
        for line in map.entity_string().lines() {
            if let Some(rest) = line.strip_prefix("\"classname\" \"") {
                *ent_classes.entry(rest.trim_end_matches('"').to_owned()).or_insert(0usize) += 1;
            }
        }
        println!("    entity classes: {ent_classes:?}");
        if let Some(s) = map.sun {
            println!("  sun: dir {:?} colour {:?}", s.dir, s.color);
        }
        let raw: Vec<&str> = cap.raw_files.iter().map(|r| r.name.as_str()).filter(|n| n.contains("createart") || n.ends_with(".vision") || n.contains("_fx")).collect();
        println!("  raw files of note: {raw:?}");
        if std::env::var_os("IW6_DUMP_RAW").is_some() {
            for n in &raw {
                if n.contains("createart") || n.ends_with(".vision") && !n.contains("indoor") {
                    let t = fastfile_iw6::map::raw_file(&cap, &zone.image, n).map(|b| String::from_utf8_lossy(&b).into_owned());
                    println!("=== {n}\n{}", t.unwrap_or_else(|| "(unreadable)".into()));
                }
            }
        }
    }
}
