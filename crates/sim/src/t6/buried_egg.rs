//! Buried's easter egg test aids.
//!
//! IW4L_T6_LANTERN=<start>[,<blast>] (seconds): the ghost lantern step
//! (`zm_buried_sq_gl`) starts at <start> as the sidequest would start it
//! (its `init_stage`, then `stage_logic`): the lantern flies from street to
//! street. Every second: where it is and how it flies (the flight log adds
//! a line at each goal). At <blast> a grenade blast goes off where the
//! lantern is (the same damage a frag's explosion does), so it crashes; 6 s
//! later the player stands where it landed and holds use for half a second
//! to pick it up; at +9 s the level flag `sq_gl_lantern_aquired` and the
//! parts he carries are logged. <start> 0 leaves the step to the
//! sidequest (with IW4L_T6_SQ_SKIP): only the blast and the pickup run.
//!
//! IW4L_T6_SQ_SKIP=<stage>@<seconds>: from <seconds> the sidequest jumps to
//! <stage> (bt mta gl ftl ll ts ctw tpo ip ows) as Buried's own devgui
//! command `sq_start_stage_<stage>` does (`skip_to_sq_stage`): each earlier
//! stage is ended and marked completed (ctw with `sq_wisp_success`), and the
//! main loop starts the next. Unlike the devgui, it ends a stage only once
//! the loop has started it (`_last_stage_started`), one at a time.
//!
//! IW4L_T6_SIGNS=<seconds>[,<knife>]: the tunnel signs step (`ts`). At
//! <seconds> he is given the knife (default `bowie_knife_zm`) the way the
//! wall buy gives it (`give_melee_weapon_by_name`); at +6 s every tunnel sign
//! is logged (which side's code it carries, lit or not). Then the three
//! signs of the built tower's side (Maxis's when `sq_is_max_tower_built`,
//! else Richtofen's), one every 4 s: he hangs in front of the sign, looking
//! at it, knifes it once at +1 s, and at +3 s the sign is logged again.
//!
//! IW4L_T6_SQ_SIDE=maxis|richtofen: with IW4L_T6_SQ_SKIP, the side is set
//! first the way Buried's devgui does (`set_side_maxis` /
//! `set_side_richtofen`: the built tower's flag).
//!
//! IW4L_T6_FOLLOW=<seconds>: the wisp step (`ctw`). From <seconds> he keeps
//! up with the wisp: Maxis's (`level.vh_wisp`, he walks under it) or
//! Richtofen's (`sq_wisp`, he is given Vulture Aid, the only way to see it,
//! and stays within 20 of it). Every second: where it is, its energy
//! (`n_sq_energy`) and the zombies within 256; then whether
//! `sq_wisp_success` or `sq_wisp_failed` came.
//!
//! IW4L_T6_OWS=<seconds>: the fountain targets step (`ows`). At <seconds> he
//! stands by the fountain looking at it and holds use for half a second (the
//! coin toss). Then he goes to each target that comes up, hangs 120 out
//! from it, looks at its middle and fires his gun until it is down (1 s at
//! most). Every second: targets up, targets shot, and the step's flags
//! (`sq_ows_start`, `sq_ows_target_missed`, `sq_ows_success`).
//!
//! IW4L_T6_BULBS=<seconds>: Maxis's light board (`ip`, Maxis's side). At
//! <seconds> he throws the board's switch (`level.t_start`); then each bulb
//! that lights (`level.str_sq_bp_active_light`) he goes to its button (the
//! `sq_bp_button` with that tag) and presses it, as a player does reading
//! the board. Every second: the lit bulb and `sq_ip_puzzle_complete`.
//!
//! IW4L_T6_LEVERS=<seconds>: Richtofen's maze levers (`ip`, Richtofen's
//! side). From <seconds> he flips the levers (`sq_ml_lever`) one every 3 s
//! in the puzzle's order (`n_lever_order`: BO2 shuffles it, a player finds
//! it by the sparks). Every second: each lever up or down (and when it was
//! flipped) and `sq_ip_puzzle_complete`.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

use bevy_ecs::prelude::World;
use gsc_t6::{ObjRef, Value};
use playerstate_iw4::buttons;

use super::{Zm, entnum, frame, with_vm};
use crate::world::ClientId;

fn want() -> Option<(i64, Option<i64>)> {
    static WANT: OnceLock<Option<(i64, Option<i64>)>> = OnceLock::new();
    *WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_LANTERN").ok()?;
        let ms = |s: &str| s.trim().parse::<f32>().ok().map(|s| (s * 1000.0) as i64);
        let mut it = v.split(',');
        let start = it.next().and_then(ms).unwrap_or(20000);
        Some((start, it.next().and_then(ms)))
    })
}

fn skip() -> Option<&'static (String, i64)> {
    static SKIP: OnceLock<Option<(String, i64)>> = OnceLock::new();
    SKIP.get_or_init(|| {
        let v = std::env::var("IW4L_T6_SQ_SKIP").ok()?;
        let (stage, secs) = v.split_once('@')?;
        let ms = (secs.trim().parse::<f32>().ok()? * 1000.0) as i64;
        Some((stage.trim().to_owned(), ms))
    })
    .as_ref()
}

/// Buried's sidequest stages in the order its main loop (`sidequest_logic`)
/// starts them.
const SQ_ORDER: [&str; 10] = ["bt", "mta", "gl", "ftl", "ll", "ts", "ctw", "tpo", "ip", "ows"];

/// The last stage the skip ended (each is ended once).
static SKIP_ENDED: Mutex<Option<String>> = Mutex::new(None);
static SKIP_THERE: AtomicBool = AtomicBool::new(false);

/// `level._last_stage_started`: set by `stage_start` once the stage's
/// `init_stage` is done, just before the main loop waits for its end.
fn last_stage(world: &mut World) -> Option<String> {
    with_vm(world, |vm, _| {
        let f = vm.intern("_last_stage_started");
        match vm.raw_field(vm.level, f) {
            Value::Undefined => None,
            v => Some(vm.to_text(&v)),
        }
    })
    .flatten()
}

fn sq_skip(world: &mut World, now: i64) {
    let Some((stage, at)) = skip() else {
        return;
    };
    let tick = i64::from(crate::MATCH_TICK_MS);
    if (*at..*at + tick).contains(&now) {
        if let Some(side) = side() {
            with_vm(world, |vm, world| {
                let level = Value::Object(vm.level);
                vm.spawn_named(world, "maps/mp/zm_buried_sq", &format!("set_side_{side}"), level, vec![]);
            });
            diag::info!(Sim, "bo2zm t6 sq skip: side set to {side}");
        }
    }
    if now < *at || SKIP_THERE.load(Ordering::Relaxed) {
        return;
    }
    let Some(want) = SQ_ORDER.iter().position(|s| s == stage) else {
        return;
    };
    let Some(cur) = last_stage(world) else {
        return;
    };
    if cur == *stage {
        SKIP_THERE.store(true, Ordering::Relaxed);
        diag::info!(Sim, "bo2zm t6 sq skip: {stage} started at {}s", now / 1000);
        return;
    }
    let mut ended = SKIP_ENDED.lock().unwrap_or_else(|e| e.into_inner());
    let before = SQ_ORDER.iter().position(|s| *s == cur).is_some_and(|i| i < want);
    if !before || ended.as_deref() == Some(cur.as_str()) {
        return;
    }
    // One stage at a time, each only once the main loop has started it
    // (BO2's own `skip_to_sq_stage` ends the next stage a network frame
    // later, before gl's start, which waits a network frame for the
    // lantern, is done: the loop misses `sq_gl_over` and stays at gl).
    with_vm(world, |vm, world| {
        let level = Value::Object(vm.level);
        if cur == "ctw" {
            let f = vm.string("sq_wisp_success");
            vm.spawn_named(world, "common_scripts/utility", "flag_set", level.clone(), vec![f]);
        }
        let (sq, name) = (vm.string("sq"), vm.string(&cur));
        vm.spawn_named(world, "maps/mp/zombies/_zm_sidequests", "stage_completed", level, vec![sq, name]);
    });
    diag::info!(Sim, "bo2zm t6 sq skip: ended {cur} at {}s", now / 1000);
    *ended = Some(cur);
}

/// `level.vh_lantern`: its entity number.
fn lantern(world: &mut World) -> Option<u32> {
    with_vm(world, |vm, _| {
        let k = vm.intern("vh_lantern");
        let v = vm.raw_field(vm.level, k);
        entnum(vm, &v)
    })
    .flatten()
}

fn signs_want() -> Option<&'static (i64, String)> {
    static SIGNS: OnceLock<Option<(i64, String)>> = OnceLock::new();
    SIGNS
        .get_or_init(|| {
            let v = std::env::var("IW4L_T6_SIGNS").ok()?;
            let mut it = v.split(',');
            let ms = (it.next()?.trim().parse::<f32>().ok()? * 1000.0) as i64;
            let knife = it.next().map_or("bowie_knife_zm", str::trim).to_owned();
            Some((ms, knife))
        })
        .as_ref()
}

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

/// A tunnel sign: entity, origin, yaw, Maxis's code, Richtofen's code, lit.
type Sign = (u32, [f32; 3], f32, bool, bool, bool);

/// Every `sq_tunnel_sign` model, by entity number.
fn tunnel_signs(world: &mut World) -> Vec<Sign> {
    let ents: Vec<(u32, ObjRef, [f32; 3], f32)> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter_map(|(n, e)| e.obj.map(|o| (*n, o, e.origin, e.angles[1])))
        .collect();
    with_vm(world, |vm, _| {
        let tn = vm.intern("targetname");
        let [max, ric, lit] = ["is_max_sign", "is_ric_sign", "ts_sign_activated"].map(|k| vm.intern(k));
        let mut out: Vec<Sign> = ents
            .iter()
            .filter(|(_, o, ..)| vm.to_text(&vm.raw_field(*o, tn)) == "sq_tunnel_sign")
            .map(|(n, o, at, yaw)| {
                let is = |k| gsc_t6::truthy(&vm.raw_field(*o, k));
                (*n, *at, *yaw, is(max), is(ric), is(lit))
            })
            .collect();
        out.sort_by_key(|s| s.0);
        out
    })
    .unwrap_or_default()
}

fn signs_test(world: &mut World, now: i64) {
    let Some((base, knife)) = signs_want() else {
        return;
    };
    let tick = i64::from(crate::MATCH_TICK_MS);
    let at = |t: i64| (t..t + tick).contains(&now);
    let client = ClientId(0);
    if at(*base) {
        with_vm(world, |vm, world| {
            if let Some(p) = world.resource::<Zm>().players.get(&client.0).map(|p| p.obj) {
                let g = vm.string(knife);
                let f = "give_melee_weapon_by_name";
                vm.spawn_named(world, "maps/mp/zombies/_zm_melee_weapon", f, Value::Object(p), vec![g]);
            }
        });
        diag::info!(Sim, "bo2zm t6 signs test: given {knife}");
    }
    if now < base + 6000 || now > base + 8000 + 3 * 4000 {
        return;
    }
    let max = level_flag(world, "sq_is_max_tower_built");
    let all = tunnel_signs(world);
    if at(base + 6000) {
        let side = if max { "Maxis" } else { "Richtofen" };
        diag::info!(Sim, "bo2zm t6 signs test: {} signs, {side}'s side", all.len());
        for s in &all {
            diag::info!(
                Sim,
                "bo2zm t6 signs test: sign {} at ({:.0} {:.0} {:.0}) yaw {:.0} maxis {} richtofen {} lit {}",
                s.0,
                s.1[0],
                s.1[1],
                s.1[2],
                s.2,
                s.3,
                s.4,
                s.5
            );
        }
    }
    let chosen: Vec<&Sign> = all.iter().filter(|s| if max { s.3 } else { s.4 }).collect();
    for (k, s) in chosen.iter().take(3).enumerate() {
        let t = base + 8000 + 4000 * k as i64;
        if !(t..t + 3000 + tick).contains(&now) {
            continue;
        }
        // The sign's face looks along its yaw; its middle hangs ~20 below
        // its origin (BO2 lights it there, `ts_sign_activate`).
        let (sin, cos) = s.2.to_radians().sin_cos();
        let aim = [s.1[0], s.1[1], s.1[2] - 20.0];
        let eye = [aim[0] + cos * 30.0, aim[1] + sin * 30.0, aim[2] + 6.0];
        let feet = [eye[0], eye[1], eye[2] - 60.0];
        let pitch = (6.0f32).atan2(30.0).to_degrees();
        if at(t) {
            super::teleport_player(world, client, feet);
            diag::info!(
                Sim,
                "bo2zm t6 signs test: at sign {} from ({:.0} {:.0} {:.0})",
                s.0,
                feet[0],
                feet[1],
                feet[2]
            );
        } else if now < t + 1500 {
            // Hang there: no fall while he knifes.
            let mut f = frame(world);
            f.set_origin(client, feet);
            if let Some(ps) = f.player_mut(client) {
                ps.velocity = [0.0; 3];
            }
        }
        if now < t + 1500 {
            super::set_player_view(world, client, [pitch, s.2 + 180.0, 0.0]);
        }
        if (t + 1000..t + 1300).contains(&now) {
            let mut req = world.resource_mut::<crate::step::StepRequest>();
            for (id, cmd) in &mut req.input.cmds {
                if *id == client {
                    cmd.buttons |= weapon_iw4::BUTTON_MELEE;
                }
            }
        }
        if at(t + 3000) {
            diag::info!(Sim, "bo2zm t6 signs test: sign {} lit {} after the knife", s.0, s.5);
        }
    }
}

pub(crate) fn lantern_test(world: &mut World, now: i64) {
    sq_skip(world, now);
    signs_test(world, now);
    follow_test(world, now);
    ows_test(world, now);
    bulbs_test(world, now);
    levers_test(world, now);
    let Some((start, blast)) = want() else {
        return;
    };
    let tick = i64::from(crate::MATCH_TICK_MS);
    let at = |t: i64| (t..t + tick).contains(&now);
    if start > 0 && at(start) {
        with_vm(world, |vm, world| {
            let level = Value::Object(vm.level);
            let script = "maps/mp/zm_buried_sq_gl";
            let ok = vm
                .spawn_named(world, script, "init_stage", level.clone(), Vec::new())
                .is_some()
                && vm
                    .spawn_named(world, script, "stage_logic", level, Vec::new())
                    .is_some();
            diag::info!(Sim, "bo2zm t6 lantern test: step started {ok}");
        });
    }
    if now > start && now % 1000 < tick {
        if let Some(v) = lantern(world) {
            let o = world.resource::<Zm>().ents.get(&v).map(|e| e.origin);
            let how = super::flight::describe(world, v).unwrap_or_default();
            if let Some(o) = o {
                diag::info!(
                    Sim,
                    "bo2zm t6 lantern test: lantern {v} at ({:.0} {:.0} {:.0}) {how}",
                    o[0],
                    o[1],
                    o[2]
                );
            }
        }
    }
    let Some(blast) = blast else {
        return;
    };
    if at(blast) {
        let Some(o) = lantern(world).and_then(|v| world.resource::<Zm>().ents.get(&v).map(|e| e.origin))
        else {
            diag::warn!(Sim, "bo2zm t6 lantern test: no lantern to blast");
            return;
        };
        diag::info!(
            Sim,
            "bo2zm t6 lantern test: blast at ({:.0} {:.0} {:.0})",
            o[0],
            o[1],
            o[2]
        );
        super::damage_triggers(
            world,
            o,
            o,
            128.0,
            150,
            ClientId(0),
            0,
            super::DamageKind::Explosion,
        );
    }
    let client = ClientId(0);
    if at(blast + 6000) {
        let Some(o) = lantern(world).and_then(|v| world.resource::<Zm>().ents.get(&v).map(|e| e.origin))
        else {
            diag::warn!(Sim, "bo2zm t6 lantern test: no lantern to pick up");
            return;
        };
        let stand = [o[0], o[1], o[2] + 20.0];
        let down = [stand[0], stand[1], stand[2] - 300.0];
        let t = frame(world).trace_static_world(
            stand,
            down,
            crate::bullet_collision::PLAYER_MINS,
            crate::bullet_collision::PLAYER_MAXS,
            crate::bullet_collision::MASK_PLAYER_SOLID,
        );
        let feet: [f32; 3] = std::array::from_fn(|i| stand[i] + (down[i] - stand[i]) * t.fraction);
        super::teleport_player(world, client, feet);
        super::set_player_view(world, client, [60.0, 0.0, 0.0]);
        diag::info!(
            Sim,
            "bo2zm t6 lantern test: lantern down at ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
            o[0],
            o[1],
            o[2],
            feet[0],
            feet[1],
            feet[2]
        );
    }
    if (blast + 6500..blast + 7000).contains(&now) {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= playerstate_iw4::buttons::USE;
            }
        }
    }
    if at(blast + 9000) {
        let got = with_vm(world, |vm, _| {
            let f = vm.intern("flag");
            match vm.raw_field(vm.level, f) {
                Value::Array(a) => a
                    .get(&gsc_t6::Key::Str(vm.intern("sq_gl_lantern_aquired")))
                    .is_some_and(|v| gsc_t6::truthy(&v)),
                _ => false,
            }
        })
        .unwrap_or(false);
        let pieces = super::use_test::held_parts(world);
        diag::info!(
            Sim,
            "bo2zm t6 lantern test: sq_gl_lantern_aquired {got}, carrying {pieces:?}, lantern left {:?}",
            lantern(world)
        );
    }
}

/// IW4L_T6_SQ_SIDE=maxis|richtofen.
fn side() -> Option<&'static str> {
    static SIDE: OnceLock<Option<&'static str>> = OnceLock::new();
    *SIDE.get_or_init(|| match std::env::var("IW4L_T6_SQ_SIDE").ok()?.trim() {
        "maxis" => Some("maxis"),
        "richtofen" => Some("richtofen"),
        _ => None,
    })
}

fn follow_want() -> Option<i64> {
    static FOLLOW: OnceLock<Option<i64>> = OnceLock::new();
    *FOLLOW.get_or_init(|| {
        let v = std::env::var("IW4L_T6_FOLLOW").ok()?;
        Some((v.trim().parse::<f32>().ok()? * 1000.0) as i64)
    })
}

/// Richtofen's wisp model (`sq_wisp`), found once.
static RIC_WISP: AtomicU32 = AtomicU32::new(u32::MAX);
static FOLLOWING: AtomicBool = AtomicBool::new(false);
static LET_GO: AtomicBool = AtomicBool::new(false);

/// The wisp: Maxis's `level.vh_wisp` vehicle, or Richtofen's `sq_wisp`
/// model. Its entity, object and origin.
fn wisp(world: &mut World, max: bool) -> Option<(u32, ObjRef, [f32; 3])> {
    let n = if max {
        with_vm(world, |vm, _| {
            let k = vm.intern("vh_wisp");
            let v = vm.raw_field(vm.level, k);
            entnum(vm, &v)
        })
        .flatten()?
    } else if RIC_WISP.load(Ordering::Relaxed) != u32::MAX {
        RIC_WISP.load(Ordering::Relaxed)
    } else {
        let ents: Vec<(u32, ObjRef)> =
            world.resource::<Zm>().ents.iter().filter_map(|(n, e)| e.obj.map(|o| (*n, o))).collect();
        let n = with_vm(world, |vm, _| {
            let tn = vm.intern("targetname");
            ents.iter().find(|(_, o)| vm.to_text(&vm.raw_field(*o, tn)) == "sq_wisp").map(|(n, _)| *n)
        })
        .flatten()?;
        RIC_WISP.store(n, Ordering::Relaxed);
        n
    };
    let e = world.resource::<Zm>().ents.get(&n)?;
    Some((n, e.obj?, e.origin))
}

fn follow_test(world: &mut World, now: i64) {
    let Some(base) = follow_want() else {
        return;
    };
    if now < base || LET_GO.load(Ordering::Relaxed) {
        return;
    }
    let tick = i64::from(crate::MATCH_TICK_MS);
    let client = ClientId(0);
    let max = level_flag(world, "sq_is_max_tower_built");
    if (base..base + tick).contains(&now) {
        // Only Vulture Aid shows Richtofen's wisp (`ctw_ric_watch_wisp_dist`).
        if !max {
            with_vm(world, |vm, world| {
                if let Some(p) = world.resource::<Zm>().players.get(&client.0).map(|p| p.obj) {
                    let perk = vm.string("specialty_nomotionsensor");
                    vm.spawn_named(world, "maps/mp/zombies/_zm_perks", "give_perk", Value::Object(p), vec![perk]);
                }
            });
        }
        let side = if max { "Maxis" } else { "Richtofen (given Vulture Aid)" };
        diag::info!(Sim, "bo2zm t6 follow test: {side}'s wisp");
    }
    let won = level_flag(world, "sq_wisp_success");
    let lost = level_flag(world, "sq_wisp_failed");
    if won || lost {
        LET_GO.store(true, Ordering::Relaxed);
        diag::info!(Sim, "bo2zm t6 follow test: wisp success {won} failed {lost} at {:.1}s", now as f32 / 1000.0);
        return;
    }
    let second = now % 1000 < tick;
    let Some((_, obj, at)) = wisp(world, max) else {
        if second {
            diag::info!(Sim, "bo2zm t6 follow test: no wisp yet");
        }
        return;
    };
    // Maxis's wisp leads at head height: he walks under it. Richtofen's
    // moves on once he is within 64 of it.
    let feet = [at[0], at[1], at[2] - if max { 50.0 } else { 20.0 }];
    if !FOLLOWING.swap(true, Ordering::Relaxed) {
        super::teleport_player(world, client, feet);
    } else {
        let mut f = frame(world);
        f.set_origin(client, feet);
        if let Some(ps) = f.player_mut(client) {
            ps.velocity = [0.0; 3];
        }
    }
    if second {
        let near = {
            let zm = world.resource::<Zm>();
            zm.actors
                .iter()
                .filter(|(_, a)| a.alive)
                .filter_map(|(n, _)| zm.ents.get(n))
                .filter(|e| {
                    let d = [e.origin[0] - at[0], e.origin[1] - at[1], e.origin[2] - at[2]];
                    d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < 256.0 * 256.0
                })
                .count()
        };
        let energy = with_vm(world, |vm, _| {
            let k = vm.intern("n_sq_energy");
            vm.to_text(&vm.raw_field(obj, k))
        })
        .unwrap_or_default();
        diag::info!(
            Sim,
            "bo2zm t6 follow test: wisp at ({:.0} {:.0} {:.0}) energy {energy} zombies within 256: {near}",
            at[0],
            at[1],
            at[2]
        );
    }
}

fn ows_want() -> Option<i64> {
    static OWS: OnceLock<Option<i64>> = OnceLock::new();
    *OWS.get_or_init(|| {
        let v = std::env::var("IW4L_T6_OWS").ok()?;
        Some((v.trim().parse::<f32>().ok()? * 1000.0) as i64)
    })
}

/// The target he is on (entity number), since when, and the targets done.
static OWS_ON: Mutex<Option<(u32, i64)>> = Mutex::new(None);
static OWS_DONE: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn ows_test(world: &mut World, now: i64) {
    let Some(base) = ows_want() else {
        return;
    };
    if now < base || now > base + 50_000 {
        return;
    }
    let tick = i64::from(crate::MATCH_TICK_MS);
    let at = |t: i64| (t..t + tick).contains(&now);
    let client = ClientId(0);
    // The coin toss: by the fountain, looking at it, use held 0.5 s.
    let fountain = [1082.0, 626.0, 0.0];
    if now < base + 1500 {
        let eye = [fountain[0] - 40.0, fountain[1], fountain[2] + 60.0];
        let feet = [eye[0], eye[1], eye[2] - 60.0];
        if at(base) {
            super::teleport_player(world, client, feet);
            diag::info!(Sim, "bo2zm t6 ows test: at the fountain");
        } else {
            let mut f = frame(world);
            f.set_origin(client, feet);
            if let Some(ps) = f.player_mut(client) {
                ps.velocity = [0.0; 3];
            }
        }
        super::set_player_view(world, client, super::autoplay::look(eye, [fountain[0], fountain[1], fountain[2] + 20.0]));
        if (base + 500..base + 1000).contains(&now) {
            let mut req = world.resource_mut::<crate::step::StepRequest>();
            for (id, cmd) in &mut req.input.cmds {
                if *id == client {
                    cmd.buttons |= buttons::USE;
                }
            }
        }
        return;
    }
    let started = level_flag(world, "sq_ows_start");
    let missed = level_flag(world, "sq_ows_target_missed");
    let won = level_flag(world, "sq_ows_success");
    // The targets standing: shootable `p6_zm_bu_target` models, shown.
    let up: Vec<(u32, [f32; 3], f32)> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter(|(_, e)| e.model == "p6_zm_bu_target" && e.can_damage && !e.hidden && !e.ghosted)
        .map(|(n, e)| (*n, e.origin, e.angles[1]))
        .collect();
    if now % 1000 < tick {
        let done = OWS_DONE.lock().unwrap().len();
        diag::info!(
            Sim,
            "bo2zm t6 ows test: {}s started {started} targets up {} shot {done} missed {missed} success {won}",
            (now - base) / 1000,
            up.len()
        );
    }
    if won {
        return;
    }
    // The target he is on: still up, and under 1 s on it; else the next.
    let mut on = OWS_ON.lock().unwrap();
    let keep = on.filter(|(n, since)| now - since < 1000 && up.iter().any(|u| u.0 == *n));
    if keep.is_none() {
        if let Some((n, _)) = *on {
            let gone = !up.iter().any(|u| u.0 == n);
            diag::info!(Sim, "bo2zm t6 ows test: target {n} {}", if gone { "down" } else { "still up after 1 s" });
        }
        let done = OWS_DONE.lock().unwrap();
        *on = up.iter().find(|u| !done.contains(&u.0)).map(|u| (u.0, now));
    }
    let Some((n, since)) = *on else {
        return;
    };
    drop(on);
    let Some(&(_, at_t, yaw)) = up.iter().find(|u| u.0 == n) else {
        return;
    };
    // Its middle: the board stands ~30 over its origin.
    let aim = [at_t[0], at_t[1], at_t[2] + 30.0];
    let eye = clear_eye(world, aim, yaw, 120.0);
    let feet = [eye[0], eye[1], eye[2] - 60.0];
    if now == since {
        OWS_DONE.lock().unwrap().push(n);
        super::teleport_player(world, client, feet);
        diag::info!(
            Sim,
            "bo2zm t6 ows test: target {n} at ({:.0} {:.0} {:.0}) from ({:.0} {:.0} {:.0})",
            at_t[0],
            at_t[1],
            at_t[2],
            feet[0],
            feet[1],
            feet[2]
        );
    } else {
        let mut f = frame(world);
        f.set_origin(client, feet);
        if let Some(ps) = f.player_mut(client) {
            ps.velocity = [0.0; 3];
        }
    }
    super::set_player_view(world, client, super::autoplay::look(eye, aim));
    // A pistol fires once per press: fire every other tick after 100 ms.
    if now - since >= 100 && (now / tick) % 2 == 0 {
        {
            let mut f = frame(world);
            if let Some(w) = f.player(client).map(|p| p.weapon) {
                crate::script_player::set_ammo_clip(&mut f, client, w, 8);
            }
        }
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= buttons::ATTACK;
            }
        }
    }
}

fn secs_env(name: &str) -> Option<i64> {
    let v = std::env::var(name).ok()?;
    Some((v.trim().parse::<f32>().ok()? * 1000.0) as i64)
}

/// The script structs with that targetname (`getstructarray`).
fn structs(world: &mut World, name: &str) -> Vec<ObjRef> {
    with_vm(world, |vm, _| {
        let f = vm.intern("struct_class_names");
        let Value::Array(types) = vm.raw_field(vm.level, f) else {
            return Vec::new();
        };
        let Some(Value::Array(names)) = types.get(&gsc_t6::Key::Str(vm.intern("targetname"))) else {
            return Vec::new();
        };
        let Some(Value::Array(list)) = names.get(&gsc_t6::Key::Str(vm.intern(name))) else {
            return Vec::new();
        };
        list.snapshot().values_in_order().filter_map(Value::as_obj).collect()
    })
    .unwrap_or_default()
}

/// The entity an object's field holds (`s_button.trig`): its number and the
/// middle of its volume.
fn field_ent(world: &mut World, o: ObjRef, field: &str) -> Option<(u32, [f32; 3])> {
    let n = with_vm(world, |vm, _| {
        let k = vm.intern(field);
        let v = vm.raw_field(o, k);
        entnum(vm, &v)
    })
    .flatten()?;
    let e = world.resource::<Zm>().ents.get(&n)?.clone();
    Some((n, super::triggers::center(world, &e)))
}

/// An eye spot `dist` out from `at` with open air back to it: along `yaw`
/// first, then the other way, then the eight ways round.
fn clear_eye(world: &mut World, at: [f32; 3], yaw: f32, dist: f32) -> [f32; 3] {
    let ways = [yaw, yaw + 180.0].into_iter().chain((0..8).map(|k| 45.0 * k as f32));
    for w in ways {
        let (s, c) = w.to_radians().sin_cos();
        let to = [at[0] + dist * c, at[1] + dist * s, at[2]];
        let t = frame(world).trace_static_world(at, to, [0.0; 3], [0.0; 3], crate::bullet_collision::MASK_SHOT);
        if t.startsolid == 0 && t.fraction > 0.95 {
            return to;
        }
    }
    let (s, c) = yaw.to_radians().sin_cos();
    [at[0] + dist * c, at[1] + dist * s, at[2]]
}

/// He hangs 45 out from the use trigger's middle `c`, looking at it, from
/// `since`; use is held from +500 ms to +700 ms (one press).
fn use_at(world: &mut World, now: i64, since: i64, c: [f32; 3]) {
    let client = ClientId(0);
    let eye = clear_eye(world, c, 0.0, 45.0);
    let feet = [eye[0], eye[1], eye[2] - 60.0];
    if now == since {
        super::teleport_player(world, client, feet);
    } else {
        let mut f = frame(world);
        f.set_origin(client, feet);
        if let Some(ps) = f.player_mut(client) {
            ps.velocity = [0.0; 3];
        }
    }
    super::set_player_view(world, client, super::autoplay::look(eye, c));
    if (since + 500..since + 700).contains(&now) {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= buttons::USE;
            }
        }
    }
}

/// The bulb he is pressing for and since when.
static BULB_ON: Mutex<Option<(String, i64)>> = Mutex::new(None);

fn bulbs_test(world: &mut World, now: i64) {
    static WANT: OnceLock<Option<i64>> = OnceLock::new();
    let Some(base) = *WANT.get_or_init(|| secs_env("IW4L_T6_BULBS")) else {
        return;
    };
    if now < base || now > base + 120_000 {
        return;
    }
    let tick = i64::from(crate::MATCH_TICK_MS);
    let done = level_flag(world, "sq_ip_puzzle_complete");
    let lit = with_vm(world, |vm, _| {
        let k = vm.intern("str_sq_bp_active_light");
        let v = vm.raw_field(vm.level, k);
        if matches!(v, Value::Undefined) { String::new() } else { vm.to_text(&v) }
    })
    .unwrap_or_default();
    if now % 1000 < tick {
        diag::info!(Sim, "bo2zm t6 bulbs test: {}s lit '{lit}' complete {done}", (now - base) / 1000);
    }
    if done {
        return;
    }
    // The board's switch (`level.t_start`) first, while no bulb is lit.
    if now < base + 2000 {
        let start = with_vm(world, |vm, _| vm.level).and_then(|l| field_ent(world, l, "t_start"));
        match start {
            Some((n, c)) => {
                if (base..base + tick).contains(&now) {
                    diag::info!(Sim, "bo2zm t6 bulbs test: the board's switch {n} at ({:.0} {:.0} {:.0})", c[0], c[1], c[2]);
                }
                use_at(world, now, base, c);
            }
            None if (base..base + tick).contains(&now) => {
                diag::info!(Sim, "bo2zm t6 bulbs test: no board switch yet");
            }
            None => {}
        }
        return;
    }
    if lit.is_empty() {
        return;
    }
    let mut on = BULB_ON.lock().unwrap();
    // A new bulb: go to its button. The same one 3 s on: press again.
    let fresh = match on.as_ref() {
        Some((tag, since)) => *tag != lit || now - since > 3000,
        None => true,
    };
    if fresh {
        *on = Some((lit.clone(), now));
    }
    let since = on.as_ref().map_or(now, |o| o.1);
    drop(on);
    let buttons = structs(world, "sq_bp_button");
    let button = buttons.into_iter().find(|s| {
        with_vm(world, |vm, _| {
            let k = vm.intern("script_string");
            vm.to_text(&vm.raw_field(*s, k)) == lit
        })
        .unwrap_or(false)
    });
    let Some((n, c)) = button.and_then(|s| field_ent(world, s, "trig")) else {
        if fresh {
            diag::info!(Sim, "bo2zm t6 bulbs test: no button for '{lit}'");
        }
        return;
    };
    if fresh {
        diag::info!(Sim, "bo2zm t6 bulbs test: bulb '{lit}': button {n} at ({:.0} {:.0} {:.0})", c[0], c[1], c[2]);
    }
    use_at(world, now, since, c);
}

/// IW4L_T6_LEVERS: the lever he is on (index into the order) and since when.
static LEVER_ON: Mutex<Option<(usize, i64)>> = Mutex::new(None);

fn levers_test(world: &mut World, now: i64) {
    static WANT: OnceLock<Option<i64>> = OnceLock::new();
    let Some(base) = *WANT.get_or_init(|| secs_env("IW4L_T6_LEVERS")) else {
        return;
    };
    if now < base || now > base + 120_000 {
        return;
    }
    let tick = i64::from(crate::MATCH_TICK_MS);
    let done = level_flag(world, "sq_ip_puzzle_complete");
    // The levers (`sq_ml_lever` models) in the puzzle's order
    // (`n_lever_order`, which BO2 shuffles; a player finds it by the sparks).
    let ents: Vec<(u32, ObjRef)> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter_map(|(n, e)| e.obj.map(|o| (*n, o)))
        .collect();
    let mut levers: Vec<(i64, u32, ObjRef, bool, String)> = with_vm(world, |vm, _| {
        let [tn, order, flipped, number] = ["targetname", "n_lever_order", "is_flipped", "n_flip_number"].map(|k| vm.intern(k));
        ents.iter()
            .filter(|(_, o)| vm.to_text(&vm.raw_field(*o, tn)) == "sq_ml_lever")
            .map(|(n, o)| {
                let ord = vm.raw_field(*o, order).as_int().map_or(-1, i64::from);
                let f = gsc_t6::truthy(&vm.raw_field(*o, flipped));
                (ord, *n, *o, f, vm.to_text(&vm.raw_field(*o, number)))
            })
            .collect()
    })
    .unwrap_or_default();
    levers.sort_by_key(|l| l.0);
    if now % 1000 < tick {
        let s: Vec<String> = levers.iter().map(|l| format!("{}:{}{}", l.0, if l.3 { "down#" } else { "up" }, if l.3 { l.4.as_str() } else { "" })).collect();
        diag::info!(Sim, "bo2zm t6 levers test: {}s levers [{}] complete {done}", (now - base) / 1000, s.join(" "));
    }
    if done || levers.is_empty() {
        return;
    }
    let mut on = LEVER_ON.lock().unwrap();
    // The first lever still up in the order; 3 s on each.
    let next = levers.iter().position(|l| !l.3);
    let cur = match *on {
        Some((i, since)) if Some(i) == next && now - since < 3000 => (i, since),
        _ => match next {
            Some(i) => (i, now),
            None => return,
        },
    };
    *on = Some(cur);
    drop(on);
    let (i, since) = cur;
    let (_, n, o, ..) = levers[i];
    let Some((t, c)) = field_ent(world, o, "trig") else {
        return;
    };
    if now == since {
        diag::info!(Sim, "bo2zm t6 levers test: lever {n} (order {i}) trigger {t} at ({:.0} {:.0} {:.0})", c[0], c[1], c[2]);
    }
    use_at(world, now, since, c);
}
