//! bo2mp: Black Ops II multiplayer on the same runtime as Zombies.
//!
//! A multiplayer map (`mp_*`) runs its game type from `maps/mp/gametypes/`
//! (Zombies': `maps/mp/gametypes_zm/`). Its scripts call one script no PC
//! zone carries, `maps/mp/gametypes/_globallogic` (the match: start, clock,
//! limits, end, next round); the engine provides it as GSC source
//! ([`GLOBALLOGIC`], OUR CODE written from what the game's scripts call and
//! wait for), compiled with `gsc_t6::compile` and linked beside the zones'.

use bevy_ecs::prelude::World;
use gsc_t6::ScriptObject;

use super::Zm;

/// Our `maps/mp/gametypes/_globallogic`.
const GLOBALLOGIC: &str = include_str!("scripts/mp_globallogic.gsc");

static MOAB_KILLS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// bo2mp: the lobby's M.O.A.B. setting for the next match: the kills in
/// one life that earn it, 0 for none (native Black Ops II).
pub fn set_moab_kills(kills: u32) {
    MOAB_KILLS.store(kills, std::sync::atomic::Ordering::Relaxed);
}

pub(super) fn moab_kills() -> u32 {
    MOAB_KILLS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Our optional M.O.A.B. (`bo2mp/moab`, off unless `scr_moab 1`).
const MOAB: &str = include_str!("scripts/moab.gsc");

/// The M.O.A.B.'s two visions, in the IW games' film form: a white-hot,
/// orange-lit flash, then a dim, desaturated, ash-brown aftermath. OUR
/// values, after the look of Modern Warfare 2's `mpnuke` and
/// `mpnuke_aftermath` (high contrast, half desaturated, warm light tint
/// for the flash; cold-dark, red-brown dark tint for the aftermath).
pub(super) fn register_moab_visions() {
    let flash = asset_world::FilmVision {
        enable: true,
        contrast: 1.6,
        brightness: 0.03,
        desaturation: 0.45,
        light_tint: [1.9, 1.3, 0.7],
        medium_tint: [1.2, 1.05, 0.9],
        dark_tint: [0.75, 0.9, 1.05],
        ..Default::default()
    };
    let aftermath = asset_world::FilmVision {
        enable: true,
        contrast: 1.7,
        brightness: 0.04,
        desaturation: 0.6,
        light_tint: [1.0, 0.95, 0.88],
        medium_tint: [0.9, 0.75, 0.65],
        dark_tint: [0.7, 0.32, 0.2],
        ..Default::default()
    };
    asset_world::register_film_visions(vec![("mpnuke".to_owned(), flash), ("mpnuke_aftermath".to_owned(), aftermath)]);
}

/// A multiplayer map, by its zone name.
pub(super) fn is_mp(map: &str) -> bool {
    map.starts_with("mp_")
}

/// The scripts the engine provides for a multiplayer map, compiled.
/// BO2MP_DEBUG_GSC=<file>: a script of ours (`bo2mp/debug`) whose `main`
/// runs on the level once the match has started (a test aid: print what
/// the game's scripts hold, `println(...)` lands in the log).
pub(super) fn engine_scripts() -> Result<Vec<ScriptObject>, String> {
    let mut out = vec![
        gsc_t6::compile("maps/mp/gametypes/_globallogic", GLOBALLOGIC)?,
        gsc_t6::compile("bo2mp/moab", MOAB)?,
    ];
    register_moab_visions();
    if let Ok(path) = std::env::var("BO2MP_DEBUG_GSC") {
        let src = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
        out.push(gsc_t6::compile("bo2mp/debug", &src)?);
    }
    Ok(out)
}

/// Where the game type's scripts live.
pub(super) fn gametypes(world: &World) -> &'static str {
    if world.get_resource::<Zm>().is_some_and(|z| z.mp) {
        "maps/mp/gametypes"
    } else {
        "maps/mp/gametypes_zm"
    }
}

/// The engine's callbacks script (`codecallback_*`).
pub(super) fn callbacks(world: &World) -> String {
    format!("{}/_callbacksetup", gametypes(world))
}

#[cfg(test)]
mod globallogic_tests {
    #[test]
    fn our_globallogic_compiles() {
        let scripts = super::engine_scripts().expect("mp_globallogic.gsc compiles");
        assert!(!scripts.is_empty());
    }
}

#[cfg(test)]
mod moab_tests {
    #[test]
    fn the_moab_script_compiles_with_its_parts() {
        let s = gsc_t6::compile("bo2mp/moab", super::MOAB).expect("moab.gsc compiles");
        for f in ["main", "moab_watchstreak", "moab_launch", "moab_deaths", "moab_ticks"] {
            assert!(s.functions.iter().any(|x| x.name == f), "{f}");
        }
    }

    #[test]
    fn the_moab_is_off_unless_chosen() {
        assert_eq!(super::moab_kills(), 0);
        assert!(super::MOAB.contains("getdvarint( \"scr_moab\" ) == 0"));
    }
}
