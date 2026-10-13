//! Origins (zm_tomb) portal test aid: the gramophone route to the crypt and
//! the Crazy Place.
//!
//! IW4L_T6_PORTAL=<n>[,<n>...][@<seconds>] (from 30 s unless given; n is
//! the gramophone spot's `script_int`: 0 the crypt door under the
//! excavation site with the master record, 1 fire, 2 wind, 3 lightning,
//! 4 ice portal with that record). For each n in turn the player picks up
//! the gramophone and that record where they lie (stands beside each and
//! presses use, 8 s a part), then stands at the spot (`chamber_entrance_position`
//! or `stargate_gramophone_pos`) and presses use every 1.5 s until the game
//! places the gramophone (`gramophone_placed`). The crypt: once the door is
//! down (`chamber_entrance_opened`) he takes the gramophone back (use every
//! 1.5 s until `gramophone_placed` clears) and goes on to the next n. A
//! portal: once it has opened (`enable_teleporter_<n>` set and
//! `teleporter_building_<n>` clear) he is put 140 units from it
//! (`trigger_teleport_pad`) and walks in; on the other side he faces the
//! staff's gem stand (`crystal_plinth<n>`) for 12 s, which is what makes the
//! gem come up in the real game. Logged every second: the step,
//! where he is, those flags, the zone flag the spot sets
//! (`activate_zone_crypt` or `activate_zone_chamber`) and his `teleporting`.

use std::sync::{Mutex, OnceLock};

use bevy_ecs::prelude::World;
use gsc_t6::{Key, ObjRef, Value, Vm};

use super::tomb_test::{look_and_use, place_near};
use super::{frame, with_vm};
use crate::world::ClientId;

fn want() -> Option<&'static (Vec<i32>, i64)> {
    static WANT: OnceLock<Option<(Vec<i32>, i64)>> = OnceLock::new();
    WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_PORTAL").ok()?;
        let (n, at) = match v.split_once('@') {
            Some((n, s)) => (n.to_owned(), s.parse::<f32>().map_or(30_000, |s| (s * 1000.0) as i64)),
            None => (v, 30_000),
        };
        let list: Vec<i32> = n.split(',').filter_map(|s| s.trim().parse().ok()).collect();
        (!list.is_empty()).then_some((list, at))
    })
    .as_ref()
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Step {
    Pick,
    Place,
    Open,
    Retake,
    Walk,
    Through,
}

struct State {
    /// Which of the wanted spots.
    idx: usize,
    step: Step,
    /// When the step began, and (Pick) the part being picked.
    since: i64,
    part: Option<[f32; 3]>,
    /// Where he walks from, and the portal.
    walk: Option<([f32; 3], [f32; 3])>,
    /// The spots still to walk in from (and how far a player gets from each
    /// toward the trigger; NaN: where he stood), and when this one began.
    tries: Vec<([f32; 3], f32)>,
    try_at: i64,
    /// The trigger's foot (30 under the portal's middle) he walks to.
    to: [f32; 3],
    /// When he had faced the gem stand long enough (0 until then).
    done_at: i64,
}

static STATE: Mutex<State> = Mutex::new(State {
    idx: 0,
    step: Step::Pick,
    since: 0,
    part: None,
    walk: None,
    tries: Vec::new(),
    try_at: 0,
    to: [0.0; 3],
    done_at: 0,
});

/// When this aid was done (he faced the gem stand on the other side), so an
/// aid that needs the Crazy Place starts after it; at once without it.
pub(crate) fn done_at() -> Option<i64> {
    if want().is_none() {
        return Some(0);
    }
    let st = STATE.lock().unwrap();
    (st.done_at != 0).then_some(st.done_at)
}

fn level_flag(vm: &mut Vm<World>, name: &str) -> bool {
    let f = vm.intern("flag");
    match vm.raw_field(vm.level, f) {
        Value::Array(a) => a
            .get(&Key::Str(vm.intern(name)))
            .is_some_and(|v| gsc_t6::truthy(&v)),
        _ => false,
    }
}

/// The struct with this targetname and script_int: origin and angles.
fn find_struct(vm: &mut Vm<World>, targetname: &str, n: i32) -> Option<([f32; 3], [f32; 3])> {
    let f = vm.intern("struct");
    let Value::Array(arr) = vm.raw_field(vm.level, f) else {
        return None;
    };
    let (tn, si, o, a) = (
        vm.intern("targetname"),
        vm.intern("script_int"),
        vm.intern("origin"),
        vm.intern("angles"),
    );
    let list: Vec<ObjRef> = arr.snapshot().values_in_order().filter_map(Value::as_obj).collect();
    list.into_iter().find_map(|s| {
        let name = vm.raw_field(s, tn);
        if vm.to_text(&name) != targetname || !matches!(vm.raw_field(s, si), Value::Int(i) if i == n) {
            return None;
        }
        Some((vm.raw_field(s, o).as_vec3()?, vm.raw_field(s, a).as_vec3().unwrap_or_default()))
    })
}

fn record(n: i32) -> &'static str {
    match n {
        0 => "p6_zm_tm_record_master",
        1 => "p6_zm_tm_record_fire",
        2 => "p6_zm_tm_record_wind",
        3 => "p6_zm_tm_record_lightning",
        _ => "p6_zm_tm_record_ice",
    }
}

/// The gramophone spot's targetname and the zone flag it sets.
fn spot(n: i32) -> (&'static str, &'static str) {
    if n == 0 {
        ("chamber_entrance_position", "activate_zone_crypt")
    } else {
        ("stargate_gramophone_pos", "activate_zone_chamber")
    }
}

pub(crate) fn portal_test(world: &mut World, now: i64) {
    let Some((list, at)) = want() else {
        return;
    };
    if now < *at {
        return;
    }
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let Some(ps) = frame(world).player(client).copied() else {
        return;
    };
    let mut st = STATE.lock().unwrap();
    let Some(&n) = list.get(st.idx) else {
        return;
    };
    if st.since == 0 {
        st.since = now;
    }
    let t = now - st.since;
    let (spot_name, zone) = spot(n);
    let (placed, enabled, building, opened, zone_on, teleporting) = with_vm(world, |vm, world| {
        let tp = world
            .resource::<super::Zm>()
            .players
            .get(&0)
            .map(|p| p.obj)
            .map(|p| {
                let f = vm.intern("teleporting");
                gsc_t6::truthy(&vm.get_field(world, p, f))
            })
            .unwrap_or(false);
        (
            level_flag(vm, "gramophone_placed"),
            level_flag(vm, &format!("enable_teleporter_{n}")),
            level_flag(vm, &format!("teleporter_building_{n}")),
            level_flag(vm, "chamber_entrance_opened"),
            level_flag(vm, zone),
            tp,
        )
    })
    .unwrap_or_default();
    let mut next = None;
    match st.step {
        Step::Pick => {
            if t % 8_000 < tick {
                let parts = with_vm(world, |vm, world| super::tomb_craft::survey(vm, world, "gramophone").0)
                    .unwrap_or_default();
                let me = ps.origin;
                let lying = parts
                    .iter()
                    .filter(|p| !p.crafted && (p.name == "p6_zm_tm_gramophone" || p.name == record(n)))
                    .filter_map(|p| p.at.map(|a| (p.name.clone(), a)))
                    .min_by(|a, b| d2(a.1, me).total_cmp(&d2(b.1, me)));
                match lying {
                    Some((name, c)) => {
                        let feet = place_near(world, c, 0.0, 24.0).unwrap_or(c);
                        super::teleport_player(world, client, feet);
                        st.part = Some(c);
                        diag::info!(Sim, "bo2zm t6 portal test {}s: pick {name} at ({:.0} {:.0} {:.0})", now / 1000, c[0], c[1], c[2]);
                    }
                    None => {
                        st.part = None;
                        next = Some(Step::Place);
                    }
                }
            }
            let tt = t % 8_000;
            if let Some(c) = st.part {
                if (0..4).any(|k| (1_000 + 1_500 * k..1_300 + 1_500 * k).contains(&tt)) {
                    look_and_use(world, client, [c[0], c[1], c[2] + 8.0]);
                }
            }
        }
        Step::Place | Step::Retake => {
            let found = with_vm(world, |vm, _| find_struct(vm, spot_name, n)).flatten();
            let Some((o, _)) = found else {
                if t < tick {
                    diag::warn!(Sim, "bo2zm t6 portal test: no {spot_name} {n}");
                }
                return;
            };
            if t < tick {
                let feet = place_near(world, o, 0.0, 40.0).unwrap_or(o);
                super::teleport_player(world, client, feet);
                diag::info!(Sim, "bo2zm t6 portal test {}s: {:?} at {spot_name} {n} ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})", now / 1000, st.step, o[0], o[1], o[2], feet[0], feet[1], feet[2]);
            }
            if st.step == Step::Place && placed {
                next = Some(Step::Open);
            } else if st.step == Step::Retake && !placed {
                st.idx += 1;
                next = Some(Step::Pick);
            } else if (1_000..1_300).contains(&(t % 1_500)) {
                look_and_use(world, client, o);
            }
        }
        Step::Open => {
            if n == 0 {
                if opened && t > 1_000 {
                    next = Some(Step::Retake);
                }
            } else if enabled && !building && t > 1_000 {
                next = Some(Step::Walk);
            }
        }
        Step::Walk => {
            if t < tick {
                let pad = with_vm(world, |vm, _| find_struct(vm, "trigger_teleport_pad", n)).flatten();
                let Some((p, a)) = pad else {
                    diag::warn!(Sim, "bo2zm t6 portal test: no trigger_teleport_pad {n}");
                    return;
                };
                // The trigger sits 30 under the portal's middle; walk in from
                // the first of 8 headings (from its facing) with a floor and a
                // clear way to it. The floor can be well under the trigger
                // (the wind portal's is 84 under it).
                let low = [p[0], p[1], p[2] - 30.0];
                let (mut clear, mut rest) = (Vec::new(), Vec::new());
                for j in 0..24 {
                    let (k, dz) = (j % 8, [0.0, 50.0, 100.0][j / 8]);
                    let Some(f) = place_near(world, [low[0], low[1], low[2] - dz], a[1] + 45.0 * k as f32, 140.0) else {
                        continue;
                    };
                    let tr = frame(world).trace_static_world(
                        [f[0], f[1], f[2] + 4.0],
                        [low[0], low[1], f[2] + 4.0],
                        crate::bullet_collision::PLAYER_MINS,
                        crate::bullet_collision::PLAYER_MAXS,
                        crate::bullet_collision::MASK_PLAYER_SOLID,
                    );
                    if tr.fraction > 0.6 { &mut clear } else { &mut rest }.push((f, tr.fraction));
                }
                rest.sort_by(|a, b| b.1.total_cmp(&a.1));
                // None clear (staffair2/4, sair60: the wind portal's): where
                // he stands, by the record he just placed in front of it,
                // then the rest, the farthest-reaching first (selec60: at the
                // lightning portal he stood still there for 28 s).
                let d = ((ps.origin[0] - low[0]).powi(2) + (ps.origin[1] - low[1]).powi(2)).sqrt();
                let here = (d < 400.0).then_some((ps.origin, f32::NAN));
                st.tries = clear.into_iter().chain(here).chain(rest).collect();
                st.walk = None;
                st.try_at = 0;
                diag::info!(Sim, "bo2zm t6 portal test {}s: portal {n} open at ({:.0} {:.0} {:.0}); {} spots to walk in from", now / 1000, p[0], p[1], p[2], st.tries.len());
                if st.tries.is_empty() {
                    diag::warn!(Sim, "bo2zm t6 portal test: no way into portal {n} at ({:.0} {:.0} {:.0})", p[0], p[1], p[2]);
                }
                st.to = low;
            }
            // The next spot once he has stood still 1.5 s on this one, or
            // walked 6 s without going through.
            let moved = st.walk.map(|(f, _)| d2(ps.origin, f).sqrt());
            let stuck = moved.is_none_or(|m| {
                let tt = now - st.try_at;
                (tt >= 1_500 && m < 8.0) || tt >= 6_000
            });
            if teleporting {
                next = Some(Step::Through);
            } else if stuck && !st.tries.is_empty() {
                let (f, reach) = st.tries.remove(0);
                super::teleport_player(world, client, f);
                if let Some(m) = moved {
                    diag::info!(Sim, "bo2zm t6 portal test {}s: {m:.0} units walked, not through; next spot", now / 1000);
                }
                st.walk = Some((f, st.to));
                st.try_at = now;
                let how = if reach.is_nan() { "where he stands".to_owned() } else { format!("clear {reach:.2} of the way") };
                diag::info!(Sim, "bo2zm t6 portal test {}s: walking in from ({:.0} {:.0} {:.0}), {how}; {} spots left", now / 1000, f[0], f[1], f[2], st.tries.len());
            } else if stuck && st.walk.is_some() {
                diag::warn!(Sim, "bo2zm t6 portal test {}s: walked in from every spot, never through portal {n}", now / 1000);
                st.walk = None;
            } else if let Some((_, to)) = st.walk {
                let d = [to[0] - ps.origin[0], to[1] - ps.origin[1]];
                super::set_player_view(world, client, [0.0, d[1].atan2(d[0]).to_degrees(), 0.0]);
                let mut req = world.resource_mut::<crate::step::StepRequest>();
                for (id, cmd) in &mut req.input.cmds {
                    if *id == client {
                        cmd.forwardmove = 127;
                    }
                }
            }
        }
        Step::Through => {
            // The staff's gem stays hidden until someone who came through
            // this portal looks at its stand (`crystal_plinth<n>`) for a
            // second (`staff_crystal_wait_for_teleport`): face it for 12 s.
            if t >= 15_000 && st.done_at == 0 {
                st.done_at = now;
            }
            if t < 12_000 {
                let plinth = super::autoplay::ent_by(world, "targetname", &format!("crystal_plinth{n}"));
                if let Some((_, o, _)) = plinth {
                    let eye = [ps.origin[0], ps.origin[1], ps.origin[2] + ps.view_height_current];
                    super::set_player_view(world, client, super::autoplay::look(eye, o));
                    if t < tick {
                        diag::info!(Sim, "bo2zm t6 portal test {}s: facing crystal_plinth{n} at ({:.0} {:.0} {:.0})", now / 1000, o[0], o[1], o[2]);
                    }
                }
            }
        }
    }
    if let Some(s) = next {
        diag::info!(Sim, "bo2zm t6 portal test {}s: {n} {:?} -> {s:?}", now / 1000, st.step);
        st.step = s;
        // This aid runs every tick: the next call is the new step's first (t 0).
        st.since = now + tick;
        st.part = None;
    }
    if now % 1000 < tick && (st.step != Step::Through || t < 15_000) {
        diag::info!(
            Sim,
            "bo2zm t6 portal test {}s: {n} {:?}; player ({:.0} {:.0} {:.0}); gramophone_placed {} chamber_entrance_opened {} enable_teleporter_{n} {} teleporter_building_{n} {} {zone} {} teleporting {}",
            now / 1000,
            st.step,
            ps.origin[0],
            ps.origin[1],
            ps.origin[2],
            u8::from(placed),
            u8::from(opened),
            u8::from(enabled),
            u8::from(building),
            u8::from(zone_on),
            u8::from(teleporting)
        );
    }
}

fn d2(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}
