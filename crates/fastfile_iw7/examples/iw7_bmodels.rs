//! A map's brush models (`*<n>`: doors, movers, destructibles) and the
//! entities that name them: `iw7_bmodels <map.ff>` prints each brush
//! model's surfaces, their bounds, its own bounds, and each entity naming it
//! (classname, targetname, origin, angles), to see whether its surfaces are
//! stored about the entity's origin or already in place.
use fastfile_iw7::map::{Iw7Map, MapSources};

fn main() {
    let path = std::env::args().nth(1).expect("usage: iw7_bmodels <map.ff>");
    let map = Iw7Map::load(&{ let src = MapSources::beside(std::path::Path::new(&path)); if std::env::var("IW7_NO_COMMON").is_ok() { src } else { src.with_common() } }).expect("load");
    let ents = fastfile_iw7::map::parse_entities(&map.entities);
    let n = map.surfaces.len();
    println!("{} surfaces, {} brush models", n, map.brush_models.len());
    for (i, b) in map.brush_models.iter().enumerate() {
        let r = b.first_surface as usize..(b.first_surface + b.surface_count) as usize;
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for s in &map.surfaces[r.start.min(n)..r.end.min(n)] {
            for k in 0..3 {
                lo[k] = lo[k].min(s.mins[k]);
                hi[k] = hi[k].max(s.maxs[k]);
            }
        }
        let name = format!("*{i}");
        let users: Vec<String> = ents
            .iter()
            .filter(|e| e.get("model").map(String::as_str) == Some(name.as_str()))
            .map(|e| {
                format!(
                    "{} tn={} sn={} o={} a={}",
                    e.get("classname").map_or("", String::as_str),
                    e.get("targetname").map_or("", String::as_str),
                    e.get("script_noteworthy").map_or("", String::as_str),
                    e.get("origin").map_or("-", String::as_str),
                    e.get("angles").map_or("-", String::as_str)
                )
            })
            .collect();
        let mats: Vec<String> = map.surfaces[r.start.min(n)..r.end.min(n)]
            .iter()
            .filter_map(|s| s.material.map(|m| map.materials[m].name.clone()))
            .take(3)
            .collect();
        println!(
            "{name}: surfs {r:?} surf bounds {:?}..{:?} model bounds {:?}..{:?} mats {mats:?}",
            lo.map(|v| v as i32),
            hi.map(|v| v as i32),
            b.mins.map(|v| v as i32),
            b.maxs.map(|v| v as i32)
        );
        for u in users {
            println!("    {u}");
        }
    }
}
