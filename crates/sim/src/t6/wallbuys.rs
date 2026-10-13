//! Wall buys as the player sees them, which BO2 draws from its client
//! scripts (clientscripts/mp/zombies/_zm_weapons.csc), done here on the
//! server: each wall buy the server put in this mode and start
//! (`level._spawned_wallbuys`) gets its chalk effect
//! (`level._effect[<weapon>_fx]`, the M14's by default; Buried's built wall
//! buys a "?") once the player is in; when the scripts flip its world
//! clientfield (`<weapon>_<origin>` = 1, bought) the gun's model (the target
//! struct's) slides out of the wall to its place over a second, from 8 units
//! along its right turned by the target struct's `script_vector` (the
//! Claymore's (0, -90, 0): its model faces out of the wall where a gun's lies
//! along it). A built wall buy learns its gun from `<field>_idx` (its index
//! in `level.buildable_wallbuy_weapons`, plus one): the "?" goes, that gun's
//! chalk plays, and the gun's own model is the one that slides.

use std::collections::HashMap;

use bevy_ecs::prelude::World;
use gsc_t6::{ObjKind, ObjRef, Value, Vm};

use super::{Ent, Zm, with_vm};

#[derive(Clone, Debug, Default)]
pub(crate) struct WallBuys {
    pub chalk_done: bool,
    /// World clientfields the scripts set (name -> value).
    pub world_fields: HashMap<String, i32>,
    /// Wall buys whose gun is out (by clientfield name).
    pub shown: HashMap<String, u32>,
    /// Each wall buy's chalk: its effect and where (by clientfield name).
    pub chalk: HashMap<String, (String, [f32; 3])>,
    /// Built wall buys' guns (by clientfield name).
    pub built: HashMap<String, String>,
}

fn field(vm: &mut Vm<World>, o: ObjRef, name: &str) -> Value {
    let f = vm.intern(name);
    vm.raw_field(o, f)
}

/// The struct kinds the server's `init_spawnable_weapon_upgrade` gathers as
/// wall buys: guns and grenades, the Bowie knife, the sickle, Galvaknuckles,
/// built wall buys and Claymores (Nuketown has all but the sickle and the
/// built ones).
fn is_wallbuy(targetname: &str) -> bool {
    matches!(
        targetname,
        "weapon_upgrade"
            | "bowie_upgrade"
            | "sickle_upgrade"
            | "tazer_upgrade"
            | "buildable_wallbuy"
            | "claymore_purchase"
    )
}

/// Every map struct (level.struct).
fn structs(vm: &mut Vm<World>) -> Vec<ObjRef> {
    let all = field(vm, vm.level, "struct");
    let Value::Array(arr) = all else {
        return Vec::new();
    };
    arr.snapshot()
        .values_in_order()
        .filter_map(|v| match v {
            Value::Object(o) => Some(*o),
            _ => None,
        })
        .collect()
}

/// The wall buys the server put in this mode and start
/// (`level._spawned_wallbuys`: those whose `script_noteworthy` is empty or
/// names `<gametype>_<location>`, no Claymore or Semtex in headshots only),
/// the ones the client scripts pick too. TranZit's bus adds its own trigger,
/// an entity with no chalk: only structs count. Every wall buy struct if the
/// server never made the list.
fn spawned(vm: &mut Vm<World>) -> Vec<ObjRef> {
    let list = field(vm, vm.level, "_spawned_wallbuys");
    let Value::Array(arr) = list else {
        return structs(vm)
            .into_iter()
            .filter(|&s| {
                let tn = field(vm, s, "targetname");
                is_wallbuy(&vm.to_text(&tn))
            })
            .collect();
    };
    arr.snapshot()
        .values_in_order()
        .filter_map(|v| match v {
            Value::Object(o) if matches!(vm.kind(*o), Some(ObjKind::Struct)) => Some(*o),
            _ => None,
        })
        .collect()
}

/// A wall buy's clientfield name: `<weapon>_<origin>`.
fn label(vm: &mut Vm<World>, s: ObjRef) -> String {
    let w = field(vm, s, "zombie_weapon_upgrade");
    let o = field(vm, s, "origin");
    format!("{}_{}", vm.to_text(&w), vm.to_text(&o))
}

/// `level._effect[key]`.
fn effect_id(vm: &mut Vm<World>, key: &str) -> Option<i32> {
    let table = field(vm, vm.level, "_effect");
    let Value::Array(a) = &table else {
        return None;
    };
    let k = gsc_t6::Key::Str(vm.intern(key));
    a.get(&k).and_then(|v| v.as_int())
}

/// A wall buy's chalk: its gun's (`<weapon>_fx`, the M14's when it has
/// none); a built wall buy's "?" until it is built, then its gun's.
fn chalk_fx(
    vm: &mut Vm<World>,
    targetname: &str,
    weapon: &str,
    built: Option<&str>,
) -> Option<i32> {
    match (targetname, built) {
        ("buildable_wallbuy", None) => effect_id(vm, "dynamic_wallbuy_fx"),
        ("buildable_wallbuy", Some(w)) => effect_id(vm, &format!("{w}_fx")),
        _ => effect_id(vm, &format!("{weapon}_fx")),
    }
    .or_else(|| effect_id(vm, "m14_zm_fx"))
}

/// Each tick: the chalk once a player has begun (and a moment after, so
/// his client is listening).
pub(crate) fn advance(world: &mut World, now: i64) {
    if world.resource::<Zm>().wallbuys.chalk_done {
        return;
    }
    let begun = world.resource::<Zm>().players.values().any(|p| p.begun);
    if !begun {
        return;
    }
    let since = {
        let mut zm = world.resource_mut::<Zm>();
        let t = *zm.wallbuy_since.get_or_insert(now);
        now - t
    };
    if since < 1500 {
        return;
    }
    world.resource_mut::<Zm>().wallbuys.chalk_done = true;
    let built = world.resource::<Zm>().wallbuys.built.clone();
    let effects: Vec<(String, i32, [f32; 3], [f32; 3], [f32; 3])> = with_vm(world, |vm, _| {
        let mut out = Vec::new();
        for s in spawned(vm) {
            let tn = field(vm, s, "targetname");
            let tn = vm.to_text(&tn);
            let w = field(vm, s, "zombie_weapon_upgrade");
            let w = vm.to_text(&w);
            let Some(origin) = field(vm, s, "origin").as_vec3() else {
                continue;
            };
            let angles = field(vm, s, "angles").as_vec3().unwrap_or([0.0; 3]);
            let label = label(vm, s);
            let Some(fx) = chalk_fx(vm, &tn, &w, built.get(&label).map(String::as_str)) else {
                continue;
            };
            let (f, _, u) = gsc_t6::math::angle_vectors(angles);
            out.push((label, fx, origin, f, u));
        }
        out
    })
    .unwrap_or_default();
    diag::info!(
        Sim,
        "bo2zm wall buy chalk: {} ({:?})",
        effects.len(),
        effects.iter().map(|e| e.0.as_str()).collect::<Vec<_>>()
    );
    for (label, fx, origin, forward, up) in effects {
        if let Some(name) = super::natives_fx::fx_name(world, fx) {
            super::natives_fx::effect_axis(world, &name, origin, forward, up);
            world
                .resource_mut::<Zm>()
                .wallbuys
                .chalk
                .insert(label, (name, origin));
        }
    }
}

/// A built wall buy learned its gun (`<field>_idx`), the first time only:
/// its "?" goes and its gun's chalk plays (before the chalk is drawn,
/// `advance` draws that one).
fn built(vm: &mut Vm<World>, world: &mut World, base: &str, idx: i32) {
    if world.resource::<Zm>().wallbuys.built.contains_key(base) {
        return;
    }
    let mut at = None;
    for s in spawned(vm) {
        let tn = field(vm, s, "targetname");
        if vm.to_text(&tn) == "buildable_wallbuy" && label(vm, s) == base {
            at = Some(s);
            break;
        }
    }
    let Some(s) = at else {
        return;
    };
    let list = field(vm, vm.level, "buildable_wallbuy_weapons");
    let Value::Array(arr) = list else {
        return;
    };
    let Some(weapon) = arr.get(&gsc_t6::Key::Int(idx - 1)).map(|v| vm.to_text(&v)) else {
        return;
    };
    world
        .resource_mut::<Zm>()
        .wallbuys
        .built
        .insert(base.to_owned(), weapon.clone());
    diag::info!(Sim, "bo2zm wall buy {base}: built as {weapon}");
    if !world.resource::<Zm>().wallbuys.chalk_done {
        return;
    }
    let (Some(fx), Some(origin)) = (
        chalk_fx(vm, "buildable_wallbuy", "", Some(weapon.as_str())),
        field(vm, s, "origin").as_vec3(),
    ) else {
        return;
    };
    let angles = field(vm, s, "angles").as_vec3().unwrap_or([0.0; 3]);
    let old = world.resource_mut::<Zm>().wallbuys.chalk.remove(base);
    if let Some((name, at)) = old {
        super::natives_fx::effect_stop_world(world, &name, at);
    }
    if let Some(name) = super::natives_fx::fx_name(world, fx) {
        let (f, _, u) = gsc_t6::math::angle_vectors(angles);
        super::natives_fx::effect_axis(world, &name, origin, f, u);
        world
            .resource_mut::<Zm>()
            .wallbuys
            .chalk
            .insert(base.to_owned(), (name, origin));
    }
}

/// A world clientfield changed: a wall buy bought shows its gun.
pub(super) fn world_field(vm: &mut Vm<World>, world: &mut World, name: &str, value: i32) {
    let old = world
        .resource_mut::<Zm>()
        .wallbuys
        .world_fields
        .insert(name.to_owned(), value);
    if old == Some(value) {
        return;
    }
    if let Some(base) = name.strip_suffix("_idx") {
        if value > 0 {
            built(vm, world, base, value);
        }
        return;
    }
    if value != 1 || world.resource::<Zm>().wallbuys.shown.contains_key(name) {
        return;
    }
    // The wall buy this field belongs to: `<weapon>_<origin>`.
    let mut found = None;
    for s in spawned(vm) {
        if label(vm, s) == name {
            let tn = field(vm, s, "targetname");
            let target = field(vm, s, "target");
            let weapon = field(vm, s, "zombie_weapon_upgrade");
            found = Some((vm.to_text(&target), vm.to_text(&tn), vm.to_text(&weapon)));
            break;
        }
    }
    let Some((target, tn, mut weapon)) = found else {
        return;
    };
    let claymore = tn == "claymore_purchase";
    // A built wall buy shows its gun's own model (Buried's give it no other).
    let gun = if tn == "buildable_wallbuy" {
        let Some(w) = world.resource::<Zm>().wallbuys.built.get(name).cloned() else {
            return;
        };
        weapon.clone_from(&w);
        let Ok(w) = super::weapon(world, &w) else {
            return;
        };
        let Some(m) = super::frame(world)
            .weapon_world_model(w)
            .map(|(m, _)| m.to_owned())
        else {
            return;
        };
        Some(m)
    } else {
        None
    };
    let mut model_at = None;
    for s in structs(vm) {
        let tn = field(vm, s, "targetname");
        if vm.to_text(&tn) != target {
            continue;
        }
        let m = field(vm, s, "model");
        let (Some(origin), angles) = (
            field(vm, s, "origin").as_vec3(),
            field(vm, s, "angles").as_vec3().unwrap_or([0.0; 3]),
        ) else {
            continue;
        };
        // The map gives the Claymore's on Nuketown, Mob, Buried and Origins;
        // TranZit's and Die Rise's client scripts give theirs the same
        // (`zm_transit.csc`, `zm_highrise.csc`).
        let turn = field(vm, s, "script_vector")
            .as_vec3()
            .or(claymore.then_some([0.0, -90.0, 0.0]))
            .unwrap_or([0.0; 3]);
        let model = gun.clone().unwrap_or_else(|| vm.to_text(&m));
        model_at = Some((model, origin, angles, turn));
        break;
    }
    let Some((model, origin, angles, turn)) = model_at else {
        return;
    };
    // The client spawns it as the weapon's model (`spawn_weapon_model`,
    // `useweaponmodel`): it wears the gun's parts (the MP5's magazine, a
    // sniper's scope) and hides its hide tags. Before, every wall gun was
    // the bare gun.
    let look = super::weapon(world, &weapon)
        .ok()
        .and_then(|w| super::weapon_look(world, w))
        .unwrap_or_default();
    // Out of the wall: from 8 units along its (turned) right to its place.
    let (_, right, _) = gsc_t6::math::angle_vectors(std::array::from_fn(|i| angles[i] + turn[i]));
    let from = std::array::from_fn(|i| origin[i] + right[i] * 8.0);
    let now = world.resource::<Zm>().now_ms;
    let n = world.resource_mut::<Zm>().alloc_entnum();
    let obj = vm.alloc_object(ObjKind::Entity(n));
    let mut zm = world.resource_mut::<Zm>();
    zm.ents.insert(
        n,
        Ent {
            obj: Some(obj),
            classname: "script_model".into(),
            origin: from,
            angles,
            model,
            weapon_parts: look.parts,
            hidden_parts: look.hide,
            ..Default::default()
        },
    );
    zm.movers.move_to(n, from, origin, now, 1.0, 0.0, 0.0);
    zm.wallbuys.shown.insert(name.to_owned(), n);
    diag::info!(
        Sim,
        "bo2zm wall buy {name}: {} slides from ({:.1} {:.1} {:.1}) to ({:.1} {:.1} {:.1})",
        zm.ents[&n].model,
        from[0],
        from[1],
        from[2],
        origin[0],
        origin[1],
        origin[2]
    );
}
