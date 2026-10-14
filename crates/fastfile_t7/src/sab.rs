//! bo2mp: Black Ops III sound banks (`zone/snd/<lang>/<zone>.<lang>.sabl`
//! loaded, `.sabs` streamed): Black Ops II's `2UX#` bank grown to 64-bit
//! offsets.
//!
//! ```text
//! header (2048)  0x00 "2UX#"; 0x04 u32 version (15); 0x08 u32 entry size (36);
//!                0x0c u32 (16); 0x10 u32 dependency name size (64); 0x14 u32 entry count;
//!                0x18 u32 dependency count; 0x1c u32; 0x20 u64 file size;
//!                0x28 u64 entry table offset; 0x30 u64 second table offset;
//!                0x38 16 bytes (checksum); 0x48 dependency names (64 each)
//! entry (36)     0x00 u32 id (the alias's `assetId`); 0x04 u32 size; 0x08 u64 frame count;
//!                0x10 u64 offset; 0x18 u8 frame rate index; 0x19 u8 channels;
//!                0x1a u8 looping; 0x1b u8 format (8: FLAC, 0: 16-bit PCM); 0x1c 8 bytes
//! ```
//! (Read on retail `mp_common.all.sabl` and `.sabs`: every entry a FLAC
//! stream, the first at 0x800, each next one right after; the frame rate
//! indices are Black Ops II's.)

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const MAGIC: &[u8; 4] = b"2UX#";
pub const VERSION: u32 = 15;
pub const HEADER_SIZE: usize = 2048;
pub const ENTRY_SIZE: usize = 36;

/// Frame rates by `frameRateIndex` (Black Ops II's table).
pub const FRAME_RATES: [u32; 9] = [8000, 12000, 16000, 24000, 32000, 44100, 48000, 96000, 192000];

#[derive(Clone, Debug, Default)]
pub struct SabHeader {
    pub version: u32,
    pub entry_size: u32,
    pub entry_count: u32,
    pub file_size: u64,
    pub entry_offset: u64,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SabEntry {
    pub id: u32,
    pub size: u32,
    pub offset: u64,
    pub frames: u64,
    pub rate_index: u8,
    pub channels: u8,
    pub looping: bool,
    pub format: u8,
}

impl SabEntry {
    pub fn rate(&self) -> u32 {
        FRAME_RATES.get(usize::from(self.rate_index)).copied().unwrap_or(48000)
    }
}

/// One opened bank: header and entries (sorted by id); the sounds' bytes
/// are read on demand.
#[derive(Clone, Debug, Default)]
pub struct SabFile {
    pub path: PathBuf,
    pub header: SabHeader,
    pub entries: Vec<SabEntry>,
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from(u32_at(b, at)) | (u64::from(u32_at(b, at + 4)) << 32)
}

pub fn parse_header(head: &[u8]) -> Result<SabHeader, String> {
    if head.len() < HEADER_SIZE || &head[..4] != MAGIC {
        return Err("not a sound bank".into());
    }
    let version = u32_at(head, 4);
    if version != VERSION {
        return Err(format!("bank version {version}, want {VERSION}"));
    }
    let entry_size = u32_at(head, 8);
    if entry_size as usize != ENTRY_SIZE {
        return Err(format!("entry size {entry_size}, want {ENTRY_SIZE}"));
    }
    let dep_size = (u32_at(head, 0x10) as usize).clamp(1, 256);
    let dep_count = u32_at(head, 0x18) as usize;
    let mut dependencies = Vec::new();
    for i in 0..dep_count.min(32) {
        let at = 0x48 + i * dep_size;
        let Some(raw) = head.get(at..at + dep_size) else { break };
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        if end > 0 {
            dependencies.push(String::from_utf8_lossy(&raw[..end]).into_owned());
        }
    }
    Ok(SabHeader {
        version,
        entry_size,
        entry_count: u32_at(head, 0x14),
        file_size: u64_at(head, 0x20),
        entry_offset: u64_at(head, 0x28),
        dependencies,
    })
}

pub fn parse_entry(e: &[u8]) -> SabEntry {
    SabEntry {
        id: u32_at(e, 0),
        size: u32_at(e, 4),
        frames: u64_at(e, 8),
        offset: u64_at(e, 0x10),
        rate_index: e[0x18],
        channels: e[0x19],
        looping: e[0x1a] != 0,
        format: e[0x1b],
    }
}

impl SabFile {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut head = vec![0u8; HEADER_SIZE];
        f.read_exact(&mut head).map_err(|e| format!("{}: header: {e}", path.display()))?;
        let header = parse_header(&head).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut table = vec![0u8; header.entry_count as usize * ENTRY_SIZE];
        f.seek(SeekFrom::Start(header.entry_offset)).map_err(|e| format!("{}: {e}", path.display()))?;
        f.read_exact(&mut table).map_err(|e| format!("{}: entry table: {e}", path.display()))?;
        let mut entries: Vec<SabEntry> = table.chunks_exact(ENTRY_SIZE).map(parse_entry).collect();
        entries.sort_by_key(|e| e.id);
        Ok(Self { path: path.to_path_buf(), header, entries })
    }

    pub fn find(&self, id: u32) -> Option<&SabEntry> {
        self.entries.binary_search_by_key(&id, |e| e.id).ok().map(|i| &self.entries[i])
    }

    /// One sound's bytes (a FLAC stream, or 16-bit PCM).
    pub fn read(&self, e: &SabEntry) -> Result<Vec<u8>, String> {
        let mut f = std::fs::File::open(&self.path).map_err(|x| format!("{}: {x}", self.path.display()))?;
        f.seek(SeekFrom::Start(e.offset)).map_err(|x| x.to_string())?;
        let mut out = vec![0u8; e.size as usize];
        f.read_exact(&mut out).map_err(|x| format!("{}: entry {:#x}: {x}", self.path.display(), e.id))?;
        Ok(out)
    }
}
