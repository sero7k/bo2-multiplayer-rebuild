//! A player's grenade hand as BO2's engine shows it to the scripts:
//! `isthrowinggrenade` (a grenade or other offhand is out), and
//! `isswitchingweapons`. TranZit's Denizen grab calls `forcegrenadethrow`
//! on a player holding a cooked grenade: the grenade leaves his hand as if
//! he let go of the button, and his hands go to the Denizen.

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};
use weapon_iw4::WeaponState as S;

use super::{Zm, client, frame};
use crate::world::ClientId;

const OFFHAND: u32 = playerstate_iw4::buttons::FRAG | playerstate_iw4::buttons::SMOKE;

fn state(world: &mut World, id: ClientId) -> i32 {
    frame(world).player(id).map_or(0, |ps| ps.weaponstate_primary)
}

/// Taking the grenade out, cooking or holding it: the throw waits on him.
fn held_back(state: i32) -> bool {
    matches!(
        S::from_i32(state).ok(),
        Some(S::OffhandInit | S::OffhandPrepare | S::OffhandHold)
    )
}

pub(super) fn bind(vm: &mut Vm<World>) {
    vm.bind("isthrowinggrenade", true, |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::bool(matches!(
            S::from_i32(state(world, id)).ok(),
            Some(
                S::OffhandInit
                    | S::OffhandPrepare
                    | S::OffhandHold
                    | S::OffhandStart
                    | S::Offhand
                    | S::OffhandEnd
            )
        )))
    });
    vm.bind("isswitchingweapons", true, |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::bool(crate::script::host::weapons::is_changing_weapon(
            state(world, id),
        )))
    });
    // player forcegrenadethrow(): a grenade he is holding goes now.
    vm.bind("forcegrenadethrow", true, |vm, world, s, _| {
        let id = client(vm, world, s)?;
        if held_back(state(world, id)) {
            if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
                p.force_throw = true;
            }
        }
        Ok(Value::Undefined)
    });
}

/// Each tick, before the weapon step reads his buttons: a forced throw
/// lets go of the grenade button for him, until the grenade is gone and he
/// has let go himself (so the button still down is not a fresh throw).
pub(super) fn tick(world: &mut World) {
    let forced: Vec<u32> = world
        .resource::<Zm>()
        .players
        .iter()
        .filter(|(_, p)| p.force_throw)
        .map(|(c, _)| *c)
        .collect();
    for c in forced {
        let id = ClientId(c);
        let mut down = false;
        {
            let mut req = world.resource_mut::<crate::step::StepRequest>();
            for (who, cmd) in &mut req.input.cmds {
                if *who == id {
                    down |= cmd.buttons & OFFHAND != 0;
                    cmd.buttons &= !OFFHAND;
                }
            }
        }
        if !down && !held_back(state(world, id)) {
            if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&c) {
                p.force_throw = false;
            }
        }
    }
}

/// bo2mp: a flashbang caught him: Black Ops II's `_flashgrenades.gsc`
/// (`monitorflash`) waits on his player for `"flashbang", amount_distance,
/// amount_angle, attacker` and blinds and deafens him by them.
pub(crate) fn flashbang(
    world: &mut World,
    target: ClientId,
    amount_distance: f32,
    amount_angle: f32,
    attacker: ClientId,
) {
    let (obj, by) = {
        let Some(zm) = world.get_resource::<super::Zm>() else {
            return;
        };
        let Some(obj) = zm.players.get(&target.0).map(|p| p.obj) else {
            return;
        };
        let by = zm.players.get(&attacker.0).map_or(gsc_t6::Value::Undefined, |p| gsc_t6::Value::Object(p.obj));
        (obj, by)
    };
    super::with_vm(world, |vm, world| {
        if vm.alive(obj) {
            vm.notify_str(world, obj, "flashbang", &[gsc_t6::Value::Float(amount_distance), gsc_t6::Value::Float(amount_angle), by]);
        }
    });
}
