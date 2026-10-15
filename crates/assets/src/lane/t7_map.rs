//! bo2mp: Call of Duty: Black Ops III (T7) maps, multiplayer and Zombies.
//!
//! A Black Ops III map zone (`TAff0000`, version 0x251) is read whole by
//! `fastfile_t7::map` (the zone's exact walk; placed models' meshes and
//! pictures from the zone or the `.xpak` packages beside it); this turns
//! it into the match's world the way the Infinite Warfare lane does
//! (`super::iw7`): the world goes to the IW3 world builder (Black Ops III's
//! world vertex is CoD4's layout but for its unit vectors and its UVs as
//! floats), the placed models through Black Ops II's model builder (their
//! surfaces packed as Black Ops II's vertices), materials onto BO2's draw
//! path with their colour maps.
//!
//! Collision is the zone's own (`asset_world::world_t7`): brushes under the
//! BSP and the triangle mesh, plus the coarsest LOD of each static model
//! the clip map lists (Black Ops III collides with those through Havok
//! shapes). The entity string is plain text with Black Ops II's class
//! names (`mp_tdm_spawn`, ...), so Black Ops II's rules read it as a
//! Treyarch map (`attach_t6_rules`).
//!
//! No ZoneGame of its own: the map walk sends a zone of Black Ops III's
//! version here (`asset_transport::legacy_game_for_path` says `t7`).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use asset_core::{AssetEdge, AssetNamespace, AssetRef, T6Blend, T6Cull, T6Draw, ZoneOwner};
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP};
use asset_transport::progress::{LoadProgress, StageId};
use asset_world::WorldDrawPolicy;
use fastfile_t7::map::{MapSources, T7Map};

use super::LoadedWorld;
use crate::lane_capability::PreparedCapability;
use crate::session_load::PreparedWorld;

/// The largest level kept of a surface's or model's colour map
/// (`BO2MP_T7_MAP_TEXTURE=<size>` changes it).
const TEXTURE_MAX: u32 = 512;

/// A placed model smaller than this (its radius, scaled) does not collide.
const COLLIDE_MIN_RADIUS: f32 = 16.0;

fn texture_max() -> u32 {
    std::env::var("BO2MP_T7_MAP_TEXTURE")
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
pub(super) fn t6_unit_vec(v: [f32; 3]) -> u32 {
    let mut out = 0u32;
    for (k, c) in v.iter().enumerate() {
        let s = (c.clamp(-1.0, 1.0) * 511.0).round() as i32;
        out |= ((s as u32) & 0x3ff) << (10 * k);
    }
    out
}

/// IEEE half from f32 (round to nearest; UVs only, so no NaN care).
pub(super) fn f16_bits(x: f32) -> u16 {
    let b = x.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let man = b & 0x7f_ffff;
    if exp >= 31 {
        return sign | 0x7bff;
    }
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        let m = (man | 0x80_0000) >> (1 - exp);
        return sign | ((m + 0x1000) >> 13) as u16;
    }
    let h = (sign as u32) | ((exp as u32) << 10) | (man >> 13);
    (h + ((man >> 12) & 1)) as u16
}

/// A tangent for a normal (any vector across it).
fn tangent_of(n: [f32; 3]) -> [f32; 3] {
    let a = if n[2].abs() < 0.9 { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] };
    let t = [a[1] * n[2] - a[2] * n[1], a[2] * n[0] - a[0] * n[2], a[0] * n[1] - a[1] * n[0]];
    let l = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().max(1e-6);
    t.map(|x| x / l)
}

/// A model vertex as Black Ops II's packed one (32 bytes: position,
/// binormal sign, colour, UV halves, normal, tangent).
fn t6_vertex(p: [f32; 3], n: [f32; 3], uv: [f32; 2]) -> [u8; 32] {
    let mut v = [0u8; 32];
    for k in 0..3 {
        v[4 * k..4 * k + 4].copy_from_slice(&p[k].to_le_bytes());
    }
    v[12..16].copy_from_slice(&1.0f32.to_le_bytes());
    v[16..20].copy_from_slice(&[255, 255, 255, 255]);
    let tex = u32::from(f16_bits(uv[0])) | u32::from(f16_bits(uv[1])) << 16;
    v[20..24].copy_from_slice(&tex.to_le_bytes());
    v[24..28].copy_from_slice(&t6_unit_vec(n).to_le_bytes());
    v[28..32].copy_from_slice(&t6_unit_vec(tangent_of(n)).to_le_bytes());
    v
}

/// What a material is not drawn as, by the words of its name and its
/// technique set's.
fn hidden_kind(material: &str, technique_set: &str) -> Option<&'static str> {
    let words: Vec<&str> = material.split(['_', '/', '#']).chain(technique_set.split(['_', '/', '#'])).collect();
    if words.iter().any(|w| w.starts_with("sky")) {
        Some("sky")
    } else if words.iter().any(|w| matches!(*w, "tools" | "tool" | "caulk" | "clip" | "portal" | "trigger" | "volume" | "hint" | "occluder")) {
        Some("tools")
    } else if words.iter().any(|w| matches!(*w, "shadowcaster" | "onlycast")) {
        Some("shadow only")
    } else if words.contains(&"distortion") {
        Some("distortion")
    } else {
        None
    }
}

/// A Black Ops III material as Black Ops II's draw: by its technique set's
/// and its own name (Black Ops III's technique names are not Black Ops
/// II's: blended and alpha-tested ones are told by their words), single
/// layered, drawn double-sided for models.
pub(super) fn t7_draw(material: &str, technique_set: &str, model: bool) -> T6Draw {
    let mut draw = T6Draw::from_technique_set(technique_set, 0, None);
    draw.layered = false;
    draw.layers = [0, 0];
    draw.water = false;
    draw.flow = false;
    draw.tile = false;
    draw.flag = false;
    draw.tile_blend = false;
    draw.objective = false;
    // (Black Ops III's technique sets read `mc/lit_decal#<hash>`,
    // `mc/lit_micro_tile_blend`, `mc/vegetation`, `mc/glass`, ...)
    let words: Vec<&str> = material.split(['_', '/', '#']).chain(technique_set.split(['_', '/', '#'])).collect();
    let has = |w: &str| words.contains(&w);
    let tiled = has("tile") || has("tiled");
    if has("decal") || has("dcl") || has("transparent") || has("water") || (has("glass") && !has("opaque")) || (has("blend") && !tiled) {
        draw.blend = T6Blend::Blend;
        if has("decal") || has("dcl") {
            draw.polygon_offset = draw.polygon_offset.max(1);
        }
    } else if has("alphatest") || has("vegetation") || has("foliage") || has("leaf") || has("leaves") || has("grass") || has("chainlink") {
        draw.alpha_test = true;
    }
    if model {
        draw.cull = T6Cull::None;
        draw.foliage = false;
        if draw.blend == T6Blend::Sky {
            draw.blend = T6Blend::Opaque;
        }
    }
    draw
}

/// Is a picture's format sRGB (a colour map)?
fn is_srgb(format: u32) -> bool {
    matches!(format, 29 | 72 | 75 | 78 | 91 | 93 | 99)
}

/// The colour map of a material: its first sRGB picture, else the first
/// whose name reads as a colour map (`_c`, `_col`), else the first.
fn colour_map(map: &T7Map, textures: &[(u32, String)]) -> Option<(u32, String)> {
    let srgb = textures.iter().find(|(_, n)| map.image_info(n).is_some_and(|i| is_srgb(i.format)));
    let named = || {
        textures.iter().find(|(_, n)| {
            let n = n.to_ascii_lowercase();
            n.ends_with("_c") || n.ends_with("_col") || n.contains("_c_") || n.ends_with("_color")
        })
    };
    srgb.or_else(named).or_else(|| textures.first()).cloned()
}

/// Materials into the catalog: their colour maps' pixels from the zone or
/// the packages.
struct Materials<'a> {
    map: &'a T7Map,
    catalog: MaterialCatalog,
    owner: ZoneOwner,
    max: u32,
    /// (map material, as a model's) -> catalog material.
    linked: HashMap<(usize, bool), Option<usize>>,
    images: HashMap<String, Option<usize>>,
    decoded: usize,
    failures: Vec<String>,
    /// bo2mp: Black Ops III's own look (`super::t7_look`), when on.
    looks: Option<super::t7_look::Looks>,
}

impl Materials<'_> {
    fn image(&mut self, name: &str) -> Option<usize> {
        if let Some(slot) = self.images.get(name) {
            return *slot;
        }
        let slot = match self.map.image(name, self.max).and_then(|l| {
            let px = fastfile_iw6::image::ImagePixels { width: l.width, height: l.height, levels: l.levels, dxgi_format: l.format, data: l.data };
            super::iw6::gpu_image(&px, true).map(|img| (px, img))
        }) {
            Ok((px, gpu)) => {
                self.decoded += 1;
                Some(self.catalog.link_image(AuthoredImage {
                    namespace: AssetNamespace::T6,
                    name: AssetRef::Real(format!("t7:{name}#{}", self.max)),
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
        let mut draw = t7_draw(&m.name, &m.technique_set, model);
        let mut textures = Vec::new();
        // bo2mp: an emissive surface drawn unlit from its emissive map
        // (`super::t7_look::emissive`).
        let glow = if self.looks.is_some() { super::t7_look::emissive(self.map, index) } else { None };
        let glow_slot = glow.as_ref().and_then(|(name, _)| self.image(name));
        if let (Some(slot), Some((_, scale))) = (glow_slot, glow) {
            draw.unlit = true;
            draw.unlit_scale_exp = scale;
            if let Some(looks) = self.looks.as_mut() {
                looks.stats.emissive += 1;
            }
            textures.push(MaterialTextureBinding { name_hash: 0xa0ab_1041, name_start: 0, name_end: 0, sampler_state: 0x13, semantic: TS_COLOR_MAP, image: Some(slot) });
        }
        // bo2mp: Black Ops III's look (`super::t7_look`): its colour map as
        // its G-buffer makes it, its normal and specular maps.
        let look = match (self.looks.as_mut(), super::t7_look::Looks::cover(&draw)) {
            (Some(looks), Some(cover)) => looks.link(self.map, &mut self.catalog, index, cover),
            _ => [None; 3],
        };
        let m = &self.map.materials[index];
        let glowing = !textures.is_empty();
        if let Some(slot) = look[0].filter(|_| !glowing) {
            textures.push(MaterialTextureBinding { name_hash: 0xa0ab_1041, name_start: 0, name_end: 0, sampler_state: 0x13, semantic: TS_COLOR_MAP, image: Some(slot) });
        } else if !glowing
            && let Some((hash, name)) = colour_map(self.map, &m.textures)
            && let Some(slot) = self.image(&name)
        {
            textures.push(MaterialTextureBinding { name_hash: hash, name_start: 0, name_end: 0, sampler_state: 0x13, semantic: TS_COLOR_MAP, image: Some(slot) });
        }
        textures.extend(super::t7_look::Looks::bindings(&look));
        let m = &self.map.materials[index];
        let index_in_catalog = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(format!("t7:{}{}", m.name, if model { "#m" } else { "" })),
            namespace: AssetNamespace::T6,
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
        self.linked.insert((index, model), Some(index_in_catalog));
        Some(index_in_catalog)
    }
}

/// The sun: the zone's sun volume (pitch and yaw toward the sun, colour,
/// intensity), else the worldspawn's `sundirection`/`suncolor`.
fn sun_of(map: &T7Map, entities: &[iw3::ents::Entity]) -> ([f32; 3], [f32; 3], f32) {
    if let Some(s) = map.sun
        && s.pitch.is_finite()
        && s.yaw.is_finite()
        && s.color.iter().all(|c| c.is_finite() && *c >= 0.0)
        && s.color.iter().any(|c| *c > 0.0)
    {
        let peak = s.color.iter().copied().fold(0.0f32, f32::max).max(1e-3);
        let colour = s.color.map(|c| (c / peak).clamp(0.0, 1.0));
        // (Black Ops III's sun intensity is in its HDR units.)
        let light = if s.intensity.is_finite() && s.intensity > 0.0 { (s.intensity / 5.0).clamp(0.6, 2.0) } else { 1.0 };
        return ([-s.pitch.abs(), s.yaw, 0.0], colour, light);
    }
    let world = entities.iter().find(|e| e.classname() == "worldspawn");
    let vec3 = |k: &str| -> Option<[f32; 3]> {
        let v: Vec<f32> = world?.get(k)?.split_whitespace().filter_map(|x| x.parse().ok()).collect();
        (v.len() == 3).then(|| [v[0], v[1], v[2]])
    };
    (vec3("sundirection").unwrap_or([-60.0, 45.0, 0.0]), vec3("suncolor").unwrap_or([1.0; 3]), 1.0)
}

/// The world as CoD4's `GfxWorld` (materials beside it, not in it).
fn iw3_world(map: &T7Map, sun: ([f32; 3], [f32; 3], f32)) -> iw3::zone::GfxWorld {
    iw3::zone::GfxWorld {
        name: map.name.clone(),
        base_name: map.name.clone(),
        indices: map.indices.clone(),
        vertices: map
            .vertices
            .iter()
            .map(|v| iw3::zone::WorldVertex {
                xyz: v.xyz,
                binormal_sign: 1.0,
                color: v.color,
                tex_coord: v.uv,
                lmap_coord: [0.0, 0.0],
                normal: iw3_unit_vec(v.normal),
                tangent: iw3_unit_vec(v.tangent),
            })
            .collect(),
        surfaces: map
            .surfaces
            .iter()
            .map(|s| iw3::zone::GfxSurface {
                first_vertex: s.first_vertex as i32,
                vertex_count: s.vertex_count.min(u32::from(u16::MAX)) as u16,
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
        sky_start_surfs: Vec::new(),
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
        lit_surfs: 0..0,
        decal_surfs: 0..0,
        emissive_surfs: 0..0,
        models: Vec::new(),
        light_grid: Default::default(),
        dpvs: None,
    }
}

/// The spawn points in the map's entities (`mp_*_spawn*`, the player
/// starts, Zombies' `initial_spawn_points` structs).
fn spawns_of(entities: &[iw3::ents::Entity]) -> Vec<asset_world::SpawnPoint> {
    entities
        .iter()
        .filter(|e| {
            let c = e.classname();
            c.starts_with("mp_") && c.contains("spawn")
                || c == "info_player_start"
                || e.get("targetname") == Some("initial_spawn_points")
                || e.get("targetname") == Some("player_respawn_point")
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

/// Loads a Black Ops III map zone into the match's world. A zone that
/// cannot be read becomes a gap with the reason, never a crash.
pub(crate) fn load_t7_world(path: &Path, progress: &LoadProgress, material_seed: MaterialCatalog) -> LoadedWorld {
    let mut report = vec![format!("game: T7 / Black Ops III ({})", path.display())];
    let stage = progress.begin_scoped(StageId::MapAssets, "black ops iii zone", None);
    let started = std::time::Instant::now();
    let read = std::panic::catch_unwind(|| T7Map::load(&MapSources::beside(path)));
    let map = match read {
        Ok(Ok(map)) => {
            stage.done();
            map
        }
        Ok(Err(error)) => {
            stage.fail();
            diag::warn!(World, "bo2mp t7: {}: {error}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Black Ops III zone: {error}"),
                Some("assets::lane::t7_map::load_world/zone"),
            );
        }
        Err(panic) => {
            stage.fail();
            let what = panic
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_owned());
            diag::warn!(World, "bo2mp t7: {}: reader panicked: {what}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Black Ops III zone could not be read: {what}"),
                Some("assets::lane::t7_map::load_world/panic"),
            );
        }
    };
    report.push(format!("t7 zone: read in {:.1}s", started.elapsed().as_secs_f32()));
    report.extend(map.report.iter().map(|l| format!("t7 {l}")));
    let loaded = build_t7_world(path, &map, report, progress, material_seed);
    diag::info!(
        World,
        "bo2mp t7: {} loaded: {}",
        path.display(),
        loaded.report.iter().filter(|l| l.starts_with("t7 ") || l.starts_with("world mesh")).cloned().collect::<Vec<_>>().join("; ")
    );
    loaded
}

/// A model's bounds and radius about its origin.
fn model_bounds(m: &fastfile_t7::map::MapModel) -> ([f32; 3], [f32; 3], f32) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    let mut r2 = 0.0f32;
    for s in &m.surfaces {
        for p in &s.positions {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
            r2 = r2.max(p[0] * p[0] + p[1] * p[1] + p[2] * p[2]);
        }
    }
    if lo[0] > hi[0] {
        return ([0.0; 3], [0.0; 3], 0.0);
    }
    (lo, hi, r2.sqrt())
}

/// The collision: the zone's clip map as Black Ops II's tables, with the
/// listed static models' coarsest LODs as extra triangles. `None` when the
/// zone has no clip map (the drawn world stands in then).
fn collision_of(map: &T7Map, hidden: &[bool], report: &mut Vec<String>) -> asset_world::ClipCollision {
    use asset_world::world_iw7::{IW7_SOLID, SoupTri};
    use asset_world::world_t7::{T7AabbNode, T7Brush, T7ClipInput, T7Cmodel, build_t7_clip_collision};
    let by_name: HashMap<&str, usize> = map.models.iter().enumerate().map(|(i, m)| (m.name.as_str(), i)).collect();
    let mut extra = Vec::new();
    let place = |p: [f32; 3], origin: [f32; 3], axis: [[f32; 3]; 3], scale: f32| -> [f32; 3] {
        std::array::from_fn(|k| origin[k] + (p[0] * axis[0][k] + p[1] * axis[1][k] + p[2] * axis[2][k]) * scale)
    };
    let mut missing = 0usize;
    let mut models_used = 0usize;
    let add_model = |m: &fastfile_t7::map::MapModel, origin: [f32; 3], axis: [[f32; 3]; 3], scale: f32, extra: &mut Vec<SoupTri>| {
        let surfaces = if m.coarse.is_empty() { &m.surfaces } else { &m.coarse };
        for s in surfaces {
            for t in &s.tris {
                let (Some(a), Some(b), Some(c)) = (s.positions.get(usize::from(t[0])), s.positions.get(usize::from(t[1])), s.positions.get(usize::from(t[2]))) else {
                    continue;
                };
                extra.push(SoupTri { p: [place(*a, origin, axis, scale), place(*b, origin, axis, scale), place(*c, origin, axis, scale)], surface_flags: 0, contents: IW7_SOLID });
            }
        }
    };
    let Some(clip) = &map.clip else {
        // No clip map read: the drawn world and every placed model.
        for (i, s) in map.surfaces.iter().enumerate() {
            if hidden.get(i).copied().unwrap_or(false) {
                continue;
            }
            for t in 0..s.tri_count as usize {
                let v = |k: usize| {
                    let ix = map.indices.get(s.base_index as usize + 3 * t + k).copied().unwrap_or(0);
                    map.vertices.get((s.first_vertex + u32::from(ix)) as usize).map_or([0.0; 3], |v| v.xyz)
                };
                extra.push(SoupTri { p: [v(0), v(1), v(2)], surface_flags: 0, contents: IW7_SOLID });
            }
        }
        for sm in &map.static_models {
            let m = &map.models[sm.model];
            if model_bounds(m).2 * sm.scale >= COLLIDE_MIN_RADIUS {
                add_model(m, sm.origin, sm.axis, sm.scale, &mut extra);
            }
        }
        report.push(format!("t7 clip: none in the zone; {} triangles from the drawn world and placed models", extra.len()));
        return asset_world::world_iw7::build_soup_clip_collision(&extra, &[]);
    };
    for sm in &clip.static_models {
        match by_name.get(sm.name.as_str()) {
            Some(&i) if model_bounds(&map.models[i]).2 * sm.scale >= COLLIDE_MIN_RADIUS * 0.5 => {
                models_used += 1;
                add_model(&map.models[i], sm.origin, sm.axis, sm.scale, &mut extra);
            }
            Some(_) => {}
            None => missing += 1,
        }
    }
    let input = T7ClipInput {
        materials: clip.materials.clone(),
        brushes: clip.brushes.iter().map(|b| T7Brush { contents: b.contents, planes: b.planes.clone(), plane_surface_flags: b.plane_surface_flags.clone() }).collect(),
        nodes: clip.nodes.iter().map(|n| (n.plane, n.children)).collect(),
        leaf_brushes: clip.leafs.iter().map(|l| l.brushes.clone()).collect(),
        verts: clip.verts.clone(),
        tris: clip.tris.clone(),
        walkable: clip.walkable.clone(),
        partitions: clip.partitions.iter().map(|p| (p.tri_count, p.first_tri)).collect(),
        aabb_trees: clip
            .aabb_trees
            .iter()
            .map(|a| T7AabbNode { origin: a.origin, half_size: a.half_size, material: a.material, child_count: a.child_count, u: a.u })
            .collect(),
        cmodels: clip.models.iter().map(|m| T7Cmodel { mins: m.mins, maxs: m.maxs, radius: m.radius, brushes: m.leaf.brushes.clone() }).collect(),
    };
    let (collision, stats) = build_t7_clip_collision(&input, &extra);
    report.push(format!(
        "t7 clip: {} brushes, {} leaves, {} mesh triangles, {} model triangles from {} static models ({} not built), {} tree roots gathered, {} partitions out of range",
        stats.brushes, stats.leaves, stats.tris, stats.extra_tris, models_used, missing, stats.roots, stats.bad_partitions
    ));
    collision
}

/// Triangles to lay the Zombies walk graph over: the drawn world's opaque
/// surfaces and the clip map's triangle mesh.
fn walk_soup(map: &T7Map, hidden: &[bool]) -> Vec<asset_world::world_iw7::SoupTri> {
    use asset_world::world_iw7::{IW7_SOLID, SoupTri};
    let mut tris = Vec::new();
    for (i, s) in map.surfaces.iter().enumerate() {
        if hidden.get(i).copied().unwrap_or(false) {
            continue;
        }
        if let Some(m) = s.material
            && t7_draw(&map.materials[m].name, &map.materials[m].technique_set, false).blend != T6Blend::Opaque
        {
            continue;
        }
        for t in 0..s.tri_count as usize {
            let v = |k: usize| {
                let ix = map.indices.get(s.base_index as usize + 3 * t + k).copied().unwrap_or(0);
                map.vertices.get((s.first_vertex + u32::from(ix)) as usize).map_or([0.0; 3], |v| v.xyz)
            };
            tris.push(SoupTri { p: [v(0), v(1), v(2)], surface_flags: 0, contents: IW7_SOLID });
        }
    }
    if let Some(c) = &map.clip {
        for t in &c.tris {
            let v = |k: usize| c.verts.get(t[k] as usize).copied().unwrap_or([0.0; 3]);
            tris.push(SoupTri { p: [v(0), v(1), v(2)], surface_flags: 0, contents: IW7_SOLID });
        }
    }
    tris
}

fn build_t7_world(path: &Path, map: &T7Map, mut report: Vec<String>, progress: &LoadProgress, material_seed: MaterialCatalog) -> LoadedWorld {
    super::bo2_look::set_pending_grade(None);
    let map_name = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());

    // Surfaces not drawn: the sky's, tool, shadow-only and distortion
    // surfaces.
    let mut hidden = vec![false; map.surfaces.len()];
    let mut hidden_kinds: HashMap<&'static str, usize> = HashMap::new();
    for (i, s) in map.surfaces.iter().enumerate() {
        if let Some(kind) = s.material.and_then(|m| hidden_kind(&map.materials[m].name, &map.materials[m].technique_set)) {
            hidden[i] = true;
            *hidden_kinds.entry(kind).or_default() += 1;
        }
    }

    // Entities and spawns.
    let entities = iw3::ents::parse(&map.entities);
    let spawns = spawns_of(&entities);

    // Collision.
    let stage = progress.begin_scoped(StageId::MapAssets, "black ops iii collision", None);
    let mut collision = collision_of(map, &hidden, &mut report);
    collision.trigger_models = map
        .trigger_models
        .iter()
        .map(|hulls| hulls.iter().map(|&(mid, half)| asset_world::MapTriggerHull { mid, half, slabs: Vec::new() }).collect())
        .collect();
    // bo2mp: a Zombies map: its layout in the Infinite Warfare Zombies
    // pipeline's terms and a walk graph over the world (played as Black
    // Ops II Zombies under `attach_t6_zm_rules`).
    if map_name.starts_with("zm_") {
        let (layout, triggers, lines) = zm::read(map, collision.trigger_models.len());
        report.extend(lines);
        collision.trigger_models.extend(triggers);
        let soup = walk_soup(map, &hidden);
        super::iw7_zombies::register_foreign_zm_map(&map_name);
        report.extend(super::iw7_zombies::prepare_layout(&map_name, layout, &soup));
    }
    let census = asset_world::world_iw3::spawn_ground_census(&collision, &spawns.iter().map(|s| s.origin).collect::<Vec<_>>());
    report.push(format!("t7 spawns: {census}"));
    if census.spawns > 0 && census.on_ground * 2 < census.spawns {
        diag::warn!(World, "bo2mp t7: {map_name}: most spawn points have no floor of their own in the collision ({census})");
    }
    stage.done();

    report.push(format!("t7 entities: {}, spawn points {}", entities.len(), spawns.len()));
    let mut scripts = crate::ScriptSources::default();
    scripts.set_entities(map.entities.clone());
    let mut facts = crate::MapFacts::default();
    let corners: Vec<[f32; 2]> =
        entities.iter().filter(|e| e.get("targetname") == Some("minimap_corner")).filter_map(|e| e.origin().map(|o| [o[0], o[1]])).collect();
    if let [a, b] = corners[..] {
        facts.minimap_corners = Some(asset_world::MinimapCorners { a, b });
    }
    for compass_name in [format!("compass_map_{}", map_name.to_ascii_lowercase()), format!("minimap_{}", map_name.to_ascii_lowercase())] {
        if map.image_info(&compass_name).is_some() {
            facts.compass.material = Some(compass_name.clone());
            facts.compass.image = Some(compass_name);
            break;
        }
    }
    report.push(format!(
        "t7 compass: corners {}, picture {}",
        if facts.minimap_corners.is_some() { "found" } else { "missing" },
        facts.compass.image.as_deref().unwrap_or("missing")
    ));

    // Materials: world surfaces, then the placed models'.
    let stage = progress.begin_scoped(StageId::MapAssets, "black ops iii materials", None);
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
        looks: super::t7_look::on().then(|| super::t7_look::Looks::new(texture_max())),
    };
    let surface_materials: Vec<Option<usize>> =
        map.surfaces.iter().enumerate().map(|(i, s)| if hidden[i] { None } else { s.material.and_then(|m| b.material(m, false)) }).collect();

    // The placed models.
    let (prop_meshes, prop_placements) = {
        use bevy::prelude::{Mat3, Quat, Transform, Vec3};
        let mut meshes: Vec<asset_world::ModelMesh> = Vec::new();
        let mut built: HashMap<usize, Option<usize>> = HashMap::new();
        let mut placements = Vec::with_capacity(map.static_models.len());
        for sm in &map.static_models {
            let entry = *built.entry(sm.model).or_insert_with(|| {
                let m = &map.models[sm.model];
                let (mins, maxs, radius) = model_bounds(m);
                let key = |i: usize| asset_t6::AssetKey { ty: fastfile_t6::AssetType::Material, index: i };
                let lod0: Vec<asset_t6::XSurfaceRef> = m
                    .surfaces
                    .iter()
                    .filter(|s| !s.positions.is_empty() && s.positions.len() <= usize::from(u16::MAX))
                    .filter(|s| !s.material.is_some_and(|k| hidden_kind(&map.materials[k].name, &map.materials[k].technique_set).is_some()))
                    .map(|s| asset_t6::XSurfaceRef {
                        verts: (0..s.positions.len())
                            .flat_map(|i| t6_vertex(s.positions[i], s.normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]), s.uvs.get(i).copied().unwrap_or_default()))
                            .collect(),
                        vert_count: s.positions.len() as u16,
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
                    mins,
                    maxs,
                    radius,
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
                let linked: HashMap<usize, usize> =
                    m.surfaces.iter().filter_map(|s| s.material).filter_map(|k| Some((k, b.material(k, true)?))).collect();
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
                transform: Transform { translation: Vec3::from_array(sm.origin), rotation: Quat::from_mat3(&basis), scale: Vec3::splat(sm.scale) },
                origin: sm.origin,
                axis: sm.axis,
                scale: sm.scale,
                cull_dist: 0,
                reflection_probe_index: 0,
                primary_light_index: 0,
                flags: 0,
                // No light grid read: a neutral light (sqrt(1 / 32)).
                packed_lighting: Some([45, 45, 45, 0]),
                t6_primary: [0, 0],
            }));
        }
        report.push(format!(
            "t7 static models: {} placed of {}, {} distinct models built",
            placements.iter().filter(|p| p.is_some()).count(),
            map.static_models.len(),
            meshes.len()
        ));
        (meshes, placements)
    };
    report.push(format!(
        "t7 materials: {} linked, {} colour maps decoded (at most {}px), {} failed{}; not drawn: {hidden_kinds:?}",
        b.linked.len(),
        b.decoded,
        b.max,
        b.failures.len(),
        if b.failures.is_empty() { String::new() } else { format!(" (first: {})", b.failures.iter().take(4).cloned().collect::<Vec<_>>().join("; ")) },
    ));
    report.push(b.looks.as_ref().map_or_else(|| "t7 look: off (BO2MP_T7_MATERIALS=0): colour maps only".to_owned(), super::t7_look::Looks::report));
    let mut materials = std::mem::take(&mut b.catalog);
    drop(b);
    stage.done();

    let sun = sun_of(map, &entities);
    let iw3w = iw3_world(map, sun);
    let (pitch, yaw) = (sun.0[0].to_radians(), sun.0[1].to_radians());
    let dir = [pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), -pitch.sin()];
    let colour = [sun.1[0] * sun.2, sun.1[1] * sun.2, sun.1[2] * sun.2, 1.0];
    report.push(format!("t7 sun: toward {dir:?} colour {colour:?}; sky models {:?} (not drawn)", map.sky_models));
    let lighting = asset_world::T6WorldLighting {
        sun: asset_world::WorldSunLight { direction: dir, diffuse: colour, specular: colour },
        exposure_scale: 1.0,
        sky: [0.0, 1.0, 0.0, 1.0],
        sky_parms: [1.0, 0.0],
        saturation: None,
        grade: None,
    };
    match asset_world::world_iw3::build_iw3_world_draw(&iw3w, &surface_materials, &hidden, Vec::new(), Some(lighting)) {
        Ok(draw) => {
            report.push(format!(
                "world mesh: {} vertices, {} triangles, {} surfaces (Black Ops III, fallback draw with colour maps, no lightmaps)",
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
                format!("Black Ops III world mesh: {error}"),
                Some("assets::lane::t7_map::load_world/world_mesh"),
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
    fn halves_and_unit_vectors() {
        for x in [0.0f32, 1.0, -1.0, 0.5, 2.25, -3.75, 1024.0] {
            assert_eq!(asset_model::half_to_f32(super::f16_bits(x)), x);
        }
        let v = super::t6_unit_vec([0.6, 0.0, -0.8]);
        assert_eq!(((v & 0x3ff) as i32) << 22 >> 22, 307);
        let t = super::tangent_of([0.0, 0.0, 1.0]);
        assert!(t[2].abs() < 1e-6 && (t[0] * t[0] + t[1] * t[1] - 1.0).abs() < 1e-5);
    }
}

/// bo2mp: a Black Ops III Zombies map (`zm_*`) in the terms of the
/// Infinite Warfare Zombies pipeline (`super::iw7_zombies`), which writes
/// it out as Black Ops II Zombies on Buried's kit. Black Ops III keeps
/// Black Ops II's own vocabulary, so this is mostly a reading:
/// - zones: `info_volume` `player_volume`s (their brush model's box, a
///   trigger model of its own is added for it), their spawners the structs
///   their `target` names (`riser_location` climb out of the ground,
///   `dog_location`s are the dog spots);
/// - windows: `exterior_goal` structs, the boards where the zbarrier they
///   target stands;
/// - doors and debris: `zombie_door`/`zombie_debris` triggers (cost, the
///   zone their `script_flag` opens, the brushes their `target` names);
/// - perk machines by their `specialty_*` (as the pipeline's machine
///   names), Mystery Box spots (`treasure_chest_use`), Pack-a-Punch, the
///   power switch, wall buys (each weapon as one of its kind), traversals.
/// Not carried: teleporters, traps, the Gobblegum machines, the Easter egg.
pub(crate) mod zm {
    use std::collections::HashMap;

    use fastfile_iw7::zombies::{Area, Blocker, BlockerKind, Generator, Machine, Spawner, Spot, Traversal, WallBuy, Window, ZmLayout};
    use fastfile_t7::map::T7Map;

    type Ent = iw3::ents::Entity;

    fn spot(e: &Ent) -> Option<Spot> {
        Some(Spot { origin: e.origin()?, angles: e.angles() })
    }

    fn model_index(e: &Ent, prefix: char) -> Option<u32> {
        e.get("model")?.strip_prefix(prefix)?.parse().ok()
    }

    /// A Black Ops III perk as the pipeline's machine name.
    fn machine_name(specialty: &str) -> Option<&'static str> {
        Some(match specialty {
            "specialty_armorvest" => "perk_machine_tough",
            "specialty_quickrevive" => "perk_machine_revive",
            "specialty_fastreload" => "perk_machine_flash",
            "specialty_doubletap2" | "specialty_rof" => "perk_machine_rat_a_tat",
            "specialty_staminup" | "specialty_longersprint" => "perk_machine_run",
            "specialty_additionalprimaryweapon" => "perk_machine_more",
            _ => return None,
        })
    }

    /// A Black Ops III wall weapon as the pipeline's weapon of its kind.
    fn wall_weapon(t7: &str) -> Option<&'static str> {
        let kind = t7.split('_').next()?;
        Some(match (kind, t7) {
            (_, "pistol_standard" | "pistol_burst" | "pistol_fullauto") | ("pistol", _) => "iw7_g18_zm",
            ("shotgun", _) => "iw7_spas_zm",
            (_, "ar_standard" | "ar_marksman" | "ar_longburst") => "iw7_m4_zm",
            ("ar", _) => "iw7_ar57_zm",
            (_, "smg_standard" | "smg_versatile") => "iw7_crb_zm",
            ("smg", _) => "iw7_erad_zm",
            ("lmg", _) => "iw7_rvn_zm",
            ("sniper", _) => "iw7_kbs_zm",
            _ => return None,
        })
    }

    /// The layout, the trigger models added for its zones (appended to the
    /// clip's), and report lines.
    pub(crate) fn read(map: &T7Map, first_trigger: usize) -> (ZmLayout, Vec<Vec<asset_world::MapTriggerHull>>, Vec<String>) {
        let ents = iw3::ents::parse(&map.entities);
        let by_name: HashMap<&str, Vec<&Ent>> = ents.iter().filter_map(|e| Some((e.get("targetname")?, e))).fold(HashMap::new(), |mut m, (k, e)| {
            m.entry(k).or_default().push(e);
            m
        });
        let named = |n: &str| by_name.get(n).cloned().unwrap_or_default();
        let mut l = ZmLayout::default();
        let mut report = Vec::new();
        let mut triggers = Vec::new();
        let cmodel_box = |e: &Ent| -> Option<([f32; 3], [f32; 3])> {
            let m = map.clip.as_ref()?.models.get(model_index(e, '*')? as usize)?;
            let o = e.origin().unwrap_or([0.0; 3]);
            // (Sub-models' bounds are about the entity's origin.)
            let inside = (0..3).all(|k| o[k] >= m.mins[k] - 1.0 && o[k] <= m.maxs[k] + 1.0);
            Some(if inside { (m.mins, m.maxs) } else { (std::array::from_fn(|k| o[k] + m.mins[k]), std::array::from_fn(|k| o[k] + m.maxs[k])) })
        };
        l.starts = named("initial_spawn_points").iter().filter_map(|e| spot(e)).collect();
        l.respawns = named("player_respawn_point").iter().filter_map(|e| spot(e)).collect();
        // Zones.
        let mut areas: Vec<Area> = Vec::new();
        for v in ents.iter().filter(|e| e.classname() == "info_volume" && e.get("script_noteworthy") == Some("player_volume")) {
            let Some(name) = v.get("targetname") else { continue };
            let Some(b) = cmodel_box(v) else { continue };
            let area = match areas.iter().position(|a| a.name == name) {
                Some(i) => &mut areas[i],
                None => {
                    areas.push(Area { name: name.to_owned(), origin: v.origin().unwrap_or_default(), ..Default::default() });
                    let a = areas.last_mut().unwrap();
                    for s in v.get("target").map(named).unwrap_or_default() {
                        let Some(sp) = spot(s) else { continue };
                        match s.get("script_noteworthy") {
                            Some("dog_location") => l.dog_spots.push(sp),
                            Some(n) => a.spawners.push(Spawner { spot: sp, ground: n == "riser_location", ..Default::default() }),
                            None => a.spawners.push(Spawner { spot: sp, ..Default::default() }),
                        }
                    }
                    a
                }
            };
            area.boxes.push(b);
        }
        for a in &mut areas {
            let hulls: Vec<asset_world::MapTriggerHull> = a
                .boxes
                .iter()
                .map(|(lo, hi)| asset_world::MapTriggerHull {
                    mid: std::array::from_fn(|k| (lo[k] + hi[k]) * 0.5),
                    half: std::array::from_fn(|k| (hi[k] - lo[k]) * 0.5),
                    slabs: Vec::new(),
                })
                .collect();
            a.model = Some((first_trigger + triggers.len()) as u32);
            triggers.push(hulls);
        }
        l.areas = areas;
        // Windows: the outside goal, the boards where its zbarrier is.
        for g in named("exterior_goal") {
            let Some(outside) = spot(g) else { continue };
            let boards = g.get("target").map(named).unwrap_or_default().iter().find(|e| e.classname().starts_with("zbarrier")).and_then(|e| e.origin());
            l.windows.push(Window { outside, attack: Vec::new(), inside: None, boards });
        }
        // Doors and debris.
        let zone_of_flag = |flag: &str| -> Option<String> {
            let want = flag.strip_prefix("enter_")?;
            l.areas.iter().map(|a| a.name.as_str()).find(|n| n.trim_end_matches("_zone") == want || n.starts_with(want)).map(str::to_owned)
        };
        let mut groups: HashMap<String, Blocker> = HashMap::new();
        for t in named("zombie_door").into_iter().chain(named("zombie_debris")) {
            let group = t.get("target").unwrap_or_default().to_owned();
            let cost = t.get("zombie_cost").and_then(|c| c.parse().ok()).unwrap_or(1000);
            let b = groups.entry(group.clone()).or_insert_with(|| Blocker {
                kind: BlockerKind::Debris,
                cost,
                sides: Vec::new(),
                triggers: Vec::new(),
                brushes: named(&group).iter().filter_map(|e| Some((model_index(e, '*')?, e.origin().unwrap_or_default()))).collect(),
                group: group.clone(),
            });
            if let (Some(sp), Some(zone)) = (spot(t), t.get("script_flag").and_then(zone_of_flag)) {
                b.sides.push((sp, zone));
            }
            if let (Some(o), Some(m)) = (t.origin(), model_index(t, '?')) {
                b.triggers.push((o, m));
            }
        }
        let mut blockers: Vec<Blocker> = groups.into_values().collect();
        blockers.sort_by(|a, b| a.group.cmp(&b.group));
        let no_side = blockers.iter().filter(|b| b.sides.is_empty()).count();
        l.blockers = blockers;
        // Machines, the box, Pack-a-Punch, power, wall buys.
        let mut dropped_perks = Vec::new();
        for p in named("zm_perk_machine") {
            match (p.get("script_noteworthy").and_then(machine_name), spot(p)) {
                (Some(name), Some(sp)) => l.perks.push(Machine { name: name.to_owned(), spot: sp }),
                _ => dropped_perks.push(p.get("script_noteworthy").unwrap_or("?").to_owned()),
            }
        }
        l.wheels = named("treasure_chest_use").iter().filter_map(|e| spot(e)).collect();
        l.pap = named("zm_pack_a_punch").first().and_then(|e| spot(e));
        l.generators = named("use_power_switch")
            .iter()
            .filter_map(|e| Some(Generator { spot: spot(e)?, areas: l.areas.iter().map(|a| a.name.clone()).collect(), lever: None }))
            .collect();
        let mut dropped_guns = Vec::new();
        for w in named("weapon_upgrade") {
            let gun = w.get("zombie_weapon_upgrade").unwrap_or_default();
            match (wall_weapon(gun), spot(w)) {
                (Some(g), Some(sp)) => l.wall_buys.push(WallBuy { weapon: g.to_owned(), spot: sp, model_spot: None }),
                _ => dropped_guns.push(gun.to_owned()),
            }
        }
        // Traversals: begin node to the node it targets.
        for b in ents.iter().filter(|e| e.classname() == "node_negotiation_begin") {
            let end = b.get("target").map(named).unwrap_or_default().first().and_then(|e| e.origin());
            if let (Some(begin), Some(end)) = (b.origin(), end) {
                l.traversals.push(Traversal { begin, end, animscript: b.get("animscript").unwrap_or_default().to_owned() });
            }
        }
        report.push(format!(
            "t7 zombies layout: {} starts, {} respawns, {} zones ({} spawners), {} windows, {} doors/debris ({} with no zone found), {} perks (left out: {:?}), {} box spots, pack-a-punch {}, {} power switches, {} wall buys (left out: {:?}), {} dog spots, {} traversals; not carried: teleporters, traps, Gobblegum machines",
            l.starts.len(),
            l.respawns.len(),
            l.areas.len(),
            l.areas.iter().map(|a| a.spawners.len()).sum::<usize>(),
            l.windows.len(),
            l.blockers.len(),
            no_side,
            l.perks.len(),
            dropped_perks,
            l.wheels.len(),
            if l.pap.is_some() { "found" } else { "missing" },
            l.generators.len(),
            l.wall_buys.len(),
            dropped_guns,
            l.dog_spots.len(),
            l.traversals.len()
        ));
        (l, triggers, report)
    }
}
