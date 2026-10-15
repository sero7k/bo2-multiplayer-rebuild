//! bo2mp: Call of Duty: Ghosts (IW6) multiplayer maps.
//!
//! A Ghosts map zone (`IWffu100`, version 0x235) is walked by
//! `fastfile_iw6`'s exact walk and read into plain data by
//! `fastfile_iw6::map`; this turns it into the match's world the way the
//! Call of Duty 4 / World at War lane does (`super::cod4`): the world is
//! handed to the IW3 world builder (Ghosts' world vertex is CoD4's layout),
//! the collision to the IW3 clip builder (brushes, terrain triangles under
//! the AABB forest, brush models), the lightmaps converted to CoD4's form
//! (`fastfile_iw6::map::iw3_lightmap`), the entities named (Ghosts stores
//! their keys as numbers and its spawn points beside them) and its trigger
//! models (`"model" "?N"`) kept for the scripts. Materials and placed
//! models go through the Ghosts guest-gun builder (`super::iw6::Builder`):
//! colour maps and layers from the zone or its `imagefile<N>.pak`s.
//! Black Ops II's rules then add everything else (`attach_t6_rules`).
//!
//! No ZoneGame of its own: the map walk sends a zone of Ghosts' version
//! here (`asset_transport::legacy_game_for_path` says `iw6`).

use std::collections::HashMap;
use std::path::Path;

use asset_core::AssetNamespace;
use asset_material::MaterialCatalog;
use asset_transport::progress::{LoadProgress, StageId};
use asset_world::WorldDrawPolicy;
use fastfile_iw6::map::IwMap;

use super::LoadedWorld;
use crate::lane_capability::PreparedCapability;
use crate::session_load::PreparedWorld;

/// The largest level kept of a world surface's pictures and of a placed
/// model's (`BO2MP_GH_MAP_TEXTURE=<size>` changes both).
const WORLD_TEXTURE_MAX: u32 = 512;

fn texture_max() -> u32 {
    std::env::var("BO2MP_GH_MAP_TEXTURE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|&v: &u32| (64..=4096).contains(&v))
        .unwrap_or(WORLD_TEXTURE_MAX)
}

/// The images and materials Ghosts' map zones name but do not hold
/// (`,name`): those of `common_mp` and `common_core_mp` (and their patches)
/// beside the map, read once per process (two walks of a second or two;
/// image records, inline pixels and material texture lists kept, streamed
/// pixels read from the paks when a map wants them).
fn shared_images(zone_dir: &Path) -> Option<std::sync::Arc<fastfile_iw6::shared::SharedImages>> {
    use std::sync::{Arc, Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<std::path::PathBuf, Arc<fastfile_iw6::shared::SharedImages>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut guard = cache.lock().ok()?;
    if let Some(hit) = guard.get(zone_dir) {
        return Some(hit.clone());
    }
    let mut shared = fastfile_iw6::shared::SharedImages::default();
    for z in ["patch_common_mp", "common_mp", "patch_common_core_mp", "common_core_mp"] {
        let p = zone_dir.join(format!("{z}.ff"));
        if !p.is_file() {
            continue;
        }
        let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| shared.add_zone(&p)));
        match read {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => diag::warn!(World, "bo2mp iw6: shared images: {e}"),
            Err(_) => diag::warn!(World, "bo2mp iw6: shared images: reading {} panicked", p.display()),
        }
    }
    let shared = Arc::new(shared);
    guard.insert(zone_dir.to_path_buf(), shared.clone());
    Some(shared)
}

fn rotate(v: [f32; 3], axis: &[[f32; 3]; 3]) -> [f32; 3] {
    std::array::from_fn(|k| axis[0][k] * v[0] + axis[1][k] * v[1] + axis[2][k] * v[2])
}

fn place_point(v: [f32; 3], origin: [f32; 3], axis: &[[f32; 3]; 3]) -> [f32; 3] {
    let r = rotate(v, axis);
    std::array::from_fn(|k| origin[k] + r[k])
}

/// CoD4's packed unit vector (as `asset_model::unpack_unit_vec` reads it,
/// scale byte 63: 1/127 a step).
fn iw3_unit_vec(v: [f32; 3]) -> u32 {
    let b = |x: f32| (x.clamp(-1.0, 1.0) * 127.0 + 127.0).round().clamp(0.0, 255.0) as u32;
    b(v[0]) | b(v[1]) << 8 | b(v[2]) << 16 | 63 << 24
}

/// Loads a Ghosts map zone into the match's world. A zone that cannot be
/// read becomes a gap with the reason, never a crash.
pub(crate) fn load_iw6_world(path: &Path, progress: &LoadProgress, material_seed: MaterialCatalog) -> LoadedWorld {
    let mut report = vec![format!("game: IW6 / Ghosts ({})", path.display())];
    let stage = progress.begin_scoped(StageId::MapAssets, "ghosts zone", None);
    let started = std::time::Instant::now();
    let read = std::panic::catch_unwind(|| -> Result<_, String> {
        let zone = fastfile_iw6::Zone::open(path).map_err(|e| e.to_string())?;
        let (cap, stop) = fastfile_iw6::walk::walk_zone(&zone);
        Ok((zone, cap, stop))
    });
    let (zone, cap, stop) = match read {
        Ok(Ok(z)) => {
            stage.done();
            z
        }
        Ok(Err(error)) => {
            stage.fail();
            diag::warn!(World, "bo2mp iw6: {}: {error}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Ghosts zone: {error}"),
                Some("assets::lane::iw6_map::load_world/zone"),
            );
        }
        Err(panic) => {
            stage.fail();
            let what = panic
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_owned());
            diag::warn!(World, "bo2mp iw6: {}: reader panicked: {what}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Ghosts zone could not be read: {what}"),
                Some("assets::lane::iw6_map::load_world/panic"),
            );
        }
    };
    report.push(format!(
        "iw6 zone: walked {} of {} assets [{:.1}s]{}",
        cap.walked,
        zone.assets.assets.len(),
        started.elapsed().as_secs_f32(),
        stop.as_ref().map(|s| format!(", stopped at {} ({:?}): {}", s.asset_index, s.asset_type, s.error)).unwrap_or_default()
    ));
    let map = fastfile_iw6::map::read_map(&cap, &zone.image, &zone.assets.script_strings);
    report.extend(map.notes.iter().cloned());
    let loaded = build_iw6_world(path, &zone, &cap, &map, report, progress, material_seed);
    diag::info!(
        World,
        "bo2mp iw6: {} loaded: {}",
        path.display(),
        loaded.report.iter().filter(|l| l.starts_with("iw6 ") || l.starts_with("world mesh")).cloned().collect::<Vec<_>>().join("; ")
    );
    loaded
}

/// The spawn points in the map's entities (`mp_*_spawn*`).
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

/// Ghosts' collision as CoD4's `clipMap_t` (what the IW3 clip builder reads).
fn iw3_clip(clip: &fastfile_iw6::map::Clip) -> iw3::zone::ClipMap {
    iw3::zone::ClipMap {
        name: clip.name.clone(),
        planes: clip.planes.iter().map(|p| iw3::zone::Plane { normal: p.normal, dist: p.dist }).collect(),
        static_models: Vec::new(),
        materials: clip
            .materials
            .iter()
            .map(|(name, surface_flags, content_flags)| iw3::zone::ClipMaterial {
                name: name.clone(),
                surface_flags: *surface_flags,
                content_flags: *content_flags,
            })
            .collect(),
        brushes: clip
            .brushes
            .iter()
            .map(|b| iw3::zone::Brush {
                mins: b.mins,
                maxs: b.maxs,
                contents: b.contents,
                side_planes: b.sides.iter().map(|s| s.0).collect(),
                side_materials: b.sides.iter().map(|s| u32::from(s.1)).collect(),
                axial_materials: b.axial_materials,
            })
            .collect(),
        verts: clip.verts.clone(),
        tri_indices: clip.tri_indices.clone(),
        partitions: clip
            .partitions
            .iter()
            .map(|&(tri_count, first_tri, _)| iw3::zone::CollisionPartition { tri_count, first_tri })
            .collect(),
        aabb_trees: clip
            .aabb_trees
            .iter()
            .map(|t| iw3::zone::CollisionAabbTree {
                origin: t.origin,
                half_size: t.half_size,
                material_index: t.material_index,
                child_count: t.child_count,
                index: t.index,
            })
            .collect(),
        cmodels: clip
            .cmodels
            .iter()
            .map(|m| iw3::zone::CModel {
                mins: m.mins,
                maxs: m.maxs,
                radius: m.radius,
                leaf_brush_node: m.leaf_brush_node,
                first_coll_aabb: m.first_coll_aabb,
                coll_aabb_count: m.coll_aabb_count,
            })
            .collect(),
        map_ents: None,
        dyn_ent_counts: [0, 0],
        dyn_ents: Vec::new(),
        leaf_brush_nodes: clip
            .leaf_brush_nodes
            .iter()
            .map(|n| iw3::zone::LeafBrushNode {
                leaf_brush_count: n.leaf_brush_count,
                brushes: n.brushes.clone(),
                child_offsets: n.child_offsets,
            })
            .collect(),
    }
}

/// Ghosts' HDR sun light (linear, its brightest channel) as CoD4's
/// gamma-space `sunlight`, under the exposure the map's lightmaps take
/// (`fastfile_iw6::map::light_exposure`), at most
/// `fastfile_iw6::map::SUN_GAMMA_MAX`.
fn sun_gamma(peak: f32, exposure: f32) -> f32 {
    fastfile_iw6::map::iw3_sun_gamma(peak, exposure)
}

/// Ghosts' world vertex colour (bytes R, G, B, A: a layered surface's
/// weights in green for layer 1 and blue for layer 2) as CoD4's D3DCOLOR
/// (bytes B, G, R, A), which the IW3 world builder reads. Read as it was,
/// layer 2's weight came from the red byte (near 255 everywhere: the second
/// layer covered every two-layer surface) and tints swapped red and blue.
fn iw3_colour(rgba: u32) -> u32 {
    let [r, g, b, a] = rgba.to_le_bytes();
    u32::from_le_bytes([b, g, r, a])
}

/// Ghosts' drawn world as CoD4's `GfxWorld` (materials and lightmaps go
/// beside it, not in it).
fn iw3_world(w: &fastfile_iw6::map::World, sun: Option<fastfile_iw6::map::Sun>, exposure: f32) -> iw3::zone::GfxWorld {
    let range = |(a, b): (u32, u32)| a.min(b)..b;
    // Lit opaque, decal and transparent run on from one another.
    let lit = w.ranges[0].0.min(w.ranges[2].0)..w.ranges[0].1.max(w.ranges[2].1);
    // The sun's angles from its direction (toward the sun): pitch down.
    let (angles, colour) = match sun {
        Some(s) => {
            let d = s.dir;
            let yaw = d[1].atan2(d[0]).to_degrees();
            let pitch = (-d[2]).clamp(-1.0, 1.0).asin().to_degrees();
            let peak = s.color.iter().copied().fold(0.0f32, f32::max).max(1e-6);
            ([pitch, yaw, 0.0], (s.color.map(|c| c / peak), peak))
        }
        None => ([-60.0, 45.0, 0.0], ([1.0; 3], 1.0)),
    };
    iw3::zone::GfxWorld {
        name: w.name.clone(),
        base_name: w.name.clone(),
        indices: w.indices.clone(),
        vertices: w
            .vertices
            .iter()
            .map(|v| iw3::zone::WorldVertex {
                xyz: v.xyz,
                binormal_sign: v.binormal_sign,
                color: iw3_colour(v.color),
                tex_coord: v.tex_coord,
                lmap_coord: v.lmap_coord,
                normal: iw3_unit_vec(v.normal),
                tangent: iw3_unit_vec(v.tangent),
            })
            .collect(),
        surfaces: w
            .surfaces
            .iter()
            .map(|s| iw3::zone::GfxSurface {
                first_vertex: s.first_vertex as i32,
                vertex_count: s.vertex_count,
                tri_count: s.tri_count,
                base_index: s.base_index as i32,
                material: None,
                lightmap_index: s.lightmap_index,
                reflection_probe_index: s.reflection_probe_index,
                primary_light_index: 0,
                flags: s.flags,
                bounds: [s.mins, s.maxs],
            })
            .collect(),
        static_models: Vec::new(),
        sky_start_surfs: w.sky_surfaces.iter().map(|&s| s as i32).collect(),
        sky_image: None,
        sun: iw3::zone::SunParse {
            name: String::new(),
            ambient_scale: 0.0,
            ambient_color: [0.0; 3],
            diffuse_fraction: 0.0,
            // (Ghosts' sun colour is HDR linear light, 9 on Prison Break,
            // its brightest channel the light and the rest the colour. The
            // fallback draw adds the sun in CoD4's gamma space beside the
            // lightmap's sqrt(light), both under one exposure: `sun_gamma`.)
            sun_light: sun_gamma(colour.1, exposure),
            sun_color: colour.0,
            diffuse_color: colour.0,
            angles,
        },
        sun_flare: Default::default(),
        sun_color_from_bsp: colour.0,
        lightmaps: Vec::new(),
        reflection_probes: Vec::new(),
        mins: w.mins,
        maxs: w.maxs,
        lit_surfs: lit,
        decal_surfs: range(w.ranges[1]),
        emissive_surfs: range(w.ranges[3]),
        models: w
            .brush_models
            .iter()
            .map(|m| iw3::zone::GfxBrushModel {
                bounds: [m.mins, m.maxs],
                surface_count: m.surface_count.min(u32::from(u16::MAX)) as u16,
                start_surf_index: m.start_surf.min(u32::from(u16::MAX)) as u16,
            })
            .collect(),
        light_grid: Default::default(),
        dpvs: None,
    }
}

/// The createart fog of a Ghosts map (`maps/createart/<map>_fog.gsc`): its
/// first `create_vision_set_fog` block's `ent.<field> = value;` lines.
fn ghosts_fog(script: &str) -> Option<super::cod4::LegacyFog> {
    let first = script.split("create_vision_set_fog").nth(1)?;
    let mut num: HashMap<String, f32> = HashMap::new();
    let mut vec3: HashMap<String, [f32; 3]> = HashMap::new();
    for line in first.lines() {
        let Some((key, value)) = line.trim().strip_prefix("ent.").and_then(|l| l.split_once('=')) else { continue };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().trim_end_matches(';').trim();
        if let Some(inner) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) {
            let v: Vec<f32> = inner.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            if v.len() == 3 {
                vec3.insert(key, [v[0], v[1], v[2]]);
            }
        } else if let Ok(v) = value.parse::<f32>() {
            num.insert(key, v);
        }
    }
    let n = |k: &str| num.get(k).copied();
    let half = n("halfwaydist")?;
    if half <= 0.0 {
        return None;
    }
    let colour = [n("red")?, n("green")?, n("blue")?].map(|c| c.clamp(0.0, 1.0));
    let sun = (n("sunfogenabled").unwrap_or(0.0) > 0.0)
        .then(|| {
            Some(asset_world::SunFog {
                color_rgb: [n("sunred")?, n("sungreen")?, n("sunblue")?].map(|c| c.clamp(0.0, 1.0)),
                sun_dir: *vec3.get("sundir")?,
                begin_angle_deg: n("sunbeginfadeangle").unwrap_or(0.0),
                end_angle_deg: n("sunendfadeangle").unwrap_or(90.0),
                scale: n("normalfogscale").unwrap_or(1.0),
            })
        })
        .flatten();
    Some(super::cod4::LegacyFog {
        start: n("startdist").unwrap_or(0.0).max(0.0),
        half,
        opacity: n("maxopacity").unwrap_or(1.0).clamp(0.0, 1.0),
        colour,
        sun,
        base_height: 1.0e7,
        half_height: 1.0e9,
    })
}

fn build_iw6_world(
    path: &Path,
    zone: &fastfile_iw6::Zone,
    cap: &fastfile_iw6::walk::Capture,
    map: &IwMap,
    mut report: Vec<String>,
    progress: &LoadProgress,
    material_seed: MaterialCatalog,
) -> LoadedWorld {
    super::bo2_look::set_pending_grade(None);
    let map_name = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let lower = map_name.to_ascii_lowercase();

    // Collision.
    let collision = match map.clip.as_ref().map(|c| asset_world::world_iw3::build_iw3_clip_collision(&iw3_clip(c))) {
        Some(Ok(mut clip)) => {
            // Each partition's vertex segment (CoD4's have none): its
            // triangles index the vertices from `segment * 1024`.
            if let Some(src) = &map.clip {
                let mesh = std::sync::Arc::make_mut(&mut clip.mesh);
                for (p, &(_, _, segment)) in mesh.partitions.iter_mut().zip(&src.partitions) {
                    p.first_vert_segment = segment.into();
                }
            }
            clip.trigger_models = map
                .trigger_models
                .iter()
                .map(|hulls| {
                    hulls
                        .iter()
                        .map(|h| asset_world::MapTriggerHull { mid: h.mid, half: h.half, slabs: h.slabs.clone() })
                        .collect()
                })
                .collect();
            let origins: Vec<[f32; 3]> = map.spawns.iter().map(|s| s.origin).collect();
            let census = asset_world::world_iw3::spawn_ground_census(&clip, &origins);
            report.push(format!(
                "iw6 clip: {} brushes ({} the world's), {} mesh tris in {} trees, {} models, {} trigger models; {census}",
                clip.brushes.len(),
                clip.cmodels.first().map_or(0, |m| m.num_brushes),
                clip.mesh.tri_indices.len() / 3,
                clip.mesh.aabb_roots.len(),
                clip.cmodels.len(),
                clip.trigger_models.len(),
            ));
            if census.spawns > 0 && census.on_ground * 2 < census.spawns {
                diag::warn!(World, "bo2mp iw6: {map_name}: most spawn points have no floor of their own in the collision ({census})");
            }
            Some(clip)
        }
        Some(Err(error)) => {
            report.push(format!("iw6 clip gap: {error}"));
            None
        }
        None => {
            report.push("iw6 clip gap: zone has no clipMap".to_owned());
            None
        }
    };

    // Entities and spawns. Spawn records over nothing are left out: Ghosts
    // parks the spawns of modes a map does not play in an empty corner
    // (Prison Break's Capture the Flag and Search and Rescue spawns stand
    // over a void); Black Ops II's rules copy those modes' spawns from the
    // map's others instead.
    // A second level built above the first (Strikezone's after-the-strike
    // map): its spawn points, geometry and placed models are left out.
    let level_cut = fastfile_iw6::map::alternate_level_cut(&map.spawns);
    let mut spawn_records = map.spawns.clone();
    if let Some(cut) = level_cut {
        let before = spawn_records.len();
        spawn_records.retain(|s| s.origin[2] < cut);
        report.push(format!(
            "iw6 second level above z {cut:.0} left out: {} of {before} spawn records there",
            before - spawn_records.len()
        ));
    }
    let mut parked = 0usize;
    if let Some(clip) = &collision {
        const MINS: [f32; 3] = [-15.0, -15.0, 0.0];
        const MAXS: [f32; 3] = [15.0, 15.0, 70.0];
        spawn_records.retain(|s| {
            let o = s.origin;
            let (down, _) = asset_world::world_iw3::trace_clip_collision(
                clip,
                [o[0], o[1], o[2] + 16.0],
                [o[0], o[1], o[2] - 1024.0],
                MINS,
                MAXS,
                asset_world::MASK_PLAYER_SOLID,
            );
            let keep = down < 1.0;
            parked += usize::from(!keep);
            keep
        });
    }
    let entity_string = IwMap { spawns: spawn_records, entities: map.entities.clone(), ..Default::default() }.entity_string();
    let entities = iw3::ents::parse(&entity_string);
    // Brush models their entities place as the map starts (doors,
    // elevators, walls, water sheets; `fastfile_iw6::map::placed_brush_models`).
    let placed_bmodels: Vec<(std::ops::Range<usize>, [f32; 3], [[f32; 3]; 3])> =
        fastfile_iw6::map::placed_brush_models(map, level_cut).into_iter().map(|b| (b.surfaces, b.origin, b.axis)).collect();
    let spawns = spawns_of(&entities);
    report.push(format!(
        "iw6 entities: {}, spawn points {} ({} spawn records, {parked} over nothing left out), {} trigger models",
        entities.len(),
        spawns.len(),
        map.spawns.len(),
        map.trigger_models.len()
    ));
    let mut scripts = crate::ScriptSources::default();
    scripts.set_entities(entity_string);
    // The compass frame (the two `minimap_corner` origins) and picture.
    let corners: Vec<[f32; 2]> = entities
        .iter()
        .filter(|e| e.get("targetname") == Some("minimap_corner"))
        .filter_map(|e| e.origin().map(|o| [o[0], o[1]]))
        .collect();
    let mut facts = crate::MapFacts::default();
    if let [a, b] = corners[..] {
        facts.minimap_corners = Some(asset_world::MinimapCorners { a, b });
    }
    let compass_name = format!("compass_map_{lower}");
    if cap.images.iter().any(|i| i.name.eq_ignore_ascii_case(&compass_name)) {
        facts.compass.material = Some(compass_name.clone());
        facts.compass.image = Some(compass_name);
    }
    report.push(format!(
        "iw6 compass: corners {}, picture {}",
        if facts.minimap_corners.is_some() { "found" } else { "missing" },
        facts.compass.image.as_deref().unwrap_or("missing")
    ));

    let Some(world) = map.world.as_ref() else {
        let mut loaded = LoadedWorld::with_gap(
            WorldDrawPolicy::t5(),
            PreparedCapability::PreparedWorld,
            "Ghosts zone has no GfxWorld",
            Some("assets::lane::iw6_map::load_world/no_world"),
        );
        loaded.scripts = scripts;
        loaded.collision = collision;
        loaded.spawns = spawns;
        loaded.report.extend(report);
        return loaded;
    };

    // Materials: world surfaces, then the placed models'.
    let stage = progress.begin_scoped(StageId::MapAssets, "ghosts materials", None);
    let max = texture_max();
    let mut materials = material_seed;
    materials.set_capture_ns(AssetNamespace::T6);
    let mut b = super::iw6::Builder::new(cap, zone, path, materials);
    b.shared = path.parent().and_then(shared_images);
    // Surfaces not drawn: the sky's (the sky pass draws behind them), tool,
    // shadow-only and distortion (refraction) surfaces.
    let mut hidden = vec![false; world.surfaces.len()];
    let mut surface_materials = Vec::with_capacity(world.surfaces.len());
    let mut hidden_kinds: HashMap<&'static str, usize> = HashMap::new();
    // Materials of the decal range (`dpvs.litDecalSurfs`).
    let decal_materials: std::collections::HashSet<u32> = {
        let (a, e) = world.ranges[1];
        world.surfaces[(a.min(e) as usize).min(world.surfaces.len())..(e as usize).min(world.surfaces.len())]
            .iter()
            .filter_map(|s| s.material.map(|m| m.index))
            .collect()
    };
    for (i, s) in world.surfaces.iter().enumerate() {
        let Some(m) = s.material else {
            surface_materials.push(None);
            continue;
        };
        let techset = cap
            .materials
            .get(m.index as usize)
            .and_then(|mr| mr.techset)
            .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
            .unwrap_or_default();
        let words: Vec<&str> = techset.split('_').collect();
        let kind = if level_cut.is_some_and(|cut| s.mins[2] > cut) {
            Some("second level")
        } else if words.iter().any(|w| w.starts_with("sky")) {
            Some("sky")
        } else if words.contains(&"tools") {
            Some("tools")
        } else if words.contains(&"shadowcaster") || words.contains(&"onlycast") {
            Some("shadow only")
        } else if words.contains(&"distortion") {
            Some("distortion")
        } else if words.contains(&"distfalloff") {
            // (`hdrportal_lighten`: an exposure portal fading with distance,
            // its colour map the engine's `$white`; drawn, a white sheet.)
            Some("exposure portal")
        } else {
            None
        };
        if let Some(kind) = kind {
            hidden[i] = true;
            *hidden_kinds.entry(kind).or_default() += 1;
            surface_materials.push(None);
            continue;
        }
        surface_materials.push(b.world_material(m.index as usize, max, decal_materials.contains(&m.index)));
    }
    // bo2mp: the sky: its cube under a sky material (the sky surface itself
    // is not drawn; the sky pass draws the cube behind the world).
    let sky_line = match world.sky_images.iter().flatten().next() {
        Some(id) => match b.sky_material(id.index as usize, max.max(1024)) {
            Ok(_) => format!("iw6 sky: {} (cube)", cap.name_of(*id)),
            Err(e) => format!("iw6 sky: {e}"),
        },
        None => "iw6 sky: none in the world".to_owned(),
    };
    report.push(sky_line);
    report.push(format!(
        "iw6 materials: {} world materials linked, {} pictures decoded (at most {max}px; {} from {:?} or built in), {} failed{}; not drawn: {:?}",
        b.materials.len(),
        b.decoded,
        b.from_shared,
        b.shared.as_ref().map(|s| s.zones.clone()).unwrap_or_default(),
        b.image_failures.len(),
        if b.image_failures.is_empty() {
            String::new()
        } else {
            format!(" (first: {})", b.image_failures.iter().take(4).cloned().collect::<Vec<_>>().join("; "))
        },
        hidden_kinds
    ));

    // The map's exposure (Ghosts tone-maps HDR light per map): from its
    // baked light, for the lightmaps, the sun and the models' ground light.
    let light_key = fastfile_iw6::map::lightmap_key(cap, &zone.image, &world.lightmaps);
    let exposure = fastfile_iw6::map::light_exposure(light_key);
    report.push(format!(
        "iw6 exposure: {exposure:.3} (baked light 90th percentile {})",
        light_key.map_or_else(|| "unmeasured".to_owned(), |k| format!("{k:.3}"))
    ));

    // The placed models (rocks, trees, vehicles, furniture).
    let (prop_meshes, prop_placements) = {
        use bevy::prelude::{Mat3, Quat, Transform, Vec3};
        let mut meshes = Vec::new();
        let mut shared: HashMap<u32, Option<usize>> = HashMap::new();
        let mut placements = vec![None; world.static_models.len()];
        let (mut unreadable, mut lit) = (Vec::new(), 0usize);
        let failures_before = b.image_failures.len();
        for (i, sm) in world.static_models.iter().enumerate() {
            let Some(id) = sm.model else { continue };
            if level_cut.is_some_and(|cut| sm.origin[2] > cut) {
                continue;
            }
            let entry = *shared.entry(id.index).or_insert_with(|| {
                let model = cap.xmodels.get(id.index as usize)?;
                let Some((mut xref, keys)) = b.model_ref(model) else {
                    if unreadable.len() < 8 {
                        unreadable.push(model.name.clone());
                    }
                    return None;
                };
                // A placed model's vertex colour is no tint in Ghosts: its
                // foliage keeps wind and reactive-motion weights there (red,
                // green, blue, alpha all vary), which tinted ferns and grass
                // red; the rest is white.
                for surf in &mut xref.lod0 {
                    for v in surf.verts.chunks_exact_mut(32) {
                        v[16..20].fill(0xff);
                    }
                }
                let linked: HashMap<usize, usize> = keys.into_iter().filter_map(|k| Some((k, b.material(k, max)?))).collect();
                let mesh = asset_world::build_t6_model_mesh(&xref, |k| linked.get(&k.index).copied())?;
                meshes.push(mesh);
                Some(meshes.len() - 1)
            });
            let Some(mesh) = entry else { continue };
            // (Ghosts' ground light is linear, under the map's exposure as
            // its lightmaps; the IW3 grid's packing is sqrt(light / 32).)
            let g = sm.ground_lighting.map(|v| v * exposure);
            let packed_lighting = (g[0] + g[1] + g[2] > 0.0).then(|| {
                lit += 1;
                let p = |v: f32| ((v.max(0.0) / 32.0).sqrt() * 255.0).round().clamp(0.0, 255.0) as u8;
                [p(g[0]), p(g[1]), p(g[2]), 0]
            });
            let basis = Mat3::from_cols(Vec3::from_array(sm.axis[0]), Vec3::from_array(sm.axis[1]), Vec3::from_array(sm.axis[2]));
            placements[i] = Some(asset_world::StaticModelPlacement {
                mesh,
                transform: Transform {
                    translation: Vec3::from_array(sm.origin),
                    rotation: Quat::from_mat3(&basis),
                    scale: Vec3::splat(sm.scale),
                },
                origin: sm.origin,
                axis: sm.axis,
                scale: sm.scale,
                cull_dist: sm.cull_dist.clamp(0.0, f32::from(u16::MAX)) as u16,
                reflection_probe_index: 0,
                primary_light_index: 0,
                flags: sm.flags as u8,
                packed_lighting,
                t6_primary: [0, 0],
            });
        }
        report.push(format!(
            "iw6 static models: {} placed of {}, {} distinct models built ({} unreadable {:?}), {} with ground light, {} more picture failures",
            placements.iter().filter(|p| p.is_some()).count(),
            world.static_models.len(),
            meshes.len(),
            unreadable.len(),
            unreadable,
            lit,
            b.image_failures.len() - failures_before
        ));
        (meshes, placements)
    };
    let mut materials = std::mem::take(&mut b.catalog);
    drop(b);
    stage.done();

    // The map's film vision (`vision/<map>.vision` in the zone).
    let vision_name = format!("vision/{lower}.vision");
    let film_vision = match fastfile_iw6::map::raw_file(cap, &zone.image, &vision_name) {
        Some(bytes) => match asset_world::parse_film_vision_rawfile(&vision_name, &bytes, false) {
            Ok(Some(v)) => {
                report.push(format!(
                    "film vision: READY source=zone enable={} contrast={:.4} brightness={:.4} desaturation={:.5}",
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
            report.push(format!("film vision: {vision_name} not in the zone"));
            None
        }
    };

    // The baked light, in CoD4's form.
    let mut lightmap_gaps = Vec::new();
    let lightmaps: Vec<Option<asset_world::WorldLightmap>> = world
        .lightmaps
        .iter()
        .enumerate()
        .map(|(i, &pair)| {
            let names = (
                pair.0.map(|id| cap.name_of(id).to_owned()).unwrap_or_default(),
                pair.1.map(|id| cap.name_of(id).to_owned()).unwrap_or_default(),
            );
            let page = match fastfile_iw6::map::iw3_lightmap_exposed(cap, &zone.image, pair, exposure) {
                Ok(page) => page,
                Err(e) => {
                    lightmap_gaps.push(format!("page {i}: {e}"));
                    return None;
                }
            };
            let (pw, ph, p) = &page.primary;
            let (sw, sh, s) = &page.secondary;
            match asset_world::world_iw3::iw3_lightmap(
                (*pw, *ph, asset_world::world_iw3::IW3_LIGHTMAP_PRIMARY_FORMAT, p),
                (*sw, *sh, asset_world::world_iw3::IW3_LIGHTMAP_SECONDARY_FORMAT, s),
                (&names.0, &names.1),
            ) {
                Ok(page) => Some(page),
                Err(e) => {
                    lightmap_gaps.push(format!("page {i}: {e:?}"));
                    None
                }
            }
        })
        .collect();
    report.push(format!(
        "iw6 lightmaps: {}/{} pages converted{}",
        lightmaps.iter().filter(|p| p.is_some()).count(),
        lightmaps.len(),
        if lightmap_gaps.is_empty() { String::new() } else { format!(" ({})", lightmap_gaps.join("; ")) }
    ));

    let mut iw3w = iw3_world(world, map.sun, exposure);
    // The brush models their entities place (doors, elevators, walls,
    // water sheets): drawn where the entity stands, as the map starts.
    {
        let n = iw3w.surfaces.len();
        let listed = |i: usize| {
            let i = i as u32;
            [&iw3w.lit_surfs, &iw3w.decal_surfs, &iw3w.emissive_surfs].iter().any(|r| r.contains(&i))
        };
        let listed: Vec<bool> = (0..n).map(listed).collect();
        let mut placed = vec![false; n];
        // (A vertex is moved once, should two surfaces share it.)
        let mut moved = vec![false; iw3w.vertices.len()];
        for (range, origin, axis) in &placed_bmodels {
            for i in range.clone().filter(|&i| i < n) {
                if listed[i] || placed[i] {
                    continue;
                }
                placed[i] = true;
                let surf = &mut iw3w.surfaces[i];
                let first = surf.first_vertex.max(0) as usize;
                let verts = first..(first + usize::from(surf.vertex_count)).min(iw3w.vertices.len());
                if verts.is_empty() {
                    continue;
                }
                let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
                for (vi, v) in iw3w.vertices[verts.clone()].iter_mut().enumerate() {
                    if std::mem::replace(&mut moved[verts.start + vi], true) {
                        continue;
                    }
                    v.xyz = place_point(v.xyz, *origin, axis);
                    v.normal = iw3_unit_vec(rotate(asset_model::unpack_unit_vec(v.normal), axis));
                    v.tangent = iw3_unit_vec(rotate(asset_model::unpack_unit_vec(v.tangent), axis));
                }
                for v in &iw3w.vertices[verts] {
                    for k in 0..3 {
                        lo[k] = lo[k].min(v.xyz[k]);
                        hi[k] = hi[k].max(v.xyz[k]);
                    }
                }
                if lo[0] <= hi[0] {
                    surf.bounds = [lo, hi];
                }
            }
        }
        // Every surface is then listed; what was not (and is not placed)
        // stays undrawn.
        for i in 0..n {
            if !listed[i] && !placed[i] {
                hidden[i] = true;
            }
        }
        iw3w.lit_surfs = 0..n as u32;
        report.push(format!(
            "iw6 brush models: {} placed by their entities ({} surfaces)",
            placed_bmodels.len(),
            placed.iter().filter(|&&p| p).count()
        ));
    }
    let (pitch, yaw) = (iw3w.sun.angles[0].to_radians(), iw3w.sun.angles[1].to_radians());
    let dir = [pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), -pitch.sin()];
    let colour = [
        iw3w.sun.sun_color[0] * iw3w.sun.sun_light,
        iw3w.sun.sun_color[1] * iw3w.sun.sun_light,
        iw3w.sun.sun_color[2] * iw3w.sun.sun_light,
        1.0,
    ];
    report.push(format!(
        "iw6 sun: {} toward {dir:?} colour {colour:?}",
        if map.sun.is_some() { "from the ComWorld" } else { "none in the ComWorld (a default)" }
    ));
    let lighting = asset_world::T6WorldLighting {
        sun: asset_world::WorldSunLight { direction: dir, diffuse: colour, specular: colour },
        exposure_scale: 1.0,
        sky: [0.0, 1.0, 0.0, 1.0],
        sky_parms: [1.0, 0.0],
        saturation: None,
        grade: None,
    };
    report.push(format!(
        "iw6 world: {} surfaces ({} not drawn), {} sky surfaces, ranges {:?}",
        world.surfaces.len(),
        hidden.iter().filter(|&&h| h).count(),
        world.sky_surfaces.len(),
        world.ranges
    ));
    match asset_world::world_iw3::build_iw3_world_draw(&iw3w, &surface_materials, &hidden, lightmaps, Some(lighting)) {
        Ok(mut draw) => {
            let fog_name = format!("maps/createart/{lower}_fog.gsc");
            let fog = fastfile_iw6::map::raw_file(cap, &zone.image, &fog_name)
                .and_then(|b| ghosts_fog(&String::from_utf8_lossy(&b)));
            match &fog {
                Some(f) => {
                    report.push(format!(
                        "iw6 fog: READY start={:.1} half={:.1} opacity={:.3} sun={}",
                        f.start,
                        f.half,
                        f.opacity,
                        f.sun.is_some()
                    ));
                    draw.t6_lights = super::cod4::legacy_fog_table(f);
                }
                None => report.push(format!("iw6 fog: none ({fog_name} sets none)")),
            }
            report.push(format!(
                "world mesh: {} vertices, {} triangles, {} surfaces (Ghosts, fallback draw with colour maps{})",
                draw.stats.vertices,
                draw.stats.triangles,
                draw.stats.surfaces,
                if draw.iw3_lightmaps { " and its lightmaps" } else { "" }
            ));
            let (min, max) = (draw.stats.min, draw.stats.max);
            materials.set_capture_ns(AssetNamespace::T6);
            LoadedWorld {
                scripts,
                world: PreparedWorld {
                    draw: Some(draw),
                    min,
                    max,
                    film_vision,
                    policy: WorldDrawPolicy::t5(),
                    static_model_meshes: prop_meshes,
                    static_model_instances: prop_placements,
                    ..Default::default()
                },
                materials,
                collision,
                spawns,
                facts,
                report,
                ..Default::default()
            }
        }
        Err(error) => {
            let mut loaded = LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Ghosts world mesh: {error}"),
                Some("assets::lane::iw6_map::load_world/world_mesh"),
            );
            loaded.scripts = scripts;
            loaded.collision = collision;
            loaded.spawns = spawns;
            loaded.materials = materials;
            loaded.facts = facts;
            loaded.report.extend(report);
            loaded
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn axes_of_angles() {
        let a = fastfile_iw6::map::axis_of([0.0, 90.0, 0.0]);
        let close = |x: [f32; 3], y: [f32; 3]| x.iter().zip(y).all(|(a, b)| (a - b).abs() < 1e-5);
        assert!(close(a[0], [0.0, 1.0, 0.0]) && close(a[1], [-1.0, 0.0, 0.0]) && close(a[2], [0.0, 0.0, 1.0]));
        let p = super::place_point([10.0, 0.0, 5.0], [100.0, 0.0, 0.0], &a);
        assert!(close(p, [100.0, 10.0, 5.0]));
    }

    #[test]
    fn unit_vectors_pack_as_cod4() {
        for v in [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.6, 0.0, 0.8]] {
            let back = asset_model::unpack_unit_vec(super::iw3_unit_vec(v));
            for k in 0..3 {
                assert!((back[k] - v[k]).abs() < 0.01, "{v:?} -> {back:?}");
            }
        }
    }

    #[test]
    fn ghosts_fog_reads_its_first_block() {
        let script = "main()\n{\n\tent = maps\\mp\\_art::create_vision_set_fog( \"mp_x\" );\n\tent.startDist = 500;\n\tent.halfwayDist = 25000;\n\tent.red = 0.62;\n\tent.green = 0.78;\n\tent.blue = 0.86;\n\tent.maxOpacity = 0.50;\n\tent.sunFogEnabled = 1;\n\tent.sunRed = 0.55;\n\tent.sunGreen = 0.45;\n\tent.sunBlue = 0.31;\n\tent.sunDir = (0.99, 0.112215, -0.0120798);\n\tent.sunBeginFadeAngle = 10;\n\tent.sunEndFadeAngle = 92;\n\tent = maps\\mp\\_art::create_vision_set_fog( \"mp_x_hdr\" );\n\tent.startDist = 1;\n}";
        let f = super::ghosts_fog(script).expect("fog");
        assert_eq!((f.start, f.half, f.opacity), (500.0, 25000.0, 0.5));
        assert!(f.sun.is_some_and(|s| (s.sun_dir[0] - 0.99).abs() < 1e-6));
    }
}
