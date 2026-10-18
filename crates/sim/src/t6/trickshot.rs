//! bo2mp: his own Plutonium trickshot script (`bo2mp/trickshot`, source in
//! `scripts/trickshot/main.gsc`, kept as he wrote it), loaded as Plutonium
//! loads a custom script: its `init()` once the game type has started, the
//! connect and spawn threads it starts from there. Optional (Custom Games'
//! TRICKSHOT MODE, or `BO2MP_TRICKSHOT=1`); never in a public match.
//!
//! The engine side it needs that Black Ops II's own scripts never called:
//! Plutonium's `notifyonplayercommand` (a client command, typed in the
//! console or a button's `+command`, notifies the player), `printf`, and
//! a few engine builtins our multiplayer had not bound yet (`suicide`,
//! `kick`, `closer`, `getangles`, `clearalltextafterhudelem`).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy_ecs::prelude::{Resource, World};
use gsc_t6::{Value, Vm};

use super::{Zm, arg, client, entnum, frame, is_player, text};
use crate::world::ClientId;

/// His script, as he wrote it.
pub(super) const SCRIPT: &str = include_str!("scripts/trickshot/main.gsc");

/// The script's name in the linked program.
pub(super) const NAME: &str = "bo2mp/trickshot";

static ON: AtomicBool = AtomicBool::new(false);

/// bo2mp: the lobby's TRICKSHOT MODE for the next match (Custom Games
/// only: a public match never sets it).
pub fn set_trickshot(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// Whether this match runs his script: the lobby's choice, or
/// `BO2MP_TRICKSHOT=0/1`.
pub(super) fn enabled() -> bool {
    match std::env::var("BO2MP_TRICKSHOT") {
        Ok(v) => v.trim() == "1",
        Err(_) => ON.load(Ordering::Relaxed),
    }
}

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether the running match links his script (set when it is compiled).
pub(super) fn set_active(on: bool) {
    ACTIVE.store(on, Ordering::Relaxed);
}

pub(super) fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// Each player's `notifyonplayercommand` registrations, and the buttons
/// he held last tick (a `+command` fires on the press, `-command` on the
/// release).
#[derive(Resource, Default)]
pub(super) struct PlayerCommands {
    binds: HashMap<u32, Vec<(String, String)>>,
    held: HashMap<u32, u32>,
}

/// Plutonium's button command names, as our commands carry them.
/// A button's bits by its command name (`melee`, `gostand`, `prone`).
pub(super) fn button_bits(name: &str) -> Option<u32> {
    button_commands().iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, b)| *b)
}

fn button_commands() -> &'static [(&'static str, u32)] {
    use playerstate_iw4::buttons::*;
    &[
        ("attack", ATTACK),
        ("speed_throw", ADS),
        ("toggleads_throw", ADS),
        ("ads", ADS),
        ("melee", MELEE_CHARGE),
        ("melee_breath", MELEE_CHARGE),
        ("frag", FRAG),
        ("smoke", SMOKE),
        ("gostand", JUMP),
        ("usereload", USE | USE_RELOAD),
        ("activate", USE),
        ("reload", RELOAD | USE_RELOAD),
        ("breath_sprint", SPRINT),
        ("sprint", SPRINT),
        ("stance", STANCE_HELD),
        ("movedown", CROUCH),
        ("prone", PRONE),
        ("throw", THROW),
        ("holdbreath", BREATH),
    ]
}

fn normalize(cmd: &str) -> String {
    cmd.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_start_matches('/')
        .trim_start_matches('\\')
        .to_ascii_lowercase()
}

/// He sent a client command (typed in the console, `cmd <text>`, or a
/// button's `+command`): the notifies his scripts asked for. False when
/// nothing listens for it.
pub(crate) fn player_command(world: &mut World, client: u32, line: &str) -> bool {
    let line = normalize(line);
    let first = line.split(' ').next().unwrap_or("").to_owned();
    let notifies: Vec<String> = world
        .get_resource::<PlayerCommands>()
        .and_then(|c| c.binds.get(&client))
        .map(|b| {
            b.iter()
                .filter(|(cmd, _)| *cmd == line || *cmd == first)
                .map(|(_, n)| n.clone())
                .collect()
        })
        .unwrap_or_default();
    if notifies.is_empty() {
        return false;
    }
    super::with_vm(world, |vm, world| {
        let Some(p) = world.resource::<Zm>().players.get(&client).map(|p| p.obj) else {
            return;
        };
        for n in &notifies {
            vm.notify_str(world, p, n, &[]);
        }
    });
    true
}

/// A d-pad slot's key (0-based slot): `+actionslot <n>`.
pub(super) fn action_slot(world: &mut World, client: u32, slot: u8) {
    if world.contains_resource::<PlayerCommands>() {
        player_command(world, client, &format!("+actionslot {}", u32::from(slot) + 1));
    }
}

/// Each tick: the button `+commands` pressed and released since the last.
pub(super) fn buttons(world: &mut World) {
    let Some(cmds) = world.get_resource::<PlayerCommands>() else {
        return;
    };
    let clients: Vec<u32> = cmds.binds.keys().copied().collect();
    for c in clients {
        let now = crate::script_player::buttons(&mut frame(world), ClientId(c));
        let was = world
            .resource_mut::<PlayerCommands>()
            .held
            .insert(c, now)
            .unwrap_or(0);
        if now == was {
            continue;
        }
        for &(name, mask) in button_commands() {
            let (down, before) = (now & mask != 0, was & mask != 0);
            if down != before {
                let line = format!("{}{name}", if down { '+' } else { '-' });
                player_command(world, c, &line);
            }
        }
    }
}

/// Plutonium-style XUID (hex text, one per client).
fn xuid(n: u32) -> String {
    format!("{:x}", 0x0110_0001_0000_0000_u64 + u64::from(n))
}

pub(super) fn bind(vm: &mut Vm<World>) {
    // self notifyonplayercommand( <notify>, <command> )
    vm.bind("notifyonplayercommand", true, |vm, world, s, a| {
        let id = client(vm, world, s)?;
        let notify = text(vm, a, 0);
        let cmd = normalize(&text(vm, a, 1));
        if notify.is_empty() || cmd.is_empty() {
            return Ok(Value::Undefined);
        }
        if !world.contains_resource::<PlayerCommands>() {
            world.init_resource::<PlayerCommands>();
        }
        let mut c = world.resource_mut::<PlayerCommands>();
        let list = c.binds.entry(id.0).or_default();
        if !list.iter().any(|(k, n)| *k == cmd && *n == notify) {
            list.push((cmd, notify));
        }
        Ok(Value::Undefined)
    });
    // Plutonium's printf( <text>, ... ): to the log.
    vm.bind("printf", false, |vm, _, _, a| {
        let line: Vec<String> = a.iter().map(|v| vm.to_text(v)).collect();
        diag::info!(Sim, "gsc printf: {}", line.join(" "));
        Ok(Value::Undefined)
    });
    vm.bind("getxuid", true, |vm, world, s, _| {
        let id = client(vm, world, s)?;
        Ok(vm.string(&xuid(id.0)))
    });
    // self suicide(): he dies by his own hand (MOD_SUICIDE), BO2's
    // CodeCallback_PlayerKilled as for any death.
    vm.bind("suicide", true, |vm, world, s, _| {
        let id = client(vm, world, s)?;
        let alive = frame(world)
            .client_meta(id)
            .is_some_and(|m| m.lifecycle == crate::match_state::ClientLifecycle::Alive);
        if !alive {
            return Ok(Value::Undefined);
        }
        let tick = super::tick(world);
        super::playeranim::death(world, id, "MOD_SUICIDE", None, "none");
        crate::script_player::kill(&mut frame(world), tick, id, Some(id), None);
        let Some(obj) = s.as_obj() else {
            return Ok(Value::Undefined);
        };
        let none = vm.string("none");
        vm.notify_str(world, obj, "death", &[s.clone(), Value::Int(0), none.clone()]);
        let health = frame(world).player(id).map_or(100, |ps| ps.health.max(1));
        let means = vm.string("MOD_SUICIDE");
        let args = vec![
            s.clone(),
            s.clone(),
            Value::Int(health),
            means,
            none.clone(),
            Value::Vec3([0.0; 3]),
            none,
            Value::Int(0),
            Value::Int(0),
        ];
        let cb = super::mp::callbacks(world);
        vm.spawn_named(world, &cb, "codecallback_playerkilled", Value::Object(obj), args);
        Ok(Value::Undefined)
    });
    // kick( <entity number> ): a bot leaves; a player stays (a local match).
    vm.bind("kick", false, |vm, world, _, a| {
        let n = match arg(a, 0) {
            Value::Int(n) => *n as u32,
            v => entnum(vm, v).unwrap_or(u32::MAX),
        };
        if super::is_bot(world, n) {
            world.resource_mut::<Zm>().bot_leaves.push(n);
        } else {
            diag::info!(Sim, "gsc kick({n}): not a bot, kept");
        }
        Ok(Value::Undefined)
    });
    // closer( <ref>, <a>, <b> ): a nearer ref than b.
    vm.bind("closer", false, |_, _, _, a| {
        let p = |i: usize| arg(a, i).as_vec3().unwrap_or([0.0; 3]);
        let d = |x: [f32; 3], y: [f32; 3]| {
            (x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2) + (x[2] - y[2]).powi(2)
        };
        let (r, x, y) = (p(0), p(1), p(2));
        Ok(Value::bool(d(r, x) < d(r, y)))
    });
    // self getangles(): a player's view, an entity's angles.
    vm.bind("getangles", true, |vm, world, s, _| {
        if is_player(vm, world, s) {
            let id = client(vm, world, s)?;
            return Ok(Value::Vec3(frame(world).player(id).map_or([0.0; 3], |ps| ps.viewangles)));
        }
        let Some(o) = s.as_obj() else {
            return Ok(Value::Vec3([0.0; 3]));
        };
        let f = vm.intern("angles");
        Ok(vm.get_field(world, o, f))
    });
    // The config-string overflow fix's reset: our HUD text has no string
    // table to run out of.
    vm.bind("clearalltextafterhudelem", true, |_, _, _, _| Ok(Value::Undefined));
    // Shim (his script's `giveweapon( w, 0, true( camo, 0, 0, 0, 0 ) )`,
    // where BO2 has calcweaponoptions): the default weapon options.
    vm.bind("true", false, |_, _, _, _| Ok(Value::Int(0)));
}
