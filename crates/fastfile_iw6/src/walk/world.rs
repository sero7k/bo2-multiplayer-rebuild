//! Map assets.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;
use crate::walk::{ClipInfoRec, ClipMapRec, ComWorldRec, GfxWorldRec, MapEntsRec, Span};

pub(super) fn com_world(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ComWorld as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::name)?.unwrap_or_default();
    let n = w.s.u32_at(h + C::primaryLightCount)? as usize;
    let pl = L::ComPrimaryLight::SIZE;
    let mut primary_lights = None;
    if let Some(at) = w.inline_array(h + C::primaryLights, pl * n, 3)? {
        primary_lights = Some(Span {
            pos: at,
            len: pl * n,
        });
        for i in 0..n {
            w.xstr(at + pl * i + L::ComPrimaryLight::defName)?;
        }
    }
    let e = w.s.u32_at(h + C::primaryLightEnvCount)? as usize;
    w.blob(h + C::primaryLightEnvs, L::ComPrimaryLightEnv::SIZE * e, 3)?;
    w.s.pop();
    w.cap.com_worlds.push(ComWorldRec {
        name,
        record: Span {
            pos: h,
            len: C::SIZE,
        },
        primary_lights,
        primary_light_count: n as u32,
    });
    Ok(AssetId {
        ty: AssetType::ComWorld,
        index: (w.cap.com_worlds.len() - 1) as u32,
    })
}

/// `FxWorld`: its name and breakable-glass system. The glass definitions
/// (materials, physics, effects, sounds) are in the file; the pieces'
/// working arrays are RUNTIME allocations (no file data); the initial
/// piece states and geometry follow in VIRTUAL (x64-zt's `fx_world::write`
/// order).
///
/// A `FxGlassDef`'s sub-assets load physics preset first, then material,
/// then shattered material (fields as x64-zt lays them out): retail
/// mp_prisonbreak's four defs load `,glass_chunk`, `m/pbk_window_01`,
/// `m/com_glass_clear_shattered`, three effects and three sounds, and the
/// zone walks to its exact end; mp_strikezone's references resolve to a
/// material in `material` and the preset in `physPreset`.
pub(super) fn fx_world(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::FxGlassSystem as G;
    use L::FxWorld as F;
    use crate::stream::RUNTIME;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + F::name)?.unwrap_or_default();
    let g = h + F::glassSys;
    let u = |w: &Walker, off: usize| -> Result<usize> { Ok(w.s.u32_at(g + off)? as usize) };
    let defs = u(w, G::defCount)?;
    let piece_limit = u(w, G::pieceLimit)?;
    let words = u(w, G::pieceWordCount)?;
    let init_pieces = u(w, G::initPieceCount)?;
    let cells = u(w, G::cellCount)?;
    let geo_limit = u(w, G::geoDataLimit)?;
    let init_geo = u(w, G::initGeoDataCount)?;
    let d = L::FxGlassDef::SIZE;
    let trace = std::env::var_os("IW6_TRACE_FXWORLD").is_some();
    if trace {
        eprintln!("FXWORLD {name} at {h:#x}: defs {defs} pieceLimit {piece_limit} words {words} initPieces {init_pieces} cells {cells} geoLimit {geo_limit} initGeo {init_geo}; ptrs {:?}", (0..13).map(|k| w.s.ptr_at(g + G::defs + 8 * k)).collect::<Vec<_>>());
    }
    if let Some(at) = w.inline_array(g + G::defs, d * defs, 3)? {
        for i in 0..defs {
            let def = at + d * i;
            use L::FxGlassDef as D;
            // The loader's order: the physics preset first, then the two
            // materials (each in its own field, x64-zt's layout; x64-zt's
            // writer has the materials first). A walk in field order read
            // Prison Break only because all three load there (Strikezone's
            // third def loads only its material, which a physics preset's
            // read swallowed).
            for (ty, f) in [
                (AssetType::PhysPreset, D::physPreset),
                (AssetType::Material, D::material),
                (AssetType::Material, D::materialShattered),
                (AssetType::Fx, D::pieceBreakEffect),
                (AssetType::Fx, D::shatterEffect),
                (AssetType::Fx, D::shatterSmallEffect),
                (AssetType::Fx, D::crackDecalEffect),
            ] {
                let raw = w.s.ptr_at(def + f)?;
                let before = w.s.pos();
                let id = w.asset(ty, def + f)?;
                if trace {
                    eprintln!("FXWORLD def {i} field {f} {ty:?} {raw:?} -> {:?} {:?} pos {before:#x}->{:#x}", id, id.map(|id| w.cap.name_of(id).to_owned()), w.s.pos());
                }
            }
            for snd in [D::damagedSound, D::destroyedSound, D::destroyedQuietSound] {
                let n = super::misc::sound_custom(w, def + snd)?;
                if trace {
                    eprintln!("FXWORLD def {i} sound {snd} -> {n:?} pos {:#x}", w.s.pos());
                }
            }
        }
    }
    w.s.push(RUNTIME);
    w.blob(g + G::piecePlaces, L::FxGlassPiecePlace::SIZE * piece_limit, 3)?;
    w.blob(g + G::pieceStates, L::FxGlassPieceState::SIZE * piece_limit, 3)?;
    w.blob(g + G::pieceDynamics, L::FxGlassPieceDynamics::SIZE * piece_limit, 7)?;
    w.blob(g + G::geoData, L::FxGlassGeometryData::SIZE * geo_limit, 3)?;
    w.blob(g + G::isInUse, 4 * words, 3)?;
    w.blob(g + G::cellBits, 4 * words * cells, 3)?;
    w.blob(g + G::visData, (piece_limit + 15) & !15, 15)?;
    w.blob(g + G::linkOrg, 12 * piece_limit, 3)?;
    w.blob(g + G::halfThickness, 4 * ((piece_limit + 3) & !3), 15)?;
    w.s.pop();
    w.blob(g + G::lightingHandles, 2 * init_pieces, 1)?;
    w.blob(g + G::initPieceStates, L::FxGlassInitPieceState::SIZE * init_pieces, 3)?;
    w.blob(g + G::initGeoData, L::FxGlassGeometryData::SIZE * init_geo, 3)?;
    if trace {
        eprintln!("FXWORLD end pos {:#x}", w.s.pos());
    }
    w.s.pop();
    Ok(w.name_only(AssetType::FxWorld, name))
}

/// `GfxWorld`: the map's drawn world, in x64-zt's `gfx_world::write`
/// order. RUNTIME arrays take no file bytes but still move that block.
pub(super) fn gfx_world(w: &mut Walker, h: usize) -> Result<AssetId> {
    use crate::stream::RUNTIME;
    use L::GfxWorld as G;
    let u32_ = |w: &Walker, off: usize| -> Result<usize> { Ok(w.s.u32_at(h + off)? as usize) };
    w.s.push(VIRTUAL);
    let name = w.xstr(h + G::name)?.unwrap_or_default();
    w.xstr(h + G::baseName)?;
    let plane_count = u32_(w, G::planeCount)?;
    let node_count = u32_(w, G::nodeCount)?;
    let surface_count = u32_(w, G::surfaceCount)?;
    let sky_count = u32_(w, G::skyCount)?;
    let cell_count = u32_(w, G::dpvsPlanes__cellCount)?;
    let primary_lights = u32_(w, G::primaryLightCount)?;
    let last_sun = u32_(w, G::lastSunPrimaryLightIndex)?;
    let mut rec = GfxWorldRec { name: name.clone(), header: h, surface_count: surface_count as u32, ..Default::default() };

    // Skies.
    let sk = L::GfxSky::SIZE;
    if let Some(at) = w.inline_array(h + G::skies, sk * sky_count, 3)? {
        for i in 0..sky_count {
            let s = at + sk * i;
            let n = w.s.u32_at(s + L::GfxSky::skySurfCount)? as usize;
            let surfs = w.blob(s + L::GfxSky::skyStartSurfs, 4 * n, 3)?;
            rec.sky_surfaces.push(surfs.map(|pos| Span { pos, len: 4 * n }));
            let img = w.asset(AssetType::Image, s + L::GfxSky::skyImage)?;
            rec.sky_images.push(img);
        }
    }
    // Planes and nodes.
    w.blob(h + G::dpvsPlanes__planes, L::cplane_s::SIZE * plane_count, 3)?;
    w.blob(h + G::dpvsPlanes__nodes, 2 * node_count, 1)?;
    w.s.push(RUNTIME);
    w.blob(h + G::dpvsPlanes__sceneEntCellBits, 4 * (cell_count << 9), 3)?;
    w.s.pop();
    // Cell trees.
    let counts = w.blob(h + G::aabbTreeCounts, 4 * cell_count, 3)?;
    if let Some(trees) = w.inline_array(h + G::aabbTrees, 8 * cell_count, 127)? {
        for i in 0..cell_count {
            let n = match counts {
                Some(c) => w.s.u32_at(c + 4 * i)? as usize,
                None => 0,
            };
            let a = L::GfxAabbTree::SIZE;
            if let Some(at) = w.inline_array(trees + 8 * i, a * n, 3)? {
                for k in 0..n {
                    let t = at + a * k;
                    let m = w.s.u32_at(t + L::GfxAabbTree::smodelIndexCount)? as usize;
                    w.blob(t + L::GfxAabbTree::smodelIndexes, 2 * m, 1)?;
                }
            }
        }
    }
    // Cells and portals.
    let c = L::GfxCell::SIZE;
    if let Some(at) = w.inline_array(h + G::cells, c * cell_count, 3)? {
        for i in 0..cell_count {
            let cell = at + c * i;
            let portals = w.s.u32_at(cell + L::GfxCell::portalCount)? as usize;
            let p = L::GfxPortal::SIZE;
            if let Some(pa) = w.inline_array(cell + L::GfxCell::portals, p * portals, 3)? {
                for k in 0..portals {
                    let portal = pa + p * k;
                    let v = w.s.u32_at(portal + L::GfxPortal::vertexCount)? as usize;
                    w.blob(portal + L::GfxPortal::vertices, 12 * v, 3)?;
                }
            }
            let probes = w.s.u8_at(cell + L::GfxCell::reflectionProbeCount)? as usize;
            w.blob(cell + L::GfxCell::reflectionProbes, probes, 0)?;
            let refs = w.s.u8_at(cell + L::GfxCell::reflectionProbeReferenceCount)? as usize;
            w.blob(cell + L::GfxCell::reflectionProbeReferences, refs, 0)?;
        }
    }
    // Reflection probes.
    let probes = u32_(w, G::draw__reflectionProbeCount)?;
    if let Some(at) = w.inline_array(h + G::draw__reflectionProbes, 8 * probes, 7)? {
        for i in 0..probes {
            w.asset(AssetType::Image, at + 8 * i)?;
        }
    }
    let rp = L::GfxReflectionProbe::SIZE;
    if let Some(at) = w.inline_array(h + G::draw__reflectionProbeOrigins, rp * probes, 3)? {
        for i in 0..probes {
            let o = at + rp * i;
            let n = w.s.u32_at(o + L::GfxReflectionProbe::probeVolumeCount)? as usize;
            w.blob(o + L::GfxReflectionProbe::probeVolumes, L::GfxReflectionProbeVolumeData::SIZE * n, 3)?;
        }
    }
    w.s.push(RUNTIME);
    w.blob(h + G::draw__reflectionProbeTextures, L::GfxTexture::SIZE * probes, 3)?;
    w.s.pop();
    let probe_refs = u32_(w, G::draw__reflectionProbeReferenceCount)?;
    w.blob(h + G::draw__reflectionProbeReferenceOrigins, 12 * probe_refs, 3)?;
    w.blob(h + G::draw__reflectionProbeReferences, probe_refs, 0)?;
    // Lightmaps.
    let lightmaps = u32_(w, G::draw__lightmapCount)?;
    let lm = L::GfxLightmapArray::SIZE;
    if let Some(at) = w.inline_array(h + G::draw__lightmaps, lm * lightmaps, 3)? {
        for i in 0..lightmaps {
            let a = w.asset(AssetType::Image, at + lm * i + L::GfxLightmapArray::primary)?;
            let b = w.asset(AssetType::Image, at + lm * i + L::GfxLightmapArray::secondary)?;
            rec.lightmaps.push((a, b));
        }
    }
    w.s.push(RUNTIME);
    w.blob(h + G::draw__lightmapPrimaryTextures, L::GfxTexture::SIZE * lightmaps, 3)?;
    w.s.pop();
    w.s.push(RUNTIME);
    w.blob(h + G::draw__lightmapSecondaryTextures, L::GfxTexture::SIZE * lightmaps, 3)?;
    w.s.pop();
    w.asset(AssetType::Image, h + G::draw__lightmapOverridePrimary)?;
    w.asset(AssetType::Image, h + G::draw__lightmapOverrideSecondary)?;
    // Geometry.
    let vertex_count = u32_(w, G::draw__vertexCount)?;
    rec.vertex_count = vertex_count as u32;
    rec.vertices = w.blob(h + G::draw__vd__vertices, L::GfxWorldVertex::SIZE * vertex_count, 3)?;
    let layer = u32_(w, G::draw__vertexLayerDataSize)?;
    w.blob(h + G::draw__vld__data, layer, 0)?;
    let index_count = u32_(w, G::draw__indexCount)?;
    rec.index_count = index_count as u32;
    rec.indices = w.blob(h + G::draw__indices, 2 * index_count, 1)?;
    // Light grid.
    let row_axis = u32_(w, G::lightGrid__rowAxis)?.min(2);
    let mins = w.s.u16_at(h + G::lightGrid__mins + 2 * row_axis)? as usize;
    let maxs = w.s.u16_at(h + G::lightGrid__maxs + 2 * row_axis)? as usize;
    w.blob(h + G::lightGrid__rowDataStart, 2 * (maxs.saturating_sub(mins) + 1), 1)?;
    let raw = u32_(w, G::lightGrid__rawRowDataSize)?;
    w.blob(h + G::lightGrid__rawRowData, raw, 0)?;
    let entries = u32_(w, G::lightGrid__entryCount)?;
    w.blob(h + G::lightGrid__entries, L::GfxLightGridEntry::SIZE * entries, 3)?;
    let colors = u32_(w, G::lightGrid__colorCount)?;
    w.blob(h + G::lightGrid__colors, L::GfxLightGridColors::SIZE * colors, 3)?;
    let stages = w.s.u8_at(h + G::lightGrid__stageCount)? as usize;
    w.blob(h + G::lightGrid__stageLightingContrastGain, 4 * stages, 3)?;
    let palette = u32_(w, G::lightGrid__paletteEntryCount)?;
    w.blob(h + G::lightGrid__paletteEntryAddress, 4 * palette, 3)?;
    let bits = u32_(w, G::lightGrid__paletteBitstreamSize)?;
    w.blob(h + G::lightGrid__paletteBitstream, bits, 0)?;
    let nodes = u32_(w, G::lightGrid__tree__nodeCount)?;
    w.blob(h + G::lightGrid__tree__p_nodeTable, 4 * nodes, 3)?;
    let leaves = u32_(w, G::lightGrid__tree__leafTableSize)?;
    w.blob(h + G::lightGrid__tree__p_leafTable, leaves, 0)?;
    // Brush models, material memory, sun, outdoor image.
    let models = u32_(w, G::modelCount)?;
    rec.model_count = models as u32;
    rec.models = w.blob(h + G::models, L::GfxBrushModel::SIZE * models, 3)?;
    let mm = u32_(w, G::materialMemoryCount)?;
    if let Some(at) = w.inline_array(h + G::materialMemory, L::MaterialMemory::SIZE * mm, 3)? {
        for i in 0..mm {
            w.asset(AssetType::Material, at + L::MaterialMemory::SIZE * i + L::MaterialMemory::material)?;
        }
    }
    w.asset(AssetType::Material, h + G::sun__spriteMaterial)?;
    w.asset(AssetType::Material, h + G::sun__flareMaterial)?;
    w.asset(AssetType::Image, h + G::outdoorImage)?;
    // Runtime visibility.
    let dyn0 = u32_(w, G::dpvsDyn__dynEntClientCount)?;
    let dyn1 = u32_(w, G::dpvsDyn__dynEntClientCount + 4)?;
    let non_sun = primary_lights.saturating_sub(last_sun + 1);
    w.s.push(RUNTIME);
    w.blob(h + G::cellCasterBits, 4 * cell_count * cell_count.div_ceil(32), 3)?;
    w.blob(h + G::cellHasSunLitSurfsBits, 4 * cell_count.div_ceil(32), 3)?;
    w.blob(h + G::sceneDynModel, L::GfxSceneDynModel::SIZE * dyn0, 3)?;
    w.blob(h + G::sceneDynBrush, L::GfxSceneDynBrush::SIZE * dyn1, 3)?;
    w.blob(h + G::primaryLightEntityShadowVis, 4 * (non_sun << 13), 3)?;
    w.blob(h + G::primaryLightDynEntShadowVis, 4 * dyn0 * non_sun, 3)?;
    w.blob(h + G::primaryLightDynEntShadowVis + 8, 4 * dyn1 * non_sun, 3)?;
    w.blob(h + G::nonSunPrimaryLightForModelDynEnt, 2 * dyn0, 1)?;
    w.s.pop();
    // Shadows and light regions.
    for field in [G::shadowGeom, G::shadowGeomOptimized] {
        let sg = L::GfxShadowGeometry::SIZE;
        if let Some(at) = w.inline_array(h + field, sg * primary_lights, 3)? {
            for i in 0..primary_lights {
                let g = at + sg * i;
                let s = w.s.u16_at(g + L::GfxShadowGeometry::surfaceCount)? as usize;
                let m = w.s.u16_at(g + L::GfxShadowGeometry::smodelCount)? as usize;
                w.blob(g + L::GfxShadowGeometry::sortedSurfIndex, 4 * s, 3)?;
                w.blob(g + L::GfxShadowGeometry::smodelIndex, 2 * m, 1)?;
            }
        }
    }
    let lr = L::GfxLightRegion::SIZE;
    if let Some(at) = w.inline_array(h + G::lightRegion, lr * primary_lights, 3)? {
        for i in 0..primary_lights {
            let r = at + lr * i;
            let n = w.s.u32_at(r + L::GfxLightRegion::hullCount)? as usize;
            let hl = L::GfxLightRegionHull::SIZE;
            if let Some(ha) = w.inline_array(r + L::GfxLightRegion::hulls, hl * n, 3)? {
                for k in 0..n {
                    let hull = ha + hl * k;
                    let axes = w.s.u32_at(hull + L::GfxLightRegionHull::axisCount)? as usize;
                    w.blob(hull + L::GfxLightRegionHull::axis, L::GfxLightRegionAxis::SIZE * axes, 3)?;
                }
            }
        }
    }
    // Static visibility.
    let smodels = u32_(w, G::dpvs__smodelCount)?;
    let static_surfs = u32_(w, G::dpvs__staticSurfaceCount)?;
    let smodel_vis = u32_(w, G::dpvs__smodelVisDataCount)?;
    let surface_vis = u32_(w, G::dpvs__surfaceVisDataCount)?;
    w.s.push(RUNTIME);
    for k in 0..3 {
        w.blob(h + G::dpvs__smodelVisData + 8 * k, 4 * smodel_vis, 127)?;
    }
    for k in 0..3 {
        w.blob(h + G::dpvs__surfaceVisData + 8 * k, 4 * surface_vis, 127)?;
    }
    for k in 0..3 {
        w.blob(h + G::dpvs__unknownData01 + 8 * k, 4 * (smodel_vis + 1), 127)?;
    }
    for k in 0..3 {
        w.blob(h + G::dpvs__unknownData02 + 8 * k, 4 * surface_vis, 127)?;
    }
    w.s.pop();
    w.blob(h + G::dpvs__lodData, 4 * (smodels + 1), 3)?;
    w.s.push(RUNTIME);
    for k in 0..3 {
        w.blob(h + G::dpvs__tessellationCutoffVisData + 8 * k, 4 * surface_vis, 127)?;
    }
    w.s.pop();
    rec.sorted_surfs = w.blob(h + G::dpvs__sortedSurfIndex, 4 * static_surfs, 3)?;
    rec.static_surface_count = static_surfs as u32;
    w.blob(h + G::dpvs__smodelInsts, L::GfxStaticModelInst::SIZE * smodels, 3)?;
    let gs = L::GfxSurface::SIZE;
    rec.surfaces = w.inline_array(h + G::dpvs__surfaces, gs * surface_count, 3)?;
    if let Some(at) = rec.surfaces {
        for i in 0..surface_count {
            let m = w.asset(AssetType::Material, at + gs * i + L::GfxSurface::material)?;
            rec.surface_materials.push(m);
        }
    }
    rec.surface_bounds = w.blob(h + G::dpvs__surfacesBounds, L::GfxSurfaceBounds::SIZE * surface_count, 3)?;
    let sd = L::GfxStaticModelDrawInst::SIZE;
    rec.smodel_count = smodels as u32;
    rec.smodels = w.inline_array(h + G::dpvs__smodelDrawInsts, sd * smodels, 3)?;
    if let Some(at) = rec.smodels {
        for i in 0..smodels {
            let d = at + sd * i;
            let m = w.asset(AssetType::XModel, d + L::GfxStaticModelDrawInst::model)?;
            rec.smodel_models.push(m);
            let n = w.s.u32_at(d + L::GfxStaticModelDrawInst::vertexLightingInfo__numLightingValues)? as usize;
            w.blob(d + L::GfxStaticModelDrawInst::vertexLightingInfo__lightingValues, L::GfxStaticModelVertexLighting::SIZE * n, 3)?;
        }
    }
    let sun_opt = u32_(w, G::dpvs__sunShadowOptCount)?;
    let sun_vis = u32_(w, G::dpvs__sunSurfVisDataCount)?;
    w.s.push(RUNTIME);
    w.blob(h + G::dpvs__surfaceMaterials, L::GfxDrawSurf::SIZE * surface_count, 7)?;
    w.blob(h + G::dpvs__surfaceCastsSunShadow, 4 * surface_vis, 127)?;
    w.blob(h + G::dpvs__surfaceCastsSunShadowOpt, 4 * sun_opt * sun_vis, 127)?;
    w.blob(h + G::dpvs__constantBuffersLit, 8 * smodels, 7)?;
    w.blob(h + G::dpvs__constantBuffersAmbient, 8 * smodels, 7)?;
    w.s.pop();
    let words0 = u32_(w, G::dpvsDyn__dynEntClientWordCount)?;
    let words1 = u32_(w, G::dpvsDyn__dynEntClientWordCount + 4)?;
    w.s.push(RUNTIME);
    w.blob(h + G::dpvsDyn__dynEntCellBits, 4 * words0 * cell_count, 3)?;
    w.blob(h + G::dpvsDyn__dynEntCellBits + 8, 4 * words1 * cell_count, 3)?;
    for j in 0..3 {
        w.blob(h + G::dpvsDyn__dynEntVisData + 8 * j, 32 * words0, 15)?;
        w.blob(h + G::dpvsDyn__dynEntVisData + 8 * (3 + j), 32 * words1, 15)?;
    }
    w.s.pop();
    let heroes = u32_(w, G::heroOnlyLightCount)?;
    w.blob(h + G::heroOnlyLights, L::GfxHeroOnlyLight::SIZE * heroes, 3)?;
    let tome = u32_(w, G::umbraTomeSize)?;
    w.blob(h + G::umbraTomeData, tome, 15)?;
    w.s.pop();
    w.cap.gfx_worlds.push(rec);
    Ok(AssetId { ty: AssetType::GfxWorld, index: (w.cap.gfx_worlds.len() - 1) as u32 })
}

/// A `MapTriggers`' arrays: (models, hulls, slabs) as file spans.
type TriggerSpans = [Option<Span>; 3];

fn map_triggers(w: &mut Walker, t: usize) -> Result<TriggerSpans> {
    use L::MapTriggers as T;
    let count = w.s.u32_at(t + T::count)? as usize;
    let hulls = w.s.u32_at(t + T::hullCount)? as usize;
    let slabs = w.s.u32_at(t + T::slabCount)? as usize;
    let span = |pos: Option<usize>, len: usize| pos.map(|pos| Span { pos, len });
    let m = w.blob(t + T::models, L::TriggerModel::SIZE * count, 3)?;
    let h = w.blob(t + T::hulls, L::TriggerHull::SIZE * hulls, 3)?;
    let s = w.blob(t + T::slabs, L::TriggerSlab::SIZE * slabs, 3)?;
    Ok([
        span(m, L::TriggerModel::SIZE * count),
        span(h, L::TriggerHull::SIZE * hulls),
        span(s, L::TriggerSlab::SIZE * slabs),
    ])
}

/// `MapEnts`: the entity string, triggers, client triggers, spawns and
/// splines (x64-zt's `map_ents::write`).
pub(super) fn map_ents(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::MapEnts as M;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + M::name)?.unwrap_or_default();
    let chars = w.s.u32_at(h + M::numEntityChars)? as usize;
    let ents = w.blob(h + M::entityString, chars, 0)?;
    let entity_string = match ents {
        Some(at) => {
            let b = w.s.slice(at, chars)?;
            String::from_utf8_lossy(b.split(|&c| c == 0).next().unwrap_or(&[])).into_owned()
        }
        None => String::new(),
    };
    let triggers = map_triggers(w, h + M::trigger)?;
    map_triggers(w, h + M::clientTrigger__trigger)?;
    let ct = h + M::clientTrigger;
    use L::ClientTriggers as C;
    let count = w.s.u32_at(ct + C::trigger__count)? as usize;
    let nodes = w.s.u32_at(ct + C::numClientTriggerNodes)? as usize;
    w.blob(ct + C::clientTriggerAabbTree, L::ClientTriggerAabbNode::SIZE * nodes, 3)?;
    let len = w.s.u32_at(ct + C::triggerStringLength)? as usize;
    w.blob(ct + C::triggerString, len, 0)?;
    w.blob(ct + C::visionSetTriggers, 2 * count, 1)?;
    w.blob(ct + C::triggerType, count, 0)?;
    w.blob(ct + C::origins, 12 * count, 3)?;
    w.blob(ct + C::scriptDelay, 4 * count, 3)?;
    w.blob(ct + C::audioTriggers, 2 * count, 1)?;
    w.blob(ct + C::blendLookup, 2 * count, 1)?;
    w.blob(ct + C::npcTriggers, 2 * count, 1)?;
    let blends = w.s.u32_at(h + M::clientTriggerBlend__numClientTriggerBlendNodes)? as usize;
    w.blob(h + M::clientTriggerBlend__blendNodes, L::ClientTriggerBlendNode::SIZE * blends, 3)?;
    let spawns = w.s.u32_at(h + M::spawnList__spawnsCount)? as usize;
    let spawn_at = w.blob(h + M::spawnList__spawns, L::SpawnPointEntityRecord::SIZE * spawns, 3)?;
    let splines = w.s.u16_at(h + M::splineList__splineCount)? as usize;
    let sl = L::SplinePointRecordList::SIZE;
    if let Some(at) = w.inline_array(h + M::splineList__splines, sl * splines, 3)? {
        for i in 0..splines {
            let s = at + sl * i;
            let n = w.s.u16_at(s + L::SplinePointRecordList::splinePointCount)? as usize;
            let pr = L::SplinePointEntityRecord::SIZE;
            if let Some(pa) = w.inline_array(s + L::SplinePointRecordList::splinePoints, pr * n, 3)? {
                for k in 0..n {
                    let p = pa + pr * k;
                    w.blob(p + L::SplinePointEntityRecord::splineNodeLabel, 64, 0)?;
                    w.blob(p + L::SplinePointEntityRecord::positionCubic, 48, 3)?;
                    w.blob(p + L::SplinePointEntityRecord::tangentQuadratic, 36, 3)?;
                }
            }
        }
    }
    w.s.pop();
    w.cap.map_ents.push(MapEntsRec {
        name,
        entity_string,
        spawns: spawn_at.map(|pos| Span { pos, len: L::SpawnPointEntityRecord::SIZE * spawns }),
        triggers,
    });
    Ok(AssetId { ty: AssetType::MapEnts, index: (w.cap.map_ents.len() - 1) as u32 })
}

/// `GlassWorld`: the glass pieces' states and their names.
pub(super) fn glass_world(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::G_GlassData as D;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + L::GlassWorld::name)?.unwrap_or_default();
    if let Some(d) = w.inline_array(h + L::GlassWorld::g_glassData, D::SIZE, 3)? {
        let pieces = w.s.u32_at(d + D::pieceCount)? as usize;
        w.blob(d + D::glassPieces, L::G_GlassPiece::SIZE * pieces, 3)?;
        let names = w.s.u32_at(d + D::glassNameCount)? as usize;
        let gn = L::G_GlassName::SIZE;
        if let Some(at) = w.inline_array(d + D::glassNames, gn * names, 3)? {
            for i in 0..names {
                let n = at + gn * i;
                w.xstr(n + L::G_GlassName::nameStr)?;
                let c = w.s.u16_at(n + L::G_GlassName::pieceCount)? as usize;
                w.blob(n + L::G_GlassName::pieceIndices, 2 * c, 1)?;
            }
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::GlassWorld, name))
}

/// A `ClipInfo`'s arrays (x64-zt's `clip_map::write_info`).
/// The file offset a pointer field names when it refers to VIRTUAL data
/// read earlier (no read: `inline_array` reads what follows).
fn ref_pos(w: &Walker, field: usize) -> Result<Option<usize>> {
    Ok(match w.s.ptr_at(field)? {
        crate::ptr::Ptr::Ref { block: VIRTUAL, offset } => w.vmap.file_pos(offset as u64),
        _ => None,
    })
}

fn clip_info(w: &mut Walker, c: usize) -> Result<ClipInfoRec> {
    use L::ClipInfo as I;
    let mut rec = ClipInfoRec::default();
    let n = |w: &Walker, off: usize| -> Result<usize> { Ok(w.s.u32_at(c + off)? as usize) };
    let planes = n(w, I::planeCount)?;
    rec.plane_count = planes as u32;
    rec.planes = w.blob(c + I::planes, L::cplane_s::SIZE * planes, 3)?;
    let mats = n(w, I::numMaterials)?;
    let cm = L::ClipMaterial::SIZE;
    if let Some(at) = w.inline_array(c + I::materials, cm * mats, 3)? {
        for i in 0..mats {
            let m = at + cm * i;
            let name = w.xstr(m + L::ClipMaterial::name)?.unwrap_or_default();
            let flags = w.s.i32_at(m + L::ClipMaterial::surfaceFlags)?;
            let contents = w.s.i32_at(m + L::ClipMaterial::contents)?;
            rec.materials.push((name, flags, contents));
        }
    }
    let sides = n(w, I::numBrushSides)?;
    let bs = L::cbrushside_t::SIZE;
    if let Some(at) = w.inline_array(c + I::brushsides, bs * sides, 3)? {
        for i in 0..sides {
            if let Some(p) = w.blob(at + bs * i + L::cbrushside_t::plane, L::cplane_s::SIZE, 3)? {
                rec.side_planes.insert(at + bs * i, p);
            }
        }
    }
    let edges = n(w, I::numBrushEdges)?;
    w.blob(c + I::brushEdges, edges, 0)?;
    let lbn = n(w, I::leafbrushNodesCount)?;
    rec.leaf_brush_node_count = lbn as u32;
    let ln = L::cLeafBrushNode_s::SIZE;
    rec.leaf_brush_nodes = w.inline_array(c + I::leafbrushNodes, ln * lbn, 3)?;
    if let Some(at) = rec.leaf_brush_nodes {
        for i in 0..lbn {
            let node = at + ln * i;
            let count = w.s.u16_at(node + L::cLeafBrushNode_s::leafBrushCount)? as i16;
            rec.leaf_brush_lists.push(if count > 0 {
                w.blob(node + L::cLeafBrushNode_s::data, 2 * count as usize, 1)?
            } else {
                None
            });
        }
    }
    let leaf_brushes = n(w, I::numLeafBrushes)?;
    w.blob(c + I::leafbrushes, 2 * leaf_brushes, 1)?;
    let brushes = w.s.u16_at(c + I::numBrushes)? as usize;
    rec.brush_count = brushes as u32;
    let b = L::cbrush_t::SIZE;
    rec.brushes = w.inline_array(c + I::brushes, b * brushes, 127)?;
    if let Some(at) = rec.brushes {
        for i in 0..brushes {
            let br = at + b * i;
            let referred = ref_pos(w, br + L::cbrush_t::sides)?;
            let side = match w.inline_array(br + L::cbrush_t::sides, bs, 3)? {
                Some(side) => {
                    if let Some(p) = w.blob(side + L::cbrushside_t::plane, L::cplane_s::SIZE, 3)? {
                        rec.side_planes.insert(side, p);
                    }
                    Some(side)
                }
                None => referred,
            };
            rec.brush_sides.push(side);
            w.blob(br + L::cbrush_t::baseAdjacentSide, 1, 0)?;
        }
    }
    rec.brush_bounds = w.blob(c + I::brushBounds, 24 * brushes, 127)?;
    rec.brush_contents = w.blob(c + I::brushContents, 4 * brushes, 3)?;
    Ok(rec)
}

/// `clipMap_t` offsets (MSVC x64; the struct is `alignas(128)`, 640 bytes).
mod cm {
    pub const NAME: usize = 0;
    pub const INFO: usize = 16;
    pub const P_INFO: usize = 144;
    pub const NUM_STATIC_MODELS: usize = 152;
    pub const STATIC_MODEL_LIST: usize = 160;
    pub const NUM_NODES: usize = 168;
    pub const NODES: usize = 176;
    pub const NUM_LEAFS: usize = 184;
    pub const LEAFS: usize = 192;
    pub const VERT_COUNT: usize = 200;
    pub const VERTS: usize = 208;
    pub const TRI_COUNT: usize = 216;
    pub const TRI_INDICES: usize = 224;
    pub const TRI_EDGE_IS_WALKABLE: usize = 232;
    pub const BORDER_COUNT: usize = 240;
    pub const BORDERS: usize = 248;
    pub const PARTITION_COUNT: usize = 256;
    pub const PARTITIONS: usize = 264;
    pub const AABB_TREE_COUNT: usize = 272;
    pub const AABB_TREES: usize = 280;
    pub const NUM_SUB_MODELS: usize = 288;
    pub const CMODELS: usize = 296;
    pub const MAP_ENTS: usize = 304;
    pub const STAGES: usize = 312;
    pub const STAGE_COUNT: usize = 320;
    pub const STAGE_TRIGGER: usize = 328;
    pub const SMODEL_NODE_COUNT: usize = 376;
    pub const SMODEL_NODES: usize = 384;
    pub const DYN_ENT_COUNT: usize = 392;
    pub const DYN_ENT_DEF_LIST: usize = 400;
    pub const DYN_ENT_POSE_LIST: usize = 416;
    pub const DYN_ENT_CLIENT_LIST: usize = 432;
    pub const DYN_ENT_COLL_LIST: usize = 448;
    pub const DYN_ENT_ANCHOR_COUNT: usize = 464;
    pub const DYN_ENT_ANCHOR_NAMES: usize = 472;
    pub const SCRIPTABLE_MAP_ENTS: usize = 480;
}

/// `clipMap_t`: collision (brushes, terrain triangles, static models),
/// dynamic entities and scriptables (x64-zt's `clip_map::write`).
pub(super) fn clip_map(w: &mut Walker, h: usize) -> Result<AssetId> {
    use crate::stream::{RUNTIME, TEMP};
    let n32 = |w: &Walker, off: usize| -> Result<usize> { Ok(w.s.u32_at(h + off)? as usize) };
    w.s.push(VIRTUAL);
    let name = w.xstr(h + cm::NAME)?.unwrap_or_default();
    let info = clip_info(w, h + cm::INFO)?;
    w.s.push(TEMP);
    if let Some(at) = w.inline_array(h + cm::P_INFO, L::ClipInfo::SIZE, 3)? {
        clip_info(w, at)?;
    }
    w.s.pop();
    let mut rec = ClipMapRec { name: name.clone(), header: h, info, ..Default::default() };
    let smodels = n32(w, cm::NUM_STATIC_MODELS)?;
    let sm = L::cStaticModel_s::SIZE;
    rec.static_models = w.inline_array(h + cm::STATIC_MODEL_LIST, sm * smodels, 3)?;
    if let Some(at) = rec.static_models {
        for i in 0..smodels {
            let m = w.asset(AssetType::XModel, at + sm * i + L::cStaticModel_s::xmodel)?;
            rec.static_model_models.push(m);
        }
    }
    let nodes = n32(w, cm::NUM_NODES)?;
    let nd = L::cNode_t::SIZE;
    if let Some(at) = w.inline_array(h + cm::NODES, nd * nodes, 3)? {
        for i in 0..nodes {
            w.blob(at + nd * i + L::cNode_t::plane, L::cplane_s::SIZE, 3)?;
        }
    }
    let leafs = n32(w, cm::NUM_LEAFS)?;
    w.blob(h + cm::LEAFS, L::cLeaf_t::SIZE * leafs, 3)?;
    let verts = n32(w, cm::VERT_COUNT)?;
    rec.vert_count = verts as u32;
    rec.verts = w.blob(h + cm::VERTS, 12 * verts, 3)?;
    let tris = n32(w, cm::TRI_COUNT)?;
    rec.tri_count = tris as u32;
    rec.tri_indices = w.blob(h + cm::TRI_INDICES, 6 * tris, 1)?;
    w.blob(h + cm::TRI_EDGE_IS_WALKABLE, ((3 * tris + 31) >> 3) & !3, 0)?;
    let borders = n32(w, cm::BORDER_COUNT)?;
    w.blob(h + cm::BORDERS, L::CollisionBorder::SIZE * borders, 3)?;
    let parts = n32(w, cm::PARTITION_COUNT)?;
    let cp = L::CollisionPartition::SIZE;
    rec.partition_count = parts as u32;
    rec.partitions = w.inline_array(h + cm::PARTITIONS, cp * parts, 3)?;
    if let Some(at) = rec.partitions {
        for i in 0..parts {
            w.blob(at + cp * i + L::CollisionPartition::borders, L::CollisionBorder::SIZE, 3)?;
        }
    }
    let trees = n32(w, cm::AABB_TREE_COUNT)?;
    rec.aabb_tree_count = trees as u32;
    rec.aabb_trees = w.blob(h + cm::AABB_TREES, L::CollisionAabbTree::SIZE * trees, 15)?;
    let subs = n32(w, cm::NUM_SUB_MODELS)?;
    let cmo = L::cmodel_t::SIZE;
    rec.cmodel_count = subs as u32;
    rec.cmodels = w.inline_array(h + cm::CMODELS, cmo * subs, 3)?;
    if let Some(at) = rec.cmodels {
        for i in 0..subs {
            let own = w.inline_array(at + cmo * i + L::cmodel_t::info, L::ClipInfo::SIZE, 3)?;
            if let Some(info) = own {
                clip_info(w, info)?;
            }
            rec.cmodel_own_info.push(own.is_some());
        }
    }
    let smn = w.s.u16_at(h + cm::SMODEL_NODE_COUNT)? as usize;
    w.blob(h + cm::SMODEL_NODES, L::SModelAabbNode::SIZE * smn, 3)?;
    w.asset(AssetType::MapEnts, h + cm::MAP_ENTS)?;
    let stages = w.s.u8_at(h + cm::STAGE_COUNT)? as usize;
    let st = L::Stage::SIZE;
    if let Some(at) = w.inline_array(h + cm::STAGES, st * stages, 3)? {
        for i in 0..stages {
            w.xstr(at + st * i + L::Stage::name)?;
        }
    }
    map_triggers(w, h + cm::STAGE_TRIGGER)?;
    let dyn_counts = [w.s.u16_at(h + cm::DYN_ENT_COUNT)? as usize, w.s.u16_at(h + cm::DYN_ENT_COUNT + 2)? as usize];
    let de = L::DynEntityDef::SIZE;
    for (k, &count) in dyn_counts.iter().enumerate() {
        rec.dyn_ent_counts[k] = count as u32;
        rec.dyn_ent_defs[k] = w.inline_array(h + cm::DYN_ENT_DEF_LIST + 8 * k, de * count, 3)?;
        if let Some(at) = rec.dyn_ent_defs[k] {
            for i in 0..count {
                let d = at + de * i;
                let m = w.asset(AssetType::XModel, d + L::DynEntityDef::baseModel)?;
                rec.dyn_ent_models[k].push(m);
                w.asset(AssetType::Fx, d + L::DynEntityDef::destroyFx)?;
                w.asset(AssetType::PhysPreset, d + L::DynEntityDef::physPreset)?;
                w.blob(d + L::DynEntityDef::hinge, L::DynEntityHingeDef::SIZE, 3)?;
                w.blob(d + L::DynEntityDef::linkTo, L::DynEntityLinkToDef::SIZE, 3)?;
            }
        }
    }
    w.s.push(RUNTIME);
    for (base, size) in [
        (cm::DYN_ENT_POSE_LIST, L::DynEntityPose::SIZE),
        (cm::DYN_ENT_CLIENT_LIST, L::DynEntityClient::SIZE),
        (cm::DYN_ENT_COLL_LIST, L::DynEntityColl::SIZE),
    ] {
        for (k, &count) in dyn_counts.iter().enumerate() {
            w.blob(h + base + 8 * k, size * count, 3)?;
        }
    }
    w.s.pop();
    let anchors = n32(w, cm::DYN_ENT_ANCHOR_COUNT)?;
    w.blob(h + cm::DYN_ENT_ANCHOR_NAMES, 4 * anchors, 3)?;
    let sme = h + cm::SCRIPTABLE_MAP_ENTS;
    use L::ScriptableMapEnts as S;
    let instances = w.s.u32_at(sme + S::instanceCount)? as usize;
    let si = L::ScriptableInstance::SIZE;
    if let Some(at) = w.inline_array(sme + S::instances, si * instances, 3)? {
        for i in 0..instances {
            let inst = at + si * i;
            let def_ptr = w.s.ptr_at(inst + L::ScriptableInstance::def)?;
            let Some(def) = w.asset(AssetType::Scriptable, inst + L::ScriptableInstance::def)? else { continue };
            let Some(dh) = w
                .header_of(def)
                .or_else(|| w.cap.scriptable_headers.get(&def.index).copied())
                .or_else(|| def_ptr.virtual_offset().and_then(|v| w.vmap.file_pos(v as u64)))
            else {
                return Err(crate::cursor::ReadError::Invalid("scriptable instance def not found", inst));
            };
            let constants = w.s.u8_at(dh + L::ScriptableDef::eventConstantsSize)? as usize;
            let parts = w.s.u8_at(dh + L::ScriptableDef::partCount)? as usize;
            let stream = w.s.u8_at(dh + L::ScriptableDef::eventStreamSize)? as usize;
            w.blob(inst + L::ScriptableInstance::eventConstantsBuf, constants, 0)?;
            w.s.push(RUNTIME);
            w.blob(inst + L::ScriptableInstance::partStates, 4 * parts, 3)?;
            w.blob(inst + L::ScriptableInstance::eventStreamBuf, stream, 0)?;
            w.s.pop();
        }
    }
    let anims = w.s.u32_at(sme + S::animEntryCount)? as usize;
    let ae = L::ScriptableAnimationEntry::SIZE;
    if let Some(at) = w.inline_array(sme + S::animEntries, ae * anims, 3)? {
        for i in 0..anims {
            w.xstr(at + ae * i + L::ScriptableAnimationEntry::animName)?;
        }
    }
    w.s.pop();
    w.cap.clip_maps.push(rec);
    Ok(AssetId { ty: AssetType::ClipMap, index: (w.cap.clip_maps.len() - 1) as u32 })
}

/// `ScriptableEventType`: which member of an event's data union it uses
/// (the kinds with a pointer; the others hold numbers only).
mod scriptable_event {
    pub const MODEL: u32 = 0;
    pub const FX: u32 = 1;
    pub const STOP_FX: u32 = 2;
    pub const SOUND: u32 = 3;
    pub const ANIMATION: u32 = 4;
    pub const PHYSICS_LAUNCH: u32 = 6;
    /// (Retail: a light flicker's event 7 names its `noteworthy`.)
    pub const LIGHT_SETTINGS: u32 = 7;
    pub const SHAKE: u32 = 9;
}

/// A stop-effect event's handle: stored as a sound alias is
/// (`misc::sound_custom`): one that follows is an 8-byte slot holding the
/// effect's name (read right there, or a reference to an earlier string),
/// not the effect. (Retail mp_strikezone's and mp_dart's; a play-effect
/// event's effect loads as an asset: mp_fahrenheit's security camera loads
/// a name-only `FxEffectDef` there.)
fn fx_name_slot(w: &mut Walker, field: usize) -> Result<Option<String>> {
    super::misc::sound_custom(w, field)
}

/// `ScriptableDef` (a map's scriptable objects: destructible cars, lights,
/// fans), loaded inline by `clipMap_t::scriptableMapEnts` instances: its
/// name, base model, collision brush names, parts (states, each state's
/// on-enter events with the asset or name each event's kind points at) and
/// notetracks, in x64-zt's `scriptable::write` order.
pub(super) fn scriptable(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ScriptableDef as S;
    let trace = std::env::var_os("IW6_TRACE_SCRIPTABLE").is_some();
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    if trace {
        eprintln!("SCRIPTABLE {name} header at {h:#x}, name read to {:#x}; parts {} notetracks {} ptrs {:?}", w.s.pos(), w.s.u8_at(h + S::partCount)?, w.s.u8_at(h + S::notetrackCount)?, (0..6).map(|k| w.s.ptr_at(h + 8 * k)).collect::<Vec<_>>());
    }
    w.asset(AssetType::XModel, h + S::baseModel)?;
    w.xstr(h + S::baseCollisionBrush)?;
    w.xstr(h + S::destroyedCollisionBrush)?;
    let parts = usize::from(w.s.u8_at(h + S::partCount)?);
    let pd = L::ScriptablePartDef::SIZE;
    if let Some(at) = w.inline_array(h + S::parts, pd * parts, 3)? {
        for p in 0..parts {
            let part = at + pd * p;
            let states = usize::from(w.s.u8_at(part + L::ScriptablePartDef::stateCount)?);
            let sd = L::ScriptableStateDef::SIZE;
            let Some(sat) = w.inline_array(part + L::ScriptablePartDef::states, sd * states, 3)? else { continue };
            if trace {
                eprintln!("SCRIPTABLE part {p} at {part:#x}: {states} states at {sat:#x}");
            }
            for s in 0..states {
                let state = sat + sd * s;
                let events = usize::from(w.s.u8_at(state + L::ScriptableStateDef::onEnterEventCount)?);
                let ed = L::ScriptableEventDef::SIZE;
                let Some(eat) = w.inline_array(state + L::ScriptableStateDef::onEnterEvents, ed * events, 3)? else {
                    continue;
                };
                for e in 0..events {
                    let ev = eat + ed * e;
                    let ty = w.s.u32_at(ev + L::ScriptableEventDef::r#type)?;
                    let data = ev + L::ScriptableEventDef::data;
                    if trace {
                        eprintln!(
                            "SCRIPTABLE {name} part {p} state {s} event {e} type {ty} data {:?} {:?} pos {:#x}",
                            w.s.ptr_at(data),
                            w.s.ptr_at(data + 16),
                            w.s.pos()
                        );
                    }
                    use scriptable_event as T;
                    match ty {
                        T::MODEL | T::PHYSICS_LAUNCH => {
                            w.asset(AssetType::XModel, data)?;
                        }
                        T::FX => {
                            w.asset(AssetType::Fx, data)?;
                        }
                        T::STOP_FX => {
                            fx_name_slot(w, data)?;
                        }
                        // The alias's name itself (retail mp_dart: a name
                        // that follows is right there; mp_strikezone's refer
                        // to earlier strings).
                        T::SOUND => {
                            w.xstr(data)?;
                        }
                        T::ANIMATION | T::SHAKE => {
                            w.xstr(data)?;
                        }
                        T::LIGHT_SETTINGS => {
                            w.xstr(data + L::ScriptableEventLightSettingsDef::noteworthy)?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let notes = usize::from(w.s.u8_at(h + S::notetrackCount)?);
    let nd = L::ScriptableNotetrackDef::SIZE;
    if let Some(at) = w.inline_array(h + S::notetracks, nd * notes, 3)? {
        for n in 0..notes {
            let note = at + nd * n;
            let data = note + L::ScriptableNotetrackDef::data;
            match w.s.u32_at(note + L::ScriptableNotetrackDef::r#type)? {
                0 => {
                    w.asset(AssetType::Fx, data)?;
                }
                1 => {
                    w.xstr(data)?;
                }
                _ => {}
            }
        }
    }
    w.s.pop();
    if trace {
        eprintln!("SCRIPTABLE {name} end pos {:#x}", w.s.pos());
    }
    let id = w.name_only(AssetType::Scriptable, name);
    // (Instances read their part and event counts from it, also when it
    // was loaded inline by an earlier instance.)
    w.cap.scriptable_headers.insert(id.index, h);
    Ok(id)
}
