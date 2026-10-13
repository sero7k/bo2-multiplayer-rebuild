//! Advanced Warfare's 2-D pictures (Create-a-Class `weapon_<gun>`, kill
//! icons `hud_icon_<gun>`) by material name, as RGBA, from `common_mp.ff`.
//!
//! The first call walks common_mp once and keeps the inline top level of
//! the small pictures and where the streamed ones are (`imagefile<N>.pak`);
//! the streamed ones are read when asked for.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::image::{ImagePaks, ImagePixels, dxgi_unit, image_pixels};
use crate::walk::ImageRec;

/// Material name prefixes kept.
const KEPT: &[&str] = &["weapon_", "hud_icon_"];

enum Picture {
    Inline(ImagePixels),
    Streamed(ImageRec),
}

type Pictures = HashMap<String, Picture>;

fn store() -> &'static Mutex<HashMap<PathBuf, Pictures>> {
    static STORE: OnceLock<Mutex<HashMap<PathBuf, Pictures>>> = OnceLock::new();
    STORE.get_or_init(Default::default)
}

fn kept(name: &str) -> bool {
    KEPT.iter().any(|p| name.starts_with(p))
        // (the camo-loot and gold variants of each gun's picture)
        && !name.contains("_loot")
        && !name.ends_with("_gold")
        && !name.ends_with("_atlas")
}

fn read_pictures(zone_dir: &Path) -> Pictures {
    let mut out = Pictures::new();
    let Ok(z) = crate::Zone::open(&zone_dir.join("common_mp.ff")) else {
        return out;
    };
    let mut w = crate::walk::Walker::new(&z.image, &z.assets, &z.header.stream_files);
    let _ = w.walk();
    let cap = &w.cap;
    let mut paks = ImagePaks::new(zone_dir);
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
        let pic = if rec.streamed {
            Picture::Streamed(rec.clone())
        } else {
            match image_pixels(rec, &z.image, &mut paks, 1024) {
                Ok(px) => Picture::Inline(px),
                Err(_) => continue,
            }
        };
        out.insert(name.to_owned(), pic);
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
    names.iter().find_map(|n| match pictures.get(n.as_str())? {
        Picture::Inline(px) => decode_rgba(px),
        Picture::Streamed(rec) => {
            let mut paks = ImagePaks::new(&zone_dir);
            let px = image_pixels(rec, &[], &mut paks, 1024).ok()?;
            decode_rgba(&px)
        }
    })
}

/// The top level of `p` as RGBA.
fn decode_rgba(p: &ImagePixels) -> Option<(u32, u32, Vec<u8>)> {
    let (w, h) = (p.width as usize, p.height as usize);
    let format = p.dxgi_format;
    let (unit, block) = dxgi_unit(format)?;
    let mut out = vec![0u8; w * h * 4];
    if !block {
        if unit != 4 {
            return None;
        }
        out.copy_from_slice(p.data.get(..w * h * 4)?);
        if matches!(format, 87..=91) {
            for px in out.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
        }
        return Some((p.width, p.height, out));
    }
    let bw = w.div_ceil(4);
    let blocks = bw * h.div_ceil(4);
    let mut tmp = [0u8; 64];
    for (i, b) in p.data.get(..blocks * unit)?.chunks_exact(unit).enumerate() {
        let (bx, by) = (i % bw * 4, i / bw * 4);
        match format {
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
    //! `S1_INSTALL=<install> cargo test -p fastfile_s1 --release ui_image -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn ui_image() {
        let Some(install) = std::env::var_os("S1_INSTALL") else {
            return;
        };
        let install = std::path::PathBuf::from(install);
        for n in ["weapon_ak12", "hud_icon_ak12", "hud_icon_bal27"] {
            let got = super::read_ui_image(&install, &[n.to_owned()]);
            eprintln!(
                "{n}: {:?}",
                got.as_ref().map(|(w, h, d)| (w, h, d.len(), d.chunks(4).filter(|p| p[3] > 0).count()))
            );
        }
    }
}
