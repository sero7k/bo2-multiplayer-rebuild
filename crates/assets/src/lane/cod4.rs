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
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP};
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

fn rgba_image(w: u32, h: u32, rgba: Vec<u8>) -> Image {
    Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
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
) -> Vec<Option<usize>> {
    let owner = ZoneOwner::intern(owner);
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
        if let Some(texture) = colour
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
    let surface_materials =
        link_world_materials(tag, zone, world, main.as_deref(), &mut materials, &map, &mut report);
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
    let colour = [
        sun.sun_color[0] * sun.sun_light,
        sun.sun_color[1] * sun.sun_light,
        sun.sun_color[2] * sun.sun_light,
        1.0,
    ];
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
        Ok(draw) => {
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
