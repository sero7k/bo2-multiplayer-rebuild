//! "Call of Claude": one folder holding only the files the rebuild reads
//! from every game, made by the `call_of_claude` tool from the installed
//! games (`call_of_claude pack <folder>`).
//!
//! Inside it each game has its own folder named for its key in
//! [`crate::game_paths::GAMES`] (`bo2`, `bo3`, `jup`, ...), laid out as the
//! game's install is, so the zone search reads it exactly as it reads the
//! game. A marker file at its top says it is one. The first launch asks
//! whether the games are installed or only this folder is; for the folder
//! every game's line in `game_paths.cfg` points into it.

use std::path::{Path, PathBuf};

/// The marker file at the folder's top.
pub const MARKER: &str = "call_of_claude.txt";

/// The folder's usual name.
pub const FOLDER_NAME: &str = "Call of Claude";

/// Whether `root` is a Call of Claude folder.
#[must_use]
pub fn is_pack(root: &Path) -> bool {
    root.join(MARKER).is_file()
}

/// The games the folder holds: (key, its folder).
#[must_use]
pub fn games_in(root: &Path) -> Vec<(String, PathBuf)> {
    crate::game_paths::GAMES
        .iter()
        .map(|(key, _, _)| ((*key).to_owned(), root.join(key)))
        .filter(|(_, dir)| dir.is_dir())
        .collect()
}

/// Points every game's line in `game_paths.cfg` into the folder (a game it
/// does not hold is left unset). Returns how many games it holds.
pub fn adopt(root: &Path) -> Result<usize, String> {
    if !is_pack(root) {
        return Err(format!("{} is not a {FOLDER_NAME} folder (it has no {MARKER})", root.display()));
    }
    let games = games_in(root);
    if !games.iter().any(|(k, _)| k == "bo2") {
        return Err(format!("{} has no Black Ops II folder (bo2), which the game needs", root.display()));
    }
    crate::game_paths::save(&games)?;
    Ok(games.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pack_is_known_by_its_marker_and_needs_bo2() {
        let dir = std::env::temp_dir().join(format!("coc-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("bo3")).unwrap();
        assert!(!is_pack(&dir));
        std::fs::write(dir.join(MARKER), "x").unwrap();
        assert!(is_pack(&dir));
        assert_eq!(games_in(&dir).len(), 1);
        // (No bo2: refused before anything is written.)
        assert!(adopt(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
