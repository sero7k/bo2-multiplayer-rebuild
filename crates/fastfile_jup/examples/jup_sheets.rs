//! Lists a map's surfaces wider than 16384 units (sea, sky, far ground)
//! with their materials' pictures: `jup_sheets <install> <map>`.

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(install), Some(map)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: jup_sheets <install> <map>");
        std::process::exit(2);
    };
    let oodle = fastfile_jup::oodle::load(std::path::Path::new(install)).expect("oodle");
    let src = fastfile_jup::map::MapSources {
        install: PathBuf::from(install),
        name: map.clone(),
    };
    let m = fastfile_jup::map::JupMap::load(&src, oodle.as_ref()).expect("map");
    for s in &m.surfaces {
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in &s.positions {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        if hi[0] - lo[0] <= 16384.0 && hi[1] - lo[1] <= 16384.0 {
            continue;
        }
        let pics: Vec<String> = s
            .material
            .map(|k| {
                m.materials[k]
                    .images
                    .iter()
                    .map(|&i| format!("{:#x}:{}x{}", m.images[i].format, m.images[i].width, m.images[i].height))
                    .collect()
            })
            .unwrap_or_default();
        println!(
            "surface {} zone {} verts {} tris {} lo {:?} hi {:?} material {:?} {:016x} pictures {pics:?} uv0 {:?}",
            s.index,
            s.zone,
            s.positions.len(),
            s.tris.len(),
            lo.map(|v| v.round()),
            hi.map(|v| v.round()),
            s.material,
            s.material.map_or(0, |k| m.materials[k].hash),
            s.uvs.first()
        );
    }
    // Props far out or huge (a sky dome?).
    for (i, pm) in m.prop_models.iter().enumerate() {
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for s in &pm.surfaces {
            for p in &s.positions {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
        }
        let n = m.props.iter().filter(|p| p.model == i).count();
        let find = std::env::var("JUP_FIND").unwrap_or_default();
        let found = find.split(',').filter(|w| !w.is_empty()).any(|w| pm.name.contains(w));
        if hi[0] - lo[0] > 8000.0 || found {
            println!("prop model {i} {} placed {n} lo {:?} hi {:?}", pm.name, lo.map(|v| v.round()), hi.map(|v| v.round()));
        }
    }
}
