//! A map's baked light in the form the CoD4 world draw reads.
//!
//! An IW7 `GfxLightMap` is Ghosts' lightmap carried on: a primary (BC4,
//! the sun's baked visibility, one page in size), a secondary (BC6H, two
//! pages stacked, linear light: the upper the base light, the lower the
//! directional light) and a second-unorm image (R8G8 at a page's size: the
//! dominant light's tangent-space x and y, 0..1 for -1..1, which Ghosts
//! kept in the secondary's alpha). Read on mp_flip: one 4096 x 2048 page;
//! its surfaces' `lightmapIndex` is 0 or 31 (31: none), their vertices'
//! lightmap coordinates span 0..1 of the page.
//!
//! `LightmapPage` is that page as Call of Duty 4's (as
//! `fastfile_iw6::map::iw3_lightmap` makes Ghosts'): the primary as 8-bit
//! luminance, the secondary as `A8R8G8B8` bytes (B, G, R, A): the light as
//! gamma bytes (`sqrt`, clamped at 1), the direction in CoD4's alpha code
//! (`a * 4.08 - 2.08`, `a * 4.064516 - 2.064516`).

use crate::map::Iw7Map;

/// One lightmap page in Call of Duty 4's form.
#[derive(Clone, Debug)]
pub struct LightmapPage {
    /// Width, height, L8 texels.
    pub primary: (u16, u16, Vec<u8>),
    /// Width, height (two pages), B, G, R, A bytes.
    pub secondary: (u16, u16, Vec<u8>),
    /// The scale the light was given (linear) and the geometric mean of the
    /// lit texels' base plus directional luminance before it.
    pub exposure: f32,
    pub mean_light: f32,
    /// The exposure the pages alone would take (`TARGET_LIGHT`).
    pub auto_exposure: f32,
}

/// The light a lit texel of a page is brought to on average (linear, base
/// plus directional): IW7's pages differ by map (its renderer exposes each
/// map by its vision's `tonemapExposure`, 9.5 to 15); here each page is
/// scaled so its lit texels' geometric mean is this, within
/// `EXPOSURE_RANGE`.
pub const TARGET_LIGHT: f32 = 0.22;
pub const EXPOSURE_RANGE: (f32, f32) = (0.03, 4.0);

/// The scale that turns a vision's exposure (`tonemapExposure` plus
/// `tonemapExposureAdjust`, in stops) into the pages' linear scale:
/// `VISION_EXPOSURE_K * 2^-stops`, fitted on mp_riot. For reports only (see
/// `Iw7Map::lightmap_page`: the pages' own exposure is used).
pub const VISION_EXPOSURE_K: f32 = 965.0;

/// A vision's exposure as the pages' linear scale (`VISION_EXPOSURE_K`),
/// when it sets `tonemapExposure`.
pub fn vision_exposure(vision: &str) -> Option<f32> {
    let get = |k: &str| {
        vision.lines().find_map(|l| {
            let (key, v) = l.trim().split_once(' ')?;
            key.eq_ignore_ascii_case(k).then(|| v.trim().trim_matches('"').split_whitespace().next()?.parse::<f32>().ok()).flatten()
        })
    };
    let stops = get("tonemapExposure")? + get("tonemapExposureAdjust").unwrap_or(0.0);
    (stops.is_finite() && (4.0..=20.0).contains(&stops)).then(|| VISION_EXPOSURE_K * 2f32.powf(-stops))
}

/// What a point on the world takes from its lightmap: the base and
/// directional light (CoD4 gamma, 0..1 each) and the sun's visibility.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightSample {
    pub base: [f32; 3],
    pub directional: [f32; 3],
    pub sun: f32,
}

impl LightSample {
    /// The light as the world shader adds it up for a surface facing the
    /// dominant light, without the sun: `(A + B)^2`, linear.
    pub fn ambient_linear(&self) -> [f32; 3] {
        std::array::from_fn(|k| {
            let g = self.base[k] + self.directional[k];
            g * g
        })
    }
}

impl LightmapPage {
    /// The page at `(u, v)` (nearest texel, clamped).
    pub fn sample(&self, uv: [f32; 2]) -> LightSample {
        let (sw, sh, s) = (&self.secondary.0, &self.secondary.1, &self.secondary.2);
        let (sw, page_h) = (usize::from(*sw), usize::from(*sh) / 2);
        let px = |w: usize, h: usize| {
            let x = ((uv[0].clamp(0.0, 1.0) * w as f32) as usize).min(w.saturating_sub(1));
            let y = ((uv[1].clamp(0.0, 1.0) * h as f32) as usize).min(h.saturating_sub(1));
            (x, y)
        };
        let (x, y) = px(sw, page_h);
        let at = |page: usize| {
            let o = ((page * page_h + y) * sw + x) * 4;
            [s[o + 2], s[o + 1], s[o]].map(|b| f32::from(b) / 255.0)
        };
        let (pw, ph) = (usize::from(self.primary.0), usize::from(self.primary.1));
        let (qx, qy) = px(pw, ph);
        LightSample { base: at(0), directional: at(1), sun: f32::from(self.primary.2[qy * pw + qx]) / 255.0 }
    }
}

/// The sun's light in CoD4's gamma for an HDR sun of brightest channel
/// `peak`, in the units of a page exposed by `exposure`: `sqrt(peak *
/// exposure)` (Ghosts' `sqrt(peak) / 2` is an exposure of a quarter),
/// within 0.5..1.15 (brighter burnt every sunlit surface white).
pub fn sun_gamma(peak: f32, exposure: f32) -> f32 {
    (peak.max(0.0) * exposure).sqrt().clamp(0.5, 1.15)
}

/// An IEEE half float.
fn half(h: u16) -> f32 {
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = (h >> 10) & 0x1f;
    let frac = f32::from(h & 0x3ff);
    sign * match exp {
        0 => frac * 2f32.powi(-24),
        31 => f32::INFINITY,
        e => (1.0 + frac / 1024.0) * 2f32.powi(i32::from(e) - 15),
    }
}

impl Iw7Map {
    /// Light map `index` as Call of Duty 4's page, its pages at most
    /// `max_width` wide (whole halvings).
    pub fn lightmap_page(&self, index: usize, max_width: u32) -> Result<LightmapPage, String> {
        let names = self.light_maps.get(index).ok_or_else(|| format!("no light map {index}"))?;
        let sec = self.image(&names[1], 1 << 14).map_err(|e| format!("secondary: {e}"))?;
        let (w, h) = (sec.width as usize, sec.height as usize);
        if w < 4 || h < 8 || w % 4 != 0 || h % 8 != 0 {
            return Err(format!("secondary {w}x{h}"));
        }
        let page_h = h / 2;
        let mut f = 1usize;
        while w / f > max_width.max(4) as usize && (w / f) % 8 == 0 && (page_h / f) % 8 == 0 {
            f *= 2;
        }
        // The secondary in linear light, box-filtered down by `f` as it is
        // decoded.
        let ow = w / f;
        let mut small = vec![[0f32; 3]; ow * (h / f)];
        let weight = 1.0 / (f * f) as f32;
        let mut add = |x: usize, y: usize, v: [f32; 3]| {
            let o = &mut small[(y / f) * ow + x / f];
            for c in 0..3 {
                if v[c].is_finite() {
                    o[c] += v[c].max(0.0) * weight;
                }
            }
        };
        match sec.format {
            94..=96 => {
                let bw = w / 4;
                let need = bw * (h / 4) * 16;
                if sec.data.len() < need {
                    return Err(format!("secondary has {} of {need} bytes", sec.data.len()));
                }
                let mut tmp = [0f32; 48];
                for (i, b) in sec.data[..need].chunks_exact(16).enumerate() {
                    bcdec_rs::bc6h_float(b, &mut tmp, 12, sec.format == 96);
                    let (bx, by) = (i % bw * 4, i / bw * 4);
                    for y in 0..4 {
                        for x in 0..4 {
                            let t = (y * 4 + x) * 3;
                            add(bx + x, by + y, [tmp[t], tmp[t + 1], tmp[t + 2]]);
                        }
                    }
                }
            }
            10 => {
                if sec.data.len() < w * h * 8 {
                    return Err("secondary is short".to_owned());
                }
                for (i, p) in sec.data.chunks_exact(8).take(w * h).enumerate() {
                    add(i % w, i / w, std::array::from_fn(|c| half(u16::from_le_bytes([p[2 * c], p[2 * c + 1]]))));
                }
            }
            // R9G9B9E5_SHAREDEXP (mp_geneva, mp_renaissance2, mp_hawkwar).
            67 => {
                if sec.data.len() < w * h * 4 {
                    return Err("secondary is short".to_owned());
                }
                for (i, p) in sec.data.chunks_exact(4).take(w * h).enumerate() {
                    let v = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
                    let scale = 2f32.powi((v >> 27) as i32 - 15 - 9);
                    add(i % w, i / w, [v & 0x1ff, (v >> 9) & 0x1ff, (v >> 18) & 0x1ff].map(|m| m as f32 * scale));
                }
            }
            // R11G11B10_FLOAT.
            26 => {
                if sec.data.len() < w * h * 4 {
                    return Err("secondary is short".to_owned());
                }
                let small = |bits: u32, mant: u32| -> f32 {
                    let (e, m) = (bits >> mant, bits & ((1 << mant) - 1));
                    match e {
                        0 => m as f32 / (1 << mant) as f32 * 2f32.powi(-14),
                        31 => 0.0,
                        _ => (1.0 + m as f32 / (1 << mant) as f32) * 2f32.powi(e as i32 - 15),
                    }
                };
                for (i, p) in sec.data.chunks_exact(4).take(w * h).enumerate() {
                    let v = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
                    add(i % w, i / w, [small(v & 0x7ff, 6), small((v >> 11) & 0x7ff, 6), small(v >> 22, 5)]);
                }
            }
            f => return Err(format!("secondary format {f} (BC6H, R9G9B9E5 or R11G11B10 expected)")),
        }
        drop(sec);
        let (ow, oh) = (w / f, h / f);
        let opage = oh / 2;
        // The directions (a page in size).
        let dirs: Option<Vec<[f32; 2]>> = self.image(&names[2], 1 << 14).ok().and_then(|d| {
            let (dw, dh) = (d.width as usize, d.height as usize);
            (d.format == 49 && dw > 0 && dh > 0 && d.data.len() >= dw * dh * 2).then(|| {
                let (fx, fy) = ((dw / ow).max(1), (dh / opage).max(1));
                let mut out = vec![[0.5f32; 2]; ow * opage];
                for (i, o) in out.iter_mut().enumerate() {
                    let (x, y) = (i % ow, i / ow);
                    let mut acc = [0f32; 2];
                    let mut n = 0f32;
                    for dy in 0..fy {
                        for dx in 0..fx {
                            let (sx, sy) = (x * fx + dx, y * fy + dy);
                            if sx < dw && sy < dh {
                                let at = (sy * dw + sx) * 2;
                                acc[0] += f32::from(d.data[at]) / 255.0;
                                acc[1] += f32::from(d.data[at + 1]) / 255.0;
                                n += 1.0;
                            }
                        }
                    }
                    if n > 0.0 {
                        *o = acc.map(|a| a / n);
                    }
                }
                out
            })
        });
        // The exposure: the lit texels' geometric mean (base page plus the
        // directional page under it) brought to `TARGET_LIGHT`.
        let (mut log_sum, mut lit) = (0f64, 0usize);
        for y in 0..opage {
            for x in 0..ow {
                let (a, b) = (small[y * ow + x], small[(y + opage) * ow + x]);
                let l = 0.2126 * (a[0] + b[0]) + 0.7152 * (a[1] + b[1]) + 0.0722 * (a[2] + b[2]);
                if l > 1e-3 && l.is_finite() {
                    log_sum += f64::from(l.ln());
                    lit += 1;
                }
            }
        }
        let mean_light = if lit > 0 { (log_sum / lit as f64).exp() as f32 } else { TARGET_LIGHT };
        let auto = (TARGET_LIGHT / mean_light).clamp(EXPOSURE_RANGE.0, EXPOSURE_RANGE.1);
        // (The vision's own exposure, `vision_exposure`, was tried and left:
        // against the pages it runs from 3.8 stops darker to half a stop
        // brighter map by map, mp_riot and mp_proto about even, mp_mansion
        // and mp_pixel nearly four stops under: IW7 adapts most maps' exposure
        // at run time, `tonemapAuto`.)
        let exposure = auto;
        // Highlights rolled off above `KNEE` toward 1 (as IW7's filmic tone
        // map's shoulder does) rather than clipped: mp_proto's snow and
        // mp_fallen's sunlit concrete kept no detail clipped.
        const KNEE: f32 = 0.6;
        let shoulder = |v: f32| if v <= KNEE { v } else { KNEE + (1.0 - KNEE) * (1.0 - (-(v - KNEE) / (1.0 - KNEE)).exp()) };
        let gamma = |v: f32| (shoulder((v * exposure).max(0.0)).sqrt() * 255.0).round() as u8;
        let mut secondary = vec![0u8; ow * oh * 4];
        for y in 0..oh {
            for x in 0..ow {
                let (page, py) = (y / opage, y % opage);
                let d = dirs.as_ref().map_or([0.5, 0.5], |d| d[py * ow + x]);
                let a = if page == 0 {
                    ((d[0] * 2.0 - 1.0 + 2.08) / 4.08 * 255.0).round()
                } else {
                    ((d[1] * 2.0 - 1.0 + 2.064516) / 4.064516 * 255.0).round()
                };
                let l = small[y * ow + x];
                let o = (y * ow + x) * 4;
                secondary[o..o + 4].copy_from_slice(&[gamma(l[2]), gamma(l[1]), gamma(l[0]), a.clamp(0.0, 255.0) as u8]);
            }
        }
        // The primary, down to the page's size.
        let primary = match self.image(&names[0], 1 << 14) {
            Ok(p) if matches!(p.format, 79..=81) && p.width % 4 == 0 && p.height % 4 == 0 => {
                let (pw, ph) = (p.width as usize, p.height as usize);
                let blocks = (pw / 4) * (ph / 4);
                if p.data.len() < blocks * 8 {
                    return Err("primary is short".to_owned());
                }
                let mut full = vec![0u8; pw * ph];
                for by in 0..ph / 4 {
                    for bx in 0..pw / 4 {
                        let b = (by * (pw / 4) + bx) * 8;
                        bcdec_rs::bc4(&p.data[b..b + 8], &mut full[(by * 4) * pw + bx * 4..], pw, p.format == 81);
                    }
                }
                let (fx, fy) = ((pw / ow).max(1), (ph / opage).max(1));
                let mut out = vec![0u8; ow * opage];
                for (i, o) in out.iter_mut().enumerate() {
                    let (x, y) = (i % ow, i / ow);
                    let mut sum = 0u32;
                    let mut n = 0u32;
                    for dy in 0..fy {
                        for dx in 0..fx {
                            let (sx, sy) = (x * fx + dx, y * fy + dy);
                            if sx < pw && sy < ph {
                                sum += u32::from(full[sy * pw + sx]);
                                n += 1;
                            }
                        }
                    }
                    *o = (sum / n.max(1)) as u8;
                }
                out
            }
            // No sun visibility: lit everywhere.
            _ => vec![255u8; ow * opage],
        };
        let to16 = |v: usize| u16::try_from(v).map_err(|_| format!("{v} is too large"));
        Ok(LightmapPage { primary: (to16(ow)?, to16(opage)?, primary), secondary: (to16(ow)?, to16(oh)?, secondary), exposure, mean_light, auto_exposure: auto })
    }

    /// The light under each point: the highest lightmapped, upward-facing
    /// world triangle below it (within `reach` units), its lightmap
    /// sampled where the point stands. `None` where nothing is below.
    pub fn ground_light(&self, pages: &[Option<LightmapPage>], points: &[[f32; 3]], reach: f32) -> Vec<Option<LightSample>> {
        let floors = FloorIndex::new(self, pages);
        points
            .iter()
            .map(|p| {
                floors
                    .floors(self, pages, p[0], p[1])
                    .into_iter()
                    .filter(|(z, _)| *z <= p[2] + 4.0 && p[2] - *z <= reach)
                    .max_by(|a, b| a.0.total_cmp(&b.0))
                    .map(|f| f.1)
            })
            .collect()
    }

    /// The light each placed model takes: the floor's light under it and
    /// around it (eight points on a ring of about its radius), the middle
    /// one by brightness. The floor right under a model is in its own baked
    /// shadow (mp_proto's crates, mp_hawkwar's station wagon drew black by
    /// it); one ring point out in the sun would light a model in the shade.
    /// `None` where no floor is found.
    pub fn static_model_light(&self, pages: &[Option<LightmapPage>]) -> Vec<Option<LightSample>> {
        let floors = FloorIndex::new(self, pages);
        self.static_models
            .iter()
            .map(|sm| {
                let m = &self.models[sm.model];
                let c = sm.lighting_origin;
                let r = (m.radius * sm.scale * 0.75).clamp(8.0, 192.0);
                let mut found: Vec<LightSample> = Vec::with_capacity(9);
                for k in 0..9 {
                    let (x, y) = if k == 0 {
                        (c[0], c[1])
                    } else {
                        let a = (k - 1) as f32 * std::f32::consts::FRAC_PI_4;
                        (c[0] + r * a.cos(), c[1] + r * a.sin())
                    };
                    let below = floors
                        .floors(self, pages, x, y)
                        .into_iter()
                        .filter(|(z, _)| *z <= c[2] + 4.0 && c[2] - *z <= 1024.0)
                        .max_by(|a, b| a.0.total_cmp(&b.0));
                    if let Some((_, l)) = below {
                        found.push(l);
                    }
                }
                let lum = |l: &LightSample| {
                    let a = l.ambient_linear();
                    0.2126 * a[0] + 0.7152 * a[1] + 0.0722 * a[2]
                };
                found.sort_by(|a, b| lum(a).total_cmp(&lum(b)));
                found.get(found.len() / 2).copied()
            })
            .collect()
    }

    /// Baked light through the space players move in, for lighting them
    /// (`LightVolume`): at points `spacing` apart over `mins..maxs`, the
    /// light of the floor under each point (`ground_light`, within `reach`
    /// units), or `None` where there is none.
    pub fn light_volume(&self, pages: &[Option<LightmapPage>], mins: [f32; 3], maxs: [f32; 3], spacing: [f32; 3], reach: f32) -> LightVolume {
        let floors = FloorIndex::new(self, pages);
        let dims: [usize; 3] = std::array::from_fn(|k| (((maxs[k] - mins[k]) / spacing[k]).floor().max(0.0) as usize + 1).min(4096));
        let mut samples = Vec::with_capacity(dims[0] * dims[1] * dims[2]);
        for ix in 0..dims[0] {
            for iy in 0..dims[1] {
                let (x, y) = (mins[0] + ix as f32 * spacing[0], mins[1] + iy as f32 * spacing[1]);
                let mut col = floors.floors(self, pages, x, y);
                col.sort_by(|a, b| a.0.total_cmp(&b.0));
                for iz in 0..dims[2] {
                    let z = mins[2] + iz as f32 * spacing[2];
                    let below = col.iter().rev().find(|(fz, _)| *fz <= z + 4.0 && z - *fz <= reach);
                    samples.push(below.map(|f| f.1));
                }
            }
        }
        LightVolume { mins, spacing, dims, samples }
    }
}

/// Baked light at points of a box (`Iw7Map::light_volume`): x slowest, z
/// fastest.
#[derive(Clone, Debug, Default)]
pub struct LightVolume {
    pub mins: [f32; 3],
    pub spacing: [f32; 3],
    pub dims: [usize; 3],
    pub samples: Vec<Option<LightSample>>,
}

impl LightVolume {
    pub fn at(&self, ix: usize, iy: usize, iz: usize) -> Option<LightSample> {
        self.samples.get((ix * self.dims[1] + iy) * self.dims[2] + iz).copied().flatten()
    }
}

/// The world's lightmapped, upward-facing triangles binned by their XY
/// bounds (256-unit cells), to find the floors under points.
struct FloorIndex {
    x0: f32,
    y0: f32,
    cols: usize,
    rows: usize,
    /// Per cell: (surface, first index).
    bins: Vec<Vec<(u32, u32)>>,
}

impl FloorIndex {
    const CELL: f32 = 256.0;

    fn cell(&self, x: f32, y: f32) -> Option<usize> {
        let (cx, cy) = (((x - self.x0) / Self::CELL).floor(), ((y - self.y0) / Self::CELL).floor());
        (cx >= 0.0 && cy >= 0.0 && (cx as usize) < self.cols && (cy as usize) < self.rows).then(|| cy as usize * self.cols + cx as usize)
    }

    fn vert<'a>(map: &'a Iw7Map, s: &crate::map::WorldSurface, k: u32) -> Option<&'a crate::map::WorldVertex> {
        let ix = *map.indices.get(s.base_index as usize + k as usize)?;
        map.vertices.get((s.first_vertex + u32::from(ix)) as usize)
    }

    fn new(map: &Iw7Map, pages: &[Option<LightmapPage>]) -> Self {
        let (x0, y0) = (map.mins[0], map.mins[1]);
        let cols = (((map.maxs[0] - x0) / Self::CELL).ceil() as usize).clamp(1, 4096);
        let rows = (((map.maxs[1] - y0) / Self::CELL).ceil() as usize).clamp(1, 4096);
        let mut out = FloorIndex { x0, y0, cols, rows, bins: vec![Vec::new(); cols * rows] };
        for (si, s) in map.surfaces.iter().enumerate() {
            if s.hidden_brush || !pages.get(usize::from(s.lightmap_index)).is_some_and(Option::is_some) {
                continue;
            }
            for t in 0..u32::from(s.tri_count) {
                let (Some(a), Some(b), Some(c)) = (Self::vert(map, s, 3 * t), Self::vert(map, s, 3 * t + 1), Self::vert(map, s, 3 * t + 2)) else { continue };
                if a.normal[2] + b.normal[2] + c.normal[2] < 0.9 {
                    continue;
                }
                let lo = [a.xyz[0].min(b.xyz[0]).min(c.xyz[0]), a.xyz[1].min(b.xyz[1]).min(c.xyz[1])];
                let hi = [a.xyz[0].max(b.xyz[0]).max(c.xyz[0]), a.xyz[1].max(b.xyz[1]).max(c.xyz[1])];
                let (Some(c0), Some(c1)) = (out.cell(lo[0].max(x0), lo[1].max(y0)), out.cell(hi[0].min(map.maxs[0] - 0.01), hi[1].min(map.maxs[1] - 0.01))) else {
                    continue;
                };
                let (cx0, cy0, cx1, cy1) = (c0 % cols, c0 / cols, c1 % cols, c1 / cols);
                if (cx1 - cx0 + 1) * (cy1 - cy0 + 1) > 256 {
                    continue;
                }
                for cy in cy0..=cy1 {
                    for cx in cx0..=cx1 {
                        out.bins[cy * cols + cx].push((si as u32, 3 * t));
                    }
                }
            }
        }
        out
    }

    /// Every floor at (x, y): its height and its light there.
    fn floors(&self, map: &Iw7Map, pages: &[Option<LightmapPage>], x: f32, y: f32) -> Vec<(f32, LightSample)> {
        let Some(cell) = self.cell(x, y) else { return Vec::new() };
        let mut out = Vec::new();
        for &(si, k) in &self.bins[cell] {
            let s = &map.surfaces[si as usize];
            let (Some(a), Some(b), Some(c)) = (Self::vert(map, s, k), Self::vert(map, s, k + 1), Self::vert(map, s, k + 2)) else { continue };
            // Barycentric in XY.
            let d = (b.xyz[1] - c.xyz[1]) * (a.xyz[0] - c.xyz[0]) + (c.xyz[0] - b.xyz[0]) * (a.xyz[1] - c.xyz[1]);
            if d.abs() < 1e-6 {
                continue;
            }
            let l1 = ((b.xyz[1] - c.xyz[1]) * (x - c.xyz[0]) + (c.xyz[0] - b.xyz[0]) * (y - c.xyz[1])) / d;
            let l2 = ((c.xyz[1] - a.xyz[1]) * (x - c.xyz[0]) + (a.xyz[0] - c.xyz[0]) * (y - c.xyz[1])) / d;
            let l3 = 1.0 - l1 - l2;
            if l1 < -1e-4 || l2 < -1e-4 || l3 < -1e-4 {
                continue;
            }
            let z = l1 * a.xyz[2] + l2 * b.xyz[2] + l3 * c.xyz[2];
            let uv = [0, 1].map(|k| l1 * a.lmap_uv[k] + l2 * b.lmap_uv[k] + l3 * c.lmap_uv[k]);
            if let Some(Some(page)) = pages.get(usize::from(s.lightmap_index)) {
                out.push((z, page.sample(uv)));
            }
        }
        out
    }
}

/// A map's reflection probes as one cube array: per probe six faces, each
/// with its mip chain, as RGBA8 holding `rgb / m` and `1 / m` (`m` the
/// texel's brightest channel, at least 1), which Black Ops II's lit shaders
/// decode as `rgb / a`; in the lightmap pages' exposure.
#[derive(Clone, Debug)]
pub struct ProbeArray {
    pub size: u32,
    pub levels: u32,
    pub count: u32,
    pub data: Vec<u8>,
}

impl Iw7Map {
    /// The reflection probes' cube array (`*reflection_probe_array`: BC6H,
    /// 128 a side, 8 levels, a cube per probe), as RGBA8 (`ProbeArray`).
    /// `exposure` scales every probe; with `normalize` each probe is
    /// scaled further so its top level's mean luminance is 1 (the form
    /// Black Ops II's lit shaders expect: a reflection lit as the light
    /// where it is seen).
    pub fn probe_array(&self, exposure: f32, normalize: bool) -> Result<ProbeArray, String> {
        let name = self.probe_image.as_deref().ok_or("no reflection probe array")?;
        let rec = self.image_info(name).ok_or_else(|| format!("{name}: not in the zone"))?.clone();
        if !matches!(rec.format, 94..=96) {
            return Err(format!("{name}: format {} (BC6H expected)", rec.format));
        }
        let (size, levels) = (u32::from(rec.width), u32::from(rec.level_count.max(1)));
        let count = u32::from(rec.num_elements.max(1));
        let l = self.image(name, 1 << 14)?;
        let face_bytes = crate::texture::chain_size(rec.format, size, size, levels).ok_or("size")?;
        let faces = 6 * count as usize;
        if l.data.len() < face_bytes * faces {
            return Err(format!("{name}: {} bytes for {faces} faces of {face_bytes}", l.data.len()));
        }
        let mut data = Vec::with_capacity(faces * face_bytes * 4);
        let mut tmp = [0f32; 48];
        // Per probe, the scale: `exposure`, over its top level's mean
        // luminance when normalising.
        let lum = |c: &[f32]| 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
        let scales: Vec<f32> = (0..count as usize)
            .map(|p| {
                if !normalize {
                    return exposure;
                }
                let (mut sum, mut n) = (0.0f64, 0usize);
                for f in 6 * p..6 * p + 6 {
                    let at = f * face_bytes;
                    let bw = (size as usize).div_ceil(4);
                    for b in l.data[at..at + bw * bw * 16].chunks_exact(16) {
                        bcdec_rs::bc6h_float(b, &mut tmp, 12, rec.format == 96);
                        for t in tmp.chunks_exact(3) {
                            if t.iter().all(|v| v.is_finite()) {
                                sum += f64::from(lum(t).max(0.0));
                                n += 1;
                            }
                        }
                    }
                }
                let mean = (sum / n.max(1) as f64) as f32;
                if mean > 1e-5 { 1.0 / mean } else { 1.0 }
            })
            .collect();
        for f in 0..faces {
            let exposure = scales[f / 6];
            let mut at = f * face_bytes;
            for level in 0..levels {
                let d = (size >> level).max(1) as usize;
                let bw = d.div_ceil(4);
                let mut px = vec![0u8; d * d * 4];
                for (i, b) in l.data[at..at + bw * bw * 16].chunks_exact(16).enumerate() {
                    bcdec_rs::bc6h_float(b, &mut tmp, 12, rec.format == 96);
                    let (bx, by) = (i % bw * 4, i / bw * 4);
                    for y in 0..4 {
                        for x in 0..4 {
                            if bx + x >= d || by + y >= d {
                                continue;
                            }
                            let t = (y * 4 + x) * 3;
                            let c = [tmp[t], tmp[t + 1], tmp[t + 2]].map(|v| if v.is_finite() { v.max(0.0) * exposure } else { 0.0 });
                            let m = c[0].max(c[1]).max(c[2]).max(1.0).min(255.0);
                            let o = ((by + y) * d + bx + x) * 4;
                            px[o..o + 4].copy_from_slice(&[
                                ((c[0] / m).min(1.0) * 255.0).round() as u8,
                                ((c[1] / m).min(1.0) * 255.0).round() as u8,
                                ((c[2] / m).min(1.0) * 255.0).round() as u8,
                                (255.0 / m).round() as u8,
                            ]);
                        }
                    }
                }
                data.extend_from_slice(&px);
                at += bw * bw * 16;
            }
        }
        Ok(ProbeArray { size, levels, count, data })
    }

    /// The probe nearest a point (0 when there are none).
    pub fn nearest_probe(&self, p: [f32; 3]) -> usize {
        self.probe_origins
            .iter()
            .enumerate()
            .min_by(|a, b| {
                let d = |o: &[f32; 3]| (0..3).map(|k| (o[k] - p[k]).powi(2)).sum::<f32>();
                d(a.1).total_cmp(&d(b.1))
            })
            .map_or(0, |(i, _)| i)
    }
}
