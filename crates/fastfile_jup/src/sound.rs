//! Sounds inside a decompressed zone: the `sndasset` records (type 193),
//! the sound aliases that point at them, and the framing of the streamed
//! audio they name.
//!
//! ```text
//! sndasset (0x58 bytes, then its preload inline when it has one)
//! 0x00 u64 name hash
//! 0x08 u64 0
//! 0x10 u64 key of a loaded sound  } one of the two: the row of a KAPI
//! 0x18 u64 key of a streamed one  } package (.xsub) holding the stream
//! 0x20 u64 stream length     = seek table + packet data (streamed; 0 loaded)
//! 0x28 ptr preload           -2: follows this header inline; 0: none
//! 0x30 u64 preload length    the first bytes of the packet data
//! 0x38 u32 seek table length
//! 0x3c u32 packet data length
//! 0x40 u32 frames (samples per channel)
//! 0x44 u32 sample rate (48000 in every zone read)
//! 0x48 u32 checksum-like
//! 0x4c u8 flags (streamed: 0x10 with a preload, 0x08 without; loaded: 0, 1, 4)
//!      u8 format (5 = Opus; 2 on four long stereo 32 kHz sounds of
//!      ingame_br, which have no seek table: not read)
//!      u8 0, u8 channels
//! 0x50 u8[8] small per-sound numbers (unread)
//!
//! stream = u32 offset per packet (the seek table)
//!          then per packet: u16 length, the Opus packet (20 ms CELT/SILK frames)
//! ```
//!
//! A sound alias names its sound by pointing at the sndasset's entry in the
//! zone's asset list, as an encoded block-15 pointer
//! `0x0F_0000_0000 | (base + 16 * asset index) + 1`, the base being 8 plus
//! the script-string table's bytes (pointers and strings, as they are in
//! the zone image) rounded up to 8. The 16 bytes before that pointer are the
//! alias's name hash and (likely) its secondary alias's (0: none); a u32
//! 0x3FFFFFFF follows it. Alias names are FNV-1a
//! 64-bit hashes of the alias as written (offset basis 0xCBF29CE484222325,
//! no mask) — not the asset-name hash of `hash.rs`.
//!
//! Checked on eng_mp_jup_escape5, eng_ingame_br and ingame_br of build
//! 1.21: every type-193 asset of the asset list has its record, in order.
#![forbid(unsafe_code)]

use std::ops::Range;

pub const SNDASSET_LEN: usize = 0x58;
/// The sndasset `format` byte of an Opus stream.
pub const FORMAT_OPUS: u8 = 5;
pub const ASSET_SNDASSET: u64 = 193;
/// Sound alias name hash: FNV-1a 64 over the bytes as written.
pub const ALIAS_HASH_OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
/// Encoded pointers into the zone's block 15.
const BLOCK15: u64 = 0x0F;

/// The alias name hash of `name` (lower-case names, as the game writes them).
pub fn alias_hash(name: &str) -> u64 {
    let mut h = ALIAS_HASH_OFFSET;
    for b in name.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}

/// The zone header's asset list: each asset's type, and where the asset
/// data starts. (Minimal: enough to number the sndassets.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetList {
    pub types: Vec<u64>,
    pub data_start: usize,
    /// Where the asset list is in block 15: after a u64, the script-string
    /// pointers and the strings, on 8 bytes (so 8 in a zone without script
    /// strings; 0x1d968 in ingame_br, 0x15650 in mp_jup_escape5).
    pub block15_base: u64,
}

const FOLLOWS: [u64; 2] = [u64::MAX, u64::MAX - 1];

/// Read the decompressed zone's header, script-string table and asset list.
pub fn asset_list(zone: &[u8]) -> Option<AssetList> {
    if zone.len() < 0x20 {
        return None;
    }
    let (strings, strings_ptr) = (u64_at(zone, 0), u64_at(zone, 8));
    let (count, assets_ptr) = (u64_at(zone, 0x10), u64_at(zone, 0x18));
    let mut at = 0x20usize;
    let mut block15_base = 8u64;
    if FOLLOWS.contains(&strings_ptr) {
        let n = usize::try_from(strings).ok()?;
        let table = zone.get(at..at.checked_add(n.checked_mul(8)?)?)?;
        at += n * 8;
        for i in 0..n {
            if FOLLOWS.contains(&u64_at(table, i * 8)) {
                let end = zone.get(at..)?.iter().position(|&c| c == 0)?;
                at += end + 1;
            }
        }
        // ...rounded up to 8 bytes.
        block15_base = (block15_base + (at - 0x20) as u64 + 7) & !7;
    }
    if !FOLLOWS.contains(&assets_ptr) {
        return None;
    }
    let n = usize::try_from(count).ok()?;
    let list = zone.get(at..at.checked_add(n.checked_mul(16)?)?)?;
    let types = (0..n).map(|i| u64_at(list, i * 16)).collect();
    Some(AssetList {
        types,
        data_start: at + n * 16,
        block15_base,
    })
}

/// One sndasset record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SndAsset {
    /// Where its header is in the zone.
    pub offset: usize,
    pub name: u64,
    pub key: u64,
    pub stream_len: u64,
    /// The preload's bytes in the zone (the first bytes of the packet data).
    pub preload: Option<Range<usize>>,
    pub table_len: u32,
    pub data_len: u32,
    pub frames: u32,
    pub rate: u32,
    pub flags: u8,
    pub format: u8,
    pub channels: u8,
}

impl SndAsset {
    /// Parse a record at `at`, if it is one (zeros, lengths that add up, a
    /// known rate, a preload that fits).
    pub fn parse(zone: &[u8], at: usize) -> Option<Self> {
        let h = zone.get(at..at.checked_add(SNDASSET_LEN)?)?;
        // The key is at 0x18 (streamed sounds) or 0x10 (loaded ones), never both.
        let key = match (u64_at(h, 8), u64_at(h, 0x10), u64_at(h, 0x18)) {
            (0, 0, k) | (0, k, 0) if k != 0 => k,
            _ => return None,
        };
        let rate = u32_at(h, 0x44);
        if !matches!(
            rate,
            8000 | 11025 | 16000 | 22050 | 24000 | 32000 | 44100 | 48000
        ) {
            return None;
        }
        let (table_len, data_len) = (u32_at(h, 0x38), u32_at(h, 0x3c));
        let stream_len = u64_at(h, 0x20);
        // (0 for a loaded sound.)
        let format = h[0x4d];
        // (No seek table: the few format-2 sounds of ingame_br.)
        if (table_len == 0 && format == FORMAT_OPUS)
            || (stream_len != 0 && stream_len != u64::from(table_len) + u64::from(data_len))
        {
            return None;
        }
        let channels = h[0x4f];
        if channels == 0 || channels > 8 {
            return None;
        }
        let preload = match u64_at(h, 0x28) {
            0 => None,
            p if FOLLOWS.contains(&p) => {
                let n = usize::try_from(u64_at(h, 0x30)).ok()?;
                if n as u64 > u64::from(data_len) {
                    return None;
                }
                let start = at + SNDASSET_LEN;
                zone.get(start..start + n)?;
                Some(start..start + n)
            }
            _ => return None,
        };
        Some(Self {
            offset: at,
            name: u64_at(h, 0),
            key,
            stream_len,
            preload,
            table_len,
            data_len,
            frames: u32_at(h, 0x40),
            rate,
            flags: h[0x4c],
            format,
            channels,
        })
    }

    /// The bytes after its header (and preload).
    pub fn end(&self) -> usize {
        self.preload
            .as_ref()
            .map_or(self.offset + SNDASSET_LEN, |p| p.end)
    }

    pub fn seconds(&self) -> f32 {
        self.frames as f32 / self.rate.max(1) as f32
    }

    /// The whole stream is in the zone (no package read needed).
    pub fn fully_preloaded(&self) -> bool {
        self.preload
            .as_ref()
            .is_some_and(|p| p.len() as u64 >= u64::from(self.data_len))
    }
}

/// Every sndasset record of the zone, in zone order, found by its shape
/// from `from` on (they are packed between other assets' data).
pub fn find_sndassets(zone: &[u8], from: usize) -> Vec<SndAsset> {
    let mut out = Vec::new();
    let mut at = from;
    // The rate is at +0x44; scan for it and check the record around it.
    while at + SNDASSET_LEN <= zone.len() {
        let rate = u32_at(zone, at + 0x44);
        if matches!(
            rate,
            48000 | 44100 | 24000 | 32000 | 22050 | 16000 | 11025 | 8000
        ) && let Some(s) = SndAsset::parse(zone, at)
        {
            at = s.end();
            out.push(s);
            continue;
        }
        at += 1;
    }
    out
}

/// A sound alias's link to a sndasset of the same zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AliasLink {
    pub alias: u64,
    /// Index among the zone's sndassets (in asset-list order).
    pub sndasset: usize,
    /// Where the pointer is in the zone.
    pub at: usize,
}

/// The alias -> sndasset links of a zone: every encoded block-15 pointer
/// that lands on a type-193 entry of the asset list and is followed by
/// 0x3FFFFFFF, with the alias hash 16 bytes before it.
pub fn alias_links(zone: &[u8], list: &AssetList) -> Vec<AliasLink> {
    let mut ordinal = vec![usize::MAX; list.types.len()];
    let mut n = 0usize;
    for (i, &t) in list.types.iter().enumerate() {
        if t == ASSET_SNDASSET {
            ordinal[i] = n;
            n += 1;
        }
    }
    let mut out = Vec::new();
    let mut at = list.data_start.max(16);
    while at + 8 <= zone.len() {
        // The block byte is at +4 of the pointer.
        if zone[at + 4] != BLOCK15 as u8 || zone[at + 5..at + 8] != [0, 0, 0] {
            at += 1;
            continue;
        }
        let v = u64_at(zone, at);
        let off = (v & 0xffff_ffff).wrapping_sub(1);
        if v >> 32 == BLOCK15
            && off >= list.block15_base
            && (off - list.block15_base).is_multiple_of(16)
            && zone.get(at + 8..at + 12) == Some(&[0xff, 0xff, 0xff, 0x3f][..])
        {
            let idx = ((off - list.block15_base) / 16) as usize;
            if let Some(&o) = ordinal.get(idx)
                && o != usize::MAX
            {
                out.push(AliasLink {
                    alias: u64_at(zone, at - 16),
                    sndasset: o,
                    at,
                });
                at += 8;
                continue;
            }
        }
        at += 1;
    }
    out
}

/// The Opus packets of a stream (`seek table` + `u16 length, packet`...).
/// Stops at the first length that does not fit.
pub fn packets(stream: &[u8], table_len: usize) -> impl Iterator<Item = &[u8]> {
    let mut at = table_len;
    std::iter::from_fn(move || {
        let len = usize::from(u16::from_le_bytes(stream.get(at..at + 2)?.try_into().ok()?));
        let p = stream.get(at + 2..at + 2 + len)?;
        if len == 0 {
            return None;
        }
        at += 2 + len;
        Some(p)
    })
}

/// The packets of the preload alone (packet data without a seek table).
pub fn preload_packets(preload: &[u8]) -> impl Iterator<Item = &[u8]> {
    packets(preload, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_hash_is_plain_fnv1a() {
        assert_eq!(alias_hash(""), ALIAS_HASH_OFFSET);
        // FNV-1a 64 test vector.
        assert_eq!(alias_hash("a"), 0xaf63_dc4c_8601_ec8c);
    }

    fn record(preload: &[u8]) -> Vec<u8> {
        let mut r = vec![0u8; SNDASSET_LEN];
        r[0..8].copy_from_slice(&0x1111u64.to_le_bytes());
        r[0x18..0x20].copy_from_slice(&0x2222u64.to_le_bytes());
        r[0x20..0x28].copy_from_slice(&(8u64 + 100).to_le_bytes());
        if !preload.is_empty() {
            r[0x28..0x30].copy_from_slice(&(u64::MAX - 1).to_le_bytes());
            r[0x30..0x38].copy_from_slice(&(preload.len() as u64).to_le_bytes());
        }
        r[0x38..0x3c].copy_from_slice(&8u32.to_le_bytes());
        r[0x3c..0x40].copy_from_slice(&100u32.to_le_bytes());
        r[0x40..0x44].copy_from_slice(&960u32.to_le_bytes());
        r[0x44..0x48].copy_from_slice(&48000u32.to_le_bytes());
        r[0x4c] = 0x10;
        r[0x4d] = FORMAT_OPUS;
        r[0x4f] = 1;
        r.extend_from_slice(preload);
        r
    }

    #[test]
    fn finds_records_and_skips_preloads() {
        let mut z = vec![0xAAu8; 13];
        z.extend(record(&[3, 0, 0xf8, 0xff, 0xfe]));
        z.extend([0x55u8; 7]);
        z.extend(record(&[]));
        let s = find_sndassets(&z, 0);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].offset, 13);
        assert_eq!(s[0].preload, Some(13 + SNDASSET_LEN..13 + SNDASSET_LEN + 5));
        assert_eq!(s[1].preload, None);
        assert_eq!((s[1].key, s[1].frames, s[1].channels), (0x2222, 960, 1));
        let pkts: Vec<&[u8]> = preload_packets(&z[s[0].preload.clone().unwrap()]).collect();
        assert_eq!(pkts, vec![&[0xf8u8, 0xff, 0xfe][..]]);
    }

    #[test]
    fn links_point_at_the_asset_list() {
        // assets: localize, sndasset, sndasset
        let mut z = vec![0u8; 0x20];
        z[0x10..0x18].copy_from_slice(&3u64.to_le_bytes());
        z[0x18..0x20].copy_from_slice(&(u64::MAX - 1).to_le_bytes());
        for t in [59u64, 193, 193] {
            z.extend(t.to_le_bytes());
            z.extend((u64::MAX - 1).to_le_bytes());
        }
        let list = asset_list(&z).unwrap();
        assert_eq!(list.types, vec![59, 193, 193]);
        z.extend(0xABCDu64.to_le_bytes());
        z.extend(0u64.to_le_bytes());
        let ptr = (BLOCK15 << 32) | (8 + 16 * 2 + 1);
        z.extend(ptr.to_le_bytes());
        z.extend(0x3fff_ffffu32.to_le_bytes());
        z.extend(0u32.to_le_bytes());
        let l = alias_links(&z, &list);
        assert_eq!(l.len(), 1);
        assert_eq!((l[0].alias, l[0].sndasset), (0xABCD, 1));
    }
}
