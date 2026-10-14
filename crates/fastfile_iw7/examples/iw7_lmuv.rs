//! Lightmap coordinates against world size per surface:
//! `iw7_lmuv <map.ff> [n]`: for the biggest surfaces, their world extent,
//! their lightmap coordinates' span and the texels a unit that makes.
use fastfile_iw7::map::{Iw7Map, MapSources};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let map = Iw7Map::load(&MapSources::beside(std::path::Path::new(&a[1]))).expect("map");
    let n: usize = a.get(2).and_then(|v| v.parse().ok()).unwrap_or(12);
    let mut order: Vec<usize> = (0..map.surfaces.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(map.surfaces[i].tri_count));
    for &i in order.iter().take(n) {
        let s = &map.surfaces[i];
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        let (mut ulo, mut uhi) = ([f32::MAX; 2], [f32::MIN; 2]);
        let mut raw = Vec::new();
        for t in 0..3 * s.tri_count as usize {
            let k = u32::from(map.indices[s.base_index as usize + t]);
            let v = &map.vertices[(s.first_vertex + k) as usize];
            for j in 0..3 {
                lo[j] = lo[j].min(v.xyz[j]);
                hi[j] = hi[j].max(v.xyz[j]);
            }
            for j in 0..2 {
                ulo[j] = ulo[j].min(v.lmap_uv[j]);
                uhi[j] = uhi[j].max(v.lmap_uv[j]);
            }
            if raw.len() < 3 {
                raw.push((v.xyz, v.uv, v.lmap_uv));
            }
        }
        let ext: Vec<f32> = (0..3).map(|j| hi[j] - lo[j]).collect();
        let span = [(uhi[0] - ulo[0]) * 4096.0, (uhi[1] - ulo[1]) * 2048.0];
        println!(
            "surf {i} lm {} tris {} mat {} ext {:?} lm uv {:?}..{:?} texels {:?}",
            s.lightmap_index,
            s.tri_count,
            s.material.map_or("-", |m| map.materials[m].name.as_str()),
            ext.iter().map(|e| e.round()).collect::<Vec<_>>(),
            ulo,
            uhi,
            span
        );
        for r in raw {
            println!("    {r:?}");
        }
    }
}
