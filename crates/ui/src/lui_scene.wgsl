#import bevy_ui::ui_vertex_output::UiVertexOutput

// bo2mp menus: the front end's 3D backdrop on the graphics card, as BO2 draws
// it (its pixel shaders over the quads): the holotable floor and the globe.
// The same sums as `lui_scene.rs` (its notes give the sources), once a pixel
// a frame; the textures are read raw, as stored (BO2's shaders work on the
// raw values).

// The game's clock (s), the root's width (units), the fade, 1 when the
// optional picture is there (floor: `tv_lookup`; globe: the dotted mesh).
@group(1) @binding(0) var<uniform> params: vec4<f32>;
// Floor: the small stars (`plus_tile`), `plus_grid3`, `plus_grid2`,
// `tv_lookup`. Globe: `globe_map`, `globe_map_mesh`, the last two unused.
@group(1) @binding(1) var tex0: texture_2d<f32>;
@group(1) @binding(2) var samp0: sampler;
@group(1) @binding(3) var tex1: texture_2d<f32>;
@group(1) @binding(4) var samp1: sampler;
@group(1) @binding(5) var tex2: texture_2d<f32>;
@group(1) @binding(6) var samp2: sampler;
@group(1) @binding(7) var tex3: texture_2d<f32>;
@group(1) @binding(8) var samp3: sampler;
// 0 the floor, 1 the globe; the globe's spin and tilt (degrees), its pad.
@group(1) @binding(9) var<uniform> extra: vec4<f32>;

const EYE: f32 = 1000.0;
const TILE: f32 = 80.0;
const GLOBE_U: f32 = -490.0;
const QUAD: f32 = 2560.0;
const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318531;

// C's `x - trunc(x)`.
fn sfrac(x: f32) -> f32 {
    return x - trunc(x);
}

// The picture's own turn, 25.2 degrees, about its middle.
fn turn(q: vec2<f32>) -> vec2<f32> {
    let c = 0.904827;
    let s = 0.425779;
    return vec2<f32>(c * (q.x - 0.5) + s * (q.y - 0.5) + 0.5, -s * (q.x - 0.5) + c * (q.y - 0.5) + 0.5);
}

fn tv_at(u: f32, v: f32) -> f32 {
    if (params.w < 0.5) {
        return 0.5;
    }
    return textureSampleLevel(tex3, samp3, vec2<f32>(u, v), 0.0).r;
}

// `sw4_2d_holotable_grid` (the small stars).
fn grid_ps(q: vec2<f32>, t: f32, dn: f32) -> vec3<f32> {
    let fy = sfrac(0.4 + 0.175 * t);
    let ring = clamp(dn - 2.0 * fy + 1.0, 0.0, 1.0);
    let g = 1.0 - min(12.0 * abs(ring - 0.5), 1.0);
    let off = g * 0.005;
    let bright = g * 0.1 + 0.25;
    let rq = turn(q);
    let l = tv_at(fract(0.5 + 0.25 * t), rq.y);
    let split = 2.0 * l * off;
    let sx = fract(t * 0.000625);
    let sy = fract(t * -0.00125);
    let y = (rq.y + sy) * 32.0;
    let rp = textureSampleLevel(tex0, samp0, vec2<f32>((rq.x + split + sx) * 32.0, y), 0.0);
    let gc = textureSampleLevel(tex0, samp0, vec2<f32>((rq.x + sx) * 32.0, y), 0.0);
    let bm = textureSampleLevel(tex0, samp0, vec2<f32>((rq.x - split + sx) * 32.0, y), 0.0);
    let base = vec3<f32>(rp.r, gc.g, bm.b) * bright;
    let tri = 4.0 - 2.0 * abs(2.0 * abs(sfrac(0.175 * t)) - 1.0);
    let k = 0.5 * (1.0 - clamp(tri * min(dn, 1.0), 0.0, 1.0));
    return base + k * (vec3<f32>(1.0, 0.5, 0.0) - base);
}

fn grid2_sample(which: i32, uv: vec2<f32>) -> vec4<f32> {
    if (which == 1) {
        return textureSampleLevel(tex1, samp1, uv, 0.0);
    }
    return textureSampleLevel(tex2, samp2, uv, 0.0);
}

// `sw4_2d_holotable_grid2` (`_grid2` and `_grid3`: the orange crosses).
fn grid2_ps(q: vec2<f32>, t: f32, which: i32) -> vec3<f32> {
    let pulse = 0.75 + 1.25 * abs(2.0 * abs(sfrac(0.175 * t)) - 1.0);
    let rq = turn(q);
    let tc = vec2<f32>((rq.x + fract(t * 0.00125)) * 8.0, (rq.y + fract(t * -0.0025)) * 8.0);
    let l = tv_at(fract(0.25 * t + 0.5), tc.y);
    let split = 0.01 * (l * l) * (l * l) * 8.0;
    let c = grid2_sample(which, tc);
    let xp = grid2_sample(which, vec2<f32>(tc.x + split, tc.y));
    let xm = grid2_sample(which, vec2<f32>(tc.x - split, tc.y));
    return vec3<f32>(xp.r, xm.g, c.b) * pulse;
}

// The holotable floor over the whole root (`w` x 720 units): the three
// grids added up (additive in BO2), their brightness as alpha.
fn floor_at(uv: vec2<f32>) -> vec4<f32> {
    let t = params.x;
    let w = params.y;
    let s = sin(radians(80.0));
    let c = cos(radians(80.0));
    let sx = uv.x * w - w * 0.5;
    let sy = uv.y * 720.0 - 360.0;
    var rgb = vec3<f32>(0.0);
    // (the small stars; `_grid3` at half; `_grid2` 32 units lower)
    for (var layer = 0; layer < 3; layer++) {
        var tint = 1.0;
        var dy = 0.0;
        if (layer == 1) {
            tint = 0.5;
        }
        if (layer == 2) {
            dy = -32.0;
        }
        let v = EYE * (sy - (400.0 + dy)) / (sy * s + c * EYE);
        let depth = EYE - v * s;
        if (depth <= 1.0 || abs(v) > 1280.0) {
            continue;
        }
        let u = sx * depth / EYE;
        if (abs(u) > 1280.0) {
            continue;
        }
        let near = clamp((v + 1280.0) / 1280.0, 0.0, 1.0);
        let fade = near * near * (3.0 - 2.0 * near);
        let du = (u - GLOBE_U) / QUAD;
        let dv = v / QUAD;
        let dn = sqrt(du * du + dv * dv) * 1.41421356;
        let q = vec2<f32>(u / (TILE * 32.0) + 0.5, v / (TILE * 32.0) + 0.5);
        var col: vec3<f32>;
        if (layer == 0) {
            col = grid_ps(q, t, dn);
        } else {
            col = grid2_ps(q, t, layer);
        }
        rgb += col * tint * fade;
    }
    let a = min(max(rgb.r, max(rgb.g, rgb.b)), 1.0);
    if (a <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(min(rgb / a, vec3<f32>(1.0)), a);
}

// The glow's strength (0..1) at radius `r` (the disc's radius is 1).
fn glow_alpha(r: f32) -> f32 {
    let g = clamp(1.0 - (r - 1.0) / 0.09, 0.0, 1.0);
    return pow(g, 1.3) * 0.11;
}

// The globe: the map on a sphere tilted toward us, turned by the spin, the
// mesh's dots in the sea, a light rim and a soft glow past its limb. `px` is
// one screen pixel in the disc's units.
fn globe_at(uv: vec2<f32>, px: f32) -> vec4<f32> {
    let pad = extra.w;
    let nx = uv.x * 2.0 * pad - pad;
    let ny = pad - uv.y * 2.0 * pad;
    let r2 = nx * nx + ny * ny;
    let r = sqrt(r2);
    let gcol = vec3<f32>(1.0, 247.0 / 255.0, 235.0 / 255.0);
    if (r > 1.0 + px) {
        return vec4<f32>(gcol, glow_alpha(r));
    }
    let edge = clamp((1.0 + px - r) / (2.0 * px), 0.0, 1.0);
    let nz = sqrt(1.0 - min(r2, 1.0));
    let ts = sin(radians(extra.z));
    let tc = cos(radians(extra.z));
    let gy = ny * tc + nz * ts;
    let gz = -ny * ts + nz * tc;
    let lat = asin(clamp(gy, -1.0, 1.0));
    let lon = atan2(nx, gz) + radians(extra.y);
    let u = lon / TAU + 0.5;
    let v = 0.5 - lat / PI;
    let m = textureSampleLevel(tex0, samp0, vec2<f32>(u, v), 0.0);
    var d = vec4<f32>(0.0);
    if (params.w > 0.5) {
        d = textureSampleLevel(tex1, samp1, vec2<f32>(u, v), 0.0);
    }
    let land = m.a;
    let dots = min(nz * 4.0, 1.0);
    let lit = clamp(0.4 + 0.9 * (0.3 * nx - 0.6 * ny + 0.75 * nz), 0.0, 1.0);
    let cell = 0.75 + 0.8 * clamp(d.r / 0.16, 0.0, 1.0);
    let heat = vec3<f32>(1.0, 0.62, 0.2);
    let ring = pow(clamp((0.32 - nz) / 0.32, 0.0, 1.0), 1.1) * 0.14;
    let warm = vec3<f32>(1.0, 0.97, 0.92);
    let sea = d.rgb * dots * 0.9 + heat * 0.09 * lit * cell;
    var rgb = min(sea * (1.0 - land) + m.rgb * land + ring * warm, vec3<f32>(1.0));
    let ga = glow_alpha(r);
    let a = ga + (1.0 - ga) * edge;
    if (a > 0.0 && edge < 1.0) {
        rgb = min((rgb * edge + gcol * ga * (1.0 - edge)) / max(edge + ga * (1.0 - edge), 1e-4), vec3<f32>(1.0));
    }
    return vec4<f32>(rgb, a);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let px = max(abs(dpdx(in.uv.x)) + abs(dpdy(in.uv.x)), 1e-5) * 2.0 * extra.w;
    var c: vec4<f32>;
    if (extra.x > 0.5) {
        c = globe_at(in.uv, px);
    } else {
        c = floor_at(in.uv);
    }
    return vec4<f32>(c.rgb, c.a * params.z);
}
