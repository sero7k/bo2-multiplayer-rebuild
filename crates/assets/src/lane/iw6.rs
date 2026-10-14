//! bo2mp: Call of Duty: Ghosts' guns as guest guns in a Black Ops II match
//! (`gh_<gun>_mp`), from Ghosts' `common_core_mp` zone.
//!
//! `fastfile_iw6`'s exact walk reads the zone; this turns what it kept into
//! the engine's catalogs the way Black Ops II's own guns go in:
//!
//! - models into Black Ops II's captured form (`asset_t6::XModelRef`) and
//!   through its skinned-model builder: Ghosts' packed vertex is BO2's but
//!   for its unit vectors (unsigned 10:10:10 here, signed there), and its
//!   eight blend buckets fold into BO2's four;
//! - animations into BO2's captured form (the same IW-family part types and
//!   packing) under the Ghosts namespace;
//! - materials onto BO2's draw path (`T6Draw` from the technique set's
//!   name: Ghosts' `m_`/`mc_` model techniques read like BO2's `mc_` ones)
//!   with their colour, normal and specular maps, whose pixels come from
//!   the zone or, streamed, from `imagefile<N>.pak` beside it;
//! - weapons through `WeaponCatalog::capture_iw6`.
//!
//! Arms are Black Ops II's (the catalog's hands resolve to BO2's).

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use asset_anim::XAnimBuild;
use asset_core::{AssetEdge, AssetNamespace, AssetRef, T6Blend, T6Cull, T6Draw, ZoneOwner};
use asset_game::{Iw6WeaponView, WeaponBuild, WeaponCatalog};
use asset_material::{
    AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP,
    TS_NORMAL_MAP, TS_SPECULAR_MAP,
};
use asset_model::{FpvMeshBuild, WorldWeaponBuild};
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use fastfile_iw6::AssetType as G;
use fastfile_iw6::walk::{AssetId, Capture, ImageRec, MaterialRec, XAnimRec, XModelRec};

/// `R_HashString` of the texture slots Black Ops II's lit model draw reads
/// (Ghosts hashes them the same way).
const COLOR_MAP_HASH: u32 = 0xa0ab_1041;
const NORMAL_MAP_HASH: u32 = 0x59d3_0d0f;
const SPECULAR_MAP_HASH: u32 = 0x34ec_ccb3;

/// The largest level kept of a gun's pictures (first person), and of the
/// third-person models' (`BO2MP_GH_TEXTURE=<size>` changes the first).
const FIRST_PERSON_MAX: u32 = 1024;
const WORLD_MAX: u32 = 512;

/// Ghosts' guns, ready to join the match's catalogs.
pub(crate) struct Iw6Guests {
    pub weapons: WeaponBuild,
    pub fpv: FpvMeshBuild,
    pub world_guns: WorldWeaponBuild,
    pub xanims: XAnimBuild,
    pub material_seed: MaterialCatalog,
    pub report: Vec<String>,
}

impl Iw6Guests {
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

/// Ghosts' `zone/common_core_mp.ff`, if Ghosts is installed.
pub(crate) fn ghosts_common_core() -> Result<PathBuf, String> {
    let root = asset_transport::games_root_from_env().ok().map(|r| r.0);
    let extra: Vec<PathBuf> = asset_transport::game_paths::get("ghosts")
        .into_iter()
        .collect();
    let install = fastfile_iw6::install::find_install(
        root.as_deref().filter(|p| !p.as_os_str().is_empty()),
        &extra,
    )
    .ok_or("Ghosts is not installed (no zone/code_post_gfx_mp.ff found)")?;
    let path = install.join("zone").join("common_core_mp.ff");
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("{} is missing", path.display()))
    }
}

/// Ghosts' common_core_mp read ahead of the build (inflated and walked),
/// or why there is none.
pub(crate) enum Iw6Prefetch {
    Off(Vec<String>),
    Ready(Box<Iw6Zone>),
}

pub(crate) struct Iw6Zone {
    path: PathBuf,
    zone: fastfile_iw6::Zone,
    cap: Capture,
    report: Vec<String>,
    started: std::time::Instant,
}

/// bo2mp: reads Ghosts' common_core_mp for its guns (`wanted`: Black Ops II
/// rules with guest guns on), off the match's critical path; the build
/// (`build_iw6_guest_guns`) links them after.
pub(crate) fn prefetch_iw6_guest_guns(wanted: bool) -> Iw6Prefetch {
    if !wanted {
        return Iw6Prefetch::Off(vec!["iw6 guest guns: off".into()]);
    }
    let path = match ghosts_common_core() {
        Ok(path) => path,
        Err(e) => return Iw6Prefetch::Off(vec![format!("iw6 guest guns: {e}")]),
    };
    let started = std::time::Instant::now();
    let zone = match fastfile_iw6::Zone::open(&path) {
        Ok(zone) => zone,
        Err(e) => {
            return Iw6Prefetch::Off(vec![format!(
                "iw6 guest guns: open {}: {e}",
                path.display()
            )]);
        }
    };
    let (cap, stop) = fastfile_iw6::walk::walk_zone(&zone);
    let report = vec![format!(
        "iw6 guest guns: walked {} of {} assets of {} ({} weapons, {} models, {} animations, {} materials, {} images) [{:.1} s]{}",
        cap.walked,
        zone.assets.assets.len(),
        path.display(),
        cap.weapons.len(),
        cap.xmodels.len(),
        cap.xanims.len(),
        cap.materials.len(),
        cap.images.len(),
        started.elapsed().as_secs_f64(),
        stop.map(|s| format!(
            "; stopped at asset {} ({:?}): {}",
            s.asset_index, s.asset_type, s.error
        ))
        .unwrap_or_default()
    )];
    Iw6Prefetch::Ready(Box::new(Iw6Zone {
        path,
        zone,
        cap,
        report,
        started,
    }))
}

/// bo2mp: the match's Ghosts guest guns from what `prefetch_iw6_guest_guns`
/// read, their materials linked into `material_seed`.
pub(crate) fn build_iw6_guest_guns(
    prefetch: Iw6Prefetch,
    material_seed: MaterialCatalog,
    progress: &asset_transport::LoadProgress,
) -> Iw6Guests {
    let zone = match prefetch {
        Iw6Prefetch::Off(report) => return Iw6Guests::empty(material_seed, report),
        Iw6Prefetch::Ready(zone) => zone,
    };
    let stage = progress.begin_scoped(
        asset_transport::StageId::CommonAssets,
        "iw6_guest_weapons",
        None,
    );
    let guests = load_iw6_guests(*zone, material_seed);
    stage.done();
    guests
}

/// bo2mp: the match's Ghosts guest guns read and built in one go
/// (`iw4l check-guns`).
pub(crate) fn walk_iw6_guest_guns(
    wanted: bool,
    material_seed: MaterialCatalog,
    progress: &asset_transport::LoadProgress,
) -> Iw6Guests {
    build_iw6_guest_guns(prefetch_iw6_guest_guns(wanted), material_seed, progress)
}

/// Ghosts' guns from its walked `common_core_mp`, their materials linked
/// into `material_seed`.
fn load_iw6_guests(z: Iw6Zone, material_seed: MaterialCatalog) -> Iw6Guests {
    let Iw6Zone {
        path,
        zone,
        cap,
        mut report,
        started,
    } = z;
    let path = path.as_path();
    let mut b = Builder {
        cap: &cap,
        image: &zone.image,
        strings: &zone.assets.script_strings,
        paks: fastfile_iw6::image::ImagePaks::new(path.parent().unwrap_or(Path::new("."))),
        catalog: material_seed,
        owner: ZoneOwner::from_zone_path(path),
        materials: HashMap::new(),
        images: HashMap::new(),
        image_failures: Vec::new(),
        decoded: 0,
    };
    let first_person_max = std::env::var("BO2MP_GH_TEXTURE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(FIRST_PERSON_MAX);

    // The guns: Ghosts' own multiplayer weapons (`iw6_*_mp`).
    let guns: Vec<&fastfile_iw6::walk::WeaponRec> = cap
        .weapons
        .iter()
        .filter(|w| {
            let n = w.name.to_ascii_lowercase();
            n.starts_with("iw6_") && n.ends_with("_mp")
        })
        .collect();
    let mut first_person: BTreeSet<u32> = BTreeSet::new();
    let mut third_person: BTreeSet<u32> = BTreeSet::new();
    for w in &guns {
        first_person.extend(w.gun_models.first().copied().flatten().map(|id| id.index));
        third_person.extend(w.world_models.first().copied().flatten().map(|id| id.index));
        for (off, id) in &w.models {
            use fastfile_iw6::layout::WeaponDef as d;
            if [d::projectileModel, d::rocketModel, d::knifeModel].contains(off)
                && id.ty == G::XModel
            {
                first_person.insert(id.index);
            }
        }
    }

    let mut fpv = FpvMeshBuild::default();
    fpv.set_capture_ns(AssetNamespace::Iw6);
    let mut world_guns = WorldWeaponBuild::default();
    world_guns.set_capture_ns(AssetNamespace::Iw6);
    let mut unreadable = Vec::new();
    for (set, max, first) in [
        (&first_person, first_person_max, true),
        (&third_person, WORLD_MAX, false),
    ] {
        for &mi in set {
            let Some(model) = cap.xmodels.get(mi as usize) else {
                continue;
            };
            let Some((xref, keys)) = b.model_ref(model) else {
                unreadable.push(format!("{} (no surfaces)", model.name));
                continue;
            };
            let linked: HashMap<usize, usize> = keys
                .into_iter()
                .filter_map(|k| Some((k, b.material(k, max)?)))
                .collect();
            match asset_model::capture_model_skel_t6(&xref, |k| linked.get(&k.index).copied()) {
                Some(skel) if first => fpv.insert_in(AssetNamespace::Iw6, skel, Some(&b.catalog)),
                Some(skel) => world_guns.insert_in(AssetNamespace::Iw6, skel, Some(&b.catalog)),
                None => unreadable.push(format!("{} (unreadable)", model.name)),
            }
        }
    }
    report.push(format!(
        "iw6 guest guns: models: {} first person, {} third person built; {} not {:?}",
        fpv.len(),
        world_guns.len(),
        unreadable.len(),
        &unreadable[..unreadable.len().min(8)]
    ));
    report.push(format!(
        "iw6 guest guns: materials: {} linked, {} pictures decoded, {} failed {:?}",
        b.materials.len(),
        b.decoded,
        b.image_failures.len(),
        &b.image_failures[..b.image_failures.len().min(6)]
    ));

    // Animations every gun names.
    let mut xanims = XAnimBuild::default();
    xanims.set_capture_ns(AssetNamespace::Iw6);
    let mut wanted_anims: BTreeSet<u32> = BTreeSet::new();
    for w in &guns {
        for id in w.anims.iter().chain(&w.anims_right).flatten() {
            wanted_anims.insert(id.index);
        }
    }
    let mut anim_n = 0usize;
    for &ai in &wanted_anims {
        if let Some(a) = cap.xanims.get(ai as usize)
            && let Some(r) = b.anim_ref(a)
        {
            anim_n += usize::from(xanims.insert_t6_as(AssetNamespace::Iw6, &r));
        }
    }
    report.push(format!(
        "iw6 guest guns: animations: {anim_n} of {} named kept ({} gaps)",
        wanted_anims.len(),
        xanims.capture_gaps
    ));

    // The weapon catalog.
    let mut catalog = WeaponCatalog::default();
    catalog.set_capture_ns(AssetNamespace::Iw6);
    let mut no_def = Vec::new();
    for w in &guns {
        let view = b.weapon_view(w);
        if view.def.is_empty() {
            no_def.push(w.name.clone());
        }
        catalog.capture_iw6(&view);
    }
    let captured = catalog.len();
    let mut weapons = catalog.into_build();
    weapons.stamp_namespace(AssetNamespace::Iw6);
    weapons.resolve_sz_xanim_edges(&xanims);
    weapons.resolve_fpv_mesh_edges(&fpv);
    weapons.resolve_world_model_edges(&world_guns);
    report.push(format!(
        "iw6 guest guns: {captured} weapons captured, {} in the catalog ({} without a WeaponDef {:?}) [{:.1} s]",
        weapons.len(),
        no_def.len(),
        &no_def[..no_def.len().min(6)],
        started.elapsed().as_secs_f64()
    ));
    let Builder {
        catalog: material_seed,
        ..
    } = b;
    Iw6Guests {
        weapons,
        fpv,
        world_guns,
        xanims,
        material_seed,
        report,
    }
}

/// bo2mp `iw4l check-guns`: whether each Ghosts gun's sounds play — its
/// fire sound and its notetrack (reload) sounds found in Ghosts' sound
/// zones with a clip to play (as the match's sound bank takes them).
pub(crate) fn iw6_sound_check(weapons: &asset_game::WeaponRegistry) -> Vec<String> {
    let root = asset_transport::games_root_from_env().ok();
    let Some(install) = asset_audio::ghosts_install(root.as_ref().map(|r| r.0.as_path())) else {
        return vec!["iw6 sounds: Ghosts is not installed".into()];
    };
    let mut bank = asset_audio::SoundCatalog::default();
    let mut lines = Vec::new();
    for zone in asset_audio::ghosts_sound_zones(&install) {
        match asset_audio::ghosts_weapon_sounds(&zone) {
            Ok(c) => {
                lines.push(format!(
                    "iw6 sounds: {}: {} alias lists, {} clips",
                    zone.display(),
                    c.sounds.len(),
                    c.loaded.len()
                ));
                bank.absorb_missing_aliases((*c).clone());
            }
            Err(e) => lines.push(format!("iw6 sounds: {e}")),
        }
    }
    let ns = AssetNamespace::Iw6;
    let playable = |alias: &str| {
        bank.sound_in(ns, alias).is_some_and(|s| {
            s.aliases
                .iter()
                .any(|a| a.loaded.is_bound() || a.streamed.is_some())
        })
    };
    let (mut fire_ok, mut guns, mut nt_ok, mut nt_all) = (0, 0, 0, 0);
    let mut missing = Vec::new();
    for i in 1..=weapons.len() as u32 {
        if weapons.namespace_of(i) != Some(ns) {
            continue;
        }
        let Some(sounds) = weapons.sounds_of(i) else {
            continue;
        };
        guns += 1;
        let fire = sounds.fire_player.as_deref().or(sounds.fire.as_deref());
        if fire.is_some_and(playable) {
            fire_ok += 1;
        } else {
            missing.push(format!(
                "{} fire `{}`",
                weapons.name_of(i),
                fire.unwrap_or("-")
            ));
        }
        // (The match links these to the animations' notetracks.)
        for (_, alias) in &sounds.notetrack_sound_map {
            nt_all += 1;
            nt_ok += usize::from(playable(alias));
        }
    }
    lines.push(format!(
        "iw6 sounds: fire sound playable for {fire_ok} of {guns} weapons; notetrack (reload/handling) sounds {nt_ok} of {nt_all}{}",
        if missing.is_empty() { String::new() } else { format!("; missing: {}", missing.iter().take(8).cloned().collect::<Vec<_>>().join(", ")) }
    ));
    lines
}

/// bo2mp `iw4l check-guns`: every Ghosts gun animation decoded by the
/// engine's IW-family decoder (which refuses a data stream it does not use
/// up exactly), and the decoded rotations' unit length.
pub(crate) fn iw6_anim_check(xanims: &XAnimBuild) -> String {
    let (mut ok, mut bad, mut worst) = (0usize, Vec::new(), 0f32);
    let names: Vec<String> = xanims.names().map(str::to_owned).collect();
    for name in &names {
        match xanims
            .get(AssetNamespace::Iw6, name)
            .map(|c| xmodel_runtime::AnimClip::from_parts(&c.parts))
        {
            Some(Ok(clip)) => {
                ok += 1;
                for t in &clip.tracks {
                    use xmodel_runtime::Rotation as R;
                    let qs: Vec<[f32; 4]> = match &t.rotation {
                        R::HalfKeyed(k) | R::FullKeyed(k) => k.values.clone(),
                        R::HalfConstant(q) | R::FullConstant(q) => vec![*q],
                        R::Default => Vec::new(),
                    };
                    for q in qs {
                        worst =
                            worst.max((q.iter().map(|c| c * c).sum::<f32>().sqrt() - 1.0).abs());
                    }
                }
            }
            Some(Err(e)) => bad.push(format!("{name}: {e:?}")),
            None => {}
        }
    }
    format!(
        "iw6 animations: {ok} decode, {} refused{}; largest |quat|-1 = {worst:.4}",
        bad.len(),
        if bad.is_empty() {
            String::new()
        } else {
            format!(
                " ({})",
                bad.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
            )
        }
    )
}

pub(super) struct Builder<'a> {
    pub(super) cap: &'a Capture,
    pub(super) image: &'a [u8],
    pub(super) strings: &'a [Option<String>],
    pub(super) paks: fastfile_iw6::image::ImagePaks,
    pub(super) catalog: MaterialCatalog,
    pub(super) owner: ZoneOwner,
    /// Capture material index -> catalog material index.
    pub(super) materials: HashMap<usize, usize>,
    /// (capture image index, size cap) -> catalog image slot.
    pub(super) images: HashMap<(usize, u32), Option<usize>>,
    pub(super) image_failures: Vec<String>,
    pub(super) decoded: usize,
}

pub(super) fn le_u16s(b: &[u8]) -> Vec<u16> {
    b.chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect()
}

pub(super) fn le_u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Ghosts' packed unit vector (unsigned 10:10:10 over 1023) as Black Ops
/// II's (signed 10:10:10 over 511).
pub(super) fn iw6_unit_vec_to_t6(packed: u32) -> u32 {
    let mut out = 0u32;
    for k in 0..3 {
        let v = ((packed >> (10 * k)) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0;
        let s = (v.clamp(-1.0, 1.0) * 511.0).round() as i32;
        out |= ((s as u32) & 0x3ff) << (10 * k);
    }
    out
}

/// One Ghosts `GfxPackedVertex` as Black Ops II's: the same position,
/// binormal sign, colour and texture coordinates; the unit vectors
/// repacked.
fn iw6_vertex_to_t6(v: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..24].copy_from_slice(&v[..24]);
    for at in [24usize, 28] {
        let p = u32::from_le_bytes([v[at], v[at + 1], v[at + 2], v[at + 3]]);
        out[at..at + 4].copy_from_slice(&iw6_unit_vec_to_t6(p).to_le_bytes());
    }
    out
}

/// Ghosts' blend streams (eight buckets: vertices with 1..8 influences, a
/// bone word then a bone and weight word per extra influence) as Black Ops
/// II's four: a vertex with more than four keeps its first four (the rest's
/// weight goes to its first bone).
pub(super) fn fold_blend(counts: [u16; 8], words: &[u16]) -> ([i16; 4], Vec<u16>) {
    let mut out_counts = [0i16; 4];
    let mut out = Vec::with_capacity(words.len());
    let mut at = 0usize;
    for (bucket, &count) in counts.iter().enumerate() {
        let per = 1 + 2 * bucket;
        let keep = per.min(7);
        for _ in 0..count {
            let Some(w) = words.get(at..at + per) else {
                break;
            };
            out.extend_from_slice(&w[..keep]);
            at += per;
            out_counts[bucket.min(3)] += 1;
        }
    }
    (out_counts, out)
}

/// Ghosts' model technique set as Black Ops II's draw: by its name (the
/// `m_`/`mc_` model techniques' tokens read as BO2's), drawn double-sided,
/// none of BO2's own special kinds.
pub(super) fn iw6_model_draw(technique_set: &str, sort_key: u8) -> T6Draw {
    let name = technique_set.trim_start_matches(',');
    let name = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let name = match name.strip_prefix("m_") {
        Some(rest) => format!("mc_{rest}"),
        None => name.to_owned(),
    };
    let mut draw = T6Draw::from_technique_set(&name, sort_key, None);
    draw.cull = T6Cull::None;
    draw.water = false;
    draw.flow = false;
    draw.tile = false;
    draw.flag = false;
    draw.tile_blend = false;
    draw.foliage = false;
    draw.objective = false;
    draw.layered = false;
    draw.layers = [0, 0];
    if draw.blend == T6Blend::Sky {
        draw.blend = T6Blend::Opaque;
    }
    // The reflex and holo sights' reticle: a glowing dot, added on unlit.
    if name
        .split('_')
        .any(|t| t.contains("reflex") || t.contains("holo"))
        && draw.blend != T6Blend::Opaque
    {
        draw.unlit = true;
        draw.blend = T6Blend::Add;
        draw.alpha_test = false;
    }
    draw
}

/// Ghosts' BC5 normal maps as Black Ops II's model shader reads them.
///
/// Ghosts stores the tangent-space normal's x and y (z rebuilt), signed
/// (`BC5_SNORM`, or unsigned over 0..1 for `BC5_UNORM`). Black Ops II's lit
/// model draw reads its normal maps as unsigned bytes and takes
/// `xy * 4.0157 - 2.0157`, twice the signed value, as the slope it adds to
/// the vertex normal: so each value here becomes `v / 2` stored unsigned
/// (`128 + v / 2` in bytes). Interpolation is linear, so moving each
/// channel's two endpoints moves every texel of the block; a block whose
/// endpoints meet once halved is made flat (one value, as its texels all
/// but were).
fn bc5_normals_for_bo2(data: &mut [u8], signed: bool) {
    for channel in data.chunks_exact_mut(8) {
        let map = |b: u8| -> u8 {
            if signed {
                (128 + i16::from(b as i8) / 2).clamp(0, 255) as u8
            } else {
                64 + b / 2
            }
        };
        let (a, b) = (channel[0], channel[1]);
        let six_values = if signed { (a as i8) > (b as i8) } else { a > b };
        let (na, nb) = (map(a), map(b));
        channel[0] = na;
        channel[1] = nb;
        if six_values && na == nb {
            channel[2..8].fill(0);
        }
    }
}

/// A GPU image of Ghosts pixels: block formats as they are (BC5 signed
/// normals made unsigned), colour maps read as sRGB.
pub(super) fn gpu_image(px: &fastfile_iw6::image::ImagePixels, colour: bool) -> Result<Image, String> {
    let srgb = |linear: TextureFormat, s: TextureFormat| if colour { s } else { linear };
    let mut data = px.data.clone();
    let format = match px.dxgi_format {
        70..=72 => srgb(TextureFormat::Bc1RgbaUnorm, TextureFormat::Bc1RgbaUnormSrgb),
        73..=75 => srgb(TextureFormat::Bc2RgbaUnorm, TextureFormat::Bc2RgbaUnormSrgb),
        76..=78 => srgb(TextureFormat::Bc3RgbaUnorm, TextureFormat::Bc3RgbaUnormSrgb),
        79 | 80 => TextureFormat::Bc4RUnorm,
        81 => TextureFormat::Bc4RSnorm,
        82..=84 => {
            bc5_normals_for_bo2(&mut data, px.dxgi_format == 84);
            TextureFormat::Bc5RgUnorm
        }
        95 => TextureFormat::Bc6hRgbUfloat,
        96 => TextureFormat::Bc6hRgbFloat,
        97 | 98 | 99 => srgb(TextureFormat::Bc7RgbaUnorm, TextureFormat::Bc7RgbaUnormSrgb),
        27..=29 => srgb(TextureFormat::Rgba8Unorm, TextureFormat::Rgba8UnormSrgb),
        87 | 90 | 91 => srgb(TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb),
        other => return Err(format!("DXGI format {other} is not drawn")),
    };
    let block = matches!(px.dxgi_format, 70..=84 | 94..=99);
    if block && (px.width % 4 != 0 || px.height % 4 != 0) {
        return Err(format!(
            "{}x{} is not whole 4x4 blocks",
            px.width, px.height
        ));
    }
    let mut image = Image::new_uninit(
        Extent3d {
            width: px.width,
            height: px.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = px.levels.max(1);
    image.data = Some(data);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 8,
        ..Default::default()
    });
    Ok(image)
}

impl<'a> Builder<'a> {
    /// bo2mp: a builder over a walked zone (a Ghosts map's: `iw6_map`),
    /// its streamed pictures read from the paks in `zone_dir`.
    pub(super) fn new(
        cap: &'a Capture,
        zone: &'a fastfile_iw6::Zone,
        zone_path: &Path,
        catalog: MaterialCatalog,
    ) -> Self {
        Self {
            cap,
            image: &zone.image,
            strings: &zone.assets.script_strings,
            paks: fastfile_iw6::image::ImagePaks::new(zone_path.parent().unwrap_or(Path::new("."))),
            catalog,
            owner: ZoneOwner::from_zone_path(zone_path),
            materials: HashMap::new(),
            images: HashMap::new(),
            image_failures: Vec::new(),
            decoded: 0,
        }
    }

    /// bo2mp: a world surface's material (a Ghosts map's `GfxWorld`): drawn
    /// as its technique set names it (`l_sm_r0c0..` lit, `b1c1` layers with
    /// the vertex colour as their weights, `t0` alpha tested, `b0` blended),
    /// with its colour map and its layers' (`colorMap1`, `colorMap2`).
    /// `None` when the material is not in the zone.
    pub(super) fn world_material(&mut self, index: usize, max: u32) -> Option<usize> {
        // (World materials keep their own key space apart from models'.)
        const WORLD: usize = 1 << 40;
        if let Some(&i) = self.materials.get(&(index | WORLD)) {
            return Some(i);
        }
        let m: &MaterialRec = self.cap.materials.get(index)?;
        let technique_set = m.techset.map(|t| self.cap.name_of(t).to_owned()).unwrap_or_default();
        let name = technique_set.trim_start_matches(',');
        let mut draw = T6Draw::from_technique_set(name, m.sort_key, None);
        // A replacing base layer (`r0c0`) is opaque whatever the sort key.
        let base = name.split('_').find(|t| {
            let b = t.as_bytes();
            b.len() >= 4 && matches!(b[0], b'r' | b'b' | b't') && b[1] == b'0' && (b[2] == b'c' || b[2] == b'h')
        });
        match base.map(|t| t.as_bytes()[0]) {
            Some(b'r') if draw.blend == T6Blend::Blend => draw.blend = T6Blend::Opaque,
            Some(b't') => {
                draw.alpha_test = true;
                if draw.blend == T6Blend::Blend {
                    draw.blend = T6Blend::Opaque;
                }
            }
            _ => {}
        }
        let mut textures = Vec::new();
        let colour = m
            .textures
            .iter()
            .find(|t| t.name_hash == COLOR_MAP_HASH && t.image.is_some())
            .or_else(|| m.textures.iter().find(|t| t.semantic == 2 && t.image.is_some()));
        let mut wanted: Vec<(u32, u8, u8, usize)> = Vec::new();
        if let Some(t) = colour
            && let Some(img) = t.image
        {
            wanted.push((t.name_hash, t.sampler_state, TS_COLOR_MAP, img.index as usize));
        }
        if draw.layered {
            for (hash, semantic) in super::t6_materials::LAYER_MAPS {
                if let Some(t) = m.textures.iter().find(|t| t.name_hash == hash)
                    && let Some(img) = t.image
                {
                    wanted.push((hash, t.sampler_state, semantic, img.index as usize));
                }
            }
        }
        for (hash, sampler_state, semantic, image) in wanted {
            // Layer pictures are colour pictures (sRGB) too.
            let read_as = if semantic == TS_NORMAL_MAP || semantic == TS_SPECULAR_MAP { semantic } else { TS_COLOR_MAP };
            if let Some(slot) = self.image(image, max, read_as) {
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
        let index_in_catalog = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(format!("iw6:{}", m.name)),
            namespace: AssetNamespace::T6,
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
            constants: Vec::new(),
            zone: self.owner,
            t6_draw: Some(draw),
        });
        self.materials.insert(index | WORLD, index_in_catalog);
        Some(index_in_catalog)
    }

    /// bo2mp: a Ghosts map's sky (`sky_<map>_hdr`: a cube, its six faces
    /// one after another in one pak range, BC6H linear light) as a cube
    /// image under a material whose draw is Black Ops II's sky
    /// (`T6Blend::Sky`: the sky pass draws it behind the world).
    pub(super) fn sky_material(&mut self, image_index: usize, max: u32) -> Result<usize, String> {
        let rec: &ImageRec = self.cap.images.get(image_index).ok_or("sky image not in the zone")?;
        let name = rec.name.clone();
        let px = fastfile_iw6::image::image_pixels(rec, self.image, &mut self.paks, max)?;
        let m = link_sky(&mut self.catalog, &format!("iw6:sky:{name}"), &px, self.owner).map_err(|e| format!("{name}: {e}"))?;
        self.decoded += 1;
        Ok(m)
    }

    fn string(&self, index: u32) -> String {
        self.strings
            .get(index as usize)
            .cloned()
            .flatten()
            .unwrap_or_default()
    }

    fn name(&self, id: AssetId) -> String {
        self.cap.name_of(id).trim_start_matches(',').to_owned()
    }

    /// A model's first LOD in Black Ops II's captured form, and the capture
    /// materials its surfaces use.
    pub(super) fn model_ref(&self, m: &XModelRec) -> Option<(asset_t6::XModelRef, Vec<usize>)> {
        let lod = m.lods.first()?;
        let surfs = self.cap.model_surfs.get(lod.surfs?.index as usize)?;
        let material_key = |k: usize| asset_t6::AssetKey {
            ty: fastfile_t6::AssetType::Material,
            index: k,
        };
        let mut keys = Vec::new();
        let mut lod0 = Vec::with_capacity(surfs.surfs.len());
        for (i, s) in surfs.surfs.iter().enumerate() {
            let verts: Vec<u8> = s
                .verts
                .map_or(&[][..], |sp| sp.get(self.image))
                .chunks_exact(32)
                .flat_map(iw6_vertex_to_t6)
                .collect();
            let indices = le_u16s(s.tris.map_or(&[][..], |sp| sp.get(self.image)));
            let blend_words = le_u16s(s.blend_verts.map_or(&[][..], |sp| sp.get(self.image)));
            let (blend_counts, blend) = fold_blend(s.blend_vert_counts, &blend_words);
            let material = m
                .materials
                .get(usize::from(lod.surf_index) + i)
                .copied()
                .flatten()
                .map(|id| id.index as usize);
            keys.extend(material);
            let mut part_bits = [0u32; 6];
            part_bits.copy_from_slice(&s.part_bits[..6]);
            lod0.push(asset_t6::XSurfaceRef {
                verts,
                vert_count: s.vert_count,
                indices,
                vert_lists: s.rigid.clone(),
                flags: 0,
                material: material.map(material_key),
                blend_counts,
                blend,
                part_bits,
            });
        }
        let (mid, half) = m.bounds;
        let bone_names: Vec<String> = m.bone_names.iter().map(|&s| self.string(s)).collect();
        Some((
            asset_t6::XModelRef {
                lighting_origin_offset: [0.0; 3],
                lighting_origin_range: 0.0,
                name: m.name.clone(),
                numsurfs: m.numsurfs,
                materials: m
                    .materials
                    .iter()
                    .map(|k| k.map(|id| material_key(id.index as usize)))
                    .collect(),
                num_bones: m.num_bones,
                num_lods: u16::from(m.num_lods),
                mins: std::array::from_fn(|k| mid[k] - half[k]),
                maxs: std::array::from_fn(|k| mid[k] + half[k]),
                radius: m.radius,
                lod0,
                bone_names,
                num_root_bones: m.num_root_bones,
                parent_list: m.parent_list.clone(),
                quats: m.tag_angles.clone(),
                trans: m.tag_positions.clone(),
                base_mat: m
                    .base_mat
                    .iter()
                    .map(|b| ([b[0], b[1], b[2], b[3]], [b[4], b[5], b[6]]))
                    .collect(),
                part_classification: m.part_classification.clone(),
                bone_info: vec![None; usize::from(m.num_bones)],
                lods: m
                    .lods
                    .iter()
                    .map(|l| (l.dist, l.numsurfs, l.surf_index))
                    .collect(),
                contents: m.contents as u32,
                coll_lod: i16::from(m.coll_lod),
                coll_surfs: Vec::new(),
                phys_preset: None,
            },
            keys,
        ))
    }

    /// One picture into the catalog (`None` when its pixels are not here).
    fn image(&mut self, index: usize, max: u32, semantic: u8) -> Option<usize> {
        if let Some(slot) = self.images.get(&(index, max)) {
            return *slot;
        }
        let rec: &ImageRec = self.cap.images.get(index)?;
        let slot = match fastfile_iw6::image::image_pixels(rec, self.image, &mut self.paks, max)
            .and_then(|px| gpu_image(&px, semantic == TS_COLOR_MAP).map(|img| (px, img)))
        {
            Ok((px, gpu)) => {
                self.decoded += 1;
                Some(self.catalog.link_image(AuthoredImage {
                    namespace: AssetNamespace::Iw6,
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
        self.images.insert((index, max), slot);
        slot
    }

    /// One material into the catalog, with its colour, normal and specular
    /// maps.
    pub(super) fn material(&mut self, index: usize, max: u32) -> Option<usize> {
        if let Some(&i) = self.materials.get(&index) {
            return Some(i);
        }
        let m: &MaterialRec = self.cap.materials.get(index)?;
        let technique_set = m
            .techset
            .map(|t| self.cap.name_of(t).to_owned())
            .unwrap_or_default();
        let draw = iw6_model_draw(&technique_set, m.sort_key);
        let mut textures = Vec::new();
        let colour = m
            .textures
            .iter()
            .find(|t| t.name_hash == COLOR_MAP_HASH && t.image.is_some())
            .or_else(|| {
                m.textures
                    .iter()
                    .find(|t| t.semantic == 2 && t.image.is_some())
            });
        let mut slots: Vec<(&fastfile_iw6::walk::MaterialTexture, u8)> = Vec::new();
        if let Some(t) = colour {
            slots.push((t, TS_COLOR_MAP));
        }
        if !draw.unlit {
            for (hash, semantic) in [
                (NORMAL_MAP_HASH, TS_NORMAL_MAP),
                (SPECULAR_MAP_HASH, TS_SPECULAR_MAP),
            ] {
                if let Some(t) = m
                    .textures
                    .iter()
                    .find(|t| t.name_hash == hash && t.image.is_some())
                {
                    slots.push((t, semantic));
                }
            }
        }
        let wanted: Vec<(u32, u8, u8, usize)> = slots
            .iter()
            .filter_map(|(t, s)| Some((t.name_hash, t.sampler_state, *s, t.image?.index as usize)))
            .collect();
        for (hash, sampler_state, semantic, image) in wanted {
            if let Some(slot) = self.image(image, max, semantic) {
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
        let index_in_catalog = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(m.name.clone()),
            namespace: AssetNamespace::Iw6,
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
        self.materials.insert(index, index_in_catalog);
        Some(index_in_catalog)
    }

    /// An animation in Black Ops II's captured form.
    fn anim_ref(&self, a: &XAnimRec) -> Option<asset_t6::XAnimRef> {
        let bytes = |s: Option<fastfile_iw6::walk::Span>| s.map_or(&[][..], |s| s.get(self.image));
        let indices = if a.numframes >= 256 {
            le_u16s(bytes(a.indices))
        } else {
            bytes(a.indices).iter().map(|&b| u16::from(b)).collect()
        };
        Some(asset_t6::XAnimRef {
                delta_quat: Vec::new(),
            name: a.name.trim_start_matches(',').to_owned(),
            numframes: a.numframes,
            framerate: a.framerate,
            looping: a.flags & 1 != 0,
            delta: false,
            delta_trans: Vec::new(),
            bone_count: a.bone_count,
            names: a.names.iter().map(|&s| self.string(s)).collect(),
            notifies: a.notify.iter().map(|&(s, t)| (self.string(s), t)).collect(),
            data_byte: bytes(a.data_byte).to_vec(),
            data_short: le_u16s(bytes(a.data_short)),
            data_int: le_u32s(bytes(a.data_int)),
            random_data_byte: bytes(a.random_data_byte).to_vec(),
            random_data_short: le_u16s(bytes(a.random_data_short)),
            random_data_int: le_u32s(bytes(a.random_data_int)),
            indices,
        })
    }

    fn weapon_view(&self, w: &fastfile_iw6::walk::WeaponRec) -> Iw6WeaponView {
        let name_of = |id: &Option<AssetId>| id.map(|id| self.name(id)).filter(|n| !n.is_empty());
        let anims = if w.anims.iter().any(Option::is_some) {
            &w.anims
        } else {
            &w.anims_right
        };
        let notetrack_sounds = w
            .notetrack_sound_keys
            .iter()
            .zip(&w.notetrack_sound_values)
            .map(|(&k, &v)| (self.string(k), self.string(v)))
            .filter(|(k, v)| !k.is_empty() && !v.is_empty())
            .collect();
        Iw6WeaponView {
            name: w.name.clone(),
            display_name: w.display_name.clone(),
            complete: w.complete.get(self.image).to_vec(),
            def: w
                .def
                .map(|s| s.get(self.image).to_vec())
                .unwrap_or_default(),
            gun_model: w.gun_models.first().and_then(name_of),
            world_model: w.world_models.first().and_then(name_of),
            xanims: anims
                .iter()
                .map(|a| name_of(a).unwrap_or_default().to_ascii_lowercase())
                .collect(),
            hide_tags: w
                .hide_tags
                .iter()
                .map(|&s| self.string(s))
                .filter(|s| !s.is_empty())
                .collect(),
            alt_weapon_name: w.alt_weapon_name.clone(),
            sounds: w.sounds.clone(),
            fx: w.fx.iter().map(|(o, id)| (*o, self.name(*id))).collect(),
            models: w
                .models
                .iter()
                .map(|(o, id)| (*o, self.name(*id)))
                .collect(),
            materials: w
                .materials
                .iter()
                .map(|(o, id)| (*o, self.name(*id)))
                .collect(),
            notetrack_sounds,
            location_damage: w.location_damage.map(|s| {
                s.get(self.image)
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect()
            }),
            tracer: w.tracer.map(|id| self.name(id)).filter(|n| !n.is_empty()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normals_halve_into_bo2_range() {
        // Signed +127 / -127 endpoints (6-value mode) -> 191 / 65.
        let mut block = [127u8, 0x81, 1, 2, 3, 4, 5, 6, 127, 0x81, 0, 0, 0, 0, 0, 0];
        bc5_normals_for_bo2(&mut block, true);
        assert_eq!((block[0], block[1], block[2]), (191, 65, 1));
        // Endpoints that meet: one value.
        let mut flat = [3u8, 2, 9, 9, 9, 9, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0];
        bc5_normals_for_bo2(&mut flat, true);
        assert_eq!(&flat[0..8], &[129, 129, 0, 0, 0, 0, 0, 0]);
        // Unsigned 255 / 0 -> 191 / 64.
        let mut u = [255u8, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0];
        bc5_normals_for_bo2(&mut u, false);
        assert_eq!((u[0], u[1]), (191, 64));
    }

    #[test]
    fn unit_vectors_repack() {
        // +x: 1023 unsigned -> 511 signed; -1 (0) -> -511 (513).
        assert_eq!(iw6_unit_vec_to_t6(1023) & 0x3ff, 511);
        assert_eq!(iw6_unit_vec_to_t6(0) & 0x3ff, 513);
        let mid = iw6_unit_vec_to_t6(512 << 20) >> 20;
        assert!(mid <= 1 || mid >= 1023);
    }

    #[test]
    fn blend_folds_into_four_buckets() {
        // One vertex with five influences: 9 words, keeps 7.
        let mut counts = [0u16; 8];
        counts[4] = 1;
        let words: Vec<u16> = (0..9).collect();
        let (c, w) = fold_blend(counts, &words);
        assert_eq!(c, [0, 0, 0, 1]);
        assert_eq!(w, vec![0, 1, 2, 3, 4, 5, 6]);
    }
}

/// bo2mp: a sky cube (Ghosts' and Infinite Warfare's `sky_<map>`: six
/// faces one after another, each with its levels, BC6H linear light) as a
/// cube image under a material whose draw is Black Ops II's sky
/// (`T6Blend::Sky`: the sky pass draws the first such material's colour
/// map behind the world). The catalog material's index.
pub(super) fn link_sky(
    catalog: &mut MaterialCatalog,
    name: &str,
    px: &fastfile_iw6::image::ImagePixels,
    owner: ZoneOwner,
) -> Result<usize, String> {
    use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};
    let mut face = 0usize;
    let (mut w, mut h) = (px.width, px.height);
    for _ in 0..px.levels.max(1) {
        face += fastfile_iw6::image::level_bytes(px.dxgi_format, w, h).ok_or("sky format not read")?;
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
    if px.width != px.height || px.data.len() != 6 * face {
        return Err(format!("{}x{}, {} bytes ({} wanted for six faces)", px.width, px.height, px.data.len(), 6 * face));
    }
    let format = match px.dxgi_format {
        95 => TextureFormat::Bc6hRgbUfloat,
        96 => TextureFormat::Bc6hRgbFloat,
        70..=72 => TextureFormat::Bc1RgbaUnormSrgb,
        76..=78 => TextureFormat::Bc3RgbaUnormSrgb,
        97..=99 => TextureFormat::Bc7RgbaUnormSrgb,
        27..=29 => TextureFormat::Rgba8UnormSrgb,
        other => return Err(format!("DXGI format {other} is not drawn as a sky")),
    };
    let mut image = Image::new_uninit(
        Extent3d { width: px.width, height: px.height, depth_or_array_layers: 6 },
        TextureDimension::D2,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = px.levels.max(1);
    image.data = Some(px.data.clone());
    image.texture_view_descriptor = Some(TextureViewDescriptor { dimension: Some(TextureViewDimension::Cube), ..Default::default() });
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    let slot = catalog.link_image(AuthoredImage {
        namespace: AssetNamespace::T6,
        name: AssetRef::Real(name.to_owned()),
        map_type: 5,
        semantic: TS_COLOR_MAP,
        category: 0,
        use_srgb_reads: false,
        width: px.width as u16,
        height: px.height as u16,
        depth: 1,
        level_count: px.levels as u8,
        format: 0,
        payload: Arc::new(Vec::new()),
        decoded: Some(Arc::new(image)),
        common_owned: false,
        decoded_variant: None,
        decoded_by: None,
        pending_decode: None,
    });
    Ok(catalog.link_material(AuthoredMaterial {
        name: AssetRef::Real(name.to_owned()),
        namespace: AssetNamespace::T6,
        technique_set: AssetRef::default(),
        technique_set_edge: AssetEdge::Absent,
        draw_surf: 0,
        sort_key: 0,
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
        textures: vec![MaterialTextureBinding {
            name_hash: COLOR_MAP_HASH,
            name_start: 0,
            name_end: 0,
            sampler_state: 0x13,
            semantic: TS_COLOR_MAP,
            image: Some(slot),
        }],
        constants: Vec::new(),
        zone: owner,
        t6_draw: Some(T6Draw::from_technique_set("wc_sky", 0, None)),
    }))
}
