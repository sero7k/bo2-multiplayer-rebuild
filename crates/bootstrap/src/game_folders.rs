//! The first launch asks where each game is installed.
//!
//! Black Ops II is needed; Black Ops, Modern Warfare 2, Modern Warfare 3,
//! World at War and Call of Duty 4 add their maps when present. Each game
//! gets a folder picker (Cancel skips it); a folder without a `zone` folder
//! is asked for again once. The answers go to `game_paths.cfg`
//! (`asset_transport::game_paths`) and can be changed later in the front
//! end's Game Folders panel.

use std::path::PathBuf;

use asset_transport::game_paths;

/// Asks for the game folders when they were never set.
pub(crate) fn first_run() {
    if game_paths::configured() || std::env::var_os("IW4L_NO_FOLDER_PROMPT").is_some() {
        return;
    }
    let mut chosen: Vec<(String, PathBuf)> = Vec::new();
    for (key, title, needed) in game_paths::GAMES {
        let note = if needed { "needed" } else { "optional: Cancel to skip" };
        let mut tries = 0;
        while tries < 2 {
            tries += 1;
            let Some(folder) = game_paths::pick_folder(&format!("Where is {title} installed? ({note})")) else {
                break;
            };
            if game_paths::looks_valid(&folder) {
                chosen.push((key.to_owned(), folder));
                break;
            }
            message(&format!(
                "{} does not look like {title}'s folder (it has no `zone` folder). Pick the folder the game's .exe is in.",
                folder.display()
            ));
        }
    }
    match game_paths::save(&chosen) {
        Ok(()) => diag::info!(Zone, "game folders: {} set", chosen.len()),
        Err(error) => diag::warn!(Zone, "game folders: {error}"),
    }
}

/// A plain message box (Windows); the log elsewhere.
fn message(text: &str) {
    diag::warn!(Zone, "game folders: {text}");
    #[cfg(windows)]
    {
        let text = text.replace('\'', "''");
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('{text}', 'Game folders') | Out-Null"
        );
        let _ = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-STA", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .status();
    }
}
