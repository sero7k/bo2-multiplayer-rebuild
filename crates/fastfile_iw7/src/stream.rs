//! The loader's view of the zone: one stream of bytes read in order, each
//! allocation landing in the block on top of a block stack.
//!
//! Every block keeps its own position; `align` rounds the current block's
//! position up (the zone tools' `align(mask)`, `mask` = alignment - 1).
//! Data allocated in RUNTIME takes no bytes of the file (the loader zeroes
//! it), every other block's data is read from the stream. Positions are
//! counted for all blocks, so VIRTUAL (and the others) can be addressed by
//! the references `<block>_<offset+1>` of later data (`ptr.rs`).

use crate::cursor::{ReadError, Result};
use crate::ptr::Ptr;
use crate::xfile::BLOCK_COUNT;

// Infinite Warfare's `XFileBlock`s.
pub const TEMP: u8 = 0;
pub const TEMP_PRELOAD: u8 = 1;
pub const TEMP_POSTLOAD: u8 = 2;
pub const IMAGE_STREAM: u8 = 3;
pub const SHARED_STREAM: u8 = 4;
pub const CALLBACK: u8 = 5;
pub const RUNTIME: u8 = 6;
pub const UNK7: u8 = 7;
pub const VIRTUAL: u8 = 8;
pub const SCRIPT: u8 = 9;

#[derive(Clone, Debug)]
pub struct Stream<'a> {
    data: &'a [u8],
    pos: usize,
    stack: Vec<u8>,
    off: [u64; BLOCK_COUNT],
}

impl<'a> Stream<'a> {
    /// A stream at file offset `pos`, every block at offset `off`.
    pub fn new(data: &'a [u8], pos: usize, off: [u64; BLOCK_COUNT]) -> Self {
        Self {
            data,
            pos,
            stack: vec![TEMP],
            off,
        }
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn block(&self) -> u8 {
        *self.stack.last().unwrap_or(&TEMP)
    }
    pub fn offset(&self, block: u8) -> u64 {
        self.off[block as usize]
    }
    pub fn offsets(&self) -> [u64; BLOCK_COUNT] {
        self.off
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

    /// Takes `len` bytes of a block without file bytes (the loader's
    /// position bumps for runtime tables).
    pub fn inc(&mut self, block: u8, len: u64) {
        self.off[block as usize] += len;
    }

    /// Allocates `len` bytes in the current block; the file offset of the
    /// data, `None` in RUNTIME (no file bytes).
    pub fn alloc(&mut self, len: usize) -> Result<Option<usize>> {
        let b = self.block();
        self.off[b as usize] += len as u64;
        if b == RUNTIME {
            return Ok(None);
        }
        let at = self.pos;
        let end = at.checked_add(len).ok_or(ReadError::Eof(at))?;
        if end > self.data.len() {
            return Err(ReadError::Eof(at));
        }
        self.pos = end;
        Ok(Some(at))
    }

    /// `alloc` after aligning; the file offset (RUNTIME data is an error:
    /// callers that may hit it use `alloc`).
    pub fn load(&mut self, len: usize, mask: u64) -> Result<usize> {
        self.align(mask);
        self.alloc(len)?
            .ok_or(ReadError::Invalid("runtime data read", self.pos))
    }

    /// The current block's position (after aligning to `mask + 1`).
    pub fn here(&mut self, mask: u64) -> u64 {
        self.align(mask);
        self.off[self.block() as usize]
    }

    /// A NUL-terminated string in the current block: (bytes, file offset,
    /// block offset).
    pub fn cstr(&mut self) -> Result<(&'a [u8], usize, u64)> {
        let at = self.pos;
        let window = self.data.get(at..).unwrap_or(&[]);
        let n = window
            .iter()
            .take(crate::cursor::MAX_STRING * 16)
            .position(|&b| b == 0)
            .ok_or(ReadError::BadString(at))?;
        let block_off = self.off[self.block() as usize];
        self.alloc(n + 1)?;
        Ok((&self.data[at..at + n], at, block_off))
    }

    /// The 8-byte VIRTUAL slot an "insert" pointer takes; its offset.
    pub fn insert_slot(&mut self) -> u64 {
        self.push(VIRTUAL);
        self.align(7);
        let v = self.off[VIRTUAL as usize];
        self.off[VIRTUAL as usize] += 8;
        self.pop();
        v
    }

    fn bytes(&self, at: usize, n: usize) -> Result<&'a [u8]> {
        self.data
            .get(at..at.checked_add(n).ok_or(ReadError::Eof(at))?)
            .ok_or(ReadError::Eof(at))
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
        let data = b"abcd\0efgh".to_vec();
        let mut s = Stream::new(&data, 0, [0; BLOCK_COUNT]);
        s.push(VIRTUAL);
        s.inc(VIRTUAL, 3);
        let (str_, at, off) = s.cstr().unwrap();
        assert_eq!((str_, at, off), (&b"abcd"[..], 0, 3));
        s.push(RUNTIME);
        assert_eq!(s.alloc(100).unwrap(), None);
        s.pop();
        s.push(TEMP);
        assert_eq!(s.alloc(2).unwrap(), Some(5));
        s.pop();
        assert_eq!(s.load(1, 3).unwrap(), 7);
        assert_eq!(s.offset(VIRTUAL), 9);
        assert_eq!(s.insert_slot(), 16);
        assert_eq!(s.offset(VIRTUAL), 24);
        assert_eq!(s.offset(RUNTIME), 100);
    }
}
