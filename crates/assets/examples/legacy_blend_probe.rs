//! For a CoD4 / World at War map: how the vertex colour channels vary on
//! surfaces with layered (`b1c1...`) materials vs plain ones, to tell which
//! channel carries the layer blend: `legacy_blend_probe <zone.ff>`.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let z = t4::load_map(std::path::Path::new(&a[1])).unwrap().zone;
    let w = z.gfx_world().expect("world");
    // [channel][layers 1..=3] -> (min, max, sum, n)
    let mut st = [[(255u32, 0u32, 0u64, 0u64); 3]; 4];
    for s in &w.surfaces {
        let Some(m) = s.material.and_then(|id| z.material(id)) else { continue };
        let ts = m.technique_set.and_then(|t| z.technique_set(t)).map_or("", |t| t.name.as_str());
        let layers = 1 + ts.matches("_b1").count() + ts.matches("_b2").count();
        let li = layers.clamp(1, 3) - 1;
        for v in &w.vertices[s.first_vertex.max(0) as usize..][..usize::from(s.vertex_count)] {
            for c in 0..4 {
                let x = (v.color >> (8 * c)) & 0xff;
                let e = &mut st[c][li];
                e.0 = e.0.min(x);
                e.1 = e.1.max(x);
                e.2 += u64::from(x);
                e.3 += 1;
            }
        }
    }
    for (li, name) in ["plain", "2 layers", "3 layers"].iter().enumerate() {
        let line: Vec<String> = (0..4)
            .map(|c| {
                let e = st[c][li];
                format!("byte{c} min {} max {} mean {:.0}", e.0, e.1, if e.3 > 0 { e.2 as f64 / e.3 as f64 } else { 0.0 })
            })
            .collect();
        println!("{name}: {}", line.join(" | "));
    }
}
