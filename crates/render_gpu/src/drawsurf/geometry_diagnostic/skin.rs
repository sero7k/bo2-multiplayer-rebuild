//! bo2mp: live bodies skinned on the graphics card. Extract hands over each
//! body part's bone matrices and, per drawn surface, which of its shape's
//! vertices to skin with them; the shapes (bind pose and bone weights) are
//! sent once and kept here. A compute pass writes the skinned vertices into
//! the moving models' vertex buffer after the ones the CPU wrote, before
//! anything draws this frame (the sun shadow and the main pass both read
//! them). Skinning and copying ~150k vertices a frame on the CPU cost a
//! drawn window about 5 ms of game thread and 10 ms of render thread.

use super::*;
use bevy::render::renderer::PendingCommandBuffers;
use bevy::render::render_resource::{
    BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferDescriptor,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, PipelineLayoutDescriptor,
    RawComputePipelineDescriptor, ShaderModuleDescriptor, ShaderSource,
};

/// The shapes kept on the card at most (48 + 32 bytes each); past it they
/// start over with this frame's.
const KEPT_VERTICES: u32 = 1 << 20;
/// Frames with nothing to skin before the shapes are let go (a menu).
const IDLE_FRAMES: u32 = 600;
const WORKGROUP: u32 = 64;
const VERTEX_BYTES: u64 = std::mem::size_of::<SmodelVertex>() as u64;
const SKIN_BYTES: u64 = 32;

#[derive(Resource)]
pub(super) struct SkinPipeline {
    layout: BindGroupLayout,
    pipeline: ComputePipeline,
}

pub(super) fn init(mut commands: Commands, device: Res<RenderDevice>) {
    let storage = |binding, read_only| BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let layout = device.create_bind_group_layout(
        "bo2mp_skin",
        &[
            storage(0, true),
            storage(1, true),
            storage(2, true),
            storage(3, true),
            storage(4, false),
        ],
    );
    let module = device.create_and_validate_shader_module(ShaderModuleDescriptor {
        label: Some("bo2mp_skin"),
        source: ShaderSource::Wgsl(include_str!("skin.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("bo2mp_skin"),
        bind_group_layouts: &[Some(&*layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&RawComputePipelineDescriptor {
        label: Some("bo2mp_skin"),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("skin_main"),
        compilation_options: Default::default(),
        cache: None,
    });
    commands.insert_resource(SkinPipeline { layout, pipeline });
}

/// The shapes on the card: where each starts (in vertices), kept with what
/// names it; and this frame's bones and runs.
#[derive(Resource, Default)]
pub(super) struct SkinCache {
    shapes: HashMap<usize, (u32, Arc<dyn std::any::Any + Send + Sync>)>,
    used: u32,
    room: u32,
    shape: Option<Buffer>,
    skin: Option<Buffer>,
    bones: Option<Buffer>,
    runs: Option<Buffer>,
    vertices: Vec<SmodelVertex>,
    weights: Vec<[u32; 8]>,
    rows: Vec<[u32; 8]>,
    idle: u32,
}

impl SkinCache {
    /// A frame with nothing skinned; after a while the shapes go.
    pub(super) fn idle(&mut self) {
        self.idle = self.idle.saturating_add(1);
        if self.idle == IDLE_FRAMES {
            *self = Self {
                idle: self.idle,
                ..Default::default()
            };
        }
    }
}

/// `need` bytes of `usage` in `slot`, made (bigger) when it is short.
fn room(
    device: &RenderDevice,
    slot: &mut Option<Buffer>,
    label: &str,
    usage: BufferUsages,
    need: u64,
) -> Buffer {
    if slot.as_ref().is_none_or(|b| b.size() < need) {
        *slot = Some(device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: (need + need / 2).max(64 << 10).next_multiple_of(4),
            usage: usage | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }
    slot.clone().expect("made above")
}

/// This frame's card-skinned vertices written into `dst` from vertex
/// `first` on; false when they could not be (nothing is drawn from them).
/// The pass goes first in the frame's one submit (`pending`), ahead of the
/// draws that read it; a submit of its own cost the render thread ~0.5 ms.
#[allow(clippy::too_many_arguments)]
pub(super) fn skin_into(
    device: &RenderDevice,
    queue: &RenderQueue,
    pending: &mut PendingCommandBuffers,
    pipeline: &SkinPipeline,
    cache: &mut SkinCache,
    skinned: &crate::drawsurf::geometry_diagnostic::T6Skinned,
    dst: &Buffer,
    first: u32,
) -> bool {
    let _span = bevy::log::info_span!(target: "iw4l", "t6 skin").entered();
    cache.idle = 0;
    let key = |s: &crate::drawsurf::geometry_diagnostic::T6SkinShape| {
        Arc::as_ptr(&s.source) as *const () as usize
    };
    let missing = |cache: &SkinCache| -> u32 {
        let mut seen = HashSet::new();
        skinned
            .shapes
            .iter()
            .filter(|s| !cache.shapes.contains_key(&key(s)) && seen.insert(key(s)))
            .map(|s| s.vertices as u32)
            .sum()
    };
    let mut new = missing(cache);
    if cache.used + new > KEPT_VERTICES {
        cache.shapes.clear();
        cache.used = 0;
        new = missing(cache);
    }
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("bo2mp_skin"),
    });
    let need = cache.used + new;
    if need > cache.room || cache.shape.is_none() {
        let room = (need + need / 2).max(1 << 16);
        let usage = BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC;
        let make = |label, bytes: u64| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size: bytes,
                usage,
                mapped_at_creation: false,
            })
        };
        let shape = make("bo2mp_skin_shapes", u64::from(room) * VERTEX_BYTES);
        let skin = make("bo2mp_skin_weights", u64::from(room) * SKIN_BYTES);
        if cache.used > 0
            && let (Some(old_shape), Some(old_skin)) = (&cache.shape, &cache.skin)
        {
            let used = u64::from(cache.used);
            encoder.copy_buffer_to_buffer(old_shape, 0, &shape, 0, used * VERTEX_BYTES);
            encoder.copy_buffer_to_buffer(old_skin, 0, &skin, 0, used * SKIN_BYTES);
        }
        cache.shape = Some(shape);
        cache.skin = Some(skin);
        cache.room = room;
    }
    // (The copies above go to the card whatever happens below.)
    let ok = write_and_dispatch(
        device,
        queue,
        pipeline,
        cache,
        skinned,
        dst,
        first,
        &mut encoder,
    );
    // (Not `push_encoder`: those go after the frame's other buffers.)
    pending.push([encoder.finish()]);
    ok
}

/// The shapes the card lacks, this frame's bones and runs, and the pass
/// that skins them into `dst`.
#[allow(clippy::too_many_arguments)]
fn write_and_dispatch(
    device: &RenderDevice,
    queue: &RenderQueue,
    pipeline: &SkinPipeline,
    cache: &mut SkinCache,
    skinned: &crate::drawsurf::geometry_diagnostic::T6Skinned,
    dst: &Buffer,
    first: u32,
    encoder: &mut bevy::render::render_resource::CommandEncoder,
) -> bool {
    let key = |s: &crate::drawsurf::geometry_diagnostic::T6SkinShape| {
        Arc::as_ptr(&s.source) as *const () as usize
    };
    let (Some(shape_buffer), Some(skin_buffer)) = (cache.shape.clone(), cache.skin.clone()) else {
        return false;
    };
    // The shapes the card does not have yet, after the others.
    for s in &skinned.shapes {
        let k = key(s);
        if cache.shapes.contains_key(&k) {
            continue;
        }
        cache.vertices.clear();
        cache.weights.clear();
        (s.fill)(&mut cache.vertices, &mut cache.weights);
        let n = s.vertices;
        if cache.vertices.len() != n
            || cache.weights.len() != n
            || cache.used + n as u32 > cache.room
        {
            return false;
        }
        let at = u64::from(cache.used);
        queue.write_buffer(
            &shape_buffer,
            at * VERTEX_BYTES,
            bytemuck::cast_slice(&cache.vertices),
        );
        queue.write_buffer(
            &skin_buffer,
            at * SKIN_BYTES,
            bytemuck::cast_slice(&cache.weights),
        );
        cache.shapes.insert(k, (cache.used, Arc::clone(&s.source)));
        cache.used += n as u32;
    }
    // The runs: a header, then per run where it lands, where it reads
    // from, how many, and its bones.
    cache.rows.clear();
    cache.rows.push([0; 8]);
    let mut total = 0u32;
    for r in &skinned.runs {
        let Some(&(at, _)) = skinned
            .shapes
            .get(r[0] as usize)
            .and_then(|s| cache.shapes.get(&key(s)))
        else {
            return false;
        };
        if r[2] == 0 {
            continue;
        }
        cache
            .rows
            .push([total, at + r[1], r[2], r[3], r[4], 0, 0, 0]);
        total += r[2];
    }
    let runs = cache.rows.len() as u32 - 1;
    if total == 0 || total as usize != skinned.vertices || total.div_ceil(WORKGROUP) > 65535 {
        return false;
    }
    cache.rows[0] = [runs, total, 0, first, 0, 0, 0, 0];
    let bones_buffer = room(
        device,
        &mut cache.bones,
        "bo2mp_skin_bones",
        BufferUsages::STORAGE,
        (skinned.bones.len().max(1) * 64) as u64,
    );
    queue.write_buffer(&bones_buffer, 0, bytemuck::cast_slice(&skinned.bones));
    let runs_buffer = room(
        device,
        &mut cache.runs,
        "bo2mp_skin_runs",
        BufferUsages::STORAGE,
        (cache.rows.len() * 32) as u64,
    );
    queue.write_buffer(&runs_buffer, 0, bytemuck::cast_slice(&cache.rows));
    let bind_group = device.create_bind_group(
        "bo2mp_skin",
        &pipeline.layout,
        &BindGroupEntries::sequential((
            shape_buffer.as_entire_binding(),
            skin_buffer.as_entire_binding(),
            bones_buffer.as_entire_binding(),
            runs_buffer.as_entire_binding(),
            dst.as_entire_binding(),
        )),
    );
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("bo2mp_skin"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline.pipeline);
        pass.set_bind_group(0, &*bind_group, &[]);
        pass.dispatch_workgroups(total.div_ceil(WORKGROUP), 1, 1);
    }
    true
}
