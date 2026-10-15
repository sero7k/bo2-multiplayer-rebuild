//! What the world surfaces' extra vertex stream holds, by material:
//! `jup_vcol <install> <map>`.

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(install), Some(map)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: jup_vcol <install> <map>");
        std::process::exit(2);
    };
    let oodle = fastfile_jup::oodle::load(std::path::Path::new(install)).expect("oodle");
    let src = fastfile_jup::map::MapSources {
        install: PathBuf::from(install),
        name: map.clone(),
    };
    let m = fastfile_jup::map::JupMap::load(&src, oodle.as_ref()).expect("map");
    let mut by: std::collections::HashMap<Option<usize>, (usize, usize, usize, Vec<u32>)> = Default::default();
    for s in &m.surfaces {
        let e = by.entry(s.material).or_default();
        e.0 += s.tris.len();
        e.1 += 1;
        if !s.extra.is_empty() {
            e.2 += 1;
            if e.3.len() < 12 {
                let step = (s.extra.len() / 6).max(1);
                e.3.extend(s.extra.iter().step_by(step).take(6));
            }
        }
    }
    let mut v: Vec<_> = by.into_iter().collect();
    v.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    let with: usize = m.surfaces.iter().filter(|s| !s.extra.is_empty()).count();
    println!("surfaces with an extra stream: {with} of {}", m.surfaces.len());
    for (mat, (tris, n, ex, sample)) in v.iter().take(40) {
        let colour = mat.and_then(|k| m.materials[k].colour).is_some();
        let pics = mat.map_or(0, |k| m.materials[k].images.len());
        println!(
            "material {mat:?} tris {tris} surfaces {n} with extra {ex} colour {colour} pictures {pics} sample {:?}",
            sample.iter().map(|w| format!("{w:08x}")).collect::<Vec<_>>()
        );
    }
}
