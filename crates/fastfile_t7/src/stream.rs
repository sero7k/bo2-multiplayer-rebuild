//! The loader's view of the zone: one stream of bytes read in order, each
//! allocation landing in the block on top of a block stack.
//!
//! Every block keeps its own position. The file has no padding, but the
//! loader rounds the current block's position up to each allocation's
//! alignment (`align(mask)`, mask = alignment - 1) before taking it, so
//! the VIRTUAL offsets that later references name (`ptr.rs`) are the
//! aligned ones. Blocks that take no file bytes (`STREAMER`, the runtime
//! blocks) only move their position.
//!
//! Read on retail zones (`walk` checks every reference against it, and at
//! the end of a zone the block positions against the header's sizes):
//! * the `XAssetList` (48 bytes) and each asset's top-level struct are in
//!   TEMP (the header's TEMP size is 0x30 + the largest asset header:
//!   0x220e0 = 0x30 + SndBank's 0x220b0), also for sub-assets loaded
//!   through a pointer; everything they point to is in VIRTUAL, the script
//!   string and asset tables included;
//! * PHYSICAL holds GPU data with file bytes: image pixels (aligned 256),
//!   material constant buffer init data (aligned 256), non-streamed mesh
//!   vertex/index data; STREAMER (no file bytes) streamed images' pixels;
//! * strings take no alignment; most arrays their element's natural x64
//!   one, with type-specific exceptions in the walkers (rawfile buffers 16,
//!   techniques 16, ...);
//! * an "insert" pointer (`-2`) first takes an 8-byte, 8-aligned VIRTUAL
//!   slot (no file bytes) that later references name (inline technique
//!   sets in materials, LOD meshes in models); a `-1` asset pointer in a
//!   VIRTUAL struct is itself what later references name;
//! * references to PHYSICAL data (`6 << 60 | offset + 1`) occur too
//!   (shared constant buffer init data) and are checked the same way.

use std::collections::HashSet;

use crate::cursor::{MAX_STRING, ReadError, Result};
use crate::ptr::Ptr;

pub const BLOCK_COUNT: usize = 10;
// Black Ops III's `XFileBlock`s (the order of the header's block sizes).
pub const TEMP: u8 = 0;
pub const RUNTIME_VIRTUAL: u8 = 1;
pub const RUNTIME_PHYSICAL: u8 = 2;
pub const DELAY_VIRTUAL: u8 = 3;
pub const DELAY_PHYSICAL: u8 = 4;
pub const VIRTUAL: u8 = 5;
pub const PHYSICAL: u8 = 6;
pub const STREAMER_RESERVE: u8 = 7;
pub const STREAMER: u8 = 8;
pub const MEMMAPPED: u8 = 9;

/// Blocks whose data is not in the file.
pub const fn is_fileless(block: u8) -> bool {
    matches!(block, RUNTIME_VIRTUAL | RUNTIME_PHYSICAL | STREAMER_RESERVE | STREAMER)
}

/// One contiguous run of VIRTUAL data read from the file.
#[derive(Clone, Copy, Debug)]
struct Segment {
    virt: u64,
    pos: usize,
    len: u64,
}

/// File offsets of VIRTUAL data, both ways.
#[derive(Default, Debug)]
pub struct VirtualMap {
    segs: Vec<Segment>,
}

impl VirtualMap {
    fn add(&mut self, virt: u64, pos: usize, len: u64) {
        if len == 0 {
            return;
        }
        if let Some(last) = self.segs.last_mut()
            && last.virt + last.len == virt
            && last.pos as u64 + last.len == pos as u64
        {
            last.len += len;
            return;
        }
        self.segs.push(Segment { virt, pos, len });
    }

    /// The file offset of VIRTUAL offset `virt`, if it was read from the file.
    pub fn file_pos(&self, virt: u64) -> Option<usize> {
        let i = self.segs.partition_point(|s| s.virt <= virt).checked_sub(1)?;
        let s = self.segs[i];
        (virt < s.virt + s.len).then(|| s.pos + (virt - s.virt) as usize)
    }

    /// The VIRTUAL offset of file offset `pos`, if VIRTUAL data was read there.
    pub fn virt_of(&self, pos: usize) -> Option<u64> {
        let i = self.segs.partition_point(|s| s.pos <= pos).checked_sub(1)?;
        let s = self.segs[i];
        ((pos - s.pos) < s.len as usize).then(|| s.virt + (pos - s.pos) as u64)
    }
}

/// How the references read so far landed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RefStats {
    /// References to VIRTUAL (or PHYSICAL) data read earlier that land on
    /// the start of an allocation, a string or a pointer slot.
    pub resolved: usize,
    /// References into data read earlier that land elsewhere (drift).
    pub unresolved: usize,
    /// References past what has been read (the zone may name data it loads later).
    pub forward: usize,
    /// References to blocks other than VIRTUAL and PHYSICAL.
    pub other_block: usize,
}

/// Debug: `T7_TRACE=<hex from>,<hex to>` prints every allocation whose
/// file offset is in the range.
fn trace_range() -> Option<(usize, usize)> {
    use std::sync::OnceLock;
    static RANGE: OnceLock<Option<(usize, usize)>> = OnceLock::new();
    *RANGE.get_or_init(|| {
        let v = std::env::var("T7_TRACE").ok()?;
        let (a, b) = v.split_once(',')?;
        Some((usize::from_str_radix(a, 16).ok()?, usize::from_str_radix(b, 16).ok()?))
    })
}

#[derive(Debug)]
pub struct Stream<'a> {
    data: &'a [u8],
    pos: usize,
    stack: Vec<u8>,
    off: [u64; BLOCK_COUNT],
    pub vmap: VirtualMap,
    /// VIRTUAL offsets where allocations, strings and pointer slots start.
    starts: HashSet<u64>,
    /// PHYSICAL offsets where allocations start.
    phys_starts: HashSet<u64>,
    pub refs: RefStats,
    /// The first unresolved reference: (file offset of the pointer, block offset).
    pub first_unresolved: Option<(usize, u64)>,
}

impl<'a> Stream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            stack: vec![TEMP],
            off: [0; BLOCK_COUNT],
            vmap: VirtualMap::default(),
            starts: HashSet::new(),
            phys_starts: HashSet::new(),
            refs: RefStats::default(),
            first_unresolved: None,
        }
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    /// Debug: moves the stream to file offset `pos`.
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }
    pub fn block(&self) -> u8 {
        *self.stack.last().unwrap_or(&TEMP)
    }
    pub fn offset(&self, block: u8) -> u64 {
        self.off[block as usize]
    }
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, block: u8) {
        self.stack.push(block);
    }
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Rounds the current block's position up to `mask + 1`.
    pub fn align(&mut self, mask: u64) {
        let b = self.block() as usize;
        self.off[b] = (self.off[b] + mask) & !mask;
    }

    /// Takes `len` bytes in the current block after aligning to `mask + 1`;
    /// the file offset (`None` in a block without file bytes).
    pub fn alloc(&mut self, len: usize, mask: u64) -> Result<Option<usize>> {
        self.align(mask);
        let b = self.block();
        let v = self.off[b as usize];
        self.off[b as usize] += len as u64;
        if is_fileless(b) {
            return Ok(None);
        }
        let at = self.pos;
        let end = at.checked_add(len).ok_or(ReadError::Eof(at))?;
        if end > self.data.len() {
            return Err(ReadError::Eof(at));
        }
        self.pos = end;
        if let Some((a, z)) = trace_range()
            && (a..z).contains(&at)
        {
            eprintln!("ALLOC file {at:#x} len {len:#x} block {b} off {v:#x} mask {mask:#x}");
        }
        if b == VIRTUAL {
            self.vmap.add(v, at, len as u64);
            self.starts.insert(v);
        } else if b == PHYSICAL {
            self.phys_starts.insert(v);
        }
        Ok(Some(at))
    }

    /// `alloc` of data that must be in the file.
    pub fn load(&mut self, len: usize, mask: u64) -> Result<usize> {
        let at = self.pos;
        self.alloc(len, mask)?.ok_or(ReadError::Invalid("file data in a block without file bytes", at))
    }

    /// A NUL-terminated string in the current block: (bytes, file offset).
    pub fn cstr(&mut self) -> Result<(&'a [u8], usize)> {
        let at = self.pos;
        let window = self.data.get(at..).unwrap_or(&[]);
        let n = window
            .iter()
            .take(MAX_STRING * 16)
            .position(|&b| b == 0)
            .ok_or(if window.len() < MAX_STRING * 16 { ReadError::Eof(at) } else { ReadError::BadString(at) })?;
        self.load(n + 1, 0)?;
        Ok((&self.data[at..at + n], at))
    }

    /// Moves the current block's position without file bytes.
    pub fn skip(&mut self, len: u64) {
        let b = self.block() as usize;
        self.off[b] += len;
    }

    /// The 8-byte VIRTUAL slot an "insert" pointer takes; its offset.
    pub fn insert_slot(&mut self) -> u64 {
        let v = (self.off[VIRTUAL as usize] + 7) & !7;
        if let Some((a, z)) = trace_range()
            && (a..z).contains(&self.pos)
        {
            eprintln!("SLOT at file {:#x} off {v:#x}", self.pos);
        }
        self.off[VIRTUAL as usize] = v + 8;
        self.starts.insert(v);
        v
    }

    /// Marks the pointer field at file offset `field` as a slot later
    /// references may name (a field that loaded an asset).
    pub fn mark_slot(&mut self, field: usize) {
        if let Some(v) = self.vmap.virt_of(field) {
            self.starts.insert(v);
        }
    }

    /// Counts a reference read at file offset `field`.
    pub fn note_ref(&mut self, field: usize, p: Ptr) {
        let Ptr::Ref { block, offset } = p else { return };
        if let Some(range) = std::env::var("T7_REFLOG").ok().and_then(|r| {
            let (a, b) = r.split_once(',')?;
            Some(usize::from_str_radix(a, 16).ok()?..usize::from_str_radix(b, 16).ok()?)
        }) && range.contains(&field)
        {
            let text = self.string_at(offset).map(|t| String::from_utf8_lossy(&t[..t.len().min(40)]).into_owned());
            eprintln!(
                "REF field {field:#x} block {block} -> {offset:#x} start {} now {:#x} {text:?}",
                self.starts.contains(&offset),
                self.off[VIRTUAL as usize]
            );
        }
        if block == PHYSICAL {
            if offset >= self.off[PHYSICAL as usize] {
                self.refs.forward += 1;
            } else if self.phys_starts.contains(&offset) {
                self.refs.resolved += 1;
            } else {
                self.refs.unresolved += 1;
                if self.first_unresolved.is_none() {
                    self.first_unresolved = Some((field, offset));
                }
            }
            return;
        }
        if block != VIRTUAL {
            self.refs.other_block += 1;
            if self.refs.other_block <= 5 && std::env::var_os("T7_MISSES").is_some() {
                eprintln!("OTHER field {field:#x} block {block} offset {offset:#x} (now {:#x})", self.off[block as usize]);
            }
        } else if offset >= self.off[VIRTUAL as usize] {
            self.refs.forward += 1;
        } else if self.starts.contains(&offset) {
            self.refs.resolved += 1;
        } else {
            self.refs.unresolved += 1;
            if self.refs.unresolved <= 200 && std::env::var_os("T7_MISSES").is_some() {
                let near = (offset.saturating_sub(64)..offset + 64).filter(|v| self.starts.contains(v)).collect::<Vec<_>>();
                eprintln!(
                    "MISS field {field:#x} (virt {:x?}) want {offset:#x} (file {:x?}) virt now {:#x} starts near {near:x?}",
                    self.vmap.virt_of(field),
                    self.vmap.file_pos(offset),
                    self.off[VIRTUAL as usize]
                );
            }
            if self.first_unresolved.is_none() {
                self.first_unresolved = Some((field, offset));
            }
        }
    }

    /// The text of a string read earlier at VIRTUAL `virt`.
    pub fn string_at(&self, virt: u64) -> Option<&'a [u8]> {
        let at = self.vmap.file_pos(virt)?;
        let n = self.data.get(at..)?.iter().take(MAX_STRING).position(|&b| b == 0)?;
        Some(&self.data[at..at + n])
    }

    fn bytes(&self, at: usize, n: usize) -> Result<&'a [u8]> {
        self.data.get(at..at.checked_add(n).ok_or(ReadError::Eof(at))?).ok_or(ReadError::Eof(at))
    }
    pub fn slice(&self, at: usize, n: usize) -> Result<&'a [u8]> {
        self.bytes(at, n)
    }
    pub fn u8_at(&self, at: usize) -> Result<u8> {
        Ok(self.bytes(at, 1)?[0])
    }
    pub fn u16_at(&self, at: usize) -> Result<u16> {
        let b = self.bytes(at, 2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn u32_at(&self, at: usize) -> Result<u32> {
        let b = self.bytes(at, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn i32_at(&self, at: usize) -> Result<i32> {
        Ok(self.u32_at(at)? as i32)
    }
    pub fn f32_at(&self, at: usize) -> Result<f32> {
        Ok(f32::from_bits(self.u32_at(at)?))
    }
    pub fn u64_at(&self, at: usize) -> Result<u64> {
        let b = self.bytes(at, 8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(b);
        Ok(u64::from_le_bytes(a))
    }
    pub fn ptr_at(&self, at: usize) -> Result<Ptr> {
        Ok(Ptr::decode(self.u64_at(at)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_count_apart() {
        let data = b"abcd\0efghijklmnop".to_vec();
        let mut s = Stream::new(&data);
        s.push(VIRTUAL);
        s.alloc(3, 0).unwrap();
        s.seek(0);
        let (str_, at) = s.cstr().unwrap();
        assert_eq!((str_, at), (&b"abcd"[..], 0));
        assert_eq!(s.offset(VIRTUAL), 8);
        assert_eq!(s.vmap.file_pos(3), Some(0));
        s.push(STREAMER);
        assert_eq!(s.alloc(100, 0).unwrap(), None);
        s.pop();
        assert_eq!(s.alloc(4, 7).unwrap(), Some(5));
        assert_eq!(s.offset(VIRTUAL), 12);
        s.note_ref(0, Ptr::Ref { block: VIRTUAL, offset: 3 });
        s.note_ref(0, Ptr::Ref { block: VIRTUAL, offset: 4 });
        s.note_ref(0, Ptr::Ref { block: VIRTUAL, offset: 40 });
        assert_eq!(s.refs, RefStats { resolved: 1, unresolved: 1, forward: 1, other_block: 0 });
    }
}
