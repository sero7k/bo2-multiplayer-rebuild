use crate::frame::FrameWorld;
use std::cell::Cell;

use bevy_ecs::prelude::{Resource, World};
use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule};
use movement_iw4::{
    ANGLE2SHORT, AirMoveContext, CmdScaleWalkContext, CollisionBackend, GroundTraceInput,
    JumpLaunchContext, MoveBounds, PmoveSingleContext, SHORT2ANGLE, SprintContext, ViewAngleClamp,
    WalkMoveContext, consume_player_events, footsteps_anim_move_type, get_max_sprint_time, pmove,
};
use trace_iw4::{HITTYPE_ENTITY, Trace};

use crate::bullet_collision::{LinkedBrushCollisionBrush, PLAYER_MAXS, PLAYER_MINS};
use crate::combat::{advance_weapon_command, phase_emit, phase_trace};
use crate::identities::MatchPhase;
use crate::input::{ClientAction, TickInput};
use crate::match_state::{ClientLifecycle, EventAudience, SimEvent};
use crate::snapshot::Snapshot;
use crate::world::{
    ClientId, SimBrush, SimClipBsp, SimClipMesh, SimState, Tick, clip_move_to_bmodels, clip_trace,
    give_weapon_to_ps_akimbo, gsc_give_weapon_is_akimbo, inventory_add_weapon,
};
use playerstate_iw4::PlayerState;
use playerstate_iw4::buttons;

#[derive(Resource)]
pub(crate) struct StepRequest {
    pub(crate) tick: Tick,
    pub(crate) input: TickInput,
    msec: i32,
    pub(crate) reason: crate::StepReason,
    output: Option<Snapshot>,
}

/// bo2mp: a part of the authority step slower than this names itself in
/// the log (ms), profiling or not: a freeze he would feel says where it was.
const STALL_MS: f64 = 250.0;

/// Times one part of the step; a slow one logs when it ends.
pub(crate) struct StallGuard(std::time::Instant, &'static str, Option<f64>);

impl StallGuard {
    pub(crate) fn new(part: &'static str) -> Self {
        Self(std::time::Instant::now(), part, diag::thread_cpu_ms())
    }
}

impl Drop for StallGuard {
    fn drop(&mut self) {
        let ms = self.0.elapsed().as_secs_f64() * 1000.0;
        if ms > STALL_MS {
            diag::warn!(Sim, "bo2mp stall: {} took {ms:.0} ms{}", self.1, diag::busy_note(self.2));
        }
    }
}

pub(crate) fn schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            advance_time_system,
            expire_transient_events_system,
            crate::script::apply_disconnects,
            crate::script::advance_mechanics,
            crate::script::advance_scheduler,
            crate::t6::advance,
            (
                apply_script_signals_system,
                apply_actions_system,
                crate::script::sync_players,
                crate::script::sync_presence,
                run_players_system,
                record_collision_state_system,
                run_entity_types_system,
                dispatch_touches_system,
                crate::t6::collect_item_pickups,
                crate::script::sync_engine_events,
                finalize_system,
                publish_snapshot_system,
            )
                .chain()
                .run_if(crate::script::healthy),
        )
            .chain(),
    );
    schedule
}

pub(crate) fn run_schedule(
    ecs: &mut World,
    schedule: &mut Schedule,
    tick: Tick,
    input: &TickInput,
    msec: i32,
    reason: crate::StepReason,
) -> Result<Snapshot, crate::script::Fault> {
    crate::script::preflight(ecs, tick, reason)?;
    assert!(
        !ecs.contains_resource::<StepRequest>(),
        "simulation schedule already has a pending frame"
    );
    ecs.insert_resource(StepRequest {
        tick,
        input: input.clone(),
        msec: msec.clamp(1, 200),
        reason,
        output: None,
    });
    schedule.run(ecs);
    let request = ecs
        .remove_resource::<StepRequest>()
        .expect("simulation request missing");
    if let Some(fault) = ecs.resource::<crate::script::Runtime>().fault.as_ref() {
        return Err(fault.clone());
    }
    Ok(request
        .output
        .expect("simulation schedule did not publish a snapshot"))
}

fn frame_world(world: &mut World) -> FrameWorld<'_> {
    crate::frame::FrameWorld::from_world(world)
}

fn apply_script_signals_system(world: &mut World) {
    let signals = crate::script::take_signals(world);
    if signals.is_empty() {
        return;
    }
    let mut frame = frame_world(world);
    for signal in signals {
        match &*signal {
            crate::script::EXIT_LEVEL
                if !matches!(
                    frame.phase(),
                    MatchPhase::Intermission | MatchPhase::PostGame
                ) =>
            {
                frame.set_phase(MatchPhase::Intermission);
            }
            crate::script::MAP_RESTART => {
                let tick = frame.ecs().resource::<StepRequest>().tick;
                crate::script::restart_level(frame.ecs(), tick);
            }
            name => crate::script::host::iw4_gametype::apply_level_notify(&mut frame, name),
        }
    }
}

pub fn phase_materialize_entity_dobjs(world: &mut SimState) {
    for capabilities in world.entity_collision_capabilities_mut() {
        if let Some(dobj) = &mut capabilities.dobj {
            dobj.materialize();
        }
    }
}

fn phase_animated_map_models(world: &mut FrameWorld, tick: Tick, msec: i32) {
    let dt = msec as f32 / 1000.0;

    let at_time = i32::try_from(tick.0.saturating_mul(crate::MATCH_TICK_MS)).unwrap_or(i32::MAX);
    let mut mover_apos = Vec::new();
    world.visit_script_movers(|mover| {
        mover_apos.push((
            mover.id,
            crate::gentity::apos_from_entity_state(&mover.state),
        ));
    });
    for capabilities in world.entity_collision_capabilities_mut() {
        let Some(dobj) = capabilities.dobj.as_mut() else {
            continue;
        };
        if dobj.play_anim.is_some() {
            dobj.advance_script_model_play_anim(dt);
        }
        if let Some(id) = capabilities.owner.script_model() {
            if let Some((_, apos)) = mover_apos.iter().find(|(mover_id, _)| *mover_id == id) {
                dobj.apply_trajectory_apos_at(*apos, at_time);
                continue;
            }
        }
        if dobj.apos.is_some() {
            dobj.apply_apos_at(at_time);
        }
    }
}

fn phase_stuck_in_client(world: &mut FrameWorld) {
    let mut ids: Vec<ClientId> = world.client_ids_sorted();
    ids.retain(|id| {
        world
            .client_meta(*id)
            .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
            && world.player(*id).is_some()
    });
    let mut rows: Vec<gamemode_iw4::StuckClient> = ids
        .iter()
        .filter_map(|id| world.player(*id).map(stuck_row))
        .collect();
    let mut pairs = Vec::new();
    let eject_speed = gamemode_iw4::G_PLAYER_COLLISION_EJECT_SPEED_DEFAULT;
    let mut holdrand = *world.stuck_holdrand_mut();
    for i in 0..rows.len() {
        if let Some(pair) = gamemode_iw4::stuck_in_client(i, &mut rows, eject_speed, &mut holdrand)
        {
            pairs.push((ids[pair.self_idx], ids[pair.other_idx]));
        }
    }
    *world.stuck_holdrand_mut() = holdrand;
    for (id, row) in ids.iter().zip(rows.iter()) {
        if let Some(ps) = world.player_mut(*id) {
            ps.velocity[0] = row.velocity[0];
            ps.velocity[1] = row.velocity[1];
            ps.pm_time = row.pm_time;
            ps.pm_flags = row.pm_flags;
        }
    }
    world.set_stuck_ejects(pairs);
}

fn stuck_row(ps: &PlayerState) -> gamemode_iw4::StuckClient {
    gamemode_iw4::StuckClient {
        origin: ps.origin,
        velocity: ps.velocity,
        speed: ps.speed,
        other_flags: ps.other_flags,
        health: ps.health,
        pm_time: ps.pm_time,
        pm_flags: ps.pm_flags,
        maxs_x: PLAYER_MAXS[0],
        bounds_mid: [
            ps.origin[0] + (PLAYER_MINS[0] + PLAYER_MAXS[0]) * 0.5,
            ps.origin[1] + (PLAYER_MINS[1] + PLAYER_MAXS[1]) * 0.5,
            ps.origin[2] + (PLAYER_MINS[2] + PLAYER_MAXS[2]) * 0.5,
        ],
        bounds_half: [
            (PLAYER_MAXS[0] - PLAYER_MINS[0]) * 0.5,
            (PLAYER_MAXS[1] - PLAYER_MINS[1]) * 0.5,
            (PLAYER_MAXS[2] - PLAYER_MINS[2]) * 0.5,
        ],
    }
}

fn advance_time_system(world: &mut World) {
    let tick;
    {
        let mut request = world.resource_mut::<StepRequest>();
        request.input.canonicalize();
        tick = request.tick;
    }
    let mut frame = frame_world(world);
    frame.mark_running();
    frame.begin_entity_frame(tick);
}

fn expire_transient_events_system(world: &mut World) {
    let tick = world.resource::<StepRequest>().tick;
    let mut frame = frame_world(world);
    frame.enter_kernel_phase(crate::gentity::KernelPhase::ExpireTransientEvents);
    frame.expire_dying_missiles(tick);
    frame.clear_tick_events(tick);
}

fn apply_actions_system(world: &mut World) {
    let tick = world.resource::<StepRequest>().tick;
    let actions = world.resource::<StepRequest>().input.actions.clone();
    let mut frame = frame_world(world);
    for (id, action) in &actions {
        if action_needs_player_row(action) {
            let _ = frame.ensure_player(*id);
        } else {
            let _ = frame.client_meta_mut(*id);
        }
    }

    frame.enter_kernel_phase(crate::gentity::KernelPhase::ApplyActions);
    apply_actions(&mut frame, tick, &actions);
}

fn run_players_system(ecs: &mut World) {
    let _stall = StallGuard::new("run_players");
    let tick = ecs.resource::<StepRequest>().tick;
    let level_time = crate::level_time_ms(tick);
    let input = ecs.resource::<StepRequest>().input.clone();
    let authority = ecs
        .resource::<StepRequest>()
        .reason
        .advances_authority_world();
    let mut world = frame_world(ecs);
    world.enter_kernel_phase(crate::gentity::KernelPhase::RunPlayers);

    let allow_move = matches!(world.phase(), MatchPhase::Playing | MatchPhase::Warmup);
    let content = world.content();
    let cmodel_models = &content.clip_cmodels().models;
    let (brushes, bsp, mesh) = (
        content.clip_brushes(),
        content.clip_bsp(),
        content.clip_mesh(),
    );
    let original_buttons = world.old_buttons_mut().clone();
    let original_angles = world.old_cmd_angles_mut().clone();
    let original_moves = world.old_cmd_moves_mut().clone();
    let mut consumed = Vec::new();
    if allow_move {
        phase_materialize_entity_dobjs(&mut world);
        for (id, cmd) in &input.cmds {
            world.select_lagcomp_command(*id, cmd.server_time);
            if !world
                .client_meta(*id)
                .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
            {
                continue;
            }
            let Some(ps) = world.player(*id) else {
                continue;
            };

            let delta = cmd.server_time.wrapping_sub(ps.command_time);
            if delta <= 0 {
                continue;
            }

            let old_buttons = world
                .old_buttons_mut()
                .iter()
                .find(|(c, _)| c == id)
                .map_or(0, |(_, buttons)| *buttons);
            let Some(ps) = world.player(*id).copied() else {
                continue;
            };
            let weapon = ps.weapon;
            let scales = world.scales_for(weapon);
            let facts = world.combat_facts_for(weapon);
            let aim_down_sight = facts.is_some_and(|f| f.aim_down_sight);
            let overlay_reticle = facts.map(|f| f.overlay_reticle).unwrap_or(0);
            let (ads_in_rate, ads_out_rate, rechamber_while_ads, ads_fire_only) = facts
                .map(|f| f.ads_frac_context())
                .unwrap_or((1.0 / 200.0, 1.0 / 200.0, true, false));
            let ads_reload = facts
                .map(|f| f.ads_reload_rule())
                .unwrap_or((false, 0, false));
            let predicting = !world.publishes_snapshot();
            let sent = world.feel();
            let mut feel = bo2_feel(world.ecs(), predicting, sent);
            if !predicting {
                // Every snapshot carries the rules to the other PCs.
                world.set_feel(feel);
            }
            if feel.on
                && let Some(f) = facts
            {
                // The gun in hand sets its own footstep rhythm scales.
                feel.sprint_cycle_scale = f.sprint_cycle_scale;
                feel.ducked_sprint_cycle_scale = f.ducked_sprint_cycle_scale;
                feel.dtp_cycle_scale = f.dtp_cycle_scale;
            }
            let ads_allowed = facts.is_some_and(|f| {
                weapon != 0 && {
                    let mut allow = f.ads_allow_facts();

                    allow.clip_index = weapon_iw4::clip_table_key(allow.clip_index, weapon);
                    let allowed = if feel.on {
                        weapon_iw4::is_ads_allowed_bo2
                    } else {
                        weapon_iw4::is_ads_allowed
                    };
                    allowed(&ps, &allow, &ps.ammoclip, ps.last_weapon_hand.min(1) as u8)
                }
            });
            let (ads_in_rate, ads_out_rate) = movement_iw4::quickdraw_ads_rates(
                &ps,
                ads_in_rate,
                ads_out_rate,
                facts.is_some_and(|f| f.weap_class == WEAPCLASS_SNIPER),
                feel,
            );
            let (melee_delay_ms, melee_charge_delay_ms) = facts
                .map(|f| (f.melee_delay_ms, f.melee_charge_delay_ms))
                .unwrap_or((0, 0));
            let mut context = pmove_context(
                old_buttons,
                scales,
                ads_allowed,
                aim_down_sight,
                ads_in_rate,
                ads_out_rate,
                rechamber_while_ads,
                ads_reload,
                ads_fire_only,
                melee_delay_ms,
                melee_charge_delay_ms,
                overlay_reticle,
                world
                    .client_meta(*id)
                    .and_then(|m| m.shellshock.as_ref())
                    .map_or(1.0, |shock| {
                        movement_iw4::shellshock_walk_scale(
                            shock.movement,
                            shock.movement_scale,
                            feel,
                        )
                    }),
                feel,
            );
            // bo2mp: the Hybrid Optic's swap key is the sprint key; tapping it while aimed
            // must not drop ADS (it only swaps the sight).
            context.ads_intent.sprint_hold_ads = facts.is_some_and(|f| f.optic_switch);
            // bo2mp: in a scorestreak vehicle his view keeps to its limits.
            if let Some([up, down]) = world.client_meta(*id).and_then(|m| m.view_pitch_clamp) {
                context.view_angles.pitch_up = up;
                context.view_angles.pitch_down = down;
            }
            crate::script::player_commands(world.ecs(), id.0, cmd.buttons, old_buttons);
            let mut cmd = *cmd;
            crate::script_player::constrain_cmd(&mut world, *id, &mut cmd, feel.zombies);
            // bo2mp: a scorestreak switch the scripts would refuse never starts.
            crate::t6::gate_streak_switch(world.ecs(), id.0, &mut cmd);
            crate::script::select_location(world.ecs(), id.0, &mut cmd, old_buttons);
            if world
                .client_meta(*id)
                .is_some_and(|m| m.remote_missile.is_some())
            {
                crate::remote_missile::steer(&mut world, *id, &cmd, delta.min(200));
                cmd.forwardmove = 0;
                cmd.rightmove = 0;
                cmd.buttons &= playerstate_iw4::buttons::CROUCH | playerstate_iw4::buttons::PRONE;
            }
            let commanded_move = cmd.forwardmove != 0 || cmd.rightmove != 0;
            // bo2zm: the keys as sent, before the move holds any back.
            let sent_move = [cmd.forwardmove, cmd.rightmove];
            context.old_move = world.old_cmd_move(*id).unwrap_or([0, 0]);
            // bo2zm: a brush hidden from this player does not stop him (BO2's
            // query_should_pass_gentity); it stops the others.
            let linked_brushes: Vec<LinkedBrushCollisionBrush> = world
                .entity_collision_capabilities()
                .iter()
                .flat_map(|c| c.solid_brushes_for(id.0).iter().cloned())
                .collect();
            let bodies = alive_body_clips(&world);
            let blockers = oriented_blocker_brushes(&world);
            let slick = world.client_meta(*id).is_some_and(|m| m.controls.slick);
            let glass_damage = world.world_objects().glass_damage_pairs();
            let backend = ClipBackend {
                brushes: &brushes,
                bsp: &bsp,
                mesh: &mesh,
                glass_damage: &glass_damage,
                bodies: &bodies,
                self_entnum: id.0 as u16,
                cmodels: &cmodel_models,
                linked_brushes: &linked_brushes,
                blockers: &blockers,
                slick,
            };
            let script = world.player_anim_script();
            let mantle = world.xanims();
            let applied_mt = Cell::new(None);
            // bo2zm: a bus rider is carried through the map's clip brushes (BO2
            // moves him with its mover, no trace: G_PushEntity); only the
            // movement code's own 1-unit fix runs then, not the wider unstick
            // (runs/sl3: the bus's front sweeps through the road's player clip
            // on its turns and shoved him 1-3 units each tick). The client's
            // predicted step has no riders list: it starts from the server's
            // place, so it skips it too while he stands on a mover or is off
            // the ground (runs/sl4: 9 predicted shoves, none on the server).
            let carried = world
                .ecs()
                .get_resource::<crate::t6::Zm>()
                .is_some_and(|zm| zm.riders.carried.contains(&id.0))
                || (!authority
                    && world.player(*id).is_some_and(|ps| {
                        ps.ground_entity_num != i32::from(trace_iw4::ENTITYNUM_WORLD)
                    }));
            let (
                walking,
                linked_bounds,
                anim_movetype,
                view_w,
                primary,
                moved_from,
                moved_to,
                stance_event,
                reset_torso,
                view_change,
            ) = {
                let ps = world
                    .player_mut(*id)
                    .expect("Alive client has a player row");

                if ps.shellshock_time.wrapping_add(ps.shellshock_duration) < level_time {
                    ps.pm_flags &= !playerstate_iw4::pm_flags::SHELLSHOCKED;
                }
                // bo2zm: a spot a hair inside a brush (Ascension's lift top is 0.1 above its
                // spawn spots) leaves him unable to move; lift him the smallest step that clears.
                let stuck_at = ps.origin;
                if !carried {
                    ps.origin = unstick_feet(&backend, ps.origin);
                }
                if ps.origin != stuck_at {
                    unstick_log(&backend, stuck_at, ps.origin);
                }
                // bo2zm: BO2 sets the walk speed every frame (move-faster +7%).
                if let Some(speed) = movement_iw4::player_speed(ps, feel) {
                    ps.speed = speed;
                }
                let moved_from = ps.origin;
                let result = pmove(
                    ps,
                    &mut cmd,
                    context,
                    &backend,
                    mantle.as_ref(),
                    mantle.as_ref(),
                );
                let pml = result.pml;
                let anim_movetype = footsteps_anim_move_type(
                    ps,
                    cmd.forwardmove,
                    cmd.rightmove,
                    pml.almost_ground_plane != 0,
                );
                let (view_w, primary) = crate::pmove_anim_weapon_ids(ps);
                let moved_to = ps.origin;
                stance_room_trace(&backend, ps);
                (
                    pml.walking as i32,
                    result.bounds,
                    anim_movetype,
                    view_w,
                    primary,
                    moved_from,
                    moved_to,
                    result.stance_event,
                    result.reset_torso,
                    result.view_change,
                )
            };
            world
                .client_meta_mut(*id)
                .input_receipt
                .record(commanded_move, moved_from, moved_to);
            if let Some(event) = stance_event {
                crate::combat::apply_player_anim_event(&mut world, *id, event);
            }
            // The torso restart on a stance change or a sprint start is the
            // MW2 script's (its own animation-tree numbering): without that
            // script the values are Black Ops II's picks, which it would
            // overwrite with an MW2-numbered "none" (to BO2's script a stray
            // animation, entry 0x200, on his torso as each sprint starts).
            if reset_torso
                && script.is_some()
                && let Some(ps) = world.player_mut(*id)
            {
                crate::player_anim_script::reset_stance_torso(ps);
            }
            if let Some(movetype) = anim_movetype {
                let view_facts = world.combat_facts_for(view_w);
                let primary_facts = world.combat_facts_for(primary);
                let ps = world
                    .player_mut(*id)
                    .expect("Alive client has a player row");
                let conds = crate::anim_conditions_from_pmove(ps, view_facts, primary_facts);
                let requested_matches = script
                    .as_ref()
                    .map(|script| script.matches_movetype(movetype, &conds))
                    .unwrap_or(true);
                if let Some(script) = script.as_ref() {
                    script.apply(ps, movetype, id.0, &conds);
                }
                if requested_matches {
                    applied_mt.set(Some(movetype));
                }
            }
            if let Some(mt) = applied_mt.get() {
                world.set_last_anim_movetype(*id, mt);
            }
            world.set_pmove_walking(*id, walking);
            world.set_pmove_view_change(*id, view_change); // bo2zm
            world.link_player_area(*id, linked_bounds);

            // bo2mp: Black Ops II's switch rules (the instaswap) follow the
            // match's rules each step, on the host and in a joiner's
            // prediction alike (the rules come with every snapshot); a flag
            // set once by the host's script install left a joiner predicting
            // MW2's switches, and stayed on into later non-BO2 matches.
            weapon_iw4::set_bo2_switch_rules(feel.on);
            let shots = advance_weapon_command(&mut world, tick, *id, cmd, delta.min(200));
            for shot in shots {
                crate::missile::fire_accepted_shot(&mut world, tick, &shot);

                let hitbox_ids = [*id];
                let hitboxes = if world.publishes_snapshot() {
                    None
                } else {
                    Some(hitbox_ids.as_slice())
                };
                world.record_collision_history(tick, 0, hitboxes);
                if world.publishes_snapshot() {
                    world.record_entity_collision_history(tick);
                }
                let emissions = phase_emit(&world, core::slice::from_ref(&shot));
                phase_trace(&mut world, tick, &emissions);
            }
            crate::equipment::phase_offhand(&mut world, tick, &[(*id, cmd)]);
            world.set_old_cmd(*id, cmd.buttons, cmd.angles);
            world.set_old_cmd_move(*id, sent_move);
            consumed.push((*id, cmd));
        }
    }

    *world.old_buttons_mut() = original_buttons;
    *world.old_cmd_angles_mut() = original_angles;
    *world.old_cmd_moves_mut() = original_moves;

    restamp_debug_move_look(&mut world, &input.actions);

    if allow_move && world.publishes_snapshot() {
        phase_stuck_in_client(&mut world);
    }
    drop(world);
    let mut request = ecs.resource_mut::<StepRequest>();
    if allow_move {
        request.input.cmds = consumed;
    }
}

fn record_collision_state_system(ecs: &mut World) {
    let _stall = StallGuard::new("record_collision_state");
    let tick = ecs.resource::<StepRequest>().tick;
    let msec = ecs.resource::<StepRequest>().msec;
    let cmd_ids: Vec<crate::world::ClientId> = ecs
        .resource::<StepRequest>()
        .input
        .cmds
        .iter()
        .map(|(id, _)| *id)
        .collect();
    let mut world = frame_world(ecs);
    world.enter_kernel_phase(crate::gentity::KernelPhase::RecordCollisionState);

    let hitbox_cmds = if world.publishes_snapshot() {
        None
    } else {
        Some(cmd_ids)
    };
    world.record_collision_history(tick, msec, hitbox_cmds.as_deref());
}

fn run_entity_types_system(ecs: &mut World) {
    let tick = ecs.resource::<StepRequest>().tick;
    let msec = ecs.resource::<StepRequest>().msec;
    let mut world = frame_world(ecs);
    let allow_move = world.phase() == MatchPhase::Playing;
    world.enter_kernel_phase(crate::gentity::KernelPhase::RunEntityTypes);
    if allow_move {
        {
            let _stall = StallGuard::new("materialize_entity_dobjs");
            phase_materialize_entity_dobjs(&mut world);
        }

        if world.publishes_snapshot() {
            let _stall = StallGuard::new("entity_collision_history");
            world.record_entity_collision_history(tick);
        }

        {
            let _stall = StallGuard::new("remote_missiles");
            crate::remote_missile::advance(&mut world, tick);
        }
        {
            let _stall = StallGuard::new("entity_thinks");
            crate::entity_run::phase_run_entity_thinks(&mut world, tick);
        }
        if world.publishes_snapshot() {
            {
                let _stall = StallGuard::new("glass");
                world.world_objects_mut().glass_update(
                    i32::try_from(tick.0.saturating_mul(crate::MATCH_TICK_MS)).unwrap_or(i32::MAX),
                );
            }
            let _stall = StallGuard::new("animated_map_models");
            phase_animated_map_models(&mut world, tick, msec);
        }
    } else {
        crate::entity_run::phase_walk_entity_thinks(&mut world);
    }
}

fn dispatch_touches_system(ecs: &mut World) {
    let _stall = StallGuard::new("dispatch_touches");
    let tick = ecs.resource::<StepRequest>().tick;
    let input = ecs.resource::<StepRequest>().input.clone();
    let mut world = frame_world(ecs);
    let allow_move = world.phase() == MatchPhase::Playing;
    world.enter_kernel_phase(crate::gentity::KernelPhase::DispatchTouches);
    if allow_move {
        crate::item::phase_touch_items(&mut world, tick);
    }
    let command_buttons: Vec<_> = input
        .cmds
        .iter()
        .map(|(id, cmd)| (id.0, cmd.buttons))
        .collect();
    let latest: std::collections::BTreeMap<_, _> = input.cmds.iter().copied().collect();
    let latest_cmds: Vec<_> = latest.into_iter().collect();
    let cmds: Vec<_> = latest_cmds
        .iter()
        .map(|(id, cmd)| (id.0, cmd.buttons))
        .collect();
    let old: Vec<(u32, u32)> = world
        .old_buttons_mut()
        .iter()
        .map(|(id, bits)| (id.0, *bits))
        .collect();
    let presses = crate::collect_use_presses(&command_buttons, &old);
    world.stamp_use_presses(presses.clone());
    if allow_move && world.publishes_snapshot() {
        crate::item::phase_use_items(&mut world, tick, &presses, &cmds);
    }
}

fn finalize_system(ecs: &mut World) {
    let _stall = StallGuard::new("finalize");
    let tick = ecs.resource::<StepRequest>().tick;
    let input = ecs.resource::<StepRequest>().input.clone();
    let mut world = frame_world(ecs);
    world.enter_kernel_phase(crate::gentity::KernelPhase::Finalize);
    for &(id, cmd) in &input.cmds {
        emit_attack_events(&mut world, tick, &[(id, cmd)]);
        if world.client_meta(id).is_some() {
            world.set_old_cmd(id, cmd.buttons, cmd.angles);
            world.set_old_cmd_move(id, [cmd.forwardmove, cmd.rightmove]);
        }
    }

    let mut old_buttons = std::mem::take(world.old_buttons_mut());
    old_buttons.retain(|(id, _)| world.client_meta(*id).is_some());
    *world.old_buttons_mut() = old_buttons;
    let mut old_angles = std::mem::take(world.old_cmd_angles_mut());
    old_angles.retain(|(id, _)| world.client_meta(*id).is_some());
    *world.old_cmd_angles_mut() = old_angles;
    let mut old_moves = std::mem::take(world.old_cmd_moves_mut());
    old_moves.retain(|(id, _)| world.client_meta(*id).is_some());
    *world.old_cmd_moves_mut() = old_moves;

    harvest_predictable_events(&mut world, tick);
    world.script_gaps_mut().report();
}

fn publish_snapshot_system(ecs: &mut World) {
    let _stall = StallGuard::new("publish_snapshot");
    let tick = ecs.resource::<StepRequest>().tick;
    let mut world = frame_world(ecs);
    world.enter_kernel_phase(crate::gentity::KernelPhase::PublishSnapshot);

    if world.publishes_snapshot() {
        emit_player_ticks(&world, tick);
    }
    let snapshot = if world.publishes_snapshot() {
        let snapshot = world.snapshot(tick);
        emit_snapshot_events(&world, &snapshot);
        snapshot
    } else {
        Snapshot::unpublished(tick)
    };
    drop(world);
    ecs.resource_mut::<StepRequest>().output = Some(snapshot);
}

fn emit_player_ticks(world: &FrameWorld, tick: Tick) {
    let bot_name = entity_iw4::pack_client_state_name("bot");
    let level_time_ms = crate::level_time_ms(tick);
    let mut rows = Vec::new();
    world.visit_players(|id, ps| {
        rows.push((
            id,
            ps.command_time,
            ps.origin,
            ps.velocity[2],
            ps.viewangles[1],
            ps.viewangles[0],
            ps.jump_time,
            ps.weaponstate_primary,
            ps.legs_anim,
            ps.torso_anim,
        ));
    });
    for (id, time_ms, origin, vz, yaw, pitch, jump_time, weaponstate, legs_anim, torso_anim) in rows
    {
        let Some(meta) = world.client_meta(id) else {
            continue;
        };
        perf::player_tick(
            id.0,
            meta.name == bot_name,
            time_ms,
            level_time_ms,
            origin,
            vz,
            yaw,
            pitch,
            jump_time,
            world.old_buttons(id).unwrap_or(0),
            weaponstate,
            meta.ammo_clip,
            world.pmove_walking(id),
            legs_anim,
            torso_anim,
            lifecycle_label(meta.lifecycle),
        );
    }
}

fn lifecycle_label(life: ClientLifecycle) -> &'static str {
    match life {
        ClientLifecycle::Connecting => "Connecting",
        ClientLifecycle::ChoosingClass => "ChoosingClass",
        ClientLifecycle::SpawnPending => "SpawnPending",
        ClientLifecycle::Alive => "Alive",
        ClientLifecycle::Dead => "Dead",
        ClientLifecycle::RespawnPending => "RespawnPending",
        ClientLifecycle::Spectating => "Spectating",
        ClientLifecycle::Intermission => "Intermission",
    }
}

fn emit_snapshot_events(_world: &FrameWorld, snapshot: &Snapshot) {
    for slot in &snapshot.meta.corpses.slots {
        perf::corpse(
            i64::from(slot.occupied),
            slot.occupied.then_some(slot.origin[2]),
            None,
            None,
        );
    }
    for es in snapshot
        .meta
        .entities
        .iter()
        .filter(|es| es.e_type == entity_iw4::ET_ITEM)
    {
        let ammo = snapshot
            .meta
            .item_ammo
            .iter()
            .find(|row| row.entnum == es.number);
        perf::item(
            es.e_type,
            ammo.map(|row| row.clip_r),
            ammo.map(|row| row.scavenger),
            None,
        );
    }
}

/// IW4 weapon class numbering (T6 classes are mapped onto it at load).
const WEAPCLASS_SNIPER: i32 = 1;

fn harvest_predictable_events(world: &mut FrameWorld, tick: Tick) {
    let mut falls = Vec::new();
    world.for_each_player_mut(|id, ps| {
        let mut cursor = ps.old_event_sequence;
        consume_player_events(ps, &mut cursor, |e| {
            if entity_iw4::EntityEventKind(e.event).is_landing_pain() {
                falls.push((id, e.event_parm, ps.max_health, ps.origin));
            }
        });
        ps.old_event_sequence = cursor;
    });
    for (id, parm, max_health, origin) in falls {
        apply_fall_damage(world, tick, id, parm, max_health, origin);
    }
}

/// BO2 feel M12: a landing that hurts (only Black Ops movement sends one)
/// costs parm% of max health, or 110% from a full-height fall, worked out in
/// floats and rounded down (to be checked in real BO2); BO2's own scripts take it as
/// MOD_FALLING through CodeCallback_PlayerDamage.
fn fall_damage(parm: i32, max_health: i32) -> i32 {
    let fraction = if parm < 100 { parm as f32 * 0.01 } else { 1.1 };
    (max_health as f32 * fraction) as i32
}

fn apply_fall_damage(
    world: &mut FrameWorld,
    tick: Tick,
    id: ClientId,
    parm: i32,
    max_health: i32,
    origin: [f32; 3],
) {
    if !world
        .client_meta(id)
        .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
    {
        return;
    }
    let amount = fall_damage(parm, max_health);
    if amount <= 0 {
        return;
    }
    let hit = crate::script_player::Hit {
        victim: id,
        attacker: None,
        amount,
        flags: 0,
        means: "MOD_FALLING",
        weapon: 0,
        point: origin,
        dir: [0.0; 3],
        hitloc: 0,
        inflictor: None,
        commit: None,
    };
    crate::script::player_damage(world.ecs(), tick, &hit);
}

fn emit_attack_events(
    world: &mut FrameWorld,
    tick: Tick,
    cmds: &[(ClientId, playerstate_iw4::UserCmd)],
) {
    for (id, cmd) in cmds {
        if !world
            .client_meta(*id)
            .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
        {
            continue;
        }
        let old = world
            .old_buttons_mut()
            .iter()
            .find(|(c, _)| c == id)
            .map_or(0, |(_, b)| *b);
        let attack = cmd.buttons & buttons::ATTACK != 0;
        let was = old & buttons::ATTACK != 0;
        if !attack && was {
            world.push_event(tick, EventAudience::Client(*id), SimEvent::AttackReleased);
        }
    }
}

fn action_needs_player_row(action: &ClientAction) -> bool {
    !matches!(
        action,
        ClientAction::JoinMatch { .. }
            | ClientAction::LeaveMatch { .. }
            | ClientAction::SetName { .. }
            | ClientAction::UseCopycat { .. }
            | ClientAction::ActionSlot { .. }
            | ClientAction::ChooseDefaultClass { .. }
            | ClientAction::MenuResponse { .. }
    )
}

fn apply_actions(world: &mut FrameWorld, tick: Tick, actions: &[(ClientId, ClientAction)]) {
    for (id, action) in actions {
        match *action {
            ClientAction::JoinMatch { request_id: _ } => {
                let meta = world.client_meta_mut(*id);
                if meta.lifecycle == ClientLifecycle::Connecting {
                    meta.lifecycle = ClientLifecycle::ChoosingClass;
                }
            }
            ClientAction::ChooseDefaultClass {
                request_id: _,
                index,
            } => {
                crate::script::answer_join(world.ecs(), id.0);
                choose_bot_class(world, *id, index);
            }
            ClientAction::MenuResponse {
                request_id: _,
                menu,
                response,
            } => {
                let menu = crate::menu_response_text(&menu);
                let response = crate::menu_response_text(&response);
                // A BO2 map's menus answer its own scripts.
                if crate::t6::menu_response(world.ecs(), id.0, menu, response) {
                    continue;
                }
                crate::script::note_team_answer(world.ecs(), id.0, menu);
                if !answer_custom_class(world, *id, menu, response) {
                    crate::script::answer_menu(world.ecs(), id.0, menu, response);
                }
            }
            ClientAction::LeaveMatch { request_id: _ } => {
                world.retire_client(*id);
            }
            ClientAction::SetName {
                request_id: _,
                name,
            } => {
                world.client_meta_mut(*id).name = name;
            }
            ClientAction::UseCopycat { .. } | ClientAction::SpawnClient { .. } => {}
            ClientAction::ActionSlot {
                request_id: _,
                slot,
            } => {
                crate::script::action_slot_command(world.ecs(), id.0, slot);
                crate::t6::action_slot_pressed(world.ecs(), id.0, slot);
            }
            ClientAction::SelectClass {
                request_id,
                class_id,
                revision,
                loadout,
            } => {
                apply_select_class(world, tick, *id, request_id, class_id, revision, loadout);
            }
            ClientAction::GiveWeapon { request_id, weapon } => {
                apply_give_weapon(world, tick, *id, request_id, weapon);
            }
            ClientAction::ChangeWeaponConfiguration {
                request_id,
                from,
                to,
            } => {
                apply_configuration_change(world, tick, *id, request_id, from, to);
            }
            ClientAction::ForceSpawn {
                request_id: _,
                pick,
            } => {
                if !world.bootstrap_ref().allow_debug_actions {
                    continue;
                }
                apply_force_spawn(world, *id, pick);
            }
            ClientAction::SpawnIntermission { request_id: _ } => {
                apply_spawn_intermission(world, *id);
            }
            ClientAction::GiveKillstreak {
                request_id: _,
                name,
            } => {
                if !world.bootstrap_ref().allow_debug_actions {
                    continue;
                }
                let name = crate::menu_response_text(&name);
                if !crate::t6::give_killstreak(world.ecs(), id.0, name) {
                    crate::script::give_killstreak(world.ecs(), id.0, name);
                }
            }
            ClientAction::ForceDeath { request_id: _ } => {
                if !world.bootstrap_ref().allow_debug_actions {
                    continue;
                }
                crate::script::force_death(world.ecs(), tick, id.0);
            }
            ClientAction::SetMatchPhase {
                request_id: _,
                phase,
            } => {
                if !world.bootstrap_ref().allow_debug_actions {
                    continue;
                }
                if phase == MatchPhase::Playing && world.phase() == MatchPhase::Warmup {
                    if !crate::script::host::iw4_gametype::force_match_start(world.ecs(), tick) {
                        crate::score::finish_prematch(world);
                    }
                } else {
                    world.set_phase(phase);
                }
            }
            ClientAction::Move {
                request_id: _,
                origin,
                angles,
            } => {
                apply_debug_move(world, *id, origin, angles);
            }
            ClientAction::BeginScriptMoverRotateVelocity {
                request_id: _,
                speed,
            } => {
                if !world.bootstrap_ref().allow_debug_actions {
                    continue;
                }
                world.begin_script_movers_rotate_velocity_supplied(
                    speed,
                    crate::corpse::level_time_ms(tick),
                );
            }
            ClientAction::DebugDamage {
                request_id: _,
                amount,
            } => {
                apply_debug_damage(world, tick, *id, amount);
            }
        }
    }
}

fn apply_force_spawn(world: &mut FrameWorld, id: ClientId, pick: crate::SpawnPick) {
    if world
        .client_meta(id)
        .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
    {
        crate::script_player::move_to_forced_spawn(world, id, pick);
    } else {
        world.client_meta_mut(id).forced_spawn = Some(pick);
    }
}

fn apply_spawn_intermission(world: &mut FrameWorld, id: ClientId) {
    if world
        .client_meta(id)
        .is_none_or(|m| m.lifecycle == ClientLifecycle::Intermission)
    {
        return;
    }
    let view = world.bootstrap_ref().intermission_view.clone();
    {
        let meta = world.client_meta_mut(id);
        meta.lifecycle = ClientLifecycle::Intermission;
        meta.dead_since_tick = None;
    }
    world.unlink_player_area(id);
    let Some(view) = view else {
        return;
    };
    let Some(ps) = world.player_mut(id) else {
        return;
    };
    ps.origin = view.origin;
    ps.velocity = [0.0, 0.0, 0.0];
    ps.viewangles = view.angles;
    ps.delta_angles = packed_look_delta(view.angles);
    ps.e_flags ^= playerstate_iw4::eflags::TELEPORT;
}

fn apply_debug_move(world: &mut FrameWorld, id: ClientId, origin: [f32; 3], angles: [f32; 3]) {
    if !world.bootstrap_ref().allow_debug_actions {
        return;
    }
    if !world
        .client_meta(id)
        .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
    {
        return;
    }
    if !origin.iter().chain(angles.iter()).all(|v| v.is_finite()) {
        return;
    }
    let old_origin = {
        let Some(ps) = world.player_mut(id) else {
            return;
        };
        let old_origin = ps.origin;
        ps.origin = origin;
        ps.velocity = [0.0, 0.0, 0.0];
        ps.viewangles = angles;

        ps.e_flags ^= playerstate_iw4::eflags::TELEPORT;
        ps.delta_angles = packed_look_delta(angles);
        old_origin
    };
    world.translate_player_area(
        id,
        [
            origin[0] - old_origin[0],
            origin[1] - old_origin[1],
            origin[2] - old_origin[2],
        ],
    );
}

fn packed_look_delta(viewangles: [f32; 3]) -> [f32; 3] {
    [
        viewangles[0] - (viewangles[0] * ANGLE2SHORT) as i32 as f32 * SHORT2ANGLE,
        viewangles[1] - (viewangles[1] * ANGLE2SHORT) as i32 as f32 * SHORT2ANGLE,
        viewangles[2] - (viewangles[2] * ANGLE2SHORT) as i32 as f32 * SHORT2ANGLE,
    ]
}

fn restamp_debug_move_look(world: &mut FrameWorld, actions: &[(ClientId, ClientAction)]) {
    for (id, action) in actions {
        let ClientAction::Move { angles, .. } = *action else {
            continue;
        };
        if !world.bootstrap_ref().allow_debug_actions {
            continue;
        }
        if !world
            .client_meta(*id)
            .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
        {
            continue;
        }
        let Some(ps) = world.player_mut(*id) else {
            continue;
        };
        ps.viewangles = angles;
        ps.delta_angles = packed_look_delta(angles);
    }
}

fn apply_debug_damage(world: &mut FrameWorld, tick: Tick, id: ClientId, amount: i32) {
    if !world.bootstrap_ref().allow_debug_actions {
        return;
    }
    if amount <= 0 {
        return;
    }
    if !world
        .client_meta(id)
        .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
    {
        return;
    }
    crate::script_player::debug_damage(world, tick, id, amount);
}

fn configuration_change_ammo(
    clip0: i32,
    clip1: i32,
    stock: i32,
    clip_capacity: i32,
    max_ammo: i32,
    dual: bool,
) -> (i32, i32, i32) {
    let total = clip0
        .max(0)
        .saturating_add(clip1.max(0))
        .saturating_add(stock.max(0));
    let cap = clip_capacity.max(0);
    let first = clip0.max(0).min(cap).min(total);
    let second = if dual {
        clip1.max(0).min(cap).min(total - first)
    } else {
        0
    };
    (first, second, (total - first - second).min(max_ammo.max(0)))
}

fn apply_configuration_change(
    world: &mut FrameWorld,
    tick: Tick,
    id: ClientId,
    request_id: u32,
    from: u32,
    to: u32,
) {
    let reject = |world: &mut FrameWorld, reason: crate::ConfigurationChangeRejectReason| {
        world.push_event(
            tick,
            EventAudience::Client(id),
            SimEvent::ConfigurationChangeRejected {
                request_id,
                from,
                to,
                reason,
            },
        );
    };
    use crate::ConfigurationChangeRejectReason as Reason;
    if !world
        .client_meta(id)
        .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
    {
        reject(world, Reason::NotAlive);
        return;
    }
    let Some(ps) = world.player(id).copied() else {
        reject(world, Reason::NotAlive);
        return;
    };
    if ps.weapon != from {
        reject(world, Reason::StaleSource);
        return;
    }
    if !world.can_transition_weapon(from, to) {
        reject(world, Reason::DifferentFamily);
        return;
    }
    let (Some(old), Some(new)) = (world.combat_facts_for(from), world.combat_facts_for(to)) else {
        reject(world, Reason::InvalidTarget);
        return;
    };
    if world
        .equipment_facts_for(to)
        .is_some_and(|eq| eq.is_offhand())
    {
        reject(world, Reason::InvalidTarget);
        return;
    }
    if ps.weaponstate_primary != weapon_iw4::WeaponState::Ready as i32
        || ps.weapon_time > 0
        || (ps.last_weapon_hand >= 1
            && (ps.weaponstate_secondary != weapon_iw4::WeaponState::Ready as i32
                || ps.weapon_time_secondary > 0))
        || ps.weap_flags & playerstate_iw4::weap_flags::OFFHAND_VIEW != 0
    {
        reject(world, Reason::Busy);
        return;
    }
    let Some(slot) = ps.weapons.iter().position(|&weapon| weapon == from as i32) else {
        reject(world, Reason::NoInventorySlot);
        return;
    };
    if ps.weapons.contains(&(to as i32)) {
        reject(world, Reason::InvalidTarget);
        return;
    }
    let old_ammo_key = weapon_iw4::ammo_table_key(old.ammo_index, from);
    let old_clip_key = weapon_iw4::clip_table_key(old.clip_index, from);
    let new_ammo_key = weapon_iw4::ammo_table_key(new.ammo_index, to);
    let new_clip_key = weapon_iw4::clip_table_key(new.clip_index, to);
    let another_owns = |key: i32, clip: bool| {
        ps.weapons
            .iter()
            .filter(|&&weapon| weapon > 0 && weapon != from as i32)
            .any(|&weapon| {
                world.combat_facts_for(weapon as u32).is_some_and(|facts| {
                    key == if clip {
                        weapon_iw4::clip_table_key(facts.clip_index, weapon as u32)
                    } else {
                        weapon_iw4::ammo_table_key(facts.ammo_index, weapon as u32)
                    }
                })
            })
    };
    let clip0 = weapon_iw4::get_clip_for_hand(&ps.ammoclip, old_clip_key, 0);
    let raw_clip1 = weapon_iw4::get_clip_for_hand(&ps.ammoclip, old_clip_key, 1);
    let clip1 = if ps.last_weapon_hand >= 1 {
        raw_clip1
    } else {
        0
    };
    let stock = weapon_iw4::get_ammo_not_in_clip(&ps.ammo, old_ammo_key);
    let akimbo = gsc_give_weapon_is_akimbo(world.weapon_script_name(to));
    let (next_clip0, next_clip1, next_stock) =
        configuration_change_ammo(clip0, clip1, stock, new.clip_size, new.max_ammo, akimbo);
    if (old_ammo_key != new_ammo_key
        && (another_owns(old_ammo_key, false) || another_owns(new_ammo_key, false)))
        || (old_clip_key != new_clip_key
            && (another_owns(old_clip_key, true) || another_owns(new_clip_key, true)))
        || (old_ammo_key == new_ammo_key
            && next_stock != stock
            && another_owns(old_ammo_key, false))
        || (old_clip_key == new_clip_key
            && (next_clip0 != clip0 || next_clip1 != raw_clip1)
            && another_owns(old_clip_key, true))
    {
        reject(world, Reason::SharedAmmoConflict);
        return;
    }
    let mut next = ps;
    next.weapons[slot] = to as i32;
    next.weapon_data[slot * 5..slot * 5 + 5].fill(0);
    weapon_iw4::latch_weapon_dual_wield(&next.weapons, &mut next.weapon_data, to, akimbo);
    next.weapon = to;
    next.weapon_primary = to;
    next.last_weapon_hand = weapon_iw4::num_hands_for_held(&next.weapons, &next.weapon_data, to);
    if old_ammo_key != new_ammo_key {
        for row in next.ammo.chunks_exact_mut(8) {
            if row[..4] == old_ammo_key.to_le_bytes() {
                row.fill(0);
            }
        }
    }
    if old_clip_key != new_clip_key {
        for row in next.ammoclip.chunks_exact_mut(12) {
            if row[..4] == old_clip_key.to_le_bytes() {
                row.fill(0);
            }
        }
    }
    if !weapon_iw4::set_ammo_not_in_clip(&mut next.ammo, new_ammo_key, next_stock)
        || !weapon_iw4::set_clip_for_hand(&mut next.ammoclip, new_clip_key, 0, next_clip0)
        || !weapon_iw4::set_clip_for_hand(&mut next.ammoclip, new_clip_key, 1, next_clip1)
    {
        reject(world, Reason::AmmoTableFull);
        return;
    }
    let alternate_ammo = match (
        world.combat_facts_for(old.alternate_weapon),
        world.combat_facts_for(new.alternate_weapon),
    ) {
        (Some(old_alt), Some(new_alt))
            if old.alternate_weapon != 0
                && new.alternate_weapon != 0
                && old_alt.weap_type == new_alt.weap_type
                && old_alt.weap_class == new_alt.weap_class =>
        {
            let old_key = weapon_iw4::ammo_table_key(old_alt.ammo_index, old.alternate_weapon);
            let old_clip = weapon_iw4::clip_table_key(old_alt.clip_index, old.alternate_weapon);
            let new_key = weapon_iw4::ammo_table_key(new_alt.ammo_index, new.alternate_weapon);
            let new_clip = weapon_iw4::clip_table_key(new_alt.clip_index, new.alternate_weapon);
            if old_key != old_ammo_key
                && old_clip != old_clip_key
                && new_key != new_ammo_key
                && new_clip != new_clip_key
                && weapon_iw4::ammo_row_present(&ps.ammo, old_key)
                && weapon_iw4::clip_row_present(&ps.ammoclip, old_clip)
            {
                let shared = ps
                    .weapons
                    .iter()
                    .filter(|&&w| w > 0 && w != from as i32)
                    .any(|&w| {
                        let alt = world
                            .combat_facts_for(w as u32)
                            .map_or(0, |f| f.alternate_weapon);
                        [w as u32, alt].into_iter().filter(|&w| w != 0).any(|w| {
                            world.combat_facts_for(w).is_some_and(|f| {
                                let ammo = weapon_iw4::ammo_table_key(f.ammo_index, w);
                                let clip = weapon_iw4::clip_table_key(f.clip_index, w);
                                (old_key != new_key && (ammo == old_key || ammo == new_key))
                                    || (old_clip != new_clip
                                        && (clip == old_clip || clip == new_clip))
                            })
                        })
                    });
                if shared {
                    reject(world, Reason::SharedAmmoConflict);
                    return;
                }
                let (clip, _, stock) = configuration_change_ammo(
                    weapon_iw4::get_clip_for_hand(&ps.ammoclip, old_clip, 0),
                    0,
                    weapon_iw4::get_ammo_not_in_clip(&ps.ammo, old_key),
                    new_alt.clip_size,
                    new_alt.max_ammo,
                    false,
                );
                if old_key != new_key {
                    for row in next.ammo.chunks_exact_mut(8) {
                        if row[..4] == old_key.to_le_bytes() {
                            row.fill(0);
                        }
                    }
                }
                if old_clip != new_clip {
                    for row in next.ammoclip.chunks_exact_mut(12) {
                        if row[..4] == old_clip.to_le_bytes() {
                            row.fill(0);
                        }
                    }
                }
                if !weapon_iw4::set_ammo_not_in_clip(&mut next.ammo, new_key, stock)
                    || !weapon_iw4::set_clip_for_hand(&mut next.ammoclip, new_clip, 0, clip)
                {
                    reject(world, Reason::AmmoTableFull);
                    return;
                }
                Some((new.alternate_weapon, clip, stock))
            } else {
                None
            }
        }
        _ => None,
    };
    let hand = weapon_iw4::spawn_weapon_hand(to, &new);
    next.weaponstate_primary = hand.weaponstate;
    next.weapon_time = hand.weapon_time;
    next.weapon_delay = hand.weapon_delay;
    next.weap_anim = hand.weap_anim;
    next.weaponstate_secondary = hand.weaponstate;
    next.weapon_time_secondary = hand.weapon_time;
    next.weapon_delay_secondary = hand.weapon_delay;
    next.weap_anim_secondary = hand.weap_anim;
    next.weapon_shot_count = 0;
    next.weapon_shot_count_secondary = 0;
    next.weap_flags &= !(playerstate_iw4::weap_flags::NO_ADS
        | playerstate_iw4::weap_flags::DOUBLEBARREL_RECOIL
        | playerstate_iw4::weap_flags::RECOIL_SCALE);
    *world.player_mut(id).expect("validated alive player") = next;
    let meta = world.client_meta_mut(id);
    if let Some((alternate, clip, stock)) = alternate_ammo {
        meta.ammo_by_weapon
            .retain(|row| row.0 != old.alternate_weapon);
        meta.set_ammo(alternate, clip, stock);
    }
    let quick_reload_ready = meta.quick_reload_ready(from);
    meta.ammo_by_weapon.retain(|row| row.0 != from);
    meta.set_ammo(to, next_clip0, next_stock);
    meta.set_quick_reload_ready(from, true);
    meta.set_quick_reload_ready(to, quick_reload_ready);
    meta.mirror_held_ammo(to);
    meta.weapon_shot_count = 0;
    meta.burst_latch = false;
    meta.burst_latch_secondary = false;
    meta.rechamber_pending = false;
    meta.rechamber_pending_secondary = false;
    world.push_event(
        tick,
        EventAudience::Client(id),
        SimEvent::ConfigurationChangeAccepted {
            request_id,
            from,
            to,
        },
    );
}

fn apply_give_weapon(
    world: &mut FrameWorld,
    tick: Tick,
    id: ClientId,
    request_id: u32,
    weapon: u32,
) {
    let reject = |world: &mut FrameWorld, reason: crate::GiveRejectReason| {
        world.push_event(
            tick,
            EventAudience::Client(id),
            SimEvent::GiveRejected {
                request_id,
                weapon,
                reason,
            },
        );
    };

    if !world
        .client_meta(id)
        .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
    {
        reject(world, crate::GiveRejectReason::NotAlive);
        return;
    }
    if weapon == 0 {
        reject(world, crate::GiveRejectReason::InvalidWeapon);
        return;
    }
    let table_len = world.weapon_combat_len();
    if (weapon as usize) >= table_len {
        reject(world, crate::GiveRejectReason::UnknownWeaponId);
        return;
    }
    if !world.weapon_runnable(weapon) {
        reject(world, crate::GiveRejectReason::UnsupportedWeapon);
        return;
    }
    let Some(facts) = world.combat_facts_for(weapon) else {
        reject(world, crate::GiveRejectReason::UnknownWeaponId);
        return;
    };
    if world
        .equipment_facts_for(weapon)
        .is_some_and(|eq| eq.is_offhand())
    {
        apply_give_offhand(world, tick, id, request_id, weapon, &facts);
        return;
    }
    if facts.fire_time_ms <= 0 && facts.raise_time_ms <= 0 {
        reject(world, crate::GiveRejectReason::EmptyCombatProfile);
        return;
    }

    if world.weapon_script_name(weapon).starts_with("killstreak_") {
        let Some(ps) = world.player_mut(id) else {
            reject(world, crate::GiveRejectReason::NotAlive);
            return;
        };
        inventory_add_weapon(ps, weapon, false);
        if !ps.weapons.contains(&(weapon as i32)) {
            reject(world, crate::GiveRejectReason::InvalidWeapon);
            return;
        }
        let (clip, clip_alt, stock) = weapon_iw4::spawn_clip_stock(&facts, 0);
        seed_ps_ammo_tables(ps, weapon, &facts, clip, clip_alt, false, stock);
        world.client_meta_mut(id).set_ammo(weapon, clip, stock);
        world.push_event(
            tick,
            EventAudience::Client(id),
            SimEvent::GiveAccepted { request_id, weapon },
        );
        return;
    }
    let akimbo = gsc_give_weapon_is_akimbo(world.weapon_script_name(weapon));
    let Some(mut next) = world.player(id).copied() else {
        reject(world, crate::GiveRejectReason::NotAlive);
        return;
    };

    let outgoing = if world
        .combat_facts_for(next.weapon)
        .is_some_and(|facts| facts.inventory_type == 3)
        && next.weapons.contains(&(next.weapon_primary as i32))
    {
        next.weapon_primary
    } else {
        next.weapon
    };
    let mut replaced = false;
    if !next.weapons.contains(&(weapon as i32)) {
        if let Some(slot) = next
            .weapons
            .iter()
            .position(|&w| w != 0 && w == outgoing as i32)
        {
            next.weapons[slot] = weapon as i32;
            next.weapon_data[slot * 5..slot * 5 + 5].fill(0);
            replaced = true;
        }
    }
    if replaced {
        let outgoing_facts = world
            .combat_facts_for(outgoing)
            .expect("held weapon combat facts");

        for (table, stride, clip) in [
            (&mut next.ammo[..], 8, false),
            (&mut next.ammoclip[..], 12, true),
        ] {
            let key = if clip {
                weapon_iw4::clip_table_key(outgoing_facts.clip_index, outgoing)
            } else {
                weapon_iw4::ammo_table_key(outgoing_facts.ammo_index, outgoing)
            };
            let owned = next.weapons.iter().filter(|&&w| w > 0).any(|&w| {
                world.combat_facts_for(w as u32).is_some_and(|f| {
                    key == if clip {
                        weapon_iw4::clip_table_key(f.clip_index, w as u32)
                    } else {
                        weapon_iw4::ammo_table_key(f.ammo_index, w as u32)
                    }
                })
            });
            if !owned {
                for row in table.chunks_exact_mut(stride) {
                    if row[..4] == key.to_le_bytes() {
                        row.fill(0);
                    }
                }
            }
        }
    }
    give_weapon_to_ps_akimbo(&mut next, weapon, akimbo);
    *world.player_mut(id).expect("validated alive player") = next;
    let (clip, stock) = if let Some(ps_mut) = world.player_mut(id) {
        arm_held_weapon(ps_mut, weapon, &facts)
    } else {
        (0, 0)
    };
    let meta = world.client_meta_mut(id);
    if replaced {
        meta.ammo_by_weapon.retain(|row| row.0 != outgoing);
        meta.set_quick_reload_ready(outgoing, true);
    }
    meta.set_ammo(weapon, clip, stock);
    meta.set_quick_reload_ready(weapon, true);
    meta.mirror_held_ammo(weapon);
    meta.weapon_shot_count = 0;
    meta.burst_latch = false;
    meta.burst_latch_secondary = false;
    meta.rechamber_pending = false;
    meta.rechamber_pending_secondary = false;
    world.push_event(
        tick,
        EventAudience::Client(id),
        SimEvent::GiveAccepted { request_id, weapon },
    );
}

fn apply_give_offhand(
    world: &mut FrameWorld,
    tick: Tick,
    id: ClientId,
    request_id: u32,
    weapon: u32,
    facts: &weapon_iw4::WeaponCombatFacts,
) {
    let reject = |world: &mut FrameWorld, reason: crate::GiveRejectReason| {
        world.push_event(
            tick,
            EventAudience::Client(id),
            SimEvent::GiveRejected {
                request_id,
                weapon,
                reason,
            },
        );
    };
    let Some(eq) = world.equipment_facts_for(weapon) else {
        reject(world, crate::GiveRejectReason::InvalidWeapon);
        return;
    };
    let Some(mut next) = world.player(id).copied() else {
        reject(world, crate::GiveRejectReason::NotAlive);
        return;
    };
    for slot in &mut next.weapons {
        if *slot > 0 && *slot != weapon as i32 {
            let same_slot = world
                .equipment_facts_for(*slot as u32)
                .is_some_and(|other| {
                    (matches!(eq.offhand_class, 1 | 4 | 5)
                        && matches!(other.offhand_class, 1 | 4 | 5))
                        || (matches!(eq.offhand_class, 2 | 3)
                            && matches!(other.offhand_class, 2 | 3))
                });
            if same_slot {
                *slot = 0;
            }
        }
    }
    inventory_add_weapon(&mut next, weapon, false);
    if !next.weapons.contains(&(weapon as i32)) {
        reject(world, crate::GiveRejectReason::InvalidWeapon);
        return;
    }
    match eq.offhand_class {
        1 | 4 | 5 => next.offhand_primary = eq.offhand_class,
        2 | 3 => next.offhand_secondary = eq.offhand_class,
        _ => {}
    }
    let clip = eq.spawn_clip_count();
    seed_ps_ammo_tables(&mut next, weapon, facts, clip, 0, false, 0);
    *world.player_mut(id).expect("validated alive player") = next;
    {
        let meta = world.client_meta_mut(id);
        meta.ammo_by_weapon
            .retain(|(id, _, _)| next.weapons.contains(&(*id as i32)));
        meta.set_ammo(weapon, clip, 0);
        if let Some(loadout) = meta.loadout.as_mut() {
            match eq.offhand_class {
                1 | 4 | 5 => loadout.lethal = weapon,
                2 | 3 => loadout.tactical = weapon,
                _ => {}
            }
        }
    }
    world.push_event(
        tick,
        EventAudience::Client(id),
        SimEvent::GiveAccepted { request_id, weapon },
    );
}

fn choose_bot_class(world: &mut FrameWorld, id: ClientId, index: u8) {
    let t5 = crate::script::is_t5(world.ecs());
    let bot_classes = &world.bootstrap_ref().bot_classes;
    let def = (!bot_classes.is_empty() && !t5)
        .then(|| bot_classes[index as usize % bot_classes.len()].clone())
        .filter(|def| !def.locked && validate_class_content(world, def).is_ok());
    match def {
        Some(def) => crate::script::choose_class(world.ecs(), id.0, &def),
        None => crate::script::choose_default_class(world.ecs(), id.0, index),
    }
}

fn answer_custom_class(world: &mut FrameWorld, id: ClientId, menu: &str, response: &str) -> bool {
    if !menu.eq_ignore_ascii_case("changeclass") {
        return false;
    }
    let Some(slot) = response
        .strip_prefix("custom")
        .and_then(|n| n.parse::<u32>().ok())
        .and_then(|n| n.checked_sub(1))
    else {
        return false;
    };
    let Some(def) = crate::script::personal_class(world.ecs(), id.0, crate::ClassId(slot)) else {
        return true;
    };
    if validate_class_content(world, &def).is_err() {
        return true;
    }
    crate::script::choose_class(world.ecs(), id.0, &def);
    true
}

fn apply_select_class(
    world: &mut FrameWorld,
    tick: Tick,
    id: ClientId,
    request_id: u32,
    class_id: crate::ClassId,
    revision: u32,
    loadout: crate::PersonalClass,
) {
    let accepted = (class_id.0 < crate::match_state::PERSONAL_CLASS_SLOTS as u32)
        .then(|| loadout.definition(class_id, revision))
        .flatten();
    let Some(def) = accepted else {
        diag::info!(
            Sim,
            "class select: rejected client={} id={} rev={} reason=unknown_or_stale_class",
            id.0,
            class_id.0,
            revision
        );
        world.push_event(
            tick,
            EventAudience::Client(id),
            SimEvent::ClassRejected {
                request_id,
                class_id,
                revision,
                reason: crate::ClassRejectReason::UnknownOrStaleClass,
            },
        );
        return;
    };

    if let Err(reason) = validate_class_content(world, &def) {
        diag::info!(
            Sim,
            "class select: rejected client={} id={} rev={} reason={}",
            id.0,
            class_id.0,
            revision,
            reason.as_str()
        );
        world.push_event(
            tick,
            EventAudience::Client(id),
            SimEvent::ClassRejected {
                request_id,
                class_id,
                revision,
                reason,
            },
        );
        return;
    }

    crate::script::answer_join(world.ecs(), id.0);
    crate::script::choose_class(world.ecs(), id.0, &def);
    world.push_event(
        tick,
        EventAudience::Client(id),
        SimEvent::ClassAccepted {
            request_id,
            class_id,
            revision,
        },
    );
}

fn validate_class_content(
    world: &FrameWorld,
    def: &crate::ClassDef,
) -> Result<(), crate::ClassRejectReason> {
    for (slot, perk) in def.perks.iter().copied().enumerate() {
        if perk != 0 && crate::match_state::perk_slot_from_class_catalog(perk) != Some(slot) {
            return Err(crate::ClassRejectReason::LockedContent);
        }
    }
    let max_attachments = if def.perks[0] == 12 { 2 } else { 1 };
    let table_len = world.weapon_combat_len();
    for id in [def.primary, def.secondary] {
        if id == 0 {
            continue;
        }
        if (id as usize) >= table_len {
            return Err(crate::ClassRejectReason::UnknownWeaponId);
        }
        let Some(row) = world.weapon_combat_row(id) else {
            return Err(crate::ClassRejectReason::UnknownWeaponId);
        };
        if !row.is_usable()
            || !world.weapon_runnable(id)
            || world
                .weapon_setup(id)
                .is_some_and(|setup| setup.attachments.len() > max_attachments)
        {
            return Err(crate::ClassRejectReason::LockedContent);
        }
    }
    for (slot, id) in [def.lethal, def.tactical].into_iter().enumerate() {
        if id == 0 {
            continue;
        }
        if (id as usize) >= table_len {
            return Err(crate::ClassRejectReason::UnknownWeaponId);
        }
        let Some(facts) = world.offhand_loadout_row(id) else {
            return Err(crate::ClassRejectReason::LockedContent);
        };
        if !world.weapon_runnable(id)
            || !matches!((slot, facts.offhand_class), (0, 1 | 4 | 5) | (1, 2 | 3))
        {
            return Err(crate::ClassRejectReason::LockedContent);
        }
    }

    if def.primary_attachments.iter().any(|&a| a != 0)
        || def.secondary_attachments.iter().any(|&a| a != 0)
    {
        return Err(crate::ClassRejectReason::LockedContent);
    }

    Ok(())
}

pub(crate) fn log_forced_spawn(
    id: ClientId,
    pick: crate::SpawnPick,
    report: &crate::spawn::SpawnAttemptReport,
) {
    let asked = match pick {
        crate::SpawnPick::Seeded(seed) => format!("seed={seed}"),
        crate::SpawnPick::At { origin, yaw } => format!(
            "at=[{:.1}, {:.1}, {:.1}] yaw={yaw:.1}",
            origin[0], origin[1], origin[2]
        ),
    };
    let fallback = report
        .rejected
        .first()
        .filter(|(source, _)| *source == crate::spawn::FORCED_SPAWN_SOURCE)
        .map(|(_, reason)| format!(" fallback={}", reason.as_str()))
        .unwrap_or_default();
    match &report.accepted {
        Some(d) => diag::info!(
            Sim,
            "spawn: forced client={} {asked} origin=[{:.1}, {:.1}, {:.1}] yaw={:.1} source={} tried={}{fallback}",
            id.0,
            d.traced_origin[0],
            d.traced_origin[1],
            d.traced_origin[2],
            d.raw_angles[1],
            if d.source_index == crate::spawn::FORCED_SPAWN_SOURCE {
                String::from("at")
            } else {
                d.source_index.to_string()
            },
            report.tried
        ),
        None => diag::info!(
            Sim,
            "spawn: forced client={} {asked} refused tried={}{fallback}",
            id.0,
            report.tried
        ),
    }
}

struct PlayerBodyClip {
    entnum: u16,
    origin: [f32; 3],
    /// bo2zm M3: an entity's own box (a script mover's packed `solid`: a
    /// Black Ops II zombie); None = a player's.
    bounds: Option<([f32; 3], [f32; 3])>,
}

fn alive_body_clips(world: &FrameWorld) -> Vec<PlayerBodyClip> {
    let mut bodies: Vec<_> = world
        .client_ids_sorted()
        .into_iter()
        .filter(|id| {
            world
                .client_meta(*id)
                .is_some_and(|m| m.lifecycle == ClientLifecycle::Alive)
        })
        .filter_map(|id| {
            world.player(id).map(|ps| PlayerBodyClip {
                entnum: id.0 as u16,
                origin: ps.origin,
                bounds: None,
            })
        })
        .collect();
    bodies.extend(
        world
            .prediction_remote_bodies()
            .iter()
            .map(|body| PlayerBodyClip {
                entnum: body.client.0 as u16,
                origin: body.origin,
                bounds: None,
            }),
    );
    // bo2zm M3: entities solid as a packed box block as that box, as IW4's
    // movement clips against entities' packed bounds (BO2's zombies body-
    // block: a crowd corners him). The same rows reach his client's
    // prediction in the snapshot.
    bodies.extend(
        world
            .packed_solid_movers()
            .into_iter()
            .map(|(entnum, origin, solid)| {
                let x = (solid & 0xff) as f32;
                let below = ((solid >> 8) & 0xff) as f32;
                let above = ((solid >> 16) & 0xff) as f32 - 32.0;
                PlayerBodyClip {
                    entnum,
                    origin,
                    bounds: Some(([-x, -x, -below], [x, x, above])),
                }
            }),
    );
    bodies
}

/// Does the box at `a` (`amins..amaxs`) overlap the box at `b`?
fn boxes_overlap(
    a: [f32; 3],
    amins: [f32; 3],
    amaxs: [f32; 3],
    b: [f32; 3],
    bmins: [f32; 3],
    bmaxs: [f32; 3],
) -> bool {
    (0..3).all(|i| a[i] + amins[i] < b[i] + bmaxs[i] && a[i] + amaxs[i] > b[i] + bmins[i])
}

#[derive(Clone, Copy)]
struct ClipBackend<'a> {
    brushes: &'a [SimBrush],
    bsp: &'a SimClipBsp,
    mesh: &'a SimClipMesh,
    glass_damage: &'a [(u32, u16)],
    bodies: &'a [PlayerBodyClip],
    self_entnum: u16,
    cmodels: &'a [clipmap_iw4::ClipCmodel],
    linked_brushes: &'a [LinkedBrushCollisionBrush],
    /// bo2zm: solid collision models as turned boxes (entity number, brush).
    blockers: &'a [(u16, SimBrush)],
    /// bo2zm: `forceslick` on him: every surface he touches is slick (SURF_SLICK 0x2).
    slick: bool,
}

/// bo2zm M3: the world's solid collision models (Nuketown's patch walls,
/// perk machines, the Mystery Box) as six-plane brushes.
fn oriented_blocker_brushes(world: &FrameWorld) -> Vec<(u16, SimBrush)> {
    world
        .oriented_blockers()
        .iter()
        .map(|b| {
            (
                b.entnum,
                SimBrush {
                    planes: b.planes().to_vec(),
                    contents: crate::bullet_collision::CONTENTS_SOLID,
                    plane_surface_flags: vec![b.surface_flags; 6],
                    glass_encoded: 0,
                },
            )
        })
        .collect()
}

/// bo2zm M3: the move clipped against the turned boxes too. One he is
/// already inside never holds him (he walks out of it), as with entities'
/// boxes.
fn clip_move_to_blockers(
    mut hit: Trace,
    blockers: &[(u16, SimBrush)],
    input: GroundTraceInput,
) -> Trace {
    for (entnum, brush) in blockers {
        let other = trace_iw4::trace_box(
            [clipmap_iw4::brush_ref(brush)],
            input.start,
            input.end,
            input.mins,
            input.maxs,
            input.tracemask,
        );
        if other.startsolid != 0 || other.allsolid != 0 || other.fraction >= hit.fraction {
            continue;
        }
        hit = Trace {
            hit_type: HITTYPE_ENTITY,
            hit_id: *entnum,
            ..other
        };
    }
    hit
}

/// The lower part of his box (prone height) at `origin`, moved the shortest way out of any
/// brush: up to 4 units sideways or 18 up. Unchanged when it is already clear, or when no
/// spot that close is. The movement code's own fix tries only 1 unit, and Ascension's lander
/// spawn spots overlap its centre pole by 1-2 units.
fn unstick_feet(backend: &ClipBackend<'_>, origin: [f32; 3]) -> [f32; 3] {
    let solid = |d: [f32; 3]| {
        let at = [origin[0] + d[0], origin[1] + d[1], origin[2] + d[2]];
        let tr = backend.trace(GroundTraceInput {
            start: at,
            end: at,
            mins: [-15.0, -15.0, 0.0],
            maxs: [15.0, 15.0, movement_iw4::PRONE_MAXS_Z],
            tracemask: crate::bullet_collision::MASK_PLAYER_SOLID,
        });
        tr.startsolid != 0 || tr.allsolid != 0
    };
    if !solid([0.0; 3]) {
        return origin;
    }
    let mut tries: Vec<[f32; 3]> = (-4..=4i8)
        .flat_map(|x| (-4..=4i8).map(move |y| [f32::from(x), f32::from(y), 0.0]))
        .chain((1..=18u8).map(|z| [0.0, 0.0, f32::from(z)]))
        .filter(|d| *d != [0.0; 3])
        .collect();
    tries.sort_by(|a, b| {
        let len = |d: &[f32; 3]| d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        len(a).total_cmp(&len(b))
    });
    tries
        .into_iter()
        .find(|&d| !solid(d))
        .map_or(origin, |d| [origin[0] + d[0], origin[1] + d[1], origin[2] + d[2]])
}

/// bo2zm test aid (IW4L_T6_UNSTICK_LOG or IW4L_T6_RIDELOG set): when `unstick_feet` moved him,
/// what his feet were inside: the map's own brushes, or which linked brush
/// model (its clip handle, origin and angles). At most 80 lines.
fn unstick_log(backend: &ClipBackend<'_>, from: [f32; 3], to: [f32; 3]) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    static LINES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("IW4L_T6_UNSTICK_LOG").is_some() || std::env::var_os("IW4L_T6_RIDELOG").is_some())
        || LINES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= 80
    {
        return;
    }
    let mins = [-15.0, -15.0, 0.0];
    let maxs = [15.0, 15.0, movement_iw4::PRONE_MAXS_Z];
    let mask = crate::bullet_collision::MASK_PLAYER_SOLID;
    let world_only = ClipBackend {
        bodies: &[],
        linked_brushes: &[],
        blockers: &[],
        ..*backend
    };
    let tr = world_only.trace(GroundTraceInput {
        start: from,
        end: from,
        mins,
        maxs,
        tracemask: mask,
    });
    let map = tr.startsolid != 0 || tr.allsolid != 0;
    let models: Vec<String> = backend
        .linked_brushes
        .iter()
        .filter(|b| {
            clipmap_iw4::clip_handle_to_model(backend.cmodels, b.cmodel_handle).is_some_and(|m| {
                let t = clipmap_iw4::transformed_capsule_trace(
                    m,
                    &backend.bsp.leafbrushes,
                    backend.brushes,
                    from,
                    from,
                    mins,
                    maxs,
                    b.origin,
                    b.angles,
                    mask,
                );
                t.startsolid != 0 || t.allsolid != 0
            })
        })
        .map(|b| {
            format!(
                " model {} at ({:.1} {:.1} {:.1}) angles ({:.1} {:.1} {:.1})",
                b.cmodel_handle, b.origin[0], b.origin[1], b.origin[2], b.angles[0], b.angles[1], b.angles[2]
            )
        })
        .collect();
    diag::info!(
        Sim,
        "bo2zm t6 unstick: ({:.2} {:.2} {:.2}) -> ({:.2} {:.2} {:.2}), inside map brushes {map} (contents {:#x}), linked models:{}",
        from[0],
        from[1],
        from[2],
        to[0],
        to[1],
        to[2],
        tr.contents,
        if models.is_empty() { " none".to_string() } else { models.concat() }
    );
}

/// bo2zm down test (IW4L_T6_DOWN set): while he is crouched or prone and not
/// in last stand, what fills the stand and crouch boxes at his feet, layer
/// by layer (after a revive the stand-up check found no room, 10-09). At
/// most 40 lines.
fn stance_room_trace(backend: &ClipBackend<'_>, ps: &PlayerState) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    static LINES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("IW4L_T6_DOWN").is_some())
        || ps.pm_type != 0
        || ps.pm_flags & 3 == 0
        || LINES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= 40
    {
        return;
    }
    let world_only = ClipBackend {
        bodies: &[],
        cmodels: &[],
        linked_brushes: &[],
        blockers: &[],
        ..*backend
    };
    let no_bodies = ClipBackend {
        bodies: &[],
        ..*backend
    };
    let show = |t: Trace| format!("{}{} {:?}:{}", t.allsolid, t.startsolid, t.hit_type, t.hit_id);
    let parts: Vec<String> = [
        ("stand", movement_iw4::STAND_MAXS_Z),
        ("crouch", movement_iw4::CROUCH_MAXS_Z),
    ]
    .into_iter()
    .map(|(name, top)| {
        let input = GroundTraceInput {
            start: ps.origin,
            end: ps.origin,
            mins: [-15.0, -15.0, 0.0],
            maxs: [15.0, 15.0, top],
            tracemask: 0x0281_0011,
        };
        format!(
            "{name} world {} +models {} +bodies {}",
            show(world_only.trace(input)),
            show(no_bodies.trace(input)),
            show(backend.trace(input))
        )
    })
    .collect();
    // Which world part fills the stand hull (brushes or the triangle
    // mesh), and from how high up it is clear.
    let no_mesh = crate::world::SimClipMesh::default();
    let brushes_only = ClipBackend {
        mesh: &no_mesh,
        ..world_only
    };
    let mesh_only = ClipBackend {
        brushes: &[],
        ..world_only
    };
    let stand_at = |b: &ClipBackend<'_>, up: f32| {
        let o = [ps.origin[0], ps.origin[1], ps.origin[2] + up];
        b.trace(GroundTraceInput {
            start: o,
            end: o,
            mins: [-15.0, -15.0, 0.0],
            maxs: [15.0, 15.0, movement_iw4::STAND_MAXS_Z],
            tracemask: 0x0281_0011,
        })
    };
    let lift: Vec<String> = [0.25_f32, 1.0, 4.0, 16.0]
        .into_iter()
        .map(|up| format!("+{up} {}", show(stand_at(&world_only, up))))
        .collect();
    let m = stand_at(&mesh_only, 0.0);
    diag::info!(
        Sim,
        "bo2zm t6 stance room at {:?} pm_flags {:#x} target {}: {} | stand brushes {} mesh {} (contents {:#x} surf {:#x}) | world lifted {}",
        ps.origin,
        ps.pm_flags,
        ps.view_height_target,
        parts.join(" | "),
        show(stand_at(&brushes_only, 0.0)),
        show(m),
        m.contents,
        m.surface_flags,
        lift.join(" ")
    );
}

impl CollisionBackend for ClipBackend<'_> {
    fn trace(&self, input: GroundTraceInput) -> Trace {
        let world_hit = clip_trace(
            self.brushes,
            self.bsp,
            self.mesh,
            input.start,
            input.end,
            input.mins,
            input.maxs,
            input.tracemask,
            &|piece| {
                let damage = self
                    .glass_damage
                    .iter()
                    .find(|(id, _)| *id == u32::from(piece))
                    .map_or(0, |(_, d)| *d);
                crate::world_objects::glass_piece_is_solid(damage)
            },
        );
        let with_bmodels = clip_move_to_bmodels(
            world_hit,
            self.cmodels,
            &self.bsp.leafbrushes,
            self.brushes,
            self.linked_brushes,
            input,
        );
        let with_bodies = clip_move_to_players(with_bmodels, self.bodies, self.self_entnum, input);
        let mut hit = clip_move_to_blockers(with_bodies, self.blockers, input);
        if self.slick {
            hit.surface_flags |= 2;
        }
        hit
    }
}

/// Diag: a move that started inside another player's body (the pair that
/// used to fuse for good). Logged sparingly: the first few, then one in 256.
fn note_body_overlap(
    self_entnum: u16,
    other: u16,
    start: [f32; 3],
    other_origin: [f32; 3],
    blocked_deeper: bool,
) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEEN: AtomicU64 = AtomicU64::new(0);
    let n = SEEN.fetch_add(1, Ordering::Relaxed);
    if n < 16 || n % 256 == 0 {
        let d = ((start[0] - other_origin[0]).powi(2) + (start[1] - other_origin[1]).powi(2)).sqrt();
        diag::info!(
            Sim,
            "player body overlap: client {self_entnum} inside {other} apart {d:.1} at [{:.0}, {:.0}, {:.0}] move {} (seen {})",
            start[0],
            start[1],
            start[2],
            if blocked_deeper { "deeper blocked" } else { "away allowed" },
            n + 1
        );
    }
}

fn clip_move_to_players(
    mut hit: Trace,
    bodies: &[PlayerBodyClip],
    self_entnum: u16,
    input: GroundTraceInput,
) -> Trace {
    // bo2zm: players and zombies only stop traces that ask for bodies: BO2's
    // stand-up (PM_CanStand), lie-down (0x810011) and mantle checks leave them out.
    if hit.fraction == 0.0 || input.tracemask & crate::world::CONTENTS_BODY == 0 {
        return hit;
    }
    for body in bodies {
        if body.entnum == self_entnum {
            continue;
        }
        if hit.fraction == 0.0 && hit.hit_type == HITTYPE_ENTITY && hit.contents == crate::world::CONTENTS_BODY {
            // Already stopped dead against a body he stands in.
            break;
        }
        let (body_mins, body_maxs) = body.bounds.unwrap_or((PLAYER_MINS, PLAYER_MAXS));
        // bo2zm M3: an entity's box never traps him: inside one (a zombie
        // stepped into him as he stepped into it) he may move away from its
        // centre, never deeper; deeper stops him on a plane facing away.
        if body.bounds.is_some()
            && boxes_overlap(
                input.start,
                input.mins,
                input.maxs,
                body.origin,
                body_mins,
                body_maxs,
            )
        {
            let flat =
                |p: [f32; 3]| (p[0] - body.origin[0]).powi(2) + (p[1] - body.origin[1]).powi(2);
            if flat(input.end) >= flat(input.start) {
                continue;
            }
            let d = [
                input.start[0] - body.origin[0],
                input.start[1] - body.origin[1],
            ];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            hit.fraction = 0.0;
            hit.endpos = input.start;
            hit.normal = if len > 1e-3 {
                [d[0] / len, d[1] / len, 0.0]
            } else {
                [1.0, 0.0, 0.0]
            };
            hit.contents = crate::world::CONTENTS_BODY;
            hit.hit_type = HITTYPE_ENTITY;
            hit.hit_id = body.entnum;
            hit.walkable = 0;
            return hit;
        }
        let other = clipmap_iw4::transformed_temp_capsule_trace(
            input.start,
            input.end,
            input.mins,
            input.maxs,
            body.origin,
            body_mins,
            body_maxs,
            crate::world::CONTENTS_BODY,
            input.tracemask,
        );
        if other.fraction >= hit.fraction && other.startsolid == 0 && other.allsolid == 0 {
            continue;
        }
        // A player he already stands in (a spawn on top of him, standing up
        // under him, a stale prediction body) never traps him: IW's
        // PM_playerTrace retraces a move that starts inside a body without
        // CONTENTS_BODY, and G_StuckInClient's ejection then pushes the
        // two apart. Ours keeps the BO2 body block one way: he may move
        // away from the other's centre (the ejection, a sidestep) but never
        // deeper; deeper stops him on a plane facing away so the slide
        // carries him round. Before this, the overlap traced allsolid, the
        // slide move froze both players, and two bots stayed fused forever.
        if body.bounds.is_none() && (other.startsolid != 0 || other.allsolid != 0) {
            let flat =
                |p: [f32; 3]| (p[0] - body.origin[0]).powi(2) + (p[1] - body.origin[1]).powi(2);
            if flat(input.end) >= flat(input.start) {
                note_body_overlap(self_entnum, body.entnum, input.start, body.origin, false);
                continue;
            }
            note_body_overlap(self_entnum, body.entnum, input.start, body.origin, true);
            let d = [
                input.start[0] - body.origin[0],
                input.start[1] - body.origin[1],
            ];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            // Same spot: split by entity number so the pair part, not tie.
            let away = if len > 1e-3 {
                [d[0] / len, d[1] / len, 0.0]
            } else if self_entnum < body.entnum {
                [-1.0, 0.0, 0.0]
            } else {
                [1.0, 0.0, 0.0]
            };
            if hit.fraction > 0.0 {
                hit.fraction = 0.0;
                hit.endpos = input.start;
                hit.normal = away;
                hit.contents = crate::world::CONTENTS_BODY;
                hit.hit_type = HITTYPE_ENTITY;
                hit.hit_id = body.entnum;
                hit.walkable = 0;
            }
            continue;
        }
        let startsolid = hit.startsolid | other.startsolid;
        let allsolid = hit.allsolid | other.allsolid;
        if other.fraction < hit.fraction {
            hit = other;
            hit.hit_type = HITTYPE_ENTITY;
            hit.hit_id = body.entnum;
        }
        hit.startsolid = startsolid;
        hit.allsolid = allsolid;
    }
    hit
}

/// bo2zm: the rules the game in this process last ran with. The local
/// player's own guess of their move (the prediction world) has no scripts of
/// its own, so it borrows these; without them it guessed with MW2 rules while
/// the game ran BO2's, and a dive jumped back on every correction.
static HOST_FEEL: std::sync::Mutex<Option<movement_iw4::Bo2Feel>> = std::sync::Mutex::new(None);

/// bo2zm: the Black Ops II movement rules, on in every game. Values the scripts set win over the defaults. `predicting` = this
/// is the local player's guess, not the game itself; `sent` = the rules the
/// game sent in its last snapshot.
fn bo2_feel(
    ecs: &World,
    predicting: bool,
    sent: Option<movement_iw4::Bo2Feel>,
) -> movement_iw4::Bo2Feel {
    // The Black Ops II scripts' own dvars (setdvar); none = no T6 game.
    // bo2mp: no Black Ops II scripts running: Black Ops II's own defaults
    // all the same (MW2's rules never apply).
    let defaults = || bo2_feel_with(&std::collections::HashMap::<String, String>::new(), omni_movement_setting());
    let Some(t6) = ecs.get_resource::<crate::t6::T6Runtime>() else {
        if predicting {
            // The game's own rules first: a second PC has no scripts, and
            // a game hosted here before may have left its rules behind.
            return sent
                .or_else(|| HOST_FEEL.lock().ok().and_then(|f| *f))
                .unwrap_or_else(defaults);
        }
        let feel = defaults();
        if let Ok(mut f) = HOST_FEEL.lock() {
            *f = Some(feel);
        }
        return feel;
    };
    let feel = bo2_feel_from(t6);
    if !predicting && let Ok(mut f) = HOST_FEEL.lock() {
        *f = Some(feel);
    }
    feel
}

/// bo2mp: the MW2 rule set is gone (`BO2_FEEL=mw2` no longer exists):
/// every game plays by Black Ops II's.
pub(crate) fn bo2_feel_mw2() -> bool {
    false
}

/// bo2zm: the Controls setting "Omni Movement" (off by
/// default, an option in the Controls menu for whoever wants it).
static OMNI_SETTING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// bo2zm: the "Omni Movement" setting; the next game started here reads it.
pub fn set_omni_movement(on: bool) {
    OMNI_SETTING.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// bo2zm: the "Omni Movement" setting as a game starting now takes it.
pub(crate) fn omni_movement_setting() -> bool {
    OMNI_SETTING.load(std::sync::atomic::Ordering::Relaxed)
}

/// A dvar's number by its name in any case (the lower-case key the VM
/// stores first, then the name as written).
fn dvar_number<S: std::hash::BuildHasher>(
    dvars: &std::collections::HashMap<String, String, S>,
    name: &str,
) -> Option<f32> {
    dvars
        .get(&name.to_ascii_lowercase())
        .or_else(|| dvars.get(name))
        .and_then(|v| v.trim().parse::<f32>().ok())
}

fn bo2_feel_from(t6: &crate::t6::T6Runtime) -> movement_iw4::Bo2Feel {
    bo2_feel_with(&t6.vm.dvars, t6.omni)
}

/// bo2mp: Black Ops II's rules from these dvars (none set: its defaults).
/// Every game runs by them: there is no MW2 rule set.
fn bo2_feel_with<S: std::hash::BuildHasher>(
    dvars: &std::collections::HashMap<String, String, S>,
    omni: bool,
) -> movement_iw4::Bo2Feel {
    // Dvar names are case-blind: the VM keeps what the scripts set (and the
    // engine defaults it seeds) under lower case (`Vm::dvar_name`), so a
    // look-up by the engine's mixed-case name never found them and every
    // script-set value here (jump_slowdownEnable, bg_fallDamage*,
    // perk_*Multiplier, player_sprintStrafeSpeedScale) fell back to the
    // default.
    let number = |name: &str, default: f32| dvar_number(dvars, name).unwrap_or(default);
    let zombies = dvars
        .get("mapname")
        .is_some_and(|m| m.starts_with("zm_") || crate::is_foreign_zm_map(m));
    movement_iw4::Bo2Feel {
        on: true,
        sprint_strafe_speed_scale: number("player_sprintStrafeSpeedScale", 0.667),
        // BO2 turns the repeated-jump slowdown off in Zombies and on in
        // multiplayer, unless a script sets the dvar.
        jump_slowdown: number("jump_slowdownEnable", if zombies { 0.0 } else { 1.0 }) != 0.0,
        // Both modes read the same fall heights.
        fall_damage_min_height: number("bg_fallDamageMinHeight", 128.0),
        fall_damage_max_height: number("bg_fallDamageMaxHeight", 300.0),
        zombies,
        gravity: number("bg_gravity", 800.0) as i32,
        // The fall perk works in both modes (Turned's zombies get it);
        // Flak Jacket only changes falls in Zombies, where it is PhD Flopper.
        fall_damage_perk: movement_iw4::PERK_FALLHEIGHT,
        flak_jacket_perk: movement_iw4::PERK_FLAKJACKET,
        fast_ads_perk: movement_iw4::PERK_FASTADS,
        fast_ads_multiplier: number("perk_weapAdsMultiplier", 0.5),
        // Stamin-Up: sprint time x perk_sprintMultiplier, and in Zombies a
        // sprint at least 1.1 times the gun's speed.
        longer_sprint_perk: movement_iw4::PERK_LONGERSPRINT,
        sprint_multiplier: number("perk_sprintMultiplier", 2.0),
        // Move-faster: the walk speed x perk_speedMultiplier (190 -> 203).
        move_faster_perk: movement_iw4::PERK_MOVEFASTER,
        speed_multiplier: number("perk_speedMultiplier", 1.07),
        g_speed: number("g_speed", 190.0),
        // dtp: BO2's dive to prone switch, on in both modes.
        dive: number("dtp", 1.0) != 0.0,
        // Set per player from the gun in hand.
        sprint_cycle_scale: 1.0,
        ducked_sprint_cycle_scale: 1.0,
        dtp_cycle_scale: 1.0,
        // BO2 sprints forward only (forwardmove above
        // player_sprintForwardMinimum); off by default, the
        // Controls menu's "Omni Movement" turns it on (read at game start).
        omni,
        // Low climbs keep the gun in hand (defaults until real BO2 is read).
        mantle_weapon_height: number("mantle_weapon_height", 39.0),
        mantle_weapon_anim_height: number("mantle_weapon_anim_height", 32.0),
    }
}

fn pmove_context(
    old_buttons: u32,
    weapon_scales: (f32, f32, f32),
    ads_allowed: bool,
    aim_down_sight: bool,
    ads_in_rate: f32,
    ads_out_rate: f32,
    rechamber_while_ads: bool,
    ads_reload: (bool, i32, bool),
    ads_fire_only: bool,
    melee_delay_ms: i32,
    melee_charge_delay_ms: i32,
    overlay_reticle: i32,
    shellshock_movement_scale: f32,
    feel: movement_iw4::Bo2Feel,
) -> PmoveSingleContext {
    let player_sprint_time = 4.0_f32;
    let weapon_max_sprint_time = get_max_sprint_time(weapon_scales.2, player_sprint_time);

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
                weapon_move_speed_scale: weapon_scales.0,
                weapon_ads_move_speed_scale: weapon_scales.1,

                shellshock_movement_scale,
                // BO2 feel M10b: going prone takes 600 ms (MW2 400).
                prone_lerp_ms: if feel.on {
                    movement_iw4::PRONE_LERP_MS
                } else {
                    0
                },
                // BO2 feel M10c: no 0.4 prone-aim slow; sprint speed only
                // while standing.
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
            weapon_max_sprint_time,
            sprint_forever: false,
            min_sprint_time_seconds: 1.0,
            sprint_delay_seconds: 0.0,

            sprint_forward_minimum: 105,
            stand_up_clear: true,
            sprint_recharge_pause_seconds: 0.0,
        },
        ads_intent: movement_iw4::AdsIntentContext {
            ads_allowed,
            weapon_def_scope: overlay_reticle != 0,
            sprint_hold_ads: false,
            bo2: feel.on,
        },
        ads_frac: movement_iw4::AdsFracContext {
            aim_down_sight,
            ads_in_rate,
            ads_out_rate,
            rechamber_while_ads,
            ads_fire_only,
            bo2: feel.on,
            reload_lowers_ads: ads_reload.0,
            position_reload_trans_ms: ads_reload.1,
            segmented_reload: ads_reload.2,
        },
        melee_charge: movement_iw4::MeleeChargeWeaponDelays {
            melee_delay_ms,
            melee_charge_delay_ms,
        },
        player_melee_range: movement_iw4::MELEE_CHARGE_PLAYER_MELEE_RANGE_DEFAULT,
        old_buttons,
        old_move: [0, 0],
        weapon_blocks_prone: false,
    }
}

pub(crate) fn seed_ps_ammo_tables(
    ps: &mut PlayerState,
    weapon: u32,
    facts: &weapon_iw4::WeaponCombatFacts,
    clip: i32,
    clip_alt: i32,
    dual: bool,
    stock: i32,
) {
    let ammo_index = weapon_iw4::ammo_table_key(facts.ammo_index, weapon);
    let clip_index = weapon_iw4::clip_table_key(facts.clip_index, weapon);
    if ammo_index != 0 {
        let _ = weapon_iw4::set_ammo_not_in_clip(&mut ps.ammo, ammo_index, stock);
    }
    if clip_index != 0 {
        let _ = weapon_iw4::set_clip_for_hand(&mut ps.ammoclip, clip_index, 0, clip);
        if dual {
            let _ = weapon_iw4::set_clip_for_hand(&mut ps.ammoclip, clip_index, 1, clip_alt);
        }
    }
}

pub(crate) fn arm_held_weapon(
    ps: &mut PlayerState,
    weapon: u32,
    facts: &weapon_iw4::WeaponCombatFacts,
) -> (i32, i32) {
    ps.weapon_primary = weapon;
    let last_hand = weapon_iw4::num_hands_for_held(&ps.weapons, &ps.weapon_data, weapon);
    ps.last_weapon_hand = last_hand;
    let (clip0, clip1, stock) = weapon_iw4::spawn_clip_stock(facts, last_hand);
    let hand = weapon_iw4::spawn_weapon_hand(weapon, facts);
    ps.weaponstate_primary = hand.weaponstate;
    ps.weapon_time = hand.weapon_time;
    ps.weapon_delay = hand.weapon_delay;
    ps.weap_anim = hand.weap_anim;
    if last_hand >= 1 {
        ps.weaponstate_secondary = hand.weaponstate;
        ps.weapon_time_secondary = hand.weapon_time;
        ps.weapon_delay_secondary = hand.weapon_delay;
        ps.weap_anim_secondary = hand.weap_anim;
    }
    seed_ps_ammo_tables(ps, weapon, facts, clip0, clip1, last_hand >= 1, stock);
    (clip0, stock)
}

#[cfg(test)]
mod sent_feel_tests {
    use super::*;

    fn zombies_feel() -> movement_iw4::Bo2Feel {
        movement_iw4::Bo2Feel {
            on: true,
            zombies: true,
            g_speed: 175.0,
            ..movement_iw4::Bo2Feel::IW4
        }
    }

    #[test]
    fn a_second_pc_guesses_with_the_rules_the_game_sent() {
        let feel = zombies_feel();
        assert_eq!(bo2_feel(&World::new(), true, Some(feel)), feel);
    }

    #[test]
    fn the_snapshot_carries_the_rules_to_the_other_pc() {
        let feel = zombies_feel();
        let mut host = crate::carrier::SimWorld::default();
        host.frame().set_feel(feel);
        let snapshot = host.snapshot(crate::world::Tick(1));
        assert_eq!(snapshot.meta.feel, Some(feel));
        let mut guess = crate::carrier::SimWorld::default();
        guess.adopt_snapshot(&snapshot);
        assert_eq!(guess.frame().feel(), Some(feel));
    }
}

#[cfg(test)]
mod fall_damage_tests {
    use super::fall_damage;

    #[test]
    fn falls_cost_a_share_of_max_health_rounded_down() {
        // Worked out in floats: event parm 15 on 100 health is 14 damage.
        assert_eq!(fall_damage(15, 100), 14);
        assert_eq!(fall_damage(0, 100), 0);
        assert_eq!(fall_damage(100, 100), 110);
        assert_eq!(fall_damage(250, 150), 165);
    }
}

#[cfg(test)]
mod body_clip_tests {
    use super::*;

    fn open() -> Trace {
        Trace {
            fraction: 1.0,
            ..Trace::default()
        }
    }

    fn walk(start: [f32; 3], end: [f32; 3]) -> GroundTraceInput {
        GroundTraceInput {
            start,
            end,
            mins: PLAYER_MINS,
            maxs: PLAYER_MAXS,
            tracemask: crate::bullet_collision::MASK_PLAYER_SOLID,
        }
    }

    fn body(entnum: u16, origin: [f32; 3]) -> PlayerBodyClip {
        PlayerBodyClip {
            entnum,
            origin,
            bounds: None,
        }
    }

    #[test]
    fn a_player_body_blocks_a_player_walking_into_it() {
        let bodies = [body(2, [40.0, 0.0, 0.0])];
        let hit = clip_move_to_players(open(), &bodies, 1, walk([0.0; 3], [30.0, 0.0, 0.0]));
        assert!(hit.fraction < 1.0, "walked into a body: {hit:?}");
        assert_eq!(hit.allsolid, 0);
        assert_eq!(hit.hit_id, 2);
    }

    #[test]
    fn two_overlapping_players_can_walk_apart() {
        // Fused: 6 units apart (the screenshot's pair).
        let bodies = [body(2, [6.0, 0.0, 0.0])];
        // The root cause: the raw body trace from inside reads solid, which
        // froze the slide move (allsolid) of both players for good.
        let raw = clipmap_iw4::transformed_temp_capsule_trace(
            [0.0; 3],
            [-10.0, 0.0, 0.0],
            PLAYER_MINS,
            PLAYER_MAXS,
            [6.0, 0.0, 0.0],
            PLAYER_MINS,
            PLAYER_MAXS,
            crate::world::CONTENTS_BODY,
            crate::bullet_collision::MASK_PLAYER_SOLID,
        );
        assert!(raw.startsolid != 0 || raw.allsolid != 0, "{raw:?}");
        let hit = clip_move_to_players(open(), &bodies, 1, walk([0.0; 3], [-10.0, 0.0, 0.0]));
        assert_eq!(hit.allsolid, 0, "overlap must not freeze him: {hit:?}");
        assert_eq!(hit.fraction, 1.0, "moving away is free: {hit:?}");
        // Straight down (the ground trace) is not deeper either.
        let hit = clip_move_to_players(open(), &bodies, 1, walk([0.0; 3], [0.0, 0.0, -0.25]));
        assert_eq!(hit.allsolid, 0);
        assert_eq!(hit.fraction, 1.0);
    }

    #[test]
    fn an_overlapping_player_cannot_push_deeper_and_slides_off() {
        let bodies = [body(2, [6.0, 0.0, 0.0])];
        let hit = clip_move_to_players(open(), &bodies, 1, walk([0.0; 3], [10.0, 3.0, 0.0]));
        assert_eq!(hit.allsolid, 0);
        assert_eq!(hit.fraction, 0.0);
        assert!(hit.normal[0] < -0.99, "plane faces away from him: {hit:?}");
    }
}

#[cfg(test)]
mod dvar_case_tests {
    use super::dvar_number;

    #[test]
    fn script_set_dvars_are_found_by_their_mixed_case_name() {
        // The VM stores `setdvar("jump_slowdownEnable", 0)` as lower case.
        let mut dvars = std::collections::HashMap::new();
        dvars.insert("jump_slowdownenable".to_owned(), "0".to_owned());
        dvars.insert("perk_weapadsmultiplier".to_owned(), "0.5".to_owned());
        assert_eq!(dvar_number(&dvars, "jump_slowdownEnable"), Some(0.0));
        assert_eq!(dvar_number(&dvars, "perk_weapAdsMultiplier"), Some(0.5));
        assert_eq!(dvar_number(&dvars, "bg_gravity"), None);
        dvars.insert("bg_gravity".to_owned(), " 600 ".to_owned());
        assert_eq!(dvar_number(&dvars, "bg_gravity"), Some(600.0));
    }
}
