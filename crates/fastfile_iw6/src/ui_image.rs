//! Ghosts' 2-D pictures (Create-a-Class weapon and attachment pictures,
//! kill icons) by material name, as RGBA, from `zone/ui_mp.ff` (and
//! `common_core_mp.ff` for what ui_mp lacks).
//!
//! The first call walks the zones once and keeps only the top level of
//! the inline pictures of 2-D materials whose names start with one of
//! `KEPT` (a few MB); later calls decode from that.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::image::{dxgi_unit, level_bytes};

/// Material name prefixes kept.
const KEPT: &[&str] = &["weapon_", "hud_icon_", "scope_", "mod_", "hud_"];

/// The attachment pictures (`<prefix>_360`) besides `KEPT`'s.
const ATTACHMENT_PICTURES: &[&str] = &[
    "armor_piercing",
    "burst_fire",
    "full_auto",
    "single_fire",
    "extended_mags",
    "flash_suppressor",
    "iron_sight",
    "muzzle_break",
    "rapid_fire",
    "slug_round",
    "chrome_lined",
    "titanium_frame",
];

#[derive(Clone)]
struct Picture {
    width: u32,
    height: u32,
    format: u32,
    data: Vec<u8>,
}

type Pictures = HashMap<String, Picture>;

fn store() -> &'static Mutex<HashMap<std::path::PathBuf, Pictures>> {
    static STORE: OnceLock<Mutex<HashMap<std::path::PathBuf, Pictures>>> = OnceLock::new();
    STORE.get_or_init(Default::default)
}

fn kept(name: &str) -> bool {
    KEPT.iter().any(|p| name.starts_with(p))
        || ATTACHMENT_PICTURES.iter().any(|p| name.starts_with(p))
}

fn read_pictures(zone_dir: &Path) -> Pictures {
    let mut out = Pictures::new();
    for zone in ["ui_mp", "common_core_mp"] {
        let Ok(z) = crate::Zone::open(&zone_dir.join(format!("{zone}.ff"))) else {
            continue;
        };
        let (cap, _) = crate::walk::walk_zone(&z);
        for m in &cap.materials {
            let name = m.name.trim_start_matches(',');
            if out.contains_key(name) || !kept(name) {
                continue;
            }
            let Some(img) = m
                .textures
                .iter()
                .find(|t| t.semantic == 0 || t.semantic == 2)
                .and_then(|t| t.image)
            else {
                continue;
            };
            let rec = &cap.images[img.index as usize];
            if rec.streamed {
                continue;
            }
            let (w, h) = (u32::from(rec.width), u32::from(rec.height));
            let Some(len) = level_bytes(rec.format, w, h) else {
                continue;
            };
            let Some(bytes) = rec
                .pixels
                .map(|p| p.get(&z.image))
                .and_then(|b| b.get(..len))
            else {
                continue;
            };
            out.insert(
                name.to_owned(),
                Picture {
                    width: w,
                    height: h,
                    format: rec.format,
                    data: bytes.to_vec(),
                },
            );
        }
    }
    out
}

/// The first of `names` the install's 2-D pictures have, as RGBA.
pub fn read_ui_image(install: &Path, names: &[String]) -> Option<(u32, u32, Vec<u8>)> {
    let zone_dir = install.join("zone");
    let mut guard = store().lock().ok()?;
    let pictures = guard
        .entry(zone_dir.clone())
        .or_insert_with(|| read_pictures(&zone_dir));
    names
        .iter()
        .find_map(|n| pictures.get(n.as_str()))
        .and_then(decode_rgba)
}

fn decode_rgba(p: &Picture) -> Option<(u32, u32, Vec<u8>)> {
    let (w, h) = (p.width as usize, p.height as usize);
    let (unit, block) = dxgi_unit(p.format)?;
    let mut out = vec![0u8; w * h * 4];
    if !block {
        if unit != 4 {
            return None;
        }
        out.copy_from_slice(p.data.get(..w * h * 4)?);
        if matches!(p.format, 87..=91) {
            for px in out.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
        }
        return Some((p.width, p.height, out));
    }
    let bw = w.div_ceil(4);
    let mut tmp = [0u8; 64];
    for (i, b) in p
        .data
        .chunks_exact(unit)
        .take(bw * h.div_ceil(4))
        .enumerate()
    {
        let (bx, by) = (i % bw * 4, i / bw * 4);
        match p.format {
            70..=72 => bcdec_rs::bc1(b, &mut tmp, 16),
            73..=75 => bcdec_rs::bc2(b, &mut tmp, 16),
            76..=78 => bcdec_rs::bc3(b, &mut tmp, 16),
            97..=99 => bcdec_rs::bc7(b, &mut tmp, 16),
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
    Some((p.width, p.height, out))
}

#[cfg(test)]
mod tests {
    //! `IW6_INSTALL=<install> cargo test -p fastfile_iw6 --release ui_image -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn ui_image() {
        let Some(install) = std::env::var_os("IW6_INSTALL") else {
            return;
        };
        let install = std::path::PathBuf::from(install);
        for n in [
            "weapon_ak12_360",
            "hud_icon_ak12",
            "scope_acog_360",
            "weapon_maaws_360",
        ] {
            let got = super::read_ui_image(&install, &[n.to_owned()]);
            eprintln!(
                "{n}: {:?}",
                got.as_ref().map(|(w, h, d)| (
                    w,
                    h,
                    d.len(),
                    d.chunks(4).filter(|p| p[3] > 0).count()
                ))
            );
            if let (Some((w, h, d)), Ok(dir)) = (got, std::env::var("IW6_DUMP")) {
                let f = std::fs::File::create(std::path::Path::new(&dir).join(format!("{n}.png")))
                    .unwrap();
                let mut e = png::Encoder::new(f, w, h);
                e.set_color(png::ColorType::Rgba);
                e.write_header().unwrap().write_image_data(&d).unwrap();
            }
        }
    }
}
