//! The inflated zone's head: `XAssetList`, its script strings and depends,
//! the `XAsset` table, then the assets' data in table order.
//!
//! ```text
//! 0x00 XAssetList { i32 stringCount; pad; char** strings;
//!                   i32 dependCount; pad; char** depends;
//!                   i32 assetCount; pad; XAsset* assets; }       48 bytes
//! 0x30 (strings = -1) stringCount x char*, each followed in turn by its
//!      string when -1 (string 0 is null)
//!      (depends = -1) dependCount x char*, then their strings
//!      (assets = -1) assetCount x XAsset { u32 type; pad; void* header; }
//!      asset data...
//! ```
//!
//! There is no padding in the stream: with 54 script strings the table of
//! `core_gfx` starts at an odd offset (0x467). A header pointer is `-1` when
//! the asset's data follows (in table order), or a reference
//! (`5 << 60 | offset + 1`) to an asset the zone loaded earlier, which is
//! how each zone's image list ends (every image the zone's materials
//! already loaded).

use crate::asset_type::AssetType;
use crate::cursor::{Cursor, ReadError, Result};
use crate::ptr::Ptr;

pub const XASSET_LIST_LEN: usize = 48;
pub const XASSET_LEN: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetEntry {
    pub raw_type: u32,
    pub header: Ptr,
}

impl AssetEntry {
    pub fn asset_type(&self) -> Option<AssetType> {
        AssetType::from_u32(self.raw_type)
    }
    /// The asset's data is in the stream here (not a reference to one loaded earlier).
    pub fn is_inline(&self) -> bool {
        self.header.is_inline()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssetList {
    /// `None` for null entries (string 0 is null) and references.
    pub script_strings: Vec<Option<String>>,
    pub depends: Vec<Option<String>>,
    pub assets: Vec<AssetEntry>,
    /// File offset of the `XAsset` table.
    pub table_offset: usize,
    /// File offset where the first asset's data begins.
    pub body_offset: usize,
}

impl AssetList {
    /// Assets per type (in type order), unknown types under `None`.
    pub fn counts(&self) -> Vec<(Option<AssetType>, usize)> {
        let mut map: std::collections::BTreeMap<Option<AssetType>, usize> = Default::default();
        for a in &self.assets {
            *map.entry(a.asset_type()).or_default() += 1;
        }
        map.into_iter().collect()
    }

    /// Assets per type, inline ones and references apart.
    pub fn counts_split(&self) -> Vec<(Option<AssetType>, usize, usize)> {
        let mut map: std::collections::BTreeMap<Option<AssetType>, (usize, usize)> = Default::default();
        for a in &self.assets {
            let e = map.entry(a.asset_type()).or_default();
            if a.is_inline() {
                e.0 += 1;
            } else {
                e.1 += 1;
            }
        }
        map.into_iter().map(|(t, (i, r))| (t, i, r)).collect()
    }

    pub fn count_of(&self, ty: AssetType) -> usize {
        self.assets.iter().filter(|a| a.raw_type == ty as u32).count()
    }
}

/// Counts beyond these are not a real zone.
const MAX_ASSETS: i32 = 1 << 20;
const MAX_STRINGS: i32 = 1 << 20;

fn string_table(c: &mut Cursor, count: i32, ptr: Ptr, field: usize) -> Result<Vec<Option<String>>> {
    match ptr {
        Ptr::Null => Ok(Vec::new()),
        p if p.is_inline() => {
            let table = c.take(8 * count as usize)?;
            (0..count as usize).map(|i| c.string_field(table + 8 * i)).collect()
        }
        _ => Err(ReadError::Invalid("string table pointer", field)),
    }
}

pub fn parse_asset_list(zone: &[u8]) -> Result<AssetList> {
    let mut c = Cursor::new(zone, 0);
    let l = c.take(XASSET_LIST_LEN)?;
    let string_count = c.i32_at(l)?;
    let strings_ptr = c.ptr_at(l + 8)?;
    let depend_count = c.i32_at(l + 16)?;
    let depends_ptr = c.ptr_at(l + 24)?;
    let asset_count = c.i32_at(l + 32)?;
    let assets_ptr = c.ptr_at(l + 40)?;
    if !(0..=MAX_STRINGS).contains(&string_count) {
        return Err(ReadError::Invalid("script string count", l));
    }
    if !(0..=MAX_STRINGS).contains(&depend_count) {
        return Err(ReadError::Invalid("depend count", l + 16));
    }
    if !(0..=MAX_ASSETS).contains(&asset_count) {
        return Err(ReadError::Invalid("asset count", l + 32));
    }
    let script_strings = string_table(&mut c, string_count, strings_ptr, l + 8)?;
    let depends = string_table(&mut c, depend_count, depends_ptr, l + 24)?;

    let mut assets = Vec::with_capacity(asset_count as usize);
    let table_offset = c.pos();
    match assets_ptr {
        Ptr::Null => {}
        p if p.is_inline() => {
            let table = c.take(XASSET_LEN * asset_count as usize)?;
            for i in 0..asset_count as usize {
                assets.push(AssetEntry {
                    raw_type: c.u32_at(table + XASSET_LEN * i)?,
                    header: c.ptr_at(table + XASSET_LEN * i + 8)?,
                });
            }
        }
        _ => return Err(ReadError::Invalid("asset table pointer", l + 40)),
    }
    Ok(AssetList {
        script_strings,
        depends,
        assets,
        table_offset,
        body_offset: c.pos(),
    })
}

/// The non-null script strings, for printing.
pub fn script_string_texts(list: &AssetList) -> impl Iterator<Item = &str> {
    list.script_strings.iter().filter_map(|s| s.as_deref())
}

