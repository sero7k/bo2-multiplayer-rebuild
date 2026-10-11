use math_iw4::{angle_normalize_360, angle_subtract, yaw_vectors_2d};
use playerstate_iw4::{ENTITYNUM_NONE, PM_TYPE_LAST_STAND, PlayerState, pm_flags};
use trace_iw4::Trace;

use crate::{CollisionBackend, GroundTraceInput, StanceSurface, stance_surface_type};

pub const PRONE_CHECK_HEIGHT: f32 = 30.0;

pub const PRONE_FEET_DIST: f32 = 50.0;

const PRONE_TRACE_MASK: u32 = 0x0081_0011;

const PRONE_CAPSULE: f32 = 6.0;

const PRONE_FIRST_LIFT: f32 = 10.0;

const PRONE_YAW_FLIP: f32 = 180.0;

const PRONE_WAIST_ALONG: f32 = 18.0;

const PRONE_SHORT_FEET_FRAC: f32 = 0.699_999_988_079_071;

const PRONE_SHORT_FEET_BUMP: f32 = 22.0;

const PRONE_MIN_FIRST_PAD: f32 = 2.0;

const PRONE_GROUNDED_WAIST_SCALE: f32 = 2.5;

const PRONE_WAIST_FEET_REJECT: f32 = -0.75;

const PRONE_POINT_LIFT: f32 = 5.0;

const PRONE_PITCH_DELTA_MIN: f32 = -50.0;
const PRONE_PITCH_DELTA_MAX: f32 = 70.0;

/// bo2zm: Black Ops II puts the waist 12 units along the body, not 18.
const BO2_PRONE_WAIST_ALONG: f32 = 12.0;

/// bo2zm: a blocked far-feet trace is retried this much higher (BO2 keeps
/// 18 here when its waist moves to 12).
const PRONE_FAR_RETRY_LIFT: f32 = 18.0;

/// bo2zm: Black Ops II's `lastStandDistScale`: in last stand the body lies
/// in front of the player, three quarters as long (half in Zombies).
const BO2_LAST_STAND_SCALE: f32 = -0.75;
const BO2_LAST_STAND_SCALE_ZOMBIES: f32 = -0.5;

/// bo2zm: in Zombies last stand the body may bend 65 degrees, not 50.
const BO2_PRONE_PITCH_DELTA_MIN_ZOMBIES: f32 = -65.0;

/// bo2zm: a waist-to-feet line blocked by the ground is forgiven when the
/// legs run downhill by less than this (BO2's `enable_new_prone_check` 0).
const BO2_PRONE_DOWNHILL_FORGIVE: f32 = -20.0;

/// bo2zm: whose prone check runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProneRules {
    /// IW4's (MW2 settings): in last stand the body is not turned around.
    Iw4 { not_last_stand: bool },

    /// Black Ops II's.
    Bo2 {
        /// pm_type 7: the body lies in front, its pitches are flipped.
        last_stand: bool,

        /// Zombies' last-stand numbers.
        zombies: bool,

        /// In the air diving: the body pitches read 0.
        diving: bool,
    },
}

impl ProneRules {
    /// The rules for this player: Black Ops II's when `bo2`.
    #[must_use]
    pub fn for_player(ps: &PlayerState, bo2: bool, zombies: bool) -> Self {
        if bo2 {
            Self::Bo2 {
                last_stand: ps.pm_type == PM_TYPE_LAST_STAND,
                zombies,
                diving: (ps.pm_flags & crate::dive::PMF_DIVE) != 0,
            }
        } else {
            Self::Iw4 {
                not_last_stand: stance_surface_type(ps) != StanceSurface::LastStand,
            }
        }
    }
}

fn prone_special_air_ok(ps: &PlayerState) -> bool {
    let flags = ps.pm_flags;
    if (flags & 0x800) != 0 {
        return (flags & 0xffff_ff00) != 0;
    }
    (flags & 0x40_0000) != 0 && ps.pm_time == 0
}

#[must_use]
pub fn player_prone_allowed<C: CollisionBackend>(
    ps: &PlayerState,
    collision: &C,
    f_size: f32,
    weapon_blocks_prone: bool,
    rules: ProneRules,
) -> bool {
    if weapon_blocks_prone {
        return false;
    }
    if (ps.pm_flags & pm_flags::PRONE) != 0 {
        return true;
    }
    if ps.ground_entity_num == ENTITYNUM_NONE && !prone_special_air_ok(ps) {
        return false;
    }
    check_prone(
        collision,
        ps.origin,
        f_size,
        PRONE_CHECK_HEIGHT,
        ps.viewangles[1],
        false,
        ps.ground_entity_num != ENTITYNUM_NONE,
        true,
        rules,
        PRONE_FEET_DIST,
        None,
        None,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "one call boundary for the prone check"
)]
#[must_use]
pub fn check_prone<C: CollisionBackend>(
    collision: &C,
    origin: [f32; 3],
    f_size: f32,
    f_height: f32,
    f_yaw: f32,
    already_prone: bool,
    on_ground: bool,
    ground_walkable: bool,
    rules: ProneRules,
    prone_feet_dist: f32,
    mut torso_pitch: Option<&mut f32>,
    mut waist_pitch: Option<&mut f32>,
) -> bool {
    if !already_prone {
        let lifted = [origin[0], origin[1], origin[2] + PRONE_FIRST_LIFT];
        let hull = collision.trace(GroundTraceInput {
            start: origin,
            end: lifted,
            mins: [-f_size, -f_size, 0.0],
            maxs: [f_size, f_size, f_height],
            tracemask: PRONE_TRACE_MASK,
        });
        if hull.allsolid != 0 {
            return false;
        }
    }
    if on_ground && !ground_walkable {
        return false;
    }

    let (yaw, waist_along, prone_feet_dist, pitch_min) = match rules {
        ProneRules::Iw4 { not_last_stand } => (
            if not_last_stand {
                f_yaw - PRONE_YAW_FLIP
            } else {
                f_yaw
            },
            PRONE_WAIST_ALONG,
            prone_feet_dist,
            PRONE_PITCH_DELTA_MIN,
        ),
        // bo2zm: always turned around; last stand flips the body to the
        // front by a negative length instead.
        ProneRules::Bo2 {
            last_stand,
            zombies,
            ..
        } => {
            let scale = match (last_stand, zombies) {
                (false, _) => 1.0,
                (true, false) => BO2_LAST_STAND_SCALE,
                (true, true) => BO2_LAST_STAND_SCALE_ZOMBIES,
            };
            (
                f_yaw - PRONE_YAW_FLIP,
                scale * BO2_PRONE_WAIST_ALONG,
                scale * prone_feet_dist,
                if last_stand && zombies {
                    BO2_PRONE_PITCH_DELTA_MIN_ZOMBIES
                } else {
                    PRONE_PITCH_DELTA_MIN
                },
            )
        }
    };
    let bo2 = matches!(rules, ProneRules::Bo2 { .. });
    let not_last_stand = match rules {
        ProneRules::Iw4 { not_last_stand } => not_last_stand,
        ProneRules::Bo2 { .. } => true,
    };
    let (fwd_xy, _) = yaw_vectors_2d(yaw);
    let mut forward = [fwd_xy[0], fwd_xy[1], 0.0];
    let f_trace_height = f_height - PRONE_CAPSULE;
    let mut start = [origin[0], origin[1], origin[2] + f_trace_height];
    let along = prone_feet_dist - PRONE_CAPSULE;
    let mut end = [
        start[0] + along * forward[0],
        start[1] + along * forward[1],
        start[2] + along * forward[2],
    ];
    let cap_mins = [-PRONE_CAPSULE, -PRONE_CAPSULE, -PRONE_CAPSULE];
    let cap_maxs = [PRONE_CAPSULE, PRONE_CAPSULE, PRONE_CAPSULE];
    let mut trace = prone_trace(collision, start, end, cap_mins, cap_maxs);
    let mut first_hit = false;
    let first_dist = if trace.fraction >= 1.0 {
        prone_feet_dist
    } else {
        if !on_ground {
            return false;
        }
        first_hit = true;
        let dist = along * trace.fraction + PRONE_CAPSULE;
        if dist < f_size + PRONE_MIN_FIRST_PAD {
            return false;
        }
        let short = f_trace_height * PRONE_SHORT_FEET_FRAC + waist_along;
        if dist < short {
            first_hit = false;
            end[2] += PRONE_SHORT_FEET_BUMP;
            let delta = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
            let pitch_len = vec3_normalize_to(delta, &mut forward);
            trace = prone_trace(collision, start, end, cap_mins, cap_maxs);
            if trace.fraction >= 1.0 {
                prone_feet_dist
            } else {
                first_hit = true;
                let retry = trace.fraction * pitch_len + PRONE_CAPSULE;
                if retry < short {
                    return false;
                }
                retry
            }
        } else {
            dist
        }
    };

    let mut feet = vec3_lerp(start, end, trace.fraction);
    let waist_scale = if on_ground {
        PRONE_GROUNDED_WAIST_SCALE
    } else {
        1.0
    };
    start = [
        origin[0] + waist_along * forward[0],
        origin[1] + waist_along * forward[1],
        origin[2] + f_trace_height + waist_along * forward[2],
    ];
    let down = waist_scale * f_size + f_trace_height - PRONE_CAPSULE;
    end = [start[0], start[1], start[2] - down];
    trace = prone_trace(collision, start, end, cap_mins, cap_maxs);
    if trace.fraction == 1.0 {
        return prone_fail(
            on_ground,
            torso_pitch.as_deref_mut(),
            waist_pitch.as_deref_mut(),
        );
    }
    if trace.walkable == 0 {
        return false;
    }
    let waist_dist = down * trace.fraction + PRONE_CAPSULE;
    let mut waist = vec3_lerp(start, end, trace.fraction);
    waist[2] -= PRONE_CAPSULE;
    if first_hit {
        if first_dist - waist_dist < waist_dist * PRONE_WAIST_FEET_REJECT {
            return prone_fail(
                on_ground,
                torso_pitch.as_deref_mut(),
                waist_pitch.as_deref_mut(),
            );
        }
        let mut delta = [feet[0] - waist[0], feet[1] - waist[1], feet[2] - waist[2]];
        delta[0] += PRONE_CAPSULE * forward[0];
        delta[1] += PRONE_CAPSULE * forward[1];
        delta[2] += PRONE_CAPSULE * forward[2] + PRONE_CAPSULE;
        vec3_normalize(&mut delta);
        let mad_scale = along - waist_along;
        end = [
            start[0] + mad_scale * delta[0],
            start[1] + mad_scale * delta[1],
            start[2] + mad_scale * delta[2],
        ];
        end[0] = (along * forward[0] + origin[0] + end[0]) * 0.5;
        end[1] = (along * forward[1] + origin[1] + end[1]) * 0.5;
        trace = prone_trace(collision, start, end, cap_mins, cap_maxs);
        if trace.fraction < 1.0 {
            start = vec3_lerp(start, end, trace.fraction);
            start[2] += PRONE_FAR_RETRY_LIFT;
            end[2] += PRONE_FAR_RETRY_LIFT;
            trace = prone_trace(collision, start, end, cap_mins, cap_maxs);
            if trace.fraction < 1.0 {
                return prone_fail(
                    on_ground,
                    torso_pitch.as_deref_mut(),
                    waist_pitch.as_deref_mut(),
                );
            }
        }
        feet = vec3_lerp(start, end, trace.fraction);
    }

    start = feet;
    end = [
        feet[0],
        feet[1],
        feet[2] - ((feet[2] - waist[2]) + (feet[2] - waist[2]) + f_size),
    ];
    trace = prone_trace(collision, start, end, cap_mins, cap_maxs);
    if trace.fraction == 1.0 {
        return prone_fail(
            on_ground,
            torso_pitch.as_deref_mut(),
            waist_pitch.as_deref_mut(),
        );
    }
    if trace.walkable == 0 {
        return false;
    }
    feet = vec3_lerp(start, end, trace.fraction);
    feet[2] -= PRONE_CAPSULE;
    let torso = origin;
    let torso_delta = if not_last_stand {
        [
            torso[0] - waist[0],
            torso[1] - waist[1],
            torso[2] - waist[2],
        ]
    } else {
        [
            waist[0] - torso[0],
            waist[1] - torso[1],
            waist[2] - torso[2],
        ]
    };
    // bo2zm: Black Ops II keeps the pitches in -180..180.
    let normalize = |pitch: f32| {
        if bo2 {
            angle_subtract(pitch, 0.0)
        } else {
            angle_normalize_360(pitch)
        }
    };
    let f_torso = normalize(vec_to_pitch(torso_delta));
    let waist_delta = if not_last_stand {
        [waist[0] - feet[0], waist[1] - feet[1], waist[2] - feet[2]]
    } else {
        [feet[0] - waist[0], feet[1] - waist[1], feet[2] - waist[2]]
    };
    let mut f_waist = normalize(vec_to_pitch(waist_delta));
    let pitch_diff = angle_subtract(f_torso, f_waist);
    let mut success = true;
    if !(pitch_min..=PRONE_PITCH_DELTA_MAX).contains(&pitch_diff) {
        success = false;
    }
    let zero = [0.0, 0.0, 0.0];
    trace = prone_trace(
        collision,
        [torso[0], torso[1], torso[2] + PRONE_POINT_LIFT],
        [waist[0], waist[1], waist[2] + PRONE_POINT_LIFT],
        zero,
        zero,
    );
    if trace.fraction < 1.0 {
        success = false;
    }
    trace = prone_trace(
        collision,
        [waist[0], waist[1], waist[2] + PRONE_POINT_LIFT],
        [feet[0], feet[1], feet[2] + PRONE_POINT_LIFT],
        zero,
        zero,
    );
    if trace.fraction < 1.0 {
        if bo2 {
            // bo2zm: legs a little downhill may touch the ground; the
            // waist reads flat either way.
            if !(BO2_PRONE_DOWNHILL_FORGIVE < f_waist && f_waist < 0.0) {
                success = false;
            }
            f_waist = 0.0;
        } else {
            success = false;
        }
    }
    let (mut out_torso, mut out_waist) = (f_torso, f_waist);
    if let ProneRules::Bo2 {
        last_stand, diving, ..
    } = rules
    {
        if last_stand {
            out_torso = -out_torso;
            out_waist = -out_waist;
        }
        if diving {
            out_torso = 0.0;
            out_waist = 0.0;
        }
    }
    if let Some(slot) = torso_pitch.as_deref_mut() {
        *slot = out_torso;
    }
    if let Some(slot) = waist_pitch.as_deref_mut() {
        *slot = out_waist;
    }
    if success {
        return true;
    }
    prone_fail(on_ground, torso_pitch, waist_pitch)
}

fn prone_fail(
    on_ground: bool,
    torso_pitch: Option<&mut f32>,
    waist_pitch: Option<&mut f32>,
) -> bool {
    if on_ground {
        return false;
    }
    if let Some(slot) = torso_pitch {
        *slot = 0.0;
    }
    if let Some(slot) = waist_pitch {
        *slot = 0.0;
    }
    true
}

fn prone_trace<C: CollisionBackend>(
    collision: &C,
    start: [f32; 3],
    end: [f32; 3],
    mins: [f32; 3],
    maxs: [f32; 3],
) -> Trace {
    collision.trace(GroundTraceInput {
        start,
        end,
        mins,
        maxs,
        tracemask: PRONE_TRACE_MASK,
    })
}

fn vec3_lerp(start: [f32; 3], end: [f32; 3], frac: f32) -> [f32; 3] {
    [
        start[0] + frac * (end[0] - start[0]),
        start[1] + frac * (end[1] - start[1]),
        start[2] + frac * (end[2] - start[2]),
    ]
}

fn vec3_normalize(v: &mut [f32; 3]) -> f32 {
    let len = libm::sqrtf(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    let div = if len <= 0.0 { 1.0 } else { len };
    let s = 1.0 / div;
    v[0] *= s;
    v[1] *= s;
    v[2] *= s;
    len
}

fn vec3_normalize_to(src: [f32; 3], dst: &mut [f32; 3]) -> f32 {
    *dst = src;
    vec3_normalize(dst)
}

fn vec_to_pitch(vec: [f32; 3]) -> f32 {
    if vec[0] == 0.0 && vec[1] == 0.0 {
        return if -vec[2] < 0.0 { 270.0 } else { 90.0 };
    }
    let xy = libm::sqrtf(vec[0] * vec[0] + vec[1] * vec[1]);
    let pitch = libm::atan2f(vec[2], xy) * (-180.0 / core::f32::consts::PI);
    if pitch < 0.0 { pitch + 360.0 } else { pitch }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::cell::RefCell;
    use std::vec::Vec;

    use super::*;

    /// Ground tilted to z = slope * x. With `block_legs` something sits on
    /// the line from the waist to the feet (only the thin line traces that
    /// start behind the player stop on it).
    struct Slope {
        slope: f32,
        block_legs: bool,
        traces: RefCell<Vec<GroundTraceInput>>,
    }

    impl Slope {
        fn new(slope: f32, block_legs: bool) -> Self {
            Self {
                slope,
                block_legs,
                traces: RefCell::new(Vec::new()),
            }
        }
    }

    impl CollisionBackend for Slope {
        fn trace(&self, input: GroundTraceInput) -> Trace {
            self.traces.borrow_mut().push(input);
            let mut hit = Trace {
                fraction: 1.0,
                endpos: input.end,
                ..Trace::default()
            };
            if input.mins == [0.0; 3] && input.maxs == [0.0; 3] {
                if self.block_legs && input.start[0] < -1.0 {
                    hit.fraction = 0.5;
                }
                return hit;
            }
            // Only straight-down traces meet the ground in this world.
            if input.start[0] != input.end[0] || input.start[1] != input.end[1] {
                return hit;
            }
            let ground = self.slope * input.start[0];
            let start = input.start[2] + input.mins[2];
            let end = input.end[2] + input.mins[2];
            if start >= ground && end < ground {
                hit.fraction = (start - ground) / (start - end);
                hit.normal = [0.0, 0.0, 1.0];
                hit.walkable = 1;
            }
            hit
        }
    }

    const BO2: ProneRules = ProneRules::Bo2 {
        last_stand: false,
        zombies: false,
        diving: false,
    };

    const MW2: ProneRules = ProneRules::Iw4 {
        not_last_stand: true,
    };

    /// Facing +x on the given ground: (allowed, torso pitch, waist pitch).
    fn lie_down(world: &Slope, rules: ProneRules) -> (bool, f32, f32) {
        let (mut torso, mut waist) = (f32::NAN, f32::NAN);
        let ok = check_prone(
            world,
            [0.0, 0.0, 0.0],
            15.0,
            PRONE_CHECK_HEIGHT,
            0.0,
            false,
            true,
            true,
            rules,
            PRONE_FEET_DIST,
            Some(&mut torso),
            Some(&mut waist),
        );
        (ok, torso, waist)
    }

    /// Where the waist is looked for: the first box trace straight down.
    fn waist_x(world: &Slope) -> f32 {
        let traces = world.traces.borrow();
        let down = traces
            .iter()
            .find(|t| t.mins != [0.0; 3] && t.end[2] < t.start[2])
            .expect("a waist trace");
        down.start[0]
    }

    /// Where the feet are looked for: the end of the box trace along the
    /// body.
    fn feet_x(world: &Slope) -> f32 {
        let traces = world.traces.borrow();
        let along = traces
            .iter()
            .find(|t| t.mins != [0.0; 3] && t.end[2] == t.start[2])
            .expect("a trace along the body");
        along.end[0]
    }

    /// BO2's BG_CheckProne: the waist is 12 units behind the player (IW4:
    /// 18), and the body pitches stay in -180..180.
    #[test]
    fn bo2_waist_sits_twelve_back() {
        let world = Slope::new(0.2, false);
        let (ok, torso, waist) = lie_down(&world, MW2);
        assert!(ok);
        assert!((waist_x(&world) + 18.0).abs() < 1e-4);
        assert!((torso - (360.0 - 11.309_932)).abs() < 1e-3, "{torso}");
        assert!((waist - (360.0 - 11.309_932)).abs() < 1e-3, "{waist}");

        let world = Slope::new(0.2, false);
        let (ok, torso, waist) = lie_down(&world, BO2);
        assert!(ok);
        assert!((waist_x(&world) + 12.0).abs() < 1e-4);
        assert!((torso + 11.309_932).abs() < 1e-3, "{torso}");
        assert!((waist + 11.309_932).abs() < 1e-3, "{waist}");
    }

    /// BO2 (enable_new_prone_check 0): legs running a little downhill may
    /// touch the ground and the waist reads flat; uphill legs still fail.
    #[test]
    fn bo2_forgives_legs_a_little_downhill() {
        assert!(!lie_down(&Slope::new(0.2, true), MW2).0);

        let (ok, torso, waist) = lie_down(&Slope::new(0.2, true), BO2);
        assert!(ok);
        assert!((torso + 11.309_932).abs() < 1e-3, "{torso}");
        assert_eq!(waist, 0.0);

        assert!(!lie_down(&Slope::new(-0.2, true), BO2).0);
    }

    /// BO2's last stand lays the body in front of the player, three
    /// quarters as long (half in Zombies), with its pitches flipped back.
    #[test]
    fn bo2_last_stand_lies_in_front() {
        for (zombies, scale) in [(false, -0.75_f32), (true, -0.5)] {
            let world = Slope::new(0.2, false);
            let rules = ProneRules::Bo2 {
                last_stand: true,
                zombies,
                diving: false,
            };
            let (ok, torso, waist) = lie_down(&world, rules);
            assert!(ok);
            assert!((waist_x(&world) + scale * 12.0).abs() < 1e-4);
            assert!((feet_x(&world) + (scale * PRONE_FEET_DIST - 6.0)).abs() < 1e-4);
            assert!((torso + 11.309_932).abs() < 1e-3, "{torso}");
            assert!((waist + 11.309_932).abs() < 1e-3, "{waist}");
        }
    }

    /// BO2: in the air diving, the body pitches read 0.
    #[test]
    fn bo2_dive_reads_flat() {
        let rules = ProneRules::Bo2 {
            last_stand: false,
            zombies: false,
            diving: true,
        };
        assert_eq!(lie_down(&Slope::new(0.2, false), rules), (true, 0.0, 0.0));
    }
}
