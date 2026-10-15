//! What a map's world vertex colours hold, by how many layers the surface's
//! material blends: `iw7_vcol <map.ff>` prints, for one-, two- and
//! three-layer materials, each colour byte's (in memory order) mean, the
//! share of vertices at 0 and at 255, and a few technique sets.
use fastfile_iw7::map::{Iw7Map, MapSources};

fn layers(techset: &str) -> usize {
    1 + techset
        .split('_')
        .filter(|t| {
            let b = t.as_bytes();
            b.len() >= 3 && matches!(b[0], b'b' | b'm' | b'a') && (b'1'..=b'3').contains(&b[1]) && b[2] == b'c'
        })
        .count()
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: iw7_vcol <map.ff>");
    let map = Iw7Map::load(&MapSources::beside(std::path::Path::new(&path))).expect("load");
    let mut stats = [[(0f64, 0usize, 0usize); 4]; 4];
    let mut counts = [0usize; 4];
    for s in &map.surfaces {
        let Some(m) = s.material else { continue };
        let l = layers(&map.materials[m].technique_set).min(3);
        for v in &map.vertices[s.first_vertex as usize..(s.first_vertex as usize + usize::from(s.vertex_count)).min(map.vertices.len())] {
            let b = v.color.to_le_bytes();
            for k in 0..4 {
                let st = &mut stats[l][k];
                st.0 += f64::from(b[k]);
                st.1 += usize::from(b[k] == 0);
                st.2 += usize::from(b[k] == 255);
            }
            counts[l] += 1;
        }
    }
    for l in 1..4 {
        let n = counts[l].max(1) as f64;
        println!(
            "{l} layer(s): {} vertices; bytes 0..3: {}",
            counts[l],
            stats[l].iter().map(|s| format!("mean {:.0} zero {:.0}% full {:.0}%", s.0 / n, s.1 as f64 * 100.0 / n, s.2 as f64 * 100.0 / n)).collect::<Vec<_>>().join(" | ")
        );
    }
    // Per two-layer surface: a few vertices' colours.
    for s in map.surfaces.iter().filter(|s| s.material.is_some_and(|m| layers(&map.materials[m].technique_set) == 2)).take(6) {
        let m = &map.materials[s.material.unwrap()];
        let vs: Vec<String> = map.vertices[s.first_vertex as usize..]
            .iter()
            .take(usize::from(s.vertex_count).min(6))
            .map(|v| format!("{:08x}", v.color))
            .collect();
        println!("  {} {} {vs:?}", m.name, m.technique_set);
    }
}
