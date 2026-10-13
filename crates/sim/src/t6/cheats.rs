//! bo2zm: the solo cheat menu's side in the game (Num 1, for recording a
//! map's features): the lines he picks are queued by the menu and land at
//! the next tick, each through the scripts' own functions; the toggles are
//! flags the game reads while they are on. Only a local game of one player
//! (no Public Match lobby, one player expected) takes them.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use bevy_ecs::prelude::World;
use gsc_t6::Value;

use super::{Zm, frame, with_vm};

/// One line of the menu, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cheat {
    Points,
    GodMode,
    NoClip,
    IgnoreMe,
    InfiniteAmmo,
    EndRound,
    DropPowerUp,
    PowerOn,
    OpenDoors,
    PackAPunch,
    AllPerks,
}

impl Cheat {
    pub const ALL: [Cheat; 11] = [
        Cheat::Points,
        Cheat::GodMode,
        Cheat::NoClip,
        Cheat::IgnoreMe,
        Cheat::InfiniteAmmo,
        Cheat::EndRound,
        Cheat::DropPowerUp,
        Cheat::PowerOn,
        Cheat::OpenDoors,
        Cheat::PackAPunch,
        Cheat::AllPerks,
    ];

    /// The line as the menu shows it.
    pub fn label(self) -> &'static str {
        match self {
            Cheat::Points => "+1000 Points",
            Cheat::GodMode => "God Mode",
            Cheat::NoClip => "No Clip",
            Cheat::IgnoreMe => "Zombies Ignore You",
            Cheat::InfiniteAmmo => "Infinite Ammo",
            Cheat::EndRound => "End Round",
            Cheat::DropPowerUp => "Drop Power-Up",
            Cheat::PowerOn => "Power On",
            Cheat::OpenDoors => "Open All Doors",
            Cheat::PackAPunch => "Pack-a-Punch Gun",
            Cheat::AllPerks => "All Perks",
        }
    }

    /// ON / OFF at the end of its line (else an action, picked again and
    /// again).
    pub fn is_toggle(self) -> bool {
        matches!(
            self,
            Cheat::GodMode | Cheat::NoClip | Cheat::IgnoreMe | Cheat::InfiniteAmmo
        )
    }

    fn bit(self) -> u32 {
        1 << self as u32
    }
}

static PICKS: Mutex<Vec<Cheat>> = Mutex::new(Vec::new());
static ON: AtomicU32 = AtomicU32::new(0);
/// Zombies Ignore You set the player's `ignoreme` (cleared once when it
/// goes off, so the scripts' own use of it is left alone).
static IGNORING: AtomicBool = AtomicBool::new(false);

/// A local game of one player: no Public Match lobby and one player.
pub fn cheats_allowed() -> bool {
    super::PARTY.load(Ordering::Relaxed) == 0 && super::expected_players() == 1
}

/// He picked `cheat`: a toggle flips (its new state is returned), an action
/// is queued for the next tick (true).
pub fn cheat_pick(cheat: Cheat) -> bool {
    if cheat.is_toggle() {
        return ON.fetch_xor(cheat.bit(), Ordering::Relaxed) & cheat.bit() == 0;
    }
    if let Ok(mut picks) = PICKS.lock() {
        picks.push(cheat);
    }
    true
}

/// Whether a toggle is on.
pub fn cheat_on(cheat: Cheat) -> bool {
    ON.load(Ordering::Relaxed) & cheat.bit() != 0
}

/// A new game starts with every cheat off and nothing queued.
pub(crate) fn reset() {
    ON.store(0, Ordering::Relaxed);
    OPENING.store(false, Ordering::Relaxed);
    IGNORING.store(false, Ordering::Relaxed);
    if let Ok(mut picks) = PICKS.lock() {
        picks.clear();
    }
}

/// The queued picks, applied (each tick, before the scripts run).
pub(crate) fn apply(world: &mut World) {
    let picks = PICKS
        .lock()
        .map(|mut p| std::mem::take(&mut *p))
        .unwrap_or_default();
    if picks.is_empty() {
        return;
    }
    if !cheats_allowed() || world.resource::<Zm>().players.len() != 1 {
        diag::info!(
            Sim,
            "bo2zm cheats: {} picks dropped (not a solo game)",
            picks.len()
        );
        return;
    }
    for cheat in picks {
        let done = match cheat {
            Cheat::Points => player_call(
                world,
                "maps/mp/zombies/_zm_score",
                "add_to_player_score",
                vec![Value::Int(1000)],
            ),
            Cheat::EndRound => end_round(world),
            Cheat::DropPowerUp => drop_powerup(world),
            Cheat::PowerOn => power_on(world),
            Cheat::OpenDoors => {
                NEXT_DOOR.store(0, Ordering::Relaxed);
                OPENING.store(true, Ordering::Relaxed);
                true
            }
            Cheat::PackAPunch => pack_a_punch(world),
            Cheat::AllPerks => all_perks(world),
            // The toggles are flags, never queued.
            Cheat::GodMode | Cheat::NoClip | Cheat::IgnoreMe | Cheat::InfiniteAmmo => true,
        };
        diag::info!(
            Sim,
            "bo2zm cheats: {} {}",
            cheat.label(),
            if done { "applied" } else { "not found" }
        );
    }
}

/// No Clip held each tick: a walking player flies (the movement code's
/// pm_type 2, which his client predicts too); off, he walks and falls
/// from where he is. Down, linked or dead he is left as he is.
pub(crate) fn hold(world: &mut World) {
    use movement_iw4::noclip::PM_TYPE_NOCLIP;
    let fly = cheat_on(Cheat::NoClip);
    let clients: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    for c in clients {
        if let Some(ps) = frame(world).player_mut(crate::world::ClientId(c)) {
            if fly && ps.pm_type == 0 {
                ps.pm_type = PM_TYPE_NOCLIP;
                diag::info!(Sim, "bo2zm cheats: no clip on (client {c})");
            } else if !fly && ps.pm_type == PM_TYPE_NOCLIP {
                ps.pm_type = 0;
                diag::info!(Sim, "bo2zm cheats: no clip off (client {c})");
            }
        }
        // Infinite Ammo: the gun in his hands, clip and stock full (each
        // clamps to the weapon's own size).
        if cheat_on(Cheat::InfiniteAmmo) {
            let id = crate::world::ClientId(c);
            let mut f = frame(world);
            if let Some(w) = f.player(id).map(|p| p.weapon as u32).filter(|w| *w != 0) {
                crate::script_player::set_ammo_clip(&mut f, id, w, i32::MAX);
                crate::script_player::set_ammo_stock(&mut f, id, w, i32::MAX);
            }
        }
    }
    open_next_door(world);
    // Zombies Ignore You: the scripts' own `ignoreme`, which their target
    // pick (is_player_valid) skips.
    let ignore = cheat_on(Cheat::IgnoreMe);
    if ignore || IGNORING.load(Ordering::Relaxed) {
        if let Some(p) = player(world) {
            with_vm(world, |vm, world| {
                let k = vm.intern("ignoreme");
                vm.set_field(world, p, k, Value::Int(i32::from(ignore)));
            });
        }
        if IGNORING.swap(ignore, Ordering::Relaxed) != ignore {
            let state = if ignore { "on" } else { "off" };
            diag::info!(Sim, "bo2zm cheats: zombies ignore you {state}");
        }
    }
}

/// End Round: no more zombies to spawn this round and every one alive
/// killed (as BO2's kill_all_zombies does: its own damage, MOD_SUICIDE), so
/// the scripts end the round and start the next as normal.
fn end_round(world: &mut World) -> bool {
    let alive: Vec<(u32, gsc_t6::ObjRef)> = world
        .resource::<Zm>()
        .actors
        .iter()
        .filter(|(_, a)| a.alive)
        .filter_map(|(n, a)| a.obj.map(|o| (*n, o)))
        .collect();
    with_vm(world, |vm, world| {
        let total = vm.intern("zombie_total");
        vm.set_field(world, vm.level, total, Value::Int(0));
        let origin = vm.intern("origin");
        for (n, obj) in &alive {
            let at = vm
                .get_field(world, *obj, origin)
                .as_vec3()
                .unwrap_or([0.0; 3]);
            let me = Value::Object(*obj);
            super::actors::damage(
                vm,
                world,
                *n,
                me.clone(),
                me,
                100_000_000,
                0,
                "MOD_SUICIDE",
                "none",
                at,
                [0.0; 3],
                "none",
            );
        }
        diag::info!(
            Sim,
            "bo2zm cheats: end round: {} zombies killed",
            alive.len()
        );
    })
    .is_some()
}

/// Which of the map's power-ups the next Drop Power-Up drops.
static NEXT_POWERUP: AtomicU32 = AtomicU32::new(0);

/// Drop Power-Up: the next of the map's own power-ups (its
/// `level.zombie_powerup_array`, by name) at his feet, by the scripts'
/// specific_powerup_drop.
fn drop_powerup(world: &mut World) -> bool {
    let Some(at) = player_origin(world) else {
        return false;
    };
    with_vm(world, |vm, world| {
        let k = vm.intern("zombie_powerup_array");
        let mut names: Vec<String> = match vm.raw_field(vm.level, k) {
            Value::Array(a) => a.read().values_in_order().map(|v| vm.to_text(v)).collect(),
            _ => Vec::new(),
        };
        names.sort();
        names.dedup();
        if names.is_empty() {
            return false;
        }
        let i = NEXT_POWERUP.fetch_add(1, Ordering::Relaxed) as usize % names.len();
        diag::info!(
            Sim,
            "bo2zm cheats: drop power-up {} ({} of {})",
            names[i],
            i + 1,
            names.len()
        );
        let name = vm.string(&names[i]);
        vm.spawn_named(
            world,
            "maps/mp/zombies/_zm_powerups",
            "specific_powerup_drop",
            Value::Object(vm.level),
            vec![name, Value::Vec3(at)],
        )
        .is_some()
    })
    .unwrap_or(false)
}

/// The map's script triggers named `targetname` (entity number, object,
/// its `script_noteworthy`).
fn triggers_named(
    vm: &mut gsc_t6::Vm<World>,
    world: &World,
    targetname: &[&str],
) -> Vec<(u32, gsc_t6::ObjRef, String)> {
    let (tn, sn) = (vm.intern("targetname"), vm.intern("script_noteworthy"));
    let mut out: Vec<_> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter(|(_, e)| e.classname.starts_with("trigger"))
        .filter_map(|(n, e)| {
            let o = e.obj?;
            let name = vm.to_text(&vm.raw_field(o, tn));
            targetname
                .contains(&name.as_str())
                .then(|| (*n, o, vm.to_text(&vm.raw_field(o, sn))))
        })
        .collect();
    out.sort_by_key(|t| t.0);
    out
}

/// Power On: the power switch's own trigger fired by him (its lever and
/// lights) and the scripts' `power_on` flag set, as the switch sets it.
fn power_on(world: &mut World) -> bool {
    let Some(p) = player(world) else {
        return false;
    };
    with_vm(world, |vm, world| {
        let switches = triggers_named(vm, world, &["use_elec_switch"]);
        for (_, t, _) in &switches {
            vm.notify_str(world, *t, "trigger", &[Value::Object(p)]);
        }
        let f = vm.string("power_on");
        diag::info!(
            Sim,
            "bo2zm cheats: power on ({} switches thrown)",
            switches.len()
        );
        vm.spawn_named(
            world,
            "common_scripts/utility",
            "flag_set",
            Value::Object(vm.level),
            vec![f],
        )
        .is_some()
    })
    .unwrap_or(false)
}

/// Open All Doors runs one door or debris pile a tick (a bought debris
/// pile deletes its sibling triggers): the next one by entity number.
static OPENING: AtomicBool = AtomicBool::new(false);
static NEXT_DOOR: AtomicU32 = AtomicU32::new(0);

/// The next door or debris pile not yet opened, bought by him for free
/// (its trigger with `force`, as the scripts' own open-all does).
fn open_next_door(world: &mut World) {
    if !OPENING.load(Ordering::Relaxed) {
        return;
    }
    let Some(p) = player(world) else {
        return;
    };
    let next = NEXT_DOOR.load(Ordering::Relaxed);
    with_vm(world, |vm, world| {
        let opened = vm.intern("has_been_opened");
        let door = triggers_named(vm, world, &["zombie_door", "zombie_debris"])
            .into_iter()
            .find(|(n, o, _)| *n >= next && !gsc_t6::truthy(&vm.raw_field(*o, opened)));
        match door {
            Some((n, t, _)) => {
                NEXT_DOOR.store(n + 1, Ordering::Relaxed);
                diag::info!(Sim, "bo2zm cheats: open door ent{n}");
                vm.notify_str(world, t, "trigger", &[Value::Object(p), Value::Int(1)]);
            }
            None => {
                OPENING.store(false, Ordering::Relaxed);
                diag::info!(Sim, "bo2zm cheats: open all doors: done");
            }
        }
    });
}

/// Pack-a-Punch Gun: the gun in his hands swapped for its upgrade (the
/// map's `level.zombie_weapons[gun].upgrade_name`), given by the scripts'
/// weapon_give as an upgrade.
fn pack_a_punch(world: &mut World) -> bool {
    let (Some(p), Some(c)) = (
        player(world),
        world.resource::<Zm>().players.keys().next().copied(),
    ) else {
        return false;
    };
    let id = crate::world::ClientId(c);
    let Some(w) = frame(world)
        .player(id)
        .map(|ps| ps.weapon as u32)
        .filter(|w| *w != 0)
    else {
        return false;
    };
    let gun = super::weapon_text(world, w);
    with_vm(world, |vm, world| {
        let (zw, up) = (vm.intern("zombie_weapons"), vm.intern("upgrade_name"));
        let upgrade = match vm.raw_field(vm.level, zw) {
            Value::Array(a) => a.read().get(&gsc_t6::Key::Str(vm.intern(&gun))).cloned(),
            _ => None,
        }
        .and_then(|v| v.as_obj())
        .map(|o| vm.to_text(&vm.raw_field(o, up)))
        .filter(|u| !u.is_empty() && u != "undefined" && *u != gun);
        let Some(upgrade) = upgrade else {
            diag::info!(Sim, "bo2zm cheats: pack-a-punch: {gun} has no upgrade");
            return false;
        };
        diag::info!(Sim, "bo2zm cheats: pack-a-punch: {gun} -> {upgrade}");
        crate::script_player::take_weapon(&mut frame(world), id, w);
        let u = vm.string(&upgrade);
        vm.spawn_named(
            world,
            "maps/mp/zombies/_zm_weapons",
            "weapon_give",
            Value::Object(p),
            vec![u, Value::Int(1)],
        )
        .is_some()
    })
    .unwrap_or(false)
}

/// All Perks: every perk machine's perk the map has (its vending triggers'
/// `script_noteworthy`) that he lacks, by the scripts' give_perk.
fn all_perks(world: &mut World) -> bool {
    let Some(p) = player(world) else {
        return false;
    };
    let has: Vec<String> = world
        .resource::<Zm>()
        .players
        .values()
        .next()
        .map(|pl| pl.perks.iter().cloned().collect())
        .unwrap_or_default();
    with_vm(world, |vm, world| {
        let mut perks: Vec<String> = triggers_named(vm, world, &["zombie_vending"])
            .into_iter()
            .map(|(_, _, perk)| perk)
            .filter(|perk| {
                perk.starts_with("specialty_")
                    && perk != "specialty_weapupgrade"
                    && !has.contains(perk)
            })
            .collect();
        perks.sort();
        perks.dedup();
        diag::info!(Sim, "bo2zm cheats: all perks: {}", perks.join(" "));
        for perk in &perks {
            let s = vm.string(perk);
            vm.spawn_named(
                world,
                "maps/mp/zombies/_zm_perks",
                "give_perk",
                Value::Object(p),
                vec![s, Value::Int(0)],
            );
        }
        true
    })
    .unwrap_or(false)
}

/// Where the one player stands.
fn player_origin(world: &mut World) -> Option<[f32; 3]> {
    let c = *world.resource::<Zm>().players.keys().next()?;
    frame(world)
        .player(crate::world::ClientId(c))
        .map(|p| p.origin)
}

/// The one player's script object.
fn player(world: &World) -> Option<gsc_t6::ObjRef> {
    world
        .resource::<Zm>()
        .players
        .values()
        .next()
        .map(|p| p.obj)
}

/// A script function run on the player (false when it or he is missing).
fn player_call(world: &mut World, file: &str, func: &str, args: Vec<Value>) -> bool {
    let Some(p) = player(world) else {
        return false;
    };
    with_vm(world, |vm, world| {
        vm.spawn_named(world, file, func, Value::Object(p), args)
            .is_some()
    })
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_action_is_queued_each_pick_a_toggle_flips() {
        reset();
        assert!(cheat_pick(Cheat::Points));
        assert!(cheat_pick(Cheat::Points));
        assert_eq!(PICKS.lock().unwrap().len(), 2);
        assert!(!cheat_on(Cheat::Points));
        reset();
        assert!(PICKS.lock().unwrap().is_empty());
        // A toggle flips and is not queued (one test: the state is global).
        assert!(cheat_pick(Cheat::GodMode));
        assert!(cheat_on(Cheat::GodMode));
        assert!(!cheat_pick(Cheat::GodMode));
        assert!(!cheat_on(Cheat::GodMode));
        assert!(PICKS.lock().unwrap().is_empty());
        cheat_pick(Cheat::GodMode);
        reset();
        assert!(!cheat_on(Cheat::GodMode));
        assert!(PICKS.lock().unwrap().is_empty());
    }
}
