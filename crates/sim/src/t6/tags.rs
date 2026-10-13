//! Where an entity's tag (a bone of its model) is this tick: its models
//! (body and what is attached) posed by the playing animation and placed
//! where the entity stands. `gettagorigin`, `gettagangles` and `linkto` with
//! a tag read it (Origins' giant robots carry their stomp triggers and soles
//! on their feet's hatch tags).

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::prelude::{Resource, World};

use super::{Zm, frame};

/// One entity's posed bones at one time: (lower-case name, world matrix).
pub(crate) struct Posed {
    bones: Vec<(String, glam::Mat4)>,
}

/// The poses worked out this tick, by entity (a robot's feet are read by
/// several triggers and the scripts each tick).
#[derive(Resource, Default)]
struct Cache(HashMap<u32, (i64, String, Arc<Posed>)>);

/// The tag's world origin and (forward, left, up) axes.
pub(crate) fn tag_world(world: &mut World, n: u32, tag: &str) -> Option<([f32; 3], [[f32; 3]; 3])> {
    let Some(posed) = posed(world, n) else {
        miss(world, n, tag, None);
        return None;
    };
    let Some(m) = posed
        .bones
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(tag))
        .map(|(_, m)| *m)
    else {
        miss(world, n, tag, Some(&posed));
        return None;
    };
    let axis = |v: glam::Vec4| v.truncate().normalize_or_zero().to_array();
    Some((
        m.w_axis.truncate().to_array(),
        [axis(m.x_axis), axis(m.y_axis), axis(m.z_axis)],
    ))
}

/// Logged once per (model, tag): why a tag was not found (the model not
/// loaded, or posed without that bone), and some of the bones it has.
fn miss(world: &mut World, n: u32, tag: &str, posed: Option<&Posed>) {
    static SEEN: std::sync::Mutex<Vec<(String, String)>> = std::sync::Mutex::new(Vec::new());
    let (model, attached) = {
        let zm = world.resource::<Zm>();
        (
            body_model(zm, n).unwrap_or_default(),
            zm.ents.get(&n).map_or(0, |e| e.attached.len()),
        )
    };
    let key = (model.clone(), tag.to_ascii_lowercase());
    {
        let mut seen = SEEN.lock().unwrap();
        if seen.contains(&key) {
            return;
        }
        seen.push(key);
    }
    let cap = match frame(world).model_capability(&model) {
        None => "never loaded",
        Some(None) => "failed to load",
        Some(Some(_)) => "loaded",
    };
    let bones = posed.map_or_else(
        || "not posed".to_owned(),
        |p| {
            let want = tag.to_ascii_lowercase();
            let near: Vec<&str> = p
                .bones
                .iter()
                .map(|(b, _)| b.as_str())
                .filter(|b| ["hatch", "foot", "toe", "attach"].iter().any(|k| b.contains(k)) || want.contains(*b))
                .take(12)
                .collect();
            let first: Vec<&str> = p.bones.iter().map(|(b, _)| b.as_str()).take(8).collect();
            format!("{} bones, like [{}], first [{}]", p.bones.len(), near.join(" "), first.join(" "))
        },
    );
    diag::info!(
        Sim,
        "bo2zm t6 tag miss: ent {n} model \"{model}\" ({cap}, {attached} attached) has no {tag}: {bones}"
    );
}

/// The tag's world angles (pitch, yaw, roll) from its axes.
pub(crate) fn axis_to_angles(axes: [[f32; 3]; 3]) -> [f32; 3] {
    let [f, l, u] = axes;
    let flat = (f[0] * f[0] + f[1] * f[1]).sqrt();
    let yaw = f[1].atan2(f[0]).to_degrees();
    let pitch = (-f[2]).atan2(flat).to_degrees();
    let roll = if flat > 1e-4 { l[2].atan2(u[2]).to_degrees() } else { 0.0 };
    [pitch, yaw, roll]
}

/// The entity's body model: a player's is his own `setmodel` (kept apart,
/// as his body is not drawn here), the rest are their entity's. A player
/// has no entity record (he lives in the sim), so his is read first: Die
/// Rise's egg reward flies to his J_SpineLower and found no model at all.
fn body_model(zm: &Zm, n: u32) -> Option<String> {
    if let Some(p) = zm.players.get(&n) {
        return Some(p.model.clone());
    }
    Some(zm.ents.get(&n)?.model.clone())
}

fn posed(world: &mut World, n: u32) -> Option<Arc<Posed>> {
    let (now, model) = {
        let zm = world.resource::<Zm>();
        (zm.now_ms, body_model(zm, n)?)
    };
    if model.is_empty() {
        return None;
    }
    if let Some(c) = world.get_resource::<Cache>()
        && let Some((t, m, p)) = c.0.get(&n)
        && *t == now
        && *m == model
    {
        return Some(p.clone());
    }
    let p = Arc::new(pose(world, n, now)?);
    if !world.contains_resource::<Cache>() {
        world.insert_resource(Cache::default());
    }
    let mut c = world.resource_mut::<Cache>();
    c.0.retain(|_, (t, ..)| *t == now);
    c.0.insert(n, (now, model, p.clone()));
    Some(p)
}

fn pose(world: &mut World, n: u32, now: i64) -> Option<Posed> {
    use xmodel_runtime::{AnimInstance, Attach, DObj};
    let (model, attached, origin, angles) = if world.resource::<Zm>().players.contains_key(&n) {
        // A player's body stands where the sim has him, upright, turned
        // only by his yaw (his look up and down is not the body's).
        let model = body_model(world.resource::<Zm>(), n)?;
        let (origin, yaw) = frame(world)
            .player(crate::world::ClientId(n))
            .map(|ps| (ps.origin, ps.viewangles[1]))?;
        (model, Vec::new(), origin, [0.0, yaw, 0.0])
    } else {
        let zm = world.resource::<Zm>();
        let e = zm.ents.get(&n)?;
        (body_model(zm, n)?, e.attached.clone(), e.origin, e.angles)
    };
    let caps: Vec<(
        Arc<xmodel_runtime::RetainedModelCapability>,
        Option<String>,
    )> = {
        let f = frame(world);
        let body = f.model_capability(&model).flatten()?;
        let mut v = vec![(body, None)];
        for (m, tag) in &attached {
            if let Some(c) = f.model_capability(m).flatten() {
                v.push((c, Some(tag.clone())));
            }
        }
        v
    };
    let models: Vec<(&xmodel_runtime::ModelPoseSrc, Option<Attach>)> = caps
        .iter()
        .map(|(c, tag)| {
            let attach = tag.as_ref().map(|t| Attach {
                parent_model: 0,
                tag: if t.is_empty() {
                    xmodel_runtime::empty_tag_attach(&caps[0].0.pose, &c.pose)
                } else {
                    t.clone()
                },
            });
            (&c.pose, attach)
        })
        .collect();
    // An attached model the body cannot take (no such tag on it) fails the
    // whole build: keep the body and the attachments that do build (the
    // giant robots' feet tags are on the body).
    let dobj = match DObj::build(&models) {
        Ok(d) => d,
        Err(e) => {
            let mut keep = vec![models[0].clone()];
            for m in &models[1..] {
                keep.push(m.clone());
                if DObj::build(&keep).is_err() {
                    keep.pop();
                }
            }
            static SEEN: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
            let mut seen = SEEN.lock().unwrap();
            if !seen.contains(&model) {
                seen.push(model.clone());
                diag::info!(
                    Sim,
                    "bo2zm t6 tag pose: model \"{model}\" keeps {} of {} attached ({e})",
                    keep.len() - 1,
                    models.len() - 1
                );
            }
            drop(seen);
            DObj::build(&keep).ok()?
        }
    };
    // Placed as the model is drawn (`bullet_collision`'s angles to matrix).
    let [pitch, yaw, roll] = angles.map(f32::to_radians);
    let rotation = glam::Quat::from_euler(glam::EulerRot::ZYX, yaw, pitch, roll);
    let world_from_model =
        glam::Mat4::from_rotation_translation(rotation, glam::Vec3::from_array(origin));
    // The playing clip at its time: an actor's, a prop's or a script
    // model's `setanim` (bind pose without one, or a clip the server never
    // loaded).
    let playing = super::actors::present_anim(world, n, now)
        .or_else(|| super::fxanims::present(world, n, now))
        .or_else(|| super::presence::script_anim(world, n, now))
        .and_then(|(name, frac, _)| {
            let zm = world.resource::<Zm>();
            let clip = zm
                .clips
                .get(&name)
                .or_else(|| zm.clips.get(&name.to_ascii_lowercase()))?
                .clone();
            Some((clip, frac))
        });
    let posed = match &playing {
        Some((clip, frac)) => {
            let tracks: Vec<Option<usize>> =
                clip.tracks.iter().map(|t| dobj.find(&t.name)).collect();
            let inst = AnimInstance {
                clip,
                tracks: &tracks,
                time: frac * clip.duration().max(1e-3),
                weight: 1.0,
                parts: None,
            };
            dobj.pose(&[inst], &dobj.all_parts(), world_from_model)
        }
        None => dobj.pose(&[], &dobj.all_parts(), world_from_model),
    };
    Some(Posed {
        bones: dobj
            .bones
            .iter()
            .zip(posed)
            .map(|(b, m)| (b.name.to_ascii_lowercase(), m))
            .collect(),
    })
}
