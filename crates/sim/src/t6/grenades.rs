//! Thrown grenades as the scripts see them (BO2's engine side): a player's
//! throw becomes a script entity ("grenade") and the player is notified
//! `grenade_fire` (grenade, weapon name); the entity follows the grenade,
//! is notified `stationary` when it comes to rest and `explode` (origin)
//! then `death` when it goes off. The Monkey Bomb lives on this: its script
//! hides the grenade, links the monkey model to it, waits for it to land,
//! then claps, sings and draws the zombies until the fuse it resets runs out.
//!
//! A gun's shot that flies (a projectile weapon: Origins' staffs, the Ray
//! Gun) is a "rocket" entity the same way: the shooter is notified
//! `missile_fire` (shot, weapon name); a damageable model its path goes
//! through is damaged (the wind staff turns the rings over its chamber and
//! the smokes); where it goes off the shooter is notified
//! `projectile_impact` (weapon name, point, radius, shot, 0), which the
//! staffs' effects and the fire and lightning puzzles wait for. A shot
//! whose entity the script deletes (the wind staff's, after 0.75 s) goes.

use bevy_ecs::prelude::World;
use gsc_t6::{ObjKind, ObjRef, Value};

use super::{Ent, Zm, frame, with_vm};
use crate::{ClientId, ProjectileId};

#[derive(Clone, Debug)]
pub(crate) struct Thrown {
    id: ProjectileId,
    entnum: i32,
    weapon: u32,
    n: u32,
    stationary: bool,
    hidden: bool,
    /// The last zombie-distance log line (IW4L_T6_SNDLOG).
    logged: i64,
    /// A gun's flying shot (`missile_fire`), not a throw.
    missile: bool,
    /// A gun's shot the scripts see as a grenade (the Sliquifier's bolt):
    /// in flight it still hits damageable models as the gun, like a missile
    /// (BO2's MissileImpact damages what it hits with the shot's weapon;
    /// Die Rise's egg counts Sliquifier hits on its two balls).
    shot: bool,
    /// A shot BO2 marks retrievable (the ballistic knife's blade): it comes
    /// to rest instead of going off (`retrieve`).
    retrievable: bool,
    owner: ClientId,
    /// The shot's impact damage, where it was last tick, and the models its
    /// path already went through.
    amount: i32,
    last: [f32; 3],
    passed: Vec<ObjRef>,
}

fn hitlog() -> bool {
    std::env::var_os("IW4L_T6_HITLOG").is_some()
}

/// Where a shot went off (equipment's detonation): its `projectile_impact`
/// point.
pub(crate) fn note_detonation(world: &mut World, id: ProjectileId, at: [f32; 3]) {
    if let Some(mut zm) = world.get_resource_mut::<Zm>()
        && zm.thrown.iter().any(|t| t.id == id && (t.missile || t.shot))
    {
        zm.detonations.push((id, at));
    }
}

/// The damageable models the shot's path went through since last tick.
fn pass_through(world: &mut World, t: &Thrown, to: [f32; 3]) -> Vec<ObjRef> {
    super::triggers::damage_triggers_but(
        world,
        t.last,
        to,
        0.0,
        t.amount.max(1),
        t.owner,
        t.weapon,
        super::DamageKind::Projectile,
        &t.passed,
    )
}

/// A shot that lands without going off (the ballistic knife's blade) hits
/// the damage triggers its last stretch crosses, as BO2's landing does
/// (MOD_IMPACT, its impact damage); one that goes off, or does no damage,
/// does not.
fn land_on_triggers(world: &mut World, t: &Thrown, at: [f32; 3]) {
    let explodes = {
        let f = frame(world);
        f.equipment_facts_for(t.weapon)
            .or_else(|| f.missile_launch_facts(t.weapon))
            .is_some_and(|f| f.proj_impact_explode)
    };
    if explodes || t.amount <= 0 {
        return;
    }
    super::triggers::damage_triggers_but(
        world,
        t.last,
        at,
        0.0,
        t.amount,
        t.owner,
        t.weapon,
        super::DamageKind::Impact,
        &[],
    );
}

/// A flying shot gone off: the models on its last stretch, then
/// `explode`, the shooter's `projectile_impact`, `death`; its entity stays
/// where it hit until next tick (the fire puzzle asks whether it touches a
/// sacrifice volume).
fn missile_gone(world: &mut World, t: &Thrown, at: Option<[f32; 3]>, now: i64) {
    let Some((o, last)) = world
        .resource::<Zm>()
        .ents
        .get(&t.n)
        .and_then(|e| Some((e.obj?, e.origin)))
    else {
        return;
    };
    let point = at.unwrap_or(last);
    pass_through(world, t, point);
    if at.is_some() {
        land_on_triggers(world, t, point);
    }
    let radius = crate::equipment::projectile_impact_facts(&frame(world), t.weapon).map_or(0.0, |f| f.1);
    let player = {
        let mut zm = world.resource_mut::<Zm>();
        zm.movers.list.remove(&t.n);
        if let Some(e) = zm.ents.get_mut(&t.n) {
            e.origin = point;
        }
        zm.players.get(&t.owner.0).map(|p| p.obj)
    };
    let name = super::weapon_text(world, t.weapon);
    if hitlog() {
        diag::info!(
            Sim,
            "bo2zm t6 projectile_impact {name} ent{} at ({:.0} {:.0} {:.0}) radius {radius}{}",
            t.n,
            point[0],
            point[1],
            point[2],
            if at.is_some() { "" } else { " (no detonation, last place)" }
        );
    }
    with_vm(world, |vm, world| {
        if !vm.alive(o) {
            return;
        }
        vm.notify_str(world, o, "explode", &[Value::Vec3(point)]);
        if let Some(p) = player
            && vm.alive(p)
        {
            let w = vm.string(&name);
            vm.notify_str(
                world,
                p,
                "projectile_impact",
                &[w, Value::Vec3(point), Value::Float(radius), Value::Object(o), Value::Int(0)],
            );
        }
        vm.notify_str(world, o, "death", &[]);
    });
    world.resource_mut::<Zm>().thrown_free.push((o, now + 1));
}

/// Test log (IW4L_T6_HITLOG): the models a gun's shot hit on its way (the
/// Sliquifier on Die Rise's sliquify balls).
fn log_shot_hits(world: &mut World, t: &Thrown, hit: Vec<ObjRef>) {
    if hit.is_empty() || !hitlog() {
        return;
    }
    let name = super::weapon_text(world, t.weapon);
    diag::info!(Sim, "bo2zm t6 slipgun shot hit {name} ent{} hit {}", t.n, hit.len());
}

/// Test log: the first gun shot's projectile settings, once (IW4L_T6_SNDLOG).
static BOLT_FACTS_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// How long a gone-off grenade's script object outlives it. BO2 frees the
/// entity at once but the scripts running on it keep going (a removed
/// entity): the Sliquifier's bolt runs its puddle on itself after `explode`
/// (slip_bolt -> add_slippery_spot: half a second, then 24 s, 36 s upgraded,
/// longer when the puddles are many). Our VM lets a freed object's threads
/// run on (BO2's delete rule), so the object only waits out the tick its
/// `explode` waiters wake on. Entity numbers are never reused, so nothing
/// else can reach it.
const GONE_SCRIPT_HOLD_MS: i64 = 1_000;

/// The bones a retrievable shot can stay in (BO2's ragdoll bones).
const STICK_BONES: [&str; 12] = [
    "j_head",
    "j_neck",
    "j_spineupper",
    "j_spinelower",
    "j_elbow_ri",
    "j_elbow_le",
    "j_wrist_ri",
    "j_wrist_le",
    "j_knee_ri",
    "j_knee_le",
    "j_ankle_ri",
    "j_ankle_le",
];

/// A retrievable shot come to rest (the ballistic knife's blade): BO2 does
/// not set it off. Its entity is notified `stationary` (the point, the
/// surface's normal, its angles, its owner, the zombie it went into and
/// that zombie's bone nearest the point, or "" when it hit no one), which
/// the knife's script waits for to put the blade to pick up there; then
/// the shot goes away with no `explode` or `death`. One that went nowhere
/// just goes.
fn retrieve(world: &mut World, t: &Thrown, at: Option<[f32; 3]>, now: i64) {
    {
        let mut f = frame(world);
        if f.projectile_mut_by_number(t.entnum).is_some_and(|p| p.id == t.id) {
            let _ = f.remove_projectile_by_number(t.entnum);
        }
    }
    let (o, player) = {
        let mut zm = world.resource_mut::<Zm>();
        zm.thrown.retain(|x| x.id != t.id);
        zm.movers.list.remove(&t.n);
        let o = zm.ents.get(&t.n).and_then(|e| e.obj);
        if let (Some(e), Some(at)) = (zm.ents.get_mut(&t.n), at) {
            e.origin = at;
        }
        (o, zm.players.get(&t.owner.0).map(|p| p.obj))
    };
    let Some(o) = o else { return };
    world.resource_mut::<Zm>().thrown_free.push((o, now + 1));
    let Some(at) = at else { return };
    let d: [f32; 3] = std::array::from_fn(|i| at[i] - t.last[i]);
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let dir = if len > 0.01 { d.map(|x| x / len) } else { [0.0, 0.0, -1.0] };
    let tr = frame(world).trace_static_world(
        std::array::from_fn(|i| at[i] - dir[i] * 8.0),
        std::array::from_fn(|i| at[i] + dir[i] * 8.0),
        [0.0; 3],
        [0.0; 3],
        crate::bullet_collision::MASK_SHOT,
    );
    let normal = if tr.fraction < 1.0 { tr.normal } else { dir.map(|x| -x) };
    let angles = math_iw4::vect_to_angles(dir);
    // The zombie it went into: the nearest one standing round the point.
    let prey = {
        let zm = world.resource::<Zm>();
        zm.actors
            .keys()
            .filter_map(|n| zm.ents.get(n).and_then(|e| Some((*n, e.obj?, e.origin))))
            .map(|(n, obj, o)| (n, obj, o, (o[0] - at[0]).hypot(o[1] - at[1]), at[2] - o[2]))
            .filter(|&(.., flat, up)| flat <= 24.0 && (-4.0..=80.0).contains(&up))
            .min_by(|a, b| a.3.total_cmp(&b.3))
            .map(|(n, obj, ..)| (n, obj))
    };
    let bone = prey.map_or("", |(n, _)| {
        STICK_BONES
            .iter()
            .filter_map(|b| {
                let (p, _) = super::tags::tag_world(world, n, b)?;
                Some((*b, (0..3).map(|i| (p[i] - at[i]) * (p[i] - at[i])).sum::<f32>()))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or("j_knee_ri", |(b, _)| b)
    });
    if hitlog() {
        let name = super::weapon_text(world, t.weapon);
        diag::info!(
            Sim,
            "bo2zm t6 stationary {name} ent{} at ({:.0} {:.0} {:.0}) normal ({:.2} {:.2} {:.2}) prey {:?} bone {bone:?}",
            t.n,
            at[0],
            at[1],
            at[2],
            normal[0],
            normal[1],
            normal[2],
            prey.map(|(n, _)| n)
        );
    }
    with_vm(world, |vm, world| {
        if !vm.alive(o) {
            return;
        }
        let owner = player.filter(|p| vm.alive(*p)).map_or(Value::Undefined, Value::Object);
        let prey = prey.map_or(Value::Undefined, |(_, p)| Value::Object(p));
        let bone = vm.string(bone);
        vm.notify_str(
            world,
            o,
            "stationary",
            &[Value::Vec3(at), Value::Vec3(normal), Value::Vec3(angles), owner, prey, bone],
        );
    });
}

/// Far enough ahead that the grenade is never drawn (the client skips a
/// missile until its launch time).
const NEVER_DRAWN_MS: i32 = 1 << 30;

/// A grenade the scripts already have an entity for (an Acid Gat dart's
/// charge, a `magicbullet`; `darts`): followed, reset and gone off like a
/// thrown one, with no second entity and no `grenade_fire`.
pub(super) fn adopt(world: &mut World, p: &crate::ProjectileState, n: u32) {
    world.resource_mut::<Zm>().thrown.push(Thrown {
        id: p.id,
        entnum: p.entnum,
        weapon: p.weapon,
        n,
        stationary: false,
        hidden: false,
        logged: 0,
        // Not a gun's flying shot: its hits are the scripts' own.
        missile: false,
        shot: false,
        retrievable: false,
        owner: p.owner,
        amount: 0,
        last: p.pos.tr_base,
        passed: Vec::new(),
    });
}

pub(super) fn advance(world: &mut World, now: i64) {
    // A grenade that went off a while ago goes away now (see
    // GONE_SCRIPT_HOLD_MS).
    let free: Vec<gsc_t6::ObjRef> = {
        let mut zm = world.resource_mut::<Zm>();
        let (due, keep) = std::mem::take(&mut zm.thrown_free)
            .into_iter()
            .partition(|(_, at)| now >= *at);
        zm.thrown_free = keep;
        due.into_iter().map(|(o, _)| o).collect()
    };
    if !free.is_empty() {
        with_vm(world, |vm, world| {
            for o in free {
                if vm.alive(o) {
                    vm.free_object(world, o);
                }
                // A shot's entity stayed where it hit until now.
                world.resource_mut::<Zm>().ents.retain(|_, e| e.obj != Some(o));
            }
        });
    }
    let live: Vec<crate::ProjectileState> = crate::frame::collect_projectiles(world)
        .into_iter()
        .filter(|p| p.live)
        .collect();
    // Gone off (or cleaned up): `explode`, then `death`, then away.
    let gone: Vec<Thrown> = {
        let mut zm = world.resource_mut::<Zm>();
        let (gone, keep) = std::mem::take(&mut zm.thrown)
            .into_iter()
            .partition(|t| !live.iter().any(|p| p.id == t.id));
        zm.thrown = keep;
        gone
    };
    let notes = std::mem::take(&mut world.resource_mut::<Zm>().detonations);
    for t in gone {
        if t.missile {
            let at = notes.iter().find(|(id, _)| *id == t.id).map(|(_, at)| *at);
            if t.retrievable {
                if let Some(at) = at {
                    pass_through(world, &t, at);
                    land_on_triggers(world, &t, at);
                }
                retrieve(world, &t, at, now);
            } else {
                missile_gone(world, &t, at, now);
            }
            continue;
        }
        // A gun's shot: the models on its last stretch, to where it went off.
        if t.shot
            && !t.stationary
            && let Some(at) = notes.iter().find(|(id, _)| *id == t.id).map(|(_, at)| *at)
        {
            let hit = pass_through(world, &t, at);
            log_shot_hits(world, &t, hit);
        }
        let (obj, origin) = {
            let mut zm = world.resource_mut::<Zm>();
            let e = zm.ents.remove(&t.n);
            zm.movers.list.remove(&t.n);
            (e.as_ref().and_then(|e| e.obj), e.map_or([0.0; 3], |e| e.origin))
        };
        if std::env::var_os("IW4L_T6_SNDLOG").is_some() {
            diag::info!(Sim, "bo2zm t6 grenade ent{} went off at {:?} t={now}", t.n, origin.map(|x| x as i32));
        }
        let Some(o) = obj else { continue };
        if std::env::var_os("IW4L_T6_PLACE").is_some() {
            diag::info!(
                Sim,
                "bo2zm t6 thrown ent{} gone at ({:.0} {:.0} {:.0}): explode",
                t.n,
                origin[0],
                origin[1],
                origin[2]
            );
        }
        with_vm(world, |vm, world| {
            if vm.alive(o) {
                vm.notify_str(world, o, "explode", &[Value::Vec3(origin)]);
                vm.notify_str(world, o, "death", &[]);
            }
        });
        world.resource_mut::<Zm>().thrown_free.push((o, now + GONE_SCRIPT_HOLD_MS));
    }
    // A shot still flying that went off this tick keeps its point.
    {
        let mut zm = world.resource_mut::<Zm>();
        let keep: Vec<_> = notes
            .into_iter()
            .filter(|(id, _)| zm.thrown.iter().any(|t| t.id == *id))
            .collect();
        zm.detonations.extend(keep);
    }
    // Follow the rest; tell the scripts when one comes to rest.
    let tracked: Vec<Thrown> = world.resource::<Zm>().thrown.clone();
    for t in &tracked {
        let Some(p) = live.iter().find(|p| p.id == t.id) else {
            continue;
        };
        if t.missile && !world.resource::<Zm>().ents.contains_key(&t.n) {
            // Its script deleted it (the wind staff's, after 0.75 s): the
            // shot goes too.
            let mut f = frame(world);
            if f.projectile_mut_by_number(t.entnum).is_some_and(|p| p.id == t.id) {
                let _ = f.remove_projectile_by_number(t.entnum);
            }
            drop(f);
            world.resource_mut::<Zm>().thrown.retain(|x| x.id != t.id);
            continue;
        }
        let at = p.origin_at(now as i32);
        let (obj, hidden) = {
            let mut zm = world.resource_mut::<Zm>();
            let Some(e) = zm.ents.get_mut(&t.n) else {
                continue;
            };
            e.origin = at;
            e.angles = p.apos.tr_base;
            (e.obj, e.hidden)
        };
        if t.missile {
            let hit = pass_through(world, t, at);
            if t.retrievable && p.pos.tr_type == entity_iw4::TR_STATIONARY {
                land_on_triggers(world, t, at);
                retrieve(world, t, Some(at), now);
                continue;
            }
            let mut zm = world.resource_mut::<Zm>();
            if let Some(x) = zm.thrown.iter_mut().find(|x| x.id == t.id) {
                x.last = at;
                x.passed.extend(hit);
            }
            continue;
        }
        if t.shot && !t.stationary {
            let hit = pass_through(world, t, at);
            log_shot_hits(world, t, hit.clone());
            let mut zm = world.resource_mut::<Zm>();
            if let Some(x) = zm.thrown.iter_mut().find(|x| x.id == t.id) {
                x.last = at;
                x.passed.extend(hit);
            }
        }
        if hidden && !t.hidden {
            let mut f = frame(world);
            if let Some(p) = f.projectile_mut_by_number(t.entnum).filter(|p| p.id == t.id) {
                p.launch_time = (now as i32).saturating_add(NEVER_DRAWN_MS);
            }
        }
        // At rest: bounced to a stop on the floor, or stuck where it landed
        // (the claymore: its script waits for `stationary` before it arms).
        let still = p.grounded || p.pos.tr_type == entity_iw4::TR_STATIONARY;
        let rest = still && !t.stationary;
        {
            let mut zm = world.resource_mut::<Zm>();
            if let Some(x) = zm.thrown.iter_mut().find(|x| x.id == t.id) {
                x.hidden |= hidden;
                x.stationary |= still;
            }
        }
        if t.stationary && now - t.logged >= 1000 && std::env::var_os("IW4L_T6_SNDLOG").is_some() {
            log_zombies(world, t.n, at, now);
        }
        // Test log: a gun's shot in flight (the Sliquifier's bolt), once a second.
        if !t.stationary
            && now - t.logged >= 1000
            && std::env::var_os("IW4L_T6_SNDLOG").is_some()
            && frame(world)
                .combat_facts_for(t.weapon)
                .is_some_and(|f| f.weap_type != weapon_iw4::WEAPTYPE_GRENADE)
        {
            diag::info!(
                Sim,
                "bo2zm t6 grenade ent{} flying at {:?} tr_type {} grounded {} stuck {:?} detonate {:?} cleanup {} t={now}",
                t.n,
                at.map(|x| x as i32),
                p.pos.tr_type,
                p.grounded,
                p.stuck_pane,
                p.detonate_at_ms,
                p.cleanup_at_ms
            );
            if let Some(x) = world.resource_mut::<Zm>().thrown.iter_mut().find(|x| x.id == t.id) {
                x.logged = now;
            }
        }
        if rest && let Some(o) = obj {
            if std::env::var_os("IW4L_T6_PLACE").is_some() {
                diag::info!(
                    Sim,
                    "bo2zm t6 thrown ent{} stationary at ({:.0} {:.0} {:.0})",
                    t.n,
                    at[0],
                    at[1],
                    at[2]
                );
            }
            with_vm(world, |vm, world| {
                if vm.alive(o) {
                    vm.notify_str(world, o, "stationary", &[]);
                }
            });
        }
    }
    // New throws: an entity for each and `grenade_fire` to the thrower.
    for p in &live {
        if world.resource::<Zm>().thrown.iter().any(|t| t.id == p.id) {
            continue;
        }
        let Some(player) = world.resource::<Zm>().players.get(&p.owner.0).map(|x| x.obj) else {
            continue;
        };
        // A thrown grenade, or a gun's shot that becomes a grenade for the
        // scripts (the Sliquifier's slip bolt: its script waits for this
        // `grenade_fire`, then the bolt's `explode`, for the puddle).
        let facts = frame(world).combat_facts_for(p.weapon);
        let (as_weapon, missile) = match facts {
            Some(f) if f.weap_type == weapon_iw4::WEAPTYPE_GRENADE => (p.weapon, false),
            Some(f) if f.spawned_grenade != 0 => (f.spawned_grenade, false),
            // A gun's flying shot (the staffs): `missile_fire`.
            Some(f) if f.weap_type == weapon_iw4::WEAPTYPE_PROJECTILE => (p.weapon, true),
            _ => continue,
        };
        let name = super::weapon_text(world, as_weapon);
        if as_weapon != p.weapon
            && std::env::var_os("IW4L_T6_SNDLOG").is_some()
            && !BOLT_FACTS_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            for w in [p.weapon, as_weapon] {
                let text = super::weapon_text(world, w);
                let f = frame(world);
                let c = f.combat_facts_for(w).map(|c| (c.weap_type, c.weap_class, c.spawned_grenade));
                let r = f.equipment_row_raw(w);
                diag::info!(Sim, "bo2zm t6 bolt facts {text} ({w}): type/class/spawned {c:?} {r:?}");
            }
        }
        let at = p.origin_at(now as i32);
        let shot = !missile && as_weapon != p.weapon;
        let amount = if missile || shot {
            crate::equipment::projectile_impact_facts(&frame(world), p.weapon).map_or(0, |f| f.0)
        } else {
            0
        };
        let n = world.resource_mut::<Zm>().alloc_entnum();
        with_vm(world, |vm, world| {
            let obj = vm.alloc_object(ObjKind::Entity(n));
            world.resource_mut::<Zm>().ents.insert(
                n,
                Ent {
                    obj: Some(obj),
                    classname: if missile { "rocket" } else { "grenade" }.to_owned(),
                    origin: at,
                    angles: p.apos.tr_base,
                    ..Default::default()
                },
            );
            let owner = vm.strings.intern("owner");
            vm.set_field(world, obj, owner, Value::Object(player));
            let weaponname = vm.strings.intern("weaponname");
            let w = vm.string(&name);
            vm.set_field(world, obj, weaponname, w.clone());
            let note = if missile { "missile_fire" } else { "grenade_fire" };
            vm.notify_str(world, player, note, &[Value::Object(obj), w]);
        });
        if std::env::var_os("IW4L_T6_SNDLOG").is_some()
            || std::env::var_os("IW4L_T6_PLACE").is_some()
            || (missile && hitlog())
        {
            diag::info!(Sim, "bo2zm t6 {} {name} ent{n}", if missile { "missile_fire" } else { "grenade_fire" });
        }
        world.resource_mut::<Zm>().thrown.push(Thrown {
            id: p.id,
            entnum: p.entnum,
            weapon: p.weapon,
            n,
            stationary: false,
            hidden: false,
            logged: 0,
            missile,
            shot,
            retrievable: missile && facts.is_some_and(|f| f.retrievable),
            owner: p.owner,
            amount,
            // From where it was fired, so its first stretch counts.
            last: if missile || shot { p.pos.tr_base } else { at },
            passed: Vec::new(),
        });
    }
}

/// Test log: how near the closest zombies are to a grenade at rest (the
/// Monkey Bomb draws them in).
fn log_zombies(world: &mut World, n: u32, at: [f32; 3], now: i64) {
    let mut zm = world.resource_mut::<Zm>();
    if let Some(x) = zm.thrown.iter_mut().find(|x| x.n == n) {
        x.logged = now;
    }
    let mut d: Vec<i32> = zm
        .actors
        .keys()
        .filter_map(|a| zm.ents.get(a))
        .map(|e| {
            let v = [e.origin[0] - at[0], e.origin[1] - at[1], e.origin[2] - at[2]];
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() as i32
        })
        .collect();
    d.sort_unstable();
    d.truncate(6);
    diag::info!(Sim, "bo2zm t6 grenade ent{n} at rest, nearest zombies {d:?} t={now}");
    // The monkey the script put on it: where, whether shown, its clip.
    let models: Vec<String> = zm
        .ents
        .iter()
        .filter(|(_, e)| e.model.contains("monkey"))
        .map(|(k, e)| {
            format!(
                "ent{k} {} at {:?} hidden={} invisible_to_all={} link={:?} anim={:?}",
                e.model,
                e.origin.map(|x| x as i32),
                e.hidden,
                e.invisible_to_all,
                e.link.map(|l| l.0),
                e.anim
            )
        })
        .collect();
    diag::info!(Sim, "bo2zm t6 grenade ent{n} monkey models {models:?}");
    // What the scripts made of it: the grenade as a point of interest, and
    // whether the zombies were sent to it (`enemyoverride`) and where.
    let probe = with_vm(world, |vm, world| {
        let zm = world.resource::<Zm>();
        let text = |vm: &gsc_t6::Vm<World>, o: gsc_t6::ObjRef, f: &str| -> String {
            let k = vm.strings.find(f);
            match k.map(|k| vm.raw_field(o, k)) {
                Some(Value::Array(a)) => format!("[{} items]", a.read().len()),
                Some(v) => vm.to_text(&v),
                None => "-".to_owned(),
            }
        };
        let g = zm.ents.get(&n).and_then(|e| e.obj).map(|o| {
            ["script_noteworthy", "poi_active", "poi_radius", "num_poi_attracts", "attract_to_origin", "attractor_array", "attractor_positions", "claimed_attractor_positions"]
                .map(|f| format!("{f}={}", text(vm, o, f)))
                .join(" ")
        });
        let mut zs: Vec<(i32, String)> = zm
            .actors
            .iter()
            .filter(|(_, a)| a.alive)
            .filter_map(|(k, a)| {
                let e = zm.ents.get(k)?;
                let o = a.obj?;
                let v = [e.origin[0] - at[0], e.origin[1] - at[1]];
                let dist = (v[0] * v[0] + v[1] * v[1]).sqrt() as i32;
                Some((
                    dist,
                    format!(
                        "{dist}: override={} ignoreall={} goal={:?} script={} ai_state={}",
                        text(vm, o, "enemyoverride"),
                        text(vm, o, "ignoreall"),
                        a.goal.map(|g| g.map(|x| x as i32)),
                        a.script,
                        text(vm, o, "ai_state")
                    ),
                ))
            })
            .collect();
        zs.sort_by_key(|z| z.0);
        (g, zs.into_iter().take(3).map(|z| z.1).collect::<Vec<_>>())
    });
    if let Some((g, zs)) = probe {
        diag::info!(Sim, "bo2zm t6 grenade ent{n} as a point of interest: {g:?}; zombies {zs:?}");
    }
}

/// `grenade resetmissiledetonationtime([seconds])`: the fuse starts again
/// (the weapon's own fuse when no time is given).
pub(super) fn reset_fuse(world: &mut World, n: u32, secs: Option<f32>, now: i64) {
    let Some(t) = world.resource::<Zm>().thrown.iter().find(|t| t.n == n).cloned() else {
        return;
    };
    let mut f = frame(world);
    let fuse = match secs {
        Some(s) => (s * 1000.0) as i32,
        None => f.equipment_facts_for(t.weapon).map_or(0, |x| x.fuse_time_ms),
    };
    if fuse <= 0 {
        return;
    }
    if let Some(p) = f.projectile_mut_by_number(t.entnum).filter(|p| p.id == t.id) {
        p.detonate_at_ms = Some((now as i32).saturating_add(fuse));
        p.cleanup_at_ms = p.cleanup_at_ms.max((now as i32).saturating_add(fuse + 1000));
    }
}

/// `grenade detonate([attacker])`: it goes off now, through the engine's own
/// fuse (a claymore a zombie walked into).
pub(super) fn detonate_now(world: &mut World, n: u32, now: i64) {
    let Some(t) = world.resource::<Zm>().thrown.iter().find(|t| t.n == n).cloned() else {
        return;
    };
    if hitlog() {
        diag::info!(Sim, "bo2zm t6 detonate ent{n} t{now}");
    }
    let mut f = frame(world);
    if let Some(p) = f.projectile_mut_by_number(t.entnum).filter(|p| p.id == t.id) {
        p.detonate_at_ms = Some(now as i32);
        p.cleanup_at_ms = p.cleanup_at_ms.max((now as i32).saturating_add(1000));
    }
}
