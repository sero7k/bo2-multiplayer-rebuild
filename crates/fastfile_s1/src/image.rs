//! Image pixels: an image's mip chain from the zone itself or, for a
//! streamed image, from the `imagefile<N>.pak` files beside the zones.
//!
//! A streamed image names four parts (`XStreamFile`s in the zone header).
//! Part 0 holds the smallest streamed size with all its smaller levels;
//! each later part adds the next larger level (`GfxImageStreamData`:
//! width, height and the cumulative byte count). Each part is packed in its
//! pak as the zone data is: u64 size, u32 flags, then LZ4 blocks
//! (`container::inflate_blocks`; read on retail `imagefile1.pak`).

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::container::StreamFile;
use crate::walk::ImageRec;

/// One image's pixels, levels largest first, in its DXGI format.
#[derive(Clone, Debug)]
pub struct ImagePixels {
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub dxgi_format: u32,
    pub data: Vec<u8>,
}

/// Bytes per 4x4 block (block formats) or per pixel, and whether the
/// format is block compressed.
pub fn dxgi_unit(format: u32) -> Option<(usize, bool)> {
    Some(match format {
        // BC1, BC4: 8 bytes a block.
        70..=72 | 79..=81 => (8, true),
        // BC2, BC3, BC5, BC6H, BC7: 16.
        73..=78 | 82..=84 | 94..=99 => (16, true),
        // R8G8B8A8, B8G8R8A8 / X8, R10G10B10A2, R32: 4 bytes.
        27..=32 | 87..=93 | 23..=26 | 39..=43 => (4, false),
        // R16G16B16A16: 8.
        9..=14 => (8, false),
        // R8G8, R16: 2.
        48..=52 | 53..=59 => (2, false),
        // R8, A8: 1.
        60..=65 => (1, false),
        _ => return None,
    })
}

/// Bytes of one level of `w` x `h`.
pub fn level_bytes(format: u32, w: u32, h: u32) -> Option<usize> {
    let (unit, block) = dxgi_unit(format)?;
    let (w, h) = (w.max(1) as usize, h.max(1) as usize);
    Some(if block {
        w.div_ceil(4) * h.div_ceil(4) * unit
    } else {
        w * h * unit
    })
}

/// The `imagefile<N>.pak` files beside a zone, opened as needed.
pub struct ImagePaks {
    dir: PathBuf,
    open: HashMap<u16, Option<File>>,
}

impl ImagePaks {
    pub fn new(zone_dir: &Path) -> Self {
        Self {
            dir: zone_dir.to_path_buf(),
            open: HashMap::new(),
        }
    }

    /// One streamed part, inflated; `None` when its pak is missing or the
    /// part does not read.
    pub fn read(&mut self, part: &StreamFile, expect: usize) -> Option<Vec<u8>> {
        if part.offset == 0 || part.offset_end <= part.offset {
            return None;
        }
        let dir = &self.dir;
        let file = self
            .open
            .entry(part.file_index)
            .or_insert_with(|| {
                File::open(dir.join(format!("imagefile{}.pak", part.file_index))).ok()
            })
            .as_mut()?;
        file.seek(SeekFrom::Start(part.offset)).ok()?;
        let mut packed = vec![0u8; (part.offset_end - part.offset) as usize];
        file.read_exact(&mut packed).ok()?;
        let mut out = crate::container::inflate_blocks(&packed).ok()?;
        (out.len() >= expect).then(|| {
            out.truncate(expect);
            out
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

/// An image's 2-D pixels, its largest level no bigger than `max_size` on a
/// side (larger levels are left out; at least the smallest level kept).
pub fn image_pixels(
    rec: &ImageRec,
    zone_image: &[u8],
    paks: &mut ImagePaks,
    max_size: u32,
) -> Result<ImagePixels, String> {
    let format = rec.format;
    dxgi_unit(format).ok_or_else(|| format!("DXGI format {format} is not read"))?;
    if rec.streamed {
        return streamed(rec, paks, max_size);
    }
    let pixels = rec.pixels.ok_or("no pixels in the zone")?;
    let data = pixels.get(zone_image);
    let (mut w, mut h) = (u32::from(rec.width), u32::from(rec.height));
    let levels = u32::from(rec.level_count.max(1));
    let mut at = 0usize;
    let mut kept = levels;
    // Leave out levels above the size cap.
    while kept > 1 && w.max(h) > max_size {
        at += level_bytes(format, w, h).unwrap_or(0);
        w = (w / 2).max(1);
        h = (h / 2).max(1);
        kept -= 1;
    }
    let mut len = 0usize;
    let (mut lw, mut lh) = (w, h);
    for _ in 0..kept {
        len += level_bytes(format, lw, lh).unwrap_or(0);
        lw = (lw / 2).max(1);
        lh = (lh / 2).max(1);
    }
    let bytes = data
        .get(at..at + len)
        .ok_or_else(|| format!("{} bytes in the zone, {} wanted", data.len(), at + len))?;
    Ok(ImagePixels {
        width: w,
        height: h,
        levels: kept,
        dxgi_format: format,
        data: bytes.to_vec(),
    })
}

fn streamed(rec: &ImageRec, paks: &mut ImagePaks, max_size: u32) -> Result<ImagePixels, String> {
    let format = rec.format;
    // The largest part that fits the cap, with every smaller part readable.
    let parts: Vec<usize> = (0..4)
        .filter(|&i| rec.streams[i].0 > 0 && rec.stream_files[i].is_some())
        .collect();
    let Some(&top) = parts
        .iter()
        .rev()
        .find(|&&i| u32::from(rec.streams[i].0.max(rec.streams[i].1)) <= max_size)
        .or(parts.first())
    else {
        return Err("no streamed parts".into());
    };
    let mut chunks = Vec::new();
    for i in (0..=top).rev() {
        let (_, _, cum, _) = rec.streams[i];
        let before = if i == 0 { 0 } else { rec.streams[i - 1].2 };
        let want = (cum.saturating_sub(before)) as usize;
        let part = rec.stream_files[i].ok_or_else(|| format!("part {i} has no pak range"))?;
        let bytes = paks.read(&part, want).ok_or_else(|| {
            format!(
                "part {i} not read from {}",
                paks.dir()
                    .join(format!("imagefile{}.pak", part.file_index))
                    .display()
            )
        })?;
        chunks.push(bytes);
    }
    let (w, h, _, _) = rec.streams[top];
    // One level from each part above part 0; part 0 its whole chain.
    let levels = top as u32 + u32::from(rec.streams[0].3.max(1));
    Ok(ImagePixels {
        width: u32::from(w),
        height: u32::from(h),
        levels,
        dxgi_format: format,
        data: chunks.concat(),
    })
}
