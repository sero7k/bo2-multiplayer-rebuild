//! The asset types with a flat layout: a header, the name, one buffer.
//!
//! ```text
//! LocalizeEntry   { char* value; char* name; }                       16
//!                 value, then name follow
//! RawFile         { char* name; i32 len; pad; char* buffer; }         24
//!                 name, then buffer (len + 1 bytes, NUL included; aligned 16)
//! ScriptParseTree { char* name; i32 len; pad; u8* buffer; }           24
//!                 name, then buffer (len + 1: the compiled "\x80GSC\r\n" script and a NUL;
//!                 aligned 32: `mp_patch` and `mp_common` resolve with it)
//! TTFDef          { char* name; i32 len; pad; u8* file; 0x20008 bytes inline } 0x20020
//!                 name, then file (len + 1 bytes, a NUL after the font); most retail fonts are
//!                 "," placeholders (len 0, no file)
//! KeyValuePairs   { char* name; i32 count; pad; KeyValuePair* pairs }  24
//! ```
//!
//! (Read on retail `core_post_gfx`, `en_mp_patch`, `zm_factory_patch`:
//! the next asset's header starts right after each buffer. The buffers'
//! 16-byte VIRTUAL alignment is what makes the sound banks' references
//! after 125 rawfiles, and after a real font, land on their strings.)

use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result};
use crate::stream::Stream;

pub fn localize(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let value = s.string(h)?;
    let key = s.string(h + 8)?;
    // bo2mp: kept with the animations (the guns' names).
    if s.capture.keep_anims
        && let (Some(k), Some(v)) = (&key, value)
    {
        s.capture.localize.push((k.clone(), v));
    }
    Ok(key)
}

/// A name and a `len + 1`-byte buffer (rawfile, scriptparsetree, ttf).
fn named_buffer(s: &mut Stream, h: usize) -> Result<Option<String>> {
    named_buffer_aligned(s, h, 15)
}

fn named_buffer_aligned(s: &mut Stream, h: usize, mask: u64) -> Result<Option<String>> {
    let name = s.string(h)?;
    let len = s.i32_at(h + 8)?;
    if s.ptr_at(h + 16)?.is_inline() && len < 0 {
        return Err(ReadError::Invalid("buffer length", h + 8));
    }
    s.array(h + 16, len.max(0) as usize + 1, mask)?;
    Ok(name)
}

pub fn rawfile(s: &mut Stream, h: usize) -> Result<Option<String>> {
    named_buffer(s, h)
}

pub fn script_parse_tree(s: &mut Stream, h: usize) -> Result<Option<String>> {
    named_buffer_aligned(s, h, 31)
}

pub fn ttf(s: &mut Stream, h: usize) -> Result<Option<String>> {
    named_buffer(s, h)
}

/// `KeyValuePairs { char* name; i32 count; KeyValuePair* pairs }` (24),
/// `KeyValuePair { u32 keyHash; u32 namespaceHash; char* value }` (16):
/// name, pairs (aligned 8), then each value.
pub fn key_value_pairs(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::KeyValuePairs.header_size(), &[0, 16])?;
    let name = s.string(h)?;
    let n = s.i32_at(h + 8)?.max(0) as usize;
    if let Some(t) = s.records(h + 16, n, 16, 7)? {
        for i in 0..n {
            s.string(t + 16 * i + 8)?;
        }
    }
    Ok(name)
}

/// `FontIcon { char* name; u32 numEntries, numAliases; FontIconEntry*;
/// FontIconAlias* }` (32): name, the entries (40 each: `{ char* name;
/// u32 hash; Material*; i32 size; f32 xScale, yScale }`), each entry's name
/// and material, then the aliases (12 each: three u32 hashes, aligned 4).
pub fn font_icon(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::FontIcon.header_size(), &[0, 16, 24])?;
    let entries = s.u32_at(h + 8)? as usize;
    let name = s.string(h)?;
    if let Some(t) = s.records(h + 16, entries, 40, 7)? {
        for i in 0..entries {
            let e = t + 40 * i;
            s.no_loads(e, 40, &[0, 16])?;
            s.string(e)?;
            s.asset(AssetType::Material, e + 16)?;
        }
    }
    let aliases = s.u32_at(h + 12)? as usize;
    s.records(h + 24, aliases, 12, 3)?;
    Ok(name)
}

/// `StringTable { char* name; i32 columnCount, rowCount; StringTableCell*
/// values; i16* cellIndex }` (32), `StringTableCell { char* string; u32
/// hash }` (16): name, the cells, each cell's string, then the cell index
/// (`rows * columns` i16s).
pub fn string_table(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::StringTable.header_size(), &[0, 16, 24])?;
    let name = s.string(h)?;
    let cells = s.i32_at(h + 8)?.max(0) as usize * s.i32_at(h + 12)?.max(0) as usize;
    if let Some(t) = s.records(h + 16, cells, 16, 7)? {
        for i in 0..cells {
            s.string(t + 16 * i)?;
        }
    }
    s.records(h + 24, cells, 2, 1)?;
    Ok(name)
}

/// `TextureCombo` (0x80, read on retail `mp_common`/`mp_patch`; not in
/// x64-zt): `{ char* name; Record* records; u64 count; zero... }`, records
/// 0x30 each: `{ u64; Entry* entries; u64 entryCount; GfxImage* image;
/// 16 bytes of small fields }`, entries 16: `{ Material* material; u64 }`.
/// Order: name, records, then per record its entries, each entry's
/// material, then its image.
///
/// Map zones (`mp_nuketown_x`) also fill the header's four pointers at
/// 0x28, 0x40, 0x58, 0x70 (`-2`, null in `mp_common`): images, loaded after
/// the records.
pub fn texture_combo(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::TextureCombo.header_size(), &[0, 8, 0x28, 0x40, 0x58, 0x70])?;
    let name = s.string(h)?;
    let n = s.u64_at(h + 16)? as usize;
    if let Some(t) = s.records(h + 8, n, 0x30, 7)? {
        for i in 0..n {
            let r = t + 0x30 * i;
            s.no_loads(r, 0x30, &[8, 0x18])?;
            let entries = s.u64_at(r + 16)? as usize;
            if let Some(e) = s.records(r + 8, entries, 16, 7)? {
                for k in 0..entries {
                    s.asset(AssetType::Material, e + 16 * k)?;
                }
            }
            s.asset(AssetType::Image, r + 0x18)?;
        }
    }
    for at in [0x28, 0x40, 0x58, 0x70] {
        s.asset(AssetType::Image, h + at)?;
    }
    Ok(name)
}

/// `Bitfield` (0x20, read on retail `mp_common`): `{ char* name; u32;
/// u32 count; u32 words; pad; u64* data }`: name, then `count * words`
/// u64s (no pointers; `words` is 2 or 8 in the retail state-field tables).
pub fn bitfield(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::Bitfield.header_size(), &[0, 0x18])?;
    let name = s.string(h)?;
    let n = s.u32_at(h + 0xc)? as usize * s.u32_at(h + 0x10)? as usize;
    s.records(h + 0x18, n, 8, 7)?;
    Ok(name)
}

/// An asset that is only its header and its name: the "," placeholders
/// that stand in for an asset another zone loads (`,mc/vegetation#e6142445`
/// as a technique set, `,fonts/escom.ttf`). Every other 8-byte field of the
/// header must not load data here.
pub fn placeholder(s: &mut Stream, ty: AssetType, h: usize, name_at: usize) -> Result<Option<String>> {
    s.no_loads(h, ty.header_size(), &[name_at])?;
    s.string(h + name_at)
}
