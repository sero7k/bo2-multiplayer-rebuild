//! The Jet Gun as BO2's engine gives it to TranZit's `_zm_weap_jetgun`.
//! Its trigger fires no shots. Held, it spins the engine up (the spin the
//! script reads with `getcurrentweaponspinlerp`: past 0.2 it pulls, grinds
//! and flings zombies) and heats it; at full heat it is overheated, which
//! in TranZit blows it up in his hands. Let go, it spins down and cools.

use bevy_ecs::prelude::World;
use gsc_t6::{ObjRef, Value, Vm};

use super::{Zm, arg, client, frame, text, with_vm};
use crate::world::ClientId;

/// One Jet Gun's numbers, read from the game files (`t6m2 --weapons`,
/// zm_transit: iSpinUpTime, iSpinDownTime, overheatRate, cooldownRate,
/// overheatEndVal, iFireTime). Heat runs 0..100; the rates are per second.
/// It does not cool while firing (coolWhileFiring 0).
#[derive(Clone, Copy)]
pub(crate) struct Gun {
    spin_up_ms: f32,
    spin_down_ms: f32,
    heat_rate: f32,
    cool_rate: f32,
    overheat_end: f32,
    fire_ms: i64,
}

const JETGUN: Gun = Gun {
    spin_up_ms: 3000.0,
    spin_down_ms: 5000.0,
    heat_rate: 17.0,
    cool_rate: 1.0,
    overheat_end: 77.0,
    fire_ms: 50,
};
const JETGUN_UPGRADED: Gun = Gun {
    spin_up_ms: 4000.0,
    ..JETGUN
};
const FULL_HEAT: f32 = 100.0;

pub(crate) fn gun(name: &str) -> Option<Gun> {
    match name.strip_suffix("_mp").unwrap_or(name) {
        "jetgun_zm" => Some(JETGUN),
        "jetgun_upgraded_zm" => Some(JETGUN_UPGRADED),
        _ => None,
    }
}

/// The weapon step leaves a Jet Gun's trigger alone: it fires no shots.
pub(crate) fn owns_trigger(weapon_name: &str) -> bool {
    gun(weapon_name).is_some()
}

/// His Jet Gun's engine: the spin of the gun in hand, and the gun's heat
/// (kept while he holds another weapon; the script puts it back when he
/// drops and picks the gun up again).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct JetState {
    pub spin: f32,
    pub heat: f32,
    pub overheated: bool,
    pub last_fired: i64,
}

fn with_state<T>(world: &mut World, c: u32, f: impl FnOnce(&mut JetState) -> T) -> Option<T> {
    world
        .resource_mut::<Zm>()
        .players
        .get_mut(&c)
        .map(|p| f(&mut p.jet))
}

/// His Jet Gun's heat, 0..100 (the autoplay lets go before it blows up).
pub(crate) fn heat(world: &World, c: u32) -> f32 {
    world.resource::<Zm>().players.get(&c).map_or(0.0, |p| p.jet.heat)
}

/// The Jet Gun a native names (its weapon argument), else the one in hand.
fn named_gun(vm: &Vm<World>, world: &mut World, id: ClientId, a: &[Value], i: usize) -> bool {
    let name = if matches!(arg(a, i), Value::Undefined) {
        let w = frame(world).player(id).map_or(0, |ps| ps.weapon);
        super::weapon_text(world, w)
    } else {
        text(vm, a, i)
    };
    gun(&name).is_some()
}

pub(super) fn bind(vm: &mut Vm<World>) {
    // player getcurrentweaponspinlerp(): the gun in hand's spin, 0..1.
    vm.bind("getcurrentweaponspinlerp", true, |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::Float(with_state(world, id.0, |j| j.spin).unwrap_or(0.0)))
    });
    vm.bind("setcurrentweaponspinlerp", true, |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let v = arg(a, 0).as_float().unwrap_or(0.0).clamp(-1.0, 1.0);
        with_state(world, id.0, |j| j.spin = v);
        Ok(Value::Undefined)
    });
    // player isweaponoverheating(heat?, weapon?): 0 asks whether it is
    // overheated, 1 how hot it is (0..100). Only the Jet Gun heats.
    vm.bind("isweaponoverheating", true, |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let want_heat = arg(a, 0).as_int().unwrap_or(0) != 0;
        if !named_gun(vm, world, id, a, 1) {
            return Ok(if want_heat { Value::Float(0.0) } else { Value::Int(0) });
        }
        let j = with_state(world, id.0, |j| *j).unwrap_or_default();
        Ok(if want_heat {
            Value::Float(j.heat)
        } else {
            Value::Int(i32::from(j.overheated))
        })
    });
    // player setweaponoverheating(overheated, heat, weapon?): puts a
    // dropped Jet Gun's heat back when he picks it up.
    vm.bind("setweaponoverheating", true, |vm, world, s, a| {
        let id = client(vm, world, s)?;
        if !named_gun(vm, world, id, a, 2) {
            return Ok(Value::Undefined);
        }
        let over = arg(a, 0).as_int().is_some_and(|v| v != 0);
        let heat = arg(a, 1).as_float().unwrap_or(0.0).clamp(0.0, FULL_HEAT);
        with_state(world, id.0, |j| {
            j.overheated = over;
            j.heat = heat;
        });
        Ok(Value::Undefined)
    });
    // The jet blowing loose physics objects about, and the shredded-zombie
    // spray out of the exhaust: no loose physics objects yet, and drawing
    // only.
    vm.bind("physicsjetthrust", false, |_, _, _, _| Ok(Value::Undefined));
    vm.bind("weaponplayejectbrass", true, |_, _, _, _| Ok(Value::Undefined));
}

/// Each tick: a held trigger on a ready Jet Gun spins it up and heats it,
/// and fires `weapon_fired` every shot's time (the script's pull loop
/// starts on it); otherwise it spins down and cools.
pub(super) fn tick(world: &mut World, dt_ms: f32) {
    use weapon_iw4::WeaponState as S;
    let now = world.resource::<Zm>().now_ms;
    let clients: Vec<(u32, ObjRef)> = world
        .resource::<Zm>()
        .players
        .iter()
        .map(|(c, p)| (*c, p.obj))
        .collect();
    for (c, obj) in clients {
        let id = ClientId(c);
        let Some(ps) = frame(world).player(id).copied() else {
            continue;
        };
        let name = super::weapon_text(world, ps.weapon);
        let Some(g) = gun(&name) else {
            // Spin belongs to the gun in hand; heat stays with the Jet Gun
            // until it is gone (taken, or blown up).
            let owned = ps
                .weapons
                .iter()
                .any(|&w| w > 0 && gun(&super::weapon_text(world, w as u32)).is_some());
            with_state(world, c, |j| {
                j.spin = 0.0;
                if !owned {
                    *j = JetState::default();
                }
            });
            continue;
        };
        let held = crate::script_player::buttons(&mut frame(world), id)
            & playerstate_iw4::buttons::ATTACK
            != 0;
        let ready = matches!(
            S::from_i32(ps.weaponstate_primary).ok(),
            Some(S::Ready | S::Firing)
        );
        let fired = with_state(world, c, |j| {
            let firing = held && ready && !j.overheated;
            if firing {
                j.spin = (j.spin + dt_ms / g.spin_up_ms).min(1.0);
                j.heat = (j.heat + g.heat_rate * dt_ms / 1000.0).min(FULL_HEAT);
                if j.heat >= FULL_HEAT {
                    j.overheated = true;
                }
            } else {
                j.spin = (j.spin - dt_ms / g.spin_down_ms).max(0.0);
                j.heat = (j.heat - g.cool_rate * dt_ms / 1000.0).max(0.0);
                if j.overheated && j.heat <= g.overheat_end {
                    j.overheated = false;
                }
            }
            let fire = firing && now - j.last_fired >= g.fire_ms;
            if fire {
                j.last_fired = now;
            }
            (fire, *j)
        })
        .unwrap_or_default();
        let (fired, j) = fired;
        if std::env::var_os("IW4L_T6_PLACE").is_some()
            && (held || j.spin > 0.0)
            && now % 1000 < i64::from(crate::MATCH_TICK_MS)
        {
            diag::info!(
                Sim,
                "bo2zm t6 jetgun: {name} held {held} ready {ready} spin {:.2} heat {:.0} overheated {}",
                j.spin,
                j.heat,
                j.overheated
            );
        }
        if fired {
            with_vm(world, |vm, world| {
                let n = vm.string(&name);
                vm.notify_str(world, obj, "weapon_fired", &[n]);
            });
        }
    }
}
