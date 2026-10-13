//! The inflated zone's head: the `XFile` sizes, then `XAssetList`
//! (script strings, gfx globals, the `XAsset` table), then the assets'
//! data in table order.
//!
//! ```text
//! 0x00 XFile    { u64 size; u64 externalSize; u64 blockSize[7]; }   72 bytes
//! 0x48 XAssetList { ScriptStringList { i32 count; char** strings; };
//!                   i32 assetCount; XAsset* assets; XGlobals* globals; } 40 bytes
//!      (VIRTUAL) count x char*, then the strings that "follow"
//!      (VIRTUAL) XGlobals { XGfxGlobals* }, XGfxGlobals (104 bytes),
//!                u64 depthStencilStateBits[], u32 blendStateBits[][3],
//!                u32 per-prim / per-object / stable constant buffer sizes[]
//!                (the state and buffer tables are RUNTIME: not in the file)
//!      (VIRTUAL, aligned 4) assetCount x XAsset { u32 type; pad; void* header; }
//!      asset data...
//! ```
//!
//! The VIRTUAL alignments are the loader's (as the x64 zone tools write
//! them): the string table and globals unaligned, each globals array and the
//! asset table on 4. They were checked against the retail zones through
//! the string references of their sound aliases (see `scan.rs`).

use crate::asset_type::AssetType;
use crate::cursor::{Cursor, ReadError, Result, align_up, text};
use crate::ptr::Ptr;

pub const XFILE_HEADER_LEN: usize = 72;
pub const XASSET_LIST_LEN: usize = 40;
pub const XASSET_LEN: usize = 16;
pub const XGFX_GLOBALS_LEN: usize = 104;
pub const BLOCK_COUNT: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Block {
    Temp = 0,
    Physical = 1,
    Runtime = 2,
    Virtual = 3,
    Large = 4,
    Callback = 5,
    Script = 6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XFile {
    pub size: u64,
    pub external_size: u64,
    pub block_sizes: [u64; BLOCK_COUNT],
}

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
    pub xfile: XFile,
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
    let mut block_sizes = [0u64; BLOCK_COUNT];
    for (i, b) in block_sizes.iter_mut().enumerate() {
        *b = c.u64_at(16 + 8 * i)?;
    }
    let xfile = XFile {
        size: c.u64_at(0)?,
        external_size: c.u64_at(8)?,
        block_sizes,
    };
    let l = XFILE_HEADER_LEN;
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
        let (table, _) = c.virt_struct(8 * string_count as usize, 1)?;
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
        let (slot, _) = c.virt_struct(8, 1)?;
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
                c.virt_struct(12 * counts.blend_states as usize, 4)?;
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
    let mut asset_table_virtual = align_up(c.virt().unwrap_or(0), 4);
    if assets_ptr.is_inline() {
        let (table, v) = c.virt_struct(XASSET_LEN * asset_count as usize, 4)?;
        asset_table_virtual = v.unwrap_or(asset_table_virtual);
        for i in 0..asset_count as usize {
            assets.push(AssetEntry {
                raw_type: c.u32_at(table + XASSET_LEN * i)?,
                header: c.ptr_at(table + XASSET_LEN * i + 8)?,
            });
        }
    }
    Ok(AssetList {
        xfile,
        script_strings,
        globals,
        assets,
        asset_table_virtual,
        body_offset: c.pos(),
        body_virtual: c.virt().unwrap_or(0),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ptr::FOLLOW;

    /// Builds a zone image head: strings, optional globals, and the asset
    /// table; returns the bytes (body empty).
    pub(crate) fn zone_head(
        strings: &[Option<&str>],
        globals: bool,
        assets: &[(u32, u64)],
    ) -> Vec<u8> {
        let mut z = vec![0u8; XFILE_HEADER_LEN];
        z[16 + 8 * 3..16 + 8 * 4].copy_from_slice(&0x1000u64.to_le_bytes());
        z.extend_from_slice(&(strings.len() as u64).to_le_bytes());
        z.extend_from_slice(&FOLLOW.to_le_bytes());
        z.extend_from_slice(&(assets.len() as u64).to_le_bytes());
        z.extend_from_slice(&FOLLOW.to_le_bytes());
        z.extend_from_slice(&(if globals { FOLLOW } else { 0 }).to_le_bytes());
        for s in strings {
            z.extend_from_slice(&(if s.is_some() { FOLLOW } else { 0 }).to_le_bytes());
        }
        for s in strings.iter().flatten() {
            z.extend_from_slice(s.as_bytes());
            z.push(0);
        }
        if globals {
            z.extend_from_slice(&FOLLOW.to_le_bytes());
            let mut g = vec![0u8; XGFX_GLOBALS_LEN];
            g[0..4].copy_from_slice(&2u32.to_le_bytes()); // depth stencil
            g[4..8].copy_from_slice(&3u32.to_le_bytes()); // blend
            g[40..44].copy_from_slice(&1u32.to_le_bytes());
            for off in [8, 16, 24, 32, 56, 64, 72, 80, 88, 96] {
                g[off..off + 8].copy_from_slice(&FOLLOW.to_le_bytes());
            }
            z.extend_from_slice(&g);
            z.extend_from_slice(&[0x11; 16]); // 2 x u64
            z.extend_from_slice(&[0x22; 36]); // 3 x u32[3]
            z.extend_from_slice(&[0x33; 4]); // 1 x u32
        }
        for (t, h) in assets {
            z.extend_from_slice(&(*t as u64).to_le_bytes());
            z.extend_from_slice(&h.to_le_bytes());
        }
        z
    }

    #[test]
    fn parses_strings_globals_and_table() {
        let z = zone_head(
            &[None, Some("end"), Some("dm")],
            true,
            &[(0x39, FOLLOW), (0x11, FOLLOW)],
        );
        let list = parse_asset_list(&z).unwrap();
        assert_eq!(
            list.script_strings,
            vec![None, Some("end".into()), Some("dm".into())]
        );
        assert_eq!(list.xfile.block_sizes[Block::Virtual as usize], 0x1000);
        assert_eq!(list.globals.unwrap().blend_states, 3);
        assert_eq!(list.assets.len(), 2);
        assert_eq!(list.assets[1].asset_type(), Some(AssetType::Sound));
        assert_eq!(list.body_offset, z.len());
        // VIRTUAL: 24 (ptrs) + 7 (strings) = 31; +8 slot +104 globals = 143;
        // align 4 -> 144 +16 = 160 -> +36 = 196 -> +4 = 200; table at 200.
        assert_eq!(list.asset_table_virtual, 200);
        assert_eq!(list.body_virtual, 200 + 32);
        let header_ref = Ptr::Ref {
            block: 3,
            offset: (200 + 16 + 8) as u32,
        };
        assert_eq!(list.asset_index_of(header_ref), Some(1));
        assert_eq!(
            list.counts(),
            vec![
                (Some(AssetType::Sound), 1),
                (Some(AssetType::NetConstStrings), 1)
            ]
        );
    }
}
