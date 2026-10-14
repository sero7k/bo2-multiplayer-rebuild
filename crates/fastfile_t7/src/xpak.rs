//! Black Ops III streamed data packages (`.xpak`): the pixels of streamed
//! images and the vertex / index data of streamed meshes, looked up by the
//! 64-bit key the zone stores (`walk::image::XPakEntry`, the mesh's
//! `XPakEntryInfo`).
//!
//! Layout (read on the retail `base.xpak`, `core_common.xpak`; x64-zt's
//! `t7/common/xpak.cpp` names the header fields):
//!
//! ```text
//! header 0x78: 0x00 u32 magic "KAPI"; 0x06 u16 version (10); 0x10 u64 file size;
//!              0x18 u64 entry count; 0x20 u64 data offset; 0x28 u64 data size;
//!              0x30 u64 hash count; 0x38 u64 hash offset; 0x40 u64 hash size;
//!              0x50 u64 offset of a section with no entries (= index offset);
//!              0x60 u64 index count; 0x68 u64 index offset; 0x70 u64 index size
//! hash entry 24: u64 key; u64 offset (from the data offset); u64 size:56 | flags:8
//! entry data:    blocks of up to 30 chunks: a 0x80-byte block header
//!                { u32 count; u32 offset (0 so far); u32 commands[30] } then
//!                the chunks, command = size:24 | codec:8 (0 stored, 3 LZ4 to
//!                at most 64 KiB, 0xcf filler with no output: a one-chunk block
//!                that pads the next block to a 64 KiB boundary of the file);
//!                each block padded with 0x93 bytes to the next 0x80 (the
//!                entry's stored size is a multiple of 0x80)
//! ```
//!
//! Only byte ranges are read (the header, the hash table, one entry's
//! range), never a whole package.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const MAGIC: u32 = u32::from_le_bytes(*b"KAPI");
pub const HEADER_LEN: usize = 0x78;
pub const HASH_ENTRY_LEN: usize = 24;
pub const BLOCK_HEADER_LEN: usize = 0x80;
pub const BLOCK_COMMANDS: usize = 30;
/// A chunk decompresses to at most this many bytes.
pub const CHUNK_MAX: usize = 0x10000;

#[derive(Debug)]
pub enum XpakError {
    Io(std::io::Error),
    Magic(u32),
    /// A hash table larger than the file.
    Table,
    NotFound(u64),
    /// A block header or chunk that does not fit, or an unknown codec.
    Block { at: usize, what: &'static str },
    Lz4 { at: usize, error: String },
    Size { want: usize, got: usize },
}

impl std::fmt::Display for XpakError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "xpak: {e}"),
            Self::Magic(m) => write!(f, "xpak: bad magic {m:#010x}"),
            Self::Table => write!(f, "xpak: hash table outside the file"),
            Self::NotFound(k) => write!(f, "xpak: no entry {k:#018x}"),
            Self::Block { at, what } => write!(f, "xpak: entry data at {at:#x}: {what}"),
            Self::Lz4 { at, error } => write!(f, "xpak: LZ4 chunk at {at:#x}: {error}"),
            Self::Size { want, got } => write!(f, "xpak: entry decodes to {got:#x} bytes, expected {want:#x}"),
        }
    }
}

impl std::error::Error for XpakError {}

impl From<std::io::Error> for XpakError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XpakHeader {
    pub version: u16,
    pub file_size: u64,
    pub entry_count: u64,
    pub data_offset: u64,
    pub data_size: u64,
    pub hash_count: u64,
    pub hash_offset: u64,
    pub hash_size: u64,
    pub index_count: u64,
    pub index_offset: u64,
    pub index_size: u64,
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap_or([0; 8]))
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap_or([0; 4]))
}

impl XpakHeader {
    pub fn parse(b: &[u8]) -> Result<Self, XpakError> {
        if b.len() < HEADER_LEN {
            return Err(XpakError::Block { at: 0, what: "short header" });
        }
        let magic = u32_at(b, 0);
        if magic != MAGIC {
            return Err(XpakError::Magic(magic));
        }
        Ok(Self {
            version: u16::from_le_bytes([b[6], b[7]]),
            file_size: u64_at(b, 0x10),
            entry_count: u64_at(b, 0x18),
            data_offset: u64_at(b, 0x20),
            data_size: u64_at(b, 0x28),
            hash_count: u64_at(b, 0x30),
            hash_offset: u64_at(b, 0x38),
            hash_size: u64_at(b, 0x40),
            index_count: u64_at(b, 0x60),
            index_offset: u64_at(b, 0x68),
            index_size: u64_at(b, 0x70),
        })
    }
}

/// One hash table entry: where a key's data is in the package.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XpakEntry {
    pub key: u64,
    /// Absolute file offset of the entry's data.
    pub offset: u64,
    /// Bytes in the file (block headers and chunks).
    pub size: u64,
    /// The size word's top byte (0x80 on some entries of the newer packages).
    pub flags: u8,
}

/// An opened package: its header and hash table (sorted by key).
#[derive(Clone, Debug)]
pub struct Xpak {
    pub path: PathBuf,
    pub header: XpakHeader,
    pub entries: Vec<XpakEntry>,
}

impl Xpak {
    /// Reads the header and the hash table only.
    pub fn open(path: &Path) -> Result<Self, XpakError> {
        let mut f = File::open(path)?;
        let mut h = [0u8; HEADER_LEN];
        f.read_exact(&mut h)?;
        let header = XpakHeader::parse(&h)?;
        let len = f.metadata()?.len();
        let table_len = header.hash_count.checked_mul(HASH_ENTRY_LEN as u64).ok_or(XpakError::Table)?;
        if header.hash_offset.checked_add(table_len).is_none_or(|end| end > len) {
            return Err(XpakError::Table);
        }
        f.seek(SeekFrom::Start(header.hash_offset))?;
        let mut table = vec![0u8; table_len as usize];
        f.read_exact(&mut table)?;
        let mut entries: Vec<XpakEntry> = table
            .chunks_exact(HASH_ENTRY_LEN)
            .map(|e| {
                let size = u64_at(e, 16);
                XpakEntry {
                    key: u64_at(e, 0),
                    offset: header.data_offset + u64_at(e, 8),
                    size: size & 0x00ff_ffff_ffff_ffff,
                    flags: (size >> 56) as u8,
                }
            })
            .collect();
        entries.sort_by_key(|e| e.key);
        Ok(Self { path: path.to_owned(), header, entries })
    }

    pub fn find(&self, key: u64) -> Option<&XpakEntry> {
        self.entries.binary_search_by_key(&key, |e| e.key).ok().map(|i| &self.entries[i])
    }

    /// The entry's bytes as stored (block headers and chunks).
    pub fn read_raw(&self, e: &XpakEntry) -> Result<Vec<u8>, XpakError> {
        let mut f = File::open(&self.path)?;
        f.seek(SeekFrom::Start(e.offset))?;
        let mut raw = vec![0u8; e.size as usize];
        f.read_exact(&mut raw)?;
        Ok(raw)
    }

    /// The entry's data, decoded.
    pub fn read(&self, key: u64) -> Result<Vec<u8>, XpakError> {
        let e = self.find(key).ok_or(XpakError::NotFound(key))?;
        decode(&self.read_raw(e)?)
    }
}

/// One chunk of an entry: where it is in the raw bytes, its codec and sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub at: usize,
    pub stored: usize,
    pub codec: u8,
}

/// Blocks start on 0x80 boundaries of the entry.
pub const BLOCK_ALIGN: usize = 0x80;

/// The chunks of an entry's raw bytes, block by block.
pub fn chunks(raw: &[u8]) -> Result<Vec<Chunk>, XpakError> {
    let mut out = Vec::new();
    let mut p = 0;
    while p < raw.len() {
        if p + BLOCK_HEADER_LEN > raw.len() {
            return Err(XpakError::Block { at: p, what: "block header past the end" });
        }
        let count = u32_at(raw, p) as usize;
        if count == 0 || count > BLOCK_COMMANDS {
            return Err(XpakError::Block { at: p, what: "block chunk count" });
        }
        let mut q = p + BLOCK_HEADER_LEN;
        for i in 0..count {
            let cmd = u32_at(raw, p + 8 + 4 * i);
            let (stored, codec) = ((cmd & 0x00ff_ffff) as usize, (cmd >> 24) as u8);
            if q + stored > raw.len() {
                return Err(XpakError::Block { at: q, what: "chunk past the end" });
            }
            out.push(Chunk { at: q, stored, codec });
            q += stored;
        }
        p = q.next_multiple_of(BLOCK_ALIGN);
    }
    Ok(out)
}

/// Chunk codecs.
pub const CODEC_STORED: u8 = 0;
pub const CODEC_LZ4: u8 = 3;
/// Filler: no output (pads the following block to a 64 KiB file boundary).
pub const CODEC_SKIP: u8 = 0xcf;

/// Decodes an entry's raw bytes: every chunk stored (codec 0) or LZ4
/// (codec 3), concatenated; filler chunks (0xcf) skipped.
pub fn decode(raw: &[u8]) -> Result<Vec<u8>, XpakError> {
    let mut out = Vec::new();
    for c in chunks(raw)? {
        let src = &raw[c.at..c.at + c.stored];
        match c.codec {
            CODEC_STORED => out.extend_from_slice(src),
            CODEC_SKIP => {}
            CODEC_LZ4 => {
                let mut buf = vec![0u8; CHUNK_MAX];
                let n = lz4_flex::block::decompress_into(src, &mut buf)
                    .map_err(|e| XpakError::Lz4 { at: c.at, error: e.to_string() })?;
                out.extend_from_slice(&buf[..n]);
            }
            _ => return Err(XpakError::Block { at: c.at, what: "unknown chunk codec" }),
        }
    }
    Ok(out)
}

/// The packages of a directory (`zone/`): every `*.xpak`, opened.
#[derive(Clone, Debug, Default)]
pub struct XpakSet {
    pub paks: Vec<Xpak>,
}

impl XpakSet {
    /// Opens every `*.xpak` in `dir` (sorted by name); unreadable ones are
    /// returned as errors next to the set.
    pub fn open_dir(dir: &Path) -> std::io::Result<(Self, Vec<(PathBuf, XpakError)>)> {
        let mut names: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("xpak")))
            .collect();
        names.sort();
        let mut set = Self::default();
        let mut bad = Vec::new();
        for p in names {
            match Xpak::open(&p) {
                Ok(x) => set.paks.push(x),
                Err(e) => bad.push((p, e)),
            }
        }
        Ok((set, bad))
    }

    /// Every package holding `key`, with its entry.
    pub fn find_all(&self, key: u64) -> Vec<(&Xpak, &XpakEntry)> {
        self.paks.iter().filter_map(|x| x.find(key).map(|e| (x, e))).collect()
    }

    /// The data of `key` from the first package (in name order) holding it.
    pub fn read(&self, key: u64) -> Result<Vec<u8>, XpakError> {
        let (x, e) = self.find_all(key).into_iter().next().ok_or(XpakError::NotFound(key))?;
        decode(&x.read_raw(e)?)
    }
}
