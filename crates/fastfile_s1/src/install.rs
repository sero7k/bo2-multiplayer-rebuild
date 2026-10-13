//! Where Advanced Warfare is installed: a folder with `zone/common_mp.ff`
//! in AW's format (`IW4L_AW_DIR`, a path the launcher was given, or a folder
//! under the games root whose name says AW).

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::container::is_s1_zone;

pub const PROBE_ZONE: &str = "common_mp.ff";
pub const ENV_DIR: &str = "IW4L_AW_DIR";

pub fn is_install(dir: &Path) -> bool {
    let Ok(mut f) = std::fs::File::open(dir.join("zone").join(PROBE_ZONE)) else {
        return false;
    };
    let mut head = [0u8; 12];
    f.read_exact(&mut head).is_ok() && is_s1_zone(&head)
}

pub fn name_says_aw(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "aw"
        || n == "s1"
        || n.starts_with("aw_")
        || n.contains("advanced")
        || n.contains("s1x")
        || n.starts_with("s1_")
}

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
    let mut children: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir() || t.is_symlink()))
        .map(|e| e.path())
        .collect();
    children.sort();
    children
        .into_iter()
        .filter(|d| d.file_name().is_some_and(|n| name_says_aw(&n.to_string_lossy())))
        .find(|d| is_install(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert!(name_says_aw("aw"));
        assert!(name_says_aw("aw_game_files"));
        assert!(name_says_aw("Call of Duty Advanced Warfare"));
        assert!(!name_says_aw("ghosts"));
        assert!(!name_says_aw("bo2"));
    }
}
