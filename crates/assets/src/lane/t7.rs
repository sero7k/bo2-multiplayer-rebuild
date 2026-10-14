//! bo2mp: Call of Duty: Black Ops III's guns as guest guns in a Black Ops
//! II match (`bo3_<gun>_mp`) and in Zombies (`<gun>_upgraded` for
//! Pack-a-Punch).
//!
//! Every Black Ops III gun is in `zone/core_common.ff` (campaign `_cp`,
//! multiplayer `_mp` and zombies `_zm` variants sharing models, animations
//! and pictures); streamed meshes and pictures are in the `.xpak` packages
//! beside it. `fastfile_t7`'s walk reads the zone (it stops near the end,
//! at the character customization tables, after every gun); this turns
//! what it kept into the engine's catalogs the way Black Ops II's own guns
//! go in, Black Ops III being Black Ops II's engine grown:
//!
//! - models: the densest LOD (the model's list runs coarsest first), its
//!   vertices as Black Ops II's packed ones, its skin weights as BO2's
//!   blend buckets (vertices ordered one to four bones), its skeleton (the
//!   rest pose's quaternions are stored as halves in Black Ops III);
//! - materials: the colour map, the normal map (BC7 with x, y in red and
//!   green: turned into BC5 for BO2's draw), the specular colour with the
//!   gloss in its alpha (one BC3: BO2's specular map);
//! - animations: Black Ops II's part types and packing, but every
//!   quaternion stored as four halves (turned into BO2's 16-bit
//!   fractions), the bone counts 16-bit; notetracks are (kind, parameter)
//!   pairs (`sound`, `fly_arak_mag_out`), turned into Black Ops II's
//!   inline sound and rumble notes;
//! - weapons: Black Ops III's numbers copied into Black Ops II's weapon
//!   structs by field name (`t7_fields`) and read by Black Ops II's own
//!   capture (`WeaponCatalog::capture_t6`), the fire type and player
//!   animation type renumbered.
//!
//! Effects and tracers are Black Ops II's (the weapon registry gives each
//! Black Ops III gun a Black Ops II gun's of its class); sounds are Black
//! Ops III's own (`asset_audio::bo3_weapon_sounds`). Arms are Black Ops
//! II's.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use asset_anim::XAnimBuild;
use asset_core::{AssetEdge, AssetNamespace, AssetRef, ZoneOwner};
use asset_game::{WeaponBuild, WeaponCatalog};
use asset_material::{
    AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP, TS_NORMAL_MAP,
    TS_SPECULAR_MAP,
};
use asset_model::{FpvMeshBuild, WorldWeaponBuild};
use fastfile_t6::layout::{WeaponDef as d6, WeaponVariantDef as v6};
use fastfile_t7::texture::{self, ImageLevels};
use fastfile_t7::walk::capture::{AnimRec, Capture, FieldValue, ModelRec, WeaponRec};
use fastfile_t7::xpak::XpakSet;

use super::t7_fields::T7_TO_T6;
use super::t7_map::{f16_bits, t6_unit_vec, t7_draw};

const NS: AssetNamespace = AssetNamespace::T7;

/// `R_HashString` of the texture slots Black Ops II's lit model draw reads
/// (Black Ops III names its colour and normal maps the same way).
const COLOR_MAP_HASH: u32 = 0xa0ab_1041;
const NORMAL_MAP_HASH: u32 = 0x59d3_0d0f;
const SPECULAR_MAP_HASH: u32 = 0x34ec_ccb3;
/// Black Ops III's specular colour and gloss maps (`i_..._s`, `i_..._g`).
const T7_SPECULAR_HASH: u32 = 0xec44_3804;
const T7_GLOSS_HASH: u32 = 0x6d0a_6c98;
/// Black Ops III's emissive map (a lit-emissive tritium dot's picture).
const T7_EMISSIVE_HASH: u32 = 0x3461_4347;

/// The largest level kept of a gun's pictures (first person), and of the
/// third-person models' (`BO2MP_BO3_TEXTURE=<size>` changes the first).
const FIRST_PERSON_MAX: u32 = 1024;
const WORLD_MAX: u32 = 512;

/// Black Ops III's multiplayer guns (`<gun>_mp` in core_common), without
/// the dual-optic, dual-wield and left-hand variants, the specialist
/// weapons, scorestreaks and equipment. Create-a-Class's names are in
/// `bo2_profile::guest::BO3`.
pub(crate) const BO3_GUNS: &[&str] = &[
    // Assault rifles.
    "ar_standard", "ar_accurate", "ar_fastburst", "ar_cqb", "ar_damage", "ar_longburst", "ar_marksman",
    // Submachine guns.
    "smg_standard", "smg_versatile", "smg_fastfire", "smg_longrange", "smg_burst", "smg_capacity",
    // Light machine guns.
    "lmg_light", "lmg_heavy", "lmg_cqb", "lmg_slowfire",
    // Sniper rifles.
    "sniper_powerbolt", "sniper_fastbolt", "sniper_fastsemi", "sniper_chargeshot",
    // Shotguns.
    "shotgun_pump", "shotgun_semiauto", "shotgun_precision", "shotgun_fullauto",
    // Pistols.
    "pistol_standard", "pistol_burst", "pistol_fullauto",
    // Launchers.
    "launcher_standard", "launcher_lockonly",
];

/// Black Ops III's guns, ready to join the match's catalogs.
pub(crate) struct T7Guests {
    pub weapons: WeaponBuild,
    pub fpv: FpvMeshBuild,
    pub world_guns: WorldWeaponBuild,
    pub xanims: XAnimBuild,
    pub material_seed: MaterialCatalog,
    pub report: Vec<String>,
}

impl T7Guests {
    pub(crate) fn empty(material_seed: MaterialCatalog, report: Vec<String>) -> Self {
        Self {
            weapons: WeaponBuild::default(),
            fpv: FpvMeshBuild::default(),
            world_guns: WorldWeaponBuild::default(),
            xanims: XAnimBuild::default(),
            material_seed,
            report,
        }
    }
}

/// Black Ops III's `zone` folder (the one with `core_common.ff`), if it is
/// installed.
pub(crate) fn bo3_zone_dir() -> Result<PathBuf, String> {
    let root = asset_transport::games_root_from_env()?;
    let zone = asset_transport::find_zone_file_version(&root, "core_common", asset_transport::T7_ZONE_VERSION)
        .map_err(|e| format!("Black Ops III is not installed (no core_common.ff in its format: {e})"))?;
    zone.path.parent().map(Path::to_path_buf).ok_or_else(|| "core_common.ff has no folder".into())
}

/// core_common, read ahead of the build.
pub(crate) enum T7Prefetch {
    Off(Vec<String>),
    Ready(Box<T7Zone>),
}

pub(crate) struct T7Zone {
    dir: PathBuf,
    zone: fastfile_t7::Zone,
    cap: Capture,
    report: Vec<String>,
    started: std::time::Instant,
}

/// bo2mp: reads Black Ops III's core_common for its guns (`wanted`: Black
/// Ops II rules with guest guns on), off the match's critical path.
pub(crate) fn prefetch_t7_guest_guns(wanted: bool) -> T7Prefetch {
    if !wanted {
        return T7Prefetch::Off(vec!["t7 guest guns: off".into()]);
    }
    let dir = match bo3_zone_dir() {
        Ok(dir) => dir,
        Err(e) => return T7Prefetch::Off(vec![format!("t7 guest guns: {e}")]),
    };
    let started = std::time::Instant::now();
    let path = dir.join("core_common.ff");
    let zone = match fastfile_t7::Zone::open(&path) {
        Ok(z) => z,
        Err(e) => return T7Prefetch::Off(vec![format!("t7 guest guns: {}: {e}", path.display())]),
    };
    let w = fastfile_t7::walk::walk_with(&zone, fastfile_t7::walk::Options { keep_anims: true, ..Default::default() });
    let report = vec![format!(
        "t7 guest guns: walked {} assets of {} ({} weapons, {} models, {} meshes, {} animations, {} materials, {} images; {}) [{:.1} s]",
        w.walked.len(),
        path.display(),
        w.capture.weapons.len(),
        w.capture.models.len(),
        w.capture.meshes.len(),
        w.capture.anims.len(),
        w.capture.materials.len(),
        w.capture.images.len(),
        match &w.stop {
            fastfile_t7::walk::Stop::Done { .. } => "to its end".to_owned(),
            other => format!("stopped at {other:?}"),
        },
        started.elapsed().as_secs_f64()
    )];
    T7Prefetch::Ready(Box::new(T7Zone { dir, zone, cap: w.capture, report, started }))
}

/// bo2mp: the match's Black Ops III guest guns from what
/// `prefetch_t7_guest_guns` read, their materials linked into
/// `material_seed`.
pub(crate) fn build_t7_guest_guns(
    prefetch: T7Prefetch,
    material_seed: MaterialCatalog,
    progress: &asset_transport::LoadProgress,
) -> T7Guests {
    let zone = match prefetch {
        T7Prefetch::Off(report) => return T7Guests::empty(material_seed, report),
        T7Prefetch::Ready(zone) => zone,
    };
    let stage = progress.begin_scoped(asset_transport::StageId::CommonAssets, "t7_guest_weapons", None);
    let guests = load_t7_guests(*zone, material_seed);
    stage.done();
    guests
}

/// bo2mp: read and built in one go (`iw4l check-guns`).
pub(crate) fn walk_t7_guest_guns(
    wanted: bool,
    material_seed: MaterialCatalog,
    progress: &asset_transport::LoadProgress,
) -> T7Guests {
    build_t7_guest_guns(prefetch_t7_guest_guns(wanted), material_seed, progress)
}

/// What a texture slot of a Black Ops III material is, for Black Ops II's
/// draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Kind {
    Colour,
    Normal,
    /// The specular colour map (its gloss map beside it).
    Specular,
}

/// The slot of a material texture, by the slot's name hash, else by the
/// picture's name (`_c`, `_n`, `_s`, `_g`).
fn slot_kind(hash: u32, image: &str) -> Option<Kind> {
    match hash {
        COLOR_MAP_HASH => return Some(Kind::Colour),
        NORMAL_MAP_HASH => return Some(Kind::Normal),
        T7_SPECULAR_HASH => return Some(Kind::Specular),
        _ => {}
    }
    let n = image.to_ascii_lowercase();
    if n.ends_with("_c") || n.ends_with("_col") {
        Some(Kind::Colour)
    } else if n.ends_with("_n") || n.ends_with("_nml") {
        Some(Kind::Normal)
    } else {
        None
    }
}

/// A Black Ops III vertex's binormal sign from its packed tangent's top two
/// bits: 3 where cross(normal, tangent) runs along the UVs' v (+1), 0 where
/// it runs against (-1). (Measured on `wpn_t7_arak_view`: of its triangles'
/// UV bitangents, 22074 run against cross(n, t) with 0, 8173 along with 3,
/// a handful either way.)
fn binormal_sign(w: u8) -> f32 {
    if w & 2 != 0 { 1.0 } else { -1.0 }
}

/// One vertex as Black Ops II's packed one (32 bytes: position, binormal
/// sign, colour, UV halves, normal, tangent).
fn t6_vertex(p: [f32; 3], sign: f32, colour: [u8; 4], uv: [f32; 2], n: [f32; 3], t: [f32; 3]) -> [u8; 32] {
    let mut v = [0u8; 32];
    for k in 0..3 {
        v[4 * k..4 * k + 4].copy_from_slice(&p[k].to_le_bytes());
    }
    v[12..16].copy_from_slice(&sign.to_le_bytes());
    v[16..20].copy_from_slice(&colour);
    let tex = u32::from(f16_bits(uv[0])) | u32::from(f16_bits(uv[1])) << 16;
    v[20..24].copy_from_slice(&tex.to_le_bytes());
    v[24..28].copy_from_slice(&t6_unit_vec(n).to_le_bytes());
    v[28..32].copy_from_slice(&t6_unit_vec(t).to_le_bytes());
    v
}

/// A quaternion component as Black Ops II's 16-bit fraction.
fn frac16(x: f32) -> i16 {
    (x.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// A half's bits as Black Ops II's 16-bit fraction's bits.
fn half_to_frac_bits(h: u16) -> u16 {
    frac16(fastfile_t7::mesh::f16(h)) as u16
}

/// Black Ops III's animation data in Black Ops II's packing: every
/// quaternion component (the rotation sections' keyed values in
/// `randomDataShort`, their constants in `dataShort`) from a half to a
/// 16-bit fraction. The sections are walked in the order the decoder reads
/// them (`xmodel_runtime::AnimClip::from_parts`): per keyed bone its key
/// count in `dataShort`, its key frames (bytes when `numframes < 256`,
/// else `dataShort`, or `indices` with `dataShort` checkpoints for 64 keys
/// and more), then its values. `None` when the data runs short.
fn t7_quats_to_t6(r: &mut asset_t6::XAnimRef) -> Option<()> {
    let byte_indices = r.numframes < 256;
    let (mut ds, mut db, mut idx, mut rs) = (0usize, 0usize, 0usize, 0usize);
    for (section, per) in [(1usize, 2usize), (2, 4)] {
        for _ in 0..r.bone_count[section] {
            let size = usize::from(*r.data_short.get(ds)?);
            ds += 1;
            let count = size + 1;
            if byte_indices {
                db += count;
            } else if size >= 64 {
                idx += count;
                ds += (count - 2) / 256 + 2;
            } else {
                ds += count;
            }
            for v in r.random_data_short.get_mut(rs..rs + count * per)? {
                *v = half_to_frac_bits(*v);
            }
            rs += count * per;
        }
    }
    let _ = (db, idx);
    for (section, per) in [(3usize, 2usize), (4, 4)] {
        for _ in 0..r.bone_count[section] {
            for v in r.data_short.get_mut(ds..ds + per)? {
                *v = half_to_frac_bits(*v);
            }
            ds += per;
        }
    }
    Some(())
}

/// The colour (linear) of a material with no picture to colour it: see
/// `Builder::stand_in_colour`. `placeholder` is its colour slot's
/// placeholder (`,$black_color`, `,$white_diffuse`).
fn solid_colour(name: &str, placeholder: &str, consts: Option<[f32; 8]>) -> [f32; 3] {
    let tint = consts.map(|c| [c[0], c[1], c[2]]);
    let spec = consts.map(|c| [c[4], c[5], c[6]]);
    let lit = |c: &[f32; 3]| c.iter().all(|x| x.is_finite()) && c.iter().any(|&x| x > 0.004);
    let white = |c: &[f32; 3]| c.iter().all(|&x| x > 0.99);
    if placeholder.contains("black")
        && let Some(s) = spec.filter(lit)
    {
        return s;
    }
    if let Some(t) = tint.filter(|t| lit(t) && !white(t)) {
        return t;
    }
    // (A tritium dot without constants: its colour by its name.)
    for (word, rgb) in [
        ("red", [0.9, 0.02, 0.02]),
        ("grn", [0.05, 0.9, 0.1]),
        ("green", [0.05, 0.9, 0.1]),
        ("ylw", [0.9, 0.75, 0.05]),
        ("yellow", [0.9, 0.75, 0.05]),
        ("tritium", [0.05, 0.9, 0.1]),
    ] {
        if name.split(['_', '/']).any(|w| w == word) {
            return rgb;
        }
    }
    [0.03, 0.03, 0.03]
}

/// One colour as a 4x4 sRGB picture (DXGI R8G8B8A8_UNORM_SRGB).
fn solid_levels(rgb: [f32; 3]) -> ImageLevels {
    let srgb = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        let s = if x <= 0.003_130_8 { 12.92 * x } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
        (s * 255.0).round() as u8
    };
    let px = [srgb(rgb[0]), srgb(rgb[1]), srgb(rgb[2]), 255];
    ImageLevels { format: 29, width: 4, height: 4, levels: 1, data: px.repeat(16), part: None }
}

/// Black Ops III's animation slots -> Black Ops II's (`szXAnims`, the slots
/// `WeaponCatalog::capture_t6` reads).
const T7_TO_T6_ANIMS: &[(usize, usize)] = &[
    (1, 0x01), (2, 0x02), (3, 0x03), (4, 0x04), (7, 0x05), (10, 0x06), (11, 0x08), (12, 0x09), (14, 0x0D),
    (20, 0x0E), (40, 0x10), (41, 0x12), (42, 0x13), (43, 0x14), (44, 0x15), (45, 0x16), (50, 0x17), (51, 0x18),
    (52, 0x19), (53, 0x1A), (54, 0x1B), (57, 0x1C), (58, 0x1D), (59, 0x1E), (60, 0x1F), (72, 0x20), (73, 0x21),
    (74, 0x22), (75, 0x23), (76, 0x24), (77, 0x25), (84, 0x2C), (85, 0x2D), (86, 0x2E), (87, 0x2F), (88, 0x30),
    (89, 0x31), (90, 0x32), (91, 0x33), (92, 0x34), (93, 0x35), (94, 0x36), (95, 0x37), (103, 0x3A), (104, 0x3D),
    (106, 0x3E), (107, 0x3F), (108, 0x40), (173, 0x55), (174, 0x56),
];

/// Black Ops II's `szXAnims` slot count.
const T6_ANIM_SLOTS: usize = 88;

/// Black Ops III's `weapFireType_t` (full auto, single shot, burst, auto
/// burst, stacked, all locked, minigun, charge shot, ...) as Black Ops II's
/// (full auto, single, burst 2..5, stacked, minigun, charge shot):
/// bursts by their `burstCount`.
fn t7_fire_type(raw: i32, burst: i32) -> i32 {
    match raw {
        0 => 0,
        2 | 3 => 2 + (burst.clamp(2, 5) - 2),
        4 => 6,
        6 => 7,
        7..=9 => 8,
        _ => 1,
    }
}

/// Black Ops III's `playerAnimType` as Black Ops II's: the first entries
/// (none, default, other, sniper) and hold .. remote control are the same;
/// the rest are drawn as the default rifle's.
fn t7_player_anim_type(raw: i32) -> i32 {
    match raw {
        0..=3 | 5..=10 => raw,
        _ => 1,
    }
}

struct Builder<'a> {
    zone: &'a fastfile_t7::Zone,
    cap: &'a Capture,
    paks: XpakSet,
    catalog: MaterialCatalog,
    owner: ZoneOwner,
    /// (material name, size cap) -> catalog material.
    materials: HashMap<(String, u32), Option<usize>>,
    /// (picture key, size cap) -> catalog image slot.
    images: HashMap<(String, u32), Option<usize>>,
    image_failures: Vec<String>,
    decoded: usize,
}

impl<'a> Builder<'a> {
    fn string(&self, index: u32) -> String {
        self.zone
            .assets
            .script_strings
            .get(index as usize)
            .cloned()
            .flatten()
            .unwrap_or_default()
    }

    /// The densest LOD's mesh bytes (in the zone or its `.xpak` entry).
    fn mesh_bytes(&self, mesh: &fastfile_t7::walk::capture::MeshRec) -> Result<Vec<u8>, String> {
        match (mesh.data, mesh.xpak) {
            (Some((at, len)), _) => self.zone.image.get(at..at + len).map(<[u8]>::to_vec).ok_or_else(|| "mesh data past the zone".into()),
            (None, Some(x)) => self.paks.read(x.key).map_err(|e| e.to_string()),
            _ => Err("mesh has no data".into()),
        }
    }

    /// A model in Black Ops II's captured form (its densest LOD) and the
    /// material name of each surface.
    fn model_ref(&self, rec: &ModelRec) -> Result<(asset_t6::XModelRef, Vec<Option<String>>), String> {
        let lod = rec
            .lods
            .iter()
            .max_by_key(|l| self.cap.mesh(&l.mesh).map_or(0, |m| m.vert_count))
            .ok_or("no LODs")?;
        let mesh = self.cap.mesh(&lod.mesh).ok_or_else(|| format!("mesh {} not read", lod.mesh))?;
        let data = self.mesh_bytes(mesh)?;
        let g = fastfile_t7::mesh::decode(mesh, &data).map_err(|e| e.to_string())?;
        let k = &rec.skeleton;
        let n_bones = rec.bones;
        if n_bones == 0 || n_bones > 255 || k.base_mat.len() != n_bones || k.bone_names.len() != n_bones {
            return Err(format!("skeleton not read ({n_bones} bones)"));
        }
        // (Skin weights name bones by index; a few models count them in
        // 64-byte bone units as Black Ops II does.)
        let max_bone = g.weights.iter().flatten().map(|w| w.0).max().unwrap_or(0);
        let units = usize::from(max_bone) >= n_bones && g.weights.iter().flatten().all(|w| w.0 % 64 == 0);
        let bone_of = |b: u16| -> u16 {
            let b = if units { b / 64 } else { b };
            if usize::from(b) < n_bones { b } else { 0 }
        };
        let (mut mins, mut maxs) = ([f32::MAX; 3], [f32::MIN; 3]);
        let mut lod0 = Vec::with_capacity(mesh.surfaces.len());
        let mut materials = Vec::with_capacity(mesh.surfaces.len());
        for (si, sf) in mesh.surfaces.iter().enumerate() {
            let base = sf.base_vert as usize;
            let n = usize::from(sf.vert_count);
            if base + n > g.positions.len() {
                return Err(format!("surface {si} past the vertices"));
            }
            // Each vertex's influences, heaviest first, at most four.
            let infl: Vec<Vec<(u16, f32)>> = (base..base + n)
                .map(|v| {
                    let mut w: Vec<(u16, f32)> = g.weights[v].iter().map(|&(b, x)| (bone_of(b), x)).filter(|x| x.1 > 0.0).collect();
                    w.sort_by(|a, b| b.1.total_cmp(&a.1));
                    w.truncate(4);
                    if w.is_empty() {
                        w.push((0, 1.0));
                    }
                    let sum: f32 = w.iter().map(|x| x.1).sum();
                    for x in &mut w {
                        x.1 /= sum.max(1e-6);
                    }
                    w
                })
                .collect();
            // Black Ops II orders a surface's vertices by influence count.
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by_key(|&i| infl[i].len());
            let mut new_of = vec![0u16; n];
            for (new, &old) in order.iter().enumerate() {
                new_of[old] = new as u16;
            }
            let mut verts = Vec::with_capacity(n * 32);
            let mut part_bits = [0u32; 6];
            let mut blend_counts = [0i16; 4];
            let mut blend = Vec::new();
            for &old in &order {
                let v = base + old;
                let p = g.positions[v];
                for c in 0..3 {
                    mins[c] = mins[c].min(p[c]);
                    maxs[c] = maxs[c].max(p[c]);
                }
                verts.extend_from_slice(&t6_vertex(
                    p,
                    binormal_sign(g.tangent_w.get(v).copied().unwrap_or(0)),
                    g.colors.get(v).copied().unwrap_or([255; 4]),
                    g.uvs[v],
                    g.normals[v],
                    g.tangents.get(v).copied().unwrap_or([1.0, 0.0, 0.0]),
                ));
                let w = &infl[old];
                for &(bone, _) in w {
                    if let Some(word) = part_bits.get_mut(usize::from(bone) / 32) {
                        *word |= 1 << (bone % 32);
                    }
                }
                blend_counts[w.len() - 1] += 1;
                blend.push(w[0].0 * 64);
                for &(b, x) in &w[1..] {
                    blend.push(b * 64);
                    blend.push((x * 65536.0).round().clamp(0.0, 65535.0) as u16);
                }
            }
            let indices = g.surfaces[si]
                .indices
                .iter()
                .map(|&i| new_of.get((i as usize).wrapping_sub(base)).copied().unwrap_or(0))
                .collect();
            lod0.push(asset_t6::XSurfaceRef {
                verts,
                vert_count: n as u16,
                indices,
                vert_lists: Vec::new(),
                flags: 0,
                material: None,
                blend_counts,
                blend,
                part_bits,
            });
            materials.push(lod.materials.get(si).cloned().filter(|m| !m.is_empty()));
        }
        if lod0.is_empty() {
            return Err("no surfaces".into());
        }
        let radius = (0..3).map(|c| mins[c].abs().max(maxs[c].abs()).powi(2)).sum::<f32>().sqrt();
        let roots = k.num_root_bones.max(1);
        Ok((
            asset_t6::XModelRef {
                name: rec.name.clone(),
                numsurfs: lod0.len().min(255) as u8,
                materials: Vec::new(),
                num_bones: n_bones as u8,
                num_lods: 1,
                mins,
                maxs,
                radius,
                lighting_origin_offset: [0.0; 3],
                lighting_origin_range: 0.0,
                lods: vec![(0.0, lod0.len() as u16, 0)],
                lod0,
                bone_names: k.bone_names.iter().map(|&s| self.string(s)).collect(),
                num_root_bones: roots,
                parent_list: k.parent_list.clone(),
                quats: k.quats.iter().map(|q| q.map(frac16)).collect(),
                trans: k.trans.clone(),
                base_mat: k.base_mat.iter().map(|m| ([m[0], m[1], m[2], m[3]], [m[4], m[5], m[6]])).collect(),
                part_classification: k.part_classification.clone(),
                bone_info: vec![None; n_bones],
                contents: 0,
                coll_lod: -1,
                coll_surfs: Vec::new(),
                phys_preset: None,
            },
            materials,
        ))
    }

    /// A picture's levels at most `max` on a side.
    fn levels(&self, name: &str, max: u32) -> Result<ImageLevels, String> {
        let im = self.cap.image(name).ok_or_else(|| format!("{name}: not in core_common"))?;
        texture::image_levels_max(im, &self.zone.image, &self.paks, max).map_err(|e| format!("{name}: {e}"))
    }

    /// One picture into the catalog (`key` names the converted form).
    fn link_image(&mut self, key: String, max: u32, semantic: u8, make: impl FnOnce(&Self) -> Result<ImageLevels, String>) -> Option<usize> {
        if let Some(slot) = self.images.get(&(key.clone(), max)) {
            return *slot;
        }
        let colour = semantic == TS_COLOR_MAP;
        let slot = match make(self).and_then(|l| {
            let px = fastfile_iw6::image::ImagePixels {
                width: l.width,
                height: l.height,
                levels: l.levels,
                dxgi_format: l.format,
                data: l.data,
            };
            super::iw6::gpu_image(&px, colour).map(|img| (px, img))
        }) {
            Ok((px, gpu)) => {
                self.decoded += 1;
                Some(self.catalog.link_image(AuthoredImage {
                    namespace: NS,
                    name: AssetRef::Real(format!("{key}#{max}")),
                    map_type: 0,
                    semantic,
                    category: 0,
                    use_srgb_reads: colour,
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
                if self.image_failures.len() < 64 {
                    self.image_failures.push(e);
                }
                None
            }
        };
        self.images.insert((key, max), slot);
        slot
    }

    /// One material into the catalog: its colour map, its normal map (as
    /// BC5), its specular colour with its gloss in alpha (as BC3).
    fn material(&mut self, name: &str, max: u32) -> Option<usize> {
        if let Some(&m) = self.materials.get(&(name.to_owned(), max)) {
            return m;
        }
        let Some(m) = self.cap.material(name).filter(|_| !name.starts_with(',')) else {
            self.materials.insert((name.to_owned(), max), None);
            return None;
        };
        let technique = m.techset.clone().unwrap_or_default();
        let draw = t7_draw(&m.name, technique.trim_start_matches(','), true);
        let real = |n: &String| !n.starts_with(',') && !n.starts_with('$');
        let mut picks: HashMap<Kind, String> = HashMap::new();
        for (hash, image) in m.textures.iter().filter(|(_, n)| real(n)) {
            if let Some(kind) = slot_kind(*hash, image) {
                picks.entry(kind).or_insert_with(|| image.clone());
            }
        }
        let gloss = m.textures.iter().find(|(h, n)| *h == T7_GLOSS_HASH && real(n)).map(|(_, n)| n.clone());
        let mut textures = Vec::new();
        let colour_slot = match picks.get(&Kind::Colour).cloned() {
            Some(colour) => self.link_image(colour.clone(), max, TS_COLOR_MAP, |b| b.levels(&colour, max)),
            None => self.stand_in_colour(m, &technique, picks.get(&Kind::Specular).cloned(), max),
        };
        if let Some(slot) = colour_slot {
            textures.push(MaterialTextureBinding {
                name_hash: COLOR_MAP_HASH,
                name_start: 0,
                name_end: 0,
                sampler_state: 0x13,
                semantic: TS_COLOR_MAP,
                image: Some(slot),
            });
        }
        if !draw.unlit {
            if let Some(normal) = picks.get(&Kind::Normal).cloned()
                && let Some(slot) = self.link_image(format!("{normal}#bc5"), max, TS_NORMAL_MAP, |b| {
                    let l = b.levels(&normal, max)?;
                    texture::normal_map_bc5(&l).ok_or_else(|| format!("{normal}: format {} not decoded", l.format))
                })
            {
                textures.push(MaterialTextureBinding {
                    name_hash: NORMAL_MAP_HASH,
                    name_start: 0,
                    name_end: 0,
                    sampler_state: 0x13,
                    semantic: TS_NORMAL_MAP,
                    image: Some(slot),
                });
            }
            if let (Some(spec), Some(gloss)) = (picks.get(&Kind::Specular).cloned(), gloss)
                && let Some(slot) = self.link_image(format!("{spec}+{gloss}"), max, TS_SPECULAR_MAP, |b| {
                    let s = b.levels(&spec, max)?;
                    // (The gloss map at least as big as the specular map.)
                    let g = b.levels(&gloss, max.max(s.width.max(s.height)))?;
                    texture::specular_gloss_bc3(&s, &g).ok_or_else(|| format!("{spec} + {gloss}: formats {} and {} not paired", s.format, g.format))
                })
            {
                textures.push(MaterialTextureBinding {
                    name_hash: SPECULAR_MAP_HASH,
                    name_start: 0,
                    name_end: 0,
                    sampler_state: 0x13,
                    semantic: TS_SPECULAR_MAP,
                    image: Some(slot),
                });
            }
        }
        let index = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(format!("{}#{max}", m.name)),
            namespace: NS,
            technique_set: AssetRef::default(),
            technique_set_edge: AssetEdge::Absent,
            draw_surf: 0,
            sort_key: draw.sort_key,
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
            constants: Vec::new(),
            zone: self.owner,
            t6_draw: Some(draw),
        });
        self.materials.insert((name.to_owned(), max), Some(index));
        Some(index)
    }

    /// The colour a material without a colour map of its own is drawn with
    /// (Black Ops III draws many gun parts from their specular colour, or
    /// from constants alone: a bullet's brass, a tritium dot):
    ///
    /// - its specular colour map (`_s`; metal parts' colour is their
    ///   reflectance, and BO2 draws a gun's metal from its colour map);
    /// - else its emissive picture (a lit-emissive tritium dot's);
    /// - else one colour from its constants: the specular colour where the
    ///   colour slot is Black Ops III's black placeholder (metals: brass),
    ///   else its tint (a tritium dot's red), else a dark grey.
    ///
    /// `None` for what Black Ops II's gun pass should not draw solid: a
    /// paint-shop overlay, a scope's stencil or glass, a reticle, a
    /// thermal screen, a 3D-UI panel.
    fn stand_in_colour(
        &mut self,
        m: &fastfile_t7::walk::capture::MaterialRec,
        technique: &str,
        specular: Option<String>,
        max: u32,
    ) -> Option<usize> {
        let t = technique.trim_start_matches(',');
        let skip = ["paintshop", "stencil", "reticle", "thermal", "ui_3d", "transparent", "glass"];
        if skip.iter().any(|w| t.contains(w) || m.name.contains(w)) {
            return None;
        }
        if let Some(spec) = specular
            && let Some(slot) = self.link_image(spec.clone(), max, TS_COLOR_MAP, |b| b.levels(&spec, max))
        {
            return Some(slot);
        }
        let real = |n: &str| !n.starts_with(',') && !n.starts_with('$');
        if t.contains("emissive")
            && let Some((_, glow)) = m.textures.iter().find(|(h, n)| *h == T7_EMISSIVE_HASH && real(n))
        {
            let glow = glow.clone();
            if let Some(slot) = self.link_image(glow.clone(), max, TS_COLOR_MAP, |b| b.levels(&glow, max)) {
                return Some(slot);
            }
        }
        let placeholder = m
            .textures
            .iter()
            .find(|(h, _)| *h == COLOR_MAP_HASH)
            .map_or("", |(_, n)| n.as_str());
        let rgb = solid_colour(&m.name, placeholder, m.colour_consts);
        let key = format!("$solid:{:.3},{:.3},{:.3}", rgb[0], rgb[1], rgb[2]);
        self.link_image(key, 4, TS_COLOR_MAP, |_| Ok(solid_levels(rgb)))
    }

    /// An animation in Black Ops II's captured form; `None` when its bone
    /// counts do not fit Black Ops II's bytes or its data is short.
    fn anim_ref(&self, a: &AnimRec) -> Option<asset_t6::XAnimRef> {
        let h = &a.header;
        let u16_at = |o: usize| h.get(o..o + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
        let mut bone_count = [0u8; 10];
        for (k, c) in bone_count.iter_mut().enumerate() {
            *c = u8::try_from(u16_at(0x30 + 2 * k)).ok()?;
        }
        let framerate = h.get(0x50..0x54).map_or(30.0, |b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
        let bytes = |s: Option<(usize, usize)>| s.and_then(|(at, len)| self.zone.image.get(at..at + len)).unwrap_or(&[]);
        let indices = if a.numframes >= 256 {
            super::iw6::le_u16s(bytes(a.indices))
        } else {
            bytes(a.indices).iter().map(|&b| u16::from(b)).collect()
        };
        let notifies = a
            .notifies
            .iter()
            .map(|&(kind, time, param, _)| {
                let (kind, param) = (self.string(kind), self.string(param));
                let name = match kind.as_str() {
                    "sound" if !param.is_empty() => format!("{}{param}", asset_game::T5_NOTE_SOUND_PREFIX),
                    "rumble" if !param.is_empty() => format!("{}{param}", asset_game::T5_NOTE_RUMBLE_PREFIX),
                    _ if !param.is_empty() => param,
                    _ => kind,
                };
                (name, time)
            })
            .collect();
        let mut r = asset_t6::XAnimRef {
            delta_quat: Vec::new(),
            name: a.name.clone(),
            numframes: a.numframes,
            framerate: if framerate.is_finite() && framerate > 0.0 { framerate } else { 30.0 },
            looping: h.get(0x44).is_some_and(|&b| b != 0),
            delta: false,
            delta_trans: Vec::new(),
            bone_count,
            names: a.names.iter().map(|&s| self.string(s)).collect(),
            notifies,
            data_byte: bytes(a.data_byte).to_vec(),
            data_short: super::iw6::le_u16s(bytes(a.data_short)),
            data_int: super::iw6::le_u32s(bytes(a.data_int)),
            random_data_byte: bytes(a.random_data_byte).to_vec(),
            random_data_short: super::iw6::le_u16s(bytes(a.random_data_short)),
            random_data_int: super::iw6::le_u32s(bytes(a.random_data_int)),
            indices,
        };
        t7_quats_to_t6(&mut r)?;
        Some(r)
    }

    /// The weapon's `WeaponDef` bytes (its own, or the variant's whose it shares).
    fn def_at(&self, w: &WeaponRec) -> Option<usize> {
        w.def_at.or_else(|| {
            let owner = w.def_shared_with.as_deref()?;
            self.cap.weapons.iter().find(|x| x.name == owner)?.def_at
        })
    }

    /// A string field of the weapon, else of another variant of the same
    /// gun that loaded the list (`attachViewModelTag[4]`).
    fn field_str(&self, w: &WeaponRec, field: &str) -> Option<String> {
        let own = |r: &WeaponRec| match r.field(field) {
            Some(FieldValue::Str(s)) => Some(s.clone()),
            Some(FieldValue::Asset { name, .. }) => Some(name.clone()),
            None => None,
        };
        own(w).or_else(|| {
            self.cap
                .weapons
                .iter()
                .filter(|r| r.base_name == w.base_name && r.name != w.name)
                .find_map(own)
        })
        .filter(|s| !s.is_empty() && !s.starts_with("<ref"))
    }

    /// The weapon as Black Ops II's `WeaponRef` (`catalog`: its name in the
    /// catalog, `ar_standard`; an upgraded Zombies gun `ar_standard_upgraded`).
    fn weapon_ref(&self, w: &WeaponRec, catalog: &str, display: &str) -> Option<asset_t6::WeaponRef> {
        let img = &self.zone.image;
        let variant7 = img.get(w.header_at..w.header_at + 0x300)?;
        let def7 = self.def_at(w).and_then(|d| img.get(d..d + 0x15f0))?;
        let mut variant = vec![0u8; v6::SIZE];
        let mut def = vec![0u8; d6::SIZE];
        for &(dst, at6, at7, n) in T7_TO_T6 {
            let (at6, at7, n) = (usize::from(at6), usize::from(at7), usize::from(n));
            let src = if at7 < 0x300 { variant7.get(at7..at7 + n) } else { def7.get(at7 - 0x300..at7 - 0x300 + n) };
            let out = if dst == 0 { &mut variant } else { &mut def };
            if let (Some(src), Some(out)) = (src, out.get_mut(at6..at6 + n)) {
                out.copy_from_slice(src);
            }
        }
        let i32_7 = |at7: usize| def7.get(at7 - 0x300..at7 - 0x300 + 4).map_or(0, |b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]));
        let put = |buf: &mut Vec<u8>, at: usize, v: i32| {
            if let Some(b) = buf.get_mut(at..at + 4) {
                b.copy_from_slice(&v.to_le_bytes());
            }
        };
        // (HydraX's offsets: fireType 0x380, burstCount 0x11f8,
        // playerAnimType 0x368.)
        put(&mut def, d6::fireType, t7_fire_type(i32_7(0x380), i32_7(0x11f8)));
        put(&mut def, d6::playerAnimType, t7_player_anim_type(i32_7(0x368)));
        // Animations onto Black Ops II's slots.
        let mut xanims = vec![String::new(); T6_ANIM_SLOTS];
        for (slot, _, anim) in &w.anims {
            if let Some(&(_, t6)) = T7_TO_T6_ANIMS.iter().find(|(t7, _)| t7 == slot) {
                xanims[t6] = anim.to_ascii_lowercase();
            }
        }
        // Sound aliases onto Black Ops II's string fields (by name; Black
        // Ops III's looping fire is Black Ops II's per-shot fire when the gun
        // has no fire sound of its own).
        let s = |f: &str| self.field_str(w, f);
        let mut strings: Vec<(usize, String)> = Vec::new();
        let mut set = |off: usize, v: Option<String>| {
            if let Some(v) = v {
                strings.push((off, v));
            }
        };
        set(d6::fireSound, s("fireSound").or_else(|| s("loopFireSound")));
        set(d6::fireSoundPlayer, s("fireSoundPlayer").or_else(|| s("loopFireSoundPlayer")));
        set(d6::fireLastSound, s("lastShotSound"));
        set(d6::fireLastSoundPlayer, s("lastShotSoundPlayer"));
        set(d6::emptyFireSound, s("emptyFireSound"));
        set(d6::emptyFireSoundPlayer, s("emptyFireSoundPlayer"));
        for (off, f) in [
            (d6::reloadSound, "reloadSound"),
            (d6::reloadSoundPlayer, "reloadSoundPlayer"),
            (d6::reloadEmptySound, "reloadEmptySound"),
            (d6::reloadEmptySoundPlayer, "reloadEmptySoundPlayer"),
            (d6::reloadStartSound, "reloadStartSound"),
            (d6::reloadStartSoundPlayer, "reloadStartSoundPlayer"),
            (d6::reloadEndSound, "reloadEndSound"),
            (d6::reloadEndSoundPlayer, "reloadEndSoundPlayer"),
            (d6::rechamberSound, "rechamberSound"),
            (d6::rechamberSoundPlayer, "rechamberSoundPlayer"),
            (d6::pullbackSound, "pullbackSound"),
            (d6::pullbackSoundPlayer, "pullbackSoundPlayer"),
            (d6::adsZoomSound, "adsZoomSound"),
            (d6::projExplosionSound, "projExplosionSound"),
            (d6::projectileSound, "projectileSound"),
            (d6::projIgnitionSound, "projIgnitionSound"),
        ] {
            set(off, s(f));
        }
        // The attached models (a magazine, a scope): view and world, with
        // their tags (Black Ops III leaves most empty: the gun's root),
        // offsets and rotations (HydraX: 0x78.., 0xb4.., 0xf0.., 0x12c..).
        let f32_v = |at: usize| variant7.get(at..at + 4).map_or(0.0, |b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
        let vec3 = |at: usize| [f32_v(at), f32_v(at + 4), f32_v(at + 8)];
        let attached = |models: &str, tags: &str, ofs: usize, rot: usize| -> Vec<(usize, String, String, [f32; 3], [f32; 3])> {
            (0..5)
                .filter_map(|i| {
                    let model = s(&format!("{models}[{i}]"))?;
                    if model.starts_with(',') {
                        return None;
                    }
                    let tag = s(&format!("{tags}[{i}]")).unwrap_or_else(|| "tag_weapon".to_owned());
                    Some((i, model, tag, vec3(ofs + 12 * i), vec3(rot + 12 * i)))
                })
                .collect()
        };
        let location_damage = w.location_damage_at.and_then(|at| {
            let b = img.get(at..at + 21 * 4)?;
            Some(b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
        });
        let model = |f: &str| s(f).filter(|m| !m.starts_with(','));
        Some(asset_t6::WeaponRef {
            name: catalog.to_owned(),
            variant,
            def,
            display_name: display.to_owned(),
            alt_weapon_name: String::new(),
            ammo_name: s("ammoName").unwrap_or_default(),
            clip_name: s("clipName").unwrap_or_default(),
            xanims,
            hide_tags: w.hide_tags.iter().map(|&t| self.string(t)).filter(|t| !t.is_empty()).collect(),
            attach_view_models: attached("attachViewModel", "attachViewModelTag", 0x78, 0xf0),
            attach_world_models: attached("attachWorldModel", "attachWorldModelTag", 0xb4, 0x12c),
            gun_models: model("gunModel").into_iter().collect(),
            world_models: model("worldModel").into_iter().collect(),
            strings,
            fx: Vec::new(),
            materials: Vec::new(),
            models: Vec::new(),
            tracers: Vec::new(),
            overlay_material: String::new(),
            overlay_material_low: String::new(),
            dpad_icon: String::new(),
            notetrack_sounds: Vec::new(),
            bounce_sounds: Vec::new(),
            parallel_bounce: None,
            perpendicular_bounce: None,
            location_damage,
            camo: String::new(),
        })
    }
}

/// The guns this match takes: each multiplayer gun (`<gun>_mp`, catalog
/// name `<gun>`) and, for Pack-a-Punch, its upgraded Zombies copy when
/// core_common has one (`<gun>_upgraded_zm`, catalog name
/// `<gun>_upgraded`).
fn chosen<'a>(cap: &'a Capture) -> Vec<(String, &'a WeaponRec)> {
    let mut out = Vec::new();
    for gun in BO3_GUNS {
        if let Some(w) = cap.weapons.iter().find(|w| w.name == format!("{gun}_mp")) {
            out.push(((*gun).to_owned(), w));
        }
        if let Some(w) = cap.weapons.iter().find(|w| w.name == format!("{gun}_upgraded_zm")) {
            out.push((format!("{gun}_upgraded"), w));
        }
    }
    out
}

/// Black Ops III's guns from core_common, their materials linked into
/// `material_seed`.
fn load_t7_guests(z: T7Zone, material_seed: MaterialCatalog) -> T7Guests {
    let T7Zone { dir, zone, cap, mut report, started } = z;
    let paks = match XpakSet::open_dir(&dir) {
        Ok((paks, bad)) => {
            if !bad.is_empty() {
                report.push(format!("t7 guest guns: {} packages unreadable", bad.len()));
            }
            paks
        }
        Err(e) => {
            report.push(format!("t7 guest guns: packages: {e} (streamed meshes and pictures left out)"));
            XpakSet::default()
        }
    };
    let mut b = Builder {
        zone: &zone,
        cap: &cap,
        paks,
        catalog: material_seed,
        owner: ZoneOwner::from_zone_path(&dir.join("core_common.ff")),
        materials: HashMap::new(),
        images: HashMap::new(),
        image_failures: Vec::new(),
        decoded: 0,
    };
    let first_person_max = std::env::var("BO2MP_BO3_TEXTURE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(FIRST_PERSON_MAX);
    let guns = chosen(&cap);

    // Models: each gun's view and world model and the models it hangs on
    // them (its magazine).
    let mut first_person: BTreeSet<String> = BTreeSet::new();
    let mut third_person: BTreeSet<String> = BTreeSet::new();
    for (_, w) in &guns {
        for (f, v) in &w.fields {
            let FieldValue::Asset { name, .. } = v else { continue };
            if name.starts_with(',') || name.starts_with("<ref") {
                continue;
            }
            if f == "gunModel" || f.starts_with("attachViewModel[") {
                first_person.insert(name.clone());
            } else if f == "worldModel" || f.starts_with("attachWorldModel[") {
                third_person.insert(name.clone());
            }
        }
    }
    let mut fpv = FpvMeshBuild::default();
    fpv.set_capture_ns(NS);
    let mut world_guns = WorldWeaponBuild::default();
    world_guns.set_capture_ns(NS);
    let mut unreadable = Vec::new();
    for (set, max, first) in [(&first_person, first_person_max, true), (&third_person, WORLD_MAX, false)] {
        for name in set {
            let Some(rec) = cap.model(name) else {
                unreadable.push(format!("{name} (not in core_common)"));
                continue;
            };
            let (xref, mats) = match b.model_ref(rec) {
                Ok(x) => x,
                Err(e) => {
                    unreadable.push(format!("{name} ({e})"));
                    continue;
                }
            };
            // Surface materials: local key = surface index.
            let linked: Vec<Option<usize>> = mats.iter().map(|m| m.as_deref().and_then(|m| b.material(m, max))).collect();
            let mut xref = xref;
            let key = |k: usize| asset_t6::AssetKey { ty: fastfile_t6::AssetType::Material, index: k };
            for (i, s) in xref.lod0.iter_mut().enumerate() {
                s.material = linked.get(i).copied().flatten().map(|_| key(i));
            }
            xref.materials = xref.lod0.iter().map(|s| s.material).collect();
            match asset_model::capture_model_skel_t6(&xref, |k| linked.get(k.index).copied().flatten()) {
                Some(skel) if first => fpv.insert_in(NS, skel, Some(&b.catalog)),
                Some(skel) => world_guns.insert_in(NS, skel, Some(&b.catalog)),
                None => unreadable.push(format!("{name} (unreadable)")),
            }
        }
    }
    report.push(format!(
        "t7 guest guns: models: {} first person, {} third person built; {} not {:?}",
        fpv.len(),
        world_guns.len(),
        unreadable.len(),
        &unreadable[..unreadable.len().min(8)]
    ));
    report.push(format!(
        "t7 guest guns: materials: {} linked, {} pictures decoded, {} failed {:?}",
        b.materials.values().filter(|m| m.is_some()).count(),
        b.decoded,
        b.image_failures.len(),
        &b.image_failures[..b.image_failures.len().min(6)]
    ));

    // Animations every gun names.
    let mut xanims = XAnimBuild::default();
    xanims.set_capture_ns(NS);
    let wanted: BTreeSet<&str> = guns.iter().flat_map(|(_, w)| w.anims.iter().map(|a| a.2.as_str())).collect();
    let (mut anim_n, mut anim_refused) = (0usize, Vec::new());
    for name in &wanted {
        match cap.anim(name).map(|a| (a, b.anim_ref(a))) {
            Some((_, Some(r))) => anim_n += usize::from(xanims.insert_t6_as(NS, &r)),
            Some((a, None)) => anim_refused.push(format!("{} ({} bones)", a.name, a.names.len())),
            None => anim_refused.push(format!("{name} (not kept)")),
        }
    }
    report.push(format!(
        "t7 guest guns: animations: {anim_n} of {} named kept ({} gaps; {} left out {:?})",
        wanted.len(),
        xanims.capture_gaps,
        anim_refused.len(),
        &anim_refused[..anim_refused.len().min(6)]
    ));

    // The weapon catalog, through Black Ops II's own capture.
    let mut catalog = WeaponCatalog::default();
    catalog.set_capture_ns(NS);
    let mut no_def = Vec::new();
    for (name, w) in &guns {
        let base = name.strip_suffix("_upgraded").unwrap_or(name);
        let display = format!(
            "WEAPON_BO3_{}{}",
            base.to_ascii_uppercase(),
            if name.ends_with("_upgraded") { "_UPGRADED" } else { "" }
        );
        match b.weapon_ref(w, name, &display) {
            Some(r) => {
                catalog.capture_t6(&r, None, &HashMap::new());
            }
            None => no_def.push(w.name.clone()),
        }
    }
    let captured = catalog.len();
    let mut weapons = catalog.into_build();
    weapons.stamp_namespace(NS);
    weapons.resolve_sz_xanim_edges(&xanims);
    weapons.resolve_fpv_mesh_edges(&fpv);
    weapons.resolve_world_model_edges(&world_guns);
    report.push(format!(
        "t7 guest guns: {captured} weapons captured, {} in the catalog ({} without a WeaponDef {:?}) [{:.1} s]",
        weapons.len(),
        no_def.len(),
        &no_def[..no_def.len().min(6)],
        started.elapsed().as_secs_f64()
    ));
    let Builder { catalog: material_seed, .. } = b;
    T7Guests { weapons, fpv, world_guns, xanims, material_seed, report }
}

/// bo2mp `iw4l check-guns`: each Black Ops III gun's fire sound and its
/// animations' notetrack sounds against mp_common's sound bank (an alias
/// is playable when it names a sound in the banks).
pub(crate) fn t7_sound_check(weapons: &asset_game::WeaponRegistry) -> Vec<String> {
    let Some(path) = asset_transport::games_root_from_env().ok().and_then(|r| asset_audio::bo3_mp_common(&r)) else {
        return vec!["t7 sounds: Black Ops III's mp_common is not installed".into()];
    };
    let bank = match asset_audio::bo3_weapon_sounds(&path) {
        Ok(b) => b,
        Err(e) => return vec![format!("t7 sounds: {e}")],
    };
    let playable = |alias: &str| {
        bank.sound_in(NS, alias)
            .is_some_and(|s| s.aliases.iter().any(|a| a.loaded.is_bound() || a.streamed.is_some()))
    };
    let (mut fire_ok, mut guns, mut nt_ok, mut nt_all) = (0, 0, 0, 0);
    let mut missing = Vec::new();
    for i in 1..=weapons.len() as u32 {
        if weapons.namespace_of(i) != Some(NS) {
            continue;
        }
        let Some(sounds) = weapons.sounds_of(i) else { continue };
        guns += 1;
        let fire = sounds.fire_player.as_deref().or(sounds.fire.as_deref());
        if fire.is_some_and(playable) {
            fire_ok += 1;
        } else {
            missing.push(format!("{} fire `{}`", weapons.name_of(i), fire.unwrap_or("-")));
        }
        for (_, alias) in &sounds.notetrack_sound_map {
            nt_all += 1;
            nt_ok += usize::from(playable(alias));
        }
    }
    vec![format!(
        "t7 sounds: {} alias lists; fire sound playable for {fire_ok} of {guns} weapons; notetrack sounds {nt_ok} of {nt_all}{}",
        bank.sounds.len(),
        if missing.is_empty() { String::new() } else { format!("; missing: {}", missing.iter().take(8).cloned().collect::<Vec<_>>().join(", ")) }
    )]
}

/// bo2mp `iw4l check-guns`: every Black Ops III gun animation decoded by
/// the engine's decoder, and whether its quaternions came out unit length.
pub(crate) fn t7_anim_check(xanims: &XAnimBuild) -> String {
    let (mut ok, mut bad) = (0usize, Vec::new());
    let names: Vec<String> = xanims.names_in(NS).map(str::to_owned).collect();
    for name in &names {
        match xanims.get(NS, name).map(|c| xmodel_runtime::AnimClip::from_parts(&c.parts)) {
            Some(Ok(_)) => ok += 1,
            Some(Err(e)) => bad.push(format!("{name}: {e:?}")),
            None => {}
        }
    }
    format!(
        "t7 animations: {ok} decode, {} refused{}",
        bad.len(),
        if bad.is_empty() { String::new() } else { format!(" ({})", bad.iter().take(3).cloned().collect::<Vec<_>>().join("; ")) }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `IW4L_GAMES=<root with Black Ops III> cargo test -p assets bo3_guns
    /// -- --ignored --nocapture`: Black Ops III's guns read and built as a
    /// match does.
    #[test]
    #[ignore]
    fn bo3_guns() {
        let g = walk_t7_guest_guns(true, MaterialCatalog::default(), &asset_transport::LoadProgress::default());
        for line in &g.report {
            println!("{line}");
        }
        println!("{}", t7_anim_check(&g.xanims));
        for i in 1..=g.weapons.len() as u32 {
            let anims = g.weapons.sz_xanims_of(i).map_or(0, |a| a.iter().flatten().count());
            println!(
                "  {} gun {:?} world {:?} anims {anims}",
                g.weapons.name_of(i),
                g.weapons.gun_xmodel_of(i),
                g.weapons.world_model_of(i)
            );
        }
    }

    #[test]
    fn fire_types_renumber() {
        assert_eq!(t7_fire_type(0, 0), 0);
        assert_eq!(t7_fire_type(1, 0), 1);
        assert_eq!(t7_fire_type(2, 3), 3);
        assert_eq!(t7_fire_type(3, 4), 4);
        assert_eq!(t7_fire_type(2, 9), 5);
    }

    #[test]
    fn colourless_materials_get_a_colour() {
        // A bullet case: black placeholder, brass specular.
        let brass = solid_colour("mc/mtl_wpn_t7_bullet_case", ",$black_color", Some([0.0, 0.0, 0.0, 0.0, 0.591, 0.527, 0.352, 1.0]));
        assert_eq!(brass, [0.591, 0.527, 0.352]);
        // A tritium dot: its tint.
        let red = solid_colour("mc/mtl_wpn_t7_sniper_locus_tritium_red", ",$white_diffuse", Some([0.905, 0.009, 0.009, 0.0, 0.5, 0.5, 0.5, 1.0]));
        assert_eq!(red, [0.905, 0.009, 0.009]);
        // White tint, no constants of use: by name, else dark.
        assert_eq!(solid_colour("mc/mtl_wpn_t7_xr2_tritium_red", ",$white_diffuse", None), [0.9, 0.02, 0.02]);
        assert_eq!(solid_colour("mc/mtl_x_base", "", Some([1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0])), [0.03, 0.03, 0.03]);
        assert_eq!(solid_levels([1.0, 0.0, 0.0]).data[..4], [255, 0, 0, 255]);
    }

    #[test]
    fn half_quats_become_fractions() {
        // 0x3c00 is 1.0, 0xb800 is -0.5.
        assert_eq!(half_to_frac_bits(0x3c00) as i16, 32767);
        assert_eq!(half_to_frac_bits(0xb800) as i16, -16384);
    }

    #[test]
    fn keyed_and_constant_quats_convert() {
        let mut r = asset_t6::XAnimRef {
            name: String::new(),
            numframes: 2,
            framerate: 30.0,
            looping: false,
            delta: false,
            delta_trans: Vec::new(),
            delta_quat: Vec::new(),
            bone_count: [0, 0, 1, 0, 1, 0, 0, 0, 2, 2],
            names: Vec::new(),
            notifies: Vec::new(),
            // Keyed full quat: size 1 (two keys); then a constant.
            data_short: vec![1, 0x3c00, 0, 0, 0],
            data_byte: vec![0, 1],
            data_int: Vec::new(),
            random_data_byte: Vec::new(),
            random_data_short: vec![0x3c00, 0, 0, 0, 0, 0x3c00, 0, 0],
            random_data_int: Vec::new(),
            indices: Vec::new(),
        };
        t7_quats_to_t6(&mut r).unwrap();
        assert_eq!(r.random_data_short[0] as i16, 32767);
        assert_eq!(r.random_data_short[5] as i16, 32767);
        assert_eq!(r.data_short[0], 1);
        assert_eq!(r.data_short[1] as i16, 32767);
    }
}
