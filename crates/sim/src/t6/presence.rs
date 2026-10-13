//! Script models on screen: every entity with a model gets a script mover
//! (a networked entity plus a model row the snapshot carries, as MW2's
//! script models do); its pose, visibility and attachments follow the
//! entity every frame, and it goes when the entity does.

use std::collections::BTreeMap;

use bevy_ecs::prelude::World;

use super::{Ent, Zm, frame};
use crate::ScriptModelId;
use crate::bullet_collision::{AuthorityDObjState, EntityCollisionCapabilities};

/// Spawned script movers for BO2 entities: inside the spawned range the
/// client spawns models for, clear of IW4L's own.
const T6_PRESENCE_BASE: u32 = 0x6000_0000;

/// A living zombie's box as IW4 packs an entity's `solid` (half-width 15,
/// 1 below its feet, 72 tall: BO2's actor size, as a player's): players
/// cannot walk through zombies.
const ZOMBIE_SOLID: u32 = ((72 + 32) << 16) | (1 << 8) | 15;

/// A model hung on a tag of another (`attach`), and for a weapon's parts
/// where on the tag (`useweaponmodel`: the magazine, scope or rocket).
pub(crate) type Attached = (String, String, Option<xmodel_runtime::TagOffset>);

/// What `sync` draws for an entity or a box piece: origin, angles, model,
/// what hangs on it, hidden, and the tags hidden on its model (none: left
/// alone, a destructible's or a piece's).
type Wanted = (
    [f32; 3],
    [f32; 3],
    String,
    Vec<Attached>,
    bool,
    Option<Vec<String>>,
);

/// An entity's `destructibledef` name, `None` when it has none (empty).
/// bo2 merge: the Zombies fork kept it as a `String` (empty: none), the
/// multiplayer fork as an `Option<String>`; this reads either.
trait DestructibleName {
    fn def_name(&self) -> Option<&str>;
}

impl DestructibleName for String {
    fn def_name(&self) -> Option<&str> {
        (!self.is_empty()).then_some(self.as_str())
    }
}

impl DestructibleName for Option<String> {
    fn def_name(&self) -> Option<&str> {
        self.as_deref().filter(|s| !s.is_empty())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Shown {
    pub id: ScriptModelId,
    pub number: i32,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub model: String,
    pub attachments: Vec<Attached>,
    pub hidden: bool,
    /// Hidden from some players only (`setinvisibletoplayer`): a bit per
    /// player it is still drawn for. 0 for everything else.
    pub shown_to: u64,
    /// It moved last tick: its client carries it on at that speed until it
    /// is told it stopped.
    pub moving: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Presences {
    pub by_ent: BTreeMap<u32, Shown>,
    pub next: u32,
}

fn near(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 0.01)
}

/// Rows for a zbarrier's pieces: the entity's number shifted, the piece
/// in the low bits, this bit set.
const PIECE_BIT: u32 = 0x8000_0000;

/// Loops the scripts play on entities (`playloopsound`: the box's tune, a
/// perk machine's hum, a power-up's hum), each where its entity stands,
/// owned by its model row (or its number, for an entity with no model).
pub(super) fn publish_loops(world: &mut World) {
    let engines = super::engine_sounds::rows(world);
    let mut rows: Vec<(ScriptModelId, String, [f32; 3], f32, f32)> = {
        let zm = world.resource::<Zm>();
        zm.ents
            .iter()
            .filter_map(|(n, e)| {
                let alias = e.loop_sound.as_ref()?.to_ascii_lowercase();
                let owner = zm.presences.by_ent.get(n).map_or_else(
                    || ScriptModelId::from_wire(0x2000_0000 | (n & 0x0fff_ffff)),
                    |s| s.id,
                );
                Some((owner, alias, e.origin, 1.0, 1.0))
            })
            .collect()
    };
    // Helicopter and drone engines (BO2's client script's loops), owned by
    // the vehicle's model row as its other loops are.
    {
        let zm = world.resource::<Zm>();
        for (n, alias, origin, volume, pitch) in engines {
            let owner = zm.presences.by_ent.get(&n).map_or_else(
                || ScriptModelId::from_wire(0x2000_0000 | (n & 0x0fff_ffff)),
                |s| s.id,
            );
            rows.push((owner, alias.to_owned(), origin, volume, pitch));
        }
    }
    let mut f = frame(world);
    let rows = rows
        .into_iter()
        .map(
            |(owner, alias, origin, volume, pitch)| crate::world_objects::DestructibleLoopSound {
                owner,
                alias_index: f.sound_alias_index(&alias),
                origin,
                volume: crate::world_objects::DestructibleLoopSound::hundredths(volume),
                pitch: crate::world_objects::DestructibleLoopSound::hundredths(pitch),
            },
        )
        .collect();
    f.world_objects_mut().set_destructible_loop_sounds(rows);
}

/// A model that is only something to bump into (Treyarch names them
/// `collision_*`, `zm_collision_*`; their material never draws in BO2):
/// kept solid, never shown.
/// A collision wall's half-size from its name (`collision_wall_128x128x10_
/// standard`: width x height x thickness, centred on its origin; local x,
/// z, y). These models carry no collision surfaces and only a cube for
/// bounds; Nuketown's script builds a closed box from six of them, which
/// fixes the axes.
fn named_wall_size(model: &str) -> Option<[f32; 3]> {
    if !model.starts_with("collision_") || !(model.contains("_wall_") || model.contains("_player_"))
    {
        return None;
    }
    let dims = model.split('_').find_map(|part| {
        let v: Vec<f32> = part.split('x').filter_map(|d| d.parse().ok()).collect();
        (v.len() == 3).then_some(v)
    })?;
    Some([dims[0] * 0.5, dims[2] * 0.5, dims[1] * 0.5])
}

fn collision_only(model: &str) -> bool {
    model.starts_with("collision_")
        || model.starts_with("zm_collision_")
        || model.contains("_collision_")
}

/// IW4L_T6_NEAR=<seconds>: at that time, every drawn entity within 250
/// units of player 0 (debugging what stands in view).
fn report_near(world: &mut World, now_ms: i64) {
    static AT: std::sync::OnceLock<Option<i64>> = std::sync::OnceLock::new();
    let Some(at) = *AT.get_or_init(|| {
        std::env::var("IW4L_T6_NEAR")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
    }) else {
        return;
    };
    if !(at * 1000..at * 1000 + i64::from(crate::MATCH_TICK_MS)).contains(&now_ms) {
        return;
    }
    let Some(me) = frame(world)
        .player(crate::world::ClientId(0))
        .map(|p| p.origin)
    else {
        return;
    };
    let zm = world.resource::<Zm>();
    for (n, s) in &zm.presences.by_ent {
        let d = ((s.origin[0] - me[0]).powi(2)
            + (s.origin[1] - me[1]).powi(2)
            + (s.origin[2] - me[2]).powi(2))
        .sqrt();
        if d < 250.0 {
            let class = zm.ents.get(n).map_or("", |e| e.classname.as_str());
            diag::info!(
                Sim,
                "bo2zm t6 near: ent{n} {class} model {} hidden {} at {:.0} ({:.0} {:.0} {:.0}) attached {:?}",
                s.model,
                s.hidden,
                d,
                s.origin[0],
                s.origin[1],
                s.origin[2],
                s.attachments.iter().map(|(m, t, _)| (m, t)).collect::<Vec<_>>()
            );
        }
    }
}

/// The level restarts (`map_restart`): every model it showed goes (a car
/// blown up last round comes back whole); its serials keep counting.
pub(super) fn retire_all(world: &mut World) {
    let gone = std::mem::take(&mut world.resource_mut::<Zm>().presences.by_ent);
    let mut f = frame(world);
    for shown in gone.into_values() {
        f.remove_script_mover_by_number(shown.number);
        f.remove_collision_owner(shown.id);
    }
}

pub(super) fn sync(world: &mut World, now_ms: i64) {
    report_near(world, now_ms);
    let now = now_ms as i32;
    let zm = world.resource::<Zm>();
    let pieces: Vec<(u32, Wanted)> = zm
        .ents
        .iter()
        .filter_map(|(n, e)| e.zbarrier.as_ref().map(|z| (*n, e, z)))
        .flat_map(|(n, e, z)| {
            z.pieces.iter().enumerate().filter_map(move |(i, p)| {
                // A weapon-rise piece (a bare tag) shows the box's
                // spinning gun while it opens, where its clip's root
                // motion has lifted it, turned as the box turns the gun
                // it picks (the chest's angles + 180 yaw), wearing its
                // parts (magazine, scope, rocket) and hiding its hide tags.
                let anim = zm.anims.get(&p.open_anim);
                if let Some((gun, off)) = super::zbarrier::rise_gun(p, anim, now_ms) {
                    let (f, r, u) = gsc_t6::math::angle_vectors(e.angles);
                    let origin = std::array::from_fn(|c| {
                        e.origin[c] + f[c] * off[0] - r[c] * off[1] + u[c] * off[2]
                    });
                    let angles = [e.angles[0], e.angles[1] + 180.0, e.angles[2]];
                    let parts = gun.parts.into_iter().map(|(m, t, at)| (m, t, Some(at)));
                    return Some((
                        PIECE_BIT | (n << 4) | i as u32,
                        (origin, angles, gun.model, parts.collect(), e.hidden, Some(gun.hide)),
                    ));
                }
                if p.model.is_empty() || p.model == "tag_origin" {
                    return None;
                }
                Some((
                    PIECE_BIT | (n << 4) | i as u32,
                    (
                        super::zbarrier::piece_origin(zm, e, z, p, now_ms),
                        super::zbarrier::piece_angles(zm, e, z, p, now_ms),
                        p.model.clone(),
                        Vec::new(),
                        !p.shown
                            || e.hidden
                            || (p.state == "open"
                                && z.def.as_ref().is_some_and(|d| d.auto_hide_open_pieces)),
                        None,
                    ),
                ))
            })
        })
        .collect();
    // Collision models (a perk machine's zm_collision_perks1, Nuketown's
    // patch walls, some lying on their sides) block players as their own
    // bounds turned with them while the scripts keep them solid (fix list
    // 3: they were square boxes of their widest side, ignoring the turn: a
    // 128-unit block for each 10-unit patch wall, one by the bunker, one in
    // the middle of the map, and the walls on their sides stayed open).
    let mut blocker_ents: Vec<(u32, [f32; 3], [f32; 3], String)> = zm
        .ents
        .iter()
        .filter(|(_, e)| e.solid && collision_only(&e.model))
        .map(|(n, e)| (*n, e.origin, e.angles, e.model.clone()))
        .collect();
    // The Mystery Box is solid where it stands (any box piece shown there:
    // the box, or the closed one): before, he walked through it.
    blocker_ents.extend(zm.ents.iter().filter_map(|(n, e)| {
        let z = e.zbarrier.as_ref()?;
        let piece = z
            .pieces
            .iter()
            .find(|p| p.shown && p.model.contains("magic_box"))?;
        (!e.hidden).then(|| (*n, e.origin, e.angles, piece.model.clone()))
    }));
    // bo2mp: a model shown only to some players (`showtoplayer`) is drawn
    // when one of them plays at this PC: the server's own humans, not its
    // bots.
    let humans: Vec<u32> = zm
        .players
        .iter()
        .filter(|(_, p)| !p.test_client)
        .map(|(c, _)| *c)
        .collect();
    let shown_here = |e: &Ent| {
        e.shown_to
            .as_ref()
            .is_none_or(|to| humans.iter().any(|h| to.contains(h)))
    };
    // Tranzit's bus: its body as boxes (`transit`), and its ladder once
    // built (with the faces' surface flags).
    type FlaggedBox = (([f32; 3], [f32; 3]), u32);
    let platform_ents: Vec<(u32, [f32; 3], [f32; 3], Vec<FlaggedBox>)> = zm
        .ents
        .iter()
        .filter(|(_, e)| e.platform && !e.hidden)
        .filter_map(|(n, e)| {
            let boxes = super::transit::platform_boxes(&e.model);
            (!boxes.is_empty()).then(|| {
                let mut all: Vec<FlaggedBox> = boxes.iter().map(|&b| (b, 0)).collect();
                all.extend(super::transit::platform_ladder(zm, *n, &e.model));
                (*n, e.origin, e.angles, all)
            })
        })
        .collect();
    // Die Rise's elevator cars: their floor and roof (`highrise`), under
    // the car's own entity number as its clients know the car, so a
    // player's ground entity there names it (`platform_carry` draws his
    // ride smooth by it).
    let slab_ents: Vec<(u32, [f32; 3], [f32; 3], String, u16)> = zm
        .ents
        .iter()
        .filter(|(_, e)| {
            e.platform && !e.hidden && !super::highrise::platform_slabs(&e.model).is_empty()
        })
        .map(|(n, e)| {
            let number = zm
                .presences
                .by_ent
                .get(n)
                .and_then(|s| u16::try_from(s.number).ok())
                .unwrap_or(*n as u16);
            (*n, e.origin, e.angles, e.model.clone(), number)
        })
        .collect();
    let mut wanted: BTreeMap<u32, Wanted> = zm
        .ents
        .iter()
        .filter_map(|(n, e)| {
            // A brush door (a map script_brushmodel's `*N`) draws its own
            // surfaces; brushes.rs gives it collision.
            let model = match &e.brush {
                Some(b) if e.model.is_empty() && e.classname == "script_brushmodel" => b,
                _ => &e.model,
            };
            (e.obj.is_some()
                && !model.is_empty()
                && (!model.starts_with('*') || e.classname == "script_brushmodel")
                // Spawners are never drawn (the actors they spawn are).
                && (!e.classname.starts_with("actor_") || zm.actors.contains_key(n)))
            .then(|| (n, e, model.clone()))
        })
        .map(|(n, e, model)| {
            (
                *n,
                (
                    e.origin,
                    e.angles,
                    model,
                    e.attached
                        .iter()
                        .map(|(m, t)| (m.clone(), t.clone(), None))
                        .chain(
                            e.weapon_parts
                                .iter()
                                .map(|(m, t, at)| (m.clone(), t.clone(), Some(*at))),
                        )
                        .collect(),
                    // Hidden from everyone. Hidden from some players only
                    // (Mob's afterlife doors while one is a ghost) is the
                    // mask below.
                    e.hidden
                        || e.invisible_to_all
                        || !shown_here(e)
                        || collision_only(&e.model),
                    // A breakable prop hides its own launched parts.
                    e.destructible.def_name().is_none().then(|| e.hidden_parts.clone()),
                ),
            )
        })
        .collect();
    wanted.extend(pieces);
    // Retire the gone.
    let gone: Vec<(u32, Shown)> = world
        .resource::<Zm>()
        .presences
        .by_ent
        .iter()
        .filter(|(n, _)| !wanted.contains_key(n))
        .map(|(n, s)| (*n, s.clone()))
        .collect();
    for (n, shown) in gone {
        let mut f = frame(world);
        f.remove_script_mover_by_number(shown.number);
        f.remove_collision_owner(shown.id);
        world.resource_mut::<Zm>().presences.by_ent.remove(&n);
    }
    // A bit per player in the match (a player's entity number is his client).
    let everyone: u64 = world
        .resource::<Zm>()
        .players
        .keys()
        .filter(|c| **c < 64)
        .fold(0, |m, c| m | (1 << c));
    for (n, (origin, angles, model, attachments, hidden, hide)) in wanted {
        // BO2's `setinvisibletoplayer` hides it from those players only;
        // the rest still see it (before, only player 1's view counted).
        let mask: u64 = world.resource::<Zm>().ents.get(&n).map_or(0, |e| {
            e.invisible_to
                .iter()
                .filter(|c| **c < 64)
                .fold(0, |m, c| m | (1 << c))
        }) & everyone;
        // Hidden from every player: no hit boxes either (as solo was).
        let hidden_all = hidden || (mask != 0 && mask == everyone);
        // Not drawn by default; shown back to the players in `shown_to`.
        let draw_hidden = hidden_all || mask != 0;
        let shown_to = if hidden_all || mask == 0 { 0 } else { everyone & !mask };
        let existing = world.resource::<Zm>().presences.by_ent.get(&n).cloned();
        let mut shown = match existing {
            Some(s) => s,
            None => {
                let (destructible, mp) = {
                    let zm = world.resource::<Zm>();
                    let def = zm
                        .ents
                        .get(&n)
                        .and_then(|e| e.destructible.def_name())
                        .and_then(|name| zm.destructibles.get(name).cloned());
                    (def, zm.mp)
                };
                let serial = {
                    let mut zm = world.resource_mut::<Zm>();
                    let s = zm.presences.next;
                    zm.presences.next += 1;
                    s
                };
                let id = ScriptModelId::from_wire(T6_PRESENCE_BASE + serial);
                let mut f = frame(world);
                let Ok(number) = f.spawn_script_mover(id, origin, angles) else {
                    diag::warn!(Sim, "bo2zm t6: no free entity for model {model}");
                    continue;
                };
                let capability = f.model_capability(&model).flatten();
                // IW4L_T6_PRESLOG=<part of a model name>: a line as it shows.
                if std::env::var("IW4L_T6_PRESLOG").is_ok_and(|w| model.contains(w.as_str())) {
                    diag::info!(
                        Sim,
                        "bo2zm t6 presence ent{n} {model} hidden={draw_hidden} capability={}",
                        capability.is_some()
                    );
                }
                let mut dobj = AuthorityDObjState::at_pose(&model, capability, origin, angles);
                // A map prop that breaks in stages (a mannequin's head).
                // bo2mp: on a multiplayer map its breaks are kept for the
                // scripts (hosted, `destructible::flush`); Zombies hears them
                // through `destructible_broken`.
                if let Some(def) = destructible.clone() {
                    if mp {
                        crate::t5_destructible::install_hosted(&mut dobj, def);
                    } else {
                        crate::t5_destructible::install_shown(&mut dobj, def);
                    }
                }
                f.insert_collision_owner(EntityCollisionCapabilities::current_tick(
                    crate::AuthorityModelOwner::ScriptModel(id),
                    Some(dobj),
                    Vec::new(),
                ));
                Shown {
                    id,
                    number,
                    origin,
                    angles,
                    model: String::new(),
                    attachments: Vec::new(),
                    hidden: !draw_hidden,
                    shown_to: 0,
                    moving: false,
                }
            }
        };
        let (yaw_goal, holds_still) = world
            .resource::<Zm>()
            .actors
            .get(&n)
            .filter(|a| a.alive)
            .map_or((None, false), |a| {
                let still = matches!(a.script.as_str(), "combat" | "stop")
                    && a.scripted.is_none()
                    && a.traverse.is_none();
                (a.yaw_goal, still)
            });
        let prev_yaw = shown.angles[1];
        let is_vehicle = {
            let zm = world.resource::<Zm>();
            vehicle_borne(&zm, n)
        };
        let mut f = frame(world);
        let moved = !near(shown.origin, origin) || !near(shown.angles, angles);
        // A move that ended goes out once more, standing still (else the
        // client keeps it going: perk machines flew off into the sky).
        if moved || shown.moving || yaw_goal.is_some() {
            shown.moving = moved;
            f.set_script_mover_pose(shown.number, now, origin, angles);
            // The last send of a move that ended stands still, facing where
            // it faces. Its turn was measured from the facing the turn below
            // started from (a tick behind), so a zombie killed mid-turn was
            // sent turning on with no end: its corpse spun until it went.
            if !moved && let Some(mover) = f.script_mover_mut_by_number(shown.number) {
                let state = &mut mover.state;
                state.tr_type = entity_iw4::TR_STATIONARY;
                state.tr_time = now;
                state.tr_base = origin;
                state.tr_delta = [0.0; 3];
                state.tr_duration = 0;
                state.apos_tr_type = entity_iw4::TR_STATIONARY;
                state.apos_tr_time = now;
                state.apos_tr_base = angles;
                state.apos_tr_delta = [0.0; 3];
                state.apos_tr_duration = 0;
            }
            // bo2zm M3 fix list 2: a turning zombie's client turns it toward
            // the facing it is turning to at the server's rate and stops
            // there, as the next tick will. Carried on at the last tick's
            // turn instead, the body swung past the end of every turn (up to
            // 12 degrees) and snapped back.
            // A zombie that has stopped (attacking, standing) moves no
            // further next tick: it is drawn where it stands, not carried on
            // a few units past its stop and pulled back.
            if holds_still && let Some(mover) = f.script_mover_mut_by_number(shown.number) {
                let state = &mut mover.state;
                state.tr_type = entity_iw4::TR_STATIONARY;
                state.tr_time = now;
                state.tr_base = origin;
                state.tr_delta = [0.0; 3];
                state.tr_duration = 0;
            }
            // IW4L_T6_NOTURNGOAL=1: the old carry-on, to compare (test aid).
            let use_goal = std::env::var_os("IW4L_T6_NOTURNGOAL").is_none();
            if use_goal
                && let Some(goal) = yaw_goal
                && let Some(mover) = f.script_mover_mut_by_number(shown.number)
            {
                // From the facing it had last tick (where his client shows it
                // now): the turn trails the server by a tick, so it starts
                // smoothly instead of jumping a whole tick's turn (18 degrees).
                let from = [angles[0], prev_yaw, angles[2]];
                let left = math_iw4::angle_subtract(goal, from[1]);
                let state = &mut mover.state;
                if left.abs() > 0.05 {
                    let rate = super::actors::TURN_DEG_PER_S;
                    state.apos_tr_type = entity_iw4::TR_LINEAR_STOP;
                    state.apos_tr_time = now;
                    state.apos_tr_base = from;
                    state.apos_tr_delta = [0.0, rate.copysign(left), 0.0];
                    state.apos_tr_duration = ((left.abs() / rate) * 1000.0).round().max(1.0) as i32;
                } else {
                    state.apos_tr_type = entity_iw4::TR_STATIONARY;
                    state.apos_tr_time = now;
                    state.apos_tr_base = angles;
                    state.apos_tr_delta = [0.0; 3];
                    state.apos_tr_duration = 0;
                }
            }
            // bo2zm: a vehicle (Tranzit's bus) turns in steps at each path
            // node. Carried on at its last tick's speed and turn, it swung up
            // to 14 degrees past each turn and snapped back. It is drawn
            // easing from last tick's place to this tick's and stopping
            // there (a tick behind the server; his ride view uses the end).
            // What is linked to it (the driver, its doors and ladder, the
            // zombies on it) is drawn the same way: carried on a tick ahead
            // while the bus was a tick behind, the driver slid around in his
            // seat (a tester's playtest, "the bus driver is sliding around").
            if moved
                && is_vehicle
                && let Some(mover) = f.script_mover_mut_by_number(shown.number)
            {
                let tick = crate::MATCH_TICK_MS as i32;
                let per_sec = 1000.0 / tick as f32;
                let state = &mut mover.state;
                state.tr_type = entity_iw4::TR_LINEAR_STOP;
                state.tr_time = now;
                state.tr_duration = tick;
                state.tr_base = shown.origin;
                state.tr_delta = std::array::from_fn(|i| (origin[i] - shown.origin[i]) * per_sec);
                state.apos_tr_type = entity_iw4::TR_LINEAR_STOP;
                state.apos_tr_time = now;
                state.apos_tr_duration = tick;
                state.apos_tr_base = shown.angles;
                state.apos_tr_delta = std::array::from_fn(|i| {
                    math_iw4::angle_subtract(angles[i], shown.angles[i]) * per_sec
                });
            }
            if let Some(row) = f.collision_owner_mut(shown.id) {
                if let Some(dobj) = row.dobj.as_mut() {
                    dobj.set_world_pose(origin, angles);
                }
                row.followed_pose = Some((origin, angles));
            }
            shown.origin = origin;
            shown.angles = angles;
        }
        if shown.hidden != draw_hidden || shown.shown_to != shown_to {
            if let Some(mover) = f.script_mover_mut_by_number(shown.number) {
                if draw_hidden {
                    mover.state.e_flags |= entity_iw4::CG_SCRIPT_MOVER_NODRAW;
                } else {
                    mover.state.e_flags &= !entity_iw4::CG_SCRIPT_MOVER_NODRAW;
                }
                mover.nonsolid = true;
                mover.shown_to = shown_to;
            }
            if let Some(row) = f.collision_owner_mut(shown.id) {
                row.hidden = draw_hidden;
                row.solid = false;
            }
            // IW4L_T6_SHOWNLOG=1: a line each time one is drawn for some
            // players only (a test aid for co-op).
            if (shown_to != 0 || shown.shown_to != 0)
                && std::env::var_os("IW4L_T6_SHOWNLOG").is_some()
            {
                diag::info!(
                    Sim,
                    "bo2zm t6 shown ent{n} {model} to {shown_to:#b} (was {:#b})",
                    shown.shown_to
                );
            }
            shown.hidden = draw_hidden;
            shown.shown_to = shown_to;
        }
        if shown.model != model || shown.attachments != attachments {
            let capability = f.model_capability(&model).flatten();
            if let Some(row) = f.collision_owner_mut(shown.id) {
                match row.dobj.as_mut() {
                    Some(dobj) => {
                        if dobj.current_model != model {
                            dobj.replace_model(&model, capability);
                        }
                    }
                    None => {
                        row.dobj = Some(AuthorityDObjState::at_pose(
                            &model, capability, origin, angles,
                        ))
                    }
                }
                if let Some(dobj) = row.dobj.as_mut() {
                    let a: Vec<(&str, &str, Option<xmodel_runtime::TagOffset>)> = attachments
                        .iter()
                        .map(|(m, t, at)| (m.as_str(), t.as_str(), *at))
                        .collect();
                    dobj.set_attachments_at(&a);
                }
            }
            shown.model = model;
            shown.attachments = attachments;
        }
        // bo2mp: a multiplayer map's destructible (a mannequin, a car)
        // breaks by its definition, its breaks kept for the scripts
        // (`destructible::flush`); installed hosted when shown, and again
        // if its model row lost it.
        if world.resource::<Zm>().mp {
            let destructible = (n & PIECE_BIT == 0)
                .then(|| {
                    let zm = world.resource::<Zm>();
                    let name = zm.ents.get(&n)?.destructible.def_name()?;
                    zm.destructibles.get(name).cloned()
                })
                .flatten();
            if let Some(def) = &destructible {
                let mut f = frame(world);
                if let Some(row) = f.collision_owner_mut(shown.id)
                    && let Some(dobj) = row.dobj.as_mut()
                    && !crate::t5_destructible::installed(dobj, &def.name)
                {
                    crate::t5_destructible::install_hosted(dobj, def.clone());
                    diag::info!(
                        Sim,
                        "bo2mp destructible {} ent{n} at {:?}",
                        def.name,
                        shown.origin
                    );
                }
            }
        }
        if let Some(hide) = &hide {
            set_hidden_parts(world, hide, shown.id);
        }
        // Actors: bullets hit the living (their posed hit boxes), and the
        // playing animation goes out as the model row's one leaf.
        let alive_actor = world
            .resource::<Zm>()
            .actors
            .get(&n)
            .is_some_and(|a| a.alive);
        let boxes = if alive_actor && !hidden_all {
            let f = frame(world);
            let placement = f
                .entity_collision_capabilities()
                .iter()
                .find(|row| row.owner.script_model() == Some(shown.id))
                .and_then(|row| row.dobj.as_ref())
                .map(|d| d.world_from_model);
            placement.and_then(|m| super::actors::hitboxes(world, n, now_ms, m))
        } else {
            None
        };
        // A breakable prop (the map's `destructibledef`: Nuketown's
        // mannequins) keeps the hit shape its model builds, so bullets
        // reach its pieces.
        let breakable = !hidden_all
            && world
                .resource::<Zm>()
                .ents
                .get(&n)
                .is_some_and(|e| e.destructible.def_name().is_some());
        if breakable {
            let mut f = frame(world);
            if let Some(row) = f.collision_owner_mut(shown.id) {
                row.solid = true;
            }
        } else {
            let mut f = frame(world);
            let packed = if boxes.is_some() { ZOMBIE_SOLID } else { 0 };
            if let Some(mover) = f.script_mover_mut_by_number(shown.number) {
                if mover.state.solid != packed {
                    mover.state.solid = packed;
                }
                // The movers pass (`presence::follow_movers`, every tick)
                // copies this onto the hit row: left non-solid from the
                // first show, a living zombie lost its hit boxes after
                // the tick it was posed, and the Sliquifier bolt flew
                // through it (it only met zombies on the tick it was fired).
                mover.nonsolid = boxes.is_none();
            }
            if let Some(row) = f.collision_owner_mut(shown.id) {
                row.solid = boxes.is_some();
                if let Some(dobj) = row.dobj.as_mut() {
                    dobj.current_collision = boxes.map(|bones| {
                        crate::bullet_collision::AuthorityDObjCollision {
                            bones,
                            coll: None,
                            surface_flags: 0,
                        }
                    });
                    dobj.materialized_model_revision = Some(dobj.model_revision);
                    dobj.materialized_pose_revision = Some(dobj.pose_revision);
                }
            }
        }
        let anim = if n & PIECE_BIT != 0 {
            let (e, i) = ((n & !PIECE_BIT) >> 4, (n & 0xf) as usize);
            world
                .resource::<Zm>()
                .ents
                .get(&e)
                .and_then(|e| e.zbarrier.as_ref())
                .and_then(|z| {
                    let boards = super::zbarrier::has_boards(z);
                    z.pieces.get(i).map(|p| (p, boards))
                })
                .and_then(|(p, boards)| super::zbarrier::piece_pose(p, now_ms, boards).1)
        } else {
            super::actors::present_anim(world, n, now_ms)
                .or_else(|| super::fxanims::present(world, n, now_ms))
                .or_else(|| script_anim(world, n, now_ms))
        };
        if let Some((clip, time, rate)) = anim {
            let mut f = frame(world);
            if let Some(row) = f.collision_owner_mut(shown.id)
                && let Some(dobj) = row.dobj.as_mut()
            {
                use xmodel_runtime::{
                    XAnimNodeState, XAnimSemanticNode, XAnimSemanticNodeKind, XAnimTreeSnapshot,
                };
                let rev = dobj.semantic_state.pose_revision.wrapping_add(1);
                dobj.semantic_state.pose_revision = rev;
                dobj.semantic_state.tree = Some(XAnimTreeSnapshot {
                    definition_revision: 1,
                    state_revision: rev,
                    nodes: vec![XAnimSemanticNode {
                        parent: None,
                        kind: XAnimSemanticNodeKind::Leaf,
                        clip: Some(clip),
                        parts: None,
                        state: XAnimNodeState {
                            time,
                            old_time: time,
                            weight: 1.0,
                            goal_weight: 1.0,
                            rate,
                            ..XAnimNodeState::default()
                        },
                    }],
                });
            }
        }
        // Gone limp (`startragdoll`): the clients drop the body.
        let ragdoll = world.resource::<Zm>().ents.get(&n).and_then(|e| e.ragdoll);
        if let Some(row) = frame(world).collision_owner_mut(shown.id)
            && let Some(dobj) = row.dobj.as_mut()
            && dobj.semantic_state.ragdoll != ragdoll
        {
            dobj.semantic_state.ragdoll = ragdoll;
        }
        // bo2zm round 1: its eye brightness rides along for the eye shader.
        let eyes = world.resource::<Zm>().ents.get(&n).and_then(|e| e.eye_luminance);
        if let Some(row) = frame(world).collision_owner_mut(shown.id)
            && let Some(dobj) = row.dobj.as_mut()
            && dobj.semantic_state.eye_luminance != eyes
        {
            dobj.semantic_state.eye_luminance = eyes;
        }
        world.resource_mut::<Zm>().presences.by_ent.insert(n, shown);
    }
    // The collision models as turned boxes, for his movement (the server's
    // and his client's prediction: they go out in the snapshot).
    let mut f = frame(world);
    let blockers: Vec<crate::OrientedBlocker> = blocker_ents
        .iter()
        .filter_map(|(n, origin, angles, model)| {
            let (lo, hi) = match named_wall_size(model) {
                Some(half) => (half.map(|v| -v), half),
                None => {
                    // The model's bounds come as middle and half-size.
                    let (mid, half) = f.model_capability(model).flatten().and_then(|c| c.bounds)?;
                    (
                        std::array::from_fn(|i| mid[i] - half[i]),
                        std::array::from_fn(|i| mid[i] + half[i]),
                    )
                }
            };
            let (fw, rt, up) = gsc_t6::math::angle_vectors(*angles);
            Some(crate::OrientedBlocker {
                entnum: *n as u16,
                origin: *origin,
                axis: [fw, [-rt[0], -rt[1], -rt[2]], up],
                mins: lo,
                maxs: hi,
                surface_flags: 0,
            })
        })
        .collect();
    let mut blockers = blockers;
    for (_, origin, angles, model, number) in &slab_ents {
        let Some((mid, half)) = f.model_capability(model).flatten().and_then(|c| c.bounds) else {
            continue;
        };
        let (fw, rt, up) = gsc_t6::math::angle_vectors(*angles);
        for &(z0, z1) in super::highrise::platform_slabs(model) {
            blockers.push(crate::OrientedBlocker {
                entnum: *number,
                origin: *origin,
                axis: [fw, [-rt[0], -rt[1], -rt[2]], up],
                mins: [mid[0] - half[0], mid[1] - half[1], z0],
                maxs: [mid[0] + half[0], mid[1] + half[1], z1],
                surface_flags: 0,
            });
        }
    }
    for (n, origin, angles, boxes) in &platform_ents {
        let (fw, rt, up) = gsc_t6::math::angle_vectors(*angles);
        blockers.extend(boxes.iter().map(|&((mins, maxs), surface_flags)| crate::OrientedBlocker {
            entnum: *n as u16,
            origin: *origin,
            axis: [fw, [-rt[0], -rt[1], -rt[2]], up],
            mins,
            maxs,
            surface_flags,
        }));
    }
    // IW4L_T6_BLOCKERLOG=1: each blocker when the list changes (test aid).
    if std::env::var_os("IW4L_T6_BLOCKERLOG").is_some()
        && f.oriented_blockers() != blockers.as_slice()
    {
        for b in &blockers {
            diag::info!(
                Sim,
                "bo2zm t6 blocker ent{} at ({:.0} {:.0} {:.0}) fwd ({:.2} {:.2} {:.2}) up ({:.2} {:.2} {:.2}) bounds {:?}..{:?} flags {:#x}",
                b.entnum,
                b.origin[0],
                b.origin[1],
                b.origin[2],
                b.axis[0][0],
                b.axis[0][1],
                b.axis[0][2],
                b.axis[2][0],
                b.axis[2][1],
                b.axis[2][2],
                b.mins,
                b.maxs,
                b.surface_flags
            );
        }
    }
    f.set_oriented_blockers(blockers);
}

/// A vehicle, or an entity linked (through its link chain) to one.
fn vehicle_borne(zm: &Zm, mut n: u32) -> bool {
    for _ in 0..8 {
        if zm.vehicles.speed.contains_key(&n) {
            return true;
        }
        match zm.ents.get(&n).and_then(|e| e.link) {
            Some((p, _, _)) => n = p,
            None => break,
        }
    }
    false
}

/// The tags its scripts hid (`hidepart`) as the model's hide bits. Props
/// that break in stages keep their own.
fn set_hidden_parts(world: &mut World, parts: &[String], id: ScriptModelId) {
    let mut f = frame(world);
    let Some(dobj) = f.collision_owner_mut(id).and_then(|r| r.dobj.as_mut()) else {
        return;
    };
    let mut words = [0u32; 6];
    if let Some(cap) = &dobj.capability {
        for (i, bone) in cap.pose.bone_names.iter().enumerate().take(192) {
            if parts.iter().any(|p| p.eq_ignore_ascii_case(bone)) {
                words[i / 32] |= 0x8000_0000 >> (i % 32);
            }
        }
    }
    let hide = xmodel_runtime::HidePartBits::from_words(words);
    if hide != dobj.semantic_state.hide_part_bits {
        dobj.semantic_state.hide_part_bits = hide;
        dobj.pose_request.hide_part_bits = hide;
        dobj.pose_revision = dobj.pose_revision.wrapping_add(1);
        dobj.semantic_state.pose_revision = dobj.pose_revision;
        dobj.current_collision = None;
        dobj.materialized_pose_revision = None;
    }
}

/// A script model's `setanim` clip: (clip, fraction, rate per second),
/// looping when the clip loops, else held on its last frame.
pub(super) fn script_anim(world: &World, n: u32, now: i64) -> Option<(String, f32, f32)> {
    let zm = world.resource::<Zm>();
    let (clip, start) = zm.ents.get(&n)?.anim.clone()?;
    let a = zm.anims.get(&clip)?;
    let len = ((f32::from(a.numframes) / a.framerate.max(1.0)) * 1000.0).max(1.0) as i64;
    let elapsed = (now - start).max(0);
    Some(if a.looping {
        (clip, (elapsed % len) as f32 / len as f32, 1000.0 / len as f32)
    } else if elapsed >= len {
        (clip, 1.0, 0.0)
    } else {
        (clip, elapsed as f32 / len as f32, 1000.0 / len as f32)
    })
}
