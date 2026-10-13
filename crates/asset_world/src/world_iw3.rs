//! bo2mp: Call of Duty 4 (IW3) GfxWorld and clipMap_t -> the engine's world
//! draw and collision. Not part of upstream IW4L.
//!
//! The IW3 world vertex is the same 44 bytes the renderer uploads for IW4
//! (position, binormal sign, D3DCOLOR, texcoord f32x2, lightmap coord
//! f32x2, packed normal and tangent), so each vertex is repacked as is.
//! A surface's indices are relative to its first vertex. The surfaces draw
//! through the renderer's fallback pass with their colour maps (as Black Ops
//! II's do), in one synthetic visibility cell.
//!
//! Collision: every brush with its axial and side planes (IW3 keeps IW4's
//! surface flag bits), the collision triangles under their AABB trees, and
//! the brush models' brushes through their leaf brush trees. The world's
//! BSP is not read yet: one leaf holds every brush no brush model owns, and
//! the world's triangles are every tree of the AABB forest.

use asset_iw4::size as iw4_sz;
use dpvs_iw4::{AabbNodeView, Bounds, SurfRange};
use iw3::zone::{ClipMap, GfxWorld};

use crate::clip_collision::{
    ClipBrush, ClipBspLeaf, ClipBspNode, ClipCmodel, ClipCollision, ClipCollisionError, ClipMapMaterial,
    LEAFBRUSH_NODE_MAX_DEPTH, leafbrush_node_visits, leafbrush_range, t5_axial_plane_flags,
};
use crate::world_draw::{
    CameraRangeKind, CameraSurfRange, CameraSurfRanges, DpvsWorldData, SurfaceDrawFields,
    WorldDraw, WorldLightmap, WorldLightmapGap, WorldVertexPayload,
};
use crate::world_mesh::{WorldMeshError, WorldMeshStats};
use crate::world_t6::T6WorldLighting;
use crate::{SurfaceCastsSunShadow, world_capture_from_casters};

/// Which of the world's surfaces the game draws: the lit, decal and emissive
/// ranges, not the sky's.
pub fn iw3_drawn_surfaces(world: &GfxWorld) -> Vec<bool> {
    let n = world.surfaces.len();
    let mut drawn = vec![false; n];
    for range in [&world.lit_surfs, &world.decal_surfs, &world.emissive_surfs] {
        for i in range.clone() {
            if let Some(d) = drawn.get_mut(i as usize) {
                *d = true;
            }
        }
    }
    // A world whose ranges say nothing draws everything.
    if !drawn.iter().any(|&d| d) {
        drawn = vec![true; n];
    }
    for &i in &world.sky_start_surfs {
        if let Some(d) = usize::try_from(i).ok().and_then(|i| drawn.get_mut(i)) {
            *d = false;
        }
    }
    drawn
}

pub fn build_iw3_world_draw(
    world: &GfxWorld,
    surface_materials: &[Option<usize>],
    lightmaps: Vec<Option<WorldLightmap>>,
    lighting: Option<T6WorldLighting>,
) -> Result<WorldDraw, WorldMeshError> {
    if world.surfaces.is_empty() || world.vertices.is_empty() {
        return Err(WorldMeshError::NoGeometry);
    }
    let mut packed_vertices = Vec::with_capacity(world.vertices.len());
    let mut positions = Vec::with_capacity(world.vertices.len());
    let mut normals = Vec::with_capacity(world.vertices.len());
    let mut tangents = Vec::with_capacity(world.vertices.len());
    let mut colors = Vec::with_capacity(world.vertices.len());
    let mut texture_uvs = Vec::with_capacity(world.vertices.len());
    let mut lightmap_uvs = Vec::with_capacity(world.vertices.len());
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for v in &world.vertices {
        let mut packed = [0u8; iw4_sz::GFX_WORLD_VERTEX];
        packed[0..4].copy_from_slice(&v.xyz[0].to_le_bytes());
        packed[4..8].copy_from_slice(&v.xyz[1].to_le_bytes());
        packed[8..12].copy_from_slice(&v.xyz[2].to_le_bytes());
        packed[12..16].copy_from_slice(&v.binormal_sign.to_le_bytes());
        // D3DCOLOR: bytes B, G, R, A.
        let c = v.color.to_le_bytes();
        packed[16..20].copy_from_slice(&c);
        packed[20..24].copy_from_slice(&v.tex_coord[0].to_le_bytes());
        packed[24..28].copy_from_slice(&v.tex_coord[1].to_le_bytes());
        packed[28..32].copy_from_slice(&v.lmap_coord[0].to_le_bytes());
        packed[32..36].copy_from_slice(&v.lmap_coord[1].to_le_bytes());
        packed[36..40].copy_from_slice(&v.normal.to_le_bytes());
        packed[40..44].copy_from_slice(&v.tangent.to_le_bytes());
        for axis in 0..3 {
            min[axis] = min[axis].min(v.xyz[axis]);
            max[axis] = max[axis].max(v.xyz[axis]);
        }
        let normal = asset_model::normalize_or_up(asset_model::unpack_unit_vec(v.normal));
        let tangent = asset_model::normalize_or_up(asset_model::unpack_unit_vec(v.tangent));
        packed_vertices.push(packed);
        positions.push(v.xyz);
        normals.push(normal);
        tangents.push([tangent[0], tangent[1], tangent[2], v.binormal_sign]);
        colors.push([c[2], c[1], c[0], c[3]].map(|x| f32::from(x) / 255.0));
        texture_uvs.push(v.tex_coord);
        lightmap_uvs.push(v.lmap_coord);
    }

    let drawn = iw3_drawn_surfaces(world);
    let n = world.surfaces.len();
    let mut packed_indices: Vec<u32> = Vec::new();
    let mut surface_index_ranges = Vec::with_capacity(n);
    let mut surface_first_vertex = Vec::with_capacity(n);
    let mut surface_draw_fields = Vec::with_capacity(n);
    let mut surface_lightmap_indices = Vec::with_capacity(n);
    let mut surface_reflection_probes = Vec::with_capacity(n);
    let mut surface_primary_lights = Vec::with_capacity(n);
    let mut surface_bounds = Vec::with_capacity(n);
    let vertex_count = world.vertices.len() as u32;
    for (i, surf) in world.surfaces.iter().enumerate() {
        let base = surf.first_vertex.max(0) as u32;
        let first = surf.base_index.max(0) as usize;
        let count = if drawn[i] { surf.tri_count as usize * 3 } else { 0 };
        let range_start = packed_indices.len() as u32;
        let mut kept = 0usize;
        if let Some(indices) = world.indices.get(first..first + count) {
            // A triangle that reaches past the vertex table is left out
            // (whole triangles only).
            for tri in indices.chunks_exact(3) {
                if tri.iter().all(|&ix| base + u32::from(ix) < vertex_count) {
                    packed_indices.extend(tri.iter().map(|&ix| base + u32::from(ix)));
                    kept += 3;
                }
            }
        }
        surface_index_ranges.push((range_start, kept as u32));
        surface_first_vertex.push(base);
        surface_lightmap_indices.push(surf.lightmap_index);
        surface_reflection_probes.push(surf.reflection_probe_index);
        surface_primary_lights.push(surf.primary_light_index);
        surface_draw_fields.push(SurfaceDrawFields {
            first_vertex: base,
            tri_count: (kept / 3) as u16,
            base_index: range_start,
            lightmap_index: surf.lightmap_index,
            reflection_probe_index: surf.reflection_probe_index,
            primary_light_index: surf.primary_light_index,
        });
        surface_bounds.push(Bounds::from_mins_maxs(surf.bounds[0], surf.bounds[1]));
    }

    let surface_materials: Vec<Option<usize>> = (0..n)
        .map(|i| surface_materials.get(i).copied().flatten())
        .collect();
    let surface_lightmapped: Vec<bool> = surface_lightmap_indices
        .iter()
        .map(|&i| lightmaps.get(usize::from(i)).is_some_and(Option::is_some))
        .collect();
    let lightmaps_present = lightmaps.iter().any(Option::is_some);
    let (batches, surface_batch_ranges) = crate::world_t5::make_material_batches(
        &positions,
        &normals,
        &tangents,
        &colors,
        &texture_uvs,
        &lightmap_uvs,
        &packed_indices,
        &surface_index_ranges,
        &surface_materials,
        &surface_lightmapped,
        &surface_lightmap_indices,
        &surface_primary_lights,
        &surface_reflection_probes,
    );
    let dpvs = one_cell(n, world.mins, world.maxs, surface_bounds)?;
    // No extra texture layers: one empty entry per vertex, as the T6 draw
    // keeps them.
    let layer_uvs = vec![[0.0f32; 4]; positions.len()];
    let stats = WorldMeshStats {
        vertices: positions.len(),
        triangles: packed_indices.len() / 3,
        surfaces: n,
        skipped_surfaces: drawn.iter().filter(|d| !**d).count(),
        sky_surfaces: world.sky_start_surfs.len(),
        sky_material: None,
        unrouted_surfaces: n,
        undecided_state_bits_surfaces: 0,
        min,
        max,
        bounds: None,
    };
    Ok(WorldDraw {
        batches,
        sky_model: None,
        lightmap: if lightmaps_present {
            Ok(lightmaps)
        } else {
            Err(WorldLightmapGap::Missing)
        },
        stats,
        packed_vertices: WorldVertexPayload::T6(packed_vertices),
        vertex_layer: Vec::new(),
        surface_vertex_layer: vec![0; n],
        surface_first_vertex,
        surface_draw_fields,
        positions,
        normals,
        tangents,
        colors,
        texture_uvs,
        lightmap_uvs,
        packed_indices,
        surface_index_ranges,
        surface_batch_ranges,
        surface_lightmapped,
        surface_lightmap_indices,
        surface_reflection_probes,
        surface_primary_lights,
        sort_key_distortion: None,
        capture: world_capture_from_casters(SurfaceCastsSunShadow::with_len(n)),
        brush_models: Vec::new(),
        brush_model_bounds: Vec::new(),
        surface_materials,
        primary_lights: Vec::new(),
        light_defs: Vec::new(),
        sun_primary_light_count: 0,
        light_region_hulls: None,
        shadow_geometry: Vec::new(),
        reflection_probes: Vec::new(),
        dpvs,
        outdoor_image_name: None,
        outdoor_image: None,
        outdoor_lookup: [0; 16],
        sun_effects: None,
        t5_sun_parse_exposure: lighting.map(|l| l.exposure_scale),
        t5_sky_dynamic_intensity: lighting.map(|l| l.sky),
        t5_sun_light: lighting.map(|l| l.sun),
        iw3_lightmaps: lightmaps_present,
        t5_tree_scatter_intensity: None,
        t5_tree_scatter_amount: None,
        t5_exposure_volume_count: 0,
        t6_lights: Vec::new(),
        t6_fog_banks: Vec::new(),
        t6_probes: Vec::new(),
        t6_layer_uvs: layer_uvs,
    })
}

/// bo2mp: D3DFMT_L8 and D3DFMT_A8R8G8B8, the formats of a CoD4 (and World
/// at War) lightmap's primary and secondary images.
pub const IW3_LIGHTMAP_PRIMARY_FORMAT: u32 = 50;
pub const IW3_LIGHTMAP_SECONDARY_FORMAT: u32 = 21;

/// bo2mp: one Call of Duty 4 / World at War lightmap page as the fallback
/// pass reads a lightmap: one RGBA8 image of three stacked pages, addressed
/// at `(u, v / 3 + page / 3)`, in the secondary slot (as Black Ops II's).
///
/// CoD4's own lit world shaders (`lm_*_sm3`, `lm_sm_sun_*_sm3`, read from
/// the zone's pixel shaders) sample the secondary image (A8R8G8B8, two pages
/// stacked: A at `(u, v / 2)`, B at `(u, v / 2 + 1/2)`) and the primary
/// (L8 at twice the size: the sun's baked visibility) at `(u, v)`:
/// `light = A.rgb + B.rgb * saturate(rsqrt(1 + x^2 + y^2)) + primary *
/// saturate(N.sun) * sunDiffuse` with x, y the dominant light's tangent
/// direction from the alphas (`A.a * 4.08 - 2.08`, `B.a * 4.064516 -
/// 2.064516`), all in gamma space. The pages here, at the primary's size:
/// 0 = A (alpha kept), 1 = B (alpha kept), 2 = the primary in alpha. The
/// shader's IW3 branch reads them so (`fragment_lightmapped`).
pub fn iw3_lightmap(
    primary: (u16, u16, u32, &[u8]),
    secondary: (u16, u16, u32, &[u8]),
    names: (&str, &str),
) -> Result<WorldLightmap, WorldLightmapGap> {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
    use bevy::prelude::{Image, UVec2};
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let (pw, ph, pformat, pdata) = primary;
    let (sw, sh, sformat, sdata) = secondary;
    if sformat != IW3_LIGHTMAP_SECONDARY_FORMAT {
        return Err(WorldLightmapGap::UnsupportedSecondaryFormat(sformat));
    }
    if pformat != IW3_LIGHTMAP_PRIMARY_FORMAT {
        return Err(WorldLightmapGap::UnsupportedPrimaryFormat(pformat));
    }
    if sw == 0 || sh < 2 || sh % 2 != 0 || pw == 0 || ph == 0 {
        return Err(WorldLightmapGap::InvalidDimensions { width: sw, height: sh, depth: 1 });
    }
    let (sw, page_h) = (usize::from(sw), usize::from(sh) / 2);
    let (a, b) = crate::world_draw::secondary_page_payloads(sdata, sw * 4, page_h)?;
    let mask = crate::world_draw::primary_mask_payload(pdata, pw, ph)?;
    let (w, h) = (usize::from(pw), usize::from(ph));
    if w * page_h != h * sw {
        return Err(WorldLightmapGap::MismatchedAspectRatio {
            primary: UVec2::new(w as u32, h as u32),
            secondary: UVec2::new(sw as u32, page_h as u32),
        });
    }
    // A secondary page read at the primary's texel centres, filtered as the
    // GPU's linear sampler with clamped edges does.
    let sample = |page: &[u8], x: usize, y: usize| -> [u8; 4] {
        let fx = ((x as f32 + 0.5) * sw as f32 / w as f32 - 0.5).max(0.0);
        let fy = ((y as f32 + 0.5) * page_h as f32 / h as f32 - 0.5).max(0.0);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(sw - 1), (y0 + 1).min(page_h - 1));
        let (x0, y0) = (x0.min(sw - 1), y0.min(page_h - 1));
        let (tx, ty) = (fx - fx.floor(), fy - fy.floor());
        let at = |xx: usize, yy: usize, c: usize| f32::from(page[(yy * sw + xx) * 4 + c]);
        [0, 1, 2, 3].map(|c| {
            let top = at(x0, y0, c) + (at(x1, y0, c) - at(x0, y0, c)) * tx;
            let bottom = at(x0, y1, c) + (at(x1, y1, c) - at(x0, y1, c)) * tx;
            (top + (bottom - top) * ty).round().clamp(0.0, 255.0) as u8
        })
    };
    let mut rgba = vec![0u8; w * h * 3 * 4];
    for y in 0..h {
        for x in 0..w {
            let at = |page: usize| ((page * h + y) * w + x) * 4;
            rgba[at(0)..at(0) + 4].copy_from_slice(&sample(&a, x, y));
            rgba[at(1)..at(1) + 4].copy_from_slice(&sample(&b, x, y));
            rgba[at(2)..at(2) + 4].copy_from_slice(&[128, 128, 255, mask[y * w + x]]);
        }
    }
    let mut page = Image::new(
        Extent3d { width: w as u32, height: (h * 3) as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    page.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    let stand_in = || {
        Image::new(
            Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            TextureDimension::D2,
            vec![255, 255, 255, 255],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        )
    };
    Ok(WorldLightmap {
        primary_image: None,
        secondary_image: Some(page),
        secondary_b_image: None,
        ambient_image: stand_in(),
        directional_image: stand_in(),
        sun_mask_image: stand_in(),
        ambient_source_name: names.1.to_owned(),
        sun_mask_source_name: names.0.to_owned(),
        ambient_size: UVec2::new(w as u32, h as u32 * 3),
        sun_mask_size: UVec2::new(w as u32, h as u32),
    })
}

/// One cell, one AABB node over every surface, identity sort order.
fn one_cell(
    n: usize,
    mins: [f32; 3],
    maxs: [f32; 3],
    surface_bounds: Vec<Bounds>,
) -> Result<DpvsWorldData, WorldMeshError> {
    let count = u16::try_from(n).map_err(|_| WorldMeshError::NoGeometry)?;
    let mut out = DpvsWorldData::new(CameraSurfRanges::new(
        CameraSurfRange {
            kind: CameraRangeKind::LitOpaque,
            begin: 0,
            end: n as u32,
        },
        Vec::new(),
    ));
    out.cell_count = 1;
    out.static_surface_count = n;
    out.static_surface_count_no_decal = n;
    out.lit_opaque_begin = 0;
    out.lit_opaque_end = n as u32;
    out.sorted_surf_index = (0..count).collect();
    out.surface_bounds = surface_bounds;
    out.cell_roots = vec![SurfRange { start: 0, count }];
    out.aabb_trees = vec![vec![AabbNodeView {
        bounds: Bounds::from_mins_maxs(mins, maxs),
        child_count: 0,
        children_offset: 0,
        start_surf: 0,
        surface_count: count,
        start_surf_no_decal: 0,
        surface_count_no_decal: count,
        smodel_index_start: 0,
        smodel_index_count: 0,
    }]];
    out.aabb_smodel_indices = vec![Vec::new()];
    out.portals_per_cell = vec![Vec::new()];
    out.cell_reflection_probes = vec![Vec::new()];
    out.checked()
}

/// The brushes under one brush tree node (a model's `leaf_brush_node`),
/// appended to `out`; their range there.
fn append_tree_brushes(clip: &ClipMap, root: i32, out: &mut Vec<u16>) -> Result<(u32, u16), ClipCollisionError> {
    let first_len = out.len();
    if root >= 0 && !clip.leaf_brush_nodes.is_empty() {
        let mut pending = vec![(root as usize, 0usize)];
        while let Some((index, depth)) = pending.pop() {
            if depth > LEAFBRUSH_NODE_MAX_DEPTH {
                return Err(ClipCollisionError::Truncated);
            }
            let Some(node) = clip.leaf_brush_nodes.get(index) else {
                continue;
            };
            if node.leaf_brush_count > 0 {
                out.extend_from_slice(&node.brushes);
            } else {
                pending.extend(
                    leafbrush_node_visits(
                        index,
                        node.leaf_brush_count,
                        usize::from(node.child_offsets[0]),
                        usize::from(node.child_offsets[1]),
                    )
                    .into_iter()
                    .map(|i| (i, depth + 1)),
                );
            }
        }
    }
    leafbrush_range(first_len, out.len())
}

/// The world's collision triangle trees: every root of the AABB forest (a
/// node no other node lists as a child) but the brush models' own.
pub fn legacy_aabb_roots(clip: &ClipMap) -> Vec<u16> {
    let nodes: Vec<clipmap_iw4::ClipAabbNode> = clip
        .aabb_trees
        .iter()
        .map(|t| clipmap_iw4::ClipAabbNode {
            origin: t.origin,
            half_size: t.half_size,
            material_index: t.material_index,
            child_count: t.child_count,
            u: t.index,
        })
        .collect();
    let mut roots: std::collections::BTreeSet<u16> = clipmap_iw4::aabb_forest_roots(&nodes).into_iter().collect();
    // A brush model's own trees move with it: not the world's.
    for m in clip.cmodels.iter().skip(1) {
        let first = u32::from(m.first_coll_aabb);
        for id in first..first + u32::from(m.coll_aabb_count) {
            if let Ok(id) = u16::try_from(id) {
                roots.remove(&id);
            }
        }
    }
    roots.into_iter().collect()
}

pub fn build_iw3_clip_collision(clip: &ClipMap) -> Result<ClipCollision, ClipCollisionError> {
    if clip.brushes.is_empty() && clip.tri_indices.is_empty() {
        return Err(ClipCollisionError::MissingTables);
    }
    let mut out = ClipCollision {
        brushes: Vec::with_capacity(clip.brushes.len()),
        ..ClipCollision::default()
    };
    let flags_of = |material: i64| -> u32 {
        usize::try_from(material)
            .ok()
            .and_then(|m| clip.materials.get(m))
            .map_or(0, |m| m.surface_flags as u32)
    };
    for b in &clip.brushes {
        let (mins, maxs) = (b.mins, b.maxs);
        let mut planes = Vec::with_capacity(6 + b.side_planes.len());
        planes.push([1.0, 0.0, 0.0, maxs[0]]);
        planes.push([-1.0, 0.0, 0.0, -mins[0]]);
        planes.push([0.0, 1.0, 0.0, maxs[1]]);
        planes.push([0.0, -1.0, 0.0, -mins[1]]);
        planes.push([0.0, 0.0, 1.0, maxs[2]]);
        planes.push([0.0, 0.0, -1.0, -mins[2]]);
        // Axial materials: [min x, y, z], [max x, y, z].
        let axial = [
            flags_of(i64::from(b.axial_materials[0][0])),
            flags_of(i64::from(b.axial_materials[0][1])),
            flags_of(i64::from(b.axial_materials[0][2])),
            flags_of(i64::from(b.axial_materials[1][0])),
            flags_of(i64::from(b.axial_materials[1][1])),
            flags_of(i64::from(b.axial_materials[1][2])),
        ];
        let mut plane_surface_flags = Vec::with_capacity(planes.capacity());
        plane_surface_flags.extend_from_slice(&t5_axial_plane_flags(axial));
        for (k, &p) in b.side_planes.iter().enumerate() {
            let Some(plane) = clip.planes.get(p as usize) else {
                continue;
            };
            planes.push([plane.normal[0], plane.normal[1], plane.normal[2], plane.dist]);
            let material = b.side_materials.get(k).map_or(-1, |&m| i64::from(m));
            plane_surface_flags.push(flags_of(material));
        }
        out.brushes.push(ClipBrush {
            planes,
            contents: b.contents as u32,
            plane_surface_flags,
            glass_encoded: 0,
        });
    }
    for m in &clip.materials {
        out.materials.push(ClipMapMaterial {
            name: m.name.clone(),
            surface_flags: m.surface_flags as u32,
            content_flags: m.content_flags as u32,
        });
    }
    let mesh = std::sync::Arc::make_mut(&mut out.mesh);
    mesh.verts = clip.verts.clone();
    mesh.tri_indices = clip.tri_indices.clone();
    mesh.tri_edge_is_walkable = Vec::new();
    let partitions: Vec<(u8, i32)> = clip.partitions.iter().map(|p| (p.tri_count, p.first_tri)).collect();
    mesh.partitions = partitions
        .iter()
        .map(|&(tri_count, first_tri)| clipmap_iw4::ClipPartition {
            tri_count,
            first_tri,
            first_vert_segment: 0,
            border_count: 0,
            first_border: 0,
        })
        .collect();
    mesh.aabb_trees = clip
        .aabb_trees
        .iter()
        .map(|t| clipmap_iw4::ClipAabbNode {
            origin: t.origin,
            half_size: t.half_size,
            material_index: t.material_index,
            child_count: t.child_count,
            u: t.index,
        })
        .collect();
    let tri_count = clip.tri_indices.len() / 3;
    let leaves: Vec<_> = mesh
        .aabb_trees
        .iter()
        .filter(|n| n.child_count == 0)
        .map(|n| (n.material_index, n.u))
        .collect();
    let sflags: Vec<_> = out.materials.iter().map(|m| m.surface_flags).collect();
    let cflags: Vec<_> = out.materials.iter().map(|m| m.content_flags).collect();
    mesh.tri_surface_flags = clipmap_iw4::flatten_tri_surface_flags(&leaves, &partitions, &sflags, tri_count);
    mesh.tri_content_flags = clipmap_iw4::flatten_tri_surface_flags(&leaves, &partitions, &cflags, tri_count);
    out.tri_material_index = clipmap_iw4::flatten_tri_material_index(&leaves, &partitions, tri_count);
    // The triangle trees' roots: every tree in the forest. The world model's
    // own range is empty (`coll_aabb_count` 0): its terrain and patches hang
    // off the collision BSP's leaves, which are not read, so the models'
    // ranges alone would leave the world's floors out of every trace.
    {
        let mesh = std::sync::Arc::make_mut(&mut out.mesh);
        let roots = legacy_aabb_roots(clip);
        mesh.aabb_roots = top_tree_over_roots(&mut mesh.aabb_trees, roots);
    }
    for (i, m) in clip.cmodels.iter().enumerate() {
        // The world model (0) takes its range below, once the brush models
        // have claimed theirs.
        let (first_brush, num_brushes) = if i == 0 {
            leafbrush_range(out.leafbrushes.len(), out.leafbrushes.len())?
        } else {
            append_tree_brushes(clip, m.leaf_brush_node, &mut out.leafbrushes)?
        };
        out.cmodels.push(ClipCmodel {
            mins: m.mins,
            maxs: m.maxs,
            radius: m.radius,
            first_brush,
            num_brushes,
        });
    }
    // The world's brushes: every brush no brush model owns. A brush model's
    // brushes are stored about the model's own origin (a door, a
    // script_brushmodel); traced where they are stored they stand as phantom
    // walls around the map's centre, so the world must not test them.
    let world = world_brush_ids(clip.brushes.len(), &out.leafbrushes);
    let first = out.leafbrushes.len();
    out.leafbrushes.extend_from_slice(&world);
    let (first_brush, num_brushes) = leafbrush_range(first, out.leafbrushes.len())?;
    if let Some(m) = out.cmodels.first_mut() {
        m.first_brush = first_brush;
        m.num_brushes = num_brushes;
    }
    // The collision BSP is not read; in its place a k-d tree over the world's
    // brushes (axial planes, each leaf listing the brushes whose bounds reach
    // into it), so a trace tests the brushes along its path, not all of them
    // (one leaf of every world brush made each trace on a CoD4 or World at
    // War map test thousands: bodies, bots and bullets all paid it). Its
    // triangles still come from the AABB forest, `aabb_roots`, as the leaves
    // list none.
    let bounds: Vec<([f32; 3], [f32; 3])> = world
        .iter()
        .map(|&b| brush_bounds(&out.brushes[usize::from(b)]))
        .collect();
    let (nodes, leaves) = build_brush_kd_tree(&world, &bounds, &mut out.leafbrushes)?;
    out.nodes = nodes;
    out.leaves = leaves;
    Ok(out)
}

/// Roots per group in the tree over the triangle forest.
const ROOT_GROUP: usize = 8;

/// bo2mp: a tree over the triangle forest's roots, so a trace opens the
/// boxes along its path, not every root (1,000 to 1,900 on a World at War
/// map, each tested by every trace). The roots are copied (children are
/// absolute indices, so a copy is the same tree) in an order that keeps
/// neighbours together, then grouped under boxes of eight, level by level,
/// to one top. Few roots, or a forest too large to index, stay as they are.
fn top_tree_over_roots(nodes: &mut Vec<clipmap_iw4::ClipAabbNode>, roots: Vec<u16>) -> Vec<u16> {
    if roots.len() <= ROOT_GROUP * 4 {
        return roots;
    }
    // Copies plus parents (about a seventh more) must stay indexable by u16.
    let extra = roots.len() + roots.len() / (ROOT_GROUP - 1) + ROOT_GROUP;
    if nodes.len() + extra >= usize::from(u16::MAX) {
        return roots;
    }
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for &r in &roots {
        let n = &nodes[usize::from(r)];
        for k in 0..3 {
            lo[k] = lo[k].min(n.origin[k]);
            hi[k] = hi[k].max(n.origin[k]);
        }
    }
    // Morton order of the roots' centres (ten bits an axis).
    let spread = |mut v: u32| {
        v &= 0x3ff;
        v = (v | (v << 16)) & 0x0300_00ff;
        v = (v | (v << 8)) & 0x0300_f00f;
        v = (v | (v << 4)) & 0x030c_30c3;
        (v | (v << 2)) & 0x0924_9249
    };
    let key = |n: &clipmap_iw4::ClipAabbNode| {
        let q = |k: usize| (((n.origin[k] - lo[k]) / (hi[k] - lo[k]).max(1.0)) * 1023.0) as u32;
        spread(q(0)) | (spread(q(1)) << 1) | (spread(q(2)) << 2)
    };
    let mut order = roots;
    order.sort_by_key(|&r| key(&nodes[usize::from(r)]));
    let mut level: std::ops::Range<usize> = nodes.len()..nodes.len() + order.len();
    for &r in &order {
        let copy = nodes[usize::from(r)];
        nodes.push(copy);
    }
    while level.len() > 1 {
        let start = nodes.len();
        let mut at = level.start;
        while at < level.end {
            let end = (at + ROOT_GROUP).min(level.end);
            let mut mn = [f32::MAX; 3];
            let mut mx = [f32::MIN; 3];
            for c in &nodes[at..end] {
                for k in 0..3 {
                    mn[k] = mn[k].min(c.origin[k] - c.half_size[k]);
                    mx[k] = mx[k].max(c.origin[k] + c.half_size[k]);
                }
            }
            nodes.push(clipmap_iw4::ClipAabbNode {
                origin: [(mn[0] + mx[0]) * 0.5, (mn[1] + mx[1]) * 0.5, (mn[2] + mx[2]) * 0.5],
                half_size: [(mx[0] - mn[0]) * 0.5, (mx[1] - mn[1]) * 0.5, (mx[2] - mn[2]) * 0.5],
                material_index: 0,
                child_count: (end - at) as u16,
                u: at as i32,
            });
            at = end;
        }
        level = start..nodes.len();
    }
    vec![level.start as u16]
}

/// A brush's bounds: its first six planes are its axial sides.
fn brush_bounds(b: &ClipBrush) -> ([f32; 3], [f32; 3]) {
    let p = &b.planes;
    if p.len() < 6 {
        return ([f32::MIN; 3], [f32::MAX; 3]);
    }
    ([-p[1][3], -p[3][3], -p[5][3]], [p[0][3], p[2][3], p[4][3]])
}

/// Brushes per leaf before a split is tried, and the deepest split.
const KD_LEAF_BRUSHES: usize = 12;
const KD_MAX_DEPTH: usize = 28;

/// A k-d tree over `ids` (their `bounds`, by position): nodes as the trace
/// walks them (child `!leaf` for a leaf, node 0 the root), each leaf's
/// brushes appended to `leafbrushes`.
fn build_brush_kd_tree(
    ids: &[u16],
    bounds: &[([f32; 3], [f32; 3])],
    leafbrushes: &mut Vec<u16>,
) -> Result<(Vec<ClipBspNode>, Vec<ClipBspLeaf>), ClipCollisionError> {
    let mut nodes: Vec<ClipBspNode> = Vec::new();
    let mut leaves: Vec<ClipBspLeaf> = Vec::new();
    let all: Vec<usize> = (0..ids.len()).collect();
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for (mn, mx) in bounds {
        for k in 0..3 {
            lo[k] = lo[k].min(mn[k]);
            hi[k] = hi[k].max(mx[k]);
        }
    }
    fn make_leaf(
        set: &[usize],
        ids: &[u16],
        leafbrushes: &mut Vec<u16>,
        leaves: &mut Vec<ClipBspLeaf>,
    ) -> Result<i32, ClipCollisionError> {
        let first = leafbrushes.len();
        leafbrushes.extend(set.iter().map(|&i| ids[i]));
        let (first_brush, num_brushes) = leafbrush_range(first, leafbrushes.len())?;
        leaves.push(ClipBspLeaf { first_brush, num_brushes, first_coll_aabb_index: 0, coll_aabb_count: 0 });
        Ok(!((leaves.len() - 1) as i32))
    }
    #[allow(clippy::too_many_arguments)]
    fn build(
        set: Vec<usize>,
        lo: [f32; 3],
        hi: [f32; 3],
        depth: usize,
        ids: &[u16],
        bounds: &[([f32; 3], [f32; 3])],
        nodes: &mut Vec<ClipBspNode>,
        leaves: &mut Vec<ClipBspLeaf>,
        leafbrushes: &mut Vec<u16>,
    ) -> Result<i32, ClipCollisionError> {
        if set.len() <= KD_LEAF_BRUSHES || depth >= KD_MAX_DEPTH {
            return make_leaf(&set, ids, leafbrushes, leaves);
        }
        // The region's longest side, split at the median of the brushes'
        // centres along it (kept inside the region).
        let axis = (0..3).max_by(|&a, &b| (hi[a] - lo[a]).total_cmp(&(hi[b] - lo[b]))).unwrap_or(0);
        let mut centres: Vec<f32> = set.iter().map(|&i| (bounds[i].0[axis] + bounds[i].1[axis]) * 0.5).collect();
        let mid = centres.len() / 2;
        centres.select_nth_unstable_by(mid, |a, b| a.total_cmp(b));
        let split = centres[mid].clamp(lo[axis], hi[axis]);
        let (mut front, mut back) = (Vec::new(), Vec::new());
        for &i in &set {
            if bounds[i].1[axis] >= split {
                front.push(i);
            }
            if bounds[i].0[axis] <= split {
                back.push(i);
            }
        }
        // A split that sends (nearly) everything both ways helps nothing.
        if front.len() + back.len() > set.len() * 3 / 2 || front.len() == set.len() || back.len() == set.len() {
            return make_leaf(&set, ids, leafbrushes, leaves);
        }
        let at = nodes.len();
        let mut normal = [0.0f32; 3];
        normal[axis] = 1.0;
        nodes.push(ClipBspNode { plane: [normal[0], normal[1], normal[2], split], children: [0, 0] });
        let mut front_lo = lo;
        front_lo[axis] = split;
        let mut back_hi = hi;
        back_hi[axis] = split;
        let f = build(front, front_lo, hi, depth + 1, ids, bounds, nodes, leaves, leafbrushes)?;
        let b = build(back, lo, back_hi, depth + 1, ids, bounds, nodes, leaves, leafbrushes)?;
        nodes[at].children = [f, b];
        Ok(at as i32)
    }
    let root = build(all, lo, hi, 0, ids, bounds, &mut nodes, &mut leaves, leafbrushes)?;
    if root < 0 {
        // One leaf: a node whose both sides are it (the walk starts at a node).
        nodes.push(ClipBspNode { plane: [0.0, 0.0, 1.0, -1.0e9], children: [root, root] });
    }
    Ok((nodes, leaves))
}

/// The brushes of `brush_count` that no brush model lists in
/// `model_brushes`, in order.
fn world_brush_ids(brush_count: usize, model_brushes: &[u16]) -> Vec<u16> {
    let mut owned = vec![false; brush_count];
    for &b in model_brushes {
        if let Some(o) = owned.get_mut(usize::from(b)) {
            *o = true;
        }
    }
    (0..brush_count).filter(|&b| !owned[b]).filter_map(|b| u16::try_from(b).ok()).collect()
}

/// Whether a decoded RGBA picture has no colour: every sampled pixel's red,
/// green and blue within 3 of each other (a concrete or a detail map; a
/// whole world of them reads as a greyscale decode).
pub fn is_greyscale_rgba(rgba: &[u8]) -> bool {
    let pixels = rgba.len() / 4;
    let step = (pixels / 4096).max(1);
    rgba.chunks_exact(4).step_by(step).all(|p| {
        let (lo, hi) = (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2]));
        hi - lo <= 3
    })
}

/// How a legacy map's spawn points stand on its converted collision, the way
/// Black Ops II's `placespawnpoint` finds their floor: a player box lifted up
/// to 128 units, then dropped straight down.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpawnGroundCensus {
    pub spawns: usize,
    /// A floor within [`SPAWN_FLOOR_REACH`] below the point, the box free there.
    pub on_ground: usize,
    /// The floor found is further down than that: the point's own floor is
    /// missing from the collision (the player would fall through it).
    pub dropped_far: usize,
    /// Nothing below at all.
    pub no_floor: usize,
    /// The player box is stuck at its floor (it could not move there).
    pub in_solid: usize,
}

/// How far below a spawn point its floor may be and still count as its own.
pub const SPAWN_FLOOR_REACH: f32 = 64.0;

impl std::fmt::Display for SpawnGroundCensus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} spawns: {} on the ground, {} fall more than {SPAWN_FLOOR_REACH} units, {} over nothing, {} in solid",
            self.spawns, self.on_ground, self.dropped_far, self.no_floor, self.in_solid
        )
    }
}

/// A player box trace through `clip` as the match's sim does it (its BSP's
/// brushes, then the collision triangles): the fraction of the way it got,
/// and whether it started in solid.
pub fn trace_clip_collision(
    clip: &ClipCollision,
    start: [f32; 3],
    end: [f32; 3],
    mins: [f32; 3],
    maxs: [f32; 3],
    mask: u32,
) -> (f32, bool) {
    let map = clipmap_iw4::ClipMapRef {
        nodes: &clip.nodes,
        leaves: &clip.leaves,
        leafbrushes: &clip.leafbrushes,
        brushes: &clip.brushes,
    };
    let ext = clipmap_iw4::TraceExtents::new(start, end, mins, maxs, mask);
    let mesh = clipmap_iw4::ClipMeshTables::as_ref(&clip.mesh);
    let best = clipmap_iw4::trace_brush_and_mesh(&map, &mesh, &ext, &|_| true);
    (best.fraction, best.startsolid != 0)
}

/// Each spawn point dropped onto `clip` with BO2's player box.
pub fn spawn_ground_census(clip: &ClipCollision, spawns: &[[f32; 3]]) -> SpawnGroundCensus {
    const MINS: [f32; 3] = [-15.0, -15.0, 0.0];
    const MAXS: [f32; 3] = [15.0, 15.0, 70.0];
    const MASK: u32 = crate::clip_collision::MASK_PLAYER_SOLID;
    let mut out = SpawnGroundCensus { spawns: spawns.len(), ..Default::default() };
    for &o in spawns {
        let (up, _) = trace_clip_collision(clip, o, [o[0], o[1], o[2] + 128.0], MINS, MAXS, MASK);
        let top = [o[0], o[1], o[2] + 128.0 * up];
        let drop = 128.0 * up + 8192.0;
        let (down, _) = trace_clip_collision(clip, top, [o[0], o[1], top[2] - drop], MINS, MAXS, MASK);
        if down >= 1.0 {
            out.no_floor += 1;
            continue;
        }
        let floor = [o[0], o[1], top[2] - drop * down];
        if trace_clip_collision(clip, floor, floor, MINS, MAXS, MASK).1 {
            out.in_solid += 1;
        } else if o[2] - floor[2] > SPAWN_FLOOR_REACH {
            out.dropped_far += 1;
        } else {
            out.on_ground += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use iw3::zone::{Brush, CModel, ClipMaterial, CollisionAabbTree, CollisionPartition, LeafBrushNode};

    fn boxed(mins: [f32; 3], maxs: [f32; 3]) -> Brush {
        Brush { mins, maxs, contents: 1, side_planes: Vec::new(), side_materials: Vec::new(), axial_materials: [[0; 3]; 2] }
    }

    fn cmodel(leaf_brush_node: i32, first_coll_aabb: u16, coll_aabb_count: u16) -> CModel {
        CModel { mins: [-1.0; 3], maxs: [1.0; 3], radius: 1.0, leaf_brush_node, first_coll_aabb, coll_aabb_count }
    }

    /// A CoD4-shaped clipMap: a world floor brush at z 0, a brush model's
    /// brush stored about its own origin (a box over the map's centre), and
    /// a terrain square at z 100 over (-400..-200) whose AABB tree no model
    /// names (the world model's range is empty; its terrain hangs off the
    /// BSP leaves).
    fn clip_map() -> ClipMap {
        ClipMap {
            name: "test".into(),
            planes: Vec::new(),
            static_models: Vec::new(),
            materials: vec![ClipMaterial { name: "solid".into(), surface_flags: 0, content_flags: 1 }],
            brushes: vec![
                boxed([-512.0, -512.0, -16.0], [512.0, 512.0, 0.0]),
                boxed([150.0, 150.0, -10.0], [250.0, 250.0, 100.0]),
            ],
            verts: vec![[-400.0, -400.0, 100.0], [-200.0, -400.0, 100.0], [-200.0, -200.0, 100.0], [-400.0, -200.0, 100.0]],
            // Wound as the game winds a floor (facing up).
            tri_indices: vec![0, 2, 1, 0, 3, 2],
            partitions: vec![CollisionPartition { tri_count: 2, first_tri: 0 }],
            aabb_trees: vec![CollisionAabbTree {
                origin: [-300.0, -300.0, 100.0],
                half_size: [100.0, 100.0, 1.0],
                material_index: 0,
                child_count: 0,
                index: 0,
            }],
            cmodels: vec![cmodel(0, 0, 0), cmodel(0, 1, 0)],
            map_ents: None,
            dyn_ent_counts: [0; 2],
            dyn_ents: Vec::new(),
            leaf_brush_nodes: vec![LeafBrushNode { leaf_brush_count: 1, brushes: vec![1], child_offsets: [0; 2] }],
        }
    }

    #[test]
    fn greyscale_pictures() {
        assert!(is_greyscale_rgba(&[10, 11, 12, 255, 200, 200, 203, 0]));
        assert!(!is_greyscale_rgba(&[10, 11, 12, 255, 180, 72, 98, 255]));
        assert!(is_greyscale_rgba(&[]));
    }

    #[test]
    fn world_brushes_are_those_no_brush_model_owns() {
        assert_eq!(world_brush_ids(5, &[1, 3, 3, 9]), vec![0, 2, 4]);
        assert_eq!(world_brush_ids(0, &[0]), Vec::<u16>::new());
    }

    #[test]
    fn world_triangles_are_every_tree_but_the_brush_models() {
        let mut clip = clip_map();
        assert_eq!(legacy_aabb_roots(&clip), vec![0]);
        // A brush model owning that tree takes it out of the world.
        clip.cmodels[1].coll_aabb_count = 0;
        clip.cmodels[1].first_coll_aabb = 0;
        clip.cmodels.push(cmodel(0, 0, 1));
        assert!(legacy_aabb_roots(&clip).is_empty());
    }

    #[test]
    fn world_leaf_holds_the_world_brushes_only() {
        let built = build_iw3_clip_collision(&clip_map()).unwrap();
        assert_eq!(built.nodes.len(), 1);
        let leaf = built.leaves[0];
        let world = &built.leafbrushes[leaf.first_brush as usize..][..usize::from(leaf.num_brushes)];
        assert_eq!(world, &[0]);
        let m0 = built.cmodels[0];
        assert_eq!(&built.leafbrushes[m0.first_brush as usize..][..usize::from(m0.num_brushes)], world);
        let model = built.cmodels[1];
        assert_eq!(&built.leafbrushes[model.first_brush as usize..][..usize::from(model.num_brushes)], &[1]);
        assert_eq!(built.mesh.aabb_roots, vec![0]);
    }

    #[test]
    fn the_brush_tree_finds_what_one_leaf_of_every_brush_found() {
        // A grid of 400 small boxes; traces through, along and into them hit
        // the same as testing every box (one leaf of them all).
        let mut ids = Vec::new();
        let mut bounds = Vec::new();
        for i in 0..400u16 {
            let (x, y) = (f32::from(i % 20) * 64.0, f32::from(i / 20) * 64.0);
            ids.push(i);
            bounds.push(([x, y, 0.0], [x + 32.0, y + 32.0, 32.0]));
        }
        let brushes: Vec<ClipBrush> = bounds
            .iter()
            .map(|(mn, mx)| ClipBrush {
                planes: vec![
                    [1.0, 0.0, 0.0, mx[0]],
                    [-1.0, 0.0, 0.0, -mn[0]],
                    [0.0, 1.0, 0.0, mx[1]],
                    [0.0, -1.0, 0.0, -mn[1]],
                    [0.0, 0.0, 1.0, mx[2]],
                    [0.0, 0.0, -1.0, -mn[2]],
                ],
                contents: 1,
                plane_surface_flags: vec![0; 6],
                glass_encoded: 0,
            })
            .collect();
        let mut leafbrushes = Vec::new();
        let (nodes, leaves) = build_brush_kd_tree(&ids, &bounds, &mut leafbrushes).unwrap();
        assert!(leaves.len() > 20, "{} leaves", leaves.len());
        let mut flat_lb = ids.clone();
        flat_lb.extend_from_slice(&leafbrushes);
        let tree = ClipCollision { brushes: brushes.clone(), nodes, leaves, leafbrushes, ..ClipCollision::default() };
        let one = ClipCollision {
            brushes,
            nodes: vec![ClipBspNode { plane: [0.0, 0.0, 1.0, -1.0e9], children: [-1, -1] }],
            leaves: vec![ClipBspLeaf { first_brush: 0, num_brushes: 400, first_coll_aabb_index: 0, coll_aabb_count: 0 }],
            leafbrushes: ids,
            ..ClipCollision::default()
        };
        let mut n = 0;
        for k in 0..60 {
            let t = k as f32;
            let start = [-50.0 + t * 7.0, -40.0 + t * 13.0, 16.0 + (t * 0.7).sin() * 30.0];
            let end = [1300.0 - t * 11.0, 1250.0 - t * 5.0, 10.0];
            for half in [0.0, 4.0, 15.0] {
                let a = tree.sweep_box(start, end, [-half; 3], [half; 3], 1);
                let b = one.sweep_box(start, end, [-half; 3], [half; 3], 1);
                assert!((a.fraction - b.fraction).abs() < 1e-5, "{start:?} {half}: {} vs {}", a.fraction, b.fraction);
                assert_eq!(a.startsolid, b.startsolid);
                n += usize::from(b.fraction < 1.0);
            }
        }
        assert!(n > 100, "only {n} hits");
    }

    #[test]
    fn spawns_stand_on_brush_floors_and_terrain() {
        let built = build_iw3_clip_collision(&clip_map()).unwrap();
        // On the floor brush, inside where the brush model's box would be
        // if it were traced where it is stored; and on the terrain square,
        // 100 units over the floor brush.
        let census = spawn_ground_census(&built, &[[200.0, 200.0, 1.0], [-300.0, -300.0, 108.0], [-300.0, 300.0, 200.0]]);
        assert_eq!(census, SpawnGroundCensus { spawns: 3, on_ground: 2, dropped_far: 1, no_floor: 0, in_solid: 0 });
        assert_eq!(
            census.to_string(),
            "3 spawns: 2 on the ground, 1 fall more than 64 units, 0 over nothing, 0 in solid"
        );
    }
}

#[cfg(test)]
mod iw3_lightmap_tests {
    use super::*;

    #[test]
    fn a_page_pair_becomes_three_stacked_pages() {
        // Secondary 2x4 (two 2x2 pages, BGRA bytes), primary 4x4 (L8).
        let mut sec = Vec::new();
        for _ in 0..4 {
            sec.extend_from_slice(&[10, 20, 30, 40]); // A: B=10 G=20 R=30 A=40
        }
        for _ in 0..4 {
            sec.extend_from_slice(&[50, 60, 70, 80]); // B
        }
        let prim: Vec<u8> = (0..16).map(|i| i as u8 * 10).collect();
        let lm = iw3_lightmap((4, 4, 50, &prim), (2, 4, 21, &sec), ("p", "s")).expect("decodes");
        let img = lm.secondary_image.expect("page");
        assert_eq!((img.width(), img.height()), (4, 12));
        let d = img.data.expect("pixels");
        assert_eq!(&d[0..4], &[30, 20, 10, 40]);
        let p1 = 4 * 4 * 4;
        assert_eq!(&d[p1..p1 + 4], &[70, 60, 50, 80]);
        let p2 = 2 * 4 * 4 * 4;
        assert_eq!(d[p2 + 3], 0);
        assert_eq!(d[p2 + 4 * 5 + 3], 50);
        assert!(iw3_lightmap((4, 4, 21, &prim), (2, 4, 21, &sec), ("p", "s")).is_err());
    }
}
