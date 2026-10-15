//! bo2mp: Black Ops III guest guns' pictures, converted, kept on disk.
//!
//! Building the Black Ops III guns reads and converts about two thousand
//! pictures from core_common and its packages (colour maps cut to size,
//! normal maps to BC5, specular colour with gloss to BC3) every time the
//! game starts; for the same install what comes out is the same. So the
//! converted pictures are written to the game's cache
//! (`iw4l-artifacts/cache/t7_gun_pics`) as one file, and the next start
//! takes them from there and only converts the ones it lacks.
//!
//! The file's key covers what the pictures are made from: core_common and
//! the `.xpak` packages beside it (name, size, time written), the picture
//! size limit (`BO2MP_BO3_TEXTURE`), this format's number and the code
//! that converts them (a change to it makes a new entry, never a stale
//! one). A file that does not read back whole is a miss.

use std::collections::HashMap;
use std::path::Path;

use fastfile_iw6::image::ImagePixels;

const KIND: &str = "t7_gun_pics";
const MAGIC: &[u8; 8] = b"BO2T7GP\0";
/// The layout's number (raised when what is written changes).
const FORMAT: u64 = 1;

/// The text of the code the pictures come out of.
const CODE: &[&str] = &[
    include_str!("t7_gun_cache.rs"),
    include_str!("t7.rs"),
    include_str!("../../../fastfile_t7/src/texture.rs"),
];

/// A converted picture by (its key, its size limit).
pub(crate) type Pictures = HashMap<(String, u32), ImagePixels>;

/// The cache key for the guns' pictures from `dir` (Black Ops III's
/// `zone` folder), or `None` when core_common is not there.
pub(crate) fn key_for(dir: &Path, first_person_max: u32) -> Option<String> {
    let mut h = asset_transport::fnv1a64(&FORMAT.to_le_bytes());
    for text in CODE {
        h = asset_transport::fnv1a64_more(h, text.as_bytes());
    }
    h = asset_transport::fnv1a64_more(h, &first_person_max.to_le_bytes());
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
            name == "core_common.ff" || name.ends_with(".xpak")
        })
        .collect();
    files.sort();
    if !files.iter().any(|p| p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("core_common.ff"))) {
        return None;
    }
    for f in &files {
        let meta = std::fs::metadata(f).ok()?;
        let written = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs());
        h = asset_transport::fnv1a64_more(h, f.file_name()?.to_string_lossy().to_ascii_lowercase().as_bytes());
        h = asset_transport::fnv1a64_more(h, &meta.len().to_le_bytes());
        h = asset_transport::fnv1a64_more(h, &written.to_le_bytes());
    }
    Some(format!("{h:016x}"))
}

fn u32_at(b: &[u8], at: &mut usize) -> Option<u32> {
    let v = u32::from_le_bytes(b.get(*at..*at + 4)?.try_into().ok()?);
    *at += 4;
    Some(v)
}

/// The pictures kept under `key` (empty on a miss or a broken file).
pub(crate) fn load(key: &str) -> Pictures {
    let mut out = Pictures::new();
    let Some(bytes) = asset_transport::cache_get(KIND, key) else { return out };
    let Some(packed) = bytes.strip_prefix(MAGIC.as_slice()) else { return out };
    let Ok(body) = lz4_flex::decompress_size_prepended(packed) else { return out };
    let mut at = 0usize;
    let read = (|| -> Option<Pictures> {
        let n = u32_at(&body, &mut at)?;
        let mut pics = Pictures::with_capacity(n as usize);
        for _ in 0..n {
            let name_len = u32_at(&body, &mut at)? as usize;
            let name = std::str::from_utf8(body.get(at..at + name_len)?).ok()?.to_owned();
            at += name_len;
            let max = u32_at(&body, &mut at)?;
            let width = u32_at(&body, &mut at)?;
            let height = u32_at(&body, &mut at)?;
            let levels = u32_at(&body, &mut at)?;
            let dxgi_format = u32_at(&body, &mut at)?;
            let len = u32_at(&body, &mut at)? as usize;
            let data = body.get(at..at + len)?.to_vec();
            at += len;
            pics.insert((name, max), ImagePixels { width, height, levels, dxgi_format, data });
        }
        (at == body.len()).then_some(pics)
    })();
    if let Some(pics) = read {
        out = pics;
    }
    out
}

/// Keeps `pictures` under `key` (written off to the side and swapped in
/// whole, so a reader never sees half a file).
pub(crate) fn store(key: &str, pictures: &Pictures) -> Result<(), String> {
    let mut body = Vec::with_capacity(pictures.values().map(|p| p.data.len() + 64).sum::<usize>() + 4);
    body.extend_from_slice(&(pictures.len() as u32).to_le_bytes());
    let mut entries: Vec<_> = pictures.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    for ((name, max), px) in entries {
        body.extend_from_slice(&(name.len() as u32).to_le_bytes());
        body.extend_from_slice(name.as_bytes());
        for v in [*max, px.width, px.height, px.levels, px.dxgi_format, px.data.len() as u32] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        body.extend_from_slice(&px.data);
    }
    let mut out = Vec::with_capacity(body.len() / 2 + 16);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&lz4_flex::compress_prepend_size(&body));
    asset_transport::cache_put(KIND, key, &out)?;
    asset_transport::cache_prune(KIND, "", key);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trip() {
        let mut pics = super::Pictures::new();
        pics.insert(
            ("i_wpn_t7_test_c".into(), 1024),
            fastfile_iw6::image::ImagePixels { width: 4, height: 4, levels: 1, dxgi_format: 71, data: vec![7; 8] },
        );
        // (Encoded and decoded as `store`/`load` do, without the disk.)
        let mut body = Vec::new();
        body.extend_from_slice(&1u32.to_le_bytes());
        let ((name, max), px) = pics.iter().next().unwrap();
        body.extend_from_slice(&(name.len() as u32).to_le_bytes());
        body.extend_from_slice(name.as_bytes());
        for v in [*max, px.width, px.height, px.levels, px.dxgi_format, px.data.len() as u32] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        body.extend_from_slice(&px.data);
        let packed = lz4_flex::compress_prepend_size(&body);
        assert_eq!(lz4_flex::decompress_size_prepended(&packed).unwrap(), body);
    }
}
