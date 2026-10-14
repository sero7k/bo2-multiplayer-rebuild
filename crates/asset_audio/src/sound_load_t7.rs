//! bo2mp: Call of Duty: Black Ops III's weapon sounds for its guest guns in
//! a Black Ops II match.
//!
//! `fastfile_t7`'s walk reads `mp_common`'s sound bank (every multiplayer
//! gun's aliases: `wpn_ar_standard_fire_npc` and the layers its secondary
//! aliases chain to, the reload foley its animations' notetracks name,
//! `fly_arak_mag_out`). Each alias names its sound by `assetId`, an entry
//! of the `.sabl`/`.sabs` banks in `zone/snd/all` (`fastfile_t7::sab`):
//! every one a FLAC stream, named here by bank path and range (`<bank>`,
//! `<offset>:<length>`, as Ghosts' streamed sounds are) and decoded when
//! played, under the Black Ops III namespace.
//!
//! Of the alias record (0xd8) only the volume (u16 fractions at 0xb4) and
//! the distances (u16 at 0xb8, 0xba) are read; the rest of its flags and
//! curves are left to Black Ops II's defaults.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use fastfile_t7::walk::capture::SoundAliasRec;

use crate::sound_catalog::{CapturedAlias, CapturedSound, SoundCatalog};
use crate::{AssetNamespace, ZoneGame, ZoneOwner};

/// The alias name prefixes kept (weapons' fire and handling, the foley
/// their notetracks play, projectiles and shell casings), with every alias
/// they chain to.
const KEEP_PREFIXES: &[&str] = &["wpn_", "fly_", "prj_", "fly_reload"];

/// Black Ops III's multiplayer zone with the guns' sounds, when it is
/// installed (`zone/mp_common.ff` beside core_common).
pub fn bo3_mp_common(games: &crate::GamesRoot) -> Option<PathBuf> {
    let zone = asset_transport::find_zone_file_version(games, "mp_common", asset_transport::T7_ZONE_VERSION).ok()?;
    Some(zone.path)
}

fn cache() -> &'static Mutex<HashMap<PathBuf, Arc<SoundCatalog>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<SoundCatalog>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The weapon sounds of Black Ops III's mp_common (read once per process).
pub fn bo3_weapon_sounds(path: &Path) -> Result<Arc<SoundCatalog>, String> {
    if let Some(hit) = cache().lock().ok().and_then(|c| c.get(path).cloned()) {
        return Ok(hit);
    }
    let catalog = Arc::new(load_sound_catalog_t7(path)?);
    if let Ok(mut c) = cache().lock() {
        c.insert(path.to_path_buf(), catalog.clone());
    }
    Ok(catalog)
}

/// The banks of a zone (`snd/all/<zone>.all.sabl`, `.sabs`) that open.
fn banks(zone_path: &Path) -> Vec<fastfile_t7::sab::SabFile> {
    let dir = zone_path.parent().unwrap_or(Path::new(".")).join("snd").join("all");
    let stem = zone_path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    ["sabl", "sabs"]
        .iter()
        .filter_map(|ext| match fastfile_t7::sab::SabFile::open(&dir.join(format!("{stem}.all.{ext}"))) {
            Ok(b) => Some(b),
            Err(e) => {
                diag::warn!(Zone, "sound bank: BO3 {e}");
                None
            }
        })
        .collect()
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    b.get(at..at + 2).map_or(0, |x| u16::from_le_bytes([x[0], x[1]]))
}

fn alias_row(a: &SoundAliasRec) -> CapturedAlias {
    let r = &a.record;
    let vol = |at: usize| f32::from(u16_at(r, at)) / 32767.0;
    let (vol_min, vol_max) = (vol(0xb4), vol(0xb6));
    let dist_min = f32::from(u16_at(r, 0xb8));
    let dist_max = f32::from(u16_at(r, 0xba).max(u16_at(r, 0xbc)));
    CapturedAlias {
        alias_name: a.name.clone(),
        secondary: Some(a.secondary.clone()).filter(|s| !s.is_empty()),
        vol_min: if vol_min > 0.0 { vol_min } else { 1.0 },
        vol_max: if vol_max > 0.0 { vol_max } else { 1.0 },
        pitch_min: 1.0,
        pitch_max: 1.0,
        dist_min,
        // (A gun heard across the map, as Black Ops II's are.)
        dist_max: if a.name.starts_with("wpn_") && a.name.ends_with("_npc") { dist_max.max(5000.0) } else { dist_max.max(dist_min + 1.0) },
        flags: None,
        probability: 1.0,
        ..CapturedAlias::default()
    }
}

/// One Black Ops III zone's weapon sound aliases (see the module docs).
pub fn load_sound_catalog_t7(path: &Path) -> Result<SoundCatalog, String> {
    let zone = fastfile_t7::Zone::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let w = fastfile_t7::walk::walk_with(&zone, fastfile_t7::walk::Options { keep_anims: true, ..Default::default() });
    if !matches!(w.stop, fastfile_t7::walk::Stop::Done { .. }) {
        diag::warn!(Zone, "sound bank: BO3 {} walked as far as {:?}", path.display(), w.stop);
    }
    let aliases = &w.capture.sound_aliases;
    let banks = banks(path);
    let owner = ZoneOwner::from_zone_path(path);
    // Lists by name (an alias list is its consecutive records of one name).
    let mut by_name: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, a) in aliases.iter().enumerate() {
        by_name.entry(a.name.as_str()).or_default().push(i);
    }
    let mut keep: BTreeSet<&str> = BTreeSet::new();
    let mut todo: Vec<&str> = by_name.keys().copied().filter(|n| KEEP_PREFIXES.iter().any(|p| n.starts_with(p))).collect();
    while let Some(n) = todo.pop() {
        if !keep.insert(n) {
            continue;
        }
        for &i in by_name.get(n).map(Vec::as_slice).unwrap_or(&[]) {
            let s = aliases[i].secondary.as_str();
            if !s.is_empty() && by_name.contains_key(s) {
                todo.push(s);
            }
        }
    }
    let mut catalog = SoundCatalog::default();
    catalog.set_capture_game(ZoneGame::T6);
    catalog.set_capture_zone(owner);
    let (mut lists, mut playable, mut missing) = (0usize, 0usize, 0usize);
    for name in &keep {
        let mut rows = Vec::new();
        for &i in &by_name[name] {
            let a = &aliases[i];
            let mut row = alias_row(a);
            let hit = banks.iter().find_map(|b| b.find(a.asset_id).map(|e| (b, *e)));
            match hit.filter(|(_, e)| e.format == 8 && e.size > 0) {
                Some((bank, e)) => {
                    row.file_type = Some(2);
                    row.file_exists = Some(1);
                    row.streamed = Some((bank.path.to_string_lossy().into_owned(), format!("{}:{}", e.offset, e.size)));
                    row.flags = Some(u32::from(e.looping));
                    playable += 1;
                }
                None => {
                    missing += 1;
                    row.file_type = Some(1);
                    row.file_name = Some("null".to_owned());
                }
            }
            rows.push(row);
        }
        lists += 1;
        catalog.ingest_sound(CapturedSound {
            name: (*name).to_owned(),
            aliases: rows,
            game: ZoneGame::T6,
            zone: owner,
            namespace: Some(AssetNamespace::T7),
        });
    }
    catalog.finalize();
    diag::info!(
        Zone,
        "sound bank: BO3 {}: {lists} weapon alias lists of {} aliases, {playable} with a clip, {missing} not in the banks",
        path.display(),
        aliases.len(),
    );
    Ok(catalog)
}

#[cfg(test)]
mod tests {
    /// `IW4L_GAMES=<root> cargo test -p asset_audio bo3_sounds -- --ignored
    /// --nocapture`: the BO3 guns' sounds read and one of each gun's fire
    /// aliases decoded.
    #[test]
    #[ignore]
    fn bo3_sounds() {
        let games = crate::games_root_from_env().expect("games root");
        let path = super::bo3_mp_common(&games).expect("Black Ops III's mp_common");
        let c = super::load_sound_catalog_t7(&path).expect("sounds");
        println!("{} lists", c.sounds.len());
        for name in ["wpn_ar_standard_fire_npc", "wpn_ar_standard_fire_plr", "fly_arak_mag_out", "wpn_ar_accurate_fire_plr"] {
            let Some(s) = c.sound_in(crate::AssetNamespace::T7, name) else {
                println!("{name}: not kept");
                continue;
            };
            for a in &s.aliases {
                let decoded = a.streamed.as_ref().map(|(dir, n)| {
                    crate::read_ghosts_stream(dir, n).and_then(|b| sab_t6::decode_flac(&b).map(|f| (f.pcm.rate, f.pcm.channels, f.pcm.frames())).map_err(|e| e.to_string()))
                });
                println!(
                    "{name}: secondary {:?} vol {}..{} dist {}..{} flags {:?} -> {decoded:?}",
                    a.secondary, a.vol_min, a.vol_max, a.dist_min, a.dist_max, a.flags
                );
            }
        }
    }
}
