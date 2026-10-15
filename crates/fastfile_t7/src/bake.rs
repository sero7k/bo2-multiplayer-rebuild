//! bo2mp: light baked at load time from a map's own geometry, for games
//! whose own bake is not read (Black Ops III's sun shadow tree and probe
//! volume): a bounding volume tree over the map's opaque triangles, and per
//! point how much of the sun it sees (`sun_visibility`) and how open it is
//! to the sky and its surroundings (`openness`, a few short rays over the
//! hemisphere round its normal). Game-neutral: triangles in, numbers out.

/// A triangle as its first corner and two edges.
#[derive(Clone, Copy, Debug)]
struct Tri {
    p: [f32; 3],
    e1: [f32; 3],
    e2: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
struct Node {
    lo: [f32; 3],
    hi: [f32; 3],
    /// Leaf: first triangle; inner: the right child (the left is next).
    at: u32,
    /// Leaf: triangle count; inner: 0.
    count: u32,
}

/// The tree.
pub struct Tracer {
    tris: Vec<Tri>,
    nodes: Vec<Node>,
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

const LEAF: usize = 4;

impl Tracer {
    /// A tree over the triangles (corners), degenerate ones left out.
    pub fn new(corners: &[[[f32; 3]; 3]]) -> Self {
        let mut tris: Vec<Tri> = corners
            .iter()
            .filter(|t| t.iter().all(|p| p.iter().all(|v| v.is_finite())))
            .map(|t| Tri { p: t[0], e1: sub(t[1], t[0]), e2: sub(t[2], t[0]) })
            .filter(|t| {
                let n = cross(t.e1, t.e2);
                dot(n, n) > 1e-8
            })
            .collect();
        let mut nodes = Vec::with_capacity(2 * tris.len() / LEAF + 1);
        if !tris.is_empty() {
            let n = tris.len();
            build(&mut tris, 0, n, &mut nodes);
        }
        Tracer { tris, nodes }
    }

    pub fn len(&self) -> usize {
        self.tris.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tris.is_empty()
    }

    /// Whether the segment from `o` along `d` (unit) up to `max` hits any
    /// triangle (either side).
    pub fn occluded(&self, o: [f32; 3], d: [f32; 3], max: f32) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let inv = d.map(|v| if v.abs() > 1e-12 { 1.0 / v } else { 1e12f32.copysign(v) });
        let mut stack = [0u32; 64];
        let mut sp = 1usize;
        while sp > 0 {
            sp -= 1;
            let node = &self.nodes[stack[sp] as usize];
            // Slab test.
            let (mut t0, mut t1) = (0.0f32, max);
            for k in 0..3 {
                let a = (node.lo[k] - o[k]) * inv[k];
                let b = (node.hi[k] - o[k]) * inv[k];
                t0 = t0.max(a.min(b));
                t1 = t1.min(a.max(b));
            }
            if t0 > t1 {
                continue;
            }
            if node.count > 0 {
                for t in &self.tris[node.at as usize..(node.at + node.count) as usize] {
                    let pv = cross(d, t.e2);
                    let det = dot(t.e1, pv);
                    if det.abs() < 1e-9 {
                        continue;
                    }
                    let id = 1.0 / det;
                    let tv = sub(o, t.p);
                    let u = dot(tv, pv) * id;
                    if !(0.0..=1.0).contains(&u) {
                        continue;
                    }
                    let qv = cross(tv, t.e1);
                    let v = dot(d, qv) * id;
                    if v < 0.0 || u + v > 1.0 {
                        continue;
                    }
                    let dist = dot(t.e2, qv) * id;
                    if dist > 1e-3 && dist < max {
                        return true;
                    }
                }
            } else if sp + 2 <= stack.len() {
                let me = stack[sp];
                stack[sp] = me + 1;
                stack[sp + 1] = node.at;
                sp += 2;
            }
        }
        false
    }

    /// How much of the sun (direction `sun`, unit, toward it) point `p` on a
    /// surface facing `n` sees: 0..1, over a few rays spread over the sun's
    /// width `spread` (radians), offset `bias` units off the surface.
    pub fn sun_visibility(&self, p: [f32; 3], n: [f32; 3], sun: [f32; 3], spread: f32, bias: f32) -> f32 {
        let o: [f32; 3] = std::array::from_fn(|k| p[k] + n[k] * bias + sun[k] * bias);
        let (t, b) = basis(sun);
        const RAYS: [[f32; 2]; 5] = [[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]];
        let mut lit = 0.0;
        for r in RAYS {
            let d: [f32; 3] = std::array::from_fn(|k| sun[k] + (t[k] * r[0] + b[k] * r[1]) * spread);
            let l = dot(d, d).sqrt();
            let d = d.map(|v| v / l);
            if !self.occluded(o, d, 65536.0) {
                lit += 1.0;
            }
        }
        lit / RAYS.len() as f32
    }

    /// How open point `p` (facing `n`) is: the cosine-weighted share of
    /// `rays` hemisphere rays that travel `reach` units unhit (`seed`
    /// varies the pattern per point).
    pub fn openness(&self, p: [f32; 3], n: [f32; 3], reach: f32, rays: u32, seed: u32, bias: f32) -> f32 {
        let o: [f32; 3] = std::array::from_fn(|k| p[k] + n[k] * bias);
        let (t, b) = basis(n);
        let mut open = 0u32;
        let turn = (seed.wrapping_mul(2_654_435_761) >> 8) as f32 / (1u32 << 24) as f32;
        for i in 0..rays {
            // Cosine-weighted, a golden-angle spiral.
            let u = (i as f32 + 0.5) / rays as f32;
            let r = u.sqrt();
            let phi = (i as f32 * 2.399_963 + turn * std::f32::consts::TAU) % std::f32::consts::TAU;
            let (x, y) = (r * phi.cos(), r * phi.sin());
            let z = (1.0 - u).max(0.0).sqrt();
            let d: [f32; 3] = std::array::from_fn(|k| t[k] * x + b[k] * y + n[k] * z);
            if !self.occluded(o, d, reach) {
                open += 1;
            }
        }
        open as f32 / rays.max(1) as f32
    }
}

/// Two unit vectors across `n` (unit).
fn basis(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = if n[2].abs() < 0.9 { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] };
    let t = cross(a, n);
    let l = dot(t, t).sqrt().max(1e-9);
    let t = t.map(|v| v / l);
    (t, cross(n, t))
}

fn bounds(tris: &[Tri]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for t in tris {
        for c in [t.p, std::array::from_fn(|k| t.p[k] + t.e1[k]), std::array::from_fn(|k| t.p[k] + t.e2[k])] {
            for k in 0..3 {
                lo[k] = lo[k].min(c[k]);
                hi[k] = hi[k].max(c[k]);
            }
        }
    }
    (lo, hi)
}

fn centre(t: &Tri, k: usize) -> f32 {
    t.p[k] + (t.e1[k] + t.e2[k]) / 3.0
}

/// Builds the node for `tris[a..b]` (median split on the widest axis of
/// the centres), returns its index.
fn build(tris: &mut [Tri], a: usize, b: usize, nodes: &mut Vec<Node>) -> u32 {
    let (lo, hi) = bounds(&tris[a..b]);
    let me = nodes.len();
    nodes.push(Node { lo, hi, at: a as u32, count: (b - a) as u32 });
    if b - a <= LEAF {
        return me as u32;
    }
    let mut clo = [f32::MAX; 3];
    let mut chi = [f32::MIN; 3];
    for t in &tris[a..b] {
        for k in 0..3 {
            let c = centre(t, k);
            clo[k] = clo[k].min(c);
            chi[k] = chi[k].max(c);
        }
    }
    let axis = (0..3).max_by(|&i, &j| (chi[i] - clo[i]).total_cmp(&(chi[j] - clo[j]))).unwrap_or(0);
    let mid = (b - a) / 2;
    tris[a..b].select_nth_unstable_by(mid, |x, y| centre(x, axis).total_cmp(&centre(y, axis)));
    build(tris, a, a + mid, nodes);
    let right = build(tris, a + mid, b, nodes);
    nodes[me].at = right;
    nodes[me].count = 0;
    me as u32
}

/// Runs `f` over `0..n` on every core, results in order.
pub fn parallel_map<T: Send + Default + Clone>(n: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism().map_or(4, |c| c.get()).clamp(1, 32);
    let mut out = vec![T::default(); n];
    let chunk = n.div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (i, part) in out.chunks_mut(chunk).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (j, v) in part.iter_mut().enumerate() {
                    *v = f(i * chunk + j);
                }
            });
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roof_shades_the_floor() {
        // A floor at z 0 and a roof 100 units over x -50..50.
        let floor = [[[-500.0, -500.0, 0.0], [500.0, -500.0, 0.0], [500.0, 500.0, 0.0]], [[-500.0, -500.0, 0.0], [500.0, 500.0, 0.0], [-500.0, 500.0, 0.0]]];
        let roof = [[[-50.0, -50.0, 100.0], [50.0, -50.0, 100.0], [50.0, 50.0, 100.0]], [[-50.0, -50.0, 100.0], [50.0, 50.0, 100.0], [-50.0, 50.0, 100.0]]];
        let t = Tracer::new(&[floor[0], floor[1], roof[0], roof[1]]);
        let up = [0.0, 0.0, 1.0];
        assert_eq!(t.sun_visibility([0.0, 0.0, 0.0], up, up, 0.01, 0.5), 0.0);
        assert_eq!(t.sun_visibility([200.0, 0.0, 0.0], up, up, 0.01, 0.5), 1.0);
        let under = t.openness([0.0, 0.0, 0.0], up, 1000.0, 32, 1, 0.5);
        let out = t.openness([400.0, 400.0, 0.0], up, 1000.0, 32, 1, 0.5);
        assert!(under < 0.85 && out > 0.95, "{under} {out}");
    }
}
