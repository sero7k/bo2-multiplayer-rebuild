//! bo2mp: World at War (T4) maps.
//!
//! A WaW map zone (`IWffu100`, version 0x183) is read with the `t4` crate
//! (ported from COD4-Rewrite; its schema is OpenAssetTools' T4 definitions)
//! and converted to an `iw3::zone::Zone`: World at War is CoD4's engine one
//! step on, so the CoD4 lane's builders then make its world draw, collision,
//! entities and spawns (`super::cod4::build_legacy_world`). Colour maps are
//! read from WaW's own IWDs (`main/iw_*.iwd`, `.iwi` version 6 as CoD4's).

use std::path::Path;

use asset_material::MaterialCatalog;
use asset_transport::progress::{LoadProgress, StageId};
use asset_world::WorldDrawPolicy;

use super::LoadedWorld;
use super::cod4::{Legacy, build_legacy_world};
use crate::lane_capability::PreparedCapability;

const WAW: Legacy = Legacy {
    tag: "t4",
    title: "World at War",
    stage_materials: "waw materials",
    addr_no_world: "assets::lane::t4::load_world/no_world",
    addr_world_mesh: "assets::lane::t4::load_world/world_mesh",
};

/// Loads a World at War map zone into the match's world. A zone that cannot
/// be read becomes a gap with the reason, never a crash: the reader's own
/// errors are reported, and a panic in it is caught.
pub(crate) fn load_t4_world(path: &Path, progress: &LoadProgress, material_seed: MaterialCatalog) -> LoadedWorld {
    let mut report = vec![format!("game: T4 / World at War ({})", path.display())];
    let stage = progress.begin_scoped(StageId::MapAssets, "waw zone", None);
    let read = std::panic::catch_unwind(|| t4::load_map(path));
    let map = match read {
        Ok(Ok(map)) => {
            stage.done();
            map
        }
        Ok(Err(error)) => {
            stage.fail();
            diag::warn!(World, "bo2mp t4: {}: {error:#}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("World at War zone: {error:#}"),
                Some("assets::lane::t4::load_world/zone"),
            );
        }
        Err(panic) => {
            stage.fail();
            let what = panic
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_owned());
            diag::warn!(World, "bo2mp t4: {}: reader panicked: {what}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("World at War zone could not be read: {what}"),
                Some("assets::lane::t4::load_world/panic"),
            );
        }
    };
    if let Some(stop) = &map.zone.stats.stopped_at {
        // Assets after the world (weapons, effects, menus) are not needed
        // for the map: a stop there still plays.
        diag::info!(World, "bo2mp t4: {}: reading stopped at {stop}", path.display());
    }
    for note in &map.notes {
        diag::warn!(World, "bo2mp {note}");
    }
    report.extend(map.notes.iter().cloned());
    let loaded = build_legacy_world(&WAW, path, &map.zone, report, progress, material_seed);
    diag::info!(
        World,
        "bo2mp t4: {} loaded: {}",
        path.display(),
        loaded.report.iter().filter(|l| l.starts_with("t4 ") || l.starts_with("world mesh")).cloned().collect::<Vec<_>>().join("; ")
    );
    loaded
}
