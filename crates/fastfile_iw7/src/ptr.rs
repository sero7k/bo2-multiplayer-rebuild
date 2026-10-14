//! Pointers in an inflated Infinite Warfare zone (8 bytes, little endian).
//!
//! IW7's pointers carry a `0xFFFFFFF` fill (x64-zt `iw7/zonetool.cpp`:
//! `set_fields(RUNTIME, 0xFFFFFFF000000000, -2, -4, -3, -1)`). With block
//! `F`, read on retail (`mp_weapon_world_iw7_m8_tr`: the zone's table names
//! each model by the field of the streaming info's per-model table that
//! loaded it, which lands only when neither `-1` nor `-2` takes a slot):
//! * `...FFFE` (`-2`): the data follows in the stream ("follow");
//! * `...FFFF` (`-1`): the data follows, shared (strings, the models a
//!   streaming info lists): laid out as "follow";
//! * `...FFFD` (`-3`): the data follows and the loader also takes an
//!   8-byte, 8-aligned pointer slot in VIRTUAL for it ("insert");
//! * `...FFFC` (`-4`): an offset into the shared data;
//! * `<fill or 0>` | `block << 32` | `off+1`: data loaded earlier at
//!   offset `off` of `block` (`8` = VIRTUAL). Whole assets are named with
//!   no fill (`8 << 32 | slot+1`, `iw7/zone.cpp` `get_asset_pointer`),
//!   other data with the fill.
//! * `0`: null.

pub const FOLLOW: u64 = 0xFFFF_FFFF_FFFF_FFFE;
pub const SHARED: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const INSERT: u64 = 0xFFFF_FFFF_FFFF_FFFD;
pub const SHARED_OFFSET: u64 = 0xFFFF_FFFF_FFFF_FFFC;
const FILL_MASK: u64 = 0xFFFF_FFF0_0000_0000;
const FILL: u64 = 0xFFFF_FFF0_0000_0000;
/// The VIRTUAL block's number.
pub const VIRTUAL_BLOCK: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ptr {
    Null,
    Follow,
    Insert,
    SharedOffset,
    Shared,
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
            SHARED => Ptr::Follow,
            _ if raw & FILL_MASK == FILL || raw >> 36 == 0 => {
                let block = ((raw >> 32) & 0xF) as u8;
                let low = raw as u32;
                if block == 0xF || low == 0 || block >= 10 {
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

    /// The pointer's encoding (inverse of `decode`; references with the fill).
    pub const fn encode(self) -> u64 {
        match self {
            Ptr::Null => 0,
            Ptr::Follow => FOLLOW,
            Ptr::Insert => INSERT,
            Ptr::SharedOffset => SHARED_OFFSET,
            Ptr::Shared => SHARED,
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
            Ptr::Ref { block: VIRTUAL_BLOCK, offset } => Some(offset as u64),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_iw7_shapes() {
        assert_eq!(Ptr::decode(FOLLOW), Ptr::Follow);
        assert_eq!(Ptr::decode(INSERT), Ptr::Insert);
        assert_eq!(Ptr::decode(0), Ptr::Null);
        // An asset reference: no fill.
        assert_eq!(Ptr::decode(0x0000_0008_0000_0119), Ptr::Ref { block: 8, offset: 0x118 });
        assert_eq!(Ptr::decode(0xFFFF_FFF8_0000_0119).virtual_offset(), Some(0x118));
        assert_eq!(Ptr::decode(0x3F80_0000), Ptr::Other(0x3F80_0000));
        for p in [Ptr::Null, Ptr::Follow, Ptr::Insert, Ptr::Ref { block: 8, offset: 7 }] {
            assert_eq!(Ptr::decode(p.encode()), p);
        }
    }
}
