//! A map's baked light, its vision's exposure and tone map, in the form the
//! fallback world draw reads.
//!
//! IW7's lit world shaders (`lmap_specenv_*`, fxc disassembly of retail
//! mp_flip's `lib_lmap_specenv_g0_0__ng0_0__c0_r0_n0_s0_p0_sm_fplusbt_*`)
//! light a surface in linear HDR light:
//!
//! * the secondary (BC6H, two pages stacked) at `(u, v / 2)` is the base
//!   light A, at `(u, v / 2 + 1 / 2)` the directional light B;
//! * the second unorm (R8G8, a page in size) is the dominant light's
//!   tangent-space direction, stored turned 45 degrees as the packed normal
//!   maps are: `r = R * 2 - 1`, `g = G * 2 - 1`, `x = (r + g) / 2`,
//!   `y = (r - g) / 2`, `z = 1 - |x| - |y|`, normalised; in world space
//!   `T x + B y + N z`;
//! * the diffuse light is `A + B * ssao * saturate(N'.D)` (N' the normal
//!   mapped normal);
//! * the primary (BC4, a page in size) is the sun's shadow as a signed
//!   distance: `s = p * 2 - 1`, the sun's visibility
//!   `smoothstep(saturate((s + k) / 2k))` (k a code constant), times the
//!   sun's shadow maps; the sun adds `albedo * sunColour * N'.L *
//!   visibility` (`sunColour` the ComWorld's primary light 0, HDR);
//! * the output is HDR (clamped at 32255), which the post pass tone maps
//!   (`postfx_tonemap_macroclut`, see `Vision`).
//!
//! `LightmapPage` is a page in the form the fallback draw's IW3 branch
//! takes in its linear mode (`WorldDraw::linear_lightmaps`): the secondary
//! as `A8R8G8B8` bytes (B, G, R, A), the light times the map's draw scale
//! (`Vision::draw_scale`) over `LIGHT_RANGE` as `sqrt` (the shader squares
//! each page and adds them up, times `LIGHT_RANGE`), the direction as Call
//! of Duty 4's tangent slopes in the alphas (`a * 4.08 - 2.08`,
//! `a * 4.064516 - 2.064516`); the primary as the sun's signed distance
//! (the shader thresholds it, `SUN_SHADOW_SOFTNESS`).

use crate::map::Iw7Map;

/// The headroom of a page's light over the draw's unit (`sqrt(light /
/// LIGHT_RANGE)` in eight bits: four, an exposed 256, lit lamps and
/// sunlit bounce included).
pub const LIGHT_RANGE: f32 = 4.0;

/// The width of the sun shadow's edge in the primary's signed distance
/// (`k`, half of it each side of 0.5). The fallback draw's linear mode
/// takes the same.
pub const SUN_SHADOW_SOFTNESS: f32 = 0.25;

/// The scale IW7's exposure (`tonemapExposure` plus
/// `tonemapExposureAdjust`, in stops) turns its HDR light by:
/// `EXPOSURE_K * 2^-stops`, the tone map's input (`Vision::exposure`).
/// Fitted to the maps' preview pictures (see `iw7_render`).
pub const EXPOSURE_K: f32 = 32768.0;

/// What IW7's automatic exposure (`tonemapAuto`) brings the scene's
/// log-average HDR luminance to, exposed (the tone map's input). Fitted to
/// the maps' preview pictures (see `iw7_render`).
pub const AUTO_KEY: f32 = 4.0;

/// The albedo the metering proxy (`Iw7Map::metered_light`) takes surfaces
/// to have.
pub const METER_ALBEDO: f32 = 0.25;

/// What the proxy's estimate is times to stand for the views' own metering
/// (the median over the multiplayer maps of their ratio, `iw7_render
/// meter:8` against `metered_light`).
pub const PROXY_TO_VIEW: f32 = 0.45;

/// The log-average HDR luminance of each multiplayer map's views, as IW7's
/// automatic exposure meters them: `iw7_render <map> - meter:8` (eight
/// spawn views spread over the spawn list, the geometric mean of each
/// view's log-average; the sky drawn).
pub const METERED: &[(&str, f32)] = &[
    ("mp_afghan", 0.39135),
    ("mp_breakneck", 0.03851),
    ("mp_carnage2", 0.28498),
    ("mp_codphish", 0.13218),
    ("mp_depot", 0.03762),
    ("mp_desert", 0.10671),
    ("mp_divide", 0.21472),
    ("mp_dome_iw", 0.35551),
    ("mp_fallen", 1.25439),
    ("mp_flip", 0.11408),
    ("mp_frontier", 0.12695),
    ("mp_geneva", 0.07642),
    ("mp_hawkwar", 0.29666),
    ("mp_junk", 0.04828),
    ("mp_mansion", 0.67600),
    ("mp_marsoasis", 0.51156),
    ("mp_metropolis", 1.33450),
    ("mp_neon", 1.55041),
    ("mp_nova", 0.72618),
    ("mp_overflow", 0.03780),
    ("mp_paris", 0.10544),
    ("mp_parkour", 0.18395),
    ("mp_permafrost2", 0.03364),
    ("mp_pixel", 0.82741),
    ("mp_prime", 0.45268),
    ("mp_proto", 5.51926),
    ("mp_quarry", 0.02308),
    ("mp_rally", 0.28594),
    ("mp_renaissance2", 0.09453),
    ("mp_riot", 0.87391),
    ("mp_rivet", 0.02487),
    ("mp_skyway", 0.34679),
    ("mp_turista2", 0.56764),
];

/// The metered light of a map: its views' (`METERED`), else the proxy's
/// estimate (`PROXY_TO_VIEW`).
pub fn metered_view_light(map: &str, proxy: Option<f32>) -> Option<f32> {
    METERED.iter().find(|(m, _)| m.eq_ignore_ascii_case(map)).map(|(_, l)| *l).or(proxy.map(|p| p * PROXY_TO_VIEW))
}

/// The exposed light (the tone map's input) the draw's unit light stands
/// for: the draw works in `exposed / DRAW_UNIT` (the BO2 output's square
/// root of 1 is an exposed 64; its roll-off and the grade's table go to
/// about 250).
pub const DRAW_UNIT: f32 = 64.0;

/// A map's vision (`vision/<map>.vision`): its exposure and tone map
/// (`tonemapMode 2` on every multiplayer map: Insomniac's (Mike Day's)
/// piecewise rational curve, which `postfx_tonemap_macroclut` evaluates
/// per channel as `(a x + b) / (c x + d)`, toe below the crossover,
/// shoulder above), its colour table and sky.
#[derive(Clone, Debug, PartialEq)]
pub struct Vision {
    /// `tonemapExposure - tonemapExposureAdjust` (stops; 12 when unset,
    /// mp_neon's -10).
    pub ev: f32,
    /// `tonemapAuto`: the game adapts the exposure at run time.
    pub auto: bool,
    /// `tonemapAutoExposureAdjust` (stops, brighter when positive).
    pub auto_adjust: f32,
    /// `tonemapExposureAdjust` (stops).
    pub adjust: f32,
    /// `tonemapMaxExposure`: the brightest exposure the automatic one may
    /// take (stops; none when far below zero).
    pub max_exposure: Option<f32>,
    pub black: f32,
    pub white: f32,
    pub crossover: f32,
    pub shoulder: f32,
    pub toe: f32,
    /// `clut0Image`: the 32^3 colour table looked up after the tone map.
    pub clut: Option<String>,
    /// `r_hdrSkyIntensity`.
    pub sky_intensity: f32,
}

impl Default for Vision {
    fn default() -> Self {
        Vision { ev: 12.0, auto: true, auto_adjust: 0.0, adjust: 0.0, max_exposure: None, black: 0.0, white: 650.0, crossover: 2.0, shoulder: 0.95, toe: 0.2, clut: None, sky_intensity: 1.0 }
    }
}

/// A vision line's first value (`name "values"`).
pub fn vision_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.trim().split_once(char::is_whitespace)?;
        k.eq_ignore_ascii_case(key).then(|| v.trim().trim_matches('"').to_owned())
    })
}

impl Vision {
    pub fn parse(text: &str) -> Self {
        let d = Vision::default();
        let num = |k: &str| vision_value(text, k).and_then(|v| v.split_whitespace().next()?.parse::<f32>().ok()).filter(|v| v.is_finite());
        // (The adjustments are exposure compensation, brighter when
        // positive: mp_mansion's and mp_pixel's +1.5 and mp_dome_iw's +1
        // drew those maps darkest of all taken the other way.)
        let ev = num("tonemapExposure").filter(|&e| e > 0.0).unwrap_or(d.ev) - num("tonemapExposureAdjust").unwrap_or(0.0);
        let white = num("tonemapWhitePoint").filter(|&w| w > 0.0).unwrap_or(d.white);
        let crossover = num("tonemapCrossover").filter(|&c| c > 0.0 && c < white).unwrap_or(d.crossover);
        Vision {
            ev,
            auto: num("tonemapAuto").is_none_or(|v| v != 0.0),
            auto_adjust: num("tonemapAutoExposureAdjust").unwrap_or(0.0),
            adjust: num("tonemapExposureAdjust").unwrap_or(0.0),
            max_exposure: num("tonemapMaxExposure").filter(|&m| m > -5.0),
            black: num("tonemapBlackPoint").unwrap_or(d.black).clamp(0.0, crossover * 0.5),
            white,
            crossover,
            shoulder: num("tonemapShoulder").unwrap_or(d.shoulder).clamp(0.0, 0.999),
            toe: num("tonemapToe").unwrap_or(d.toe).clamp(0.0, 0.999),
            clut: vision_value(text, "clut0Image").filter(|c| !c.is_empty()),
            sky_intensity: num("r_hdrSkyIntensity").unwrap_or(1.0).max(0.0),
        }
    }

    /// The scale from HDR light to the tone map's input, fixed
    /// (`tonemapExposure` plus its adjustment).
    pub fn exposure(&self) -> f32 {
        EXPOSURE_K * (-self.ev).exp2()
    }

    /// The scale from HDR light to the tone map's input for a scene of
    /// log-average HDR luminance `metered`: with `tonemapAuto`, the one
    /// bringing it to `AUTO_KEY` (moved by the vision's adjustments), no
    /// brighter than `tonemapMaxExposure`; else the fixed one.
    pub fn exposure_for(&self, metered: f32) -> f32 {
        if !self.auto || !(metered.is_finite() && metered > 0.0) {
            return self.exposure();
        }
        let e = AUTO_KEY / metered * (self.auto_adjust + self.adjust).exp2();
        match self.max_exposure {
            Some(m) => e.min(EXPOSURE_K * (-m).exp2()),
            None => e,
        }
    }

    /// The scale from HDR light to the draw's units (`DRAW_UNIT`), fixed.
    pub fn draw_scale(&self) -> f32 {
        self.exposure() / DRAW_UNIT
    }

    /// The same for a metered scene (`exposure_for`).
    pub fn draw_scale_for(&self, metered: f32) -> f32 {
        self.exposure_for(metered) / DRAW_UNIT
    }

    /// The tone map of an exposed value, linear 0..1: toe `k (1 - t) (x -
    /// b) / (c - (1 - t) b - t x)` below the crossover c, shoulder `k + (1 -
    /// k) (x - c) / (s x + (1 - s) w - c)` above, `k = (1 - t)(c - b) / ((1 -
    /// s)(w - c) + (1 - t)(c - b))` (both meet at c with one slope).
    pub fn tonemap(&self, x: f32) -> f32 {
        let (b, w, c, s, t) = (self.black, self.white, self.crossover, self.shoulder, self.toe);
        let k = (1.0 - t) * (c - b) / ((1.0 - s) * (w - c) + (1.0 - t) * (c - b));
        let x = x.max(0.0);
        let y = if x < c {
            k * (1.0 - t) * (x - b) / (c - (1.0 - t) * b - t * x)
        } else {
            k + (1.0 - k) * (x - c) / (s * x + (1.0 - s) * w - c)
        };
        y.clamp(0.0, 1.0)
    }
}

/// The sun's visibility from the primary's signed distance (0..1, 0.5 the
/// shadow's edge), as the lit shaders threshold it.
pub fn sun_visibility(p: f32) -> f32 {
    let k = SUN_SHADOW_SOFTNESS;
    let t = ((p * 2.0 - 1.0 + k) / (2.0 * k)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Black Ops II's highlight roll-off above 0.75 (gamma values), and its
/// inverse (which the final pass applies before the grade's table).
pub fn bo2_rolloff(e: f32) -> f32 {
    if e <= 0.75 { e.max(0.0) } else { 0.75 + 0.25 * (1.0 - (-5.77078 * (e - 0.75)).exp2()) }
}

pub fn bo2_unroll(r: f32) -> f32 {
    if r <= 0.75 {
        r.max(0.0)
    } else {
        let rest = ((1.0 - r) * 4.0).max(0.5 / 255.0 * 4.0);
        0.75 - rest.log2() / 5.77078
    }
}

/// Side of a colour table.
pub const LUT_SIZE: usize = 32;

/// Trilinear lookup in a 32^3 RGBA8 table (red fastest, then green, then
/// blue) of a colour 0..1.
pub fn lut_sample(lut: &[u8], c: [f32; 3]) -> [f32; 3] {
    let n = LUT_SIZE;
    if lut.len() < n * n * n * 4 {
        return c;
    }
    let f: [f32; 3] = c.map(|v| v.clamp(0.0, 1.0) * (n - 1) as f32);
    let i0: [usize; 3] = f.map(|v| (v.floor() as usize).min(n - 1));
    let i1: [usize; 3] = i0.map(|v| (v + 1).min(n - 1));
    let t: [f32; 3] = std::array::from_fn(|k| f[k] - i0[k] as f32);
    let at = |r: usize, g: usize, b: usize, ch: usize| f32::from(lut[((b * n + g) * n + r) * 4 + ch]) / 255.0;
    std::array::from_fn(|ch| {
        let mut v = 0.0;
        for (bz, wb) in [(i0[2], 1.0 - t[2]), (i1[2], t[2])] {
            for (gy, wg) in [(i0[1], 1.0 - t[1]), (i1[1], t[1])] {
                for (rx, wr) in [(i0[0], 1.0 - t[0]), (i1[0], t[0])] {
                    v += wb * wg * wr * at(rx, gy, bz, ch);
                }
            }
        }
        v
    })
}

fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// IW7's final colour of a light in the draw's units (`DRAW_UNIT`): the
/// tone map per channel, sRGB, then the colour table (`clut`, the vision's
/// `clut0Image`: 32^3 RGBA8, red fastest; `None`: none).
pub fn iw7_display(v: &Vision, clut: Option<&[u8]>, x: [f32; 3]) -> [f32; 3] {
    let s = x.map(|c| linear_to_srgb(v.tonemap(c * DRAW_UNIT)));
    match clut {
        Some(t) => lut_sample(t, s),
        None => s,
    }
}

/// The table Black Ops II's final pass looks the frame up in
/// (`asset_world::T6Grade::lut`: 32^3 RGBA8, red fastest; its input the
/// frame's rolled-off square root of the light) that gives IW7's final
/// colour: each entry unrolled, squared and put through `iw7_display`.
pub fn grade_lut(v: &Vision, clut: Option<&[u8]>) -> Vec<u8> {
    let n = LUT_SIZE;
    let mut out = Vec::with_capacity(n * n * n * 4);
    for b in 0..n {
        for g in 0..n {
            for r in 0..n {
                let x = [r, g, b].map(|i| {
                    let e = bo2_unroll(i as f32 / (n - 1) as f32);
                    e * e
                });
                let d = iw7_display(v, clut, x);
                out.extend(d.map(|c| (c * 255.0).round().clamp(0.0, 255.0) as u8));
                out.push(255);
            }
        }
    }
    out
}

/// One lightmap page in the fallback draw's linear form.
#[derive(Clone, Debug)]
pub struct LightmapPage {
    /// Width, height, the sun's signed distance as L8 texels.
    pub primary: (u16, u16, Vec<u8>),
    /// Width, height (two pages), B, G, R, A bytes.
    pub secondary: (u16, u16, Vec<u8>),
    /// The scale the HDR light was given (`Vision::draw_scale`).
    pub scale: f32,
    /// The geometric mean of the lit texels' base plus directional HDR
    /// light (for reports).
    pub mean_light: f32,
}

/// What a point on the world takes from its lightmap: the base and
/// directional light (the draw's linear units) and the sun's visibility.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightSample {
    pub base: [f32; 3],
    pub directional: [f32; 3],
    pub sun: f32,
    /// How much of the directional light a surface lying along the page
    /// takes (`N.D` for the vertex normal: the direction's tangent z).
    pub facing: f32,
}

impl LightSample {
    /// The light of a surface facing the dominant light, without the sun
    /// (linear, the draw's units).
    pub fn ambient_linear(&self) -> [f32; 3] {
        std::array::from_fn(|k| self.base[k] + self.directional[k])
    }
}

impl LightmapPage {
    /// The page at `(u, v)` (bilinear).
    pub fn sample(&self, uv: [f32; 2]) -> LightSample {
        let (sw, sh, s) = (usize::from(self.secondary.0), usize::from(self.secondary.1), &self.secondary.2);
        let page_h = sh / 2;
        // Texel centres, clamped.
        let taps = |w: usize, h: usize| {
            let fx = (uv[0].clamp(0.0, 1.0) * w as f32 - 0.5).max(0.0);
            let fy = (uv[1].clamp(0.0, 1.0) * h as f32 - 0.5).max(0.0);
            let (x0, y0) = ((fx as usize).min(w - 1), (fy as usize).min(h - 1));
            let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
            let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
            [(x0, y0, (1.0 - tx) * (1.0 - ty)), (x1, y0, tx * (1.0 - ty)), (x0, y1, (1.0 - tx) * ty), (x1, y1, tx * ty)]
        };
        let light = |page: usize| -> [f32; 3] {
            let mut acc = [0f32; 3];
            for (x, y, wt) in taps(sw, page_h) {
                let o = ((page * page_h + y) * sw + x) * 4;
                for (k, b) in [s[o + 2], s[o + 1], s[o]].into_iter().enumerate() {
                    let g = f32::from(b) / 255.0;
                    acc[k] += wt * g * g * LIGHT_RANGE;
                }
            }
            acc
        };
        let mut slope = [0f32; 2];
        for (x, y, wt) in taps(sw, page_h) {
            for (page, (scale, bias)) in [(4.08f32, 2.08f32), (4.064516, 2.064516)].into_iter().enumerate() {
                let a = f32::from(s[((page * page_h + y) * sw + x) * 4 + 3]) / 255.0;
                slope[page] += wt * (a * scale - bias);
            }
        }
        let facing = (1.0 + slope[0] * slope[0] + slope[1] * slope[1]).sqrt().recip();
        let (pw, ph) = (usize::from(self.primary.0), usize::from(self.primary.1));
        let mut sdf = 0.0;
        for (x, y, wt) in taps(pw, ph) {
            sdf += wt * f32::from(self.primary.2[y * pw + x]) / 255.0;
        }
        LightSample { base: light(0), directional: light(1), sun: sun_visibility(sdf), facing }
    }
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

/// The dominant light's tangent-space direction from the second unorm's
/// bytes (turned 45 degrees, as the lit shaders read it), unit length.
pub fn light_direction(r: u8, g: u8) -> [f32; 3] {
    let (r, g) = (f32::from(r) / 255.0 * 2.0 - 1.0, f32::from(g) / 255.0 * 2.0 - 1.0);
    let (x, y) = ((r + g) * 0.5, (r - g) * 0.5);
    let z = (1.0 - x.abs() - y.abs()).max(0.0);
    let l = (x * x + y * y + z * z).sqrt().max(1e-6);
    [x / l, y / l, z / l]
}

impl Iw7Map {
    /// The map's vision (`Vision::default` when it has none).
    pub fn vision_params(&self) -> Vision {
        self.vision.as_ref().map_or_else(Vision::default, |(_, t)| Vision::parse(t))
    }

    /// The vision's colour table (`clut0Image`) as 32^3 RGBA8, red fastest.
    pub fn colour_table(&self, v: &Vision) -> Option<Vec<u8>> {
        let name = v.clut.as_deref()?;
        let rec = self.image_info(name)?;
        if usize::from(rec.width) != LUT_SIZE || usize::from(rec.height) != LUT_SIZE || usize::from(rec.depth) != LUT_SIZE {
            return None;
        }
        let l = self.image(name, 64).ok()?;
        let n = LUT_SIZE * LUT_SIZE * LUT_SIZE;
        match l.format {
            // R8G8B8A8 (UNORM, sRGB: the bytes as they are), B8G8R8A8.
            27..=32 if l.data.len() >= n * 4 => Some(l.data[..n * 4].to_vec()),
            87 | 88 | 90 | 91 if l.data.len() >= n * 4 => Some(l.data[..n * 4].chunks_exact(4).flat_map(|p| [p[2], p[1], p[0], p[3]]).collect()),
            _ => None,
        }
    }

    /// Light map `index` as the fallback draw's linear page, its pages at
    /// most `max_width` wide (whole halvings), the HDR light times `scale`
    /// (`Vision::draw_scale`).
    pub fn lightmap_page(&self, index: usize, max_width: u32, scale: f32) -> Result<LightmapPage, String> {
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
        // The directions (a page in size): each texel's unit vector,
        // averaged over the texels a page texel covers, as Call of Duty
        // 4's tangent slopes (x / z, y / z).
        let dirs: Option<Vec<[f32; 2]>> = self.image(&names[2], 1 << 14).ok().and_then(|d| {
            let (dw, dh) = (d.width as usize, d.height as usize);
            (d.format == 49 && dw > 0 && dh > 0 && d.data.len() >= dw * dh * 2).then(|| {
                let (fx, fy) = ((dw / ow).max(1), (dh / opage).max(1));
                let mut out = vec![[0.0f32; 2]; ow * opage];
                for (i, o) in out.iter_mut().enumerate() {
                    let (x, y) = (i % ow, i / ow);
                    let mut acc = [0f32; 3];
                    for dy in 0..fy {
                        for dx in 0..fx {
                            let (sx, sy) = (x * fx + dx, y * fy + dy);
                            if sx < dw && sy < dh {
                                let at = (sy * dw + sx) * 2;
                                let v = light_direction(d.data[at], d.data[at + 1]);
                                for k in 0..3 {
                                    acc[k] += v[k];
                                }
                            }
                        }
                    }
                    let z = acc[2].max(1e-3 * (acc[0].abs() + acc[1].abs()).max(1e-6));
                    *o = [acc[0] / z, acc[1] / z];
                }
                out
            })
        });
        // For reports: the lit texels' geometric mean (base page plus the
        // directional page under it).
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
        let mean_light = if lit > 0 { (log_sum / lit as f64).exp() as f32 } else { 0.0 };
        let gamma = |v: f32| ((v * scale / LIGHT_RANGE).clamp(0.0, 1.0).sqrt() * 255.0).round() as u8;
        let mut secondary = vec![0u8; ow * oh * 4];
        for y in 0..oh {
            for x in 0..ow {
                let (page, py) = (y / opage, y % opage);
                let d = dirs.as_ref().map_or([0.0, 0.0], |d| d[py * ow + x]);
                let a = if page == 0 { ((d[0] + 2.08) / 4.08 * 255.0).round() } else { ((d[1] + 2.064516) / 4.064516 * 255.0).round() };
                let l = small[y * ow + x];
                let o = (y * ow + x) * 4;
                secondary[o..o + 4].copy_from_slice(&[gamma(l[2]), gamma(l[1]), gamma(l[0]), a.clamp(0.0, 255.0) as u8]);
            }
        }
        // The primary (the sun's signed distance), down to the page's size.
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
            // No sun shadow: lit everywhere.
            _ => vec![255u8; ow * opage],
        };
        let to16 = |v: usize| u16::try_from(v).map_err(|_| format!("{v} is too large"));
        Ok(LightmapPage { primary: (to16(ow)?, to16(opage)?, primary), secondary: (to16(ow)?, to16(oh)?, secondary), scale, mean_light })
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

    /// The log-average HDR luminance a player sees round `points` (spawn
    /// points), estimated for the automatic exposure (`Vision::exposure_for`):
    /// at each, the floor's light (base, three quarters of the directional)
    /// plus half the sun where the floor sees it, times `METER_ALBEDO`.
    /// `None` when no point stands over a lit floor.
    pub fn metered_light(&self, pages: &[Option<LightmapPage>], points: &[[f32; 3]]) -> Option<f32> {
        let scale = pages.iter().flatten().next()?.scale;
        if scale <= 0.0 {
            return None;
        }
        let sun = self.lights.iter().find(|l| l.kind == 1).map_or([0.0; 3], |l| l.color);
        let lifted: Vec<[f32; 3]> = points.iter().map(|p| [p[0], p[1], p[2] + 8.0]).collect();
        let (mut sum, mut n) = (0f64, 0usize);
        for s in self.ground_light(pages, &lifted, 256.0).into_iter().flatten() {
            let c: [f32; 3] = std::array::from_fn(|k| (s.base[k] + 0.75 * s.directional[k]) / scale + 0.5 * sun[k] * s.sun);
            let l = (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]) * METER_ALBEDO;
            if l > 1e-6 && l.is_finite() {
                sum += f64::from(l.ln());
                n += 1;
            }
        }
        (n > 0).then(|| (sum / n as f64).exp() as f32)
    }

    /// The light each placed model takes: the light grid's probes round its
    /// lighting origin (`ProbeIndex`), as IW7 lights them; where no probe is
    /// near, the floor's light under it and around it (eight points on a
    /// ring of about its radius), the middle one by brightness. The floor right under a model is in its own baked
    /// shadow (mp_proto's crates, mp_hawkwar's station wagon drew black by
    /// it); one ring point out in the sun would light a model in the shade.
    /// `None` where no floor is found.
    pub fn static_model_light(&self, pages: &[Option<LightmapPage>]) -> Vec<Option<LightSample>> {
        let floors = FloorIndex::new(self, pages);
        // The light grid's probes at the model's lighting origin, as IW7
        // lights its placed models (`lprobe_*`); the floor where none is
        // near.
        let probes = ProbeIndex::new(self);
        let scale = pages.iter().flatten().next().map_or(1.0, |p| p.scale);
        self.static_models
            .iter()
            .map(|sm| {
                if let Some(l) = probes.light(self, sm.lighting_origin, scale) {
                    return Some(l);
                }
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
    /// light grid's probes round each point (`ProbeIndex`), else the light
    /// of the floor under it (`ground_light`, within `reach` units), or
    /// `None` where there is neither.
    pub fn light_volume(&self, pages: &[Option<LightmapPage>], mins: [f32; 3], maxs: [f32; 3], spacing: [f32; 3], reach: f32) -> LightVolume {
        let floors = FloorIndex::new(self, pages);
        let probes = ProbeIndex::new(self);
        let scale = pages.iter().flatten().next().map_or(1.0, |p| p.scale);
        let dims: [usize; 3] = std::array::from_fn(|k| (((maxs[k] - mins[k]) / spacing[k]).floor().max(0.0) as usize + 1).min(4096));
        let mut samples = Vec::with_capacity(dims[0] * dims[1] * dims[2]);
        for ix in 0..dims[0] {
            for iy in 0..dims[1] {
                let (x, y) = (mins[0] + ix as f32 * spacing[0], mins[1] + iy as f32 * spacing[1]);
                let mut col = floors.floors(self, pages, x, y);
                col.sort_by(|a, b| a.0.total_cmp(&b.0));
                for iz in 0..dims[2] {
                    let z = mins[2] + iz as f32 * spacing[2];
                    // (The light grid's probes where there are some near,
                    // as IW7 lights moving things; else the floor's.)
                    if let Some(l) = probes.light(self, [x, y, z], scale) {
                        samples.push(Some(l));
                        continue;
                    }
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

/// The light grid's probes (`Iw7Map::light_probes`) binned in cubes, to
/// find the light at a point.
///
/// A probe is 27 halves, the red, green and blue light's second order
/// spherical harmonics (nine each, the real basis IW7's `lprobe_*` vertex
/// shaders evaluate: 0.282095, 0.488603, 1.092548, 0.315392, 0.546274),
/// then the sun's visibility (a half), then padding. Read on mp_riot and
/// mp_frontier: the first coefficient times 0.282095 (the light averaged
/// over every direction, a lightmap's units) is the floor's base plus most
/// of its directional light under it, and the visibility is the floor's
/// sun shadow (0.97 in the sun, 0 in the shade).
pub struct ProbeIndex {
    cells: std::collections::HashMap<[i32; 3], Vec<u32>>,
}

impl ProbeIndex {
    const CELL: f32 = 128.0;

    pub fn new(map: &Iw7Map) -> Self {
        let mut cells: std::collections::HashMap<[i32; 3], Vec<u32>> = std::collections::HashMap::new();
        for (i, (p, _)) in map.light_probes.iter().enumerate() {
            cells.entry(p.map(|v| (v / Self::CELL).floor() as i32)).or_default().push(i as u32);
        }
        ProbeIndex { cells }
    }

    /// One probe's light averaged over every direction (HDR) and its sun
    /// visibility.
    pub fn probe(d: &[u8; 64]) -> ([f32; 3], f32) {
        let h = |i: usize| half(u16::from_le_bytes([d[2 * i], d[2 * i + 1]]));
        let avg = [h(0), h(9), h(18)].map(|c| if c.is_finite() { (c * 0.282_095).max(0.0) } else { 0.0 });
        let vis = h(27);
        (avg, if vis.is_finite() { vis.clamp(0.0, 1.0) } else { 0.0 })
    }

    /// The light at `p` (the draw's units, `scale` times HDR): the probes
    /// within two cells, weighted by the inverse square of their distance;
    /// `None` with none near.
    pub fn light(&self, map: &Iw7Map, p: [f32; 3], scale: f32) -> Option<LightSample> {
        let c = p.map(|v| (v / Self::CELL).floor() as i32);
        let (mut acc, mut vis, mut wsum) = ([0f32; 3], 0f32, 0f32);
        for reach in [1, 2] {
            for dx in -reach..=reach {
                for dy in -reach..=reach {
                    for dz in -reach..=reach {
                        let Some(list) = self.cells.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) else { continue };
                        for &i in list {
                            let (q, d) = &map.light_probes[i as usize];
                            let d2: f32 = (0..3).map(|k| (q[k] - p[k]).powi(2)).sum();
                            if d2 > (reach as f32 * Self::CELL).powi(2) {
                                continue;
                            }
                            let w = 1.0 / (d2 + 64.0);
                            let (l, v) = Self::probe(d);
                            for k in 0..3 {
                                acc[k] += w * l[k];
                            }
                            vis += w * v;
                            wsum += w;
                        }
                    }
                }
            }
            if wsum > 0.0 {
                break;
            }
        }
        (wsum > 0.0).then(|| LightSample { base: acc.map(|a| a / wsum * scale), directional: [0.0; 3], sun: vis / wsum, facing: 1.0 })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_map_meets_at_the_crossover() {
        let v = Vision::default();
        let c = v.crossover;
        let (a, b) = (v.tonemap(c - 1e-3), v.tonemap(c + 1e-3));
        assert!((a - b).abs() < 1e-3, "{a} {b}");
        assert!((v.tonemap(v.white) - 1.0).abs() < 1e-4);
        assert_eq!(v.tonemap(0.0), 0.0);
        // Rising throughout.
        let mut last = -1.0;
        for i in 0..2000 {
            let y = v.tonemap(i as f32 * 0.5);
            assert!(y >= last);
            last = y;
        }
    }

    #[test]
    fn roll_off_turns_back() {
        for i in 0..95 {
            let e = i as f32 / 50.0;
            assert!((bo2_unroll(bo2_rolloff(e)) - e).abs() < 2e-3, "{e}");
        }
    }

    #[test]
    fn directions_turn_back() {
        // Straight out: R = G = 128.
        let d = light_direction(128, 128);
        assert!(d[2] > 0.99, "{d:?}");
        // Along x: r = g > 0.
        let d = light_direction(191, 191);
        assert!(d[0] > 0.3 && d[1].abs() < 0.02, "{d:?}");
    }

    #[test]
    fn vision_reads() {
        let v = Vision::parse("tonemapExposure \"11.5\"\ntonemapExposureAdjust \"-0.25\"\ntonemapWhitePoint \"650\"\nclut0Image \"mp_riot_final_clut\"\n");
        assert_eq!(v.ev, 11.75);
        assert_eq!(v.clut.as_deref(), Some("mp_riot_final_clut"));
        let lut = grade_lut(&v, None);
        assert_eq!(lut.len(), 32 * 32 * 32 * 4);
        // Black stays black; the table rises.
        assert_eq!(&lut[..3], &[0, 0, 0]);
        let last = lut.len() - 4;
        assert!(lut[last] > 200);
    }
}
