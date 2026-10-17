//! bo2mp: Black Ops' and Black Ops II's own calling cards and emblems in the
//! Barracks pickers, beside Modern Warfare's.
//!
//! Black Ops II's are the backgrounds and icons its emblem editor offers
//! (`mp/emblemsOrBackings.csv` in `patch_mp`), their pictures in its image
//! packs (`t6:<name>`). An animated card (`em_bg_ani_*`) is its frames
//! stacked in one picture, each a card's shape (4:1); `t6:<name>#<n>` is
//! frame `n`. Black Ops' are the `emblem_bg_*` (cards) and `em_*` (emblems)
//! pictures in its IWDs (`t5:<name>`).
//!
//! Both games' emblems are shapes (the editor colours them): their pictures
//! carry only brightness, which becomes the emblem's cover, drawn white.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(crate) const BO2: &str = "t6";
pub(crate) const BO1: &str = "t5";

const TABLE: &str = "mp/emblemsorbackings.csv";

/// Black Ops II's (cards, emblems), as `t6:<name>`, in the editor's order.
pub(crate) fn bo2_names() -> (Vec<String>, Vec<String>) {
    static NAMES: OnceLock<(Vec<String>, Vec<String>)> = OnceLock::new();
    NAMES.get_or_init(read_bo2_names).clone()
}

fn read_bo2_names() -> (Vec<String>, Vec<String>) {
    let Some(zone) = asset_transport::games_root_from_env()
        .ok()
        .and_then(|root| asset_transport::find_zone_file(&root, "t6:patch_mp").ok())
    else {
        return Default::default();
    };
    let capture = match asset_t6::capture_zone(&zone.path) {
        Ok(c) => c,
        Err(e) => {
            diag::warn!(
                Ui,
                "bo2mp shell: Black Ops II's emblems: {}: {e}",
                zone.path.display()
            );
            return Default::default();
        }
    };
    let Some(t) = capture
        .string_tables
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(TABLE))
    else {
        return Default::default();
    };
    let (mut cards, mut emblems) = (Vec::new(), Vec::new());
    for row in t.cells.chunks(t.columns.max(1)).filter(|r| r.len() > 3) {
        let name = row[3].trim();
        // (The blank default card is not one to pick.)
        if name.is_empty() || name == "emblem_bg_default" {
            continue;
        }
        let out = match row[0].as_str() {
            "background" => &mut cards,
            "icon" => &mut emblems,
            _ => continue,
        };
        let name = format!("{BO2}:{name}");
        if !out.contains(&name) {
            out.push(name);
        }
    }
    diag::info!(
        Ui,
        "bo2mp shell: {} Black Ops II calling cards and {} emblems",
        cards.len(),
        emblems.len()
    );
    (cards, emblems)
}

/// Black Ops' (cards, emblems), as `t5:<name>`, by name.
pub(crate) fn bo1_names() -> (Vec<String>, Vec<String>) {
    let tag = |n: String| format!("{BO1}:{n}");
    let cards = crate::foreign_maps::game_image_names(BO1, "emblem_bg_")
        .into_iter()
        .map(tag)
        .collect();
    let emblems = crate::foreign_maps::game_image_names(BO1, "em_")
        .into_iter()
        .filter(|n| !n.starts_with("em_bg") && !n.contains("rank") && !n.contains("prestige"))
        .map(tag)
        .collect();
    (cards, emblems)
}

/// Whether a shell name is one of these (`t5:` / `t6:` and a card's or an
/// emblem's name, any frame).
pub(crate) fn is_look(material: &str) -> bool {
    material.split_once(':').is_some_and(|(g, n)| {
        (g == BO1 || g == BO2) && (n.starts_with("em_") || n.starts_with("emblem_bg_"))
    })
}

/// `name#3` -> (`name`, 3); `name` -> (`name`, 0).
fn split_frame(name: &str) -> (&str, u32) {
    match name.rsplit_once('#') {
        Some((n, f)) if f.bytes().all(|b| b.is_ascii_digit()) => (n, f.parse().unwrap_or(0)),
        _ => (name, 0),
    }
}

/// How many frames each animated card has, once read.
static FRAMES: Mutex<Option<HashMap<String, u32>>> = Mutex::new(None);

/// How many frames a Black Ops II card has (1: a still one, or not read
/// yet).
pub(crate) fn frames(shell_name: &str) -> u32 {
    let Some((_, n)) = shell_name.split_once(':') else {
        return 1;
    };
    let n = split_frame(n).0;
    FRAMES
        .lock()
        .ok()
        .and_then(|f| f.as_ref().and_then(|f| f.get(n).copied()))
        .unwrap_or(1)
}

/// A Black Ops II picture by name (`em_bg_ani_dragon#4`: that frame), from
/// its image packs.
pub(crate) fn read_bo2(name: &str) -> Option<(u32, u32, Vec<u8>)> {
    static ZONE_ALL: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
    let (name, frame) = split_frame(name);
    let zone_all = ZONE_ALL
        .get_or_init(|| {
            asset_transport::games_root_from_env()
                .ok()
                .and_then(|root| asset_transport::find_zone_file(&root, "t6:patch_mp").ok())
                .and_then(|z| z.path.parent().map(std::path::Path::to_path_buf))
        })
        .as_deref()?;
    let (w, h, px) = assets::t6_mod_picture_rgba(name, Some(zone_all))?;
    // (A card is four times as wide as it is tall: a taller one is frames.)
    let frame_h = w / 4;
    if name.starts_with("em_bg") && frame_h > 0 && h > frame_h && h % frame_h == 0 {
        let n = h / frame_h;
        if let Ok(mut f) = FRAMES.lock() {
            f.get_or_insert_with(HashMap::new)
                .insert(name.to_owned(), n);
        }
        let k = (frame % n) as usize;
        let row = (w * 4) as usize;
        let start = k * frame_h as usize * row;
        let px = px.get(start..start + frame_h as usize * row)?.to_vec();
        return Some((w, frame_h, px));
    }
    Some((w, h, px))
}

/// An emblem's shape: a picture with no cover anywhere carries the shape in
/// its brightness; it becomes the cover, drawn white.
pub(crate) fn shape_to_cover(mut img: (u32, u32, Vec<u8>)) -> (u32, u32, Vec<u8>) {
    let px = &mut img.2;
    if px.chunks_exact(4).all(|p| p[3] == 0) {
        for p in px.chunks_exact_mut(4) {
            let a = p[0].max(p[1]).max(p[2]);
            p.copy_from_slice(&[255, 255, 255, a]);
        }
    }
    img
}
