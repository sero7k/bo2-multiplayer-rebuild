//! A map zone's world: `iw7_map <map.ff>` prints its GfxWorld (surfaces,
//! vertices, static models, transient zones), its spawn points (by script
//! string name) and the first entities of its entity string.
use fastfile_iw7::walk::walk_fitted;

fn main() {
    let path = std::env::args().nth(1).expect("usage: iw7_map <map.ff>");
    let zone = fastfile_iw7::Zone::open(std::path::Path::new(&path)).expect("zone");
    let (w, stop) = walk_fitted(&zone);
    let cap = &w.cap;
    println!("walked {}/{} stop {:?}", cap.walked, zone.assets.assets.len(), stop.map(|s| s.error));
    let sstr = |i: u32| zone.assets.script_strings.get(i as usize).cloned().flatten().unwrap_or_else(|| format!("#{i}"));
    for g in &cap.gfx_worlds {
        let tz: Vec<String> = g.transient_zones.iter().flatten().map(|id| cap.name_of(*id).to_owned()).collect();
        println!(
            "gfxworld {}: {} surfaces ({} static), {} indices, {} smodels ({} with a model), {} brush models, {} light maps, transient zones {:?}",
            g.name,
            g.surface_count,
            g.static_surface_count,
            g.index_count,
            g.smodel_count,
            g.smodel_models.iter().filter(|m| m.is_some()).count(),
            g.model_count,
            g.light_maps.iter().flatten().count(),
            tz
        );
        let mut names: Vec<&str> = g.smodel_models.iter().flatten().map(|id| cap.name_of(*id)).collect();
        names.sort();
        names.dedup();
        println!("  {} distinct smodel models, first {:?}", names.len(), &names[..names.len().min(8)]);
        let mats: std::collections::BTreeSet<&str> = g.surface_materials.iter().flatten().map(|id| cap.name_of(*id)).collect();
        println!("  {} distinct surface materials, first {:?}", mats.len(), mats.iter().take(6).collect::<Vec<_>>());
    }
    let inline_surfs = cap.model_surfs.iter().filter(|m| m.surfs.iter().any(|s| s.verts.is_some())).count();
    println!("xmodels {}, xmodel_surfs {} ({} with vertex data here)", cap.xmodels.len(), cap.model_surfs.len(), inline_surfs);
    if let Some(m) = cap.xmodels.iter().find(|m| m.name == "barrier_handrail_cap_01") {
        let lods: Vec<String> = m.lods.iter().map(|l| format!("{} surfs {:?}", l.numsurfs, l.surfs.map(|id| cap.name_of(id).to_owned()))).collect();
        println!("  e.g. {} lods {lods:?}", m.name);
    }
    for t in &cap.gfx_world_trs {
        println!("transient zone {} #{}: {} vertices, layer data {:?}", t.name, t.index, t.vertex_count, t.layer_data.map(|s| s.len));
    }
    for m in &cap.map_ents {
        println!("map_ents {}: {} entity chars, {} spawns, {} cmodels, {} scriptables, {} spawners", m.name, m.entity_string.len(), m.spawn_count, m.cmodel_count, m.scriptable_count, m.spawner_count);
        if let Some(s) = m.spawns {
            let d = &zone.image[s.pos..s.pos + s.len];
            let mut by: std::collections::BTreeMap<String, usize> = Default::default();
            for r in d.chunks_exact(40) {
                let u = |o: usize| u32::from_le_bytes(r[o..o + 4].try_into().unwrap());
                *by.entry(sstr(u(4))).or_default() += 1;
            }
            println!("  spawns by name: {by:?}");
            let r = &d[..40];
            let f = |o: usize| f32::from_le_bytes(r[o..o + 4].try_into().unwrap());
            println!("  first: origin ({}, {}, {}) angles ({}, {}, {})", f(16), f(20), f(24), f(28), f(32), f(36));
        }
        if let Ok(out) = std::env::var("IW7_ENTS") {
            std::fs::write(out, &m.entity_string).expect("write ents");
        }
        println!("  entity string head: {:?}", &m.entity_string[..m.entity_string.len().min(400)]);
    }
    for c in &cap.clip_maps {
        println!("clipmap {}: {} static models, havok {:?}", c.name, c.static_model_count, c.havok_world_shape.map(|s| s.len));
        if let Some(g) = cap.gfx_worlds.first() {
            let (mut same, mut both) = (0, 0);
            for (a, b) in g.smodel_models.iter().zip(&c.static_model_models) {
                if let (Some(a), Some(b)) = (a, b) {
                    both += 1;
                    same += usize::from(cap.name_of(*a) == cap.name_of(*b));
                }
            }
            let mut h: std::collections::HashMap<&str, i64> = Default::default();
            for a in g.smodel_models.iter().flatten() {
                *h.entry(cap.name_of(*a)).or_default() += 1;
            }
            for b in c.static_model_models.iter().flatten() {
                *h.entry(cap.name_of(*b)).or_default() -= 1;
            }
            let off: i64 = h.values().map(|v| v.abs()).sum();
            println!("  gfx and clip static models agree: {same} of {both} in order; per-model count differences {off}");
        }
    }
    if std::env::var_os("IW7_STRINGS").is_some() {
        for (i, s) in zone.assets.script_strings.iter().enumerate() {
            println!("  sstr {i} {s:?}");
        }
    }
}
