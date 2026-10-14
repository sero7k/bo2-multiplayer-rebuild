//! A map's light, for working out how to draw it: `iw7_light <map.ff>`
//! prints its `GfxLightMap`s (their images), the lightmap reindex table, the
//! surfaces' lightmap indices and lightmap coordinates, the placed models'
//! vertex lighting, the primary lights, the worldspawn and the raw files.
use fastfile_iw7::layout as L;
use fastfile_iw7::walk::walk_fitted;

fn f16(h: u16) -> f32 {
    let s = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let e = i32::from((h >> 10) & 0x1f);
    let m = f32::from(h & 0x3ff);
    s * match e {
        0 => m / 1024.0 * 2f32.powi(-14),
        31 => f32::INFINITY,
        _ => (1.0 + m / 1024.0) * 2f32.powi(e - 15),
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: iw7_light <map.ff>");
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&path)).expect("zone");
    let (w, stop) = walk_fitted(&zone);
    let cap = &w.cap;
    let im = &zone.image;
    println!("walk stop {:?}", stop.map(|s| s.error));
    let u32_at = |o: usize| u32::from_le_bytes(im[o..o + 4].try_into().unwrap());
    let f32_at = |o: usize| f32::from_le_bytes(im[o..o + 4].try_into().unwrap());
    let u16_at = |o: usize| u16::from_le_bytes(im[o..o + 2].try_into().unwrap());
    let img = |id: fastfile_iw7::walk::AssetId| {
        let n = cap.name_of(id);
        match cap.images.iter().find(|i| i.name == n.trim_start_matches(',')) {
            Some(r) => format!(
                "{n} fmt {} {}x{}x{} elems {} levels {} sem {} map {} cat {} streamed {} inline {} streams {:?}",
                r.format, r.width, r.height, r.depth, r.num_elements, r.level_count, r.semantic, r.map_type, r.category, r.streamed,
                r.pixels.is_some(), r.streams
            ),
            None => format!("{n} (no record here)"),
        }
    };
    for (i, l) in cap.light_maps.iter().enumerate() {
        println!("lightmap {i} {}", l.name);
        for t in l.textures.iter().flatten() {
            println!("   {}", img(*t));
        }
    }
    for g in &cap.gfx_worlds {
        let h = g.header;
        use L::GfxWorld as G;
        println!("world images: {:?}", g.images.iter().map(|(k, id)| format!("{k}={}", img(*id))).collect::<Vec<_>>());
        println!("sky images: {:?}", g.sky_images.iter().flatten().map(|id| img(*id)).collect::<Vec<_>>());
        println!(
            "lightmap reindex: pixel size {} reindex {} packed {} lightMapCount {}",
            u32_at(h + G::draw__lightmapReindexData__imagePixelSize),
            u32_at(h + G::draw__lightmapReindexData__reindexCount),
            u32_at(h + G::draw__lightmapReindexData__packedLightmapCount),
            u32_at(h + G::draw__lightMapCount)
        );
        if let Some(sp) = g.lightmap_reindex {
            for e in sp.get(im).chunks_exact(20) {
                println!("  reindex {:?}", (0..5).map(|k| u32::from_le_bytes(e[4 * k..4 * k + 4].try_into().unwrap())).collect::<Vec<_>>());
            }
        }
        if let Some(sp) = g.packed_lightmaps {
            for e in sp.get(im).chunks_exact(8) {
                println!("  packed {:?}", (0..2).map(|k| u32::from_le_bytes(e[4 * k..4 * k + 4].try_into().unwrap())).collect::<Vec<_>>());
            }
        }
        let mut hist = std::collections::BTreeMap::new();
        if let Some(at) = g.surfaces {
            for i in 0..g.surface_count as usize {
                let s = at + L::GfxSurface::SIZE * i;
                *hist.entry(im[s + L::GfxSurface::lightmapIndex]).or_insert(0) += 1;
            }
        }
        println!("surface lightmap indices: {hist:?}");
        // Static model lighting.
        let mut with = 0;
        let mut lm = std::collections::BTreeMap::new();
        let mut sample = Vec::new();
        if let Some(at) = g.smodels {
            use L::GfxStaticModelDrawInst as D;
            for i in 0..g.smodel_count as usize {
                let d = at + D::SIZE * i;
                *lm.entry(u32_at(d + D::modelLightmapInfo__lightmapIndex) as i32).or_insert(0) += 1;
                if let Some(Some(sp)) = g.smodel_lighting.get(i) {
                    with += 1;
                    if sample.len() < 6 {
                        let n = sp.len / 20;
                        let mut acc = [0f32; 4];
                        for k in 0..n {
                            let v = sp.pos + 20 * k;
                            for c in 0..4 {
                                acc[c] += f16(u16_at(v + 4 + 2 * c));
                            }
                        }
                        sample.push(format!(
                            "#{i} {} verts, mean ambient {:?}, flags {:#x} off {:?} scale {:?}",
                            n,
                            acc.map(|a| a / n.max(1) as f32),
                            u32_at(d + D::vertexLightingInfo__flags),
                            [f32_at(d + D::modelLightmapInfo__offset), f32_at(d + D::modelLightmapInfo__offset + 4)],
                            [f32_at(d + D::modelLightmapInfo__scale), f32_at(d + D::modelLightmapInfo__scale + 4)],
                        ));
                    }
                }
            }
        }
        println!("smodels {} with vertex lighting {with}; lightmap index hist {:?}", g.smodel_count, lm.iter().take(12).collect::<Vec<_>>());
        for s in sample {
            println!("   {s}");
        }
    }
    // Lightmap coordinates per transient zone.
    for t in &cap.gfx_world_trs {
        let Some(at) = t.vertices else { continue };
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        for i in 0..t.vertex_count as usize {
            let v = at + 44 * i;
            for k in 0..2 {
                let x = f32_at(v + 28 + 4 * k);
                lo[k] = lo[k].min(x);
                hi[k] = hi[k].max(x);
            }
        }
        println!("transient zone {} #{}: {} verts lmap uv {lo:?}..{hi:?}", t.name, t.index, t.vertex_count);
    }
    for c in &cap.com_worlds {
        for (i, l) in c.lights.iter().enumerate().take(4) {
            println!("light {i}: kind {} colour {:?} dir {:?} def {:?}", l.kind, l.color, l.dir, l.def_name);
        }
        println!("{} primary lights", c.lights.len());
    }
    if let Some(m) = cap.map_ents.first() {
        let first: String = m.entity_string.split('}').next().unwrap_or("").to_owned();
        let named = fastfile_iw7::map::entity_text(&(first + "}\n"), &[]);
        println!("worldspawn:\n{named}");
    }
    for r in &cap.raw_files {
        println!("rawfile {} ({} bytes, compressed {})", r.name, r.len, r.compressed_len);
    }
}
