//! The inflated zone's head: `XAssetList` (script strings, gfx globals,
//! the `XAsset` table), then the assets' data in table order. Unlike
//! Advanced Warfare's, the image does not start with the block sizes: they
//! are in the file header (`container`).
//!
//! ```text
//! 0x00 (TEMP) XAssetList { u64 stringCount; char** strings;
//!                          u64 assetCount; XAsset* assets; XGlobals* globals; } 40 bytes
//!      (VIRTUAL, aligned 8) count x char*, then the strings that "follow"
//!      (VIRTUAL, aligned 8) XGlobals { XGfxGlobals* }, XGfxGlobals (104 bytes),
//!                u64 depthStencilStateBits[], u32 blendStateBits[][4]
//!                (aligned 4), the state tables in RUNTIME, then the u32
//!                per-prim / per-object / stable constant buffer sizes[]
//!                (aligned 4)
//!      (VIRTUAL, aligned 8) assetCount x XAsset { u32 type; pad; void* header; }
//!      asset data...
//! ```
//!
//! (x64-zt `iw7/zone.cpp`, `zone_interface::build`.)

use crate::asset_type::AssetType;
use crate::cursor::{Cursor, ReadError, Result, align_up, text};
use crate::ptr::Ptr;

pub const XASSET_LIST_LEN: usize = 40;
pub const XASSET_LEN: usize = 16;
pub const XGFX_GLOBALS_LEN: usize = 104;
pub const BLOCK_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GfxGlobalsCounts {
    pub depth_stencil_states: u32,
    pub blend_states: u32,
    pub per_prim_constant_buffers: u32,
    pub per_obj_constant_buffers: u32,
    pub stable_constant_buffers: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetEntry {
    pub raw_type: u32,
    pub header: Ptr,
}

impl AssetEntry {
    pub fn asset_type(&self) -> Option<AssetType> {
        AssetType::from_u32(self.raw_type)
    }
    /// The asset's data is in this zone (not a reference to an earlier one).
    pub fn is_inline(&self) -> bool {
        self.header.is_inline()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssetList {
    /// `None` for the null entries (string 0 is always null).
    pub script_strings: Vec<Option<String>>,
    pub globals: Option<GfxGlobalsCounts>,
    pub assets: Vec<AssetEntry>,
    /// VIRTUAL offset of the `XAsset` table.
    pub asset_table_virtual: u64,
    /// File offset and VIRTUAL offset where the first asset's data begins.
    pub body_offset: usize,
    pub body_virtual: u64,
}

impl AssetList {
    /// Assets per type (in type order), with unknown types under `None`.
    pub fn counts(&self) -> Vec<(Option<AssetType>, usize)> {
        let mut map: std::collections::BTreeMap<Option<AssetType>, usize> = Default::default();
        for a in &self.assets {
            *map.entry(a.asset_type()).or_default() += 1;
        }
        map.into_iter().collect()
    }

    pub fn count_of(&self, ty: AssetType) -> usize {
        self.assets
            .iter()
            .filter(|a| a.raw_type == ty as u32)
            .count()
    }

    /// The table index a reference to an asset's header slot points at
    /// (references to whole assets are `XAsset[i].header` + 1 in VIRTUAL).
    pub fn asset_index_of(&self, ptr: Ptr) -> Option<usize> {
        let v = ptr.virtual_offset()?;
        let rel = v.checked_sub(self.asset_table_virtual + 8)?;
        (rel % XASSET_LEN as u64 == 0)
            .then_some((rel / XASSET_LEN as u64) as usize)
            .filter(|i| *i < self.assets.len())
    }
}

/// Asset counts beyond this are not a real zone.
const MAX_ASSETS: u32 = 1 << 20;
const MAX_STRINGS: u32 = 1 << 20;

pub fn parse_asset_list(zone: &[u8]) -> Result<AssetList> {
    let c = Cursor::new(zone, 0, None);
    let l = 0;
    let string_count = c.u32_at(l)?;
    let strings_ptr = c.ptr_at(l + 8)?;
    let asset_count = c.u32_at(l + 16)?;
    let assets_ptr = c.ptr_at(l + 24)?;
    let globals_ptr = c.ptr_at(l + 32)?;
    if string_count > MAX_STRINGS {
        return Err(ReadError::Invalid("script string count", l));
    }
    if asset_count > MAX_ASSETS {
        return Err(ReadError::Invalid("asset count", l + 16));
    }

    let mut c = Cursor::new(zone, l + XASSET_LIST_LEN, Some(0));
    let mut script_strings = Vec::new();
    if strings_ptr.is_inline() {
        let (table, _) = c.virt_struct(8 * string_count as usize, 8)?;
        let ptrs: Vec<Ptr> = (0..string_count as usize)
            .map(|i| c.ptr_at(table + 8 * i))
            .collect::<Result<_>>()?;
        for p in ptrs {
            script_strings.push(if p.is_inline() {
                Some(text(c.cstr()?.0))
            } else {
                None
            });
        }
    } else if strings_ptr != Ptr::Null {
        return Err(ReadError::Invalid("script string table pointer", l + 8));
    }

    let mut globals = None;
    if globals_ptr.is_inline() {
        let (slot, _) = c.virt_struct(8, 8)?;
        if c.ptr_at(slot)?.is_inline() {
            let (g, _) = c.virt_struct(XGFX_GLOBALS_LEN, 1)?;
            let counts = GfxGlobalsCounts {
                depth_stencil_states: c.u32_at(g)?,
                blend_states: c.u32_at(g + 4)?,
                per_prim_constant_buffers: c.u32_at(g + 40)?,
                per_obj_constant_buffers: c.u32_at(g + 44)?,
                stable_constant_buffers: c.u32_at(g + 48)?,
            };
            // depthStencilStateBits (u64), blendStateBits (u32[3]); the state
            // tables (g+24, g+32) are RUNTIME.
            if c.ptr_at(g + 8)?.is_inline() {
                c.virt_struct(8 * counts.depth_stencil_states as usize, 4)?;
            }
            if c.ptr_at(g + 16)?.is_inline() {
                c.virt_struct(16 * counts.blend_states as usize, 4)?;
            }
            for (off, n) in [
                (56, counts.per_prim_constant_buffers),
                (64, counts.per_obj_constant_buffers),
                (72, counts.stable_constant_buffers),
            ] {
                if c.ptr_at(g + off)?.is_inline() {
                    c.virt_struct(4 * n as usize, 4)?;
                }
            }
            globals = Some(counts);
        }
    }

    let mut assets = Vec::with_capacity(asset_count as usize);
    let mut asset_table_virtual = align_up(c.virt().unwrap_or(0), 8);
    if assets_ptr.is_inline() {
        let (table, v) = c.virt_struct(XASSET_LEN * asset_count as usize, 8)?;
        asset_table_virtual = v.unwrap_or(asset_table_virtual);
        for i in 0..asset_count as usize {
            assets.push(AssetEntry {
                raw_type: c.u32_at(table + XASSET_LEN * i)?,
                header: c.ptr_at(table + XASSET_LEN * i + 8)?,
            });
        }
    }
    Ok(AssetList {
        script_strings,
        globals,
        assets,
        asset_table_virtual,
        body_offset: c.pos(),
        body_virtual: c.virt().unwrap_or(0),
    })
}

