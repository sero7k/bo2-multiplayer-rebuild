//! bo2mp: where things stand in a Black Ops III map, to place a camera:
//! `t7_where <zone.ff> <substring>...` prints every placed model whose
//! model name holds one of the substrings (origin, scale), and every
//! entity whose classname, targetname or model does (origin, angles).
use fastfile_t7::map::{MapSources, T7Map};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[1]);
    let map = T7Map::load(&MapSources::beside(path)).expect("map");
    // `t7_where <zone.ff> near <x> <y> <radius>`: every placed model within
    // the radius (in the plane), nearest first, each model name once.
    if a.get(2).map(String::as_str) == Some("near") {
        let f = |i: usize| a.get(i).and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
        let (x, y, r) = (f(3), f(4), f(5));
        let mut hits: Vec<(f32, &str, [f32; 3])> = map
            .static_models
            .iter()
            .map(|sm| (((sm.origin[0] - x).powi(2) + (sm.origin[1] - y).powi(2)).sqrt(), map.models[sm.model].name.as_str(), sm.origin))
            .filter(|h| h.0 <= r)
            .collect();
        hits.sort_by(|p, q| p.0.total_cmp(&q.0));
        let mut seen = std::collections::HashSet::new();
        for (d, n, o) in hits {
            if seen.insert(n) {
                println!("{d:6.0} {n} at {:.0} {:.0} {:.0}", o[0], o[1], o[2]);
            }
        }
        return;
    }
    let want: Vec<String> = a[2..].iter().map(|s| s.to_ascii_lowercase()).collect();
    let hit = |s: &str| {
        let s = s.to_ascii_lowercase();
        want.iter().any(|w| s.contains(w.as_str()))
    };
    for sm in &map.static_models {
        let name = &map.models[sm.model].name;
        if hit(name) {
            println!("model {name} at {:.0} {:.0} {:.0} scale {:.2}", sm.origin[0], sm.origin[1], sm.origin[2], sm.scale);
        }
    }
    for block in map.entities.split('}') {
        let field = |k: &str| {
            block.lines().find_map(|l| {
                let l = l.trim();
                let rest = l.strip_prefix(&format!("\"{k}\" "))?;
                Some(rest.trim_matches('"').to_owned())
            })
        };
        let class = field("classname").unwrap_or_default();
        let target = field("targetname").unwrap_or_default();
        let model = field("model").unwrap_or_default();
        if hit(&class) || hit(&target) || hit(&model) {
            println!(
                "entity {class} {target} {model} origin {} angles {}",
                field("origin").unwrap_or_default(),
                field("angles").unwrap_or_default()
            );
        }
    }
}
