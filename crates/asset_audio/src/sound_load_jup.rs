//! bo2mp: Call of Duty: Modern Warfare III's (2023, "jup") sounds, read
//! from the owner's install at runtime: Resurgence's announcer, music,
//! interface and gameplay sounds on Rebirth Island.
//!
//! `fastfile_jup`'s sound reader takes the sndassets and alias links of the
//! Resurgence zones ([`RESURGENCE_ZONES`]) and finds each sound's stream in
//! the KAPI packages of the install's `zone` folder. Alias names are FNV-1a
//! hashes there, so every alias is kept under its hash, `#<16 hex digits>`
//! ([`jup_alias_key`] turns a name into it). A clip names its package and
//! row (`<package path>`, `<key hex>:<offset>:<size>:<decoded>:<compressed>:
//! <seek table>:<rate>:<channels>:<frames>`) and is decoded when played:
//! the package entry through the game's own Oodle DLL, the Opus packets by
//! `rusty-opus` (pure Rust).
//!
//! The install is `$BO2_JUP_INSTALL`, else the `jup` line of the game
//! folders file. Nothing of the game is copied.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use fastfile_jup::oodle::{self, Decompressor};
use fastfile_jup::sound::{FORMAT_OPUS, alias_hash, loaded_stream, packets};
use fastfile_jup::sound_bank::{Pcm, SoundBank, decode_opus};
use fastfile_jup::xsub::{XSubEntry, decode_entry};

use crate::sound_catalog::{CapturedAlias, CapturedSound, SoundCatalog};
use crate::{AssetNamespace, ZoneGame, ZoneOwner};

pub const JUP_INSTALL_ENV: &str = "BO2_JUP_INSTALL";

/// The zones Resurgence's sounds come from: the battle royale's own
/// (gameplay, interface, music), its English voice lines, Rebirth Island's
/// English ones, and the shared multiplayer one (the battle royale's
/// pickups, cash, armor hit markers, purchase and victory/defeat stings). (The map zone, `mp_jup_escape5`, holds the
/// island's emitters and ambience; 270 MB to inflate, read once with them.)
pub const RESURGENCE_ZONES: &[&str] = &[
    "ingame_br",
    "eng_ingame_br",
    "eng_mp_jup_escape5",
    "ingame_shared_mp",
];

/// Rebirth Island's map zone (its ambience emitters), read with Resurgence's sounds.
pub const REBIRTH_MAP_ZONE: &str = "mp_jup_escape5";

/// Language packs asked first.
const LANG: &str = "eng";

/// 0: none; 1: Resurgence's sounds; 2: and Rebirth Island's map zone.
static WANTED: AtomicU8 = AtomicU8::new(0);

/// Whether the next sound bank takes in Resurgence's sounds (set before a
/// match's bank is composed), and Rebirth Island's map zone with them.
pub fn want_jup_sounds(resurgence: bool, rebirth_map: bool) {
    WANTED.store(
        u8::from(resurgence) * (1 + u8::from(rebirth_map)),
        Ordering::Relaxed,
    );
}

pub(crate) fn jup_sounds_wanted() -> Option<bool> {
    match WANTED.load(Ordering::Relaxed) {
        0 => None,
        n => Some(n == 2),
    }
}

/// Whether a composed bank already holds Resurgence's sounds.
pub fn has_jup_sounds(catalog: &SoundCatalog) -> bool {
    catalog
        .sounds
        .iter()
        .any(|s| s.namespace == Some(AssetNamespace::Jup))
}

/// The Modern Warfare III install, when it is set and looks like one
/// (`zone/ingame_br.ff` and the Oodle DLL beside it).
pub fn jup_install() -> Option<PathBuf> {
    let set = std::env::var_os(JUP_INSTALL_ENV)
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| asset_transport::game_paths::get("jup"))?;
    (set.join("zone").join("ingame_br.ff").is_file() && oodle::dll_path(&set).is_file())
        .then_some(set)
}

/// The catalog key of an alias: `#<hash>` (a name is hashed; a `#<hash>`
/// or `0x<hash>` stays as it is).
pub fn jup_alias_key(alias: &str) -> String {
    let hex = alias.strip_prefix('#').or_else(|| alias.strip_prefix("0x"));
    match hex.and_then(|h| u64::from_str_radix(h, 16).ok()) {
        Some(h) => format!("#{h:016x}"),
        None => format!("#{:016x}", alias_hash(alias)),
    }
}

fn available() -> &'static Mutex<HashSet<u64>> {
    static SET: OnceLock<Mutex<HashSet<u64>>> = OnceLock::new();
    SET.get_or_init(Default::default)
}

/// Whether the install has this alias (a name, or `#<hash>`), with a clip
/// to play. The first question reads Resurgence's sounds (and Rebirth
/// Island's zone) when the install is set and they are not read yet, so the
/// match script knows its sounds before the match's sound bank is composed.
pub fn jup_has(alias: &str) -> bool {
    let key = jup_alias_key(alias);
    let Ok(h) = u64::from_str_radix(&key[1..], 16) else {
        return false;
    };
    static READ: OnceLock<()> = OnceLock::new();
    READ.get_or_init(|| {
        if available().lock().is_ok_and(|s| s.is_empty())
            && let Some(install) = jup_install()
            && let Err(e) = jup_resurgence_sounds(&install, true)
        {
            diag::warn!(Zone, "sound bank: MW3 (2023) {}: {e}", install.display());
        }
    });
    available().lock().is_ok_and(|s| s.contains(&h))
}

/// Catalogs read, by install and whether the map zone is in.
type Cache = Mutex<HashMap<(PathBuf, bool), Arc<SoundCatalog>>>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

fn oodle_for(install: &Path) -> Result<&'static dyn Decompressor, String> {
    static OODLE: OnceLock<Result<Box<dyn Decompressor>, String>> = OnceLock::new();
    OODLE
        .get_or_init(|| oodle::load(install).map_err(|e| e.to_string()))
        .as_ref()
        .map(|b| b.as_ref())
        .map_err(Clone::clone)
}

/// Resurgence's sounds of the install (read once per process; with the
/// Rebirth Island map zone when `with_map`).
pub fn jup_resurgence_sounds(install: &Path, with_map: bool) -> Result<Arc<SoundCatalog>, String> {
    let key = (install.to_path_buf(), with_map);
    if let Some(hit) = cache().lock().ok().and_then(|c| c.get(&key).cloned()) {
        return Ok(hit);
    }
    let mut zones: Vec<&str> = RESURGENCE_ZONES.to_vec();
    if with_map {
        zones.push(REBIRTH_MAP_ZONE);
    }
    let catalog = Arc::new(load_sound_catalog_jup(install, &zones)?);
    if let Ok(mut c) = cache().lock() {
        c.insert(key, catalog.clone());
    }
    Ok(catalog)
}

/// The sounds of some zones of the install, every alias under `#<hash>`.
pub fn load_sound_catalog_jup(install: &Path, zones: &[&str]) -> Result<SoundCatalog, String> {
    let oodle = oodle_for(install)?;
    let zone_dir = install.join("zone");
    let mut bank = SoundBank::new();
    for z in zones {
        let path = zone_dir.join(format!("{z}.ff"));
        match fastfile_jup::read_zone(&path, oodle) {
            Ok((_, bytes)) => {
                let (sounds, links) = bank.add_zone(z, &bytes);
                diag::info!(
                    Zone,
                    "sound bank: MW3 (2023) {z}: {sounds} sounds, {links} alias links"
                );
            }
            Err(e) => diag::warn!(Zone, "sound bank: MW3 (2023) {}: {e}", path.display()),
        }
    }
    let streams = bank
        .index_packages(&zone_dir, LANG)
        .map_err(|e| format!("{}: {e}", zone_dir.display()))?;
    let owner = ZoneOwner::from_zone_path(&zone_dir.join(format!(
        "{}.ff",
        zones.first().copied().unwrap_or("ingame_br")
    )));
    let mut catalog = SoundCatalog::default();
    catalog.set_capture_game(ZoneGame::T6);
    catalog.set_capture_zone(owner);
    let (mut playable, mut missing) = (0usize, 0usize);
    let mut have = Vec::new();
    let mut hashes: Vec<u64> = bank.aliases.keys().copied().collect();
    hashes.sort_unstable();
    for h in hashes {
        let mut rows = Vec::new();
        for &si in &bank.aliases[&h] {
            let s = &bank.sounds[si];
            let a = &s.asset;
            let Some((package, entry)) = bank.row_of(s) else {
                missing += 1;
                continue;
            };
            if a.format != FORMAT_OPUS {
                missing += 1;
                continue;
            }
            rows.push(CapturedAlias {
                alias_name: format!("#{h:016x}"),
                file_type: Some(2),
                file_exists: Some(1),
                streamed: Some((
                    package.to_string_lossy().into_owned(),
                    format!(
                        "{:016x}:{}:{}:{}:{}:{}:{}:{}:{}",
                        entry.key,
                        entry.offset,
                        entry.size,
                        entry.decompressed,
                        u8::from(entry.compressed),
                        a.table_len,
                        a.rate,
                        a.channels,
                        a.frames
                    ),
                )),
                vol_min: 1.0,
                vol_max: 1.0,
                pitch_min: 1.0,
                pitch_max: 1.0,
                dist_min: 0.0,
                dist_max: 100_000.0,
                probability: 1.0,
                ..CapturedAlias::default()
            });
            playable += 1;
        }
        if rows.is_empty() {
            continue;
        }
        have.push(h);
        catalog.ingest_sound(CapturedSound {
            name: format!("#{h:016x}"),
            aliases: rows,
            game: ZoneGame::T6,
            zone: owner,
            namespace: Some(AssetNamespace::Jup),
        });
    }
    catalog.finalize();
    if let Ok(mut s) = available().lock() {
        s.extend(have);
    }
    diag::info!(
        Zone,
        "sound bank: MW3 (2023) {}: {} aliases, {playable} clips, {missing} without a stream ({streams} streams found in the packages)",
        install.display(),
        catalog.sounds.len()
    );
    Ok(catalog)
}

/// A clip (see the module docs): its package entry read, decompressed and
/// decoded.
pub fn read_jup_clip(package: &str, clip: &str) -> Result<Pcm, String> {
    let bad = |why: &str| format!("clip `{clip}`: {why}");
    let mut it = clip.split(':');
    let key = it
        .next()
        .and_then(|k| u64::from_str_radix(k, 16).ok())
        .ok_or_else(|| bad("no key"))?;
    let f: Vec<u64> = it
        .map(str::parse::<u64>)
        .collect::<Result<_, _>>()
        .map_err(|e| bad(&e.to_string()))?;
    let [
        offset,
        size,
        decompressed,
        compressed,
        table,
        rate,
        channels,
        frames,
    ] = f[..]
    else {
        return Err(bad("9 fields wanted"));
    };
    let install = Path::new(package)
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| format!("{package}: not in an install's zone folder"))?;
    let oodle = oodle_for(install)?;
    let entry = XSubEntry {
        key,
        offset,
        size: u32::try_from(size).map_err(|_| bad("size"))?,
        decompressed: u32::try_from(decompressed).map_err(|_| bad("decoded size"))?,
        compressed: compressed != 0,
    };
    let mut file = std::fs::File::open(package).map_err(|e| format!("{package}: {e}"))?;
    let mut blob = vec![0u8; entry.size as usize];
    file.seek(SeekFrom::Start(offset))
        .and_then(|_| file.read_exact(&mut blob))
        .map_err(|e| format!("{package}: {e}"))?;
    let bytes = decode_entry(&entry, &blob, oodle).map_err(|e| format!("{package}: {e}"))?;
    let (rate, channels, frames) = (
        u32::try_from(rate).map_err(|_| bad("rate"))?,
        u8::try_from(channels).map_err(|_| bad("channels"))?,
        u32::try_from(frames).map_err(|_| bad("frames"))?,
    );
    // (A loaded sound's entry has a header before its seek table.)
    let stream = loaded_stream(&bytes, u32::try_from(table).map_err(|_| bad("table"))?);
    decode_opus(packets(stream, table as usize), rate, channels, frames).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_keys() {
        assert_eq!(jup_alias_key("#00000000000000ff"), "#00000000000000ff");
        assert_eq!(jup_alias_key("0xff"), "#00000000000000ff");
        assert_eq!(jup_alias_key("a"), "#af63dc4c8601ec8c");
        assert!(!jup_has("never_loaded_alias"));
    }

    /// `BO2_JUP_INSTALL=<install> cargo test -p asset_audio jup_sounds --
    /// --ignored --nocapture`: Resurgence's sounds read and a few decoded.
    #[test]
    #[ignore]
    fn jup_sounds() {
        let install = jup_install().expect("BO2_JUP_INSTALL");
        let c = jup_resurgence_sounds(&install, false).expect("sounds");
        println!("{} aliases", c.sounds.len());
        for s in c.sounds.iter().take(5) {
            let a = &s.aliases[0];
            let (dir, name) = a.streamed.as_ref().unwrap();
            let pcm = read_jup_clip(dir, name).map(|p| (p.rate, p.channels, p.seconds()));
            println!("{}: {pcm:?}", s.name);
        }
    }
}
