//! bo2mp: Black Ops III's own light for its maps (`fastfile_t7::light`):
//! the sky picture as the sky, the reflection probes (their cubes, in the
//! map-wide probe's unit) for the ambient light and the reflections, the
//! sun volume's sun (direction from its angles, brightness from its
//! intensity) with shadows from a sun map baked at load over the map's own
//! triangles, and a filmic tone curve in place of Black Ops II's
//! square-root output. The probe volume and the sun shadow tree are not
//! read: the ambient light is the probes' ambient cubes times an occlusion
//! traced at load (`fastfile_t7::bake`).
//!
//! What the renderer gets: the world's vertex colours hold each vertex's
//! ambient light, every lit world surface reads a blank lightmap page (so
//! it draws in the lightmapped pipelines, with its normal and specular maps
//! and its probe), the placed models' light rows their ambient light, the
//! colour-table slot the sun map, and light rows 254 and 255 the numbers
//! (`asset_world::T7_LIGHT_ROW`); the fallback pass's T7 branch draws them.
//!
//! A self-contained path: `BO2MP_T7_LIGHT=0` turns it off (the old flat
//! light). Other games' maps never come here; the branch could take any
//! map whose light reads into these same pieces.

use std::sync::Arc;

use asset_core::{AssetEdge, AssetNamespace, AssetRef, T6Draw, ZoneOwner};
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP};
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension};
use fastfile_t7::bake::Tracer;
use fastfile_t7::light::{Bake, Blocks, SunMap};
use fastfile_t7::map::T7Map;

/// Whether Black Ops III maps get their own light (`BO2MP_T7_LIGHT=0`: the
/// old flat light).
pub(super) fn on() -> bool {
    std::env::var("BO2MP_T7_LIGHT").map_or(true, |v| v.trim() != "0")
}

/// The renderer's probe table holds this many (probe 0 the map-wide one).
const MAX_PROBES: usize = 32;
/// The sky cube's face size (the 2048-wide picture's quarter).
const SKY_FACE: u32 = 512;
/// What the tone curve's input is: light in the probes' unit times this
/// (`BO2MP_T7_EXPOSURE`).
const EXPOSURE: f32 = 0.6;
/// How far, round the spawn points' square, the sun map and the occlusion
/// reach.
const MARGIN: f32 = 3000.0;

/// A test view (`BO2MP_T7_VIEW`): 1 the sun's visibility, 2 the ambient
/// light, 3 the albedo, 4 the light (no albedo); 0 the picture.
fn view() -> f32 {
    match std::env::var("BO2MP_T7_VIEW").as_deref().map(str::trim) {
        Ok("sun") => 1.0,
        Ok("ambient") => 2.0,
        Ok("albedo") => 3.0,
        Ok("light") => 4.0,
        _ => 0.0,
    }
}

fn exposure() -> f32 {
    std::env::var("BO2MP_T7_EXPOSURE").ok().and_then(|v| v.trim().parse().ok()).filter(|v: &f32| *v > 0.0 && *v < 100.0).unwrap_or(EXPOSURE)
}

/// The map's light, baked.
pub(super) struct Lit {
    pub bake: Bake,
    /// The probes drawn (`keep[0]` is 0, the map-wide one).
    pub keep: Vec<usize>,
    pub sun_map: SunMap,
    tracer: Tracer,
    /// Toward the sun.
    pub sun: [f32; 3],
    /// The sun's brightness in the probes' unit (its colour's peak one).
    pub sun_light: f32,
    pub exposure: f32,
    pub report: Vec<String>,
}

impl Lit {
    /// Reads the light and bakes the sun map and the occluders round the
    /// spawn points (`spawns`); `blocks` says what a material does to light.
    pub fn prepare(map: &T7Map, spawns: &[[f32; 3]], blocks: &dyn Fn(usize) -> Blocks) -> Result<Lit, String> {
        let started = std::time::Instant::now();
        let bake = Bake::read(map)?;
        if bake.probes.is_empty() {
            return Err("no reflection probes".to_owned());
        }
        let mut report: Vec<String> = bake.report.iter().map(|l| format!("t7 light: {l}")).collect();
        // The spawn points' square (else the probes' boxes).
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        for p in spawns {
            lo = [lo[0].min(p[0]), lo[1].min(p[1])];
            hi = [hi[0].max(p[0]), hi[1].max(p[1])];
        }
        if lo[0] > hi[0] {
            lo = [bake.bounds[0][0], bake.bounds[0][1]];
            hi = [bake.bounds[1][0], bake.bounds[1][1]];
        }
        let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
        let span = ((hi[0] - lo[0]).max(hi[1] - lo[1]) + MARGIN).clamp(2048.0, 16000.0);
        let area = [[centre[0] - span * 0.5 - MARGIN, centre[1] - span * 0.5 - MARGIN], [centre[0] + span * 0.5 + MARGIN, centre[1] + span * 0.5 + MARGIN]];
        let casters = fastfile_t7::light::casters(map, blocks, 8.0, area);
        let sun = map.sun.as_ref().map_or([0.4, 0.3, 0.866], fastfile_t7::light::sun_toward);
        let z = spawns.iter().map(|p| p[2]).sum::<f32>() / spawns.len().max(1) as f32;
        let sun_map = SunMap::render(&casters, sun, [centre[0], centre[1], z], span * 1.15, 4096);
        let solid: Vec<[[f32; 3]; 3]> = casters.iter().filter(|c| !c.partial).map(|c| c.p).collect();
        let tracer = Tracer::new(&solid);
        let units = bake.units();
        let sun_light = map.sun.filter(|s| s.intensity.is_finite() && s.intensity > 0.0).map_or(5.0, |s| s.intensity / units);
        // The probes drawn: those the most surfaces and models take light
        // from round the spawn points, the map-wide one first.
        let mut uses = vec![0usize; bake.probes.len()];
        let inside = |p: [f32; 3]| p[0] >= area[0][0] && p[0] <= area[1][0] && p[1] >= area[0][1] && p[1] <= area[1][1];
        for s in &map.surfaces {
            let c = std::array::from_fn(|k| (s.mins[k] + s.maxs[k]) * 0.5);
            if inside(c)
                && let Some((_, i)) = bake.box_at(c)
            {
                uses[i] += 1;
            }
        }
        for sm in &map.static_models {
            if inside(sm.origin)
                && let Some((_, i)) = bake.box_at(sm.origin)
            {
                uses[i] += 1;
            }
        }
        let mut order: Vec<usize> = (1..bake.probes.len()).filter(|&i| uses[i] > 0).collect();
        order.sort_by(|&a, &b| uses[b].cmp(&uses[a]));
        let mut keep = vec![0usize];
        keep.extend(order.into_iter().take(MAX_PROBES - 1));
        report.push(format!(
            "t7 light: {} of {} probes drawn (unit {units:.1}), sun toward {sun:.3?} at {sun_light:.2}, sun map {}px over {:.0} units ({:.2} a texel), {} blockers ({} partial), {:.1}s",
            keep.len(),
            bake.probes.len(),
            sun_map.depth.size,
            span * 1.15,
            sun_map.depth.texel,
            casters.len(),
            casters.iter().filter(|c| c.partial).count(),
            started.elapsed().as_secs_f32()
        ));
        Ok(Lit { bake, keep, sun_map, tracer, sun, sun_light, exposure: exposure(), report })
    }

    /// The drawn probe a point reflects (an index into `keep`): the
    /// smallest kept box holding it, else the nearest kept box within its
    /// blend, else the map-wide one.
    pub fn probe_for(&self, p: [f32; 3]) -> u8 {
        let mut best: Option<(f32, f32, usize)> = None;
        for (k, &i) in self.keep.iter().enumerate().skip(1) {
            let pr = &self.bake.probes[i];
            let o = pr.outside(p);
            if o > 96.0 {
                continue;
            }
            let key = (o, pr.volume());
            if best.is_none_or(|(bo, bv, _)| key < (bo, bv)) {
                best = Some((key.0, key.1, k));
            }
        }
        best.map_or(0, |b| b.2 as u8)
    }

    /// How open a point is (0..1): short rays over the hemisphere round
    /// `n`, softened (the probes already hold the room's light).
    fn occlusion(&self, p: [f32; 3], n: [f32; 3], seed: u32, reach: f32) -> f32 {
        0.35 + 0.65 * self.tracer.openness(p, n, reach, 12, seed, 1.0)
    }

    /// Each world vertex's ambient light (probes' unit, occlusion in), as
    /// the vertex colour (alpha kept from `colours`).
    pub fn world_colours(&self, map: &T7Map, colours: &[[f32; 4]]) -> Vec<[f32; 4]> {
        fastfile_t7::bake::parallel_map(map.vertices.len(), |i| {
            let v = &map.vertices[i];
            let n = v.normal;
            let a = self.bake.ambient_lit(v.xyz, n);
            let o = self.occlusion(v.xyz, n, i as u32, 128.0);
            let alpha = colours.get(i).map_or(1.0, |c| c[3]);
            [a[0] * o, a[1] * o, a[2] * o, alpha]
        })
    }

    /// A placed model's light (`packed_lighting`: sqrt(light / 32)): the
    /// mean ambient light at its origin, its occlusion seen from just
    /// above it.
    pub fn prop_light(&self, origin: [f32; 3], seed: u32) -> [u8; 4] {
        let a = self.bake.ambient_mean(origin);
        let up = [origin[0], origin[1], origin[2] + 8.0];
        let o = self.occlusion(up, [0.0, 0.0, 1.0], seed, 256.0);
        let p = |v: f32| (((v * o).max(0.0) / 32.0).sqrt() * 255.0).round().clamp(1.0, 255.0) as u8;
        [p(a[0]), p(a[1]), p(a[2]), 0]
    }

    /// The light rows (`asset_world::T7_LIGHT_ROW`): exposure, the sun's
    /// light over the sun colour drawn (`drawn_sun`), kind 7; then the sun
    /// map's rows.
    pub fn light_rows(&self, drawn_sun: f32) -> [[f32; 16]; 2] {
        let m = self.sun_map.rows();
        let mut a = [0f32; 16];
        a[0] = self.exposure;
        a[1] = self.sun_light / drawn_sun.max(1e-3);
        a[3] = 7.0;
        for k in 0..3 {
            a[4 + 4 * k..8 + 4 * k].copy_from_slice(&m[k]);
        }
        let mut b = [0f32; 16];
        b[..4].copy_from_slice(&m[3]);
        b[4] = view();
        b[7] = 7.0;
        [a, b]
    }

    /// The probe table rows: each drawn probe's gain (its light over the
    /// map-wide probe's).
    pub fn probe_rows(&self) -> Vec<[[f32; 4]; 3]> {
        self.keep
            .iter()
            .map(|&i| {
                let g = self.bake.gain(i);
                [[g, g, g, 0.0], [0.0, 0.0, 0.0, 1.0], [0.0; 4]]
            })
            .collect()
    }

    /// The sun map as the renderer's colour-table slot.
    pub fn sun_map_lut(&self) -> asset_world::T6Lut {
        let s = self.sun_map.depth.size;
        asset_world::T6Lut { width: s, height: s, rgba: self.sun_map.texels() }
    }

    /// Links the sky (a cube from the picture, in the probes' unit) and the
    /// drawn probes' cubes into the catalog.
    pub fn link(&self, map: &T7Map, name: &str, catalog: &mut MaterialCatalog, owner: ZoneOwner) -> Vec<String> {
        let mut report = Vec::new();
        match self.bake.sky.as_ref() {
            Some(sky) => {
                let gain = self.bake.sky_gain();
                let cube = fastfile_t7::light::sky_cube(sky, SKY_FACE);
                let mut data = Vec::with_capacity(cube.len() * 8);
                for c in &cube {
                    for k in 0..4 {
                        let v = if k == 3 { 1.0 } else { c[k] * gain };
                        data.extend_from_slice(&super::t7_map::f16_bits(v).to_le_bytes());
                    }
                }
                let mut image = Image::new_uninit(
                    Extent3d { width: SKY_FACE, height: SKY_FACE, depth_or_array_layers: 6 },
                    TextureDimension::D2,
                    TextureFormat::Rgba16Float,
                    RenderAssetUsages::RENDER_WORLD,
                );
                image.data = Some(data);
                image.texture_view_descriptor = Some(TextureViewDescriptor { dimension: Some(TextureViewDimension::Cube), ..Default::default() });
                image.sampler = clamped();
                let slot = link_image(catalog, format!("t7:{name}#sky"), SKY_FACE, 1, TS_COLOR_MAP, image);
                catalog.link_material(material(format!("t7:{name}#sky"), TS_COLOR_MAP, slot, owner, Some(T6Draw::from_technique_set("wc_sky", 0, None))));
                report.push(format!("t7 light: sky {SKY_FACE}px cube from the picture, times {gain:.5}"));
            }
            None => report.push("t7 light: no sky picture; the sky is not drawn".to_owned()),
        }
        match fastfile_t7::light::probe_array(map, &self.keep) {
            Ok(arr) if arr.format == 95 || arr.format == 96 => {
                let mut image = Image::new_uninit(
                    Extent3d { width: arr.size, height: arr.size, depth_or_array_layers: 6 * self.keep.len() as u32 },
                    TextureDimension::D2,
                    if arr.format == 95 { TextureFormat::Bc6hRgbUfloat } else { TextureFormat::Bc6hRgbFloat },
                    RenderAssetUsages::RENDER_WORLD,
                );
                image.texture_descriptor.mip_level_count = arr.levels;
                image.texture_view_descriptor = Some(TextureViewDescriptor { dimension: Some(TextureViewDimension::CubeArray), ..Default::default() });
                image.data = Some(arr.data);
                image.sampler = clamped();
                let semantic = super::t6_materials::T6_PROBES_SEMANTIC;
                let slot = link_image(catalog, format!("t7:{name}#probes"), arr.size, arr.levels, semantic, image);
                catalog.link_material(material(super::t6_materials::T6_PROBES_MATERIAL.to_owned(), semantic, slot, owner, None));
                report.push(format!("t7 light: {} probe cubes ({}px, {} levels)", self.keep.len(), arr.size, arr.levels));
            }
            Ok(arr) => report.push(format!("t7 light: probes: format {} not drawn", arr.format)),
            Err(e) => report.push(format!("t7 light: probes: {e}")),
        }
        report
    }
}

fn clamped() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..Default::default()
    })
}

fn link_image(catalog: &mut MaterialCatalog, name: String, size: u32, levels: u32, semantic: u8, image: Image) -> usize {
    catalog.link_image(AuthoredImage {
        namespace: AssetNamespace::T6,
        name: AssetRef::Real(name),
        map_type: 5,
        semantic,
        category: 0,
        use_srgb_reads: false,
        width: size.min(u32::from(u16::MAX)) as u16,
        height: size.min(u32::from(u16::MAX)) as u16,
        depth: 1,
        level_count: levels.min(255) as u8,
        format: 0,
        payload: Arc::new(Vec::new()),
        decoded: Some(Arc::new(image)),
        common_owned: false,
        decoded_variant: None,
        decoded_by: None,
        pending_decode: None,
    })
}

fn material(name: String, semantic: u8, slot: usize, owner: ZoneOwner, t6_draw: Option<T6Draw>) -> AuthoredMaterial {
    AuthoredMaterial {
        name: AssetRef::Real(name),
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
        textures: vec![MaterialTextureBinding { name_hash: 0, name_start: 0, name_end: 0, sampler_state: 0x13, semantic, image: Some(slot) }],
        constants: Vec::new(),
        zone: owner,
        t6_draw,
    }
}

/// A blank lightmap page (white, the lightmapped pipelines' input), so the
/// world's lit surfaces draw with their normal and specular maps and probe.
pub(super) fn blank_page() -> asset_world::WorldLightmap {
    use bevy::prelude::UVec2;
    let one = || Image::new(Extent3d { width: 1, height: 1, depth_or_array_layers: 1 }, TextureDimension::D2, vec![255; 4], TextureFormat::Rgba8Unorm, RenderAssetUsages::RENDER_WORLD);
    let page = Image::new(Extent3d { width: 1, height: 3, depth_or_array_layers: 1 }, TextureDimension::D2, vec![255; 12], TextureFormat::Rgba8Unorm, RenderAssetUsages::RENDER_WORLD);
    asset_world::WorldLightmap {
        primary_image: None,
        secondary_image: Some(page),
        secondary_b_image: None,
        ambient_image: one(),
        directional_image: one(),
        sun_mask_image: one(),
        ambient_source_name: "t7 light".to_owned(),
        sun_mask_source_name: "t7 light".to_owned(),
        ambient_size: UVec2::new(1, 3),
        sun_mask_size: UVec2::new(1, 1),
    }
}
