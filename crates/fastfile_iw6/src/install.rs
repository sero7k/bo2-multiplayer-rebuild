//! Where Ghosts is installed: a folder with `zone/code_post_gfx_mp.ff` that
//! is an IW6 zone. Looked for, in order, at `IW4L_GHOSTS_DIR`, the folders
//! the caller passes (a `ghosts` entry of game_paths.cfg), `<games root>/ghosts`,
//! and any folder under the games root (or one level further down) whose
//! name says Ghosts (`ghosts_game_files`, `Call of Duty Ghosts`, `iw6`).
//! Only directory listings and one 12-byte header read are done: the
//! image and sound paks (100+ GB) are never opened here.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::container::is_iw6_zone;

/// The zone every Ghosts MP install has (and that holds the menu music).
pub const PROBE_ZONE: &str = "code_post_gfx_mp.ff";
pub const ENV_DIR: &str = "IW4L_GHOSTS_DIR";

/// Is `dir` a Ghosts install (its `zone/code_post_gfx_mp.ff` an IW6 zone)?
pub fn is_install(dir: &Path) -> bool {
    let probe = dir.join("zone").join(PROBE_ZONE);
    let Ok(mut f) = std::fs::File::open(probe) else {
        return false;
    };
    let mut head = [0u8; 12];
    f.read_exact(&mut head).is_ok() && is_iw6_zone(&head)
}

/// A folder name that says Ghosts.
pub fn name_says_ghosts(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("ghosts")
        || n == "iw6"
        || n.contains("iw6x")
        || n.starts_with("iw6_")
        || n.ends_with("_iw6")
}

/// Finds the install (see the module docs).
pub fn find_install(games_root: Option<&Path>, extra: &[PathBuf]) -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(ENV_DIR)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        && is_install(&dir)
    {
        return Some(dir);
    }
    if let Some(dir) = extra.iter().find(|d| is_install(d)) {
        return Some(dir.clone());
    }
    let root = games_root?;
    if is_install(root) {
        return Some(root.to_path_buf());
    }
    let named = |dir: &Path| -> Vec<PathBuf> {
        let Ok(read) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut out: Vec<PathBuf> = read
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir() || t.is_symlink()))
            .map(|e| e.path())
            .collect();
        out.sort();
        out
    };
    let children = named(root);
    if let Some(dir) = children
        .iter()
        .filter(|d| {
            d.file_name()
                .is_some_and(|n| name_says_ghosts(&n.to_string_lossy()))
        })
        .find(|d| is_install(d))
    {
        return Some(dir.clone());
    }
    // One level down (`<root>/<store>/<game>`), named folders only.
    children
        .iter()
        .filter(|d| {
            !d.file_name()
                .is_some_and(|n| name_says_ghosts(&n.to_string_lossy()))
        })
        .flat_map(|d| named(d))
        .filter(|d| {
            d.file_name()
                .is_some_and(|n| name_says_ghosts(&n.to_string_lossy()))
        })
        .find(|d| is_install(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_named_install() {
        let root = std::env::temp_dir().join(format!("iw6install{}", std::process::id()));
        let game = root.join("Steam").join("Call of Duty Ghosts");
        std::fs::create_dir_all(game.join("zone")).unwrap();
        std::fs::create_dir_all(root.join("bo2").join("zone")).unwrap();
        assert!(find_install(Some(&root), &[]).is_none());
        let mut head = b"IWff0100".to_vec();
        head.extend_from_slice(&0x235u32.to_le_bytes());
        std::fs::write(game.join("zone").join(PROBE_ZONE), &head).unwrap();
        assert_eq!(find_install(Some(&root), &[]), Some(game.clone()));
        assert!(name_says_ghosts("ghosts_game_files"));
        assert!(!name_says_ghosts("bo2_game_files"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
