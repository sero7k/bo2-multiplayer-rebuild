//! Pointers in an inflated IW6 zone (8 bytes, little endian).
//!
//! The top 28 bits are the `0xFDFDFDF` fill, bits 32..36 a block number and
//! the low 32 bits a value:
//! * `0xFDFDFDFF_FFFFFFFF` (block `F`, value `-1`): the data follows in the
//!   stream ("follow");
//! * `0xFDFDFDFF_FFFFFFFE` (`-2`): the data follows, and the loader also
//!   takes an 8-byte, 8-aligned pointer slot in the VIRTUAL block for it
//!   ("insert");
//! * `0xFDFDFDFF_FFFFFFFD` (`-3`): an offset into the shared data (unused by
//!   the assets read here);
//! * `0xFDFDFDF<b>_<off+1>`: a reference to data loaded earlier, at offset
//!   `off` of block `b` (`3` = VIRTUAL). Strings, sound files and whole
//!   assets are shared this way.
//! * `0`: null.

/// The bytes of a "follow" pointer.
pub const FOLLOW_BYTES: [u8; 8] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFD, 0xFD, 0xFD];
pub const FOLLOW: u64 = 0xFDFD_FDFF_FFFF_FFFF;
pub const INSERT: u64 = 0xFDFD_FDFF_FFFF_FFFE;
pub const SHARED_OFFSET: u64 = 0xFDFD_FDFF_FFFF_FFFD;
const FILL_MASK: u64 = 0xFFFF_FFF0_0000_0000;
const FILL: u64 = 0xFDFD_FDF0_0000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ptr {
    Null,
    Follow,
    Insert,
    SharedOffset,
    /// Data loaded earlier: `offset` bytes into `block`.
    Ref {
        block: u8,
        offset: u32,
    },
    /// Anything else (a raw value; not a zone pointer).
    Other(u64),
}

impl Ptr {
    pub const fn decode(raw: u64) -> Self {
        match raw {
            0 => Ptr::Null,
            FOLLOW => Ptr::Follow,
            INSERT => Ptr::Insert,
            SHARED_OFFSET => Ptr::SharedOffset,
            _ if raw & FILL_MASK == FILL => {
                let block = ((raw >> 32) & 0xF) as u8;
                let low = raw as u32;
                if block == 0xF || low == 0 {
                    Ptr::Other(raw)
                } else {
                    Ptr::Ref {
                        block,
                        offset: low - 1,
                    }
                }
            }
            _ => Ptr::Other(raw),
        }
    }

    /// The pointer's encoding (inverse of `decode`).
    pub const fn encode(self) -> u64 {
        match self {
            Ptr::Null => 0,
            Ptr::Follow => FOLLOW,
            Ptr::Insert => INSERT,
            Ptr::SharedOffset => SHARED_OFFSET,
            Ptr::Ref { block, offset } => FILL | ((block as u64 & 0xF) << 32) | (offset as u64 + 1),
            Ptr::Other(raw) => raw,
        }
    }

    /// Data stored inline right here in the stream (follow or insert).
    pub const fn is_inline(self) -> bool {
        matches!(self, Ptr::Follow | Ptr::Insert)
    }

    /// A VIRTUAL-block reference's offset.
    pub const fn virtual_offset(self) -> Option<u64> {
        match self {
            Ptr::Ref { block: 3, offset } => Some(offset as u64),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_retail_shapes() {
        assert_eq!(Ptr::decode(u64::from_le_bytes(FOLLOW_BYTES)), Ptr::Follow);
        assert_eq!(Ptr::decode(INSERT), Ptr::Insert);
        assert_eq!(Ptr::decode(0), Ptr::Null);
        // `music_mainmenu_mp`'s name in code_post_gfx_mp: 0xfdfdfdf3_0001718f.
        assert_eq!(
            Ptr::decode(0xFDFD_FDF3_0001_718F),
            Ptr::Ref {
                block: 3,
                offset: 0x1718E
            }
        );
        assert_eq!(
            Ptr::decode(0xFDFD_FDF3_0001_718F).virtual_offset(),
            Some(0x1718E)
        );
        assert_eq!(Ptr::decode(0x3F80_0000), Ptr::Other(0x3F80_0000));
        for p in [
            Ptr::Null,
            Ptr::Follow,
            Ptr::Insert,
            Ptr::Ref {
                block: 3,
                offset: 7,
            },
        ] {
            assert_eq!(Ptr::decode(p.encode()), p);
        }
    }
}
