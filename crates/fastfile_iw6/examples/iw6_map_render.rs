//! A software render of a Ghosts map as `fastfile_iw6` reads it: world
//! surfaces and placed models, colour maps (at most `IW6_RENDER_TEX` px,
//! default 256), the lightmap as the game's IW3 fallback draw applies it,
//! written to PNG for checking by eye.
//!
//! `iw6_map_render <mp_map.ff> <out_prefix> [view...]`, a view being
//! `top` (orthographic, the spawns' area from above), `spawnN` (eye height
//! at the Nth spawn point, its facing) or `x,y,z,yaw,pitch`.
//! `IW6_RENDER_MODE`: `lit` (default: colour x lightmap as the game's
//! shader), `albedo` (colour maps only), `light` (lightmap only), `id`
//! (a colour per surface: z-fighting and duplicates show as noise),
//! `untex` (textured grey, untextured magenta). `IW6_RENDER_NO_MODELS=1`
//! leaves the placed models out. `IW6_RENDER_DUMP_LM=1` writes the
//! lightmap pages.
use std::collections::HashMap;

use fastfile_iw6::image::{ImagePaks, ImagePixels, image_pixels};

const COLOR_MAP_HASH: u32 = 0xa0ab_1041;
const W: usize = 960;
const H: usize = 540;

#[derive(Clone)]
struct Tex {
    w: usize,
    h: usize,
    rgba: Vec<[u8; 4]>,
}

fn decode(px: &ImagePixels) -> Option<Tex> {
    let (w, h) = (px.width as usize, px.height as usize);
    let f = px.dxgi_format;
    let mut out = vec![[0u8; 4]; w * h];
    let mut block = |unit: usize, dec: &dyn Fn(&[u8], &mut [u8], usize)| -> Option<()> {
        let (bw, bh) = (w.div_ceil(4), h.div_ceil(4));
        if px.data.len() < bw * bh * unit {
            return None;
        }
        let mut tmp = [0u8; 64];
        for by in 0..bh {
            for bx in 0..bw {
                let b = (by * bw + bx) * unit;
                dec(&px.data[b..b + unit], &mut tmp, 16);
                for y in 0..4 {
                    for x in 0..4 {
                        let (px_x, px_y) = (bx * 4 + x, by * 4 + y);
                        if px_x < w && px_y < h {
                            let t = &tmp[y * 16 + x * 4..y * 16 + x * 4 + 4];
                            out[px_y * w + px_x] = [t[0], t[1], t[2], t[3]];
                        }
                    }
                }
            }
        }
        Some(())
    };
    match f {
        70..=72 => block(8, &|s, d, p| bcdec_rs::bc1(s, d, p))?,
        73..=75 => block(16, &|s, d, p| bcdec_rs::bc2(s, d, p))?,
        76..=78 => block(16, &|s, d, p| bcdec_rs::bc3(s, d, p))?,
        97..=99 => block(16, &|s, d, p| bcdec_rs::bc7(s, d, p))?,
        79..=81 => block(8, &|s, d, _| {
            let mut r = [0u8; 16];
            bcdec_rs::bc4(s, &mut r, 4, f == 81);
            for i in 0..16 {
                d[(i / 4) * 16 + (i % 4) * 4..][..4].copy_from_slice(&[r[i], r[i], r[i], 255]);
            }
        })?,
        82..=84 => block(16, &|s, d, _| {
            let mut r = [0u8; 32];
            bcdec_rs::bc5(s, &mut r, 8, f == 84);
            for i in 0..16 {
                d[(i / 4) * 16 + (i % 4) * 4..][..4].copy_from_slice(&[
                    r[2 * i],
                    r[2 * i + 1],
                    128,
                    255,
                ]);
            }
        })?,
        27..=29 | 87..=91 => {
            if px.data.len() < w * h * 4 {
                return None;
            }
            for i in 0..w * h {
                let p = &px.data[i * 4..i * 4 + 4];
                out[i] = if f >= 87 {
                    [p[2], p[1], p[0], if f == 88 { 255 } else { p[3] }]
                } else {
                    [p[0], p[1], p[2], p[3]]
                };
            }
        }
        _ => return None,
    }
    Some(Tex { w, h, rgba: out })
}

impl Tex {
    fn sample(&self, u: f32, v: f32) -> [f32; 4] {
        let x = ((u - u.floor()) * self.w as f32) as usize % self.w;
        let y = ((v - v.floor()) * self.h as f32) as usize % self.h;
        let p = self.rgba[y * self.w + x];
        [
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
            p[3] as f32 / 255.0,
        ]
    }
}

#[derive(Clone, Copy, Default)]
struct Vtx {
    p: [f32; 3],
    uv: [f32; 2],
    lm: [f32; 2],
    col: [f32; 4],
    n: [f32; 3],
}

struct Mat {
    tex: Option<usize>,
    /// Layers 1, 2: kind (b'b' blend, b'm' multiply, b't' test, b'a' add)
    /// and colour map.
    layers: [Option<(u8, Option<usize>)>; 2],
    /// Vertex colour tints the colour map.
    tint: bool,
    /// 0 opaque, 1 blend, 2 add, 3 multiply.
    blend_mode: u8,
    alpha_test: bool,
    id_colour: [f32; 3],
}

struct Cam {
    ortho: Option<([f32; 2], [f32; 2])>,
    pos: [f32; 3],
    fwd: [f32; 3],
    right: [f32; 3],
    up: [f32; 3],
    f: f32,
}

impl Cam {
    fn persp(pos: [f32; 3], yaw: f32, pitch: f32) -> Self {
        let (y, p) = (yaw.to_radians(), pitch.to_radians());
        let fwd = [p.cos() * y.cos(), p.cos() * y.sin(), -p.sin()];
        let right = norm(cross(fwd, [0.0, 0.0, 1.0]));
        let up = cross(right, fwd);
        Cam {
            ortho: None,
            pos,
            fwd,
            right,
            up,
            f: (W as f32 / 2.0) / (40f32).to_radians().tan(),
        }
    }
    /// Screen x, y and depth (larger = farther), or None behind the eye.
    fn project(&self, p: [f32; 3]) -> [f32; 3] {
        if let Some((mn, mx)) = self.ortho {
            let sx = (p[0] - mn[0]) / (mx[0] - mn[0]) * W as f32;
            let sy = (mx[1] - p[1]) / (mx[1] - mn[1]) * H as f32;
            return [sx, sy, 1.0e6 - p[2]];
        }
        let d = sub(p, self.pos);
        let z = dot(d, self.fwd);
        [
            W as f32 / 2.0 + dot(d, self.right) * self.f / z,
            H as f32 / 2.0 - dot(d, self.up) * self.f / z,
            z,
        ]
    }
    fn depth(&self, p: [f32; 3]) -> f32 {
        if self.ortho.is_some() {
            1.0e6 - p[2]
        } else {
            dot(sub(p, self.pos), self.fwd)
        }
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt().max(1e-9);
    [a[0] / l, a[1] / l, a[2] / l]
}
fn lerpv(a: &Vtx, b: &Vtx, t: f32) -> Vtx {
    let l = |x: f32, y: f32| x + (y - x) * t;
    Vtx {
        p: std::array::from_fn(|k| l(a.p[k], b.p[k])),
        uv: std::array::from_fn(|k| l(a.uv[k], b.uv[k])),
        lm: std::array::from_fn(|k| l(a.lm[k], b.lm[k])),
        col: std::array::from_fn(|k| l(a.col[k], b.col[k])),
        n: std::array::from_fn(|k| l(a.n[k], b.n[k])),
    }
}

struct Light {
    /// Page size and the three pages as CoD4's: A (rgb, a), B (rgb, a),
    /// primary (sun visibility).
    w: usize,
    h: usize,
    a: Vec<[u8; 4]>,
    b: Vec<[u8; 4]>,
    vis: Vec<u8>,
}

impl Light {
    fn at(&self, u: f32, v: f32) -> ([f32; 3], [f32; 3], f32, f32, f32) {
        let x = ((u.clamp(0.0, 0.9999)) * self.w as f32) as usize;
        let y = ((v.clamp(0.0, 0.9999)) * self.h as f32) as usize;
        let i = y * self.w + x;
        let (a, b) = (self.a[i], self.b[i]);
        let c = |p: [u8; 4]| {
            [
                p[2] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[0] as f32 / 255.0,
            ]
        };
        (
            c(a),
            c(b),
            a[3] as f32 / 255.0,
            b[3] as f32 / 255.0,
            self.vis[i] as f32 / 255.0,
        )
    }
}

struct Ctx {
    colour: Vec<[f32; 3]>,
    depth: Vec<f32>,
    /// What each pixel shows: material index, world (true) or model.
    what: Vec<(u32, bool)>,
    cur: (u32, bool),
    mode: u8,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&args[1]);
    let prefix = &args[2];
    let views: Vec<String> = if args.len() > 3 {
        args[3..].to_vec()
    } else {
        vec!["top".into(), "spawn0".into(), "spawn40".into()]
    };
    let mode = std::env::var("IW6_RENDER_MODE").unwrap_or_else(|_| "lit".into());
    let tex_max: u32 = std::env::var("IW6_RENDER_TEX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256);
    let models = std::env::var_os("IW6_RENDER_NO_MODELS").is_none();
    // `IW6_RENDER_BEFORE=1`: as the game drew Ghosts maps before these
    // fixes (light 1 : 1 and the sun at sqrt(peak) / 2, vertex colours read
    // as D3DCOLOR, `ha` layers unread, wind weights tinting models).
    let before = std::env::var_os("IW6_RENDER_BEFORE").is_some();
    let env_f = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
    let sun_max = env_f("IW6_SUN_MAX").unwrap_or(fastfile_iw6::map::SUN_GAMMA_MAX);
    // Vertex colour bytes: Ghosts' are R, G, B, A in memory (layer weights in
    // G and B); `IW6_RENDER_BGRA=1` reads them as D3DCOLOR (B, G, R, A).
    let bgra = before || std::env::var_os("IW6_RENDER_BGRA").is_some();

    let zone = fastfile_iw6::Zone::open(path).expect("open");
    let (cap, _) = fastfile_iw6::walk::walk_zone(&zone);
    let map = fastfile_iw6::map::read_map(&cap, &zone.image, &zone.assets.script_strings);
    let world = map.world.as_ref().expect("world");
    // The map's exposure from its baked light (as the game's), or as given.
    let key = fastfile_iw6::map::lightmap_key(&cap, &zone.image, &world.lightmaps);
    let exposure: f32 = env_f("IW6_EXPOSURE").unwrap_or(if before {
        1.0
    } else {
        match (key, env_f("IW6_EXPOSURE_KEY")) {
            (Some(k), Some(c)) => (c / k).clamp(0.05, 32.0),
            _ => fastfile_iw6::map::light_exposure(key),
        }
    });
    eprintln!("exposure {exposure} (lightmap key {key:?})");
    let mut paks = ImagePaks::new(path.parent().unwrap());
    // Images named from other zones: common_mp's (not before the fixes).
    let mut shared = fastfile_iw6::shared::SharedImages::default();
    if !before {
        for z in [
            "patch_common_mp",
            "common_mp",
            "patch_common_core_mp",
            "common_core_mp",
        ] {
            let _ = shared.add_zone(&path.parent().unwrap().join(format!("{z}.ff")));
        }
    }

    // Materials.
    let mut texes: Vec<Tex> = Vec::new();
    let mut tex_of: HashMap<usize, Option<usize>> = HashMap::new();
    let mut named_tex: HashMap<String, Option<usize>> = HashMap::new();
    let mut mats: HashMap<u32, Option<Mat>> = HashMap::new();
    // (After the fixes a placed model's vertex colour is no tint.)
    let world_mats: std::collections::HashSet<u32> = world
        .surfaces
        .iter()
        .filter_map(|s| s.material.map(|m| m.index))
        .collect();
    let mut resolve = |mi: u32| -> Option<()> {
        if mats.contains_key(&mi) {
            return mats[&mi].as_ref().map(|_| ());
        }
        let m = &cap.materials[mi as usize];
        // A material the zone only names: the shared zones' (its colour map
        // by name).
        let shared_mat = (m.techset.is_none() && m.textures.is_empty())
            .then(|| shared.material(&m.name))
            .flatten();
        let ts = match shared_mat {
            Some(sm) => sm.techset.clone(),
            None => m
                .techset
                .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                .unwrap_or_default(),
        };
        let words: Vec<&str> = ts.split('_').collect();
        let hidden = words.iter().any(|w| w.starts_with("sky"))
            || words.contains(&"tools")
            || words.contains(&"shadowcaster")
            || words.contains(&"onlycast")
            || words.contains(&"distortion")
            || (!before && words.contains(&"distfalloff"));
        if hidden {
            mats.insert(mi, None);
            return None;
        }
        let colour = m
            .textures
            .iter()
            .find(|t| t.name_hash == COLOR_MAP_HASH && t.image.is_some())
            .or_else(|| {
                m.textures
                    .iter()
                    .find(|t| t.semantic == 2 && t.image.is_some())
            });
        let named_colour = shared_mat.and_then(|sm| {
            sm.textures
                .iter()
                .find(|t| t.0 == COLOR_MAP_HASH)
                .or_else(|| sm.textures.iter().find(|t| t.2 == 2))
                .map(|t| t.3.clone())
        });
        let tex = if let Some(name) = named_colour {
            *named_tex.entry(name.clone()).or_insert_with(|| {
                let px = fastfile_iw6::shared::builtin_pixels(&name)
                    .map_or_else(|| shared.pixels(&name, &mut paks, tex_max), Ok)
                    .ok()?;
                let t = decode(&px)?;
                texes.push(t);
                Some(texes.len() - 1)
            })
        } else {
            colour.and_then(|t| {
                let ii = t.image.unwrap().index as usize;
                *tex_of.entry(ii).or_insert_with(|| {
                    let px =
                        load_px(&cap.images[ii], &zone.image, &shared, &mut paks, tex_max).ok()?;
                    let t = decode(&px)?;
                    texes.push(t);
                    Some(texes.len() - 1)
                })
            })
        };
        let mut layers = [None, None];
        let mut any_b1 = false;
        for w in &words {
            any_b1 |= w.starts_with("b1");
            // (`b1ha1c1`: a height-blended layer; the game's reader wants
            // the colour map right after the layer number.)
            let stripped = if before { w.to_string() } else { strip_ha(w) };
            let b = stripped.as_bytes();
            if b.len() >= 4
                && matches!(b[0], b'b' | b'm' | b't' | b'a')
                && (b[1] == b'1' || b[1] == b'2')
                && b[2] == b'c'
            {
                let k = usize::from(b[1] - b'1');
                let hash = if k == 0 { 0xb60d_1850 } else { 0xb60d_1853 };
                let t = m
                    .textures
                    .iter()
                    .find(|t| t.name_hash == hash)
                    .and_then(|t| t.image)
                    .and_then(|id| {
                        let ii = id.index as usize;
                        *tex_of.entry(ii).or_insert_with(|| {
                            let px =
                                load_px(&cap.images[ii], &zone.image, &shared, &mut paks, tex_max)
                                    .ok()?;
                            let t = decode(&px)?;
                            texes.push(t);
                            Some(texes.len() - 1)
                        })
                    });
                layers[k] = Some((b[0], t));
            }
        }
        let tint = !(any_b1 || layers.iter().any(Option::is_some))
            && !(!before && !world_mats.contains(&mi))
            && !(!before
                && words
                    .iter()
                    .any(|w| w.starts_with("wv") || w.starts_with("flag")));
        let blend_mode = if words.contains(&"add") {
            2u8
        } else if words.contains(&"multiply") {
            3
        } else if words.contains(&"blend") || words.iter().any(|w| w.starts_with("b0c0")) {
            1
        } else {
            0
        };
        let alpha_test = words.iter().any(|w| {
            let b = w.as_bytes();
            b.len() >= 4 && (b[0] == b't' || b[0] == b'b') && b[1] == b'0' && b[2] == b'c'
        });
        let h = mi.wrapping_mul(2654435761);
        let id_colour = [
            (h & 255) as f32 / 255.0,
            ((h >> 8) & 255) as f32 / 255.0,
            ((h >> 16) & 255) as f32 / 255.0,
        ];
        mats.insert(
            mi,
            Some(Mat {
                tex,
                layers,
                tint,
                blend_mode,
                alpha_test,
                id_colour,
            }),
        );
        Some(())
    };

    // Triangles: (material, three vertices, has lightmap).
    let mut tris: Vec<(u32, [Vtx; 3], bool, u32)> = Vec::new();
    // Each triangle's lightmap page (255: none).
    let mut tri_lm: Vec<u8> = Vec::new();
    // (After the fixes, a second level above the first is left out.)
    let cut = if before {
        None
    } else {
        fastfile_iw6::map::alternate_level_cut(&map.spawns)
    };
    eprintln!("second level cut: {cut:?}");
    // Brush models their entities place (after the fixes): surface -> place.
    let mut placed: HashMap<usize, ([f32; 3], [[f32; 3]; 3])> = HashMap::new();
    if !before {
        for b in fastfile_iw6::map::placed_brush_models(&map, cut) {
            for i in b.surfaces {
                placed.entry(i).or_insert((b.origin, b.axis));
            }
        }
        eprintln!("placed brush model surfaces: {}", placed.len());
    }
    let in_range = |i: u32| world.ranges.iter().any(|&(a, b)| i >= a.min(b) && i < b);
    for (si, s) in world.surfaces.iter().enumerate() {
        let Some(m) = s.material else { continue };
        // (As the game: surfaces outside the dpvs ranges are brush models'
        // and other entities', drawn by their entities.)
        let place = if in_range(si as u32) {
            None
        } else {
            placed.get(&si).copied()
        };
        if !in_range(si as u32)
            && place.is_none()
            && std::env::var_os("IW6_RENDER_ALL_SURFACES").is_none()
        {
            continue;
        }
        if cut.is_some_and(|c| s.mins[2] > c) {
            continue;
        }
        if resolve(m.index).is_none() {
            continue;
        }
        for t in 0..s.tri_count as usize {
            let v: [Vtx; 3] = std::array::from_fn(|k| {
                let ix = world.indices[s.base_index as usize + 3 * t + k] as usize;
                let wv = &world.vertices[s.first_vertex as usize + ix];
                let c = wv.color.to_le_bytes();
                let c = if bgra { [c[2], c[1], c[0], c[3]] } else { c };
                let rot = |v: [f32; 3], a: &[[f32; 3]; 3]| -> [f32; 3] {
                    std::array::from_fn(|k| a[0][k] * v[0] + a[1][k] * v[1] + a[2][k] * v[2])
                };
                let (p, n) = match &place {
                    Some((o, a)) => {
                        let r = rot(wv.xyz, a);
                        (std::array::from_fn(|k| o[k] + r[k]), rot(wv.normal, a))
                    }
                    None => (wv.xyz, wv.normal),
                };
                Vtx {
                    p,
                    uv: wv.tex_coord,
                    lm: wv.lmap_coord,
                    col: c.map(|b| b as f32 / 255.0),
                    n,
                }
            });
            tris.push((m.index, v, true, si as u32));
            tri_lm.push(s.lightmap_index);
        }
    }
    let world_tris = tris.len();
    if models {
        let mut cache: HashMap<u32, Vec<(u32, Vec<Vtx>, Vec<u16>)>> = HashMap::new();
        for (mi_, sm) in world.static_models.iter().enumerate() {
            let Some(id) = sm.model else { continue };
            if cut.is_some_and(|c| sm.origin[2] > c) {
                continue;
            }
            let parts = cache.entry(id.index).or_insert_with(|| {
                let mut out = Vec::new();
                let Some(m) = cap.xmodels.get(id.index as usize) else {
                    return out;
                };
                let Some(lod) = m.lods.first() else {
                    return out;
                };
                let Some(surfs) = lod
                    .surfs
                    .and_then(|s| cap.model_surfs.get(s.index as usize))
                else {
                    return out;
                };
                for (i, s) in surfs.surfs.iter().enumerate() {
                    let Some(mat) = m
                        .materials
                        .get(usize::from(lod.surf_index) + i)
                        .copied()
                        .flatten()
                    else {
                        continue;
                    };
                    let vb = s.verts.map_or(&[][..], |sp| sp.get(&zone.image));
                    let verts: Vec<Vtx> = vb
                        .chunks_exact(32)
                        .map(|v| {
                            let f =
                                |o: usize| f32::from_le_bytes([v[o], v[o + 1], v[o + 2], v[o + 3]]);
                            let hf = |o: usize| {
                                fastfile_iw6::map::half_to_f32(u16::from_le_bytes([v[o], v[o + 1]]))
                            };
                            Vtx {
                                p: [f(0), f(4), f(8)],
                                uv: [hf(20), hf(22)],
                                lm: [0.0; 2],
                                col: [v[16], v[17], v[18], v[19]].map(|b| b as f32 / 255.0),
                                n: fastfile_iw6::map::unpack_unit_vec(u32::from_le_bytes([
                                    v[24], v[25], v[26], v[27],
                                ])),
                            }
                        })
                        .collect();
                    let ib = s.tris.map_or(&[][..], |sp| sp.get(&zone.image));
                    let idx: Vec<u16> = ib
                        .chunks_exact(2)
                        .map(|b| u16::from_le_bytes([b[0], b[1]]))
                        .collect();
                    out.push((mat.index, verts, idx));
                }
                out
            });
            for (matid, verts, idx) in parts.iter() {
                if resolve(*matid).is_none() {
                    continue;
                }
                let place = |v: &Vtx| -> Vtx {
                    let mut o = *v;
                    for k in 0..3 {
                        o.p[k] = sm.origin[k]
                            + sm.scale
                                * (sm.axis[0][k] * v.p[0]
                                    + sm.axis[1][k] * v.p[1]
                                    + sm.axis[2][k] * v.p[2]);
                        o.n[k] = sm.axis[0][k] * v.n[0]
                            + sm.axis[1][k] * v.n[1]
                            + sm.axis[2][k] * v.n[2];
                    }
                    o
                };
                for t in idx.chunks_exact(3) {
                    if t.iter().any(|&i| i as usize >= verts.len()) {
                        continue;
                    }
                    tris.push((
                        *matid,
                        [
                            place(&verts[t[0] as usize]),
                            place(&verts[t[1] as usize]),
                            place(&verts[t[2] as usize]),
                        ],
                        false,
                        1_000_000 + mi_ as u32,
                    ));
                }
            }
        }
    }
    eprintln!(
        "{} world tris, {} model tris, {} textures decoded",
        world_tris,
        tris.len() - world_tris,
        texes.len()
    );

    // Lightmap images and the coordinates' range.
    for (i, pair) in world.lightmaps.iter().enumerate() {
        for id in [pair.0, pair.1].into_iter().flatten() {
            let r = &cap.images[id.index as usize];
            eprintln!(
                "lightmap {i}: {} f{} {}x{} depth {} elements {} levels {} map_type {} streamed {} streams {:?}",
                r.name,
                r.format,
                r.width,
                r.height,
                r.depth,
                r.num_elements,
                r.level_count,
                r.map_type,
                r.streamed,
                r.streams
            );
        }
    }
    {
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        let mut per_index: HashMap<u8, usize> = HashMap::new();
        for s in &world.surfaces {
            *per_index.entry(s.lightmap_index).or_default() += 1;
            for v in &world.vertices
                [s.first_vertex as usize..s.first_vertex as usize + s.vertex_count as usize]
            {
                for k in 0..2 {
                    lo[k] = lo[k].min(v.lmap_coord[k]);
                    hi[k] = hi[k].max(v.lmap_coord[k]);
                }
            }
        }
        eprintln!("lightmap coords {lo:?}..{hi:?}; surfaces per lightmap index {per_index:?}");
    }
    // The secondary's raw values: per page, channel percentiles.
    if let Some(sid) = world.lightmaps.first().and_then(|p| p.1) {
        let r = &cap.images[sid.index as usize];
        let got = image_pixels(r, &zone.image, &mut paks, 1 << 14);
        if let Err(e) = &got {
            eprintln!("secondary: {e}");
        }
        if let Ok(px) = got {
            let (w, h) = (px.width as usize, px.height as usize / 2);
            for page in 0..2 {
                let mut ch: [Vec<f32>; 4] = Default::default();
                for i in 0..w * h {
                    let at = (page * w * h + i) * 8;
                    for c in 0..4 {
                        ch[c].push(fastfile_iw6::map::half_to_f32(u16::from_le_bytes([
                            px.data[at + 2 * c],
                            px.data[at + 2 * c + 1],
                        ])));
                    }
                }
                for (c, v) in ch.iter_mut().enumerate() {
                    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let q = |f: f32| v[((v.len() - 1) as f32 * f) as usize];
                    eprintln!(
                        "secondary page {page} ch {c}: min {:.3} p10 {:.3} p50 {:.3} p90 {:.3} p99 {:.3} max {:.3}",
                        q(0.0),
                        q(0.1),
                        q(0.5),
                        q(0.9),
                        q(0.99),
                        q(1.0)
                    );
                }
            }
        }
    }
    // Lightmap page 0 as the game converts it.
    let lights: Vec<Option<Light>> = world
        .lightmaps
        .iter()
        .map(|&pair| {
            let lm = fastfile_iw6::map::iw3_lightmap_exposed(&cap, &zone.image, pair, exposure)
                .map_err(|e| eprintln!("lightmap: {e}"))
                .ok()?;
            let (pw, ph, p) = lm.primary;
            let (sw, sh, s) = lm.secondary;
            let (w, h) = (sw as usize, sh as usize / 2);
            eprintln!(
                "lightmap: primary {pw}x{ph}, secondary {sw}x{sh}; {} pages in the world",
                world.lightmaps.len()
            );
            let px: Vec<[u8; 4]> = s
                .chunks_exact(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect();
            let vis: Vec<u8> = if (pw as usize, ph as usize) == (w, h) {
                p
            } else {
                (0..w * h)
                    .map(|i| {
                        p[((i / w) * ph as usize / h) * pw as usize + (i % w) * pw as usize / w]
                    })
                    .collect()
            };
            if std::env::var_os("IW6_RENDER_DUMP_LM").is_some() {
                let rgb = |pg: &[[u8; 4]]| {
                    pg.iter()
                        .flat_map(|c| [c[2], c[1], c[0]])
                        .collect::<Vec<u8>>()
                };
                save(&format!("{prefix}_lmA.png"), w, h, &rgb(&px[..w * h]));
                save(&format!("{prefix}_lmB.png"), w, h, &rgb(&px[w * h..]));
                save_grey(&format!("{prefix}_lmVis.png"), w, h, &vis);
            }
            Some(Light {
                w,
                h,
                a: px[..w * h].to_vec(),
                b: px[w * h..2 * w * h].to_vec(),
                vis,
            })
        })
        .collect();

    // Sun (as the game's: gamma-space colour under the same exposure).
    let (sun_dir, sun_col) = match map.sun {
        Some(s) => {
            let peak = s.color.iter().copied().fold(0.0f32, f32::max).max(1e-6);
            let g = if before {
                (peak.sqrt() * 0.5).clamp(0.6, 1.6)
            } else {
                (peak.max(0.0) * exposure).sqrt().min(sun_max)
            };
            eprintln!("sun: peak {peak} -> gamma {g} (exposure {exposure})");
            (
                norm(s.dir),
                [
                    s.color[0] / peak * g,
                    s.color[1] / peak * g,
                    s.color[2] / peak * g,
                ],
            )
        }
        None => (norm([0.5, 0.2, 0.8]), [1.0; 3]),
    };

    // Spawn area bounds.
    let (mut mn, mut mx) = ([f32::MAX; 2], [f32::MIN; 2]);
    for s in &map.spawns {
        for k in 0..2 {
            mn[k] = mn[k].min(s.origin[k]);
            mx[k] = mx[k].max(s.origin[k]);
        }
    }
    for view in &views {
        let cam = if view == "top" {
            let pad = 600.0;
            let (mut a, mut b) = ([mn[0] - pad, mn[1] - pad], [mx[0] + pad, mx[1] + pad]);
            // Keep the aspect.
            let (cx, cy) = ((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0);
            let (hw, hh) = ((b[0] - a[0]) / 2.0, (b[1] - a[1]) / 2.0);
            let s = (hw / W as f32).max(hh / H as f32);
            a = [cx - s * W as f32, cy - s * H as f32];
            b = [cx + s * W as f32, cy + s * H as f32];
            Cam {
                ortho: Some((a, b)),
                pos: [0.0; 3],
                fwd: [0.0, 0.0, -1.0],
                right: [1.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                f: 1.0,
            }
        } else if let Some(n) = view.strip_prefix("spawn") {
            // `spawnN` at eye height, its facing; `spawnN_up` looking up.
            let (n, up) = match n.strip_suffix("_up") {
                Some(n) => (n, true),
                None => (n, false),
            };
            let n: usize = n.parse().unwrap_or(0);
            let s = &map.spawns[n.min(map.spawns.len() - 1)];
            Cam::persp(
                [s.origin[0], s.origin[1], s.origin[2] + 60.0],
                s.angles[1],
                if up { -30.0 } else { 5.0 },
            )
        } else {
            let v: Vec<f32> = view.split(',').filter_map(|x| x.parse().ok()).collect();
            Cam::persp([v[0], v[1], v[2]], v[3], v[4])
        };
        let mut ctx = Ctx {
            colour: vec![[0.25, 0.35, 0.5]; W * H],
            depth: vec![f32::MAX; W * H],
            what: vec![(u32::MAX, false); W * H],
            cur: (0, false),
            mode: 0,
        };
        let mut order: Vec<usize> = (0..tris.len()).collect();
        let key = |i: usize| {
            mats.get(&tris[i].0)
                .and_then(|m| m.as_ref())
                .map_or(0, |m| {
                    if m.blend_mode == 0 {
                        0
                    } else {
                        1 + cap.materials[tris[i].0 as usize].sort_key as u32
                    }
                })
        };
        order.sort_by_key(|&i| key(i));
        for &ti in &order {
            let (mi, v, has_lm, surf) = &tris[ti];
            let light = tri_lm
                .get(ti)
                .and_then(|&l| lights.get(usize::from(l)))
                .and_then(Option::as_ref);
            let Some(Some(m)) = mats.get(mi) else {
                continue;
            };
            ctx.cur = (*mi, *has_lm);
            ctx.mode = m.blend_mode;
            raster(&cam, v, &mut ctx, |a: &Vtx| -> Option<([f32; 3], f32)> {
                let tex = m.tex.map(|t| texes[t].sample(a.uv[0], a.uv[1]));
                if m.alpha_test && tex.is_some_and(|t| t[3] < 0.5) {
                    return None;
                }
                let srgb = |c: f32| c * c; // (the game's approximation: squares)
                let mut albedo = match tex {
                    Some(t) if !m.tint => [t[0], t[1], t[2]],
                    Some(t) => [t[0] * a.col[0], t[1] * a.col[1], t[2] * a.col[2]],
                    None => [1.0, 0.0, 1.0],
                };
                // The layers, as the game's fallback draw: weights in the
                // vertex colour's green (layer 1) and blue (layer 2).
                for (k, l) in m.layers.iter().enumerate() {
                    let Some((kind, Some(t))) = l else { continue };
                    let s = texes[*t].sample(a.uv[0], a.uv[1]);
                    let wgt = a.col[1 + k];
                    for c in 0..3 {
                        albedo[c] = match kind {
                            b'b' => albedo[c] + (s[c] - albedo[c]) * s[3] * wgt,
                            b'm' => albedo[c] * (1.0 + wgt * (s[c] - 1.0)),
                            b'a' => albedo[c] + s[c] * s[3] * wgt,
                            _ => {
                                if s[3] * wgt >= 0.5 {
                                    s[c]
                                } else {
                                    albedo[c]
                                }
                            }
                        };
                    }
                }
                let n = norm(a.n);
                let ndl = dot(n, sun_dir).max(0.0);
                let lit = |base: [f32; 3]| -> [f32; 3] {
                    match (light, *has_lm) {
                        (Some(l), true) => {
                            let (la, lb, aa, ba, vis) = l.at(a.lm[0], a.lm[1]);
                            let x = aa * 4.08 - 2.08;
                            let y = ba * 4.064516 - 2.064516;
                            let f = (1.0 / (1.0 + x * x + y * y).sqrt()).clamp(0.0, 1.0);
                            std::array::from_fn(|k| {
                                let lg = la[k] + lb[k] * f + sun_col[k] * vis * ndl;
                                base[k] * lg * lg
                            })
                        }
                        _ => std::array::from_fn(|k| {
                            base[k] * (0.35 + sun_col[k] * sun_col[k] * ndl * 0.6)
                        }),
                    }
                };
                let out = match mode.as_str() {
                    "albedo" => albedo.map(srgb),
                    "light" => lit([1.0; 3]),
                    "id" => {
                        let h = surf.wrapping_mul(2654435761);
                        let c = [
                            (h & 255) as f32 / 255.0,
                            ((h >> 8) & 255) as f32 / 255.0,
                            ((h >> 16) & 255) as f32 / 255.0,
                        ];
                        let _ = m.id_colour;
                        c.map(|x| (0.3 + 0.7 * x) * (0.4 + 0.6 * ndl)).map(srgb)
                    }
                    "untex" => {
                        if tex.is_some() {
                            [0.5 * (0.4 + 0.6 * ndl); 3].map(srgb)
                        } else {
                            [1.0, 0.0, 1.0]
                        }
                    }
                    _ => lit(albedo.map(srgb)),
                };
                let alpha = tex.map_or(1.0, |t| t[3]);
                Some((out, alpha))
            });
        }
        // What covers the picture.
        let mut cover: HashMap<(u32, bool), usize> = HashMap::new();
        for w in &ctx.what {
            if w.0 != u32::MAX {
                *cover.entry(*w).or_default() += 1;
            }
        }
        let mut cover: Vec<_> = cover.into_iter().collect();
        cover.sort_by(|a, b| b.1.cmp(&a.1));
        for ((mi, world), px) in cover.iter().take(10) {
            let m = &cap.materials[*mi as usize];
            let ts = m
                .techset
                .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                .unwrap_or_default();
            eprintln!(
                "  {view}: {:>5.1}% {} {} [{ts}] sort {}",
                100.0 * *px as f32 / (W * H) as f32,
                if *world { "world" } else { "model" },
                m.name,
                m.sort_key
            );
        }
        if let Ok(probe) = std::env::var("IW6_PROBE") {
            for xy in probe.split(';') {
                let v: Vec<usize> = xy
                    .split(',')
                    .filter_map(|t| t.trim().parse().ok())
                    .collect();
                if v.len() != 2 || v[0] >= W || v[1] >= H {
                    continue;
                }
                let (mi, world) = ctx.what[v[1] * W + v[0]];
                if mi == u32::MAX {
                    continue;
                }
                let m = &cap.materials[mi as usize];
                let ts = m
                    .techset
                    .map(|t| cap.name_of(t).trim_start_matches(',').to_owned())
                    .unwrap_or_default();
                let tex: Vec<String> = m
                    .textures
                    .iter()
                    .map(|t| {
                        let r = t.image.map(|i| &cap.images[i.index as usize]);
                        format!(
                            "{:08x}/s{}={}",
                            t.name_hash,
                            t.semantic,
                            r.map_or("-".to_owned(), |r| format!(
                                "{}[f{} {}x{}]",
                                r.name, r.format, r.width, r.height
                            ))
                        )
                    })
                    .collect();
                let c = ctx.colour[v[1] * W + v[0]];
                eprintln!(
                    "  probe {view} {xy}: {} {} [{ts}] colour {c:?} textures {}",
                    if world { "world" } else { "model" },
                    m.name,
                    tex.join(" ")
                );
            }
        }
        // Linear -> display (square root, as the game's output).
        let bytes: Vec<u8> = ctx
            .colour
            .iter()
            .flat_map(|c| c.map(|x| (x.max(0.0).sqrt().min(1.0) * 255.0) as u8))
            .collect();
        let name = format!("{prefix}_{}_{mode}.png", view.replace(',', "_"));
        save(&name, W, H, &bytes);
        eprintln!("wrote {name}");
    }
}

fn raster(cam: &Cam, v: &[Vtx; 3], ctx: &mut Ctx, shade: impl Fn(&Vtx) -> Option<([f32; 3], f32)>) {
    // Clip against the near plane (perspective).
    let near = 4.0;
    let mut poly: Vec<Vtx> = v.to_vec();
    if cam.ortho.is_none() {
        let d: Vec<f32> = poly.iter().map(|p| cam.depth(p.p) - near).collect();
        if d.iter().all(|&x| x < 0.0) {
            return;
        }
        if d.iter().any(|&x| x < 0.0) {
            let mut out = Vec::new();
            for i in 0..3 {
                let (a, b) = (&poly[i], &poly[(i + 1) % 3]);
                let (da, db) = (d[i], d[(i + 1) % 3]);
                if da >= 0.0 {
                    out.push(*a);
                }
                if (da >= 0.0) != (db >= 0.0) {
                    out.push(lerpv(a, b, da / (da - db)));
                }
            }
            poly = out;
        }
    }
    for i in 1..poly.len().saturating_sub(1) {
        tri(cam, [&poly[0], &poly[i], &poly[i + 1]], ctx, &shade);
    }
}

fn tri(cam: &Cam, v: [&Vtx; 3], ctx: &mut Ctx, shade: &impl Fn(&Vtx) -> Option<([f32; 3], f32)>) {
    let s: [[f32; 3]; 3] = std::array::from_fn(|k| cam.project(v[k].p));
    let area =
        (s[1][0] - s[0][0]) * (s[2][1] - s[0][1]) - (s[2][0] - s[0][0]) * (s[1][1] - s[0][1]);
    if area.abs() < 1e-6 {
        return;
    }
    let x0 = s.iter().map(|p| p[0]).fold(f32::MAX, f32::min).max(0.0) as usize;
    let x1 = (s
        .iter()
        .map(|p| p[0])
        .fold(f32::MIN, f32::max)
        .min(W as f32 - 1.0)) as isize;
    let y0 = s.iter().map(|p| p[1]).fold(f32::MAX, f32::min).max(0.0) as usize;
    let y1 = (s
        .iter()
        .map(|p| p[1])
        .fold(f32::MIN, f32::max)
        .min(H as f32 - 1.0)) as isize;
    if x1 < 0 || y1 < 0 {
        return;
    }
    let persp = cam.ortho.is_none();
    let iz: [f32; 3] = std::array::from_fn(|k| if persp { 1.0 / s[k][2] } else { 1.0 });
    for y in y0..=y1 as usize {
        for x in x0..=x1 as usize {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = ((s[1][0] - px) * (s[2][1] - py) - (s[2][0] - px) * (s[1][1] - py)) / area;
            let w1 = ((s[2][0] - px) * (s[0][1] - py) - (s[0][0] - px) * (s[2][1] - py)) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let z = w0 * s[0][2] + w1 * s[1][2] + w2 * s[2][2];
            let z = if persp {
                1.0 / (w0 * iz[0] + w1 * iz[1] + w2 * iz[2])
            } else {
                z
            };
            let i = y * W + x;
            if z >= ctx.depth[i] {
                continue;
            }
            // Perspective-correct weights.
            let (a, b, c) = if persp {
                (w0 * iz[0] * z, w1 * iz[1] * z, w2 * iz[2] * z)
            } else {
                (w0, w1, w2)
            };
            let l = |f: &dyn Fn(&Vtx) -> f32| a * f(v[0]) + b * f(v[1]) + c * f(v[2]);
            let at = Vtx {
                p: std::array::from_fn(|k| l(&|q| q.p[k])),
                uv: std::array::from_fn(|k| l(&|q| q.uv[k])),
                lm: std::array::from_fn(|k| l(&|q| q.lm[k])),
                col: std::array::from_fn(|k| l(&|q| q.col[k])),
                n: std::array::from_fn(|k| l(&|q| q.n[k])),
            };
            if let Some((c, a)) = shade(&at) {
                let d = ctx.colour[i];
                match ctx.mode {
                    0 => {
                        ctx.depth[i] = z;
                        ctx.colour[i] = c;
                    }
                    1 => ctx.colour[i] = std::array::from_fn(|k| d[k] + (c[k] - d[k]) * a),
                    2 => ctx.colour[i] = std::array::from_fn(|k| d[k] + c[k] * a),
                    _ => ctx.colour[i] = std::array::from_fn(|k| d[k] * c[k]),
                }
                ctx.what[i] = ctx.cur;
            }
        }
    }
}

fn load_px(
    rec: &fastfile_iw6::walk::ImageRec,
    zone_image: &[u8],
    shared: &fastfile_iw6::shared::SharedImages,
    paks: &mut ImagePaks,
    max: u32,
) -> Result<ImagePixels, String> {
    if fastfile_iw6::shared::is_reference(rec) && !shared.is_empty() {
        if let Some(px) = fastfile_iw6::shared::builtin_pixels(&rec.name) {
            return Ok(px);
        }
        return shared.pixels(&rec.name, paks, max);
    }
    image_pixels(rec, zone_image, paks, max)
}

fn strip_ha(w: &str) -> String {
    let b = w.as_bytes();
    if b.len() >= 5
        && matches!(b[0], b'r' | b'b' | b'm' | b't' | b'a')
        && b[1].is_ascii_digit()
        && b[2] == b'h'
        && b[3].is_ascii_lowercase()
    {
        let rest = &w[4..];
        let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        format!("{}{}", &w[..2], rest)
    } else {
        w.to_string()
    }
}

fn save(path: &str, w: usize, h: usize, rgb: &[u8]) {
    let f = std::fs::File::create(path).expect("create");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(rgb).unwrap();
}

fn save_grey(path: &str, w: usize, h: usize, g: &[u8]) {
    let f = std::fs::File::create(path).expect("create");
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    e.set_color(png::ColorType::Grayscale);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(g).unwrap();
}
