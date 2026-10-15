use std::collections::{BTreeMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

const IW4_ZONE_VERSION: u32 = 0x114;

/// bo2mp: the first four bytes of an IW4x-built zone (ZoneBuilder's
/// `XFILE_HEADER_IW4X`, `'IW4x'`), in place of `IWff`; the next four hold
/// IW4x's own header version, then the stock version (0x114) follows.
pub const IW4X_MAGIC: &[u8; 4] = b"IW4x";

/// bo2mp: zone versions IW4x accepts besides 0x114 (CoD Online / alpha
/// builds, remapped by IW4x's `Zones.cpp` per-asset fixups), plus 0x10D seen
/// in some usermap packs. Their layouts differ from stock MW2, so a usermap
/// in one of them is listed (so he sees it) but fails to load with a clear
/// gap message instead of being misread.
pub fn is_iw4_variant_version(version: u32) -> bool {
    version == 0x10D || (316..=360).contains(&version) || (423..=461).contains(&version)
}

/// bo2mp: whether a zone file starts with IW4x's own header.
pub fn is_iw4x_envelope(path: &Path) -> bool {
    use std::io::Read;
    let mut magic = [0u8; 4];
    std::fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut magic))
        .is_ok()
        && &magic == IW4X_MAGIC
}

/// bo2mp: a custom map in a game's `usermaps` folder (IW4x layout:
/// `usermaps/<map>/<map>.ff`, often `<map>_load.ff`, and `<map>.iwd` with
/// its images and sounds beside them).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Usermap {
    /// The zone prefix it loads under (`iw4`).
    pub game: &'static str,
    /// Its zone name (`mp_waw_castle`).
    pub stem: String,
    pub zone: PathBuf,
    pub load_zone: Option<PathBuf>,
    /// The archives beside it (searched after the game's own).
    pub iwds: Vec<PathBuf>,
    /// The version its envelope carries (`None` when unreadable).
    pub version: Option<u32>,
    /// Built by IW4x's ZoneBuilder with its obfuscated body.
    pub iw4x_envelope: bool,
}

impl Usermap {
    /// Why it cannot load, when its zone is not one the readers take.
    pub fn gap(&self) -> Option<String> {
        usermap_gap(&self.zone, self.version, self.iw4x_envelope)
    }
}

fn usermap_gap(zone: &Path, version: Option<u32>, iw4x: bool) -> Option<String> {
    let kind = if iw4x { "IW4x envelope, " } else { "" };
    match version {
        Some(IW4_ZONE_VERSION) => None,
        Some(v) if is_iw4_variant_version(v) => Some(format!(
            "usermap zone {} is version {v:#x} ({kind}an IW4x / CoD Online variant): \
             its asset layouts differ from stock MW2 0x114 and this reader does not convert \
             them yet (gap: IW4x Zones.cpp per-version fixups). Rebuild the map with a \
             0x114 ZoneBuilder to play it",
            zone.display()
        )),
        Some(v) => Some(format!(
            "usermap zone {} has version {v:#x} ({kind}want MW2 0x114)",
            zone.display()
        )),
        None => Some(format!(
            "usermap zone {} is not a readable IWff/IW4x envelope",
            zone.display()
        )),
    }
}

/// bo2mp: (map, is its `_load` zone) when `path` is a usermap zone:
/// `…/usermaps/<map>/<map>.ff` or `…/usermaps/<map>/<map>_load.ff`.
pub fn usermap_zone_of(path: &Path) -> Option<(String, bool)> {
    let is_ff = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("ff"));
    if !is_ff {
        return None;
    }
    let stem = path.file_stem()?.to_str()?.to_ascii_lowercase();
    let folder = path.parent()?;
    let map = folder.file_name()?.to_str()?.to_ascii_lowercase();
    let usermaps = folder.parent()?.file_name()?.to_str()?;
    if !usermaps.eq_ignore_ascii_case("usermaps") || map.is_empty() {
        return None;
    }
    if stem == map {
        Some((map, false))
    } else if stem.strip_suffix("_load") == Some(map.as_str()) {
        Some((map, true))
    } else {
        None
    }
}

/// bo2mp: the zone prefix a usermap loads under: its envelope's game, or
/// MW2 for an IW4x / CoD Online variant (listed, then refused clearly).
fn usermap_game(path: &Path) -> Option<&'static str> {
    if let Some(game) = zone_game_for_path(path) {
        return Some(game.prefix());
    }
    match peek_zone_version(path) {
        Some(v) if is_iw4_variant_version(v) || is_iw4x_envelope(path) => Some("iw4"),
        _ => None,
    }
}

/// bo2mp: every usermap in the game folders, by (game, map); a folder
/// whose zone is missing or unreadable is logged and left out.
pub fn list_usermaps(root: &GamesRoot) -> Vec<Usermap> {
    usermaps_in(game_files(&root.0))
}

fn usermaps_in(files: impl Iterator<Item = Result<PathBuf, String>>) -> Vec<Usermap> {
    let mut zones: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut loads: BTreeMap<String, PathBuf> = BTreeMap::new();
    for path in files.flatten() {
        let Some((map, is_load)) = usermap_zone_of(&path) else {
            continue;
        };
        let slot = if is_load { &mut loads } else { &mut zones };
        slot.entry(map).or_insert(path);
    }
    let mut out = Vec::new();
    for (stem, zone) in zones {
        let Some(game) = usermap_game(&zone) else {
            diag::warn!(
                Zone,
                "usermap {stem}: {} is not a zone this rebuild reads (version {}); skipped",
                zone.display(),
                peek_zone_version(&zone).map_or_else(|| "unreadable".to_owned(), |v| format!("{v:#x}"))
            );
            continue;
        };
        let folder = zone.parent().map(Path::to_path_buf).unwrap_or_default();
        let mut iwds: Vec<PathBuf> = std::fs::read_dir(&folder)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_file() && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("iwd")))
                    .collect()
            })
            .unwrap_or_default();
        iwds.sort();
        out.push(Usermap {
            game,
            version: peek_zone_version(&zone),
            iw4x_envelope: is_iw4x_envelope(&zone),
            load_zone: loads.get(&stem).cloned(),
            stem,
            zone,
            iwds,
        });
    }
    out
}

/// bo2mp: the archives of every usermap of the game tree whose `main` is
/// `main` (`<main>/../usermaps/*/*.iwd`), searched after `main`'s own.
/// bo2mp: IW4x's own archives for a MW2 `main` folder - its DLC map packs
/// (`iw_dlc*_00.iwd`: Shipment, Dome, the CoD4 and MW3 packs) and
/// `iw4x_*.iwd` - in any of the places IW4x keeps them: `main/iw4x/x86`,
/// `main/iw4x`, `<game>/iw4x`. Sorted by name; none when the folders are
/// absent.
pub fn iw4x_iwds_for_main(main: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![main.join("iw4x").join("x86"), main.join("iw4x")];
    if let Some(tree) = main.parent() {
        dirs.push(tree.join("iw4x"));
    }
    let mut out = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut found: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("iwd")))
            .collect();
        found.sort();
        out.extend(found);
    }
    out
}

pub fn usermap_iwds_for_main(main: &Path) -> Vec<PathBuf> {
    if !main
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("main"))
    {
        return Vec::new();
    }
    let Some(tree) = main.parent() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(tree) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for usermaps in entries.flatten().map(|e| e.path()) {
        if !usermaps.is_dir()
            || !usermaps
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("usermaps"))
        {
            continue;
        }
        let Ok(maps) = std::fs::read_dir(&usermaps) else {
            continue;
        };
        let mut folders: Vec<PathBuf> = maps.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        folders.sort();
        for folder in folders {
            let Ok(files) = std::fs::read_dir(&folder) else {
                continue;
            };
            let mut iwds: Vec<PathBuf> = files
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("iwd")))
                .collect();
            iwds.sort();
            out.extend(iwds);
        }
    }
    out
}

/// bo2mp: Call of Duty 4's fast file version (`IWffu100`, 5).
pub const IW3_ZONE_VERSION: u32 = 5;
/// bo2mp: World at War's fast file version (`IWffu100`, 0x183).
pub const T4_ZONE_VERSION: u32 = 0x183;
/// bo2mp: Call of Duty: Ghosts' fast file version (`IWffu100`, 0x235): its
/// maps load through `fastfile_iw6` (`assets::lane::iw6_map`).
pub const IW6_ZONE_VERSION: u32 = 0x235;
/// bo2mp: Call of Duty: Infinite Warfare's fast file version (`IWff0100`,
/// 0x653): its maps load through `fastfile_iw7` (`assets::lane::iw7`).
pub const IW7_ZONE_VERSION: u32 = 0x653;
/// bo2mp: Call of Duty: Black Ops III's fast file version (`TAff0000`,
/// 0x251): its maps load through `fastfile_t7` (`assets::lane::t7_map`).
pub const T7_ZONE_VERSION: u32 = 0x251;
/// bo2mp: Black Ops III's envelope magic.
const T7_MAGIC: &[u8; 8] = b"TAff0000";
/// bo2mp: Call of Duty: Modern Warfare III's (2023) fast file version
/// (`IWffa100`, 0x18): its maps load through `fastfile_jup`
/// (`assets::lane::jup_map`).
pub const JUP_ZONE_VERSION: u32 = 0x18;

/// bo2mp: the Modern Warfare III (2023) maps this rebuild plays, by zone
/// name: Rebirth Island (`mp_jup_escape5`, Resurgence's map) and Tokyo
/// (`mp_jup_elevated`, Season 4: its zone is the one dressed as Japan —
/// `jup_appliance_vending_machine_food_japan_01`, `jup_sign_arcade_japan_*`,
/// `jup_sign_convenience_japan_*`, pachinko and ramen props; no other map
/// zone has more than one such name). The install's other maps (`mp_jup_*`)
/// are left out on purpose.
pub const JUP_MAPS: [&str; 2] = ["mp_jup_escape5", "mp_jup_elevated"];

/// bo2mp: a Modern Warfare III zone is a map this rebuild plays.
pub fn is_jup_map(stem: &str) -> bool {
    JUP_MAPS.contains(&stem)
}

/// bo2mp: zone names no map has (Modern Warfare III's gun and character
/// parts, streamed world tiles: thousands of `mp_*` zones in its folder),
/// left out before their headers are read.
fn is_part_zone(stem: &str) -> bool {
    stem.ends_with("_tr")
        || ["mp_av_", "mp_aw_", "mp_vm_", "mp_wm_", "mp_hd_", "mp_bd_", "mp_vh_"].iter().any(|p| stem.starts_with(p))
}

/// bo2mp: the version of a legacy game's zones by its prefix.
pub fn legacy_version(prefix: &str) -> Option<u32> {
    match prefix {
        "iw3" => Some(IW3_ZONE_VERSION),
        "t4" => Some(T4_ZONE_VERSION),
        "iw6" => Some(IW6_ZONE_VERSION),
        "iw7" => Some(IW7_ZONE_VERSION),
        "t7" => Some(T7_ZONE_VERSION),
        "jup" => Some(JUP_ZONE_VERSION),
        _ => None,
    }
}

/// bo2mp: the legacy game (`iw3` Call of Duty 4, `t4` World at War) a zone
/// file belongs to, by its version.
pub fn legacy_game_for_path(path: &Path) -> Option<&'static str> {
    match peek_zone_version(path)? {
        IW3_ZONE_VERSION => Some("iw3"),
        T4_ZONE_VERSION => Some("t4"),
        IW6_ZONE_VERSION => Some("iw6"),
        IW7_ZONE_VERSION => Some("iw7"),
        T7_ZONE_VERSION => Some("t7"),
        JUP_ZONE_VERSION => Some("jup"),
        _ => None,
    }
}

/// bo2mp: every Call of Duty 4 and World at War multiplayer map installed:
/// (prefix, map), loading zones (`_load`) left out.
pub fn list_legacy_mp_maps(root: &GamesRoot) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> =
        list_legacy_mp_map_files(root).into_iter().map(|(g, s, _)| (g, s)).collect();
    out.dedup();
    out
}

/// bo2mp: [`list_legacy_mp_maps`] with each map's zone file (the first
/// found when a game has two copies).
pub fn list_legacy_mp_map_files(root: &GamesRoot) -> Vec<(String, String, PathBuf)> {
    let mut out: Vec<(String, String, PathBuf)> = Vec::new();
    for path in game_files(&root.0).flatten() {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(str::to_ascii_lowercase) else {
            continue;
        };
        let is_ff = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("ff"));
        if !is_ff || !stem.starts_with("mp_") || stem.ends_with("_load") || is_part_zone(&stem) {
            continue;
        }
        if let Some(game) = legacy_game_for_path(&path)
            && !out.iter().any(|(g, s, _)| g == game && *s == stem)
            && (game != "iw6" || is_ghosts_mp_map(&path, &stem))
            && (game != "iw7" || is_iw7_map(&path, &stem))
            && (game != "t7" || is_t7_map(&stem))
            && (game != "jup" || is_jup_map(&stem))
        {
            out.push((game.to_owned(), stem, path));
        }
    }
    out.sort();
    out
}

/// bo2mp: a Ghosts zone named `mp_*` is a multiplayer map when its
/// loading zone (`<map>_load.ff`) is beside it: Ghosts' `mp_` zones are
/// also its characters' bodies, heads and arms (`mp_body_*`, ...) and its
/// character room; Extinction's maps (`mp_alien_*`) are not multiplayer.
///
/// The DLC maps (Onslaught, Devastation, Invasion, Nemesis) count by their
/// names too, and a map whose `<map>_path.ff` is beside it: a DLC install
/// that ships no loading zone of its own still lists its maps.
fn is_ghosts_mp_map(path: &Path, stem: &str) -> bool {
    const NOT_MAPS: [&str; 5] = ["mp_alien", "mp_body_", "mp_head_", "mp_viewhands_", "mp_character_room"];
    const DLC: [&str; 16] = [
        "mp_swamp",
        "mp_ca_rumble",
        "mp_ca_red_river",
        "mp_boneyard_ns",
        "mp_battery3",
        "mp_ca_impact",
        "mp_ca_behemoth",
        "mp_dome_ns",
        "mp_dig",
        "mp_favela_iw6",
        "mp_pirate",
        "mp_zulu",
        "mp_zerosub",
        "mp_conflict",
        "mp_mine",
        "mp_shipment_ns",
    ];
    if NOT_MAPS.iter().any(|p| stem.starts_with(p)) || stem.ends_with("_path") {
        return false;
    }
    let beside = |suffix: &str| path.parent().is_some_and(|dir| dir.join(format!("{stem}{suffix}.ff")).is_file());
    beside("_load") || beside("_path") || DLC.contains(&stem)
}

/// bo2mp: an Infinite Warfare zone is a map when its material zone
/// (`techsets_<map>.ff`) is beside it: IW7's `mp_` zones are also its
/// characters' bodies, heads and arms and its guns' parts (`*_tr`), and its
/// front ends (`mp_frontend`, `cp_frontend`) are no maps.
pub fn is_iw7_map(path: &Path, stem: &str) -> bool {
    !stem.ends_with("_frontend")
        && path
            .parent()
            .is_some_and(|dir| dir.join(format!("techsets_{stem}.ff")).is_file())
}

/// bo2mp: a Black Ops III zone named `mp_*`/`zm_*` is a map unless it is
/// one of the shared zones (`mp_common`, `zm_levelcommon`, the `_patch` and
/// `_all` zones) or a Freerun course (`mp_freerun_*`, no multiplayer map).
pub fn is_t7_map(stem: &str) -> bool {
    (stem.starts_with("mp_") || stem.starts_with("zm_"))
        && !stem.ends_with("_patch")
        && !stem.ends_with("_all")
        && !stem.starts_with("mp_freerun")
        && !matches!(stem, "mp_common" | "mp_patch" | "zm_common" | "zm_levelcommon" | "zm_patch")
}

/// bo2mp: every Black Ops III Zombies map installed (`zm_*` zones of Black
/// Ops III's version): (map, zone file).
pub fn list_t7_zombies_maps(root: &GamesRoot) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    for path in game_files(&root.0).flatten() {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(str::to_ascii_lowercase) else {
            continue;
        };
        if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("ff")) || !stem.starts_with("zm_") || !is_t7_map(&stem) {
            continue;
        }
        if legacy_game_for_path(&path) == Some("t7") && !out.iter().any(|(s, _)| *s == stem) {
            out.push((stem, path));
        }
    }
    out.sort();
    out
}

/// bo2mp: every Infinite Warfare Zombies map installed (`cp_*` with its
/// material zone beside it): (map, zone file).
pub fn list_iw7_zombies_maps(root: &GamesRoot) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    for path in game_files(&root.0).flatten() {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(str::to_ascii_lowercase) else {
            continue;
        };
        if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("ff")) || !stem.starts_with("cp_") {
            continue;
        }
        if legacy_game_for_path(&path) == Some("iw7") && is_iw7_map(&path, &stem) && !out.iter().any(|(s, _)| *s == stem) {
            out.push((stem, path));
        }
    }
    out.sort();
    out
}

const T5_ZONE_VERSION: u32 = 0x1D9;

const IW5_ZONE_VERSION: u32 = 1;

// bo2zm: Black Ops II PC zones (`TAff0100` envelope).
const T6_ZONE_VERSION: u32 = fastfile_t6::ZONE_VERSION_PC;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GamesRoot(pub PathBuf);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneFile {
    pub path: PathBuf,
    pub zone_name: String,

    pub alias_note: Option<String>,
}

pub fn load_dotenv() {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join(".env");
        if candidate.is_file() {
            let _ = dotenvy::from_path(&candidate);
            return;
        }
    }
    let Ok(mut dir) = std::env::current_dir() else {
        return;
    };
    loop {
        let candidate = dir.join(".env");
        if candidate.is_file() {
            let _ = dotenvy::from_path(&candidate);
            return;
        }
        if !dir.pop() {
            return;
        }
    }
}

pub fn games_root_from_env() -> Result<GamesRoot, String> {
    load_dotenv();
    let path = match std::env::var_os("IW4L_GAMES") {
        Some(raw) => PathBuf::from(raw),
        None => default_games_root()?,
    };
    if !path.is_dir() {
        return Err(format!("IW4L_GAMES is not a directory: {}", path.display()));
    }
    Ok(GamesRoot(path))
}

#[cfg(windows)]
fn default_games_root() -> Result<PathBuf, String> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .ok_or_else(|| {
            "IW4L_GAMES is unset and the portable launcher directory is unavailable".to_owned()
        })
}

#[cfg(not(windows))]
fn default_games_root() -> Result<PathBuf, String> {
    Err("IW4L_GAMES is not set — copy .env.example to .env and set the games root".to_owned())
}

pub(crate) fn search_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![root.to_path_buf()];
    // The game folders he set (game_paths.cfg), searched as if they were
    // under the games root.
    for folder in crate::game_paths::search_folders() {
        if !roots.contains(&folder) {
            roots.push(folder);
        }
    }
    #[cfg(windows)]
    {
        let Ok(entries) = std::fs::read_dir(root) else {
            return roots;
        };
        let mut links = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("lnk"))
            })
            .collect::<Vec<_>>();
        links.sort();
        for link_path in links {
            match shortcut_target_root(&link_path) {
                Ok(target) if !roots.contains(&target) => roots.push(target),
                Ok(_) => {}
                Err(error) => diag::warn!(
                    Zone,
                    "game shortcut {} ignored: {error}",
                    link_path.display()
                ),
            }
        }
    }
    roots
}

#[cfg(windows)]
fn shortcut_target_root(link_path: &Path) -> Result<PathBuf, String> {
    let link = lnk::ShellLink::open(link_path, lnk::encoding::WINDOWS_1252)
        .map_err(|error| format!("cannot parse .lnk: {error}"))?;
    let info = link
        .link_info()
        .as_ref()
        .ok_or_else(|| ".lnk has no LinkInfo filesystem target".to_owned())?;
    if !info.link_info_flags().has_volume_id_and_local_base_path() {
        return Err(".lnk target is not on a local filesystem".to_owned());
    }
    let mut target = info
        .local_base_path_unicode()
        .as_deref()
        .or_else(|| info.local_base_path())
        .map(PathBuf::from)
        .ok_or_else(|| ".lnk has no local base path".to_owned())?;
    let suffix = info
        .common_path_suffix_unicode()
        .as_deref()
        .unwrap_or_else(|| info.common_path_suffix());
    if !suffix.is_empty() {
        target.push(suffix);
    }
    game_root_for_shortcut_target(&target)
}

#[cfg(windows)]
fn game_root_for_shortcut_target(target: &Path) -> Result<PathBuf, String> {
    if target.is_dir() {
        return Ok(target.to_path_buf());
    }
    if target.is_file()
        && let Some(parent) = target.parent()
    {
        return Ok(parent.to_path_buf());
    }
    Err(format!("target does not exist: {}", target.display()))
}

pub fn peek_zone_version(path: &Path) -> Option<u32> {
    read_zone_version(path).ok()
}

fn read_zone_version(path: &Path) -> Result<u32, String> {
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut header = [0u8; 12];
    use std::io::Read;
    file.read_exact(&mut header)
        .map_err(|error| error.to_string())?;
    // bo2zm: T6 envelopes are `TAff`/`TAsv`, version at the same offset.
    // bo2mp: IW4x usermaps (`IW4x` + its header version) keep the stock
    // version at the same offset too.
    if &header[0..4] != b"IWff"
        && &header[0..4] != IW4X_MAGIC
        && !fastfile_t6::is_t6_magic(&header[..8])
        && &header[..8] != T7_MAGIC
    {
        return Err(format!("not an IWff envelope: {:02x?}", &header[..8]));
    }
    Ok(u32::from_le_bytes(header[8..12].try_into().unwrap()))
}

fn game_files(root: &Path) -> impl Iterator<Item = Result<PathBuf, String>> {
    // bo2zm: then the extra read-only folders (`IW4L_T6_EXTRA`).
    let mut roots = search_roots(root);
    roots.extend(crate::t6_extra::extra_roots().iter().cloned());
    files_under(roots)
}

/// Build output and logs a games root may hold (a cargo target folder, the
/// game's own artifacts): never game files, and slow to walk. (From
/// upstream IW4L.)
const NOT_GAME_DIRS: [&str; 5] = ["incremental", "deps", ".fingerprint", "examples", "iw4l-artifacts"];

fn is_not_game_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| NOT_GAME_DIRS.iter().any(|skip| name.eq_ignore_ascii_case(skip)))
}

fn files_under(roots: Vec<PathBuf>) -> impl Iterator<Item = Result<PathBuf, String>> {
    let mut pending: VecDeque<_> = roots.into_iter().map(Ok).collect();
    let mut visited = HashSet::new();
    std::iter::from_fn(move || {
        loop {
            let path = match pending.pop_front()? {
                Ok(path) => path,
                Err(error) => return Some(Err(error)),
            };
            if is_not_game_dir(&path) {
                continue;
            }
            let canonical = match std::fs::canonicalize(&path) {
                Ok(path) => path,
                Err(error) => {
                    return Some(Err(format!("cannot access {}: {error}", path.display())));
                }
            };
            if !visited.insert(canonical) {
                continue;
            }
            let metadata = match std::fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) => return Some(Err(format!("cannot stat {}: {error}", path.display()))),
            };
            if metadata.is_file() {
                return Some(Ok(path));
            }
            if !metadata.is_dir() {
                continue;
            }
            let entries = match std::fs::read_dir(&path) {
                Ok(entries) => entries,
                Err(error) => return Some(Err(format!("cannot read {}: {error}", path.display()))),
            };
            // Every caller looks for zones (`.ff`): plain files of any other
            // kind are dropped here, before the per-path canonicalize and
            // stat (Ghosts and later keep 100+ GB in hundreds of
            // `imagefile*.pak` / `soundfile*.pak` files beside their zones).
            // Links are kept: they may point at folders.
            let mut children = entries
                .filter(|entry| {
                    entry.as_ref().map_or(true, |entry| {
                        !entry.file_type().is_ok_and(|t| t.is_file())
                            || entry
                                .path()
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("ff"))
                    })
                })
                .map(|entry| {
                    entry.map(|entry| entry.path()).map_err(|error| {
                        format!("cannot read entry in {}: {error}", path.display())
                    })
                })
                .collect::<Vec<_>>();
            children.sort();
            pending.extend(children);
        }
    })
}

pub fn zone_game_for_path(path: &Path) -> Option<crate::ZoneGame> {
    match peek_zone_version(path)? {
        IW4_ZONE_VERSION => Some(crate::ZoneGame::Iw4),
        T5_ZONE_VERSION => Some(crate::ZoneGame::T5),
        IW5_ZONE_VERSION => Some(crate::ZoneGame::Iw5),
        T6_ZONE_VERSION => Some(crate::ZoneGame::T6),
        _ => None,
    }
}

pub fn find_zone_file_under(search_root: &Path, zone: &str) -> Result<ZoneFile, String> {
    let file_name = format!("{zone}.ff");
    let mut iw4: Option<PathBuf> = None;
    let mut t5: Option<PathBuf> = None;
    let mut iw5: Option<PathBuf> = None;
    let mut t6: Option<PathBuf> = None;
    let mut other: Option<(PathBuf, u32)> = None;
    let mut errors = Vec::new();
    for entry in game_files(search_root) {
        let path = match entry {
            Ok(path) => path,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        if !path
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case(&file_name))
        {
            continue;
        }
        match peek_zone_version(&path) {
            Some(IW4_ZONE_VERSION) if iw4.is_none() => iw4 = Some(path),
            Some(T5_ZONE_VERSION) if t5.is_none() => t5 = Some(path),
            Some(IW5_ZONE_VERSION) if iw5.is_none() => iw5 = Some(path),
            Some(T6_ZONE_VERSION) if t6.is_none() => t6 = Some(path),
            Some(v) if other.is_none() => other = Some((path, v)),
            None if other.is_none() => other = Some((path, 0)),
            _ => {}
        }
    }
    if let Some(path) = iw4 {
        return Ok(ZoneFile {
            path,
            zone_name: zone.to_owned(),
            alias_note: None,
        });
    }
    if let Some(path) = t5 {
        return Ok(ZoneFile {
            path,
            zone_name: zone.to_owned(),
            alias_note: None,
        });
    }
    if let Some(path) = iw5 {
        return Ok(ZoneFile {
            path,
            zone_name: zone.to_owned(),
            alias_note: None,
        });
    }
    if let Some(path) = t6 {
        return Ok(ZoneFile {
            path,
            zone_name: zone.to_owned(),
            alias_note: None,
        });
    }
    if let Some((path, version)) = other {
        if version == 0 {
            return Err(format!(
                "zone `{file_name}` at {} is not a readable IWff envelope \
                 (need IW4 0x114, T5 0x1d9, IW5 0x1, or T6 0x93)",
                path.display()
            ));
        }
        return Err(format!(
            "zone `{file_name}` found at {} but version is {version:#x} \
             (want IW4 0x114, T5 0x1d9, IW5 0x1, or T6 0x93)",
            path.display()
        ));
    }
    Err(format!(
        "zone `{file_name}` not found under {}; scan errors: {}",
        search_root.display(),
        if errors.is_empty() {
            "none".to_owned()
        } else {
            errors.join("; ")
        }
    ))
}

pub fn find_zone_file_version(
    root: &GamesRoot,
    zone: &str,
    version: u32,
) -> Result<ZoneFile, String> {
    let file_name = format!("{zone}.ff");
    let mut found: Option<PathBuf> = None;
    let mut rejected = Vec::new();
    let mut usermap_gap_message: Option<String> = None;
    for entry in game_files(&root.0) {
        let path = match entry {
            Ok(path) => path,
            Err(error) => {
                rejected.push(error);
                continue;
            }
        };
        if !path
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case(&file_name))
        {
            continue;
        }
        let actual = read_zone_version(&path);
        if actual == Ok(version) {
            found = Some(path);
            break;
        }
        // bo2mp: a usermap in a layout the readers do not take says why.
        if version == IW4_ZONE_VERSION
            && usermap_gap_message.is_none()
            && usermap_zone_of(&path).is_some()
        {
            usermap_gap_message =
                usermap_gap(&path, actual.as_ref().ok().copied(), is_iw4x_envelope(&path));
        }
        rejected.push(format!(
            "{} ({})",
            path.display(),
            actual.map_or_else(|error| error, |v| format!("version {v:#x}"))
        ));
    }
    if found.is_none()
        && let Some(gap) = usermap_gap_message
    {
        diag::warn!(Zone, "{gap}");
        return Err(gap);
    }
    let path = found.ok_or_else(|| {
        format!(
            "zone `{file_name}` version {version:#x} not found; {}; other candidates: {}",
            games_root_report(root),
            if rejected.is_empty() {
                "none".to_owned()
            } else {
                rejected.join("; ")
            }
        )
    })?;
    if let Some((map, is_load)) = usermap_zone_of(&path) {
        diag::info!(
            Zone,
            "usermap {map}: {} zone {}{}",
            if is_load { "loading" } else { "map" },
            path.display(),
            if is_iw4x_envelope(&path) { " (IW4x envelope)" } else { "" }
        );
    }
    Ok(ZoneFile {
        path,
        zone_name: zone.to_owned(),
        alias_note: None,
    })
}

pub fn split_zone_key(zone: &str) -> (Option<crate::ZoneGame>, &str) {
    let Some((prefix, rest)) = zone.split_once(':') else {
        return (None, zone);
    };
    match crate::ZoneGame::from_prefix(prefix) {
        Some(game) if !rest.is_empty() => (Some(game), rest),
        _ => (None, zone),
    }
}

pub fn map_load_title(key: &str, game: Option<crate::ZoneGame>) -> String {
    let (parsed, stem) = split_zone_key(key);
    match parsed.or(game) {
        Some(game) => format!("{}:{stem}", game.prefix()),
        None => stem.to_owned(),
    }
}

pub fn group_mp_maps(maps: &[String]) -> [Vec<&str>; 3] {
    let mut cols = [Vec::new(), Vec::new(), Vec::new()];
    for map in maps {
        let col = match split_zone_key(map).0 {
            Some(crate::ZoneGame::Iw4) | None => 0,
            Some(crate::ZoneGame::Iw5) => 1,
            // bo2zm: Treyarch maps share the Black Ops column.
            Some(crate::ZoneGame::T5) | Some(crate::ZoneGame::T6) => 2,
        };
        cols[col].push(map.as_str());
    }
    cols
}

pub fn find_zone_file(root: &GamesRoot, zone: &str) -> Result<ZoneFile, String> {
    let zone = zone.trim().to_ascii_lowercase();
    if zone.is_empty() {
        return Err("zone name is empty".into());
    }
    // bo2mp: Call of Duty 4 (`iw3:`) and World at War (`t4:`) maps, which
    // load through their own readers (no `ZoneGame` of their own).
    if let Some((prefix, stem)) = zone.split_once(':')
        && let Some(version) = legacy_version(prefix)
    {
        return find_zone_file_version(root, stem, version);
    }
    let (game, stem) = split_zone_key(&zone);
    find_zone_stem(root, game, stem)
}

pub fn zone_version(game: crate::ZoneGame) -> u32 {
    match game {
        crate::ZoneGame::Iw4 => IW4_ZONE_VERSION,
        crate::ZoneGame::T5 => T5_ZONE_VERSION,
        crate::ZoneGame::Iw5 => IW5_ZONE_VERSION,
        crate::ZoneGame::T6 => T6_ZONE_VERSION,
    }
}

fn find_stem_file(
    root: &GamesRoot,
    game: Option<crate::ZoneGame>,
    stem: &str,
) -> Result<ZoneFile, String> {
    match game {
        Some(game) => find_zone_file_version(root, stem, zone_version(game)),
        None => find_zone_file_under(&root.0, stem),
    }
}

fn find_zone_stem(
    root: &GamesRoot,
    game: Option<crate::ZoneGame>,
    stem: &str,
) -> Result<ZoneFile, String> {
    if stem.starts_with("mp_") {
        return find_stem_file(root, game, stem);
    }
    let Some(prefixed) = resolve_mp_zone_alias(stem) else {
        return find_stem_file(root, game, stem);
    };
    match find_stem_file(root, game, &prefixed) {
        Ok(mut found) => {
            let note = format!(
                "zone alias: `{stem}` -> `{prefixed}` (MP stem preferred over bare `{stem}.ff`)"
            );
            found.alias_note = Some(note.clone());
            match std::fs::metadata(&found.path) {
                Ok(meta) => diag::info!(Zone, "{note} ({} bytes)", meta.len()),
                Err(_) => diag::info!(Zone, "{note}"),
            }
            Ok(found)
        }
        Err(mp_err) => match find_stem_file(root, game, stem) {
            Ok(found) => Ok(found),
            Err(bare_err) => Err(format!("{mp_err}; {bare_err}")),
        },
    }
}

pub fn resolve_mp_zone_alias(zone: &str) -> Option<String> {
    if zone.is_empty() || zone.starts_with("mp_") || zone.ends_with("_mp") {
        return None;
    }
    Some(format!("mp_{zone}"))
}

pub fn find_common_mp_for_zone(zone_ff: &Path) -> Result<ZoneFile, String> {
    find_named_zone_for_tree(zone_ff, "common_mp")
}

pub fn find_runtime_zone(root: &GamesRoot, zone_ff: &Path, zone: &str) -> Result<ZoneFile, String> {
    match zone_game_for_path(zone_ff) {
        Some(crate::ZoneGame::Iw4) | None => find_zone_for_tree(zone_ff, zone),
        Some(_) => {
            let found = find_zone_file_under(&root.0, zone)?;
            match peek_zone_version(&found.path) {
                Some(IW4_ZONE_VERSION) => Ok(found),
                Some(version) => Err(format!(
                    "runtime profile is IW4 FFA; {zone} at {} is version {version:#x} \
                     (want IW4 {IW4_ZONE_VERSION:#x})",
                    found.path.display()
                )),
                None => Err(format!(
                    "runtime profile is IW4 FFA; {zone} at {} is not a readable IWff envelope",
                    found.path.display()
                )),
            }
        }
    }
}

pub fn find_runtime_common_mp(root: &GamesRoot, zone_ff: &Path) -> Result<ZoneFile, String> {
    find_runtime_zone(root, zone_ff, "common_mp")
}

pub fn find_common_mp_for_envelope(root: &GamesRoot, version: u32) -> Result<ZoneFile, String> {
    find_zone_file_version(root, "common_mp", version)
}

pub fn find_localized_common_mp_for_zone(zone_ff: &Path) -> Result<ZoneFile, String> {
    find_named_zone_for_tree(zone_ff, "localized_common_mp")
}

pub fn find_zone_for_tree(zone_ff: &Path, zone: &str) -> Result<ZoneFile, String> {
    find_named_zone_for_tree(zone_ff, zone)
}

fn find_named_zone_for_tree(zone_ff: &Path, zone: &str) -> Result<ZoneFile, String> {
    find_zone_file_under(&game_root_for_zone(zone_ff)?, zone)
}

pub fn game_root_for_zone(zone_ff: &Path) -> Result<PathBuf, String> {
    let mut cursor = zone_ff.parent();
    while let Some(dir) = cursor {
        if dir.join("zone").is_dir() {
            return Ok(dir.to_path_buf());
        }
        cursor = dir.parent();
    }
    Err(format!(
        "no game tree with a `zone/` directory above {}",
        zone_ff.display()
    ))
}

pub fn games_root_report(root: &GamesRoot) -> String {
    let roots = search_roots(&root.0);
    let mut line = format!("games root: {}", root.0.display());
    for extra in roots.iter().skip(1) {
        line.push_str(&format!("; shortcut root: {}", extra.display()));
    }
    line
}

pub fn games_content_report(root: &GamesRoot) -> Vec<String> {
    #[derive(Default)]
    struct Content {
        versions: BTreeMap<u32, usize>,
        files: usize,
        dlc: usize,
        iw4_common: bool,
    }
    let mut trees: BTreeMap<PathBuf, Content> = BTreeMap::new();
    let mut report = Vec::new();
    for entry in game_files(&root.0) {
        let path = match entry {
            Ok(path) => path,
            Err(error) => {
                report.push(format!("asset scan: {error}"));
                continue;
            }
        };
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ff"))
        {
            continue;
        }
        let tree = game_root_for_zone(&path).unwrap_or_else(|_| root.0.clone());
        let is_dlc = path.strip_prefix(&tree).ok().is_some_and(|relative| {
            relative.parent().is_some_and(|parent| {
                parent
                    .components()
                    .any(|part| part.as_os_str().eq_ignore_ascii_case("dlc"))
            })
        });
        let content = trees.entry(tree).or_default();
        content.files += 1;
        if is_dlc {
            content.dlc += 1;
        }
        match read_zone_version(&path) {
            Ok(version) => {
                *content.versions.entry(version).or_default() += 1;
                if version == IW4_ZONE_VERSION
                    && path
                        .file_name()
                        .is_some_and(|name| name.eq_ignore_ascii_case("common_mp.ff"))
                {
                    content.iw4_common = true;
                }
            }
            Err(error) => report.push(format!("asset header {}: {error}", path.display())),
        }
    }
    if trees.is_empty() {
        report.push(
            "No .ff files found in the configured game folders and shortcut targets.".to_owned(),
        );
    }
    for (tree, content) in trees {
        let versions = content
            .versions
            .iter()
            .map(|(version, count)| format!("{version:#x}: {count}"))
            .collect::<Vec<_>>()
            .join(", ");
        report.push(format!(
            "assets {}: {} FastFiles, {} in DLC folders; envelope versions {{{versions}}}",
            tree.display(),
            content.files,
            content.dlc
        ));
        if content.files == content.dlc && !content.iw4_common {
            report.push("Only DLC FastFiles were found in this tree; DLC maps do not supply the base multiplayer menu assets.".to_owned());
        }
        if content.versions.contains_key(&IW4_ZONE_VERSION) {
            report.push(format!(
                "IW4 common_mp.ff: {}",
                if content.iw4_common {
                    "found"
                } else {
                    "missing from this tree"
                }
            ));
        }
    }
    report
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapPack {
    pub label: String,
    pub maps: Vec<String>,
}

pub fn list_mp_maps(root: &GamesRoot) -> Vec<String> {
    let mut maps: Vec<String> = list_mp_map_packs(root)
        .into_iter()
        .flat_map(|pack| pack.maps)
        .collect();
    maps.sort();
    maps
}

pub fn list_mp_map_packs(root: &GamesRoot) -> Vec<MapPack> {
    map_packs_in(game_files(&root.0))
}

fn map_packs_in(files: impl Iterator<Item = Result<PathBuf, String>>) -> Vec<MapPack> {
    let mut zones = Vec::new();
    // bo2mp: usermaps (`usermaps/<map>/<map>.ff`) are listed on their own:
    // they need no `_load` zone and keep their own pack.
    let mut usermap_files = Vec::new();
    for entry in files {
        let path = match entry {
            Ok(path) => path,
            Err(error) => {
                diag::warn!(Zone, "map discovery: {error}");
                continue;
            }
        };
        if usermap_zone_of(&path).is_some() {
            usermap_files.push(Ok(path));
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let stem = stem.to_ascii_lowercase();
        let is_ff = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ff"));
        if !(is_ff && stem.starts_with("mp_")) {
            continue;
        }
        if let Some(game) = zone_game_for_path(&path) {
            zones.push((game.prefix(), stem, map_pack_folder(&path)));
        }
    }
    let loads: HashSet<(&str, &str)> = zones
        .iter()
        .filter_map(|(game, stem, _)| stem.strip_suffix("_load").map(|map| (*game, map)))
        .collect();
    let mut packs: BTreeMap<(&str, &str), Vec<String>> = BTreeMap::new();
    let mut seen = HashSet::new();
    for (game, stem, folder) in &zones {
        if stem.ends_with("_load")
            || !(loads.contains(&(*game, stem.as_str())) || !loads.iter().any(|(g, _)| g == game))
        {
            continue;
        }
        let map = format!("{game}:{stem}");
        if seen.insert(map.clone()) {
            packs.entry((game, folder)).or_default().push(map);
        }
    }
    let mut out: Vec<MapPack> = packs
        .into_iter()
        .map(|((game, folder), mut maps)| {
            maps.sort();
            MapPack {
                label: format!("{} {folder}", game.to_ascii_uppercase()),
                maps,
            }
        })
        .collect();
    let mut custom: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for usermap in usermaps_in(usermap_files.into_iter()) {
        let map = format!("{}:{}", usermap.game, usermap.stem);
        if seen.insert(map.clone()) {
            custom.entry(usermap.game).or_default().push(map);
        } else {
            diag::warn!(
                Zone,
                "usermap {} at {} has the name of an installed map; the installed one is listed",
                usermap.stem,
                usermap.zone.display()
            );
        }
    }
    out.extend(custom.into_iter().map(|(game, maps)| MapPack {
        label: format!("{} {USERMAPS_PACK}", game.to_ascii_uppercase()),
        maps,
    }));
    out
}

/// bo2mp: the pack label suffix of a game's usermaps (`IW4 USERMAPS`).
pub const USERMAPS_PACK: &str = "USERMAPS";

fn map_pack_folder(zone_ff: &Path) -> String {
    game_root_for_zone(zone_ff)
        .ok()
        .and_then(|tree| {
            zone_ff
                .strip_prefix(tree.join("zone"))
                .ok()?
                .components()
                .next()?
                .as_os_str()
                .to_str()
                .map(str::to_ascii_uppercase)
        })
        .filter(|folder| folder.starts_with("DLC"))
        .unwrap_or_else(|| "BASE".to_owned())
}

pub fn ensure_artifacts_dir() -> Result<PathBuf, String> {
    let dir = Path::new("iw4l-artifacts");
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir.to_path_buf())
}

/// T5 language archives carry a language prefix rather than `localized_`.
pub fn find_t5_localized_zones(
    anchor: &Path,
    language: Option<&str>,
) -> Result<Vec<ZoneFile>, String> {
    let (dir, prefix) = t5_language_archive(anchor, language)?;
    Ok(["code_post_gfx_mp", "common_mp", "ui_mp"]
        .into_iter()
        .filter_map(|stem| t5_localized_zone(&dir, &prefix, stem))
        .collect())
}

pub fn find_t5_localized_zone(
    anchor: &Path,
    language: Option<&str>,
    stem: &str,
) -> Result<Option<ZoneFile>, String> {
    let (dir, prefix) = t5_language_archive(anchor, language)?;
    Ok(t5_localized_zone(&dir, &prefix, stem))
}

fn t5_localized_zone(dir: &Path, prefix: &str, stem: &str) -> Option<ZoneFile> {
    let zone_name = format!("{prefix}{stem}");
    let path = dir.join(format!("{zone_name}.ff"));
    path.is_file().then_some(ZoneFile {
        path,
        zone_name,
        alias_note: None,
    })
}

fn t5_language_archive(anchor: &Path, language: Option<&str>) -> Result<(PathBuf, String), String> {
    let root = game_root_for_zone(anchor)?.join("zone");
    let mut choices: Vec<_> = files_under(vec![root])
        .filter_map(Result::ok)
        .filter(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|name| name.ends_with("_code_post_gfx_mp.ff"))
                && zone_game_for_path(path) == Some(crate::ZoneGame::T5)
        })
        .collect();
    choices.sort();
    let preferred = choices.iter().position(|path| {
        path.parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .zip(language)
            .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b))
    });
    let chosen = match preferred {
        Some(index) => &choices[index],
        None if choices.len() == 1 => &choices[0],
        _ => {
            return Err(format!(
                "T5 language archive selection is ambiguous or missing: {} candidates",
                choices.len()
            ));
        }
    };
    let name = chosen
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("T5 language archive name")?;
    let prefix = name
        .strip_suffix("code_post_gfx_mp.ff")
        .ok_or("T5 language prefix")?;
    let dir = chosen.parent().ok_or("T5 language directory")?;
    Ok((dir.to_path_buf(), prefix.to_owned()))
}

#[cfg(test)]
mod usermap_tests {
    use super::*;
    use std::io::Write;

    fn zone_bytes(magic: &[u8; 8], version: u32) -> Vec<u8> {
        let mut bytes = magic.to_vec();
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 9]);
        bytes.extend_from_slice(b"body");
        bytes
    }

    fn put(path: &Path, bytes: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    fn put_iwd(path: &Path, entries: &[&str]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for entry in entries {
            zip.start_file(*entry, options).unwrap();
            zip.write_all(b"data").unwrap();
        }
        zip.finish().unwrap();
    }

    /// A games root with MW2 (stock maps with `_load` zones) and usermaps:
    /// an IW4x-built castle with its `_load` and `.iwd`, a CoD Online alpha
    /// zone, a broken zone, a broken archive, and stray files.
    fn games(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("bo2mp-usermaps-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mw2 = root.join("mw2");
        put(&mw2.join("zone/english/common_mp.ff"), &zone_bytes(b"IWffu100", 0x114));
        put(&mw2.join("zone/english/mp_rust.ff"), &zone_bytes(b"IWffu100", 0x114));
        put(&mw2.join("zone/english/mp_rust_load.ff"), &zone_bytes(b"IWffu100", 0x114));
        put_iwd(&mw2.join("main/iw_00.iwd"), &["images/stock.iwi", "sound/stock/a.wav"]);
        let iw4x = *b"IW4x\x03\x00\x00\x00";
        let castle = mw2.join("usermaps/mp_waw_castle");
        put(&castle.join("mp_waw_castle.ff"), &zone_bytes(&iw4x, 0x114));
        put(&castle.join("mp_waw_castle_load.ff"), &zone_bytes(&iw4x, 0x114));
        put_iwd(
            &castle.join("mp_waw_castle.iwd"),
            &["images/castle_wall.iwi", "images/stock.iwi", "sound/castle/amb.wav"],
        );
        put(&mw2.join("usermaps/mp_alpha/mp_alpha.ff"), &zone_bytes(b"IWffu100", 316));
        put(&mw2.join("usermaps/mp_broken/mp_broken.ff"), b"nope");
        put(&mw2.join("usermaps/mp_broken/mp_broken.iwd"), b"not a zip");
        put(&mw2.join("usermaps/mp_other/mp_misnamed.ff"), &zone_bytes(b"IWffu100", 0x114));
        put(&mw2.join("usermaps/stray.ff"), &zone_bytes(b"IWffu100", 0x114));
        std::fs::create_dir_all(mw2.join("usermaps/empty")).unwrap();
        root
    }

    #[test]
    fn zone_scan_skips_paks_and_lists_ghosts_zones_without_reading_them_as_maps() {
        let root = games("paks");
        let ghosts = root.join("ghosts");
        put(&ghosts.join("zone/code_post_gfx_mp.ff"), &zone_bytes(b"IWff0100", 0x235));
        put(&ghosts.join("zone/mp_prisonbreak.ff"), &zone_bytes(b"IWff0100", 0x235));
        put(&ghosts.join("zone/imagefile3.pak"), b"pixels");
        put(&ghosts.join("zone/soundfile1.pak"), b"fLaC");
        let files: Vec<PathBuf> = files_under(vec![root.clone()]).flatten().collect();
        assert!(files.iter().any(|p| p.ends_with("ghosts/zone/code_post_gfx_mp.ff")));
        assert!(!files.iter().any(|p| p.extension().is_some_and(|e| e == "pak" || e == "iwd")));
        // Ghosts maps are not offered (no lane reads version 0x235).
        let maps = map_packs_in(files_under(vec![root.clone()]));
        assert!(!maps.iter().flat_map(|p| &p.maps).any(|m| m.contains("prisonbreak")));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn usermap_zone_names() {
        let p = Path::new("/g/mw2/usermaps/mp_waw_castle/mp_waw_castle.ff");
        assert_eq!(usermap_zone_of(p), Some(("mp_waw_castle".into(), false)));
        let p = Path::new("/g/mw2/UserMaps/mp_waw_castle/MP_WAW_CASTLE_LOAD.FF");
        assert_eq!(usermap_zone_of(p), Some(("mp_waw_castle".into(), true)));
        assert_eq!(usermap_zone_of(Path::new("/g/mw2/usermaps/mp_x/mp_y.ff")), None);
        assert_eq!(usermap_zone_of(Path::new("/g/mw2/usermaps/mp_x/mp_x.iwd")), None);
        assert_eq!(usermap_zone_of(Path::new("/g/mw2/zone/english/mp_rust.ff")), None);
        assert_eq!(usermap_zone_of(Path::new("/g/mw2/usermaps/stray.ff")), None);
    }

    #[test]
    fn discovery_lists_usermaps_without_load_rule() {
        let root = games("discover");
        let packs = map_packs_in(files_under(vec![root.clone()]));
        let base = packs.iter().find(|p| p.label == "IW4 BASE").expect("base pack");
        assert_eq!(base.maps, vec!["iw4:mp_rust".to_owned()]);
        let custom = packs.iter().find(|p| p.label == "IW4 USERMAPS").expect("usermaps pack");
        assert_eq!(custom.maps, vec!["iw4:mp_alpha".to_owned(), "iw4:mp_waw_castle".to_owned()]);
        assert!(packs.iter().flat_map(|p| &p.maps).all(|m| !m.ends_with("_load")
            && !m.contains("broken")
            && !m.contains("misnamed")
            && !m.contains("stray")));

        let maps = usermaps_in(files_under(vec![root.clone()]));
        let castle = maps.iter().find(|m| m.stem == "mp_waw_castle").unwrap();
        assert_eq!(castle.game, "iw4");
        assert!(castle.iw4x_envelope);
        assert_eq!(castle.version, Some(0x114));
        assert!(castle.gap().is_none());
        assert!(castle.load_zone.as_ref().unwrap().ends_with("mp_waw_castle/mp_waw_castle_load.ff"));
        assert_eq!(castle.iwds.len(), 1);
        let alpha = maps.iter().find(|m| m.stem == "mp_alpha").unwrap();
        assert!(alpha.gap().unwrap().contains("0x13c"));
        assert!(!maps.iter().any(|m| m.stem == "mp_broken"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn find_zone_resolves_usermaps() {
        let root = games("find");
        let games = GamesRoot(root.clone());
        let found = find_zone_file(&games, "iw4:mp_waw_castle").unwrap();
        assert!(found.path.ends_with("usermaps/mp_waw_castle/mp_waw_castle.ff"));
        assert_eq!(found.zone_name, "mp_waw_castle");
        let load = find_zone_file(&games, "iw4:mp_waw_castle_load").unwrap();
        assert!(load.path.ends_with("usermaps/mp_waw_castle/mp_waw_castle_load.ff"));
        // Untagged lookups see the IW4x envelope as MW2's.
        assert_eq!(zone_game_for_path(&found.path), Some(crate::ZoneGame::Iw4));
        // Its common zones come from the MW2 tree above `usermaps`.
        assert_eq!(game_root_for_zone(&found.path).unwrap(), root.join("mw2"));
        assert!(find_common_mp_for_zone(&found.path).is_ok());
        assert_eq!(crate::iwd::game_main_for_zone(&found.path).unwrap(), root.join("mw2/main"));
        // A CoD Online layout fails with the gap, not a misread.
        let alpha = find_zone_file(&games, "iw4:mp_alpha").unwrap_err();
        assert!(alpha.contains("0x13c") && alpha.contains("gap"), "{alpha}");
        assert!(find_zone_file(&games, "iw4:mp_broken").is_err());
        // Stock maps are untouched.
        assert!(find_zone_file(&games, "iw4:mp_rust").unwrap().path.ends_with("zone/english/mp_rust.ff"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn usermap_iwds_join_main() {
        let root = games("iwd");
        let main = root.join("mw2/main");
        let iwds = usermap_iwds_for_main(&main);
        assert_eq!(iwds.len(), 2, "{iwds:?}");
        assert!(usermap_iwds_for_main(&root.join("mw2/zone")).is_empty());

        let index = crate::IwdIndex::open(&main).expect("a broken usermap IWD does not fail main");
        assert!(index.image_candidates("castle_wall").is_some());
        // Stock archives come first for a name both hold.
        let stock = index.image_candidates("stock").unwrap();
        assert!(stock[0].read().is_ok());
        assert_eq!(stock.len(), 2);
        assert!(stock[1].read().is_ok());
        let sounds = crate::IwdSoundIndex::open(&main).unwrap();
        assert!(sounds.read_sound("castle/amb.wav").is_some());
        assert!(sounds.read_sound("stock/a.wav").is_some());
        let _ = std::fs::remove_dir_all(&root);
    }
}
