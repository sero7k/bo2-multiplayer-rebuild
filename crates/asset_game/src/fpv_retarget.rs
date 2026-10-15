//! bo2mp: Black Ops III's first-person animations on Black Ops II's arms.
//!
//! A Black Ops III gun's view animations key Black Ops III's arm skeleton:
//! the gun's root (`tag_weapon`) is a child of the arms' `tag_weapon_right`
//! (Black Ops II's arms have no such bone, and their own `tag_weapon` hangs
//! off `tag_torso`, at the eye), and every arm and finger bone sits in a frame
//! of its own (measured on the bind poses: the arm bones turned about 135
//! degrees about their length, the finger bones 40 to 70 degrees, the finger
//! chains one joint further along). Played straight onto Black Ops II's arms
//! the gun lands on the eye and the arms twist.
//!
//! So the clips pose Black Ops III's own arm skeleton (a reference viewhands
//! model the lane keeps, [`T7_REFERENCE_ARMS`]), and each Black Ops II arm
//! bone takes its counterpart's place and turn, the turn carried over by the
//! difference of the two bind poses: at the bind pose Black Ops II's arms
//! are their own bind pose, and every turn Black Ops III's animators gave
//! their bone, the Black Ops II bone makes. The gun's root takes Black Ops
//! III's `tag_weapon` (on `tag_weapon_right`), so the gun is where Black Ops
//! III holds it and the hands close on it. Bones with no counterpart (sleeve
//! shapers, finger tips, the knife tag) follow their parent at their bind
//! offset.

use std::collections::HashMap;

use bevy::math::{Quat, Vec3};
use xmodel_runtime::{AnimInstance, DObj, ModelPoseSrc};

/// The name the Black Ops III lane gives its reference arms in the
/// first-person catalog (namespace T7).
pub const T7_REFERENCE_ARMS: &str = "t7_reference_viewhands";

/// Black Ops II arm bone -> Black Ops III arm bone. Black Ops II's finger
/// chains start at the knuckle (`j_index_le_0`); Black Ops III's at a palm
/// bone, the knuckle being `_1`. Black Ops II's ring and pinky palm bones are
/// Black Ops III's ring and pinky bases.
fn t7_counterpart(bo2: &str) -> Option<String> {
    let direct = [
        "tag_view",
        "tag_ads",
        "tag_cambone",
        "tag_camera",
        "tag_torso",
        "j_shoulder_le",
        "j_shoulder_ri",
        "j_elbow_le",
        "j_elbow_ri",
        "j_wrist_le",
        "j_wrist_ri",
        "j_wristtwist_le",
        "j_wristtwist_ri",
    ];
    if direct.contains(&bo2) {
        return Some(bo2.to_owned());
    }
    for (from, to) in [("j_pinkypalm_", "j_pinkybase_"), ("j_ringpalm_", "j_ringbase_")] {
        if let Some(side) = bo2.strip_prefix(from) {
            return Some(format!("{to}{side}"));
        }
    }
    // j_<finger>_<side>_<n>, n 0..2 -> n + 1 (the tips, n = 3, have none).
    let rest = bo2.strip_prefix("j_")?;
    let mut parts = rest.split('_');
    let (finger, side, n) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || !["index", "mid", "ring", "pinky", "thumb"].contains(&finger) || !["le", "ri"].contains(&side) {
        return None;
    }
    let n: u8 = n.parse().ok()?;
    (n < 3).then(|| format!("j_{finger}_{side}_{}", n + 1))
}

#[derive(Clone, Copy, Debug)]
enum Source {
    /// A Black Ops III arm bone: its place, and its turn times `k`.
    Bone(usize, Quat),
    /// The gun's root on Black Ops III's `tag_weapon_right`.
    GunRoot,
    /// None: the bind offset from the parent.
    Bind,
}

#[derive(Debug)]
pub struct FpvArmsRetarget {
    /// Black Ops III's arms: names, parents, rest local turn and offset.
    src_index: HashMap<String, usize>,
    src_parent: Vec<Option<usize>>,
    /// Per bone: its bind turn and offset on its parent (an unkeyed bone),
    /// and its stored rest offset (a keyed bone's offset is that plus the
    /// clip's, as the engine composes it).
    src_rest: Vec<(Quat, Vec3, Vec3)>,
    /// `tag_weapon_right` (the gun root's parent).
    weapon_right: usize,
    /// Per Black Ops II arm bone (model 0 of the DObj): where its pose comes
    /// from, and its bind offset from its parent.
    dst: Vec<(Source, Quat, Vec3)>,
    /// How many Black Ops II bones have a counterpart.
    pub mapped: usize,
}

fn rest_local(pose: &ModelPoseSrc, bone: usize) -> (Quat, Vec3) {
    if bone < pose.num_root_bones {
        return (Quat::IDENTITY, Vec3::ZERO);
    }
    let child = bone - pose.num_root_bones;
    let q = pose.quats.get(child).map_or(Quat::IDENTITY, |q| {
        let f = |v: i16| f32::from(v) / 32767.0;
        Quat::from_xyzw(f(q[0]), f(q[1]), f(q[2]), f(q[3])).normalize()
    });
    let t = pose.trans.get(child).map_or(Vec3::ZERO, |t| Vec3::from_array(*t));
    (q, t)
}

/// A bone's place on its parent at the bind pose (from the bind matrices:
/// a viewhands model's own rest offsets are zero, its animations carrying
/// the whole offset).
fn bind_local(pose: &ModelPoseSrc, parent: Option<usize>, bone: usize) -> (Quat, Vec3) {
    let (q, t) = pose.base_mat[bone];
    let q = q.normalize();
    match parent {
        Some(p) => {
            let (pq, pt) = pose.base_mat[p];
            let inv = pq.normalize().inverse();
            ((inv * q).normalize(), inv * (t - pt))
        }
        None => (q, t),
    }
}

fn parent_of(pose: &ModelPoseSrc, bone: usize) -> Option<usize> {
    if bone < pose.num_root_bones {
        return None;
    }
    let step = usize::from(*pose.parent_list.get(bone - pose.num_root_bones)?);
    bone.checked_sub(step).filter(|_| step > 0)
}

impl FpvArmsRetarget {
    /// `src`: Black Ops III's reference arms. `dobj`: the assembly, its model
    /// 0 Black Ops II's arms. `None` when the reference lacks the bones the
    /// gun hangs on.
    pub fn build(src: &ModelPoseSrc, dobj: &DObj) -> Option<Self> {
        let n = src.num_bones.min(src.bone_names.len()).min(src.base_mat.len());
        let src_index: HashMap<String, usize> =
            src.bone_names.iter().take(n).enumerate().map(|(i, name)| (name.to_ascii_lowercase(), i)).collect();
        let weapon_right = *src_index.get("tag_weapon_right")?;
        let src_parent: Vec<Option<usize>> = (0..n).map(|b| parent_of(src, b).filter(|&p| p < n)).collect();
        // An unkeyed bone rests at its bind pose; a keyed one takes the
        // clip's turn and offset (Black Ops III's viewhands store no rest
        // offsets of their own: `rest_local` is zero for them).
        let src_rest = (0..n)
            .map(|b| {
                let (_, t) = rest_local(src, b);
                let (bq, bt) = bind_local(src, src_parent[b], b);
                (bq, bt, t)
            })
            .collect();
        let arms = dobj.models.first()?;
        let mut mapped = 0;
        let dst = (arms.base..arms.base + arms.bone_count)
            .map(|b| {
                let bone = &dobj.bones[b];
                let (_, bind_q, bind_t) = bone.bind_world.to_scale_rotation_translation();
                let (offset_q, offset_t) = match bone.parent {
                    Some(p) => {
                        let (_, pq, pt) = dobj.bones[p].bind_world.to_scale_rotation_translation();
                        let inv = pq.inverse();
                        (inv * bind_q, inv * (bind_t - pt))
                    }
                    None => (bind_q, bind_t),
                };
                let name = bone.name.to_ascii_lowercase();
                let source = if name == "tag_weapon" {
                    Source::GunRoot
                } else if let Some(s) = t7_counterpart(&name).and_then(|s| src_index.get(&s).copied()) {
                    let (src_q, _) = src.base_mat[s];
                    Source::Bone(s, src_q.normalize().inverse() * bind_q)
                } else {
                    Source::Bind
                };
                if !matches!(source, Source::Bind) {
                    mapped += 1;
                }
                (source, offset_q, offset_t)
            })
            .collect();
        Some(Self { src_index, src_parent, src_rest, weapon_right, dst, mapped })
    }

    /// Black Ops II's arm bones' locals (model 0 of `dobj`) from the clips
    /// posing Black Ops III's arms; the gun's own bones keep what the clips
    /// gave them.
    pub fn apply(&self, dobj: &DObj, anims: &[AnimInstance<'_>], locals: &mut [anim_iw4::Local]) {
        let n = self.src_rest.len();
        // The clips' blend on Black Ops III's bones (as `DObj::calc_anim`),
        // the gun's root last.
        let mut rot = vec![[0.0f32; 4]; n + 1];
        let mut trans = vec![[0.0f32; 3]; n + 1];
        let mut weight = vec![0.0f32; n + 1];
        let mut keyed = vec![false; n + 1];
        for anim in anims {
            if anim.weight <= 0.0 {
                continue;
            }
            for (track, t) in anim.clip.tracks.iter().enumerate() {
                let name = t.name.to_ascii_lowercase();
                let bone = if name == "tag_weapon" {
                    n
                } else {
                    match self.src_index.get(&name) {
                        Some(&b) => b,
                        None => continue,
                    }
                };
                let Some(sample) = anim.clip.sample_track(track, anim.time) else { continue };
                keyed[bone] = true;
                weight[bone] += anim.weight;
                rot[bone] = anim_iw4::quat_add_weighted(rot[bone], sample.rotation.unwrap_or(anim_iw4::QUAT_IDENTITY), anim.weight);
                if let Some(v) = sample.translation {
                    for k in 0..3 {
                        trans[bone][k] += v[k] * anim.weight;
                    }
                }
            }
        }
        let local = |b: usize| -> (Quat, Vec3) {
            let (bind_q, bind_t, rest_t) = if b < n { self.src_rest[b] } else { (Quat::IDENTITY, Vec3::ZERO, Vec3::ZERO) };
            if !keyed[b] {
                return (bind_q, bind_t);
            }
            let q = anim_iw4::normalize(rot[b]);
            let w = if weight[b] > 0.0 { 1.0 / weight[b] } else { 1.0 };
            (Quat::from_xyzw(q[0], q[1], q[2], q[3]), rest_t + Vec3::from_array(trans[b]) * w)
        };
        // Black Ops III's arms posed (parents come first).
        let mut world: Vec<(Quat, Vec3)> = Vec::with_capacity(n + 1);
        for b in 0..n {
            let (q, t) = local(b);
            world.push(match self.src_parent[b] {
                Some(p) if p < b => (world[p].0 * q, world[p].0 * t + world[p].1),
                _ => (q, t),
            });
        }
        let (q, t) = local(n);
        let (pq, pt) = world[self.weapon_right];
        world.push((pq * q, pq * t + pt));
        // Black Ops II's arms on them.
        let Some(arms) = dobj.models.first() else { return };
        let mut posed: Vec<(Quat, Vec3)> = Vec::with_capacity(self.dst.len());
        for (i, &(source, offset_q, offset_t)) in self.dst.iter().enumerate() {
            let b = arms.base + i;
            let parent = dobj.bones[b].parent.and_then(|p| p.checked_sub(arms.base)).filter(|&p| p < i);
            let here = match source {
                Source::Bone(s, k) => ((world[s].0 * k).normalize(), world[s].1),
                Source::GunRoot => world[n],
                Source::Bind => match parent {
                    Some(p) => (posed[p].0 * offset_q, posed[p].0 * offset_t + posed[p].1),
                    None => (offset_q, offset_t),
                },
            };
            posed.push(here);
            let (lq, lt) = match parent {
                Some(p) => {
                    let inv = posed[p].0.inverse();
                    (inv * here.0, inv * (here.1 - posed[p].1))
                }
                None => here,
            };
            if let Some(slot) = locals.get_mut(b) {
                let bind_t = dobj.bones[b].bind_translation;
                slot.rotation = lq.normalize().to_array();
                slot.translation = (lt - bind_t).to_array();
                slot.control = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counterparts() {
        assert_eq!(t7_counterpart("j_index_le_0").as_deref(), Some("j_index_le_1"));
        assert_eq!(t7_counterpart("j_thumb_ri_2").as_deref(), Some("j_thumb_ri_3"));
        assert_eq!(t7_counterpart("j_pinky_ri_3"), None);
        assert_eq!(t7_counterpart("j_pinkypalm_le").as_deref(), Some("j_pinkybase_le"));
        assert_eq!(t7_counterpart("j_wrist_ri").as_deref(), Some("j_wrist_ri"));
        assert_eq!(t7_counterpart("j_sleave_reshape_top_ri_1"), None);
    }

    fn q16(q: Quat) -> [i16; 4] {
        q.to_array().map(|v| (v * 32767.0).round() as i16)
    }

    /// A chain of bones (model space bind poses), parents one back.
    fn pose(name: &str, bones: &[(&str, Quat, Vec3)]) -> ModelPoseSrc {
        let mut quats = Vec::new();
        let mut trans = Vec::new();
        for i in 1..bones.len() {
            let (pq, pt) = (bones[i - 1].1, bones[i - 1].2);
            quats.push(q16(pq.inverse() * bones[i].1));
            trans.push((pq.inverse() * (bones[i].2 - pt)).to_array());
        }
        ModelPoseSrc {
            name: name.into(),
            num_bones: bones.len(),
            num_root_bones: 1,
            scale: 1.0,
            no_scale_part_bits: [0; 6],
            bone_names: bones.iter().map(|b| b.0.to_owned()).collect(),
            parent_list: vec![1; bones.len() - 1],
            quats,
            trans,
            base_mat: bones.iter().map(|b| (b.1, b.2)).collect(),
        }
    }

    /// With no clip, Black Ops III's arms rest at their bind pose and Black
    /// Ops II's arms come out at theirs (the frames differ by a roll).
    #[test]
    fn bind_pose_maps_to_bind_pose() {
        let roll = Quat::from_rotation_x(2.4);
        let src = pose(
            "t7",
            &[
                ("tag_view", Quat::IDENTITY, Vec3::ZERO),
                ("tag_torso", Quat::IDENTITY, Vec3::ZERO),
                ("tag_weapon_right", Quat::from_rotation_z(0.3), Vec3::new(14.0, -18.0, -17.0)),
                ("j_shoulder_ri", Quat::from_rotation_y(0.5), Vec3::new(-1.0, -8.0, -4.0)),
                ("j_elbow_ri", Quat::from_rotation_y(0.9), Vec3::new(2.0, -15.0, -11.0)),
            ],
        );
        let dst = pose(
            "t6",
            &[
                ("tag_view", Quat::IDENTITY, Vec3::ZERO),
                ("tag_torso", Quat::IDENTITY, Vec3::new(-4.6, 0.0, -1.4)),
                ("tag_weapon", Quat::IDENTITY, Vec3::new(10.5, 0.0, -8.1)),
                ("j_shoulder_ri", Quat::from_rotation_y(0.5) * roll, Vec3::new(-3.5, -7.9, -3.2)),
                ("j_elbow_ri", Quat::from_rotation_y(0.9) * roll, Vec3::new(2.2, -8.1, -13.8)),
            ],
        );
        let dobj = DObj::build(&[(&dst, None)]).expect("dobj");
        let rt = FpvArmsRetarget::build(&src, &dobj).expect("retarget");
        assert_eq!(rt.mapped, 5);
        let mut locals = dobj.bind_locals();
        rt.apply(&dobj, &[], &mut locals);
        let world = dobj.compose(&locals, bevy::math::Mat4::IDENTITY);
        // The elbow: Black Ops III's place, Black Ops II's bind turn.
        let (_, q, t) = world[4].to_scale_rotation_translation();
        assert!((t - Vec3::new(2.0, -15.0, -11.0)).length() < 0.01, "{t}");
        assert!(q.dot(Quat::from_rotation_y(0.9) * roll).abs() > 0.9999);
        // The gun's root on tag_weapon_right.
        let (_, q, t) = world[2].to_scale_rotation_translation();
        assert!((t - Vec3::new(14.0, -18.0, -17.0)).length() < 0.01, "{t}");
        assert!(q.dot(Quat::from_rotation_z(0.3)).abs() > 0.9999);
    }
}
