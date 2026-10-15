//! How an IW7 material draws, read from its technique set's name, and its
//! colour picture with the right alpha.
//!
//! IW7's world and model technique sets name their parts in words, as
//! Ghosts' do: `w_` world, `wc_` world decal, `mo_`/`mopw_` model; `l_sm`
//! lit, `unlit`; the blend (`replace`, `replacex0`, `atest`, `blend`,
//! `blendadd`, `blendaddrgb`, `add`, `multiply`); `ndw` no depth write (a
//! decal or a glass pane over the world); then the layer words
//! (`i0c0s0o0n0p0`: colour, specular, occlusion, normal...). Read on
//! mp_flip's 655 materials.
//!
//! The colour maps of most IW7 materials are `*_packed_cs` pictures: colour
//! with the smoothness in alpha (0.04 on average on mp_flip), so the alpha
//! is no opacity. A material that cuts out (`atest`) or blends takes its
//! opacity from its alpha map (`*_packed_a`, `*_o`) instead, in the slot
//! whose name hashes to `ALPHA_MAP_HASH`.

use crate::map::{Iw7Map, MapMaterial};

/// The texture slot of a material's alpha (opacity) map.
pub const ALPHA_MAP_HASH: u32 = 0xa52c_26b4;

/// How a material draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Blend {
    Opaque,
    /// Alpha blended (decals, glass, unlit cards).
    Blend,
    Add,
    Multiply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Look {
    /// Not drawn: sky, tools, shadow casters, distortion, 3D hud.
    pub hidden: Option<&'static str>,
    pub blend: Blend,
    pub alpha_test: bool,
    pub unlit: bool,
    /// Painted over the world (no depth write): drawn a step toward the
    /// camera.
    pub decal: bool,
}

/// A technique set's look, by its words.
pub fn look(technique_set: &str) -> Look {
    let words: Vec<&str> = technique_set.trim_start_matches(',').split('_').collect();
    let has = |w: &str| words.contains(&w);
    let hidden = if words.iter().any(|w| w.starts_with("sky")) {
        Some("sky")
    } else if has("tools") {
        Some("tools")
    } else if has("shadowcaster") || has("onlycast") {
        Some("shadow only")
    } else if has("distortion") {
        Some("distortion")
    } else if has("threed") {
        Some("3d")
    } else {
        None
    };
    let unlit = words.iter().any(|w| w.starts_with("unlit"));
    let alpha_test = has("atest") || words.iter().any(|w| w.len() >= 4 && w.starts_with("t0c"));
    let blend = if has("add") {
        Blend::Add
    } else if has("multiply") {
        Blend::Multiply
    } else if has("blend") || has("blendadd") || has("blendaddrgb") || words.iter().any(|w| w.starts_with("b0c")) {
        Blend::Blend
    } else {
        Blend::Opaque
    };
    // (A `wc_` set is a world decal whatever its blend: mp_riot's
    // alpha-tested `wc_l_sm_flag_atest_*` grime and flags lie on the walls.)
    let decal = (has("ndw") && blend != Blend::Opaque) || words.first() == Some(&"wc");
    Look { hidden, blend, alpha_test: alpha_test && blend == Blend::Opaque, unlit, decal }
}

/// A picture as RGBA8, its mip chain biggest first.
#[derive(Clone, Debug)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub levels: Vec<Vec<u8>>,
}

/// Box-filters an RGBA8 level down by two (odd edges clamp).
pub(crate) fn half_level(w: u32, h: u32, px: &[u8]) -> (u32, u32, Vec<u8>) {
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

impl Iw7Map {
    /// A material's colour map as RGBA8 at most `max_dim`, its alpha from
    /// its alpha map when it has one (else the colour map's own when
    /// `keep_alpha`, else opaque), with a full mip chain.
    pub fn colour_rgba(&self, m: &MapMaterial, max_dim: u32, keep_alpha: bool) -> Result<Rgba, String> {
        self.layer_rgba(m, 0, max_dim, keep_alpha)
    }

    /// A layer's colour map (`colorMap`, `colorMap1`, `colorMap2`) as RGBA8
    /// with its opacity from the layer's alpha map (`layer_alpha_hash`), as
    /// `colour_rgba` does for the base.
    pub fn layer_rgba(&self, m: &MapMaterial, layer: usize, max_dim: u32, keep_alpha: bool) -> Result<Rgba, String> {
        let slot = if layer == 0 { "colorMap".to_owned() } else { format!("colorMap{layer}") };
        let name = m.texture(&slot).ok_or_else(|| format!("no {slot}"))?;
        let levels = self.image(name, max_dim)?;
        let (w, h, mut px) = crate::texture::level_rgba(&levels, 0).ok_or_else(|| format!("{name}: format {} not decoded", levels.format))?;
        let alpha_hash = layer_alpha_hash(layer);
        let alpha_map = m.textures.iter().find(|t| t.0 == alpha_hash).map(|t| t.3.clone());
        match alpha_map.as_deref().map(|a| (a, self.image(a, max_dim))) {
            Some((a, Ok(al))) => {
                let (aw, ah, apx) = crate::texture::level_rgba(&al, 0).ok_or_else(|| format!("{a}: format {} not decoded", al.format))?;
                // A one-channel picture decodes as grey (BC4, R8) or white
                // with alpha (A8): take the red unless only alpha varies.
                let use_alpha = al.format == 65;
                // A revealed layer (`v<k>` in its technique set's layer
                // word, its map a `*_packed_r` height): it shows where the
                // height clears the threshold (`revealParams`: softness,
                // threshold), not in proportion to it (mp_riot's street
                // debris covered the road as a white sheet).
                let reveal = layer > 0 && reveals(&m.technique_set, layer) && a.ends_with("_packed_r");
                let (soft, threshold) = m
                    .constants
                    .iter()
                    .find(|c| c.0 == "revealParams")
                    .map_or((0.1, 0.5), |c| (c.1[0].clamp(0.01, 1.0), c.1[1].clamp(0.0, 1.0)));
                let curve: [u8; 256] = std::array::from_fn(|v| {
                    if !reveal {
                        return v as u8;
                    }
                    let t = ((v as f32 / 255.0 - (threshold - soft)) / (2.0 * soft)).clamp(0.0, 1.0);
                    ((t * t * (3.0 - 2.0 * t)) * 255.0).round() as u8
                });
                for y in 0..h {
                    for x in 0..w {
                        let (sx, sy) = ((x * aw / w).min(aw - 1), (y * ah / h).min(ah - 1));
                        let s = ((sy * aw + sx) * 4) as usize;
                        px[((y * w + x) * 4 + 3) as usize] = curve[usize::from(if use_alpha { apx[s + 3] } else { apx[s] })];
                    }
                }
            }
            Some((_, Err(e))) => return Err(format!("alpha map: {e}")),
            None if !keep_alpha => {
                for p in px.chunks_exact_mut(4) {
                    p[3] = 255;
                }
            }
            None => {}
        }
        // The layer's tint (`colorTint`, `colorTint1`...: mp_proto's red
        // hull plates are a grey picture tinted red), in linear light.
        if let Some(t) = tint(m, layer) {
            apply_tint(&mut px, t);
        }
        let mut out = Rgba { width: w, height: h, levels: vec![px] };
        let (mut lw, mut lh) = (w, h);
        while lw > 1 || lh > 1 {
            let (nw, nh, next) = half_level(lw, lh, out.levels.last().unwrap());
            out.levels.push(next);
            (lw, lh) = (nw, nh);
        }
        Ok(out)
    }
}

/// The texture slot of layer `k`'s alpha map: the base's slot name with the
/// layer's digit (`ALPHA_MAP_HASH` for the base, 0x4ab0fd05 for layer 1,
/// as mp_riot's two-layer walls name their crack decals' alpha).
pub fn layer_alpha_hash(layer: usize) -> u32 {
    if layer == 0 {
        return ALPHA_MAP_HASH;
    }
    let digit = b'0' + (layer as u8 % 10);
    u32::from(digit | 0x20) ^ ALPHA_MAP_HASH.wrapping_mul(33)
}

/// Whether layer `k`'s word in a technique set (`b2c2s2ol2nl2v2p2`) names
/// a reveal map (`v2`).
pub fn reveals(technique_set: &str, layer: usize) -> bool {
    let digit = char::from(b'0' + (layer as u8 % 10));
    technique_set.split('_').any(|w| {
        let b = w.as_bytes();
        b.len() >= 3 && b[1] == digit as u8 && b[2] == b'c' && matches!(b[0], b'b' | b'm' | b't' | b'a') && w.contains(&format!("v{digit}"))
    })
}

/// A layer's tint (`colorTint` for the base, `colorTint<k>` for layer k),
/// when it is not white.
pub fn tint(m: &MapMaterial, layer: usize) -> Option<[f32; 3]> {
    let name = if layer == 0 { "colorTint".to_owned() } else { format!("colorTint{layer}") };
    let t = m.constants.iter().find(|c| c.0 == name)?.1;
    (t[..3].iter().all(|v| v.is_finite()) && t[..3].iter().any(|v| (v - 1.0).abs() > 0.01)).then(|| [t[0], t[1], t[2]])
}

/// RGBA8 pixels (sRGB colour) times a linear tint.
pub fn apply_tint(px: &mut [u8], t: [f32; 3]) {
    let lut: Vec<[u8; 256]> = (0..3)
        .map(|c| {
            std::array::from_fn(|v| {
                let lin = srgb_to_linear(v as f32 / 255.0) * t[c].max(0.0);
                (linear_to_srgb(lin).clamp(0.0, 1.0) * 255.0).round() as u8
            })
        })
        .collect();
    for p in px.chunks_exact_mut(4) {
        for c in 0..3 {
            p[c] = lut[c][usize::from(p[c])];
        }
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// Whether a colour map's alpha is its own opacity: a plain `_c` picture,
/// not a packed one (`*_packed_cs`: smoothness in alpha).
pub fn colour_alpha_is_opacity(colour_map: &str) -> bool {
    !colour_map.contains("_packed_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_by_words() {
        assert_eq!(look("w_l_sm_replace_i0c0s0o0n0p0").blend, Blend::Opaque);
        let a = look("w_l_sm_atest_i0c0s0o0n0pa0");
        assert!(a.alpha_test && a.blend == Blend::Opaque);
        let d = look("w_l_sm_ndw_blendaddrgb_i0c0s0a0t0_nop_trans");
        assert!(d.decal && d.blend == Blend::Blend);
        assert_eq!(look("w_unlit_add_lin_ct_ndw").blend, Blend::Add);
        assert!(look("w_unlit_add_lin_ct_ndw").unlit);
        assert_eq!(look("w_sky").hidden, Some("sky"));
        assert!(look("wc_l_sm_flag_atest_i0c0s0n0a0t0pa0_ssvsc_trans").decal);
        assert!(!look("w_l_sm_replace_i0c0s0n0p0").decal);
        assert!(reveals("w_l_sm_replace_r0c0s0o0nl0p0_b1cx1_b2c2s2ol2nl2v2p2", 2));
        assert!(!reveals("w_l_sm_replace_r0c0s0o0nl0p0_b1cx1_b2c2s2ol2nl2v2p2", 1));
        assert_eq!(look("wc_distortion_scale_ua").hidden, Some("distortion"));
        assert_eq!(look("w_unlit_multiply_lin_ndw").blend, Blend::Multiply);
        assert_eq!(crate::texture::name_hash("colorMap"), 0xa0ab_1041);
        assert_eq!(layer_alpha_hash(1), 0x4ab0_fd05);
        assert_eq!(layer_alpha_hash(0), ALPHA_MAP_HASH);
    }
}
