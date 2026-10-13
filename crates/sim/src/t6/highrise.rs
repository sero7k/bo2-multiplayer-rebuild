//! Die Rise (zm_highrise): its elevator cars carry players.
//!
//! A car is a script model (`p6_anim_zm_hr_elevator_common`, the freight
//! car `..._freight`) made a moving platform by `init_elevator`
//! (`setmovingplatformenabled`). Script models do not block player
//! movement here, so a car's floor and roof are kept as thin turned boxes
//! (presence's blockers) and `riders` carries whoever stands on one.

/// A car model's solid slabs in its own space: (bottom, top) heights; the
/// box's other sides are the model's bounds. The floor's top is 8 over the
/// origin (a perk machine linked to a car stands there, e.g. Jugger-Nog at
/// z 1296 in the car at 1288); the roof's top is the model's top.
pub(crate) fn platform_slabs(model: &str) -> &'static [(f32, f32)] {
    match model {
        "p6_anim_zm_hr_elevator_common" => &[(-8.0, 8.0), (130.0, 138.0)],
        "p6_anim_zm_hr_elevator_freight" => &[(-8.0, 8.0), (213.0, 221.0)],
        _ => &[],
    }
}

/// A car a player rides by standing on its floor or roof (his ground
/// entity is the car): the clients draw that ride smooth.
pub fn platform_rides_on_ground(model: &str) -> bool {
    !platform_slabs(model).is_empty()
}

/// Inside a moving car: a point kept within its walls (its box in its own
/// space, a player's radius in from each side), given the car's pose.
pub(crate) fn keep_inside(
    p: [f32; 3],
    origin: [f32; 3],
    angles: [f32; 3],
    (lo, hi): ([f32; 3], [f32; 3]),
) -> [f32; 3] {
    const RADIUS: f32 = 16.0;
    let (f, r, _) = gsc_t6::math::angle_vectors(angles);
    let d = gsc_t6::math::sub(p, origin);
    let l = [gsc_t6::math::dot(d, f), -gsc_t6::math::dot(d, r)];
    let mut m = l;
    for i in 0..2 {
        let (a, b) = (lo[i] + RADIUS, hi[i] - RADIUS);
        if a < b {
            m[i] = l[i].clamp(a, b);
        }
    }
    let (dx, dy) = (m[0] - l[0], m[1] - l[1]);
    [
        p[0] + f[0] * dx - r[0] * dy,
        p[1] + f[1] * dx - r[1] * dy,
        p[2] + f[2] * dx - r[2] * dy,
    ]
}

/// Each tick (every map; Die Rise's own parts return at once elsewhere).
pub(crate) fn tick(world: &mut bevy_ecs::prelude::World, now: i64) {
    zone_log(world, now);
    power_look(world, now);
    lion_balls(world, now);
    sq_test(world, now);
    dragon_fire(world, now);
    tag_probe(world, now);
}

/// IW4L_T6_TAGPROBE=<tag>@<seconds>: a test aid; at that time each player's
/// tag (what `gettagorigin` answers) is logged against where he stands, with
/// his body model. The egg reward flies to J_SpineLower: his waist, not his
/// feet.
fn tag_probe(world: &mut bevy_ecs::prelude::World, now: i64) {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};
    static AT: OnceLock<Option<(String, i64)>> = OnceLock::new();
    static DONE: AtomicBool = AtomicBool::new(false);
    let Some((tag, at)) = AT.get_or_init(|| {
        let v = std::env::var("IW4L_T6_TAGPROBE").ok()?;
        let (t, s) = v.split_once('@')?;
        Some((t.trim().to_owned(), (s.trim().parse::<f32>().ok()? * 1000.0) as i64))
    }) else {
        return;
    };
    if now < *at || DONE.swap(true, Ordering::Relaxed) {
        return;
    }
    let players: Vec<(u32, String)> =
        world.resource::<super::Zm>().players.iter().map(|(n, p)| (*n, p.model.clone())).collect();
    for (n, model) in players {
        let feet = super::frame(world).player(crate::world::ClientId(n)).map(|ps| ps.origin);
        let on = super::tags::tag_world(world, n, tag).map(|(o, _)| o);
        diag::info!(Sim, "bo2zm t6 highrise tag probe: player {n} model \"{model}\" stands at {feet:?}, {tag} at {on:?}");
    }
}

/// The easter egg's step 2 after a lion ball is shot (zm_highrise_sq_slb.gsc
/// `snipe_balls_watch_ball`): BO2 deletes the ball, waits half a second in
/// the same thread, then has the dragon's tail spark (`clientnotify` of the
/// ball's `script_noteworthy`), shows the matching Sliquifier ball
/// (`sq_sliquify_r`, `sq_sliquify_m` for `m_drg_tail`) with a flash, threads
/// its pickup (`lion_ball_enable_pickup`) and plays the first or second
/// ball's line. BO2 lets that thread run on after its ball is gone; ours ends
/// it at the delete (NEEDS-MAIN 2026-10-10 highrise, the VM's free_object,
/// WAITING), so this does the half second's work with BO2's own functions.
/// It stands down by itself once the VM keeps the thread: the Sliquifier
/// ball is then already shown.
#[derive(bevy_ecs::prelude::Resource, Default)]
struct LionBalls {
    level: Option<gsc_t6::ObjRef>,
    looked: bool,
    seen: Vec<(gsc_t6::ObjRef, String)>,
    due: Vec<(i64, String)>,
}

fn lion_balls(world: &mut bevy_ecs::prelude::World, now: i64) {
    use gsc_t6::{Key, Value};
    if world.resource::<super::Zm>().map != "zm_highrise" {
        return;
    }
    let mut b = std::mem::take(&mut *world.get_resource_or_insert_with(LionBalls::default));
    super::with_vm(world, |vm, world| {
        if b.level != Some(vm.level) {
            b = LionBalls { level: Some(vm.level), ..LionBalls::default() };
        }
        let (tn, sn) = (vm.intern("targetname"), vm.intern("script_noteworthy"));
        if !b.looked {
            b.looked = true;
            b.seen = world
                .resource::<super::Zm>()
                .ents
                .values()
                .filter_map(|e| e.obj)
                .filter(|&o| vm.to_text(&vm.raw_field(o, tn)) == "sq_dragon_lion_ball")
                .map(|o| (o, vm.to_text(&vm.raw_field(o, sn))))
                .collect();
        }
        // A ball gone during the egg's step 2 was shot (BO2 deletes them
        // otherwise only in sq_easy_cleanup, before any step).
        let stage = vm.intern("_cur_stage_name");
        let slb = vm.to_text(&vm.raw_field(vm.level, stage)) == "slb";
        let (gone, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut b.seen).into_iter().partition(|(o, _)| !vm.alive(*o));
        b.seen = kept;
        if slb {
            b.due.extend(gone.into_iter().map(|(_, tail)| (now + 500, tail)));
        }
        let (ready, wait): (Vec<_>, Vec<_>) = std::mem::take(&mut b.due).into_iter().partition(|(t, _)| *t <= now);
        b.due = wait;
        for (_, tail) in ready {
            let want = if tail == "m_drg_tail" { "sq_sliquify_m" } else { "sq_sliquify_r" };
            let Some((n, o, origin, hidden)) = world
                .resource::<super::Zm>()
                .ents
                .iter()
                .find(|(_, e)| e.obj.is_some_and(|o| vm.to_text(&vm.raw_field(o, sn)) == want))
                .and_then(|(&n, e)| Some((n, e.obj?, e.origin, e.hidden)))
            else {
                continue;
            };
            let pick = vm.intern("can_pickup");
            if !hidden || gsc_t6::truthy(&vm.raw_field(o, pick)) {
                continue;
            }
            let lvl = Value::Object(vm.level);
            let s = vm.string(&tail);
            vm.spawn_named(world, "maps/mp/_utility", "clientnotify", lvl.clone(), vec![s]);
            let (fx, flash) = (vm.intern("_effect"), vm.intern("sidequest_flash"));
            let id = match vm.raw_field(vm.level, fx) {
                Value::Array(a) => a.get(&Key::Str(flash)),
                _ => None,
            };
            if let Some(name) = id.and_then(|v| v.as_int()).and_then(|id| super::natives_fx::fx_name(world, id)) {
                super::natives_fx::effect(world, &name, origin, [0.0, 0.0, 1.0]);
            }
            if let Some(e) = world.resource_mut::<super::Zm>().ents.get_mut(&n) {
                e.hidden = false;
                e.ghosted = false;
            }
            vm.spawn_named(world, "maps/mp/zm_highrise_sq_slb", "lion_ball_enable_pickup", Value::Object(o), vec![]);
            let flags = vm.intern("flag");
            let first = vm.intern("sq_slb_first_ball_sniped");
            let sniped = match vm.raw_field(vm.level, flags) {
                Value::Array(a) => a.get(&Key::Str(first)).is_some_and(|v| gsc_t6::truthy(&v)),
                _ => false,
            };
            if sniped {
                vm.spawn_named(world, "maps/mp/zm_highrise_sq_slb", "vo_maxis_atd_ball2_sniped", lvl, vec![]);
            } else {
                let f = vm.string("sq_slb_first_ball_sniped");
                vm.spawn_named(world, "common_scripts/utility", "flag_set", lvl.clone(), vec![f]);
                vm.spawn_named(world, "maps/mp/zm_highrise_sq_slb", "vo_atd_ball1_sniped", lvl, vec![]);
            }
            diag::info!(
                Sim,
                "bo2zm t6 highrise: lion ball {tail} shot, Sliquifier ball {want} up for pickup half a second on"
            );
        }
    });
    *world.resource_mut::<LionBalls>() = b;
}

/// IW4L_T6_SQ=<seconds>: a test aid for the easter egg's first step
/// (zm_highrise_sq_atd.gsc), which BO2 gives four players (`bot dummy 3`
/// adds three). From that time player k (0-3) is kept standing in car k's
/// roof trigger (`elevator_bldg1b/1d/3b/3c_trigger`, riding with the car)
/// until the cars' dragons light (flag `sq_atd_elevator_activated`); then
/// player 0 steps on a floor dragon (`trig_atd_drg_puzzle`, an order BO2
/// shuffles each game) every 3 s, learning the order from
/// `level.sq_atd_cur_drg` as a player would (a wrong one sets it back to
/// 0). Step 2 (zm_highrise_sq_slb.gsc, stage `slb`): player 0 gets the
/// DSR-50 and shoots each lion ball (`sq_dragon_lion_ball`) from a floor
/// spot that sees it, aiming down the sight. The flags, the count and the
/// balls left are logged every 3 s.
fn sq_test(world: &mut bevy_ecs::prelude::World, now: i64) {
    use gsc_t6::Value;
    use std::sync::{Mutex, OnceLock, PoisonError};
    struct Drg {
        order: Vec<usize>,
        wrong: Vec<usize>,
        pending: Option<(usize, i64)>,
        next: i64,
    }
    static AT: OnceLock<Option<i64>> = OnceLock::new();
    static DRG: Mutex<Drg> = Mutex::new(Drg { order: Vec::new(), wrong: Vec::new(), pending: None, next: 0 });
    let Some(at) = *AT.get_or_init(|| {
        std::env::var("IW4L_T6_SQ")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .map(|s| (s * 1000.0) as i64)
    }) else {
        return;
    };
    if now < at {
        return;
    }
    const ROOFS: [&str; 4] = [
        "elevator_bldg1b_trigger",
        "elevator_bldg1d_trigger",
        "elevator_bldg3b_trigger",
        "elevator_bldg3c_trigger",
    ];
    let tick = i64::from(crate::MATCH_TICK_MS);
    let found = super::with_vm(world, |vm, world| {
        let tn = vm.intern("targetname");
        let named = |vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, n: &str, class: &str| -> Vec<super::Ent> {
            world
                .resource::<super::Zm>()
                .ents
                .values()
                .filter(|e| e.classname.starts_with(class) && e.obj.is_some_and(|o| vm.to_text(&vm.raw_field(o, tn)) == n))
                .cloned()
                .collect()
        };
        let roofs: Vec<Option<super::Ent>> =
            ROOFS.iter().map(|n| named(vm, n, "trigger").into_iter().next()).collect();
        let drgs = named(vm, "trig_atd_drg_puzzle", "trigger");
        let balls = named(vm, "sq_dragon_lion_ball", "script_model");
        let f = vm.intern("flag");
        let flags = match vm.raw_field(vm.level, f) {
            Value::Array(a) => a,
            _ => return None,
        };
        let flag = |vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, n: &str| {
            flags.get(&gsc_t6::Key::Str(vm.intern(n))).is_some_and(|v| gsc_t6::truthy(&v))
        };
        let on: Vec<bool> = (0..4).map(|i| flag(vm, &format!("sq_atd_elevator{i}"))).collect();
        let lit = flag(vm, "sq_atd_elevator_activated");
        let done = flag(vm, "sq_atd_drg_puzzle_complete");
        let cur_k = vm.intern("sq_atd_cur_drg");
        let cur = match vm.raw_field(vm.level, cur_k) {
            Value::Int(n) => n as i64,
            _ => -1,
        };
        let stage_k = vm.intern("_cur_stage_name");
        let stage = vm.to_text(&vm.raw_field(vm.level, stage_k));
        Some((roofs, drgs, balls, on, lit, done, cur, stage))
    })
    .flatten();
    let Some((roofs, mut drgs, mut balls, on, lit, done, cur, stage)) = found else {
        return;
    };
    // Stand a player in a trigger: on the floor under its middle when that
    // touches it, else with its middle at his waist.
    let stand = |world: &mut bevy_ecs::prelude::World, client: u32, trig: &super::Ent| {
        let c = super::triggers::center(world, trig);
        let (mins, maxs) = (crate::bullet_collision::PLAYER_MINS, crate::bullet_collision::PLAYER_MAXS);
        let from = [c[0], c[1], c[2] + 20.0];
        let to = [c[0], c[1], c[2] - 200.0];
        let t = super::frame(world).trace_world(from, to, mins, maxs, crate::bullet_collision::MASK_PLAYER_SOLID);
        let floor: [f32; 3] = std::array::from_fn(|i| from[i] + (to[i] - from[i]) * t.fraction);
        let lo = std::array::from_fn(|k| floor[k] + mins[k]);
        let hi = std::array::from_fn(|k| floor[k] + maxs[k]);
        let feet = if t.startsolid == 0 && t.fraction < 1.0 && super::triggers::touches(world, trig, lo, hi) {
            floor
        } else {
            [c[0], c[1], c[2] - 35.0]
        };
        super::teleport_player(world, crate::world::ClientId(client), feet);
        feet
    };
    let clients: Vec<u32> = world.resource::<super::Zm>().players.keys().copied().collect();
    if !lit {
        for (k, client) in clients.iter().take(4).enumerate() {
            if let Some(Some(trig)) = roofs.get(k) {
                stand(world, *client, trig);
            }
        }
    } else if !done {
        // The same order every run: by place.
        drgs.sort_by_key(|e| e.origin.map(|v| v as i64));
        let mut d = DRG.lock().unwrap_or_else(PoisonError::into_inner);
        if now >= d.next && !drgs.is_empty() {
            if let Some((idx, before)) = d.pending.take() {
                if cur == before + 1 && d.order.len() as i64 == before {
                    d.order.push(idx);
                    d.wrong.clear();
                } else if cur == 0 && d.order.len() as i64 == before {
                    d.wrong.push(idx);
                }
            }
            let pos = usize::try_from(cur).unwrap_or(0);
            let target = if pos < d.order.len() {
                Some(d.order[pos])
            } else {
                (0..drgs.len()).find(|i| !d.order.contains(i) && !d.wrong.contains(i))
            };
            if let Some(i) = target {
                let feet = stand(world, clients.first().copied().unwrap_or(0), &drgs[i]);
                diag::info!(
                    Sim,
                    "bo2zm t6 sq test: dragon {i} (at {:.0} {:.0} {:.0}), count {cur}, order {:?}, wrong {:?}",
                    feet[0],
                    feet[1],
                    feet[2],
                    d.order,
                    d.wrong
                );
                d.pending = Some((i, cur));
            }
            d.next = now + 3000;
        }
    } else if stage == "slb" && !balls.is_empty() {
        // The stage name stays "slb" after step 2, so this part only runs
        // while a lion ball is left; then steps 3-5 take over.
        struct Slb {
            given: bool,
            aim: Option<([f32; 3], [f32; 3], i64)>,
            next: i64,
            tries: usize,
        }
        static SLB: Mutex<Slb> = Mutex::new(Slb { given: false, aim: None, next: 0, tries: 0 });
        let client = crate::world::ClientId(clients.first().copied().unwrap_or(0));
        let mut s = SLB.lock().unwrap_or_else(PoisonError::into_inner);
        if !s.given {
            s.given = true;
            s.next = now + 4000;
            super::with_vm(world, |vm, world| {
                if let Some(p) = world.resource::<super::Zm>().players.get(&client.0).map(|p| p.obj) {
                    let g = vm.string("dsr50_zm");
                    vm.spawn_named(world, "maps/mp/zombies/_zm_weapons", "weapon_give", Value::Object(p), vec![g]);
                }
            });
            diag::info!(Sim, "bo2zm t6 sq test: given dsr50_zm for the lion balls");
        }
        if now % 2000 < tick {
            let mut f = super::frame(world);
            if let Some(w) = f.player(client).map(|p| p.weapon) {
                crate::script_player::set_ammo_stock(&mut f, client, w, 40);
            }
        }
        // A new try every 4 s: stand, aim down the sight from 0.5 s, fire
        // once at 1.5 s.
        if now >= s.next {
            s.next = now + 4000;
            s.aim = None;
            balls.sort_by_key(|e| e.origin.map(|v| v as i64));
            if let Some(ball) = balls.first() {
                let b = ball.origin;
                let n = s.tries;
                s.tries += 1;
                match sq_vantage_nth(world, b, &[300.0, 450.0, 200.0, 650.0, 900.0], n) {
                    Some(feet) => {
                        super::teleport_player(world, client, feet);
                        s.aim = Some((b, feet, now));
                        diag::info!(
                            Sim,
                            "bo2zm t6 sq test: ball at ({:.0} {:.0} {:.0}), shooting from ({:.0} {:.0} {:.0})",
                            b[0],
                            b[1],
                            b[2],
                            feet[0],
                            feet[1],
                            feet[2]
                        );
                    }
                    None => diag::warn!(Sim, "bo2zm t6 sq test: no floor spot sees the ball at ({:.0} {:.0} {:.0})", b[0], b[1], b[2]),
                }
            }
        }
        if let Some((ball, feet, from)) = s.aim {
            let eye = [feet[0], feet[1], feet[2] + 60.0];
            super::set_player_view(world, client, super::autoplay::look(eye, ball));
            let since = now - from;
            let mut req = world.resource_mut::<crate::step::StepRequest>();
            for (id, cmd) in &mut req.input.cmds {
                if *id == client {
                    if (500..3000).contains(&since) {
                        cmd.buttons |= playerstate_iw4::buttons::ADS;
                    }
                    if (1500..1600).contains(&since) {
                        cmd.buttons |= playerstate_iw4::buttons::ATTACK;
                    }
                }
            }
        }
    } else {
        if sq_side_maxis() {
            sq_maxis(world, now, &clients, &stage);
        }
        sq_richtofen(world, now, &clients, &stage);
    }
    if (now - at) % 3000 < tick {
        let places: Vec<String> = clients
            .iter()
            .map(|c| {
                let o = super::frame(world).player(crate::world::ClientId(*c)).map(|p| p.origin).unwrap_or_default();
                format!("{c}:({:.0} {:.0} {:.0})", o[0], o[1], o[2])
            })
            .collect();
        diag::info!(
            Sim,
            "bo2zm t6 sq test: stage {stage}, roofs {on:?}, lit {lit}, dragons {cur} done {done}, balls {}, players {}",
            balls.len(),
            places.join(" ")
        );
    }
}

/// Die Rise's four `pts_ghoul` spots, where Richtofen's Trample Steams go,
/// and the way each must face (map data).
const SQ_GHOULS: [([f32; 3], f32); 4] = [
    ([1841.5, -290.0, 2880.0], 77.5),
    ([1664.5, -81.5, 2880.0], 55.9),
    ([2449.0, 343.5, 3016.0], 124.4),
    ([2061.0, 1586.5, 3392.0], 279.1),
];

/// The egg test's steps 3-5, Richtofen's side. Stage `ssp_1`
/// (zm_highrise_sq_ssp.gsc): player 0 shoots each sliquify ball
/// (`sq_sliquify_ball`) with the Sliquifier until its flag is set, never
/// picking it up. Stage `pts_1` (zm_highrise_sq_pts.gsc): player k puts a
/// Trample Steam on ghoul spot k, standing on it facing its way, then
/// stands just past it so the zombies coming at him cross it (16 flings;
/// after 150 s without them the players stand on their pads, which BO2
/// counts too). Then the tower (zm_highrise_sq.gsc): player 0 gets the
/// Galvaknuckles and punches the leg BO2 waits for next
/// (`level.a_wind_order[level.n_cur_leg]`, the order the tiles show).
fn sq_richtofen(world: &mut bevy_ecs::prelude::World, now: i64, clients: &[u32], stage: &str) {
    use gsc_t6::Value;
    use std::sync::{Mutex, PoisonError};
    struct Rich {
        slip: bool,
        ball: usize,
        done: Vec<usize>,
        presses: u32,
        flags_done: usize,
        aim: Option<([f32; 3], [f32; 3], i64)>,
        spot: usize,
        next: i64,
        place_at: Option<i64>,
        placed: Option<i64>,
        tries: u32,
        stands: Vec<([f32; 3], f32)>,
        set_at: Option<i64>,
        on_pads: bool,
        tazer: Option<i64>,
        log: i64,
    }
    static RICH: Mutex<Rich> = Mutex::new(Rich {
        slip: false,
        ball: 0,
        done: Vec::new(),
        presses: 0,
        flags_done: 0,
        aim: None,
        spot: 0,
        next: 0,
        place_at: None,
        placed: None,
        tries: 0,
        stands: Vec::new(),
        set_at: None,
        on_pads: false,
        tazer: None,
        log: 0,
    });
    const FLAGS: [&str; 7] = [
        "ssp1_ball0_complete",
        "ssp1_ball1_complete",
        "ssp1_complete",
        "pts_1_springpads_placed",
        "sq_branch_complete",
        "sq_tower_active",
        "sq_ball_picked_up",
    ];
    let found = super::with_vm(world, |vm, world| {
        let tn = vm.intern("targetname");
        let sn = vm.intern("script_noteworthy");
        let mut balls = Vec::new();
        let mut legs = Vec::new();
        for e in world.resource::<super::Zm>().ents.values() {
            let Some(o) = e.obj else { continue };
            match vm.to_text(&vm.raw_field(o, tn)).as_str() {
                "sq_sliquify_ball" if !e.hidden => balls.push(e.clone()),
                "sq_tower_leg" => legs.push((vm.to_text(&vm.raw_field(o, sn)), e.clone())),
                _ => {}
            }
        }
        let f = vm.intern("flag");
        let Value::Array(flags) = vm.raw_field(vm.level, f) else {
            return None;
        };
        let fl: [bool; 7] = std::array::from_fn(|i| {
            let k = vm.intern(FLAGS[i]);
            flags.get(&gsc_t6::Key::Str(k)).is_some_and(|v| gsc_t6::truthy(&v))
        });
        let ints: [i64; 3] = std::array::from_fn(|i| {
            let k = vm.intern(["n_launched_zombies", "n_cur_leg", "sq_leg_punches"][i]);
            match vm.raw_field(vm.level, k) {
                Value::Int(n) => i64::from(n),
                _ => -1,
            }
        });
        // In the order BO2's foreach walks it: tower_punch_watcher builds the
        // legs it waits for (`a_leg_trigs`) with a foreach over this array.
        let wk = vm.intern("a_wind_order");
        let wind: Vec<String> = match vm.raw_field(vm.level, wk) {
            // first_key/next_key, as foreach walks it (keys() is
            // getarraykeys' order, newest first).
            Value::Array(a) => {
                let a = a.read();
                let mut out = Vec::new();
                let mut k = a.first_key();
                while let Some(key) = k {
                    if let Some(v) = a.get(&key) {
                        out.push(vm.to_text(v));
                    }
                    k = a.next_key(&key);
                }
                out
            }
            _ => Vec::new(),
        };
        let mk = vm.intern("current_melee_weapon");
        let melee = world
            .resource::<super::Zm>()
            .players
            .get(&clients.first().copied().unwrap_or(0))
            .map(|p| vm.to_text(&vm.raw_field(p.obj, mk)))
            .unwrap_or_default();
        // Only steps 1-2 set level._cur_stage_name; after the lion balls it
        // stays "slb", so step 3 is known by slb's `completed` in the
        // sidequest table (_zm_sidequests.gsc stage_completed_internal).
        let [zs, sq, st, slb, cp] = ["_zombie_sidequests", "sq", "stages", "slb", "completed"].map(|s| vm.intern(s));
        let field = |v: Option<Value>, k| match v {
            Some(Value::Object(o)) => Some(vm.raw_field(o, k)),
            _ => None,
        };
        let sq = match vm.raw_field(vm.level, zs) {
            Value::Array(a) => a.get(&gsc_t6::Key::Str(sq)),
            _ => None,
        };
        let slb_done = match field(sq, st) {
            Some(Value::Array(s)) => field(s.get(&gsc_t6::Key::Str(slb)), cp).is_some_and(|v| gsc_t6::truthy(&v)),
            _ => false,
        };
        Some((balls, legs, fl, ints, wind, melee, slb_done))
    })
    .flatten();
    let Some((mut balls, legs, fl, ints, wind, melee, slb_done)) = found else {
        return;
    };
    let stage = if slb_done && stage == "slb" { "ssp/pts" } else { stage };
    let [b0, b1, ssp_done, pads_set, branch, tower, picked] = fl;
    let [launched, cur_leg, punches] = ints;
    let tick = i64::from(crate::MATCH_TICK_MS);
    let client = crate::world::ClientId(clients.first().copied().unwrap_or(0));
    let press = |world: &mut bevy_ecs::prelude::World, client: crate::world::ClientId, b: u32| {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= b;
            }
        }
    };
    // IW4L_T6_SQ_SIDE=maxis: steps 3-4 are Maxis's (sq_maxis); the tower
    // is the same for both.
    let maxis = sq_side_maxis();
    let mut r = RICH.lock().unwrap_or_else(PoisonError::into_inner);
    if !maxis && !ssp_done && slb_done && !balls.is_empty() {
        if !r.slip {
            r.slip = true;
            r.next = now + 3000;
            r.flags_done = usize::from(b0) + usize::from(b1);
            super::with_vm(world, |vm, world| {
                if let Some(p) = world.resource::<super::Zm>().players.get(&client.0).map(|p| p.obj) {
                    let g = vm.string("slipgun_zm");
                    vm.spawn_named(world, "maps/mp/zombies/_zm_weapons", "weapon_give", Value::Object(p), vec![g]);
                }
            });
            diag::info!(Sim, "bo2zm t6 sq test: given slipgun_zm for the sliquify balls");
        }
        if now % 2000 < tick {
            let mut f = super::frame(world);
            if let Some(w) = f.player(client).map(|p| p.weapon) {
                crate::script_player::set_ammo_stock(&mut f, client, w, 60);
            }
        }
        // Which ball a flag belongs to is BO2's array order: the one being
        // shot when a flag comes on is done.
        let n_done = usize::from(b0) + usize::from(b1);
        if n_done > r.flags_done {
            r.flags_done = n_done;
            let b = r.ball;
            if !r.done.contains(&b) {
                r.done.push(b);
            }
            r.next = now;
            diag::info!(Sim, "bo2zm t6 sq test: sliquify ball {b} done after {} presses, flags {b0} {b1}", r.presses);
            r.presses = 0;
        }
        balls.sort_by_key(|e| e.origin.map(|v| v as i64));
        // A new spot every 6 s (another side of the ball each time, past a
        // model in the way); the other ball after 60 presses.
        if now >= r.next {
            r.next = now + 6000;
            r.aim = None;
            if n_done < 2 && (0..balls.len()).all(|i| r.done.contains(&i)) {
                r.done.clear();
            }
            let left: Vec<usize> = (0..balls.len()).filter(|i| !r.done.contains(i)).collect();
            if !left.is_empty() && (!left.contains(&r.ball) || r.presses >= 60) {
                let at = left.iter().position(|&i| i == r.ball).map_or(0, |p| (p + 1) % left.len());
                r.ball = left[at];
                r.presses = 0;
            }
            if let Some(b) = balls.get(r.ball).map(|e| e.origin) {
                let n = r.spot;
                r.spot += 1;
                match sq_vantage_nth(world, b, &[160.0, 240.0, 120.0, 320.0], n) {
                    Some(feet) => {
                        super::teleport_player(world, client, feet);
                        r.aim = Some((b, feet, now));
                        diag::info!(
                            Sim,
                            "bo2zm t6 sq test: sliquify ball {} at ({:.0} {:.0} {:.0}) from ({:.0} {:.0} {:.0}), {} presses",
                            r.ball,
                            b[0],
                            b[1],
                            b[2],
                            feet[0],
                            feet[1],
                            feet[2],
                            r.presses
                        );
                    }
                    None => diag::warn!(
                        Sim,
                        "bo2zm t6 sq test: no floor spot sees the sliquify ball at ({:.0} {:.0} {:.0})",
                        b[0],
                        b[1],
                        b[2]
                    ),
                }
            }
        }
        if let Some((ball, feet, from)) = r.aim {
            let eye = [feet[0], feet[1], feet[2] + 60.0];
            super::set_player_view(world, client, super::autoplay::look(eye, ball));
            let since = now - from;
            if since >= 700 && since % 700 < 100 {
                if since % 700 < tick {
                    r.presses += 1;
                }
                press(world, client, playerstate_iw4::buttons::ATTACK);
            }
        }
    } else if !maxis && ssp_done && !branch {
        let n = clients.len().min(SQ_GHOULS.len());
        let place_at = *r.place_at.get_or_insert(now + 17000);
        if !pads_set {
            if r.placed.is_none() && now >= place_at - 1500 {
                // Pitch 0: BO2 wants the pad's forward within 0.98 of the
                // spot's.
                for (k, c) in clients.iter().take(n).enumerate() {
                    let (spot, yaw) = SQ_GHOULS[k];
                    let feet = sq_floor(world, spot, 200.0).unwrap_or(spot);
                    super::teleport_player(world, crate::world::ClientId(*c), feet);
                    super::set_player_view(world, crate::world::ClientId(*c), [0.0, yaw, 0.0]);
                }
                if now >= place_at {
                    let ok = super::with_vm(world, |vm, world| {
                        let mut ok = 0;
                        for c in clients.iter().take(n) {
                            let Some(p) = world.resource::<super::Zm>().players.get(c).map(|p| p.obj) else {
                                continue;
                            };
                            let e = vm.string("equip_springpad_zm");
                            let lvl = Value::Object(vm.level);
                            let f = "equipment_drop_to_planted";
                            if vm
                                .spawn_named(world, "maps/mp/zombies/_zm_equipment", f, lvl, vec![e, Value::Object(p)])
                                .is_some()
                            {
                                ok += 1;
                            }
                        }
                        ok
                    })
                    .unwrap_or(0);
                    r.placed = Some(now);
                    diag::info!(Sim, "bo2zm t6 sq test: {ok} of {n} Trample Steams put on the ghoul spots (try {})", r.tries + 1);
                }
            } else if let Some(t) = r.placed {
                if now >= t + 10000 && r.tries < 3 {
                    r.tries += 1;
                    r.placed = None;
                    r.place_at = Some(now + 1500);
                    diag::warn!(Sim, "bo2zm t6 sq test: not all ghoul spots took a Trample Steam, again");
                }
            }
        } else {
            let set_at = *r.set_at.get_or_insert(now);
            if r.stands.is_empty() {
                // Just past the pad on its own roof, else just behind it.
                for &(spot, yaw) in SQ_GHOULS.iter().take(n) {
                    let a = yaw.to_radians();
                    let feet = [50.0f32, 30.0, -60.0]
                        .into_iter()
                        .find_map(|d| {
                            sq_floor(world, [spot[0] + a.cos() * d, spot[1] + a.sin() * d, spot[2]], 200.0)
                                .filter(|f| (f[2] - spot[2]).abs() < 24.0)
                        })
                        .unwrap_or(spot);
                    r.stands.push((feet, yaw + 180.0));
                }
                diag::info!(Sim, "bo2zm t6 sq test: pads in place, the players stand past them: {:?}", r.stands);
            }
            if !r.on_pads && now >= set_at + 150_000 && launched < 16 {
                r.on_pads = true;
                diag::info!(Sim, "bo2zm t6 sq test: {launched} flings in 150 s, the players stand on their pads");
            }
            for (k, c) in clients.iter().take(n).enumerate() {
                let (feet, yaw) = if r.on_pads {
                    let (spot, yaw) = SQ_GHOULS[k];
                    (sq_floor(world, spot, 200.0).unwrap_or(spot), yaw)
                } else {
                    r.stands[k]
                };
                super::teleport_player(world, crate::world::ClientId(*c), feet);
                super::set_player_view(world, crate::world::ClientId(*c), [10.0, yaw, 0.0]);
            }
        }
    } else if branch && !tower {
        let t0 = match r.tazer {
            Some(t) => t,
            None => {
                r.tazer = Some(now);
                super::with_vm(world, |vm, world| {
                    if let Some(p) = world.resource::<super::Zm>().players.get(&client.0).map(|p| p.obj) {
                        let g = vm.string("tazer_knuckles_zm");
                        let f = "give_melee_weapon_by_name";
                        vm.spawn_named(world, "maps/mp/zombies/_zm_melee_weapon", f, Value::Object(p), vec![g]);
                    }
                });
                diag::info!(Sim, "bo2zm t6 sq test: given tazer_knuckles_zm for the tower legs, order {wind:?}");
                now
            }
        };
        if (t0 + 7000..t0 + 7000 + tick).contains(&now) && !melee.starts_with("tazer_knuckles") {
            diag::warn!(Sim, "bo2zm t6 sq test: melee weapon still {melee:?} 7 s after give_melee_weapon_by_name, set by the test");
            super::with_vm(world, |vm, world| {
                if let Some(p) = world.resource::<super::Zm>().players.get(&client.0).map(|p| p.obj) {
                    let k = vm.intern("current_melee_weapon");
                    let g = vm.string("tazer_knuckles_zm");
                    vm.set_field(world, p, k, g);
                }
            });
        }
        // One punch every 3 s, always at the leg BO2 waits for next, from
        // a spot 45 out that turns a little each try.
        let leg_no = usize::try_from(cur_leg).ok().filter(|&i| i < wind.len());
        if let (true, Some(i)) = (now >= t0 + 8000 && !legs.is_empty(), leg_no) {
            let slot = (now - t0 - 8000) / 3000;
            let since = (now - t0 - 8000) % 3000;
            let want = format!("sq_tower_{}", wind[i]);
            let mut mid = [0.0f32; 3];
            let mut aim = None;
            for (name, leg) in &legs {
                let c = super::triggers::center(world, leg);
                for k in 0..3 {
                    mid[k] += c[k] / legs.len() as f32;
                }
                if *name == want {
                    aim = Some((c, leg));
                }
            }
            if let Some((c, leg)) = aim {
                let base = (c[1] - mid[1]).atan2(c[0] - mid[0]);
                let offs = [0.0f32, 0.4, -0.4, 0.8, -0.8];
                let start = usize::try_from(slot).unwrap_or(0) % offs.len();
                let feet = (0..offs.len()).find_map(|j| {
                    let a = base + offs[(start + j) % offs.len()];
                    sq_floor(world, [c[0] + a.cos() * 45.0, c[1] + a.sin() * 45.0, c[2]], 300.0)
                });
                if let Some(feet) = feet {
                    let eye = [feet[0], feet[1], feet[2] + 60.0];
                    // Level with the eye when the leg reaches that high.
                    let level = [c[0], c[1], eye[2]];
                    let lo = [level[0] - 1.0, level[1] - 1.0, level[2] - 1.0];
                    let hi = [level[0] + 1.0, level[1] + 1.0, level[2] + 1.0];
                    let target = if super::triggers::touches(world, leg, lo, hi) { level } else { c };
                    super::teleport_player(world, client, feet);
                    super::set_player_view(world, client, super::autoplay::look(eye, target));
                    if (1200..1200 + tick).contains(&since) {
                        press(world, client, weapon_iw4::BUTTON_MELEE);
                        diag::info!(
                            Sim,
                            "bo2zm t6 sq test: punch {want} (leg {i} of {wind:?}) from ({:.0} {:.0} {:.0}), punches before {punches}",
                            feet[0],
                            feet[1],
                            feet[2]
                        );
                    }
                } else if since < tick {
                    diag::warn!(Sim, "bo2zm t6 sq test: no floor by {want} at ({:.0} {:.0} {:.0})", c[0], c[1], c[2]);
                }
            }
        }
    }
    if ((!maxis && slb_done) || branch) && now >= r.log {
        r.log = now + 3000;
        diag::info!(
            Sim,
            "bo2zm t6 sq test (Richtofen): stage {stage}, sliquify balls shown {} flags {b0} {b1} done {ssp_done} picked up {picked}, \
             pads {pads_set} flings {launched}, branch {branch}, tower order {wind:?} leg {cur_leg} punches {punches} melee {melee:?} active {tower}",
            balls.len()
        );
    }
}

/// IW4L_T6_SQ_SIDE=maxis: the egg test takes Maxis's side after step 2.
fn sq_side_maxis() -> bool {
    static M: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *M.get_or_init(|| std::env::var("IW4L_T6_SQ_SIDE").is_ok_and(|s| s.trim().eq_ignore_ascii_case("maxis")))
}

/// Die Rise's four `pts_lion` spots and the way each faces (map data), as
/// two pairs: 0 with 1, 2 with 3 (each spot names its buddy).
const SQ_LIONS: [([f32; 3], f32); 4] = [
    ([2472.0, -592.0, 2880.0], 117.3),
    ([1400.0, 1528.0, 3392.0], 297.3),
    ([2816.0, -40.0, 2880.0], 123.2),
    ([2003.0, 1200.5, 3216.0], 303.5),
];

/// The egg test's steps 3-4 on Maxis's side (IW4L_T6_SQ_SIDE=maxis).
/// Stage `ssp_2`: BO2 counts the zombies that die in a row touching
/// `corpse_room_trigger` (one dying anywhere else sets it back to 0); at 15
/// it waits for that trigger's "damage" from a player holding the upgraded
/// ballistic knife, type MOD_IMPACT. So the players stand in the room,
/// player 0 kills each zombie that touches it (a pistol shot's damage),
/// then gets knife_ballistic_upgraded_zm and fires it into the room every
/// 3 s. Stage `pts_2`: player k puts a Trample Steam on lion spot k,
/// standing on it facing its way, then steps behind it; player 0 picks up
/// a lion ball (use, looking at it), which makes put-down triggers on both
/// spots of each pair with two pads; he uses one behind a pair's first pad
/// (generator 1), then the same with the other ball at the other pair.
fn sq_maxis(world: &mut bevy_ecs::prelude::World, now: i64, clients: &[u32], stage: &str) {
    use gsc_t6::Value;
    use std::sync::{Mutex, PoisonError};
    struct Max {
        stands: Vec<[f32; 3]>,
        next_kill: i64,
        kills: u32,
        knife: Option<i64>,
        shots: u32,
        standin: bool,
        place_at: Option<i64>,
        placed: Option<i64>,
        tries: u32,
        no_trig_since: Option<i64>,
        /// (holding a ball, since, try)
        step: Option<(bool, i64, u32)>,
        log: i64,
    }
    static MAX: Mutex<Max> = Mutex::new(Max {
        stands: Vec::new(),
        next_kill: 0,
        kills: 0,
        knife: None,
        shots: 0,
        standin: false,
        place_at: None,
        placed: None,
        tries: 0,
        no_trig_since: None,
        step: None,
        log: 0,
    });
    const FLAGS: [&str; 6] = [
        "ssp2_maxis_keep_going_said",
        "ssp2_corpses_in_place",
        "ssp2_resurrection_done",
        "pts_2_generator_1_started",
        "pts_2_generator_2_started",
        "sq_branch_complete",
    ];
    let mut cl = clients.to_vec();
    cl.sort_unstable();
    let me = cl.first().copied().unwrap_or(0);
    let found = super::with_vm(world, |vm, world| {
        let tn = vm.intern("targetname");
        let mut room = None;
        let mut balls = Vec::new();
        let mut pickups = Vec::new();
        let mut places = Vec::new();
        for e in world.resource::<super::Zm>().ents.values() {
            let Some(o) = e.obj else { continue };
            match vm.to_text(&vm.raw_field(o, tn)).as_str() {
                "corpse_room_trigger" if e.classname.starts_with("trigger") => room = Some(e.clone()),
                "sq_sliquify_ball" if !e.hidden => balls.push(e.clone()),
                "ball_pickup_trig" => pickups.push(e.clone()),
                "ball_place_trig" => places.push(e.clone()),
                _ => {}
            }
        }
        let f = vm.intern("flag");
        let Value::Array(flags) = vm.raw_field(vm.level, f) else {
            return None;
        };
        let fl: [bool; 6] = std::array::from_fn(|i| {
            let k = vm.intern(FLAGS[i]);
            flags.get(&gsc_t6::Key::Str(k)).is_some_and(|v| gsc_t6::truthy(&v))
        });
        let (hk, wk) = (vm.intern("zm_sq_has_ball"), vm.intern("currentweapon"));
        let (has_ball, weapon) = world
            .resource::<super::Zm>()
            .players
            .get(&me)
            .map(|p| (gsc_t6::truthy(&vm.raw_field(p.obj, hk)), vm.to_text(&vm.raw_field(p.obj, wk))))
            .unwrap_or_default();
        Some((room, balls, pickups, places, fl, has_ball, weapon))
    })
    .flatten();
    let Some((room, mut balls, pickups, mut places, fl, has_ball, weapon)) = found else {
        return;
    };
    let [said, in_place, revived, gen1, gen2, branch] = fl;
    if branch {
        return;
    }
    balls.sort_by_key(|e| e.origin.map(|v| v as i64));
    places.sort_by_key(|e| e.origin.map(|v| v as i64));
    let tick = i64::from(crate::MATCH_TICK_MS);
    let client = crate::world::ClientId(me);
    let hold = |world: &mut bevy_ecs::prelude::World, b: u32| {
        let mut req = world.resource_mut::<crate::step::StepRequest>();
        for (id, cmd) in &mut req.input.cmds {
            if *id == client {
                cmd.buttons |= b;
            }
        }
    };
    let stand_look = |world: &mut bevy_ecs::prelude::World, c: u32, feet: [f32; 3], to: [f32; 3]| {
        super::teleport_player(world, crate::world::ClientId(c), feet);
        let eye = [feet[0], feet[1], feet[2] + 60.0];
        super::set_player_view(world, crate::world::ClientId(c), super::autoplay::look(eye, to));
    };
    let hint = world.resource::<super::Zm>().hints.get(&me).cloned().unwrap_or_default();
    let mut m = MAX.lock().unwrap_or_else(PoisonError::into_inner);
    if !in_place || !revived {
        let Some(room) = room else {
            if now >= m.log {
                m.log = now + 3000;
                diag::warn!(Sim, "bo2zm t6 sq test (Maxis): no corpse_room_trigger");
            }
            return;
        };
        let c = super::triggers::center(world, &room);
        if m.stands.is_empty() {
            for k in 0..cl.len().min(4) {
                let a = k as f32 * std::f32::consts::FRAC_PI_2;
                let at = [c[0] + a.cos() * 40.0, c[1] + a.sin() * 40.0, c[2]];
                let feet = sq_floor(world, at, 400.0).unwrap_or(at);
                m.stands.push(feet);
            }
            let (mins, maxs) = (crate::bullet_collision::PLAYER_MINS, crate::bullet_collision::PLAYER_MAXS);
            let inside: Vec<bool> = m
                .stands
                .clone()
                .into_iter()
                .map(|f| {
                    let lo = std::array::from_fn(|i| f[i] + mins[i]);
                    let hi = std::array::from_fn(|i| f[i] + maxs[i]);
                    super::triggers::touches(world, &room, lo, hi)
                })
                .collect();
            diag::info!(
                Sim,
                "bo2zm t6 sq test (Maxis): corpse room middle ({:.0} {:.0} {:.0}), the players stand at {:?}, in the room {inside:?}",
                c[0],
                c[1],
                c[2],
                m.stands
            );
        }
        for (k, cid) in cl.iter().enumerate().take(m.stands.len()) {
            let feet = m.stands[k];
            stand_look(world, *cid, feet, c);
        }
        if !in_place && now >= m.next_kill {
            m.next_kill = now + 250;
            let living: Vec<(u32, [f32; 3], f32, f32, i32)> = {
                let zm = world.resource::<super::Zm>();
                zm.actors
                    .iter()
                    .filter(|(_, a)| a.alive)
                    .filter_map(|(n, a)| zm.ents.get(n).map(|e| (*n, e.origin, a.radius, a.height, a.health)))
                    .collect()
            };
            let mut hit = Vec::new();
            for (n, p, r, h, hp) in living {
                let lo = [p[0] - r, p[1] - r, p[2]];
                let hi = [p[0] + r, p[1] + r, p[2] + h];
                if super::triggers::touches(world, &room, lo, hi) {
                    hit.push((n, p, hp));
                }
            }
            if !hit.is_empty() {
                let k = super::with_vm(world, |vm, world| {
                    let Some(p) = world.resource::<super::Zm>().players.get(&me).map(|p| p.obj) else {
                        return 0;
                    };
                    for (n, o, hp) in &hit {
                        let at = [o[0], o[1], o[2] + 48.0];
                        let who = Value::Object(p);
                        super::actors::damage(
                            vm,
                            world,
                            *n,
                            who.clone(),
                            who,
                            hp + 100,
                            0,
                            "MOD_PISTOL_BULLET",
                            "m1911_zm",
                            at,
                            [0.0; 3],
                            "torso_upper",
                        );
                    }
                    hit.len()
                })
                .unwrap_or(0);
                m.kills += u32::try_from(k).unwrap_or(0);
            }
        } else if in_place {
            let t0 = match m.knife {
                Some(t) => t,
                None => {
                    m.knife = Some(now);
                    super::with_vm(world, |vm, world| {
                        if let Some(p) = world.resource::<super::Zm>().players.get(&me).map(|p| p.obj) {
                            let g = vm.string("knife_ballistic_upgraded_zm");
                            vm.spawn_named(world, "maps/mp/zombies/_zm_weapons", "weapon_give", Value::Object(p), vec![g]);
                        }
                    });
                    diag::info!(
                        Sim,
                        "bo2zm t6 sq test (Maxis): 15 in the room after {} kills, given knife_ballistic_upgraded_zm",
                        m.kills
                    );
                    now
                }
            };
            let since = now - t0 - 3000;
            if since >= 0 && since % 3000 < 150 {
                if since % 3000 < tick {
                    m.shots += 1;
                    diag::info!(Sim, "bo2zm t6 sq test (Maxis): knife shot {} into the room, holding {weapon:?}", m.shots);
                }
                hold(world, playerstate_iw4::buttons::ATTACK);
            }
            // IW4L_T6_SQ_STANDIN=1: past 3 shots, the room gets the
            // knife's hit itself (what BO2's engine sends a trigger_damage
            // a missile passes), to test what comes after. Not proof of the
            // knife step.
            if m.shots >= 3 && !m.standin && std::env::var_os("IW4L_T6_SQ_STANDIN").is_some() {
                m.standin = true;
                if let Some(o) = room.obj {
                    super::with_vm(world, |vm, world| {
                        if let Some(p) = world.resource::<super::Zm>().players.get(&me).map(|p| p.obj) {
                            let args = [
                                Value::Int(100),
                                Value::Object(p),
                                Value::Vec3([0.0, 0.0, -1.0]),
                                Value::Vec3(c),
                                vm.string("MOD_IMPACT"),
                            ];
                            vm.notify_str(world, o, "damage", &args);
                        }
                    });
                }
                diag::warn!(
                    Sim,
                    "bo2zm t6 sq test (Maxis): STAND-IN: corpse_room_trigger takes the knife's hit from the test (MOD_IMPACT, \
                     player 0 holding {weapon:?})"
                );
            }
        }
    } else {
        let n = cl.len().min(SQ_LIONS.len());
        let place_at = *m.place_at.get_or_insert(now + 3000);
        let Some(placed) = m.placed else {
            if now >= place_at - 1500 {
                for (k, cid) in cl.iter().take(n).enumerate() {
                    let (spot, yaw) = SQ_LIONS[k];
                    let feet = sq_floor(world, spot, 200.0).unwrap_or(spot);
                    super::teleport_player(world, crate::world::ClientId(*cid), feet);
                    super::set_player_view(world, crate::world::ClientId(*cid), [0.0, yaw, 0.0]);
                }
                if now >= place_at {
                    let ok = super::with_vm(world, |vm, world| {
                        let mut ok = 0;
                        for c in cl.iter().take(n) {
                            let Some(p) = world.resource::<super::Zm>().players.get(c).map(|p| p.obj) else {
                                continue;
                            };
                            let e = vm.string("equip_springpad_zm");
                            let lvl = Value::Object(vm.level);
                            let f = "equipment_drop_to_planted";
                            if vm
                                .spawn_named(world, "maps/mp/zombies/_zm_equipment", f, lvl, vec![e, Value::Object(p)])
                                .is_some()
                            {
                                ok += 1;
                            }
                        }
                        ok
                    })
                    .unwrap_or(0);
                    m.placed = Some(now);
                    diag::info!(
                        Sim,
                        "bo2zm t6 sq test (Maxis): {ok} of {n} Trample Steams put on the lion spots (try {})",
                        m.tries + 1
                    );
                }
            }
            return;
        };
        // A spot `d` behind lion spot k (the floor there, else level with
        // the spot), and the spot.
        let behind = |world: &mut bevy_ecs::prelude::World, k: usize, d: f32| -> ([f32; 3], [f32; 3]) {
            let (spot, yaw) = SQ_LIONS[k];
            let a = yaw.to_radians();
            let at = [spot[0] - a.cos() * d, spot[1] - a.sin() * d, spot[2]];
            let feet = sq_floor(world, at, 200.0).filter(|f| (f[2] - spot[2]).abs() < 24.0).unwrap_or(at);
            (feet, spot)
        };
        if now >= placed + 2000 {
            for (k, cid) in cl.iter().take(n).enumerate().skip(1) {
                let (feet, spot) = behind(world, k, 60.0);
                stand_look(world, *cid, feet, spot);
            }
        }
        if now < placed + 3000 || (gen1 && gen2) {
            // the pads settle, or both generators are going
        } else if !has_ball {
            if let Some((true, ..)) = m.step {
                diag::info!(Sim, "bo2zm t6 sq test (Maxis): ball put down, generators {gen1} {gen2}");
                m.step = None;
            }
            let (t0, tr) = match m.step {
                Some((false, t, tr)) => (t, tr),
                _ => {
                    m.step = Some((false, now, 0));
                    (now, 0)
                }
            };
            // A shown ball with its pick-up trigger by it, else any shown
            // ball.
            let trig_by = |b: [f32; 3]| {
                pickups
                    .iter()
                    .filter(|e| gsc_t6::math::length(gsc_t6::math::sub(e.origin, b)) < 40.0)
                    .min_by_key(|e| gsc_t6::math::length(gsc_t6::math::sub(e.origin, b)) as i64)
            };
            let ball = balls.iter().find(|e| trig_by(e.origin).is_some()).or(balls.first());
            if let Some(ball) = ball {
                let b = ball.origin;
                let radii = [[36.0f32, 52.0, 24.0], [52.0, 24.0, 36.0], [24.0, 36.0, 52.0]][tr as usize % 3];
                let target = match trig_by(b) {
                    Some(e) => super::triggers::center(world, e),
                    None => b,
                };
                if let Some(feet) = sq_vantage(world, b, &radii) {
                    stand_look(world, me, feet, target);
                    if (t0 + 1000..t0 + 1500).contains(&now) {
                        hold(world, playerstate_iw4::buttons::USE);
                    }
                    if (t0 + 1000..t0 + 1000 + tick).contains(&now) {
                        diag::info!(
                            Sim,
                            "bo2zm t6 sq test (Maxis): use on the ball at ({:.0} {:.0} {:.0}) from ({:.0} {:.0} {:.0}), hint {hint:?}, try {}",
                            b[0],
                            b[1],
                            b[2],
                            feet[0],
                            feet[1],
                            feet[2],
                            tr + 1
                        );
                    }
                }
                if now >= t0 + 3000 {
                    diag::warn!(Sim, "bo2zm t6 sq test (Maxis): no ball picked up, again");
                    m.step = Some((false, now, tr + 1));
                }
            }
        } else {
            let (t0, tr) = match m.step {
                Some((true, t, tr)) => (t, tr),
                _ => {
                    diag::info!(Sim, "bo2zm t6 sq test (Maxis): holding a ball, put-down triggers {}", places.len());
                    m.step = Some((true, now, 0));
                    (now, 0)
                }
            };
            // The put-down trigger on a pair's first spot (0 or 2), else
            // any.
            let near = |e: &super::Ent, k: usize| {
                let s = SQ_LIONS[k].0;
                ((e.origin[0] - s[0]).powi(2) + (e.origin[1] - s[1]).powi(2)).sqrt() < 40.0
            };
            let pick = places
                .iter()
                .find_map(|e| [0usize, 2].into_iter().find(|&k| near(e, k)).map(|k| (e, k)))
                .or_else(|| places.iter().find_map(|e| (0..SQ_LIONS.len()).find(|&k| near(e, k)).map(|k| (e, k))));
            match pick {
                Some((trig, k)) => {
                    m.no_trig_since = None;
                    let d = [44.0f32, 36.0, 56.0][tr as usize % 3];
                    let target = super::triggers::center(world, trig);
                    let (feet, _) = behind(world, k, d);
                    stand_look(world, me, feet, target);
                    if (t0 + 1000..t0 + 1500).contains(&now) {
                        hold(world, playerstate_iw4::buttons::USE);
                    }
                    if (t0 + 1000..t0 + 1000 + tick).contains(&now) {
                        diag::info!(
                            Sim,
                            "bo2zm t6 sq test (Maxis): use on the put-down trigger at lion spot {k} from {d:.0} behind, hint {hint:?}, try {}",
                            tr + 1
                        );
                    }
                    if now >= t0 + 3000 {
                        diag::warn!(Sim, "bo2zm t6 sq test (Maxis): still holding the ball, again");
                        m.step = Some((true, now, tr + 1));
                    }
                }
                None => {
                    let since = *m.no_trig_since.get_or_insert(now);
                    if now >= since + 15000 && m.tries < 3 {
                        diag::warn!(
                            Sim,
                            "bo2zm t6 sq test (Maxis): no put-down trigger in 15 s (pads not in place?), the pads again"
                        );
                        m.tries += 1;
                        m.placed = None;
                        m.place_at = Some(now + 3000);
                        m.no_trig_since = None;
                    }
                }
            }
        }
    }
    if now >= m.log {
        m.log = now + 3000;
        diag::info!(
            Sim,
            "bo2zm t6 sq test (Maxis): stage {stage}, room kills {} said {said} 15 in {in_place} knife shots {} holding {weapon:?} \
             revived {revived}, pads try {} balls shown {} has ball {has_ball} put-down triggers {} generators {gen1} {gen2}, hint {hint:?}",
            m.kills,
            m.shots,
            m.tries + 1,
            balls.len(),
            places.len()
        );
    }
}

/// For the egg test: the floor under `at` (from 40 above it, down to
/// `depth` below), where a player fits.
fn sq_floor(world: &mut bevy_ecs::prelude::World, at: [f32; 3], depth: f32) -> Option<[f32; 3]> {
    use crate::bullet_collision::{MASK_PLAYER_SOLID, PLAYER_MAXS, PLAYER_MINS};
    let from = [at[0], at[1], at[2] + 40.0];
    let down = [at[0], at[1], at[2] - depth];
    let t = super::frame(world).trace_world(from, down, PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID);
    (t.startsolid == 0 && t.fraction < 1.0).then(|| [at[0], at[1], from[2] + (down[2] - from[2]) * t.fraction])
}

/// For the egg test: a floor spot at one of `radii` from `to` from which a
/// standing player's eye (60 up) sees it with nothing in the way.
fn sq_vantage(world: &mut bevy_ecs::prelude::World, to: [f32; 3], radii: &[f32]) -> Option<[f32; 3]> {
    sq_vantage_nth(world, to, radii, 0)
}

/// The `n`th floor spot (wrapping) whose eye sees `to` past the map's
/// walls. The shot also stops on models (a lion statue's head in front of
/// its ball), which this does not see, so a try that misses takes the next.
fn sq_vantage_nth(world: &mut bevy_ecs::prelude::World, to: [f32; 3], radii: &[f32], n: usize) -> Option<[f32; 3]> {
    use crate::bullet_collision::{MASK_PLAYER_SOLID, MASK_SHOT, PLAYER_MAXS, PLAYER_MINS};
    let mut found = Vec::new();
    for &r in radii {
        for k in 0..16 {
            // Every fifth sixteenth of a turn, so the next try is on another side.
            let a = ((k * 5) % 16) as f32 * std::f32::consts::PI / 8.0;
            let from = [to[0] + r * a.cos(), to[1] + r * a.sin(), to[2] + 80.0];
            let down = [from[0], from[1], to[2] - 500.0];
            let f = super::frame(world);
            let t = f.trace_world(from, down, PLAYER_MINS, PLAYER_MAXS, MASK_PLAYER_SOLID);
            if t.startsolid != 0 || t.fraction >= 1.0 {
                continue;
            }
            let feet = [from[0], from[1], from[2] + (down[2] - from[2]) * t.fraction];
            let eye = [feet[0], feet[1], feet[2] + 60.0];
            let d = gsc_t6::math::length(gsc_t6::math::sub(to, eye));
            let s = f.trace_world(eye, to, [0.0; 3], [0.0; 3], MASK_SHOT);
            if s.fraction * d >= d - 16.0 {
                found.push(feet);
            }
        }
    }
    (!found.is_empty()).then(|| found[n % found.len()])
}

/// A client system state set by a script (`clientsyssetstate`): Die
/// Rise's client script waits on two kinds of level notify, a car setting
/// off (`floor_arrows`) and the power (`power_notify`).
pub(crate) fn client_notify(
    vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>,
    world: &mut bevy_ecs::prelude::World,
    sys: i32,
    state: &str,
) {
    let notify = usize::try_from(sys)
        .ok()
        .and_then(|i| world.resource::<super::Zm>().client_sys.get(i).cloned())
        .is_some_and(|n| n.eq_ignore_ascii_case("levelnotify"));
    if !notify || world.resource::<super::Zm>().map != "zm_highrise" {
        return;
    }
    floor_arrows(vm, world, state);
    power_notify(world, state);
    dragon_sparks(vm, world, state);
    dragon_fireball(vm, world, state);
    super::fxanims::level_notify(world, state);
}

/// The easter egg's end (BO2 zm_highrise_sq.csc `dragon_fireball_go`):
/// "start_fireball_dragon_a" / "_b" puts the fireball on that dragon's
/// mouth (`tag_mouth_fx`; BO2 always uses Maxis's fireball, its own
/// script's slip), the start sound there and a loop that follows the
/// mouth, for the dragon's animation's length; then the effect stops and
/// the end sound plays where the loop was.
#[derive(bevy_ecs::prelude::Resource, Default)]
struct DragonFire(Vec<(u32, u32, i64)>);

const FIREBALL: &str = "maps/zombie_highrise/fx_highrise_dragon_fireball_max";

fn dragon_fireball(vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, world: &mut bevy_ecs::prelude::World, state: &str) {
    let Some(which) = state.strip_prefix("start_fireball_") else {
        return;
    };
    let (tn, scene) = (vm.intern("targetname"), vm.intern("fxanim_scene_1"));
    let props: Vec<u32> = world
        .resource::<super::Zm>()
        .ents
        .iter()
        .filter(|(_, e)| {
            e.obj.is_some_and(|o| {
                vm.to_text(&vm.raw_field(o, tn)) == "fxanim" && vm.to_text(&vm.raw_field(o, scene)) == which
            })
        })
        .map(|(&n, _)| n)
        .collect();
    let len = {
        let zm = world.resource::<super::Zm>();
        zm.anims
            .get(&format!("fxanim_zom_highrise_{which}_anim"))
            .map(|a| ((f32::from(a.numframes) / a.framerate.max(1.0)) * 1000.0).round() as i64)
    };
    let now = world.resource::<super::Zm>().now_ms;
    for prop in props {
        super::natives_fx::bolt_effect(world, prop, FIREBALL, "tag_mouth_fx", false);
        let origin = super::tags::tag_world(world, prop, "tag_mouth_fx")
            .map(|(o, _)| o)
            .or_else(|| world.resource::<super::Zm>().ents.get(&prop).map(|e| e.origin))
            .unwrap_or_default();
        super::natives_fx::sound(world, crate::EventAudience::All, "zmb_sq_fireball_start", origin);
        let n = world.resource_mut::<super::Zm>().alloc_entnum();
        let obj = vm.alloc_object(gsc_t6::ObjKind::Entity(n));
        world.resource_mut::<super::Zm>().ents.insert(
            n,
            super::Ent {
                obj: Some(obj),
                classname: "script_origin".into(),
                origin,
                loop_sound: Some("zmb_sq_fireball_loop".into()),
                ..Default::default()
            },
        );
        let until = now + len.unwrap_or(0);
        world.get_resource_or_insert_with(DragonFire::default).0.push((prop, n, until));
        diag::info!(
            Sim,
            "bo2zm t6 highrise: dragon fireball on {which} at {origin:?} for {len:?} ms"
        );
    }
}

fn dragon_fire(world: &mut bevy_ecs::prelude::World, now: i64) {
    let Some(mut fire) = world.get_resource_mut::<DragonFire>().map(|mut f| std::mem::take(&mut f.0)) else {
        return;
    };
    if fire.is_empty() {
        return;
    }
    fire.retain(|&(prop, n, until)| {
        let mouth = super::tags::tag_world(world, prop, "tag_mouth_fx").map(|(o, _)| o);
        if let Some(o) = mouth
            && let Some(e) = world.resource_mut::<super::Zm>().ents.get_mut(&n)
        {
            e.origin = o;
        }
        if now < until {
            return true;
        }
        super::natives_fx::bolt_effect(world, prop, FIREBALL, "tag_mouth_fx", true);
        let gone = world.resource_mut::<super::Zm>().ents.remove(&n);
        if let Some(e) = gone {
            super::natives_fx::sound(world, crate::EventAudience::All, "zmb_sq_fireball_end", e.origin);
            if let Some(o) = e.obj {
                super::with_vm(world, |vm, world| vm.free_object(world, o));
            }
        }
        diag::info!(Sim, "bo2zm t6 highrise: dragon fireball out");
        false
    });
    world.resource_mut::<DragonFire>().0 = fire;
}

/// The easter egg's dragon fireworks (BO2 zm_highrise_sq.csc
/// `dragon_fireworks_think`, one per dragon, r = Richtofen's, m = Maxis's):
/// the tail sparks on "r_drg_tail" / "m_drg_tail" (a lion ball shot), then
/// each "<r|m>_start_firework" lights the next `<r|m>_drg_firework_<n>`,
/// counting from 0. Each spark plays the dragon's spark effect along its
/// point's forward, the start sound and a sparkler loop that stays.
/// Fireworks before the tail are not heard (BO2's thread waits on it).
#[derive(bevy_ecs::prelude::Resource, Default)]
struct DragonSparks {
    level: Option<gsc_t6::ObjRef>,
    /// Per dragon (r, m): the next firework once the tail has sparked.
    next: [Option<i32>; 2],
}

fn dragon_sparks(vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, world: &mut bevy_ecs::prelude::World, state: &str) {
    let (which, tail) = match state {
        "r_drg_tail" => (0, true),
        "m_drg_tail" => (1, true),
        "r_start_firework" => (0, false),
        "m_start_firework" => (1, false),
        _ => return,
    };
    let mut d = std::mem::take(&mut *world.get_resource_or_insert_with(DragonSparks::default));
    if d.level != Some(vm.level) {
        d = DragonSparks { level: Some(vm.level), ..DragonSparks::default() };
    }
    let drg = ["r", "m"][which];
    let point = match (tail, d.next[which]) {
        (true, None) => {
            d.next[which] = Some(0);
            format!("{drg}_drg_tail")
        }
        (false, Some(n)) => {
            d.next[which] = Some(n + 1);
            format!("{drg}_drg_firework_{n}")
        }
        _ => {
            *world.resource_mut::<DragonSparks>() = d;
            return;
        }
    };
    *world.resource_mut::<DragonSparks>() = d;
    let Some((origin, angles)) = struct_named(vm, &point) else {
        diag::info!(Sim, "bo2zm t6 highrise: dragon spark {point} has no struct");
        return;
    };
    let fx = if which == 0 {
        "maps/zombie_highrise/fx_highrise_dragon_spark_ric"
    } else {
        "maps/zombie_highrise/fx_highrise_dragon_spark_max"
    };
    let (forward, _, _) = gsc_t6::math::angle_vectors(angles);
    super::natives_fx::effect(world, fx, origin, forward);
    super::natives_fx::sound(world, crate::EventAudience::All, "zmb_sq_fireworks_start", origin);
    let n = world.resource_mut::<super::Zm>().alloc_entnum();
    let obj = vm.alloc_object(gsc_t6::ObjKind::Entity(n));
    world.resource_mut::<super::Zm>().ents.insert(
        n,
        super::Ent {
            obj: Some(obj),
            classname: "script_model".into(),
            model: "tag_origin".into(),
            origin,
            angles,
            loop_sound: Some("zmb_sq_sparkler_loop".into()),
            ..Default::default()
        },
    );
    diag::info!(Sim, "bo2zm t6 highrise: dragon spark {point} at {origin:?}");
}

/// The first `level.struct` with this targetname: its origin and angles.
fn struct_named(vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, targetname: &str) -> Option<([f32; 3], [f32; 3])> {
    use gsc_t6::Value;
    let f = vm.intern("struct");
    let Value::Array(arr) = vm.raw_field(vm.level, f) else {
        return None;
    };
    let (tn, o, a) = (vm.intern("targetname"), vm.intern("origin"), vm.intern("angles"));
    let list: Vec<gsc_t6::ObjRef> = arr.snapshot().values_in_order().filter_map(Value::as_obj).collect();
    list.into_iter().find_map(|s| {
        if vm.to_text(&vm.raw_field(s, tn)) != targetname {
            return None;
        }
        Some((vm.raw_field(s, o).as_vec3()?, vm.raw_field(s, a).as_vec3().unwrap_or_default()))
    })
}

/// Die Rise's look before and after the power (BO2 zm_highrise.csc
/// `highrise_watch_for_power` with _zm_utility.csc's fog-volume vision
/// monitor): the dim blue `zm_highrise_off` grade until the power, then
/// `zm_highrise_on`, faded in over the monitor's 2 s, and world fog bank 2.
/// Only while no script vision is up (Who's Who's own comes first).
#[derive(bevy_ecs::prelude::Resource, Default)]
struct PowerLook {
    on: bool,
    /// When it last changed (ms); never changed shows the off look as is.
    since: Option<i64>,
}

/// The server's power notifies: "power_on" at the end of the power switch
/// (zm_highrise.gsc), "pwr" / "pwo" from the dev menu; "ZPO" / "ZPOff"
/// the client script also waits on.
fn power_notify(world: &mut bevy_ecs::prelude::World, state: &str) {
    let on = match state {
        "power_on" | "pwr" | "ZPO" => true,
        "pwo" | "ZPOff" => false,
        _ => return,
    };
    let now = world.resource::<super::Zm>().now_ms;
    let mut look = world.get_resource_or_insert_with(PowerLook::default);
    if look.on != on {
        look.on = on;
        look.since = Some(now);
    }
}

/// Die Rise only: the power look's vision and fog bank, every tick while it
/// fades (in 5% steps), else once a second so a player who joins gets it.
fn power_look(world: &mut bevy_ecs::prelude::World, now: i64) {
    if world.resource::<super::Zm>().map != "zm_highrise" {
        return;
    }
    let look = world.get_resource_or_insert_with(PowerLook::default);
    let (on, since) = (look.on, look.since);
    let tick = i64::from(crate::MATCH_TICK_MS);
    let fading = since.is_some_and(|s| now - s <= 2000 + tick);
    if !fading && now % 1000 >= tick {
        return;
    }
    let (to, from) = if on {
        ("zm_highrise_on", "zm_highrise_off")
    } else {
        ("zm_highrise_off", "zm_highrise_on")
    };
    let t = since.map_or(1.0, |s| ((now - s) as f32 / 2000.0).clamp(0.0, 1.0));
    let t = (t * 20.0).round() / 20.0;
    let text = if t < 1.0 { format!("{to} {t} {from}") } else { format!("{to} 1") };
    super::clientfields::set_map_vision(world, &text);
    // setworldfogactivebank( i, 2 ) after the power; bank 1 before.
    let bank = if on { "2" } else { "" };
    let players: Vec<u32> = world.resource::<super::Zm>().players.keys().copied().collect();
    for c in players {
        super::set_client_dvar(world, c, "bo2zm_fogbank", bank);
    }
}

/// IW4L_T6_ZONELOG=1: every 10 s, `level.active_zone_names` and each zone
/// that is active, occupied, or that the player's box touches a volume of
/// (_zm_zonemgr): enabled / active / occupied and the touch. Unitrigger
/// stubs (box, buildable parts) are only offered in active zones.
pub(crate) fn zone_log(world: &mut bevy_ecs::prelude::World, now: i64) {
    use gsc_t6::Value;
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var_os("IW4L_T6_ZONELOG").is_some()) {
        return;
    }
    if now % 10_000 >= i64::from(crate::MATCH_TICK_MS) {
        return;
    }
    let Some(pbox) = super::triggers::player_box(world, 0) else {
        return;
    };
    let line = super::with_vm(world, |vm, world| {
        let k = |vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, s: &str| vm.intern(s);
        let (zones_k, azn_k) = (k(vm, "zones"), k(vm, "active_zone_names"));
        let (en_k, ac_k, oc_k, vol_k) = (
            k(vm, "is_enabled"),
            k(vm, "is_active"),
            k(vm, "is_occupied"),
            k(vm, "volumes"),
        );
        let active: Vec<String> = match vm.raw_field(vm.level, azn_k) {
            Value::Array(a) => {
                let vals: Vec<Value> = a.read().values_in_order().cloned().collect();
                vals.iter().map(|v| vm.to_text(v)).collect()
            }
            _ => Vec::new(),
        };
        let Value::Array(zones) = vm.raw_field(vm.level, zones_k) else {
            return format!("no level.zones; active [{}]", active.join(" "));
        };
        let zones: Vec<(String, Value)> = zones
            .read()
            .map
            .iter()
            .map(|(key, v)| (vm.to_text(&key.value()), v.clone()))
            .collect();
        let mut out = Vec::new();
        for (name, z) in zones {
            let Some(z) = z.as_obj() else { continue };
            let flag = |vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, f| {
                gsc_t6::truthy(&vm.raw_field(z, f))
            };
            let (en, ac, oc) = (flag(vm, en_k), flag(vm, ac_k), flag(vm, oc_k));
            let vols: Vec<u32> = match vm.raw_field(z, vol_k) {
                Value::Array(a) => {
                    let vals: Vec<Value> = a.read().values_in_order().cloned().collect();
                    vals.iter().filter_map(|v| super::entnum(vm, v)).collect()
                }
                _ => Vec::new(),
            };
            let touch = vols.iter().any(|n| {
                let e = world.resource::<super::Zm>().ents.get(n).cloned();
                e.is_some_and(|e| super::triggers::touches(world, &e, pbox.0, pbox.1))
            });
            if ac || oc || touch {
                out.push(format!(
                    "{name} en {} ac {} oc {} touch {touch} vols {}",
                    en as u8,
                    ac as u8,
                    oc as u8,
                    vols.len()
                ));
            }
        }
        // The showing box's use stub: the zone it was filed under.
        let (ch_k, ci_k, us_k, iz_k) = (
            k(vm, "chests"),
            k(vm, "chest_index"),
            k(vm, "unitrigger_stub"),
            k(vm, "in_zone"),
        );
        let index = vm.raw_field(vm.level, ci_k).as_int().unwrap_or(-1);
        let chest = match vm.raw_field(vm.level, ch_k) {
            Value::Array(a) => usize::try_from(index)
                .ok()
                .and_then(|i| a.read().values_in_order().nth(i).cloned()),
            _ => None,
        };
        let box_zone = chest
            .as_ref()
            .and_then(|c| c.as_obj())
            .map(|c| vm.raw_field(c, us_k))
            .and_then(|s| s.as_obj())
            .map(|s| {
                let v = vm.raw_field(s, iz_k);
                let o = k(vm, "origin");
                let at = vm.raw_field(s, o).as_vec3().unwrap_or_default();
                format!("{} at ({:.0} {:.0} {:.0})", vm.to_text(&v), at[0], at[1], at[2])
            })
            .unwrap_or_default();
        format!(
            "active [{}]; box {index} stub zone {box_zone:?}; {}",
            active.join(" "),
            out.join("; ")
        )
    })
    .unwrap_or_default();
    let o = pbox.0;
    diag::info!(
        Sim,
        "bo2zm t6 zonelog: player box min ({:.0} {:.0} {:.0}) {line}",
        o[0],
        o[1],
        o[2]
    );
}

/// The arrow lamps over the perk cars' shaft doors. BO2's Die Rise client
/// script (zm_highrise_classic.csc `floor_indicator`) turns them: a car
/// setting off sends the level notify "<shaft>_u" or "<shaft>_d"
/// (zm_highrise_elevators.gsc `clientnotify( elevator.name + "_d" )`) and
/// every lamp of that shaft (`elevator_bldg<shaft>_indicator`, 2 to 5 per
/// shaft) drops its arrow and shows the up or down one on `tag_origin`.
/// IW4L_T6_ARROWLOG=1 logs each change.
fn floor_arrows(
    vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>,
    world: &mut bevy_ecs::prelude::World,
    state: &str,
) {
    const SHAFTS: [&str; 7] = ["3d", "3b", "3", "3c", "1d", "1c", "1b"];
    const UP: &str = "maps/zombie_highrise/fx_highrise_elevator_arrow_up";
    const DOWN: &str = "maps/zombie_highrise/fx_highrise_elevator_arrow_down";
    let Some((shaft, way)) = state.rsplit_once('_') else {
        return;
    };
    if !SHAFTS.contains(&shaft) || !matches!(way, "u" | "d") {
        return;
    }
    let name = format!("elevator_bldg{shaft}_indicator");
    let tn = vm.intern("targetname");
    let lamps: Vec<u32> = world
        .resource::<super::Zm>()
        .ents
        .iter()
        .filter_map(|(n, e)| {
            let o = e.obj?;
            (vm.to_text(&vm.raw_field(o, tn)) == name).then_some(*n)
        })
        .collect();
    let on = if way == "u" { UP } else { DOWN };
    let mut lit = 0;
    for &n in &lamps {
        super::natives_fx::bolt_effect(world, n, UP, "tag_origin", true);
        super::natives_fx::bolt_effect(world, n, DOWN, "tag_origin", true);
        lit += usize::from(super::natives_fx::bolt_effect(world, n, on, "tag_origin", false));
    }
    if std::env::var_os("IW4L_T6_ARROWLOG").is_some() {
        diag::info!(
            Sim,
            "bo2zm t6 floor arrows: {state} -> {} of {} lamps of {name} lit",
            lit,
            lamps.len()
        );
    }
}

/// Maxis's voice on his side of the easter egg (BO2 zm_highrise_sq.csc
/// `sq_play_maxis_vo`): the server's `maxissay` sets each player's
/// `clientfield_sq_vo` to the line's number in `level.sq_clientfield_vo`;
/// his client plays that line at the three `sq_maxis_vo_loc` spots nearest
/// him, for him only. Like any client field, the same number twice in a
/// row plays nothing (the server never sets it back to 0).
#[derive(bevy_ecs::prelude::Resource, Default)]
struct MaxisVo {
    level: Option<gsc_t6::ObjRef>,
    last: std::collections::HashMap<u32, i32>,
}

pub(super) fn maxis_vo(vm: &mut gsc_t6::Vm<bevy_ecs::prelude::World>, world: &mut bevy_ecs::prelude::World, n: u32, value: i32) {
    use gsc_t6::Value;
    {
        let mut m = world.get_resource_or_insert_with(MaxisVo::default);
        if m.level != Some(vm.level) {
            *m = MaxisVo { level: Some(vm.level), ..MaxisVo::default() };
        }
        if m.last.insert(n, value).unwrap_or(0) == value || value == 0 {
            return;
        }
    }
    let f = vm.intern("sq_clientfield_vo");
    let Value::Array(lines) = vm.raw_field(vm.level, f) else {
        return;
    };
    let lines = lines.snapshot();
    let Some(alias) = lines
        .keys()
        .find(|k| lines.get(k).and_then(Value::as_int) == Some(value))
        .map(|k| vm.to_text(&k.value()))
    else {
        return;
    };
    if alias == "none" {
        return;
    }
    let f = vm.intern("struct");
    let Value::Array(arr) = vm.raw_field(vm.level, f) else {
        return;
    };
    let (tn, o) = (vm.intern("targetname"), vm.intern("origin"));
    let list: Vec<gsc_t6::ObjRef> = arr.snapshot().values_in_order().filter_map(Value::as_obj).collect();
    let mut spots: Vec<[f32; 3]> = list
        .into_iter()
        .filter(|&s| vm.to_text(&vm.raw_field(s, tn)) == "sq_maxis_vo_loc")
        .filter_map(|s| vm.raw_field(s, o).as_vec3())
        .collect();
    if spots.len() < 3 {
        return;
    }
    let Some(me) = super::frame(world).player(crate::world::ClientId(n)).map(|p| p.origin) else {
        return;
    };
    let dist = |p: &[f32; 3]| (0..3).map(|i| (p[i] - me[i]).powi(2)).sum::<f32>();
    spots.sort_by(|a, b| dist(a).total_cmp(&dist(b)));
    for spot in &spots[..3] {
        super::natives_fx::sound(world, crate::EventAudience::Client(crate::world::ClientId(n)), &alias, *spot);
    }
    diag::info!(Sim, "bo2zm t6 highrise: Maxis says {alias} to player {n} at {:?}", &spots[..3]);
}
