//! bo2mp: the world surfaces over a point, for reading a Black Ops III
//! map's materials where they show: `t7_surfs <zone.ff> <x> <y> [r]` prints
//! each world surface whose bounds come within r units (default 64) of the
//! point in x and y: its material, technique set, textures and constants,
//! and the mean vertex colour (bytes) over it.
use fastfile_t7::map::{MapSources, T7Map};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let map = T7Map::load(&MapSources::beside(std::path::Path::new(&a[1]))).expect("map");
    let f = |i: usize, d: f32| a.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
    let (x, y, r) = (f(2, 0.0), f(3, 0.0), f(4, 64.0));
    for (i, s) in map.surfaces.iter().enumerate() {
        if x < s.mins[0] - r || x > s.maxs[0] + r || y < s.mins[1] - r || y > s.maxs[1] + r {
            continue;
        }
        let Some(m) = s.material.map(|m| &map.materials[m]) else { continue };
        let mut mean = [0u64; 4];
        let n = s.vertex_count.max(1) as u64;
        for v in map.vertices.iter().skip(s.first_vertex as usize).take(s.vertex_count as usize) {
            for (k, c) in mean.iter_mut().enumerate() {
                *c += u64::from((v.color >> (8 * k)) & 0xff);
            }
        }
        println!(
            "surface {i}: z {:.0}..{:.0}, {} tris, vertex colour {:?}: {} ({})",
            s.mins[2],
            s.maxs[2],
            s.tri_count,
            mean.map(|c| c / n),
            m.name,
            m.technique_set
        );
        for (h, t) in &m.textures {
            println!("    {h:08x} {t}");
        }
        for (k, v) in m.constants.iter() {
            println!("    {k} = {v:?}");
        }
    }
}
