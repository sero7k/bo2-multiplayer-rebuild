//! Origins (zm_tomb) fire staff biplane test aid.
//!
//! IW4L_T6_BIPLANE=<log|shoot>[@<seconds>] (from 20 s unless given). Every
//! second while the biplane (`spawnvehicle` model `veh_t6_dlc_zm_biplane`)
//! flies: where it is, how far from the player, its health and its
//! `biplane_down` flag. `shoot`: the player looks at it and fires every
//! 0.3 s (clip refilled) while it is within 7000 units (bullets reach 8192;
//! its loop never comes nearer than about 4000 to the spawn), until it is
//! down (one hit does it, `level.callbackvehicledamage`).
//! It flies only once `activate_zone_village_0` is set and a round has gone
//! by (`staff_biplane_drop_pieces`), so run it with IW4L_T6_OPENALL=buy;
//! `shoot` ends the round he bought the village in (round_number + 1, once)
//! so the plane takes off without a round to play.

use std::sync::{Mutex, OnceLock};

use bevy_ecs::prelude::World;
use gsc_t6::{ObjRef, Value};
use playerstate_iw4::buttons;

use super::{frame, with_vm};
use crate::world::ClientId;

fn want() -> Option<&'static (String, i64)> {
    static WANT: OnceLock<Option<(String, i64)>> = OnceLock::new();
    WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_BIPLANE").ok()?;
        Some(match v.split_once('@') {
            Some((m, s)) => (
                m.to_owned(),
                s.parse::<f32>().map_or(20_000, |s| (s * 1000.0) as i64),
            ),
            None => (v, 20_000),
        })
    })
    .as_ref()
}

struct State {
    seen: bool,
    gone: bool,
    round_ended: bool,
    village_at: Option<i64>,
    moved_at: Option<i64>,
}

static STATE: Mutex<State> = Mutex::new(State {
    seen: false,
    gone: false,
    round_ended: false,
    village_at: None,
    moved_at: None,
});

/// The shot from `eye` at the plane goes clear to within 300 of it.
fn clear_to(world: &mut World, eye: [f32; 3], plane: [f32; 3]) -> bool {
    let t = frame(world).trace_static_world(eye, plane, [0.0; 3], [0.0; 3], crate::bullet_collision::MASK_SHOT);
    let hit: [f32; 3] = std::array::from_fn(|i| eye[i] + (plane[i] - eye[i]) * t.fraction);
    t.startsolid == 0 && gsc_t6::math::length(gsc_t6::math::sub(hit, plane)) < 300.0
}

/// A floor within 3200 of `from`, open overhead, whose shot at the plane
/// goes clear: the one nearest the plane.
fn open_spot(world: &mut World, from: [f32; 3], plane: [f32; 3]) -> Option<[f32; 3]> {
    let mut best: Option<([f32; 3], f32)> = None;
    for dist in [400.0, 800.0, 1200.0, 1800.0, 2400.0, 3200.0] {
        for k in 0..12 {
            let yaw = (30.0 * k as f32).to_radians();
            for up in [80.0, 400.0] {
                let c = [from[0] + dist * yaw.cos(), from[1] + dist * yaw.sin(), from[2] + up];
                let Some(f) = super::tomb_staffs::floor_under(world, c) else {
                    continue;
                };
                let eye = [f[0], f[1], f[2] + 60.0];
                let sky = frame(world).trace_static_world(
                    eye,
                    [eye[0], eye[1], eye[2] + 600.0],
                    [0.0; 3],
                    [0.0; 3],
                    crate::bullet_collision::MASK_SHOT,
                );
                if sky.startsolid != 0 || sky.fraction < 1.0 || !clear_to(world, eye, plane) {
                    continue;
                }
                let d = gsc_t6::math::length(gsc_t6::math::sub(plane, eye));
                if best.is_none_or(|b| d < b.1) {
                    best = Some((f, d));
                }
            }
        }
    }
    best.map(|b| b.0)
}

pub(crate) fn biplane_test(world: &mut World, now: i64) {
    let Some((mode, at)) = want() else {
        return;
    };
    if now < *at {
        return;
    }
    let client = ClientId(0);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let Some(ps) = frame(world).player(client).copied() else {
        return;
    };
    let plane: Option<ObjRef> = world
        .resource::<super::Zm>()
        .ents
        .values()
        // The flying one (`spawnvehicle`), not the map's parked template.
        .find(|e| e.model == "veh_t6_dlc_zm_biplane" && !e.map)
        .and_then(|e| e.obj);
    let mut st = STATE.lock().unwrap();
    if plane.is_none() && !st.seen && !st.round_ended && mode == "shoot" {
        // Two seconds after the village opens (the quest thread has noted
        // its round by then).
        let open = with_vm(world, |vm, _| super::tomb_test::level_flag(vm, "activate_zone_village_0"))
            .unwrap_or(false);
        if open && st.village_at.is_none() {
            st.village_at = Some(now);
        }
        if st.village_at.is_some_and(|t| now >= t + 2000) {
            st.round_ended = true;
            let n = with_vm(world, |vm, world| {
                let f = vm.intern("round_number");
                let level = vm.level;
                let n = vm.get_field(world, level, f).as_int().unwrap_or(1) + 1;
                vm.set_field(world, level, f, Value::Int(n));
                n
            })
            .unwrap_or(0);
            diag::info!(Sim, "bo2zm t6 biplane test {}s: the village is open; round now {n}", now / 1000);
        }
    }
    let Some(p) = plane else {
        if st.seen && !st.gone {
            st.gone = true;
            diag::info!(Sim, "bo2zm t6 biplane test {}s: the biplane is gone", now / 1000);
        }
        return;
    };
    let (origin, health, down) = with_vm(world, |vm, world| {
        let o = super::origin_of(vm, world, &Value::Object(p));
        let h = vm.intern("health");
        let h = vm.get_field(world, p, h);
        let d = super::tomb_test::ent_flag(vm, world, p, "biplane_down");
        (o, vm.to_text(&h), d)
    })
    .unwrap_or((None, String::new(), String::new()));
    let Some(origin) = origin else {
        return;
    };
    if !st.seen {
        st.seen = true;
        diag::info!(Sim, "bo2zm t6 biplane test {}s: the biplane flies", now / 1000);
    }
    let eye = [ps.origin[0], ps.origin[1], ps.origin[2] + ps.view_height_current];
    // Under a roof (plane3: the spawn's dugout) every shot stops in it: while
    // his shot at it is blocked, every 3 s stand him on open ground with a
    // clear shot, the nearest such to the plane.
    if mode == "shoot"
        && down != "1"
        && now % 1000 < tick
        && st.moved_at.is_none_or(|t| now - t >= 3000)
        && !clear_to(world, eye, origin)
    {
        st.moved_at = Some(now);
        if let Some(f) = open_spot(world, ps.origin, origin) {
            super::teleport_player(world, client, f);
            diag::info!(
                Sim,
                "bo2zm t6 biplane test {}s: no clear shot at it; player to ({:.0} {:.0} {:.0})",
                now / 1000,
                f[0],
                f[1],
                f[2]
            );
        }
    }
    let dist = gsc_t6::math::length(gsc_t6::math::sub(origin, eye));
    if mode == "shoot" && down != "1" && dist < 7000.0 {
        super::set_player_view(world, client, super::autoplay::look(eye, origin));
        if now % 300 < tick {
            {
                let mut f = frame(world);
                if let Some(w) = f.player(client).map(|p| p.weapon) {
                    crate::script_player::set_ammo_clip(&mut f, client, w, 8);
                }
            }
            let mut req = world.resource_mut::<crate::step::StepRequest>();
            for (id, cmd) in &mut req.input.cmds {
                if *id == client {
                    cmd.buttons |= buttons::ATTACK;
                }
            }
        }
    }
    if now % 1000 < tick {
        diag::info!(
            Sim,
            "bo2zm t6 biplane test {}s: biplane at ({:.0} {:.0} {:.0}) {dist:.0} from the player, health {health}, biplane_down {down}",
            now / 1000,
            origin[0],
            origin[1],
            origin[2]
        );
    }
}
