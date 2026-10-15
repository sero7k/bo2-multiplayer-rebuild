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
        // bo2mp: Modern Warfare III's (2023) maps.
        "jup" => crate::lane::load_jup_world(path, &progress, Default::default()),
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
            format!("OK: {line}; {materials}")
        }
        (_, Err(why)) => format!("FAIL {why}"),
    }
}
