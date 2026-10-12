//! Death barriers: the map's `trigger_hurt` volumes hurt (and mostly kill)
//! the players inside them, as the engine's `hurt_touch` does, and a fall
//! kill plane below the map's lowest collision ends a fall off the map that
//! would otherwise never land.
//!
//! `trigger_hurt` (the Quake III / RTCW lineage every IW and Treyarch engine
//! keeps, `SP_trigger_hurt` / `hurt_touch`):
//! - `dmg` per touch, 5 when the map leaves it 0;
//! - a touch hurts once per server frame (50 ms), or once a second with
//!   spawnflag SLOW (16);
//! - spawnflag NO_PROTECTION (8) passes the scripts' `level.idflags_no_protection`;
//! - the hit is `G_Damage(player, self, self, ..., MOD_TRIGGER_HURT)`: the
//!   trigger itself is the inflictor and attacker (BO2's
//!   `callback_playerdamage` reads `einflictor.script_noteworthy ==
//!   "ragdoll_now"` off it);
//! - START_OFF (1) is logged, not honoured (`HURT_HONOUR_START_OFF`): the
//!   IW/Treyarch scripts toggle hazards with `trigger_off` / `trigger_on`
//!   (moving the trigger 10000 down), which the volume test follows.
//!
//! BO3's `trigger_out_of_bounds` classname is treated as an instant kill
//! should a map carry one. Triggers whose names merely suggest "out of
//! bounds" (a `trigger_multiple` named `oob`...) are listed in the log but
//! left to the map's scripts, which own what their `trigger` notify means.
//!
//! The fall kill plane: no engine plane exists in BO2; its maps close the
//! void with `trigger_hurt`. Maps from other games don't always (and their
//! own scripts aren't here), so a plane is derived per map from its
//! collision: the lowest point of any collision (world model bounds, clip
//! brushes' bottoms, the clip mesh, static models) minus a margin, and
//! never above the lowest spawn point minus the same margin. Nothing can be
//! stood on below it, so it cannot cut playable space.

use std::collections::BTreeMap;

use bevy_ecs::prelude::{Resource, World};
use gsc_t6::{ObjRef, Value};

use super::{Ent, Zm, frame, natives_ai, triggers, with_vm};
use crate::world::ClientId;

/// `SP_trigger_hurt`: a 0 `dmg` hurts 5.
pub(crate) const HURT_DEFAULT_DMG: i32 = 5;
/// `hurt_touch`: once per server frame...
pub(crate) const HURT_INTERVAL_MS: i64 = crate::MATCH_TICK_MS as i64;
/// ...or once a second with spawnflag SLOW.
pub(crate) const HURT_SLOW_INTERVAL_MS: i64 = 1000;
pub(crate) const HURT_SPAWNFLAG_START_OFF: i32 = 1;
pub(crate) const HURT_SPAWNFLAG_NO_PROTECTION: i32 = 8;
pub(crate) const HURT_SPAWNFLAG_SLOW: i32 = 16;
/// Off: a START_OFF `trigger_hurt` still hurts (see the module notes).
pub(crate) const HURT_HONOUR_START_OFF: bool = false;
/// A hurt trigger's hit is logged from this much (a lethal-sized one).
const HURT_LOG_DAMAGE: i32 = 100;
/// Damage a kill plane / out-of-bounds trigger deals: more than any health.
pub(crate) const KILL_DAMAGE: i32 = 100_000;
/// The kill plane sits this far (at least) below the lowest collision...
pub(crate) const KILL_PLANE_MARGIN_MIN: f32 = 512.0;
/// ...or this share of the map's collision height, if more (a tall map
/// gets a longer visible fall before the plane), capped.
pub(crate) const KILL_PLANE_MARGIN_SPAN_FRACTION: f32 = 0.1;
pub(crate) const KILL_PLANE_MARGIN_MAX: f32 = 2048.0;
/// Dvar `bo2mp_killplane` 0 turns the fall kill plane off.
pub(crate) const KILL_PLANE_DVAR: &str = "bo2mp_killplane";

/// `next_hurt`'s key for the kill plane (no entity has this number).
const KILL_PLANE_KEY: u32 = u32::MAX;

/// Per-map barrier state (outlives a round restart's `Zm`).
#[derive(Resource, Default)]
pub(crate) struct Barriers {
    map: String,
    planned: bool,
    kill_z: Option<f32>,
    /// (trigger entity, client) -> when it may hurt again.
    next_hurt: BTreeMap<(u32, u32), i64>,
    /// Trigger entities already described in the log.
    reported: std::collections::BTreeSet<(String, u32)>,
    /// The map's barrier summary is logged.
    summarized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Barrier {
    Hurt,
    Kill,
}

fn barrier(classname: &str) -> Option<Barrier> {
    match classname {
        "trigger_hurt" => Some(Barrier::Hurt),
        "trigger_out_of_bounds" => Some(Barrier::Kill),
        _ => None,
    }
}

fn int_key(vm: &mut gsc_t6::Vm<World>, obj: ObjRef, key: &str) -> i32 {
    let f = vm.intern(key);
    match vm.raw_field(obj, f) {
        Value::Int(i) => i,
        Value::Float(x) => x as i32,
        _ => 0,
    }
}

fn text_key(vm: &mut gsc_t6::Vm<World>, obj: ObjRef, key: &str) -> String {
    let f = vm.intern(key);
    match vm.raw_field(obj, f) {
        Value::Undefined => String::new(),
        v => vm.to_text(&v),
    }
}

/// The damage one touch of a hurt trigger deals (`dmg`, 5 when 0).
pub(crate) fn hurt_damage(dmg: i32) -> i32 {
    if dmg == 0 { HURT_DEFAULT_DMG } else { dmg }
}

/// How long after a touch the trigger hurts the same player again.
pub(crate) fn hurt_interval(spawnflags: i32) -> i64 {
    if spawnflags & HURT_SPAWNFLAG_SLOW != 0 {
        HURT_SLOW_INTERVAL_MS
    } else {
        HURT_INTERVAL_MS
    }
}

/// The kill plane from the map's lowest collision, its highest, and its
/// lowest spawn point. None without collision.
pub(crate) fn kill_plane_z(
    lowest_collision: Option<f32>,
    highest_collision: Option<f32>,
    lowest_spawn: Option<f32>,
) -> Option<(f32, f32)> {
    let low = lowest_collision.filter(|z| z.is_finite())?;
    let span = highest_collision
        .filter(|z| z.is_finite() && *z > low)
        .map_or(0.0, |h| h - low);
    let margin = (span * KILL_PLANE_MARGIN_SPAN_FRACTION)
        .clamp(KILL_PLANE_MARGIN_MIN, KILL_PLANE_MARGIN_MAX);
    let mut z = low - margin;
    if let Some(s) = lowest_spawn.filter(|z| z.is_finite()) {
        z = z.min(s - margin);
    }
    Some((z, margin))
}

/// The lowest point a brush's axial bottom plane allows (a plane
/// `n.x <= d` with `n = (0, 0, -1)` holds `z >= -d`).
fn brush_bottom(planes: &[[f32; 4]]) -> Option<f32> {
    planes
        .iter()
        .filter(|p| p[2] < -0.999 && p[0].abs() < 1e-3 && p[1].abs() < 1e-3)
        .map(|p| -p[3] / -p[2])
        .reduce(f32::max)
}

fn brush_top(planes: &[[f32; 4]]) -> Option<f32> {
    planes
        .iter()
        .filter(|p| p[2] > 0.999 && p[0].abs() < 1e-3 && p[1].abs() < 1e-3)
        .map(|p| p[3] / p[2])
        .reduce(f32::min)
}

fn plan(world: &mut World) {
    let map = world.resource::<Zm>().map.clone();
    let lowest_spawn = world
        .resource::<Zm>()
        .ents
        .values()
        .filter(|e| is_spawn_point(&e.classname))
        .map(|e| e.origin[2])
        .reduce(f32::min);
    let spawns = world
        .resource::<Zm>()
        .ents
        .values()
        .filter(|e| is_spawn_point(&e.classname))
        .count();
    let (sources, low, high) = {
        let f = frame(world);
        let mut sources: Vec<(&str, f32, f32)> = Vec::new();
        if let Some(m) = f.clip_cmodels().models.first()
            && m.mins[2].is_finite()
            && m.maxs[2] > m.mins[2]
        {
            sources.push(("world model", m.mins[2], m.maxs[2]));
        }
        let brushes = f.clip_brushes();
        let lo = brushes
            .iter()
            .filter_map(|b| brush_bottom(&b.planes))
            .reduce(f32::min);
        let hi = brushes
            .iter()
            .filter_map(|b| brush_top(&b.planes))
            .reduce(f32::max);
        if let (Some(lo), Some(hi)) = (lo, hi) {
            sources.push(("clip brushes", lo, hi));
        }
        let verts = &f.clip_mesh().tables.verts;
        let lo = verts.iter().map(|v| v[2]).reduce(f32::min);
        let hi = verts.iter().map(|v| v[2]).reduce(f32::max);
        if let (Some(lo), Some(hi)) = (lo, hi) {
            sources.push(("clip mesh", lo, hi));
        }
        let sm = &f.clip_mesh().static_models;
        let lo = sm
            .iter()
            .map(|s| s.model.bounds_mid[2] - s.model.bounds_half[2])
            .reduce(f32::min);
        let hi = sm
            .iter()
            .map(|s| s.model.bounds_mid[2] + s.model.bounds_half[2])
            .reduce(f32::max);
        if let (Some(lo), Some(hi)) = (lo, hi) {
            sources.push(("static models", lo, hi));
        }
        sources.retain(|(_, lo, hi)| lo.is_finite() && hi.is_finite() && lo.abs() < 1.0e6);
        let low = sources.iter().map(|s| s.1).reduce(f32::min);
        let high = sources.iter().map(|s| s.2).reduce(f32::max);
        let text: Vec<String> = sources
            .iter()
            .map(|(n, lo, hi)| format!("{n} {lo:.0}..{hi:.0}"))
            .collect();
        (text.join(", "), low, high)
    };
    let enabled = with_vm(world, |vm, _| {
        vm.dvars
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(KILL_PLANE_DVAR))
            .is_none_or(|(_, v)| v.trim() != "0")
    })
    .unwrap_or(true);
    let planned = kill_plane_z(low, high, lowest_spawn);
    let kill_z = planned.filter(|_| enabled).map(|(z, _)| z);
    match planned {
        Some((z, margin)) => diag::info!(
            Sim,
            "bo2mp death barriers {map}: fall kill plane z {z:.0}{} (lowest collision {:.0}, \
             margin {margin:.0}; lowest of {spawns} spawn points {}; from {sources})",
            if enabled {
                ""
            } else {
                " OFF (dvar bo2mp_killplane 0)"
            },
            low.unwrap_or(f32::NAN),
            lowest_spawn.map_or("none".to_owned(), |z| format!("{z:.0}")),
        ),
        None => diag::warn!(
            Sim,
            "bo2mp death barriers {map}: no fall kill plane (no collision bounds: {sources})"
        ),
    }
    let mut b = world.resource_mut::<Barriers>();
    b.map = map;
    b.planned = true;
    b.kill_z = kill_z;
}

fn is_spawn_point(classname: &str) -> bool {
    (classname.starts_with("mp_") && classname.contains("_spawn"))
        || classname.starts_with("info_player")
}

fn oob_name(s: &str) -> bool {
    let s = s.to_ascii_lowercase();
    s.contains("out_of_bounds")
        || s.contains("outofbounds")
        || s == "oob"
        || s.starts_with("oob_")
        || s.ends_with("_oob")
        || s.contains("kill_trigger")
        || s.contains("death_trigger")
}

/// Describe the map's barrier triggers once.
fn report(world: &mut World, list: &[(u32, Ent, Barrier)]) {
    let map = world.resource::<Barriers>().map.clone();
    let first = !world.resource::<Barriers>().summarized;
    {
        let b = world.resource::<Barriers>();
        if !first
            && list
                .iter()
                .all(|(n, _, _)| b.reported.contains(&(map.clone(), *n)))
        {
            return;
        }
    }
    world.resource_mut::<Barriers>().summarized = true;
    let named: Vec<(u32, String)> = if !first {
        Vec::new()
    } else {
        with_vm(world, |vm, world| {
            world
                .resource::<Zm>()
                .ents
                .iter()
                .filter(|(_, e)| e.map && triggers::kind(&e.classname).is_some())
                .filter_map(|(n, e)| {
                    let o = e.obj?;
                    let names = [
                        text_key(vm, o, "targetname"),
                        text_key(vm, o, "script_noteworthy"),
                    ];
                    names
                        .iter()
                        .any(|s| oob_name(s))
                        .then(|| (*n, format!("{} {:?}", e.classname, names)))
                })
                .collect()
        })
        .unwrap_or_default()
    };
    let mut lines = Vec::new();
    for (n, e, k) in list {
        if !world
            .resource_mut::<Barriers>()
            .reported
            .insert((map.clone(), *n))
        {
            continue;
        }
        let (dmg, flags, noteworthy) = with_vm(world, |vm, _| {
            let o = e.obj.unwrap();
            (
                int_key(vm, o, "dmg"),
                int_key(vm, o, "spawnflags"),
                text_key(vm, o, "script_noteworthy"),
            )
        })
        .unwrap_or_default();
        let volume = if let Some(b) = triggers::brush_index(e) {
            format!("brush *{b}")
        } else if let Some(h) = triggers::trigger_hull_index(e) {
            format!("trigger hulls ?{h}")
        } else if e.radius > 0.0 {
            format!("cylinder r{} h{}", e.radius, e.height)
        } else {
            "no volume".to_owned()
        };
        let centre = triggers::center(world, e);
        lines.push(format!(
            "#{n} {} {:?} dmg {} spawnflags {flags} {volume} at ({:.0} {:.0} {:.0}){}",
            e.classname,
            k,
            if *k == Barrier::Kill {
                KILL_DAMAGE
            } else {
                hurt_damage(dmg)
            },
            centre[0],
            centre[1],
            centre[2],
            if noteworthy.is_empty() {
                String::new()
            } else {
                format!(" [{noteworthy}]")
            },
        ));
    }
    if first || !lines.is_empty() {
        diag::info!(
            Sim,
            "bo2mp death barriers {map}: {} hurt/oob triggers: {}; out-of-bounds-named script triggers \
             (left to the scripts): {:?}",
            lines.len(),
            lines.join("; "),
            named
        );
    }
}

/// Hurt the players in the map's hurt triggers; kill those below the
/// fall kill plane. Run each authority frame after the triggers.
pub(super) fn advance(world: &mut World, now: i64) {
    if !world.contains_resource::<Barriers>() {
        world.insert_resource(Barriers::default());
    }
    if !world.resource::<Zm>().started {
        return;
    }
    let map = world.resource::<Zm>().map.clone();
    if !world.resource::<Barriers>().planned || world.resource::<Barriers>().map != map {
        *world.resource_mut::<Barriers>() = Barriers::default();
        world.resource_mut::<Barriers>().map = map;
        plan(world);
    }
    let list: Vec<(u32, Ent, Barrier)> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter_map(|(n, e)| barrier(&e.classname).map(|k| (*n, e.clone(), k)))
        .filter(|(_, e, _)| e.obj.is_some() && !e.trigger_off)
        .collect();
    report(world, &list);
    let players: Vec<u32> = world
        .resource::<Zm>()
        .players
        .iter()
        .filter(|(_, p)| p.sessionstate == "playing")
        .map(|(c, _)| *c)
        .collect();
    let kill_z = world.resource::<Barriers>().kill_z;
    // (client, trigger or none, damage, spawnflags)
    let mut hits: Vec<(u32, Option<(u32, ObjRef)>, i32, i32)> = Vec::new();
    for client in players {
        let Some((mins, maxs)) = triggers::player_box(world, client) else {
            continue;
        };
        if let Some(z) = kill_z
            && mins[2] < z
        {
            let mut b = world.resource_mut::<Barriers>();
            if b.next_hurt
                .get(&(KILL_PLANE_KEY, client))
                .is_none_or(|t| *t <= now)
            {
                b.next_hurt
                    .insert((KILL_PLANE_KEY, client), now + HURT_SLOW_INTERVAL_MS / 2);
                hits.push((client, None, KILL_DAMAGE, 0));
            }
            continue;
        }
        for (n, e, k) in &list {
            if e.invisible_to.contains(&client) || e.invisible_to_all {
                continue;
            }
            if !triggers::touches(world, e, mins, maxs) {
                continue;
            }
            let obj = e.obj.unwrap();
            let (dmg, flags) = with_vm(world, |vm, _| {
                (int_key(vm, obj, "dmg"), int_key(vm, obj, "spawnflags"))
            })
            .unwrap_or_default();
            if HURT_HONOUR_START_OFF && *k == Barrier::Hurt && flags & HURT_SPAWNFLAG_START_OFF != 0
            {
                continue;
            }
            let mut b = world.resource_mut::<Barriers>();
            if b.next_hurt.get(&(*n, client)).is_some_and(|t| *t > now) {
                continue;
            }
            b.next_hurt.insert((*n, client), now + hurt_interval(flags));
            let amount = if *k == Barrier::Kill {
                KILL_DAMAGE
            } else {
                hurt_damage(dmg)
            };
            hits.push((client, Some((*n, obj)), amount, flags));
        }
    }
    // Forget touches long past (round restarts renumber entities).
    world
        .resource_mut::<Barriers>()
        .next_hurt
        .retain(|_, t| *t + HURT_SLOW_INTERVAL_MS > now);
    if hits.is_empty() {
        return;
    }
    let log = std::env::var_os("BO2MP_HITLOG").is_some();
    with_vm(world, |vm, world| {
        let np = vm.intern("idflags_no_protection");
        let no_protection = match vm.raw_field(vm.level, np) {
            Value::Int(i) => i,
            _ => 0,
        };
        for (client, trig, amount, flags) in hits {
            let point = frame(world)
                .player(ClientId(client))
                .map_or([0.0; 3], |ps| ps.origin);
            let idflags = if flags & HURT_SPAWNFLAG_NO_PROTECTION != 0 || trig.is_none() {
                no_protection
            } else {
                0
            };
            let who = match trig {
                Some((n, o)) if vm.alive(o) => {
                    if log || amount >= HURT_LOG_DAMAGE {
                        diag::info!(
                            Sim,
                            "bo2mp death barrier: trigger #{n} hurts player {client} for {amount} at ({:.0} {:.0} {:.0})",
                            point[0],
                            point[1],
                            point[2]
                        );
                    }
                    Value::Object(o)
                }
                Some(_) => continue,
                None => {
                    diag::info!(
                        Sim,
                        "bo2mp death barrier: player {client} fell below the kill plane at ({:.0} {:.0} {:.0})",
                        point[0],
                        point[1],
                        point[2]
                    );
                    Value::Undefined
                }
            };
            let weapon = "none";
            natives_ai::player_damage(
                vm,
                world,
                client,
                who.clone(),
                who,
                amount,
                idflags,
                "MOD_TRIGGER_HURT",
                weapon,
                point,
                [0.0; 3],
                "none",
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hurt_defaults_follow_sp_trigger_hurt() {
        assert_eq!(hurt_damage(0), 5);
        assert_eq!(hurt_damage(1000), 1000);
        assert_eq!(hurt_interval(0), 50);
        assert_eq!(hurt_interval(HURT_SPAWNFLAG_SLOW), 1000);
        assert_eq!(
            hurt_interval(HURT_SPAWNFLAG_SLOW | HURT_SPAWNFLAG_NO_PROTECTION),
            1000
        );
    }

    #[test]
    fn kill_plane_sits_below_collision_and_spawns() {
        // A 2000-unit tall map: 512 margin (10% is 200).
        let (z, m) = kill_plane_z(Some(-100.0), Some(1900.0), Some(0.0)).unwrap();
        assert_eq!(m, 512.0);
        assert_eq!(z, -612.0);
        // A tall one: 10% of 12000.
        let (z, m) = kill_plane_z(Some(-4000.0), Some(8000.0), Some(2000.0)).unwrap();
        assert_eq!(m, 1200.0);
        assert_eq!(z, -5200.0);
        // Capped margin.
        let (_, m) = kill_plane_z(Some(0.0), Some(1.0e5), None).unwrap();
        assert_eq!(m, KILL_PLANE_MARGIN_MAX);
        // A spawn below the collision (bad data) still keeps the plane below it.
        let (z, _) = kill_plane_z(Some(0.0), Some(1000.0), Some(-300.0)).unwrap();
        assert!(z <= -300.0 - KILL_PLANE_MARGIN_MIN);
        assert!(kill_plane_z(None, Some(10.0), Some(0.0)).is_none());
    }

    #[test]
    fn brush_bottom_reads_axial_planes() {
        // A box 0..64 in each axis: -x<=0, x<=64, -y<=0, y<=64, -z<=0, z<=64,
        // shifted to z -128..-64.
        let planes = [
            [-1.0, 0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 64.0],
            [0.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 64.0],
            [0.0, 0.0, -1.0, 128.0],
            [0.0, 0.0, 1.0, -64.0],
        ];
        assert_eq!(brush_bottom(&planes), Some(-128.0));
        assert_eq!(brush_top(&planes), Some(-64.0));
    }

    #[test]
    fn barrier_classes() {
        assert_eq!(barrier("trigger_hurt"), Some(Barrier::Hurt));
        assert_eq!(barrier("trigger_out_of_bounds"), Some(Barrier::Kill));
        assert_eq!(barrier("trigger_multiple"), None);
        assert!(oob_name("out_of_bounds_trig"));
        assert!(!oob_name("flag_trigger"));
    }
}
