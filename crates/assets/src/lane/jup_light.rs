//! bo2mp: light baked into a Modern Warfare III map's world vertices.
//!
//! The game's lightmaps are not read (their pages are streamed packages
//! the install does not keep whole), so the world is lit as the fallback
//! draw's Call of Duty 4 branch lights a lightmapped surface, with one
//! uniform page: `light = ambient + sun * N.L` (in gamma space), times
//! the colour map and the vertex colour. The vertex colour carries what a
//! lightmap would: per vertex, the sun's visibility (a ray toward the sun
//! through the world and the larger placed models) and the sky's (a few
//! rays over the hemisphere), as the share of that light which reaches it:
//! `(ambient * sky + sun * seen * N.L) / (ambient + sun * N.L)`.

use fastfile_t7::map::T7Map;

/// One triangle: a corner and its two edges.
#[derive(Clone, Copy)]
struct Tri {
    a: [f32; 3],
    e1: [f32; 3],
    e2: [f32; 3],
}

/// A bounding volume hierarchy over triangles, for "does this ray hit
/// anything" queries.
struct Bvh {
    /// Per node: bounds; `count` 0 for an inner node (`first` its left
    /// child, the right one follows the left subtree), else its leaf's
    /// triangles `first..first + count`.
    nodes: Vec<Node>,
    tris: Vec<Tri>,
}

#[derive(Clone, Copy)]
struct Node {
    lo: [f32; 3],
    hi: [f32; 3],
    first: u32,
    count: u32,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt();
    if l > 1e-12 { [a[0] / l, a[1] / l, a[2] / l] } else { [0.0, 0.0, 1.0] }
}

const LEAF: usize = 4;
const BINS: usize = 12;

/// The triangles' boxes and middles, which the splits read.
struct Boxes {
    centre: Vec<[f32; 3]>,
    lo_of: Vec<[f32; 3]>,
    hi_of: Vec<[f32; 3]>,
}

/// A range of triangles this many or more is split on threads of its own
/// (both halves at once), down to this many levels.
const PARALLEL_MIN: usize = 1 << 16;
const PARALLEL_LEVELS: usize = 6;

impl Boxes {
    /// A node over `order`'s triangles: its bounds, and where its
    /// triangles split in two (`order` put in that order), or `None` for
    /// a leaf. The split: binned surface area heuristic on the widest axis
    /// of the centres.
    fn split(&self, order: &mut [u32]) -> (Node, Option<usize>) {
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        let (mut clo, mut chi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for &i in order.iter() {
            let i = i as usize;
            for k in 0..3 {
                lo[k] = lo[k].min(self.lo_of[i][k]);
                hi[k] = hi[k].max(self.hi_of[i][k]);
                clo[k] = clo[k].min(self.centre[i][k]);
                chi[k] = chi[k].max(self.centre[i][k]);
            }
        }
        let node = Node { lo, hi, first: 0, count: 0 };
        let count = order.len();
        if count <= LEAF {
            return (node, None);
        }
        let axis = (0..3)
            .max_by(|&a, &b| (chi[a] - clo[a]).total_cmp(&(chi[b] - clo[b])))
            .unwrap_or(0);
        let span = chi[axis] - clo[axis];
        let mid = if span <= 1e-6 {
            count / 2
        } else {
            let bin_of = |i: usize| {
                (((self.centre[i][axis] - clo[axis]) / span * BINS as f32) as usize).min(BINS - 1)
            };
            let mut bcount = [0usize; BINS];
            let mut blo = [[f32::MAX; 3]; BINS];
            let mut bhi = [[f32::MIN; 3]; BINS];
            for &i in order.iter() {
                let i = i as usize;
                let b = bin_of(i);
                bcount[b] += 1;
                for k in 0..3 {
                    blo[b][k] = blo[b][k].min(self.lo_of[i][k]);
                    bhi[b][k] = bhi[b][k].max(self.hi_of[i][k]);
                }
            }
            let area = |lo: [f32; 3], hi: [f32; 3]| {
                let d = sub(hi, lo);
                if d[0] < 0.0 { 0.0 } else { d[0] * d[1] + d[1] * d[2] + d[2] * d[0] }
            };
            let mut best = (f32::MAX, BINS / 2);
            for split in 1..BINS {
                let (mut l_lo, mut l_hi, mut lc) = ([f32::MAX; 3], [f32::MIN; 3], 0usize);
                for b in 0..split {
                    lc += bcount[b];
                    for k in 0..3 {
                        l_lo[k] = l_lo[k].min(blo[b][k]);
                        l_hi[k] = l_hi[k].max(bhi[b][k]);
                    }
                }
                let (mut r_lo, mut r_hi, mut rc) = ([f32::MAX; 3], [f32::MIN; 3], 0usize);
                for b in split..BINS {
                    rc += bcount[b];
                    for k in 0..3 {
                        r_lo[k] = r_lo[k].min(blo[b][k]);
                        r_hi[k] = r_hi[k].max(bhi[b][k]);
                    }
                }
                if lc == 0 || rc == 0 {
                    continue;
                }
                let cost = area(l_lo, l_hi) * lc as f32 + area(r_lo, r_hi) * rc as f32;
                if cost < best.0 {
                    best = (cost, split);
                }
            }
            let split = best.1;
            let mut l = 0usize;
            for j in 0..order.len() {
                if bin_of(order[j] as usize) < split {
                    order.swap(l, j);
                    l += 1;
                }
            }
            if l == 0 || l == count { count / 2 } else { l }
        };
        (node, Some(mid))
    }

    /// The subtree over `order` (whose first triangle is `offset` in the
    /// whole order), root first, each inner node's two children side by
    /// side.
    fn subtree(&self, order: &mut [u32], offset: usize) -> Vec<Node> {
        let mut nodes: Vec<Node> = Vec::with_capacity(2 * order.len() / LEAF + 1);
        nodes.push(Node { lo: [0.0; 3], hi: [0.0; 3], first: 0, count: 0 });
        // (node index, first, end) still to split.
        let mut todo = vec![(0usize, 0usize, order.len())];
        while let Some((ni, first, end)) = todo.pop() {
            let (node, mid) = self.split(&mut order[first..end]);
            nodes[ni] = node;
            let Some(mid) = mid else {
                nodes[ni].first = (offset + first) as u32;
                nodes[ni].count = (end - first) as u32;
                continue;
            };
            let left = nodes.len();
            nodes.push(Node { lo: [0.0; 3], hi: [0.0; 3], first: 0, count: 0 });
            nodes.push(Node { lo: [0.0; 3], hi: [0.0; 3], first: 0, count: 0 });
            nodes[ni].first = left as u32;
            todo.push((left + 1, first + mid, end));
            todo.push((left, first, first + mid));
        }
        nodes
    }

    /// [`Self::subtree`], the two halves of a big range built at once.
    fn subtree_parallel(&self, order: &mut [u32], offset: usize, levels: usize) -> Vec<Node> {
        if levels == 0 || order.len() < PARALLEL_MIN {
            return self.subtree(order, offset);
        }
        let (mut root, mid) = self.split(order);
        let Some(mid) = mid else {
            root.first = offset as u32;
            root.count = order.len() as u32;
            return vec![root];
        };
        let (left_order, right_order) = order.split_at_mut(mid);
        let (left, right) = std::thread::scope(|s| {
            let right = s.spawn(|| self.subtree_parallel(right_order, offset + mid, levels - 1));
            let left = self.subtree_parallel(left_order, offset, levels - 1);
            (left, right.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
        });
        // Root, its two children, then the left subtree's other nodes,
        // then the right's (their child links moved along).
        let mut nodes = Vec::with_capacity(1 + left.len() + right.len());
        let (left_at, right_at) = (3usize, 3 + left.len() - 1);
        let moved = |n: Node, at: usize| {
            if n.count == 0 { Node { first: (n.first as usize - 1 + at) as u32, ..n } } else { n }
        };
        root.first = 1;
        root.count = 0;
        nodes.push(root);
        nodes.push(moved(left[0], left_at));
        nodes.push(moved(right[0], right_at));
        nodes.extend(left[1..].iter().map(|&n| moved(n, left_at)));
        nodes.extend(right[1..].iter().map(|&n| moved(n, right_at)));
        nodes
    }
}

impl Bvh {
    fn build(tris: Vec<[[f32; 3]; 3]>) -> Self {
        let n = tris.len();
        let mut boxes = Boxes {
            centre: Vec::with_capacity(n),
            lo_of: Vec::with_capacity(n),
            hi_of: Vec::with_capacity(n),
        };
        for t in &tris {
            let lo = std::array::from_fn(|k| t[0][k].min(t[1][k]).min(t[2][k]));
            let hi = std::array::from_fn(|k| t[0][k].max(t[1][k]).max(t[2][k]));
            boxes.centre.push(std::array::from_fn(|k| (t[0][k] + t[1][k] + t[2][k]) / 3.0));
            boxes.lo_of.push(lo);
            boxes.hi_of.push(hi);
        }
        let mut order: Vec<u32> = (0..n as u32).collect();
        let nodes = boxes.subtree_parallel(&mut order, 0, PARALLEL_LEVELS);
        let tris = order
            .iter()
            .map(|&i| {
                let t = tris[i as usize];
                Tri { a: t[0], e1: sub(t[1], t[0]), e2: sub(t[2], t[0]) }
            })
            .collect();
        Self { nodes, tris }
    }

    /// Does the ray from `o` along unit `d` hit anything closer than `far`?
    fn occluded(&self, o: [f32; 3], d: [f32; 3], far: f32) -> bool {
        if self.nodes.is_empty() || self.tris.is_empty() {
            return false;
        }
        let inv = d.map(|x| if x.abs() > 1e-9 { 1.0 / x } else { 1e9_f32.copysign(x) });
        let mut stack = [0u32; 64];
        let mut sp = 1usize;
        while sp > 0 {
            sp -= 1;
            let n = self.nodes[stack[sp] as usize];
            // Slab test.
            let mut t0 = 0.0f32;
            let mut t1 = far;
            for k in 0..3 {
                let a = (n.lo[k] - o[k]) * inv[k];
                let b = (n.hi[k] - o[k]) * inv[k];
                t0 = t0.max(a.min(b));
                t1 = t1.min(a.max(b));
            }
            if t0 > t1 {
                continue;
            }
            if n.count > 0 {
                for t in &self.tris[n.first as usize..(n.first + n.count) as usize] {
                    let p = cross(d, t.e2);
                    let det = dot(t.e1, p);
                    if det.abs() < 1e-8 {
                        continue;
                    }
                    let id = 1.0 / det;
                    let s = sub(o, t.a);
                    let u = dot(s, p) * id;
                    if !(0.0..=1.0).contains(&u) {
                        continue;
                    }
                    let q = cross(s, t.e1);
                    let v = dot(d, q) * id;
                    if v < 0.0 || u + v > 1.0 {
                        continue;
                    }
                    let tt = dot(t.e2, q) * id;
                    if tt > 0.0 && tt < far {
                        return true;
                    }
                }
            } else if sp + 2 <= stack.len() {
                stack[sp] = n.first;
                stack[sp + 1] = n.first + 1;
                sp += 2;
            }
        }
        false
    }
}

/// How the bake lights: the sun (toward it, unit) and its colour, the
/// sky's (ambient) colour, all in the gamma space the draw lights in.
pub(crate) struct BakeLight {
    pub toward_sun: [f32; 3],
    pub sun: [f32; 3],
    pub ambient: [f32; 3],
}

/// Placed models smaller than this (bounding radius) cast no baked shadow.
const OCCLUDER_MIN_RADIUS: f32 = 48.0;
/// The sky rays' reach.
const SKY_REACH: f32 = 768.0;
/// The sun ray's reach.
const SUN_REACH: f32 = 24000.0;
/// A placed model's N.L to the sun, taken as one over its faces.
const PROP_SLANT: f32 = 0.65;

/// Bakes the world vertices' colours (see the module notes). `skip`:
/// surfaces neither lit by the bake nor shadowing (the sea).
pub(crate) fn bake(t7: &mut T7Map, light: &BakeLight, skip: &[bool], report: &mut Vec<String>) {
    let started = std::time::Instant::now();
    let mut tris: Vec<[[f32; 3]; 3]> = Vec::new();
    for (i, s) in t7.surfaces.iter().enumerate() {
        if skip.get(i).copied().unwrap_or(false) {
            continue;
        }
        for t in 0..s.tri_count as usize {
            let corner = |k: usize| {
                let ix = t7.indices.get(s.base_index as usize + 3 * t + k).copied().unwrap_or(0);
                t7.vertices.get((s.first_vertex + u32::from(ix)) as usize).map_or([0.0; 3], |v| v.xyz)
            };
            tris.push([corner(0), corner(1), corner(2)]);
        }
    }
    let world_tris = tris.len();
    // The larger placed models (coarse LODs), not foliage.
    // Each model's bounding radius and its top (local z).
    let (radius, top): (Vec<f32>, Vec<f32>) = t7
        .models
        .iter()
        .map(|m| {
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for p in m.coarse.iter().chain(m.surfaces.iter()).flat_map(|s| s.positions.iter()) {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            if lo[0] > hi[0] { (0.0, 0.0) } else { (0.5 * dot(sub(hi, lo), sub(hi, lo)).sqrt(), hi[2]) }
        })
        .unzip();
    let mut placed = 0usize;
    for sm in &t7.static_models {
        let m = &t7.models[sm.model];
        if radius[sm.model] * sm.scale < OCCLUDER_MIN_RADIUS
            || ["foliage", "grass", "decal", "vfx", "fx_", "leaves", "weeds", "tree", "bush", "ivy", "vine"]
                .iter()
                .any(|w| m.name.contains(w))
        {
            continue;
        }
        let surfaces = if m.coarse.is_empty() { &m.surfaces } else { &m.coarse };
        let place = |p: [f32; 3]| -> [f32; 3] {
            std::array::from_fn(|k| {
                sm.origin[k] + sm.scale * (p[0] * sm.axis[0][k] + p[1] * sm.axis[1][k] + p[2] * sm.axis[2][k])
            })
        };
        for s in surfaces {
            for t in &s.tris {
                let [a, b, c] = t.map(|i| s.positions.get(usize::from(i)).copied().unwrap_or_default());
                tris.push([place(a), place(b), place(c)]);
            }
        }
        placed += 1;
    }
    let total = tris.len();
    let bvh = Bvh::build(tris);
    let built = started.elapsed().as_secs_f32();
    // Which vertices to bake: those of the surfaces lit.
    let mut lit = vec![false; t7.vertices.len()];
    for (i, s) in t7.surfaces.iter().enumerate() {
        if skip.get(i).copied().unwrap_or(false) {
            continue;
        }
        for v in s.first_vertex as usize..(s.first_vertex + s.vertex_count) as usize {
            if let Some(l) = lit.get_mut(v) {
                *l = true;
            }
        }
    }
    let l = normalize(light.toward_sun);
    // Sky directions: a fixed cosine-weighted set over the hemisphere
    // about +z, turned onto each normal.
    const SKY: usize = 6;
    let sky: Vec<[f32; 3]> = (0..SKY)
        .map(|k| {
            let u = (k as f32 + 0.5) / SKY as f32;
            let phi = k as f32 * 2.399_963;
            let r = u.sqrt();
            [r * phi.cos(), r * phi.sin(), (1.0 - u).sqrt()]
        })
        .collect();
    let lum = |c: [f32; 3]| 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
    let (amb, sun) = (lum(light.ambient), lum(light.sun));
    let colours: Vec<u32> = {
        let verts = &t7.vertices;
        let next = std::sync::atomic::AtomicUsize::new(0);
        let out = std::sync::Mutex::new(vec![0xFFFF_FFFFu32; verts.len()]);
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get().clamp(2, 16));
        const CHUNK: usize = 4096;
        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| {
                    let mut local: Vec<(usize, u32)> = Vec::new();
                    loop {
                        let start = next.fetch_add(CHUNK, std::sync::atomic::Ordering::Relaxed);
                        if start >= verts.len() {
                            break;
                        }
                        for i in start..(start + CHUNK).min(verts.len()) {
                            if !lit[i] {
                                continue;
                            }
                            let v = &verts[i];
                            let n = normalize(v.normal);
                            let o = [v.xyz[0] + n[0] * 2.0, v.xyz[1] + n[1] * 2.0, v.xyz[2] + n[2] * 2.0];
                            let ndl = dot(n, l).max(0.0);
                            let seen = if ndl > 0.0 && !bvh.occluded(o, l, SUN_REACH) { 1.0 } else { 0.0 };
                            // The sky over the normal's hemisphere.
                            let t = normalize(if n[2].abs() < 0.9 { cross(n, [0.0, 0.0, 1.0]) } else { cross(n, [1.0, 0.0, 0.0]) });
                            let b = cross(n, t);
                            let mut open = 0usize;
                            for d in &sky {
                                let dir = normalize([
                                    t[0] * d[0] + b[0] * d[1] + n[0] * d[2],
                                    t[1] * d[0] + b[1] * d[1] + n[1] * d[2],
                                    t[2] * d[0] + b[2] * d[1] + n[2] * d[2],
                                ]);
                                if !bvh.occluded(o, dir, SKY_REACH) {
                                    open += 1;
                                }
                            }
                            // (Fully closed corners keep a little light.)
                            let sky_seen = 0.25 + 0.75 * open as f32 / SKY as f32;
                            let full = amb + sun * ndl;
                            let share = if full > 1e-4 { (amb * sky_seen + sun * seen * ndl) / full } else { sky_seen };
                            let c = (share.clamp(0.0, 1.0) * 255.0).round() as u32;
                            local.push((i, c | c << 8 | c << 16 | 0xFF << 24));
                        }
                        if local.len() > 1 << 16
                            && let Ok(mut o) = out.lock()
                        {
                            for (i, c) in local.drain(..) {
                                o[i] = c;
                            }
                        }
                    }
                    if let Ok(mut o) = out.lock() {
                        for (i, c) in local {
                            o[i] = c;
                        }
                    }
                });
            }
        });
        out.into_inner().unwrap_or_default()
    };
    let baked = lit.iter().filter(|&&b| b).count();
    for (v, c) in t7.vertices.iter_mut().zip(colours) {
        v.color = c;
    }
    // The placed models: one light each, as a light grid sample, seen from
    // just over the model's top (inside it, its own walls would shade it):
    // the sky's light by how open the sky is there, the sun's by whether
    // it reaches there, at a typical slant.
    let up = [0.0, 0.0, 1.0];
    let (t, b) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let static_light: Vec<[u8; 4]> = t7
        .static_models
        .iter()
        .map(|sm| {
            let lift = top[sm.model] * sm.scale;
            let o = [
                sm.origin[0] + sm.axis[2][0] * lift,
                sm.origin[1] + sm.axis[2][1] * lift,
                sm.origin[2] + sm.axis[2][2] * lift + 8.0,
            ];
            let seen = if l[2] > 0.0 && !bvh.occluded(o, l, SUN_REACH) { 1.0 } else { 0.0 };
            let open = sky
                .iter()
                .filter(|d| {
                    let dir = normalize([
                        t[0] * d[0] + b[0] * d[1] + up[0] * d[2],
                        t[1] * d[0] + b[1] * d[1] + up[1] * d[2],
                        t[2] * d[0] + b[2] * d[1] + up[2] * d[2],
                    ]);
                    !bvh.occluded(o, dir, SKY_REACH)
                })
                .count();
            let sky_seen = 0.25 + 0.75 * open as f32 / SKY as f32;
            let c = |k: usize| {
                let g = light.ambient[k] * sky_seen + light.sun[k] * seen * PROP_SLANT;
                (g * 45.07).round().clamp(0.0, 254.0) as u8
            };
            [c(0), c(1), c(2), 0]
        })
        .collect();
    t7.static_light = static_light;
    report.push(format!(
        "jup light: {baked} vertices baked (sun and {SKY} sky rays) against {total} triangles ({world_tris} world, {placed} placed models) in {:.1}s (tree {built:.1}s)",
        started.elapsed().as_secs_f32()
    ));
}

#[cfg(test)]
mod tests {
    /// The tree built on threads is the one-thread tree with its nodes in
    /// another order: the same leaves, so every ray answers the same.
    #[test]
    fn the_threaded_tree_answers_as_the_serial_one() {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut rnd = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 40) as f32 / (1u64 << 24) as f32
        };
        let tris: Vec<[[f32; 3]; 3]> = (0..super::PARALLEL_MIN * 3)
            .map(|_| {
                let c = [rnd() * 4000.0, rnd() * 4000.0, rnd() * 400.0];
                std::array::from_fn(|_| [c[0] + rnd() * 30.0, c[1] + rnd() * 30.0, c[2] + rnd() * 30.0])
            })
            .collect();
        let threaded = super::Bvh::build(tris.clone());
        let mut serial = super::Bvh::build(Vec::new());
        {
            let boxes = super::Boxes {
                centre: tris.iter().map(|t| std::array::from_fn(|k| (t[0][k] + t[1][k] + t[2][k]) / 3.0)).collect(),
                lo_of: tris.iter().map(|t| std::array::from_fn(|k| t[0][k].min(t[1][k]).min(t[2][k]))).collect(),
                hi_of: tris.iter().map(|t| std::array::from_fn(|k| t[0][k].max(t[1][k]).max(t[2][k]))).collect(),
            };
            let mut order: Vec<u32> = (0..tris.len() as u32).collect();
            serial.nodes = boxes.subtree(&mut order, 0);
            serial.tris = order
                .iter()
                .map(|&i| {
                    let t = tris[i as usize];
                    super::Tri { a: t[0], e1: super::sub(t[1], t[0]), e2: super::sub(t[2], t[0]) }
                })
                .collect();
        }
        assert_eq!(threaded.nodes.len(), serial.nodes.len());
        let mut hits = 0;
        for _ in 0..4000 {
            let o = [rnd() * 4000.0, rnd() * 4000.0, rnd() * 400.0];
            let d = super::normalize([rnd() - 0.5, rnd() - 0.5, rnd() - 0.5]);
            let a = threaded.occluded(o, d, 3000.0);
            assert_eq!(a, serial.occluded(o, d, 3000.0));
            hits += usize::from(a);
        }
        assert!(hits > 100 && hits < 3900);
    }

    #[test]
    fn a_floor_under_a_roof_is_shadowed() {
        let quad = |z: f32, s: f32| {
            vec![
                [[-s, -s, z], [s, -s, z], [s, s, z]],
                [[-s, -s, z], [s, s, z], [-s, s, z]],
            ]
        };
        let bvh = super::Bvh::build(quad(100.0, 50.0));
        assert!(bvh.occluded([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1000.0));
        assert!(!bvh.occluded([200.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1000.0));
        assert!(!bvh.occluded([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 50.0));
    }
}
