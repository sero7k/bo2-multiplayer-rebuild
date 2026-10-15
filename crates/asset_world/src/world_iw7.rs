//! bo2mp: Infinite Warfare (IW7) maps' collision. Not part of upstream
//! IW4L.
//!
//! IW7 collides through Havok shapes the engine does not read; the map's
//! drawn triangles (its world surfaces and its placed models) stand in for
//! them. They become one collision mesh: an AABB tree split top down at
//! the triangles' median (up to eight children a node, children
//! contiguous, the root first) down to partitions of at most `PART_TRIS`,
//! each with its own vertices (the partition's segment,
//! `ClipPartition::first_vert_segment`, addresses them). The world has no
//! brushes; each brush model (`*1`, `*2`, ...: a door, a debris pile) gets
//! one box brush of its bounds, about its own origin, for the entity that
//! names it (a one-leaf tree with no brushes in it keeps them out of the
//! world's own trace, which reaches the mesh through its forest of trees).

use crate::clip_collision::{ClipBrush, ClipBspLeaf, ClipBspNode, ClipCmodel, ClipCollision, ClipMapMaterial};

/// Triangles per partition.
const PART_TRIS: usize = 32;
/// Children per tree node.
const FANOUT: usize = 8;

/// `CONTENTS_SOLID`.
pub const IW7_SOLID: u32 = 1;

/// A triangle of the collision soup: its corners, and its surface and
/// content flags.
#[derive(Clone, Copy, Debug)]
pub struct SoupTri {
    pub p: [[f32; 3]; 3],
    pub surface_flags: u32,
    pub contents: u32,
}

/// The brush models' bounds (`cmodel_t`: midpoint, half size, radius), by
/// their place in the table (`*1`, `*2`, ...; the first, the world, is
/// skipped): each gets a box brush of them.
pub type SoupCmodel = ([f32; 3], [f32; 3], f32);

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
        // A little slack so a trace grazing a face still enters its node.
        half_size: std::array::from_fn(|k| (hi[k] - lo[k]) * 0.5 + 0.5),
        material_index: 0,
        child_count,
        u,
    }
}

/// The collision of a triangle soup (`tris`) with the map's brush models
/// listed (index 0 is the world). Degenerate triangles are left out.
pub fn build_soup_clip_collision(tris: &[SoupTri], cmodels: &[SoupCmodel]) -> ClipCollision {
    let mut out = ClipCollision::default();
    out.materials.push(ClipMapMaterial { name: "iw7_soup".to_owned(), surface_flags: 0, content_flags: IW7_SOLID });
    let keep: Vec<usize> = (0..tris.len())
        .filter(|&i| {
            let [a, b, c] = tris[i].p;
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            n[0] * n[0] + n[1] * n[1] + n[2] * n[2] > 1e-6 && tris[i].p.iter().flatten().all(|v| v.is_finite())
        })
        .collect();
    let (lo, hi) = bounds_of(keep.iter().flat_map(|&i| tris[i].p));
    let world = if keep.is_empty() { ([0.0; 3], [0.0; 3]) } else { (lo, hi) };
    out.cmodels.push(ClipCmodel {
        mins: world.0,
        maxs: world.1,
        radius: (0..3).map(|k| (world.1[k] - world.0[k]).powi(2)).sum::<f32>().sqrt() * 0.5,
        first_brush: 0,
        num_brushes: 0,
    });
    for &(mid, half, radius) in cmodels.iter().skip(1) {
        let (mins, maxs): ([f32; 3], [f32; 3]) = (std::array::from_fn(|k| mid[k] - half[k]), std::array::from_fn(|k| mid[k] + half[k]));
        let solid = half.iter().all(|h| *h > 0.0 && h.is_finite());
        let first = out.leafbrushes.len() as u32;
        if solid && out.brushes.len() < usize::from(u16::MAX) {
            out.leafbrushes.push(out.brushes.len() as u16);
            out.brushes.push(ClipBrush {
                planes: vec![
                    [1.0, 0.0, 0.0, maxs[0]],
                    [-1.0, 0.0, 0.0, -mins[0]],
                    [0.0, 1.0, 0.0, maxs[1]],
                    [0.0, -1.0, 0.0, -mins[1]],
                    [0.0, 0.0, 1.0, maxs[2]],
                    [0.0, 0.0, -1.0, -mins[2]],
                ],
                contents: IW7_SOLID,
                plane_surface_flags: vec![0; 6],
                glass_encoded: 0,
            });
        }
        out.cmodels.push(ClipCmodel { mins, maxs, radius, first_brush: first, num_brushes: (out.leafbrushes.len() as u32 - first) as u16 });
    }
    if !out.brushes.is_empty() {
        // One node every trace passes to one empty leaf.
        out.nodes.push(ClipBspNode { plane: [0.0, 0.0, 1.0, -1.0e9], children: [-1, -1] });
        out.leaves.push(ClipBspLeaf { first_brush: 0, num_brushes: 0, first_coll_aabb_index: 0, coll_aabb_count: 0 });
    }
    if keep.is_empty() {
        return out;
    }
    // The tree, built top down: a node's triangles (by their middles) are
    // halved along the longest side of their middles' bounds three times
    // over, up to eight children, until a node holds at most `PART_TRIS`
    // (a partition). Nodes are laid out breadth first, root first, each
    // node's children contiguous. (A Morton order cut into runs of 32 left
    // runs that jump across the curve's seams, and nodes whose bounds
    // spanned the map: a trace visited hundreds of nodes.)
    let mids: Vec<[f32; 3]> = keep
        .iter()
        .map(|&i| {
            let [a, b, c] = tris[i].p;
            std::array::from_fn(|k| (a[k] + b[k] + c[k]) / 3.0)
        })
        .collect();
    let mut order: Vec<usize> = (0..keep.len()).collect();
    // Each node: its range of `order`, and (once its children are made)
    // its first child and child count; leaves get a partition.
    struct Build {
        range: (usize, usize),
        first_child: usize,
        children: usize,
        part: Option<usize>,
    }
    let mut nodes: Vec<Build> = vec![Build { range: (0, keep.len()), first_child: 0, children: 0, part: None }];
    let longest = |order: &[usize]| -> usize {
        let (l, h) = bounds_of(order.iter().map(|&o| mids[o]));
        (0..3).max_by(|&a, &b| (h[a] - l[a]).total_cmp(&(h[b] - l[b]))).unwrap_or(0)
    };
    let mut at = 0usize;
    while at < nodes.len() {
        let (s0, s1) = nodes[at].range;
        if s1 - s0 > PART_TRIS {
            let mut pieces = vec![(s0, s1)];
            for _ in 0..3 {
                let mut next = Vec::with_capacity(pieces.len() * 2);
                for &(a, b) in &pieces {
                    if b - a <= PART_TRIS {
                        next.push((a, b));
                        continue;
                    }
                    let axis = longest(&order[a..b]);
                    let half = (b - a) / 2;
                    order[a..b].select_nth_unstable_by(half, |&x, &y| mids[x][axis].total_cmp(&mids[y][axis]));
                    next.push((a, a + half));
                    next.push((a + half, b));
                }
                pieces = next;
            }
            debug_assert!(pieces.len() <= FANOUT);
            nodes[at].first_child = nodes.len();
            nodes[at].children = pieces.len();
            for range in pieces {
                nodes.push(Build { range, first_child: 0, children: 0, part: None });
            }
        }
        at += 1;
    }

    let mesh = std::sync::Arc::make_mut(&mut out.mesh);
    // Partitions (the leaves, in layout order), each with its own vertices
    // (shared within it).
    for n in nodes.iter_mut().filter(|n| n.children == 0) {
        let chunk = &order[n.range.0..n.range.1];
        let start = mesh.verts.len();
        let seg = start / clipmap_iw4::VERTS_PER_SEGMENT;
        let base = seg * clipmap_iw4::VERTS_PER_SEGMENT;
        let first_tri = mesh.tri_indices.len() / 3;
        let mut local: Vec<[u32; 3]> = Vec::new();
        for &o in chunk {
            let t = &tris[keep[o]];
            for p in t.p {
                let bits = p.map(f32::to_bits);
                let at = match local.iter().position(|q| *q == bits) {
                    Some(at) => at,
                    None => {
                        local.push(bits);
                        mesh.verts.push(p);
                        local.len() - 1
                    }
                };
                mesh.tri_indices.push((start - base + at) as u16);
            }
            mesh.tri_surface_flags.push(t.surface_flags);
            mesh.tri_content_flags.push(t.contents);
            out.tri_material_index.push(0);
        }
        n.part = Some(mesh.partitions.len());
        mesh.partitions.push(clipmap_iw4::ClipPartition {
            tri_count: chunk.len() as u8,
            first_tri: first_tri as i32,
            first_vert_segment: seg as u16,
            border_count: 0,
            first_border: 0,
        });
    }
    // Bounds bottom up (children always come after their parent).
    let mut tree = vec![clipmap_iw4::ClipAabbNode::default(); nodes.len()];
    for i in (0..nodes.len()).rev() {
        let n = &nodes[i];
        tree[i] = match n.part {
            Some(part) => {
                let (plo, phi) = bounds_of(order[n.range.0..n.range.1].iter().flat_map(|&o| tris[keep[o]].p));
                node(plo, phi, 0, part as i32)
            }
            None => {
                let kids = &tree[n.first_child..n.first_child + n.children];
                let (glo, ghi) = bounds_of(kids.iter().flat_map(|c| {
                    let a: [f32; 3] = std::array::from_fn(|k| c.origin[k] - c.half_size[k]);
                    let b: [f32; 3] = std::array::from_fn(|k| c.origin[k] + c.half_size[k]);
                    [a, b]
                }));
                node(glo, ghi, n.children as u16, n.first_child as i32)
            }
        };
    }
    mesh.aabb_trees = tree;
    mesh.aabb_roots = vec![0];
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_floor_stops_a_drop() {
        // (Wound to face up: a trace hits a triangle's front only.)
        let mut tris = Vec::new();
        for x in -10..10 {
            for y in -10..10 {
                let (x0, y0) = (x as f32 * 64.0, y as f32 * 64.0);
                let (x1, y1) = (x0 + 64.0, y0 + 64.0);
                for p in [[[x0, y0, 0.0], [x1, y1, 0.0], [x1, y0, 0.0]], [[x0, y0, 0.0], [x0, y1, 0.0], [x1, y1, 0.0]]] {
                    tris.push(SoupTri { p, surface_flags: 0, contents: IW7_SOLID });
                }
            }
        }
        let clip = build_soup_clip_collision(&tris, &[]);
        assert_eq!(clip.mesh.tri_count(), 800);
        assert_eq!(clip.mesh.aabb_roots, vec![0]);
        // Every leaf names a partition, every partition's triangles are in range.
        for n in &clip.mesh.aabb_trees {
            if n.child_count == 0 {
                assert!((n.u as usize) < clip.mesh.partitions.len());
            } else {
                assert!(n.u as usize + n.child_count as usize <= clip.mesh.aabb_trees.len());
            }
        }
        let census = crate::world_iw3::spawn_ground_census(&clip, &[[10.0, 10.0, 40.0], [5000.0, 0.0, 40.0]]);
        assert_eq!(census.spawns, 2);
        assert_eq!(census.on_ground, 1, "{census}");
    }

    #[test]
    fn brush_models_get_a_box_brush_the_world_does_not_list() {
        let tris = vec![SoupTri { p: [[0.0, 0.0, 0.0], [64.0, 0.0, 0.0], [0.0, 64.0, 0.0]], surface_flags: 0, contents: IW7_SOLID }];
        let clip = build_soup_clip_collision(&tris, &[([0.0; 3], [1.0; 3], 1.0), ([0.0; 3], [10.0, 20.0, 30.0], 37.0), ([0.0; 3], [0.0; 3], 0.0)]);
        assert_eq!(clip.cmodels.len(), 3);
        assert_eq!(clip.brushes.len(), 1);
        assert_eq!((clip.cmodels[1].first_brush, clip.cmodels[1].num_brushes), (0, 1));
        assert_eq!(clip.cmodels[2].num_brushes, 0);
        assert_eq!(clip.brushes[0].planes[2], [0.0, 1.0, 0.0, 20.0]);
        assert_eq!((clip.nodes.len(), clip.leaves.len(), clip.leaves[0].num_brushes), (1, 1, 0));
    }
}
