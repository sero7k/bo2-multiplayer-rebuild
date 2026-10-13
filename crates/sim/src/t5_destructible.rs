use crate::frame::FrameWorld;
use crate::world::ClientId;
use crate::{AuthorityDObjState, AuthorityModelOwner, EventAudience, Tick};
use std::sync::Arc;
use xmodel_runtime::T5DestructibleDef;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct State {
    definition: Arc<T5DestructibleDef>,
    health: Vec<i16>,
    /// bo2mp: a Black Ops II map's scripts hear its breaks (`notices`,
    /// t6/destructible.rs).
    hosted: bool,
    notices: Vec<Notice>,
}

/// bo2mp: what a break tells a Black Ops II map's scripts.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Notice {
    /// `codecallback_destructibleevent("broken", notify, attacker, weapon)`:
    /// a stage with a break notify broke (a mannequin's "headless").
    Broken {
        notify: String,
        attacker: Option<ClientId>,
        weapon: u32,
    },
    /// `codecallback_destructibleevent("breakafter", piece, time, damage)`:
    /// a stage that breaks on its own after `time` seconds (a burning car).
    BreakAfter { piece: usize, time: f32, damage: i32 },
    /// The base piece is destroyed: the entity's "death" (by `attacker`).
    Death { attacker: Option<ClientId> },
}

/// bo2mp: which of a piece's damage scales a hit counts by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DamageKind {
    Bullet,
    Explosive,
    Melee,
    /// A script's `dodamage` (unscaled).
    Script,
}

pub fn install(dobj: &mut AuthorityDObjState, definition: Arc<T5DestructibleDef>) {
    dobj.t5_destructible = Some(State {
        health: definition.pieces.iter().map(|p| p.health as i16).collect(),
        definition,
        hosted: false,
        notices: Vec::new(),
    });
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
/// bo2mp: a Black Ops II map's destructible: every piece at full health,
/// the bones the later stages show hidden (a mannequin's broken neck), its
/// breaks kept for the scripts.
pub(crate) fn install_hosted(dobj: &mut AuthorityDObjState, definition: Arc<T5DestructibleDef>) {
    install(dobj, definition);
    if let Some(state) = &mut dobj.t5_destructible {
        state.hosted = true;
    }
    update_hide_parts(dobj);
}

/// bo2mp: whether `dobj` breaks by the definition `name`.
pub(crate) fn installed(dobj: &AuthorityDObjState, name: &str) -> bool {
    dobj.t5_destructible
        .as_ref()
        .is_some_and(|s| s.definition.name.eq_ignore_ascii_case(name))
}

/// bo2mp: the breaks the scripts have not heard yet.
pub(crate) fn take_notices(dobj: &mut AuthorityDObjState) -> Vec<Notice> {
    dobj.t5_destructible
        .as_mut()
        .map(|s| std::mem::take(&mut s.notices))
        .unwrap_or_default()
}

=======
/// [`install`], with the parts its first stage hides hidden now (a prop the
/// server shows itself, not one the map install posed).
pub(crate) fn install_shown(dobj: &mut AuthorityDObjState, definition: Arc<T5DestructibleDef>) {
    install(dobj, definition);
    update_hide_parts(dobj);
}

>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
#[derive(Clone)]
struct Break {
    piece: usize,
    stage: usize,
}

/// What one hit did: the stages left, the stages that break on their own
/// later (piece, seconds, damage), whether the base piece died.
#[derive(Default)]
struct Outcome {
    breaks: Vec<Break>,
    after: Vec<(usize, f32, i32)>,
    died: bool,
}

impl State {
    fn damage(
        &mut self,
        index: usize,
        amount: i32,
        exclude: Option<usize>,
        depth: u32,
        out: &mut Outcome,
    ) {
        if depth > 20 {
            diag::warn!(
                Sim,
                "T5 DamagePiece recursion limit: {}",
                self.definition.name
            );
            return;
        }
        let piece = &self.definition.pieces[index];
        let old = piece.stage(index, self.health[index]);
        let mut health = self.health[index]
            .wrapping_sub(amount.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
            .max(-1);
        let mut next = piece.stage(index, health);
        let previous = old.map(|s| &piece.stages[s]);
        if previous.is_some_and(|s| s.flags & 1 != 0) {
            return;
        }
        if let Some(end) = next {
            for j in old.map_or(0, |s| s + 1)..=end {
                if piece.stages[j].flags & 1 != 0 {
                    health = ((piece.health as f32 * piece.stages[j].break_health) as i16)
                        .wrapping_add(1);
                    next = piece.stage(index, health);
                    break;
                }
            }
        }
        let parent = usize::from(piece.parent_piece);
        if old != next
            && previous.is_some_and(|s| s.has_phys_preset && s.flags & 4 != 0)
            && self.health.get(parent).is_some_and(|h| *h > 0)
        {
            return;
        }
        let damage_parent = previous.is_some_and(|s| s.flags & 2 != 0);
        if index == 0 && self.health[index] > 0 && health <= 0 {
            out.died = true;
        }
        self.health[index] = health;
        if old != next {
            if let Some(start) = old {
                for j in start..next.unwrap_or(5) {
                    if piece.stages[j].show_bone.is_some() {
                        out.breaks.push(Break {
                            piece: index,
                            stage: j,
                        });
                    }
                }
            }
            let after = next.map_or(0.0, |next| piece.stages[next].max_time);
            if after > 0.0 && self.hosted {
                out.after.push((index, after, piece.health));
            } else if after > 0.0 {
                diag::warn!(
                    Sim,
                    "T5 destructible breakafter callback unhosted: {} piece={}",
                    self.definition.name,
                    index
                );
            }
        }
        for child in 0..self.health.len() {
            let p = &self.definition.pieces[child];
            if usize::from(p.parent_piece) == index && Some(index) != exclude {
                let damage = (amount as f32 * p.parent_damage_percent) as i32;
                if damage != 0 {
                    self.damage(child, damage, exclude, depth + 1, out);
                }
            }
        }
        if damage_parent && parent < self.health.len() {
            self.damage(parent, amount, Some(index), depth + 1, out);
        }
    }
}

fn update_hide_parts(dobj: &mut AuthorityDObjState) {
    let Some(state) = &dobj.t5_destructible else {
        return;
    };
    let Some(cap) = &dobj.capability else {
        return;
    };
    let hide = state
        .definition
        .hide_parts(&state.health, &cap.pose.bone_names);
    if hide != dobj.semantic_state.hide_part_bits {
        if std::env::var_os("IW4L_T6_HITLOG").is_some() {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
            diag::info!(
                Sim,
                "bo2mp destructible {} hides {:08x?} (bones {:?}, health {:?})",
                state.definition.name,
                hide.words(),
                cap.pose.bone_names,
                state.health
            );
=======
            diag::info!(Sim, "destructible hide {}: {:08x?}", state.definition.name, hide.words());
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
        }
        dobj.semantic_state.hide_part_bits = hide;
        dobj.pose_request.hide_part_bits = hide;
        dobj.pose_revision = dobj.pose_revision.wrapping_add(1);
        dobj.semantic_state.pose_revision = dobj.pose_revision;
        dobj.current_collision = None;
        dobj.materialized_pose_revision = None;
    }
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
/// bo2mp: Black Ops II's push on a loose piece per unit of hit
/// (`dynEnt_bulletForce`, `dynEnt_explodeForce`) and a blast's lift
/// (`dynEnt_explodeUpbias`).
const BULLET_FORCE: f32 = 0.8;
const EXPLODE_FORCE: f32 = 12.0;
const EXPLODE_UPBIAS: f32 = 0.5;

/// bo2mp: how fast a piece leaves when a hit going `dir` breaks it off:
/// the engine's push scaled by the piece's preset, over its mass. A
/// script's damage (no direction) lets it drop.
fn launch_velocity(
    preset: &xmodel_runtime::DebrisPreset,
    kind: DamageKind,
    dir: Option<[f32; 3]>,
) -> [f32; 3] {
    let Some(dir) = dir.and_then(|d| glam::Vec3::from_array(d).try_normalize()) else {
        return [0.0; 3];
    };
    let push = match kind {
        DamageKind::Explosive => {
            let lifted = (dir + glam::Vec3::Z * EXPLODE_UPBIAS)
                .try_normalize()
                .unwrap_or(dir);
            lifted * EXPLODE_FORCE * preset.explosive_force_scale
        }
        DamageKind::Bullet | DamageKind::Melee => dir * BULLET_FORCE * preset.bullet_force_scale,
        DamageKind::Script => glam::Vec3::ZERO,
    };
    (push / preset.mass.max(1e-4)).to_array()
}

#[allow(clippy::too_many_arguments)]
=======
/// Returns the stages' script notifies (BO2 hands them to the map's
/// scripts: a mannequin's "headless").
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
fn publish(
    world: &mut FrameWorld,
    tick: Tick,
    owner: AuthorityModelOwner,
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
    out: Outcome,
    kind: DamageKind,
    dir: Option<[f32; 3]>,
    attacker: Option<ClientId>,
    weapon: u32,
) {
    let Outcome {
        breaks,
        after,
        died,
    } = out;
    if breaks.is_empty() && after.is_empty() && !died {
        return;
=======
    breaks: Vec<Break>,
) -> Vec<String> {
    let mut notifies = Vec::new();
    if breaks.is_empty() {
        return notifies;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
    }
    let Some(dobj) = world
        .entity_collision_capabilities_mut()
        .iter_mut()
        .find(|r| r.owner == owner)
        .and_then(|r| r.dobj.as_mut())
    else {
        return notifies;
    };
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
    let Some(state) = &mut dobj.t5_destructible else {
        return;
=======
    let Some(state) = &dobj.t5_destructible else {
        return notifies;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
    };
    let definition = state.definition.clone();
    let hosted = state.hosted;
    if hosted {
        for (piece, time, damage) in after {
            state.notices.push(Notice::BreakAfter {
                piece,
                time,
                damage,
            });
        }
    }
    let health = state.health.clone();
    let mut notices = Vec::new();
    let pose = dobj
        .capability
        .as_ref()
        .and_then(|cap| cap.pose(&dobj.pose_request, dobj.world_from_model).ok());
    let mut events = Vec::new();
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
=======
    // The pieces a stage throws (a mannequin's head), from the bone.
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
    let mut debris = Vec::new();
    for event in breaks {
        let piece = &definition.pieces[event.piece];
        let st = &piece.stages[event.stage];

        let bone = dobj.capability.as_ref().and_then(|cap| {
            cap.pose
                .bone_names
                .iter()
                .position(|name| Some(name) == piece.stages[0].show_bone.as_ref())
        });
        let matrix = bone
            .and_then(|b| pose.as_ref()?.get(b))
            .copied()
            .unwrap_or(dobj.world_from_model);
        let origin = matrix.w_axis.truncate().to_array();
        let direction = matrix.x_axis.truncate().normalize().to_array();
        diag::info!(
            Sim,
            "T5 destructible {} piece={} leaving_stage={} fx={:?} health={}",
            definition.name,
            event.piece,
            event.stage,
            st.break_effect,
            health[event.piece]
        );
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
        // The pieces this stage drops come loose where its bone was.
        let shown = dobj.capability.as_ref().and_then(|cap| {
            cap.pose
                .bone_names
                .iter()
                .position(|name| Some(name) == st.show_bone.as_ref())
        });
        let (_, turn, at) = shown
            .and_then(|b| pose.as_ref()?.get(b))
            .copied()
            .unwrap_or(matrix)
            .to_scale_rotation_translation();
        for (model, preset) in st.spawn_models.iter().zip(&st.spawn_presets) {
            let (Some(model), Some(preset)) = (model, preset) else {
                continue;
            };
            let velocity = launch_velocity(preset, kind, dir);
            if std::env::var_os("IW4L_T6_HITLOG").is_some() {
                diag::info!(
                    Sim,
                    "bo2mp destructible {} drops {model} at {at:?} flying {velocity:?}",
                    definition.name
                );
            }
            debris.push(crate::DebrisRecord {
                model: model.clone(),
                origin: at.to_array(),
                rotation: turn.to_array(),
                velocity,
                mass: preset.mass,
                bounce: preset.bounce,
                friction: preset.friction,
                bullet_force_scale: preset.bullet_force_scale,
                explosive_force_scale: preset.explosive_force_scale,
            });
        }
        if let Some(notify) = &st.break_notify
            && hosted
        {
            notices.push(Notice::Broken {
                notify: notify.clone(),
                attacker,
                weapon,
            });
        } else if let Some(notify) = &st.break_notify {
            diag::warn!(
                Sim,
                "T5 destructible script notify unhosted: {} {}",
                definition.name,
                notify
            );
=======
        if st.has_phys_preset && st.spawn_models.iter().all(Option::is_none) {
            diag::info!(
                Sim,
                "T5 destructible {} piece={} stage={}: a physics piece with no model to throw",
                definition.name,
                event.piece,
                event.stage
            );
        }
        for model in st.spawn_models.iter().flatten() {
            debris.push((model.clone(), origin, direction));
        }
        if let Some(notify) = &st.break_notify {
            notifies.push(notify.clone());
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
        }
        if let Some(fx) = &st.break_effect {
            events.push((
                entity_iw4::EntityEventKind::PLAY_FX,
                fx.clone(),
                origin,
                direction,
            ));
        }
        if let Some(sound) = &st.break_sound {
            events.push((
                entity_iw4::EntityEventKind::SOUND_ALIAS,
                sound.clone(),
                origin,
                direction,
            ));
        }
    }
    if hosted && died {
        notices.push(Notice::Death { attacker });
    }
    if let Some(state) = &mut dobj.t5_destructible {
        state.notices.extend(notices);
    }
    update_hide_parts(dobj);
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
    for record in debris {
        world.push_debris(record);
=======
    for (model, origin, direction) in debris {
        // Out along the bone, or the nearest way round with room (never
        // into the wall the prop stands against).
        let (fx, fy) = (direction[0], direction[1]);
        let len = (fx * fx + fy * fy).sqrt();
        let base = if len > 0.1 { fy.atan2(fx) } else { 0.0 };
        let mut out = [base.cos(), base.sin(), 0.0];
        for turn in [0.0f32, 45.0, -45.0, 90.0, -90.0, 135.0, -135.0, 180.0] {
            let a = base + turn.to_radians();
            let d = [a.cos(), a.sin(), 0.0];
            let t = world.trace_static_world(
                origin,
                [origin[0] + d[0] * 48.0, origin[1] + d[1] * 48.0, origin[2]],
                [-4.0; 3],
                [4.0; 3],
                crate::bullet_collision::MASK_PLAYER_SOLID,
            );
            if t.fraction >= 1.0 && t.startsolid == 0 {
                out = d;
                break;
            }
        }
        let direction = out;
        // Where it lands: the floor under the bone.
        let t = world.trace_static_world(
            origin,
            [origin[0], origin[1], origin[2] - 2000.0],
            [0.0; 3],
            [0.0; 3],
            crate::bullet_collision::MASK_PLAYER_SOLID,
        );
        let floor = if t.fraction < 1.0 { t.endpos[2] } else { origin[2] - 64.0 };
        crate::t6::throw_debris(world.ecs(), &model, origin, direction, floor);
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
    }
    for (kind, name, origin, direction) in events {
        let index = if kind == entity_iw4::EntityEventKind::PLAY_FX {
            world.effect_name_index(&name)
        } else {
            world.sound_alias_index(&name)
        };
        world.push_entity_event(
            tick,
            EventAudience::All,
            kind,
            crate::EntityEventPayload {
                number: i32::from(trace_iw4::ENTITYNUM_WORLD),
                event_parm: i32::from(index),
                origin,
                direction,
                ..Default::default()
            },
        );
    }
    notifies
}

/// A blast (a grenade) damages each standing piece of every breakable prop
/// in reach, by the distance from the piece's bone, times the piece's
/// explosive scale. Returns each prop's script notifies.
pub(crate) fn apply_blast(
    world: &mut FrameWorld,
    tick: Tick,
    origin: [f32; 3],
    radius: f32,
    amount_at: impl Fn(f32) -> i32,
) -> Vec<(AuthorityModelOwner, Vec<String>)> {
    let origin = glam::Vec3::from_array(origin);
    let mut hits = Vec::new();
    for row in world.entity_collision_capabilities() {
        let Some(dobj) = row.dobj.as_ref() else {
            continue;
        };
        let (Some(state), Some(cap)) = (&dobj.t5_destructible, &dobj.capability) else {
            continue;
        };
        if dobj.world_from_model.w_axis.truncate().distance(origin) > radius + 128.0 {
            continue;
        }
        let Ok(pose) = cap.pose(&dobj.pose_request, dobj.world_from_model) else {
            continue;
        };
        for (i, piece) in state.definition.pieces.iter().enumerate() {
            let Some(stage) = piece.stage(i, state.health[i]) else {
                continue;
            };
            let at = piece.stages[stage]
                .show_bone
                .as_ref()
                .and_then(|name| cap.pose.bone_names.iter().position(|n| n == name))
                .and_then(|b| pose.get(b))
                .map_or(dobj.world_from_model.w_axis, |m| m.w_axis)
                .truncate();
            let d = at.distance(origin);
            if d > radius {
                continue;
            }
            // Not through walls (a wall prop's own wall stops the line just
            // short of it).
            let start = (origin + glam::Vec3::Z * 4.0).to_array();
            let t = world.trace_static_world(
                start,
                at.to_array(),
                [0.0; 3],
                [0.0; 3],
                crate::bullet_collision::MASK_SHOT,
            );
            if (1.0 - t.fraction) * d > 12.0 {
                continue;
            }
            let amount = (amount_at(d) as f32 * piece.explosive_damage_scale) as i32;
            if amount > 0 {
                hits.push((row.owner, i, amount));
            }
        }
    }
    let mut out = Vec::new();
    for (owner, index, amount) in hits {
        let Some(dobj) = world
            .entity_collision_capabilities_mut()
            .iter_mut()
            .find(|r| r.owner == owner)
            .and_then(|r| r.dobj.as_mut())
        else {
            continue;
        };
        let Some(state) = &mut dobj.t5_destructible else {
            continue;
        };
        let mut breaks = Vec::new();
        state.damage(index, amount, None, 0, &mut breaks);
        if std::env::var_os("IW4L_T6_HITLOG").is_some() {
            diag::info!(
                Sim,
                "destructible blast {}: piece {index} -{amount} -> health {:?} breaks {}",
                state.definition.name,
                state.health,
                breaks.len()
            );
        }
        let notifies = publish(world, tick, owner, breaks);
        out.push((owner, notifies));
    }
    out
}

pub(crate) fn apply_hit(
    world: &mut FrameWorld,
    tick: Tick,
    owner: AuthorityModelOwner,
    bone: u16,
    amount: u32,
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
    dir: [f32; 3],
    attacker: Option<ClientId>,
    weapon: u32,
) -> bool {
    let amount = i32::try_from(amount).unwrap_or(i32::MAX);
    apply_damage(
        world,
        tick,
        owner,
        Some(bone),
        amount,
        DamageKind::Bullet,
        Some(dir),
        attacker,
        weapon,
    )
}

/// bo2mp: damage a destructible: the piece whose shown bone (or a bone
/// under it) was struck, else the base piece, by that piece's scale for the
/// kind. A piece it breaks off flies along `dir`. False when `owner` is
/// not a destructible.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_damage(
    world: &mut FrameWorld,
    tick: Tick,
    owner: AuthorityModelOwner,
    bone: Option<u16>,
    amount: i32,
    kind: DamageKind,
    dir: Option<[f32; 3]>,
    attacker: Option<ClientId>,
    weapon: u32,
) -> bool {
=======
) -> Option<Vec<String>> {
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
    let Some(dobj) = world
        .entity_collision_capabilities_mut()
        .iter_mut()
        .find(|r| r.owner == owner)
        .and_then(|r| r.dobj.as_mut())
    else {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
        return false;
    };
    let Some(state) = &mut dobj.t5_destructible else {
        return false;
=======
        return None;
    };
    let Some(state) = &mut dobj.t5_destructible else {
        return None;
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
    };
    let index = bone
        .zip(dobj.capability.as_ref())
        .and_then(|(bone, cap)| struck_piece(state, &cap.pose, usize::from(bone)))
        .unwrap_or(0);
    let piece = &state.definition.pieces[index];
    let scale = match kind {
        DamageKind::Bullet => piece.bullet_damage_scale,
        DamageKind::Explosive => piece.explosive_damage_scale,
        DamageKind::Melee => piece.melee_damage_scale,
        DamageKind::Script => 1.0,
    };
    let damage = (amount as f32 * scale) as i32;
    if std::env::var_os("IW4L_T6_HITLOG").is_some() {
        let tag = bone
            .zip(dobj.capability.as_ref())
            .and_then(|(b, cap)| cap.pose.bone_names.get(usize::from(b)).cloned());
        let shows: Vec<_> = piece.stages.iter().map(|s| s.show_bone.clone()).collect();
        diag::info!(
            Sim,
            "bo2mp destructible {} hit on {tag:?}: piece {index} of {} (stages show {shows:?}) \
             takes {damage} ({kind:?} x{scale}), health {:?}",
            state.definition.name,
            state.definition.pieces.len(),
            state.health
        );
    }
    let mut out = Outcome::default();
    state.damage(index, damage, None, 0, &mut out);
    publish(world, tick, owner, out, kind, dir, attacker, weapon);
    true
}

/// The piece showing the struck bone, or the nearest bone above it that a
/// piece shows.
fn struck_piece(state: &State, pose: &xmodel_runtime::ModelPoseSrc, bone: usize) -> Option<usize> {
    let mut b = bone;
    loop {
        let tag = pose.bone_names.get(b)?;
        let found = state.definition.pieces.iter().enumerate().position(|(i, p)| {
            p.stage(i, state.health[i])
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/sim/src/t5_destructible.rs
                .is_some_and(|st| p.stages[st].show_bone.as_ref() == Some(tag))
        });
        if found.is_some() {
            return found;
        }
        let step = usize::from(*pose.parent_list.get(b.checked_sub(pose.num_root_bones)?)?);
        b = b.checked_sub(step).filter(|_| step > 0)?;
    }
=======
                .is_some_and(|st| p.stages[st].show_bone.as_ref() == tag)
        })
        .unwrap_or(0);
    let damage = (amount as f32 * state.definition.pieces[index].bullet_damage_scale) as i32;
    let mut breaks = Vec::new();
    state.damage(index, damage, None, 0, &mut breaks);
    if std::env::var_os("IW4L_T6_HITLOG").is_some() {
        diag::info!(
            Sim,
            "destructible hit {}: bone {bone} {:?} piece {index} -{damage} -> health {:?} breaks {}",
            state.definition.name,
            tag,
            state.health,
            breaks.len()
        );
    }
    Some(publish(world, tick, owner, breaks))
>>>>>>> /tmp/claude-0/zmhead/crates/sim/src/t5_destructible.rs
}
