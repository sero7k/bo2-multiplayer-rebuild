//! bo2mp: Call of Duty: Infinite Warfare (IW7) maps, multiplayer and
//! Zombies.
//!
//! An IW7 map zone (`IWff0100`, version 0x653) is read whole by
//! `fastfile_iw7::map` (the zone's exact walk, its materials from
//! `techsets_<map>.ff`, its placed models' surfaces from the zone's shared
//! streams in `assetfile<N>.pak`, pictures from `imagefile<N>.pak`); this
//! turns it into the match's world the way the Call of Duty 4 lane does
//! (`super::cod4`): the world goes to the IW3 world builder (IW7's world
//! vertex is CoD4's layout but for its unit vectors), the placed models
//! through Black Ops II's model builder (IW7's packed vertex is BO2's but
//! for its unit vectors), materials onto BO2's draw path by their technique
//! set's name with their colour maps.
//!
//! IW7 collides through Havok shapes; the collision here is the drawn
//! triangles instead (`asset_world::world_iw7`): the world's surfaces and
//! each placed model's coarsest LOD. Spawn points come from the zone's
//! spawn list; the entity string's numbered keys are named
//! (`fastfile_iw7::map::ENTITY_KEYS`). Black Ops II's rules then add
//! everything else (`attach_t6_rules`).
//!
//! No ZoneGame of its own: the map walk sends a zone of IW7's version here
//! (`asset_transport::legacy_game_for_path` says `iw7`).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use asset_core::{AssetEdge, AssetNamespace, AssetRef, T6Blend, T6Cull, T6Draw, ZoneOwner};
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP};
use asset_transport::progress::{LoadProgress, StageId};
use asset_world::WorldDrawPolicy;
use fastfile_iw7::map::{Iw7Map, MapSources};

use super::LoadedWorld;
use crate::lane_capability::PreparedCapability;
use crate::session_load::PreparedWorld;

/// The largest level kept of a surface's or model's colour map
/// (`BO2MP_IW7_MAP_TEXTURE=<size>` changes it).
const TEXTURE_MAX: u32 = 512;

/// A placed model smaller than this (its radius, scaled) does not collide.
const COLLIDE_MIN_RADIUS: f32 = 24.0;

fn texture_max() -> u32 {
    std::env::var("BO2MP_IW7_MAP_TEXTURE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|&v: &u32| (64..=4096).contains(&v))
        .unwrap_or(TEXTURE_MAX)
}

/// CoD4's packed unit vector (as `asset_model::unpack_unit_vec` reads it,
/// scale byte 63: 1/127 a step).
fn iw3_unit_vec(v: [f32; 3]) -> u32 {
    let b = |x: f32| (x.clamp(-1.0, 1.0) * 127.0 + 127.0).round().clamp(0.0, 255.0) as u32;
    b(v[0]) | b(v[1]) << 8 | b(v[2]) << 16 | 63 << 24
}

/// Black Ops II's packed unit vector: signed 10:10:10 over 511.
fn t6_unit_vec(v: [f32; 3]) -> u32 {
    let mut out = 0u32;
    for (k, c) in v.iter().enumerate() {
        let s = (c.clamp(-1.0, 1.0) * 511.0).round() as i32;
        out |= ((s as u32) & 0x3ff) << (10 * k);
    }
    out
}

/// An IW7 model vertex (`GfxPackedVertex`) as Black Ops II's: the same
/// layout but for the unit vectors (unsigned 10:10:10 over 1023 there) and
/// the binormal sign (packed differently: taken as +1).
fn iw7_vertex_to_t6(v: &[u8; 32]) -> [u8; 32] {
    let mut out = *v;
    out[12..16].copy_from_slice(&1.0f32.to_le_bytes());
    for at in [24usize, 28] {
        let p = u32::from_le_bytes([v[at], v[at + 1], v[at + 2], v[at + 3]]);
        out[at..at + 4].copy_from_slice(&t6_unit_vec(fastfile_iw7::map::unpack_unit(p)).to_le_bytes());
    }
    out
}

/// What a technique set is, by the words of its name.
fn hidden_kind(technique_set: &str) -> Option<&'static str> {
    let words: Vec<&str> = technique_set.split('_').collect();
    if words.iter().any(|w| w.starts_with("sky")) {
        Some("sky")
    } else if words.contains(&"tools") {
        Some("tools")
    } else if words.contains(&"shadowcaster") || words.contains(&"onlycast") {
        Some("shadow only")
    } else if words.contains(&"distortion") {
        Some("distortion")
    } else if words.contains(&"threed") {
        Some("3d")
    } else {
        None
    }
}

/// An IW7 technique set as Black Ops II's draw: by its name (`w_`/`wc_`
/// world and `mo_`/`mopw_`/`mcopw_` model techniques read like BO2's, the
/// same `r0c0`, `t0c0`, `b0c0` base and blend words), without BO2's layers
/// (IW7's vertex colour is no layer weight) and drawn double-sided for
/// models.
fn iw7_draw(technique_set: &str, sort_key: u8, model: bool) -> T6Draw {
    let mut draw = T6Draw::from_technique_set(technique_set, sort_key, None);
    draw.layered = false;
    draw.layers = [0, 0];
    draw.water = false;
    draw.flow = false;
    draw.tile = false;
    draw.flag = false;
    draw.tile_blend = false;
    draw.objective = false;
    if model {
        draw.cull = T6Cull::None;
        draw.foliage = false;
        if draw.blend == T6Blend::Sky {
            draw.blend = T6Blend::Opaque;
        }
    }
    draw
}

/// Materials into the catalog: their colour maps' pixels from the paks.
struct Materials<'a> {
    map: &'a Iw7Map,
    catalog: MaterialCatalog,
    owner: ZoneOwner,
    max: u32,
    /// (map material, as a model's) -> catalog material.
    linked: HashMap<(usize, bool), Option<usize>>,
    images: HashMap<String, Option<usize>>,
    decoded: usize,
    failures: Vec<String>,
}

impl Materials<'_> {
    fn image(&mut self, name: &str) -> Option<usize> {
        if let Some(slot) = self.images.get(name) {
            return *slot;
        }
        let slot = match self.map.image(name, self.max).and_then(|l| {
            let px = fastfile_iw6::image::ImagePixels {
                width: l.width,
                height: l.height,
                levels: l.levels,
                dxgi_format: l.format,
                data: l.data,
            };
            super::iw6::gpu_image(&px, true).map(|img| (px, img))
        }) {
            Ok((px, gpu)) => {
                self.decoded += 1;
                Some(self.catalog.link_image(AuthoredImage {
                    namespace: AssetNamespace::T6,
                    name: AssetRef::Real(format!("iw7:{name}#{}", self.max)),
                    map_type: 0,
                    semantic: TS_COLOR_MAP,
                    category: 0,
                    use_srgb_reads: true,
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
                if self.failures.len() < 64 {
                    self.failures.push(e);
                }
                None
            }
        };
        self.images.insert(name.to_owned(), slot);
        slot
    }

    fn material(&mut self, index: usize, model: bool) -> Option<usize> {
        if let Some(&m) = self.linked.get(&(index, model)) {
            return m;
        }
        let m = &self.map.materials[index];
        let draw = iw7_draw(&m.technique_set, m.sort_key, model);
        let colour = m.texture("colorMap").map(str::to_owned);
        let mut textures = Vec::new();
        if let Some(name) = colour {
            let hash = fastfile_iw7::texture::name_hash("colorMap");
            let sampler = m.textures.iter().find(|t| t.0 == hash).map_or(0x13, |t| t.1);
            if let Some(slot) = self.image(&name) {
                textures.push(MaterialTextureBinding {
                    name_hash: hash,
                    name_start: 0,
                    name_end: 0,
                    sampler_state: sampler,
                    semantic: TS_COLOR_MAP,
                    image: Some(slot),
                });
            }
        }
        let index_in_catalog = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(format!("iw7:{}{}", m.name, if model { "#m" } else { "" })),
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
        self.linked.insert((index, model), Some(index_in_catalog));
        Some(index_in_catalog)
    }
}

/// The map's sun: the worldspawn's `sundirection` (pitch yaw roll toward
/// the sun), `suncolor` and `sunlight`.
fn sun_of(entities: &[iw3::ents::Entity]) -> ([f32; 3], [f32; 3], f32) {
    let world = entities.iter().find(|e| e.classname() == "worldspawn");
    let vec3 = |k: &str| -> Option<[f32; 3]> {
        let v: Vec<f32> = world?.get(k)?.split_whitespace().filter_map(|x| x.parse().ok()).collect();
        (v.len() == 3).then(|| [v[0], v[1], v[2]])
    };
    let angles = vec3("sundirection").unwrap_or([-60.0, 45.0, 0.0]);
    let colour = vec3("suncolor").unwrap_or([1.0; 3]);
    // (IW7's `sunlight` is in its HDR units: about 7 outdoors.)
    let light = world.and_then(|e| e.get("sunlight")).and_then(|v| v.trim().parse::<f32>().ok()).map_or(1.0, |l| (l / 5.0).clamp(0.5, 2.0));
    (angles, colour, light)
}

/// The world as CoD4's `GfxWorld` (materials beside it, not in it).
fn iw3_world(map: &Iw7Map, sun: ([f32; 3], [f32; 3], f32)) -> iw3::zone::GfxWorld {
    iw3::zone::GfxWorld {
        name: map.name.clone(),
        base_name: map.name.clone(),
        indices: map.indices.clone(),
        vertices: map
            .vertices
            .iter()
            .map(|v| iw3::zone::WorldVertex {
                xyz: v.xyz,
                binormal_sign: v.binormal_sign,
                color: v.color,
                tex_coord: v.uv,
                lmap_coord: v.lmap_uv,
                normal: iw3_unit_vec(v.normal),
                tangent: iw3_unit_vec(v.tangent),
            })
            .collect(),
        surfaces: map
            .surfaces
            .iter()
            .map(|s| iw3::zone::GfxSurface {
                first_vertex: s.first_vertex as i32,
                vertex_count: s.vertex_count,
                tri_count: s.tri_count,
                base_index: s.base_index as i32,
                material: None,
                lightmap_index: 0,
                reflection_probe_index: 0,
                primary_light_index: 0,
                flags: s.flags,
                bounds: [s.mins, s.maxs],
            })
            .collect(),
        static_models: Vec::new(),
        sky_start_surfs: map.sky_surfaces.iter().map(|&s| s as i32).collect(),
        sky_image: None,
        sun: iw3::zone::SunParse {
            name: String::new(),
            ambient_scale: 0.0,
            ambient_color: [0.0; 3],
            diffuse_fraction: 0.0,
            sun_light: sun.2,
            sun_color: sun.1,
            diffuse_color: sun.1,
            angles: sun.0,
        },
        sun_flare: Default::default(),
        sun_color_from_bsp: sun.1,
        lightmaps: Vec::new(),
        reflection_probes: Vec::new(),
        mins: map.mins,
        maxs: map.maxs,
        // (No ranges: every surface not hidden is drawn.)
        lit_surfs: 0..0,
        decal_surfs: 0..0,
        emissive_surfs: 0..0,
        models: Vec::new(),
        light_grid: Default::default(),
    }
}

/// The spawn points in the map's entities (`mp_*_spawn*`, the Zombies
/// player starts).
fn spawns_of(entities: &[iw3::ents::Entity]) -> Vec<asset_world::SpawnPoint> {
    entities
        .iter()
        .filter(|e| {
            let c = e.classname();
            c.starts_with("mp_") && c.contains("spawn") || c == "info_player_start" || c.starts_with("cp_") && c.contains("spawn")
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

/// Loads an Infinite Warfare map zone into the match's world. A zone that
/// cannot be read becomes a gap with the reason, never a crash.
pub(crate) fn load_iw7_world(path: &Path, progress: &LoadProgress, material_seed: MaterialCatalog) -> LoadedWorld {
    let mut report = vec![format!("game: IW7 / Infinite Warfare ({})", path.display())];
    let stage = progress.begin_scoped(StageId::MapAssets, "infinite warfare zone", None);
    let started = std::time::Instant::now();
    let read = std::panic::catch_unwind(|| Iw7Map::load(&MapSources::beside(path)));
    let map = match read {
        Ok(Ok(map)) => {
            stage.done();
            map
        }
        Ok(Err(error)) => {
            stage.fail();
            diag::warn!(World, "bo2mp iw7: {}: {error}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Infinite Warfare zone: {error}"),
                Some("assets::lane::iw7::load_world/zone"),
            );
        }
        Err(panic) => {
            stage.fail();
            let what = panic
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_owned());
            diag::warn!(World, "bo2mp iw7: {}: reader panicked: {what}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Infinite Warfare zone could not be read: {what}"),
                Some("assets::lane::iw7::load_world/panic"),
            );
        }
    };
    report.push(format!("iw7 zone: read in {:.1}s", started.elapsed().as_secs_f32()));
    report.extend(map.report.iter().map(|l| format!("iw7 {l}")));
    let loaded = build_iw7_world(path, &map, report, progress, material_seed);
    diag::info!(
        World,
        "bo2mp iw7: {} loaded: {}",
        path.display(),
        loaded.report.iter().filter(|l| l.starts_with("iw7 ") || l.starts_with("world mesh")).cloned().collect::<Vec<_>>().join("; ")
    );
    loaded
}

/// The collision: the drawn world's triangles (not the sky's, tools',
/// decals' or blended surfaces') and each placed model's coarsest LOD.
///
/// Each brush model (`*1`, `*2`...: a door, a debris pile) gets a box brush
/// of its bounds for the entity that names it, and each trigger model
/// (`?1`...: a Zombies area, a buy trigger) its hulls. Also the triangles,
/// for the Zombies maps' walk graph.
fn collision_of(map: &Iw7Map, hidden: &[bool]) -> (asset_world::ClipCollision, Vec<asset_world::world_iw7::SoupTri>) {
    use asset_world::world_iw7::build_soup_clip_collision;
    let tris = collision_tris(map, hidden);
    let cmodels: Vec<asset_world::world_iw7::SoupCmodel> = map
        .brush_models
        .iter()
        .map(|b| {
            let mid: [f32; 3] = std::array::from_fn(|k| (b.mins[k] + b.maxs[k]) * 0.5);
            let half: [f32; 3] = std::array::from_fn(|k| (b.maxs[k] - b.mins[k]) * 0.5);
            (mid, half, (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt())
        })
        .collect();
    let mut clip = build_soup_clip_collision(&tris, &cmodels);
    clip.trigger_models = map
        .trigger_hulls
        .iter()
        .map(|hulls| hulls.iter().map(|&(mid, half)| asset_world::MapTriggerHull { mid, half, slabs: Vec::new() }).collect())
        .collect();
    (clip, tris)
}

fn collision_tris(map: &Iw7Map, hidden: &[bool]) -> Vec<asset_world::world_iw7::SoupTri> {
    use asset_world::world_iw7::{IW7_SOLID, SoupTri};
    let mut tris = Vec::new();
    for (i, s) in map.surfaces.iter().enumerate() {
        if hidden.get(i).copied().unwrap_or(false) {
            continue;
        }
        if let Some(m) = s.material {
            let draw = iw7_draw(&map.materials[m].technique_set, map.materials[m].sort_key, false);
            if draw.blend != T6Blend::Opaque {
                continue;
            }
        }
        for t in 0..s.tri_count as usize {
            let v = |k: usize| {
                let ix = map.indices.get(s.base_index as usize + 3 * t + k).copied().unwrap_or(0);
                map.vertices.get((s.first_vertex + u32::from(ix)) as usize).map_or([0.0; 3], |v| v.xyz)
            };
            tris.push(SoupTri { p: [v(0), v(1), v(2)], surface_flags: 0, contents: IW7_SOLID });
        }
    }
    for sm in &map.static_models {
        let m = &map.models[sm.model];
        if m.radius * sm.scale < COLLIDE_MIN_RADIUS {
            continue;
        }
        let surfaces = if m.coarse.is_empty() { &m.surfaces } else { &m.coarse };
        let place = |v: &[u8; 32]| -> [f32; 3] {
            let p = [0, 4, 8].map(|o| f32::from_le_bytes([v[o], v[o + 1], v[o + 2], v[o + 3]]));
            std::array::from_fn(|k| sm.origin[k] + (p[0] * sm.axis[0][k] + p[1] * sm.axis[1][k] + p[2] * sm.axis[2][k]) * sm.scale)
        };
        for s in surfaces {
            for t in &s.tris {
                let (Some(a), Some(b), Some(c)) = (s.verts.get(usize::from(t[0])), s.verts.get(usize::from(t[1])), s.verts.get(usize::from(t[2]))) else {
                    continue;
                };
                tris.push(SoupTri { p: [place(a), place(b), place(c)], surface_flags: 0, contents: IW7_SOLID });
            }
        }
    }
    tris
}

fn build_iw7_world(
    path: &Path,
    map: &Iw7Map,
    mut report: Vec<String>,
    progress: &LoadProgress,
    material_seed: MaterialCatalog,
) -> LoadedWorld {
    super::bo2_look::set_pending_grade(None);
    let map_name = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());

    // Surfaces not drawn: the sky's (the sky pass draws behind them), tool,
    // shadow-only, distortion and 3D-hud surfaces, by their technique set.
    // (Not `sky_surfaces`: those indices name ordinary surfaces in the
    // drawn order: mp_flip's is a perforated metal deck, which went
    // missing, from the collision too.)
    let mut hidden = vec![false; map.surfaces.len()];
    let mut hidden_kinds: HashMap<&'static str, usize> = HashMap::new();
    for (i, s) in map.surfaces.iter().enumerate() {
        if let Some(kind) = s.material.and_then(|m| hidden_kind(&map.materials[m].technique_set)) {
            hidden[i] = true;
            *hidden_kinds.entry(kind).or_default() += 1;
        }
    }

    // Collision.
    let stage = progress.begin_scoped(StageId::MapAssets, "infinite warfare collision", None);
    let (collision, soup) = collision_of(map, &hidden);
    let census = asset_world::world_iw3::spawn_ground_census(&collision, &map.spawns.iter().map(|s| s.origin).collect::<Vec<_>>());
    report.push(format!(
        "iw7 clip: {} triangles ({} vertices, {} partitions) from the drawn world and placed models; {census}",
        collision.mesh.tri_count(),
        collision.mesh.verts.len(),
        collision.mesh.partitions.len()
    ));
    if census.spawns > 0 && census.on_ground * 2 < census.spawns {
        diag::warn!(World, "bo2mp iw7: {map_name}: most spawn points have no floor of their own in the collision ({census})");
    }
    stage.done();
    // bo2mp: a Zombies map: its layout and a walk graph for Black Ops II's
    // Zombies (`super::iw7_zombies`, attached under its rules).
    if super::iw7_zombies::is_iw7_zombies_map(&map_name) {
        let stage = progress.begin_scoped(StageId::MapAssets, "infinite warfare zombies layout", None);
        report.extend(super::iw7_zombies::prepare(&map_name, map, &soup));
        stage.done();
    }
    drop(soup);

    // Entities and spawns.
    let entities = iw3::ents::parse(&map.entities);
    let spawns = spawns_of(&entities);
    report.push(format!("iw7 entities: {}, spawn points {}", entities.len(), spawns.len()));
    let mut scripts = crate::ScriptSources::default();
    scripts.set_entities(map.entities.clone());
    let mut facts = crate::MapFacts::default();
    let corners: Vec<[f32; 2]> = entities
        .iter()
        .filter(|e| e.get("targetname") == Some("minimap_corner"))
        .filter_map(|e| e.origin().map(|o| [o[0], o[1]]))
        .collect();
    if let [a, b] = corners[..] {
        facts.minimap_corners = Some(asset_world::MinimapCorners { a, b });
    }
    let compass_name = format!("compass_map_{}", map_name.to_ascii_lowercase());
    if map.image_info(&compass_name).is_some() {
        facts.compass.material = Some(compass_name.clone());
        facts.compass.image = Some(compass_name);
    }
    report.push(format!(
        "iw7 compass: corners {}, picture {}",
        if facts.minimap_corners.is_some() { "found" } else { "missing" },
        facts.compass.image.as_deref().unwrap_or("missing")
    ));

    // Materials: world surfaces, then the placed models'.
    let stage = progress.begin_scoped(StageId::MapAssets, "infinite warfare materials", None);
    let mut catalog = material_seed;
    catalog.set_capture_ns(AssetNamespace::T6);
    let mut b = Materials {
        map,
        catalog,
        owner: ZoneOwner::from_zone_path(path),
        max: texture_max(),
        linked: HashMap::new(),
        images: HashMap::new(),
        decoded: 0,
        failures: Vec::new(),
    };
    let surface_materials: Vec<Option<usize>> = map
        .surfaces
        .iter()
        .enumerate()
        .map(|(i, s)| if hidden[i] { None } else { s.material.and_then(|m| b.material(m, false)) })
        .collect();

    // bo2mp: the sky: the sky material's cube (`sky_<map>`) for the sky
    // pass, which draws it behind the world.
    // (A space map's, mp_proto's, is `sky_lighting_ref_*`: a daylight cube
    // for its lighting, not what it shows: its sky is its own geometry.)
    let sky = map
        .materials
        .iter()
        .filter(|m| hidden_kind(&m.technique_set) == Some("sky"))
        .filter_map(|m| m.texture("colorMap"))
        .find(|name| !name.contains("lighting_ref"));
    report.push(match sky {
        Some(name) => match map.image(name, 4096).and_then(|l| {
            let px = fastfile_iw6::image::ImagePixels { width: l.width, height: l.height, levels: l.levels, dxgi_format: l.format, data: l.data };
            super::iw6::link_sky(&mut b.catalog, &format!("iw7:sky:{name}"), &px, b.owner)
        }) {
            Ok(_) => format!("iw7 sky: {name} (cube)"),
            Err(e) => format!("iw7 sky: {name}: {e}"),
        },
        None => "iw7 sky: no sky material".to_owned(),
    });

    // The placed models.
    let (prop_meshes, prop_placements) = {
        use bevy::prelude::{Mat3, Quat, Transform, Vec3};
        let mut meshes: Vec<asset_world::ModelMesh> = Vec::new();
        let mut built: HashMap<usize, Option<usize>> = HashMap::new();
        let mut placements = Vec::with_capacity(map.static_models.len());
        for sm in &map.static_models {
            let entry = *built.entry(sm.model).or_insert_with(|| {
                let m = &map.models[sm.model];
                let key = |i: usize| asset_t6::AssetKey { ty: fastfile_t6::AssetType::Material, index: i };
                let lod0: Vec<asset_t6::XSurfaceRef> = m
                    .surfaces
                    .iter()
                    .filter(|s| !s.verts.is_empty() && s.verts.len() <= usize::from(u16::MAX))
                    .map(|s| asset_t6::XSurfaceRef {
                        verts: s.verts.iter().flat_map(iw7_vertex_to_t6).collect(),
                        vert_count: s.verts.len() as u16,
                        indices: s.tris.iter().flatten().copied().collect(),
                        vert_lists: Vec::new(),
                        flags: 0,
                        material: s.material.map(key),
                        blend_counts: [0; 4],
                        blend: Vec::new(),
                        part_bits: [0; 6],
                    })
                    .collect();
                let xref = asset_t6::XModelRef {
                    lighting_origin_offset: [0.0; 3],
                    lighting_origin_range: 0.0,
                    name: m.name.clone(),
                    numsurfs: lod0.len().min(255) as u8,
                    materials: lod0.iter().map(|s| s.material).collect(),
                    num_bones: 1,
                    num_lods: 1,
                    mins: m.mins,
                    maxs: m.maxs,
                    radius: m.radius,
                    lod0,
                    bone_names: vec!["tag_origin".to_owned()],
                    num_root_bones: 1,
                    parent_list: Vec::new(),
                    quats: Vec::new(),
                    trans: Vec::new(),
                    base_mat: vec![([0.0, 0.0, 0.0, 1.0], [0.0; 3])],
                    part_classification: vec![0],
                    bone_info: vec![None],
                    lods: vec![(0.0, 0, 0)],
                    contents: 1,
                    coll_lod: -1,
                    coll_surfs: Vec::new(),
                    phys_preset: None,
                };
                let linked: HashMap<usize, usize> = m
                    .surfaces
                    .iter()
                    .filter_map(|s| s.material)
                    .filter_map(|k| Some((k, b.material(k, true)?)))
                    .collect();
                let mesh = asset_world::build_t6_model_mesh(&xref, |k| linked.get(&k.index).copied())?;
                meshes.push(mesh);
                Some(meshes.len() - 1)
            });
            let Some(mesh) = entry else {
                placements.push(None);
                continue;
            };
            let basis = Mat3::from_cols(Vec3::from_array(sm.axis[0]), Vec3::from_array(sm.axis[1]), Vec3::from_array(sm.axis[2]));
            placements.push(Some(asset_world::StaticModelPlacement {
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
                flags: 0,
                // No light grid read: a neutral light (sqrt(1 / 32)).
                packed_lighting: Some([45, 45, 45, 0]),
                t6_primary: [0, 0],
            }));
        }
        report.push(format!(
            "iw7 static models: {} placed of {}, {} distinct models built",
            placements.iter().filter(|p| p.is_some()).count(),
            map.static_models.len(),
            meshes.len()
        ));
        (meshes, placements)
    };
    report.push(format!(
        "iw7 materials: {} linked, {} colour maps decoded (at most {}px), {} failed{}; not drawn: {hidden_kinds:?}",
        b.linked.len(),
        b.decoded,
        b.max,
        b.failures.len(),
        if b.failures.is_empty() { String::new() } else { format!(" (first: {})", b.failures.iter().take(4).cloned().collect::<Vec<_>>().join("; ")) },
    ));
    let mut materials = std::mem::take(&mut b.catalog);
    drop(b);
    stage.done();

    let sun = sun_of(&entities);
    let iw3w = iw3_world(map, sun);
    let (pitch, yaw) = (sun.0[0].to_radians(), sun.0[1].to_radians());
    let dir = [pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), -pitch.sin()];
    let colour = [sun.1[0] * sun.2, sun.1[1] * sun.2, sun.1[2] * sun.2, 1.0];
    report.push(format!("iw7 sun: toward {dir:?} colour {colour:?}"));
    let lighting = asset_world::T6WorldLighting {
        sun: asset_world::WorldSunLight { direction: dir, diffuse: colour, specular: colour },
        exposure_scale: 1.0,
        sky: [0.0, 1.0, 0.0, 1.0],
        sky_parms: [1.0, 0.0],
        saturation: None,
        grade: None,
    };
    // (IW7's lightmaps are GfxLightMap pages the fallback draw cannot read
    // yet: the world draws by the sun and its colour maps.)
    match asset_world::world_iw3::build_iw3_world_draw(&iw3w, &surface_materials, &hidden, Vec::new(), Some(lighting)) {
        Ok(draw) => {
            report.push(format!(
                "world mesh: {} vertices, {} triangles, {} surfaces (Infinite Warfare, fallback draw with colour maps, no lightmaps)",
                draw.stats.vertices, draw.stats.triangles, draw.stats.surfaces
            ));
            let (min, max) = (draw.stats.min, draw.stats.max);
            materials.set_capture_ns(AssetNamespace::T6);
            LoadedWorld {
                scripts,
                world: PreparedWorld {
                    draw: Some(draw),
                    min,
                    max,
                    policy: WorldDrawPolicy::t5(),
                    static_model_meshes: prop_meshes,
                    static_model_instances: prop_placements,
                    ..Default::default()
                },
                materials,
                collision: Some(collision),
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
                format!("Infinite Warfare world mesh: {error}"),
                Some("assets::lane::iw7::load_world/world_mesh"),
            );
            loaded.scripts = scripts;
            loaded.collision = Some(collision);
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
    fn unit_vectors_pack_for_bo2() {
        let v = [0.6, 0.0, -0.8];
        let back = asset_model::unpack_unit_vec(super::iw3_unit_vec(v));
        for k in 0..3 {
            assert!((back[k] - v[k]).abs() < 0.01);
        }
        let t6 = super::t6_unit_vec(v);
        let x = ((t6 & 0x3ff) as i32) << 22 >> 22;
        assert_eq!(x, 307);
    }
}
