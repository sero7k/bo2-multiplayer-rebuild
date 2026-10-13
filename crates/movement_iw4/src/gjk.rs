//! bo2zm: Black Ops II's player collision (its "GJK" character controller),
//! the plain-math parts: the player's shape, the plane solver that bends a
//! move along what it touches, and the "can I stand on this" rule. Facts and
//! numbers from Black Ops II's server.
//! Nothing calls this yet: the traces and the slide move come next.
#![allow(dead_code)]

/// How far the shape sits inside the box Black Ops II gives it, before the
/// 0.125 trace margin is taken back off (0.5 - 0.125 = 0.375 per side).
pub const RADIUS_ADJUST: f32 = 0.5;
const TRACE_MARGIN: f32 = 0.125;

/// The 12-sided body's top is a cap this tall; its feet a cone this tall.
const HEAD_OFFSET: f32 = 10.0;
const FOOT_OFFSET: f32 = 2.0;

/// Ground with an up-slope of at least this much is ground you can stand on.
pub const WALKABLE_NORMAL_Z: f32 = 0.7;

/// A contact counts as "on" a face within this share of the face's distance.
const ON_FACE: f32 = 0.999;

/// Black Ops II's plane solver gives up below this (two or three planes that
/// are nearly the same plane).
const SOLVER_DEGENERATE: f32 = 1.0e-5;

/// Most planes one solve takes.
pub const MAX_PLANES: usize = 128;

/// cos and sin of the 12 corners of the body's ring, every 30 degrees.
const RING: [[f32; 2]; 12] = [
    [1.0, 0.0],
    [0.866_025_4, 0.5],
    [0.5, 0.866_025_4],
    [0.0, 1.0],
    [-0.5, 0.866_025_4],
    [-0.866_025_4, 0.5],
    [-1.0, 0.0],
    [-0.866_025_4, -0.5],
    [-0.5, -0.866_025_4],
    [0.0, -1.0],
    [0.5, -0.866_025_4],
    [0.866_025_4, -0.5],
];

/// The player's collision shape. Against brushes it is a 12-sided upright
/// prism, flat sides at `capsule_radius`, with a 10-unit pointed cap on top
/// and a 2-unit cone under the feet; against terrain and entity boxes it is a
/// capsule. Both get a `geom_radius` skin all round.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hull {
    /// Middle of the shape, from the player's origin.
    pub center: [f32; 3],
    pub half_height: f32,
    pub capsule_radius: f32,
    /// Corner radius of the 12-sided ring: `capsule_radius / cos(15)`.
    pub polygon_radius: f32,
    pub geom_radius: f32,
    pub head_offset: f32,
    pub foot_offset: f32,
    /// true: the capsule shape (terrain, boxes); false: the 12-sided body.
    pub capsule: bool,
}

/// Black Ops II's player shape from the player's box. `adjust_height` is the
/// dvar phys_player_collision_adjust_height (0 by default).
pub fn player_hull(mins: [f32; 3], maxs: [f32; 3], adjust_height: f32) -> Hull {
    let adj = RADIUS_ADJUST - TRACE_MARGIN;
    let low = |a: usize| {
        if mins[a] + adj <= maxs[a] {
            mins[a] + adj
        } else {
            maxs[a]
        }
    };
    let high = |a: usize| {
        if mins[a] <= maxs[a] - adj {
            maxs[a] - adj
        } else {
            mins[a]
        }
    };
    let dim_x = high(0) - low(0);
    let dim_z = high(2) - low(2);
    let mut radius = (if dim_z < dim_x { dim_z } else { dim_x }) * 0.5;
    if radius < 1.0 {
        radius = 1.0;
    }
    let mut half_height = dim_z * 0.5;
    if half_height < 1.0 {
        half_height = 1.0;
    }
    let cos15 = libm::cosf(core::f32::consts::PI / 12.0);
    Hull {
        center: [0.0, 0.0, (high(2) + low(2)) * 0.5 + adjust_height],
        half_height,
        capsule_radius: radius,
        polygon_radius: radius / cos15,
        geom_radius: RADIUS_ADJUST,
        head_offset: if HEAD_OFFSET < half_height { HEAD_OFFSET } else { half_height },
        foot_offset: if FOOT_OFFSET < half_height { FOOT_OFFSET } else { half_height },
        capsule: false,
    }
}

impl Hull {
    /// The shape's farthest point toward `d` (before the skin), as Black Ops
    /// II picks it: the ring corner nearest d's heading, upper ring when d
    /// points up, lower ring when down, or the cap's tip when that is farther.
    pub fn support(&self, d: [f32; 3]) -> [f32; 3] {
        let c = self.center;
        if self.capsule {
            let reach = self.half_height - self.capsule_radius;
            let z = if d[2] * (2.0 * reach) >= 0.0 { c[2] + reach } else { c[2] - reach };
            return [c[0], c[1], z];
        }
        let tau = core::f32::consts::TAU;
        let mut heading = libm::atan2f(d[1], d[0]);
        if heading < 0.0 {
            heading += tau;
        }
        let corner = ((heading * 12.0 * 0.159_154_94 + 0.5) as i32 % 12) as usize;
        let (ring_z, tip_z) = if d[2] < 0.0 {
            (c[2] + self.foot_offset - self.half_height, c[2] - self.half_height)
        } else {
            (c[2] + self.half_height - self.head_offset, c[2] + self.half_height)
        };
        let [cos, sin] = RING[corner];
        let ring = [
            c[0] + self.polygon_radius * cos,
            c[1] + self.polygon_radius * sin,
            ring_z,
        ];
        let tip = [c[0], c[1], tip_z];
        if dot(d, sub(ring, tip)) >= 0.0 { ring } else { tip }
    }

    /// The skin GJK adds around the shape: just the skin for the 12-sided
    /// body, skin plus the capsule's radius for the capsule.
    pub fn skin(&self) -> f32 {
        if self.capsule {
            self.geom_radius + self.capsule_radius
        } else {
            self.geom_radius
        }
    }

    /// Whether a contact `z` (from the player's origin) is low enough to be
    /// the feet: the bottom 2.125 units standing.
    pub fn is_foot(&self, z: f32) -> bool {
        let bottom = self.center[2] - self.half_height;
        if self.capsule {
            z <= bottom + self.capsule_radius - 1.0
        } else {
            let sin30 = libm::sinf(core::f32::consts::FRAC_PI_6);
            z <= bottom + self.foot_offset - sin30 * self.geom_radius
        }
    }
}

/// One plane for [`project`]: the result must end up with
/// `normal . x >= rs`. Inactive planes are skipped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolvePlane {
    pub normal: [f32; 3],
    pub rs: f32,
    pub active: bool,
}

/// Black Ops II's plane solver: the smallest change to `start` that clears
/// every active plane (`normal . x >= rs`), working with at most three planes
/// at once. Each round takes the plane the current answer cuts deepest
/// (each plane once), then keeps whichever of "this plane", "this plane and
/// one kept plane" or "this plane and two kept planes" pushes farthest, so a
/// floor, a wall and the crease between them all come out right. It stops
/// when a round would push less than the last one.
pub fn project(start: [f32; 3], planes: &[SolvePlane]) -> [f32; 3] {
    assert!(planes.len() <= MAX_PLANES, "{} planes", planes.len());
    let mut picked = [false; MAX_PLANES];
    let mut p = start;
    let mut prev_score = 0.0f32;
    // The kept planes: four slots, a bit per slot in `set`.
    let mut set = 0u32;
    let mut slot_plane = [0usize; 4];
    let mut slot_e = [0.0f32; 4];
    let mut gram = [[0.0f32; 4]; 4];
    let mut rounds = 0usize;
    while !planes.is_empty() {
        let mut deepest = 1.0e13f32;
        let mut chosen = None;
        for (i, plane) in planes.iter().enumerate() {
            if !plane.active || picked[i] {
                continue;
            }
            let cut = dot(plane.normal, p) - plane.rs;
            if cut < deepest {
                deepest = cut;
                chosen = Some(i);
            }
        }
        let Some(new_plane) = chosen else { break };
        if deepest >= 0.0 {
            break;
        }
        picked[new_plane] = true;
        let slot = (0..4).find(|s| set & (1 << s) == 0).expect("a free slot");
        let n_new = planes[new_plane].normal;
        slot_plane[slot] = new_plane;
        slot_e[slot] = planes[new_plane].rs - dot(n_new, start);
        gram[slot][slot] = 1.0;
        for j in (0..4).filter(|j| set & (1 << j) != 0) {
            let g = dot(n_new, planes[slot_plane[j]].normal);
            gram[slot][j] = g;
            gram[j][slot] = g;
        }

        let e = slot_e;
        let mut best = e[slot] * e[slot];
        let mut best_set = 1u32 << slot;
        let mut best_lambda = [0.0f32; 4];
        best_lambda[slot] = e[slot];
        for j in (0..4).filter(|j| set & (1 << j) != 0) {
            let g = gram[slot][j];
            let a = e[slot] - g * e[j];
            let b = e[j] - g * e[slot];
            let det = 1.0 - g * g;
            if a >= 0.0 && b >= 0.0 && det > SOLVER_DEGENERATE {
                let score = (a * a + 2.0 * a * b * g + b * b) / (det * det);
                if score > best {
                    best = score;
                    best_set = (1 << slot) | (1 << j);
                    best_lambda = [0.0; 4];
                    best_lambda[slot] = a / det;
                    best_lambda[j] = b / det;
                }
            }
            for k in (j + 1..4).filter(|k| set & (1 << k) != 0) {
                let idx = [slot, j, k];
                let m = |r: usize, c: usize| gram[idx[r]][idx[c]];
                let rhs = [e[slot], e[j], e[k]];
                // Adjugate of the 3x3 Gram matrix (it is symmetric).
                let adj = [
                    [
                        m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1),
                        m(0, 2) * m(2, 1) - m(0, 1) * m(2, 2),
                        m(0, 1) * m(1, 2) - m(0, 2) * m(1, 1),
                    ],
                    [
                        m(1, 2) * m(2, 0) - m(1, 0) * m(2, 2),
                        m(0, 0) * m(2, 2) - m(0, 2) * m(2, 0),
                        m(0, 2) * m(1, 0) - m(0, 0) * m(1, 2),
                    ],
                    [
                        m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0),
                        m(0, 1) * m(2, 0) - m(0, 0) * m(2, 1),
                        m(0, 0) * m(1, 1) - m(0, 1) * m(1, 0),
                    ],
                ];
                let det = m(0, 0) * adj[0][0] + m(0, 1) * adj[1][0] + m(0, 2) * adj[2][0];
                let num = [0, 1, 2].map(|r| {
                    adj[r][0] * rhs[0] + adj[r][1] * rhs[1] + adj[r][2] * rhs[2]
                });
                if num.iter().all(|&x| x >= 0.0) && det > SOLVER_DEGENERATE {
                    let l = num.map(|x| x / det);
                    let mut score = 0.0;
                    for r in 0..3 {
                        for c in 0..3 {
                            score += l[r] * m(r, c) * l[c];
                        }
                    }
                    if score > best {
                        best = score;
                        best_set = (1 << slot) | (1 << j) | (1 << k);
                        best_lambda = [0.0; 4];
                        for r in 0..3 {
                            best_lambda[idx[r]] = l[r];
                        }
                    }
                }
            }
        }
        if best < prev_score {
            break;
        }
        prev_score = best;
        set = best_set;
        let lambda = best_lambda;
        p = start;
        for s in (0..4).filter(|s| set & (1 << s) != 0) {
            let n = planes[slot_plane[s]].normal;
            p = [
                p[0] + lambda[s] * n[0],
                p[1] + lambda[s] * n[1],
                p[2] + lambda[s] * n[2],
            ];
        }
        rounds += 1;
        if rounds >= planes.len() {
            break;
        }
    }
    p
}

/// Black Ops II's walkable rule for a contact: never unless it is at the
/// feet; yes if the contact's normal points up at least 0.7; otherwise the
/// thing touched decides (`face_rule`, e.g. [`brush_face_walkable`]).
pub fn is_walkable(at_feet: bool, hit_normal_z: f32, face_rule: impl FnOnce() -> bool) -> bool {
    if !at_feet {
        return false;
    }
    if hit_normal_z >= WALKABLE_NORMAL_Z {
        return true;
    }
    face_rule()
}

/// Whether a point at `dist` along a face's normal sits on a face at `face`.
fn on_face(dist: f32, face: f32) -> bool {
    face * ON_FACE <= dist || face <= dist * ON_FACE
}

/// Picks the walkable answer over a list of faces (normal . point, face
/// distance, normal . up): walkable if the point sits on any face that points
/// up 0.7 or more, else whatever the face the point is farthest out of says.
struct FaceVote {
    nearest: f32,
    nearest_up: f32,
}

impl FaceVote {
    fn new() -> Self {
        Self { nearest: -100_000.0, nearest_up: 0.0 }
    }

    /// true: the point is on a walkable face, stop.
    fn face(&mut self, dist: f32, face: f32, up: f32) -> bool {
        if on_face(dist, face) && up >= WALKABLE_NORMAL_Z {
            return true;
        }
        if self.nearest < dist - face {
            self.nearest = dist - face;
            self.nearest_up = up;
        }
        false
    }
}

/// The brush face rule: the brush's six box faces (low then high side, x
/// then y then z), then its other faces (`[nx, ny, nz, dist]`). `point` and
/// `up` are in the brush's own space.
pub fn brush_face_walkable(
    mins: [f32; 3],
    maxs: [f32; 3],
    sides: &[[f32; 4]],
    point: [f32; 3],
    up: [f32; 3],
) -> bool {
    let mut vote = FaceVote::new();
    for a in 0..3 {
        if vote.face(-point[a], -mins[a], -up[a]) || vote.face(point[a], maxs[a], up[a]) {
            return true;
        }
    }
    for side in sides {
        let n = [side[0], side[1], side[2]];
        if vote.face(dot(point, n), side[3], dot(up, n)) {
            return true;
        }
    }
    vote.nearest_up >= WALKABLE_NORMAL_Z
}

/// The terrain face rule, over triangles (skipping ones too thin to have a
/// direction). No triangles: not walkable.
pub fn triangles_face_walkable(tris: &[[[f32; 3]; 3]], point: [f32; 3], up: [f32; 3]) -> bool {
    let mut vote = FaceVote::new();
    for [v0, v1, v2] in tris {
        let n = cross(sub(*v0, *v2), sub(*v0, *v1));
        let len = libm::sqrtf(dot(n, n));
        if len <= 0.0001 {
            continue;
        }
        let inv = 1.0 / len;
        if vote.face(dot(point, n) * inv, dot(*v0, n) * inv, dot(up, n) * inv) {
            return true;
        }
    }
    !tris.is_empty() && vote.nearest_up >= WALKABLE_NORMAL_Z
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAND_MINS: [f32; 3] = [-15.0, -15.0, 0.0];
    const STAND_MAXS: [f32; 3] = [15.0, 15.0, 70.0];

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn close3(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|i| close(a[i], b[i]))
    }

    /// Standing: 14.625 flat sides, 34.625 half height, middle 35 up;
    /// lying down: same width, middle 15 up.
    #[test]
    fn the_player_shape_matches_bo2s_numbers() {
        let stand = player_hull(STAND_MINS, STAND_MAXS, 0.0);
        assert_eq!(stand.center, [0.0, 0.0, 35.0]);
        assert_eq!(stand.half_height, 34.625);
        assert_eq!(stand.capsule_radius, 14.625);
        assert!(close(stand.polygon_radius, 14.625 / 0.965_925_8));
        assert_eq!((stand.head_offset, stand.foot_offset), (10.0, 2.0));
        assert_eq!(stand.skin(), 0.5);
        let prone = player_hull(STAND_MINS, [15.0, 15.0, 30.0], 0.0);
        assert_eq!(prone.center[2], 15.0);
        assert_eq!((prone.half_height, prone.capsule_radius), (14.625, 14.625));
        // A box only 20 tall: the width follows the height.
        let short = player_hull(STAND_MINS, [15.0, 15.0, 20.0], 0.0);
        assert_eq!(short.capsule_radius, (20.0 - 0.75) * 0.5);
        // The height dvar lifts the middle.
        assert_eq!(player_hull(STAND_MINS, STAND_MAXS, 3.0).center[2], 38.0);
    }

    /// Feet are the bottom 2.125 units standing (2.0 cone minus a 0.25 skin
    /// share); as a capsule, a unit less than its radius.
    #[test]
    fn feet_are_the_bottom_two_and_an_eighth() {
        let mut stand = player_hull(STAND_MINS, STAND_MAXS, 0.0);
        assert!(stand.is_foot(2.125));
        assert!(!stand.is_foot(2.13));
        stand.capsule = true;
        assert!(stand.is_foot(0.375 + 14.625 - 1.0));
        assert!(!stand.is_foot(0.375 + 14.625 - 0.99));
        assert_eq!(stand.skin(), 15.125);
    }

    /// Sideways the ring corner wins; straight up and down the cap's tip;
    /// a heading between corners rounds to the nearer one.
    #[test]
    fn the_shape_reaches_out_like_bo2s() {
        let h = player_hull(STAND_MINS, STAND_MAXS, 0.0);
        let r = h.polygon_radius;
        assert!(close3(h.support([1.0, 0.0, 0.1]), [r, 0.0, 69.625 - 10.0]));
        assert!(close3(h.support([1.0, 0.0, -0.1]), [r, 0.0, 0.375 + 2.0]));
        assert!(close3(h.support([0.0, 0.0, 1.0]), [0.0, 0.0, 69.625]));
        assert!(close3(h.support([0.0, 0.0, -1.0]), [0.0, 0.0, 0.375]));
        // 40 degrees rounds to the 30-degree corner, -100 to 270.
        let d40 = [0.766_044_4, 0.642_787_6, 0.0];
        assert!(close3(h.support(d40), [r * 0.866_025_4, r * 0.5, 59.625]));
        let d = [-0.173_648_2, -0.984_807_8, 0.0];
        assert!(close3(h.support(d), [0.0, -r, 59.625]));
        // Steeply down: the cone under the feet is only 2 tall, so the ring
        // wins unless the direction is almost straight down.
        let steep = [0.2, 0.0, -1.0];
        assert!(close3(h.support(steep), [r, 0.0, 2.375]));
        let capsule = Hull { capsule: true, ..h };
        assert!(close3(capsule.support([1.0, 0.0, 0.5]), [0.0, 0.0, 35.0 + 20.0]));
        assert!(close3(capsule.support([1.0, 0.0, -0.5]), [0.0, 0.0, 35.0 - 20.0]));
    }

    fn plane(normal: [f32; 3], rs: f32) -> SolvePlane {
        SolvePlane { normal, rs, active: true }
    }

    /// Nothing in the way: the move is kept.
    #[test]
    fn a_clear_move_is_kept() {
        let v = [100.0, 50.0, -10.0];
        assert_eq!(project(v, &[plane([0.0, 0.0, 1.0], -20.0)]), v);
        assert_eq!(project(v, &[]), v);
        let off = SolvePlane { active: false, ..plane([0.0, 0.0, 1.0], 0.0) };
        assert_eq!(project(v, &[off]), v);
    }

    /// Into the floor: the down part goes, the rest is kept.
    #[test]
    fn the_floor_takes_the_down_part() {
        let out = project([100.0, 50.0, -30.0], &[plane([0.0, 0.0, 1.0], 0.0)]);
        assert!(close3(out, [100.0, 50.0, 0.0]), "{out:?}");
    }

    /// Into a wall standing on the floor: slides along both.
    #[test]
    fn the_floor_and_a_wall_slide_along_the_corner() {
        let planes = [plane([0.0, 0.0, 1.0], 0.0), plane([-1.0, 0.0, 0.0], 0.0)];
        let out = project([100.0, 50.0, -30.0], &planes);
        assert!(close3(out, [0.0, 50.0, 0.0]), "{out:?}");
    }

    /// Two walls meeting at 90 degrees, running into the corner: the move
    /// stops, except up the crease.
    #[test]
    fn a_corner_keeps_only_the_crease() {
        let planes = [plane([-1.0, 0.0, 0.0], 0.0), plane([0.0, -1.0, 0.0], 0.0)];
        let out = project([100.0, 80.0, 5.0], &planes);
        assert!(close3(out, [0.0, 0.0, 5.0]), "{out:?}");
    }

    /// A floor and two walls: everything stops.
    #[test]
    fn floor_and_two_walls_stop_everything() {
        let planes = [
            plane([0.0, 0.0, 1.0], 0.0),
            plane([-1.0, 0.0, 0.0], 0.0),
            plane([0.0, -1.0, 0.0], 0.0),
        ];
        let out = project([100.0, 80.0, -30.0], &planes);
        assert!(close3(out, [0.0, 0.0, 0.0]), "{out:?}");
    }

    /// Into a crease of two 45-degree walls (a V): the answer runs along the
    /// crease, the smallest change that clears both.
    #[test]
    fn a_v_crease_is_solved_with_both_planes() {
        let s = core::f32::consts::FRAC_1_SQRT_2;
        let planes = [plane([-s, 0.0, s], 0.0), plane([-s, 0.0, -s], 0.0)];
        let out = project([10.0, 3.0, 0.0], &planes);
        assert!(close3(out, [0.0, 3.0, 0.0]), "{out:?}");
        // Each plane alone would leave the other cut: the pair wins.
        for p in planes {
            assert!(dot(p.normal, out) - p.rs > -1e-4);
        }
    }

    /// A plane the start already clears is never pushed toward.
    #[test]
    fn a_plane_behind_is_left_alone() {
        let planes = [plane([0.0, 0.0, 1.0], 0.0), plane([1.0, 0.0, 0.0], -500.0)];
        let out = project([100.0, 0.0, -30.0], &planes);
        assert!(close3(out, [100.0, 0.0, 0.0]), "{out:?}");
    }

    /// A positive rs pushes the move out by that much (a touch distance).
    #[test]
    fn rs_sets_how_far_out_the_answer_lands() {
        let out = project([0.0, 0.0, -1.0], &[plane([0.0, 0.0, 1.0], 0.25)]);
        assert!(close3(out, [0.0, 0.0, 0.25]), "{out:?}");
    }

    #[test]
    fn steep_contacts_ask_the_face() {
        assert!(!is_walkable(false, 1.0, || true));
        assert!(is_walkable(true, 0.7, || false));
        assert!(!is_walkable(true, 0.69, || false));
        assert!(is_walkable(true, 0.2, || true));
    }

    /// A box brush 64 wide, top at z 32: a contact on the top edge is
    /// walkable even if the contact's normal leans off the side; a contact
    /// halfway down the side is not.
    #[test]
    fn the_brush_face_rule_counts_the_top_edge() {
        let (mins, maxs) = ([-32.0, -32.0, 0.0], [32.0, 32.0, 32.0]);
        let up = [0.0, 0.0, 1.0];
        assert!(brush_face_walkable(mins, maxs, &[], [32.0, 0.0, 32.0], up));
        assert!(!brush_face_walkable(mins, maxs, &[], [32.0, 0.0, 16.0], up));
        // A 60-degree ramp side cut across the box's top corner: on the ramp
        // only (below the top), the steep ramp decides: not walkable.
        let n = [0.866_025_4, 0.0, 0.5];
        let ramp = [n[0], n[1], n[2], dot(n, [28.0, 0.0, 32.0])];
        let on_ramp = [30.0, 0.0, 32.0 - 2.0 * 0.866_025_4 / 0.5];
        assert!(!brush_face_walkable(mins, maxs, &[ramp], on_ramp, up));
        // A gentle 30-degree ramp side: walkable.
        let n = [0.5, 0.0, 0.866_025_4];
        let ramp = [n[0], n[1], n[2], dot(n, [28.0, 0.0, 32.0])];
        let on_ramp = [30.0, 0.0, 32.0 - 2.0 * 0.5 / 0.866_025_4];
        assert!(brush_face_walkable(mins, maxs, &[ramp], on_ramp, up));
    }

    /// Terrain: a flat triangle under the point is walkable, a wall triangle
    /// is not, and no triangles is not.
    #[test]
    fn the_terrain_face_rule() {
        let up = [0.0, 0.0, 1.0];
        // Wound so (v0 - v2) x (v0 - v1) points up.
        let floor = [[0.0, 0.0, 0.0], [0.0, 10.0, 0.0], [10.0, 0.0, 0.0]];
        let wall = [[0.0, 0.0, 0.0], [0.0, 10.0, 0.0], [0.0, 0.0, 10.0]];
        assert!(triangles_face_walkable(&[floor], [2.0, 2.0, 0.0], up));
        // Wound the other way the triangle faces down: not walkable.
        let down = [floor[0], floor[2], floor[1]];
        assert!(!triangles_face_walkable(&[down], [2.0, 2.0, 0.0], up));
        assert!(!triangles_face_walkable(&[wall], [0.0, 2.0, 2.0], up));
        assert!(!triangles_face_walkable(&[], [0.0, 0.0, 0.0], up));
    }
}
