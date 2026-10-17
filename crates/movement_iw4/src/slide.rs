use playerstate_iw4::{ENTITYNUM_NONE, PlayerState, pm_flags};

use crate::{CollisionBackend, GroundTraceInput, Pml, jump};

const OVERCLIP: f32 = 1.001;

const PROJECT_NORMAL_Z_EPSILON: f32 = 0.001;

const MAX_CLIP_PLANES: usize = 8;

const STEP_SIZE: f32 = 18.0;

/// bo2zm: the body width Black Ops II checks a prone player still fits.
const PRONE_VERIFY_SIZE: f32 = 15.0;

const PRONE_STEP_SIZE: f32 = 10.0;

const STEP_UP_EXTRA: f32 = 1.0;


const STEP_GROUND_NORMAL_Z: f32 = 0.9;

const WALKING_STEP_DOWN_EXTRA: f32 = 9.0;

const STEP_PROGRESS_EPSILON: f32 = 0.001;

/// bo2zm: Black Ops II's com_maxclients (18): entity numbers below it are
/// players (ours too: corpses start at 18).
const MAX_CLIENTS: u16 = playerstate_iw4::PLAYER_CORPSE_ENTITY_BASE as u16;

/// bo2zm: Black Ops II's stair step (PM_StepSlideMove's end).
const STEP_SLOW_MIN_RISE: f32 = 0.5;
const STEP_ROUND_BIAS: f64 = 9.313_225_746_154_785e-10;
const STEP_SLOW_RANGE: f32 = 0.8;
const STEP_SLOW_FLOOR: f32 = f32::from_bits(0x3e4c_cccc);
const STEP_BOB_PER_UNIT: f32 = 1.25;
const STEP_BOB_BASE: f32 = 7.0;

const DUPLICATE_PLANE_DOT: f64 = 0.999_000_012_874_603_3;

const PLANE_INTO: f32 = 0.1;

#[allow(clippy::too_many_arguments)]
pub fn slide_move<C: CollisionBackend>(
    ps: &mut PlayerState,
    pml: &Pml,
    collision: &C,
    mins: [f32; 3],
    maxs: [f32; 3],
    tracemask: u32,
    gravity: Option<f32>,
) -> bool {
    let mut end_velocity = ps.velocity;
    let mut primal_velocity = ps.velocity;
    if let Some(g) = gravity {
        end_velocity[2] -= g * pml.frametime;
        primal_velocity[2] = end_velocity[2];
        ps.velocity[2] = (ps.velocity[2] + end_velocity[2]) * 0.5;
        if pml.ground_plane != 0 {
            let normal = [
                f32::from_bits(pml.ground_trace[1]),
                f32::from_bits(pml.ground_trace[2]),
                f32::from_bits(pml.ground_trace[3]),
            ];
            let velocity = ps.velocity;
            clip_velocity(&velocity, &normal, &mut ps.velocity);
        }
    }

    let mut planes = [[0.0_f32; 3]; MAX_CLIP_PLANES];
    let mut plane_count = 0usize;
    if pml.ground_plane != 0 {
        planes[0] = [
            f32::from_bits(pml.ground_trace[1]),
            f32::from_bits(pml.ground_trace[2]),
            f32::from_bits(pml.ground_trace[3]),
        ];
        plane_count = 1;
    }
    let mut dir = ps.velocity;
    let _ = normalize(&mut dir);
    if plane_count < MAX_CLIP_PLANES {
        planes[plane_count] = dir;
        plane_count += 1;
    }

    let mut time_left = pml.frametime;
    let mut bumped = false;
    for _ in 0..4 {
        if time_left <= 0.0 {
            break;
        }
        let end = [
            ps.origin[0] + ps.velocity[0] * time_left,
            ps.origin[1] + ps.velocity[1] * time_left,
            ps.origin[2] + ps.velocity[2] * time_left,
        ];
        let trace = collision.trace(GroundTraceInput {
            start: ps.origin,
            end,
            mins,
            maxs,
            tracemask,
        });
        if trace.allsolid != 0 {
            ps.velocity[2] = 0.0;
            return true;
        }
        if trace.fraction > 0.0 {
            if trace.fraction >= 1.0 {
                ps.origin = end;
            } else {
                ps.origin = trace.endpos;
            }
        }
        if trace.fraction >= 1.0 {
            break;
        }
        bumped = true;
        time_left *= 1.0 - trace.fraction;
        if plane_count >= MAX_CLIP_PLANES {
            ps.velocity = [0.0; 3];
            return true;
        }

        let mut duplicate = false;
        for plane in planes.iter().take(plane_count) {
            if f64::from(dot(&trace.normal, plane)) > DUPLICATE_PLANE_DOT {
                duplicate = true;
                break;
            }
        }
        if duplicate {
            let velocity = ps.velocity;
            clip_velocity(&velocity, &trace.normal, &mut ps.velocity);
            ps.velocity[0] += trace.normal[0];
            ps.velocity[1] += trace.normal[1];
            ps.velocity[2] += trace.normal[2];
            continue;
        }
        planes[plane_count] = trace.normal;
        plane_count += 1;

        let mut permutation = [0usize; MAX_CLIP_PLANES];
        let into =
            permute_restrictive_clip_planes(&ps.velocity, &planes[..plane_count], &mut permutation);
        if into < PLANE_INTO {
            let first = permutation[0];
            let mut clip_velocity_out = [0.0_f32; 3];
            let mut end_clip = [0.0_f32; 3];
            clip_velocity(&ps.velocity, &planes[first], &mut clip_velocity_out);
            clip_velocity(&end_velocity, &planes[first], &mut end_clip);
            for j in 1..plane_count {
                let pj = permutation[j];
                if dot(&clip_velocity_out, &planes[pj]) >= PLANE_INTO {
                    continue;
                }
                let current = clip_velocity_out;
                clip_velocity(&current, &planes[pj], &mut clip_velocity_out);
                let current_end = end_clip;
                clip_velocity(&current_end, &planes[pj], &mut end_clip);
                if dot(&clip_velocity_out, &planes[first]) >= 0.0 {
                    continue;
                }
                let mut crease = cross(&planes[first], &planes[pj]);
                let _ = normalize(&mut crease);
                let along = dot(&ps.velocity, &crease);
                clip_velocity_out = [crease[0] * along, crease[1] * along, crease[2] * along];
                let along_end = dot(&end_velocity, &crease);
                end_clip = [
                    crease[0] * along_end,
                    crease[1] * along_end,
                    crease[2] * along_end,
                ];
                for k in 1..plane_count {
                    if k == j {
                        continue;
                    }
                    let pk = permutation[k];
                    if dot(&clip_velocity_out, &planes[pk]) < PLANE_INTO {
                        ps.velocity = [0.0; 3];
                        return true;
                    }
                }
            }
            ps.velocity = clip_velocity_out;
            end_velocity = end_clip;
        }
    }

    if gravity.is_some() {
        ps.velocity = end_velocity;
    }
    // While pm_time runs (just after a landing) the move keeps the speed you
    // came in with: a wall moves you, it does not slow you.
    if ps.pm_time != 0 {
        ps.velocity = primal_velocity;
    }
    bumped
}

/// Returns how far a stair step moved you up or down (0 when there was
/// none): the camera eases over it.
#[allow(clippy::too_many_arguments)]
pub fn step_slide_move<C: CollisionBackend>(
    ps: &mut PlayerState,
    pml: &Pml,
    collision: &C,
    mins: [f32; 3],
    maxs: [f32; 3],
    tracemask: u32,
    gravity: Option<f32>,
    zombies: bool,
) -> f32 {
    let had_ground = if (ps.pm_flags & pm_flags::LADDER) != 0 {
        jump::clear_state(ps);
        false
    } else if pml.ground_plane != 0 {
        true
    } else {
        if (ps.pm_flags & pm_flags::JUMPING) != 0 && ps.pm_time != 0 {
            jump::clear_state(ps);
        }
        false
    };

    let start_origin = ps.origin;
    let start_velocity = ps.velocity;

    let bumped = slide_move(ps, pml, collision, mins, maxs, tracemask, gravity);
    let down_origin = ps.origin;
    let down_velocity = ps.velocity;

    // In last stand a move that hit nothing never steps.
    if !bumped
        && ps.pm_type == playerstate_iw4::PM_TYPE_LAST_STAND
        && (ps.pm_flags & pm_flags::LAST_STAND) != 0
    {
        return 0.0;
    }

    // A jump stepping up near its top is capped.
    let mut jump_step = false;
    let mut step_size = if (ps.pm_flags & pm_flags::PRONE) != 0 {
        PRONE_STEP_SIZE
    } else {
        STEP_SIZE
    };

    if ps.ground_entity_num == ENTITYNUM_NONE {
        if (ps.pm_flags & pm_flags::JUMPING) != 0 && ps.pm_time != 0 {
            jump::clear_state(ps);
        }
        let jumping = (ps.pm_flags & pm_flags::JUMPING) != 0;
        let ladder_up = (ps.pm_flags & pm_flags::LADDER) != 0 && ps.velocity[2] > 0.0;
        if bumped && jumping {
            match jump::get_step_height(ps, start_origin) {
                Some(height) if height < 1.0 => return 0.0,
                Some(height) => {
                    step_size = height;
                    jump_step = true;
                }
                None if !ladder_up => return 0.0,
                None => {}
            }
        } else if !ladder_up {
            return 0.0;
        }
    }

    let ground_normal_z = f32::from_bits(pml.ground_trace[3]);
    let sloped_ground = pml.ground_plane != 0 && ground_normal_z < STEP_GROUND_NORMAL_Z;
    let mut step_height = 0.0;

    if bumped || sloped_ground {
        let up = collision.trace(GroundTraceInput {
            start: start_origin,
            end: [
                start_origin[0],
                start_origin[1],
                start_origin[2] + step_size + STEP_UP_EXTRA,
            ],
            mins,
            maxs,
            tracemask,
        });

        let step_amount = (step_size + STEP_UP_EXTRA) * up.fraction - STEP_UP_EXTRA;
        if step_amount >= 1.0 {
            step_height = step_amount;
            ps.origin = [
                start_origin[0],
                start_origin[1],
                start_origin[2] + step_amount,
            ];
            ps.velocity = start_velocity;
            let _ = slide_move(ps, pml, collision, mins, maxs, tracemask, gravity);
        }
    }

    if had_ground || step_height != 0.0 {
        let mut descent = step_height;
        if had_ground {
            descent += WALKING_STEP_DOWN_EXTRA;
        }
        let down_target = [ps.origin[0], ps.origin[1], ps.origin[2] - descent];
        let down = collision.trace(GroundTraceInput {
            start: ps.origin,
            end: down_target,
            mins,
            maxs,
            tracemask,
        });
        // The whole step is taken back when the step down would put you on
        // another player.
        if trace_iw4::trace_get_entity_hit_id(down.hit_type, down.hit_id) < MAX_CLIENTS {
            ps.origin = down_origin;
            ps.velocity = down_velocity;
            return 0.0;
        }
        if down.fraction >= 1.0 {
            if step_height != 0.0 {
                ps.origin[2] -= step_height;
            }
        } else if down.walkable != 0 {
            // Only walkable ground is stepped down onto; a steeper face never
            // holds you.
            ps.origin = down.endpos;

            project_velocity(&mut ps.velocity, &down.normal);
        } else {
            ps.origin = down_origin;
            ps.velocity = down_velocity;
            return 0.0;
        }
    }

    let gained = start_velocity[0] * (ps.origin[0] - start_origin[0])
        + start_velocity[1] * (ps.origin[1] - start_origin[1]);
    let flat = start_velocity[0] * (down_origin[0] - start_origin[0])
        + start_velocity[1] * (down_origin[1] - start_origin[1]);
    // A jump's step that ends at or past the top of the jump is taken back.
    if gained <= flat + STEP_PROGRESS_EPSILON || (jump_step && jump::above_max(ps)) {
        ps.origin = down_origin;
        ps.velocity = down_velocity;

        if had_ground {
            let down_target = [
                ps.origin[0],
                ps.origin[1],
                ps.origin[2] - WALKING_STEP_DOWN_EXTRA,
            ];
            let down = collision.trace(GroundTraceInput {
                start: ps.origin,
                end: down_target,
                mins,
                maxs,
                tracemask,
            });
            if down.fraction < 1.0 {
                ps.origin = down.endpos;
                // The speed is kept along the slope.
                project_velocity(&mut ps.velocity, &down.normal);
            }
        }
    }

    if jump_step {
        jump::clamp_velocity(ps, down_origin);
    }
    if had_ground && ps.pm_type < playerstate_iw4::PM_TYPE_DEAD {
        // A prone player the move left with no room to lie goes back to
        // where they were, and the step counts for nothing.
        if !verify_prone_position(ps, collision, start_origin, start_velocity, zombies) {
            return 0.0;
        }
        return stair_step(ps, pml, start_origin, down_origin, step_size);
    }
    0.0
}

/// bo2zm: Black Ops II after a move on the ground: a prone player must still
/// have room to lie where they ended up. If not, they go back to where they
/// started; if that spot is no better, the move stands. False when the move
/// was taken back.
fn verify_prone_position<C: CollisionBackend>(
    ps: &mut PlayerState,
    collision: &C,
    old_origin: [f32; 3],
    old_velocity: [f32; 3],
    zombies: bool,
) -> bool {
    if (ps.pm_flags & pm_flags::PRONE) == 0 {
        return true;
    }
    if prone_fits(ps, collision, zombies) {
        return true;
    }
    let (new_origin, new_velocity) = (ps.origin, ps.velocity);
    ps.origin = old_origin;
    ps.velocity = old_velocity;
    if prone_fits(ps, collision, zombies) {
        return false;
    }
    ps.origin = new_origin;
    ps.velocity = new_velocity;
    true
}

fn prone_fits<C: CollisionBackend>(ps: &mut PlayerState, collision: &C, zombies: bool) -> bool {
    let (mut torso, mut waist) = (ps.f_torso_pitch, ps.f_waist_pitch);
    let fits = crate::check_prone(
        collision,
        ps.origin,
        PRONE_VERIFY_SIZE,
        crate::PRONE_CHECK_HEIGHT,
        ps.prone_direction,
        true,
        true,
        true,
        crate::ProneRules::for_player(ps, zombies),
        crate::PRONE_FEET_DIST,
        Some(&mut torso),
        Some(&mut waist),
    );
    ps.f_torso_pitch = torso;
    ps.f_waist_pitch = waist;
    fits
}

/// bo2zm: Black Ops II on a stair: a step up or down on the ground costs
/// speed by its height (a full step leaves 0.2), and one of more than 3 units
/// moves the walk rhythm on, which can sound a footstep. Returns the step's
/// height for the camera (0 when it does not count).
fn stair_step(
    ps: &mut PlayerState,
    pml: &Pml,
    start_origin: [f32; 3],
    down_origin: [f32; 3],
    step_size: f32,
) -> f32 {
    if ps.pm_type >= playerstate_iw4::PM_TYPE_DEAD {
        return 0.0;
    }
    let rise = ps.origin[2] - down_origin[2];
    if libm::fabsf(rise) <= STEP_SLOW_MIN_RISE {
        return 0.0;
    }
    let delta = libm::rint(rise as f64 + STEP_ROUND_BIAS) as i32;
    if delta == 0 {
        return 0.0;
    }
    let scale = (1.0 - libm::fabsf(ps.origin[2] - start_origin[2]) / step_size) * STEP_SLOW_RANGE
        + STEP_SLOW_FLOOR;
    for v in &mut ps.velocity {
        *v *= scale;
    }
    let delta = delta.abs();
    if delta > 3 && ps.ground_entity_num != ENTITYNUM_NONE {
        let beats = (delta / 2).min(4);
        let old = ps.bob_cycle;
        ps.bob_cycle =
            ((beats as f32 * STEP_BOB_PER_UNIT + STEP_BOB_BASE + old as f32) as i32) & 0xff;
        let footsteps = crate::should_make_footsteps(ps);
        crate::footstep_event(ps, old as u8, ps.bob_cycle as u8, pml.ground_trace[4], footsteps);
    }
    rise
}

pub(crate) fn project_velocity(velocity: &mut [f32; 3], normal: &[f32; 3]) {
    let length_sq_2d = velocity[0] * velocity[0] + velocity[1] * velocity[1];
    if libm::fabsf(normal[2]) < PROJECT_NORMAL_Z_EPSILON || length_sq_2d == 0.0 {
        return;
    }
    let new_z = -(normal[1] * velocity[1] + normal[0] * velocity[0]) / normal[2];
    let original_length_sq = velocity[2] * velocity[2] + length_sq_2d;
    let adjusted_length_sq = new_z * new_z + length_sq_2d;
    let length_scale = libm::sqrtf(original_length_sq / adjusted_length_sq);
    if length_scale >= 1.0 && new_z >= 0.0 && velocity[2] <= 0.0 {
        return;
    }
    velocity[0] *= length_scale;
    velocity[1] *= length_scale;
    velocity[2] = new_z * length_scale;
}

fn permute_restrictive_clip_planes(
    velocity: &[f32; 3],
    planes: &[[f32; 3]],
    permutation: &mut [usize],
) -> f32 {
    let mut parallel = [0.0_f32; MAX_CLIP_PLANES];
    for (plane_index, plane) in planes.iter().enumerate() {
        parallel[plane_index] = dot(velocity, plane);
        let mut permuted_index = plane_index;
        while permuted_index > 0
            && parallel[plane_index] <= parallel[permutation[permuted_index - 1]]
        {
            permutation[permuted_index] = permutation[permuted_index - 1];
            permuted_index -= 1;
        }
        permutation[permuted_index] = plane_index;
    }
    parallel[permutation[0]]
}

fn clip_velocity(input: &[f32; 3], normal: &[f32; 3], output: &mut [f32; 3]) {
    let d = input[0] * normal[0] + input[1] * normal[1] + input[2] * normal[2];
    let backoff = -(d - d.abs() * (OVERCLIP - 1.0));
    output[0] = input[0] + backoff * normal[0];
    output[1] = input[1] + backoff * normal[1];
    output[2] = input[2] + backoff * normal[2];
}

fn dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: &mut [f32; 3]) -> f32 {
    let len = libm::sqrtf(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    let scale = if len <= 0.0 { 1.0 } else { 1.0 / len };
    v[0] *= scale;
    v[1] *= scale;
    v[2] *= scale;
    len
}

