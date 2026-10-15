//! bo2mp: one Call of Duty: Modern Warfare III (2023, `jup`) gun as a guest
//! gun in a Black Ops II match: the Renetti (`jup_jp12_pi_mike93`).
//!
//! `fastfile_jup::gun` reads its part models (view and world) from their
//! `_tr` zones and puts them together on the receiver's tags; this builds
//! them as Black Ops II models (one bone, `tag_weapon`, with `tag_flash`
//! and `tag_brass` where the gun has them) and their materials from the
//! install's `global_stream_mp`:
//!
//! - a gun material's pictures are its normal map (format 0x2f, BC7: a
//!   tangent-space normal in red, green and blue; turned into BC5 for
//!   Black Ops II's draw), its occlusion picture (0x30, usage 0x36:
//!   occlusion in alpha) and masks; its colour is not a picture but a
//!   tint in its constants (row 6, sRGB bytes), the game laying its camo
//!   and wear over it. The colour map is the tint darkened by the
//!   occlusion.
//! - a surface whose material the model points back to (a part's shared
//!   material, not read) draws in dark gun metal.
//!
//! The weapon itself is a copy of a Black Ops II gun of the same kind
//! (`DONOR`: the B23R, Black Ops II's Beretta 93R) with its own models,
//! name and handling (`jup_guest_rows`): Black Ops II's animations, arms,
//! effects and rules, Modern Warfare III's look. MW3's own view
//! animations are not read.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use asset_core::{AssetEdge, AssetNamespace, AssetRef, ZoneOwner};
use asset_material::{
    AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP,
    TS_NORMAL_MAP,
};
use asset_model::{FpvMeshBuild, WorldWeaponBuild};
use fastfile_jup::gun::{self, Gun, PartModel};
use fastfile_jup::image::{self, ImageRun};
use fastfile_jup::map::ZoneImage;
use fastfile_jup::map::materials::{MapImage, Packages};
use fastfile_jup::oodle::Decompressor;
use fastfile_t7::texture::ImageLevels;

use super::t7_map::{f16_bits, t6_unit_vec, t7_draw};

const NS: AssetNamespace = AssetNamespace::Jup;

/// The texture slots Black Ops II's lit model draw reads.
const COLOR_MAP_HASH: u32 = 0xa0ab_1041;
const NORMAL_MAP_HASH: u32 = 0x59d3_0d0f;

/// The largest picture side kept for the first-person and world models.
const FIRST_PERSON_MAX: u32 = 1024;
const WORLD_MAX: u32 = 512;

/// One part of a gun: its view and world part zones and the receiver's
/// tag it goes on (`None`: the receiver).
struct Part {
    view: &'static str,
    world: &'static str,
    tag: Option<&'static str>,
}

/// A Modern Warfare III gun played as a guest gun.
pub(crate) struct JupGun {
    /// Internal name without `jup_` (`bo2_profile::jup_names`).
    pub gun: &'static str,
    /// The Black Ops II weapon it is a copy of.
    pub donor: &'static str,
    /// Its own handling over the donor's (Black Ops II's numbers).
    pub fire_type: i32,
    pub fire_time_ms: i32,
    pub clip_size: i32,
    pub max_ammo: i32,
    parts: &'static [Part],
}

/// The Renetti: its receiver, slide and barrel, 15-round magazine and grip
/// panels (the default build).
pub(crate) const GUNS: &[JupGun] = &[JupGun {
    gun: "jp12_pi_mike93",
    donor: "beretta93r_mp",
    // Semi-automatic (Black Ops II's single shot), 15 rounds.
    fire_type: 1,
    fire_time_ms: 100,
    clip_size: 15,
    max_ammo: 75,
    parts: &[
        Part {
            view: "mp_av_jup_jp19_pi_mike93_rec_000_lod0_tr",
            world: "mp_aw_jup_jp19_pi_mike93_rec_000_lod0_tr",
            tag: None,
        },
        Part {
            view: "mp_av_jup_jp12_pi_mike93_bar_000_lod0_tr",
            world: "mp_aw_wm_jup_jp12_pi_mike93_bar_000_lod0_tr",
            tag: Some("tag_barrel_attach"),
        },
        Part {
            view: "mp_av_vm_jup_jp12_pi_mike93_mag_15_9mm_000_lod0_tr",
            world: "mp_aw_wm_jup_jp12_pi_mike93_mag_15_9mm_000_lod0_tr",
            tag: Some("tag_mag_attach"),
        },
        Part {
            view: "mp_av_jup_jp12_pi_mike93_pgrip_000_lod0_tr",
            world: "mp_aw_jup_jp12_pi_mike93_pgrip_000_lod0_tr",
            tag: Some("tag_pistolgrip_attach"),
        },
    ],
}];

/// A gun's first-person model's name.
pub(crate) fn view_model(gun: &str) -> String {
    format!("viewmodel_jup_{gun}")
}

/// A gun's world model's name.
pub(crate) fn world_model(gun: &str) -> String {
    format!("weapon_jup_{gun}")
}

/// The pictures of one material, decoded.
#[derive(Default)]
struct MaterialPixels {
    /// sRGB bytes.
    tint: [u8; 3],
    /// (width, height, RGBA8): the occlusion picture, the normal map.
    occlusion: Option<(u32, u32, Vec<u8>)>,
    normal: Option<(u32, u32, Vec<u8>)>,
}

/// What is read off the match's critical path.
pub(crate) enum JupPrefetch {
    Off(Vec<String>),
    Ready(Box<JupRead>),
}

pub(crate) struct JupRead {
    install: PathBuf,
    /// Per gun: (view gun, world gun).
    guns: Vec<(&'static JupGun, Gun, Gun)>,
    /// By material hash and size cap.
    materials: HashMap<(u64, u32), MaterialPixels>,
    report: Vec<String>,
}

pub(crate) struct JupGuests {
    pub fpv: FpvMeshBuild,
    pub world_guns: WorldWeaponBuild,
    pub material_seed: MaterialCatalog,
    pub report: Vec<String>,
}

fn oodle(install: &std::path::Path) -> Result<Box<dyn Decompressor>, String> {
    fastfile_jup::oodle::load(install).map_err(|e| e.to_string())
}

fn read_part(
    install: &std::path::Path,
    oodle: &dyn Decompressor,
    zone: &str,
) -> Result<PartModel, String> {
    let path = install.join("zone").join(format!("{zone}.ff"));
    let (_, bytes) = fastfile_jup::read_zone(&path, oodle).map_err(|e| format!("{zone}: {e}"))?;
    gun::read_part(&bytes).map_err(|e| format!("{zone}: {e}"))
}

/// The gun's parts read and put together (view, else world zones).
fn read_gun(
    install: &std::path::Path,
    oodle: &dyn Decompressor,
    g: &JupGun,
    view: bool,
) -> Result<Gun, String> {
    let zone = |p: &Part| if view { p.view } else { p.world };
    let receiver = g
        .parts
        .iter()
        .find(|p| p.tag.is_none())
        .ok_or("no receiver")?;
    let rec = read_part(install, oodle, zone(receiver))?;
    let mut parts = Vec::new();
    for p in g.parts.iter().filter(|p| p.tag.is_some()) {
        parts.push((
            read_part(install, oodle, zone(p))?,
            p.tag.unwrap_or_default(),
        ));
    }
    let refs: Vec<(&PartModel, &str)> = parts.iter().map(|(m, t)| (m, *t)).collect();
    Ok(gun::assemble(&rec, &refs))
}

fn u64_at(z: &[u8], o: usize) -> u64 {
    z.get(o..o + 8)
        .map_or(0, |b| u64::from_le_bytes(b.try_into().unwrap_or([0; 8])))
}

/// A material's header: its hash then the word most materials hold.
const MATERIAL_WORD: [u8; 4] = [0xc0, 0xb6, 0x73, 0x2c];
const MATERIAL_HEADER: usize = 0x78;

/// The materials' pictures from `global_stream_mp`, where gun materials
/// are defined. A picture pointer leads to the image's entry of the
/// asset list: the list's place in the pointers' block is found from the
/// pointers themselves (the one base that makes the most of them images).
fn read_materials(
    install: &std::path::Path,
    oodle: &dyn Decompressor,
    wanted: &[(u64, u32)],
    report: &mut Vec<String>,
) -> Result<HashMap<(u64, u32), MaterialPixels>, String> {
    let path = install.join("zone").join("global_stream_mp.ff");
    let (_, bytes) =
        fastfile_jup::read_zone(&path, oodle).map_err(|e| format!("global_stream_mp: {e}"))?;
    let zone = ZoneImage::parse(bytes).map_err(|e| format!("global_stream_mp: {e}"))?;
    let z = &zone.bytes;
    let hashes: std::collections::HashSet<u64> = wanted.iter().map(|w| w.0).collect();
    let mut at: HashMap<u64, usize> = HashMap::new();
    let mut i = 8;
    while let Some(p) = z
        .get(i..)
        .and_then(|r| r.windows(4).position(|w| w == MATERIAL_WORD))
    {
        let o = i + p - 8;
        let h = u64_at(z, o);
        if hashes.contains(&h) {
            at.entry(h).or_insert(o);
        }
        i += p + 1;
    }
    let pictures = |m: usize| -> Vec<u64> {
        (0..usize::from(z[m + 0x1b]))
            .map(|k| u64_at(z, m + MATERIAL_HEADER + 8 * k))
            .collect()
    };
    let all: Vec<u64> = at.values().flat_map(|&m| pictures(m)).collect();
    let fits = |base: usize, p: u64| {
        let off = ((p & 0xffff_ffff) as usize).wrapping_sub(1);
        off >= base && (off - base) % 16 == 0 && zone.types.get((off - base) / 16) == Some(&21)
    };
    let base = (zone.list_at.saturating_sub(0x200)..zone.list_at + 0x40)
        .max_by_key(|&b| {
            (
                all.iter().filter(|&&p| fits(b, p)).count(),
                std::cmp::Reverse(b),
            )
        })
        .unwrap_or(zone.list_at);
    let mut nth = vec![usize::MAX; zone.types.len()];
    let mut k = 0;
    for (i, &t) in zone.types.iter().enumerate() {
        if t == 21 {
            nth[i] = k;
            k += 1;
        }
    }
    let run = ImageRun::find(z, k).ok_or("global_stream_mp: image run not found")?;
    let paks = Packages::open(install);
    let mut out = HashMap::new();
    let (mut pics, mut missing) = (0usize, 0usize);
    for &(hash, max) in wanted {
        let Some(&m) = at.get(&hash) else {
            report.push(format!(
                "jup guest guns: material {hash:016x} not in global_stream_mp"
            ));
            continue;
        };
        let mut px = MaterialPixels::default();
        // (Row 6 of the constants: the colour, then the pictures' pointers.)
        let consts = m + MATERIAL_HEADER + 8 * usize::from(z[m + 0x1b]);
        if usize::from(z[m + 0x1c]) > 6
            && let Some(c) = z.get(consts + 6 * 16 + 4..consts + 6 * 16 + 7)
        {
            px.tint = [c[0], c[1], c[2]];
        }
        for p in pictures(m) {
            if !fits(base, p) {
                continue;
            }
            let asset = (((p & 0xffff_ffff) as usize) - 1 - base) / 16;
            let Some(rec) = run
                .at
                .get(nth[asset])
                .and_then(|&o| image::parse(z, o).ok().flatten())
            else {
                continue;
            };
            let slot = match (rec.format, rec.usage) {
                (0x2f, _) => &mut px.normal,
                (0x30, 0x36) => &mut px.occlusion,
                _ => continue,
            };
            let img = MapImage {
                hash: rec.hash,
                width: rec.width,
                height: rec.height,
                format: rec.format,
                groups: rec.groups.clone(),
                inline: rec.inline.map(|(o, n)| z[o..o + n].to_vec()),
            };
            match paks.level(&img, max, oodle) {
                Ok(l) => {
                    match fastfile_jup::cards::decode_bcn(l.dxgi, l.width, l.height, &l.data) {
                        Some(rgba) => {
                            pics += 1;
                            *slot = Some((l.width, l.height, rgba));
                        }
                        None => missing += 1,
                    }
                }
                Err(_) => missing += 1,
            }
        }
        out.insert((hash, max), px);
    }
    report.push(format!(
        "jup guest guns: materials: {} of {} found, {pics} pictures, {missing} not in the install",
        out.len(),
        wanted.len()
    ));
    Ok(out)
}

/// bo2mp: reads the Modern Warfare III guest guns' parts and materials
/// from the install (`wanted`: Black Ops II rules with guest guns on).
pub(crate) fn prefetch_jup_guest_guns(wanted: bool) -> JupPrefetch {
    if !wanted {
        return JupPrefetch::Off(vec!["jup guest guns: off".into()]);
    }
    let Some(install) = asset_audio::jup_install() else {
        return JupPrefetch::Off(vec![
            "jup guest guns: Modern Warfare III (2023) is not installed".into(),
        ]);
    };
    let started = std::time::Instant::now();
    let oodle = match oodle(&install) {
        Ok(o) => o,
        Err(e) => return JupPrefetch::Off(vec![format!("jup guest guns: {e}")]),
    };
    let mut report = Vec::new();
    let mut guns = Vec::new();
    let mut wanted_mats: Vec<(u64, u32)> = Vec::new();
    for g in GUNS {
        match (
            read_gun(&install, oodle.as_ref(), g, true),
            read_gun(&install, oodle.as_ref(), g, false),
        ) {
            (Ok(view), Ok(world)) => {
                for (gun, max) in [(&view, FIRST_PERSON_MAX), (&world, WORLD_MAX)] {
                    for s in &gun.surfaces {
                        if let Some(m) = s.material
                            && !wanted_mats.contains(&(m, max))
                        {
                            wanted_mats.push((m, max));
                        }
                    }
                }
                report.push(format!(
                    "jup guest guns: {}: view {} surfaces {} bones, world {} surfaces{}",
                    g.gun,
                    view.surfaces.len(),
                    view.bones.len(),
                    world.surfaces.len(),
                    if view.missing.is_empty() && world.missing.is_empty() {
                        String::new()
                    } else {
                        format!(" (not placed: {:?} {:?})", view.missing, world.missing)
                    }
                ));
                guns.push((g, view, world));
            }
            (Err(e), _) | (_, Err(e)) => report.push(format!("jup guest guns: {}: {e}", g.gun)),
        }
    }
    let materials = match read_materials(&install, oodle.as_ref(), &wanted_mats, &mut report) {
        Ok(m) => m,
        Err(e) => {
            report.push(format!("jup guest guns: {e} (drawn without pictures)"));
            HashMap::new()
        }
    };
    report.push(format!(
        "jup guest guns: read in {:.1} s",
        started.elapsed().as_secs_f64()
    ));
    JupPrefetch::Ready(Box::new(JupRead {
        install,
        guns,
        materials,
        report,
    }))
}

/// sRGB byte to linear.
fn linear(c: u8) -> f32 {
    let s = f32::from(c) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb(x: f32) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let s = if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// The colour map: the tint, darkened by the occlusion (alpha of the
/// occlusion picture), as sRGB RGBA8.
fn colour_levels(px: &MaterialPixels) -> ImageLevels {
    let tint = px.tint.map(linear);
    match &px.occlusion {
        Some((w, h, rgba)) => {
            let data = rgba
                .chunks_exact(4)
                .flat_map(|p| {
                    let ao = f32::from(p[3]) / 255.0;
                    [
                        srgb(tint[0] * ao),
                        srgb(tint[1] * ao),
                        srgb(tint[2] * ao),
                        255,
                    ]
                })
                .collect();
            ImageLevels {
                format: 29,
                width: *w,
                height: *h,
                levels: 1,
                data,
                part: None,
            }
        }
        None => ImageLevels {
            format: 29,
            width: 4,
            height: 4,
            levels: 1,
            data: [px.tint[0], px.tint[1], px.tint[2], 255].repeat(16),
            part: None,
        },
    }
}

/// Dark gun metal, for surfaces whose material is not read.
const GUN_METAL: [u8; 3] = [0x1c, 0x1c, 0x1e];

struct Builder {
    catalog: MaterialCatalog,
    owner: ZoneOwner,
    images: HashMap<String, Option<usize>>,
    materials: HashMap<(Option<u64>, u32), Option<usize>>,
    failures: Vec<String>,
}

impl Builder {
    fn image(
        &mut self,
        key: String,
        semantic: u8,
        make: impl FnOnce() -> Option<ImageLevels>,
    ) -> Option<usize> {
        if let Some(slot) = self.images.get(&key) {
            return *slot;
        }
        let colour = semantic == TS_COLOR_MAP;
        let slot = make().and_then(|l| {
            let px = fastfile_iw6::image::ImagePixels {
                width: l.width,
                height: l.height,
                levels: l.levels,
                dxgi_format: l.format,
                data: l.data,
            };
            match super::iw6::gpu_image(&px, colour) {
                Ok(gpu) => Some(self.catalog.link_image(AuthoredImage {
                    namespace: NS,
                    name: AssetRef::Real(key.clone()),
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
                })),
                Err(e) => {
                    self.failures.push(format!("{key}: {e}"));
                    None
                }
            }
        });
        self.images.insert(key, slot);
        slot
    }

    /// A material of the gun into the catalog (`None`: dark gun metal).
    fn material(
        &mut self,
        hash: Option<u64>,
        max: u32,
        read: &HashMap<(u64, u32), MaterialPixels>,
    ) -> Option<usize> {
        if let Some(&m) = self.materials.get(&(hash, max)) {
            return m;
        }
        let fallback = MaterialPixels {
            tint: GUN_METAL,
            ..Default::default()
        };
        let px = hash.and_then(|h| read.get(&(h, max))).unwrap_or(&fallback);
        let name = hash.map_or_else(|| "jup_gun_metal".to_owned(), |h| format!("jup_{h:016x}"));
        let mut textures = Vec::new();
        if let Some(slot) = self.image(format!("{name}#{max}"), TS_COLOR_MAP, || {
            Some(colour_levels(px))
        }) {
            textures.push(MaterialTextureBinding {
                name_hash: COLOR_MAP_HASH,
                name_start: 0,
                name_end: 0,
                sampler_state: 0x13,
                semantic: TS_COLOR_MAP,
                image: Some(slot),
            });
        }
        if let Some((w, h, rgba)) = &px.normal {
            let levels = ImageLevels {
                format: 28,
                width: *w,
                height: *h,
                levels: 1,
                data: rgba.clone(),
                part: None,
            };
            if let Some(slot) = self.image(format!("{name}#{max}#n"), TS_NORMAL_MAP, || {
                fastfile_t7::texture::normal_map_bc5(&levels)
            }) {
                textures.push(MaterialTextureBinding {
                    name_hash: NORMAL_MAP_HASH,
                    name_start: 0,
                    name_end: 0,
                    sampler_state: 0x13,
                    semantic: TS_NORMAL_MAP,
                    image: Some(slot),
                });
            }
        }
        let draw = t7_draw(&name, "", true);
        let index = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(format!("{name}#{max}")),
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
        self.materials.insert((hash, max), Some(index));
        Some(index)
    }
}

/// One vertex as Black Ops II's packed one (32 bytes: position, binormal
/// sign, colour, UV halves, normal, tangent).
fn t6_vertex(p: [f32; 3], sign: f32, uv: [f32; 2], n: [f32; 3], t: [f32; 3]) -> [u8; 32] {
    let mut v = [0u8; 32];
    for k in 0..3 {
        v[4 * k..4 * k + 4].copy_from_slice(&p[k].to_le_bytes());
    }
    v[12..16].copy_from_slice(&sign.to_le_bytes());
    v[16..20].copy_from_slice(&[255; 4]);
    let tex = u32::from(f16_bits(uv[0])) | u32::from(f16_bits(uv[1])) << 16;
    v[20..24].copy_from_slice(&tex.to_le_bytes());
    v[24..28].copy_from_slice(&t6_unit_vec(n).to_le_bytes());
    v[28..32].copy_from_slice(&t6_unit_vec(t).to_le_bytes());
    v
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Each vertex's binormal sign: whether cross(normal, tangent) runs along
/// the UVs' v on its triangles.
fn binormal_signs(s: &gun::PartSurface) -> Vec<f32> {
    let mut acc = vec![[0.0f32; 3]; s.positions.len()];
    for t in s.indices.chunks_exact(3) {
        let [a, b, c] = [t[0], t[1], t[2]].map(|i| i as usize);
        if a.max(b).max(c) >= s.positions.len() {
            continue;
        }
        let (e1, e2) = (
            sub(s.positions[b], s.positions[a]),
            sub(s.positions[c], s.positions[a]),
        );
        let (du1, dv1) = (s.uvs[b][0] - s.uvs[a][0], s.uvs[b][1] - s.uvs[a][1]);
        let (du2, dv2) = (s.uvs[c][0] - s.uvs[a][0], s.uvs[c][1] - s.uvs[a][1]);
        let r = du1 * dv2 - du2 * dv1;
        if r.abs() < 1e-12 {
            continue;
        }
        let bt = [
            (e2[0] * du1 - e1[0] * du2) / r,
            (e2[1] * du1 - e1[1] * du2) / r,
            (e2[2] * du1 - e1[2] * du2) / r,
        ];
        for &v in &[a, b, c] {
            for k in 0..3 {
                acc[v][k] += bt[k];
            }
        }
    }
    (0..s.positions.len())
        .map(|v| {
            if dot(cross(s.normals[v], s.tangents[v]), acc[v]) >= 0.0 {
                1.0
            } else {
                -1.0
            }
        })
        .collect()
}

/// Where the gun's model space puts Black Ops II's `tag_weapon` (the
/// hand): the receiver's grip tag. Black Ops II's guns are modelled with
/// the hand at the origin; Modern Warfare III's receivers sit ahead of
/// theirs.
fn hand_anchor(g: &Gun) -> [f32; 3] {
    g.bone("tag_pistolgrip_attach").map_or([0.0; 3], |b| b.pos)
}

/// The gun as a Black Ops II model: its surfaces on `tag_weapon`, and the
/// tags Black Ops II's effects and shells come out of.
fn model_ref(name: &str, g: &Gun, offset: [f32; 3]) -> Option<asset_t6::XModelRef> {
    let anchor = hand_anchor(g);
    let place = |p: [f32; 3]| {
        [
            p[0] - anchor[0] + offset[0],
            p[1] - anchor[1] + offset[1],
            p[2] - anchor[2] + offset[2],
        ]
    };
    let mut bone_names = vec!["tag_weapon".to_owned()];
    let mut base_mat: Vec<([f32; 4], [f32; 3])> = vec![([0.0, 0.0, 0.0, 1.0], [0.0; 3])];
    for tag in ["tag_flash", "tag_brass"] {
        if let Some(b) = g.bone(tag) {
            bone_names.push(tag.to_owned());
            base_mat.push((b.rot, place(b.pos)));
        }
    }
    let (mut mins, mut maxs) = ([f32::MAX; 3], [f32::MIN; 3]);
    let mut lod0 = Vec::new();
    for s in &g.surfaces {
        let n = s.positions.len();
        if n == 0 || n > usize::from(u16::MAX) {
            continue;
        }
        let signs = binormal_signs(s);
        let mut verts = Vec::with_capacity(n * 32);
        for v in 0..n {
            let p = place(s.positions[v]);
            for k in 0..3 {
                mins[k] = mins[k].min(p[k]);
                maxs[k] = maxs[k].max(p[k]);
            }
            verts.extend_from_slice(&t6_vertex(
                p,
                signs[v],
                s.uvs[v],
                s.normals[v],
                s.tangents[v],
            ));
        }
        lod0.push(asset_t6::XSurfaceRef {
            verts,
            vert_count: n as u16,
            indices: s.indices.iter().map(|&i| i as u16).collect(),
            vert_lists: Vec::new(),
            flags: 0,
            material: None,
            blend_counts: [n as i16, 0, 0, 0],
            blend: vec![0; n],
            part_bits: [1, 0, 0, 0, 0, 0],
        });
    }
    if lod0.is_empty() {
        return None;
    }
    let n_bones = bone_names.len();
    let radius = (0..3)
        .map(|c| mins[c].abs().max(maxs[c].abs()).powi(2))
        .sum::<f32>()
        .sqrt();
    Some(asset_t6::XModelRef {
        name: name.to_owned(),
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
        num_root_bones: 1,
        parent_list: (1..n_bones).map(|i| i as u8).collect(),
        quats: base_mat[1..]
            .iter()
            .map(|(q, _)| q.map(|x| (x.clamp(-1.0, 1.0) * 32767.0).round() as i16))
            .collect(),
        trans: base_mat[1..].iter().map(|(_, t)| *t).collect(),
        bone_names,
        part_classification: vec![0; n_bones],
        bone_info: vec![None; n_bones],
        base_mat,
        contents: 0,
        coll_lod: -1,
        coll_surfs: Vec::new(),
        phys_preset: None,
    })
}

/// bo2mp: the Modern Warfare III guest guns' models from what
/// `prefetch_jup_guest_guns` read, their materials linked into
/// `material_seed`.
pub(crate) fn build_jup_guest_guns(
    prefetch: JupPrefetch,
    material_seed: MaterialCatalog,
) -> JupGuests {
    let read = match prefetch {
        JupPrefetch::Off(report) => {
            return JupGuests {
                fpv: FpvMeshBuild::default(),
                world_guns: WorldWeaponBuild::default(),
                material_seed,
                report,
            };
        }
        JupPrefetch::Ready(r) => r,
    };
    let JupRead {
        install,
        guns,
        materials,
        mut report,
    } = *read;
    let mut b = Builder {
        catalog: material_seed,
        owner: ZoneOwner::from_zone_path(&install.join("zone").join("global_stream_mp.ff")),
        images: HashMap::new(),
        materials: HashMap::new(),
        failures: Vec::new(),
    };
    let mut fpv = FpvMeshBuild::default();
    fpv.set_capture_ns(NS);
    let mut world_guns = WorldWeaponBuild::default();
    world_guns.set_capture_ns(NS);
    for (g, view, world) in &guns {
        for (gun, name, max, first) in [
            (view, view_model(g.gun), FIRST_PERSON_MAX, true),
            (world, world_model(g.gun), WORLD_MAX, false),
        ] {
            let Some(mut xref) = model_ref(&name, gun, [0.0; 3]) else {
                report.push(format!("jup guest guns: {name}: no surfaces"));
                continue;
            };
            let linked: Vec<Option<usize>> = gun
                .surfaces
                .iter()
                .filter(|s| !s.positions.is_empty())
                .map(|s| b.material(s.material, max, &materials))
                .collect();
            let key = |k: usize| asset_t6::AssetKey {
                ty: fastfile_t6::AssetType::Material,
                index: k,
            };
            for (i, s) in xref.lod0.iter_mut().enumerate() {
                s.material = linked.get(i).copied().flatten().map(|_| key(i));
            }
            xref.materials = xref.lod0.iter().map(|s| s.material).collect();
            match asset_model::capture_model_skel_t6(&xref, |k| {
                linked.get(k.index).copied().flatten()
            }) {
                Some(skel) if first => fpv.insert_in(NS, skel, Some(&b.catalog)),
                Some(skel) => world_guns.insert_in(NS, skel, Some(&b.catalog)),
                None => report.push(format!("jup guest guns: {name}: model not built")),
            }
            report.push(format!(
                "jup guest guns: {name}: {} surfaces, bounds {:?}..{:?}, bones {:?}",
                xref.lod0.len(),
                xref.mins.map(|v| (v * 10.0).round() / 10.0),
                xref.maxs.map(|v| (v * 10.0).round() / 10.0),
                xref.bone_names
            ));
        }
    }
    if !b.failures.is_empty() {
        report.push(format!(
            "jup guest guns: pictures not built: {:?}",
            &b.failures[..b.failures.len().min(6)]
        ));
    }
    let Builder {
        catalog: material_seed,
        ..
    } = b;
    JupGuests {
        fpv,
        world_guns,
        material_seed,
        report,
    }
}

/// bo2mp `iw4l check-guns`: the Modern Warfare III guest guns read and
/// built as a match does (their weapons are copies made in the match:
/// the donor each is made from is named).
pub(crate) fn jup_check() -> Vec<String> {
    let guests = build_jup_guest_guns(prefetch_jup_guest_guns(true), MaterialCatalog::default());
    let mut out: Vec<String> = guests.report.iter().map(|l| format!("  {l}")).collect();
    for g in GUNS {
        let view = guests.fpv.index_by_name(NS, &view_model(g.gun)).is_some();
        let world = guests
            .world_guns
            .index_by_name(NS, &world_model(g.gun))
            .is_some();
        out.push(format!(
            "jup_{:<20} {} first person {}, world {}, a copy of {} with its handling (fire type {}, {} rounds)",
            g.gun,
            if view && world { "OK" } else { "MISSING" },
            if view { "built" } else { "missing" },
            if world { "built" } else { "missing" },
            g.donor,
            g.fire_type,
            g.clip_size
        ));
    }
    out
}

/// bo2mp: the Modern Warfare III guest guns as weapons: each a copy of its
/// Black Ops II donor (animations, arms, effects, sounds, damage) with its
/// own models, name and handling. Before the guest names are given and
/// anything resolves against the catalog.
pub(crate) fn jup_guest_rows(weapons: &mut asset_game::WeaponBuild, report: &mut Vec<String>) {
    for g in GUNS {
        let Some(donor) = weapons
            .resolve_index(g.donor)
            .ok()
            .flatten()
            .filter(|&i| i != 0)
        else {
            report.push(format!("jup guest guns: {}: no {} to copy", g.gun, g.donor));
            continue;
        };
        let id = weapons.add_guest_copy(
            donor,
            asset_game::GuestCopy {
                namespace: NS,
                name: format!("{}_mp", g.gun),
                display_name_key: format!("WEAPON_JUP_{}", g.gun.to_ascii_uppercase()),
                gun_xmodel: view_model(g.gun),
                world_model: world_model(g.gun),
                fire_type: g.fire_type,
                fire_time_ms: g.fire_time_ms,
                clip_size: g.clip_size,
                max_ammo: g.max_ammo,
            },
        );
        report.push(format!(
            "jup guest guns: {}_mp from {} (row {id})",
            g.gun, g.donor
        ));
    }
}
