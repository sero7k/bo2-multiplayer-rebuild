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
    let (mut decoded, mut missing) = (0usize, Vec::new());
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
                    name: AssetRef::Real(format!("iw3:{}", image.name)),
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
            name: AssetRef::Real(format!("iw3:{}", material.name)),
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
        "iw3 materials: {} linked, {decoded} colour maps decoded from the IWDs, {} not found{}",
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

/// Loads a Call of Duty 4 map zone into the match's world.
pub(crate) fn load_iw3_world(
    path: &Path,
    progress: &LoadProgress,
    material_seed: MaterialCatalog,
) -> LoadedWorld {
    let mut report = vec![format!("game: IW3 / Call of Duty 4 ({})", path.display())];
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
    report.push(format!(
        "iw3 zone: {} of {} assets{}",
        zone.stats.top_level_parsed,
        zone.stats.top_level_total,
        zone.stats.stopped_at.as_ref().map_or_else(String::new, |s| format!(", stopped at {s}"))
    ));
    let map = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());

    // Entities and spawns.
    let entity_string = zone.map_ents().map(|m| m.entity_string.clone()).unwrap_or_default();
    let entities = iw3::ents::parse(&entity_string);
    let spawns = spawns_of(&entities);
    report.push(format!("iw3 entities: {}, spawn points {}", entities.len(), spawns.len()));
    let mut scripts = crate::ScriptSources::default();
    scripts.set_entities(entity_string);

    // Collision.
    let collision = match zone.clip_map().map(asset_world::world_iw3::build_iw3_clip_collision) {
        Some(Ok(clip)) => {
            report.push(format!(
                "iw3 clip: {} brushes, {} mesh tris, {} models",
                clip.brushes.len(),
                clip.mesh.tri_indices.len() / 3,
                clip.cmodels.len()
            ));
            Some(clip)
        }
        Some(Err(error)) => {
            report.push(format!("iw3 clip gap: {error}"));
            None
        }
        None => {
            report.push("iw3 clip gap: zone has no clipMap".to_owned());
            None
        }
    };

    let Some(world) = zone.gfx_world() else {
        let mut loaded = LoadedWorld::with_gap(
            WorldDrawPolicy::t5(),
            PreparedCapability::PreparedWorld,
            "CoD4 zone has no GfxWorld",
            Some("assets::lane::iw3::load_world/no_world"),
        );
        loaded.scripts = scripts;
        loaded.collision = collision;
        loaded.spawns = spawns;
        loaded.report.extend(report);
        return loaded;
    };

    // Materials and their colour maps, from the game's own IWDs.
    let stage = progress.begin_scoped(StageId::MapAssets, "cod4 materials", None);
    let main = asset_transport::game_root_for_zone(path).ok().map(|root| root.join("main"));
    let mut materials = material_seed;
    materials.set_capture_ns(AssetNamespace::T6);
    let surface_materials =
        link_world_materials(&zone, world, main.as_deref(), &mut materials, &map, &mut report);
    stage.done();

    // The sun (no lightmaps yet: the fallback pass lights by it).
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
    match asset_world::world_iw3::build_iw3_world_draw(world, &surface_materials, Vec::new(), Some(lighting)) {
        Ok(draw) => {
            report.push(format!(
                "world mesh: {} vertices, {} triangles, {} surfaces (CoD4, fallback draw with colour maps)",
                draw.stats.vertices, draw.stats.triangles, draw.stats.surfaces
            ));
            let (min, max) = (draw.stats.min, draw.stats.max);
            LoadedWorld {
                scripts,
                world: PreparedWorld {
                    draw: Some(draw),
                    min,
                    max,
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
                format!("CoD4 world mesh: {error}"),
                Some("assets::lane::iw3::load_world/world_mesh"),
            );
            loaded.scripts = scripts;
            loaded.collision = collision;
            loaded.spawns = spawns;
            loaded.report.extend(report);
            loaded
        }
    }
}
