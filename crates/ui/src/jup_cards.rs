//! bo2mp: Modern Warfare III's (2023) emblems and calling cards, read from
//! the owner's install at runtime (`fastfile_jup::cards`): the names from
//! its `global` zone, a picture from the package entry its image names,
//! decoded from BC1/BC7 when it is first shown. Nothing of the game is
//! copied or kept on disk.
//!
//! The shell names one `jup:<name>` (`jup:ui_emblem_005`,
//! `jup:jup_playercard_animated_opc_..._atlas`); a flipbook's frames are
//! `jup:<name>#<frame>` (`#0` the same as no frame). Their titles are
//! encrypted in the game's files and are not read: the names stand in.
//!
//! The install is `$BO2_JUP_INSTALL`, else the `jup` line of the game
//! folders file. Without it (or off Windows, where the game's Oodle DLL
//! cannot load) the lists are empty.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use fastfile_jup::cards::{self as jc, Item, Kind, Packages, Rgba};
use fastfile_jup::oodle::Decompressor;

/// The shell's prefix for these.
pub(crate) const PREFIX: &str = "jup";
/// Where the install is, when not in the game folders file.
pub(crate) const INSTALL_ENV: &str = "BO2_JUP_INSTALL";

/// The largest side a picture is kept at: an emblem's frame 256 square, a
/// calling card's 512x128.
const EMBLEM_SIDE: u32 = 256;
const CARD_SIZE: (u32, u32) = (512, 128);
/// Flipbooks decoded and kept (each up to 16 frames).
const KEEP_BOOKS: usize = 12;

/// Every one there is, read once.
struct Catalog {
    install: PathBuf,
    items: HashMap<String, Item>,
    cards: Vec<String>,
    emblems: Vec<String>,
    oodle: Box<dyn Decompressor>,
    packages: Mutex<Packages>,
}

/// The install folder, when it has the zone the names are in.
pub(crate) fn install() -> Option<PathBuf> {
    let set = std::env::var_os(INSTALL_ENV).filter(|p| !p.is_empty()).map(PathBuf::from).or_else(|| asset_transport::game_paths::get(PREFIX))?;
    set.join("zone").join(format!("{}.ff", jc::ZONE)).is_file().then_some(set)
}

fn catalog() -> Option<&'static Catalog> {
    static CAT: OnceLock<Option<Catalog>> = OnceLock::new();
    CAT.get_or_init(|| {
        let install = install()?;
        let oodle = match fastfile_jup::oodle::load(&install) {
            Ok(o) => o,
            Err(e) => {
                diag::warn!(Ui, "bo2mp: Modern Warfare III emblems and calling cards: no Oodle ({e})");
                return None;
            }
        };
        let t = std::time::Instant::now();
        let zone_dir = install.join("zone");
        let zone = match fastfile_jup::read_zone(&zone_dir.join(format!("{}.ff", jc::ZONE)), oodle.as_ref()) {
            Ok((_, z)) => z,
            Err(e) => {
                diag::warn!(Ui, "bo2mp: Modern Warfare III's {}.ff not read: {e}", jc::ZONE);
                return None;
            }
        };
        // (Every picture is in a package: none is kept in the zone, which
        // is dropped once its names are read.)
        let found: Vec<Item> = jc::find_items(&zone).into_iter().filter(|i| i.image.inline.is_none() && fastfile_jup::image::dxgi(i.image.format).is_some()).collect();
        let (mut cards, mut emblems) = (Vec::new(), Vec::new());
        for it in &found {
            let name = format!("{PREFIX}:{}", it.name);
            match it.kind {
                Kind::Card => cards.push(name),
                Kind::Emblem => emblems.push(name),
            }
        }
        diag::info!(Ui, "bo2mp: Modern Warfare III: {} emblems and {} calling cards ({:?})", emblems.len(), cards.len(), t.elapsed());
        Some(Catalog {
            items: found.into_iter().map(|i| (i.name.clone(), i)).collect(),
            cards,
            emblems,
            oodle,
            packages: Mutex::new(Packages::new(&zone_dir, false)),
            install,
        })
    })
    .as_ref()
}

/// (Calling cards, emblems) as `jup:<name>`, in the game's order; empty
/// without the install. Slow the first time (the zone is read): call it
/// off the main thread.
pub(crate) fn names() -> (Vec<String>, Vec<String>) {
    catalog().map_or_else(Default::default, |c| (c.cards.clone(), c.emblems.clone()))
}

/// Whether a shell name is one of these (`jup:` and an emblem's or card's
/// name, any frame).
pub(crate) fn is_look(material: &str) -> bool {
    material.split_once(':').is_some_and(|(g, n)| g == PREFIX && jc::kind_of(split_frame(n).0).is_some())
}

/// `name#3` -> (`name`, 3).
fn split_frame(name: &str) -> (&str, u32) {
    match name.rsplit_once('#') {
        Some((n, f)) => (n, f.parse().unwrap_or(0)),
        None => (name, 0),
    }
}

/// A frame's shell name.
pub(crate) fn frame_name(name: &str, frame: u32) -> String {
    if frame == 0 { name.to_owned() } else { format!("{name}#{frame}") }
}

/// A flipbook's frames, decoded and shrunk.
struct Book {
    frames: Vec<Rgba>,
}

/// The decoded books by name, the last used last; and how many frames each
/// name has once it was decoded (kept for good).
#[derive(Default)]
struct Books {
    kept: Vec<(String, Arc<Book>)>,
    counts: HashMap<String, u32>,
}

static BOOKS: Mutex<Option<Books>> = Mutex::new(None);

/// How many frames a name's picture has: 1 for a still one, and until its
/// picture is first read.
pub(crate) fn frames(material: &str) -> u32 {
    let Some((_, n)) = material.split_once(':') else { return 1 };
    BOOKS.lock().ok().and_then(|b| b.as_ref().and_then(|b| b.counts.get(split_frame(n).0).copied())).unwrap_or(1)
}

/// The kept book of a name (now the last used).
fn kept(name: &str) -> Option<Arc<Book>> {
    let mut b = BOOKS.lock().ok()?;
    let b = b.get_or_insert_with(Books::default);
    let i = b.kept.iter().position(|(n, _)| n == name)?;
    let hit = b.kept.remove(i);
    let book = hit.1.clone();
    b.kept.push(hit);
    Some(book)
}

/// A name's book: kept, else decoded (one at a time, so a flipbook's
/// frames asked for together decode it once, and the reads stay light).
fn book(name: &str) -> Option<Arc<Book>> {
    static DECODING: Mutex<()> = Mutex::new(());
    if let Some(b) = kept(name) {
        return Some(b);
    }
    let _one = DECODING.lock().ok()?;
    if let Some(b) = kept(name) {
        return Some(b);
    }
    decode(name)
}

fn decode(name: &str) -> Option<Arc<Book>> {
    let cat = catalog()?;
    let item = cat.items.get(name)?;
    let max = match item.kind {
        Kind::Emblem => EMBLEM_SIDE,
        Kind::Card => CARD_SIZE.0,
    };
    let mut fetch = |key: u64| {
        // (The row is found under the lock; the read and decode are not.)
        let at = cat.packages.lock().ok()?.locate(key);
        at.and_then(|(part, entry)| jc::read_located(&part, &entry, cat.oodle.as_ref()))
    };
    let Some(pic) = jc::picture(None, item, &mut fetch, max) else {
        diag::warn!(Ui, "bo2mp: Modern Warfare III's `{name}`: its picture is not in the install's packages ({})", cat.install.display());
        return None;
    };
    let grid = jc::detect_grid(item, &pic);
    let n = grid.0 * grid.1;
    let (w, h) = match item.kind {
        Kind::Emblem => (EMBLEM_SIDE, EMBLEM_SIDE),
        Kind::Card => CARD_SIZE,
    };
    let frames: Vec<Rgba> = (0..n).map(|i| jc::shrink(&jc::frame(&pic, grid, i), w, h)).collect();
    let book = Arc::new(Book { frames });
    if let Ok(mut b) = BOOKS.lock() {
        let b = b.get_or_insert_with(Books::default);
        b.counts.insert(name.to_owned(), n);
        b.kept.push((name.to_owned(), book.clone()));
        if b.kept.len() > KEEP_BOOKS {
            b.kept.remove(0);
        }
    }
    Some(book)
}

/// A picture for the shell: the first of `names` (`<name>` or
/// `<name>#<frame>`, no prefix) it has, as RGBA. Reads the install: call
/// it off the main thread.
pub(crate) fn read_picture(names: &[String]) -> Option<(u32, u32, Vec<u8>)> {
    names.iter().find_map(|n| {
        let (name, f) = split_frame(n);
        let book = book(name)?;
        let frame = book.frames.get(f as usize % book.frames.len().max(1))?;
        Some((frame.width, frame.height, frame.pixels.clone()))
    })
}

/// A name as he reads it: the words of its name without the game's
/// bookkeeping (`jup_emblem_animated_opc_greekmythhades_01_atlas` ->
/// `Greekmythhades 01`, `ui_emblem_005` -> `Emblem 005`,
/// `ui_playercard_100` -> `Calling Card 100`).
pub(crate) fn title(name: &str) -> String {
    let name = name.split_once(':').map_or(name, |(_, n)| n);
    let name = split_frame(name).0;
    let card = name.contains("playercard");
    let words: Vec<&str> = name
        .split('_')
        .filter(|w| !w.is_empty() && !matches!(*w, "ui" | "jup" | "wzm" | "emblem" | "playercard" | "animated" | "opc" | "atlas" | "altas"))
        .collect();
    let numbered = words.first().is_some_and(|w| w.chars().all(|c| c.is_ascii_digit()));
    let mut out: Vec<String> = Vec::new();
    if words.is_empty() || numbered {
        out.push(if card { "Calling Card".to_owned() } else { "Emblem".to_owned() });
    }
    for w in words {
        let mut c = w.chars();
        out.push(c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect()));
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_and_frames() {
        assert_eq!(title("jup:jup_emblem_animated_opc_greekmythhades_01_atlas"), "Greekmythhades 01");
        assert_eq!(title("jup:ui_emblem_005"), "Emblem 005");
        assert_eq!(title("jup:ui_playercard_100#3"), "Calling Card 100");
        assert_eq!(title("jup:ui_emblem_mastery_gold_apapa"), "Mastery Gold Apapa");
        assert_eq!(split_frame("ui_emblem_1#12"), ("ui_emblem_1", 12));
        assert_eq!(frame_name("jup:x", 0), "jup:x");
        assert_eq!(frame_name("jup:x", 2), "jup:x#2");
        assert!(is_look("jup:ui_emblem_005#2"));
        assert!(!is_look("jup:ui_group_bg_emblem_1"));
        assert!(!is_look("iw4:cardicon_skull"));
    }
}
