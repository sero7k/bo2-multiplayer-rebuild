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
pub mod image;
pub mod layout;
pub mod install;
pub mod music;
pub mod ncs;
pub mod pak;
pub mod ptr;
pub mod scan;
pub mod sound;
pub mod stream;
pub mod walk;
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

#[cfg(test)]
mod walk_retail {
    //! `IW6_WALK=<zone.ff> cargo test -p fastfile_iw6 --release walk_retail -- --ignored --nocapture`
    use super::*;

    #[test]
    #[ignore]
    fn walk_retail() {
        let Some(path) = std::env::var_os("IW6_WALK") else { return };
        let t = std::time::Instant::now();
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        eprintln!("opened in {:.2}s, {} assets", t.elapsed().as_secs_f64(), zone.assets.assets.len());
        let mut w = walk::Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
        let r = w.walk();
        eprintln!(
            "walked {} of {} in {:.2}s; pos {:#x} of {:#x}; virtual {:#x} (block size {:#x}); vmap segments {}",
            w.cap.walked,
            zone.assets.assets.len(),
            t.elapsed().as_secs_f64(),
            w.stream_pos(),
            zone.image.len(),
            w.virtual_offset(),
            zone.assets.xfile.block_sizes[3],
            w.virtual_map().segments()
        );
        let c = &w.cap;
        eprintln!(
            "techsets {} materials {} images {} xmodels {} surfs {} xanims {} weapons {} attachments {} fx {} sounds {} others {}",
            c.techsets.len(), c.materials.len(), c.images.len(), c.xmodels.len(), c.model_surfs.len(), c.xanims.len(),
            c.weapons.len(), c.attachments.len(), c.fx.len(), c.sounds.len(), c.others.len()
        );
        // The first table asset whose header does not start with its name pointer.
        let names_at_zero = [AssetType::Material, AssetType::XModel, AssetType::TechniqueSet, AssetType::XAnimParts, AssetType::Fx, AssetType::Weapon, AssetType::RawFile, AssetType::Vehicle, AssetType::Sound];
        for (i, &start) in w.cap.starts.iter().enumerate() {
            let a = zone.assets.assets[i];
            if !a.is_inline() || !a.asset_type().is_some_and(|t| names_at_zero.contains(&t)) {
                continue;
            }
            let p = Ptr::decode(u64::from_le_bytes(zone.image[start..start + 8].try_into().unwrap()));
            if !matches!(p, Ptr::Follow | Ptr::Ref { .. }) {
                eprintln!("FIRST BAD HEADER: asset {i} {:?} at {start:#x}: {p:?}", a.asset_type());
                for k in i.saturating_sub(6)..i {
                    eprintln!("   prev {k} {:?} {:?} at {:#x} -> {:?}", zone.assets.assets[k].asset_type(), zone.assets.assets[k].header, w.cap.starts[k], w.cap.by_table.get(&k).map(|id| w.cap.name_of(*id).to_owned()));
                }
                for row in (start.saturating_sub(0x40)..start + 0x60).step_by(16) {
                    let b = &zone.image[row..row + 16];
                    eprintln!("{row:#x}: {}  {}", b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" "), b.iter().map(|&x| if (32..127).contains(&x) { x as char } else { '.' }).collect::<String>());
                }
                break;
            }
        }
        // Cross-check with the sound scan: every walked sound list where the scan found one.
        let scan = zone.sounds();
        let scanned: std::collections::HashSet<usize> = scan.lists.iter().map(|l| l.offset).collect();
        let walked_sounds: Vec<usize> = w.cap.sounds.iter().map(|l| l.offset).collect();
        let agree = walked_sounds.iter().filter(|o| scanned.contains(o)).count();
        eprintln!("sound lists: walked {} at scan offsets {agree}; first walked not scanned: {:?}", walked_sounds.len(), walked_sounds.iter().find(|o| !scanned.contains(o)));
        // VIRTUAL check: an inline-named sound list's aliases point at its name.
        let mut ok = 0;
        let mut bad = Vec::new();
        for l in &w.cap.sounds {
            if let (sound::Name::Inline(_), Some(a)) = (&l.name, l.aliases.first())
                && let sound::Name::Ref { virt, .. } = a.name
            {
                let want = l.offset + 24;
                match w.virtual_map().file_pos(virt) {
                    Some(p) if p == want => ok += 1,
                    got => bad.push((l.name(), virt, got.map(|g| g as i64 - want as i64))),
                }
            }
        }
        eprintln!("suspect string refs: {} first at {:?}", w.suspect.get(), w.first_suspect.get().map(|(i, p)| (i, zone.assets.assets.get(i).map(|a| a.asset_type()), format!("{p:#x}"))));
        eprintln!("unresolved asset refs: {} first at {:?}", w.unresolved.get(), w.first_unresolved.get().map(|(i, p, v)| (i, zone.assets.assets.get(i).map(|a| a.asset_type()), format!("{p:#x} -> {v:#x}"))));
        eprintln!("last resolved before: {:x?}; good registrations until {:#x}; bad from {:x?}", w.last_resolved.get(), w.good_until.get(), w.bad_from.get());
        if let Some((_, _, v)) = w.first_unresolved.get() {
            eprintln!("  target {v:#x} -> file {:x?}", w.virtual_map().file_pos(v));
            if let Some(fp) = w.virtual_map().file_pos(v) {
                for row in (fp.saturating_sub(0x40)..fp + 0x40).step_by(16) {
                    let b = &zone.image[row..row + 16];
                    eprintln!("  {row:#x}: {}  {}", b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" "), b.iter().map(|&x| if (32..127).contains(&x) { x as char } else { '.' }).collect::<String>());
                }
            }
            let mut near: Vec<_> = w.slot_list().into_iter().filter(|(s, _)| s.abs_diff(v) < 0x400).collect();
            near.sort_by_key(|(s, _)| *s);
            eprintln!("  slots near: {:x?}", near.iter().map(|(s, id)| (s, w.cap.name_of(*id).to_owned())).collect::<Vec<_>>());
        }
        eprintln!("virtual anchors: {ok} ok, {} off; first off {:?}", bad.len(), bad.first());
        if let Ok(want) = std::env::var("IW6_MODEL") {
            // A model's surfaces, materials, technique sets and images.
            for m in w.cap.xmodels.iter().filter(|m| m.name == want) {
                eprintln!("MODEL {} bones {} root {} surfs {} lods {:?}", m.name, m.num_bones, m.num_root_bones, m.numsurfs, m.lods.iter().map(|l| (l.dist, l.numsurfs, l.surf_index, l.surfs.map(|s| w.cap.name_of(s).to_owned()))).collect::<Vec<_>>());
                for mat in m.materials.iter().flatten() {
                    let r = &w.cap.materials[mat.index as usize];
                    let ts = r.techset.map(|t| w.cap.name_of(t).to_owned());
                    let sb: Vec<String> = r.state_bits.iter().map(|b| format!("{:08x}:{:08x}:{:08x}", u32::from_le_bytes(b[0..4].try_into().unwrap()), u32::from_le_bytes(b[4..8].try_into().unwrap()), u32::from_le_bytes(b[8..12].try_into().unwrap()))).collect();
                    eprintln!("  MAT {} sort {} ts {ts:?} entries[2,3,7,8,9]={:?} bits {sb:?}", r.name, r.sort_key, [2, 3, 7, 8, 9].map(|k| r.state_bits_entry[k]));
                    for t in &r.textures {
                        let img = t.image.map(|i| &w.cap.images[i.index as usize]);
                        eprintln!("    TEX hash {:08x} sem {} samp {:#x} img {:?}", t.name_hash, t.semantic, t.sampler_state, img.map(|i| (i.name.clone(), i.format, i.width, i.height, i.level_count, i.streamed, i.data_len1, i.streams, i.stream_files.iter().map(|f| f.map(|f| (f.file_index, f.offset, f.offset_end))).collect::<Vec<_>>())));
                    }
                }
            }
        }
        if std::env::var_os("IW6_ANCHORS").is_some() {
            // Every VIRTUAL reference in the walked part of the zone: does
            // its target land on a slot or on a string?
            let slots: std::collections::HashSet<u64> = w.slot_list().into_iter().map(|(s, _)| s).collect();
            let img = &zone.image[..w.stream_pos()];
            let range = std::env::var("IW6_ANCHOR_RANGE").ok().and_then(|r| {
                let (a, b) = r.split_once(',')?;
                Some((u64::from_str_radix(a.trim_start_matches("0x"), 16).ok()?, u64::from_str_radix(b.trim_start_matches("0x"), 16).ok()?))
            });
            let mut misses = Vec::new();
            let (mut n_slot, mut n_str) = (0, 0);
            for i in 4..img.len().saturating_sub(4) {
                if img[i..i + 4] != [0xf3, 0xfd, 0xfd, 0xfd] {
                    continue;
                }
                let t = u32::from_le_bytes(img[i - 4..i].try_into().unwrap()).wrapping_sub(1) as u64;
                if let Some((lo, hi)) = range
                    && (lo..hi).contains(&t)
                {
                    let class = if slots.contains(&t) { "slot" } else if w.alloc_starts.contains(&t) { "alloc" } else if w.str_slots.contains_key(&t) { "strslot" } else { "other" };
                    let fp = w.virtual_map().file_pos(t);
                    let txt = fp.map(|p| String::from_utf8_lossy(&zone.image[p..(p + 24).min(zone.image.len())]).replace(|c: char| !c.is_ascii_graphic(), "."));
                    eprintln!("  ref {t:#x} from {:#x} {class} -> {fp:x?} {txt:?}", i - 4);
                }
                if slots.contains(&t) || w.alloc_starts.contains(&t) || w.str_slots.contains_key(&t) {
                    n_slot += 1;
                    continue;
                }
                let Some(fp) = w.virtual_map().file_pos(t) else { continue };
                let s = &zone.image[fp..(fp + 64).min(zone.image.len())];
                let n = s.iter().take_while(|&&b| (0x20..0x7f).contains(&b)).count();
                if (n >= 2 || n == 0) && s.get(n) == Some(&0) {
                    n_str += 1;
                    continue;
                }
                misses.push((t, i - 4));
            }
            misses.sort();
            misses.dedup_by_key(|m| m.0);
            eprintln!("anchors: {n_slot} slots, {n_str} strings, {} misses", misses.len());
            for (t, at) in misses.iter().take(40) {
                eprintln!("  miss {t:#x} from {at:#x} (maps to {:x?})", w.virtual_map().file_pos(*t));
            }
        }
        if std::env::var_os("IW6_LIST").is_some() {
            for wp in &w.cap.weapons {
                let gun = wp.gun_models.first().copied().flatten().map(|id| w.cap.name_of(id).to_owned());
                let world = wp.world_models.first().copied().flatten().map(|id| w.cap.name_of(id).to_owned());
                let anims = wp.anims.iter().filter(|a| a.is_some()).count();
                eprintln!("W {} [{}] gun={gun:?} world={world:?} hands={:?} anims={anims} sounds={} atts={}/{}/{}", wp.name, wp.display_name,
                    wp.hand_model.map(|id| w.cap.name_of(id).to_owned()), wp.sounds.len(),
                    wp.scopes.iter().flatten().count(), wp.under_barrels.iter().flatten().count(), wp.others.iter().flatten().count());
            }
            for wp in w.cap.weapons.iter().filter(|wp| wp.name.contains("maawschild") || wp.name == "frag_grenade_mp") {
                let p = Ptr::decode(u64::from_le_bytes(zone.image[wp.complete.pos + 16..wp.complete.pos + 24].try_into().unwrap()));
                let fp = p.virtual_offset().and_then(|v| w.virtual_map().file_pos(v));
                eprintln!("DBG {} display ptr {p:?} -> file {fp:x?} complete at {:#x}", wp.name, wp.complete.pos);
            }
            for a in &w.cap.attachments { eprintln!("A {} [{}] type={} views={}", a.name, a.display_name, a.ty, a.view_models.iter().flatten().count()); }
        }
        if let Err(stop) = r {
            let types: Vec<String> = zone.assets.assets[stop.asset_index.saturating_sub(3)..(stop.asset_index + 4).min(zone.assets.assets.len())]
                .iter()
                .map(|a| format!("{:?}", a.asset_type()))
                .collect();
            eprintln!("STOP {stop:?}; around: {types:?}");
            let p = stop.file_pos;
            for row in (p.saturating_sub(0x60)..p + 0x80).step_by(16) {
                let b = &zone.image[row..(row + 16).min(zone.image.len())];
                eprintln!("{row:#x}: {}", b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" "));
            }
            let last: Vec<String> = c.names(AssetType::Material).into_iter().rev().take(3).collect();
            eprintln!("last materials {last:?}");
        }
    }
}
