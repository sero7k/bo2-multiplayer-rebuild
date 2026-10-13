use std::sync::Arc;

use crate::{
    AnimClip, DObjPoseRequest, HidePartBits, XAnimNodeDefinition, XAnimNodeId, XAnimNodeKind,
    XAnimNodeState, XAnimTreeDefinition, XAnimTreeError, XAnimTreeRuntime,
};
use anim_iw4::PartBits;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DObjModelDescriptor {
    pub model: String,
    pub parent_model: Option<u16>,
    pub attach_tag: Option<String>,
    pub ignore_collision: bool,
    /// bo2zm: where the model's root sits on `attach_tag`; none = on it.
    pub tag_offset: Option<TagOffset>,
}

/// bo2zm: a model attached at an offset from its tag, the way Black Ops II
/// hangs a gun's magazine, scope or rocket on the gun (`DObjModel_s` `trans`
/// and `rot`, from `attachWorldModelOffsets` / `...Rotations`). Kept as bits
/// so a descriptor stays `Eq`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TagOffset([u32; 6]);

impl TagOffset {
    /// `rotation` is pitch, yaw, roll in degrees.
    pub fn new(offset: [f32; 3], rotation: [f32; 3]) -> Self {
        let mut bits = [0; 6];
        for i in 0..3 {
            bits[i] = offset[i].to_bits();
            bits[i + 3] = rotation[i].to_bits();
        }
        Self(bits)
    }

    pub fn offset(self) -> [f32; 3] {
        std::array::from_fn(|i| f32::from_bits(self.0[i]))
    }

    pub fn rotation(self) -> [f32; 3] {
        std::array::from_fn(|i| f32::from_bits(self.0[i + 3]))
    }

    /// The root bind `DObj::build_with_root_binds` takes: yaw about z, then
    /// pitch about y, then roll about x (as the view model's parts).
    pub fn root_bind(self) -> (glam::Quat, glam::Vec3) {
        let [pitch, yaw, roll] = self.rotation().map(f32::to_radians);
        let q = glam::Quat::from_rotation_z(yaw)
            * glam::Quat::from_rotation_y(pitch)
            * glam::Quat::from_rotation_x(roll);
        (q, glam::Vec3::from_array(self.offset()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DObjCompositionDescriptor {
    pub revision: u32,
    pub models: Vec<DObjModelDescriptor>,
}

impl DObjCompositionDescriptor {
    pub fn single(revision: u32, model: String) -> Self {
        Self {
            revision,
            models: vec![DObjModelDescriptor {
                model,
                parent_model: None,
                attach_tag: None,
                ignore_collision: false,
                tag_offset: None,
            }],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XAnimSemanticNodeKind {
    Blend,
    Additive,
    Leaf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct XAnimSemanticNode {
    pub parent: Option<XAnimNodeId>,
    pub kind: XAnimSemanticNodeKind,
    pub clip: Option<String>,
    pub parts: Option<PartBits>,
    pub state: XAnimNodeState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct XAnimTreeSnapshot {
    pub definition_revision: u32,
    pub state_revision: u32,
    pub nodes: Vec<XAnimSemanticNode>,
}

/// A limp body (`startragdoll`): each client drops the pose it had then under
/// gravity, thrown by `launch` (`launchragdoll`, counted by `launches`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DObjRagdoll {
    /// Server time it went limp (tells one fall from the next).
    pub start_ms: i32,
    pub launch: [f32; 3],
    pub launches: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DObjSemanticState {
    pub composition: DObjCompositionDescriptor,
    pub pose_revision: u32,
    pub tree: Option<XAnimTreeSnapshot>,
    pub requested_parts: Option<PartBits>,
    pub hide_part_bits: HidePartBits,
    pub ragdoll: Option<DObjRagdoll>,
    /// bo2zm round 1: a zombie's eye brightness (`scriptVector2.y` from its
    /// client script: 2 alive, Origins 3, 0.25 once the eyes go out).
    pub eye_luminance: Option<f32>,
}

impl DObjSemanticState {
    pub fn bind_pose(model: String, composition_revision: u32, pose_revision: u32) -> Self {
        Self {
            composition: DObjCompositionDescriptor::single(composition_revision, model),
            pose_revision,
            tree: None,
            requested_parts: None,
            hide_part_bits: HidePartBits::default(),
            ragdoll: None,
            eye_luminance: None,
        }
    }

    pub fn one_leaf(
        model: String,
        clip: String,
        composition_revision: u32,
        pose_revision: u32,
        time: f32,
    ) -> Self {
        Self {
            composition: DObjCompositionDescriptor::single(composition_revision, model),
            pose_revision,
            tree: Some(XAnimTreeSnapshot {
                definition_revision: 1,
                state_revision: pose_revision,
                nodes: vec![XAnimSemanticNode {
                    parent: None,
                    kind: XAnimSemanticNodeKind::Leaf,
                    clip: Some(clip),
                    parts: None,
                    state: XAnimNodeState {
                        time,
                        old_time: time,
                        weight: 1.0,
                        goal_weight: 1.0,
                        rate: 1.0,
                        ..XAnimNodeState::default()
                    },
                }],
            }),
            requested_parts: None,
            hide_part_bits: HidePartBits::default(),
            ragdoll: None,
            eye_luminance: None,
        }
    }

    pub fn resolve_request(
        &self,
        mut resolve_clip: impl FnMut(&str) -> Option<Arc<AnimClip>>,
    ) -> Result<DObjPoseRequest, SemanticResolveError> {
        let tree = self
            .tree
            .as_ref()
            .map(|tree| tree.resolve(&mut resolve_clip))
            .transpose()?;
        Ok(DObjPoseRequest {
            tree,
            requested_parts: self.requested_parts,
            hide_part_bits: self.hide_part_bits,
        })
    }
}

impl XAnimTreeSnapshot {
    pub fn resolve(
        &self,
        mut resolve_clip: impl FnMut(&str) -> Option<Arc<AnimClip>>,
    ) -> Result<XAnimTreeRuntime, SemanticResolveError> {
        let mut definitions = Vec::with_capacity(self.nodes.len());
        for (node, semantic) in self.nodes.iter().enumerate() {
            let kind = match semantic.kind {
                XAnimSemanticNodeKind::Blend => XAnimNodeKind::Blend,
                XAnimSemanticNodeKind::Additive => XAnimNodeKind::Additive,
                XAnimSemanticNodeKind::Leaf => {
                    let name = semantic
                        .clip
                        .as_deref()
                        .ok_or(SemanticResolveError::LeafClipMissing { node })?;
                    let clip = resolve_clip(name).ok_or_else(|| {
                        SemanticResolveError::ClipUnavailable {
                            node,
                            clip: name.to_owned(),
                        }
                    })?;
                    XAnimNodeKind::Leaf {
                        clip,
                        parts: semantic.parts,
                    }
                }
            };
            definitions.push(XAnimNodeDefinition {
                parent: semantic.parent,
                kind,
            });
        }
        let definition = Arc::new(XAnimTreeDefinition::new(definitions)?);
        let mut runtime = XAnimTreeRuntime::new(definition);
        for (node, semantic) in self.nodes.iter().enumerate() {
            runtime.set_state(XAnimNodeId(node as u16), semantic.state)?;
        }
        Ok(runtime)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticResolveError {
    LeafClipMissing { node: usize },
    ClipUnavailable { node: usize, clip: String },
    Tree(XAnimTreeError),
}

impl From<XAnimTreeError> for SemanticResolveError {
    fn from(value: XAnimTreeError) -> Self {
        Self::Tree(value)
    }
}
