//! bo2mp: dedicated server map packs (`server_pack`): one built from a map
//! as its game's reader reads it, and a match's map world built from one
//! (collision, spawns, entities and the placements read from them, no
//! drawing), which Black Ops II's rules then take as they take the map's
//! own. Maps from the other games only: a Black Ops II map is read from
//! Black Ops II's zones, which the server has for the rules anyway.

use std::path::Path;

use asset_transport::progress::LoadProgress;
use server_pack::MapPack;

use crate::lane::{LoadedWorld, lane};

/// Reads a map as a match reads it and keeps its gameplay half.
pub fn build_map_pack(game: &str, map: &str, path: &Path) -> Result<MapPack, String> {
    let progress = LoadProgress::new(0);
    let seed = asset_material::MaterialCatalog::default();
    let loaded = match game {
        "iw3" => crate::lane::load_iw3_world(path, &progress, seed),
        "t4" => crate::lane::load_t4_world(path, &progress, seed),
        "iw6" => crate::lane::load_iw6_world(path, &progress, seed),
        "iw7" => crate::lane::load_iw7_world(path, &progress, seed),
        "t7" => crate::lane::load_t7_world(path, &progress, seed),
        "jup" => crate::lane::load_jup_world(path, &progress, seed),
        "t6" => return Err("Black Ops II maps are read from Black Ops II's zones".to_owned()),
        _ => {
            let image =
                asset_transport::open_zone(path).map_err(|e| format!("zone does not open: {e}"))?;
            let mut visions = std::collections::BTreeMap::new();
            lane(image.game).load_world(
                path,
                &image,
                &progress,
                Default::default(),
                seed,
                &mut visions,
            )
        }
    };
    let collision = loaded.collision.ok_or_else(|| {
        format!(
            "no collision: {}",
            loaded
                .gaps
                .iter()
                .map(|g| g.reason.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    if loaded.spawns.is_empty() {
        return Err("no spawn points".to_owned());
    }
    let w = loaded.world;
    Ok(MapPack {
        game: game.to_owned(),
        map: map.to_owned(),
        entities: loaded
            .scripts
            .entities()
            .map_or_else(String::new, str::to_owned),
        spawns: loaded.spawns,
        intermission_view: w.intermission_view,
        script_brush_models: w.script_brush_models,
        script_structs: w.script_structs,
        flag_descriptors: w.flag_descriptors,
        min: w.min,
        max: w.max,
        world_bounds: w.world_bounds,
        collision: Some(collision),
    })
}

/// The map world a match builds from a pack (nothing to draw).
pub(crate) fn loaded_world_from_pack(pack: MapPack) -> LoadedWorld {
    let mut loaded = LoadedWorld::default();
    loaded.scripts.set_entities(pack.entities);
    loaded.collision = pack.collision;
    loaded.spawns = pack.spawns;
    let w = &mut loaded.world;
    w.intermission_view = pack.intermission_view;
    w.script_brush_models = pack.script_brush_models;
    w.script_structs = pack.script_structs;
    w.flag_descriptors = pack.flag_descriptors;
    w.min = pack.min;
    w.max = pack.max;
    w.world_bounds = pack.world_bounds;
    loaded.report.push(format!(
        "server pack: {}:{} (collision, spawns and entities; nothing drawn)",
        pack.game, pack.map
    ));
    loaded
}
