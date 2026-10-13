//! Mob of the Dead's wolf heads (feeding the Cerberus soul catchers). BO2
//! plays them from a client script (zm_prison_weap_quest.csc) our engine
//! does not run: one world client field per `wolf_position` struct (named
//! by its `script_parameters`) says what the head does, and the client
//! shows, hides and animates the map's head (the struct's `script_label`),
//! its rune on the wall (`script_noteworthy`) and the body model it keeps
//! for the bite (`script_friendname`). The engine plays that half here:
//! 0 gone (the rune shows), 1 arriving (the portal opens, the intro),
//! 2 idle (the idle, then a twitch, over and over), 3 / 4 / 5 eating in
//! front / right / left (pre-eat while the body is pulled in, then the
//! bite), 6 leaving (the outro, the portal closes, the rune shows), 7 done
//! (the rune turns into the dream catcher and glows). The client also
//! takes the dead zombie the server marked `make_client_clone` (ghosted at
//! its `portal_death` notetrack) and pulls it into the head's mouth for the
//! bite: here that dead actor itself is shown again and moved.

use bevy_ecs::prelude::World;
use gsc_t6::{Value, Vm};

use super::Zm;
use crate::EventAudience;

const INTRO: &str = "o_zombie_dreamcatcher_intro";
const OUTRO: &str = "o_zombie_dreamcatcher_outtro";
const IDLE: &str = "o_zombie_dreamcatcher_idle";
const TWITCHES: [&str; 3] = [
    "o_zombie_dreamcatcher_idle_twitch_scan",
    "o_zombie_dreamcatcher_idle_twitch_shake",
    "o_zombie_dreamcatcher_idle_twitch_yawn",
];
const PORTAL_FX: &str = "maps/zombie_alcatraz/fx_alcatraz_portal_hell";
const PORTAL_CLOSE_FX: &str = "maps/zombie_alcatraz/fx_alcatraz_portal_hell_close";
const CHARGED_FX: &str = "maps/zombie_alcatraz/fx_alcatraz_soul_charged";
const DONE_MODEL: &str = "p6_zm_al_dream_catcher";
/// The client pulls the body in before the bite: 1.5 s, then 0.5 s (its
/// two `moveto`s); the head plays its pre-eat meanwhile.
const PULL_MS: i64 = 2000;
const RISE_MS: i64 = 1500;
/// The first move goes a fifth of the way to the head, the second to 36
/// in front of it.
const RISE_SHARE: f32 = 0.2;
const FRONT: f32 = 36.0;
const RISE: &str = "ai_zombie_dreamcatch_rise";
const SHRINK: &str = "ai_zombie_dreamcatch_shrink_a";
const SOUL_START_FX: &str = "maps/zombie_alcatraz/fx_alcatraz_soul_charge_start";
/// The bite's blood at the mouth, on the head clip's `blood` / `blood_sm`
/// notetracks.
const BITE_FX: &str = "maps/zombie_alcatraz/fx_alcatraz_soul_charge_impact";
const BITE_SMALL_FX: &str = "maps/zombie_alcatraz/fx_alcatraz_soul_charge_impact_sm";

/// The eat direction a field value names (3 front, 4 right, 5 left), as
/// the clip names' suffix.
fn side(value: i64) -> Option<&'static str> {
    match value {
        3 => Some("f"),
        4 => Some("r"),
        5 => Some("l"),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
enum Next {
    Idle,
    Twitch,
    Bite(&'static str),
}

#[derive(Clone, Debug)]
struct Head {
    field: String,
    /// The struct's place: the client picks a body's head by it.
    at: [f32; 3],
    head: Option<u32>,
    rune: Option<u32>,
    body: Option<u32>,
    value: i64,
    /// The head's next clip and when (the idle cycle, or the bite once the
    /// body is in).
    next: Option<(i64, Next)>,
}

/// A dead zombie being pulled into head `head` (index) since `start`.
#[derive(Clone, Debug)]
struct Pull {
    n: u32,
    head: usize,
    side: &'static str,
    start: i64,
    from: [f32; 3],
    mid: [f32; 3],
    to: [f32; 3],
    shrinking: bool,
    /// When the bite began and its length (ms).
    bite: Option<(i64, i64)>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Wolves {
    heads: Vec<Head>,
    seed: u64,
    /// Dead zombies marked `make_client_clone`, oldest first.
    clones: Vec<u32>,
    pulls: Vec<Pull>,
}

impl Wolves {
    fn rand(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.seed >> 33
    }
}

fn text_field(vm: &mut Vm<World>, o: gsc_t6::ObjRef, name: &str) -> String {
    let k = vm.intern(name);
    let v = vm.raw_field(o, k);
    vm.to_text(&v)
}

/// The map entity named `targetname`.
fn map_ent(vm: &mut Vm<World>, world: &World, targetname: &str) -> Option<u32> {
    if targetname.is_empty() {
        return None;
    }
    let k = vm.intern("targetname");
    world.resource::<Zm>().ents.iter().find_map(|(n, e)| {
        let o = e.obj?;
        (e.map && vm.to_text(&vm.raw_field(o, k)) == targetname).then_some(*n)
    })
}

/// The heads from `level.a_wolf_structs` (set by the server's weapon quest
/// before it sets their fields).
fn find(vm: &mut Vm<World>, world: &mut World) {
    let lvl = vm.level;
    let k = vm.intern("a_wolf_structs");
    let Value::Array(a) = vm.raw_field(lvl, k) else {
        return;
    };
    let mut heads = Vec::new();
    for i in 0..8 {
        let Some(Value::Object(s)) = a.get(&gsc_t6::Key::Int(i)) else {
            break;
        };
        let field = text_field(vm, s, "script_parameters");
        let ko = vm.intern("origin");
        let at = match vm.raw_field(s, ko) {
            Value::Vec3(v) => v,
            _ => [0.0; 3],
        };
        let head = text_field(vm, s, "script_label");
        let rune = text_field(vm, s, "script_noteworthy");
        let body = text_field(vm, s, "script_friendname");
        let (head, rune, body) = (
            map_ent(vm, world, &head),
            map_ent(vm, world, &rune),
            map_ent(vm, world, &body),
        );
        diag::info!(
            Sim,
            "bo2zm t6 wolf head {field}: head {head:?} rune {rune:?} body {body:?} at ({:.0} {:.0} {:.0})",
            at[0],
            at[1],
            at[2]
        );
        heads.push(Head {
            field,
            at,
            head,
            rune,
            body,
            value: -1,
            next: None,
        });
    }
    let mut zm = world.resource_mut::<Zm>();
    zm.wolves.heads = heads;
    zm.wolves.seed = 0x0d06_f00d;
}

fn show(world: &mut World, n: Option<u32>, on: bool) {
    let mut zm = world.resource_mut::<Zm>();
    if let Some(e) = n.and_then(|n| zm.ents.get_mut(&n)) {
        e.hidden = !on;
        e.ghosted = false;
    }
}

/// Start `clip` on the head from its first frame (`clearanim` + `setanim`);
/// its length in ms.
fn play(world: &mut World, n: Option<u32>, clip: &str, now: i64) -> i64 {
    let mut zm = world.resource_mut::<Zm>();
    let len = zm.anims.get(clip).map(super::actors::anim_length_ms);
    if len.is_none() {
        diag::info!(Sim, "bo2zm t6 wolf head: animation {clip} not loaded");
    }
    if let Some(e) = n.and_then(|n| zm.ents.get_mut(&n)) {
        e.anim = Some((clip.to_owned(), now));
    }
    len.unwrap_or(1000).max(1)
}

fn origin(world: &World, n: Option<u32>) -> Option<[f32; 3]> {
    n.and_then(|n| world.resource::<Zm>().ents.get(&n).map(|e| e.origin))
}

/// Where the client puts the portal: just off the rune and 24 below it,
/// facing out of the wall (the rune's forward turned 90 degrees).
fn portal(world: &World, rune: Option<u32>) -> Option<([f32; 3], [f32; 3], [f32; 3])> {
    let e = world.resource::<Zm>().ents.get(&rune?)?.clone();
    let (fwd, _, _) = math_iw4::angle_vectors([e.angles[0], e.angles[1] + 90.0, e.angles[2]]);
    let (_, _, up) = math_iw4::angle_vectors(e.angles);
    let at = std::array::from_fn(|i| e.origin[i] - fwd[i] * 2.5 - up[i] * 24.0);
    Some((at, fwd, up))
}

fn sound(world: &mut World, n: Option<u32>, alias: &str) {
    if let Some(at) = origin(world, n) {
        super::natives_fx::sound(world, EventAudience::All, alias, at);
    }
}

fn loop_sound(world: &mut World, n: Option<u32>, alias: Option<&str>) {
    let mut zm = world.resource_mut::<Zm>();
    if let Some(e) = n.and_then(|n| zm.ents.get_mut(&n)) {
        e.loop_sound = alias.map(str::to_owned);
    }
}

/// `make_client_clone` set on a zombie dying in a head's area: its body is
/// the one that head eats next.
pub(super) fn clone_body(world: &mut World, n: u32) {
    let mut zm = world.resource_mut::<Zm>();
    diag::info!(Sim, "bo2zm t6 wolf head: body {n} marked t{}", zm.now_ms);
    // Entity numbers come back: keep only bodies still lying dead.
    let Zm { wolves, actors, .. } = &mut *zm;
    wolves
        .clones
        .retain(|c| *c != n && actors.get(c).is_some_and(|a| !a.alive));
    wolves.clones.push(n);
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

fn dist2(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]) * (a[i] - b[i])).sum()
}

/// Head `i` starts eating: the newest marked body nearest that head (as
/// the client picks it) is shown again and pulled in.
fn start_pull(world: &mut World, i: usize, side: &'static str, now: i64) {
    let (heads, clones) = {
        let zm = world.resource::<Zm>();
        (zm.wolves.heads.clone(), zm.wolves.clones.clone())
    };
    let nearest_head = |at: [f32; 3]| {
        (0..heads.len()).min_by(|a, b| dist2(heads[*a].at, at).total_cmp(&dist2(heads[*b].at, at)))
    };
    // Only a dead one: its number may since belong to a new zombie.
    let pick = clones.iter().rev().copied().find(|n| {
        let zm = world.resource::<Zm>();
        zm.actors.get(n).is_some_and(|a| !a.alive)
            && zm
                .ents
                .get(n)
                .is_some_and(|e| nearest_head(e.origin) == Some(i))
    });
    let Some(n) = pick else {
        diag::info!(
            Sim,
            "bo2zm t6 wolf head {}: no marked body to pull",
            heads[i].field
        );
        return;
    };
    let (Some(from), Some(head)) = (origin(world, Some(n)), heads[i].head) else {
        return;
    };
    let Some(e) = world.resource::<Zm>().ents.get(&head).cloned() else {
        return;
    };
    let (fwd, _, _) = math_iw4::angle_vectors(e.angles);
    let mid = lerp(from, e.origin, RISE_SHARE);
    let to = std::array::from_fn(|k| e.origin[k] + fwd[k] * FRONT);
    // What it played before the ghost: the server's `zm_portal_death`,
    // waited on to its `end` (`donotetracks("portal_death")`); a short
    // play here means the script went on early (task check, local log).
    let before = world
        .resource::<Zm>()
        .actors
        .get(&n)
        .and_then(|a| a.playing.clone());
    if let Some(p) = before {
        diag::info!(
            Sim,
            "bo2zm t6 wolf head {}: body {n} played {} {} for {} of {} ms",
            heads[i].field,
            p.state,
            p.anim,
            p.elapsed(now),
            p.length_ms
        );
    }
    show(world, Some(n), true);
    super::actors::show_clip(world, n, RISE, now);
    sound(world, Some(n), "evt_soulsuck_body");
    super::clientfields::bolt_later(world, n, 0, SOUL_START_FX, "j_spinelower");
    diag::info!(
        Sim,
        "bo2zm t6 wolf head {}: pulling body {n} from ({:.0} {:.0} {:.0})",
        heads[i].field,
        from[0],
        from[1],
        from[2]
    );
    let mut zm = world.resource_mut::<Zm>();
    zm.wolves.clones.retain(|c| *c != n);
    zm.wolves.pulls.push(Pull {
        n,
        head: i,
        side,
        start: now,
        from,
        mid,
        to,
        shrinking: false,
        bite: None,
    });
}

/// Each tick: the pulled bodies move in (each `moveto` speeds up all the
/// way, its accel time being its whole time), then ride the head's mouth
/// through the bite and are hidden at its end (the server deletes them).
fn advance_pulls(world: &mut World, now: i64) {
    let pulls = std::mem::take(&mut world.resource_mut::<Zm>().wolves.pulls);
    let mut keep = Vec::new();
    for mut p in pulls {
        let dead = world
            .resource::<Zm>()
            .actors
            .get(&p.n)
            .is_some_and(|a| !a.alive);
        if !dead || !world.resource::<Zm>().ents.contains_key(&p.n) {
            continue;
        }
        let t = now - p.start;
        let head = world
            .resource::<Zm>()
            .wolves
            .heads
            .get(p.head)
            .and_then(|h| h.head);
        if t < RISE_MS {
            let k = t as f32 / RISE_MS as f32;
            set_origin(world, p.n, lerp(p.from, p.mid, k * k));
        } else if t < PULL_MS {
            if !p.shrinking {
                p.shrinking = true;
                super::actors::show_clip(world, p.n, SHRINK, now);
            }
            let k = (t - RISE_MS) as f32 / (PULL_MS - RISE_MS) as f32;
            set_origin(world, p.n, lerp(p.mid, p.to, k * k));
        } else {
            if p.bite.is_none() {
                super::natives_fx::bolt_effect(world, p.n, SOUL_START_FX, "j_spinelower", true);
                let clip = format!("ai_zombie_dreamcatcher_wallconsume_align_{}", p.side);
                let len = super::actors::show_clip(world, p.n, &clip, now);
                p.bite = Some((now, len));
            }
            // Linked to the head's `tag_mouth_fx` for the bite.
            let mouth = head.and_then(|h| super::tags::tag_world(world, h, "tag_mouth_fx"));
            match mouth {
                Some((at, axes)) => {
                    set_origin(world, p.n, at);
                    if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&p.n) {
                        e.angles = super::tags::axis_to_angles(axes);
                    }
                }
                None => set_origin(world, p.n, p.to),
            }
            if let Some((since, len)) = p.bite
                && now - since >= len
            {
                show(world, Some(p.n), false);
                continue;
            }
        }
        keep.push(p);
    }
    world.resource_mut::<Zm>().wolves.pulls.extend(keep);
}

fn set_origin(world: &mut World, n: u32, at: [f32; 3]) {
    if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n) {
        e.origin = at;
    }
}

/// `codesetworldclientfield` / `level setclientfield`: a wolf head's field
/// changed (other fields are left alone).
pub(super) fn world_field(vm: &mut Vm<World>, world: &mut World, name: &str, value: i64) {
    if world.resource::<Zm>().map != "zm_prison" {
        return;
    }
    if world.resource::<Zm>().wolves.heads.is_empty() {
        find(vm, world);
    }
    let Some(i) = world
        .resource::<Zm>()
        .wolves
        .heads
        .iter()
        .position(|h| h.field == name)
    else {
        return;
    };
    let now = world.resource::<Zm>().now_ms;
    let h = world.resource::<Zm>().wolves.heads[i].clone();
    let old = h.value;
    diag::info!(Sim, "bo2zm t6 wolf head {name}: {old} -> {value} t{now}");
    show(world, h.head, !matches!(value, 0 | 7));
    show(world, h.rune, matches!(value, 0 | 6 | 7));
    show(world, h.body, false);
    let mut next = None;
    match value {
        1 => {
            if let Some((at, fwd, up)) = portal(world, h.rune) {
                super::natives_fx::effect_held(world, PORTAL_FX, at, fwd, Some(up));
            }
            sound(world, h.head, "evt_wolfhead_spawn");
            loop_sound(world, h.head, Some("evt_wolfhead_fire_loop"));
            play(world, h.head, INTRO, now);
        }
        2 => {
            if (3..=5).contains(&old) {
                sound(world, h.head, "evt_wolfhead_body_count");
            }
            let len = play(world, h.head, IDLE, now);
            next = Some((now + len, Next::Twitch));
        }
        3..=5 => {
            let s = side(value).unwrap_or("f");
            play(
                world,
                h.head,
                &format!("o_zombie_dreamcatcher_wallconsume_pre_eat_{s}"),
                now,
            );
            next = Some((now + PULL_MS, Next::Bite(s)));
            start_pull(world, i, s, now);
        }
        6 => {
            play(world, h.head, OUTRO, now);
            if let Some((at, fwd, up)) = portal(world, h.rune) {
                super::natives_fx::effect_stop_world(world, PORTAL_FX, at);
                super::natives_fx::effect_axis(world, PORTAL_CLOSE_FX, at, fwd, up);
            }
            sound(world, h.head, "evt_wolfhead_depart");
            loop_sound(world, h.head, None);
            loop_sound(world, h.rune, None);
        }
        7 => {
            if let Some(n) = h.rune
                && let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n)
            {
                e.model = DONE_MODEL.to_owned();
            }
            if let Some(n) = h.rune {
                super::clientfields::bolt_later(world, n, 0, CHARGED_FX, "tag_origin");
            }
            loop_sound(world, h.rune, Some("evt_runeglow_loop"));
        }
        _ => {}
    }
    let mut zm = world.resource_mut::<Zm>();
    let h = &mut zm.wolves.heads[i];
    h.value = value;
    h.next = next;
}

/// Each tick: the heads' idle cycle and their bites once the body is in.
pub(super) fn advance(world: &mut World, now: i64) {
    advance_pulls(world, now);
    let due: Vec<(usize, Next, Option<u32>)> = {
        let zm = world.resource::<Zm>();
        zm.wolves
            .heads
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                h.next
                    .filter(|(t, _)| now >= *t)
                    .map(|(_, n)| (i, n, h.head))
            })
            .collect()
    };
    for (i, step, head) in due {
        let next = match step {
            Next::Idle => {
                let len = play(world, head, IDLE, now);
                Some((now + len, Next::Twitch))
            }
            Next::Twitch => {
                let pick = (world.resource_mut::<Zm>().wolves.rand() % 3) as usize;
                let len = play(world, head, TWITCHES[pick], now);
                Some((now + len, Next::Idle))
            }
            Next::Bite(s) => {
                sound(world, head, "evt_wolfhead_eat");
                let clip = format!("o_zombie_dreamcatcher_wallconsume_align_{s}");
                let len = play(world, head, &clip, now);
                let notes = world
                    .resource::<Zm>()
                    .anims
                    .get(&clip)
                    .map(|a| a.notifies.clone())
                    .unwrap_or_default();
                for (note, at) in notes {
                    let fx = match note.as_str() {
                        "blood" => BITE_FX,
                        "blood_sm" => BITE_SMALL_FX,
                        _ => continue,
                    };
                    if let Some(n) = head {
                        let after = (at * len as f32) as i64;
                        super::clientfields::bolt_later(world, n, after, fx, "tag_mouth_fx");
                    }
                }
                None
            }
        };
        world.resource_mut::<Zm>().wolves.heads[i].next = next;
    }
}
