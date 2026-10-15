//! `iw7_zm_doors <cp_map.ff>`: each Zombies blocker (debris, team door):
//! its group's entities, brush models (surfaces, materials, bounds) and the
//! placed models standing in those bounds.
use fastfile_iw7::map::{Iw7Map, MapSources, parse_entities};
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let map = Iw7Map::load(&MapSources::beside(Path::new(&a[1]))).expect("map");
    let layout = fastfile_iw7::zombies::ZmLayout::read(&map);
    let ents = parse_entities(&map.entities);
    for b in &layout.blockers {
        println!("== {} {:?} cost {} sides {:?}", b.group, b.kind, b.cost, b.sides.iter().map(|s| &s.1).collect::<Vec<_>>());
        for e in ents.iter().filter(|e| e.get("targetname") == Some(&b.group) || e.get("target") == Some(&b.group)) {
            let mut kv: Vec<String> = e.iter().filter(|(k, _)| !k.starts_with("_k") || k.as_str() == "_k502").map(|(k, v)| format!("{k}={v}")).collect();
            kv.sort();
            println!("   ent {}", kv.join(" "));
        }
        for (n, origin) in &b.brushes {
            let Some(bm) = map.brush_models.get(*n as usize) else { continue };
            let surfs: Vec<_> = (bm.first_surface..bm.first_surface.saturating_add(bm.surface_count)).filter_map(|i| map.surfaces.get(i as usize)).collect();
            let mats: std::collections::BTreeSet<String> = surfs.iter().filter_map(|s| s.material).map(|m| format!("{} [{}]", map.materials[m].name, map.materials[m].technique_set)).collect();
            println!("   *{n} at {origin:?}: {} surfaces, {} tris, bounds {:?}..{:?}, mats {:?}", surfs.len(), surfs.iter().map(|s| s.tri_count as usize).sum::<usize>(), bm.mins, bm.maxs, mats);
            let lo: [f32; 3] = std::array::from_fn(|k| origin[k] + bm.mins[k] - 16.0);
            let hi: [f32; 3] = std::array::from_fn(|k| origin[k] + bm.maxs[k] + 16.0);
            let near: Vec<String> = map
                .static_models
                .iter()
                .filter(|s| (0..3).all(|k| s.origin[k] >= lo[k] && s.origin[k] <= hi[k]))
                .map(|s| format!("{}@{:.0},{:.0},{:.0}", map.models[s.model].name, s.origin[0], s.origin[1], s.origin[2]))
                .collect();
            println!("      placed models inside: {near:?}");
        }
    }
    // Every brush model the entities name, and those that name none.
    let mut named = std::collections::BTreeMap::new();
    for e in &ents {
        if let Some(n) = e.get("model").and_then(|m| m.strip_prefix('*')).and_then(|m| m.parse::<usize>().ok()) {
            named.entry(n).or_insert_with(Vec::new).push(format!("{}:{}:{}", e.get("classname").map_or("", |s| s), e.get("targetname").map_or("", |s| s), e.get("script_noteworthy").map_or("", |s| s)));
        }
    }
    for (k, bm) in map.brush_models.iter().enumerate().skip(1) {
        if bm.surface_count == 0 || bm.first_surface == u32::MAX { continue; }
        println!("*{k}: {} surfaces; named by {:?}", bm.surface_count, named.get(&k));
    }
}
