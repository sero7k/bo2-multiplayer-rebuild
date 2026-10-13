//! bo2mp: Call of Duty: Ghosts' weapon sounds for its guest guns in a
//! Black Ops II match.
//!
//! `fastfile_iw6`'s exact walk reads a Ghosts zone's sound alias lists with
//! every shared loaded sound and sound file resolved. The aliases go into
//! the catalog under the Ghosts namespace (`AssetNamespace::Iw6`, which
//! the Ghosts guns' weapon rows carry), so they never take the name of a
//! Black Ops II, Black Ops or MW2/MW3 alias. Ghosts' gun sounds are loaded
//! 16-bit PCM in the zone (`ERiffAudioFormat` 1); streamed ones are FLAC in
//! `zone/soundfile<N>.pak` and are named by pak path and range
//! (`<pak>`, `<offset>:<length>`), decoded when played.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use fastfile_iw6::sound::{LoadedRef, LoadedSound, SoundAlias, SoundData, StreamedSound};

use crate::sound_catalog::{CapturedAlias, CapturedSound, LoadedSoundPcm, SoundCatalog};
use crate::{AssetNamespace, ZoneGame, ZoneOwner};

/// The alias name prefixes kept (weapons' fire, reload, handling and
/// melee sounds, and what their notetracks play), with every alias they
/// chain to.
const KEEP_PREFIXES: &[&str] = &["weap_", "wpn_", "melee_", "gear_", "foly_", "mvmt_", "US_1mc_achieve_kem", "US_1mc_use_tnuke", "US_1mc_enemy_tnuke", "ui_mp_kem_timer", "kem_", "mp_killstreak_kem"];

/// Where a Ghosts install is, from the games root and game_paths.cfg.
pub fn ghosts_install(games_root: Option<&Path>) -> Option<PathBuf> {
    let extra: Vec<PathBuf> = asset_transport::game_paths::get("ghosts")
        .into_iter()
        .collect();
    fastfile_iw6::install::find_install(games_root.filter(|p| !p.as_os_str().is_empty()), &extra)
}

/// The Ghosts zones its guns' sounds come from (`common_mp` holds them;
/// `common_core_mp` some more), those that exist.
pub fn ghosts_sound_zones(install: &Path) -> Vec<PathBuf> {
    ["common_mp", "common_core_mp"]
        .into_iter()
        .map(|z| install.join("zone").join(format!("{z}.ff")))
        .filter(|p| p.is_file())
        .collect()
}

fn cache() -> &'static Mutex<HashMap<PathBuf, Arc<SoundCatalog>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<SoundCatalog>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The weapon sounds of one Ghosts zone (read once per process).
pub fn ghosts_weapon_sounds(path: &Path) -> Result<Arc<SoundCatalog>, String> {
    if let Some(hit) = cache().lock().ok().and_then(|c| c.get(path).cloned()) {
        return Ok(hit);
    }
    let catalog = Arc::new(load_sound_catalog_iw6(path)?);
    if let Ok(mut c) = cache().lock() {
        c.insert(path.to_path_buf(), catalog.clone());
    }
    Ok(catalog)
}

fn text(name: &fastfile_iw6::sound::Name) -> Option<String> {
    name.text().filter(|s| !s.is_empty()).map(str::to_owned)
}

/// A loaded sound's catalog name: its own, else one from where it is.
fn loaded_name(l: &LoadedSound) -> String {
    text(&l.name).unwrap_or_else(|| format!("iw6_loaded_{:x}", l.data_offset.unwrap_or(0)))
}

fn stream_ref(zone_dir: &Path, s: &StreamedSound) -> Option<(String, String)> {
    if s.file_index == 0 || s.length == 0 {
        return None;
    }
    let pak = if s.localized {
        // (Localized streams live in the language folder; English.)
        zone_dir
            .join("english")
            .join(format!("eng_soundfile{}.pak", s.file_index))
    } else {
        zone_dir.join(format!("soundfile{}.pak", s.file_index))
    };
    Some((
        pak.to_string_lossy().into_owned(),
        format!("{}:{}", s.offset, s.length),
    ))
}

/// One Ghosts zone's weapon sound aliases (see the module docs).
pub fn load_sound_catalog_iw6(path: &Path) -> Result<SoundCatalog, String> {
    let zone = fastfile_iw6::Zone::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (cap, stop) = fastfile_iw6::walk::walk_zone(&zone);
    if let Some(stop) = &stop {
        diag::warn!(
            Zone,
            "sound bank: Ghosts {} walked {} of {} assets (stopped at {:?}: {})",
            path.display(),
            stop.asset_index,
            zone.assets.assets.len(),
            stop.asset_type,
            stop.error
        );
    }
    let zone_dir = path.parent().unwrap_or(Path::new("."));
    let owner = ZoneOwner::from_zone_path(path);
    // The lists kept: the weapon prefixes, and everything they chain to.
    let by_name: HashMap<&str, usize> = cap
        .sounds
        .iter()
        .enumerate()
        .filter_map(|(i, l)| Some((l.name.text()?, i)))
        .collect();
    let mut keep: BTreeSet<usize> = BTreeSet::new();
    let mut todo: Vec<usize> = by_name
        .iter()
        .filter(|(n, _)| KEEP_PREFIXES.iter().any(|p| n.starts_with(p)))
        .map(|(_, &i)| i)
        .collect();
    while let Some(i) = todo.pop() {
        if !keep.insert(i) {
            continue;
        }
        for a in &cap.sounds[i].aliases {
            for next in [&a.secondary, &a.chain] {
                if let Some(&j) = next.text().and_then(|n| by_name.get(n)) {
                    todo.push(j);
                }
            }
        }
    }

    let mut catalog = SoundCatalog::default();
    catalog.set_capture_game(ZoneGame::T6);
    catalog.set_capture_zone(owner);
    let mut pcm_seen: HashMap<String, ()> = HashMap::new();
    let (mut pcm_bytes, mut lists) = (0usize, 0usize);
    for &i in &keep {
        let list = &cap.sounds[i];
        let Some(name) = text(&list.name) else {
            continue;
        };
        let mut aliases = Vec::with_capacity(list.aliases.len());
        for a in &list.aliases {
            let mut row = alias_row(&name, a);
            match a.sound.get().map(|f| &f.data) {
                Some(SoundData::Loaded(LoadedRef::Inline(l)))
                    if l.format == 1 && l.bits == 16 && l.data_len > 0 =>
                {
                    let lname = loaded_name(l);
                    row.file_type = Some(1);
                    row.file_exists = Some(1);
                    row.loaded_name = Some(lname.clone());
                    if pcm_seen.insert(lname.clone(), ()).is_none()
                        && let Some(at) = l.data_offset
                        && let Some(bytes) = zone.image.get(at..at + l.data_len as usize)
                    {
                        pcm_bytes += bytes.len();
                        let frames = l.data_len / (2 * u32::from(l.channels.max(1)));
                        let mut pcm = LoadedSoundPcm::pcm16(
                            lname,
                            l.sample_rate,
                            i32::from(l.channels.max(1)),
                            frames,
                            bytes.to_vec(),
                        );
                        pcm.namespace = Some(AssetNamespace::Iw6);
                        pcm.zone = owner;
                        catalog.ingest_loaded(pcm);
                    }
                }
                Some(SoundData::Streamed(s)) => {
                    row.file_type = Some(2);
                    row.streamed = stream_ref(zone_dir, s);
                    row.file_exists = Some(u8::from(row.streamed.is_some()));
                }
                Some(SoundData::Primed { stream, .. }) => {
                    row.file_type = Some(3);
                    row.streamed = stream_ref(zone_dir, stream);
                    row.file_exists = Some(u8::from(row.streamed.is_some()));
                }
                // (Silent: a null loaded sound, or no file.)
                _ => {
                    row.file_type = Some(1);
                    row.file_name = Some("null".to_owned());
                }
            }
            aliases.push(row);
        }
        lists += 1;
        catalog.ingest_sound(CapturedSound {
            name,
            aliases,
            game: ZoneGame::T6,
            zone: owner,
            namespace: Some(AssetNamespace::Iw6),
        });
    }
    catalog.finalize();
    diag::info!(
        Zone,
        "sound bank: Ghosts {}: {lists} weapon alias lists of {}, {} loaded clips ({:.1} MB PCM)",
        path.display(),
        cap.sounds.len(),
        catalog.loaded.len(),
        pcm_bytes as f64 / 1.0e6
    );
    Ok(catalog)
}

fn alias_row(list_name: &str, a: &SoundAlias) -> CapturedAlias {
    let unit = |v: f32, fallback: f32| {
        if v.is_finite() && v > 0.0 {
            v
        } else {
            fallback
        }
    };
    CapturedAlias {
        alias_name: list_name.to_owned(),
        subtitle: text(&a.subtitle),
        secondary: text(&a.secondary),
        chain: text(&a.chain),
        mixer_group: text(&a.mixer_group),
        sequence: a.sequence,
        vol_min: a.vol_min.max(0.0),
        vol_max: a.vol_max.max(0.0),
        pitch_min: unit(a.pitch_min, 1.0),
        pitch_max: unit(a.pitch_max, 1.0),
        dist_min: a.dist_min.max(0.0),
        dist_max: a.dist_max.max(0.0),
        // (Ghosts' flag word is not IW4's: left unread, so nothing is taken
        // for a loop or a channel it is not.)
        flags: None,
        probability: if a.probability > 0.0 {
            a.probability
        } else {
            1.0
        },
        start_delay: a.start_delay,
        ..CapturedAlias::default()
    }
}

/// Whether the Ghosts sound file reference is one `prepare` can play:
/// (`<pak path>`, `<offset>:<length>`).
pub fn ghosts_stream_range(dir: &str, name: &str) -> Option<(PathBuf, u64, u64)> {
    let (offset, length) = name.split_once(':')?;
    Some((
        PathBuf::from(dir),
        offset.parse().ok()?,
        length.parse().ok()?,
    ))
}

/// The bytes of a Ghosts streamed sound (a FLAC stream) from its pak.
pub fn read_ghosts_stream(dir: &str, name: &str) -> Result<Vec<u8>, String> {
    use std::io::{Read, Seek, SeekFrom};
    let (pak, offset, length) =
        ghosts_stream_range(dir, name).ok_or_else(|| format!("bad Ghosts stream `{name}`"))?;
    let mut f = std::fs::File::open(&pak).map_err(|e| format!("{}: {e}", pak.display()))?;
    f.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
    let mut out = vec![0u8; length as usize];
    f.read_exact(&mut out)
        .map_err(|e| format!("{}: {e}", pak.display()))?;
    Ok(out)
}
