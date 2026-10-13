//! bo2mp: Call of Duty 4 (IW3) maps.
//!
//! A CoD4 map zone (`IWffu100`, version 5) is read with the `iw3` crate
//! (ported from COD4-Rewrite): its GfxWorld becomes the world draw, its
//! clipMap_t the collision, its MapEnts the entities and spawns. Every world
//! surface's colour map is read from CoD4's own IWDs (`main/iw_*.iwd`) and
//! drawn through the renderer's fallback pass, as Black Ops II's surfaces
//! are; Black Ops II's rules then add everything else
//! (`attach_t6_rules`). No ZoneGame of its own: the map walk sends a CoD4
//! zone here by its version.
//!
//! World at War (`t4`) maps share everything after the zone read: the `t4`
//! crate converts a WaW zone to an `iw3::zone::Zone` (see `super::waw`),
//! and [`build_legacy_world`] turns either into the match's world.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use asset_core::{AssetEdge, AssetNamespace, AssetRef, T6Draw, ZoneOwner};
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP, TS_NORMAL_MAP, TS_SPECULAR_MAP};
use asset_transport::progress::{LoadProgress, StageId};
use asset_world::WorldDrawPolicy;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::LoadedWorld;
use crate::lane_capability::PreparedCapability;
use crate::session_load::PreparedWorld;

/// The techniques whose draw state a surface takes, first present wins: lit
/// with sun shadow, lit with sun, lit, emissive, unlit.
const TECHNIQUES: [usize; 5] = [9, 8, 7, 5, 4];

/// bo2mp: a colour map with its whole mip chain, so the fallback pass's
/// trilinear, anisotropic sampler has smaller levels to read at a distance
/// (one level shimmered and aliased across every far floor and wall).
fn rgba_image(w: u32, h: u32, rgba: Vec<u8>) -> Image {
    let (data, levels) = mip_chain(w, h, rgba);
    let mut image = Image::new_uninit(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.mip_level_count = levels;
    image.data = Some(data);
    image
}

/// The levels of an sRGB RGBA8 picture, largest first, concatenated: each a
/// 2x2 box of the one above, averaged in linear light. Alpha keeps the top
/// level's coverage at the 0.5 test the alpha-tested shaders make (each
/// level's alpha scaled until as much of it passes), so leaves and fences
/// do not thin away with distance.
fn mip_chain(w: u32, h: u32, top: Vec<u8>) -> (Vec<u8>, u32) {
    let levels = 32 - w.max(h).max(1).leading_zeros();
    let to_lin: Vec<f32> = (0..256).map(|i| (i as f32 / 255.0).powf(2.2)).collect();
    // Linear back to 8-bit sRGB through a 4096-step table.
    let to_srgb: Vec<u8> =
        (0..4096).map(|i| ((i as f32 / 4095.0).powf(1.0 / 2.2) * 255.0).round().clamp(0.0, 255.0) as u8).collect();
    let coverage = |px: &[u8], scale: f32| {
        let n = px.len() / 4;
        let pass = px.chunks_exact(4).filter(|p| f32::from(p[3]) * scale >= 127.5).count();
        pass as f32 / n.max(1) as f32
    };
    let want = coverage(&top, 1.0);
    let cut = top.chunks_exact(4).any(|p| p[3] < 255);
    let mut out = top.clone();
    let (mut pw, mut ph, mut prev) = (w as usize, h as usize, top);
    for _ in 1..levels {
        let (nw, nh) = ((pw / 2).max(1), (ph / 2).max(1));
        let mut next = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                let mut acc = [0.0f32; 4];
                let mut n = 0.0;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((x * 2 + dx).min(pw - 1), (y * 2 + dy).min(ph - 1));
                    let p = &prev[(sy * pw + sx) * 4..][..4];
                    for c in 0..3 {
                        acc[c] += to_lin[usize::from(p[c])];
                    }
                    acc[3] += f32::from(p[3]);
                    n += 1.0;
                }
                let o = (y * nw + x) * 4;
                for c in 0..3 {
                    next[o + c] = to_srgb[((acc[c] / n) * 4095.0).round().clamp(0.0, 4095.0) as usize];
                }
                next[o + 3] = (acc[3] / n).round().clamp(0.0, 255.0) as u8;
            }
        }
        if cut && want > 0.0 {
            // The alpha scale whose coverage matches the top level's.
            let (mut lo, mut hi) = (0.0f32, 4.0f32);
            for _ in 0..12 {
                let mid = (lo + hi) * 0.5;
                if coverage(&next, mid) < want { lo = mid } else { hi = mid }
            }
            for a in next.chunks_exact_mut(4) {
                a[3] = (f32::from(a[3]) * hi).round().clamp(0.0, 255.0) as u8;
            }
        }
        out.extend_from_slice(&next);
        (pw, ph, prev) = (nw, nh, next);
    }
    (out, levels)
}

/// Links every surface material the world uses into `catalog` with its
/// colour map (from the game's IWDs); world surface index -> catalog index.
fn link_world_materials(
    tag: &str,
    zone: &iw3::zone::Zone,
    world: &iw3::zone::GfxWorld,
    main: Option<&Path>,
    catalog: &mut MaterialCatalog,
    owner: &str,
    report: &mut Vec<String>,
    look: Option<&super::bo2_look::Look>,
) -> Vec<Option<usize>> {
    let owner = ZoneOwner::intern(owner);
    // bo2mp: Black Ops II pictures linked for swapped surfaces (colour,
    // normal, specular slots), by World at War colour image.
    let mut swapped: HashMap<&'static str, [Option<usize>; 3]> = HashMap::new();
    let mut swap_count = 0usize;
    let mut by_material: HashMap<usize, Option<usize>> = HashMap::new();
    let mut images: HashMap<String, Option<usize>> = HashMap::new();
    let (mut decoded, mut grey, mut missing) = (0usize, 0usize, Vec::new());
    let mut out = Vec::with_capacity(world.surfaces.len());
    for surf in &world.surfaces {
        let Some(mid) = surf.material else {
            out.push(None);
            continue;
        };
        if let Some(found) = by_material.get(&mid) {
            out.push(*found);
            continue;
        }
        let Some(material) = zone.material(mid) else {
            by_material.insert(mid, None);
            out.push(None);
            continue;
        };
        let technique_set = material
            .technique_set
            .and_then(|t| zone.technique_set(t))
            .map_or("", |t| t.name.as_str());
        let state = TECHNIQUES.iter().find_map(|&t| material.state_bits_for(t));
        let draw = T6Draw::from_technique_set(technique_set, material.sort_key, state);
        let mut textures = Vec::new();
        let colour = material
            .textures
            .iter()
            .find(|t| t.semantic == iw3::zone::TextureSemantic::Color && t.image.is_some());
        let swap = colour
            .and_then(|t| t.image)
            .and_then(|i| zone.image(i))
            .and_then(|image| look.and_then(|l| l.swaps.get_key_value(image.name.as_str())));
        if let Some((&key, swap)) = swap
            && let Some(texture) = colour
        {
            let slots = *swapped.entry(key).or_insert_with(|| {
                let link = |catalog: &mut MaterialCatalog, img: &Arc<Image>, part: &str, semantic: u8, srgb: bool| {
                    let size = img.texture_descriptor.size;
                    catalog.link_image(AuthoredImage {
                        namespace: AssetNamespace::T6,
                        name: AssetRef::Real(format!("bo2look:{}:{part}", swap.material)),
                        map_type: 0,
                        semantic,
                        category: 0,
                        use_srgb_reads: srgb,
                        width: size.width as u16,
                        height: size.height as u16,
                        depth: 1,
                        level_count: img.texture_descriptor.mip_level_count as u8,
                        format: 0,
                        payload: Arc::new(Vec::new()),
                        decoded: Some(Arc::clone(img)),
                        common_owned: false,
                        decoded_variant: None,
                        decoded_by: None,
                        pending_decode: None,
                    })
                };
                [
                    Some(link(catalog, &swap.colour, "c", TS_COLOR_MAP, true)),
                    swap.normal.as_ref().map(|i| link(catalog, i, "n", TS_NORMAL_MAP, false)),
                    swap.specular.as_ref().map(|i| link(catalog, i, "s", TS_SPECULAR_MAP, true)),
                ]
            });
            swap_count += 1;
            for (slot, semantic) in slots.into_iter().zip([TS_COLOR_MAP, TS_NORMAL_MAP, TS_SPECULAR_MAP]) {
                if slot.is_some() {
                    textures.push(MaterialTextureBinding {
                        name_hash: texture.name_hash,
                        name_start: 0,
                        name_end: 0,
                        sampler_state: texture.sampler_state,
                        semantic,
                        image: slot,
                    });
                }
            }
        } else if let Some(texture) = colour
            && let Some(image) = texture.image.and_then(|i| zone.image(i))
        {
            let slot = *images.entry(image.name.clone()).or_insert_with(|| {
                let pixels = main
                    .and_then(|main| asset_material::decode_ui_image_from_main(main, &image.name).ok().flatten());
                let (w, h, rgba) = match pixels {
                    Some(p) => {
                        decoded += 1;
                        grey += usize::from(asset_world::world_iw3::is_greyscale_rgba(&p.2));
                        p
                    }
                    None => {
                        missing.push(image.name.clone());
                        // Flat grey where the picture is not found.
                        (1, 1, vec![128, 128, 128, 255])
                    }
                };
                Some(catalog.link_image(AuthoredImage {
                    namespace: AssetNamespace::T6,
                    name: AssetRef::Real(format!("{tag}:{}", image.name)),
                    map_type: 0,
                    semantic: TS_COLOR_MAP,
                    category: 0,
                    use_srgb_reads: true,
                    width: w as u16,
                    height: h as u16,
                    depth: 1,
                    level_count: 1,
                    format: 0,
                    payload: Arc::new(Vec::new()),
                    decoded: Some(Arc::new(rgba_image(w, h, rgba))),
                    common_owned: false,
                    decoded_variant: None,
                    decoded_by: None,
                    pending_decode: None,
                }))
            });
            textures.push(MaterialTextureBinding {
                name_hash: texture.name_hash,
                name_start: 0,
                name_end: 0,
                sampler_state: texture.sampler_state,
                semantic: TS_COLOR_MAP,
                image: slot,
            });
        }
        let index = catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(format!("{tag}:{}", material.name)),
            namespace: AssetNamespace::T6,
            technique_set: AssetRef::default(),
            technique_set_edge: AssetEdge::Absent,
            draw_surf: 0,
            sort_key: material.sort_key,
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
            zone: owner,
            t6_draw: Some(draw),
        });
        by_material.insert(mid, Some(index));
        out.push(Some(index));
    }
    report.push(format!(
        "{tag} materials: {} linked, {decoded} colour maps decoded from the IWDs ({grey} of them grey), {} not found{}",
        by_material.values().filter(|m| m.is_some()).count(),
        missing.len(),
        if missing.is_empty() {
            String::new()
        } else {
            format!(" (first: {})", missing.iter().take(5).cloned().collect::<Vec<_>>().join(", "))
        } + &if swap_count > 0 {
            format!("; {swap_count} drawn with Black Ops II materials")
        } else {
            String::new()
        }
    ));
    out
}

/// The spawn points in the map's entities (`mp_*_spawn*`), and the first
/// one's place for the start-up scripts.
fn spawns_of(entities: &[iw3::ents::Entity]) -> Vec<asset_world::SpawnPoint> {
    entities
        .iter()
        .filter(|e| {
            let c = e.classname();
            c.starts_with("mp_") && c.contains("spawn") || c == "info_player_start"
        })
        .filter_map(|e| {
            Some(asset_world::SpawnPoint {
                classname: e.classname().to_owned(),
                origin: e.origin()?,
                angles: e.angles(),
                script_linkto: e.get("script_linkto").unwrap_or_default().to_owned(),
                script_destructable_area: e.get("script_destructable_area").unwrap_or_default().to_owned(),
            })
        })
        .collect()
}

/// Which legacy game a map comes from, for the report and the gaps.
pub(crate) struct Legacy {
    /// Asset and report prefix: `iw3`, `t4`.
    pub tag: &'static str,
    /// For messages: `CoD4`, `World at War`.
    pub title: &'static str,
    /// The loading screen's stage for the material links.
    pub stage_materials: &'static str,
    pub addr_no_world: &'static str,
    pub addr_world_mesh: &'static str,
}

pub(crate) const COD4: Legacy = Legacy {
    tag: "iw3",
    title: "CoD4",
    stage_materials: "cod4 materials",
    addr_no_world: "assets::lane::iw3::load_world/no_world",
    addr_world_mesh: "assets::lane::iw3::load_world/world_mesh",
};

/// Loads a Call of Duty 4 map zone into the match's world.
pub(crate) fn load_iw3_world(
    path: &Path,
    progress: &LoadProgress,
    material_seed: MaterialCatalog,
) -> LoadedWorld {
    let report = vec![format!("game: IW3 / Call of Duty 4 ({})", path.display())];
    let stage = progress.begin_scoped(StageId::MapAssets, "cod4 zone", None);
    let parsed = iw3::fastfile::load(path).and_then(|bytes| {
        iw3::zone::Zone::parse(&bytes, iw3::zone::ParseOptions::default())
    });
    let zone = match parsed {
        Ok(zone) => {
            stage.done();
            zone
        }
        Err(error) => {
            stage.fail();
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("CoD4 zone: {error:#}"),
                Some("assets::lane::iw3::load_world/zone"),
            );
        }
    };
    let loaded = build_legacy_world(&COD4, path, &zone, report, progress, material_seed);
    diag::info!(
        World,
        "bo2mp iw3: {} loaded: {}",
        path.display(),
        loaded.report.iter().filter(|l| l.starts_with("iw3 ") || l.starts_with("world mesh")).cloned().collect::<Vec<_>>().join("; ")
    );
    loaded
}

/// bo2mp: a CoD4 / World at War map's createart fog: distance fog
/// (CoD4's `setExpFog`) or height fog (WaW's `setVolFog`, base height and
/// half height as Black Ops II's own fog).
#[derive(Clone, Copy, Debug, PartialEq)]
struct LegacyFog {
    start: f32,
    half: f32,
    opacity: f32,
    colour: [f32; 3],
    sun: Option<asset_world::SunFog>,
    base_height: f32,
    half_height: f32,
}

/// The fog a createart script sets: `setVolFog(start, halfway,
/// halfwayHeight, baseHeight, r, g, b, transition)` (WaW), else
/// `setExpFog(...)` (CoD4: no height, so a base far above every map and a
/// vast half height).
fn legacy_fog(script: &str) -> Option<LegacyFog> {
    let lower = script.to_ascii_lowercase();
    // (The usage comment after the call names it too: the first call with
    // numbers wins.)
    let mut from = 0;
    while let Some(at) = lower[from..].find("setvolfog(").map(|i| from + i) {
        from = at + 10;
        let close = lower[from..].find(')').map(|i| from + i)?;
        let args: Vec<f32> = script[from..close].split(',').filter_map(|a| a.trim().parse::<f32>().ok()).collect();
        if args.len() >= 7 && args[1] > 0.0 {
            return Some(LegacyFog {
                start: args[0].max(0.0),
                half: args[1],
                opacity: 1.0,
                colour: [args[4], args[5], args[6]].map(|c| c.clamp(0.0, 1.0)),
                sun: None,
                base_height: args[3],
                half_height: args[2].max(1.0),
            });
        }
    }
    let f = asset_world::parse_set_exp_fog(script)?;
    Some(LegacyFog {
        start: f.start_dist,
        half: f.halfway_dist,
        opacity: f.max_opacity,
        colour: f.color_rgb,
        sun: f.sun,
        base_height: 1.0e7,
        half_height: 1.0e9,
    })
}

/// bo2mp: the fallback pass's light table for a CoD4 / World at War world:
/// no lights of its own (slots 0..29 empty), and its createart fog
/// (`setExpFog` in `maps/createart/<map>_art.gsc`) in the fog slots (31:
/// start, half distance, opacity, on; colour, base height; sun fog colour
/// and opacity; its direction and half height; 30: the sun fog blend), as
/// `lane::t6` lays them out. The IW fog is distance fog only: a base far
/// above every map and a vast half height flatten the shader's height term
/// to 1, leaving 1 - 2^-((d - start) / half). Its colours are gamma values,
/// squared for the linear fallback pass.
fn legacy_fog_table(fog: &LegacyFog) -> Vec<[f32; 16]> {
    let lin = |c: [f32; 3]| c.map(|x| x * x);
    let colour = lin(fog.colour);
    let (sun_colour, sun_dir, blend) = match fog.sun {
        Some(sun) => {
            let n = (sun.sun_dir.iter().map(|x| x * x).sum::<f32>()).sqrt().max(1e-6);
            let (ci, co) = (sun.begin_angle_deg.to_radians().cos(), sun.end_angle_deg.to_radians().cos());
            let scale = if (ci - co).abs() > 1e-4 { 1.0 / (ci - co) } else { 0.0 };
            (lin(sun.color_rgb), sun.sun_dir.map(|x| x / n), [-co * scale, scale])
        }
        None => (colour, [0.0, 0.0, 1.0], [0.0, 0.0]),
    };
    let mut table = vec![[0.0f32; 16]; 32];
    table[31] = [
        fog.start,
        fog.half,
        fog.opacity,
        1.0,
        colour[0],
        colour[1],
        colour[2],
        fog.base_height,
        sun_colour[0],
        sun_colour[1],
        sun_colour[2],
        fog.opacity,
        sun_dir[0],
        sun_dir[1],
        sun_dir[2],
        fog.half_height,
    ];
    table[30][0] = blend[0];
    table[30][1] = blend[1];
    table
}

/// A legacy (CoD4-format) map zone, already read, into the match's world:
/// entities and spawns, collision, materials and the world draw.
pub(crate) fn build_legacy_world(
    game: &Legacy,
    path: &Path,
    zone: &iw3::zone::Zone,
    mut report: Vec<String>,
    progress: &LoadProgress,
    material_seed: MaterialCatalog,
) -> LoadedWorld {
    let tag = game.tag;
    let title = game.title;
    super::bo2_look::set_pending_grade(None);
    report.push(format!(
        "{tag} zone: {} of {} assets{}",
        zone.stats.top_level_parsed,
        zone.stats.top_level_total,
        zone.stats.stopped_at.as_ref().map_or_else(String::new, |s| format!(", stopped at {s}"))
    ));
    let map = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());

    // Entities and spawns.
    let entity_string = zone.map_ents().map(|m| m.entity_string.clone()).unwrap_or_default();
    let entities = iw3::ents::parse(&entity_string);
    let spawns = spawns_of(&entities);
    report.push(format!("{tag} entities: {}, spawn points {}", entities.len(), spawns.len()));
    let mut scripts = crate::ScriptSources::default();
    scripts.set_entities(entity_string);

    // Collision.
    let collision = match zone.clip_map().map(asset_world::world_iw3::build_iw3_clip_collision) {
        Some(Ok(clip)) => {
            let origins: Vec<[f32; 3]> = spawns.iter().map(|s| s.origin).collect();
            let census = asset_world::world_iw3::spawn_ground_census(&clip, &origins);
            report.push(format!(
                "{tag} clip: {} brushes ({} the world's), {} mesh tris in {} trees, {} models; {census}",
                clip.brushes.len(),
                clip.leaves.first().map_or(0, |l| l.num_brushes),
                clip.mesh.tri_indices.len() / 3,
                clip.mesh.aabb_roots.len(),
                clip.cmodels.len()
            ));
            if census.spawns > 0 && census.on_ground * 2 < census.spawns {
                diag::warn!(
                    World,
                    "bo2mp {tag}: {map}: most spawn points have no floor of their own in the collision ({census})"
                );
            }
            Some(clip)
        }
        Some(Err(error)) => {
            report.push(format!("{tag} clip gap: {error}"));
            None
        }
        None => {
            report.push(format!("{tag} clip gap: zone has no clipMap"));
            None
        }
    };

    let Some(world) = zone.gfx_world() else {
        let reason = match &zone.stats.stopped_at {
            Some(stop) => format!("{title} zone has no GfxWorld (reading stopped at {stop})"),
            None => format!("{title} zone has no GfxWorld"),
        };
        let mut loaded = LoadedWorld::with_gap(
            WorldDrawPolicy::t5(),
            PreparedCapability::PreparedWorld,
            reason,
            Some(game.addr_no_world),
        );
        loaded.scripts = scripts;
        loaded.collision = collision;
        loaded.spawns = spawns;
        loaded.report.extend(report);
        return loaded;
    };

    // Materials and their colour maps, from the game's own IWDs.
    let stage = progress.begin_scoped(StageId::MapAssets, game.stage_materials, None);
    let main = asset_transport::game_root_for_zone(path).ok().map(|root| root.join("main"));
    let mut materials = material_seed;
    materials.set_capture_ns(AssetNamespace::T6);
    // bo2mp: a World at War map in a Black Ops II map's look (`bo2_look`).
    let look = (tag == "t4")
        .then(|| super::bo2_look::recipe(&map))
        .flatten()
        .and_then(|recipe| super::bo2_look::load(recipe, &mut report));
    let surface_materials =
        link_world_materials(tag, zone, world, main.as_deref(), &mut materials, &map, &mut report, look.as_ref());
    stage.done();

    // The map's film vision (`vision/<map>.vision` in the game's IWDs): its
    // colour grade, drawn under Black Ops II's rules as the colour table.
    let vision_name = format!("vision/{}.vision", map.to_ascii_lowercase());
    let film_vision = match main.as_deref().and_then(|m| asset_transport::read_main_iwd_file(m, &vision_name)) {
        Some(bytes) => match asset_world::parse_film_vision_rawfile(&vision_name, &bytes, false) {
            Ok(Some(v)) => {
                report.push(format!(
                    "film vision: READY source=iwd enable={} contrast={:.4} brightness={:.4} desaturation={:.5}",
                    v.enable, v.contrast, v.brightness, v.desaturation
                ));
                Some(v)
            }
            Ok(None) => None,
            Err(e) => {
                report.push(format!("film vision: {vision_name} unreadable ({e:?})"));
                None
            }
        },
        None => {
            report.push(format!("film vision: {vision_name} not in the game's IWDs"));
            None
        }
    };

    // The sun (the fallback pass's IW3 branch adds it by its baked
    // visibility, as CoD4's lit shaders do).
    let sun = &world.sun;
    let (pitch, yaw) = (sun.angles[0].to_radians(), sun.angles[1].to_radians());
    // Toward the sun: the sun's angles as a forward vector (pitch down).
    let dir = [pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), -pitch.sin()];
    let mut colour = [
        sun.sun_color[0] * sun.sun_light,
        sun.sun_color[1] * sun.sun_light,
        sun.sun_color[2] * sun.sun_light,
        1.0,
    ];
    // bo2mp: in a Black Ops II map's look, its sun's hue at this sun's
    // strength (the baked light stays this map's).
    if let Some(model_sun) = look.as_ref().and_then(|l| l.capture.world.as_ref()).and_then(|w| w.sun) {
        let strength = (colour[0] + colour[1] + colour[2]) / 3.0;
        // (Its linear colour's ratios, square-rooted: this map lights in
        // gamma space.)
        let hue = model_sun.color.map(|c| c.max(0.0).sqrt());
        let mean = (hue[0] + hue[1] + hue[2]) / 3.0;
        if mean > 1e-4 {
            for k in 0..3 {
                colour[k] = strength * hue[k] / mean;
            }
            report.push(format!("bo2 look: sun hue {:?} from the model map", hue));
        }
    }
    let lighting = asset_world::T6WorldLighting {
        sun: asset_world::WorldSunLight { direction: dir, diffuse: colour, specular: colour },
        exposure_scale: 1.0,
        sky: [1.0, 0.0, 0.0, 1.0],
    };
    // The baked light: each lightmap page pair from the zone (the images
    // carry their pixels), packed for the fallback pass's IW3 branch.
    let mut lightmap_gaps = Vec::new();
    let lightmaps: Vec<Option<asset_world::WorldLightmap>> = world
        .lightmaps
        .iter()
        .enumerate()
        .map(|(i, pair)| {
            let raw = |id: Option<iw3::zone::AssetId>| {
                let image = id.and_then(|id| zone.image(id))?;
                let def = image.load_def.as_ref()?;
                Some((def.dimensions[0], def.dimensions[1], def.format, def.data.as_slice(), image.name.as_str()))
            };
            let (Some(p), Some(sec)) = (raw(pair.primary), raw(pair.secondary)) else {
                lightmap_gaps.push(format!("page {i}: images not in the zone"));
                return None;
            };
            match asset_world::world_iw3::iw3_lightmap((p.0, p.1, p.2, p.3), (sec.0, sec.1, sec.2, sec.3), (p.4, sec.4)) {
                Ok(page) => Some(page),
                Err(e) => {
                    lightmap_gaps.push(format!("page {i}: {e}"));
                    None
                }
            }
        })
        .collect();
    report.push(format!(
        "{tag} lightmaps: {}/{} pages decoded{}",
        lightmaps.iter().filter(|p| p.is_some()).count(),
        lightmaps.len(),
        if lightmap_gaps.is_empty() { String::new() } else { format!(" ({})", lightmap_gaps.join("; ")) }
    ));
    match asset_world::world_iw3::build_iw3_world_draw(world, &surface_materials, lightmaps, Some(lighting)) {
        Ok(mut draw) => {
            // The createart fog (the zone's `maps/createart/<map>_art.gsc`).
            // (CoD4: maps/createart/<map>_art.gsc; WaW: maps/mp/createart/.)
            let lower = map.to_ascii_lowercase();
            let art = [format!("maps/createart/{lower}_art.gsc"), format!("maps/mp/createart/{lower}_art.gsc")];
            let fog = zone.assets.iter().find_map(|a| match a {
                iw3::zone::Asset::RawFile(r) if art.iter().any(|n| r.name.eq_ignore_ascii_case(n)) => {
                    legacy_fog(&String::from_utf8_lossy(&r.data))
                }
                _ => None,
            });
            let art = &art[0];
            match &fog {
                Some(f) => {
                    report.push(format!(
                        "{tag} fog: READY start={:.1} half={:.1} opacity={:.3} base={:.0} half height={:.0} sun={}",
                        f.start, f.half, f.opacity, f.base_height, f.half_height, f.sun.is_some()
                    ));
                    draw.t6_lights = legacy_fog_table(f);
                }
                None => report.push(format!("{tag} fog: none ({art} sets none)")),
            }
            // bo2mp: the model map's fog, probes and grade (`bo2_look`).
            if let Some(look) = &look
                && let Some(model) = look.capture.world.as_ref()
            {
                if let Some(rows) = super::t6::world_fog_rows(model) {
                    draw.t6_lights.resize(32, [0.0; 16]);
                    draw.t6_lights[30] = rows[0];
                    draw.t6_lights[31] = rows[1];
                    report.push("bo2 look: fog from the model map".to_owned());
                }
                if let Some(line) =
                    super::t6_materials::link_reflection_probes(&mut materials, &look.capture, model, look.model)
                {
                    draw.t6_probes = model.reflection_probes.iter().map(|p| p.lighting_sh).collect();
                    // Every surface reflects the model map's brightest real
                    // probe (this map has none of its own in that form).
                    let best = model
                        .reflection_probes
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            p.image
                                .and_then(|k| look.capture.images.get(k.index))
                                .is_some_and(|img| img.embedded.as_ref().is_some_and(|e| matches!(e.dxgi_format, 77 | 78)))
                        })
                        .max_by(|a, b| {
                            let l = |p: &asset_t6::ReflectionProbeRef| p.lighting_sh[0][0] + p.lighting_sh[0][1] + p.lighting_sh[0][2];
                            l(a.1).total_cmp(&l(b.1))
                        })
                        .map_or(0, |(i, _)| i as u8);
                    draw.surface_reflection_probes.iter_mut().for_each(|p| *p = best);
                    report.push(format!("bo2 look: {line}; surfaces reflect probe {best}"));
                }
                let (grade, line) = super::t6::t6_grade(&look.capture, model, Some(&look.packs), look.model);
                report.push(format!("bo2 look: {line}"));
                super::bo2_look::set_pending_grade(grade);
            }
            report.push(format!(
                "world mesh: {} vertices, {} triangles, {} surfaces ({title}, fallback draw with colour maps{})",
                draw.stats.vertices, draw.stats.triangles, draw.stats.surfaces,
                if draw.iw3_lightmaps { " and its lightmaps" } else { "" }
            ));
            let (min, max) = (draw.stats.min, draw.stats.max);
            LoadedWorld {
                scripts,
                world: PreparedWorld {
                    draw: Some(draw),
                    min,
                    max,
                    film_vision,
                    policy: WorldDrawPolicy::t5(),
                    ..Default::default()
                },
                materials,
                collision,
                spawns,
                report,
                ..Default::default()
            }
        }
        Err(error) => {
            let mut loaded = LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("{title} world mesh: {error}"),
                Some(game.addr_world_mesh),
            );
            loaded.scripts = scripts;
            loaded.collision = collision;
            loaded.spawns = spawns;
            loaded.report.extend(report);
            loaded
        }
    }
}

#[cfg(test)]
mod fog_tests {
    #[test]
    fn exp_fog_fills_the_fog_slots() {
        let fog = super::legacy_fog("setExpFog(800, 6000, 0.5, 0.6, 0.7, 0);").expect("parses");
        let t = super::legacy_fog_table(&fog);
        assert_eq!(t.len(), 32);
        assert_eq!(&t[31][0..4], &[800.0, 6000.0, 1.0, 1.0]);
        assert!((t[31][4] - 0.25).abs() < 1e-6);
        assert_eq!(t[30][1], 0.0);
        assert!(t[..30].iter().all(|r| r[3] == 0.0));
        // WaW's height fog, past the usage comment's own mention.
        let waw = "setVolFog(0, 4400, 1400, 0, 0.552, 0.682, 0.71, 0);\n// setVolFog(<startDist>, <halfwayDist>, ...)";
        let f = super::legacy_fog(waw).expect("parses");
        assert_eq!((f.start, f.half, f.half_height, f.base_height), (0.0, 4400.0, 1400.0, 0.0));
        assert!((f.colour[0] - 0.552).abs() < 1e-6);
    }
}

#[cfg(test)]
mod mip_tests {
    #[test]
    fn a_full_chain_keeps_alpha_coverage() {
        // 8x4: the left half opaque, the right clear (a fence's edge).
        let mut top = Vec::new();
        for _y in 0..4 {
            for x in 0..8 {
                top.extend_from_slice(&[200, 100, 50, if x < 4 { 255 } else { 0 }]);
            }
        }
        let (data, levels) = super::mip_chain(8, 4, top);
        assert_eq!(levels, 4);
        // 8x4 + 4x2 + 2x1 + 1x1 texels.
        assert_eq!(data.len(), (32 + 8 + 2 + 1) * 4);
        // Level 1 (4x2): half of it still passes the 0.5 test.
        let l1 = &data[32 * 4..40 * 4];
        let pass = l1.chunks_exact(4).filter(|p| p[3] >= 128).count();
        assert_eq!(pass, 4);
        // Colour averaged in linear light stays the colour.
        assert_eq!(l1[0], 200);
    }
}

#[cfg(test)]
mod mip_bench {
    #[test]
    #[ignore = "timing"]
    fn a_1024_chain() {
        let top: Vec<u8> = (0..1024 * 1024 * 4).map(|i| (i % 251) as u8).collect();
        let t = std::time::Instant::now();
        let (_, levels) = super::mip_chain(1024, 1024, top);
        eprintln!("1024^2 chain: {levels} levels in {:?}", t.elapsed());
    }
}
