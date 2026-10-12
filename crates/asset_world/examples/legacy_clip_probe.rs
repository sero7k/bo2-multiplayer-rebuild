//! bo2mp offline probe: a CoD4 (`iw3`) or World at War (`t4`) map zone's
//! collision as the match builds it, and BO2's spawn placement against it.
//!
//! `cargo run -p asset_world --example legacy_clip_probe -- <zone.ff>...`

use asset_world::MASK_PLAYER_SOLID;

const PLAYER_MINS: [f32; 3] = [-15.0, -15.0, 0.0];
const PLAYER_MAXS: [f32; 3] = [15.0, 15.0, 70.0];

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
        println!("== {} (version {version:#x})", path.display());
        println!(
            "clip: brushes {} planes {} verts {} tris {} partitions {} aabb {} cmodels {} leaf_brush_nodes {} materials {}",
            clip.brushes.len(),
            clip.planes.len(),
            clip.verts.len(),
            clip.tri_indices.len() / 3,
            clip.partitions.len(),
            clip.aabb_trees.len(),
            clip.cmodels.len(),
            clip.leaf_brush_nodes.len(),
            clip.materials.len()
        );
        for (i, m) in clip.cmodels.iter().take(3).enumerate() {
            println!("  cmodel {i}: {m:?}");
        }
        let mut contents_hist = std::collections::BTreeMap::<i32, usize>::new();
        for b in &clip.brushes {
            *contents_hist.entry(b.contents).or_default() += 1;
        }
        println!("  brush contents: {contents_hist:x?}");
        let mut sides_hist = std::collections::BTreeMap::<usize, usize>::new();
        for b in &clip.brushes {
            *sides_hist.entry(b.side_planes.len()).or_default() += 1;
        }
        println!("  brush side counts: {sides_hist:?}");
        for m in clip.materials.iter().take(40) {
            println!("  material {:40} surface {:#010x} contents {:#010x}", m.name, m.surface_flags, m.content_flags);
        }
        if let Ok(n) = std::env::var("PROBE_MATERIALS") {
            materials(&zone, n.parse().unwrap_or(0));
        }
        let collision = asset_world::world_iw3::build_iw3_clip_collision(clip).unwrap();
        println!(
            "built: brushes {} mesh tris {} aabb roots {} (forest roots {})",
            collision.brushes.len(),
            collision.mesh.tri_count(),
            collision.mesh.aabb_roots.len(),
            clipmap_iw4::aabb_forest_roots(&collision.mesh.aabb_trees).len()
        );
        let ents = iw3::ents::parse(&zone.map_ents().map(|m| m.entity_string.clone()).unwrap_or_default());
        let spawns: Vec<[f32; 3]> = ents
            .iter()
            .filter(|e| e.classname().starts_with("mp_") && e.classname().contains("spawn"))
            .filter_map(|e| e.origin())
            .collect();
        let at = std::time::Instant::now();
        if std::env::var("PROBE_MODELS").is_ok() {
            for e in ents.iter().filter(|e| e.get("model").is_some_and(|m| m.starts_with('*'))).take(12) {
                let k: usize = e.get("model").unwrap()[1..].parse().unwrap_or(0);
                let m = collision.cmodels.get(k);
                let bs: Vec<_> = m.map(|m| collision.leafbrushes[m.first_brush as usize..][..m.num_brushes as usize].to_vec()).unwrap_or_default();
                let bounds: Vec<_> = bs.iter().take(3).map(|&b| { let c = &clip.brushes[b as usize]; (c.mins, c.maxs, c.contents) }).collect();
                println!("  ent {} model *{k} origin {:?}: cmodel {:?} brushes {} {bounds:?}", e.classname(), e.origin(), clip.cmodels.get(k).map(|c| (c.mins, c.maxs)), bs.len());
            }
        }
        let census = asset_world::world_iw3::spawn_ground_census(&collision, &spawns);
        println!("spawns: {census:?} ({:.0} ms)", at.elapsed().as_secs_f64() * 1000.0);
        println!("census line: {census}");
        if let Ok(at) = std::env::var("PROBE_AT") {
            let v: Vec<f32> = at.split(',').map(|x| x.parse().unwrap()).collect();
            column(&zone, clip, [v[0], v[1], 0.0]);
        }
        if let Ok(n) = std::env::var("PROBE_COLUMNS") {
            for s in spawns.iter().step_by(37).take(n.parse().unwrap_or(3)) {
                column(&zone, clip, *s);
            }
        }
        details(&collision, &spawns, std::env::var("PROBE_DETAILS").ok().and_then(|v| v.parse().ok()).unwrap_or(0));
        // Mesh only / brushes only, to tell which one holds the floor.
        let map = clipmap_iw4::ClipMapRef { nodes: &collision.nodes, leaves: &collision.leaves, leafbrushes: &collision.leafbrushes, brushes: &collision.brushes };
    let no_mesh = clipmap_iw4::ClipMeshRef::from_linear(&[], &[], &[]);
        let mesh = clipmap_iw4::ClipMeshTables::as_ref(&collision.mesh);
        let (mut mesh_hits, mut brush_hits) = (0, 0);
        for s in spawns.iter().take(200) {
            let top = [s[0], s[1], s[2] + 32.0];
            let bottom = [s[0], s[1], s[2] - 1024.0];
            let ext = clipmap_iw4::TraceExtents::new(top, bottom, PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID);
            let b = clipmap_iw4::trace_brush_and_mesh(&map, &no_mesh, &ext, &|_| true);
            let m = clipmap_iw4::trace_through_mesh(&mesh, &ext);
            if b.fraction < 1.0 && (b.fraction * 1056.0) < 80.0 {
                brush_hits += 1;
            }
            if m.fraction < 1.0 && (m.fraction * 1056.0) < 80.0 {
                mesh_hits += 1;
            }
        }
        println!("first 200 spawns: floor within 48 below from brushes {brush_hits}, from mesh {mesh_hits}");
    }
}

/// Details for the first spawns that end up in solid or far below.
pub fn details(collision: &asset_world::ClipCollision, spawns: &[[f32; 3]], limit: usize) {
    let map = clipmap_iw4::ClipMapRef { nodes: &collision.nodes, leaves: &collision.leaves, leafbrushes: &collision.leafbrushes, brushes: &collision.brushes };
    let no_mesh = clipmap_iw4::ClipMeshRef::from_linear(&[], &[], &[]);
    let mesh = clipmap_iw4::ClipMeshTables::as_ref(&collision.mesh);
    let tr = |a: [f32; 3], b: [f32; 3]| {
        let ext = clipmap_iw4::TraceExtents::new(a, b, PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID);
        let br = clipmap_iw4::trace_brush_and_mesh(&map, &no_mesh, &ext, &|_| true);
        let me = clipmap_iw4::trace_through_mesh(&mesh, &ext);
        (br, me)
    };
    let mut shown = 0;
    for &o in spawns {
        if shown >= limit {
            break;
        }
        let (b0, m0) = tr(o, o);
        let down_end = [o[0], o[1], o[2] - 2048.0];
        let (bd, md) = tr([o[0], o[1], o[2] + 16.0], down_end);
        let (ub, um) = tr(o, [o[0], o[1], o[2] + 128.0]);
        let interesting = b0.startsolid != 0 || m0.startsolid != 0 || (bd.fraction.min(md.fraction) * 2064.0 > 80.0);
        if !interesting {
            continue;
        }
        shown += 1;
        println!(
            "  spawn {o:?}: at origin solid brush={} mesh={}; up frac brush {:.3} mesh {:.3} (startsolid b{} m{}); down from +16: brush frac {:.4} (z {:.1}, n {:?}, solid {}) mesh frac {:.4} (z {:.1}, n {:?}, solid {})",
            b0.startsolid, m0.startsolid, ub.fraction, um.fraction, ub.startsolid, um.startsolid,
            bd.fraction, o[2] + 16.0 - 2064.0 * bd.fraction, bd.normal, bd.startsolid,
            md.fraction, o[2] + 16.0 - 2064.0 * md.fraction, md.normal, md.startsolid
        );
    }
}

fn tri_z_at(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], x: f32, y: f32) -> Option<f32> {
    let d = (v1[1] - v2[1]) * (v0[0] - v2[0]) + (v2[0] - v1[0]) * (v0[1] - v2[1]);
    if d.abs() < 1e-6 {
        return None;
    }
    let a = ((v1[1] - v2[1]) * (x - v2[0]) + (v2[0] - v1[0]) * (y - v2[1])) / d;
    let b = ((v2[1] - v0[1]) * (x - v2[0]) + (v0[0] - v2[0]) * (y - v2[1])) / d;
    let c = 1.0 - a - b;
    (a >= 0.0 && b >= 0.0 && c >= 0.0).then(|| a * v0[2] + b * v1[2] + c * v2[2])
}

/// The heights of the drawn world, the collision triangles, and the brushes
/// holding the point, straight up and down through `p`.
pub fn column(zone: &iw3::zone::Zone, clip: &iw3::zone::ClipMap, p: [f32; 3]) {
    let mut gfx = Vec::new();
    if let Some(w) = zone.gfx_world() {
        for s in &w.surfaces {
            if p[0] < s.bounds[0][0] || p[0] > s.bounds[1][0] || p[1] < s.bounds[0][1] || p[1] > s.bounds[1][1] {
                continue;
            }
            for t in 0..s.tri_count as usize {
                let idx = |k: usize| w.indices[s.base_index as usize + t * 3 + k] as usize + s.first_vertex as usize;
                let (a, b, c) = (w.vertices[idx(0)].xyz, w.vertices[idx(1)].xyz, w.vertices[idx(2)].xyz);
                if let Some(z) = tri_z_at(a, b, c, p[0], p[1]) {
                    gfx.push(z);
                }
            }
        }
    }
    gfx.sort_by(f32::total_cmp);
    let mut col = Vec::new();
    for t in clip.tri_indices.chunks_exact(3) {
        let v = |i: u16| clip.verts[i as usize];
        if let Some(z) = tri_z_at(v(t[0]), v(t[1]), v(t[2]), p[0], p[1]) {
            col.push(z);
        }
    }
    col.sort_by(f32::total_cmp);
    println!("    column at {:.0} {:.0}: drawn z {gfx:.0?}; collision tris z {col:.0?}", p[0], p[1]);
    for (i, b) in clip.brushes.iter().enumerate() {
        let inside_box = (0..3).all(|k| p[k] >= b.mins[k] - 1.0 && p[k] <= b.maxs[k] + 1.0);
        if !inside_box {
            continue;
        }
        let inside = b.side_planes.iter().all(|&pi| {
            clip.planes.get(pi as usize).is_none_or(|pl| pl.normal[0] * p[0] + pl.normal[1] * p[1] + pl.normal[2] * p[2] - pl.dist <= 1.0)
        });
        if inside {
            let planes: Vec<_> = b.side_planes.iter().filter_map(|&pi| clip.planes.get(pi as usize)).map(|pl| (pl.normal, pl.dist)).collect();
            println!(
                "    inside brush {i}: contents {:#x} mins {:?} maxs {:?} sides {} {planes:.2?} mats {:?}",
                b.contents, b.mins, b.maxs, b.side_planes.len(), b.side_materials
            );
        }
    }
}

/// The world surfaces' materials: their textures (name hash, semantic, image).
pub fn materials(zone: &iw3::zone::Zone, limit: usize) {
    let Some(w) = zone.gfx_world() else { return };
    let mut seen = std::collections::BTreeMap::<usize, usize>::new();
    for s in &w.surfaces {
        if let Some(m) = s.material {
            *seen.entry(m).or_default() += 1;
        }
    }
    let mut no_colour = 0;
    let mut hist = std::collections::BTreeMap::<String, usize>::new();
    for (&mid, &n) in &seen {
        let Some(m) = zone.material(mid) else { continue };
        let ts = m.technique_set.and_then(|t| zone.technique_set(t)).map_or("", |t| t.name.as_str());
        let has = m.textures.iter().any(|t| t.semantic == iw3::zone::TextureSemantic::Color && t.image.is_some());
        if !has {
            no_colour += 1;
        }
        for t in &m.textures {
            *hist.entry(format!("{:#010x}/{:?}", t.name_hash, t.semantic)).or_default() += 1;
        }
        if seen.len() < limit || !has {
            let tex: Vec<String> = m
                .textures
                .iter()
                .map(|t| format!("{:#x}/{:?}={}", t.name_hash, t.semantic, t.image.and_then(|i| zone.image(i)).map_or("-", |i| i.name.as_str())))
                .collect();
            println!("  mat {} ({n} surfs) ts {ts}: {}", m.name, tex.join(" "));
        }
    }
    println!("  world materials {} without a colour texture {no_colour}; textures by hash/semantic {hist:?}", seen.len());
}
