//! The `KAPI` packages of streamed data: `zone/*.xsub` (and `.xpak`, same
//! format). One header, a data region of entries on 128-byte boundaries, and
//! a hash table at the end mapping a 64-bit key to an entry.
//!
//! ```text
//! header (0x800 bytes)
//! 0x000 "KAPI"
//! 0x004 u16 1, u16 version 0x17
//! 0x008 u64 1
//! 0x010 u64 package kind (3 in the xsubs seen)
//! 0x018 u64 file length
//! 0x020 u64 hash (the package's own)
//! 0x130..0x250 signature-like bytes (unused here)
//! 0x788 u64 file count, u64 data offset (0x8000), u64 data size,
//!       u64 hash count, u64 hash table offset, u64 hash table size (count * 20)
//! hash table entry (20 bytes, sorted by key):
//!   u64 key
//!   u64 packed: bit 0 = Oodle-compressed (1) or stored (0),
//!               bits 1..31 = size in the file, bits 32..63 = offset >> 7
//!   u32 decompressed size
//! stored entry: the bytes themselves
//! compressed entry:
//!   u16 0xF01D, u64 key (= the table's), u64 0, u32 1, u8 block count
//!   block count x 21 bytes:
//!     u8 compressor (6), u32 compressed length, u32 decompressed length,
//!     u32 offset of the stream from the entry start, u32 offset in the
//!     decoded entry, u32 checksum (not checked here)
//!   the Oodle streams (each a whole Oodle LZ stream; streams 4-aligned;
//!   decoded blocks of 0x3ffe0 bytes, the last shorter)
//! ```
#![forbid(unsafe_code)]

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::oodle::{Decompressor, OodleError};

pub const MAGIC: &[u8; 4] = b"KAPI";
pub const VERSION: u16 = 0x17;
pub const HEADER_LEN: usize = 0x800;
pub const TOC_OFFSET: usize = 0x788;
pub const HASH_ENTRY_LEN: usize = 20;
pub const ENTRY_ALIGN: u64 = 128;
pub const ENTRY_MAGIC: u16 = 0xF01D;
pub const ENTRY_HEAD_LEN: usize = 23;
pub const ENTRY_BLOCK_LEN: usize = 21;

#[derive(Debug)]
pub enum XSubError {
    Io(std::io::Error),
    BadMagic([u8; 4]),
    WrongVersion(u16),
    /// The header's sizes do not fit the file.
    BadToc(&'static str),
    /// An entry's own header or block table is not what its table row says.
    BadEntry {
        key: u64,
        why: &'static str,
    },
    Oodle {
        key: u64,
        block: usize,
        error: OodleError,
    },
}

impl std::fmt::Display for XSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "reading package: {e}"),
            Self::BadMagic(m) => write!(f, "not a KAPI package (magic {m:02x?})"),
            Self::WrongVersion(v) => write!(f, "KAPI version {v:#x}, expected {VERSION:#x}"),
            Self::BadToc(why) => write!(f, "bad package table: {why}"),
            Self::BadEntry { key, why } => write!(f, "entry {key:016x}: {why}"),
            Self::Oodle { key, block, error } => {
                write!(f, "entry {key:016x} block {block}: {error}")
            }
        }
    }
}

impl std::error::Error for XSubError {}

impl From<std::io::Error> for XSubError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(b[at..at + 2].try_into().unwrap())
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XSubHeader {
    /// 1 in the `.xsub` parts (and the xpak_cache tables), 2 in the zone
    /// `.xpak` indexes.
    pub flavor: u16,
    pub version: u16,
    pub kind: u64,
    /// 0 in a part; in a zone `.xpak`, how many `.xsub` parts it indexes.
    pub part_count: u64,
    pub file_len: u64,
    pub hash: u64,
    pub file_count: u64,
    pub data_offset: u64,
    pub data_size: u64,
    pub hash_count: u64,
    pub hash_offset: u64,
    pub hash_size: u64,
}

pub fn parse_header(h: &[u8]) -> Result<XSubHeader, XSubError> {
    if h.len() < HEADER_LEN {
        return Err(XSubError::BadToc("shorter than the header"));
    }
    let magic: [u8; 4] = h[..4].try_into().unwrap();
    if &magic != MAGIC {
        return Err(XSubError::BadMagic(magic));
    }
    let version = u16_at(h, 6);
    if version != VERSION {
        return Err(XSubError::WrongVersion(version));
    }
    let t = TOC_OFFSET;
    let hdr = XSubHeader {
        flavor: u16_at(h, 4),
        version,
        kind: u64_at(h, 0x10),
        part_count: u64_at(h, 0x28),
        file_len: u64_at(h, 0x18),
        hash: u64_at(h, 0x20),
        file_count: u64_at(h, t),
        data_offset: u64_at(h, t + 8),
        data_size: u64_at(h, t + 16),
        hash_count: u64_at(h, t + 24),
        hash_offset: u64_at(h, t + 32),
        hash_size: u64_at(h, t + 40),
    };
    if hdr.hash_size != hdr.hash_count * HASH_ENTRY_LEN as u64 {
        return Err(XSubError::BadToc("hash table size is not count * 20"));
    }
    Ok(hdr)
}

/// One row of the hash table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XSubEntry {
    pub key: u64,
    pub offset: u64,
    /// Bytes in the file.
    pub size: u32,
    pub decompressed: u32,
    pub compressed: bool,
}

impl XSubEntry {
    pub fn from_row(row: &[u8]) -> Self {
        let key = u64_at(row, 0);
        let packed = u64_at(row, 8);
        Self {
            key,
            offset: (packed >> 32) * ENTRY_ALIGN,
            size: ((packed >> 1) & 0x7fff_ffff) as u32,
            decompressed: u32_at(row, 16),
            compressed: packed & 1 != 0,
        }
    }
}

/// One Oodle stream of a compressed entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryBlock {
    pub compressor: u8,
    pub compressed: u32,
    pub decompressed: u32,
    pub src_offset: u32,
    pub dst_offset: u32,
    pub checksum: u32,
}

/// Parse a compressed entry's header and block table.
pub fn parse_entry_blocks(entry: &XSubEntry, blob: &[u8]) -> Result<Vec<EntryBlock>, XSubError> {
    let bad = |why| XSubError::BadEntry {
        key: entry.key,
        why,
    };
    if blob.len() < ENTRY_HEAD_LEN {
        return Err(bad("shorter than its header"));
    }
    if u16_at(blob, 0) != ENTRY_MAGIC {
        return Err(bad("no 0xF01D entry magic"));
    }
    if u64_at(blob, 2) != entry.key {
        return Err(bad("entry key differs from the table's"));
    }
    let count = blob[22] as usize;
    let table_end = ENTRY_HEAD_LEN + count * ENTRY_BLOCK_LEN;
    if count == 0 || blob.len() < table_end {
        return Err(bad("block table missing"));
    }
    let mut blocks = Vec::with_capacity(count);
    let mut next_dst = 0u32;
    for i in 0..count {
        let r = ENTRY_HEAD_LEN + i * ENTRY_BLOCK_LEN;
        let b = EntryBlock {
            compressor: blob[r],
            compressed: u32_at(blob, r + 1),
            decompressed: u32_at(blob, r + 5),
            src_offset: u32_at(blob, r + 9),
            dst_offset: u32_at(blob, r + 13),
            checksum: u32_at(blob, r + 17),
        };
        if (b.src_offset as usize) < table_end
            || b.src_offset as usize + b.compressed as usize > blob.len()
        {
            return Err(bad("block stream outside the entry"));
        }
        if b.dst_offset != next_dst {
            return Err(bad("decoded block offsets are not contiguous"));
        }
        next_dst = next_dst
            .checked_add(b.decompressed)
            .ok_or(bad("decoded size overflows"))?;
        blocks.push(b);
    }
    if next_dst != entry.decompressed {
        return Err(bad("blocks do not add up to the decompressed size"));
    }
    Ok(blocks)
}

/// Decode an entry's bytes as read from the file.
pub fn decode_entry(
    entry: &XSubEntry,
    blob: &[u8],
    oodle: &dyn Decompressor,
) -> Result<Vec<u8>, XSubError> {
    if !entry.compressed {
        if blob.len() != entry.decompressed as usize {
            return Err(XSubError::BadEntry {
                key: entry.key,
                why: "stored entry size differs from its decoded size",
            });
        }
        return Ok(blob.to_vec());
    }
    let blocks = parse_entry_blocks(entry, blob)?;
    let mut out = vec![0u8; entry.decompressed as usize];
    for (i, b) in blocks.iter().enumerate() {
        let src = &blob[b.src_offset as usize..(b.src_offset + b.compressed) as usize];
        let dst = &mut out[b.dst_offset as usize..(b.dst_offset + b.decompressed) as usize];
        oodle
            .decompress(src, dst)
            .map_err(|error| XSubError::Oodle {
                key: entry.key,
                block: i,
                error,
            })?;
    }
    Ok(out)
}

/// An open package: header, hash table, and the file to fetch entries from.
pub struct XSub<R> {
    pub header: XSubHeader,
    /// Sorted by key, as stored.
    pub entries: Vec<XSubEntry>,
    reader: R,
}

impl XSub<std::fs::File> {
    /// Open read-only.
    pub fn open(path: &Path) -> Result<Self, XSubError> {
        Self::from_reader(std::fs::File::open(path)?)
    }
}

/// Read a package's header and hash table without checking where the rows
/// point (a zone `.xpak`'s rows point into its `.xsub` parts).
pub fn read_index<R: Read + Seek>(
    reader: &mut R,
) -> Result<(XSubHeader, Vec<XSubEntry>), XSubError> {
    let mut h = vec![0u8; HEADER_LEN];
    reader.seek(SeekFrom::Start(0))?;
    reader.read_exact(&mut h)?;
    let header = parse_header(&h)?;
    let actual = reader.seek(SeekFrom::End(0))?;
    if header.hash_offset + header.hash_size > actual {
        return Err(XSubError::BadToc("hash table runs past the file"));
    }
    let mut table = vec![0u8; header.hash_size as usize];
    reader.seek(SeekFrom::Start(header.hash_offset))?;
    reader.read_exact(&mut table)?;
    let entries = table
        .chunks_exact(HASH_ENTRY_LEN)
        .map(XSubEntry::from_row)
        .collect();
    Ok((header, entries))
}

impl<R: Read + Seek> XSub<R> {
    pub fn from_reader(mut reader: R) -> Result<Self, XSubError> {
        let (header, entries) = read_index(&mut reader)?;
        let actual = reader.seek(SeekFrom::End(0))?;
        if actual != header.file_len {
            return Err(XSubError::BadToc("file length differs from the header's"));
        }
        for e in &entries {
            if e.offset < header.data_offset || e.offset + e.size as u64 > header.file_len {
                return Err(XSubError::BadToc("an entry lies outside the data"));
            }
        }
        Ok(Self {
            header,
            entries,
            reader,
        })
    }

    /// The table is sorted by key: binary search.
    pub fn find(&self, key: u64) -> Option<&XSubEntry> {
        self.entries
            .binary_search_by_key(&key, |e| e.key)
            .ok()
            .map(|i| &self.entries[i])
    }

    /// The entry's bytes as stored.
    pub fn read_raw(&mut self, entry: &XSubEntry) -> Result<Vec<u8>, XSubError> {
        let mut blob = vec![0u8; entry.size as usize];
        self.reader.seek(SeekFrom::Start(entry.offset))?;
        self.reader.read_exact(&mut blob)?;
        Ok(blob)
    }

    /// The entry's bytes, decompressed.
    pub fn read(
        &mut self,
        entry: &XSubEntry,
        oodle: &dyn Decompressor,
    ) -> Result<Vec<u8>, XSubError> {
        let blob = self.read_raw(entry)?;
        decode_entry(entry, &blob, oodle)
    }
}

/// A zone `.xpak`: one hash table over all of its `.xsub` parts
/// (`<name>-00000.xsub`, ...). A row's offset is
/// `part * stride + offset in the part`, with the stride the smallest power
/// of two that holds the largest part (256 MiB for `boot`, 1 GiB for
/// `codhq23a`); the sizes and the decoded size are the part's own. Checked
/// by key against every row of `boot` and `codhq23a`.
pub struct XPak {
    pub header: XSubHeader,
    pub entries: Vec<XSubEntry>,
    pub parts: Vec<std::path::PathBuf>,
    pub stride: u64,
}

impl XPak {
    pub fn open(path: &Path) -> Result<Self, XSubError> {
        let (header, entries) = read_index(&mut std::fs::File::open(path)?)?;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();
        let dir = path.parent().unwrap_or(Path::new("."));
        let mut parts = Vec::new();
        let mut largest = 0u64;
        for i in 0..header.part_count {
            let p = dir.join(format!("{stem}-{i:05}.xsub"));
            largest = largest.max(std::fs::metadata(&p)?.len());
            parts.push(p);
        }
        let stride = largest.next_power_of_two();
        Ok(Self {
            header,
            entries,
            parts,
            stride,
        })
    }

    pub fn find(&self, key: u64) -> Option<&XSubEntry> {
        self.entries
            .binary_search_by_key(&key, |e| e.key)
            .ok()
            .map(|i| &self.entries[i])
    }

    /// Which part an entry is in, and the entry as that part's table has it.
    pub fn locate(&self, entry: &XSubEntry) -> Option<(usize, XSubEntry)> {
        if self.stride == 0 {
            return None;
        }
        let part = (entry.offset / self.stride) as usize;
        (part < self.parts.len()).then(|| {
            (
                part,
                XSubEntry {
                    offset: entry.offset % self.stride,
                    ..*entry
                },
            )
        })
    }

    pub fn read(&self, entry: &XSubEntry, oodle: &dyn Decompressor) -> Result<Vec<u8>, XSubError> {
        let (part, local) = self.locate(entry).ok_or(XSubError::BadEntry {
            key: entry.key,
            why: "offset past the last part",
        })?;
        let mut f = std::fs::File::open(&self.parts[part])?;
        let mut blob = vec![0u8; local.size as usize];
        f.seek(SeekFrom::Start(local.offset))?;
        f.read_exact(&mut blob)?;
        decode_entry(&local, &blob, oodle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oodle::StoredOnly;
    use std::io::Cursor;

    /// A compressed entry whose blocks are Oodle stored chunks.
    fn synth_entry(key: u64, data: &[u8], block: usize) -> Vec<u8> {
        let chunks: Vec<&[u8]> = data.chunks(block).collect();
        let mut e = Vec::new();
        e.extend_from_slice(&ENTRY_MAGIC.to_le_bytes());
        e.extend_from_slice(&key.to_le_bytes());
        e.extend_from_slice(&0u64.to_le_bytes());
        e.extend_from_slice(&1u32.to_le_bytes());
        e.push(chunks.len() as u8);
        let mut src = ENTRY_HEAD_LEN + chunks.len() * ENTRY_BLOCK_LEN;
        let mut dst = 0;
        for c in &chunks {
            src = (src + 3) & !3;
            e.push(6);
            e.extend_from_slice(&(c.len() as u32 + 2).to_le_bytes());
            e.extend_from_slice(&(c.len() as u32).to_le_bytes());
            e.extend_from_slice(&(src as u32).to_le_bytes());
            e.extend_from_slice(&(dst as u32).to_le_bytes());
            e.extend_from_slice(&0xdead_beefu32.to_le_bytes());
            src += c.len() + 2;
            dst += c.len();
        }
        for c in &chunks {
            while !e.len().is_multiple_of(4) {
                e.push(0);
            }
            e.extend_from_slice(&[0xCC, 0x06]);
            e.extend_from_slice(c);
        }
        e
    }

    fn synth_pak(entries: &[(u64, Vec<u8>, bool, u32)]) -> Vec<u8> {
        let mut f = vec![0u8; 0x8000];
        let mut rows = Vec::new();
        for (key, blob, compressed, dec) in entries {
            while !(f.len() as u64).is_multiple_of(ENTRY_ALIGN) {
                f.push(0);
            }
            let packed = ((f.len() as u64 / ENTRY_ALIGN) << 32)
                | ((blob.len() as u64) << 1)
                | *compressed as u64;
            rows.push((*key, packed, *dec));
            f.extend_from_slice(blob);
        }
        rows.sort();
        let data_size = f.len() as u64 - 0x8000;
        while !f.len().is_multiple_of(0x8000) {
            f.push(0);
        }
        let hash_offset = f.len() as u64;
        for (k, p, d) in &rows {
            f.extend_from_slice(&k.to_le_bytes());
            f.extend_from_slice(&p.to_le_bytes());
            f.extend_from_slice(&d.to_le_bytes());
        }
        let len = f.len() as u64;
        f[..4].copy_from_slice(MAGIC);
        f[4..6].copy_from_slice(&1u16.to_le_bytes());
        f[6..8].copy_from_slice(&VERSION.to_le_bytes());
        f[0x10..0x18].copy_from_slice(&3u64.to_le_bytes());
        f[0x18..0x20].copy_from_slice(&len.to_le_bytes());
        let toc = [
            rows.len() as u64,
            0x8000,
            data_size,
            rows.len() as u64,
            hash_offset,
            rows.len() as u64 * 20,
        ];
        for (i, v) in toc.iter().enumerate() {
            f[TOC_OFFSET + i * 8..TOC_OFFSET + i * 8 + 8].copy_from_slice(&v.to_le_bytes());
        }
        f
    }

    #[test]
    fn reads_stored_and_compressed_entries() {
        let big: Vec<u8> = (0..5000u32).map(|i| (i % 253) as u8).collect();
        let small = b"just some bytes".to_vec();
        let pak = synth_pak(&[
            (
                0x00aa_0000_0000_0001,
                synth_entry(0x00aa_0000_0000_0001, &big, 1999),
                true,
                big.len() as u32,
            ),
            (
                0x0011_0000_0000_0002,
                small.clone(),
                false,
                small.len() as u32,
            ),
        ]);
        let mut x = XSub::from_reader(Cursor::new(pak)).unwrap();
        assert_eq!(x.entries.len(), 2);
        assert!(x.entries[0].key < x.entries[1].key);
        let e = *x.find(0x00aa_0000_0000_0001).unwrap();
        assert!(e.compressed && e.offset.is_multiple_of(ENTRY_ALIGN));
        assert_eq!(x.read(&e, &StoredOnly).unwrap(), big);
        let e = *x.find(0x0011_0000_0000_0002).unwrap();
        assert!(!e.compressed);
        assert_eq!(x.read(&e, &StoredOnly).unwrap(), small);
        assert!(x.find(7).is_none());
    }

    #[test]
    fn rejects_mismatched_entries() {
        let data = vec![9u8; 300];
        let blob = synth_entry(5, &data, 128);
        let e = XSubEntry {
            key: 6,
            offset: 0x8000,
            size: blob.len() as u32,
            decompressed: 300,
            compressed: true,
        };
        assert!(
            matches!(parse_entry_blocks(&e, &blob), Err(XSubError::BadEntry { why, .. }) if why.contains("key"))
        );
        let e = XSubEntry {
            key: 5,
            decompressed: 301,
            ..e
        };
        assert!(
            matches!(parse_entry_blocks(&e, &blob), Err(XSubError::BadEntry { why, .. }) if why.contains("add up"))
        );
        let e = XSubEntry {
            decompressed: 300,
            ..e
        };
        assert_eq!(parse_entry_blocks(&e, &blob).unwrap().len(), 3);
    }

    #[test]
    fn rejects_bad_headers() {
        let mut pak = synth_pak(&[(1, vec![1, 2, 3], false, 3)]);
        pak[6] = 0x16;
        assert!(matches!(
            XSub::from_reader(Cursor::new(pak)),
            Err(XSubError::WrongVersion(0x16))
        ));
        let pak = vec![0u8; 0x900];
        assert!(matches!(
            XSub::from_reader(Cursor::new(pak)),
            Err(XSubError::BadMagic(_))
        ));
    }

    #[test]
    fn xpak_rows_split_into_part_and_offset() {
        // An xpak row's packed word is the part's row with the part number
        // added above the stride (boot: 256 MiB per part).
        let mut row = [0u8; 20];
        let part_packed: u64 = 0x0000_c584_0000_9e2d;
        let xpak_packed = part_packed + ((1u64 << 28) >> 7 << 32);
        assert_eq!(xpak_packed, 0x0020_c584_0000_9e2d);
        row[..8].copy_from_slice(&0x0017_d17b_8c2b_9d65_u64.to_le_bytes());
        row[8..16].copy_from_slice(&xpak_packed.to_le_bytes());
        row[16..].copy_from_slice(&0x8000u32.to_le_bytes());
        let e = XSubEntry::from_row(&row);
        let pak = XPak {
            header: parse_header(&synth_pak(&[])).unwrap(),
            entries: vec![e],
            parts: vec!["a".into(), "b".into(), "c".into()],
            stride: 1 << 28,
        };
        let (part, local) = pak.locate(&e).unwrap();
        assert_eq!(part, 1);
        assert_eq!(local.offset, 0xc584 * ENTRY_ALIGN);
        assert_eq!(local.size, 0x9e2d >> 1);
        assert!(local.compressed);
        let far = XSubEntry {
            offset: 3 << 28,
            ..e
        };
        assert!(pak.locate(&far).is_none());
    }
}
