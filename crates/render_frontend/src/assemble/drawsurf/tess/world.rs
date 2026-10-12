use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::render_resource::Face;

use crate::assemble::drawsurf::{
    SurfaceLightmapId, SurfaceReflectionProbeId, SurfaceSamplerInputs,
};
use crate::prepare::scene::world::{WorldScene, WorldSun};

pub use render_frame::WorldVertex;

#[derive(Clone, Debug)]
pub struct WorldPassMaterial {
    pub color: Option<Handle<Image>>,
    /// bo2zm: the colour map's slot in the uploaded material images, for the
    /// fallback draw of materials with no runnable technique (Black Ops II).
    pub color_slot: Option<u32>,
    /// bo2zm: a Black Ops II material's fallback draw.
    pub t6_draw: Option<asset_core::T6Draw>,
    /// bo2zm: a layered Black Ops II material's layer 1 and 2 colour map
    /// slots in the uploaded material images.
    pub t6_layer_slots: [Option<u32>; 2],
    /// bo2zm: a Black Ops II material's normal and specular map slots in
    /// the uploaded material images (its shine).
    pub t6_shine_slots: [Option<u32>; 2],
    pub normal: Option<Handle<Image>>,
    pub specular: Option<Handle<Image>>,
    pub probe: Option<Handle<Image>>,
    pub ambient: Option<Handle<Image>>,
    pub directional: Option<Handle<Image>>,
    pub sun_mask: Option<Handle<Image>>,
    pub sun: Option<WorldSun>,
    pub alpha_mode: AlphaMode,

    pub draw_mode: Option<asset_material::MaterialDrawMode>,
    pub square_color_map: bool,
    pub env_map_parms: [f32; 4],
    pub uv_anim: [f32; 4],
    pub falloff_parms: [f32; 4],
    pub falloff_begin: [f32; 4],
    pub falloff_end: [f32; 4],
    pub cull_mode: Option<Face>,
    /// bo2mp (MW3 maps: decals drawn as opaque quads with black edges,
    /// floating cards): an IW4/IW5 material the renderer's fallback pass
    /// must leave out rather than draw solid (it blends, refracts, is a
    /// particle cloud or casts shadow only). Only read for a material with
    /// no `t6_draw`; its own technique, when it runs, still draws it.
    pub fallback_no_draw: Option<asset_material::IwNoOpaqueDraw>,
}

/// bo2mp: the draw code that makes the fallback pass leave a world surface
/// out: `T6Blend::ShadowOnly` in `T6Draw::code`'s blend nibble (the pass
/// skips every code whose blend nibble is 4 or more).
pub const FALLBACK_NO_DRAW_CODE: u32 = 4;

/// bo2mp: an IW4/IW5 runtime material the fallback pass cannot draw right.
pub fn iw_fallback_no_draw(
    material: &render_material::RuntimeMaterial,
) -> Option<asset_material::IwNoOpaqueDraw> {
    if material.t6_draw.is_some()
        || !matches!(
            material.namespace,
            asset_material::AssetNamespace::Iw4 | asset_material::AssetNamespace::Iw5
        )
    {
        return None;
    }
    asset_material::iw_no_opaque_draw(
        &material.name,
        &material.technique_set,
        material.state_bits_entry.as_ref().map(|e| e.as_slice()),
        &material.state_bits_table,
    )
}

/// bo2mp: a world surface's fallback draw code: its BO2 draw, else
/// `FALLBACK_NO_DRAW_CODE` for an IW material the fallback cannot draw
/// right, else none (the plain fallback).
pub fn fallback_draw_code(material: Option<&WorldPassMaterial>) -> u32 {
    match material {
        Some(WorldPassMaterial {
            t6_draw: Some(draw),
            ..
        }) => draw.code(),
        Some(WorldPassMaterial {
            fallback_no_draw: Some(_),
            ..
        }) => FALLBACK_NO_DRAW_CODE,
        _ => asset_core::T6Draw::NONE_CODE,
    }
}

pub use render_frame::WorldVertexRefusal;

#[derive(Resource, Clone, Debug, Default)]
pub struct WorldDrawGpuPlan {
    pub packed_vertices: asset_world::WorldVertexPayload,

    pub surface_material: Vec<u32>,

    pub surface_sampler_inputs: Vec<SurfaceSamplerInputs>,
    /// bo2zm: per surface, its primary light index (the world's
    /// `surface_primary_lights`), for the fallback draw.
    pub surface_primary_light: Vec<u8>,
    pub materials: Vec<WorldPassMaterial>,

    pub upload_pending: bool,
    pub vertex_share: Option<Arc<Vec<[u8; asset_iw4::size::GFX_WORLD_VERTEX]>>>,
    pub index_share: Option<Arc<Vec<u32>>>,
    pub range_share: Option<Arc<Vec<(u32, u32)>>>,
    pub layer_share: Option<Arc<Vec<u8>>>,
    pub decoded_share: Option<Arc<Vec<WorldVertex>>>,
    /// bo2zm: per decoded vertex, a layered Black Ops II surface's layer 1
    /// and 2 texcoords (empty when the world has none).
    pub t6_layer_share: Arc<Vec<[f32; 4]>>,
}

/// bo2zm: the uploaded image slot of a Black Ops II world's reflection
/// probes (one cube array, linked under a private semantic).
pub fn t6_probe_slot(catalog: &crate::assemble::drawsurf::RuntimeMaterialCatalog) -> Option<u32> {
    catalog
        .materials
        .iter()
        .find_map(|m| m.texture_semantic(0xf3))
        .map(|id| id.0)
}

/// bo2zm: the uploaded image slot of a Black Ops II sky's cube colour map
/// (the material whose draw is `T6Blend::Sky`).
pub fn t6_sky_slot(catalog: &crate::assemble::drawsurf::RuntimeMaterialCatalog) -> Option<u32> {
    catalog
        .materials
        .iter()
        .find(|m| m.t6_draw.is_some_and(|d| d.blend == asset_core::T6Blend::Sky))
        .and_then(|m| m.texture_semantic(asset_material::TS_COLOR_MAP))
        .map(|id| id.0)
}

impl WorldDrawGpuPlan {
    /// bo2zm: per world surface, its colour map's uploaded image slot
    /// (`u32::MAX` = none), its Black Ops II draw code (`T6Draw::NONE_CODE`
    /// = none) and its lightmap page (`u8::MAX` = none), for the renderer's
    /// fallback pass. Primary lights: `surface_primary_light`.
    pub fn fallback_surface_styles(&self) -> (Vec<u32>, Vec<u32>, Vec<u8>) {
        let mut colors = Vec::with_capacity(self.surface_material.len());
        let mut draws = Vec::with_capacity(self.surface_material.len());
        let mut lightmaps = Vec::with_capacity(self.surface_material.len());
        for (i, &m) in self.surface_material.iter().enumerate() {
            let material = self.materials.get(m as usize);
            colors.push(material.and_then(|mat| mat.color_slot).unwrap_or(u32::MAX));
            draws.push(fallback_draw_code(material));
            lightmaps.push(
                self.surface_sampler_inputs
                    .get(i)
                    .and_then(|inputs| inputs.secondary_lightmap)
                    .map_or(u8::MAX, |id| id.0),
            );
        }
        (colors, draws, lightmaps)
    }

    /// bo2zm: per world surface, its normal and specular map slots
    /// (`u32::MAX` = none) and its reflection probe, for the fallback draw's
    /// shine.
    pub fn fallback_surface_shine(&self) -> Vec<[u32; 3]> {
        self.surface_material
            .iter()
            .enumerate()
            .map(|(i, &m)| {
                let [n, s] = self
                    .materials
                    .get(m as usize)
                    .map_or([None; 2], |mat| mat.t6_shine_slots);
                let probe = self
                    .surface_sampler_inputs
                    .get(i)
                    .and_then(|inputs| inputs.reflection_probe)
                    .map_or(0, |id| u32::from(id.0));
                [n.unwrap_or(u32::MAX), s.unwrap_or(u32::MAX), probe]
            })
            .collect()
    }

    /// bo2zm: per world surface, its layer 1 and 2 colour map slots
    /// (`u32::MAX` = none), for the fallback draw of layered materials.
    pub fn fallback_surface_layers(&self) -> Vec<[u32; 2]> {
        self.surface_material
            .iter()
            .map(|&m| {
                self.materials.get(m as usize).map_or([u32::MAX; 2], |mat| {
                    mat.t6_layer_slots.map(|s| s.unwrap_or(u32::MAX))
                })
            })
            .collect()
    }

    pub fn exact_packed_vertices(
        &self,
    ) -> Result<&[[u8; asset_iw4::size::GFX_WORLD_VERTEX]], WorldVertexRefusal> {
        let table_stride =
            asset_iw4::vertex_decl::stream_extent(asset_iw4::vertex_decl::WORLD_VERTEX_TYPE, 0);
        if table_stride != Some(asset_iw4::size::GFX_WORLD_VERTEX as u16) {
            return Err(WorldVertexRefusal::StrideMismatch { table_stride });
        }
        let vertices = if let Some(share) = self.vertex_share.as_ref() {
            share.as_slice()
        } else {
            match self.packed_vertices.type2_stream0() {
                Ok(vertices) => vertices,
                Err(source_layout) => {
                    return Err(WorldVertexRefusal::ForeignLayout { source_layout });
                }
            }
        };
        if vertices.len() != self.decoded_vertices().len() {
            return Err(WorldVertexRefusal::VertexCountMismatch {
                packed: vertices.len(),
                decoded: self.decoded_vertices().len(),
            });
        }
        Ok(vertices)
    }

    pub fn indices(&self) -> &[u32] {
        super::published_rows(&self.index_share)
    }

    pub fn surface_ranges(&self) -> &[(u32, u32)] {
        super::published_rows(&self.range_share)
    }

    pub fn decoded_vertices(&self) -> &[WorldVertex] {
        super::published_rows(&self.decoded_share)
    }

    pub fn vertex_layer_rows(&self) -> &[u8] {
        super::published_rows(&self.layer_share)
    }

    pub fn publish_extract_shares(&mut self) {
        if self.vertex_share.is_none() {
            match std::mem::replace(
                &mut self.packed_vertices,
                asset_world::WorldVertexPayload::Unavailable {
                    source_layout: "packed payload published for extract",
                },
            ) {
                asset_world::WorldVertexPayload::Iw4(rows)
                | asset_world::WorldVertexPayload::Iw5(rows)
                | asset_world::WorldVertexPayload::T5(rows)
                | asset_world::WorldVertexPayload::T6(rows) => {
                    self.vertex_share = Some(Arc::new(rows));
                }
                unavailable => self.packed_vertices = unavailable,
            }
        }
    }

    pub fn from_scene(
        scene: &mut WorldScene,
        exact_handles: &[Option<Handle<Image>>],
        lightmap_handles: &[Option<crate::assemble::drawsurf::RuntimeLightmapHandles>],
        reflection_probe_handles: &[Option<Handle<Image>>],
    ) -> Self {
        let packed_vertices = std::mem::replace(
            &mut scene.retained_packed_vertices,
            asset_world::WorldVertexPayload::Unavailable {
                source_layout: "packed payload consumed by WorldDrawGpuPlan",
            },
        );
        let vertex_layer = scene.retained_vertex_layer.clone();
        let n = scene.retained_positions.len();
        let mut vertices = Vec::with_capacity(n);
        for i in 0..n {
            vertices.push(WorldVertex {
                position: scene.retained_positions[i],
                normal: scene
                    .retained_normals
                    .get(i)
                    .copied()
                    .unwrap_or([0.0, 0.0, 1.0]),
                tangent: scene
                    .retained_tangents
                    .get(i)
                    .copied()
                    .unwrap_or([1.0, 0.0, 0.0, 1.0]),
                color: scene.retained_colors.get(i).copied().unwrap_or([1.0; 4]),
                uv0: scene
                    .retained_texture_uvs
                    .get(i)
                    .copied()
                    .unwrap_or([0.0; 2]),
                uv1: scene
                    .retained_lightmap_uvs
                    .get(i)
                    .copied()
                    .unwrap_or([0.0; 2]),
            });
        }

        scene.retained_tangents.clear();
        scene.retained_colors.clear();
        scene.retained_texture_uvs.clear();

        let mut materials = Vec::new();
        let mut fallback_names: Vec<String> = Vec::new();
        let mut batch_to_mat = vec![u32::MAX; scene.batches.len()];

        for (batch_i, batch) in scene.batches.iter().enumerate() {
            let runtime = batch
                .material
                .and_then(|id| scene.runtime_material_catalog.derived(id));
            let color = runtime.and_then(|m| {
                m.texture_semantic(asset_material::TS_COLOR_MAP)
                    .and_then(|id| exact_handles.get(id.0 as usize).and_then(Clone::clone))
            });
            let normal = runtime.and_then(|m| {
                m.texture_semantic(asset_material::TS_NORMAL_MAP)
                    .and_then(|id| exact_handles.get(id.0 as usize).and_then(Clone::clone))
            });
            let specular = runtime.and_then(|m| {
                m.texture_semantic(asset_material::TS_SPECULAR_MAP)
                    .and_then(|id| exact_handles.get(id.0 as usize).and_then(Clone::clone))
            });
            let probe = reflection_probe_handles
                .get(usize::from(batch.reflection_probe_index))
                .and_then(Clone::clone);

            let (ambient, directional, sun_mask) = if batch.lightmapped {
                lightmap_handles
                    .get(usize::from(batch.lightmap_index))
                    .and_then(|page| page.as_ref())
                    .map(|page| {
                        (
                            Some(page.ambient_diagnostic.clone()),
                            Some(page.directional_diagnostic.clone()),
                            Some(page.sun_mask_diagnostic.clone()),
                        )
                    })
                    .unwrap_or((None, None, None))
            } else {
                (None, None, None)
            };

            let color_slot = runtime
                .and_then(|m| m.texture_semantic(asset_material::TS_COLOR_MAP))
                .map(|id| id.0);
            let mat = WorldPassMaterial {
                color,
                color_slot,
                t6_draw: runtime.and_then(|m| m.t6_draw),
                t6_layer_slots: [0xf1u8, 0xf2].map(|semantic| {
                    runtime
                        .and_then(|m| m.texture_semantic(semantic))
                        .map(|id| id.0)
                }),
                t6_shine_slots: [asset_material::TS_NORMAL_MAP, asset_material::TS_SPECULAR_MAP]
                    .map(|semantic| {
                        runtime
                            .filter(|m| m.t6_draw.is_some())
                            .and_then(|m| m.texture_semantic(semantic))
                            .map(|id| id.0)
                    }),
                normal,
                specular,
                probe,
                ambient,
                directional,
                sun_mask,
                sun: batch.sun,
                alpha_mode: AlphaMode::Opaque,
                draw_mode: None,
                square_color_map: runtime.is_some_and(|m| m.square_color_map),
                env_map_parms: runtime.map(|m| m.env_map_parms()).unwrap_or([0.0; 4]),
                uv_anim: runtime.map(|m| m.uv_anim()).unwrap_or([0.0; 4]),
                falloff_parms: runtime.map(|m| m.falloff_parms()).unwrap_or([0.0; 4]),
                falloff_begin: runtime.map(|m| m.falloff_begin()).unwrap_or([0.0; 4]),
                falloff_end: runtime.map(|m| m.falloff_end()).unwrap_or([0.0; 4]),
                cull_mode: runtime.and_then(|m| super::super::runtime_cull_face(m.cull_mode)),
                fallback_no_draw: runtime.and_then(iw_fallback_no_draw),
            };
            let idx = materials.len() as u32;
            fallback_names.push(match (mat.fallback_no_draw, runtime) {
                (Some(_), Some(m)) => m.name.clone(),
                _ => String::new(),
            });
            materials.push(mat);
            if let Some(slot) = batch_to_mat.get_mut(batch_i) {
                *slot = idx;
            }
        }

        let Some(cull) = scene.cull.as_ref() else {
            let upload_pending = !vertices.is_empty();
            let mut plan = Self {
                packed_vertices,
                surface_material: Vec::new(),
                surface_sampler_inputs: Vec::new(),
                surface_primary_light: Vec::new(),
                materials,
                upload_pending,
                vertex_share: None,
                index_share: Some(Arc::new(Vec::new())),
                range_share: Some(Arc::new(Vec::new())),
                layer_share: Some(Arc::new(vertex_layer)),
                decoded_share: Some(Arc::new(vertices)),
                t6_layer_share: Arc::new(scene.retained_t6_layer_uvs.clone()),
            };
            plan.publish_extract_shares();
            return plan;
        };

        let indices = cull.packed_indices.clone();
        let surface_ranges = cull.surface_index_ranges.clone();

        let surface_material: Vec<u32> = cull
            .surface_batch_ranges
            .iter()
            .zip(surface_ranges.iter())
            .map(|(&(batch, _, _), &(_, count))| {
                if count == 0 {
                    return u32::MAX;
                }
                batch_to_mat.get(batch).copied().unwrap_or(u32::MAX)
            })
            .collect();
        log_fallback_no_draw_census(&materials, &fallback_names, &surface_material);
        let surface_sampler_inputs = cull
            .surface_batch_ranges
            .iter()
            .map(|&(batch, _, _)| {
                let Some(batch) = scene.batches.get(batch) else {
                    return SurfaceSamplerInputs::default();
                };
                SurfaceSamplerInputs {
                    reflection_probe: Some(SurfaceReflectionProbeId(batch.reflection_probe_index)),
                    primary_lightmap: batch
                        .lightmapped
                        .then_some(SurfaceLightmapId(batch.lightmap_index)),
                    secondary_lightmap: batch
                        .lightmapped
                        .then_some(SurfaceLightmapId(batch.lightmap_index)),
                }
            })
            .collect();

        let upload_pending = !vertices.is_empty() && !indices.is_empty();
        let mut plan = Self {
            packed_vertices,
            surface_material,
            surface_sampler_inputs,
            surface_primary_light: cull.surface_primary_lights.clone(),
            materials,
            upload_pending,
            vertex_share: None,
            index_share: Some(Arc::new(indices)),
            range_share: Some(Arc::new(surface_ranges)),
            layer_share: Some(Arc::new(vertex_layer)),
            decoded_share: Some(Arc::new(vertices)),
            t6_layer_share: Arc::new(scene.retained_t6_layer_uvs.clone()),
        };
        plan.publish_extract_shares();
        plan
    }
}

/// bo2mp: once per world plan, the world surfaces the fallback pass leaves
/// out (IW materials it cannot draw right) by class, and their materials.
fn log_fallback_no_draw_census(
    materials: &[WorldPassMaterial],
    material_names: &[String],
    surface_material: &[u32],
) {
    let mut by_class: std::collections::BTreeMap<&'static str, usize> = Default::default();
    let mut by_material: std::collections::BTreeMap<u32, usize> = Default::default();
    for &m in surface_material {
        let Some(class) = materials.get(m as usize).and_then(|mat| {
            mat.t6_draw.is_none().then_some(mat.fallback_no_draw).flatten()
        }) else {
            continue;
        };
        *by_class.entry(class.as_str()).or_default() += 1;
        *by_material.entry(m).or_default() += 1;
    }
    if by_class.is_empty() {
        return;
    }
    let mut top: Vec<(u32, usize)> = by_material.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));
    let names: Vec<String> = top
        .iter()
        .take(8)
        .map(|&(m, n)| {
            let name = material_names.get(m as usize).map_or("?", String::as_str);
            format!("{name}x{n}")
        })
        .collect();
    let total: usize = by_class.values().sum();
    diag::info!(
        World,
        "world fallback: {total} surfaces left out of the fallback draw (IW materials it cannot draw solid) {by_class:?}; plan materials {names:?}"
    );
}

pub(crate) fn build_world_draw_gpu_plan(
    mut scene: Option<ResMut<WorldScene>>,
    job: Res<crate::prepare::scene::spawn::WorldSpawnJob>,
    existing: Option<Res<WorldDrawGpuPlan>>,
    mut commands: Commands,
) {
    if job.phase != crate::prepare::scene::spawn::WorldSpawnPhase::WorldTess {
        return;
    }
    if existing.is_some() {
        return;
    }
    let Some(scene) = scene.as_mut() else {
        return;
    };
    let (exact, lightmaps, probes) = job.tess_image_handles();
    let plan = WorldDrawGpuPlan::from_scene(scene, exact, lightmaps, probes);
    for batch in &scene.batches {
        let runtime = batch
            .material
            .and_then(|material| scene.runtime_material_catalog.derived(material));
        if runtime.is_some_and(|m| m.unlit)
            && runtime.is_some_and(|m| m.uv_anim() != [0.0; 4] || m.falloff_begin()[3] >= 0.5)
        {
            diag::info!(
                World,
                "world unlit anim: {} uvAnim={:?} falloffParms={:?} falloffBegin={:?} falloffEnd={:?}",
                runtime.map(|m| m.name.as_str()).unwrap_or("<none>"),
                runtime.map(|m| m.uv_anim()),
                runtime.map(|m| m.falloff_parms()),
                runtime.map(|m| m.falloff_begin()),
                runtime.map(|m| m.falloff_end()),
            );
        }
    }
    let exact_vertex_status = match plan.exact_packed_vertices() {
        Ok(vertices) => format!("READY records={} stride=0x2c", vertices.len()),
        Err(refusal) => format!("REFUSED({refusal:?})"),
    };
    let probe_surfaces = plan
        .surface_sampler_inputs
        .iter()
        .filter(|inputs| inputs.reflection_probe.is_some())
        .count();
    let lightmap_surfaces = plan
        .surface_sampler_inputs
        .iter()
        .filter(|inputs| inputs.secondary_lightmap.is_some())
        .count();
    diag::info!(
        World,
        "world drawsurf tess plan: batches={} verts={} indices={} materials={} retail_vb={} surface_inputs={}/{} probe/lightmap (material execution deferred to Colour product); mark_clip_verts pos={} lmap={} nrm={}",
        scene.batches.len(),
        plan.decoded_vertices().len(),
        plan.indices().len(),
        plan.materials.len(),
        exact_vertex_status,
        probe_surfaces,
        lightmap_surfaces,
        scene.retained_positions.len(),
        scene.retained_lightmap_uvs.len(),
        scene.retained_normals.len(),
    );
    commands.insert_resource(plan);
}

#[cfg(test)]
mod fallback_no_draw_tests {
    use super::*;

    #[test]
    fn no_draw_code_is_the_shadow_only_code_the_fallback_skips() {
        let shadow = asset_core::T6Draw::from_technique_set("wpc_shadowcaster", 0, None);
        assert_eq!(shadow.blend, asset_core::T6Blend::ShadowOnly);
        assert_eq!(shadow.code(), FALLBACK_NO_DRAW_CODE);
        // The fallback pass leaves out every code whose blend nibble is 4+.
        assert!(FALLBACK_NO_DRAW_CODE != asset_core::T6Draw::NONE_CODE);
        assert!(FALLBACK_NO_DRAW_CODE & 0x0f >= 4);
    }
}
