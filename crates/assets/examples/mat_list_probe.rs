//! Lists a map's world materials by surface area share: `mat_list_probe t4|t6 <zone.ff>`.
use std::collections::HashMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&a[2]);
    match a[1].as_str() {
        "t4" => {
            let z = t4::load_map(path).unwrap().zone;
            let w = z.gfx_world().unwrap();
            let mut tris: HashMap<usize, u64> = HashMap::new();
            for s in &w.surfaces {
                if let Some(m) = s.material {
                    *tris.entry(m).or_default() += s.tri_count as u64;
                }
            }
            let mut v: Vec<_> = tris.into_iter().collect();
            v.sort_by_key(|(_, t)| std::cmp::Reverse(*t));
            for (m, t) in v {
                let mat = z.material(m).unwrap();
                let tex: Vec<String> = mat
                    .textures
                    .iter()
                    .map(|t| format!("{:?}={}", t.semantic, t.image.and_then(|i| z.image(i)).map(|i| i.name.as_str()).unwrap_or("-")))
                    .collect();
                let ts = mat.technique_set.and_then(|t| z.technique_set(t)).map(|t| t.name.as_str()).unwrap_or("");
                println!("{t}\t{}\tsurf={:#x}\t{ts}\t{}", mat.name, mat.surface_type_bits, tex.join(" "));
            }
        }
        _ => {
            let c = asset_t6::capture_zone(path).unwrap();
            let w = c.world.as_ref().unwrap();
            let mut tris: HashMap<usize, u64> = HashMap::new();
            for s in &w.surfaces {
                if let Some(m) = s.material {
                    *tris.entry(m.index).or_default() += s.tri_count as u64;
                }
            }
            let mut v: Vec<_> = tris.into_iter().collect();
            v.sort_by_key(|(_, t)| std::cmp::Reverse(*t));
            for (m, t) in v {
                let mat = &c.materials[m];
                let tex: Vec<String> = mat
                    .textures
                    .iter()
                    .map(|t| format!("{:08x}={}", t.name_hash, t.image.map(|i| c.images[i.index].name.as_str()).unwrap_or("-")))
                    .collect();
                println!("{t}\t{}\tsurf={:#x}\t{}", mat.name, mat.surface_type_bits, tex.join(" "));
            }
        }
    }
}
