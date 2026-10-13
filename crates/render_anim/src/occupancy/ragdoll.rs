<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/render_anim/src/occupancy/ragdoll.rs
//! bo2mp: a dead player's body going limp, as BO2's own ragdoll does it.
//! Once the server says the body is a ragdoll (its corpse's trajectory
//! turns to one: the script's `startragdoll` and `launchragdoll`), the body
//! leaves its death animation where it is and falls under its own weight,
//! at the speed the animation had it moving.
//!
//! The body is the game's ragdoll definition 0 (`ragdoll.cfg`, read by
//! `sim::t6_ragdoll`). Each of its bodies is a stick between two joints of
//! the skeleton (an upper arm from shoulder to elbow); a body that others
//! hang from at points along it (the torso, which the neck, shoulders and
//! hips hang from) is held rigid. Joints keep to the file's limits: a hinge
//! (elbow, knee) bends one way only, within its range; a swivel (neck,
//! shoulder, hip) swings within its box. Twist along a limb is not
//! simulated: each limb turns with the body it hangs from. Bodies in the
//! file's self pairs stay apart; every joint stays out of the world's
//! brushes and meshes (a box about it a little smaller than its body),
//! sliding on what it rests on with its body's friction.
//!
//! Position-based steps of 1/90 s, gravity 800. The body sleeps (stops and
//! keeps its last pose) once nothing on it moves faster than 4 units a
//! second for 0.4 s, or after 10 s.

use bevy::prelude::*;
use sim::t6_ragdoll::{RagdollDef, RagdollJointKind};

const GRAVITY: f32 = 800.0;
const STEP: f32 = 1.0 / 90.0;
const MAX_STEPS_PER_FRAME: u32 = 6;
const ITERATIONS: usize = 8;
/// What a body or a loose prop lands on: solid, glass and item clip. A Black
/// Ops II map's ground can be clip-only (Nuketown's has no solid bit), so the
/// solid bit alone lets a body fall through it.
pub(crate) const MASK_PHYS_WORLD: u32 = 0xc11;
const DRAG_PER_SECOND: f32 = 0.5;
const MAX_SPEED: f32 = 2000.0;
const START_SPEED_MAX: f32 = 600.0;
const SLEEP_SPEED: f32 = 4.0;
const SLEEP_SECONDS: f32 = 0.4;
const MAX_LIFE_SECONDS: f32 = 10.0;
/// A joint's box against the world, against its body's radius.
const BOX_OF_RADIUS: f32 = 0.85;
/// How far past the pose it starts in a joint's limit lets go (degrees).
const START_MARGIN_DEG: f32 = 3.0;
/// How far a hinge may lean off its plane from the pose it starts in.
const HINGE_LEAN_DEG: f32 = 5.0;
/// How hard a joint limit pulls back per pass (0..1).
const LIMIT_STIFFNESS: f32 = 0.5;
/// `launchragdoll`'s vector as a speed (units a second). Not yet measured
/// against BO2's own throws.
const LAUNCH_SCALE: f32 = 4.0;

#[derive(Clone, Debug)]
struct Body {
    p0: usize,
    p1: usize,
    radius: f32,
    parent: Option<usize>,
    /// The skeleton bone at its start joint.
    bone: usize,
    /// Its direction (start joint to end joint) and length when it began.
    dir_start: Vec3,
    len: f32,
    /// Its joints, with those of the bodies hanging from points along it;
    /// more than two make it rigid, kept to where they began (`rest`).
    cluster: Vec<usize>,
    rest: Vec<Vec3>,
    rest_center: Vec3,
}

impl Body {
    fn rigid(&self) -> bool {
        self.cluster.len() > 2
    }
}

#[derive(Clone, Debug)]
struct Limit {
    body: usize,
    /// The skeleton parent of the body's start joint, and the body that
    /// carries that bone.
    parent_bone: usize,
    parent_body: usize,
    /// The joint's turn against its parent in the bind pose.
    rel_bind: Quat,
    /// The body's direction in its own joint's frame.
    e: Vec3,
    /// Per axis (y, z): the sign the file's angles take on this skeleton,
    /// and the (least, most) angle in radians, widened to the start pose.
    y: Option<(f32, f32, f32)>,
    z: Option<(f32, f32, f32)>,
}

#[derive(Clone, Debug)]
struct Rig {
    bodies: Vec<Body>,
    limits: Vec<Limit>,
    /// Bodies kept apart: (a, b, the least distance between their sticks).
    pairs: Vec<(usize, usize, f32)>,
    inv_mass: Vec<f32>,
    /// Each joint's box against the world (half its size).
    half: Vec<f32>,
    friction: Vec<f32>,
    x_start: Vec<Vec3>,
    /// Every bone where it began (world), and its turn.
    bone_start: Vec<Mat4>,
    rot_start: Vec<Quat>,
    /// The body each bone moves with.
    owner: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct Contact {
    normal: Vec3,
    point: Vec3,
    /// Its speed into the surface when it met it (negative going in).
    speed_in: f32,
}

/// One dead body going limp.
#[derive(Clone, Debug)]
pub struct Ragdoll {
    rig: Rig,
    x: Vec<Vec3>,
    /// The joints one step back, to draw between steps.
    x_prev: Vec<Vec3>,
    v: Vec<Vec3>,
    /// Each rigid body's turn from where it began (the last fit).
    q: Vec<Quat>,
    carry: f32,
    age: f32,
    calm: f32,
    asleep: bool,
    /// The ground when there is no world to land on: just under its
    /// lowest joint at the start.
    floor: f32,
    slept_pose: Option<(Mat4, Vec<Mat4>)>,
    note: String,
}

/// The definition's joints in order (each body's start, then its end).
fn particle_names(def: &RagdollDef) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for b in &def.bodies {
        for n in [b.joint.as_str(), b.child.as_str()] {
            if !names.contains(&n) {
                names.push(n);
            }
        }
    }
    names
}

fn particle_bones(def: &RagdollDef, dobj: &xmodel_runtime::DObj) -> Option<Vec<usize>> {
    particle_names(def).iter().map(|n| dobj.find(n)).collect()
}

/// Where the ragdoll's joints are in the world in this pose: what a body
/// still in its death animation keeps from frame to frame, so its ragdoll
/// starts at the speed it was moving.
pub fn joint_points(
    def: &RagdollDef,
    dobj: &xmodel_runtime::DObj,
    world: &[Mat4],
    world_from_local: Mat4,
) -> Option<Vec<Vec3>> {
    particle_bones(def, dobj)?
        .iter()
        .map(|&b| world.get(b).map(|m| world_from_local.transform_point3(m.w_axis.truncate())))
        .collect()
}

impl Ragdoll {
    /// Starts from the body as drawn this frame (its bones in model space,
    /// placed by `world_from_local`), moving as it moved since `last` (its
    /// joints' world points `dt` seconds ago), plus `launch` (the script's
    /// `launchragdoll` vector). None when the skeleton lacks a joint the
    /// definition names.
    pub fn start(
        def: &RagdollDef,
        dobj: &xmodel_runtime::DObj,
        world: &[Mat4],
        world_from_local: Mat4,
        last: Option<(&[Vec3], f32)>,
        launch: Vec3,
        clip: Option<&asset_world::ClipCollision>,
    ) -> Option<Self> {
        let names = particle_names(def);
        let pbones = particle_bones(def, dobj)?;
        let count = dobj.bones.len().min(world.len());
        if def.bodies.is_empty() || pbones.iter().any(|&b| b >= count) {
            return None;
        }
        let n = names.len();
        let bone_start: Vec<Mat4> = world[..count].iter().map(|m| world_from_local * *m).collect();
        let rot_start: Vec<Quat> = bone_start.iter().map(|m| m.to_scale_rotation_translation().1).collect();
        let x: Vec<Vec3> = pbones.iter().map(|&b| bone_start[b].w_axis.truncate()).collect();
        let index = |name: &str| names.iter().position(|n| *n == name);

        let mut bodies: Vec<Body> = Vec::with_capacity(def.bodies.len());
        for b in &def.bodies {
            let (p0, p1) = (index(&b.joint)?, index(&b.child)?);
            let d = x[p1] - x[p0];
            bodies.push(Body {
                p0,
                p1,
                radius: b.radius.max(0.5),
                parent: b.parent,
                bone: pbones[p0],
                dir_start: d.try_normalize().unwrap_or(Vec3::Z),
                len: d.length(),
                cluster: vec![p0, p1],
                rest: Vec::new(),
                rest_center: Vec3::ZERO,
            });
        }
        for i in 0..bodies.len() {
            let hung: Vec<usize> = bodies.iter().filter(|c| c.parent == Some(i)).map(|c| c.p0).collect();
            for p in hung {
                if !bodies[i].cluster.contains(&p) {
                    bodies[i].cluster.push(p);
                }
            }
        }

        // A rigid body's mass is spread over its joints; a stick's is split
        // between its ends.
        let mut mass = vec![0.0f32; n];
        for (b, src) in bodies.iter().zip(&def.bodies) {
            let m = src.mass.max(0.0);
            if b.rigid() {
                for &p in &b.cluster {
                    mass[p] += m / b.cluster.len() as f32;
                }
            } else {
                mass[b.p0] += m * 0.5;
                mass[b.p1] += m * 0.5;
            }
        }
        let inv_mass: Vec<f32> = mass.iter().map(|m| 1.0 / m.max(0.05)).collect();
        for b in &mut bodies {
            if b.rigid() {
                b.rest = b.cluster.iter().map(|&p| x[p]).collect();
                b.rest_center = weighted_center(&b.cluster, &b.rest, &inv_mass);
            }
        }

        let mut radius = vec![0.0f32; n];
        let mut friction = vec![0.0f32; n];
        for (b, src) in bodies.iter().zip(&def.bodies) {
            for p in [b.p0, b.p1] {
                radius[p] = radius[p].max(b.radius);
                friction[p] = friction[p].max(src.friction.max(0.0));
            }
        }
        let mut half: Vec<f32> = radius.iter().map(|r| (r * BOX_OF_RADIUS).max(1.0)).collect();
        // A joint the start pose has in a wall or the floor gets a smaller
        // box, so it does not fall through what it lies in.
        if let Some(clip) = clip {
            for (p, h) in half.iter_mut().enumerate() {
                let at = x[p].to_array();
                while *h > 0.25 && clip.sweep_box(at, at, [-*h; 3], [*h; 3], MASK_PHYS_WORLD).startsolid {
                    *h *= 0.5;
                }
            }
        }

        // Each bone moves with the nearest body that starts at it or above it.
        let mut owner = vec![0usize; count];
        for (i, own) in owner.iter_mut().enumerate() {
            let mut at = Some(i);
            for _ in 0..count {
                let Some(k) = at else { break };
                if let Some(b) = bodies.iter().position(|b| b.bone == k) {
                    *own = b;
                    break;
                }
                at = dobj.bones[k].parent.filter(|&p| p < count);
            }
        }

        let bind_rot = |i: usize| dobj.bones[i].bind_world.to_scale_rotation_translation().1;
        let mut limits = Vec::new();
        let mut notes = Vec::new();
        for j in &def.joints {
            let Some(b) = bodies.get(j.body) else { continue };
            let Some(parent_bone) = dobj.bones[b.bone].parent.filter(|&p| p < count) else {
                continue;
            };
            let rel_bind = bind_rot(parent_bone).inverse() * bind_rot(b.bone);
            let e = (rot_start[b.bone].inverse() * b.dir_start).normalize();
            let frame = rot_start[parent_bone] * rel_bind;
            let s0 = swing(e, (frame.inverse() * b.dir_start).normalize());
            let hinge = j.kind == RagdollJointKind::Hinge;
            let z = j.limits[2].map(|(lo, hi)| fit_axis(lo, hi, s0.z));
            let y = if hinge {
                let lean = HINGE_LEAN_DEG.to_radians();
                Some((1.0, s0.y - lean, s0.y + lean))
            } else {
                j.limits[1].map(|(lo, hi)| fit_axis(lo, hi, s0.y))
            };
            let sign = |a: Option<(f32, f32, f32)>| match a {
                Some((s, _, _)) if s < 0.0 => "-",
                Some(_) => "+",
                None => "free",
            };
            notes.push(format!(
                "{} {} y{} z{} (starts y {:.0} z {:.0})",
                def.bodies[j.body].joint,
                if hinge { "hinge" } else { "swivel" },
                sign(y),
                sign(z),
                s0.y.to_degrees(),
                s0.z.to_degrees()
            ));
            limits.push(Limit {
                body: j.body,
                parent_bone,
                parent_body: owner[parent_bone],
                rel_bind,
                e,
                y,
                z,
            });
        }

        let mut pairs = Vec::new();
        for &(a, b) in &def.self_pairs {
            let (Some(ba), Some(bb)) = (bodies.get(a), bodies.get(b)) else { continue };
            if [ba.p0, ba.p1].iter().any(|p| *p == bb.p0 || *p == bb.p1) {
                continue;
            }
            let (s, t) = closest_on_sticks(x[ba.p0], x[ba.p1], x[bb.p0], x[bb.p1]);
            let gap = (x[ba.p0].lerp(x[ba.p1], s) - x[bb.p0].lerp(x[bb.p1], t)).length();
            pairs.push((a, b, (ba.radius + bb.radius).min(gap)));
        }

        let mut v = vec![Vec3::ZERO; n];
        if let Some((prev, dt)) = last
            && prev.len() == n
            && (0.004..=0.1).contains(&dt)
        {
            for (vi, (now, then)) in v.iter_mut().zip(x.iter().zip(prev)) {
                *vi = ((*now - *then) / dt).clamp_length_max(START_SPEED_MAX);
            }
        }
        for vi in &mut v {
            *vi += launch * LAUNCH_SCALE;
        }
        let floor = x.iter().map(|p| p.z).fold(f32::MAX, f32::min) - 1.0;
        let note = format!(
            "bo2mp ragdoll: {} bodies, {} joints, {} pairs; limits on this skeleton: {}",
            bodies.len(),
            limits.len(),
            pairs.len(),
            notes.join("; ")
        );
        let rigs = bodies.len();
        Some(Self {
            rig: Rig {
                bodies,
                limits,
                pairs,
                inv_mass,
                half,
                friction,
                x_start: x.clone(),
                bone_start,
                rot_start,
                owner,
            },
            x_prev: x.clone(),
            x,
            v,
            q: vec![Quat::IDENTITY; rigs],
            carry: 0.0,
            age: 0.0,
            calm: 0.0,
            asleep: false,
            floor,
            slept_pose: None,
            note,
        })
    }

    /// What its joints' limits came out as on this skeleton (for the log).
    pub fn note(&self) -> &str {
        &self.note
    }

    pub fn asleep(&self) -> bool {
        self.asleep
    }

    /// Throws it by `launch` (a `launchragdoll` after it went limp).
    pub fn push(&mut self, launch: Vec3) {
        for v in &mut self.v {
            *v += launch * LAUNCH_SCALE;
        }
        self.asleep = false;
        self.slept_pose = None;
        self.calm = 0.0;
    }

    /// Moves it on by `dt` seconds.
    pub fn step(&mut self, dt: f32, clip: Option<&asset_world::ClipCollision>) {
        if self.asleep {
            return;
        }
        self.carry += dt.clamp(0.0, 0.1);
        let mut steps = 0;
        while self.carry >= STEP && steps < MAX_STEPS_PER_FRAME {
            self.x_prev.clone_from(&self.x);
            self.substep(STEP, clip);
            self.carry -= STEP;
            steps += 1;
            if self.asleep {
                self.carry = 0.0;
                break;
            }
        }
        self.carry = self.carry.min(STEP);
    }

    fn substep(&mut self, h: f32, clip: Option<&asset_world::ClipCollision>) {
        let rig = &self.rig;
        let n = self.x.len();
        let old = self.x.clone();
        let drag = (1.0 - DRAG_PER_SECOND * h).max(0.0);
        let mut p = Vec::with_capacity(n);
        let mut contacts: Vec<Option<Contact>> = vec![None; n];
        for i in 0..n {
            let mut v = self.v[i];
            v.z -= GRAVITY * h;
            v = (v * drag).clamp_length_max(MAX_SPEED);
            self.v[i] = v;
            let (at, hit) = sweep(clip, self.floor, rig.half[i], old[i], old[i] + v * h);
            p.push(at);
            contacts[i] = hit.map(|normal| Contact { normal, point: at, speed_in: v.dot(normal) });
        }
        let swept = p.clone();
        for _ in 0..ITERATIONS {
            solve_sticks(rig, &mut p);
            solve_rigid(rig, &mut p, &mut self.q);
            solve_limits(rig, &mut p, &self.q);
            solve_pairs(rig, &mut p);
            for (pi, c) in p.iter_mut().zip(&contacts) {
                if let Some(c) = c {
                    let d = (*pi - c.point).dot(c.normal);
                    if d < 0.0 {
                        *pi -= c.normal * d;
                    }
                }
            }
        }
        // What the passes moved, through the world: every move, however
        // small (a joint nudged a fraction of a unit under a floor's face
        // is under it, and the next step falls through).
        for i in 0..n {
            if (p[i] - swept[i]).length_squared() > 1e-6 {
                let (at, hit) = sweep(clip, self.floor, rig.half[i], old[i], p[i]);
                if let Some(normal) = hit {
                    p[i] = at;
                    if contacts[i].is_none() {
                        contacts[i] = Some(Contact { normal, point: at, speed_in: self.v[i].dot(normal) });
                    }
                }
            }
        }
        if p.iter().any(|pi| !pi.is_finite()) {
            // Never a body flung to nowhere: it stops where it was.
            diag::warn!(World, "bo2mp ragdoll: a step went wrong; the body stops where it was");
            self.v.iter_mut().for_each(|v| *v = Vec3::ZERO);
            self.asleep = true;
            return;
        }
        // Speeds from the moves; what touches the world stops going into
        // it and drags along it (by how hard it pressed in).
        let mut top = 0.0f32;
        for i in 0..n {
            let mut v = (p[i] - old[i]) / h;
            if let Some(c) = contacts[i] {
                let vn = v.dot(c.normal);
                if vn < 0.0 {
                    v -= c.normal * vn;
                }
                let vt = v - c.normal * v.dot(c.normal);
                let speed = vt.length();
                if speed > 1e-4 {
                    let cut = rig.friction[i] * (-c.speed_in).max(0.0);
                    v -= vt * (cut.min(speed) / speed);
                }
            }
            top = top.max(v.length());
            self.v[i] = v;
        }
        self.x = p;
        self.age += h;
        self.calm = if top < SLEEP_SPEED { self.calm + h } else { 0.0 };
        if (self.calm >= SLEEP_SECONDS && self.age > 0.5) || self.age > MAX_LIFE_SECONDS {
            self.asleep = true;
            self.v.iter_mut().for_each(|v| *v = Vec3::ZERO);
            self.x_prev.clone_from(&self.x);
        }
    }

    /// The body as it lies now: every bone of the skeleton in model space
    /// (written into `world`, which holds the same skeleton) and where that
    /// model sits (at its root joint, unturned).
    pub fn pose(&mut self, world: &mut [Mat4]) -> Mat4 {
        if let Some((placed, bones)) = &self.slept_pose {
            let count = world.len().min(bones.len());
            world[..count].copy_from_slice(&bones[..count]);
            return *placed;
        }
        let rig = &self.rig;
        let alpha = if self.asleep { 1.0 } else { (self.carry / STEP).clamp(0.0, 1.0) };
        let p: Vec<Vec3> = self.x_prev.iter().zip(&self.x).map(|(a, b)| a.lerp(*b, alpha)).collect();
        let mut q = self.q.clone();
        for (i, b) in rig.bodies.iter().enumerate().filter(|(_, b)| b.rigid()) {
            q[i] = fit_turn(rig, b, &p, q[i]);
        }
        let turns = body_turns(rig, &p, &q);
        let root = p[rig.bodies[0].p0];
        let count = world.len().min(rig.bone_start.len());
        for (i, bone) in world.iter_mut().enumerate().take(count) {
            let own = rig.owner[i];
            let b = &rig.bodies[own];
            *bone = Mat4::from_translation(p[b.p0] - root)
                * Mat4::from_quat(turns[own])
                * Mat4::from_translation(-rig.x_start[b.p0])
                * rig.bone_start[i];
        }
        let placed = Mat4::from_translation(root);
        if self.asleep {
            self.slept_pose = Some((placed, world[..count].to_vec()));
        }
        placed
    }
}

fn weighted_center(cluster: &[usize], points: &[Vec3], inv_mass: &[f32]) -> Vec3 {
    let (sum, total) = cluster.iter().zip(points).fold((Vec3::ZERO, 0.0f32), |(s, t), (&i, p)| {
        let m = 1.0 / inv_mass[i];
        (s + *p * m, t + m)
    });
    if total > 0.0 { sum / total } else { Vec3::ZERO }
}

/// The ragdoll's ground: the world's brushes and meshes it lands on, or with no
/// world loaded a flat floor. Where a joint's box moving `from` to `to`
/// stops, and the face it stopped on.
fn sweep(
    clip: Option<&asset_world::ClipCollision>,
    floor: f32,
    half: f32,
    from: Vec3,
    to: Vec3,
) -> (Vec3, Option<Vec3>) {
    match clip {
        Some(clip) => {
            if from.distance_squared(to) < 1e-8 {
                return (to, None);
            }
            let hit = clip.sweep_box(from.to_array(), to.to_array(), [-half; 3], [half; 3], MASK_PHYS_WORLD);
            // A box that starts in something moves free until it is out,
            // but never down: down is deeper into a floor.
            if hit.startsolid {
                return match to.z < from.z {
                    true => (Vec3::new(to.x, to.y, from.z), Some(Vec3::Z)),
                    false => (to, None),
                };
            }
            if hit.fraction >= 1.0 {
                return (to, None);
            }
            let normal = Vec3::from_array(hit.normal).try_normalize().unwrap_or(Vec3::Z);
            (Vec3::from_array(hit.endpos), Some(normal))
        }
        None if to.z < floor => (Vec3::new(to.x, to.y, floor), Some(Vec3::Z)),
        None => (to, None),
    }
}

fn solve_sticks(rig: &Rig, p: &mut [Vec3]) {
    for b in rig.bodies.iter().filter(|b| !b.rigid()) {
        let (wa, wb) = (rig.inv_mass[b.p0], rig.inv_mass[b.p1]);
        let d = p[b.p1] - p[b.p0];
        let l = d.length();
        if l < 1e-4 || wa + wb <= 0.0 {
            continue;
        }
        let k = (l - b.len) / (l * (wa + wb));
        p[b.p0] += d * (k * wa);
        p[b.p1] -= d * (k * wb);
    }
}

fn solve_rigid(rig: &Rig, p: &mut [Vec3], q: &mut [Quat]) {
    for (i, b) in rig.bodies.iter().enumerate().filter(|(_, b)| b.rigid()) {
        let turn = fit_turn(rig, b, p, q[i]);
        q[i] = turn;
        let center = weighted_center(&b.cluster, &b.cluster.iter().map(|&k| p[k]).collect::<Vec<_>>(), &rig.inv_mass);
        for (k, &pi) in b.cluster.iter().enumerate() {
            p[pi] = center + turn * (b.rest[k] - b.rest_center);
        }
    }
}

/// The turn that best carries a rigid body's start shape onto its joints
/// now (from `warm`, the last one).
fn fit_turn(rig: &Rig, b: &Body, p: &[Vec3], warm: Quat) -> Quat {
    let now: Vec<Vec3> = b.cluster.iter().map(|&k| p[k]).collect();
    let center = weighted_center(&b.cluster, &now, &rig.inv_mass);
    let mut a = Mat3::ZERO;
    for (k, &pi) in b.cluster.iter().enumerate() {
        let x = (p[pi] - center) / rig.inv_mass[pi];
        let r = b.rest[k] - b.rest_center;
        a = a + Mat3::from_cols(x * r.x, x * r.y, x * r.z);
    }
    extract_rotation(a, warm)
}

/// The turn closest to `a` (Müller et al. 2016), from `q`.
fn extract_rotation(a: Mat3, mut q: Quat) -> Quat {
    for _ in 0..16 {
        let r = Mat3::from_quat(q);
        let num = r.x_axis.cross(a.x_axis) + r.y_axis.cross(a.y_axis) + r.z_axis.cross(a.z_axis);
        let den = (r.x_axis.dot(a.x_axis) + r.y_axis.dot(a.y_axis) + r.z_axis.dot(a.z_axis)).abs() + 1e-9;
        let w = num / den;
        let angle = w.length();
        if angle < 1e-7 {
            break;
        }
        q = (Quat::from_axis_angle(w / angle, angle) * q).normalize();
    }
    q
}

/// Each body's turn from where it began: a rigid body's from its fit, a
/// stick's the least turn from where its parent carried it to where it
/// points now.
fn body_turns(rig: &Rig, p: &[Vec3], q: &[Quat]) -> Vec<Quat> {
    let mut turns: Vec<Quat> = Vec::with_capacity(rig.bodies.len());
    for (i, b) in rig.bodies.iter().enumerate() {
        let parent = b.parent.and_then(|k| turns.get(k).copied()).unwrap_or(Quat::IDENTITY);
        let turn = if b.rigid() {
            q[i]
        } else {
            let carried = (parent * b.dir_start).normalize();
            match (p[b.p1] - p[b.p0]).try_normalize() {
                Some(now) => Quat::from_rotation_arc(carried, now) * parent,
                None => parent,
            }
        };
        turns.push(turn);
    }
    turns
}

fn solve_limits(rig: &Rig, p: &mut [Vec3], q: &[Quat]) {
    let turns = body_turns(rig, p, q);
    for l in &rig.limits {
        let b = &rig.bodies[l.body];
        let frame = turns[l.parent_body] * rig.rot_start[l.parent_bone] * l.rel_bind;
        let d = p[b.p1] - p[b.p0];
        let len = d.length();
        if len < 1e-3 {
            continue;
        }
        let s = swing(l.e, (frame.inverse() * (d / len)).normalize());
        let mut t = s;
        if let Some((sign, lo, hi)) = l.y {
            t.y = sign * (sign * s.y).clamp(lo, hi);
        }
        if let Some((sign, lo, hi)) = l.z {
            t.z = sign * (sign * s.z).clamp(lo, hi);
        }
        if (t - s).length_squared() < 1e-8 {
            continue;
        }
        let t = t - l.e * t.dot(l.e);
        let target = p[b.p0] + frame * (Quat::from_scaled_axis(t) * l.e) * len;
        p[b.p1] += (target - p[b.p1]) * LIMIT_STIFFNESS;
    }
}

fn solve_pairs(rig: &Rig, p: &mut [Vec3]) {
    for &(a, b, least) in &rig.pairs {
        let (ba, bb) = (&rig.bodies[a], &rig.bodies[b]);
        let (s, t) = closest_on_sticks(p[ba.p0], p[ba.p1], p[bb.p0], p[bb.p1]);
        let d = p[ba.p0].lerp(p[ba.p1], s) - p[bb.p0].lerp(p[bb.p1], t);
        let dist = d.length();
        if dist >= least || dist < 1e-4 {
            continue;
        }
        let n = d / dist;
        let w = |i: usize, f: f32| f * f * rig.inv_mass[i];
        let sum = w(ba.p0, 1.0 - s) + w(ba.p1, s) + w(bb.p0, 1.0 - t) + w(bb.p1, t);
        if sum <= 0.0 {
            continue;
        }
        let lambda = (least - dist) / sum;
        p[ba.p0] += n * (lambda * (1.0 - s) * rig.inv_mass[ba.p0]);
        p[ba.p1] += n * (lambda * s * rig.inv_mass[ba.p1]);
        p[bb.p0] -= n * (lambda * (1.0 - t) * rig.inv_mass[bb.p0]);
        p[bb.p1] -= n * (lambda * t * rig.inv_mass[bb.p1]);
    }
}

/// The nearest points of two sticks, as how far along each (0..1).
fn closest_on_sticks(p1: Vec3, q1: Vec3, p2: Vec3, q2: Vec3) -> (f32, f32) {
    const EPS: f32 = 1e-6;
    let (d1, d2, r) = (q1 - p1, q2 - p2, p1 - p2);
    let (a, e, f) = (d1.dot(d1), d2.dot(d2), d2.dot(r));
    if a <= EPS && e <= EPS {
        return (0.0, 0.0);
    }
    if a <= EPS {
        return (0.0, (f / e).clamp(0.0, 1.0));
    }
    let c = d1.dot(r);
    if e <= EPS {
        return ((-c / a).clamp(0.0, 1.0), 0.0);
    }
    let b = d1.dot(d2);
    let denom = a * e - b * b;
    let mut s = if denom > EPS { ((b * f - c * e) / denom).clamp(0.0, 1.0) } else { 0.0 };
    let mut t = (b * s + f) / e;
    if t < 0.0 {
        t = 0.0;
        s = (-c / a).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((b - c) / a).clamp(0.0, 1.0);
    }
    (s, t)
}

/// The least turn (axis times angle, radians) that takes unit `from` to
/// unit `to`.
fn swing(from: Vec3, to: Vec3) -> Vec3 {
    let axis = from.cross(to);
    let sin = axis.length();
    let cos = from.dot(to);
    if sin < 1e-6 {
        if cos >= 0.0 {
            return Vec3::ZERO;
        }
        let other = if from.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
        return from.cross(other).normalize() * std::f32::consts::PI;
    }
    axis * (sin.atan2(cos) / sin)
}

/// A limit's sign on this skeleton and its range in radians: the file's
/// range (degrees) as it stands, or turned round when the start pose fits
/// it clearly better that way; then widened to take in the start pose.
fn fit_axis(lo: f32, hi: f32, start: f32) -> (f32, f32, f32) {
    let (lo, hi) = (lo.to_radians(), hi.to_radians());
    let miss = |v: f32| (lo - v).max(0.0) + (v - hi).max(0.0);
    let sign = if miss(-start) + 2f32.to_radians() < miss(start) { -1.0 } else { 1.0 };
    let v = sign * start;
    let margin = START_MARGIN_DEG.to_radians();
    (sign, lo.min(v - margin), hi.max(v + margin))
=======
//! bo2zm (Die Rise chat): a limp body, Black Ops II's ragdoll. The server
//! marks a body limp (`startragdoll`) and may throw it (`launchragdoll`, the
//! Trample Steam's kills); each client then drops it from the pose it had,
//! under gravity. Sixteen points stand for the body (pelvis, chest, neck,
//! head and the arm and leg joints), held at their lengths, each swept
//! against the map's clip every 17 ms step. The body is stiff at first and
//! goes limp over 3 s; every bone of the model follows the part it hangs
//! from. A body lies still (its pose kept) once most of it has stopped for a
//! quarter second, after 9 s, or when it falls fast; at most 16 move at once.

use std::collections::HashMap;

use asset_world::ClipCollision;
use bevy::math::{Mat3, Mat4, Quat, Vec3};
use bevy::prelude::Resource;
use xmodel_runtime::{DObj, DObjRagdoll};

const GRAVITY_Z: f32 = -800.0;
const MSEC_STEP: i32 = 17;
/// Steps one frame may run; a longer frame skips the rest.
const MAX_STEPS: i32 = 4;
const CONTENTS_SOLID: u32 = 1;
const ITERATIONS: usize = 8;
const DRAG: f32 = 0.03;
const BOUNCE: f32 = 0.1;
const FRICTION: f32 = 0.3;
const CONTACT_SLOP: f32 = 0.125;
/// How hard a limb holds the angle it had when the body went limp, at
/// first; it lets go over `STIFF_MS`.
const HOLD: f32 = 0.35;
const STIFF_MS: f32 = 3000.0;
/// The death animation's own motion carries on: the speed each point had
/// (from the pose a moment later), at most 600 and rising at most 50.
pub(crate) const AHEAD_S: f32 = 0.05;
const ANIM_SPEED_MAX: f32 = 600.0;
const ANIM_RISE_MAX: f32 = 50.0;
/// Speed per unit of the server's throw. The Trample Steam throws a zombie
/// with a quarter of the vector it throws a player with (1024 u/s), so 4
/// sends both off at the same speed.
const LAUNCH_SPEED: f32 = 4.0;
/// A random share of the throw on each point: the body tumbles.
const LAUNCH_TUMBLE: f32 = 0.2;
const MAX_LAUNCHES: u16 = 2;
const STILL_SPEED: f32 = 6.0;
const STILL_MS: i32 = 250;
const STILL_SHARE: f32 = 0.75;
const MAX_LIFE_MS: i32 = 9000;
const FALL_STILL_VZ: f32 = -2500.0;
const MAX_MOVING: usize = 16;
/// A body nobody drew for this long is gone (a corpse lasts 15 s).
const KEEP_UNSEEN_MS: i32 = 20_000;

const PELVIS: usize = 0;
const CHEST: usize = 1;
const NECK: usize = 2;
const HEAD: usize = 3;
const SHOULDER_L: usize = 4;
const ELBOW_L: usize = 5;
const WRIST_L: usize = 6;
const SHOULDER_R: usize = 7;
const ELBOW_R: usize = 8;
const WRIST_R: usize = 9;
const HIP_L: usize = 10;
const KNEE_L: usize = 11;
const ANKLE_L: usize = 12;
const HIP_R: usize = 13;
const KNEE_R: usize = 14;
const ANKLE_R: usize = 15;
const POINTS: usize = 16;

/// Each point's bone, how far it keeps off a wall and its weight.
const POINT_BONES: [(&str, f32, f32); POINTS] = [
    ("j_mainroot", 4.5, 1.2),
    ("j_spine4", 4.5, 1.2),
    ("j_neck", 3.0, 0.5),
    ("j_head", 3.5, 0.4),
    ("j_shoulder_le", 2.5, 0.4),
    ("j_elbow_le", 2.5, 0.35),
    ("j_wrist_le", 2.0, 0.2),
    ("j_shoulder_ri", 2.5, 0.4),
    ("j_elbow_ri", 2.5, 0.35),
    ("j_wrist_ri", 2.0, 0.2),
    ("j_hip_le", 3.0, 0.6),
    ("j_knee_le", 3.0, 0.5),
    ("j_ankle_le", 2.5, 0.3),
    ("j_hip_ri", 3.0, 0.6),
    ("j_knee_ri", 3.0, 0.5),
    ("j_ankle_ri", 2.5, 0.3),
];

/// The torso: one stiff piece (part 0, turning about the pelvis).
const TORSO: [usize; 7] = [PELVIS, CHEST, NECK, SHOULDER_L, SHOULDER_R, HIP_L, HIP_R];

/// The parts that swing, parts 1 to 9: (from point, to point, parent part),
/// parents first.
const LIMBS: [(usize, usize, usize); 9] = [
    (NECK, HEAD, 0),
    (SHOULDER_L, ELBOW_L, 0),
    (ELBOW_L, WRIST_L, 2),
    (SHOULDER_R, ELBOW_R, 0),
    (ELBOW_R, WRIST_R, 4),
    (HIP_L, KNEE_L, 0),
    (KNEE_L, ANKLE_L, 6),
    (HIP_R, KNEE_R, 0),
    (KNEE_R, ANKLE_R, 8),
];
const PARTS: usize = 10;

/// The bone each swinging part starts at: every bone under it moves with
/// it (the hand with the forearm, the foot with the shin, the head model
/// with the neck); the rest move with the torso.
fn part_of_bone(name: &str) -> Option<u8> {
    Some(match name {
        "j_neck" => 1,
        "j_shoulder_le" => 2,
        "j_elbow_le" => 3,
        "j_shoulder_ri" => 4,
        "j_elbow_ri" => 5,
        "j_hip_le" => 6,
        "j_knee_le" => 7,
        "j_hip_ri" => 8,
        "j_knee_ri" => 9,
        _ => return None,
    })
}

fn pivot(part: usize) -> usize {
    match part {
        0 => PELVIS,
        limb => LIMBS[limb - 1].0,
    }
}

/// One limp body's pose this frame.
pub(crate) struct RagdollPose {
    /// Each bone's matrix in the model's own space (as `pose_dobj` gives it).
    pub bones: Vec<Mat4>,
    /// How far the pelvis has moved since the body went limp.
    pub shift: Vec3,
    /// Changes whenever the pose does.
    pub stamp: u32,
}

#[derive(Resource, Default)]
pub(crate) struct Ragdolls {
    bodies: HashMap<u32, Slot>,
}

enum Slot {
    Body(Box<Body>),
    /// Not this skeleton (a dog), or its pelvis started in a wall: it keeps
    /// its animation.
    Refused { start_ms: i32, seen_ms: i32 },
}

struct Link {
    a: usize,
    b: usize,
    min: f32,
    max: f32,
}

struct Body {
    /// The server's mark: a new one is a new fall.
    start_ms: i32,
    /// Each bone's part.
    parts_of_bones: Vec<u8>,
    /// Each bone's world matrix when the body went limp.
    start_world: Vec<Mat4>,
    /// The points then.
    rest: [Vec3; POINTS],
    torso0: Quat,
    /// The way the body faced, in the torso's own frame: knees bend this way.
    forward_local: Vec3,
    inv_mass: [f32; POINTS],
    links: Vec<Link>,
    x: [Vec3; POINTS],
    v: [Vec3; POINTS],
    still_ms: [i32; POINTS],
    born_ms: i32,
    clock_ms: i32,
    seen_ms: i32,
    /// The server's throw count last seen, and the throws taken (at most 2).
    launches: u16,
    thrown: u16,
    steps: u32,
    lying: Option<&'static str>,
    seed: u32,
}

impl Ragdolls {
    /// How far a limp body's pelvis has moved (where to cull and light it).
    pub(crate) fn shift(&self, id: u32) -> Option<Vec3> {
        match self.bodies.get(&id)? {
            Slot::Body(body) => Some(body.x[PELVIS] - body.rest[PELVIS]),
            Slot::Refused { .. } => None,
        }
    }

    /// The body's pose now, run on to `now_ms`. `start` gives the pose
    /// when it goes limp, in the model's space: now and `AHEAD_S` later.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn pose(
        &mut self,
        id: u32,
        ragdoll: &DObjRagdoll,
        dobj: &DObj,
        world_from_local: Mat4,
        now_ms: i32,
        clip: Option<&ClipCollision>,
        start: impl FnOnce() -> Option<(Vec<Mat4>, Vec<Mat4>)>,
    ) -> Option<RagdollPose> {
        let fresh = match self.bodies.get(&id) {
            Some(Slot::Body(body)) => {
                body.start_ms != ragdoll.start_ms || body.parts_of_bones.len() != dobj.bones.len()
            }
            Some(Slot::Refused { start_ms, .. }) => *start_ms != ragdoll.start_ms,
            None => true,
        };
        if fresh {
            let made = start().ok_or("no pose").and_then(|(now, ahead)| {
                Body::start(id, ragdoll, dobj, world_from_local, &now, &ahead, clip, now_ms)
            });
            match made {
                Ok(body) => {
                    self.make_room(id);
                    diag::info!(
                        World,
                        "ragdoll start #{id}: thrown {} x{} at {}",
                        show(Vec3::from_array(ragdoll.launch)),
                        ragdoll.launches,
                        show(body.x[PELVIS])
                    );
                    self.bodies.insert(id, Slot::Body(Box::new(body)));
                }
                Err(why) => {
                    diag::info!(World, "ragdoll refused #{id}: {why}");
                    self.bodies.insert(
                        id,
                        Slot::Refused {
                            start_ms: ragdoll.start_ms,
                            seen_ms: now_ms,
                        },
                    );
                }
            }
        }
        match self.bodies.get_mut(&id)? {
            Slot::Body(body) => {
                body.seen_ms = now_ms;
                body.throw(ragdoll);
                body.run(id, now_ms, clip);
                Some(body.pose(world_from_local))
            }
            Slot::Refused { seen_ms, .. } => {
                *seen_ms = now_ms;
                None
            }
        }
    }

    /// At most `MAX_MOVING` bodies move: the oldest lies still for a new one.
    fn make_room(&mut self, id: u32) {
        loop {
            let moving = self.bodies.iter_mut().filter_map(|(other, slot)| match slot {
                Slot::Body(body) if *other != id && body.lying.is_none() => Some(body),
                _ => None,
            });
            let (count, oldest) = moving.fold((0usize, None::<&mut Box<Body>>), |(n, old), b| {
                let older = match &old {
                    Some(o) => b.born_ms < o.born_ms,
                    None => true,
                };
                (n + 1, if older { Some(b) } else { old })
            });
            match oldest {
                Some(body) if count >= MAX_MOVING => body.lying = Some("crowd"),
                _ => return,
            }
        }
    }

    /// Bodies nobody drew for a while are gone.
    pub(crate) fn forget_unseen(&mut self, now_ms: i32) {
        self.bodies.retain(|_, slot| {
            let seen = match slot {
                Slot::Body(body) => body.seen_ms,
                Slot::Refused { seen_ms, .. } => *seen_ms,
            };
            seen <= now_ms && now_ms - seen <= KEEP_UNSEEN_MS
        });
    }
}

impl Body {
    #[allow(clippy::too_many_arguments)]
    fn start(
        id: u32,
        ragdoll: &DObjRagdoll,
        dobj: &DObj,
        world_from_local: Mat4,
        now: &[Mat4],
        ahead: &[Mat4],
        clip: Option<&ClipCollision>,
        now_ms: i32,
    ) -> Result<Body, &'static str> {
        if now.len() != dobj.bones.len() || ahead.len() != now.len() {
            return Err("pose does not fit the model");
        }
        let start_world: Vec<Mat4> = now.iter().map(|m| world_from_local * *m).collect();
        let find = |name: &str| dobj.bones.iter().position(|b| b.name == name);
        let mut x = [Vec3::ZERO; POINTS];
        let mut v = [Vec3::ZERO; POINTS];
        for (k, (name, _, _)) in POINT_BONES.iter().enumerate() {
            let Some(bone) = find(name) else {
                if k == HEAD {
                    continue;
                }
                return Err("not a body's skeleton");
            };
            x[k] = start_world[bone].w_axis.truncate();
            let later = (world_from_local * ahead[bone]).w_axis.truncate();
            let mut speed = ((later - x[k]) / AHEAD_S).clamp_length_max(ANIM_SPEED_MAX);
            speed.z = speed.z.min(ANIM_RISE_MAX);
            v[k] = speed;
        }
        if find("j_head").is_none() {
            x[HEAD] = x[NECK] + (x[NECK] - x[CHEST]).normalize_or(Vec3::Z) * 6.0;
            v[HEAD] = v[NECK];
        }
        if !x.iter().chain(&v).all(|p| p.is_finite()) {
            return Err("pose not finite");
        }
        let rest = x;
        let torso0 = torso_frame(&rest).ok_or("torso folded flat")?;
        let mut links = Vec::new();
        for (i, &a) in TORSO.iter().enumerate() {
            for &b in &TORSO[i + 1..] {
                let d = rest[a].distance(rest[b]);
                links.push(Link { a, b, min: d, max: d });
            }
        }
        for &(a, b, _) in &LIMBS {
            let d = rest[a].distance(rest[b]);
            links.push(Link { a, b, min: d, max: d });
        }
        // Elbows and knees fold only so far; the head nods only so far.
        for (s, e, w, share) in [
            (SHOULDER_L, ELBOW_L, WRIST_L, 0.5),
            (SHOULDER_R, ELBOW_R, WRIST_R, 0.5),
            (HIP_L, KNEE_L, ANKLE_L, 0.4),
            (HIP_R, KNEE_R, ANKLE_R, 0.4),
        ] {
            let reach = rest[s].distance(rest[e]) + rest[e].distance(rest[w]);
            links.push(Link {
                a: s,
                b: w,
                min: reach * share,
                max: reach,
            });
        }
        let reach = rest[CHEST].distance(rest[NECK]) + rest[NECK].distance(rest[HEAD]);
        links.push(Link {
            a: CHEST,
            b: HEAD,
            min: rest[CHEST].distance(rest[HEAD]).min(reach) * 0.85,
            max: reach,
        });
        // A joint through a wall from the pelvis comes back inside it; a
        // pelvis in a wall keeps the animation.
        if let Some(clip) = clip {
            let pelvis = x[PELVIS];
            let up = (pelvis + Vec3::Z).to_array();
            if clip
                .sweep_box(pelvis.to_array(), up, [0.0; 3], [0.0; 3], CONTENTS_SOLID)
                .startsolid
            {
                return Err("pelvis in a wall");
            }
            for point in x.iter_mut().skip(1) {
                let hit = clip.sweep_box(
                    pelvis.to_array(),
                    point.to_array(),
                    [0.0; 3],
                    [0.0; 3],
                    CONTENTS_SOLID,
                );
                if hit.fraction < 1.0 {
                    *point = pelvis + (*point - pelvis) * hit.fraction * 0.8;
                }
            }
        }
        let parts_of_bones = dobj
            .bones
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let mut at = Some(i);
                while let Some(bone) = at {
                    if let Some(part) = part_of_bone(&dobj.bones[bone].name) {
                        return part;
                    }
                    at = dobj.bones[bone].parent;
                }
                0
            })
            .collect();
        let facing = world_from_local
            .x_axis
            .truncate()
            .normalize_or(Vec3::X);
        Ok(Body {
            start_ms: ragdoll.start_ms,
            parts_of_bones,
            start_world,
            rest,
            torso0,
            forward_local: torso0.inverse() * facing,
            inv_mass: POINT_BONES.map(|(_, _, mass)| 1.0 / mass),
            links,
            x,
            v,
            still_ms: [0; POINTS],
            born_ms: now_ms,
            clock_ms: now_ms,
            seen_ms: now_ms,
            launches: 0,
            thrown: 0,
            steps: 0,
            lying: None,
            seed: id.wrapping_mul(0x9E37_79B9) ^ (ragdoll.start_ms as u32) | 1,
        })
    }

    fn random(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.seed as f32 / u32::MAX as f32 * 2.0 - 1.0
    }

    /// The server's new throws (two at most per body): the same push on
    /// every point, so the light hands and head fly off faster, and a
    /// random share each, so it tumbles. A body lying still moves again.
    fn throw(&mut self, ragdoll: &DObjRagdoll) {
        let new = ragdoll.launches.wrapping_sub(self.launches);
        self.launches = ragdoll.launches;
        let take = new.min(MAX_LAUNCHES - self.thrown);
        if take == 0 {
            return;
        }
        self.thrown += take;
        let speed = Vec3::from_array(ragdoll.launch) * LAUNCH_SPEED;
        let weight: f32 = self.inv_mass.iter().map(|inv| 1.0 / inv).sum();
        let mean = weight / POINTS as f32;
        for _ in 0..take {
            for k in 0..POINTS {
                let tumble = Vec3::new(self.random(), self.random(), self.random());
                self.v[k] += speed * (mean * self.inv_mass[k])
                    + tumble * (speed.length() * LAUNCH_TUMBLE);
                self.still_ms[k] = 0;
            }
        }
        self.lying = None;
    }

    fn run(&mut self, id: u32, now_ms: i32, clip: Option<&ClipCollision>) {
        if self.lying.is_some() {
            self.clock_ms = now_ms;
            return;
        }
        let mut steps = 0;
        while self.clock_ms + MSEC_STEP <= now_ms {
            if steps == MAX_STEPS {
                self.clock_ms = now_ms;
                break;
            }
            steps += 1;
            self.clock_ms += MSEC_STEP;
            self.step(clip);
            if let Some(why) = self.settled() {
                self.lying = Some(why);
                diag::info!(
                    World,
                    "ragdoll still #{id} ({why}) after {} ms at {}, moved {:.0}",
                    self.clock_ms - self.born_ms,
                    show(self.x[PELVIS]),
                    self.x[PELVIS].distance(self.rest[PELVIS])
                );
                break;
            }
        }
    }

    fn step(&mut self, clip: Option<&ClipCollision>) {
        let dt = MSEC_STEP as f32 / 1000.0;
        let age = (self.clock_ms - self.born_ms) as f32;
        let loose = (1.0 - age / STIFF_MS).max(0.0);
        let hold = HOLD * loose * loose;
        let mut p = self.x;
        for k in 0..POINTS {
            self.v[k].z += GRAVITY_Z * dt;
            self.v[k] *= 1.0 - DRAG * dt;
            p[k] = self.x[k] + self.v[k] * dt;
        }
        let parts = self.parts(&p);
        if hold > 0.0 {
            for &(a, b, parent) in &LIMBS {
                let held = p[a] + parts[parent] * (self.rest[b] - self.rest[a]);
                p[b] += (held - p[b]) * hold;
            }
        }
        let forward = parts[0] * self.forward_local;
        for _ in 0..ITERATIONS {
            for link in &self.links {
                let delta = p[link.b] - p[link.a];
                let d = delta.length();
                if d < 1e-4 {
                    continue;
                }
                let want = d.clamp(link.min, link.max);
                if want == d {
                    continue;
                }
                let (wa, wb) = (self.inv_mass[link.a], self.inv_mass[link.b]);
                let fix = delta * ((d - want) / (d * (wa + wb)));
                p[link.a] += fix * wa;
                p[link.b] -= fix * wb;
            }
            // Knees bend forward only.
            for (hip, knee, ankle) in [(HIP_L, KNEE_L, ANKLE_L), (HIP_R, KNEE_R, ANKLE_R)] {
                let axis = p[ankle] - p[hip];
                let len2 = axis.length_squared();
                if len2 < 1e-3 {
                    continue;
                }
                let on = p[hip] + axis * ((p[knee] - p[hip]).dot(axis) / len2);
                let back = (p[knee] - on).dot(forward);
                if back < 0.0 {
                    p[knee] -= forward * back;
                }
            }
        }
        let mut touch = [None; POINTS];
        if let Some(clip) = clip {
            for k in 0..POINTS {
                let r = POINT_BONES[k].1;
                let (from, to) = (self.x[k].to_array(), p[k].to_array());
                let mut hit = clip.sweep_box(from, to, [-r; 3], [r; 3], CONTENTS_SOLID);
                if hit.startsolid {
                    hit = clip.sweep_box(from, to, [0.0; 3], [0.0; 3], CONTENTS_SOLID);
                    if hit.startsolid {
                        continue;
                    }
                }
                if hit.fraction < 1.0 {
                    let n = Vec3::from_array(hit.normal);
                    p[k] = Vec3::from_array(hit.endpos) + n * CONTACT_SLOP;
                    touch[k] = Some(n);
                }
            }
        }
        for k in 0..POINTS {
            let mut vel = (p[k] - self.x[k]) / dt;
            if let Some(n) = touch[k] {
                let vn = vel.dot(n);
                if vn < 0.0 {
                    vel -= n * ((1.0 + BOUNCE) * vn);
                }
                let along = vel - n * vel.dot(n);
                vel -= along * FRICTION;
            }
            if !vel.is_finite() || !p[k].is_finite() {
                vel = Vec3::ZERO;
                p[k] = self.x[k];
            }
            self.v[k] = vel;
            self.x[k] = p[k];
            self.still_ms[k] = if vel.length() < STILL_SPEED {
                self.still_ms[k] + MSEC_STEP
            } else {
                0
            };
        }
        self.steps = self.steps.wrapping_add(1);
    }

    fn settled(&self) -> Option<&'static str> {
        let still = self.still_ms.iter().filter(|ms| **ms >= STILL_MS).count();
        if still as f32 > POINTS as f32 * STILL_SHARE {
            return Some("still");
        }
        if self.clock_ms - self.born_ms >= MAX_LIFE_MS {
            return Some("9 s");
        }
        let fall = self.v.iter().map(|v| v.z).sum::<f32>() / POINTS as f32;
        if fall < FALL_STILL_VZ {
            return Some("falling");
        }
        None
    }

    /// Each part's turn since the body went limp: the torso from its frame,
    /// each limb carried by its parent then swung to where it points now.
    fn parts(&self, p: &[Vec3; POINTS]) -> [Quat; PARTS] {
        let mut turn = [Quat::IDENTITY; PARTS];
        if let Some(now) = torso_frame(p) {
            turn[0] = (now * self.torso0.inverse()).normalize();
        }
        for (i, &(a, b, parent)) in LIMBS.iter().enumerate() {
            let carried = turn[parent] * (self.rest[b] - self.rest[a]);
            turn[i + 1] = match (carried.try_normalize(), (p[b] - p[a]).try_normalize()) {
                (Some(from), Some(to)) => (Quat::from_rotation_arc(from, to) * turn[parent]).normalize(),
                _ => turn[parent],
            };
        }
        turn
    }

    fn pose(&self, world_from_local: Mat4) -> RagdollPose {
        let turn = self.parts(&self.x);
        let moves: [Mat4; PARTS] = std::array::from_fn(|part| {
            let at = pivot(part);
            Mat4::from_translation(self.x[at])
                * Mat4::from_quat(turn[part])
                * Mat4::from_translation(-self.rest[at])
        });
        let local_from_world = world_from_local.inverse();
        let bones = self
            .start_world
            .iter()
            .zip(&self.parts_of_bones)
            .map(|(world, part)| local_from_world * moves[*part as usize] * *world)
            .collect();
        RagdollPose {
            bones,
            shift: self.x[PELVIS] - self.rest[PELVIS],
            stamp: self.steps,
        }
    }
}

/// The torso's frame: up the spine, across the shoulders and hips.
fn torso_frame(p: &[Vec3; POINTS]) -> Option<Quat> {
    let up = (p[NECK] - p[PELVIS]).try_normalize()?;
    let across = p[SHOULDER_L] - p[SHOULDER_R] + p[HIP_L] - p[HIP_R];
    let side = (across - up * across.dot(up)).try_normalize()?;
    let forward = side.cross(up);
    Some(Quat::from_mat3(&Mat3::from_cols(forward, side, up)).normalize())
}

fn show(p: Vec3) -> String {
    format!("({:.0} {:.0} {:.0})", p.x, p.y, p.z)
>>>>>>> /tmp/claude-0/zmhead/crates/render_anim/src/occupancy/ragdoll.rs
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/render_anim/src/occupancy/ragdoll.rs
    use sim::t6_ragdoll::parse;

    /// A made-up body plan in the file's format (not the game's numbers).
    const DEF: &str = "\
        ragdoll_bone 0 j_mainroot j_neck 5 0.5 3 0.3 -1 0\n\
        ragdoll_bone 0 j_neck j_head 3 0.5 0.5 0.3 0 0\n\
        ragdoll_bone 0 j_shoulder_le j_elbow_le 2 0.5 0.5 0.3 0 0\n\
        ragdoll_bone 0 j_elbow_le j_wrist_le 2 0.5 0.5 0.3 2 0\n\
        ragdoll_bone 0 j_shoulder_ri j_elbow_ri 2 0.5 0.5 0.3 0 1\n\
        ragdoll_bone 0 j_elbow_ri j_wrist_ri 2 0.5 0.5 0.3 4 1\n\
        ragdoll_bone 0 j_hip_le j_knee_le 3 0.5 1 0.3 0 0\n\
        ragdoll_bone 0 j_knee_le j_ankle_le 3 0.5 1 0.3 6 0\n\
        ragdoll_bone 0 j_hip_ri j_knee_ri 3 0.5 1 0.3 0 1\n\
        ragdoll_bone 0 j_knee_ri j_ankle_ri 3 0.5 1 0.3 8 1\n\
        ragdoll_selfpair 0 3 5\n\
        ragdoll_selfpair 0 7 9\n\
        ragdoll_joint 0 1 swivel\n\
        ragdoll_joint 0 3 hinge\n\
        ragdoll_joint 0 7 hinge\n\
        ragdoll_limit 0 0 y 0 -40 40\n\
        ragdoll_limit 0 0 z 0 -40 40\n\
        ragdoll_limit 0 1 z 0 -150 0\n\
        ragdoll_limit 0 2 z 0 -150 0\n";

    /// A standing stick figure: every bone's x along it, pointing down.
    fn figure() -> (xmodel_runtime::DObj, Vec<Mat4>) {
        let rows: [(&str, Option<usize>, [f32; 3]); 18] = [
            ("tag_origin", None, [0.0, 0.0, 0.0]),
            ("j_mainroot", Some(0), [0.0, 0.0, 40.0]),
            ("j_spine4", Some(1), [0.0, 0.0, 55.0]),
            ("j_neck", Some(2), [0.0, 0.0, 60.0]),
            ("j_head", Some(3), [0.0, 0.0, 66.0]),
            ("j_clavicle_le", Some(2), [0.0, 3.0, 58.0]),
            ("j_shoulder_le", Some(5), [0.0, 8.0, 58.0]),
            ("j_elbow_le", Some(6), [0.0, 8.0, 46.0]),
            ("j_wrist_le", Some(7), [0.0, 8.0, 34.0]),
            ("j_clavicle_ri", Some(2), [0.0, -3.0, 58.0]),
            ("j_shoulder_ri", Some(9), [0.0, -8.0, 58.0]),
            ("j_elbow_ri", Some(10), [0.0, -8.0, 46.0]),
            ("j_wrist_ri", Some(11), [0.0, -8.0, 34.0]),
            ("j_hip_le", Some(1), [0.0, 4.0, 38.0]),
            ("j_knee_le", Some(13), [0.0, 4.0, 20.0]),
            ("j_ankle_le", Some(14), [0.0, 4.0, 2.0]),
            ("j_hip_ri", Some(1), [0.0, -4.0, 38.0]),
            ("j_knee_ri", Some(16), [0.0, -4.0, 20.0]),
        ];
        let turn = Quat::from_mat3(&Mat3::from_cols(Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y));
        let mut world: Vec<Mat4> = rows
            .iter()
            .map(|(_, _, at)| Mat4::from_rotation_translation(turn, Vec3::from_array(*at)))
            .collect();
        world.push(Mat4::from_rotation_translation(turn, Vec3::new(0.0, -4.0, 2.0)));
        let mut bones: Vec<xmodel_runtime::Bone> = rows
            .iter()
            .enumerate()
            .map(|(i, (name, parent, _))| bone(name, *parent, &world, i))
            .collect();
        bones.push(bone("j_ankle_ri", Some(17), &world, 18));
        let dobj = xmodel_runtime::DObj {
            models: vec![xmodel_runtime::ModelSlot {
                name: "figure".into(),
                base: 0,
                bone_count: bones.len(),
                scale: 1.0,
            }],
            bones,
            duplicates: Vec::new(),
        };
        (dobj, world)
    }

    fn bone(name: &str, parent: Option<usize>, world: &[Mat4], i: usize) -> xmodel_runtime::Bone {
        let local = parent.map_or(world[i], |p| world[p].inverse() * world[i]);
        let (_, rotation, translation) = local.to_scale_rotation_translation();
        xmodel_runtime::Bone {
            name: name.into(),
            model: 0,
            parent,
            bind_rotation: rotation,
            bind_translation: translation,
            bind_world: world[i],
            no_scale: false,
=======

    /// A standing body's points, facing +X, 1 unit = 1 inch.
    fn standing() -> [Vec3; POINTS] {
        let mut x = [Vec3::ZERO; POINTS];
        x[PELVIS] = Vec3::new(0.0, 0.0, 37.0);
        x[CHEST] = Vec3::new(0.0, 0.0, 52.0);
        x[NECK] = Vec3::new(0.0, 0.0, 60.0);
        x[HEAD] = Vec3::new(1.0, 0.0, 66.0);
        x[SHOULDER_L] = Vec3::new(0.0, 7.0, 57.0);
        x[ELBOW_L] = Vec3::new(0.0, 8.0, 46.0);
        x[WRIST_L] = Vec3::new(1.0, 8.0, 36.0);
        x[SHOULDER_R] = Vec3::new(0.0, -7.0, 57.0);
        x[ELBOW_R] = Vec3::new(0.0, -8.0, 46.0);
        x[WRIST_R] = Vec3::new(1.0, -8.0, 36.0);
        x[HIP_L] = Vec3::new(0.0, 5.0, 37.0);
        x[KNEE_L] = Vec3::new(0.5, 5.0, 20.0);
        x[ANKLE_L] = Vec3::new(0.0, 5.0, 5.0);
        x[HIP_R] = Vec3::new(0.0, -5.0, 37.0);
        x[KNEE_R] = Vec3::new(0.5, -5.0, 20.0);
        x[ANKLE_R] = Vec3::new(0.0, -5.0, 5.0);
        x
    }

    fn body(x: [Vec3; POINTS]) -> Body {
        let torso0 = torso_frame(&x).unwrap();
        let mut links = Vec::new();
        for (i, &a) in TORSO.iter().enumerate() {
            for &b in &TORSO[i + 1..] {
                let d = x[a].distance(x[b]);
                links.push(Link { a, b, min: d, max: d });
            }
        }
        for &(a, b, _) in &LIMBS {
            let d = x[a].distance(x[b]);
            links.push(Link { a, b, min: d, max: d });
        }
        Body {
            start_ms: 0,
            parts_of_bones: Vec::new(),
            start_world: Vec::new(),
            rest: x,
            torso0,
            forward_local: torso0.inverse() * Vec3::X,
            inv_mass: POINT_BONES.map(|(_, _, mass)| 1.0 / mass),
            links,
            x,
            v: [Vec3::ZERO; POINTS],
            still_ms: [0; POINTS],
            born_ms: 0,
            clock_ms: 0,
            seen_ms: 0,
            launches: 0,
            thrown: 0,
            steps: 0,
            lying: None,
            seed: 7,
>>>>>>> /tmp/claude-0/zmhead/crates/render_anim/src/occupancy/ragdoll.rs
        }
    }

    #[test]
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/render_anim/src/occupancy/ragdoll.rs
    fn swing_is_axis_times_angle() {
        let s = swing(Vec3::X, Vec3::Y);
        assert!((s - Vec3::Z * std::f32::consts::FRAC_PI_2).length() < 1e-5, "{s}");
        let s = swing(Vec3::NEG_X, Vec3::NEG_Y);
        assert!((s - Vec3::Z * std::f32::consts::FRAC_PI_2).length() < 1e-5, "{s}");
        assert_eq!(swing(Vec3::Z, Vec3::Z), Vec3::ZERO);
    }

    #[test]
    fn finds_a_turn_from_its_matrix() {
        let want = Quat::from_axis_angle(Vec3::new(1.0, 2.0, 3.0).normalize(), 1.2);
        let got = extract_rotation(Mat3::from_quat(want) * 7.0, Quat::IDENTITY);
        assert!(got.dot(want).abs() > 0.9999, "{got} vs {want}");
    }

    #[test]
    fn a_pushed_body_falls_and_lies_still_on_the_floor() {
        let defs = parse(DEF);
        let (dobj, world) = figure();
        let mut doll = Ragdoll::start(&defs[0], &dobj, &world, Mat4::IDENTITY, None, Vec3::new(60.0, 0.0, 0.0), None)
            .expect("the figure has every joint");
        assert_eq!(doll.rig.limits.len(), 3, "{}", doll.note());
        for _ in 0..(8 * 60) {
            doll.step(1.0 / 60.0, None);
        }
        assert!(doll.asleep(), "still moving: {:?}", doll.v);
        for (i, p) in doll.x.iter().enumerate() {
            assert!(p.is_finite() && p.z >= doll.floor - 0.5, "joint {i} at {p}");
        }
        for b in doll.rig.bodies.iter().filter(|b| !b.rigid()) {
            let len = (doll.x[b.p1] - doll.x[b.p0]).length();
            assert!((len - b.len).abs() < b.len * 0.05, "stick {} {} grew to {len}", b.p0, b.p1);
        }
        let mut bones = world.clone();
        let placed = doll.pose(&mut bones);
        let head = (placed * bones[dobj.find("j_head").unwrap()]).w_axis.truncate();
        assert!(head.z < 25.0, "the head stayed up at {head}");
        // Lying still keeps the same pose.
        let again = doll.pose(&mut bones);
        assert_eq!(placed, again);
=======
    fn a_throw_moves_the_body_at_four_times_the_vector() {
        let mut b = body(standing());
        b.throw(&DObjRagdoll {
            start_ms: 0,
            launch: [128.0, 0.0, 221.7],
            launches: 1,
        });
        let weight: f32 = b.inv_mass.iter().map(|inv| 1.0 / inv).sum();
        let centre = (0..POINTS)
            .map(|k| b.v[k] / b.inv_mass[k])
            .sum::<Vec3>()
            / weight;
        let want = Vec3::new(512.0, 0.0, 886.8);
        assert!(centre.distance(want) < want.length() * 0.15, "{centre}");
        // A third throw is not taken.
        b.throw(&DObjRagdoll {
            start_ms: 0,
            launch: [128.0, 0.0, 221.7],
            launches: 3,
        });
        assert_eq!(b.thrown, 2);
    }

    #[test]
    fn a_free_fall_keeps_its_lengths_and_falls_under_gravity() {
        let mut b = body(standing());
        for _ in 0..30 {
            b.clock_ms += MSEC_STEP;
            b.step(None);
        }
        let t = 30.0 * MSEC_STEP as f32 / 1000.0;
        let drop = b.rest[PELVIS].z - b.x[PELVIS].z;
        let want = 0.5 * 800.0 * t * t;
        assert!((drop - want).abs() < want * 0.1, "{drop} vs {want}");
        for link in &b.links {
            let d = b.x[link.a].distance(b.x[link.b]);
            assert!(d >= link.min - 0.2 && d <= link.max + 0.2);
        }
    }

    #[test]
    fn the_parts_follow_the_points() {
        let x = standing();
        let b = body(x);
        let turn = b.parts(&x);
        for q in turn {
            assert!(q.angle_between(Quat::IDENTITY) < 1e-3);
        }
        // The whole body turned a quarter about Z: every part turns with it.
        let spin = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let moved = x.map(|p| spin * p);
        for q in b.parts(&moved) {
            assert!(q.angle_between(spin) < 1e-3);
        }
    }

    #[test]
    fn bones_map_to_their_parts() {
        assert_eq!(part_of_bone("j_knee_ri"), Some(9));
        assert_eq!(part_of_bone("j_spine4"), None);
>>>>>>> /tmp/claude-0/zmhead/crates/render_anim/src/occupancy/ragdoll.rs
    }
}
