//! Image pixels: a `GfxImage`'s levels from its inline data or its
//! `imagefile<N>.pak` streams (`pak.rs`), written as `.dds` (DX10 header,
//! the image's own DXGI format) or decoded to RGBA8.
//!
//! A streamed image has four stream levels (`ImageRec::streams`, smallest
//! first, unused ones zero): each names a top width x height, a level count
//! and the bytes of the whole mip chain from that size down (cumulative).
//! Stream 0's `.pak` range inflates to its whole chain; each later stream's
//! range holds only what it adds, its top level (bytes = its cumulative size
//! minus the previous stream's). The full chain is the highest stream's
//! range, then the next lower one's, ... then stream 0's. (Read on retail
//! `weapon_m8_garand_c_..._packed_cs`: 2048x2048 BC7, stream 3 inflates to
//! 0x400000 bytes, the chain is 0x555570.)

use crate::container::{ContainerError, StreamFile};
use crate::walk::ImageRec;

/// (bytes per 4x4 block, true) for block-compressed formats, (bytes per
/// pixel, false) for the others; None for formats not handled.
pub fn dxgi_layout(format: u32) -> Option<(usize, bool)> {
    Some(match format {
        70..=72 | 79..=81 => (8, true),             // BC1, BC4
        73..=78 | 82..=84 | 94..=99 => (16, true),  // BC2, BC3, BC5, BC6H, BC7
        1..=4 => (16, false),                        // R32G32B32A32
        9..=14 => (8, false),                        // R16G16B16A16
        15..=22 => (8, false),                       // R32G32, R32G8X24...
        23..=47 => (4, false),                       // R10G10B10A2 .. R32 / R24G8
        48..=59 => (2, false),                       // R8G8, R16
        60..=65 => (1, false),                       // R8, A8
        85..=93 => if (85..=86).contains(&format) { (2, false) } else { (4, false) }, // B5G6R5/B5G5R5A1, B8G8R8A8/X8
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

/// Bytes of a mip chain of `levels` levels from `w` x `h` down.
pub fn chain_size(format: u32, w: u32, h: u32, levels: u32) -> Option<usize> {
    (0..levels).map(|l| level_size(format, w >> l, h >> l)).sum()
}

/// The pixel data of one image at its best available size.
#[derive(Clone, Debug)]
pub struct ImageLevels {
    pub format: u32,
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    /// The mip chain, biggest level first (one element / face).
    pub data: Vec<u8>,
    /// Which stream it came from (None: inline in the zone).
    pub stream: Option<usize>,
}

/// The highest stream level whose `.pak` range is non-empty.
pub fn best_stream(im: &ImageRec) -> Option<(usize, StreamFile)> {
    (0..4).rev().find_map(|k| {
        let f = im.stream_files[k]?;
        (im.streams[k].0 != 0 && f.offset_end > f.offset).then_some((k, f))
    })
}

#[derive(Debug)]
pub enum TextureError {
    Container(ContainerError),
    NotStreamed,
    NoPixels,
    Size { want: usize, got: usize },
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Container(e) => e.fmt(f),
            Self::NotStreamed => write!(f, "image has no stream range"),
            Self::NoPixels => write!(f, "image has neither inline pixels nor streams"),
            Self::Size { want, got } => write!(f, "stream inflates to {got:#x} bytes, the image says {want:#x}"),
        }
    }
}

impl std::error::Error for TextureError {}

impl From<ContainerError> for TextureError {
    fn from(e: ContainerError) -> Self {
        Self::Container(e)
    }
}

/// The bytes stream `k`'s range of a streamed image inflates to.
pub fn stream_part_len(im: &ImageRec, k: usize) -> usize {
    let prev = if k == 0 { 0 } else { im.streams[k - 1].2 };
    im.streams[k].2.saturating_sub(prev) as usize
}

/// Inflates stream `k`'s raw `.pak` range and checks its size.
pub fn stream_part(im: &ImageRec, k: usize, raw: &[u8]) -> Result<Vec<u8>, TextureError> {
    let data = crate::pak::inflate_image_range(raw)?;
    let want = stream_part_len(im, k);
    if data.len() != want {
        return Err(TextureError::Size { want, got: data.len() });
    }
    Ok(data)
}

/// A streamed image from stream `k` down: `parts[i]` is stream `i`'s
/// inflated range (`stream_part`) for every `i <= k`; with only `parts[k]`
/// given (the others empty), just the top level.
pub fn stream_levels(im: &ImageRec, k: usize, parts: &[Vec<u8>]) -> ImageLevels {
    let (w, h, _, levels) = im.streams[k];
    let full = (0..k).all(|i| parts.get(i).is_some_and(|p| !p.is_empty() || stream_part_len(im, i) == 0));
    let mut data = Vec::new();
    if full {
        for i in (0..=k).rev() {
            data.extend_from_slice(&parts[i]);
        }
    } else {
        data.extend_from_slice(&parts[k]);
    }
    ImageLevels {
        format: im.format,
        width: w as u32,
        height: h as u32,
        levels: if full { levels as u32 } else { 1 },
        data,
        stream: Some(k),
    }
}

/// An image with inline pixels (all its levels in the zone image).
pub fn inline_levels(im: &ImageRec, zone_image: &[u8]) -> Result<ImageLevels, TextureError> {
    let span = im.pixels.ok_or(TextureError::NoPixels)?;
    Ok(ImageLevels {
        format: im.format,
        width: im.width as u32,
        height: im.height as u32,
        levels: im.level_count.max(1) as u32,
        data: span.get(zone_image).to_vec(),
        stream: None,
    })
}

/// A `.dds` file (DX10 extended header) of the levels.
pub fn to_dds(l: &ImageLevels, array_size: u32, cube: bool) -> Vec<u8> {
    let mut o = Vec::with_capacity(148 + l.data.len());
    let u = |o: &mut Vec<u8>, v: u32| o.extend_from_slice(&v.to_le_bytes());
    o.extend_from_slice(b"DDS ");
    u(&mut o, 124);
    // CAPS | HEIGHT | WIDTH | PIXELFORMAT | MIPMAPCOUNT | LINEARSIZE
    u(&mut o, 0x1 | 0x2 | 0x4 | 0x1000 | 0x20000 | 0x80000);
    u(&mut o, l.height);
    u(&mut o, l.width);
    u(&mut o, level_size(l.format, l.width, l.height).unwrap_or(0) as u32);
    u(&mut o, 0); // depth
    u(&mut o, l.levels);
    for _ in 0..11 {
        u(&mut o, 0);
    }
    // DDS_PIXELFORMAT: FOURCC "DX10"
    u(&mut o, 32);
    u(&mut o, 0x4);
    o.extend_from_slice(b"DX10");
    for _ in 0..5 {
        u(&mut o, 0);
    }
    // caps: TEXTURE | MIPMAP | COMPLEX; caps2: cubemap (all faces)
    u(&mut o, 0x1000 | 0x400000 | 0x8);
    u(&mut o, if cube { 0xFE00 } else { 0 });
    for _ in 0..3 {
        u(&mut o, 0);
    }
    // DDS_HEADER_DXT10
    u(&mut o, l.format);
    u(&mut o, 3); // TEXTURE2D
    u(&mut o, if cube { 0x4 } else { 0 });
    u(&mut o, array_size.max(1));
    u(&mut o, 0);
    o.extend_from_slice(&l.data);
    o
}

/// The top level decoded to RGBA8 (block formats via `bcdec_rs`; BC4 and
/// BC5 as grey / red-green; BC6H not handled).
pub fn top_level_rgba(l: &ImageLevels) -> Option<Vec<u8>> {
    level_rgba(l, 0).map(|(_, _, px)| px)
}

/// Level `mip` decoded to RGBA8: (width, height, pixels).
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

/// The engine's name hash (texture slot and constant names in materials):
/// lower-cased bytes, `h = c ^ 33 * h` (checked: "colorMap" = 0xa0ab1041,
/// "normalMap" = 0x59d30d0f, "colorTint" = 0xb60c3b3a, the hashes the M8's
/// materials store).
pub fn name_hash(s: &str) -> u32 {
    s.bytes().fold(0u32, |h, c| (c | 0x20) as u32 ^ h.wrapping_mul(33))
}

const SAMPLER_BASES: [&str; 7] = ["colorMap", "normalMap", "specularMap", "emissiveMap", "detailMap", "detailNormalMap", "glossMap"];

/// A texture slot's name from its hash, for the names known so far.
pub fn sampler_name(hash: u32) -> Option<&'static str> {
    const SUFFIX: [&str; 4] = ["", "0", "1", "2"];
    static NAMES: std::sync::OnceLock<Vec<(u32, &'static str)>> = std::sync::OnceLock::new();
    NAMES
        .get_or_init(|| {
            let mut v = Vec::new();
            for b in SAMPLER_BASES {
                for s in SUFFIX {
                    let n: &'static str = Box::leak(format!("{b}{s}").into_boxed_str());
                    v.push((name_hash(n), n));
                }
            }
            v
        })
        .iter()
        .find(|(h, _)| *h == hash)
        .map(|(_, n)| *n)
}

/// A material constant's full name from its hash, for the names known so
/// far (the stored name is cut to 12 bytes).
pub fn constant_name(hash: u32) -> Option<&'static str> {
    ["colorTint", "colorTint1", "colorTint2"].into_iter().find(|n| name_hash(n) == hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_hash_matches_the_retail_slots() {
        assert_eq!(name_hash("colorMap"), 0xa0ab_1041);
        assert_eq!(name_hash("normalMap"), 0x59d3_0d0f);
        assert_eq!(name_hash("colorTint"), 0xb60c_3b3a);
        assert_eq!(sampler_name(0x34ec_ccb3), Some("specularMap"));
        assert_eq!(sampler_name(0xb60d_1850), Some("colorMap1"));
    }

    #[test]
    fn chain_sizes() {
        // 2048x2048 BC7, 12 levels: the M8 colour map's chain.
        assert_eq!(chain_size(98, 2048, 2048, 12), Some(0x55_5570));
        // 256x256 BC4, 9 levels.
        assert_eq!(chain_size(80, 256, 256, 9), Some(43704));
    }

    #[test]
    fn dds_header() {
        let l = ImageLevels { format: 98, width: 8, height: 8, levels: 1, data: vec![0; 64], stream: None };
        let d = to_dds(&l, 1, false);
        assert_eq!(&d[..4], b"DDS ");
        assert_eq!(&d[84..88], b"DX10");
        assert_eq!(u32::from_le_bytes(d[128..132].try_into().unwrap()), 98);
        assert_eq!(d.len(), 148 + 64);
    }
}
