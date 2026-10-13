//! Origins (zm_tomb) giant robot test aid.
//!
//! IW4L_T6_ROBOT=<mode>[@<seconds>] (from 20 s unless given). Every second
//! while a robot walks (`level.a_giant_robots`, `is_walking`): its id, where
//! it is, its `kill_trigger_active` flag, the hatch foot and whether that
//! hatch is open (`level.gr_foot_hatch_closed`), how many zombies a foot has
//! marked for death so far, and the player's `is_stomped` /
//! `in_giant_robot_head`. Each time a foot's kill trigger switches on, both
//! foot triggers' heights are logged.
//!
//! Modes: `log` only logs. `stomp`: each time a kill trigger switches on
//! (until he is stomped) the player is put on the ground under the higher
//! foot, the one coming down. `hatch`: once the robot has picked its hatch
//! foot he stands 260 units from that foot and shoots its sole every 0.3 s
//! until the hatch opens; then, each time a kill trigger switches on, he is
//! put under the hatch foot until the game takes him into the head.

use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};

use bevy_ecs::prelude::World;
use gsc_t6::{Key, ObjRef, Value, Vm};
use playerstate_iw4::buttons;

use super::{frame, with_vm};
use crate::world::ClientId;

fn want() -> Option<&'static (String, i64)> {
    static WANT: OnceLock<Option<(String, i64)>> = OnceLock::new();
    WANT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_ROBOT").ok()?;
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

struct Robot {
    id: i32,
    origin: [f32; 3],
    active: bool,
    hatch_foot: String,
    hatch_open: bool,
    /// Right then left foot trigger origins.
    feet: [Option<[f32; 3]>; 2],
}

fn field(vm: &mut Vm<World>, world: &mut World, o: ObjRef, f: &str) -> Value {
    let f = vm.intern(f);
    vm.get_field(world, o, f)
}

/// The walking robots.
fn walking(vm: &mut Vm<World>, world: &mut World) -> Vec<Robot> {
    let f = vm.intern("a_giant_robots");
    let list: Vec<ObjRef> = match vm.get_field(world, vm.level, f) {
        Value::Array(a) => a.read().values_in_order().filter_map(Value::as_obj).collect(),
        _ => Vec::new(),
    };
    let f = vm.intern("gr_foot_hatch_closed");
    let closed = match vm.get_field(world, vm.level, f) {
        Value::Array(a) => Some(a),
        _ => None,
    };
    let mut out = Vec::new();
    for r in list {
        if !gsc_t6::truthy(&field(vm, world, r, "is_walking")) {
            continue;
        }
        let id = match field(vm, world, r, "giant_robot_id") {
            Value::Int(i) => i,
            _ => -1,
        };
        let origin = super::origin_of(vm, world, &Value::Object(r)).unwrap_or_default();
        let active = super::tomb_test::ent_flag(vm, world, r, "kill_trigger_active") == "1";
        let hf = field(vm, world, r, "hatch_foot");
        let hatch_foot = match hf {
            Value::Undefined => String::new(),
            v => vm.to_text(&v),
        };
        let hatch_open = closed
            .as_ref()
            .and_then(|a| a.get(&Key::Int(id)))
            .is_some_and(|v| !gsc_t6::truthy(&v));
        let mut feet = [None, None];
        for (i, name) in ["trig_stomp_kill_right", "trig_stomp_kill_left"].iter().enumerate() {
            let t = field(vm, world, r, name);
            feet[i] = super::origin_of(vm, world, &t);
        }
        out.push(Robot {
            id,
            origin,
            active,
            hatch_foot,
            hatch_open,
            feet,
        });
    }
    out
}

/// The ground under `at` (a foot trigger sits 72 units over the foot).
fn ground_under(world: &mut World, at: [f32; 3]) -> Option<[f32; 3]> {
    let stand = [at[0], at[1], at[2]];
    let down = [stand[0], stand[1], stand[2] - 400.0];
    let t = frame(world).trace_static_world(
        stand,
        down,
        crate::bullet_collision::PLAYER_MINS,
        crate::bullet_collision::PLAYER_MAXS,
        crate::bullet_collision::MASK_PLAYER_SOLID,
    );
    (t.startsolid == 0 && t.fraction < 1.0)
        .then(|| std::array::from_fn(|i| stand[i] + (down[i] - stand[i]) * t.fraction))
}

fn press(world: &mut World, client: ClientId, b: u32) {
    let mut req = world.resource_mut::<crate::step::StepRequest>();
    for (id, cmd) in &mut req.input.cmds {
        if *id == client {
            cmd.buttons |= b;
        }
    }
}

struct State {
    was_active: bool,
    stomped_seen: bool,
    head_seen: bool,
    marked: BTreeSet<u32>,
    /// Where he stands to shoot the sole.
    shoot_spot: Option<[f32; 3]>,
}

static STATE: Mutex<State> = Mutex::new(State {
    was_active: false,
    stomped_seen: false,
    head_seen: false,
    marked: BTreeSet::new(),
    shoot_spot: None,
});

pub(crate) fn robot_test(world: &mut World, now: i64) {
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
    // Zombies a foot has marked for death (counted once each).
    let actors: Vec<(u32, ObjRef)> = {
        let zm = world.resource::<super::Zm>();
        zm.actors.iter().filter_map(|(n, a)| Some((*n, a.obj?))).collect()
    };
    let (robots, marked_now, stomped, head) = with_vm(world, |vm, world| {
        let robots = walking(vm, world);
        let mf = vm.intern("marked_for_death");
        let marked: Vec<u32> = if robots.is_empty() {
            Vec::new()
        } else {
            actors
                .iter()
                .filter(|(_, o)| gsc_t6::truthy(&vm.get_field(world, *o, mf)))
                .map(|(n, _)| *n)
                .collect()
        };
        let p = world.resource::<super::Zm>().players.get(&0).map(|p| p.obj);
        let (stomped, head) = match p {
            Some(p) => {
                let s = gsc_t6::truthy(&field(vm, world, p, "is_stomped"));
                let h = field(vm, world, p, "in_giant_robot_head");
                (s, if matches!(h, Value::Undefined) { String::new() } else { vm.to_text(&h) })
            }
            None => (false, String::new()),
        };
        (robots, marked, stomped, head)
    })
    .unwrap_or_default();
    let mut st = STATE.lock().unwrap();
    st.marked.extend(marked_now);
    let active = robots.iter().any(|r| r.active);
    let rising = active && !st.was_active;
    st.was_active = active;
    if stomped && !st.stomped_seen {
        st.stomped_seen = true;
        diag::info!(Sim, "bo2zm t6 robot test {}s: player stomped (is_stomped 1, health {})", now / 1000, ps.health);
    }
    if !head.is_empty() && !st.head_seen {
        st.head_seen = true;
        diag::info!(Sim, "bo2zm t6 robot test {}s: player in robot head {head}", now / 1000);
    }
    if rising {
        for r in robots.iter().filter(|r| r.active) {
            let z = |f: Option<[f32; 3]>| f.map_or("-".to_owned(), |p| format!("{:.0}", p[2]));
            diag::info!(
                Sim,
                "bo2zm t6 robot test {}s: robot {} kill trigger on; right foot z {} left foot z {}",
                now / 1000,
                r.id,
                z(r.feet[0]),
                z(r.feet[1])
            );
        }
    }
    let robot = robots.iter().find(|r| r.active).or(robots.first());
    match mode.as_str() {
        "stomp" if rising && !st.stomped_seen => {
            if let Some(r) = robot {
                let foot = match (r.feet[0], r.feet[1]) {
                    (Some(a), Some(b)) => Some(if a[2] >= b[2] { a } else { b }),
                    (a, b) => a.or(b),
                };
                if let Some(g) = foot.and_then(|f| ground_under(world, f)) {
                    super::teleport_player(world, client, g);
                    diag::info!(Sim, "bo2zm t6 robot test {}s: put under robot {} foot at ({:.0} {:.0} {:.0})", now / 1000, r.id, g[0], g[1], g[2]);
                }
            }
        }
        "hatch" if !st.head_seen => {
            if let Some(r) = robot.filter(|r| !r.hatch_foot.is_empty()) {
                let i = usize::from(r.hatch_foot == "left");
                if let Some(foot) = r.feet[i] {
                    if r.hatch_open {
                        if rising {
                            if let Some(g) = ground_under(world, foot) {
                                super::teleport_player(world, client, g);
                                diag::info!(Sim, "bo2zm t6 robot test {}s: hatch open, put under robot {} {} foot", now / 1000, r.id, r.hatch_foot);
                            }
                        }
                    } else {
                        // Stand 260 units off the hatch foot (moved again when the
                        // foot has walked 500 away) and shoot its sole.
                        let sole = [foot[0], foot[1], foot[2] - 72.0];
                        let far = st.shoot_spot.is_none_or(|s| {
                            ((s[0] - sole[0]).powi(2) + (s[1] - sole[1]).powi(2)).sqrt() > 500.0
                        });
                        if far {
                            for deg in [0.0f32, 90.0, 180.0, 270.0, 45.0, 135.0, 225.0, 315.0] {
                                let a = deg.to_radians();
                                let p = [sole[0] + a.cos() * 260.0, sole[1] + a.sin() * 260.0, sole[2] + 120.0];
                                if let Some(g) = ground_under(world, p) {
                                    super::teleport_player(world, client, g);
                                    st.shoot_spot = Some(g);
                                    break;
                                }
                            }
                        }
                        let feet = st.shoot_spot.unwrap_or(ps.origin);
                        let eye = [feet[0], feet[1], feet[2] + ps.view_height_current];
                        super::set_player_view(world, client, super::autoplay::look(eye, sole));
                        if now % 300 < tick {
                            {
                                let mut f = frame(world);
                                if let Some(w) = f.player(client).map(|p| p.weapon) {
                                    crate::script_player::set_ammo_clip(&mut f, client, w, 8);
                                }
                            }
                            press(world, client, buttons::ATTACK);
                        }
                    }
                }
            }
        }
        _ => {}
    }
    if now % 1000 >= tick || robots.is_empty() {
        return;
    }
    for r in &robots {
        diag::info!(
            Sim,
            "bo2zm t6 robot test {}s: robot {} at ({:.0} {:.0} {:.0}) kill_trigger_active {} hatch_foot {} hatch_open {}; zombies marked {}; player ({:.0} {:.0} {:.0}) health {} stomped {} head {}",
            now / 1000,
            r.id,
            r.origin[0],
            r.origin[1],
            r.origin[2],
            u8::from(r.active),
            if r.hatch_foot.is_empty() { "-" } else { r.hatch_foot.as_str() },
            u8::from(r.hatch_open),
            st.marked.len(),
            ps.origin[0],
            ps.origin[1],
            ps.origin[2],
            ps.health,
            u8::from(stomped),
            if head.is_empty() { "-" } else { head.as_str() }
        );
    }
}
