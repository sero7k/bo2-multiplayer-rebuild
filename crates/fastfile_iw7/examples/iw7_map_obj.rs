//! A map read as the game lane reads it (`fastfile_iw7::map`), checked:
//! `iw7_map_obj <map.ff> [<shared ranges.bin>] [<out.obj>]` prints the
//! report and (given `out.obj`) writes the world and placed models as OBJ.
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let zone = std::path::Path::new(&a[1]);
    let mut src = fastfile_iw7::map::MapSources::beside(zone);
    src.shared_ranges = a.get(2).map(Into::into);
    let t = std::time::Instant::now();
    let map = fastfile_iw7::map::Iw7Map::load(&src).expect("map");
    println!("{} read in {:?}", map.name, t.elapsed());
    for l in &map.report {
        println!("  {l}");
    }
    println!("  bounds {:?} .. {:?}; lights {} (first {:?})", map.mins, map.maxs, map.lights.len(), map.lights.first());
    let mut techs: std::collections::BTreeMap<String, usize> = Default::default();
    for m in &map.materials {
        let key: String = m.technique_set.split('_').take(2).collect::<Vec<_>>().join("_");
        *techs.entry(key).or_default() += 1;
    }
    println!("  technique families {techs:?}");
    // Index sanity: every triangle within its surface's vertices.
    let (mut bad, mut tris) = (0usize, 0usize);
    for s in &map.surfaces {
        for k in 0..s.tri_count as usize * 3 {
            let Some(&ix) = map.indices.get(s.base_index as usize + k) else { bad += 1; continue };
            if (s.first_vertex + u32::from(ix)) as usize >= map.vertices.len() || ix >= s.vertex_count {
                bad += 1;
            }
        }
        tris += s.tri_count as usize;
    }
    println!("  {tris} triangles, {bad} indices out of range");
    // A colour map, decoded.
    if let Some(m) = map.materials.iter().find(|m| m.texture("colorMap").is_some()) {
        let name = m.texture("colorMap").unwrap();
        match map.image(name, 512) {
            Ok(l) => println!("  {} colorMap {name}: {}x{} fmt {} levels {}", m.name, l.width, l.height, l.format, l.levels),
            Err(e) => println!("  {} colorMap {name}: {e}", m.name),
        }
    }
    // Collision budget: instance triangles, first and coarsest LOD, by model size.
    let (mut fine, mut coarse, mut small) = (0usize, 0usize, 0usize);
    for sm in &map.static_models {
        let m = &map.models[sm.model];
        let t = |s: &[fastfile_iw7::map::ModelSurface]| s.iter().map(|s| s.tris.len()).sum::<usize>();
        fine += t(&m.surfaces);
        let c = if m.coarse.is_empty() { t(&m.surfaces) } else { t(&m.coarse) };
        coarse += c;
        if m.radius * sm.scale < 24.0 {
            small += c;
        }
    }
    println!("  model instance tris: lod0 {fine}, coarsest {coarse} (of which radius < 24: {small})");
    if let Ok(n) = std::env::var("IW7_RANGES") {
        // The ranges the first n colour maps need (512 px), as a list for copy_ranges.ps1.
        let n: usize = n.parse().unwrap_or(10);
        let mut seen = std::collections::HashSet::new();
        for m in map.materials.iter().filter_map(|m| m.texture("colorMap")).take(n) {
            for f in map.image_ranges(m, 512) {
                if seen.insert((f.file_index, f.offset)) {
                    eprintln!("RANGE {} {} {} {m}", f.file_index, f.offset, f.offset_end);
                }
            }
        }
        for m in map.materials.iter().filter_map(|m| m.texture("colorMap")).take(n) {
            match map.image(m, 512) {
                Ok(l) => println!("  image {m}: {}x{} fmt {} levels {} rgba {}", l.width, l.height, l.format, l.levels, fastfile_iw7::texture::level_rgba(&l, 0).is_some()),
                Err(e) => println!("  image {m}: {e}"),
            }
        }
    }
    println!("  entities head: {}", &map.entities[..map.entities.len().min(300)].replace('\n', " | "));
    let Some(out) = a.get(3) else { return };
    let mut f = std::io::BufWriter::new(std::fs::File::create(out).unwrap());
    for v in &map.vertices {
        writeln!(f, "v {} {} {}", v.xyz[0], v.xyz[2], -v.xyz[1]).unwrap();
    }
    for s in &map.surfaces {
        for t in 0..s.tri_count as usize {
            let i = |k: usize| s.first_vertex + u32::from(map.indices[s.base_index as usize + 3 * t + k]) + 1;
            writeln!(f, "f {} {} {}", i(0), i(1), i(2)).unwrap();
        }
    }
    let mut base = map.vertices.len() as u32 + 1;
    for sm in &map.static_models {
        for s in &map.models[sm.model].surfaces {
            for v in &s.verts {
                let p = [0, 4, 8].map(|o| f32::from_le_bytes(v[o..o + 4].try_into().unwrap()));
                let w: [f32; 3] = std::array::from_fn(|k| sm.origin[k] + (p[0] * sm.axis[0][k] + p[1] * sm.axis[1][k] + p[2] * sm.axis[2][k]) * sm.scale);
                writeln!(f, "v {} {} {}", w[0], w[2], -w[1]).unwrap();
            }
            for t in &s.tris {
                writeln!(f, "f {} {} {}", base + u32::from(t[0]), base + u32::from(t[1]), base + u32::from(t[2])).unwrap();
            }
            base += s.verts.len() as u32;
        }
    }
}
