//! The decompressed zone: its header, script strings and asset list.
//!
//! ```text
//! 0x00 u64 script string count
//! 0x08 ptr script strings     (0 = none, -2 = follows)
//! 0x10 u64 asset count
//! 0x18 ptr assets             (-2 = follows)
//! if script strings follow: u64 ptr[count] (0 = null, -2 = string
//!    follows), then each -2 slot's NUL-terminated string in order
//!    (obfuscated: they are kept as bytes and not read)
//! XAsset[count] = { u64 type; ptr header }   (header -2 or -1)
//! then each asset's header and the data it leads to, in asset order,
//! packed with no alignment.
//! ```
//!
//! Asset data is not self-delimiting without a walker for every type, so
//! assets are found by their name hash ([`HashIndex`]): an asset header
//! starts with its hash, top bit clear where the asset is defined (a
//! reference stub, the same header size holding only the hash with the
//! top bit set, stands for an asset defined in another zone).
#![forbid(unsafe_code)]

use std::collections::HashMap;

use crate::ptr::{Bytes, OutOfBounds, Ptr};

/// Asset types (the game's own numbering) this crate reads or names.
pub mod asset_type {
    pub const XANIM: u64 = 7;
    pub const XMODEL_SURFS: u64 = 8;
    pub const XMODEL: u64 = 9;
    pub const MATERIAL: u64 = 11;
    pub const TECHSET: u64 = 20;
    pub const IMAGE: u64 = 21;
    pub const SOUNDBANK: u64 = 37;
    pub const LOCALIZE: u64 = 59;
    pub const ATTACHMENT: u64 = 60;
    pub const WEAPON: u64 = 61;
    pub const STREAMING_INFO: u64 = 80;
    pub const ANIMPKG: u64 = 92;
    pub const SFXPKG: u64 = 93;
    pub const VFXPKG: u64 = 94;
    pub const SCRIPTBUNDLE: u64 = 115;
    pub const XMODEL_DETAIL_COLLISION: u64 = 120;
    pub const WEAPON_BLUEPRINT: u64 = 163;
    pub const ATTACHMENT_BLUEPRINT: u64 = 164;
}

#[derive(Debug)]
pub enum ZoneError {
    Bounds(OutOfBounds),
    /// A header pointer of a form the layout does not have.
    BadHeader { field: &'static str, value: u64 },
    /// More entries than the zone could hold.
    TooMany { what: &'static str, count: u64 },
}

impl std::fmt::Display for ZoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bounds(e) => write!(f, "{e}"),
            Self::BadHeader { field, value } => write!(f, "zone header {field} = {value:#x}"),
            Self::TooMany { what, count } => write!(f, "{count} {what}: more than the zone holds"),
        }
    }
}

impl std::error::Error for ZoneError {}

impl From<OutOfBounds> for ZoneError {
    fn from(e: OutOfBounds) -> Self {
        Self::Bounds(e)
    }
}

/// One entry of the asset list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetEntry {
    pub kind: u64,
    pub header: Ptr,
}

/// The zone header and asset list.
#[derive(Clone, Debug, Default)]
pub struct ZoneHeader {
    /// Script string slots: `None` for a null slot, else where its
    /// (obfuscated) bytes are in the zone.
    pub script_strings: Vec<Option<(usize, usize)>>,
    pub assets: Vec<AssetEntry>,
    /// Where the first asset's header starts.
    pub data_start: usize,
}

impl ZoneHeader {
    /// How many assets of a type the zone lists.
    #[must_use]
    pub fn count(&self, kind: u64) -> usize {
        self.assets.iter().filter(|a| a.kind == kind).count()
    }
}

/// Parse a decompressed zone's header and asset list.
pub fn parse_header(zone: &[u8]) -> Result<ZoneHeader, ZoneError> {
    let b = Bytes(zone);
    let string_count = b.u64(0)?;
    let strings = b.ptr(8)?;
    let asset_count = b.u64(0x10)?;
    let assets = b.ptr(0x18)?;
    let mut at = 0x20usize;
    let mut out = ZoneHeader::default();
    let limit = zone.len() as u64 / 8;
    if string_count > limit {
        return Err(ZoneError::TooMany { what: "script strings", count: string_count });
    }
    if asset_count > limit / 2 {
        return Err(ZoneError::TooMany { what: "assets", count: asset_count });
    }
    match strings {
        Ptr::Null => {}
        Ptr::Follow | Ptr::Insert => {
            let slots: Vec<Ptr> = (0..string_count as usize).map(|i| b.ptr(at + 8 * i)).collect::<Result<_, _>>()?;
            at += 8 * string_count as usize;
            for p in slots {
                if p.follows() {
                    let start = at;
                    let (_, next) = b.cstr(at)?;
                    out.script_strings.push(Some((start, next - 1 - start)));
                    at = next;
                } else {
                    out.script_strings.push(None);
                }
            }
        }
        _ => return Err(ZoneError::BadHeader { field: "script strings", value: b.u64(8)? }),
    }
    match assets {
        Ptr::Null => {}
        Ptr::Follow | Ptr::Insert => {
            for i in 0..asset_count as usize {
                out.assets.push(AssetEntry { kind: b.u64(at + 16 * i)?, header: b.ptr(at + 16 * i + 8)? });
            }
            at += 16 * asset_count as usize;
        }
        _ => return Err(ZoneError::BadHeader { field: "assets", value: b.u64(0x18)? }),
    }
    out.data_start = at;
    Ok(out)
}

/// Where hashes of interest are stored as u64s in a zone (every 1-byte
/// position, since the stream is unaligned). Built for a set of wanted
/// hashes in one pass over the zone.
#[derive(Clone, Debug, Default)]
pub struct HashIndex {
    /// hash (top bit clear) -> (defined at, referenced at) offsets.
    pub hits: HashMap<u64, Hits>,
}

#[derive(Clone, Debug, Default)]
pub struct Hits {
    /// Offsets where the hash is stored with its top bit clear.
    pub plain: Vec<usize>,
    /// Offsets where it is stored with its top bit set (a reference).
    pub reference: Vec<usize>,
}

impl HashIndex {
    /// Every offset of `zone` holding one of `wanted` (with or without the
    /// reference bit), at most `per_hash` of each kind.
    #[must_use]
    pub fn build(zone: &[u8], wanted: impl IntoIterator<Item = u64>, per_hash: usize) -> Self {
        let mut hits: HashMap<u64, Hits> = wanted.into_iter().map(|h| (h & crate::hash::MASK, Hits::default())).collect();
        if hits.is_empty() || zone.len() < 8 {
            return Self { hits };
        }
        // (A cheap prefilter on the low 16 bits before the map lookup.)
        let mut low = vec![false; 1 << 16];
        for h in hits.keys() {
            low[(*h & 0xffff) as usize] = true;
        }
        for at in 0..=zone.len() - 8 {
            let lo = u16::from_le_bytes([zone[at], zone[at + 1]]);
            if !low[lo as usize] {
                continue;
            }
            let v = u64::from_le_bytes(zone[at..at + 8].try_into().unwrap_or([0; 8]));
            if let Some(h) = hits.get_mut(&(v & crate::hash::MASK)) {
                let list = if v & crate::hash::REF_BIT == 0 { &mut h.plain } else { &mut h.reference };
                if list.len() < per_hash {
                    list.push(at);
                }
            }
        }
        Self { hits }
    }

    #[must_use]
    pub fn plain(&self, hash: u64) -> &[usize] {
        self.hits.get(&(hash & crate::hash::MASK)).map_or(&[], |h| h.plain.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le(v: u64) -> [u8; 8] {
        v.to_le_bytes()
    }

    #[test]
    fn header_with_strings_and_assets() {
        let mut z = Vec::new();
        z.extend(le(2));
        z.extend(le(crate::ptr::FOLLOW));
        z.extend(le(2));
        z.extend(le(crate::ptr::FOLLOW));
        z.extend(le(0));
        z.extend(le(crate::ptr::FOLLOW));
        z.extend(b"\x83ab\0");
        z.extend(le(9));
        z.extend(le(crate::ptr::FOLLOW));
        z.extend(le(8));
        z.extend(le(crate::ptr::INSERT));
        let h = parse_header(&z).unwrap();
        assert_eq!(h.script_strings, vec![None, Some((0x30, 3))]);
        assert_eq!(h.assets[0], AssetEntry { kind: 9, header: Ptr::Follow });
        assert_eq!(h.assets[1].header, Ptr::Insert);
        assert_eq!(h.data_start, z.len());
        assert_eq!(h.count(asset_type::XMODEL), 1);
    }

    #[test]
    fn hash_index_finds_unaligned() {
        let h = crate::hash::name("mp_jup_rust");
        let mut z = vec![0u8; 3];
        z.extend(le(h));
        z.push(7);
        z.extend(le(h | crate::hash::REF_BIT));
        let ix = HashIndex::build(&z, [h], 8);
        assert_eq!(ix.plain(h), &[3]);
        assert_eq!(ix.hits[&h].reference, vec![12]);
    }
}
