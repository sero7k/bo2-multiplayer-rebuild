//! Pointers as a decompressed zone stores them, and a bounds-checked
//! little-endian reader over the zone image.
//!
//! A zone is written depth first: a struct, then the data its pointers
//! lead to, in field order, packed with no alignment. A stored pointer is
//!
//! - `0`: null;
//! - `-2` ([`Ptr::Follow`]) or `-1` ([`Ptr::Insert`]): the data follows in
//!   the stream (`-1` is what a later pointer may point back to);
//! - `-3`: seen in sound packages, meaning not known;
//! - `(block << 32) | (offset + 1)` ([`Ptr::Back`]): data loaded earlier,
//!   at `offset` in load block `block` (0x0f for most shared data, 0x1f for
//!   slots local to the asset being loaded). A block offset is not a zone
//!   offset: the blocks interleave in the stream.
#![forbid(unsafe_code)]

/// `-2`.
pub const FOLLOW: u64 = 0xffff_ffff_ffff_fffe;
/// `-1`.
pub const INSERT: u64 = 0xffff_ffff_ffff_ffff;
/// `-3`.
pub const MINUS3: u64 = 0xffff_ffff_ffff_fffd;

/// One stored pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ptr {
    Null,
    Follow,
    Insert,
    Minus3,
    /// Data loaded earlier: its load block and offset there.
    Back { block: u32, offset: u32 },
    /// A value of none of these forms (not a pointer).
    Other(u64),
}

impl Ptr {
    #[must_use]
    pub fn from_raw(v: u64) -> Self {
        match v {
            0 => Self::Null,
            FOLLOW => Self::Follow,
            INSERT => Self::Insert,
            MINUS3 => Self::Minus3,
            v if v & 1 == 1 && (v >> 32) != 0 && (v >> 32) < 0x40 => Self::Back {
                block: (v >> 32) as u32,
                offset: ((v & 0xffff_ffff) - 1) as u32,
            },
            v => Self::Other(v),
        }
    }

    /// Whether the data follows in the stream (`-2` or `-1`).
    #[must_use]
    pub fn follows(self) -> bool {
        matches!(self, Self::Follow | Self::Insert)
    }
}

/// Reading past the zone image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutOfBounds {
    pub at: usize,
    pub len: usize,
}

impl std::fmt::Display for OutOfBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "read of {} bytes at {:#x} past the zone", self.len, self.at)
    }
}

impl std::error::Error for OutOfBounds {}

/// Little-endian reads at absolute offsets of a zone image.
#[derive(Clone, Copy)]
pub struct Bytes<'a>(pub &'a [u8]);

impl<'a> Bytes<'a> {
    pub fn get(&self, at: usize, len: usize) -> Result<&'a [u8], OutOfBounds> {
        self.0.get(at..at.checked_add(len).ok_or(OutOfBounds { at, len })?).ok_or(OutOfBounds { at, len })
    }
    pub fn u8(&self, at: usize) -> Result<u8, OutOfBounds> {
        Ok(self.get(at, 1)?[0])
    }
    pub fn u16(&self, at: usize) -> Result<u16, OutOfBounds> {
        let b = self.get(at, 2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn u32(&self, at: usize) -> Result<u32, OutOfBounds> {
        let b = self.get(at, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn i32(&self, at: usize) -> Result<i32, OutOfBounds> {
        Ok(self.u32(at)? as i32)
    }
    pub fn u64(&self, at: usize) -> Result<u64, OutOfBounds> {
        let b = self.get(at, 8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    pub fn f32(&self, at: usize) -> Result<f32, OutOfBounds> {
        Ok(f32::from_bits(self.u32(at)?))
    }
    pub fn ptr(&self, at: usize) -> Result<Ptr, OutOfBounds> {
        Ok(Ptr::from_raw(self.u64(at)?))
    }
    /// A NUL-terminated string at `at` and the offset after its NUL.
    pub fn cstr(&self, at: usize) -> Result<(&'a str, usize), OutOfBounds> {
        let rest = self.0.get(at..).ok_or(OutOfBounds { at, len: 1 })?;
        let n = rest.iter().position(|&b| b == 0).ok_or(OutOfBounds { at, len: rest.len() + 1 })?;
        let s = std::str::from_utf8(&rest[..n]).unwrap_or("");
        Ok((s, at + n + 1))
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_forms() {
        assert_eq!(Ptr::from_raw(0), Ptr::Null);
        assert_eq!(Ptr::from_raw(FOLLOW), Ptr::Follow);
        assert_eq!(Ptr::from_raw(INSERT), Ptr::Insert);
        assert_eq!(Ptr::from_raw(0x0f_006a_669f), Ptr::Back { block: 0x0f, offset: 0x006a_669e });
        assert_eq!(Ptr::from_raw(0x1f_0000_01c1), Ptr::Back { block: 0x1f, offset: 0x1c0 });
        // (A hash is not a pointer.)
        assert!(matches!(Ptr::from_raw(0x7440_a16f_4a7c_61f2), Ptr::Other(_)));
    }

    #[test]
    fn reads_are_bounded() {
        let z = [1u8, 0, 0, 0, b'a', b'b', 0];
        let b = Bytes(&z);
        assert_eq!(b.u32(0), Ok(1));
        assert!(b.u64(0).is_err());
        assert_eq!(b.cstr(4).unwrap(), ("ab", 7));
    }
}
