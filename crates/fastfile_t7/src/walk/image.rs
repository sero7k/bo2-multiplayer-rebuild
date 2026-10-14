//! Images (`GfxImage`, 0x108 bytes; x64-zt `t7/structs.hpp`).
//!
//! ```text
//! 0x00 GfxStreamedPartInfo streamedParts[4]   40 each: u32 levelCount:4 | pixelSize:28; u16 width, height;
//!                                             XPakEntryInfo { u64 key; i64 offset; u64 size; u64 bits }
//! 0xa0 u8 streamedPartCount, semantic, mapType, category, useFallback
//! 0xa8 texture, 0xb0 fallbackTexture (runtime); 0xb8 u32 flags
//! 0xc4 u16 width, height, depth; 0xcc i32 alignment (16)
//! 0xd0 u8 levelCount, fallbackLevel, streaming
//! 0xd8 u8* pixels; 0xe0 u8* fallbackPixels; 0xe8 u32 totalSize, fallbackSize; 0xf0 DXGI_FORMAT
//! 0xf8 char* name; 0x100 u32 hash
//! ```
//!
//! Stream order: the name, then the pixels (`totalSize` bytes, aligned
//! 256): in PHYSICAL, or for a streamed image (`streamedPartCount > 0`) in
//! STREAMER without file bytes; then the fallback pixels (`fallbackSize`,
//! the lowest mips, in the file) in PHYSICAL. Streamed images have no
//! pixels in the zone: their parts name the `.xpak` entries (key, offset,
//! size) that hold each mip range.

use crate::cursor::{ReadError, Result};
use crate::stream::{PHYSICAL, STREAMER, Stream};

/// `GfxImage.name` (after the four streamed parts and the texture fields).
pub const IMAGE_NAME_AT: usize = 248;
pub const STREAMED_PART: usize = 40;
/// Pixels are 256-aligned in PHYSICAL (whatever the struct's `alignment`
/// field says, 16 in every image read): `core_post_gfx`'s 86 images then
/// end PHYSICAL exactly at the header's size.
const PIXEL_ALIGN: u64 = 255;

/// One streamed part's `.xpak` entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XPakEntry {
    pub key: u64,
    pub offset: i64,
    pub size: u64,
    /// `xpakIndex:5 compressed:1 valid:1 ...`
    pub bits: u64,
    pub width: u16,
    pub height: u16,
    pub levels: u8,
}

/// The streamed parts of the image struct at `h`.
pub fn streamed_parts(s: &Stream, h: usize) -> Result<Vec<XPakEntry>> {
    let n = s.u8_at(h + 0xa0)?.min(4) as usize;
    (0..n)
        .map(|i| {
            let p = h + STREAMED_PART * i;
            Ok(XPakEntry {
                levels: (s.u32_at(p)? & 0xf) as u8,
                width: s.u16_at(p + 4)?,
                height: s.u16_at(p + 6)?,
                key: s.u64_at(p + 8)?,
                offset: s.u64_at(p + 16)? as i64,
                size: s.u64_at(p + 24)?,
                bits: s.u64_at(p + 32)?,
            })
        })
        .collect()
}

pub fn image(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::Image.header_size(), &[0xd8, 0xe0, IMAGE_NAME_AT])?;
    let name = s.string(h + IMAGE_NAME_AT)?;
    if s.i32_at(h + 0xcc)? < 0 {
        return Err(ReadError::Invalid("image alignment", h + 0xcc));
    }
    // A streamed image's pixels (its mips from the `.xpak`) take STREAMER
    // space, with no file bytes; its fallback (lowest mips) is in the file.
    let streamed = s.u8_at(h + 0xa0)? > 0;
    s.push(if streamed { STREAMER } else { PHYSICAL });
    let r = s.array(h + 0xd8, s.u32_at(h + 0xe8)? as usize, PIXEL_ALIGN);
    s.pop();
    r?;
    s.push(PHYSICAL);
    let r = s.array(h + 0xe0, s.u32_at(h + 0xec)? as usize, PIXEL_ALIGN);
    s.pop();
    r?;
    Ok(name)
}
