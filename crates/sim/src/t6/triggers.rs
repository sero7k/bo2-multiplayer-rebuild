//! Triggers: touch triggers (`trigger_multiple`, `trigger_radius`) notify
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/triggers.rs
//! `trigger` with the player every frame he is inside; use triggers
=======
//! `trigger` with the player every frame he is inside (and with zombies when
//! the spawnflags take AI); use triggers
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/triggers.rs
//! (`trigger_use`, `trigger_use_touch`, `trigger_radius_use`) when he
//! presses use inside the one he faces best. A trigger hidden from a player
//! (`setinvisibletoplayer`) ignores him.

use bevy_ecs::prelude::World;
use gsc_t6::{ObjRef, Value};

use super::{Ent, Zm, frame, with_vm};
use crate::bullet_collision::{PLAYER_MAXS, PLAYER_MINS};
use crate::script::host::triggers::{Volume, t6_volume};
use crate::world::ClientId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Touch,
    Use,
}

pub(super) fn kind(classname: &str) -> Option<Kind> {
    match classname {
        "trigger_multiple" | "trigger_radius" | "trigger_once" | "trigger_box" => Some(Kind::Touch),
        "trigger_use" | "trigger_use_touch" | "trigger_radius_use" | "trigger_box_use" => {
            Some(Kind::Use)
        }
        _ => None,
    }
}

pub(super) fn brush_index(e: &Ent) -> Option<u32> {
    e.brush.as_deref()?.strip_prefix('*')?.parse().ok()
}

/// An IW4/IW5 map's trigger model (`"model" "?N"`).
pub(super) fn trigger_hull_index(e: &Ent) -> Option<u32> {
    e.model.strip_prefix('?')?.parse().ok()
}

fn cylinder(e: &Ent) -> Option<(f32, f32)> {
    (e.radius > 0.0).then_some((e.radius, e.height.max(1.0)))
}

/// Does `e`'s volume touch the box `mins..maxs`?
pub(super) fn touches(world: &mut World, e: &Ent, mins: [f32; 3], maxs: [f32; 3]) -> bool {
    // A turned box: the other box's centre against it, grown by that box's
    // reach along each of its axes.
    if let Some([w, l, h]) = e.box_dims {
        let (f, r, u) = gsc_t6::math::angle_vectors(e.angles);
        let c: [f32; 3] = std::array::from_fn(|i| (mins[i] + maxs[i]) * 0.5 - e.origin[i]);
        let half: [f32; 3] = std::array::from_fn(|i| (maxs[i] - mins[i]) * 0.5);
        let reach =
            |a: [f32; 3]| a[0].abs() * half[0] + a[1].abs() * half[1] + a[2].abs() * half[2];
        let dot = |a: [f32; 3]| a[0] * c[0] + a[1] * c[1] + a[2] * c[2];
        return dot(f).abs() <= w * 0.5 + reach(f)
            && dot(r).abs() <= l * 0.5 + reach(r)
            && dot(u).abs() <= h * 0.5 + reach(u);
    }
    let f = frame(world);
    // A map from IW4/IW5 names its trigger brushes `?N`: the map entities'
    // own trigger hulls (MapEnts trigger models), not clip brushes; the
    // IW4 script host reads them the same way (`script/host/triggers.rs`).
    if let Some(n) = trigger_hull_index(e) {
        return match f.clip_cmodels().triggers.get(n as usize) {
            Some(hulls) if !hulls.is_empty() => {
                Volume::Hulls { origin: e.origin, hulls }.touches(mins, maxs)
            }
            _ => false,
        };
    }
    match t6_volume(&f, e.origin, cylinder(e), brush_index(e)) {
        // A brush trigger turned since the map placed it (linked to the
        // Tranzit bus) turns its brushes with it.
        Some(v) => match e.spawn_angles.filter(|a| *a != e.angles) {
            Some(spawn) => v.touches_turned(mins, maxs, turn_between(spawn, e.angles)),
            None => v.touches(mins, maxs),
        },
        None => false,
    }
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/triggers.rs
/// A look-at use trigger's test: how far along the player's view (from
/// his eye, within the use radius) it meets the trigger's turned box, with
/// nothing solid in front of it (the chalk board the weapon sits on is
/// allowed: a few units). BO2 builds each wall buy's box from the
/// weapon model on the wall, nudged out of the wall so the view meets it
/// before the wall, and asks the engine for a look (`require_look_at`).
fn look_hit(world: &mut World, e: &Ent, eye: [f32; 3], dir: [f32; 3], reach: f32) -> Option<f32> {
    let [w, l, h] = e.box_dims?;
    let (f, r, u) = gsc_t6::math::angle_vectors(e.angles);
    let rel = gsc_t6::math::sub(eye, e.origin);
    let (mut near, mut far) = (0.0f32, reach);
    for (axis, half) in [(f, w * 0.5), (r, l * 0.5), (u, h * 0.5)] {
        let o = gsc_t6::math::dot(rel, axis);
        let d = gsc_t6::math::dot(dir, axis);
        if d.abs() < 1e-6 {
            if o.abs() > half {
                return None;
            }
            continue;
=======
/// Does a trigger faced at `score` (low priority or not) win over the best
/// so far (its score and priority)? A normal priority trigger beats a low
/// one whatever their scores; among equals the better faced wins.
fn hint_beats(score: f32, low: bool, best: Option<(f32, bool)>) -> bool {
    best.is_none_or(|(b_score, b_low)| (!low && b_low) || (low == b_low && score > b_score))
}

/// The rotation from angles `from` to angles `to` (row i = world axis i):
/// R(to) * R(from)^T, R's columns being forward, left and up.
fn turn_between(from: [f32; 3], to: [f32; 3]) -> [[f32; 3]; 3] {
    let (f0, r0, u0) = gsc_t6::math::angle_vectors(from);
    let (f1, r1, u1) = gsc_t6::math::angle_vectors(to);
    std::array::from_fn(|i| std::array::from_fn(|j| f1[i] * f0[j] + r1[i] * r0[j] + u1[i] * u0[j]))
}

/// A look-at box trigger (every wall buy, the box, the craft tables): BO2
/// offers it when the player's body touches the turned box and he faces its
/// centre (dot 0.76). No use radius and no line of sight. The body is the
/// upright line through the middle of his box, 15 in from top and bottom,
/// grown 15 all round; the box grows 15 each way to meet it. Our old test
/// asked his view to land on the gun's own box within reach, so a chalk
/// whose box came out thin never offered (a tester, Bus Depot shotgun).
fn box_use_reach(e: &Ent, mins: [f32; 3], maxs: [f32; 3], eye: [f32; 3], fwd: [f32; 3]) -> bool {
    let Some([w, l, h]) = e.box_dims else {
        return false;
    };
    let r = (maxs[0] - mins[0]).min(maxs[1] - mins[1]) * 0.5;
    let mid: [f32; 3] = std::array::from_fn(|i| (mins[i] + maxs[i]) * 0.5);
    let reach = ((maxs[2] - mins[2]) * 0.5 - r).max(0.0);
    let (f, rt, u) = gsc_t6::math::angle_vectors(e.angles);
    let rel = gsc_t6::math::sub(mid, e.origin);
    // The body line's middle and half length in the box's own axes.
    let m = [
        gsc_t6::math::dot(rel, f),
        gsc_t6::math::dot(rel, rt),
        gsc_t6::math::dot(rel, u),
    ];
    let d = [f[2] * reach, rt[2] * reach, u[2] * reach];
    let half = [w * 0.5 + r, l * 0.5 + r, h * 0.5 + r];
    for i in 0..3 {
        if m[i].abs() > half[i] + d[i].abs() {
            return false;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/triggers.rs
        }
    }
    for (a, b) in [(1, 2), (2, 0), (0, 1)] {
        if (m[a] * d[b] - m[b] * d[a]).abs() > half[a] * d[b].abs() + half[b] * d[a].abs() {
            return false;
        }
    }
    if !e.look_at {
        return true;
    }
    let v = gsc_t6::math::sub(e.origin, eye);
    let len = gsc_t6::math::dot(v, v).sqrt();
    len < 1e-3 || gsc_t6::math::dot(v, fwd) / len >= 0.76
}

/// Zombies use a `trigger_radius_use` like any use entity, without touching
/// it: within the use reach of its centre from the eye (BO1 SP
/// Player_FindUsableEntities: 72; BO2's _zm_unitrigger raises
/// player_useRadius_zm to its largest stub radius instead), and facing it
/// (dot 0.76) only when it asks for a look. Buried's head chopper blade on
/// the courthouse shelf: its trigger starts above the player's head.
fn radius_use_reach(world: &mut World, e: &Ent, eye: [f32; 3], fwd: [f32; 3], reach: f32) -> bool {
    if e.classname != "trigger_radius_use" {
        return false;
    }
    let v = gsc_t6::math::sub(center(world, e), eye);
    let d = gsc_t6::math::dot(v, v).sqrt();
    d <= reach && (!e.look_at || d < 1e-3 || gsc_t6::math::dot(v, fwd) / d >= 0.76)
}

pub(super) fn center(world: &mut World, e: &Ent) -> [f32; 3] {
    if e.box_dims.is_some() {
        return e.origin;
    }
    if cylinder(e).is_some() {
        return [e.origin[0], e.origin[1], e.origin[2] + e.height * 0.5];
    }
    let f = frame(world);
    let model = brush_index(e).and_then(|n| {
        f.clip_cmodels()
            .models
            .get(n as usize)
            .map(|m| (m.mins, m.maxs))
    });
    match model {
        Some((lo, hi)) => std::array::from_fn(|i| e.origin[i] + (lo[i] + hi[i]) * 0.5),
        None => e.origin,
    }
}

/// A brush entity's clip model bounds, as the map stores them (the
/// touch log: Die Rise's elevator volumes against Who's Who's body).
pub(super) fn brush_bounds(world: &mut World, e: &Ent) -> Option<([f32; 3], [f32; 3])> {
    let f = frame(world);
    let n = brush_index(e)?;
    f.clip_cmodels()
        .models
        .get(n as usize)
        .map(|m| (m.mins, m.maxs))
}

pub(super) fn player_box(world: &mut World, client: u32) -> Option<([f32; 3], [f32; 3])> {
    let o = frame(world).player(ClientId(client))?.origin;
    Some((
        std::array::from_fn(|i| o[i] + PLAYER_MINS[i]),
        std::array::from_fn(|i| o[i] + PLAYER_MAXS[i]),
    ))
}

/// Fire the frame's triggers.
pub(super) fn dispatch(world: &mut World) {
    let players: Vec<(u32, ObjRef)> = world
        .resource::<Zm>()
        .players
        .iter()
        .filter(|(_, p)| p.sessionstate == "playing")
        .map(|(c, p)| (*c, p.obj))
        .collect();
    let triggers: Vec<(u32, Ent, Kind)> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter_map(|(n, e)| kind(&e.classname).map(|k| (*n, e.clone(), k)))
        .filter(|(_, e, _)| e.obj.is_some() && !e.trigger_off)
        .collect();
    // The use reach: the scripts raise player_useRadius_zm to their
    // largest trigger's (64 on Nuketown).
    let use_radius = with_vm(world, |vm, _| {
        vm.dvars
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("player_useRadius_zm"))
            .and_then(|(_, v)| v.parse::<f32>().ok())
    })
    .flatten()
    .unwrap_or(64.0)
    .max(1.0);
    // bo2mp: a use trigger only its claimant (`clientclaimtrigger`) or its
    // team (`setteamfortrigger`) may use: a turret's pick-up is its owner's.
    // (trigger entity, claimant client, team) and each player's team.
    let mp = world.resource::<Zm>().mp;
    let (claims, teams): (Vec<(u32, Option<u32>, Option<String>)>, Vec<(u32, String)>) = if !mp {
        Default::default()
    } else {
        with_vm(world, |vm, world| {
            let (by, team) = (vm.intern("claimedby"), vm.intern("triggerteam"));
            let pteam = vm.intern("team");
            let claims = triggers
                .iter()
                .filter(|(_, _, k)| *k == Kind::Use)
                .filter_map(|(n, e, _)| {
                    let o = e.obj?;
                    let claimant = super::entnum(vm, &vm.raw_field(o, by));
                    let t = match vm.raw_field(o, team) {
                        Value::Undefined => None,
                        v => Some(vm.to_text(&v)),
                    };
                    (claimant.is_some() || t.is_some()).then_some((*n, claimant, t))
                })
                .collect();
            let teams = world
                .resource::<Zm>()
                .players
                .iter()
                .map(|(c, p)| (*c, vm.to_text(&vm.raw_field(p.obj, pteam))))
                .collect();
            (claims, teams)
        })
        .unwrap_or_default()
    };
    let playing: Vec<u32> = players.iter().map(|(c, _)| *c).collect();
    let mut fire: Vec<(ObjRef, ObjRef)> = Vec::new();
    for (client, pobj) in players {
        let Some((mins, maxs)) = player_box(world, client) else {
            continue;
        };
        let held = crate::script_player::buttons(&mut frame(world), ClientId(client))
            & (playerstate_iw4::buttons::USE | playerstate_iw4::buttons::USE_RELOAD)
            != 0;
        let pressed = {
            let mut zm = world.resource_mut::<Zm>();
            let was = zm.use_held.contains(&client);
            if held {
                zm.use_held.insert(client);
            } else {
                zm.use_held.remove(&client);
            }
            held && !was
        };
        let (eye, fwd) = {
            let f = frame(world);
            let ps = f.player(ClientId(client));
            let eye = ps.map_or([0.0; 3], |p| {
                [
                    p.origin[0],
                    p.origin[1],
                    p.origin[2] + p.view_height_current,
                ]
            });
            let fwd = gsc_t6::math::angle_vectors(ps.map_or([0.0; 3], |p| p.viewangles)).0;
            (eye, fwd)
        };
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/triggers.rs
        // In a vehicle he has no use hint and uses nothing (BO2's
        // Player_UpdateCursorHints and the use scan both stop while
        // eFlags 0x4000 is set).
        let in_vehicle = super::killstreaks::riding(world, client);
        let mut best: Option<(f32, ObjRef, Option<String>)> = None;
        let my_team = teams
            .iter()
            .find(|(c, _)| *c == client)
            .map(|(_, t)| t.as_str());
        for (n, e, k) in &triggers {
            if in_vehicle && *k == Kind::Use {
                continue;
            }
            // (A claim by someone no longer playing holds nothing.)
            if let Some((_, claimant, team)) = claims.iter().find(|(t, _, _)| t == n)
                && (claimant.is_some_and(|c| c != client && playing.contains(&c))
                    || team.as_deref().is_some_and(|t| Some(t) != my_team))
            {
                continue;
            }
=======
        // The faced trigger: (score, trigger, hint, low priority). BO2 offers a
        // low priority one (`sethintlowpriority`: every buyable door) only
        // when no other is there to use; among equals, the best faced.
        let mut best: Option<(f32, ObjRef, Option<String>, bool)> = None;
        let beats = |score: f32, low: bool, best: &Option<(f32, ObjRef, Option<String>, bool)>| {
            hint_beats(score, low, best.as_ref().map(|b| (b.0, b.3)))
        };
        for (_, e, k) in &triggers {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/triggers.rs
            if e.invisible_to.contains(&client) || e.invisible_to_all {
                continue;
            }
            let looked = *k == Kind::Use && e.look_at && e.box_dims.is_some();
            if looked {
                if !box_use_reach(e, mins, maxs, eye, fwd) {
                    continue;
                }
            } else if !touches(world, e, mins, maxs)
                && !radius_use_reach(world, e, eye, fwd, use_radius)
            {
                continue;
            }
            let obj = e.obj.unwrap();
            match k {
                Kind::Touch => {
                    fire.push((obj, pobj));
                    // BO2 (zombies) puts a touched trigger_radius or
                    // trigger_box that has a hint string on the use list, so
                    // its hint shows (a window's "Rebuild Barrier"); with
                    // look-at required, only within a 0.76 view dot.
                    if matches!(e.classname.as_str(), "trigger_radius" | "trigger_box")
                        && e.hint.is_some()
                        && touches(world, e, mins, maxs)
                    {
                        let c = center(world, e);
                        let d = gsc_t6::math::normalize(gsc_t6::math::sub(c, eye));
                        let score = gsc_t6::math::dot(d, fwd);
                        if (!e.look_at || score >= 0.76) && beats(score, e.hint_low, &best) {
                            best = Some((score, obj, e.hint.clone(), e.hint_low));
                        }
                    }
                }
                Kind::Use => {
                    let c = center(world, e);
                    let d = gsc_t6::math::normalize(gsc_t6::math::sub(c, eye));
                    let score = gsc_t6::math::dot(d, fwd);
                    if beats(score, e.hint_low, &best) {
                        best = Some((score, obj, e.hint.clone(), e.hint_low));
                    }
                }
            }
        }
        {
            let mut zm = world.resource_mut::<Zm>();
            // IW4L_T6_HINTLOG=1: each change of the faced trigger's hint and
            // the trigger it came from.
            if std::env::var_os("IW4L_T6_HINTLOG").is_some() {
                let now_hint = best.as_ref().and_then(|b| b.2.clone());
                if zm.hints.get(&client) != now_hint.as_ref() {
                    let from = best.as_ref().and_then(|b| {
                        triggers
                            .iter()
                            .find(|(_, e, _)| e.obj == Some(b.1))
                            .map(|(n, e, _)| {
                                format!(
                                    "ent{n} {} at ({:.0} {:.0} {:.0}) angles {:?} box {:?}",
                                    e.classname,
                                    e.origin[0],
                                    e.origin[1],
                                    e.origin[2],
                                    e.angles,
                                    e.box_dims
                                )
                            })
                    });
                    diag::info!(
                        Sim,
                        "bo2zm t6 hint for {client}: {now_hint:?} from {from:?}"
                    );
                }
            }
            match &best {
                Some((_, _, Some(h), _)) => {
                    zm.hints.insert(client, h.clone());
                }
                _ => {
                    zm.hints.remove(&client);
                }
            }
        }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/triggers.rs
        // bo2mp: else a usable model's (a care package's capture prompt).
        if !in_vehicle
            && best.as_ref().is_none_or(|b| b.2.is_none())
            && let Some(origin) = frame(world).player(ClientId(client)).map(|ps| ps.origin)
            && let Some(h) = super::killstreaks::usable_hint(world, origin)
        {
            world.resource_mut::<Zm>().hints.insert(client, h);
        }
        if pressed && let Some((_, obj, _)) = best {
=======
        if pressed && let Some((_, obj, _, _)) = best {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/triggers.rs
            if std::env::var("IW4L_T6_TRIGLOG").is_ok() {
                diag::info!(Sim, "bo2zm t6 use by {client} on {obj:?}");
            }
            fire.push((obj, pobj));
        }
    }
    actor_touches(world, &triggers, &mut fire);
    if fire.is_empty() {
        return;
    }
    with_vm(world, |vm, world| {
        if let Ok(names) = std::env::var("IW4L_T6_FIRELOG") {
            fire_log(vm, world, &triggers, &fire, &names);
        }
        for (t, p) in fire {
            if vm.alive(t) && !super::bo1_maps::cabinet_used(vm, world, t, p) {
                vm.notify_str(world, t, "trigger", &[Value::Object(p)]);
            }
        }
    });
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/triggers.rs
=======
/// IW4L_T6_FIRELOG=<targetname>[,...]: each trigger with that name that
/// fires, which entity it is, its volume and who set it off (at most once a
/// second per pair; Die Rise's squash triggers refuse Who's Who's body).
fn fire_log(
    vm: &mut gsc_t6::Vm<World>,
    world: &mut World,
    triggers: &[(u32, Ent, Kind)],
    fire: &[(ObjRef, ObjRef)],
    names: &str,
) {
    static LAST: std::sync::Mutex<Vec<(String, std::time::Instant)>> =
        std::sync::Mutex::new(Vec::new());
    let (tn, org) = (vm.intern("targetname"), vm.intern("origin"));
    for (t, p) in fire {
        let name = vm.to_text(&vm.raw_field(*t, tn));
        if !names.split(',').any(|n| n == name) {
            continue;
        }
        let key = format!("{t:?}{p:?}");
        {
            let Ok(mut last) = LAST.lock() else { return };
            match last.iter_mut().find(|(k, _)| *k == key) {
                Some((_, at)) if at.elapsed().as_secs_f32() < 1.0 => continue,
                Some((_, at)) => *at = std::time::Instant::now(),
                None => last.push((key, std::time::Instant::now())),
            }
        }
        let who = vm.get_field(world, *p, org);
        let Some((n, e, _)) = triggers.iter().find(|(_, e, _)| e.obj == Some(*t)) else {
            continue;
        };
        diag::info!(
            Sim,
            "bo2zm t6 fire {name}: ent{n} {} {:?} at {:?} box {:?} radius {} by {p:?} at {}",
            e.classname,
            e.brush,
            e.origin,
            e.box_dims,
            e.radius,
            vm.to_text(&who)
        );
    }
}

/// Touch triggers whose spawnflags take AI (1 axis, 2 allies, 4 neutral)
/// fire for each living actor of that team inside (Verrückt's electric
/// traps: zombies walking through the live `trigger_multiple` die).
fn actor_touches(
    world: &mut World,
    triggers: &[(u32, Ent, Kind)],
    fire: &mut Vec<(ObjRef, ObjRef)>,
) {
    let ai: Vec<(Ent, i32)> = with_vm(world, |vm, world| {
        let sf = vm.intern("spawnflags");
        triggers
            .iter()
            .filter(|(_, _, k)| *k == Kind::Touch)
            .filter_map(|(_, e, _)| match vm.get_field(world, e.obj?, sf) {
                // BO2 (G_DoTouchTriggers): any AI flag lets every zombie
                // in; a team the scripts set then picks (the claymore's
                // flag 4 trigger ignores the owner's side).
                Value::Int(f) if f & 7 != 0 && e.trigger_team.is_some() => Some((e.clone(), 7)),
                Value::Int(f) if f & 7 != 0 => Some((e.clone(), f)),
                _ => None,
            })
            .collect()
    })
    .unwrap_or_default();
    if ai.is_empty() {
        return;
    }
    let actors: Vec<(ObjRef, i32, String, [f32; 3], [f32; 3])> = {
        let zm = world.resource::<Zm>();
        zm.actors
            .iter()
            .filter(|(_, a)| a.alive)
            .filter_map(|(n, a)| {
                let p = zm.ents.get(n)?.origin;
                let team = match a.team.as_str() {
                    "axis" => 1,
                    "allies" => 2,
                    _ => 4,
                };
                Some((
                    a.obj?,
                    team,
                    a.team.clone(),
                    [p[0] - a.radius, p[1] - a.radius, p[2]],
                    [p[0] + a.radius, p[1] + a.radius, p[2] + a.height],
                ))
            })
            .collect()
    };
    for (e, flags) in &ai {
        for (obj, team, name, mins, maxs) in &actors {
            let side = match &e.trigger_team {
                Some((t, exclude)) => (t == name) != *exclude,
                None => true,
            };
            if flags & team != 0 && side && touches(world, e, *mins, *maxs) {
                fire.push((e.obj.unwrap(), *obj));
            }
        }
    }
}

/// How a `trigger_damage` was hit: its spawnflags turn each kind away
/// (1 pistol, 2 rifle, 4 projectile, 8 explosion, 16 splash, 32 melee).
#[derive(Clone, Copy)]
pub(crate) enum DamageKind {
    Bullet,
    Explosion,
    Melee,
    /// A flying projectile's path this tick (the staffs' shots): only the
    /// damageable models it goes through, not zombies or triggers.
    Projectile,
    /// A shot that lands without going off (the ballistic knife's blade):
    /// BO2 damages every `trigger_damage` its last stretch crosses with
    /// MOD_IMPACT, which no spawnflag turns away, and only triggers.
    Impact,
}

/// A shot, a knife or a blast through a `trigger_damage` brush fires it:
/// "damage" then "trigger" (Nuketown's bunker hatch plays Marlton when
/// knifed). `start..end` is the shot's line (to its first hit), or the
/// blast's centre to itself with `radius`.
pub(crate) fn damage_triggers(
    world: &mut World,
    start: [f32; 3],
    end: [f32; 3],
    radius: f32,
    amount: i32,
    attacker: ClientId,
    weapon: u32,
    how: DamageKind,
) {
    damage_triggers_but(world, start, end, radius, amount, attacker, weapon, how, &[]);
}

/// `damage_triggers` passing over `skip` (what this projectile already
/// went through); returns what it damaged.
#[allow(clippy::too_many_arguments)]
pub(crate) fn damage_triggers_but(
    world: &mut World,
    start: [f32; 3],
    end: [f32; 3],
    radius: f32,
    amount: i32,
    attacker: ClientId,
    weapon: u32,
    how: DamageKind,
    skip: &[ObjRef],
) -> Vec<ObjRef> {
    if !world.contains_resource::<super::T6Runtime>() {
        return Vec::new();
    }
    let projectile = matches!(how, DamageKind::Projectile);
    let impact = matches!(how, DamageKind::Impact);
    let cands: Vec<Ent> = {
        let zm = world.resource::<Zm>();
        zm.ents
            .iter()
            .filter(|(n, e)| {
                let trigger = e.classname == "trigger_damage" && !e.trigger_off;
                e.obj.is_some_and(|o| !skip.contains(&o))
                    && if projectile {
                        damageable_model(e) && !zm.actors.contains_key(n)
                    } else if impact {
                        trigger
                    } else {
                        trigger || damageable_model(e)
                    }
            })
            .map(|(_, e)| e.clone())
            .collect()
    };
    if cands.is_empty() {
        return Vec::new();
    }
    let w = super::weapon_text(world, weapon);
    let block = match how {
        DamageKind::Melee => 32,
        DamageKind::Explosion => 8 | 16,
        DamageKind::Projectile => 4,
        DamageKind::Impact => 0,
        DamageKind::Bullet => {
            let pistol = [
                "m1911",
                "python",
                "fiveseven",
                "judge",
                "kard",
                "beretta93r",
                "c96",
                "rnma",
            ]
            .iter()
            .any(|p| w.starts_with(p));
            if pistol { 1 } else { 2 }
        }
    };
    // A knife names the knife the scripts gave him, as a knifed zombie does
    // (`entity_hit`): Buried's tunnel signs light only for the Bowie Knife or
    // the Galvaknuckles (`zm_buried_sq_ts`).
    let w = if matches!(how, DamageKind::Melee) {
        world
            .resource::<Zm>()
            .players
            .get(&attacker.0)
            .and_then(|p| p.melee_weapon.clone())
            .unwrap_or_else(|| "knife_zm".to_owned())
    } else {
        w
    };
    let len = gsc_t6::math::length(gsc_t6::math::sub(end, start));
    let steps = ((len / 4.0).ceil() as usize).clamp(1, 2048);
    let mut hit = Vec::new();
    let mut boxed: Vec<(ObjRef, [f32; 3], [f32; 3], usize)> = Vec::new();
    for e in &cands {
        // A model the scripts made damageable (`setcandamage`): its model's
        // box (Mob's Afterlife shock boxes, zapped with the lightning hands).
        if damageable_model(e) && e.brush.is_none() {
            let Some((lo, hi)) = super::brushes::model_box(world, e) else {
                continue;
            };
            let near =
                |at: [f32; 3], r: f32| (0..3).all(|k| at[k] >= lo[k] - r && at[k] <= hi[k] + r);
            let first = match how {
                DamageKind::Explosion => near(start, radius).then_some(0),
                _ => (0..=steps).position(|i| {
                    let t = i as f32 / steps as f32;
                    near(
                        std::array::from_fn(|k| start[k] + (end[k] - start[k]) * t),
                        2.0,
                    )
                }),
            };
            let touched = first.is_some();
            if std::env::var_os("IW4L_T6_HITLOG").is_some() {
                diag::info!(
                    Sim,
                    "bo2zm t6 damageable {} box {lo:.0?}-{hi:.0?} shot {start:.0?}->{end:.0?} touched {touched}",
                    e.model
                );
            }
            if let Some(first) = first {
                hit.push(e.obj.unwrap());
                boxed.push((e.obj.unwrap(), lo, hi, first));
            }
            continue;
        }
        let touched = match how {
            DamageKind::Explosion => {
                let c = center(world, e);
                gsc_t6::math::length(gsc_t6::math::sub(c, start)) <= radius
                    || touches(
                        world,
                        e,
                        start.map(|v| v - radius * 0.5),
                        start.map(|v| v + radius * 0.5),
                    )
            }
            _ => (0..=steps).any(|i| {
                let t = i as f32 / steps as f32;
                let at: [f32; 3] = std::array::from_fn(|k| start[k] + (end[k] - start[k]) * t);
                touches(world, e, at.map(|v| v - 2.0), at.map(|v| v + 2.0))
            }),
        };
        if std::env::var_os("IW4L_T6_HITLOG").is_some() && e.classname != "trigger_damage" {
            diag::info!(
                Sim,
                "bo2zm t6 damageable brush {} {} at {:.0?} shot {start:.0?}->{end:.0?} touched {touched}",
                e.classname,
                e.model,
                e.origin
            );
        }
        if touched {
            hit.push(e.obj.unwrap());
        }
    }
    // Models whose boxes nest (Origins' wind rings, one inside the other
    // on the chamber ceiling): a shot meets the ring whose band it goes
    // through, so the outer box does not count where the shot was already
    // inside an inner one when it reached it.
    if !matches!(how, DamageKind::Explosion) {
        let inside = |a: &([f32; 3], [f32; 3]), b: &([f32; 3], [f32; 3])| {
            a != b && (0..3).all(|k| b.0[k] >= a.0[k] && b.1[k] <= a.1[k])
        };
        let drop: Vec<ObjRef> = boxed
            .iter()
            .filter(|(_, lo, hi, first)| {
                boxed
                    .iter()
                    .any(|(_, l, h, f)| f <= first && inside(&(*lo, *hi), &(*l, *h)))
            })
            .map(|b| b.0)
            .collect();
        hit.retain(|o| !drop.contains(o));
    }
    if hit.is_empty() {
        return Vec::new();
    }
    let done = hit.clone();
    let pobj = world
        .resource::<Zm>()
        .players
        .get(&attacker.0)
        .map(|p| p.obj);
    let means = match how {
        DamageKind::Melee => "MOD_MELEE",
        DamageKind::Explosion => "MOD_GRENADE_SPLASH",
        DamageKind::Projectile => "MOD_PROJECTILE",
        DamageKind::Impact => "MOD_IMPACT",
        DamageKind::Bullet => {
            if block == 1 {
                "MOD_PISTOL_BULLET"
            } else {
                "MOD_RIFLE_BULLET"
            }
        }
    };
    let models: Vec<ObjRef> = cands
        .iter()
        .filter(|e| e.classname != "trigger_damage")
        .filter_map(|e| e.obj)
        .collect();
    let vehicles: Vec<ObjRef> = cands
        .iter()
        .filter(|e| e.classname == "script_vehicle")
        .filter_map(|e| e.obj)
        .collect();
    with_vm(world, |vm, world| {
        let sf = vm.intern("spawnflags");
        let cb = vm.intern("callbackvehicledamage");
        let vehicle_cb = !matches!(vm.get_field(world, vm.level, cb), Value::Undefined);
        for t in hit {
            if !vm.alive(t) {
                continue;
            }
            let model = models.contains(&t);
            let flags = match vm.get_field(world, t, sf) {
                Value::Int(i) if !model => i,
                _ => 0,
            };
            if flags & block != 0 {
                continue;
            }
            if std::env::var("IW4L_T6_TRIGLOG").is_ok() {
                diag::info!(
                    Sim,
                    "bo2zm t6 damage trigger {t:?} by {attacker:?} {means} {amount}"
                );
            }
            let att = pobj.map_or(Value::Undefined, Value::Object);
            let dir = Value::Vec3(gsc_t6::math::normalize(gsc_t6::math::sub(end, start)));
            // BO2's landing hit sends five: no point (the origin), no tag.
            if impact {
                let args = [
                    Value::Int(amount),
                    att.clone(),
                    dir,
                    Value::Vec3([0.0; 3]),
                    vm.string(means),
                ];
                vm.notify_str(world, t, "damage", &args);
                if vm.alive(t) {
                    vm.notify_str(world, t, "trigger", &[att]);
                }
                continue;
            }
            let args = [
                Value::Int(amount),
                att.clone(),
                dir,
                Value::Vec3(end),
                vm.string(means),
                vm.string(""),
                vm.string(""),
                vm.string(""),
                vm.string(&w),
                Value::Int(0),
            ];
            // A vehicle goes through the code callback too (Origins' fire
            // staff biplane comes down in `level.callbackvehicledamage`).
            if vehicle_cb && vehicles.contains(&t) {
                let cargs = vec![
                    att.clone(),
                    att.clone(),
                    Value::Int(amount),
                    Value::Int(0),
                    vm.string(means),
                    vm.string(&w),
                    Value::Vec3(end),
                    args[2].clone(),
                    vm.string("none"),
                    Value::Int(0),
                    Value::Int(0),
                    Value::Int(0),
                    vm.string(""),
                ];
                vm.spawn_named(
                    world,
                    "maps/mp/gametypes_zm/_callbacksetup",
                    "codecallback_vehicledamage",
                    Value::Object(t),
                    cargs,
                );
            }
            vm.notify_str(world, t, "damage", &args);
            if !model && vm.alive(t) {
                vm.notify_str(world, t, "trigger", &[att]);
            }
        }
    });
    done
}

/// A shown (or ghosted) model entity the scripts made damageable
/// (`setcandamage`).
fn damageable_model(e: &Ent) -> bool {
    // A script brush ("*N", kept in `brush`, no model name) counts too:
    // Origins' ghosted wind smoke detectors (air_puzzle_run_smoke_direction).
    e.can_damage
        && (!e.hidden || e.ghosted)
        && (!e.model.is_empty() || e.brush.is_some())
        && !e.classname.starts_with("trigger")
}

#[allow(dead_code)]
fn _volume_type(_: Volume<'_>) {}

#[cfg(test)]
mod tests {
    use super::*;

    /// A long thin brush trigger along +x, turned a quarter to the left
    /// (yaw 90), lies along +y: a box there touches it, one on the old
    /// spot does not.
    #[test]
    fn a_turned_brush_trigger_turns_its_brushes() {
        let brush = crate::SimBrush {
            planes: vec![
                [1.0, 0.0, 0.0, 100.0],
                [-1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 10.0],
                [0.0, -1.0, 0.0, 10.0],
                [0.0, 0.0, 1.0, 10.0],
                [0.0, 0.0, -1.0, 0.0],
            ],
            contents: 0,
            plane_surface_flags: Vec::new(),
            glass_encoded: 0,
        };
        let brushes = [brush];
        let v = Volume::Brushes {
            origin: [0.0; 3],
            ids: &[0],
            brushes: &brushes,
        };
        let at = |x: f32, y: f32| ([x - 1.0, y - 1.0, 4.0], [x + 1.0, y + 1.0, 6.0]);
        let turn = turn_between([0.0; 3], [0.0, 90.0, 0.0]);
        let (mins, maxs) = at(50.0, 0.0);
        assert!(v.touches(mins, maxs));
        assert!(!v.touches_turned(mins, maxs, turn));
        let (mins, maxs) = at(0.0, 50.0);
        assert!(!v.touches(mins, maxs));
        assert!(v.touches_turned(mins, maxs, turn));
        // No turn is the plain test.
        let none = turn_between([0.0, 30.0, 0.0], [0.0, 30.0, 0.0]);
        let (mins, maxs) = at(50.0, 0.0);
        assert!(v.touches_turned(mins, maxs, none));
    }

    /// A buyable door's hint (low priority) yields to a window's even when
    /// he faces the door better; between two of a kind the better faced wins.
    #[test]
    fn a_low_priority_hint_yields() {
        assert!(hint_beats(0.2, false, Some((0.9, true))));
        assert!(!hint_beats(0.9, true, Some((0.2, false))));
        assert!(hint_beats(0.9, true, Some((0.2, true))));
        assert!(!hint_beats(0.2, false, Some((0.9, false))));
        assert!(hint_beats(0.1, true, None));
    }
}
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/triggers.rs
