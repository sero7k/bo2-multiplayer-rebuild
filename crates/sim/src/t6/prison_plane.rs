//! Mob of the Dead (zm_prison) plane test aid.
//!
//! IW4L_T6_PLANE=<fly seconds>: the plane's first flight, the way a player
//! gets there (zm_alcatraz_sq, zm_alcatraz_sq_nixie, zm_alcatraz_craftables).
//! Once he holds the lightning hands (a ghost) and the puzzles are set up
//! (`initial_blackscreen_passed`), every 3.5 s he is put by the next panel
//! and taps fire at it until it reacts: the three generator panels, the
//! laundry switch and the warden's key pulley (`master_key_pulley_<west|
//! east>`, `level.is_master_key_west`); he stands within 240 of each (BO2
//! counts a zap from within 256). From 17.5 s, 5.5 s a tube, he shocks
//! each nixie tube until it shows its digit of the code (`a_nixie_tube_
//! solution`), which drops the Citadel elevator with the rigging. 34-44 s he
//! is put by his body and revived, and at 44 s every zone is opened (the
//! steps after put him behind doors no one bought, where the out-of-playable
//! check would take his lives); at 45 s he opens the elevator's gate (Use,
//! 60 s before it rises); at 50 s he takes the key (Use for 3 s); from 54 s,
//! every 4 s, the next key door (put in its use box looking at its lock, Use
//! pressed 3 s), then a second pass for any door still shut, from another
//! side; at 95 s the dryer (its 30 s cycle leaves the sheets). From 135 s he
//! picks the parts up, 3 s a try, standing a step from each and looking at
//! its pick-up trigger, a new spot each try (6 tries a part).
//! IW4L_T6_CRAFT=plane@<s> then builds the plane on the roof (from 215 s:
//! the parts are carried by then). At the fly time he is put by the takeoff trigger
//! (`plane_fly_trigger`) and presses Use for 2 s; every 5 s for 100 s the
//! flight's flags and where he is are logged. Every step is logged, with his
//! afterlife meter (`manacur`, 200 full) while he is a ghost.

use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicI64, Ordering};

use bevy_ecs::prelude::World;
use gsc_t6::{ObjRef, Value, Vm};

use super::autoplay_prison::{revive_at_body, zap_spot_within};
use super::tomb_test::{look_and_use, place_near};
use super::{Zm, frame, with_vm};
use crate::world::ClientId;

/// The panels a ghost shocks, by the level notify each one sends
/// (`afterlife_interact_object_think`: its `script_string`). The pulley goes
/// last: the sidequest only listens for it once the intro is over.
const SHOCKS: [&str; 5] = [
    "generator_panel_1_afterlife",
    "generator_panel_2_afterlife",
    "generator_panel_3_afterlife",
    "laundry_power_switch_afterlife",
    "master_key_pulley_",
];
const SHOCK_MS: i64 = 3500;
const NIXIE_AT: i64 = SHOCKS.len() as i64 * SHOCK_MS;
const NIXIE_MS: i64 = 5500;
const REVIVE_AT: i64 = 34_000;
const ELEVATOR_AT: i64 = 45_000;
const KEY_AT: i64 = 50_000;
const DOORS_AT: i64 = 54_000;
const DOOR_MS: i64 = 4000;
/// Two passes over the five doors; the second only tries a door still shut.
const DRYER_AT: i64 = DOORS_AT + 10 * DOOR_MS + 1000;
/// The parts picked up one by one (the dryer's 30 s cycle leaves the sheets
/// first): a 3 s try each, a new standing spot per try, at most 6 tries a part.
const PICK_AT: i64 = DRYER_AT + 40_000;
const PICK_MS: i64 = 3000;
const PICKS: i64 = 24;
const PICK_TRIES: i64 = 6;

fn field(vm: &mut Vm<World>, o: ObjRef, s: &str) -> Value {
    let k = vm.intern(s);
    vm.raw_field(o, k)
}

pub(super) fn level_flag(vm: &mut Vm<World>, name: &str) -> bool {
    let lvl = vm.level;
    match field(vm, lvl, "flag") {
        Value::Array(a) => a
            .get(&gsc_t6::Key::Str(vm.intern(name)))
            .is_some_and(|v| gsc_t6::truthy(&v)),
        _ => false,
    }
}

/// The first map entity whose `name` field reads `value`: (entnum, origin,
/// angles, model).
fn ent_by(world: &mut World, name: &str, value: &str) -> Option<(u32, [f32; 3], [f32; 3], String)> {
    with_vm(world, |vm, world| {
        let k = vm.intern(name);
        world.resource::<Zm>().ents.iter().find_map(|(n, e)| {
            let o = e.obj?;
            (vm.to_text(&vm.raw_field(o, k)) == value)
                .then(|| (*n, e.origin, e.angles, e.model.clone()))
        })
    })
    .flatten()
}

/// The key doors (`create_key_door_unitrigger`: static unitrigger stubs
/// carrying `n_door_index`): (index, origin, opened).
fn key_doors(world: &mut World) -> Vec<(i32, [f32; 3], bool)> {
    with_vm(world, |vm, _| {
        let lvl = vm.level;
        let Value::Object(u) = field(vm, lvl, "_unitriggers") else {
            return Vec::new();
        };
        let Value::Array(a) = field(vm, u, "trigger_stubs") else {
            return Vec::new();
        };
        let stubs: Vec<Value> = a.read().values_in_order().cloned().collect();
        stubs
            .into_iter()
            .filter_map(|v| {
                let Value::Object(s) = v else {
                    return None;
                };
                let Value::Int(i) = field(vm, s, "n_door_index") else {
                    return None;
                };
                let o = field(vm, s, "origin").as_vec3()?;
                let open = gsc_t6::truthy(&field(vm, s, "master_key_door_opened"));
                Some((i, o, open))
            })
            .collect()
    })
    .unwrap_or_default()
}

/// Use pressed half of every half second (a press, not a hold: the use
/// triggers fire on a press).
fn tap_use(world: &mut World, client: ClientId, at: [f32; 3], now: i64) {
    if now % 500 < 250 {
        look_and_use(world, client, at);
    } else {
        let eye = frame(world).player(client).map(|p| [p.origin[0], p.origin[1], p.origin[2] + 60.0]);
        if let Some(eye) = eye {
            super::set_player_view(world, client, super::autoplay::look(eye, at));
        }
    }
}

/// A floor spot `dist` from `c`, or from 55 or 110 under it (a thing that
/// hangs, or the middle of a trigger volume).
fn stand_by(world: &mut World, c: [f32; 3], dist: f32) -> Option<[f32; 3]> {
    [0.0, 55.0, 110.0]
        .iter()
        .find_map(|dz| place_near(world, [c[0], c[1], c[2] - dz], 0.0, dist))
}

fn origin_of(world: &mut World, client: ClientId) -> [f32; 3] {
    frame(world).player(client).map_or([0.0; 3], |p| p.origin)
}

/// Fire held this tick.
fn hold_fire(world: &mut World, client: ClientId) {
    let mut req = world.resource_mut::<crate::step::StepRequest>();
    for (id, cmd) in &mut req.input.cmds {
        if *id == client {
            cmd.buttons |= playerstate_iw4::buttons::ATTACK;
        }
    }
}

/// His afterlife meter (`manacur`, 200 full; `afterlife_mana_watch`).
fn mana(world: &mut World) -> String {
    with_vm(world, |vm, world| {
        let p = world.resource::<Zm>().players.get(&0).map(|p| p.obj)?;
        let v = field(vm, p, "manacur");
        Some(vm.to_text(&v))
    })
    .flatten()
    .unwrap_or_default()
}

/// Nixie tube `i`: (the digit it shows, the code's digit), from
/// `level.a_nixie_tube_code` and `level.a_nixie_tube_solution`.
fn nixie_digits(world: &mut World, i: i32) -> (Option<i32>, Option<i32>) {
    with_vm(world, |vm, _| {
        let lvl = vm.level;
        let mut digit = |name: &str| match field(vm, lvl, name) {
            Value::Array(a) => match a.get(&gsc_t6::Key::Int(i)) {
                Some(Value::Int(n)) => Some(n),
                _ => None,
            },
            _ => None,
        };
        (digit("a_nixie_tube_code"), digit("a_nixie_tube_solution"))
    })
    .unwrap_or((None, None))
}

/// A plane part (`a_piecespawns` of the plane craftable).
struct Part {
    name: String,
    /// Its model, while it lies in sight (not carried, not hidden).
    at: Option<[f32; 3]>,
    /// Its pick-up trigger's height (the piece's `height`).
    height: f32,
    taken: bool,
}

fn plane_parts(world: &mut World) -> Vec<Part> {
    with_vm(world, |vm, world| {
        let objs = |v: Value| -> Vec<ObjRef> {
            match v {
                Value::Array(a) => a.read().values_in_order().filter_map(Value::as_obj).collect(),
                _ => Vec::new(),
            }
        };
        let lvl = vm.level;
        for s in objs(field(vm, lvl, "a_uts_craftables")) {
            let Some(cs) = field(vm, s, "craftablespawn").as_obj() else { continue };
            let name = field(vm, cs, "craftable_name");
            if vm.to_text(&name) != "plane" {
                continue;
            }
            return objs(field(vm, cs, "a_piecespawns"))
                .into_iter()
                .map(|p| {
                    let m = field(vm, p, "model");
                    let shown = super::entnum(vm, &m).and_then(|n| world.resource::<Zm>().ents.get(&n).cloned())
                        .is_some_and(|e| !e.hidden && !e.invisible_to_all);
                    let at = super::origin_of(vm, world, &m).filter(|_| shown);
                    let height = field(vm, p, "unitrigger")
                        .as_obj()
                        .and_then(|u| field(vm, u, "script_height").as_float())
                        .unwrap_or(15.0);
                    let name = field(vm, p, "modelname");
                    Part {
                        name: vm.to_text(&name),
                        at,
                        height,
                        taken: gsc_t6::truthy(&field(vm, p, "in_shared_inventory"))
                            || gsc_t6::truthy(&field(vm, p, "crafted")),
                    }
                })
                .collect();
        }
        Vec::new()
    })
    .unwrap_or_default()
}

/// The middle of an entity's model box, else its origin.
fn aim_point(world: &mut World, n: u32, o: [f32; 3]) -> [f32; 3] {
    let e = world.resource::<Zm>().ents.get(&n).cloned();
    e.and_then(|e| super::brushes::model_box(world, &e))
        .map_or(o, |(lo, hi)| std::array::from_fn(|i| (lo[i] + hi[i]) * 0.5))
}

pub(crate) fn plane_test(world: &mut World, now: i64) {
    static FLY: OnceLock<Option<i64>> = OnceLock::new();
    let Some(fly) = *FLY.get_or_init(|| {
        std::env::var("IW4L_T6_PLANE")
            .ok()
            .map(|s| s.parse::<f32>().map_or(330_000, |s| (s * 1000.0) as i64))
    }) else {
        return;
    };
    piece_probe(world, now);
    static START: AtomicI64 = AtomicI64::new(i64::MIN);
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let mut start = START.load(Ordering::Relaxed);
    if start == i64::MIN {
        let held = frame(world).player(client).map_or(0, |p| p.weapon);
        if held == 0 || super::weapon_text(world, held) != "lightning_hands_zm" {
            return;
        }
        // The sidequest sets its puzzles up after the intro's black screen.
        if !with_vm(world, |vm, _| level_flag(vm, "initial_blackscreen_passed")).unwrap_or(false) {
            return;
        }
        start = now;
        START.store(now, Ordering::Relaxed);
        let m = mana(world);
        diag::info!(Sim, "bo2zm t6 plane test: a ghost at {:.1} s, meter {m}", now as f32 / 1000.0);
    }
    let rel = now - start;
    let first = |t: i64| (t..t + tick).contains(&rel);

    // The ghost's shocks.
    static PANEL: Mutex<String> = Mutex::new(String::new());
    if (0..NIXIE_AT).contains(&rel) {
        let k = (rel / SHOCK_MS) as usize;
        let t = rel % SHOCK_MS;
        let mut want = SHOCKS[k].to_owned();
        if want.ends_with('_') {
            let west = with_vm(world, |vm, _| {
                let lvl = vm.level;
                gsc_t6::truthy(&field(vm, lvl, "is_master_key_west"))
            })
            .unwrap_or(false);
            want.push_str(if west { "west" } else { "east" });
        }
        // By the notify it sends, else by its name (the generator panels
        // are `generator_panel_<n>`).
        let found = ent_by(world, "script_string", &want)
            .or_else(|| ent_by(world, "targetname", want.trim_end_matches("_afterlife")));
        let Some((n, o, a, model)) = found else {
            if t < tick {
                diag::warn!(Sim, "bo2zm t6 plane test: no panel {want}");
            }
            return;
        };
        let target = aim_point(world, n, o);
        if t < tick {
            match zap_spot_within(world, target, a[1], Some(o)) {
                Some(feet) => super::teleport_player(world, client, feet),
                None => diag::warn!(Sim, "bo2zm t6 plane test: no open side at {want}"),
            }
            diag::info!(
                Sim,
                "bo2zm t6 plane test: shock {want} ent{n} {model} at ({:.0} {:.0} {:.0})",
                o[0],
                o[1],
                o[2]
            );
            *PANEL.lock().unwrap() = model;
            return;
        }
        let me = origin_of(world, client);
        super::set_player_view(world, client, super::autoplay::look([me[0], me[1], me[2] + 60.0], target));
        if t >= SHOCK_MS - tick {
            let lvl = with_vm(world, |vm, _| {
                format!(
                    "key_found {} generator_challenge_completed {} panels {}",
                    level_flag(vm, "key_found"),
                    level_flag(vm, "generator_challenge_completed"),
                    {
                        let lvl = vm.level;
                        let v = field(vm, lvl, "n_generator_panels_active");
                        vm.to_text(&v)
                    }
                )
            })
            .unwrap_or_default();
            let m = mana(world);
            diag::info!(Sim, "bo2zm t6 plane test: after shocking {want}: model {model}; {lvl}; meter {m}");
        } else if now % 500 < 250 && model == *PANEL.lock().unwrap() {
            // Until it reacts (its model changes): every zap costs meter.
            hold_fire(world, client);
        }
        return;
    }

    // The nixie tubes: each shock counts its digit up one; the right code
    // drops the Citadel elevator (`nixie_tube_elevator_drops`).
    if (NIXIE_AT..REVIVE_AT).contains(&rel) {
        let i = ((rel - NIXIE_AT) / NIXIE_MS) as i32 + 1;
        let t = (rel - NIXIE_AT) % NIXIE_MS;
        if i > 3 {
            return;
        }
        let Some((n, o, a, _)) = ent_by(world, "targetname", &format!("nixie_tube_{i}")) else {
            if t < tick {
                diag::warn!(Sim, "bo2zm t6 plane test: no nixie tube {i}");
            }
            return;
        };
        let target = aim_point(world, n, o);
        let (shows, code) = nixie_digits(world, i);
        if t < tick {
            match zap_spot_within(world, target, a[1], Some(o)) {
                Some(feet) => super::teleport_player(world, client, feet),
                None => diag::warn!(Sim, "bo2zm t6 plane test: no open side at nixie tube {i}"),
            }
            diag::info!(
                Sim,
                "bo2zm t6 plane test: nixie tube {i} ent{n} at ({:.0} {:.0} {:.0}) shows {shows:?}, code {code:?}",
                o[0],
                o[1],
                o[2]
            );
            return;
        }
        let me = origin_of(world, client);
        super::set_player_view(world, client, super::autoplay::look([me[0], me[1], me[2] + 60.0], target));
        if t >= NIXIE_MS - tick {
            let solved = with_vm(world, |vm, _| level_flag(vm, "nixie_puzzle_solved")).unwrap_or(false);
            let m = mana(world);
            diag::info!(
                Sim,
                "bo2zm t6 plane test: after nixie tube {i}: shows {shows:?}, code {code:?}, nixie_puzzle_solved {solved}; meter {m}"
            );
        } else if now % 500 < 250 && shows.is_some() && shows != code {
            hold_fire(world, client);
        }
        return;
    }
    if (REVIVE_AT + tick..REVIVE_AT + 10_000).contains(&rel) {
        revive_at_body(world, rel, REVIVE_AT);
        return;
    }
    if first(REVIVE_AT + 10_000) {
        super::autoplay::open_all_zones(world);
        diag::info!(Sim, "bo2zm t6 plane test: every zone opened");
        return;
    }

    // The Citadel elevator's gate (`nixie_tube_elevator_door`): the rigging
    // rides the elevator down.
    if (ELEVATOR_AT..ELEVATOR_AT + 4000 + tick).contains(&rel) {
        let Some((_, o, _, _)) = ent_by(world, "targetname", "nixie_elevator_door") else {
            if first(ELEVATOR_AT) {
                diag::warn!(Sim, "bo2zm t6 plane test: no elevator gate");
            }
            return;
        };
        if first(ELEVATOR_AT) {
            if o[2] < -5000.0 {
                diag::info!(Sim, "bo2zm t6 plane test: elevator gate switched off (the elevator is up)");
            } else if let Some(feet) = stand_by(world, o, 30.0) {
                super::teleport_player(world, client, feet);
            }
            return;
        }
        if first(ELEVATOR_AT + 4000) {
            let done = with_vm(world, |vm, _| level_flag(vm, "nixie_puzzle_completed")).unwrap_or(false);
            let rigging = world
                .resource::<Zm>()
                .ents
                .values()
                .find(|e| e.model == "veh_t6_dlc_zombie_part_rigging")
                .map(|e| e.origin);
            diag::info!(
                Sim,
                "bo2zm t6 plane test: elevator gate at ({:.0} {:.0} {:.0}): nixie_puzzle_completed {done}, rigging at {rigging:?}",
                o[0],
                o[1],
                o[2]
            );
            return;
        }
        if o[2] > -5000.0 && rel < ELEVATOR_AT + 1500 {
            tap_use(world, client, o, now);
        }
        return;
    }

    // The warden's key, where the pulley let it down.
    if (KEY_AT..KEY_AT + 3000 + tick).contains(&rel) {
        let key = world
            .resource::<Zm>()
            .ents
            .values()
            .find(|e| e.model == "p6_zm_al_key" && !e.hidden && !e.invisible_to_all)
            .map(|e| e.origin);
        let Some(key) = key else {
            if first(KEY_AT) {
                diag::warn!(Sim, "bo2zm t6 plane test: no key lying");
            }
            return;
        };
        if first(KEY_AT) {
            // Its pickup reaches 32 units (`generate_zombie_craftable_piece`).
            if let Some(feet) = stand_by(world, key, 20.0) {
                super::teleport_player(world, client, feet);
            }
            diag::info!(Sim, "bo2zm t6 plane test: key at ({:.0} {:.0} {:.0})", key[0], key[1], key[2]);
            return;
        }
        if first(KEY_AT + 3000) {
            let found = with_vm(world, |vm, _| level_flag(vm, "key_found")).unwrap_or(false);
            diag::info!(Sim, "bo2zm t6 plane test: key_found {found}");
            return;
        }
        tap_use(world, client, key, now);
        return;
    }

    // The five key doors, twice: the second pass stands somewhere else
    // and only tries a door still shut (an opened door's stub is gone:
    // `unregister_unitrigger`).
    if (DOORS_AT..DOORS_AT + 10 * DOOR_MS).contains(&rel) {
        let n = (rel - DOORS_AT) / DOOR_MS;
        let (k, again) = ((n % 5) as i32 + 1, n >= 5);
        let t = (rel - DOORS_AT) % DOOR_MS;
        let Some((_, stub, _)) = key_doors(world).into_iter().find(|d| d.0 == k) else {
            if t < tick && !again {
                diag::warn!(Sim, "bo2zm t6 plane test: no key door {k}");
            }
            return;
        };
        let lock = ent_by(world, "targetname", &format!("masterkey_lock_{k}")).map_or(stub, |l| l.1);
        if t < tick {
            // The first pass stands 20 from the trigger's middle, the second
            // 40 from the lock on another side.
            let feet = if again {
                [180.0, 90.0, 270.0]
                    .iter()
                    .find_map(|yaw| place_near(world, [lock[0], lock[1], lock[2] - 10.0], *yaw, 40.0))
            } else {
                stand_by(world, stub, 20.0)
            };
            super::teleport_player(world, client, feet.unwrap_or(stub));
            return;
        }
        if t < 3000 {
            tap_use(world, client, lock, now);
            return;
        }
        if (3500..3500 + tick).contains(&t) {
            let open = key_doors(world).into_iter().find(|d| d.0 == k).is_some_and(|d| d.2);
            let hint = world.resource::<Zm>().hints.get(&0).cloned().unwrap_or_default();
            let me = origin_of(world, client);
            diag::info!(
                Sim,
                "bo2zm t6 plane test: key door {k}{} at ({:.0} {:.0} {:.0}) opened {open}, hint {hint:?}, he is at ({:.0} {:.0} {:.0})",
                if again { " again" } else { "" },
                stub[0],
                stub[1],
                stub[2],
                me[0],
                me[1],
                me[2]
            );
        }
        return;
    }

    // The dryer: its cycle leaves the sheets.
    if (DRYER_AT..DRYER_AT + 1500).contains(&rel) {
        let Some((_, o, _, _)) = ent_by(world, "targetname", "dryer_trigger") else {
            if first(DRYER_AT) {
                diag::warn!(Sim, "bo2zm t6 plane test: no dryer trigger");
            }
            return;
        };
        if first(DRYER_AT) {
            let feet = stand_by(world, o, 30.0).unwrap_or(o);
            super::teleport_player(world, client, feet);
            return;
        }
        tap_use(world, client, o, now);
        return;
    }
    for t in [DRYER_AT + 3000, DRYER_AT + 40_000] {
        if first(t) {
            let line = with_vm(world, |vm, _| {
                format!(
                    "dryer_cycle_active {} key_found {} generator_challenge_completed {}",
                    level_flag(vm, "dryer_cycle_active"),
                    level_flag(vm, "key_found"),
                    level_flag(vm, "generator_challenge_completed")
                )
            })
            .unwrap_or_default();
            diag::info!(Sim, "bo2zm t6 plane test: dryer +{}s: {line}", (t - DRYER_AT) / 1000);
        }
    }

    // The parts, picked up the way he would: stand a step away and look at
    // the part's pick-up trigger, which rides 12 over the part
    // (`piecestub_get_unitrigger_origin`; standing under it and looking
    // down faces away from it, and Deadshot's machine beside the steering
    // part wins).
    if (PICK_AT..PICK_AT + PICKS * PICK_MS).contains(&rel) {
        static TRIES: Mutex<Vec<(String, i64)>> = Mutex::new(Vec::new());
        static NOW: Mutex<Option<(String, [f32; 3])>> = Mutex::new(None);
        let t = (rel - PICK_AT) % PICK_MS;
        if t < tick {
            let parts = plane_parts(world);
            let mut tries = TRIES.lock().unwrap();
            let next = parts.iter().find(|p| {
                !p.taken
                    && p.at.is_some()
                    && tries.iter().find(|x| x.0 == p.name).is_none_or(|x| x.1 < PICK_TRIES)
            });
            let Some(p) = next else {
                *NOW.lock().unwrap() = None;
                if rel % 15_000 < tick {
                    let left: Vec<&str> = parts.iter().filter(|p| !p.taken).map(|p| p.name.as_str()).collect();
                    diag::info!(Sim, "bo2zm t6 plane test: no part left to pick up; not taken {left:?}");
                }
                return;
            };
            let n = match tries.iter_mut().find(|x| x.0 == p.name) {
                Some(x) => {
                    x.1 += 1;
                    x.1 - 1
                }
                None => {
                    tries.push((p.name.clone(), 1));
                    0
                }
            };
            let o = p.at.unwrap_or_default();
            let aim = [o[0], o[1], o[2] + 12.0 + p.height * 0.5];
            // Around the part a quarter turn a try, then the diagonals.
            let yaw = (90 * (n % 4) + 45 * (n / 4)) as f32;
            let dist = if n < 4 { 22.0 } else { 14.0 };
            let feet = place_near(world, o, yaw, dist)
                .or_else(|| place_near(world, [o[0], o[1], o[2] - 40.0], yaw, dist));
            diag::info!(
                Sim,
                "bo2zm t6 plane test: pick {} try {n}: part at ({:.0} {:.0} {:.0}), he stands at {:?}",
                p.name,
                o[0],
                o[1],
                o[2],
                feet.map(|f| f.map(|x| x as i32))
            );
            if let Some(f) = feet {
                super::teleport_player(world, client, f);
            }
            *NOW.lock().unwrap() = Some((p.name.clone(), aim));
            return;
        }
        let now_part = NOW.lock().unwrap().clone();
        let Some((name, aim)) = now_part else {
            return;
        };
        if t < 2200 {
            tap_use(world, client, aim, now);
            return;
        }
        if (2600..2600 + tick).contains(&t) {
            let taken = plane_parts(world).iter().find(|p| p.name == name).is_some_and(|p| p.taken);
            let hint = world.resource::<Zm>().hints.get(&0).cloned().unwrap_or_default();
            diag::info!(Sim, "bo2zm t6 plane test: pick {name}: taken {taken}, hint {hint:?}");
        }
        return;
    }

    // Takeoff.
    let t = now - fly;
    if (0..2000).contains(&t) {
        let Some((_, o, _, _)) = ent_by(world, "targetname", "plane_fly_trigger") else {
            if t < tick {
                diag::warn!(Sim, "bo2zm t6 plane test: no takeoff trigger");
            }
            return;
        };
        // trigger_off parks it 10000 under the map until the plane is built.
        if o[2] < -5000.0 {
            if t < tick {
                diag::info!(Sim, "bo2zm t6 plane test: takeoff trigger switched off (no plane built)");
            }
            return;
        }
        if t < tick {
            let feet = stand_by(world, o, 0.0)
                .or_else(|| stand_by(world, o, 30.0))
                .unwrap_or(o);
            super::teleport_player(world, client, feet);
            diag::info!(Sim, "bo2zm t6 plane test: takeoff trigger at ({:.0} {:.0} {:.0})", o[0], o[1], o[2]);
            return;
        }
        tap_use(world, client, o, now);
        return;
    }
    if (0..100_000 + tick).contains(&t) && t % 5000 < tick {
        let me = origin_of(world, client);
        let hint = world.resource::<Zm>().hints.get(&0).cloned().unwrap_or_default();
        let line = with_vm(world, |vm, _| {
            [
                "plane_built",
                "plane_boarded",
                "plane_departed",
                "plane_approach_bridge",
                "plane_zapped",
                "activate_player_zone_bridge",
            ]
            .iter()
            .map(|f| format!("{f} {}", u8::from(level_flag(vm, f))))
            .collect::<Vec<_>>()
            .join(", ")
        })
        .unwrap_or_default();
        diag::info!(
            Sim,
            "bo2zm t6 plane test: fly +{}s: {line}; he is at ({:.0} {:.0} {:.0}), hint {hint:?}",
            t / 1000,
            me[0],
            me[1],
            me[2]
        );
    }
}

/// IW4L_T6_PIECELOG=1: every 5 s from 140 s to 300 s, each plane part's
/// pick-up trigger as BO2's unitrigger system sees it (`_zm_unitrigger`: a
/// static stub is only a candidate while the zone it was filed under is
/// active, or from the dynamic list), the active zones, and the trigger
/// the player holds (`level._unitriggers.trigger_pool[0]`).
fn piece_probe(world: &mut World, now: i64) {
    if std::env::var_os("IW4L_T6_PIECELOG").is_none()
        || !(140_000..300_000).contains(&now)
        || now % 5000 >= i64::from(crate::MATCH_TICK_MS)
    {
        return;
    }
    let me = origin_of(world, ClientId(0));
    let lines = with_vm(world, |vm, world| {
        let lvl = vm.level;
        let objs = |v: Value| -> Vec<ObjRef> {
            match v {
                Value::Array(a) => a.read().values_in_order().filter_map(Value::as_obj).collect(),
                _ => Vec::new(),
            }
        };
        let zones: Vec<String> = match field(vm, lvl, "active_zone_names") {
            Value::Array(a) => a.read().values_in_order().map(|v| vm.to_text(v)).collect(),
            _ => Vec::new(),
        };
        let ut = field(vm, lvl, "_unitriggers").as_obj();
        let dynamic = ut.map(|u| objs(field(vm, u, "dynamic_stubs"))).unwrap_or_default();
        let held = ut
            .and_then(|u| match field(vm, u, "trigger_pool") {
                Value::Array(a) => a.get(&gsc_t6::Key::Int(0)).and_then(|v| v.as_obj()),
                _ => None,
            })
            .map(|t| {
                let s = field(vm, t, "stub").as_obj();
                let hint = s.map(|s| {
                    let v = field(vm, s, "hint_string");
                    vm.to_text(&v)
                });
                let o = s.and_then(|s| field(vm, s, "origin").as_vec3());
                format!("{hint:?} stub at {:?}", o.map(|o| o.map(|x| x as i32)))
            });
        let mut out = vec![format!(
            "active zones [{}]; dynamic stubs {}; he holds {held:?}",
            zones.join(" "),
            dynamic.len()
        )];
        for s in objs(field(vm, lvl, "a_uts_craftables")) {
            let Some(cs) = field(vm, s, "craftablespawn").as_obj() else { continue };
            let name = field(vm, cs, "craftable_name");
            if vm.to_text(&name) != "plane" {
                continue;
            }
            for p in objs(field(vm, cs, "a_piecespawns")) {
                let modelname = field(vm, p, "modelname");
                let m = field(vm, p, "model");
                let at = super::origin_of(vm, world, &m).map(|o| o.map(|x| x as i32));
                let Some(u) = field(vm, p, "unitrigger").as_obj() else {
                    out.push(format!("{} at {at:?}: no unitrigger", vm.to_text(&modelname)));
                    continue;
                };
                let uo = field(vm, u, "origin").as_vec3().map(|o| o.map(|x| x as i32));
                let zone = field(vm, u, "in_zone");
                let reg = field(vm, u, "registered");
                let r = field(vm, u, "test_radius_sq");
                let pt = objs(field(vm, u, "playertrigger")).len();
                out.push(format!(
                    "{} at {at:?}: stub at {uo:?} in_zone {} registered {} test_radius_sq {} dynamic {} player triggers {pt}",
                    vm.to_text(&modelname),
                    vm.to_text(&zone),
                    vm.to_text(&reg),
                    vm.to_text(&r),
                    dynamic.contains(&u)
                ));
            }
            break;
        }
        out
    })
    .unwrap_or_default();
    for l in lines {
        diag::info!(
            Sim,
            "bo2zm t6 piece probe {}s (he is at {:.0} {:.0} {:.0}): {l}",
            now / 1000,
            me[0],
            me[1],
            me[2]
        );
    }
}
