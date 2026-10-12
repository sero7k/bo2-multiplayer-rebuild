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
//! BSP is not read yet: traces test every brush (the trace's own fallback).

use asset_iw4::size as iw4_sz;
use dpvs_iw4::{AabbNodeView, Bounds, SurfRange};
use iw3::zone::{ClipMap, GfxWorld};

use crate::clip_collision::{
    ClipBrush, ClipCmodel, ClipCollision, ClipCollisionError, ClipMapMaterial,
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
        lightmap: if lightmaps.iter().any(Option::is_some) {
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
        t5_tree_scatter_intensity: None,
        t5_tree_scatter_amount: None,
        t5_exposure_volume_count: 0,
        t6_lights: Vec::new(),
        t6_fog_banks: Vec::new(),
        t6_probes: Vec::new(),
        t6_layer_uvs: layer_uvs,
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
    // The triangle trees' roots: every model's own.
    let mut roots = std::collections::BTreeSet::new();
    for m in &clip.cmodels {
        for id in u32::from(m.first_coll_aabb)..u32::from(m.first_coll_aabb) + u32::from(m.coll_aabb_count) {
            if let Ok(id) = u16::try_from(id) {
                roots.insert(id);
            }
        }
    }
    std::sync::Arc::make_mut(&mut out.mesh).aabb_roots = roots.into_iter().collect();
    for (i, m) in clip.cmodels.iter().enumerate() {
        // The world model (0) has no brush range of its own here: traces
        // test every brush.
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
    Ok(out)
}
