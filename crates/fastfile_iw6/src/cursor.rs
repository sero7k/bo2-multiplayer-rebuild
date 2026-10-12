//! A read position in the inflated zone that also counts the VIRTUAL block.
//!
//! The zone is the loader's stream: every struct and string is read in
//! order, whichever block it lands in, with no padding in the file. The
//! loader aligns its *block* positions instead, so to know the VIRTUAL
//! offset of something (which is what references point at) the reader must
//! count every VIRTUAL allocation with the loader's alignment. TEMP data
//! (asset headers, loaded-sound PCM) and RUNTIME data take no VIRTUAL space.
//! The VIRTUAL count is `None` where it is not known.

use crate::ptr::Ptr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    /// The data ends at this file offset.
    Eof(usize),
    /// A string with no terminator within the limit.
    BadString(usize),
    /// A value the structure cannot have (`what`, at file offset).
    Invalid(&'static str, usize),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof(at) => write!(f, "zone ends at {at:#x}"),
            Self::BadString(at) => write!(f, "unterminated string at {at:#x}"),
            Self::Invalid(what, at) => write!(f, "invalid {what} at {at:#x}"),
        }
    }
}

impl std::error::Error for ReadError {}

pub type Result<T> = std::result::Result<T, ReadError>;

/// Longest string accepted (asset names, paths).
pub const MAX_STRING: usize = 4096;

pub const fn align_up(v: u64, align: u64) -> u64 {
    (v + align - 1) & !(align - 1)
}

#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    virt: Option<u64>,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8], pos: usize, virt: Option<u64>) -> Self {
        Self { data, pos, virt }
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn virt(&self) -> Option<u64> {
        self.virt
    }

    fn bytes(&self, at: usize, n: usize) -> Result<&'a [u8]> {
        self.data
            .get(at..at.checked_add(n).ok_or(ReadError::Eof(at))?)
            .ok_or(ReadError::Eof(at))
    }

    /// Little-endian reads at an absolute file offset (no movement).
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

    /// A struct (or array) in the VIRTUAL block: `len` bytes, the block
    /// position aligned to `align` first. Returns (file offset, VIRTUAL offset).
    pub fn virt_struct(&mut self, len: usize, align: u64) -> Result<(usize, Option<u64>)> {
        self.bytes(self.pos, len)?;
        let at = self.pos;
        self.virt = self.virt.map(|v| align_up(v, align));
        let v = self.virt;
        self.pos += len;
        self.virt = self.virt.map(|v| v + len as u64);
        Ok((at, v))
    }

    /// Data in a block that is not VIRTUAL (TEMP headers, sample data).
    pub fn other_block(&mut self, len: usize) -> Result<usize> {
        self.bytes(self.pos, len)?;
        let at = self.pos;
        self.pos += len;
        Ok(at)
    }

    /// The VIRTUAL pointer slot an "insert" pointer takes (no file bytes).
    pub fn insert_slot(&mut self) {
        self.virt = self.virt.map(|v| align_up(v, 8) + 8);
    }

    /// A NUL-terminated string in the VIRTUAL block (no alignment).
    /// Returns (bytes without the NUL, file offset, VIRTUAL offset).
    pub fn cstr(&mut self) -> Result<(&'a [u8], usize, Option<u64>)> {
        let at = self.pos;
        let window =
            &self.data[at.min(self.data.len())..self.data.len().min(at.saturating_add(MAX_STRING))];
        let Some(n) = window.iter().position(|&b| b == 0) else {
            return Err(if window.len() < MAX_STRING {
                ReadError::Eof(at)
            } else {
                ReadError::BadString(at)
            });
        };
        let v = self.virt;
        self.pos += n + 1;
        self.virt = self.virt.map(|v| v + n as u64 + 1);
        Ok((&self.data[at..at + n], at, v))
    }

    /// Skip `n` bytes of the VIRTUAL block's byte data (no alignment).
    pub fn virt_bytes(&mut self, n: usize) -> Result<usize> {
        Ok(self.virt_struct(n, 1)?.0)
    }
}

/// Lossy text of a zone string.
pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_virtual_with_alignment() {
        let data = b"ab\0xxxxxxxxxxxxxxxxxxxxxxxx".to_vec();
        let mut c = Cursor::new(&data, 0, Some(0x101));
        let (s, at, v) = c.cstr().unwrap();
        assert_eq!((s, at, v), (&b"ab"[..], 0, Some(0x101)));
        assert_eq!(c.virt(), Some(0x104));
        let (at, v) = c.virt_struct(8, 8).unwrap();
        assert_eq!((at, v), (3, Some(0x108)));
        c.insert_slot();
        assert_eq!(c.virt(), Some(0x118));
        c.other_block(4).unwrap();
        assert_eq!((c.pos(), c.virt()), (15, Some(0x118)));
        assert!(c.virt_struct(100, 4).is_err());
        let mut unknown = Cursor::new(&data, 0, None);
        unknown.cstr().unwrap();
        assert_eq!(unknown.virt(), None);
    }

    #[test]
    fn strings_need_a_terminator() {
        let data = b"abc".to_vec();
        assert!(matches!(
            Cursor::new(&data, 0, None).cstr(),
            Err(ReadError::Eof(0))
        ));
    }
}
