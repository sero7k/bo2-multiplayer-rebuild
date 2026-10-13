//! Carried equipment as BO2's engine gives it to the zombies scripts
//! (`_zm_equipment::equipment_watch_placement`): when a player raises a
//! placeable equipment weapon (Tranzit's turbine, turret, electric trap;
//! Die Rise's Trample Steam; Buried's head chopper, subwoofer) the script
//! makes a turret (`spawnturret`) with the equipment's model, has him carry
//! it in front of him (`carryturret`), and when he fires asks the engine
//! where it would go down (`canplayerplaceturret`): `["result"]`,
//! `["origin"]`, `["angles"]`. The placed copy is the script's own model.
//!
//! A placed turret (TranZit's buildable turret) aims itself in mode
//! `auto_nonai` as BO2's engine does (`turret_target_sentry`): it keeps its
//! target for a second whatever happens, then while it can see it, and each
//! frame looks at one more enemy in turn, taking it if it can see it and it
//! is nearer. It aims at the eye, or the feet when the eye is outside its
//! pitch arcs. For its convergence time it fires wide (63 units beside the
//! target at its feet at first, closing to 33), then on target. Its state
//! (0 idle, 2 firing wide, 1 firing) gives `isfiringturret`, and
//! `turretstatechange` when it starts or stops; the scripts'
//! `_zm_mgturret::burst_fire_unmanned` waits on that and fires in bursts
//! with `shootturret` (one bullet each). The numbers are
//! `zombie_bullet_crouch_zm`'s (T6LIST_WEAPON): 300 damage to 1800 units,
//! 250 from 3000, no range limit, arcs 45 left / right and 20 up / down, ai
//! spread 2 degrees, turns 1 to 360 degrees a second, convergence 0.3 s
//! pitch and (the script's `setconvergencetime( 0.3 )`) yaw. The aim and
//! muzzle come from the turret model's own bones (`tag_aim`, `tag_flash`).

use std::collections::HashMap;

use bevy_ecs::prelude::{Resource, World};
use gsc_t6::{Key, ObjKind, ObjRef, Value, Vm};

use super::{Ent, Zm, actors, arg, entnum, frame, num, text, vec3, with_vm};
use crate::world::ClientId;
use gsc_t6::math::angle_clamp180;

/// `zombie_bullet_crouch_zm`'s arcs (left, right, up, down), its ai spread,
/// its slowest and fastest turn (degrees a second) and its convergence
/// times (pitch, yaw; ms), read from the zone.
const ARCS: [f32; 4] = [45.0, 45.0, 20.0, 20.0];
const AI_SPREAD_DEG: f32 = 2.0;
const TURN_MIN: f32 = 1.0;
const TURN_MAX: f32 = 360.0;
const CONVERGE_MS: [i64; 2] = [300, 1500];
/// Its max range is 0: no limit.
const SHOT_RANGE: f32 = 8192.0;
/// `turret_SentryTargetTime`: it keeps a target this long without looking.
const KEEP_MS: i64 = 1000;
/// For its first 250 ms on a target it opens fire only within 5 degrees.
const SETTLE_MS: i64 = 250;
const SETTLE_DEG: f32 = 5.0;
/// Firing wide: first 63 beside the target at its feet, then
/// 33 + 30 * (convergence left) from where it would aim.
const MISS_SIDE: f32 = 63.0;
const MISS_NEAR: f32 = 33.0;
const MISS_FAR: f32 = 30.0;
/// A standing zombie's eye (the engine's `geteye` for actors) and height.
const EYE: f32 = 60.0;
const STAND: f32 = 72.0;
/// `p6_anim_zm_buildable_turret`'s bones from its base: the gun swings
/// about `tag_aim`, and `tag_flash` is 28.24 in front of it.
const TAG_AIM: [f32; 3] = [-13.47, 0.0, 46.90];
const AIM_TO_FLASH: f32 = 28.24;

/// What the engine keeps for a turret the scripts made.
#[derive(Clone, Debug)]
struct Turret {
    obj: ObjRef,
    weapon: String,
    mode: String,
    /// `settargetentity`: it aims there whatever its mode.
    manual: Option<u32>,
    target: Option<u32>,
    /// When it took its target (the engine's `targetTime`).
    target_at: i64,
    /// The enemy it looked at last (one a frame, in turn).
    prev: Option<u32>,
    /// Its gun's pitch and yaw from the way it was set down.
    ang: [f32; 2],
    /// The same as a direction in the world.
    aim: [f32; 3],
    /// `setconvergencetime` (pitch, yaw) in ms.
    converge_ms: [i64; 2],
    /// Which way it fires wide of this target, once chosen.
    miss: Option<[f32; 3]>,
    /// 0 idle, 2 firing wide, 1 firing.
    state: u8,
}

#[derive(Resource, Default)]
pub(crate) struct Turrets(HashMap<u32, Turret>);

fn with_turret<R>(world: &mut World, n: u32, f: impl FnOnce(&mut Turret) -> R) -> Option<R> {
    world
        .get_resource_mut::<Turrets>()
        .and_then(|mut t| t.0.get_mut(&n).map(f))
}

fn log_on() -> bool {
    std::env::var_os("IW4L_T6_TURRETLOG").is_some()
}

/// How far in front of his feet a carried turret is set down, and how far
/// above / below that spot the ground may be (steps, a kerb).
const PLACE_AHEAD: f32 = 40.0;
const PLACE_UP: f32 = 32.0;
const PLACE_DOWN: f32 = 64.0;
/// The ground must be flatter than about 45 degrees.
const PLACE_MIN_NORMAL_Z: f32 = 0.7;

pub(super) fn bind(vm: &mut Vm<World>) {
    // spawnturret(classname, origin, weaponinfo)
    vm.bind("spawnturret", false, |vm, world, _, a| {
        let origin = vec3(a, 1).unwrap_or([0.0; 3]);
        let n = world.resource_mut::<Zm>().alloc_entnum();
        let obj = vm.alloc_object(ObjKind::Entity(n));
        world.resource_mut::<Zm>().ents.insert(
            n,
            Ent {
                obj: Some(obj),
                classname: "misc_turret".to_owned(),
                origin,
                ..Default::default()
            },
        );
        let k = vm.intern("weaponinfo");
        let w = text(vm, a, 2);
        let wv = vm.string(&w);
        vm.set_field(world, obj, k, wv);
        let aim = [1.0, 0.0, 0.0];
        world.get_resource_or_insert_with(Turrets::default).0.insert(
            n,
            Turret {
                obj,
                weapon: w,
                mode: "manual".to_owned(),
                manual: None,
                target: None,
                target_at: 0,
                prev: None,
                ang: [0.0; 2],
                aim,
                converge_ms: CONVERGE_MS,
                miss: None,
                state: 0,
            },
        );
        Ok(Value::Object(obj))
    });
    for name in [
        "setturretowner",
        "setturrettype",
        "makeunusable",
        "maketurretunusable",
        "setdefaultdroppitch",
        "setturretteam",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    // turret setconvergencetime(seconds, "pitch" | "yaw"): yaw unless told.
    vm.bind("setconvergencetime", true, |vm, world, s, a| {
        if let Some(n) = entnum(vm, s) {
            let ms = (num(a, 0).unwrap_or(0.0) * 1000.0).round().max(0.0) as i64;
            let axis = usize::from(text(vm, a, 1) != "pitch");
            with_turret(world, n, |t| t.converge_ms[axis] = ms);
        }
        Ok(Value::Undefined)
    });
    // turret setmode("auto_nonai" | "manual" | ...)
    vm.bind("setmode", true, |vm, world, s, a| {
        if let Some(n) = entnum(vm, s) {
            let mode = text(vm, a, 0);
            with_turret(world, n, |t| t.mode = mode);
        }
        Ok(Value::Undefined)
    });
    vm.bind("settargetentity", true, |vm, world, s, a| {
        if let (Some(n), Some(e)) = (entnum(vm, s), entnum(vm, arg(a, 0))) {
            with_turret(world, n, |t| t.manual = Some(e));
        }
        Ok(Value::Undefined)
    });
    vm.bind("cleartargetentity", true, |vm, world, s, _| {
        if let Some(n) = entnum(vm, s) {
            with_turret(world, n, |t| t.manual = None);
        }
        Ok(Value::Undefined)
    });
    vm.bind("isfiringturret", true, |vm, world, s, _| {
        let firing = entnum(vm, s)
            .and_then(|n| with_turret(world, n, |t| t.state != 0))
            .unwrap_or(false);
        Ok(Value::bool(firing))
    });
    // One bullet down its aim (the scripts' bursts call it every 0.112 s).
    vm.bind("shootturret", true, |vm, world, s, _| {
        if let Some(n) = entnum(vm, s) {
            shoot(vm, world, n);
        }
        Ok(Value::Undefined)
    });
    // player carryturret(turret, offset, angles): it goes where he goes.
    vm.bind("carryturret", true, |vm, world, s, a| {
        let (Some(me), Some(t)) = (entnum(vm, s), entnum(vm, arg(a, 0))) else {
            return Err("carryturret needs a player and a turret".into());
        };
        let off = vec3(a, 1).unwrap_or([PLACE_AHEAD, 0.0, 0.0]);
        let aoff = vec3(a, 2).unwrap_or([0.0; 3]);
        if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&t) {
            e.link = Some((me, off, aoff));
        }
        Ok(Value::Undefined)
    });
    vm.bind("stopcarryturret", true, |vm, world, _, a| {
        if let Some(t) = entnum(vm, arg(a, 0))
            && let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&t)
        {
            e.link = None;
        }
        Ok(Value::Undefined)
    });
    // player canplayerplaceturret(turret): the ground under the carried
    // turret (it rides 22 units in front of him, _zm_equipment's offset).
    vm.bind("canplayerplaceturret", true, |vm, world, s, a| {
        let Some(me) = entnum(vm, s) else {
            return Err("canplayerplaceturret needs a player".into());
        };
        let carried = entnum(vm, arg(a, 0))
            .and_then(|t| world.resource::<Zm>().ents.get(&t).map(|e| e.origin));
        let (ok, origin, angles) = place_spot(world, ClientId(me), carried);
        let mut arr = gsc_t6::Array::new();
        let k = |vm: &mut Vm<World>, s: &str| Key::Str(vm.intern(s));
        arr.set(k(vm, "result"), Value::bool(ok));
        arr.set(k(vm, "origin"), Value::Vec3(origin));
        arr.set(k(vm, "angles"), Value::Vec3(angles));
        if std::env::var_os("IW4L_T6_PLACE").is_some() {
            diag::info!(
                Sim,
                "bo2zm t6 canplayerplaceturret: {ok} at ({:.0} {:.0} {:.0}) yaw {:.0}",
                origin[0],
                origin[1],
                origin[2],
                angles[1]
            );
        }
        Ok(Value::array(arr))
    });
}

/// The ground under the carried turret, or in front of him: (fits, origin,
/// angles facing his way).
pub(super) fn place_spot(
    world: &mut World,
    client: ClientId,
    carried: Option<[f32; 3]>,
) -> (bool, [f32; 3], [f32; 3]) {
    let Some((feet, yaw)) = frame(world)
        .player(client)
        .map(|ps| (ps.origin, ps.viewangles[1]))
    else {
        return (false, [0.0; 3], [0.0; 3]);
    };
    let angles = [0.0, yaw, 0.0];
    let (f, _, _) = gsc_t6::math::angle_vectors(angles);
    let at = match carried {
        Some(c) => [c[0], c[1], feet[2]],
        None => [feet[0] + f[0] * PLACE_AHEAD, feet[1] + f[1] * PLACE_AHEAD, feet[2]],
    };
    let from = [at[0], at[1], at[2] + PLACE_UP];
    let to = [at[0], at[1], at[2] - PLACE_DOWN];
    let mask = crate::bullet_collision::MASK_PLAYER_SOLID;
    let t = frame(world).trace_static_world(from, to, [0.0; 3], [0.0; 3], mask);
    // Nothing solid between his feet and the spot (no setting it down
    // through a wall).
    let eye = [feet[0], feet[1], feet[2] + PLACE_UP];
    let path = frame(world).trace_static_world(eye, from, [0.0; 3], [0.0; 3], mask);
    let clear = path.fraction >= 1.0;
    let hit = t.startsolid == 0 && t.fraction < 1.0 && t.normal[2] >= PLACE_MIN_NORMAL_Z;
    if std::env::var_os("IW4L_T6_PLACE").is_some() {
        diag::info!(
            Sim,
            "bo2zm t6 place spot: ground trace fraction {:.2} startsolid {} normal z {:.2}, path clear {clear}",
            t.fraction,
            t.startsolid,
            t.normal[2]
        );
        if !clear {
            diag::info!(
                Sim,
                "bo2zm t6 place spot: path ({:.1} {:.1} {:.1}) to ({:.1} {:.1} {:.1}) stopped at {:.2} ({:.1} {:.1} {:.1}) startsolid {} allsolid {} normal ({:.2} {:.2} {:.2}) contents {:#x} surface {:#x} material {} hit type {} id {} model {}",
                eye[0],
                eye[1],
                eye[2],
                from[0],
                from[1],
                from[2],
                path.fraction,
                path.endpos[0],
                path.endpos[1],
                path.endpos[2],
                path.startsolid,
                path.allsolid,
                path.normal[0],
                path.normal[1],
                path.normal[2],
                path.contents,
                path.surface_flags,
                path.material,
                path.hit_type,
                path.hit_id,
                path.model_index
            );
        }
    }
    let z = from[2] + (to[2] - from[2]) * t.fraction.clamp(0.0, 1.0);
    (hit && clear, [at[0], at[1], if hit { z } else { feet[2] }], angles)
}

/// Where its gun pivots (`tag_aim`) for a turret standing at `origin`
/// turned `yaw`.
fn pivot(origin: [f32; 3], yaw: f32) -> [f32; 3] {
    let (f, r, _) = gsc_t6::math::angle_vectors([0.0, yaw, 0.0]);
    [
        origin[0] + f[0] * TAG_AIM[0] - r[0] * TAG_AIM[1],
        origin[1] + f[1] * TAG_AIM[0] - r[1] * TAG_AIM[1],
        origin[2] + TAG_AIM[2],
    ]
}

/// A body's feet and eye (a standing zombie's eye is the engine's `geteye`
/// for actors; a shorter body's is lower in step).
fn eye_of(world: &World, n: u32) -> Option<([f32; 3], [f32; 3])> {
    let zm = world.resource::<Zm>();
    let o = zm.ents.get(&n)?.origin;
    let h = zm.actors.get(&n).map_or(STAND, |a| a.height);
    Some((o, [o[0], o[1], o[2] + EYE * (h / STAND).min(1.0)]))
}

/// The pitch and yaw to `at` from `from` as the gun turns (its yaw from the
/// way it was set down; pitch down is positive).
fn local_angles(from: [f32; 3], base_yaw: f32, at: [f32; 3]) -> [f32; 2] {
    let a = gsc_t6::math::vector_to_angles(gsc_t6::math::sub(at, from));
    [angle_clamp180(a[0]), angle_clamp180(a[1] - base_yaw)]
}

fn pitch_in(p: f32) -> bool {
    (-ARCS[2]..=ARCS[3]).contains(&p)
}

fn yaw_in(y: f32) -> bool {
    (-ARCS[1]..=ARCS[0]).contains(&y)
}

/// `turret_CanTargetPoint`: inside its arcs.
fn can_point(from: [f32; 3], base_yaw: f32, at: [f32; 3]) -> bool {
    let a = local_angles(from, base_yaw, at);
    pitch_in(a[0]) && yaw_in(a[1])
}

/// `turret_CanTargetSentient`: where it aims at enemy `n`. Its eye, or its
/// feet when the eye is outside the pitch arcs (feet first while it fires
/// wide), or level at the gun when the eye is above the arcs and the feet
/// below them; nothing outside the yaw arcs.
fn aim_point(world: &World, from: [f32; 3], base_yaw: f32, n: u32, wide: bool) -> Option<[f32; 3]> {
    let (origin, eye) = eye_of(world, n)?;
    let feet = [origin[0], origin[1], origin[2] + 2.0];
    let e = local_angles(from, base_yaw, eye);
    if !yaw_in(e[1]) {
        return None;
    }
    let f = local_angles(from, base_yaw, feet)[0];
    if wide {
        return if pitch_in(f) {
            Some(feet)
        } else {
            pitch_in(e[0]).then_some(eye)
        };
    }
    if pitch_in(e[0]) {
        Some(eye)
    } else if pitch_in(f) {
        Some(feet)
    } else {
        (e[0] < -ARCS[2] && f > ARCS[3]).then_some([eye[0], eye[1], from[2]])
    }
}

/// Nothing of the map between it and the spot.
fn clear(world: &mut World, from: [f32; 3], to: [f32; 3]) -> bool {
    let mask = crate::bullet_collision::MASK_SHOT;
    frame(world)
        .trace_static_world(from, to, [0.0; 3], [0.0; 3], mask)
        .fraction
        >= 0.99
}

/// `turret_isTargetVisible`: it can aim at enemy `n` and nothing of the map
/// is in the way (no range limit).
fn sees(world: &mut World, from: [f32; 3], base_yaw: f32, n: u32, wide: bool) -> bool {
    aim_point(world, from, base_yaw, n, wide).is_some_and(|at| clear(world, from, at))
}

/// `turret_findBestTarget`: its target outright for its first second, then
/// while it can see it; and each frame one more enemy in turn (by entity
/// number), taken if it can see it and it is nearer.
fn find_best(
    world: &mut World,
    t: &mut Turret,
    origin: [f32; 3],
    from: [f32; 3],
    base_yaw: f32,
    team: &str,
    now: i64,
) -> Option<u32> {
    let foes: Vec<u32> = world
        .resource::<Zm>()
        .actors
        .iter()
        .filter(|(_, a)| a.alive && a.team != team)
        .map(|(n, _)| *n)
        .collect();
    let wide = t.state == 2;
    let mut keep = None;
    if let Some(cur) = t.target.filter(|c| foes.contains(c)) {
        if now <= t.target_at + KEEP_MS {
            return Some(cur);
        }
        if sees(world, from, base_yaw, cur, wide) {
            keep = Some(cur);
        }
    }
    let next = match t.prev {
        None => foes.first().copied(),
        Some(p) => foes.iter().copied().find(|&n| n > p),
    };
    t.prev = next;
    let Some(c) = next.filter(|&c| Some(c) != keep) else {
        return keep;
    };
    if !sees(world, from, base_yaw, c, wide) {
        return keep;
    }
    let d2 = |n: u32| {
        world.resource::<Zm>().ents.get(&n).map_or(f32::MAX, |e| {
            let d = gsc_t6::math::sub(e.origin, origin);
            gsc_t6::math::dot(d, d)
        })
    };
    match keep {
        Some(k) if d2(k) <= d2(c) => Some(k),
        _ => Some(c),
    }
}

/// `turret_aimat_Sentient_Internal`: take `n` as its target and say where
/// to point. For its yaw convergence time it points wide: the first frame
/// 63 beside the target at its feet, on the side the gun already points,
/// then 33 + 30 * (time left) / (convergence) that way from the aim point.
/// None when it cannot aim at `n`.
fn aim_at(world: &World, t: &mut Turret, n: u32, from: [f32; 3], base_yaw: f32, now: i64) -> Option<[f32; 3]> {
    if t.target != Some(n) {
        t.target = Some(n);
        t.target_at = now;
        t.miss = None;
    }
    let at = aim_point(world, from, base_yaw, n, t.state == 2)?;
    let left = t.converge_ms[1] + t.target_at - now;
    if left <= 0 {
        return Some(at);
    }
    match t.miss {
        None => {
            let feet = eye_of(world, n)?.0[2];
            let (dx, dy) = (at[0] - from[0], at[1] - from[1]);
            let len = (dx * dx + dy * dy).sqrt();
            let len = if len > 0.0 { len } else { 1.0 };
            let mut side = [dy / len * MISS_SIDE, -dx / len * MISS_SIDE];
            if t.aim[0] * side[0] + t.aim[1] * side[1] < 0.0 {
                side = [-side[0], -side[1]];
            }
            let mut miss = [at[0] + side[0], at[1] + side[1], feet];
            if !can_point(from, base_yaw, miss) {
                miss[2] = at[2];
            }
            if can_point(from, base_yaw, miss) {
                t.miss = Some(gsc_t6::math::normalize(gsc_t6::math::sub(miss, at)));
                return Some(miss);
            }
            t.miss = Some([0.0; 3]);
            Some(at)
        }
        Some(off) => {
            let frac = left as f32 / t.converge_ms[1].max(1) as f32;
            let p = gsc_t6::math::add(at, gsc_t6::math::scale(off, MISS_NEAR + MISS_FAR * frac));
            Some(if can_point(from, base_yaw, p) { p } else { at })
        }
    }
}

/// `turret_aimat_vector_internal`: idle to firing wide once it is within 5
/// degrees of `want` or 250 ms have gone, then firing once its yaw
/// convergence time is up.
fn step_state(t: &mut Turret, want: [f32; 2], now: i64) {
    if t.state == 0 {
        let near = (0..2).all(|i| angle_clamp180(want[i] - t.ang[i]).abs() < SETTLE_DEG);
        if now < t.target_at + SETTLE_MS && !near {
            return;
        }
        t.state = 2;
    } else if t.state != 2 {
        return;
    }
    if t.converge_ms[1] + t.target_at - now < 0 {
        t.state = 1;
    }
}

/// `turret_UpdateTargetAngles`: pitch and yaw each turn at the speed that
/// lands on `want` as its convergence time for that axis runs out (in one
/// frame once it has), kept between the weapon's slowest and fastest turn.
fn turn(t: &mut Turret, want: [f32; 2], now: i64) {
    let tick = crate::MATCH_TICK_MS as f32;
    for i in 0..2 {
        let delta = angle_clamp180(want[i] - t.ang[i]);
        let mut speed = delta.abs() * 1000.0 / tick;
        let left = (t.converge_ms[i] + t.target_at - now) as f32;
        if left > tick {
            speed *= tick / left;
        }
        let step = speed.clamp(TURN_MIN, TURN_MAX) * tick * 0.001;
        t.ang[i] += delta.clamp(-step, step);
    }
}

/// Every placed turret picks a zombie, swings onto it (wide at first) and
/// says `turret_target_aquired` when it takes one and `turretstatechange`
/// when it starts or stops firing (once a tick, before the scripts run).
pub(super) fn advance(world: &mut World) {
    let Some(list) = world
        .get_resource::<Turrets>()
        .map(|t| t.0.keys().copied().collect::<Vec<u32>>())
    else {
        return;
    };
    let now = world.resource::<Zm>().now_ms;
    for n in list {
        let place = world
            .resource::<Zm>()
            .ents
            .get(&n)
            .map(|e| (e.origin, e.angles, e.link));
        let Some((origin, angles, link)) = place else {
            world.resource_mut::<Turrets>().0.remove(&n);
            continue;
        };
        let Some(mut t) = world.resource::<Turrets>().0.get(&n).cloned() else {
            continue;
        };
        // Carried (linked to a player) it is only being set down.
        let carried = link.is_some_and(|(p, ..)| world.resource::<Zm>().players.contains_key(&p));
        let base_yaw = angles[1];
        let from = pivot(origin, base_yaw);
        let team = with_vm(world, |vm, world| {
            let k = vm.intern("team");
            let v = vm.get_field(world, t.obj, k);
            vm.to_text(&v)
        })
        .unwrap_or_default();
        let auto = matches!(t.mode.as_str(), "auto_nonai" | "auto_ai" | "sentry");
        let (was, before) = (t.state, t.target);
        let pick = if carried {
            None
        } else if let Some(m) = t.manual {
            Some(m)
        } else if auto {
            let team = if team.is_empty() { "allies" } else { team.as_str() };
            find_best(world, &mut t, origin, from, base_yaw, team, now)
        } else {
            None
        };
        let point = pick.and_then(|p| aim_at(world, &mut t, p, from, base_yaw, now));
        let want = match point {
            Some(p) => {
                let want = local_angles(from, base_yaw, p);
                step_state(&mut t, want, now);
                want
            }
            None => {
                t.target = None;
                t.miss = None;
                t.state = 0;
                [0.0; 2]
            }
        };
        turn(&mut t, want, now);
        t.aim = gsc_t6::math::angle_vectors([t.ang[0], base_yaw + t.ang[1], 0.0]).0;
        let (obj, state, target) = (t.obj, t.state, t.target);
        world.resource_mut::<Turrets>().0.insert(n, t);
        let taken = target.is_some() && target != before;
        if log_on() && (taken || state != was) {
            let d = target
                .and_then(|e| world.resource::<Zm>().ents.get(&e).map(|e| e.origin))
                .map_or(0.0, |o| gsc_t6::math::length(gsc_t6::math::sub(o, origin)));
            diag::info!(
                Sim,
                "bo2zm t6 turret ent{n}: target {target:?} at {d:.0}, state {state} (firing {})",
                state != 0
            );
        }
        if taken {
            with_vm(world, |vm, world| {
                vm.notify_str(world, obj, "turret_target_aquired", &[]);
            });
        }
        if (state != 0) != (was != 0) {
            with_vm(world, |vm, world| {
                vm.notify_str(world, obj, "turretstatechange", &[]);
            });
        }
    }
}

/// One bullet from `tag_flash` down its aim, spread by its ai spread: the
/// first thing it meets takes BO2's damage for that distance (attacker and
/// inflictor the turret, as BO2 has it: the Avogadro counts
/// `zombie_bullet_crouch_zm` hits from a non-player).
fn shoot(vm: &mut Vm<World>, world: &mut World, n: u32) {
    use crate::bullet_collision::{BulletTraceQuery, ColliderId, MASK_SHOT, TraceOutcome};
    let Some(t) = world
        .get_resource::<Turrets>()
        .and_then(|t| t.0.get(&n).cloned())
    else {
        return;
    };
    let Some((origin, angles)) = world
        .resource::<Zm>()
        .ents
        .get(&n)
        .map(|e| (e.origin, e.angles))
    else {
        return;
    };
    let muzzle = gsc_t6::math::add(
        pivot(origin, angles[1]),
        gsc_t6::math::scale(t.aim, AIM_TO_FLASH),
    );
    // A random way inside the spread cone.
    let (_, right, up) = gsc_t6::math::angle_vectors(gsc_t6::math::vector_to_angles(t.aim));
    let k = (AI_SPREAD_DEG.to_radians() * vm.rand_unit().sqrt()).tan();
    let turn = std::f32::consts::TAU * vm.rand_unit();
    let (c, s) = (turn.cos() * k, turn.sin() * k);
    let dir = gsc_t6::math::normalize(std::array::from_fn(|i| {
        t.aim[i] + right[i] * c + up[i] * s
    }));
    let end = gsc_t6::math::add(muzzle, gsc_t6::math::scale(dir, SHOT_RANGE));
    let own = world.resource::<Zm>().presences.by_ent.get(&n).map(|s| s.id);
    let outcome = frame(world).current_sensor_trace(BulletTraceQuery {
        start: muzzle,
        end,
        mask: MASK_SHOT,
        ignore: None,
        ignore_hit: None,
        ignore_model: own,
    });
    let TraceOutcome::Hit {
        end: point,
        collider:
            ColliderId::EntityDObjBone {
                owner,
                part_classification: part,
                ..
            },
        ..
    } = outcome
    else {
        return;
    };
    let Some(model) = owner.script_model() else {
        return;
    };
    let hit = world
        .resource::<Zm>()
        .presences
        .by_ent
        .iter()
        .find(|(_, s)| s.id == model)
        .map(|(n, _)| *n);
    let Some(hit) = hit.filter(|h| world.resource::<Zm>().actors.get(h).is_some_and(|a| a.alive))
    else {
        return;
    };
    let dist = gsc_t6::math::length(gsc_t6::math::sub(point, muzzle));
    let facts = super::weapon(world, &t.weapon)
        .ok()
        .and_then(|w| frame(world).combat_facts_for(w));
    let amount = facts.map_or(0.0, |f| {
        crate::bullet::bullet_damage_at_distance(&f, dist) as f32 * f.location_scale(part)
    });
    let amount = amount.round().max(1.0) as i32;
    let hitloc = super::T6_HITLOC_NAMES
        .get(usize::from(part))
        .copied()
        .unwrap_or("none");
    if log_on() {
        diag::info!(Sim, "bo2zm t6 turret ent{n}: hit ent{hit} {hitloc} at {dist:.0} for {amount}");
    }
    let me = Value::Object(t.obj);
    actors::damage(
        vm,
        world,
        hit,
        me.clone(),
        me,
        amount,
        0,
        "MOD_RIFLE_BULLET",
        &t.weapon,
        point,
        dir,
        hitloc,
    );
}
