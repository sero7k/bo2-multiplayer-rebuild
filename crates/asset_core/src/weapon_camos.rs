//! bo2mp: Black Ops II weapon camos, shared by the match load (which reads
//! them), the client (which knows who wears which) and the draw.
//!
//! A camo is a weapon option (`mp/attachmenttable.csv` "camo" rows, the
//! number Create-a-Class saves as `primarycamo` / `secondarycamo`).
//! `mp/weaponoptions.csv` says how it looks: a material camo (gold,
//! diamond, carbon fibre, the DLC camos) swaps each of a gun's camo-ready
//! materials for the camo's material, its pictures tiled across the gun
//! (the gun's `WeaponCamo` material sets); a pattern camo lays a pattern
//! over them (not drawn yet).
//!
//! Who wears what rides on each player's script values (`bo2mp_camos`:
//! `<weapon>:<option>,...`, written by the server's `giveweapon`).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A camo's material for one of a gun's materials: the catalog material
/// drawn instead and how many times its pictures repeat across the gun.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCamo {
    pub material: u32,
    pub tile: f32,
}

/// The match's camos: per material camo option, each gun material it
/// replaces (catalog index) and what it draws instead.
#[derive(Clone, Debug, Default)]
pub struct CamoTable {
    pub swaps: HashMap<(u32, u32), MaterialCamo>,
    /// Options that are material camos, and their names (`camo_gold`).
    pub options: HashMap<u32, String>,
}

static TABLE: RwLock<Option<Arc<CamoTable>>> = RwLock::new(None);

/// The match load's camos (None: no camos this match).
pub fn install(table: Option<CamoTable>) {
    if let Ok(mut t) = TABLE.write() {
        *t = table.map(Arc::new);
    }
}

pub fn table() -> Option<Arc<CamoTable>> {
    TABLE.read().ok().and_then(|t| t.clone())
}

/// What `option` draws a gun's `material` with, if it changes it.
#[must_use]
pub fn swap(option: u32, material: u32) -> Option<MaterialCamo> {
    if option == 0 {
        return None;
    }
    TABLE.read().ok()?.as_ref()?.swaps.get(&(option, material)).copied()
}

/// Each player's camos as the server sent them: client number -> (weapon,
/// option) pairs.
static WORN: RwLock<Option<HashMap<u32, Vec<(u32, u32)>>>> = RwLock::new(None);
/// The camo on the gun in the local player's hands (the first-person draw).
static VIEWMODEL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// `bo2mp_camos`'s text: `<weapon>:<option>` pairs, comma separated.
#[must_use]
pub fn parse_worn(text: &str) -> Vec<(u32, u32)> {
    text.split(',')
        .filter_map(|pair| {
            let (w, o) = pair.split_once(':')?;
            Some((w.trim().parse().ok()?, o.trim().parse().ok()?))
        })
        .collect()
}

/// `parse_worn`'s text for these pairs.
#[must_use]
pub fn format_worn(pairs: &[(u32, u32)]) -> String {
    pairs.iter().map(|(w, o)| format!("{w}:{o}")).collect::<Vec<_>>().join(",")
}

/// This frame's camos of every player (client number, pairs).
pub fn set_worn(all: HashMap<u32, Vec<(u32, u32)>>) {
    if let Ok(mut w) = WORN.write() {
        *w = Some(all);
    }
}

/// The camo option player `client` has on `weapon` (0: none).
#[must_use]
pub fn worn(client: u32, weapon: u32) -> u32 {
    WORN.read()
        .ok()
        .and_then(|w| {
            w.as_ref()?
                .get(&client)?
                .iter()
                .find(|(x, _)| *x == weapon)
                .map(|(_, o)| *o)
        })
        .unwrap_or(0)
}

pub fn set_viewmodel(option: u32) {
    VIEWMODEL.store(option, std::sync::atomic::Ordering::Relaxed);
}

#[must_use]
pub fn viewmodel() -> u32 {
    VIEWMODEL.load(std::sync::atomic::Ordering::Relaxed)
}

/// Instance slot 22's value for a camo surface: 1000 + its tiling (the
/// model shader tiles the surface's texture coordinates by it).
#[must_use]
pub fn tile_slot(tile: f32) -> f32 {
    1000.0 + if tile > 0.0 { tile } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worn_text_round_trips() {
        let pairs = vec![(12, 15), (40, 3)];
        assert_eq!(parse_worn(&format_worn(&pairs)), pairs);
        assert!(parse_worn("").is_empty());
        assert_eq!(parse_worn("7:2,bad,9:x"), vec![(7, 2)]);
    }
}
