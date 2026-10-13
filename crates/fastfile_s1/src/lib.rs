//! Call of Duty: Advanced Warfare (S1), PC x64 zones.
//!
//! AW's zones are Ghosts' (IW6) with an LZ4 envelope (`container`) and
//! AW's own asset types and struct layouts (`asset_type`, `layout`, from
//! x64-zt's S1 headers). The guns: weapon definitions and animations are in
//! `common_mp`, each gun's models in its own transient zones
//! (`mp_vm_<gun>_tr` first person, `mp_wm_<gun>_tr` world).
#![forbid(unsafe_code)]

pub mod asset_type;
pub mod container;
pub mod cursor;
pub mod layout;
pub mod ptr;
pub mod stream;
pub mod xfile;
pub mod zone;

pub use asset_type::AssetType;
pub use container::{
    ContainerError, FileHeader, ZONE_VERSION, inflate_zone, is_s1_zone, parse_file_header,
};
pub use ptr::Ptr;
pub use xfile::{AssetEntry, AssetList, parse_asset_list};
pub use zone::{Zone, ZoneError};

#[cfg(test)]
mod retail {
    //! Opt-in checks on real files: `S1_ZONE_DIR=<aw>/zone cargo test -p
    //! fastfile_s1 --release -- --ignored --nocapture`.

    use super::*;
    use std::path::PathBuf;

    pub(crate) fn zone_dir() -> Option<PathBuf> {
        std::env::var_os("S1_ZONE_DIR")
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
    }

    #[test]
    #[ignore]
    fn retail_zones_read() {
        let Some(dir) = zone_dir() else { return };
        let mut names: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "ff"))
            .collect();
        names.sort();
        for path in names {
            let zone = Zone::open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let unknown = zone.assets.assets.iter().filter(|a| a.asset_type().is_none()).count();
            println!(
                "{}: image {:#x}, {} strings, {} assets, {:?}",
                path.file_name().unwrap().to_string_lossy(),
                zone.image.len(),
                zone.assets.script_strings.len(),
                zone.assets.assets.len(),
                zone.assets.counts().iter().map(|(t, n)| (t.map(|t| t.name()), *n)).collect::<Vec<_>>()
            );
            assert_eq!(unknown, 0, "{}", path.display());
            assert_eq!(zone.image.len() as u64, zone.assets.xfile.size + 0x48);
        }
    }
}
