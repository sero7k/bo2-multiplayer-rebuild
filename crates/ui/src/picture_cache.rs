//! bo2mp: menu pictures read from other games, kept on disk.
//!
//! A picture another game holds (a guest gun's icon, a calling card, a
//! map's preview) is read from that game's files the first time a menu asks
//! for it, and for some games that means walking a whole zone (Black Ops
//! III's core_common for one weapon icon, Ghosts' ui_mp). The picture
//! that comes out is the same every run for the same install, so it is
//! written to the game's cache (`iw4l-artifacts/cache/ui_picture`) as its
//! RGBA pixels, and read back from there on the next run, before any of
//! the game's files are opened. A picture the game does not have is kept
//! as a miss, so the zone is not walked again to look for it.
//!
//! An entry's key is the picture's key (`<game>:<name>`, `map:<map>`), the
//! readers' code (so a change to how a picture is read makes a new entry)
//! and this format's number.

const KIND: &str = "ui_picture";
const MAGIC: &[u8; 8] = b"BO2PIC1\0";
/// The layout's number (raised when what is written changes).
const FORMAT: u64 = 1;

/// The text of the code the pictures come out of.
const CODE: &[&str] = &[
    include_str!("picture_cache.rs"),
    include_str!("foreign_maps.rs"),
    include_str!("../../fastfile_iw6/src/ui_image.rs"),
    include_str!("../../fastfile_s1/src/ui_image.rs"),
    include_str!("../../fastfile_iw7/src/ui_image.rs"),
    include_str!("../../fastfile_t7/src/ui_image.rs"),
];

pub(crate) type Rgba = (u32, u32, Vec<u8>);

fn code_hash() -> u64 {
    static HASH: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *HASH.get_or_init(|| {
        let mut h = asset_transport::fnv1a64(&FORMAT.to_le_bytes());
        for text in CODE {
            h = asset_transport::fnv1a64_more(h, text.as_bytes());
        }
        h
    })
}

fn key_of(picture: &str) -> String {
    let h = asset_transport::fnv1a64_more(code_hash(), picture.as_bytes());
    format!("{h:016x}")
}

/// Whether pictures under `key` are worth keeping (Modern Warfare III's
/// come from an index of their own already, in a moment).
pub(crate) fn kept(key: &str) -> bool {
    !key.starts_with("jup:") && !key.starts_with("gamebanner:") && std::env::var("BO2MP_PICTURE_CACHE").map_or(true, |v| v.trim() != "0")
}

/// What the cache holds for `picture`: `Some(Some(_))` its pixels,
/// `Some(None)` a miss kept from an earlier run, `None` nothing kept.
pub(crate) fn get(picture: &str) -> Option<Option<Rgba>> {
    let bytes = asset_transport::cache_get(KIND, &key_of(picture))?;
    let body = bytes.strip_prefix(MAGIC.as_slice())?;
    let (&found, body) = body.split_first()?;
    if found == 0 {
        return Some(None);
    }
    let w = u32::from_le_bytes(body.get(0..4)?.try_into().ok()?);
    let h = u32::from_le_bytes(body.get(4..8)?.try_into().ok()?);
    let rgba = lz4_flex::decompress_size_prepended(body.get(8..)?).ok()?;
    (rgba.len() as u64 == u64::from(w) * u64::from(h) * 4).then_some(Some((w, h, rgba)))
}

/// Keeps `picture`'s pixels (or that the game has none).
pub(crate) fn put(picture: &str, found: Option<&Rgba>) {
    let mut out = Vec::with_capacity(32 + found.map_or(0, |f| f.2.len() / 2));
    out.extend_from_slice(MAGIC);
    match found {
        None => out.push(0),
        Some((w, h, rgba)) => {
            out.push(1);
            out.extend_from_slice(&w.to_le_bytes());
            out.extend_from_slice(&h.to_le_bytes());
            out.extend_from_slice(&lz4_flex::compress_prepend_size(rgba));
        }
    }
    if let Err(e) = asset_transport::cache_put(KIND, &key_of(picture), &out) {
        diag::warn!(Ui, "picture `{picture}`: not kept in the cache: {e}");
    }
}

/// `read`'s picture for `picture`, from the cache when it holds it, else
/// read and kept.
pub(crate) fn read_through(picture: &str, read: impl FnOnce() -> Option<Rgba>) -> (Option<Rgba>, bool) {
    if !kept(picture) {
        return (read(), false);
    }
    if let Some(hit) = get(picture) {
        return (hit, true);
    }
    let found = read();
    put(picture, found.as_ref());
    (found, false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn keys_are_cache_safe_and_differ() {
        let a = super::key_of("t7:t7_icon_weapon_ar_standard");
        let b = super::key_of("iw4:cardtitle_tf141");
        assert_ne!(a, b);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()) && a.len() == 16);
        assert!(!super::kept("jup:ui_playercard_438"));
    }
}
