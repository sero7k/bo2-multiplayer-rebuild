//! Ghosts' front-end music: the `music_*` aliases of `code_post_gfx_mp`
//! (and its patch zone, which the game loads after it and whose aliases win),
//! streamed from `soundfile1.pak` as FLAC.
//!
//! Retail `code_post_gfx_mp` holds `music_mainmenu_mp` (the MP main menu),
//! `music_mainmenu`, `music_mainmenu_extinct`, `music_mainmenu_squad`,
//! `music_mainmenu_rorke_files`, `music_credits_entire_scroll`, and two
//! (`music_challenge_factory`, `music_opening`) that share the zone's silent
//! `null` loaded sound.

use std::path::{Path, PathBuf};

use crate::pak::{self, FLAC_MAGIC};
use crate::sound::StreamedSound;
use crate::zone::Zone;

/// Read in this order; an alias in an earlier zone wins.
pub const MENU_MUSIC_ZONES: [&str; 2] = ["patch_code_post_gfx_mp", "code_post_gfx_mp"];
/// The MP main menu's track.
pub const MAIN_MENU_ALIAS: &str = "music_mainmenu_mp";
pub const LANGUAGE: &str = "english";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicTrack {
    pub alias: String,
    /// The zone the alias was taken from.
    pub zone: String,
    pub stream: StreamedSound,
    /// The pak it streams from.
    pub pak: PathBuf,
}

impl MusicTrack {
    pub fn seconds(&self) -> f64 {
        f64::from(self.stream.total_msec) / 1000.0
    }
}

/// Every streamed `music_*` alias of the menu zones, sorted by alias.
/// A missing patch zone is fine; a missing or unreadable base zone is an error.
pub fn menu_music(install: &Path) -> Result<Vec<MusicTrack>, String> {
    let zone_dir = install.join("zone");
    let mut out: Vec<MusicTrack> = Vec::new();
    for (i, name) in MENU_MUSIC_ZONES.iter().enumerate() {
        let path = zone_dir.join(format!("{name}.ff"));
        if !path.is_file() && i + 1 < MENU_MUSIC_ZONES.len() {
            continue;
        }
        let zone = match Zone::open(&path) {
            Ok(zone) => zone,
            // A patch zone that does not read leaves the base zone's aliases.
            Err(_) if i + 1 < MENU_MUSIC_ZONES.len() => continue,
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let scan = zone.sounds();
        for (list, stream) in scan.streamed() {
            let Some(alias) = list.name.text() else {
                continue;
            };
            if !alias.to_ascii_lowercase().starts_with("music_")
                || out.iter().any(|t| t.alias.eq_ignore_ascii_case(alias))
            {
                continue;
            }
            let Some(pak) = pak::sound_pak_path(&zone_dir, stream, LANGUAGE) else {
                continue;
            };
            out.push(MusicTrack {
                alias: alias.to_owned(),
                zone: (*name).to_owned(),
                stream: stream.clone(),
                pak,
            });
        }
    }
    out.sort_by(|a, b| a.alias.cmp(&b.alias));
    Ok(out)
}

/// The track's FLAC bytes, checked to start with `fLaC`.
pub fn read_track(install: &Path, track: &MusicTrack) -> Result<Vec<u8>, String> {
    let bytes = pak::read_stream(&install.join("zone"), &track.stream, LANGUAGE)
        .map_err(|e| e.to_string())?;
    if bytes.get(..4) != Some(FLAC_MAGIC.as_slice()) {
        return Err(format!(
            "{} @ {:#x} is not a FLAC stream (starts {:02x?})",
            track.pak.display(),
            track.stream.offset,
            &bytes[..bytes.len().min(4)]
        ));
    }
    Ok(bytes)
}
