//! Ghosts (IW6) menu music for the director: the `music_*` aliases of
//! Ghosts' `code_post_gfx_mp` (and its patch zone) streamed as FLAC out of
//! `zone/soundfile1.pak`, read with `fastfile_iw6` and played like a user's
//! FLAC file (`StreamedMusic`).
//!
//! Ghosts stays outside `AssetNamespace` on purpose: that enum is matched
//! all over the asset pipeline (sound banks, zone lanes, guest-gun prefixes)
//! and Ghosts contributes nothing to those yet. Its music needs only an
//! install folder, two zones and a pak, so it has this path of its own; a
//! namespace comes with Ghosts' guns and maps.
//!
//! The install is found by `fastfile_iw6::install::find_install`
//! (`IW4L_GHOSTS_DIR`, a `ghosts=` line in game_paths.cfg, `<games>/ghosts`,
//! or a folder under the games root named for Ghosts). No install: no
//! Ghosts tracks, nothing read.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use fastfile_iw6::music::{MusicTrack, menu_music, read_track};

use super::stream::{Probe, probe};

pub const GAME: &str = "Ghosts";
/// Ghosts' tracks in the default menu playlist.
pub const DEFAULT_TRACKS: [&str; 1] = [fastfile_iw6::MAIN_MENU_ALIAS];

/// The Ghosts install, if there is one.
pub fn install(games_root: Option<&Path>) -> Option<PathBuf> {
    let extra: Vec<PathBuf> = asset_transport::game_paths::get("ghosts")
        .into_iter()
        .collect();
    fastfile_iw6::install::find_install(games_root.filter(|p| !p.as_os_str().is_empty()), &extra)
}

type TrackCache = Mutex<HashMap<PathBuf, Arc<Result<Vec<MusicTrack>, String>>>>;

/// The install's menu-music aliases (its zones are read once per run).
pub fn tracks(install: &Path) -> Arc<Result<Vec<MusicTrack>, String>> {
    static CACHE: OnceLock<TrackCache> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(hit) = cache.lock().ok().and_then(|c| c.get(install).cloned()) {
        return hit;
    }
    let started = std::time::Instant::now();
    let read = Arc::new(menu_music(install));
    match read.as_ref() {
        Ok(list) => diag::info!(
            Audio,
            "audio: music: Ghosts at {}: {} menu tracks ({}) [{:.2} s]",
            install.display(),
            list.len(),
            list.iter()
                .map(|t| t.alias.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            started.elapsed().as_secs_f64()
        ),
        Err(error) => diag::warn!(
            Audio,
            "audio: music: Ghosts at {}: {error}",
            install.display()
        ),
    }
    if let Ok(mut c) = cache.lock() {
        c.insert(install.to_path_buf(), Arc::clone(&read));
    }
    read
}

pub fn find_track(install: &Path, alias: &str) -> Result<MusicTrack, String> {
    let all = tracks(install);
    let list = all.as_ref().as_ref().map_err(Clone::clone)?;
    list.iter()
        .find(|t| t.alias.eq_ignore_ascii_case(alias))
        .cloned()
        .ok_or_else(|| format!("no streamed alias `{alias}` in Ghosts' menu zones"))
}

/// A track's FLAC bytes from its pak, probed (format, layout, length).
pub fn read_and_probe(install: Option<&Path>, alias: &str) -> Result<(Arc<[u8]>, Probe), String> {
    let install = install.ok_or_else(|| "Ghosts is not installed".to_owned())?;
    let track = find_track(install, alias)?;
    let bytes: Arc<[u8]> = read_track(install, &track)?.into();
    let probed = probe(Arc::clone(&bytes), Some("flac"))?;
    Ok((bytes, probed))
}

/// Where a track streams from, for the manifest.
pub fn origin(track: &MusicTrack) -> String {
    format!(
        "{}: {} @ {:#x} (+{:#x})",
        track.zone,
        track
            .pak
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
        track.stream.offset,
        track.stream.length
    )
}

/// `iw4l check-ghosts [zone]`: the install, a zone's asset counts and sound
/// scan, and every menu-music alias with its stream checked (`fLaC` at the
/// offset, STREAMINFO length against the alias's, a full symphonia probe).
pub fn check_report(games_root: Option<&Path>, zone: Option<&str>) -> Vec<String> {
    use fastfile_iw6::{Zone, pak};
    let mut out = Vec::new();
    let Some(install) = install(games_root) else {
        out.push(format!(
            "Ghosts: not found (set {} or link <games>/ghosts to the install; games root {})",
            fastfile_iw6::install::ENV_DIR,
            games_root.map_or_else(|| "unset".to_owned(), |p| p.display().to_string())
        ));
        return out;
    };
    out.push(format!("Ghosts install: {}", install.display()));
    let zone_name = zone.unwrap_or("code_post_gfx_mp").trim_end_matches(".ff");
    let path = install.join("zone").join(format!("{zone_name}.ff"));
    let started = std::time::Instant::now();
    match Zone::open(&path) {
        Err(error) => out.push(format!("zone {}: {error}", path.display())),
        Ok(z) => {
            out.push(format!(
                "zone {}: version {:#x}, {} streamed images, {} bytes inflated, {} script strings, {} assets [{:.2} s]",
                zone_name,
                z.header.version,
                z.header.stream_files.len(),
                z.image.len(),
                z.assets.script_strings.len(),
                z.assets.assets.len(),
                started.elapsed().as_secs_f64()
            ));
            for (ty, n) in z.assets.counts() {
                out.push(format!(
                    "  {:<22} {n}",
                    ty.map_or("(unknown type)", |t| t.name())
                ));
            }
            let scan = z.sounds();
            out.push(format!(
                "  sounds: {} of {} lists found ({} adjacent, {} anchored, {} calibrated, {} conflicts), {} names and {} shared sound files unresolved, {} net const string lists",
                scan.lists.len(),
                scan.expected,
                scan.chained,
                scan.anchored,
                scan.calibrated,
                scan.conflicts,
                scan.unresolved_names(),
                scan.unresolved_sound_files(),
                scan.net_const_strings.len()
            ));
            for l in scan
                .lists
                .iter()
                .filter(|l| l.name.text().is_none())
                .take(5)
            {
                out.push(format!(
                    "    unresolved list name at {:#x}: {}",
                    l.offset,
                    l.name()
                ));
            }
            let streamed = scan.streamed().count();
            out.push(format!("  streamed aliases: {streamed}"));
            for (list, s) in scan
                .streamed()
                .filter(|(l, _)| l.name().starts_with("music_"))
            {
                out.push(format!(
                    "    {} -> {}soundfile{}.pak @ {:#x} +{:#x}, {:.3} s",
                    list.name(),
                    if s.localized { "<lang>_" } else { "" },
                    s.file_index,
                    s.offset,
                    s.length,
                    f64::from(s.total_msec) / 1000.0
                ));
            }
        }
    }
    let all = tracks(&install);
    match all.as_ref() {
        Err(error) => out.push(format!("menu music: {error}")),
        Ok(list) => {
            out.push(format!("menu music: {} tracks", list.len()));
            for t in list {
                let head = pak::peek_stream(
                    &install.join("zone"),
                    &t.stream,
                    fastfile_iw6::music::LANGUAGE,
                    64,
                );
                let info = head.as_ref().ok().and_then(|h| pak::flac_info(h));
                let verdict = match (&head, info) {
                    (Err(e), _) => format!("pak not readable ({e})"),
                    (Ok(h), None) => format!("NOT FLAC (starts {:02x?})", &h[..h.len().min(4)]),
                    (Ok(_), Some(i)) => format!(
                        "fLaC ok, {} Hz {} ch {}-bit, {:.3} s (alias says {:.3} s){}",
                        i.sample_rate,
                        i.channels,
                        i.bits_per_sample,
                        i.seconds(),
                        t.seconds(),
                        if (i.seconds() - t.seconds()).abs() < 0.01 {
                            ""
                        } else {
                            " MISMATCH"
                        }
                    ),
                };
                let decode = match read_and_probe(Some(&install), &t.alias) {
                    Ok((bytes, p)) => format!(
                        "decodes: {} {} ch {} Hz, {:.3} s, {} bytes",
                        p.codec,
                        p.channels,
                        p.rate,
                        p.length.unwrap_or(0.0),
                        bytes.len()
                    ),
                    Err(e) => format!("does not decode: {e}"),
                };
                out.push(format!(
                    "  {:<28} {} — {verdict}; {decode}",
                    t.alias,
                    origin(t)
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::StreamedMusic;
    use bevy::audio::Decodable;

    /// `IW4L_GHOSTS_DIR=<ghosts> cargo test -p audio ghosts_menu_music -- --ignored --nocapture`:
    /// every menu track found, read from its pak and fully decoded.
    #[test]
    #[ignore]
    fn ghosts_menu_music_decodes() {
        let Some(dir) = install(None) else {
            eprintln!("no Ghosts install (set {})", fastfile_iw6::install::ENV_DIR);
            return;
        };
        for line in check_report(None, None) {
            eprintln!("{line}");
        }
        let (bytes, p) = read_and_probe(Some(&dir), fastfile_iw6::MAIN_MENU_ALIAS).unwrap();
        let music = StreamedMusic::new(bytes, Some("flac".into()), &p);
        let samples = music.decoder().count();
        let secs = samples as f64 / f64::from(p.channels) / f64::from(p.rate);
        eprintln!("{p:?}; streamed {secs:.3} s");
        assert!((secs - p.length.unwrap()).abs() < 0.05);
        assert!((secs - 131.095).abs() < 0.05);
    }
}
