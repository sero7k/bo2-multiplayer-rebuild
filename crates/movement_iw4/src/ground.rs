use playerstate_iw4::{ENTITYNUM_NONE, PlayerState};
use trace_iw4::Trace;

use crate::{CollisionBackend, GroundTraceInput, MoveBounds, Pml, jump};

const STARTSOLID_RETRY_EPSILON: f32 = 0.001;
/// bo2zm: PM_GroundTraceMissed's probe in the air (1) and on leaving the
/// ground (64), and the gap that still counts as almost on the ground.
const MISSED_AIR_PROBE: f32 = 1.0;
const MISSED_LEFT_PROBE: f32 = 64.0;
const MISSED_ALMOST_FRACTION: f32 = 1.0 / 64.0;

pub fn complete_ground_trace<C: CollisionBackend>(
    ps: &mut PlayerState,
    pml: &mut Pml,
    bounds: MoveBounds,
    collision: &C,
    feel: crate::Bo2Feel, // bo2zm
) {
    let origin = ps.origin;
    let on_turret = (ps.e_flags & 0xc00) != 0;
    let (above, below) = match (on_turret, feel.on) {
        (false, _) => (0.25, 0.25),
        // bo2zm: on a turret Black Ops II still probes 1 unit down.
        (true, true) => (0.0, 1.0),
        (true, false) => (0.0, 0.0),
    };
    let input = GroundTraceInput {
        start: [origin[0], origin[1], origin[2] + above],
        end: [origin[0], origin[1], origin[2] - below],
        mins: bounds.mins,
        maxs: bounds.maxs,
        tracemask: bounds.tracemask,
    };

    let mut trace = collision.trace(input);
    copy_ground_trace(pml, &trace);

    if trace.allsolid != 0 {
        let Some(corrected) =
            collision.correct_solid(origin, bounds.mins, bounds.maxs, bounds.tracemask)
        else {
            clear_ground_state(ps, pml);
            jump::clear_state(ps);
            return;
        };
        ps.origin = corrected.origin;
        trace = corrected.trace;
        copy_ground_trace(pml, &trace);
    }

    if trace.startsolid != 0 {
        let retry = GroundTraceInput {
            start: [origin[0], origin[1], origin[2] - STARTSOLID_RETRY_EPSILON],
            ..input
        };
        trace = collision.trace(retry);
        if trace.startsolid != 0 {
            if feel.on {
                // bo2zm: BO2 keeps a player stuck in the floor on it.
                ps.ground_entity_num = trace_entity_id(&trace);
                pml.ground_plane = 1;
                pml.almost_ground_plane = 1;
                pml.walking = u32::from(trace.walkable);
                return;
            }
            clear_ground_state(ps, pml);
            return;
        }
        copy_ground_trace(pml, &trace);
    }

    if trace.fraction == 1.0 {
        if feel.on {
            ground_trace_missed(ps, pml, bounds, collision);
            return;
        }
        clear_ground_state(ps, pml);
        return;
    }

    let velocity_dot_normal = ps.velocity[2] * trace.normal[2]
        + trace.normal[0] * ps.velocity[0]
        + ps.velocity[1] * trace.normal[1];
    // bo2zm: BO2's kick-off here only runs in collision mode 0; its
    // phys_player_collision_mode is 1, so no kick-off there either.
    if (ps.pm_flags & 8) == 0 && ps.velocity[2] > 0.0 && 10.0 < velocity_dot_normal {}

    if trace.walkable != 0 && trace.contents != 0x0200_0000 {
        pml.ground_plane = 1;
        pml.almost_ground_plane = 1;
        pml.walking = 1;
        if ps.ground_entity_num == ENTITYNUM_NONE {
            crate::crash_land(ps, pml, feel);
        }
        let entity = trace_entity_id(&trace);
        ps.ground_entity_num = entity;
        collision.touch_entity(entity);
    } else {
        ps.ground_entity_num = ENTITYNUM_NONE;
        pml.ground_plane = 1;
        pml.almost_ground_plane = 1;
        pml.walking = 0;
        jump::clear_state(ps);
    }
}

/// bo2zm: BO2's PM_GroundTraceMissed. In the air a 1-unit probe, on leaving
/// the ground a 64-unit one, decides whether the player is still almost on
/// the ground (footsteps and the bob keep going down stairs and off curbs).
fn ground_trace_missed<C: CollisionBackend>(
    ps: &mut PlayerState,
    pml: &mut Pml,
    bounds: MoveBounds,
    collision: &C,
) {
    let origin = ps.origin;
    let was_airborne = ps.ground_entity_num == ENTITYNUM_NONE;
    let depth = if was_airborne {
        MISSED_AIR_PROBE
    } else {
        MISSED_LEFT_PROBE
    };
    let trace = collision.trace(GroundTraceInput {
        start: origin,
        end: [origin[0], origin[1], origin[2] - depth],
        mins: bounds.mins,
        maxs: bounds.maxs,
        tracemask: bounds.tracemask,
    });
    let almost = if was_airborne {
        trace.fraction != 1.0
    } else {
        trace.fraction < MISSED_ALMOST_FRACTION
    };
    ps.ground_entity_num = ENTITYNUM_NONE;
    pml.ground_plane = 0;
    pml.almost_ground_plane = u32::from(almost);
    pml.walking = 0;
}

fn clear_ground_state(ps: &mut PlayerState, pml: &mut Pml) {
    ps.ground_entity_num = ENTITYNUM_NONE;
    pml.ground_plane = 0;
    pml.almost_ground_plane = 0;
    pml.walking = 0;
}

fn copy_ground_trace(pml: &mut Pml, trace: &Trace) {
    pml.ground_trace = [
        trace.fraction.to_bits(),
        trace.normal[0].to_bits(),
        trace.normal[1].to_bits(),
        trace.normal[2].to_bits(),
        trace.surface_flags,
        trace.contents,
        trace.material,
        trace.hit_type as u32,
        u32::from(trace.hit_id) | (u32::from(trace.model_index) << 16),
        u32::from(trace.part_name) | (u32::from(trace.part_group) << 16),
        u32::from(trace.allsolid)
            | (u32::from(trace.startsolid) << 8)
            | (u32::from(trace.walkable) << 16),
    ];
}

fn trace_entity_id(trace: &Trace) -> i32 {
    i32::from(trace_iw4::trace_get_entity_hit_id(
        trace.hit_type,
        trace.hit_id,
    ))
}
