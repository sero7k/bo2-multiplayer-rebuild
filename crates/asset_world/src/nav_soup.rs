//! bo2mp: a walk graph for a map that has none the AI can use: Infinite
//! Warfare's maps route their AI over a nav mesh this engine does not
//! read, and keep path nodes only for their traversals. Black Ops II's
//! Zombies AI walks path nodes (`sim::t6::nav`), so nodes are laid out over
//! the map's collision triangles: on a square lattice from seed points
//! (player starts, zombie spawners...), each joined to its eight
//! neighbours where a player-sized body can walk between them (a floor
//! under each, a step of at most `STEP` up or down, head room, nothing
//! across the way at knee or chest height). Not part of upstream IW4L.

use std::collections::{HashMap, VecDeque};

use crate::world_iw7::SoupTri;

/// Highest step up or down between neighbours.
const STEP: f32 = 30.0;
/// Head room a node needs above its floor.
const HEAD: f32 = 60.0;
/// A walkable floor's least upward normal (about 45 degrees).
const FLOOR_NZ: f32 = 0.7;
/// The cells triangles are binned in.
const CELL: f32 = 128.0;

/// A node of the walk graph and its links (node indices).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WalkNode {
    pub origin: [f32; 3],
    pub links: Vec<u32>,
}

struct Tri {
    p: [[f32; 3]; 3],
    floor: bool,
}

struct Soup {
    tris: Vec<Tri>,
    cells: HashMap<(i32, i32), Vec<u32>>,
}

fn cell_of(x: f32, y: f32) -> (i32, i32) {
    ((x / CELL).floor() as i32, (y / CELL).floor() as i32)
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

/// Where the segment `a`..`b` crosses the triangle (0..1 along it), if it does.
fn segment_hits(t: &[[f32; 3]; 3], a: [f32; 3], b: [f32; 3]) -> Option<f32> {
    let d = sub(b, a);
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    let h = cross(d, e2);
    let det = dot(e1, h);
    if det.abs() < 1e-7 {
        return None;
    }
    let inv = 1.0 / det;
    let s = sub(a, t[0]);
    let u = dot(s, h) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let f = dot(e2, q) * inv;
    (0.0..=1.0).contains(&f).then_some(f)
}

impl Soup {
    fn new(tris: &[SoupTri]) -> Self {
        let mut out = Soup { tris: Vec::with_capacity(tris.len()), cells: HashMap::new() };
        for t in tris {
            let n = cross(sub(t.p[1], t.p[0]), sub(t.p[2], t.p[0]));
            let len = dot(n, n).sqrt();
            if !(len > 1e-6) {
                continue;
            }
            let i = out.tris.len() as u32;
            out.tris.push(Tri { p: t.p, floor: (n[2] / len).abs() >= FLOOR_NZ });
            let (lo, hi) = (
                cell_of(t.p.iter().map(|p| p[0]).fold(f32::MAX, f32::min), t.p.iter().map(|p| p[1]).fold(f32::MAX, f32::min)),
                cell_of(t.p.iter().map(|p| p[0]).fold(f32::MIN, f32::max), t.p.iter().map(|p| p[1]).fold(f32::MIN, f32::max)),
            );
            // (A huge triangle is binned in at most 64 x 64 cells.)
            if (hi.0 - lo.0) > 64 || (hi.1 - lo.1) > 64 {
                continue;
            }
            for cx in lo.0..=hi.0 {
                for cy in lo.1..=hi.1 {
                    out.cells.entry((cx, cy)).or_default().push(i);
                }
            }
        }
        out
    }

    /// The triangles in the cells a segment's box reaches.
    fn near(&self, a: [f32; 3], b: [f32; 3], out: &mut Vec<u32>) {
        out.clear();
        let lo = cell_of(a[0].min(b[0]), a[1].min(b[1]));
        let hi = cell_of(a[0].max(b[0]), a[1].max(b[1]));
        for cx in lo.0..=hi.0 {
            for cy in lo.1..=hi.1 {
                if let Some(list) = self.cells.get(&(cx, cy)) {
                    out.extend_from_slice(list);
                }
            }
        }
        if lo != hi {
            out.sort_unstable();
            out.dedup();
        }
    }

    fn blocked(&self, a: [f32; 3], b: [f32; 3], scratch: &mut Vec<u32>) -> bool {
        self.near(a, b, scratch);
        scratch.iter().any(|&i| segment_hits(&self.tris[i as usize].p, a, b).is_some())
    }

    /// The highest floor under `p` within `drop` below it.
    fn floor_under(&self, p: [f32; 3], drop: f32, scratch: &mut Vec<u32>) -> Option<f32> {
        let b = [p[0], p[1], p[2] - drop];
        self.near(p, b, scratch);
        scratch
            .iter()
            .filter_map(|&i| {
                let t = &self.tris[i as usize];
                let f = segment_hits(&t.p, p, b)?;
                Some((p[2] - drop * f, t.floor))
            })
            // The first surface met going down must be a floor.
            .max_by(|x, y| x.0.total_cmp(&y.0))
            .and_then(|(z, floor)| floor.then_some(z))
    }

    /// A player-sized body fits standing at `p` (on its floor).
    fn room(&self, p: [f32; 3], scratch: &mut Vec<u32>) -> bool {
        !self.blocked([p[0], p[1], p[2] + 8.0], [p[0], p[1], p[2] + HEAD], scratch)
    }

    /// A body walks straight from `a` to `b` (both on their floors).
    fn walkable(&self, a: [f32; 3], b: [f32; 3], scratch: &mut Vec<u32>) -> bool {
        for h in [STEP + 4.0, 48.0] {
            if self.blocked([a[0], a[1], a[2] + h], [b[0], b[1], b[2] + h], scratch) {
                return false;
            }
        }
        // Floor all the way (no gap to fall through): the midpoint's.
        let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, a[2].max(b[2]) + STEP];
        self.floor_under(mid, STEP * 2.0 + (a[2] - b[2]).abs(), scratch).is_some()
    }
}

/// Collision triangles binned for floor and walk queries.
pub struct WalkSoup(Soup);

impl WalkSoup {
    pub fn new(tris: &[SoupTri]) -> Self {
        WalkSoup(Soup::new(tris))
    }

    /// The highest floor under `p` within `drop` below it (the first
    /// surface met going down, when it is walkable).
    pub fn floor_under(&self, p: [f32; 3], drop: f32) -> Option<f32> {
        self.0.floor_under(p, drop, &mut Vec::new())
    }

    /// See [`soup_walk_graph`].
    pub fn graph(&self, seeds: &[[f32; 3]], spacing: f32, max_nodes: usize) -> Vec<WalkNode> {
        graph(&self.0, seeds, spacing, max_nodes)
    }
}

/// The walk graph over `tris` reachable from `seeds`, nodes `spacing`
/// apart, at most `max_nodes` of them (the flood stops there).
pub fn soup_walk_graph(tris: &[SoupTri], seeds: &[[f32; 3]], spacing: f32, max_nodes: usize) -> Vec<WalkNode> {
    graph(&Soup::new(tris), seeds, spacing, max_nodes)
}

fn graph(soup: &Soup, seeds: &[[f32; 3]], spacing: f32, max_nodes: usize) -> Vec<WalkNode> {
    let mut scratch = Vec::new();
    let mut nodes: Vec<WalkNode> = Vec::new();
    // Lattice column -> nodes in it (several floors).
    let mut column: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
    let mut queue: VecDeque<u32> = VecDeque::new();
    let key = |x: f32, y: f32| ((x / spacing).round() as i32, (y / spacing).round() as i32);
    let find_or_add = |nodes: &mut Vec<WalkNode>, column: &mut HashMap<(i32, i32), Vec<u32>>, p: [f32; 3]| -> (u32, bool) {
        let k = key(p[0], p[1]);
        if let Some(list) = column.get(&k)
            && let Some(&n) = list.iter().find(|&&n| (nodes[n as usize].origin[2] - p[2]).abs() < STEP)
        {
            return (n, false);
        }
        let n = nodes.len() as u32;
        nodes.push(WalkNode { origin: p, links: Vec::new() });
        column.entry(k).or_default().push(n);
        (n, true)
    };
    for s in seeds {
        // The seed's own lattice point, on the floor under it.
        let (kx, ky) = key(s[0], s[1]);
        let p = [kx as f32 * spacing, ky as f32 * spacing, s[2] + 40.0];
        let Some(z) = soup.floor_under(p, 140.0, &mut scratch) else { continue };
        let at = [p[0], p[1], z];
        if !soup.room(at, &mut scratch) {
            continue;
        }
        let (n, new) = find_or_add(&mut nodes, &mut column, at);
        if new {
            queue.push_back(n);
        }
    }
    const DIRS: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    while let Some(n) = queue.pop_front() {
        let from = nodes[n as usize].origin;
        let (kx, ky) = key(from[0], from[1]);
        for (dx, dy) in DIRS {
            let (x, y) = ((kx + dx) as f32 * spacing, (ky + dy) as f32 * spacing);
            let Some(z) = soup.floor_under([x, y, from[2] + STEP + 2.0], STEP * 2.0 + 2.0, &mut scratch) else { continue };
            let to = [x, y, z];
            if !soup.room(to, &mut scratch) || !soup.walkable(from, to, &mut scratch) {
                continue;
            }
            if nodes.len() >= max_nodes && column.get(&(kx + dx, ky + dy)).is_none_or(|l| !l.iter().any(|&m| (nodes[m as usize].origin[2] - z).abs() < STEP)) {
                continue;
            }
            let (m, new) = find_or_add(&mut nodes, &mut column, to);
            if m == n {
                continue;
            }
            if !nodes[n as usize].links.contains(&m) {
                nodes[n as usize].links.push(m);
            }
            if !nodes[m as usize].links.contains(&n) {
                nodes[m as usize].links.push(n);
            }
            if new {
                queue.push_back(m);
            }
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_iw7::IW7_SOLID;

    fn quad(tris: &mut Vec<SoupTri>, a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) {
        tris.push(SoupTri { p: [a, b, c], surface_flags: 0, contents: IW7_SOLID });
        tris.push(SoupTri { p: [a, c, d], surface_flags: 0, contents: IW7_SOLID });
    }

    #[test]
    fn a_wall_splits_a_floor_and_a_doorway_joins_it() {
        let mut tris = Vec::new();
        quad(&mut tris, [0.0, 0.0, 0.0], [1024.0, 0.0, 0.0], [1024.0, 512.0, 0.0], [0.0, 512.0, 0.0]);
        // A wall at x = 500 with a doorway between y = 200 and 300.
        quad(&mut tris, [500.0, 0.0, 0.0], [500.0, 200.0, 0.0], [500.0, 200.0, 128.0], [500.0, 0.0, 128.0]);
        quad(&mut tris, [500.0, 300.0, 0.0], [500.0, 512.0, 0.0], [500.0, 512.0, 128.0], [500.0, 300.0, 128.0]);
        let nodes = soup_walk_graph(&tris, &[[100.0, 100.0, 10.0]], 64.0, 10_000);
        assert!(nodes.len() > 100, "{}", nodes.len());
        // Both sides reached, the far one only through the doorway.
        assert!(nodes.iter().any(|n| n.origin[0] > 600.0));
        for (i, n) in nodes.iter().enumerate() {
            for &l in &n.links {
                let m = &nodes[l as usize];
                let crosses = (n.origin[0] - 500.0).signum() != (m.origin[0] - 500.0).signum();
                if crosses {
                    let y = (n.origin[1] + m.origin[1]) * 0.5;
                    assert!((200.0..=300.0).contains(&y), "node {i} crosses the wall at y {y}");
                }
            }
        }
        // Without the doorway, the far side is never reached.
        let mut closed = tris.clone();
        quad(&mut closed, [500.0, 200.0, 0.0], [500.0, 300.0, 0.0], [500.0, 300.0, 128.0], [500.0, 200.0, 128.0]);
        let nodes = soup_walk_graph(&closed, &[[100.0, 100.0, 10.0]], 64.0, 10_000);
        assert!(nodes.iter().all(|n| n.origin[0] < 500.0));
    }

    #[test]
    fn a_ledge_too_high_is_not_climbed() {
        let mut tris = Vec::new();
        quad(&mut tris, [0.0, 0.0, 0.0], [256.0, 0.0, 0.0], [256.0, 256.0, 0.0], [0.0, 256.0, 0.0]);
        quad(&mut tris, [256.0, 0.0, 96.0], [512.0, 0.0, 96.0], [512.0, 256.0, 96.0], [256.0, 256.0, 96.0]);
        quad(&mut tris, [256.0, 0.0, 0.0], [256.0, 256.0, 0.0], [256.0, 256.0, 96.0], [256.0, 0.0, 96.0]);
        let nodes = soup_walk_graph(&tris, &[[64.0, 64.0, 10.0]], 64.0, 10_000);
        assert!(!nodes.is_empty());
        assert!(nodes.iter().all(|n| n.origin[2] < 1.0));
    }
}
