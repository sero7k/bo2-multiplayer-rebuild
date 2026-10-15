//! The world's own surfaces: the map's `gfx_map` lists them (a 128-byte
//! record each: where its vertex streams are, how many vertices, where its
//! packed triangles are) and the `gfx_map_trzone` holds the data they name.
//!
//! A world zone (`gfx_map_trzone`, named after its map or its tile):
//! ```text
//! +0x00 u64 name hash
//! +0x18 u32 vertex bytes, u32 index list length (u16s),
//!       u32 triangle tables, u32 packed triangle bytes
//! +0x178 the vertex streams, then the index list (u16, padded to 4),
//!        the tables (40 bytes each), the packed triangles
//! ```
//! A surface (`gfx_map`, 128 bytes):
//! ```text
//! +0x00 f32[3] origin, f32 step       position = origin + q * step
//! +0x10 u32 its world zone: 0 the map's own, k the k-th streamed tile
//!       (the map's `<map>_*_0_tr` zones in name order, see [`super::tiles`])
//! +0x14 u32 0xffff
//! +0x18 u32 lightmap (510: none), u32 stream count (1..4)
//! +0x20 u32 vertices, u32 packed triangle bytes
//! +0x28 u32 positions (8 each: three 21-bit q), tangent frames (4),
//!       lightmap UVs (f32 x2, 0: none), extra stream, texture UVs (f32 x2)
//!       (offsets in its world zone's vertex streams)
//! +0x40 u32 its packed triangles' offset
//! ```
//! A table (40 bytes, at most 64 triangles): +0x1C u32 its first entry in
//! the surface's index list, +0x20 u16 first triangle, u8 entries, u8
//! triangles, +0x24 u32 its bits' offset in the surface's packed triangles.
//! Each corner is `ceil(log2(entries))` bits, least significant first, an
//! entry of the index list. Tables run surface by surface (a surface's first
//! has first triangle 0) in packed-triangle order, and the index list too.

use super::index::{f32_at, find_all, u16_at, u32_at, u64_at, v3_at};

/// A surface of the world, decoded.
#[derive(Clone, Debug, Default)]
pub struct WorldSurface {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub lightmap_uvs: Vec<[f32; 2]>,
    pub tris: Vec<[u16; 3]>,
    /// Its index in the map's surface list.
    pub index: usize,
    /// Its world zone (0: the map's own; k: the k-th tile).
    pub zone: u32,
    /// Its lightmap (510: none).
    pub lightmap: u32,
}

const SURFACE: usize = 0x80;
const TABLE: usize = 40;
const TRZONE_DATA: usize = 0x178;

/// The world zone's buffers, by their place in the image.
#[derive(Clone, Copy, Debug)]
pub struct WorldBuffers {
    pub header: usize,
    pub vertices: usize,
    pub vertex_bytes: usize,
    pub list: usize,
    pub list_len: usize,
    pub tables: usize,
    pub table_count: usize,
    pub packed: usize,
    pub packed_bytes: usize,
}

/// Finds a world zone's header by its name hash (and its counts fitting
/// the image).
pub fn find_buffers(z: &[u8], name_hash: u64) -> Option<WorldBuffers> {
    for at in find_all(z, &name_hash.to_le_bytes()) {
        let vb = u32_at(z, at + 0x18) as usize;
        let list_len = u32_at(z, at + 0x1C) as usize;
        let tables = u32_at(z, at + 0x20) as usize;
        let packed_bytes = u32_at(z, at + 0x24) as usize;
        if vb == 0
            || tables == 0
            || (1..=4).any(|k| u64_at(z, at + 0x20 + 8 * k) != super::index::FOLLOW)
        {
            continue;
        }
        let vertices = at + TRZONE_DATA;
        let list = vertices + vb;
        let tab = list + (2 * list_len).next_multiple_of(4);
        let packed = tab + TABLE * tables;
        if packed + packed_bytes > z.len() {
            continue;
        }
        // The first table starts a surface.
        if u16_at(z, tab + 0x20) != 0 || u32_at(z, tab + 0x1C) != 0 {
            continue;
        }
        return Some(WorldBuffers {
            header: at,
            vertices,
            vertex_bytes: vb,
            list,
            list_len,
            tables: tab,
            table_count: tables,
            packed,
            packed_bytes,
        });
    }
    None
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceRec {
    pub zone: u32,
    pub lightmap: u32,
    origin: [f32; 3],
    step: f32,
    verts: usize,
    packed_size: usize,
    pos: usize,
    uv: usize,
    lmap: usize,
    packed_at: usize,
}

fn surface_rec(z: &[u8], at: usize) -> SurfaceRec {
    SurfaceRec {
        zone: u32_at(z, at + 0x10),
        lightmap: u32_at(z, at + 0x18),
        origin: v3_at(z, at),
        step: f32_at(z, at + 0x0C),
        verts: u32_at(z, at + 0x20) as usize,
        packed_size: u32_at(z, at + 0x24) as usize,
        pos: u32_at(z, at + 0x28) as usize,
        lmap: u32_at(z, at + 0x30) as usize,
        uv: u32_at(z, at + 0x38) as usize,
        packed_at: u32_at(z, at + 0x40) as usize,
    }
}

/// A record reads as a surface whose streams follow one another (its
/// world zone's buffers are not needed to tell).
fn plausible(z: &[u8], at: usize) -> bool {
    if at + SURFACE > z.len() {
        return false;
    }
    let r = surface_rec(z, at);
    let tan = u32_at(z, at + 0x2C) as usize;
    let streams = u32_at(z, at + 0x1C);
    r.verts > 0
        && r.verts < 0x10000
        && u32_at(z, at + 0x14) == 0xFFFF
        && (1..=8).contains(&streams)
        && r.step.is_finite()
        && r.step > 0.0
        && r.step < 1.0
        && tan == r.pos + 8 * r.verts
        && (r.lmap == 0 || r.lmap == tan + 4 * r.verts)
        && r.packed_size > 0
}

/// The map's surface list: the longest run of plausible records.
pub fn find_surfaces(z: &[u8]) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut at = 0usize;
    while at + SURFACE <= z.len() {
        if plausible(z, at) {
            let mut n = 1;
            while plausible(z, at + SURFACE * n) {
                n += 1;
            }
            if n > best.map_or(0, |x| x.1) {
                best = Some((at, n));
            }
            at += SURFACE * n;
            continue;
        }
        at += 1;
    }
    best
}

/// The records of the list at `list_at`.
pub fn records(z: &[u8], list_at: usize, count: usize) -> Vec<SurfaceRec> {
    (0..count)
        .map(|i| surface_rec(z, list_at + SURFACE * i))
        .collect()
}

/// The packed triangle bytes the records give a world zone.
pub fn packed_of(recs: &[SurfaceRec], zone: u32) -> usize {
    recs.iter()
        .filter(|r| r.zone == zone)
        .map(|r| r.packed_size)
        .sum()
}

fn bits_for(n: usize) -> u32 {
    let mut b = 0;
    while (1usize << b) < n {
        b += 1;
    }
    b
}

/// The packed bytes a surface's tables `first..end` take and the most
/// index list entries one of them has.
fn group_needs(z: &[u8], b: &WorldBuffers, first: usize, end: usize) -> (usize, usize) {
    let last = b.tables + TABLE * (end - 1);
    let entries = z[last + 0x22] as usize;
    let ntri = z[last + 0x23] as usize;
    let bits = bits_for(entries) as usize;
    let need = u32_at(z, last + 0x24) as usize + (3 * ntri * bits).div_ceil(8);
    let most = (first..end)
        .map(|t| z[b.tables + TABLE * t + 0x22] as usize)
        .max()
        .unwrap_or(0);
    (need, most)
}

/// Decodes the surfaces of the list that are in world zone `zone` (its
/// buffers `b` in the zone image `z`).
pub fn read_surfaces(
    z: &[u8],
    b: &WorldBuffers,
    recs: &[SurfaceRec],
    zone: u32,
) -> Result<Vec<WorldSurface>, String> {
    let mut pending: Vec<usize> = (0..recs.len()).filter(|&i| recs[i].zone == zone).collect();
    pending.sort_by_key(|&i| recs[i].packed_at);
    let mut out: Vec<WorldSurface> = Vec::with_capacity(pending.len());
    let mut table = 0usize;
    let mut list = 0usize;
    while !pending.is_empty() {
        if table >= b.table_count {
            return Err(format!(
                "{} surfaces left, no triangle table left ({} read)",
                pending.len(),
                b.table_count
            ));
        }
        // The next surface's tables: the next, and the ones after it that do
        // not start at triangle 0.
        let first = table;
        table += 1;
        while table < b.table_count && u16_at(z, b.tables + TABLE * table + 0x20) != 0 {
            table += 1;
        }
        // Its surface: the tables run in packed-triangle order but for a few
        // surfaces placed later, so the first nearby one whose packed bytes
        // and vertices fit them.
        let (need, most) = group_needs(z, b, first, table);
        let pick = pending
            .iter()
            .take(32)
            .position(|&i| {
                let r = &recs[i];
                (need..need + 8).contains(&r.packed_size) && most <= r.verts
            })
            .unwrap_or(0);
        let si = pending.remove(pick);
        let r = recs[si];
        let mut used = 0usize;
        let mut tris = Vec::new();
        for t in first..table {
            let e = b.tables + TABLE * t;
            let x = u32_at(z, e + 0x1C) as usize;
            let entries = z[e + 0x22] as usize;
            let ntri = z[e + 0x23] as usize;
            let bit_at = u32_at(z, e + 0x24) as usize;
            used = used.max(x + entries);
            let nb = bits_for(entries);
            let base = b.packed + r.packed_at + bit_at;
            let mut bp = 0usize;
            for _ in 0..ntri {
                let mut tri = [0u16; 3];
                for c in &mut tri {
                    let mut v = 0usize;
                    for q in 0..nb as usize {
                        let p = bp + q;
                        let byte = z
                            .get(base + (p >> 3))
                            .copied()
                            .ok_or("packed triangles run off the zone")?;
                        v |= usize::from((byte >> (p & 7)) & 1) << q;
                    }
                    bp += nb as usize;
                    if v >= entries {
                        return Err(format!(
                            "surface {si}: corner {v} past the table's {entries} entries"
                        ));
                    }
                    *c = u16_at(z, b.list + 2 * (list + x + v));
                }
                tris.push(tri);
            }
        }
        list += used;
        let mut s = WorldSurface {
            index: si,
            zone,
            lightmap: r.lightmap,
            ..Default::default()
        };
        s.positions = (0..r.verts)
            .map(|k| {
                let q = u64_at(z, b.vertices + r.pos + 8 * k);
                std::array::from_fn(|c| r.origin[c] + ((q >> (21 * c)) & 0x1F_FFFF) as f32 * r.step)
            })
            .collect();
        s.uvs = (0..r.verts)
            .map(|k| {
                [
                    f32_at(z, b.vertices + r.uv + 8 * k),
                    f32_at(z, b.vertices + r.uv + 8 * k + 4),
                ]
            })
            .collect();
        s.lightmap_uvs = (0..r.verts)
            .filter(|_| r.lmap != 0)
            .map(|k| {
                [
                    f32_at(z, b.vertices + r.lmap + 8 * k),
                    f32_at(z, b.vertices + r.lmap + 8 * k + 4),
                ]
            })
            .collect();
        tris.retain(|t| t.iter().all(|&v| usize::from(v) < r.verts));
        s.tris = tris;
        out.push(s);
    }
    if list != b.list_len {
        return Err(format!("index list: {list} entries used of {}", b.list_len));
    }
    Ok(out)
}
