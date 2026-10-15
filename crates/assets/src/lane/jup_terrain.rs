//! bo2mp: Modern Warfare III's terrain as world surfaces.
//!
//! The ground between a map's buildings (Rebirth Island's hills, paths and
//! shore) is not among its world surfaces: the terrain system draws it from
//! a height field and a virtual texture (`fastfile_jup::map::terrain`).
//! Here the zone's coarse height grid becomes a mesh of world surfaces in
//! square chunks (so the ones under the sea can be left out of collision),
//! textured by the virtual texture's colour tiles laid side by side: each
//! level of the texture's quadtree is one mip level of a BC1 picture, the
//! tiles' 4 px borders (one BC1 block) cut off so the picture maps
//! linearly onto the ground.

use fastfile_jup::map::materials::Packages;
use fastfile_jup::map::terrain::{Terrain, VirtualTexture, level_start};
use fastfile_jup::oodle::Decompressor;
use fastfile_t7::map::{T7Map, WorldSurface, WorldVertex};
use fastfile_t7::texture::ImageLevels;

/// Cells a side of a chunk (one world surface).
const CHUNK: usize = 16;

/// The terrain mesh's surfaces, added to `t7` with `material`. Returns
/// their indices.
pub(crate) fn add_mesh(
    t7: &mut T7Map,
    terrain: &Terrain,
    vt: Option<&VirtualTexture>,
    material: Option<usize>,
) -> Vec<usize> {
    let n = terrain.size;
    let (o, cell) = (terrain.origin, terrain.cell);
    let pos = |i: usize, j: usize| [o[0] + i as f32 * cell, o[1] + j as f32 * cell, terrain.at(i, j)];
    let normal = |i: usize, j: usize| {
        let (il, ir) = (i.saturating_sub(1), (i + 1).min(n - 1));
        let (jl, jr) = (j.saturating_sub(1), (j + 1).min(n - 1));
        let dx = (terrain.at(ir, j) - terrain.at(il, j)) / ((ir - il).max(1) as f32 * cell);
        let dy = (terrain.at(i, jr) - terrain.at(i, jl)) / ((jr - jl).max(1) as f32 * cell);
        let l = (dx * dx + dy * dy + 1.0).sqrt();
        [-dx / l, -dy / l, 1.0 / l]
    };
    let uv = |p: [f32; 3]| match vt {
        Some(v) => [(p[0] - v.origin[0]) / v.extent, (p[1] - v.origin[1]) / v.extent],
        None => [p[0] / 512.0, p[1] / 512.0],
    };
    let mut out = Vec::new();
    let cells = n - 1;
    for cj in (0..cells).step_by(CHUNK) {
        for ci in (0..cells).step_by(CHUNK) {
            let (wi, wj) = (CHUNK.min(cells - ci), CHUNK.min(cells - cj));
            let first_vertex = t7.vertices.len() as u32;
            let base_index = t7.indices.len() as u32;
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for j in 0..=wj {
                for i in 0..=wi {
                    let p = pos(ci + i, cj + j);
                    for k in 0..3 {
                        lo[k] = lo[k].min(p[k]);
                        hi[k] = hi[k].max(p[k]);
                    }
                    let nrm = normal(ci + i, cj + j);
                    t7.vertices.push(WorldVertex {
                        xyz: p,
                        color: 0xFFFF_FFFF,
                        uv: uv(p),
                        normal: nrm,
                        tangent: [1.0, 0.0, 0.0],
                    });
                }
            }
            let row = (wi + 1) as u16;
            for j in 0..wj as u16 {
                for i in 0..wi as u16 {
                    let a = j * row + i;
                    let (b, c, d) = (a + 1, a + row, a + row + 1);
                    // Clockwise seen from above, as the world's.
                    t7.indices.extend([a, d, b, a, c, d]);
                }
            }
            out.push(t7.surfaces.len());
            t7.surfaces.push(WorldSurface {
                first_vertex,
                vertex_count: ((wi + 1) * (wj + 1)) as u32,
                tri_count: (2 * wi * wj) as u16,
                base_index,
                material,
                flags: 0,
                mins: lo,
                maxs: hi,
            });
        }
    }
    out
}

/// BC1 blocks a tile side, and its border (one block).
const TILE_BLOCKS: usize = 64;
const BORDER: usize = 1;
const INNER: usize = TILE_BLOCKS - 2 * BORDER;
/// BC1 in the builder's numbering (DXGI BC1_UNORM).
const BC1: u32 = 71;
/// A plain sandy block for a tile the install lacks.
const SAND: [u8; 8] = {
    // 565 colours (180, 160, 120) twice, all texels colour 0.
    let c: u16 = ((180 >> 3) << 11) | ((160 >> 2) << 5) | (120 >> 3);
    let b = c.to_le_bytes();
    [b[0], b[1], b[0], b[1], 0, 0, 0, 0]
};

/// The virtual texture's colour as one BC1 picture: level `top` of the
/// quadtree its biggest mip level, each coarser level the next.
pub(crate) fn picture(
    vt: &VirtualTexture,
    paks: &Packages,
    oodle: &dyn Decompressor,
    top: u32,
    report: &mut Vec<String>,
) -> Option<ImageLevels> {
    let top = top.min(vt.levels.checked_sub(1)?);
    let started = std::time::Instant::now();
    let wanted: Vec<usize> = (0..=top)
        .flat_map(|l| level_start(l)..level_start(l) + (1usize << (2 * l)))
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let got: std::sync::Mutex<std::collections::HashMap<usize, Vec<u8>>> = Default::default();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get().clamp(2, 8));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&t) = wanted.get(k) else { break };
                    let Some(Some(img)) = vt.tiles.get(t) else { continue };
                    if let Ok(l) = paks.level(img, 256, oodle)
                        && l.dxgi == BC1
                        && l.width == 256
                        && l.height == 256
                        && l.data.len() >= TILE_BLOCKS * TILE_BLOCKS * 8
                        && let Ok(mut g) = got.lock()
                    {
                        g.insert(t, l.data);
                    }
                }
            });
        }
    });
    let got = got.into_inner().unwrap_or_default();
    let mut data = Vec::new();
    let mut missing = 0usize;
    for level in (0..=top).rev() {
        let n = 1usize << level;
        let side = n * INNER;
        let mut blocks = vec![0u8; side * side * 8];
        for ty in 0..n {
            for tx in 0..n {
                let tile = got.get(&(level_start(level) + ty * n + tx));
                if tile.is_none() {
                    missing += 1;
                }
                for by in 0..INNER {
                    for bx in 0..INNER {
                        let dst = ((ty * INNER + by) * side + tx * INNER + bx) * 8;
                        let src = ((by + BORDER) * TILE_BLOCKS + bx + BORDER) * 8;
                        let b = tile.map_or(&SAND[..], |t| &t[src..src + 8]);
                        blocks[dst..dst + 8].copy_from_slice(b);
                    }
                }
            }
        }
        data.push(blocks);
    }
    // Biggest level first.
    data.reverse();
    let side = (1u32 << top) * (INNER as u32) * 4;
    report.push(format!(
        "jup terrain: texture {side} px a side, {} levels, {} of {} tiles ({missing} missing) in {:.1}s",
        top + 1,
        got.len(),
        wanted.len(),
        started.elapsed().as_secs_f32()
    ));
    (got.len() > wanted.len() / 2).then(|| ImageLevels {
        format: BC1,
        width: side,
        height: side,
        levels: top + 1,
        data: data.concat(),
        part: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_faces_up_and_tiles_the_grid() {
        let terrain = Terrain {
            size: 20,
            origin: [-100.0, -100.0],
            cell: 10.0,
            half: 80.0,
            heights: vec![5.0; 400],
        };
        let mut t7 = T7Map::default();
        let s = add_mesh(&mut t7, &terrain, None, None);
        // 19 cells a side: chunks of 16 and 3.
        assert_eq!(s.len(), 4);
        let tris: usize = t7.surfaces.iter().map(|s| usize::from(s.tri_count)).sum();
        assert_eq!(tris, 2 * 19 * 19);
        let f = &t7.surfaces[0];
        let v = |k: usize| t7.vertices[f.first_vertex as usize + usize::from(t7.indices[f.base_index as usize + k])].xyz;
        let (a, b, c) = (v(0), v(1), v(2));
        let z = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        assert!(z < 0.0, "clockwise from above");
        assert_eq!(t7.vertices[0].normal, [0.0, 0.0, 1.0]);
    }
}
