//! Model meshes (asset type 8, `xmodelsurfs`): surfaces, their packed
//! vertex streams and packed triangle lists, in one blob inline in the zone
//! (the `_tr` part zones of guns, the map tiles).
//!
//! ```text
//! header 0x50: u64 hash, ptr surfaces (-2), u64 0, u64 0,
//!   ptr shared blob, u32 surface count, u32 flags, 0x20 zero bytes
//! surface 0xe0:
//!   0x00 u16 flags, u16 triangle count, u16 packet count, u16 packet count
//!   0x08 u32 vertex count, u32 packed index bytes, u32 0x10000, u32 0
//!   0x18 u32 vertex count, u32 triangle count, u32 ?
//!   0x3c f32 -1
//!   0x40 i32 stream offsets[13] into the blob (-1: none):
//!        0 positions (8 bytes), 1 UVs (2 halves), 2 tangent frame (4),
//!        3 packet vertex lists (u16), 4 packet table (40 bytes each),
//!        5 packed indices, 8 second UVs (2 halves), 9 colours (4 bytes)
//!   0x78 ptr shared blob ({ptr -2, u32 bytes, u32 ?} then the bytes;
//!        the later surfaces point back at the first's)
//!   0x80 ptr rigid ranges (-2: `range count` 12-byte records)
//!   0xb0 f32 centre[3], extents[3] (half sizes): positions are 21-bit
//!        fractions of twice the largest extent about the centre
//!   0xc8 ptr 12 bytes (not read)
//! then: blob struct and bytes; per surface its rigid ranges and its
//! 12 bytes.
//! packet (40 bytes): 0x00 16 bytes (bounds), 0x10 12 bytes,
//!   0x1c u32 first entry in the packet vertex list,
//!   0x20 u16 first triangle, u8 vertex count, u8 triangle count,
//!   0x24 u32 offset of its indices in the packed indices.
//!   Indices are `ceil(log2(vertex count))` bits each, low bits first,
//!   indexing the packet's run of the vertex list (u16 surface vertices).
//! rigid range (12 bytes): u16 bone, vertex count, first vertex,
//!   triangle count, first packet, packet count.
//! ```
#![forbid(unsafe_code)]

use crate::ptr::{Bytes, OutOfBounds, Ptr};

pub const HEADER_LEN: usize = 0x50;
pub const SURFACE_LEN: usize = 0xe0;
pub const PACKET_LEN: usize = 40;
pub const RANGE_LEN: usize = 12;

#[derive(Debug)]
pub enum SurfsError {
    Bounds(OutOfBounds),
    Layout(&'static str),
}

impl std::fmt::Display for SurfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bounds(e) => write!(f, "{e}"),
            Self::Layout(w) => write!(f, "mesh layout: {w}"),
        }
    }
}

impl std::error::Error for SurfsError {}

impl From<OutOfBounds> for SurfsError {
    fn from(e: OutOfBounds) -> Self {
        Self::Bounds(e)
    }
}

/// One rigid range of a surface: vertices bound to one bone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RigidRange {
    pub bone: u16,
    pub vert_count: u16,
    pub first_vert: u16,
    pub tri_count: u16,
    pub first_packet: u16,
    pub packet_count: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceRec {
    pub flags: u16,
    pub tri_count: u32,
    pub packet_count: u32,
    pub vert_count: u32,
    pub packed_bytes: u32,
    /// Stream offsets in the blob (`None`: absent).
    pub streams: [Option<u32>; 13],
    pub centre: [f32; 3],
    pub extents: [f32; 3],
    pub ranges: Vec<RigidRange>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurfsRec {
    pub at: usize,
    pub hash: u64,
    pub flags: u32,
    pub surfaces: Vec<SurfaceRec>,
    /// The blob in the zone: (offset, length); (0, 0) when streamed.
    pub blob: (usize, usize),
    /// A streamed blob (a map model's mesh): the package key (header
    /// +0x10) and its length.
    pub streamed: Option<(u64, usize)>,
    /// Where the next asset starts.
    pub end: usize,
}

pub mod stream {
    pub const POSITION: usize = 0;
    pub const UV: usize = 1;
    pub const TANGENT_FRAME: usize = 2;
    pub const PACKET_VERTS: usize = 3;
    pub const PACKETS: usize = 4;
    pub const PACKED_INDICES: usize = 5;
    pub const UV2: usize = 8;
    pub const COLOUR: usize = 9;
}

/// Read an `xmodelsurfs` asset whose header is at `at`.
pub fn parse(zone: &[u8], at: usize) -> Result<SurfsRec, SurfsError> {
    let b = Bytes(zone);
    let hash = b.u64(at)?;
    if b.ptr(at + 8)? != Ptr::Follow && b.ptr(at + 8)? != Ptr::Insert {
        return Err(SurfsError::Layout("surfaces do not follow the header"));
    }
    let count = b.u32(at + 0x28)? as usize;
    if count == 0 || count > 1024 {
        return Err(SurfsError::Layout("surface count"));
    }
    let flags = b.u32(at + 0x2c)?;
    let mut surfaces = Vec::with_capacity(count);
    let mut range_counts = Vec::with_capacity(count);
    let mut blob_follows = Vec::with_capacity(count);
    for i in 0..count {
        let s = at + HEADER_LEN + SURFACE_LEN * i;
        let mut streams = [None; 13];
        for (k, st) in streams.iter_mut().enumerate() {
            let v = b.i32(s + 0x40 + 4 * k)?;
            *st = (v >= 0).then_some(v as u32);
        }
        let f = |o: usize| b.f32(s + o);
        surfaces.push(SurfaceRec {
            flags: b.u16(s)?,
            tri_count: b.u32(s + 0x1c)?,
            packet_count: u32::from(b.u16(s + 4)?),
            vert_count: b.u32(s + 0x18)?,
            packed_bytes: b.u32(s + 0x0c)?,
            streams,
            centre: [f(0xb0)?, f(0xb4)?, f(0xb8)?],
            extents: [f(0xbc)?, f(0xc0)?, f(0xc4)?],
            ranges: Vec::new(),
        });
        blob_follows.push(b.ptr(s + 0x78)?.follows());
        range_counts.push((b.ptr(s + 0x80)?.follows(), b.ptr(s + 0xc8)?.follows()));
    }
    let mut cur = at + HEADER_LEN + SURFACE_LEN * count;
    let mut blob = None;
    let mut streamed = None;
    for i in 0..count {
        if blob_follows[i] {
            let len = b.u32(cur + 8)? as usize;
            match b.ptr(cur)? {
                // A map model's mesh: streamed from the packages.
                Ptr::Null if blob.is_none() && streamed.is_none() => {
                    streamed = Some((b.u64(at + 0x10)?, len));
                    cur += 16;
                }
                p if p.follows() => {
                    cur += 16;
                    b.get(cur, len)?;
                    if blob.is_none() {
                        blob = Some((cur, len));
                    }
                    cur += len;
                }
                // Elsewhere (another zone's): its length known, no bytes.
                _ if blob.is_none() && streamed.is_none() => {
                    streamed = Some((b.u64(at + 0x10)?, len));
                    cur += 16;
                }
                _ => return Err(SurfsError::Layout("blob struct")),
            }
        }
        let (ranges_follow, tail_follows) = range_counts[i];
        if ranges_follow {
            // The ranges cover the surface's packets: read until they do.
            let s = &mut surfaces[i];
            let mut packets = 0u32;
            while packets < s.packet_count {
                let r = RigidRange {
                    bone: b.u16(cur)?,
                    vert_count: b.u16(cur + 2)?,
                    first_vert: b.u16(cur + 4)?,
                    tri_count: b.u16(cur + 6)?,
                    first_packet: b.u16(cur + 8)?,
                    packet_count: b.u16(cur + 10)?,
                };
                if u32::from(r.first_packet) != packets || r.packet_count == 0 {
                    return Err(SurfsError::Layout("rigid ranges do not run over the packets"));
                }
                packets += u32::from(r.packet_count);
                s.ranges.push(r);
                cur += RANGE_LEN;
            }
        }
        if tail_follows {
            cur += 12;
        }
    }
    if blob.is_none() && streamed.is_none() {
        return Err(SurfsError::Layout("no blob"));
    }
    Ok(SurfsRec { at, hash, flags, surfaces, blob: blob.unwrap_or((0, 0)), streamed, end: cur })
}

/// A surface's decoded geometry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// The packed tangent frame words, as stored.
    pub tangent_frames: Vec<u32>,
    pub colours: Vec<[u8; 4]>,
    pub indices: Vec<u32>,
}

/// A half's value.
#[must_use]
pub fn f16(h: u16) -> f32 {
    let s = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let e = i32::from((h >> 10) & 0x1f);
    let m = f32::from(h & 0x3ff);
    match e {
        0 => s * m * (1.0 / 16_777_216.0),
        31 => s * f32::INFINITY,
        e => s * (1.0 + m / 1024.0) * 2f32.powi(e - 15),
    }
}

/// One packed position: the extents are half sizes, a 21-bit fraction
/// runs from the centre minus the largest of them to the centre plus it
/// (a 96-unit door model measures 96, not 48).
#[must_use]
pub fn position(raw: u64, centre: [f32; 3], extents: [f32; 3]) -> [f32; 3] {
    let scale = extents[0].max(extents[1]).max(extents[2]);
    let mut p = [0.0; 3];
    for (k, c) in p.iter_mut().enumerate() {
        let v = ((raw >> (21 * k)) & 0x1f_ffff) as f32 / 2_097_151.0;
        *c = centre[k] + (2.0 * v - 1.0) * scale;
    }
    p
}

/// Decode a surface of `rec` from the zone.
pub fn geometry(zone: &[u8], rec: &SurfsRec, surface: usize) -> Result<Geometry, SurfsError> {
    let (blob_at, blob_len) = rec.blob;
    geometry_in(zone.get(blob_at..blob_at + blob_len).ok_or(SurfsError::Layout("blob"))?, rec, surface)
}

/// Decode a surface of `rec` from its blob (the zone's, or a streamed
/// one fetched by [`SurfsRec::streamed`]).
pub fn geometry_in(blob: &[u8], rec: &SurfsRec, surface: usize) -> Result<Geometry, SurfsError> {
    let s = rec.surfaces.get(surface).ok_or(SurfsError::Layout("no such surface"))?;
    let blob = Bytes(blob);
    let n = s.vert_count as usize;
    let at = |k: usize| s.streams[k].map(|v| v as usize);
    let pos = at(stream::POSITION).ok_or(SurfsError::Layout("no positions"))?;
    let mut g = Geometry::default();
    for v in 0..n {
        g.positions.push(position(blob.u64(pos + 8 * v)?, s.centre, s.extents));
    }
    if let Some(o) = at(stream::UV) {
        for v in 0..n {
            g.uvs.push([f16(blob.u16(o + 4 * v)?), f16(blob.u16(o + 4 * v + 2)?)]);
        }
    }
    if let Some(o) = at(stream::TANGENT_FRAME) {
        for v in 0..n {
            g.tangent_frames.push(blob.u32(o + 4 * v)?);
        }
    }
    if let Some(o) = at(stream::COLOUR) {
        for v in 0..n {
            let c = blob.get(o + 4 * v, 4)?;
            g.colours.push([c[0], c[1], c[2], c[3]]);
        }
    }
    let (lists, table, packed) = match (at(stream::PACKET_VERTS), at(stream::PACKETS), at(stream::PACKED_INDICES)) {
        (Some(a), Some(b), Some(c)) => (a, b, c),
        _ => return Err(SurfsError::Layout("no packed triangles")),
    };
    for p in 0..s.packet_count as usize {
        let e = table + PACKET_LEN * p;
        let list = blob.u32(e + 0x1c)? as usize;
        let verts = usize::from(blob.u8(e + 0x22)?);
        let tris = usize::from(blob.u8(e + 0x23)?);
        let data = packed + blob.u32(e + 0x24)? as usize;
        let bits = if verts <= 1 { 1 } else { usize::BITS - (verts - 1).leading_zeros() } as usize;
        for k in 0..tris * 3 {
            let bit = k * bits;
            let mut v = 0usize;
            for j in 0..bits {
                let b = bit + j;
                if blob.u8(data + b / 8)? >> (b % 8) & 1 == 1 {
                    v |= 1 << j;
                }
            }
            if v >= verts {
                return Err(SurfsError::Layout("packed index past its packet"));
            }
            let vert = blob.u16(lists + 2 * (list + v))? as u32;
            if vert as usize >= n {
                return Err(SurfsError::Layout("index past the vertices"));
            }
            g.indices.push(vert);
        }
    }
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves() {
        assert_eq!(f16(0x3c00), 1.0);
        assert_eq!(f16(0xb800), -0.5);
        assert_eq!(f16(0), 0.0);
    }

    #[test]
    fn positions_about_the_centre() {
        let c = [2.0, 0.0, -2.0];
        let e = [2.0, 1.0, 0.5];
        // x at the top of its range, y in the middle, z at the bottom.
        let raw = 0x1f_ffffu64 | (0x10_0000 << 21);
        let p = position(raw, c, e);
        assert!((p[0] - 4.0).abs() < 1e-5);
        assert!(p[1].abs() < 1e-5);
        assert!((p[2] - -4.0).abs() < 1e-5);
    }

    /// A one-surface mesh: 3 vertices, one packet of one triangle with
    /// 2-bit indices (1, 2, 0) over a vertex list (0, 1, 2).
    #[test]
    fn packed_triangles() {
        let mut blob = Vec::new();
        let pos = blob.len();
        for v in [0u64, 0x1f_ffff, 0x1f_ffff << 21] {
            blob.extend(v.to_le_bytes());
        }
        let lists = blob.len();
        for v in [0u16, 1, 2, 0] {
            blob.extend(v.to_le_bytes());
        }
        let table = blob.len();
        let mut e = [0u8; PACKET_LEN];
        e[0x22] = 3;
        e[0x23] = 1;
        blob.extend(e);
        let packed = blob.len();
        // indices 1, 2, 0 at 2 bits: 01 | 10 << 2 | 00 << 4 = 0b1001.
        blob.extend([0b1001u8, 0, 0, 0]);
        let mut z = vec![0u8; HEADER_LEN + SURFACE_LEN];
        z[8..16].copy_from_slice(&crate::ptr::FOLLOW.to_le_bytes());
        z[0x28..0x2c].copy_from_slice(&1u32.to_le_bytes());
        let s = HEADER_LEN;
        z[s + 4..s + 6].copy_from_slice(&1u16.to_le_bytes());
        z[s + 0x18..s + 0x1c].copy_from_slice(&3u32.to_le_bytes());
        z[s + 0x1c..s + 0x20].copy_from_slice(&1u32.to_le_bytes());
        let offs: [i32; 13] = [pos as i32, -1, -1, lists as i32, table as i32, packed as i32, -1, -1, -1, -1, -1, -1, -1];
        for (k, o) in offs.iter().enumerate() {
            z[s + 0x40 + 4 * k..s + 0x44 + 4 * k].copy_from_slice(&o.to_le_bytes());
        }
        z[s + 0x78..s + 0x80].copy_from_slice(&crate::ptr::FOLLOW.to_le_bytes());
        z[s + 0xbc..s + 0xc0].copy_from_slice(&1.0f32.to_le_bytes());
        z.extend(crate::ptr::FOLLOW.to_le_bytes());
        z.extend((blob.len() as u32).to_le_bytes());
        z.extend(2u32.to_le_bytes());
        z.extend(&blob);
        let rec = parse(&z, 0).unwrap();
        assert_eq!(rec.end, z.len());
        let g = geometry(&z, &rec, 0).unwrap();
        assert_eq!(g.indices, vec![1, 2, 0]);
        assert_eq!(g.positions.len(), 3);
        assert!((g.positions[1][0] - 1.0).abs() < 1e-5);
    }
}
