//! Actors (zombies): spawned from spawners (`spawnactor`), driven by the
//! game's own animscripts the way the engine drives them, measured from
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
//! the scripts (see docs/):
=======
//! the scripts (notes kept outside the repo):
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
//! - one animscript runs at a time: `killanimscript`, then the new state's
//!   `maps/mp/animscripts/zm_<state>::main` (init on spawn; stop, move,
//!   combat, death, scripted, traverse/<node animscript>);
//! - animations come from the actor's animstatedef: `setanimstatefromasd`
//!   picks a substate; notetracks go out as `notify(<state's notify>,
//!   note)`, then `end` (every cycle of a looping state);
//! - movement: a path over the map's own nodes to `setgoalpos`, at the move
//!   animation's root-motion speed; `goal` inside goalradius, `bad_path`
//!   when nothing joins; scripted animations (`animscripted`) move by their
//!   own root motion from where they were started.

use std::collections::BTreeMap;

use bevy_ecs::prelude::World;
use gsc_t6::{ObjKind, ObjRef, Str, Value, Vm};

use super::nav::NODE_NEGOTIATION_BEGIN;
use super::{Ent, T6Anim, Zm, frame, with_vm};
use crate::world::ClientId;

/// A playing animation.
#[derive(Clone, Debug, Default)]
pub(crate) struct Playing {
    pub state: String,
    pub substate: usize,
    pub anim: String,
    pub notify: String,
    pub start_ms: i64,
    pub length_ms: i64,
    pub looping: bool,
    /// Notetracks already sent this cycle (by index), and cycles finished.
    pub sent: usize,
    pub cycle: i64,
    /// Root position at the start (scripted anims move by root motion).
    pub root_from: [f32; 3],
    pub yaw_from: f32,
    /// A linked actor's parent (or tag) pose when it began: the clip plays
    /// in that frame, carried by the parent's move since (BO2: an
    /// animscripted zombie on the bus's window tag rides the moving bus).
    pub parent_from: Option<([f32; 3], [f32; 3])>,
    pub ended: bool,
    /// `setentityanimrate`: the clip plays this fast (1 is normal; the
    /// Paralyzer and the Time Bomb slow it), and the clip time it had
    /// played when the rate last changed (ms, at `start_ms`).
    pub rate: f32,
    pub base_ms: i64,
}

impl Playing {
    /// The clip time played by `now` (ms), at its rate.
    pub(crate) fn elapsed(&self, now: i64) -> i64 {
        (self.base_ms + ((now - self.start_ms).max(0) as f32 * self.rate) as i64).max(0)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Scripted {
    pub origin: [f32; 3],
    pub angles: [f32; 3],
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Traverse {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub start_ms: i64,
    pub length_ms: i64,
    pub end_node: u32,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Actor {
    pub obj: Option<ObjRef>,
    pub aitype: String,
    pub asd: String,
    pub alive: bool,
    pub health: i32,
    pub maxhealth: i32,
    pub team: String,
    pub playing: Option<Playing>,
    pub animmode: String,
    pub orientmode: String,
    pub orient_yaw: f32,
    pub orient_point: [f32; 3],
    /// `setpitchorient`: the body tilts to the ground's slope along its
    /// facing (Nuketown's crater crawlers climbing out).
    pub pitch_orient: bool,
    /// The facing `face` turns toward this tick (`None` when nothing turns
    /// it): his client turns the body toward it at the same rate and stops
    /// there (`presence::sync`).
    pub yaw_goal: Option<f32>,
    pub script: String,
    pub scripted: Option<Scripted>,
    /// Counts `animscripted`/`startscriptedanim` calls: a scripted clip's
    /// `end` that starts the next scripted clip keeps the actor scripted.
    pub scripted_starts: u32,
    /// `setentityanimrate` other than 1 (the Paralyzer's slow-down): its
    /// clips and the walking they drive play this fast.
    pub anim_rate: Option<f32>,
    /// `animcustom`: the script function playing its animations; its AI
    /// waits until that thread ends (Die Rise's Jumping Jack wall leaps).
    pub custom: Option<gsc_t6::ThreadId>,
    pub traverse: Option<Traverse>,
    pub goal: Option<[f32; 3]>,
    pub goal_sent: bool,
    /// `setgoalpos`'s angles: the yaw it turns to once there, then
    /// "orientdone" (a window tear waits for it, up to a second, before its
    /// clip starts at the window's angles).
    pub goal_yaw: Option<f32>,
    /// The node a goal routes to, and the goal it was picked for.
    pub goal_node: Option<(u32, [f32; 3])>,
    pub path: Vec<[f32; 3]>,
    pub path_nodes: Vec<u32>,
    pub path_i: usize,
    pub enemy: Option<ObjRef>,
    /// bo2mp: who it belongs to (`setentityowner`: a K9 Unit dog's caller,
    /// never its enemy).
    pub owner: Option<u32>,
    pub radius: f32,
    pub height: f32,
    pub dead_since: Option<i64>,
    pub character_index: i32,
    pub missing_legs: bool,
    /// Where it stood when it last got anywhere, and when (a body that
    /// stays put while it should move plans its way again).
    pub progress: Option<([f32; 3], i64)>,
    /// Path nodes it stalled on, kept out of its plans until then (ms).
    pub avoid: Vec<(u32, i64)>,
    /// The path node it last stalled short of.
    pub stalled_on: Option<u32>,
    /// A straight walk to the goal stalled: plan over the path nodes until
    /// then (the scripts set the goal again several times a second, and
    /// each straight plan walked it back into what stopped it).
    pub no_straight_until: i64,
    /// The goal its last plan found no way to: `setgoalpos` to it again
    /// says nothing new (no second `bad_path`). Buried's breadcrumb loop
    /// sets an unreachable fallback goal again on every `bad_path`, each try
    /// adding 100 ms to the ghost's path timer: one `bad_path` a frame grew
    /// the timer faster than the clock and the ghost never chose again.
    pub failed_goal: Option<[f32; 3]>,
    /// It has stood clear of solid since it spawned: from then on no step
    /// takes it into solid (monster clip keeps zombies out of places; only a
    /// zombie spawned inside scenery may walk out of it).
    pub clear: bool,
    /// How fast it moved over the last tick (`getvelocity`).
    pub velocity: [f32; 3],
}

/// An actor's goal also holds it only this close in height (the engine's
/// `goalheight`): one standing in the teal house's raised ground floor right
/// above him counted as there and stood idle (fix list 3).
const GOAL_HEIGHT: f32 = 80.0;

/// A path point this close (x, y) counts as passed: a crowd going the same
/// way keeps every body but one off the point itself (fix list 3).
const PASS_REACH: f32 = 32.0;

/// A body that has not got this far in `STALL_MS` while moving plans again
/// from where it stands.
const STALL_DIST: f32 = 16.0;
const STALL_MS: i64 = 3000;

pub(crate) const ANIMSCRIPTS: &str = "maps/mp/animscripts";

fn dist2(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1]];
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    gsc_t6::math::length(gsc_t6::math::sub(a, b))
}

/// The root-motion distance an animation covers per second.
pub(crate) fn anim_speed(anim: &T6Anim) -> f32 {
    let len = if anim.framerate > 0.0 {
        f32::from(anim.numframes) / anim.framerate
    } else {
        0.0
    };
    if len <= 0.0 {
        return 0.0;
    }
    let first = anim.delta_trans.first().map_or([0.0; 3], |d| d.1);
    let last = anim.delta_trans.last().map_or([0.0; 3], |d| d.1);
    dist2(first, last) / len
}

/// Root translation at fraction `f` of the animation (keys by frame).
pub(crate) fn anim_root(anim: &T6Anim, f: f32) -> [f32; 3] {
    let keys = &anim.delta_trans;
    if keys.is_empty() {
        return [0.0; 3];
    }
    let frame = f.clamp(0.0, 1.0) * f32::from(anim.numframes.max(1));
    let mut prev = keys[0];
    for &k in keys {
        if f32::from(k.0) >= frame {
            let span = f32::from(k.0.saturating_sub(prev.0)).max(1e-3);
            let t = ((frame - f32::from(prev.0)) / span).clamp(0.0, 1.0);
            return std::array::from_fn(|i| prev.1[i] + (k.1[i] - prev.1[i]) * t);
        }
        prev = k;
    }
    prev.1
}

pub(crate) fn anim_length_ms(anim: &T6Anim) -> i64 {
    if anim.framerate > 0.0 {
        ((f32::from(anim.numframes) / anim.framerate) * 1000.0).round() as i64
    } else {
        1000
    }
}

/// Start `state` (an ASD state) on actor `n`; `sub` = substate index or
/// alias (none = random). Returns the substate picked.
pub(crate) fn set_state(
    vm: &mut Vm<World>,
    world: &mut World,
    n: u32,
    state: &str,
    sub: &Value,
) -> Option<usize> {
    let now = world.resource::<Zm>().now_ms;
    let (asd, missing_legs) = {
        let zm = world.resource::<Zm>();
        let a = zm.actors.get(&n)?;
        (a.asd.clone(), a.missing_legs)
    };
    let found = {
        let zm = world.resource::<Zm>();
        zm.asds.get(&asd).is_some_and(|def| {
            def.state(state, false)
                .or_else(|| def.state(state, missing_legs))
                .is_some_and(|st| !st.substates.is_empty())
        })
    };
    if !found {
        vm.report_once(format!("bo2zm t6: animstatedef {asd} has no state {state}"));
        return None;
    }
    let picked = {
        let zm = world.resource::<Zm>();
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
        // bo2mp: a K9 Unit dog has no animstatedef; BO2's exe plays its
        // states itself (`dog_state_anim`), each notifying "done".
        let Some(def) = zm.asds.get(&asd) else {
            let dog = zm.actors.get(&n).is_some_and(is_mp_dog);
            let anim = dog.then(|| dog_state_anim(state)).flatten()?;
            let looping = zm.anims.get(anim).is_some_and(|x| x.looping);
            return Some(start_state(world, n, state, 0, anim, "done", !looping, sub));
        };
=======
        let def = zm.asds.get(&asd)?;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
        let st = def
            .state(state, false)
            .or_else(|| def.state(state, missing_legs))?;
        let i = match sub {
            Value::Int(i) => (*i).clamp(0, st.substates.len() as i32 - 1) as usize,
            Value::Float(f) => (*f as i32).clamp(0, st.substates.len() as i32 - 1) as usize,
            Value::Str(s) => {
                let alias = vm.str(*s).to_ascii_lowercase();
                st.substates
                    .iter()
                    .position(|(a, _)| *a == alias)
                    .unwrap_or(0)
            }
            _ => (vm.rand_u32() as usize) % st.substates.len(),
        };
        (
            st.name.clone(),
            i,
            st.substates[i].1.to_ascii_lowercase(),
            st.notify.clone(),
            st.restart,
        )
    };
    let (state_name, i, anim, notify, restart) = picked;
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    Some(start_state(world, n, &state_name, i, &anim, &notify, restart, sub))
}

/// Start (or keep) state `state_name`'s substate `i` playing `anim`.
#[allow(clippy::too_many_arguments)]
fn start_state(
    world: &mut World,
    n: u32,
    state_name: &str,
    i: usize,
    anim: &str,
    notify: &str,
    restart: bool,
    sub: &Value,
) -> usize {
    let now = world.resource::<Zm>().now_ms;
    let (state_name, anim, notify) = (state_name.to_owned(), anim.to_owned(), notify.to_owned());
=======
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
    let (length_ms, looping) = {
        let zm = world.resource::<Zm>();
        match zm.anims.get(&anim) {
            Some(a) => (anim_length_ms(a), a.looping),
            None => (1000, false),
        }
    };
    let (origin, yaw) = {
        let zm = world.resource::<Zm>();
        zm.ents
            .get(&n)
            .map_or(([0.0; 3], 0.0), |e| (e.origin, e.angles[1]))
    };
    let parent_from = super::movers::link_parent_pose(world, n);
    let mut zm = world.resource_mut::<Zm>();
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    let Some(a) = zm.actors.get_mut(&n) else {
        return i;
    };
=======
    let a = zm.actors.get_mut(&n)?;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
    // A looping state already playing goes on unless asked to restart.
    if let Some(p) = &a.playing
        && p.state == state_name
        && p.substate == i
        && !restart
        && matches!(sub, Value::Undefined)
    {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
        return i;
=======
        return Some(i);
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
    }
    // IW4L_T6_ANIMLOG=1: every animation start (test aid): actor, state,
    // clip, and whether it restarts the clip already playing.
    if std::env::var_os("IW4L_T6_ANIMLOG").is_some() {
        let same = a.playing.as_ref().is_some_and(|p| p.anim == anim);
        let since = a.playing.as_ref().map_or(-1, |p| now - p.start_ms);
        diag::info!(
            Sim,
            "bo2zm t6 anim start {n} t{now}: {state_name}/{i} {anim} script {} restart {same} after {since} ms at ({:.0} {:.0} {:.0})",
            a.script,
            origin[0],
            origin[1],
            origin[2]
        );
    }
    a.playing = Some(Playing {
        state: state_name,
        substate: i,
        anim,
        notify,
        start_ms: now,
        length_ms: length_ms.max(1),
        looping,
        sent: 0,
        cycle: 0,
        root_from: origin,
        yaw_from: yaw,
        parent_from,
        ended: false,
        rate: a.anim_rate.unwrap_or(1.0),
        base_ms: 0,
    });
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    i
}

/// bo2mp: a K9 Unit dog (`enemy_dog_mp`): BO2's exe runs it with its own
/// dog animscripts (`dog_*`) and animation states.
pub(crate) fn is_mp_dog(a: &Actor) -> bool {
    a.aitype.ends_with("dog_mp")
}

/// The animation states BO2's dog scripts ask for, played with his
/// `german_shepherd` animations (named for them; there is no walk).
fn dog_state_anim(state: &str) -> Option<&'static str> {
    Some(match state {
        "move_run" | "move_walk" => "german_shepherd_run",
        "move_start" => "german_shepherd_run_start",
        "move_stop" => "german_shepherd_run_stop",
        "stop_idle" => "german_shepherd_idle",
        "stop_attackidle" | "combat_attackidle" => "german_shepherd_attackidle",
        "stop_attackidle_bark" | "combat_attackidle_bark" => "german_shepherd_attackidle_bark",
        "stop_attackidle_growl" | "combat_attackidle_growl" => "german_shepherd_attackidle_growl",
        "combat_attack_run" => "german_shepherd_run_attack",
        "combat_attack_player_close_range" => "german_shepard_run_attack_low",
        "flashed" => "german_shepherd_run_flashbang",
        "traverse_wallhop" => "german_shepherd_run_jump_window_40",
        "move_turn_left" => "german_shepard_turn_90_left",
        "move_turn_right" => "german_shepard_turn_90_right",
        "move_run_turn_left" => "german_shepard_run_turn_90_left",
        "move_run_turn_right" => "german_shepard_run_turn_90_right",
        "move_turn_around_left" => "german_shepherd_run_start_180_l",
        "move_turn_around_right" => "german_shepherd_run_start_180_r",
        "move_run_turn_around_left" => "german_shepard_run_turn_180_left",
        "move_run_turn_around_right" => "german_shepard_run_turn_180_right",
        "pain_front" => "german_shepard_pain_hit_front",
        "pain_back" => "german_shepard_pain_hit_back",
        "pain_left" => "german_shepard_pain_hit_left",
        "pain_right" => "german_shepard_pain_hit_right",
        "pain_run_front" => "german_shepard_run_pain_hit_front",
        "pain_run_back" | "pain_run_left" | "pain_run_right" => "german_shepherd_run_pain",
        "death_front" => "german_shepherd_death_front",
        "death_back" => "german_shepard_death_hit_back",
        "death_left" => "german_shepard_death_hit_left",
        "death_right" => "german_shepard_death_hit_right",
        _ => return None,
    })
}

/// How close a dog leaps from: as far as its run attack carries it before
/// the bite (`dog_melee`), plus a bite's reach.
fn dog_attack_range(world: &World) -> f32 {
    let zm = world.resource::<Zm>();
    let lunge = zm.anims.get("german_shepherd_run_attack").map_or(0.0, |a| {
        let at = a
            .notifies
            .iter()
            .find(|(name, _)| name == "dog_melee")
            .map_or(0.2, |(_, f)| *f);
        let r = anim_root(a, at);
        (r[0] * r[0] + r[1] * r[1]).sqrt()
    });
    lunge + 64.0
}

/// A dog's enemy, as BO2's exe picks one: the nearest living player it
/// can see who is not its owner nor (in a team game) on its side; the one
/// it has while he stays in sight.
fn dog_enemy(
    vm: &mut Vm<World>,
    world: &mut World,
    a: &Actor,
    obj: ObjRef,
    me: [f32; 3],
) -> Option<ObjRef> {
    if gsc_t6::truthy(&field(vm, obj, "ignoreall")) {
        return None;
    }
    let aiteam = field(vm, obj, "aiteam");
    let aiteam = vm.to_text(&aiteam);
    let tb = vm.intern("teambased");
    let teambased = gsc_t6::truthy(&vm.raw_field(vm.level, tb));
    let team_s = vm.intern("team");
    let ignore_s = vm.intern("ignoreme");
    let players: Vec<(u32, ObjRef)> = world
        .resource::<Zm>()
        .players
        .iter()
        .filter(|(_, p)| p.sessionstate == "playing")
        .map(|(n, p)| (*n, p.obj))
        .collect();
    let eye = [me[0], me[1], me[2] + 24.0];
    let mut best: Option<(f32, ObjRef)> = None;
    for (pn, pobj) in players {
        if Some(pn) == a.owner || gsc_t6::truthy(&vm.raw_field(pobj, ignore_s)) {
            continue;
        }
        if teambased && vm.to_text(&vm.raw_field(pobj, team_s)) == aiteam {
            continue;
        }
        let Some(ps) = frame(world).player(ClientId(pn)).copied() else {
            continue;
        };
        if ps.health <= 0 {
            continue;
        }
        let chest = [ps.origin[0], ps.origin[1], ps.origin[2] + 40.0];
        let seen = frame(world)
            .trace_static_world(eye, chest, [0.0; 3], [0.0; 3], MASK_AI)
            .fraction
            >= 1.0;
        if !seen {
            continue;
        }
        // The one it has wins while he is in sight.
        let d = if a.enemy == Some(pobj) {
            -1.0
        } else {
            dist(me, ps.origin)
        };
        if best.is_none_or(|(b, _)| d < b) {
            best = Some((d, pobj));
        }
    }
    best.map(|(_, o)| o)
=======
    Some(i)
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
}

/// Switch actor `n` to animscript `script` (killanimscript first).
pub(crate) fn run_script(vm: &mut Vm<World>, world: &mut World, n: u32, script: &str) {
    let obj = {
        let mut zm = world.resource_mut::<Zm>();
        let Some(a) = zm.actors.get_mut(&n) else {
            return;
        };
        if a.script == script && !script.starts_with("traverse/") {
            return;
        }
        a.script = script.to_owned();
        a.obj
    };
    let Some(obj) = obj else { return };
    vm.notify_str(world, obj, "killanimscript", &[]);
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    // bo2mp: a K9 Unit dog runs BO2's dog animscripts.
    let kind = if world.resource::<Zm>().actors.get(&n).is_some_and(is_mp_dog) {
        "dog"
    } else {
        "zm"
    };
    let path = if let Some(rest) = script.strip_prefix("traverse/") {
        format!("{ANIMSCRIPTS}/traverse/{rest}")
    } else {
        format!("{ANIMSCRIPTS}/{kind}_{script}")
=======
    let path = if let Some(rest) = script.strip_prefix("traverse/") {
        format!("{ANIMSCRIPTS}/traverse/{rest}")
    } else {
        format!("{ANIMSCRIPTS}/zm_{script}")
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
    };
    if vm
        .spawn_named(world, &path, "main", Value::Object(obj), vec![])
        .is_none()
    {
        vm.report_once(format!("no animscript {path}::main"));
    }
}

/// Field values the engine reads off an actor's script object.
fn field(vm: &mut Vm<World>, obj: ObjRef, name: &str) -> Value {
    let f = vm.intern(name);
    vm.raw_field(obj, f)
}

fn field_num(vm: &mut Vm<World>, obj: ObjRef, name: &str, default: f32) -> f32 {
    field(vm, obj, name).as_float().unwrap_or(default)
}

/// A position for an entity value (players from the sim).
fn pos_of(vm: &Vm<World>, world: &mut World, v: &Value) -> Option<[f32; 3]> {
    super::origin_of(vm, world, v)
}

/// Step height an actor walks up, and what it collides with: solid,
/// glass and monster clip (player clip is the player's alone).
const STEP: f32 = 18.0;
const MASK_AI: u32 = 0x0082_0011;

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

fn body(r: f32, h: f32) -> ([f32; 3], [f32; 3]) {
    ([-r, -r, 0.0], [r, r, (h - STEP - 14.0).max(8.0)])
}

/// Can an actor walk straight from `from` to `to`, its body clear of the
/// world above step height?
pub(crate) fn can_walk(world: &mut World, from: [f32; 3], to: [f32; 3], r: f32, h: f32) -> bool {
    if (to[2] - from[2]).abs() > 64.0 {
        return false;
    }
    let (mins, maxs) = body(r, h);
    let s = [from[0], from[1], from[2] + STEP];
    let e = [to[0], to[1], to[2] + STEP];
    let plats = platform_brushes(world, lerp3(from, to, 0.5), dist2(from, to) * 0.5);
    let t = clip_platforms(
        frame(world).trace_static_world(s, e, mins, maxs, MASK_AI),
        &plats,
        (s, e),
        (mins, maxs),
    );
    t.startsolid == 0 && t.fraction >= 1.0
}

/// Is there ground all along a straight walk from `from` to `to`, no drop
/// deeper than a walk steps down? `can_walk` only checks the body is clear:
/// at Buried's start, zombies in through one window walked straight at the
/// player off the ledge between, fell 144 units and stood in the room below
/// till distance tracking took them (soak 2).
fn floor_along(world: &mut World, from: [f32; 3], to: [f32; 3], r: f32, h: f32) -> bool {
    let (mins, maxs) = body(r, h);
    let f = frame(world);
    let len = ((to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2)).sqrt();
    let n = ((len / 32.0).ceil() as usize).max(1);
    (1..=n).all(|i| {
        let q = lerp3(from, to, i as f32 / n as f32);
        let s = [q[0], q[1], q[2] + STEP];
        let e = [q[0], q[1], q[2] - 48.0];
        f.trace_static_world(s, e, mins, maxs, MASK_AI).fraction < 1.0
            || f.trace_world(s, e, mins, maxs, MASK_AI).fraction < 1.0
            // A thin turned box is ground too, as in `walk_move`.
            || f.oriented_blockers().iter().any(|b| {
                if b.maxs[2] - b.mins[2] > 16.0 {
                    return false;
                }
                let d = [q[0] - b.origin[0], q[1] - b.origin[1], q[2] - b.origin[2]];
                let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
                let (lx, ly) = (dot(b.axis[0]), dot(b.axis[1]));
                if lx < b.mins[0] || lx > b.maxs[0] || ly < b.mins[1] || ly > b.maxs[1] {
                    return false;
                }
                let top =
                    b.origin[2] + b.axis[0][2] * lx + b.axis[1][2] * ly + b.axis[2][2] * b.maxs[2];
                top >= e[2] && top <= s[2]
            })
    })
}

/// Walk `d` from `from` (sliding along walls, stepping up `STEP`, down to
/// the ground): the new place, and whether the body stands clear of solid
/// there. `clear`: it has stood clear before, so it never steps into solid
/// (a body already inside only takes a step that ends outside).
pub(crate) fn walk_move(
    world: &mut World,
    from: [f32; 3],
    d: [f32; 2],
    r: f32,
    h: f32,
    clear: bool,
) -> ([f32; 3], bool) {
    let (mins, maxs) = body(r, h);
    let plats = platform_brushes(world, from, d[0].abs() + d[1].abs());
    let f = frame(world);
    let mut p = [from[0], from[1], from[2] + STEP];
    let mut rem = d;
    for _ in 0..3 {
        if rem[0].abs() + rem[1].abs() < 1e-3 {
            break;
        }
        let end = [p[0] + rem[0], p[1] + rem[1], p[2]];
        let t = clip_platforms(
            f.trace_static_world(p, end, mins, maxs, MASK_AI),
            &plats,
            (p, end),
            (mins, maxs),
        );
        if t.startsolid != 0 {
            // Inside the world already. One spawned in it walks out
            // freely; one that was clear (pushed in, dropped in) only takes
            // a step that ends outside - before, it could wander into
            // monster clip and stand there for good (the sprinters in the
            // clipped ditch behind the ledge, M4 retest 1).
            let inside = f.trace_static_world(end, end, mins, maxs, MASK_AI).startsolid != 0;
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
            if !clear || !inside {
=======
            // A walking step is ~1 unit: one left just inside a window's
            // clip by its climb-in anim never got a step that ends outside
            // and stood there till the scripts killed it (Origins round 1
            // never ended). A step on a heading that is out within 16 units
            // is fine.
            let len = (rem[0] * rem[0] + rem[1] * rem[1]).sqrt().max(1e-3);
            let ahead = [p[0] + rem[0] / len * 16.0, p[1] + rem[1] / len * 16.0, p[2]];
            let out_ahead = || f.trace_static_world(ahead, ahead, mins, maxs, MASK_AI).startsolid == 0;
            if !clear || !inside || (len < 16.0 && out_ahead()) {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
                p = end;
            }
            break;
        }
        p = lerp3(p, end, t.fraction);
        if t.fraction >= 1.0 {
            break;
        }
        // Slide along what stopped it.
        let left = 1.0 - t.fraction;
        let r2 = [rem[0] * left, rem[1] * left];
        let nl = (t.normal[0] * t.normal[0] + t.normal[1] * t.normal[1])
            .sqrt()
            .max(1e-3);
        let n = [t.normal[0] / nl, t.normal[1] / nl];
        let dot = r2[0] * n[0] + r2[1] * n[1];
        rem = [r2[0] - n[0] * dot, r2[1] - n[1] * dot];
        p = [p[0] + n[0] * 0.125, p[1] + n[1] * 0.125, p[2]];
    }
    let down = [p[0], p[1], p[2] - STEP - 48.0];
    let t = clip_platforms(
        f.trace_static_world(p, down, mins, maxs, MASK_AI),
        &plats,
        (p, down),
        (mins, maxs),
    );
    if t.startsolid != 0 {
        return ([p[0], p[1], from[2]], false);
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    (lerp3(p, down, t.fraction), true)
=======
    // A brush model under it is ground too, and so is a thin turned box (a
    // floor slab of a Die Rise elevator car or the crashed escape pod over
    // its pit: the zombies that came in fell through it).
    let on = f.trace_world(p, down, mins, maxs, MASK_AI);
    let frac = if on.startsolid == 0 { on.fraction.min(t.fraction) } else { t.fraction };
    let mut at = lerp3(p, down, frac);
    for b in f.oriented_blockers() {
        if b.maxs[2] - b.mins[2] > 16.0 {
            continue;
        }
        let d = [at[0] - b.origin[0], at[1] - b.origin[1], at[2] - b.origin[2]];
        let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
        let (lx, ly) = (dot(b.axis[0]), dot(b.axis[1]));
        if lx < b.mins[0] || lx > b.maxs[0] || ly < b.mins[1] || ly > b.maxs[1] {
            continue;
        }
        let top = b.origin[2] + b.axis[0][2] * lx + b.axis[1][2] * ly + b.axis[2][2] * b.maxs[2];
        if top > at[2] && top <= p[2] {
            at[2] = top;
        }
    }
    (at, true)
}

/// The solid boxes of the moving platforms near `p`, where they are now
/// (Tranzit's bus: its floor, door steps, walls and roof, `transit`): BO2's
/// actors walk on and against the bus's collision as players do. Ours only
/// met its thin roof: they walked through its walls and stood inside it at
/// the road's height, sunk through its floor. `reach`: how far from `p`
/// the move goes.
fn platform_brushes(world: &World, p: [f32; 3], reach: f32) -> Vec<crate::world::SimBrush> {
    let zm = world.resource::<Zm>();
    let mut out = Vec::new();
    for n in zm.riders.poses.keys() {
        let Some(e) = zm.ents.get(n) else {
            continue;
        };
        let boxes = super::transit::platform_boxes(&e.model);
        if e.hidden || boxes.is_empty() || dist2(e.origin, p) > 720.0 + reach {
            continue;
        }
        let (fw, rt, up) = gsc_t6::math::angle_vectors(e.angles);
        for &(mins, maxs) in boxes {
            let b = crate::OrientedBlocker {
                entnum: *n as u16,
                origin: e.origin,
                axis: [fw, [-rt[0], -rt[1], -rt[2]], up],
                mins,
                maxs,
                surface_flags: 0,
            };
            out.push(crate::world::SimBrush {
                planes: b.planes().to_vec(),
                contents: crate::bullet_collision::CONTENTS_SOLID,
                plane_surface_flags: vec![0; 6],
                glass_encoded: 0,
            });
        }
    }
    out
}

/// A trace of the world cut short by the platforms' boxes. One the body
/// starts inside never holds it (it walks out), as with players; one
/// already in the world's solid keeps that.
fn clip_platforms(
    mut t: trace_iw4::Trace,
    plats: &[crate::world::SimBrush],
    (s, e): ([f32; 3], [f32; 3]),
    (mins, maxs): ([f32; 3], [f32; 3]),
) -> trace_iw4::Trace {
    if t.startsolid != 0 {
        return t;
    }
    for b in plats {
        let o = trace_iw4::trace_box([clipmap_iw4::brush_ref(b)], s, e, mins, maxs, MASK_AI);
        if o.startsolid != 0 || o.allsolid != 0 || o.fraction >= t.fraction {
            continue;
        }
        t = o;
    }
    t
}

/// BO2's `maymovetopoint`: can the actor walk straight on from `from` to `to`,
/// stepping up and down as its walk does? No sliding round what is in the
/// way and no drop of more than a step. Buried's, Die Rise's and Origins'
/// distance tracking ask it after a `bad_path` and send a zombie that can not
/// reach anyone off to respawn; ours said yes always, so one with no route
/// kept the player as its enemy and stood still for good.
pub(crate) fn may_move(world: &mut World, from: [f32; 3], to: [f32; 3], r: f32, h: f32) -> bool {
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if len > 4096.0 {
        return false;
    }
    let n = ((len / 16.0).ceil() as usize).max(1);
    let step = [d[0] / n as f32, d[1] / n as f32];
    let mut p = from;
    let mut clear = false;
    for _ in 0..n {
        let want = [p[0] + step[0], p[1] + step[1]];
        let (at, ok) = walk_move(world, p, step, r, h, clear);
        let off = (at[0] - want[0]).powi(2) + (at[1] - want[1]).powi(2);
        if (clear && !ok) || off > 1.0 || at[2] < p[2] - 24.0 {
            return false;
        }
        clear |= ok;
        p = at;
    }
    (p[2] - to[2]).abs() <= 64.0
}

/// Where an actor put at `p` (`forceteleport`) comes to stand: the floor
/// under it, within 1024 units (else `p`). BO2's actors fall; ours only step
/// down while they walk, and one with no route doesn't walk: the zombies
/// alive when Buried's ghost round starts turn into ghosts that the map drops
/// from its drop-down spots (280 units over the street), and ours hung there
/// with no route anywhere, so the ghost round never ended.
pub(crate) fn floor_below(world: &mut World, p: [f32; 3], r: f32, h: f32) -> [f32; 3] {
    let (mins, maxs) = body(r, h);
    let f = frame(world);
    let from = [p[0], p[1], p[2] + STEP];
    let down = [p[0], p[1], p[2] - 1024.0];
    let t = f.trace_static_world(from, down, mins, maxs, MASK_AI);
    if t.startsolid != 0 {
        return p;
    }
    let on = f.trace_world(from, down, mins, maxs, MASK_AI);
    let frac = if on.startsolid == 0 { on.fraction.min(t.fraction) } else { t.fraction };
    if frac >= 1.0 {
        return p;
    }
    let at = lerp3(from, down, frac);
    if at[2] < p[2] { at } else { p }
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
}

/// The node a goal routes to: the nearest one that walks to it.
fn goal_node(world: &mut World, goal: [f32; 3], r: f32, h: f32) -> Option<u32> {
    let near: Vec<(u32, [f32; 3])> = {
        let zm = world.resource::<Zm>();
        zm.nav
            .within(goal, 384.0)
            .into_iter()
            .take(6)
            .map(|(i, _)| (i, zm.nav.nodes[i as usize].origin))
            .collect()
    };
    for (i, o) in near {
        if can_walk(world, o, goal, r, h) && floor_along(world, o, goal, r, h) {
            return Some(i);
        }
    }
    // None reaches it: no path, as BO2's engine finds none (its scripts
    // then send the zombie along the player's breadcrumbs, `bad_path`).
    // Before, the route ended at the nearest node and the zombie stood
    // there pushing at a ledge it could not climb (M4 retest 1, "still
    // getting stuck on some spots").
    None
}

/// Each path node down onto the floor below it (up to 512 units; one
/// inside solid or over nothing stays where it is).
pub(crate) fn drop_nodes_to_floor(world: &mut World, nodes: &mut [super::T6PathNode]) {
    let f = frame(world);
    let (mut moved, mut total, mut most, mut solid, mut none) = (0usize, 0.0f32, 0.0f32, 0usize, 0usize);
    for n in nodes.iter_mut() {
        // One on a mover (a Die Rise car) stands on it, not on the floor
        // under it.
        if n.spawnflags & super::nav::NODE_ON_MOVER != 0 {
            continue;
        }
        let from = [n.origin[0], n.origin[1], n.origin[2] + 16.0];
        let to = [n.origin[0], n.origin[1], n.origin[2] - 512.0];
        let t = f.trace_static_world(from, to, [-4.0, -4.0, 0.0], [4.0, 4.0, 4.0], MASK_AI);
        if t.startsolid != 0 {
            solid += 1;
            continue;
        }
        if t.fraction >= 1.0 {
            none += 1;
            continue;
        }
        let z = from[2] + (to[2] - from[2]) * t.fraction;
        let d = n.origin[2] - z;
        if d.abs() > 1.0 {
            moved += 1;
            total += d.abs();
            most = most.max(d.abs());
        }
        n.origin[2] = z;
    }
    diag::info!(
        Sim,
        "bo2zm t6 nav: {moved} of {} path nodes dropped to the floor (avg {:.0}, most {most:.0}); {solid} in solid, {none} over nothing",
        nodes.len(),
        if moved > 0 { total / moved as f32 } else { 0.0 }
    );
    let mut kinds = std::collections::BTreeMap::<&str, usize>::new();
    for n in nodes.iter() {
        *kinds.entry(super::nav::NODE_TYPE_NAMES.get(n.ty as usize).copied().unwrap_or("?")).or_default() += 1;
    }
    diag::info!(Sim, "bo2zm t6 nav kinds: {kinds:?}");
}

/// IW4L_T6_STALL_LOG=1: every actor's stalls (where, heading where).
fn stall_log() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("IW4L_T6_STALL_LOG").is_ok())
}

/// IW4L_T6_ACTOR_TRACE=<entity number>: that actor's every step.
pub(crate) fn trace_actor(n: u32) -> bool {
    static WHO: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *WHO.get_or_init(|| {
        std::env::var("IW4L_T6_ACTOR_TRACE")
            .ok()
            .and_then(|s| s.parse().ok())
    }) == Some(n)
}

fn set_path(
    world: &mut World,
    n: u32,
    goal: [f32; 3],
    nodes: Vec<u32>,
    node: Option<(u32, [f32; 3])>,
) {
    let mut zm = world.resource_mut::<Zm>();
    let mut path: Vec<[f32; 3]> = nodes
        .iter()
        .map(|&i| zm.nav.nodes[i as usize].origin)
        .collect();
    path.push(goal);
    if let Some(a) = zm.actors.get_mut(&n) {
        a.path = path;
        a.path_nodes = nodes;
        a.path_i = 0;
        a.goal_node = node;
    }
}

/// Plan a path to `goal` for actor `n`; false when none joins.
pub(crate) fn plan(world: &mut World, n: u32, goal: [f32; 3]) -> bool {
    let Some(from) = world.resource::<Zm>().ents.get(&n).map(|e| e.origin) else {
        return false;
    };
    let now = world.resource::<Zm>().now_ms;
    let Some((r, h, kept, has_nodes, straight)) = world.resource::<Zm>().actors.get(&n).map(|a| {
        (
            a.radius,
            a.height,
            a.goal_node,
            !a.path_nodes.is_empty(),
            a.no_straight_until <= now,
        )
    }) else {
        return false;
    };
    // Straight there when the way is clear and has floor all along.
    if straight
        && dist(from, goal) < 1024.0
        && can_walk(world, from, goal, r, h)
        && floor_along(world, from, goal, r, h)
    {
        set_path(world, n, goal, Vec::new(), None);
        return true;
    }
    // The same route while the goal's node holds (repaths come ~5 a second).
    let node = match kept {
        Some((b, at)) if dist(at, goal) < 48.0 => b,
        _ => match goal_node(world, goal, r, h) {
            Some(b) => b,
            None => return false,
        },
    };
    if has_nodes && kept.is_some_and(|(b, _)| b == node) {
        let mut zm = world.resource_mut::<Zm>();
        if let Some(a) = zm.actors.get_mut(&n)
            && let Some(last) = a.path.last_mut()
        {
            *last = goal;
        }
        return true;
    }
    // Start from the nodes this actor can walk to (else the nearest).
    let near: Vec<(u32, f32, [f32; 3])> = {
        let zm = world.resource::<Zm>();
        zm.nav
            .within(from, 400.0)
            .into_iter()
            .take(8)
            .map(|(i, d)| (i, d, zm.nav.nodes[i as usize].origin))
            .collect()
    };
    let mut seeds: Vec<(u32, f32)> = Vec::new();
    for (i, d, o) in near {
        if can_walk(world, from, o, r, h) && floor_along(world, from, o, r, h) {
            seeds.push((i, d));
        }
    }
    if seeds.is_empty() {
        let zm = world.resource::<Zm>();
        let Some(a) = zm.nav.nearest(from) else {
            return false;
        };
        seeds.push((a, dist(from, zm.nav.nodes[a as usize].origin)));
    }
    // Not through a node it stalled on lately (fix list 3); any way at all
    // when that leaves none.
    let avoid: Vec<u32> = {
        let zm = world.resource::<Zm>();
        zm.actors.get(&n).map_or(Vec::new(), |a| {
            a.avoid
                .iter()
                .filter(|(_, until)| *until > zm.now_ms)
                .map(|(i, _)| *i)
                .collect()
        })
    };
    let found = {
        let nav = &world.resource::<Zm>().nav;
        nav.path_from_avoiding(&seeds, node, &avoid).or_else(|| {
            (!avoid.is_empty())
                .then(|| nav.path_from(&seeds, node))
                .flatten()
        })
    };
    let Some(nodes) = found else {
        return false;
    };
    if trace_actor(n) {
        let zm = world.resource::<Zm>();
        let pts: Vec<String> = nodes
            .iter()
            .map(|&i| {
                let o = zm.nav.nodes[i as usize].origin;
                format!("{i}({:.0} {:.0} {:.0})", o[0], o[1], o[2])
            })
            .collect();
        diag::info!(
            Sim,
            "bo2zm t6 actor {n} plan from ({:.0} {:.0} {:.0}): {}",
            from[0],
            from[1],
            from[2],
            pts.join(" ")
        );
    }
    set_path(world, n, goal, nodes, Some((node, goal)));
    true
}

/// Path length from `from` to `to` (calcpathlength, findpath) for a body of
/// radius `r` and height `h`: its route starts at the nodes it can walk to
/// and ends at the nearest node that walks to `to`, as its own route does
/// (`plan`, `goal_node`). Each end was the single nearest node before, which
/// can stand behind a wall: Mob's Brutus 124 from the player read "no way
/// there", BO2's get_closest_player_using_paths skips a player with a 0
/// length past 36 units, and he stood idle (prison bhit11, 90-200 s).
pub(crate) fn path_length(world: &mut World, from: [f32; 3], to: [f32; 3], r: f32, h: f32) -> Option<f32> {
    let near: Vec<(u32, f32, [f32; 3])> = {
        let nav = &world.resource::<Zm>().nav;
        nav.within(from, 400.0)
            .into_iter()
            .take(8)
            .map(|(i, d)| (i, d, nav.nodes[i as usize].origin))
            .collect()
    };
    let mut seeds: Vec<(u32, f32)> = Vec::new();
    for (i, d, o) in near {
        if can_walk(world, from, o, r, h) {
            seeds.push((i, d));
        }
    }
    if seeds.is_empty() {
        let nav = &world.resource::<Zm>().nav;
        let a = nav.nearest(from)?;
        seeds.push((a, dist(from, nav.nodes[a as usize].origin)));
    }
    let b = match goal_node(world, to, r, h) {
        Some(b) => b,
        None => world.resource::<Zm>().nav.nearest(to)?,
    };
    let zm = world.resource::<Zm>();
    let nav = &zm.nav;
    let nodes = nav.path_from(&seeds, b)?;
    let mut len = dist(from, nav.nodes[nodes[0] as usize].origin);
    for w in nodes.windows(2) {
        len += dist(
            nav.nodes[w[0] as usize].origin,
            nav.nodes[w[1] as usize].origin,
        );
    }
    len += dist(nav.nodes[*nodes.last().unwrap() as usize].origin, to);
    Some(len)
}

/// Every tick: enemies, animscript states, animations and notetracks,
/// movement along paths, scripted animations, deaths.
pub(crate) fn think(world: &mut World, now: i64) {
    let ids: Vec<u32> = world.resource::<Zm>().actors.keys().copied().collect();
    if ids.is_empty() {
        return;
    }
    let before: Vec<(u32, [f32; 3])> = {
        let zm = world.resource::<Zm>();
        ids.iter()
            .filter_map(|n| zm.ents.get(n).map(|e| (*n, e.origin)))
            .collect()
    };
    with_vm(world, |vm, world| {
        for n in ids {
            think_one(vm, world, n, now);
        }
        // His number of corpses (BO2's `ai_corpseCount`): past it the
        // oldest goes, as the 15-second clock takes them.
        let cap = super::corpse_limit();
        let mut dead: Vec<(i64, u32, Option<ObjRef>)> = world
            .resource::<Zm>()
            .actors
            .iter()
            .filter(|(_, a)| !a.alive)
            .filter_map(|(n, a)| a.dead_since.map(|t| (t, *n, a.obj)))
            .collect();
        if dead.len() > cap {
            dead.sort_unstable_by_key(|d| (d.0, d.1));
            let extra = dead.len() - cap;
            for (_, n, obj) in dead.into_iter().take(extra) {
                world.resource_mut::<Zm>().actors.remove(&n);
                world.resource_mut::<Zm>().ents.remove(&n);
                if let Some(obj) = obj
                    && vm.alive(obj)
                {
                    vm.free_object(world, obj);
                }
            }
        }
    });
    separate(world);
    // What `getvelocity` says: how far each went this tick (none for the
    // dead, or for a jump no walk makes in a tick, a teleport).
    let dt = f32::from(crate::MATCH_TICK_MS as u16) / 1000.0;
    let mut zm = world.resource_mut::<Zm>();
    for (n, was) in before {
        let Some(at) = zm.ents.get(&n).map(|e| e.origin) else {
            continue;
        };
        if let Some(a) = zm.actors.get_mut(&n) {
            let v: [f32; 3] = std::array::from_fn(|i| (at[i] - was[i]) / dt);
            let fast = v.iter().map(|x| x * x).sum::<f32>() > 2000.0 * 2000.0;
            a.velocity = if a.alive && !fast { v } else { [0.0; 3] };
        }
    }
    // IW4L_T6_TEARLOG=1 (test aid): every living actor's place, clip and
    // scripted flag each tick (a board pull's jumps between its clips).
    static TEARLOG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *TEARLOG.get_or_init(|| std::env::var_os("IW4L_T6_TEARLOG").is_some()) {
        for (n, a) in zm.actors.iter().filter(|(_, a)| a.alive) {
            let Some(e) = zm.ents.get(n) else { continue };
            let clip = a.playing.as_ref().map_or("-", |p| p.anim.as_str());
            diag::info!(
                Sim,
                "bo2zm t6 tear {n} t{now} s{} {} {clip} ({:.1} {:.1} {:.1}) y{:.0}",
                u8::from(a.scripted.is_some()),
                a.script,
                e.origin[0],
                e.origin[1],
                e.origin[2],
                e.angles[1]
            );
        }
    }
}

fn think_one(vm: &mut Vm<World>, world: &mut World, n: u32, now: i64) {
    let Some(a) = world.resource::<Zm>().actors.get(&n).cloned() else {
        return;
    };
    let Some(obj) = a.obj else { return };
    if !vm.alive(obj) {
        world.resource_mut::<Zm>().actors.remove(&n);
        return;
    }
    let dt = f32::from(crate::MATCH_TICK_MS as u16) / 1000.0;
    // Corpses go after a while.
    if !a.alive {
        if let Some(since) = a.dead_since
            && now - since > 15_000
        {
            world.resource_mut::<Zm>().actors.remove(&n);
            world.resource_mut::<Zm>().ents.remove(&n);
            vm.free_object(world, obj);
            return;
        }
        advance_anim(vm, world, n, obj, now, dt);
        return;
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    let dog = is_mp_dog(&a);
    let here = world
        .resource::<Zm>()
        .ents
        .get(&n)
        .map(|e| e.origin)
        .unwrap_or([0.0; 3]);
    // Enemy: the favorite enemy when not ignoring all (a dog finds its
    // own).
=======
    // Enemy: the favorite enemy when not ignoring all.
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
    let ignoreall = gsc_t6::truthy(&field(vm, obj, "ignoreall"));
    let fav = field(vm, obj, "favoriteenemy");
    let enemy = match fav {
        _ if dog => dog_enemy(vm, world, &a, obj, here),
        Value::Object(o) if !ignoreall && vm.alive(o) => Some(o),
        _ => None,
    };
    if enemy != a.enemy {
        if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
            act.enemy = enemy;
        }
        vm.notify_str(world, obj, "enemy", &[]);
    }
    // Animscript state.
    let me = world
        .resource::<Zm>()
        .ents
        .get(&n)
        .map(|e| e.origin)
        .unwrap_or([0.0; 3]);
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    let scripted = a.scripted.is_some();
    let traversing = a.traverse.is_some();
    let mut a = a;
    if dog && !scripted && !traversing && a.alive {
        // bo2mp: a dog runs at its enemy (its patrol waits meanwhile) and
        // leaps when he is in reach; one leap done, its combat script runs
        // again while he is.
        let target = enemy.and_then(|e| pos_of(vm, world, &Value::Object(e)));
        if let Some(t) = target
            && dist2(t, me) > dog_attack_range(world)
            && a.goal.is_none_or(|g| dist2(g, t) > 64.0)
        {
            let found = plan(world, n, t);
            if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
                act.goal = Some(t);
                act.goal_sent = true;
                if !found {
                    act.path.clear();
                    act.path_nodes.clear();
                }
            }
        }
        let leapt = a.script == "combat"
            && a
                .playing
                .as_ref()
                .is_some_and(|p| p.ended && p.state.starts_with("combat_attack"));
        if leapt && let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
            act.script.clear();
        }
        if let Some(x) = world.resource::<Zm>().actors.get(&n).cloned() {
            a = x;
        }
=======
    let custom = a.custom.is_some_and(|t| vm.thread_running(t));
    if a.custom.is_some() && !custom {
        if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
            act.custom = None;
            act.script.clear();
        }
        diag::info!(
            Sim,
            "bo2zm t6 animcustom ent{n} done at ({:.0} {:.0} {:.0})",
            me[0],
            me[1],
            me[2]
        );
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
    }
    let scripted = a.scripted.is_some() || custom;
    let traversing = a.traverse.is_some();
    if !scripted && !traversing && a.script != "init" {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
        let fight = if dog {
            dog_attack_range(world)
        } else {
            field_num(vm, obj, "pathenemyfightdist", 64.0).max(1.0)
        };
=======
        let fight = field_num(vm, obj, "pathenemyfightdist", 64.0).max(1.0);
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
        let goalradius = field_num(vm, obj, "goalradius", 32.0).max(1.0);
        let goalheight = field_num(vm, obj, "goalheight", GOAL_HEIGHT).max(1.0);
        let near_enemy = enemy
            .and_then(|e| pos_of(vm, world, &Value::Object(e)))
            .is_some_and(|p| dist2(p, me) <= fight && (p[2] - me[2]).abs() < 72.0);
        let in_goal = a
            .goal
            .is_none_or(|g| dist2(g, me) <= goalradius && (g[2] - me[2]).abs() <= goalheight);
        let want = if near_enemy {
            "combat"
        } else if !in_goal && !a.path.is_empty() {
            "move"
        } else {
            "stop"
        };
        // Combat holds through a swing while the enemy is still about (the
        // edge of the fight distance flickers as he moves); once he is
        // clearly out of reach it ends at once, mid-swing, as BO2's engine
        // changes script unless the melee script says it is not safe to
        // (`safetochangescript` false, only with `nochangeduringmelee`). Before,
        // the combat script started the next swing in the same tick the last
        // one ended, so a zombie that swung kept swinging at the air within
        // the scripts' 512-unit long-range melee and never followed him.
        let swinging = a
            .playing
            .as_ref()
            .is_some_and(|p| p.notify == "melee_anim" && !p.ended);
        let enemy_far = enemy
            .and_then(|e| pos_of(vm, world, &Value::Object(e)))
            .is_none_or(|p| dist2(p, me) > fight + 24.0);
        let safe = field(vm, obj, "safetochangescript");
        let unsafe_change = !safe.is_undefined() && !gsc_t6::truthy(&safe);
        let hold = a.script == "combat" && (unsafe_change || (swinging && !enemy_far));
        if want != a.script && !hold {
            run_script(vm, world, n, want);
            // The stall clock runs only while it walks: one that stood and
            // swung for a while was taken as stalled the moment it walked
            // again (and since M4 retest 1 a stall on the last stretch says
            // `bad_path`: it left him for his breadcrumbs).
            if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
                act.progress = None;
            }
        }
    }
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
        act.yaw_goal = None;
    }
    let a = match world.resource::<Zm>().actors.get(&n).cloned() {
        Some(a) => a,
        None => return,
    };
    // Movement. A linked actor goes where its parent takes it (a zombie
    // held on the bus's window between its clips): no walk or turn of its
    // own, which the link put back each tick.
    let linked = world
        .resource::<Zm>()
        .ents
        .get(&n)
        .is_some_and(|e| e.link.is_some());
    if let Some(tr) = &a.traverse {
        let f = ((now - tr.start_ms) as f32 / tr.length_ms.max(1) as f32).clamp(0.0, 1.0);
        let p: [f32; 3] = std::array::from_fn(|i| tr.from[i] + (tr.to[i] - tr.from[i]) * f);
        set_origin(world, n, p);
        if f >= 1.0 {
            let end = tr.end_node;
            if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
                act.traverse = None;
                // Continue the path after the end node.
                if let Some(i) = act.path_nodes.iter().position(|&x| x == end) {
                    act.path_i = i + 1;
                }
                act.script.clear();
            }
        }
    } else if a.scripted.is_none() && a.script == "move" && !linked {
        follow_path(vm, world, n, obj, &a, me, dt, now);
    } else if a.scripted.is_none()
        && !linked
        && (a.script == "combat" || a.script == "stop")
        && !(a.goal_sent && a.goal_yaw.is_some())
    {
        // A scripted clip (a window tear) keeps the angles it began with;
        // turning to the player there spun the zombie off its window.
        face(vm, world, n, obj, &a, me, None);
    }
    // Goal reached.
    if let Some(g) = a.goal {
        let goalradius = field_num(vm, obj, "goalradius", 32.0).max(1.0);
        let goalheight = field_num(vm, obj, "goalheight", GOAL_HEIGHT).max(1.0);
        if !a.goal_sent && dist2(g, me) <= goalradius && (g[2] - me[2]).abs() <= goalheight {
            if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
                act.goal_sent = true;
            }
            vm.notify_str(world, obj, "goal", &[]);
        }
    }
    orient_at_goal(vm, world, n, obj);
    pitch_to_ground(world, n, a.pitch_orient);
    advance_anim(vm, world, n, obj, now, dt);
}

/// bo2zm M3 fix list 2: a zombie asked to `setpitchorient` tilts to the
/// ground along its facing (the ground 16 units ahead against 16 behind),
/// eased a third of the way each tick; level again once it is cleared. The
/// map's crater zombies crawl up the slope this way; before they crawled
/// level, half in the slope.
fn pitch_to_ground(world: &mut World, n: u32, on: bool) {
    let Some(e) = world.resource::<Zm>().ents.get(&n).cloned() else {
        return;
    };
    let want = if on {
        let (s, c) = e.angles[1].to_radians().sin_cos();
        let ground = |world: &mut World, d: f32| -> Option<f32> {
            let at = [e.origin[0] + c * d, e.origin[1] + s * d];
            let from = [at[0], at[1], e.origin[2] + 40.0];
            let to = [at[0], at[1], e.origin[2] - 80.0];
            let t = frame(world).trace_static_world(
                from,
                to,
                [0.0; 3],
                [0.0; 3],
                crate::bullet_collision::MASK_PLAYER_SOLID,
            );
            (t.fraction < 1.0).then(|| from[2] + (to[2] - from[2]) * t.fraction)
        };
        match (ground(world, 16.0), ground(world, -16.0)) {
            // IW pitch: positive looks down; climbing tilts the nose up.
            (Some(front), Some(back)) => {
                -((front - back).atan2(32.0).to_degrees()).clamp(-60.0, 60.0)
            }
            _ => e.angles[0],
        }
    } else {
        0.0
    };
    // IW4L_T6_PITCHLOG=1: a tilting zombie's wanted and current pitch.
    if on && std::env::var_os("IW4L_T6_PITCHLOG").is_some() {
        diag::info!(
            Sim,
            "bo2zm t6 pitch {n}: want {want:.1} now {:.1} at ({:.0} {:.0} {:.0})",
            e.angles[0],
            e.origin[0],
            e.origin[1],
            e.origin[2]
        );
    }
    if (want - e.angles[0]).abs() < 0.01 {
        return;
    }
    if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n) {
        e.angles[0] += (want - e.angles[0]) / 3.0;
        if !on && e.angles[0].abs() < 0.5 {
            e.angles[0] = 0.0;
        }
    }
}

fn set_origin(world: &mut World, n: u32, p: [f32; 3]) {
    if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n) {
        e.origin = p;
    }
}

fn set_yaw(world: &mut World, n: u32, yaw: f32) {
    if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n) {
        // The pitch stays (a crawler tilted to its slope, `pitch_to_ground`).
        e.angles = [e.angles[0], yaw, 0.0];
    }
}

fn yaw_to(from: [f32; 3], to: [f32; 3]) -> Option<f32> {
    let d = [to[0] - from[0], to[1] - from[1]];
    (d[0] * d[0] + d[1] * d[1] > 1e-4).then(|| d[1].atan2(d[0]).to_degrees())
}

/// Turn by orientmode: face motion (`dir`), the enemy, an angle or a point.
#[allow(clippy::too_many_arguments)]
fn face(
    vm: &mut Vm<World>,
    world: &mut World,
    n: u32,
    _obj: ObjRef,
    a: &Actor,
    me: [f32; 3],
    motion: Option<f32>,
) {
    let target = match a.orientmode.as_str() {
        "face angle" => Some(a.orient_yaw),
        "face point" => yaw_to(me, a.orient_point),
        "face enemy" | "face enemy or motion" | "face default" | "" => {
            let e = a.enemy.and_then(|e| pos_of(vm, world, &Value::Object(e)));
            match (a.orientmode.as_str(), motion, e) {
                ("face enemy", _, Some(p)) => yaw_to(me, p),
                (_, Some(m), _) => Some(m),
                (_, None, Some(p)) => yaw_to(me, p),
                _ => None,
            }
        }
        "face motion" => motion,
        _ => motion,
    };
    let Some(want) = target else { return };
    let cur = world
        .resource::<Zm>()
        .ents
        .get(&n)
        .map_or(0.0, |e| e.angles[1]);
    let delta = gsc_t6::math::angle_clamp180(want - cur);
    let step = TURN_DEG_PER_S * 0.05;
    let yaw = cur + delta.clamp(-step, step);
    set_yaw(world, n, yaw);
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
        act.yaw_goal = Some(want);
    }
}

/// There with `setgoalpos`'s angles: turn to them at the turn rate, then
/// "orientdone" (once; the facing is then spent). Before, a window tear
/// waited its full second unturned and snapped round when its clip began.
fn orient_at_goal(vm: &mut Vm<World>, world: &mut World, n: u32, obj: ObjRef) {
    let Some(want) = world
        .resource::<Zm>()
        .actors
        .get(&n)
        .filter(|a| a.goal_sent && a.scripted.is_none())
        .and_then(|a| a.goal_yaw)
    else {
        return;
    };
    let cur = world
        .resource::<Zm>()
        .ents
        .get(&n)
        .map_or(want, |e| e.angles[1]);
    let delta = gsc_t6::math::angle_clamp180(want - cur);
    let step = TURN_DEG_PER_S * 0.05;
    set_yaw(world, n, cur + delta.clamp(-step, step));
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
        act.yaw_goal = Some(want);
        if delta.abs() <= step {
            act.goal_yaw = None;
        }
    }
    if delta.abs() <= step {
        vm.notify_str(world, obj, "orientdone", &[]);
    }
}

/// How fast a zombie turns toward its facing (18 degrees a 50 ms tick).
pub(crate) const TURN_DEG_PER_S: f32 = 360.0;

/// Is the path's point `i` the start of a traversal it takes?
fn traversal_at(world: &World, a: &Actor, i: usize) -> bool {
    let (Some(&node), Some(&next)) = (a.path_nodes.get(i), a.path_nodes.get(i + 1)) else {
        return false;
    };
    let nav = &world.resource::<Zm>().nav;
    nav.nodes[node as usize].ty == NODE_NEGOTIATION_BEGIN && nav.negotiation(node, next)
}

#[allow(clippy::too_many_arguments)]
fn follow_path(
    vm: &mut Vm<World>,
    world: &mut World,
    n: u32,
    obj: ObjRef,
    a: &Actor,
    me: [f32; 3],
    dt: f32,
    now: i64,
) {
    let mut i = a.path_i;
    // bo2zm M3 fix list 3: a point within reach is passed (not a
    // traversal's start, which it must stand on). Before, a body only moved
    // on by landing on the point itself; zombies rising together in the
    // crater crowded their first point, pushed each other off it and stood
    // there until the scripts' 30-second failsafe killed and respawned them:
    // rounds that dragged on with zombies heard but never coming.
    while i + 1 < a.path.len() && !traversal_at(world, a, i) && dist2(me, a.path[i]) < PASS_REACH {
        i += 1;
    }
    // One that has stood still too long plans again from here (the kept
    // route only changes when the goal's node does).
    match a.progress {
        Some((at, since)) if dist2(at, me) < STALL_DIST => {
            if now - since >= STALL_MS {
                if let Some(goal) = a.goal {
                    // Stalled again on the same node: plan round it for 15 s.
                    let stuck_on = a.path_nodes.get(i).copied();
                    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
                        act.goal_node = None;
                        act.path_nodes.clear();
                        act.progress = Some((me, now));
                        act.avoid.retain(|(_, until)| *until > now);
                        if let Some(node) = stuck_on {
                            if a.avoid.iter().any(|(x, _)| *x == node) || a.stalled_on == Some(node)
                            {
                                act.avoid.push((node, now + 15_000));
                            }
                            act.stalled_on = Some(node);
                        }
                    }
                    let last_leg = i + 1 >= a.path.len();
                    if trace_actor(n) || stall_log() {
                        let t = a.path.get(i).copied().unwrap_or(goal);
                        // What the walking tests say from here: a straight
                        // sweep there, and a 16-unit step towards it.
                        let sweep = can_walk(world, me, t, a.radius, a.height);
                        let d = [t[0] - me[0], t[1] - me[1]];
                        let len = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-3);
                        let (step, clear) =
                            walk_move(world, me, [d[0] / len * 16.0, d[1] / len * 16.0], a.radius, a.height, a.clear);
                        diag::info!(
                            Sim,
                            "bo2zm t6 actor {n} stall probe: sweep {sweep} step ({:.0} {:.0} {:.0}) moved {:.1} clear {clear} was clear {} speed anim {:?}",
                            step[0],
                            step[1],
                            step[2],
                            ((step[0] - me[0]).powi(2) + (step[1] - me[1]).powi(2)).sqrt(),
                            a.clear,
                            a.playing.as_ref().map(|p| p.anim.as_str())
                        );
                        diag::info!(
                            Sim,
                            "bo2zm t6 actor {n} t{now}: stalled at ({:.0} {:.0} {:.0}) to ({:.0} {:.0} {:.0}) point {i}/{} goal ({:.0} {:.0} {:.0}){}, plans again",
                            me[0],
                            me[1],
                            me[2],
                            t[0],
                            t[1],
                            t[2],
                            a.path.len(),
                            goal[0],
                            goal[1],
                            goal[2],
                            if last_leg { " last leg" } else { "" }
                        );
                    }
                    // Stuck on the way to the goal itself, or no way found:
                    // BO2's engine says `bad_path` (the scripts pick another
                    // breadcrumb). It keeps the route it had meanwhile.
                    // A straight walk that stalled tries the path nodes.
                    let straight = a.path_nodes.is_empty();
                    if last_leg
                        && straight
                        && let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n)
                    {
                        act.no_straight_until = now + 5_000;
                    }
                    let found = (!last_leg || straight) && plan(world, n, goal);
                    if !found {
                        if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
                            act.goal_node = a.goal_node;
                            act.path_nodes = a.path_nodes.clone();
                        }
                        super::natives_game::notify_later_once(vm, world, obj, "bad_path", 50);
                    }
                }
                return;
            }
        }
        _ => {
            if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
                act.progress = Some((me, now));
            }
        }
    }
    let Some(&target) = a.path.get(i) else { return };
    // A traversal starts at its begin node.
    if traversal_at(world, a, i) && dist2(me, target) < 24.0 {
        let (end, script) = {
            let zm = world.resource::<Zm>();
            let node = a.path_nodes[i];
            let next = a.path_nodes[i + 1];
            (
                zm.nav.nodes[next as usize].origin,
                zm.nav.nodes[node as usize].animscript.clone(),
            )
        };
        if !script.is_empty() {
            start_traverse(
                vm,
                world,
                n,
                obj,
                a.path_nodes[i],
                a.path_nodes[i + 1],
                me,
                end,
                &script,
                now,
            );
            return;
        }
    }
    // Look ahead: on to the next point when it is in plain walking reach.
    if i + 1 < a.path.len()
        && !traversal_at(world, a, i)
        && can_walk(world, me, a.path[i + 1], a.radius, a.height)
    {
        i += 1;
    }
    let speed = a
        .playing
        .as_ref()
        .and_then(|p| world.resource::<Zm>().anims.get(&p.anim).map(anim_speed))
        .filter(|s| *s > 1.0)
        .unwrap_or(60.0)
        * a.anim_rate.unwrap_or(1.0);
    // The step along the path's points (x, y; the ground gives the height).
    let mut step = speed * dt;
    let mut at = [me[0], me[1]];
    let mut motion = None;
    while step > 1e-3 {
        let Some(&t) = a.path.get(i) else { break };
        let d = [t[0] - at[0], t[1] - at[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        if len <= step {
            at = [t[0], t[1]];
            step -= len;
            if traversal_at(world, a, i) || i + 1 >= a.path.len() {
                break;
            }
            i += 1;
        } else {
            at = [at[0] + d[0] / len * step, at[1] + d[1] / len * step];
            motion = yaw_to(me, [at[0], at[1], me[2]]);
            step = 0.0;
        }
    }
    let (pos, clear) = walk_move(
        world,
        me,
        [at[0] - me[0], at[1] - me[1]],
        a.radius,
        a.height,
        a.clear,
    );
    if clear
        && !a.clear
        && let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n)
    {
        act.clear = true;
    }
    if trace_actor(n) {
        diag::info!(
            Sim,
            "bo2zm t6 actor {n} t{now}: me ({:.1} {:.1} {:.1}) -> ({:.1} {:.1} {:.1}) target ({:.0} {:.0} {:.0}) speed {speed:.1} path {}->{i}/{}",
            me[0],
            me[1],
            me[2],
            pos[0],
            pos[1],
            pos[2],
            target[0],
            target[1],
            target[2],
            a.path_i,
            a.path.len()
        );
    }
    set_origin(world, n, pos);
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
        act.path_i = i;
    }
    let toward = a.path.get(i).copied().unwrap_or(target);
    face(
        vm,
        world,
        n,
        obj,
        a,
        pos,
        motion.or_else(|| yaw_to(me, toward)),
    );
}

/// Bodies don't overlap: actors push apart from each other and step
/// back from players.
fn separate(world: &mut World) {
    let bodies: Vec<(u32, [f32; 3], f32, f32)> = {
        let zm = world.resource::<Zm>();
        zm.actors
            .iter()
            .filter(|(_, a)| {
                a.alive && a.scripted.is_none() && a.traverse.is_none() && a.script != "init"
            })
            .filter_map(|(n, a)| zm.ents.get(n).map(|e| (*n, e.origin, a.radius, a.height)))
            .collect()
    };
    if bodies.is_empty() {
        return;
    }
    let players: Vec<[f32; 3]> = {
        let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
        let f = frame(world);
        ids.iter()
            .filter_map(|&c| f.player(ClientId(c)).map(|p| p.origin))
            .collect()
    };
    let mut push: BTreeMap<u32, [f32; 2]> = BTreeMap::new();
    let mut shove = |n: u32, d: [f32; 2]| {
        let e = push.entry(n).or_insert([0.0; 2]);
        e[0] += d[0];
        e[1] += d[1];
    };
    for (k, &(na, pa, ra, _)) in bodies.iter().enumerate() {
        for &(nb, pb, rb, _) in &bodies[k + 1..] {
            if (pa[2] - pb[2]).abs() > 48.0 {
                continue;
            }
            let d = [pb[0] - pa[0], pb[1] - pa[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let overlap = ra + rb - len;
            if overlap <= 0.0 {
                continue;
            }
            let dir = if len > 1e-3 {
                [d[0] / len, d[1] / len]
            } else {
                [1.0, 0.0]
            };
            let k = (overlap * 0.5).min(3.0);
            shove(na, [-dir[0] * k, -dir[1] * k]);
            shove(nb, [dir[0] * k, dir[1] * k]);
        }
        for p in &players {
            if (pa[2] - p[2]).abs() > 48.0 {
                continue;
            }
            let d = [pa[0] - p[0], pa[1] - p[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let overlap = ra + 15.0 - len;
            if overlap > 0.0 {
                let dir = if len > 1e-3 {
                    [d[0] / len, d[1] / len]
                } else {
                    [1.0, 0.0]
                };
                let k = overlap.min(4.0);
                shove(na, [dir[0] * k, dir[1] * k]);
            }
        }
    }
    for (n, d) in push {
        let Some(&(_, p, r, h)) = bodies.iter().find(|b| b.0 == n) else {
            continue;
        };
        let clear = world.resource::<Zm>().actors.get(&n).is_some_and(|a| a.clear);
        let (np, _) = walk_move(world, p, d, r, h, clear);
        set_origin(world, n, np);
    }
}

#[allow(clippy::too_many_arguments)]
fn start_traverse(
    vm: &mut Vm<World>,
    world: &mut World,
    n: u32,
    _obj: ObjRef,
    begin: u32,
    end: u32,
    from: [f32; 3],
    to: [f32; 3],
    script: &str,
    now: i64,
) {
    let _ = begin;
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
        act.traverse = Some(Traverse {
            from,
            to,
            start_ms: now,
            length_ms: 1200,
            end_node: end,
        });
        act.script.clear();
    }
    if let Some(yaw) = yaw_to(from, to) {
        set_yaw(world, n, yaw);
    }
    run_script(vm, world, n, &format!("traverse/{script}"));
    // The traverse animation's length times the crossing.
    let len = world
        .resource::<Zm>()
        .actors
        .get(&n)
        .and_then(|a| a.playing.as_ref().map(|p| p.length_ms))
        .unwrap_or(1200);
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n)
        && let Some(t) = act.traverse.as_mut()
    {
        t.length_ms = len.max(300);
    }
}

/// Advance the playing animation: notetracks, `end`, scripted root motion.
fn advance_anim(vm: &mut Vm<World>, world: &mut World, n: u32, obj: ObjRef, now: i64, _dt: f32) {
    let Some(a) = world.resource::<Zm>().actors.get(&n).cloned() else {
        return;
    };
    let Some(p) = a.playing.clone() else { return };
    if p.ended {
        return;
    }
    let anim = world.resource::<Zm>().anims.get(&p.anim).cloned();
    let elapsed = p.elapsed(now);
    let cycle = if p.looping { elapsed / p.length_ms } else { 0 };
    let frac = if p.looping {
        (elapsed % p.length_ms) as f32 / p.length_ms as f32
    } else {
        (elapsed as f32 / p.length_ms as f32).min(1.0)
    };
    let mut events: Vec<String> = Vec::new();
    let notes: Vec<(String, f32)> = anim
        .as_ref()
        .map(|x| x.notifies.clone())
        .unwrap_or_default();
    // Clips whose notetracks carry their own `end` send it once: a second
    // one would wake the wait the script starts on the first (the window
    // boards' tear in, then out, cut the out short).
    let own_end = notes.iter().any(|(name, _)| name == "end");
    let mut sent = p.sent;
    let mut cur_cycle = p.cycle;
    while cur_cycle < cycle {
        // Finish the old cycle's notes, then its end.
        for (name, _) in notes.iter().skip(sent) {
            events.push(name.clone());
        }
        if !own_end {
            events.push("end".to_owned());
        }
        sent = 0;
        cur_cycle += 1;
    }
    while sent < notes.len() && notes[sent].1 <= frac {
        events.push(notes[sent].0.clone());
        sent += 1;
    }
    let finished = !p.looping && frac >= 1.0;
    if finished {
        for (name, _) in notes.iter().skip(sent) {
            events.push(name.clone());
        }
        if !own_end {
            events.push("end".to_owned());
        }
        sent = notes.len();
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    // Scripted animations move by their root motion from where they began
    // (and a dog's leap: its attack carries it at him).
    let leap = is_mp_dog(&a) && p.state.starts_with("combat_attack");
    if (a.scripted.is_some() || leap)
=======
    // Scripted animations move by their root motion from where they began.
    if (a.scripted.is_some() || a.custom.is_some())
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
        && let Some(anim) = &anim
    {
        let root = anim_root(anim, frac);
        let (s, c) = p.yaw_from.to_radians().sin_cos();
        let off = [
            root[0] * c - root[1] * s,
            root[0] * s + root[1] * c,
            root[2],
        ];
        let mut pos = [
            p.root_from[0] + off[0],
            p.root_from[1] + off[1],
            p.root_from[2] + off[2],
        ];
        // Linked (a zombie on the bus's window or roof tag): the clip is
        // carried by its parent's move and turn since it began, and the
        // link keeps the place it reached (BO2's linked animscripted).
        let carry = p
            .parent_from
            .and_then(|from| super::movers::link_parent_pose(world, n).map(|now| (from, now)));
        if let Some(((o0, a0), (o1, a1))) = carry {
            pos = super::riders::to_world(super::riders::to_local(pos, o0, a0), o1, a1);
            let yaw = p.yaw_from + math_iw4::angle_subtract(a1[1], a0[1]);
            set_origin(world, n, pos);
            set_yaw(world, n, yaw);
            super::movers::relink_here(world, n);
        } else {
            set_origin(world, n, pos);
        }
    }
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n)
        && let Some(pp) = act.playing.as_mut()
    {
        pp.sent = sent;
        pp.cycle = cur_cycle;
        if finished {
            pp.ended = true;
        }
    }
    for ev in events {
        if ev.is_empty() {
            continue;
        }
        let note = vm.string(&ev);
        vm.notify_str(world, obj, &p.notify, &[note]);
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    // A scripted animation's end hands the actor back to its AI.
    if finished && a.scripted.is_some() && a.alive {
=======
    // A scripted animation's end hands the actor back to its AI, unless its
    // `end` already started the next scripted one (the window boards' tear
    // in, out). A plain `setanimstatefromasd` there does not keep it
    // scripted (Buried's ghost: spawn clip, then its run: it stood still).
    let restarted = world
        .resource::<Zm>()
        .actors
        .get(&n)
        .is_some_and(|x| x.scripted_starts != a.scripted_starts);
    if finished && a.scripted.is_some() && a.alive && !restarted {
        // The anim put it where it ends, like a spawn: one it left inside
        // solid walks out freely (Buried's candy store, middle window: the
        // climb-in ends in solid and it walked in place till the round
        // could not end; the left and right climbs end clear).
        let at = world.resource::<Zm>().ents.get(&n).map(|e| e.origin);
        let inside = at.is_some_and(|q| {
            let (mins, maxs) = body(a.radius, a.height);
            let q = [q[0], q[1], q[2] + STEP];
            frame(world).trace_static_world(q, q, mins, maxs, MASK_AI).startsolid != 0
        });
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
        if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
            act.scripted = None;
            act.script.clear();
            if inside {
                act.clear = false;
            }
        }
        vm.notify_str(world, obj, "end_sequence", &[]);
        vm.notify_str(world, obj, "killanimscript", &[]);
    }
}

/// Spawn an actor from a spawner entity: the aitype's main (model, head,
/// fields), then the init animscript, as the engine does inside spawnactor.
pub(crate) fn spawn_actor(
    vm: &mut Vm<World>,
    world: &mut World,
    spawner: ObjRef,
) -> Option<ObjRef> {
    let sn = match vm.kind(spawner)? {
        ObjKind::Entity(n) => n,
        _ => return None,
    };
    let sp = world.resource::<Zm>().ents.get(&sn).cloned()?;
    let aitype = sp.classname.strip_prefix("actor_")?.to_owned();
    let n = world.resource_mut::<Zm>().alloc_entnum();
    let obj = vm.alloc_object(ObjKind::Entity(n));
    // The spawner's script fields come along (spawn_funcs, script_*).
    for f in vm.field_names(spawner) {
        let v = vm.raw_field(spawner, f);
        vm.set_raw_field(obj, f, v);
    }
    if std::env::var_os("IW4L_T6_SPAWNLOG").is_some() {
        let f = vm.intern("spawn_funcs");
        let funcs = match vm.raw_field(spawner, f) {
            Value::Array(a) => a.snapshot().len(),
            _ => 0,
        };
        diag::info!(
            Sim,
            "bo2zm t6 spawnactor: ent{n} {obj:?} {aitype} from spawner ent{sn} {spawner:?} at {:?}, {funcs} spawn_funcs",
            sp.origin
        );
    }
    for (k, v) in [("isdog", Value::Int(0)), ("delayeddeath", Value::Int(0))] {
        let f = vm.intern(k);
        vm.set_raw_field(obj, f, v);
    }
    world.resource_mut::<Zm>().ents.insert(
        n,
        Ent {
            obj: Some(obj),
            classname: sp.classname.clone(),
            origin: sp.origin,
            angles: sp.angles,
            solid: true,
            ..Default::default()
        },
    );
    world.resource_mut::<Zm>().actors.insert(
        n,
        Actor {
            obj: Some(obj),
            aitype: aitype.clone(),
            asd: String::new(),
            alive: true,
            health: 100,
            maxhealth: 100,
            team: "axis".into(),
            script: "init".into(),
            radius: 15.0,
            height: 72.0,
            orientmode: "face default".into(),
            animmode: "normal".into(),
            ..Default::default()
        },
    );
    let script = format!("aitype/{aitype}");
    vm.spawn_named(world, &script, "main", Value::Object(obj), vec![]);
    // The aitype names its animstatedef ("zm_nuked_basic.asd").
    let asd = field(vm, obj, "animstatedef");
    let asd = vm
        .to_text(&asd)
        .trim_end_matches(".asd")
        .to_ascii_lowercase();
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
        act.asd = asd;
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
    let kind = if aitype.ends_with("dog_mp") { "dog" } else { "zm" };
    vm.spawn_named(
        world,
        &format!("{ANIMSCRIPTS}/{kind}_init"),
=======
    let team = field(vm, obj, "team");
    vm.spawn_named(
        world,
        &format!("{ANIMSCRIPTS}/zm_init"),
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
        "main",
        Value::Object(obj),
        vec![],
    );
    // zm_init puts every actor on level.zombie_team; the aitype's own team
    // wins (Origins' giant robots are "neutral": on axis they counted as
    // round enemies and round 1 never ended).
    if let Value::Str(_) = team {
        let f = vm.intern("team");
        let text = vm.to_text(&team);
        vm.set_raw_field(obj, f, team);
        if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
            act.team = text;
        }
    }
    if let Some(act) = world.resource_mut::<Zm>().actors.get_mut(&n) {
        act.script.clear();
    }
    Some(obj)
}

/// Kill an actor: callbacks, `death`, `killanimscript`, the death animscript.
pub(crate) fn die(
    vm: &mut Vm<World>,
    world: &mut World,
    n: u32,
    attacker: Value,
    args: Vec<Value>,
) {
    let obj = {
        let mut zm = world.resource_mut::<Zm>();
        let now = zm.now_ms;
        let Some(a) = zm.actors.get_mut(&n) else {
            return;
        };
        if !a.alive {
            return;
        }
        a.alive = false;
        a.dead_since = Some(now);
        a.scripted = None;
        a.traverse = None;
        a.path.clear();
        a.obj
    };
    let Some(obj) = obj else { return };
    let cb = super::mp::callbacks(world);
    vm.spawn_named(
        world,
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
        &cb,
=======
        "maps/mp/gametypes_zm/_callbacksetup",
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
        "codecallback_actorkilled",
        Value::Object(obj),
        args,
    );
    // The killed callback may have removed it already (a zombie cleaned
    // up): nothing is left to die.
    if !vm.alive(obj) {
        return;
    }
    vm.notify_str(world, obj, "death", &[attacker]);
    run_script(vm, world, n, "death");
}

/// `CodeCallback_ActorDamage` for a hit on actor `n`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn damage(
    vm: &mut Vm<World>,
    world: &mut World,
    n: u32,
    inflictor: Value,
    attacker: Value,
    amount: i32,
    flags: i32,
    means: &str,
    weapon: &str,
    point: [f32; 3],
    dir: [f32; 3],
    hitloc: &str,
) {
    let Some(obj) = world
        .resource::<Zm>()
        .actors
        .get(&n)
        .filter(|a| a.alive)
        .and_then(|a| a.obj)
    else {
        return;
    };
    let means = vm.string(means);
    let weapon = vm.string(weapon);
    let hitloc = vm.string(hitloc);
    let args = vec![
        inflictor,
        attacker,
        Value::Int(amount),
        Value::Int(flags),
        means,
        weapon,
        Value::Vec3(point),
        Value::Vec3(dir),
        hitloc,
        Value::Int(0),
        Value::Int(0),
    ];
    let cb = super::mp::callbacks(world);
    vm.spawn_named(
        world,
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/actors.rs
        &cb,
=======
        "maps/mp/gametypes_zm/_callbacksetup",
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/actors.rs
        "codecallback_actordamage",
        Value::Object(obj),
        args,
    );
}

/// An actor's hit boxes this tick: its models (body and what is attached)
/// posed by the playing clip, each bone's box placed by `world_from_model`;
/// `None` without a skeleton.
pub(crate) fn hitboxes(
    world: &mut World,
    n: u32,
    now: i64,
    world_from_model: glam::Mat4,
) -> Option<Vec<xmodel_runtime::CollisionBone>> {
    use xmodel_runtime::{AnimInstance, Attach, DObj};
    let (model, attached) = {
        let zm = world.resource::<Zm>();
        let e = zm.ents.get(&n)?;
        (e.model.clone(), e.attached.clone())
    };
    let caps: Vec<(
        std::sync::Arc<xmodel_runtime::RetainedModelCapability>,
        Option<String>,
    )> = {
        let f = frame(world);
        let body = f.model_capability(&model).flatten()?;
        let mut v = vec![(body, None)];
        for (m, tag) in &attached {
            if let Some(c) = f.model_capability(m).flatten() {
                v.push((c, Some(tag.clone())));
            }
        }
        v
    };
    let models: Vec<(&xmodel_runtime::ModelPoseSrc, Option<Attach>)> = caps
        .iter()
        .map(|(c, tag)| {
            let attach = tag.as_ref().map(|t| Attach {
                parent_model: 0,
                tag: if t.is_empty() {
                    xmodel_runtime::empty_tag_attach(&caps[0].0.pose, &c.pose)
                } else {
                    t.clone()
                },
            });
            (&c.pose, attach)
        })
        .collect();
    let dobj = DObj::build(&models).ok()?;
    // The playing clip at its time (bind pose without one).
    let playing = {
        let zm = world.resource::<Zm>();
        let a = zm.actors.get(&n)?;
        a.playing.as_ref().and_then(|p| {
            let clip = zm.clips.get(&p.anim)?.clone();
            let elapsed = p.elapsed(now);
            let frac = if p.looping {
                (elapsed % p.length_ms) as f32 / p.length_ms as f32
            } else {
                (elapsed as f32 / p.length_ms as f32).min(1.0)
            };
            Some((clip, frac))
        })
    };
    let posed = match &playing {
        Some((clip, frac)) => {
            let tracks: Vec<Option<usize>> =
                clip.tracks.iter().map(|t| dobj.find(&t.name)).collect();
            let inst = AnimInstance {
                clip,
                tracks: &tracks,
                time: frac * clip.duration().max(1e-3),
                weight: 1.0,
                parts: None,
            };
            dobj.pose(&[inst], &dobj.all_parts(), world_from_model)
        }
        None => dobj.pose(&[], &dobj.all_parts(), world_from_model),
    };
    let mut out = Vec::new();
    for (slot, (cap, _)) in dobj.models.iter().zip(&caps) {
        for local in 0..slot.bone_count {
            let Some(Some(bc)) = cap.bone_collision.get(local) else {
                continue;
            };
            let Some(m) = posed.get(slot.base + local) else {
                continue;
            };
            let Some(mut b) = xmodel_runtime::collision_bone_from_local_box(
                u16::try_from(slot.base + local).ok()?,
                bc.midpoint,
                bc.half_size,
                *m,
            ) else {
                continue;
            };
            b.part_classification = bc.part_classification;
            out.push(b);
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Store the anim tree leaf (clip, time, rate) on an actor's model row.
pub(crate) fn present_anim(world: &World, n: u32, now: i64) -> Option<(String, f32, f32)> {
    let zm = world.resource::<Zm>();
    let a = zm.actors.get(&n)?;
    let p = a.playing.as_ref()?;
    let elapsed = p.elapsed(now);
    let frac = if p.looping {
        (elapsed % p.length_ms) as f32 / p.length_ms as f32
    } else {
        (elapsed as f32 / p.length_ms as f32).min(1.0)
    };
    let rate = if p.ended && !p.looping {
        0.0
    } else {
        1000.0 / p.length_ms as f32 * p.rate
    };
    Some((p.anim.clone(), frac, rate))
}

/// Show `anim` on actor `n` from its first frame, outside its
/// animstatedef and sending no notetracks (a dead zombie standing in for
/// BO2's client clone: Mob's wolf-head feeding); its length in ms.
pub(crate) fn show_clip(world: &mut World, n: u32, anim: &str, now: i64) -> i64 {
    let mut zm = world.resource_mut::<Zm>();
    let (length_ms, looping) = zm
        .anims
        .get(anim)
        .map_or((1000, false), |a| (anim_length_ms(a).max(1), a.looping));
    if let Some(a) = zm.actors.get_mut(&n) {
        a.playing = Some(Playing {
            anim: anim.to_owned(),
            start_ms: now,
            length_ms,
            looping,
            ended: true,
            rate: 1.0,
            ..Playing::default()
        });
    }
    length_ms
}

/// `setentityanimrate`: actor `n`'s clips (and the walking they drive)
/// play at `rate` from `now` on, the playing clip keeping its place.
pub(crate) fn set_anim_rate(world: &mut World, n: u32, rate: f32, now: i64) {
    let mut zm = world.resource_mut::<Zm>();
    let Some(a) = zm.actors.get_mut(&n) else {
        return;
    };
    let rate = rate.max(0.0);
    if std::env::var_os("IW4L_T6_ANIMLOG").is_some() && a.anim_rate.unwrap_or(1.0) != rate {
        diag::info!(Sim, "bo2zm t6 anim rate {n} t{now}: {rate}");
    }
    a.anim_rate = (rate != 1.0).then_some(rate);
    if let Some(p) = a.playing.as_mut() {
        p.base_ms = p.elapsed(now);
        p.start_ms = now;
        p.rate = rate;
    }
}

#[allow(dead_code)]
fn _types(_: Str, _: BTreeMap<u32, u32>, _: ClientId) {}
