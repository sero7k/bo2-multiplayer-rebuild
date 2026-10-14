//! Mesh geometry: an `XModelMesh`'s shared vertex/index data (in the zone
//! or its `.xpak` entry) decoded to positions, normals, UVs, triangle
//! lists per surface and skin weights, and written as `.obj`.
//!
//! Layout (read on retail `core_common`'s `wpn_t7_arak_*` meshes, the
//! sections at the `XSurfaceShared` offsets; x64-zt names the fields):
//!
//! ```text
//! indices      at indicesOffset (0): u16 per index, each surface's triangles from
//!              baseIndexOffset (in indices), local to the surface (add baseVertOffset)
//! positions    at posOffset: vec3 f32 per vertex (16-aligned section)
//! vertices     at vertsOffset: 16 per vertex: u8 color[4]; f16 uv[2];
//!              u32 normal, u32 tangent (10:10:10:2, unsigned: v / 1023 * 2 - 1)
//! skin weights at skinWeightsOffset: 12 per skinned vertex (the first
//!              skinVertCount vertices, the skinned surfaces first):
//!              u8 weight[4] (/ 255); u16 bone[4]
//! ```
//!
//! Every surface also lists its vertices in runs on one bone (rigid vertex
//! lists, `SurfaceRec::rigid`: `boneOffset` is the bone index, read on
//! `wpn_t7_arak_view`: the lists of its skinned surfaces name the bones
//! their weights do); unskinned surfaces (after the skinned ones, no
//! weights) take their bone from these.

use crate::walk::capture::MeshRec;

#[derive(Clone, Debug, Default)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Per vertex: (bone, weight) pairs.
    pub weights: Vec<Vec<(u16, f32)>>,
    pub surfaces: Vec<SurfaceGeometry>,
}

#[derive(Clone, Debug, Default)]
pub struct SurfaceGeometry {
    pub name: String,
    /// Triangle list, mesh-wide vertex indices.
    pub indices: Vec<u32>,
}

#[derive(Debug)]
pub enum MeshError {
    Range(&'static str, usize),
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Range(what, at) => write!(f, "mesh data: {what} past the end at {at:#x}"),
        }
    }
}

impl std::error::Error for MeshError {}

fn f16(h: u16) -> f32 {
    let (sign, exp, frac) = ((h >> 15) as u32, ((h >> 10) & 0x1f) as i32, (h & 0x3ff) as f32);
    let v = match exp {
        0 => frac / 1024.0 * 2f32.powi(-14),
        31 => f32::INFINITY,
        e => (1.0 + frac / 1024.0) * 2f32.powi(e - 15),
    };
    if sign == 1 { -v } else { v }
}

/// A 10:10:10:2 unit vector (unsigned fields).
pub fn unpack_normal(v: u32) -> [f32; 3] {
    let c = |s: u32| ((v >> s) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0;
    [c(0), c(10), c(20)]
}

/// Bytes per vertex in the vertex section, per skin weight entry.
pub const VERTEX_LEN: usize = 16;
pub const WEIGHT_LEN: usize = 12;

pub fn decode(me: &MeshRec, data: &[u8]) -> Result<Geometry, MeshError> {
    let n = me.vert_count as usize;
    let [pos, verts, idx, skin] = me.offsets.map(|o| o as usize);
    let get = |what, at: usize, len: usize| data.get(at..at + len).ok_or(MeshError::Range(what, at));
    let f32_at = |b: &[u8], i: usize| f32::from_le_bytes(b[i..i + 4].try_into().unwrap_or([0; 4]));
    let u16_at = |b: &[u8], i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let u32_at = |b: &[u8], i: usize| u32::from_le_bytes(b[i..i + 4].try_into().unwrap_or([0; 4]));
    let p = get("positions", pos, 12 * n)?;
    let v = get("vertices", verts, VERTEX_LEN * n)?;
    let ix = get("indices", idx, 2 * me.index_count as usize)?;
    let mut g = Geometry::default();
    for i in 0..n {
        g.positions.push([f32_at(p, 12 * i), f32_at(p, 12 * i + 4), f32_at(p, 12 * i + 8)]);
        let a = 16 * i;
        g.uvs.push([f16(u16_at(v, a + 4)), f16(u16_at(v, a + 6))]);
        g.normals.push(unpack_normal(u32_at(v, a + 8)));
    }
    let skinned = (me.skin_vert_count as usize).min(n);
    g.weights = vec![Vec::new(); n];
    if skinned > 0 {
        let w = get("skin weights", skin, WEIGHT_LEN * skinned)?;
        for (i, out) in g.weights.iter_mut().enumerate().take(skinned) {
            let e = WEIGHT_LEN * i;
            for k in 0..4 {
                let wt = w[e + k];
                if wt != 0 {
                    out.push((u16_at(w, e + 4 + 2 * k), wt as f32 / 255.0));
                }
            }
        }
    }
    for sf in &me.surfaces {
        let (base_v, base_i, tris) = (sf.base_vert as usize, sf.base_index as usize, sf.tri_count as usize);
        if 2 * (base_i + 3 * tris) > ix.len() || base_v + sf.vert_count as usize > n {
            return Err(MeshError::Range("surface", base_i));
        }
        let indices = (0..3 * tris).map(|k| (u16_at(ix, 2 * (base_i + k)) as usize + base_v) as u32).collect();
        // Rigid lists: consecutive runs of the surface's vertices on one bone.
        let mut at = base_v;
        for r in &sf.rigid {
            for wts in g.weights.iter_mut().skip(at).take(r[1] as usize) {
                if wts.is_empty() {
                    wts.push((r[0], 1.0));
                }
            }
            at += r[1] as usize;
        }
        g.surfaces.push(SurfaceGeometry { name: sf.name.clone(), indices });
    }
    Ok(g)
}

/// The geometry as Wavefront `.obj` (one group per surface, `usemtl` the
/// LOD's material for that surface).
pub fn to_obj(g: &Geometry, materials: &[String]) -> String {
    use std::fmt::Write;
    let mut o = String::new();
    for p in &g.positions {
        let _ = writeln!(o, "v {} {} {}", p[0], p[1], p[2]);
    }
    for t in &g.uvs {
        let _ = writeln!(o, "vt {} {}", t[0], 1.0 - t[1]);
    }
    for n in &g.normals {
        let _ = writeln!(o, "vn {} {} {}", n[0], n[1], n[2]);
    }
    let (has_t, has_n) = (g.uvs.len() == g.positions.len(), g.normals.len() == g.positions.len());
    for (i, s) in g.surfaces.iter().enumerate() {
        let _ = writeln!(o, "g {}", if s.name.is_empty() { format!("surface{i}") } else { s.name.clone() });
        if let Some(m) = materials.get(i) {
            let _ = writeln!(o, "usemtl {m}");
        }
        for t in s.indices.chunks_exact(3) {
            let f: Vec<String> = t
                .iter()
                .map(|&v| {
                    let v = v + 1;
                    match (has_t, has_n) {
                        (true, true) => format!("{v}/{v}/{v}"),
                        (true, false) => format!("{v}/{v}"),
                        (false, true) => format!("{v}//{v}"),
                        _ => v.to_string(),
                    }
                })
                .collect();
            let _ = writeln!(o, "f {}", f.join(" "));
        }
    }
    o
}
