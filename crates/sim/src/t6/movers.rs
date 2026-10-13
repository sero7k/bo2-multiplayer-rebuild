//! Script movers: `moveto`, `rotateto` and friends, `movegravity`, and
//! entities linked to another (`linkto`). Each finished move notifies
//! `movedone` / `rotatedone` on its entity.

use std::collections::BTreeMap;

use bevy_ecs::prelude::World;
use gsc_t6::Value;

use super::{Zm, frame, with_vm};
use crate::world::ClientId;

#[derive(Clone, Debug)]
pub(crate) struct Track {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub start_ms: i64,
    pub dur_ms: i64,
    pub accel_ms: i64,
    pub decel_ms: i64,
}

impl Track {
    /// Share of the way at `now` (trapezoid speed: accelerate, cruise,
    /// decelerate), 1 at the end.
    fn fraction(&self, now: i64) -> f32 {
        let t = (now - self.start_ms) as f32;
        let total = self.dur_ms.max(1) as f32;
        if t >= total {
            return 1.0;
        }
        if t <= 0.0 {
            return 0.0;
        }
        let a = (self.accel_ms as f32).clamp(0.0, total);
        let d = (self.decel_ms as f32).clamp(0.0, total - a);
        let v = 1.0 / (total - 0.5 * a - 0.5 * d);
        if t < a {
            0.5 * v / a * t * t
        } else if t <= total - d {
            0.5 * v * a + v * (t - a)
        } else {
            let r = total - t;
            1.0 - 0.5 * v / d.max(1e-3) * r * r
        }
    }

    fn at(&self, now: i64) -> [f32; 3] {
        let f = self.fraction(now);
        [
            self.from[0] + (self.to[0] - self.from[0]) * f,
            self.from[1] + (self.to[1] - self.from[1]) * f,
            self.from[2] + (self.to[2] - self.from[2]) * f,
        ]
    }

    fn done(&self, now: i64) -> bool {
        now - self.start_ms >= self.dur_ms
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Move {
    pub pos: Option<Track>,
    pub rot: Option<Track>,
    /// Launch origin, velocity, start, duration.
    pub gravity: Option<([f32; 3], [f32; 3], i64, i64)>,
    pub vibe: Option<Vibe>,
}

/// A `vibrate`: the angles it rocks about, the tilt per degree of
/// amplitude (pitch, roll), the amplitude, period and time.
#[derive(Clone, Debug)]
pub(crate) struct Vibe {
    base: [f32; 3],
    tilt: [f32; 2],
    amp: f32,
    period_ms: f32,
    start_ms: i64,
    dur_ms: i64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Movers {
    pub list: BTreeMap<u32, Move>,
}

impl Movers {
    pub(crate) fn stop(&mut self, n: u32, pos: bool, rot: bool) {
        if let Some(m) = self.list.get_mut(&n) {
            if pos {
                m.pos = None;
                m.gravity = None;
            }
            if rot {
                m.rot = None;
            }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/movers.rs
            if m.pos.is_none() && m.rot.is_none() && m.gravity.is_none() {
=======
            if m.pos.is_none() && m.rot.is_none() && m.gravity.is_none() && m.vibe.is_none() {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/movers.rs
                self.list.remove(&n);
            }
        }
    }

    pub(crate) fn move_to(
        &mut self,
        n: u32,
        from: [f32; 3],
        to: [f32; 3],
        now: i64,
        secs: f32,
        accel: f32,
        decel: f32,
    ) {
        let m = self.list.entry(n).or_default();
        m.gravity = None;
        m.pos = Some(Track {
            from,
            to,
            start_ms: now,
            dur_ms: (secs * 1000.0).round().max(1.0) as i64,
            accel_ms: (accel * 1000.0) as i64,
            decel_ms: (decel * 1000.0) as i64,
        });
    }

    pub(crate) fn rotate_to(
        &mut self,
        n: u32,
        from: [f32; 3],
        to: [f32; 3],
        now: i64,
        secs: f32,
        accel: f32,
        decel: f32,
    ) {
        let m = self.list.entry(n).or_default();
        m.rot = Some(Track {
            from,
            to,
            start_ms: now,
            dur_ms: (secs * 1000.0).round().max(1.0) as i64,
            accel_ms: (accel * 1000.0) as i64,
            decel_ms: (decel * 1000.0) as i64,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn vibrate(
        &mut self,
        n: u32,
        base: [f32; 3],
        dir: [f32; 3],
        amp: f32,
        period: f32,
        now: i64,
        secs: f32,
    ) {
        let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt().max(1e-3);
        self.list.entry(n).or_default().vibe = Some(Vibe {
            base,
            tilt: [dir[0] / len, dir[1] / len],
            amp,
            period_ms: (period * 1000.0).max(1.0),
            start_ms: now,
            dur_ms: (secs * 1000.0).round() as i64,
        });
    }

    pub(crate) fn gravity(&mut self, n: u32, from: [f32; 3], vel: [f32; 3], now: i64, secs: f32) {
        let m = self.list.entry(n).or_default();
        m.pos = None;
        m.gravity = Some((from, vel, now, (secs * 1000.0).round().max(1.0) as i64));
    }
}

pub(crate) const GRAVITY: f32 = 800.0;

/// Advance every mover to `now`; then linked entities follow their parents.
pub(crate) fn advance(world: &mut World, now: i64) {
    let mut done: Vec<(u32, &'static str)> = Vec::new();
    {
        let mut zm = world.resource_mut::<Zm>();
        let zm = &mut *zm;
        let mut finished = Vec::new();
        for (&n, m) in &mut zm.movers.list {
            let Some(e) = zm.ents.get_mut(&n) else {
                finished.push(n);
                continue;
            };
            if let Some(t) = &m.pos {
                e.origin = t.at(now);
                if t.done(now) {
                    e.origin = t.to;
                    m.pos = None;
                    done.push((n, "movedone"));
                }
            }
            if let Some((from, vel, start, dur)) = m.gravity {
                let t = ((now - start).min(dur) as f32) / 1000.0;
                e.origin = [
                    from[0] + vel[0] * t,
                    from[1] + vel[1] * t,
                    from[2] + vel[2] * t - 0.5 * GRAVITY * t * t,
                ];
                if now - start >= dur {
                    m.gravity = None;
                    done.push((n, "movedone"));
                }
            }
            if let Some(t) = &m.rot {
                e.angles = t.at(now);
                if t.done(now) {
                    e.angles = t.to;
                    m.rot = None;
                    done.push((n, "rotatedone"));
                }
            }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t6/movers.rs
            if m.pos.is_none() && m.rot.is_none() && m.gravity.is_none() {
=======
            if let Some(v) = &m.vibe {
                let t = now - v.start_ms;
                if t >= v.dur_ms {
                    e.angles = v.base;
                    m.vibe = None;
                } else {
                    let fade = 1.0 - t as f32 / v.dur_ms.max(1) as f32;
                    let w = (std::f32::consts::TAU * t as f32 / v.period_ms).sin() * v.amp * fade;
                    e.angles = [v.base[0] + w * v.tilt[0], v.base[1], v.base[2] + w * v.tilt[1]];
                }
            }
            if m.pos.is_none() && m.rot.is_none() && m.gravity.is_none() && m.vibe.is_none() {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t6/movers.rs
                finished.push(n);
            }
        }
        for n in finished {
            zm.movers.list.remove(&n);
        }
    }
    follow_links(world);
    if done.is_empty() {
        return;
    }
    with_vm(world, |vm, world| {
        for (n, what) in done {
            let obj = world.resource::<Zm>().ents.get(&n).and_then(|e| e.obj);
            if let Some(o) = obj {
                vm.notify_str(world, o, what, &[]);
            }
        }
    });
}

/// Linked entities: origin = parent's + offset turned by the parent's yaw;
/// angles = parent's + offset.
pub(crate) fn follow_links(world: &mut World) {
    let links: Vec<(u32, u32, [f32; 3], [f32; 3], Option<String>)> = world
        .resource::<Zm>()
        .ents
        .iter()
        .filter_map(|(n, e)| e.link.map(|(p, o, a)| (*n, p, o, a, e.link_tag.clone())))
        .collect();
    for (n, parent, off, aoff, tag) in links {
        let Some((po, pa)) = parent_pose(world, parent, tag.as_deref()) else {
            continue;
        };
        let (f, r, u) = gsc_t6::math::angle_vectors(pa);
        let origin = [
            po[0] + f[0] * off[0] - r[0] * off[1] + u[0] * off[2],
            po[1] + f[1] * off[0] - r[1] * off[1] + u[1] * off[2],
            po[2] + f[2] * off[0] - r[2] * off[1] + u[2] * off[2],
        ];
        if let Some(e) = world.resource_mut::<Zm>().ents.get_mut(&n) {
            e.origin = origin;
            e.angles = [pa[0] + aoff[0], pa[1] + aoff[1], pa[2] + aoff[2]];
        }
    }
    let _ = Value::Undefined;
}

/// Where a link parent is this tick (on a tag: where the parent's posed
/// model has it): what `follow_links` places its children by.
fn parent_pose(
    world: &mut World,
    parent: u32,
    tag: Option<&str>,
) -> Option<([f32; 3], [f32; 3])> {
    if let Some(on_tag) = tag
        .and_then(|t| super::tags::tag_world(world, parent, t))
        .map(|(o, axes)| (o, super::tags::axis_to_angles(axes)))
    {
        return Some(on_tag);
    }
    if world.resource::<Zm>().players.contains_key(&parent) {
        return frame(world)
            .player(ClientId(parent))
            .map(|ps| (ps.origin, [0.0, ps.viewangles[1], 0.0]));
    }
    world
        .resource::<Zm>()
        .ents
        .get(&parent)
        .map(|e| (e.origin, e.angles))
}

/// Where entity `n`'s link parent (or the tag it is linked on) is now;
/// none when it is not linked.
pub(crate) fn link_parent_pose(world: &mut World, n: u32) -> Option<([f32; 3], [f32; 3])> {
    let (parent, tag) = world
        .resource::<Zm>()
        .ents
        .get(&n)
        .and_then(|e| e.link.map(|(p, _, _)| (p, e.link_tag.clone())))?;
    parent_pose(world, parent, tag.as_deref())
}

/// A linked entity moved by its own animation keeps the place it is at
/// now relative to its parent (BO2's G_CalcTagAxis after an animscripted
/// think): `follow_links` carries it from there, and it stays there when
/// the animation ends.
pub(crate) fn relink_here(world: &mut World, n: u32) {
    let Some((po, pa)) = link_parent_pose(world, n) else {
        return;
    };
    let mut zm = world.resource_mut::<Zm>();
    let Some(e) = zm.ents.get_mut(&n) else {
        return;
    };
    let Some((parent, _, _)) = e.link else {
        return;
    };
    let (f, r, u) = gsc_t6::math::angle_vectors(pa);
    let d = gsc_t6::math::sub(e.origin, po);
    let off = [
        gsc_t6::math::dot(d, f),
        -gsc_t6::math::dot(d, r),
        gsc_t6::math::dot(d, u),
    ];
    let aoff = [0, 1, 2].map(|i| e.angles[i] - pa[i]);
    e.link = Some((parent, off, aoff));
}
