//! bo2mp: a Black Ops III world or prop material's look in the form Black
//! Ops II's lit draw reads: its colour map (tinted, its ambient occlusion
//! and its metal's diffuse taken out), its normal map and its specular map.
//!
//! Black Ops III draws the world deferred; its G-buffer programs
//! (`gbuffer_lit`, `gbuffer_decal`, `gbuffer_vegetation`, ...; read with
//! `t7_dxbc` and `fxc /dumpbin` from mp_biodome's and core_common's
//! technique sets) fill the G-buffer from the material's textures and its
//! `$Globals` numbers (`MapMaterial::constants`, by the names the programs
//! give them):
//!
//! * albedo: `colorMap.rgb * lerp(1, colorTint, mask) * vertex colour`, the
//!   mask `tintMask.r` when the technique binds a tint mask, else
//!   `colorMap.a` (an opaque colour map's alpha is its tint mask);
//! * metal: where the technique binds `specColorMap`, its red is a metal
//!   mask `m`: albedo `* (1 - m)`, reflectance `lerp(0.04, specColorTint,
//!   m)`; elsewhere the reflectance is 0.04;
//! * gloss: `lerp(glossRange.x, glossRange.y, glossMap.r) / 17` (no gloss
//!   map: `glossRange.y / 17`), less the normal map's variance (its blue:
//!   `v = min(b^2 / 3, 1)`; `g' = -log2(2^(-17 g) + v) / 17`); the lit
//!   programs' highlight power is `2^(17 g')`;
//! * normal: `xy = ((rg - 0.5) * baseNormalHeight + 0.5) * 1.992188 - 1`,
//!   `z = sqrt(1 - x^2 - y^2)` (tangent space); a detail normal map
//!   (`detailMap`, at `uv * detailScale.xy`) adds its `xy *
//!   detailScaleHeight`, and its variance;
//! * occlusion: `aoMap.r`, Black Ops III's ambient light and reflections'.
//!
//! Micro tile blends (`gbuffer_micro_tile_blend`: most walls, floors and
//! props) build their surface from two tiled layers over a macro map:
//! layer 1 `colorMap` at `uv * detailScale1.xy` (turned `rotateUVs1`
//! radians about the middle), tinted by `colorTint` through its alpha;
//! layer 2 `colorMapDetail2` at `uv * detailScale2.xy` (`rotateUVs2`),
//! tinted by `colorTint1`; mixed by `mixMap.r` at `uv * mixScale`. The
//! normal is the macro `normalMap` (at uv) plus the layer's detail normal
//! map (`detailMap1`, `detailMap2`) times `detailScaleHeight1`/`2`; gloss
//! the layer's gloss map (`glossMap`, `glossMapDetail2`) over `glossRange`
//! / `glossRange2`; `aoMap` at uv. Here the layers are baked into one
//! picture over uv (each tile shrunk to the texels it covers).
//!
//! Black Ops II's lit draw reads a colour map (sRGB, squared), a normal map
//! (BC5: the slope x / z, y / z over -1..1, as Ghosts' are) and a specular
//! map (sRGB colour whose square is the reflectance, gloss in alpha: power
//! `2^(13 gloss)`). Made here at most `max_dim` on a side (the specular map
//! at most `spec_max`), each with its full mip chain. The occlusion goes
//! into the colour map (Black Ops II's fallback draw has no ambient
//! occlusion of its own).

use crate::map::{MapMaterial, T7Map};
use crate::texture::{self, ImageLevels};

/// How a material covers what is behind it (what its colour map's alpha
/// is).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cover {
    /// Opaque: alpha is the tint mask; the colour map made opaque.
    Opaque,
    /// Cut out or blended (decals, foliage, glass): alpha is coverage, kept.
    Alpha,
}

/// A material's look (`T7Map::look`).
#[derive(Clone, Debug, Default)]
pub struct Look {
    /// BC1 sRGB (`Cover::Opaque`) or BC3 sRGB.
    pub colour: Option<ImageLevels>,
    /// BC5 (`DXGI_FORMAT_BC5_UNORM`): the slope over -1..1.
    pub normal: Option<ImageLevels>,
    /// BC3 sRGB: reflectance (square-rooted), gloss in alpha.
    pub specular: Option<ImageLevels>,
    /// What was used, for the report: tinted, metal, gloss map, ao, detail.
    pub used: [bool; 5],
}

/// RGBA8 pixels.
#[derive(Clone, Debug)]
pub struct Rgba {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u8>,
}

impl Rgba {
    /// The texel at uv (wrapped), bilinear.
    fn sample(&self, u: f32, v: f32, c: usize) -> f32 {
        let (w, h) = (self.w as f32, self.h as f32);
        let (x, y) = (u.rem_euclid(1.0) * w - 0.5, v.rem_euclid(1.0) * h - 0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let at = |xi: f32, yi: f32| {
            let xi = (xi as i64).rem_euclid(self.w as i64) as usize;
            let yi = (yi as i64).rem_euclid(self.h as i64) as usize;
            f32::from(self.px[(yi * self.w as usize + xi) * 4 + c])
        };
        let top = at(x0, y0) * (1.0 - fx) + at(x0 + 1.0, y0) * fx;
        let bottom = at(x0, y0 + 1.0) * (1.0 - fx) + at(x0 + 1.0, y0 + 1.0) * fx;
        (top * (1.0 - fy) + bottom * fy) / 255.0
    }
    /// The texel for pixel (x, y) of a `w` x `h` grid over the same uv.
    fn at(&self, x: u32, y: u32, w: u32, h: u32, c: usize) -> f32 {
        if self.w == w && self.h == h {
            return f32::from(self.px[((y * w + x) * 4) as usize + c]) / 255.0;
        }
        self.sample((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32, c)
    }
    /// Halved until at most `max` on a side.
    fn at_most(mut self, max: u32) -> Self {
        while self.w.max(self.h) > max && self.w > 4 && self.h > 4 {
            let (w, h, px) = half_level(self.w, self.h, &self.px);
            self = Self { w, h, px };
        }
        self
    }
}

/// An engine placeholder picture (`$white_gloss`, `$black_color`,
/// `$identitynormalmap`, ...) as the one colour it is.
pub fn placeholder(name: &str) -> Option<[u8; 4]> {
    let n = name.trim_start_matches(',');
    let n = n.strip_prefix('$')?;
    Some(if n.contains("normal") {
        [128, 128, 0, 255]
    } else if n.contains("black") {
        [0, 0, 0, 255]
    } else if n.contains("gray") || n.contains("grey") {
        [128, 128, 128, 255]
    } else if n.contains("specular_mask") {
        // (A dielectric: no metal.)
        [0, 0, 0, 255]
    } else {
        [255, 255, 255, 255]
    })
}

/// sRGB byte to linear.
pub fn to_linear(b: u8) -> f32 {
    static LUT: std::sync::OnceLock<[f32; 256]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| {
        std::array::from_fn(|i| {
            let x = i as f32 / 255.0;
            if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
        })
    })[usize::from(b)]
}

/// sRGB (0..1, filtered) to linear.
fn to_linear_f(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

/// Linear to an sRGB byte (a 4096-step table: exact to the byte).
pub fn to_srgb(x: f32) -> u8 {
    static LUT: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    let lut = LUT.get_or_init(|| {
        (0..=4096)
            .map(|i| {
                let x = i as f32 / 4096.0;
                let s = if x <= 0.003_130_8 { 12.92 * x } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
                (s * 255.0).round() as u8
            })
            .collect()
    });
    let x = x.clamp(0.0, 1.0);
    // (Below 1/16 the curve is steep: computed.)
    if x < 0.0625 {
        let s = if x <= 0.003_130_8 { 12.92 * x } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
        return (s * 255.0).round() as u8;
    }
    lut[(x * 4096.0).round() as usize]
}

/// Black Ops II's gloss (power `2^(13 g)`) for Black Ops III's (power
/// `2^(17 g)`): the same power, capped at Black Ops II's.
pub fn bo2_gloss(g: f32) -> f32 {
    ((17.0 * g.clamp(0.0, 1.0)).exp2() + 2.0).log2().min(13.0) / 13.0
}

/// Black Ops III's gloss less a normal map's variance (`v`, from its blue).
fn gloss_less_variance(g: f32, v: f32) -> f32 {
    (-((-17.0 * g).exp2() + v).log2() / 17.0).clamp(0.0, 1.0)
}

/// What a material reads from a texture slot: its picture at most
/// `max_dim`, a placeholder's colour, or nothing.
enum Slot {
    None,
    Solid([u8; 4]),
    Image(Rgba),
}

impl T7Map {
    /// A picture as RGBA8 (its top level at most `max_dim`).
    pub fn rgba(&self, name: &str, max_dim: u32) -> Result<Rgba, String> {
        let l = self.image(name, max_dim)?;
        let (w, h, px) = texture::level_rgba(&l, 0).ok_or_else(|| format!("{name}: format {} not decoded", l.format))?;
        Ok(Rgba { w, h, px }.at_most(max_dim))
    }

    fn slot(&self, m: &MapMaterial, slot: &str, max_dim: u32) -> Result<Slot, String> {
        let Some(name) = m.texture_raw(slot) else { return Ok(Slot::None) };
        if let Some(c) = placeholder(name) {
            return Ok(Slot::Solid(c));
        }
        self.rgba(name, max_dim).map(Slot::Image)
    }

    /// A material's look in Black Ops II's form (see the module). Its
    /// colour map is made when it has a real one (else `colour` is `None`
    /// and the caller draws what it drew before); the normal and specular
    /// maps when the material is lit.
    pub fn look(&self, m: &MapMaterial, max_dim: u32, spec_max: u32, cover: Cover) -> Result<Look, String> {
        if m.texture("mixMap").is_some() && m.constant::<2>("detailScale1").is_some() {
            return self.micro_tile_blend(m, max_dim, spec_max);
        }
        if m.technique_set.starts_with("mc/glass_nocolor") || m.technique_set.starts_with("wc/glass_nocolor") {
            return self.clear_glass(m, max_dim, spec_max);
        }
        self.look_inner(m, max_dim, spec_max, cover)
    }

    /// `look` for a plain lit material.
    fn look_inner(&self, m: &MapMaterial, max_dim: u32, spec_max: u32, cover: Cover) -> Result<Look, String> {
        let mut out = Look::default();
        let binds = |s: &str| m.bound.iter().any(|b| b == s);
        let known = !m.bound.is_empty();
        let c3 = |name: &str, d: [f32; 3]| m.constant::<3>(name).unwrap_or(d);
        let gloss_range = m.constant::<2>("glossRange");
        let tint = c3("colorTint", [1.0; 3]);
        let spec_tint = c3("specColorTint", [0.04; 3]);
        let normal_height = m.constant::<1>("baseNormalHeight").map_or(1.0, |v| v[0]);

        let colour = match self.slot(m, "colorMap", max_dim)? {
            Slot::Image(c) => Some(c),
            _ => None,
        };
        // The texture slots this material's technique reads (all of the
        // material's when its programs are not known).
        let read = |s: &str| !known || binds(s);
        let tint_mask = if read("tintMask") { self.slot(m, "tintMask", max_dim)? } else { Slot::None };
        let metal = if read("specColorMap") { self.slot(m, "specColorMap", spec_max)? } else { Slot::None };
        let gloss = if read("glossMap") { self.slot(m, "glossMap", spec_max)? } else { Slot::None };
        let ao = if read("aoMap") { self.slot(m, "aoMap", max_dim)? } else { Slot::None };
        let normal = if read("normalMap") { self.slot(m, "normalMap", max_dim)? } else { Slot::None };
        let detail = if known && binds("detailMap") { self.slot(m, "detailMap", max_dim)? } else { Slot::None };
        let detail_scale = m.constant::<2>("detailScale").unwrap_or([1.0; 2]);
        let detail_height = m.constant::<1>("detailScaleHeight").map_or(1.0, |v| v[0]);
        // (A detail map tiled finer than eight times averages out at this
        // size: left out.)
        let detail = match detail {
            Slot::Image(d) if detail_scale.iter().all(|s| s.abs() <= 8.0 && s.abs() > 0.0) => Some(d),
            _ => None,
        };

        // The normal map: the slope, and its variance per texel.
        let (nw, nh) = match (&normal, &colour) {
            (Slot::Image(n), _) => (n.w, n.h),
            (_, Some(c)) => (c.w, c.h),
            _ => (64, 64),
        };
        let mut variance = vec![0f32; (nw * nh) as usize];
        if let Slot::Image(n) = &normal {
            let mut rgba = vec![0u8; (nw * nh * 4) as usize];
            for y in 0..nh {
                for x in 0..nw {
                    let i = (y * nw + x) as usize;
                    let p = &n.px[i * 4..i * 4 + 4];
                    let s = |b: u8| ((f32::from(b) / 255.0 - 0.5) * normal_height + 0.5) * 1.992_188 - 1.0;
                    let (mut nx, mut ny) = (s(p[0]), s(p[1]));
                    let b = f32::from(p[2]) / 255.0 * normal_height;
                    let mut v = (b * b / 3.0).min(1.0);
                    if let Some(d) = &detail {
                        let (u, w) = ((x as f32 + 0.5) / nw as f32 * detail_scale[0], (y as f32 + 0.5) / nh as f32 * detail_scale[1]);
                        nx += (d.sample(u, w, 0) * 1.992_188 - 1.0) * detail_height;
                        ny += (d.sample(u, w, 1) * 1.992_188 - 1.0) * detail_height;
                        let db = d.sample(u, w, 2);
                        v += (db * db / 3.0).min(1.0) * detail_height;
                    }
                    variance[i] = v;
                    let z = (1.0 - nx * nx - ny * ny).max(0.0).sqrt().max(0.25);
                    let q = |t: f32| (((t / z).clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0).round() as u8;
                    rgba[i * 4..i * 4 + 4].copy_from_slice(&[q(nx), q(ny), 0, 255]);
                }
            }
            out.normal = encode(nw, nh, rgba, Format::Bc5);
            out.used[4] = detail.is_some();
        }

        // The specular map: reflectance and gloss.
        let (sw, sh) = {
            let mut d = match (&gloss, &metal) {
                (Slot::Image(g), _) => (g.w, g.h),
                (_, Slot::Image(s)) => (s.w, s.h),
                _ => (nw, nh),
            };
            while d.0.max(d.1) > spec_max && d.0 > 4 && d.1 > 4 {
                d = (d.0 / 2, d.1 / 2);
            }
            d
        };
        let read_slot = |s: &Slot, x: u32, y: u32, w: u32, h: u32, d: f32| match s {
            Slot::Image(i) => i.at(x, y, w, h, 0),
            Slot::Solid(c) => f32::from(c[0]) / 255.0,
            Slot::None => d,
        };
        let has_metal = !matches!(metal, Slot::None);
        let mut spec = vec![0u8; (sw * sh * 4) as usize];
        for y in 0..sh {
            for x in 0..sw {
                let o = ((y * sw + x) * 4) as usize;
                let mm = read_slot(&metal, x, y, sw, sh, 0.0);
                for k in 0..3 {
                    let f0 = 0.04 + mm * (spec_tint[k] - 0.04);
                    spec[o + k] = (f0.max(0.0).sqrt() * 255.0).round() as u8;
                }
                let g = match (gloss_range, &gloss) {
                    (Some(r), Slot::None) => r[1] / 17.0,
                    (Some(r), s) => (r[0] + (r[1] - r[0]) * read_slot(s, x, y, sw, sh, 1.0)) / 17.0,
                    // (No numbers: the middle of Black Ops III's range.)
                    (None, s) => read_slot(s, x, y, sw, sh, 0.5) * 0.6 + 0.2,
                };
                let (vx, vy) = ((x * nw / sw).min(nw - 1), (y * nh / sh).min(nh - 1));
                let g = gloss_less_variance(g.clamp(0.0, 1.0), variance[(vy * nw + vx) as usize]);
                spec[o + 3] = (bo2_gloss(g) * 255.0).round() as u8;
            }
        }
        out.specular = encode(sw, sh, spec, Format::Bc3);
        out.used[1] = has_metal;
        out.used[2] = matches!(gloss, Slot::Image(_));

        // The colour map: tinted, the metal's diffuse and the occlusion
        // taken out.
        if let Some(c) = colour {
            let tinted = tint.iter().any(|t| (t - 1.0).abs() > 0.004);
            let alpha_is_tint = cover == Cover::Opaque && !(known && binds("tintMask"));
            let (w, h) = (c.w, c.h);
            let mut rgba = c.px.clone();
            let mut any_alpha = false;
            // A reveal decal (`gbuffer_decal` with `revealMap`) covers by its
            // reveal map at `uv * revealScale` against a threshold its vertex
            // alpha sets (`alphaRevealSoftEdge`, `alphaRevealRamp`), not by
            // its colour's alpha: here the reveal map, as at full vertex
            // alpha less a soft edge.
            let reveal = match (cover, known && binds("revealMap")) {
                (Cover::Alpha, true) => match self.slot(m, "revealMap", max_dim)? {
                    Slot::Image(r) => Some(r),
                    _ => None,
                },
                _ => None,
            };
            let reveal_scale = m.constant::<2>("revealScale").filter(|s| s.iter().all(|v| *v > 0.0)).unwrap_or([1.0; 2]);
            let soft = m.constant::<1>("alphaRevealSoftEdge").map_or(0.1, |v| v[0]).clamp(0.02, 1.0);
            for y in 0..h {
                for x in 0..w {
                    let o = ((y * w + x) * 4) as usize;
                    let mask = if alpha_is_tint { f32::from(c.px[o + 3]) / 255.0 } else { read_slot(&tint_mask, x, y, w, h, 1.0) };
                    let keep = 1.0 - read_slot(&metal, x, y, w, h, 0.0);
                    let occ = read_slot(&ao, x, y, w, h, 1.0);
                    for k in 0..3 {
                        let t = if tinted { 1.0 + mask * (tint[k] - 1.0) } else { 1.0 };
                        rgba[o + k] = to_srgb(to_linear(c.px[o + k]) * t * keep * occ);
                    }
                    if cover == Cover::Opaque {
                        rgba[o + 3] = 255;
                    } else if let Some(r) = &reveal {
                        let (u, v) = ((x as f32 + 0.5) / w as f32 * reveal_scale[0], (y as f32 + 0.5) / h as f32 * reveal_scale[1]);
                        let a = ((r.sample(u, v, 0) - 0.5 + soft) / (2.0 * soft)).clamp(0.0, 1.0);
                        rgba[o + 3] = (a * 255.0).round() as u8;
                    }
                    any_alpha |= rgba[o + 3] < 250;
                }
            }
            out.used[0] = tinted;
            out.used[3] = matches!(ao, Slot::Image(_));
            out.colour = encode(w, h, rgba, if any_alpha { Format::Bc3 } else { Format::Bc1 });
        }
        Ok(out)
    }
}

impl T7Map {
    /// Clear glass (`glass_nocolor`: windows, tanks, condensation): Black
    /// Ops III's forward glass program (`lit_glass_nocolor`) colours it
    /// `colorTint` and covers `(1 - g)^3.4 * (1 - a0) + a0` of what is
    /// behind, `g` its gloss (`glossMap` over `glossRange`, less its normal
    /// map's variance), `a0 = tintAmount * 0.92 + 0.08`: smooth glass shows
    /// through, rough or dirty glass less so. Made here as a colour map with
    /// that coverage in alpha, and the specular map (glass reflects 0.04).
    fn clear_glass(&self, m: &MapMaterial, max_dim: u32, spec_max: u32) -> Result<Look, String> {
        let mut out = self.look_inner(m, max_dim, spec_max, Cover::Alpha)?;
        let tint = m.constant::<3>("colorTint").unwrap_or([1.0; 3]);
        let a0 = m.constant::<1>("tintAmount").map_or(0.0, |v| v[0]) * 0.92 + 0.08;
        let range = m.constant::<2>("glossRange").unwrap_or([6.0, 17.0]);
        let gloss = match self.slot(m, "glossMap", spec_max)? {
            Slot::Image(i) => i,
            Slot::Solid(c) => Rgba { w: 4, h: 4, px: c.repeat(16) },
            Slot::None => Rgba { w: 4, h: 4, px: [255u8; 4].repeat(16) },
        };
        let (w, h) = (gloss.w.max(4), gloss.h.max(4));
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let o = ((y * w + x) * 4) as usize;
                let g = ((range[0] + (range[1] - range[0]) * gloss.at(x, y, w, h, 0)) / 17.0).clamp(0.0, 1.0);
                let a = (1.0 - g).powf(3.4) * (1.0 - a0) + a0;
                for k in 0..3 {
                    rgba[o + k] = to_srgb(tint[k]);
                }
                rgba[o + 3] = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
        out.colour = encode(w, h, rgba, Format::Bc3);
        out.used[0] = true;
        Ok(out)
    }
}

/// One tiled layer of a micro tile blend.
struct Layer {
    colour: Option<Rgba>,
    tint: [f32; 3],
    normal: Option<Rgba>,
    height: f32,
    gloss: Slot,
    gloss_range: [f32; 2],
    scale: [f32; 2],
    /// cos, sin of the turn.
    turn: [f32; 2],
}

impl Layer {
    fn uv(&self, u: f32, v: f32) -> (f32, f32) {
        let (du, dv) = (u - 0.5, v - 0.5);
        let (c, s) = (self.turn[0], self.turn[1]);
        ((du * c - dv * s + 0.5) * self.scale[0], (du * s + dv * c + 0.5) * self.scale[1])
    }
}

impl T7Map {
    /// A tile picture shrunk to the texels a tile covers in a `w`-wide
    /// picture repeating it `scale` times.
    fn tile(&self, m: &MapMaterial, slot: &str, w: u32, scale: f32, max_dim: u32) -> Result<Slot, String> {
        let cover = ((w as f32 / scale.abs().max(1.0)).ceil() as u32).next_power_of_two().clamp(4, max_dim);
        Ok(match self.slot(m, slot, max_dim)? {
            Slot::Image(i) => Slot::Image(i.at_most(cover)),
            // (A placeholder is that colour all over: `$white_diffuse` is a
            // layer drawn by its tint alone.)
            Slot::Solid(c) => Slot::Image(Rgba { w: 4, h: 4, px: c.repeat(16) }),
            Slot::None => Slot::None,
        })
    }

    fn micro_tile_blend(&self, m: &MapMaterial, max_dim: u32, spec_max: u32) -> Result<Look, String> {
        let mut out = Look::default();
        let c = |name: &str| m.constants.get(name).cloned().unwrap_or_default();
        let at = |v: &Vec<f32>, k: usize, d: f32| v.get(k).copied().unwrap_or(d);
        let (s1, s2, mix_scale) = (c("detailScale1"), c("detailScale2"), c("mixScale"));
        let (t0, t1) = (c("colorTint"), c("colorTint1"));
        let (g1, g2) = (c("glossRange"), c("glossRange2"));
        let (r1, r2) = (c("rotateUVs1"), c("rotateUVs2"));
        let heights = [at(&c("detailScaleHeight1"), 0, 1.0), at(&c("detailScaleHeight2"), 0, 1.0)];
        let base_height = at(&c("baseNormalHeight"), 0, 1.0);
        // The picture: the macro maps' size, at least 512 (the tiles need
        // room), at most `max_dim`.
        let normal = match self.slot(m, "normalMap", max_dim)? {
            Slot::Image(i) => Some(i),
            _ => None,
        };
        let mix = match self.slot(m, "mixMap", max_dim)? {
            Slot::Image(i) => Some(i),
            _ => None,
        };
        let (mut w, mut h) = normal.as_ref().or(mix.as_ref()).map_or((max_dim, max_dim), |i| (i.w, i.h));
        while w.max(h) < 512.min(max_dim) {
            (w, h) = (w * 2, h * 2);
        }
        while w.max(h) > max_dim && w > 4 && h > 4 {
            (w, h) = (w / 2, h / 2);
        }
        let layer = |k: usize| -> Result<Layer, String> {
            let (scale, tint, gr, rot) = if k == 0 { (&s1, &t0, &g1, &r1) } else { (&s2, &t1, &g2, &r2) };
            let scale = [at(scale, 0, 1.0), at(scale, 1, 1.0)];
            let (cs, ns, gs) = if k == 0 { ("colorMap", "detailMap1", "glossMap") } else { ("colorMapDetail2", "detailMap2", "glossMapDetail2") };
            let img = |s: Slot| match s {
                Slot::Image(i) => Some(i),
                _ => None,
            };
            let a = at(rot, 0, 0.0);
            Ok(Layer {
                colour: img(self.tile(m, cs, w, scale[0], max_dim)?),
                tint: [at(tint, 0, 1.0), at(tint, 1, 1.0), at(tint, 2, 1.0)],
                normal: img(self.tile(m, ns, w, scale[0], max_dim)?),
                height: heights[k],
                gloss: self.tile(m, gs, w, scale[0], max_dim)?,
                gloss_range: [at(gr, 0, 4.0), at(gr, 1, 10.0)],
                scale,
                turn: [a.cos(), a.sin()],
            })
        };
        let layers = [layer(0)?, layer(1)?];
        if layers.iter().all(|l| l.colour.is_none()) {
            return Err(format!("{}: no tile colour maps", m.name));
        }
        let ao = self.slot(m, "aoMap", max_dim)?;
        let mix_scale = [at(&mix_scale, 0, 1.0), at(&mix_scale, 1, 1.0)];
        let (mut colour, mut nrm, mut gloss) = (vec![0u8; (w * h * 4) as usize], vec![0u8; (w * h * 4) as usize], vec![0f32; (w * h) as usize]);
        // One row of the bake (rows spread over the cores: a map has some
        // hundreds of these).
        let row = |y: u32, colour: &mut [u8], nrm: &mut [u8], gloss: &mut [f32]| {
            for x in 0..w {
                let (u, v) = ((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32);
                let t = mix.as_ref().map_or(0.0, |i| i.sample(u * mix_scale[0], v * mix_scale[1], 0));
                let mut rgb = [[0f32; 3]; 2];
                let mut n = [[0f32; 2]; 2];
                let mut g = [0f32; 2];
                for (k, l) in layers.iter().enumerate() {
                    let (lu, lv) = l.uv(u, v);
                    if let Some(ci) = &l.colour {
                        let a = ci.sample(lu, lv, 3);
                        for ch in 0..3 {
                            let lin = to_linear_f(ci.sample(lu, lv, ch));
                            rgb[k][ch] = lin * (1.0 + a * (l.tint[ch] - 1.0));
                        }
                    }
                    if let Some(ni) = &l.normal {
                        n[k] = [(ni.sample(lu, lv, 0) * 1.992_188 - 1.0) * l.height, (ni.sample(lu, lv, 1) * 1.992_188 - 1.0) * l.height];
                    }
                    let gm = match &l.gloss {
                        Slot::Image(i) => i.sample(lu, lv, 0),
                        Slot::Solid(c) => f32::from(c[0]) / 255.0,
                        Slot::None => 1.0,
                    };
                    g[k] = (l.gloss_range[0] + (l.gloss_range[1] - l.gloss_range[0]) * gm) / 17.0;
                }
                // (A layer with no picture of its own takes the other's.)
                if layers[0].colour.is_none() {
                    rgb[0] = rgb[1];
                }
                if layers[1].colour.is_none() {
                    rgb[1] = rgb[0];
                }
                let occ = match &ao {
                    Slot::Image(i) => i.sample(u, v, 0),
                    _ => 1.0,
                };
                let i = x as usize;
                for ch in 0..3 {
                    colour[i * 4 + ch] = to_srgb((rgb[0][ch] + t * (rgb[1][ch] - rgb[0][ch])) * occ);
                }
                colour[i * 4 + 3] = 255;
                let (mut bx, mut by) = (0.0, 0.0);
                if let Some(ni) = &normal {
                    let s = |c: usize| ((ni.sample(u, v, c) - 0.5) * base_height + 0.5) * 1.992_188 - 1.0;
                    (bx, by) = (s(0), s(1));
                }
                let nx = bx + n[0][0] + t * (n[1][0] - n[0][0]);
                let ny = by + n[0][1] + t * (n[1][1] - n[0][1]);
                let z = (1.0 - nx * nx - ny * ny).max(0.0).sqrt().max(0.25);
                let q = |s: f32| (((s / z).clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0).round() as u8;
                nrm[i * 4..i * 4 + 4].copy_from_slice(&[q(nx), q(ny), 0, 255]);
                gloss[i] = (g[0] + t * (g[1] - g[0])).clamp(0.0, 1.0);
            }
        };
        let threads = std::thread::available_parallelism().map_or(4, |n| usize::from(n) / 2).clamp(1, 8);
        let rows_each = (h as usize).div_ceil(threads).max(1);
        let wu = w as usize;
        std::thread::scope(|sc| {
            for (((k, c), n), g) in
                colour.chunks_mut(rows_each * wu * 4).enumerate().zip(nrm.chunks_mut(rows_each * wu * 4)).zip(gloss.chunks_mut(rows_each * wu))
            {
                let row = &row;
                sc.spawn(move || {
                    for (r, ((c, n), g)) in c.chunks_mut(wu * 4).zip(n.chunks_mut(wu * 4)).zip(g.chunks_mut(wu)).enumerate() {
                        row((k * rows_each + r) as u32, c, n, g);
                    }
                });
            }
        });
        out.colour = encode(w, h, colour, Format::Bc1);
        out.normal = encode(w, h, nrm, Format::Bc5);
        // The specular map: a dielectric's 0.04, the gloss.
        let mut sw = w;
        let mut sh = h;
        let mut step = 1;
        while sw.max(sh) > spec_max && sw > 4 && sh > 4 {
            (sw, sh, step) = (sw / 2, sh / 2, step * 2);
        }
        let mut spec = vec![0u8; (sw * sh * 4) as usize];
        let f0 = (0.04f32.sqrt() * 255.0).round() as u8;
        for y in 0..sh {
            for x in 0..sw {
                let o = ((y * sw + x) * 4) as usize;
                spec[o..o + 3].fill(f0);
                spec[o + 3] = (bo2_gloss(gloss[((y * step) * w + x * step) as usize]) * 255.0).round() as u8;
            }
        }
        out.specular = encode(sw, sh, spec, Format::Bc3);
        out.used = [layers.iter().any(|l| l.tint.iter().any(|t| (t - 1.0).abs() > 0.004)), false, layers.iter().any(|l| matches!(l.gloss, Slot::Image(_))), matches!(ao, Slot::Image(_)), layers.iter().any(|l| l.normal.is_some())];
        Ok(out)
    }
}

/// A 2x box filter down (odd sizes rounded down).
pub fn half_level(w: u32, h: u32, px: &[u8]) -> (u32, u32, Vec<u8>) {
    let (ow, oh) = ((w / 2).max(1), (h / 2).max(1));
    let mut out = vec![0u8; (ow * oh * 4) as usize];
    for y in 0..oh {
        for x in 0..ow {
            for c in 0..4 {
                let mut sum = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((2 * x + dx).min(w - 1), (2 * y + dy).min(h - 1));
                    sum += u32::from(px[((sy * w + sx) * 4 + c) as usize]);
                }
                out[((y * ow + x) * 4 + c) as usize] = ((sum + 2) / 4) as u8;
            }
        }
    }
    (ow, oh, out)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Bc1,
    Bc3,
    Bc5,
}

/// RGBA8 into a block format with its full mip chain (`None` when not
/// whole blocks).
fn encode(w: u32, h: u32, rgba: Vec<u8>, format: Format) -> Option<ImageLevels> {
    if w == 0 || h == 0 || !w.is_multiple_of(4) || !h.is_multiple_of(4) {
        return None;
    }
    let mut data = Vec::new();
    let (mut lw, mut lh, mut px) = (w, h, rgba);
    let mut levels = 0;
    loop {
        encode_level(lw, lh, &px, format, &mut data);
        levels += 1;
        if lw == 1 && lh == 1 {
            break;
        }
        (lw, lh, px) = half_level(lw, lh, &px);
    }
    let dxgi = match format {
        Format::Bc1 => 72,
        Format::Bc3 => 78,
        Format::Bc5 => 83,
    };
    Some(ImageLevels { format: dxgi, width: w, height: h, levels, data, part: None })
}

fn encode_level(w: u32, h: u32, px: &[u8], format: Format, out: &mut Vec<u8>) {
    let (w, h) = (w as usize, h as usize);
    for by in 0..h.div_ceil(4) {
        for bx in 0..w.div_ceil(4) {
            let block: [[u8; 4]; 16] = std::array::from_fn(|t| {
                let (x, y) = ((bx * 4 + t % 4).min(w - 1), (by * 4 + t / 4).min(h - 1));
                let o = (y * w + x) * 4;
                [px[o], px[o + 1], px[o + 2], px[o + 3]]
            });
            match format {
                Format::Bc1 => out.extend_from_slice(&bc1_block(&block)),
                Format::Bc3 => {
                    out.extend_from_slice(&texture::encode_bc4_block(&block.map(|p| p[3])));
                    out.extend_from_slice(&bc1_block(&block));
                }
                Format::Bc5 => {
                    out.extend_from_slice(&texture::encode_bc4_block(&block.map(|p| p[0])));
                    out.extend_from_slice(&texture::encode_bc4_block(&block.map(|p| p[1])));
                }
            }
        }
    }
}

fn rgb565(c: [f32; 3]) -> u16 {
    let q = |v: f32, max: f32| (v.clamp(0.0, 255.0) / 255.0 * max).round() as u16;
    q(c[0], 31.0) << 11 | q(c[1], 63.0) << 5 | q(c[2], 31.0)
}

fn unpack565(c: u16) -> [f32; 3] {
    let (r, g, b) = ((c >> 11) & 31, (c >> 5) & 63, c & 31);
    [f32::from(r) * 255.0 / 31.0, f32::from(g) * 255.0 / 63.0, f32::from(b) * 255.0 / 31.0]
}

/// One BC1 block in its four-colour mode: the ends of the block's colours
/// along their widest spread from the mean.
pub fn bc1_block(px: &[[u8; 4]; 16]) -> [u8; 8] {
    let c: [[f32; 3]; 16] = px.map(|p| [f32::from(p[0]), f32::from(p[1]), f32::from(p[2])]);
    let mean: [f32; 3] = std::array::from_fn(|k| c.iter().map(|v| v[k]).sum::<f32>() / 16.0);
    let d2 = |v: &[f32; 3], m: &[f32; 3]| (0..3).map(|k| (v[k] - m[k]).powi(2)).sum::<f32>();
    let far = c.iter().copied().max_by(|a, b| d2(a, &mean).total_cmp(&d2(b, &mean))).unwrap_or(mean);
    let axis: [f32; 3] = std::array::from_fn(|k| far[k] - mean[k]);
    let len2 = axis.iter().map(|a| a * a).sum::<f32>();
    let (mut tmin, mut tmax) = (0.0f32, 0.0f32);
    if len2 > 0.0 {
        for v in &c {
            let t = (0..3).map(|k| (v[k] - mean[k]) * axis[k]).sum::<f32>() / len2;
            tmin = tmin.min(t);
            tmax = tmax.max(t);
        }
    }
    let end = |t: f32| std::array::from_fn::<f32, 3, _>(|k| mean[k] + axis[k] * t);
    let (mut c0, mut c1) = (rgb565(end(tmax)), rgb565(end(tmin)));
    if c0 < c1 {
        std::mem::swap(&mut c0, &mut c1);
    }
    let mut out = [0u8; 8];
    out[0..2].copy_from_slice(&c0.to_le_bytes());
    out[2..4].copy_from_slice(&c1.to_le_bytes());
    if c0 == c1 {
        return out;
    }
    let (a, b) = (unpack565(c0), unpack565(c1));
    let pal: [[f32; 3]; 4] = [a, b, std::array::from_fn(|k| (2.0 * a[k] + b[k]) / 3.0), std::array::from_fn(|k| (a[k] + 2.0 * b[k]) / 3.0)];
    let mut bits = 0u32;
    for (t, v) in c.iter().enumerate() {
        let best = (0..4).min_by(|&i, &j| d2(&pal[i], v).total_cmp(&d2(&pal[j], v))).unwrap_or(0) as u32;
        bits |= best << (2 * t);
    }
    out[4..8].copy_from_slice(&bits.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gloss_matches_the_highlight_power() {
        // 2^(17 g) + 2 == 2^(13 g') below the cap.
        for g in [0.1f32, 0.3, 0.5, 0.7] {
            let b = bo2_gloss(g);
            assert!(((13.0 * b).exp2() - ((17.0 * g).exp2() + 2.0)).abs() < 0.01 * (17.0 * g).exp2(), "{g} -> {b}");
        }
        assert_eq!(bo2_gloss(1.0), 1.0);
        // Variance only lowers it.
        assert!(gloss_less_variance(0.8, 0.1) < 0.8);
        assert!((gloss_less_variance(0.8, 0.0) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn placeholders_and_srgb() {
        assert_eq!(placeholder(",$white_gloss"), Some([255; 4]));
        assert_eq!(placeholder("$black_color"), Some([0, 0, 0, 255]));
        assert_eq!(placeholder("$identitynormalmap"), Some([128, 128, 0, 255]));
        assert_eq!(placeholder("i_t7_concrete_c"), None);
        for b in [0u8, 1, 50, 128, 200, 255] {
            assert_eq!(to_srgb(to_linear(b)), b);
        }
    }

    #[test]
    fn whole_mip_chains() {
        let l = encode(8, 4, vec![200; 8 * 4 * 4], Format::Bc1).unwrap();
        assert_eq!(l.levels, 4);
        assert_eq!(l.data.len(), 8 * 2 + 8 + 8 + 8);
        assert!(encode(6, 4, vec![0; 6 * 4 * 4], Format::Bc5).is_none());
    }
}
