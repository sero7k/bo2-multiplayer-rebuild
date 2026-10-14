//! bo2mp: Black Ops II Zombies on a Zombies map from another game
//! (Infinite Warfare's `cp_zmb`, `cp_rave`, ...). Such a map carries no
//! Black Ops II map script; ours (`scripts/zm_foreign_map.gsc`) is linked
//! as its `maps/mp/<map>` and sets the level up as Black Ops II's own
//! Zombies maps do, from the entities the map's lane wrote
//! (`assets::lane::iw7_zombies`). Everything after is Black Ops II's own
//! Zombies scripts. Not part of upstream IW4L.

use gsc_t6::ScriptObject;

/// Our map script for these maps.
const MAP_SCRIPT: &str = include_str!("scripts/zm_foreign_map.gsc");

/// Other games' Zombies maps registered by name (Black Ops III's, whose
/// names are `zm_` too).
static REGISTERED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// bo2mp: a map that plays this way though its name does not say so.
pub fn register_foreign_zm_map(name: &str) {
    let mut g = REGISTERED.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if !g.iter().any(|n| n == name) {
        g.push(name.to_owned());
    }
}

/// A Zombies map from another game: Infinite Warfare's (`cp_*`), or one
/// registered.
pub fn is_foreign_zm_map(map: &str) -> bool {
    map.starts_with("cp_") || REGISTERED.lock().unwrap_or_else(std::sync::PoisonError::into_inner).iter().any(|n| n == map)
}

/// The map script to link for `map`, when it is such a map and the zones
/// carry none of their own.
pub(super) fn map_script(map: &str, objects: &[ScriptObject]) -> Result<Option<ScriptObject>, String> {
    let name = format!("maps/mp/{map}");
    if !is_foreign_zm_map(map) || objects.iter().any(|o| o.name.trim_end_matches(".gsc") == name) {
        return Ok(None);
    }
    gsc_t6::compile(&name, MAP_SCRIPT).map(Some)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_foreign_map_script_compiles_as_the_map() {
        let s = super::map_script("cp_zmb", &[]).expect("zm_foreign_map.gsc compiles").expect("a script");
        assert_eq!(s.name.trim_end_matches(".gsc"), "maps/mp/cp_zmb");
        for f in ["gamemode_callback_setup", "main", "location_main", "foreign_zone_init", "power_switches", "portals"] {
            assert!(s.functions.iter().any(|x| x.name == f), "{f}");
        }
        assert!(super::map_script("zm_buried", &[]).unwrap().is_none());
    }
}
