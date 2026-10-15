//! The map assets: com world (primary lights), nav mesh, impact fx, fx
//! world (glass), light maps, the gfx world and its transient zones, map
//! ents, glass world, path data and the clip map (x64-zt
//! `iw7/assets/{comworld,navmesh,impactfx,fxworld,gfxlightmap,gfxworld,
//! gfxworld_tr,mapents,glassworld,aipaths,clipmap}.cpp`), in the writers'
//! order.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::{RUNTIME, TEMP, VIRTUAL};
use crate::walk::weapon::fx_combined;
use crate::walk::{
    ClipMapRec, ComLightRec, ComWorldRec, GfxLightMapRec, GfxWorldRec, GfxWorldTrRec, MapEntsRec, PathDataRec, Span,
};

/// `count` elements of `elem` bytes a pointer field names (inline or a
/// reference); the span when the data is in the zone.
pub(super) fn arr(w: &mut Walker, field: usize, count: usize, elem: usize, mask: u64) -> Result<Option<Span>> {
    let len = count * elem;
    Ok(w.blob(field, len, mask)?.map(|pos| Span { pos, len }))
}

/// Like `arr`, in the RUNTIME block (allocated, not in the file).
fn rt(w: &mut Walker, field: usize, count: usize, elem: usize, mask: u64) -> Result<()> {
    w.s.push(RUNTIME);
    w.blob(field, count * elem, mask)?;
    w.s.pop();
    Ok(())
}

pub(super) fn u32f(w: &Walker, at: usize) -> Result<usize> {
    Ok(w.s.u32_at(at)? as usize)
}

fn u16f(w: &Walker, at: usize) -> Result<usize> {
    Ok(w.s.u16_at(at)? as usize)
}

fn v3(w: &Walker, at: usize) -> Result<[f32; 3]> {
    Ok([w.s.f32_at(at)?, w.s.f32_at(at + 4)?, w.s.f32_at(at + 8)?])
}

pub(super) fn comworld(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ComPrimaryLight as P;
    use L::ComWorld as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::name)?.unwrap_or_default();
    let n = u32f(w, h + C::primaryLightCount)?;
    let mut lights = Vec::new();
    let mut primary_lights = None;
    if let Some(at) = w.inline_array(h + C::primaryLights, P::SIZE * n, 7)? {
        primary_lights = Some(Span { pos: at, len: P::SIZE * n });
        for i in 0..n {
            let l = at + P::SIZE * i;
            let def_name = w.xstr(l + P::defName)?;
            lights.push(ComLightRec {
                kind: w.s.u8_at(l + P::r#type)?,
                color: v3(w, l + P::color)?,
                dir: v3(w, l + P::dir)?,
                origin: v3(w, l + P::origin)?,
                radius: w.s.f32_at(l + P::radius)?,
                cos_half_fov_outer: w.s.f32_at(l + P::cosHalfFovOuter)?,
                cos_half_fov_inner: w.s.f32_at(l + P::cosHalfFovInner)?,
                def_name,
            });
        }
    }
    let envs = u32f(w, h + C::primaryLightEnvCount)?;
    arr(w, h + C::primaryLightEnvs, envs, L::ComPrimaryLightEnv::SIZE, 3)?;
    w.xstr(h + C::changeListInfo__userName)?;
    let gates = u32f(w, h + C::numUmbraGates)?;
    if let Some(at) = w.inline_array(h + C::umbraGateNames, 8 * gates, 7)? {
        for i in 0..gates {
            w.xstr(at + 8 * i)?;
        }
    }
    w.s.pop();
    w.cap.com_worlds.push(ComWorldRec {
        name,
        record: Span { pos: h, len: C::SIZE },
        primary_lights,
        primary_light_count: n as u32,
        lights,
    });
    Ok(AssetId { ty: AssetType::ComWorld, index: (w.cap.com_worlds.len() - 1) as u32 })
}

pub(super) fn navmesh(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::NavMeshData as N;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + N::name)?.unwrap_or_default();
    let plane = L::nav_boundary_plane_s::SIZE;
    {
        use L::nav_resource_s as R;
        let n = u32f(w, h + N::numNavResources)?;
        if let Some(at) = w.inline_array(h + N::navResources, R::SIZE * n, 7)? {
            for i in 0..n {
                let r = at + R::SIZE * i;
                let size = u32f(w, r + R::graphSize)?;
                w.blob(r + R::pGraphBuffer, size, 3)?;
                w.blob(r + R::pWorkingGraph, size, 3)?;
            }
        }
    }
    {
        use L::nav_obstacle_bounds_s as B;
        use L::nav_obstacle_hull_s as H;
        let n = u32f(w, h + N::numObstacleBounds)?;
        if let Some(at) = w.inline_array(h + N::obstacleBounds, B::SIZE * n, 7)? {
            for i in 0..n {
                let b = at + B::SIZE * i;
                let hulls = u32f(w, b + B::m_NumHulls)?;
                if let Some(ha) = w.inline_array(b + B::m_Hulls, H::SIZE * hulls, 7)? {
                    for j in 0..hulls {
                        let hl = ha + H::SIZE * j;
                        let nb = u32f(w, hl + H::m_NumBoundaries)?;
                        w.blob(hl + H::m_Boundaries, plane * nb, 3)?;
                    }
                }
            }
        }
    }
    for (count, field, size, bf, nbf) in [
        (N::numGlassBounds, N::glassBounds, L::nav_glass_bounds_s::SIZE, L::nav_glass_bounds_s::m_Boundaries, L::nav_glass_bounds_s::m_NumBoundaries),
        (N::numModifiers, N::modifiers, L::nav_modifier_s::SIZE, L::nav_modifier_s::m_Boundaries, L::nav_modifier_s::m_NumBoundaries),
    ] {
        let n = u32f(w, h + count)?;
        if let Some(at) = w.inline_array(h + field, size * n, 7)? {
            for i in 0..n {
                let e = at + size * i;
                let nb = u32f(w, e + nbf)?;
                w.blob(e + bf, plane * nb, 3)?;
            }
        }
    }
    let n = u32f(w, h + N::numLinkCreationData)?;
    w.blob(h + N::linkCreationData, L::nav_link_creation_data_s::SIZE * n, 3)?;
    let n = u32f(w, h + N::numVolumeSeeds)?;
    w.blob(h + N::volumeSeeds, 12 * n, 3)?;
    let n = u32f(w, h + N::numRawVolumes)?;
    w.blob(h + N::rawVolumes, L::nav_raw_volume_s::SIZE * n, 3)?;
    {
        use L::nav_raw_custom_volume_s as V;
        let n = u32f(w, h + N::numRawCustomVolumes)?;
        if let Some(at) = w.inline_array(h + N::rawCustomVolumes, V::SIZE * n, 7)? {
            for i in 0..n {
                let v = at + V::SIZE * i;
                let t = u32f(w, v + V::m_NumTris)?;
                w.blob(v + V::m_Tris, L::nav_raw_custom_volume_tri_s::SIZE * t, 3)?;
            }
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::NavMesh, name))
}

pub(super) fn impact_fx(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::FxImpactEntry as E;
    use L::FxImpactTable as T;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + T::name)?.unwrap_or_default();
    if let Some(at) = w.inline_array(h + T::table, E::SIZE * 49, 7)? {
        let fx = L::FxCombinedDef::SIZE;
        for i in 0..49 {
            let e = at + E::SIZE * i;
            for j in 0..64 {
                fx_combined(w, e + E::nonflesh + fx * j)?;
            }
            for j in 0..23 {
                fx_combined(w, e + E::flesh + fx * j)?;
            }
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::ImpactFx, name))
}

pub(super) fn fxworld(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::FxGlassDef as D;
    use L::FxWorld as F;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + F::name)?.unwrap_or_default();
    let defs = u32f(w, h + F::glassSys__defCount)?;
    if let Some(at) = w.inline_array(h + F::glassSys__defs, D::SIZE * defs, 7)? {
        for i in 0..defs {
            let d = at + D::SIZE * i;
            w.asset(AssetType::PhysicsAsset, d + D::physicsAsset)?;
            w.asset(AssetType::Material, d + D::material)?;
            w.asset(AssetType::Material, d + D::materialShattered)?;
            for f in [D::pieceBreakEffect, D::shatterEffect, D::shatterSmallEffect, D::crackDecalEffect] {
                fx_combined(w, d + f)?;
            }
            for f in [D::damagedSound, D::destroyedSound, D::destroyedQuietSound] {
                w.xstr(d + f)?;
            }
        }
    }
    let limit = u32f(w, h + F::glassSys__pieceLimit)?;
    let words = u32f(w, h + F::glassSys__pieceWordCount)?;
    let cells = u32f(w, h + F::glassSys__cellCount)?;
    let geo_limit = u32f(w, h + F::glassSys__geoDataLimit)?;
    rt(w, h + F::glassSys__piecePlaces, limit, L::FxGlassPiecePlace::SIZE, 3)?;
    rt(w, h + F::glassSys__pieceStates, limit, L::FxGlassPieceState::SIZE, 3)?;
    rt(w, h + F::glassSys__pieceDynamics, limit, L::FxGlassPieceDynamics::SIZE, 3)?;
    rt(w, h + F::glassSys__geoData, geo_limit, L::FxGlassGeometryData::SIZE, 3)?;
    rt(w, h + F::glassSys__isInUse, words, 4, 3)?;
    rt(w, h + F::glassSys__cellBits, words * cells, 4, 3)?;
    rt(w, h + F::glassSys__visData, (limit + 15) & !15, 1, 15)?;
    rt(w, h + F::glassSys__linkOrg, limit, 28, 3)?;
    rt(w, h + F::glassSys__halfThickness, (limit + 3) & !3, 4, 15)?;
    let init = u32f(w, h + F::glassSys__initPieceCount)?;
    let init_geo = u32f(w, h + F::glassSys__initGeoDataCount)?;
    w.blob(h + F::glassSys__lightingHandles, 2 * init, 1)?;
    w.blob(h + F::glassSys__initPieceStates, L::FxGlassInitPieceState::SIZE * init, 3)?;
    w.blob(h + F::glassSys__initGeoData, L::FxGlassGeometryData::SIZE * init_geo, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::FxWorld, name))
}

pub(super) fn gfx_light_map(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::GfxLightMap as G;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + G::name)?.unwrap_or_default();
    let mut textures = [None; 3];
    for (i, t) in textures.iter_mut().enumerate() {
        *t = w.asset(AssetType::Image, h + G::textures + 8 * i)?;
    }
    w.s.pop();
    w.cap.light_maps.push(GfxLightMapRec { name, textures });
    Ok(AssetId { ty: AssetType::GfxLightMap, index: (w.cap.light_maps.len() - 1) as u32 })
}

/// An `IesProfile` (no tools struct): its name, then its candela table
/// (`count` floats; the tools have no writer, the layout is the game's).
pub(super) fn ies_profile(w: &mut Walker, h: usize) -> Result<AssetId> {
    w.s.push(VIRTUAL);
    let name = w.xstr(h)?.unwrap_or_default();
    let n = u32f(w, h + 8)?;
    w.blob(h + 16, 4 * n, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::IesProfile, name))
}

pub(super) fn gfx_world_tr(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::GfxWorldTransientZone as T;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + T::name)?.unwrap_or_default();
    let index = w.s.u32_at(h + T::transientZoneIndex)?;
    let vertex_count = w.s.u32_at(h + T::vertexCount)?;
    let vertices = w.blob(h + T::vd__vertices, L::GfxWorldVertex::SIZE * vertex_count as usize, 3)?;
    let layer = u32f(w, h + T::vertexLayerDataSize)?;
    let layer_data = arr(w, h + T::vld__data, layer, 1, 3)?;
    let cells = u32f(w, h + T::cellCount)?;
    let counts = w.blob(h + T::aabbTreeCounts, 4 * cells, 3)?;
    if let Some(trees) = w.inline_array(h + T::aabbTrees, 8 * cells, 127)? {
        use L::GfxAabbTree as A;
        for i in 0..cells {
            let n = match counts {
                Some(c) => u32f(w, c + 4 * i)?,
                None => 0,
            };
            // The loader reads the tree's count again in TEMP first.
            if w.s.ptr_at(trees + 8 * i)?.is_inline() {
                w.s.push(TEMP);
                w.data(4, 3)?;
                w.s.pop();
            }
            if let Some(at) = w.inline_array(trees + 8 * i, A::SIZE * n, 7)? {
                for j in 0..n {
                    let a = at + A::SIZE * j;
                    let k = u32f(w, a + A::smodelIndexCount)?;
                    w.blob(a + A::smodelIndexes, 2 * k, 1)?;
                }
            }
        }
    }
    w.s.pop();
    w.cap.gfx_world_trs.push(GfxWorldTrRec { name, header: h, index, vertices, vertex_count, layer_data });
    Ok(AssetId { ty: AssetType::GfxWorldTransientZone, index: (w.cap.gfx_world_trs.len() - 1) as u32 })
}

pub(super) fn gfxworld(w: &mut Walker, h: usize) -> Result<AssetId> {
    use AssetType as T;
    use L::GfxWorld as G;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + G::name)?.unwrap_or_default();
    w.xstr(h + G::baseName)?;
    let mut rec = GfxWorldRec { name: name.clone(), header: h, ..Default::default() };

    // Skies.
    {
        use L::GfxSky as S;
        let n = u32f(w, h + G::skyCount)?;
        if let Some(at) = w.inline_array(h + G::skies, S::SIZE * n, 7)? {
            for i in 0..n {
                let s = at + S::SIZE * i;
                let k = u32f(w, s + S::skySurfCount)?;
                let mut surfs = Vec::new();
                if let Some(p) = w.blob(s + S::skyStartSurfs, 4 * k, 3)? {
                    for j in 0..k {
                        surfs.push(w.s.i32_at(p + 4 * j)?);
                    }
                }
                rec.sky_surfs.push(surfs);
                rec.sky_images.push(w.asset(T::Image, s + S::skyImage)?);
            }
        }
    }
    let cells = u32f(w, h + G::dpvsPlanes__cellCount)?;
    let planes = u32f(w, h + G::planeCount)?;
    let nodes = u32f(w, h + G::nodeCount)?;
    w.blob(h + G::dpvsPlanes__planes, 20 * planes, 3)?;
    w.blob(h + G::dpvsPlanes__nodes, 2 * nodes, 1)?;
    rt(w, h + G::dpvsPlanes__sceneEntCellBits, cells << 9, 4, 3)?;
    w.blob(h + G::cellTransientInfos, L::GfxCellTransientInfo::SIZE * cells, 127)?;
    {
        use L::GfxCell as C;
        use L::GfxPortal as P;
        if let Some(at) = w.inline_array(h + G::cells, C::SIZE * cells, 7)? {
            for i in 0..cells {
                let c = at + C::SIZE * i;
                let n = u32f(w, c + C::portalCount)?;
                if let Some(pa) = w.inline_array(c + C::portals, P::SIZE * n, 7)? {
                    for j in 0..n {
                        let p = pa + P::SIZE * j;
                        let k = w.s.u8_at(p + P::vertexCount)? as usize;
                        w.blob(p + P::vertices, 12 * k, 3)?;
                    }
                }
            }
        }
    }
    // Reflection probes.
    {
        use L::GfxReflectionProbe as R;
        use L::GfxReflectionProbeInstance as I;
        let n = u32f(w, h + G::draw__reflectionProbeData__reflectionProbeCount)?;
        if let Some(at) = w.inline_array(h + G::draw__reflectionProbeData__reflectionProbes, R::SIZE * n, 7)? {
            for i in 0..n {
                let r = at + R::SIZE * i;
                let o = |k: usize| w.s.f32_at(r + R::origin + 4 * k);
                rec.probe_origins.push([o(0)?, o(1)?, o(2)?]);
                w.xstr(r + R::livePath)?;
                let k = u32f(w, r + R::probeInstanceCount)?;
                w.blob(r + R::probeInstances, 4 * k, 3)?;
            }
        }
        if let Some(id) = w.asset(T::Image, h + G::draw__reflectionProbeData__reflectionProbeArrayImage)? {
            rec.images.push(("reflectionProbeArray", id));
        }
        let k = u32f(w, h + G::draw__reflectionProbeData__probeRelightingCount)?;
        w.blob(h + G::draw__reflectionProbeData__probeRelightingData, L::GfxReflectionProbeRelightingData::SIZE * k, 3)?;
        let g = u32f(w, h + G::draw__reflectionProbeData__reflectionProbeGBufferImageCount)?;
        if let Some(at) = w.inline_array(h + G::draw__reflectionProbeData__reflectionProbeGBufferImages, 8 * g, 7)? {
            for j in 0..g {
                w.asset(T::Image, at + 8 * j)?;
            }
        }
        rt(w, h + G::draw__reflectionProbeData__reflectionProbeGBufferTextures, g, 8, 3)?;
        let m = u32f(w, h + G::draw__reflectionProbeData__reflectionProbeInstanceCount)?;
        if let Some(at) = w.inline_array(h + G::draw__reflectionProbeData__reflectionProbeInstances, I::SIZE * m, 7)? {
            for j in 0..m {
                w.xstr(at + I::SIZE * j + I::livePath)?;
                w.xstr(at + I::SIZE * j + I::livePath2)?;
            }
        }
        w.blob(
            h + G::draw__reflectionProbeData__reflectionProbeLightgridSampleData,
            L::GfxReflectionProbeSampleData::SIZE * n,
            63,
        )?;
    }
    let k = u32f(w, h + G::draw__lightmapReindexData__reindexCount)?;
    rec.lightmap_reindex = arr(w, h + G::draw__lightmapReindexData__reindexElement, k, L::GfxWorldLightmapReindexDataElement::SIZE, 3)?;
    let k = u32f(w, h + G::draw__lightmapReindexData__packedLightmapCount)?;
    rec.packed_lightmaps = arr(w, h + G::draw__lightmapReindexData__packedLightmap, k, L::GfxWorldPackedLightmap::SIZE, 3)?;
    if let Some(id) = w.asset(T::Image, h + G::draw__iesLookupTexture)? {
        rec.images.push(("iesLookup", id));
    }
    let k = u32f(w, h + G::draw__decalVolumeCollectionCount)?;
    w.blob(h + G::draw__decalVolumeCollections, L::GfxDecalVolumeCollection::SIZE * k, 3)?;
    if let Some(id) = w.asset(T::Image, h + G::draw__lightmapOverridePrimary)? {
        rec.images.push(("lightmapOverridePrimary", id));
    }
    if let Some(id) = w.asset(T::Image, h + G::draw__lightmapOverrideSecondary)? {
        rec.images.push(("lightmapOverrideSecondary", id));
    }
    let lms = u32f(w, h + G::draw__lightMapCount)?;
    if let Some(at) = w.inline_array(h + G::draw__lightMaps, 8 * lms, 7)? {
        for j in 0..lms {
            let id = w.asset(T::GfxLightMap, at + 8 * j)?;
            rec.light_maps.push(id);
        }
    }
    rt(w, h + G::draw__lightmapTextures, lms, 8, 3)?;
    for j in 0..32 {
        let id = w.asset(T::GfxWorldTransientZone, h + G::draw__transientZones + 8 * j)?;
        rec.transient_zones.push(id);
    }
    let indices = u32f(w, h + G::draw__indexCount)?;
    rec.index_count = indices as u32;
    rec.indices = w.blob(h + G::draw__indices, 2 * indices, 3)?;
    {
        use L::GfxVolumetric as V;
        use L::GfxVolumetricMask as M;
        let n = u32f(w, h + G::draw__volumetrics__volumetricCount)?;
        if let Some(at) = w.inline_array(h + G::draw__volumetrics__volumetrics, V::SIZE * n, 7)? {
            for i in 0..n {
                let v = at + V::SIZE * i;
                w.xstr(v + V::livePath)?;
                for m in 0..4 {
                    w.asset(T::Image, v + V::masks + M::SIZE * m + M::image)?;
                }
            }
        }
    }
    // The light grid.
    {
        let stages = w.s.u8_at(h + G::lightGrid__stageCount)? as usize;
        w.blob(h + G::lightGrid__stageLightingContrastGain, 4 * stages, 3)?;
        let n = u32f(w, h + G::lightGrid__paletteEntryCount)?;
        w.blob(h + G::lightGrid__paletteEntryAddress, 4 * n, 3)?;
        let n = u32f(w, h + G::lightGrid__paletteBitstreamSize)?;
        w.blob(h + G::lightGrid__paletteBitstream, n, 0)?;
        let n = u32f(w, h + G::lightGrid__tree__nodeCount)?;
        w.blob(h + G::lightGrid__tree__p_nodeTable, 4 * n, 3)?;
        let n = u32f(w, h + G::lightGrid__tree__leafTableSize)?;
        w.blob(h + G::lightGrid__tree__p_leafTable, n, 0)?;
        let vis = u32f(w, h + G::lightGrid__probeData__gpuVisibleProbesCount)?;
        w.blob(h + G::lightGrid__probeData__gpuVisibleProbePositions, L::GfxGpuLightGridProbePosition::SIZE * vis, 3)?;
        w.blob(h + G::lightGrid__probeData__gpuVisibleProbesData, (vis + 0x2000) * 64, 63)?;
        let probes = u32f(w, h + G::lightGrid__probeData__probeCount)?;
        let data = w.blob(h + G::lightGrid__probeData__probes, probes * 64, 63)?;
        let positions = w.blob(h + G::lightGrid__probeData__probePositions, L::GfxGpuLightGridProbePosition::SIZE * probes, 3)?;
        rec.light_probes = data.map(|pos| Span { pos, len: probes * 64 });
        rec.light_probe_positions = positions.map(|pos| Span { pos, len: L::GfxGpuLightGridProbePosition::SIZE * probes });
        let zones = u32f(w, h + G::lightGrid__probeData__zoneCount)?;
        w.blob(h + G::lightGrid__probeData__zones, L::GfxGpuLightGridZone::SIZE * zones, 63)?;
        let tets = u32f(w, h + G::lightGrid__probeData__tetrahedronCount)?;
        w.blob(h + G::lightGrid__probeData__tetrahedrons, L::GfxGpuLightGridTetrahedron::SIZE * tets, 63)?;
        w.blob(
            h + G::lightGrid__probeData__tetrahedronNeighbors,
            L::GfxGpuLightGridTetrahedronNeighbors::SIZE * tets,
            63,
        )?;
        let visible = u32f(w, h + G::lightGrid__probeData__tetrahedronCountVisible)?;
        w.blob(h + G::lightGrid__probeData__tetrahedronVisibility, visible * 64, 63)?;
        let n = u32f(w, h + G::lightGrid__probeData__voxelStartTetrahedronCount)?;
        w.blob(
            h + G::lightGrid__probeData__voxelStartTetrahedron,
            L::GfxGpuLightGridVoxelStartTetrahedron::SIZE * n,
            63,
        )?;
    }
    {
        use L::GfxVoxelTree as V;
        let n = u32f(w, h + G::voxelTreeCount)?;
        if let Some(at) = w.inline_array(h + G::voxelTree, V::SIZE * n, 7)? {
            for i in 0..n {
                let v = at + V::SIZE * i;
                w.blob(v + V::voxelTreeHeader, L::GfxVoxelTreeHeader::SIZE, 15)?;
                let k = u32f(w, v + V::voxelTopDownViewNodeCount)?;
                w.blob(v + V::voxelTopDownViewNodeArray, L::GfxVoxelTopDownViewNode::SIZE * k, 3)?;
                let internal = u32f(w, v + V::voxelInternalNodeCount)?;
                w.blob(v + V::voxelInternalNodeArray, L::GfxVoxelInternalNode::SIZE * internal, 3)?;
                let k = u32f(w, v + V::voxelLeafNodeCount)?;
                w.blob(v + V::voxelLeafNodeArray, L::GfxVoxelLeafNode::SIZE * k, 1)?;
                let k = u32f(w, v + V::lightListArraySize)?;
                w.blob(v + V::lightListArray, 2 * k, 3)?;
                rt(w, v + V::voxelInternalNodeDynamicLightList, 2 * internal, 4, 3)?;
            }
        }
    }
    let lights = u32f(w, h + G::primaryLightCount)?;
    {
        use L::GfxFrustumLights as F;
        if let Some(at) = w.inline_array(h + G::frustumLights, F::SIZE * lights, 7)? {
            for i in 0..lights {
                let f = at + F::SIZE * i;
                let k = u32f(w, f + F::indexCount)?;
                w.blob(f + F::indices, 2 * k, 3)?;
                let k = u32f(w, f + F::vertexCount)?;
                w.blob(f + F::vertices, 32 * k, 3)?;
            }
        }
    }
    {
        use L::GfxLightViewFrustum as F;
        if let Some(at) = w.inline_array(h + G::lightViewFrustums, F::SIZE * lights, 7)? {
            for i in 0..lights {
                let f = at + F::SIZE * i;
                let k = u32f(w, f + F::planeCount)?;
                w.blob(f + F::planes, 16 * k, 3)?;
                let k = u32f(w, f + F::indexCount)?;
                w.blob(f + F::indices, 2 * k, 1)?;
                let k = u32f(w, f + F::vertexCount)?;
                w.blob(f + F::vertices, 12 * k, 3)?;
            }
        }
    }
    {
        use L::GfxHeightfield as F;
        let n = u32f(w, h + G::heightfieldCount)?;
        if let Some(at) = w.inline_array(h + G::heightfields, F::SIZE * n, 7)? {
            for i in 0..n {
                w.asset(T::Image, at + F::SIZE * i + F::image)?;
            }
        }
    }
    let n = u32f(w, h + G::lightLists__listsSize)?;
    w.blob(h + G::lightLists__lists, 2 * n, 1)?;
    let models = u32f(w, h + G::modelCount)?;
    rec.model_count = models as u32;
    rec.models = w.blob(h + G::models, L::GfxBrushModel::SIZE * models, 3)?;
    {
        use L::MaterialMemory as M;
        let n = u32f(w, h + G::materialMemoryCount)?;
        if let Some(at) = w.inline_array(h + G::materialMemory, M::SIZE * n, 7)? {
            for i in 0..n {
                w.asset(T::Material, at + M::SIZE * i + M::material)?;
            }
        }
    }
    w.asset(T::Material, h + G::sun__spriteMaterial)?;
    w.asset(T::Material, h + G::sun__flareMaterial)?;
    if let Some(id) = w.asset(T::Image, h + G::outdoorImage)? {
        rec.images.push(("outdoor", id));
    }
    w.asset(T::Material, h + G::dustMaterial)?;
    {
        let dyn0 = u32f(w, h + G::dpvsDyn__dynEntClientCount)?;
        let dyn1 = u32f(w, h + G::dpvsDyn__dynEntClientCount + 4)?;
        rt(w, h + G::cellCasterBits, cells * ((cells + 31) >> 5), 4, 3)?;
        rt(w, h + G::cellHasSunLitSurfsBits, (cells + 31) >> 5, 4, 3)?;
        rt(w, h + G::sceneDynModel, dyn0, 8, 3)?;
        rt(w, h + G::sceneDynBrush, dyn1, 8, 3)?;
        let n = u32f(w, h + G::primaryLightMotionDetectBitsEntries)?;
        rt(w, h + G::primaryLightMotionDetectBits, n, 4, 3)?;
        let n = u32f(w, h + G::entityMotionBitsEntries)?;
        rt(w, h + G::entityMotionBits, n, 4, 3)?;
        let n = u32f(w, h + G::numPrimaryLightEntityShadowVisEntries)?;
        rt(w, h + G::primaryLightEntityShadowVis, n, 4, 3)?;
        for j in 0..2 {
            let n = u32f(w, h + G::dynEntMotionBitsEntries + 4 * j)?;
            rt(w, h + G::dynEntMotionBits + 8 * j, n, 4, 3)?;
        }
        for j in 0..2 {
            let n = u32f(w, h + G::numPrimaryLightDynEntShadowVisEntries + 4 * j)?;
            rt(w, h + G::primaryLightDynEntShadowVis + 8 * j, n, 4, 3)?;
        }
    }
    {
        use L::GfxShadowGeometry as S;
        if let Some(at) = w.inline_array(h + G::shadowGeomOptimized, S::SIZE * lights, 7)? {
            for i in 0..lights {
                let s = at + S::SIZE * i;
                let k = u16f(w, s + S::surfaceCount)?;
                w.blob(s + S::sortedSurfIndex, 4 * k, 3)?;
                let k = u16f(w, s + S::smodelCount)?;
                w.blob(s + S::smodelIndex, 2 * k, 1)?;
            }
        }
    }
    {
        use L::GfxLightRegion as R;
        use L::GfxLightRegionHull as H;
        if let Some(at) = w.inline_array(h + G::lightRegion, R::SIZE * lights, 7)? {
            for i in 0..lights {
                let r = at + R::SIZE * i;
                let k = u32f(w, r + R::hullCount)?;
                if let Some(ha) = w.inline_array(r + R::hulls, H::SIZE * k, 7)? {
                    for j in 0..k {
                        let hl = ha + H::SIZE * j;
                        let n = u32f(w, hl + H::axisCount)?;
                        w.blob(hl + H::axis, L::GfxLightRegionAxis::SIZE * n, 3)?;
                    }
                }
            }
        }
    }
    let nodes = u16f(w, h + G::lightAABB__nodeCount)?;
    w.blob(h + G::lightAABB__nodeArray, L::GfxLightAABBNode::SIZE * nodes, 3)?;
    let k = u16f(w, h + G::lightAABB__lightCount)?;
    w.blob(h + G::lightAABB__lightArray, 2 * k, 1)?;

    // DPVS.
    let smodels = u32f(w, h + G::dpvs__smodelCount)?;
    let static_surfs = u32f(w, h + G::dpvs__staticSurfaceCount)?;
    let surfaces = u32f(w, h + G::surfaceCount)?;
    {
        let smodel_vis = u32f(w, h + G::dpvs__smodelVisDataCount)?;
        let surface_vis = u32f(w, h + G::dpvs__surfaceVisDataCount)?;
        const ENABLED: [usize; 21] = [0, 1, 2, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29];
        for j in ENABLED {
            rt(w, h + G::dpvs__smodelVisData + 8 * j, smodel_vis, 4, 127)?;
        }
        for j in ENABLED {
            rt(w, h + G::dpvs__surfaceVisData + 8 * j, surface_vis, 4, 127)?;
        }
        for (count, field) in [
            (G::dpvs__primaryLightVisDataCount, G::dpvs__primaryLightVisData),
            (G::dpvs__reflectionProbeVisDataCount, G::dpvs__reflectionProbeVisData),
            (G::dpvs__volumetricVisDataCount, G::dpvs__volumetricVisData),
            (G::dpvs__decalVisDataCount, G::dpvs__decalVisData),
        ] {
            let n = u32f(w, h + count)?;
            rt(w, h + field, n, 4, 127)?;
        }
        w.blob(h + G::dpvs__lodData, 4 * (smodels + 1), 3)?;
        rt(w, h + G::dpvs__tessellationCutoffVisData, surface_vis, 4, 127)?;
    }
    rec.static_surface_count = static_surfs as u32;
    rec.sorted_surfs = w.blob(h + G::dpvs__sortedSurfIndex, 4 * static_surfs, 3)?;
    rec.smodel_insts = w.blob(h + G::dpvs__smodelInsts, L::GfxStaticModelInst::SIZE * smodels, 3)?;
    {
        use L::GfxSurface as S;
        rec.surface_count = surfaces as u32;
        if let Some(at) = w.inline_array(h + G::dpvs__surfaces, S::SIZE * surfaces, 7)? {
            rec.surfaces = Some(at);
            for i in 0..surfaces {
                let m = w.asset(T::Material, at + S::SIZE * i + S::material)?;
                rec.surface_materials.push(m);
            }
        }
    }
    rec.surface_bounds = w.blob(h + G::dpvs__surfacesBounds, L::GfxSurfaceBounds::SIZE * surfaces, 3)?;
    {
        use L::GfxStaticModelDrawInst as D;
        rec.smodel_count = smodels as u32;
        if let Some(at) = w.inline_array(h + G::dpvs__smodelDrawInsts, D::SIZE * smodels, 15)? {
            rec.smodels = Some(at);
            for i in 0..smodels {
                let d = at + D::SIZE * i;
                let m = w.asset(T::XModel, d + D::model)?;
                rec.smodel_models.push(m);
                let k = u32f(w, d + D::vertexLightingInfo__numLightingValues)?;
                let at = w.blob(d + D::vertexLightingInfo__lightingValues, L::GfxStaticModelVertexLighting::SIZE * k, 3)?;
                rec.smodel_lighting.push(at.map(|pos| Span { pos, len: L::GfxStaticModelVertexLighting::SIZE * k }));
            }
        }
    }
    {
        let surface_vis = u32f(w, h + G::dpvs__surfaceVisDataCount)?;
        rt(w, h + G::dpvs__surfaceMaterials, surfaces, L::GfxDrawSurf::SIZE, 7)?;
        rt(w, h + G::dpvs__surfaceCastsSunShadow, surface_vis, 4, 127)?;
    }
    w.blob(h + G::dpvs__sortedSmodelIndices, 2 * smodels, 1)?;
    {
        let opt = u32f(w, h + G::dpvs__sunShadowOptCount)?;
        let sun_vis = u32f(w, h + G::dpvs__sunSurfVisDataCount)?;
        rt(w, h + G::dpvs__surfaceCastsSunShadowOpt, opt * sun_vis, 4, 127)?;
        rt(w, h + G::dpvs__constantBuffers, smodels, 8, 15)?;
    }
    {
        let words = [u32f(w, h + G::dpvsDyn__dynEntClientWordCount)?, u32f(w, h + G::dpvsDyn__dynEntClientWordCount + 4)?];
        for j in 0..2 {
            rt(w, h + G::dpvsDyn__dynEntCellBits + 8 * j, words[j] * cells, 4, 3)?;
        }
        for i in 0..3 {
            for j in 0..2 {
                rt(w, h + G::dpvsDyn__dynEntVisData + 240 * j + 8 * i, 32 * words[j], 1, 15)?;
            }
        }
    }
    let n = u32f(w, h + G::heroOnlyLightCount)?;
    w.blob(h + G::heroOnlyLights, L::GfxHeroOnlyLight::SIZE * n, 3)?;
    let n = u32f(w, h + G::numUmbraGates)?;
    w.blob(h + G::umbraGates, L::UmbraGate::SIZE * n, 3)?;
    let n = u32f(w, h + G::umbraTomeSize)?;
    w.blob(h + G::umbraTomeData, n, 15)?;
    let n = u32f(w, h + G::numUmbraGates2)?;
    w.blob(h + G::umbraGates2, L::UmbraGate2::SIZE * n, 3)?;
    let n = u32f(w, h + G::umbraTomeSize2)?;
    w.blob(h + G::umbraTomeData2, n, 15)?;
    let n = u32f(w, h + G::umbraUnkSize)?;
    w.blob(h + G::umbraUnkData, n, 15)?;
    w.s.pop();
    w.cap.gfx_worlds.push(rec);
    Ok(AssetId { ty: T::GfxWorld, index: (w.cap.gfx_worlds.len() - 1) as u32 })
}

/// `MapTriggers`: models (their physics assets when inline), hulls, slabs,
/// windings, winding points.
/// A `MapTriggers`: where its models and hulls are (`None` when not inline).
fn triggers(w: &mut Walker, t: usize) -> Result<(Option<Span>, Option<Span>)> {
    use L::MapTriggers as M;
    use L::TriggerModel as TM;
    let n = u32f(w, t + M::count)?;
    let models = w.inline_array(t + M::models, TM::SIZE * n, 7)?;
    if let Some(at) = models {
        for i in 0..n {
            w.asset(AssetType::PhysicsAsset, at + TM::SIZE * i + TM::physicsAsset)?;
        }
    }
    let models = models.map(|pos| Span { pos, len: TM::SIZE * n });
    let n = u32f(w, t + M::hullCount)?;
    let hulls = arr(w, t + M::hulls, n, L::TriggerHull::SIZE, 3)?;
    let n = u32f(w, t + M::slabCount)?;
    w.blob(t + M::slabs, L::TriggerSlab::SIZE * n, 3)?;
    let n = u32f(w, t + M::windingCount)?;
    w.blob(t + M::windings, L::TriggerWinding::SIZE * n, 3)?;
    let n = u32f(w, t + M::windingPointCount)?;
    w.blob(t + M::windingPoints, L::TriggerWindingPoint::SIZE * n, 3)?;
    Ok((models, hulls))
}

fn clip_info(w: &mut Walker, info: usize) -> Result<()> {
    let n = u32f(w, info + L::ClipInfo::planeCount)?;
    w.blob(info + L::ClipInfo::planes, 20 * n, 3)?;
    Ok(())
}

/// `ScriptableInstanceContextHeaderLocalClient`: the context and ten ints.
const LOCAL_CLIENT_HEADER: usize = 152;

pub(super) fn mapents(w: &mut Walker, h: usize) -> Result<AssetId> {
    use AssetType as T;
    use L::MapEnts as M;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + M::name)?.unwrap_or_default();
    let chars = u32f(w, h + M::numEntityChars)?;
    let entity_string = match w.blob(h + M::entityString, chars, 0)? {
        Some(at) => {
            let b = w.s.slice(at, chars)?;
            let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
            super::text(&b[..end])
        }
        None => String::new(),
    };
    let mut rec = MapEntsRec { name: name.clone(), entity_string, ..Default::default() };
    (rec.trigger_models, rec.trigger_hulls) = triggers(w, h + M::trigger)?;
    triggers(w, h + M::clientTrigger__trigger)?;
    {
        let n = u32f(w, h + M::clientTrigger__trigger__count)?;
        let len = u32f(w, h + M::clientTrigger__triggerStringLength)?;
        w.blob(h + M::clientTrigger__triggerString, len, 0)?;
        w.blob(h + M::clientTrigger__visionSetTriggers, 2 * n, 1)?;
        w.blob(h + M::clientTrigger__triggerType, n, 0)?;
        w.blob(h + M::clientTrigger__origins, 12 * n, 3)?;
        w.blob(h + M::clientTrigger__scriptDelay, 4 * n, 3)?;
        w.blob(h + M::clientTrigger__audioTriggers, 2 * n, 1)?;
        w.blob(h + M::clientTrigger__blendLookup, 2 * n, 1)?;
        w.blob(h + M::clientTrigger__npcTriggers, 2 * n, 1)?;
        w.blob(h + M::clientTrigger__audioStateIds, 2 * n, 1)?;
        w.blob(h + M::clientTrigger__audioRvbPanInfo, L::CTAudRvbPanInfo::SIZE * n, 3)?;
        w.blob(h + M::clientTrigger__transientIndex, 2 * n, 1)?;
        if let Some(at) = w.inline_array(h + M::clientTrigger__linkTo, 8 * n, 7)? {
            for i in 0..n {
                w.blob(at + 8 * i, L::ClientEntityLinkToDef::SIZE, 3)?;
            }
        }
    }
    let n = u32f(w, h + M::clientTriggerBlend__numClientTriggerBlendNodes)?;
    w.blob(h + M::clientTriggerBlend__blendNodes, L::ClientTriggerBlendNode::SIZE * n, 3)?;
    let n = u16f(w, h + M::spawnList__spawnsCount)?;
    rec.spawn_count = n as u32;
    rec.spawns = arr(w, h + M::spawnList__spawns, n, L::SpawnPointEntityRecord::SIZE, 3)?;
    {
        use L::SplinePointRecordList as S;
        let n = u16f(w, h + M::splineList__splineCount)?;
        if let Some(at) = w.inline_array(h + M::splineList__splines, S::SIZE * n, 7)? {
            for i in 0..n {
                let s = at + S::SIZE * i;
                let k = u16f(w, s + S::splinePointCount)?;
                w.blob(s + S::splinePoints, L::SplinePointEntityRecord::SIZE * k, 3)?;
            }
        }
    }
    let n = u32f(w, h + M::havokEntsShapeDataSize)?;
    w.blob(h + M::havokEntsShapeData, n, 15)?;
    {
        use L::cmodel_t as C;
        let n = u32f(w, h + M::numSubModels)?;
        rec.cmodel_count = n as u32;
        if let Some(at) = w.inline_array(h + M::cmodels, C::SIZE * n, 7)? {
            rec.cmodels = Some(Span { pos: at, len: C::SIZE * n });
            for i in 0..n {
                let c = at + C::SIZE * i;
                if let Some(info) = w.inline_array(c + C::info, L::ClipInfo::SIZE, 7)? {
                    clip_info(w, info)?;
                }
                w.asset(T::PhysicsAsset, c + C::physicsAsset)?;
            }
        }
    }
    let counts = [u16f(w, h + M::dynEntCount)?, u16f(w, h + M::dynEntCount + 2)?];
    {
        use L::DynEntityDef as D;
        for j in 0..2 {
            let n = counts[j];
            if let Some(at) = w.inline_array(h + M::dynEntDefList + 8 * j, D::SIZE * n, 7)? {
                rec.dyn_ents[j] = Some(Span { pos: at, len: D::SIZE * n });
                for i in 0..n {
                    let d = at + D::SIZE * i;
                    let m = w.asset(T::XModel, d + D::baseModel)?;
                    rec.dyn_ent_models[j].push(m);
                    w.blob(d + D::linkTo, L::DynEntityLinkToDef::SIZE, 3)?;
                }
            }
        }
    }
    {
        use L::DynEntityPose as P;
        // [client][list]: [0][0], [1][0], [0][1], [1][1].
        for (c, j) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let n = counts[j];
            let field = h + M::dynEntPoseList + 16 * c + 8 * j;
            if let Some(at) = w.inline_array(field, P::SIZE * n, 7)? {
                for i in 0..n {
                    let p = at + P::SIZE * i;
                    let k = u32f(w, p + P::numPoses)?;
                    w.blob(p + P::poses, L::GfxPlacement::SIZE * k, 3)?;
                    // (Read in TEMP: not in VIRTUAL.)
                    w.s.push(TEMP);
                    w.blob(p + P::detailBodyToBoneMap, k, 0)?;
                    w.s.pop();
                }
            }
        }
    }
    for (c, j) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        rt(w, h + M::dynEntClientList + 16 * c + 8 * j, counts[j], L::DynEntityClient::SIZE, 3)?;
    }
    let total = u32f(w, h + M::dynEntCountTotal)?;
    for c in 0..2 {
        w.blob(h + M::dynEntGlobalIdList + 8 * c, L::DynEntityGlobalId::SIZE * total, 3)?;
    }
    {
        use L::DynEntityTransientGroup as G;
        let n = u32f(w, h + M::dynEntTransientGroupCount)?;
        if let Some(at) = w.inline_array(h + M::dynEntTransientGroups, G::SIZE * n, 7)? {
            for i in 0..n {
                let g = at + G::SIZE * i;
                let k = u32f(w, g + G::dynEntCount)?;
                w.blob(g + G::dynEnts, L::DynEntityTransientEntry::SIZE * k, 3)?;
            }
        }
        for c in 0..2 {
            w.blob(h + M::dynEntTransientGroupRuntime + 8 * c, L::DynEntityTransientGroupRuntime::SIZE * n, 3)?;
        }
        for c in 0..2 {
            w.blob(h + M::dynEntTransientGroupState + 8 * c, L::DynEntityTransientGroupState::SIZE * n, 3)?;
        }
    }
    let n = u32f(w, h + M::unk3Count)?;
    w.blob(h + M::unk3, 4 * n, 3)?;
    let n = u32f(w, h + M::clientEntAnchorCount)?;
    w.blob(h + M::clientEntAnchors, L::ClientEntAnchor::SIZE * n, 3)?;
    {
        use L::ScriptableInstance as I;
        use L::ScriptableInstanceContextHeader as C;
        let n = u32f(w, h + M::scriptableMapEnts__totalInstanceCount)?;
        rec.scriptable_count = n as u32;
        if let Some(at) = w.inline_array(h + M::scriptableMapEnts__instances, I::SIZE * n, 7)? {
            rec.scriptables = Some(Span { pos: at, len: I::SIZE * n });
            for i in 0..n {
                let s = at + I::SIZE * i;
                for (k, ctx) in [s + I::contextHeader, s + I::contextHeaderLocalClient, s + I::contextHeaderLocalClient + LOCAL_CLIENT_HEADER]
                    .into_iter()
                    .enumerate()
                {
                    let def = w.asset(T::Scriptable, ctx + C::context__def)?;
                    let model = w.asset(T::XModel, ctx + C::context__modelData__model)?;
                    if k == 0 {
                        rec.scriptable_defs.push(def);
                        rec.scriptable_models.push(model);
                    }
                    let len = u32f(w, ctx + C::context__eventStreamBufferSize)?;
                    w.blob(ctx + C::context__eventStreamBuffer, len, 0)?;
                }
                w.xstr(s + I::debugName)?;
            }
            let n = u32f(w, h + M::scriptableMapEnts__runtimeData__partRuntimeCount)?;
            w.blob(h + M::scriptableMapEnts__runtimeData__partRuntime, L::ScriptablePartRuntime::SIZE * n, 3)?;
            let n = u32f(w, h + M::scriptableMapEnts__runtimeData__partRuntimeLocalClientCount)?;
            for c in 0..2 {
                w.blob(
                    h + M::scriptableMapEnts__runtimeData__partRuntimeLocalClient + 8 * c,
                    L::ScriptablePartRuntime::SIZE * n,
                    3,
                )?;
            }
            use L::ScriptableReservedDynents as R;
            for c in 0..2 {
                let r = h + M::scriptableMapEnts__reservedDynents + R::SIZE * c;
                let k = u32f(w, r + R::numReservedDynents)?;
                w.blob(r + R::reservedDynents, L::ScriptableReservedDynent::SIZE * k, 3)?;
            }
        }
    }
    {
        use L::MayhemInstance as I;
        let n = u32f(w, h + M::numMayhemScenes)?;
        if let Some(at) = w.inline_array(h + M::mayhemScenes, I::SIZE * n, 7)? {
            for i in 0..n {
                let m = at + I::SIZE * i;
                w.asset(T::Mayhem, m + I::mayhem)?;
                w.blob(m + I::linkTo, L::DynEntityLinkToDef::SIZE, 3)?;
            }
        }
    }
    {
        use L::Spawner as S;
        let n = u32f(w, h + M::spawners__spawnerCount)?;
        rec.spawner_count = n as u32;
        if let Some(at) = w.inline_array(h + M::spawners__spawnerList, S::SIZE * n, 7)? {
            rec.spawners = Some(Span { pos: at, len: S::SIZE * n });
            for i in 0..n {
                let s = at + S::SIZE * i;
                let k = u32f(w, s + S::numFields)?;
                w.blob(s + S::fields, L::SpawnerField::SIZE * k, 3)?;
            }
        }
    }
    let n = u32f(w, h + M::audioPASpeakerCount)?;
    w.blob(h + M::audioPASpeakers, L::AudioPASpeaker::SIZE * n, 3)?;
    w.s.pop();
    w.cap.map_ents.push(rec);
    Ok(AssetId { ty: T::MapEnts, index: (w.cap.map_ents.len() - 1) as u32 })
}

pub(super) fn glassworld(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::G_GlassData as D;
    use L::G_GlassName as N;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + L::GlassWorld::name)?.unwrap_or_default();
    if let Some(d) = w.inline_array(h + L::GlassWorld::g_glassData, D::SIZE, 7)? {
        let n = u32f(w, d + D::pieceCount)?;
        w.blob(d + D::glassPieces, L::G_GlassPiece::SIZE * n, 3)?;
        let n = u32f(w, d + D::glassNameCount)?;
        if let Some(at) = w.inline_array(d + D::glassNames, N::SIZE * n, 7)? {
            for i in 0..n {
                let g = at + N::SIZE * i;
                w.xstr(g + N::nameStr)?;
                let k = u16f(w, g + N::pieceCount)?;
                w.blob(g + N::pieceIndices, 2 * k, 1)?;
            }
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::GlassWorld, name))
}

/// A `pathnode_tree_t` whose struct is already read: its nodes or children.
fn node_tree(w: &mut Walker, t: usize, depth: usize) -> Result<()> {
    use L::pathnode_tree_t as P;
    if depth > 64 {
        return Ok(());
    }
    if w.s.i32_at(t + P::axis)? < 0 {
        let n = u32f(w, t + P::u__s__nodeCount)?;
        w.blob(t + P::u__s__nodes, 2 * n, 1)?;
    } else {
        for i in 0..2 {
            if let Some(c) = w.inline_array(t + P::u__child + 8 * i, P::SIZE, 7)? {
                node_tree(w, c, depth + 1)?;
            }
        }
    }
    Ok(())
}

pub(super) fn pathdata(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PathData as P;
    use L::pathnode_t as N;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let n = u32f(w, h + P::nodeCount)?;
    let mut rec = PathDataRec { name: name.clone(), ..Default::default() };
    if let Some(at) = w.inline_array(h + P::nodes, N::SIZE * n, 7)? {
        rec.nodes = Some(Span { pos: at, len: N::SIZE * n });
        for i in 0..n {
            let node = at + N::SIZE * i;
            let k = u16f(w, node + N::constant__totalLinkCount)?;
            let links = w.blob(node + N::constant__Links, L::pathlink_s::SIZE * k, 3)?;
            rec.links.push(links.map(|pos| Span { pos, len: L::pathlink_s::SIZE * k }));
        }
    }
    let n = u32f(w, h + P::visBytes)?;
    w.blob(h + P::pathVis, n, 0)?;
    {
        use L::pathnode_tree_t as T;
        let n = u32f(w, h + P::nodeTreeCount)?;
        if let Some(at) = w.inline_array(h + P::nodeTree, T::SIZE * n, 7)? {
            for i in 0..n {
                node_tree(w, at + T::SIZE * i, 0)?;
            }
        }
        use L::PathDynamicNodeGroup as G;
        let n = u16f(w, h + P::dynamicNodeGroupCount)?;
        if let Some(at) = w.inline_array(h + P::dynamicNodeGroups, G::SIZE * n, 7)? {
            for i in 0..n {
                let g = at + G::SIZE * i;
                let k = u32f(w, g + G::nodeTreeCount)?;
                if let Some(ta) = w.inline_array(g + G::nodeTree, T::SIZE * k, 7)? {
                    for j in 0..k {
                        node_tree(w, ta + T::SIZE * j, 0)?;
                    }
                }
            }
        }
    }
    for (count, field) in [
        (P::exposureBytes, P::pathExposure),
        (P::noPeekVisBytes, P::pathNoPeekVis),
        (P::zonesBytes, P::pathZones),
        (P::dynStatesBytes, P::pathDynStates),
    ] {
        let n = u32f(w, h + count)?;
        w.blob(h + field, n, 0)?;
    }
    w.s.pop();
    w.cap.path_datas.push(rec);
    Ok(w.name_only(AssetType::PathData, name))
}

pub(super) fn clipmap(w: &mut Walker, h: usize) -> Result<AssetId> {
    use AssetType as T;
    use L::clipMap_t as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::name)?.unwrap_or_default();
    clip_info(w, h + C::info)?;
    w.s.push(TEMP);
    if let Some(info) = w.inline_array(h + C::pInfo, L::ClipInfo::SIZE, 7)? {
        clip_info(w, info)?;
    }
    w.s.pop();
    let mut rec = ClipMapRec { name: name.clone(), header: h, ..Default::default() };
    {
        use L::cStaticModel_s as S;
        let n = u32f(w, h + C::numStaticModels)?;
        rec.static_model_count = n as u32;
        if let Some(at) = w.inline_array(h + C::staticModelList, S::SIZE * n, 7)? {
            rec.static_models = Some(Span { pos: at, len: S::SIZE * n });
            for i in 0..n {
                let m = w.asset(T::XModel, at + S::SIZE * i + S::xmodel)?;
                rec.static_model_models.push(m);
            }
        }
    }
    let n = u32f(w, h + C::staticModelCollisionModelList__numModels)?;
    w.blob(h + C::staticModelCollisionModelList__staticModelIndex, 4 * n, 3)?;
    {
        use L::StaticModelCollisionModelList as S;
        let n = u32f(w, h + C::numStaticModelCollisionModelLists)?;
        if let Some(at) = w.inline_array(h + C::staticModelCollisionModelLists, S::SIZE * n, 7)? {
            for i in 0..n {
                let s = at + S::SIZE * i;
                let k = u32f(w, s + S::numModels)?;
                w.blob(s + S::staticModelIndex, 4 * k, 3)?;
            }
        }
    }
    w.asset(T::MapEnts, h + C::mapEnts)?;
    {
        use L::Stage as S;
        let n = w.s.u8_at(h + C::stageCount)? as usize;
        if let Some(at) = w.inline_array(h + C::stages, S::SIZE * n, 7)? {
            for i in 0..n {
                w.xstr(at + S::SIZE * i + S::name)?;
            }
        }
    }
    triggers(w, h + C::stageTrigger)?;
    let n = u32f(w, h + C::havokWorldShapeDataSize)?;
    rec.havok_world_shape = arr(w, h + C::havokWorldShapeData, n, 1, 15)?;
    let n = u32f(w, h + C::numCollisionHeatmapEntries)?;
    w.blob(h + C::collisionHeatmap, L::CollisionHeatmapEntry::SIZE * n, 3)?;
    w.blob(h + C::topDownMapData, L::TopDownMapData::SIZE, 3)?;
    w.s.pop();
    w.cap.clip_maps.push(rec);
    Ok(AssetId { ty: T::ClipMap, index: (w.cap.clip_maps.len() - 1) as u32 })
}
