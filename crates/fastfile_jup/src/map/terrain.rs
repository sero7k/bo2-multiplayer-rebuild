//! The map's terrain: its coarse height grid and its virtual texture's
//! colour tiles. The ground between the buildings of a map like Rebirth
//! Island is not in the world's surfaces: the terrain system draws it from
//! a height field and a virtual texture.
//!
//! - The height field (asset of the map's name, ahead of the `stterrain`'s
//!   data): +0x00 name hash, +0x14 two f32 half extents (16384), +0x20 u32
//!   samples a side (1025) twice, +0x28 u32 128, +0x2c f32 lowest height,
//!   +0x30 f32 8191, +0x38 u32 1026, +0x3c two f32 extents (32768), +0x44
//!   u32 85 (a quadtree's nodes to depth 3). A coarse grid follows at
//!   +0x100: 132 x 132 u16 (128 cells of 256 units and two more each side),
//!   row by row along +y, sample `i` at `-16384 - 2 * 256 + (i + 0.5) * 256`,
//!   height `lowest + v / 8`. The full 1025-sample field is not in the zone.
//!   Checked on Rebirth: the path nodes on no world surface stand on it
//!   (73% within 32 units), about 20 units above it (a low bound per cell).
//! - The virtual texture: a table of `(4^L - 1) / 3` tiles (a quadtree,
//!   level by level, each level row by row from -x -y), 5 back-pointers to
//!   image entries a tile; the BC1 one (format 0x25) is its colour. Tiles
//!   are 256 px with a 4 px border, the inner 248 px covering a level-`l`
//!   tile's square of `extent / 2^l`. Where the texture lies was fitted
//!   against the height grid's coast (Rebirth: from (-16225, -16225), 34900
//!   units a side; the land masks agree on 95%).
use super::index::{ZoneImage, f32_at, find_all, u16_at, u32_at, u64_at};
use super::materials::MapImage;
use crate::image::ImageRun;

/// The coarse height grid.
#[derive(Clone, Debug, Default)]
pub struct Terrain {
    /// Samples a side.
    pub size: usize,
    /// World place of sample (0, 0).
    pub origin: [f32; 2],
    /// Units between samples.
    pub cell: f32,
    /// Half the side of the square the terrain's 1025 samples cover.
    pub half: f32,
    /// Heights, row by row (rows along +y, samples along +x).
    pub heights: Vec<f32>,
}

impl Terrain {
    /// The height at a sample.
    #[must_use]
    pub fn at(&self, i: usize, j: usize) -> f32 {
        self.heights[j * self.size + i]
    }
}

/// The virtual texture's colour tiles.
#[derive(Clone, Debug, Default)]
pub struct VirtualTexture {
    /// Levels of the quadtree (Rebirth: 6, 1 to 32 tiles a side).
    pub levels: u32,
    /// Each tile's colour picture, in the table's order.
    pub tiles: Vec<Option<MapImage>>,
    /// World place of the texture's -x -y corner and its side.
    pub origin: [f32; 2],
    pub extent: f32,
}

/// Coarse samples beyond the 128 cells on each side.
const BORDER_CELLS: f32 = 2.0;
/// The coarse grid's place after the header.
const GRID_AT: usize = 0x100;
/// How far the path nodes stand above the coarse grid (a low bound).
const LIFT: f32 = 20.0;

/// The map's coarse height grid, where it has a terrain.
#[must_use]
pub fn height_grid(z: &[u8], name_hash: u64) -> Option<Terrain> {
    for at in find_all(z, &name_hash.to_le_bytes()) {
        let half = [f32_at(z, at + 0x14), f32_at(z, at + 0x18)];
        let samples = [u32_at(z, at + 0x20), u32_at(z, at + 0x24)];
        let cells = u32_at(z, at + 0x28);
        let lowest = f32_at(z, at + 0x2c);
        let extent = f32_at(z, at + 0x3c);
        if samples[0] != samples[1]
            || samples[0] != cells * 8 + 1
            || half[0] != half[1]
            || half[0] <= 0.0
            || extent != 2.0 * half[0]
            || !lowest.is_finite()
            || cells == 0
            || cells > 1024
        {
            continue;
        }
        let cell = extent / cells as f32;
        let size = cells as usize + 2 * BORDER_CELLS as usize;
        let grid = at + GRID_AT;
        if grid + 2 * size * size > z.len() {
            continue;
        }
        let heights = (0..size * size)
            .map(|k| lowest + f32::from(u16_at(z, grid + 2 * k)) / 8.0 + LIFT)
            .collect();
        let first = -half[0] - BORDER_CELLS * cell + 0.5 * cell;
        return Some(Terrain {
            size,
            origin: [first, first],
            cell,
            half: half[0],
            heights,
        });
    }
    None
}

/// The virtual texture's colour tiles: the table of `(4^L - 1) / 3`
/// tiles' 5 image back-pointers. `None` when the zone has none. It lies
/// over the terrain's square (`half` its half side) as fitted on Rebirth.
#[must_use]
pub fn virtual_texture(zone: &ZoneImage, run: &ImageRun, half: f32) -> Option<VirtualTexture> {
    let z = &zone.bytes;
    // Image entries by asset index: the k-th image is the run's k-th.
    let mut nth = vec![usize::MAX; zone.types.len()];
    let mut k = 0usize;
    for (i, &t) in zone.types.iter().enumerate() {
        if t == 21 {
            nth[i] = k;
            k += 1;
        }
    }
    for levels in (4..=8u32).rev() {
        let count = ((1u64 << (2 * levels)) - 1) / 3;
        for at in find_all(z, &count.to_le_bytes()) {
            let table = at + 8;
            if table + 8 * 5 * count as usize > z.len() {
                continue;
            }
            let first = u64_at(z, table);
            if first >> 32 != 0x0f || first & 1 == 0 {
                continue;
            }
            let Some(base) = super::models::entry_base(zone, table) else {
                continue;
            };
            let mut tiles = Vec::with_capacity(count as usize);
            let mut ok = true;
            for t in 0..count as usize {
                let mut colour = None;
                for e in 0..5 {
                    let v = u64_at(z, table + 8 * (5 * t + e));
                    let Some(a) = super::models::entry_of(zone, base, v) else {
                        ok = false;
                        break;
                    };
                    if zone.types[a] != 21 {
                        ok = false;
                        break;
                    }
                    if colour.is_none()
                        && let Some(&img_at) = run.at.get(nth[a])
                    {
                        let img = super::materials::map_image(z, img_at);
                        if img.format == 0x25 && img.width == 256 && img.height == 256 {
                            colour = Some(img);
                        }
                    }
                }
                if !ok {
                    break;
                }
                tiles.push(colour);
            }
            if ok {
                return Some(VirtualTexture {
                    levels,
                    tiles,
                    origin: [half * VT_ORIGIN, half * VT_ORIGIN],
                    extent: half * VT_EXTENT,
                });
            }
        }
    }
    None
}

/// Where the virtual texture lies, in the terrain's half sides (fitted on
/// Rebirth, see the module: from -16225, 34900 a side, the half side
/// 16384).
const VT_ORIGIN: f32 = -16225.0 / 16384.0;
const VT_EXTENT: f32 = 34900.0 / 16384.0;

/// The first tile of a level in the table.
#[must_use]
pub fn level_start(level: u32) -> usize {
    (((1u64 << (2 * level)) - 1) / 3) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_from_its_header() {
        let hash = 0x2967_331b_f0f1_8bc6u64;
        let at = 16;
        let mut z = vec![0u8; at + GRID_AT + 2 * 12 * 12 + 64];
        let put = |z: &mut Vec<u8>, o: usize, b: &[u8]| z[at + o..at + o + b.len()].copy_from_slice(b);
        put(&mut z, 0, &hash.to_le_bytes());
        put(&mut z, 0x14, &1024f32.to_le_bytes());
        put(&mut z, 0x18, &1024f32.to_le_bytes());
        put(&mut z, 0x20, &65u32.to_le_bytes());
        put(&mut z, 0x24, &65u32.to_le_bytes());
        put(&mut z, 0x28, &8u32.to_le_bytes());
        put(&mut z, 0x2c, &(-100f32).to_le_bytes());
        put(&mut z, 0x3c, &2048f32.to_le_bytes());
        // Sample (1, 0) 100 units over the lowest.
        put(&mut z, GRID_AT + 2, &800u16.to_le_bytes());
        let g = height_grid(&z, hash).expect("grid");
        assert_eq!(g.size, 12);
        assert_eq!(g.cell, 256.0);
        assert_eq!(g.origin, [-1024.0 - 512.0 + 128.0; 2]);
        assert_eq!(g.at(1, 0), -100.0 + 100.0 + LIFT);
        assert_eq!(g.at(0, 0), -100.0 + LIFT);
        assert!(height_grid(&z, hash ^ 1).is_none());
    }

    #[test]
    fn levels_start_where_the_coarser_end() {
        assert_eq!(level_start(0), 0);
        assert_eq!(level_start(3), 21);
        assert_eq!(level_start(6), 1365);
    }
}
