//! Texture coverage of a Ghosts map's world: for every material its world
//! surfaces use, whether a colour map is named and whether its pixels read
//! (inline or from the `imagefile<N>.pak` beside the zone), weighted by the
//! triangles drawn with it. `iw6_material_census <mp_map.ff>...`
//! (`IW6_CENSUS_LIST=1` lists the untextured materials, `IW6_CENSUS_MAX`
//! the picture size cap, default 512).
use std::collections::{BTreeMap, HashMap};

const COLOR_MAP_HASH: u32 = 0xa0ab_1041;

fn main() {
    let max: u32 = std::env::var("IW6_CENSUS_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(512);
    let list = std::env::var_os("IW6_CENSUS_LIST").is_some();
    for path in std::env::args().skip(1) {
        let p = std::path::Path::new(&path);
        let zone = match fastfile_iw6::Zone::open(p) {
            Ok(z) => z,
            Err(e) => {
                println!("{path}: open: {e}");
                continue;
            }
        };
        let (cap, _) = fastfile_iw6::walk::walk_zone(&zone);
        if let Ok(want) = std::env::var("IW6_RAW") {
            for r in &cap.raw_files {
                if r.name.contains(&want) {
                    println!("RAW {} ({} bytes)", r.name, r.len);
                }
            }
            for r in &cap.raw_files {
                if r.name.ends_with(&want) {
                    if let Some(b) = fastfile_iw6::map::raw_file(&cap, &zone.image, &r.name) {
                        println!("{}", String::from_utf8_lossy(&b));
                    }
                }
            }
        }
        let map = fastfile_iw6::map::read_map(&cap, &zone.image, &zone.assets.script_strings);
        let Some(w) = &map.world else {
            println!("{path}: no world");
            continue;
        };
        let mut paks = fastfile_iw6::image::ImagePaks::new(p.parent().unwrap());
        // `IW6_CENSUS_SHARED=1`: images named from other zones are looked
        // up in common_mp (as the game now does), built-ins made.
        let mut shared = fastfile_iw6::shared::SharedImages::default();
        if std::env::var_os("IW6_CENSUS_SHARED").is_some() {
            for z in [
                "patch_common_mp",
                "common_mp",
                "patch_common_core_mp",
                "common_core_mp",
            ] {
                let _ = shared.add_zone(&p.parent().unwrap().join(format!("{z}.ff")));
            }
            println!(
                "  shared images: {} from {:?}, {} bytes inline",
                shared.len(),
                shared.zones,
                shared.inline_bytes()
            );
        }
        let read_image = |rec: &fastfile_iw6::walk::ImageRec,
                          paks: &mut fastfile_iw6::image::ImagePaks|
         -> Result<(u32, u32, u32), String> {
            let px = if fastfile_iw6::shared::is_reference(rec) && !shared.is_empty() {
                match fastfile_iw6::shared::builtin_pixels(&rec.name) {
                    Some(px) => Ok(px),
                    None => shared.pixels(&rec.name, paks, max),
                }
            } else {
                fastfile_iw6::image::image_pixels(rec, &zone.image, paks, max)
            };
            px.map(|px| (px.width, px.height, px.dxgi_format))
        };
        // Per material: tris, surfaces.
        let mut use_: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
        let mut no_mat = 0usize;
        for s in &w.surfaces {
            match s.material {
                Some(m) => {
                    let e = use_.entry(m.index).or_default();
                    e.0 += s.tri_count as usize;
                    e.1 += 1;
                }
                None => no_mat += s.tri_count as usize,
            }
        }
        // Outcome per material.
        let mut by_outcome: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        let mut by_techset_untextured: HashMap<String, usize> = HashMap::new();
        let mut img_cache: HashMap<usize, Result<(u32, u32, u32), String>> = HashMap::new();
        let mut formats: BTreeMap<u32, usize> = BTreeMap::new();
        let total_tris: usize = w.surfaces.iter().map(|s| s.tri_count as usize).sum();
        for (&mi, &(tris, _surfs)) in &use_ {
            let m = &cap.materials[mi as usize];
            let ts = m
                .techset
                .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                .unwrap_or_default();
            let words: Vec<&str> = ts.split('_').collect();
            let hidden = if words.iter().any(|w| w.starts_with("sky")) {
                Some("hidden sky")
            } else if words.contains(&"tools") {
                Some("hidden tools")
            } else if words.contains(&"shadowcaster") || words.contains(&"onlycast") {
                Some("hidden shadow")
            } else if words.contains(&"distortion") {
                Some("hidden distortion")
            } else {
                None
            };
            let colour = m
                .textures
                .iter()
                .find(|t| t.name_hash == COLOR_MAP_HASH && t.image.is_some())
                .or_else(|| {
                    m.textures
                        .iter()
                        .find(|t| t.semantic == 2 && t.image.is_some())
                });
            let outcome = if let Some(h) = hidden {
                h.to_owned()
            } else if let Some(t) = colour {
                let ii = t.image.unwrap().index as usize;
                let r = img_cache
                    .entry(ii)
                    .or_insert_with(|| read_image(&cap.images[ii], &mut paks));
                let rec = &cap.images[ii];
                *formats.entry(rec.format).or_default() += tris;
                match r {
                    Ok(_) => "textured".to_owned(),
                    Err(e) => format!(
                        "colour map unreadable: {}",
                        e.split(" from ").next().unwrap_or(e)
                    ),
                }
            } else if m.textures.is_empty() {
                "no textures at all".to_owned()
            } else {
                "no colour map".to_owned()
            };
            if outcome != "textured" && !outcome.starts_with("hidden") {
                *by_techset_untextured.entry(ts.clone()).or_default() += tris;
                if list {
                    let tex: Vec<String> = m
                        .textures
                        .iter()
                        .map(|t| {
                            format!(
                                "{:08x}/s{}={}",
                                t.name_hash,
                                t.semantic,
                                t.image
                                    .map(|i| {
                                        let r = &cap.images[i.index as usize];
                                        format!(
                                            "{}[f{} {}x{} d{} st{}]",
                                            r.name,
                                            r.format,
                                            r.width,
                                            r.height,
                                            r.depth,
                                            r.streamed
                                        )
                                    })
                                    .unwrap_or("-".into())
                            )
                        })
                        .collect();
                    println!(
                        "  UNTEX {tris:>6} tris {} [{ts}] sort {} : {outcome} : {}",
                        m.name,
                        m.sort_key,
                        tex.join(" ")
                    );
                }
            }
            let e = by_outcome.entry(outcome).or_default();
            e.0 += 1;
            e.1 += tris;
        }
        if std::env::var_os("IW6_CENSUS_TECH").is_some() {
            // Per technique set: triangles, materials, the texture names seen.
            let mut per: BTreeMap<String, (usize, usize, std::collections::BTreeSet<String>)> =
                BTreeMap::new();
            for (&mi, &(tris, _)) in &use_ {
                let m = &cap.materials[mi as usize];
                let ts = m
                    .techset
                    .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                    .unwrap_or_default();
                let e = per.entry(ts).or_default();
                e.0 += tris;
                e.1 += 1;
                for t in &m.textures {
                    let n = match t.name_hash {
                        0xa0ab_1041 => "colorMap".to_owned(),
                        0xb60d_1850 => "colorMap1".to_owned(),
                        0xb60d_1853 => "colorMap2".to_owned(),
                        0x59d3_0d0f => "normalMap".to_owned(),
                        0x34ec_ccb3 => "specularMap".to_owned(),
                        0xeb52_9b4d => "detailMap".to_owned(),
                        0x34d8_49d5 => "revealMap".to_owned(),
                        h => format!("{h:08x}/s{}", t.semantic),
                    };
                    e.2.insert(n);
                }
            }
            let mut v: Vec<_> = per.into_iter().collect();
            v.sort_by(|a, b| b.1.0.cmp(&a.1.0));
            for (ts, (tris, n, tex)) in v.iter().take(60) {
                println!(
                    "  TECH {tris:>7} tris {n:>4} mats {ts:<50} {}",
                    tex.iter().cloned().collect::<Vec<_>>().join(",")
                );
            }
        }
        if std::env::var_os("IW6_CENSUS_VCOL").is_some() {
            // Vertex colour bytes by kind of material: mean, share below 250.
            let mut acc: BTreeMap<&str, ([f64; 4], [usize; 4], usize)> = BTreeMap::new();
            for s in &w.surfaces {
                let Some(m) = s.material else { continue };
                let ts = cap.materials[m.index as usize]
                    .techset
                    .map(|t| cap.name_of(t).to_owned())
                    .unwrap_or_default();
                let toks: Vec<&str> = ts.trim_start_matches(',').split('_').collect();
                let l1 = toks
                    .iter()
                    .any(|t| t.len() >= 4 && t.as_bytes()[1] == b'1' && t.as_bytes()[2] == b'c');
                let l2 = toks
                    .iter()
                    .any(|t| t.len() >= 4 && t.as_bytes()[1] == b'2' && t.as_bytes()[2] == b'c');
                let kind = match (l1, l2) {
                    (false, false) => "no layer",
                    (true, false) => "layer 1",
                    _ => "layers 1+2",
                };
                let e = acc.entry(kind).or_default();
                for v in &w.vertices
                    [s.first_vertex as usize..s.first_vertex as usize + s.vertex_count as usize]
                {
                    let b = v.color.to_le_bytes();
                    for k in 0..4 {
                        e.0[k] += f64::from(b[k]);
                        e.1[k] += usize::from(b[k] < 250);
                    }
                    e.2 += 1;
                }
            }
            for (k, (sum, low, n)) in acc {
                let n = n.max(1) as f64;
                println!(
                    "  VCOL {k:<10} bytes mean {:?} share<250 {:?}",
                    sum.map(|x| (x / n).round()),
                    low.map(|x| (x as f64 / n * 100.0).round())
                );
            }
        }
        if std::env::var_os("IW6_CENSUS_MVCOL").is_some() {
            // Placed models' vertex colours by the technique's wind tokens.
            let mut acc: BTreeMap<
                String,
                (
                    [f64; 4],
                    [usize; 4],
                    usize,
                    std::collections::BTreeSet<String>,
                ),
            > = BTreeMap::new();
            let mut seen = std::collections::HashSet::new();
            for sm in &w.static_models {
                let Some(id) = sm.model else { continue };
                if !seen.insert(id.index) {
                    continue;
                }
                let Some(m) = cap.xmodels.get(id.index as usize) else {
                    continue;
                };
                let Some(lod) = m.lods.first() else { continue };
                let Some(surfs) = lod
                    .surfs
                    .and_then(|s| cap.model_surfs.get(s.index as usize))
                else {
                    continue;
                };
                for (i, s) in surfs.surfs.iter().enumerate() {
                    let Some(mat) = m
                        .materials
                        .get(usize::from(lod.surf_index) + i)
                        .copied()
                        .flatten()
                    else {
                        continue;
                    };
                    let ts = cap.materials[mat.index as usize]
                        .techset
                        .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                        .unwrap_or_default();
                    let toks: Vec<&str> = ts.split('_').collect();
                    let kind = if toks
                        .iter()
                        .any(|t| t.starts_with("wv") || t.starts_with("flag"))
                    {
                        "wind"
                    } else if toks.iter().any(|t| t.starts_with("unlit")) {
                        "unlit"
                    } else {
                        "lit"
                    };
                    let e = acc.entry(kind.to_owned()).or_default();
                    if e.3.len() < 6 {
                        e.3.insert(ts.clone());
                    }
                    for v in s
                        .verts
                        .map_or(&[][..], |sp| sp.get(&zone.image))
                        .chunks_exact(32)
                    {
                        for k in 0..4 {
                            e.0[k] += f64::from(v[16 + k]);
                            e.1[k] += usize::from(v[16 + k] < 250);
                        }
                        e.2 += 1;
                    }
                }
            }
            for (k, (sum, low, n, ts)) in acc {
                let n = n.max(1) as f64;
                println!(
                    "  MVCOL {k:<6} bytes mean {:?} share<250 {:?} e.g. {:?}",
                    sum.map(|x| (x / n).round()),
                    low.map(|x| (x as f64 / n * 100.0).round()),
                    ts
                );
            }
        }
        {
            // Surfaces outside the lit / decal / trans / emissive ranges.
            let inr = |i: u32| w.ranges.iter().any(|&(a, b)| i >= a.min(b) && i < b);
            let mut out: BTreeMap<String, (usize, usize)> = BTreeMap::new();
            for (i, s) in w.surfaces.iter().enumerate() {
                if inr(i as u32) {
                    continue;
                }
                let ts = s
                    .material
                    .map(|m| {
                        let mr = &cap.materials[m.index as usize];
                        format!(
                            "{} [{}]",
                            mr.name,
                            mr.techset
                                .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                                .unwrap_or_default()
                        )
                    })
                    .unwrap_or_default();
                let e = out.entry(ts).or_default();
                e.0 += 1;
                e.1 += s.tri_count as usize;
            }
            for (ri, label) in [(1usize, "DECAL"), (2, "TRANS"), (3, "EMISSIVE")] {
                let (a, b) = w.ranges[ri];
                let mut per: BTreeMap<String, (usize, usize)> = BTreeMap::new();
                for s in &w.surfaces
                    [(a.min(b) as usize).min(w.surfaces.len())..(b as usize).min(w.surfaces.len())]
                {
                    let ts = s
                        .material
                        .map(|m| {
                            cap.materials[m.index as usize]
                                .techset
                                .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();
                    let e = per.entry(ts).or_default();
                    e.0 += 1;
                    e.1 += s.tri_count as usize;
                }
                let mut v: Vec<_> = per.into_iter().collect();
                v.sort_by(|x, y| y.1.1.cmp(&x.1.1));
                for (ts, (n, t)) in v.iter().take(12) {
                    println!("    {label} {n:>4} surfs {t:>6} tris {ts}");
                }
            }
            // Placed models' ground light.
            let mut g: Vec<f32> = w
                .static_models
                .iter()
                .map(|m| {
                    m.ground_lighting[0]
                        .max(m.ground_lighting[1])
                        .max(m.ground_lighting[2])
                })
                .filter(|&x| x > 0.0)
                .collect();
            g.sort_by(|a, b| a.partial_cmp(b).unwrap());
            if !g.is_empty() {
                let q = |f: f32| g[((g.len() - 1) as f32 * f) as usize];
                println!(
                    "  ground light ({} lit): p10 {:.3} p50 {:.3} p90 {:.3} max {:.3}",
                    g.len(),
                    q(0.1),
                    q(0.5),
                    q(0.9),
                    q(1.0)
                );
            }
            println!(
                "  ranges {:?} of {} surfaces; outside: {} surfaces",
                w.ranges,
                w.surfaces.len(),
                out.values().map(|v| v.0).sum::<usize>()
            );
            for (k, (n, t)) in out.iter().take(25) {
                println!("    OUTSIDE {n:>4} surfs {t:>6} tris {k}");
            }
        }
        // Placed models: their materials' colour maps, weighted by the
        // triangles placed.
        let mut model_out: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        {
            let mut per_model: HashMap<u32, usize> = HashMap::new();
            for sm in &w.static_models {
                if let Some(id) = sm.model {
                    *per_model.entry(id.index).or_default() += 1;
                }
            }
            let mut seen_mat: HashMap<u32, String> = HashMap::new();
            for (&mi, &count) in &per_model {
                let Some(m) = cap.xmodels.get(mi as usize) else {
                    continue;
                };
                let Some(lod) = m.lods.first() else { continue };
                let Some(surfs) = lod
                    .surfs
                    .and_then(|s| cap.model_surfs.get(s.index as usize))
                else {
                    continue;
                };
                for (i, s) in surfs.surfs.iter().enumerate() {
                    let Some(mat) = m
                        .materials
                        .get(usize::from(lod.surf_index) + i)
                        .copied()
                        .flatten()
                    else {
                        continue;
                    };
                    let outcome = seen_mat
                        .entry(mat.index)
                        .or_insert_with(|| {
                            let mr = &cap.materials[mat.index as usize];
                            if mr.techset.is_none() && mr.textures.is_empty() && !shared.is_empty()
                            {
                                let Some(sm) = shared.material(&mr.name) else {
                                    return format!("material in none of {:?}", shared.zones);
                                };
                                let Some(t) = sm
                                    .textures
                                    .iter()
                                    .find(|t| t.0 == COLOR_MAP_HASH)
                                    .or_else(|| sm.textures.iter().find(|t| t.2 == 2))
                                else {
                                    return "no colour map (shared material)".to_owned();
                                };
                                return match fastfile_iw6::shared::builtin_pixels(&t.3)
                                    .map_or_else(|| shared.pixels(&t.3, &mut paks, max), Ok)
                                {
                                    Ok(_) => "textured".to_owned(),
                                    Err(e) => format!("shared colour map unreadable: {e}"),
                                };
                            }
                            let colour = mr
                                .textures
                                .iter()
                                .find(|t| t.name_hash == COLOR_MAP_HASH && t.image.is_some())
                                .or_else(|| {
                                    mr.textures
                                        .iter()
                                        .find(|t| t.semantic == 2 && t.image.is_some())
                                });
                            match colour {
                                Some(t) => {
                                    let ii = t.image.unwrap().index as usize;
                                    match img_cache
                                        .entry(ii)
                                        .or_insert_with(|| read_image(&cap.images[ii], &mut paks))
                                    {
                                        Ok(_) => "textured".to_owned(),
                                        Err(e) => format!(
                                            "colour map unreadable: {}",
                                            e.split(" from ").next().unwrap_or(e)
                                        ),
                                    }
                                }
                                None => "no colour map".to_owned(),
                            }
                        })
                        .clone();
                    if list && outcome != "textured" {
                        let mr = &cap.materials[mat.index as usize];
                        let ts = mr
                            .techset
                            .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                            .unwrap_or_default();
                        let tex: Vec<String> = mr
                            .textures
                            .iter()
                            .map(|t| {
                                format!(
                                    "{:08x}/s{}={}",
                                    t.name_hash,
                                    t.semantic,
                                    t.image
                                        .map(|i| cap.images[i.index as usize].name.clone())
                                        .unwrap_or_default()
                                )
                            })
                            .collect();
                        println!(
                            "  MUNTEX {} x{count} {} tris: {} [{ts}] sort {} : {outcome} : {}",
                            m.name,
                            s.tri_count,
                            mr.name,
                            mr.sort_key,
                            tex.join(" ")
                        );
                    }
                    let e = model_out.entry(outcome).or_default();
                    e.0 += 1;
                    e.1 += s.tri_count as usize * count;
                }
            }
        }
        if std::env::var_os("IW6_CENSUS_LIGHT").is_some() {
            let sun = map.sun.map(|s| s.color);
            let mut lum: Vec<f32> = Vec::new();
            for &(_, sid) in &w.lightmaps {
                let Some(sid) = sid else { continue };
                let r = &cap.images[sid.index as usize];
                if let Ok(px) =
                    fastfile_iw6::image::image_pixels(r, &zone.image, &mut paks, 1 << 14)
                {
                    let n = px.data.len() / 8;
                    for i in (0..n).step_by(7) {
                        let h = |c: usize| {
                            fastfile_iw6::map::half_to_f32(u16::from_le_bytes([
                                px.data[i * 8 + 2 * c],
                                px.data[i * 8 + 2 * c + 1],
                            ]))
                        };
                        let l = 0.3 * h(0) + 0.59 * h(1) + 0.11 * h(2);
                        if l > 0.0 {
                            lum.push(l);
                        }
                    }
                }
            }
            lum.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q = |f: f32| {
                lum.get(((lum.len().max(1) - 1) as f32 * f) as usize)
                    .copied()
                    .unwrap_or(0.0)
            };
            println!(
                "  LIGHT sun {sun:?}; {} lightmaps; lit texels p50 {:.3} p90 {:.3} p99 {:.3}",
                w.lightmaps.len(),
                q(0.5),
                q(0.9),
                q(0.99)
            );
        }
        if std::env::var_os("IW6_CENSUS_Z").is_some() {
            // Where the geometry is in height: triangles by surface floor,
            // placed models by origin, in 4000-unit bands; the spawns' band.
            let mut tz: BTreeMap<i32, usize> = BTreeMap::new();
            for s in &w.surfaces {
                *tz.entry((s.mins[2] / 4000.0).floor() as i32).or_default() += s.tri_count as usize;
            }
            let mut mz: BTreeMap<i32, usize> = BTreeMap::new();
            for m in &w.static_models {
                *mz.entry((m.origin[2] / 4000.0).floor() as i32).or_default() += 1;
            }
            let sz: Vec<f32> = map.spawns.iter().map(|s| s.origin[2]).collect();
            println!(
                "  Z spawns {:.0}..{:.0}; world tris by floor band (x4000) {tz:?}; models by band {mz:?}",
                sz.iter().copied().fold(f32::MAX, f32::min),
                sz.iter().copied().fold(f32::MIN, f32::max)
            );
        }
        if std::env::var_os("IW6_CENSUS_BMODELS").is_some() {
            // Brush models with surfaces, and their entities.
            let text = fastfile_iw6::map::named_entity_text(&map.entity_string());
            for (bi, b) in w.brush_models.iter().enumerate() {
                if b.surface_count == 0 || b.start_surf == u32::MAX {
                    continue;
                }
                let tag = format!("\"model\" \"*{bi}\"");
                let ents: Vec<String> = text
                    .split('{')
                    .filter(|e| e.contains(&tag))
                    .map(|e| {
                        e.lines()
                            .filter(|l| {
                                !l.contains("\"model\"") && !l.trim().is_empty() && !l.contains('}')
                            })
                            .map(str::trim)
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect();
                let mut mats: BTreeMap<String, usize> = BTreeMap::new();
                let mut tris = 0usize;
                for s in w
                    .surfaces
                    .iter()
                    .skip(b.start_surf as usize)
                    .take(b.surface_count as usize)
                {
                    tris += s.tri_count as usize;
                    if let Some(m) = s.material {
                        *mats
                            .entry(cap.materials[m.index as usize].name.clone())
                            .or_default() += s.tri_count as usize;
                    }
                }
                let ms: Vec<String> = mats
                    .iter()
                    .take(4)
                    .map(|(k, v)| format!("{k}:{v}"))
                    .collect();
                println!(
                    "  BMODEL *{bi} surfs {}+{} {tris} tris {:?}..{:?} [{}] ents {:?}",
                    b.start_surf,
                    b.surface_count,
                    b.mins,
                    b.maxs,
                    ms.join(", "),
                    ents
                );
            }
        }
        {
            // Layered surfaces: those whose layers the reader missed (`b1ha1c1`,
            // `b1hb1c1`) and those with a second layer (weights in blue).
            let (mut h, mut two, mut layered) = (0usize, 0usize, 0usize);
            for s in &w.surfaces {
                let Some(m) = s.material else { continue };
                let ts = cap.materials[m.index as usize]
                    .techset
                    .map(|t| cap.name_of(t).to_owned())
                    .unwrap_or_default();
                let toks: Vec<&str> = ts.trim_start_matches(',').split('_').collect();
                let is_layer = |t: &&str, k: u8| {
                    let b = t.as_bytes();
                    b.len() >= 4 && matches!(b[0], b'b' | b'm' | b't' | b'a') && b[1] == k
                };
                let t = s.tri_count as usize;
                if toks.iter().any(|t| is_layer(t, b'1') || is_layer(t, b'2')) {
                    layered += t;
                }
                if toks
                    .iter()
                    .any(|t| (is_layer(t, b'1') || is_layer(t, b'2')) && t.as_bytes()[2] == b'h')
                {
                    h += t;
                }
                if toks.iter().any(|t| is_layer(t, b'2')) {
                    two += t;
                }
            }
            let tot: usize = w.surfaces.iter().map(|s| s.tri_count as usize).sum();
            println!(
                "  LAYERS layered {layered} tris ({:.1}%), height-blend layers unread before {h} ({:.1}%), second layer {two} ({:.1}%)",
                100.0 * layered as f64 / tot as f64,
                100.0 * h as f64 / tot as f64,
                100.0 * two as f64 / tot as f64
            );
        }
        let name = p.file_stem().unwrap().to_string_lossy();
        let model_tris: usize = model_out.values().map(|v| v.1).sum();
        for (o, (n, t)) in &model_out {
            println!(
                "  MODELS {o:<45} {n:>5} surfaces {t:>9} placed tris ({:.1}%)",
                100.0 * *t as f64 / model_tris.max(1) as f64
            );
        }
        println!(
            "{name}: {} world materials, {} tris ({} without material)",
            use_.len(),
            total_tris,
            no_mat
        );
        for (o, (n, t)) in &by_outcome {
            println!(
                "  {o:<50} {n:>5} materials {t:>8} tris ({:.1}%)",
                100.0 * *t as f64 / total_tris.max(1) as f64
            );
        }
        let mut v: Vec<_> = by_techset_untextured.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        for (ts, t) in v.iter().take(12) {
            println!("    untextured techset {ts:<40} {t:>8} tris");
        }
        println!("  colour map formats (tris): {formats:?}");
    }
}
