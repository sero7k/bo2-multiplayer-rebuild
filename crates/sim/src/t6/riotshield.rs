//! The Zombie Shield as BO2's engine gives it to the zombies scripts
//! (`_zm_weap_riotshield`, `_zm_riotshield`; TranZit, Mob of the Dead and
//! Origins build one). Fire with the shield in hand: the engine notifies
//! `raise_riotshield`, the script asks where it would stand
//! (`canplaceriotshield`), and when the plant lands the engine notifies
//! `deploy_riotshield` (1 = planted, 0 = called off by a weapon switch). The
//! script then makes the standing shield's model and calls
//! `deployriotshield(player, shield)`, which hands back the trigger he uses
//! to pick it up again.

use bevy_ecs::prelude::World;
use gsc_t6::{Key, ObjKind, ObjRef, Value, Vm};

use super::{Ent, Zm, arg, entnum, frame, origin_of, with_vm};
use crate::world::ClientId;

/// From the press to the shield standing (the plant animation). Not yet
/// measured against the real game.
const DEPLOY_MS: i64 = 800;
/// The pick-up trigger round the planted shield.
const PICKUP_RADIUS: f32 = 64.0;
const PICKUP_HEIGHT: f32 = 72.0;
/// Half a standing body: an actor's or player's middle (`getcentroid`).
const CENTROID_UP: f32 = 36.0;

pub(super) fn bind(vm: &mut Vm<World>) {
    // player canplaceriotshield(event): where the shield would stand, in
    // front of him (`["result"]`, `["origin"]`, `["angles"]`).
    vm.bind("canplaceriotshield", true, |vm, world, s, _| {
        let Some(me) = entnum(vm, s) else {
            return Err("canplaceriotshield needs a player".into());
        };
        let (ok, origin, angles) = super::turrets::place_spot(world, ClientId(me), None);
        let mut arr = gsc_t6::Array::new();
        let k = |vm: &mut Vm<World>, s: &str| Key::Str(vm.intern(s));
        arr.set(k(vm, "result"), Value::bool(ok));
        arr.set(k(vm, "origin"), Value::Vec3(origin));
        arr.set(k(vm, "angles"), Value::Vec3(angles));
        Ok(Value::array(arr))
    });
    // deployriotshield(player, shield): the trigger that picks it up again.
    vm.bind("deployriotshield", false, |vm, world, _, a| {
        let at = origin_of(vm, world, arg(a, 1)).unwrap_or([0.0; 3]);
        let angles = entnum(vm, arg(a, 1))
            .and_then(|n| world.resource::<Zm>().ents.get(&n).map(|e| e.angles))
            .unwrap_or([0.0; 3]);
        let n = world.resource_mut::<Zm>().alloc_entnum();
        let obj = vm.alloc_object(ObjKind::Entity(n));
        world.resource_mut::<Zm>().ents.insert(
            n,
            Ent {
                obj: Some(obj),
                classname: "trigger_radius_use".to_owned(),
                origin: at,
                angles,
                radius: PICKUP_RADIUS,
                height: PICKUP_HEIGHT,
                ..Default::default()
            },
        );
        if std::env::var_os("IW4L_T6_PLACE").is_some() {
            diag::info!(
                Sim,
                "bo2zm t6 riotshield planted at ({:.0} {:.0} {:.0}), pick-up trigger ent{n}",
                at[0],
                at[1],
                at[2]
            );
        }
        Ok(Value::Object(obj))
    });
    // entity getcentroid(): the middle of a body, else its origin.
    vm.bind("getcentroid", true, |vm, world, s, _| {
        let n = entnum(vm, s);
        let o = origin_of(vm, world, s).unwrap_or([0.0; 3]);
        let zm = world.resource::<Zm>();
        let body = n.is_some_and(|n| zm.players.contains_key(&n) || zm.actors.contains_key(&n));
        Ok(Value::Vec3([o[0], o[1], o[2] + if body { CENTROID_UP } else { 0.0 }]))
    });
    // The held / standing shield's look and the "can't place" hint: drawing
    // only, nothing the game keeps.
    for name in [
        "setheldweaponmodel",
        "setplacementhint",
        "setriotshieldfailhint",
        "setscriptmoverflag",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
}

/// Each tick: fire pressed with the shield in hand raises it; DEPLOY_MS
/// later it is planted, or called off if he switched away.
pub(super) fn deploy_events(world: &mut World) {
    let now = world.resource::<Zm>().now_ms;
    let clients: Vec<(u32, ObjRef, bool, Option<i64>)> = world
        .resource::<Zm>()
        .players
        .iter()
        .map(|(c, p)| (*c, p.obj, p.last_attack, p.shield_raised))
        .collect();
    for (c, obj, was_down, raised) in clients {
        let id = ClientId(c);
        let Some(w) = frame(world).player(id).map(|ps| ps.weapon as u32) else {
            continue;
        };
        let held = crate::script_player::buttons(&mut frame(world), id);
        let down = held & playerstate_iw4::buttons::ATTACK != 0;
        let shield = super::weapon_text(world, w).starts_with("riotshield");
        let mut event: Option<(&str, Option<bool>)> = None;
        let mut next = raised;
        match raised {
            None if shield && down && !was_down => {
                event = Some(("raise_riotshield", None));
                next = Some(now);
            }
            Some(_) if !shield => {
                event = Some(("deploy_riotshield", Some(false)));
                next = None;
            }
            Some(t) if now - t >= DEPLOY_MS => {
                event = Some(("deploy_riotshield", Some(true)));
                next = None;
            }
            _ => {}
        }
        if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&c) {
            p.last_attack = down;
            p.shield_raised = next;
        }
        if let Some((name, attempt)) = event {
            if std::env::var_os("IW4L_T6_PLACE").is_some() {
                diag::info!(Sim, "bo2zm t6 riotshield: {name} {attempt:?}");
            }
            with_vm(world, |vm, world| {
                let args: Vec<Value> = attempt.map(Value::bool).into_iter().collect();
                vm.notify_str(world, obj, name, &args);
            });
        }
    }
}
