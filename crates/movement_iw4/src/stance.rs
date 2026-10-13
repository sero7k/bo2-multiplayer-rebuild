use playerstate_iw4::{PM_TYPE_LAST_STAND, PlayerState, UserCmd, buttons, eflags, pm_flags};

use crate::check_prone::{ProneRules, player_prone_allowed};
use crate::collision::CollisionBackend;
use crate::{GroundTraceInput, MoveBounds, Pml, add_predictable_event};

pub mod view_height {
    pub const PRONE: i32 = 0x0b;

    pub const LAST_STAND: i32 = 0x16;

    pub const CROUCH: i32 = 0x28;

    pub const STAND: i32 = 0x3c;
}

pub const STAND_MAXS_Z: f32 = 70.0;

pub const CROUCH_MAXS_Z: f32 = 50.0;

pub const PRONE_MAXS_Z: f32 = 30.0;

const EV_STANCE_FORCE_STAND: i32 = 6;

const EV_STANCE_FORCE_CROUCH: i32 = 7;

const EV_STANCE_FORCE_PRONE: i32 = 8;

const PMF_STANCE_LOCKED: u32 = 0xc00;

/// What a stance change checks for room against: the move mask without
/// players and zombies (BO2's `tracemask & 0xfdff7fff`).
pub const STANCE_ROOM_MASK: u32 = 0xfdff_7fff;

/// bo2zm: lying down, Black Ops II lifts the body this far and drops it back.
const LIE_DOWN_LIFT: f32 = 10.0;

/// bo2zm: how far below a body lying down Black Ops II looks for the ground.
const LIE_DOWN_GROUND_PROBE: f32 = 0.25;

/// bo2zm: the torso's slope pitch stays within this of where the player looks.
const PRONE_TORSO_PITCH_RANGE: f32 = 45.0;

const EV_RESET_ADS: i32 = 0x11;

fn stance_hull_allsolid<C: CollisionBackend>(
    collision: &C,
    origin: [f32; 3],
    bounds: MoveBounds,
    maxs_z: f32,
    bo2: bool,
) -> bool {
    // bo2zm: BO2 (PM_CanStand, PM_CheckDuck) leaves players (0x0200_0000)
    // and zombies (0x8000) out: someone in the way never stops a stand-up.
    let tracemask = if bo2 {
        bounds.tracemask & STANCE_ROOM_MASK
    } else {
        bounds.tracemask
    };
    let trace = collision.trace(GroundTraceInput {
        start: origin,
        end: origin,
        mins: bounds.mins,
        maxs: [bounds.maxs[0], bounds.maxs[1], maxs_z],
        tracemask,
    });
    trace.allsolid != 0
}

pub fn update_stance_flags<C: CollisionBackend>(
    ps: &mut PlayerState,
    cmd: &mut UserCmd,
    collision: &C,
    bounds: MoveBounds,
    weapon_blocks_prone: bool,
    feel: &crate::Bo2Feel,
) {
    let bo2 = feel.on;
    if ps.pm_type == 5 {
        ps.pm_flags &= !(pm_flags::PRONE | pm_flags::CROUCH);
        if (cmd.buttons & buttons::PRONE) != 0 {
            cmd.buttons &= !buttons::PRONE;
            add_predictable_event(ps, EV_STANCE_FORCE_STAND, 0);
        }
        ps.view_height_target = 0;
        return;
    }

    if ps.pm_type == 8 {
        ps.view_height_target = 8;
        return;
    }

    if (ps.pm_flags & pm_flags::SPRINTING) != 0
        && (ps.pm_flags & (pm_flags::PRONE | pm_flags::CROUCH)) != 0
    {
        ps.view_height_target = view_height::STAND;
        ps.e_flags &= !(eflags::DUCK | eflags::PRONE);
        ps.pm_flags &= !(pm_flags::PRONE | pm_flags::CROUCH);
        add_predictable_event(ps, EV_STANCE_FORCE_STAND, 0);
        return;
    }

    if (ps.pm_flags & PMF_STANCE_LOCKED) != 0 {
        return;
    }

    if ps.pm_type == 7 {
        ps.pm_flags = (ps.pm_flags & !pm_flags::CROUCH) | pm_flags::PRONE;
        return;
    }

    if (ps.pm_flags & pm_flags::LADDER) != 0 && (cmd.buttons & 0x300) != 0 {
        cmd.buttons &= !0x300;
        add_predictable_event(ps, EV_STANCE_FORCE_STAND, 0);
    }

    let crouch_btn = (cmd.buttons & buttons::CROUCH) != 0;
    let prone_btn = (cmd.buttons & buttons::PRONE) != 0;
    let stance_held = (cmd.buttons & buttons::STANCE_HELD) != 0;
    let origin = ps.origin;

    if prone_btn && (ps.pm_flags & 0x400) == 0 {
        let rules = ProneRules::for_player(ps, bo2, feel.zombies);
        if player_prone_allowed(ps, collision, bounds.maxs[0], weapon_blocks_prone, rules) {
            ps.pm_flags = (ps.pm_flags & !pm_flags::CROUCH) | pm_flags::PRONE;
        } else if ps.ground_entity_num != playerstate_iw4::ENTITYNUM_NONE && !stance_held {
            let event = if (ps.pm_flags & (pm_flags::PRONE | pm_flags::CROUCH)) == 0 {
                EV_STANCE_FORCE_STAND
            } else {
                EV_STANCE_FORCE_CROUCH
            };
            add_predictable_event(ps, event, 3);
        }
    } else if crouch_btn {
        if (ps.pm_flags & pm_flags::PRONE) == 0 {
            ps.pm_flags |= pm_flags::CROUCH;
        } else if !stance_hull_allsolid(collision, origin, bounds, CROUCH_MAXS_Z, bo2) {
            ps.pm_flags = (ps.pm_flags & !pm_flags::PRONE) | pm_flags::CROUCH;
        } else if !stance_held {
            add_predictable_event(ps, EV_STANCE_FORCE_PRONE, 2);
        }
    } else if (ps.pm_flags & pm_flags::PRONE) != 0 {
        if !stance_hull_allsolid(collision, origin, bounds, STAND_MAXS_Z, bo2) {
            ps.pm_flags &= !(pm_flags::PRONE | pm_flags::CROUCH);
        } else if !stance_hull_allsolid(collision, origin, bounds, CROUCH_MAXS_Z, bo2) {
            ps.pm_flags = (ps.pm_flags & !pm_flags::PRONE) | pm_flags::CROUCH;
        } else if !stance_held {
            add_predictable_event(ps, EV_STANCE_FORCE_PRONE, 1);
        }
    } else if (ps.pm_flags & pm_flags::CROUCH) != 0 {
        if !stance_hull_allsolid(collision, origin, bounds, STAND_MAXS_Z, bo2) {
            ps.pm_flags &= !pm_flags::CROUCH;
        } else if !stance_held {
            add_predictable_event(ps, EV_STANCE_FORCE_CROUCH, 1);
        }
    }
}

/// bo2zm: Black Ops II on the frame a player lies down (`was_prone` false):
/// moving keys held drop the aim; the body is lifted 10 units and dropped
/// back, points where the player looks, and takes its slope pitch from the
/// ground just under it (flat in last stand, over nothing or stuck); the
/// torso's pitch follows it but stays within 45 of the view. In last stand
/// the body points where the player looks every frame.
pub fn bo2_lie_down<C: CollisionBackend>(
    ps: &mut PlayerState,
    cmd: &UserCmd,
    collision: &C,
    bounds: MoveBounds,
    was_prone: bool,
) {
    if (ps.pm_flags & pm_flags::PRONE) == 0 {
        return;
    }
    if !was_prone {
        if cmd.forwardmove != 0 || cmd.rightmove != 0 {
            ps.pm_flags &= !pm_flags::PRONEMOVE_OVERRIDDEN;
            crate::add_event(ps, EV_RESET_ADS);
            ps.pm_flags &= !pm_flags::ADS_INTENT;
        }
        let trace = |start: [f32; 3], end: [f32; 3]| {
            collision.trace(GroundTraceInput {
                start,
                end,
                mins: bounds.mins,
                maxs: bounds.maxs,
                tracemask: bounds.tracemask & STANCE_ROOM_MASK,
            })
        };
        let lerp = |from: [f32; 3], to: [f32; 3], frac: f32| {
            [
                (to[0] - from[0]) * frac + from[0],
                (to[1] - from[1]) * frac + from[1],
                (to[2] - from[2]) * frac + from[2],
            ]
        };
        let origin = ps.origin;
        let up = [origin[0], origin[1], origin[2] + LIE_DOWN_LIFT];
        let lifted = lerp(origin, up, trace(origin, up).fraction);
        ps.origin = lerp(lifted, origin, trace(lifted, origin).fraction);
        ps.prone_direction = ps.viewangles[1];
        let below = [ps.origin[0], ps.origin[1], ps.origin[2] - LIE_DOWN_GROUND_PROBE];
        let ground = trace(ps.origin, below);
        ps.prone_direction_pitch =
            if ground.startsolid != 0 || ground.fraction >= 1.0 || ps.pm_type == PM_TYPE_LAST_STAND {
                0.0
            } else {
                math_iw4::pitch_for_yaw_on_normal(ps.prone_direction, ground.normal)
            };
        let off = math_iw4::angle_subtract(ps.prone_direction_pitch, ps.viewangles[0]);
        ps.prone_torso_pitch = if off < -PRONE_TORSO_PITCH_RANGE {
            ps.viewangles[0] - PRONE_TORSO_PITCH_RANGE
        } else if off > PRONE_TORSO_PITCH_RANGE {
            ps.viewangles[0] + PRONE_TORSO_PITCH_RANGE
        } else {
            ps.prone_direction_pitch
        };
    }
    if ps.pm_type == PM_TYPE_LAST_STAND {
        ps.prone_direction = ps.viewangles[1];
    }
}

pub fn sync_stance_tail(ps: &mut PlayerState) -> f32 {
    match stance_surface_type(ps) {
        StanceSurface::Prone => {
            ps.e_flags = (ps.e_flags & !eflags::DUCK) | eflags::PRONE;
            ps.pm_flags = (ps.pm_flags & !pm_flags::CROUCH) | pm_flags::PRONE;
            PRONE_MAXS_Z
        }
        StanceSurface::Crouch => {
            ps.e_flags = (ps.e_flags & !eflags::PRONE) | eflags::DUCK;
            ps.pm_flags = (ps.pm_flags & !pm_flags::PRONE) | pm_flags::CROUCH;
            CROUCH_MAXS_Z
        }
        StanceSurface::LastStand => {
            ps.e_flags = (ps.e_flags & !eflags::DUCK) | eflags::PRONE;
            ps.pm_flags = (ps.pm_flags & !pm_flags::CROUCH) | pm_flags::PRONE;
            CROUCH_MAXS_Z
        }
        StanceSurface::Stand => {
            ps.e_flags &= !(eflags::DUCK | eflags::PRONE);
            ps.pm_flags &= !(pm_flags::PRONE | pm_flags::CROUCH);
            STAND_MAXS_Z
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum StanceSurface {
    Stand = 0,

    Prone = 1,

    Crouch = 2,

    LastStand = 3,
}

#[must_use]
pub fn stance_surface_type(ps: &PlayerState) -> StanceSurface {
    match ps.view_height_target {
        view_height::LAST_STAND => StanceSurface::LastStand,
        view_height::CROUCH => StanceSurface::Crouch,
        view_height::PRONE => StanceSurface::Prone,
        _ => StanceSurface::Stand,
    }
}

#[must_use]
pub fn view_height_lerp_duration(lerp_target: i32, lerp_down: i32) -> i32 {
    match lerp_target {
        view_height::PRONE => 400,
        view_height::CROUCH => {
            if lerp_down != 0 {
                200
            } else {
                400
            }
        }
        _ => 200,
    }
}

/// bo2zm: the stance change's time. `prone_ms` is the crouch-to-prone
/// stage's time (BO2 600; 0 = MW2's 400). bo2mp: getting up from a dive runs
/// at the usual speed, as BO2's (it has no get-up dvar of its own; an earlier
/// double speed was too fast in testing).
fn lerp_duration(_ps: &PlayerState, lerp_target: i32, lerp_down: i32, prone_ms: i32) -> i32 {
    if lerp_target == view_height::PRONE && prone_ms > 0 {
        prone_ms
    } else {
        view_height_lerp_duration(lerp_target, lerp_down)
    }
}

type Knot = (i32, f32);

const CROUCH_FROM_STAND: &[Knot] = &[
    (0, 60.0),
    (1, 59.5),
    (4, 58.5),
    (30, 56.0),
    (80, 44.0),
    (90, 41.5),
    (95, 40.5),
    (100, 40.0),
];

const STAND_CURVE: &[Knot] = &[
    (0, 40.0),
    (5, 40.5),
    (10, 41.5),
    (20, 44.0),
    (70, 56.0),
    (96, 58.5),
    (99, 59.5),
    (100, 60.0),
];

const PRONE_CURVE: &[Knot] = &[
    (0, 40.0),
    (11, 38.0),
    (22, 33.0),
    (34, 25.0),
    (45, 16.0),
    (50, 15.0),
    (55, 16.0),
    (70, 18.0),
    (90, 17.0),
    (100, 11.0),
];

const CROUCH_FROM_PRONE: &[Knot] = &[
    (0, 11.0),
    (5, 10.0),
    (30, 21.0),
    (50, 25.0),
    (67, 31.0),
    (83, 34.0),
    (100, 40.0),
];

fn view_height_curve(lerp_target: i32, lerp_down: i32) -> &'static [Knot] {
    if lerp_target == view_height::PRONE {
        return PRONE_CURVE;
    }
    if lerp_target == view_height::CROUCH {
        if lerp_down != 0 {
            return CROUCH_FROM_STAND;
        }
        return CROUCH_FROM_PRONE;
    }
    STAND_CURVE
}

#[must_use]
fn view_height_lerp_sample(table: &[Knot], key: i32) -> f32 {
    if key != 0 {
        let mut index = 1;
        while index < table.len() {
            let (knot_key, knot_value) = table[index];
            if knot_key == key {
                return knot_value;
            }
            if key <= knot_key {
                let (prev_key, prev_value) = table[index - 1];
                let fraction = ((key - prev_key) as f32) / ((knot_key - prev_key) as f32);
                return (knot_value - prev_value) * fraction + prev_value;
            }
            index += 1;
        }
    }
    table[0].1
}

const VIEW_HEIGHT_RATE: f32 = 180.0;

const LAST_STAND_VIEW_HEIGHT_RATE: f32 = 120.0;

const REVERSE_REWIND_SCALE: f32 = 0.01;

#[allow(clippy::too_many_lines)]
pub fn update_view_height(ps: &mut PlayerState, pml: &Pml, cmd: &UserCmd, prone_lerp_ms: i32) {
    let target = ps.view_height_target;
    if target == 0 || ps.view_height_current == 0.0 {
        ps.view_height_current = if ps.pm_type == 5 { 0.0 } else { target as f32 };
        return;
    }

    let target_height = target as f32;
    if target_height == ps.view_height_current && ps.view_height_lerp_time == 0 {
        return;
    }

    let last_stand = (ps.pm_flags & pm_flags::LAST_STAND) != 0;
    let unknown_token = target != view_height::PRONE
        && target != view_height::CROUCH
        && target != view_height::STAND;
    if unknown_token || (last_stand && target_height < ps.view_height_current) {
        let rate = if last_stand {
            LAST_STAND_VIEW_HEIGHT_RATE
        } else {
            VIEW_HEIGHT_RATE
        };
        ps.view_height_lerp_time = 0;
        if ps.view_height_current < target_height {
            let raised = pml.frametime * rate + ps.view_height_current;
            ps.view_height_current = raised;
            if raised < target_height {
                return;
            }
        } else {
            let lowered = ps.view_height_current - pml.frametime * rate;
            ps.view_height_current = lowered;

            if lowered > target_height {
                return;
            }
        }
        ps.view_height_current = target_height;
        return;
    }

    let mut progress = 0i32;
    if ps.view_height_lerp_time != 0 {
        let lerp_target = ps.view_height_lerp_target;
        let duration = lerp_duration(ps, lerp_target, ps.view_height_lerp_down, prone_lerp_ms);
        progress = cmd
            .server_time
            .wrapping_sub(ps.view_height_lerp_time)
            .wrapping_mul(100)
            / duration;
        if progress >= 100 {
            progress = 100;
            ps.view_height_lerp_time = 0;
            ps.view_height_current = lerp_target as f32;
        } else {
            if progress < 0 {
                progress = 0;
            }
            let table = view_height_curve(lerp_target, ps.view_height_lerp_down);
            ps.view_height_current = view_height_lerp_sample(table, progress);
        }
    }

    if ps.view_height_lerp_time == 0 {
        start_view_height_lerp(ps, cmd);
    } else {
        reverse_view_height_lerp(ps, cmd, progress, prone_lerp_ms);
    }
}

fn start_view_height_lerp(ps: &mut PlayerState, cmd: &UserCmd) {
    let target = ps.view_height_target;
    if (target as f32) == ps.view_height_current {
        return;
    }
    ps.view_height_lerp_time = cmd.server_time;

    const CROUCH_HEIGHT: f32 = 40.0;
    match target {
        view_height::PRONE => {
            ps.view_height_lerp_down = 1;
            ps.view_height_lerp_target = if ps.view_height_current <= CROUCH_HEIGHT {
                view_height::PRONE
            } else {
                view_height::CROUCH
            };
        }
        view_height::CROUCH => {
            if (target as f32) < ps.view_height_current {
                ps.view_height_lerp_down = 1;
                ps.view_height_lerp_target = view_height::CROUCH;
            } else {
                ps.view_height_lerp_down = 0;
                ps.view_height_lerp_target = view_height::CROUCH;
            }
        }
        view_height::STAND => {
            ps.view_height_lerp_down = 0;
            ps.view_height_lerp_target = if CROUCH_HEIGHT <= ps.view_height_current {
                view_height::STAND
            } else {
                view_height::CROUCH
            };
        }
        _ => {}
    }
}

fn reverse_view_height_lerp(ps: &mut PlayerState, cmd: &UserCmd, progress: i32, prone_ms: i32) {
    let target = ps.view_height_target;
    let lerp_target = ps.view_height_lerp_target;
    if target == lerp_target {
        return;
    }

    let down = if target < lerp_target {
        if ps.view_height_lerp_down != 0 {
            return;
        }
        0
    } else {
        if ps.view_height_lerp_down == 0 {
            return;
        }
        ps.view_height_lerp_down
    };

    let progress = 100 - progress;
    let down = down ^ 1;
    ps.view_height_lerp_down = down;
    if down == 0 {
        match lerp_target {
            view_height::PRONE => ps.view_height_lerp_target = view_height::CROUCH,
            view_height::CROUCH => ps.view_height_lerp_target = view_height::STAND,
            _ => {}
        }
    } else {
        match lerp_target {
            view_height::STAND => ps.view_height_lerp_target = view_height::CROUCH,
            view_height::CROUCH => ps.view_height_lerp_target = view_height::PRONE,
            _ => {}
        }
    }

    if progress == 100 {
        ps.view_height_lerp_time = 0;
        ps.view_height_current = ps.view_height_lerp_target as f32;
        return;
    }

    let duration = lerp_duration(ps, ps.view_height_lerp_target, down, prone_ms);
    let elapsed = ((duration as f32) * (progress as f32) * REVERSE_REWIND_SCALE) as i32;
    ps.view_height_lerp_time = cmd.server_time.wrapping_sub(elapsed);
}

#[must_use]
pub fn update_stance_target(ps: &mut PlayerState) -> StanceChange {
    if ps.view_height_lerp_time != 0 {
        return StanceChange::Unchanged;
    }

    if ps.pm_type == 7 {
        if ps.view_height_target == view_height::LAST_STAND {
            return StanceChange::Unchanged;
        }
        ps.view_height_target = view_height::LAST_STAND;
    } else if (ps.pm_flags & pm_flags::PRONE) == 0 {
        let previous = ps.view_height_target;
        if previous == view_height::PRONE {
            ps.view_height_target = view_height::CROUCH;
        } else {
            ps.view_height_target = if (ps.pm_flags & pm_flags::CROUCH) != 0 {
                view_height::CROUCH
            } else {
                view_height::STAND
            };
            if previous != view_height::LAST_STAND {
                return StanceChange::Unchanged;
            }
        }
    } else {
        if ps.view_height_target == view_height::STAND {
            ps.view_height_target = view_height::CROUCH;
            return StanceChange::Unchanged;
        }
        if ps.view_height_target == view_height::PRONE {
            return StanceChange::Unchanged;
        }
        ps.view_height_target = view_height::PRONE;
        return StanceChange::EnteredProne;
    }

    StanceChange::Changed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StanceChange {
    Unchanged,

    Changed,

    EnteredProne,
}

#[must_use]
pub fn stance_speed_scale(
    ps: &PlayerState,
    server_time: i32,
    last_stand_scale: f32,
    prone_lerp_ms: i32,
) -> f32 {
    const PRONE_SCALE: f32 = 0.15;

    const CROUCH_SCALE: f32 = 0.65;

    if ps.view_height_lerp_time != 0 && ps.view_height_lerp_target == view_height::PRONE {
        let transition_ms = lerp_duration(ps, view_height::PRONE, 1, prone_lerp_ms) as f32;
        let fraction = (server_time.wrapping_sub(ps.view_height_lerp_time) as f32) / transition_ms;
        if fraction >= 0.0 {
            let fraction = if fraction > 1.0 { 1.0 } else { fraction };
            if fraction != 0.0 {
                return fraction * PRONE_SCALE + (1.0 - fraction) * CROUCH_SCALE;
            }
        }
    }

    if ps.view_height_lerp_time != 0
        && ps.view_height_lerp_target == view_height::CROUCH
        && ps.view_height_lerp_down == 0
    {
        let transition_ms = lerp_duration(ps, view_height::CROUCH, 0, prone_lerp_ms) as f32;
        let fraction = (server_time.wrapping_sub(ps.view_height_lerp_time) as f32) / transition_ms;
        if fraction >= 0.0 {
            let fraction = if fraction > 1.0 { 1.0 } else { fraction };
            if fraction != 0.0 {
                return fraction * CROUCH_SCALE + (1.0 - fraction) * PRONE_SCALE;
            }
        }
    }

    match stance_surface_type(ps) {
        StanceSurface::LastStand => last_stand_scale,
        StanceSurface::Crouch => CROUCH_SCALE,
        StanceSurface::Prone => PRONE_SCALE,
        StanceSurface::Stand => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trace_iw4::Trace;

    /// Someone standing over the player: only a trace that hits bodies
    /// (0x0200_0000) is stuck.
    struct BodyOverhead;

    impl CollisionBackend for BodyOverhead {
        fn trace(&self, input: GroundTraceInput) -> Trace {
            Trace {
                fraction: 1.0,
                endpos: input.end,
                allsolid: u8::from((input.tracemask & 0x0200_0000) != 0),
                ..Trace::default()
            }
        }
    }

    /// BO2's stand-up and crouch-up checks leave bodies out: another player
    /// or a zombie never keeps you down.
    #[test]
    fn bo2_gets_up_under_a_body() {
        let bounds = MoveBounds {
            mins: [-15.0, -15.0, 0.0],
            maxs: [15.0, 15.0, PRONE_MAXS_Z],
            tracemask: 0x0281_0011,
        };
        let origin = [0.0; 3];
        assert!(stance_hull_allsolid(&BodyOverhead, origin, bounds, STAND_MAXS_Z, false));
        assert!(!stance_hull_allsolid(&BodyOverhead, origin, bounds, STAND_MAXS_Z, true));
    }

    /// Ground 4 units under the start, rising toward +x: a trace down stops
    /// on it, a trace up is clear.
    struct Ramp;

    const RAMP_NORMAL: [f32; 3] = [-0.196_116_14, 0.0, 0.980_580_7];

    impl CollisionBackend for Ramp {
        fn trace(&self, input: GroundTraceInput) -> Trace {
            let floor = -4.0;
            if input.end[2] < input.start[2] && input.end[2] <= floor {
                let fraction = (input.start[2] - floor) / (input.start[2] - input.end[2]);
                return Trace {
                    fraction,
                    endpos: [input.start[0], input.start[1], floor],
                    normal: RAMP_NORMAL,
                    walkable: 1,
                    ..Trace::default()
                };
            }
            Trace {
                fraction: 1.0,
                endpos: input.end,
                ..Trace::default()
            }
        }
    }

    fn lying(pm_type: i32, view: [f32; 3], was_prone: bool) -> PlayerState {
        let bounds = MoveBounds {
            mins: [-15.0, -15.0, 0.0],
            maxs: [15.0, 15.0, PRONE_MAXS_Z],
            tracemask: 0x0281_0011,
        };
        let mut ps = PlayerState {
            pm_type,
            pm_flags: pm_flags::PRONE,
            origin: [0.0, 0.0, -3.9],
            viewangles: view,
            ..PlayerState::ZERO
        };
        bo2_lie_down(&mut ps, &UserCmd::default(), &Ramp, bounds, was_prone);
        ps
    }

    /// Lying down on a ramp: the body points where you look and takes the
    /// ramp's slope; the torso stays within 45 of the view.
    #[test]
    fn bo2_lying_down_takes_the_slope() {
        let ps = lying(0, [0.0, 30.0, 0.0], false);
        assert_eq!(ps.prone_direction, 30.0);
        let slope = math_iw4::pitch_for_yaw_on_normal(30.0, RAMP_NORMAL);
        assert!(slope < -5.0, "{slope}");
        assert_eq!(ps.prone_direction_pitch, slope);
        assert_eq!(ps.prone_torso_pitch, slope);
        let ps = lying(0, [60.0, 30.0, 0.0], false);
        assert_eq!(ps.prone_torso_pitch, 60.0 - PRONE_TORSO_PITCH_RANGE);
    }

    /// Already lying: nothing changes; last stand lies flat and turns with
    /// the view every frame.
    #[test]
    fn bo2_last_stand_lies_flat_and_follows_the_view() {
        let ps = lying(0, [0.0, 30.0, 0.0], true);
        assert_eq!((ps.prone_direction, ps.prone_direction_pitch), (0.0, 0.0));
        let ps = lying(PM_TYPE_LAST_STAND, [0.0, 30.0, 0.0], false);
        assert_eq!((ps.prone_direction, ps.prone_direction_pitch), (30.0, 0.0));
        let ps = lying(PM_TYPE_LAST_STAND, [0.0, 75.0, 0.0], true);
        assert_eq!(ps.prone_direction, 75.0);
    }
}
