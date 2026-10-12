//! bo2mp: a scorestreak weapon's switch BO2 refuses before it starts.
//!
//! BO2's `_killstreaks::killstreakwaiter` only hears of a streak once its
//! weapon is raised (`weapon_change`); when the streak may not be used it
//! switches straight back (`switchtolastnonkillstreakweapon`). Its exe never
//! lets that happen for a streak that is not there: a d-pad key whose
//! streak weapon is empty (not earned yet, already used) raises nothing.
//! Ours refuses the switch up front, silently and without an animation, for
//! everything the scripts would refuse it for that is known before the
//! raise: not earned (scorestreaks: no ammo, as the waiter checks;
//! momentum: below the cost), a location being picked, a remote in use, the
//! final killcam, the game over, last stand, an EMP, the airspace or team
//! limits (`_killstreakrules::iskillstreakallowed`, read from the same
//! `level.killstreakrules` / `level.killstreaktype` tables), and the moves
//! that hold any weapon switch (mantle, ladder) or a ride.

use bevy_ecs::prelude::World;
use gsc_t6::{Key, ObjRef, Value, Vm, truthy};

use super::super::{Zm, frame, with_vm};
use super::streaks;
use crate::world::ClientId;

fn field(vm: &mut Vm<World>, o: ObjRef, name: &str) -> Value {
    let f = vm.intern(name);
    vm.raw_field(o, f)
}

fn obj_field(vm: &mut Vm<World>, v: &Value, name: &str) -> Value {
    match v {
        Value::Object(o) => field(vm, *o, name),
        _ => Value::Undefined,
    }
}

fn at(vm: &mut Vm<World>, v: &Value, key: &str) -> Value {
    match v {
        Value::Array(a) => {
            let k = Key::Str(vm.intern(key));
            a.get(&k).unwrap_or_default()
        }
        _ => Value::Undefined,
    }
}

fn num(v: &Value) -> f32 {
    match v {
        Value::Int(i) => *i as f32,
        Value::Float(f) => *f,
        _ => 0.0,
    }
}

/// BO2's held scorestreak weapons (the Death Machine, the War Machine):
/// raised again while in use, they are just guns.
fn held_streak_weapon(name: &str) -> bool {
    name.contains("minigun") || name.contains("m32")
}

/// What the scripts' checks say, from their own state.
enum Scripted {
    /// Not one of `level.killstreakweapons` (or no scripts yet).
    NotStreak,
    Allowed,
    Refused(&'static str),
}

fn script_refusal(
    vm: &mut Vm<World>,
    world: &mut World,
    client: u32,
    name: &str,
    ammo: i32,
    from_inventory: bool,
) -> Scripted {
    let level = vm.level;
    let weapons = field(vm, level, "killstreakweapons");
    let ks = at(vm, &weapons, name);
    if ks.is_undefined() {
        return Scripted::NotStreak;
    }
    let ks = vm.to_text(&ks);
    let Some(obj) = world.resource::<Zm>().players.get(&client).map(|p| p.obj) else {
        return Scripted::Allowed;
    };
    if truthy(&field(vm, obj, "usingkillstreakheldweapon")) && held_streak_weapon(name) {
        return Scripted::Allowed;
    }
    if truthy(&field(vm, level, "infinalkillcam")) {
        return Scripted::Refused("final killcam");
    }
    if truthy(&field(vm, level, "gameended")) {
        return Scripted::Refused("game ended");
    }
    if !field(vm, obj, "selectinglocation").is_undefined() {
        return Scripted::Refused("selecting a location");
    }
    if !field(vm, obj, "usingremote").is_undefined() {
        return Scripted::Refused("using a remote");
    }
    if truthy(&field(vm, obj, "laststand")) {
        return Scripted::Refused("last stand");
    }
    // Earned? (killstreakwaiter's own test.)
    if truthy(&field(vm, level, "usingscorestreaks")) {
        if !from_inventory && ammo <= 0 && name != "killstreak_ai_tank_mp" {
            return Scripted::Refused("not earned");
        }
    } else if truthy(&field(vm, level, "usingmomentum")) && !from_inventory {
        let killstreaks = field(vm, level, "killstreaks");
        let entry = at(vm, &killstreaks, &ks);
        let cost = num(&obj_field(vm, &entry, "momentumcost"));
        if num(&field(vm, obj, "momentum")) < cost {
            return Scripted::Refused("not earned");
        }
    }
    // iskillstreakallowed: each rule the streak checks.
    let types = field(vm, level, "killstreaktype");
    let rules_of = at(vm, &types, &ks);
    let Value::Array(rules_of) = rules_of else {
        return Scripted::Allowed;
    };
    let keys: Vec<Key> = rules_of.read().keys().collect();
    let teambased = truthy(&field(vm, level, "teambased"));
    let team = {
        let t = field(vm, obj, "team");
        vm.to_text(&t)
    };
    let rules = field(vm, level, "killstreakrules");
    for key in keys {
        let entry = rules_of.get(&key).unwrap_or_default();
        if !truthy(&obj_field(vm, &entry, "checks")) {
            continue;
        }
        let rule = match &rules {
            Value::Array(r) => r.get(&key).unwrap_or_default(),
            _ => Value::Undefined,
        };
        let max = num(&obj_field(vm, &rule, "max"));
        if max != 0.0 && num(&obj_field(vm, &rule, "cur")) >= max {
            return Scripted::Refused("airspace full");
        }
        let per_team = num(&obj_field(vm, &rule, "maxperteam"));
        if teambased && per_team != 0.0 {
            let cur_team = obj_field(vm, &rule, "curteam");
            if num(&at(vm, &cur_team, &team)) >= per_team {
                return Scripted::Refused("team limit reached");
            }
        }
    }
    Scripted::Allowed
}

/// Why his switch to `weapon` must not start (None: let it). Only a BO2
/// multiplayer game's scorestreak weapons are ever refused.
pub(crate) fn refusal(
    world: &mut World,
    client: u32,
    weapon: u32,
    inventory_pressed: bool,
) -> Option<&'static str> {
    if weapon == 0 || !world.get_resource::<Zm>().is_some_and(|z| z.mp) {
        return None;
    }
    let name = super::super::weapon_text(world, weapon);
    let (pm_type, pm_flags, ammo) = {
        let f = frame(world);
        let ps = f.player(ClientId(client))?;
        let (t, fl) = (ps.pm_type, ps.pm_flags);
        let ammo = crate::script_player::ammo_clip(&f, ClientId(client), weapon)
            + crate::script_player::ammo_stock(&f, ClientId(client), weapon);
        (t, fl, ammo)
    };
    let from_inventory = inventory_pressed
        && streaks(world).inventory.get(&client).is_some_and(|w| *w == name);
    let scripted = with_vm(world, |vm, world| {
        script_refusal(vm, world, client, &name, ammo, from_inventory)
    })
    .unwrap_or(Scripted::NotStreak);
    match scripted {
        Scripted::NotStreak => return None,
        Scripted::Refused(why) => return Some(why),
        Scripted::Allowed => {}
    }
    use playerstate_iw4::pm_flags;
    if pm_type == playerstate_iw4::PM_TYPE_LAST_STAND || pm_flags & pm_flags::LAST_STAND != 0 {
        return Some("last stand");
    }
    if pm_flags & pm_flags::MANTLE != 0 {
        return Some("mantling");
    }
    if pm_flags & (pm_flags::LADDER | pm_flags::LADDER_FALL) != 0 {
        return Some("on a ladder");
    }
    if super::riding(world, client) || super::remote_control(world, client) {
        return Some("steering a scorestreak");
    }
    if streaks(world).emp_jammed.contains(&client) {
        return Some("EMP jammed");
    }
    None
}

/// The server's half: his command's weapon switch to a scorestreak weapon
/// the scripts would refuse goes nowhere (he keeps the weapon in hand, no
/// drop, no raise). A switch the scripts ordered (`switchtoweapon`) and one
/// already approved and under way pass.
pub(crate) fn gate_cmd(world: &mut World, client: u32, cmd: &mut playerstate_iw4::UserCmd) {
    if !world.get_resource::<Zm>().is_some_and(|z| z.mp) {
        return;
    }
    let (held, switch_to) = {
        let f = frame(world);
        let held = f.player(ClientId(client)).map_or(0, |ps| ps.weapon);
        let switch_to = f
            .client_meta(ClientId(client))
            .map_or(0, |m| m.controls.switch_to);
        (held, switch_to)
    };
    let target = u32::from(cmd.weapon);
    if target == 0 || target == held || target == switch_to {
        streaks(world).switch_ok.remove(&client);
        return;
    }
    if streaks(world).switch_ok.get(&client) == Some(&target) {
        return;
    }
    let inventory = cmd.buttons & playerstate_iw4::buttons::INVENTORY != 0;
    match refusal(world, client, target, inventory) {
        Some(why) => {
            cmd.weapon = held as u16;
            if inventory {
                cmd.buttons &= !playerstate_iw4::buttons::INVENTORY;
            }
            let now = world.resource::<Zm>().now_ms;
            let log = {
                let mut st = streaks(world);
                let due = st
                    .refused_log
                    .get(&client)
                    .is_none_or(|(w, r, t)| *w != target || *r != why || now - *t >= 3000);
                if due {
                    st.refused_log.insert(client, (target, why, now));
                }
                due
            };
            if log {
                let name = super::super::weapon_text(world, target);
                diag::info!(
                    Sim,
                    "bo2mp streaks: player {client} switch to {name} refused ({why}): no raise"
                );
            }
        }
        None => {
            streaks(world).switch_ok.insert(client, target);
        }
    }
}

/// His client's half: which of his d-pad slots hold a scorestreak he may
/// not raise now (`bo2mp_streak_block`: four characters, slot 1 first, '1'
/// refused), so his own key never starts the raise the server would take
/// back.
pub(crate) fn publish_slots(world: &mut World) {
    let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    for client in ids {
        let slots = frame(world)
            .player(ClientId(client))
            .map(|ps| (ps.action_slot_type, ps.action_slot_param));
        let Some((types, params)) = slots else {
            continue;
        };
        let mut mask = String::with_capacity(4);
        for i in 0..4 {
            let refused = types.get(i).copied() == Some(1)
                && params.get(i).is_some_and(|p| *p > 0)
                && refusal(world, client, params[i] as u32, false).is_some();
            mask.push(if refused { '1' } else { '0' });
        }
        let changed = {
            let mut st = streaks(world);
            let prev = st.slot_mask.insert(client, mask.clone());
            prev.as_deref() != Some(mask.as_str())
        };
        if changed {
            super::super::set_client_dvar(world, client, "bo2mp_streak_block", &mask);
            if !world.resource::<Zm>().bots.contains_key(&client) {
                diag::info!(Sim, "bo2mp streaks: player {client} d-pad blocked {mask}");
            }
        }
    }
}
