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

/// Whether a composed bank already holds Resurgence's sounds (its drop
/// music: the MW3 guns' sounds are in the other banks too).
pub fn has_jup_sounds(catalog: &SoundCatalog) -> bool {
    catalog
        .index_in(
            AssetNamespace::Jup,
            &jup_alias_key("mus_mx_mp_br_ri_infil_1_intro"),
        )
        .is_some()
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
    // (One read at a time: a match's bank asking while the prefetch reads
    // waits for it and takes its catalog.)
    static LOAD: Mutex<()> = Mutex::new(());
    let _one = LOAD.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
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

/// bo2mp: read Resurgence's sounds (with Rebirth Island's map zone) on a
/// background thread ahead of the match (the WARZONE menu opened, the
/// mode chosen): the match's sound bank then takes them from the cache
/// instead of reading 270 MB of zones while the island loads. Once per
/// process; nothing when the install is not there.
pub fn prefetch_jup_resurgence_sounds() {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, Ordering::Relaxed) {
        return;
    }
    let Some(install) = jup_install() else {
        return;
    };
    let spawned = std::thread::Builder::new()
        .name("jup-sounds".to_owned())
        .spawn(move || {
            let t = std::time::Instant::now();
            match jup_resurgence_sounds(&install, true) {
                Ok(c) => diag::info!(
                    Zone,
                    "sound bank: Resurgence's MW3 (2023) sounds read ahead in {:.1} s ({} aliases)",
                    t.elapsed().as_secs_f32(),
                    c.sounds.len()
                ),
                Err(e) => diag::warn!(Zone, "sound bank: Resurgence's MW3 (2023) sounds not read ahead: {e}"),
            }
        });
    if spawned.is_err() {
        STARTED.store(false, Ordering::Relaxed);
    }
}

/// The zone Modern Warfare III's guns' sounds are read from: `ingame`,
/// the guns' fire layers (`wpn_pi_mike93_fire_plr_transient`, `_shot`,
/// `_mech`, `_lfe`, `_atmo` ...) and their foley (`wfoly_plr_pi_mike93_
/// reload_magout` ...).
pub const JUP_GUN_ZONE: &str = "ingame";

/// A Modern Warfare III gun's own sounds, in the slots of the Black Ops II
/// gun it is a copy of (see `asset_game::GuestCopy`): alias names (or
/// `#<hash>`), "" for none (the slot keeps its donor's sound).
#[derive(Clone, Copy, Debug)]
pub struct JupGunSounds {
    /// The gun (`jp12_pi_mike93`).
    pub gun: &'static str,
    /// Fired, heard in the world (another player's shot).
    pub fire: &'static str,
    /// Fired, in first person: the head of its layers.
    pub fire_player: &'static str,
    /// The last round, in first person (the slide locking back).
    pub fire_last_player: &'static str,
    pub raise_player: &'static str,
    pub first_raise_player: &'static str,
    pub putaway_player: &'static str,
    pub pickup: &'static str,
    pub pickup_player: &'static str,
    pub ads_up: &'static str,
    pub ads_down: &'static str,
    /// The donor's notetrack sounds by a word of their alias, played as
    /// this gun's (the first pair whose word the alias holds; the others
    /// keep the donor's).
    pub notes: &'static [(&'static str, &'static str)],
    /// Layers played together (alias, the one played with it) where the
    /// zone's chain stops short (its end is in no zone read): the shot's
    /// low end into the long tail.
    pub layers: &'static [(&'static str, &'static str)],
}

impl JupGunSounds {
    /// Every alias it names (the layers' chains are followed when read).
    pub fn aliases(&self) -> impl Iterator<Item = &'static str> {
        [
            self.fire,
            self.fire_player,
            self.fire_last_player,
            self.raise_player,
            self.first_raise_player,
            self.putaway_player,
            self.pickup,
            self.pickup_player,
            self.ads_up,
            self.ads_down,
        ]
        .into_iter()
        .chain(self.notes.iter().map(|(_, a)| *a))
        .chain(self.layers.iter().flat_map(|(a, b)| [*a, *b]))
        .filter(|a| !a.is_empty())
    }
}

/// The Modern Warfare III guns played and their sounds (in the install's
/// `ingame`; names recovered by hashing, see docs/JUP.md).
pub const JUP_GUN_SOUNDS: &[JupGunSounds] = &[JupGunSounds {
    gun: "jp12_pi_mike93",
    // (Mono, eight variants) -> fire_npc_mech (-> fire_npc_atmo, below).
    fire: "wpn_pi_mike93_fire_npc",
    // transient -> shot -> mech -> lfe (-> atmo, below).
    fire_player: "wpn_pi_mike93_fire_plr_transient",
    // last_mech -> transient -> shot -> mech -> lfe.
    fire_last_player: "wpn_pi_mike93_fire_plr_last_mech",
    raise_player: "wfoly_plr_pi_mike93_raise_quick_01",
    first_raise_player: "wfoly_plr_pi_mike93_raise_01",
    putaway_player: "wfoly_plr_pi_mike93_drop_quick_01",
    // (`weap_pickup_pistol_npc` is in `ingame_shared_mp`, not read.)
    pickup: "",
    pickup_player: "weap_pickup_pistol_plr",
    // (`wpn_pi_mike93_ads_up` and `_ads_down` name other aliases, not a
    // sound of their own: not followed yet, the donor's play.)
    ads_up: "",
    ads_down: "",
    notes: &[
        ("magout", "wfoly_plr_pi_mike93_reload_magout"),
        ("mag_out", "wfoly_plr_pi_mike93_reload_magout"),
        ("clipout", "wfoly_plr_pi_mike93_reload_magout"),
        ("clip_out", "wfoly_plr_pi_mike93_reload_magout"),
        ("magin", "wfoly_plr_pi_mike93_reload_magin"),
        ("mag_in", "wfoly_plr_pi_mike93_reload_magin"),
        ("clipin", "wfoly_plr_pi_mike93_reload_magin"),
        ("clip_in", "wfoly_plr_pi_mike93_reload_magin"),
        ("slide", "wfoly_plr_pi_mike93_reload_slide_release"),
        ("chamber", "wfoly_plr_pi_mike93_reload_slide_release"),
        ("release", "wfoly_plr_pi_mike93_reload_slide_release"),
        ("lift", "wfoly_plr_pi_mike93_reload_hand"),
        ("grab", "wfoly_plr_pi_mike93_reload_hand"),
        ("hand", "wfoly_plr_pi_mike93_reload_hand"),
        ("raise", "wfoly_plr_pi_mike93_raise_quick_01"),
        ("pullout", "wfoly_plr_pi_mike93_raise_quick_01"),
        ("drop", "wfoly_plr_pi_mike93_drop_quick_01"),
        ("putaway", "wfoly_plr_pi_mike93_drop_quick_01"),
    ],
    layers: &[
        ("wpn_pi_mike93_fire_plr_lfe", "wpn_pi_mike93_fire_plr_atmo"),
        ("wpn_pi_mike93_fire_npc_mech", "wpn_pi_mike93_fire_npc_atmo"),
    ],
}];

/// The gun sounds of the install (`JUP_GUN_ZONE`, only the aliases
/// [`JUP_GUN_SOUNDS`] name and the layers chained from them, each layer the
/// secondary of the one before), read once per process.
pub fn jup_gun_sounds(install: &Path) -> Result<Arc<SoundCatalog>, String> {
    type GunCache = Mutex<HashMap<PathBuf, Arc<SoundCatalog>>>;
    static CACHE: OnceLock<GunCache> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(hit) = cache.lock().ok().and_then(|c| c.get(install).cloned()) {
        return Ok(hit);
    }
    let hash = |a: &str| u64::from_str_radix(&jup_alias_key(a)[1..], 16).ok();
    let keep: HashSet<u64> = JUP_GUN_SOUNDS
        .iter()
        .flat_map(JupGunSounds::aliases)
        .filter_map(hash)
        .collect();
    let extra: HashMap<u64, u64> = JUP_GUN_SOUNDS
        .iter()
        .flat_map(|g| g.layers.iter())
        .filter_map(|(a, b)| Some((hash(a)?, hash(b)?)))
        .collect();
    let catalog = Arc::new(load_jup(install, &[JUP_GUN_ZONE], Some((keep, extra)))?);
    if let Ok(mut c) = cache.lock() {
        c.insert(install.to_path_buf(), catalog.clone());
    }
    Ok(catalog)
}

/// The sounds of some zones of the install, every alias under `#<hash>`.
pub fn load_sound_catalog_jup(install: &Path, zones: &[&str]) -> Result<SoundCatalog, String> {
    load_jup(install, zones, None)
}

/// Aliases kept, and layers added to the zone's chains.
type Keep = (HashSet<u64>, HashMap<u64, u64>);

/// One alias's clips as the catalog keeps them: its hash, its next layer
/// (a secondary alias), and per clip its package (index into
/// [`JupRows::packages`]) and row text (see the module docs).
struct AliasRows {
    hash: u64,
    secondary: Option<u64>,
    clips: Vec<(u32, String)>,
}

/// What some zones' sounds come to: every alias with the clips the install
/// has, and how many had none.
struct JupRows {
    packages: Vec<String>,
    aliases: Vec<AliasRows>,
    missing: usize,
    streams: usize,
}

/// The sounds of some zones; with `keep`, only those aliases and the
/// layers chained from them (each the secondary of the one before).
///
/// Reading the zones (Resurgence's are 400 MB inflated) and the packages'
/// indexes takes many seconds, and comes to the same rows each time for
/// the same install: the rows are kept in the game's cache
/// (`iw4l-artifacts/cache/jup_sound`), found by the zones' and packages'
/// sizes and times and this code's own text, and read from there after.
fn load_jup(install: &Path, zones: &[&str], keep: Option<Keep>) -> Result<SoundCatalog, String> {
    let zone_dir = install.join("zone");
    let started = std::time::Instant::now();
    let key = rows_key(&zone_dir, zones, keep.is_some());
    let cached = key.as_deref().and_then(|k| {
        let bytes = asset_transport::cache_get(ROWS_KIND, k)?;
        let rows = decode_rows(&bytes);
        if rows.is_none() {
            diag::warn!(Zone, "sound bank: MW3 (2023) cache {k} unreadable; reading the zones");
        }
        rows
    });
    let from_cache = cached.is_some();
    let rows = match cached {
        Some(rows) => rows,
        None => {
            let rows = read_rows(install, zones, keep.as_ref())?;
            if let Some(k) = &key
                && let Err(e) = asset_transport::cache_put(ROWS_KIND, k, &encode_rows(&rows))
            {
                diag::warn!(Zone, "sound bank: MW3 (2023) cache {k} not written: {e}");
            }
            rows
        }
    };
    let owner = ZoneOwner::from_zone_path(&zone_dir.join(format!(
        "{}.ff",
        zones.first().copied().unwrap_or("ingame_br")
    )));
    let mut catalog = SoundCatalog::default();
    catalog.set_capture_game(ZoneGame::T6);
    catalog.set_capture_zone(owner);
    let mut playable = 0usize;
    let mut have = Vec::with_capacity(rows.aliases.len());
    for a in &rows.aliases {
        let name = format!("#{:016x}", a.hash);
        let secondary = a.secondary.map(|n| format!("#{n:016x}"));
        let clips: Vec<CapturedAlias> = a
            .clips
            .iter()
            .filter_map(|(package, clip)| {
                Some(CapturedAlias {
                    alias_name: name.clone(),
                    secondary: secondary.clone(),
                    file_type: Some(2),
                    file_exists: Some(1),
                    streamed: Some((rows.packages.get(*package as usize)?.clone(), clip.clone())),
                    vol_min: 1.0,
                    vol_max: 1.0,
                    pitch_min: 1.0,
                    pitch_max: 1.0,
                    dist_min: 0.0,
                    dist_max: 100_000.0,
                    probability: 1.0,
                    ..CapturedAlias::default()
                })
            })
            .collect();
        if clips.is_empty() {
            continue;
        }
        playable += clips.len();
        have.push(a.hash);
        catalog.ingest_sound(CapturedSound {
            name,
            aliases: clips,
            game: ZoneGame::T6,
            zone: owner,
            namespace: Some(AssetNamespace::Jup),
        });
    }
    catalog.finalize();
    // (What Resurgence's script may play: the guns' sounds are not asked
    // for, and `jup_has` reads Resurgence's zones while this is empty.)
    if keep.is_none()
        && let Ok(mut s) = available().lock()
    {
        s.extend(have);
    }
    diag::info!(
        Zone,
        "sound bank: MW3 (2023) {}: {} aliases, {playable} clips, {} without a stream ({} streams found in the packages){} in {:.1}s",
        install.display(),
        catalog.sounds.len(),
        rows.missing,
        rows.streams,
        if from_cache { ", from the cache," } else { "" },
        started.elapsed().as_secs_f32()
    );
    Ok(catalog)
}

/// The zones' aliases and their clips' rows, read from the zones and the
/// packages' indexes.
fn read_rows(install: &Path, zones: &[&str], keep: Option<&Keep>) -> Result<JupRows, String> {
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
    let mut missing = 0usize;
    // A layer's next: the zone's chain, else one added.
    let next_of = |h: u64| -> Option<u64> {
        let (_, extra) = keep?;
        bank.next
            .get(&h)
            .copied()
            .filter(|n| bank.aliases.contains_key(n))
            .or_else(|| extra.get(&h).copied())
    };
    let mut hashes: Vec<u64> = match keep {
        None => bank.aliases.keys().copied().collect(),
        Some((wanted, _)) => {
            // The wanted aliases and every layer chained from them.
            let mut set = HashSet::new();
            for &w in wanted {
                let mut at = Some(w);
                while let Some(h) = at.filter(|h| set.insert(*h)) {
                    at = next_of(h);
                }
            }
            set.into_iter()
                .filter(|h| bank.aliases.contains_key(h))
                .collect()
        }
    };
    hashes.sort_unstable();
    let mut packages: Vec<String> = Vec::new();
    let mut package_index: HashMap<String, u32> = HashMap::new();
    let mut aliases = Vec::with_capacity(hashes.len());
    for h in hashes {
        let mut clips = Vec::new();
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
            let package = package.to_string_lossy().into_owned();
            let pi = *package_index.entry(package.clone()).or_insert_with(|| {
                packages.push(package);
                (packages.len() - 1) as u32
            });
            clips.push((
                pi,
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
            ));
        }
        aliases.push(AliasRows { hash: h, secondary: next_of(h), clips });
    }
    Ok(JupRows { packages, aliases, missing, streams })
}

/// The cache kind of the rows.
const ROWS_KIND: &str = "jup_sound";
/// The rows' layout's number.
const ROWS_FORMAT: u32 = 1;

/// The cache key of some zones' rows: over the zones' and the packages'
/// names, sizes and times, whether only the guns' are kept, and the code
/// that reads them. `None` when the folder cannot be listed.
fn rows_key(zone_dir: &Path, zones: &[&str], kept: bool) -> Option<String> {
    const CODE: &[&str] = &[
        include_str!("sound_load_jup.rs"),
        include_str!("../../fastfile_jup/src/sound_bank.rs"),
        include_str!("../../fastfile_jup/src/sound.rs"),
        include_str!("../../fastfile_jup/src/xsub.rs"),
    ];
    let fnv = |mut h: u64, b: &[u8]| {
        for &x in b {
            h ^= u64::from(x);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    };
    let stamp = |meta: &std::fs::Metadata| {
        let written = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs());
        (meta.len(), written)
    };
    let mut h = fnv(0xcbf2_9ce4_8422_2325, &ROWS_FORMAT.to_le_bytes());
    h = fnv(h, &[u8::from(kept)]);
    for code in CODE {
        h = fnv(h, code.as_bytes());
    }
    for z in zones {
        let (len, written) = std::fs::metadata(zone_dir.join(format!("{z}.ff"))).map_or((0, 0), |m| stamp(&m));
        h = fnv(h, z.as_bytes());
        h = fnv(h, &len.to_le_bytes());
        h = fnv(h, &written.to_le_bytes());
    }
    let mut subs: Vec<(String, u64, u64)> = std::fs::read_dir(zone_dir)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            if !name.ends_with(".xsub") {
                return None;
            }
            let (len, written) = stamp(&e.metadata().ok()?);
            Some((name, len, written))
        })
        .collect();
    subs.sort();
    for (name, len, written) in &subs {
        h = fnv(h, name.as_bytes());
        h = fnv(h, &len.to_le_bytes());
        h = fnv(h, &written.to_le_bytes());
    }
    Some(format!("{}-{h:016x}", if kept { "guns" } else { "zones" }))
}

fn encode_rows(rows: &JupRows) -> Vec<u8> {
    let mut out = Vec::new();
    let u64s = |out: &mut Vec<u8>, v: u64| out.extend_from_slice(&v.to_le_bytes());
    let text = |out: &mut Vec<u8>, s: &str| {
        out.extend_from_slice(&(s.len() as u64).to_le_bytes());
        out.extend_from_slice(s.as_bytes());
    };
    out.extend_from_slice(&ROWS_FORMAT.to_le_bytes());
    u64s(&mut out, rows.packages.len() as u64);
    for p in &rows.packages {
        text(&mut out, p);
    }
    u64s(&mut out, rows.aliases.len() as u64);
    for a in &rows.aliases {
        u64s(&mut out, a.hash);
        u64s(&mut out, a.secondary.unwrap_or(u64::MAX));
        u64s(&mut out, a.clips.len() as u64);
        for (p, clip) in &a.clips {
            out.extend_from_slice(&p.to_le_bytes());
            text(&mut out, clip);
        }
    }
    u64s(&mut out, rows.missing as u64);
    u64s(&mut out, rows.streams as u64);
    out
}

/// A cursor over the rows' bytes.
struct Bytes<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Bytes<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(s)
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    /// A count (never more than the bytes left could hold).
    fn count(&mut self) -> Option<usize> {
        usize::try_from(self.u64()?).ok().filter(|&n| n <= self.b.len() - self.at)
    }
    fn text(&mut self) -> Option<String> {
        let n = self.count()?;
        String::from_utf8(self.take(n)?.to_vec()).ok()
    }
}

fn decode_rows(b: &[u8]) -> Option<JupRows> {
    let mut r = Bytes { b, at: 0 };
    if r.u32()? != ROWS_FORMAT {
        return None;
    }
    let n = r.count()?;
    let packages = (0..n).map(|_| r.text()).collect::<Option<Vec<_>>>()?;
    let n = r.count()?;
    let mut aliases = Vec::with_capacity(n);
    for _ in 0..n {
        let hash = r.u64()?;
        let secondary = Some(r.u64()?).filter(|&s| s != u64::MAX);
        let m = r.count()?;
        let mut clips = Vec::with_capacity(m);
        for _ in 0..m {
            let p = r.u32()?;
            if p as usize >= packages.len() {
                return None;
            }
            clips.push((p, r.text()?));
        }
        aliases.push(AliasRows { hash, secondary, clips });
    }
    let missing = usize::try_from(r.u64()?).ok()?;
    let streams = usize::try_from(r.u64()?).ok()?;
    (r.at == b.len()).then_some(JupRows { packages, aliases, missing, streams })
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
    fn rows_read_back_as_written() {
        let rows = JupRows {
            packages: vec!["a.xsub".into(), "b.xsub".into()],
            aliases: vec![
                AliasRows { hash: 1, secondary: Some(2), clips: vec![(0, "00:1:2:3:1:4:48000:2:960".into()), (1, "x".into())] },
                AliasRows { hash: 2, secondary: None, clips: Vec::new() },
            ],
            missing: 3,
            streams: 4,
        };
        let bytes = encode_rows(&rows);
        let back = decode_rows(&bytes).unwrap();
        assert_eq!(encode_rows(&back), bytes);
        assert!(decode_rows(&bytes[..bytes.len() - 1]).is_none());
    }

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
