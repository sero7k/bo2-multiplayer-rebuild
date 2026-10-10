//! bo2zm: Black Ops II's dive to prone (the "dolphin dive"). Not part of
//! upstream IW4L.
//!
//! Built step for step on BO2's own dive: while sprinting on the ground,
//! pressing prone (or holding a pad's stance button a moment) throws the
//! player up at twice a jump's launch speed, keeping the run's speed and
//! direction. From the next frame the view drops to prone (standing to
//! crouched, then crouched to lying, the usual two steps) and the body
//! shrinks with it: standing until the view is below crouch height, then
//! crouched, then prone, so a dive goes through a window. At a jump's height
//! the dive stops rising, and for its first 0.4 s it hangs there. In the air
//! the player steers with the usual air control. On touch-down they slide on
//! their front: no friction at first, the ground's full friction by 0.3 s,
//! when the slide ends (no friction over a ledge top they could climb). A
//! slide that comes to a stop (into a wall) ends at once. The player then
//! lies still for 0.1 s and stays prone until they press jump, crouch, prone
//! or sprint. Getting up from a dive is twice as quick as from an ordinary
//! prone (by request; BO2's own quick get-up is sprinting out of it).
//! A get-up pressed during the dive is kept, so the player rises the moment
//! it is over.
//!
//! The numbers are BO2's dive dvar defaults: `dtp_startup_delay` 250 ms of
//! sprint before a dive, `dtp_min_speed` 3.16, `dtp_new_trajectory_multiplier`
//! 2, `dtp_max_apex_duration` 400 ms, `dtp_max_slide_duration` 300 ms,
//! `dtp_post_move_pause` 100 ms and `dtp_exhaustion_window` 1.5 s from one
//! dive's end to the next dive.
//!
//! State rides in the replicated player state, so the host and the local
//! prediction agree: `PMF_DIVE` in the air, `PMF_DIVE_SLIDE` sliding,
//! `PMF_DIVE_PRONE` held prone after the slide (the still moment included),
//! `PMF_DIVE_GETUP` from a get-up press until the player is up; `jump_time` =
//! take-off, then touch-down; `jump_origin_z` = the take-off height;
//! `dive_end_time` = when the slide ended.

use playerstate_iw4::{ENTITYNUM_NONE, PlayerState, UserCmd, buttons, eflags, pm_flags};

use crate::stance::{CROUCH_MAXS_Z, PRONE_MAXS_Z, STAND_MAXS_Z};
use crate::{
    AirMoveContext, CONTENTS_MANTLE, CollisionBackend, GroundTraceInput, MoveBounds, Pml,
    add_predictable_event, end_sprint, view_height,
};

/// In the air, diving.
pub const PMF_DIVE: u32 = 0x0100_0000;
/// On the ground after a dive, sliding.
pub const PMF_DIVE_SLIDE: u32 = 0x0200_0000;
/// After a dive's slide: the player lies still a moment, then stays prone
/// (as BO2 leaves them) until they press jump, crouch, prone or sprint,
/// whatever the keys' hold or toggle style.
pub const PMF_DIVE_PRONE: u32 = 0x0400_0000;
/// Getting up from a dive (or waiting to, once it ends): the stance change
/// runs at twice the speed, and a held prone key does not drop the player
/// back down until it is let go.
pub const PMF_DIVE_GETUP: u32 = 0x0800_0000;

/// A dive starts only after more than this long sprinting
/// (`dtp_startup_delay`).
const STARTUP_MS: i32 = 250;
/// A dive starts only above this ground speed, and a slide slower than this
/// is over (`dtp_min_speed`).
const MIN_SPEED: f32 = 3.16;
/// Another dive waits until more than this long after the last one ended
/// (`dtp_exhaustion_window`).
const EXHAUSTION_MS: i32 = 1500;
/// The take-off's upward speed is this many times a jump's
/// (`dtp_new_trajectory_multiplier`).
const POP_SCALE: f32 = 2.0;
/// For this long after take-off a dive at a jump's height hangs there
/// (`dtp_max_apex_duration`).
const GLIDE_MS: i32 = 400;
/// The slide's friction comes in over this long, then the slide ends
/// (`dtp_max_slide_duration`).
const SLIDE_MS: i32 = 300;
/// The still moment after the slide (`dtp_post_move_pause`).
const PAUSE_MS: i32 = 100;
/// Pushback flag: no ground friction while it is on (cleared by a dive).
const PMF_TIME_KNOCKBACK: u32 = 0x100;

/// A dive is under way: in the air or sliding.
pub fn active(ps: &PlayerState) -> bool {
    ps.pm_flags & (PMF_DIVE | PMF_DIVE_SLIDE) != 0
}

/// BO2's `divetoprone` for scripts: from take-off until the slide ends.
/// PhD Flopper's blast and the dive notifies read this.
pub fn dive_to_prone(ps: &PlayerState) -> bool {
    active(ps)
}

/// Any part of a dive, up to the player being back on their feet.
pub fn any(ps: &PlayerState) -> bool {
    ps.pm_flags & (PMF_DIVE | PMF_DIVE_SLIDE | PMF_DIVE_PRONE | PMF_DIVE_GETUP) != 0
}

/// The still moment after a dive's slide: the player's push is ignored.
pub fn pausing(ps: &PlayerState, now: i32) -> bool {
    !active(ps)
        && ps.dive_end_time != 0
        && (0..PAUSE_MS).contains(&now.wrapping_sub(ps.dive_end_time))
}

/// The dive owns the stance, from the frame after take-off to the end of the
/// still moment after the slide.
pub fn owns_stance(ps: &PlayerState, now: i32) -> bool {
    active(ps) || pausing(ps, now)
}

/// BO2 aims the view at prone through the whole dive; it gets there by the
/// usual steps, standing to crouched, then crouched to lying.
pub fn force_prone(ps: &mut PlayerState) {
    ps.e_flags &= !eflags::DUCK;
    ps.pm_flags = (ps.pm_flags & !pm_flags::CROUCH) | pm_flags::PRONE;
    ps.view_height_target = view_height::PRONE;
}

/// The body's height through a dive, from where the view has dropped to:
/// standing until the view is below crouch height, crouched until it is
/// below prone height, then prone.
pub fn hull_height(ps: &PlayerState) -> f32 {
    let view = ps.view_height_current;
    if view < PRONE_MAXS_Z {
        PRONE_MAXS_Z
    } else if view < CROUCH_MAXS_Z {
        CROUCH_MAXS_Z
    } else {
        STAND_MAXS_Z
    }
}

/// Whether this command starts a dive: sprinting on the ground long enough
/// and fast enough, and prone newly pressed.
pub fn wants(ps: &PlayerState, cmd: &UserCmd, old_buttons: u32) -> bool {
    if active(ps)
        || ps.pm_flags & pm_flags::SPRINTING == 0
        || ps.pm_flags & (pm_flags::LADDER | pm_flags::MANTLE | pm_flags::LAST_STAND) != 0
        || ps.ground_entity_num == ENTITYNUM_NONE
        || (ps.pm_type != 0 && ps.pm_type != 1)
    {
        return false;
    }
    // Prone newly pressed. A pad's stance button sends prone once held
    // `cl_dtpHoldTime` while sprinting (input_iw4); a crouch only crouches.
    if (cmd.buttons & !old_buttons) & buttons::PRONE == 0 {
        return false;
    }
    if cmd.server_time.wrapping_sub(ps.last_sprint_start) <= STARTUP_MS {
        return false;
    }
    if ps.dive_end_time != 0
        && (0..=EXHAUSTION_MS).contains(&cmd.server_time.wrapping_sub(ps.dive_end_time))
    {
        return false;
    }
    ground_speed(ps) > MIN_SPEED
}

fn ground_speed(ps: &PlayerState) -> f32 {
    libm::sqrtf(ps.velocity[0] * ps.velocity[0] + ps.velocity[1] * ps.velocity[1])
}

/// The take-off's upward speed: `POP_SCALE` times a jump's.
fn pop_speed(ps: &PlayerState, jump_height: f32) -> f32 {
    POP_SCALE * libm::sqrtf(2.0 * jump_height * ps.gravity as f32)
}

/// Launch the dive: sprint ends and the player leaves the ground at their
/// running speed and direction, with `pop_speed` upward. The stance is left
/// as it is this frame; the drop to prone starts on the next.
pub fn start(ps: &mut PlayerState, pml: &mut Pml, cmd: &UserCmd, jump_height: f32) {
    end_sprint(ps, cmd);
    ps.velocity[2] = pop_speed(ps, jump_height);
    ps.pm_flags = (ps.pm_flags
        & !(pm_flags::TIME_HARDLANDING
            | PMF_TIME_KNOCKBACK
            | pm_flags::JUMPING
            | PMF_DIVE_PRONE
            | PMF_DIVE_GETUP))
        | PMF_DIVE;
    ps.pm_time = 0;
    ps.ground_entity_num = ENTITYNUM_NONE;
    ps.jump_time = cmd.server_time;
    ps.jump_origin_z = ps.origin[2];
    pml.walking = 0;
    pml.ground_plane = 0;
    pml.almost_ground_plane = 0;
}

/// A key that gets the player up from a dive: jump, crouch, prone or sprint
/// newly pressed, or a stance toggle switched off (a pad, or toggle-style
/// keys).
fn getup_pressed(cmd: &UserCmd, old_buttons: u32) -> bool {
    let new = cmd.buttons & !old_buttons;
    if new & (buttons::JUMP | buttons::CROUCH | buttons::PRONE | buttons::SPRINT) != 0 {
        return true;
    }
    old_buttons & buttons::STANCE_HELD == 0
        && old_buttons & !cmd.buttons & (buttons::PRONE | buttons::CROUCH) != 0
}

/// While the dive owns the stance, a get-up press is kept for when it ends.
pub fn queue_getup(ps: &mut PlayerState, cmd: &UserCmd, old_buttons: u32) {
    if getup_pressed(cmd, old_buttons) {
        ps.pm_flags |= PMF_DIVE_GETUP;
    }
}

/// Getting up: a prone toggle still on would drop the player straight back
/// down, so the client is told to let it go (as when a sprint stands a
/// prone player up).
fn begin_getup(ps: &mut PlayerState, cmd: &UserCmd) {
    ps.pm_flags = (ps.pm_flags & !PMF_DIVE_PRONE) | PMF_DIVE_GETUP;
    if cmd.buttons & buttons::PRONE != 0 && cmd.buttons & buttons::STANCE_HELD == 0 {
        add_predictable_event(ps, EV_STANCE_FORCE_STAND, 0);
    }
}

const EV_STANCE_FORCE_STAND: i32 = 6;

/// A slide that has come to a stop (into a wall) is over at once. Run
/// before the stance update.
pub fn end_stopped_slide(ps: &mut PlayerState, now: i32) {
    if ps.pm_flags & PMF_DIVE_SLIDE != 0 && speed_3d(ps) < MIN_SPEED {
        end_slide(ps, now);
    }
}

/// The slide is over: the player stops, lies still for `PAUSE_MS`, then
/// holds prone.
fn end_slide(ps: &mut PlayerState, now: i32) {
    ps.pm_flags = (ps.pm_flags & !(PMF_DIVE | PMF_DIVE_SLIDE)) | PMF_DIVE_PRONE;
    ps.velocity = [0.0; 3];
    ps.jump_time = 0;
    ps.dive_end_time = now;
    ps.sprint_button_up_required = 0;
}

/// Move one frame of a dive: touch-down starts the slide, the dive does not
/// rise above a jump's height (and hangs there early on), then the ordinary
/// air move runs, its ground friction scaled through the slide.
pub fn advance<C: CollisionBackend>(
    ps: &mut PlayerState,
    pml: &Pml,
    cmd: &UserCmd,
    bounds: MoveBounds,
    collision: &C,
    jump_height: f32,
    air: AirMoveContext,
) {
    let now = cmd.server_time;
    if ps.pm_flags & PMF_DIVE != 0 && pml.walking != 0 {
        ps.pm_flags = (ps.pm_flags & !PMF_DIVE) | PMF_DIVE_SLIDE;
        ps.jump_time = now;
    }
    if ps.origin[2] >= ps.jump_origin_z + jump_height {
        if ps.pm_flags & PMF_DIVE != 0 && now.wrapping_sub(ps.jump_time) < GLIDE_MS {
            ps.velocity[2] = 0.0;
        }
        ps.velocity[2] = ps.velocity[2].min(0.0);
    }
    crate::air::air_move_with(ps, pml, cmd, air, bounds, collision, true, |ps| {
        slide_friction(ps, now, collision, bounds)
    });
}

/// The slide's share of the ground's friction: none at touch-down, rising
/// to all of it at `SLIDE_MS`, when the slide ends; none over a ledge top
/// the player could climb (mantle ground). Asked only when ground friction
/// applies.
fn slide_friction<C: CollisionBackend>(
    ps: &mut PlayerState,
    now: i32,
    collision: &C,
    bounds: MoveBounds,
) -> f32 {
    if ps.pm_flags & PMF_DIVE_SLIDE == 0 {
        return 1.0;
    }
    let elapsed = now.wrapping_sub(ps.jump_time);
    if elapsed < SLIDE_MS {
        if on_mantle_ground(ps, collision, bounds) {
            return 0.0;
        }
        return elapsed.max(0) as f32 / SLIDE_MS as f32;
    }
    end_slide(ps, now);
    1.0
}

fn speed_3d(ps: &PlayerState) -> f32 {
    let v = ps.velocity;
    libm::sqrtf(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
}

/// Ground the player could mantle (a ledge top) under them.
fn on_mantle_ground<C: CollisionBackend>(
    ps: &PlayerState,
    collision: &C,
    bounds: MoveBounds,
) -> bool {
    let o = ps.origin;
    collision
        .trace(GroundTraceInput {
            start: [o[0], o[1], o[2] + 1.0],
            end: [o[0], o[1], o[2] - 1.0],
            mins: bounds.mins,
            maxs: bounds.maxs,
            tracemask: CONTENTS_MANTLE,
        })
        .fraction
        < 1.0
}

/// After a dive the player holds prone until a get-up key (or one pressed
/// during the dive, or something else stood them up); `true` while it
/// holds, so the stance update is skipped.
pub fn holds_prone(ps: &mut PlayerState, cmd: &UserCmd, old_buttons: u32) -> bool {
    if ps.pm_flags & PMF_DIVE_PRONE == 0 {
        return false;
    }
    if ps.pm_flags & PMF_DIVE_GETUP != 0
        || getup_pressed(cmd, old_buttons)
        || ps.pm_flags & pm_flags::SPRINTING != 0
    {
        begin_getup(ps, cmd);
        return false;
    }
    if ps.pm_flags & pm_flags::PRONE == 0 {
        ps.pm_flags &= !PMF_DIVE_PRONE;
        return false;
    }
    true
}

/// Getting up from a dive, as opposed to a get-up only queued while the dive
/// still owns the stance.
pub fn getting_up(ps: &PlayerState) -> bool {
    ps.pm_flags & PMF_DIVE_GETUP != 0
        && ps.pm_flags & (PMF_DIVE | PMF_DIVE_SLIDE | PMF_DIVE_PRONE) == 0
}

/// The command the stance and sprint updates see: while getting up from a
/// dive, a held prone key is left out, so it does not drop the player back
/// down (a hold-style key pressed to dive, or a prone toggle the client has
/// not yet let go).
pub fn stance_cmd(ps: &PlayerState, cmd: &UserCmd) -> UserCmd {
    let mut out = *cmd;
    if ps.pm_flags & PMF_DIVE_GETUP != 0 {
        out.buttons &= !buttons::PRONE;
    }
    out
}

/// The command the aim update sees: the sprint key, held through a dive,
/// does not drop the aim, and the player's own push (it only steers a dive)
/// does not count as crawling.
pub fn ads_cmd(ps: &PlayerState, cmd: &UserCmd, diving: bool) -> UserCmd {
    let mut out = *cmd;
    if diving || ps.pm_flags & PMF_DIVE_PRONE != 0 {
        out.buttons &= !buttons::SPRINT;
    }
    if diving {
        out.forwardmove = 0;
        out.rightmove = 0;
    }
    out
}

/// The get-up is over once the view has come to rest out of prone (both
/// steps, lying to crouched and crouched to standing, are done) and the
/// prone key is let go. Run before the view update, so a step that has just
/// ended and the next one not yet begun does not end it early.
pub fn finish_getup(ps: &mut PlayerState, cmd: &UserCmd) {
    if !getting_up(ps) {
        return;
    }
    if ps.view_height_lerp_time == 0
        && ps.view_height_target != view_height::PRONE
        && ps.view_height_current == ps.view_height_target as f32
        && cmd.buttons & buttons::PRONE == 0
    {
        ps.pm_flags &= !PMF_DIVE_GETUP;
    }
}
