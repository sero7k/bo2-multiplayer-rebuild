//! bo2zm: runs a player on a flat floor with the Black Ops II movement rules
//! and with the old IW4 ones, and checks what changes.

use movement_iw4::{
    AdsFracContext, AdsIntentContext, AirMoveContext, Bo2Feel, CmdScaleWalkContext,
    CollisionBackend, FlatMantleAnimLength, GroundTraceInput, JumpLaunchContext,
    MeleeChargeWeaponDelays, MoveBounds, PmoveSingleContext, SprintContext, ViewAngleClamp,
    WalkMoveContext, ZeroMantleRootDelta, pmove,
};
use playerstate_iw4::{PlayerState, UserCmd};
use trace_iw4::Trace;

const BO2: Bo2Feel = Bo2Feel {
    on: true,
    sprint_strafe_speed_scale: 0.667,
    jump_slowdown: true,
    ..Bo2Feel::IW4
};

/// A floor at z = 0 and nothing else.
struct Floor;

impl CollisionBackend for Floor {
    fn trace(&self, input: GroundTraceInput) -> Trace {
        let start = input.start[2] + input.mins[2];
        let end = input.end[2] + input.mins[2];
        let mut hit = Trace {
            fraction: 1.0,
            endpos: input.end,
            ..Trace::default()
        };
        if start < 0.0 {
            hit.startsolid = 1;
            hit.allsolid = u8::from(end < 0.0);
            hit.fraction = 0.0;
            hit.endpos = input.start;
            hit.normal = [0.0, 0.0, 1.0];
            return hit;
        }
        if end < 0.0 {
            let fraction = start / (start - end);
            hit.fraction = fraction;
            hit.normal = [0.0, 0.0, 1.0];
            hit.walkable = 1;
            hit.hit_type = 1;
            hit.hit_id = trace_iw4::ENTITYNUM_WORLD;
            for i in 0..3 {
                hit.endpos[i] = input.start[i] + (input.end[i] - input.start[i]) * fraction;
            }
        }
        hit
    }
}

/// Stairs up along +x: flat floor at z = 0 until x = `first`, then a step
/// of `rise` every `run` units. Only sweeps forward along x or straight up
/// and down are exact (all this probe makes).
#[derive(Clone, Copy)]
struct Stairs {
    first: f32,
    run: f32,
    rise: f32,
}

impl Stairs {
    /// The tread under a box whose front is at `x`.
    fn height(&self, x: f32) -> f32 {
        if x < self.first {
            0.0
        } else {
            (((x - self.first) / self.run).floor() + 1.0) * self.rise
        }
    }

    fn trace(&self, input: &GroundTraceInput) -> Trace {
        const BACK_OFF: f32 = 0.03125;
        let (s, e) = (input.start, input.end);
        let bottom = s[2] + input.mins[2];
        let front = s[0] + input.maxs[0];
        let (dx, dz) = (e[0] - s[0], e[2] - s[2]);
        let mut hit = Trace {
            fraction: 1.0,
            endpos: input.end,
            ..Trace::default()
        };
        let at = |hit: &mut Trace, f: f32, normal: [f32; 3]| {
            hit.fraction = f;
            hit.normal = normal;
            hit.walkable = u8::from(normal[2] > 0.7);
            hit.hit_type = 1;
            hit.hit_id = trace_iw4::ENTITYNUM_WORLD;
            for i in 0..3 {
                hit.endpos[i] = s[i] + (e[i] - s[i]) * f;
            }
        };
        if bottom < self.height(front) - 0.001 {
            hit.startsolid = 1;
            at(&mut hit, 0.0, [0.0, 0.0, 1.0]);
            return hit;
        }
        let mut t = 0.0_f32;
        loop {
            let x = front + dx * t;
            let h = self.height(x);
            let edge = if x < self.first {
                self.first
            } else {
                self.first + (((x - self.first) / self.run).floor() + 1.0) * self.run
            };
            let t_edge = if dx > 0.0 { (edge - front) / dx } else { f32::INFINITY };
            if dz < 0.0 {
                let t_floor = (h - bottom) / dz;
                if t_floor >= t && t_floor <= t_edge.min(1.0) {
                    at(&mut hit, t_floor, [0.0, 0.0, 1.0]);
                    return hit;
                }
            }
            if t_edge > 1.0 {
                return hit;
            }
            if bottom + dz * t_edge < self.height(edge) - 0.001 {
                let f = ((edge - BACK_OFF - front) / dx).max(0.0);
                at(&mut hit, f, [-1.0, 0.0, 0.0]);
                return hit;
            }
            t = t_edge;
        }
    }
}

/// The floor and a ceiling at `roof` (none at infinity). A `slick` floor is
/// ice; a `mantle` floor is a ledge top the player could climb. A `window`
/// is a thin wall across the way ahead with a window in it.
struct Room {
    roof: f32,
    slick: bool,
    mantle: bool,
    window: Option<Window>,
    stairs: Option<Stairs>,
}

/// A thin wall across +x at x = `wall`, open from z = `sill` to `lintel`.
#[derive(Clone, Copy)]
struct Window {
    wall: f32,
    sill: f32,
    lintel: f32,
}

impl Window {
    /// Where the box first hits the wall, sill or lintel on its way, and the
    /// face's normal.
    fn hit(&self, input: &GroundTraceInput) -> Option<(f32, [f32; 3])> {
        const EPS: f32 = 0.01;
        let (s, e) = (input.start, input.end);
        let at = |f: f32| s[2] + (e[2] - s[2]) * f;
        let fits = |z: f32| z + input.mins[2] >= self.sill - EPS && z + input.maxs[2] <= self.lintel + EPS;
        let front = s[0] + input.maxs[0];
        let back = s[0] + input.mins[0];
        let dx = e[0] - s[0];
        if front <= self.wall + EPS {
            // Not through yet: the box's front reaches the wall this move.
            if dx <= 0.0 || front + dx <= self.wall + EPS {
                return None;
            }
            let f = ((self.wall - front) / dx).max(0.0);
            if !fits(at(f)) {
                return Some((f, [-1.0, 0.0, 0.0]));
            }
        } else if back >= self.wall {
            return None;
        } else if !fits(s[2]) {
            return Some((0.0, [-1.0, 0.0, 0.0]));
        }
        // In the opening: the sill and lintel are a floor and a ceiling until
        // the box's back is through.
        let entered = if front <= self.wall + EPS { ((self.wall - front) / dx).max(0.0) } else { 0.0 };
        let left = if dx > 0.0 { ((self.wall - back) / dx).min(1.0) } else { 1.0 };
        let dz = e[2] - s[2];
        if dz < 0.0 {
            let f = (self.sill - (s[2] + input.mins[2])) / dz;
            if (entered..left).contains(&f) {
                return Some((f, [0.0, 0.0, 1.0]));
            }
        } else if dz > 0.0 {
            let f = (self.lintel - (s[2] + input.maxs[2])) / dz;
            if (entered..left).contains(&f) {
                return Some((f, [0.0, 0.0, -1.0]));
            }
        }
        None
    }
}

impl CollisionBackend for Room {
    fn trace(&self, input: GroundTraceInput) -> Trace {
        if input.tracemask == movement_iw4::CONTENTS_MANTLE && !self.mantle {
            return Trace {
                fraction: 1.0,
                endpos: input.end,
                ..Trace::default()
            };
        }
        let mut hit = match self.stairs {
            Some(stairs) => stairs.trace(&input),
            None => Floor.trace(input),
        };
        if self.slick && hit.walkable != 0 {
            hit.surface_flags = 2;
        }
        let start = input.start[2] + input.maxs[2];
        let end = input.end[2] + input.maxs[2];
        if start <= self.roof && end > self.roof {
            let fraction = (self.roof - start) / (end - start);
            if fraction < hit.fraction {
                hit = Trace {
                    fraction,
                    normal: [0.0, 0.0, -1.0],
                    endpos: input.end,
                    ..Trace::default()
                };
                for i in 0..3 {
                    hit.endpos[i] = input.start[i] + (input.end[i] - input.start[i]) * fraction;
                }
            }
        }
        if let Some((fraction, normal)) = self.window.and_then(|w| w.hit(&input))
            && fraction < hit.fraction
        {
            hit = Trace {
                fraction,
                normal,
                walkable: u8::from(normal[2] > 0.7),
                hit_type: 1,
                hit_id: trace_iw4::ENTITYNUM_WORLD,
                endpos: input.end,
                ..Trace::default()
            };
            for i in 0..3 {
                hit.endpos[i] = input.start[i] + (input.end[i] - input.start[i]) * fraction;
            }
        }
        hit
    }
}

fn context(
    feel: Bo2Feel,
    old_buttons: u32,
    ads_allowed: bool,
    weapon_blocks_prone: bool,
) -> PmoveSingleContext {
    let air = AirMoveContext {
        player_spectate_speed_scale: 1.0,
        shellshock_gravity_scale: 1.0,
        shellshock_gravity_bias: 0.0,
    };
    PmoveSingleContext {
        walk: WalkMoveContext {
            cmd_scale: CmdScaleWalkContext {
                player_back_speed_scale: 0.7,
                player_strafe_speed_scale: 0.8,
                player_sprint_speed_scale: 1.5,
                player_last_stand_crawl_speed_scale: 0.15,
                weapon_move_speed_scale: 1.0,
                weapon_ads_move_speed_scale: 1.0,
                shellshock_movement_scale: 1.0,
                prone_lerp_ms: if feel.on {
                    movement_iw4::PRONE_LERP_MS
                } else {
                    0
                },
                bo2: feel.on,
            },
            weapon_move_scale: 1.0,
            old_buttons,
            jump: JumpLaunchContext {
                jump_height: 39.0,
                dive: false,
                crouch_jump_scale: 1.0,
                jump_ladder_push_vel: 128.0,
            },
            air,
            feel,
        },
        air,
        bounds: MoveBounds {
            mins: [-15.0, -15.0, 0.0],
            maxs: [15.0, 15.0, 70.0],
            tracemask: 0x0281_0011,
        },
        view_angles: ViewAngleClamp {
            pitch_up: 85.0,
            pitch_down: 85.0,
            unclamped_pitch_bit: false,
        },
        sprint: SprintContext {
            weapon_max_sprint_time: 4000,
            sprint_forever: false,
            min_sprint_time_seconds: 1.0,
            sprint_delay_seconds: 0.0,
            sprint_forward_minimum: 105,
            stand_up_clear: true,
            sprint_recharge_pause_seconds: 0.0,
        },
        ads_intent: AdsIntentContext {
            ads_allowed,
            weapon_def_scope: false,
            sprint_hold_ads: false,
            bo2: false,
        },
        ads_frac: AdsFracContext {
            aim_down_sight: false,
            ads_in_rate: 1.0 / 200.0,
            ads_out_rate: 1.0 / 200.0,
            rechamber_while_ads: true,
            ads_fire_only: false,
            ..AdsFracContext::default()
        },
        melee_charge: MeleeChargeWeaponDelays {
            melee_delay_ms: 0,
            melee_charge_delay_ms: 0,
        },
        player_melee_range: movement_iw4::MELEE_CHARGE_PLAYER_MELEE_RANGE_DEFAULT,
        old_buttons,
        old_move: [0, 0],
        weapon_blocks_prone,
    }
}

struct Run {
    ps: PlayerState,
    time: i32,
    old_buttons: u32,
    old_move: [i8; 2],
    feel: Bo2Feel,
    /// The gun in hand can aim.
    ads: bool,
    /// The gun in hand blocks prone.
    blocks_prone: bool,
    /// A ceiling this high (none at infinity).
    roof: f32,
    /// The floor is ice.
    slick: bool,
    /// The floor is a ledge top the player could climb.
    mantle: bool,
    /// Walking speed scale from a shellshock (1 = none).
    shock: f32,
    /// A wall with a window in it ahead.
    window: Option<Window>,
    /// Stairs ahead instead of a flat floor.
    stairs: Option<Stairs>,
}

impl Run {
    fn new(feel: Bo2Feel) -> Self {
        let mut ps = PlayerState::ZERO;
        ps.gravity = 800;
        ps.speed = 190;
        ps.move_speed_scale_multiplier = 1.0;
        ps.ground_entity_num = 1022;
        ps.view_height_target = 60;
        ps.view_height_current = 60.0;
        ps.command_time = 1000;
        Self {
            ps,
            time: 1000,
            old_buttons: 0,
            old_move: [0, 0],
            feel,
            ads: false,
            blocks_prone: false,
            roof: f32::INFINITY,
            slick: false,
            mantle: false,
            shock: 1.0,
            window: None,
            stairs: None,
        }
    }

    /// Runs `ms` of 50 ms frames holding these inputs.
    fn hold(&mut self, ms: i32, forward: i8, right: i8, buttons: u32) {
        let frames = ms / 50;
        for _ in 0..frames {
            self.time += 50;
            let mut cmd = UserCmd {
                server_time: self.time,
                forwardmove: forward,
                rightmove: right,
                buttons,
                ..UserCmd::default()
            };
            let mut ctx = context(self.feel, self.old_buttons, self.ads, self.blocks_prone);
            ctx.walk.cmd_scale.shellshock_movement_scale = self.shock;
            ctx.old_move = self.old_move;
            pmove(
                &mut self.ps,
                &mut cmd,
                ctx,
                &Room {
                    roof: self.roof,
                    slick: self.slick,
                    mantle: self.mantle,
                    window: self.window,
                    stairs: self.stairs,
                },
                &FlatMantleAnimLength::default(),
                &ZeroMantleRootDelta,
            );
            self.old_buttons = buttons;
            self.old_move = [forward, right];
        }
    }

    fn ground_speed(&self) -> f32 {
        (self.ps.velocity[0].powi(2) + self.ps.velocity[1].powi(2)).sqrt()
    }

    fn sideways_speed(&self) -> f32 {
        self.ps.velocity[1].abs()
    }
}

const SPRINT: u32 = 0x2;
const JUMP: u32 = 0x400;

#[test]
fn sprint_strafe_is_slower_sideways_in_bo2() {
    let mut bo2 = Run::new(BO2);
    let mut iw4 = Run::new(Bo2Feel::IW4);
    for run in [&mut bo2, &mut iw4] {
        run.hold(1000, 127, 127, SPRINT);
    }
    eprintln!(
        "sprint+strafe sideways: bo2 {:.1} iw4 {:.1}",
        bo2.sideways_speed(),
        iw4.sideways_speed()
    );
    assert_ne!(bo2.ps.pm_flags & playerstate_iw4::pm_flags::SPRINTING, 0);
    assert!(bo2.sideways_speed() < iw4.sideways_speed() * 0.8);
}

#[test]
fn flat_jump_lands_faster_in_bo2() {
    let mut bo2 = Run::new(BO2);
    let mut iw4 = Run::new(Bo2Feel::IW4);
    for run in [&mut bo2, &mut iw4] {
        run.hold(1000, 127, 0, 0);
        run.hold(50, 127, 0, JUMP);
        // Up and down: about 0.62 s of air at jump height 39.
        while run.ps.ground_entity_num == 0x7FF {
            run.hold(50, 127, 0, 0);
        }
        // The landing slowdown runs on the first frame on the ground.
        run.hold(50, 127, 0, 0);
    }
    eprintln!(
        "speed right after a flat jump lands: bo2 {:.1} iw4 {:.1}",
        bo2.ground_speed(),
        iw4.ground_speed()
    );
    assert!(bo2.ground_speed() > iw4.ground_speed() * 1.05);
}

/// Jumps from standing, lands, jumps again on the first frame back on the
/// ground; the two jumps' peak heights.
fn hop_twice(feel: Bo2Feel) -> (f32, f32) {
    let mut run = Run::new(feel);
    run.hold(500, 0, 0, 0);
    let mut peaks = [0.0_f32; 2];
    for peak in &mut peaks {
        run.hold(50, 0, 0, JUMP);
        while run.ps.ground_entity_num == 0x7FF {
            *peak = peak.max(run.ps.origin[2]);
            run.hold(50, 0, 0, 0);
        }
    }
    (peaks[0], peaks[1])
}

/// BO2's Jump_GetLandFactor: in multiplayer a jump taken right after landing
/// one goes lower (energy / 2.5, so about 2/5 of the height); in Zombies
/// (jump slowdown off) both go full height.
#[test]
fn a_jump_right_after_landing_goes_lower_in_bo2() {
    let (first, second) = hop_twice(BO2);
    let (zm_first, zm_second) = hop_twice(Bo2Feel { jump_slowdown: false, ..BO2 });
    eprintln!("hop peaks: mp {first:.1} then {second:.1}; zombies {zm_first:.1} then {zm_second:.1}");
    assert!((35.0..=40.0).contains(&first), "{first}");
    assert!(second < first * 0.5 && second > first * 0.3, "{second}");
    assert!((zm_second - zm_first).abs() < 2.0, "{zm_first} {zm_second}");
}

/// BO2's PM_GroundTraceMissed: a player within a unit of the floor still
/// counts as almost on it, so the step rhythm keeps going (down stairs, off
/// a curb); MW2's rules stop it the moment the feet leave the floor.
#[test]
fn rhythm_keeps_going_just_above_the_floor_in_bo2() {
    let bob = |feel| {
        let mut run = Run::new(feel);
        run.ps.origin[2] = 1.5;
        run.ps.ground_entity_num = playerstate_iw4::ENTITYNUM_NONE;
        run.ps.velocity = [190.0, 0.0, 0.0];
        run.hold(50, 127, 0, 0);
        (run.ps.bob_cycle, run.ps.origin[2])
    };
    let (bo2, bo2_z) = bob(BO2);
    let (iw4, iw4_z) = bob(Bo2Feel::IW4);
    eprintln!("one frame just above the floor: bo2 bob {bo2} (z {bo2_z:.2}), iw4 bob {iw4} (z {iw4_z:.2})");
    assert!(bo2_z > 0.25 && iw4_z > 0.25, "still off the floor");
    assert!(bo2 > 0 && iw4 == 0, "{bo2} {iw4}");
}

/// BO2's PM_StepSlideMove: each stair step on the ground costs speed by its
/// height (a full 18 leaves 0.2), so stairs climb slower than in MW2 rules.
#[test]
fn stairs_climb_slower_in_bo2() {
    let walk = |feel, stairs| {
        let mut run = Run::new(feel);
        run.stairs = stairs;
        run.hold(3000, 127, 0, 0);
        (run.ps.origin[0], run.ps.origin[2])
    };
    let stairs = Some(Stairs {
        first: 40.0,
        run: 12.0,
        rise: 8.0,
    });
    let (bo2_flat, _) = walk(BO2, None);
    let (bo2_x, bo2_z) = walk(BO2, stairs);
    let (iw4_flat, _) = walk(Bo2Feel::IW4, None);
    let (iw4_x, iw4_z) = walk(Bo2Feel::IW4, stairs);
    eprintln!(
        "3 s up stairs: bo2 {bo2_x:.1} (up {bo2_z:.0}, flat {bo2_flat:.1}), iw4 {iw4_x:.1} (up {iw4_z:.0}, flat {iw4_flat:.1})"
    );
    assert!(bo2_z > 100.0 && iw4_z > 100.0, "both climb");
    let (bo2_share, iw4_share) = (bo2_x / bo2_flat, iw4_x / iw4_flat);
    assert!(bo2_share < iw4_share - 0.05, "{bo2_share} {iw4_share}");
}

/// BO2's PM_SlideMove: while pm_time runs (just after a landing) grazing a
/// wall keeps the speed you came in with; MW2, and BO2 once it runs out,
/// loses the part into the wall. (Both drop the speed when a wall stops you
/// dead: the tick's end sets it from the real move.)
#[test]
fn a_wall_just_after_landing_keeps_speed_in_bo2() {
    let into_wall = |feel: Bo2Feel, pm_time| {
        let mut run = Run::new(feel);
        run.hold(500, 0, 0, 0);
        let x = run.ps.origin[0];
        run.window = Some(Window {
            wall: x + 15.0,
            sill: f32::INFINITY,
            lintel: f32::INFINITY,
        });
        run.ps.velocity = [100.0, 120.0, 0.0];
        run.ps.pm_time = pm_time;
        run.hold(50, 0, 0, 0);
        assert!(run.ps.origin[0] - x < 0.1, "the wall holds");
        assert!(run.ps.origin[1] > 4.0, "slides along it");
        run.ps.velocity[0]
    };
    let (bo2, bo2_later, iw4) = (into_wall(BO2, 1000), into_wall(BO2, 0), into_wall(Bo2Feel::IW4, 1000));
    eprintln!("speed into the wall: bo2 {bo2:.1}, bo2 later {bo2_later:.1}, iw4 {iw4:.1}");
    assert!(bo2 > 50.0, "{bo2}");
    assert!(bo2_later.abs() < 1.0, "{bo2_later}");
    assert!(iw4.abs() < 1.0, "{iw4}");
}

#[test]
fn jumping_out_of_a_sprint_costs_sprint_in_bo2() {
    let mut bo2 = Run::new(BO2);
    let mut iw4 = Run::new(Bo2Feel::IW4);
    for run in [&mut bo2, &mut iw4] {
        run.hold(1000, 127, 0, SPRINT);
        run.hold(50, 127, 0, SPRINT | JUMP);
    }
    eprintln!(
        "sprint end after a sprint-jump: bo2 {} iw4 {} (time {})",
        bo2.ps.last_sprint_end, iw4.ps.last_sprint_end, bo2.time
    );
    assert_eq!(bo2.ps.last_sprint_end - iw4.ps.last_sprint_end, 800);
}

#[test]
fn sprint_then_prone_dives_only_when_dtp_is_on() {
    let off = Bo2Feel { dive: false, ..BO2 };
    let on = Bo2Feel { dive: true, ..BO2 };
    let mut runs = [Run::new(off), Run::new(on)];
    for run in &mut runs {
        run.hold(1000, 127, 0, SPRINT);
        run.hold(50, 127, 0, SPRINT | playerstate_iw4::buttons::PRONE);
    }
    let dived = |run: &Run| run.ps.pm_flags & movement_iw4::PMF_DIVE != 0;
    assert!(!dived(&runs[0]), "dtp 0 turns the dive off");
    assert!(dived(&runs[1]));
}

const PRONE: u32 = playerstate_iw4::buttons::PRONE;
const STANCE_HELD: u32 = playerstate_iw4::buttons::STANCE_HELD;
const ADS: u32 = 0x800;
const DIVE: Bo2Feel = Bo2Feel {
    dive: true,
    omni: true,
    ..BO2
};

fn flag(run: &Run, flag: u32) -> bool {
    run.ps.pm_flags & flag != 0
}

/// Sprints 1 s, dives pushing this way (a tap of the default hold-style
/// prone key), and runs until the dive leaves the air. Returns the time in
/// the air (ms) and the highest point.
fn dive(run: &mut Run, forward: i8, right: i8, buttons: u32) -> (i32, f32) {
    run.hold(1000, forward, right, SPRINT);
    run.hold(50, forward, right, SPRINT | PRONE | STANCE_HELD | buttons);
    assert!(flag(run, movement_iw4::PMF_DIVE), "dived");
    let start = run.time - 50;
    let mut top = run.ps.origin[2];
    while flag(run, movement_iw4::PMF_DIVE) {
        run.hold(50, 0, 0, SPRINT | buttons);
        top = top.max(run.ps.origin[2]);
        assert!(run.time - start < 3000, "the dive came down");
    }
    (run.time - start, top)
}

/// Runs until the view is at standing height and still.
fn ms_to_stand(run: &mut Run, buttons: u32) -> i32 {
    let start = run.time;
    while run.ps.view_height_current != 60.0 || run.ps.view_height_lerp_time != 0 {
        run.hold(50, 0, 0, buttons);
        assert!(run.time - start < 3000, "stood up");
    }
    run.time - start
}

#[test]
fn a_dive_needs_over_a_quarter_second_of_sprint() {
    let mut early = Run::new(DIVE);
    let mut late = Run::new(DIVE);
    early.hold(150, 127, 0, SPRINT);
    early.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
    late.hold(300, 127, 0, SPRINT);
    late.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
    assert!(
        !flag(&early, movement_iw4::PMF_DIVE),
        "0.15 s of sprint is too soon"
    );
    assert!(flag(&late, movement_iw4::PMF_DIVE), "0.3 s of sprint dives");
}

#[test]
fn another_dive_waits_a_second_and_a_half_after_the_slide() {
    // Dive, get straight up, then sprint and dive again: 0.3 s of sprint is
    // too soon after the slide, 1.3 s is not.
    let again = |sprint_ms: i32| {
        let mut run = Run::new(DIVE);
        dive(&mut run, 127, 0, 0);
        while flag(&run, movement_iw4::PMF_DIVE_SLIDE) || run.ps.view_height_lerp_time != 0 {
            run.hold(50, 0, 0, 0);
        }
        let slide_end = run.ps.dive_end_time;
        run.hold(50, 0, 0, JUMP);
        ms_to_stand(&mut run, 0);
        run.hold(sprint_ms, 127, 0, SPRINT);
        run.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
        (flag(&run, movement_iw4::PMF_DIVE), run.time - slide_end)
    };
    let (soon, soon_ms) = again(300);
    let (later, later_ms) = again(1300);
    eprintln!("second dive {soon_ms} ms after the slide: {soon}; {later_ms} ms: {later}");
    assert!(!soon && soon_ms <= 1500);
    assert!(later && later_ms > 1500);
}

#[test]
fn omni_sprint_runs_full_speed_any_way_in_bo2() {
    let mut forward = Run::new(DIVE);
    let mut back = Run::new(DIVE);
    let mut side = Run::new(DIVE);
    let mut old_back = Run::new(BO2);
    forward.hold(1000, 127, 0, SPRINT);
    back.hold(1000, -127, 0, SPRINT);
    side.hold(1000, 0, 127, SPRINT);
    old_back.hold(1000, -127, 0, SPRINT);
    eprintln!(
        "sprint speed: forward {:.1} back {:.1} side {:.1}; without omni back {:.1}",
        forward.ground_speed(),
        back.ground_speed(),
        side.ground_speed(),
        old_back.ground_speed()
    );
    for run in [&forward, &back, &side] {
        assert!(flag(run, playerstate_iw4::pm_flags::SPRINTING));
        assert!((run.ground_speed() - forward.ground_speed()).abs() < 1.0);
    }
    assert!(!flag(&old_back, playerstate_iw4::pm_flags::SPRINTING));
}

#[test]
fn a_dive_keeps_the_runs_direction() {
    // Sprinting ahead (yaw 0: ahead is +x, right is -y), then diving while
    // pushing right: BO2 keeps the run's speed and direction, and the push
    // only steers, a frame of air control at a time.
    let mut run = Run::new(DIVE);
    run.hold(1000, 127, 0, SPRINT);
    let sprint = run.ground_speed();
    run.hold(50, 0, 127, SPRINT | PRONE | STANCE_HELD);
    eprintln!("sprint {sprint:.1}, dive velocity {:?}", run.ps.velocity);
    assert!(flag(&run, movement_iw4::PMF_DIVE));
    assert!((run.ps.velocity[0] - sprint).abs() < 1.0, "the run's speed");
    // One frame of air control: 9.5 at 50 ms, snapped to a whole number.
    assert!(run.sideways_speed() <= 10.0, "one frame of steering");
}

#[test]
fn dive_rises_to_jump_height_and_lands_with_the_animation() {
    let mut run = Run::new(DIVE);
    let (air_ms, top) = dive(&mut run, 127, 0, 0);
    eprintln!("dive: {air_ms} ms in the air, top {top:.1}");
    // pb_dive_prone: 0.733 s from take-off to touch-down.
    assert!((700..=800).contains(&air_ms), "air time {air_ms}");
    // The rise stops at a jump's height, past it by up to a frame's climb
    // (the test's frames are 50 ms; at 125 fps it is about 3.5).
    assert!((39.0..=39.0 + 25.0).contains(&top), "top {top}");
    assert!(flag(&run, movement_iw4::PMF_DIVE_SLIDE));
    assert!(flag(&run, playerstate_iw4::pm_flags::PRONE));
    assert_eq!(run.ps.view_height_target, 11, "the view drops in the air");
}

#[test]
fn a_dive_hangs_at_the_top_for_0_4_s() {
    // At a jump's height the dive stops rising and, until 0.4 s after
    // take-off, sinks only by a frame's gravity at a time; from the frame at
    // 0.4 s (BO2's check is "less than 0.4 s") it falls.
    let mut run = Run::new(DIVE);
    run.hold(1000, 127, 0, SPRINT);
    run.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
    let take_off = run.time;
    let mut heights = Vec::new();
    while run.time - take_off < 350 {
        run.hold(50, 0, 0, SPRINT);
        heights.push(run.ps.origin[2]);
    }
    eprintln!("dive heights to 0.35 s: {heights:.1?}");
    let top = heights.iter().cloned().fold(0.0, f32::max);
    let at_350 = *heights.last().unwrap();
    assert!(at_350 >= 39.0, "still at the top at 0.35 s: {at_350}");
    assert!(top - at_350 < 10.0, "hung, not fell: {top} to {at_350}");
    run.hold(50, 0, 0, SPRINT);
    assert!(
        run.ps.origin[2] < at_350 - 2.0,
        "falls at 0.4 s: {at_350} to {}",
        run.ps.origin[2]
    );
}

#[test]
fn a_dive_goes_through_a_window_a_jump_hits() {
    // A window 65 high, from 35 to 100 off the floor, 60 ahead of the
    // player's front: a sprint jump (a standing body, 70 tall) cannot fit;
    // a dive's body is crouch-height (50) by then, so it goes through.
    let through = |dive: bool| {
        let mut run = Run::new(DIVE);
        run.hold(1000, 127, 0, SPRINT);
        let wall = run.ps.origin[0] + 15.0 + 60.0;
        run.window = Some(Window {
            wall,
            sill: 35.0,
            lintel: 100.0,
        });
        let take_off = if dive { SPRINT | PRONE | STANCE_HELD } else { SPRINT | JUMP };
        run.hold(50, 127, 0, take_off);
        run.hold(1500, 127, 0, SPRINT);
        eprintln!(
            "{}: at x {:.1} (wall {wall:.1}), z {:.1}",
            if dive { "dive" } else { "jump" },
            run.ps.origin[0],
            run.ps.origin[2]
        );
        run.ps.origin[0] - 15.0 > wall
    };
    assert!(!through(false), "a sprint jump hits the wall");
    assert!(through(true), "a dive goes through the window");
}

#[test]
fn a_dive_steers_a_little() {
    // Dive straight ahead, then push right through the flight: BO2's
    // ordinary air control cut to 0.4 through a dive (PM_CmdScale),
    // acceleration 1 toward the 76 wish speed, 3.8 a frame at 50 ms. Touch-down is a hard landing, as in BO2 (a fall of
    // over 12): the speed drops to 0.67 of it, a dive's too.
    let steer = |right: i8| {
        let mut run = Run::new(DIVE);
        run.hold(1000, 127, 0, SPRINT);
        run.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
        assert!(flag(&run, movement_iw4::PMF_DIVE), "dived");
        let run_speed = run.ps.velocity[0];
        let mut top_sideways = 0.0_f32;
        while flag(&run, movement_iw4::PMF_DIVE) {
            top_sideways = top_sideways.max(run.sideways_speed());
            run.hold(50, 0, right, SPRINT);
            assert!(run.time < 5000, "the dive came down");
        }
        (top_sideways, run_speed, run.ps.velocity[0])
    };
    let (still, run_speed, ahead) = steer(0);
    let (pushed, _, pushed_ahead) = steer(127);
    eprintln!(
        "dive: sideways speed in the air {still:.1} still, {pushed:.1} pushing right; \
         ahead {run_speed:.1} at take-off, {ahead:.1} after touch-down"
    );
    assert!(still < 0.01);
    // 16 frames of it before touch-down, never past the 76 wish speed.
    assert!((50.0..=76.0).contains(&pushed), "pushed {pushed}");
    assert!(
        (pushed_ahead - ahead).abs() < 0.01,
        "pushing sideways keeps the forward speed"
    );
    assert!(
        (ahead - run_speed * 0.67).abs() < 1.0,
        "touch-down keeps 0.67 of the speed: {run_speed} to {ahead}"
    );
}

#[test]
fn a_dive_under_a_low_ceiling_drops_at_once() {
    // The ceiling stops the rise at 20 of the 39: no hold, straight down.
    let mut run = Run::new(DIVE);
    run.roof = 90.0;
    let (air_ms, top) = dive(&mut run, 127, 0, 0);
    eprintln!("dive under a ceiling: {air_ms} ms in the air, top {top:.1}");
    assert!(top <= 20.01, "top {top}");
    assert!(air_ms < 400, "air time {air_ms}");
}

#[test]
fn a_gun_that_blocks_prone_cannot_dive() {
    let mut run = Run::new(DIVE);
    run.blocks_prone = true;
    run.hold(1000, 127, 0, SPRINT);
    run.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
    assert!(!flag(&run, movement_iw4::PMF_DIVE));
}

#[test]
fn dive_slides_pauses_then_holds_prone_without_the_prone_lock() {
    let mut run = Run::new(DIVE);
    dive(&mut run, 127, 0, 0);
    let landed = run.time;
    while flag(&run, movement_iw4::PMF_DIVE_SLIDE) {
        assert_eq!(run.ps.pm_time, 0, "no 1.8 s prone lock");
        run.hold(50, 0, 0, SPRINT);
    }
    let slide = run.time - landed;
    eprintln!("slide: {slide} ms");
    assert_eq!(slide, 300, "dtp_max_slide_duration");
    assert!(flag(&run, movement_iw4::PMF_DIVE_PRONE));
    assert_eq!(run.ground_speed(), 0.0);
    // Then 0.1 s still: a push does nothing.
    run.hold(50, 127, 0, 0);
    assert_eq!(run.ground_speed(), 0.0, "the still moment");
    run.hold(500, 0, 0, 0);
    assert!(flag(&run, playerstate_iw4::pm_flags::PRONE), "stays prone");
    assert_eq!(run.ps.pm_flags & playerstate_iw4::pm_flags::JUMPING, 0);
}

#[test]
fn a_slide_that_stops_ends_the_dive_at_once() {
    let mut run = Run::new(DIVE);
    dive(&mut run, 127, 0, 0);
    let landed = run.time;
    // Stopped dead (into a wall) just after touch-down.
    run.ps.velocity = [1.0, 0.0, 0.0];
    run.hold(50, 0, 0, SPRINT);
    assert!(!movement_iw4::dive_to_prone(&run.ps), "the dive ended");
    assert_eq!(
        run.ps.dive_end_time,
        landed + 50,
        "the next dive counts from the stop"
    );
    // The still moment, 0.1 s from the stop, then the push moves him.
    run.hold(50, 127, 0, 0);
    assert_eq!(run.ground_speed(), 0.0, "still");
    run.hold(50, 127, 0, 0);
    eprintln!("crawl after the stop: {:.1}", run.ground_speed());
    assert!(run.ground_speed() > 0.0, "free 0.1 s after the stop");
}

/// Dives forward and returns the ground speed on the slide's last frame.
fn slide_end_speed(mantle: bool) -> f32 {
    let mut run = Run::new(DIVE);
    run.mantle = mantle;
    dive(&mut run, 127, 0, 0);
    let mut speed = 0.0;
    while movement_iw4::dive_to_prone(&run.ps) {
        speed = run.ground_speed();
        run.hold(50, 0, 0, SPRINT);
    }
    speed
}

#[test]
fn a_slide_over_a_ledge_top_keeps_its_speed() {
    let floor = slide_end_speed(false);
    let ledge = slide_end_speed(true);
    eprintln!("slide's last frame: floor {floor}, ledge top {ledge}");
    assert!(ledge > floor + 100.0);
}

#[test]
fn a_crouch_while_sprinting_only_crouches() {
    let mut run = Run::new(DIVE);
    run.hold(1000, 127, 0, SPRINT);
    run.hold(50, 127, 0, SPRINT | playerstate_iw4::buttons::CROUCH);
    assert!(!flag(&run, movement_iw4::PMF_DIVE), "no dive on a crouch");
}

#[test]
fn scripts_see_the_dive_from_take_off_to_the_slides_end() {
    let mut run = Run::new(DIVE);
    run.hold(1000, 127, 0, SPRINT);
    assert!(
        !movement_iw4::dive_to_prone(&run.ps),
        "sprinting is not diving"
    );
    run.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
    while flag(&run, movement_iw4::PMF_DIVE) {
        assert!(movement_iw4::dive_to_prone(&run.ps), "in the air");
        run.hold(50, 0, 0, SPRINT);
    }
    let landed = run.time;
    while movement_iw4::dive_to_prone(&run.ps) {
        run.hold(50, 0, 0, SPRINT);
        assert!(run.time - landed < 1000, "the slide ended");
    }
    let slide = run.time - landed;
    eprintln!("dive seen on the ground for {slide} ms");
    assert_eq!(slide, 300);
    assert_eq!(run.ps.dive_end_time, run.time, "the slide ended this frame");
    assert!(
        flag(&run, movement_iw4::PMF_DIVE_PRONE),
        "then he lies still, prone"
    );
}

#[test]
fn getting_up_from_a_dive_is_twice_as_quick() {
    let mut dived = Run::new(DIVE);
    dive(&mut dived, 127, 0, 0);
    while flag(&dived, movement_iw4::PMF_DIVE_SLIDE) || dived.ps.view_height_lerp_time != 0 {
        dived.hold(50, 0, 0, 0);
    }
    dived.hold(50, 0, 0, JUMP);
    let from_dive = 50 + ms_to_stand(&mut dived, 0);

    let mut prone = Run::new(DIVE);
    prone.hold(2500, 0, 0, PRONE | STANCE_HELD);
    assert_eq!(prone.ps.view_height_current, 11.0);
    let from_prone = ms_to_stand(&mut prone, 0);
    eprintln!("prone to standing: after a dive {from_dive} ms, ordinary {from_prone} ms");
    // Both steps (lying to crouched, crouched to standing) take half as
    // long: 300 ms of view change, not 600. Each step also waits a 50 ms
    // test frame to start, and the dive's count holds the jump press.
    assert_eq!((from_dive - 50 - 100) * 2, from_prone - 100);
    dived.hold(50, 0, 0, 0);
    assert!(!flag(&dived, movement_iw4::PMF_DIVE_GETUP), "get-up over");
}

#[test]
fn a_held_prone_key_does_not_drop_you_back_down() {
    // Dived on a held prone key, kept holding it, and pressed jump.
    let held = PRONE | STANCE_HELD;
    let mut run = Run::new(DIVE);
    dive(&mut run, 127, 0, held);
    while !flag(&run, movement_iw4::PMF_DIVE_PRONE) {
        run.hold(50, 0, 0, held);
    }
    run.hold(50, 0, 0, held | JUMP);
    ms_to_stand(&mut run, held);
    run.hold(500, 0, 0, held);
    assert_eq!(run.ps.pm_flags & 3, 0, "still standing");
    run.hold(50, 0, 0, 0);
    assert!(!flag(&run, movement_iw4::PMF_DIVE_GETUP));
    run.hold(1000, 0, 0, held);
    assert!(
        flag(&run, playerstate_iw4::pm_flags::PRONE),
        "prone key works again"
    );
}

#[test]
fn a_get_up_pressed_mid_dive_is_kept() {
    let mut run = Run::new(DIVE);
    run.hold(1000, 127, 0, SPRINT);
    run.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
    run.hold(100, 0, 0, 0);
    run.hold(50, 0, 0, JUMP);
    assert!(flag(&run, movement_iw4::PMF_DIVE), "still in the air");
    while flag(&run, movement_iw4::PMF_DIVE) || flag(&run, movement_iw4::PMF_DIVE_SLIDE) {
        run.hold(50, 0, 0, 0);
    }
    // The slide is over: the still moment (0.1 s), the get-up kept...
    assert!(flag(&run, movement_iw4::PMF_DIVE_PRONE));
    assert!(flag(&run, movement_iw4::PMF_DIVE_GETUP), "the get-up is kept");
    run.hold(100, 0, 0, 0);
    // ...then up the moment it ends, with no key pressed.
    assert!(!flag(&run, movement_iw4::PMF_DIVE_PRONE), "getting up");
    ms_to_stand(&mut run, 0);
    assert_eq!(run.ps.pm_flags & 3, 0);
}

#[test]
fn aiming_works_through_a_dive_with_sprint_held() {
    let mut run = Run::new(DIVE);
    run.ads = true;
    run.hold(1000, 127, 0, SPRINT);
    run.hold(50, 127, 0, SPRINT | PRONE | STANCE_HELD);
    run.hold(50, 127, 0, SPRINT | ADS);
    assert!(flag(&run, movement_iw4::PMF_DIVE));
    assert!(
        flag(&run, playerstate_iw4::pm_flags::ADS_INTENT),
        "aims in the air"
    );
    while !flag(&run, movement_iw4::PMF_DIVE_PRONE) {
        run.hold(50, 127, 0, SPRINT | ADS);
        assert!(
            flag(&run, playerstate_iw4::pm_flags::ADS_INTENT),
            "aims on the ground"
        );
    }
}

#[test]
fn a_prone_toggle_switched_off_gets_you_up() {
    // Toggle-style prone: the key latches prone on (no STANCE_HELD) and a
    // second press lets it go.
    let mut run = Run::new(DIVE);
    run.hold(1000, 127, 0, SPRINT);
    run.hold(50, 127, 0, SPRINT | PRONE);
    while !flag(&run, movement_iw4::PMF_DIVE_PRONE) {
        run.hold(50, 0, 0, PRONE);
    }
    run.hold(500, 0, 0, PRONE);
    assert!(
        flag(&run, movement_iw4::PMF_DIVE_PRONE),
        "the latch alone holds prone"
    );
    run.hold(50, 0, 0, 0);
    ms_to_stand(&mut run, 0);
    assert_eq!(run.ps.pm_flags & 3, 0);
}

const STAMIN_UP: Bo2Feel = Bo2Feel {
    longer_sprint_perk: movement_iw4::PERK_LONGERSPRINT,
    sprint_multiplier: 2.0,
    ..BO2
};

/// Holds sprint with a gun in hand until the sprint runs out. Returns how
/// long it lasted (ms) and the speed after 1 s of it.
fn sprint_out(feel: Bo2Feel, perk: bool) -> (i32, f32) {
    let mut run = Run::new(feel);
    run.ps.weapon = 1;
    if perk {
        run.ps.perks[0] |= movement_iw4::PERK_LONGERSPRINT;
    }
    run.hold(1000, 127, 0, SPRINT);
    let speed = run.ground_speed();
    while flag(&run, playerstate_iw4::pm_flags::SPRINTING) && run.time < 20_000 {
        run.hold(50, 127, 0, SPRINT);
    }
    (run.ps.last_sprint_end - run.ps.last_sprint_start, speed)
}

#[test]
fn stamin_up_sprints_twice_as_long_and_a_tenth_faster_in_zombies() {
    let zombies = Bo2Feel {
        zombies: true,
        ..STAMIN_UP
    };
    let (plain_ms, plain_speed) = sprint_out(zombies, false);
    let (perk_ms, perk_speed) = sprint_out(zombies, true);
    let (mp_ms, mp_speed) = sprint_out(STAMIN_UP, true);
    eprintln!(
        "sprint: no perk {plain_ms} ms {plain_speed:.1}; Stamin-Up {perk_ms} ms {perk_speed:.1}; MP {mp_ms} ms {mp_speed:.1}"
    );
    assert_eq!(plain_ms, 4000);
    assert_eq!(perk_ms, 8000);
    assert_eq!(mp_ms, 8000);
    assert!((perk_speed / plain_speed - 1.1).abs() < 0.01);
    assert!(
        (mp_speed - plain_speed).abs() < 0.5,
        "multiplayer's perk adds no speed"
    );
}

/// Walks forward on ice for `ms`, then lets go for `coast_ms`. Returns the
/// speed after the walk and after the coast.
fn walk_on_ice(feel: Bo2Feel, ms: i32, coast_ms: i32) -> (f32, f32) {
    let mut run = Run::new(feel);
    run.slick = true;
    run.hold(ms, 127, 0, 0);
    let walked = run.ground_speed();
    run.hold(coast_ms, 0, 0, 0);
    (walked, run.ground_speed())
}

/// BO2's Jump_Check: no jumping off ice (MW2 jumps).
#[test]
fn no_jumping_off_ice_in_bo2() {
    let jumped = |feel| {
        let mut run = Run::new(feel);
        run.slick = true;
        run.hold(200, 0, 0, 0);
        run.hold(50, 0, 0, JUMP);
        run.ps.velocity[2] > 0.0
    };
    assert!(!jumped(BO2));
    assert!(jumped(Bo2Feel::IW4));
}

#[test]
fn ice_picks_up_twice_as_fast_and_still_slows_you_in_bo2() {
    let (bo2_start, _) = walk_on_ice(BO2, 200, 0);
    let (iw4_start, _) = walk_on_ice(Bo2Feel::IW4, 200, 0);
    let (bo2_full, bo2_coast) = walk_on_ice(BO2, 3000, 500);
    let (iw4_full, iw4_coast) = walk_on_ice(Bo2Feel::IW4, 3000, 500);
    eprintln!(
        "ice: 0.2 s in BO2 {bo2_start:.1} old {iw4_start:.1}; full {bo2_full:.1}/{iw4_full:.1}; after 0.5 s coasting BO2 {bo2_coast:.1} old {iw4_coast:.1}"
    );
    // Acceleration 2 against sliding friction 1.5: 19 a frame, less 7.5%
    // (speeds snap to whole numbers each frame).
    assert!((bo2_start - 68.0).abs() < 1.0);
    assert!((iw4_start - 40.0).abs() < 1.0);
    assert!((bo2_full - 190.0).abs() < 1.0 && (iw4_full - 190.0).abs() < 1.0);
    // About 190 x 0.925^10: BO2's ice still slows you; the old ice never did.
    assert!((bo2_coast - 88.0).abs() < 1.0);
    assert!((iw4_coast - 190.0).abs() < 1.0);
}

fn walk_shocked(feel: Bo2Feel, shock_file_movement: f32) -> f32 {
    let mut run = Run::new(feel);
    run.ps.pm_flags |= playerstate_iw4::pm_flags::SHELLSHOCKED;
    run.shock = movement_iw4::shellshock_walk_scale(true, shock_file_movement, feel);
    run.hold(3000, 127, 0, 0);
    run.ground_speed()
}

#[test]
fn a_shellshock_slows_you_by_its_own_number_in_bo2() {
    // BO2's shock files: explosion and pain 1.0, flashbang 0.8, most 0.4.
    let bo2: Vec<f32> = [1.0, 0.8, 0.4].map(|m| walk_shocked(BO2, m)).to_vec();
    let old: Vec<f32> = [1.0, 0.8, 0.4]
        .map(|m| walk_shocked(Bo2Feel::IW4, m))
        .to_vec();
    eprintln!("shocked walk: BO2 {bo2:?} old {old:?}");
    assert!(
        (bo2[0] - 190.0).abs() < 1.0,
        "an explosion does not slow you"
    );
    assert!((bo2[1] - 152.0).abs() < 1.0);
    assert!((bo2[2] - 76.0).abs() < 1.0);
    assert!(
        old.iter().all(|s| (s - 76.0).abs() < 1.0),
        "the old rule: 0.4"
    );
}

/// Crawling, then aiming, then pressing forward again: ground speeds.
fn crawl_then_aim(feel: Bo2Feel, weaponstate: i32) -> (f32, f32, f32) {
    let lie = PRONE | STANCE_HELD;
    let mut run = Run::new(feel);
    run.ads = true;
    run.hold(2500, 0, 0, lie);
    run.hold(1000, 127, 0, lie);
    let crawl = run.ground_speed();
    run.hold(1000, 127, 0, lie | ADS);
    let aimed = run.ground_speed();
    run.ps.weaponstate_primary = weaponstate;
    run.hold(50, 0, 0, lie | ADS);
    run.hold(1000, 127, 0, lie | ADS);
    (crawl, aimed, run.ground_speed())
}

#[test]
fn aiming_lying_down_stops_the_crawl_in_bo2() {
    let (crawl, aimed, again) = crawl_then_aim(BO2, 0);
    let (_, _, firing) = crawl_then_aim(BO2, 6);
    let (iw4_crawl, iw4_aimed, _) = crawl_then_aim(Bo2Feel::IW4, 0);
    eprintln!(
        "bo2 crawl {crawl:.1} aimed {aimed:.1} pressed again {again:.1} firing {firing:.1}; iw4 crawl {iw4_crawl:.1} aimed {iw4_aimed:.1}"
    );
    assert!(crawl > 20.0);
    assert!(aimed < 1.0, "aiming holds still");
    assert!(again > crawl * 0.8, "a fresh press drops the aim and crawls");
    assert!(firing < 1.0, "not while the gun fires");
    assert!(iw4_aimed > iw4_crawl * 0.5, "MW2 keeps crawling");
}
