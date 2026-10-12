//! Call of Duty: Ghosts (IW6), PC x64 zones.
//!
//! Phase 1 of reading Ghosts: open any `.ff` (`container`), read its asset
//! table (`xfile`, `asset_type`), find and read its sound alias lists
//! (`scan`, `sound`) and play their streams out of the `soundfile*.pak`
//! files (`pak`, `music`). Asset types other than `NetConstStrings` and
//! sounds are counted, not read; see `docs/IW6.md` for the format notes and
//! what reading weapons and maps needs next.
#![forbid(unsafe_code)]

pub mod asset_type;
pub mod container;
pub mod cursor;
pub mod install;
pub mod music;
pub mod ncs;
pub mod pak;
pub mod ptr;
pub mod scan;
pub mod sound;
pub mod xfile;
pub mod zone;

pub use asset_type::AssetType;
pub use container::{
    ContainerError, FileHeader, ZONE_VERSION, inflate_zone, is_iw6_zone, parse_file_header,
};
pub use music::{MAIN_MENU_ALIAS, MusicTrack, menu_music, read_track};
pub use ptr::Ptr;
pub use scan::{SoundScan, scan_sounds};
pub use sound::{
    Name, SoundAlias, SoundAliasList, SoundData, SoundFile, SoundFileRef, StreamedSound,
};
pub use xfile::{AssetEntry, AssetList, parse_asset_list};
pub use zone::{Zone, ZoneError};

#[cfg(test)]
mod retail {
    //! Opt-in checks on real files: `IW6_ZONE_DIR=<ghosts>/zone cargo test -p
    //! fastfile_iw6 --release -- --ignored --nocapture`.

    use super::*;
    use std::path::PathBuf;

    fn zone_dir() -> Option<PathBuf> {
        std::env::var_os("IW6_ZONE_DIR")
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
    }

    #[test]
    #[ignore]
    fn retail_zones_read() {
        let Some(dir) = zone_dir() else { return };
        let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "ff"))
            .collect();
        names.sort();
        for path in names {
            let zone = match Zone::open(&path) {
                Ok(z) => z,
                Err(e) => panic!("{}: {e}", path.display()),
            };
            let scan = zone.sounds();
            eprintln!(
                "{}: {} bytes, {} assets, sounds {}/{} (chained {}, anchored {}, calibrated {}, conflicts {}, unresolved names {}, files {})",
                path.display(),
                zone.image.len(),
                zone.assets.assets.len(),
                scan.lists.len(),
                scan.expected,
                scan.chained,
                scan.anchored,
                scan.calibrated,
                scan.conflicts,
                scan.unresolved_names(),
                scan.unresolved_sound_files(),
            );
            assert_eq!(scan.lists.len(), scan.expected, "{}", path.display());
            assert_eq!(scan.conflicts, 0, "{}", path.display());
        }
    }

    #[test]
    #[ignore]
    fn retail_menu_music() {
        let Some(dir) = zone_dir() else { return };
        let install = dir.parent().unwrap();
        let tracks = menu_music(install).unwrap();
        for t in &tracks {
            eprintln!(
                "{} [{}] {} @ {:#x} +{:#x} {:.1} s",
                t.alias,
                t.zone,
                t.pak.display(),
                t.stream.offset,
                t.stream.length,
                t.seconds()
            );
        }
        let main = tracks
            .iter()
            .find(|t| t.alias == MAIN_MENU_ALIAS)
            .expect("music_mainmenu_mp");
        if main.pak.is_file() {
            let bytes = read_track(install, main).unwrap();
            let info = pak::flac_info(&bytes).unwrap();
            assert!((info.seconds() - main.seconds()).abs() < 0.01, "{info:?}");
        }
    }
}
