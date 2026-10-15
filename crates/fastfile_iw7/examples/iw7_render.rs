//! A software render of a map, to check how it reads and lights:
//! `iw7_render <map.ff> <out.png> [view]`, view one of
//! - `top`: straight down over the spawn points' area (orthographic),
//! - `spawn[:n]`: from spawn point n (default 0), at eye height,
//! - `cam:x,y,z,yaw,pitch`: from a point (degrees).
//!
//! The world is drawn as the game's fallback pass draws an Infinite Warfare
//! map (its IW3 branch in linear mode, `fastfile_iw7::light`): the colour
//! map squared (vertex colour: layer weights), times the page's base light
//! plus its directional light by the surface's facing plus the sun by N.L
//! and the sun shadow's threshold; the frame holds the rolled-off square
//! root, blended surfaces mix there, and the final pass looks the frame up
//! in the grade's table (IW7's tone map, sRGB and the vision's colour
//! table: `light::grade_lut`). The placed models take the light under them
//! (`Iw7Map::static_model_light`) plus the sun.
//!
//! Settings (environment): `IW7_RENDER_TEX=<px>` the colour maps' largest
//! level (default 512; each triangle takes the level its screen size
//! wants), `IW7_RENDER_SIZE=WxH`, `IW7_RENDER_EV=<stops>` and
//! `IW7_RENDER_K=<k>` the exposure (default the vision's and
//! `light::EXPOSURE_K`), `IW7_RENDER_NOLIGHT=1` colour maps flat,
//! `IW7_RENDER_NOGRADE=1` without the colour table, `IW7_RENDER_PICK=x,y;..`
//! what is under those pixels, `IW7_RENDER_REF=<png>` a reference picture
//! to compare the frame's mean colour and brightness with.

use fastfile_iw7::light::{self, LightSample, LightmapPage, Vision};
use fastfile_iw7::map::{Iw7Map, MapSources};
use fastfile_iw7::texture;
use std::collections::HashMap;
use std::path::Path;

/// A colour map's mip chain, texels as raw 0..1 values (what the game's
/// shaders square).
struct Tex {
    levels: Vec<(usize, usize, Vec<[f32; 4]>)>,
}

impl Tex {
    fn from_rgba(r: &fastfile_iw7::look::Rgba) -> Self {
        let to = |b: u8| f32::from(b) / 255.0;
        let mut levels = Vec::new();
        let (mut w, mut h) = (r.width as usize, r.height as usize);
        for l in &r.levels {
            levels.push((
                w,
                h,
                l.chunks_exact(4)
                    .map(|p| [to(p[0]), to(p[1]), to(p[2]), to(p[3])])
                    .collect(),
            ));
            (w, h) = ((w / 2).max(1), (h / 2).max(1));
        }
        Tex { levels }
    }
    fn size(&self) -> (usize, usize) {
        (self.levels[0].0, self.levels[0].1)
    }
    /// Bilinear at a level.
    fn sample(&self, uv: [f32; 2], level: usize) -> [f32; 4] {
        let (w, h, px) = &self.levels[level.min(self.levels.len() - 1)];
        let (w, h) = (*w, *h);
        let fx = uv[0].rem_euclid(1.0) * w as f32 - 0.5;
        let fy = uv[1].rem_euclid(1.0) * h as f32 - 0.5;
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let wrap = |v: f32, n: usize| (v as i64).rem_euclid(n as i64) as usize;
        let (x0i, y0i, x1i, y1i) = (
            wrap(x0, w),
            wrap(y0, h),
            wrap(x0 + 1.0, w),
            wrap(y0 + 1.0, h),
        );
        let a = px[y0i * w + x0i];
        let b = px[y0i * w + x1i];
        let c = px[y1i * w + x0i];
        let d = px[y1i * w + x1i];
        std::array::from_fn(|k| {
            (a[k] * (1.0 - tx) + b[k] * tx) * (1.0 - ty) + (c[k] * (1.0 - tx) + d[k] * tx) * ty
        })
    }
}

/// How a material draws, by its technique set's words.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Hidden,
    Opaque,
    AlphaTest,
    Blend,
    Add,
}

/// Layers 1 and 2 of a technique set: 0 none, 1 blend (`b1c1`, `t1c1`,
/// `a1c1`), 2 multiply (`m1c1`).
fn layer_kinds(techset: &str) -> [u8; 2] {
    let mut out = [0u8; 2];
    for t in techset.split('_') {
        let b = t.as_bytes();
        if b.len() >= 3
            && matches!(b[0], b'b' | b'm' | b'a' | b't')
            && (b'1'..=b'2').contains(&b[1])
            && b[2] == b'c'
        {
            out[usize::from(b[1] - b'1')] = if b[0] == b'm' { 2 } else { 1 };
        }
    }
    out
}

fn kind_of(techset: &str) -> (Kind, bool) {
    use fastfile_iw7::look::{Blend, look};
    let l = look(techset);
    if l.hidden.is_some() {
        return (Kind::Hidden, l.unlit);
    }
    let k = match l.blend {
        Blend::Add => Kind::Add,
        Blend::Blend | Blend::Multiply => Kind::Blend,
        Blend::Opaque if l.alpha_test => Kind::AlphaTest,
        Blend::Opaque => Kind::Opaque,
    };
    (k, l.unlit)
}

#[derive(Clone, Copy)]
struct V {
    p: [f32; 3],
    uv: [f32; 2],
    lm: [f32; 2],
    n: [f32; 3],
    a: f32,
    /// Layer 1 and 2 weights (a layered world surface's vertex colour).
    w: [f32; 2],
}

fn lerp(a: &V, b: &V, t: f32) -> V {
    let l = |x: f32, y: f32| x + (y - x) * t;
    V {
        p: std::array::from_fn(|k| l(a.p[k], b.p[k])),
        uv: std::array::from_fn(|k| l(a.uv[k], b.uv[k])),
        lm: std::array::from_fn(|k| l(a.lm[k], b.lm[k])),
        n: std::array::from_fn(|k| l(a.n[k], b.n[k])),
        a: l(a.a, b.a),
        w: std::array::from_fn(|k| l(a.w[k], b.w[k])),
    }
}

struct Cam {
    eye: [f32; 3],
    f: [f32; 3],
    r: [f32; 3],
    u: [f32; 3],
    ortho: Option<f32>,
    focal: f32,
    w: usize,
    h: usize,
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt().max(1e-9);
    a.map(|x| x / l)
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

impl Cam {
    /// View space: x right, y up, z forward.
    fn view(&self, p: [f32; 3]) -> [f32; 3] {
        let d = sub(p, self.eye);
        [dot(d, self.r), dot(d, self.u), dot(d, self.f)]
    }
    fn project(&self, v: [f32; 3]) -> [f32; 3] {
        let (w, h) = (self.w as f32, self.h as f32);
        match self.ortho {
            Some(half) => {
                let s = (h * 0.5) / half;
                [w * 0.5 + v[0] * s, h * 0.5 - v[1] * s, v[2]]
            }
            None => [
                w * 0.5 + v[0] / v[2] * self.focal,
                h * 0.5 - v[1] / v[2] * self.focal,
                v[2],
            ],
        }
    }
}

struct Target {
    /// The frame as the game holds it: the rolled-off square root of the
    /// light (`light::bo2_rolloff`).
    frame: Vec<[f32; 3]>,
    depth: Vec<f32>,
    /// Per pixel: (material + 1, placed model + 1) of what was drawn there.
    ids: Vec<(u32, u32)>,
    /// Per pixel: the HDR luminance (before the exposure), for metering.
    hdr: Vec<f32>,
}

/// The light and fog a pass draws with.
struct Scene<'a> {
    sun_dir: [f32; 3],
    /// The sun's linear colour in the draw's units.
    sun: [f32; 3],
    lit: bool,
    fog: Option<Fog>,
    eye: [f32; 3],
    pages: &'a [Option<LightmapPage>],
    /// The draw's scale (HDR light to the draw's units).
    scale: f32,
}

#[derive(Clone, Copy)]
struct Fog {
    start: f32,
    half: f32,
    opacity: f32,
    /// Linear, the draw's units.
    colour: [f32; 3],
}

impl Fog {
    fn apply(&self, x: [f32; 3], d: f32) -> [f32; 3] {
        let f = (1.0 - (-((d - self.start).max(0.0)) / self.half).exp2()) * self.opacity;
        std::array::from_fn(|k| x[k] + (self.colour[k] - x[k]) * f)
    }
}

/// What a surface draws with.
struct Shade<'a> {
    tex: Option<&'a Tex>,
    kind: Kind,
    unlit: bool,
    page: Option<&'a LightmapPage>,
    /// For models: the light under them (None: lit by the page).
    ground: Option<LightSample>,
    /// Layer 1 and 2 colour maps (opacity in alpha) and whether each
    /// multiplies (else blends) by its weight.
    layers: [Option<(&'a Tex, bool)>; 2],
    /// (material + 1, placed model + 1), for `IW7_RENDER_PICK`.
    id: (u32, u32),
    /// A model material's own light (`fastfile_iw7::glow`), linear HDR.
    glow: Option<&'a Tex>,
}

fn draw_tri(cam: &Cam, t: &mut Target, tri: [V; 3], sh: &Shade, sc: &Scene) {
    // Into view space, clipped to the near plane.
    let near = if cam.ortho.is_some() { -1.0e9 } else { 4.0 };
    let mut poly: Vec<(V, [f32; 3])> = tri.iter().map(|v| (*v, cam.view(v.p))).collect();
    if cam.ortho.is_none() {
        let mut out = Vec::with_capacity(4);
        for i in 0..poly.len() {
            let (a, b) = (&poly[i], &poly[(i + 1) % poly.len()]);
            let (ina, inb) = (a.1[2] >= near, b.1[2] >= near);
            if ina {
                out.push(*a);
            }
            if ina != inb {
                let s = (near - a.1[2]) / (b.1[2] - a.1[2]);
                let v = lerp(&a.0, &b.0, s);
                out.push((v, cam.view(v.p)));
            }
        }
        poly = out;
    }
    if poly.len() < 3 {
        return;
    }
    // The colour map's level: texels per pixel over the triangle.
    let level = sh.tex.map_or(0, |tx| {
        let s: Vec<[f32; 3]> = poly.iter().map(|(_, v)| cam.project(*v)).collect();
        let mut screen = 0.0;
        let mut texels = 0.0;
        let (tw, th) = tx.size();
        for k in 1..poly.len() - 1 {
            let (a, b, c) = (s[0], s[k], s[k + 1]);
            screen += ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() * 0.5;
            let (ua, ub, uc) = (poly[0].0.uv, poly[k].0.uv, poly[k + 1].0.uv);
            texels += ((ub[0] - ua[0]) * (uc[1] - ua[1]) - (uc[0] - ua[0]) * (ub[1] - ua[1])).abs()
                * 0.5
                * (tw * th) as f32;
        }
        if screen < 1e-6 {
            tx.levels.len() - 1
        } else {
            (0.5 * (texels / screen).max(1.0).log2()).floor() as usize
        }
    });
    for k in 1..poly.len() - 1 {
        raster(cam, t, [poly[0], poly[k], poly[k + 1]], sh, sc, level);
    }
}

fn raster(
    cam: &Cam,
    t: &mut Target,
    tri: [(V, [f32; 3]); 3],
    sh: &Shade,
    sc: &Scene,
    level: usize,
) {
    let s = tri.map(|(_, v)| cam.project(v));
    let area =
        (s[1][0] - s[0][0]) * (s[2][1] - s[0][1]) - (s[2][0] - s[0][0]) * (s[1][1] - s[0][1]);
    if area.abs() < 1e-8 {
        return;
    }
    let minx = s
        .iter()
        .map(|p| p[0])
        .fold(f32::MAX, f32::min)
        .floor()
        .max(0.0) as usize;
    let maxx = s
        .iter()
        .map(|p| p[0])
        .fold(f32::MIN, f32::max)
        .ceil()
        .min(cam.w as f32 - 1.0);
    let miny = s
        .iter()
        .map(|p| p[1])
        .fold(f32::MAX, f32::min)
        .floor()
        .max(0.0) as usize;
    let maxy = s
        .iter()
        .map(|p| p[1])
        .fold(f32::MIN, f32::max)
        .ceil()
        .min(cam.h as f32 - 1.0);
    if maxx < 0.0 || maxy < 0.0 {
        return;
    }
    let (maxx, maxy) = (maxx as usize, maxy as usize);
    // Perspective: interpolate attr / z and 1 / z.
    let iz = tri.map(|(_, v)| if cam.ortho.is_some() { 1.0 } else { 1.0 / v[2] });
    for y in miny..=maxy {
        for x in minx..=maxx {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = ((s[1][0] - px) * (s[2][1] - py) - (s[2][0] - px) * (s[1][1] - py)) / area;
            let w1 = ((s[2][0] - px) * (s[0][1] - py) - (s[0][0] - px) * (s[2][1] - py)) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let z = if cam.ortho.is_some() {
                w0 * tri[0].1[2] + w1 * tri[1].1[2] + w2 * tri[2].1[2]
            } else {
                1.0 / (w0 * iz[0] + w1 * iz[1] + w2 * iz[2])
            };
            let o = y * cam.w + x;
            if z >= t.depth[o] {
                continue;
            }
            let b = [w0 * iz[0] * z, w1 * iz[1] * z, w2 * iz[2] * z];
            let at = |f: &dyn Fn(&V) -> f32| {
                b[0] * f(&tri[0].0) + b[1] * f(&tri[1].0) + b[2] * f(&tri[2].0)
            };
            let uv = [at(&|v| v.uv[0]), at(&|v| v.uv[1])];
            let mut texel = sh
                .tex
                .map_or([0.6, 0.6, 0.6, 1.0], |tx| tx.sample(uv, level));
            for (k, l) in sh.layers.iter().enumerate() {
                let Some((lt, multiply)) = l else { continue };
                let wgt = at(&|v| v.w[k]);
                let lx = lt.sample(uv, level);
                for c in 0..3 {
                    texel[c] = if *multiply {
                        texel[c] * (1.0 + wgt * (lx[c] - 1.0))
                    } else {
                        texel[c] + (lx[c] - texel[c]) * lx[3] * wgt
                    };
                }
            }
            if matches!(sh.kind, Kind::AlphaTest) && texel[3] < 0.501_961 {
                continue;
            }
            let p = [at(&|v| v.p[0]), at(&|v| v.p[1]), at(&|v| v.p[2])];
            let n = norm([at(&|v| v.n[0]), at(&|v| v.n[1]), at(&|v| v.n[2])]);
            let base: [f32; 3] = std::array::from_fn(|k| texel[k] * texel[k]);
            let ndl = dot(n, sc.sun_dir).max(0.0);
            let light: [f32; 3] = if !sc.lit || sh.unlit {
                [1.0; 3]
            } else if let Some(g) = sh.ground {
                std::array::from_fn(|k| {
                    g.base[k] + g.directional[k] * 0.75 + sc.sun[k] * g.sun * ndl
                })
            } else if let Some(pg) = sh.page {
                let l = pg.sample([at(&|v| v.lm[0]), at(&|v| v.lm[1])]);
                std::array::from_fn(|k| {
                    l.base[k] + l.directional[k] * l.facing + sc.sun[k] * l.sun * ndl
                })
            } else {
                [1.0; 3]
            };
            let g = sh.glow.map_or([0.0; 4], |t| t.sample(uv, level));
            let mut lit: [f32; 3] = std::array::from_fn(|k| (base[k] * light[k] + g[k] * sc.scale).max(0.0));
            if let Some(f) = sc.fog {
                let d = dot(sub(p, sc.eye), sub(p, sc.eye)).sqrt();
                lit = f.apply(lit, d);
            }
            let out = lit.map(|v| light::bo2_rolloff(v.sqrt()));
            let hdr = (0.2126 * lit[0] + 0.7152 * lit[1] + 0.0722 * lit[2]) / sc.scale;
            match sh.kind {
                Kind::Blend => {
                    let a = texel[3] * at(&|v| v.a);
                    for k in 0..3 {
                        t.frame[o][k] = t.frame[o][k] * (1.0 - a) + out[k] * a;
                    }
                    t.hdr[o] = t.hdr[o] * (1.0 - a) + hdr * a;
                }
                Kind::Add => {
                    for k in 0..3 {
                        t.frame[o][k] =
                            (t.frame[o][k] + texel[k] * texel[3] * at(&|v| v.a)).min(1.0);
                    }
                }
                _ => {
                    t.frame[o] = out;
                    t.hdr[o] = hdr;
                    t.depth[o] = z;
                    t.ids[o] = sh.id;
                }
            }
        }
    }
}

/// A sky cube's faces at a small level, linear HDR.
struct SkyCube {
    size: usize,
    faces: Vec<Vec<[f32; 3]>>,
}

impl SkyCube {
    /// Direct3D's cube face and coordinates for a direction.
    fn sample(&self, d: [f32; 3]) -> [f32; 3] {
        let (ax, ay, az) = (d[0].abs(), d[1].abs(), d[2].abs());
        let (face, u, v) = if ax >= ay && ax >= az {
            if d[0] > 0.0 {
                (0, -d[2] / ax, -d[1] / ax)
            } else {
                (1, d[2] / ax, -d[1] / ax)
            }
        } else if ay >= az {
            if d[1] > 0.0 {
                (2, d[0] / ay, d[2] / ay)
            } else {
                (3, d[0] / ay, -d[2] / ay)
            }
        } else if d[2] > 0.0 {
            (4, d[0] / az, -d[1] / az)
        } else {
            (5, -d[0] / az, -d[1] / az)
        };
        let n = self.size;
        let x = (((u + 1.0) * 0.5 * n as f32) as usize).min(n - 1);
        let y = (((v + 1.0) * 0.5 * n as f32) as usize).min(n - 1);
        self.faces[face][y * n + x]
    }
}

/// The map's sky cube (its sky material's colour map, not a lighting
/// reference: as the game's lane picks it), BC6H, at its level of at most
/// 128 texels a side.
fn sky_cube(map: &Iw7Map) -> Option<SkyCube> {
    let name = map
        .materials
        .iter()
        .filter(|m| fastfile_iw7::look::look(&m.technique_set).hidden == Some("sky"))
        .filter_map(|m| m.texture("colorMap"))
        .find(|n| !n.contains("lighting_ref"))?;
    let l = map.image(name, 4096).ok()?;
    if !matches!(l.format, 95 | 96) || l.width != l.height {
        return None;
    }
    let face_bytes: usize = (0..l.levels)
        .map(|k| {
            texture::level_size(l.format, (l.width >> k).max(1), (l.height >> k).max(1))
                .unwrap_or(0)
        })
        .sum();
    if l.data.len() < 6 * face_bytes {
        return None;
    }
    let mut level = 0;
    while (l.width >> level) > 128 && level + 1 < l.levels {
        level += 1;
    }
    let skip: usize = (0..level)
        .map(|k| {
            texture::level_size(l.format, (l.width >> k).max(1), (l.height >> k).max(1))
                .unwrap_or(0)
        })
        .sum();
    let n = (l.width >> level).max(4) as usize;
    let mut faces = Vec::new();
    let mut tmp = [0f32; 48];
    for f in 0..6 {
        let at = f * face_bytes + skip;
        let bw = n.div_ceil(4);
        let mut px = vec![[0f32; 3]; n * n];
        for (i, b) in l.data[at..at + bw * bw * 16].chunks_exact(16).enumerate() {
            bcdec_rs::bc6h_float(b, &mut tmp, 12, l.format == 96);
            let (bx, by) = (i % bw * 4, i / bw * 4);
            for y in 0..4 {
                for x in 0..4 {
                    if bx + x < n && by + y < n {
                        let t = (y * 4 + x) * 3;
                        px[(by + y) * n + bx + x] = [tmp[t], tmp[t + 1], tmp[t + 2]]
                            .map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 });
                    }
                }
            }
        }
        faces.push(px);
    }
    Some(SkyCube { size: n, faces })
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A PNG's RGB as 0..1.
fn read_png(path: &str) -> Option<(usize, usize, Vec<[f32; 3]>)> {
    let dec = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).ok()?));
    let mut r = dec.read_info().ok()?;
    let mut buf = vec![0; r.output_buffer_size()?];
    let info = r.next_frame(&mut buf).ok()?;
    let ch = info.color_type.samples();
    let px = buf[..info.buffer_size()]
        .chunks_exact(ch)
        .map(|p| [0, 1, 2].map(|k| f32::from(p[k.min(ch - 1)]) / 255.0))
        .collect();
    Some((info.width as usize, info.height as usize, px))
}

/// Mean display colour, mean luminance and its spread (display values).
fn stats(px: &[[f32; 3]]) -> ([f32; 3], f32, f32) {
    let n = px.len().max(1) as f32;
    let mean: [f32; 3] = std::array::from_fn(|k| px.iter().map(|p| p[k]).sum::<f32>() / n);
    let lum = |p: &[f32; 3]| 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
    let ml = px.iter().map(lum).sum::<f32>() / n;
    let sd = (px.iter().map(|p| (lum(p) - ml).powi(2)).sum::<f32>() / n).sqrt();
    (mean, ml, sd)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let started = std::time::Instant::now();
    let map = Iw7Map::load(&{
        let src = MapSources::beside(Path::new(&args[1]));
        if std::env::var("IW7_NO_COMMON").is_ok() {
            src
        } else {
            src.with_common()
        }
    })
    .expect("map");
    let out_path = &args[2];
    let view = args.get(3).map(String::as_str).unwrap_or("top");
    let env_f = |k: &str| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
    };
    let tex_size: u32 = std::env::var("IW7_RENDER_TEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(512);
    let lit = std::env::var("IW7_RENDER_NOLIGHT").is_err();
    let (w, h) = std::env::var("IW7_RENDER_SIZE")
        .ok()
        .and_then(|v| {
            v.split_once('x')
                .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
        })
        .unwrap_or((1280usize, 720usize));

    // The vision: exposure, tone map and colour table, as the game's lane
    // reads them.
    let mut vision: Vision = map.vision_params();
    if let Some(ev) = env_f("IW7_RENDER_EV") {
        vision.ev = ev;
    }
    let k = env_f("IW7_RENDER_K").unwrap_or(light::EXPOSURE_K);
    let scale = k * (-vision.ev).exp2() / light::DRAW_UNIT;
    let clut = map.colour_table(&vision);
    let grade = (std::env::var("IW7_RENDER_NOGRADE").is_err())
        .then(|| light::grade_lut(&vision, clut.as_deref()));
    println!(
        "vision: ev {} (auto {}), K {k}, draw scale {scale:.5}, tone map {:?}, colour table {} {}",
        vision.ev,
        vision.auto,
        (
            vision.black,
            vision.white,
            vision.crossover,
            vision.shoulder,
            vision.toe
        ),
        vision.clut.as_deref().unwrap_or("-"),
        if clut.is_some() { "read" } else { "not read" }
    );

    let pages: Vec<Option<LightmapPage>> = (0..map.light_maps.len())
        .map(|i| match map.lightmap_page(i, 2048, scale) {
            Ok(p) => Some(p),
            Err(e) => {
                println!("lightmap {i}: {e}");
                None
            }
        })
        .collect();
    println!(
        "lightmaps read at {:.1}s: mean HDR light {:?}",
        started.elapsed().as_secs_f32(),
        pages
            .iter()
            .flatten()
            .map(|p| p.mean_light)
            .collect::<Vec<_>>()
    );

    // The sun: the ComWorld's (HDR) in the draw's units.
    let (sun_dir, sun) = match map.lights.iter().find(|l| l.kind == 1) {
        Some(l) => (norm(l.dir), l.color.map(|c| c * scale)),
        None => (norm([0.3, 0.3, 0.9]), [0.0; 3]),
    };
    println!("sun toward {sun_dir:?} colour {sun:?} (draw units)");

    // The fog, as the lane makes it.
    let fog = map.vision.as_ref().and_then(|(_, text)| {
        let v = |k: &str| light::vision_value(text, k);
        let one = |k: &str| v(k).and_then(|s| s.split_whitespace().next()?.parse::<f32>().ok());
        let half = one("fogDensity")?;
        let colour: Vec<f32> = v("fogColor")?
            .split_whitespace()
            .filter_map(|x| x.parse().ok())
            .collect();
        let intensity = one("fogColorIntensity").unwrap_or(1.0);
        (half > 0.0 && colour.len() >= 3).then(|| Fog {
            start: one("fogStartDist").unwrap_or(0.0),
            half,
            opacity: one("fogMaxOpacity").unwrap_or(1.0).clamp(0.0, 1.0),
            colour: [0, 1, 2].map(|k| srgb_to_linear(colour[k]) * intensity * scale),
        })
    });

    // Colour maps by material (and layered materials' layers), read on
    // every core.
    let used: Vec<bool> = {
        let mut u = vec![false; map.materials.len()];
        for s in &map.surfaces {
            if let Some(m) = s.material {
                u[m] = true;
            }
        }
        for sm in &map.static_models {
            for s in &map.models[sm.model].surfaces {
                if let Some(m) = s.material {
                    u[m] = true;
                }
            }
        }
        u
    };
    type Read = (Option<Tex>, [Option<Tex>; 2]);
    let read_one = |m: usize| -> Read {
        let mm = &map.materials[m];
        if !used[m] {
            return (None, [None, None]);
        }
        let base = mm.texture("colorMap").and_then(|name| {
            let keep = fastfile_iw7::look::colour_alpha_is_opacity(name);
            map.colour_rgba(mm, tex_size, keep)
                .ok()
                .map(|r| Tex::from_rgba(&r))
        });
        let layers = layer_kinds(&mm.technique_set);
        let ls = std::array::from_fn(|k| {
            if layers[k] == 0 || std::env::var("IW7_RENDER_NOLAYERS").is_ok() {
                return None;
            }
            let keep = mm
                .texture(&format!("colorMap{}", k + 1))
                .is_some_and(fastfile_iw7::look::colour_alpha_is_opacity);
            map.layer_rgba(mm, k + 1, tex_size, keep)
                .ok()
                .map(|r| Tex::from_rgba(&r))
        });
        (base, ls)
    };
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: std::sync::Mutex<Vec<(usize, Read)>> = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..std::thread::available_parallelism()
            .map_or(4, |n| n.get())
            .min(12)
        {
            scope.spawn(|| {
                loop {
                    let m = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if m >= map.materials.len() {
                        break;
                    }
                    let r = read_one(m);
                    results.lock().unwrap().push((m, r));
                }
            });
        }
    });
    let mut texs: HashMap<usize, Option<Tex>> = HashMap::new();
    let mut layered_tex: Vec<[Option<Tex>; 2]> =
        (0..map.materials.len()).map(|_| [None, None]).collect();
    for (m, (base, ls)) in results.into_inner().unwrap() {
        texs.insert(m, base);
        layered_tex[m] = ls;
    }
    let layer_texs: Vec<[Option<(&Tex, bool)>; 2]> = map
        .materials
        .iter()
        .zip(&layered_tex)
        .map(|(mm, t)| {
            let kinds = layer_kinds(&mm.technique_set);
            std::array::from_fn(|k| t[k].as_ref().map(|t| (t, kinds[k] == 2)))
        })
        .collect();
    println!("textures read at {:.1}s", started.elapsed().as_secs_f32());
    // Model materials' glow (`fastfile_iw7::glow`: emissive map times
    // `emissiveParams.x`, linear HDR, drawn at the draw's scale;
    // `IW7_RENDER_NOGLOW` leaves it out).
    let glows: HashMap<usize, Tex> = if std::env::var("IW7_RENDER_NOGLOW").is_ok() {
        HashMap::new()
    } else {
        map.materials
            .iter()
            .enumerate()
            .filter(|(_, m)| fastfile_iw7::glow::emissive(m).is_some())
            .filter_map(|(i, m)| {
                let g = map.glow(m, tex_size).ok()?;
                let px = g.light.iter().map(|l| [l[0], l[1], l[2], 1.0]).collect();
                Some((i, Tex { levels: vec![(g.width as usize, g.height as usize, px)] }))
            })
            .collect()
    };
    println!("glowing materials: {}", glows.len());

    // The camera.
    // (A Zombies map has no spawn list: its player starts, from the
    // entities.)
    let starts: Vec<fastfile_iw7::map::SpawnPoint> =
        fastfile_iw7::map::parse_entities(&map.entities)
            .into_iter()
            .filter(|e| e.get("classname").is_some_and(|c| c == "info_player_start"))
            .filter_map(|e| {
                let v = |k: &str| -> Option<[f32; 3]> {
                    let v: Vec<f32> = e
                        .get(k)?
                        .split_whitespace()
                        .filter_map(|x| x.parse().ok())
                        .collect();
                    (v.len() == 3).then(|| [v[0], v[1], v[2]])
                };
                Some(fastfile_iw7::map::SpawnPoint {
                    classname: "info_player_start_spawn".to_owned(),
                    origin: v("origin")?,
                    angles: v("angles").unwrap_or_default(),
                    ..Default::default()
                })
            })
            .collect();
    let spawns: Vec<_> = map
        .spawns
        .iter()
        .filter(|s| s.classname.contains("spawn"))
        .chain(starts.iter())
        .collect();
    let make_cam = |view: &str, w: usize, h: usize| -> Cam {
        if view == "top" {
            let pts: Vec<[f32; 3]> = if spawns.is_empty() {
                vec![map.mins, map.maxs]
            } else {
                spawns.iter().map(|s| s.origin).collect()
            };
            let lo = [0, 1].map(|k| pts.iter().map(|p| p[k]).fold(f32::MAX, f32::min));
            let hi = [0, 1].map(|k| pts.iter().map(|p| p[k]).fold(f32::MIN, f32::max));
            let mid = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
            let half = ((hi[1] - lo[1]) * 0.5).max((hi[0] - lo[0]) * 0.5 * h as f32 / w as f32)
                * 1.25
                + 512.0;
            let top = pts.iter().map(|p| p[2]).fold(f32::MIN, f32::max) + 1500.0;
            Cam {
                eye: [mid[0], mid[1], top],
                f: [0.0, 0.0, -1.0],
                r: [1.0, 0.0, 0.0],
                u: [0.0, 1.0, 0.0],
                ortho: Some(half),
                focal: 0.0,
                w,
                h,
            }
        } else {
            let (eye, yaw, pitch) = if let Some(rest) = view.strip_prefix("cam:") {
                let v: Vec<f32> = rest.split(',').filter_map(|x| x.parse().ok()).collect();
                ([v[0], v[1], v[2]], v[3], v[4])
            } else {
                let n: usize = view
                    .strip_prefix("spawn:")
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
                let s = spawns
                    .get(n.min(spawns.len().saturating_sub(1)))
                    .expect("no spawn");
                println!(
                    "spawn {n}: {} at {:?} angles {:?}",
                    s.classname, s.origin, s.angles
                );
                (
                    [s.origin[0], s.origin[1], s.origin[2] + 60.0],
                    s.angles[1],
                    5.0,
                )
            };
            let (y, p) = (yaw.to_radians(), pitch.to_radians());
            let f = [p.cos() * y.cos(), p.cos() * y.sin(), -p.sin()];
            let r = norm([f[1], -f[0], 0.0]);
            let u = cross(r, f);
            Cam {
                eye,
                f,
                r,
                u,
                ortho: None,
                focal: w as f32 * 0.5 / (65f32.to_radians() * 0.5).tan(),
                w,
                h,
            }
        }
    };
    let sky_cube = sky_cube(&map);
    println!(
        "sky: {}",
        sky_cube
            .as_ref()
            .map_or("none (the fog's colour)".to_owned(), |c| format!(
                "{}px cube",
                c.size
            ))
    );
    let ground = map.static_model_light(&pages);
    let half_f = |b: u16| {
        let s = if b & 0x8000 != 0 { -1.0 } else { 1.0 };
        let e = i32::from((b >> 10) & 0x1f);
        let m = f32::from(b & 0x3ff);
        s * if e == 0 {
            m * 2f32.powi(-24)
        } else {
            (1.0 + m / 1024.0) * 2f32.powi(e - 15)
        }
    };
    // A view drawn at the vision's fixed exposure.
    let render = |cam: &Cam| -> Target {
        let (w, h) = (cam.w, cam.h);
        let scene = Scene {
            sun_dir,
            sun,
            lit,
            fog: if cam.ortho.is_some() { None } else { fog },
            eye: cam.eye,
            pages: &pages,
            scale,
        };

        // The background: the sky's cube (`sky_<map>`, HDR) as the game's sky
        // pass draws it, times the draw's scale and `r_hdrSkyIntensity`,
        // fogged at its distance; else the fog's colour.
        let mut t = Target {
            frame: vec![[0.0; 3]; w * h],
            depth: vec![f32::MAX; w * h],
            ids: vec![(0, 0); w * h],
            hdr: vec![0.0; w * h],
        };
        for y in 0..h {
            for x in 0..w {
                let dir = if cam.ortho.is_some() {
                    [0.0, 0.0, 1.0]
                } else {
                    let (vx, vy) = (
                        (x as f32 + 0.5 - w as f32 * 0.5) / cam.focal,
                        (h as f32 * 0.5 - y as f32 - 0.5) / cam.focal,
                    );
                    norm(std::array::from_fn(|k| {
                        cam.f[k] + cam.r[k] * vx + cam.u[k] * vy
                    }))
                };
                let mut c = match &sky_cube {
                    Some(cube) => cube.sample(dir).map(|v| v * scale * vision.sky_intensity),
                    None => fog.map_or([0.25; 3], |f| f.colour),
                };
                if let (Some(f), Some(_)) = (scene.fog, &sky_cube) {
                    c = f.apply(c, 2.0e7);
                }
                t.frame[y * w + x] = c.map(|v| light::bo2_rolloff(v.max(0.0).sqrt()));
                t.hdr[y * w + x] = (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]) / scale;
            }
        }

        // The world, then the placed models, by pass: opaque, alpha-tested,
        // blended, added.
        let order = [Kind::Opaque, Kind::AlphaTest, Kind::Blend, Kind::Add];
        for &pass in &order {
            for s in &map.surfaces {
                let (kind, unlit) = s.material.map_or((Kind::Opaque, false), |m| {
                    kind_of(&map.materials[m].technique_set)
                });
                if kind != pass || s.hidden_brush {
                    continue;
                }
                let shade = Shade {
                    tex: s
                        .material
                        .and_then(|m| texs.get(&m))
                        .and_then(Option::as_ref),
                    kind,
                    unlit,
                    page: scene
                        .pages
                        .get(usize::from(s.lightmap_index))
                        .and_then(Option::as_ref),
                    ground: None,
                    layers: s.material.map_or([None, None], |m| layer_texs[m]),
                    id: (s.material.map_or(0, |m| m as u32 + 1), 0),
                    glow: None,
                };
                for k in 0..s.tri_count as usize {
                    let v = |j: usize| -> Option<V> {
                        let ix = *map.indices.get(s.base_index as usize + 3 * k + j)?;
                        let wv = map
                            .vertices
                            .get((s.first_vertex + u32::from(ix)) as usize)?;
                        let raw = wv.color.to_le_bytes();
                        // (A world surface's vertex alpha is no opacity unless
                        // it blends.)
                        let a = if pass == Kind::Blend {
                            f32::from(raw[3]) / 255.0
                        } else {
                            1.0
                        };
                        Some(V {
                            p: wv.xyz,
                            uv: wv.uv,
                            lm: wv.lmap_uv,
                            n: wv.normal,
                            a,
                            w: [f32::from(raw[1]) / 255.0, f32::from(raw[2]) / 255.0],
                        })
                    };
                    let (Some(a), Some(b), Some(c)) = (v(0), v(1), v(2)) else {
                        continue;
                    };
                    draw_tri(&cam, &mut t, [a, b, c], &shade, &scene);
                }
            }
            for (i, sm) in map.static_models.iter().enumerate() {
                let m = &map.models[sm.model];
                let g = ground[i].unwrap_or(LightSample {
                    base: [0.1; 3],
                    directional: [0.05; 3],
                    sun: 0.5,
                    facing: 1.0,
                });
                for surf in &m.surfaces {
                    let (kind, unlit) = surf.material.map_or((Kind::Opaque, false), |k| {
                        kind_of(&map.materials[k].technique_set)
                    });
                    if kind != pass {
                        continue;
                    }
                    let shade = Shade {
                        tex: surf
                            .material
                            .and_then(|k| texs.get(&k))
                            .and_then(Option::as_ref),
                        kind,
                        unlit,
                        page: None,
                        ground: Some(g),
                        layers: [None, None],
                        id: (surf.material.map_or(0, |k| k as u32 + 1), i as u32 + 1),
                        glow: surf.material.and_then(|k| glows.get(&k)).filter(|_| matches!(kind, Kind::Opaque)),
                    };
                    let vert = |ix: u16| -> Option<V> {
                        let v = surf.verts.get(usize::from(ix))?;
                        let f = |o: usize| f32::from_le_bytes([v[o], v[o + 1], v[o + 2], v[o + 3]]);
                        let lp = [f(0), f(4), f(8)];
                        let p = std::array::from_fn(|k| {
                            sm.origin[k]
                                + (lp[0] * sm.axis[0][k]
                                    + lp[1] * sm.axis[1][k]
                                    + lp[2] * sm.axis[2][k])
                                    * sm.scale
                        });
                        let ln = fastfile_iw7::map::unpack_unit(u32::from_le_bytes([
                            v[24], v[25], v[26], v[27],
                        ]));
                        let n = std::array::from_fn(|k| {
                            ln[0] * sm.axis[0][k] + ln[1] * sm.axis[1][k] + ln[2] * sm.axis[2][k]
                        });
                        let uv = [
                            half_f(u16::from_le_bytes([v[20], v[21]])),
                            half_f(u16::from_le_bytes([v[22], v[23]])),
                        ];
                        Some(V {
                            p,
                            uv,
                            lm: [0.0; 2],
                            n,
                            a: 1.0,
                            w: [0.0; 2],
                        })
                    };
                    for tri in &surf.tris {
                        let (Some(a), Some(b), Some(c)) =
                            (vert(tri[0]), vert(tri[1]), vert(tri[2]))
                        else {
                            continue;
                        };
                        draw_tri(&cam, &mut t, [a, b, c], &shade, &scene);
                    }
                }
            }
        }
        t
    };
    // `meter:<n>`: the log-average HDR luminance of n spawn views (spread
    // over the spawn list), small, for the automatic exposure: each and
    // their geometric mean.
    if let Some(n) = view
        .strip_prefix("meter:")
        .and_then(|n| n.parse::<usize>().ok())
    {
        let n = n.clamp(1, spawns.len().max(1));
        let mut logs = Vec::new();
        for i in 0..n {
            let k = i * spawns.len() / n;
            let cam = make_cam(&format!("spawn:{k}"), 240, 135);
            let t = render(&cam);
            let m = (t
                .hdr
                .iter()
                .map(|l| f64::from(l.max(1e-5).ln()))
                .sum::<f64>()
                / t.hdr.len().max(1) as f64) as f32;
            println!("meter view {k}: log-average {:.4}", m.exp());
            logs.push(m);
        }
        let mean = (logs.iter().sum::<f32>() / logs.len().max(1) as f32).exp();
        println!(
            "METERED {} {mean:.5} ({} views, {:.1}s)",
            map.name,
            logs.len(),
            started.elapsed().as_secs_f32()
        );
        return;
    }
    let cam = make_cam(view, w, h);
    let mut t = render(&cam);
    println!("drawn at {:.1}s", started.elapsed().as_secs_f32());

    // The exposure: the vision's fixed one, or (`tonemapAuto`) the one its
    // metering gives: the spawn points' estimate the game's lane takes
    // (`Iw7Map::metered_light`), or this view's own log-average
    // (`IW7_RENDER_METER=view`); `IW7_RENDER_METER=fixed` keeps the fixed
    // one. The frame is brought from the scale it was drawn at.
    let lin = |e: f32| {
        let u = light::bo2_unroll(e);
        u * u
    };
    let view_meter = {
        let (mut sum, mut n) = (0f64, 0usize);
        for &l in &t.hdr {
            sum += f64::from(l.max(1e-5).ln());
            n += 1;
        }
        (sum / n.max(1) as f64).exp() as f32
    };
    let spawn_points: Vec<[f32; 3]> = spawns.iter().map(|s| s.origin).collect();
    let proxy_meter = map.metered_light(&pages, &spawn_points);
    let meter = std::env::var("IW7_RENDER_METER").unwrap_or_default();
    let metered = match meter.as_str() {
        "view" => Some(view_meter),
        "fixed" => None,
        // (The game's: the maps' metered table, else the estimate.)
        _ => light::metered_view_light(&map.name, proxy_meter),
    };
    let new_scale = if std::env::var("IW7_RENDER_EV").is_ok() {
        scale
    } else {
        metered.map_or(scale, |m| {
            vision.draw_scale_for(
                m / env_f("IW7_RENDER_KEY").map_or(1.0, |key| key / light::AUTO_KEY),
            )
        })
    };
    println!(
        "metering: view log-average {view_meter:.4}, spawn estimate {proxy_meter:?} (view/estimate {:.2}); exposure fixed {:.3}, auto {:.3} ({:+.2} stops from fixed)",
        proxy_meter.map_or(0.0, |p| view_meter / p),
        scale * light::DRAW_UNIT,
        new_scale * light::DRAW_UNIT,
        (new_scale / scale).log2()
    );
    if (new_scale / scale - 1.0).abs() > 1e-4 {
        let r = new_scale / scale;
        for c in t.frame.iter_mut() {
            *c = c.map(|e| light::bo2_rolloff((lin(e) * r).sqrt()));
        }
    }

    // The final pass: the frame through the grade's table.
    let display: Vec<[f32; 3]> = t
        .frame
        .iter()
        .map(|c| grade.as_ref().map_or(*c, |g| light::lut_sample(g, *c)))
        .collect();

    // `IW7_RENDER_PICK=x,y;x,y`: what is drawn at those pixels.
    if let Ok(picks) = std::env::var("IW7_RENDER_PICK") {
        for p in picks.split(';') {
            let Some((x, y)) = p.split_once(',').and_then(|(a, b)| {
                Some((
                    a.trim().parse::<usize>().ok()?,
                    b.trim().parse::<usize>().ok()?,
                ))
            }) else {
                continue;
            };
            let o = y.min(h - 1) * w + x.min(w - 1);
            let (m, sm) = t.ids[o];
            let model = (sm > 0).then(|| {
                let s = &map.static_models[sm as usize - 1];
                format!(
                    "{} at {:?} (ground {:?})",
                    map.models[s.model].name,
                    s.origin,
                    ground[sm as usize - 1]
                )
            });
            println!(
                "pick {x},{y}: frame {:?} display {:?} model {model:?}",
                t.frame[o], display[o]
            );
            if m > 0 {
                let mm = &map.materials[m as usize - 1];
                let tx: Vec<String> = mm
                    .textures
                    .iter()
                    .map(|t| {
                        format!(
                            "{}={}{}",
                            texture::sampler_name(t.0)
                                .map_or_else(|| format!("{:08x}", t.0), str::to_owned),
                            t.3,
                            map.image_info(&t.3)
                                .map_or(" (MISSING)".to_owned(), |i| format!(
                                    " fmt {} {}x{}",
                                    i.format, i.width, i.height
                                ))
                        )
                    })
                    .collect();
                println!(
                    "    material {} techset {} sort {} [{}] consts {:?}",
                    mm.name,
                    mm.technique_set,
                    mm.sort_key,
                    tx.join(", "),
                    mm.constants
                );
            }
        }
    }
    let (mean, lum, sd) = stats(&display);
    println!("frame: mean colour {mean:.3?}, luminance {lum:.3} (spread {sd:.3})");
    if let Some((rw, rh, rpx)) = std::env::var("IW7_RENDER_REF")
        .ok()
        .and_then(|r| read_png(&r))
    {
        let (rm, rl, rs) = stats(&rpx);
        println!(
            "reference {rw}x{rh}: mean colour {rm:.3?}, luminance {rl:.3} (spread {rs:.3}); render/reference luminance {:.3}",
            lum / rl.max(1e-4)
        );
    }
    let mut rgba = vec![0u8; w * h * 4];
    for (i, c) in display.iter().enumerate() {
        for k in 0..3 {
            rgba[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        rgba[i * 4 + 3] = 255;
    }
    let f = std::fs::File::create(out_path).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&rgba).unwrap();
    println!(
        "wrote {out_path} in {:.1}s",
        started.elapsed().as_secs_f32()
    );
}
