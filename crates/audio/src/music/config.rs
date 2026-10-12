//! `music.cfg` and the user's music folders.
//!
//! Where things are looked for:
//! * the music folder: `IW4L_MUSIC_DIR`, else `<exe dir>/music`; its
//!   `menu/`, `lobby/` and `match/` subfolders feed those playlists (`.mp3`,
//!   `.ogg`, `.wav`, `.flac`);
//! * the config: `IW4L_MUSIC_CFG`, else `<exe dir>/music.cfg`, else
//!   `<music folder>/music.cfg`. A commented template is written beside the
//!   exe when none exists.
//!
//! The config is `key = value` lines (`//` comments, or `#` at a line's start):
//! ```text
//! enabled = 1            // 0: no director music at all
//! shuffle = 1            // 0: play each playlist in order
//! menu_level = 1.0       // per-context level, under music_volume
//! lobby_level = 1.0
//! match_level = 0.25     // in-match ambience stays low
//! fade_in = 2.0          // seconds
//! fade_out = 1.5
//! crossfade = 4.0
//! menu = defaults        // a playlist line; any line for a context replaces
//! menu = music/menu      // its defaults. An entry is `defaults`, a game
//! menu = t6:mus_mp_frontend  // alias (`t6:`, `t5:`, `iw4:`, `iw5:`), a
//! lobby = D:/Music/x.mp3 // folder, or a file (relative to the exe dir).
//! match = none           // an empty playlist
//! ```

use std::path::{Path, PathBuf};

use asset_core::AssetNamespace;

use super::logic::{FadeTimings, MusicContext};
use super::stream::MUSIC_EXTENSIONS;

/// The game tracks the code knows how to load: each game's main-menu music,
/// from the front end's bank (BO2's own; BO1's, MW2's and MW3's are taken
/// in by `asset_audio::menu_music_guests`).
pub const GAME_TRACKS: [(AssetNamespace, &str, &str); 4] = [
    (AssetNamespace::T6, "mus_mp_frontend", "Black Ops II"),
    (AssetNamespace::T5, "mus_mp_frontend", "Black Ops"),
    (AssetNamespace::Iw4, "music_mainmenu_mp", "Modern Warfare 2"),
    (AssetNamespace::Iw5, "music_mainmenu_mp", "Modern Warfare 3"),
];

pub fn game_label(ns: AssetNamespace) -> &'static str {
    match ns {
        AssetNamespace::T6 => "Black Ops II",
        AssetNamespace::T5 => "Black Ops",
        AssetNamespace::Iw4 => "Modern Warfare 2",
        AssetNamespace::Iw5 => "Modern Warfare 3",
    }
}

fn ns_of_prefix(prefix: &str) -> Option<AssetNamespace> {
    match prefix.to_ascii_lowercase().as_str() {
        "t6" | "bo2" => Some(AssetNamespace::T6),
        "t5" | "bo1" => Some(AssetNamespace::T5),
        "iw4" | "mw2" => Some(AssetNamespace::Iw4),
        "iw5" | "mw3" => Some(AssetNamespace::Iw5),
        _ => None,
    }
}

/// One playlist line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Defaults,
    None,
    Game(AssetNamespace, String),
    Path(PathBuf),
}

#[derive(Clone, Debug, PartialEq)]
pub struct MusicConfig {
    pub enabled: bool,
    pub shuffle: bool,
    pub levels: [f32; 3],
    pub timings: FadeTimings,
    /// Per context (menu, lobby, match): `None` = the defaults.
    pub playlists: [Option<Vec<Entry>>; 3],
    /// Where it was read from, if anywhere.
    pub source: Option<PathBuf>,
    pub warnings: Vec<String>,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shuffle: true,
            levels: [1.0, 1.0, 0.25],
            timings: FadeTimings::default(),
            playlists: [None, None, None],
            source: None,
            warnings: Vec::new(),
        }
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub fn parse_entry(value: &str, base: &Path) -> Entry {
    let v = value.trim().trim_matches('"');
    if v.eq_ignore_ascii_case("defaults") {
        return Entry::Defaults;
    }
    if v.eq_ignore_ascii_case("none") || v.is_empty() {
        return Entry::None;
    }
    // `t6:alias` (but not a Windows drive, `C:\...`).
    if let Some((prefix, alias)) = v.split_once(':')
        && prefix.len() > 1
        && let Some(ns) = ns_of_prefix(prefix)
        && !alias.is_empty()
    {
        return Entry::Game(ns, alias.to_owned());
    }
    let path = PathBuf::from(v);
    Entry::Path(if path.is_absolute() { path } else { base.join(path) })
}

impl MusicConfig {
    /// Parses `text`; relative paths are taken from `base`.
    pub fn parse(text: &str, base: &Path) -> Self {
        let mut cfg = Self::default();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.split("//").next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                cfg.warnings.push(format!("line {}: no `=` in `{line}`", n + 1));
                continue;
            };
            let (key, value) = (key.trim().to_ascii_lowercase(), value.trim());
            let num = value.parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.0);
            let mut bad = false;
            match key.as_str() {
                "enabled" => match parse_bool(value) {
                    Some(b) => cfg.enabled = b,
                    None => bad = true,
                },
                "shuffle" => match parse_bool(value) {
                    Some(b) => cfg.shuffle = b,
                    None => bad = true,
                },
                "menu_level" | "lobby_level" | "match_level" => match num {
                    Some(v) => {
                        let slot = match key.as_str() {
                            "menu_level" => 0,
                            "lobby_level" => 1,
                            _ => 2,
                        };
                        cfg.levels[slot] = v.min(1.0) as f32;
                    }
                    None => bad = true,
                },
                "fade_in" | "fade_out" | "crossfade" | "prefetch" => match num {
                    Some(v) => {
                        let v = v.min(60.0);
                        match key.as_str() {
                            "fade_in" => cfg.timings.fade_in = v,
                            "fade_out" => cfg.timings.fade_out = v,
                            "crossfade" => cfg.timings.crossfade = v,
                            _ => cfg.timings.prefetch = v.max(5.0),
                        }
                    }
                    None => bad = true,
                },
                "menu" | "lobby" | "match" => {
                    let slot = match key.as_str() {
                        "menu" => 0,
                        "lobby" => 1,
                        _ => 2,
                    };
                    cfg.playlists[slot]
                        .get_or_insert_with(Vec::new)
                        .push(parse_entry(value, base));
                }
                _ => cfg.warnings.push(format!("line {}: unknown key `{key}`", n + 1)),
            }
            if bad {
                cfg.warnings
                    .push(format!("line {}: bad value `{value}` for `{key}`", n + 1));
            }
        }
        cfg
    }

    pub fn level(&self, context: MusicContext) -> f32 {
        context.slot().map_or(0.0, |s| self.levels[s])
    }
}

/// The folder the exe is in (the working directory when unknown).
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn music_root(exe_dir: &Path) -> PathBuf {
    std::env::var_os("IW4L_MUSIC_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| exe_dir.join("music"))
}

/// The config's path: the override, else beside the exe, else in the music
/// folder; `(path, exists)`.
pub fn config_path(exe_dir: &Path, root: &Path) -> (PathBuf, bool) {
    if let Some(p) = std::env::var_os("IW4L_MUSIC_CFG").filter(|v| !v.is_empty()) {
        let p = PathBuf::from(p);
        let exists = p.is_file();
        return (p, exists);
    }
    let beside = exe_dir.join("music.cfg");
    if beside.is_file() {
        return (beside, true);
    }
    let inside = root.join("music.cfg");
    if inside.is_file() {
        return (inside, true);
    }
    (beside, false)
}

pub const TEMPLATE: &str = "\
// Music director (bo2mp). Lines are `key = value`; `//` starts a comment.
// Drop your own music (.mp3 .ogg .wav .flac) into music/menu, music/lobby and
// music/match beside the exe (or set IW4L_MUSIC_DIR); they join the
// playlists below. Any `menu =`, `lobby =` or `match =` line replaces that
// playlist's defaults; list `defaults` to keep them as well.
enabled = 1
shuffle = 1
menu_level = 1.0
lobby_level = 1.0
match_level = 0.25
fade_in = 2.0
fade_out = 1.5
crossfade = 4.0
// menu = defaults
// menu = t6:mus_mp_frontend
// lobby = music/lobby
// match = none
";

/// Is `path` a music file this build reads?
pub fn is_music_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| MUSIC_EXTENSIONS.iter().any(|m| e.eq_ignore_ascii_case(m)))
}

/// The music files in `dir` (not recursive), sorted; and the files there
/// that are not music, so the manifest can say so.
pub fn scan_dir(dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return (Vec::new(), Vec::new());
    };
    let mut music = Vec::new();
    let mut other = Vec::new();
    for entry in read.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.file_name().is_some_and(|n| n == "music.cfg" || n.to_string_lossy().starts_with('.')) {
            continue;
        }
        if is_music_file(&path) {
            music.push(path);
        } else {
            other.push(path);
        }
    }
    music.sort();
    other.sort();
    (music, other)
}

/// Game folders whose formats this build cannot read, by folder name.
pub fn unreadable_game(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    let has = |s: &str| n.contains(s);
    if has("advanced warfare") || has("advanced_warfare") || n == "aw" || has("s1_mp") || n.ends_with(" aw") {
        Some("Advanced Warfare")
    } else if has("ghosts") || n == "iw6" || has("iw6x") {
        Some("Ghosts")
    } else if has("infinite warfare") || has("infinite_warfare") || n == "iw" || n == "iw7" || n.ends_with(" iw") {
        Some("Infinite Warfare")
    } else {
        None
    }
}

/// Game folders this build reads, but whose music the director does not
/// load (no menu-music loader for them).
pub fn unwired_game(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    if n.contains("world at war") || n == "waw" || n == "t4" || n.contains("cod5") {
        Some("World at War")
    } else if n.contains("call of duty 4") || n == "cod4" || n == "iw3" || n.contains("cod4") {
        Some("Call of Duty 4")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_settings_and_playlists() {
        let base = Path::new("/games/bo2mp");
        let cfg = MusicConfig::parse(
            "enabled = 1\nshuffle=0 // in order\nmatch_level = 0.1\ncrossfade = 6\n\
             menu = defaults\nmenu = t6:mus_mp_frontend\nmenu = music/extra\n\
             lobby = /abs/song.mp3\nmatch = none\nbogus = 1\nfade_in = x\n",
            base,
        );
        assert!(cfg.enabled);
        assert!(!cfg.shuffle);
        assert!((cfg.levels[2] - 0.1).abs() < 1e-6);
        assert_eq!(cfg.timings.crossfade, 6.0);
        assert_eq!(
            cfg.playlists[0],
            Some(vec![
                Entry::Defaults,
                Entry::Game(AssetNamespace::T6, "mus_mp_frontend".into()),
                Entry::Path(base.join("music/extra")),
            ])
        );
        assert_eq!(cfg.playlists[1], Some(vec![Entry::Path("/abs/song.mp3".into())]));
        assert_eq!(cfg.playlists[2], Some(vec![Entry::None]));
        assert_eq!(cfg.warnings.len(), 2, "{:?}", cfg.warnings);
    }

    #[test]
    fn template_parses_cleanly() {
        let cfg = MusicConfig::parse(TEMPLATE, Path::new("."));
        assert!(cfg.warnings.is_empty(), "{:?}", cfg.warnings);
        assert_eq!(cfg.playlists, [None, None, None]);
    }

    #[test]
    fn drive_letters_are_paths() {
        assert!(matches!(parse_entry("C:/Music/a.mp3", Path::new(".")), Entry::Path(_)));
        assert_eq!(
            parse_entry("mw2:music_mainmenu_mp", Path::new(".")),
            Entry::Game(AssetNamespace::Iw4, "music_mainmenu_mp".into())
        );
    }

    #[test]
    fn folder_names() {
        assert_eq!(unreadable_game("Call of Duty Advanced Warfare"), Some("Advanced Warfare"));
        assert_eq!(unreadable_game("Call of Duty Ghosts"), Some("Ghosts"));
        assert_eq!(unreadable_game("Call of Duty Infinite Warfare"), Some("Infinite Warfare"));
        assert_eq!(unreadable_game("Call of Duty Black Ops II"), None);
        assert_eq!(unwired_game("Call of Duty World at War"), Some("World at War"));
        assert!(is_music_file(Path::new("x/Rising Tide.MP3")));
        assert!(!is_music_file(Path::new("x/cover.jpg")));
    }
}
