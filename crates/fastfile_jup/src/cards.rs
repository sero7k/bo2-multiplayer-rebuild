//! Emblems and calling cards ("player cards"): every one the game's
//! `global` zone defines, with its picture.
//!
//! Each is a script bundle (type 115) whose string cells hold its name in
//! the clear (`ui_emblem_005`, `ui_playercard_100`,
//! `jup_emblem_animated_opc_..._atlas`) and its title's localize key (the
//! title itself is encrypted and not read). Its picture is an image asset
//! with the same name (the image comes just before its bundle in the
//! zone), its pixels in one streamed group: every level, largest first,
//! in a `.xsub` entry under the group's key.
//!
//! Animated ones are a flipbook: the frames side by side in rows. An
//! emblem's is 4 by 4 square frames (1024x1024, 4096x4096) or 3 by 4
//! (768x1024, 1536x2048); a calling card's frames are 4:1 like a still
//! card, 2 by 8 in a 1024x1024 picture, 3 by 4 in a 1536x512 one. Neither
//! the bundle nor the name tells a flipbook from a still picture of the
//! same shape (`ui_emblem_143` and `ui_emblem_rankedplay_02` have the same
//! cells; some `_atlas` names are 256x256 stills), so the shape decides
//! where it can ([`Item::grid_by_size`]: a card that is not itself 4:1, an
//! emblem of 3:4) and for a square emblem [`detect_grid`] looks at the
//! pixels: a flipbook's neighbouring frames are nearly the same picture, or
//! its frames stand apart with clear gutters between them, where a still
//! picture's tiles are parts of one picture. Every picture of build 1.21
//! has one level, in one group.
//!
//! The names are found as they are stored in a bundle's string cells:
//! `-2`, a 32-bit hash, 4 zero bytes, the name and its NUL.
#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::image::{self, ImageRec};
use crate::oodle::Decompressor;
use crate::ptr::FOLLOW;
use crate::xsub::{XPak, XSubEntry};
use crate::zone::HashIndex;

/// The zone that defines them (`zone/global.ff`).
pub const ZONE: &str = "global";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Emblem,
    Card,
}

/// What a name is, by its words; `None` for the other pictures that share
/// them (group emblems, weapon decals, prestige models, mobile icons).
#[must_use]
pub fn kind_of(name: &str) -> Option<Kind> {
    if !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
        return None;
    }
    let ours = name.starts_with("ui_") || name.starts_with("jup_") || name.starts_with("wzm_");
    if !ours || name.starts_with("ui_group_") || name.starts_with("jup_wc_") {
        return None;
    }
    if name.contains("playercard") {
        Some(Kind::Card)
    } else if name.contains("emblem") && !name.contains("prestigeemblem") {
        Some(Kind::Emblem)
    } else {
        None
    }
}

/// Whether a name is a flipbook's (`_atlas`, also misspelt `_altas`).
#[must_use]
pub fn is_animated(name: &str) -> bool {
    name.contains("_atlas") || name.contains("_altas")
}

/// Every emblem and calling card name stored as a bundle's string cell, in
/// zone order, each once.
#[must_use]
pub fn scan_names(zone: &[u8]) -> Vec<(String, Kind)> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let follow = FOLLOW.to_le_bytes();
    let mut at = 0usize;
    while at + 17 <= zone.len() {
        // (Every cell string starts with its pointer, -2.)
        let Some(i) = zone[at..].windows(8).position(|w| w == follow) else { break };
        let p = at + i;
        at = p + 1;
        let s = p + 16;
        if s >= zone.len() || zone[p + 12..s] != [0, 0, 0, 0] {
            continue;
        }
        let Some(len) = zone[s..zone.len().min(s + 160)].iter().position(|&b| b == 0) else { continue };
        if len < 6 {
            continue;
        }
        let Ok(name) = std::str::from_utf8(&zone[s..s + len]) else { continue };
        if let Some(kind) = kind_of(name)
            && seen.insert(name.to_owned())
        {
            out.push((name.to_owned(), kind));
        }
        at = s + len;
    }
    out
}

/// One emblem or calling card and its picture.
#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub kind: Kind,
    pub image: ImageRec,
}

impl Item {
    /// Whether the name says flipbook (`_atlas`).
    #[must_use]
    pub fn named_animated(&self) -> bool {
        is_animated(&self.name)
    }

    /// The (columns, rows) its picture would have as a flipbook, `None`
    /// when its shape is not one: a square emblem of 512 or more is 4 by 4,
    /// a 3:4 one 3 by 4; a calling card that is not itself 4:1 is the grid
    /// of 16, 12 or 8 frames of 4:1 that tiles it (2 by 8 for 1024x1024, 3
    /// by 4 for 1536x512, 2 by 6 for 1024x768).
    #[must_use]
    pub fn candidate_grid(&self) -> Option<(u32, u32)> {
        let (w, h) = (u32::from(self.image.width), u32::from(self.image.height));
        match self.kind {
            Kind::Emblem if w == h && w >= 512 && w % 4 == 0 => Some((4, 4)),
            Kind::Emblem if w * 4 == h * 3 && w % 3 == 0 && h % 4 == 0 => Some((3, 4)),
            Kind::Emblem => None,
            Kind::Card if w == 4 * h => None,
            Kind::Card => [16u32, 12, 8].into_iter().find_map(|n| {
                (1..=n).filter(|c| n % c == 0).find_map(|c| {
                    let r = n / c;
                    (w % c == 0 && h % r == 0 && w / c == 4 * (h / r)).then_some((c, r))
                })
            }),
        }
    }

    /// What its shape alone says: `Some(grid)` (1 by 1 for a still), or
    /// `None` when only the pixels can tell (a square emblem of several
    /// frames).
    #[must_use]
    pub fn grid_by_size(&self) -> Option<(u32, u32)> {
        let Some(grid) = self.candidate_grid() else { return Some((1, 1)) };
        match self.kind {
            Kind::Emblem if self.image.width == self.image.height => None,
            _ => Some(grid),
        }
    }
}

/// Every emblem and calling card the zone defines with a picture, in zone
/// order (a format [`image::dxgi`] does not know is kept; it does not
/// decode).
#[must_use]
pub fn find_items(zone: &[u8]) -> Vec<Item> {
    let names = scan_names(zone);
    let hashes: Vec<u64> = names.iter().map(|(n, _)| crate::hash::name(n)).collect();
    let index = HashIndex::build(zone, hashes.iter().copied(), 4);
    let mut out = Vec::new();
    for ((name, kind), hash) in names.into_iter().zip(hashes) {
        let found = index.plain(hash).iter().find_map(|&at| match image::parse(zone, at) {
            Ok(Some(rec)) if rec.hash == hash && usable(&rec) => Some(rec),
            _ => None,
        });
        if let Some(image) = found {
            out.push(Item { name, kind, image });
        }
    }
    out
}

fn usable(rec: &ImageRec) -> bool {
    rec.depth == 1 && rec.array == 1 && (rec.inline.is_some() || !rec.groups.is_empty())
}

/// The zone `.xpak` packages, opened as asked for.
pub struct Packages {
    dir: PathBuf,
    open: Vec<XPak>,
    tried: HashSet<PathBuf>,
    /// Whether a key not in the likely packages is looked for in every
    /// other one (their indexes are big: off for the game, on for probes).
    search_all: bool,
}

/// The packages the pictures are in on build 1.21, asked in this order
/// (`boot_shared` the older ones, `codhq23b` and `codhq23a` the seasons',
/// `codhq_shared` 87 more); 2 of 5,272 are in no installed package.
pub const LIKELY_PACKAGES: &[&str] = &["boot_shared", "codhq23b", "codhq23a", "boot", "codhq_shared"];

impl Packages {
    /// Over an install's `zone` folder: the likely packages only, or with
    /// `search_all` every `.xpak` there.
    #[must_use]
    pub fn new(zone_dir: &Path, search_all: bool) -> Self {
        Self { dir: zone_dir.to_path_buf(), open: Vec::new(), tried: HashSet::new(), search_all }
    }

    fn open_one(&mut self, path: PathBuf) -> bool {
        if !self.tried.insert(path.clone()) {
            return false;
        }
        match XPak::open(&path) {
            Ok(p) if !p.parts.is_empty() => {
                self.open.push(p);
                true
            }
            _ => false,
        }
    }

    fn find_open(&self, key: u64) -> Option<(usize, XSubEntry)> {
        self.open.iter().enumerate().find_map(|(i, p)| p.find(key).map(|e| (i, *e)))
    }

    /// The package row of `key`: the open packages, then the likely ones,
    /// then (`search_all`) every other `.xpak` of the folder.
    pub fn find(&mut self, key: u64) -> Option<(usize, XSubEntry)> {
        if let Some(hit) = self.find_open(key) {
            return Some(hit);
        }
        for name in LIKELY_PACKAGES {
            if self.open_one(self.dir.join(format!("{name}.xpak")))
                && let Some(hit) = self.find_open(key)
            {
                return Some(hit);
            }
        }
        if !self.search_all {
            return None;
        }
        let mut all: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .ok()?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "xpak"))
            .collect();
        all.sort();
        for p in all {
            if self.open_one(p)
                && let Some(hit) = self.find_open(key)
            {
                return Some(hit);
            }
        }
        None
    }

    /// The decoded bytes of `key`.
    pub fn read(&mut self, key: u64, oodle: &dyn Decompressor) -> Option<Vec<u8>> {
        let (i, e) = self.find(key)?;
        self.open[i].read(&e, oodle).ok()
    }

    /// Where `key`'s bytes are: its part file and the row as the part has
    /// it (read with [`read_located`], no lock needed).
    pub fn locate(&mut self, key: u64) -> Option<(PathBuf, XSubEntry)> {
        let (i, e) = self.find(key)?;
        let (part, local) = self.open[i].locate(&e)?;
        Some((self.open[i].parts[part].clone(), local))
    }

    /// The packages opened so far (their file names).
    #[must_use]
    pub fn opened(&self) -> Vec<String> {
        self.open
            .iter()
            .filter_map(|p| p.parts.first().and_then(|f| f.file_name()).map(|f| f.to_string_lossy().into_owned()))
            .collect()
    }
}

/// A located entry's decoded bytes.
pub fn read_located(part: &Path, entry: &XSubEntry, oodle: &dyn Decompressor) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(part).ok()?;
    let mut blob = vec![0u8; entry.size as usize];
    f.seek(SeekFrom::Start(entry.offset)).ok()?;
    f.read_exact(&mut blob).ok()?;
    crate::xsub::decode_entry(entry, &blob, oodle).ok()
}

/// An RGBA picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// An item's picture: the first level of the largest group present whose
/// would-be frames (or whole picture) are no larger than `max_frame_side`,
/// else the smallest there is. `fetch` gives a package entry's decoded
/// bytes by key; `zone` is needed only for a picture stored in the zone.
pub fn picture(zone: Option<&[u8]>, item: &Item, fetch: &mut dyn FnMut(u64) -> Option<Vec<u8>>, max_frame_side: u32) -> Option<Rgba> {
    let img = &item.image;
    let dxgi = image::dxgi(img.format)?;
    let (data, group) = match img.inline {
        Some((at, len)) => (zone?.get(at..at + len)?.to_vec(), None),
        None => (0..img.groups.len()).rev().find_map(|g| fetch(img.groups[g].key).map(|b| (b, Some(g))))?,
    };
    let levels: Vec<(u32, u32, u32, usize)> = match group {
        Some(g) => image::group_levels(img, g, dxgi)?,
        None => {
            // A picture in the zone holds its levels from the top.
            let mut v = Vec::new();
            let (mut off, mut l) = (0usize, 0u32);
            loop {
                let (w, h) = ((u32::from(img.width) >> l).max(1), (u32::from(img.height) >> l).max(1));
                let n = image::level_bytes(dxgi, w, h)?;
                if off + n > data.len() {
                    break;
                }
                v.push((l, w, h, off));
                off += n;
                l += 1;
                if w <= 4 || h <= 4 || (img.levels != 0 && l >= u32::from(img.levels)) {
                    break;
                }
            }
            v
        }
    };
    let (cols, rows) = item.candidate_grid().unwrap_or((1, 1));
    let pick = levels.iter().find(|(_, w, h, _)| (w / cols).max(h / rows) <= max_frame_side).or(levels.last())?;
    let (_, w, h, off) = *pick;
    let n = image::level_bytes(dxgi, w, h)?;
    let pixels = decode_bcn(dxgi, w, h, data.get(off..off + n)?)?;
    Some(Rgba { width: w, height: h, pixels })
}

/// How alike neighbouring frames must be, against how much a frame varies
/// in itself, for the picture to be a flipbook.
pub const FLIPBOOK_LIKENESS: f32 = 0.5;

/// How clear a flipbook's gutters are at most: the alpha along the
/// frames' inner borders against their middles' (a still picture's tiles
/// run on across them).
pub const FLIPBOOK_GUTTER: f32 = 0.3;

/// Its picture's flipbook layout, (1, 1) for a still picture: as its shape
/// says ([`Item::grid_by_size`]), else (a square emblem of several frames)
/// its candidate grid when neighbouring frames are nearly the same picture
/// against how much a frame varies in itself, or when the frames stand
/// apart with clear gutters between them.
#[must_use]
pub fn detect_grid(item: &Item, pic: &Rgba) -> (u32, u32) {
    if let Some(grid) = item.grid_by_size() {
        return grid;
    }
    let Some(grid) = item.candidate_grid() else { return (1, 1) };
    let (near, spread) = likeness(pic, grid);
    if spread >= 2.0 && near < FLIPBOOK_LIKENESS * spread {
        return grid;
    }
    if gutters(pic, grid) < FLIPBOOK_GUTTER { grid } else { (1, 1) }
}

/// (The larger mean difference of frames 0-1 and 1-2, frame 0's spread.)
#[must_use]
pub fn likeness(pic: &Rgba, grid: (u32, u32)) -> (f32, f32) {
    let n = grid.0 * grid.1;
    let f: Vec<Rgba> = [0, 1.min(n - 1), 2.min(n - 1)].iter().map(|&i| frame(pic, grid, i)).collect();
    let near = mean_diff(&f[0], &f[1]).max(mean_diff(&f[1], &f[2]));
    (near, spread(&f[0]))
}

/// The mean alpha on the frames' inner borders (the pixel each side of
/// every line between two frames) against the mean alpha of the frames'
/// middles (their middle half each way); 0 for a blank picture.
#[must_use]
pub fn gutters(pic: &Rgba, grid: (u32, u32)) -> f32 {
    let (cols, rows) = (grid.0.max(1), grid.1.max(1));
    let (fw, fh) = (pic.width / cols, pic.height / rows);
    if fw < 4 || fh < 4 {
        return 0.0;
    }
    let alpha = |x: u32, y: u32| f64::from(pic.pixels[((y * pic.width + x) * 4 + 3) as usize]);
    let (mut line, mut nl) = (0f64, 0f64);
    for k in 1..cols {
        for x in [k * fw - 1, k * fw] {
            for y in 0..pic.height {
                line += alpha(x, y);
                nl += 1.0;
            }
        }
    }
    for k in 1..rows {
        for y in [k * fh - 1, k * fh] {
            for x in 0..pic.width {
                line += alpha(x, y);
                nl += 1.0;
            }
        }
    }
    let (mut mid, mut nm) = (0f64, 0f64);
    for r in 0..rows {
        for c in 0..cols {
            for y in r * fh + fh / 4..r * fh + fh * 3 / 4 {
                for x in c * fw + fw / 4..c * fw + fw * 3 / 4 {
                    mid += alpha(x, y);
                    nm += 1.0;
                }
            }
        }
    }
    let mid = mid / nm.max(1.0);
    if mid < 1.0 {
        return 0.0;
    }
    ((line / nl.max(1.0)) / mid) as f32
}

/// The frame to show where a flipbook stands still: the one with the most
/// in it (some books fade in from blank frames).
#[must_use]
pub fn poster_frame(pic: &Rgba, grid: (u32, u32)) -> u32 {
    let n = grid.0.max(1) * grid.1.max(1);
    (0..n).map(|i| (i, spread(&frame(pic, grid, i)))).fold((0, -1.0f32), |best, (i, s)| if s > best.1 { (i, s) } else { best }).0
}

/// Premultiplied colour and alpha of a pixel.
fn premul(p: &[u8]) -> [f32; 4] {
    let a = f32::from(p[3]) / 255.0;
    [f32::from(p[0]) * a, f32::from(p[1]) * a, f32::from(p[2]) * a, f32::from(p[3])]
}

/// The mean absolute difference of two same-sized pictures, per channel.
fn mean_diff(a: &Rgba, b: &Rgba) -> f32 {
    let mut d = 0f64;
    for (p, q) in a.pixels.chunks_exact(4).zip(b.pixels.chunks_exact(4)) {
        let (p, q) = (premul(p), premul(q));
        d += (0..4).map(|c| f64::from((p[c] - q[c]).abs())).sum::<f64>();
    }
    (d / (a.pixels.len().max(4) as f64)) as f32
}

/// How much a picture varies in itself: the mean absolute deviation from
/// its mean colour, per channel.
fn spread(a: &Rgba) -> f32 {
    let n = (a.pixels.len() / 4).max(1) as f64;
    let mut mean = [0f64; 4];
    for p in a.pixels.chunks_exact(4) {
        let p = premul(p);
        for c in 0..4 {
            mean[c] += f64::from(p[c]);
        }
    }
    for m in &mut mean {
        *m /= n;
    }
    let mut d = 0f64;
    for p in a.pixels.chunks_exact(4) {
        let p = premul(p);
        d += (0..4).map(|c| (f64::from(p[c]) - mean[c]).abs()).sum::<f64>();
    }
    (d / (n * 4.0)) as f32
}

/// A picture shrunk to fit `max_w` x `max_h` (box filter by a whole step;
/// unchanged when it fits).
#[must_use]
pub fn shrink(pic: &Rgba, max_w: u32, max_h: u32) -> Rgba {
    let step = pic.width.div_ceil(max_w.max(1)).max(pic.height.div_ceil(max_h.max(1))).max(1);
    if step == 1 {
        return pic.clone();
    }
    let (w, h) = ((pic.width / step).max(1), (pic.height / step).max(1));
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0u32; 4];
            for dy in 0..step {
                for dx in 0..step {
                    let (sx, sy) = ((x * step + dx).min(pic.width - 1), (y * step + dy).min(pic.height - 1));
                    let p = &pic.pixels[((sy * pic.width + sx) * 4) as usize..][..4];
                    // (Colour weighted by alpha, so clear pixels do not
                    // darken the edges.)
                    let a = u32::from(p[3]);
                    for c in 0..3 {
                        acc[c] += u32::from(p[c]) * a;
                    }
                    acc[3] += a;
                }
            }
            let a = acc[3];
            let colour = |c: u32| c.checked_div(a).unwrap_or(0) as u8;
            pixels.extend_from_slice(&[colour(acc[0]), colour(acc[1]), colour(acc[2]), (a / (step * step)) as u8]);
        }
    }
    Rgba { width: w, height: h, pixels }
}

/// Frame `i` of a flipbook laid out `grid` (columns, rows), left to right
/// then top to bottom.
#[must_use]
pub fn frame(atlas: &Rgba, grid: (u32, u32), i: u32) -> Rgba {
    let (cols, rows) = (grid.0.max(1), grid.1.max(1));
    let (fw, fh) = (atlas.width / cols, atlas.height / rows);
    let i = i % (cols * rows);
    let (x0, y0) = ((i % cols) * fw, (i / cols) * fh);
    let mut pixels = Vec::with_capacity((fw * fh * 4) as usize);
    for y in y0..y0 + fh {
        let row = ((y * atlas.width + x0) * 4) as usize;
        pixels.extend_from_slice(&atlas.pixels[row..row + (fw * 4) as usize]);
    }
    Rgba { width: fw, height: fh, pixels }
}

/// Block-compressed pixels (BC1, BC3, BC4, BC5, BC7) to RGBA8.
#[must_use]
pub fn decode_bcn(dxgi: u32, w: u32, h: u32, data: &[u8]) -> Option<Vec<u8>> {
    let bs = image::block_bytes(dxgi)?;
    let (w, h) = (w as usize, h as usize);
    let (bw, bh) = (w.div_ceil(4), h.div_ceil(4));
    if data.len() < bw * bh * bs {
        return None;
    }
    let mut out = vec![0u8; w * h * 4];
    let mut tmp = [0u8; 64];
    for by in 0..bh {
        for bx in 0..bw {
            let s = &data[(by * bw + bx) * bs..][..bs];
            match dxgi {
                70..=72 => bcdec_rs::bc1(s, &mut tmp, 16),
                76..=78 => bcdec_rs::bc3(s, &mut tmp, 16),
                79..=81 => {
                    let mut t = [0u8; 16];
                    bcdec_rs::bc4(s, &mut t, 4, false);
                    for (k, v) in t.iter().enumerate() {
                        tmp[k * 4..k * 4 + 4].copy_from_slice(&[*v, *v, *v, 255]);
                    }
                }
                82..=84 => {
                    let mut t = [0u8; 32];
                    bcdec_rs::bc5(s, &mut t, 8, false);
                    for k in 0..16 {
                        tmp[k * 4..k * 4 + 4].copy_from_slice(&[t[k * 2], t[k * 2 + 1], 0, 255]);
                    }
                }
                97..=99 => bcdec_rs::bc7(s, &mut tmp, 16),
                _ => return None,
            }
            for y in 0..4 {
                for x in 0..4 {
                    let (px, py) = (bx * 4 + x, by * 4 + y);
                    if px < w && py < h {
                        out[(py * w + px) * 4..][..4].copy_from_slice(&tmp[y * 16 + x * 4..][..4]);
                    }
                }
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(name: &str) -> Vec<u8> {
        let mut v = FOLLOW.to_le_bytes().to_vec();
        v.extend([1, 2, 3, 4, 0, 0, 0, 0]);
        v.extend(name.as_bytes());
        v.push(0);
        v
    }

    #[test]
    fn kinds_by_name() {
        assert_eq!(kind_of("ui_emblem_005"), Some(Kind::Emblem));
        assert_eq!(kind_of("jup_emblem_n5934_1"), Some(Kind::Emblem));
        assert_eq!(kind_of("ui_playercard_100"), Some(Kind::Card));
        assert_eq!(kind_of("jup_playercard_animated_opc_outbreak_mike4_tier1_atlas"), Some(Kind::Card));
        assert_eq!(kind_of("EMBLEMS/EMBLEM_005"), None);
        assert_eq!(kind_of("ui_group_bg_emblem_1"), None);
        assert_eq!(kind_of("jup_wc_jup_emblem_n5934_1"), None);
        assert_eq!(kind_of("cos_iw9_532_prestigeemblem"), None);
        assert!(is_animated("jup_emblem_animated_opc_goldennewyears_01_altas"));
    }

    #[test]
    fn names_from_cells() {
        let mut z = vec![9u8; 3];
        z.extend(cell("ui_emblem_005"));
        z.extend(cell("EMBLEMS/EMBLEM_005"));
        z.extend(cell("ui_playercard_100"));
        z.extend(cell("ui_emblem_005"));
        // (Not a cell: no zero word after the hash.)
        z.extend(FOLLOW.to_le_bytes());
        z.extend([1, 2, 3, 4, 5, 0, 0, 0]);
        z.extend(b"ui_emblem_999\0");
        assert_eq!(scan_names(&z), vec![("ui_emblem_005".to_owned(), Kind::Emblem), ("ui_playercard_100".to_owned(), Kind::Card)]);
    }

    fn rec(w: u16, h: u16) -> ImageRec {
        ImageRec {
            at: 0,
            hash: 0,
            flags: 0,
            data_size: 0,
            width: w,
            height: h,
            depth: 1,
            array: 1,
            format: 0x30,
            usage: 0,
            levels: 1,
            groups: Vec::new(),
            inline: None,
            end: 0,
        }
    }

    #[test]
    fn flipbook_grids() {
        let e = |w, h| Item { name: "ui_emblem_x".into(), kind: Kind::Emblem, image: rec(w, h) };
        let c = |w, h| Item { name: "ui_playercard_x".into(), kind: Kind::Card, image: rec(w, h) };
        assert_eq!(e(768, 1024).candidate_grid(), Some((3, 4)));
        assert_eq!(e(768, 1024).grid_by_size(), Some((3, 4)));
        assert_eq!(e(1024, 1024).candidate_grid(), Some((4, 4)));
        assert_eq!(e(4096, 4096).grid_by_size(), None);
        assert_eq!(e(256, 256).grid_by_size(), Some((1, 1)));
        assert_eq!(e(1920, 1920).candidate_grid(), Some((4, 4)));
        assert_eq!(e(848, 1144).grid_by_size(), Some((1, 1)));
        assert_eq!(c(1024, 1024).grid_by_size(), Some((2, 8)));
        assert_eq!(c(1536, 512).grid_by_size(), Some((3, 4)));
        assert_eq!(c(768, 256).grid_by_size(), Some((3, 4)));
        assert_eq!(c(1024, 768).grid_by_size(), Some((2, 6)));
        assert_eq!(c(1024, 512).grid_by_size(), Some((2, 4)));
        assert_eq!(c(512, 128).grid_by_size(), Some((1, 1)));
        assert_eq!(c(2048, 512).grid_by_size(), Some((1, 1)));
        assert_eq!(c(1024, 640).grid_by_size(), Some((1, 1)));
    }

    #[test]
    fn frames_and_shrinking() {
        // Frame 4 of a 3x2 atlas of 2x1 frames: row 1, column 1.
        let atlas = Rgba { width: 6, height: 2, pixels: (0..12u8).flat_map(|i| [i, 0, 0, 255]).collect() };
        let f = frame(&atlas, (3, 2), 4);
        assert_eq!((f.width, f.height), (2, 1));
        assert_eq!(f.pixels[0], 8);
        assert_eq!(f.pixels[4], 9);
        let s = shrink(&atlas, 3, 1);
        assert_eq!((s.width, s.height), (3, 1));
        assert_eq!(&s.pixels[..4], &[3, 0, 0, 255]);
    }

    /// A 4x4 book of 8x8 frames. `same`: every frame the same pattern;
    /// else the tiles are parts of one picture, opaque across the frame
    /// lines. `gutter`: each frame's outer pixel clear.
    fn book(same: bool, gutter: bool) -> Rgba {
        let mut pixels = Vec::new();
        for y in 0..32u32 {
            for x in 0..32u32 {
                let v = if same { ((x % 8) * 31 + (y % 8) * 17) % 255 } else { (x * 13 + y * 29 + (x * y) % 7 * 40) % 255 } as u8;
                let edge = matches!(x % 8, 0 | 7) || matches!(y % 8, 0 | 7);
                let a = if gutter && edge { 0 } else { 255 };
                pixels.extend_from_slice(&[v, 255 - v, v / 2, a]);
            }
        }
        Rgba { width: 32, height: 32, pixels }
    }

    #[test]
    fn a_flipbook_is_told_from_a_still() {
        let mut it = Item { name: "ui_emblem_x".into(), kind: Kind::Emblem, image: rec(1024, 1024) };
        assert_eq!(it.grid_by_size(), None);
        assert_eq!(detect_grid(&it, &book(true, false)), (4, 4));
        assert_eq!(detect_grid(&it, &book(false, false)), (1, 1));
        // (Frames that change a lot but stand apart.)
        assert_eq!(detect_grid(&it, &book(false, true)), (4, 4));
        assert!(gutters(&book(false, true), (4, 4)) < 0.01);
        assert!(gutters(&book(false, false), (4, 4)) > 0.9);
        it.image = rec(256, 256);
        assert_eq!(detect_grid(&it, &book(true, false)), (1, 1));
        // (By shape alone.)
        it.image = rec(768, 1024);
        assert_eq!(detect_grid(&it, &book(false, false)), (3, 4));
    }

    #[test]
    fn the_poster_is_the_fullest_frame() {
        // Frame 0 blank, frame 2 the busiest.
        let mut pixels = Vec::new();
        for _y in 0..2u32 {
            for x in 0..6u32 {
                let (v, a) = match x / 2 {
                    0 => (0, 0),
                    1 => (if x % 2 == 0 { 10 } else { 20 }, 255),
                    _ => (if x % 2 == 0 { 0 } else { 250 }, 255),
                };
                pixels.extend_from_slice(&[v, v, v, a]);
            }
        }
        let pic = Rgba { width: 6, height: 2, pixels };
        assert_eq!(poster_frame(&pic, (3, 1)), 2);
    }
}
