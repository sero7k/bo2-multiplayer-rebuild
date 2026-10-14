//! Pointers in an inflated Black Ops III zone (8 bytes, little endian).
//!
//! Read on retail zones:
//! * `-1` (`0xFFFF_FFFF_FFFF_FFFF`): the data follows in the stream
//!   ("follow"; asset headers in the `XAsset` table, names, buffers);
//! * `-2` (`...FFFE`): the data follows too, and the loader keeps a pointer
//!   to it for later references ("insert"; seen inside texture combos);
//! * `block << 60 | offset + 1`: data loaded earlier at `offset` of
//!   `block`. Only block 5 (`XFILE_BLOCK_VIRTUAL`) has been seen: the
//!   `XAsset` table names already-loaded assets that way, and asset
//!   headers name strings already in the zone (`0x5000_0000_0001_280e`);
//! * `0`: null.

pub const FOLLOW: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const INSERT: u64 = 0xFFFF_FFFF_FFFF_FFFE;
/// `XFILE_BLOCK_VIRTUAL`.
pub const VIRTUAL_BLOCK: u8 = 5;
const OFFSET_MASK: u64 = (1 << 60) - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ptr {
    Null,
    Follow,
    Insert,
    /// Data loaded earlier: `offset` bytes into `block`.
    Ref { block: u8, offset: u64 },
    /// Anything else (a raw value; not a zone pointer).
    Other(u64),
}

impl Ptr {
    pub const fn decode(raw: u64) -> Self {
        match raw {
            0 => Ptr::Null,
            FOLLOW => Ptr::Follow,
            INSERT => Ptr::Insert,
            _ => {
                let block = (raw >> 60) as u8;
                let low = raw & OFFSET_MASK;
                if block >= 10 || low == 0 {
                    Ptr::Other(raw)
                } else {
                    Ptr::Ref { block, offset: low - 1 }
                }
            }
        }
    }

    pub const fn encode(self) -> u64 {
        match self {
            Ptr::Null => 0,
            Ptr::Follow => FOLLOW,
            Ptr::Insert => INSERT,
            Ptr::Ref { block, offset } => ((block as u64) << 60) | ((offset + 1) & OFFSET_MASK),
            Ptr::Other(raw) => raw,
        }
    }

    /// Data stored right here in the stream (follow or insert).
    pub const fn is_inline(self) -> bool {
        matches!(self, Ptr::Follow | Ptr::Insert)
    }

    pub const fn virtual_offset(self) -> Option<u64> {
        match self {
            Ptr::Ref { block: VIRTUAL_BLOCK, offset } => Some(offset),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_t7_shapes() {
        assert_eq!(Ptr::decode(FOLLOW), Ptr::Follow);
        assert_eq!(Ptr::decode(INSERT), Ptr::Insert);
        assert_eq!(Ptr::decode(0), Ptr::Null);
        assert_eq!(Ptr::decode(0x5000_0000_0001_280e), Ptr::Ref { block: 5, offset: 0x1280d });
        assert_eq!(Ptr::decode(0x5000_0000_0001_280e).virtual_offset(), Some(0x1280d));
        assert_eq!(Ptr::decode(0xF000_0000_0000_0001), Ptr::Other(0xF000_0000_0000_0001));
        for p in [Ptr::Null, Ptr::Follow, Ptr::Insert, Ptr::Ref { block: 5, offset: 7 }] {
            assert_eq!(Ptr::decode(p.encode()), p);
        }
    }
}
