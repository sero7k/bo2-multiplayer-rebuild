//! bo2mp `iw4l check-maps`: one map from another game read the way a match
//! reads it - its zone opened by its own lane (MW2 and its usermaps,
//! Call of Duty 4, World at War), its world draw, collision and spawn
//! points built - without starting a match. One line per map: `OK:` with
//! what was built, or `FAIL` with the first reason a match would refuse it.

use std::path::Path;

use asset_transport::progress::LoadProgress;

use crate::lane::{LoadedWorld, lane};

/// The world built for one map, as a line.
fn summary(loaded: &LoadedWorld) -> Result<String, String> {
    let draw = loaded.world.draw.as_ref();
    let surfaces = draw.map_or(0, |d| d.stats.surfaces);
    let batches = draw.map_or(0, |d| d.batches.len());
    let triangles = draw.map_or(0, |d| d.stats.triangles);
    let gaps: Vec<String> = loaded.gaps.iter().map(|g| g.reason.clone()).collect();
    let brushes = loaded.collision.as_ref().map(|c| c.brushes.len());
    let line = format!(
        "world {surfaces} surfaces in {batches} batches ({triangles} triangles); collision {}; {} spawn points",
        brushes.map_or_else(|| "none".to_owned(), |b| format!("{b} brushes")),
        loaded.spawns.len()
    );
    if draw.is_none() || batches == 0 {
        return Err(format!(
            "no drawable world{}",
            if gaps.is_empty() { String::new() } else { format!(": {}", gaps.join("; ")) }
        ));
    }
    if loaded.collision.is_none() {
        return Err(format!("no collision: {}; {line}", gaps.join("; ")));
    }
    if loaded.spawns.is_empty() {
        return Err(format!("no spawn points; {line}"));
    }
    if gaps.is_empty() {
        Ok(line)
    } else {
        Ok(format!("{line}; gaps: {}", gaps.join("; ")))
    }
}

/// One map, by its game prefix (`iw4`, `iw3`, `t4`) and zone file. MW2
/// zones (stock, IW4x's packs, usermaps) also get the picture census of
/// [`crate::check_iw4_map`].
pub fn check_map(game: &str, path: &Path) -> String {
    let progress = LoadProgress::new(0);
    let loaded = match game {
        "iw3" => crate::lane::load_iw3_world(path, &progress, Default::default()),
        "t4" => crate::lane::load_t4_world(path, &progress, Default::default()),
        // bo2mp: Ghosts' maps.
        "iw6" => crate::lane::load_iw6_world(path, &progress, Default::default()),
        // bo2mp: Infinite Warfare's maps.
        "iw7" => crate::lane::load_iw7_world(path, &progress, Default::default()),
        // bo2mp: Black Ops III's maps.
        "t7" => crate::lane::load_t7_world(path, &progress, Default::default()),
        "iw4" => {
            let image = match asset_transport::open_zone(path) {
                Ok(image) => image,
                Err(e) => return format!("FAIL zone does not open: {e}"),
            };
            let mut visions = std::collections::BTreeMap::new();
            lane(image.game).load_world(
                path,
                &image,
                &progress,
                Default::default(),
                Default::default(),
                &mut visions,
            )
        }
        other => return format!("FAIL no map check for `{other}` zones"),
    };
    let world = summary(&loaded);
    match (game, world) {
        ("iw4", Ok(line)) => {
            let pictures = crate::check_iw4_map(path);
            if pictures.starts_with("OK:") {
                format!("OK: {line}; {}", pictures.trim_start_matches("OK: "))
            } else {
                format!("{pictures}; {line}")
            }
        }
        (_, Ok(line)) => {
            let materials = loaded
                .report
                .iter()
                .filter(|l| l.starts_with(&format!("{game} materials")) || l.starts_with(&format!("{game} lightmaps")) || l.starts_with(&format!("{game} fog")))
                .cloned()
                .collect::<Vec<_>>()
                .join("; ");
            format!("OK: {line}; {materials}; {}", legacy_texture_census(&loaded))
        }
        (_, Err(why)) => format!("FAIL {why}"),
    }
}

/// bo2mp: what a CoD4 / World at War map draws untextured: every material
/// a drawn world batch or a placed prop uses, by its colour binding as the
/// renderer will see it (no colour slot, no image, or the lane's 1x1 grey
/// stand-in for a picture it did not find). `textures missing: world N/M
/// materials (T tris), props P/Q [names]`.
fn legacy_texture_census(loaded: &LoadedWorld) -> String {
    use std::collections::BTreeMap;
    let cat = &loaded.materials;
    let untextured = |m: usize| -> Option<String> {
        let mat = cat.materials.get(m)?;
        let name = mat.name.as_str().to_owned();
        // (The shadow-only and sky surfaces draw no colour.)
        if mat.t6_draw.is_some_and(|d| matches!(d.blend, asset_core::T6Blend::ShadowOnly | asset_core::T6Blend::Sky)) {
            return None;
        }
        let colour = mat.textures.iter().find(|t| t.semantic == asset_material::TS_COLOR_MAP);
        let image = colour.and_then(|t| t.image).and_then(|i| cat.images.get(i));
        match image {
            None => Some(format!("{name} (no colour map)")),
            // (A 1x1 picture is the grey stand-in, unless it is a water
            // colour the lane made.)
            Some(img) if img.width <= 1 && img.height <= 1 && !img.name.as_str().ends_with("#water") => {
                Some(format!("{name} [{}]", img.name.as_str()))
            }
            // The editor's placeholder (`case64blue`) drawn as a picture.
            Some(img) if img.name.as_str().contains("case64") => Some(format!("{name} [{} placeholder]", img.name.as_str())),
            Some(img) if img.decoded.is_none() && img.payload.is_empty() && img.pending_decode.is_none() => {
                Some(format!("{name} [{}: no pixels]", img.name.as_str()))
            }
            Some(_) => None,
        }
    };
    let mut world: BTreeMap<usize, u64> = BTreeMap::new();
    if let Some(draw) = &loaded.world.draw {
        for b in &draw.batches {
            if let Some(m) = b.material {
                *world.entry(m).or_default() += (b.packed_indices.len() / 3) as u64;
            }
        }
    }
    let mut props: BTreeMap<usize, ()> = BTreeMap::new();
    for mesh in &loaded.world.static_model_meshes {
        for s in mesh.lod_surfaces.iter().flatten() {
            if let Some(m) = s.material {
                props.insert(m, ());
            }
        }
    }
    let mut names = Vec::new();
    let (mut wm, mut wt) = (0usize, 0u64);
    for (&m, &tris) in &world {
        if let Some(n) = untextured(m) {
            wm += 1;
            wt += tris;
            names.push(format!("world {n} {tris} tris"));
        }
    }
    let mut pm = 0usize;
    for &m in props.keys() {
        if let Some(n) = untextured(m) {
            pm += 1;
            names.push(format!("prop {n}"));
        }
    }
    format!(
        "textures missing: world {wm}/{} materials ({wt} tris), props {pm}/{} {names:?}",
        world.len(),
        props.len()
    )
}
