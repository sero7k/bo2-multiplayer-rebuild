//! `iw7_zm_audit <map.ff> [max_dim]`: which of a map's drawn materials'
//! pictures resolve (the map's zones, the common zones, the engine's own)
//! and decode, slot by slot; the pictures that do not, by how many
//! surfaces and placed models use them; the emissive/unlit materials.
//! `IW7_NO_COMMON=1` leaves the common zones out.
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use fastfile_iw7::map::{Iw7Map, MapSources};
use fastfile_iw7::texture;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_dim: u32 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(64);
    let started = std::time::Instant::now();
    let src = MapSources::beside(Path::new(&args[1]));
    let src = if std::env::var("IW7_NO_COMMON").is_ok() { src } else { src.with_common() };
    let map = Iw7Map::load(&src).expect("map");
    for l in &map.report {
        println!("{l}");
    }
    println!("read in {:.1}s", started.elapsed().as_secs_f32());
    if std::env::var("IW7_AUDIT_VISION").is_ok() {
        if let Some((name, text)) = &map.vision {
            println!("vision {name}:\n{text}");
        }
        for l in &map.lights {
            println!("light kind {} colour {:?} dir {:?} origin {:?} radius {}", l.kind, l.color, l.dir, l.origin, l.radius);
        }
        return;
    }
    // How often each material is drawn (world surfaces shown, placed models).
    let mut uses: HashMap<usize, usize> = HashMap::new();
    let mut hidden_uses: HashMap<usize, usize> = HashMap::new();
    for s in &map.surfaces {
        if let Some(m) = s.material {
            *if s.hidden_brush { &mut hidden_uses } else { &mut uses }.entry(m).or_default() += 1;
        }
    }
    for sm in &map.static_models {
        for s in &map.models[sm.model].surfaces {
            if let Some(m) = s.material {
                *uses.entry(m).or_default() += 1;
            }
        }
    }
    let mut missing: BTreeMap<String, (usize, String, String)> = BTreeMap::new();
    let mut undecoded: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut slots: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    let mut checked: HashMap<String, Result<(), String>> = HashMap::new();
    for (&m, &n) in uses.iter().chain(hidden_uses.iter()) {
        let mm = &map.materials[m];
        for t in &mm.textures {
            let slot = texture::sampler_name(t.0).map_or_else(|| format!("{:08x}", t.0), str::to_owned);
            let e = slots.entry(slot.clone()).or_default();
            let Some(info) = map.image_info(&t.3) else {
                if fastfile_iw7::shared::builtin(&t.3).is_some() {
                    e[0] += 1;
                    continue;
                }
                e[1] += 1;
                let x = missing.entry(t.3.clone()).or_insert((0, slot.clone(), mm.name.clone()));
                x.0 += n;
                continue;
            };
            let r = checked.entry(t.3.clone()).or_insert_with(|| {
                let l = map.image(&t.3, max_dim)?;
                texture::level_rgba(&l, 0).map(|_| ()).ok_or_else(|| format!("format {} not decoded", l.format))
            });
            match r {
                Ok(()) => e[0] += 1,
                Err(err) => {
                    e[2] += 1;
                    let x = undecoded.entry(t.3.clone()).or_insert((0, format!("{err} (fmt {}, slot {slot})", info.format)));
                    x.0 += n;
                }
            }
        }
    }
    println!("slots (resolved+decoded, missing, not decoded):");
    for (s, c) in &slots {
        println!("  {s:24} {c:?}");
    }
    let mut miss: Vec<_> = missing.into_iter().collect();
    miss.sort_by_key(|x| std::cmp::Reverse(x.1.0));
    println!("{} pictures missing; most used:", miss.len());
    for (name, (n, slot, mat)) in miss.iter().take(40) {
        println!("  {n:6} {name} ({slot} of {mat})");
    }
    let mut und: Vec<_> = undecoded.into_iter().collect();
    und.sort_by_key(|x| std::cmp::Reverse(x.1.0));
    println!("{} pictures not decoded:", und.len());
    for (name, (n, why)) in und.iter().take(30) {
        println!("  {n:6} {name}: {why}");
    }
    // Technique sets, by the words that matter for the look.
    let mut census: BTreeMap<String, usize> = BTreeMap::new();
    for (&m, &n) in &uses {
        let ts = &map.materials[m].technique_set;
        let l = fastfile_iw7::look::look(ts);
        let key = format!("{:?}{}{}{}", l.blend, if l.unlit { " unlit" } else { "" }, if l.decal { " decal" } else { "" }, l.hidden.map_or(String::new(), |h| format!(" hidden:{h}")));
        *census.entry(key).or_default() += n;
    }
    println!("looks: {census:?}");
    if std::env::var("IW7_AUDIT_TECHSETS").is_ok() {
        let mut by_ts: BTreeMap<&str, (usize, &str)> = BTreeMap::new();
        for (&m, &n) in &uses {
            let mm = &map.materials[m];
            let e = by_ts.entry(mm.technique_set.as_str()).or_insert((0, mm.name.as_str()));
            e.0 += n;
        }
        for (ts, (n, ex)) in by_ts {
            let mm = map.materials.iter().find(|m| m.name == ex).unwrap();
            let tx: Vec<String> = mm.textures.iter().map(|t| format!("{}={}", texture::sampler_name(t.0).map_or_else(|| format!("{:08x}", t.0), str::to_owned), t.3)).collect();
            let consts: Vec<String> = mm.constants.iter().map(|c| format!("{}={:?}", c.0, c.1)).collect();
            println!("  {n:6} {ts}  e.g. {ex} [{}] {{{}}}", tx.join(", "), consts.join(", "));
        }
    }
}
