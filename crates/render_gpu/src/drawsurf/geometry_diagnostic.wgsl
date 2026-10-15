#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

// bo2zm: a world surface's colour map (only the textured world pipeline
// binds group 1).
@group(1) @binding(0) var colour_map: texture_2d<f32>;
@group(1) @binding(1) var colour_sampler: sampler;
// bo2zm: a layered surface's layer 1 and 2 colour maps.
@group(1) @binding(6) var colour_map1: texture_2d<f32>;
@group(1) @binding(7) var colour_map2: texture_2d<f32>;

// bo2zm: a T6 lightmap page and the world lighting. Lightmapped world
// surfaces bind all four; every other Black Ops II surface (unlit world
// surfaces, props) binds the lighting and lights alone (bindings 2, 3).
struct T6Lighting {
    // xyz: direction to the sun; w: exposure scale.
    sun_dir_exposure: vec4<f32>,
    // rgb: sun colour.
    sun_color: vec4<f32>,
    // The sky's rotation (x, y, z of skyBoxRotation) and brightness (w,
    // negative when z flips).
    sky: vec4<f32>,
    // bo2zm: the sky's skyColorParm x (above 0: the cube is rgb / a, else
    // rgb^2) and w (the share of the fog it takes). bo2mp: z is 1 on a
    // Black Ops III world lit by its own light (`t7_shade`).
    sky_parms: vec4<f32>,
    // bo2zm: the vision's colour grade (see bo2_final): tone band slopes and
    // offsets, the red, green and blue rows of the shadow, mid and highlight
    // matrices, then gamma, saturation (luma weights, amount) and blend.
    // Identity on a multiplayer map (its grade is bo2_bloom's final pass).
    grade: array<vec4<f32>, 14>,
    // bo2zm: the world's colour tables drawn (see world_lut): the first rows
    // of tables A and B, A's weight, and 1 when the map has them.
    lut: vec4<f32>,
}
// The map's primary lights, four vec4 each: origin and type (1 sun, 2 spot,
// 5 omni), colour and radius, direction and the cone's outer cosine, then
// dAttenuation and the inner cosine. Rows 256 and 257 (vec4 1024 on) are
// the fog's.
const T6_MAX_LIGHTS: u32 = 256u;
struct T6Lights {
    v: array<vec4<f32>, 1032>,
}
@group(2) @binding(0) var lightmap: texture_2d<f32>;
@group(2) @binding(1) var lightmap_sampler: sampler;
@group(2) @binding(2) var<uniform> t6: T6Lighting;
@group(2) @binding(3) var<uniform> t6_lights: T6Lights;
// bo2zm: the lights effects throw (muzzle flashes, explosions, impact
// flashes): a count, then per light its origin and radius, its colour.
struct T6DynLights {
    count: vec4<f32>,
    v: array<vec4<f32>, 32>,
}
@group(2) @binding(4) var<uniform> t6_dyn: T6DynLights;
// bo2mp: Black Ops II's sun shadow partitions (near, far), read with a
// compare sampler: each one's clip-from-world, then (on, near texel, far
// texel, depth range), sizes in world units.
struct SunShadow {
    near_from_world: mat4x4<f32>,
    far_from_world: mat4x4<f32>,
    info: vec4<f32>,
}
@group(2) @binding(5) var sun_shadow_map: texture_depth_2d_array;
@group(2) @binding(6) var sun_shadow_sampler: sampler_comparison;
@group(2) @binding(7) var<uniform> sun_shadow: SunShadow;
// bo2zm: the world's colour tables (`GfxWorld::lutMaterial`; a 1x1 white
// texel without them).
@group(2) @binding(8) var t6_lut: texture_2d<f32>;
@group(2) @binding(9) var t6_lut_sampler: sampler;

// bo2zm: Black Ops II's lit shaders add such lights ("glights", the lmap
// pixel shaders' combined_glight path) as colour * saturate(1 - d /
// radius)^2 * N.L, diffuse only.
fn dyn_lights(p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    var sum = vec3(0.0);
    let count = min(u32(t6_dyn.count.x + 0.5), 16u);
    for (var i = 0u; i < count; i++) {
        let a = t6_dyn.v[i * 2u];
        let c = t6_dyn.v[i * 2u + 1u];
        let d = a.xyz - p;
        let dist = max(length(d), 0.0001);
        let fall = saturate(1.0 - dist / max(a.w, 1.0));
        sum += c.rgb * (fall * fall) * saturate(dot(d / dist, n));
    }
    return sum;
}

#ifdef SOFT
// bo2zm: the scene depth, for soft effect edges.
@group(3) @binding(0) var soft_depth: texture_depth_2d;

// BO2's zfeather pixel shaders: how far the scene behind this pixel lies
// past it, times `inv` (1 / the fade distance), clamped to 0..1.
fn soft_fade(clip: vec4<f32>, world: vec3<f32>, inv: f32) -> f32 {
    let scene_d = textureLoad(soft_depth, vec2<i32>(clip.xy), 0);
    let uv = (clip.xy - view.viewport.xy) / view.viewport.zw;
    let ndc = vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, scene_d, 1.0);
    let scene_view = view.view_from_clip * ndc;
    let scene_z = select(1e9, -scene_view.z / scene_view.w, scene_d > 0.0);
    let sprite_z = -(view.view_from_world * vec4(world, 1.0)).z;
    return saturate((scene_z - sprite_z) * inv);
}
#endif

#ifdef SHINE
// bo2zm: a shine surface's normal and specular maps, and the reflection
// probes (their cube array and, per probe, its lightingSH: three vec4).
@group(1) @binding(8) var normal_map: texture_2d<f32>;
@group(1) @binding(9) var specular_map: texture_2d<f32>;
// bo2mp: an emissive flow surface's colour remap (rows above) and constants
// (the last row); a 1x1 stand-in for every other shine surface.
@group(1) @binding(10) var aux_map: texture_2d<f32>;
struct T6Probes {
    v: array<vec4<f32>, 96>,
}
@group(3) @binding(0) var probe_cubes: texture_cube_array<f32>;
@group(3) @binding(1) var probe_sampler: sampler;
@group(3) @binding(2) var<uniform> probes: T6Probes;
#endif

struct VertexIn {
    @builtin(instance_index) instance: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    // bo2zm: xyz the tangent, w the binormal's sign.
    @location(2) tangent: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) uv: vec2<f32>,
    @location(5) lightmap_uv: vec2<f32>,
#ifdef LAYERS
    // bo2zm: layer 1 and 2 texcoords (a second buffer).
    @location(6) layer_uv: vec4<f32>,
#endif
}

struct SmodelVertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(6) world_from_local_0: vec4<f32>,
    @location(7) world_from_local_1: vec4<f32>,
    @location(8) world_from_local_2: vec4<f32>,
    @location(9) world_from_local_3: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) lightmap_uv: vec2<f32>,
    @location(4) world_position: vec3<f32>,
    // bo2zm: the surface's primary light (the world draw's instance index).
    @location(5) @interpolate(flat) light_index: u32,
    @location(6) layer_uv: vec4<f32>,
    // bo2zm: the tangent frame and reflection probe (shine).
    @location(7) tangent: vec3<f32>,
    @location(8) binormal: vec3<f32>,
    @location(9) @interpolate(flat) probe: u32,
}

@vertex
fn vertex(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = view.clip_from_world * vec4(in.position, 1.0);
    out.normal = in.normal;
    out.color = in.color;
    out.uv = in.uv;
    out.lightmap_uv = in.lightmap_uv;
    out.world_position = in.position;
    out.tangent = in.tangent.xyz;
    // As BO2's world vertex shaders: binormal = cross(normal, tangent)
    // times the sign of the vertex's binormal sign.
    out.binormal = cross(in.normal, in.tangent.xyz) * sign(in.tangent.w);
#ifdef SHINE
    // A shine surface's instance index: primary light, then its probe.
    out.light_index = in.instance & 63u;
    out.probe = in.instance >> 6u;
#else
    out.light_index = in.instance;
    out.probe = 0u;
#endif
#ifdef LAYERS
    out.layer_uv = in.layer_uv;
#else
    out.layer_uv = vec4(0.0);
#endif
    return out;
}

@vertex
fn vertex_smodel(in: SmodelVertexIn) -> VertexOut {
    let world_from_local = mat4x4<f32>(
        in.world_from_local_0,
        in.world_from_local_1,
        in.world_from_local_2,
        in.world_from_local_3,
    );
    var out: VertexOut;
    let world = world_from_local * vec4(in.position, 1.0);
    out.clip_position = view.clip_from_world * world;
    out.normal = (world_from_local * vec4(in.normal, 0.0)).xyz;
    out.color = in.color;
    out.uv = in.uv;
    out.lightmap_uv = vec2(0.0);
    out.world_position = world.xyz;
    out.light_index = 0u;
    out.layer_uv = vec4(0.0);
    out.tangent = vec3(1.0, 0.0, 0.0);
    out.binormal = vec3(0.0, 1.0, 0.0);
    out.probe = 0u;
    return out;
}

// The untextured fallback: a surface whose material the exact pass refused
// (most often an image that did not decode). Neutral grey with a little
// up-facing shade so the shape still reads; the load report names the
// material ("unresolved material images"). IW4L_DIAG_CHECKER=1 restores the
// developer checker (normal colour x UV cells).
@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
#ifdef DIAG_CHECKER
    let normal_colour = abs(normalize(in.normal)) * 0.72 + vec3(0.08, 0.02, 0.10);
    let cell = i32(floor(in.uv.x * 16.0) + floor(in.uv.y * 16.0)) & 1;
    let diagnostic_tint = select(vec3(0.72, 0.22, 0.78), vec3(0.22, 0.72, 0.78), cell == 0);
    return vec4(mix(normal_colour, diagnostic_tint, 0.28) * max(in.color.rgb, vec3(0.35)), 1.0);
#else
    let n = normalize(in.normal);
    let shade = 0.85 + 0.15 * n.z;
    return vec4(vec3(0.38) * shade, 1.0);
#endif
}

// bo2zm: the colour map's stored texel. The material images decode as sRGB;
// Black Ops II's shaders read the raw texel and square it themselves.
fn raw_texel(c: vec3<f32>) -> vec3<f32> {
    let x = max(c, vec3(0.0));
    let lo = x * 12.92;
    let hi = 1.055 * pow(x, vec3(1.0 / 2.4)) - 0.055;
    return select(hi, lo, x <= vec3(0.0031308));
}

// bo2zm: Black Ops II's final image, after its surface shaders: the game's
// resolve rolls highlights off above 0.75 (hdr_bloom_apply: 0.75 + 0.25 *
// (1 - 2^(-5.77 * (x - 0.75)))), here, so the 8-bit frame keeps them. The
// final pass (bo2_bloom) undoes it, adds bloom, rolls off again and, on a
// multiplayer map, looks the frame up in the map's colour table (its vision
// file and lut image). bo2zm: on a Zombies map the world's colour tables and
// the vision's grade (scripts switch visions) are applied here instead.
fn bo2_final(encoded: vec3<f32>) -> vec3<f32> {
    return bo2_finish(bo2_rolloff(encoded), t6, t6_lut, t6_lut_sampler);
}

// bo2zm: the highlights' roll-off above 0.75 (gamma-encoded).
fn bo2_rolloff(encoded: vec3<f32>) -> vec3<f32> {
    let rolled = 0.75 + 0.25 * (1.0 - exp2(-5.77078 * (encoded - 0.75)));
    return max(select(encoded, rolled, encoded > vec3(0.75)), vec3(0.0));
}

// bo2zm: a rolled-off colour through the world's colour table and the
// vision's grade (both identity on a multiplayer map).
fn bo2_finish(rolled: vec3<f32>, l: T6Lighting, lut: texture_2d<f32>, s: sampler) -> vec3<f32> {
    return bo2_grade(world_lut(lut, s, rolled, l.lut), l);
}

// bo2zm: the world's colour table, looked up before the vision grade (the
// game bakes both into one table each frame): a strip of 32 slices of 32 x
// 32, red across a slice, green down it, blue picking the slice (blended
// with the next one), one table per 32 rows. Tables A and B blend by A's
// weight as the camera moves between colour table volumes.
fn world_lut(lut: texture_2d<f32>, s: sampler, x: vec3<f32>, pick: vec4<f32>) -> vec3<f32> {
    if pick.w < 0.5 {
        return x;
    }
    let size = vec2<f32>(textureDimensions(lut));
    let c = saturate(x) * 31.0;
    let slice = floor(c.z);
    let next = min(slice + 1.0, 31.0);
    let f = c.z - slice;
    let u0 = (slice * 32.0 + c.x + 0.5) / size.x;
    let u1 = (next * 32.0 + c.x + 0.5) / size.x;
    let va = (pick.x + c.y + 0.5) / size.y;
    let vb = (pick.y + c.y + 0.5) / size.y;
    let a = mix(
        textureSampleLevel(lut, s, vec2(u0, va), 0.0).rgb,
        textureSampleLevel(lut, s, vec2(u1, va), 0.0).rgb,
        f,
    );
    let b = mix(
        textureSampleLevel(lut, s, vec2(u0, vb), 0.0).rgb,
        textureSampleLevel(lut, s, vec2(u1, vb), 0.0).rgb,
        f,
    );
    return mix(b, a, pick.z);
}

// bo2zm: the vision's colour grade of a rolled-off, gamma-encoded colour.
fn bo2_grade(x: vec3<f32>, l: T6Lighting) -> vec3<f32> {
    let lin = x * x;
    let g = l.grade;
    let r = saturate(dot(x, vec3(0.212585, 0.715195, 0.07222)) * g[0] + g[1]);
    let s = vec3(r.x, r.z * r.w, r.y);
    let band = s * s * (3.0 - 2.0 * s);
    let w = band / max(band.x + band.y + band.z, 0.000001);
    let c = vec4(lin, 1.0);
    let red = saturate(dot(vec3(dot(g[2], c), dot(g[3], c), dot(g[4], c)), w));
    let green = saturate(dot(vec3(dot(g[5], c), dot(g[6], c), dot(g[7], c)), w));
    let blue = saturate(dot(vec3(dot(g[8], c), dot(g[9], c), dot(g[10], c)), w));
    let graded = exp2(log2(max(vec3(red, green, blue), vec3(0.0000001))) * g[11].xyz);
    let luma = dot(graded, g[12].xyz);
    let toned = vec3(luma) + g[12].w * (graded - vec3(luma));
    return sqrt(saturate(lin + g[13].xyz * (toned - lin)));
}

// bo2zm: what every lit and unlit Black Ops II shader writes: the colour
// times the exposure (`hdrControl0.x`), square-rooted; the colour target
// holds gamma-encoded values. Then the game's final image steps.
fn bo2_out(colour: vec3<f32>) -> vec3<f32> {
    return bo2_final(sqrt(max(colour * t6.sun_dir_exposure.w, vec3(0.0))));
}

// bo2zm A7: what an additive draw adds: the final image of its colour less
// that of black. The game grades the frame once, after the blend; graded on
// its own, each additive quad added the grade's black (Nuketown's shadow
// matrix biases, about 3/8/16 of 255) over its whole square: pale-blue
// rectangles round the fires and the light shafts.
fn bo2_added(encoded: vec3<f32>) -> vec3<f32> {
    return max(bo2_final(encoded) - bo2_final(vec3(0.0)), vec3(0.0));
}

// bo2zm: an unlit surface's output (additive ones add no black, above).
fn unlit_out(colour: vec3<f32>) -> vec3<f32> {
#ifdef ADD
    return bo2_added(sqrt(max(colour * t6.sun_dir_exposure.w, vec3(0.0))));
#else
    return bo2_out(colour);
#endif
}

// bo2zm: the map's fog as the game's shaders apply it (vertex shaders of
// the lit, unlit and sky techniques). Density halves every half height
// above the base height and doubles every half height below it, up to 100x
// (BO2's fog constants: x carries ln(density / max density) = ln(0.01), max
// density 100x; Die Rise's cloud deck under the rooftops). Along the ray
// from the eye its mean is 100 (F(x1) - F(x0)) / (x1 - x0) with x = -ln2 *
// (z - base) / half height + ln(0.01) and F(x) = e^x below zero, x + 1
// above. Fog = 1 - min(1, 2^-((ln2 * mean * d - start) / half distance)):
// with height fog on, BO2 scales the distance term by ln2 and the shader's
// last step is a base-2 exp. A half height under 1 is no height
// fog (the world fog's rows already raise theirs to 1, as BO2 does). Its
// colour and opacity blend toward the
// sun fog by t = saturate(y * dot(sun fog dir, view dir) + x); the colour
// moves toward the fog colour (HDR, before the exposure) by fog * opacity.
// `a`..`e` are light-table slot 257 (four vec4) and slot 256's first vec4.
fn fog_f(x: f32) -> f32 {
    return select(x + 1.0, exp(x), x < 0.0);
}

// BO2's ln2 on the distance term, only with height fog on.
fn fog_ln2(half_height: f32) -> f32 {
    return select(1.0, 0.6931472, half_height >= 1.0);
}

// The mean fog density (in units of the base density) from the eye to `pz`.
fn fog_mean(eye_z: f32, pz: f32, base: f32, half_height: f32) -> f32 {
    let k = select(0.0, -0.6931472 / half_height, half_height >= 1.0);
    let x0 = k * (eye_z - base) - 4.6051702;
    let x1 = k * (pz - base) - 4.6051702;
    let dx = x1 - x0;
    let flat_mean = select(1.0, exp(x0), x0 < 0.0);
    return 100.0 * select((fog_f(x1) - fog_f(x0)) / dx, flat_mean, abs(dx) < 0.0001);
}

fn fog_apply(
    colour: vec3<f32>,
    p: vec3<f32>,
    a: vec4<f32>,
    c: vec4<f32>,
    s: vec4<f32>,
    e: vec4<f32>,
    q: vec4<f32>,
) -> vec3<f32> {
    if (a.w < 0.5) {
        return colour;
    }
    let f = fog_at(p, a, c, s, e, q);
    return mix(colour, f.tint, f.fog * f.opacity);
}

// The fog toward `p`: its colour, amount and opacity.
struct FogAt {
    tint: vec3<f32>,
    fog: f32,
    opacity: f32,
}

fn fog_at(
    p: vec3<f32>,
    a: vec4<f32>,
    c: vec4<f32>,
    s: vec4<f32>,
    e: vec4<f32>,
    q: vec4<f32>,
) -> FogAt {
    let eye = view.world_position;
    let d = distance(p, eye);
    let mean = fog_mean(eye.z, p.z, c.w, e.w) * fog_ln2(e.w);
    let fog = 1.0 - min(exp2(-(mean * d - a.x) / max(a.y, 1.0)), 1.0);
    let t = saturate(q.y * dot(e.xyz, (p - eye) / max(d, 0.0001)) + q.x);
    return FogAt(mix(c.rgb, s.rgb, t), fog, mix(a.z, s.w, t));
}

// bo2zm: the fog as Black Ops II's effect vertex shaders hand it on: the
// fog colour times the exposure, square-rooted (display space), and how
// much of the effect's own colour stays, (1 - fog * opacity)^2.
fn effect_fog(p: vec3<f32>) -> vec4<f32> {
    let a = t6_lights.v[1028u];
    let c = t6_lights.v[1029u];
    let s = t6_lights.v[1030u];
    let e = t6_lights.v[1031u];
    let q = t6_lights.v[1024u];
    if (a.w < 0.5) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    let eye = view.world_position;
    let d = distance(p, eye);
    let mean = fog_mean(eye.z, p.z, c.w, e.w) * fog_ln2(e.w);
    let fog = 1.0 - min(exp2(-(mean * d - a.x) / max(a.y, 1.0)), 1.0);
    let t = saturate(q.y * dot(e.xyz, (p - eye) / max(d, 0.0001)) + q.x);
    let tint = mix(c.rgb, s.rgb, t);
    let opacity = mix(a.z, s.w, t);
    let keep = 1.0 - fog * opacity;
    return vec4(sqrt(saturate(tint * t6.sun_dir_exposure.w)), keep * keep);
}

fn bo2_fog(colour: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    return fog_apply(
        colour,
        p,
        t6_lights.v[1028u],
        t6_lights.v[1029u],
        t6_lights.v[1030u],
        t6_lights.v[1031u],
        t6_lights.v[1024u],
    );
}

// bo2zm: an unlit surface's fog. Additive ones fade toward black by the
// fog's keep factor (going toward the fog colour would add it over the
// whole surface: Tranzit's 4000-unit fire glow cards lit the town's skyline
// and the power station's window in bright rectangles); the rest go toward
// the fog colour.
fn unlit_fog(colour: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
#ifdef ADD
    return colour * sqrt(effect_fog(p).w);
#else
    return bo2_fog(colour, p);
#endif
}

// bo2zm: the direct light of primary light `index` at `p` with normal `n`,
// before its baked visibility. The sun uses the world's sun; spot and omni
// lights fall off with dAttenuation / d^2 (full strength inside) times a
// smoothstep to zero at their radius, and spots fade between their outer
// and inner cone cosines (the beam runs along -direction). Structure from
// the game's lmap shaders; the parameter mapping is inferred.
// bo2zm: a primary light at a point: the direction to it and its colour
// there (attenuated, before N.L).
struct LightRay {
    dir: vec3<f32>,
    colour: vec3<f32>,
}

// bo2mp: one sun partition at a point (its clip position): four compare
// taps half a texel apart, the point moved toward the sun by a texel and a
// half plus half a unit so a lit side does not shadow itself.
fn sun_shadow_partition(layer: i32, clip: vec4<f32>, texel: f32) -> f32 {
    let uv = clip.xy * vec2(0.5, -0.5) + vec2(0.5);
    let depth = clip.z - (texel * 1.5 + 0.5) / sun_shadow.info.w;
    let o = 0.5 / 2048.0;
    var lit = textureSampleCompareLevel(sun_shadow_map, sun_shadow_sampler, uv + vec2(-o, -o), layer, depth);
    lit += textureSampleCompareLevel(sun_shadow_map, sun_shadow_sampler, uv + vec2(o, -o), layer, depth);
    lit += textureSampleCompareLevel(sun_shadow_map, sun_shadow_sampler, uv + vec2(-o, o), layer, depth);
    lit += textureSampleCompareLevel(sun_shadow_map, sun_shadow_sampler, uv + vec2(o, o), layer, depth);
    return lit * 0.25;
}

// bo2mp: how much sun reaches `p` past the things that move (1 = all): the
// near partition where it covers `p`, else the far one, fading out at its
// edge; past both, all of it.
fn sun_shadow_at(p: vec3<f32>) -> f32 {
    if (sun_shadow.info.x < 0.5) {
        return 1.0;
    }
    let near = sun_shadow.near_from_world * vec4(p, 1.0);
    if (all(abs(near.xy) < vec2(0.96)) && near.z > 0.0 && near.z < 1.0) {
        return sun_shadow_partition(0, near, sun_shadow.info.y);
    }
    let far = sun_shadow.far_from_world * vec4(p, 1.0);
    let edge = max(abs(far.x), abs(far.y));
    if (edge >= 1.0 || far.z <= 0.0 || far.z >= 1.0) {
        return 1.0;
    }
    return mix(sun_shadow_partition(1, far, sun_shadow.info.z), 1.0, smoothstep(0.8, 1.0, edge));
}

fn primary_light_ray(index: u32, p: vec3<f32>) -> LightRay {
    var ray: LightRay;
    ray.dir = vec3(0.0, 0.0, 1.0);
    ray.colour = vec3(0.0);
    if (index >= T6_MAX_LIGHTS) {
        return ray;
    }
    let a = t6_lights.v[index * 4u];
    let b = t6_lights.v[index * 4u + 1u];
    let c = t6_lights.v[index * 4u + 2u];
    let d = t6_lights.v[index * 4u + 3u];
    let kind = u32(a.w + 0.5);
    if (kind == 1u) {
        ray.dir = t6.sun_dir_exposure.xyz;
        ray.colour = t6.sun_color.rgb * sun_shadow_at(p);
        return ray;
    }
    if (kind == 0u) {
        return ray;
    }
    let to_light = a.xyz - p;
    let d2 = max(dot(to_light, to_light), 1.0);
    let dist = sqrt(d2);
    let l = to_light / dist;
    var atten = saturate(d.x / d2);
    let fall = saturate(1.0 - dist / max(b.w, 1.0));
    atten *= fall * fall * (3.0 - 2.0 * fall);
    if (kind >= 2u && kind <= 4u) {
        atten *= smoothstep(c.w, max(d.y, c.w + 0.0001), dot(l, c.xyz));
    }
    ray.dir = l;
    ray.colour = b.rgb * atten;
    return ray;
}

// bo2mp: a model's light plus a primary light's `lit * N.L`. On a Call of
// Duty 4 / World at War map (the sun colour's alpha 3 or more) as their lit
// model shaders add them (`lp_sun_*`: 2 * grid + visibility * N.L * sun),
// in gamma space: `light` is the grid's (2 * colour)^2, the sun colour a
// gamma value. Black Ops II adds them linear.
fn add_light(light: vec3<f32>, lit: vec3<f32>, ndotl: f32) -> vec3<f32> {
    // (bo2mp: an Infinite Warfare map's, alpha 5, adds them linear.)
    if (t6.sun_color.a > 2.5 && t6.sun_color.a < 4.5) {
        let g = sqrt(max(light, vec3(0.0))) + lit * ndotl;
        return g * g;
    }
    return light + lit * ndotl;
}

fn primary_light(index: u32, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let ray = primary_light_ray(index, p);
    return ray.colour * saturate(dot(n, ray.dir));
}

// bo2mp: Black Ops III's own light (`assets::lane::t7_light`), on a world
// that has it (`t7_on`). Its numbers sit in light rows 254 and 255: the
// exposure and the sun's brightness over the sun colour drawn; the baked
// sun map's axes (u, v: (axis, origin), toward the sun: (axis, 1 / texel))
// and its height packing (base, range, size). The sun map is the colour
// table's slot (`t6_lut`): per texel the height toward the sun of the
// nearest surface in 24 bits (rgb), alpha 0 where nothing is.
const T7_ROW: u32 = 254u;

fn t7_on() -> bool {
    return t6.sky_parms.z > 0.5;
}

fn t7_row(i: u32) -> vec4<f32> {
    return t6_lights.v[T7_ROW * 4u + i];
}

fn t7_sun_height(x: i32, y: i32, size: i32) -> f32 {
    if (x < 0 || y < 0 || x >= size || y >= size) {
        return -1e30;
    }
    let t = textureLoad(t6_lut, vec2<i32>(x, y), 0);
    if (t.a < 0.5) {
        return -1e30;
    }
    let q = round(t.r * 255.0) * 65536.0 + round(t.g * 255.0) * 256.0 + round(t.b * 255.0);
    let b = t6_lights.v[(T7_ROW + 1u) * 4u];
    return b.x + b.y * q / 16777215.0;
}

// How much of the baked sun reaches `p` (surface normal `n`): the sun map
// read 4 x 4 texels round it, bilinear weights (a 3-texel tent), the point
// moved off the surface by a texel and a half and up toward the sun by two.
fn t7_sun_visibility(p: vec3<f32>, n: vec3<f32>) -> f32 {
    let ru = t7_row(1u);
    let rv = t7_row(2u);
    let rd = t7_row(3u);
    let size = i32(t6_lights.v[(T7_ROW + 1u) * 4u].z + 0.5);
    if (size < 2) {
        return 1.0;
    }
    let texel = 1.0 / max(rd.w, 0.000001);
    let q = p + n * (texel * 1.5);
    let fx = (dot(q, ru.xyz) - ru.w) * rd.w - 0.5;
    let fy = (dot(q, rv.xyz) - rv.w) * rd.w - 0.5;
    let h = dot(q, rd.xyz) + texel * 2.0 + 1.0;
    let x0 = i32(floor(fx));
    let y0 = i32(floor(fy));
    let f = vec2(fx - floor(fx), fy - floor(fy));
    var lit = 0.0;
    for (var j = -1; j <= 2; j++) {
        let wy = select(select(1.0, f.y, j == 2), 1.0 - f.y, j == -1);
        for (var i = -1; i <= 2; i++) {
            let wx = select(select(1.0, f.x, i == 2), 1.0 - f.x, i == -1);
            lit += wx * wy * select(0.0, 1.0, h >= t7_sun_height(x0 + i, y0 + j, size));
        }
    }
    return lit / 9.0;
}

// The tone curve (a filmic fit) of light times the exposure, gamma-encoded
// for the frame (which then takes Black Ops II's roll-off, bloom and grade).
fn t7_tone(c: vec3<f32>, exposure: f32) -> vec3<f32> {
    let x = max(c * exposure, vec3(0.0));
    let a = (x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14);
    return pow(saturate(a), vec3(1.0 / 2.2));
}

fn t7_out(c: vec3<f32>) -> vec3<f32> {
    return bo2_final(t7_tone(c, t7_row(0u).x));
}

// A surface lit as Black Ops III's forward `lit` shader does, reduced:
// albedo (linear) times the ambient light (the vertex's or the model's,
// baked from the probes) and the sun (Lambert, its baked visibility and
// the moving things' shadows); on a surface with its normal and specular
// maps (SHINE) the ambient bent by the normal map (the probe's smallest
// level), the sun's GGX highlight, and the probe's reflection at a level
// by gloss (5 (1 - gloss)), weighted by the split-sum fit and darkened
// where the ambient is. Gloss: Black Ops II's specular alpha back to Black
// Ops III's (power 2^(17 g)).
fn t7_shade(base: vec3<f32>, amb_in: vec3<f32>, vn: vec3<f32>, n: vec3<f32>, st: vec4<f32>, probe: u32, p: vec3<f32>) -> vec3<f32> {
    let sun_dir = t6.sun_dir_exposure.xyz;
    let sun = t6.sun_color.rgb * t7_row(0u).y;
    let ndl = saturate(dot(n, sun_dir));
    var vis = 0.0;
    if (dot(vn, sun_dir) > -0.05) {
        vis = t7_sun_visibility(p, vn) * sun_shadow_at(p);
    }
    var amb = amb_in;
    var colour = vec3(0.0);
#ifdef SHINE
    let lum = vec3(0.2126, 0.7152, 0.0722);
    let pi = min(probe, 31u);
    let gain = probes.v[pi * 3u].x;
    let l6v = textureSampleLevel(probe_cubes, probe_sampler, vn, i32(pi), 6.0).rgb;
    let l6n = textureSampleLevel(probe_cubes, probe_sampler, n, i32(pi), 6.0).rgb;
    amb *= clamp(dot(l6n, lum) / max(dot(l6v, lum), 0.0001), 0.5, 2.0);
    let sraw = raw_texel(st.rgb);
    let f0 = sraw * sraw;
    let g = log2(max(exp2(13.0 * st.a) - 2.0, 1.0)) / 17.0;
    let a2 = clamp(2.0 / (exp2(17.0 * g) + 2.0), 0.0005, 1.0);
    let v = normalize(view.world_position - p);
    let ndv = max(dot(n, v), 0.0001);
    let h = normalize(sun_dir + v);
    let ndh = saturate(dot(n, h));
    let ldh = saturate(dot(sun_dir, h));
    let dd = ndh * ndh * (a2 - 1.0) + 1.0;
    let d = a2 / (3.14159265 * dd * dd);
    let fq = 1.0 - ldh;
    let fr = f0 + (1.0 - f0) * (fq * fq * fq * fq * fq);
    colour += sun * (vis * ndl * d / max(4.0 * ldh * ldh, 0.05)) * fr;
    let r = reflect(-v, n);
    let mip = clamp(5.0 * (1.0 - g) - 0.35, 0.0, 6.0);
    let env = textureSampleLevel(probe_cubes, probe_sampler, r, i32(pi), mip).rgb * gain;
    let rough = sqrt(sqrt(a2));
    let rr = rough * vec4(-1.0, -0.0275, -0.572, 0.022) + vec4(1.0, 0.0425, 1.04, -0.04);
    let a004 = min(rr.x * rr.x, exp2(-9.28 * ndv)) * rr.x + rr.y;
    let ab = vec2(-1.04, 1.04) * a004 + rr.zw;
    let so = saturate(dot(amb_in, lum) / max(dot(l6v, lum) * gain, 0.0001));
    colour += env * (f0 * ab.x + ab.y) * so;
#endif
    colour += base * (amb + sun * (vis * ndl));
    return colour;
}

// bo2zm: world surfaces without a lightmap. Unlit ones as the game's unlit
// shaders draw them: (texel * vertex colour)^2 times the material's colour
// scale, then the BO2 output. A multiply decal scales what is behind by
// 1 + a * (texel * vertex colour - 1). Lit ones without a lightmap get a
// plain sun-and-sky light. Alpha feeds the blended variants.
@fragment
fn fragment_textured(in: VertexOut) -> @location(0) vec4<f32> {
    let albedo = textureSample(colour_map, colour_sampler, in.uv);
#ifdef ALPHA_TEST
    // The game's alpha-tested shaders discard below 128/255.
    if (albedo.a < 0.501961) {
        discard;
    }
#endif
    let texel = raw_texel(albedo.rgb);
#ifdef UNLIT
    // (bo2mp: on a world lit by Black Ops III's own light the vertex
    // colour is its baked light, not a tint.)
    let tint = select(in.color.rgb, vec3(1.0), t7_on());
#ifdef MULTIPLY
    let a = albedo.a * in.color.a;
    return vec4(mix(vec3(1.0), texel * tint, a), a);
#else
    let c = texel * tint;
    return vec4(
        unlit_out(unlit_fog(c * c * f32(#{UNLIT_SCALE}), in.world_position)),
        albedo.a * in.color.a,
    );
#endif
#else
    let n = normalize(in.normal);
    let sun = normalize(vec3(0.35, 0.25, 0.9));
    let light = 0.55 + 0.45 * max(dot(n, sun), 0.0);
    return vec4(texel * light, albedo.a);
#endif
}

#ifdef FLOW
// bo2mp: Black Ops II's emissive flow (`cod7emissiveflow`, the molten
// lava; its lit technique, fxc disassembly). The specular map holds its
// emissive masks and noise (hi.a, lo.a, noise.g, noise.a, packed by the
// assets lane), the auxiliary map its colour remap and, in the last
// row, its constants: diffuse uv control (scale, scroll), emissive uv
// scale and scroll, noise uv control, normal scale and bias, remap uv,
// diffuse filter (w: noise weight), emissive filter. The heat is the
// masks' product plus noise times the vertex blue; the remap's row is a
// smoothstep of noise plus the vertex red. The rock: the sun alone
// (Burley diffuse, a GGX lobe of alpha^2 0.2401, Schlick from 0.15), no
// lightmap; the glow: remap^2 times the emissive filter, plus remap^2
// times the diffuse filter by n.v. The normal map fades where the remap
// alpha does. No fog (the shader has none).
fn bo2_flow(
    uv: vec2<f32>,
    vcolour: vec4<f32>,
    vn: vec3<f32>,
    tangent: vec3<f32>,
    binormal: vec3<f32>,
    p: vec3<f32>,
) -> vec3<f32> {
    let crow = i32(textureDimensions(aux_map).y) - 1;
    let fc0 = textureLoad(aux_map, vec2<i32>(0, crow), 0);
    let fc1 = textureLoad(aux_map, vec2<i32>(1, crow), 0);
    let fc2 = textureLoad(aux_map, vec2<i32>(2, crow), 0);
    let fc3 = textureLoad(aux_map, vec2<i32>(3, crow), 0);
    let fc4 = textureLoad(aux_map, vec2<i32>(4, crow), 0);
    let fc5 = textureLoad(aux_map, vec2<i32>(5, crow), 0);
    let fc6 = textureLoad(aux_map, vec2<i32>(6, crow), 0);
    let fc7 = textureLoad(aux_map, vec2<i32>(7, crow), 0);
    let ft = t6_dyn.count.y;
    let fuv0 = uv * fc0.xy;
    let fuv = fuv0 + ft * fc0.zw;
    let fnoise = fuv.xyxy * fc3;
    let femit = fuv0.xyxy * fc1 + ft * fc2;
    let fhi = textureSample(specular_map, colour_sampler, femit.xy).r;
    let flo = textureSample(specular_map, colour_sampler, femit.zw).g;
    let fny = textureSample(specular_map, colour_sampler, fnoise.xy).b;
    let fna = textureSample(specular_map, colour_sampler, fnoise.zw).a;
    let fheat = fhi * flo + fny * vcolour.b * fc6.w;
    var frow = saturate((fna - 0.5) * 0.25 + vcolour.r);
    frow = frow * frow * (3.0 - 2.0 * frow);
    let fruv = vec2(fheat, frow) * fc5.xy + fc5.zw;
    let frows = f32(crow);
    let fremap = textureSample(aux_map, colour_sampler, vec2(fruv.x, fruv.y * frows / (frows + 1.0)));
    let falbedo = textureSample(colour_map, colour_sampler, fuv);
    let fnm = (textureSample(normal_map, colour_sampler, fuv).xy * fc4.x + fc4.y) * fremap.a;
    let fn_ = normalize(fnm.x * tangent + fnm.y * binormal + vn);
    let fv = normalize(p - view.world_position);
    let fl = t6.sun_dir_exposure.xyz;
    let fh = normalize(fl - fv);
    let fndoth = saturate(dot(fn_, fh));
    let fdd = fndoth * fndoth * -0.7599 + 1.0;
    let fd = 0.2401 / max(fdd * fdd, 0.000001);
    let fhv = saturate(dot(fh, -fv));
    let fhl = saturate(dot(fh, fl));
    let fnv = saturate(dot(fn_, -fv));
    let fnl = saturate(dot(fn_, fl));
    let fq = 1.0 - fhl;
    let fspec = fd * (fq * fq * fq * fq * fq * 0.85 + 0.15) / max(fhv * 4.0, 0.0001) * 0.3184;
    let f90 = fhl * fhl * 0.98 - 0.5;
    let fqv = 1.0 - fnv;
    let fql = 1.0 - fnl;
    let fburley = (1.0 + f90 * fqv * fqv * fqv * fqv * fqv) * (1.0 + f90 * fql * fql * fql * fql * fql);
    let fbase = raw_texel(falbedo.rgb);
    let fr2 = fremap * fremap;
    let flit = fnl * (fbase * fbase * fburley + vec3(fspec)) * t6.sun_color.rgb;
    let fglow = fr2.rgb * fc6.rgb * fnv + fr2.rgb * fc7.rgb;
    return flit + fglow;
}
#endif

// bo2zm: Black Ops II's lit world colour, as its lightmap shaders compute it
// (pimp_shader_lmap_*, read from the game's own shader code): the colour
// map times the vertex colour, squared into linear; the lightmap's three
// pages at (u, v / 3 + page / 3): base light and directional light (rgb / a),
// light direction (rgb * 2 - 1) with the baked visibility of the surface's
// primary light in alpha; light = base + directional * N.dir + primary *
// visibility; then the BO2 output. Normal maps, specular and fog are not
// drawn.
@fragment
fn fragment_lightmapped(in: VertexOut) -> @location(0) vec4<f32> {
    let albedo = textureSample(colour_map, colour_sampler, in.uv);
#ifdef ALPHA_TEST
    // The game's alpha-tested shaders discard below 128/255.
    if (albedo.a < 0.501961) {
        discard;
    }
#endif
    let texel = raw_texel(albedo.rgb);
#ifdef UNTINTED
    // Layered materials: the vertex colour is the layer weights (green
    // layer 1, blue layer 2), not a tint. As the game's layer shaders: a
    // blend layer mixes in by its alpha times its weight, a multiply layer
    // scales by 1 + weight * (layer - 1); all on raw texels, then squared.
    var c = texel;
#ifdef LAYERS
#ifdef LAYER1_BLEND
    let l1 = textureSample(colour_map1, colour_sampler, in.layer_uv.xy);
    c = mix(c, raw_texel(l1.rgb), l1.a * in.color.g);
#endif
#ifdef LAYER1_MULTIPLY
    let m1 = raw_texel(textureSample(colour_map1, colour_sampler, in.layer_uv.xy).rgb);
    c = c * (1.0 + in.color.g * (m1 - 1.0));
#endif
#ifdef LAYER2_BLEND
    let l2 = textureSample(colour_map2, colour_sampler, in.layer_uv.zw);
    c = mix(c, raw_texel(l2.rgb), l2.a * in.color.b);
#endif
#ifdef LAYER2_MULTIPLY
    let m2 = raw_texel(textureSample(colour_map2, colour_sampler, in.layer_uv.zw).rgb);
    c = c * (1.0 + in.color.b * (m2 - 1.0));
#endif
    // bo2mp: a test layer (`t1c1`, the `lit_sm_*_t1c1n1` pixel shader):
    // the layer replaces the colour where its alpha times its weight is at
    // least one half (its normal map is not drawn).
#ifdef LAYER1_TEST
    let t1 = textureSample(colour_map1, colour_sampler, in.layer_uv.xy);
    c = select(c, raw_texel(t1.rgb), t1.a * in.color.g >= 0.5);
#endif
    // bo2mp: an add layer (`aK`, the `lit_sm_*_a1c1` pixel shader): the
    // layer times its alpha times its weight, added before squaring.
#ifdef LAYER1_ADD
    let a1 = textureSample(colour_map1, colour_sampler, in.layer_uv.xy);
    c = c + raw_texel(a1.rgb) * a1.a * in.color.g;
#endif
#ifdef LAYER2_ADD
    let a2 = textureSample(colour_map2, colour_sampler, in.layer_uv.zw);
    c = c + raw_texel(a2.rgb) * a2.a * in.color.b;
#endif
#endif
    let base = c * c;
#else
    let tinted = texel * in.color.rgb;
    let base = tinted * tinted;
#endif
    // bo2mp: Black Ops III's own light (`t7_shade`): the vertex colour is
    // the baked ambient light, not a tint.
    if (t7_on()) {
        let vn7 = normalize(in.normal);
        var n7 = vn7;
        var st7 = vec4(0.0);
#ifdef SHINE
        let nm7 = textureSample(normal_map, colour_sampler, in.uv).xy * 4.015748 - 2.015748;
        n7 = normalize(nm7.x * in.tangent + nm7.y * in.binormal + vn7);
        st7 = textureSample(specular_map, colour_sampler, in.uv);
#endif
        return vec4(t7_out(t7_shade(texel * texel, in.color.rgb, vn7, n7, st7, in.probe, in.world_position)), albedo.a);
    }
    let v = in.lightmap_uv.y / 3.0;
    let page0 = textureSample(lightmap, lightmap_sampler, vec2(in.lightmap_uv.x, v));
    let page1 = textureSample(lightmap, lightmap_sampler, vec2(in.lightmap_uv.x, v + 1.0 / 3.0));
    let page2 = textureSample(lightmap, lightmap_sampler, vec2(in.lightmap_uv.x, v + 2.0 / 3.0));
    // bo2mp: a Call of Duty 4 / World at War lightmap (the sun colour's
    // alpha is 2 or more: 2 Ghosts' and Infinite Warfare's, 3 CoD4's and
    // World at War's, 4 those with their own reflection probes;
    // `world_iw3::iw3_lightmap` packs its pages): CoD4's own lit world
    // shaders (`lm_sm_sun_*_sm3`, read from the zone's pixel shaders)
    // light in gamma space: A + B * f, x and y the dominant light's
    // tangent direction from the alphas, f = saturate(rsqrt(1 + x^2 +
    // y^2)) on a flat surface, plus the sun by N.L and its baked visibility
    // (the primary image, page 2 alpha), times the colour map and vertex
    // colour. Written as base * light^2 so the BO2 output's square root
    // gives CoD4's texel * light back.
    if (t6.sun_color.a > 1.5) {
        let x = page0.a * 4.08 - 2.08;
        let y = page1.a * 4.064516 - 2.064516;
        var f = saturate(inverseSqrt(1.0 + x * x + y * y));
        var n_iw3 = normalize(in.normal);
#ifdef WATER
        // bo2mp: CoD4 / World at War water (`wc_water`; the lane's
        // `legacy_water_table`). The games simulate their waves each frame;
        // here four scrolled samples of a ripple map make the normal (water
        // lies flat: no tangent frame), and, in CoD4's gamma space, the
        // water's colour under its baked light and the sun mixes toward the
        // reflection probe by the material's fresnel (envMapParms x to y by
        // (1 - N.V)^z), plus the sun's glint on the ripples.
        let wl0 = textureLoad(colour_map, vec2<i32>(0, 0), 0);
        let wl1 = textureLoad(colour_map, vec2<i32>(1, 0), 0);
        let wl2 = textureLoad(colour_map, vec2<i32>(2, 0), 0);
        let wl3 = textureLoad(colour_map, vec2<i32>(3, 0), 0);
        let wns = textureLoad(colour_map, vec2<i32>(4, 0), 0);
        let wlw = textureLoad(colour_map, vec2<i32>(5, 0), 0);
        let wc0 = textureLoad(colour_map, vec2<i32>(6, 0), 0);
        let wc1 = textureLoad(colour_map, vec2<i32>(7, 0), 0);
        let wca = textureLoad(colour_map, vec2<i32>(8, 0), 0);
        let wcb = textureLoad(colour_map, vec2<i32>(9, 0), 0);
        let wt = t6_dyn.count.y;
        let wp = in.world_position.xy;
        let na = textureSample(normal_map, colour_sampler, wp * wl0.xy + wt * wl0.zw).xy * wns.x + wns.y;
        let nb = textureSample(normal_map, colour_sampler, wp * wl1.xy + wt * wl1.zw).xy * wns.x + wns.y;
        let nc = textureSample(specular_map, colour_sampler, wp * wl2.xy + wt * wl2.zw).xy * wns.z + wns.w;
        let nd = textureSample(specular_map, colour_sampler, wp * wl3.xy + wt * wl3.zw).xy * wns.z + wns.w;
        let wn = normalize(vec3(na * wlw.x + nb * wlw.y + nc * wlw.z + nd * wlw.w, 1.0));
        let wv = normalize(in.world_position - view.world_position);
        let wndotv = saturate(dot(wn, -wv));
        let wfres = saturate(wc0.z + wc0.w * pow(1.0 - wndotv, max(wc0.y, 0.5)));
        let wvis = page2.a * sun_shadow_at(in.world_position);
        let wsun = t6.sun_color.rgb * wvis;
        // (The games light water by the sun and its reflection, not a
        // lightmap: where its baked light is dark it keeps a third of the
        // sun's colour, so a gutter is never a black trench.)
        let wlg = max(
            page0.rgb + page1.rgb * f + wsun * saturate(dot(wn, t6.sun_dir_exposure.xyz)),
            t6.sun_color.rgb * 0.35,
        );
        let wr = wv - 2.0 * dot(wv, wn) * wn;
        let we = textureSampleLevel(probe_cubes, probe_sampler, wr, i32(in.probe), 0.0);
        // (The map's own probes are gamma values, their alpha folded in; a
        // Black Ops II model map's are linear rgb / a for that map's light,
        // relit by the light here as the shine branch does. A black one,
        // the compiler's stand-in, reflects the light here instead.)
        let wlum = clamp(dot(wlg * wlg, vec3(0.299, 0.587, 0.114)), 0.0, 2.0);
        let wprobe = select(sqrt(max(we.rgb / (we.a + 0.000001) * wlum, vec3(0.0))), we.rgb, t6.sun_color.a > 3.5);
        let wenv = select(wprobe, wlg * 0.6, dot(wprobe, vec3(0.333)) < 0.02);
        let wbody = mix(wca, wcb, wndotv);
        var wg = mix(wbody.rgb * wlg, wenv, wfres);
        let wglint = exp2(log2(max(saturate(dot(wr, t6.sun_dir_exposure.xyz)), 0.000001)) * max(wc0.x, 1.0));
        wg += wsun * (wglint * wc1.x);
        let wa = saturate(mix(wbody.a, 1.0, wfres));
        return vec4(bo2_out(bo2_fog(wg * wg, in.world_position)), wa);
#endif
#ifdef SHINE
        // bo2mp: the normal map (Black Ops II's layout: a CoD4 map's xGxR
        // one is moved there when linked, `world_iw3::iw3_normal_to_bo2`)
        // bends the baked dominant light (tangent space (x, y, 1)) and the
        // sun: CoD4's `lm_*n0*` shaders take f = saturate(dot of the two,
        // normalised) and the world normal N + nx * T + ny * B.
        let nm3 = textureSample(normal_map, colour_sampler, in.uv).xy * 4.015748 - 2.015748;
        f = saturate(dot(normalize(vec3(nm3, 1.0)), normalize(vec3(x, y, 1.0))));
        n_iw3 = normalize(nm3.x * in.tangent + nm3.y * in.binormal + n_iw3);
#endif
        // bo2mp: an Infinite Warfare map's pages (the sun colour's alpha 5,
        // `fastfile_iw7::light`) hold linear light: each page is the square
        // root of its light over 4 (squared and added: A + B * f, IW7's
        // `lmap_specenv_*` shaders), the sun linear, its baked shadow a
        // signed distance (`smoothstep(saturate((2p - 1 + k) / 2k))`, k
        // `light::SUN_SHADOW_SOFTNESS`).
        let linear_iw3 = t6.sun_color.a > 4.5;
        var sun_vis = page2.a;
        if (linear_iw3) {
            let st = saturate((page2.a * 2.0 - 1.0 + 0.25) / 0.5);
            sun_vis = st * st * (3.0 - 2.0 * st);
        }
        sun_vis *= sun_shadow_at(in.world_position);
        let sun_iw3 = t6.sun_color.rgb * sun_vis * saturate(dot(n_iw3, t6.sun_dir_exposure.xyz));
        let lg = page0.rgb + page1.rgb * f + sun_iw3;
        var here_iw3 = lg * lg;
        if (linear_iw3) {
            here_iw3 = (page0.rgb * page0.rgb + page1.rgb * page1.rgb * f) * 4.0 + sun_iw3;
        }
        var lit_iw3 = base * (here_iw3 + dyn_lights(in.world_position, n_iw3));
#ifdef SHINE
        let st3 = textureSample(specular_map, colour_sampler, in.uv);
        let v3 = normalize(in.world_position - view.world_position);
        let r3 = v3 - 2.0 * dot(v3, n_iw3) * n_iw3;
        let gloss3 = st3.a;
        if (t6.sun_color.a > 3.5 && !linear_iw3) {
            // bo2mp: the map's own specular map and reflection probes, as
            // CoD4's `lm_sm_sun_r0c0n0s0` adds them, in gamma space: the
            // probe (rgb times its alpha, folded in when linked) at level
            // 6 - 8 * gloss, plus the sun's highlight e^((R.L - 0.99925) *
            // (2^(9.3775 * gloss) + 7)) times the sun's colour, its
            // visibility and 0.625 (envMapParms.w); both times the specular
            // colour and the fresnel lerp(0.8, 2, 1 - |N.V|) (envMapParms
            // 0.8 2 1: most of these maps' materials carry it).
            let spec_g = raw_texel(st3.rgb);
            let env_g = textureSampleLevel(probe_cubes, probe_sampler, r3, i32(in.probe), clamp(6.0 - 8.0 * gloss3, 0.0, 6.0)).rgb;
            let power = exp2(gloss3 * 9.377518) + 7.0;
            let lobe = saturate(exp((dot(r3, t6.sun_dir_exposure.xyz) - 0.99925) * power));
            let sun_spec = t6.sun_color.rgb * (sun_vis * lobe * 0.625);
            let fres = mix(0.8, 2.0, 1.0 - abs(dot(n_iw3, v3)));
            let g = sqrt(lit_iw3) + (env_g + sun_spec) * spec_g * fres;
            lit_iw3 = g * g;
        } else {
            let sr3 = raw_texel(st3.rgb);
            let spec3 = sr3 * sr3;
            let l3 = t6.sun_dir_exposure.xyz;
            let h3 = normalize(l3 - v3);
            let ndotl3 = saturate(dot(n_iw3, l3));
            let ndoth3 = saturate(dot(n_iw3, h3));
            let ldoth3 = saturate(dot(h3, l3));
            let power3 = exp2(gloss3 * 13.0);
            let lobe3 = exp2(log2(max(ndoth3, 0.000001)) * power3) * (power3 * 0.125 + 0.25);
            let k3 = saturate(gloss3 + 0.545);
            let vis3 = ndotl3 / (ldoth3 * ldoth3 * k3 - k3 + 1.01);
            let fres3 = spec3 + (1.0 - spec3) * exp2(-10.0 * ldoth3);
            let sun_lin = t6.sun_color.rgb * sun_vis;
            lit_iw3 += fres3 * lobe3 * vis3 * select(sun_lin * sun_lin, sun_lin, linear_iw3);
            // The reflection: the model map's probe (`bo2_look` picks one
            // for every surface), lit as the light here.
            let ndotv3 = saturate(dot(n_iw3, -v3));
            let fk3 = gloss3 * vec4(1.041667, 0.475, 0.018229, 0.25) + vec4(0.0, 0.0, -0.015625, 0.75);
            var ff3 = min(exp2(-9.28 * ndotv3), fk3.y);
            ff3 = fk3.x * ff3 + fk3.z;
            let ew3 = saturate(spec3 * (fk3.w - ff3) + ff3);
            let env3 = textureSampleLevel(probe_cubes, probe_sampler, r3, i32(in.probe), 4.0 - 4.0 * gloss3);
            let env3_rgb = env3.rgb / (env3.a + 0.000001);
            let here3 = here_iw3;
            let probe3 = max(dot(here3, vec3(0.299, 0.587, 0.114)), 0.0001);
            lit_iw3 += ew3 * env3_rgb * clamp(probe3, 0.0, 2.0);
        }
#endif
        return vec4(bo2_out(bo2_fog(lit_iw3, in.world_position)), albedo.a);
    }
    let ambient = page0.rgb / (page0.a + 0.000001);
    let directional = page1.rgb / (page1.a + 0.000001);
    let dir = page2.rgb * 2.0 - 1.0;
#ifdef SHINE
#ifdef FLOW
    // bo2mp: Black Ops II's emissive flow (`bo2_flow`).
    return vec4(
        bo2_out(bo2_flow(in.uv, in.color, normalize(in.normal), in.tangent, in.binormal, in.world_position)),
        1.0,
    );
#endif
#ifdef WATER
    // bo2mp: Black Ops II's water (`cod7water` pools, `cod7watershore` seas
    // and rivers; their sun-shadow techniques, fxc disassembly). The colour
    // map holds the material's constants (assets lane `water_table`), the
    // normal and specular maps its two normal maps. Four normal samples
    // scrolled over the world's xy by game time make the water's normal
    // (as the game's, no tangent frame: water lies flat); fresnel (bias and
    // scale on (1 - n.v)^5) mixes its colour with the reflection probe,
    // relit by the light here over the light the probe saw straight up;
    // the sun adds two highlight lobes. The game refracts the scene behind
    // by the water's opacity; here the water blends over it instead.
    let wl0 = textureLoad(colour_map, vec2<i32>(0, 0), 0);
    let wl1 = textureLoad(colour_map, vec2<i32>(1, 0), 0);
    let wl2 = textureLoad(colour_map, vec2<i32>(2, 0), 0);
    let wl3 = textureLoad(colour_map, vec2<i32>(3, 0), 0);
    let wns = textureLoad(colour_map, vec2<i32>(4, 0), 0);
    let wlw = textureLoad(colour_map, vec2<i32>(5, 0), 0);
    let wc0 = textureLoad(colour_map, vec2<i32>(6, 0), 0);
    let wc1 = textureLoad(colour_map, vec2<i32>(7, 0), 0);
    let wca = textureLoad(colour_map, vec2<i32>(8, 0), 0);
    let wcb = textureLoad(colour_map, vec2<i32>(9, 0), 0);
    let wsh = textureLoad(colour_map, vec2<i32>(10, 0), 0);
    let wt = t6_dyn.count.y;
    let wp = in.world_position.xy;
    let na = textureSample(normal_map, colour_sampler, wp * wl0.xy + wt * wl0.zw).xy * wns.x + wns.y;
    let nb = textureSample(normal_map, colour_sampler, wp * wl1.xy + wt * wl1.zw).xy * wns.x + wns.y;
    let nc = textureSample(specular_map, colour_sampler, wp * wl2.xy + wt * wl2.zw).xy * wns.z + wns.w;
    let nd = textureSample(specular_map, colour_sampler, wp * wl3.xy + wt * wl3.zw).xy * wns.z + wns.w;
    let wn = normalize(vec3(na * wlw.x + nb * wlw.y + nc * wlw.z + nd * wlw.w, 1.0));
    let wv = normalize(in.world_position - view.world_position);
    let wndotv = saturate(dot(wn, -wv));
    let wf = 1.0 - wndotv;
    let wfres = wc0.w * wf * wf * wf * wf * wf + wc0.z;
    let wray = primary_light_ray(in.light_index, in.world_position);
    let wndotl = saturate(dot(wn, wray.dir));
    let wamb = ambient + directional * 0.25;
    let wr = wv - 2.0 * dot(wv, wn) * wn;
    let wenv = textureSampleLevel(probe_cubes, probe_sampler, wr, i32(in.probe), 0.0);
    let wpi = min(in.probe, 31u) * 3u;
    let ws0 = probes.v[wpi];
    let ws1 = probes.v[wpi + 1u];
    let wprobe = max((ws1.z + ws1.w + ws0.w) * ws0.xyz, vec3(0.1));
    // The pool relights the probe by the light here over the probe's own;
    // the sea reflects its sky (the probe's, for the sea's own sky cube).
    let wenv_rgb = wenv.rgb / (wenv.a + 0.000001);
    let wrefl = select(wenv_rgb * wamb / wprobe, wenv_rgb, wc1.w > 0.5);
    let wbody = mix(wca, wcb, wndotv);
    let wsea = wc1.w > 0.5;
    // The sun: the pool's by its baked visibility; the sea's at least
    // three quarters (its shadow term plus 0.75).
    let wsun = wray.colour * select(page2.a, saturate(page2.a + 0.75), wsea);
    var wsolid: vec3<f32>;
    var wadd: vec3<f32>;
    if (wsea) {
        let wlit = wsun * (wndotl / max(wndotv + wndotl, 0.0001)) + wsh.rgb;
        wsolid = mix(mix(wca.rgb, wcb.rgb, wfres) * wlit, wrefl, wfres);
        wadd = vec3(0.0);
    } else {
        let wlight = wamb + wsun * wndotl + dyn_lights(in.world_position, wn);
        wsolid = wbody.rgb * (1.0 - wfres) * wlight;
        wadd = wrefl * wfres;
    }
    let wh = normalize(wray.dir - wv);
    let wndoth = max(saturate(dot(wn, wh)), 0.000001);
    let wlobe = exp2(log2(wndoth) * wc0.x) * wc1.x + exp2(log2(wndoth) * wc0.y) * wc1.y;
    let whl = 1.0 - saturate(dot(wh, wray.dir));
    let wfh = wc0.w * whl * whl * whl * whl * whl + wc0.z;
    let whv = max(saturate(dot(wh, -wv)) * 4.0, 0.0001);
    wadd += wsun * (wlobe * wfh * wndotl / whv * wc1.z);
    // The open sea (kind 2) never reads the vertex colour: its far quads
    // carry alpha 0 at their edges (Frostbite's distant sea).
    let wvalpha = select(in.color.a, 1.0, wc1.w > 1.5);
    let wa = saturate(select(wbody.a, wca.a, wsea) * wvalpha);
    let wout = wsolid + wadd / max(wa, 0.05);
    return vec4(bo2_out(bo2_fog(wout, in.world_position)), wa);
#endif
    // bo2zm: Black Ops II's lit world shader (`wpc_lit_sm_r0c0n0s0`, its
    // sun-shadow technique, from the fxc disassembly), with the lightmap's
    // baked sun visibility (page 2 alpha) for its shadow map.
    let vn = normalize(in.normal);
    let nm = textureSample(normal_map, colour_sampler, in.uv).xy * 4.015748 - 2.015748;
    let n = normalize(nm.x * in.tangent + nm.y * in.binormal + vn);
    let spec_tex = textureSample(specular_map, colour_sampler, in.uv);
    let spec_raw = raw_texel(spec_tex.rgb);
    let spec = spec_raw * spec_raw;
    let gloss = spec_tex.a;
    let vdir = normalize(in.world_position - view.world_position);
    var light = ambient + directional * saturate(dot(dir, n));
    // The reflection: the surface's probe by the reflected view, blurrier
    // the lower the gloss, weighted by a gloss-shaped fresnel and scaled by
    // the light here (vertex normal) over the light the probe saw (its SH
    // at the normal).
    let ndotv = saturate(dot(n, -vdir));
    let fk = gloss * vec4(1.041667, 0.475, 0.018229, 0.25) + vec4(0.0, 0.0, -0.015625, 0.75);
    var f = min(exp2(-9.28 * ndotv), fk.y);
    f = fk.x * f + fk.z;
    let env_weight = saturate(spec * (fk.w - f) + f);
    let r = vdir - 2.0 * dot(vdir, n) * n;
    let env = textureSampleLevel(probe_cubes, probe_sampler, r, i32(in.probe), 4.0 - 4.0 * gloss);
    let env_rgb = env.rgb / (env.a + 0.000001);
    let pi = min(in.probe, 31u) * 3u;
    let s0 = probes.v[pi];
    let s1 = probes.v[pi + 1u];
    let s2 = probes.v[pi + 2u];
    let sh = dot(s2, vec4(vn.z * vn.x, vn.y * vn.z, vn.x * vn.y, vn.x * vn.x - vn.y * vn.y))
        + dot(s1, vec4(vn, 1.0)) + s0.w * vn.z * vn.z;
    let probe_light = max(sh * s0.xyz, vec3(0.1));
    let here = ambient + directional * saturate(dot(dir, vn));
    let reflection = env_weight * env_rgb * here / probe_light;
    // The primary light: diffuse, and its highlight (normalised
    // Blinn-Phong by gloss, BO2's visibility and fresnel terms).
    let ray = primary_light_ray(in.light_index, in.world_position);
    let lit = ray.colour * page2.a;
    let ndotl = saturate(dot(n, ray.dir));
    light += lit * ndotl;
    light += dyn_lights(in.world_position, n);
    let h = normalize(ray.dir - vdir);
    let ndoth = saturate(dot(n, h));
    let ldoth = saturate(dot(h, ray.dir));
    let power = exp2(gloss * 13.0);
    let lobe = exp2(log2(max(ndoth, 0.000001)) * power) * (power * 0.125 + 0.25);
    let k = saturate(gloss + 0.545);
    let vis = ndotl / (ldoth * ldoth * k - k + 1.01);
    let fres = spec + (1.0 - spec) * exp2(-10.0 * ldoth);
    let highlight = fres * lobe * vis * lit;
    let colour = base * light + reflection + highlight;
    return vec4(bo2_out(bo2_fog(colour, in.world_position)), albedo.a);
#else
    let n = normalize(in.normal);
    var light = ambient + directional * saturate(dot(dir, n));
    light += primary_light(in.light_index, in.world_position, n) * page2.a;
    light += dyn_lights(in.world_position, n);
    return vec4(bo2_out(bo2_fog(base * light, in.world_position)), albedo.a);
#endif
}

// bo2zm: Black Ops II props. The instance carries its world-from-local
// columns, its light: rgb from the light grid (linear, already times the
// exposure, w = 0), or for an instance with baked vertex light w = the
// exposure, the vertex colour then holding that light ((c^2) * 32, as the
// game's mlv_* shaders read it); then its primary light and that light's
// visibility from the light grid.
struct PropVertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(6) world_from_local_0: vec4<f32>,
    @location(7) world_from_local_1: vec4<f32>,
    @location(8) world_from_local_2: vec4<f32>,
    @location(9) world_from_local_3: vec4<f32>,
    @location(10) light: vec4<f32>,
    @location(11) primary: vec4<f32>,
}

struct PropOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) light: vec4<f32>,
    @location(4) world_position: vec3<f32>,
    @location(5) primary: vec4<f32>,
#ifdef DECAL
    // A projected decal's box from the world (columns).
    @location(6) @interpolate(flat) box0: vec4<f32>,
    @location(7) @interpolate(flat) box1: vec4<f32>,
    @location(8) @interpolate(flat) box2: vec4<f32>,
    @location(9) @interpolate(flat) box3: vec4<f32>,
#endif
}

@vertex
fn vertex_prop(in: PropVertexIn) -> PropOut {
    let world_from_local = mat4x4<f32>(
        in.world_from_local_0,
        in.world_from_local_1,
        in.world_from_local_2,
        in.world_from_local_3,
    );
    var out: PropOut;
#ifdef CLOUD
    // bo2zm: Black Ops II particle clouds (`particlecloud_*` vertex shader):
    // the point of the cloud's box to the world by the cloud's placement,
    // then out to a quad facing the camera, the corner (texcoord less one
    // half) times the particle's width and height (instance primary.xy);
    // the colour is the cloud's (instance light), and nothing feathers it.
    let centre = world_from_local * vec4(in.position, 1.0);
    let corner = in.uv - vec2(0.5);
    let right = view.world_from_view[0].xyz;
    let up = view.world_from_view[1].xyz;
    let world = vec4(centre.xyz + right * (corner.x * in.primary.x) - up * (corner.y * in.primary.y), 1.0);
    out.clip_position = view.clip_from_world * world;
    out.normal = view.world_from_view[2].xyz;
    out.color = in.light;
    out.uv = in.uv;
    out.light = vec4(0.0);
    out.world_position = world.xyz;
    out.primary = vec4(0.0);
#else
    var world = world_from_local * vec4(in.position, 1.0);
#ifdef EFFECT
    // bo2zm M3: BO2's eye offset (an `_eo` effect material's
    // eyeOffsetParms.x, instance slot 23): the sprite is drawn that many
    // units nearer the eye along its line of sight (at least half way),
    // where it already shows on screen, so a flame on a wall or a pole is
    // not cut into by it.
    let eo = in.primary.w;
    if (eo > 0.0) {
        let to_sprite = world.xyz - view.world_position;
        let d = max(length(to_sprite), 0.0001);
        world = vec4(view.world_position + to_sprite * (max(d - eo, d * 0.5) / d), 1.0);
    }
#endif
#ifdef DECAL
    // bo2zm M4 retest 5: a projected decal's box is already in the world;
    // its instance columns are the box from the world, for the pixels.
    world = vec4(in.position, 1.0);
    out.box0 = in.world_from_local_0;
    out.box1 = in.world_from_local_1;
    out.box2 = in.world_from_local_2;
    out.box3 = in.world_from_local_3;
#endif
    out.clip_position = view.clip_from_world * world;
    out.normal = (world_from_local * vec4(in.normal, 0.0)).xyz;
    out.color = in.color;
    out.uv = in.uv;
    out.light = in.light;
    out.world_position = world.xyz;
    out.primary = in.primary;
    // bo2mp: a weapon camo's surfaces (gold, diamond, ...) tile the camo's
    // pictures across the gun: instance slot 22 holds 1000 + the tiling
    // (its effect feather and unlit dimming are below 1000).
    // (3000 + a pattern camo's repeat passes on to the pixels.)
    if (in.primary.z >= 1000.0 && in.primary.z < 3000.0) {
        out.uv = in.uv * (in.primary.z - 1000.0);
        out.primary.z = 0.0;
    }
#endif
    return out;
}

@fragment
fn fragment_prop(in: PropOut) -> @location(0) vec4<f32> {
#ifdef GHOST
    // bo2zm: Black Ops II's ghost shader (`sw4_3d_char_ghost2`, from the
    // fxc disassembly; Mob's Afterlife ghost hands), with the material's
    // own numbers: two edge noises drifting over the surface (the second
    // turned, here the colour map's again), their product shaping a rim
    // (facing the eye) and a hollow (edge-on) term; the alpha is the
    // smaller of the two times Overall_Intensity, square-rooted. The colour
    // is (Inside_Color + Inside_Add) times the vertex colour, squared,
    // times the light, times Overall_Intensity: a bright cyan glow.
    let t = t6_dyn.count.y;
    let to_eye = normalize(view.world_position - in.world_position);
    let ndv = saturate(dot(normalize(in.normal), to_eye));
    let e2 = raw_texel(textureSample(colour_map, colour_sampler, in.uv * 5.0 + fract(t * vec2(-0.03, 0.01) * 5.0)).rgb).x;
    let d = in.uv - vec2(0.5);
    let turned = vec2(dot(d, vec2(0.982287, -0.187382)), dot(d, vec2(0.187382, 0.982287))) + vec2(0.5);
    let e3 = raw_texel(textureSample(colour_map, colour_sampler, turned * 0.5 + fract(t * vec2(0.05, 0.05) * 0.5)).rgb).x;
    let e = e2 * e3;
    let rim_t = saturate((ndv + 0.02) / 0.73);
    let hollow_t = saturate((ndv - 0.89) / -1.21);
    let rim = rim_t * rim_t * (3.0 - 2.0 * rim_t) * saturate(e * 0.3);
    let hollow = hollow_t * hollow_t * (3.0 - 2.0 * hollow_t) * (e * 1.09);
    let ghost_a = sqrt(max(min(rim, hollow) * 3.99, 0.0));
    let ghost_exposure = max(t6.sun_dir_exposure.w, 0.000001);
    var ghost_light = in.light.rgb / ghost_exposure;
    var ghost_tint = in.color.rgb;
    if (in.light.w > 0.0) {
        ghost_light = in.color.rgb * in.color.rgb * 32.0;
        ghost_tint = vec3(1.0);
    }
    let inside = vec3(0.0, 3.75, 5.0) + vec3(0.5);
    let ghost_c = inside * ghost_tint;
    return vec4(bo2_out(ghost_c * ghost_c * ghost_light * 3.99), saturate(ghost_a));
#else
#ifdef OBJECTIVE
    // bo2zm M3: Black Ops II's objective shader (`mc_objective`, from the
    // fxc disassembly; the magic box's question marks): the colour pulses
    // between colorObjMin and colorObjMax once a second (gameTime),
    // shifted across the surface by how it faces the eye, times one half
    // (its colour map is black), added on (its blend is one, one). The
    // colour map here holds the two colours (texels 0 and 1).
    let lo = textureSampleLevel(colour_map, colour_sampler, vec2(0.25, 0.5), 0.0).rgb;
    let hi = textureSampleLevel(colour_map, colour_sampler, vec2(0.75, 0.5), 0.0).rgb;
    let to_eye = normalize(view.world_position - in.world_position);
    let nz = -dot(normalize(in.normal), to_eye);
    let pulse = 0.5 - 0.5 * sin((nz * -0.5 + t6_dyn.count.y) * 6.28318);
    return vec4(bo2_out(mix(lo, hi, pulse) * 0.5), 1.0);
#else
#ifdef DECAL
    // bo2zm M4 retest 5: Black Ops II's projected decal (`mc_projecteddecal`,
    // from the fxc disassembly; the blood a hit paints on a zombie): the
    // scene behind the box's pixel, from the depth, into the box (-1..1);
    // outside it nothing is drawn. Inside, the texture projected along the
    // box's x axis (at y, z), its alpha fading to the box's x faces, its
    // colour the texture squared times the light grid's light, no fog.
    let scene_d = textureLoad(soft_depth, vec2<i32>(in.clip_position.xy), 0);
    if (scene_d <= 0.0) {
        discard;
    }
    let suv = (in.clip_position.xy - view.viewport.xy) / view.viewport.zw;
    let scene_clip = vec4(suv.x * 2.0 - 1.0, 1.0 - suv.y * 2.0, scene_d, 1.0);
    let scene_h = view.world_from_clip * scene_clip;
    let box_from_world = mat4x4<f32>(in.box0, in.box1, in.box2, in.box3);
    let b = (box_from_world * vec4(scene_h.xyz / scene_h.w, 1.0)).xyz;
    if (any(abs(b) > vec3(1.0))) {
        discard;
    }
    let decal = textureSampleLevel(colour_map, colour_sampler, b.yz * 0.5 + 0.5, 0.0);
    let dtex = raw_texel(decal.rgb);
    let decal_exposure = max(t6.sun_dir_exposure.w, 0.000001);
    return vec4(bo2_out(dtex * dtex * in.light.rgb / decal_exposure), (1.0 - abs(b.x)) * decal.a);
#else
#ifdef SCROLL
    // bo2zm 27c: BO2's scrolling cloud sheets (`mc_sw4_3d_scrolling_zfeather`,
    // from the fxc disassembly): the texcoord times ColorScale.x (instance
    // light.w), drifting by the game time times ColorSpeed (light.yz) times
    // ColorScale.x.
    let albedo = textureSample(
        colour_map,
        colour_sampler,
        in.uv * in.light.w + fract(t6_dyn.count.y * in.light.yz * in.light.w),
    );
#else
    let albedo = textureSample(colour_map, colour_sampler, in.uv);
#endif
#ifdef ALPHA_TEST
    // The game's alpha-tested shaders discard below 128/255.
    if (albedo.a < 0.501961) {
        discard;
    }
#endif
    let texel = raw_texel(albedo.rgb);
#ifdef EFFECT
    // bo2zm: Black Ops II effect shaders (`effect_*`): vertex colour times
    // texture, written as is. Their vertex shaders fade the alpha by the
    // view depth times the material's `featherParms.x` (instance slot 22),
    // so a sprite fades out as it nears the camera.
    let feather = in.primary.z;
    let near = select(
        1.0,
        saturate(distance(in.world_position, view.world_position) * feather),
        feather > 0.0,
    );
    // bo2zm M4 retest 5: the `_ds` (dissolve) ones, from the fxc
    // disassembly: alpha = saturate(alphaDissolveParms.x * (texture alpha +
    // vertex alpha - 1)) (minus instance slot 20, which is otherwise a
    // primary light, 0 or more), so the texture's faint floor never shows
    // and the sprite dissolves as it fades.
    let dissolve = -min(in.primary.x, 0.0);
    var a = albedo.a * in.color.a * near;
    if (dissolve > 0.0) {
        a = saturate(dissolve * (albedo.a + in.color.a * near - 1.0));
    }
    // Nothing to blend where the sprite is clear (most of a puff's corners,
    // a near-faded sprite): stop before the depth read and the blend.
    if (a < 0.002) {
        discard;
    }
#ifdef SOFT
    // BO2's zfeather pixel shaders: the alpha fades as the sprite nears the
    // scene behind it, (scene depth - sprite depth) * featherParms.x.
    if (feather > 0.0) {
        a *= soft_fade(in.clip_position, in.world_position, feather);
        if (a < 0.002) {
            discard;
        }
    }
#endif
    // bo2zm M3: BO2's falloff effect shaders (`falloffParms`, from the fxc
    // disassembly): the colour times f * lerp(end, begin, f), f =
    // saturate(dot(eye ray, normal)^2 * parms.z + parms.w): a sprite (a
    // flame sheet, a zombie's eye beam) fades as it turns edge-on to the eye.
    // Instance slots 16..19 (see the extraction).
    var tint = in.color.rgb;
    if (in.light.w > 0.5) {
        let ray = normalize(in.world_position - view.world_position);
        let d = dot(ray, normalize(in.normal));
        let f = saturate(d * d * in.light.x + in.light.y);
        tint = tint * f * mix(vec3(in.light.z), vec3(1.0), f);
    }
#ifdef EFFECT_ADD
    // Additive ones take no fog (it would add the fog colour over the whole
    // quad); the blend scales the colour by the alpha.
    return vec4(bo2_added(texel * tint), a);
#else
#ifdef EFFECT_MULTIPLY
    return vec4(mix(vec3(1.0), texel * tint, a), a);
#else
    // Blended ones go toward the effect fog by its keep factor.
    let fx_fog = effect_fog(in.world_position);
    let fx = mix(fx_fog.rgb, texel * tint, fx_fog.w);
    return vec4(bo2_final(fx), a);
#endif
#endif
#else
#ifdef UNLIT
#ifdef MULTIPLY
    let a = albedo.a * in.color.a;
    return vec4(mix(vec3(1.0), texel * in.color.rgb, a), a);
#else
#ifdef SCROLL
    // A cloud sheet's colour is the texture times hdrAmount, squared
    // (instance primary.x holds hdrAmount squared), no vertex colour; the
    // vertex alpha fades it, and so does how near the scene behind it lies
    // (1 / FadeFactor, instance light.x).
    var sheet_a = albedo.a * in.color.a;
#ifdef SOFT
    sheet_a *= soft_fade(in.clip_position, in.world_position, in.light.x);
#endif
    if (sheet_a < 0.002) {
        discard;
    }
    return vec4(
        unlit_out(unlit_fog(texel * texel * in.primary.x, in.world_position)),
        sheet_a,
    );
#else
    // bo2zm round 1: a zombie's eyes (`unlit_cheap_zombie_eyes`): BO2's
    // shader scales the texture by the scriptVector2.y its client script set
    // (2 alive, Origins 3, 0.25 once the eyes go out) before squaring it;
    // it rides negated in instance slot 22, which no other unlit row makes
    // negative.
    var c = texel * in.color.rgb;
    if (in.primary.z < 0.0) {
        c *= -in.primary.z;
    }
    return vec4(
        unlit_out(unlit_fog(c * c * f32(#{UNLIT_SCALE}), in.world_position)),
        albedo.a * in.color.a,
    );
#endif
#endif
#else
    // The grid light is already times the exposure; the BO2 output adds it
    // again, so divide it out here.
    let exposure = max(t6.sun_dir_exposure.w, 0.000001);
    var light = in.light.rgb / exposure;
#ifdef FOLIAGE
    // bo2mp: Black Ops II's foliage (`mc_treecanopy`, fxc disassembly): the
    // vertex colour's rgb picks the wind variant and its alpha scales the
    // grid light; the leaves are their colour map alone.
    light *= in.color.a;
    let tinted = texel;
#else
    let tinted = texel * in.color.rgb;
#endif
    var base = tinted * tinted;
    if (in.light.w > 0.0) {
        light = in.color.rgb * in.color.rgb * 32.0;
        base = texel * texel;
    }
#ifdef SHINE
    // bo2mp: a pattern camo (instance slot 22 = 3000 + its repeat), as Black
    // Ops II's gun shader lays it (fxc disassembly of `mc_lit_sm_r0c0n0s0o0`'s
    // light-probe pass): the pattern at (uv - 0.5) * repeat + 0.5, under the
    // gun's colour by the colour map's alpha (alpha 0: all pattern). The
    // pattern rides in the specular map's slot here; the camo areas shine as
    // painted metal. (Sampled either way: texture reads stay in uniform
    // control flow.)
    let camo_on = in.primary.z >= 3000.0;
    let camo_uv = (in.uv - vec2(0.5)) * max(in.primary.z - 3000.0, 0.0001) + vec2(0.5);
    let camo_pat = textureSample(specular_map, colour_sampler, camo_uv);
    if (camo_on) {
        let camo_mixed = mix(raw_texel(camo_pat.rgb), texel, albedo.a) * in.color.rgb;
        base = camo_mixed * camo_mixed;
    }
    // bo2zm: Black Ops II's lit model shader (`mc_lit_sm_r0c0n0s0`, its
    // light-probe technique, from the fxc disassembly): as the world's, the
    // light here from the light grid. A model's vertices carry no tangent
    // frame here, so it comes from the screen-space derivatives of position
    // and texcoord (a cotangent frame).
    let vn = normalize(in.normal);
    let dp1 = dpdx(in.world_position);
    let dp2 = dpdy(in.world_position);
    let duv1 = dpdx(in.uv);
    let duv2 = dpdy(in.uv);
    let dp2perp = cross(dp2, vn);
    let dp1perp = cross(vn, dp1);
    let tg = dp2perp * duv1.x + dp1perp * duv2.x;
    let bt = dp2perp * duv1.y + dp1perp * duv2.y;
    let frame = inverseSqrt(max(max(dot(tg, tg), dot(bt, bt)), 1e-20));
    // bo2mp: Black Ops III's own light (`t7_shade`).
    if (t7_on()) {
        let nm7 = textureSample(normal_map, colour_sampler, in.uv).xy * 4.015748 - 2.015748;
        let n7 = normalize(nm7.x * tg * frame + nm7.y * bt * frame + vn);
        let st7 = textureSample(specular_map, colour_sampler, in.uv);
        let pr7 = u32(max(in.primary.w - 0.5, 0.0));
        return vec4(t7_out(t7_shade(base, light, vn, n7, st7, pr7, in.world_position)), albedo.a * in.color.a);
    }
#ifdef FLOW
    // bo2mp: a lava prop (`mc_cod7emissiveflow`), as the world's.
    return vec4(bo2_out(bo2_flow(in.uv, in.color, vn, tg * frame, bt * frame, in.world_position)), 1.0);
#endif
#ifdef TILE
    // bo2mp: Black Ops II's emissive tile (`sw4_3d_cod7_emissive_tile`,
    // Magma's lava rocks; its lit technique, fxc disassembly): the colour
    // map tiled by Micro_Scale times the vertex colour, squared, lit (here
    // the grid light, the primary light by the macro normal map times
    // Macro_Height; the micro normal map is not drawn), plus the glow
    // squared: (emissive A + emissive B) * A's alpha * hdrAmount, combined
    // by the assets lane in the specular map's place. Auxiliary texel 0:
    // Micro_Scale, Micro_Height, Macro_Height.
    let tc = textureLoad(aux_map, vec2<i32>(0, 0), 0);
    let tcol = textureSample(colour_map, colour_sampler, in.uv * tc.xy);
    let ttint = select(in.color.rgb, vec3(1.0), in.light.w > 0.0);
    let tbase = raw_texel(tcol.rgb) * ttint;
    let tnm = (textureSample(normal_map, colour_sampler, in.uv).xy * 4.015748 - 2.015748) * tc.w;
    let tn = normalize(tnm.x * tg * frame + tnm.y * bt * frame + vn);
    let tray = primary_light_ray(u32(in.primary.x + 0.5), in.world_position);
    var tlight = light + tray.colour * in.primary.y * saturate(dot(tn, tray.dir));
    tlight += dyn_lights(in.world_position, tn);
    let tglow = textureSample(specular_map, colour_sampler, in.uv).rgb;
    return vec4(
        bo2_out(bo2_fog(tbase * tbase * tlight + tglow * tglow, in.world_position)),
        1.0,
    );
#endif
#ifdef TILE_BLEND
    // bo2mp: Black Ops II's tile blend (`sw4_3d_cod7_tile_blend`, `_edge`,
    // `_spec`; cliffs, rocks, vista mountains; lit technique, fxc
    // disassembly): Micro_1 (colour map, at uv * Micro_1_Scale) and Micro_2
    // (specular map's place, at uv * Micro_2_Scale) mixed by the macro
    // map's green squared (normal map's place), times 1 + blue^2 *
    // (EdgeHighlight - 1) (the edge variant; 1 elsewhere), times the vertex
    // colour, squared; the ambient light times saturate(red^2 +
    // AO_Diffuse_Adj); the primary light by the vertex normal (the macro
    // and micro normal maps and the spec variant's specular are not drawn).
    let bc0 = textureLoad(aux_map, vec2<i32>(0, 0), 0);
    let bc1 = textureLoad(aux_map, vec2<i32>(1, 0), 0);
    let bm = raw_texel(textureSample(normal_map, colour_sampler, in.uv).rgb);
    let b1 = raw_texel(textureSample(colour_map, colour_sampler, in.uv * bc0.xy).rgb);
    let b2 = raw_texel(textureSample(specular_map, colour_sampler, in.uv * bc0.zw).rgb);
    let bedge = 1.0 + bm.z * bm.z * (bc1.y - 1.0);
    let btint = select(in.color.rgb, vec3(1.0), in.light.w > 0.0);
    let bcol = saturate(mix(b1, b2, bm.y * bm.y) * bedge) * btint;
    let bao = saturate(bm.x * bm.x + bc1.x);
    let bray = primary_light_ray(u32(in.primary.x + 0.5), in.world_position);
    var blight = light * bao + bray.colour * in.primary.y * saturate(dot(vn, bray.dir));
    blight += dyn_lights(in.world_position, vn);
    return vec4(bo2_out(bo2_fog(bcol * bcol * blight, in.world_position)), 1.0);
#endif
#ifdef FLAG
    // bo2mp: Black Ops II's tattered flag (`sw4_3d_phong_simple_flag_tatters`,
    // Dig's tarps; its lit technique, fxc disassembly): the frayed edge
    // texture (specular map's place, at uv * EdgeScale, auxiliary texel 0)
    // mixed into the diffuse by the vertex colour's red, squared; alpha
    // mixes the edge's alpha toward 1 by saturate(2 * red), tested at
    // 128/255. Lit by the grid and the primary light (its gloss, detail
    // normal and wind are not drawn).
    let gc = textureLoad(aux_map, vec2<i32>(0, 0), 0);
    let gedge = textureSample(specular_map, colour_sampler, in.uv * gc.xy);
    let gw = select(in.color.r, 1.0, in.light.w > 0.0);
    let ga = mix(gedge.a, 1.0, saturate(2.0 * gw));
    if (ga < 0.501961) {
        discard;
    }
    let gcol = mix(raw_texel(gedge.rgb), texel, gw);
    let gray = primary_light_ray(u32(in.primary.x + 0.5), in.world_position);
    var glight = light + gray.colour * in.primary.y * saturate(dot(vn, gray.dir));
    glight += dyn_lights(in.world_position, vn);
    return vec4(bo2_out(bo2_fog(gcol * gcol * glight, in.world_position)), 1.0);
#endif
    let nm = textureSample(normal_map, colour_sampler, in.uv).xy * 4.015748 - 2.015748;
    let n = normalize(nm.x * tg * frame + nm.y * bt * frame + vn);
    let spec_tex = select(
        textureSample(specular_map, colour_sampler, in.uv),
        vec4(0.24, 0.24, 0.24, 0.45),
        camo_on,
    );
    let spec_raw = raw_texel(spec_tex.rgb);
    let spec = spec_raw * spec_raw;
    let gloss = spec_tex.a;
    let vdir = normalize(in.world_position - view.world_position);
    // The reflection, when the model has a probe (instance slot 23, plus
    // one): scaled by the light here over the light the probe saw.
    let ndotv = saturate(dot(n, -vdir));
    let fk = gloss * vec4(1.041667, 0.475, 0.018229, 0.25) + vec4(0.0, 0.0, -0.015625, 0.75);
    var f = min(exp2(-9.28 * ndotv), fk.y);
    f = fk.x * f + fk.z;
    let env_weight = saturate(spec * (fk.w - f) + f);
    let r = vdir - 2.0 * dot(vdir, n) * n;
    let probe_plus = u32(in.primary.w + 0.5);
    let probe = select(0u, probe_plus - 1u, probe_plus > 0u);
    let env = textureSampleLevel(probe_cubes, probe_sampler, r, i32(probe), 4.0 - 4.0 * gloss);
    // bo2mp: a CoD4 / World at War map's own probes (sun colour alpha 4)
    // are lit already (gamma, no lighting table): squared, not relit.
    let own_probes = t6.sun_color.a > 3.5 && t6.sun_color.a < 4.5;
    let env_rgb = select(env.rgb / (env.a + 0.000001), env.rgb * env.rgb, own_probes);
    let pi = min(probe, 31u) * 3u;
    let s0 = probes.v[pi];
    let s1 = probes.v[pi + 1u];
    let s2 = probes.v[pi + 2u];
    let sh = dot(s2, vec4(vn.z * vn.x, vn.y * vn.z, vn.x * vn.y, vn.x * vn.x - vn.y * vn.y))
        + dot(s1, vec4(vn, 1.0)) + s0.w * vn.z * vn.z;
    let probe_light = max(sh * s0.xyz, vec3(0.1));
    let relit = select(light / probe_light, vec3(1.0), own_probes);
    let reflection = select(vec3(0.0), env_weight * env_rgb * relit, probe_plus > 0u);
    // The primary light: diffuse and its highlight.
    let ray = primary_light_ray(u32(in.primary.x + 0.5), in.world_position);
    // (bo2mp: a CoD4 / World at War sun is a gamma colour: its highlight
    // takes it squared.)
    let lit_g = ray.colour * in.primary.y;
    let lit = select(lit_g, lit_g * lit_g, t6.sun_color.a > 2.5 && t6.sun_color.a < 4.5);
    let ndotl = saturate(dot(n, ray.dir));
    light = add_light(light, lit_g, ndotl);
    light += dyn_lights(in.world_position, n);
    let h = normalize(ray.dir - vdir);
    let ndoth = saturate(dot(n, h));
    let ldoth = saturate(dot(h, ray.dir));
    let power = exp2(gloss * 13.0);
    let lobe = exp2(log2(max(ndoth, 0.000001)) * power) * (power * 0.125 + 0.25);
    let k = saturate(gloss + 0.545);
    let vis = ndotl / (ldoth * ldoth * k - k + 1.01);
    let fres = spec + (1.0 - spec) * exp2(-10.0 * ldoth);
    let highlight = fres * lobe * vis * lit;
    // The vertex alpha fades a blended model surface (a decal's painted
    // patches); opaque surfaces ignore the alpha.
    return vec4(
        bo2_out(bo2_fog(base * light + reflection + highlight, in.world_position)),
        albedo.a * in.color.a,
    );
#else
    // bo2mp: Black Ops III's own light (`t7_shade`).
    if (t7_on()) {
        let vn7 = normalize(in.normal);
        return vec4(t7_out(t7_shade(base, light, vn7, vn7, vec4(0.0), 0u, in.world_position)), albedo.a * in.color.a);
    }
    let n = normalize(in.normal);
    let pray = primary_light_ray(u32(in.primary.x + 0.5), in.world_position);
    light = add_light(light, pray.colour * in.primary.y, saturate(dot(n, pray.dir)));
    light += dyn_lights(in.world_position, n);
    return vec4(bo2_out(bo2_fog(base * light, in.world_position)), albedo.a * in.color.a);
#endif
#endif
#endif
#endif
#endif
#endif
}

// bo2zm: the Black Ops II sky: a full-screen triangle at the far plane
// (reverse-Z: depth 0, drawn only where nothing else is), sampling the
// sky's cube map by view direction as the game's skycubemaphdr shader does
// (world x, y, z; rgb / a), times its brightness, then the BO2 output (times
// the exposure, square-rooted). bo2mp: its fog as that shader applies it:
// the pixel shader works the fog out again and multiplies it by the vertex
// shader's fog times opacity, so the sky fogs by fog^2 * opacity. Its
// skyColorMultiplier (GfxWorld::skyDynIntensity, factors 1 on both
// Nuketowns) and colour matrix (identity) change nothing.
@group(1) @binding(3) var sky_map: texture_cube<f32>;
@group(1) @binding(4) var sky_sampler: sampler;
@group(1) @binding(5) var<uniform> sky_t6: T6Lighting;
// bo2zm: the light table again, for the sky fog (slots 256, 257).
@group(1) @binding(6) var<uniform> sky_lights: T6Lights;
// bo2zm: the world's colour tables, for the sky's final colour.
@group(1) @binding(12) var sky_lut: texture_2d<f32>;
@group(1) @binding(13) var sky_lut_sampler: sampler;

struct SkyOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

@vertex
fn vertex_sky(@builtin(vertex_index) index: u32) -> SkyOut {
    let x = f32((index & 1u) * 4u) - 1.0;
    let y = f32((index >> 1u) * 4u) - 1.0;
    var out: SkyOut;
    out.clip_position = vec4(x, y, 0.0, 1.0);
    out.ndc = vec2(x, y);
    return out;
}

@fragment
fn fragment_sky(in: SkyOut) -> @location(0) vec4<f32> {
    let near = view.world_from_clip * vec4(in.ndc, 1.0, 1.0);
    let dir = normalize(near.xyz / near.w - view.world_position);
    // The game's sky vertex shader turns the lookup about z:
    // x' = x * r.y + y * r.z, y' = x * r.x + y * r.y.
    let r = sky_t6.sky;
    let z = select(dir.z, -dir.z, r.w < 0.0);
    let lookup = vec3(dir.x * r.y + dir.y * r.z, dir.x * r.x + dir.y * r.y, z);
    let texel = textureSample(sky_map, sky_sampler, lookup);
    // bo2zm: its skyColorParm x picks how the cube is stored (above 0: rgb /
    // a, else rgb^2).
    let parms = sky_t6.sky_parms;
    let decoded = select(texel.rgb * texel.rgb, texel.rgb / (texel.a + 0.000001), parms.x > 0.0);
    let sky = decoded * abs(r.w);
    // The game puts the sky 2e7 units out and fogs it there, taking only
    // its skyColorParm w of the fog (1 on most maps, Buried 0.16).
    var fogged = sky;
    if (sky_lights.v[1028u].w >= 0.5) {
        let f = fog_at(
            view.world_position + dir * 20000000.0,
            sky_lights.v[1028u],
            sky_lights.v[1029u],
            sky_lights.v[1030u],
            sky_lights.v[1031u],
            sky_lights.v[1024u],
        );
        fogged = mix(sky, f.tint, f.fog * f.fog * f.opacity * parms.y);
    }
    // bo2mp: a Black Ops III sky (its picture in its probes' unit) through
    // the world's tone curve (`t7_tone`).
    if (sky_t6.sky_parms.z > 0.5) {
        let t7_x = bo2_rolloff(t7_tone(fogged, sky_lights.v[T7_ROW * 4u].x));
        return vec4(bo2_finish(t7_x, sky_t6, sky_lut, sky_lut_sampler), 1.0);
    }
    let sky_x = bo2_rolloff(sqrt(max(fogged * sky_t6.sun_dir_exposure.w, vec3(0.0))));
    return vec4(bo2_finish(sky_x, sky_t6, sky_lut, sky_lut_sampler), 1.0);
}
