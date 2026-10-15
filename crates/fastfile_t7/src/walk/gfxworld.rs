//! `GfxWorld` (0x2040): the drawn world. Layout from x64-zt's T7
//! `GfxWorld` (offsets checked by its asserts), record sizes and the
//! stream order read on retail `mp_nuketown_x` (file offsets: there is no
//! padding in the file, so a wrong size shows at once):
//!
//! * the stream info's AABB trees (48) and leaf refs (4);
//! * the sun volumes (4536 each), then per volume: the four multi-res
//!   shadow trees (`GfxStreamedShadowTreeBuffer`, 144 on retail: its mip
//!   offsets and counts, the elements' name, their data when resident, the
//!   fallback buffer, 56, and its elements), the four min/max trees (CPU
//!   u16s, then the GPU image), the light indices, the four reflection probe
//!   arrays (the probe image first, then `count + 1` configs of 328 and the
//!   blends, 604, then the pixel data's `StreamWrappedBuffer`, 64), the four
//!   probe volumes (three images and the pixel data), the four sky boxes
//!   (the model is a reference; `GfxSkyBoxImage`, 72: image and pixels)
//!   and the four exposure grids (probes of 80, then per probe its voxels,
//!   12 bytes each, and its blends, 80, each with its faces);
//! * the volumes and their planes (outdoor 424 with four shadow trees, fog
//!   624, LUT 60, weather grime 120 with six images, generic 12);
//! * the DPVS planes (20) and nodes (u32), the cells (56) with their AABB
//!   trees (48, each with its static model indices);
//! * the draw: the cookie image array (an inserted image), the positions
//!   (12 per vertex), the vertex attributes (20 per vertex) and the indices,
//!   all three in VIRTUAL;
//! * brush models (64), material memory (16), the sorted surface indices,
//!   the semi- and fully static model lists (u16), the surfaces (96, as
//!   many as `surfaceCount`), the static model draw instances (152);
//! * the LUT material (inserted), the umbra tomes, lens pack, lit fog
//!   volumes and their planes, volume decals (184) and the decal reveal
//!   image.

use super::world::{Span, span};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::{PHYSICAL, Stream};

pub const SURFACE: usize = 96;
pub const SMODEL_DRAW: usize = 152;
pub const BRUSH_MODEL: usize = 64;
const SUN_VOLUME: usize = 4536;
const SHADOW_TREE: usize = 144;
const SHADOW_TREE_FALLBACK: usize = 56;
const STREAM_WRAPPED: usize = 64;
const PROBE_CONFIG: usize = 328;
const PROBE_BLEND: usize = 604;
const EXPOSURE_PROBE: usize = 80;
const EXPOSURE_BLEND: usize = 80;
const OUTDOOR_VOLUME: usize = 424;
const FOG_VOLUME: usize = 624;
const LUT_VOLUME: usize = 60;
const EXPOSURE_VOLUME: usize = 108;
const VOLUME_DECAL: usize = 184;

/// The post-effect lists at the end of the `GfxWorld` header: (count's
/// offset, record size); the pointer follows the count. Vukko (boxes 144:
/// `zm_factory`'s seventeen), bloom (modifiers
/// 384, boxes 464), the LUT (boxes 144), colour (modifiers 336, boxes 688),
/// underwater (boxes 164: `mp_waterpark`'s two).
const POST_FX_LISTS: &[(usize, Option<usize>)] = &[
    (0x1610, None),
    (0x1620, Some(144)),
    (0x1630, Some(144)),
    (0x1b40, Some(384)),
    (0x1b50, Some(464)),
    (0x1b60, Some(464)),
    (0x1b80, None),
    (0x1b90, Some(144)),
    (0x1ba0, Some(144)),
    (0x1ff0, Some(336)),
    (0x2000, Some(688)),
    (0x2010, Some(688)),
    (0x2020, Some(164)),
    (0x2030, Some(164)),
];
const GRIME_VOLUME: usize = 120;
const GENERIC_VOLUME: usize = 12;
const CELL: usize = 56;
const AABB_TREE: usize = 48;

/// What the walk keeps of a `GfxWorld`.
#[derive(Clone, Debug, Default)]
pub struct GfxWorldRec {
    pub name: String,
    pub header_at: usize,
    pub vertex_count: usize,
    /// Positions (12 bytes a vertex) and attributes (20 bytes a vertex).
    pub positions: Option<(usize, usize)>,
    pub attributes: Option<(usize, usize)>,
    pub indices: Span,
    pub surfaces: Span,
    /// Per surface: its material's name.
    pub surface_materials: Vec<Option<String>>,
    pub smodels: Span,
    /// Per static model draw instance: its model's name.
    pub smodel_models: Vec<Option<String>>,
    pub brush_models: Span,
    /// The sky boxes' models (first sun volume).
    pub sky_models: Vec<String>,
    /// The sun's settings (first sun volume's first state): file offset.
    pub sun_at: usize,
    /// bo2mp: the sun volumes' file offsets (4536 each).
    pub sun_volumes: Vec<usize>,
    /// bo2mp: per sun volume, its lighting as the walk met it.
    pub sun_lighting: Vec<SunVolumeLighting>,
    /// bo2mp: the fog volumes (624 each), LUT volumes (60), exposure
    /// volumes (108), lit fog volumes (360): (file offset, count).
    pub fog_volumes: Span,
    pub lut_volumes: Span,
    pub exposure_volumes: Span,
    pub lit_fog_volumes: Span,
    /// bo2mp: the LUT material's name (0x520..0x538: the four materials).
    pub header_materials: Vec<Option<String>>,
}

/// bo2mp: what a sun volume holds for lighting: its four reflection probe
/// arrays (image, configs, blends), its four probe volumes (three images
/// each) and its four sky boxes' images, by name and file offset.
#[derive(Clone, Debug, Default)]
pub struct SunVolumeLighting {
    /// (image, configs (file offset, count + 1), blends (file offset, count)).
    pub reflection_probes: Vec<(Option<String>, Span, Span)>,
    pub probe_volumes: Vec<[Option<String>; 3]>,
    pub sky_images: Vec<Option<String>>,
    /// The exposure grids' probes (file offset, count).
    pub exposure_grids: Vec<Span>,
    /// The `StreamWrappedBuffer`s (64 bytes: file offsets) of the four
    /// reflection probe arrays, the four probe volumes and the four sky
    /// box images: where their pixels are.
    pub probe_buffers: Vec<Option<usize>>,
    pub volume_buffers: Vec<Option<usize>>,
    pub sky_buffers: Vec<Option<usize>>,
}

fn count(s: &Stream, at: usize) -> Result<usize> {
    Ok(s.u32_at(at)? as usize)
}

/// A `StreamWrappedBuffer` at `b` (64): its name, and its data when
/// resident (mode 0); streamed data is in the `.xpak`.
pub(crate) fn stream_wrapped_body(s: &mut Stream, b: usize) -> Result<()> {
    s.no_loads(b, STREAM_WRAPPED, &[0x28, 0x38])?;
    s.string(b + 0x28)?;
    if s.u8_at(b + 0x31)? == 0 {
        let n = count(s, b + 0x34)?;
        s.push(PHYSICAL);
        let r = s.array(b + 0x38, n, 255);
        s.pop();
        r?;
    }
    Ok(())
}

pub(crate) fn stream_wrapped(s: &mut Stream, field: usize) -> Result<Option<usize>> {
    let b = s.array(field, STREAM_WRAPPED, 7)?;
    if let Some(b) = b {
        stream_wrapped_body(s, b)?;
    }
    Ok(b)
}

/// `GfxStreamedShadowTreeBuffer*` at `field`.
fn shadow_tree(s: &mut Stream, field: usize) -> Result<()> {
    let Some(t) = s.array(field, SHADOW_TREE, 15)? else { return Ok(()) };
    s.no_loads(t, SHADOW_TREE, &[0x28, 0x38, 0x40, 0x48, 0x88])?;
    let mips = count(s, t + 0x54)?;
    s.records(t + 0x40, mips, 4, 3)?;
    s.records(t + 0x48, mips, 4, 3)?;
    stream_wrapped_body(s, t)?;
    // (The fallback is 16-aligned: a later tree's reference to it lands so.)
    if let Some(f) = s.array(t + 0x88, SHADOW_TREE_FALLBACK, 15)? {
        s.no_loads(f, SHADOW_TREE_FALLBACK, &[0])?;
        let n = count(s, f + 8)? * count(s, f + 12)?;
        s.array(f, n, 3)?;
    }
    Ok(())
}

fn image(s: &mut Stream, field: usize) -> Result<Option<String>> {
    s.asset_named(AssetType::Image, field)
}

fn sun_volume(s: &mut Stream, v: usize, rec: &mut GfxWorldRec, first: bool) -> Result<()> {
    for k in 0..4 {
        shadow_tree(s, v + 1056 + 112 * k + 104)?;
    }
    for k in 0..4 {
        let m = v + 1504 + 48 * k;
        let n = count(s, m + 16)?;
        s.records(m + 24, n, 2, 1)?;
        image(s, m + 32)?;
    }
    let n = count(s, v + 1696)?;
    s.records(v + 1704, n, 2, 1)?;
    let mut light = SunVolumeLighting::default();
    for k in 0..4 {
        let r = v + 1712 + 40 * k;
        let im = s.asset_named(AssetType::Image, r + 8)?;
        let n = count(s, r + 32)?;
        let configs = span(s, r + 16, n + 1, PROBE_CONFIG, 15)?;
        let n = count(s, r + 36)?;
        let blends = span(s, r + 24, n, PROBE_BLEND, 15)?;
        light.probe_buffers.push(stream_wrapped(s, r)?);
        light.reflection_probes.push((im, configs, blends));
    }
    for k in 0..4 {
        let p = v + 1872 + 32 * k;
        let mut names: [Option<String>; 3] = Default::default();
        for (j, n) in names.iter_mut().enumerate() {
            *n = s.asset_named(AssetType::Image, p + 8 * j)?;
        }
        light.volume_buffers.push(stream_wrapped(s, p + 24)?);
        light.probe_volumes.push(names);
    }
    for k in 0..4 {
        let b = v + 2000 + 24 * k;
        if let Some(m) = s.asset_named(AssetType::XModel, b)?
            && first
        {
            rec.sky_models.push(m);
        }
        let mut sky = None;
        let mut buffer = None;
        if let Some(i) = s.array(b + 16, 72, 7)? {
            sky = s.asset_named(AssetType::Image, i)?;
            stream_wrapped_body(s, i + 8)?;
            buffer = Some(i + 8);
        }
        light.sky_buffers.push(buffer);
        light.sky_images.push(sky);
    }
    for k in 0..4 {
        let g = v + 2096 + 16 * k;
        let n = count(s, g)?;
        let t = s.records(g + 8, n, EXPOSURE_PROBE, 7)?;
        light.exposure_grids.push(t.map(|t| (t, n)));
        let Some(t) = t else { continue };
        for i in 0..n {
            let e = t + EXPOSURE_PROBE * i;
            let voxels = s.u16_at(e + 40)? as usize * s.u16_at(e + 42)? as usize * s.u16_at(e + 44)? as usize;
            s.records(e + 48, voxels, 12, 1)?;
            let blends = s.u16_at(e + 56)? as usize;
            if let Some(b) = s.records(e + 64, blends, EXPOSURE_BLEND, 7)? {
                for j in 0..blends {
                    let faces = count(s, b + EXPOSURE_BLEND * j + 60)?;
                    s.records(b + EXPOSURE_BLEND * j + 64, faces, 16, 3)?;
                }
            }
        }
    }
    rec.sun_volumes.push(v);
    rec.sun_lighting.push(light);
    Ok(())
}

/// `planes` (vec4 each) of a volume list.
fn planes(s: &mut Stream, h: usize, count_at: usize) -> Result<()> {
    let n = count(s, h + count_at)?;
    s.records(h + count_at + 8, n, 16, 3)?;
    Ok(())
}

pub fn gfx_world(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let name = s.string(h)?;
    s.string(h + 8)?;
    let mut rec = GfxWorldRec { name: name.clone().unwrap_or_default(), header_at: h, ..Default::default() };
    // Stream info.
    let n = count(s, h + 0x20)?;
    s.records(h + 0x28, n, 48, 15)?;
    let n = count(s, h + 0x30)?;
    s.records(h + 0x38, n, 4, 3)?;
    // Coronas (32 each).
    let n = count(s, h + 0xd8)?;
    s.records(h + 0xe0, n, 32, 3)?;
    // Sun volumes.
    let n = count(s, h + 0xe8)?;
    if let Some(t) = s.records(h + 0xf0, n, SUN_VOLUME, 15)? {
        rec.sun_at = t + 16;
        for i in 0..n {
            sun_volume(s, t + SUN_VOLUME * i, &mut rec, i == 0)?;
        }
    }
    planes(s, h, 0xf8)?;
    let n = count(s, h + 0x108)?;
    if let Some(t) = s.records(h + 0x110, n, OUTDOOR_VOLUME, 15)? {
        for i in 0..n {
            for k in 0..4 {
                shadow_tree(s, t + OUTDOOR_VOLUME * i + 8 + 104 * k + 88)?;
            }
        }
    }
    planes(s, h, 0x118)?;
    // Exposure volumes (`GfxExposureVolume`, 108; `mp_sector` has eight).
    let n = count(s, h + 0x128)?;
    rec.exposure_volumes = span(s, h + 0x130, n, EXPOSURE_VOLUME, 3)?;
    planes(s, h, 0x138)?;
    let n = count(s, h + 0x148)?;
    rec.fog_volumes = span(s, h + 0x150, n, FOG_VOLUME, 15)?;
    planes(s, h, 0x158)?;
    let n = count(s, h + 0x168)?;
    s.records(h + 0x170, n, 44, 15)?;
    planes(s, h, 0x178)?;
    let n = count(s, h + 0x188)?;
    rec.lut_volumes = span(s, h + 0x190, n, LUT_VOLUME, 3)?;
    planes(s, h, 0x198)?;
    let n = count(s, h + 0x1a8)?;
    if let Some(t) = s.records(h + 0x1b0, n, GRIME_VOLUME, 7)? {
        for i in 0..n {
            for j in 0..6 {
                image(s, t + GRIME_VOLUME * i + 72 + 8 * j)?;
            }
        }
    }
    planes(s, h, 0x1b8)?;
    let n = count(s, h + 0x1c8)?;
    s.records(h + 0x1d0, n, GENERIC_VOLUME, 3)?;
    planes(s, h, 0x1d8)?;
    // (x64-zt's `unk` list: 48 bytes each; `mp_banzai`'s one puts its
    // DPVS cell 48 bytes on.)
    let n = count(s, h + 0x1e8)?;
    s.records(h + 0x1f0, n, 48, 3)?;
    planes(s, h, 0x1f8)?;
    let n = count(s, h + 0x208)?;
    s.records(h + 0x210, n, 144, 15)?;
    // DPVS planes and nodes (the scene entities' cell bits are runtime).
    let n = count(s, h + 0x10)?;
    s.records(h + 0x230, n, 20, 3)?;
    let n = count(s, h + 0x14)?;
    s.records(h + 0x238, n, 4, 3)?;
    let cells = count(s, h + 0x228)?;
    if let Some(t) = s.records(h + 0x250, cells, CELL, 7)? {
        for c in 0..cells {
            let cell = t + CELL * c;
            let n = count(s, cell + 24)?;
            if let Some(trees) = s.records(cell + 32, n, AABB_TREE, 7)? {
                for i in 0..n {
                    let a = trees + AABB_TREE * i;
                    let k = s.u16_at(a + 30)? as usize;
                    s.records(a + 32, k, 2, 1)?;
                }
            }
            let portals = count(s, cell + 40)?;
            if portals > 0 && s.ptr_at(cell + 48)?.is_inline() {
                return Err(crate::cursor::ReadError::Invalid("gfxworld cell portals (not read yet)", cell + 48));
            }
        }
    }
    // The draw.
    image(s, h + 0x260)?;
    rec.vertex_count = count(s, h + 0x268)?;
    // (The positions, attributes and indices are in VIRTUAL: the later
    // references land so.)
    for (field, size_at, slot) in [(0x270, 0x26c, 0), (0x288, 0x280, 1)] {
        let n = count(s, h + size_at)?;
        let at = s.array(h + field, n, 15)?.map(|at| (at, n));
        if slot == 0 {
            rec.positions = at;
        } else {
            rec.attributes = at;
        }
    }
    let n = count(s, h + 0x298)?;
    rec.indices = s.array(h + 0x2a0, 2 * n, 15)?.map(|at| (at, n));
    let n = count(s, h + 0x2b0)?;
    rec.brush_models = span(s, h + 0x2b8, n, BRUSH_MODEL, 3)?;
    let n = count(s, h + 0x2dc)?;
    if let Some(t) = s.records(h + 0x2e0, n, 16, 7)? {
        for i in 0..n {
            s.asset(AssetType::Material, t + 16 * i)?;
        }
    }
    // DPVS (the vis arrays are runtime).
    let surfaces = count(s, h + 0x18)?;
    let static_surfaces = count(s, h + 0x300)?;
    s.records(h + 0x408, static_surfaces, 2, 1)?;
    let n = count(s, h + 0x418)?;
    s.records(h + 0x420, n, 2, 1)?;
    let n = count(s, h + 0x41c)?;
    s.records(h + 0x428, n, 2, 1)?;
    rec.surfaces = span(s, h + 0x430, surfaces, SURFACE, 7)?;
    if let Some((t, n)) = rec.surfaces {
        for i in 0..n {
            rec.surface_materials.push(s.asset_named(AssetType::Material, t + SURFACE * i + 72)?);
        }
    }
    let smodels = count(s, h + 0x2fc)?;
    rec.smodels = span(s, h + 0x438, smodels, SMODEL_DRAW, 7)?;
    if let Some((t, n)) = rec.smodels {
        for i in 0..n {
            let d = t + SMODEL_DRAW * i;
            rec.smodel_models.push(s.asset_named(AssetType::XModel, d + 88)?);
            // Posed bones (`GfxStaticModelBone`, 64: a 3x4 matrix and one
            // more vec4; count at 128), then the siege animation (an asset).
            let bones = count(s, d + 128)?;
            s.records(d + 120, bones, 64, 15)?;
            s.asset(AssetType::SAnim, d + 136)?;
        }
    }
    s.capture.world.gfx_worlds.push(rec);
    let at = s.capture.world.gfx_worlds.len() - 1;
    // Materials, umbra, lenses, lit fog, decals.
    for f in [0x520, 0x528, 0x530, 0x538] {
        let m = s.asset_named(AssetType::Material, h + f)?;
        s.capture.world.gfx_worlds[at].header_materials.push(m);
    }
    let n = count(s, h + 0x540)?;
    s.records(h + 0x548, n, 68, 3)?;
    // The two umbra tome lists (0x558/0x560, 0x56c/0x570), the siege
    // animations (count at 0x578, `mp_sector`'s seven; asset pointers, 8
    // each) and those that play by themselves (0x588: `SiegeAutoPlay`, 20
    // each).
    // (The second list's count is the u32 at 0x56c: the one at 0x568 is 1
    // on `zm_factory`, `zm_genesis` and `zm_island` alike, whose lists
    // hold none, eleven and thirteen.)
    for (count_at, field) in [(0x558, 0x560), (0x56c, 0x570)] {
        umbra_tomes(s, h + count_at, h + field)?;
    }
    let n = count(s, h + 0x578)?;
    if let Some(t) = s.records(h + 0x580, n, 8, 7)? {
        for i in 0..n {
            s.asset(AssetType::SAnim, t + 8 * i)?;
        }
    }
    let n = count(s, h + 0x588)?;
    s.records(h + 0x590, n, 20, 3)?;
    let n = count(s, h + 0x598)?;
    s.records(h + 0x5a0, n, 84, 3)?;
    // Lit fog volumes (360: planeStart and planeCount first, then x64-zt's
    // `GfxLitFogVolume`) and their planes (the 0x5c0 array: vec4s).
    let n = count(s, h + 0x5a8)?;
    let lit_fog = span(s, h + 0x5b0, n, 360, 15)?;
    s.capture.world.gfx_worlds[at].lit_fog_volumes = lit_fog;
    planes(s, h, 0x5b8)?;
    // Volume decals (`GfxVolumeDecal`, 184, the material at 152:
    // `mp_nuketown_x`'s twelve).
    let n = count(s, h + 0x5c8)?;
    if let Some(t) = s.records(h + 0x5d0, n, VOLUME_DECAL, 7)? {
        for i in 0..n {
            s.asset(AssetType::Material, t + VOLUME_DECAL * i + 152)?;
        }
    }
    image(s, h + 0x5d8)?;
    // Water (`GfxWorldWaterDisk` at 0x5e0): the interactive waves (count at
    // 0x5e4, 192 each, `mp_apartments`' one) and the grid memory (0x5f0
    // bytes at 0x5f8: runtime, no file bytes).
    let n = count(s, h + 0x5e4)?;
    if let Some(t) = s.records(h + 0x5e8, n, INTERACTIVE_WATER, 7)? {
        for i in 0..n {
            interactive_water(s, t + INTERACTIVE_WATER * i)?;
        }
    }
    let n = count(s, h + 0x5f0)?;
    s.push(crate::stream::RUNTIME_VIRTUAL);
    let r = s.array(h + 0x5f8, n, 15);
    s.pop();
    r?;
    // The post-effect volumes: per effect its base settings (inline in the
    // header), then (count, records) lists of modifiers ("Mod") and of
    // boxes (six planes, settings, a name) that override or set them.
    // Sizes read on `mp_crucible` and `mp_stronghold` (the records fill the
    // file up to the next asset); `None`: not seen loaded yet.
    s.no_loads(h + 0x600, 0x1610 - 0x600, &[])?;
    s.no_loads(h + 0x1640, 0x1b40 - 0x1640, &[])?;
    for &(count_at, size) in POST_FX_LISTS {
        let n = count(s, h + count_at)?;
        match size {
            Some(size) => {
                s.records(h + count_at + 8, n, size, 15)?;
            }
            None if n > 0 && s.ptr_at(h + count_at + 8)?.is_inline() => {
                return Err(crate::cursor::ReadError::Invalid("gfxworld post-effect list (size not read yet; its data at)", s.pos()));
            }
            None => {}
        }
    }
    s.no_loads(h + 0x1bb0, 0x1ff0 - 0x1bb0, &[])?;
    Ok(name)
}

/// An interactive wave water (192: bounds, 0x1c u32 count and 0x20 cells,
/// 40 each; 0x28 u32 count and 0x30 outlines, 32 each with their points,
/// 8 each, at 0; 0x38 u32 count and 0x40 wave clamps; 0x48 a reference;
/// 0x50 its name; then settings). `mp_apartments`, `mp_metro`.
const INTERACTIVE_WATER: usize = 192;

fn interactive_water(s: &mut Stream, w: usize) -> Result<()> {
    s.no_loads(w, INTERACTIVE_WATER, &[0x20, 0x30, 0x40, 0x48, 0x50, 0x90])?;
    let n = count(s, w + 0x1c)?;
    s.records(w + 0x20, n, 40, 3)?;
    let n = count(s, w + 0x28)?;
    if let Some(t) = s.records(w + 0x30, n, 32, 7)? {
        for i in 0..n {
            let k = count(s, t + 32 * i + 8)?;
            s.records(t + 32 * i, k, 8, 3)?;
        }
    }
    // Wave clamps (count at 0x38, 48 each: their outline's points, 8
    // each, at 8 with the count at 0x10; their name at 0x28).
    let n = count(s, w + 0x38)?;
    if let Some(t) = s.records(w + 0x40, n, 48, 7)? {
        for i in 0..n {
            let r = t + 48 * i;
            let k = count(s, r + 0x10)?;
            s.records(r + 8, k, 8, 3)?;
            s.string(r + 0x28)?;
        }
    }
    if s.ptr_at(w + 0x48)?.is_inline() {
        return Err(crate::cursor::ReadError::Invalid("interactive water field 0x48 (not read yet)", w + 0x48));
    }
    s.string(w + 0x50)?;
    // Splines or emitters (count at 0x88, 72 each, their name at 0x40;
    // `mp_banzai`'s river has five).
    let n = count(s, w + 0x88)?;
    if let Some(t) = s.records(w + 0x90, n, 72, 3)? {
        for i in 0..n {
            s.string(t + 72 * i + 0x40)?;
        }
    }
    Ok(())
}

/// `GfxUmbraTome` lists (40 each: the tome's bytes, then its parameter
/// volumes, 40 each).
fn umbra_tomes(s: &mut Stream, count_at: usize, field: usize) -> Result<()> {
    let n = count(s, count_at)?;
    if let Some(t) = s.records(field, n, 40, 7)? {
        for i in 0..n {
            let u = t + 40 * i;
            let size = count(s, u)?;
            s.array(u + 8, size, 15)?;
            let k = count(s, u + 0x18)?;
            s.records(u + 0x20, k, 40, 3)?;
        }
    }
    Ok(())
}
