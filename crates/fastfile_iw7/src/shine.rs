//! A material's normal and specular maps in Black Ops II's form, and its
//! colour map with the metal's diffuse taken out.
//!
//! IW7's lit world and model shaders (their `lmap_specenv_*_ng0[0]_*`
//! pixel shaders, read with `fxc /dumpbin` on retail mp_flip) read:
//!
//! * the colour map (`*_packed_cs`, BC7): colour in RGB (squared into
//!   linear light), the specular level in alpha: metalness
//!   `saturate(a - 0.1) / 0.9`, the dielectric reflectance `min(a, 0.1)`;
//!   diffuse `colour * (1 - metalness)`, specular colour
//!   `metalness * colour + min(a, 0.1)`;
//! * the normal map (`*_packed_ng`, `*_packed_nog`, BC7): the tangent-space
//!   normal turned 45 degrees in green and alpha (`g = G * 2 - 1`,
//!   `a = A * 2 - 1`; `x = (g + a) / 2`, `y = (g - a) / 2`,
//!   `z = max(1 - |x| - |y|, 0)`, normalised), and in red the gloss
//!   (`gloss = 1 - sqrt(1 - R)`, the highlight's power `2^(20 gloss) + 2`).
//!   A plain normal map (`*_n`, borrowed Black Ops III pictures) holds x and
//!   y in red and green. Older materials (a plain `*_c` colour map) carry
//!   their own specular map (`*_sg`: colour, gloss in alpha), kept as is;
//!   `$`-named pictures are engine placeholders, left out.
//!
//! Black Ops II's lit shaders read a normal map's red and green as the
//! slope added to the vertex normal and a specular map as the specular
//! colour (sRGB) with the gloss in alpha (power `2^(13 gloss)`). The
//! pictures made here: the normal map as BC5 (unsigned x and y over -1..1,
//! the form Ghosts' are, which the assets lane turns into Black Ops II's),
//! the specular map as BC3 (sRGB colour, gloss in alpha), the colour map
//! (only when it has metal) as BC1 (sRGB). Each with its full mip chain.

use crate::map::{Iw7Map, MapMaterial};
use crate::texture::{self, ImageLevels};

/// What `Iw7Map::shine` makes for a material.
#[derive(Clone, Debug, Default)]
pub struct Shine {
    /// BC5 (`DXGI_FORMAT_BC5_UNORM`, 83).
    pub normal: Option<ImageLevels>,
    /// BC3 sRGB (`DXGI_FORMAT_BC3_UNORM_SRGB`, 78).
    pub specular: Option<ImageLevels>,
    /// BC1 sRGB (`DXGI_FORMAT_BC1_UNORM_SRGB`, 72): the colour map with the
    /// metal's diffuse taken out, when some of it is metal.
    pub colour: Option<ImageLevels>,
}

/// The gloss a material without a gloss map gets (Black Ops II's 0..1).
const DEFAULT_GLOSS: f32 = 0.35;

/// How a normal map holds its normal.
fn packed_normal(name: &str) -> bool {
    name.contains("_packed_n")
}

/// BO2's gloss (power `2^(13 g)`) from IW7's normal map red (power
/// `2^(20 (1 - sqrt(1 - r))) + 2`): the same power, capped at BO2's.
fn gloss_of(red: u8) -> f32 {
    let g = 1.0 - (1.0 - f32::from(red) / 255.0).max(0.0).sqrt();
    (((20.0 * g).exp2() + 2.0).log2() / 13.0).min(1.0)
}

/// A texel's tangent-space normal.
fn normal_of(px: &[u8], packed: bool) -> [f32; 3] {
    let s = |b: u8| f32::from(b) / 255.0 * 2.0 - 1.0;
    let (x, y, z) = if packed {
        let (g, a) = (s(px[1]), s(px[3]));
        let (x, y) = ((g + a) * 0.5, (g - a) * 0.5);
        (x, y, (1.0 - x.abs() - y.abs()).max(0.0))
    } else {
        let (x, y) = (s(px[0]), s(px[1]));
        (x, y, (1.0 - x * x - y * y).max(0.0).sqrt())
    };
    let l = (x * x + y * y + z * z).sqrt().max(1e-6);
    [x / l, y / l, z / l]
}

impl Iw7Map {
    /// A material's normal and specular maps in Black Ops II's form, and its
    /// colour map without the metal's diffuse when `colour` is asked for
    /// (opaque materials; a cut-out's colour keeps its alpha), at most
    /// `max_dim` on a side (the specular map at most `spec_max`).
    pub fn shine(&self, m: &MapMaterial, max_dim: u32, spec_max: u32, colour: bool) -> Result<Shine, String> {
        let mut out = Shine::default();
        // (`$identitynormalmap` and the like: engine placeholders, flat.)
        let normal_name = m.texture("normalMap").filter(|n| !n.starts_with('$'));
        let normal = match normal_name.map(|n| (n, self.image(n, max_dim))) {
            Some((n, Ok(l))) => Some((n, texture::level_rgba(&l, 0).ok_or_else(|| format!("{n}: format {} not decoded", l.format))?)),
            Some((_, Err(e))) => return Err(e),
            None => None,
        };
        let packed = normal_name.is_some_and(packed_normal);
        if let Some((_, (w, h, px))) = &normal {
            let mut rgba = vec![0u8; px.len()];
            for (o, p) in rgba.chunks_exact_mut(4).zip(px.chunks_exact(4)) {
                let [x, y, z] = normal_of(p, packed);
                // The slope Black Ops II adds (x / z, y / z), over -1..1.
                let z = z.max(0.5);
                let b = |v: f32| (((v / z).clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0).round() as u8;
                o.copy_from_slice(&[b(x), b(y), 0, 255]);
            }
            out.normal = encode(*w, *h, rgba, Format::Bc5);
        }
        // The colour map: specular colour and metal.
        let colour_name = m.texture("colorMap");
        let colour_px = match colour_name.map(|n| (n, self.image(n, max_dim))) {
            Some((n, Ok(l))) => Some(texture::level_rgba(&l, 0).ok_or_else(|| format!("{n}: format {} not decoded", l.format))?),
            Some((_, Err(e))) => return Err(e),
            None => None,
        };
        // Its alpha is the specular level only on the packed colour maps.
        let level_in_alpha = colour_name.is_some_and(|n| n.contains("_packed_c"));
        // An older material's own specular map (`*_sg`, BC3: specular
        // colour, gloss in alpha, as Ghosts' and Black Ops II's), taken as
        // it is.
        let spec_map = m.texture("specularMap").filter(|n| !n.starts_with('$'));
        if let (false, Some(sname)) = (level_in_alpha, spec_map) {
            let l = self.image(sname, spec_max)?;
            let (sw, sh, spx) = texture::level_rgba(&l, 0).ok_or_else(|| format!("{sname}: format {} not decoded", l.format))?;
            out.specular = encode(sw, sh, spx, Format::Bc3);
            return Ok(out);
        }
        if let Some((cw, ch, cpx)) = &colour_px {
            // At most `spec_max`: a box filter down from the colour map.
            let (mut sw, mut sh, mut spx) = (*cw, *ch, cpx.clone());
            while sw.max(sh) > spec_max && sw > 4 && sh > 4 {
                (sw, sh, spx) = crate::look::half_level(sw, sh, &spx);
            }
            let mut spec = vec![0u8; spx.len()];
            for y in 0..sh {
                for x in 0..sw {
                    let o = ((y * sw + x) * 4) as usize;
                    let a = if level_in_alpha { f32::from(spx[o + 3]) / 255.0 } else { 0.04 };
                    let metal = ((a - 0.1).max(0.0) / 0.9).min(1.0);
                    let f0 = a.min(0.1);
                    let gloss = match &normal {
                        Some((_, (nw, nh, npx))) if packed => {
                            let (nx, ny) = ((x * nw / sw).min(nw - 1), (y * nh / sh).min(nh - 1));
                            gloss_of(npx[((ny * nw + nx) * 4) as usize])
                        }
                        _ => DEFAULT_GLOSS,
                    };
                    for k in 0..3 {
                        let c = f32::from(spx[o + k]) / 255.0;
                        // Linear specular colour; stored as its square root
                        // (Black Ops II squares what it reads).
                        let lin = metal * c * c + f0;
                        spec[o + k] = (lin.sqrt() * 255.0).round().clamp(0.0, 255.0) as u8;
                    }
                    spec[o + 3] = (gloss * 255.0).round() as u8;
                }
            }
            out.specular = encode(sw, sh, spec, Format::Bc3);
            // The diffuse: colour times (1 - metalness), stored as its
            // square root's factor (the colour is squared when read).
            if colour && level_in_alpha && cpx.chunks_exact(4).any(|p| p[3] > 28) {
                let mut rgba = cpx.clone();
                for p in rgba.chunks_exact_mut(4) {
                    let a = f32::from(p[3]) / 255.0;
                    let keep = (1.0 - ((a - 0.1).max(0.0) / 0.9).min(1.0)).sqrt();
                    for c in &mut p[..3] {
                        *c = (f32::from(*c) * keep).round() as u8;
                    }
                    p[3] = 255;
                }
                out.colour = encode(*cw, *ch, rgba, Format::Bc1);
            }
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Bc1,
    Bc3,
    Bc5,
}

/// RGBA8 `w` x `h` into a block format with its full mip chain (`None`
/// when the size is not whole blocks).
fn encode(w: u32, h: u32, rgba: Vec<u8>, format: Format) -> Option<ImageLevels> {
    if !w.is_multiple_of(4) || !h.is_multiple_of(4) || w == 0 || h == 0 {
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
        (lw, lh, px) = crate::look::half_level(lw, lh, &px);
    }
    let dxgi = match format {
        Format::Bc1 => 72,
        Format::Bc3 => 78,
        Format::Bc5 => 83,
    };
    Some(ImageLevels { format: dxgi, width: w, height: h, levels, data, stream: None })
}

fn encode_level(w: u32, h: u32, px: &[u8], format: Format, out: &mut Vec<u8>) {
    let (w, h) = (w as usize, h as usize);
    for by in 0..h.div_ceil(4) {
        for bx in 0..w.div_ceil(4) {
            let texel = |t: usize| {
                let (x, y) = ((bx * 4 + t % 4).min(w - 1), (by * 4 + t / 4).min(h - 1));
                let o = (y * w + x) * 4;
                [px[o], px[o + 1], px[o + 2], px[o + 3]]
            };
            let block: [[u8; 4]; 16] = std::array::from_fn(texel);
            match format {
                Format::Bc1 => out.extend_from_slice(&bc1_block(&block)),
                Format::Bc3 => {
                    out.extend_from_slice(&bc4_block(&block.map(|p| p[3])));
                    out.extend_from_slice(&bc1_block(&block));
                }
                Format::Bc5 => {
                    out.extend_from_slice(&bc4_block(&block.map(|p| p[0])));
                    out.extend_from_slice(&bc4_block(&block.map(|p| p[1])));
                }
            }
        }
    }
}

/// One BC4 block (eight values from the block's extremes).
pub fn bc4_block(v: &[u8; 16]) -> [u8; 8] {
    let (lo, hi) = (*v.iter().min().unwrap(), *v.iter().max().unwrap());
    let mut out = [0u8; 8];
    out[0] = hi;
    out[1] = lo;
    if hi == lo {
        return out;
    }
    let pal: [i32; 8] = std::array::from_fn(|k| match k {
        0 => i32::from(hi),
        1 => i32::from(lo),
        k => (i32::from(hi) * (8 - k as i32) + i32::from(lo) * (k as i32 - 1)) / 7,
    });
    let mut bits = 0u64;
    for (t, &x) in v.iter().enumerate() {
        let best = (0..8).min_by_key(|&k| (pal[k] - i32::from(x)).abs()).unwrap_or(0) as u64;
        bits |= best << (3 * t);
    }
    out[2..8].copy_from_slice(&bits.to_le_bytes()[..6]);
    out
}

fn rgb565(c: [f32; 3]) -> u16 {
    let q = |v: f32, max: f32| (v.clamp(0.0, 255.0) / 255.0 * max).round() as u16;
    q(c[0], 31.0) << 11 | q(c[1], 63.0) << 5 | q(c[2], 31.0)
}

fn unpack565(c: u16) -> [f32; 3] {
    let (r, g, b) = ((c >> 11) & 31, (c >> 5) & 63, c & 31);
    [f32::from(r) * 255.0 / 31.0, f32::from(g) * 255.0 / 63.0, f32::from(b) * 255.0 / 31.0]
}

/// One BC1 block (four colours, the ends of the block's colours along
/// their widest spread), its colour in RGB.
pub fn bc1_block(px: &[[u8; 4]; 16]) -> [u8; 8] {
    let c: Vec<[f32; 3]> = px.iter().map(|p| [f32::from(p[0]), f32::from(p[1]), f32::from(p[2])]).collect();
    let mean: [f32; 3] = std::array::from_fn(|k| c.iter().map(|v| v[k]).sum::<f32>() / 16.0);
    // The widest spread: the axis from the mean to the farthest colour.
    let far = c
        .iter()
        .max_by(|a, b| {
            let d = |v: &[f32; 3]| (0..3).map(|k| (v[k] - mean[k]).powi(2)).sum::<f32>();
            d(a).total_cmp(&d(b))
        })
        .copied()
        .unwrap_or(mean);
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
    let pal: [[f32; 3]; 4] = [
        a,
        b,
        std::array::from_fn(|k| (2.0 * a[k] + b[k]) / 3.0),
        std::array::from_fn(|k| (a[k] + 2.0 * b[k]) / 3.0),
    ];
    let mut bits = 0u32;
    for (t, v) in c.iter().enumerate() {
        let best = (0..4)
            .min_by(|&i, &j| {
                let d = |p: &[f32; 3]| (0..3).map(|k| (p[k] - v[k]).powi(2)).sum::<f32>();
                d(&pal[i]).total_cmp(&d(&pal[j]))
            })
            .unwrap_or(0) as u32;
        bits |= best << (2 * t);
    }
    out[4..8].copy_from_slice(&bits.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_normals_turn_back() {
        // Flat: G = A = 128.
        let n = normal_of(&[0, 128, 0, 128], true);
        assert!(n[2] > 0.99, "{n:?}");
        // Leaning along x: g = a > 0.
        let n = normal_of(&[0, 191, 0, 191], true);
        assert!(n[0] > 0.3 && n[1].abs() < 0.02, "{n:?}");
        // Leaning along y: g > 0, a < 0.
        let n = normal_of(&[0, 191, 0, 64], true);
        assert!(n[1] > 0.3 && n[0].abs() < 0.02, "{n:?}");
    }

    #[test]
    fn blocks_round_trip() {
        let v: [u8; 16] = std::array::from_fn(|i| (i * 16) as u8);
        let b = bc4_block(&v);
        let mut d = [0u8; 16];
        bcdec_rs::bc4(&b, &mut d, 4, false);
        assert!(v.iter().zip(d).all(|(a, b)| (i32::from(*a) - i32::from(b)).abs() <= 18), "{d:?}");
        let px: [[u8; 4]; 16] = std::array::from_fn(|i| [(i * 16) as u8, 100, (255 - i * 16) as u8, 255]);
        let b = bc1_block(&px);
        let mut d = [0u8; 64];
        bcdec_rs::bc1(&b, &mut d, 16);
        for (i, p) in px.iter().enumerate() {
            for k in 0..3 {
                assert!((i32::from(p[k]) - i32::from(d[4 * i + k])).abs() <= 44, "texel {i}: {p:?} vs {:?}", &d[4 * i..4 * i + 4]);
            }
        }
    }
}
