//! bo2mp: Call of Duty: Advanced Warfare's effects into the effect catalog,
//! for its guest guns (their flashes, shell ejects, trails, impacts).
//!
//! `fastfile_s1`'s walk hands an effect over as its element records
//! (`FxElemDef`, 320 bytes) plus every name an element points at; the
//! `assets` lane resolves those names and passes them here as an
//! `S1FxView`. AW keeps the IW element in a wider record (a second flag
//! word, a lit type, HDR colours, pivots), and numbers its element types
//! with Black Ops II's sprites and lines plus flares, particle simulations
//! and vector fields. This maps it to the catalog's IW4 element:
//!
//! - fields are read by AW's offsets (x64-zt's S1 layout) into the view the
//!   runtime plays;
//! - velocity samples have IW4's layout and are kept as they are;
//! - visual samples (AW: float colour, emissive scale, pivot) become IW4's
//!   (colour bytes blue first): the colour times its emissive scale, scaled
//!   back into range keeping its hue when it is brighter than white;
//! - element types: rotated sprites draw oriented, lines as tails that run
//!   forward (as Black Ops II's), clouds as sprites; flares, spot lights,
//!   particle simulations and vector fields have no Black Ops II
//!   counterpart and play as silent sound elements (the effect's other
//!   elements keep their places and counts);
//! - flag bits are the family's; AW's "drawn with the view model" bit
//!   (0x800, set on the parts of a flash that sit on the first-person gun)
//!   adds the one the runtime's sprite pass reads (0x1000).

use fastfile_s1::layout::{
    FxEffectDef as fd, FxElemDef as e, FxElemVelStateSample, FxElemVisStateSample,
    FxSparkFountainDef as sf, FxTrailDef as td, FxTrailVertex as tv,
};

use super::t6::{T6_FX_ELEM_DRAW_WITH_VIEWMODEL, T6_FX_ELEM_LINE};
use super::*;

/// One AW visual with its names resolved (see `fastfile_s1::walk::
/// FxVisualRec`). A material the lane could not bring over (it lives in a
/// zone not read, or it bends the picture behind it: distortion) is `None`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum S1FxVisual {
    #[default]
    None,
    Material(String),
    Model(String),
    Effect(String),
    Sound(String),
    Mark([Option<String>; 2]),
    /// A light definition, vector field or particle simulation.
    Other,
}

/// An AW trail: its `FxTrailDef` record, vertices and indices (bytes).
#[derive(Clone, Debug, Default)]
pub struct S1FxTrail {
    pub record: Vec<u8>,
    pub verts: Vec<u8>,
    pub inds: Vec<u8>,
}

/// One AW element (`FxElemDef`) as the lane read it.
#[derive(Clone, Debug, Default)]
pub struct S1FxElemView {
    /// The 320-byte record.
    pub record: Vec<u8>,
    pub vel_samples: Vec<u8>,
    pub vis_samples: Vec<u8>,
    pub visuals: Vec<S1FxVisual>,
    pub effect_on_impact: Option<String>,
    pub effect_on_death: Option<String>,
    pub effect_emitted: Option<String>,
    pub trail: Option<S1FxTrail>,
    /// A spark fountain's (or light's) extended record.
    pub extended: Vec<u8>,
}

/// One AW effect (`FxEffectDef`) as the lane read it.
#[derive(Clone, Debug, Default)]
pub struct S1FxView {
    pub name: String,
    /// The 64-byte header.
    pub header: Vec<u8>,
    pub elems: Vec<S1FxElemView>,
}

/// AW element types (`FxElemType`).
pub mod s1_elem_type {
    pub const BILLBOARD: u8 = 0;
    pub const ORIENTED: u8 = 1;
    pub const ROTATED: u8 = 2;
    pub const TAIL: u8 = 3;
    pub const LINE: u8 = 4;
    pub const TRAIL: u8 = 5;
    pub const FLARE: u8 = 6;
    pub const PARTICLE_SIM: u8 = 7;
    pub const CLOUD: u8 = 8;
    pub const SPARK_CLOUD: u8 = 9;
    pub const SPARK_FOUNTAIN: u8 = 10;
    pub const MODEL: u8 = 11;
    pub const OMNI_LIGHT: u8 = 12;
    pub const SPOT_LIGHT: u8 = 13;
    pub const SOUND: u8 = 14;
    pub const DECAL: u8 = 15;
    pub const RUNNER: u8 = 16;
    pub const VECTOR_FIELD: u8 = 17;
}

/// AW's element flag for parts drawn with the first-person gun.
const S1_FX_ELEM_DRAW_WITH_VIEWMODEL: i32 = 0x800;

/// AW element type -> the catalog's (IW4) type; `None` for a type Black
/// Ops II has no counterpart for (played as a silent sound element).
pub fn s1_elem_type(raw: u8) -> Option<u8> {
    use s1_elem_type as s;
    Some(match raw {
        s::BILLBOARD | s::CLOUD => elem_type::BILLBOARD,
        s::ORIENTED | s::ROTATED => elem_type::ORIENTED,
        s::TAIL | s::LINE => elem_type::TAIL,
        s::TRAIL => elem_type::TRAIL,
        s::SPARK_CLOUD => elem_type::SPARK_CLOUD,
        s::SPARK_FOUNTAIN => elem_type::SPARK_FOUNTAIN,
        s::MODEL => elem_type::MODEL,
        s::OMNI_LIGHT => elem_type::OMNI_LIGHT,
        s::SOUND => elem_type::SOUND,
        s::DECAL => elem_type::DECAL,
        s::RUNNER => elem_type::RUNNER,
        _ => return None,
    })
}

fn rd_i32(b: &[u8], at: usize) -> i32 {
    b.get(at..at + 4)
        .and_then(|s| s.try_into().ok())
        .map_or(0, i32::from_le_bytes)
}

fn rd_f32(b: &[u8], at: usize) -> f32 {
    b.get(at..at + 4)
        .and_then(|s| s.try_into().ok())
        .map_or(0.0, f32::from_le_bytes)
}

/// The element's flags in the runtime's terms (see the module docs).
pub fn s1_elem_flags(raw_flags: i32, raw_type: u8) -> i32 {
    let mut flags = raw_flags & !(T6_FX_ELEM_DRAW_WITH_VIEWMODEL | T6_FX_ELEM_LINE);
    // (The family's own bit stays too: the effect-model pass leaves such
    // models out, as it does an IW4 effect's.)
    if raw_flags & S1_FX_ELEM_DRAW_WITH_VIEWMODEL != 0 {
        flags |= T6_FX_ELEM_DRAW_WITH_VIEWMODEL;
    }
    if raw_type == s1_elem_type::LINE {
        flags |= T6_FX_ELEM_LINE;
    }
    flags
}

fn s1_elem_view(r: &[u8]) -> (FxElemDefView, [u8; 8]) {
    let f = |off: usize| rd_f32(r, off);
    let i = |off: usize| rd_i32(r, off);
    let range = |off: usize| [f(off), f(off + 4)];
    let b = |off: usize| r.get(off).copied().unwrap_or(0);
    let mut atlas = [0u8; 8];
    if let Some(src) = r.get(e::atlas..e::atlas + 8) {
        atlas.copy_from_slice(src);
    }
    let raw_type = b(e::elemType);
    let mapped = s1_elem_type(raw_type);
    let mid = |k: usize| f(e::collBounds__midPoint + 4 * k);
    let half = |k: usize| f(e::collBounds__halfSize + 4 * k);
    let view = FxElemDefView {
        flags: s1_elem_flags(i(e::flags), raw_type),
        spawn_a: i(e::spawn),
        spawn_b: i(e::spawn + 4),
        spawn_range_base: f(e::spawnRange),
        spawn_range_amplitude: f(e::spawnRange + 4),
        fade_in_range: range(e::fadeInRange),
        fade_out_range: range(e::fadeOutRange),
        spawn_frustum_cull_radius: f(e::spawnFrustumCullRadius),
        spawn_delay_msec_base: i(e::spawnDelayMsec),
        spawn_delay_msec_amplitude: i(e::spawnDelayMsec + 4),
        life_span_msec_base: i(e::lifeSpanMsec),
        life_span_msec_amplitude: i(e::lifeSpanMsec + 4),
        spawn_origin: core::array::from_fn(|a| range(e::spawnOrigin + a * 8)),
        spawn_offset_radius_base: f(e::spawnOffsetRadius),
        spawn_offset_radius_amplitude: f(e::spawnOffsetRadius + 4),
        spawn_offset_height_base: f(e::spawnOffsetHeight),
        spawn_offset_height_amplitude: f(e::spawnOffsetHeight + 4),
        spawn_angles: core::array::from_fn(|a| range(e::spawnAngles + a * 8)),
        angular_velocity: core::array::from_fn(|a| range(e::angularVelocity + a * 8)),
        initial_rotation: range(e::initialRotation),
        gravity_base: f(e::gravity),
        gravity_amplitude: f(e::gravity + 4),
        reflection_factor: range(e::reflectionFactor),
        coll_mins: core::array::from_fn(|k| mid(k) - half(k)),
        coll_maxs: core::array::from_fn(|k| mid(k) + half(k)),
        elem_type: mapped.unwrap_or(elem_type::SOUND),
        visual_count: if mapped.is_some() {
            b(e::visualCount)
        } else {
            1
        },
        vel_interval_count: b(e::velIntervalCount),
        vis_state_interval_count: b(e::visStateIntervalCount),
        lighting_frac: b(e::lightingFrac),
        use_item_clip: b(e::useItemClip),
        sort_order: b(e::sortOrder),
        emit_dist: range(e::emitDist),
        emit_dist_variance: range(e::emitDistVariance),
    };
    (view, atlas)
}

/// IW4's visual sample: per half (base, amplitude) colour bytes (blue,
/// green, red, alpha), rotation delta and total, size[2], scale.
const IW4_VIS_SAMPLE: usize = 48;

/// One AW visual sample (112 bytes) as IW4's (48): see the module docs for
/// the colour.
pub fn s1_vis_sample_to_iw4(s: &[u8]) -> [u8; IW4_VIS_SAMPLE] {
    use FxElemVisStateSample as v;
    let f = |at: usize| rd_f32(s, at);
    let lit = |color: usize, emissive: usize| -> [f32; 4] {
        let mut c = [f(color), f(color + 4), f(color + 8), f(color + 12)];
        for (k, ch) in c.iter_mut().take(3).enumerate() {
            let scale = f(emissive + 4 * k);
            if scale.is_finite() && scale > 0.0 {
                *ch *= scale;
            }
        }
        c
    };
    let base = lit(v::base__color, v::base__emissiveScale);
    let amp = lit(v::amplitude__color, v::amplitude__emissiveScale);
    // Brighter than white: scaled back keeping the hue (base and range by
    // the same factor).
    let peak = (0..3)
        .map(|k| base[k] + amp[k].max(0.0))
        .fold(0.0f32, f32::max);
    let squeeze = if peak > 1.0 && peak.is_finite() { 1.0 / peak } else { 1.0 };
    let byte = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut out = [0u8; IW4_VIS_SAMPLE];
    for (half, (color, src)) in [(base, v::base), (amp, v::amplitude)].into_iter().enumerate() {
        let o = half * IW4_VIS_SAMPLE / 2;
        let rgb = |k: usize| byte(color[k] * squeeze);
        out[o] = rgb(2);
        out[o + 1] = rgb(1);
        out[o + 2] = rgb(0);
        out[o + 3] = byte(color[3]);
        let rest = [
            v::base__rotationDelta,
            v::base__rotationTotal,
            v::base__size,
            v::base__size + 4,
            v::base__scale,
        ];
        for (k, at) in rest.iter().enumerate() {
            let x = f(src + at);
            out[o + 4 + 4 * k..o + 8 + 4 * k].copy_from_slice(&x.to_le_bytes());
        }
    }
    out
}

fn s1_material_visual(name: &str) -> OwnedFxVisual {
    if name.is_empty() {
        return OwnedFxVisual::None;
    }
    OwnedFxVisual::Material {
        material: FxElemMaterial::Unresolved(AssetEdgeReason::CatalogMiss),
        hint: Some(name.to_owned()),
        material_namespace: crate::AssetNamespace::S1,
        authored: AuthoredRef {
            slot: true,
            alias: false,
        },
    }
}

fn s1_visuals(el: &S1FxElemView, view: &FxElemDefView, supported: bool) -> Vec<OwnedFxVisual> {
    let t = view.elem_type;
    if !supported || matches!(t, elem_type::OMNI_LIGHT | elem_type::SPOT_LIGHT) || el.visuals.is_empty() {
        return vec![OwnedFxVisual::None];
    }
    el.visuals
        .iter()
        .map(|v| match v {
            S1FxVisual::Material(name) => s1_material_visual(name),
            S1FxVisual::Mark([a, b]) => mark_pair(
                s1_material_visual(a.as_deref().unwrap_or("")),
                s1_material_visual(b.as_deref().unwrap_or("")),
            ),
            S1FxVisual::Model(name) => model_visual(Some(name.clone())),
            S1FxVisual::Effect(name) => runner_visual(name.clone()),
            S1FxVisual::Sound(name) => sound_visual(name.clone()),
            S1FxVisual::Other | S1FxVisual::None => OwnedFxVisual::None,
        })
        .collect()
}

fn s1_trail(trail: &S1FxTrail) -> OwnedFxTrailDef {
    let r = &trail.record;
    OwnedFxTrailDef {
        scroll_time_msec: rd_i32(r, td::scrollTimeMsec),
        repeat_dist: rd_i32(r, td::repeatDist),
        inv_split_dist: rd_f32(r, td::invSplitDist),
        inv_split_arc_dist: rd_f32(r, td::invSplitArcDist),
        inv_split_time: rd_f32(r, td::invSplitTime),
        verts: trail
            .verts
            .chunks_exact(tv::SIZE)
            .map(|v| FxTrailVertex {
                pos: [rd_f32(v, tv::pos), rd_f32(v, tv::pos + 4)],
                normal: [rd_f32(v, tv::normal), rd_f32(v, tv::normal + 4)],
                tex_coord: rd_f32(v, tv::texCoord),
            })
            .collect(),
        inds: trail
            .inds
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect(),
    }
}

fn s1_spark_fountain(x: &[u8]) -> Option<OwnedFxSparkFountainDef> {
    if x.len() < sf::SIZE {
        return None;
    }
    Some(OwnedFxSparkFountainDef {
        gravity: rd_f32(x, sf::gravity),
        bounce_frac: rd_f32(x, sf::bounceFrac),
        bounce_rand: rd_f32(x, sf::bounceRand),
        spark_spacing: rd_f32(x, sf::sparkSpacing),
        spark_length: rd_f32(x, sf::sparkLength),
        spark_count: rd_i32(x, sf::sparkCount),
        loop_time: rd_f32(x, sf::loopTime),
        vel_min: rd_f32(x, sf::velMin),
        vel_max: rd_f32(x, sf::velMax),
        vel_cone_frac: rd_f32(x, sf::velConeFrac),
        rest_speed: rd_f32(x, sf::restSpeed),
        boost_time: rd_f32(x, sf::boostTime),
        boost_factor: rd_f32(x, sf::boostFactor),
    })
}

fn s1_elem(el: &S1FxElemView) -> OwnedFxElemDef {
    let raw_type = el.record.get(e::elemType).copied().unwrap_or(0);
    let supported = s1_elem_type(raw_type).is_some();
    let (view, atlas) = s1_elem_view(&el.record);
    let vel_count = usize::from(view.vel_interval_count) + 1;
    let vel_samples = el
        .vel_samples
        .get(..vel_count * FxElemVelStateSample::SIZE)
        .unwrap_or(&el.vel_samples)
        .to_vec();
    let vis_samples: Vec<u8> = el
        .vis_samples
        .chunks_exact(FxElemVisStateSample::SIZE)
        .take(usize::from(view.vis_state_interval_count) + 1)
        .flat_map(s1_vis_sample_to_iw4)
        .collect();
    let vel_graph_local = parse_vel_graph_channel(&vel_samples, vel_count, false);
    let vel_graph_world = parse_vel_graph_channel(&vel_samples, vel_count, true);
    let visuals = s1_visuals(el, &view, supported);
    let child = |name: &Option<String>| capture_named_child(name.clone().unwrap_or_default());
    let (effect_on_impact, effect_on_impact_hint) = child(&el.effect_on_impact);
    let (effect_on_death, effect_on_death_hint) = child(&el.effect_on_death);
    let (effect_emitted, effect_emitted_hint) = child(&el.effect_emitted);
    let trail_def = (view.elem_type == elem_type::TRAIL)
        .then(|| el.trail.as_ref().map(s1_trail))
        .flatten();
    let spark_fountain_def = (view.elem_type == elem_type::SPARK_FOUNTAIN)
        .then(|| s1_spark_fountain(&el.extended))
        .flatten();
    OwnedFxElemDef {
        raw: leftover_pack_iw4_elem_raw(&view, atlas),
        view,
        vel_samples,
        vel_graph_local,
        vel_graph_world,
        vis_samples,
        visuals,
        effect_on_impact,
        effect_on_impact_hint,
        effect_on_death,
        effect_on_death_hint,
        effect_emitted,
        effect_emitted_hint,
        has_extended: trail_def.is_some() || spark_fountain_def.is_some(),
        trail_def,
        spark_fountain_def,
        cloud_density: None,
    }
}

impl FxCatalog {
    /// bo2mp: one Advanced Warfare effect, under the S1 namespace. False
    /// for a reference stub (a comma-named effect another zone defines), an
    /// unnamed one, or one whose element counts do not add up.
    pub fn capture_s1(&mut self, fx: &S1FxView) -> bool {
        if fx.name.is_empty() || fx.name.starts_with(',') {
            return false;
        }
        let h = &fx.header;
        let view = FxEffectDefView {
            // (AW's effect flags are not IW4's: none kept.)
            flags: 0,
            msec_looping_life: rd_i32(h, fd::msecLoopingLife),
            looping_count: rd_i32(h, fd::elemDefCountLooping),
            one_shot_count: rd_i32(h, fd::elemDefCountOneShot),
            emission_count: rd_i32(h, fd::elemDefCountEmission),
        };
        let total = view.looping_count + view.one_shot_count + view.emission_count;
        if total < 0 || total as usize != fx.elems.len() {
            self.capture_gaps += 1;
            return false;
        }
        let elems: Vec<OwnedFxElemDef> = fx.elems.iter().map(s1_elem).collect();
        self.last_captured = Some(fx.name.clone());
        self.insert_owned(OwnedFxEffectDef {
            namespace: crate::AssetNamespace::S1,
            name: fx.name.clone(),
            view,
            header_raw: leftover_pack_iw4_effect_header(&view),
            elems,
        });
        true
    }
}
