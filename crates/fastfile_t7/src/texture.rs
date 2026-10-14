//! Image pixels: a `GfxImage`'s mip chain from the zone (inline pixels) or
//! its streamed parts in the `.xpak` packages, written as `.dds` (DX10
//! header, the image's own DXGI format) or decoded to RGBA8.
//!
//! A streamed image has up to four parts (`ImageRec::xpak`, smallest
//! first), each naming its top width x height. Part 0 decodes to its whole
//! mip chain (its top level down to 1x1); each later part to its top level
//! only. The full chain is the last part, then the one before, ... then
//! part 0. (Read on retail `i_wpn_t7_arak_plastic_c`, BC1 2048x2048: the
//! 256 part decodes to 0xaab8 bytes, the chain from 256; the 512 part to
//! 0x20000, the 2048 part to 0x200000.)

use crate::walk::capture::ImageRec;
use crate::xpak::{XpakError, XpakSet};

/// (bytes per 4x4 block, true) for block-compressed formats, (bytes per
/// pixel, false) for the others; None for formats not handled.
pub fn dxgi_layout(format: u32) -> Option<(usize, bool)> {
    Some(match format {
        70..=72 | 79..=81 => (8, true),
        73..=78 | 82..=84 | 94..=99 => (16, true),
        1..=4 => (16, false),
        9..=14 => (8, false),
        15..=22 => (8, false),
        23..=47 => (4, false),
        48..=59 => (2, false),
        60..=65 => (1, false),
        85..=86 => (2, false),
        87..=93 => (4, false),
        _ => return None,
    })
}

pub fn dxgi_name(format: u32) -> &'static str {
    match format {
        2 => "R32G32B32A32_FLOAT",
        10 => "R16G16B16A16_FLOAT",
        11 => "R16G16B16A16_UNORM",
        24 => "R10G10B10A2_UNORM",
        26 => "R11G11B10_FLOAT",
        28 => "R8G8B8A8_UNORM",
        29 => "R8G8B8A8_UNORM_SRGB",
        34 => "R16G16_FLOAT",
        41 => "R32_FLOAT",
        49 => "R8G8_UNORM",
        54 => "R16_FLOAT",
        56 => "R16_UNORM",
        61 => "R8_UNORM",
        65 => "A8_UNORM",
        71 => "BC1_UNORM",
        72 => "BC1_UNORM_SRGB",
        74 => "BC2_UNORM",
        75 => "BC2_UNORM_SRGB",
        77 => "BC3_UNORM",
        78 => "BC3_UNORM_SRGB",
        80 => "BC4_UNORM",
        81 => "BC4_SNORM",
        83 => "BC5_UNORM",
        84 => "BC5_SNORM",
        87 => "B8G8R8A8_UNORM",
        91 => "B8G8R8A8_UNORM_SRGB",
        95 => "BC6H_UF16",
        96 => "BC6H_SF16",
        98 => "BC7_UNORM",
        99 => "BC7_UNORM_SRGB",
        _ => "?",
    }
}

/// Bytes of one level `w` x `h`.
pub fn level_size(format: u32, w: u32, h: u32) -> Option<usize> {
    let (n, block) = dxgi_layout(format)?;
    let (w, h) = (w.max(1) as usize, h.max(1) as usize);
    Some(if block { w.div_ceil(4) * h.div_ceil(4) * n } else { w * h * n })
}

/// The number of levels from `w` x `h` down to 1x1.
pub fn full_levels(w: u32, h: u32) -> u32 {
    32 - w.max(h).max(1).leading_zeros()
}

/// Bytes of a mip chain of `levels` levels from `w` x `h` down.
pub fn chain_size(format: u32, w: u32, h: u32, levels: u32) -> Option<usize> {
    (0..levels).map(|l| level_size(format, w >> l, h >> l)).sum()
}

/// One image's pixels at its best available size.
#[derive(Clone, Debug)]
pub struct ImageLevels {
    pub format: u32,
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    /// The mip chain, biggest level first.
    pub data: Vec<u8>,
    /// Which streamed part is the top level (None: the zone's pixels).
    pub part: Option<usize>,
}

#[derive(Debug)]
pub enum TextureError {
    Xpak(XpakError),
    NoPixels,
    Format(u32),
    Size { what: &'static str, want: usize, got: usize },
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xpak(e) => e.fmt(f),
            Self::NoPixels => write!(f, "image has neither pixels in the zone nor streamed parts"),
            Self::Format(x) => write!(f, "DXGI format {x} not handled"),
            Self::Size { what, want, got } => write!(f, "{what}: {got:#x} bytes, expected {want:#x}"),
        }
    }
}

impl std::error::Error for TextureError {}

impl From<XpakError> for TextureError {
    fn from(e: XpakError) -> Self {
        Self::Xpak(e)
    }
}

/// The image with its pixels in the zone (`zone_image` is the inflated zone).
pub fn zone_levels(im: &ImageRec, zone_image: &[u8]) -> Result<ImageLevels, TextureError> {
    let (at, len) = im.pixels.ok_or(TextureError::NoPixels)?;
    let data = zone_image.get(at..at + len).ok_or(TextureError::NoPixels)?.to_vec();
    Ok(ImageLevels {
        format: im.format,
        width: im.width as u32,
        height: im.height as u32,
        levels: (im.levels as u32).max(1),
        data,
        part: None,
    })
}

/// A streamed image's full chain from the packages: every part, checked
/// against the sizes its width x height give.
pub fn streamed_levels(im: &ImageRec, paks: &XpakSet) -> Result<ImageLevels, TextureError> {
    let last = im.xpak.len().checked_sub(1).ok_or(TextureError::NoPixels)?;
    let f = im.format;
    let mut data = Vec::new();
    for (i, p) in im.xpak.iter().enumerate().rev() {
        let part = paks.read(p.key)?;
        let (w, h) = (p.width as u32, p.height as u32);
        let want = if i == 0 { chain_size(f, w, h, full_levels(w, h)) } else { level_size(f, w, h) }.ok_or(TextureError::Format(f))?;
        if part.len() != want {
            return Err(TextureError::Size { what: "streamed part", want, got: part.len() });
        }
        data.extend_from_slice(&part);
    }
    let top = &im.xpak[last];
    let (w, h) = (top.width as u32, top.height as u32);
    let levels = last as u32 + full_levels(im.xpak[0].width as u32, im.xpak[0].height as u32);
    Ok(ImageLevels { format: f, width: w, height: h, levels, data, part: Some(last) })
}

/// `streamed_levels` for a streamed image, else `zone_levels`.
pub fn image_levels(im: &ImageRec, zone_image: &[u8], paks: &XpakSet) -> Result<ImageLevels, TextureError> {
    if im.xpak.is_empty() { zone_levels(im, zone_image) } else { streamed_levels(im, paks) }
}

/// A `.dds` file (DX10 extended header) of the levels.
pub fn to_dds(l: &ImageLevels) -> Vec<u8> {
    let mut o = Vec::with_capacity(148 + l.data.len());
    let u = |o: &mut Vec<u8>, v: u32| o.extend_from_slice(&v.to_le_bytes());
    o.extend_from_slice(b"DDS ");
    u(&mut o, 124);
    // CAPS | HEIGHT | WIDTH | PIXELFORMAT | MIPMAPCOUNT | LINEARSIZE
    u(&mut o, 0x1 | 0x2 | 0x4 | 0x1000 | 0x20000 | 0x80000);
    u(&mut o, l.height);
    u(&mut o, l.width);
    u(&mut o, level_size(l.format, l.width, l.height).unwrap_or(0) as u32);
    u(&mut o, 0);
    u(&mut o, l.levels);
    for _ in 0..11 {
        u(&mut o, 0);
    }
    u(&mut o, 32);
    u(&mut o, 0x4);
    o.extend_from_slice(b"DX10");
    for _ in 0..5 {
        u(&mut o, 0);
    }
    u(&mut o, 0x1000 | 0x400000 | 0x8);
    for _ in 0..4 {
        u(&mut o, 0);
    }
    u(&mut o, l.format);
    u(&mut o, 3); // TEXTURE2D
    u(&mut o, 0);
    u(&mut o, 1);
    u(&mut o, 0);
    o.extend_from_slice(&l.data);
    o
}

/// Level `mip` decoded to RGBA8: (width, height, pixels). Block formats
/// via `bcdec_rs` (BC4 grey, BC5 red-green); BC6H not handled.
pub fn level_rgba(l: &ImageLevels, mip: u32) -> Option<(u32, u32, Vec<u8>)> {
    if mip >= l.levels {
        return None;
    }
    let start: usize = (0..mip).map(|m| level_size(l.format, l.width >> m, l.height >> m)).sum::<Option<usize>>()?;
    let (lw, lh) = ((l.width >> mip).max(1), (l.height >> mip).max(1));
    let (w, h) = (lw as usize, lh as usize);
    let (n, block) = dxgi_layout(l.format)?;
    let src = l.data.get(start..start + level_size(l.format, lw, lh)?)?;
    let mut out = vec![0u8; w * h * 4];
    if block {
        let bw = w.div_ceil(4);
        let mut tmp = [0u8; 64];
        for (i, b) in src.chunks_exact(n).enumerate() {
            let (bx, by) = (i % bw * 4, i / bw * 4);
            match l.format {
                70..=72 => bcdec_rs::bc1(b, &mut tmp, 16),
                73..=75 => bcdec_rs::bc2(b, &mut tmp, 16),
                76..=78 => bcdec_rs::bc3(b, &mut tmp, 16),
                79..=81 => {
                    let mut g = [0u8; 16];
                    bcdec_rs::bc4(b, &mut g, 4, l.format == 81);
                    for (p, v) in g.iter().enumerate() {
                        tmp[p * 4..p * 4 + 4].copy_from_slice(&[*v, *v, *v, 255]);
                    }
                }
                82..=84 => {
                    let mut g = [0u8; 32];
                    bcdec_rs::bc5(b, &mut g, 8, l.format == 84);
                    for p in 0..16 {
                        tmp[p * 4..p * 4 + 4].copy_from_slice(&[g[2 * p], g[2 * p + 1], 0, 255]);
                    }
                }
                97..=99 => bcdec_rs::bc7(b, &mut tmp, 16),
                _ => return None,
            }
            for y in 0..4 {
                for x in 0..4 {
                    if bx + x < w && by + y < h {
                        let o = ((by + y) * w + bx + x) * 4;
                        out[o..o + 4].copy_from_slice(&tmp[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4]);
                    }
                }
            }
        }
        return Some((lw, lh, out));
    }
    for (i, px) in src.chunks_exact(n).enumerate().take(w * h) {
        let rgba = match l.format {
            27..=32 => [px[0], px[1], px[2], px[3]],
            87..=93 if n == 4 => [px[2], px[1], px[0], if l.format == 87 || l.format == 91 { px[3] } else { 255 }],
            48..=52 => [px[0], px[1], 0, 255],
            60..=64 => [px[0], px[0], px[0], 255],
            65 => [255, 255, 255, px[0]],
            _ => return None,
        };
        out[i * 4..i * 4 + 4].copy_from_slice(&rgba);
    }
    Some((lw, lh, out))
}
