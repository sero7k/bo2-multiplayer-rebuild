//! bo2mp: a CoD4 / World at War map loaded as a match loads it (the full
//! walk: lane, BO2 rules, common zones, merge and finalize), then every
//! material a drawn world batch or a placed prop uses checked by the colour
//! map the renderer would bind: `legacy_texture_probe <zone.ff>...`. One
//! line per map: `world N/M materials (T tris), props P/Q` untextured, and
//! the first names. Untextured: no colour slot, an image row with no pixels
//! (no decode, payload or pending decode), or a 1x1 stand-in.
use std::collections::BTreeMap;
use std::path::PathBuf;

fn main() {
    for path in std::env::args().skip(1) {
        let started = std::time::Instant::now();
        let zone = PathBuf::from(&path);
        let progress = asset_transport::LoadProgress::default();
        let outcome = bevy::tasks::futures_lite::future::block_on(assets::load_prepared_match(
            Ok(zone.clone()),
            Err("legacy map: no MW2 common zone".to_owned()),
            progress,
        ));
        let prepared = match outcome {
            assets::MatchLoadOutcome::Ready(p) => p,
            assets::MatchLoadOutcome::Canceled => {
                println!("{path}: canceled");
                continue;
            }
        };
        let pop = &prepared.materials.population;
        let ids = &prepared.materials.map_ids;
        let canonical = |local: usize| ids.get(local).copied().flatten();
        let why = |m: usize| -> Option<String> {
            let mat = pop.materials.get(m)?;
            if mat.t6_draw.is_some_and(|d| matches!(d.blend, asset_core::T6Blend::ShadowOnly | asset_core::T6Blend::Sky)) {
                return None;
            }
            let name = mat.name.as_str();
            let colour = mat.textures.iter().find(|t| t.semantic == asset_material::TS_COLOR_MAP);
            match colour.and_then(|t| t.image).and_then(|i| pop.images.get(i).map(|img| (i, img))) {
                None => Some(format!("{name} (no colour map{})", if mat.t6_draw.is_none() { ", no t6 draw" } else { "" })),
                Some((_, img)) if img.width <= 1 && img.height <= 1 => Some(format!("{name} [{} 1x1]", img.name.as_str())),
                Some((_, img)) if img.decoded.is_none() && img.payload.is_empty() && img.pending_decode.is_none() => {
                    Some(format!("{name} [{}: no pixels]", img.name.as_str()))
                }
                Some(_) if mat.t6_draw.is_none() => Some(format!("{name} (no t6 draw)")),
                Some(_) => None,
            }
        };
        let mut world: BTreeMap<Option<usize>, u64> = BTreeMap::new();
        if let Some(draw) = &prepared.world.draw {
            for b in &draw.batches {
                *world.entry(b.material.and_then(canonical)).or_default() += (b.packed_indices.len() / 3) as u64;
            }
        }
        let mut props: BTreeMap<Option<usize>, u32> = BTreeMap::new();
        for mesh in &prepared.world.static_model_meshes {
            for s in mesh.lod_surfaces.iter().flatten() {
                *props.entry(s.material.and_then(canonical)).or_default() += 1;
            }
        }
        let mut names = Vec::new();
        let (mut wm, mut wt) = (0usize, 0u64);
        for (m, tris) in &world {
            let n = match m {
                Some(m) => why(*m),
                None => Some("(no material)".to_owned()),
            };
            if let Some(n) = n {
                wm += 1;
                wt += tris;
                names.push(format!("world {n} {tris} tris"));
            }
        }
        let mut pm = 0usize;
        for m in props.keys() {
            let n = match m {
                Some(m) => why(*m),
                None => Some("(no material)".to_owned()),
            };
            if let Some(n) = n {
                pm += 1;
                names.push(format!("prop {n}"));
            }
        }
        let gaps: Vec<&String> = prepared.report.iter().filter(|l| l.contains("material images") || l.contains("never decoded")).collect();
        println!(
            "{} [{:.0}s] untextured: world {wm}/{} materials ({wt} tris), props {pm}/{} {names:?} {gaps:?}",
            zone.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned()),
            started.elapsed().as_secs_f32(),
            world.len(),
            props.len()
        );
    }
}
