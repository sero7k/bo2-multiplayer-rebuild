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
/// texture slot) and how many times it repeats across the gun. `full`: it
/// covers the whole material (another game's gun, whose colour map's alpha
/// is no camo mask).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternCamo {
    pub image: u32,
    pub scale: f32,
    pub full: bool,
}

impl PatternCamo {
    /// Instance slot 22's value for its surfaces: 3000 + its repeat laid
    /// under the gun's colour by the colour map's alpha, as Black Ops II's
    /// gun shaders do; 5000 + its repeat over the whole surface.
    #[must_use]
    pub fn slot(&self) -> f32 {
        let repeat = if self.scale > 0.0 { self.scale.min(900.0) } else { 1.0 };
        if self.full { 5000.0 + repeat } else { 3000.0 + repeat }
    }
}

/// The camo numbers of the other games' camos (Black Ops II's are its own
/// weapon options, below 100): Modern Warfare 2's slot `n` is
/// `MW2_FIRST + n`, Modern Warfare 3's `MW3_FIRST + n`, Black Ops' camo
/// `n` (counted from 1) `BO1_FIRST + n`. All fit a class's camo byte.
pub const MW2_FIRST: u32 = 100;
pub const MW3_FIRST: u32 = 120;
pub const BO1_FIRST: u32 = 140;

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

/// Each material camo's most used material (and its tiling): what it
/// draws on a gun that has no camo material set of its own for it.
fn representatives(swaps: &HashMap<(u32, u32), MaterialCamo>) -> HashMap<u32, MaterialCamo> {
    let mut counts: HashMap<(u32, u32, u32), usize> = HashMap::new();
    for (&(option, _), c) in swaps {
        *counts.entry((option, c.material, c.tile.to_bits())).or_default() += 1;
    }
    let mut best: HashMap<u32, ((u32, u32), usize)> = HashMap::new();
    for ((option, material, tile), n) in counts {
        let e = best.entry(option).or_insert(((material, tile), n));
        if n > e.1 || (n == e.1 && (material, tile) < e.0) {
            *e = ((material, tile), n);
        }
    }
    best.into_iter().map(|(o, ((material, tile), _))| (o, MaterialCamo { material, tile: f32::from_bits(tile) })).collect()
}

/// Each pattern camo's most used pattern, laid over a whole material.
fn pattern_representatives(patterns: &HashMap<(u32, u32), PatternCamo>) -> HashMap<u32, PatternCamo> {
    let mut counts: HashMap<(u32, u32, u32), usize> = HashMap::new();
    for (&(option, _), p) in patterns {
        *counts.entry((option, p.image, p.scale.to_bits())).or_default() += 1;
    }
    let mut best: HashMap<u32, ((u32, u32), usize)> = HashMap::new();
    for ((option, image, scale), n) in counts {
        let e = best.entry(option).or_insert(((image, scale), n));
        if n > e.1 || (n == e.1 && (image, scale) < e.0) {
            *e = ((image, scale), n);
        }
    }
    best.into_iter().map(|(o, ((image, scale), _))| (o, PatternCamo { image, scale: f32::from_bits(scale), full: true })).collect()
}

/// Every camo for a gun material with no camo of its own for it: material
/// camos (Black Ops II's, Modern Warfare 2's and 3's) as one material, the
/// pattern camos (Black Ops II's, Black Ops') as one pattern over it all.
#[derive(Default)]
struct Anywhere {
    materials: HashMap<u32, MaterialCamo>,
    patterns: HashMap<u32, PatternCamo>,
    /// Black Ops II's guns' camo-ready materials (the others' camos draw
    /// there on its guns).
    ready: std::collections::HashSet<u32>,
    /// Modern Warfare 2's and 3's camos, and the gun materials their own
    /// camo models change (where only their own swap draws).
    guest_options: std::collections::HashSet<u32>,
    guest_bases: std::collections::HashSet<u32>,
}

static ANYWHERE: RwLock<Option<Anywhere>> = RwLock::new(None);

fn rebuild_anywhere() {
    let mut a = Anywhere::default();
    if let Some(t) = table() {
        a.materials.extend(representatives(&t.swaps));
        a.patterns.extend(pattern_representatives(&t.patterns));
        a.ready.extend(t.swaps.keys().map(|k| k.1));
        a.ready.extend(t.patterns.keys().map(|k| k.1));
    }
    if let Some(g) = GUEST.read().ok().and_then(|g| g.clone()) {
        a.materials.extend(representatives(&g));
        a.guest_options.extend(g.keys().map(|k| k.0));
        a.guest_bases.extend(g.keys().map(|k| k.1));
    }
    if let Some(e) = EXTRA.read().ok().and_then(|e| e.clone()) {
        a.patterns.extend(e);
    }
    if let Ok(mut w) = ANYWHERE.write() {
        *w = Some(a);
    }
}

/// The other games' pattern camos (Black Ops' camo pictures), by camo
/// number, each over a whole material.
static EXTRA: RwLock<Option<HashMap<u32, PatternCamo>>> = RwLock::new(None);

pub fn install_extra_patterns(patterns: HashMap<u32, PatternCamo>) {
    if let Ok(mut e) = EXTRA.write() {
        *e = (!patterns.is_empty()).then_some(patterns);
    }
    rebuild_anywhere();
}

/// Whether another game's camo draws on this gun material: any game's gun
/// material in the wrap set, or one of Black Ops II's camo-ready ones.
fn camo_ready(material: u32) -> bool {
    wraps(material) || ANYWHERE.read().ok().is_some_and(|a| a.as_ref().is_some_and(|a| a.ready.contains(&material)))
}

static TABLE: RwLock<Option<Arc<CamoTable>>> = RwLock::new(None);

/// The match load's camos (None: no camos this match).
pub fn install(table: Option<CamoTable>) {
    if let Ok(mut t) = TABLE.write() {
        *t = table.map(Arc::new);
    }
    rebuild_anywhere();
}

/// bo2mp: the materials of every other game's guns (glass, sights and hands
/// left out): every game's camos draw over them whole.
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
    rebuild_anywhere();
}

/// bo2mp: Black Ops II's Pack-a-Punch camo on other games' guns in
/// Zombies (their upgrades: `bo2_profile::guest::ZmUpgrade`): the camo
/// option the server marks such a gun with, and the zombies camo's
/// material (the one Black Ops II's upgraded guns' camo set swaps in most).
static PAP: RwLock<Option<(u32, MaterialCamo)>> = RwLock::new(None);
/// The other games' guns' materials it draws over (every game's, Modern
/// Warfare 2's and 3's too; glass, sights and hands left out).
static PAP_WRAP: RwLock<Option<std::collections::HashSet<u32>>> = RwLock::new(None);

/// The match's Pack-a-Punch camo for other games' guns (None: off a
/// Zombies map, or the map's camo set gave none).
pub fn install_pap(pap: Option<(u32, MaterialCamo)>) {
    if let Ok(mut p) = PAP.write() {
        *p = pap;
    }
}

pub fn install_pap_wrap(materials: std::collections::HashSet<u32>) {
    if let Ok(mut w) = PAP_WRAP.write() {
        *w = (!materials.is_empty()).then_some(materials);
    }
}

/// The Pack-a-Punch camo's material for a guest gun's `material`, when
/// `option` is the Pack-a-Punch option.
fn pap_swap(option: u32, material: u32) -> Option<MaterialCamo> {
    let (o, camo) = (*PAP.read().ok()?)?;
    (o == option && PAP_WRAP.read().ok()?.as_ref()?.contains(&material)).then_some(camo)
}

/// What `option` draws a gun's `material` with, if it changes it.
#[must_use]
pub fn swap(option: u32, material: u32) -> Option<MaterialCamo> {
    if option == 0 {
        return None;
    }
    if let Some(camo) = pap_swap(option, material) {
        return Some(camo);
    }
    let own = TABLE.read().ok().and_then(|t| t.as_ref()?.swaps.get(&(option, material)).copied());
    own.or_else(|| GUEST.read().ok()?.as_ref()?.get(&(option, material)).copied()).or_else(|| {
        if own_pattern(option, material).is_some() || !camo_ready(material) {
            return None;
        }
        let a = ANYWHERE.read().ok()?;
        let a = a.as_ref()?;
        if a.guest_options.contains(&option) && a.guest_bases.contains(&material) {
            return None;
        }
        a.materials.get(&option).copied()
    })
}

fn own_pattern(option: u32, material: u32) -> Option<PatternCamo> {
    TABLE.read().ok()?.as_ref()?.patterns.get(&(option, material)).copied()
}

/// The pattern `option` lays on a gun's `material`, if it is camo-ready.
#[must_use]
pub fn pattern(option: u32, material: u32) -> Option<PatternCamo> {
    if option == 0 {
        return None;
    }
    own_pattern(option, material).or_else(|| {
        if !camo_ready(material) {
            return None;
        }
        ANYWHERE.read().ok()?.as_ref()?.patterns.get(&option).copied()
    })
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
