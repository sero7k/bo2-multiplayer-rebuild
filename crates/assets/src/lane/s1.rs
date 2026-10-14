//! bo2mp: Call of Duty: Advanced Warfare's guns as guest guns in a Black
//! Ops II match (`aw_<gun>_mp`).
//!
//! `fastfile_s1`'s exact walk reads AW's `common_mp` (the weapons, their
//! animations, materials and streamed pictures) and each gun's transient
//! zones (`mp_vm_<gun>_tr` first person, `mp_wm_<gun>_tr` world), which
//! hold its models. AW builds a gun from parts: the weapon names a
//! composite model whose parts (receiver, stock, barrel, sight) each have
//! their own few bones; each part's first bone is a bone of the receiver
//! (`tag_loot`, `tag_range`, `j_gun`), so the parts join the receiver's
//! skeleton there, their vertices moved by that bone's bind pose (vertices
//! are stored in bind-pose model space; a part's root is its origin).
//!
//! The rest is the Ghosts lane's: models into Black Ops II's captured form
//! (`lane::iw6`'s vertex repack and blend fold), materials onto BO2's draw
//! path by technique-set name, animations in BO2's captured form, weapons
//! through `WeaponCatalog::capture_s1`. Arms are Black Ops II's.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use asset_anim::XAnimBuild;
use asset_core::{AssetEdge, AssetNamespace, AssetRef, ZoneOwner};
use asset_game::{
    FxCatalog, FxModelCatalog, S1FxElemView, S1FxTrail, S1FxView, S1FxVisual, S1WeaponView,
    TracerCatalog, WeaponBuild, WeaponCatalog,
};
use asset_material::{
    AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP,
    TS_NORMAL_MAP, TS_SPECULAR_MAP,
};
use asset_model::{FpvMeshBuild, WorldWeaponBuild};
use fastfile_s1::walk::{
    AssetId, Capture, FxVisualRec, ImageRec, MaterialRec, WeaponRec, XAnimRec, XModelRec,
};

use super::iw6::{fold_blend, gpu_image, iw6_model_draw, le_u16s, le_u32s};

const NS: AssetNamespace = AssetNamespace::S1;

/// `R_HashString` of the texture slots BO2's lit model draw reads (AW
/// hashes them the same way: `colorMap` is `0xa0ab1041` in both).
const COLOR_MAP_HASH: u32 = 0xa0ab_1041;
const NORMAL_MAP_HASH: u32 = 0x59d3_0d0f;
const SPECULAR_MAP_HASH: u32 = 0x34ec_ccb3;

/// The largest level kept of a gun's pictures (first person), and of the
/// third-person models' (`BO2MP_AW_TEXTURE=<size>` changes the first).
const FIRST_PERSON_MAX: u32 = 1024;
const WORLD_MAX: u32 = 512;
/// The largest level kept of an effect's pictures (and its models').
const FX_MAX: u32 = 512;

/// AW's own multiplayer guns (`iw5_<gun>_mp`), without the variants (loot,
/// gold, Juggernaut) and the killstreak and equipment weapons. Guns another
/// game already brings are left to it by Create-a-Class (`bo2_profile::
/// guest`).
pub(crate) const AW_GUNS: &[&str] = &[
    // Assault rifles.
    "bal27", "ak12", "arx160", "hbra3", "himar", "m182spr",
    // Submachine guns.
    "sn6", "mp11", "hmr9", "sac3", "asm1", "kf5",
    // Light machine guns.
    "pbw", "lsat", "asaw", "em1", "exoxmg",
    // Sniper rifles.
    "mors", "gm6", "thor",
    // Shotguns.
    "uts19", "maul", "m990", "vbr",
    // Pistols and launchers.
    "rhino", "titan45", "rw1", "maaws", "mahem", "stingerm7", "exocrossbow",
    "microdronelauncher", "epm3",
];

/// AW's guns, ready to join the match's catalogs.
pub(crate) struct S1Guests {
    pub weapons: WeaponBuild,
    pub fpv: FpvMeshBuild,
    pub world_guns: WorldWeaponBuild,
    pub xanims: XAnimBuild,
    pub material_seed: MaterialCatalog,
    /// bo2mp: the guns' own effects (flashes, shell ejects, trails, what
    /// their projectiles do), their models and tracers, under the S1
    /// namespace.
    pub fx: FxCatalog,
    pub fx_models: FxModelCatalog,
    pub tracers: TracerCatalog,
    pub report: Vec<String>,
}

impl S1Guests {
    pub(crate) fn empty(material_seed: MaterialCatalog, report: Vec<String>) -> Self {
        Self {
            weapons: WeaponBuild::default(),
            fpv: FpvMeshBuild::default(),
            world_guns: WorldWeaponBuild::default(),
            xanims: XAnimBuild::default(),
            material_seed,
            fx: FxCatalog::default(),
            fx_models: FxModelCatalog::default(),
            tracers: TracerCatalog::default(),
            report,
        }
    }
}

/// AW's `zone` folder, if AW is installed.
pub(crate) fn aw_zone_dir() -> Result<PathBuf, String> {
    let root = asset_transport::games_root_from_env().ok().map(|r| r.0);
    let extra: Vec<PathBuf> = asset_transport::game_paths::get("aw").into_iter().collect();
    let install = fastfile_s1::install::find_install(
        root.as_deref().filter(|p| !p.as_os_str().is_empty()),
        &extra,
    )
    .ok_or("Advanced Warfare is not installed (no zone/common_mp.ff in AW's format)")?;
    Ok(install.join("zone"))
}

/// One walked zone.
struct Walked {
    zone: fastfile_s1::Zone,
    cap: Capture,
}

fn walk(path: &Path) -> Result<Walked, String> {
    let zone = fastfile_s1::Zone::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut w = fastfile_s1::walk::Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
    let stop = w.walk().err();
    let cap = w.cap;
    if let Some(s) = stop {
        return Err(format!(
            "{} stopped at asset {} ({:?}): {}",
            path.display(),
            s.asset_index,
            s.asset_type,
            s.error
        ));
    }
    Ok(Walked { zone, cap })
}

/// A zone walked as far as it reads: the capture so far, and where it
/// stopped.
fn walk_partial(path: &Path) -> Result<(Walked, Option<String>), String> {
    let zone = fastfile_s1::Zone::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut w = fastfile_s1::walk::Walker::new(&zone.image, &zone.assets, &zone.header.stream_files);
    let stop = w.walk().err().map(|s| format!("asset {} ({:?}): {}", s.asset_index, s.asset_type, s.error));
    let cap = w.cap;
    Ok((Walked { zone, cap }, stop))
}

/// AW's common_mp and the guns' transient zones, read ahead of the build.
pub(crate) enum S1Prefetch {
    Off(Vec<String>),
    Ready(Box<S1Zones>),
}

pub(crate) struct S1Zones {
    dir: PathBuf,
    common: Walked,
    /// `code_post_gfx_mp`, walked as far as it reads (see `Builder::post`).
    post: Option<Walked>,
    /// Gun -> (first-person zone, world zone).
    parts: BTreeMap<String, (Option<Walked>, Option<Walked>)>,
    report: Vec<String>,
    started: std::time::Instant,
}

/// bo2mp: reads AW's zones for its guns (`wanted`: Black Ops II rules with
/// guest guns on), off the match's critical path.
pub(crate) fn prefetch_s1_guest_guns(wanted: bool) -> S1Prefetch {
    if !wanted {
        return S1Prefetch::Off(vec!["s1 guest guns: off".into()]);
    }
    let dir = match aw_zone_dir() {
        Ok(dir) => dir,
        Err(e) => return S1Prefetch::Off(vec![format!("s1 guest guns: {e}")]),
    };
    let started = std::time::Instant::now();
    let common = match walk(&dir.join("common_mp.ff")) {
        Ok(w) => w,
        Err(e) => return S1Prefetch::Off(vec![format!("s1 guest guns: {e}")]),
    };
    let mut report = vec![format!(
        "s1 guest guns: walked {} assets of {} ({} weapons, {} models, {} animations, {} materials, {} images) [{:.1} s]",
        common.cap.walked,
        dir.join("common_mp.ff").display(),
        common.cap.weapons.len(),
        common.cap.xmodels.len(),
        common.cap.xanims.len(),
        common.cap.materials.len(),
        common.cap.images.len(),
        started.elapsed().as_secs_f64(),
    )];
    // bo2mp: code_post_gfx_mp: the effects and materials common_mp only
    // references. Its walk stops near the end (at its leaderboards, after
    // every effect, material and impact table): what came before is kept.
    let post = match walk_partial(&dir.join("code_post_gfx_mp.ff")) {
        Ok((w, stop)) => {
            report.push(format!(
                "s1 guest guns: code_post_gfx_mp: walked {} assets ({} effects, {} materials, {} impact tables){}",
                w.cap.walked,
                w.cap.fx.len(),
                w.cap.materials.len(),
                w.cap.impact_tables.len(),
                stop.map(|s| format!("; stopped: {s}")).unwrap_or_default()
            ));
            Some(w)
        }
        Err(e) => {
            report.push(format!("s1 guest guns: code_post_gfx_mp: {e} (effects common_mp only references are left out)"));
            None
        }
    };
    let mut parts = BTreeMap::new();
    let mut failed = Vec::new();
    for gun in AW_GUNS {
        if parts.contains_key(*gun) {
            continue;
        }
        let mut one = |kind: &str| {
            let path = dir.join(format!("mp_{kind}_{gun}_tr.ff"));
            if !path.is_file() {
                failed.push(format!("{} missing", path.display()));
                return None;
            }
            walk(&path).map_err(|e| failed.push(e)).ok()
        };
        let vm = one("vm");
        let wm = one("wm");
        parts.insert((*gun).to_owned(), (vm, wm));
    }
    report.push(format!(
        "s1 guest guns: {} gun zone pairs read; {} problems {:?} [{:.1} s]",
        parts.len(),
        failed.len(),
        &failed[..failed.len().min(4)],
        started.elapsed().as_secs_f64()
    ));
    S1Prefetch::Ready(Box::new(S1Zones {
        dir,
        common,
        post,
        parts,
        report,
        started,
    }))
}

/// bo2mp: the match's AW guest guns from what `prefetch_s1_guest_guns`
/// read, their materials linked into `material_seed`.
pub(crate) fn build_s1_guest_guns(
    prefetch: S1Prefetch,
    material_seed: MaterialCatalog,
    progress: &asset_transport::LoadProgress,
) -> S1Guests {
    let zones = match prefetch {
        S1Prefetch::Off(report) => return S1Guests::empty(material_seed, report),
        S1Prefetch::Ready(zones) => zones,
    };
    let stage = progress.begin_scoped(asset_transport::StageId::CommonAssets, "s1_guest_weapons", None);
    let guests = load_s1_guests(*zones, material_seed);
    stage.done();
    guests
}

/// bo2mp: read and built in one go (`iw4l check-guns`).
pub(crate) fn walk_s1_guest_guns(
    wanted: bool,
    material_seed: MaterialCatalog,
    progress: &asset_transport::LoadProgress,
) -> S1Guests {
    build_s1_guest_guns(prefetch_s1_guest_guns(wanted), material_seed, progress)
}

/// A bind pose (`DObjAnimMat`: unit quaternion xyzw, translation).
#[derive(Clone, Copy, Debug)]
struct Pose {
    q: [f32; 4],
    t: [f32; 3],
}

impl Pose {
    const IDENTITY: Self = Self {
        q: [0.0, 0.0, 0.0, 1.0],
        t: [0.0; 3],
    };

    fn from_base_mat(m: &[f32; 8]) -> Self {
        let q = [m[0], m[1], m[2], m[3]];
        let len = q.iter().map(|c| c * c).sum::<f32>().sqrt();
        let q = if len > 1e-6 { q.map(|c| c / len) } else { [0.0, 0.0, 0.0, 1.0] };
        Self {
            q,
            t: [m[4], m[5], m[6]],
        }
    }

    fn rotate(&self, v: [f32; 3]) -> [f32; 3] {
        let [x, y, z, w] = self.q;
        // v + 2w (q x v) + 2 q x (q x v)
        let c1 = [y * v[2] - z * v[1], z * v[0] - x * v[2], x * v[1] - y * v[0]];
        let c2 = [y * c1[2] - z * c1[1], z * c1[0] - x * c1[2], x * c1[1] - y * c1[0]];
        std::array::from_fn(|k| v[k] + 2.0 * (w * c1[k] + c2[k]))
    }

    fn apply(&self, v: [f32; 3]) -> [f32; 3] {
        let r = self.rotate(v);
        std::array::from_fn(|k| r[k] + self.t[k])
    }

    /// `self` after `inner` (inner's pose placed in self's space).
    fn compose(&self, inner: &Pose) -> Pose {
        let [ax, ay, az, aw] = self.q;
        let [bx, by, bz, bw] = inner.q;
        Pose {
            q: [
                aw * bx + ax * bw + ay * bz - az * by,
                aw * by - ax * bz + ay * bw + az * bx,
                aw * bz + ax * by - ay * bx + az * bw,
                aw * bw - ax * bx - ay * by - az * bz,
            ],
            t: self.apply(inner.t),
        }
    }

    fn base_mat(&self) -> ([f32; 4], [f32; 3]) {
        (self.q, self.t)
    }
}

/// AW's packed unit vector (Ghosts': unsigned 10:10:10 over 1023) as a
/// vector, and a vector as Black Ops II's (signed 10:10:10 over 511).
fn unpack_unit(packed: u32) -> [f32; 3] {
    std::array::from_fn(|k| ((packed >> (10 * k)) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0)
}

fn pack_t6_unit(v: [f32; 3], high: u32) -> u32 {
    let mut out = high & !0x3fff_ffff;
    for (k, c) in v.iter().enumerate() {
        let s = (c.clamp(-1.0, 1.0) * 511.0).round() as i32;
        out |= ((s as u32) & 0x3ff) << (10 * k);
    }
    out
}

/// One AW vertex as Black Ops II's, moved by `pose` (a part joined to its
/// receiver).
fn s1_vertex_to_t6(v: &[u8], pose: &Pose) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..32].copy_from_slice(&v[..32]);
    let f = |at: usize| f32::from_le_bytes([v[at], v[at + 1], v[at + 2], v[at + 3]]);
    let p = pose.apply([f(0), f(4), f(8)]);
    for (k, c) in p.iter().enumerate() {
        out[4 * k..4 * k + 4].copy_from_slice(&c.to_le_bytes());
    }
    for at in [24usize, 28] {
        let packed = u32::from_le_bytes([v[at], v[at + 1], v[at + 2], v[at + 3]]);
        let n = pose.rotate(unpack_unit(packed));
        out[at..at + 4].copy_from_slice(&pack_t6_unit(n, packed).to_le_bytes());
    }
    out
}

/// A model built from AW parts (or one whole model) in BO2's terms: the
/// joined skeleton and the parts' first-LOD surfaces.
#[derive(Default)]
struct Joined {
    bone_names: Vec<String>,
    num_root_bones: u8,
    /// Per bone (index >= roots): the parent's index.
    parents: Vec<usize>,
    quats: Vec<[i16; 4]>,
    trans: Vec<[f32; 3]>,
    poses: Vec<Pose>,
    part_classification: Vec<u8>,
    lod0: Vec<asset_t6::XSurfaceRef>,
    /// Common_mp material index per surface.
    materials: Vec<Option<usize>>,
    mins: [f32; 3],
    maxs: [f32; 3],
}

struct Builder<'a> {
    common: &'a Walked,
    /// bo2mp: `code_post_gfx_mp` (zone 1), which defines the effects and
    /// materials common_mp only references (its impact effects, the shared
    /// smoke, glow and spark materials).
    post: Option<&'a Walked>,
    /// Its materials by name.
    post_material_names: HashMap<String, usize>,
    paks: fastfile_s1::image::ImagePaks,
    catalog: MaterialCatalog,
    owner: ZoneOwner,
    /// Common_mp materials by name.
    material_names: HashMap<String, usize>,
    /// (zone, material index) -> catalog material index.
    materials: HashMap<(u8, usize), usize>,
    /// (zone, image index, size cap) -> catalog image slot.
    images: HashMap<(u8, usize, u32), Option<usize>>,
    image_failures: Vec<String>,
    decoded: usize,
    /// Effect materials by name: usable (`true`) or left out.
    fx_materials: HashMap<String, bool>,
    /// Effect models by name: built (`true`) or left out.
    fx_model_names: HashMap<String, bool>,
}

/// An effect technique Black Ops II cannot draw: it bends or blurs the
/// picture behind it (AW's shockwaves, heat haze, the ADS blur) or dirties
/// the lens. Such visuals are left out, not drawn as flat sprites.
fn fx_technique_unsupported(technique_set: &str) -> bool {
    let t = technique_set.trim_start_matches(',');
    t.starts_with("distortion")
        || t.contains("radialblur")
        || t.contains("dustonlens")
}

fn string(strings: &[Option<String>], index: u32) -> String {
    strings
        .get(index as usize)
        .cloned()
        .flatten()
        .unwrap_or_default()
}

fn find_model<'a>(w: &'a Walked, name: &str) -> Option<&'a XModelRec> {
    w.cap
        .xmodels
        .iter()
        .find(|m| m.name.trim_start_matches(',') == name && m.lods.first().is_some_and(|l| l.surfs.is_some()))
}

impl<'a> Builder<'a> {
    /// Joins `part` (a model of `w`) into `j`: its root bone onto `j`'s bone
    /// of the same name (none yet: it starts the skeleton), its other bones
    /// appended, its first-LOD surfaces moved by the root's bind pose.
    fn join(&self, j: &mut Joined, w: &Walked, part: &XModelRec) -> bool {
        let strings = &w.zone.assets.script_strings;
        let names: Vec<String> = part.bone_names.iter().map(|&s| string(strings, s)).collect();
        let roots = usize::from(part.num_root_bones).max(1);
        let first = j.bone_names.is_empty();
        let attach = if first {
            Pose::IDENTITY
        } else {
            match names.first().and_then(|n| j.bone_names.iter().position(|b| b == n)) {
                Some(i) => j.poses[i],
                None => return false,
            }
        };
        // Part bone -> joined bone.
        let mut map: Vec<usize> = Vec::with_capacity(names.len());
        for (i, name) in names.iter().enumerate() {
            let pose = attach.compose(
                &part
                    .base_mat
                    .get(i)
                    .map_or(Pose::IDENTITY, Pose::from_base_mat),
            );
            if first {
                map.push(i);
                j.bone_names.push(name.clone());
                j.poses.push(pose);
                j.part_classification.push(part.part_classification.get(i).copied().unwrap_or(0));
                if i >= roots {
                    let step = usize::from(part.parent_list.get(i - roots).copied().unwrap_or(0));
                    j.parents.push(i.saturating_sub(step));
                    j.quats.push(part.tag_angles.get(i - roots).copied().unwrap_or([0, 0, 0, 0x7fff]));
                    j.trans.push(part.tag_positions.get(i - roots).copied().unwrap_or([0.0; 3]));
                }
                continue;
            }
            if let Some(at) = j.bone_names.iter().position(|b| b == name) {
                map.push(at);
                continue;
            }
            if i < roots {
                // (A second root that the receiver does not have: drawn
                // with the first.)
                map.push(map.first().copied().unwrap_or(0));
                continue;
            }
            let step = usize::from(part.parent_list.get(i - roots).copied().unwrap_or(0));
            let parent = map.get(i.saturating_sub(step)).copied().unwrap_or(0);
            map.push(j.bone_names.len());
            j.bone_names.push(name.clone());
            j.poses.push(pose);
            j.part_classification.push(0);
            j.parents.push(parent);
            j.quats.push(part.tag_angles.get(i - roots).copied().unwrap_or([0, 0, 0, 0x7fff]));
            j.trans.push(part.tag_positions.get(i - roots).copied().unwrap_or([0.0; 3]));
        }
        if first {
            j.num_root_bones = roots as u8;
        }
        let bone = |offset: u16| -> u16 {
            let i = usize::from(offset / 64);
            (map.get(i).copied().unwrap_or(0) * 64) as u16
        };
        let Some(lod) = part.lods.first() else { return true };
        let Some(surfs) = lod.surfs.and_then(|s| w.cap.model_surfs.get(s.index as usize)) else {
            return true;
        };
        let image = &w.zone.image[..];
        for (i, s) in surfs.surfs.iter().enumerate() {
            let verts: Vec<u8> = s
                .verts
                .map_or(&[][..], |sp| sp.get(image))
                .chunks_exact(32)
                .flat_map(|v| s1_vertex_to_t6(v, &attach))
                .collect();
            for v in verts.chunks_exact(32) {
                for k in 0..3 {
                    let c = f32::from_le_bytes(v[4 * k..4 * k + 4].try_into().unwrap_or_default());
                    j.mins[k] = j.mins[k].min(c);
                    j.maxs[k] = j.maxs[k].max(c);
                }
            }
            let indices = le_u16s(s.tris.map_or(&[][..], |sp| sp.get(image)));
            let mut words = le_u16s(s.blend_verts.map_or(&[][..], |sp| sp.get(image)));
            // Bone words (each vertex: a bone, then bone and weight pairs).
            let mut at = 0usize;
            for (bucket, &count) in s.blend_vert_counts.iter().enumerate() {
                for _ in 0..count {
                    let per = 1 + 2 * bucket;
                    for k in (0..per).filter(|k| k % 2 == 0) {
                        if let Some(word) = words.get_mut(at + k) {
                            *word = bone(*word);
                        }
                    }
                    at += per;
                }
            }
            let (blend_counts, blend) = fold_blend(s.blend_vert_counts, &words);
            let material = part
                .materials
                .get(usize::from(lod.surf_index) + i)
                .copied()
                .flatten()
                .and_then(|id| self.material_names.get(w.cap.name_of(id).trim_start_matches(',')).copied());
            let mut part_bits = [0u32; 6];
            part_bits.copy_from_slice(&s.part_bits[..6]);
            j.lod0.push(asset_t6::XSurfaceRef {
                verts,
                vert_count: s.vert_count,
                indices,
                vert_lists: s.rigid.iter().map(|&(b, n, t, c)| (bone(b), n, t, c)).collect(),
                flags: 0,
                material: None,
                blend_counts,
                blend,
                part_bits,
            });
            j.materials.push(material);
        }
        true
    }

    /// The model a weapon names, its parts joined, in Black Ops II's
    /// captured form with the common_mp materials its surfaces use.
    fn model_ref(&self, name: &str, parts_zone: Option<&Walked>) -> Option<(asset_t6::XModelRef, Vec<usize>)> {
        let common = self.common;
        let rec = common.cap.xmodels.iter().find(|m| m.name.trim_start_matches(',') == name)?;
        let part_names: Vec<String> = if rec.composite.is_empty() {
            vec![name.to_owned()]
        } else {
            rec.composite
                .iter()
                .flatten()
                .map(|id| common.cap.name_of(*id).trim_start_matches(',').to_owned())
                .collect()
        };
        let mut j = Joined {
            mins: [f32::MAX; 3],
            maxs: [f32::MIN; 3],
            ..Joined::default()
        };
        for part in &part_names {
            let found = parts_zone
                .and_then(|z| find_model(z, part).map(|m| (z, m)))
                .or_else(|| find_model(common, part).map(|m| (common, m)));
            if let Some((z, m)) = found {
                self.join(&mut j, z, m);
            }
        }
        if j.lod0.is_empty() || j.bone_names.is_empty() || j.bone_names.len() > 255 {
            return None;
        }
        // (Debug: `S1_DUMP_OBJ=<dir>` writes each joined model as OBJ.)
        #[cfg(test)]
        if let Some(dir) = std::env::var_os("S1_DUMP_OBJ") {
            let mut obj = String::new();
            let mut base = 1usize;
            for s in &j.lod0 {
                for v in s.verts.chunks_exact(32) {
                    let f = |a: usize| f32::from_le_bytes(v[a..a + 4].try_into().unwrap_or_default());
                    obj.push_str(&format!("v {} {} {}\n", f(0), f(4), f(8)));
                }
                for t in s.indices.chunks_exact(3) {
                    obj.push_str(&format!("f {} {} {}\n", base + t[0] as usize, base + t[1] as usize, base + t[2] as usize));
                }
                base += s.verts.len() / 32;
            }
            let _ = std::fs::write(Path::new(&dir).join(format!("{name}.obj")), obj);
        }
        let key = |k: usize| asset_t6::AssetKey {
            ty: fastfile_t6::AssetType::Material,
            index: k,
        };
        for (s, m) in j.lod0.iter_mut().zip(&j.materials) {
            s.material = m.map(key);
        }
        let keys: Vec<usize> = j.materials.iter().flatten().copied().collect();
        let n = j.bone_names.len();
        let roots = usize::from(j.num_root_bones);
        let parent_list = j
            .parents
            .iter()
            .enumerate()
            .map(|(k, &p)| (roots + k).saturating_sub(p).min(255) as u8)
            .collect();
        let radius = (0..3)
            .map(|k| j.mins[k].abs().max(j.maxs[k].abs()).powi(2))
            .sum::<f32>()
            .sqrt();
        Some((
            asset_t6::XModelRef {
                lighting_origin_offset: [0.0; 3],
                lighting_origin_range: 0.0,
                name: name.to_owned(),
                numsurfs: j.lod0.len().min(255) as u8,
                materials: j.materials.iter().map(|m| m.map(key)).collect(),
                num_bones: n as u8,
                num_lods: 1,
                mins: j.mins,
                maxs: j.maxs,
                radius,
                lods: vec![(0.0, j.lod0.len() as u16, 0)],
                lod0: j.lod0,
                bone_names: j.bone_names,
                num_root_bones: j.num_root_bones,
                parent_list,
                quats: j.quats,
                trans: j.trans,
                base_mat: j.poses.iter().map(Pose::base_mat).collect(),
                part_classification: j.part_classification,
                bone_info: vec![None; n],
                contents: 0,
                coll_lod: -1,
                coll_surfs: Vec::new(),
                phys_preset: None,
            },
            keys,
        ))
    }

    /// One picture into the catalog (`None` when its pixels are not here).
    fn image(&mut self, z: u8, index: usize, max: u32, semantic: u8) -> Option<usize> {
        if let Some(slot) = self.images.get(&(z, index, max)) {
            return *slot;
        }
        let zone = self.zone(z);
        let rec: &ImageRec = zone.cap.images.get(index)?;
        let slot = match fastfile_s1::image::image_pixels(rec, &zone.zone.image, &mut self.paks, max)
            .map(|px| fastfile_iw6::image::ImagePixels {
                width: px.width,
                height: px.height,
                levels: px.levels,
                dxgi_format: px.dxgi_format,
                data: px.data,
            })
            .and_then(|px| gpu_image(&px, semantic == TS_COLOR_MAP).map(|img| (px, img)))
        {
            Ok((px, gpu)) => {
                self.decoded += 1;
                Some(self.catalog.link_image(AuthoredImage {
                    namespace: NS,
                    name: AssetRef::Real(format!("{}#{max}", rec.name)),
                    map_type: rec.map_type,
                    semantic,
                    category: 0,
                    use_srgb_reads: semantic == TS_COLOR_MAP,
                    width: px.width as u16,
                    height: px.height as u16,
                    depth: 1,
                    level_count: px.levels as u8,
                    format: 0,
                    payload: Arc::new(Vec::new()),
                    decoded: Some(Arc::new(gpu)),
                    common_owned: false,
                    decoded_variant: None,
                    decoded_by: None,
                    pending_decode: None,
                }))
            }
            Err(e) => {
                self.image_failures.push(format!("{}: {e}", rec.name));
                None
            }
        };
        self.images.insert((z, index, max), slot);
        slot
    }

    /// Zone `z` (0 common_mp, 1 code_post_gfx_mp).
    fn zone(&self, z: u8) -> &'a Walked {
        match (z, self.post) {
            (1, Some(post)) => post,
            _ => self.common,
        }
    }

    /// One material into the catalog, with its colour, normal and specular
    /// maps.
    fn material(&mut self, index: usize, max: u32) -> Option<usize> {
        self.material_in(0, index, max)
    }

    fn material_in(&mut self, z: u8, index: usize, max: u32) -> Option<usize> {
        if let Some(&i) = self.materials.get(&(z, index)) {
            return Some(i);
        }
        let zone = self.zone(z);
        let m: &MaterialRec = zone.cap.materials.get(index)?;
        let technique_set = m
            .techset
            .map(|t| zone.cap.name_of(t).to_owned())
            .unwrap_or_default();
        let draw = iw6_model_draw(&technique_set, m.sort_key);
        let pick = |hash: u32| {
            m.textures
                .iter()
                .find(|t| t.name_hash == hash && t.image.is_some())
        };
        let mut slots = Vec::new();
        if let Some(t) = pick(COLOR_MAP_HASH).or_else(|| m.textures.iter().find(|t| t.semantic == 2 && t.image.is_some())) {
            slots.push((t.name_hash, t.sampler_state, TS_COLOR_MAP, t.image));
        }
        if !draw.unlit {
            for (hash, semantic) in [(NORMAL_MAP_HASH, TS_NORMAL_MAP), (SPECULAR_MAP_HASH, TS_SPECULAR_MAP)] {
                if let Some(t) = pick(hash) {
                    slots.push((t.name_hash, t.sampler_state, semantic, t.image));
                }
            }
        }
        let mut textures = Vec::new();
        for (hash, sampler_state, semantic, image) in slots {
            let Some(image) = image else { continue };
            if let Some(slot) = self.image(z, image.index as usize, max, semantic) {
                textures.push(MaterialTextureBinding {
                    name_hash: hash,
                    name_start: 0,
                    name_end: 0,
                    sampler_state,
                    semantic,
                    image: Some(slot),
                });
            }
        }
        let m: &MaterialRec = &zone.cap.materials[index];
        let index_in_catalog = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(m.name.clone()),
            namespace: NS,
            technique_set: AssetRef::default(),
            technique_set_edge: AssetEdge::Absent,
            draw_surf: 0,
            sort_key: m.sort_key,
            info_game_flags: 0,
            texture_atlas: None,
            surface_type_bits: None,
            t5_layered_surface_types: None,
            state_flags: 0,
            camera_region: 0,
            state_bits: Vec::new(),
            state_bits_entry: None,
            t5_state_bits_entry: None,
            iw5_state_bits_entry: None,
            technique_table: None,
            route: None,
            textures,
            constants: m
                .constants
                .iter()
                .map(|c| {
                    let mut short = [0u8; 12];
                    for (d, b) in short.iter_mut().zip(c.name.bytes()) {
                        *d = b;
                    }
                    asset_material::MaterialConstant {
                        name_hash: c.name_hash,
                        name: short,
                        literal: c.literal,
                    }
                })
                .collect(),
            zone: self.owner,
            t6_draw: Some(draw),
        });
        self.materials.insert((z, index), index_in_catalog);
        Some(index_in_catalog)
    }

    /// An animation in Black Ops II's captured form.
    fn anim_ref(&self, a: &XAnimRec) -> asset_t6::XAnimRef {
        let image = &self.common.zone.image[..];
        let strings = &self.common.zone.assets.script_strings;
        let bytes = |s: Option<fastfile_s1::walk::Span>| s.map_or(&[][..], |s| s.get(image));
        let indices = if a.numframes >= 256 {
            le_u16s(bytes(a.indices))
        } else {
            bytes(a.indices).iter().map(|&b| u16::from(b)).collect()
        };
        asset_t6::XAnimRef {
            delta_quat: Vec::new(),
            name: a.name.trim_start_matches(',').to_owned(),
            numframes: a.numframes,
            framerate: a.framerate,
            looping: a.flags & 1 != 0,
            delta: false,
            delta_trans: Vec::new(),
            bone_count: a.bone_count,
            names: a.names.iter().map(|&s| string(strings, s)).collect(),
            notifies: a.notify.iter().map(|&(s, t)| (string(strings, s), t)).collect(),
            data_byte: bytes(a.data_byte).to_vec(),
            data_short: le_u16s(bytes(a.data_short)),
            data_int: le_u32s(bytes(a.data_int)),
            random_data_byte: bytes(a.random_data_byte).to_vec(),
            random_data_short: le_u16s(bytes(a.random_data_short)),
            random_data_int: le_u32s(bytes(a.random_data_int)),
            indices,
        }
    }

    fn name(&self, id: AssetId) -> String {
        self.common.cap.name_of(id).trim_start_matches(',').to_owned()
    }

    fn technique_of(&self, index: usize) -> String {
        self.technique_in(0, index)
    }

    fn technique_in(&self, z: u8, index: usize) -> String {
        let zone = self.zone(z);
        zone.cap
            .materials
            .get(index)
            .and_then(|m| m.techset)
            .map(|t| zone.cap.name_of(t).to_owned())
            .unwrap_or_default()
    }

    /// An effect's material by name, linked into the catalog: `None` when
    /// common_mp only references it (it lives in `code_post_gfx_mp`), its
    /// technique cannot be drawn here, or it has no colour map.
    fn fx_material(&mut self, z: u8, id: AssetId) -> Option<String> {
        let name = self.zone(z).cap.name_of(id).trim_start_matches(',').to_owned();
        if let Some(&ok) = self.fx_materials.get(&name) {
            return ok.then_some(name);
        }
        let ok = (|| {
            // (Defined in common_mp, else in code_post_gfx_mp.)
            let (z, index) = match self.material_names.get(&name) {
                Some(&i) => (0, i),
                None => (1, *self.post_material_names.get(&name)?),
            };
            if fx_technique_unsupported(&self.technique_in(z, index)) {
                return None;
            }
            let slot = self.material_in(z, index, FX_MAX)?;
            self.catalog.materials[slot]
                .textures
                .iter()
                .any(|t| t.semantic == TS_COLOR_MAP && t.image.is_some())
                .then_some(())
        })()
        .is_some();
        self.fx_materials.insert(name.clone(), ok);
        ok.then_some(name)
    }

    /// An effect's model (a shell casing, a glow on the gun) into
    /// `models`: false when it is not in common_mp or draws only with
    /// techniques left out (`fx_technique_unsupported`).
    fn fx_model(&mut self, name: &str, models: &mut FxModelCatalog) -> bool {
        if let Some(&ok) = self.fx_model_names.get(name) {
            return ok;
        }
        let ok = (|| {
            let (xref, keys) = self.model_ref(name, None)?;
            if !keys.is_empty() && keys.iter().all(|&k| fx_technique_unsupported(&self.technique_of(k))) {
                return None;
            }
            let keys: Vec<usize> = keys
                .into_iter()
                .filter(|&k| !fx_technique_unsupported(&self.technique_of(k)))
                .collect();
            let linked: HashMap<usize, usize> = keys
                .into_iter()
                .filter_map(|k| Some((k, self.material(k, FX_MAX)?)))
                .collect();
            let skel = asset_model::capture_model_skel_t6(&xref, |k| linked.get(&k.index).copied())?;
            models.insert_skel(skel, &self.catalog);
            Some(())
        })()
        .is_some();
        self.fx_model_names.insert(name.to_owned(), ok);
        ok
    }

    /// One walked effect with its names resolved (materials and models
    /// brought over as they are met).
    fn fx_view(&mut self, z: u8, f: &fastfile_s1::walk::FxRec, models: &mut FxModelCatalog) -> S1FxView {
        let zone = self.zone(z);
        let image = &zone.zone.image[..];
        let bytes = |s: Option<fastfile_s1::walk::Span>| s.map_or(Vec::new(), |s| s.get(image).to_vec());
        let mut elems = Vec::with_capacity(f.elems.len());
        for el in &f.elems {
            let mut visuals = Vec::with_capacity(el.visuals.len());
            for v in &el.visuals {
                visuals.push(match v {
                    FxVisualRec::None => S1FxVisual::None,
                    FxVisualRec::Other => S1FxVisual::Other,
                    FxVisualRec::Material(id) => self.fx_material(z, *id).map_or(S1FxVisual::None, S1FxVisual::Material),
                    FxVisualRec::Mark(pair) => S1FxVisual::Mark(pair.map(|id| id.and_then(|id| self.fx_material(z, id)))),
                    FxVisualRec::Model(id) => {
                        let name = zone.cap.name_of(*id).trim_start_matches(',').to_owned();
                        if self.fx_model(&name, models) {
                            S1FxVisual::Model(name)
                        } else {
                            S1FxVisual::None
                        }
                    }
                    FxVisualRec::Effect(name) => S1FxVisual::Effect(name.trim_start_matches(',').to_owned()),
                    FxVisualRec::Sound(name) => S1FxVisual::Sound(name.clone()),
                });
            }
            let bytes = |s: Option<fastfile_s1::walk::Span>| s.map_or(Vec::new(), |s| s.get(image).to_vec());
            let child = |n: &Option<String>| n.as_ref().map(|n| n.trim_start_matches(',').to_owned());
            elems.push(S1FxElemView {
                record: el.record.get(image).to_vec(),
                vel_samples: bytes(el.vel_samples),
                vis_samples: bytes(el.vis_samples),
                visuals,
                effect_on_impact: child(&el.effect_on_impact),
                effect_on_death: child(&el.effect_on_death),
                effect_emitted: child(&el.effect_emitted),
                trail: el.trail.map(|t| S1FxTrail {
                    record: t.record.get(image).to_vec(),
                    verts: bytes(t.verts),
                    inds: bytes(t.inds),
                }),
                extended: bytes(el.extended),
            });
        }
        S1FxView {
            name: f.name.clone(),
            header: bytes(Some(f.record)),
            elems,
        }
    }

    fn weapon_view(&self, w: &WeaponRec) -> S1WeaponView {
        let image = &self.common.zone.image[..];
        let strings = &self.common.zone.assets.script_strings;
        let name_of = |id: &Option<AssetId>| id.map(|id| self.name(id)).filter(|n| !n.is_empty());
        let anims = if w.anims.iter().any(Option::is_some) {
            &w.anims
        } else {
            &w.anims_right
        };
        S1WeaponView {
            name: w.name.clone(),
            display_name: w.display_name.clone(),
            def: w.def.get(image).to_vec(),
            gun_model: w.gun_models.first().and_then(name_of),
            world_model: w.world_models.first().and_then(name_of),
            xanims: anims
                .iter()
                .map(|a| name_of(a).unwrap_or_default().to_ascii_lowercase())
                .collect(),
            hide_tags: w
                .hide_tags
                .iter()
                .map(|&s| string(strings, s))
                .filter(|s| !s.is_empty())
                .collect(),
            alt_weapon_name: w.alt_weapon_name.clone(),
            sounds: w.sounds.clone(),
            fx: w.fx.iter().map(|(o, id)| (*o, self.name(*id))).collect(),
            models: w.models.iter().map(|(o, id)| (*o, self.name(*id))).collect(),
            materials: w.materials.iter().map(|(o, id)| (*o, self.name(*id))).collect(),
            notetrack_sounds: w
                .notetrack_sound_keys
                .iter()
                .zip(&w.notetrack_sound_values)
                .map(|(&k, &v)| (string(strings, k), string(strings, v)))
                .filter(|(k, v)| !k.is_empty() && !v.is_empty())
                .collect(),
            location_damage: w.location_damage.map(|s| {
                s.get(image)
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect()
            }),
            tracer: w.tracer.map(|id| self.name(id)).filter(|n| !n.is_empty()),
            impact_row: Vec::new(),
        }
    }
}

/// AW's impact table row for an AW `ImpactType` (its rows measured on
/// code_post_gfx_mp: small 0, large 2, explosive 4, shotgun 5, armour
/// piercing 9, energy shotgun 8, laser 16).
fn s1_impact_row(impact_type: i32) -> Option<usize> {
    Some(match impact_type {
        1 => 0,
        2 => 2,
        3 => 9,
        4 => 4,
        5 => 5,
        6 => 7,
        7 => 8,
        12 => 16,
        _ => return None,
    })
}

/// The AW impact types a guest gun keeps its own impacts for: those Black
/// Ops II has nothing like (the Tac-19's energy shotgun, the EM1's laser).
/// Every other AW gun hits as the Black Ops II bullet, shotgun or explosion
/// it maps to (`asset_game::remap_s1_impact_type`), as Black Ops II's own
/// guns do on the same map.
fn s1_keeps_own_impacts(impact_type: i32) -> bool {
    matches!(impact_type, 7 | 12)
}

/// Our surface type (IW4 order, `weapon_iw4::SURFACE_TYPE_NAMES`) -> AW's
/// impact table column (measured: AW adds a second metal at 14 and moves
/// cushion to 52; flesh is 53..56).
fn s1_impact_column(surf: usize) -> usize {
    match surf {
        0..=13 => surf,
        14..=25 => surf + 1,
        26 => 52,
        _ => surf,
    }
}

const S1_IMPACT_COLUMNS: usize = 60;
/// Our impact surfaces (`fx_iw4::FX_IMPACT_NONFLESH_COUNT`).
const S1_IMPACT_SURFACES: usize = 31;
const S1_IMPACT_FLESH: usize = 53;

/// bo2mp: the guns' effects: every effect a gun names (flashes, shell
/// ejects, projectile trails and explosions), the impacts of the guns that
/// keep their own (`s1_keeps_own_impacts`), and every effect those play
/// (runners, impact, death and emitted effects), with their materials and
/// models; and the guns' tracers. Also each such gun's impact row (31
/// surfaces then 4 flesh cells, `WeaponCombatFx::impact_row`).
fn build_s1_effects(
    b: &mut Builder<'_>,
    guns: &[(&str, &WeaponRec)],
    report: &mut Vec<String>,
) -> (FxCatalog, FxModelCatalog, TracerCatalog, HashMap<String, Vec<String>>) {
    // Effects by name: common_mp's own first, then code_post_gfx_mp's.
    let mut by_name: HashMap<String, (u8, usize)> = HashMap::new();
    for z in [1u8, 0] {
        let zone = b.zone(z);
        if z == 1 && b.post.is_none() {
            continue;
        }
        for (i, f) in zone.cap.fx.iter().enumerate() {
            if !f.name.starts_with(',') {
                by_name.insert(f.name.clone(), (z, i));
            }
        }
    }
    let mut todo: Vec<String> = guns
        .iter()
        .flat_map(|(_, w)| w.fx.iter().map(|(_, id)| b.name(*id)))
        .collect();

    // The impact rows of the guns that keep their own impacts.
    let mut impact_rows: HashMap<String, Vec<String>> = HashMap::new();
    let table = b.post.and_then(|post| {
        post.cap
            .impact_tables
            .iter()
            .find(|t| !t.name.contains("paintball") && t.cells.len() >= 19 * S1_IMPACT_COLUMNS)
            .map(|t| (post, t))
    });
    for (gun, w) in guns {
        let raw = {
            let def = w.def.get(&b.common.zone.image);
            def.get(fastfile_s1::layout::WeaponDef::impactType..)
                .and_then(|d| d.get(..4))
                .map_or(0, |d| i32::from_le_bytes([d[0], d[1], d[2], d[3]]))
        };
        if !s1_keeps_own_impacts(raw) {
            continue;
        }
        let (Some((post, table)), Some(row)) = (table, s1_impact_row(raw)) else {
            continue;
        };
        let cell = |col: usize| {
            table
                .cells
                .get(row * S1_IMPACT_COLUMNS + col)
                .copied()
                .flatten()
                .map(|id| post.cap.name_of(id).trim_start_matches(',').to_owned())
                .unwrap_or_default()
        };
        let cells: Vec<String> = (0..S1_IMPACT_SURFACES)
            .map(|surf| cell(s1_impact_column(surf)))
            .chain((0..4).map(|k| cell(S1_IMPACT_FLESH + k)))
            .collect();
        todo.extend(cells.iter().filter(|n| !n.is_empty()).cloned());
        impact_rows.insert((*gun).to_owned(), cells);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut missing = std::collections::BTreeSet::new();
    let mut order = Vec::new();
    while let Some(name) = todo.pop() {
        if name.is_empty() || !seen.insert(name.clone()) {
            continue;
        }
        let Some(&(z, i)) = by_name.get(name.as_str()) else {
            missing.insert(name);
            continue;
        };
        order.push((z, i));
        for el in &b.zone(z).cap.fx[i].elems {
            todo.extend(el.effects.iter().map(|n| n.trim_start_matches(',').to_owned()));
        }
    }
    let mut fx = FxCatalog::default();
    fx.set_capture_ns(NS);
    fx.set_capture_zone(b.owner);
    let mut models = FxModelCatalog::default();
    models.set_capture_ns(NS);
    models.set_capture_zone(b.owner);
    let (mut captured, mut elems, mut dropped_visuals, mut from_post) = (0usize, 0usize, 0usize, 0usize);
    for &(z, i) in &order {
        let rec = &b.zone(z).cap.fx[i];
        let view = b.fx_view(z, rec, &mut models);
        elems += view.elems.len();
        dropped_visuals += view
            .elems
            .iter()
            .zip(&rec.elems)
            .map(|(v, r)| {
                v.visuals
                    .iter()
                    .zip(&r.visuals)
                    .filter(|(a, b)| matches!(a, S1FxVisual::None) && !matches!(b, FxVisualRec::None | FxVisualRec::Other))
                    .count()
            })
            .sum::<usize>();
        let ok = fx.capture_s1(&view);
        captured += usize::from(ok);
        from_post += usize::from(ok && z == 1);
    }
    let fx_material_n = b.fx_materials.values().filter(|&&ok| ok).count();
    report.push(format!(
        "s1 guest guns: effects: {captured} of {} named captured ({from_post} from code_post_gfx_mp; {elems} elements; {} not found {:?}); {fx_material_n} of {} effect materials and {} of {} effect models brought over; {dropped_visuals} visuals left out (distortion/blur/lens dirt, no colour map, not found); own impact rows: {:?}",
        order.len() + missing.len(),
        missing.len(),
        missing.iter().take(6).collect::<Vec<_>>(),
        b.fx_materials.len(),
        models.len(),
        b.fx_model_names.len(),
        impact_rows.keys().collect::<Vec<_>>(),
    ));

    // Tracers (common_mp's, else code_post_gfx_mp's).
    let mut tracers = TracerCatalog::default();
    tracers.set_capture_ns(NS);
    tracers.set_capture_zone(b.owner);
    let wanted: std::collections::BTreeSet<String> = guns
        .iter()
        .filter_map(|(_, w)| w.tracer.map(|id| b.name(id)))
        .filter(|n| !n.is_empty())
        .collect();
    let mut tracer_n = 0usize;
    for name in &wanted {
        let found = [0u8, 1].into_iter().find_map(|z| {
            let zone = b.zone(z);
            (z == 0 || b.post.is_some())
                .then(|| zone.cap.tracers.iter().find(|t| t.name == *name).map(|t| (z, t)))
                .flatten()
        });
        let Some((z, t)) = found else {
            continue;
        };
        let material = t
            .material
            .and_then(|id| b.fx_material(z, id))
            .unwrap_or_default();
        let record = t.record.get(&b.zone(z).zone.image).to_vec();
        tracer_n += usize::from(tracers.capture_s1(&t.name, &material, &record));
    }
    report.push(format!(
        "s1 guest guns: tracers: {tracer_n} of {} named ({:?})",
        wanted.len(),
        wanted.iter().take(8).collect::<Vec<_>>()
    ));
    (fx, models, tracers, impact_rows)
}

/// AW's guns from its walked zones, their materials linked into
/// `material_seed`.
fn load_s1_guests(z: S1Zones, material_seed: MaterialCatalog) -> S1Guests {
    let S1Zones {
        dir,
        common,
        post,
        parts,
        mut report,
        started,
    } = z;
    let mut b = Builder {
        common: &common,
        post: post.as_ref(),
        post_material_names: post
            .as_ref()
            .map(|p| {
                p.cap
                    .materials
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| !m.name.starts_with(','))
                    .map(|(i, m)| (m.name.clone(), i))
                    .collect()
            })
            .unwrap_or_default(),
        paks: fastfile_s1::image::ImagePaks::new(&dir),
        catalog: material_seed,
        owner: ZoneOwner::from_zone_path(&dir.join("common_mp.ff")),
        material_names: common
            .cap
            .materials
            .iter()
            .enumerate()
            .filter(|(_, m)| !m.name.starts_with(','))
            .map(|(i, m)| (m.name.clone(), i))
            .collect(),
        materials: HashMap::new(),
        images: HashMap::new(),
        image_failures: Vec::new(),
        decoded: 0,
        fx_materials: HashMap::new(),
        fx_model_names: HashMap::new(),
    };
    let first_person_max = std::env::var("BO2MP_AW_TEXTURE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(FIRST_PERSON_MAX);

    let guns: Vec<(&str, &WeaponRec)> = AW_GUNS
        .iter()
        .filter_map(|g| {
            let want = format!("iw5_{g}_mp");
            common.cap.weapons.iter().find(|w| w.name.eq_ignore_ascii_case(&want)).map(|w| (*g, w))
        })
        .collect();

    let mut fpv = FpvMeshBuild::default();
    fpv.set_capture_ns(NS);
    let mut world_guns = WorldWeaponBuild::default();
    world_guns.set_capture_ns(NS);
    let mut unreadable = Vec::new();
    let mut done = std::collections::HashSet::new();
    for (gun, w) in &guns {
        let (vm, wm) = parts.get(*gun).map_or((None, None), |(a, b)| (a.as_ref(), b.as_ref()));
        let first = w.gun_models.first().copied().flatten().map(|id| b.name(id));
        let world = w.world_models.first().copied().flatten().map(|id| b.name(id));
        for (name, zone, max, first_person) in [
            (first, vm, first_person_max, true),
            (world, wm, WORLD_MAX, false),
        ] {
            let Some(name) = name else { continue };
            if !done.insert((name.clone(), first_person)) {
                continue;
            }
            let Some((xref, keys)) = b.model_ref(&name, zone) else {
                unreadable.push(format!("{name} (no parts read)"));
                continue;
            };
            let linked: HashMap<usize, usize> = keys
                .into_iter()
                .filter_map(|k| Some((k, b.material(k, max)?)))
                .collect();
            match asset_model::capture_model_skel_t6(&xref, |k| linked.get(&k.index).copied()) {
                Some(skel) if first_person => fpv.insert_in(NS, skel, Some(&b.catalog)),
                Some(skel) => world_guns.insert_in(NS, skel, Some(&b.catalog)),
                None => unreadable.push(format!("{name} (unreadable)")),
            }
        }
    }
    report.push(format!(
        "s1 guest guns: models: {} first person, {} third person built; {} not {:?}",
        fpv.len(),
        world_guns.len(),
        unreadable.len(),
        &unreadable[..unreadable.len().min(8)]
    ));
    report.push(format!(
        "s1 guest guns: materials: {} linked, {} pictures decoded, {} failed {:?}",
        b.materials.len(),
        b.decoded,
        b.image_failures.len(),
        &b.image_failures[..b.image_failures.len().min(6)]
    ));

    // Animations every gun names.
    let mut xanims = XAnimBuild::default();
    xanims.set_capture_ns(NS);
    let mut wanted: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for (_, w) in &guns {
        for id in w.anims.iter().chain(&w.anims_right).flatten() {
            wanted.insert(id.index);
        }
    }
    let mut anim_n = 0usize;
    for &ai in &wanted {
        if let Some(a) = common.cap.xanims.get(ai as usize) {
            anim_n += usize::from(xanims.insert_t6_as(NS, &b.anim_ref(a)));
        }
    }
    report.push(format!(
        "s1 guest guns: animations: {anim_n} of {} named kept ({} gaps)",
        wanted.len(),
        xanims.capture_gaps
    ));

    // The guns' effects, then the weapon catalog (whose combat effects
    // name them).
    let (fx, fx_models, tracers, impact_rows) = build_s1_effects(&mut b, &guns, &mut report);
    let mut catalog = WeaponCatalog::default();
    catalog.set_capture_ns(NS);
    for (gun, w) in &guns {
        let mut view = b.weapon_view(w);
        view.impact_row = impact_rows.get(*gun).cloned().unwrap_or_default();
        catalog.capture_s1(&view);
    }
    let captured = catalog.len();
    let mut weapons = catalog.into_build();
    weapons.stamp_namespace(NS);
    weapons.resolve_sz_xanim_edges(&xanims);
    weapons.resolve_fpv_mesh_edges(&fpv);
    weapons.resolve_world_model_edges(&world_guns);
    report.push(format!(
        "s1 guest guns: {captured} weapons captured, {} in the catalog [{:.1} s]",
        weapons.len(),
        started.elapsed().as_secs_f64()
    ));
    let Builder {
        catalog: material_seed,
        ..
    } = b;
    S1Guests {
        weapons,
        fpv,
        world_guns,
        xanims,
        material_seed,
        fx,
        fx_models,
        tracers,
        report,
    }
}

/// bo2mp `iw4l check-guns`: every AW gun animation decoded by the engine's
/// IW-family decoder, as `lane::iw6`'s check.
pub(crate) fn s1_anim_check(xanims: &XAnimBuild) -> String {
    let (mut ok, mut bad) = (0usize, Vec::new());
    let names: Vec<String> = xanims.names().map(str::to_owned).collect();
    for name in &names {
        match xanims
            .get(NS, name)
            .map(|c| xmodel_runtime::AnimClip::from_parts(&c.parts))
        {
            Some(Ok(_)) => ok += 1,
            Some(Err(e)) => bad.push(format!("{name}: {e:?}")),
            None => {}
        }
    }
    format!(
        "s1 animations: {ok} decode, {} refused{}",
        bad.len(),
        if bad.is_empty() {
            String::new()
        } else {
            format!(" ({})", bad.iter().take(3).cloned().collect::<Vec<_>>().join("; "))
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `IW4L_AW_DIR=<aw> cargo test -p assets aw_guns -- --ignored
    /// --nocapture`: AW's guns read and built as a match does.
    #[test]
    #[ignore]
    fn aw_guns() {
        if std::env::var_os("IW4L_AW_DIR").is_none() {
            return;
        }
        let g = walk_s1_guest_guns(true, MaterialCatalog::default(), &asset_transport::LoadProgress::default());
        for line in &g.report {
            println!("{line}");
        }
        println!("{}", s1_anim_check(&g.xanims));
        for i in 1..=g.weapons.len() as u32 {
            let anims = g.weapons.sz_xanims_of(i).map_or(0, |a| a.iter().flatten().count());
            println!(
                "  {} gun {:?} world {:?} anims {anims}",
                g.weapons.name_of(i),
                g.weapons.gun_xmodel_of(i),
                g.weapons.world_model_of(i)
            );
        }
        assert!(g.fpv.len() > 20, "first-person models: {}", g.fpv.len());
    }

    #[test]
    fn poses_compose_and_move_points() {
        // A quarter turn about z then a shift.
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let p = Pose {
            q: [0.0, 0.0, h, h],
            t: [10.0, 0.0, 0.0],
        };
        let v = p.apply([1.0, 0.0, 0.0]);
        assert!((v[0] - 10.0).abs() < 1e-5 && (v[1] - 1.0).abs() < 1e-5);
        let twice = p.compose(&p);
        let v = twice.apply([1.0, 0.0, 0.0]);
        // Inner shift (10,0,0) turned by a quarter: (0,10,0) + (10,0,0);
        // the point turned a half: (-1,0,0).
        assert!((v[0] - 9.0).abs() < 1e-4 && (v[1] - 10.0).abs() < 1e-4, "{v:?}");
    }

    #[test]
    fn unit_vectors_turn_and_repack() {
        let up = (1023u32 << 20) | (512 << 10) | 512;
        let n = unpack_unit(up);
        assert!(n[2] > 0.99);
        let packed = pack_t6_unit(n, 0);
        assert_eq!((packed >> 20) & 0x3ff, 511);
    }
}
