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
pub mod image;
pub mod layout;
pub mod ptr;
pub mod stream;
pub mod walk;
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

#[cfg(test)]
mod retail_walk {
    //! `S1_WALK=<zone.ff> cargo test -p fastfile_s1 --release walk_retail
    //! -- --ignored --nocapture`: walks a zone and checks it ends exact.

    use super::*;
    use crate::stream::VIRTUAL;

    #[test]
    #[ignore]
    fn walk_retail() {
        let Some(path) = std::env::var_os("S1_WALK") else { return };
        for path in std::env::split_paths(&path) {
            let zone = Zone::open(&path).unwrap();
            let mut w = walk::Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
            let stop = w.walk().err();
            let want = zone.assets.xfile.block_sizes[VIRTUAL as usize];
            println!(
                "{}: walked {}/{} pos {:#x}/{:#x} VIRTUAL {:#x} (block {:#x}) unresolved {} stop {:?}",
                path.display(),
                w.cap.walked,
                zone.assets.assets.len(),
                w.stream_pos(),
                zone.image.len(),
                w.virtual_offset(),
                want,
                w.unresolved.get(),
                stop
            );
            println!("  first unresolved (asset, pos, virt): {:x?}", w.first_unresolved.get());
            if let Ok(v) = std::env::var("S1_PROBE_V") {
                let v = u64::from_str_radix(v.trim_start_matches("0x"), 16).unwrap();
                println!("  probe {v:#x}: file {:x?}", w.virtual_map().file_pos(v));
                let mut near: Vec<_> = w.slot_list().into_iter().filter(|(k, _)| k.abs_diff(v) < 0x100).collect();
                near.sort_by_key(|(k, _)| *k);
                for (k, id) in near {
                    println!("    slot {k:#x} {:?} {}", id.ty, w.cap.name_of(id));
                }
            }
            if let Ok(r) = std::env::var("S1_STARTS") {
                let (a, b) = r.split_once("..").unwrap();
                let (a, b): (usize, usize) = (a.parse().unwrap(), b.parse().unwrap());
                for i in a..b.min(w.cap.starts.len()) {
                    let p = w.cap.starts[i];
                    let t = zone.assets.assets[i].asset_type().map(|t| t.name()).unwrap_or("?");
                    let head: String = zone.image[p..p + 24].iter().map(|b| format!("{b:02x}")).collect();
                    println!("  start {i} {t} {p:#x} {head}");
                }
            }
            if std::env::var_os("S1_FIRST_BAD").is_some() {
                let mut shown = 0;
                for (i, &p) in w.cap.starts.iter().enumerate() {
                    if !zone.assets.assets[i].header.is_inline() {
                        continue;
                    }
                    let ok = zone.image[p + 5..p + 8] == [0xfd, 0xfd, 0xfd];
                    if !ok {
                        let t = zone.assets.assets[i].asset_type().map(|t| t.name()).unwrap_or("?");
                        println!("  bad start {i} {t} {p:#x} (prev {:?})", zone.assets.assets[i - 1].asset_type());
                        shown += 1;
                        if shown > 3 { break; }
                    }
                }
            }
            if std::env::var_os("S1_LIST").is_some() {
                for m in &w.cap.xmodels {
                    println!("  model {} bones {} surfs {} lods {:?}", m.name, m.num_bones, m.numsurfs,
                        m.lods.iter().map(|l| l.numsurfs).collect::<Vec<_>>());
                }
                for m in &w.cap.materials {
                    println!("  material {} textures {}", m.name, m.textures.len());
                }
                for i in &w.cap.images {
                    println!("  image {} {}x{} fmt {} streamed {} len1 {:#x} inline {:?} streams {:?}", i.name, i.width, i.height, i.format, i.streamed, i.data_len1, i.pixels.map(|p| p.len), i.streams);
                }
            }
            if stop.is_none() {
                assert_eq!(w.stream_pos(), zone.image.len(), "{}", path.display());
                assert_eq!(w.virtual_offset(), want, "{}", path.display());
            }
        }
    }
}
