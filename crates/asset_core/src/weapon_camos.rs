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

/// A pattern camo on one of a gun's materials: the pattern picture (a
/// texture slot) and how many times it repeats across the gun.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternCamo {
    pub image: u32,
    pub scale: f32,
}

/// The match's camos: per material camo option, each gun material it
/// replaces (catalog index) and what it draws instead; per pattern camo
/// option, the pattern on each camo-ready gun material.
#[derive(Clone, Debug, Default)]
pub struct CamoTable {
    pub swaps: HashMap<(u32, u32), MaterialCamo>,
    pub patterns: HashMap<(u32, u32), PatternCamo>,
    /// Options that are material camos, and their names (`camo_gold`).
    pub options: HashMap<u32, String>,
}

impl CamoTable {
    /// Each material camo's most used material (and its tiling): what it
    /// draws on another game's gun, which has no camo material set of its
    /// own (see `install_wrap`).
    fn representatives(&self) -> HashMap<u32, MaterialCamo> {
        let mut counts: HashMap<(u32, u32, u32), usize> = HashMap::new();
        for (&(option, _), c) in &self.swaps {
            *counts.entry((option, c.material, c.tile.to_bits())).or_default() += 1;
        }
        let mut best: HashMap<u32, ((u32, u32), usize)> = HashMap::new();
        for ((option, material, tile), n) in counts {
            let e = best.entry(option).or_insert(((material, tile), n));
            if n > e.1 || (n == e.1 && (material, tile) < e.0) {
                *e = ((material, tile), n);
            }
        }
        best.into_iter()
            .map(|(o, ((material, tile), _))| (o, MaterialCamo { material, tile: f32::from_bits(tile) }))
            .collect()
    }
}

static TABLE: RwLock<Option<Arc<CamoTable>>> = RwLock::new(None);

/// The match load's camos (None: no camos this match).
pub fn install(table: Option<CamoTable>) {
    if let Ok(mut r) = REPRESENTATIVE.write() {
        *r = table.as_ref().map(CamoTable::representatives).filter(|m| !m.is_empty());
    }
    if let Ok(mut t) = TABLE.write() {
        *t = table.map(Arc::new);
    }
}

/// Each Black Ops II material camo's material for another game's gun.
static REPRESENTATIVE: RwLock<Option<HashMap<u32, MaterialCamo>>> = RwLock::new(None);

/// bo2mp: the materials of the other games' guns that have no camos of
/// their own (Black Ops, World at War, CoD4, Ghosts, Advanced Warfare):
/// Black Ops II's material camos (Gold, Diamond, Carbon Fiber and the DLC
/// ones) draw over them whole, as on its own guns.
static WRAP: RwLock<Option<std::collections::HashSet<u32>>> = RwLock::new(None);

pub fn install_wrap(materials: std::collections::HashSet<u32>) {
    if let Ok(mut w) = WRAP.write() {
        *w = (!materials.is_empty()).then_some(materials);
    }
}

/// Whether Black Ops II's material camos draw on this gun material.
#[must_use]
pub fn wraps(material: u32) -> bool {
    WRAP.read().ok().and_then(|w| w.as_ref().map(|w| w.contains(&material))).unwrap_or(false)
}

pub fn table() -> Option<Arc<CamoTable>> {
    TABLE.read().ok().and_then(|t| t.clone())
}

/// bo2mp: the other games' guns' camos (Modern Warfare 2's and 3's camo
/// models as material swaps), by (camo slot, gun material); kept apart from
/// Black Ops II's table, which each match load replaces.
static GUEST: RwLock<Option<HashMap<(u32, u32), MaterialCamo>>> = RwLock::new(None);

pub fn install_guest(swaps: HashMap<(u32, u32), MaterialCamo>) {
    if let Ok(mut g) = GUEST.write() {
        *g = (!swaps.is_empty()).then_some(swaps);
    }
}

/// What `option` draws a gun's `material` with, if it changes it.
#[must_use]
pub fn swap(option: u32, material: u32) -> Option<MaterialCamo> {
    if option == 0 {
        return None;
    }
    let own = TABLE.read().ok().and_then(|t| t.as_ref()?.swaps.get(&(option, material)).copied());
    own.or_else(|| GUEST.read().ok()?.as_ref()?.get(&(option, material)).copied())
        .or_else(|| {
            wraps(material).then(|| REPRESENTATIVE.read().ok()?.as_ref()?.get(&option).copied()).flatten()
        })
}

/// The pattern `option` lays on a gun's `material`, if it is camo-ready.
#[must_use]
pub fn pattern(option: u32, material: u32) -> Option<PatternCamo> {
    if option == 0 {
        return None;
    }
    TABLE.read().ok()?.as_ref()?.patterns.get(&(option, material)).copied()
}

/// Instance slot 22's value for a pattern camo surface: 3000 + the
/// pattern's repeat (the model shader lays the pattern, read from the
/// surface's specular map slot, under the gun's colour where its alpha is
/// low, as Black Ops II's gun shaders do).
#[must_use]
pub fn pattern_slot(scale: f32) -> f32 {
    3000.0 + if scale > 0.0 { scale.min(900.0) } else { 1.0 }
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
    fn a_material_camo_wraps_another_games_gun() {
        let mut t = CamoTable::default();
        t.swaps.insert((15, 1), MaterialCamo { material: 100, tile: 2.0 });
        t.swaps.insert((15, 2), MaterialCamo { material: 100, tile: 2.0 });
        t.swaps.insert((15, 3), MaterialCamo { material: 101, tile: 1.0 });
        install(Some(t));
        install_wrap([500].into_iter().collect());
        assert_eq!(swap(15, 500), Some(MaterialCamo { material: 100, tile: 2.0 }));
        assert_eq!(swap(15, 501), None);
        assert_eq!(swap(0, 500), None);
        assert_eq!(swap(15, 3), Some(MaterialCamo { material: 101, tile: 1.0 }));
        install(None);
        install_wrap(Default::default());
    }

    #[test]
    fn worn_text_round_trips() {
        let pairs = vec![(12, 15), (40, 3)];
        assert_eq!(parse_worn(&format_worn(&pairs)), pairs);
        assert!(parse_worn("").is_empty());
        assert_eq!(parse_worn("7:2,bad,9:x"), vec![(7, 2)]);
    }
}
