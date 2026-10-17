use playerstate_iw4::{ENTITYNUM_NONE, PM_TYPE_NORMAL_LINKED, PlayerState, UserCmd};

use crate::{
    AdsFracContext, AdsIntentContext, AirMoveContext, CheckLadderContext, CollisionBackend,
    LadderAttachBackend, LadderMoveContext, LadderTraceHit, MantleCapViewContext,
    MantleCapsuleTrace, MantleCheckContext, MantleFindLedgeContext, MantleMoveContext,
    MantleRootDelta, MantleXAnimLength, MeleeChargeWeaponDelays, Pml, SprintContext,
    ViewAngleClamp, WalkMoveContext, air_move, calc_melee_charge_time, check_ladder_move,
    complete_ground_trace, drop_timers, end_tick_velocity, footstep_event, footsteps_bob_cycle,
    ladder_footsteps, ladder_move, mantle, melee_charge_move, should_make_footsteps,
    sync_stance_tail, update_ads_frac, update_ads_intent, update_sprint, update_stance_flags,
    update_stance_target, update_view_angles, update_view_height, walk_move,
};
use playerstate_iw4::pm_flags;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundTraceInput {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub tracemask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveBounds {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub tracemask: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PmoveResult {
    pub pml: Pml,

    pub bounds: MoveBounds,
    pub stance_event: Option<u8>,
    pub reset_torso: bool,
    /// bo2zm: how far a stair step moved the player up or down this move
    /// (Black Ops II's pmove_t viewChange); the camera eases over it.
    pub view_change: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PmoveSingleContext {
    pub walk: WalkMoveContext,

    pub air: AirMoveContext,

    pub bounds: MoveBounds,

    pub view_angles: ViewAngleClamp,

    pub sprint: SprintContext,

    pub ads_intent: AdsIntentContext,

    pub ads_frac: AdsFracContext,

    pub melee_charge: MeleeChargeWeaponDelays,

    pub player_melee_range: f32,

    pub old_buttons: u32,

    /// bo2zm: last frame's forward and side keys (BO2's oldcmd), for the
    /// prone aim hold.
    pub old_move: [i8; 2],

    pub weapon_blocks_prone: bool,
}

/// Longest stretch one step moves; a longer command runs as several steps.
const PMOVE_MAX_STEP_MS: i32 = 66;

/// A command older than this is moved only from this far back.
const PMOVE_MAX_BEHIND_MS: i32 = 150;

/// Moves the player from `ps.command_time` up to `cmd.server_time`: nothing
/// when the command is older than the state, never more than 150 ms of it,
/// in steps of at most 66 ms. Between steps the command (as the step left
/// it, cleared buttons and all) becomes the previous command.
pub fn pmove<C: CollisionBackend, L: MantleXAnimLength, R: MantleRootDelta>(
    ps: &mut PlayerState,
    cmd: &mut UserCmd,
    mut context: PmoveSingleContext,
    collision: &C,
    lengths: &L,
    root: &R,
) -> PmoveResult {
    let server_time = cmd.server_time;
    let mut result = PmoveResult {
        pml: Pml {
            forward: [0.0; 3],
            right: [0.0; 3],
            up: [0.0; 3],
            frametime: 0.0,
            msec: 0,
            walking: 0,
            ground_plane: 0,
            almost_ground_plane: 0,
            ground_trace: [0; 11],
            previous_origin: ps.origin,
            previous_velocity: ps.velocity,
            holdrand: 0,
        },
        bounds: context.bounds,
        stance_event: None,
        reset_torso: false,
        view_change: 0.0,
    };
    if ps.command_time > server_time {
        return result;
    }
    if ps.command_time + PMOVE_MAX_BEHIND_MS < server_time {
        ps.command_time = server_time - PMOVE_MAX_BEHIND_MS;
    }
    let mut view_change = 0.0;
    let mut stance_event = None;
    let mut reset_torso = false;
    while ps.command_time != server_time {
        let step = (server_time - ps.command_time).min(PMOVE_MAX_STEP_MS);
        cmd.server_time = ps.command_time + step;
        result = pmove_single(ps, cmd, context, collision, lengths, root);
        view_change += result.view_change;
        stance_event = result.stance_event.or(stance_event);
        reset_torso |= result.reset_torso;
        context.old_buttons = cmd.buttons;
        context.walk.old_buttons = cmd.buttons;
        context.old_move = [cmd.forwardmove, cmd.rightmove];
    }
    result.view_change = view_change;
    result.stance_event = stance_event;
    result.reset_torso = reset_torso;
    result
}

#[allow(clippy::too_many_lines)]
pub fn pmove_single<C: CollisionBackend, L: MantleXAnimLength, R: MantleRootDelta>(
    ps: &mut PlayerState,
    cmd: &mut UserCmd,
    context: PmoveSingleContext,
    collision: &C,
    lengths: &L,
    root: &R,
) -> PmoveResult {
    let msec = clamped_msec(cmd.server_time.wrapping_sub(ps.command_time));
    // bo2zm: lying down and aiming holds still (BO2 decides it first).
    crate::feel::prone_aim(
        ps,
        cmd,
        context.old_move,
        crate::ads_intent::sniper_scope_raised(ps, context.ads_intent),
    );
    ps.command_time = cmd.server_time;

    let mut pml = Pml {
        forward: [0.0; 3],
        right: [0.0; 3],
        up: [0.0; 3],
        frametime: (msec as f32) * 0.001_f32,
        msec,
        walking: 0,
        ground_plane: 0,
        almost_ground_plane: 0,
        ground_trace: [0; 11],
        previous_origin: ps.origin,
        previous_velocity: ps.velocity,
        holdrand: holdrand(ps.viewangles[1], cmd.server_time),
    };

    let old_yaw = ps.viewangles[1];
    update_view_angles(ps, cmd, context.view_angles);
    // Lying down, the view is held to the body's turn.
    crate::prone_turn::bo2_prone_view(ps, cmd, msec, collision, context.walk.feel.zombies, old_yaw);

    let (forward, right, up) = math_iw4::angle_vectors(ps.viewangles);
    pml.forward = forward;
    pml.right = right;
    pml.up = up;

    // bo2zm: no clip (the cheat menu) flies along the view, nothing else.
    if ps.pm_type == crate::noclip::PM_TYPE_NOCLIP {
        crate::noclip::fly(ps, cmd, forward, right, pml.frametime);
        return PmoveResult {
            pml,
            bounds: context.bounds,
            stance_event: None,
            reset_torso: false,
            view_change: 0.0,
        };
    }

    mantle::clear_hint(ps);

    // bo2zm: Black Ops II's dive to prone starts from a sprint, so it is
    // decided before the sprint and stance updates see the prone press;
    // while it lasts, the dive owns sprint, stance and movement. After its
    // slide the player lies still a moment (their push is ignored), and a
    // slide that has come to a stop is over before the stance update.
    if context.walk.feel.dive {
        if crate::dive::pausing(ps, cmd.server_time) {
            cmd.forwardmove = 0;
            cmd.rightmove = 0;
        }
        crate::dive::end_stopped_slide(ps, cmd.server_time);
    }
    let dive_stance = crate::dive::owns_stance(ps, cmd.server_time);
    let dive_start = context.walk.feel.dive
        && !context.weapon_blocks_prone
        && crate::dive::wants(ps, cmd, context.old_buttons);
    let diving = dive_start || crate::dive::active(ps);

    // bo2zm: the gun can aim through a dive (by request).
    let ads_cmd = crate::dive::ads_cmd(ps, cmd, diving);
    let _ads = update_ads_intent(ps, &ads_cmd, context.old_buttons, context.ads_intent);
    let mut sprint = context.sprint;
    // bo2zm: Stamin-Up sprints longer.
    sprint.weapon_max_sprint_time =
        crate::feel::stamin_up_sprint_time(ps, sprint.weapon_max_sprint_time, context.walk.feel);
    if ps.pm_flags & 3 != 0 {
        sprint.stand_up_clear = collision
            .trace(GroundTraceInput {
                start: ps.origin,
                end: ps.origin,
                mins: context.bounds.mins,
                maxs: context.bounds.maxs,
                tracemask: context.bounds.tracemask & crate::stance::STANCE_ROOM_MASK,
            })
            .allsolid
            == 0;
    }
    let sprint_before = (ps.pm_flags & pm_flags::SPRINTING, ps.sprint_delay);
    let sprint_change = if diving {
        crate::SprintResult::Unchanged
    } else {
        // bo2zm: a sprint in any direction (feel.omni), and a held prone key
        // does not end the sprint while getting up from a dive.
        let sprint_cmd =
            crate::feel::omni_sprint_cmd(&crate::dive::stance_cmd(ps, cmd), context.walk.feel);
        update_sprint(
            ps,
            &sprint_cmd,
            crate::feel::sprint_old_buttons(context.old_buttons),
            sprint,
        )
    };
    crate::feel::update_walking_flag(ps, cmd);
    crate::feel::jump_out_of_sprint(ps, cmd, sprint_before, sprint_change);
    let previous_stance = ps.pm_flags & 3;
    // bo2zm: from the frame after take-off to the end of the still moment,
    // the view drops to prone; a get-up pressed meanwhile is kept.
    let dive_prone = if dive_stance {
        crate::dive::queue_getup(ps, cmd, context.old_buttons);
        crate::dive::force_prone(ps);
        true
    } else {
        !diving && crate::dive::holds_prone(ps, cmd, context.old_buttons)
    };
    if !diving && !dive_prone {
        // bo2zm: a held prone key does not drop the player back down while
        // they get up from a dive. Buttons the stance update clears stay
        // cleared in the real command.
        let mut stance_cmd = crate::dive::stance_cmd(ps, cmd);
        let held_back = cmd.buttons & !stance_cmd.buttons;
        update_stance_flags(
            ps,
            &mut stance_cmd,
            collision,
            context.bounds,
            context.weapon_blocks_prone,
            &context.walk.feel,
        );
        cmd.buttons &= stance_cmd.buttons | held_back;
    }
    let stance_event = if ps.pm_flags & pm_flags::SPRINTING == 0 {
        match (previous_stance, ps.pm_flags & 3) {
            (1, 0) => Some(16),
            (1, 2) => Some(12),
            (2, 0) => Some(14),
            (0, 2) => Some(13),
            _ => None,
        }
    } else {
        None
    };
    let stance_change = update_stance_target(ps);
    // bo2zm: a dive's landing is not an ordinary drop to prone, so it skips
    // prone's 1.8 s lock (which also slows the player and their jump).
    if stance_change == crate::StanceChange::EnteredProne
        && ps.pm_flags & pm_flags::LAST_STAND == 0
        && ps.pm_time == 0
        && !crate::dive::any(ps)
    {
        ps.pm_flags |= 0x2000;
        ps.pm_time = 1800;
    }
    let reset_torso = stance_change != crate::StanceChange::Unchanged
        || sprint_change == crate::SprintResult::Started;
    crate::dive::finish_getup(ps, cmd); // bo2zm
    update_view_height(ps, &pml, cmd, context.walk.cmd_scale.prone_lerp_ms);
    let mut bounds = context.bounds;
    bounds.maxs[2] = sync_stance_tail(ps);
    if crate::dive::active(ps) {
        // bo2zm: through a dive the body shrinks as the view drops.
        bounds.maxs[2] = crate::dive::hull_height(ps);
    }
    // A dive's push does not drop the aim on lying down (the gun aims
    // through a dive).
    crate::stance::bo2_lie_down(ps, &ads_cmd, collision, bounds, previous_stance & pm_flags::PRONE != 0);

    if ps.pm_type == PM_TYPE_NORMAL_LINKED {
        update_ads_frac(ps, pml.msec, context.ads_frac);
        // The trigger link owns the origin: no walk, no air move, no jump, and
        // velocity zeroed every pmove. Viewangles and stance stay live above,
        // and the weapon ticks separately.
        ps.pm_flags &= !pm_flags::LADDER;
        ps.ground_entity_num = ENTITYNUM_NONE;
        ps.velocity = [0.0; 3];
        drop_timers(ps, &pml);
        return PmoveResult {
            pml,
            bounds,
            stance_event,
            reset_torso,
            view_change: 0.0,
        };
    }

    if (ps.pm_flags & pm_flags::MANTLE) == 0 {
        // Mantle root motion owns the path through the ledge. Ground solid
        // correction during that path would push the player back off it.
        complete_ground_trace(ps, &mut pml, bounds, collision, context.walk.feel);
        let mut mantle_tracer = CollisionMantleTrace { collision, bounds };
        let started = (!diving).then(|| {
            // BO2 Zombies: a roof hatch over him comes before any ledge.
            if context.walk.feel.zombies
                && mantle::hatch_check(
                    ps,
                    cmd.buttons,
                    pml.forward,
                    pml.up,
                    &mut mantle_tracer,
                    lengths,
                    root,
                )
            {
                return (ps.pm_flags & pm_flags::MANTLE) != 0;
            }
            mantle::check(
                ps,
                MantleCheckContext {
                    find: MantleFindLedgeContext::default(),
                    buttons: cmd.buttons,
                    forwardmove: cmd.forwardmove,
                    facing_xy: [pml.forward[0], pml.forward[1]],
                    tracemask: bounds.tracemask,
                },
                &mut mantle_tracer,
                lengths,
                root,
            )
        });
        if started == Some(true) {
            mantle::mark_bo2_weapon(ps, context.walk.feel, lengths);
        }
    }

    if (ps.pm_flags & pm_flags::MANTLE) != 0 {
        mantle::cap_view(ps, MantleCapViewContext::default());
        mantle::advance(ps, pml.msec, MantleMoveContext::default(), lengths, root);
        // The aim blends with the gun, after the move.
        update_ads_frac(ps, pml.msec, context.ads_frac);
        return PmoveResult {
            pml,
            bounds,
            stance_event,
            reset_torso,
            view_change: 0.0,
        };
    }

    // The body's slope pitches follow this tick's ground.
    crate::prone_turn::bo2_prone_pitch(ps, &pml, collision, context.walk.feel.zombies);

    drop_timers(ps, &pml);

    if !diving {
        let mut ladder_backend = CollisionLadderBackend { collision, bounds };
        check_ladder_move(
            ps,
            CheckLadderContext {
                server_time: cmd.server_time,
                walking: pml.walking != 0,
                forward_xy: [pml.forward[0], pml.forward[1]],
                forwardmove: cmd.forwardmove,
            },
            &mut ladder_backend,
        );
    }

    let view_change = if diving {
        if dive_start {
            crate::dive::start(ps, &mut pml, cmd, context.walk.jump.jump_height);
        }
        crate::dive::advance(
            ps,
            &pml,
            cmd,
            bounds,
            collision,
            context.walk.jump.jump_height,
            context.air,
        )
    } else if (ps.pm_flags & pm_flags::LADDER) != 0 {
        ladder_move(
            ps,
            &mut pml,
            cmd,
            LadderMoveContext {
                jump: context.walk.jump,
                old_buttons: context.old_buttons,
                player_spectate_speed_scale: context.air.player_spectate_speed_scale,
                zombies: context.walk.feel.zombies,
            },
            bounds,
            collision,
        )
    } else {
        calc_melee_charge_time(ps, context.melee_charge, context.player_melee_range);
        if (ps.pm_flags & pm_flags::MELEE_CHARGE) != 0 {
            melee_charge_move(
                ps,
                &pml,
                bounds.mins,
                bounds.maxs,
                bounds.tracemask,
                collision,
                context.walk.feel.zombies,
            )
        } else if pml.walking == 0 {
            air_move(
                ps,
                &pml,
                cmd,
                context.air,
                bounds,
                collision,
                context.walk.feel.zombies,
            )
        } else {
            walk_move(ps, &mut pml, cmd, context.walk, bounds, collision)
        }
    };

    complete_ground_trace(ps, &mut pml, bounds, collision, context.walk.feel);

    if (ps.pm_flags & pm_flags::LADDER) != 0 {
        ladder_footsteps(ps, pml.msec, cmd.server_time);
    } else {
        let old_bob = ps.bob_cycle as u8;
        // bo2zm: a dive runs the rhythm without footsteps.
        let dive_rhythm = (ps.pm_flags & (crate::dive::PMF_DIVE | crate::dive::PMF_DIVE_SLIDE)) != 0;
        footsteps_bob_cycle(
            ps,
            pml.msec,
            cmd.forwardmove,
            cmd.rightmove,
            pml.almost_ground_plane != 0,
            cmd.server_time,
            context.walk.cmd_scale,
            context.walk.feel,
        );
        if !dive_rhythm {
            footstep_event(
                ps,
                old_bob,
                ps.bob_cycle as u8,
                pml.ground_trace[4],
                should_make_footsteps(ps),
            );
        }
    }

    // The aim blends with the gun, after the move: the move read last
    // frame's aim.
    update_ads_frac(ps, pml.msec, context.ads_frac);
    end_tick_velocity(ps, &pml);
    PmoveResult {
        pml,
        bounds,
        stance_event,
        reset_torso,
        view_change,
    }
}

struct CollisionMantleTrace<'a, C: CollisionBackend> {
    collision: &'a C,
    bounds: MoveBounds,
}

impl<C: CollisionBackend> MantleCapsuleTrace for CollisionMantleTrace<'_, C> {
    fn trace(
        &mut self,
        start: [f32; 3],
        end: [f32; 3],
        mins: [f32; 3],
        maxs: [f32; 3],
        contentmask: u32,
    ) -> trace_iw4::Trace {
        let _ = self.bounds;
        self.collision.trace(GroundTraceInput {
            start,
            end,
            mins,
            maxs,
            tracemask: contentmask,
        })
    }
}

struct CollisionLadderBackend<'a, C: CollisionBackend> {
    collision: &'a C,
    bounds: MoveBounds,
}

impl<C: CollisionBackend> LadderAttachBackend for CollisionLadderBackend<'_, C> {
    fn ladder_trace(
        &mut self,
        origin: [f32; 3],
        dir: [f32; 3],
        dist: f32,
    ) -> Option<LadderTraceHit> {
        let end = [
            origin[0] + dir[0] * dist,
            origin[1] + dir[1] * dist,
            origin[2] + dir[2] * dist,
        ];
        let mut mins = self.bounds.mins;
        let mut maxs = self.bounds.maxs;
        mins[0] += 6.0;
        mins[1] += 6.0;
        mins[2] = 8.0;
        maxs[0] -= 6.0;
        maxs[1] -= 6.0;
        if maxs[2] < 8.0 {
            maxs[2] = mins[2];
        }
        let hit = self.collision.trace(GroundTraceInput {
            start: origin,
            end,
            mins,
            maxs,
            tracemask: self.bounds.tracemask,
        });
        if hit.fraction >= 1.0 {
            return None;
        }
        Some(LadderTraceHit {
            fraction: hit.fraction,
            normal: hit.normal,
            surface_flags: hit.surface_flags,
        })
    }
}

pub use pmove_single as PmoveSingle;

fn clamped_msec(delta: i32) -> i32 {
    delta.clamp(1, 200)
}

fn holdrand(view_yaw: f32, server_time: i32) -> i32 {
    let mut value = view_yaw.to_bits().wrapping_add(server_time as u32);

    for _ in 0..4 {
        value = value.wrapping_mul(0x343fdu32).wrapping_add(0x269ec3u32);
    }
    value as i32
}
