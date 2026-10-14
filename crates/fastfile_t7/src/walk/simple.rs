//! The asset types with a flat layout: a header, the name, one buffer.
//!
//! ```text
//! LocalizeEntry   { char* value; char* name; }                       16
//!                 value, then name follow
//! RawFile         { char* name; i32 len; pad; char* buffer; }         24
//!                 name, then buffer (len + 1 bytes, NUL included)
//! ScriptParseTree { char* name; i32 len; pad; u8* buffer; }           24
//!                 name, then buffer (len + 1: the compiled "\x80GSC\r\n" script and a NUL)
//! TTFDef          { char* name; i32 len; pad; u8* file; 0x20008 bytes inline } 0x20020
//!                 name, then file (len + 1 bytes, a NUL after the font); most retail fonts are
//!                 "," placeholders (len 0, no file)
//! placeholders    "," assets (header and name only) of the types with a
//!                 known header: techset, material, xmodel, xmodelmesh,
//!                 xanim, physpreset, physconstraints, image
//! ```
//!
//! (Read on retail `core_post_gfx`, `en_mp_patch`, `zm_factory_patch`:
//! the next asset's header starts right after each buffer.)

use crate::asset_type::AssetType;
use crate::cursor::{Cursor, ReadError};

pub fn localize(c: &mut Cursor) -> Result<Option<String>, ReadError> {
    let h = c.take(AssetType::Localize.header_size())?;
    c.string_field(h)?;
    c.string_field(h + 8)
}

/// A name and a `len + 1`-byte buffer (rawfile, scriptparsetree).
fn named_buffer(c: &mut Cursor, header: usize) -> Result<Option<String>, ReadError> {
    let h = c.take(header)?;
    let name = c.string_field(h)?;
    let len = c.i32_at(h + 8)?;
    let buffer = c.ptr_at(h + 16)?;
    if buffer.is_inline() {
        if len < 0 {
            return Err(ReadError::Invalid("buffer length", h + 8));
        }
        c.take(len as usize + 1)?;
    }
    Ok(name)
}

pub fn rawfile(c: &mut Cursor) -> Result<Option<String>, ReadError> {
    named_buffer(c, AssetType::RawFile.header_size())
}

pub fn script_parse_tree(c: &mut Cursor) -> Result<Option<String>, ReadError> {
    named_buffer(c, AssetType::ScriptParseTree.header_size())
}

pub fn ttf(c: &mut Cursor) -> Result<Option<String>, ReadError> {
    let h = c.take(AssetType::Ttf.header_size())?;
    let name = c.string_field(h)?;
    let len = c.i32_at(h + 8)?;
    if c.ptr_at(h + 16)?.is_inline() {
        if len < 0 {
            return Err(ReadError::Invalid("ttf file length", h + 8));
        }
        c.take(len as usize + 1)?;
    }
    Ok(name)
}

/// An asset that is only its header and its name: the "," placeholders
/// that stand in for an asset another zone loads (`,mc/vegetation#e6142445`
/// as a technique set, `,fonts/escom.ttf`). Every other 8-byte field of the
/// header must not load data here; an asset that does fails, naming the
/// field, until its type gets a real walker.
pub fn placeholder(c: &mut Cursor, ty: AssetType, name_at: usize) -> Result<Option<String>, ReadError> {
    let size = ty.header_size();
    let h = c.take(size)?;
    for off in (0..size).step_by(8).filter(|o| *o != name_at) {
        if c.ptr_at(h + off)?.is_inline() {
            return Err(ReadError::Invalid("loaded field (no walker for this type yet)", h + off));
        }
    }
    c.string_field(h + name_at)
}

/// `GfxImage.name` (after the four streamed parts and the texture fields).
pub const IMAGE_NAME_AT: usize = 248;
