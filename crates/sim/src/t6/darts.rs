//! `magicbullet` (BO2's MagicBullet): a shot the scripts fire from anywhere.
//!
//! Mob of the Dead's Acid Gat fires its darts this way
//! (`_zm_weap_blundersplat`: three `blundersplat_bullet_zm` a pull, each at
//! a zombie in front of him, else where he looks). A dart flies at the
//! weapon's projectile speed; where it hits it leaves the weapon's spawned
//! grenade (BO2's szSpawnedGrenadeWeaponName,
//! `blundersplat_explosive_dart_zm`) stuck in the zombie, or in the wall.
//! The zombie is hurt by the dart (the script tags him and waits on the
//! charge's fuse), the dart dies (the script then sets the charge's fuse)
//! and the charge goes off on it.
//!
//! A bullet weapon (Mob's tower trap) hits the first zombie on its line at
//! once. A missile or grenade weapon is launched by the engine and tracked
//! like a thrown grenade, so `explode` and `death` reach its entity (the
//! upgraded tower trap, Origins' fire staff).
//!
//! IW4L_T6_ACID=1 logs each dart, where it lands and its charge.

use bevy_ecs::prelude::{Resource, World};
use gsc_t6::{ObjKind, Value, Vm};

use super::{Ent, Zm, arg, frame, text, vec3, with_vm};
use crate::world::ClientId;

/// The shots that leave a grenade where they hit: the shot, the grenade
/// (BO2's szSpawnedGrenadeWeaponName, read from the map's weapon file) and
/// the model both fly as (its projectile model). Highrise's
/// `spawned_grenade` combat fact can replace this once merged.
const SPAWNS_GRENADE: &[(&str, &str, &str)] = &[(
    "blundersplat_bullet_zm",
    "blundersplat_explosive_dart_zm",
    "t6_wpn_zmb_projectile_blundergat",
)];

/// A zombie as a shot sees him: an upright cylinder on his origin.
const BODY_RADIUS: f32 = 18.0;
const BODY_HEIGHT: f32 = 72.0;
/// How far a shot flies past the point it was aimed at.
const RANGE: f32 = 8192.0;
/// Far enough ahead that a projectile is never drawn (its entity's model
/// is drawn instead, following the zombie it is stuck in).
const NEVER_DRAWN_MS: i32 = 1 << 30;

struct Flying {
    /// The dart's entity.
    n: u32,
    owner: Option<ClientId>,
    weapon: String,
    charge: String,
    model: String,
    from: [f32; 3],
    dir: [f32; 3],
    speed: f32,
    /// How far it has come, and where the world stops it.
    at: f32,
    wall: f32,
}

struct Stuck {
    /// The charge's entity and its projectile.
    n: u32,
    entnum: i32,
    id: crate::ProjectileId,
    /// The zombie it is in.
    zombie: Option<u32>,
}

#[derive(Resource, Default)]
pub(crate) struct Darts {
    flying: Vec<Flying>,
    stuck: Vec<Stuck>,
}

fn log_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("IW4L_T6_ACID").is_some())
}

fn darts(world: &mut World) -> bevy_ecs::prelude::Mut<'_, Darts> {
    if !world.contains_resource::<Darts>() {
        world.insert_resource(Darts::default());
    }
    world.resource_mut::<Darts>()
}

fn add(a: [f32; 3], b: [f32; 3], s: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + b[i] * s)
}

fn normalized(d: [f32; 3]) -> Option<[f32; 3]> {
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    (l > 0.001).then(|| d.map(|x| x / l))
}

/// How far the static world lets a shot go from `from` along `dir`.
fn wall_distance(world: &mut World, from: [f32; 3], dir: [f32; 3]) -> f32 {
    let end = add(from, dir, RANGE);
    let t = frame(world).trace_static_world(
        from,
        end,
        [0.0; 3],
        [0.0; 3],
        crate::bullet_collision::MASK_SHOT,
    );
    RANGE * t.fraction.clamp(0.0, 1.0)
}

/// The first living zombie on the line from `from` along `dir` between
/// `a` and `b` units out: (his entity, how far out, where on him).
fn first_zombie(
    world: &World,
    from: [f32; 3],
    dir: [f32; 3],
    a: f32,
    b: f32,
) -> Option<(u32, f32, [f32; 3])> {
    let zm = world.resource::<Zm>();
    let flat = (dir[0] * dir[0] + dir[1] * dir[1]).max(1e-6);
    zm.actors
        .iter()
        .filter(|(_, x)| x.alive)
        .filter_map(|(n, _)| {
            let o = zm.ents.get(n)?.origin;
            // The closest pass to his upright axis, seen from above.
            let s = ((o[0] - from[0]) * dir[0] + (o[1] - from[1]) * dir[1]) / flat;
            let s = s.clamp(a, b);
            let p = add(from, dir, s);
            let off = [p[0] - o[0], p[1] - o[1]];
            let d = (off[0] * off[0] + off[1] * off[1]).sqrt();
            // Aimed at his feet (gettagorigin gives his origin) still hits.
            let z = p[2] - o[2];
            (d <= BODY_RADIUS && (-12.0..=BODY_HEIGHT).contains(&z)).then(|| {
                // On him: at his surface toward the shot, at a body height.
                let out = if d > 0.5 {
                    [off[0] / d, off[1] / d]
                } else {
                    [-dir[0], -dir[1]]
                };
                let at = [
                    o[0] + out[0] * 10.0,
                    o[1] + out[1] * 10.0,
                    o[2] + z.clamp(24.0, 60.0),
                ];
                (*n, s, at)
            })
        })
        .min_by(|x, y| x.1.total_cmp(&y.1))
}

fn player_obj(world: &World, c: Option<ClientId>) -> Value {
    c.and_then(|c| world.resource::<Zm>().players.get(&c.0).map(|p| p.obj))
        .map_or(Value::Undefined, Value::Object)
}

/// A script entity for a shot or a charge.
fn new_ent(
    vm: &mut Vm<World>,
    world: &mut World,
    classname: &str,
    model: &str,
    origin: [f32; 3],
    angles: [f32; 3],
    owner: Option<ClientId>,
    weapon: &str,
) -> (u32, gsc_t6::ObjRef) {
    let n = world.resource_mut::<Zm>().alloc_entnum();
    let obj = vm.alloc_object(ObjKind::Entity(n));
    world.resource_mut::<Zm>().ents.insert(
        n,
        Ent {
            obj: Some(obj),
            classname: classname.to_owned(),
            model: model.to_owned(),
            origin,
            angles,
            ..Default::default()
        },
    );
    let who = player_obj(world, owner);
    if !matches!(who, Value::Undefined) {
        let k = vm.strings.intern("owner");
        vm.set_field(world, obj, k, who);
    }
    let k = vm.strings.intern("weaponname");
    let w = vm.string(weapon);
    vm.set_field(world, obj, k, w);
    (n, obj)
}

pub(super) fn bind(vm: &mut Vm<World>) {
    vm.bind("magicbullet", false, |vm, world, _, a| {
        let weapon = text(vm, a, 0);
        let start = vec3(a, 1)?;
        let end = vec3(a, 2)?;
        let owner = super::client(vm, world, arg(a, 3)).ok();
        fire(vm, world, &weapon, start, end, owner)
    });
    // BO2's ScrCmd_MagicGrenadeType: a grenade from the calling entity (a
    // player, or Brutus dropping his smoke); the fuse is in seconds.
    vm.bind("magicgrenadetype", true, |vm, world, s, a| {
        let weapon = text(vm, a, 0);
        let origin = vec3(a, 1)?;
        let velocity = vec3(a, 2)?;
        let fuse_ms = if a.len() > 3 {
            (super::num(a, 3)? * 1000.0) as i32
        } else {
            5000
        };
        let owner = super::client(vm, world, s).ok();
        magic_grenade(vm, world, &weapon, origin, velocity, fuse_ms, owner)
    });
}

fn magic_grenade(
    vm: &mut Vm<World>,
    world: &mut World,
    weapon: &str,
    origin: [f32; 3],
    velocity: [f32; 3],
    fuse_ms: i32,
    owner: Option<ClientId>,
) -> Result<Value, String> {
    let w = super::weapon(world, weapon)?;
    let grenade = frame(world)
        .combat_facts_for(w)
        .is_some_and(|f| f.weap_type == weapon_iw4::WEAPTYPE_GRENADE);
    if !grenade {
        let why = "MagicGrenadeManual invalid weapon type specified";
        return Err(format!("{why} ({weapon})"));
    }
    // Not a player's (Brutus): it goes out as the first player's.
    let shooter = owner.or_else(|| {
        let zm = world.resource::<Zm>();
        zm.players.keys().next().map(|c| ClientId(*c))
    });
    let Some(shooter) = shooter else {
        return Ok(Value::Undefined);
    };
    let tick = super::tick(world);
    let launched = crate::missile::magic_grenade(
        &mut frame(world),
        tick,
        shooter,
        w,
        origin,
        velocity,
        fuse_ms,
    );
    let p = match launched {
        Ok(p) => p,
        Err(e) => {
            diag::warn!(Sim, "bo2zm t6 magicgrenadetype {weapon}: {e}");
            return Ok(Value::Undefined);
        }
    };
    let angles = p.apos.tr_base;
    let (n, obj) = new_ent(vm, world, "grenade", "", origin, angles, owner, weapon);
    super::grenades::adopt(world, &p, n);
    diag::info!(
        Sim,
        "bo2zm t6 magicgrenadetype {weapon}: ent{n} from ({:.0} {:.0} {:.0}), fuse {fuse_ms} ms",
        origin[0],
        origin[1],
        origin[2]
    );
    Ok(Value::Object(obj))
}

fn fire(
    vm: &mut Vm<World>,
    world: &mut World,
    weapon: &str,
    start: [f32; 3],
    end: [f32; 3],
    owner: Option<ClientId>,
) -> Result<Value, String> {
    let Some(dir) = normalized(std::array::from_fn(|i| end[i] - start[i])) else {
        return Ok(Value::Undefined);
    };
    let angles = math_iw4::vect_to_angles(dir);
    if let Some((_, charge, model)) = SPAWNS_GRENADE.iter().find(|(w, ..)| *w == weapon) {
        let w = super::weapon(world, weapon)?;
        let speed = frame(world)
            .missile_launch_facts(w)
            .map(|f| f.projectile_speed as f32)
            .filter(|s| *s > 0.0)
            .unwrap_or(3000.0);
        let wall = wall_distance(world, start, dir);
        let (n, obj) = new_ent(vm, world, "rocket", model, start, angles, owner, weapon);
        if log_on() {
            diag::info!(
                Sim,
                "bo2zm t6 acid: dart ent{n} from ({:.0} {:.0} {:.0}) at ({:.0} {:.0} {:.0}), wall {wall:.0} out",
                start[0],
                start[1],
                start[2],
                end[0],
                end[1],
                end[2]
            );
        }
        darts(world).flying.push(Flying {
            n,
            owner,
            weapon: weapon.to_owned(),
            charge: (*charge).to_owned(),
            model: (*model).to_owned(),
            from: start,
            dir,
            speed,
            at: 0.0,
            wall,
        });
        return Ok(Value::Object(obj));
    }
    let w = super::weapon(world, weapon)?;
    let Some(facts) = frame(world).combat_facts_for(w) else {
        return Err(format!("magicbullet: {weapon} has no weapon data"));
    };
    if facts.weap_type == weapon_iw4::WEAPTYPE_BULLET {
        // A bullet: the first zombie on its line, at once.
        let wall = wall_distance(world, start, dir);
        if let Some((z, _, at)) = first_zombie(world, start, dir, 0.0, wall) {
            let who = player_obj(world, owner);
            super::actors::damage(
                vm,
                world,
                z,
                who.clone(),
                who,
                facts.damage,
                0,
                "MOD_RIFLE_BULLET",
                weapon,
                at,
                dir,
                "torso_upper",
            );
        }
        return Ok(Value::Undefined);
    }
    // A missile or a grenade: the engine flies it. With no one firing it
    // (a trap), it goes out as the first player's.
    let shooter = owner.or_else(|| {
        let zm = world.resource::<Zm>();
        zm.players.keys().next().map(|c| ClientId(*c))
    });
    let Some(shooter) = shooter else {
        return Ok(Value::Undefined);
    };
    let tick = super::tick(world);
    let launched = crate::missile::magic_bullet(&mut frame(world), tick, shooter, w, start, end);
    let p = match launched {
        Ok(p) => p,
        Err(e) => {
            diag::warn!(Sim, "bo2zm t6 magicbullet {weapon}: {e}");
            return Ok(Value::Undefined);
        }
    };
    let class = if facts.weap_type == weapon_iw4::WEAPTYPE_GRENADE {
        "grenade"
    } else {
        "rocket"
    };
    let (n, obj) = new_ent(vm, world, class, "", start, angles, owner, weapon);
    super::grenades::adopt(world, &p, n);
    Ok(Value::Object(obj))
}

/// Each tick (before the grenades): darts fly and land; charges stay in the
/// zombie they are stuck in.
pub(super) fn advance(world: &mut World, now: i64) {
    if !world.contains_resource::<Darts>() {
        return;
    }
    let step = crate::MATCH_TICK_MS as f32 / 1000.0;
    let flying = std::mem::take(&mut world.resource_mut::<Darts>().flying);
    let mut keep = Vec::new();
    for mut d in flying {
        let next = (d.at + d.speed * step).min(d.wall);
        if let Some((z, s, at)) = first_zombie(world, d.from, d.dir, d.at, next) {
            land(world, &d, add(d.from, d.dir, s), Some((z, at)), now);
        } else if next >= d.wall {
            land(world, &d, add(d.from, d.dir, d.wall - 1.0), None, now);
        } else {
            d.at = next;
            let p = add(d.from, d.dir, next);
            if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&d.n) {
                e.origin = p;
            }
            keep.push(d);
        }
    }
    darts(world).flying.extend(keep);
    // A charge sits where its entity is (moved with the zombie by its link).
    let stuck = std::mem::take(&mut world.resource_mut::<Darts>().stuck);
    let mut keep = Vec::new();
    for s in stuck {
        let pose = world.resource::<Zm>().ents.get(&s.n).map(|e| (e.origin, e.angles));
        let Some((o, a)) = pose else {
            if log_on() {
                diag::info!(Sim, "bo2zm t6 acid: charge ent{} went off", s.n);
            }
            continue;
        };
        let mut f = frame(world);
        let Some(p) = f.projectile_mut_by_number(s.entnum).filter(|p| p.id == s.id) else {
            continue;
        };
        hold(p, o, a, now as i32);
        keep.push(s);
    }
    darts(world).stuck.extend(keep);
}

/// A projectile held still at `o` facing `a`.
fn hold(p: &mut crate::ProjectileState, o: [f32; 3], a: [f32; 3], now: i32) {
    p.origin = o;
    p.velocity = [0.0; 3];
    p.grounded = true;
    p.pos.tr_type = entity_iw4::TR_STATIONARY;
    p.pos.tr_time = now;
    p.pos.tr_duration = 0;
    p.pos.tr_base = o;
    p.pos.tr_delta = [0.0; 3];
    p.apos.tr_type = entity_iw4::TR_STATIONARY;
    p.apos.tr_time = now;
    p.apos.tr_duration = 0;
    p.apos.tr_base = a;
    p.apos.tr_delta = [0.0; 3];
}

/// A dart lands at `point`, in a zombie (`hit`: him and where on him) or
/// the wall: its charge sticks there, the zombie is hurt, the dart dies.
fn land(world: &mut World, d: &Flying, point: [f32; 3], hit: Option<(u32, [f32; 3])>, now: i64) {
    let at = hit.map_or(point, |(_, at)| at);
    let angles = math_iw4::vect_to_angles(d.dir);
    let shooter = d.owner.or_else(|| {
        let zm = world.resource::<Zm>();
        zm.players.keys().next().map(|c| ClientId(*c))
    });
    let charge = super::weapon(world, &d.charge).ok().zip(shooter).and_then(|(w, c)| {
        let tick = super::tick(world);
        let ahead = add(at, d.dir, 1.0);
        match crate::missile::magic_bullet(&mut frame(world), tick, c, w, at, ahead) {
            Ok(p) => Some(p),
            Err(e) => {
                diag::warn!(Sim, "bo2zm t6 acid: no charge {}: {e}", d.charge);
                None
            }
        }
    });
    if let Some(p) = &charge {
        let mut f = frame(world);
        if let Some(x) = f.projectile_mut_by_number(p.entnum).filter(|x| x.id == p.id) {
            hold(x, at, angles, now as i32);
            x.launch_time = (now as i32).saturating_add(NEVER_DRAWN_MS);
        }
    }
    let weapon_damage = super::weapon(world, &d.weapon)
        .ok()
        .and_then(|w| frame(world).combat_facts_for(w))
        .map_or(1, |f| f.damage);
    with_vm(world, |vm, world| {
        let dart_obj = world
            .resource_mut::<Zm>()
            .ents
            .remove(&d.n)
            .and_then(|e| e.obj);
        if let Some(p) = &charge {
            let (n, _) = new_ent(vm, world, "grenade", &d.model, at, angles, d.owner, &d.charge);
            // In him: it rides along (linkto's offset, kept relative).
            if let Some((z, _)) = hit {
                let mut zm = world.resource_mut::<Zm>();
                let pose = zm.ents.get(&z).map(|e| (e.origin, e.angles));
                if let (Some((po, pa)), Some(e)) = (pose, zm.ents.get_mut(&n)) {
                    let (f, r, u) = gsc_t6::math::angle_vectors(pa);
                    let o = gsc_t6::math::sub(at, po);
                    let off = [
                        gsc_t6::math::dot(o, f),
                        -gsc_t6::math::dot(o, r),
                        gsc_t6::math::dot(o, u),
                    ];
                    let aoff = std::array::from_fn(|i| angles[i] - pa[i]);
                    e.link = Some((z, off, aoff));
                }
            }
            super::grenades::adopt(world, p, n);
            darts(world).stuck.push(Stuck {
                n,
                entnum: p.entnum,
                id: p.id,
                zombie: hit.map(|(z, _)| z),
            });
            if log_on() {
                diag::info!(
                    Sim,
                    "bo2zm t6 acid: dart ent{} lands in {} at ({:.0} {:.0} {:.0}); charge ent{n} (fuse {:?} ms)",
                    d.n,
                    hit.map_or("the wall".to_owned(), |(z, _)| format!("zombie ent{z}")),
                    at[0],
                    at[1],
                    at[2],
                    p.detonate_at_ms.map(|t| i64::from(t) - now)
                );
            }
        }
        let inflictor = dart_obj.map_or(Value::Undefined, Value::Object);
        if let Some((z, _)) = hit {
            let who = player_obj(world, d.owner);
            super::actors::damage(
                vm,
                world,
                z,
                inflictor.clone(),
                who,
                weapon_damage,
                0,
                "MOD_IMPACT",
                &d.weapon,
                at,
                d.dir,
                "torso_upper",
            );
        }
        if let Some(o) = dart_obj {
            if vm.alive(o) {
                vm.notify_str(world, o, "death", &[]);
            }
            world.resource_mut::<Zm>().thrown_free.push((o, now + 1));
        }
    });
}

/// IW4L_T6_ACID: every 5 s, the charges in zombies and how many of those
/// zombies still stand.
pub(super) fn log(world: &mut World, now: i64) {
    if !log_on() || !world.contains_resource::<Darts>() {
        return;
    }
    let tick = i64::from(crate::MATCH_TICK_MS);
    if now % 5000 >= tick {
        return;
    }
    let (flying, stuck, standing) = {
        let darts = world.resource::<Darts>();
        let zm = world.resource::<Zm>();
        let standing = darts
            .stuck
            .iter()
            .filter_map(|s| s.zombie)
            .filter(|z| zm.actors.get(z).is_some_and(|a| a.alive))
            .count();
        (darts.flying.len(), darts.stuck.len(), standing)
    };
    diag::info!(
        Sim,
        "bo2zm t6 acid: at {}s darts flying {flying}, charges {stuck} ({standing} in zombies still up)",
        now / 1000
    );
}
