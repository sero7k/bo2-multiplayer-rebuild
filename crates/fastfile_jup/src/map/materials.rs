//! A map's materials and their pictures: which material each world surface
//! draws with, which pictures a material has, and the pictures' pixels
//! from the install's packages.
//!
//! - The `gfx_map` keeps a table of material pointers (each a back-pointer
//!   to an entry of the zone's asset list, all of them materials) and a
//!   40-byte record per surface:
//!   ```text
//!   +0x00 u32 its surface record (the 128-byte list, see [`super::world`])
//!   +0x04 u32 its material, in that table
//!   +0x0e u16 the surface's world zone (as the surface record has it)
//!   +0x14 u16 0xffff
//!   ```
//! - A material (asset type 11) is a 0x78-byte header:
//!   ```text
//!   +0x00 u64 name hash
//!   +0x1b u8  pictures, +0x1c u8 constant rows (16 bytes each)
//!   +0x20, +0x28, +0x38 pointers (techniques, states: shared data, or
//!         data that follows)
//!   +0x30 pointer to its pictures: `pictures` pointers to entries of the
//!         asset list (images), following when -2/-1
//!   +0x40 pointer to its constants, following when -2/-1
//!   ```
//!   then what its pointers lead to. The headers are not walked: they are
//!   found by their shape (a hash, then counts and pointers of the forms
//!   above; most also hold 0x2c73b6c0 at +0x08), and the run of them that
//!   is exactly the zone's material count is the materials in asset order.
//!   A material whose run cannot be found that way gets no pictures.
//! - The pictures are the zone's image run ([`crate::image::ImageRun`]);
//!   a picture's pixels are the largest of its streamed mip groups the
//!   install has (`zone/*.xpak` over their `.xsub` parts), or its pixels in
//!   the zone.

use std::path::Path;

use super::index::{ZoneImage, u16_at, u32_at, u64_at};
use crate::image::{self, ImageRec, MipGroup};
use crate::oodle::Decompressor;
use crate::ptr::{FOLLOW, INSERT, MINUS3};
use crate::xsub::XPak;

/// A picture of the map: what it takes to fetch its pixels.
#[derive(Clone, Debug, Default)]
pub struct MapImage {
    pub hash: u64,
    pub width: u16,
    pub height: u16,
    /// The image's format byte ([`crate::image::dxgi`]).
    pub format: u8,
    pub groups: Vec<MipGroup>,
    /// Its pixels in the zone (a small picture), whole chain.
    pub inline: Option<Vec<u8>>,
}

/// A material: its hash and its pictures (indices into the map's images).
#[derive(Clone, Debug, Default)]
pub struct MapMaterial {
    pub hash: u64,
    pub images: Vec<usize>,
    /// The picture it is coloured by: its first of format 0x25 (BC1, the
    /// form colour maps take; 0x2a, also BC1, holds masks and terrain
    /// blend maps).
    pub colour: Option<usize>,
}

/// What was found: per surface record its material (index into
/// `materials`), the materials and the images they use.
#[derive(Default)]
pub struct Found {
    pub surface_material: Vec<Option<usize>>,
    pub materials: Vec<MapMaterial>,
    pub images: Vec<MapImage>,
}

const MATERIAL_HEADER: usize = 0x78;
const SURFACE_INFO: usize = 40;

fn is_back(v: u64) -> bool {
    v & 1 == 1 && v >> 32 == 0x0f
}

fn pointer_like(v: u64) -> bool {
    matches!(v, 0 | FOLLOW | INSERT | MINUS3) || is_back(v) || v >> 63 == 1
}

/// The longest run of back-pointers that all lead to material entries of
/// the asset list; (where, count, the list base that makes them so).
fn material_table(zone: &ZoneImage) -> Option<(usize, usize, usize)> {
    let z = &zone.bytes;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut at = 0usize;
    while at + 8 <= z.len() {
        if z[at + 4..at + 8] != [0x0f, 0, 0, 0] || z[at] & 1 == 0 {
            at += 1;
            continue;
        }
        let mut n = 0;
        while at + 8 * (n + 1) <= z.len() && is_back(u64_at(z, at + 8 * n)) {
            n += 1;
        }
        if n >= 8 {
            runs.push((at, n));
        }
        at += 8 * n.max(1);
    }
    runs.sort_by_key(|r| std::cmp::Reverse(r.1));
    let lo = zone.list_at.saturating_sub(0x40);
    // The entry i's pointer is base + 16 i (+1): the base that makes every
    // pointer of the run a material's.
    runs.into_iter().find_map(|(at, n)| {
        let offs: Vec<usize> = (0..n)
            .map(|k| (u64_at(z, at + 8 * k) & 0xffff_ffff) as usize - 1)
            .collect();
        (lo..zone.list_at + 0x40)
            .find(|&base| {
                offs.iter().all(|&o| {
                    o >= base
                        && (o - base) % 16 == 0
                        && zone.types.get((o - base) / 16) == Some(&11)
                })
            })
            .map(|base| (at, n, base))
    })
}

/// The 40-byte record per surface record: where the run starts.
fn surface_infos(z: &[u8], zones: &[u32], materials: usize) -> Option<usize> {
    let n = zones.len();
    let fits = |at: usize, k: usize| {
        let r = at + SURFACE_INFO * k;
        let s = u32_at(z, r) as usize;
        s < n
            && (u32_at(z, r + 4) as usize) < materials
            && u16_at(z, r + 0x14) == 0xffff
            && u32::from(u16_at(z, r + 0x0e)) == zones[s]
    };
    let last = z.len().checked_sub(SURFACE_INFO * n)?;
    (0..=last).find(|&at| (0..n.min(8)).all(|k| fits(at, k)) && (0..n).all(|k| fits(at, k)))
}

/// Does a material header stand at `h`? (Its hash, the counts' zero
/// neighbours, its pointers' forms; `magic`: its +0x08 word is the one
/// most materials have.)
fn material_at(z: &[u8], h: usize) -> Option<bool> {
    if h + MATERIAL_HEADER > z.len() || z[h + 0x17] != 0 || z[h + 0x1f] != 0 {
        return None;
    }
    let hash = u64_at(z, h);
    if hash == 0 || hash >> 63 == 1 || hash.count_ones() < 16 {
        return None;
    }
    if ![0x20, 0x28, 0x30, 0x38, 0x40]
        .iter()
        .all(|&o| pointer_like(u64_at(z, h + o)))
    {
        return None;
    }
    let magic = u32_at(z, h + 8) == 0x2c73_b6c0;
    let counts = matches!(z[h + 0x1d], 1 | 2 | 4) && matches!(z[h + 0x1e], 1 | 2 | 4);
    (magic || counts).then_some(magic)
}

/// The material headers, in asset order: the tightest run of `count`
/// header-shaped places (overlapping ones resolved toward the usual
/// +0x08 word).
fn material_headers(z: &[u8], count: usize) -> Option<Vec<usize>> {
    let mut found: Vec<(usize, bool)> = Vec::new();
    for h in 0..z.len().saturating_sub(MATERIAL_HEADER) {
        if let Some(magic) = material_at(z, h) {
            match found.last_mut() {
                Some(last) if h - last.0 < MATERIAL_HEADER => {
                    if magic && !last.1 {
                        *last = (h, magic);
                    }
                }
                _ => found.push((h, magic)),
            }
        }
    }
    if found.len() < count || count == 0 {
        return None;
    }
    let span = |i: usize| found[i + count - 1].0 - found[i].0;
    let i = (0..=found.len() - count).min_by_key(|&i| span(i))?;
    Some(found[i..i + count].iter().map(|f| f.0).collect())
}

/// Reads which material every surface record draws with and the pictures
/// of those materials. `zones`: each surface record's world zone.
pub fn read(zone: &ZoneImage, zones: &[u32], report: &mut Vec<String>) -> Found {
    let mut out = Found {
        surface_material: vec![None; zones.len()],
        ..Default::default()
    };
    let z = &zone.bytes;
    let Some((table, table_len, base)) = material_table(zone) else {
        report.push("materials: no material table".into());
        return out;
    };
    let Some(infos) = surface_infos(z, zones, table_len) else {
        report.push(format!(
            "materials: {table_len} in the table, no per-surface records"
        ));
        return out;
    };
    let asset_of = |v: u64| -> Option<usize> {
        let o = ((v & 0xffff_ffff) as usize).checked_sub(1)?;
        let rel = o.checked_sub(base)?;
        (rel % 16 == 0 && rel / 16 < zone.types.len()).then_some(rel / 16)
    };
    // The k-th material asset and image asset, by asset index.
    let mut nth = vec![usize::MAX; zone.types.len()];
    let (mut mats, mut imgs) = (0usize, 0usize);
    for (i, &t) in zone.types.iter().enumerate() {
        if t == 11 {
            nth[i] = mats;
            mats += 1;
        } else if t == 21 {
            nth[i] = imgs;
            imgs += 1;
        }
    }
    let headers = material_headers(z, mats);
    let run = image::ImageRun::find(z, imgs);
    report.push(format!(
        "materials: {table_len} drawn of {mats} ({}), {imgs} images ({})",
        if headers.is_some() {
            "headers found"
        } else {
            "headers not found"
        },
        if run.is_some() {
            "run found"
        } else {
            "run not found"
        }
    ));
    // Table entry -> material ordinal -> its pictures.
    let mut by_ordinal: std::collections::HashMap<usize, usize> = Default::default();
    let mut image_of: std::collections::HashMap<usize, usize> = Default::default();
    let mut material_of_entry = vec![None; table_len];
    for (e, slot) in material_of_entry.iter_mut().enumerate() {
        let Some(ordinal) = asset_of(u64_at(z, table + 8 * e)).map(|a| nth[a]) else {
            continue;
        };
        if let Some(&m) = by_ordinal.get(&ordinal) {
            *slot = Some(m);
            continue;
        }
        let mut mat = MapMaterial::default();
        if let Some(h) = headers.as_ref().and_then(|hs| hs.get(ordinal).copied()) {
            mat.hash = u64_at(z, h);
            let pics = usize::from(z[h + 0x1b]);
            if matches!(u64_at(z, h + 0x30), FOLLOW | INSERT) {
                for k in 0..pics {
                    let Some(a) = asset_of(u64_at(z, h + MATERIAL_HEADER + 8 * k)) else {
                        continue;
                    };
                    if zone.types[a] != 21 {
                        continue;
                    }
                    let Some(at) = run.as_ref().and_then(|r| r.at.get(nth[a]).copied()) else {
                        continue;
                    };
                    let idx = *image_of.entry(at).or_insert_with(|| {
                        out.images.push(map_image(z, at));
                        out.images.len() - 1
                    });
                    mat.images.push(idx);
                }
            }
        }
        let pick = |f: u8| {
            mat.images
                .iter()
                .copied()
                .find(|&i| out.images[i].format == f)
        };
        mat.colour = pick(0x25);
        out.materials.push(mat);
        by_ordinal.insert(ordinal, out.materials.len() - 1);
        *slot = Some(out.materials.len() - 1);
    }
    for k in 0..zones.len() {
        let r = infos + SURFACE_INFO * k;
        let s = u32_at(z, r) as usize;
        let e = u32_at(z, r + 4) as usize;
        if let Some(m) = material_of_entry.get(e).copied().flatten()
            && let Some(slot) = out.surface_material.get_mut(s)
        {
            *slot = Some(m);
        }
    }
    let coloured = out.materials.iter().filter(|m| m.colour.is_some()).count();
    let mut formats: std::collections::BTreeMap<u8, usize> = Default::default();
    for i in &out.images {
        *formats.entry(i.format).or_default() += 1;
    }
    report.push(format!(
        "materials: {} used, {coloured} with a colour picture, {} pictures (formats {formats:x?})",
        out.materials.len(),
        out.images.len()
    ));
    out
}

fn map_image(z: &[u8], at: usize) -> MapImage {
    match image::parse(z, at) {
        Ok(Some(ImageRec {
            hash,
            width,
            height,
            format,
            groups,
            inline,
            ..
        })) => MapImage {
            hash,
            width,
            height,
            format,
            groups,
            inline: inline.map(|(o, n)| z[o..o + n].to_vec()),
        },
        _ => MapImage::default(),
    }
}

/// The install's streamed packages, to fetch pictures' pixels from.
pub struct Packages {
    paks: Vec<XPak>,
}

/// One level of a picture: DXGI format (UNORM), size, the blocks.
pub struct Level {
    pub dxgi: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Packages {
    /// Every zone `.xpak` that has local parts.
    pub fn open(install: &Path) -> Self {
        let mut paks = Vec::new();
        if let Ok(dir) = std::fs::read_dir(install.join("zone")) {
            let mut files: Vec<_> = dir
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("xpak"))
                })
                .collect();
            files.sort();
            for f in files {
                if let Ok(p) = XPak::open(&f)
                    && p.stride > 0
                {
                    paks.push(p);
                }
            }
        }
        Self { paks }
    }

    pub fn len(&self) -> usize {
        self.paks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paks.is_empty()
    }

    /// Whether a package of the install has the entry.
    pub fn has(&self, key: u64) -> bool {
        self.paks.iter().any(|p| p.find(key).is_some())
    }

    fn fetch(&self, key: u64, oodle: &dyn Decompressor) -> Option<Vec<u8>> {
        self.paks
            .iter()
            .find_map(|p| p.find(key).map(|e| (p, *e)))
            .and_then(|(p, e)| p.read(&e, oodle).ok())
    }

    /// The picture's largest level at most `max_dim` wide and high that the
    /// install has.
    pub fn level(
        &self,
        img: &MapImage,
        max_dim: u32,
        oodle: &dyn Decompressor,
    ) -> Result<Level, String> {
        let dxgi = image::dxgi(img.format).ok_or_else(|| format!("format {:#x}", img.format))?;
        let rec = ImageRec {
            at: 0,
            hash: img.hash,
            flags: 0,
            data_size: 0,
            width: img.width,
            height: img.height,
            depth: 1,
            array: 1,
            format: img.format,
            usage: 0,
            levels: 0,
            groups: img.groups.clone(),
            inline: None,
            end: 0,
        };
        for g in (0..img.groups.len()).rev() {
            let Some(levels) = image::group_levels(&rec, g, dxgi) else {
                continue;
            };
            let Some(&(_, w, h, off)) = levels.iter().find(|l| l.1.max(l.2) <= max_dim) else {
                continue;
            };
            let Some(bytes) = self.fetch(img.groups[g].key, oodle) else {
                continue;
            };
            let n = image::level_bytes(dxgi, w, h).unwrap_or(0);
            if let Some(data) = bytes.get(off..off + n) {
                return Ok(Level {
                    dxgi,
                    width: w,
                    height: h,
                    data: data.to_vec(),
                });
            }
        }
        if let Some(px) = &img.inline {
            // The whole chain in the zone: its largest level that fits.
            let (mut w, mut h) = (u32::from(img.width), u32::from(img.height));
            let mut off = 0usize;
            while w.max(h) > max_dim && w > 4 && h > 4 {
                off += image::level_bytes(dxgi, w, h).unwrap_or(0);
                (w, h) = (w / 2, h / 2);
            }
            let n = image::level_bytes(dxgi, w, h).unwrap_or(0);
            if let Some(data) = px.get(off..off + n) {
                return Ok(Level {
                    dxgi,
                    width: w,
                    height: h,
                    data: data.to_vec(),
                });
            }
        }
        Err(format!("{:016x}: no group of it in the install", img.hash))
    }
}
