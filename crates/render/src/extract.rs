use std::sync::Arc;
use std::time::Instant;

use bevy::prelude::*;
use bevy::render::Extract;
use render_frontend::assemble::drawsurf::tess::fx::FxCodeMeshPlan;
use render_frontend::assemble::drawsurf::tess::glass::GfxGlassMeshPlan;
use render_frontend::assemble::drawsurf::tess::mark::GfxMarkMeshPlan;
use render_frontend::assemble::drawsurf::tess::particle_cloud::FxParticleCloudPlan;
use render_frontend::assemble::drawsurf::tess::smodel::SmodelGpuPlan;
use render_frontend::assemble::drawsurf::tess::world::WorldDrawGpuPlan;
use render_frontend::assemble::drawsurf::tess::xmodel::XModelDrawPlan;
use render_gpu::diag::render_frame_diag::SharedRenderStagesSlot;
use render_gpu::{
    ExtractedRenderFrameProducts, ExtractedRuntimeImageHandles, InstalledRenderWorld,
    PublishedRenderFrame, SamplerTable,
};

fn take_published<T>(share: Option<&Arc<Vec<T>>>) -> (Arc<Vec<T>>, u32) {
    match share {
        Some(rows) => (Arc::clone(rows), 1),
        None => (Arc::new(Vec::new()), 0),
    }
}

fn overlay_world_shares(
    plan: Option<&WorldDrawGpuPlan>,
) -> (
    Arc<Vec<render_frame::WorldVertex>>,
    Arc<Vec<u32>>,
    Arc<Vec<(u32, u32)>>,
) {
    match plan {
        Some(plan) => (
            take_published(plan.decoded_share.as_ref()).0,
            take_published(plan.index_share.as_ref()).0,
            take_published(plan.range_share.as_ref()).0,
        ),
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
        ),
    }
}

fn overlay_smodel_shares(
    plan: Option<&SmodelGpuPlan>,
) -> (
    Arc<Vec<render_frame::SmodelVertex>>,
    Arc<Vec<u32>>,
    Arc<Vec<(u32, u32)>>,
) {
    match plan {
        Some(plan) => (
            take_published(plan.decoded_share.as_ref()).0,
            take_published(plan.index_share.as_ref()).0,
            take_published(plan.range_share.as_ref()).0,
        ),
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
        ),
    }
}

fn overlay_xmodel_shares(
    plan: Option<&XModelDrawPlan>,
) -> (
    Arc<Vec<render_frame::SmodelVertex>>,
    Arc<Vec<u32>>,
    Arc<Vec<(u32, u32)>>,
) {
    match plan {
        Some(plan) => (
            take_published(plan.decoded_share.as_ref()).0,
            take_published(plan.index_share.as_ref()).0,
            take_published(plan.range_share.as_ref()).0,
        ),
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
        ),
    }
}

fn insert_empty_colour(commands: &mut Commands) {
    commands.insert_resource(InstalledRenderWorld::default());
    commands.insert_resource(PublishedRenderFrame::default());
}

fn world_colour_extract_counts(plan: Option<&WorldDrawGpuPlan>) -> (usize, usize, usize) {
    match plan {
        Some(plan) => match plan.exact_packed_vertices() {
            Ok(vertices) => (
                vertices.len(),
                plan.indices().len(),
                plan.vertex_layer_rows().len(),
            ),
            Err(_) => (0, 0, 0),
        },
        None => (0, 0, 0),
    }
}

fn smodel_colour_extract_counts(plan: Option<&SmodelGpuPlan>) -> (usize, usize) {
    match plan {
        Some(plan) => match plan.exact_packed_vertices() {
            Ok(vertices) => (vertices.len(), plan.indices().len()),
            Err(_) => (0, 0),
        },
        None => (0, 0),
    }
}

pub fn extract_exact_colour(
    mut commands: Commands,
    sealed: Extract<Option<Res<PublishedRenderFrame>>>,
    existing: Option<Res<PublishedRenderFrame>>,
    mut focus_submit: ResMut<render_gpu::FocusedOwnerSubmitState>,
    slot: Option<Res<SharedRenderStagesSlot>>,
) {
    let started = Instant::now();
    if let Some(existing) = existing.as_ref() {
        render_gpu::emit_focused_owner_submit(
            &existing.frame_products,
            &mut focus_submit,
            None,
            None,
            "render_not_scheduled",
            0,
            0,
            0,
        );
    }
    if let Some(sealed) = sealed.as_ref() {
        commands.insert_resource(sealed.world().clone());
        commands.insert_resource((**sealed).clone());
    } else {
        insert_empty_colour(&mut commands);
    }
    if let Some(slot) = slot {
        slot.stamp_extract_products(started.elapsed().as_secs_f32() * 1000.0, 1, 0);
        slot.stamp_extract_colour(0.0, 0.0, 1, 0, 0, 1, 1, 0);
    }
}

pub fn seal_render_frame(
    mut commands: Commands,
    material: (
        Option<Res<render_frontend::assemble::drawsurf::MaterialGeneration>>,
        Option<Res<render_frontend::assemble::drawsurf::MaterialFrameInputs>>,
        Option<Res<render_frontend::assemble::drawsurf::FrameAssemblyInputs>>,
        Option<Res<render_frontend::assemble::drawsurf::RenderFrameProducts>>,
    ),
    world: Option<Res<WorldDrawGpuPlan>>,
    smodel: Option<Res<SmodelGpuPlan>>,
    smc: Option<Res<render_frontend::prepare::scene::smodel_geom_cache::WorldStaticModelCache>>,
    static_identity: (
        Option<Res<render_frontend::assemble::drawsurf::StaticDrawLane>>,
        Option<Res<frame::WorldGeneration>>,
        Option<Res<frame::WorldProducts>>,
    ),
    xmodel: Option<Res<XModelDrawPlan>>,
    fx: Option<Res<FxCodeMeshPlan>>,
    particle_cloud: Option<Res<FxParticleCloudPlan>>,
    mark_mesh: Option<Res<GfxMarkMeshPlan>>,
    glass_mesh: Option<Res<GfxGlassMeshPlan>>,
    samplers: Option<Res<SamplerTable>>,
    images: Option<Res<render_frontend::assemble::drawsurf::RuntimeImageHandles>>,
    spawn_job: Option<Res<render_gpu::GpuSubmitReady>>,
    sun: (
        Option<Res<render_frontend::assemble::drawsurf::MapSunEffects>>,
        Option<Res<render_frontend::assemble::drawsurf::SunEffectsFrameInput>>,
    ),
    (existing_world, existing_frame): (
        Option<Res<InstalledRenderWorld>>,
        Option<Res<PublishedRenderFrame>>,
    ),
) {
    let (retained, world_generation, world_products) = static_identity;
    let world_products = world_products.map(|products| *products).unwrap_or_default();
    let (runtime, mat_frame, assembly, products) = material;
    let Some(runtime) = runtime.as_ref() else {
        insert_empty_colour(&mut commands);
        return;
    };
    let Some(mat_frame) = mat_frame.as_ref() else {
        insert_empty_colour(&mut commands);
        return;
    };
    let generation = runtime.catalog.generation_id;
    let world_generation = world_generation
        .as_ref()
        .map(|generation| **generation)
        .unwrap_or(frame::WorldGeneration(None));
    let Some(products) = products.as_ref() else {
        insert_empty_colour(&mut commands);
        return;
    };
    let snapshot = products.published();
    let coherent =
        frame_submission_matches(assembly.as_deref(), &snapshot, generation, world_generation);
    if !coherent {
        insert_empty_colour(&mut commands);
        return;
    }
    let frame_products = ExtractedRenderFrameProducts(snapshot);
    let cpu_port_len = runtime.programs.ports().len();
    let skip_ports = existing_world.as_ref().is_some_and(|world| {
        render_gpu::colour_ports_static(
            world.generation,
            world.ports.len(),
            generation,
            cpu_port_len,
        )
    });
    let (ports, _ports_ms) = if skip_ports {
        (Vec::new(), 0.0_f32)
    } else {
        let ports_started = Instant::now();

        let ports: Vec<render_gpu::AdmittedExactPort> = runtime
            .programs
            .ports()
            .iter()
            .map(|port| render_gpu::AdmittedExactPort {
                id: port.id(),
                abi: port.abi().clone(),
                module: port.shared_module(),
                layout: port.wgpu_layout().clone(),
            })
            .collect();
        (ports, ports_started.elapsed().as_secs_f32() * 1000.0)
    };
    let warm_pipelines = spawn_job.as_ref().is_some_and(|job| job.warm_pipelines);
    let (world_v, world_i, world_layer_n) =
        world_colour_extract_counts(world.as_ref().map(|plan| &**plan));
    let (smodel_v, smodel_i) = smodel_colour_extract_counts(smodel.as_ref().map(|plan| &**plan));

    let skip_world_smodel = existing_world.as_ref().is_some_and(|world| {
        render_gpu::colour_world_smodel_static(
            (world.world_generation, world.world_products),
            world.static_geometry.world_vertices.len(),
            world.static_geometry.world_indices.len(),
            world.static_geometry.world_layer.len(),
            world.static_geometry.smodel_vertices.len(),
            world.static_geometry.smodel_indices.len(),
            (world_generation, world_products),
            world_v,
            world_i,
            world_layer_n,
            smodel_v,
            smodel_i,
        )
    });
    let smc_revision = smc.as_ref().map(|cache| cache.content_revision());
    let skip_smc_maps = skip_world_smodel
        && existing_world
            .as_ref()
            .is_some_and(|world| world.smc_revision == smc_revision);
    let empty_static = || {
        (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
        )
    };
    let (world_vertices, world_layer, world_indices, world_surface_ranges, world_vertex_refusal) =
        if skip_world_smodel {
            empty_static()
        } else {
            match world.as_ref() {
                Some(plan) => match plan.exact_packed_vertices() {
                    Ok(_) => {
                        let (verts, _) = take_published(plan.vertex_share.as_ref());
                        let (layer, _) = take_published(plan.layer_share.as_ref());
                        let (inds, _) = take_published(plan.index_share.as_ref());
                        let (ranges, _) = take_published(plan.range_share.as_ref());
                        (verts, layer, inds, ranges, None)
                    }
                    Err(cause) => {
                        let (empty_v, empty_l, empty_i, empty_r, _) = empty_static();
                        (empty_v, empty_l, empty_i, empty_r, Some(cause))
                    }
                },
                None => empty_static(),
            }
        };
    let empty_smodel = || {
        (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
        )
    };
    let (
        smodel_vertices,
        smodel_indices,
        smodel_surface_ranges,
        smodel_vertex_refusal,
        smodel_cached_vertices,
        smodel_surface_verts,
    ) = if skip_world_smodel {
        empty_smodel()
    } else {
        match smodel.as_ref() {
            Some(plan) => match plan.exact_packed_vertices() {
                Ok(_) => {
                    let (verts, _) = take_published(plan.packed_share.as_ref());
                    let (inds, _) = take_published(plan.index_share.as_ref());
                    let (ranges, _) = take_published(plan.range_share.as_ref());
                    let (cached, _) = take_published(plan.cached_share.as_ref());
                    let (vert_ranges, _) = take_published(plan.vert_range_share.as_ref());
                    (verts, inds, ranges, None, cached, vert_ranges)
                }
                Err(cause) => {
                    let (empty_v, empty_i, empty_r, _, empty_c, empty_vr) = empty_smodel();
                    (empty_v, empty_i, empty_r, Some(cause), empty_c, empty_vr)
                }
            },
            None => empty_smodel(),
        }
    };
    let mut smc_vb_patches = Vec::new();
    let mut smc_ib_patches = Vec::new();
    let mut smc_index_baked = Vec::new();
    let smodel_pretess_indices = retained
        .as_ref()
        .map(|retained| Arc::clone(&retained.smodel_pretess_indices))
        .unwrap_or_else(|| Arc::new(Vec::new()));
    let smodel_index_layout_revision = retained
        .as_ref()
        .map(|retained| retained.smodel_index_layout_revision)
        .unwrap_or(0);
    if let Some(cache) = smc.as_ref() {
        smc_vb_patches = cache.vb_patches().to_vec();
        smc_ib_patches = cache.ib_patches().to_vec();
        if !skip_smc_maps {
            smc_index_baked = cache.baked_cache_indices();
        }
    }
    let (
        xmodel_vertices,
        xmodel_indices,
        xmodel_surface_ranges,
        xmodel_vertex_refusal,
        _xmodel_arc,
        _xmodel_i_arc,
        _xmodel_r_arc,
    ) = match xmodel.as_ref() {
        Some(plan) => match plan.exact_packed_vertices() {
            Ok(_) => {
                let (verts, packed_arc) = take_published(plan.packed_share.as_ref());
                let (inds, i_arc) = take_published(plan.index_share.as_ref());
                let (ranges, r_arc) = take_published(plan.range_share.as_ref());
                (verts, inds, ranges, None, packed_arc, i_arc, r_arc)
            }
            Err(cause) => (
                Arc::new(Vec::new()),
                Arc::new(Vec::new()),
                Arc::new(Vec::new()),
                Some(cause),
                0u32,
                1u32,
                1u32,
            ),
        },
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
            0u32,
            1u32,
            1u32,
        ),
    };
    let xmodel_packed_segments = xmodel
        .as_ref()
        .map(|plan| plan.packed_segments)
        .unwrap_or_default();
    let xmodel_revision = xmodel.as_ref().map(|plan| plan.revision).unwrap_or(0);
    let xmodel_topology_revision = xmodel
        .as_ref()
        .map(|plan| plan.topology_revision)
        .unwrap_or(0);
    let (
        fx_vertices,
        fx_indices,
        fx_surface_ranges,
        fx_vertex_refusal,
        fx_revision,
        _fx_v_arc,
        _fx_i_arc,
        _fx_r_arc,
    ) = match fx.as_ref() {
        Some(plan) => match plan.exact_packed_vertices() {
            Ok(_) => {
                let verts = Arc::clone(&plan.vertices);
                let inds = Arc::clone(&plan.indices);
                let (ranges, r_arc) = take_published(plan.range_share.as_ref());
                (verts, inds, ranges, None, plan.revision, 1u32, 1u32, r_arc)
            }
            Err(cause) => (
                Arc::new(Vec::new()),
                Arc::new(Vec::new()),
                Arc::new(Vec::new()),
                Some(cause),
                plan.revision,
                1u32,
                1u32,
                1u32,
            ),
        },
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
            0,
            1u32,
            1u32,
            1u32,
        ),
    };
    let (
        particle_cloud_vertices,
        particle_cloud_indices,
        particle_cloud_surface_ranges,
        particle_cloud_template,
        particle_cloud_revision,
        _particle_r_arc,
    ) = match particle_cloud.as_ref() {
        Some(plan) => {
            let (ranges, r_arc) = take_published(plan.range_share.as_ref());
            (
                Arc::clone(&plan.vertices),
                Arc::clone(&plan.indices),
                ranges,
                plan.template_counts(),
                plan.revision,
                r_arc,
            )
        }
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            (0, 0),
            0,
            1u32,
        ),
    };
    let (
        mark_mesh_vertices,
        mark_mesh_indices,
        mark_mesh_surface_ranges,
        mark_mesh_revision,
        _mark_v_arc,
        _mark_i_arc,
        _mark_r_arc,
    ) = match mark_mesh.as_ref() {
        Some(plan) => {
            let verts = Arc::clone(&plan.vertices);
            let inds = Arc::clone(&plan.indices);
            let (ranges, r_arc) = take_published(plan.range_share.as_ref());
            (verts, inds, ranges, plan.revision, 1u32, 1u32, r_arc)
        }
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            0,
            1u32,
            1u32,
            1u32,
        ),
    };
    let (
        glass_mesh_vertices,
        glass_mesh_indices,
        glass_mesh_surface_ranges,
        glass_mesh_vertex_refusal,
        glass_mesh_revision,
        _glass_v_arc,
        _glass_i_arc,
        _glass_r_arc,
    ) = match glass_mesh.as_ref() {
        Some(plan) => match plan.exact_packed_vertices() {
            Ok(_) => {
                let verts = Arc::clone(&plan.vertices);
                let inds = Arc::clone(&plan.indices);
                let (ranges, r_arc) = take_published(plan.range_share.as_ref());
                (verts, inds, ranges, None, plan.revision, 1u32, 1u32, r_arc)
            }
            Err(cause) => (
                Arc::new(Vec::new()),
                Arc::new(Vec::new()),
                Arc::new(Vec::new()),
                Some(cause),
                plan.revision,
                1u32,
                1u32,
                1u32,
            ),
        },
        None => (
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
            0,
            1u32,
            1u32,
            1u32,
        ),
    };
    let image_handles = images
        .as_ref()
        .map(|handles| (**handles).clone())
        .unwrap_or_default();

    let mut exec_frame = existing_frame
        .as_ref()
        .map(|frame| frame.exec_frame.clone())
        .unwrap_or_default();
    render_frontend::assemble::drawsurf::material_exec::refresh(
        &mut exec_frame,
        mat_frame,
        assembly.as_deref(),
    );
    let mut next_world = render_gpu::RenderWorldData {
        generation,
        catalog: Some(Arc::clone(&runtime.catalog)),
        prepared: Some(Arc::clone(&runtime.prepared)),
        world_generation,
        world_products,
        smc_revision,
        ports: Arc::new(ports),
        static_geometry: Arc::new(render_gpu::ExtractedStaticGeometry {
            world_vertices,
            world_layer,
            world_indices,
            world_surface_ranges,
            world_vertex_refusal,
            smodel_vertices,
            smodel_indices,
            smodel_surface_ranges,
            smodel_vertex_refusal,
            smodel_cached_vertices,
            smodel_surface_verts,
        }),
        smc_index_baked: Arc::new(smc_index_baked),
        smodel_pretess_indices,
        smodel_index_layout_revision,
        sampler_table: samplers.as_ref().map(|table| (**table).clone()),
        image_handles,
        sorted_material_names: Arc::new(if skip_ports {
            Vec::new()
        } else {
            render_gpu::dump_sorted_material_names(&runtime.catalog)
        }),
        shader_program_names: Arc::new(if skip_ports {
            Vec::new()
        } else {
            render_gpu::dump_shader_program_names(&runtime.catalog)
        }),
        sun_effects: sun.0.and_then(|sun| sun.def),
    };
    let next_frame = render_gpu::RenderFrameData {
        frame_products,
        generation,
        world_generation,
        exec_frame,
        sun_shadow: mat_frame.sun_shadow,
        sun_effects: sun.1.and_then(|input| input.frame),
        warm_pipelines,
        pipeline_world_materials: spawn_job
            .as_ref()
            .map(|job| job.pipeline_world_materials.clone())
            .unwrap_or_default(),
        pipeline_smodel_materials: spawn_job
            .as_ref()
            .map(|job| job.pipeline_smodel_materials.clone())
            .unwrap_or_default(),
        pipeline_demand_revision: spawn_job
            .as_ref()
            .map(|job| job.pipeline_demand_revision)
            .unwrap_or(0),
        smc_vb_patches,
        smc_ib_patches,
        xmodel_vertices,
        xmodel_indices,
        xmodel_surface_ranges,
        xmodel_vertex_refusal,
        fx_vertices,
        fx_indices,
        fx_surface_ranges,
        fx_vertex_refusal,
        fx_revision,
        xmodel_revision,
        xmodel_topology_revision,
        xmodel_packed_segments,
        particle_cloud_vertices,
        particle_cloud_indices,
        particle_cloud_surface_ranges,
        particle_cloud_template,
        particle_cloud_revision,
        mark_mesh_vertices,
        mark_mesh_indices,
        mark_mesh_surface_ranges,
        mark_mesh_revision,
        glass_mesh_vertices,
        glass_mesh_indices,
        glass_mesh_surface_ranges,
        glass_mesh_revision,
        glass_mesh_vertex_refusal,
        glass_mesh_bounds: glass_mesh
            .as_ref()
            .and_then(|plan| plan.bounds_share.clone())
            .unwrap_or_default(),
    };
    if let Some(existing) = existing_world.as_ref() {
        reuse_installed_rows(
            &mut next_world,
            existing,
            WorldRowReuse {
                static_geometry: skip_world_smodel,
                smc_index_baked: skip_smc_maps,
                ports: skip_ports,
            },
        );
    }
    let world = InstalledRenderWorld::new(next_world);
    commands.insert_resource(world.clone());
    commands.insert_resource(PublishedRenderFrame::seal(world, next_frame));
}

/// Which rows of the installed world the seal decided are still the ones the
/// GPU already holds. The flags come from the static comparisons above; this
/// only moves the previously published handles across so publication does not
/// hand the render world a second copy of geometry it did not rebuild.
#[derive(Clone, Copy, Debug)]
struct WorldRowReuse {
    static_geometry: bool,
    smc_index_baked: bool,
    ports: bool,
}

fn reuse_installed_rows(
    next: &mut render_gpu::RenderWorldData,
    existing: &InstalledRenderWorld,
    reuse: WorldRowReuse,
) {
    if reuse.static_geometry {
        next.static_geometry = existing.static_geometry.clone();
    }
    if reuse.smc_index_baked {
        next.smc_index_baked = existing.smc_index_baked.clone();
    }
    if reuse.ports {
        next.ports = existing.ports.clone();
        next.sorted_material_names = existing.sorted_material_names.clone();
        next.shader_program_names = existing.shader_program_names.clone();
    }
}

fn frame_submission_matches(
    inputs: Option<&render_frontend::assemble::drawsurf::FrameAssemblyInputs>,
    snapshot: &render_frame::FrameProductsSnapshot,
    generation: render_frontend::assemble::drawsurf::MaterialGenerationId,
    world_generation: frame::WorldGeneration,
) -> bool {
    inputs.is_some_and(|inputs| {
        inputs.frame_id == snapshot.frame_id
            && inputs.catalog_generation == generation
            && inputs.world_generation == world_generation
    }) && snapshot.products.iter().all(|product| {
        matches!(product.status, render_frame::FrameProductStatus::Missing(_))
            || product.generation_id == generation
    })
}

pub fn extract_image_handles(
    mut commands: Commands,
    handles: Extract<Option<Res<render_frontend::assemble::drawsurf::RuntimeImageHandles>>>,
    existing: Option<ResMut<ExtractedRuntimeImageHandles>>,
    slot: Option<Res<SharedRenderStagesSlot>>,
) {
    let Some(handles) = handles.as_ref() else {
        commands.insert_resource(ExtractedRuntimeImageHandles::default());
        return;
    };
    let started = Instant::now();
    let extract_images_arc = 1;
    let next = (**handles).clone();
    if let Some(mut existing) = existing {
        existing.handles = next;
    } else {
        commands.insert_resource(ExtractedRuntimeImageHandles { handles: next });
    }
    if let Some(slot) = slot {
        slot.stamp_extract_images(started.elapsed().as_secs_f32() * 1000.0, extract_images_arc);
    }
}

pub fn extract_postfx(
    runtime: Extract<Option<Res<render_frontend::assemble::drawsurf::MaterialGeneration>>>,
    samplers: Extract<Option<Res<SamplerTable>>>,
    frame: Extract<Res<render_frontend::assemble::drawsurf::dof::DofFrame>>,
    film: Extract<Res<render_frontend::assemble::drawsurf::FilmVisionView>>,
    glow_dvars: Extract<Res<render_frontend::assemble::drawsurf::dof::GlowDvars>>,
    draw_method: Extract<Res<render_frontend::assemble::drawsurf::ColourDrawMethod>>,
    mut extracted: ResMut<render_gpu::ExtractedPostFx>,
) {
    use render_frontend::assemble::drawsurf::postfx_plan::RuntimePostFxResources;
    let films = runtime.as_ref().and_then(|r| match &r.postfx {
        RuntimePostFxResources::Ready(films) => Some(films),
        _ => None,
    });
    match films {
        Some(films)
            if extracted.films.first().map(|f| f.generation)
                != films.first().map(|f| f.generation) =>
        {
            extracted.films = films
                .iter()
                .map(|film| render_gpu::ExtractedFilm {
                    name: film.name,
                    generation: film.generation,
                    port: render_gpu::AdmittedExactPort {
                        id: film.port.id(),
                        abi: film.port.abi().clone(),
                        module: film.port.shared_module(),
                        layout: film.port.wgpu_layout().clone(),
                    },
                    shader: film.shader.clone(),
                    shell: film.shell.clone(),
                })
                .collect();
        }
        None => extracted.films.clear(),
        _ => {}
    }
    let blood = runtime.as_ref().and_then(|r| r.blood.as_ref());
    if extracted.blood.as_ref().map(|b| b.film.generation) != blood.map(|b| b.film.generation) {
        extracted.blood = blood.map(|blood| render_gpu::ExtractedBlood {
            film: render_gpu::ExtractedFilm {
                name: blood.film.name,
                generation: blood.film.generation,
                port: render_gpu::AdmittedExactPort {
                    id: blood.film.port.id(),
                    abi: blood.film.port.abi().clone(),
                    module: blood.film.port.shared_module(),
                    layout: blood.film.port.wgpu_layout().clone(),
                },
                shader: blood.film.shader.clone(),
                shell: blood.film.shell.clone(),
            },
            texture_slots: blood.texture_slots.clone(),
        });
    }
    extracted.vision = film.current;
    extracted.frame = render_gpu::DofFrame {
        dof: render_gpu::DepthOfField {
            view_model_start: frame.dof.view_model_start,
            view_model_end: frame.dof.view_model_end,
            near_start: frame.dof.near_start,
            near_end: frame.dof.near_end,
            far_start: frame.dof.far_start,
            far_end: frame.dof.far_end,
            near_blur: frame.dof.near_blur,
            far_blur: frame.dof.far_blur,
        },
        blur: film.blur,
        grading: film.grading,
        bias: frame.bias,
        scene_near: frame.scene_near,
        view_model_near: frame.view_model_near,
        glow: render_gpu::GlowFrame {
            r_glow: glow_dvars.enable || film.script_forced,
            r_fullbright: !film.script_forced
                && matches!(
                    **draw_method,
                    render_frontend::assemble::drawsurf::ColourDrawMethod::Fullbright
                ),
            ..render_gpu::GlowFrame::from_vision(extracted.vision)
        },
    };
    extracted.sampler = samplers.as_ref().and_then(|s| s.decode(0x62).ok());
    extracted.depth_sampler = samplers.as_ref().and_then(|s| s.decode(0x61).ok());
}

/// bo2zm: each world surface's colour map slot, Black Ops II draw code and
/// lightmap page, from the world plan's per-surface material table.
fn world_surface_styles(
    world: Option<&WorldDrawGpuPlan>,
) -> (Arc<Vec<u32>>, Arc<Vec<u32>>, Arc<Vec<u8>>) {
    let (colors, draws, lightmaps) = world
        .map(WorldDrawGpuPlan::fallback_surface_styles)
        .unwrap_or_default();
    (Arc::new(colors), Arc::new(draws), Arc::new(lightmaps))
}

/// bo2zm: the T6 world lighting the fallback draw adds to lightmaps: the
/// sun's direction and exposure scale, then its colour, the sky's rotation
/// and brightness, its decode and fog share, then the vision's colour grade
/// (an identity grade at the vision's saturation without one), then the
/// world colour tables drawn (`adapters::t6_lut`). A T6 world fills the
/// T5 sun and exposure carriers; only T6 surfaces read this.
#[allow(clippy::too_many_arguments)]
fn t6_lighting(
    sun: Option<&render_frontend::assemble::drawsurf::MapDirPrimaryLight>,
    exposure: Option<&render_frontend::assemble::drawsurf::MapT5SunParseExposure>,
    sky: Option<[f32; 4]>,
    sky_parms: Option<[f32; 2]>,
    grade: Option<[[f32; 4]; 14]>,
    lut: [f32; 4],
    sun_sky_scale: [f32; 4],
) -> Option<[[f32; 4]; 19]> {
    let (sun, exposure) = (sun?, exposure?);
    // bo2mp: a Black Ops map keeps its sunParse exposure raw (its own world
    // path divides it by 8, `T5_HDRCONTROL_EXPOSURE_DIVISOR`); Black Ops
    // II's models took it raw, 8 times the light, and drew blown out white.
    let exposure_scale = if assets::map_game() == Some(asset_core::AssetNamespace::T5) {
        exposure.exposure / render_frontend::assemble::drawsurf::T5_HDRCONTROL_EXPOSURE_DIVISOR
    } else {
        exposure.exposure
    };
    let [sun_r, sun_g, sun_b, sky_scale] = sun_sky_scale;
    Some(t6_lighting_rows(
        [
            sun.direction[0],
            sun.direction[1],
            sun.direction[2],
            exposure_scale,
        ],
        [sun.color[0] * sun_r, sun.color[1] * sun_g, sun.color[2] * sun_b],
        // The sky's brightness (w, its sign the transition) times the sky
        // intensity (`skyColorMultiplier`).
        sky.map(|[x, y, z, w]| [x, y, z, w * sky_scale]),
        sky_parms,
        grade,
        exposure.saturation,
        lut,
    ))
}

/// The 19 lighting rows from their parts. Row 1's w is 1 (bo2mp: the
/// fallback pass reads 2 there as "IW3 lightmaps"; the vision's saturation
/// rides in the grade, row 16).
fn t6_lighting_rows(
    sun_dir_exposure: [f32; 4],
    sun_color: [f32; 3],
    sky: Option<[f32; 4]>,
    sky_parms: Option<[f32; 2]>,
    grade: Option<[[f32; 4]; 14]>,
    saturation: f32,
    lut: [f32; 4],
) -> [[f32; 4]; 19] {
    let grade = grade.unwrap_or([
        [-1.0, 1.0, 1.0, -1.0],
        [1.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [1.0, 1.0, 1.0, 0.0],
        [0.212585, 0.715195, 0.07222, saturation],
        [1.0, 1.0, 1.0, 0.0],
    ]);
    let rows = [
        sun_dir_exposure,
        [sun_color[0], sun_color[1], sun_color[2], 1.0],
        sky.unwrap_or([1.0, 0.0, 0.0, 1.0]),
        sky_parms.map_or([1.0, 1.0, 0.0, 0.0], |[x, w]| [x, w, 0.0, 0.0]),
    ];
    let mut all = [[0.0f32; 4]; 19];
    all[..4].copy_from_slice(&rows);
    all[4..18].copy_from_slice(&grade);
    all[18] = lut;
    all
}

/// bo2mp: the lighting Black Ops II's moving things (the first-person gun
/// and arms, players, grenades, marks, effects) draw with. A Black Ops II
/// (or Black Ops, CoD4) map has it in full (`t6_lighting`). A MW2 or MW3
/// map has a sun but no exposure carrier: under Black Ops II's rules its
/// guns and players still draw by the fallback pass, so it takes exposure
/// 1 (Infinity Ward's lit models write `albedo^2 * light` unscaled) and
/// its sun, or no sun at all, with an identity grade. Without this the pass
/// had no lighting group on those maps and drew none of them: the gun and
/// every player invisible on Favela, Countdown, Winter Crash.
fn t6_dynamic_lighting(
    full: Option<[[f32; 4]; 19]>,
    sun: Option<&render_frontend::assemble::drawsurf::MapDirPrimaryLight>,
    exposure: Option<&render_frontend::assemble::drawsurf::MapT5SunParseExposure>,
    sky: Option<[f32; 4]>,
    lut: [f32; 4],
) -> Option<[[f32; 4]; 19]> {
    if full.is_some() || !assets::t6_rules() {
        return full;
    }
    let (exposure, saturation) = exposure.map_or((1.0, 1.0), |e| (e.exposure, e.saturation));
    // bo2mp: the map's sun colour as Infinity Ward's own lit models take it
    // (sRGB to linear, `lighting_iw4::light_diffuse`); raw, MW3's sun washed
    // Black Ops II's guns and players out white.
    let (direction, color) = sun.map_or(([0.0, 0.0, 1.0], [0.0; 3]), |s| (s.direction, s.color.map(srgb_to_linear)));
    Some(t6_lighting_rows(
        [direction[0], direction[1], direction[2], exposure],
        color,
        sky,
        None,
        None,
        saturation,
        lut,
    ))
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

pub fn extract_geometry(
    mut commands: Commands,
    world: Extract<Option<Res<WorldDrawGpuPlan>>>,
    smodel: Extract<Option<Res<SmodelGpuPlan>>>,
    xmodel: Extract<Option<Res<XModelDrawPlan>>>,
    runtime: Extract<Option<Res<render_frontend::assemble::drawsurf::MaterialGeneration>>>,
    dpvs: Extract<Option<Res<render_frontend::prepare::scene::cull::DpvsFrameStats>>>,
    spawn_job: Extract<Option<Res<render_gpu::GpuSubmitReady>>>,
    existing: Option<ResMut<render_gpu::ExtractedDiagnosticGeometry>>,
    slot: Option<Res<SharedRenderStagesSlot>>,
    lighting: (
        Extract<Option<Res<render_frontend::assemble::drawsurf::MapDirPrimaryLight>>>,
        Extract<Option<Res<render_frontend::assemble::drawsurf::MapT5SunParseExposure>>>,
        Extract<Option<Res<render_frontend::prepare::scene::world::WorldScene>>>,
    ),
) {
    if !render_gpu::geometry_diagnostic_enabled() {
        return;
    }
    let sky = lighting.2.as_ref().and_then(|scene| scene.t5_sky_dynamic_intensity);
    let lut = lighting
        .2
        .as_ref()
        .map_or([0.0, 0.0, 1.0, 0.0], |scene| scene.t6_lut_pick);
    let mut full_lighting = t6_lighting(
        lighting.0.as_deref(),
        lighting.1.as_deref(),
        sky,
        lighting.2.as_ref().and_then(|scene| scene.t6_sky_parms),
        lighting.2.as_ref().and_then(|scene| scene.t6_vision_grade),
        lut,
        lighting
            .2
            .as_ref()
            .map_or([1.0; 4], |scene| scene.t6_sun_sky_scale),
    );
    // bo2mp: a CoD4 / World at War world's lightmaps are read by the
    // fallback pass's IW3 branch: the sun colour's alpha says so (2).
    if lighting.2.as_ref().is_some_and(|scene| scene.iw3_lightmaps)
        && let Some(l) = full_lighting.as_mut()
    {
        l[1][3] = 2.0;
    }
    // bo2mp: BO2's bloom and final colour only where the map has its full
    // lighting (not over a MW2 world drawn by its own path).
    let t6_bloom = full_lighting.is_some();
    let t6_lighting =
        t6_dynamic_lighting(full_lighting, lighting.0.as_deref(), lighting.1.as_deref(), sky, lut);
    // bo2mp: which lighting the moving things got this map, on change.
    {
        static LAST: std::sync::Mutex<Option<(bool, bool, bool, bool)>> = std::sync::Mutex::new(None);
        let now = (
            t6_lighting.is_some(),
            t6_bloom,
            lighting.0.is_some(),
            lighting.1.is_some(),
        );
        if let Ok(mut last) = LAST.lock()
            && *last != Some(now)
        {
            *last = Some(now);
            diag::info!(
                Fpv,
                "bo2 dynamic lighting: lighting_group_input={} full_t6={} (sun={} exposure={}) t6_rules={} light_sampler={}",
                now.0,
                now.1,
                now.2,
                now.3,
                assets::t6_rules(),
                asset_world::t6_light_sampler().map_or("none", |s| if s.foreign.is_some() { "foreign" } else { "t6" }),
            );
        }
    }
    let sun_sample_size_near = lighting
        .2
        .as_ref()
        .map_or(0.0, |scene| scene.t6_sun_sample_size_near);
    let Some(runtime) = runtime.as_ref() else {
        return;
    };
    let started = Instant::now();
    let g0_world_surfs = dpvs
        .as_ref()
        .map(|stats| stats.g0_world_surfs.clone())
        .unwrap_or_default();
    let generation = runtime.catalog.generation_id;
    // bo2zm: the BO2 props, rebuilt when the material generation or the
    // placements change.
    // The exposure too: the props carry it baked in, and a T6 map changes
    // it per exposure volume (in 1/16 stop steps).
    let props_key = (
        runtime.catalog.generation_id,
        smodel.as_ref().map_or(0, |plan| plan.placements.len()),
        t6_lighting.map_or(0, |l| l[0][3].to_bits()),
    );
    let t6_props = match existing.as_ref() {
        Some(extracted) if extracted.t6_props_key == Some(props_key) => (
            Arc::clone(&extracted.t6_props),
            Arc::clone(&extracted.t6_prop_instances),
            Arc::clone(&extracted.t6_prop_shine),
        ),
        _ => {
            let exposure = t6_lighting.map_or(1.0, |l| l[0][3]);
            let (draws, instances, shine) = smodel
                .as_ref()
                .map(|plan| plan.t6_fallback_props(&runtime.catalog, exposure))
                .unwrap_or_default();
            (Arc::new(draws), Arc::new(instances), Arc::new(shine))
        }
    };
    let world_v = world
        .as_ref()
        .map_or(0, |plan| plan.decoded_vertices().len());
    let world_i = world.as_ref().map_or(0, |plan| plan.indices().len());
    let smodel_v = smodel
        .as_ref()
        .map_or(0, |plan| plan.decoded_vertices().len());
    let smodel_i = smodel.as_ref().map_or(0, |plan| plan.indices().len());
    let overlay_gpu_wait = spawn_job.as_ref().is_some_and(|job| job.overlay_gpu_wait);
    let skip_world_smodel = overlay_gpu_wait
        || existing.as_ref().is_some_and(|extracted| {
            extracted.generation == generation
                && extracted.world_vertices.len() == world_v
                && extracted.world_indices.len() == world_i
                && extracted.smodel_vertices.len() == smodel_v
                && extracted.smodel_indices.len() == smodel_i
        });
    if skip_world_smodel {
        let Some(mut existing) = existing else {
            if let Some(slot) = slot {
                slot.stamp_extract_diag(started.elapsed().as_secs_f32() * 1000.0);
            }
            return;
        };
        existing.overlay_gpu_wait = overlay_gpu_wait;
        if overlay_gpu_wait {
            if let Some(slot) = slot {
                slot.stamp_extract_diag(started.elapsed().as_secs_f32() * 1000.0);
            }
            return;
        }
        let (_, _, world_ranges) = overlay_world_shares(world.as_ref().map(|plan| plan.as_ref()));
        let (_, _, smodel_ranges) =
            overlay_smodel_shares(smodel.as_ref().map(|plan| plan.as_ref()));
        let (xmodel_vertices, xmodel_indices, xmodel_surface_ranges) =
            overlay_xmodel_shares(xmodel.as_ref().map(|plan| plan.as_ref()));
        existing.world_surface_ranges = world_ranges;
        existing.smodel_surface_ranges = smodel_ranges;
        existing.xmodel_vertices = xmodel_vertices;
        existing.xmodel_indices = xmodel_indices;
        existing.xmodel_surface_ranges = xmodel_surface_ranges;
        existing.xmodel_revision = xmodel.as_ref().map_or(0, |plan| plan.revision);
        existing.g0_world_surfs = g0_world_surfs;
        let (colors, draws, lightmaps) =
            world_surface_styles(world.as_ref().map(|plan| plan.as_ref()));
        if colors != existing.world_surface_color {
            existing.world_surface_color = colors;
        }
        if draws != existing.world_surface_draw {
            existing.world_surface_draw = draws;
        }
        if lightmaps != existing.world_surface_lightmap {
            existing.world_surface_lightmap = lightmaps;
        }
        let layers = world
            .as_ref()
            .map_or_else(Vec::new, |plan| plan.fallback_surface_layers());
        if *existing.world_surface_layers != layers {
            existing.world_surface_layers = Arc::new(layers);
        }
        if let Some(plan) = world.as_ref()
            && !Arc::ptr_eq(&existing.world_layer_uvs, &plan.t6_layer_share)
        {
            existing.world_layer_uvs = Arc::clone(&plan.t6_layer_share);
        }
        let shine = world
            .as_ref()
            .map_or_else(Vec::new, |plan| plan.fallback_surface_shine());
        if *existing.world_surface_shine != shine {
            existing.world_surface_shine = Arc::new(shine);
        }
        existing.t6_probe_slot =
            render_frontend::assemble::drawsurf::tess::world::t6_probe_slot(&runtime.catalog);
        if let Some(scene) = lighting.2.as_ref()
            && !Arc::ptr_eq(&existing.t6_probes, &scene.t6_probes)
        {
            existing.t6_probes = Arc::clone(&scene.t6_probes);
        }
        let primaries = world.as_ref().map_or_else(Vec::new, |plan| plan.surface_primary_light.clone());
        if *existing.world_surface_primary != primaries {
            existing.world_surface_primary = Arc::new(primaries);
        }
        if let Some(scene) = lighting.2.as_ref()
            && !Arc::ptr_eq(&existing.t6_lights, &scene.t6_lights)
        {
            existing.t6_lights = Arc::clone(&scene.t6_lights);
        }
        let lut = lighting.2.as_ref().and_then(|scene| scene.t6_lut.clone());
        if !match (&existing.t6_lut, &lut) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        } {
            existing.t6_lut = lut;
        }
        existing.t6_lighting = t6_lighting;
        existing.t6_bloom = t6_bloom;
        existing.t6_sun_sample_size_near = sun_sample_size_near;
        existing.t6_props = t6_props.0;
        existing.t6_prop_instances = t6_props.1;
        existing.t6_prop_shine = t6_props.2;
        existing.t6_props_key = Some(props_key);
        existing.t6_sky_slot =
            render_frontend::assemble::drawsurf::tess::world::t6_sky_slot(&runtime.catalog);
        if let Some(slot) = slot {
            slot.stamp_extract_diag(started.elapsed().as_secs_f32() * 1000.0);
        }
        return;
    }
    let (world_vertices, world_indices, world_surface_ranges) =
        overlay_world_shares(world.as_ref().map(|plan| plan.as_ref()));
    let (smodel_vertices, smodel_indices, smodel_surface_ranges) =
        overlay_smodel_shares(smodel.as_ref().map(|plan| plan.as_ref()));
    let (xmodel_vertices, xmodel_indices, xmodel_surface_ranges) =
        overlay_xmodel_shares(xmodel.as_ref().map(|plan| plan.as_ref()));
    let (world_surface_color, world_surface_draw, world_surface_lightmap) =
        world_surface_styles(world.as_ref().map(|plan| plan.as_ref()));
    let next = render_gpu::ExtractedDiagnosticGeometry {
        generation,
        overlay_gpu_wait,
        world_vertices,
        world_indices,
        world_surface_ranges,
        smodel_vertices,
        smodel_indices,
        smodel_surface_ranges,
        xmodel_vertices,
        xmodel_indices,
        xmodel_surface_ranges,
        xmodel_revision: xmodel.as_ref().map_or(0, |plan| plan.revision),
        g0_world_surfs,
        world_surface_color,
        world_surface_draw,
        world_surface_lightmap,
        t6_lighting,
        t6_bloom,
        t6_sun_sample_size_near: sun_sample_size_near,
        t6_props: t6_props.0,
        t6_prop_instances: t6_props.1,
        t6_prop_shine: t6_props.2,
        t6_props_key: Some(props_key),
        t6_sky_slot: render_frontend::assemble::drawsurf::tess::world::t6_sky_slot(&runtime.catalog),
        world_surface_primary: Arc::new(
            world
                .as_ref()
                .map_or_else(Vec::new, |plan| plan.surface_primary_light.clone()),
        ),
        t6_lights: lighting
            .2
            .as_ref()
            .map_or_else(Default::default, |scene| Arc::clone(&scene.t6_lights)),
        world_surface_layers: Arc::new(
            world
                .as_ref()
                .map_or_else(Vec::new, |plan| plan.fallback_surface_layers()),
        ),
        world_layer_uvs: world
            .as_ref()
            .map_or_else(Default::default, |plan| Arc::clone(&plan.t6_layer_share)),
        world_surface_shine: Arc::new(
            world
                .as_ref()
                .map_or_else(Vec::new, |plan| plan.fallback_surface_shine()),
        ),
        t6_probe_slot: render_frontend::assemble::drawsurf::tess::world::t6_probe_slot(&runtime.catalog),
        t6_probes: lighting
            .2
            .as_ref()
            .map_or_else(Default::default, |scene| Arc::clone(&scene.t6_probes)),
        t6_lut: lighting.2.as_ref().and_then(|scene| scene.t6_lut.clone()),
    };
    if let Some(mut existing) = existing {
        *existing = next;
    } else {
        commands.insert_resource(next);
    }
    if let Some(slot) = slot {
        slot.stamp_extract_diag(started.elapsed().as_secs_f32() * 1000.0);
    }
}

pub fn extract_model_lighting_tiles(
    mut main_world: ResMut<bevy::render::MainWorld>,
    mut uploads: ResMut<render_gpu::ModelLightingTileUploads>,
) {
    let pending = main_world
        .get_resource::<render_scene::ModelLightingAtlasTileWrites>()
        .is_some_and(|writes| !writes.tiles.is_empty());
    if !pending {
        return;
    }
    let mut writes = main_world.resource_mut::<render_scene::ModelLightingAtlasTileWrites>();
    uploads.tiles.extend(
        writes
            .tiles
            .drain(..)
            .map(|tile| render_gpu::ModelLightingTileUpload {
                image: tile.image,
                origin: tile.origin,
                texels: tile.texels,
            }),
    );
}

/// bo2zm: one skinned IW4 packed vertex (what the first-person rig writes)
/// as the fallback pass's model vertex: position, normal (biased bytes),
/// colour (B,G,R,A bytes), texcoord (half2, u in the high half).
fn t6_packed_to_vertex(row: &[u8; asset_iw4::size::GFX_PACKED_VERTEX]) -> render_frame::SmodelVertex {
    let f = |at: usize| f32::from_le_bytes([row[at], row[at + 1], row[at + 2], row[at + 3]]);
    let u32_at = |at: usize| u32::from_le_bytes([row[at], row[at + 1], row[at + 2], row[at + 3]]);
    let tex = u32_at(20);
    render_frame::SmodelVertex {
        position: [f(0), f(4), f(8)],
        normal: asset_model::normalize_or_up(asset_model::unpack_unit_vec(u32_at(24))),
        color: [row[18], row[17], row[16], row[19]].map(|c| f32::from(c) / 255.0),
        uv0: [
            asset_model::half_to_f32((tex >> 16) as u16),
            asset_model::half_to_f32(tex as u16),
        ],
    }
}

/// `R_HashString("featherParms")`: an effect material's (1 / feather
/// distance, feather distance).
const T6_FEATHER_PARMS: u32 = 0x4d7e_a234;
/// bo2zm M3 fix list 1: `eyeOffsetParms` (its register slot in the `_eo`
/// effect techniques, from the t6shader dump): x = units the sprite is
/// drawn nearer the eye.
const T6_EYE_OFFSET_PARMS: u32 = 0x0812_c0a9;
/// bo2zm M3 fix list 1: the falloff effect techniques' `falloffParms` and
/// `falloffEndColor` (register slots from the t6shader dump): a sprite
/// fades as it turns edge-on to the eye.
const T6_FALLOFF_PARMS: u32 = 0xbdde_5cf5;
const T6_FALLOFF_END_COLOR: u32 = 0x6b1d_a6fa;
/// bo2zm M4 retest 5: `alphaDissolveParms` (register slot from the t6shader
/// dump of the `_ds` blood effects): their alpha is saturate(x * (texture
/// alpha + vertex alpha - 1)), the texture's alpha a dissolve order, not a
/// coverage (multiplied, its faint floor drew each sprite's whole square).
const T6_ALPHA_DISSOLVE_PARMS: u32 = 0x8c1d_2f74;

/// bo2zm: an effect material drawn with no colour map (white): logged once.
fn t6_note_untextured(material: u32, name: &str) {
    static SEEN: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());
    if let Ok(mut seen) = SEEN.lock()
        && !seen.contains(&material)
    {
        seen.push(material);
        diag::warn!(World, "bo2zm fx: material `{name}` ({material}) has no colour map");
    }
}

/// bo2zm M3: the shapes of things that hold their shape, kept across
/// frames: still script models, grenades and rockets in flight, and (bo2mp)
/// guns on the ground and corpses at rest. Each new shape goes up once, in
/// a chunk of its own; a frame then sends one placement row per model.
/// Re-sending them every frame grew with each gun dropped and each body
/// that fell, to ~270,000 vertices and ~12 ms a frame late in a match.
#[derive(Default)]
pub struct T6StaticCache {
    generation: Option<u64>,
    /// Bumped whenever the kept shapes are thrown out and gathered again.
    epoch: u64,
    frame: u64,
    vertex_count: u32,
    index_count: u32,
    chunks: Vec<Arc<(Vec<render_frame::SmodelVertex>, Vec<u32>)>>,
    models: std::collections::HashMap<T6StaticKey, T6Kept>,
    /// Last frame's skinned poses, by address (held, so a new pose cannot
    /// take an old one's address): one handed on again holds still.
    posed_before: std::collections::HashMap<usize, Arc<render_anim::occupancy::missile::PosedVerts>>,
}

/// A kept shape: its surfaces (first index, count, colour slot, draw code,
/// normal map slot, specular map slot, eyes; see `t6_surface_rows`), its
/// vertex count, the frame it was last drawn, and the pose it came from
/// (held, so its address names it).
struct T6Kept {
    surfaces: Vec<[u32; 7]>,
    vertices: u32,
    used: u64,
    _posed: Option<Arc<render_anim::occupancy::missile::PosedVerts>>,
}

/// A kept shape: its name, materials, (bo2mp) hidden parts and, when it was
/// skinned, its pose's address.
type T6StaticKey = (String, Vec<Option<u32>>, Option<([u32; 6], u32)>, usize);

/// Kept vertices past which the shapes nothing draws any more (guns picked
/// up, corpses gone) are thrown out, once they are half of them.
const T6_KEPT_VERTICES: u32 = 400_000;

/// bo2mp: how far a posed model may reach past its bind bounds (a player
/// lying dead or prone), in game units.
const T6_POSE_MARGIN: f32 = 48.0;

/// bo2mp: how near the eye a model still casts sun shadow into view, while
/// the sun shadow is drawn (its far partition is at most 4096 across).
const T6_SHADOW_REACH: f32 = 4096.0;

/// bo2mp: the camera's view, for leaving out the models it cannot see (a
/// live player is ~20k vertices copied and drawn a frame): its near and
/// side planes and, while the sun shadow is drawn, how near a model out
/// of view still casts into it.
struct T6ViewCull {
    planes: Vec<[f32; 4]>,
    eye: Vec3,
    shadow_reach: Option<f32>,
}

impl T6ViewCull {
    fn new(view: Option<&render_scene::PreparedSceneView>) -> Self {
        let planes = match view {
            // Near, far, left, right, bottom, top: the far plane is left
            // out (a view without one puts it at the eye).
            Some(v) if v.ready && v.frustum_planes.len() == 6 => {
                let p = &v.frustum_planes;
                vec![p[0], p[2], p[3], p[4], p[5]]
            }
            _ => Vec::new(),
        };
        Self {
            planes,
            eye: view.map_or(Vec3::ZERO, |v| v.eye),
            shadow_reach: (!asset_core::t6_fast()).then_some(T6_SHADOW_REACH),
        }
    }

    fn sees(&self, item: &render_anim::occupancy::missile::T6MissileModel) -> bool {
        if self.planes.is_empty() {
            return true;
        }
        let skel = &item.skel;
        let (centre, half) = match (skel.bounds, skel.radius) {
            (Some((lo, hi)), _) => {
                let (lo, hi) = (Vec3::from(lo), Vec3::from(hi));
                ((lo + hi) * 0.5, (hi - lo).length() * 0.5)
            }
            (None, Some(r)) => (Vec3::ZERO, r),
            _ => return true,
        };
        let m = item.world_from_local;
        let scale = m.x_axis.truncate().length().max(m.y_axis.truncate().length()).max(m.z_axis.truncate().length());
        let centre = m.transform_point3(centre);
        let radius = half * scale + T6_POSE_MARGIN;
        if self.shadow_reach.is_some_and(|reach| centre.distance(self.eye) < reach + radius) {
            return true;
        }
        !self
            .planes
            .iter()
            .any(|p| p[0] * centre.x + p[1] * centre.y + p[2] * centre.z + p[3] + radius <= 0.0)
    }
}

/// One drawn surface of a model: its vertices, its triangles (all of the
/// surface's, or those its hidden parts leave), colour slot and draw code,
/// (bo2zm) its shine maps and whether it is a zombie's eyes.
struct T6Surface {
    vbase: usize,
    vn: usize,
    triangles: T6Triangles,
    slot: u32,
    code: u32,
    /// bo2zm: its normal and specular map slots (u32::MAX without): BO2's
    /// gold power-ups, perk machines and the box gloss and reflect as the
    /// gun in his hands does.
    normal: u32,
    specular: u32,
    eyes: bool,
}

enum T6Triangles {
    /// The surface's own index range (first, count).
    All(usize, usize),
    Left(Vec<u32>),
}

impl T6Surface {
    fn index_count(&self) -> usize {
        match &self.triangles {
            T6Triangles::All(_, count) => *count,
            T6Triangles::Left(left) => left.len(),
        }
    }
}

/// A model's drawn surfaces (with a material that draws, in range, not
/// all hidden).
fn t6_model_surfaces(
    item: &render_anim::occupancy::missile::T6MissileModel,
    catalog: &render_frontend::assemble::drawsurf::RuntimeMaterialCatalog,
) -> Vec<T6Surface> {
    let skel = &item.skel;
    let mut out = Vec::new();
    // bo2mp diag: the surfaces left out, by cause (see `t6_note_refused`).
    let mut refused: Vec<(usize, &'static str, String)> = Vec::new();
    let ranges = skel.surface_vertex_ranges.iter().zip(&skel.surface_index_ranges);
    for (surface, (&(vbase, vn), &(ibase, icount))) in ranges.enumerate() {
        let Some(material) = item.materials.get(surface).copied().flatten() else {
            refused.push((surface, "no material bound", String::new()));
            continue;
        };
        let Some(m) = catalog.derived(assets::MaterialIndex::from_order(material as usize)) else {
            refused.push((surface, "material not in the scene catalog", format!("#{material}")));
            continue;
        };
        let Some(draw) = m.t6_model_draw(assets::t6_rules()) else {
            refused.push((surface, "material has no BO2 draw (not linked by a T6 lane, not an mc_ model material)", m.name.to_string()));
            continue;
        };
        if vbase + vn > skel.positions.len() || ibase + icount > skel.indices.len() {
            refused.push((surface, "surface ranges past the mesh", m.name.to_string()));
            continue;
        }
        // bo2mp: less its hidden parts (a mannequin's shot-off head).
        let triangles = match item.drawn_indices(surface, &skel.indices[ibase..ibase + icount]) {
            None => continue,
            Some(std::borrow::Cow::Borrowed(all)) if !all.is_empty() => T6Triangles::All(ibase, icount),
            Some(std::borrow::Cow::Owned(left)) if !left.is_empty() => T6Triangles::Left(left),
            Some(_) => continue,
        };
        let slot = m
            .texture_semantic(asset_material::TS_COLOR_MAP)
            .map_or(u32::MAX, |id| id.0);
        if slot == u32::MAX {
            refused.push((surface, "no colour map (draw skipped by the pass)", m.name.to_string()));
        } else if draw.blend == asset_core::T6Blend::ShadowOnly {
            refused.push((surface, "shadow-only material (never drawn in colour)", m.name.to_string()));
        }
        let shine = |semantic| m.texture_semantic(semantic).map_or(u32::MAX, |id| id.0);
        out.push(T6Surface {
            vbase,
            vn,
            triangles,
            slot,
            code: draw.code(),
            normal: shine(asset_material::TS_NORMAL_MAP),
            specular: shine(asset_material::TS_SPECULAR_MAP),
            eyes: draw.eyes,
        });
    }
    if !refused.is_empty() {
        t6_note_refused(&skel.name, skel.surface_vertex_ranges.len(), &refused);
    }
    out
}

/// bo2mp diag: a model (a player's body or head, a gun) with surfaces the
/// Black Ops II draw leaves out, once per model and cause set (the first
/// 256 models): the surface, why, and its material.
fn t6_note_refused(model: &str, surfaces: usize, refused: &[(usize, &'static str, String)]) {
    use std::collections::HashSet;
    use std::sync::Mutex;
    static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let causes: Vec<&str> = refused.iter().map(|r| r.1).collect();
    let key = format!("{model}|{causes:?}");
    let Ok(mut seen) = SEEN.lock() else { return };
    let seen = seen.get_or_insert_with(HashSet::new);
    if seen.len() >= 256 || !seen.insert(key) {
        return;
    }
    let list: Vec<String> = refused
        .iter()
        .take(12)
        .map(|(s, why, name)| {
            if name.is_empty() {
                format!("#{s} {why}")
            } else {
                format!("#{s} {why} [{name}]")
            }
        })
        .collect();
    diag::info!(
        World,
        "bo2 model surfaces refused: {model} {} of {surfaces}: {}",
        refused.len(),
        list.join("; ")
    );
}

/// Per surface (first index, count, colour slot, draw code, normal map
/// slot, specular map slot, 1 for a zombie's eyes), the first index
/// counted from `first`.
fn t6_surface_rows(surfaces: &[T6Surface], mut first: u32) -> Vec<[u32; 7]> {
    surfaces
        .iter()
        .map(|s| {
            let count = s.index_count() as u32;
            let row = [first, count, s.slot, s.code, s.normal, s.specular, u32::from(s.eyes)];
            first += count;
            row
        })
        .collect()
}

/// A model's surfaces' vertices (from its skinned pose when it has one)
/// and indices appended; the indices count from `first_vertex` plus where
/// its vertices land in `vertices`.
fn t6_fill_model(
    skel: &asset_model::ModelSkel,
    posed: Option<&render_anim::occupancy::missile::PosedVerts>,
    surfaces: &[T6Surface],
    vertices: &mut Vec<render_frame::SmodelVertex>,
    mut indices: Option<&mut Vec<u32>>,
    first_vertex: u32,
) {
    // bo2mp: a pose left to the graphics card is skinned here instead (a
    // corpse at rest, held from now on).
    let posed = posed.map(|p| p.on_cpu(skel));
    let (positions, normals) = match posed.as_deref() {
        Some(posed) => (posed.positions.as_slice(), posed.normals.as_slice()),
        None => (skel.positions.as_slice(), skel.normals.as_slice()),
    };
    for s in surfaces {
        let base = first_vertex + vertices.len() as u32;
        let range = s.vbase..s.vbase + s.vn;
        match (
            positions.get(range.clone()),
            normals.get(range.clone()),
            skel.colors.get(range.clone()),
            skel.uvs.get(range.clone()),
        ) {
            // Every list has the surface's vertices: a straight copy (a
            // live player's ~20k vertices every frame).
            (Some(p), Some(n), Some(c), Some(u)) => vertices.extend(p.iter().zip(n).zip(c).zip(u).map(
                |(((&position, &normal), &color), &uv0)| render_frame::SmodelVertex {
                    position,
                    normal,
                    color,
                    uv0,
                },
            )),
            _ => vertices.extend(range.map(|v| render_frame::SmodelVertex {
                position: positions.get(v).copied().unwrap_or(skel.positions[v]),
                normal: normals.get(v).copied().unwrap_or([0.0, 0.0, 1.0]),
                color: skel.colors.get(v).copied().unwrap_or([1.0; 4]),
                uv0: skel.uvs.get(v).copied().unwrap_or([0.0; 2]),
            })),
        }
        if let Some(indices) = indices.as_deref_mut() {
            t6_fill_indices(skel, s, base, indices);
        }
    }
}

/// A surface's indices appended, counting from `base` (where its first
/// vertex lands).
fn t6_fill_indices(skel: &asset_model::ModelSkel, s: &T6Surface, base: u32, indices: &mut Vec<u32>) {
    let vbase = s.vbase as u32;
    match &s.triangles {
        T6Triangles::All(first, count) => indices.extend(
            skel.indices[*first..*first + *count]
                .iter()
                .map(|&i| base + i.saturating_sub(vbase)),
        ),
        T6Triangles::Left(left) => indices.extend(left.iter().map(|&i| base + i.saturating_sub(vbase))),
    }
}

/// bo2mp: a model's shape as the graphics card skins it: every vertex in
/// its bind pose, and its four bones and weights (a vertex with none moves
/// with bone 0, as `skin_verts` does).
fn t6_skin_shape(skel: &Arc<asset_model::ModelSkel>) -> render_gpu::T6SkinShape {
    let source: Arc<dyn std::any::Any + Send + Sync> = skel.clone();
    let vertex_total = skel.positions.len();
    let skel = Arc::clone(skel);
    let fill = move |vertices: &mut Vec<render_frame::SmodelVertex>, weights: &mut Vec<[u32; 8]>| {
        const RIGID: [u32; 8] = [0, 0, 0, 0, 0x3f80_0000, 0, 0, 0];
        let n = skel.positions.len();
        let blended = skel.vert_skin.len() == n;
        vertices.extend((0..n).map(|v| render_frame::SmodelVertex {
            position: skel.positions[v],
            normal: skel.normals.get(v).copied().unwrap_or([0.0, 0.0, 1.0]),
            color: skel.colors.get(v).copied().unwrap_or([1.0; 4]),
            uv0: skel.uvs.get(v).copied().unwrap_or([0.0; 2]),
        }));
        weights.extend((0..n).map(|v| {
            let Some(s) = skel.vert_skin.get(v).filter(|_| blended) else {
                return RIGID;
            };
            let w = s.weights.map(|w| if w > 0.0 { w } else { 0.0 });
            if w.iter().sum::<f32>() <= 0.0 {
                return RIGID;
            }
            let b = s.bones.map(u32::from);
            [b[0], b[1], b[2], b[3], w[0].to_bits(), w[1].to_bits(), w[2].to_bits(), w[3].to_bits()]
        }));
    };
    render_gpu::T6SkinShape {
        source,
        vertices: vertex_total,
        fill: Arc::new(fill),
    }
}

/// A draw list's vertices, written now or on the render thread.
fn t6_vertex_count(draw: &render_gpu::T6DynamicDraw) -> usize {
    draw.deferred.as_ref().map_or(draw.vertices.len(), |d| d.vertices)
        + draw.skinned.as_ref().map_or(0, |s| s.vertices)
}

/// One model's drawn surfaces appended to a geometry whose first vertex
/// and index are `offset` into the whole: per surface (first index, count,
/// colour slot, draw code, ...: `t6_surface_rows`); vertices from its skinned
/// pose when it has one.
fn t6_append_model(
    item: &render_anim::occupancy::missile::T6MissileModel,
    catalog: &render_frontend::assemble::drawsurf::RuntimeMaterialCatalog,
    vertices: &mut Vec<render_frame::SmodelVertex>,
    indices: &mut Vec<u32>,
    offset: (u32, u32),
) -> Vec<[u32; 7]> {
    let surfaces = t6_model_surfaces(item, catalog);
    let rows = t6_surface_rows(&surfaces, offset.1 + indices.len() as u32);
    t6_fill_model(&item.skel, item.posed.as_deref(), &surfaces, vertices, Some(indices), offset.0);
    rows
}

/// IW4L_T6_LITLOG=1: an animated model whose light reads (near) black,
/// once a second: where it stands.
fn t6_note_dark(item: &render_anim::occupancy::missile::T6MissileModel, lit: &asset_world::T6PointLight) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var_os("IW4L_T6_LITLOG").is_some()) || lit.light.iter().sum::<f32>() >= 0.02 {
        return;
    }
    static LAST: std::sync::Mutex<u64> = std::sync::Mutex::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    if let Ok(mut last) = LAST.lock()
        && *last != now
    {
        *last = now;
        diag::info!(
            World,
            "bo2zm dark model {} at {:?}: light {:?} primary {} vis {:.2}",
            item.skel.name,
            item.light_at.map(|v| v.round()),
            lit.light,
            lit.primary,
            lit.visibility
        );
    }
}

/// bo2zm: Black Ops II things that move, every frame: the first-person gun
/// and arms (skinned by the rig into the FPV draw plan), lit by the light
/// grid where they are; and every T6 material's colour slot and draw code.
pub fn extract_t6_dynamic(
    mut commands: Commands,
    fpv: Extract<Option<Res<render_anim::FpvDrawPlan>>>,
    t6_fx: Extract<Option<Res<render_frontend::adapters::fx::system::T6FxMesh>>>,
    t6_marks: Extract<Option<Res<render_frontend::adapters::fx::system::T6MarkMesh>>>,
    t6_missiles: Extract<Option<Res<render_anim::occupancy::missile::T6MissileModels>>>,
    t6_script_models: Extract<Option<Res<render_anim::occupancy::missile::T6ScriptModels>>>,
    t6_bodies: Extract<Option<Res<render_anim::occupancy::t6_body::T6BodyModels>>>,
    t6_lights: Extract<Option<Res<render_frontend::adapters::fx::system::T6DynLights>>>,
    runtime: Extract<Option<Res<render_frontend::assemble::drawsurf::MaterialGeneration>>>,
    exposure: Extract<Option<Res<render_frontend::assemble::drawsurf::MapT5SunParseExposure>>>,
    scene_view: Extract<Option<Res<render_scene::PreparedSceneView>>>,
    existing: Option<ResMut<render_gpu::ExtractedT6Dynamic>>,
    mut static_cache: Local<T6StaticCache>,
) {
    let t0 = Instant::now();
    let Some(runtime) = runtime.as_ref() else {
        return;
    };
    let catalog = &runtime.catalog;
    // Only the material table carries over; every draw list is rebuilt
    // below (cloning last frame's vertex lists cost as much as building
    // them).
    let mut out = match existing {
        Some(existing) => {
            render_gpu::ExtractedT6Dynamic {
                generation: existing.generation,
                materials: Arc::clone(&existing.materials),
                ..Default::default()
            }
        }
        None => render_gpu::ExtractedT6Dynamic::default(),
    };
    // bo2mp: other games' guns are extra BO2 weapons under BO2's rules.
    let foreign = assets::t6_rules();
    if out.generation != catalog.generation_id || out.materials.is_empty() {
        let materials: Vec<(u32, u32)> = catalog
            .materials
            .iter()
            .filter_map(|m| {
                // bo2mp: and other games' model materials under BO2's rules
                // (their guns), for their colour maps' bind groups and
                // their draw codes' pipelines.
                let draw = m.t6_model_draw(foreign)?;
                let slot = m.texture_semantic(asset_material::TS_COLOR_MAP)?.0;
                Some((slot, draw.code()))
            })
            .collect();
        out.materials = Arc::new(materials);
        out.generation = catalog.generation_id;
    }
    let exposure = exposure.as_ref().map_or(1.0, |e| e.exposure);
    // bo2zm: what the gun extraction sees, on change.
    {
        use std::sync::Mutex;
        static LAST: Mutex<Option<String>> = Mutex::new(None);
        let line = match fpv.as_ref() {
            None => "no FPV plan".to_owned(),
            Some(plan) => format!(
                "plan visible={} geometry_ok={} placement_ok={} drawgun={:?} rows={} draws={} t6_materials={}",
                plan.visible,
                plan.geometry_ok,
                plan.placement_ok,
                plan.drawgun,
                plan.fallback_rows().map_or(0, |r| r.rows.len()),
                plan.fallback_rows().map_or(0, |r| r.draws.len()),
                plan.fallback_rows().map_or(0, |r| r.draws.iter().filter(|d| d.2.is_some()).count()),
            ),
        };
        if let Ok(mut last) = LAST.lock()
            && last.as_deref() != Some(line.as_str())
        {
            diag::info!(Fpv, "bo2zm viewmodel extract: {line}");
            *last = Some(line);
        }
    }
    out.viewmodel = fpv
        .as_ref()
        // The engine's own visibility also needs its MW2-style eye
        // lighting, which a BO2 map does not have; the fallback draw lights
        // the gun itself, so built + placed + not hidden is enough.
        .filter(|plan| {
            plan.geometry_ok
                && plan.placement_ok
                && plan.drawgun != Some(0)
                && !out.materials.is_empty()
        })
        .and_then(|plan| {
            let rows = plan.fallback_rows()?;
            let vertices = rows.rows.iter().map(t6_packed_to_vertex).collect();
            let draws = rows
                .draws
                .iter()
                .filter_map(|&(start, count, material)| {
                    let m = catalog.derived(assets::MaterialIndex::from_order(material? as usize))?;
                    let draw = m.t6_model_draw(foreign)?;
                    let slot_of = |semantic| m.texture_semantic(semantic).map_or(u32::MAX, |id| id.0);
                    // bo2mp: another game's gun draws without shine: its
                    // normal maps (two-channel, 2x-1) and specular maps
                    // are not Black Ops II's encoding, which the shine
                    // shader reads; grid light and the sun light it.
                    let own = m.t6_draw.is_some();
                    let shine_of = |semantic| if own { slot_of(semantic) } else { u32::MAX };
                    Some([
                        start,
                        count,
                        slot_of(asset_material::TS_COLOR_MAP),
                        draw.code(),
                        0,
                        shine_of(asset_material::TS_NORMAL_MAP),
                        shine_of(asset_material::TS_SPECULAR_MAP),
                    ])
                })
                .collect();
            let eye = plan.world_from_local.w_axis.truncate().to_array();
            let lit = asset_world::t6_light_sampler()
                .map(|sampler| sampler.at(eye))
                .unwrap_or_default();
            let mut instance = [0.0f32; 24];
            instance[..16].copy_from_slice(&plan.world_from_local.to_cols_array());
            for c in 0..3 {
                instance[16 + c] = lit.light[c] * exposure;
            }
            instance[20] = f32::from(lit.primary);
            instance[21] = lit.visibility;
            // The gun reflects the probe nearest the eye (plus one; 0 = none).
            instance[23] = asset_world::t6_nearest_probe(eye).map_or(0.0, |p| p as f32 + 1.0);
            Some(render_gpu::T6DynamicDraw {
                vertices,
                indices: rows.indices.to_vec(),
                draws,
                instances: vec![instance],
                deferred: None,
                skinned: None,
            })
        });
    // Effect sprites: world space, lit (when their material is) by the
    // grid at the eye.
    let eye_light = asset_world::t6_light_sampler()
        .zip(fpv.as_ref())
        .map(|(sampler, plan)| sampler.at(plan.world_from_local.w_axis.truncate().to_array()))
        .unwrap_or_default();
    // Each effect draw gets its own instance row: the eye's light, and in
    // slot 22 the material's `featherParms.x` (1 / its feather distance):
    // BO2's effect shaders fade a sprite out as it nears the camera; in
    // slot 23 its `eyeOffsetParms.x` (drawn that much nearer the eye).
    let fx_draw = |list: &Vec<(u32, u32, u32)>,
                   clouds: &[render_frontend::adapters::fx::system::T6CloudDraw],
                   mesh: &render_frontend::adapters::fx::system::T6FxMesh| {
        let mut instances: Vec<[f32; 24]> = Vec::with_capacity(list.len() + clouds.len());
        let mut draws: Vec<[u32; 7]> = list
            .iter()
            .filter_map(|&(start, count, material)| {
                let m = catalog.derived(assets::MaterialIndex::from_order(material as usize))?;
                let draw = m.t6_draw?;
                let slot = m
                    .texture_semantic(asset_material::TS_COLOR_MAP)
                    .map_or(u32::MAX, |id| id.0);
                if slot == u32::MAX {
                    t6_note_untextured(material, &m.name);
                }
                let feather = m
                    .constants
                    .iter()
                    .find(|(hash, _)| *hash == T6_FEATHER_PARMS)
                    .map_or(0.0, |(_, bits)| f32::from_bits(bits[0]));
                let eye_offset = m
                    .constants
                    .iter()
                    .find(|(hash, _)| *hash == T6_EYE_OFFSET_PARMS)
                    .map_or(0.0, |(_, bits)| f32::from_bits(bits[0]));
                let constant = |want: u32| {
                    m.constants
                        .iter()
                        .find(|(hash, _)| *hash == want)
                        .map(|(_, bits)| bits.map(f32::from_bits))
                };
                let dissolve = constant(T6_ALPHA_DISSOLVE_PARMS).map_or(0.0, |p| p[0]);
                let falloff = constant(T6_FALLOFF_PARMS).map(|p| {
                    let end = constant(T6_FALLOFF_END_COLOR).map_or(1.0, |c| c[0]);
                    [p[2], p[3], end]
                });
                // IW4L_T6_DRAWLOG_MAT=<material part>: that material's effect
                // draws as they leave for the graphics card.
                static WANT: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
                if let Some(want) = WANT.get_or_init(|| std::env::var("IW4L_T6_DRAWLOG_MAT").ok())
                    && m.name.contains(want.as_str())
                {
                    diag::info!(
                        World,
                        "bo2zm fx draw {}: first {start} count {count} slot {slot} code {:#x}",
                        m.name,
                        draw.code()
                    );
                }
                let mut instance = [0.0f32; 24];
                instance[..16].copy_from_slice(&Mat4::IDENTITY.to_cols_array());
                for c in 0..3 {
                    instance[16 + c] = eye_light.light[c] * exposure;
                }
                // A falloff material: slots 16..18 carry falloffParms.z, .w
                // and the falloff end colour (its begin colour is white in
                // every one), slot 19 says so (the effect shaders read no
                // grid light).
                if let Some(f) = falloff {
                    instance[16..19].copy_from_slice(&f);
                    instance[19] = 1.0;
                }
                // Slot 20: the primary light (0 or more), or for a dissolve
                // material (effect shaders, which read no primary light)
                // minus its alphaDissolveParms.x.
                instance[20] = if dissolve > 0.0 {
                    -dissolve
                } else {
                    f32::from(eye_light.primary)
                };
                instance[21] = eye_light.visibility;
                instance[22] = feather;
                instance[23] = eye_offset;
                // bo2zm M4 retest 5: a projected decal (a hit's blood on a
                // zombie): its box from the world in the columns, the grid's
                // light at the box.
                if draw.projected_decal {
                    let Some((box_from_world, at)) = mesh.model_boxes.get(&start) else {
                        return None;
                    };
                    instance[..16].copy_from_slice(box_from_world);
                    let lit = asset_world::t6_light_sampler()
                        .map(|sampler| sampler.at(*at))
                        .unwrap_or_default();
                    for c in 0..3 {
                        instance[16 + c] = lit.light[c] * exposure;
                    }
                    instance[19] = 0.0;
                }
                instances.push(instance);
                Some([start, count, slot, draw.code(), instances.len() as u32 - 1, u32::MAX, u32::MAX])
            })
            .collect();
        // Particle clouds: the box's placement, then (slots 16..19) the
        // colour, halved as BO2's cloud shaders halve it, and (20, 21) the
        // particle's width and height.
        for cloud in clouds {
            let Some(m) = catalog.derived(assets::MaterialIndex::from_order(cloud.material as usize)) else {
                continue;
            };
            let Some(draw) = m.t6_draw.filter(|d| d.cloud) else {
                continue;
            };
            let slot = m
                .texture_semantic(asset_material::TS_COLOR_MAP)
                .map_or(u32::MAX, |id| id.0);
            let mut instance = [0.0f32; 24];
            instance[..16].copy_from_slice(&cloud.world_from_local);
            for c in 0..3 {
                instance[16 + c] = cloud.color[c] * 0.5;
            }
            instance[19] = cloud.color[3];
            instance[20] = cloud.size[0];
            instance[21] = cloud.size[1];
            instances.push(instance);
            draws.push([
                cloud.first_index,
                cloud.index_count,
                slot,
                draw.code(),
                instances.len() as u32 - 1,
                u32::MAX,
                u32::MAX,
            ]);
        }
        render_gpu::T6DynamicDraw {
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
            draws,
            instances,
            deferred: None,
            skinned: None,
        }
    };
    // Marks: one instance per mark, lit by the grid where it landed.
    let sampler = asset_world::t6_light_sampler();
    out.marks = t6_marks.as_ref().filter(|mesh| !mesh.draws.is_empty()).map(|mesh| {
        let mut instances = Vec::with_capacity(mesh.draws.len());
        let draws = mesh
            .draws
            .iter()
            .filter_map(|&(start, count, material, at, lightmap)| {
                let m = catalog.derived(assets::MaterialIndex::from_order(material as usize))?;
                let draw = m.t6_draw?;
                let slot = m
                    .texture_semantic(asset_material::TS_COLOR_MAP)
                    .map_or(u32::MAX, |id| id.0);
                let mut row = [0.0f32; 24];
                row[..16].copy_from_slice(&Mat4::IDENTITY.to_cols_array());
                if let Some((primary, visibility)) = lightmap {
                    // bo2zm fix list 2: lit by its wall's lightmap; the
                    // vertex colours hold it (w > 0 = baked light).
                    row[19] = exposure.max(0.000001);
                    row[20] = f32::from(primary);
                    row[21] = visibility;
                } else {
                    let lit = sampler.as_ref().map(|s| s.at(at)).unwrap_or_default();
                    for c in 0..3 {
                        row[16 + c] = lit.light[c] * exposure;
                    }
                    row[20] = f32::from(lit.primary);
                    row[21] = lit.visibility;
                }
                // bo2mp: a mark lit like the one before it shares its row
                // (and, its indices following on, its draw: ~256 a frame).
                if instances.last() != Some(&row) {
                    instances.push(row);
                }
                Some([start, count, slot, draw.code(), instances.len() as u32 - 1, u32::MAX, u32::MAX])
            })
            .collect();
        render_gpu::T6DynamicDraw {
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
            draws,
            instances,
            deferred: None,
            skinned: None,
        }
    });
    out.dlights = t6_lights.as_ref().map(|l| l.lights.clone()).unwrap_or_default();
    // Projectiles in flight: each model in its own space, its instance
    // row its placement and the grid's light where it is.
    // bo2zm M3: the map's script models (perk machines, the box, ...) draw
    // the same way, and (bo2mp) other players, corpses, guns and bags on
    // the ground.
    // A model that holds its shape (no pose, or the same pose handed on
    // again: a gun on the ground, a corpse at rest) is drawn from shapes
    // uploaded once; only what was skinned anew this frame is sent whole.
    let all = t6_missiles
        .as_ref()
        .map(|m| m.items.iter())
        .into_iter()
        .flatten()
        .chain(t6_script_models.as_ref().map(|m| m.items.iter()).into_iter().flatten())
        .chain(t6_bodies.as_ref().map(|m| m.items.iter()).into_iter().flatten());
    let mut moving: Vec<&render_anim::occupancy::missile::T6MissileModel> = Vec::new();
    let view_cull = T6ViewCull::new(scene_view.as_deref());
    let mut unseen = 0usize;
    let held_models: usize;
    {
        let cache = &mut *static_cache;
        let generation = catalog.generation_id.0;
        if cache.generation != Some(generation) {
            *cache = T6StaticCache {
                generation: Some(generation),
                epoch: cache.epoch + 1,
                ..Default::default()
            };
        }
        cache.frame += 1;
        let frame = cache.frame;
        if cache.vertex_count > T6_KEPT_VERTICES {
            let drawn: u32 = cache
                .models
                .values()
                .filter(|m| m.used + 1 >= frame)
                .map(|m| m.vertices)
                .sum();
            if drawn * 2 < cache.vertex_count {
                *cache = T6StaticCache {
                    generation: cache.generation,
                    epoch: cache.epoch + 1,
                    frame,
                    posed_before: std::mem::take(&mut cache.posed_before),
                    ..Default::default()
                };
            }
        }
        let mut posed_now = std::collections::HashMap::new();
        let mut new_vertices: Vec<render_frame::SmodelVertex> = Vec::new();
        let mut new_indices: Vec<u32> = Vec::new();
        let mut draws: Vec<[u32; 7]> = Vec::new();
        let mut instances: Vec<[f32; 24]> = Vec::new();
        for item in all {
            // Out of view: not copied or drawn (its pose is still noted, so
            // a corpse at rest stays held).
            let seen = view_cull.sees(item);
            unseen += usize::from(!seen);
            let pose = match &item.posed {
                None => 0,
                Some(posed) => {
                    let at = Arc::as_ptr(posed) as usize;
                    let held = cache.posed_before.contains_key(&at);
                    posed_now.insert(at, Arc::clone(posed));
                    if !held {
                        if seen {
                            moving.push(item);
                        }
                        continue;
                    }
                    at
                }
            };
            if !seen {
                continue;
            }
            let key = (item.skel.name.clone(), item.materials.clone(), item.hide, pose);
            let kept = cache.models.entry(key).or_insert_with(|| {
                // Where this frame's new chunk starts; the append counts on
                // from what the chunk holds already.
                let offset = (cache.vertex_count, cache.index_count);
                let before = new_vertices.len();
                let surfaces = t6_append_model(item, catalog, &mut new_vertices, &mut new_indices, offset);
                T6Kept {
                    surfaces,
                    vertices: (new_vertices.len() - before) as u32,
                    used: frame,
                    _posed: item.posed.clone(),
                }
            });
            kept.used = frame;
            if kept.surfaces.is_empty() {
                continue;
            }
            let lit = sampler.as_ref().map(|s| s.at(item.light_at)).unwrap_or_default();
            if item.posed.is_some() {
                t6_note_dark(item, &lit);
            }
            let mut row = [0.0f32; 24];
            row[..16].copy_from_slice(&item.world_from_local.to_cols_array());
            for c in 0..3 {
                row[16 + c] = lit.light[c] * exposure;
            }
            row[20] = f32::from(lit.primary);
            row[21] = lit.visibility;
            // Its reflection probe (the nearest, plus one), for its shine.
            let at = item.world_from_local.w_axis.truncate().to_array();
            row[23] = asset_world::t6_nearest_probe(at).map_or(0.0, |p| p as f32 + 1.0);
            instances.push(row);
            let instance = instances.len() as u32 - 1;
            // bo2zm round 1: a zombie's eyes get their own row: its eye
            // brightness rides negated in slot 22, which the unlit shader
            // scales the eye texture by (nothing else makes it negative).
            let eye_instance = match item.eye_luminance {
                Some(luminance) => {
                    row[22] = -luminance;
                    instances.push(row);
                    instances.len() as u32 - 1
                }
                None => instance,
            };
            for s in &kept.surfaces {
                let at = if s[6] != 0 { eye_instance } else { instance };
                draws.push([s[0], s[1], s[2], s[3], at, s[4], s[5]]);
            }
        }
        cache.posed_before = posed_now;
        if !new_vertices.is_empty() {
            cache.vertex_count += new_vertices.len() as u32;
            cache.index_count += new_indices.len() as u32;
            cache.chunks.push(Arc::new((new_vertices, new_indices)));
        }
        held_models = instances.len();
        out.script_static = (!draws.is_empty()).then(|| render_gpu::T6StaticDraw {
            epoch: cache.epoch,
            chunks: cache.chunks.clone(),
            draws,
            instances,
        });
    }
    // The moving models' vertices (the live players' ~250k a frame) are
    // copied on the render thread: here only what is drawn where, and the
    // poses to copy from (shared, not copied). bo2mp: a pose that is only
    // bones is skinned on the graphics card, after the copied vertices.
    out.missiles = (!moving.is_empty()).then(|| {
        let mut draws = Vec::new();
        let mut instances = Vec::with_capacity(moving.len());
        let mut models = Vec::with_capacity(moving.len());
        let (mut vertex_count, mut index_count) = (0usize, 0usize);
        let mut skinned = render_gpu::T6Skinned::default();
        let mut shape_of = std::collections::HashMap::new();
        // What the indices are made from: the same as last frame's and
        // they are not made again.
        let mut key = std::hash::DefaultHasher::new();
        for item in moving.iter().copied() {
            let lit = sampler.as_ref().map(|s| s.at(item.light_at)).unwrap_or_default();
            t6_note_dark(item, &lit);
            let plan = t6_model_surfaces(item, catalog);
            if plan.is_empty() {
                continue;
            }
            let surfaces = t6_surface_rows(&plan, index_count as u32);
            let on_card = item.posed.as_ref().filter(|p| p.on_gpu());
            match on_card {
                Some(posed) => {
                    let shape = *shape_of.entry(Arc::as_ptr(&item.skel) as usize).or_insert_with(|| {
                        skinned.shapes.push(t6_skin_shape(&item.skel));
                        skinned.shapes.len() as u32 - 1
                    });
                    let bones = (skinned.bones.len() as u32, posed.bones.len() as u32);
                    skinned.bones.extend(posed.bones.iter().map(|m| m.to_cols_array()));
                    for s in &plan {
                        skinned.runs.push([shape, s.vbase as u32, s.vn as u32, bones.0, bones.1]);
                        skinned.vertices += s.vn;
                    }
                }
                None => vertex_count += plan.iter().map(|s| s.vn).sum::<usize>(),
            }
            index_count += plan.iter().map(T6Surface::index_count).sum::<usize>();
            {
                use std::hash::Hash;
                (Arc::as_ptr(&item.skel) as usize, on_card.is_some()).hash(&mut key);
                for s in &plan {
                    (s.vbase, s.vn).hash(&mut key);
                    match &s.triangles {
                        T6Triangles::All(first, count) => (0u8, first, count).hash(&mut key),
                        T6Triangles::Left(left) => (1u8, left).hash(&mut key),
                    }
                }
            }
            models.push((item.skel.clone(), item.posed.clone(), plan, on_card.is_some()));
            let mut row = [0.0f32; 24];
            row[..16].copy_from_slice(&item.world_from_local.to_cols_array());
            for c in 0..3 {
                row[16 + c] = lit.light[c] * exposure;
            }
            row[20] = f32::from(lit.primary);
            row[21] = lit.visibility;
            // Its reflection probe (the nearest, plus one), for its shine.
            let at = item.world_from_local.w_axis.truncate().to_array();
            row[23] = asset_world::t6_nearest_probe(at).map_or(0.0, |p| p as f32 + 1.0);
            instances.push(row);
            let instance = instances.len() as u32 - 1;
            // bo2zm round 1: a zombie's eyes get their own row: its eye
            // brightness rides negated in slot 22, which the unlit shader
            // scales the eye texture by (nothing else makes it negative).
            let eye_instance = match item.eye_luminance {
                Some(luminance) => {
                    row[22] = -luminance;
                    instances.push(row);
                    instances.len() as u32 - 1
                }
                None => instance,
            };
            for s in surfaces {
                let at = if s[6] != 0 { eye_instance } else { instance };
                draws.push([s[0], s[1], s[2], s[3], at, s[4], s[5]]);
            }
        }
        // The card-skinned models' vertices land after the copied ones;
        // their indices count from there.
        let copied = vertex_count as u32;
        let indices_key = {
            use std::hash::{Hash, Hasher};
            copied.hash(&mut key);
            key.finish().max(1)
        };
        let fill = move |vertices: &mut Vec<render_frame::SmodelVertex>, mut indices: Option<&mut Vec<u32>>| {
            let mut on_card = copied;
            for (skel, posed, plan, card) in &models {
                if !*card {
                    t6_fill_model(skel, posed.as_deref(), plan, vertices, indices.as_deref_mut(), 0);
                    continue;
                }
                let Some(indices) = indices.as_deref_mut() else {
                    continue;
                };
                for s in plan {
                    t6_fill_indices(skel, s, on_card, indices);
                    on_card += s.vn as u32;
                }
            }
        };
        render_gpu::T6DynamicDraw {
            vertices: Vec::new(),
            indices: Vec::new(),
            draws,
            instances,
            deferred: Some(render_gpu::T6DeferredFill {
                vertices: vertex_count,
                indices: index_count,
                indices_key,
                fill: Arc::new(fill),
            }),
            skinned: (skinned.vertices > 0).then_some(skinned),
        }
    });
    out.effects = t6_fx
        .as_ref()
        .filter(|mesh| !mesh.draws.is_empty() || !mesh.clouds.is_empty())
        .map(|mesh| fx_draw(&mesh.draws, &mesh.clouds, mesh));
    out.viewmodel_effects = t6_fx
        .as_ref()
        .filter(|mesh| !mesh.viewmodel_draws.is_empty())
        .map(|mesh| fx_draw(&mesh.viewmodel_draws, &[], mesh));
    // IW4L_T6_PROF=1: this extract's cost, every 300 frames, with what it
    // carried on the last of them.
    static PROF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *PROF.get_or_init(|| std::env::var_os("IW4L_T6_PROF").is_some()) {
        static ACC: std::sync::Mutex<(u32, f64, usize, usize)> = std::sync::Mutex::new((0, 0.0, 0, 0));
        if let Ok(mut a) = ACC.lock() {
            a.0 += 1;
            a.1 += t0.elapsed().as_secs_f64() * 1000.0;
            a.2 += out.missiles.as_ref().map_or(0, t6_vertex_count);
            a.3 += out.missiles.as_ref().and_then(|m| m.skinned.as_ref()).map_or(0, |s| s.vertices);
            if a.0 >= 300 {
                let list = |d: Option<&render_gpu::T6DynamicDraw>| {
                    d.map_or((0, 0), |d| (t6_vertex_count(d), d.draws.len()))
                };
                let kept = static_cache.vertex_count;
                diag::info!(
                    World,
                    "bo2zm t6 extract: {:.2} ms a frame, {} moving vertices a frame ({} skinned on the card); \
                     now {} moving models, \
                     {} held still, {} out of view ({} vertices kept in {} chunks), {} bodies and ground \
                     items in all; vertices/draws: moving {:?}, marks {:?}, effects {:?}",
                    a.1 / f64::from(a.0),
                    a.2 / a.0 as usize,
                    a.3 / a.0 as usize,
                    moving.len(),
                    held_models,
                    unseen,
                    kept,
                    static_cache.chunks.len(),
                    t6_bodies.as_ref().map_or(0, |b| b.items.len()),
                    list(out.missiles.as_ref()),
                    list(out.marks.as_ref()),
                    list(out.effects.as_ref()),
                );
                *a = (0, 0.0, 0, 0);
            }
        }
    }
    commands.insert_resource(out);
}
