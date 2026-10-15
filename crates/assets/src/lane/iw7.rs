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
//! set's name with their colour maps (tinted by `colorTint`), a layered
//! world material's layers (`colorMap1`, `colorMap2`, weighted by the vertex
//! colour), and what the map names but the common zones hold
//! (`fastfile_iw7::shared`).
//!
//! Light: the lightmaps (`fastfile_iw7::light`, exposed per map), the
//! ComWorld's sun, the vision's fog; each placed model the floor's light
//! under and around it; players and other moving things a light grid made
//! from the lightmaps (`light_grid`). Brush models (doors, signs) are drawn
//! where their entities place them as the map starts
//! (`Iw7Map::place_brush_models`).
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
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP, TS_NORMAL_MAP, TS_SPECULAR_MAP};
use asset_transport::progress::{LoadProgress, StageId};
use asset_world::WorldDrawPolicy;
use fastfile_iw7::map::{Iw7Map, MapSources};

use super::LoadedWorld;
use crate::lane_capability::PreparedCapability;
use crate::session_load::PreparedWorld;

/// The largest level kept of a surface's or model's colour map
/// (`BO2MP_IW7_MAP_TEXTURE=<size>` changes it).
const TEXTURE_MAX: u32 = 512;

/// The world light table's row the placed models' sun is in (a primary
/// light of type 1: the shader's sun, its shadow included).
const SUN_LIGHT_ROW: usize = 1;

/// A placed model smaller than this (its radius, scaled) does not collide.
const COLLIDE_MIN_RADIUS: f32 = 24.0;

/// The largest specular map kept (half the colour map's at most: the
/// specular colour and gloss vary slowly).
const SPECULAR_MAX: u32 = 256;

/// Whether materials get their normal and specular maps
/// (`BO2MP_IW7_SHINE=0` draws colour maps only).
fn shine_on() -> bool {
    std::env::var("BO2MP_IW7_SHINE").map_or(true, |v| v.trim() != "0")
}

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
/// layout but for the unit vectors (unsigned 10:10:10 over 1023 there), the
/// binormal sign (packed differently: taken as +1) and the colour: as in
/// Ghosts, a placed model's vertex colour is no tint (foliage keeps its wind
/// weights there, other props near black: mp_proto's speakers and crates
/// drew black), so white.
fn iw7_vertex_to_t6(v: &[u8; 32]) -> [u8; 32] {
    let mut out = *v;
    out[12..16].copy_from_slice(&1.0f32.to_le_bytes());
    out[16..20].fill(0xff);
    for at in [24usize, 28] {
        let p = u32::from_le_bytes([v[at], v[at + 1], v[at + 2], v[at + 3]]);
        out[at..at + 4].copy_from_slice(&t6_unit_vec(fastfile_iw7::map::unpack_unit(p)).to_le_bytes());
    }
    out
}

/// What a technique set hides as, by the words of its name
/// (`fastfile_iw7::look`).
fn hidden_kind(technique_set: &str) -> Option<&'static str> {
    fastfile_iw7::look::look(technique_set).hidden
}

/// An IW7 technique set as Black Ops II's draw: its blend, alpha test,
/// lighting and decal step by the words of its name
/// (`fastfile_iw7::look`: `replace`, `atest`, `blend`, `blendadd`, `add`,
/// `multiply`, `unlit`, `ndw`), drawn double-sided for models. A world
/// material's layers (`r0c0.._b1c1..`, `m1c1`, `t1c1`, `b2c2`: two- and
/// three-layer walls, streets and terrain) are BO2's, their weights in the
/// vertex colour (`iw3_colour`); a model's are not drawn.
fn iw7_draw(technique_set: &str, sort_key: u8, model: bool) -> T6Draw {
    use fastfile_iw7::look::Blend;
    let look = fastfile_iw7::look::look(technique_set);
    let mut draw = T6Draw::from_technique_set(technique_set, sort_key, None);
    if model {
        draw.layered = false;
        draw.layers = [0, 0];
    }
    draw.water = false;
    draw.flow = false;
    draw.tile = false;
    draw.flag = false;
    draw.tile_blend = false;
    draw.objective = false;
    draw.foliage = false;
    draw.blend = match look.hidden {
        Some("sky") => T6Blend::Sky,
        Some("shadow only") => T6Blend::ShadowOnly,
        _ => match look.blend {
            Blend::Opaque => T6Blend::Opaque,
            Blend::Blend => T6Blend::Blend,
            Blend::Add => T6Blend::Add,
            Blend::Multiply => T6Blend::Multiply,
        },
    };
    draw.alpha_test = look.alpha_test;
    draw.unlit = look.unlit;
    // A decal (no depth write over the world) sits a step toward the eye.
    draw.polygon_offset = u8::from(look.decal);
    if model {
        draw.cull = T6Cull::None;
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
    /// (material, layer) -> that layer's colour map with opacity in alpha.
    alpha_images: HashMap<(usize, usize), Option<usize>>,
    /// Layered world materials drawn with their layers.
    layered: usize,
    decoded: usize,
    with_alpha: usize,
    failures: Vec<String>,
    /// Material -> its (colour without metal, normal, specular) slots
    /// (`fastfile_iw7::shine`).
    shine: HashMap<usize, [Option<usize>; 3]>,
    shine_on: bool,
    with_shine: usize,
}

impl Materials<'_> {
    /// Block-compressed levels into the catalog.
    fn link_levels(&mut self, key: String, l: fastfile_iw7::texture::ImageLevels, semantic: u8) -> Option<usize> {
        let px = fastfile_iw6::image::ImagePixels { width: l.width, height: l.height, levels: l.levels, dxgi_format: l.format, data: l.data };
        let srgb = semantic != TS_NORMAL_MAP;
        match super::iw6::gpu_image(&px, srgb) {
            Ok(gpu) => Some(self.catalog.link_image(AuthoredImage {
                namespace: AssetNamespace::T6,
                name: AssetRef::Real(key),
                map_type: 0,
                semantic,
                category: 0,
                use_srgb_reads: srgb,
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
                if self.failures.len() < 64 {
                    self.failures.push(format!("{key}: {e}"));
                }
                None
            }
        }
    }

    /// A lit opaque or cut-out material's colour without the metal's
    /// diffuse, normal and specular maps (Black Ops II's form).
    fn shine(&mut self, material: usize, colour: bool) -> [Option<usize>; 3] {
        if let Some(s) = self.shine.get(&material) {
            return *s;
        }
        let m = &self.map.materials[material];
        let slots = match self.map.shine(m, self.max, SPECULAR_MAX.min(self.max), colour) {
            Ok(sh) => {
                let base = format!("iw7:{}#{}", m.name, self.max);
                let c = sh.colour.and_then(|l| self.link_levels(format!("{base}#diffuse"), l, TS_COLOR_MAP));
                let n = sh.normal.and_then(|l| self.link_levels(format!("{base}#normal"), l, TS_NORMAL_MAP));
                let sp = sh.specular.and_then(|l| self.link_levels(format!("{base}#specular"), l, TS_SPECULAR_MAP));
                if n.is_some() || sp.is_some() {
                    self.with_shine += 1;
                }
                [c, n, sp]
            }
            Err(e) => {
                if self.failures.len() < 64 {
                    self.failures.push(format!("{} (shine): {e}", m.name));
                }
                [None; 3]
            }
        };
        self.shine.insert(material, slots);
        slots
    }

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

    /// A material's colour map as RGBA8 with its opacity in alpha
    /// (`Iw7Map::colour_rgba`), mipmapped.
    fn image_with_alpha(&mut self, material: usize, layer: usize) -> Option<usize> {
        if let Some(slot) = self.alpha_images.get(&(material, layer)) {
            return *slot;
        }
        let m = &self.map.materials[material];
        // (A layer with no alpha map of its own and a packed colour map is
        // opaque: its vertex weight alone blends it in. A layer's plain colour map, `signage_logo_corporate_05_c_white`,
        // keeps its own alpha as its opacity, as the base's does.)
        let slot_name = if layer == 0 { "colorMap".to_owned() } else { format!("colorMap{layer}") };
        let keep = m.texture(&slot_name).is_some_and(fastfile_iw7::look::colour_alpha_is_opacity);
        let slot = match self.map.layer_rgba(m, layer, self.max, keep) {
            Ok(rgba) => {
                use bevy::asset::RenderAssetUsages;
                use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
                use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
                let mut image = bevy::prelude::Image::new_uninit(
                    Extent3d { width: rgba.width, height: rgba.height, depth_or_array_layers: 1 },
                    TextureDimension::D2,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD,
                );
                image.texture_descriptor.mip_level_count = rgba.levels.len() as u32;
                image.data = Some(rgba.levels.concat());
                image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: ImageFilterMode::Linear,
                    min_filter: ImageFilterMode::Linear,
                    mipmap_filter: ImageFilterMode::Linear,
                    anisotropy_clamp: 8,
                    ..Default::default()
                });
                self.decoded += 1;
                self.with_alpha += 1;
                Some(self.catalog.link_image(AuthoredImage {
                    namespace: AssetNamespace::T6,
                    name: AssetRef::Real(format!("iw7:{}#alpha{layer}#{}", m.name, self.max)),
                    map_type: 0,
                    semantic: TS_COLOR_MAP,
                    category: 0,
                    use_srgb_reads: true,
                    width: rgba.width as u16,
                    height: rgba.height as u16,
                    depth: 1,
                    level_count: rgba.levels.len() as u8,
                    format: 0,
                    payload: Arc::new(Vec::new()),
                    decoded: Some(Arc::new(image)),
                    common_owned: false,
                    decoded_variant: None,
                    decoded_by: None,
                    pending_decode: None,
                }))
            }
            Err(e) => {
                if self.failures.len() < 64 {
                    self.failures.push(format!("{} (with alpha): {e}", m.name));
                }
                None
            }
        };
        self.alpha_images.insert((material, layer), slot);
        slot
    }

    fn material(&mut self, index: usize, model: bool) -> Option<usize> {
        if let Some(&m) = self.linked.get(&(index, model)) {
            return m;
        }
        let m = &self.map.materials[index];
        let mut draw = iw7_draw(&m.technique_set, m.sort_key, model);
        let colour = m.texture("colorMap").map(str::to_owned);
        let mut textures = Vec::new();
        if let Some(name) = colour {
            let hash = fastfile_iw7::texture::name_hash("colorMap");
            let sampler = m.textures.iter().find(|t| t.0 == hash).map_or(0x13, |t| t.1);
            // A cut-out or blended material takes its opacity from its alpha
            // map (or a plain colour map's own alpha): the packed colour
            // maps keep smoothness in alpha (`fastfile_iw7::look`).
            let has_alpha_map = m.textures.iter().any(|t| t.0 == fastfile_iw7::look::ALPHA_MAP_HASH);
            let wants_alpha = draw.alpha_test || matches!(draw.blend, T6Blend::Blend);
            // A lit material's shine (`fastfile_iw7::shine`): its colour
            // without the metal's diffuse (opaque ones; tinted as below),
            // normal and specular maps.
            let lit = self.shine_on && !draw.unlit && matches!(draw.blend, T6Blend::Opaque);
            let shine = if lit { self.shine(index, !draw.alpha_test) } else { [None; 3] };
            // (A tinted material's colour, `colorTint`, is its own picture.)
            let tinted = fastfile_iw7::look::tint(m, 0).is_some();
            let slot = if wants_alpha && (has_alpha_map || fastfile_iw7::look::colour_alpha_is_opacity(&name)) {
                self.image_with_alpha(index, 0).or_else(|| self.image(&name))
            } else if tinted && shine[0].is_none() {
                self.image_with_alpha(index, 0).or_else(|| self.image(&name))
            } else {
                shine[0].or_else(|| self.image(&name))
            };
            for (semantic, image) in [(TS_NORMAL_MAP, shine[1]), (TS_SPECULAR_MAP, shine[2])] {
                if let Some(image) = image {
                    textures.push(MaterialTextureBinding {
                        name_hash: if semantic == TS_NORMAL_MAP { 0x59d3_0d0f } else { 0x34ec_ccb3 },
                        name_start: 0,
                        name_end: 0,
                        sampler_state: sampler,
                        semantic,
                        image: Some(image),
                    });
                }
            }
            if let Some(slot) = slot {
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
        // A layered world material's layers: `colorMap1` and `colorMap2`
        // (each with its alpha map's opacity) as BO2's layer pictures. A
        // layer whose picture is not read is not drawn (the surface stays
        // untinted by its weights all the same).
        if draw.layered {
            let mut drawn = false;
            for (k, (hash, semantic)) in super::t6_materials::LAYER_MAPS.into_iter().enumerate() {
                if draw.layers[k] == 0 {
                    continue;
                }
                let slot = m.texture(&format!("colorMap{}", k + 1)).is_some().then(|| self.image_with_alpha(index, k + 1)).flatten();
                match slot {
                    Some(slot) => {
                        drawn = true;
                        let sampler = m.textures.iter().find(|t| t.0 == hash).map_or(0x13, |t| t.1);
                        textures.push(MaterialTextureBinding {
                            name_hash: hash,
                            name_start: 0,
                            name_end: 0,
                            sampler_state: sampler,
                            semantic,
                            image: Some(slot),
                        });
                    }
                    None => draw.layers[k] = 0,
                }
            }
            self.layered += usize::from(drawn);
        }
        let m = &self.map.materials[index];
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

/// The most reflection probes kept: a cube array of six layers a probe
/// stays within the 256 layers every GPU allows.
const MAX_PROBES: usize = 42;

/// The map's reflection probes (`Iw7Map::probe_array`, each probe scaled
/// to a mean of one: a reflection is lit as the light where it is seen) as
/// Black Ops II's probe cube array (`t6_materials::T6_PROBES_SEMANTIC`).
/// Returns the probes' lighting table (a flat one each) and the report
/// line; `None` with the reason when there are none.
/// Per reflection probe, its lighting (`WorldDraw::t6_probes`).
type ProbeTable = Vec<[[f32; 4]; 3]>;

fn link_probes(map: &Iw7Map, catalog: &mut MaterialCatalog, owner: ZoneOwner) -> Result<(ProbeTable, String), String> {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension};
    let arr = map.probe_array(1.0, true)?;
    let count = (arr.count as usize).min(MAX_PROBES);
    let face_bytes: usize = (0..arr.levels).map(|l| ((arr.size >> l).max(1) as usize).pow(2) * 4).sum();
    let data = arr.data[..face_bytes * 6 * count].to_vec();
    let mut image = bevy::prelude::Image::new_uninit(
        Extent3d { width: arr.size, height: arr.size, depth_or_array_layers: 6 * count as u32 },
        TextureDimension::D2,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = arr.levels;
    image.texture_view_descriptor = Some(TextureViewDescriptor { dimension: Some(TextureViewDimension::CubeArray), ..Default::default() });
    image.data = Some(data);
    let slot = catalog.link_image(AuthoredImage {
        namespace: AssetNamespace::T6,
        name: AssetRef::Real(format!("iw7:{}#probes", map.name)),
        map_type: 5,
        semantic: super::t6_materials::T6_PROBES_SEMANTIC,
        category: 0,
        use_srgb_reads: false,
        width: arr.size as u16,
        height: arr.size as u16,
        depth: 1,
        level_count: arr.levels as u8,
        format: 0,
        payload: Arc::new(Vec::new()),
        decoded: Some(Arc::new(image)),
        common_owned: false,
        decoded_variant: None,
        decoded_by: None,
        pending_decode: None,
    });
    catalog.link_material(AuthoredMaterial {
        name: AssetRef::Real(super::t6_materials::T6_PROBES_MATERIAL.to_owned()),
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
            name_hash: 0,
            name_start: 0,
            name_end: 0,
            sampler_state: 0,
            semantic: super::t6_materials::T6_PROBES_SEMANTIC,
            image: Some(slot),
        }],
        constants: Vec::new(),
        zone: owner,
        t6_draw: None,
    });
    // (A flat light of one: the shaders divide the reflection by the light
    // the probe saw, which normalising made one.)
    let table = vec![[[1.0, 1.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0], [0.0; 4]]; count];
    Ok((table, format!("iw7 reflection probes: {count} of {} ({}px, {} levels, each to a mean of one)", arr.count, arr.size, arr.levels)))
}

/// The probe nearest a point among the first `MAX_PROBES`.
fn nearest_probe(map: &Iw7Map, p: [f32; 3]) -> u8 {
    map.probe_origins
        .iter()
        .take(MAX_PROBES)
        .enumerate()
        .min_by(|a, b| {
            let d = |o: &[f32; 3]| (0..3).map(|k| (o[k] - p[k]).powi(2)).sum::<f32>();
            d(a.1).total_cmp(&d(b.1))
        })
        .map_or(0, |(i, _)| i as u8)
}

/// The map's sun as CoD4's (angles toward the sun, colour, gamma-space
/// light): the ComWorld's sun (its primary light of type 1: HDR linear
/// light, the brightest channel the light, the direction toward the sun),
/// in the lightmap pages' exposure (`fastfile_iw7::light::sun_gamma`);
/// else the worldspawn's `sundirection`, `suncolor` and `sunlight`.
fn sun_of(map: &Iw7Map, entities: &[iw3::ents::Entity], exposure: Option<f32>) -> ([f32; 3], [f32; 3], f32, &'static str) {
    if let Some(l) = map.lights.iter().find(|l| l.kind == 1 && l.color.iter().any(|&c| c > 0.0)) {
        let d = l.dir;
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        let d = d.map(|x| x / n);
        let yaw = d[1].atan2(d[0]).to_degrees();
        let pitch = (-d[2]).clamp(-1.0, 1.0).asin().to_degrees();
        let peak = l.color.iter().copied().fold(1e-6f32, f32::max);
        // (In the lightmap pages' units when they were read: their exposure;
        // else Ghosts' quarter.)
        let light = fastfile_iw7::light::sun_gamma(peak, exposure.unwrap_or(0.25));
        return ([pitch, yaw, 0.0], l.color.map(|c| c / peak), light, "the ComWorld");
    }
    let world = entities.iter().find(|e| e.classname() == "worldspawn");
    let vec3 = |k: &str| -> Option<[f32; 3]> {
        let v: Vec<f32> = world?.get(k)?.split_whitespace().filter_map(|x| x.parse().ok()).collect();
        (v.len() == 3).then(|| [v[0], v[1], v[2]])
    };
    let angles = vec3("sundirection").unwrap_or([-60.0, 45.0, 0.0]);
    let colour = vec3("suncolor").unwrap_or([1.0; 3]);
    // (IW7's `sunlight` is in its HDR units: about 7 outdoors.)
    let light = world.and_then(|e| e.get("sunlight")).and_then(|v| v.trim().parse::<f32>().ok()).map_or(1.0, |l| (l.max(0.0).sqrt() * 0.5).clamp(0.6, 1.6));
    (angles, colour, light, "the worldspawn")
}

/// The fog of a map's vision (`vision/<map>.vision`): IW7 keeps it as
/// dvar lines, `fogStartDist`, `fogDensity` (a distance: the halfway one,
/// 300 under Geneva's water, 30000..70000 outdoors), `fogColor` (and an
/// HDR `fogColorIntensity`, not used: the fog here is in display colour),
/// `fogMaxOpacity` and the sun fog (`fogSunColor`, `fogSunDir`,
/// `fogSunBeginFadeAngle`, `fogSunEndFadeAngle`, on with
/// `fogSunColorIntensity` above zero).
fn vision_fog(text: &str) -> Option<super::cod4::LegacyFog> {
    let mut vals: HashMap<String, Vec<f32>> = HashMap::new();
    for line in text.lines() {
        let Some((key, value)) = line.trim().split_once(' ') else { continue };
        let v: Vec<f32> = value.trim().trim_matches('"').split_whitespace().filter_map(|x| x.parse().ok()).collect();
        if !v.is_empty() {
            vals.entry(key.to_ascii_lowercase()).or_insert(v);
        }
    }
    let one = |k: &str| vals.get(k).and_then(|v| v.first().copied());
    let three = |k: &str| vals.get(k).filter(|v| v.len() >= 3).map(|v| [v[0], v[1], v[2]]);
    let half = one("fogdensity")?;
    let opacity = one("fogmaxopacity").unwrap_or(1.0).clamp(0.0, 1.0);
    if half <= 0.0 || opacity <= 0.0 {
        return None;
    }
    let colour = three("fogcolor")?.map(|c| c.clamp(0.0, 1.0));
    let sun = (one("fogsuncolorintensity").unwrap_or(0.0) > 0.0)
        .then(|| {
            Some(asset_world::SunFog {
                color_rgb: three("fogsuncolor")?.map(|c| c.clamp(0.0, 1.0)),
                sun_dir: three("fogsundir")?,
                begin_angle_deg: one("fogsunbeginfadeangle").unwrap_or(0.0),
                end_angle_deg: one("fogsunendfadeangle").unwrap_or(90.0),
                scale: 1.0,
            })
        })
        .flatten();
    Some(super::cod4::LegacyFog {
        start: one("fogstartdist").unwrap_or(0.0).max(0.0),
        half,
        opacity,
        colour,
        sun,
        base_height: 1.0e7,
        half_height: 1.0e9,
    })
}

/// IW7's world vertex colour (bytes R, G, B, A) as CoD4's D3DCOLOR (bytes
/// B, G, R, A), which the IW3 world builder reads, opaque.
///
/// It is no tint: a single-layer surface's is white (every vertex of
/// mp_riot's and mp_geneva's); a layered surface's holds its layers'
/// weights, green layer 1 and blue layer 2, as Ghosts' and BO2's (red 255,
/// alpha 0 on two-layer surfaces). Read as D3DCOLOR it was the tint that
/// turned mp_riot's streets magenta and cyan; its alpha, no opacity, is
/// made opaque.
fn iw3_colour(rgba: u32) -> u32 {
    let [r, g, b, _] = rgba.to_le_bytes();
    u32::from_le_bytes([b, g, r, 0xff])
}

/// The light grid Black Ops II's rules light players, guns and other
/// moving things by (`asset_world::T6LightSampler`); with none they took a
/// flat 0.5 everywhere. IW7's own grid (its light probes) is not read; this
/// one is made from the lightmaps, in Modern Warfare 2's layout
/// (`asset_model::OwnedLightGrid`, the sampler's `row_rle`): a point every
/// 32 x 32 x 64 units over the area of the spawn points and AI path nodes
/// (`places`: a Zombies map's player starts and its nodes through every
/// room) and `GRID_MARGIN` round it, from below the lowest to above the
/// highest, each the light of
/// the floor under it (`Iw7Map::light_volume`) plus the sun where the floor
/// sees it, the same in every direction but a little brighter from above.
/// Rows run along x, columns along y, every column one solid run of every
/// height.
fn light_grid(
    map: &Iw7Map,
    places: &[[f32; 3]],
    pages: &[Option<fastfile_iw7::light::LightmapPage>],
    sun_linear: [f32; 3],
    fallback: Option<fastfile_iw7::light::LightSample>,
) -> Result<(asset_model::OwnedLightGrid, String), String> {
    const BIAS: f32 = 131_072.0;
    const SPACING: [f32; 3] = [32.0, 32.0, 64.0];
    const GRID_MARGIN: f32 = 1024.0;
    const MAX_POINTS: usize = 6_000_000;
    let spawns: Vec<[f32; 3]> = places.iter().copied().filter(|p| p.iter().all(|v| v.is_finite())).collect();
    if spawns.is_empty() {
        return Err("no spawn points or path nodes".to_owned());
    }
    // (The 1st to 99th percentile on each axis: a stray node far out does
    // not stretch the grid.)
    let bound = |k: usize, q: f32| {
        let mut v: Vec<f32> = spawns.iter().map(|p| p[k]).collect();
        v.sort_by(f32::total_cmp);
        v[((v.len() - 1) as f32 * q).round() as usize]
    };
    let lo: [f32; 3] = std::array::from_fn(|k| bound(k, 0.01));
    let hi: [f32; 3] = std::array::from_fn(|k| bound(k, 0.99));
    let mut margin = GRID_MARGIN;
    let (cmin, dims) = loop {
        let wlo = [(lo[0] - margin).max(map.mins[0]), (lo[1] - margin).max(map.mins[1]), lo[2] - 192.0];
        let whi = [(hi[0] + margin).min(map.maxs[0]), (hi[1] + margin).min(map.maxs[1]), hi[2] + 704.0];
        let cmin: [i64; 3] = std::array::from_fn(|k| ((wlo[k] + BIAS) / SPACING[k]).floor() as i64);
        let cmax: [i64; 3] = std::array::from_fn(|k| ((whi[k] + BIAS) / SPACING[k]).ceil() as i64);
        let dims: [usize; 3] = std::array::from_fn(|k| (cmax[k] - cmin[k] + 1).max(1) as usize);
        if dims[0] * dims[1] * dims[2] <= MAX_POINTS || margin <= 0.0 {
            break (cmin, dims);
        }
        margin -= 256.0;
    };
    if cmin.iter().any(|&c| c < 0) || dims[2] > 255 || dims[0] > 0xffff || dims[1] > 0xffff || dims[0] * dims[1] * dims[2] > MAX_POINTS * 2 {
        return Err(format!("grid {dims:?} out of range"));
    }
    let world0: [f32; 3] = std::array::from_fn(|k| cmin[k] as f32 * SPACING[k] - BIAS);
    let volume = map.light_volume(pages, world0, std::array::from_fn(|k| world0[k] + (dims[k] - 1) as f32 * SPACING[k]), SPACING, 640.0);
    // Light as the sampler decodes it: mean of the colours (0..1), squared,
    // times 4 (`ForeignGridLight::scale` for 8-bit colours).
    let encode = |l: f32| ((l.max(0.0) / 4.0).sqrt() * 255.0).round().clamp(0.0, 255.0) as u8;
    let dirs = iw3::zone::LightGrid::directions();
    let mut palette: HashMap<[u8; 4], u16> = HashMap::new();
    let mut colors: Vec<u8> = Vec::new();
    let mut entries: Vec<u8> = Vec::with_capacity(dims[0] * dims[1] * dims[2] * 4);
    let (mut found, mut sunlit) = (0usize, 0usize);
    for ix in 0..dims[0] {
        for iy in 0..dims[1] {
            for iz in 0..dims[2] {
                let sample = volume.at(ix, iy, iz);
                found += usize::from(sample.is_some());
                let l = sample.or(fallback).map_or([0.5; 3], |s| {
                    let a = s.ambient_linear();
                    // (Half the sun: a body faces it on one side.)
                    std::array::from_fn(|k| a[k] + sun_linear[k] * s.sun * 0.5)
                });
                let sun = sample.is_some_and(|s| s.sun > 0.5);
                sunlit += usize::from(sun);
                // Quantised to 6 bits a channel for the palette.
                let key = [encode(l[0]) & 0xfc, encode(l[1]) & 0xfc, encode(l[2]) & 0xfc, 0];
                let next = palette.len();
                let index = match palette.get(&key) {
                    Some(&i) => i,
                    None if next < 0xffff => {
                        palette.insert(key, next as u16);
                        for d in &dirs {
                            let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
                            // (A quarter brighter from above, as much darker
                            // from below: the mean is the floor's light.)
                            let up = 1.0 + 0.25 * (d[2] / n);
                            colors.extend((0..3).map(|k| encode(l[k] * up)));
                        }
                        next as u16
                    }
                    None => 0,
                };
                entries.extend_from_slice(&index.to_le_bytes());
                entries.push(if sun { SUN_LIGHT_ROW as u8 } else { 0 });
                entries.push(0);
            }
        }
    }
    // The rows (`lighting_iw4::grid_build`: every column one run of every
    // height, the entries in the order made above).
    let (cols, heights) = (dims[1] as u16, dims[2] as u16);
    let row_len = lighting_iw4::grid_build::full_box_row_len(cols);
    let mut row_data_start = Vec::with_capacity(dims[0] * 2);
    let mut raw_row_data = vec![0u8; dims[0] * row_len];
    for ix in 0..dims[0] {
        let at = u16::try_from(ix * row_len / 4).map_err(|_| "row data too large".to_owned())?;
        row_data_start.extend_from_slice(&at.to_le_bytes());
        if lighting_iw4::grid_build::full_box_row(ix, cmin[1] as u16, cols, cmin[2] as u16, heights, &mut raw_row_data[ix * row_len..]) == 0 {
            return Err(format!("row {ix} of {dims:?} does not fit"));
        }
    }
    let to16 = |c: i64| c as u16;
    let grid = asset_model::OwnedLightGrid {
        mins: [to16(cmin[0]), to16(cmin[1]), to16(cmin[2])],
        maxs: std::array::from_fn(|k| to16(cmin[k] + dims[k] as i64 - 1)),
        row_axis: 0,
        col_axis: 1,
        color_count: palette.len() as u32,
        has_light_regions: false,
        sun_primary_light_index: SUN_LIGHT_ROW as u32,
        row_data_start,
        raw_row_data,
        entries,
        colors,
        color_encoding: asset_model::LightGridColorEncoding::Rgb8,
    };
    let total = dims[0] * dims[1] * dims[2];
    Ok((grid, format!("{dims:?} = {total} points from {world0:?}, {found} over a lit floor ({sunlit} in the sun), {} colours", palette.len())))
}

/// The largest lightmap page width kept (`BO2MP_IW7_LIGHTMAP=<px>`): IW7's
/// pages are 4096 wide; the fallback draw's page is three times the page's
/// texels in RGBA8, 25 MB at 2048.
fn lightmap_max() -> u32 {
    std::env::var("BO2MP_IW7_LIGHTMAP")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|&v: &u32| (256..=4096).contains(&v))
        .unwrap_or(2048)
}

/// The world as CoD4's `GfxWorld` (materials beside it, not in it).
fn iw3_world(map: &Iw7Map, sun: ([f32; 3], [f32; 3], f32, &'static str)) -> iw3::zone::GfxWorld {
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
                color: iw3_colour(v.color),
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
                lightmap_index: s.lightmap_index,
                reflection_probe_index: nearest_probe(map, std::array::from_fn(|k| (s.mins[k] + s.maxs[k]) * 0.5)),
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

/// The pictures and materials IW7's map zones name but do not hold (the
/// common zones' and `global_core_mp`'s: `fastfile_iw7::shared`), read once
/// per folder (a few seconds; image records, small inline pictures and
/// material texture lists kept, streamed pictures read from the paks when a
/// map wants them), as Ghosts' `iw6_map::shared_images`.
fn shared_zones(dir: &Path) -> Option<Arc<fastfile_iw7::shared::SharedZones>> {
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<std::path::PathBuf, Arc<fastfile_iw7::shared::SharedZones>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(hit) = cache.lock().ok()?.get(dir) {
        return Some(hit.clone());
    }
    let zones = fastfile_iw7::shared::common_zones(dir);
    let read = std::panic::catch_unwind(|| fastfile_iw7::shared::SharedZones::read(&zones));
    let shared = match read {
        Ok(s) => Arc::new(s),
        Err(_) => {
            diag::warn!(World, "bo2mp iw7: reading the common zones in {} panicked", dir.display());
            return None;
        }
    };
    diag::info!(World, "bo2mp iw7: common zones: {:?}", shared.zones);
    cache.lock().ok()?.insert(dir.to_path_buf(), shared.clone());
    Some(shared)
}

/// Loads an Infinite Warfare map zone into the match's world. A zone that
/// cannot be read becomes a gap with the reason, never a crash.
pub(crate) fn load_iw7_world(path: &Path, progress: &LoadProgress, material_seed: MaterialCatalog) -> LoadedWorld {
    let mut report = vec![format!("game: IW7 / Infinite Warfare ({})", path.display())];
    let stage = progress.begin_scoped(StageId::MapAssets, "infinite warfare zone", None);
    let started = std::time::Instant::now();
    let read = std::panic::catch_unwind(|| {
        let mut src = MapSources::beside(path);
        src.shared = path.parent().and_then(shared_zones);
        Iw7Map::load(&src)
    });
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
fn collision_of(map: &Iw7Map, hidden: &[bool]) -> (asset_world::ClipCollision, Vec<asset_world::world_iw7::SoupTri>, usize) {
    use asset_world::world_iw7::build_soup_clip_collision;
    let (tris, cleared) = collision_tris(map, hidden);
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
    (clip, tris, cleared)
}

/// Whether a world surface of this material collides: opaque and
/// alpha-tested surfaces do (grates, fences, railings), but not cut-out
/// foliage; of the blended ones only glass (windows, glass floors) does,
/// not decals or light cards.
fn collides(m: &fastfile_iw7::map::MapMaterial) -> bool {
    let draw = iw7_draw(&m.technique_set, m.sort_key, false);
    let name = m.name.to_ascii_lowercase();
    let foliage = ["foliage", "leaf", "leaves", "grass", "ivy", "vine", "fern", "bush", "hedge", "plant", "flower"]
        .iter()
        .any(|w| name.contains(w));
    match draw.blend {
        T6Blend::Opaque => !(draw.alpha_test && foliage),
        T6Blend::Blend => name.contains("glass") && !draw.unlit && draw.polygon_offset == 0,
        _ => false,
    }
}

/// Whether a triangle overlaps a box (`mid`, `half`): the separating axis
/// test (the box's axes, the triangle's normal, their edges' crossings).
fn tri_overlaps_box(t: &[[f32; 3]; 3], mid: [f32; 3], half: [f32; 3]) -> bool {
    let v: [[f32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|k| t[i][k] - mid[k]));
    let sub = |a: [f32; 3], b: [f32; 3]| -> [f32; 3] { std::array::from_fn(|k| a[k] - b[k]) };
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let separated = |axis: [f32; 3]| {
        let p = v.map(|q| dot(q, axis));
        let r = half[0] * axis[0].abs() + half[1] * axis[1].abs() + half[2] * axis[2].abs();
        p.iter().copied().fold(f32::MAX, f32::min) > r || p.iter().copied().fold(f32::MIN, f32::max) < -r
    };
    let edges = [sub(v[1], v[0]), sub(v[2], v[1]), sub(v[0], v[2])];
    let units = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    if units.iter().any(|&u| separated(u)) || separated(cross(edges[0], edges[1])) {
        return false;
    }
    !edges.iter().any(|&e| units.iter().any(|&u| {
        let axis = cross(e, u);
        dot(axis, axis) > 1e-12 && separated(axis)
    }))
}

fn collision_tris(map: &Iw7Map, hidden: &[bool]) -> (Vec<asset_world::world_iw7::SoupTri>, usize) {
    use asset_world::world_iw7::{IW7_SOLID, SoupTri};
    // A spawn point's player box (lifted off its floor): a placed model's
    // triangle in it is left out of the collision, or the player spawned
    // stuck (50 of mp_fallen's 377 spawns stood in a prop's collision).
    const SPAWN_HALF: [f32; 3] = [16.0, 16.0, 33.0];
    const CELL: f32 = 256.0;
    let cell = |x: f32, y: f32| ((x / CELL).floor() as i32, (y / CELL).floor() as i32);
    let mut spawn_cells: HashMap<(i32, i32), Vec<[f32; 3]>> = HashMap::new();
    for sp in &map.spawns {
        let mid = [sp.origin[0], sp.origin[1], sp.origin[2] + 4.0 + SPAWN_HALF[2]];
        spawn_cells.entry(cell(mid[0], mid[1])).or_default().push(mid);
    }
    let in_a_spawn = |t: &[[f32; 3]; 3]| {
        let lo = [0, 1].map(|k| t.iter().map(|p| p[k]).fold(f32::MAX, f32::min) - SPAWN_HALF[k]);
        let hi = [0, 1].map(|k| t.iter().map(|p| p[k]).fold(f32::MIN, f32::max) + SPAWN_HALF[k]);
        let (c0, c1) = (cell(lo[0], lo[1]), cell(hi[0], hi[1]));
        if (c1.0 - c0.0 + 1) * (c1.1 - c0.1 + 1) > 64 {
            // (A huge triangle: tested against every spawn.)
            return spawn_cells.values().flatten().any(|&mid| tri_overlaps_box(t, mid, SPAWN_HALF));
        }
        (c0.0..=c1.0).any(|cx| (c0.1..=c1.1).any(|cy| spawn_cells.get(&(cx, cy)).is_some_and(|list| list.iter().any(|&mid| tri_overlaps_box(t, mid, SPAWN_HALF)))))
    };
    let mut cleared = 0usize;
    let mut tris = Vec::new();
    for (i, s) in map.surfaces.iter().enumerate() {
        // (A brush model collides as its entity's box brush instead.)
        if hidden.get(i).copied().unwrap_or(false) || s.brush_model != 0 {
            continue;
        }
        if let Some(m) = s.material
            && !collides(&map.materials[m])
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
            // (A tree's leaf cards, a decal sheet: no collision.)
            if s.material.is_some_and(|k| !collides(&map.materials[k])) {
                continue;
            }
            for t in &s.tris {
                let (Some(a), Some(b), Some(c)) = (s.verts.get(usize::from(t[0])), s.verts.get(usize::from(t[1])), s.verts.get(usize::from(t[2]))) else {
                    continue;
                };
                let p = [place(a), place(b), place(c)];
                if in_a_spawn(&p) {
                    cleared += 1;
                    continue;
                }
                tris.push(SoupTri { p, surface_flags: 0, contents: IW7_SOLID });
            }
        }
    }
    (tris, cleared)
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
        // A brush model's surface its entity does not show as the map
        // starts (`Iw7Map::place_brush_models`).
        if s.hidden_brush {
            hidden[i] = true;
            *hidden_kinds.entry("brush model not shown").or_default() += 1;
            continue;
        }
        if let Some(kind) = s.material.and_then(|m| hidden_kind(&map.materials[m].technique_set)) {
            hidden[i] = true;
            *hidden_kinds.entry(kind).or_default() += 1;
        }
    }

    // Collision.
    let stage = progress.begin_scoped(StageId::MapAssets, "infinite warfare collision", None);
    let (collision, soup, cleared) = collision_of(map, &hidden);
    let census = asset_world::world_iw3::spawn_ground_census(&collision, &map.spawns.iter().map(|s| s.origin).collect::<Vec<_>>());
    report.push(format!(
        "iw7 clip: {} triangles ({} vertices, {} partitions) from the drawn world and placed models ({cleared} props' triangles in spawn points' player boxes left out); {census}",
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
        alpha_images: HashMap::new(),
        layered: 0,
        decoded: 0,
        with_alpha: 0,
        failures: Vec::new(),
        shine: HashMap::new(),
        shine_on: shine_on(),
        with_shine: 0,
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

    // The baked light: each GfxLightMap as CoD4's page
    // (`fastfile_iw7::light`), for the world's lit surfaces; the placed
    // models take the light of the floor under them.
    let stage_light = progress.begin_scoped(StageId::MapAssets, "infinite warfare lightmaps", None);
    let lm_max = lightmap_max();
    let mut lightmap_gaps = Vec::new();
    let pages: Vec<Option<fastfile_iw7::light::LightmapPage>> = (0..map.light_maps.len())
        .map(|i| match map.lightmap_page(i, lm_max) {
            Ok(p) => Some(p),
            Err(e) => {
                lightmap_gaps.push(format!("page {i}: {e}"));
                None
            }
        })
        .collect();
    let lightmaps: Vec<Option<asset_world::WorldLightmap>> = pages
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let p = p.as_ref()?;
            let names = &map.light_maps[i];
            let (pw, ph, pp) = &p.primary;
            let (sw, sh, sp) = &p.secondary;
            match asset_world::world_iw3::iw3_lightmap(
                (*pw, *ph, asset_world::world_iw3::IW3_LIGHTMAP_PRIMARY_FORMAT, pp),
                (*sw, *sh, asset_world::world_iw3::IW3_LIGHTMAP_SECONDARY_FORMAT, sp),
                (&names[0], &names[1]),
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
        "iw7 lightmaps: {}/{} pages converted (at most {lm_max} wide){}",
        lightmaps.iter().filter(|p| p.is_some()).count(),
        lightmaps.len(),
        if lightmap_gaps.is_empty() { String::new() } else { format!(" ({})", lightmap_gaps.join("; ")) }
    ));
    let ground = map.static_model_light(&pages);
    // (A model with no floor found takes the middle light of those that
    // found one, not a flat grey.)
    let fallback_light = {
        let mut lit: Vec<fastfile_iw7::light::LightSample> = ground.iter().flatten().copied().collect();
        let lum = |l: &fastfile_iw7::light::LightSample| {
            let a = l.ambient_linear();
            0.2126 * a[0] + 0.7152 * a[1] + 0.0722 * a[2]
        };
        lit.sort_by(|a, b| lum(a).total_cmp(&lum(b)));
        lit.get(lit.len() / 2).copied()
    };
    // (The pages' exposure: the sun is brought into the same units.)
    let exposure = pages.iter().flatten().next().map(|p| p.exposure);
    report.push(format!(
        "iw7 exposure: {}",
        pages
            .iter()
            .flatten()
            .map(|p| format!("mean light {:.3} x{:.2}", p.mean_light, p.exposure))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    // The light grid for players and other moving things, in the sun's
    // units (`sun_of`: gamma; linear is its square).
    let sun = sun_of(map, &entities, exposure);
    let sun_linear: [f32; 3] = std::array::from_fn(|k| (sun.1[k] * sun.2).powi(2));
    let places: Vec<[f32; 3]> = spawns.iter().map(|s| s.origin).chain(map.path_nodes.iter().map(|n| n.origin)).collect();
    let light_grid = match light_grid(map, &places, &pages, sun_linear, fallback_light) {
        Ok((grid, line)) => {
            report.push(format!("iw7 light grid: {line}"));
            Some(grid)
        }
        Err(e) => {
            report.push(format!("iw7 light grid: none ({e}): moving things take a flat light"));
            None
        }
    };
    drop(pages);
    stage_light.done();

    // The placed models.
    let (prop_meshes, prop_placements) = {
        use bevy::prelude::{Mat3, Quat, Transform, Vec3};
        let mut meshes: Vec<asset_world::ModelMesh> = Vec::new();
        let mut built: HashMap<usize, Option<usize>> = HashMap::new();
        let mut placements = Vec::with_capacity(map.static_models.len());
        let mut lit = 0usize;
        for (i, sm) in map.static_models.iter().enumerate() {
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
                reflection_probe_index: nearest_probe(map, sm.lighting_origin),
                primary_light_index: 0,
                flags: 0,
                // The floor's light under the model (linear, packed as the
                // light grid's sqrt(light / 32)) and the sun (primary light
                // 1, `SUN_LIGHT_ROW`) by the floor's sun visibility; with no
                // lit floor below, a neutral light and half the sun.
                packed_lighting: Some(match ground.get(i).copied().flatten().inspect(|_| lit += 1).or(fallback_light) {
                    Some(g) => {
                        let a = g.ambient_linear();
                        let p = |v: f32| ((v.max(0.0) / 32.0).sqrt() * 255.0).round().clamp(1.0, 255.0) as u8;
                        [p(a[0]), p(a[1]), p(a[2]), 0]
                    }
                    None => [32, 32, 32, 0],
                }),
                t6_primary: [
                    SUN_LIGHT_ROW as u8,
                    ground.get(i).copied().flatten().or(fallback_light).map_or(128, |g| (g.sun * 255.0).round() as u8),
                ],
            }));
        }
        report.push(format!(
            "iw7 static models: {} placed of {}, {} distinct models built, {lit} lit by the floor under and around them (the rest by the median of those)",
            placements.iter().filter(|p| p.is_some()).count(),
            map.static_models.len(),
            meshes.len()
        ));
        (meshes, placements)
    };
    report.push(format!(
        "iw7 materials: {} linked ({} layered drawn with their layers, {} from the common zones), {} colour maps decoded ({} with their alpha maps, at most {}px), {} failed{}; not drawn: {hidden_kinds:?}",
        b.linked.len(),
        b.layered,
        map.shared_materials,
        b.decoded,
        b.with_alpha,
        b.max,
        b.failures.len(),
        if b.failures.is_empty() { String::new() } else { format!(" (first: {})", b.failures.iter().take(4).cloned().collect::<Vec<_>>().join("; ")) },
    ));
    report.push(format!("iw7 shine: {} materials with normal or specular maps{}", b.with_shine, if b.shine_on { "" } else { " (off: BO2MP_IW7_SHINE=0)" }));
    let mut materials = std::mem::take(&mut b.catalog);
    let owner = b.owner;
    drop(b);
    let probes = match link_probes(map, &mut materials, owner) {
        Ok((table, line)) => {
            report.push(line);
            table
        }
        Err(e) => {
            report.push(format!("iw7 reflection probes: none ({e}); surfaces draw without their normal and specular maps"));
            Vec::new()
        }
    };
    stage.done();

    let iw3w = iw3_world(map, sun);
    let (pitch, yaw) = (sun.0[0].to_radians(), sun.0[1].to_radians());
    let dir = [pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), -pitch.sin()];
    let colour = [sun.1[0] * sun.2, sun.1[1] * sun.2, sun.1[2] * sun.2, 1.0];
    report.push(format!("iw7 sun: from {} toward {dir:?} colour {colour:?}", sun.3));
    let lighting = asset_world::T6WorldLighting {
        sun: asset_world::WorldSunLight { direction: dir, diffuse: colour, specular: colour },
        exposure_scale: 1.0,
        sky: [0.0, 1.0, 0.0, 1.0],
        sky_parms: [1.0, 0.0],
        saturation: None,
        grade: None,
    };
    match asset_world::world_iw3::build_iw3_world_draw(&iw3w, &surface_materials, &hidden, lightmaps, Some(lighting)) {
        Ok(mut draw) => {
            // The fog (the map's vision) and the sun as a primary light for
            // the placed models (rows 30 and 31 are the fog's).
            let fog = map.vision.as_ref().and_then(|(_, text)| vision_fog(text));
            match (&fog, &map.vision) {
                (Some(f), Some((name, _))) => report.push(format!(
                    "iw7 fog: READY from {name}: start={:.1} half={:.1} opacity={:.3} colour={:?} sun={}",
                    f.start,
                    f.half,
                    f.opacity,
                    f.colour,
                    f.sun.is_some()
                )),
                (None, Some((name, _))) => report.push(format!("iw7 fog: none ({name} sets none)")),
                _ => report.push("iw7 fog: none (no vision in the zone)".to_owned()),
            }
            draw.t6_lights = fog.as_ref().map_or_else(|| vec![[0.0f32; 16]; 32], super::cod4::legacy_fog_table);
            draw.t6_probes = probes;
            // (bo2mp: never shorter: the fog rows sit after the 256 lights.)
            draw.t6_lights.resize(draw.t6_lights.len().max(32), [0.0; 16]);
            draw.t6_lights[SUN_LIGHT_ROW][3] = 1.0;
            report.push(format!(
                "world mesh: {} vertices, {} triangles, {} surfaces (Infinite Warfare, fallback draw with colour maps{})",
                draw.stats.vertices,
                draw.stats.triangles,
                draw.stats.surfaces,
                if draw.iw3_lightmaps { " and its lightmaps" } else { ", no lightmaps" }
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
                    light_grid,
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
    fn triangles_in_a_box() {
        let mid = [0.0, 0.0, 37.0];
        let half = [16.0, 16.0, 33.0];
        // Through the box, beside it, under its floor, a big one across it.
        assert!(super::tri_overlaps_box(&[[-50.0, 0.0, 30.0], [50.0, 0.0, 30.0], [0.0, 0.0, 60.0]], mid, half));
        assert!(!super::tri_overlaps_box(&[[20.0, 20.0, 30.0], [60.0, 20.0, 30.0], [20.0, 60.0, 30.0]], mid, half));
        assert!(!super::tri_overlaps_box(&[[-500.0, -500.0, 0.0], [500.0, -500.0, 0.0], [0.0, 500.0, 0.0]], mid, half));
        assert!(super::tri_overlaps_box(&[[-500.0, -500.0, 40.0], [500.0, -500.0, 40.0], [0.0, 500.0, 40.0]], mid, half));
        // Diagonal past a corner.
        assert!(!super::tri_overlaps_box(&[[45.0, -5.0, 0.0], [-5.0, 45.0, 0.0], [45.0, -5.0, 80.0]], mid, half));
        assert!(super::tri_overlaps_box(&[[30.0, -10.0, 0.0], [-10.0, 30.0, 0.0], [30.0, -10.0, 80.0]], mid, half));
    }

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
