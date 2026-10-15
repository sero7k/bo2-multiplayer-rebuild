//! Images (asset type 21): the header, its streamed mip groups, and the
//! zone's run of images by which other assets point at them.
//!
//! ```text
//! 0x00 u64 name hash          0x08 u64 0
//! 0x10 u32 0, u32 flags       0x18 u32 data size (every level)
//! 0x1c u32 ?                  0x20 u16 width, height, depth, array size
//! 0x28 u8 ?, u8 ?, u8 format, u8 usage, u8 level count, u8 ?, u8 group count, u8 ?
//! 0x30 ptr groups (-2: group count records follow)
//! 0x38 ptr pixels (-2: data size bytes follow, a small image in the zone)
//! group record, 24 bytes: u64 package key, u64 0, u16 ?, u16 ?,
//!   u32 bits 0..23 = bytes of every level up to this group's / 4,
//!   bits 24..31 = ?
//! ```
//!
//! Groups run smallest first (the 4x4 and 8x8 levels in one, then a
//! group per level). A group's bytes are one `.xsub` entry under its key;
//! the larger groups of most images are not in the install (fetched on
//! demand by the game) and the largest group present is used.
//!
//! A pointer to an image (from a material) is a load-block offset: every
//! image of a zone is one run of assets in its asset list, and each takes
//! 16 bytes of block 0x0f in that order, so an image pointer is
//! `base + 16 * (its index in the run)`. [`ImageRun`] finds the run in the
//! zone; the base is calibrated from a pointer whose image is known.
#![forbid(unsafe_code)]

use crate::ptr::{Bytes, FOLLOW, OutOfBounds};

pub const HEADER_LEN: usize = 0x40;
pub const GROUP_LEN: usize = 24;
/// The block-0x0f bytes each image of the run takes.
pub const SLOT: u32 = 16;

/// One streamed group of mip levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MipGroup {
    pub key: u64,
    /// Bytes of every level up to and including this group's.
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageRec {
    /// Where its header is in the zone.
    pub at: usize,
    pub hash: u64,
    pub flags: u32,
    pub data_size: u32,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub array: u16,
    pub format: u8,
    pub usage: u8,
    /// Mip levels (0 where not set: every level down to 4x4 is assumed).
    pub levels: u8,
    pub groups: Vec<MipGroup>,
    /// The pixels in the zone, for a small image: (offset, length).
    pub inline: Option<(usize, usize)>,
    /// Where the next asset starts.
    pub end: usize,
}

/// The header at `at` read as an image, or `None` where the bytes do not
/// have an image's shape.
pub fn parse(zone: &[u8], at: usize) -> Result<Option<ImageRec>, OutOfBounds> {
    let b = Bytes(zone);
    let h = b.get(at, HEADER_LEN)?;
    let u64_at = |o: usize| u64::from_le_bytes(h[o..o + 8].try_into().unwrap_or([0; 8]));
    let u32_at = |o: usize| u32::from_le_bytes(h[o..o + 4].try_into().unwrap_or([0; 4]));
    let u16_at = |o: usize| u16::from_le_bytes([h[o], h[o + 1]]);
    let (groups_ptr, pixels_ptr) = (u64_at(0x30), u64_at(0x38));
    if !matches!(groups_ptr, 0 | FOLLOW) || !matches!(pixels_ptr, 0 | FOLLOW) {
        return Ok(None);
    }
    if u64_at(8) != 0 || crate::hash::is_ref(u64_at(0)) {
        return Ok(None);
    }
    let (width, height, depth) = (u16_at(0x20), u16_at(0x22), u16_at(0x24));
    if !(1..=16384).contains(&width) || !(1..=16384).contains(&height) || !(1..=4096).contains(&depth) {
        return Ok(None);
    }
    let count = usize::from(h[0x2e]);
    let mut end = at + HEADER_LEN;
    let mut groups = Vec::new();
    if groups_ptr == FOLLOW {
        if count == 0 || count > 20 {
            return Ok(None);
        }
        for i in 0..count {
            let r = end + GROUP_LEN * i;
            groups.push(MipGroup { key: b.u64(r)?, end: (b.u32(r + 20)? & 0x00ff_ffff) * 4 });
        }
        end += GROUP_LEN * count;
    }
    let data_size = u32_at(0x18);
    let mut inline = None;
    if pixels_ptr == FOLLOW {
        b.get(end, data_size as usize)?;
        inline = Some((end, data_size as usize));
        end += data_size as usize;
    }
    Ok(Some(ImageRec {
        at,
        hash: u64_at(0),
        flags: u32_at(0x14),
        data_size,
        width,
        height,
        depth,
        array: u16_at(0x26),
        format: h[0x2a],
        usage: h[0x2b],
        levels: h[0x2c],
        groups,
        inline,
        end,
    }))
}

/// Every image of a zone, in asset order.
#[derive(Clone, Debug, Default)]
pub struct ImageRun {
    /// Header offsets.
    pub at: Vec<usize>,
    /// Block-0x0f offset of the first image's slot, once calibrated.
    pub base: Option<u32>,
}

impl ImageRun {
    /// Find the run of `count` images (the asset list's type-21 count):
    /// the longest chain of image-shaped headers, each starting where the
    /// one before ends. `None` when no chain is that long.
    #[must_use]
    pub fn find(zone: &[u8], count: usize) -> Option<Self> {
        if count == 0 || zone.len() < HEADER_LEN {
            return None;
        }
        // Candidates: offsets whose +0x30/+0x38 pointers are 0 or -2.
        let mut next: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let word = |at: usize| u64::from_le_bytes(zone[at..at + 8].try_into().unwrap_or([0; 8]));
        for at in 0..=zone.len() - HEADER_LEN {
            let (g, p) = (word(at + 0x30), word(at + 0x38));
            if !matches!(g, 0 | FOLLOW) || !matches!(p, 0 | FOLLOW) || (g == 0 && p == 0) {
                continue;
            }
            if let Ok(Some(rec)) = parse(zone, at) {
                next.insert(at, rec.end);
            }
        }
        let has_prev: std::collections::HashSet<usize> = next.values().copied().filter(|v| next.contains_key(v)).collect();
        let mut best: Option<(usize, usize)> = None;
        for &start in next.keys() {
            if has_prev.contains(&start) {
                continue;
            }
            let (mut n, mut c) = (0usize, start);
            while let Some(&e) = next.get(&c) {
                n += 1;
                c = e;
                if n > count {
                    break;
                }
            }
            if n == count && best.is_none_or(|(_, s)| start < s) {
                best = Some((n, start));
            }
        }
        let (_, start) = best?;
        let mut at = Vec::with_capacity(count);
        let mut c = start;
        while at.len() < count {
            at.push(c);
            c = next[&c];
        }
        Some(Self { at, base: None })
    }

    /// The index of the image whose header is at `at`.
    #[must_use]
    pub fn index_of(&self, at: usize) -> Option<usize> {
        self.at.binary_search(&at).ok()
    }

    /// Calibrate the slot base from pointers known to lead to images:
    /// (block-0x0f offset, the image's header offset). The base most of
    /// them agree on is kept; returns how many agreed.
    pub fn calibrate(&mut self, known: &[(u32, usize)]) -> usize {
        let mut votes: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for &(slot, at) in known {
            if let Some(i) = self.index_of(at) {
                let base = slot.wrapping_sub(SLOT * i as u32);
                *votes.entry(base).or_default() += 1;
            }
        }
        let best = votes.into_iter().max_by_key(|&(b, n)| (n, std::cmp::Reverse(b)));
        self.base = best.map(|(b, _)| b);
        best.map_or(0, |(_, n)| n)
    }

    /// The header offset of the image a block-0x0f pointer leads to.
    #[must_use]
    pub fn resolve(&self, slot: u32) -> Option<usize> {
        let base = self.base?;
        let d = slot.checked_sub(base)?;
        if d % SLOT != 0 {
            return None;
        }
        self.at.get((d / SLOT) as usize).copied()
    }
}

/// DXGI formats by an image's format byte (measured by decoding pictures
/// whose content is known; others `None`).
#[must_use]
pub fn dxgi(format: u8) -> Option<u32> {
    match format {
        0x25 | 0x2a => Some(71), // BC1_UNORM (0x25: calling cards; a 1024x1024 one is 512 KiB)
        // (0x2f: guns' normal maps, x y z in red green blue; read as BC5
        // they are noise.)
        0x2f | 0x30 => Some(98), // BC7_UNORM
        _ => None,
    }
}

/// Bytes per 4x4 block (block-compressed formats) for a DXGI format.
#[must_use]
pub fn block_bytes(dxgi: u32) -> Option<usize> {
    match dxgi {
        70..=72 | 79..=81 => Some(8),
        73..=78 | 82..=84 | 94..=99 => Some(16),
        _ => None,
    }
}

/// The byte size of one level.
#[must_use]
pub fn level_bytes(dxgi: u32, w: u32, h: u32) -> Option<usize> {
    let bs = block_bytes(dxgi)?;
    Some((w.max(1).div_ceil(4) * h.max(1).div_ceil(4)) as usize * bs)
}

/// The levels a group holds, largest first: (level, width, height, offset
/// in the group's bytes). Levels whose sizes do not add up to the group's
/// span give `None`.
///
/// The groups' ends count from the smallest streamed level, and how far
/// down the chain is streamed differs by format (a BC7 1024x1024 image's
/// first group is its 8x8 and 4x4 levels, a BC1 one's goes on to 2x2), so
/// the levels are placed from the top: the last group ends with the top
/// level, and each level lies where the levels above it end.
#[must_use]
pub fn group_levels(img: &ImageRec, group: usize, dxgi: u32) -> Option<Vec<(u32, u32, u32, usize)>> {
    let start = if group == 0 { 0 } else { img.groups.get(group - 1)?.end };
    let end = img.groups.get(group)?.end;
    let total = img.groups.last()?.end;
    // The group's span counted from the top level down.
    let (top_from, top_to) = (total.checked_sub(end)?, total.checked_sub(start)?);
    let (w0, h0) = (u32::from(img.width), u32::from(img.height));
    let mut inside = Vec::new();
    let mut at = 0u32;
    let mut l = 0u32;
    loop {
        let (w, h) = ((w0 >> l).max(1), (h0 >> l).max(1));
        let n = level_bytes(dxgi, w, h)? as u32;
        if at >= top_from && at + n <= top_to {
            inside.push((l, w, h, (at - top_from) as usize));
        }
        at += n;
        if at >= top_to || (w == 1 && h == 1) {
            break;
        }
        l += 1;
    }
    let covered: u32 = inside.iter().map(|x| level_bytes(dxgi, x.1, x.2).unwrap_or(0) as u32).sum();
    if inside.is_empty() || covered != end - start || at < top_to {
        return None;
    }
    Some(inside)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(hash: u64, w: u16, h: u16, format: u8, groups: &[(u64, u32)]) -> Vec<u8> {
        let mut v = vec![0u8; HEADER_LEN];
        v[..8].copy_from_slice(&hash.to_le_bytes());
        v[0x14..0x18].copy_from_slice(&0x0814u32.to_le_bytes());
        v[0x20..0x22].copy_from_slice(&w.to_le_bytes());
        v[0x22..0x24].copy_from_slice(&h.to_le_bytes());
        v[0x24..0x26].copy_from_slice(&1u16.to_le_bytes());
        v[0x26..0x28].copy_from_slice(&1u16.to_le_bytes());
        v[0x2a] = format;
        v[0x2e] = groups.len() as u8;
        v[0x30..0x38].copy_from_slice(&FOLLOW.to_le_bytes());
        for &(k, end) in groups {
            let mut r = [0u8; GROUP_LEN];
            r[..8].copy_from_slice(&k.to_le_bytes());
            r[20..24].copy_from_slice(&(end / 4).to_le_bytes());
            v.extend_from_slice(&r);
        }
        v
    }

    #[test]
    fn a_streamed_image_and_its_groups() {
        // 64x64 BC7: levels 64 (4096), 32 (1024), 16 (256), 8 (64), 4 (16)
        // (the measured layout: 8 and 4 in one group, 2 and 1 left out).
        let z = header(0x1234, 64, 64, 0x30, &[(1, 80), (2, 80 + 256 + 1024), (3, 80 + 256 + 1024 + 4096)]);
        let img = parse(&z, 0).unwrap().unwrap();
        assert_eq!(img.groups.len(), 3);
        assert_eq!(img.end, z.len());
        let d = dxgi(img.format).unwrap();
        assert_eq!(group_levels(&img, 2, d).unwrap(), vec![(0, 64, 64, 0)]);
        assert_eq!(group_levels(&img, 1, d).unwrap(), vec![(1, 32, 32, 0), (2, 16, 16, 1024)]);
        // One level, in one group (an emblem's picture).
        let mut z = header(0x55, 256, 256, 0x30, &[(7, 65536)]);
        z[0x2c] = 1;
        let img = parse(&z, 0).unwrap().unwrap();
        assert_eq!(img.levels, 1);
        assert_eq!(group_levels(&img, 0, d).unwrap(), vec![(0, 256, 256, 0)]);
    }

    #[test]
    fn the_run_and_its_slots() {
        let mut z = vec![0xaau8; 5];
        let first = z.len();
        z.extend(header(1, 16, 16, 0x30, &[(9, 1024)]));
        let second = z.len();
        z.extend(header(2, 16, 16, 0x30, &[(10, 1024)]));
        z.extend([0u8; 8]);
        let mut run = ImageRun::find(&z, 2).unwrap();
        assert_eq!(run.at, vec![first, second]);
        assert_eq!(run.calibrate(&[(0x1000 + 16, second)]), 1);
        assert_eq!(run.resolve(0x1000), Some(first));
        assert_eq!(run.resolve(0x1008), None);
    }
}
