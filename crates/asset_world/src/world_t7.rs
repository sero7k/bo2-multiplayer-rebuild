//! bo2mp: Black Ops III (T7) maps' collision. Not part of upstream IW4L.
//!
//! Black Ops III keeps Black Ops II's collision shapes: brushes (a box and
//! extra side planes) under a BSP whose leaves list them, and a triangle
//! mesh in partitions (at most 255 triangles) under AABB trees. Its mesh
//! indexes vertices with 32-bit numbers where Black Ops II's 16-bit ones
//! are relative to a partition's vertex segment, so each partition's
//! vertices are copied out to a segment of their own (`first_vert_segment`)
//! and its triangles re-indexed against it; triangles keep their order, so
//! the trees' partitions and the walkable-edge bits stay as they are.
//!
//! Black Ops III's leaves name AABB trees by 32-bit index; here every tree
//! root is gathered under one root of its own (copies of the roots, made
//! contiguous) and no leaf names one, so the world trace reaches the mesh
//! through that one tree (`aabb_roots`). Extra triangles (placed models'
//! collision, which Black Ops III keeps in Havok shapes) join the same way
//! as partitions of their own.
//!
//! Surface flags are read as Black Ops II's (`t5_surface_flags`); contents
//! as they are (CONTENTS_SOLID is 1 in both).

use crate::clip_collision::{ClipBrush, ClipBspLeaf, ClipBspNode, ClipCmodel, ClipCollision, ClipMapMaterial, t5_surface_flags};
use crate::world_iw7::SoupTri;

/// Children per added tree node.
const FANOUT: usize = 8;
/// Triangles per partition of the extra triangles.
const PART_TRIS: usize = 32;

/// A collision brush: planes (normal, distance; the six axial first) and
/// each plane's surface flags as the zone has them.
#[derive(Clone, Debug, Default)]
pub struct T7Brush {
    pub contents: u32,
    pub planes: Vec<[f32; 4]>,
    pub plane_surface_flags: Vec<u32>,
}

/// An AABB tree node (`u`: first child, or a leaf's partition).
#[derive(Clone, Copy, Debug, Default)]
pub struct T7AabbNode {
    pub origin: [f32; 3],
    pub half_size: [f32; 3],
    pub material: u16,
    pub child_count: u16,
    pub u: u32,
}

/// A sub-model (`*<n>`; 0 is the world): bounds and its brushes.
#[derive(Clone, Debug, Default)]
pub struct T7Cmodel {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub radius: f32,
    pub brushes: Vec<u16>,
}

/// A Black Ops III clip map in plain terms.
#[derive(Clone, Debug, Default)]
pub struct T7ClipInput {
    /// (name, surface flags, content flags).
    pub materials: Vec<(String, u32, u32)>,
    pub brushes: Vec<T7Brush>,
    /// (plane, children: negative is leaf `-1 - n`).
    pub nodes: Vec<([f32; 4], [i32; 2])>,
    /// Each leaf's brushes.
    pub leaf_brushes: Vec<Vec<u16>>,
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    /// One bit per triangle edge (`3 * tri + k`), as the zone has them.
    pub walkable: Vec<u8>,
    /// (triangle count, first triangle).
    pub partitions: Vec<(u8, u32)>,
    pub aabb_trees: Vec<T7AabbNode>,
    pub cmodels: Vec<T7Cmodel>,
}

fn bounds_of(pts: impl Iterator<Item = [f32; 3]>) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in pts {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    (lo, hi)
}

fn node(lo: [f32; 3], hi: [f32; 3], child_count: u16, u: i32) -> clipmap_iw4::ClipAabbNode {
    clipmap_iw4::ClipAabbNode {
        origin: std::array::from_fn(|k| (lo[k] + hi[k]) * 0.5),
        half_size: std::array::from_fn(|k| (hi[k] - lo[k]) * 0.5 + 0.5),
        material_index: 0,
        child_count,
        u,
    }
}

/// Brushes per k-d leaf (fewer go on splitting) and the deepest split.
const KD_LEAF: usize = 12;
const KD_DEPTH: usize = 28;

/// A brush's box: its six axial planes (+x, -x, +y, -y, +z, -z first).
fn brush_box(b: &T7Brush) -> ([f32; 3], [f32; 3]) {
    let p = |i: usize| b.planes.get(i).map_or(0.0, |p| p[3]);
    ([-p(1), -p(3), -p(5)], [p(0), p(2), p(4)])
}

/// A k-d tree over `list`'s brush boxes, as BSP nodes (an axial plane;
/// child 0 the side past it) and leaves; the child index of its root.
fn kd_build(list: Vec<u16>, bounds: &[([f32; 3], [f32; 3])], depth: usize, out: &mut ClipCollision) -> i32 {
    let leaf = |list: &[u16], out: &mut ClipCollision| -> i32 {
        let first = out.leafbrushes.len() as u32;
        out.leafbrushes.extend_from_slice(list);
        out.leaves.push(ClipBspLeaf {
            first_brush: first,
            num_brushes: list.len().min(usize::from(u16::MAX)) as u16,
            first_coll_aabb_index: 0,
            coll_aabb_count: 0,
        });
        -(out.leaves.len() as i32)
    };
    if list.len() <= KD_LEAF || depth >= KD_DEPTH {
        if depth == 0 {
            // (A tree needs a node: one every trace passes.)
            let at = out.nodes.len();
            out.nodes.push(ClipBspNode { plane: [0.0, 0.0, 1.0, -1.0e9], children: [-1, -1] });
            let l = leaf(&list, out);
            out.nodes[at].children = [l, l];
            return at as i32;
        }
        return leaf(&list, out);
    }
    let (lo, hi) = bounds_of(list.iter().flat_map(|&b| [bounds[usize::from(b)].0, bounds[usize::from(b)].1]));
    let axis = (0..3).max_by(|&a, &b| (hi[a] - lo[a]).total_cmp(&(hi[b] - lo[b]))).unwrap_or(0);
    let mut mids: Vec<f32> = list.iter().map(|&b| (bounds[usize::from(b)].0[axis] + bounds[usize::from(b)].1[axis]) * 0.5).collect();
    mids.sort_unstable_by(f32::total_cmp);
    let split = mids[mids.len() / 2];
    let front: Vec<u16> = list.iter().copied().filter(|&b| bounds[usize::from(b)].1[axis] > split).collect();
    let back: Vec<u16> = list.iter().copied().filter(|&b| bounds[usize::from(b)].0[axis] < split).collect();
    if front.len() == list.len() || back.len() == list.len() || !split.is_finite() {
        if depth == 0 {
            let at = out.nodes.len();
            out.nodes.push(ClipBspNode { plane: [0.0, 0.0, 1.0, -1.0e9], children: [-1, -1] });
            let l = leaf(&list, out);
            out.nodes[at].children = [l, l];
            return at as i32;
        }
        return leaf(&list, out);
    }
    let at = out.nodes.len();
    let mut normal = [0.0; 3];
    normal[axis] = 1.0;
    out.nodes.push(ClipBspNode { plane: [normal[0], normal[1], normal[2], split], children: [0, 0] });
    let f = kd_build(front, bounds, depth + 1, out);
    let b = kd_build(back, bounds, depth + 1, out);
    out.nodes[at].children = [f, b];
    at as i32
}

/// What `build_t7_clip_collision` made, for the world report.
#[derive(Clone, Copy, Debug, Default)]
pub struct T7ClipStats {
    pub brushes: usize,
    pub leaves: usize,
    pub tris: usize,
    pub extra_tris: usize,
    pub roots: usize,
    /// Partitions whose triangles name vertices out of range (left out).
    pub bad_partitions: usize,
}

/// The collision of a Black Ops III clip map, with `extra` triangles (the
/// placed models' stand-in collision) added to the mesh.
pub fn build_t7_clip_collision(clip: &T7ClipInput, extra: &[SoupTri]) -> (ClipCollision, T7ClipStats) {
    let mut stats = T7ClipStats::default();
    let mut out = ClipCollision::default();
    for (name, sflags, cflags) in &clip.materials {
        out.materials.push(ClipMapMaterial { name: name.clone(), surface_flags: t5_surface_flags(*sflags), content_flags: *cflags });
    }
    let extra_material = out.materials.len() as u16;
    out.materials.push(ClipMapMaterial { name: "t7_model_soup".to_owned(), surface_flags: 0, content_flags: 1 });
    for b in &clip.brushes {
        out.brushes.push(ClipBrush {
            planes: b.planes.clone(),
            contents: b.contents,
            plane_surface_flags: b.plane_surface_flags.iter().map(|&f| t5_surface_flags(f)).collect(),
            glass_encoded: 0,
        });
    }
    stats.brushes = out.brushes.len();
    // The world's brushes (every brush a leaf lists, else the world
    // model's) under a k-d tree of their boxes: Black Ops III's own BSP is
    // a handful of nodes over leaves of thousands of brushes (its leaves'
    // brush trees are not Black Ops II's flat lists).
    let mut world: Vec<u16> = clip.leaf_brushes.iter().flatten().copied().filter(|&b| usize::from(b) < out.brushes.len()).collect();
    world.sort_unstable();
    world.dedup();
    if world.is_empty() {
        world = clip.cmodels.first().map(|m| m.brushes.clone()).unwrap_or_default();
        world.retain(|&b| usize::from(b) < out.brushes.len());
    }
    if !world.is_empty() {
        let bounds: Vec<([f32; 3], [f32; 3])> = clip.brushes.iter().map(brush_box).collect();
        kd_build(world, &bounds, 0, &mut out);
    }
    stats.leaves = out.leaves.len();
    for (i, m) in clip.cmodels.iter().enumerate() {
        let first = out.leafbrushes.len() as u32;
        if i > 0 {
            out.leafbrushes.extend(m.brushes.iter().copied().filter(|&b| usize::from(b) < out.brushes.len()));
        }
        out.cmodels.push(ClipCmodel {
            mins: m.mins,
            maxs: m.maxs,
            radius: m.radius,
            first_brush: first,
            num_brushes: (out.leafbrushes.len() as u32 - first).min(u32::from(u16::MAX)) as u16,
        });
    }

    // The mesh: each partition's vertices to a segment of their own.
    let mesh = std::sync::Arc::make_mut(&mut out.mesh);
    let mut part_ok = vec![true; clip.partitions.len()];
    let tri_total = clip.tris.len();
    mesh.tri_indices = vec![0; 3 * tri_total];
    mesh.tri_surface_flags = vec![0; tri_total];
    mesh.tri_content_flags = vec![0; tri_total];
    out.tri_material_index = vec![0; tri_total];
    for (p, &(count, first)) in clip.partitions.iter().enumerate() {
        let (first, count) = (first as usize, usize::from(count));
        if first + count > tri_total {
            part_ok[p] = false;
            stats.bad_partitions += 1;
            mesh.partitions.push(clipmap_iw4::ClipPartition::default());
            continue;
        }
        // Start each partition's vertices at a fresh segment boundary when
        // they would not fit in what is left of the current one.
        let mut local: Vec<u32> = Vec::new();
        for t in &clip.tris[first..first + count] {
            for &v in t {
                if !local.contains(&v) {
                    local.push(v);
                }
            }
        }
        if local.iter().any(|&v| v as usize >= clip.verts.len()) {
            part_ok[p] = false;
            stats.bad_partitions += 1;
            mesh.partitions.push(clipmap_iw4::ClipPartition { tri_count: 0, first_tri: first as i32, ..Default::default() });
            continue;
        }
        let seg_size = clipmap_iw4::VERTS_PER_SEGMENT;
        let mut start = mesh.verts.len();
        if start % seg_size + local.len() > seg_size {
            start = start.div_ceil(seg_size) * seg_size;
            mesh.verts.resize(start, [0.0; 3]);
        }
        let seg = start / seg_size;
        let base = seg * seg_size;
        mesh.verts.extend(local.iter().map(|&v| clip.verts[v as usize]));
        for (k, t) in clip.tris[first..first + count].iter().enumerate() {
            for (j, &v) in t.iter().enumerate() {
                let at = local.iter().position(|&x| x == v).unwrap_or(0);
                mesh.tri_indices[3 * (first + k) + j] = (start - base + at) as u16;
            }
        }
        mesh.partitions.push(clipmap_iw4::ClipPartition {
            tri_count: count as u8,
            first_tri: first as i32,
            first_vert_segment: seg.min(usize::from(u16::MAX)) as u16,
            border_count: 0,
            first_border: 0,
        });
    }
    mesh.tri_edge_is_walkable = clip.walkable.clone();
    mesh.tri_edge_is_walkable.resize((3 * tri_total).div_ceil(8), 0xff);
    // The zone's trees, then the flags each leaf node gives its triangles.
    mesh.aabb_trees = clip
        .aabb_trees
        .iter()
        .map(|n| clipmap_iw4::ClipAabbNode {
            origin: n.origin,
            half_size: n.half_size,
            material_index: n.material,
            child_count: n.child_count,
            u: n.u as i32,
        })
        .collect();
    for n in &mesh.aabb_trees {
        if n.child_count != 0 || n.u < 0 {
            continue;
        }
        let Some(&(count, first)) = clip.partitions.get(n.u as usize) else { continue };
        if !part_ok[n.u as usize] {
            continue;
        }
        let m = out.materials.get(usize::from(n.material_index));
        for t in first as usize..first as usize + usize::from(count) {
            mesh.tri_surface_flags[t] = m.map_or(0, |m| m.surface_flags);
            mesh.tri_content_flags[t] = m.map_or(1, |m| m.content_flags);
            out.tri_material_index[t] = n.material_index;
        }
    }
    stats.tris = tri_total;

    // The extra triangles: partitions of their own under one tree.
    let mut extra_roots: Vec<usize> = Vec::new();
    {
        let keep: Vec<&SoupTri> = extra
            .iter()
            .filter(|t| {
                let [a, b, c] = t.p;
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                n[0] * n[0] + n[1] * n[1] + n[2] * n[2] > 1e-6 && t.p.iter().flatten().all(|v| v.is_finite())
            })
            .collect();
        stats.extra_tris = keep.len();
        let seg_size = clipmap_iw4::VERTS_PER_SEGMENT;
        let mut level: Vec<clipmap_iw4::ClipAabbNode> = Vec::new();
        for chunk in keep.chunks(PART_TRIS) {
            let mut start = mesh.verts.len();
            if start % seg_size + 3 * chunk.len() > seg_size {
                start = start.div_ceil(seg_size) * seg_size;
                mesh.verts.resize(start, [0.0; 3]);
            }
            let seg = start / seg_size;
            let base = seg * seg_size;
            let first_tri = mesh.tri_indices.len() / 3;
            for t in chunk {
                for p in t.p {
                    mesh.tri_indices.push((mesh.verts.len() - base) as u16);
                    mesh.verts.push(p);
                }
                mesh.tri_surface_flags.push(t.surface_flags);
                mesh.tri_content_flags.push(t.contents);
                out.tri_material_index.push(extra_material);
            }
            let part = mesh.partitions.len();
            mesh.partitions.push(clipmap_iw4::ClipPartition {
                tri_count: chunk.len() as u8,
                first_tri: first_tri as i32,
                first_vert_segment: seg.min(usize::from(u16::MAX)) as u16,
                border_count: 0,
                first_border: 0,
            });
            let (lo, hi) = bounds_of(chunk.iter().flat_map(|t| t.p));
            level.push(node(lo, hi, 0, part as i32));
        }
        mesh.tri_edge_is_walkable.resize((mesh.tri_indices.len()).div_ceil(8), 0xff);
        // Laid out level by level, children contiguous.
        let mut levels = vec![level];
        while levels.last().is_some_and(|l| l.len() > 1) {
            let below = levels.last().unwrap();
            let mut up = Vec::new();
            for (g, group) in below.chunks(FANOUT).enumerate() {
                let (lo, hi) = bounds_of(group.iter().flat_map(|n| {
                    let a: [f32; 3] = std::array::from_fn(|k| n.origin[k] - n.half_size[k]);
                    let b: [f32; 3] = std::array::from_fn(|k| n.origin[k] + n.half_size[k]);
                    [a, b]
                }));
                up.push(node(lo, hi, group.len() as u16, (g * FANOUT) as i32));
            }
            levels.push(up);
        }
        if levels[0].is_empty() {
            levels.clear();
        }
        let base = mesh.aabb_trees.len();
        let mut offsets = vec![0usize; levels.len()];
        let mut at = base;
        for (d, l) in levels.iter().enumerate().rev() {
            offsets[d] = at;
            at += l.len();
        }
        for d in (0..levels.len()).rev() {
            for n in &levels[d] {
                let mut n = *n;
                if n.child_count != 0 {
                    n.u += offsets[d - 1] as i32;
                }
                mesh.aabb_trees.push(n);
            }
        }
        if let Some(top) = levels.last() {
            extra_roots.extend((0..top.len()).map(|i| offsets[levels.len() - 1] + i));
        }
    }

    // Every tree root (the zone's and the extra one), copied side by side
    // under one root.
    let mut roots: Vec<usize> = clipmap_iw4::aabb_forest_roots(&mesh.aabb_trees).into_iter().map(usize::from).collect();
    if mesh.aabb_trees.len() > usize::from(u16::MAX) {
        // (`aabb_forest_roots` names nodes by u16: find the roots here.)
        let mut is_child = vec![false; mesh.aabb_trees.len()];
        for n in &mesh.aabb_trees {
            if n.child_count != 0 && n.u >= 0 {
                for c in n.u as usize..(n.u as usize + usize::from(n.child_count)).min(is_child.len()) {
                    is_child[c] = true;
                }
            }
        }
        roots = (0..mesh.aabb_trees.len()).filter(|&i| !is_child[i]).collect();
    }
    for r in extra_roots {
        if !roots.contains(&r) {
            roots.push(r);
        }
    }
    stats.roots = roots.len();
    if roots.len() <= 1 {
        mesh.aabb_roots = roots.iter().map(|&r| r.min(usize::from(u16::MAX)) as u16).collect();
        return (out, stats);
    }
    // The upper tree goes in front (its root at 0, so u16 names it): the
    // roots' copies at its bottom, the zone's nodes shifted after it.
    let mut levels: Vec<Vec<clipmap_iw4::ClipAabbNode>> = vec![roots.iter().map(|&r| mesh.aabb_trees[r]).collect()];
    while levels.last().is_some_and(|l| l.len() > 1) {
        let below = levels.last().unwrap();
        let mut up = Vec::new();
        for (g, group) in below.chunks(FANOUT).enumerate() {
            let (lo, hi) = bounds_of(group.iter().flat_map(|n| {
                let a: [f32; 3] = std::array::from_fn(|k| n.origin[k] - n.half_size[k]);
                let b: [f32; 3] = std::array::from_fn(|k| n.origin[k] + n.half_size[k]);
                [a, b]
            }));
            // (`u`: the first child's place in the level below, for now.)
            up.push(node(lo, hi, group.len() as u16, (g * FANOUT) as i32));
        }
        levels.push(up);
    }
    let shift: usize = levels.iter().map(Vec::len).sum();
    let mut offsets = vec![0usize; levels.len()];
    let mut at = 0usize;
    for (d, l) in levels.iter().enumerate().rev() {
        offsets[d] = at;
        at += l.len();
    }
    let mut upper = Vec::with_capacity(shift);
    for d in (0..levels.len()).rev() {
        for n in &levels[d] {
            let mut n = *n;
            if d > 0 {
                n.u += offsets[d - 1] as i32;
            } else if n.child_count != 0 {
                n.u += shift as i32;
            }
            upper.push(n);
        }
    }
    for n in &mut mesh.aabb_trees {
        if n.child_count != 0 {
            n.u += shift as i32;
        }
    }
    upper.append(&mut mesh.aabb_trees);
    mesh.aabb_trees = upper;
    mesh.aabb_roots = vec![0];
    (out, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> T7ClipInput {
        // A quad floor at z = 0, as two partitions under one root (wound
        // clockwise seen from above: the side collision faces).
        let verts = vec![[0.0, 0.0, 0.0], [128.0, 0.0, 0.0], [128.0, 128.0, 0.0], [0.0, 128.0, 0.0]];
        let tris = vec![[0, 2, 1], [0, 3, 2]];
        let leaf = |p: u32, lo: [f32; 3], hi: [f32; 3]| T7AabbNode {
            origin: std::array::from_fn(|k| (lo[k] + hi[k]) / 2.0),
            half_size: std::array::from_fn(|k| (hi[k] - lo[k]) / 2.0 + 1.0),
            material: 0,
            child_count: 0,
            u: p,
        };
        T7ClipInput {
            materials: vec![("floor".into(), 0, 1)],
            verts,
            tris,
            walkable: vec![0xff],
            partitions: vec![(1, 0), (1, 1)],
            aabb_trees: vec![
                T7AabbNode { origin: [64.0, 64.0, 0.0], half_size: [65.0, 65.0, 1.0], material: 0, child_count: 2, u: 1 },
                leaf(0, [0.0; 3], [128.0, 128.0, 0.0]),
                leaf(1, [0.0; 3], [128.0, 128.0, 0.0]),
            ],
            cmodels: vec![T7Cmodel { mins: [0.0; 3], maxs: [128.0, 128.0, 0.0], radius: 91.0, brushes: Vec::new() }],
            ..Default::default()
        }
    }

    fn boxed(mins: [f32; 3], maxs: [f32; 3]) -> T7Brush {
        T7Brush {
            contents: 1,
            planes: vec![
                [1.0, 0.0, 0.0, maxs[0]],
                [-1.0, 0.0, 0.0, -mins[0]],
                [0.0, 1.0, 0.0, maxs[1]],
                [0.0, -1.0, 0.0, -mins[1]],
                [0.0, 0.0, 1.0, maxs[2]],
                [0.0, 0.0, -1.0, -mins[2]],
            ],
            plane_surface_flags: vec![0; 6],
        }
    }

    #[test]
    fn brushes_split_into_a_tree_still_stop_a_drop() {
        // A row of 64 floor tiles; the brush model's box is not the world's.
        let mut input = T7ClipInput::default();
        for i in 0..64 {
            let x = i as f32 * 128.0;
            input.brushes.push(boxed([x, 0.0, -16.0], [x + 128.0, 128.0, 0.0]));
        }
        input.brushes.push(boxed([0.0, 0.0, 200.0], [64.0, 64.0, 264.0]));
        input.leaf_brushes = vec![(0..64).collect()];
        input.cmodels = vec![T7Cmodel::default(), T7Cmodel { mins: [0.0, 0.0, 200.0], maxs: [64.0, 64.0, 264.0], radius: 50.0, brushes: vec![64] }];
        let (clip, stats) = build_t7_clip_collision(&input, &[]);
        assert!(clip.nodes.len() > 1 && stats.leaves > 1, "{} nodes {} leaves", clip.nodes.len(), stats.leaves);
        assert!(clip.leaves.iter().all(|l| usize::from(l.num_brushes) <= KD_LEAF + 4));
        let spots: Vec<[f32; 3]> = (0..64).map(|i| [i as f32 * 128.0 + 60.0, 60.0, 40.0]).collect();
        let census = crate::world_iw3::spawn_ground_census(&clip, &spots);
        assert_eq!(census.on_ground, 64, "{census}");
        // The brush model's box is reached through its sub-model only.
        assert_eq!(clip.cmodels[1].num_brushes, 1);
        assert!(!clip.leafbrushes[..clip.leaves.iter().map(|l| l.first_brush as usize + usize::from(l.num_brushes)).max().unwrap_or(0)].contains(&64));
    }

    #[test]
    fn a_floor_stops_a_drop_and_models_add_partitions() {
        let model = vec![SoupTri { p: [[500.0, 0.0, 0.0], [500.0, 100.0, 0.0], [600.0, 0.0, 0.0]], surface_flags: 0, contents: 1 }];
        let (clip, stats) = build_t7_clip_collision(&floor(), &model);
        assert_eq!(stats.tris, 2);
        assert_eq!(stats.extra_tris, 1);
        assert_eq!(clip.mesh.tri_count(), 3);
        assert_eq!(clip.mesh.aabb_roots.len(), 1);
        let census = crate::world_iw3::spawn_ground_census(&clip, &[[10.0, 10.0, 40.0], [520.0, 10.0, 40.0], [5000.0, 0.0, 40.0]]);
        assert_eq!(census.on_ground, 2, "{census}");
    }
}
