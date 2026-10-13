// bo2zm: Black Ops II's Afterlife screen (generic_filter_afterlife_alcatraz,
// see bo2_afterlife.rs), as its compiled shader does it.

struct FilterParams {
    // x: vignette softness; y: mask boost; z: mask softness; w: wave warp.
    shape: vec4<f32>,
    // rgb: the vignette's colour; a: how far the filter has faded in.
    colour: vec4<f32>,
    // xy: the vignette's radii; zw: the warp mask's radii.
    radii: vec4<f32>,
    // x: scroll speed; y: seconds.
    scroll: vec4<f32>,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: FilterParams;
@group(0) @binding(3) var noise: texture_2d<f32>;
@group(0) @binding(4) var mask: texture_2d<f32>;
@group(0) @binding(5) var tile_sampler: sampler;

struct FullscreenOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> FullscreenOut {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    var out: FullscreenOut;
    out.position = vec4(uv * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
    out.uv = uv;
    return out;
}

fn smooth01(x: f32) -> f32 {
    let t = clamp(x, 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// One copy of the smoke: turned by `angle`, shrinking as `phase` goes
// 0 -> 1, faded in and out by its weight.
fn layer_uv(uv: vec2<f32>, angle: f32, phase: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    let d = (uv - 0.5) * (0.5 - 0.3 * phase);
    return vec2(dot(d, vec2(c, s)), dot(d, vec2(-s, c))) + 0.5;
}

fn layer_weight(phase: f32) -> f32 {
    return smooth01(min(abs(2.0 * phase - 1.0) * 2.0, 1.0));
}

@fragment
fn fragment(in: FullscreenOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = params.scroll.y * params.scroll.x;
    let phase1 = fract(t * 0.5);
    let uv1 = layer_uv(uv, t * 0.227357, phase1);
    let x2 = t * 0.5 + 0.5;
    let phase2 = fract(x2);
    let uv2 = layer_uv(uv, -x2 * 0.454714, phase2);
    let m1 = textureSample(mask, tile_sampler, uv1).x * layer_weight(phase1);
    let m2 = textureSample(mask, tile_sampler, uv2).x * layer_weight(phase2);
    let smoke = max(m1, m2) * params.shape.y * textureSample(noise, tile_sampler, uv * 8.0).x;
    let ndc = uv * 2.0 - 1.0;
    let r = length(ndc) * 0.707214;
    let vig = pow(smoothstep(params.radii.x, params.radii.y, r), params.shape.x);
    let glow = smoke * vig * params.colour.rgb;
    let m = pow(smoothstep(params.radii.z, params.radii.w, r), params.shape.z);
    let s = 1.0 + (1.0 - m) * params.shape.w;
    let scene = textureSample(source, source_sampler, uv).rgb;
    let warped = textureSample(source, source_sampler, (uv - 0.5) / s + 0.5).rgb;
    let filtered = warped + glow;
    return vec4(mix(scene, filtered, params.colour.a), 1.0);
}
