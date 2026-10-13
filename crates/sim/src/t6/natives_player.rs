//! Player methods: spawn, weapons, ammo, perks, controls, buttons, pose.

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};

use super::{
    Zm, arg, client, entnum, flag, frame, int, list, num, origin_of, text, tick, vec3, weapon,
    weapon_text,
};
use crate::script_player;
use crate::world::ClientId;

type R = Result<Value, String>;

/// A client system's state: "musicCmd" is the music (zm_nuked_amb.csc's
/// states: WAVE loops Nuketown's underscore, the rest are silence); others
/// are not drawn yet.
fn client_sys_state(world: &mut World, id: i32, state: &str, who: Option<u32>) {
    let name = usize::try_from(id)
        .ok()
        .and_then(|i| world.resource::<Zm>().client_sys.get(i).cloned())
        .unwrap_or_default();
    // A level notify for the client scripts (maps/mp/_utility::clientnotify
    // sends it this way): "znfg", Nuketown's game over, turns every local
    // player's world fog to bank 2 (zm_nuked.csc intermission_settings), so
    // the rocket shot sees the town.
    if name.eq_ignore_ascii_case("levelnotify") {
        if state == "znfg" {
            let clients: Vec<u32> = match who {
                Some(n) => vec![n],
                None => world.resource::<Zm>().players.keys().copied().collect(),
            };
            for n in clients {
                super::set_client_dvar(world, n, "bo2zm_fogbank", "2");
            }
        }
        return;
    }
    // Down: BO2's client script plays the heartbeat and the low loop
    // while "lsm" is "1" (audio laststand.rs).
    if name.eq_ignore_ascii_case("lsm") {
        let clients: Vec<u32> = match who {
            Some(n) => vec![n],
            None => world.resource::<Zm>().players.keys().copied().collect(),
        };
        for n in clients {
            super::set_client_dvar(world, n, "bo2zm_lsm", state);
        }
        return;
    }
    if !name.eq_ignore_ascii_case("musiccmd") {
        return;
    }
    let alias = match state.to_ascii_uppercase().as_str() {
        "WAVE" => "mus_nuked_underscore",
        _ => "",
    };
    let clients: Vec<u32> = match who {
        Some(n) => vec![n],
        None => world.resource::<Zm>().players.keys().copied().collect(),
    };
    for n in clients {
        super::set_client_dvar(world, n, "bo2zm_music", alias);
    }
}

fn controls(
    vm: &Vm<World>,
    world: &mut World,
    s: &Value,
    set: impl FnOnce(&mut crate::match_state::ScriptControls),
) -> R {
    let id = client(vm, world, s)?;
    let mut f = frame(world);
    if f.client_meta(id).is_some() {
        set(&mut f.client_meta_mut(id).controls);
    }
    Ok(Value::Undefined)
}

/// A shellshock ends (`stopshellshock` or its time ran out): the timer, the
/// SHELLSHOCKED flag and the speed factor it put on the scale go.
pub(super) fn end_shock(world: &mut World, n: u32, why: &str) {
    let id = ClientId(n);
    let taken = world
        .resource_mut::<Zm>()
        .players
        .get_mut(&n)
        .and_then(|p| {
            let name = p.shock.take()?.0;
            let factor = std::mem::replace(&mut p.shock_factor, 1.0);
            Some((name, factor))
        });
    let Some((name, factor)) = taken else {
        // Nothing this script started: still clear the timer.
        if let Some(ps) = frame(world).player_mut(id) {
            ps.shellshock_duration = 0;
            ps.pm_flags &= !playerstate_iw4::pm_flags::SHELLSHOCKED;
        }
        return;
    };
    let level = crate::level_time_ms(tick(world));
    let mut f = frame(world);
    if let Some(ps) = f.player_mut(id) {
        ps.shellshock_duration = 0;
        ps.pm_flags &= !playerstate_iw4::pm_flags::SHELLSHOCKED;
        ps.move_speed_scale_multiplier /= factor;
    }
    f.client_meta_mut(id).shellshock = None;
    diag::info!(
        Sim,
        "bo2mp shellshock end: client {n} '{name}' at {level} ms ({why}), speed x{factor} off"
    );
}

/// BO2's `allow*`: sets the switch and returns whether it was allowed before
/// (scripts keep it: `prone = self allowprone( 0 ); ... self allowprone( prone );`).
fn allow(
    vm: &Vm<World>,
    world: &mut World,
    s: &Value,
    swap: impl FnOnce(&mut crate::match_state::ScriptControls) -> bool,
) -> R {
    let id = client(vm, world, s)?;
    let mut f = frame(world);
    let was_off = f.client_meta(id).is_some() && swap(&mut f.client_meta_mut(id).controls);
    Ok(Value::Int(if was_off { 0 } else { 1 }))
}

pub(super) fn bind(vm: &mut Vm<World>) {
    macro_rules! m {
        ($name:literal, $body:expr) => {
            vm.bind($name, true, $body);
        };
    }
    m!("spawn", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let origin = vec3(a, 0)?;
        let angles = super::arg(a, 1).as_vec3().unwrap_or([0.0; 3]);
        let state = world
            .resource::<Zm>()
            .players
            .get(&id.0)
            .map_or_else(|| "playing".to_owned(), |p| p.sessionstate.clone());
        // bo2mp: where players come in, for maps from other games (a game
        // type without its spawn classes put them at the world origin):
        // the first few spawns, every one at the origin, every one in solid.
        if state == "playing" && world.resource::<Zm>().mp {
            use crate::bullet_collision::{MASK_PLAYER_SOLID, PLAYER_MAXS, PLAYER_MINS};
            static SEEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let seen = SEEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let at_origin = origin.iter().all(|v| v.abs() < 1.0);
            let in_solid = frame(world)
                .trace_clip(origin, origin, PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID)
                .startsolid
                != 0;
            if at_origin || in_solid {
                diag::warn!(
                    Sim,
                    "bo2mp spawn: client {} at ({:.0} {:.0} {:.0}){}{}",
                    id.0,
                    origin[0],
                    origin[1],
                    origin[2],
                    if at_origin { " — the world origin (no spawn point found)" } else { "" },
                    if in_solid { " — inside solid" } else { "" }
                );
            } else if seen < 12 {
                diag::info!(
                    Sim,
                    "bo2mp spawn: client {} at ({:.0} {:.0} {:.0}) yaw {:.0}",
                    id.0,
                    origin[0],
                    origin[1],
                    origin[2],
                    angles[1]
                );
            }
        }
        let t = tick(world);
        script_player::spawn(&mut frame(world), t, id, origin, angles, &state);
        Ok(Value::Undefined)
    });
    m!("giveweapon", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let name = text(vm, a, 0);
        let w = weapon(world, &name)?;
        if w == 0 {
            return Ok(Value::Undefined);
        }
        // BO2's engine holds a melee weapon it gives as his melee weapon
        // (the knife, the Bowie Knife, the Galvaknuckles): the one he stabs
        // with, replacing the last.
        if frame(world).weapon_is_melee_only(w)
            && let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0)
        {
            p.melee_weapon = Some(name.clone());
        }
        script_player::give_weapon(&mut frame(world), id, w, false)?;
        // bo2mp: its camo (the options `calcweaponoptions` made).
        wear_camo(world, id, w, arg(a, 2).as_int().unwrap_or(0) & 0xff);
        // bo2zm: Black Ops II's engine puts a grenade it gives into its
        // offhand slot (frag/Semtex lethal on the grenade key, the monkey
        // bomb tactical); IW4's scripts named the class themselves.
        let mut f = frame(world);
        let class = f.equipment_facts_for(w).map_or(0, |eq| eq.offhand_class);
        if let Some(ps) = f.player_mut(id) {
            match class {
                1 => ps.offhand_primary = 1,
                2 | 3 => ps.offhand_secondary = class,
                _ => {}
            }
        }
        Ok(Value::Undefined)
    });
    m!("takeweapon", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        if held_melee(world, id) == w
            && let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0)
        {
            p.melee_weapon = None;
        }
        script_player::take_weapon(&mut frame(world), id, w);
        Ok(Value::Undefined)
    });
    m!("takeallweapons", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            p.melee_weapon = None;
        }
        script_player::take_all_weapons(&mut frame(world), id);
        Ok(Value::Undefined)
    });
    m!("hasweapon", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let Ok(w) = weapon(world, &text(vm, a, 0)) else {
            return Ok(Value::Int(0));
        };
        Ok(Value::bool(
            w != 0
                && (held_melee(world, id) == w
                    || script_player::has_weapon(&frame(world), id, w)),
        ))
    });
    m!("getcurrentweapon", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let w = frame(world).player(id).map_or(0, |ps| ps.weapon);
        let n = weapon_text(world, w);
        Ok(vm.string(&n))
    });
    m!("getcurrentoffhand", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let w = frame(world)
            .player(id)
            .map_or(0, |ps| ps.offhand_primary as u32);
        let n = weapon_text(world, w);
        Ok(vm.string(&n))
    });
    // True when he has it to switch to (a bot calling in a map-picked
    // scorestreak goes on only then).
    m!("switchtoweapon", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        script_player::switch_to_weapon(&mut frame(world), id, w);
        Ok(Value::bool(
            w != 0 && script_player::has_weapon(&frame(world), id, w),
        ))
    });
    m!("switchtoweaponimmediate", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        script_player::switch_to_weapon(&mut frame(world), id, w);
        Ok(Value::Undefined)
    });
    m!("setspawnweapon", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        script_player::switch_to_weapon(&mut frame(world), id, w);
        Ok(Value::Undefined)
    });
    m!("getweaponslist", |vm, world, s, _| weapon_list(
        vm,
        world,
        s,
        script_player::WeaponList::All
    ));
    m!("getweaponslistprimaries", |vm, world, s, _| weapon_list(
        vm,
        world,
        s,
        script_player::WeaponList::Primaries
    ));
    m!("setweaponammoclip", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        let mut f = frame(world);
        // The left gun of a held dual-wield pair: its clip.
        match script_player::dual_wield_right_of(&f, id, w) {
            Some(right) => script_player::set_left_clip(&mut f, id, right, int(a, 1)?),
            None => script_player::set_ammo_clip(&mut f, id, w, int(a, 1)?),
        }
        Ok(Value::Undefined)
    });
    m!("setweaponammostock", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        script_player::set_ammo_stock(&mut frame(world), id, w, int(a, 1)?);
        Ok(Value::Undefined)
    });
    m!("getweaponammoclip", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        let f = frame(world);
        Ok(Value::Int(
            match script_player::dual_wield_right_of(&f, id, w) {
                Some(right) => script_player::left_clip(&f, id, right),
                None => script_player::ammo_clip(&f, id, w),
            },
        ))
    });
    m!("getweaponammostock", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        Ok(Value::Int(script_player::ammo_stock(&frame(world), id, w)))
    });
    m!("getammocount", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        let f = frame(world);
        Ok(Value::Int(
            script_player::ammo_clip(&f, id, w) + script_player::ammo_stock(&f, id, w),
        ))
    });
    m!("givemaxammo", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        script_player::give_max_ammo(&mut frame(world), id, w);
        Ok(Value::Undefined)
    });
    // T6 gives a full clip on top of the start ammo, all of which goes to the
    // stock: real BO2 starts the M1911 (clip 8, start 4 clips) at 8/32. The
    // IW4 rule takes the clip out of it (8/24), and then `ammo_give` sees the
    // stock under `weaponstartammo` and charges for ammo on a gun just bought
    // off the wall. Grenades and mines keep their own rule.
    m!("givestartammo", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        let mut f = frame(world);
        let offhand = f.equipment_facts_for(w).is_some_and(|eq| eq.is_offhand());
        match f.combat_facts_for(w) {
            Some(facts) if !offhand => {
                let start = facts.start_ammo.max(0);
                let clip = if start > 0 { facts.clip_size.max(0) } else { 0 };
                script_player::set_ammo_clip(&mut f, id, w, clip);
                script_player::set_ammo_stock(&mut f, id, w, start);
            }
            _ => script_player::give_start_ammo(&mut f, id, w),
        }
        Ok(Value::Undefined)
    });
    m!("setperk", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let p = text(vm, a, 0);
        script_player::set_perk(&mut frame(world), id, &p, true);
        if let Some(pl) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            pl.perks.insert(p);
        }
        Ok(Value::Undefined)
    });
    m!("unsetperk", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let p = text(vm, a, 0);
        script_player::set_perk(&mut frame(world), id, &p, false);
        if let Some(pl) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            pl.perks.remove(&p);
        }
        Ok(Value::Undefined)
    });
    // BO2's poisoned player-state flag: the Denizen sets it while it holds
    // him (_zm_ai_screecher), TranZit's fog gas while he coughs.
    fn poison(vm: &mut Vm<World>, world: &mut World, s: &Value, on: bool) -> Result<Value, String> {
        let id = client(vm, world, s)?;
        if let Some(pl) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            pl.poisoned = on;
        }
        Ok(Value::Undefined)
    }
    m!("startpoisoning", |vm, world, s, _| poison(vm, world, s, true));
    m!("stoppoisoning", |vm, world, s, _| poison(vm, world, s, false));
    m!("ispoisoned", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::bool(world.resource::<Zm>().players.get(&id.0).is_some_and(|p| p.poisoned)))
    });
    m!("clearperks", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        script_player::clear_perks(&mut frame(world), id);
        if let Some(pl) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            pl.perks.clear();
        }
        Ok(Value::Undefined)
    });
    m!("hasperk", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let p = text(vm, a, 0);
        Ok(Value::bool(
            world
                .resource::<Zm>()
                .players
                .get(&id.0)
                .is_some_and(|pl| pl.perks.contains(&p)),
        ))
    });
    // Every perk he has, by name (Buried's minigame loadout stops each one).
    m!("getperks", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let names: Vec<String> = world
            .resource::<Zm>()
            .players
            .get(&id.0)
            .map(|pl| pl.perks.iter().cloned().collect())
            .unwrap_or_default();
        let out = names.iter().map(|n| vm.string(n)).collect();
        Ok(list(out))
    });
    // No gun has a fuel tank (weaponfuellife is 0), so its fuel is 0 and
    // setting it does nothing (the weapon fridge, Vulture Aid, the Time Bomb).
    m!("getweaponammofuel", |_, _, _, _| Ok(Value::Int(0)));
    m!("setweaponammofuel", |_, _, _, _| Ok(Value::Undefined));
    m!("freezecontrols", |vm, world, s, a| {
        let on = flag(a, 0, true);
        controls(vm, world, s, |c| c.frozen = on)
    });
    // Die Rise's Sliquifier goo: `player forceslick( player.slick_count )`,
    // a count above 0 makes the ground ice under him (no friction).
    m!("forceslick", |vm, world, s, a| {
        let on = flag(a, 0, false);
        controls(vm, world, s, |c| c.slick = on)
    });
    m!("disableweapons", |vm, world, s, _| controls(
        vm,
        world,
        s,
        |c| c.weapons_disabled = true
    ));
    m!("enableweapons", |vm, world, s, _| controls(
        vm,
        world,
        s,
        |c| c.weapons_disabled = false
    ));
    m!("disableoffhandweapons", |vm, world, s, _| controls(
        vm,
        world,
        s,
        |c| c.offhands_disabled = true
    ));
    m!("enableoffhandweapons", |vm, world, s, _| controls(
        vm,
        world,
        s,
        |c| c.offhands_disabled = false
    ));
    m!("disableweaponcycling", |vm, world, s, _| controls(
        vm,
        world,
        s,
        |c| c.switch_disabled = true
    ));
    m!("enableweaponcycling", |vm, world, s, _| controls(
        vm,
        world,
        s,
        |c| c.switch_disabled = false
    ));
    // bo2mp: BO2's `allow*` (false drops the request in constrain_cmd;
    // the answer is whether it was allowed before, as BO2's).
    m!("allowprone", |vm, world, s, a| {
        let on = flag(a, 0, true);
        allow(vm, world, s, |c| {
            std::mem::replace(&mut c.prone_disabled, !on)
        })
    });
    m!("allowcrouch", |vm, world, s, a| {
        let on = flag(a, 0, true);
        allow(vm, world, s, |c| {
            std::mem::replace(&mut c.crouch_disabled, !on)
        })
    });
    m!("allowstand", |vm, world, s, a| {
        let on = flag(a, 0, true);
        allow(vm, world, s, |c| {
            std::mem::replace(&mut c.stand_disabled, !on)
        })
    });
    m!("allowsprint", |vm, world, s, a| {
        let on = flag(a, 0, true);
        allow(vm, world, s, |c| {
            std::mem::replace(&mut c.sprint_disabled, !on)
        })
    });
    m!("allowmelee", |vm, world, s, a| {
        let on = flag(a, 0, true);
        allow(vm, world, s, |c| {
            std::mem::replace(&mut c.melee_disabled, !on)
        })
    });
    m!("allowads", |vm, world, s, a| {
        let on = flag(a, 0, true);
        allow(vm, world, s, |c| {
            std::mem::replace(&mut c.ads_disabled, !on)
        })
    });
    // bo2mp: `setmovespeedscale(f)` is the player's own scale; a shellshock's
    // `bg_shock_movement` rides on top while it lasts (`getmovespeedscale`
    // gives his own back). The feel window reads
    // `PlayerState::move_speed_scale_multiplier` (the product).
    m!("setmovespeedscale", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let scale = num(a, 0)?;
        let factor = world
            .resource::<Zm>()
            .players
            .get(&id.0)
            .map_or(1.0, |p| p.shock_factor);
        if let Some(ps) = frame(world).player_mut(id) {
            ps.move_speed_scale_multiplier = scale * factor;
        }
        diag::info!(
            Sim,
            "bo2mp setmovespeedscale: client {} scale {scale} (shock x{factor} = {})",
            id.0,
            scale * factor
        );
        Ok(Value::Undefined)
    });
    m!("getmovespeedscale", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let factor = world
            .resource::<Zm>()
            .players
            .get(&id.0)
            .map_or(1.0, |p| p.shock_factor);
        Ok(Value::Float(
            frame(world)
                .player(id)
                .map_or(1.0, |ps| ps.move_speed_scale_multiplier / factor),
        ))
    });
    // bo2mp: BO2's shellshock (shock/<name>.shock): the timer and duration
    // on the player state, the file's screen / look / sound / movement
    // values to his client. Our flashed overlay, look limits and tinnitus
    // run from those; the movement scale is the file's, not a constant.
    m!("shellshock", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let name = text(vm, a, 0).to_ascii_lowercase();
        let seconds = num(a, 1)?;
        if seconds < 0.0 {
            return Err(format!("shellshock duration {seconds} is negative"));
        }
        let Some(mut shock) = frame(world).shock(&name).cloned() else {
            // bo2zm: a missing shock file is not a script error (a perk
            // machine's `shellshock("default", 1.5)` must not stop its thread).
            diag::warn!(Sim, "t6 shellshock '{name}': no shock file");
            return Ok(Value::Undefined);
        };
        let now = crate::level_time_ms(tick(world));
        let index = {
            let mut zm = world.resource_mut::<Zm>();
            let at = match zm.shock_names.iter().position(|n| *n == name) {
                Some(at) => at,
                None => {
                    zm.shock_names.push(name.clone());
                    zm.shock_names.len() - 1
                }
            };
            at as i32 + 1
        };
        let new_factor = if shock.movement_scale > 0.0 {
            shock.movement_scale
        } else {
            1.0
        };
        let old_factor = world
            .resource::<Zm>()
            .players
            .get(&id.0)
            .map_or(1.0, |p| p.shock_factor);
        let duration = (seconds * 1000.0) as i32;
        // BO2's file gives a scale, not a switch: IW4's fixed 0.4 stays off.
        shock.movement = false;
        let (look, sound, kick) = (
            shock.look.affect,
            shock.sound.loop_alias.clone(),
            shock.view_kick_radius,
        );
        let flashed = shock.screen_type == hud_iw4::SCREEN_BLEND_FLASHED;
        let mut f = frame(world);
        let Some(ps) = f.player_mut(id) else {
            return Ok(Value::Undefined);
        };
        ps.shellshock_index = index;
        ps.shellshock_time = now;
        ps.shellshock_duration = duration;
        ps.pm_flags |= playerstate_iw4::pm_flags::SHELLSHOCKED;
        ps.move_speed_scale_multiplier = ps.move_speed_scale_multiplier / old_factor * new_factor;
        f.client_meta_mut(id).shellshock = Some(shock);
        if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            p.shock = Some((name.clone(), now.wrapping_add(duration)));
            p.shock_factor = new_factor;
        }
        diag::info!(
            Sim,
            "bo2mp shellshock: client {} '{name}' {seconds}s from {now} ms (index {index}, flash {flashed}, look {look}, loop '{sound}', speed x{new_factor}, view kick {kick})",
            id.0
        );
        Ok(Value::Undefined)
    });
    m!("stopshellshock", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        end_shock(world, id.0, "stopshellshock");
        Ok(Value::Undefined)
    });
    m!("allowjump", |vm, world, s, a| {
        let on = flag(a, 0, true);
        controls(vm, world, s, |c| c.jump_disabled = !on)
    });
    m!("getplayerangles", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::Vec3(
            frame(world).player(id).map_or([0.0; 3], |ps| ps.viewangles),
        ))
    });
    // Last stand: up again (`reviveplayer`: full health, standing).
    m!("reviveplayer", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            p.laststand = false;
        }
        if let Some(ps) = frame(world).player_mut(id) {
            use playerstate_iw4::eflags::{DUCK, PRONE};
            use playerstate_iw4::{PM_TYPE_LAST_STAND, pm_flags};
            // Out of last stand now, not at the next sync: a move still run
            // as last stand put the prone flag back, and the last stand's
            // eye height read as prone, so he got up lying prone (10-09
            // trace). As the engine's own revive: crouched, and the move's
            // own room check stands him up the next tick. The stand event
            // resets his client's stance keys.
            let before = (ps.pm_type, ps.pm_flags, ps.view_height_target, ps.view_height_lerp_time);
            if ps.pm_type == PM_TYPE_LAST_STAND {
                ps.pm_type = 0;
            }
            ps.pm_flags = (ps.pm_flags & !(pm_flags::LAST_STAND | pm_flags::PRONE)) | pm_flags::CROUCH;
            ps.e_flags = (ps.e_flags & !PRONE) | DUCK;
            ps.view_height_target = movement_iw4::view_height::CROUCH;
            movement_iw4::add_predictable_event(ps, 6, 0);
            ps.health = ps.max_health.max(1);
            diag::info!(
                Sim,
                "bo2zm t6: reviveplayer {} (pm_type, pm_flags, target, lerp_time) before {before:?} after pm_flags {:#x}",
                id.0,
                ps.pm_flags
            );
        }
        Ok(Value::Undefined)
    });
    m!("undolaststand", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            p.laststand = false;
        }
        Ok(Value::Undefined)
    });
    m!("depthinwater", |_, _, _, _| Ok(Value::Float(0.0)));
    // bo2mp: the gun on his back (BO2's _weapons.gsc stow_on_back): every
    // client draws its world model on his tag_stowed_back.
    m!("setstowedweapon", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let name = text(vm, a, 0);
        let stowed = super::weapon(world, &name).ok().filter(|&w| w != 0).map(|w| (w, name));
        super::playeranim::edit_body(world, id.0, |l| l.stowed = stowed);
        Ok(Value::Undefined)
    });
    m!("clearstowedweapon", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        super::playeranim::edit_body(world, id.0, |l| l.stowed = None);
        Ok(Value::Undefined)
    });
    // Camo/lens/reticle options for giveweapon (Pack-a-Punch): none drawn yet.
    m!("calcweaponoptions", |_, _, _, _| Ok(Value::Int(0)));
    vm.bind("getweaponstowedmodel", false, |vm, _, _, _| {
        Ok(vm.string(""))
    });
    for name in [
        "startrevive",
        "stoprevive",
        "docowardswayanims",
        "setrevivehintstring",
        // Stats only (Mob's afterlife weapon save/restore calls it).
        "seteverhadweaponall",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    // Client systems (maps/mp/_utility::registerclientsys and
    // setclientsysstate call these): each registered name gets an id; a
    // state set on one goes to every client, or as a method to one.
    vm.bind("clientsysregister", false, |vm, world, _, a| {
        let name = text(vm, a, 0);
        let mut zm = world.resource_mut::<Zm>();
        let id = match zm
            .client_sys
            .iter()
            .position(|n| n.eq_ignore_ascii_case(&name))
        {
            Some(i) => i,
            None => {
                zm.client_sys.push(name);
                zm.client_sys.len() - 1
            }
        };
        Ok(Value::Int(id as i32))
    });
    vm.bind("clientsyssetstate", false, |vm, world, _, a| {
        let id = arg(a, 0).as_int().unwrap_or(-1);
        let state = text(vm, a, 1);
        super::highrise::client_notify(vm, world, id, &state);
        client_sys_state(world, id, &state, None);
        Ok(Value::Undefined)
    });
    vm.bind("clientsyssetstate", true, |vm, world, s, a| {
        let id = arg(a, 0).as_int().unwrap_or(-1);
        let state = text(vm, a, 1);
        let who = entnum(vm, s);
        super::highrise::client_notify(vm, world, id, &state);
        client_sys_state(world, id, &state, who);
        Ok(Value::Undefined)
    });
    // A player moved by a script (spawning, a game-over fall).
    m!("setorigin", |vm, world, s, a| {
        let to = vec3(a, 0)?;
        match client(vm, world, s) {
            Ok(id) => super::teleport_player(world, id, to),
            Err(_) => {
                if let Some(n) = super::entnum(vm, s)
                    && let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n)
                {
                    e.origin = to;
                }
            }
        }
        Ok(Value::Undefined)
    });
    m!("setplayerangles", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        super::set_player_view(world, id, vec3(a, 0)?);
        Ok(Value::Undefined)
    });
    m!("setvelocity", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let v = vec3(a, 0)?;
        if v.iter().map(|c| c * c).sum::<f32>() > 300.0 * 300.0 {
            // A script launch (Trample Steam, a Jumping Jack hit): rare, so logged for tests.
            diag::info!(Sim, "bo2zm t6 setvelocity client {id:?} {v:?}");
        }
        if let Some(ps) = frame(world).player_mut(id) {
            ps.velocity = v;
        }
        Ok(Value::Undefined)
    });
    // player applyknockback(damage, dir): the push the engine gives a hit
    // (TranZit throws you off a zombie's head, Grief's shots shove). Damage
    // over 200 pushes no harder; 1000 is g_knockback, 250 the player's mass.
    m!("applyknockback", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let damage = super::num(a, 0)?.clamp(0.0, 200.0);
        let dir = vec3(a, 1)?;
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if len > 0.0 {
            let push = 1000.0 * damage / 250.0 / len;
            if let Some(ps) = frame(world).player_mut(id) {
                for i in 0..3 {
                    ps.velocity[i] += dir[i] * push;
                }
            }
        }
        Ok(Value::Undefined)
    });
    m!("getstance", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let st = script_player::stance(&frame(world), id);
        Ok(vm.string(st))
    });
    m!("setstance", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let st = text(vm, a, 0);
        if let Some(ps) = frame(world).player_mut(id) {
            use playerstate_iw4::eflags::{DUCK, PRONE};
            use playerstate_iw4::pm_flags;
            ps.e_flags &= !(DUCK | PRONE);
            ps.pm_flags &= !(pm_flags::CROUCH | pm_flags::PRONE);
            let (e, pm, event) = match st.as_str() {
                "crouch" => (DUCK, pm_flags::CROUCH, 7),
                "prone" => (PRONE, pm_flags::PRONE, 8),
                _ => (0, 0, 6),
            };
            ps.e_flags |= e;
            ps.pm_flags |= pm;
            // The forced-stance event sets the client's own stance keys
            // (else its next move stands straight back up).
            movement_iw4::add_predictable_event(ps, event, 0);
        }
        Ok(Value::Undefined)
    });
    m!("isonground", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::bool(frame(world).player(id).is_some_and(|ps| {
            ps.ground_entity_num != playerstate_iw4::ENTITYNUM_NONE
        })))
    });
    // The entity under the feet (any entity: Die Rise asks it of zombies,
    // equipment, buildable pieces; Buried of a thrown grenade), undefined
    // in the air. A script brush model or platform there is that entity
    // (`riders::ground_ent`); the map itself is `level` (_zm_buildables
    // checks `landed_on == level`), and `level getgroundent()` is level, so
    // object_is_on_elevator's walk down stops. _zm_utility::is_jumping is
    // !isdefined(getgroundent()): Afterlife's fake death waits on it.
    m!("getgroundent", |vm, world, s, _| {
        if s.as_obj() == Some(vm.level) {
            return Ok(Value::Object(vm.level));
        }
        let Some(origin) = origin_of(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        // Not a player, in or on a vehicle and not linked to it (a zombie
        // that climbed into Tranzit's bus; one at a window is linked): that
        // vehicle. The bus's floor is not the map's, so the trace under such
        // a zombie found nothing and it was in the air: busupdatenearzombies
        // never counted it on the bus, its entering_bus stayed set and
        // zombiemoveonbus never ran (runs/pr4c: they stood by the driver).
        let vehicle = if client(vm, world, s).is_err() {
            entnum(vm, s)
                .filter(|n| world.resource::<Zm>().ents.get(n).is_some_and(|e| e.link.is_none()))
                .and_then(|_| super::riders::vehicle_under(world, origin))
        } else {
            None
        };
        let grounded = vehicle.is_some() || match client(vm, world, s) {
            Ok(id) => frame(world)
                .player(id)
                .is_some_and(|ps| ps.ground_entity_num != playerstate_iw4::ENTITYNUM_NONE),
            // Not a player: the map within a few units under it.
            Err(_) => {
                let down = [origin[0], origin[1], origin[2] - 12.0];
                let t = frame(world).trace_world(
                    [origin[0], origin[1], origin[2] + 2.0],
                    down,
                    [0.0; 3],
                    [0.0; 3],
                    crate::bullet_collision::MASK_PLAYER_SOLID,
                );
                t.fraction < 1.0 || t.startsolid != 0
            }
        };
        if !grounded {
            return Ok(Value::Undefined);
        }
        let under = vehicle.or_else(|| super::riders::ground_ent(world, origin));
        let zm = world.resource::<Zm>();
        if let Some(n) = under
            && std::env::var_os("IW4L_T6_RIDELOG").is_some()
        {
            // Once per (asker, ground) pair.
            static SEEN: std::sync::Mutex<Vec<(u32, u32)>> = std::sync::Mutex::new(Vec::new());
            let me = entnum(vm, s).unwrap_or(u32::MAX);
            let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
            if !seen.contains(&(me, n)) {
                seen.push((me, n));
                diag::info!(
                    Sim,
                    "bo2zm t6 getgroundent ent{me} -> ent{n} {} at ({:.0} {:.0} {:.0})",
                    zm.ents.get(&n).map_or("", |e| e.classname.as_str()),
                    origin[0],
                    origin[1],
                    origin[2]
                );
            }
        }
        Ok(Value::Object(
            under
                .and_then(|n| zm.ents.get(&n))
                .and_then(|e| e.obj)
                .unwrap_or(vm.level),
        ))
    });
    // The moving platform he stands on (`riders`), undefined off one:
    // Tranzit counts the bus's riders by it.
    m!("getmoverent", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let zm = world.resource::<Zm>();
        Ok(zm
            .riders
            .on
            .get(&id.0)
            .and_then(|n| zm.ents.get(n))
            .and_then(|e| e.obj)
            .map_or(Value::Undefined, Value::Object))
    });
    m!("isonladder", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::bool(frame(world).player(id).is_some_and(|ps| {
            ps.pm_flags & playerstate_iw4::pm_flags::LADDER != 0
        })))
    });
    m!("issprinting", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::bool(frame(world).player(id).is_some_and(|ps| {
            ps.pm_flags & playerstate_iw4::pm_flags::SPRINTING != 0
        })))
    });
    m!("ismantling", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::bool(frame(world).player(id).is_some_and(|ps| {
            ps.pm_flags & playerstate_iw4::pm_flags::MANTLE != 0
        })))
    });
    macro_rules! button {
        ($($name:literal => $mask:expr),* $(,)?) => {$(
            vm.bind($name, true, |vm, world, s, _| {
                let id = client(vm, world, s)?;
                let held = script_player::buttons(&mut frame(world), id);
                Ok(Value::bool(held & ($mask) != 0))
            });
        )*};
    }
    {
        use playerstate_iw4::buttons::*;
        button!(
            "attackbuttonpressed" => ATTACK,
            "usebuttonpressed" => USE | USE_RELOAD,
            "meleebuttonpressed" => MELEE_CHARGE,
            "fragbuttonpressed" => FRAG,
            "secondaryoffhandbuttonpressed" => SMOKE,
            "adsbuttonpressed" => ADS,
            "jumpbuttonpressed" => JUMP,
            "sprintbuttonpressed" => SPRINT,
            "stancebuttonpressed" => STANCE_HELD | CROUCH | PRONE,
            "throwbuttonpressed" => THROW,
        );
    }
    // Buttons our commands do not carry (d-pad slots, inventory, vehicle
    // seat): never held. Mob of the Dead's Afterlife polls them.
    for name in [
        "actionslotonebuttonpressed",
        "actionslottwobuttonpressed",
        "actionslotthreebuttonpressed",
        "actionslotfourbuttonpressed",
        "inventorybuttonpressed",
        "changeseatbuttonpressed",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::bool(false)));
    }
    m!("isfiring", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let held = script_player::buttons(&mut frame(world), id);
        Ok(Value::bool(held & playerstate_iw4::buttons::ATTACK != 0))
    });
    // Holding melee (Origins' staffs check it before a staff melee).
    m!("ismeleeing", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let held = script_player::buttons(&mut frame(world), id);
        Ok(Value::bool(
            held & playerstate_iw4::buttons::MELEE_CHARGE != 0,
        ))
    });
    // self islinkedto(ent): linked (linkto / playerlinkto) to that entity.
    m!("islinkedto", |vm, world, s, a| {
        let (Some(me), Some(parent)) = (entnum(vm, s), entnum(vm, arg(a, 0))) else {
            return Ok(Value::bool(false));
        };
        let zm = world.resource::<Zm>();
        let to = zm
            .players
            .get(&me)
            .and_then(|p| p.linked)
            .or_else(|| zm.ents.get(&me).and_then(|e| e.link.map(|l| l.0)));
        Ok(Value::bool(to == Some(parent)))
    });
    // Mob of the Dead's Afterlife saves the arms model and puts it back
    // (setviewmodel is not drawn yet): any text keeps the scripts going.
    m!("getviewmodel", |vm, _, _, _| Ok(vm.string("")));
    m!("geteye", |vm, world, s, _| {
        // Zombies have eyes too (Town's burning zombies singe players whose
        // eyes come within 25 units of theirs): the origin plus a standing eye.
        let Ok(id) = client(vm, world, s) else {
            let o = origin_of(vm, world, s).ok_or("geteye: not an entity")?;
            return Ok(Value::Vec3([o[0], o[1], o[2] + 60.0]));
        };
        let f = frame(world);
        let (o, h) = f
            .player(id)
            .map_or(([0.0; 3], 60.0), |ps| (ps.origin, ps.view_height_current));
        Ok(Value::Vec3([o[0], o[1], o[2] + h]))
    });
    // The eye without the view's bob (Buried ghosts' sight checks): the same eye.
    m!("geteyeapprox", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let f = frame(world);
        let (o, h) = f
            .player(id)
            .map_or(([0.0; 3], 60.0), |ps| (ps.origin, ps.view_height_current));
        Ok(Value::Vec3([o[0], o[1], o[2] + h]))
    });
    // The view's place: the eye (no third-person or linked cameras here).
    m!("getplayercamerapos", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let f = frame(world);
        let (o, h) = f
            .player(id)
            .map_or(([0.0; 3], 60.0), |ps| (ps.origin, ps.view_height_current));
        Ok(Value::Vec3([o[0], o[1], o[2] + h]))
    });
    // The eye's height over the feet right now, a number (the Jet Gun's smoke
    // takes it off the muzzle: BO2 reads the player state's current height).
    m!("getplayerviewheight", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let h = frame(world).player(id).map_or(60.0, |ps| ps.view_height_current);
        Ok(Value::Float(h))
    });
    m!("getvelocity", |vm, world, s, _| {
        // An actor's is its last tick's move (the Sliquifier's slide on the
        // goo reads it).
        if let Some(a) = entnum(vm, s).and_then(|n| world.resource::<Zm>().actors.get(&n)) {
            return Ok(Value::Vec3(a.velocity));
        }
        if let Some(v) = entnum(vm, s).and_then(|n| super::vehicles::velocity(world, n)) {
            return Ok(Value::Vec3(v));
        }
        let id = client(vm, world, s)?;
        Ok(Value::Vec3(
            frame(world).player(id).map_or([0.0; 3], |ps| ps.velocity),
        ))
    });
    // Juggernog's 160 (190 upgraded). A zombie's call does nothing (its
    // health is a script field).
    m!("setmaxhealth", |vm, world, s, a| {
        let Ok(id) = client(vm, world, s) else {
            return Ok(Value::Undefined);
        };
        let n = int(a, 0)?.max(1);
        let mut f = frame(world);
        f.client_meta_mut(id).max_health = n;
        if let Some(ps) = f.player_mut(id) {
            ps.max_health = n;
            ps.health = ps.health.min(n);
        }
        Ok(Value::Undefined)
    });
    m!("setnormalhealth", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let frac = super::num(a, 0)?;
        if let Some(ps) = frame(world).player_mut(id) {
            ps.health = ((ps.max_health as f32 * frac) as i32).max(1);
        }
        Ok(Value::Undefined)
    });
    m!("getguid", |vm, _, _, _| Ok(vm.string("0")));
    m!("getxuid", |vm, _, _, _| Ok(vm.string("0")));
    // The first player in is the one running the game.
    m!("ishost", |vm, world, s, _| {
        Ok(Value::Int(i32::from(client(vm, world, s)? == crate::ClientId(0))))
    });
    m!("isplayeronsamemachine", |_, _, _, _| Ok(Value::Int(0)));
    // No ufo or noclip here. BO2 answers nothing (not 0) outside them:
    // TranZit's bus fog asks isdefined() of it before it poisons him.
    m!("isinmovemode", |_, _, _, _| Ok(Value::Undefined));
    m!("isinvehicle", |_, _, _, _| Ok(Value::Int(0)));
    m!("isremotecontrolling", |_, _, _, _| Ok(Value::Int(0)));
    m!("getsnapshotackindex", |_, _, _, _| Ok(Value::Int(0)));
    // How full a gun is (the scripts' grenade award gives 2, 3 or 4 by it).
    m!("getfractionstartammo", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let w = weapon(world, &text(vm, a, 0))?;
        Ok(Value::Float(script_player::ammo_fraction(
            &frame(world),
            id,
            w,
            true,
        )))
    });
    m!("getfractionmaxammo", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let name = text(vm, a, 0);
        let w = weapon(world, &name)?;
        let f = frame(world);
        let frac = script_player::ammo_fraction(&f, id, w, false);
        if std::env::var_os("IW4L_T6_HINTLOG").is_some() {
            let eq = f.equipment_facts_for(w).map(|e| (e.clip_size, e.start_ammo));
            diag::info!(
                Sim,
                "bo2zm t6 getfractionmaxammo {name}: {frac} (clip {} stock {}, clip size/start {eq:?})",
                script_player::ammo_clip(&f, id, w),
                script_player::ammo_stock(&f, id, w)
            );
        }
        Ok(Value::Float(frac))
    });
    for name in [
        "setclientuivisibilityflag",
        "setclientminiscoreboardhide",
        "setclientammocounterhide",
        "setclientscriptmainmenu",
        "setclientthirdperson",
        "setclientthirdpersonangle",
        "setclientcgobjectivetext",
        "setclientdvar",
        "setclientdvars",
        "openmenu",
        "closemenu",
        "closeingamemenu",
        "enableinvulnerability",
        "disableinvulnerability",
        "allowlean",
        "allowspectateteam",
        "allowpitchangle",
        "allowedstances",
        "setspectatepermissions",
        "playrumbleonentity",
        "stoprumble",
        "setlowready",
        "setsprintduration",
        "setsprintcooldown",
        "resetfov",
        "setburn",
        "addplayerstat",
        "addweaponstat",
        "incrementplayerstat",
        "addplayerstatwithgametype",
        "updatestatratio",
        "setplayercurrentstreak",
        "recordplayerdeathzombies",
        "recordplayerrevivezombies",
        "recordzombiezone",
        "recordkillmodifier",
        "setclientflag",
        "clearclientflag",
        "sendfaceevent",
        "setlaststandprevweap",
        "setspawnerteam",
        "vibrate",
        "setviewmodel",
        "enableafterlife",
        "disableafterlife",
        "dropscavengeritem",
        // Turned: back to human (zcleansed, _zm_turned); his zombie side
        // is read from `.is_zombie`.
        "turnedhuman",
        // _zm_stats / _globallogic: the match history the stats screen keeps.
        "gamehistorystartmatch",
        "gamehistoryfinishmatch",
        "setempjammed",
        "setfreecameralockonallowed",
        "luinotifyevent",
        "luinotifyeventtoplayer",
        // Any entity (buildable pieces): hide it from demo playback.
        "ghostindemo",
        // Blood fx on hits to this entity ("none": Buried's ghosts don't bleed).
        "bloodimpact",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    // bo2mp: BO2's own menus open on the player's screen (lane C); the
    // server only hears his answer (`menuresponse`). BO2MP_AUTOCLASS=<class>
    // answers the class menu a second after it opens (a test aid: a hidden
    // test match spawns without anyone at the keyboard).
    m!("openmenu", |vm, world, s, a| {
        let menu = text(vm, a, 0);
        let client = super::entnum(vm, s).unwrap_or(u32::MAX);
        diag::info!(Sim, "bo2mp menu open: player {client} {menu}");
        if let (Value::Object(o), Ok(class)) = (s, std::env::var("BO2MP_AUTOCLASS"))
            && menu.contains("changeclass")
        {
            let mut zm = world.resource_mut::<Zm>();
            let due = zm.now_ms + 1000;
            zm.menu_replies.push((due, *o, menu, class));
        }
        Ok(Value::Undefined)
    });
    // BO2's own end-of-match blur (`roundenddof`: 0, 128, 512, 4000, 6, 1.8):
    // the view effect the renderer's depth-of-field pass reads. The engine
    // takes it at once. Bad numbers are logged and ignored (not a script
    // error), so no BO2 script can be stopped by it.
    m!("setdepthoffield", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let n = |i: usize| num(a, i);
        let dof = match (n(0), n(1), n(2), n(3), n(4), n(5)) {
            (Ok(a0), Ok(a1), Ok(a2), Ok(a3), Ok(a4), Ok(a5)) => {
                crate::script::host::client_effects::depth_of_field_values(a0, a1, a2, a3, a4, a5)
            }
            _ => Err("six numbers expected".to_string()),
        };
        match dof {
            Ok(dof) => {
                let mut f = frame(world);
                if f.client_meta(id).is_some() {
                    f.client_meta_mut(id).view_effects.depth_of_field = dof;
                }
            }
            Err(e) if world.resource::<Zm>().mp => {
                diag::warn!(Sim, "bo2mp setdepthoffield ignored: {e}")
            }
            // bo2zm: on a Zombies map the numbers are brought in range
            // (near blur 4-10, far blur at most the near) and kept.
            Err(_) => {
                if let (Ok(a0), Ok(a1), Ok(a2), Ok(a3), Ok(a4), Ok(a5)) =
                    (n(0), n(1), n(2), n(3), n(4), n(5))
                {
                    let near_blur = a4.clamp(4.0, 10.0);
                    let far_blur = a5.clamp(0.0, near_blur);
                    if let Ok(dof) = crate::script::host::client_effects::depth_of_field_values(
                        a0.max(0.0),
                        a1.max(0.0),
                        a2.max(0.0),
                        a3.max(0.0),
                        near_blur,
                        far_blur,
                    ) {
                        let mut f = frame(world);
                        if f.client_meta(id).is_some() {
                            f.client_meta_mut(id).view_effects.depth_of_field = dof;
                        }
                    }
                }
            }
        }
        Ok(Value::Undefined)
    });
    m!("getdstat", |_, _, _, _| Ok(Value::Int(0)));
    // bo2zm M4: setactionslot(slot, kind[, weapon]): what his d-pad slot
    // holds (BO2's HUD draws it: `bo2zm_actionslots`); "" empties it.
    m!("setactionslot", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let slot = arg(a, 0).as_int().unwrap_or(0) as i32;
        let kind = text(vm, a, 1);
        let held = if kind == "weapon" {
            text(vm, a, 2)
        } else {
            kind
        };
        // His d-pad key (`+actionslot N`) switches to that weapon: the
        // engine's own slot (BO2's equipment, Time Bomb, claymores).
        let (ty, param) = match held.to_ascii_lowercase().as_str() {
            "" | "none" => (0, 0),
            "altmode" => (2, 0),
            "nightvision" => (3, 0),
            w => (1, super::weapon(world, w).unwrap_or(0) as i32),
        };
        if let Ok(i) = usize::try_from(slot - 1)
            && i < 4
            && let Some(ps) = frame(world).player_mut(id)
        {
            ps.action_slot_type[i] = if ty == 1 && param == 0 { 0 } else { ty };
            ps.action_slot_param[i] = param;
        }
        if std::env::var_os("IW4L_T6_WEAPLOG").is_some() {
            let has = frame(world)
                .player(id)
                .is_some_and(|ps| param != 0 && ps.weapons.contains(&param));
            diag::info!(Sim, "bo2zm t6 action slot {slot}: {held} (type {ty}, weapon {param}, held {has})");
        }
        let mut zm = world.resource_mut::<super::Zm>();
        let slots = zm.action_slots.entry(id.0).or_default();
        slots.retain(|(n, _)| *n != slot);
        if !held.is_empty() {
            slots.push((slot, held));
            slots.sort();
        }
        Ok(Value::Undefined)
    });
    m!("getnormalizedmovement", |_, _, _, _| Ok(Value::Vec3(
        [0.0; 3]
    )));
    m!("getnormalizedcameramovement", |_, _, _, _| Ok(Value::Vec3(
        [0.0; 3]
    )));
    m!("iprintln", |vm, _, _, a| {
        let t = text(vm, a, 0);
        diag::info!(Sim, "bo2zm t6 iprintln: {t}");
        Ok(Value::Undefined)
    });
    m!("iprintlnbold", |vm, _, _, a| {
        let t = text(vm, a, 0);
        diag::info!(Sim, "bo2zm t6 iprintlnbold: {t}");
        Ok(Value::Undefined)
    });
    // bo2zm M3: going down and the game's end.
    m!("playerads", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(Value::Float(
            frame(world)
                .player(id)
                .map_or(0.0, |ps| ps.f_weapon_pos_frac),
        ))
    });
    m!("getweaponmuzzlepoint", |vm, world, s, _| {
        // A zombie's killer may be no player (another zombie, the world):
        // where it stands.
        let Ok(id) = client(vm, world, s) else {
            return Ok(origin_of(vm, world, s).map_or(Value::Undefined, Value::Vec3));
        };
        let Some(ps) = frame(world).player(id).copied() else {
            return Ok(Value::Undefined);
        };
        let (f, _, _) = gsc_t6::math::angle_vectors(ps.viewangles);
        let eye = [
            ps.origin[0],
            ps.origin[1],
            ps.origin[2] + ps.view_height_current,
        ];
        Ok(Value::Vec3(std::array::from_fn(|i| eye[i] + f[i] * 16.0)))
    });
    m!("getweaponforwarddir", |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let angles = frame(world).player(id).map_or([0.0; 3], |ps| ps.viewangles);
        Ok(Value::Vec3(gsc_t6::math::angle_vectors(angles).0))
    });
    // Linked to an entity: the player goes where it goes (Nuketown's
    // game-over fall is a moving script_origin).
    m!("playerlinkto", |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let to = entnum(vm, arg(a, 0));
        if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
            p.linked = to;
        }
        Ok(Value::Undefined)
    });
    // He stays where playerlinkto put him relative to the entity: our link
    // keeps him on its origin either way (it takes no entity: binding it
    // with the others unlinked him).
    m!("playerlinkedoffsetenable", |_, _, _, _| Ok(Value::Undefined));
    for name in [
        "playerlinktodelta",
        "playerlinktoabsolute",
        "playerlinktoblend",
    ] {
        vm.bind(name, true, |vm, world, s, a| {
            let id = client(vm, world, s)?;
            let to = entnum(vm, arg(a, 0));
            if let Some(p) = world.resource_mut::<Zm>().players.get_mut(&id.0) {
                p.linked = to;
            }
            Ok(Value::Undefined)
        });
    }
    m!("getlinkedent", |vm, world, s, _| {
        let Some(n) = entnum(vm, s) else {
            return Ok(Value::Undefined);
        };
        let linked = {
            let zm = world.resource::<Zm>();
            zm.players
                .get(&n)
                .and_then(|p| p.linked)
                .or_else(|| zm.ents.get(&n).and_then(|e| e.link.map(|l| l.0)))
        };
        Ok(linked
            .and_then(|l| world.resource::<Zm>().ents.get(&l).and_then(|e| e.obj))
            .map_or(Value::Undefined, Value::Object))
    });
    for name in [
        "fakedamagefrom",
        "recordplayerdownzombies",
        "recordplayerrevivezombies",
        "recordplayerdeathzombies",
        "stoprumble",
        "playrumbleonentity",
        "playrumblelooponentity",
        "setclientuivisibilityflag",
        "setclientminiscoreboardhide",
        "setblur",
        "setburn",
        "setelectrified",
        "setlowready",
        "giveachievement",
    ] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Undefined));
    }
    screen_effects(vm);
    let _ = arg;
    let _ = list;
}

fn now_ms(world: &World) -> i32 {
    crate::level_time_ms(world.resource::<crate::step::StepRequest>().tick)
}

/// Blur (the game-over rocket blurs). Depth of field and shellshock (BO2's
/// perk machines knock players flat with `shellshock("default", 1.5)`) are
/// bound in `bind`.
fn screen_effects(vm: &mut Vm<World>) {
    vm.bind("setblur", true, |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let to = super::num(a, 0)?.max(0.0);
        let secs = super::num(a, 1).unwrap_or(0.0).max(0.0);
        let now = now_ms(world);
        let mut f = frame(world);
        if f.client_meta(id).is_some() {
            let view = &mut f.client_meta_mut(id).view_effects;
            view.blur = Some(crate::ScriptBlur {
                from: view.blur.map_or(0.0, |b| b.sample(now)),
                to,
                set_ms: now,
                duration_ms: (secs * 1000.0).round() as i32,
            });
        }
        Ok(Value::Undefined)
    });
    // `startfadingblur(blur, time)`: a blur that fades from `blur` to none.
    vm.bind("startfadingblur", true, |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let from = super::num(a, 0)?.max(0.0);
        let secs = super::num(a, 1).unwrap_or(0.0).max(0.0);
        let now = now_ms(world);
        let mut f = frame(world);
        if f.client_meta(id).is_some() {
            f.client_meta_mut(id).view_effects.blur = Some(crate::ScriptBlur {
                from,
                to: 0.0,
                set_ms: now,
                duration_ms: (secs * 1000.0).round() as i32,
            });
        }
        Ok(Value::Undefined)
    });
    // setdepthoffield, shellshock and stopshellshock: bound in `bind` above
    // (one implementation for both modes: the shock's index, its movement
    // scale and its end; a Zombies map's out-of-range depth of field kept
    // in range).
}

fn weapon_list(
    vm: &mut Vm<World>,
    world: &mut World,
    s: &Value,
    which: script_player::WeaponList,
) -> R {
    let id = client(vm, world, s)?;
    let all = matches!(which, script_player::WeaponList::All);
    let mut ws = script_player::weapons(&frame(world), id, which);
    // His melee weapon is held too (BO2 lists it with the rest).
    let knife = held_melee(world, id);
    if all && knife != 0 && !ws.contains(&knife) {
        ws.push(knife);
    }
    let mut out = Vec::new();
    for w in ws {
        let n = weapon_text(world, w);
        out.push(vm.string(&n));
    }
    Ok(list(out))
}

/// The melee weapon BO2's engine holds for him (0: none).
fn held_melee(world: &mut World, id: crate::ClientId) -> u32 {
    let name = world
        .resource::<Zm>()
        .players
        .get(&id.0)
        .and_then(|p| p.melee_weapon.clone());
    name.and_then(|n| weapon(world, &n).ok()).unwrap_or(0)
}

/// bo2mp: the camo he has on a gun, for every player's draw: his script
/// value `bo2mp_camos` (`<weapon>:<option>` pairs, comma separated; read by
/// `asset_core::weapon_camos`). Option 0 takes the gun's camo off.
fn wear_camo(world: &mut World, id: ClientId, weapon: u32, option: i32) {
    const KEY: &str = "bo2mp_camos";
    let mut f = frame(world);
    let dvars = &mut f.client_meta_mut(id).client_dvars;
    let mut pairs: Vec<(u32, i32)> = dvars
        .iter()
        .find(|(k, _)| k == KEY)
        .map(|(_, v)| {
            v.split(',')
                .filter_map(|p| {
                    let (w, o) = p.split_once(':')?;
                    Some((w.parse().ok()?, o.parse().ok()?))
                })
                .collect()
        })
        .unwrap_or_default();
    pairs.retain(|(w, _)| *w != weapon);
    if option > 0 {
        pairs.push((weapon, option));
    }
    let text = pairs.iter().map(|(w, o)| format!("{w}:{o}")).collect::<Vec<_>>().join(",");
    match dvars.iter_mut().find(|(k, _)| k == KEY) {
        Some(row) => row.1 = text,
        None if !text.is_empty() => dvars.push((KEY.to_owned(), text)),
        None => {}
    }
}
