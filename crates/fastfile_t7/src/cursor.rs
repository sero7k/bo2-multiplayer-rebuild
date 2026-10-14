//! A read position in the inflated zone.
//!
//! The zone is the loader's stream: every struct and string is read in
//! order, whichever block it lands in, with no padding in the file (the
//! loader aligns its block positions, not the stream). Walking only needs
//! the file position; VIRTUAL offsets (what `Ptr::Ref` points at) are not
//! counted yet.

use crate::ptr::Ptr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    /// The data ends at this file offset.
    Eof(usize),
    /// A string with no terminator within the limit.
    BadString(usize),
    /// A value the structure cannot have (`what`, at file offset).
    Invalid(&'static str, usize),
    /// An inline asset of a type the walk cannot read yet (its name, at file offset).
    NoWalker(&'static str, usize),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof(at) => write!(f, "zone ends at {at:#x}"),
            Self::BadString(at) => write!(f, "unterminated string at {at:#x}"),
            Self::Invalid(what, at) => write!(f, "invalid {what} at {at:#x}"),
            Self::NoWalker(ty, at) => write!(f, "no walker for inline {ty} at {at:#x}"),
        }
    }
}

impl std::error::Error for ReadError {}

pub type Result<T> = std::result::Result<T, ReadError>;

/// Longest string accepted (asset names, paths, localized text).
pub const MAX_STRING: usize = 1 << 16;

#[derive(Clone, Debug)]
pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
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
    pub fn u64_at(&self, at: usize) -> Result<u64> {
        let b = self.bytes(at, 8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(b);
        Ok(u64::from_le_bytes(a))
    }
    pub fn ptr_at(&self, at: usize) -> Result<Ptr> {
        Ok(Ptr::decode(self.u64_at(at)?))
    }

    /// Takes `len` bytes of the stream; their file offset.
    pub fn take(&mut self, len: usize) -> Result<usize> {
        self.bytes(self.pos, len)?;
        let at = self.pos;
        self.pos += len;
        Ok(at)
    }

    /// A NUL-terminated string: (bytes without the NUL, file offset).
    pub fn cstr(&mut self) -> Result<(&'a [u8], usize)> {
        let at = self.pos;
        let window = &self.data[at.min(self.data.len())..self.data.len().min(at.saturating_add(MAX_STRING))];
        let Some(n) = window.iter().position(|&b| b == 0) else {
            return Err(if window.len() < MAX_STRING { ReadError::Eof(at) } else { ReadError::BadString(at) });
        };
        self.pos += n + 1;
        Ok((&self.data[at..at + n], at))
    }

    /// The string a pointer field at `field` names: read here when it
    /// follows, `None` when null or loaded earlier.
    pub fn string_field(&mut self, field: usize) -> Result<Option<String>> {
        Ok(match self.ptr_at(field)? {
            p if p.is_inline() => Some(text(self.cstr()?.0)),
            Ptr::Null | Ptr::Ref { .. } => None,
            Ptr::Other(_) => return Err(ReadError::Invalid("string pointer", field)),
            _ => None,
        })
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
    fn strings_follow_their_fields() {
        let mut data = vec![0xffu8; 8];
        data.extend_from_slice(b"abc\0");
        let mut c = Cursor::new(&data, 0);
        let h = c.take(8).unwrap();
        assert_eq!(c.string_field(h).unwrap().as_deref(), Some("abc"));
        assert_eq!(c.pos(), 12);
        assert!(matches!(Cursor::new(b"abc", 0).cstr(), Err(ReadError::Eof(0))));
    }
}
