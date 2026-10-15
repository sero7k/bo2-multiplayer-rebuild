//! A material's own light: its emissive map (`emissiveMap`, `*_e`, `*_x`)
//! times `emissiveParams.x`, in linear HDR light, as IW7's lit model
//! shaders add it after the lighting (`i0c0s0n0e0`, `i0c0ec0`: Spaceland's
//! neon tubes, lit signs, arcade panels, wall lamps).
//!
//! The emissive map is a colour picture (sRGB); IW7 adds `emissive *
//! emissiveParams.x` to the lit colour before the tone map, so a neon
//! tube (`emissiveParams` 10) outshines every lit surface by two orders of
//! magnitude and blooms white at its core.

use crate::map::{Iw7Map, MapMaterial};

/// A material's emissive map and its scale, when it has one that glows.
pub fn emissive(m: &MapMaterial) -> Option<(&str, f32)> {
    let name = m.texture("emissiveMap")?;
    if name.starts_with('$') {
        return None;
    }
    let scale = m.constants.iter().find(|c| c.0.starts_with("emissivePara")).map_or(1.0, |c| c.1[0]);
    (scale.is_finite() && scale > 0.0).then_some((name, scale))
}

/// The glow as linear HDR light: (width, height, texels).
#[derive(Clone, Debug)]
pub struct Glow {
    pub width: u32,
    pub height: u32,
    pub light: Vec<[f32; 3]>,
}

impl Glow {
    /// The brightest texel's brightest channel.
    pub fn peak(&self) -> f32 {
        self.light.iter().flatten().copied().fold(0.0, f32::max)
    }
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

impl Iw7Map {
    /// A material's glow (`emissive`) at most `max_dim`, linear HDR.
    pub fn glow(&self, m: &MapMaterial, max_dim: u32) -> Result<Glow, String> {
        let (name, scale) = emissive(m).ok_or("no emissive map")?;
        let levels = self.image(name, max_dim)?;
        let (w, h, px) = crate::texture::level_rgba(&levels, 0).ok_or_else(|| format!("{name}: format {} not decoded", levels.format))?;
        let lut: [f32; 256] = std::array::from_fn(|v| srgb_to_linear(v as f32 / 255.0) * scale);
        let light = px.chunks_exact(4).map(|p| [lut[usize::from(p[0])], lut[usize::from(p[1])], lut[usize::from(p[2])]]).collect();
        Ok(Glow { width: w, height: h, light })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn emissive_by_slot_and_scale() {
        let h = crate::texture::name_hash("emissiveMap");
        let m = crate::map::MapMaterial {
            textures: vec![(h, 0, 0, "zmb_neon_tubes_01_blue_e".to_owned())],
            constants: vec![("emissivePara".to_owned(), [10.0, 0.0, 0.0, 0.0])],
            ..Default::default()
        };
        assert_eq!(super::emissive(&m), Some(("zmb_neon_tubes_01_blue_e", 10.0)));
        let none = crate::map::MapMaterial { textures: vec![(h, 0, 0, "$black".to_owned())], ..Default::default() };
        assert_eq!(super::emissive(&none), None);
    }
}
