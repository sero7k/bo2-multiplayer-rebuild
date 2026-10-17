//! Advanced Warfare's, Ghosts' and Infinite Warfare's packs
//! (`imagefile<N>.pak`, Infinite Warfare's `assetfile<N>.pak` too) hold every
//! zone's streamed pictures and assets. Each zone's header lists its own:
//! the pack's number and the byte range in it (`XStreamFile`). From the
//! zones kept, which packs, beside them, and which parts of them the
//! rebuild can ever read. (A pack in a folder no kept zone streams from -
//! a language's - is kept whole.)
//!
//! Advanced Warfare and Ghosts lay the table out alike: image count at
//! 0x18, 24-byte entries from 0x1c (u16, u16 pack number, u32, u64 start,
//! u64 end). Infinite Warfare's is `fastfile_iw7::container`'s.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

/// A pack: its folder, its kind (`imagefile`, `assetfile`), its number.
pub type Pack = (PathBuf, &'static str, u16);

/// Byte ranges per pack, merged.
pub type Ranges = BTreeMap<Pack, Vec<(u64, u64)>>;

/// The streamed parts a zone's header lists, as (pack, start, end): none for
/// a zone that is not laid out so, or does not read. `iw7`: Infinite
/// Warfare's layout.
pub fn of_zone(path: &Path, iw7: bool) -> Vec<(Pack, u64, u64)> {
    let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
    if iw7 {
        return of_iw7_zone(path)
            .into_iter()
            .map(|(kind, n, s, e)| ((dir.clone(), kind, n), s, e))
            .collect();
    }
    of_s1_zone(path).into_iter().map(|(n, s, e)| ((dir.clone(), "imagefile", n), s, e)).collect()
}

/// Infinite Warfare: its own images (`imagefile`) and shared assets
/// (`assetfile`), not the language copies.
fn of_iw7_zone(path: &Path) -> Vec<(&'static str, u16, u64, u64)> {
    let Ok(mut f) = std::fs::File::open(path) else { return Vec::new() };
    let mut head = vec![0u8; 0xd0];
    if f.read_exact(&mut head).is_err() || !fastfile_iw7::container::is_iw7_zone(&head) {
        return Vec::new();
    }
    let count = |o: usize| u32::from_le_bytes([head[o], head[o + 1], head[o + 2], head[o + 3]]) as usize;
    let n = count(0xc4) + count(0xcc);
    if n > 4_000_000 {
        return Vec::new();
    }
    let mut rest = vec![0u8; n * 24 + 24];
    if f.read_exact(&mut rest).is_err() {
        return Vec::new();
    }
    head.extend(rest);
    let Ok(h) = fastfile_iw7::container::parse_file_header(&head) else { return Vec::new() };
    let pick = |kind: &'static str, v: &[fastfile_iw7::container::StreamFile]| -> Vec<(&'static str, u16, u64, u64)> {
        v.iter().filter(|s| !s.localized && s.offset_end > s.offset).map(|s| (kind, s.file_index, s.offset, s.offset_end)).collect()
    };
    let mut out = pick("imagefile", &h.stream_files);
    out.extend(pick("assetfile", &h.shared_files));
    out
}

fn of_s1_zone(path: &Path) -> Vec<(u16, u64, u64)> {
    let Ok(mut f) = std::fs::File::open(path) else { return Vec::new() };
    let mut head = [0u8; 0x1c];
    if f.read_exact(&mut head).is_err() {
        return Vec::new();
    }
    let count = u32::from_le_bytes([head[0x18], head[0x19], head[0x1a], head[0x1b]]) as usize;
    let len = std::fs::metadata(path).map_or(0, |m| m.len());
    if count == 0 || (count as u64) * 24 + 0x1c > len || count > 1_000_000 {
        return Vec::new();
    }
    let mut table = vec![0u8; count * 24];
    if f.read_exact(&mut table).is_err() {
        return Vec::new();
    }
    table
        .chunks_exact(24)
        .filter_map(|e| {
            let pack = u16::from_le_bytes([e[2], e[3]]);
            let start = u64::from_le_bytes(e[8..16].try_into().ok()?);
            let end = u64::from_le_bytes(e[16..24].try_into().ok()?);
            (end > start).then_some((pack, start, end))
        })
        .collect()
}

/// Adds parts to the ranges, keeping each pack's sorted and merged.
pub fn add(ranges: &mut Ranges, parts: impl IntoIterator<Item = (Pack, u64, u64)>) {
    for (pack, start, end) in parts {
        ranges.entry(pack).or_default().push((start, end));
    }
    for v in ranges.values_mut() {
        v.sort_unstable();
        let mut out: Vec<(u64, u64)> = Vec::with_capacity(v.len());
        for &(s, e) in v.iter() {
            match out.last_mut() {
                Some(last) if s <= last.1 => last.1 = last.1.max(e),
                _ => out.push((s, e)),
            }
        }
        *v = out;
    }
}

/// A pack's kind and number from its path (`.../imagefile12.pak` ->
/// `(dir, "imagefile", 12)`).
pub fn pack_of(path: &Path) -> Option<Pack> {
    let n = path.file_name()?.to_string_lossy().to_ascii_lowercase();
    let dir = path.parent()?.to_path_buf();
    for kind in ["imagefile", "assetfile"] {
        if let Some(num) = n.strip_prefix(kind).and_then(|r| r.strip_suffix(".pak")).and_then(|r| r.parse().ok()) {
            return Some((dir, kind, num));
        }
    }
    None
}

/// Writes `to` as `from`'s size with only `ranges` copied in: the rest is
/// left unwritten (a sparse file on NTFS, so it takes no room).
pub fn copy_parts(from: &Path, to: &Path, ranges: &[(u64, u64)]) -> std::io::Result<u64> {
    use std::io::{Seek, SeekFrom, Write};
    let size = std::fs::metadata(from)?.len();
    let _ = std::fs::remove_file(to);
    let mut out = std::fs::File::create(to)?;
    drop(out);
    sparse(to);
    out = std::fs::OpenOptions::new().write(true).open(to)?;
    out.set_len(size)?;
    let mut src = std::fs::File::open(from)?;
    let mut buf = vec![0u8; 8 << 20];
    let mut written = 0u64;
    for &(start, end) in ranges {
        let end = end.min(size);
        if start >= end {
            continue;
        }
        src.seek(SeekFrom::Start(start))?;
        out.seek(SeekFrom::Start(start))?;
        let mut left = end - start;
        while left > 0 {
            let n = left.min(buf.len() as u64) as usize;
            src.read_exact(&mut buf[..n])?;
            out.write_all(&buf[..n])?;
            left -= n as u64;
            written += n as u64;
        }
    }
    Ok(written)
}

/// Marks a file sparse (Windows; elsewhere unwritten parts are holes
/// already).
fn sparse(path: &Path) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("fsutil").args(["sparse", "setflag"]).arg(path).output();
    }
    #[cfg(not(windows))]
    {
        let _ = path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_merge_and_packs_are_numbered() {
        let mut r = Ranges::new();
        let p = |n| (PathBuf::from("z"), "imagefile", n);
        add(&mut r, [(p(3), 10, 20), (p(3), 15, 30), (p(3), 40, 50), (p(7), 0, 5)]);
        assert_eq!(r[&p(3)], vec![(10, 30), (40, 50)]);
        assert_eq!(r[&p(7)], vec![(0, 5)]);
        assert_eq!(pack_of(Path::new("z/imagefile12.pak")), Some(p(12)));
        assert_eq!(pack_of(Path::new("z/assetfile2.pak")), Some((PathBuf::from("z"), "assetfile", 2)));
        assert_eq!(pack_of(Path::new("z/soundfile1.pak")), None);
    }

    #[test]
    fn a_zone_header_lists_its_parts_and_parts_copy() {
        let dir = std::env::temp_dir().join(format!("coc-streams-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut zone = vec![0u8; 0x1c];
        zone[0x18] = 1;
        let mut e = vec![0u8; 24];
        e[2] = 4;
        e[8..16].copy_from_slice(&100u64.to_le_bytes());
        e[16..24].copy_from_slice(&200u64.to_le_bytes());
        zone.extend(e);
        zone.extend([0u8; 16]);
        std::fs::write(dir.join("z.ff"), &zone).unwrap();
        assert_eq!(of_zone(&dir.join("z.ff"), false), vec![((dir.clone(), "imagefile", 4), 100, 200)]);
        let pak: Vec<u8> = (0..300u32).map(|i| (i % 251) as u8 + 1).collect();
        std::fs::write(dir.join("p.pak"), &pak).unwrap();
        assert_eq!(copy_parts(&dir.join("p.pak"), &dir.join("q.pak"), &[(100, 200)]).unwrap(), 100);
        let q = std::fs::read(dir.join("q.pak")).unwrap();
        assert_eq!(q.len(), 300);
        assert_eq!(&q[100..200], &pak[100..200]);
        assert!(q[..100].iter().all(|&b| b == 0));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
