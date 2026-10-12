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
pub mod install;
pub mod layout;
pub mod music;
pub mod ncs;
pub mod pak;
pub mod ptr;
pub mod scan;
pub mod sound;
pub mod stream;
pub mod ui_image;
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
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let t = std::time::Instant::now();
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        eprintln!(
            "opened in {:.2}s, {} assets",
            t.elapsed().as_secs_f64(),
            zone.assets.assets.len()
        );
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
        if let Ok(r) = std::env::var("IW6_STARTS")
            && let Some((a, b)) = r.split_once(',')
        {
            for i in
                a.parse::<usize>().unwrap()..b.parse::<usize>().unwrap().min(w.cap.starts.len())
            {
                eprintln!(
                    "START {i} {:?} at {:#x} -> {:?}",
                    zone.assets.assets[i].asset_type(),
                    w.cap.starts[i],
                    w.cap
                        .by_table
                        .get(&i)
                        .map(|id| w.cap.name_of(*id).to_owned())
                );
            }
        }
        if let Ok(r) = std::env::var("IW6_SAVE")
            && let Some((a, b)) = r.split_once(',')
        {
            let a = usize::from_str_radix(a.trim_start_matches("0x"), 16).unwrap();
            let b = usize::from_str_radix(b.trim_start_matches("0x"), 16).unwrap();
            std::fs::write(
                std::env::var("IW6_SAVE_TO").unwrap(),
                &zone.image[a..b.min(zone.image.len())],
            )
            .unwrap();
        }
        if let Ok(r) = std::env::var("IW6_HEX")
            && let Some((a, b)) = r.split_once(',')
        {
            let a = usize::from_str_radix(a.trim_start_matches("0x"), 16).unwrap();
            let b = usize::from_str_radix(b.trim_start_matches("0x"), 16).unwrap();
            for row in (a..b).step_by(16) {
                let bytes = &zone.image[row..(row + 16).min(zone.image.len())];
                eprintln!(
                    "HEX {row:#x} v{:x?}: {}  {}",
                    w.virtual_map().virt_of(row),
                    bytes
                        .iter()
                        .map(|x| format!("{x:02x}"))
                        .collect::<Vec<_>>()
                        .join(" "),
                    bytes
                        .iter()
                        .map(|&x| if (32..127).contains(&x) {
                            x as char
                        } else {
                            '.'
                        })
                        .collect::<String>()
                );
            }
        }
        let c = &w.cap;
        eprintln!(
            "techsets {} materials {} images {} xmodels {} surfs {} xanims {} weapons {} attachments {} fx {} sounds {} others {}",
            c.techsets.len(),
            c.materials.len(),
            c.images.len(),
            c.xmodels.len(),
            c.model_surfs.len(),
            c.xanims.len(),
            c.weapons.len(),
            c.attachments.len(),
            c.fx.len(),
            c.sounds.len(),
            c.others.len()
        );
        // The first table asset whose header does not start with its name pointer.
        let names_at_zero = [
            AssetType::Material,
            AssetType::XModel,
            AssetType::TechniqueSet,
            AssetType::XAnimParts,
            AssetType::Fx,
            AssetType::Weapon,
            AssetType::RawFile,
            AssetType::Vehicle,
            AssetType::Sound,
        ];
        for (i, &start) in w.cap.starts.iter().enumerate() {
            let a = zone.assets.assets[i];
            if !a.is_inline() || !a.asset_type().is_some_and(|t| names_at_zero.contains(&t)) {
                continue;
            }
            let p = Ptr::decode(u64::from_le_bytes(
                zone.image[start..start + 8].try_into().unwrap(),
            ));
            if !matches!(p, Ptr::Follow | Ptr::Ref { .. }) {
                eprintln!(
                    "FIRST BAD HEADER: asset {i} {:?} at {start:#x}: {p:?}",
                    a.asset_type()
                );
                for k in i.saturating_sub(6)..i {
                    eprintln!(
                        "   prev {k} {:?} {:?} at {:#x} -> {:?}",
                        zone.assets.assets[k].asset_type(),
                        zone.assets.assets[k].header,
                        w.cap.starts[k],
                        w.cap
                            .by_table
                            .get(&k)
                            .map(|id| w.cap.name_of(*id).to_owned())
                    );
                }
                for row in (start.saturating_sub(0x40)..start + 0x60).step_by(16) {
                    let b = &zone.image[row..row + 16];
                    eprintln!(
                        "{row:#x}: {}  {}",
                        b.iter()
                            .map(|x| format!("{x:02x}"))
                            .collect::<Vec<_>>()
                            .join(" "),
                        b.iter()
                            .map(|&x| if (32..127).contains(&x) {
                                x as char
                            } else {
                                '.'
                            })
                            .collect::<String>()
                    );
                }
                break;
            }
        }
        // Cross-check with the sound scan: every walked sound list where the scan found one.
        let scan = zone.sounds();
        let scanned: std::collections::HashSet<usize> =
            scan.lists.iter().map(|l| l.offset).collect();
        let walked_sounds: Vec<usize> = w.cap.sounds.iter().map(|l| l.offset).collect();
        let agree = walked_sounds.iter().filter(|o| scanned.contains(o)).count();
        eprintln!(
            "sound lists: walked {} at scan offsets {agree}; first walked not scanned: {:?}",
            walked_sounds.len(),
            walked_sounds.iter().find(|o| !scanned.contains(o))
        );
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
        eprintln!(
            "suspect string refs: {} first at {:?}",
            w.suspect.get(),
            w.first_suspect.get().map(|(i, p)| (
                i,
                zone.assets.assets.get(i).map(|a| a.asset_type()),
                format!("{p:#x}")
            ))
        );
        eprintln!(
            "unresolved asset refs: {} first at {:?}",
            w.unresolved.get(),
            w.first_unresolved.get().map(|(i, p, v)| (
                i,
                zone.assets.assets.get(i).map(|a| a.asset_type()),
                format!("{p:#x} -> {v:#x}")
            ))
        );
        eprintln!(
            "last resolved before: {:x?}; good registrations until {:#x}; bad from {:x?}",
            w.last_resolved.get(),
            w.good_until.get(),
            w.bad_from.get()
        );
        if let Some((_, _, v)) = w.first_unresolved.get() {
            eprintln!("  target {v:#x} -> file {:x?}", w.virtual_map().file_pos(v));
            if let Some(fp) = w.virtual_map().file_pos(v) {
                for row in (fp.saturating_sub(0x40)..fp + 0x40).step_by(16) {
                    let b = &zone.image[row..row + 16];
                    eprintln!(
                        "  {row:#x}: {}  {}",
                        b.iter()
                            .map(|x| format!("{x:02x}"))
                            .collect::<Vec<_>>()
                            .join(" "),
                        b.iter()
                            .map(|&x| if (32..127).contains(&x) {
                                x as char
                            } else {
                                '.'
                            })
                            .collect::<String>()
                    );
                }
            }
            let mut near: Vec<_> = w
                .slot_list()
                .into_iter()
                .filter(|(s, _)| s.abs_diff(v) < 0x400)
                .collect();
            near.sort_by_key(|(s, _)| *s);
            eprintln!(
                "  slots near: {:x?}",
                near.iter()
                    .map(|(s, id)| (s, w.cap.name_of(*id).to_owned()))
                    .collect::<Vec<_>>()
            );
        }
        eprintln!(
            "virtual anchors: {ok} ok, {} off; first off {:?}",
            bad.len(),
            bad.first()
        );
        if let Ok(want) = std::env::var("IW6_MODEL") {
            // A model's surfaces, materials, technique sets and images.
            for m in w.cap.xmodels.iter().filter(|m| m.name == want) {
                eprintln!(
                    "MODEL {} bones {} root {} surfs {} lods {:?}",
                    m.name,
                    m.num_bones,
                    m.num_root_bones,
                    m.numsurfs,
                    m.lods
                        .iter()
                        .map(|l| (
                            l.dist,
                            l.numsurfs,
                            l.surf_index,
                            l.surfs.map(|s| w.cap.name_of(s).to_owned())
                        ))
                        .collect::<Vec<_>>()
                );
                for mat in m.materials.iter().flatten() {
                    let r = &w.cap.materials[mat.index as usize];
                    let ts = r.techset.map(|t| w.cap.name_of(t).to_owned());
                    let sb: Vec<String> = r
                        .state_bits
                        .iter()
                        .map(|b| {
                            format!(
                                "{:08x}:{:08x}:{:08x}",
                                u32::from_le_bytes(b[0..4].try_into().unwrap()),
                                u32::from_le_bytes(b[4..8].try_into().unwrap()),
                                u32::from_le_bytes(b[8..12].try_into().unwrap())
                            )
                        })
                        .collect();
                    eprintln!(
                        "  MAT {} sort {} ts {ts:?} entries[2,3,7,8,9]={:?} bits {sb:?}",
                        r.name,
                        r.sort_key,
                        [2, 3, 7, 8, 9].map(|k| r.state_bits_entry[k])
                    );
                    for t in &r.textures {
                        let img = t.image.map(|i| &w.cap.images[i.index as usize]);
                        eprintln!(
                            "    TEX hash {:08x} sem {} samp {:#x} img {:?}",
                            t.name_hash,
                            t.semantic,
                            t.sampler_state,
                            img.map(|i| (
                                i.name.clone(),
                                i.format,
                                i.width,
                                i.height,
                                i.level_count,
                                i.streamed,
                                i.data_len1,
                                i.streams,
                                i.stream_files
                                    .iter()
                                    .map(|f| f.map(|f| (f.file_index, f.offset, f.offset_end)))
                                    .collect::<Vec<_>>()
                            ))
                        );
                    }
                }
            }
        }
        if std::env::var_os("IW6_ANCHORS").is_some() {
            // Every VIRTUAL reference in the walked part of the zone: does
            // its target land on a slot or on a string?
            let slots: std::collections::HashSet<u64> =
                w.slot_list().into_iter().map(|(s, _)| s).collect();
            let img = &zone.image[..w.stream_pos()];
            let range = std::env::var("IW6_ANCHOR_RANGE").ok().and_then(|r| {
                let (a, b) = r.split_once(',')?;
                Some((
                    u64::from_str_radix(a.trim_start_matches("0x"), 16).ok()?,
                    u64::from_str_radix(b.trim_start_matches("0x"), 16).ok()?,
                ))
            });
            let mut misses = Vec::new();
            let (mut n_slot, mut n_str) = (0, 0);
            for i in 4..img.len().saturating_sub(4) {
                if img[i..i + 4] != [0xf3, 0xfd, 0xfd, 0xfd] {
                    continue;
                }
                let t =
                    u32::from_le_bytes(img[i - 4..i].try_into().unwrap()).wrapping_sub(1) as u64;
                if let Some((lo, hi)) = range
                    && (lo..hi).contains(&t)
                {
                    let class = if slots.contains(&t) {
                        "slot"
                    } else if w.alloc_starts.contains(&t) {
                        "alloc"
                    } else if w.str_slots.contains_key(&t) {
                        "strslot"
                    } else {
                        "other"
                    };
                    let fp = w.virtual_map().file_pos(t);
                    let txt = fp.map(|p| {
                        String::from_utf8_lossy(&zone.image[p..(p + 24).min(zone.image.len())])
                            .replace(|c: char| !c.is_ascii_graphic(), ".")
                    });
                    eprintln!("  ref {t:#x} from {:#x} {class} -> {fp:x?} {txt:?}", i - 4);
                }
                if slots.contains(&t) || w.alloc_starts.contains(&t) || w.str_slots.contains_key(&t)
                {
                    n_slot += 1;
                    continue;
                }
                let Some(fp) = w.virtual_map().file_pos(t) else {
                    continue;
                };
                let s = &zone.image[fp..(fp + 4096).min(zone.image.len())];
                let n = s.iter().take_while(|&&b| (0x20..0x7f).contains(&b)).count();
                if s.get(n) == Some(&0) && (n > 0 || fp == 0 || zone.image[fp - 1] == 0) {
                    n_str += 1;
                    continue;
                }
                misses.push((t, i - 4));
            }
            misses.sort();
            misses.dedup_by_key(|m| m.0);
            eprintln!(
                "anchors: {n_slot} slots, {n_str} strings, {} misses",
                misses.len()
            );
            for (t, at) in misses.iter().take(40) {
                eprintln!(
                    "  miss {t:#x} from {at:#x} (maps to {:x?})",
                    w.virtual_map().file_pos(*t)
                );
            }
        }
        if std::env::var_os("IW6_LIST").is_some() {
            for wp in &w.cap.weapons {
                let gun = wp
                    .gun_models
                    .first()
                    .copied()
                    .flatten()
                    .map(|id| w.cap.name_of(id).to_owned());
                let world = wp
                    .world_models
                    .first()
                    .copied()
                    .flatten()
                    .map(|id| w.cap.name_of(id).to_owned());
                let anims = wp.anims.iter().filter(|a| a.is_some()).count();
                eprintln!(
                    "W {} [{}] gun={gun:?} world={world:?} hands={:?} anims={anims} sounds={} atts={}/{}/{}",
                    wp.name,
                    wp.display_name,
                    wp.hand_model.map(|id| w.cap.name_of(id).to_owned()),
                    wp.sounds.len(),
                    wp.scopes.iter().flatten().count(),
                    wp.under_barrels.iter().flatten().count(),
                    wp.others.iter().flatten().count()
                );
            }
            for wp in w
                .cap
                .weapons
                .iter()
                .filter(|wp| wp.name.contains("maawschild") || wp.name == "frag_grenade_mp")
            {
                let p = Ptr::decode(u64::from_le_bytes(
                    zone.image[wp.complete.pos + 16..wp.complete.pos + 24]
                        .try_into()
                        .unwrap(),
                ));
                let fp = p.virtual_offset().and_then(|v| w.virtual_map().file_pos(v));
                eprintln!(
                    "DBG {} display ptr {p:?} -> file {fp:x?} complete at {:#x}",
                    wp.name, wp.complete.pos
                );
            }
            for a in &w.cap.attachments {
                eprintln!(
                    "A {} [{}] type={} views={}",
                    a.name,
                    a.display_name,
                    a.ty,
                    a.view_models.iter().flatten().count()
                );
            }
        }
        if let Err(stop) = r {
            let types: Vec<String> = zone.assets.assets[stop.asset_index.saturating_sub(3)
                ..(stop.asset_index + 4).min(zone.assets.assets.len())]
                .iter()
                .map(|a| format!("{:?}", a.asset_type()))
                .collect();
            eprintln!("STOP {stop:?}; around: {types:?}");
            let p = stop.file_pos;
            for row in (p.saturating_sub(0x60)..p + 0x80).step_by(16) {
                let b = &zone.image[row..(row + 16).min(zone.image.len())];
                eprintln!(
                    "{row:#x}: {}",
                    b.iter()
                        .map(|x| format!("{x:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
            let last: Vec<String> = c
                .names(AssetType::Material)
                .into_iter()
                .rev()
                .take(3)
                .collect();
            eprintln!("last materials {last:?}");
        }
    }
}

#[cfg(test)]
mod gun_paks {
    //! `IW6_WALK=<common_core_mp.ff> cargo test -p fastfile_iw6 --release gun_paks -- --ignored --nocapture`:
    //! the `imagefile<N>.pak`s the `iw6_*_mp` guns' pictures stream from.
    use super::*;

    #[test]
    #[ignore]
    fn gun_paks() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        let mut models = std::collections::BTreeSet::new();
        for w in cap
            .weapons
            .iter()
            .filter(|w| w.name.starts_with("iw6_") && w.name.ends_with("_mp"))
        {
            models.extend(
                w.gun_models
                    .iter()
                    .chain(&w.world_models)
                    .flatten()
                    .map(|m| m.index),
            );
        }
        let mut per_pak: std::collections::BTreeMap<u16, (usize, u64)> = Default::default();
        let mut inline = 0;
        let mut seen = std::collections::HashSet::new();
        for &mi in &models {
            for mat in cap.xmodels[mi as usize].materials.iter().flatten() {
                for t in &cap.materials[mat.index as usize].textures {
                    let Some(img) = t.image else { continue };
                    if !seen.insert(img.index) {
                        continue;
                    }
                    let rec = &cap.images[img.index as usize];
                    if !rec.streamed {
                        inline += 1;
                        continue;
                    }
                    for f in rec.stream_files.iter().flatten() {
                        let e = per_pak.entry(f.file_index).or_default();
                        e.0 += 1;
                        e.1 += f.offset_end.saturating_sub(f.offset);
                    }
                }
            }
        }
        eprintln!(
            "{} models, {} images ({inline} inline)",
            models.len(),
            seen.len()
        );
        for (pak, (n, bytes)) in per_pak {
            eprintln!("imagefile{pak}.pak: {n} parts, {bytes} bytes");
        }
    }
}

#[cfg(test)]
mod image_dump {
    //! `IW6_WALK=<common_core_mp.ff> IW6_DUMP=<dir> cargo test -p fastfile_iw6 --release image_dump -- --ignored --nocapture`:
    //! decodes the guns' pictures that can be read (inline or from the
    //! paks beside the zone) to PNGs, checking the streamed layout.
    use super::*;

    fn decode(px: &image::ImagePixels) -> Option<Vec<u8>> {
        let (w, h) = (px.width as usize, px.height as usize);
        let mut out = vec![0u8; w * h * 4];
        let (unit, block) = image::dxgi_unit(px.dxgi_format)?;
        if !block {
            if unit != 4 {
                return None;
            }
            out.copy_from_slice(px.data.get(..w * h * 4)?);
            if matches!(px.dxgi_format, 87..=91) {
                for p in out.chunks_exact_mut(4) {
                    p.swap(0, 2);
                }
            }
            return Some(out);
        }
        let bw = w.div_ceil(4);
        let mut tmp = [0u8; 64];
        let mut tmpf = [0f32; 48];
        for (i, b) in px
            .data
            .chunks_exact(unit)
            .take(bw * h.div_ceil(4))
            .enumerate()
        {
            let (bx, by) = (i % bw * 4, i / bw * 4);
            match px.dxgi_format {
                70..=72 => bcdec_rs::bc1(b, &mut tmp, 16),
                73..=75 => bcdec_rs::bc2(b, &mut tmp, 16),
                76..=78 => bcdec_rs::bc3(b, &mut tmp, 16),
                97..=99 => bcdec_rs::bc7(b, &mut tmp, 16),
                79..=81 => {
                    let mut r = [0u8; 16];
                    bcdec_rs::bc4(b, &mut r, 4, false);
                    for k in 0..16 {
                        tmp[4 * k..4 * k + 4].copy_from_slice(&[r[k], r[k], r[k], 255]);
                    }
                }
                82..=84 => {
                    let mut r = [0u8; 32];
                    bcdec_rs::bc5(b, &mut r, 8, px.dxgi_format == 84);
                    for k in 0..16 {
                        tmp[4 * k..4 * k + 4].copy_from_slice(&[r[2 * k], r[2 * k + 1], 128, 255]);
                    }
                }
                95 | 96 => {
                    bcdec_rs::bc6h_float(b, &mut tmpf, 12, px.dxgi_format == 96);
                    for k in 0..16 {
                        for c in 0..3 {
                            tmp[4 * k + c] = (tmpf[3 * k + c].clamp(0.0, 1.0) * 255.0) as u8;
                        }
                        tmp[4 * k + 3] = 255;
                    }
                }
                _ => return None,
            }
            for y in 0..4 {
                for x in 0..4 {
                    if bx + x < w && by + y < h {
                        let o = ((by + y) * w + bx + x) * 4;
                        out[o..o + 4].copy_from_slice(&tmp[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4]);
                    }
                }
            }
        }
        Some(out)
    }

    #[test]
    #[ignore]
    fn image_dump() {
        let (Some(path), Some(dir)) = (std::env::var_os("IW6_WALK"), std::env::var_os("IW6_DUMP"))
        else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let zone = Zone::open(&path).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        let filter = std::env::var("IW6_GUN").unwrap_or_default();
        let mut paks = image::ImagePaks::new(path.parent().unwrap());
        let mut seen = std::collections::HashSet::new();
        let (mut ok, mut missing, mut bad) = (0, 0, 0);
        for w in cap
            .weapons
            .iter()
            .filter(|w| w.name.starts_with("iw6_") && w.name.contains(&filter))
        {
            for m in w.gun_models.iter().chain(&w.world_models).flatten() {
                for mat in cap.xmodels[m.index as usize].materials.iter().flatten() {
                    for t in &cap.materials[mat.index as usize].textures {
                        let Some(img) = t.image else { continue };
                        if !seen.insert(img.index) {
                            continue;
                        }
                        let rec = &cap.images[img.index as usize];
                        match image::image_pixels(rec, &zone.image, &mut paks, 512) {
                            Ok(px) => {
                                let Some(rgba) = decode(&px) else {
                                    eprintln!(
                                        "{}: format {} not decoded",
                                        rec.name, px.dxgi_format
                                    );
                                    continue;
                                };
                                ok += 1;
                                let f = std::fs::File::create(dir.join(format!(
                                    "{}.png",
                                    rec.name.replace(['/', '\\', '~', '$'], "_")
                                )))
                                .unwrap();
                                let mut e = png::Encoder::new(f, px.width, px.height);
                                e.set_color(png::ColorType::Rgba);
                                e.write_header().unwrap().write_image_data(&rgba).unwrap();
                                eprintln!(
                                    "OK {} {}x{} levels {} fmt {} streamed {} sem {} hash {:08x}",
                                    rec.name,
                                    px.width,
                                    px.height,
                                    px.levels,
                                    px.dxgi_format,
                                    rec.streamed,
                                    t.semantic,
                                    t.name_hash
                                );
                            }
                            Err(e) if e.contains("not read from") => missing += 1,
                            Err(e) => {
                                bad += 1;
                                eprintln!("BAD {}: {e} (streams {:?})", rec.name, rec.streams);
                            }
                        }
                    }
                }
            }
        }
        eprintln!("decoded {ok}, pak missing {missing}, failed {bad}");
    }
}

#[cfg(test)]
mod surface_check {
    //! `IW6_WALK=<common_core_mp.ff> cargo test -p fastfile_iw6 --release surface_check -- --ignored --nocapture`:
    //! every gun model's surfaces checked for sane vertices, indices and
    //! blend weights.
    use super::*;

    #[test]
    #[ignore]
    fn surface_check() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        let img = &zone.image;
        let mut models = std::collections::BTreeSet::new();
        for w in cap.weapons.iter().filter(|w| w.name.starts_with("iw6_")) {
            models.extend(
                w.gun_models
                    .iter()
                    .chain(&w.world_models)
                    .flatten()
                    .map(|m| m.index),
            );
        }
        let (
            mut verts,
            mut bad_len,
            mut bad_pos,
            mut bad_idx,
            mut bad_blend,
            mut no_surfs,
            mut flags8,
        ) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
        let mut max_extra = 0f32;
        for &mi in &models {
            let m = &cap.xmodels[mi as usize];
            let Some(s) = m.lods.first().and_then(|l| l.surfs) else {
                no_surfs += 1;
                continue;
            };
            let (mid, half) = m.bounds;
            for srf in &cap.model_surfs[s.index as usize].surfs {
                if srf.flags & 8 != 0 {
                    flags8 += 1;
                }
                let vb = srf.verts.map_or(&[][..], |sp| sp.get(img));
                for v in vb.chunks_exact(32) {
                    verts += 1;
                    let f = |o: usize| f32::from_le_bytes(v[o..o + 4].try_into().unwrap());
                    let p = [f(0), f(4), f(8)];
                    if (0..3).any(|k| (p[k] - mid[k]).abs() > half[k] * 1.01 + 0.5) {
                        bad_pos += 1;
                    }
                    for at in [24, 28] {
                        let u = u32::from_le_bytes(v[at..at + 4].try_into().unwrap());
                        let c: Vec<f32> = (0..3)
                            .map(|k| ((u >> (10 * k)) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0)
                            .collect();
                        let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
                        if (l - 1.0).abs() > 0.02 {
                            bad_len += 1;
                        }
                    }
                }
                let ib = srf.tris.map_or(&[][..], |sp| sp.get(img));
                bad_idx += ib
                    .chunks_exact(2)
                    .filter(|c| u16::from_le_bytes([c[0], c[1]]) >= srf.vert_count)
                    .count();
                let bw: Vec<u16> = srf
                    .blend_verts
                    .map_or(&[][..], |sp| sp.get(img))
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                let mut at = 0;
                for (bucket, &n) in srf.blend_vert_counts.iter().enumerate() {
                    for _ in 0..n {
                        let w = &bw[at..at + 1 + 2 * bucket];
                        at += 1 + 2 * bucket;
                        let bone_ok =
                            |b: u16| b % 64 == 0 && usize::from(b / 64) < usize::from(m.num_bones);
                        let extra: f32 =
                            (0..bucket).map(|k| f32::from(w[2 + 2 * k]) / 65536.0).sum();
                        max_extra = max_extra.max(extra);
                        if !bone_ok(w[0]) || (0..bucket).any(|k| !bone_ok(w[1 + 2 * k])) {
                            bad_blend += 1;
                        }
                    }
                }
                let rigid: usize = srf.rigid.iter().map(|r| usize::from(r.1)).sum();
                let blended: usize = srf.blend_vert_counts.iter().map(|&c| usize::from(c)).sum();
                if rigid + blended != usize::from(srf.vert_count) {
                    eprintln!(
                        "{}: rigid {rigid} + blended {blended} != {}",
                        m.name, srf.vert_count
                    );
                }
            }
        }
        eprintln!(
            "{} models ({no_surfs} without lod0 surfs), {verts} verts: {bad_pos} outside bounds, {bad_len} non-unit vectors, {bad_idx} bad indices, {bad_blend} bad blend verts, {flags8} motion-vertex surfaces; largest sum of extra blend weights (raw/65536) {max_extra:.4}",
            models.len()
        );
    }
}

#[cfg(test)]
mod type_runs {
    //! `IW6_WALK=<zone.ff> cargo test -p fastfile_iw6 --release type_runs -- --ignored --nocapture`:
    //! the asset table as runs of one type.
    use super::*;

    #[test]
    #[ignore]
    fn type_runs() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let mut runs: Vec<(usize, Option<AssetType>, usize)> = Vec::new();
        for (i, a) in zone.assets.assets.iter().enumerate() {
            let t = a.asset_type();
            match runs.last_mut() {
                Some(r) if r.1 == t => r.2 += 1,
                _ => runs.push((i, t, 1)),
            }
        }
        for (i, t, n) in runs {
            eprintln!("{i:5} {t:?} x{n}");
        }
    }
}

#[cfg(test)]
mod gun_sounds {
    //! `IW6_WALK=<common_core_mp.ff> IW6_GUN=ak12 cargo test -p fastfile_iw6 --release gun_sounds -- --ignored --nocapture`:
    //! a gun's sound aliases as the scan finds them.
    use super::*;

    #[test]
    #[ignore]
    fn gun_sounds() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let gun = std::env::var("IW6_GUN").unwrap_or_else(|_| "ak12".into());
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        let mut lists: Vec<sound::SoundAliasList> = cap.sounds.clone();
        for extra in std::env::var("IW6_SOUND_ZONES")
            .unwrap_or_default()
            .split(',')
            .filter(|z| !z.is_empty())
        {
            let z = Zone::open(std::path::Path::new(extra)).unwrap();
            lists.extend(walk::walk_zone(&z).0.sounds);
        }
        let mut formats: std::collections::BTreeMap<String, usize> = Default::default();
        let (mut found, mut missing) = (0, 0);
        for w in cap.weapons.iter().filter(|w| w.name.starts_with("iw6_")) {
            let verbose = w.name == format!("iw6_{gun}_mp");
            for (off, alias) in &w.sounds {
                let Some(list) = lists.iter().find(|l| l.name.text() == Some(alias.as_str()))
                else {
                    missing += 1;
                    if verbose {
                        eprintln!("  {off:#x} {alias}: NOT IN ZONE");
                    }
                    continue;
                };
                found += 1;
                for a in &list.aliases {
                    let kind = match a.sound.get().map(|f| &f.data) {
                        Some(sound::SoundData::Loaded(sound::LoadedRef::Inline(l))) => format!(
                            "loaded fmt{} {}Hz {}ch {}bit",
                            l.format, l.sample_rate, l.channels, l.bits
                        ),
                        Some(sound::SoundData::Loaded(_)) => "loaded (shared)".into(),
                        Some(sound::SoundData::Streamed(s)) => {
                            format!("streamed pak{}", s.file_index)
                        }
                        Some(sound::SoundData::Primed { stream, .. }) => {
                            format!("primed pak{}", stream.file_index)
                        }
                        Some(sound::SoundData::Unknown) => "unknown".into(),
                        None => "no file".into(),
                    };
                    *formats
                        .entry(kind.split(' ').take(2).collect::<Vec<_>>().join(" "))
                        .or_default() += 1;
                    if verbose {
                        eprintln!(
                            "  {off:#x} {alias}: {kind} sec={:?} vol {:.2}-{:.2} pitch {:.2}-{:.2} dist {:.0}-{:.0} prob {:.2}",
                            a.secondary.text(),
                            a.vol_min,
                            a.vol_max,
                            a.pitch_min,
                            a.pitch_max,
                            a.dist_min,
                            a.dist_max,
                            a.probability
                        );
                    }
                }
            }
        }
        eprintln!("gun sound lists found {found}, missing {missing}; kinds {formats:?}");
    }
}

#[cfg(test)]
mod gun_attachments {
    //! `IW6_WALK=<common_core_mp.ff> cargo test -p fastfile_iw6 --release gun_attachments -- --ignored --nocapture`:
    //! each gun's attachment lists (scopes / under-barrels / others).
    use super::*;

    #[test]
    #[ignore]
    fn gun_attachments() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        for w in cap.weapons.iter().filter(|w| {
            w.name.starts_with("iw6_") && w.gun_models.first().is_some_and(Option::is_some)
        }) {
            let names = |v: &Vec<Option<walk::AssetId>>| {
                v.iter()
                    .flatten()
                    .map(|id| cap.name_of(*id).to_owned())
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let class = w
                .def
                .map(|d| zone.image[d.pos + layout::WeaponDef::weapClass])
                .unwrap_or(255);
            let ty = w
                .def
                .map(|d| zone.image[d.pos + layout::WeaponDef::weapType])
                .unwrap_or(255);
            eprintln!(
                "GUN {} class={class} type={ty} | {} | {} | {}",
                w.name,
                names(&w.scopes),
                names(&w.under_barrels),
                names(&w.others)
            );
        }
    }
}

#[cfg(test)]
mod material_names {
    //! `IW6_WALK=<zone.ff> IW6_GREP=ak12 cargo test -p fastfile_iw6 --release material_names -- --ignored --nocapture`
    use super::*;

    #[test]
    #[ignore]
    fn material_names() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let grep = std::env::var("IW6_GREP").unwrap_or_default();
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        for m in cap.materials.iter().filter(|m| m.name.contains(&grep)) {
            let imgs: Vec<String> = m
                .textures
                .iter()
                .filter_map(|t| {
                    t.image.map(|i| {
                        let r = &cap.images[i.index as usize];
                        format!(
                            "{}({}x{} f{} s{} streamed={})",
                            r.name, r.width, r.height, r.format, t.semantic, r.streamed
                        )
                    })
                })
                .collect();
            eprintln!(
                "MAT {} ts={:?} {}",
                m.name,
                m.techset.map(|t| cap.name_of(t).to_owned()),
                imgs.join(" ")
            );
        }
    }
}

#[cfg(test)]
mod gun_notes {
    //! `IW6_WALK=<common_core_mp.ff> IW6_GUN=ak12 cargo test -p fastfile_iw6 --release gun_notes -- --ignored --nocapture`
    use super::*;

    #[test]
    #[ignore]
    fn gun_notes() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let gun = std::env::var("IW6_GUN").unwrap_or_else(|_| "ak12".into());
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        let s = |i: u32| {
            zone.assets
                .script_strings
                .get(i as usize)
                .cloned()
                .flatten()
                .unwrap_or_default()
        };
        let w = cap
            .weapons
            .iter()
            .find(|w| w.name == format!("iw6_{gun}_mp"))
            .unwrap();
        eprintln!(
            "keys {:?}",
            w.notetrack_sound_keys
                .iter()
                .zip(&w.notetrack_sound_values)
                .map(|(k, v)| (s(*k), s(*v)))
                .collect::<Vec<_>>()
        );
        for (slot, a) in w.anims.iter().enumerate() {
            let Some(a) = a else { continue };
            let x = &cap.xanims[a.index as usize];
            eprintln!(
                "ANIM {slot:#x} {} notes {:?}",
                x.name,
                x.notify
                    .iter()
                    .map(|(n, t)| format!("{}@{t:.2}", s(*n)))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[cfg(test)]
mod gun_techsets {
    //! `IW6_WALK=<common_core_mp.ff> cargo test -p fastfile_iw6 --release gun_techsets -- --ignored --nocapture`
    use super::*;

    #[test]
    #[ignore]
    fn gun_techsets() {
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        let mut seen: std::collections::BTreeMap<String, (usize, String)> = Default::default();
        for w in cap.weapons.iter().filter(|w| w.name.starts_with("iw6_")) {
            for m in w.gun_models.iter().chain(&w.world_models).flatten() {
                for mat in cap.xmodels[m.index as usize].materials.iter().flatten() {
                    let r = &cap.materials[mat.index as usize];
                    let ts = r
                        .techset
                        .map(|t| cap.name_of(t).to_owned())
                        .unwrap_or_default();
                    let e = seen.entry(ts).or_insert((0, r.name.clone()));
                    e.0 += 1;
                }
            }
        }
        for (ts, (n, ex)) in seen {
            eprintln!("TS {ts} x{n} e.g. {ex}");
        }
    }
}

#[cfg(test)]
mod gun_facts {
    //! `IW6_WALK=<common_core_mp.ff> cargo test -p fastfile_iw6 --release gun_facts -- --ignored --nocapture`
    use super::*;

    #[test]
    #[ignore]
    fn gun_facts() {
        use layout::{WeaponCompleteDef as c, WeaponDef as d};
        let Some(path) = std::env::var_os("IW6_WALK") else {
            return;
        };
        let zone = Zone::open(std::path::Path::new(&path)).unwrap();
        let (cap, _) = walk::walk_zone(&zone);
        let img = &zone.image;
        let i32_at = |p: usize| i32::from_le_bytes(img[p..p + 4].try_into().unwrap());
        let f32_at = |p: usize| f32::from_le_bytes(img[p..p + 4].try_into().unwrap());
        for name in [
            "iw6_ak12_mp",
            "iw6_kriss_mp",
            "iw6_usr_mp",
            "iw6_uts15_mp",
            "iw6_p226_mp",
            "iw6_panzerfaust3_mp",
        ] {
            let w = cap.weapons.iter().find(|w| w.name == name).unwrap();
            let (cp, dp) = (w.complete.pos, w.def.unwrap().pos);
            eprintln!(
                "{name}: clip {} fireTime {} ads {} ms fov {:.1} | class {} type {} fire {} shots {} dmg {}-{} range {:.0}-{:.0} maxAmmo {} reload {} ms empty {} ms melee {} move {:.2} hipMin {:.1}",
                i32_at(cp + c::iClipSize),
                i32_at(cp + c::iFireTime),
                i32_at(cp + c::iAdsTransInTime),
                f32_at(cp + c::fAdsZoomFov),
                i32_at(dp + d::weapClass),
                i32_at(dp + d::weapType),
                i32_at(dp + d::fireType),
                i32_at(dp + d::shotCount),
                i32_at(dp + d::damage),
                i32_at(dp + d::minDamage),
                f32_at(dp + d::fMaxDamageRange),
                f32_at(dp + d::fMinDamageRange),
                i32_at(dp + d::iMaxAmmo),
                i32_at(dp + d::stateTimers__iReloadTime),
                i32_at(dp + d::stateTimers__iReloadEmptyTime),
                i32_at(dp + d::iMeleeDamage),
                f32_at(dp + d::moveSpeedScale),
                f32_at(dp + d::fHipSpreadStandMin),
            );
        }
    }
}
