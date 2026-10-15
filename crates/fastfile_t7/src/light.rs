//! bo2mp: a Black Ops III map's own light, read from its `GfxWorld`'s
//! first sun volume (`walk::gfxworld::SunVolumeLighting`):
//!
//! * the sky: the sky box model's material (`mc/sky_latlong_hdr`) names a
//!   lat-long picture (BC6H, 8192 x 4096 on the MP maps) whose pixels are
//!   the sun volume's sky `StreamWrappedBuffer` (an `.xpak` entry: the
//!   image record itself has none). Its `$Globals` hold `skyScale`. The sky
//!   pixel shader (`sky_latlong_*hdr`, fxc disassembly) looks it up by the
//!   view direction d with x and y swapped: `u = 0.5 - atan2(d.x, d.y) /
//!   2pi`, `v = acos(d.z) / pi` (row 0 straight up), times `skyScale`;
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
    /// bo2mp: the fog volumes (`GfxWorld` 0x148: 624 bytes each).
    pub fog: Vec<FogVolume>,
}

/// bo2mp: a fog volume: its planes, its priority and its settings banks.
/// Layout read off the maps (mp_redwood, mp_nuketown_x, mp_waterpark,
/// mp_apartments; the field names are ours): words 0 and 1 the first plane
/// and the plane count, word 8 a priority (0.001 for the map-wide one, 0.1
/// or 0.01 for local ones), word 10 a mask of the lighting states it holds;
/// then at 0x40, 0xd0 and 0x160 one bank of settings per lighting state
/// (36 words). The volume holds a point when every plane has it on its
/// back (`n . p <= d`).
#[derive(Clone, Debug)]
pub struct FogVolume {
    pub planes: Vec<[f32; 4]>,
    pub priority: f32,
    pub banks: Vec<[f32; 36]>,
}

/// The size of a fog volume and where its banks are.
pub const FOG_VOLUME: usize = 624;
const FOG_BANKS: [usize; 3] = [0x40, 0xd0, 0x160];

impl FogVolume {
    /// How far outside the volume `p` is (0 inside).
    pub fn outside(&self, p: [f32; 3]) -> f32 {
        self.planes.iter().map(|q| q[0] * p[0] + q[1] * p[1] + q[2] * p[2] - q[3]).fold(0.0, f32::max)
    }

    /// The bank of lighting state `state`, when it reads as atmospheric
    /// fog settings (word 0 is 1, a half distance and the sky's size set).
    pub fn bank(&self, state: usize) -> Option<&[f32; 36]> {
        self.banks.get(state).filter(|b| (b[0] - 1.0).abs() < 1e-3 && b[2] > 0.0 && b[31] > 1000.0)
    }
}

/// bo2mp: a map's fog volumes (none when its sun volume was not read).
pub fn fog(map: &T7Map) -> &[FogVolume] {
    map.light_refs.as_ref().map_or(&[], |r| r.fog.as_slice())
}

fn fog_volumes(g: &crate::walk::gfxworld::GfxWorldRec, image: &[u8]) -> Vec<FogVolume> {
    let (Some((at, n)), planes) = (g.fog_volumes, g.fog_planes) else { return Vec::new() };
    let plane = |i: usize| -> Option<[f32; 4]> {
        let (pa, pn) = planes?;
        (i < pn).then(|| std::array::from_fn(|k| f32_at(image, pa + 16 * i + 4 * k)))
    };
    (0..n)
        .filter_map(|v| {
            let r = at + FOG_VOLUME * v;
            image.get(r..r + FOG_VOLUME)?;
            let u = |o: usize| u32::from_le_bytes(image[r + o..r + o + 4].try_into().unwrap()) as usize;
            let planes = (u(0)..u(0) + u(4).min(32)).filter_map(plane).collect();
            let banks = FOG_BANKS.iter().map(|&b| std::array::from_fn(|k| f32_at(image, r + b + 4 * k))).collect();
            Some(FogVolume { planes, priority: f32_at(image, r + 0x20), banks })
        })
        .collect()
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
        out.fog = fog_volumes(g, image);
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
        let u = 0.5 - x.atan2(y) / std::f32::consts::TAU;
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

    /// The mean sky within `cos_max` of direction `d` (a soft cone).
    pub fn glow(&self, d: [f32; 3], cos_max: f32) -> f32 {
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
        let d = d.map(|v| v / n);
        let (mut sum, mut cnt) = (0f64, 0u32);
        let (w, h) = (64u32, 32u32);
        for y in 0..h {
            for x in 0..w {
                let phi = (0.5 - (x as f32 + 0.5) / w as f32) * std::f32::consts::TAU;
                let theta = (y as f32 + 0.5) / h as f32 * std::f32::consts::PI;
                let dir = [theta.sin() * phi.sin(), theta.sin() * phi.cos(), theta.cos()];
                if dir[0] * d[0] + dir[1] * d[1] + dir[2] * d[2] >= cos_max {
                    let c = self.at(dir);
                    sum += f64::from(0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]);
                    cnt += 1;
                }
            }
        }
        if cnt == 0 { 0.0 } else { (sum / f64::from(cnt)) as f32 }
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
        // (u = 0.5 - atan2(x, y) / 2pi: atan2(x, y) = phi.)
        let phi = (0.5 - u) * std::f32::consts::TAU;
        let theta = v * std::f32::consts::PI;
        ([theta.sin() * phi.sin(), theta.sin() * phi.cos(), theta.cos()], best)
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
    /// Config word 0 (the scale its texels are stored at, it seems).
    pub scale: f32,
    /// +X, -X, +Y, -Y, +Z, -Z (world axes) of the smallest level.
    pub faces: [[f32; 3]; 6],
}

impl Probe {
    /// Distance outside the box (0 inside), in units.
    pub fn outside(&self, p: [f32; 3]) -> f32 {
        let d = [p[0] - self.origin[0], p[1] - self.origin[1], p[2] - self.origin[2]];
        let mut s = 0.0;
        for k in 0..3 {
            let l = self.axes[k][0] * d[0] + self.axes[k][1] * d[1] + self.axes[k][2] * d[2];
            let o = (l.abs() - self.half[k]).max(0.0);
            s += o * o;
        }
        s.sqrt()
    }

    pub fn volume(&self) -> f32 {
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

/// How far outside a probe's box its light still reaches (blending to
/// the map-wide probe's).
const BOX_BLEND: f32 = 96.0;

/// The map's baked light, read once: the sky and the probes.
pub struct Bake {
    pub sky: Option<SkyPicture>,
    pub probes: Vec<Probe>,
    /// The map's bounds (the probes' boxes but the map-wide one).
    pub bounds: [[f32; 3]; 2],
    /// The sky's light (times `skyScale`) over the probes' in the same
    /// directions (the map-wide probe's upper four faces against the sky
    /// there): the probes are baked in exposure-relative units.
    pub probe_units: f32,
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
            // (Mip-major: every face of every probe at level 0, then level
            // 1, ...; read on mp_redwood, whose faces are then world +X..-Z.)
            let lsize = |k: u32| crate::texture::level_size(p.format, p.size >> k, p.size >> k).unwrap_or(0);
            let face: usize = (0..p.levels).map(lsize).sum();
            let last: usize = (0..p.levels - 1).map(|k| lsize(k) * 6 * p.count as usize).sum();
            let lsz = lsize(p.levels - 1);
            match paks.read(p.key) {
                Ok(d) if d.len() >= face * 6 * p.count as usize => {
                    for i in 0..p.count as usize {
                        let c = &p.configs[i * PROBE_CONFIG..(i + 1) * PROBE_CONFIG];
                        let w = |k: usize| f32_at(c, 4 * k);
                        let mut faces = [[0f32; 3]; 6];
                        for (f, out) in faces.iter_mut().enumerate() {
                            // The smallest level: 4x4 or less, one block.
                            let at = last + (6 * i + f) * lsz;
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
                            scale: w(0),
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
        // The sky against the map-wide probe: its +Z face's mean against the
        // sky's mean over the same face.
        let mut probe_units = 1.0;
        if let (Some(sky), Some(g)) = (sky.as_ref(), probes.first()) {
            let s = 16u32;
            let cube = sky_cube(sky, s);
            let lum = |c: [f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            let face = 4usize;
            let sky_up: f32 = cube[face * (s * s) as usize..(face + 1) * (s * s) as usize].iter().map(|c| lum([c[0], c[1], c[2]])).sum::<f32>() / (s * s) as f32;
            let probe_up = lum(g.faces[face]);
            if sky_up > 0.0 && probe_up > 1e-4 {
                probe_units = sky_up / probe_up;
            }
            report.push(format!("sky over probe (up face): {sky_up:.2} / {probe_up:.3} = {probe_units:.1}; probe 0 word 0 {:.1}; ratio over it {:.3}", g.scale, probe_units / g.scale.max(1e-3)));
            // Which way the sun volume's angles point: the sky round both
            // readings of them.
            if let Some(s) = map.sun {
                let (p, y) = (s.pitch.to_radians(), s.yaw.to_radians());
                let fwd = [p.cos() * y.cos(), p.cos() * y.sin(), -p.sin()];
                let mut line = format!("sun angles {:.1} {:.1} colour {:.3?} intensity {:.1}:", s.pitch, s.yaw, s.color, s.intensity);
                for (name, d) in [
                    ("fwd", fwd),
                    ("-fwd", fwd.map(|v| -v)),
                    ("fwd z-", [fwd[0], fwd[1], -fwd[2]]),
                    ("-fwd z-", [-fwd[0], -fwd[1], fwd[2]]),
                    ("yx", [fwd[1], fwd[0], fwd[2]]),
                    ("-yx", [-fwd[1], -fwd[0], -fwd[2]]),
                ] {
                    line.push_str(&format!(" {name} {d:.2?} {:.0};", sky.glow(d, 0.995)));
                }
                report.push(line);
            }
            // The brightest sky above the horizon (a 5 degree cone's mean).
            let mut best = (0f32, [0f32; 3]);
            for yy in 0..24 {
                for xx in 0..96 {
                    let phi = xx as f32 / 96.0 * std::f32::consts::TAU;
                    let theta = (yy as f32 + 0.5) / 24.0 * std::f32::consts::FRAC_PI_2;
                    let d = [theta.sin() * phi.sin(), theta.sin() * phi.cos(), theta.cos()];
                    let g = sky.glow(d, 0.996);
                    if g > best.0 {
                        best = (g, d);
                    }
                }
            }
            report.push(format!("sky's brightest above the horizon: {:.0} toward {:.3?}", best.0, best.1));
        }
        if let Some(g) = probes.first() {
            report.push(format!(
                "probes: {} (global probe faces {:.2?}), local boxes over {lo:.0?}..{hi:.0?}",
                probes.len(),
                g.faces.iter().map(|f| 0.2126 * f[0] + 0.7152 * f[1] + 0.0722 * f[2]).collect::<Vec<_>>()
            ));
        }
        Ok(Bake { sky, probes, bounds: [lo, hi], probe_units, report })
    }

    /// The ambient light at `p` for a surface facing `n` (unit): the probe
    /// whose box holds `p` (the smallest such), blended toward the nearest
    /// box's within `BLEND` units of it, else the map-wide probe.
    pub fn ambient(&self, p: [f32; 3], n: [f32; 3]) -> [f32; 3] {
        if self.probes.is_empty() {
            return [1.0; 3];
        }
        let best = self.box_at(p);
        let global = self.probes[0].ambient(n);
        match best {
            Some((o, i)) => {
                let local = self.probes[i].ambient(n);
                let t = (o / BOX_BLEND).clamp(0.0, 1.0);
                std::array::from_fn(|k| local[k] + (global[k] - local[k]) * t)
            }
            None => global,
        }
    }

    /// The probes' unit: the map-wide probe's scale (config word 0). A
    /// probe's texels times its own scale are the game's light; over this
    /// unit, daylit ground is about one.
    pub fn units(&self) -> f32 {
        self.probes.first().map(|p| p.scale).filter(|s| s.is_finite() && *s > 1e-3).unwrap_or(1.0)
    }

    /// A probe's light over the map-wide probe's (its scale over `units`).
    pub fn gain(&self, i: usize) -> f32 {
        self.probes.get(i).map_or(1.0, |p| if p.scale.is_finite() && p.scale > 0.0 { p.scale / self.units() } else { 1.0 })
    }

    /// The ambient light at `p` facing `n` in the probes' unit: as
    /// `ambient`, each probe's faces times its `gain` (a probe in a dim
    /// room is baked dim).
    pub fn ambient_lit(&self, p: [f32; 3], n: [f32; 3]) -> [f32; 3] {
        if self.probes.is_empty() {
            return [1.0; 3];
        }
        let global = self.probes[0].ambient(n);
        match self.box_at(p) {
            Some((o, i)) => {
                let g = self.gain(i);
                let local = self.probes[i].ambient(n).map(|v| v * g);
                let t = (o / BOX_BLEND).clamp(0.0, 1.0);
                std::array::from_fn(|k| local[k] + (global[k] - local[k]) * t)
            }
            None => global,
        }
    }

    /// `ambient_lit` averaged over the six axis directions.
    pub fn ambient_mean(&self, p: [f32; 3]) -> [f32; 3] {
        let mut s = [0f32; 3];
        for d in [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0f32]] {
            let a = self.ambient_lit(p, d);
            for k in 0..3 {
                s[k] += a[k] / 6.0;
            }
        }
        s
    }

    /// What the sky picture (times `skyScale`) is multiplied by to be in the
    /// probes' unit: one over `units`, unless the sky's upper face and the
    /// map-wide probe's (times `units`) disagree by more than four times
    /// (mp_waterpark's picture is 20 times dimmer than its probes saw),
    /// then matched to the probe.
    pub fn sky_gain(&self) -> f32 {
        let u = self.units();
        let (Some(sky), Some(g)) = (self.sky.as_ref(), self.probes.first()) else {
            return 1.0 / u;
        };
        let s = 8u32;
        let cube = sky_cube(sky, s);
        let lum = |c: [f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let n = (s * s) as usize;
        let sky_up: f32 = cube[4 * n..5 * n].iter().map(|c| lum([c[0], c[1], c[2]])).sum::<f32>() / n as f32;
        let probe_up = lum(g.faces[4]) * u;
        let r = probe_up / sky_up.max(1e-6);
        if (0.25..=4.0).contains(&r) { 1.0 / u } else { r / u }
    }

    /// The local probe whose box holds `p` (the smallest such), else the
    /// nearest within `BOX_BLEND` units: (distance outside it, index).
    pub fn box_at(&self, p: [f32; 3]) -> Option<(f32, usize)> {
        let mut best: Option<(f32, f32, usize)> = None;
        for (i, pr) in self.probes.iter().enumerate().skip(1) {
            let o = pr.outside(p);
            if o > BOX_BLEND {
                continue;
            }
            // Inside boxes first (smallest volume), then the nearest.
            let key = (o, pr.volume());
            if best.is_none_or(|(bo, bv, _)| (key.0, key.1) < (bo, bv)) {
                best = Some((key.0, key.1, i));
            }
        }
        best.map(|(o, _, i)| (o, i))
    }

    /// The probe a point reflects: its box's, else the map-wide one (0).
    pub fn probe_at(&self, p: [f32; 3]) -> usize {
        self.box_at(p).map_or(0, |(_, i)| i)
    }

    /// A short description of the probes (`t7_ambient`).
    pub fn describe(&self) -> String {
        let mut s = self.report.join("\n");
        for (i, p) in self.probes.iter().enumerate().take(24) {
            s.push_str(&format!(
                "\nprobe {i}: origin {:.0?} half {:.0?} scale {:.1} mean {:.2?} up {:.2?} down {:.2?}",
                p.origin, p.half, p.scale, p.mean, p.faces[4], p.faces[5]
            ));
        }
        s
    }
}

/// One probe's six faces at level `mip`, decoded.
pub struct ProbeFaces {
    pub size: u32,
    pub texels: Vec<Vec<[f32; 3]>>,
}

/// Probe `index`'s six faces at level `mip` (as stored: face-major, each
/// face its whole chain).
pub fn probe_faces(map: &T7Map, index: usize, mip: u32) -> Result<ProbeFaces, String> {
    let p = map.light_refs.as_ref().and_then(|r| r.probes.as_ref()).ok_or("no probes")?;
    let paks = map.paks().ok_or("no packages")?;
    let d = paks.read(p.key).map_err(|e| e.to_string())?;
    let lsize = |k: u32| crate::texture::level_size(p.format, p.size >> k, p.size >> k).unwrap_or(0);
    let face: usize = (0..p.levels).map(lsize).sum();
    let mip = mip.min(p.levels - 1);
    let before: usize = (0..mip).map(lsize).sum();
    let s = (p.size >> mip).max(1);
    let bw = s.div_ceil(4) as usize;
    let mut texels = Vec::new();
    let level_major = !std::env::var("T7_CUBE_ORDER").is_ok_and(|v| v == "face");
    for f in 0..6 {
        let at = if level_major {
            (0..mip).map(|k| lsize(k) * 6 * p.count as usize).sum::<usize>() + (6 * index + f) * lsize(mip)
        } else {
            (6 * index + f) * face + before
        };
        let mut t = vec![[0f32; 3]; (s * s) as usize];
        for (i, b) in d.get(at..at + lsize(mip)).ok_or("past the data")?.chunks_exact(16).enumerate() {
            let px = bc6_block(b);
            for y in 0..4 {
                for x in 0..4 {
                    let (tx, ty) = (i % bw * 4 + x, i / bw * 4 + y);
                    if tx < s as usize && ty < s as usize {
                        t[ty * s as usize + tx] = px[y * 4 + x];
                    }
                }
            }
        }
        texels.push(t);
    }
    Ok(ProbeFaces { size: s, texels })
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

/// The reflection probes `keep` as one cube array in layer-major order
/// (each probe's six faces, each face its whole chain), as uploaded: BC6H
/// (`format`), `size` px, `levels` levels.
pub struct ProbeArray {
    pub size: u32,
    pub levels: u32,
    pub format: u32,
    pub data: Vec<u8>,
}

pub fn probe_array(map: &T7Map, keep: &[usize]) -> Result<ProbeArray, String> {
    let p = map.light_refs.as_ref().and_then(|r| r.probes.as_ref()).ok_or("no probes")?;
    let paks = map.paks().ok_or("no packages")?;
    let d = paks.read(p.key).map_err(|e| e.to_string())?;
    let lsize = |k: u32| crate::texture::level_size(p.format, p.size >> k, p.size >> k).unwrap_or(0);
    let count = p.count as usize;
    let mut level_at = Vec::with_capacity(p.levels as usize);
    let mut at = 0usize;
    for k in 0..p.levels {
        level_at.push(at);
        at += lsize(k) * 6 * count;
    }
    if d.len() < at {
        return Err(format!("probes: {} bytes, {at} wanted", d.len()));
    }
    let mut data = Vec::with_capacity(keep.len() * 6 * (0..p.levels).map(lsize).sum::<usize>());
    for &i in keep {
        if i >= count {
            return Err(format!("probe {i} of {count}"));
        }
        for f in 0..6 {
            for k in 0..p.levels {
                let s = lsize(k);
                let o = level_at[k as usize] + (6 * i + f) * s;
                data.extend_from_slice(&d[o..o + s]);
            }
        }
    }
    Ok(ProbeArray { size: p.size, levels: p.levels, format: p.format, data })
}

/// The direction toward the sun from the sun volume's angles: they aim the
/// light (pitch down from level, yaw about z), so the sun lies the other
/// way. (Read against mp_redwood's, mp_nuketown_x's and mp_apartments' own
/// probes, whose brightest texel sits within a few degrees of it.)
pub fn sun_toward(s: &crate::map::Sun) -> [f32; 3] {
    let (p, y) = (s.pitch.to_radians(), s.yaw.to_radians());
    let d = [-p.cos() * y.cos(), -p.cos() * y.sin(), p.sin()];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    d.map(|v| v / l)
}

/// How a material's triangles block the sun and the sky.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blocks {
    No,
    /// Cut-out (foliage, fences): half the light through.
    Partly,
    Fully,
}

/// The map's triangles that block light, world surfaces then placed
/// models' first LODs (in the world), by `blocks(material)`; placed models
/// smaller than `min_radius` units (scaled) and outside `area` (low and
/// high corners, xy) are left out.
pub fn casters(map: &T7Map, blocks: &dyn Fn(usize) -> Blocks, min_radius: f32, area: [[f32; 2]; 2]) -> Vec<crate::bake::Caster> {
    use crate::bake::Caster;
    let mut out = Vec::new();
    for s in &map.surfaces {
        let b = s.material.map_or(Blocks::No, blocks);
        if b == Blocks::No {
            continue;
        }
        let base = s.base_index as usize;
        let first = s.first_vertex as usize;
        for t in 0..s.tri_count as usize {
            let Some(ix) = map.indices.get(base + 3 * t..base + 3 * t + 3) else { break };
            let v = |k: usize| map.vertices.get(first + usize::from(ix[k])).map(|v| v.xyz);
            if let (Some(a), Some(b2), Some(c)) = (v(0), v(1), v(2)) {
                out.push(Caster { p: [a, b2, c], partial: b == Blocks::Partly });
            }
        }
    }
    for sm in &map.static_models {
        let Some(m) = map.models.get(sm.model) else { continue };
        let r = m.surfaces.iter().flat_map(|s| s.positions.iter()).map(|q| q[0].abs().max(q[1].abs()).max(q[2].abs())).fold(0.0f32, f32::max) * sm.scale;
        if r < min_radius
            || sm.origin[0] + r < area[0][0]
            || sm.origin[0] - r > area[1][0]
            || sm.origin[1] + r < area[0][1]
            || sm.origin[1] - r > area[1][1]
        {
            continue;
        }
        let place = |q: [f32; 3]| -> [f32; 3] {
            std::array::from_fn(|k| sm.origin[k] + sm.scale * (q[0] * sm.axis[0][k] + q[1] * sm.axis[1][k] + q[2] * sm.axis[2][k]))
        };
        for surf in &m.surfaces {
            let b = surf.material.map_or(Blocks::No, blocks);
            if b == Blocks::No {
                continue;
            }
            for t in &surf.tris {
                let v = |k: usize| surf.positions.get(usize::from(t[k])).copied().map(place);
                if let (Some(a), Some(b2), Some(c)) = (v(0), v(1), v(2)) {
                    out.push(Caster { p: [a, b2, c], partial: b == Blocks::Partly });
                }
            }
        }
    }
    out
}

/// The baked sun shadow map for the GPU: `SunDepth` over the square round
/// `centre` (`span` units, at most `max_size` texels a side) and its
/// height packing (`SunDepth::packed`): base and range.
pub struct SunMap {
    pub depth: crate::bake::SunDepth,
    pub base: f32,
    pub range: f32,
}

impl SunMap {
    pub fn render(casters: &[crate::bake::Caster], toward: [f32; 3], centre: [f32; 3], span: f32, max_size: u32) -> SunMap {
        // (Never 1024 wide: the renderer tells a Black Ops II colour table,
        // 1024 wide, from this map by its width.)
        let size = ((span / 1.5).ceil() as u32).clamp(2048, max_size.max(2048));
        let depth = crate::bake::SunDepth::render(casters, toward, centre, span, size);
        let (lo, hi) = depth.height_range();
        let (base, range) = if lo <= hi { (lo - 16.0, (hi - lo + 32.0).max(1.0)) } else { (0.0, 1.0) };
        SunMap { depth, base, range }
    }

    /// The map's RGBA8 texels (`SunDepth::packed`).
    pub fn texels(&self) -> Vec<u8> {
        self.depth.packed(self.base, self.range)
    }

    /// The shader's three rows: (u, origin u), (v, origin v), (toward the
    /// sun, 1 / texel), (base, range, size, 0).
    pub fn rows(&self) -> [[f32; 4]; 4] {
        let d = &self.depth;
        [
            [d.u[0], d.u[1], d.u[2], d.origin[0]],
            [d.v[0], d.v[1], d.v[2], d.origin[1]],
            [d.dir[0], d.dir[1], d.dir[2], 1.0 / d.texel],
            [self.base, self.range, d.size as f32, 0.0],
        ]
    }
}
