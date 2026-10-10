//! bo2mp: the engine half of Black Ops II's bots.
//!
//! The game's own bot scripts (`maps/mp/bots/_bot*`) decide everything a
//! bot does: where to go (named goals with priorities: "wander",
//! "enemy_patrol", "cover"...), whom to fight (`getthreats`), where to look,
//! when the trigger may be pulled, when to aim down the sights, throw,
//! reload, melee or dive. The engine carries it out: a test client joins
//! (`addtestclient`), walks the best goal's route over the map's path
//! nodes, turns its view toward what it looks at (or down its route) at its
//! difficulty's speed, and presses the buttons. Each tick a bot's command
//! replaces its client's own, as the zombies test player does.

use std::collections::BTreeMap;

use bevy_ecs::prelude::World;
use gsc_t6::{Array, ObjKind, Value, Vm};
use playerstate_iw4::buttons;

use super::{Player, Zm, arg, entnum, frame, list, origin_of, text};
use crate::world::ClientId;

/// A named place a bot wants to be (`addgoal`): the highest priority wins,
/// the newest among equals.
#[derive(Clone, Debug)]
struct Goal {
    name: String,
    pos: [f32; 3],
    radius: f32,
    priority: i32,
    order: u64,
}

/// What a bot's scripts set up for its custom classes (`botclassadditem`,
/// `botclassaddattachment`, `botsetdefaultclass`): read by the loadout.
#[derive(Clone, Debug, Default)]
pub(crate) struct BotClass {
    pub default: Option<String>,
    pub items: Vec<String>,
    /// (weapon, attachment, slot such as "primaryattachment1").
    pub attachments: Vec<(String, String, String)>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Bot {
    goals: Vec<Goal>,
    order: u64,
    /// Route to the best goal: points, the goal last.
    path: Vec<[f32; 3]>,
    path_to: Option<[f32; 3]>,
    replan_at: i64,
    look: Option<[f32; 3]>,
    allow_attack: bool,
    ads: bool,
    attack_ticks: u32,
    use_until: i64,
    melee: bool,
    dive: bool,
    /// 0 stand, 1 crouch, 2 prone (`setstance`).
    stance: u8,
    no_sprint: bool,
    failsafe: Option<[f32; 3]>,
    tick: u32,
    /// Where it was when it last made headway, and when (stuck checks).
    progress: Option<([f32; 3], i64)>,
    /// The point its route came from to `path[0]` (where it planned, or
    /// the last point it passed): it has passed `path[0]` once it is
    /// beyond the line across that point.
    prev: Option<[f32; 3]>,
    /// Counts new routes; with the route's length it names the step a
    /// cached far-sight answer belongs to.
    path_gen: u32,
    path_new: bool,
    /// The farthest route point seen from the one it heads to, for
    /// (route, points left, within 192 units of the next).
    seen: Option<(u32, usize, bool, [f32; 3])>,
    /// BO2's steering: the angles its view chases (pitch, yaw), smoothed
    /// toward what it wants to face, and the current turn speeds
    /// (degrees a second) that ramp up toward it.
    aim_avg: Option<[f32; 2]>,
    turn_speed: [f32; 2],
    /// A semi-automatic or burst weapon's trigger rests until then.
    fire_delay_until: i64,
    /// When its current sprint began.
    sprint_since: Option<i64>,
    pub classes: BTreeMap<i32, BotClass>,
}

impl Bot {
    /// The level restarts (`map_restart`): what its scripts set up for its
    /// classes stays; where it was going and what it was doing go.
    pub(crate) fn restarted(self) -> Self {
        Self {
            classes: self.classes,
            ..Self::default()
        }
    }

    fn best_goal(&self) -> Option<&Goal> {
        self.goals
            .iter()
            .max_by_key(|g| (g.priority, g.order))
    }
}

fn dist2d(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Yaw and pitch (degrees, the game's: pitch down positive) from a to b.
fn angles_to(a: [f32; 3], b: [f32; 3]) -> (f32, f32) {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let yaw = d[1].atan2(d[0]).to_degrees();
    let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let pitch = -d[2].atan2(flat).to_degrees();
    (yaw, pitch)
}

fn angle_delta(from: f32, to: f32) -> f32 {
    (to - from + 540.0).rem_euclid(360.0) - 180.0
}

/// Weapon states (the movement code's numbering): reloading, any kind.
fn reloading(state: i32) -> bool {
    use weapon_iw4::WeaponState as S;
    (S::Reloading as i32..=S::ReloadEnd as i32).contains(&state)
}

/// The settings BO2's engine drives bots by (its own dvars; the bot
/// scripts raise `bot_yawspeed` and set the fire gaps per difficulty).
struct BotDvars {
    yaw_speed: f32,
    min_fire: i64,
    max_fire: i64,
    sprint_distance: f32,
    /// aim_turnrate_pitch / _ads, aim_turnrate_yaw / _ads (degrees a
    /// second) and aim_accel_turnrate_lerp (degrees a second, a second).
    pitch_rate: [f32; 2],
    yaw_rate: [f32; 2],
    accel: f32,
}

impl BotDvars {
    fn read(world: &mut World) -> Self {
        super::with_vm(world, |vm, _| {
            let f = |k: &str, d: f32| {
                vm.dvars
                    .get(k)
                    .and_then(|s| s.trim().parse::<f32>().ok())
                    .unwrap_or(d)
            };
            let (min_fire, max_fire) = (f("bot_minfiretime", 400.0) as i64, f("bot_maxfiretime", 600.0) as i64);
            Self {
                yaw_speed: f("bot_yawspeed", 1.0),
                min_fire,
                max_fire: max_fire.max(min_fire),
                sprint_distance: f("bot_sprintdistance", 256.0),
                pitch_rate: [f("aim_turnrate_pitch", 90.0), f("aim_turnrate_pitch_ads", 55.0)],
                yaw_rate: [f("aim_turnrate_yaw", 260.0), f("aim_turnrate_yaw_ads", 90.0)],
                accel: f("aim_accel_turnrate_lerp", 1200.0),
            }
        })
        .unwrap_or(Self {
            yaw_speed: 1.0,
            min_fire: 400,
            max_fire: 600,
            sprint_distance: 256.0,
            pitch_rate: [90.0, 55.0],
            yaw_rate: [260.0, 90.0],
            accel: 1200.0,
        })
    }
}

/// A number in lo..hi from the bot and the time (splitmix).
fn irand(n: u32, now: i64, lo: i64, hi: i64) -> i64 {
    if hi <= lo {
        return lo;
    }
    let mut z = (now as u64) ^ ((n as u64) << 40) ^ 0x9E37_79B9_7F4A_7C15;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    lo + (z % (hi - lo) as u64) as i64
}

/// BO2's lookahead down a route: along the route from where the bot is
/// until the corner it would cut (each leg's length times how far its line
/// passes from the bot, summed) reaches 32768; straight legs cost nothing.
/// The point there, and how far it is.
fn lookahead(origin: [f32; 3], path: &[[f32; 3]]) -> Option<([f32; 3], f32)> {
    const AMOUNT: f32 = 32768.0;
    let mut area = 0.0f32;
    let mut a = origin;
    for &b in path {
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        if len < 1e-3 {
            a = b;
            continue;
        }
        let dir = [d[0] / len, d[1] / len];
        let off = (dir[0] * (b[1] - origin[1]) - dir[1] * (b[0] - origin[0])).abs();
        let before = area;
        area += len * off;
        if off > 0.0 && area >= AMOUNT {
            let s = ((AMOUNT - before) / off).min(len) / len;
            let p = [a[0] + d[0] * s, a[1] + d[1] * s, a[2] + (b[2] - a[2]) * s];
            return Some((p, dist2d(origin, p)));
        }
        a = b;
    }
    path.last().map(|&p| (p, dist2d(origin, p)))
}

/// BO2's far sight down a route: from the point it heads to (the one
/// after when that is within 192 units), the farthest later point seen
/// from there; it looks toward that while it walks.
fn farthest_seen(world: &mut World, origin: [f32; 3], path: &[[f32; 3]]) -> Option<[f32; 3]> {
    let start = usize::from(path.len() > 1 && dist2d(origin, *path.first()?) < 192.0);
    let raise = |p: [f32; 3]| [p[0], p[1], p[2] + 60.0];
    let from = raise(path[start]);
    let last = (path.len() - 1).min(start + 12);
    for &p in path[start + 1..=last].iter().rev() {
        if dist2d(path[start], p) < 2048.0 && sight(world, from, raise(p)) {
            return Some(p);
        }
    }
    Some(path[start])
}

/// BO2's bot steering, one 50 ms tick: the view (pitch, yaw) turns toward
/// the angles it chases at a speed that scales with how far off it is
/// (a quarter second to close the gap from the hip, a fifth aimed in), up
/// to the turn rates, ramping up by the acceleration and dropping at once.
fn steer(view: [f32; 2], aim: [f32; 2], speed: &mut [f32; 2], ads: f32, dv: &BotDvars) -> [f32; 2] {
    let lerp = |r: [f32; 2]| r[0] + (r[1] - r[0]) * ads;
    let time = 0.2 * ads + 0.25 * (1.0 - ads);
    let rate = [lerp(dv.pitch_rate), lerp(dv.yaw_rate)];
    let mut axis: [f32; 2] =
        std::array::from_fn(|i| (angle_delta(view[i], aim[i]) / (time * rate[i]).max(1e-3)).clamp(-1.0, 1.0));
    // The smaller axis gives way to the larger (aim_scale_view_axis).
    let (p, y) = (axis[0].abs(), axis[1].abs());
    if y < p {
        axis[1] *= 1.0 - (p - y);
    } else {
        axis[0] *= 1.0 - (y - p);
    }
    let mut out = view;
    for i in 0..2 {
        let target = axis[i].abs() * rate[i] * dv.yaw_speed;
        speed[i] = if target <= speed[i] {
            target
        } else {
            (speed[i] + dv.accel * dv.yaw_speed * 0.05).min(target)
        };
        out[i] = view[i] + speed[i] * 0.05 * if axis[i] >= 0.0 { 1.0 } else { -1.0 };
    }
    out
}

/// BO2's crowd avoidance: each bot keeps a circle (30 units; a player's is
/// 50) and picks the velocity nearest the way it wants to go that misses
/// its teammates for the next two seconds (reciprocal velocity obstacles).
const AVOID_HORIZON: f32 = 2.0;
const BOT_AVOID_RADIUS: f32 = 30.0;
const BOT_AVOID_SPEED: f32 = 100.0;
const PLAYER_AVOID_RADIUS: f32 = 50.0;
/// Clip brushes the avoidance check traces against (AI clip).
const AVOID_CLIP: u32 = 0x0002_0000;

/// A player in the crowd, as the bots avoid it.
struct Avoider {
    id: u32,
    pos: [f32; 3],
    vel: [f32; 2],
    radius: f32,
    /// A player (not a bot): it does not dodge, so a bot takes all of it.
    firm: bool,
    team: String,
}

/// Everyone in play this tick, for the bots' avoidance; empty without
/// teams (free-for-all bots do not avoid).
fn crowd(world: &mut World) -> Vec<Avoider> {
    let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    let mut out = Vec::new();
    for id in ids {
        if !alive(world, id) {
            continue;
        }
        let Some((pos, vel)) = frame(world)
            .player(ClientId(id))
            .map(|ps| (ps.origin, [ps.velocity[0], ps.velocity[1]]))
        else {
            continue;
        };
        let bot = world.resource::<Zm>().bots.contains_key(&id);
        let radius = if bot { BOT_AVOID_RADIUS } else { PLAYER_AVOID_RADIUS };
        out.push(Avoider { id, pos, vel, radius, firm: !bot, team: String::new() });
    }
    let teams = super::with_vm(world, |vm, world| {
        teambased(vm).then(|| out.iter().map(|a| team_of(vm, world, a.id)).collect::<Vec<_>>())
    })
    .flatten();
    match teams {
        Some(teams) => {
            for (a, t) in out.iter_mut().zip(teams) {
                a.team = t;
            }
            out
        }
        None => Vec::new(),
    }
}

fn det2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

fn dot2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// The velocity nearest `pref` (no faster than `max_speed`) that keeps
/// `me` clear of `others` over the horizon: each neighbour gives a
/// half-plane of allowed velocities (half the dodge each between bots, all
/// of it around a player), met one at a time; when one cannot be met it
/// keeps what it had.
fn avoid_velocity(me: &Avoider, pref: [f32; 2], max_speed: f32, others: &[&Avoider]) -> [f32; 2] {
    let inv_t = 1.0 / AVOID_HORIZON;
    let inv_tick = 1000.0 / crate::MATCH_TICK_MS as f32;
    // Each half-plane as (a point on its edge, the edge's direction); the
    // allowed side is to the left.
    let mut lines: Vec<([f32; 2], [f32; 2])> = Vec::with_capacity(others.len());
    for o in others {
        let rel_pos = [o.pos[0] - me.pos[0], o.pos[1] - me.pos[1]];
        let rel_vel = [me.vel[0] - o.vel[0], me.vel[1] - o.vel[1]];
        let dist_sq = dot2(rel_pos, rel_pos);
        let r = me.radius + o.radius;
        let share = if o.firm { 1.0 } else { 0.5 };
        let (dir, u);
        // Off the edge nearest the velocity: the cut-off circle at the
        // horizon, either side cone leg, or (already touching) the circle
        // one tick ahead.
        let toward = |w: [f32; 2], k: f32| {
            let len = dot2(w, w).sqrt().max(1e-6);
            let unit = [w[0] / len, w[1] / len];
            ([unit[1], -unit[0]], [(k - len) * unit[0], (k - len) * unit[1]])
        };
        if dist_sq > r * r {
            let w = [rel_vel[0] - inv_t * rel_pos[0], rel_vel[1] - inv_t * rel_pos[1]];
            let along = dot2(w, rel_pos);
            if along < 0.0 && along * along > r * r * dot2(w, w) {
                (dir, u) = toward(w, r * inv_t);
            } else {
                let leg = (dist_sq - r * r).sqrt();
                dir = if det2(rel_pos, w) > 0.0 {
                    [(rel_pos[0] * leg - rel_pos[1] * r) / dist_sq, (rel_pos[0] * r + rel_pos[1] * leg) / dist_sq]
                } else {
                    [-(rel_pos[0] * leg + rel_pos[1] * r) / dist_sq, -(-rel_pos[0] * r + rel_pos[1] * leg) / dist_sq]
                };
                let d = dot2(rel_vel, dir);
                u = [d * dir[0] - rel_vel[0], d * dir[1] - rel_vel[1]];
            }
        } else {
            let w = [rel_vel[0] - inv_tick * rel_pos[0], rel_vel[1] - inv_tick * rel_pos[1]];
            (dir, u) = toward(w, r * inv_tick);
        }
        lines.push(([me.vel[0] + share * u[0], me.vel[1] + share * u[1]], dir));
    }
    let len = dot2(pref, pref).sqrt();
    let mut out = if len > max_speed { [pref[0] / len * max_speed, pref[1] / len * max_speed] } else { pref };
    for i in 0..lines.len() {
        let (p, d) = lines[i];
        if det2(d, [p[0] - out[0], p[1] - out[1]]) > 0.0 {
            match on_line(&lines[..=i], max_speed, pref) {
                Some(v) => out = v,
                None => break,
            }
        }
    }
    out
}

/// The point on the last line nearest `opt` inside the speed circle and the
/// earlier half-planes, if any.
fn on_line(lines: &[([f32; 2], [f32; 2])], radius: f32, opt: [f32; 2]) -> Option<[f32; 2]> {
    let (&(p, d), rest) = lines.split_last()?;
    let dp = dot2(p, d);
    let disc = dp * dp + radius * radius - dot2(p, p);
    if disc < 0.0 {
        return None;
    }
    let s = disc.sqrt();
    let (mut lo, mut hi) = (-dp - s, -dp + s);
    for &(q, e) in rest {
        let den = det2(d, e);
        let num = det2(e, [p[0] - q[0], p[1] - q[1]]);
        if den.abs() <= 1e-3 {
            if num < 0.0 {
                return None;
            }
            continue;
        }
        let t = num / den;
        if den >= 0.0 {
            hi = hi.min(t);
        } else {
            lo = lo.max(t);
        }
        if lo > hi {
            return None;
        }
    }
    let t = dot2(d, [opt[0] - p[0], opt[1] - p[1]]).max(lo).min(hi);
    Some([p[0] + t * d[0], p[1] + t * d[1]])
}

/// BO2's check on a dodge: a thin line at knee height toward the bent
/// lookahead, failing inside a clip brush or on one within 15 units of the
/// bot (measured where the line halves).
fn avoid_clear(world: &mut World, origin: [f32; 3], to: [f32; 3]) -> bool {
    let a = [origin[0], origin[1], origin[2] + 18.0];
    let b = [to[0], to[1], to[2] + 18.0];
    let t = frame(world).trace_static_world(a, b, [0.0; 3], [0.0; 3], AVOID_CLIP);
    if t.startsolid != 0 || t.allsolid != 0 {
        return false;
    }
    if t.fraction < 1.0 {
        let h = t.fraction * 0.5;
        let m = [a[0] + (b[0] - a[0]) * h, a[1] + (b[1] - a[1]) * h];
        let d = [origin[0] - m[0], origin[1] - m[1]];
        return dot2(d, d) >= 225.0;
    }
    true
}

/// The eye of a player: his origin plus his view height.
fn eye(world: &mut World, n: u32) -> Option<[f32; 3]> {
    frame(world)
        .player(ClientId(n))
        .map(|ps| [ps.origin[0], ps.origin[1], ps.origin[2] + ps.view_height_current])
}

/// A player in play: on a team, playing, with health.
fn alive(world: &mut World, n: u32) -> bool {
    let playing = world
        .resource::<Zm>()
        .players
        .get(&n)
        .is_some_and(|p| p.sessionstate == "playing");
    playing && frame(world).player(ClientId(n)).is_some_and(|ps| ps.health > 0)
}

fn team_of(vm: &mut Vm<World>, world: &World, n: u32) -> String {
    let Some(p) = world.resource::<Zm>().players.get(&n) else {
        return String::new();
    };
    let f = vm.intern("team");
    let v = vm.raw_field(p.obj, f);
    vm.to_text(&v)
}

/// Is the way toward `to` blocked at step height (a player's box raised by
/// the walk's step, 24 units on)?
fn blocked_ahead(world: &mut World, origin: [f32; 3], to: [f32; 3]) -> bool {
    use crate::bullet_collision::{MASK_PLAYER_SOLID, PLAYER_MAXS, PLAYER_MINS};
    let d = [to[0] - origin[0], to[1] - origin[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if len < 1.0 {
        return false;
    }
    let start = [origin[0], origin[1], origin[2] + 18.0];
    let end = [start[0] + d[0] / len * 24.0, start[1] + d[1] / len * 24.0, start[2]];
    let maxs = [PLAYER_MAXS[0], PLAYER_MAXS[1], PLAYER_MAXS[2] - 18.0];
    let t = frame(world).trace_clip(start, end, PLAYER_MINS, maxs, MASK_PLAYER_SOLID);
    t.startsolid != 0 || t.fraction < 1.0
}

fn sight(world: &mut World, from: [f32; 3], to: [f32; 3]) -> bool {
    let t = frame(world).trace_static_world(
        from,
        to,
        [0.0; 3],
        [0.0; 3],
        crate::bullet_collision::MASK_SHOT,
    );
    t.fraction >= 1.0
}

/// Players on other teams (or every other player without teams), in play.
/// Is this a team game (`level.teambased`)? Free-for-all players all
/// stand on one team name and are each other's enemies.
fn teambased(vm: &mut Vm<World>) -> bool {
    let f = vm.intern("teambased");
    gsc_t6::truthy(&vm.raw_field(vm.level, f))
}

fn enemies_of(vm: &mut Vm<World>, world: &mut World, n: u32, alive_only: bool) -> Vec<u32> {
    let mine = team_of(vm, world, n);
    let teambased = teambased(vm) && mine != "free" && !mine.is_empty();
    let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    let mut out = Vec::new();
    for id in ids {
        if id == n {
            continue;
        }
        let theirs = team_of(vm, world, id);
        if theirs == "spectator" || theirs.is_empty() {
            continue;
        }
        if teambased && theirs == mine {
            continue;
        }
        if alive_only && !alive(world, id) {
            continue;
        }
        out.push(id);
    }
    out
}

fn friends_of(vm: &mut Vm<World>, world: &mut World, n: u32, alive_only: bool) -> Vec<u32> {
    if !teambased(vm) {
        return Vec::new();
    }
    let mine = team_of(vm, world, n);
    let ids: Vec<u32> = world.resource::<Zm>().players.keys().copied().collect();
    let mut out = Vec::new();
    for id in ids {
        if id == n || team_of(vm, world, id) != mine {
            continue;
        }
        if alive_only && !alive(world, id) {
            continue;
        }
        out.push(id);
    }
    out
}

fn objs(world: &World, ids: &[u32]) -> Vec<Value> {
    let zm = world.resource::<Zm>();
    ids.iter()
        .filter_map(|id| zm.players.get(id).map(|p| Value::Object(p.obj)))
        .collect()
}

/// A target's place: a point, an entity, a path node (struct origin).
fn place(vm: &mut Vm<World>, world: &mut World, v: &Value) -> Option<[f32; 3]> {
    if let Some(p) = v.as_vec3() {
        return Some(p);
    }
    if let Some(p) = origin_of(vm, world, v) {
        return Some(p);
    }
    if let Value::Object(o) = v {
        let f = vm.intern("origin");
        return vm.raw_field(*o, f).as_vec3();
    }
    None
}

fn bot_mut<'w>(world: &'w mut World, n: u32) -> Option<bevy_ecs::prelude::Mut<'w, Zm>> {
    let zm = world.resource_mut::<Zm>();
    zm.bots.contains_key(&n).then_some(zm)
}

/// The free client slots: the local player is 0; bots take the next free.
fn free_client(vm: &Vm<World>, world: &mut World) -> Option<u32> {
    let max = vm
        .dvars
        .get("sv_maxclients")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(18);
    let mut used: Vec<u32> = frame(world)
        .client_ids_sorted()
        .into_iter()
        .map(|c| c.0)
        .collect();
    let zm = world.resource::<Zm>();
    used.extend(zm.players.keys().copied());
    used.extend(zm.test_client_requests.iter().copied());
    (1..max).find(|i| !used.contains(i))
}

pub(super) fn bind(vm: &mut Vm<World>) {
    macro_rules! f {
        ($name:literal, $body:expr) => {
            vm.bind($name, false, $body);
        };
    }
    macro_rules! m {
        ($name:literal, $body:expr) => {
            vm.bind($name, true, $body);
        };
    }
    // A bot joins: its player entity at once (the script marks it a bot and
    // starts its spawn thread before it connects), its client next frames.
    f!("addtestclient", |vm, world, _, _| {
        let Some(n) = free_client(vm, world) else {
            return Ok(Value::Undefined);
        };
        let obj = vm.alloc_object(ObjKind::Entity(n));
        let pers = vm.intern("pers");
        vm.set_raw_field(obj, pers, Value::array(Array::new()));
        let mut zm = world.resource_mut::<Zm>();
        let mut player = Player::new(obj);
        player.test_client = true;
        player.pending_since = Some(zm.now_ms);
        zm.players.insert(n, player);
        zm.bots.insert(n, Bot::default());
        zm.test_client_requests.push(n);
        diag::info!(Sim, "bo2mp bot: test client {n} joins");
        Ok(Value::Object(obj))
    });
    m!("istestclient", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        Ok(Value::bool(
            world
                .resource::<Zm>()
                .players
                .get(&n)
                .is_some_and(|p| p.test_client),
        ))
    });
    m!("botleavegame", |vm, world, s, _| {
        if let Some(n) = entnum(vm, s) {
            world.resource_mut::<Zm>().bot_leaves.push(n);
        }
        Ok(Value::Undefined)
    });
    // Goals.
    m!("addgoal", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let Some(pos) = place(vm, world, arg(a, 0)) else {
            return Ok(Value::Undefined);
        };
        let radius = arg(a, 1).as_float().unwrap_or(24.0);
        let priority = arg(a, 2).as_int().unwrap_or(1);
        let name = text(vm, a, 3);
        if let Some(mut zm) = bot_mut(world, n) {
            let b = zm.bots.get_mut(&n).unwrap();
            b.order += 1;
            let order = b.order;
            b.goals.retain(|g| g.name != name);
            b.goals.push(Goal {
                name,
                pos,
                radius,
                priority,
                order,
            });
        }
        Ok(Value::Undefined)
    });
    m!("cancelgoal", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let name = text(vm, a, 0);
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().goals.retain(|g| g.name != name);
        }
        Ok(Value::Undefined)
    });
    m!("hasgoal", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let name = text(vm, a, 0);
        Ok(Value::bool(world.resource::<Zm>().bots.get(&n).is_some_and(|b| {
            b.goals.iter().any(|g| g.name == name)
        })))
    });
    m!("getgoal", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let name = text(vm, a, 0);
        Ok(world
            .resource::<Zm>()
            .bots
            .get(&n)
            .and_then(|b| b.goals.iter().find(|g| g.name == name))
            .map_or(Value::Undefined, |g| Value::Vec3(g.pos)))
    });
    // At a goal (named, else the one it walks to): within its radius. No
    // goal at all: where it wants to be.
    m!("atgoal", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let name = arg(a, 0);
        let name = if vm.is_defined(name) {
            Some(vm.to_text(name))
        } else {
            None
        };
        let origin = frame(world).player(ClientId(n)).map(|ps| ps.origin);
        let zm = world.resource::<Zm>();
        let Some(b) = zm.bots.get(&n) else {
            return Ok(Value::Int(0));
        };
        let goal = match &name {
            Some(name) => b.goals.iter().find(|g| &g.name == name),
            None => b.best_goal(),
        };
        let at = match (goal, origin) {
            (Some(g), Some(o)) => dist2d(o, g.pos) <= g.radius.max(16.0) && (o[2] - g.pos[2]).abs() < 72.0,
            (None, _) => name.is_none(),
            _ => false,
        };
        Ok(Value::bool(at))
    });
    // The route ahead: its next point's direction and distance (undefined
    // with no route).
    m!("getlookaheaddir", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let origin = frame(world).player(ClientId(n)).map(|ps| ps.origin);
        let next = world
            .resource::<Zm>()
            .bots
            .get(&n)
            .and_then(|b| b.path.first().copied());
        Ok(match (next, origin) {
            (Some(p), Some(o)) => {
                let d = [p[0] - o[0], p[1] - o[1], 0.0];
                let len = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-3);
                Value::Vec3([d[0] / len, d[1] / len, 0.0])
            }
            _ => Value::Undefined,
        })
    });
    m!("getlookaheaddist", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let origin = frame(world).player(ClientId(n)).map(|ps| ps.origin);
        let next = world
            .resource::<Zm>()
            .bots
            .get(&n)
            .and_then(|b| b.path.first().copied());
        Ok(match (next, origin) {
            (Some(p), Some(o)) => Value::Float(dist2d(p, o)),
            _ => Value::Float(0.0),
        })
    });
    m!("botsetfailsafenode", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let p = place(vm, world, arg(a, 0));
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().failsafe = p;
        }
        Ok(Value::Undefined)
    });
    // Looking and buttons.
    m!("lookat", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let p = place(vm, world, arg(a, 0));
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().look = p;
        }
        Ok(Value::Undefined)
    });
    m!("clearlookat", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().look = None;
        }
        Ok(Value::Undefined)
    });
    m!("allowattack", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let on = gsc_t6::truthy(arg(a, 0));
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().allow_attack = on;
        }
        Ok(Value::Undefined)
    });
    m!("pressads", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let on = gsc_t6::truthy(arg(a, 0));
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().ads = on;
        }
        Ok(Value::Undefined)
    });
    // One pull of the trigger (an argument: held that many ticks).
    m!("pressattackbutton", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let ticks = arg(a, 0).as_int().unwrap_or(1).clamp(1, 40) as u32;
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().attack_ticks = ticks;
        }
        Ok(Value::Undefined)
    });
    // Use held for that long (reloads when nothing is there to use).
    m!("pressusebutton", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let secs = arg(a, 0).as_float().unwrap_or(0.1);
        if let Some(mut zm) = bot_mut(world, n) {
            let until = zm.now_ms + (secs * 1000.0) as i64;
            zm.bots.get_mut(&n).unwrap().use_until = until;
        }
        Ok(Value::Undefined)
    });
    m!("pressmelee", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().melee = true;
        }
        Ok(Value::Undefined)
    });
    m!("pressdtpbutton", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().dive = true;
        }
        Ok(Value::Undefined)
    });
    m!("setstance", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let stance = match text(vm, a, 0).as_str() {
            "crouch" => 1,
            "prone" => 2,
            _ => 0,
        };
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().stance = stance;
        }
        Ok(Value::Undefined)
    });
    m!("allowsprint", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let on = gsc_t6::truthy(arg(a, 0));
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().no_sprint = !on;
        }
        // bo2mp: the same switch every player has (the server drops sprint).
        let id = crate::world::ClientId(n);
        let mut f = super::frame(world);
        if f.client_meta(id).is_some() {
            f.client_meta_mut(id).controls.sprint_disabled = !on;
        }
        Ok(Value::Undefined)
    });
    // A grenade at a point: look there and throw.
    m!("throwgrenade", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let p = place(vm, world, arg(a, 1));
        if let Some(mut zm) = bot_mut(world, n) {
            let b = zm.bots.get_mut(&n).unwrap();
            if p.is_some() {
                b.look = p;
            }
            b.tick = 0;
            b.attack_ticks = 0;
        }
        world.resource_mut::<Zm>().bot_throws.push(n);
        Ok(Value::Undefined)
    });
    // Seeing.
    m!("botsighttracepassed", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let Some(from) = eye(world, n) else {
            return Ok(Value::Int(0));
        };
        let target = arg(a, 0);
        let to = match entnum(vm, target) {
            Some(t) if world.resource::<Zm>().players.contains_key(&t) => eye(world, t),
            _ => place(vm, world, target).map(|p| [p[0], p[1], p[2] + 32.0]),
        };
        Ok(Value::bool(to.is_some_and(|to| sight(world, from, to))))
    });
    // Enemies it can see, nearest first, inside its field of view: the
    // argument is the smallest cosine between its view and the enemy (its
    // scripts' `bot.fov`: 0.0872 = 85 degrees each side on normal; -1 =
    // all round).
    m!("getthreats", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let fov = arg(a, 0).as_float().unwrap_or(-1.0);
        let Some(from) = eye(world, n) else {
            return Ok(list(Vec::new()));
        };
        let view = frame(world).player(ClientId(n)).map_or([0.0; 3], |ps| ps.viewangles);
        let mut seen: Vec<(f32, u32)> = Vec::new();
        for e in enemies_of(vm, world, n, true) {
            let Some(to) = eye(world, e) else { continue };
            if fov > -1.0 {
                let (yaw, pitch) = (view[1].to_radians(), view[0].to_radians());
                let forward = [pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), -pitch.sin()];
                let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
                let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-3);
                let dot = (forward[0] * d[0] + forward[1] * d[1] + forward[2] * d[2]) / len;
                if dot < fov {
                    continue;
                }
            }
            if sight(world, from, to) {
                seen.push((dist2d(from, to), e));
            }
        }
        seen.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
        let ids: Vec<u32> = seen.into_iter().map(|(_, e)| e).collect();
        Ok(list(objs(world, &ids)))
    });
    m!("getenemies", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let alive_only = gsc_t6::truthy(arg(a, 0));
        let ids = enemies_of(vm, world, n, alive_only);
        Ok(list(objs(world, &ids)))
    });
    m!("getfriendlies", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let alive_only = gsc_t6::truthy(arg(a, 0));
        let ids = friends_of(vm, world, n, alive_only);
        Ok(list(objs(world, &ids)))
    });
    // Where a player will be in that many frames at his speed.
    m!("predictposition", |vm, world, _, a| {
        let Some(t) = entnum(vm, arg(a, 0)) else {
            return Ok(Value::Undefined);
        };
        let frames = arg(a, 1).as_float().unwrap_or(1.0);
        Ok(frame(world)
            .player(ClientId(t))
            .map_or(Value::Undefined, |ps| {
                Value::Vec3(std::array::from_fn(|i| {
                    ps.origin[i] + ps.velocity[i] * frames * 0.05
                }))
            }))
    });
    // A player's eye height over his feet (where bots aim: torso = feet +
    // height / 1.6).
    m!("getplayerviewheight", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        Ok(Value::Float(
            frame(world)
                .player(ClientId(n))
                .map_or(60.0, |ps| ps.view_height_current),
        ))
    });
    // An entity's middle: a player's chest, else 32 over its origin.
    m!("getcentroid", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        if let Some(ps) = frame(world).player(ClientId(n)) {
            let o = ps.origin;
            return Ok(Value::Vec3([o[0], o[1], o[2] + ps.view_height_current * 0.6]));
        }
        Ok(origin_of(vm, world, s).map_or(Value::Undefined, |o| Value::Vec3([o[0], o[1], o[2] + 32.0])))
    });
    f!("weaponisdualwield", |vm, _, _, a| {
        let w = text(vm, a, 0);
        Ok(Value::bool(w.contains("+dw") || w.ends_with("dw_mp")))
    });
    f!("isweaponscopeoverlay", |_, _, _, _| Ok(Value::Int(0)));
    f!("target_istarget", |_, _, _, _| Ok(Value::Int(0)));
    m!("getplayercamerapos", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        Ok(eye(world, n).map_or(Value::Undefined, Value::Vec3))
    });
    // A bot's own classes (its scripts build five from his item table).
    m!("botsetdefaultclass", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let class = arg(a, 0).as_int().unwrap_or(0);
        let name = text(vm, a, 1);
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().classes.entry(class).or_default().default = Some(name);
        }
        Ok(Value::Undefined)
    });
    m!("botclassadditem", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let class = arg(a, 0).as_int().unwrap_or(0);
        let item = text(vm, a, 1);
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots.get_mut(&n).unwrap().classes.entry(class).or_default().items.push(item);
        }
        Ok(Value::Undefined)
    });
    m!("botclassaddattachment", |vm, world, s, a| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        let class = arg(a, 0).as_int().unwrap_or(0);
        let (weapon, attachment, slot) = (text(vm, a, 1), text(vm, a, 2), text(vm, a, 3));
        if let Some(mut zm) = bot_mut(world, n) {
            zm.bots
                .get_mut(&n)
                .unwrap()
                .classes
                .entry(class)
                .or_default()
                .attachments
                .push((weapon, attachment, slot));
        }
        Ok(Value::Undefined)
    });
    m!("botclasssetweaponoption", |_, _, _, _| Ok(Value::Undefined));
    // The player the bots count teams around: the local player (client 0).
    f!("gethostplayerforbots", |_, world, _, _| Ok(world
        .resource::<Zm>()
        .players
        .get(&0)
        .map_or(Value::Undefined, |p| Value::Object(p.obj))));
    m!("ishostforbots", |vm, _, s, _| Ok(Value::bool(entnum(vm, s) == Some(0))));
    // The map's path nodes as the bot scripts ask about them.
    fn node_obj(world: &World, i: u32) -> Value {
        world
            .resource::<Zm>()
            .node_objs
            .get(i as usize)
            .map_or(Value::Undefined, |o| Value::Object(*o))
    }
    fn node_index(vm: &mut Vm<World>, world: &World, v: &Value) -> Option<u32> {
        let Value::Object(o) = v else { return None };
        world
            .resource::<Zm>()
            .node_of
            .get(o)
            .copied()
            .or_else(|| {
                let f = vm.intern("origin");
                let p = vm.raw_field(*o, f).as_vec3()?;
                world.resource::<Zm>().nav.nearest(p)
            })
    }
    fn node_eye(world: &World, i: u32) -> [f32; 3] {
        let o = world.resource::<Zm>().nav.nodes[i as usize].origin;
        [o[0], o[1], o[2] + 32.0]
    }
    f!("getnearestnode", |vm, world, _, a| {
        let Some(p) = place(vm, world, arg(a, 0)) else {
            return Ok(Value::Undefined);
        };
        let i = world.resource::<Zm>().nav.nearest(p);
        Ok(i.map_or(Value::Undefined, |i| node_obj(world, i)))
    });
    // A node near `from` that can see `to` (nearest first, within 512).
    f!("getvisiblenode", |vm, world, _, a| {
        let (Some(from), Some(to)) = (place(vm, world, arg(a, 0)), place(vm, world, arg(a, 1)))
        else {
            return Ok(Value::Undefined);
        };
        let near = world.resource::<Zm>().nav.within(from, 512.0);
        for (i, _) in near.into_iter().take(12) {
            let e = node_eye(world, i);
            if sight(world, e, [to[0], to[1], to[2] + 32.0]) {
                return Ok(node_obj(world, i));
            }
        }
        Ok(Value::Undefined)
    });
    // The nodes a node sees: its linked neighbours that it can see.
    f!("getvisiblenodes", |vm, world, _, a| {
        let Some(i) = node_index(vm, world, arg(a, 0)) else {
            return Ok(list(Vec::new()));
        };
        let links: Vec<u32> = world.resource::<Zm>().nav.nodes[i as usize]
            .links
            .iter()
            .map(|l| u32::from(l.0))
            .collect();
        let from = node_eye(world, i);
        let mut out = Vec::new();
        for j in links {
            if sight(world, from, node_eye(world, j)) {
                out.push(node_obj(world, j));
            }
        }
        Ok(list(out))
    });
    // BO2 reads the map compiler's sight table (`Path_NodesVisible`); a
    // node past it (or a map without one) is traced.
    f!("nodesvisible", |vm, world, _, a| {
        let (Some(i), Some(j)) = (node_index(vm, world, arg(a, 0)), node_index(vm, world, arg(a, 1)))
        else {
            return Ok(Value::Int(0));
        };
        if let Some(seen) = world.resource::<Zm>().nav.baked.visible(i, j) {
            return Ok(Value::bool(seen));
        }
        let (x, y) = (node_eye(world, i), node_eye(world, j));
        Ok(Value::bool(sight(world, x, y)))
    });
    // BO2's `Scr_NodesCanPath`: a straight walk joins them (the map
    // compiler's smooth table) or the first links to the second; not a
    // search for a route.
    f!("nodescanpath", |vm, world, _, a| {
        let (Some(i), Some(j)) = (node_index(vm, world, arg(a, 0)), node_index(vm, world, arg(a, 1)))
        else {
            return Ok(Value::Int(0));
        };
        let nav = &world.resource::<Zm>().nav;
        let linked = nav.nodes[i as usize].links.iter().any(|l| u32::from(l.0) == j);
        Ok(Value::bool(nav.baked.smooth(i, j) || linked))
    });
    // A route between two points: its nodes in order (undefined: none).
    f!("findpath", |vm, world, _, a| {
        let (Some(from), Some(to)) = (place(vm, world, arg(a, 0)), place(vm, world, arg(a, 1)))
        else {
            return Ok(Value::Undefined);
        };
        let route = {
            let nav = &world.resource::<Zm>().nav;
            match (nav.nearest(from), nav.nearest(to)) {
                (Some(x), Some(y)) => nav.path(x, y),
                _ => None,
            }
        };
        Ok(route.map_or(Value::Undefined, |r| {
            list(r.into_iter().map(|i| node_obj(world, i)).collect())
        }))
    });
    f!("canclaimnode", |_, _, _, _| Ok(Value::Int(1)));
    f!("claimnode", |_, _, _, _| Ok(Value::Undefined));
    f!("releaseclaimednode", |_, _, _, _| Ok(Value::Undefined));
    // Bots have every item (their scripts ask before building a class).
    m!("isitemlocked", |_, _, _, _| Ok(Value::Int(0)));
    // What a player is doing, from his state.
    m!("isreloading", |vm, world, s, _| {
        let n = entnum(vm, s).unwrap_or(u32::MAX);
        Ok(Value::bool(frame(world).player(ClientId(n)).is_some_and(|ps| {
            reloading(ps.weaponstate_primary)
        })))
    });
    for name in ["isfiring", "ismantling", "isonladder", "isthrowinggrenade", "isswitchingweapons", "isremotecontrolling", "iscarryingturret"] {
        vm.bind(name, true, |_, _, _, _| Ok(Value::Int(0)));
    }
}

/// Each tick: the bots' commands (route, view, buttons) replace their
/// clients' own; a bot whose client never came is dropped.
pub(super) fn drive(world: &mut World, now: i64) {
    let ids: Vec<u32> = world.resource::<Zm>().bots.keys().copied().collect();
    if ids.is_empty() {
        return;
    }
    let dv = BotDvars::read(world);
    let crowd = crowd(world);
    let throws = std::mem::take(&mut world.resource_mut::<Zm>().bot_throws);
    for n in ids {
        if !alive(world, n) {
            // Dead, its steering starts over (BO2 clears its bot state).
            if let Some(b) = world.resource_mut::<Zm>().bots.get_mut(&n)
                && b.aim_avg.is_some()
            {
                b.aim_avg = None;
                b.turn_speed = [0.0; 2];
                b.sprint_since = None;
                b.fire_delay_until = 0;
            }
            continue;
        }
        let Some((origin, view, height, ladder, mantling, sprinting, ads_frac, wstate, standing, weapon)) = frame(world)
            .player(ClientId(n))
            .map(|ps| {
                use playerstate_iw4::pm_flags;
                (
                    ps.origin,
                    ps.viewangles,
                    ps.view_height_current,
                    ps.pm_flags & pm_flags::LADDER != 0,
                    ps.pm_flags & pm_flags::MANTLE != 0,
                    ps.pm_flags & pm_flags::SPRINTING != 0,
                    ps.f_weapon_pos_frac.clamp(0.0, 1.0),
                    ps.weaponstate_primary,
                    ps.pm_flags & (pm_flags::CROUCH | pm_flags::PRONE) == 0,
                    ps.weapon,
                )
            })
        else {
            continue;
        };
        // Full-automatic weapons (miniguns too) hold the trigger down.
        let full_auto = frame(world)
            .combat_facts_for(weapon)
            .is_none_or(|f| f.fire_type == 0);
        // The route: replan when the best goal moved or the route ran out.
        let (goal, path_to, replan_at) = {
            let zm = world.resource::<Zm>();
            let b = &zm.bots[&n];
            (b.best_goal().cloned(), b.path_to, b.replan_at)
        };
        if let Some(g) = &goal {
            let moved = path_to.is_none_or(|p| dist2d(p, g.pos) > 16.0);
            let far = dist2d(origin, g.pos) > g.radius;
            let empty = world.resource::<Zm>().bots[&n].path.is_empty();
            if (moved || (empty && far)) && now >= replan_at {
                let route = plan(world, origin, g.pos);
                let mut zm = world.resource_mut::<Zm>();
                let b = zm.bots.get_mut(&n).unwrap();
                b.path = route;
                b.path_to = Some(g.pos);
                b.replan_at = now + 500;
                b.prev = Some(origin);
                b.path_gen = b.path_gen.wrapping_add(1);
                b.path_new = true;
            }
        } else {
            let mut zm = world.resource_mut::<Zm>();
            let b = zm.bots.get_mut(&n).unwrap();
            b.path.clear();
            b.path_to = None;
        }
        let throw = throws.contains(&n);
        // Something at step height between it and the next route point
        // (a ledge, a railing, a low wall), as opposed to stairs or a slope
        // the walk steps up by itself: only then does a jump (and the
        // mantle it starts) help; jumping on stairs under a ceiling holds
        // a bot in place.
        let climb_blocked: Option<([f32; 3], bool)> = {
            let peek = {
                let zm = world.resource::<Zm>();
                let path = &zm.bots[&n].path;
                path.iter()
                    .find(|q| dist2d(origin, **q) >= 24.0)
                    .or(path.last())
                    .copied()
            };
            peek.map(|p| (p, dist2d(origin, p) < 160.0 && blocked_ahead(world, origin, p)))
        };
        // Blocked at step height toward the next point: going up (a ledge)
        // it jumps at once; level or down (a railing it drops over, a low
        // fence) it jumps once it makes no headway.
        let ahead_blocked = climb_blocked.is_some_and(|(_, b)| b);
        let climb_blocked = climb_blocked
            .is_some_and(|(p, b)| b && p[2] - origin[2] > 18.0 && dist2d(origin, p) < 96.0);
        let at_goal = goal
            .as_ref()
            .is_none_or(|g| dist2d(origin, g.pos) <= g.radius.max(16.0));
        // Past a route point once beyond the line across it (the walk cuts
        // corners toward its lookahead, so it seldom touches one).
        let (route, seen_key, seen) = {
            let mut zm = world.resource_mut::<Zm>();
            let b = zm.bots.get_mut(&n).unwrap();
            while b.path.len() > 1 {
                let p = b.path[0];
                let from = b.prev.unwrap_or(origin);
                let d = [p[0] - from[0], p[1] - from[1]];
                let beyond = d[0] * d[0] + d[1] * d[1] > 1.0
                    && (origin[0] - p[0]) * d[0] + (origin[1] - p[1]) * d[1] >= 0.0;
                if !(beyond || dist2d(origin, p) < 24.0) {
                    break;
                }
                b.prev = Some(p);
                b.path.remove(0);
            }
            let near = b.path.first().is_some_and(|p| dist2d(origin, *p) < 192.0);
            let key = (b.path_gen, b.path.len(), near);
            let seen = b.seen.filter(|s| (s.0, s.1, s.2) == key).map(|s| s.3);
            ((!at_goal && !b.path.is_empty()).then(|| b.path.clone()), key, seen)
        };
        let far_seen = match (&route, seen) {
            (Some(_), Some(p)) => Some(p),
            (Some(r), None) => {
                let p = farthest_seen(world, origin, r);
                if let Some(p) = p {
                    let mut zm = world.resource_mut::<Zm>();
                    zm.bots.get_mut(&n).unwrap().seen = Some((seen_key.0, seen_key.1, seen_key.2, p));
                }
                p
            }
            _ => None,
        };
        let la = route.as_deref().and_then(|r| lookahead(origin, r));
        let next = la.map(|(p, _)| p);
        // Avoidance (Bot_UpdateAvoidance): off ladders and ledges, a bot on
        // a team bends its walk around teammates close by, keeping the
        // lookahead's distance; a bend into a clip brush right in front of
        // it holds it still for the tick.
        let mut walk_to = next;
        let mut hold = false;
        if let Some((lp, _)) = la
            && !ladder
            && !mantling
            && let Some(me) = crowd.iter().find(|a| a.id == n && !a.team.is_empty() && a.team != "free")
        {
            let d = [lp[0] - origin[0], lp[1] - origin[1]];
            let dist = dot2(d, d).sqrt();
            let speed = me.vel[0].hypot(me.vel[1]);
            let range = (speed + BOT_AVOID_SPEED) * 0.5 * AVOID_HORIZON;
            let others: Vec<&Avoider> = crowd
                .iter()
                .filter(|a| {
                    a.id != n
                        && a.team == me.team
                        && dist2d(a.pos, origin) < range
                        && (a.pos[2] - origin[2]).abs() < 90.0
                })
                .take(32)
                .collect();
            if dist > 1.0 && !others.is_empty() {
                let pref = [d[0] / dist * speed, d[1] / dist * speed];
                let v = avoid_velocity(me, pref, BOT_AVOID_SPEED, &others);
                let vl = v[0].hypot(v[1]);
                if vl > 0.01 {
                    let bent = [origin[0] + v[0] / vl * dist, origin[1] + v[1] / vl * dist, lp[2]];
                    hold = dist2d(bent, lp) > 15.0 && !avoid_clear(world, origin, bent);
                    walk_to = Some(bent);
                }
            }
        }
        let mut zm = world.resource_mut::<Zm>();
        let b = zm.bots.get_mut(&n).unwrap();
        b.tick = b.tick.wrapping_add(1);
        // The view (BO2's Bot_Think / Bot_UpdateLookahead): the angles it
        // chases move toward what it looks at (an eighth of the way in
        // pitch, a quarter in yaw each tick), else toward the farthest route
        // point it can see, slower the farther its lookahead reaches; the
        // view then steers toward them.
        let eye = [origin[0], origin[1], origin[2] + height];
        let desired: Option<[f32; 2]> = if let Some(l) = b.look {
            let (yaw, pitch) = angles_to(eye, l);
            let avg = b.aim_avg.unwrap_or([view[0], view[1]]);
            b.aim_avg = Some([avg[0] + angle_delta(avg[0], pitch) / 8.0, avg[1] + angle_delta(avg[1], yaw) / 4.0]);
            Some([pitch, yaw])
        } else if let Some((lp, reach)) = la {
            let t = far_seen.filter(|p| dist2d(origin, *p) >= 1.0).unwrap_or(lp);
            if dist2d(origin, t) >= 1.0 {
                let (yaw, pitch) = angles_to(origin, t);
                let h = if b.path_new || reach < 64.0 {
                    1.0
                } else if reach < 128.0 {
                    2.0
                } else {
                    4.0
                };
                let avg = b.aim_avg.unwrap_or([view[0], view[1]]);
                b.aim_avg = Some([avg[0] + angle_delta(avg[0], pitch) / (2.0 * h), avg[1] + angle_delta(avg[1], yaw) / h]);
                Some([pitch, yaw])
            } else {
                None
            }
        } else {
            b.aim_avg = None;
            None
        };
        b.path_new = false;
        let mut want = view;
        match (desired, b.aim_avg) {
            (Some(_), Some(avg)) => {
                let out = steer([view[0], view[1]], avg, &mut b.turn_speed, ads_frac, &dv);
                want[0] = out[0];
                want[1] = out[1];
            }
            _ => b.turn_speed = [0.0; 2],
        }
        let yaw_err = desired.map_or(0.0, |d| angle_delta(want[1], d[1]).abs());
        // Speed (Bot_GetMaxSpeed): full until the lookahead falls under 64
        // units, down to half; walking its route (no look-at) it also slows
        // while it faces away from where it heads, down to half at 35°.
        let speed = la.map_or(0.0, |(_, reach)| {
            let mut s = (reach / 64.0).clamp(0.5, 1.0);
            if b.look.is_none() && desired.is_some() {
                s = s.min((1.0 - yaw_err / 70.0).clamp(0.5, 1.0));
            }
            s * 127.0
        });
        // Moving toward the lookahead, relative to where it faces; on a
        // ladder straight on (up it), while mantling not at all.
        let (mut forward, mut right) = (0i8, 0i8);
        if ladder {
            forward = 127;
        } else if !mantling && !hold && let Some(p) = walk_to {
            let wish = [p[0] - origin[0], p[1] - origin[1]];
            let len = (wish[0] * wish[0] + wish[1] * wish[1]).sqrt();
            if len > 1e-3 {
                let (s, c) = want[1].to_radians().sin_cos();
                forward = (((wish[0] * c + wish[1] * s) / len).clamp(-1.0, 1.0) * speed) as i8;
                right = (((wish[0] * s - wish[1] * c) / len).clamp(-1.0, 1.0) * speed) as i8;
            }
        }
        let mut press = 0u32;
        // Sprint (Bot_UpdateMovement): on a long straight run while facing
        // the way it goes, standing and not reloading; a sprint under 3 s
        // old carries on through a turn.
        if !sprinting {
            b.sprint_since = None;
        } else if b.sprint_since.is_none() {
            b.sprint_since = Some(now);
        }
        let reach = la.map_or(0.0, |(_, r)| r);
        let sprint = if speed == 0.0 || yaw_err >= 5.0 || reach < dv.sprint_distance {
            sprinting && b.sprint_since.is_some_and(|t| now - t < 3000)
        } else {
            !reloading(wstate) && wstate != weapon_iw4::WeaponState::MeleeInit as i32 && standing
        };
        if sprint && !b.no_sprint && b.stance == 0 && !ladder && !mantling {
            press |= buttons::SPRINT;
        }
        match b.stance {
            1 => press |= buttons::CROUCH,
            2 => press |= buttons::PRONE,
            _ => {}
        }
        // Holding use, it presses nothing else (a plant, a defuse).
        if b.use_until > now {
            press = (press & (buttons::CROUCH | buttons::PRONE)) | buttons::USE_RELOAD;
        }
        // The trigger: one press when allowed, then a rest of
        // bot_minfiretime..bot_maxfiretime ms; a full-automatic holds it.
        // Firing lets go of sprint.
        if b.attack_ticks > 0 {
            press = (press & !buttons::SPRINT) | buttons::ATTACK;
            b.attack_ticks -= 1;
        } else if b.allow_attack && now >= b.fire_delay_until {
            press = (press & !buttons::SPRINT) | buttons::ATTACK;
            b.fire_delay_until = if full_auto { 0 } else { now + irand(n, now, dv.min_fire, dv.max_fire) };
        }
        if b.ads {
            press |= buttons::ADS;
        }
        // A melee only while not raising, lowering, meleeing or throwing.
        if std::mem::take(&mut b.melee) && !(1..=5).contains(&wstate) && !(14..=22).contains(&wstate) {
            press |= buttons::MELEE_CHARGE;
        }
        if std::mem::take(&mut b.dive) {
            press |= buttons::PRONE;
        }
        if throw {
            press |= buttons::FRAG;
        }
        // Jumps and vaults: BO2's engine takes a bot over the negotiation
        // links its path nodes mark (ledges, railings, low walls); ours
        // jumps where the route climbs close ahead (the movement code
        // mantles from a jump), and when walking makes no headway.
        // On a ladder it climbs (looking up its route, walking forward):
        // a jump would let go of it.
        if next.is_some() && !ladder && climb_blocked {
            press |= buttons::JUMP;
        }
        if next.is_some() {
            match b.progress {
                Some((at, since)) if dist2d(at, origin) < 12.0 => {
                    let stuck = now - since;
                    if stuck > 600 && (stuck / 300) % 2 == 0 && !ladder && ahead_blocked {
                        press |= buttons::JUMP;
                    }
                    // No headway: back off and step aside for a moment
                    // (a doorway corner, another player, a prop), then on.
                    if (1200..2400).contains(&stuck) && !ladder {
                        forward = -127;
                        right = if (n + (since / 3000) as u32) % 2 == 0 { 127 } else { -127 };
                    }
                    if std::env::var_os("BO2MP_BOTLOG").is_some() && stuck % 1000 == 0 && stuck > 0 {
                        diag::info!(
                            Sim,
                            "bo2mp bot {n} stuck {stuck} ms at {:.0} {:.0} {:.0} next {:?} ahead_blocked {ahead_blocked}",
                            origin[0],
                            origin[1],
                            origin[2],
                            next.map(|p| [p[0] as i32, p[1] as i32, p[2] as i32])
                        );
                    }
                    // Still stuck: drop the route; the next tick plans again.
                    if stuck > 3000 {
                        b.path.clear();
                        b.path_to = None;
                        b.replan_at = now + 250;
                        b.progress = Some((origin, now));
                    }
                }
                _ => b.progress = Some((origin, now)),
            }
        } else {
            b.progress = None;
        }
        // BO2MP_BOTLOG=1: each bot every 2 s (test aid).
        if std::env::var_os("BO2MP_BOTLOG").is_some() && now % 2000 == 0 {
            diag::info!(
                Sim,
                "bo2mp botlog {n}: at {:.0} {:.0} {:.0} view {:.0} {:.0} goal {:?} path {} look {:?} attack {} ads {} press {press:#x} move {forward} {right} sprint {sprint} dodge {} hold {hold}",
                origin[0],
                origin[1],
                origin[2],
                view[0],
                view[1],
                goal.as_ref().map(|g| g.name.as_str()),
                b.path.len(),
                b.look.map(|l| [l[0] as i32, l[1] as i32, l[2] as i32]),
                b.allow_attack,
                b.ads,
                walk_to != next,
            );
        }
        drop(zm);
        // BO2MP_BOTS_STILL=1: the bots stand where they are and do nothing;
        // =fire:N (or fire:N,M) and those bots fire where they face too
        // (test aids, for a target that stays put and a shooter that does).
        if let Some(still) = std::env::var_os("BO2MP_BOTS_STILL") {
            let firing = still
                .to_str()
                .and_then(|s| s.strip_prefix("fire:"))
                .is_some_and(|ids| ids.split(',').any(|i| i.trim().parse::<u32>() == Ok(n)));
            let attack = if firing && (now / 100) % 2 == 0 { buttons::ATTACK } else { 0 };
            (forward, right, press, want) = (0, 0, attack, view);
        }
        // Its command for the tick, the view turned as a script's
        // setplayerangles turns it (the delta under the command angles).
        let mut cmd_angles = None;
        {
            let mut req = world.resource_mut::<crate::step::StepRequest>();
            for (id, cmd) in &mut req.input.cmds {
                if id.0 != n {
                    continue;
                }
                cmd.buttons = press;
                cmd.forwardmove = forward;
                cmd.rightmove = right;
                cmd_angles = Some(cmd.angles);
            }
        }
        if let Some(angles) = cmd_angles {
            let mut f = frame(world);
            if let Some(p) = f.player_mut(ClientId(n)) {
                p.delta_angles = std::array::from_fn(|i| {
                    let cmd_deg = (angles[i] as u16) as f32 * (360.0 / 65536.0);
                    want[i] - cmd_deg
                });
                p.viewangles = want;
            }
        }
    }
}

/// A route from a place to a goal over the path nodes (the goal last);
/// straight there when the map has no route between them.
fn plan(world: &mut World, from: [f32; 3], to: [f32; 3]) -> Vec<[f32; 3]> {
    let zm = world.resource::<Zm>();
    let nav = &zm.nav;
    let (Some(a), Some(b)) = (nav.nearest(from), nav.nearest(to)) else {
        return vec![to];
    };
    let Some(nodes) = nav.path(a, b) else {
        return vec![to];
    };
    let mut out: Vec<[f32; 3]> = nodes
        .iter()
        .map(|&i| nav.nodes[i as usize].origin)
        .collect();
    // Skip the first node when the second is nearer (already past it).
    if out.len() > 1 && dist2d(from, out[1]) < dist2d(out[0], out[1]) {
        out.remove(0);
    }
    out.push(to);
    out
}
