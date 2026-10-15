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
/// What Black Ops III's tone curve's input is: light in the probes' unit
/// times this (`BO2MP_T7_EXPOSURE`). BO3's own is its auto-exposure's
/// (histograms and exposure volumes: not drawn); this is chosen against
/// pictures of the game.
const EXPOSURE: f32 = 1.6;
/// How far, round the spawn points' square, the sun map and the occlusion
/// reach.
const MARGIN: f32 = 3000.0;

/// A test view (`BO2MP_T7_VIEW`): 1 the sun's visibility, 2 the ambient
/// light, 3 the albedo, 4 the light (no albedo); 0 the picture.
pub(super) fn view() -> f32 {
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
    /// The fog's rows (`T7_FOG_ROW`), when the map has fog.
    pub fog: Option<[[f32; 4]; 8]>,
    pub report: Vec<String>,
}

/// bo2mp: the first of the two light rows the fog's eight vec4s sit in
/// (`T7_FOG` in the shader).
pub(super) const T7_FOG_ROW: usize = 252;

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
        let fog = Fog::read(map, &bake, sun, spawns);
        match fog.as_ref() {
            Some(f) => report.push(f.report.clone()),
            None => report.push("t7 fog: none (no fog volume with atmospheric settings, or BO2MP_T7_FOG=0)".to_owned()),
        }
        Ok(Lit { bake, keep, sun_map, tracer, sun, sun_light, exposure: exposure(), fog: fog.map(|f| f.rows), report })
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
    /// the vertex colour, its alpha kept from `colours`; on a reveal decal
    /// (`reveals`: map material, its soft edge and ramp) the alpha carries
    /// the soft edge too (`reveal_alpha`).
    pub fn world_colours(&self, map: &T7Map, colours: &[[f32; 4]], reveals: &std::collections::HashMap<usize, [f32; 2]>) -> Vec<[f32; 4]> {
        let mut soft = vec![0f32; map.vertices.len()];
        for s in &map.surfaces {
            if let Some(r) = s.material.and_then(|m| reveals.get(&m)) {
                let a = s.first_vertex as usize;
                let b = (a + s.vertex_count as usize).min(soft.len());
                if a < b {
                    soft[a..b].fill(r[0]);
                }
            }
        }
        fastfile_t7::bake::parallel_map(map.vertices.len(), |i| {
            let v = &map.vertices[i];
            let n = v.normal;
            let a = self.bake.ambient_lit(v.xyz, n);
            let o = self.occlusion(v.xyz, n, i as u32, 128.0);
            let alpha = colours.get(i).map_or(1.0, |c| c[3]);
            [a[0] * o, a[1] * o, a[2] * o, if soft[i] > 0.0 { reveal_alpha(alpha, soft[i]) } else { alpha }]
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
    /// map-wide probe's) and where it was taken; its parallax box's centre
    /// and its turn about z (cosine), its half size and the turn's sine
    /// (half size 0: no parallax, the map-wide probe). `BO2MP_T7_PARALLAX=0`
    /// leaves the parallax out.
    pub fn probe_rows(&self) -> Vec<[[f32; 4]; 3]> {
        let parallax = std::env::var("BO2MP_T7_PARALLAX").map_or(true, |v| v.trim() != "0");
        self.keep
            .iter()
            .enumerate()
            .map(|(k, &i)| {
                let g = self.bake.gain(i);
                let p = &self.bake.probes[i];
                let (c, s) = (p.axes[0][0], p.axes[0][1]);
                let flat = (p.axes[2][2] - 1.0).abs() < 0.01 && (c * c + s * s - 1.0).abs() < 0.02;
                let on = parallax && k > 0 && flat && p.box_half.iter().all(|h| *h > 1.0 && *h < 20000.0) && p.box_centre.iter().all(|v| v.is_finite());
                let h = if on { p.box_half } else { [0.0; 3] };
                [[g, p.origin[0], p.origin[1], p.origin[2]], [p.box_centre[0], p.box_centre[1], p.box_centre[2], c], [h[0], h[1], h[2], s]]
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

/// bo2mp: Black Ops III's atmospheric fog for the shader (`t7_fog_core` in
/// `geometry_diagnostic.wgsl`), from the map's fog volume: the one holding
/// most spawn points (else the map-wide one), lighting state 0. Its bank's
/// words as we read them (BO3 turns them into its shader constants in code
/// we cannot read; names ours, checked against how the maps look): 1 start
/// distance, 2 half distance (the transmittance halves every this many
/// units), 3 base height, 4 half height (the density halves every this many
/// units up), 16..18 the fog's (Rayleigh) colour, 19 extinction strength,
/// 20..22 the haze's (Mie) colour, 23 haze start, 24 haze spread, 25 the
/// haze's Schlick k (forward scattering), 26 its density, 27 its light in
/// stops (2^x, the sun's unit), 28 in-scatter strength, 30 sun strength
/// (the Rayleigh phase's share), 31 the sky's distance.
///
/// The fog's in-scatter colour is the bank's colour at the brightness of
/// the sky's horizon (a far wall fogs into the sky behind it); the haze's
/// is its colour at its light times its density. `BO2MP_T7_FOG` scales the
/// density (0: no fog), `BO2MP_T7_HAZE` the haze, `BO2MP_T7_SHAFTS` the sun
/// shafts (0: none).
pub(super) struct Fog {
    pub rows: [[f32; 4]; 8],
    pub report: String,
}

fn env_scale(name: &str, default: f32) -> f32 {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok()).filter(|v: &f32| v.is_finite() && *v >= 0.0).unwrap_or(default)
}

impl Fog {
    pub fn read(map: &T7Map, bake: &Bake, toward_sun: [f32; 3], spawns: &[[f32; 3]]) -> Option<Fog> {
        let density = env_scale("BO2MP_T7_FOG", 1.0);
        if density <= 0.0 {
            return None;
        }
        let vols = fastfile_t7::light::fog(map);
        // The volume most spawn points are in (ties: the higher priority).
        let mut best: Option<(usize, f32, usize)> = None;
        for (i, v) in vols.iter().enumerate() {
            if v.bank(0).is_none() {
                continue;
            }
            let n = v.holds(spawns);
            if best.is_none_or(|(bn, bp, _)| (n, v.priority) > (bn, bp)) {
                best = Some((n, v.priority, i));
            }
        }
        let (inside, _, vi) = best?;
        let b = vols[vi].bank(0)?;
        let lum = |c: [f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let norm = |c: [f32; 3]| {
            let l = lum(c).max(1e-4);
            c.map(|x| x.max(0.0) / l)
        };
        // The sky's horizon, in the probes' unit.
        let horizon = match bake.sky.as_ref() {
            Some(sky) => {
                let g = bake.sky_gain();
                let mut sum = [0f32; 3];
                for k in 0..64 {
                    let a = k as f32 / 64.0 * std::f32::consts::TAU;
                    let c = sky.at([a.cos(), a.sin(), 0.08]);
                    for j in 0..3 {
                        sum[j] += c[j] * g / 64.0;
                    }
                }
                lum(sum)
            }
            None => 1.0,
        };
        let units = bake.units();
        let fog_c = norm([b[16], b[17], b[18]]).map(|x| x * horizon);
        let haze_light = 2f32.powf(b[27].clamp(0.0, 24.0)) / units;
        let haze_c = norm([b[20], b[21], b[22]]).map(|x| x * haze_light * b[26].max(0.0) * env_scale("BO2MP_T7_HAZE", 1.0));
        let half = (b[2] / density).max(1.0);
        let ext = if b[19] > 0.0 { b[19].min(2.0) } else { 1.0 };
        let k = b[25].clamp(0.0, 0.95);
        let half_height = if b[4] > 1.0 { 1.0 / b[4] } else { 0.0 };
        let shafts = env_scale("BO2MP_T7_SHAFTS", 1.0);
        let rows = [
            [fog_c[0], fog_c[1], fog_c[2], b[23].max(0.0)],
            [haze_c[0], haze_c[1], haze_c[2], 1.0 / b[24].max(1.0)],
            [-1.0 / half, -1.0 / half, -1.0 / half, b[31].clamp(2000.0, 200000.0)],
            [toward_sun[0], toward_sun[1], toward_sun[2], k],
            [ext, b[28].max(0.0), b[30].clamp(0.0, 1.0), 1.0],
            [b[1].max(0.0), b[3], half_height, 0.0],
            [b[1].max(0.0), b[3], half_height, 0.0],
            [1.0 / 2000.0, 2500.0, if shafts > 0.0 { 16.0 } else { 0.0 }, shafts],
        ];
        let report = format!(
            "t7 fog: volume {vi} of {} ({inside} of {} spawns in it): start {:.0}, half {:.0}, base {:.0}, half height {:.0}, fog {:.3?} (horizon {horizon:.3}), haze {:.3?} from {:.0} over {:.0}, k {k:.2}, extinction {ext:.2}, in-scatter {:.2}, sun strength {:.2}, sky at {:.0}",
            vols.len(),
            spawns.len(),
            b[1],
            half,
            b[3],
            b[4],
            fog_c,
            haze_c,
            b[23],
            b[24],
            b[28],
            b[30],
            b[31]
        );
        Some(Fog { rows, report })
    }
}

/// bo2mp: the final pass's grade for a Black Ops III map lit by its own
/// light: its colour table (the `GfxWorld`'s LUT material's picture, a
/// `create_lut2dv` source like Black Ops II's: 32 tables of 32 x 32 side by
/// side per 32 rows, R10G10B10A2; the first band, which the map starts in)
/// and a bloom for its highlights (BO3's bloom settings are not read: its
/// post shaders are not in the zones read; the ramp and tints are ours).
/// `BO2MP_T7_GRADE=0`: no table; `BO2MP_T7_BLOOM` scales the bloom (0 none).
pub(super) fn grade(map: &T7Map, report: &mut Vec<String>) -> Option<asset_world::T6Grade> {
    let mut vision = asset_world::T6Vision::default();
    let bloom = env_scale("BO2MP_T7_BLOOM", 1.0);
    if bloom > 0.0 {
        vision.bloom_rgb_hi = [0.35 * bloom, 0.35 * bloom, 0.35 * bloom, 0.7];
        vision.bloom_rgb_lo = [0.45 * bloom, 0.45 * bloom, 0.45 * bloom, 3.0];
    }
    let mut base: Option<Vec<u8>> = None;
    if env_scale("BO2MP_T7_GRADE", 1.0) > 0.0
        && let Some(name) = fastfile_t7::light::lut_image(map)
    {
        match map.image(&name, 1024) {
            Ok(l) if l.format == 24 && l.width == 1024 && l.height >= 32 && l.data.len() >= 1024 * 32 * 4 => {
                let px: Vec<u8> = l.data[..1024 * 32 * 4]
                    .chunks_exact(4)
                    .flat_map(|w| {
                        let v = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
                        let c = |k: u32| ((((v >> (10 * k)) & 1023) as f32) * 255.0 / 1023.0).round() as u8;
                        [c(0), c(1), c(2), 255]
                    })
                    .collect();
                report.push(format!("t7 grade: {name} (band 0 of {} rows)", l.height));
                base = Some(px);
            }
            Ok(l) => report.push(format!("t7 grade: {name}: format {} {}x{} not read", l.format, l.width, l.height)),
            Err(e) => report.push(format!("t7 grade: {e}")),
        }
    }
    report.push(format!("t7 bloom: x{bloom}"));
    Some(asset_world::T6Grade { lut: asset_world::t6_grade_lut(&vision, base.as_deref()), vision })
}

/// A reveal decal vertex's alpha with its material's soft edge, for the
/// shader (`t7_reveal`): -(2 + 2 q + 0.999 a), q the soft edge in 64ths.
fn reveal_alpha(a: f32, soft: f32) -> f32 {
    let q = (soft * 64.0).round().clamp(1.0, 64.0);
    -(2.0 + 2.0 * q + 0.999 * a.clamp(0.0, 1.0))
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
