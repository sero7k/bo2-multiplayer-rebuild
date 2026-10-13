//! Script vehicles on paths: `attachpath(node)` then `startpath()` moves a
//! vehicle entity along its chain of `info_vehicle_node`s (each node's
//! `target` names the next; a chain may loop back on itself) straight from
//! node to node (BO2's curve through them takes each node's `tension`, and
//! with none set, as on every Tranzit node, it is 0: straight lines), facing
//! where it will be 1.3 seconds on (BO2's lookahead_time) and turning to it
//! eased, so it swings round each corner; between two nodes where either is
//! a rotate node it turns from one's angles to the other's instead (BO2's
//! VP_GetAngles: Tranzit's bus pulls into the Bus Depot square to the
//! building that way, though its nodes cut across sideways); `reached_node`
//! (with the node) at each node it passes, `reached_end_node` at the last
//! one. Nuketown's perk machines ride `perk_arrival_vehicle` down from the
//! sky this way (linked to it). Tranzit's bus stands on its wheels on the
//! road (BO2's VEH_GroundPlant), not at its nodes' height (28 units up).
//!
//! Speed: each node's `speed` (miles per hour, 17.6 units a second each)
//! until the script drives it with `setspeed(mph, accel, decel)` (eased,
//! miles per hour per second) or `setspeedimmediate`; `resumespeed` hands
//! it back to the nodes. Tranzit's bus is driven this way (its stop
//! schedule waits on `reached_node` and polls `getspeed`).

use std::collections::BTreeMap;

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};

use super::{Zm, arg, entnum, num, vec3, with_vm};

const MPH: f32 = 17.6;

#[derive(Clone, Debug, Default)]
pub(crate) struct VPath {
    /// The node it left last.
    pub from: u32,
    /// The node it drives to.
    pub to: u32,
    /// The node before `from` (the way it came: the curve's heading there).
    pub prev: Option<u32>,
    /// How far along the curve from `from` to `to` it is (0..1).
    pub t: f32,
    /// The way it moved last (a unit vector; its velocity's direction).
    pub dir: [f32; 3],
}

/// A vehicle path node: where it is, the node its `target` names, its speed
/// (units a second), and a rotate node's own angles.
#[derive(Clone, Debug)]
pub(crate) struct VNode {
    pub origin: [f32; 3],
    pub next: Option<u32>,
    pub speed: f32,
    pub turn: Option<[f32; 3]>,
}

/// A vehicle's speed state (units a second).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VSpeed {
    pub now: f32,
    /// The speed the script asked for (`setspeed`); `None` follows the nodes.
    pub target: Option<f32>,
    pub accel: f32,
    pub decel: f32,
    /// `setvehmaxspeed`.
    pub max: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Vehicles {
    /// The node a vehicle's path starts at (`attachpath`).
    pub attached: BTreeMap<u32, u32>,
    pub moving: BTreeMap<u32, VPath>,
    pub speed: BTreeMap<u32, VSpeed>,
    /// Every `info_vehicle_node` by entity number (built on first use).
    pub nodes: Option<BTreeMap<u32, VNode>>,
    /// Helicopters flying to goals or falling (`flight.rs`).
    pub flights: super::flight::Flights,
    /// Vehicles kept on the ground by their wheels, by entity number.
    pub plants: BTreeMap<u32, Plant>,
}

/// The vehicle types that stand on their wheels (BO2 plants its ground
/// vehicles; Tranzit's bus is the one driven on a path here).
const PLANTED: [&str; 1] = ["zombie_bus"];

/// Its wheel tags, in BO2's order: front left, front right, back left, back
/// right.
const WHEEL_TAGS: [&str; 4] = [
    "tag_wheel_front_left",
    "tag_wheel_front_right",
    "tag_wheel_back_left",
    "tag_wheel_back_right",
];

/// What a wheel stands on: solid, glass and vehicle clip.
const WHEEL_MASK: u32 = 0x211;

/// How fast a wheel off the ground falls faster (units a second, each
/// second: BO2's 40 each 20th of a second).
const WHEEL_GRAVITY: f32 = 800.0;

/// How far ahead on its path a vehicle faces (seconds at its speed).
const LOOKAHEAD_S: f32 = 1.3;

/// A vehicle kept on the ground by its wheels.
#[derive(Clone, Debug, Default)]
pub(crate) struct Plant {
    /// Each wheel's place on the vehicle (forward, left, up), from its tags.
    pub wheels: Option<[[f32; 3]; 4]>,
    /// Times its wheel tags were looked for and not found.
    pub misses: u32,
    /// Each wheel's height and how fast it falls.
    pub z: [f32; 4],
    pub fall: [f32; 4],
    /// The heights are set (from where it stood when it set off).
    pub ready: bool,
}

fn build_nodes(vm: &mut Vm<World>, world: &World) -> BTreeMap<u32, VNode> {
    let zm = world.resource::<Zm>();
    let (tn, tg, sp) = (
        vm.intern("targetname"),
        vm.intern("target"),
        vm.intern("speed"),
    );
    let raw: Vec<(u32, [f32; 3], String, String, f32, Option<[f32; 3]>)> = zm
        .ents
        .iter()
        .filter(|(_, e)| e.classname.starts_with("info_vehicle_node"))
        .filter_map(|(n, e)| {
            let o = e.obj?;
            let name = vm.to_text(&vm.raw_field(o, tn));
            let target = vm.to_text(&vm.raw_field(o, tg));
            let v = vm.raw_field(o, sp);
            let speed = v
                .as_float()
                .or_else(|| vm.to_text(&v).parse().ok())
                .unwrap_or(30.0);
            let turn = (e.classname == "info_vehicle_node_rotate").then_some(e.angles);
            Some((*n, e.origin, name, target, speed, turn))
        })
        .collect();
    let by_name: BTreeMap<&str, u32> = raw
        .iter()
        .filter(|n| !n.2.is_empty())
        .map(|n| (n.2.as_str(), n.0))
        .collect();
    raw.iter()
        .map(|n| {
            let next = (!n.3.is_empty())
                .then(|| by_name.get(n.3.as_str()).copied())
                .flatten();
            (
                n.0,
                VNode {
                    origin: n.1,
                    next,
                    speed: n.4.max(1.0) * MPH,
                    turn: n.5,
                },
            )
        })
        .collect()
}

/// Each tick: vehicles along their paths.
pub(crate) fn advance(world: &mut World, dt: f32) {
    chase_cam(world);
    super::flight::advance(world, dt);
    let ids: Vec<u32> = world
        .resource::<Zm>()
        .vehicles
        .moving
        .keys()
        .copied()
        .collect();
    if ids.is_empty() {
        return;
    }
    let Some(nodes) = world.resource_mut::<Zm>().vehicles.nodes.take() else {
        return;
    };
    // (vehicle, node reached, end of the path)
    let mut reached: Vec<(u32, u32, bool)> = Vec::new();
    for v in ids {
        let Some(mut path) = world.resource::<Zm>().vehicles.moving.get(&v).cloned() else {
            continue;
        };
        let Some(mut pos) = world.resource::<Zm>().ents.get(&v).map(|e| e.origin) else {
            world.resource_mut::<Zm>().vehicles.moving.remove(&v);
            continue;
        };
        let mut sp = world
            .resource::<Zm>()
            .vehicles
            .speed
            .get(&v)
            .copied()
            .unwrap_or_default();
        sp.now = match sp.target {
            Some(t) if sp.now < t => (sp.now + sp.accel * dt).min(t),
            Some(t) => (sp.now - sp.decel * dt).max(t),
            None => nodes.get(&path.from).map_or(0.0, |n| n.speed),
        };
        if let Some(m) = sp.max {
            sp.now = sp.now.min(m);
        }
        // Straight on from node to node at its speed (BO2's curve through
        // them, with no node's tension set: straight lines; it keeps its
        // speed even, where BO2's eases along each stretch).
        let mut step = sp.now * dt;
        let mut done = false;
        let mut hops = 0;
        while hops < 64 {
            let (Some(a), Some(b)) = (nodes.get(&path.from), nodes.get(&path.to)) else {
                done = true;
                break;
            };
            let len = len3(gsc_t6::math::sub(b.origin, a.origin));
            let left = (1.0 - path.t) * len;
            if step < left {
                path.t += step / len;
                pos = lerp(a.origin, b.origin, path.t);
                break;
            }
            step -= left;
            pos = b.origin;
            hops += 1;
            let end = b.next.is_none();
            reached.push((v, path.to, end));
            if end {
                done = true;
                break;
            }
            path.prev = Some(path.from);
            path.from = path.to;
            path.to = b.next.unwrap_or(path.to);
            path.t = 0.0;
            if step <= MIN_STEP {
                break;
            }
        }
        let turns = (
            nodes.get(&path.from).and_then(|n| n.turn),
            nodes.get(&path.to).and_then(|n| n.turn),
            path.t.clamp(0.0, 1.0),
        );
        let Some((was_at, was)) = pose(world, v) else {
            continue;
        };
        // Facing where it will be 1.3 seconds on (as BO2's path vehicles):
        // it starts round a corner before it reaches the node.
        let look = sp.now * LOOKAHEAD_S;
        let toward = (!done && look > MIN_STEP)
            .then(|| ahead(&nodes, &path, look))
            .flatten()
            .map(|p| gsc_t6::math::sub(p, pos));
        let along = toward
            .filter(|d| d[0].abs() + d[1].abs() > 1e-3)
            .map_or(was, |d| {
                // Tilted with its road (up to 14 degrees on Tranzit's); a
                // steeper path is a drop (Nuketown's perk machines fall
                // from the sky on one) and keeps it level.
                let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
                let pitch = -d[2].atan2(flat).to_degrees();
                [
                    if pitch.abs() <= 35.0 { pitch } else { 0.0 },
                    d[1].atan2(d[0]).to_degrees(),
                    0.0,
                ]
            });
        let want = node_turn(turns.0, turns.1, turns.2, along);
        // Turned toward it eased, as BO2's VEH_UpdatePathFollow: its heading
        // closes 4 times the gap a second (a fifth of it each 20th of a
        // second), its pitch and roll 6.
        let mut angles = [
            diff_track(want[0], was[0], 6.0, dt),
            diff_track(want[1], was[1], 4.0, dt),
            diff_track(want[2], was[2], 6.0, dt),
        ];
        if world.resource::<Zm>().vehicles.plants.contains_key(&v) {
            find_wheels(world, v);
            let lean = [was[0], angles[1], was[2]];
            if let Some((z, pitch, roll)) = ground_plant(world, v, pos, was_at, lean, dt) {
                pos[2] = z;
                angles[0] = pitch;
                angles[2] = roll;
            }
        }
        let mut zm = world.resource_mut::<Zm>();
        if let Some(e) = zm.ents.get_mut(&v) {
            let moved = gsc_t6::math::sub(pos, e.origin);
            let m = len3(moved);
            if m > 1e-3 {
                path.dir = moved.map(|x| x / m);
            }
            e.origin = pos;
            e.angles = angles;
        }
        if done {
            sp.now = 0.0;
            zm.vehicles.moving.remove(&v);
        } else {
            zm.vehicles.moving.insert(v, path);
        }
        zm.vehicles.speed.insert(v, sp);
    }
    world.resource_mut::<Zm>().vehicles.nodes = Some(nodes);
    if reached.is_empty() {
        return;
    }
    with_vm(world, |vm, world| {
        for (v, node, end) in reached {
            let zm = world.resource::<Zm>();
            let vo = zm.ents.get(&v).and_then(|e| e.obj);
            let no = zm.ents.get(&node).and_then(|e| e.obj);
            let Some(vo) = vo else { continue };
            if vehicle_log() {
                let at = zm.ents.get(&v).map_or([0.0; 3], |e| e.origin);
                let mph = zm.vehicles.speed.get(&v).map_or(0.0, |sp| sp.now / MPH);
                diag::info!(
                    Sim,
                    "bo2zm t6 vehicle {v} reached node {node} at ({:.0} {:.0} {:.0}) {mph:.1} mph, {} ms{}",
                    at[0],
                    at[1],
                    at[2],
                    zm.now_ms,
                    if end { ", end of path" } else { "" }
                );
            }
            let arg = no.map_or(Value::Undefined, Value::Object);
            vm.notify_str(world, vo, "reached_node", &[arg]);
            if end {
                vm.notify_str(world, vo, "reached_end_node", &[]);
            }
        }
    });
}

/// The least of a tick's travel still driven (units).
const MIN_STEP: f32 = 1e-3;

fn len3(d: [f32; 3]) -> f32 {
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// The angles between nodes `a` and `b` a fraction `t` of the way, as BO2's
/// VP_GetAngles: where either is a rotate node, from the first's angles to
/// the second's (a plain node gives `along`, the way it drives), the short
/// way round each; between two plain nodes just `along`.
fn node_turn(a: Option<[f32; 3]>, b: Option<[f32; 3]>, t: f32, along: [f32; 3]) -> [f32; 3] {
    if a.is_none() && b.is_none() {
        return along;
    }
    let (from, to) = (a.unwrap_or(along), b.unwrap_or(along));
    std::array::from_fn(|i| norm180(from[i] + norm180(to[i] - from[i]) * t))
}

/// An angle in -180..180 (BO2's AngleNormalize180).
fn norm180(a: f32) -> f32 {
    a - 360.0 * (a / 360.0 + 0.5).floor()
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

/// Where a vehicle at `path` will be `dist` units further on (straight from
/// node to node; the last node at the end of its path).
fn ahead(nodes: &BTreeMap<u32, VNode>, path: &VPath, dist: f32) -> Option<[f32; 3]> {
    let (mut from, mut to, mut t, mut dist) = (path.from, path.to, path.t, dist);
    for _ in 0..64 {
        let (a, b) = (nodes.get(&from)?, nodes.get(&to)?);
        let len = len3(gsc_t6::math::sub(b.origin, a.origin));
        let left = (1.0 - t) * len;
        if dist < left {
            return Some(lerp(a.origin, b.origin, t + dist / len));
        }
        let Some(next) = b.next else {
            return Some(b.origin);
        };
        dist -= left;
        (from, to, t) = (to, next, 0.0);
    }
    nodes.get(&to).map(|n| n.origin)
}

/// BO2's DiffTrackAngle: `cur` turned toward `target`, `rate` times the gap
/// a second (straight to it once that would pass it), in -180..180.
fn diff_track(target: f32, cur: f32, rate: f32, dt: f32) -> f32 {
    let gap = norm180(target - cur);
    let step = gap * rate * dt;
    if gap.abs() > 0.001 && step.abs() <= gap.abs() {
        norm180(cur + step)
    } else {
        norm180(target)
    }
}

/// A planted vehicle's wheels' places on it, from its wheel tags where it
/// stands now (once found; a miss is logged by the tag lookup, and given up
/// after a few seconds of tries: it then drives at its nodes' height).
fn find_wheels(world: &mut World, v: u32) {
    let Some(plant) = world.resource::<Zm>().vehicles.plants.get(&v) else {
        return;
    };
    if plant.wheels.is_some() || plant.misses >= 100 {
        return;
    }
    let Some((o, ang)) = pose(world, v) else {
        return;
    };
    let (f, r, u) = gsc_t6::math::angle_vectors(ang);
    let mut wheels = [[0.0f32; 3]; 4];
    let mut found = true;
    for (i, tag) in WHEEL_TAGS.iter().enumerate() {
        let Some((p, _)) = super::tags::tag_world(world, v, tag) else {
            found = false;
            break;
        };
        let d = gsc_t6::math::sub(p, o);
        wheels[i] = [
            gsc_t6::math::dot(d, f),
            -gsc_t6::math::dot(d, r),
            gsc_t6::math::dot(d, u),
        ];
    }
    let mut zm = world.resource_mut::<Zm>();
    let Some(plant) = zm.vehicles.plants.get_mut(&v) else {
        return;
    };
    if found {
        plant.wheels = Some(wheels);
        diag::info!(
            Sim,
            "bo2zm t6 vehicle {v} stands on its wheels at ({:.0} {:.0} {:.0}) ({:.0} {:.0} {:.0}) ({:.0} {:.0} {:.0}) ({:.0} {:.0} {:.0})",
            wheels[0][0], wheels[0][1], wheels[0][2],
            wheels[1][0], wheels[1][1], wheels[1][2],
            wheels[2][0], wheels[2][1], wheels[2][2],
            wheels[3][0], wheels[3][1], wheels[3][2]
        );
    } else {
        plant.misses += 1;
        if plant.misses == 100 {
            diag::info!(Sim, "bo2zm t6 vehicle {v}: no wheel tags, it drives at its nodes' height");
        }
    }
}

/// BO2's VEH_GroundPlant: each wheel, where the vehicle is now (at the height
/// it had), traced from 64 units above to 256 below; a wheel off the ground
/// falls to it, one under it is lifted onto it (one over a gap holds). The
/// vehicle stands on the plane through its wheels (resting on the highest),
/// tilted to it (eased, 6 times the gap a second, at most 60 degrees) at its
/// height there. Its height, pitch and roll; `None` until its wheels are
/// known.
fn ground_plant(
    world: &mut World,
    v: u32,
    pos: [f32; 3],
    was_at: [f32; 3],
    ang: [f32; 3],
    dt: f32,
) -> Option<(f32, f32, f32)> {
    use gsc_t6::math::{cross, dot, normalize, sub};
    let mut plant = world.resource::<Zm>().vehicles.plants.get(&v).cloned()?;
    let wheels = plant.wheels?;
    let (f, r, u) = gsc_t6::math::angle_vectors(ang);
    let base = [pos[0], pos[1], was_at[2]];
    let mut w = [[0.0f32; 3]; 4];
    for (i, o) in wheels.iter().enumerate() {
        let p: [f32; 3] =
            std::array::from_fn(|k| base[k] + f[k] * o[0] - r[k] * o[1] + u[k] * o[2]);
        if !plant.ready {
            plant.z[i] = p[2];
        }
        let top = [p[0], p[1], p[2] + 64.0];
        let bottom = [p[0], p[1], p[2] - 256.0];
        let t = super::frame(world).trace_static_world(top, bottom, [0.0; 3], [0.0; 3], WHEEL_MASK);
        let ground = (t.startsolid == 0 && t.fraction < 1.0)
            .then(|| top[2] + (bottom[2] - top[2]) * t.fraction);
        match ground {
            Some(g) => {
                plant.fall[i] -= WHEEL_GRAVITY * dt;
                plant.z[i] += plant.fall[i] * dt;
                if plant.z[i] < g {
                    plant.z[i] = g;
                    plant.fall[i] = 0.0;
                }
            }
            None => plant.fall[i] = 0.0,
        }
        w[i] = [p[0], p[1], plant.z[i]];
    }
    plant.ready = true;
    let mid = |a: [f32; 3], b: [f32; 3]| -> [f32; 3] { std::array::from_fn(|k| (a[k] + b[k]) * 0.5) };
    let right = normalize(sub(mid(w[1], w[3]), mid(w[0], w[2])));
    let fwd = normalize(sub(mid(w[0], w[1]), mid(w[2], w[3])));
    let n = normalize(cross(right, fwd));
    world.resource_mut::<Zm>().vehicles.plants.insert(v, plant);
    if n[2] < 0.1 {
        return None;
    }
    let d = w.iter().map(|wi| dot(n, *wi)).fold(f32::MIN, f32::max);
    let z = (d - n[0] * pos[0] - n[1] * pos[1]) / n[2];
    let side = normalize(cross(n, f));
    let on = normalize(cross(side, n));
    let [pitch, _, roll] = super::tags::axis_to_angles([on, side, n]);
    let pitch = diff_track(pitch, ang[0], 6.0, dt).clamp(-60.0, 60.0);
    let roll = diff_track(roll, ang[2], 6.0, dt).clamp(-60.0, 60.0);
    Some((z, pitch, roll))
}

fn set_speed(
    vm: &mut Vm<World>,
    world: &mut World,
    s: &Value,
    a: &[Value],
    immediate: bool,
) -> Result<Value, String> {
    let Some(v) = entnum(vm, s) else {
        return Ok(Value::Undefined);
    };
    let mph = num(a, 0).unwrap_or(0.0).max(0.0);
    let accel = num(a, 1).unwrap_or(10.0).max(0.1);
    let decel = num(a, 2).unwrap_or(accel).max(0.1);
    let mut zm = world.resource_mut::<Zm>();
    let sp = zm.vehicles.speed.entry(v).or_default();
    sp.target = Some(mph * MPH);
    sp.accel = accel * MPH;
    sp.decel = decel * MPH;
    if immediate {
        sp.now = mph * MPH;
    }
    Ok(Value::Undefined)
}

/// A vehicle's speed now (units a second); 0 off its path.
fn speed_now(vm: &Vm<World>, world: &World, s: &Value) -> f32 {
    let zm = world.resource::<Zm>();
    entnum(vm, s)
        .filter(|v| zm.vehicles.moving.contains_key(v))
        .and_then(|v| zm.vehicles.speed.get(&v))
        .map_or(0.0, |sp| sp.now)
}

/// IW4L_T6_CHASECAM="back up": from 12 s player 0 is held `back` units
/// behind and `up` above the first script-driven vehicle (Tranzit's bus),
/// looking at it: shots of it on the road.
fn chase_cam(world: &mut World) {
    static CAM: std::sync::OnceLock<Option<(f32, f32)>> = std::sync::OnceLock::new();
    let Some((back, up)) = *CAM.get_or_init(|| {
        let v = std::env::var("IW4L_T6_CHASECAM").ok()?;
        let mut n = v.split_whitespace().filter_map(|x| x.parse().ok());
        Some((n.next()?, n.next()?))
    }) else {
        return;
    };
    let zm = world.resource::<Zm>();
    if zm.now_ms < 12_000 {
        return;
    }
    let Some((o, ang)) = zm.vehicles.speed.keys().find_map(|v| pose(world, *v)) else {
        return;
    };
    let (f, _, _) = gsc_t6::math::angle_vectors([0.0, ang[1], 0.0]);
    let eye = [o[0] - f[0] * back, o[1] - f[1] * back, o[2] + up];
    let d = gsc_t6::math::sub([o[0], o[1], o[2] + 60.0], eye);
    let pitch = -(d[2].atan2((d[0] * d[0] + d[1] * d[1]).sqrt())).to_degrees();
    let yaw = d[1].atan2(d[0]).to_degrees();
    let client = crate::world::ClientId(0);
    super::teleport_player(world, client, [eye[0], eye[1], eye[2] - 60.0]);
    super::set_player_view(world, client, [pitch, yaw, 0.0]);
}

/// IW4L_T6_VEHLOG=1: a log line at each vehicle path node passed.
fn vehicle_log() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("IW4L_T6_VEHLOG").is_some())
}

/// A script vehicle's velocity (units a second): its speed along the way it
/// last moved (not always the way it faces: a rotate node turns it), zero
/// while parked; `None` for an entity that is not one (the cattle catcher's
/// shove adds the bus's `getvelocity` to its own).
pub(crate) fn velocity(world: &World, n: u32) -> Option<[f32; 3]> {
    let zm = world.resource::<Zm>();
    let e = zm.ents.get(&n).filter(|e| e.classname == "script_vehicle")?;
    let Some(path) = zm.vehicles.moving.get(&n) else {
        return Some([0.0; 3]);
    };
    let speed = zm.vehicles.speed.get(&n).map_or(0.0, |sp| sp.now);
    let dir = if len3(path.dir) > 0.5 {
        path.dir
    } else {
        gsc_t6::math::angle_vectors(e.angles).0
    };
    Some(dir.map(|c| c * speed))
}

/// An entity's origin and angles.
fn pose(world: &World, n: u32) -> Option<([f32; 3], [f32; 3])> {
    world
        .resource::<Zm>()
        .ents
        .get(&n)
        .map(|e| (e.origin, e.angles))
}

pub(super) fn bind(vm: &mut Vm<World>) {
    // getvehiclearray(team?): the live script vehicles, picked by their team
    // field like getaiarray (Mob's Acid Gat adds the "axis" ones to the
    // zombies it may aim at).
    vm.bind("getvehiclearray", false, |vm, world, _, a| {
        let team = match arg(a, 0) {
            Value::Undefined => None,
            v => Some(vm.to_text(v)),
        };
        let team_s = vm.intern("team");
        let objs: Vec<gsc_t6::ObjRef> = world
            .resource::<Zm>()
            .ents
            .values()
            .filter(|e| e.classname == "script_vehicle")
            .filter_map(|e| e.obj)
            .collect();
        let out = objs
            .into_iter()
            .filter(|o| vm.alive(*o))
            .filter(|o| match &team {
                None => true,
                Some(t) if t == "all" => true,
                Some(t) => vm.to_text(&vm.raw_field(*o, team_s)) == *t,
            })
            .map(Value::Object)
            .collect();
        Ok(super::list(out))
    });
    vm.bind("attachpath", true, |vm, world, s, a| {
        if let (Some(v), Some(node)) = (entnum(vm, s), entnum(vm, arg(a, 0))) {
            world.resource_mut::<Zm>().vehicles.attached.insert(v, node);
        }
        Ok(Value::Undefined)
    });
    vm.bind("startpath", true, |vm, world, s, _| {
        let Some(v) = entnum(vm, s) else {
            return Ok(Value::Undefined);
        };
        let Some(start) = world.resource::<Zm>().vehicles.attached.get(&v).copied() else {
            return Ok(Value::Undefined);
        };
        if world.resource::<Zm>().vehicles.nodes.is_none() {
            let nodes = build_nodes(vm, world);
            world.resource_mut::<Zm>().vehicles.nodes = Some(nodes);
        }
        let planted = match s {
            Value::Object(o) => {
                let key = vm.intern("vehicletype");
                PLANTED.contains(&vm.to_text(&vm.raw_field(*o, key)).as_str())
            }
            _ => false,
        };
        let mut zm = world.resource_mut::<Zm>();
        if planted {
            zm.vehicles.plants.entry(v).or_default();
        }
        let Some(first) = zm
            .vehicles
            .nodes
            .as_ref()
            .and_then(|n| n.get(&start))
            .cloned()
        else {
            return Ok(Value::Undefined);
        };
        if let Some(e) = zm.ents.get_mut(&v) {
            // A planted one keeps its height: its wheels set it on the road.
            e.origin = if planted {
                [first.origin[0], first.origin[1], e.origin[2]]
            } else {
                first.origin
            };
        }
        let path = VPath {
            from: start,
            to: first.next.unwrap_or(start),
            ..Default::default()
        };
        zm.vehicles.moving.insert(v, path);
        Ok(Value::Undefined)
    });
    // setspeed(mph, accel, decel) / setspeedimmediate(mph): the script
    // drives the vehicle from here on.
    vm.bind("setspeed", true, |vm, world, s, a| {
        set_speed(vm, world, s, a, false)
    });
    vm.bind("setspeedimmediate", true, |vm, world, s, a| {
        set_speed(vm, world, s, a, true)
    });
    vm.bind("resumespeed", true, |vm, world, s, _| {
        if let Some(v) = entnum(vm, s) {
            let mut zm = world.resource_mut::<Zm>();
            zm.vehicles.speed.entry(v).or_default().target = None;
        }
        Ok(Value::Undefined)
    });
    vm.bind("setvehmaxspeed", true, |vm, world, s, a| {
        if let Some(v) = entnum(vm, s) {
            let mph = num(a, 0).unwrap_or(0.0);
            let mut zm = world.resource_mut::<Zm>();
            zm.vehicles.speed.entry(v).or_default().max = (mph > 0.0).then_some(mph * MPH);
        }
        Ok(Value::Undefined)
    });
    // getspeed: units a second; getspeedmph: miles per hour.
    vm.bind("getspeed", true, |vm, world, s, _| {
        Ok(Value::Float(speed_now(vm, world, s)))
    });
    vm.bind("getspeedmph", true, |vm, world, s, _| {
        Ok(Value::Float(speed_now(vm, world, s) / MPH))
    });
    // Turret aim (Origins' tank flamethrowers): the turret's pose isn't
    // drawn apart from the hull yet, so the aim is accepted and dropped.
    vm.bind("setturrettargetvec", true, |_, _, _, _| Ok(Value::Undefined));
    vm.bind("setturrettargetent", true, |_, _, _, _| Ok(Value::Undefined));
    // Entity space <-> world space (forward, left, up), as `linkto` offsets.
    vm.bind("localtoworldcoords", true, |vm, world, s, a| {
        let local = vec3(a, 0)?;
        let Some((o, ang)) = entnum(vm, s).and_then(|n| pose(world, n)) else {
            return Ok(Value::Vec3(local));
        };
        let (f, r, u) = gsc_t6::math::angle_vectors(ang);
        Ok(Value::Vec3(std::array::from_fn(|i| {
            o[i] + f[i] * local[0] - r[i] * local[1] + u[i] * local[2]
        })))
    });
    vm.bind("worldtolocalcoords", true, |vm, world, s, a| {
        let p = vec3(a, 0)?;
        let Some((o, ang)) = entnum(vm, s).and_then(|n| pose(world, n)) else {
            return Ok(Value::Vec3(p));
        };
        let (f, r, u) = gsc_t6::math::angle_vectors(ang);
        let d = gsc_t6::math::sub(p, o);
        Ok(Value::Vec3([
            gsc_t6::math::dot(d, f),
            -gsc_t6::math::dot(d, r),
            gsc_t6::math::dot(d, u),
        ]))
    });
    // spawnvehicle(model, targetname, vehicletype, origin, angles): a
    // script vehicle (Origins' crystal biplane, Maxis drone); it drives a
    // path once given one (attachpath/startpath).
    vm.bind("spawnvehicle", false, |vm, world, _, a| {
        let text = |vm: &Vm<World>, i: usize| match arg(a, i) {
            Value::Undefined => String::new(),
            v => vm.to_text(v),
        };
        let (model, targetname, vehicletype) = (text(vm, 0), text(vm, 1), text(vm, 2));
        let origin = vec3(a, 3).unwrap_or([0.0; 3]);
        let angles = vec3(a, 4).unwrap_or([0.0; 3]);
        let n = world.resource_mut::<Zm>().alloc_entnum();
        let obj = vm.alloc_object(gsc_t6::ObjKind::Entity(n));
        world.resource_mut::<Zm>().ents.insert(
            n,
            super::Ent {
                obj: Some(obj),
                classname: "script_vehicle".to_owned(),
                origin,
                angles,
                model,
                solid: true,
                ..Default::default()
            },
        );
        for (k, v) in [
            ("targetname", targetname),
            ("vehicletype", vehicletype),
            ("classname", "script_vehicle".to_owned()),
        ] {
            let k = vm.intern(k);
            let v = vm.string(&v);
            vm.set_field(world, obj, k, v);
        }
        Ok(Value::Object(obj))
    });
    vm.bind("vehicle_detachfrompath", true, |_, _, _, _| Ok(Value::Undefined));
    super::flight::bind(vm);
}
