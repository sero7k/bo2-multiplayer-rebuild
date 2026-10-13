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
pub mod install;
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
            if std::env::var_os("S1_GUNS").is_some() {
                let n = |id: Option<walk::AssetId>| id.map(|i| w.cap.name_of(i).to_owned()).unwrap_or_default();
                for g in &w.cap.weapons {
                    let anims = g.anims.iter().filter(|a| a.is_some()).count();
                    println!(
                        "  gun {} [{}] gun {:?} hand {} world {:?} anims {} raise {} atts {}",
                        g.name, g.display_name,
                        g.gun_models.iter().map(|m| n(*m)).collect::<Vec<_>>(),
                        n(g.hand_model),
                        g.world_models.iter().map(|m| n(*m)).collect::<Vec<_>>(),
                        anims,
                        g.anims.get(1).map(|a| n(*a)).unwrap_or_default(),
                        g.attachments.len(),
                    );
                }
            }
            if let Ok(want) = std::env::var("S1_MAT") {
                for m in w.cap.materials.iter().filter(|m| m.name == want) {
                    let ts = m.techset.map(|t| w.cap.name_of(t).to_owned()).unwrap_or_default();
                    println!("  mat {} techset {ts}", m.name);
                    for t in &m.textures {
                        let Some(id) = t.image else { continue };
                        let i = &w.cap.images[id.index as usize];
                        println!("    tex sem {} hash {:08x} {} {}x{} fmt {} streamed {} streams {:?} files {:?}", t.semantic, t.name_hash, i.name, i.width, i.height, i.format, i.streamed, i.streams, i.stream_files);
                    }
                }
            }
            if let Ok(want) = std::env::var("S1_MODEL") {
                for m in w.cap.xmodels.iter().filter(|m| m.name.trim_start_matches(',') == want) {
                    let parts: Vec<String> = m.composite.iter().map(|p| p.map(|p| w.cap.name_of(p).to_owned()).unwrap_or_default()).collect();
                    println!("  model {} bones {} surfs {} composite {:?}", m.name, m.num_bones, m.numsurfs, parts);
                    let bones: Vec<String> = m.bone_names.iter().map(|&s| zone.assets.script_strings.get(s as usize).cloned().flatten().unwrap_or_default()).collect();
                    println!("    bones {bones:?} parents {:?} root {}", m.parent_list, m.num_root_bones);
                    println!("    tagpos {:?}", m.tag_positions.iter().take(4).collect::<Vec<_>>());
                    let Some(surfs) = m.lods.first().and_then(|l| l.surfs) else { continue };
                    let ms = &w.cap.model_surfs[surfs.index as usize];
                    let (mut n, mut sum_u, mut sum_s) = (0usize, 0f64, 0f64);
                    for s in &ms.surfs {
                        let Some(v) = s.verts else { continue };
                        for vert in v.get(&zone.image).chunks_exact(32) {
                            let p = u32::from_le_bytes(vert[24..28].try_into().unwrap());
                            let u: Vec<f64> = (0..3).map(|k| ((p >> (10 * k)) & 0x3ff) as f64 / 1023.0 * 2.0 - 1.0).collect();
                            let s: Vec<f64> = (0..3).map(|k| { let x = ((p >> (10 * k)) & 0x3ff) as i32; let x = if x > 511 { x - 1024 } else { x }; x as f64 / 511.0 }).collect();
                            sum_u += u.iter().map(|x| x * x).sum::<f64>().sqrt();
                            sum_s += s.iter().map(|x| x * x).sum::<f64>().sqrt();
                            n += 1;
                        }
                        let mut lo = [f32::MAX; 3];
                        let mut hi = [f32::MIN; 3];
                        for vert in v.get(&zone.image).chunks_exact(32) {
                            for k in 0..3 {
                                let x = f32::from_le_bytes(vert[4 * k..4 * k + 4].try_into().unwrap());
                                lo[k] = lo[k].min(x);
                                hi[k] = hi[k].max(x);
                            }
                        }
                        println!("    surf verts {} tris {} blend {:?} rigid {:?} flags {:#x} box {:?}..{:?}", s.vert_count, s.tri_count, s.blend_vert_counts, s.rigid, s.flags, lo, hi);
                    }
                    println!("    mean |n| unsigned {:.3} signed {:.3} over {n}", sum_u / n.max(1) as f64, sum_s / n.max(1) as f64);
                    let mats: Vec<String> = m.materials.iter().map(|p| p.map(|p| w.cap.name_of(p).to_owned()).unwrap_or_default()).collect();
                    println!("    materials {mats:?}");
                }
            }
            if std::env::var_os("S1_CLASSES").is_some() {
                for g in &w.cap.weapons {
                    let d = g.def.get(&zone.image);
                    let i = |o: usize| i32::from_le_bytes(d[o..o + 4].try_into().unwrap());
                    use crate::layout::WeaponDef as D;
                    println!("  class {} type {} class {} inv {} fire {} clip {} dmg {}", g.name, i(D::weapType), i(D::weapClass), i(D::inventoryType), i(D::fireType), i(D::clipSize), i(D::damage));
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
