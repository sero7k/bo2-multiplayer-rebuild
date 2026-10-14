//! bo2mp: Call of Duty: Advanced Warfare's weapon sounds for its guest guns
//! in a Black Ops II match.
//!
//! `fastfile_s1`'s exact walk reads common_mp's sound alias lists. AW's
//! loaded sounds keep their samples in `soundfile<N>.pak` (the zone holds
//! only a buffer for them): each names its pak and range, a FLAC stream or
//! raw PCM whose 46-byte RIFF header sits just before it. Every alias is
//! named by pak path and range (`<pak>`, `<offset>:<length>`), decoded when
//! played (`read_aw_stream`), under the AW namespace.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use fastfile_s1::walk::sound::SoundAlias;

use crate::sound_catalog::{CapturedAlias, CapturedSound, SoundCatalog};
use crate::{AssetNamespace, ZoneGame, ZoneOwner};

/// The alias name prefixes kept (weapons' fire, reload, handling and
/// melee sounds, and what their notetracks play), with every alias they
/// chain to (and the aliases weapons and effects name, below).
const KEEP_PREFIXES: &[&str] = &["weap_", "wpn_", "melee_", "gear_", "foly_", "mvmt_", "pc_exo_punch", "npc_exo_punch"];

/// The RIFF header before a raw PCM clip's samples in a sound pak.
const WAVE_HEADER: u64 = 46;

/// Where Advanced Warfare is installed, from the games root and
/// game_paths.cfg.
pub fn aw_install(games_root: Option<&Path>) -> Option<PathBuf> {
    let extra: Vec<PathBuf> = asset_transport::game_paths::get("aw").into_iter().collect();
    fastfile_s1::install::find_install(games_root.filter(|p| !p.as_os_str().is_empty()), &extra)
}

fn cache() -> &'static Mutex<HashMap<PathBuf, Arc<SoundCatalog>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<SoundCatalog>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The weapon sounds of AW's common_mp (read once per process).
pub fn aw_weapon_sounds(path: &Path) -> Result<Arc<SoundCatalog>, String> {
    if let Some(hit) = cache().lock().ok().and_then(|c| c.get(path).cloned()) {
        return Ok(hit);
    }
    let catalog = Arc::new(load_sound_catalog_s1(path)?);
    if let Ok(mut c) = cache().lock() {
        c.insert(path.to_path_buf(), catalog.clone());
    }
    Ok(catalog)
}

fn pak_ref(zone_dir: &Path, stream: (u16, u64, u64), localized: bool) -> Option<(String, String)> {
    let (index, offset, length) = stream;
    if index == 0 || length == 0 {
        return None;
    }
    let pak = if localized {
        zone_dir.join("english").join(format!("eng_soundfile{index}.pak"))
    } else {
        zone_dir.join(format!("soundfile{index}.pak"))
    };
    Some((pak.to_string_lossy().into_owned(), format!("{offset}:{length}")))
}

/// One AW zone's weapon sound aliases (see the module docs).
pub fn load_sound_catalog_s1(path: &Path) -> Result<SoundCatalog, String> {
    let zone = fastfile_s1::Zone::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut w = fastfile_s1::walk::Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
    if let Err(stop) = w.walk() {
        diag::warn!(
            Zone,
            "sound bank: AW {} walked {} of {} assets (stopped at {:?}: {})",
            path.display(),
            stop.asset_index,
            zone.assets.assets.len(),
            stop.asset_type,
            stop.error
        );
    }
    let cap = w.cap;
    let zone_dir = path.parent().unwrap_or(Path::new("."));
    let owner = ZoneOwner::from_zone_path(path);
    let by_name: HashMap<&str, usize> = cap
        .sounds
        .iter()
        .enumerate()
        .map(|(i, l)| (l.name.as_str(), i))
        .collect();
    let mut keep: BTreeSet<usize> = BTreeSet::new();
    // bo2mp: and every alias a weapon names (its projectile's bounces and
    // explosion, whatever its prefix) or an effect plays (shell casings
    // landing, a flash's crack): a guest gun keeps its own sounds.
    let named: BTreeSet<&str> = cap
        .weapons
        .iter()
        .flat_map(|w| w.sounds.iter().map(|(_, n)| n.as_str()))
        .chain(
            cap.fx
                .iter()
                .flat_map(|f| f.elems.iter().flat_map(|e| e.sounds.iter().map(String::as_str))),
        )
        .collect();
    let mut todo: Vec<usize> = by_name
        .iter()
        .filter(|(n, _)| KEEP_PREFIXES.iter().any(|p| n.starts_with(p)) || named.contains(*n))
        .map(|(_, &i)| i)
        .collect();
    while let Some(i) = todo.pop() {
        if !keep.insert(i) {
            continue;
        }
        for a in &cap.sounds[i].aliases {
            for next in [&a.secondary, &a.chain] {
                if let Some(&j) = next.as_deref().and_then(|n| by_name.get(n)) {
                    todo.push(j);
                }
            }
        }
    }

    let mut catalog = SoundCatalog::default();
    catalog.set_capture_game(ZoneGame::T6);
    catalog.set_capture_zone(owner);
    let (mut lists, mut playable) = (0usize, 0usize);
    for &i in &keep {
        let list = &cap.sounds[i];
        if list.name.is_empty() {
            continue;
        }
        let mut aliases = Vec::with_capacity(list.aliases.len());
        for a in &list.aliases {
            let mut row = alias_row(&list.name, a);
            let file = a.file.and_then(|v| cap.sound_files.get(&v));
            // (A loaded sound's samples are in its pak too.)
            let stream = file.and_then(|f| {
                f.loaded
                    .and_then(|id| cap.loaded.get(id.index as usize))
                    .and_then(|l| l.stream)
                    .or(f.stream)
            });
            match stream.and_then(|s| pak_ref(zone_dir, s, false)) {
                Some(r) => {
                    row.file_type = Some(2);
                    row.file_exists = Some(1);
                    row.streamed = Some(r);
                    playable += 1;
                }
                None => {
                    row.file_type = Some(1);
                    row.file_name = Some("null".to_owned());
                }
            }
            aliases.push(row);
        }
        lists += 1;
        catalog.ingest_sound(CapturedSound {
            name: list.name.clone(),
            aliases,
            game: ZoneGame::T6,
            zone: owner,
            namespace: Some(AssetNamespace::S1),
        });
    }
    catalog.finalize();
    diag::info!(
        Zone,
        "sound bank: AW {}: {lists} weapon alias lists of {}, {playable} aliases with a clip",
        path.display(),
        cap.sounds.len(),
    );
    Ok(catalog)
}

fn alias_row(list_name: &str, a: &SoundAlias) -> CapturedAlias {
    let unit = |v: f32, fallback: f32| if v.is_finite() && v > 0.0 { v } else { fallback };
    CapturedAlias {
        alias_name: list_name.to_owned(),
        secondary: a.secondary.clone(),
        chain: a.chain.clone(),
        vol_min: a.vol_min.max(0.0),
        vol_max: a.vol_max.max(0.0),
        pitch_min: unit(a.pitch_min, 1.0),
        pitch_max: unit(a.pitch_max, 1.0),
        dist_min: a.dist_min.max(0.0),
        dist_max: a.dist_max.max(0.0),
        // (AW's flag word is not IW4's: left unread.)
        flags: None,
        probability: if a.probability > 0.0 { a.probability } else { 1.0 },
        ..CapturedAlias::default()
    }
}

/// The bytes of an AW sound (`<pak>`, `<offset>:<length>`): a FLAC stream
/// as it is, raw PCM with its RIFF header (just before it in the pak).
pub fn read_aw_stream(dir: &str, name: &str) -> Result<Vec<u8>, String> {
    use std::io::{Read, Seek, SeekFrom};
    let (pak, offset, length) =
        crate::ghosts_stream_range(dir, name).ok_or_else(|| format!("bad AW stream `{name}`"))?;
    let mut f = std::fs::File::open(&pak).map_err(|e| format!("{}: {e}", pak.display()))?;
    let mut read = |at: u64, len: u64| -> Result<Vec<u8>, String> {
        f.seek(SeekFrom::Start(at)).map_err(|e| e.to_string())?;
        let mut out = vec![0u8; len as usize];
        f.read_exact(&mut out).map_err(|e| format!("{}: {e}", pak.display()))?;
        Ok(out)
    };
    let bytes = read(offset, length)?;
    if bytes.starts_with(b"fLaC") || offset < WAVE_HEADER {
        return Ok(bytes);
    }
    let with_header = read(offset - WAVE_HEADER, length + WAVE_HEADER)?;
    if with_header.starts_with(b"RIFF") {
        Ok(with_header)
    } else {
        Err(format!("{}: no FLAC or RIFF at {offset}", pak.display()))
    }
}
