// bo2mp: live bodies skinned on the graphics card (geometry_diagnostic/skin.rs).
// One thread per written vertex: which run it is in, that run's shape
// vertex moved by up to four bones by their weights, as the CPU's
// `skin_verts` does, written as a 48-byte vertex (position, normal, colour,
// uv) into the moving models' vertex buffer.

struct Run {
    // runs[0] is the header: `dst` the run count, `src` the vertex total,
    // `bone_base` the first vertex written in `dst_vertices`.
    dst: u32,
    src: u32,
    count: u32,
    bone_base: u32,
    bone_count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

// 12 floats a vertex: position, normal, colour, uv.
@group(0) @binding(0) var<storage, read> shape: array<f32>;
// Two a vertex: its four bones, then its four weights' bits.
@group(0) @binding(1) var<storage, read> skin: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read> bones: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage, read> runs: array<Run>;
@group(0) @binding(4) var<storage, read_write> dst_vertices: array<f32>;

fn bone(run: Run, j: u32) -> mat4x4<f32> {
    if j < run.bone_count {
        return bones[run.bone_base + j];
    }
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
}

@compute @workgroup_size(64)
fn skin_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let head = runs[0];
    let i = id.x;
    if i >= head.src {
        return;
    }
    // The last run that starts at or before this vertex (runs 1..=n).
    var lo = 1u;
    var hi = head.dst;
    while lo < hi {
        let mid = (lo + hi + 1u) / 2u;
        if runs[mid].dst <= i {
            lo = mid;
        } else {
            hi = mid - 1u;
        }
    }
    let run = runs[lo];
    let v = run.src + (i - run.dst);
    let b = skin[2u * v];
    let w = bitcast<vec4<f32>>(skin[2u * v + 1u]);
    let at = v * 12u;
    let p = vec4<f32>(shape[at], shape[at + 1u], shape[at + 2u], 1.0);
    let n = vec4<f32>(shape[at + 3u], shape[at + 4u], shape[at + 5u], 0.0);
    var op = vec3<f32>(0.0, 0.0, 0.0);
    var on = vec3<f32>(0.0, 0.0, 0.0);
    for (var k = 0u; k < 4u; k = k + 1u) {
        let wk = w[k];
        if wk > 0.0 {
            let m = bone(run, b[k]);
            op = op + (m * p).xyz * wk;
            on = on + (m * n).xyz * wk;
        }
    }
    let len = length(on);
    if len > 0.0 {
        on = on / len;
    } else {
        on = vec3<f32>(0.0, 0.0, 0.0);
    }
    let out = (head.bone_base + i) * 12u;
    dst_vertices[out] = op.x;
    dst_vertices[out + 1u] = op.y;
    dst_vertices[out + 2u] = op.z;
    dst_vertices[out + 3u] = on.x;
    dst_vertices[out + 4u] = on.y;
    dst_vertices[out + 5u] = on.z;
    for (var c = 6u; c < 12u; c = c + 1u) {
        dst_vertices[out + c] = shape[at + c];
    }
}
