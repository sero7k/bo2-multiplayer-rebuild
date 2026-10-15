//! bo2mp: `BO2MP_INPUT_REPLAY=<file>`: plays a timed input script as player
//! 1's own commands, command by command (each ~16 ms command gets the
//! buttons and weapon the script holds at its time), so trick inputs
//! measured from footage of the original game run through the real match.
//! With `BO2MP_WEAPON_TRACE` set the host logs his weapon state for each
//! command (`combat::weapon_trace`).
//!
//! Script: one `<ms> <action>` per line, ms from the script's start, which
//! is `BO2MP_INPUT_REPLAY_AT` ms of match time (default 12000); `#` starts
//! a comment. Actions:
//! - `give <weapon> <weapon>`: take all his weapons, give these (full
//!   ammo) and select the first
//! - `fire+`, `fire-`, `reload+`, `reload-`: press / let go
//! - `swap`: select the next weapon given
//! - `alt`: the Select Fire toggle (the gun up and its alternate)
//! - `log <text>`: a marker in the log

use std::sync::Mutex;

use bevy_ecs::prelude::World;
use playerstate_iw4::buttons;

use super::frame;
use crate::world::ClientId;

struct Replay {
    lines: Vec<(i64, String)>,
    next: usize,
    base_server_time: Option<i32>,
    held: u32,
    guns: Vec<u32>,
    /// Which of `guns` is up (`selected` may be its alternate).
    cur: usize,
    selected: u32,
}

static REPLAY: Mutex<Option<Replay>> = Mutex::new(None);

fn load() -> Option<Replay> {
    let path = std::env::var("BO2MP_INPUT_REPLAY").ok()?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| diag::warn!(Sim, "input replay: {path}: {e}"))
        .ok()?;
    let mut lines = Vec::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let Some((at, action)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        if let Ok(at) = at.parse::<i64>() {
            lines.push((at, action.trim().to_owned()));
        }
    }
    lines.sort_by_key(|l| l.0);
    diag::info!(Sim, "input replay: {} steps from {path}", lines.len());
    Some(Replay {
        lines,
        next: 0,
        base_server_time: None,
        held: 0,
        guns: Vec::new(),
        cur: 0,
        selected: 0,
    })
}

fn start_ms() -> i64 {
    std::env::var("BO2MP_INPUT_REPLAY_AT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12000)
}

pub(crate) fn drive(world: &mut World, now: i64) {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ENABLED.get_or_init(|| std::env::var_os("BO2MP_INPUT_REPLAY").is_some()) || now < start_ms() {
        return;
    }
    let Ok(mut slot) = REPLAY.lock() else {
        return;
    };
    if slot.is_none() {
        *slot = load();
    }
    let Some(r) = slot.as_mut() else {
        return;
    };
    let client = ClientId(0);
    let cmds: Vec<usize> = {
        let req = world.resource::<crate::step::StepRequest>();
        req.input
            .cmds
            .iter()
            .enumerate()
            .filter(|(_, (id, _))| *id == client)
            .map(|(i, _)| i)
            .collect()
    };
    for i in cmds {
        let server_time = world.resource::<crate::step::StepRequest>().input.cmds[i].1.server_time;
        let base = *r.base_server_time.get_or_insert(server_time);
        let t = i64::from(server_time.wrapping_sub(base));
        while r.next < r.lines.len() && r.lines[r.next].0 <= t {
            let (at, action) = r.lines[r.next].clone();
            r.next += 1;
            apply(world, r, client, at, &action);
        }
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        let cmd = &mut req.input.cmds[i].1;
        cmd.buttons = (cmd.buttons & !(buttons::ATTACK | buttons::RELOAD)) | r.held;
        if r.selected != 0 {
            cmd.weapon = r.selected as u16;
            cmd.weapon_mapped = r.guns.get(r.cur).copied().unwrap_or(r.selected) as u16;
        }
    }
}

fn apply(world: &mut World, r: &mut Replay, client: ClientId, at: i64, action: &str) {
    let mut words = action.split_whitespace();
    let verb = words.next().unwrap_or("");
    match verb {
        "give" => {
            let names: Vec<&str> = words.collect();
            let mut f = frame(world);
            crate::script_player::take_all_weapons(&mut f, client);
            r.guns.clear();
            for name in names {
                match crate::script_player::weapon_named(&f, name) {
                    Ok(w) => {
                        if let Err(e) = crate::script_player::give_weapon(&mut f, client, w, false) {
                            diag::warn!(Sim, "input replay: give {name}: {e}");
                            continue;
                        }
                        crate::script_player::give_max_ammo(&mut f, client, w);
                        r.guns.push(w);
                    }
                    Err(e) => diag::warn!(Sim, "input replay: {name}: {e}"),
                }
            }
            r.cur = 0;
            if let Some(&first) = r.guns.first() {
                crate::script_player::switch_to_weapon(&mut f, client, first);
                r.selected = first;
            }
        }
        "fire+" => r.held |= buttons::ATTACK,
        "fire-" => r.held &= !buttons::ATTACK,
        "reload+" => r.held |= buttons::RELOAD,
        "reload-" => r.held &= !buttons::RELOAD,
        "swap" => {
            if !r.guns.is_empty() {
                r.cur = (r.cur + 1) % r.guns.len();
                r.selected = r.guns[r.cur];
            }
        }
        // The Select Fire toggle: the gun up and its alternate, back and forth.
        "alt" => {
            let parent = r.guns.get(r.cur).copied().unwrap_or(0);
            let alt = frame(world).combat_facts_for(parent).map_or(0, |f| f.alternate_weapon);
            if alt != 0 {
                r.selected = if r.selected == alt { parent } else { alt };
            }
        }
        "log" => {}
        _ => diag::warn!(Sim, "input replay: unknown action {action:?}"),
    }
    diag::info!(Sim, "input replay at {at} ms: {action}");
}
