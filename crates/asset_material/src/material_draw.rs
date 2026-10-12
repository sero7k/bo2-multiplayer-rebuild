#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialDrawMode {
    Opaque,

    AlphaTest { ge_half: bool },
    Blend,

    Multiply,
    Additive,
    Screen,
}

impl MaterialDrawMode {
    pub fn from_state_bits(load_bits: [u32; 2]) -> Self {
        let word0 = load_bits[0];
        let alpha_test_disabled = (word0 >> 11) & 1 != 0;
        let alpha_test = (word0 >> 12) & 0b11;
        let blend_op = (word0 >> 8) & 0b111;
        let src_blend = word0 & 0xf;
        let dst_blend = (word0 >> 4) & 0xf;

        if blend_op != 0 {
            if blend_op == 1 && src_blend == 1 && dst_blend == 3 {
                return Self::Multiply;
            }
            if blend_op == 1 && src_blend == 2 && dst_blend == 2 {
                return Self::Additive;
            }
            if blend_op == 1 && src_blend == 10 && dst_blend == 2 {
                return Self::Screen;
            }
            return Self::Blend;
        }
        if !alpha_test_disabled {
            return Self::AlphaTest {
                ge_half: alpha_test == 3,
            };
        }
        Self::Opaque
    }

    pub fn alpha_cutoff(self) -> Option<f32> {
        match self {
            Self::AlphaTest { ge_half: true } => Some(0.5),
            Self::AlphaTest { ge_half: false } => Some(0.01),
            _ => None,
        }
    }
}

pub fn alpha_test_cutoff_from_state_bits(load_bits: [u32; 2]) -> Option<f32> {
    let word0 = load_bits[0];
    let alpha_test_disabled = (word0 >> 11) & 1 != 0;
    if alpha_test_disabled {
        return None;
    }
    let alpha_test = (word0 >> 12) & 0b11;
    Some(if alpha_test == 3 { 0.5 } else { 0.01 })
}

pub const GFXS0_SRGBWRITEENABLE: u32 = 0x4000_0000;

#[inline]
pub fn srgb_write_enable_from_state_bits(load_bits: [u32; 2]) -> bool {
    (load_bits[0] & GFXS0_SRGBWRITEENABLE) != 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMapTransform {
    Square,

    Unknown,
}

pub use asset_iw4::{
    AlphaTest, D3DCULL_CCW, D3DCULL_CW, D3DCULL_NONE, D3DRS_CULLMODE, GFXS0_ATEST_DISABLE,
    GFXS0_ATEST_GE_128, GFXS0_ATEST_GT_0, GFXS0_ATEST_LT_128, GFXS0_ATEST_MASK, GFXS0_ATEST_SHIFT,
    GFXS0_CULL_BACK, GFXS0_CULL_FRONT, GFXS0_CULL_MASK, GFXS0_CULL_NONE, GFXS0_CULL_SHIFT,
    Gfxs0AlphaTest, Gfxs0CullFace as MaterialCullFace, S_ALPHA_TEST_TABLE, S_CULL_TABLE,
    alpha_test_from_state_bits, cull_face_from_state_bits, d3d_cull_mode_from_state_bits,
};

pub fn material_constant_name(name: &[u8; 12]) -> &str {
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    std::str::from_utf8(&name[..end]).unwrap_or("")
}

pub fn material_alpha_test(namespace: crate::AssetNamespace, bits: [u32; 2]) -> Option<AlphaTest> {
    match namespace {
        crate::AssetNamespace::T5 => fastfile_t5::state_bits::alpha_test(bits[0])
            .map(|(func, reference)| AlphaTest::from_raw(func, reference)),
        crate::AssetNamespace::Iw4 | crate::AssetNamespace::Iw5 => {
            alpha_test_from_state_bits(bits).map(Gfxs0AlphaTest::d3d)
        }
        // bo2zm: Black Ops II state bits are not decoded yet.
        crate::AssetNamespace::T6 | crate::AssetNamespace::Iw6 => None,
    }
}

pub use fastfile_t5::state_bits::smodel_camera_emits as t5_smodel_camera_emits;

/// Why an IW4/IW5 material never draws as an opaque colour surface: what the
/// renderer's fallback (and the IW5 world build) must leave out instead of
/// drawing it solid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IwNoOpaqueDraw {
    /// A screen distortion (`distortion_*` technique set, heat haze, refracting
    /// glass): refracts the resolved frame in a pass of its own.
    Distortion,
    /// A particle cloud (`particle_cloud*`): each vertex is a point the engine
    /// expands into a camera-facing quad; drawn as triangles it is stray
    /// polygons in the air.
    ParticleCloud,
    /// Casts shadow only (`*shadowcaster*` technique set, `caulk_shadow`).
    ShadowCaster,
    /// Every colour technique blends (alpha blend, add, multiply, screen):
    /// decals, light cards, embers, caustics. Drawn opaque it is a solid
    /// quad with the texture's black (or white) background.
    Blended,
    /// No colour technique writes colour.
    NoColourWrite,
}

impl IwNoOpaqueDraw {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Distortion => "distortion",
            Self::ParticleCloud => "particle_cloud",
            Self::ShadowCaster => "shadow_caster",
            Self::Blended => "blended",
            Self::NoColourWrite => "no_colour_write",
        }
    }
}

/// The IW technique-set / material names that never draw as a colour
/// surface, whatever their state bits say.
pub fn iw_no_opaque_draw_by_name(material: &str, technique_set: &str) -> Option<IwNoOpaqueDraw> {
    let techset = technique_set.trim_start_matches(',').to_ascii_lowercase();
    let material = material.trim_start_matches(',').to_ascii_lowercase();
    let base = material.rsplit('/').next().unwrap_or(material.as_str());
    if techset.starts_with("distortion") || base.split('_').any(|t| t == "distortion") {
        return Some(IwNoOpaqueDraw::Distortion);
    }
    if techset.starts_with("particle_cloud") || techset.starts_with("particlecloud") {
        return Some(IwNoOpaqueDraw::ParticleCloud);
    }
    if techset.contains("shadowcaster") || base == "caulk_shadow" || base.contains("shadowcaster") {
        return Some(IwNoOpaqueDraw::ShadowCaster);
    }
    None
}

/// The IW4 technique slots that draw a surface's colour (unlit, emissive,
/// lit, lit instanced); not the depth, shadow-map or additive light passes.
pub fn iw4_colour_technique_slot(slot: usize) -> bool {
    crate::t5_tech_map::IW4_TECHNIQUE_TYPE_NAMES
        .get(slot)
        .is_some_and(|name| {
            *name == "unlit"
                || *name == "lit"
                || name.starts_with("emissive")
                || name.starts_with("lit ")
        })
}

/// How an IW4/IW5 material's colour techniques blend, from its own
/// `stateBitsEntry` (IW4 slot order; IW5 entries remapped) and
/// `stateBitsTable`: the first pass of every colour technique present.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IwColourPassBlend {
    pub opaque_rows: u8,
    pub blended_rows: u8,
    pub no_colour_rows: u8,
    pub first_blended: Option<MaterialDrawMode>,
}

impl IwColourPassBlend {
    pub fn from_state_bits(entries: &[u8], table: &[[u32; 2]]) -> Self {
        let mut out = Self::default();
        for (slot, &entry) in entries.iter().enumerate() {
            if entry == 0xff || !iw4_colour_technique_slot(slot) {
                continue;
            }
            let Some(&bits) = table.get(usize::from(entry)) else {
                continue;
            };
            // GFXS0_COLORWRITE_RGB (bit 27).
            if bits[0] & 0x0800_0000 == 0 {
                out.no_colour_rows = out.no_colour_rows.saturating_add(1);
                continue;
            }
            match MaterialDrawMode::from_state_bits(bits) {
                MaterialDrawMode::Opaque | MaterialDrawMode::AlphaTest { .. } => {
                    out.opaque_rows = out.opaque_rows.saturating_add(1);
                }
                mode => {
                    out.blended_rows = out.blended_rows.saturating_add(1);
                    out.first_blended.get_or_insert(mode);
                }
            }
        }
        out
    }

    /// The material never draws opaque: some colour technique blends and
    /// none is opaque, or none writes colour at all.
    pub fn no_opaque_draw(self) -> Option<IwNoOpaqueDraw> {
        if self.opaque_rows > 0 {
            return None;
        }
        if self.blended_rows > 0 {
            return Some(IwNoOpaqueDraw::Blended);
        }
        (self.no_colour_rows > 0).then_some(IwNoOpaqueDraw::NoColourWrite)
    }
}

/// An IW4/IW5 material that must not be drawn as a solid surface: by name
/// (distortion, particle cloud, shadow caster), else by its colour
/// techniques' state bits (blended, or no colour write).
pub fn iw_no_opaque_draw(
    material: &str,
    technique_set: &str,
    entries: Option<&[u8]>,
    table: &[[u32; 2]],
) -> Option<IwNoOpaqueDraw> {
    iw_no_opaque_draw_by_name(material, technique_set).or_else(|| {
        entries.and_then(|entries| IwColourPassBlend::from_state_bits(entries, table).no_opaque_draw())
    })
}

#[cfg(test)]
mod iw_no_opaque_draw_tests {
    use super::*;

    // GFXS0 words: colour write RGB|A, alpha test disabled.
    const WRITE: u32 = 0x1800_0000 | 0x800;
    const OPAQUE: [u32; 2] = [WRITE, 0];
    // srcblend SRCALPHA (5), dstblend INVSRCALPHA (6), blendop ADD (1).
    const ALPHA_BLEND: [u32; 2] = [WRITE | 0x5 | (0x6 << 4) | (1 << 8), 0];
    // srcblend ZERO (1), dstblend SRCCOLOR (3): multiply.
    const MULTIPLY: [u32; 2] = [WRITE | 0x1 | (0x3 << 4) | (1 << 8), 0];
    // srcblend ONE (2), dstblend ONE (2): additive (the light passes too).
    const ADD: [u32; 2] = [WRITE | 0x2 | (0x2 << 4) | (1 << 8), 0];

    fn entries(rows: &[(usize, u8)]) -> [u8; 48] {
        let mut out = [0xffu8; 48];
        for &(slot, row) in rows {
            out[slot] = row;
        }
        out
    }

    #[test]
    fn colour_slots_skip_depth_shadow_and_light_passes() {
        assert!(!iw4_colour_technique_slot(0)); // depth prepass
        assert!(!iw4_colour_technique_slot(3)); // build shadowmap color
        assert!(iw4_colour_technique_slot(4)); // unlit
        assert!(iw4_colour_technique_slot(5)); // emissive
        assert!(iw4_colour_technique_slot(13)); // lit sun shadow
        assert!(iw4_colour_technique_slot(36)); // lit instanced omni shadow dfog
        assert!(!iw4_colour_technique_slot(37)); // light spot (additive dlight)
        assert!(!iw4_colour_technique_slot(38)); // light omni
    }

    #[test]
    fn decal_that_only_blends_is_not_drawn_opaque() {
        // A multiply decal: lit sun shadow multiplies; its depth prepass
        // row (opaque, slot 0) and the additive light pass do not count.
        let table = [OPAQUE, MULTIPLY, ADD];
        let e = entries(&[(0, 0), (13, 1), (37, 2)]);
        let blend = IwColourPassBlend::from_state_bits(&e, &table);
        assert_eq!(blend.no_opaque_draw(), Some(IwNoOpaqueDraw::Blended));
        assert_eq!(blend.first_blended, Some(MaterialDrawMode::Multiply));
        let e = entries(&[(4, 0), (5, 1)]);
        let table = [ALPHA_BLEND, ADD];
        assert_eq!(
            IwColourPassBlend::from_state_bits(&e, &table).no_opaque_draw(),
            Some(IwNoOpaqueDraw::Blended)
        );
    }

    #[test]
    fn opaque_wall_with_additive_light_pass_still_draws() {
        let table = [OPAQUE, ADD];
        let e = entries(&[(13, 0), (9, 0), (38, 1)]);
        assert_eq!(IwColourPassBlend::from_state_bits(&e, &table).no_opaque_draw(), None);
    }

    #[test]
    fn names_that_never_draw_solid() {
        assert_eq!(
            iw_no_opaque_draw_by_name("wc/hill_glass_mosaic_distortion", "wc_l_sm_r0c0"),
            Some(IwNoOpaqueDraw::Distortion)
        );
        assert_eq!(
            iw_no_opaque_draw_by_name("wc/heat", ",distortion_scale_zfeather"),
            Some(IwNoOpaqueDraw::Distortion)
        );
        assert_eq!(
            iw_no_opaque_draw_by_name("wc/hjk_embers", "particle_cloud_add"),
            Some(IwNoOpaqueDraw::ParticleCloud)
        );
        assert_eq!(
            iw_no_opaque_draw_by_name("wc/caulk_shadow", "wc_shadowcaster"),
            Some(IwNoOpaqueDraw::ShadowCaster)
        );
        assert_eq!(iw_no_opaque_draw_by_name("wc/me_concrete_floor", "wc_l_sm_r0c0n0s0"), None);
        // "distortion" must be a whole token of the material's base name.
        assert_eq!(iw_no_opaque_draw_by_name("wc/nodistortionhere", "wc_l_sm_r0c0"), None);
    }

    #[test]
    fn no_state_bits_falls_back_to_names_only() {
        assert_eq!(iw_no_opaque_draw("wc/floor", "wc_l_sm_r0c0", None, &[]), None);
        let table = [ALPHA_BLEND];
        let e = entries(&[(13, 0)]);
        assert_eq!(
            iw_no_opaque_draw("wc/me_decal_darkstain_01", "wc_l_sm_b0c0", Some(&e), &table),
            Some(IwNoOpaqueDraw::Blended)
        );
    }
}
