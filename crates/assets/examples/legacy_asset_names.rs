//! Lists a CoD4 / World at War zone's models and materials whose names hold
//! a pattern: `legacy_asset_names <zone.ff> <pattern>...`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let pats: Vec<String> = a[2..].iter().map(|p| p.to_ascii_lowercase()).collect();
    let z = t4::load_map(path).unwrap().zone;
    // `ents <class>`: the map's entities of a classname, with their keys.
    if a.get(2).map(String::as_str) == Some("ents") {
        let text = z.map_ents().map(|m| m.entity_string.clone()).unwrap_or_default();
        for e in iw3::ents::parse(&text).iter().filter(|e| e.classname().contains(a.get(3).map_or("", |s| s.as_str()))) {
            println!("{:?}", e);
        }
        return;
    }
    let hit = |n: &str| pats.iter().any(|p| n.to_ascii_lowercase().contains(p.as_str()));
    for asset in &z.assets {
        match asset {
            iw3::zone::Asset::XModel(m) if hit(&m.name) => {
                let mats: Vec<String> = m.materials.iter().flatten().filter_map(|&id| z.material(id)).map(|mm| {
                    let ts = mm.technique_set.and_then(|t| z.technique_set(t)).map_or("", |t| t.name.as_str());
                    format!("{} [{ts}]", mm.name)
                }).collect();
                println!("model {} {:?}", m.name, mats);
            }
            iw3::zone::Asset::Material(m) if hit(&m.name) => {
                let ts = m.technique_set.and_then(|t| z.technique_set(t)).map_or("", |t| t.name.as_str());
                println!("material {} [{ts}]", m.name);
            }
            _ => {}
        }
    }
    if let Some(w) = z.gfx_world() {
        let mut placed = std::collections::BTreeMap::<String, usize>::new();
        for sm in &w.static_models {
            if let Some(m) = sm.model.and_then(|id| z.xmodel(id)) && hit(&m.name) {
                *placed.entry(m.name.clone()).or_default() += 1;
            }
        }
        for (n, c) in placed { println!("placed {c} x {n}"); }
        let mut used = std::collections::BTreeMap::<String, usize>::new();
        for srf in &w.surfaces {
            if let Some(m) = srf.material.and_then(|m| z.material(m)) && hit(&m.name) {
                *used.entry(format!("{} ts={}", m.name, m.technique_set.is_some())).or_default() += 1;
            }
        }
        for (n, c) in used { println!("surfaces {c} x {n}"); }
    }
}
