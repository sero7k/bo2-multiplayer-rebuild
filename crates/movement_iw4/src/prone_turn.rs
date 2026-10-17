//! bo2zm: Black Ops II's limits while lying down (audit D4).
//!
//! The body has its own yaw (`prone_direction`). Looking more than 40 off it,
//! or moving while it is off at all, swings it toward the view at 55 degrees
//! a second; the view itself stays within 45 of it, so a fast turn lying down
//! is held back to the body's swing. Where the body cannot turn (a wall at
//! the feet), the view is pulled back toward it. The view's pitch stays
//! within 45 of the torso's slope pitch. Each tick the body's and torso's
//! slope pitches ease toward the ground under them at 70 degrees a second,
//! and a body that no longer fits is told to crouch.
//!
//! Any change the code makes to the view is made to `delta_angles` too, so
//! the next command's angles land on it.

use math_iw4::{angle_normalize_360, angle_subtract, pitch_for_yaw_on_normal};
use playerstate_iw4::{
    ENTITYNUM_NONE, PM_TYPE_DEAD, PM_TYPE_INTERMISSION, PM_TYPE_LAST_STAND, PM_TYPE_NORMAL_LINKED,
    PlayerState, UserCmd, pm_flags,
};

use crate::stance::view_height;
use crate::{
    CollisionBackend, PRONE_CHECK_HEIGHT, PRONE_FEET_DIST, Pml, ProneRules, add_predictable_event,
    check_prone,
};

/// BO2's `bg_prone_yawcap`: the view stays this close to the body's yaw.
const PRONE_YAW_CAP: f32 = 45.0;

/// The body swings toward the view once it is this far off (the cap less 5).
const PRONE_SWING_TRIGGER: f32 = PRONE_YAW_CAP - 5.0;

/// How fast the body swings toward the view, degrees a second.
const PRONE_SWING_SPEED: f32 = 55.0;

/// The view's pitch stays this close to the torso's slope pitch.
const PRONE_PITCH_CAP: f32 = 45.0;

/// How fast the slope pitches ease toward the ground, degrees a second.
const PRONE_PITCH_SPEED: f32 = 70.0;

/// The body's half width for the turn checks.
const PRONE_SIZE: f32 = 15.0;

/// BO2's `PRONE_FEET_DIST_TURNED`.
const PRONE_FEET_DIST_TURNED: f32 = 55.0;

/// A turned body is checked part way between the view and the new yaw, by
/// this much per degree of turn.
const PRONE_TURN_BLEND: f32 = 0.004_166_666_7;

/// A blocked turn nudges the view back this share of the last tick's turn.
const PRONE_NUDGE: f32 = 0.98;

/// A blocked turn is walked back, and the view dragged, a degree at a time.
const PRONE_STEP: f32 = 1.0;

/// Black Ops II's force-crouch event (our number).
const EV_STANCE_FORCE_CROUCH: i32 = 7;

/// The last-stand body is told to crouch when bent more than this.
const LAST_STAND_BEND_LIMIT: f32 = 50.0;

fn wrap(degrees: f32) -> f32 {
    angle_subtract(degrees, 0.0)
}

fn rules(ps: &PlayerState, zombies: bool) -> ProneRules {
    ProneRules::for_player(ps, zombies)
}

/// The body fits lying along `yaw`, the turn's longer feet.
fn fits<C: CollisionBackend>(ps: &PlayerState, collision: &C, zombies: bool, yaw: f32) -> bool {
    check_prone(
        collision,
        ps.origin,
        PRONE_SIZE,
        PRONE_CHECK_HEIGHT,
        yaw,
        true,
        ps.ground_entity_num != ENTITYNUM_NONE,
        true,
        rules(ps, zombies),
        PRONE_FEET_DIST_TURNED,
        None,
        None,
    )
}

/// BO2's turned check: the body turned to `yaw` is checked part way back
/// toward the view, with feet a little longer the more it turns; it keeps
/// the body's torso and waist pitches.
fn turned_fits<C: CollisionBackend>(
    ps: &mut PlayerState,
    collision: &C,
    zombies: bool,
    yaw: f32,
) -> bool {
    let turn = angle_subtract(yaw, ps.viewangles[1]);
    let share = 1.0 - turn.abs() * PRONE_TURN_BLEND;
    let check_yaw = angle_normalize_360(yaw - share * turn);
    let feet = PRONE_FEET_DIST * share + PRONE_FEET_DIST_TURNED * turn.abs() * PRONE_TURN_BLEND;
    let (mut torso, mut waist) = (ps.f_torso_pitch, ps.f_waist_pitch);
    let ok = check_prone(
        collision,
        ps.origin,
        PRONE_SIZE,
        PRONE_CHECK_HEIGHT,
        check_yaw,
        true,
        ps.ground_entity_num != ENTITYNUM_NONE,
        true,
        rules(ps, zombies),
        feet,
        Some(&mut torso),
        Some(&mut waist),
    );
    ps.f_torso_pitch = torso;
    ps.f_waist_pitch = waist;
    ok
}

/// BO2's prone view limits, run right after the command's angles are put on
/// the view; `old_yaw` is the view's yaw before them.
pub fn bo2_prone_view<C: CollisionBackend>(
    ps: &mut PlayerState,
    cmd: &UserCmd,
    msec: i32,
    collision: &C,
    zombies: bool,
    old_yaw: f32,
) {
    if ps.pm_flags & pm_flags::PRONE == 0
        || ps.pm_flags & pm_flags::MANTLE != 0
        || matches!(ps.pm_type, PM_TYPE_NORMAL_LINKED | 2 | 3 | PM_TYPE_INTERMISSION)
        || ps.pm_type >= PM_TYPE_DEAD
    {
        return;
    }
    let view_yaw0 = ps.viewangles[1];
    let mut blocked = false;

    // Swing the body toward the view.
    let off = angle_subtract(ps.prone_direction, ps.viewangles[1]);
    let moving = cmd.forwardmove != 0 || cmd.rightmove != 0;
    if off.abs() > PRONE_SWING_TRIGGER || (moving && off != 0.0) {
        let swing = msec as f32 * PRONE_SWING_SPEED * 0.001;
        let mut yaw = if swing <= off.abs() {
            if off > 0.0 {
                ps.prone_direction - swing
            } else {
                ps.prone_direction + swing
            }
        } else {
            ps.viewangles[1]
        };
        let mut gave_up = false;
        let mut turned = true;
        while !turned_fits(ps, collision, zombies, yaw) {
            if gave_up {
                turned = false;
                break;
            }
            let back = angle_subtract(ps.prone_direction, yaw);
            let step = if back.abs() <= PRONE_STEP {
                blocked = true;
                gave_up = true;
                back
            } else if back > 0.0 {
                PRONE_STEP
            } else {
                -PRONE_STEP
            };
            yaw = angle_normalize_360(yaw + step);
        }
        if turned {
            if fits(ps, collision, zombies, ps.viewangles[1]) && fits(ps, collision, zombies, yaw) {
                ps.prone_direction = yaw;
            } else {
                blocked = true;
            }
        }
    }

    // Where the body cannot follow, drag the view back toward it.
    let mut off = angle_subtract(ps.prone_direction, ps.viewangles[1]);
    if off != 0.0 {
        let mut yaw = ps.prone_direction;
        let mut gave_up = false;
        loop {
            let ok = fits(ps, collision, zombies, yaw);
            if ok && turned_fits(ps, collision, zombies, yaw) {
                ps.prone_direction = yaw;
                break;
            }
            if gave_up {
                break;
            }
            let step = if off.abs() <= PRONE_STEP {
                gave_up = true;
                off
            } else if off > 0.0 {
                PRONE_STEP
            } else {
                -PRONE_STEP
            };
            ps.delta_angles[1] += step;
            blocked = true;
            ps.viewangles[1] = wrap(ps.viewangles[1] + step);
            off = angle_subtract(ps.prone_direction, ps.viewangles[1]);
            if !ok {
                yaw = angle_normalize_360(yaw + off);
            }
        }
    }

    yaw_clamp(ps, off, blocked, old_yaw, view_yaw0);
    pitch_clamp(ps);
}

/// The view stays within 45 of the body's yaw; a blocked turn nudges it
/// back by most of a last small turn.
fn yaw_clamp(ps: &mut PlayerState, off: f32, blocked: bool, old_yaw: f32, view_yaw0: f32) {
    let excess = if off > PRONE_YAW_CAP {
        off - PRONE_YAW_CAP
    } else if off < -PRONE_YAW_CAP {
        off + PRONE_YAW_CAP
    } else {
        0.0
    };
    if excess != 0.0 {
        ps.delta_angles[1] += excess;
        ps.viewangles[1] = wrap(if excess > 0.0 {
            ps.prone_direction - PRONE_YAW_CAP
        } else {
            ps.prone_direction + PRONE_YAW_CAP
        });
    }
    // BO2 also flags "prone blocked" here for the HUD (deferred).
    if blocked {
        let back = angle_subtract(old_yaw, ps.viewangles[1]);
        if back.abs() <= PRONE_STEP && angle_subtract(view_yaw0, ps.viewangles[1]) * back > 0.0 {
            ps.viewangles[1] = wrap(ps.viewangles[1] + back * PRONE_NUDGE);
            ps.delta_angles[1] += back * PRONE_NUDGE;
        }
    }
}

/// The view's pitch stays within 45 of the torso's slope pitch.
fn pitch_clamp(ps: &mut PlayerState) {
    let off = angle_subtract(ps.prone_torso_pitch, ps.viewangles[0]);
    let excess = if off > PRONE_PITCH_CAP {
        off - PRONE_PITCH_CAP
    } else if off < -PRONE_PITCH_CAP {
        off + PRONE_PITCH_CAP
    } else {
        return;
    };
    ps.delta_angles[0] += excess;
    ps.viewangles[0] = wrap(if excess > 0.0 {
        ps.prone_torso_pitch - PRONE_PITCH_CAP
    } else {
        ps.prone_torso_pitch + PRONE_PITCH_CAP
    });
}

fn ease(current: f32, target: f32, frametime: f32) -> f32 {
    let off = angle_subtract(target, current);
    if off == 0.0 {
        return current;
    }
    let most = frametime * PRONE_PITCH_SPEED;
    let next = if off.abs() <= most {
        current + off
    } else if off < 0.0 {
        current - most
    } else {
        current + most
    };
    wrap(next)
}

/// BO2's `PM_UpdatePronePitch`, after this tick's ground trace: a lying body
/// that no longer fits in the air, lies on ground too steep, or (last stand)
/// is bent too far is told to crouch; its slope pitches ease toward the
/// ground under it.
pub fn bo2_prone_pitch<C: CollisionBackend>(
    ps: &mut PlayerState,
    pml: &Pml,
    collision: &C,
    zombies: bool,
) {
    if ps.pm_flags & pm_flags::PRONE == 0 {
        return;
    }
    let walkable = (pml.ground_trace[10] >> 16) & 0xff != 0;
    let last_stand_view = ps.view_height_target == view_height::LAST_STAND;
    let crouch = if ps.ground_entity_num == ENTITYNUM_NONE {
        if last_stand_view {
            true
        } else {
            let (mut torso, mut waist) = (ps.f_torso_pitch, ps.f_waist_pitch);
            let ok = check_prone(
                collision,
                ps.origin,
                PRONE_SIZE,
                PRONE_CHECK_HEIGHT,
                ps.prone_direction,
                true,
                false,
                pml.ground_plane == 0 || walkable,
                rules(ps, zombies),
                PRONE_FEET_DIST,
                Some(&mut torso),
                Some(&mut waist),
            );
            ps.f_torso_pitch = torso;
            ps.f_waist_pitch = waist;
            // BO2 also flags "prone blocked" here for the HUD (deferred);
            // deep water crouches too (no water yet).
            !ok
        }
    } else {
        (pml.ground_plane != 0 && !walkable)
            || (last_stand_view
                && (ps.f_torso_pitch.abs() > LAST_STAND_BEND_LIMIT
                    || ps.f_waist_pitch.abs() > LAST_STAND_BEND_LIMIT))
    };
    if crouch {
        add_predictable_event(ps, EV_STANCE_FORCE_CROUCH, 0);
    }

    let normal = [
        f32::from_bits(pml.ground_trace[1]),
        f32::from_bits(pml.ground_trace[2]),
        f32::from_bits(pml.ground_trace[3]),
    ];
    let body_yaw = if ps.pm_type == PM_TYPE_LAST_STAND {
        ps.viewangles[1]
    } else {
        ps.prone_direction
    };
    let (body, torso) = if pml.ground_plane == 0 {
        (0.0, 0.0)
    } else {
        (
            pitch_for_yaw_on_normal(body_yaw, normal),
            pitch_for_yaw_on_normal(ps.viewangles[1], normal),
        )
    };
    ps.prone_direction_pitch = ease(ps.prone_direction_pitch, body, pml.frametime);
    ps.prone_torso_pitch = ease(ps.prone_torso_pitch, torso, pml.frametime);
}

#[cfg(test)]
mod tests {
    use playerstate_iw4::{PlayerState, UserCmd, pm_flags};
    use trace_iw4::Trace;

    use super::*;
    use crate::GroundTraceInput;

    /// Nothing anywhere.
    struct Open;

    impl CollisionBackend for Open {
        fn trace(&self, input: GroundTraceInput) -> Trace {
            Trace {
                fraction: 1.0,
                endpos: input.end,
                ..Trace::default()
            }
        }
    }

    /// Solid everywhere: every trace stops where it starts.
    struct Stuck;

    impl CollisionBackend for Stuck {
        fn trace(&self, input: GroundTraceInput) -> Trace {
            Trace {
                fraction: 0.0,
                endpos: input.start,
                startsolid: 1,
                ..Trace::default()
            }
        }
    }

    /// Lying in the air (so the open world fits), body along `dir`.
    fn lying(dir: f32, view: [f32; 3]) -> PlayerState {
        PlayerState {
            pm_flags: pm_flags::PRONE,
            ground_entity_num: ENTITYNUM_NONE,
            viewangles: view,
            prone_direction: dir,
            ..PlayerState::ZERO
        }
    }

    fn turn<C: CollisionBackend>(ps: &mut PlayerState, collision: &C, msec: i32, forward: i8) {
        let old_yaw = ps.viewangles[1];
        let cmd = UserCmd {
            forwardmove: forward,
            ..UserCmd::default()
        };
        bo2_prone_view(ps, &cmd, msec, collision, false, old_yaw);
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    /// Looking within 40 of the body, standing still: nothing moves.
    #[test]
    fn bo2_prone_small_look_leaves_the_body() {
        let mut ps = lying(0.0, [0.0, 30.0, 0.0]);
        turn(&mut ps, &Open, 50, 0);
        assert_eq!((ps.prone_direction, ps.viewangles[1]), (0.0, 30.0));
        assert_eq!(ps.delta_angles, [0.0; 3]);
    }

    /// Looking 60 off: the body swings 55 a second toward the view and the
    /// view is held 45 from it, the difference kept in `delta_angles`.
    #[test]
    fn bo2_prone_turn_is_held_to_the_body_swing() {
        let mut ps = lying(0.0, [0.0, 60.0, 0.0]);
        turn(&mut ps, &Open, 50, 0);
        assert!(close(ps.prone_direction, 2.75), "{}", ps.prone_direction);
        assert!(close(ps.viewangles[1], 47.75), "{}", ps.viewangles[1]);
        assert!(close(ps.delta_angles[1], -12.25), "{}", ps.delta_angles[1]);
    }

    /// Moving: the body swings toward the view even a little off, and lands
    /// on it when the swing would pass it.
    #[test]
    fn bo2_prone_moving_swings_the_body_to_the_view() {
        let mut ps = lying(0.0, [0.0, 10.0, 0.0]);
        turn(&mut ps, &Open, 100, 127);
        assert!(close(ps.prone_direction, 5.5), "{}", ps.prone_direction);
        let mut ps = lying(0.0, [0.0, 3.0, 0.0]);
        turn(&mut ps, &Open, 100, 127);
        assert_eq!(ps.prone_direction, 3.0);
    }

    /// Where the body cannot turn, the view is dragged back onto it.
    #[test]
    fn bo2_prone_blocked_body_pulls_the_view_back() {
        let mut ps = lying(90.0, [0.0, 150.0, 0.0]);
        turn(&mut ps, &Stuck, 50, 0);
        assert_eq!(ps.prone_direction, 90.0);
        assert!(close(ps.viewangles[1], 90.0), "{}", ps.viewangles[1]);
        // Sixty 1-degree steps add up a little float error.
        assert!((ps.delta_angles[1] + 60.0).abs() < 1e-3, "{}", ps.delta_angles[1]);
    }

    /// The view's pitch stays within 45 of the torso's.
    #[test]
    fn bo2_prone_pitch_is_held_to_the_torso() {
        let mut ps = lying(0.0, [60.0, 0.0, 0.0]);
        turn(&mut ps, &Open, 50, 0);
        assert_eq!(ps.viewangles[0], 45.0);
        assert_eq!(ps.delta_angles[0], -15.0);
    }

    /// Not lying, or mantling: the view is left alone.
    #[test]
    fn bo2_prone_view_only_lying() {
        let mut ps = lying(0.0, [0.0, 60.0, 0.0]);
        ps.pm_flags = 0;
        turn(&mut ps, &Open, 50, 0);
        assert_eq!(ps.viewangles[1], 60.0);
        ps.pm_flags = pm_flags::PRONE | pm_flags::MANTLE;
        turn(&mut ps, &Open, 50, 0);
        assert_eq!(ps.viewangles[1], 60.0);
    }

    fn pml(ground: Option<[f32; 3]>, walkable: bool, frametime: f32) -> Pml {
        let mut ground_trace = [0; 11];
        if let Some(normal) = ground {
            ground_trace[1] = normal[0].to_bits();
            ground_trace[2] = normal[1].to_bits();
            ground_trace[3] = normal[2].to_bits();
            ground_trace[10] = u32::from(walkable) << 16;
        }
        Pml {
            forward: [0.0; 3],
            right: [0.0; 3],
            up: [0.0; 3],
            frametime,
            msec: (frametime * 1000.0) as i32,
            walking: 0,
            ground_plane: u32::from(ground.is_some()),
            almost_ground_plane: 0,
            ground_trace,
            previous_origin: [0.0; 3],
            previous_velocity: [0.0; 3],
            holdrand: 0,
        }
    }

    const RAMP_NORMAL: [f32; 3] = [-0.196_116_14, 0.0, 0.980_580_7];

    /// On a ramp the slope pitches ease toward it at 70 a second.
    #[test]
    fn bo2_prone_slope_pitch_eases_at_70_a_second() {
        let mut ps = lying(0.0, [0.0, 0.0, 0.0]);
        ps.ground_entity_num = 0;
        let target = pitch_for_yaw_on_normal(0.0, RAMP_NORMAL);
        assert!(target < -10.0, "{target}");
        bo2_prone_pitch(&mut ps, &pml(Some(RAMP_NORMAL), true, 0.05), &Open, false);
        assert!(close(ps.prone_direction_pitch, -3.5), "{}", ps.prone_direction_pitch);
        assert!(close(ps.prone_torso_pitch, -3.5), "{}", ps.prone_torso_pitch);
        assert_eq!(ps.event_sequence, 0);
        for _ in 0..10 {
            bo2_prone_pitch(&mut ps, &pml(Some(RAMP_NORMAL), true, 0.05), &Open, false);
        }
        assert!(close(ps.prone_direction_pitch, target), "{}", ps.prone_direction_pitch);
    }

    /// Lying on ground too steep, or in the air where the body no longer
    /// fits: told to crouch.
    #[test]
    fn bo2_prone_no_room_is_told_to_crouch() {
        let mut ps = lying(0.0, [0.0, 0.0, 0.0]);
        ps.ground_entity_num = 0;
        bo2_prone_pitch(&mut ps, &pml(Some([0.0, 0.0, 1.0]), false, 0.05), &Open, false);
        assert_eq!((ps.event_sequence, ps.events_0), (1, EV_STANCE_FORCE_CROUCH));
        let mut ps = lying(0.0, [0.0, 0.0, 0.0]);
        bo2_prone_pitch(&mut ps, &pml(None, false, 0.05), &Open, false);
        assert_eq!(ps.event_sequence, 0);
        bo2_prone_pitch(&mut ps, &pml(None, false, 0.05), &Stuck, false);
        assert_eq!((ps.event_sequence, ps.events_0), (1, EV_STANCE_FORCE_CROUCH));
    }
}
