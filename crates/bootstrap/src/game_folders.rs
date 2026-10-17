//! The first launch asks where each game is installed.
//!
//! Black Ops II is needed; Black Ops, Modern Warfare 2, Modern Warfare 3,
//! World at War, Call of Duty 4, Ghosts and Advanced Warfare add their maps
//! when present. A game added later (Ghosts, Advanced Warfare) is asked for
//! once on the next launch, keeping the folders already set. Each game
//! gets a folder picker (Cancel skips it); a folder without a `zone` folder
//! is asked for again once. The answers go to `game_paths.cfg`
//! (`asset_transport::game_paths`) and can be changed later in the front
//! end's Game Folders panel.

use std::path::PathBuf;

use asset_transport::{call_of_claude, game_paths};

/// Asks for the game folders when they were never set.
pub(crate) fn first_run() {
    if std::env::var_os("IW4L_NO_FOLDER_PROMPT").is_some() {
        return;
    }
    let asked = game_paths::asked();
    let new: Vec<_> = game_paths::GAMES.into_iter().filter(|(k, _, _)| !asked.iter().any(|a| a == k)).collect();
    if new.is_empty() {
        return;
    }
    // The very first launch: the games installed, or only the Call of
    // Claude folder (the files the rebuild reads, copied out of them).
    if asked.is_empty() {
        match ask_source() {
            Source::Pack => {
                if pick_pack() {
                    return;
                }
            }
            Source::Games => {}
        }
    }
    let mut chosen: Vec<(String, PathBuf)> = game_paths::load();
    for (key, title, needed) in new {
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

/// Where the games' files are.
enum Source {
    /// Every game installed in its own folder.
    Games,
    /// Only the Call of Claude folder.
    Pack,
}

/// Asks which: the games installed (Yes) or only the required files (No).
/// Off Windows (no box to ask in), the games.
fn ask_source() -> Source {
    let text = format!(
        "Do you have the games installed?\n\n\
         Yes: I have Black Ops II and the other Call of Duty games installed. You will be asked where each one is.\n\n\
         No: I only have the required files (the \"{}\" folder). You will be asked where it is.",
        call_of_claude::FOLDER_NAME
    );
    match ask_yes_no(&text) {
        Some(false) => Source::Pack,
        _ => Source::Games,
    }
}

/// Asks for the Call of Claude folder and points every game into it; false
/// when he cancels or it is not one (then each game is asked for).
fn pick_pack() -> bool {
    for _ in 0..2 {
        let Some(folder) = game_paths::pick_folder(&format!("Where is the {} folder?", call_of_claude::FOLDER_NAME)) else {
            return false;
        };
        match call_of_claude::adopt(&folder) {
            Ok(n) => {
                diag::info!(Zone, "game folders: {n} games from {}", folder.display());
                return true;
            }
            Err(error) => message(&error),
        }
    }
    false
}

/// A Yes / No box (Windows): `None` off Windows or when it cannot be shown.
fn ask_yes_no(text: &str) -> Option<bool> {
    #[cfg(windows)]
    {
        // (A single-quoted PowerShell string, each line break a [char]10.)
        let text = text.replace('\'', "''").replace('\n', "' + [char]10 + '");
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $r = [System.Windows.Forms.MessageBox]::Show(('{text}'), 'Call of Claude - first setup', 'YesNo', 'Question'); \
             [Console]::Out.Write($r)"
        );
        let out = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-STA", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .output()
            .ok()?;
        match String::from_utf8_lossy(&out.stdout).trim() {
            "Yes" => Some(true),
            "No" => Some(false),
            _ => None,
        }
    }
    #[cfg(not(windows))]
    {
        let _ = text;
        None
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
