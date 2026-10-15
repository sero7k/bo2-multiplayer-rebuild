//! A software render of a map, to check how it reads and lights:
//! `iw7_render <map.ff> <out.png> [view]`, view one of
//! - `top`: straight down over the spawn points' area (orthographic),
//! - `spawn[:n]`: from spawn point n (default 0), at eye height,
//! - `cam:x,y,z,yaw,pitch`: from a point (degrees).
//!
//! The world is drawn as the game's fallback draw lights a CoD4-form
//! lightmap (`fastfile_iw7::light`): colour map times vertex colour, times
//! (base + directional + sun * visibility * N.L)^2, in gamma. The placed
//! models take the light under them (`Iw7Map::ground_light`) plus the sun.
//! `IW7_RENDER_NOLIGHT=1` draws colour maps flat; `IW7_RENDER_TEX=<px>`
//! sets the colour maps' size (default 128); `IW7_RENDER_SIZE=WxH`.

use fastfile_iw7::light::{LightSample, LightmapPage};
use fastfile_iw7::map::{Iw7Map, MapSources};
use fastfile_iw7::texture;
use std::collections::HashMap;
use std::path::Path;

struct Tex {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

impl Tex {
    fn sample(&self, uv: [f32; 2]) -> [f32; 4] {
        let x = ((uv[0].rem_euclid(1.0)) * self.w as f32) as usize % self.w;
        let y = ((uv[1].rem_euclid(1.0)) * self.h as f32) as usize % self.h;
        self.px[y * self.w + x]
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
        if b.len() >= 3 && matches!(b[0], b'b' | b'm' | b'a' | b't') && (b'1'..=b'2').contains(&b[1]) && b[2] == b'c' {
            out[usize::from(b[1] - b'1')] = if b[0] == b'm' { 2 } else { 1 };
        }
    }
    out
}

fn kind_of(techset: &str) -> Kind {
    use fastfile_iw7::look::{Blend, look};
    let l = look(techset);
    if l.hidden.is_some() {
        return Kind::Hidden;
    }
    match l.blend {
        Blend::Add => Kind::Add,
        Blend::Blend | Blend::Multiply => Kind::Blend,
        Blend::Opaque if l.alpha_test => Kind::AlphaTest,
        Blend::Opaque => Kind::Opaque,
    }
}

#[derive(Clone, Copy)]
struct V {
    p: [f32; 3],
    uv: [f32; 2],
    lm: [f32; 2],
    n: [f32; 3],
    c: [f32; 4],
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
        c: std::array::from_fn(|k| l(a.c[k], b.c[k])),
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
            None => [w * 0.5 + v[0] / v[2] * self.focal, h * 0.5 - v[1] / v[2] * self.focal, v[2]],
        }
    }
}

struct Target {
    colour: Vec<[f32; 3]>,
    depth: Vec<f32>,
    /// Per pixel: (material + 1, placed model + 1) of what was drawn there.
    ids: Vec<(u32, u32)>,
}

/// What a pixel's colour comes from.
struct Shade<'a> {
    tex: Option<&'a Tex>,
    kind: Kind,
    page: Option<&'a LightmapPage>,
    /// For models: the light under them (None: lit by the page).
    ground: Option<LightSample>,
    sun_dir: [f32; 3],
    sun: [f32; 3],
    lit: bool,
    /// Layer 1 and 2 colour maps (opacity in alpha) and whether each
    /// multiplies (else blends) by its weight.
    layers: [Option<(&'a Tex, bool)>; 2],
    /// (material + 1, placed model + 1), for `IW7_RENDER_PICK`.
    id: (u32, u32),
    /// A model material's own light (`fastfile_iw7::glow`), linear and
    /// already in the page's exposure.
    glow: Option<&'a Tex>,
}

fn draw_tri(cam: &Cam, t: &mut Target, tri: [V; 3], sh: &Shade) {
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
    for k in 1..poly.len() - 1 {
        raster(cam, t, [poly[0], poly[k], poly[k + 1]], sh);
    }
}

fn raster(cam: &Cam, t: &mut Target, tri: [(V, [f32; 3]); 3], sh: &Shade) {
    let s = tri.map(|(_, v)| cam.project(v));
    let area = (s[1][0] - s[0][0]) * (s[2][1] - s[0][1]) - (s[2][0] - s[0][0]) * (s[1][1] - s[0][1]);
    if area.abs() < 1e-8 {
        return;
    }
    let minx = s.iter().map(|p| p[0]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let maxx = s.iter().map(|p| p[0]).fold(f32::MIN, f32::max).ceil().min(cam.w as f32 - 1.0);
    let miny = s.iter().map(|p| p[1]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let maxy = s.iter().map(|p| p[1]).fold(f32::MIN, f32::max).ceil().min(cam.h as f32 - 1.0);
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
            let at = |f: &dyn Fn(&V) -> f32| b[0] * f(&tri[0].0) + b[1] * f(&tri[1].0) + b[2] * f(&tri[2].0);
            let uv = [at(&|v| v.uv[0]), at(&|v| v.uv[1])];
            let mut texel = sh.tex.map_or([0.6, 0.6, 0.6, 1.0], |tx| tx.sample(uv));
            for (k, l) in sh.layers.iter().enumerate() {
                let Some((lt, multiply)) = l else { continue };
                let wgt = b[0] * tri[0].0.w[k] + b[1] * tri[1].0.w[k] + b[2] * tri[2].0.w[k];
                let lx = lt.sample(uv);
                for c in 0..3 {
                    texel[c] = if *multiply { texel[c] * (1.0 + wgt * (lx[c] - 1.0)) } else { texel[c] + (lx[c] - texel[c]) * lx[3] * wgt };
                }
            }
            if matches!(sh.kind, Kind::AlphaTest) && texel[3] < 0.5 {
                continue;
            }
            let c = [at(&|v| v.c[0]), at(&|v| v.c[1]), at(&|v| v.c[2]), at(&|v| v.c[3])];
            let n = norm([at(&|v| v.n[0]), at(&|v| v.n[1]), at(&|v| v.n[2])]);
            let base: [f32; 3] = if std::env::var("IW7_RENDER_LMONLY").is_ok() {
                [0.5; 3]
            } else {
                std::array::from_fn(|k| (texel[k] * c[k]).powi(2))
            };
            let ndl = dot(n, sh.sun_dir).max(0.0);
            let light: [f32; 3] = if !sh.lit {
                [1.0; 3]
            } else if let Some(g) = sh.ground {
                let a = g.ambient_linear();
                std::array::from_fn(|k| a[k] + sh.sun[k] * g.sun * ndl)
            } else if let Some(p) = sh.page {
                let l = p.sample([at(&|v| v.lm[0]), at(&|v| v.lm[1])]);
                std::array::from_fn(|k| {
                    let g = l.base[k] + l.directional[k] * 0.7 + sh.sun[k] * l.sun * ndl;
                    g * g
                })
            } else {
                // Unlit (no lightmap): emissive-ish, as the fallback draws it.
                [1.0; 3]
            };
            let g = sh.glow.map_or([0.0; 4], |t| t.sample(uv));
            let out: [f32; 3] = std::array::from_fn(|k| (base[k] * light[k] + g[k]).max(0.0).sqrt());
            match sh.kind {
                Kind::Blend => {
                    let a = texel[3] * c[3];
                    for k in 0..3 {
                        t.colour[o][k] = t.colour[o][k] * (1.0 - a) + out[k] * a;
                    }
                }
                Kind::Add => {
                    for k in 0..3 {
                        t.colour[o][k] += texel[k] * texel[3];
                    }
                }
                _ => {
                    t.colour[o] = out;
                    t.depth[o] = z;
                    t.ids[o] = sh.id;
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let started = std::time::Instant::now();
    let map = Iw7Map::load(&{ let src = MapSources::beside(Path::new(&args[1])); if std::env::var("IW7_NO_COMMON").is_ok() { src } else { src.with_common() } }).expect("map");
    for l in &map.report {
        println!("{l}");
    }
    let out_path = &args[2];
    let view = args.get(3).map(String::as_str).unwrap_or("top");
    let tex_size: u32 = std::env::var("IW7_RENDER_TEX").ok().and_then(|v| v.parse().ok()).unwrap_or(128);
    let lit = std::env::var("IW7_RENDER_NOLIGHT").is_err();
    let (w, h) = std::env::var("IW7_RENDER_SIZE")
        .ok()
        .and_then(|v| v.split_once('x').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?))))
        .unwrap_or((1280usize, 720usize));

    let pages: Vec<Option<LightmapPage>> = (0..map.light_maps.len())
        .map(|i| match map.lightmap_page(i, 2048) {
            Ok(p) => Some(p),
            Err(e) => {
                println!("lightmap {i}: {e}");
                None
            }
        })
        .collect();
    println!(
        "lightmaps read at {:.1}s: {:?}",
        started.elapsed().as_secs_f32(),
        pages.iter().flatten().map(|p| (p.mean_light, p.exposure)).collect::<Vec<_>>()
    );

    // The sun: the ComWorld's (HDR, brightest channel the light) as the lane
    // turns it into CoD4's gamma sun.
    let exposure = pages.iter().flatten().next().map_or(0.25, |p| p.exposure);
    let (sun_dir, sun) = match map.lights.iter().find(|l| l.kind == 1) {
        Some(l) => {
            let peak = l.color.iter().copied().fold(1e-6f32, f32::max);
            let g = fastfile_iw7::light::sun_gamma(peak, exposure);
            (norm(l.dir), l.color.map(|c| c / peak * g))
        }
        None => (norm([0.3, 0.3, 0.9]), [1.0; 3]),
    };
    println!("sun toward {sun_dir:?} colour {sun:?}");

    // Colour maps by material.
    let mut texs: HashMap<usize, Option<Tex>> = HashMap::new();
    let mut tex_of = |m: usize| -> Option<usize> {
        texs.entry(m).or_insert_with(|| {
            let mm = &map.materials[m];
            let name = mm.texture("colorMap")?;
            let keep = fastfile_iw7::look::colour_alpha_is_opacity(name);
            let rgba = map.colour_rgba(mm, tex_size, keep).ok()?;
            let to = |b: u8| f32::from(b) / 255.0;
            Some(Tex {
                w: rgba.width as usize,
                h: rgba.height as usize,
                px: rgba.levels[0].chunks_exact(4).map(|p| [to(p[0]), to(p[1]), to(p[2]), to(p[3])]).collect(),
            })
        });
        Some(m)
    };
    for m in 0..map.materials.len() {
        tex_of(m);
    }
    // Layered world materials' layer colour maps (`IW7_RENDER_NOLAYERS`
    // draws the base alone, as before).
    let layered_tex: Vec<[Option<Tex>; 2]> = map
        .materials
        .iter()
        .map(|mm| {
            let layers = layer_kinds(&mm.technique_set);
            std::array::from_fn(|k| {
                if layers[k] == 0 || std::env::var("IW7_RENDER_NOLAYERS").is_ok() {
                    return None;
                }
                let keep = mm.texture(&format!("colorMap{}", k + 1)).is_some_and(fastfile_iw7::look::colour_alpha_is_opacity);
                let rgba = map.layer_rgba(mm, k + 1, tex_size, keep).ok()?;
                let to = |b: u8| f32::from(b) / 255.0;
                Some(Tex { w: rgba.width as usize, h: rgba.height as usize, px: rgba.levels[0].chunks_exact(4).map(|p| [to(p[0]), to(p[1]), to(p[2]), to(p[3])]).collect() })
            })
        })
        .collect();
    let layer_texs: Vec<[Option<(&Tex, bool)>; 2]> = map
        .materials
        .iter()
        .zip(&layered_tex)
        .map(|(mm, t)| {
            let kinds = layer_kinds(&mm.technique_set);
            std::array::from_fn(|k| t[k].as_ref().map(|t| (t, kinds[k] == 2)))
        })
        .collect();
    println!("layered materials: {}", layer_texs.iter().filter(|l| l[0].is_some() || l[1].is_some()).count());
    // Model materials' glow, in the page's exposure, hue kept under 2.5
    // as the game's lane does (`IW7_RENDER_NOGLOW` leaves it out).
    let glow_exposure = pages.iter().flatten().next().map_or(1.0, |p| p.exposure);
    let glows: HashMap<usize, Tex> = if std::env::var("IW7_RENDER_NOGLOW").is_ok() {
        HashMap::new()
    } else {
        map.materials
            .iter()
            .enumerate()
            .filter(|(_, m)| fastfile_iw7::glow::emissive(m).is_some())
            .filter_map(|(i, m)| {
                let g = map.glow(m, tex_size).ok()?;
                let px = g
                    .light
                    .iter()
                    .map(|l| {
                        let mut v = l.map(|c| c * glow_exposure);
                        let peak = v.iter().copied().fold(0.0, f32::max);
                        if peak > 2.5 {
                            v = v.map(|c| c * 2.5 / peak);
                        }
                        [v[0], v[1], v[2], 1.0]
                    })
                    .collect();
                Some((i, Tex { w: g.width as usize, h: g.height as usize, px }))
            })
            .collect()
    };
    println!("glowing materials: {}", glows.len());
    if std::env::var("IW7_RENDER_NMDUMP").is_ok() {
        let mut seen = std::collections::BTreeMap::new();
        for mm in &map.materials {
            for t in &mm.textures {
                let slot = texture::sampler_name(t.0).map_or_else(|| format!("{:08x}", t.0), str::to_owned);
                let kind = t.3.rsplit('_').next().unwrap_or("").to_owned();
                if let Some(i) = map.image_info(&t.3) {
                    seen.entry((slot, kind, i.format, t.2)).or_insert_with(|| t.3.clone());
                }
            }
        }
        for ((slot, kind, fmt, sem), example) in seen {
            println!("  {slot:14} *_{kind:10} fmt {fmt:3} sem {sem:2} e.g. {example}");
        }
    }
    if std::env::var("IW7_RENDER_TEXDUMP").is_ok() {
        for (m, t) in texs.iter().take(30) {
            let mm = &map.materials[*m];
            let name = mm.texture("colorMap").unwrap_or("-");
            let info = map.image_info(name).map(|i| format!("fmt {} {}x{} streams {:?}", i.format, i.width, i.height, i.streams));
            match t {
                Some(t) => {
                    let n = t.px.len() as f32;
                    let mean: [f32; 4] = std::array::from_fn(|k| t.px.iter().map(|p| p[k]).sum::<f32>() / n);
                    println!("{} {} {}x{} mean {mean:?} {info:?}", mm.name, name, t.w, t.h);
                }
                None => println!("{} {name}: none {info:?} {:?}", mm.name, map.image(name, tex_size).err()),
            }
        }
    }

    // The camera.
    // (A Zombies map has no spawn list: its player starts, from the
    // entities.)
    let starts: Vec<fastfile_iw7::map::SpawnPoint> = fastfile_iw7::map::parse_entities(&map.entities)
        .into_iter()
        .filter(|e| e.get("classname").is_some_and(|c| c == "info_player_start"))
        .filter_map(|e| {
            let v = |k: &str| -> Option<[f32; 3]> {
                let v: Vec<f32> = e.get(k)?.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                (v.len() == 3).then(|| [v[0], v[1], v[2]])
            };
            Some(fastfile_iw7::map::SpawnPoint { classname: "info_player_start_spawn".to_owned(), origin: v("origin")?, angles: v("angles").unwrap_or_default(), ..Default::default() })
        })
        .collect();
    let spawns: Vec<_> = map.spawns.iter().filter(|s| s.classname.contains("spawn")).chain(starts.iter()).collect();
    let cam = if view == "top" {
        let pts: Vec<[f32; 3]> = if spawns.is_empty() { vec![map.mins, map.maxs] } else { spawns.iter().map(|s| s.origin).collect() };
        let lo = [0, 1].map(|k| pts.iter().map(|p| p[k]).fold(f32::MAX, f32::min));
        let hi = [0, 1].map(|k| pts.iter().map(|p| p[k]).fold(f32::MIN, f32::max));
        let mid = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
        let half = ((hi[1] - lo[1]) * 0.5).max((hi[0] - lo[0]) * 0.5 * h as f32 / w as f32) * 1.25 + 512.0;
        let top = pts.iter().map(|p| p[2]).fold(f32::MIN, f32::max) + 1500.0;
        Cam { eye: [mid[0], mid[1], top], f: [0.0, 0.0, -1.0], r: [1.0, 0.0, 0.0], u: [0.0, 1.0, 0.0], ortho: Some(half), focal: 0.0, w, h }
    } else {
        let (eye, yaw, pitch) = if let Some(rest) = view.strip_prefix("cam:") {
            let v: Vec<f32> = rest.split(',').filter_map(|x| x.parse().ok()).collect();
            ([v[0], v[1], v[2]], v[3], v[4])
        } else {
            let n: usize = view.strip_prefix("spawn:").and_then(|n| n.parse().ok()).unwrap_or(0);
            let s = spawns.get(n.min(spawns.len().saturating_sub(1))).expect("no spawn");
            println!("spawn {n}: {} at {:?} angles {:?}", s.classname, s.origin, s.angles);
            ([s.origin[0], s.origin[1], s.origin[2] + 60.0], s.angles[1], 5.0)
        };
        let (y, p) = (yaw.to_radians(), pitch.to_radians());
        let f = [p.cos() * y.cos(), p.cos() * y.sin(), -p.sin()];
        let r = norm([f[1], -f[0], 0.0]);
        let u = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
        Cam { eye, f, r, u, ortho: None, focal: w as f32 * 0.5 / (65f32.to_radians() * 0.5).tan(), w, h }
    };

    let sky = [0.55, 0.65, 0.8];
    let mut t = Target { colour: vec![sky; w * h], depth: vec![f32::MAX; w * h], ids: vec![(0, 0); w * h] };

    // The world: opaque first, then alpha-tested, then blended.
    let order: Vec<Kind> = if std::env::var("IW7_RENDER_OPAQUE").is_ok() {
        vec![Kind::Opaque, Kind::AlphaTest]
    } else {
        vec![Kind::Opaque, Kind::AlphaTest, Kind::Blend, Kind::Add]
    };
    // IW7's world vertex colour is no tint (layer weights and the like:
    // mp_riot's streets turn magenta and cyan by it); only its alpha is
    // kept unless IW7_RENDER_VCOL is set.
    let use_vcol = std::env::var("IW7_RENDER_VCOL").is_ok();
    let vcol = |c: u32| {
        let b = c.to_le_bytes();
        let v = [b[2], b[1], b[0], b[3]].map(|x| f32::from(x) / 255.0);
        if use_vcol { v } else { [1.0, 1.0, 1.0, v[3]] }
    };
    let mut drawn = [0usize; 5];
    for &pass in &order {
        for s in &map.surfaces {
            let kind = s.material.map_or(Kind::Opaque, |m| kind_of(&map.materials[m].technique_set));
            if kind != pass || s.hidden_brush {
                continue;
            }
            let shade = Shade {
                tex: s.material.and_then(|m| texs.get(&m)).and_then(Option::as_ref),
                kind,
                page: pages.get(usize::from(s.lightmap_index)).and_then(Option::as_ref),
                ground: None,
                sun_dir,
                sun,
                lit,
                layers: s.material.map_or([None, None], |m| layer_texs[m]),
                id: (s.material.map_or(0, |m| m as u32 + 1), 0),
                glow: None,
            };
            for k in 0..s.tri_count as usize {
                let v = |j: usize| -> Option<V> {
                    let ix = *map.indices.get(s.base_index as usize + 3 * k + j)?;
                    let wv = map.vertices.get((s.first_vertex + u32::from(ix)) as usize)?;
                    let raw = wv.color.to_le_bytes();
                    Some(V { p: wv.xyz, uv: wv.uv, lm: wv.lmap_uv, n: wv.normal, c: vcol(wv.color), w: [f32::from(raw[1]) / 255.0, f32::from(raw[2]) / 255.0] })
                };
                let (Some(a), Some(b), Some(c)) = (v(0), v(1), v(2)) else { continue };
                draw_tri(&cam, &mut t, [a, b, c], &shade);
                drawn[pass as usize] += 1;
            }
        }
    }
    if std::env::var("IW7_RENDER_CENSUS").is_ok() {
        let mut census: std::collections::BTreeMap<(String, String), usize> = Default::default();
        for s in &map.surfaces {
            if let Some(m) = s.material {
                let mm = &map.materials[m];
                *census.entry((format!("{:?}", kind_of(&mm.technique_set)), mm.technique_set.clone())).or_default() += s.tri_count as usize;
            }
        }
        for ((k, ts), n) in census {
            println!("  {k:9} {n:7} {ts}");
            if let Some(m) = map.materials.iter().find(|m| m.technique_set == ts) {
                let t: Vec<String> = m
                    .textures
                    .iter()
                    .map(|t| format!("{}={}", texture::sampler_name(t.0).map_or_else(|| format!("{:08x}", t.0), str::to_owned), t.3))
                    .collect();
                println!("      e.g. {} sort {} [{}] consts {:?}", m.name, m.sort_key, t.join(", "), m.constants.iter().map(|c| &c.0).collect::<Vec<_>>());
            }
        }
    }
    println!("world drawn at {:.1}s: {drawn:?}", started.elapsed().as_secs_f32());

    // The placed models.
    let ground = map.static_model_light(&pages);
    for sp in spawns.iter().take(3) {
        let g = map.ground_light(&pages, &[[sp.origin[0], sp.origin[1], sp.origin[2] + 8.0]], 256.0);
        println!("spawn {:?}: floor light {:?}", sp.origin, g[0]);
    }
    println!(
        "ground light: {} of {} placed models found a lit floor ({:.1}s)",
        ground.iter().filter(|g| g.is_some()).count(),
        ground.len(),
        started.elapsed().as_secs_f32()
    );
    let half_f = |b: u16| {
        let s = if b & 0x8000 != 0 { -1.0 } else { 1.0 };
        let e = i32::from((b >> 10) & 0x1f);
        let m = f32::from(b & 0x3ff);
        s * if e == 0 { m * 2f32.powi(-24) } else { (1.0 + m / 1024.0) * 2f32.powi(e - 15) }
    };
    let prop_vcol = std::env::var("IW7_RENDER_PROPVCOL").is_ok();
    {
        // How dark the placed models' vertex colours are.
        let (mut sum, mut n, mut black) = ([0f64; 4], 0usize, 0usize);
        for m in &map.models {
            for s in &m.surfaces {
                for v in &s.verts {
                    for k in 0..4 {
                        sum[k] += f64::from(v[16 + k]);
                    }
                    n += 1;
                    black += usize::from(v[16] < 16 && v[17] < 16 && v[18] < 16);
                }
            }
        }
        println!("prop vertex colour mean (bytes 16..20) {:?}, {black} of {n} near black", sum.map(|x| (x / n.max(1) as f64) as u32));
    }
    for &pass in &order {
        for (i, sm) in map.static_models.iter().enumerate() {
            let m = &map.models[sm.model];
            let g = ground[i].unwrap_or(LightSample { base: [0.35; 3], directional: [0.1; 3], sun: 0.5 });
            for surf in &m.surfaces {
                let kind = surf.material.map_or(Kind::Opaque, |k| kind_of(&map.materials[k].technique_set));
                if kind != pass {
                    continue;
                }
                let shade = Shade {
                    tex: surf.material.and_then(|k| texs.get(&k)).and_then(Option::as_ref),
                    kind,
                    page: None,
                    ground: Some(g),
                    sun_dir,
                    sun,
                    lit,
                    layers: [None, None],
                    id: (surf.material.map_or(0, |k| k as u32 + 1), i as u32 + 1),
                    glow: surf.material.and_then(|k| glows.get(&k)).filter(|_| matches!(kind, Kind::Opaque)),
                };
                let vert = |ix: u16| -> Option<V> {
                    let v = surf.verts.get(usize::from(ix))?;
                    let f = |o: usize| f32::from_le_bytes([v[o], v[o + 1], v[o + 2], v[o + 3]]);
                    let lp = [f(0), f(4), f(8)];
                    let p = std::array::from_fn(|k| sm.origin[k] + (lp[0] * sm.axis[0][k] + lp[1] * sm.axis[1][k] + lp[2] * sm.axis[2][k]) * sm.scale);
                    let ln = fastfile_iw7::map::unpack_unit(u32::from_le_bytes([v[24], v[25], v[26], v[27]]));
                    let n = std::array::from_fn(|k| ln[0] * sm.axis[0][k] + ln[1] * sm.axis[1][k] + ln[2] * sm.axis[2][k]);
                    let uv = [half_f(u16::from_le_bytes([v[20], v[21]])), half_f(u16::from_le_bytes([v[22], v[23]]))];
                    // (A placed model's vertex colour is no tint: white, as
                    // the game's lane draws it; `IW7_RENDER_PROPVCOL` shows it.)
                    let c = if prop_vcol { [v[18], v[17], v[16], v[19]].map(|x| f32::from(x) / 255.0) } else { [1.0; 4] };
                    Some(V { p, uv, lm: [0.0; 2], n, c, w: [0.0; 2] })
                };
                for tri in &surf.tris {
                    let (Some(a), Some(b), Some(c)) = (vert(tri[0]), vert(tri[1]), vert(tri[2])) else { continue };
                    draw_tri(&cam, &mut t, [a, b, c], &shade);
                }
            }
        }
    }
    println!("models drawn at {:.1}s", started.elapsed().as_secs_f32());

    // `IW7_RENDER_PICK=x,y;x,y`: what is drawn at those pixels.
    if let Ok(picks) = std::env::var("IW7_RENDER_PICK") {
        for p in picks.split(';') {
            let Some((x, y)) = p.split_once(',').and_then(|(a, b)| Some((a.trim().parse::<usize>().ok()?, b.trim().parse::<usize>().ok()?))) else { continue };
            let Some(&(m, sm)) = t.ids.get(y.min(h - 1) * w + x.min(w - 1)) else { continue };
            let model = (sm > 0).then(|| {
                let s = &map.static_models[sm as usize - 1];
                format!("{} at {:?} (ground {:?})", map.models[s.model].name, s.origin, ground[sm as usize - 1])
            });
            println!("pick {x},{y}: colour {:?} model {model:?}", t.colour[y.min(h - 1) * w + x.min(w - 1)]);
            if m > 0 {
                let mm = &map.materials[m as usize - 1];
                let tx: Vec<String> = mm
                    .textures
                    .iter()
                    .map(|t| format!("{}={}{}", texture::sampler_name(t.0).map_or_else(|| format!("{:08x}", t.0), str::to_owned), t.3, map.image_info(&t.3).map_or(" (MISSING)".to_owned(), |i| format!(" fmt {}", i.format))))
                    .collect();
                println!("    material {} techset {} sort {} [{}] consts {:?}", mm.name, mm.technique_set, mm.sort_key, tx.join(", "), mm.constants);
            }
        }
    }
    let mut rgba = vec![0u8; w * h * 4];
    for (i, c) in t.colour.iter().enumerate() {
        rgba[i * 4..i * 4 + 4].copy_from_slice(&[c[0], c[1], c[2]].map(|x| (x.clamp(0.0, 1.0) * 255.0) as u8).into_iter().chain([255]).collect::<Vec<_>>());
    }
    let f = std::fs::File::create(out_path).expect("png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&rgba).unwrap();
    println!("wrote {out_path} in {:.1}s", started.elapsed().as_secs_f32());
}
