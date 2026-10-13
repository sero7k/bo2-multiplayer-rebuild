//! Origins (zm_tomb) test aids.
//!
//! IW4L_T6_GEN=<generator>[@<seconds>] (generator_start_bunker,
//! generator_tank_trench, generator_mid_trench, generator_nml_left,
//! generator_nml_right, generator_church): at that time (20 s unless given)
//! the player is put beside that generator, looking at it, and holds use
//! for half a second, again every 2 s until the capture starts (200
//! points). From then on the zone's state is logged every 5 s for two
//! minutes. Several generators, comma separated, are taken one after
//! another, 40 s apart (run with IW4L_T6_OPENALL=buy and IW4L_T6_GOD=1).
//!
//! IW4L_T6_GEN_PAP=1 (with IW4L_T6_GEN): 40 s after the last generator the
//! player, given 10000 points, stands at Pack-a-Punch and uses it with his
//! gun, takes it back 10 s later; `all_zones_captured`, his weapons and
//! points are logged.
//!
//! IW4L_T6_GEN_PERKS=1 (with IW4L_T6_GEN): 25 s after Pack-a-Punch's time
//! the player visits each perk machine (Quick Revive, Juggernog, Speed Cola,
//! Stamin-Up, Mule Kick), 12 s each, with 10000 points, uses it and logs
//! his perks, points and the machine's hint.
//!
//! IW4L_T6_GEN_GRAB=1 (with IW4L_T6_GEN): when a challenge box holds a
//! reward up (the church's, with the fire staff's tip, once that generator
//! is taken) the player stands at it and holds use until it is his; see
//! `grab_steps`.
//!
//! IW4L_T6_DIG=[<seconds>] (30 s unless given): the player picks up the
//! nearest shovel, then digs the nearest dig mound every 10 s, five times;
//! his shovel, weapons, points, the mounds left and the power-ups on the
//! ground are logged after each.
//! IW4L_T6_DIG=<seconds>:ice: the ice staff's parts. After the shovel he
//! digs, every 10 s until those parts are all carried (15 minutes at
//! most), the nearest mound that is up in a zone
//! a part is still buried in (`ice_staff_pieces` zone_substr: bunker, nml,
//! village; a miss there counts in any weather, the part comes up only
//! while it snows) and takes what comes up. The zombies die every 2 s (as
//! team mates would shoot them) so rounds go by and the weather turns.
//! Logged too: the snow, the parts still buried and the staff's parts.
//!
//! IW4L_T6_RECAPTURE=<seconds>: starts a generator recapture (zombies come
//! to take the generators back), see `recapture_test`.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::OnceLock;

use bevy_ecs::prelude::World;
use gsc_t6::{Key, Value, Vm};

use super::{frame, with_vm};
use crate::world::ClientId;

/// Time between generators in a list.
const SPAN: i64 = 40_000;

fn want() -> Option<&'static (Vec<String>, i64)> {
    static WANT: OnceLock<Option<(Vec<String>, i64)>> = OnceLock::new();
    WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_GEN").ok()?;
        let (names, at) = match v.split_once('@') {
            Some((n, s)) => (n, s.parse::<f32>().map_or(20_000, |s| (s * 1000.0) as i64)),
            None => (v.as_str(), 20_000),
        };
        Some((names.split(',').map(str::to_owned).collect(), at))
    })
    .as_ref()
}

/// `level.zone_capture.zones[name]`.
fn zone(vm: &mut Vm<World>, host: &mut World, name: &str) -> Option<gsc_t6::ObjRef> {
    let zc = vm.intern("zone_capture");
    let Value::Object(zc) = vm.get_field(host, vm.level, zc) else {
        return None;
    };
    let zones = vm.intern("zones");
    let Value::Array(a) = vm.get_field(host, zc, zones) else {
        return None;
    };
    match a.get(&Key::Str(vm.intern(name))) {
        Some(Value::Object(o)) => Some(o),
        _ => None,
    }
}

fn vec_field(vm: &mut Vm<World>, host: &mut World, o: gsc_t6::ObjRef, f: &str) -> [f32; 3] {
    let f = vm.intern(f);
    match vm.get_field(host, o, f) {
        Value::Vec3(v) => v,
        _ => [0.0; 3],
    }
}

/// `self.ent_flag[name]` as 0/1, or "-" when unset.
pub(super) fn ent_flag(vm: &mut Vm<World>, host: &mut World, o: gsc_t6::ObjRef, name: &str) -> String {
    let f = vm.intern("ent_flag");
    match vm.get_field(host, o, f) {
        Value::Array(a) => a
            .get(&Key::Str(vm.intern(name)))
            .map_or("-".to_owned(), |v| u8::from(gsc_t6::truthy(&v)).to_string()),
        _ => "-".to_owned(),
    }
}

pub(super) fn level_flag(vm: &mut Vm<World>, name: &str) -> bool {
    let f = vm.intern("flag");
    match vm.raw_field(vm.level, f) {
        Value::Array(a) => a
            .get(&Key::Str(vm.intern(name)))
            .is_some_and(|v| gsc_t6::truthy(&v)),
        _ => false,
    }
}

/// The first of 8 headings (from `yaw0`) `dist` out from `c` where the
/// player fits on a floor near `c`'s height.
pub(super) fn place_near(world: &mut World, c: [f32; 3], yaw0: f32, dist: f32) -> Option<[f32; 3]> {
    (0..8).find_map(|k| {
        let yaw = (yaw0 + 45.0 * k as f32).to_radians();
        let p = [
            c[0] + dist * yaw.cos(),
            c[1] + dist * yaw.sin(),
            c[2] + 40.0,
        ];
        let down = [p[0], p[1], p[2] - 200.0];
        let t = frame(world).trace_static_world(
            p,
            down,
            crate::bullet_collision::PLAYER_MINS,
            crate::bullet_collision::PLAYER_MAXS,
            crate::bullet_collision::MASK_PLAYER_SOLID,
        );
        let f: [f32; 3] = std::array::from_fn(|i| p[i] + (down[i] - p[i]) * t.fraction);
        (t.startsolid == 0 && t.fraction < 1.0 && (f[2] - c[2]).abs() < 48.0).then_some(f)
    })
}

/// The player looks at `at` and holds use this tick (added to the
/// autoplay's own keys, which ran first; no shooting meanwhile).
pub(super) fn look_and_use(world: &mut World, client: ClientId, at: [f32; 3]) {
    let eye = frame(world)
        .player_mut(client)
        .map(|p| [p.origin[0], p.origin[1], p.origin[2] + 60.0]);
    if let Some(eye) = eye {
        let d = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
        let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let view = [
            -d[2].atan2(flat.max(1.0)).to_degrees(),
            d[1].atan2(d[0]).to_degrees(),
            0.0,
        ];
        super::set_player_view(world, client, view);
    }
    let mut req = world.resource_mut::<crate::step::StepRequest>();
    for (id, cmd) in &mut req.input.cmds {
        if *id == client {
            cmd.buttons |= playerstate_iw4::buttons::USE;
            cmd.buttons &= !playerstate_iw4::buttons::ATTACK;
        }
    }
}

pub(crate) fn generator_test(world: &mut World, now: i64) {
    let Some((names, at0)) = want() else {
        return;
    };
    if std::env::var_os("IW4L_T6_GEN_PAP").is_some() {
        pap_steps(world, *at0 + SPAN * names.len() as i64, now);
    }
    if std::env::var_os("IW4L_T6_GEN_PERKS").is_some() {
        perk_steps(world, *at0 + SPAN * names.len() as i64 + 25_000, now);
    }
    if std::env::var_os("IW4L_T6_GEN_GRAB").is_some() {
        grab_steps(world, now);
    }
    let i = ((now - at0) / SPAN).clamp(0, names.len() as i64 - 1);
    let (name, at) = (names[i as usize].as_str(), at0 + SPAN * i);
    let last = i as usize + 1 == names.len();
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let spot = with_vm(world, |vm, world| {
        let z = zone(vm, world, name)?;
        Some((
            vec_field(vm, world, z, "origin"),
            vec_field(vm, world, z, "angles"),
        ))
    })
    .flatten();
    let Some((c, ang)) = spot else {
        if (at..at + tick).contains(&now) {
            diag::warn!(Sim, "bo2zm t6 gen test: no capture zone {name}");
        }
        return;
    };
    if (at..at + tick).contains(&now) {
        let feet = place_near(world, c, ang[1], 72.0);
        let feet = feet.unwrap_or([c[0], c[1], c[2] + 4.0]);
        super::teleport_player(world, client, feet);
        diag::info!(
            Sim,
            "bo2zm t6 gen test: {name} at ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
            c[0],
            c[1],
            c[2],
            feet[0],
            feet[1],
            feet[2]
        );
    }
    // Use for half a second every 2 s until the capture starts (one press
    // sometimes lands before the generator's trigger is live).
    if (at + 1000..at + 9000).contains(&now) && (now - at) % 2000 < 500 {
        let started = with_vm(world, |vm, world| {
            let z = zone(vm, world, name);
            level_flag(vm, "zone_capture_in_progress")
                || z.is_some_and(|z| ent_flag(vm, world, z, "player_controlled") == "1")
        })
        .unwrap_or(false);
        if !started {
            look_and_use(world, client, [c[0], c[1], c[2] + 30.0]);
        }
    }
    if std::env::var_os("IW4L_T6_GEN_BOX").is_some() {
        box_steps(world, name, at, now);
    }
    if now >= at + 2000 && now < at + if last { 122_000 } else { SPAN } && (now - at) % 5000 < tick
    {
        let line = with_vm(world, |vm, world| {
            let z = zone(vm, world, name)?;
            let (cur, caps) = (vm.intern("n_current_progress"), vm.intern("capture_zombies"));
            let progress = vm.get_field(world, z, cur);
            let progress = vm.to_text(&progress);
            let zombies = match vm.get_field(world, z, caps) {
                Value::Array(a) => a.len().to_string(),
                _ => "-".to_owned(),
            };
            Some(format!(
                "in_progress {} contested {} player_controlled {} progress {progress} capture_zombies {zombies} power {}",
                u8::from(level_flag(vm, "zone_capture_in_progress")),
                ent_flag(vm, world, z, "zone_contested"),
                ent_flag(vm, world, z, "player_controlled"),
                u8::from(level_flag(vm, "power_on")),
            ))
        })
        .flatten()
        .unwrap_or_default();
        let score = frame(world)
            .player_mut(client)
            .map(|p| p.origin)
            .unwrap_or_default();
        diag::info!(
            Sim,
            "bo2zm t6 gen test {}s: {name} {line}; player ({:.0} {:.0} {:.0})",
            (now - at) / 1000,
            score[0],
            score[1],
            score[2]
        );
    }
}

/// IW4L_T6_GEN_BOX=1 (with IW4L_T6_GEN): 20 s after the capture started the
/// player stands at that zone's magic box with 10000 points and uses it,
/// takes the weapon 6.5 s later, and his weapons and points are logged.
fn box_steps(world: &mut World, name: &str, at: i64, now: i64) {
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let t = now - at;
    if !(20_000..30_000 + tick).contains(&t) {
        return;
    }
    let spot = with_vm(world, |vm, world| {
        let z = zone(vm, world, name)?;
        let f = vm.intern("mystery_boxes");
        let Value::Array(a) = vm.get_field(world, z, f) else {
            return None;
        };
        let Some(Value::Object(b)) = a.get(&Key::Int(0)) else {
            return None;
        };
        Some((
            vec_field(vm, world, b, "origin"),
            vec_field(vm, world, b, "angles"),
        ))
    })
    .flatten();
    let Some((c, ang)) = spot else {
        if t < tick {
            diag::warn!(Sim, "bo2zm t6 gen box test: {name} has no box");
        }
        return;
    };
    if t < 20_000 + tick {
        with_vm(world, |vm, world| {
            if let Some(p) = world.resource::<super::Zm>().players.get(&0).map(|p| p.obj) {
                let f = vm.intern("score");
                vm.set_field(world, p, f, Value::Int(10_000));
            }
        });
        let feet = place_near(world, c, ang[1] - 90.0, 56.0).unwrap_or([c[0], c[1], c[2] + 4.0]);
        super::teleport_player(world, client, feet);
        diag::info!(
            Sim,
            "bo2zm t6 gen box test: box at ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
            c[0],
            c[1],
            c[2],
            feet[0],
            feet[1],
            feet[2]
        );
    }
    if (21_000..21_300).contains(&t) || (27_500..27_800).contains(&t) {
        look_and_use(world, client, [c[0], c[1], c[2] + 20.0]);
    }
    if (30_000..30_000 + tick).contains(&t) {
        let names: Vec<String> = {
            let f = frame(world);
            crate::script_player::weapons(&f, client, crate::script_player::WeaponList::All)
                .into_iter()
                .map(|w| crate::script_player::weapon_name(&f, w))
                .collect()
        };
        let score = frame(world).client_meta(client).map_or(0, |m| m.score);
        let hint = world
            .resource::<super::Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        diag::info!(
            Sim,
            "bo2zm t6 gen box test: weapons {names:?}, points {score}, hint {hint:?}"
        );
    }
}

/// IW4L_T6_GEN_GRAB=1 (with IW4L_T6_GEN): a challenge box holding a reward
/// up for the taking (`b_busy`, its `waiting_for_grab` flag up: the
/// church's, once that generator is taken, holds up the fire staff's tip)
/// gets the player put beside it, looking at it and holding use 0.3 s of
/// every second until it is taken. Every 2 s while a box is busy, and every
/// 5 s for 30 s after: which box, its two flags, and the fire staff's parts.
fn grab_steps(world: &mut World, now: i64) {
    static PUT: AtomicBool = AtomicBool::new(false);
    static LAST: AtomicI64 = AtomicI64::new(i64::MIN);
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let busy = with_vm(world, |vm, world| {
        let f = vm.intern("a_uts_challenge_boxes");
        let Value::Array(a) = vm.raw_field(vm.level, f) else {
            return None;
        };
        let boxes: Vec<gsc_t6::ObjRef> = a.snapshot().values_in_order().filter_map(Value::as_obj).collect();
        let (b_busy, loc) = (vm.intern("b_busy"), vm.intern("str_location"));
        for b in boxes {
            if !gsc_t6::truthy(&vm.raw_field(b, b_busy)) {
                continue;
            }
            let name = vm.to_text(&vm.raw_field(b, loc));
            let flags = format!(
                "waiting_for_grab {}, reward_timeout {}",
                ent_flag(vm, world, b, "waiting_for_grab"),
                ent_flag(vm, world, b, "reward_timeout")
            );
            let grab = ent_flag(vm, world, b, "waiting_for_grab") == "1";
            return Some((vec_field(vm, world, b, "origin"), vec_field(vm, world, b, "angles"), name, flags, grab));
        }
        None
    })
    .flatten();
    if let Some((at, ang, name, flags, grab)) = &busy {
        LAST.store(now, Ordering::Relaxed);
        if *grab {
            if !PUT.swap(true, Ordering::Relaxed) {
                let feet = place_near(world, *at, ang[1], 44.0).unwrap_or([at[0], at[1], at[2] + 4.0]);
                super::teleport_player(world, client, feet);
                diag::info!(
                    Sim,
                    "bo2zm t6 grab test {}s: the {name} box at ({:.0} {:.0} {:.0}) holds a reward up; player at ({:.0} {:.0} {:.0})",
                    now / 1000,
                    at[0],
                    at[1],
                    at[2],
                    feet[0],
                    feet[1],
                    feet[2]
                );
            }
            if now % 1000 < 300 {
                look_and_use(world, client, [at[0], at[1], at[2] + 30.0]);
            }
        }
        if now % 2000 < tick {
            let parts = fire_parts(world);
            diag::info!(Sim, "bo2zm t6 grab test {}s: the {name} box is busy, {flags}{parts}", now / 1000);
        }
    } else {
        let last = LAST.load(Ordering::Relaxed);
        if last != i64::MIN && now - last < 30_000 && now % 5000 < tick {
            let parts = fire_parts(world);
            diag::info!(Sim, "bo2zm t6 grab test {}s: no box busy{parts}", now / 1000);
        }
    }
}

/// The fire staff's parts: carried, where they lie, or nowhere (ghosted
/// pieces still count as lying where they wait).
fn fire_parts(world: &mut World) -> String {
    with_vm(world, |vm, world| {
        let (parts, _) = super::tomb_craft::survey(vm, world, "elemental_staff_fire");
        let list: Vec<String> = parts
            .iter()
            .map(|p| match (p.shared, p.at) {
                (true, _) => format!("{} carried", p.name),
                (false, Some(a)) => format!("{} at ({:.0} {:.0} {:.0})", p.name, a[0], a[1], a[2]),
                (false, None) => format!("{} nowhere", p.name),
            })
            .collect();
        format!("; fire staff parts {list:?}")
    })
    .unwrap_or_default()
}

/// IW4L_T6_GEN_PAP=1: see the module notes. `at` is when the player is
/// put at the machine.
fn pap_steps(world: &mut World, at: i64, now: i64) {
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let t = now - at;
    if !(0..20_000 + tick).contains(&t) {
        return;
    }
    let Some((_, c, ang)) =
        super::autoplay::ent_by(world, "script_noteworthy", "specialty_weapupgrade")
    else {
        if t < tick {
            diag::warn!(
                Sim,
                "bo2zm t6 gen pap test: no specialty_weapupgrade trigger"
            );
        }
        return;
    };
    if t < tick {
        with_vm(world, |vm, world| {
            if let Some(p) = world.resource::<super::Zm>().players.get(&0).map(|p| p.obj) {
                let f = vm.intern("score");
                vm.set_field(world, p, f, Value::Int(10_000));
            }
        });
        let feet = place_near(world, c, ang[1], 48.0).unwrap_or([c[0], c[1], c[2] + 4.0]);
        super::teleport_player(world, client, feet);
        diag::info!(
            Sim,
            "bo2zm t6 gen pap test: trigger at ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
            c[0],
            c[1],
            c[2],
            feet[0],
            feet[1],
            feet[2]
        );
    }
    if (1_000..1_300).contains(&t) || (11_000..11_300).contains(&t) {
        look_and_use(world, client, [c[0], c[1], c[2] + 30.0]);
    }
    if [500, 2_000, 6_000, 12_000, 20_000]
        .iter()
        .any(|s| (*s..*s + tick).contains(&t))
    {
        let names: Vec<String> = {
            let f = frame(world);
            crate::script_player::weapons(&f, client, crate::script_player::WeaponList::All)
                .into_iter()
                .map(|w| crate::script_player::weapon_name(&f, w))
                .collect()
        };
        let score = frame(world).client_meta(client).map_or(0, |m| m.score);
        let hint = world
            .resource::<super::Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        let flags = with_vm(world, |vm, _| {
            ["all_zones_captured", "power_on", "pack_machine_in_use"]
                .map(|f| format!("{f} {}", u8::from(level_flag(vm, f))))
                .join(", ")
        })
        .unwrap_or_default();
        diag::info!(
            Sim,
            "bo2zm t6 gen pap test {}s: {flags}; weapons {names:?}, points {score}, hint {hint:?}",
            t / 1000
        );
    }
}

/// The perks of Origins' fixed machines, in the order the perk test visits
/// them.
const PERKS: [&str; 5] = [
    "specialty_quickrevive",
    "specialty_armorvest",
    "specialty_fastreload",
    "specialty_longersprint",
    "specialty_additionalprimaryweapon",
];

/// IW4L_T6_GEN_PERKS=1: see the module notes. `at` is when the player is
/// put at the first machine.
fn perk_steps(world: &mut World, at: i64, now: i64) {
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let t = now - at;
    if t < 0 || t >= 12_000 * PERKS.len() as i64 {
        return;
    }
    let (perk, t) = (PERKS[(t / 12_000) as usize], t % 12_000);
    let Some((_, c, ang)) = super::autoplay::ent_by(world, "script_noteworthy", perk) else {
        if t < tick {
            diag::warn!(Sim, "bo2zm t6 gen perk test: no {perk} trigger");
        }
        return;
    };
    if t < tick {
        with_vm(world, |vm, world| {
            if let Some(p) = world.resource::<super::Zm>().players.get(&0).map(|p| p.obj) {
                let f = vm.intern("score");
                vm.set_field(world, p, f, Value::Int(10_000));
            }
        });
        let feet = place_near(world, c, ang[1], 48.0).unwrap_or([c[0], c[1], c[2] + 4.0]);
        super::teleport_player(world, client, feet);
        diag::info!(
            Sim,
            "bo2zm t6 gen perk test: {perk} trigger at ({:.0} {:.0} {:.0}); player at ({:.0} {:.0} {:.0})",
            c[0],
            c[1],
            c[2],
            feet[0],
            feet[1],
            feet[2]
        );
    }
    if (1_000..1_300).contains(&t) {
        look_and_use(world, client, [c[0], c[1], c[2] + 30.0]);
    }
    if (800..800 + tick).contains(&t) || (10_000..10_000 + tick).contains(&t) {
        let score = frame(world).client_meta(client).map_or(0, |m| m.score);
        let zm = world.resource::<super::Zm>();
        let hint = zm.hints.get(&0).cloned().unwrap_or_default();
        let perks: Vec<String> = zm
            .players
            .get(&0)
            .map(|p| p.perks.iter().cloned().collect())
            .unwrap_or_default();
        diag::info!(
            Sim,
            "bo2zm t6 gen perk test {}s: {perk}; perks {perks:?}, points {score}, hint {hint:?}",
            t / 1000
        );
    }
}

/// The entity numbers and origins of the script models showing `model`,
/// nearest to `from` first.
fn models_near(world: &World, model: &str, from: [f32; 3]) -> Vec<(u32, [f32; 3])> {
    let mut v: Vec<(u32, [f32; 3])> = world
        .resource::<super::Zm>()
        .ents
        .iter()
        .filter(|(_, e)| e.model == model)
        .map(|(n, e)| (*n, e.origin))
        .collect();
    let d = |o: &[f32; 3]| (0..3).map(|i| (o[i] - from[i]).powi(2)).sum::<f32>();
    v.sort_by(|a, b| d(&a.1).total_cmp(&d(&b.1)));
    v
}

/// IW4L_T6_DIG: see the module notes.
pub(crate) fn dig_test(world: &mut World, now: i64) {
    static AT: OnceLock<Option<(i64, bool)>> = OnceLock::new();
    let Some((at, ice)) = *AT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_DIG").ok()?;
        let (s, ice) = match v.split_once(':') {
            Some((s, mode)) => (s.to_owned(), mode == "ice"),
            None => (v, false),
        };
        Some((s.parse::<f32>().map_or(30_000, |s| (s * 1000.0) as i64), ice))
    }) else {
        return;
    };
    // Ice mode: done once the mounds' parts are all carried (a craft run
    // then takes him to the portal and the table).
    static DUG: AtomicBool = AtomicBool::new(false);
    if DUG.load(Ordering::Relaxed) {
        return;
    }
    static SPOT: std::sync::Mutex<Option<[f32; 3]>> = std::sync::Mutex::new(None);
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let t = now - at;
    // Step 0 is the shovel, 1..=5 the mounds (ice: on to 15 minutes).
    let end = if ice { 900_000 } else { 60_000 };
    if !(0..end + tick).contains(&t) {
        return;
    }
    let (step, t) = (t / 10_000, t % 10_000);
    if ice && step >= 1 && now % 2000 < tick {
        cull(world);
    }
    let me = frame(world)
        .player_mut(client)
        .map(|p| p.origin)
        .unwrap_or_default();
    if t < tick && ice && step >= 1 {
        if ice_parts_carried(world) {
            DUG.store(true, Ordering::Relaxed);
            *SPOT.lock().unwrap() = None;
            diag::info!(Sim, "bo2zm t6 dig test {step}: the ice staff's dug parts are all carried; digging done");
            return;
        }
        let (spot, text) = ice_mound(world, me);
        *SPOT.lock().unwrap() = spot;
        match spot {
            Some(c) => {
                let feet = place_near(world, c, 0.0, 40.0).unwrap_or([c[0], c[1], c[2] + 4.0]);
                super::teleport_player(world, client, feet);
                diag::info!(
                    Sim,
                    "bo2zm t6 dig test {step}: ice mound at ({:.0} {:.0} {:.0}); {text}",
                    c[0],
                    c[1],
                    c[2]
                );
            }
            None => diag::info!(Sim, "bo2zm t6 dig test {step}: no ice mound up; {text}"),
        }
    } else if t < tick {
        let model = if step == 0 {
            "p6_zm_tm_shovel"
        } else {
            "p6_zm_tm_dig_mound"
        };
        let near = models_near(world, model, me);
        let Some(&(n, c)) = near.first() else {
            diag::warn!(Sim, "bo2zm t6 dig test {step}: no {model} left");
            *SPOT.lock().unwrap() = None;
            return;
        };
        if step == 0 {
            with_vm(world, |vm, world| {
                if let Some(p) = world.resource::<super::Zm>().players.get(&0).map(|p| p.obj) {
                    let f = vm.intern("score");
                    vm.set_field(world, p, f, Value::Int(10_000));
                }
            });
        }
        let feet = place_near(world, c, 0.0, 40.0).unwrap_or([c[0], c[1], c[2] + 4.0]);
        super::teleport_player(world, client, feet);
        *SPOT.lock().unwrap() = Some(c);
        diag::info!(
            Sim,
            "bo2zm t6 dig test {step}: {model} ent{n} at ({:.0} {:.0} {:.0}) of {}; player at ({:.0} {:.0} {:.0})",
            c[0],
            c[1],
            c[2],
            near.len(),
            feet[0],
            feet[1],
            feet[2]
        );
    }
    let spot = *SPOT.lock().unwrap();
    if let Some(c) = spot
        && (1_000..1_400).contains(&t)
    {
        look_and_use(world, client, [c[0], c[1], c[2] + 10.0]);
    }
    // A dug-up gun floats over the mound until it is used.
    if let Some(c) = spot
        && (5_000..5_400).contains(&t)
    {
        look_and_use(world, client, [c[0], c[1], c[2] + 40.0]);
    }
    if (800..800 + tick).contains(&t) || (8_000..8_000 + tick).contains(&t) {
        let names: Vec<String> = {
            let f = frame(world);
            crate::script_player::weapons(&f, client, crate::script_player::WeaponList::All)
                .into_iter()
                .map(|w| crate::script_player::weapon_name(&f, w))
                .collect()
        };
        let score = frame(world).client_meta(client).map_or(0, |m| m.score);
        let hint = world
            .resource::<super::Zm>()
            .hints
            .get(&0)
            .cloned()
            .unwrap_or_default();
        let dig = with_vm(world, |vm, world| {
            let p = world
                .resource::<super::Zm>()
                .players
                .get(&0)
                .map(|p| p.obj)?;
            let f = vm.intern("dig_vars");
            let Value::Array(a) = vm.get_field(world, p, f) else {
                return None;
            };
            Some(
                ["has_shovel", "n_spots_dug", "n_losing_streak"]
                    .map(|k| {
                        let v = a.get(&Key::Str(vm.intern(k))).unwrap_or_default();
                        format!("{k} {}", vm.to_text(&v))
                    })
                    .join(", "),
            )
        })
        .flatten()
        .unwrap_or_default();
        let mounds = models_near(world, "p6_zm_tm_dig_mound", me).len();
        let powerups: Vec<String> = world
            .resource::<super::Zm>()
            .ents
            .values()
            .filter(|e| {
                (e.model.starts_with("zombie_") && !e.model.contains("vending"))
                    || e.model.starts_with("t6_wpn")
                    || e.model.contains("powerup")
            })
            .map(|e| e.model.clone())
            .collect();
        let staff = if ice { ice_parts(world) } else { String::new() };
        diag::info!(
            Sim,
            "bo2zm t6 dig test {step} {}s: {dig}; weapons {names:?}, points {score}, mounds {mounds}, pickups {powerups:?}, hint {hint:?}{staff}",
            t / 1000
        );
    }
}

/// Ice mode: the nearest mound that is up and not dug whose dig spot
/// (`level.a_dig_spots`) is in a zone an ice part is still buried in, and
/// the snow and parts left (with their misses).
fn ice_mound(world: &mut World, me: [f32; 3]) -> (Option<[f32; 3]>, String) {
    with_vm(world, |vm, _| {
        let level_field = |vm: &mut Vm<World>, name: &str| {
            let f = vm.intern(name);
            vm.raw_field(vm.level, f)
        };
        let snow = level_field(vm, "weather_snow");
        let snow = vm.to_text(&snow);
        let round = level_field(vm, "round_number");
        let round = vm.to_text(&round);
        let (zs, nm) = (vm.intern("zone_substr"), vm.intern("num_misses"));
        let left: Vec<(String, String)> = match level_field(vm, "ice_staff_pieces") {
            Value::Array(a) => {
                let list: Vec<_> = a.read().values_in_order().filter_map(Value::as_obj).collect();
                list.into_iter()
                    .map(|p| {
                        let (z, m) = (vm.raw_field(p, zs), vm.raw_field(p, nm));
                        (vm.to_text(&z), vm.to_text(&m))
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        let text = format!(
            "round {round}, snow {snow}, parts buried {:?}",
            left.iter().map(|(z, m)| format!("{z} (misses {m})")).collect::<Vec<_>>()
        );
        let (o, z, d, m) = (
            vm.intern("origin"),
            vm.intern("str_zone"),
            vm.intern("dug"),
            vm.intern("m_dig"),
        );
        let spots: Vec<_> = match level_field(vm, "a_dig_spots") {
            Value::Array(a) => a.read().values_in_order().filter_map(Value::as_obj).collect(),
            _ => Vec::new(),
        };
        let d2 = |c: &[f32; 3]| (0..3).map(|i| (c[i] - me[i]).powi(2)).sum::<f32>();
        let best = spots
            .into_iter()
            .filter(|&s| !gsc_t6::truthy(&vm.raw_field(s, d)) && vm.raw_field(s, m).as_obj().is_some())
            .filter_map(|s| {
                let zone = vm.raw_field(s, z);
                let zone = vm.to_text(&zone);
                left.iter().any(|(sub, _)| zone.contains(sub.as_str())).then_some(())?;
                vm.raw_field(s, o).as_vec3()
            })
            .min_by(|a, b| d2(a).total_cmp(&d2(b)));
        (best, text)
    })
    .unwrap_or((None, String::new()))
}

/// Ice mode: every part that comes out of the mounds (all but the gem,
/// which waits in the Crazy Place) is carried.
fn ice_parts_carried(world: &mut World) -> bool {
    with_vm(world, |vm, world| {
        let (parts, _) = super::tomb_craft::survey(vm, world, "elemental_staff_water");
        let dug: Vec<_> = parts.iter().filter(|p| !p.name.contains("crystal")).collect();
        !dug.is_empty() && dug.iter().all(|p| p.shared)
    })
    .unwrap_or(false)
}

/// Ice mode: the ice staff's parts (carried in the shared inventory, or
/// where they lie).
fn ice_parts(world: &mut World) -> String {
    with_vm(world, |vm, world| {
        let (parts, _) = super::tomb_craft::survey(vm, world, "elemental_staff_water");
        let list: Vec<String> = parts
            .iter()
            .map(|p| match (p.shared, p.at) {
                (true, _) => format!("{} carried", p.name),
                (false, Some(a)) => format!("{} at ({:.0} {:.0} {:.0})", p.name, a[0], a[1], a[2]),
                (false, None) => format!("{} nowhere", p.name),
            })
            .collect();
        format!("; ice staff parts {list:?}")
    })
    .unwrap_or_default()
}

/// Ice mode: every living zombie dies (as team mates would shoot them).
fn cull(world: &mut World) {
    let alive: Vec<(u32, i32, [f32; 3])> = {
        let zm = world.resource::<super::Zm>();
        zm.actors
            .iter()
            .filter(|(_, a)| a.alive)
            .filter_map(|(n, a)| Some((*n, a.health, zm.ents.get(n)?.origin)))
            .collect()
    };
    if alive.is_empty() {
        return;
    }
    with_vm(world, |vm, world| {
        for (n, health, origin) in alive {
            super::actors::damage(
                vm,
                world,
                n,
                Value::Undefined,
                Value::Undefined,
                health + 1,
                0,
                "MOD_RIFLE_BULLET",
                "none",
                [origin[0], origin[1], origin[2] + 60.0],
                [0.0, 0.0, -1.0],
                "head",
            );
        }
    });
}

/// IW4L_T6_RECAPTURE=<seconds>: at that time the player is put at the start
/// spawn, away from the generators (capture zombies only go for a generator
/// when no player is within 700 units of it), and the level is sent
/// `force_recapture_start`, the notify the game's own recapture tracker
/// waits for between rounds (it still wants round 10+ and four generators
/// held: run with IW4L_T6_ROUND_JUMP=10 and four IW4L_T6_GEN generators).
/// From then on, every 5 s for 150 s: the target generator, the recapture
/// flags, and each zone's `player_controlled`, progress and whether its
/// perk machines are locked.
pub(crate) fn recapture_test(world: &mut World, now: i64) {
    static AT: OnceLock<Option<i64>> = OnceLock::new();
    let Some(at) = *AT.get_or_init(|| {
        let s = std::env::var("IW4L_T6_RECAPTURE").ok()?;
        Some((s.parse::<f32>().unwrap_or(200.0) * 1000.0) as i64)
    }) else {
        return;
    };
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    if (at..at + tick).contains(&now) {
        super::teleport_player(world, client, [2688.0, 5120.0, -340.0]);
        with_vm(world, |vm, world| {
            let level = vm.level;
            vm.notify_str(world, level, "force_recapture_start", &[]);
        });
        diag::info!(Sim, "bo2zm t6 recapture test: force_recapture_start sent");
    }
    if now < at || now >= at + 150_000 || (now - at) % 5000 >= tick {
        return;
    }
    let line = with_vm(world, |vm, world| {
        let zc = vm.intern("zone_capture");
        let Value::Object(zc) = vm.get_field(world, vm.level, zc) else {
            return "no zone_capture".to_owned();
        };
        let f = vm.intern("recapture_target");
        let target = vm.get_field(world, zc, f);
        let target = vm.to_text(&target);
        let f = vm.intern("recapture_zombies");
        let rz = match vm.get_field(world, zc, f) {
            Value::Array(a) => a.len().to_string(),
            _ => "-".to_owned(),
        };
        let f = vm.intern("recapture_zombies_killed");
        let killed = vm.get_field(world, vm.level, f);
        let killed = vm.to_text(&killed);
        let f = vm.intern("zones");
        let Value::Array(zones) = vm.get_field(world, zc, f) else {
            return "no zones".to_owned();
        };
        let keys: Vec<Key> = zones.read().keys().collect();
        let mut each = Vec::new();
        for k in keys {
            let Some(Value::Object(z)) = zones.get(&k) else {
                continue;
            };
            let name = match &k {
                Key::Str(s) => vm.to_text(&Value::Str(*s)),
                _ => continue,
            };
            let short = name.trim_start_matches("generator_").to_owned();
            let pc = ent_flag(vm, world, z, "player_controlled");
            let f = vm.intern("n_current_progress");
            let progress = vm.get_field(world, z, f);
            let progress = vm.to_text(&progress);
            let f = vm.intern("perk_machines");
            let locked = match vm.get_field(world, z, f) {
                Value::Array(a) => {
                    let il = vm.intern("is_locked");
                    let vals: Vec<Value> = a.read().values_in_order().cloned().collect();
                    vals.iter()
                        .map(|v| match v {
                            Value::Object(t) => {
                                let l = vm.get_field(world, *t, il);
                                if gsc_t6::truthy(&l) { "L" } else { "u" }
                            }
                            _ => "?",
                        })
                        .collect::<String>()
                }
                _ => "-".to_owned(),
            };
            each.push(format!("{short}={pc}/{progress}/{locked}"));
        }
        format!(
            "target {target} event {} under_attack {} cleared {} lost {} zombies {rz} killed {killed}; {}",
            u8::from(level_flag(vm, "recapture_event_in_progress")),
            u8::from(level_flag(vm, "generator_under_attack")),
            u8::from(level_flag(vm, "recapture_zombies_cleared")),
            u8::from(level_flag(vm, "generator_lost_to_recapture_zombies")),
            each.join(" ")
        )
    })
    .unwrap_or_default();
    diag::info!(Sim, "bo2zm t6 recapture test {}s: {line}", (now - at) / 1000);
}
