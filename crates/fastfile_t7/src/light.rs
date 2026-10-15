//! bo2mp: a Black Ops III map's own light, read from its `GfxWorld`'s
//! first sun volume (`walk::gfxworld::SunVolumeLighting`):
//!
//! * the sky: the sky box model's material (`mc/sky_latlong_hdr`) names a
//!   lat-long picture (BC6H, 8192 x 4096 on the MP maps) whose pixels are
//!   the sun volume's sky `StreamWrappedBuffer` (an `.xpak` entry: the
//!   image record itself has none). Its `$Globals` hold `skyScale`. The sky
//!   pixel shader (`sky_latlong_*hdr`, fxc disassembly) looks it up by the
//!   view direction d: `u = 0.5 - atan2(d.y, d.x) / 2pi`, `v = acos(d.z) /
//!   pi` (row 0 straight up), times `skyScale`;
//! * the reflection probes: one BC6H cube array (`count` probes x 6 faces,
//!   each face its whole mip chain, 256 px and 7 levels on the MP maps) in
//!   the probe array's buffer, and one 328-byte config per probe (config 0
//!   the map-wide one): its mean colour (floats 1..3), its origin (11..13),
//!   its rotation (14..22, rows), the half size of its box (23..25), its
//!   parallax box (26..28) and offset (29..31), its radius (32);
//! * the light the forward `lit` shader gives a surface that is not in the
//!   sun is the probe volume's directional term times the probe cube's
//!   smallest level in the normal's direction (`sample_l ... t8 ... l(6)`);
//!   here the volume's term is left out (one) and each probe's smallest
//!   level is kept as an ambient cube (`Bake::ambient`).
//!
//! Units are the game's linear HDR ones (a probe face of daylit ground
//! about 1).

use crate::map::T7Map;
use crate::walk::capture::Capture;

const PROBE_CONFIG: usize = 328;

/// Where the first sun volume keeps its light, as the walk met it.
#[derive(Clone, Debug, Default)]
pub struct LightRefs {
    pub sky: Option<SkyRef>,
    /// The sky material's `skyScale` (1 when it has none).
    pub sky_scale: f32,
    pub sky_material: String,
    pub probes: Option<ProbeRef>,
    /// The `GfxWorld` header's LUT material's picture, when it has one.
    pub lut_image: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SkyRef {
    pub image: String,
    pub key: u64,
    pub width: u32,
    pub height: u32,
    pub format: u32,
}

#[derive(Clone, Debug)]
pub struct ProbeRef {
    pub key: u64,
    pub size: u32,
    pub levels: u32,
    pub count: u32,
    pub format: u32,
    /// The configs' bytes (`count` x 328).
    pub configs: Vec<u8>,
}

fn u64_at(b: &[u8], o: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(o..o + 8)?.try_into().ok()?))
}
fn f32_at(b: &[u8], o: usize) -> f32 {
    b.get(o..o + 4).map_or(0.0, |s| f32::from_le_bytes(s.try_into().unwrap()))
}

fn bare(n: &str) -> &str {
    n.trim_start_matches(',')
}

impl LightRefs {
    pub(crate) fn read(cap: &Capture, image: &[u8], common: &crate::map::Common) -> Option<Self> {
        let g = cap.world.gfx_worlds.first()?;
        let l = g.sun_lighting.first()?;
        let mut out = LightRefs { sky_scale: 1.0, ..Default::default() };
        // The sky: the material of the first sky box model, its colour map,
        // the sky buffer's entry.
        let sky_model = g.sky_models.first().map(|m| bare(m).to_owned()).unwrap_or_default();
        let sky_mat = cap
            .materials
            .iter()
            .filter(|m| m.techset.as_deref().is_some_and(|t| t.contains("sky")))
            .find(|m| !sky_model.is_empty() && m.name.contains(&sky_model))
            .or_else(|| cap.materials.iter().find(|m| m.techset.as_deref().is_some_and(|t| t.contains("sky_latlong"))));
        if let Some(m) = sky_mat {
            out.sky_material = m.name.clone();
            let (c, _) = crate::map::material_constants(cap, image, common, m);
            if let Some(s) = c.get("skyScale").and_then(|v| v.first()).filter(|s| s.is_finite() && **s > 0.0) {
                out.sky_scale = *s;
            }
            let im = m.textures.first().and_then(|(_, n)| cap.image(bare(n)).or_else(|| cap.image(n)));
            let key = l.sky_buffers.first().copied().flatten().and_then(|b| u64_at(image, b + 8));
            if let (Some(im), Some(key)) = (im, key) {
                out.sky = Some(SkyRef { image: im.name.clone(), key, width: u32::from(im.width), height: u32::from(im.height), format: im.format });
            }
        }
        // The reflection probes (state 0).
        if let Some((Some(name), Some((at, n)), _)) = l.reflection_probes.first()
            && let Some(im) = cap.image(bare(name)).or_else(|| cap.image(name))
            && let Some(key) = l.probe_buffers.first().copied().flatten().and_then(|b| u64_at(image, b + 8))
        {
            let count = (u32::from(im.depth)).min(*n as u32);
            let configs = image.get(*at..at + PROBE_CONFIG * count as usize).map(<[u8]>::to_vec).unwrap_or_default();
            out.probes = Some(ProbeRef {
                key,
                size: u32::from(im.width),
                levels: u32::from(im.levels).max(1),
                count: (configs.len() / PROBE_CONFIG) as u32,
                format: im.format,
                configs,
            });
        }
        out.lut_image = g
            .header_materials
            .iter()
            .flatten()
            .find_map(|m| cap.material(bare(m)).or_else(|| cap.material(m)))
            .and_then(|m| m.textures.first().map(|(_, n)| bare(n).to_owned()));
        Some(out)
    }
}

/// A lat-long sky picture, linear HDR RGB rows from the top (straight up).
pub struct SkyPicture {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<[f32; 3]>,
    pub scale: f32,
}

impl SkyPicture {
    /// The sky in direction `d` (world axes), as its shader reads it,
    /// times `skyScale`.
    pub fn at(&self, d: [f32; 3]) -> [f32; 3] {
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        let (x, y, z) = (d[0] / n, d[1] / n, d[2] / n);
        let u = 0.5 - y.atan2(x) / std::f32::consts::TAU;
        let v = z.clamp(-1.0, 1.0).acos() / std::f32::consts::PI;
        let (w, h) = (self.width as f32, self.height as f32);
        let fx = (u * w - 0.5).rem_euclid(w);
        let fy = (v * h - 0.5).clamp(0.0, h - 1.0);
        let (x0, y0) = (fx.floor() as usize % self.width as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1) % self.width as usize, (y0 + 1).min(self.height as usize - 1));
        let (tx, ty) = (fx - fx.floor(), fy - fy.floor());
        let p = |x: usize, y: usize| self.rgb[y * self.width as usize + x];
        std::array::from_fn(|k| {
            let a = p(x0, y0)[k] + (p(x1, y0)[k] - p(x0, y0)[k]) * tx;
            let b = p(x0, y1)[k] + (p(x1, y1)[k] - p(x0, y1)[k]) * tx;
            (a + (b - a) * ty) * self.scale
        })
    }

    /// The brightest texel's direction (the sun's, on a day sky) and value.
    pub fn brightest(&self) -> ([f32; 3], f32) {
        let (mut best, mut at) = (0f32, 0usize);
        for (i, c) in self.rgb.iter().enumerate() {
            let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            if l > best {
                best = l;
                at = i;
            }
        }
        let (x, y) = (at % self.width as usize, at / self.width as usize);
        let u = (x as f32 + 0.5) / self.width as f32;
        let v = (y as f32 + 0.5) / self.height as f32;
        let phi = (0.5 - u) * std::f32::consts::TAU;
        let theta = v * std::f32::consts::PI;
        ([theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos()], best)
    }
}

fn bc6_block(block: &[u8]) -> [[f32; 3]; 16] {
    let mut t = [0f32; 48];
    bcdec_rs::bc6h_float(block, &mut t, 12, false);
    std::array::from_fn(|i| [t[3 * i], t[3 * i + 1], t[3 * i + 2]].map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }))
}

/// One reflection probe: its box and its smallest level's six faces.
#[derive(Clone, Debug)]
pub struct Probe {
    pub origin: [f32; 3],
    /// Rows: world to the probe's box axes.
    pub axes: [[f32; 3]; 3],
    pub half: [f32; 3],
    /// The mean colour the config stores.
    pub mean: [f32; 3],
    /// +X, -X, +Y, -Y, +Z, -Z (world axes) of the smallest level.
    pub faces: [[f32; 3]; 6],
}

impl Probe {
    /// Distance outside the box (0 inside), in units.
    fn outside(&self, p: [f32; 3]) -> f32 {
        let d = [p[0] - self.origin[0], p[1] - self.origin[1], p[2] - self.origin[2]];
        let mut s = 0.0;
        for k in 0..3 {
            let l = self.axes[k][0] * d[0] + self.axes[k][1] * d[1] + self.axes[k][2] * d[2];
            let o = (l.abs() - self.half[k]).max(0.0);
            s += o * o;
        }
        s.sqrt()
    }

    fn volume(&self) -> f32 {
        self.half[0] * self.half[1] * self.half[2]
    }

    /// The ambient cube in direction `n` (unit).
    pub fn ambient(&self, n: [f32; 3]) -> [f32; 3] {
        let mut c = [0f32; 3];
        for k in 0..3 {
            let w = n[k] * n[k];
            let f = &self.faces[2 * k + usize::from(n[k] < 0.0)];
            for j in 0..3 {
                c[j] += w * f[j];
            }
        }
        c
    }
}

/// D3D cube face `f`'s direction at face coordinates `u, w` in -1..1.
pub fn cube_dir(f: usize, u: f32, w: f32) -> [f32; 3] {
    match f {
        0 => [1.0, -w, -u],
        1 => [-1.0, -w, u],
        2 => [u, 1.0, w],
        3 => [u, -1.0, -w],
        4 => [u, -w, 1.0],
        _ => [-u, -w, -1.0],
    }
}

/// The map's baked light, read once: the sky and the probes.
pub struct Bake {
    pub sky: Option<SkyPicture>,
    pub probes: Vec<Probe>,
    /// The map's bounds (the probes' boxes but the map-wide one).
    pub bounds: [[f32; 3]; 2],
    /// Probe cube to world axes: world = (c.x, c.y, sign * c.z) (`CUBE_Z`).
    pub report: Vec<String>,
}

/// World direction to the probe cube's D3D axes (and back: its own inverse).
/// BO3 bakes its probes with world x, y, z as the cube's x, y, z.
pub fn world_to_cube(d: [f32; 3]) -> [f32; 3] {
    d
}

impl Bake {
    pub fn read(map: &T7Map) -> Result<Bake, String> {
        let refs = map.light_refs.as_ref().ok_or("no sun volume")?;
        let paks = map.paks().ok_or("no packages")?;
        let mut report = Vec::new();
        let sky = match &refs.sky {
            Some(s) if s.format == 95 || s.format == 96 => match paks.read(s.key) {
                Ok(d) => {
                    let pic = latlong(&d, s, 2048, refs.sky_scale)?;
                    let (dir, peak) = pic.brightest();
                    report.push(format!(
                        "sky {} {}x{} (drawn {}x{}), skyScale {}, brightest {peak:.2} toward {dir:.3?}",
                        s.image, s.width, s.height, pic.width, pic.height, refs.sky_scale
                    ));
                    Some(pic)
                }
                Err(e) => {
                    report.push(format!("sky {}: {e}", s.image));
                    None
                }
            },
            Some(s) => {
                report.push(format!("sky {}: format {} not read", s.image, s.format));
                None
            }
            None => {
                report.push(format!("sky: none ({})", if refs.sky_material.is_empty() { "no sky material" } else { "no buffer" }));
                None
            }
        };
        let mut probes = Vec::new();
        if let Some(p) = &refs.probes {
            let face: usize = (0..p.levels).map(|k| crate::texture::level_size(p.format, p.size >> k, p.size >> k).unwrap_or(0)).sum();
            let last: usize = (0..p.levels - 1).map(|k| crate::texture::level_size(p.format, p.size >> k, p.size >> k).unwrap_or(0)).sum();
            match paks.read(p.key) {
                Ok(d) if d.len() >= face * 6 * p.count as usize => {
                    for i in 0..p.count as usize {
                        let c = &p.configs[i * PROBE_CONFIG..(i + 1) * PROBE_CONFIG];
                        let w = |k: usize| f32_at(c, 4 * k);
                        let mut faces = [[0f32; 3]; 6];
                        for (f, out) in faces.iter_mut().enumerate() {
                            // The smallest level: 4x4 or less, one block.
                            let at = (6 * i + f) * face + last;
                            let px = bc6_block(&d[at..at + 16]);
                            let s = (p.size >> (p.levels - 1)).clamp(1, 4) as usize;
                            let mut sum = [0f32; 3];
                            for y in 0..s {
                                for x in 0..s {
                                    for k in 0..3 {
                                        sum[k] += px[y * 4 + x][k];
                                    }
                                }
                            }
                            *out = sum.map(|v| v / (s * s) as f32);
                        }
                        // Cube faces to world faces.
                        let cube = faces;
                        let mut world = [[0f32; 3]; 6];
                        for (f, wd) in [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0f32]].iter().enumerate() {
                            let cd = world_to_cube(*wd);
                            let cf = if cd[0].abs() > 0.5 {
                                usize::from(cd[0] < 0.0)
                            } else if cd[1].abs() > 0.5 {
                                2 + usize::from(cd[1] < 0.0)
                            } else {
                                4 + usize::from(cd[2] < 0.0)
                            };
                            world[f] = cube[cf];
                        }
                        probes.push(Probe {
                            origin: [w(11), w(12), w(13)],
                            axes: [[w(14), w(15), w(16)], [w(17), w(18), w(19)], [w(20), w(21), w(22)]],
                            half: [w(23), w(24), w(25)].map(|v| if v.is_finite() { v.abs() } else { 0.0 }),
                            mean: [w(1), w(2), w(3)],
                            faces: world,
                        });
                    }
                }
                Ok(d) => report.push(format!("probes: {} bytes, {} wanted", d.len(), face * 6 * p.count as usize)),
                Err(e) => report.push(format!("probes: {e}")),
            }
        }
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for pr in probes.iter().skip(1) {
            let r = pr.half[0].max(pr.half[1]).max(pr.half[2]);
            for k in 0..3 {
                lo[k] = lo[k].min(pr.origin[k] - r);
                hi[k] = hi[k].max(pr.origin[k] + r);
            }
        }
        if lo[0] > hi[0] {
            lo = map.mins;
            hi = map.maxs;
        }
        if let Some(g) = probes.first() {
            report.push(format!(
                "probes: {} (global probe faces {:.2?}), local boxes over {lo:.0?}..{hi:.0?}",
                probes.len(),
                g.faces.iter().map(|f| 0.2126 * f[0] + 0.7152 * f[1] + 0.0722 * f[2]).collect::<Vec<_>>()
            ));
        }
        Ok(Bake { sky, probes, bounds: [lo, hi], report })
    }

    /// The ambient light at `p` for a surface facing `n` (unit): the probe
    /// whose box holds `p` (the smallest such), blended toward the nearest
    /// box's within `BLEND` units of it, else the map-wide probe.
    pub fn ambient(&self, p: [f32; 3], n: [f32; 3]) -> [f32; 3] {
        const BLEND: f32 = 96.0;
        if self.probes.is_empty() {
            return [1.0; 3];
        }
        let mut best: Option<(f32, f32, usize)> = None;
        for (i, pr) in self.probes.iter().enumerate().skip(1) {
            let o = pr.outside(p);
            if o > BLEND {
                continue;
            }
            // Inside boxes first (smallest volume), then the nearest.
            let key = (o, pr.volume());
            if best.is_none_or(|(bo, bv, _)| (key.0, key.1) < (bo, bv)) {
                best = Some((key.0, key.1, i));
            }
        }
        let global = self.probes[0].ambient(n);
        match best {
            Some((o, _, i)) => {
                let local = self.probes[i].ambient(n);
                let t = (o / BLEND).clamp(0.0, 1.0);
                std::array::from_fn(|k| local[k] + (global[k] - local[k]) * t)
            }
            None => global,
        }
    }

    /// A short description of the probes (`t7_ambient`).
    pub fn describe(&self) -> String {
        let mut s = self.report.join("\n");
        for (i, p) in self.probes.iter().enumerate().take(24) {
            s.push_str(&format!(
                "\nprobe {i}: origin {:.0?} half {:.0?} mean {:.2?} up {:.2?} down {:.2?}",
                p.origin, p.half, p.mean, p.faces[4], p.faces[5]
            ));
        }
        s
    }
}

/// A BC6H lat-long picture averaged down to at most `max_w` wide (each
/// output pixel the mean of whole 4x4 blocks).
fn latlong(d: &[u8], s: &SkyRef, max_w: u32, scale: f32) -> Result<SkyPicture, String> {
    let (bw, bh) = (s.width.div_ceil(4) as usize, s.height.div_ceil(4) as usize);
    if d.len() < bw * bh * 16 {
        return Err(format!("sky {}: {} bytes, {} wanted", s.image, d.len(), bw * bh * 16));
    }
    // Blocks per output pixel along each side (output at least 4 wide).
    let f = ((s.width / max_w.max(4)) / 4).max(1) as usize;
    let (ow, oh) = (bw / f, bh / f);
    let mut rgb = vec![[0f32; 3]; ow * oh];
    for by in 0..oh * f {
        for bx in 0..ow * f {
            let at = (by * bw + bx) * 16;
            let px = bc6_block(&d[at..at + 16]);
            let o = &mut rgb[(by / f) * ow + bx / f];
            for c in &px {
                for k in 0..3 {
                    o[k] += c[k];
                }
            }
        }
    }
    let n = (16 * f * f) as f32;
    for c in &mut rgb {
        *c = c.map(|v| v / n);
    }
    Ok(SkyPicture { width: ow as u32, height: oh as u32, rgb, scale })
}

/// A cube's six faces from the sky, `size` px each, as RGBA16F texels
/// (`rgb`, alpha one), +X -X +Y -Y +Z -Z in D3D's face layout over world
/// axes (the renderer's sky looks up with world x, y, z).
pub fn sky_cube(sky: &SkyPicture, size: u32) -> Vec<[f32; 4]> {
    let mut out = Vec::with_capacity(6 * (size * size) as usize);
    for f in 0..6 {
        for y in 0..size {
            for x in 0..size {
                let u = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let w = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let c = sky.at(cube_dir(f, u, w));
                out.push([c[0], c[1], c[2], 1.0]);
            }
        }
    }
    out
}
