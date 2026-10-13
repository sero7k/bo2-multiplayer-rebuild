//! IW4L_T6_USE=<targetname>[@<seconds>] (e.g. use_elec_switch, Buried's
//! power switch): a test aid for any use trigger the map names. At that
//! time (6 s unless given) the player stands in the trigger, looking at its
//! middle, and holds use for half a second; two seconds later the level
//! flag IW4L_T6_USE_FLAG (default power_on) and the trigger's hint are
//! logged, and again 35 s later (power events can take 30 s).
//! IW4L_T6_USE_STAY=1: he is not moved there. A comma list uses more than
//! one; a name no trigger has is looked up as a script struct, then as a
//! unitrigger stub (the banks' bank_deposit and bank_withdraw, TranZit's
//! weapon_locker). IW4L_T6_USE_HOLD=<seconds> holds use that long instead (a
//! revive takes 3 s). `mine:<field>` uses the player's own entity in that
//! field (`mine:buildableturbine`: picks his placed turbine up).
//! `<n>:` before an entry of IW4L_T6_USE, IW4L_T6_TP, IW4L_T6_GIVE or
//! IW4L_T6_PLACE drives player n (1-4) instead of player 1, in a co-op run
//! (TranZit's Maxis side wants two players' turbines at the tower:
//! `2:equip_turbine_zm@300@7700 -500 -60 180`).

use std::sync::{Mutex, OnceLock, PoisonError};

use bevy_ecs::prelude::World;
use gsc_t6::{ObjRef, Value, Vm};

use super::{Zm, frame, with_vm};
use crate::world::ClientId;

/// An entry's player (`<n>:` before it, 1-4; player 1 unless given) and the
/// rest of the entry.
fn who(entry: &str) -> (ClientId, &str) {
    let entry = entry.trim();
    match entry.split_once(':') {
        Some((n, rest)) if matches!(n.trim(), "1" | "2" | "3" | "4") => {
            (ClientId(n.trim().parse::<u32>().map_or(0, |n| n - 1)), rest.trim())
        }
        _ => (ClientId(0), entry),
    }
}

/// Each `name@secs` of IW4L_T6_USE: a comma list uses more than one (e.g.
/// TranZit's bank, `bank_deposit@60,bank_withdraw@70`).
fn wants() -> &'static [(ClientId, String, i64)] {
    static WANT: OnceLock<Vec<(ClientId, String, i64)>> = OnceLock::new();
    WANT.get_or_init(|| {
        let Ok(v) = std::env::var("IW4L_T6_USE") else {
            return Vec::new();
        };
        v.split(',')
            .map(|p| {
                let (client, p) = who(p);
                match p.split_once('@') {
                    Some((n, s)) => (
                        client,
                        n.trim().to_owned(),
                        s.trim().parse::<f32>().map_or(6000, |s| (s * 1000.0) as i64),
                    ),
                    None => (client, p.trim().to_owned(), 6000),
                }
            })
            .collect()
    })
}

/// The first script struct with that targetname: TranZit's bank and weapon
/// locker are structs a unitrigger is made on, not trigger entities.
fn struct_origin(world: &mut World, name: &str) -> Option<[f32; 3]> {
    // `name#N`: the Nth struct of that name (TranZit's safety lamps).
    let (name, nth) = match name.split_once('#') {
        Some((n, i)) => (n, i.trim().parse::<i32>().unwrap_or(0)),
        None => (name, 0),
    };
    with_vm(world, |vm, world| {
        let f = vm.intern("struct_class_names");
        let Value::Array(types) = vm.raw_field(vm.level, f) else {
            return None;
        };
        let Some(Value::Array(names)) = types.get(&gsc_t6::Key::Str(vm.intern("targetname"))) else {
            return None;
        };
        let Some(Value::Array(list)) = names.get(&gsc_t6::Key::Str(vm.intern(name))) else {
            return None;
        };
        let Some(Value::Object(s)) = list.get(&gsc_t6::Key::Int(nth)) else {
            return None;
        };
        let o = vm.intern("origin");
        match vm.get_field(world, s, o) {
            Value::Vec3(v) => Some(v),
            _ => None,
        }
    })
    .flatten()
}

/// `mine:<field>` in IW4L_T6_USE: the player's own entity in that field
/// (`2:mine:buildableturbine`: player 2's placed turbine, whose BO2 pickup
/// trigger has no name; TranZit's Maxis step C carries it to a lamp).
fn mine_origin(world: &mut World, client: ClientId, field: &str) -> Option<[f32; 3]> {
    let obj = world.resource::<Zm>().players.get(&client.0).map(|p| p.obj)?;
    with_vm(world, |vm, world| {
        let f = vm.intern(field);
        let v = vm.raw_field(obj, f);
        let n = super::entnum(vm, &v)?;
        world.resource::<Zm>().ents.get(&n).map(|e| e.origin)
    })
    .flatten()
}

/// The first unitrigger stub with that targetname (BO2 keeps them in
/// level._unitriggers.trigger_stubs and dynamic_stubs): TranZit's bank and
/// weapon locker are stubs on spawnstructs no map struct list holds.
fn stub_origin(world: &mut World, name: &str) -> Option<[f32; 3]> {
    with_vm(world, |vm, world| {
        let u = vm.intern("_unitriggers");
        let Value::Object(ut) = vm.raw_field(vm.level, u) else {
            return None;
        };
        let tn = vm.intern("targetname");
        let o = vm.intern("origin");
        for list in ["trigger_stubs", "dynamic_stubs"] {
            let f = vm.intern(list);
            let Value::Array(stubs) = vm.raw_field(ut, f) else {
                continue;
            };
            let stubs: Vec<Value> = stubs.read().values_in_order().cloned().collect();
            for s in stubs {
                let Value::Object(s) = s else {
                    continue;
                };
                if vm.to_text(&vm.raw_field(s, tn)) != name {
                    continue;
                }
                if let Value::Vec3(v) = vm.get_field(world, s, o) {
                    return Some(v);
                }
            }
        }
        None
    })
    .flatten()
}

/// IW4L_T6_USE_POINTS=<n>: half a second before the first use, player 0's
/// points are set to n (the bank takes 1000 a deposit).
fn use_points(world: &mut World, now: i64) {
    static POINTS: OnceLock<Option<i32>> = OnceLock::new();
    let Some(points) = *POINTS.get_or_init(|| std::env::var("IW4L_T6_USE_POINTS").ok()?.parse().ok()) else {
        return;
    };
    let Some(first) = wants().iter().map(|w| w.2).min() else {
        return;
    };
    let at = first - 500;
    if (at..at + i64::from(crate::MATCH_TICK_MS)).contains(&now) {
        super::autoplay::give_points(world, points);
        diag::info!(Sim, "bo2zm t6 use test: player 0 has {points} points");
    }
}

/// Player 0's points and bank account (`score`, `account_value`).
fn points(world: &mut World) -> String {
    with_vm(world, |vm, world| {
        let p = world.resource::<Zm>().players.get(&0).map(|p| p.obj)?;
        let (s, a) = (vm.intern("score"), vm.intern("account_value"));
        // `score` is the engine's field (the player state), not a script one.
        let score = vm.get_field(world, p, s);
        Some(format!(
            "points {}, bank {}",
            vm.to_text(&score),
            vm.to_text(&vm.raw_field(p, a))
        ))
    })
    .flatten()
    .unwrap_or_default()
}

/// IW4L_T6_TP="<x> <y> <z> [yaw [pitch]]@<seconds>": at that time the player is put
/// there (the console `tp` refuses: a zombies player is never "Alive" to it),
/// e.g. Buried's mansion `2547 326 260 180@20` for the ghosts. His origin is
/// logged two seconds later. A comma list moves him more than once:
/// `2916 581 240 90@10,-1100 700 8 0@60` (into the mansion, then out).
/// A script struct's targetname instead of the numbers puts him on it
/// (TranZit's navcomputer key spot: `sq_common_key@80`), else a trigger's:
/// on the floor in its middle (TranZit's tower volume, `sq_common_area`).
/// `name#N` picks the Nth struct of that name, and a yaw and pitch may
/// follow (`screecher_escape#2 0 85`: at a lamp, looking at his feet).
fn tp_test(world: &mut World, now: i64) {
    type Pose = Result<[f32; 5], (String, [f32; 2])>;
    static WANT: OnceLock<Vec<(ClientId, Pose, i64)>> = OnceLock::new();
    let want = WANT.get_or_init(|| {
        let Ok(v) = std::env::var("IW4L_T6_TP") else {
            return Vec::new();
        };
        v.split(',')
            .filter_map(|v| {
                let (client, v) = who(v);
                let (p, s) = v.split_once('@').unwrap_or((v, "20"));
                let n: Vec<f32> = p.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                let at = s.trim().parse::<f32>().map_or(20000, |s| (s * 1000.0) as i64);
                if n.len() >= 3 {
                    Some((client, Ok([n[0], n[1], n[2], n.get(3).copied().unwrap_or(0.0), n.get(4).copied().unwrap_or(0.0)]), at))
                } else {
                    // A name may carry a yaw and a pitch after it.
                    let mut t = p.split_whitespace();
                    let name = t.next()?;
                    let mut look = [0.0; 2];
                    for (l, v) in look.iter_mut().zip(t.filter_map(|s| s.parse().ok())) {
                        *l = v;
                    }
                    Some((client, Err((name.to_owned(), look)), at))
                }
            })
            .collect()
    });
    let tick = i64::from(crate::MATCH_TICK_MS);
    for (client, pose, at) in want {
        let (client, at) = (*client, *at);
        if (at..at + tick).contains(&now) {
            let spot = match pose {
                Ok(p) => Some(*p),
                Err((name, look)) => struct_origin(world, name)
                    .or_else(|| trigger_center(world, name).map(|c| ground(world, c)))
                    .map(|o| [o[0], o[1], o[2], look[0], look[1]]),
            };
            match spot {
                Some(p) => {
                    super::teleport_player(world, client, [p[0], p[1], p[2]]);
                    super::set_player_view(world, client, [p[4], p[3], 0.0]);
                }
                None => diag::warn!(Sim, "bo2zm t6 tp test: no struct or trigger {:?}", pose.as_ref().err().map(|e| &e.0)),
            }
        }
        if (at + 2000..at + 2000 + tick).contains(&now) {
            let o = frame(world).player(client).map(|p| p.origin).unwrap_or_default();
            diag::info!(
                Sim,
                "bo2zm t6 tp test: player at ({:.0} {:.0} {:.0})",
                o[0],
                o[1],
                o[2]
            );
        }
    }
    tp_on_grab(world, now);
}

/// IW4L_T6_TP_GRAB="<x> <y> <z>[@<seconds>]": each time a TranZit Denizen
/// is on his head, he is put there that long after (0.3 s unless given): a
/// solo grab lets go after three scratches, about 2 s, too quick for a timed
/// IW4L_T6_TP to reach a lamp. On his head = the Denizen's `linked_ent` is
/// set (his `screecher` is set before its leap, and moving him mid-leap
/// called the grab off), and only once the solo near miss is spent
/// (`level.near_miss` 2): the first grab always lets go after one scratch
/// before BO2 asks whether he stands in a lamp's light.
fn tp_on_grab(world: &mut World, now: i64) {
    use std::sync::atomic::{AtomicI64, Ordering::Relaxed};
    static WANT: OnceLock<Option<([f32; 3], i64)>> = OnceLock::new();
    static GRABBED_AT: AtomicI64 = AtomicI64::new(-1);
    let Some((spot, delay)) = *WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_TP_GRAB").ok()?;
        let (p, s) = v.split_once('@').unwrap_or((&v, "0.3"));
        let n: Vec<f32> = p.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        let delay = s.trim().parse::<f32>().map_or(300, |s| (s * 1000.0) as i64);
        (n.len() >= 3).then(|| ([n[0], n[1], n[2]], delay))
    }) else {
        return;
    };
    let grabbed = with_vm(world, |vm, world| {
        let p = world.resource::<Zm>().players.get(&0).map(|p| p.obj)?;
        let f = vm.intern("screecher");
        let Value::Object(s) = vm.get_field(world, p, f) else {
            return Some(false);
        };
        let linked = vm.intern("linked_ent");
        let miss = vm.intern("near_miss");
        let spent = vm.raw_field(vm.level, miss).as_int().unwrap_or(0) >= 2;
        Some(spent && !matches!(vm.get_field(world, s, linked), Value::Undefined))
    })
    .flatten()
    .unwrap_or(false);
    let at = GRABBED_AT.load(Relaxed);
    if !grabbed {
        GRABBED_AT.store(-1, Relaxed);
        return;
    }
    if at < 0 {
        GRABBED_AT.store(now, Relaxed);
        diag::info!(Sim, "bo2zm t6 tp test: grabbed at {now}");
    } else if (at + delay..at + delay + i64::from(crate::MATCH_TICK_MS)).contains(&now) {
        let client = ClientId(0);
        super::teleport_player(world, client, spot);
        diag::info!(Sim, "bo2zm t6 tp test: grabbed, put at ({:.0} {:.0} {:.0})", spot[0], spot[1], spot[2]);
    }
}

pub(crate) fn use_test(world: &mut World, now: i64) {
    tp_test(world, now);
    give_test(world, now);
    slot_test(world, now);
    use_points(world, now);
    for (client, name, at) in wants() {
        use_one(world, now, *client, name, *at);
    }
    emp_avo_test(world, now);
    avo_lure_test(world, now);
    avo_region_test(world, now);
    repair_test(world, now);
}

/// The window with the most boards torn off (BO2's `level.exterior_goals`,
/// each with its zbarrier and, once a board is off, its repair unitrigger
/// stub): the stub's origin, the zbarrier's and how many boards are off.
fn torn_window(world: &mut World) -> Option<([f32; 3], [f32; 3], usize)> {
    with_vm(world, |vm, world| {
        let f = vm.intern("exterior_goals");
        let Value::Array(goals) = vm.raw_field(vm.level, f) else {
            return None;
        };
        let goals: Vec<Value> = goals.read().values_in_order().cloned().collect();
        let (zf, sf, of) = (vm.intern("zbarrier"), vm.intern("unitrigger_stub"), vm.intern("origin"));
        let mut best: Option<([f32; 3], [f32; 3], usize)> = None;
        for g in goals {
            let Value::Object(g) = g else { continue };
            let Value::Object(stub) = vm.raw_field(g, sf) else { continue };
            let Value::Vec3(at) = vm.get_field(world, stub, of) else { continue };
            let Some(n) = super::entnum(vm, &vm.raw_field(g, zf)) else { continue };
            let zm = world.resource::<Zm>();
            let Some(e) = zm.ents.get(&n) else { continue };
            let Some(z) = &e.zbarrier else { continue };
            let torn = z.pieces.iter().filter(|p| p.state == "open").count();
            if torn > best.map_or(0, |b| b.2) {
                best = Some((at, e.origin, torn));
            }
        }
        best
    })
    .flatten()
}

/// IW4L_T6_REPAIR=<secs>: a window repair test. From then on, once a window
/// has two boards torn off (one, 30 s on), player 0 is put at its repair
/// trigger, looking at the window; 4 s later he holds use for
/// IW4L_T6_REPAIR_HOLD seconds (8), then lets go. Each second his points,
/// the round and his prompt are logged; zbarrier.rs logs each board put
/// back. IW4L_T6_GOD=1 keeps him up meanwhile.
fn repair_test(world: &mut World, now: i64) {
    static AT: OnceLock<Option<i64>> = OnceLock::new();
    let Some(from) = *AT.get_or_init(|| {
        let s = std::env::var("IW4L_T6_REPAIR").ok()?;
        s.trim().parse::<f32>().ok().map(|s| (s * 1000.0) as i64)
    }) else {
        return;
    };
    static HOLD: OnceLock<i64> = OnceLock::new();
    let hold = *HOLD.get_or_init(|| {
        std::env::var("IW4L_T6_REPAIR_HOLD")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .map_or(8000, |s| (s * 1000.0) as i64)
    });
    static START: Mutex<Option<i64>> = Mutex::new(None);
    let tick = i64::from(crate::MATCH_TICK_MS);
    if now < from {
        return;
    }
    let start = *START.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(s) = start else {
        if now % 500 >= tick {
            return;
        }
        let Some((stub, window, torn)) = torn_window(world) else {
            return;
        };
        if torn < 2 && now < from + 30_000 {
            return;
        }
        let feet = reach(world, stub);
        super::teleport_player(world, ClientId(0), feet);
        let eye = [feet[0], feet[1], feet[2] + 60.0];
        let d = [window[0] - eye[0], window[1] - eye[1], window[2] + 40.0 - eye[2]];
        let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
        super::set_player_view(
            world,
            ClientId(0),
            [-d[2].atan2(flat.max(1.0)).to_degrees(), d[1].atan2(d[0]).to_degrees(), 0.0],
        );
        diag::info!(
            Sim,
            "bo2zm t6 repair test: t{now} window ({:.0} {:.0} {:.0}) {torn} boards off; stub ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
            window[0],
            window[1],
            window[2],
            stub[0],
            stub[1],
            stub[2],
            feet[0],
            feet[1],
            feet[2]
        );
        *START.lock().unwrap_or_else(PoisonError::into_inner) = Some(now);
        return;
    };
    let held = (s + 4000..s + 4000 + hold).contains(&now);
    if held {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == ClientId(0) {
                cmd.buttons |= playerstate_iw4::buttons::USE;
            }
        }
    }
    if (now - s) % 1000 < tick && now - s <= 4000 + hold + 6000 {
        let round = with_vm(world, |vm, _| {
            let f = vm.intern("round_number");
            vm.to_text(&vm.raw_field(vm.level, f))
        })
        .unwrap_or_default();
        let hint = world.resource::<Zm>().hints.get(&0).cloned().unwrap_or_default();
        let torn = torn_window(world).map_or(0, |w| w.2);
        let points = points(world);
        diag::info!(
            Sim,
            "bo2zm t6 repair test: t{now} +{}s use {held}, round {round}, {points}, most boards off {torn}, hint {hint:?}",
            (now - s) / 1000
        );
    }
}

/// IW4L_T6_AVO_REGION="<region>@<from>-<until>" (`cornfield@200-320`): while
/// the Avogadro is up in his cloud, his current_region is that one. BO2 picks
/// it at random every 30 s and he comes down in that region's open zones, so
/// this only stands in for the roll a player would wait for (the cornfield is
/// next to the tower, the diner 13,000 units off).
fn avo_region_test(world: &mut World, now: i64) {
    static WANT: OnceLock<Option<(String, i64, i64)>> = OnceLock::new();
    let Some((region, from, until)) = WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_AVO_REGION").ok()?;
        let (region, when) = v.split_once('@')?;
        let (from, until) = when.split_once('-')?;
        let ms = |s: &str| s.trim().parse::<f32>().ok().map(|s| (s * 1000.0) as i64);
        Some((region.trim().to_owned(), ms(from)?, ms(until)?))
    }) else {
        return;
    };
    if !(*from..*until).contains(&now) {
        return;
    }
    let set = with_vm(world, |vm, _| {
        let f = vm.intern("avogadro");
        let Value::Object(a) = vm.raw_field(vm.level, f) else {
            return false;
        };
        let sf = vm.intern("state");
        let cf = vm.intern("current_region");
        let want = vm.intern(region);
        if vm.to_text(&vm.raw_field(a, sf)) != "cloud" || vm.to_text(&vm.raw_field(a, cf)) == *region {
            return false;
        }
        vm.set_raw_field(a, cf, Value::Str(want));
        true
    })
    .unwrap_or(false);
    if set {
        diag::info!(Sim, "bo2zm t6 avo region test: current_region = {region}");
    }
}

/// IW4L_T6_AVO_LURE="[<n>:]<x> <y> <z>@<from>-<until>" (TranZit's Maxis
/// step B, `2:7644 -464 -200@300-700`: player 2 brings the Avogadro to the
/// tower). BO2's Avogadro lands from his cloud in one region and only stays
/// if a player is in it within 30 s; once he chases, any player within 3000
/// keeps him. So, each second in the window: when he has landed and waits
/// (state cloud, `_in_cloud` 0) the player is put 120 units from him; while
/// he chases and comes within 700, the player is put 800 units further on
/// the zombies' path from him to (x y z), never past it, looking back at him.
fn avo_lure_test(world: &mut World, now: i64) {
    static WANT: OnceLock<Option<(ClientId, [f32; 3], i64, i64)>> = OnceLock::new();
    static LAST: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(i64::MIN);
    let Some((client, goal, from, until)) = *WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_AVO_LURE").ok()?;
        let (client, v) = who(&v);
        let (spot, when) = v.split_once('@')?;
        let n: Vec<f32> = spot.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        let (from, until) = when.split_once('-')?;
        let ms = |s: &str| s.trim().parse::<f32>().ok().map(|s| (s * 1000.0) as i64);
        (n.len() >= 3).then_some((client, [n[0], n[1], n[2]], ms(from)?, ms(until)?))
    }) else {
        return;
    };
    use std::sync::atomic::Ordering::Relaxed;
    if !(from..until).contains(&now) || now < LAST.load(Relaxed).saturating_add(1000) {
        return;
    }
    LAST.store(now, Relaxed);
    let Some((state, o)) = avogadro(world) else {
        return;
    };
    let in_cloud = with_vm(world, |vm, _| {
        let f = vm.intern("avogadro");
        let Value::Object(a) = vm.raw_field(vm.level, f) else {
            return true;
        };
        let c = vm.intern("_in_cloud");
        gsc_t6::truthy(&vm.raw_field(a, c))
    })
    .unwrap_or(true);
    let Some(me) = frame(world).player(client).map(|p| p.origin) else {
        return;
    };
    let gap = ((me[0] - o[0]).powi(2) + (me[1] - o[1]).powi(2)).sqrt();
    let ahead = match state.as_str() {
        "cloud" if !in_cloud && gap > 200.0 => 120.0,
        "idle" | "chasing" | "phasing" if gap < 700.0 => 800.0,
        _ => return,
    };
    // The zombies' path from him to the goal, and the spot that far along it.
    let spot = {
        let zm = world.resource::<Zm>();
        let nav = &zm.nav;
        let path = nav
            .nearest(o)
            .zip(nav.nearest(goal))
            .and_then(|(a, b)| nav.path(a, b))
            .unwrap_or_default();
        let mut at = o;
        let mut left = ahead;
        let mut spot = goal;
        for p in path.iter().map(|&i| nav.nodes[i as usize].origin).chain([goal]) {
            let d = ((p[0] - at[0]).powi(2) + (p[1] - at[1]).powi(2) + (p[2] - at[2]).powi(2)).sqrt();
            if d >= left {
                let t = left / d.max(1.0);
                spot = std::array::from_fn(|i| at[i] + (p[i] - at[i]) * t);
                break;
            }
            left -= d;
            at = p;
        }
        spot
    };
    let feet = ground(world, [spot[0], spot[1], spot[2] + 40.0]);
    if ((feet[0] - me[0]).powi(2) + (feet[1] - me[1]).powi(2)).sqrt() < 50.0 {
        return; // already there (at the goal, waiting for him)
    }
    super::teleport_player(world, client, feet);
    let yaw = super::autoplay::look(feet, o)[1];
    super::set_player_view(world, client, [0.0, yaw, 0.0]);
    diag::info!(
        Sim,
        "bo2zm t6 avo lure test: avogadro {state} at ({:.0} {:.0} {:.0}), player {} to ({:.0} {:.0} {:.0})",
        o[0],
        o[1],
        o[2],
        client.0 + 1,
        feet[0],
        feet[1],
        feet[2]
    );
}

/// When the running IW4L_T6_EMP_AVO throw started (ms), i64::MIN before the first.
static EMP_AVO: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(i64::MIN);

/// IW4L_T6_EMP_AVO's area (x, y, radius) and window (ms).
fn emp_avo_want() -> Option<([f32; 2], f32, i64, i64)> {
    static WANT: OnceLock<Option<([f32; 2], f32, i64, i64)>> = OnceLock::new();
    *WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_EMP_AVO").ok()?;
        let (area, when) = v.split_once('@')?;
        let n: Vec<f32> = area.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        let (from, until) = when.split_once('-')?;
        let ms = |s: &str| s.trim().parse::<f32>().ok().map(|s| (s * 1000.0) as i64);
        (n.len() >= 3).then_some(([n[0], n[1]], n[2], ms(from)?, ms(until)?))
    })
}

/// True for the first 3 s of an IW4L_T6_EMP_AVO throw (autoplay lets go).
fn emp_avo_busy(now: i64) -> bool {
    let s = EMP_AVO.load(std::sync::atomic::Ordering::Relaxed);
    s != i64::MIN && (s..s + 3000).contains(&now)
}

/// level.avogadro's state and feet, while it exists.
fn avogadro(world: &mut World) -> Option<(String, [f32; 3])> {
    with_vm(world, |vm, world| {
        let f = vm.intern("avogadro");
        let v = vm.raw_field(vm.level, f);
        let Value::Object(o) = &v else {
            return None;
        };
        let sf = vm.intern("state");
        let state = vm.to_text(&vm.raw_field(*o, sf));
        let n = super::entnum(vm, &v)?;
        let feet = world.resource::<Zm>().ents.get(&n)?.origin;
        Some((state, feet))
    })
    .flatten()
}

/// IW4L_T6_EMP_AVO="<x> <y> <r>@<from>-<until>" (TranZit's Maxis step B,
/// `7644 -464 150@300-900`: the tower's middle): a test aid that stuns the
/// Avogadro with an EMP grenade where BO2 counts it. Between those seconds,
/// whenever it is out (not in its chamber or cloud, exiting or phasing,
/// which BO2's stun ignores) within r of (x y), at most every 25 s: player 1
/// is given an emp_grenade_zm, a second later stands 150 units from it on
/// his own side looking at his feet, and throws it (smoke, 0.3 s). Each
/// throw logs the Avogadro's state and spot, and again 5 s later.
fn emp_avo_test(world: &mut World, now: i64) {
    use std::sync::atomic::Ordering::Relaxed;
    let Some((spot, r, from, until)) = emp_avo_want() else {
        return;
    };
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let start = EMP_AVO.load(Relaxed);
    if start == i64::MIN || now >= start + 25_000 {
        if !(from..until).contains(&now) {
            return;
        }
        let Some((state, o)) = avogadro(world) else {
            return;
        };
        let out = !matches!(state.as_str(), "phasing" | "chamber" | "wait_for_player" | "exiting" | "cloud");
        if !out || ((o[0] - spot[0]).powi(2) + (o[1] - spot[1]).powi(2)).sqrt() >= r {
            return;
        }
        EMP_AVO.store(now, Relaxed);
        with_vm(world, |vm, world| {
            if let Some(p) = world.resource::<Zm>().players.get(&client.0).map(|p| p.obj) {
                let g = vm.string("emp_grenade_zm");
                vm.spawn_named(world, "maps/mp/zombies/_zm_weapons", "weapon_give", Value::Object(p), vec![g]);
            }
        });
        diag::info!(
            Sim,
            "bo2zm t6 emp avo test: throw, avogadro {state} at ({:.0} {:.0} {:.0})",
            o[0],
            o[1],
            o[2]
        );
        return;
    }
    if (start + 1000..start + 1000 + tick).contains(&now) {
        if let Some((_, o)) = avogadro(world) {
            let me = frame(world).player(client).map_or(o, |p| p.origin);
            let (dx, dy) = (me[0] - o[0], me[1] - o[1]);
            let len = (dx * dx + dy * dy).sqrt();
            let (ux, uy) = if len > 1.0 { (dx / len, dy / len) } else { (1.0, 0.0) };
            let feet = ground(world, [o[0] + ux * 150.0, o[1] + uy * 150.0, o[2] + 40.0]);
            super::teleport_player(world, client, feet);
            let yaw = super::autoplay::look(feet, o)[1];
            super::set_player_view(world, client, [85.0, yaw, 0.0]);
        }
    }
    if (start + 1300..start + 1600).contains(&now) {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= playerstate_iw4::buttons::SMOKE;
            }
        }
    }
    if (start + 5000..start + 5000 + tick).contains(&now) {
        let a = avogadro(world);
        diag::info!(Sim, "bo2zm t6 emp avo test: +5s avogadro {a:?}");
    }
}

/// The middle of the first trigger entity with that targetname.
fn trigger_center(world: &mut World, name: &str) -> Option<[f32; 3]> {
    let trig = with_vm(world, |vm, world| {
        let tn = vm.intern("targetname");
        world
            .resource::<Zm>()
            .ents
            .values()
            .find(|e| {
                e.classname.starts_with("trigger")
                    && e.obj
                        .is_some_and(|o| vm.to_text(&vm.raw_field(o, tn)) == name)
            })
            .cloned()
    })
    .flatten()?;
    Some(super::triggers::center(world, &trig))
}

/// Where he stands to use something at `c`: on the floor under it when
/// BO2's unitrigger manager would offer it from there
/// (`get_closest_unitriggers` skips a stub more than 42 units off his
/// origin + 35 in height), else the first floor spot 24 or 40 units around
/// it that it would. TranZit's bank deposit is on its counter: on top of it
/// he stood 48 too high and was never offered it (runs/bk3).
fn reach(world: &mut World, c: [f32; 3]) -> [f32; 3] {
    let mut first = None;
    for r in [0.0f32, 24.0, 40.0] {
        for k in 0..if r == 0.0 { 1 } else { 8 } {
            let a = (k as f32 * 45.0).to_radians();
            let stand = [c[0] + r * a.cos(), c[1] + r * a.sin(), c[2] + 20.0];
            let down = [stand[0], stand[1], stand[2] - 300.0];
            let t = frame(world).trace_static_world(
                stand,
                down,
                crate::bullet_collision::PLAYER_MINS,
                crate::bullet_collision::PLAYER_MAXS,
                crate::bullet_collision::MASK_PLAYER_SOLID,
            );
            let feet: [f32; 3] = std::array::from_fn(|i| stand[i] + (down[i] - stand[i]) * t.fraction);
            if r == 0.0 {
                first = Some(feet);
            } else if t.startsolid != 0 || t.fraction >= 1.0 {
                continue;
            }
            if (feet[2] + 35.0 - c[2]).abs() <= 42.0 {
                return feet;
            }
        }
    }
    first.unwrap_or(c)
}

/// IW4L_T6_SLOT=<1-4>@<seconds> (comma list): player 0 presses that d-pad
/// slot and raises the weapon `setactionslot` put there, as the client's
/// switch does (TranZit's Jet Gun after it is taken off the table: BO2's
/// `equipment_give` puts it on slot 1 and does not raise it).
fn slot_test(world: &mut World, now: i64) {
    static WANT: OnceLock<Vec<(usize, i64)>> = OnceLock::new();
    let want = WANT.get_or_init(|| {
        let Ok(v) = std::env::var("IW4L_T6_SLOT") else {
            return Vec::new();
        };
        v.split(',')
            .filter_map(|p| {
                let (n, s) = p.split_once('@')?;
                let slot = n.trim().parse::<usize>().ok()?.checked_sub(1)?;
                let at = (s.trim().parse::<f32>().ok()? * 1000.0) as i64;
                (slot < 4).then_some((slot, at))
            })
            .collect()
    });
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    for &(slot, at) in want {
        if !(at..at + tick).contains(&now) {
            continue;
        }
        let Some(ps) = frame(world).player(client).copied() else {
            continue;
        };
        let w = ps.action_slot_param[slot];
        if ps.action_slot_type[slot] != 1 || w <= 0 || !ps.weapons.contains(&w) {
            diag::warn!(Sim, "bo2zm t6 slot test: slot {} holds no weapon of his", slot + 1);
            continue;
        }
        crate::script_player::switch_to_weapon(&mut frame(world), client, w as u32);
        let name = super::weapon_text(world, w as u32);
        diag::info!(Sim, "bo2zm t6 slot test: slot {} raised {name}", slot + 1);
    }
}

/// IW4L_T6_GIVE=<weapon>@<seconds> (comma list): at that time player 0 is
/// given the weapon through BO2's `weapon_give` and a second later raises
/// it, as a player presses its slot (TranZit's Jet Gun at the navcomputer's
/// tower; a box gun for the weapon locker, which refuses his M1911).
fn give_test(world: &mut World, now: i64) {
    static WANT: OnceLock<Vec<(ClientId, String, i64)>> = OnceLock::new();
    let want = WANT.get_or_init(|| {
        let Ok(v) = std::env::var("IW4L_T6_GIVE") else {
            return Vec::new();
        };
        v.split(',')
            .filter_map(|p| {
                let (client, p) = who(p);
                let (n, s) = p.split_once('@')?;
                let at = (s.trim().parse::<f32>().ok()? * 1000.0) as i64;
                Some((client, n.trim().to_owned(), at))
            })
            .collect()
    });
    let tick = i64::from(crate::MATCH_TICK_MS);
    for (client, gun, at) in want {
        let client = *client;
        if (*at..at + tick).contains(&now) {
            with_vm(world, |vm, world| {
                if let Some(p) = world.resource::<Zm>().players.get(&client.0).map(|p| p.obj) {
                    let g = vm.string(gun);
                    vm.spawn_named(world, "maps/mp/zombies/_zm_weapons", "weapon_give", Value::Object(p), vec![g]);
                }
            });
            diag::info!(Sim, "bo2zm t6 give test: given {gun}");
        }
        // A grenade stays on its offhand slot (PRESS frag / smoke throws it).
        let offhand = gun.contains("grenade") || gun.contains("monkey");
        if !offhand && (at + 1000..at + 1000 + tick).contains(&now) {
            let held = frame(world).player(client).map(|p| p.weapon as u32);
            match super::weapon(world, gun) {
                Ok(w) if held != Some(w) => {
                    crate::script_player::switch_to_weapon(&mut frame(world), client, w);
                    diag::info!(Sim, "bo2zm t6 give test: raised {gun}");
                }
                Ok(_) => diag::info!(Sim, "bo2zm t6 give test: {gun} already up"),
                Err(e) => diag::warn!(Sim, "bo2zm t6 give test: {gun}: {e}"),
            }
        }
    }
}

fn use_one(world: &mut World, now: i64, client: ClientId, name: &str, at: i64) {
    let tick = i64::from(crate::MATCH_TICK_MS);
    if (at..at + tick).contains(&now) {
        let found = match name.strip_prefix("mine:") {
            // BO2's pickup trigger sits 12 units over the placed equipment.
            Some(field) => mine_origin(world, client, field).map(|o| [o[0], o[1], o[2] + 12.0]),
            None => trigger_center(world, name)
                .or_else(|| struct_origin(world, name))
                .or_else(|| stub_origin(world, name)),
        };
        let Some(c) = found else {
            diag::warn!(Sim, "bo2zm t6 use test: no trigger or struct named {name}");
            return;
        };
        // IW4L_T6_USE_STAY=1: he presses use where he stands (in the bus at
        // its door: the trigger rides with it).
        if std::env::var_os("IW4L_T6_USE_STAY").is_some() {
            diag::info!(
                Sim,
                "bo2zm t6 use test: {name} at ({:.0} {:.0} {:.0}); player stays",
                c[0],
                c[1],
                c[2]
            );
            return;
        }
        let feet = reach(world, c);
        super::teleport_player(world, client, feet);
        let eye = [feet[0], feet[1], feet[2] + 60.0];
        let d = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
        let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let view = [
            -d[2].atan2(flat.max(1.0)).to_degrees(),
            d[1].atan2(d[0]).to_degrees(),
            0.0,
        ];
        super::set_player_view(world, client, view);
        diag::info!(
            Sim,
            "bo2zm t6 use test: {name} at ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
            c[0],
            c[1],
            c[2],
            feet[0],
            feet[1],
            feet[2]
        );
    }
    static HOLD: OnceLock<i64> = OnceLock::new();
    let hold = *HOLD.get_or_init(|| {
        std::env::var("IW4L_T6_USE_HOLD")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .map_or(500, |s| (s * 1000.0) as i64)
    });
    if (at + 1000..at + 1000 + hold).contains(&now) {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= playerstate_iw4::buttons::USE;
            }
        }
    }
    // again at +35 s: a power event can take 30 s (Tranzit's reactor)
    if (at + 3000..at + 3000 + tick).contains(&now)
        || (at + 35000..at + 35000 + tick).contains(&now)
    {
        let flag = std::env::var("IW4L_T6_USE_FLAG").unwrap_or_else(|_| "power_on".to_owned());
        let on = with_vm(world, |vm, _| {
            let f = vm.intern("flag");
            match vm.raw_field(vm.level, f) {
                gsc_t6::Value::Array(a) => a
                    .get(&gsc_t6::Key::Str(vm.intern(&flag)))
                    .is_some_and(|v| gsc_t6::truthy(&v)),
                _ => false,
            }
        })
        .unwrap_or(false);
        let hint = world
            .resource::<Zm>()
            .hints
            .get(&client.0)
            .cloned()
            .unwrap_or_default();
        let points = points(world);
        diag::info!(
            Sim,
            "bo2zm t6 use test: {name} used, flag {flag} {on}, hint {hint:?}, {points}"
        );
    }
}

/// IW4L_T6_BUILD=<buildable>[@<seconds>] (turbine, springpad_zm,
/// subwoofer_zm, headchopper_zm ...): a test aid for BO2's buildables.
/// From that time (15 s unless given) every 12 s: the player stands on the
/// next part still lying in the world (`level.buildable_stubs`, the stub's
/// `buildablezone.pieces`), presses use (pick up), then stands at the
/// stub's table looking at it and holds use 4 s (build). When every part is
/// built he presses use there once (take it). Every step is logged.
/// A comma list builds one after another, e.g. `powerswitch@20,pap@200`
/// (each takes up to 132 s).
pub(crate) fn build_test(world: &mut World, now: i64) {
    static WANT: OnceLock<Vec<(String, i64)>> = OnceLock::new();
    static PART: Mutex<Option<ObjRef>> = Mutex::new(None);
    static HELD: Mutex<bool> = Mutex::new(false);
    // A comma list runs one after another, each from its own time
    // (Buried: `keys_zm@15,sloth@80` opens Leroy's cell, then feeds him).
    let want = WANT.get_or_init(|| {
        let Ok(v) = std::env::var("IW4L_T6_BUILD") else {
            return Vec::new();
        };
        v.split(',')
            .map(|v| match v.split_once('@') {
                Some((n, s)) => (
                    n.to_owned(),
                    s.parse::<f32>().map_or(15000, |s| (s * 1000.0) as i64),
                ),
                None => (v.to_owned(), 15000),
            })
            .collect()
    });
    let Some((name, start)) = want.iter().rfind(|(_, s)| *s <= now) else {
        return;
    };
    let (name, start) = (name.as_str(), *start);
    // `<table>:<other>`: the parts come from the other buildable's stubs
    // (Buried: `sloth:booze` takes the bottle lying in the world, from the
    // `booze` buildable, and gives it to Leroy).
    let (name, from) = name.split_once(':').unwrap_or((name, name));
    // `<table>#<k>`: the k-th stub of that name (Die Rise's elevator key
    // consoles: `ekeys_zm#3:keys_zm` calls the car of the 4th console).
    let (name, which) = name
        .split_once('#')
        .map_or((name, 0), |(n, k)| (n, k.parse().unwrap_or(0)));
    if now < start {
        return;
    }
    // The place test takes over at its time (he carries what was built);
    // a build listed after that placement runs as usual.
    if place_times().iter().any(|&at| at >= start && now >= at) {
        return;
    }
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let (k, t) = ((now - start) / 12000, (now - start) % 12000);
    if k > 10 {
        return;
    }
    let Some((stub_origin, stub_angles, parts)) =
        with_vm(world, |vm, world| find_stub(vm, world, name, which)).flatten()
    else {
        if t < tick {
            diag::warn!(Sim, "bo2zm t6 build test: no buildable stub {name}");
        }
        return;
    };
    if t < tick {
        if k == 0 {
            log_stubs(world);
        }
        // The next part still lying in the world.
        // Every stub of that buildable (Buried makes several `sloth` stubs).
        let source = with_vm(world, |vm, world| find_parts(vm, world, from)).unwrap_or_default();
        let next = source
            .iter()
            .find(|p| !p.built && p.model.is_some())
            .cloned();
        *PART.lock().unwrap_or_else(PoisonError::into_inner) = next.as_ref().map(|p| p.obj);
        *HELD.lock().unwrap_or_else(PoisonError::into_inner) = false;
        let summary: Vec<String> = source
            .iter()
            .map(|p| {
                let state = if p.built {
                    "(built)"
                } else if p.model.is_none() {
                    "(away)"
                } else {
                    ""
                };
                format!("{}{state}", p.modelname)
            })
            .collect();
        diag::info!(
            Sim,
            "bo2zm t6 build test: {name} round {k}: parts {summary:?}"
        );
        match next {
            Some(p) => {
                let at = p.model.unwrap_or_default();
                let feet = stand_by_part(world, at);
                super::teleport_player(world, client, feet);
                super::set_player_view(
                    world,
                    client,
                    look_at([feet[0], feet[1], feet[2] + 60.0], at),
                );
                diag::info!(
                    Sim,
                    "bo2zm t6 build test: to part {} at ({:.0} {:.0} {:.0})",
                    p.modelname,
                    at[0],
                    at[1],
                    at[2]
                );
            }
            None => {
                stand_at(world, client, name, stub_origin, stub_angles);
                // A part already in his hands (Die Rise's one elevator key,
                // picked for another console): hold use to put it in.
                let held = held_parts(world);
                *HELD.lock().unwrap_or_else(PoisonError::into_inner) = !held.is_empty();
                diag::info!(
                    Sim,
                    "bo2zm t6 build test: no part left; at the table to take it (holding {held:?})"
                );
            }
        }
    }
    let held = *HELD.lock().unwrap_or_else(PoisonError::into_inner);
    if held && t >= 1000 {
        // Die Rise's call consoles prompt only while the car is away from
        // that stop (and the cars move on their own): hold use whenever the
        // prompt shows, logging each change of prompt.
        static LAST_HINT: Mutex<Option<String>> = Mutex::new(None);
        let hint = world
            .resource::<Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        let mut last = LAST_HINT.lock().unwrap_or_else(PoisonError::into_inner);
        if last.as_deref() != Some(hint.as_str()) {
            let held_now = held_parts(world);
            diag::info!(
                Sim,
                "bo2zm t6 build test: at {:.1} s hint {hint:?}, holding {held_now:?}",
                now as f32 / 1000.0
            );
            *last = Some(hint.clone());
        }
        if !hint.is_empty() {
            press(world, client);
        }
    }
    let have_part = PART
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some();
    // A teleport crosses no zone entry trigger, and BO2's unitriggers serve
    // only stubs in active zones: until the prompt shows, put the part's
    // zone (then the table's) in the active list, as walking there would.
    let no_hint = world
        .resource::<Zm>()
        .hints
        .get(&0)
        .is_none_or(|h| h.is_empty());
    if have_part && no_hint && (t < 2900 || (3000..9000).contains(&t)) {
        let part = *PART.lock().unwrap_or_else(PoisonError::into_inner);
        let stub = with_vm(world, |vm, _| {
            if t < 2900 {
                let ut = vm.intern("unitrigger");
                return vm.raw_field(part?, ut).as_obj();
            }
            let eq = vm.intern("equipname");
            buildable_stubs(vm)
                .into_iter()
                .filter(|&s| vm.to_text(&vm.raw_field(s, eq)) == name)
                .nth(which)
        })
        .flatten();
        if let Some(stub) = stub {
            wake_zone(world, stub);
        }
    }
    // Pick up (or take) once the prompt shows: press 0.5 s from then.
    static PICK_FROM: Mutex<Option<i64>> = Mutex::new(None);
    if (1000..2900).contains(&t) {
        let mut from = PICK_FROM.lock().unwrap_or_else(PoisonError::into_inner);
        if from.is_none_or(|f| f > now || f < now - (t - 1000)) {
            let shown = world
                .resource::<Zm>()
                .hints
                .get(&0)
                .is_some_and(|h| !h.is_empty());
            *from = shown.then_some(now);
        }
        if from.is_some_and(|f| now - f < 500) {
            press(world, client);
        }
    }
    // No floor near the part puts his origin + 35 within 42 of it (TranZit's
    // transceiver on the diner shelf): hop, as a player does there. BO2
    // lists the part while he is up (get_closest_unitriggers: 3D), then
    // keeps it while he stays within its radius on the flat
    // (unitrigger_thread: distance2dsquared only).
    if have_part
        && HOP.load(std::sync::atomic::Ordering::Relaxed)
        && [300, 1300, 2300].iter().any(|&h| (h..h + 2 * tick).contains(&t))
    {
        press_button(world, client, playerstate_iw4::buttons::JUMP);
    }
    // Probe (ee4: no prompt at all while hopping): his height and the
    // prompt every 0.1 s.
    if have_part
        && HOP.load(std::sync::atomic::Ordering::Relaxed)
        && (0..2900).contains(&t)
        && t % 100 < tick
    {
        let z = frame(world)
            .player(client)
            .map(|ps| (ps.origin[2], ps.ground_entity_num));
        let hint = world
            .resource::<Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        diag::info!(Sim, "bo2zm t6 build test: hop t {t}: feet/ground {z:?}, hint {hint:?}");
    }
    if have_part && (900..900 + tick).contains(&t) {
        let part = *PART.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(p) = part {
            log_part_trigger(world, client, p);
        }
    }
    if have_part && (3000..3000 + tick).contains(&t) {
        let held = held_parts(world);
        let hint = world
            .resource::<Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        diag::info!(Sim, "bo2zm t6 build test: holding {held:?}, hint {hint:?}");
        stand_at(world, client, name, stub_origin, stub_angles);
    }
    if have_part && (4000..4000 + tick).contains(&t) {
        let hint = world
            .resource::<Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        diag::info!(Sim, "bo2zm t6 build test: at the table, hint {hint:?}");
        log_table_check(world, client, name, which);
    }
    // Hold use 4.3 s from when the table's prompt shows (the unitrigger
    // can take a few seconds to reach him where many stubs are near, and a
    // use already held when it appears does not count).
    static HOLD_FROM: Mutex<Option<i64>> = Mutex::new(None);
    if have_part && (4200..9000).contains(&t) {
        let mut from = HOLD_FROM.lock().unwrap_or_else(PoisonError::into_inner);
        if from.is_none_or(|f| f > now || f < now - (t - 4200)) {
            let shown = world
                .resource::<Zm>()
                .hints
                .get(&0)
                .is_some_and(|h| !h.is_empty());
            *from = shown.then_some(now);
        }
        if from.is_some_and(|f| now - f < 4300) {
            press(world, client);
        }
    }
    if (11000..11000 + tick).contains(&t) {
        let built = parts.iter().filter(|p| p.built).count();
        let names: Vec<String> = {
            let f = frame(world);
            crate::script_player::weapons(&f, client, crate::script_player::WeaponList::All)
                .into_iter()
                .map(|w| crate::script_player::weapon_name(&f, w))
                .collect()
        };
        let hint = world
            .resource::<Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        diag::info!(
            Sim,
            "bo2zm t6 build test: {name} {built}/{} built (as read at the round start); weapons {names:?}, hint {hint:?}, power_on {}",
            parts.len(),
            level_flag(world, "power_on")
        );
    }
}

/// What walking into a unitrigger stub's zone would have done, at once.
/// A teleport crosses no zone entry trigger (TranZit's OnFarm_enter), and
/// BO2's unitriggers only serve stubs whose zone is in
/// `level.active_zone_names`, which `manage_zones` refreshes only every
/// 2-4 s on TranZit (0.05 s per 3 zones, then 1 s). Enable the zone as its
/// mode script would, mark it active, and add it to the list now; the next
/// refresh keeps it, as he stands in it (one already under way can drop it
/// once, so the build test calls this again until the prompt shows).
fn wake_zone(world: &mut World, stub: ObjRef) -> Option<String> {
    let z = with_vm(world, |vm, world| {
        let (iz, zones, act, azn) = (
            vm.intern("in_zone"),
            vm.intern("zones"),
            vm.intern("is_active"),
            vm.intern("active_zone_names"),
        );
        let en = vm.intern("is_enabled");
        let zv = vm.raw_field(stub, iz);
        if matches!(zv, Value::Undefined) {
            return None;
        }
        let text = vm.to_text(&zv);
        let level = vm.level;
        let key = gsc_t6::Key::Str(vm.intern(&text));
        let zone = match vm.raw_field(level, zones) {
            Value::Array(a) => a.get(&key).and_then(|v| v.as_obj()),
            _ => None,
        };
        let enabled = zone.is_some_and(|z| gsc_t6::truthy(&vm.raw_field(z, en)));
        if !enabled {
            vm.spawn_named(
                world,
                "maps/mp/zombies/_zm_zonemgr",
                "enable_zone",
                Value::Object(level),
                vec![zv.clone()],
            );
        }
        if let Some(zone) = zone {
            vm.set_raw_field(zone, act, Value::Int(1));
        }
        let mut list = match vm.raw_field(level, azn) {
            Value::Array(a) => a.snapshot(),
            _ => gsc_t6::Array::default(),
        };
        let had = list.values_in_order().any(|v| vm.to_text(v) == text);
        if !had {
            list.push(zv);
            vm.set_raw_field(level, azn, Value::Array(gsc_t6::ArrayRef::new(list)));
        }
        Some((text, had, enabled))
    })
    .flatten();
    if let Some((z, had, enabled)) = &z
        && !(*had && *enabled)
    {
        diag::info!(
            Sim,
            "bo2zm t6 build test: zone {z} made active (as walking in would; was enabled {enabled}, listed {had})"
        );
    }
    z.map(|(z, ..)| z)
}

/// `level.flag[name]` is set.
fn level_flag(world: &mut World, name: &str) -> bool {
    with_vm(world, |vm, _| {
        let f = vm.intern("flag");
        match vm.raw_field(vm.level, f) {
            Value::Array(a) => a
                .get(&gsc_t6::Key::Str(vm.intern(name)))
                .is_some_and(|v| gsc_t6::truthy(&v)),
            _ => false,
        }
    })
    .unwrap_or(false)
}

#[derive(Clone)]
struct Part {
    obj: ObjRef,
    modelname: String,
    built: bool,
    /// Where its model lies (none: carried, or not spawned).
    model: Option<[f32; 3]>,
}

fn buildable_stubs(vm: &mut Vm<World>) -> Vec<ObjRef> {
    let f = vm.intern("buildable_stubs");
    match vm.raw_field(vm.level, f) {
        Value::Array(a) => a
            .read()
            .values_in_order()
            .filter_map(Value::as_obj)
            .collect(),
        _ => Vec::new(),
    }
}

fn origin_or_field(vm: &mut Vm<World>, world: &mut World, o: ObjRef) -> [f32; 3] {
    let f = vm.intern("origin");
    // A stub riding a vehicle (TranZit's bus ladder and hatch) is where its
    // `origin_parent` trigger is now, as BO2's anystub_get_unitrigger_origin.
    let parent = vm.intern("origin_parent");
    let parent = vm.raw_field(o, parent);
    super::origin_of(vm, world, &parent)
        .or_else(|| super::origin_of(vm, world, &Value::Object(o)))
        .or_else(|| vm.raw_field(o, f).as_vec3())
        .unwrap_or_default()
}

/// Every buildable stub: its buildable, place and whether it is built.
fn log_stubs(world: &mut World) {
    with_vm(world, |vm, world| {
        let (eq, built) = (vm.intern("equipname"), vm.intern("built"));
        let (sn, sp) = (vm.intern("script_noteworthy"), vm.intern("script_parameters"));
        for s in buildable_stubs(vm) {
            let o = origin_or_field(vm, world, s);
            diag::info!(
                Sim,
                "bo2zm t6 build test: stub {} at ({:.0} {:.0} {:.0}) built {} ({} {})",
                vm.to_text(&vm.raw_field(s, eq)),
                o[0],
                o[1],
                o[2],
                vm.to_text(&vm.raw_field(s, built)),
                vm.to_text(&vm.raw_field(s, sn)),
                vm.to_text(&vm.raw_field(s, sp))
            );
        }
    });
}

/// The parts player 0 carries (`current_buildable_pieces`), by model.
pub(super) fn held_parts(world: &mut World) -> Vec<String> {
    with_vm(world, |vm, world| {
        let p = world.resource::<Zm>().players.get(&0).map(|p| p.obj)?;
        let (cur, mn) = (
            vm.intern("current_buildable_pieces"),
            vm.intern("modelname"),
        );
        let Value::Array(a) = vm.raw_field(p, cur) else {
            return Some(Vec::new());
        };
        let vals: Vec<Value> = a.read().values_in_order().cloned().collect();
        Some(
            vals.iter()
                .filter_map(Value::as_obj)
                .map(|o| vm.to_text(&vm.raw_field(o, mn)))
                .collect(),
        )
    })
    .flatten()
    .unwrap_or_default()
}

/// The parts of every stub of that buildable.
fn find_parts(vm: &mut Vm<World>, world: &mut World, name: &str) -> Vec<Part> {
    let eq = vm.intern("equipname");
    let stubs: Vec<ObjRef> = buildable_stubs(vm)
        .into_iter()
        .filter(|&s| vm.to_text(&vm.raw_field(s, eq)) == name)
        .collect();
    stubs
        .into_iter()
        .flat_map(|s| stub_parts(vm, world, s))
        .collect()
}

/// The stub of that buildable: its table's place and angles, its parts.
fn find_stub(
    vm: &mut Vm<World>,
    world: &mut World,
    name: &str,
    which: usize,
) -> Option<([f32; 3], [f32; 3], Vec<Part>)> {
    let (eq, zone, angles) = (
        vm.intern("equipname"),
        vm.intern("buildablezone"),
        vm.intern("angles"),
    );
    let stub = buildable_stubs(vm)
        .into_iter()
        .filter(|&s| vm.to_text(&vm.raw_field(s, eq)) == name)
        .nth(which)?;
    let origin = origin_or_field(vm, world, stub);
    let ang = vm.raw_field(stub, angles).as_vec3().unwrap_or_default();
    vm.raw_field(stub, zone).as_obj()?;
    Some((origin, ang, stub_parts(vm, world, stub)))
}

fn stub_parts(vm: &mut Vm<World>, world: &mut World, stub: ObjRef) -> Vec<Part> {
    let (zone, pieces, mn, built, model) = (
        vm.intern("buildablezone"),
        vm.intern("pieces"),
        vm.intern("modelname"),
        vm.intern("built"),
        vm.intern("model"),
    );
    let Some(z) = vm.raw_field(stub, zone).as_obj() else {
        return Vec::new();
    };
    let Value::Array(a) = vm.raw_field(z, pieces) else {
        return Vec::new();
    };
    let objs: Vec<ObjRef> = a
        .read()
        .values_in_order()
        .filter_map(Value::as_obj)
        .collect();
    let parts = objs
        .into_iter()
        .map(|p| {
            let m = vm.raw_field(p, model);
            Part {
                obj: p,
                modelname: vm.to_text(&vm.raw_field(p, mn)),
                built: gsc_t6::truthy(&vm.raw_field(p, built)),
                model: super::origin_of(vm, world, &m),
            }
        })
        .collect();
    parts
}

/// Why holding use at a table may stop at once: the checks of BO2's
/// `player_continue_building` on the table's trigger for this player
/// (touching it, looking at its origin within 0.4, a clear line from it
/// to the eye).
fn log_table_check(world: &mut World, client: ClientId, name: &str, which: usize) {
    let trig = with_vm(world, |vm, _| {
        let (eq, pt) = (vm.intern("equipname"), vm.intern("playertrigger"));
        let stub = buildable_stubs(vm)
            .into_iter()
            .filter(|&s| vm.to_text(&vm.raw_field(s, eq)) == name)
            .nth(which)?;
        let Value::Array(a) = vm.raw_field(stub, pt) else {
            return None;
        };
        let v = a.read().values_in_order().next().cloned()?;
        super::entnum(vm, &v)
    })
    .flatten();
    let Some(e) = trig.and_then(|n| world.resource::<Zm>().ents.get(&n).cloned()) else {
        log_stub_zone(world, name);
        return;
    };
    let touching = super::triggers::player_box(world, client.0)
        .is_some_and(|(lo, hi)| super::triggers::touches(world, &e, lo, hi));
    let Some((eye, angles)) = frame(world).player(client).map(|ps| {
        let o = ps.origin;
        ([o[0], o[1], o[2] + ps.view_height_current], ps.viewangles)
    }) else {
        return;
    };
    let (fw, _, _) = gsc_t6::math::angle_vectors(angles);
    let d = gsc_t6::math::sub(e.origin, eye);
    let len = gsc_t6::math::dot(d, d).sqrt().max(1e-3);
    let dot = gsc_t6::math::dot(d, fw) / len;
    let t = frame(world).trace_static_world(
        e.origin,
        eye,
        [0.0; 3],
        [0.0; 3],
        crate::bullet_collision::MASK_SHOT,
    );
    diag::info!(
        Sim,
        "bo2zm t6 build test: table check: trigger at ({:.0} {:.0} {:.0}) box {:?} angles {:?}; touching {touching}; eye ({:.0} {:.0} {:.0}) look dot {dot:.2}; trace to eye fraction {:.2} startsolid {}",
        e.origin[0],
        e.origin[1],
        e.origin[2],
        e.box_dims,
        e.angles,
        eye[0],
        eye[1],
        eye[2],
        t.fraction,
        t.startsolid
    );
}

/// No trigger built for the table: the unitrigger manager only offers stubs
/// whose zone (`in_zone`) is in `level.active_zone_names`, plus the dynamic
/// ones. Log the stub's zone, whether it is registered, and the active zones.
fn log_stub_zone(world: &mut World, name: &str) {
    with_vm(world, |vm, _| {
        let (eq, iz, reg, azn) = (
            vm.intern("equipname"),
            vm.intern("in_zone"),
            vm.intern("registered"),
            vm.intern("active_zone_names"),
        );
        let stub = buildable_stubs(vm)
            .into_iter()
            .find(|&s| vm.to_text(&vm.raw_field(s, eq)) == name);
        let (zone, registered) = stub
            .map(|s| {
                (
                    vm.to_text(&vm.raw_field(s, iz)),
                    vm.to_text(&vm.raw_field(s, reg)),
                )
            })
            .unwrap_or_default();
        let active: Vec<String> = match vm.raw_field(vm.level, azn) {
            Value::Array(a) => {
                let vals: Vec<Value> = a.read().values_in_order().cloned().collect();
                vals.iter().map(|v| vm.to_text(v)).collect()
            }
            _ => Vec::new(),
        };
        diag::info!(
            Sim,
            "bo2zm t6 build test: table check: no player trigger; stub in_zone {zone:?} registered {registered:?}; active zones {active:?}"
        );
    });
}

/// The part's unitrigger stub as the manager sees it: zone, place, size,
/// whether this player has a trigger on it, and where the player stands.
fn log_part_trigger(world: &mut World, client: ClientId, part: ObjRef) {
    let player = frame(world).player(client).map(|ps| ps.origin);
    with_vm(world, |vm, world| {
        let f = |vm: &mut Vm<World>, n: &str| vm.intern(n);
        let ut = f(vm, "unitrigger");
        let Some(stub) = vm.raw_field(part, ut).as_obj() else {
            diag::info!(Sim, "bo2zm t6 build test: part has no unitrigger");
            return;
        };
        let fields = [
            "in_zone",
            "registered",
            "radius",
            "script_height",
            "require_look_at",
        ];
        let vals: Vec<String> = fields
            .iter()
            .map(|n| {
                let k = f(vm, n);
                format!("{n} {}", vm.to_text(&vm.raw_field(stub, k)))
            })
            .collect();
        let o = origin_or_field(vm, world, stub);
        let (ot, pt) = (f(vm, "trigger"), f(vm, "playertrigger"));
        let has = !matches!(vm.raw_field(stub, ot), Value::Undefined)
            || matches!(vm.raw_field(stub, pt), Value::Array(_));
        diag::info!(
            Sim,
            "bo2zm t6 build test: part trigger at ({:.0} {:.0} {:.0}) {}; trigger built {has}; player at {player:?}",
            o[0],
            o[1],
            o[2],
            vals.join(", ")
        );
    });
}

fn ground(world: &mut World, at: [f32; 3]) -> [f32; 3] {
    // Inside Tranzit's bus (its hatch is built from in there): on its floor
    // (the trace below sees only the static world, not the bus).
    if let Some(feet) = super::riders::bus_floor_under(world, at) {
        return feet;
    }
    let stand = [at[0], at[1], at[2] + 20.0];
    let down = [stand[0], stand[1], stand[2] - 300.0];
    let t = frame(world).trace_static_world(
        stand,
        down,
        crate::bullet_collision::PLAYER_MINS,
        crate::bullet_collision::PLAYER_MAXS,
        crate::bullet_collision::MASK_PLAYER_SOLID,
    );
    std::array::from_fn(|i| stand[i] + (down[i] - stand[i]) * t.fraction)
}

/// Where to stand for a part at `at`, as walking up to it would: BO2 offers a
/// part's unitrigger only when its origin (the model plus 12 up) is within 42
/// of his origin plus 35 (_zm_unitrigger get_closest_unitriggers), so on a
/// high shelf the floor right under it can be too low. Tries under it, then
/// rings around it; logs every floor height it found.
fn stand_by_part(world: &mut World, at: [f32; 3]) -> [f32; 3] {
    let under = ground(world, at);
    let want = at[2] + 12.0;
    let fits = |feet: [f32; 3]| (want - (feet[2] + 35.0)).abs() <= 42.0;
    HOP.store(false, std::sync::atomic::Ordering::Relaxed);
    if fits(under) {
        return under;
    }
    let mut seen = vec![format!("under {:.1}", under[2])];
    let mut best: Option<[f32; 3]> = None;
    // Nothing fits: he hops from the highest floor of the nearest ring
    // (right under it can be inside what the part rests on: ee5 stuck).
    let mut hop: Option<(f32, [f32; 3])> = None;
    'rings: for r in [24.0f32, 40.0, 56.0] {
        for k in 0..8 {
            let a = (k as f32) * std::f32::consts::FRAC_PI_4;
            let stand = [at[0] + r * a.cos(), at[1] + r * a.sin(), at[2] + 20.0];
            let down = [stand[0], stand[1], stand[2] - 300.0];
            let t = frame(world).trace_static_world(
                stand,
                down,
                crate::bullet_collision::PLAYER_MINS,
                crate::bullet_collision::PLAYER_MAXS,
                crate::bullet_collision::MASK_PLAYER_SOLID,
            );
            if t.startsolid != 0 || t.allsolid != 0 || t.fraction >= 1.0 {
                continue;
            }
            let feet: [f32; 3] = std::array::from_fn(|i| stand[i] + (down[i] - stand[i]) * t.fraction);
            seen.push(format!("{r:.0}/{}:{:.1}", k * 45, feet[2]));
            if hop.is_none_or(|(hr, h)| hr == r && feet[2] > h[2]) {
                hop = Some((r, feet));
            }
            if fits(feet) {
                best = Some(feet);
                break 'rings;
            }
        }
    }
    diag::info!(
        Sim,
        "bo2zm t6 build test: part at z {:.1} needs feet >= {:.1}; floors {}; standing {}",
        at[2],
        want - 77.0,
        seen.join(" "),
        match (best, hop) {
            (Some(b), _) => format!("at ({:.0} {:.0} {:.1})", b[0], b[1], b[2]),
            (None, Some((_, h))) => format!("at ({:.0} {:.0} {:.1}) (no floor fits: he hops)", h[0], h[1], h[2]),
            (None, None) => "under it (no floor fits: he hops)".to_owned(),
        }
    );
    HOP.store(best.is_none(), std::sync::atomic::Ordering::Relaxed);
    best.or(hop.map(|h| h.1)).unwrap_or(under)
}

fn look_at(eye: [f32; 3], to: [f32; 3]) -> [f32; 3] {
    let d = [to[0] - eye[0], to[1] - eye[1], to[2] - eye[2]];
    let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
    [
        -d[2].atan2(flat.max(1.0)).to_degrees(),
        d[1].atan2(d[0]).to_degrees(),
        0.0,
    ]
}

/// The player in the table's use box (on it, in front or behind, wherever
/// he first fits), looking at its middle. The box (`trigger_box_use`) is
/// 100 long along the table's forward but only 32 deep (its right), so he
/// stands within 31 units of it that way. Buried's chalk walls are 16 deep
/// (zm_buried_buildables sets script_length 16), so within 23: the closest
/// spot that fits comes first.
/// Buried's `sloth` stub is Leroy himself (candy and booze are given to
/// him): both must face each other, so stand 48 in front of him looking at
/// his chest. Any other stub is a table.
fn stand_at(world: &mut World, client: ClientId, name: &str, origin: [f32; 3], angles: [f32; 3]) {
    if name != "sloth" {
        return stand_at_table(world, client, origin, angles);
    }
    let leroy = with_vm(world, |vm, world| {
        let f = vm.intern("_sloth_ai");
        let v = vm.raw_field(vm.level, f);
        let n = super::entnum(vm, &v)?;
        world
            .resource::<Zm>()
            .ents
            .get(&n)
            .map(|e| (e.origin, e.angles))
    })
    .flatten();
    let Some((at, ang)) = leroy else {
        diag::warn!(Sim, "bo2zm t6 build test: no Leroy (level._sloth_ai)");
        return;
    };
    let (fwd, _, _) = gsc_t6::math::angle_vectors([0.0, ang[1], 0.0]);
    let feet = ground(
        world,
        [at[0] + fwd[0] * 48.0, at[1] + fwd[1] * 48.0, at[2] + 16.0],
    );
    super::teleport_player(world, client, feet);
    super::set_player_view(
        world,
        client,
        look_at(
            [feet[0], feet[1], feet[2] + 60.0],
            [at[0], at[1], at[2] + 50.0],
        ),
    );
    diag::info!(
        Sim,
        "bo2zm t6 build test: Leroy at ({:.0} {:.0} {:.0}) yaw {:.0}; player at ({:.0} {:.0} {:.0})",
        at[0],
        at[1],
        at[2],
        ang[1],
        feet[0],
        feet[1],
        feet[2]
    );
}

fn stand_at_table(world: &mut World, client: ClientId, origin: [f32; 3], angles: [f32; 3]) {
    let (_, rt, _) = gsc_t6::math::angle_vectors([0.0, angles[1], 0.0]);
    let mut offsets = vec![[0.0; 3]];
    for d in [16.0, 20.0, 22.0, 26.0] {
        offsets.push(rt.map(|v| -v * d));
        offsets.push(rt.map(|v| v * d));
    }
    let mut feet = ground(world, origin);
    for o in offsets {
        let p = [origin[0] + o[0], origin[1] + o[1], origin[2] + o[2]];
        let t = frame(world).trace_static_world(
            p,
            p,
            crate::bullet_collision::PLAYER_MINS,
            crate::bullet_collision::PLAYER_MAXS,
            crate::bullet_collision::MASK_PLAYER_SOLID,
        );
        if t.startsolid == 0 && t.allsolid == 0 {
            feet = ground(world, p);
            break;
        }
    }
    super::teleport_player(world, client, feet);
    super::set_player_view(
        world,
        client,
        look_at([feet[0], feet[1], feet[2] + 60.0], origin),
    );
    diag::info!(
        Sim,
        "bo2zm t6 build test: table at ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
        origin[0],
        origin[1],
        origin[2],
        feet[0],
        feet[1],
        feet[2]
    );
}

fn press(world: &mut World, client: ClientId) {
    press_button(world, client, playerstate_iw4::buttons::USE);
}

fn press_button(world: &mut World, client: ClientId, button: u32) {
    let mut req = world.resource_mut::<crate::step::StepRequest>();
    for (id, cmd) in &mut req.input.cmds {
        if *id == client {
            cmd.buttons |= button;
        }
    }
}

/// The build test's last part sits where no floor fits (see `stand_by_part`).
static HOP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

type Placement = (String, i64, Option<[f32; 4]>, ClientId);

/// IW4L_T6_PLACE's entries: weapon, when (ms), where, who.
fn placements() -> &'static [Placement] {
    static WANT: OnceLock<Vec<Placement>> = OnceLock::new();
    WANT.get_or_init(|| {
        let Ok(v) = std::env::var("IW4L_T6_PLACE") else {
            return Vec::new();
        };
        v.split(';')
            .filter_map(|v| {
                let (client, v) = who(v);
                let mut it = v.split('@');
                let weapon = it.next()?.trim().to_owned();
                let at = it
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .map_or(60_000, |s| (s * 1000.0) as i64);
                let pose = it.next().and_then(|p| {
                    let n: Vec<f32> = p.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                    (n.len() >= 3).then(|| [n[0], n[1], n[2], n.get(3).copied().unwrap_or(0.0)])
                });
                (!weapon.is_empty()).then_some((weapon, at, pose, client))
            })
            .collect()
    })
}

/// IW4L_T6_PLACE_HOLD in ms (default half a second).
fn place_hold() -> i64 {
    static HOLD: OnceLock<i64> = OnceLock::new();
    *HOLD.get_or_init(|| {
        std::env::var("IW4L_T6_PLACE_HOLD")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .map_or(500, |s| (s * 1000.0) as i64)
    })
}

/// True while an IW4L_T6_PLACE entry drives him (from its start until a
/// second and a half after he lets go of fire): autoplay lets go meanwhile,
/// or its gun swap and shots set the equipment down somewhere else. Only
/// player 1's entries count: autoplay and the build test drive him alone.
/// An IW4L_T6_EMP_AVO throw holds him the same way.
pub(crate) fn placing(now: i64) -> bool {
    let hold = place_hold();
    placements()
        .iter()
        .any(|p| p.3 == ClientId(0) && (p.1..p.1 + 3000 + hold + 1500).contains(&now))
        || emp_avo_busy(now)
}

/// When each of player 1's IW4L_T6_PLACE entries starts (ms).
fn place_times() -> Vec<i64> {
    placements().iter().filter(|p| p.3 == ClientId(0)).map(|p| p.1).collect()
}

/// IW4L_T6_PLACE="<equipment weapon>@<seconds>[@<x> <y> <z> <yaw>]" (e.g.
/// `equip_turbine_zm@140@-7180 5000 -55 180`, Tranzit's turbine by the
/// depot's electric door): a test aid for placeable equipment. At that time
/// he is put there (when given) and raises the weapon (he must already hold
/// it: IW4L_T6_BUILD builds it first); 3 s later the weapon and the turret
/// he carries are logged and he holds fire for half a second (BO2 sets it
/// down where `canplayerplaceturret` says). 5 s and 40 s after that his
/// placed equipment (`buildable<name>`, e.g. buildableturbine) and the
/// level flag IW4L_T6_PLACE_FLAG (default OnPriDoorYar) are logged.
/// IW4L_T6_PLACE_HOLD=<seconds> holds fire longer. A `;` list places one
/// after another, each built just before it (TranZit's turret needs a
/// turbine beside it: `equip_turbine_zm@150@...;equip_turret_zm@280@...`
/// with IW4L_T6_BUILD=`turbine@40,turret@156`).
pub(crate) fn place_test(world: &mut World, now: i64) {
    for (weapon, at, pose, client) in placements() {
        place_one(world, now, *client, weapon, *at, pose.as_ref());
    }
}

fn place_one(world: &mut World, now: i64, client: ClientId, weapon: &str, at: i64, pose: Option<&[f32; 4]>) {
    let tick = i64::from(crate::MATCH_TICK_MS);
    let due = |t: i64| (at + t..at + t + tick).contains(&now);
    if due(0) {
        if let Some(p) = pose {
            let feet = ground(world, [p[0], p[1], p[2]]);
            super::teleport_player(world, client, feet);
            super::set_player_view(world, client, [0.0, p[3], 0.0]);
        }
        match super::weapon(world, weapon) {
            Ok(w) => crate::script_player::switch_to_weapon(&mut frame(world), client, w),
            Err(e) => diag::warn!(Sim, "bo2zm t6 place test: {weapon}: {e}"),
        }
    }
    if due(3000) {
        let held = frame(world).player(client).map_or(0, |ps| ps.weapon as u32);
        let facts = frame(world)
            .combat_facts_for(held)
            .map(|f| (f.weap_type, f.inventory_type));
        let name = super::weapon_text(world, held);
        let carried: Vec<String> = world
            .resource::<Zm>()
            .ents
            .values()
            .filter(|e| e.classname == "misc_turret")
            .map(|e| format!("{} at ({:.0} {:.0} {:.0}) link {:?}", e.model, e.origin[0], e.origin[1], e.origin[2], e.link.map(|l| l.0)))
            .collect();
        diag::info!(
            Sim,
            "bo2zm t6 place test: holding {name} (type, inventory) {facts:?}; turrets {carried:?}"
        );
    }
    // IW4L_T6_PLACE_HOLD=<seconds>: hold fire that long instead (the Jet
    // Gun spins and heats while it is held).
    let hold = place_hold();
    if now >= at + 3000 && now < at + 3000 + hold {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= playerstate_iw4::buttons::ATTACK;
            }
        }
    }
    if due(8000) || due(43_000) {
        let flag = std::env::var("IW4L_T6_PLACE_FLAG").unwrap_or_else(|_| "OnPriDoorYar".to_owned());
        let on = level_flag(world, &flag);
        let field = if weapon.starts_with("riotshield") {
            "riotshieldentity".to_owned()
        } else {
            format!("buildable{}", weapon.trim_start_matches("equip_").trim_end_matches("_zm"))
        };
        let placed = with_vm(world, |vm, world| {
            let p = world.resource::<Zm>().players.get(&client.0).map(|p| p.obj)?;
            let f = vm.intern(&field);
            let v = vm.raw_field(p, f);
            let n = super::entnum(vm, &v)?;
            world.resource::<Zm>().ents.get(&n).map(|e| (e.model.clone(), e.origin))
        })
        .flatten();
        let held = frame(world).player(client).map_or(0, |ps| ps.weapon as u32);
        let name = super::weapon_text(world, held);
        diag::info!(
            Sim,
            "bo2zm t6 place test +{}s: {field} {placed:?}; flag {flag} {on}; holding {name}",
            (now - at) / 1000
        );
    }
}
