//! The folders of the games the player has installed, as he set them.
//!
//! `game_paths.cfg` beside the game (`IW4L_GAME_PATHS` overrides where):
//! one `key=folder` line per game, for Black Ops II (needed), Black Ops,
//! Modern Warfare 2, Modern Warfare 3, World at War, Call of Duty 4, Ghosts,
//! Advanced Warfare, Black Ops III and Modern Warfare III (2023).
//! Every folder set here is searched for zones and archives as if it were
//! under `IW4L_GAMES`, so no links or copies are needed. The first launch
//! asks for each game's folder; the front end's Game Folders panel changes
//! them later.

use std::path::{Path, PathBuf};

/// (key, title, needed) for every game the rebuild reads, in the order the
/// first launch asks: the seven games the project is built on first, then
/// the ones it borrows maps from, then Black Ops II's mods.
pub const GAMES: [(&str, &str, bool); 13] = [
    ("bo2", "Call of Duty: Black Ops II", true),
    ("bo1", "Call of Duty: Black Ops", false),
    ("mw2", "Call of Duty: Modern Warfare 2 (2009)", false),
    ("mw3", "Call of Duty: Modern Warfare 3 (2011)", false),
    // bo2mp: its maps and guest guns (`zone/core_common.ff`, `mp_common.ff`).
    ("bo3", "Call of Duty: Black Ops III", false),
    ("aw", "Call of Duty: Advanced Warfare", false),
    // bo2mp: its multiplayer maps and Rebirth Island (`zone/mp_jup_*.ff`).
    ("jup", "Call of Duty: Modern Warfare III (2023)", false),
    ("waw", "Call of Duty: World at War", false),
    ("cod4", "Call of Duty 4: Modern Warfare", false),
    ("ghosts", "Call of Duty: Ghosts", false),
    // bo2mp: its maps and Zombies (`ui.ff`, `mp_*` / `cp_*` zones).
    ("iw", "Call of Duty: Infinite Warfare", false),
    // bo2mp: Black Ops II's mods - the @leetinservers mod's folder
    // (`mod.ff`, `mod.json`; `IW4L_T6_MOD` overrides) and the DLC5 Zombies
    // pack's (`IW4L_T6_EXTRA` overrides). Not searched as game folders:
    // each is read by its own code.
    ("bo2mod", "the @leetinservers Black Ops II mod", false),
    ("bo2extra", "the Black Ops II DLC5 Zombies mod", false),
];

/// The keys whose folders are mods, read by their own code rather than
/// searched for zones with the games'.
pub const MOD_KEYS: [&str; 2] = ["bo2mod", "bo2extra"];

/// Where the folders are kept.
pub fn config_path() -> PathBuf {
    if let Some(path) = std::env::var_os("IW4L_GAME_PATHS").filter(|p| !p.is_empty()) {
        return PathBuf::from(path);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("game_paths.cfg")
}

/// Whether the folders were ever set (the first launch asks once).
pub fn configured() -> bool {
    config_path().is_file()
}

/// Every game's folder as set (games he skipped are absent).
pub fn load() -> Vec<(String, PathBuf)> {
    let Ok(text) = std::fs::read_to_string(config_path()) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (key, path) = line.split_once('=')?;
            let path = path.trim().trim_matches('"');
            (!path.is_empty()).then(|| (key.trim().to_ascii_lowercase(), PathBuf::from(path)))
        })
        .collect()
}

/// The games the file has a line for (set or skipped): a game added to
/// [`GAMES`] after the file was written has none, and is asked for once.
pub fn asked() -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(config_path()) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().to_ascii_lowercase()))
        .collect()
}

/// One game's folder, as set.
pub fn get(key: &str) -> Option<PathBuf> {
    load().into_iter().find(|(k, _)| k == key).map(|(_, p)| p)
}

/// Writes every game's folder (an empty path clears it).
pub fn save(paths: &[(String, PathBuf)]) -> Result<(), String> {
    let mut text = String::from("# Game folders (key=folder). Set in the game: Game Folders.\n");
    for (key, _, _) in GAMES {
        let path = paths.iter().find(|(k, _)| k == key).map(|(_, p)| p.display().to_string());
        text.push_str(&format!("{key}={}\n", path.unwrap_or_default()));
    }
    let path = config_path();
    std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Sets (or, with `None`, clears) one game's folder.
pub fn set(key: &str, folder: Option<PathBuf>) -> Result<(), String> {
    let mut paths = load();
    paths.retain(|(k, _)| k != key);
    if let Some(folder) = folder {
        paths.push((key.to_owned(), folder));
    }
    save(&paths)
}

/// Whether a folder looks like that game's install: it has a `zone` folder
/// (where these games keep their fast files), or fast files of its own.
pub fn looks_valid(folder: &Path) -> bool {
    folder.join("zone").is_dir()
        || std::fs::read_dir(folder).is_ok_and(|d| {
            d.flatten().any(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("ff")))
        })
}

/// The folders set that exist, for the zone search.
pub(crate) fn search_folders() -> Vec<PathBuf> {
    load()
        .into_iter()
        .filter(|(k, _)| !MOD_KEYS.contains(&k.as_str()))
        .map(|(_, p)| p)
        .filter(|p| p.is_dir())
        .collect()
}

/// A folder picker; `None` when he cancels (or off Windows).
pub fn pick_folder(prompt: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let prompt = prompt.replace('\'', "''");
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $d = New-Object System.Windows.Forms.FolderBrowserDialog; \
             $d.Description = '{prompt}'; $d.ShowNewFolderButton = $false; \
             $owner = New-Object System.Windows.Forms.Form -Property @{{TopMost = $true}}; \
             if ($d.ShowDialog($owner) -eq [System.Windows.Forms.DialogResult]::OK) {{ [Console]::Out.Write($d.SelectedPath) }}"
        );
        let out = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-STA", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .output()
            .ok()?;
        let path = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        (!path.is_empty()).then(|| PathBuf::from(path))
    }
    #[cfg(not(windows))]
    {
        let _ = prompt;
        None
    }
}

